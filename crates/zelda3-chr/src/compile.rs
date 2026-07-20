//! Compile decoded sheets into the packed `kSprGfx`/`kBgGfx` container payloads.
//!
//! Byte-identical-by-construction rule: for each donor container item, decode the
//! donor bytes to CHR indices and compare against the sheet's indices for that
//! pack. If they match (or no sheet covers the pack), emit the donor item bytes
//! verbatim — this makes unedited trees reproduce the donor bins exactly, since
//! the container codec round-trips. Only edited items are re-encoded:
//!   * sprite items with index < 103 whose planar encoding is exactly 0x600 bytes
//!     are emitted raw (the engine reads len==0x600 sprite items as raw planar);
//!   * every other edited item (sprite >= 103, non-0x600 sprite, all bg) is
//!     emitted as a literal-compressed stream.

use crate::compress::{compress_literal, decompress_asset};
use crate::container::{pack_arrays, unpack_packed_arrays};
use crate::planar::{decode_planar_tile_indices, encode_planar_tiles};
use crate::sidecar::DecodedSheet;
use std::collections::HashMap;

/// Number of leading `kSprGfx` items stored uncompressed (raw planar) in the
/// donor container (matches `read_decoded_chr_packs`'s `uncompressed_prefix_count`).
const SPR_UNCOMPRESSED_PREFIX: usize = 12;
/// Sprite pack indices below this are read raw by the engine when the item is
/// exactly 0x600 bytes.
const SPR_RAW_MAX_INDEX: usize = 103;
const SPR_RAW_LEN: usize = 0x600;

struct SheetBlockView<'a> {
    bpp: u8,
    tiles: &'a [[u8; 64]],
}

/// Build a `(kind, pack) -> block tiles` lookup from decoded sheets.
fn index_sheets(sheets: &[DecodedSheet]) -> HashMap<(&str, u32), SheetBlockView<'_>> {
    let mut by_key = HashMap::new();
    for sheet in sheets {
        for block in &sheet.blocks {
            by_key.insert(
                (block.source_kind.as_str(), block.source_pack),
                SheetBlockView {
                    bpp: block.source_bpp,
                    tiles: sheet.block_tiles(block),
                },
            );
        }
    }
    by_key
}

/// Rebuild one container's item list, item-by-item, applying the pass-through /
/// re-encode rule.
fn rebuild_items(
    kind: &str,
    donor_items: &[Vec<u8>],
    uncompressed_prefix: usize,
    by_key: &HashMap<(&str, u32), SheetBlockView<'_>>,
) -> Result<Vec<Vec<u8>>, String> {
    let mut out = Vec::with_capacity(donor_items.len());
    for (index, donor_item) in donor_items.iter().enumerate() {
        if index < uncompressed_prefix {
            // Sprite packs 0..11 are not art: the game "loads" them as
            // deterministic junk (their compressed streams read stale WRAM
            // decompression-buffer bytes on hardware), and the donor ships
            // the original compressed streams so the runtime can reproduce
            // that exactly. They are not editable through the CHR sheet
            // authority — always pass the donor bytes through verbatim.
            out.push(donor_item.clone());
            continue;
        }
        let Some(view) = by_key.get(&(kind, index as u32)) else {
            // No sheet covers this pack (e.g. bg pack 114): pass the donor through.
            out.push(donor_item.clone());
            continue;
        };

        // Mirror the runtime's rule (`decompressed_sprite_graphics_data`):
        // sprite donors of exactly 0x600 bytes are raw planar (e.g. items this
        // compiler previously re-encoded as raw); everything else is an
        // LC-LZ2 stream.
        let donor_raw = if kind == "sprite" && donor_item.len() == SPR_RAW_LEN {
            donor_item.clone()
        } else {
            decompress_asset(donor_item)
                .map_err(|err| format!("{kind} pack {index}: donor decode failed: {err}"))?
        };
        let donor_tiles = decode_planar_tile_indices(&donor_raw, view.bpp)
            .map_err(|err| format!("{kind} pack {index}: donor planar decode failed: {err}"))?;

        if donor_tiles == view.tiles {
            // Unedited: emit the donor bytes verbatim for byte-identical output.
            out.push(donor_item.clone());
            continue;
        }

        // Edited: re-encode from the sheet's CHR indices.
        let planar = encode_planar_tiles(view.tiles, view.bpp)
            .map_err(|err| format!("{kind} pack {index}: planar encode failed: {err}"))?;
        let raw_eligible =
            kind == "sprite" && index < SPR_RAW_MAX_INDEX && planar.len() == SPR_RAW_LEN;
        if raw_eligible {
            out.push(planar);
        } else {
            out.push(compress_literal(&planar));
        }
    }
    Ok(out)
}

/// Compile decoded sheets into the `(kSprGfx, kBgGfx)` container payloads, using
/// the donor bins for verbatim pass-through of unedited packs.
pub fn compile_chr_packs(
    sheets: &[DecodedSheet],
    donor_spr_bin: &[u8],
    donor_bg_bin: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let spr_items = unpack_packed_arrays(donor_spr_bin)
        .map_err(|err| format!("kSprGfx donor unpack failed: {err}"))?;
    let bg_items = unpack_packed_arrays(donor_bg_bin)
        .map_err(|err| format!("kBgGfx donor unpack failed: {err}"))?;
    let by_key = index_sheets(sheets);

    let spr_out = rebuild_items("sprite", &spr_items, SPR_UNCOMPRESSED_PREFIX, &by_key)?;
    let bg_out = rebuild_items("bg", &bg_items, 0, &by_key)?;

    Ok((pack_arrays(&spr_out), pack_arrays(&bg_out)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidecar::SidecarBlock;

    fn tile(fill: u8) -> [u8; 64] {
        [fill & 0x7; 64]
    }

    fn block(
        name: &str,
        kind: &str,
        pack: u32,
        bpp: u8,
        start: usize,
        count: usize,
    ) -> SidecarBlock {
        SidecarBlock {
            block: name.to_string(),
            source_kind: kind.to_string(),
            source_pack: pack,
            source_bpp: bpp,
            tile_start: start,
            tile_count: count,
            tile_palette_rows: vec![0; count],
        }
    }

    // A synthetic donor: sprite packs 0..11 are junk passthrough streams,
    // packs 12 and 13 compressed art; bg pack 0 compressed, bg pack 1
    // uncovered (pass-through).
    fn synthetic() -> (Vec<DecodedSheet>, Vec<u8>, Vec<u8>) {
        let spr0: Vec<[u8; 64]> = (0..4).map(|i| tile(i as u8)).collect();
        let spr1: Vec<[u8; 64]> = (0..4).map(|i| tile((i + 1) as u8)).collect();
        let bg0: Vec<[u8; 64]> = (0..4).map(|i| tile((i + 2) as u8)).collect();
        let bg1: Vec<[u8; 64]> = (0..2).map(|i| tile((i + 3) as u8)).collect();

        // Donor sprite container: 12 junk streams (verbatim passthrough, never
        // decoded), then compressed art packs at indices 12 and 13.
        let mut spr_items: Vec<Vec<u8>> = (0..SPR_UNCOMPRESSED_PREFIX)
            .map(|i| vec![i as u8, 0xFF])
            .collect();
        spr_items.push(compress_literal(&encode_planar_tiles(&spr0, 3).unwrap()));
        spr_items.push(compress_literal(&encode_planar_tiles(&spr1, 3).unwrap()));
        let bg_items = vec![
            compress_literal(&encode_planar_tiles(&bg0, 2).unwrap()),
            compress_literal(&encode_planar_tiles(&bg1, 2).unwrap()),
        ];
        let donor_spr = pack_arrays(&spr_items);
        let donor_bg = pack_arrays(&bg_items);

        // Two sheets: one carries sprite pack 12 + bg pack 0, one carries
        // sprite pack 13. bg pack 1 is intentionally uncovered.
        let sheet_a_tiles: Vec<[u8; 64]> = spr0.iter().chain(bg0.iter()).copied().collect();
        let sheet_a = DecodedSheet {
            name: "a".to_string(),
            tiles: sheet_a_tiles,
            blocks: vec![
                block("a.DAT1", "sprite", 12, 3, 0, 4),
                block("a.DAT2", "bg", 0, 2, 4, 4),
            ],
        };
        let sheet_b = DecodedSheet {
            name: "b".to_string(),
            tiles: spr1.clone(),
            blocks: vec![block("b.DAT1", "sprite", 13, 3, 0, 4)],
        };
        (vec![sheet_a, sheet_b], donor_spr, donor_bg)
    }

    #[test]
    fn unedited_sheets_reproduce_donor_byte_for_byte() {
        let (sheets, donor_spr, donor_bg) = synthetic();
        let (spr, bg) = compile_chr_packs(&sheets, &donor_spr, &donor_bg).unwrap();
        assert_eq!(
            spr, donor_spr,
            "unedited sprite pack must be byte-identical"
        );
        assert_eq!(bg, donor_bg, "unedited bg pack must be byte-identical");
    }

    #[test]
    fn single_edit_touches_only_its_item_and_re_decodes() {
        let (mut sheets, donor_spr, donor_bg) = synthetic();
        // Edit one tile of sprite pack 13 (sheet b, block b.DAT1).
        sheets[1].tiles[2][0] = 5;
        let (spr, _bg) = compile_chr_packs(&sheets, &donor_spr, &donor_bg).unwrap();

        let donor_items = unpack_packed_arrays(&donor_spr).unwrap();
        let out_items = unpack_packed_arrays(&spr).unwrap();
        assert_eq!(
            out_items[12], donor_items[12],
            "untouched item stays verbatim"
        );
        assert_ne!(out_items[13], donor_items[13], "edited item must change");

        // Sprite pack 13 has index 13 (< 103) and 4 tiles * 24 bytes = 96
        // bytes, not 0x600, so it is re-encoded as a literal-compressed stream.
        let decoded = decompress_asset(&out_items[13]).unwrap();
        let tiles = decode_planar_tile_indices(&decoded, 3).unwrap();
        assert_eq!(
            tiles, sheets[1].tiles,
            "edited item re-decodes to sheet indices"
        );
    }

    #[test]
    fn junk_prefix_items_pass_through_verbatim() {
        // Sprite packs 0..11 are non-art junk streams: they must never be
        // decoded or re-encoded, even if a sheet claims to cover them.
        let (mut sheets, donor_spr, donor_bg) = synthetic();
        sheets[0].blocks[0] = block("a.DAT1", "sprite", 0, 3, 0, 4);
        let (spr, _bg) = compile_chr_packs(&sheets, &donor_spr, &donor_bg).unwrap();
        let donor_items = unpack_packed_arrays(&donor_spr).unwrap();
        let out_items = unpack_packed_arrays(&spr).unwrap();
        for index in 0..SPR_UNCOMPRESSED_PREFIX {
            assert_eq!(
                out_items[index], donor_items[index],
                "junk pack {index} must pass through verbatim"
            );
        }
    }

    #[test]
    fn edited_raw_sprite_stays_raw_when_0x600() {
        // Sprite pack 12 with 64 3bpp tiles encodes to exactly 0x600 bytes.
        let spr12: Vec<[u8; 64]> = (0..64).map(|i| tile(i as u8)).collect();
        let mut spr_items: Vec<Vec<u8>> = (0..SPR_UNCOMPRESSED_PREFIX)
            .map(|i| vec![i as u8, 0xFF])
            .collect();
        spr_items.push(encode_planar_tiles(&spr12, 3).unwrap());
        let donor_spr = pack_arrays(&spr_items);
        let donor_bg = pack_arrays(&[vec![0xFFu8]]); // trivial bg (uncovered)
        let mut sheet = DecodedSheet {
            name: "a".to_string(),
            tiles: spr12.clone(),
            blocks: vec![block("a.DAT1", "sprite", 12, 3, 0, 64)],
        };
        sheet.tiles[0][0] = 4; // edit
        let (spr, _bg) = compile_chr_packs(&[sheet.clone()], &donor_spr, &donor_bg).unwrap();
        let items = unpack_packed_arrays(&spr).unwrap();
        assert_eq!(
            items[12].len(),
            SPR_RAW_LEN,
            "edited raw-eligible item stays 0x600 raw"
        );
        let tiles = decode_planar_tile_indices(&items[12], 3).unwrap();
        assert_eq!(tiles, sheet.tiles);
    }
}
