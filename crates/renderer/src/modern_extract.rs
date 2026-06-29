use crate::gpu_frame::GpuFrame;
use crate::modern_assets::{atlas_entry_for_tilemap_entry, ModernTileAtlasAsset};
use crate::modern_frame::{
    ModernFrame, ModernIndexSpriteInstance, ModernIndexTileInstance, ModernTileInstance,
};
use crate::modern_index_atlas::{index_cell_for_tilemap_entry, ModernIndexAtlas, ModernIndexTile};
use crate::modern_sprite_atlas::{sprite_index_cell, ModernSpriteIndexAtlas};

// Per SNES PPU OBSEL: maps obj_size (3-bit index) to [small_px, large_px].
// Copied from sprite_renderer::SPRITE_SIZES so the modern OAM enumeration matches
// the classic resolver's size selection exactly.
const SPRITE_SIZES: [[u8; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

/// Decode OAM into palette-index sprite-tile instances, mirroring the per-sprite,
/// per-8×8-tile ENUMERATION of `sprite_renderer::resolve_obj_pixels` (NOT its
/// per-pixel resolver).
///
/// For each of the 128 OAM sprites: skip the off-screen sentinel (`y == 0xf0`),
/// derive size from `obj.obj_size` + the OAM hi-table, cull horizontally exactly
/// like the reference, then emit one `ModernIndexSpriteInstance` per 8×8 tile whose
/// UNFLIPPED pattern is present in `atlas` for `(context, effective_tile)`. The
/// instance carries the OAM palette/priority/flip; the renderer (Task 5) applies
/// `hflip`/`vflip` to the cell's 8×8 pixels.
pub fn extract_modern_sprites(
    frame: &GpuFrame<'_>,
    atlas: &ModernSpriteIndexAtlas,
    context: u64,
) -> Vec<ModernIndexSpriteInstance> {
    let oam = frame.oam;
    let obj = &frame.obj;
    let mut out = Vec::new();

    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);

        // Off-screen sentinel: the game parks hidden sprites at y == 0xf0.
        let y_byte = ((oam0 >> 8) & 0xff) as i32;
        if y_byte == 0xf0 {
            continue;
        }
        // On-screen top row = ((Y+1)&0xff) - 1 (= Y for Y < 0xff), matching the
        // reference's `yy = ((oam0>>8)+1)&0xff` then `out_y = line - 1`.
        let top_y = ((y_byte + 1) & 0xff) - 1;

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let size = SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;

        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        // extra_left_right = 0: replicate the reference's horizontal cull.
        if object_x > 256 && object_x + size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let palette = ((oam1 & 0x0e00) >> 9) as u8;
        let priority = ((oam1 & 0x3000) >> 12) as u8;
        let bank: u16 = if oam1 & 0x0100 != 0 { 256 } else { 0 };
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;

        let tiles_per_side = size / 8;
        for sty in 0..tiles_per_side {
            for stx in 0..tiles_per_side {
                // Source tile honoring flip at TILE granularity (matches the
                // reference's used_col = hflip ? size-1-col : col, taken >> 3).
                let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                let used_tile = (((tile_row_base + src_row_tile) << 4)
                    | ((tile_col_base + src_col_tile) & 0x0f))
                    as u16;
                let effective_tile = bank + used_tile;

                let Some(cell) = sprite_index_cell(atlas, context, effective_tile) else {
                    continue;
                };
                out.push(ModernIndexSpriteInstance {
                    cell_id: cell.id,
                    screen_x: (x + stx * 8) as i16,
                    screen_y: (top_y + sty * 8) as i16,
                    palette,
                    priority,
                    hflip,
                    vflip,
                });
            }
        }
    }

    out
}

/// Decode one SNES 4bpp 8×8 OBJ/BG tile from live VRAM into 64 palette indices
/// (row-major, 0–15). `chr_base_words` is the VRAM word base of the CHR page;
/// `tilemap_entry` carries the tile number in bits [9:0] and flip in bits 14/15.
///
/// Mirrors `decode_snes_4bpp_tile_indices` in the binary: each 8×8 tile occupies
/// 16 words (two bitplane words per row, planes 0+1 then 2+3), pixel x=0 is the
/// MSB. Call with NO flip bits to obtain the UNFLIPPED pattern (the renderer
/// applies hflip/vflip when sampling the cell).
pub fn decode_snes_4bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 16;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let w23 = vram.get(tile_base + 8 + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        let (bp2, bp3) = ((w23 & 0xff) as u8, (w23 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            out[y * 8 + x] = ((bp0 & bit != 0) as u8)
                | (((bp1 & bit != 0) as u8) << 1)
                | (((bp2 & bit != 0) as u8) << 2)
                | (((bp3 & bit != 0) as u8) << 3);
        }
    }
    out
}

/// Decode one SNES 2bpp 8×8 tile from live VRAM into 64 palette indices (0–3),
/// applying `tilemap_entry` flip. Each tile occupies 8 words (one word = two
/// bitplanes per row); pixel x=0 is the MSB. Used for BG3 in PPU mode 1 (the
/// dungeon HUD/message layer), which is 2bpp — decoding it as 4bpp reads adjacent
/// tile data as planes 2/3 and produces garbage indices.
pub fn decode_snes_2bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 8;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            out[y * 8 + x] = ((bp0 & bit != 0) as u8) | (((bp1 & bit != 0) as u8) << 1);
        }
    }
    out
}

/// Extract a dungeon `ModernFrame` whose BG tile patterns are decoded from LIVE
/// VRAM (not the static `dungeon_index_tiles` atlas), returning the decoded cell
/// set alongside.
///
/// The static dungeon atlas bakes each `(theme, tile#)` pattern from a snapshot of
/// CHR, but several dungeon BG tiles are ANIMATED — their CHR is re-DMA'd into VRAM
/// each frame (e.g. the animated floor tile #255, water/lava). A fixed atlas cannot
/// follow that, so those tiles render with a stale palette index. Because the
/// dungeon floor is composited as backdrop + BG1-subscreen color-math, a stale BG1
/// index shows up as a ~1-LSB-low blended floor versus the classic renderer (which
/// reads live VRAM). Decoding the BG CHR straight from `frame.vram` — exactly as
/// `extract_modern_sprites_from_vram` does for OBJ — makes BG byte-exact with the
/// classic PPU.
///
/// Only BG1/BG2 (4bpp) are decoded; BG3 (the 2bpp HUD/message layer in mode 1) is
/// left out because the modern compositor does not window it (drawing it over the
/// play field would diverge from the classic) — see the loop comment.
/// Geometry (64×64 four-quadrant tilemap, main||sub visibility, scroll) matches
/// [`extract_modern_frame_with_dungeon_atlas`]. Cells are deduplicated by
/// `(layer CHR base, tile# + flip, bpp)` and store the FLIP-BAKED 8×8 pattern (the
/// dungeon composite samples cells without re-applying flip). The backdrop is set
/// from CGRAM[0] (the classic main color-math operand for backdrop pixels).
pub fn extract_modern_dungeon_frame_from_vram(
    frame: &GpuFrame<'_>,
) -> (ModernFrame, Vec<ModernIndexTile>) {
    use std::collections::HashMap;
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    modern.backdrop_color_rgba =
        crate::modern_palette::snes_cgram_to_rgba(*frame.cgram.first().unwrap_or(&0));

    let mut cells: Vec<ModernIndexTile> = Vec::new();
    // key: (CHR word base, tilemap word masked to tile#+flip) -> cell id
    let mut cell_ids: HashMap<(usize, u16), u32> = HashMap::new();

    // BG1 (floor/statues, the subscreen color-math operand) and BG2 (walls) are
    // decoded; BG3 (the 2bpp HUD/message layer in mode 1) is intentionally left out
    // — the modern compositor does not window it, so drawing it over the play field
    // would diverge from the classic. The prior static atlas drew ~no BG3 tiles too.
    for layer_index in 0..2usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        let enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        let enabled = enabled_main || enabled_sub;
        modern.bg_layers[layer_index].enabled_main = enabled;
        modern.bg_layers[layer_index].enabled_sub = enabled_sub;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let chr_base = frame.bg[layer_index].tile_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        let wide = frame.bg[layer_index].tilemap_wider;
        let tall = frame.bg[layer_index].tilemap_higher;
        let cols = if wide { 64usize } else { 32 };
        let rows = if tall { 64usize } else { 32 };
        for ty in 0..rows {
            for tx in 0..cols {
                let q = (if wide && tx >= 32 { 1 } else { 0 })
                    + (if tall && ty >= 32 {
                        if wide {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    });
                let within = (ty % 32) * 32 + (tx % 32);
                let addr = base + q * 0x400 + within;
                let entry_word = *frame.vram.get(addr).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                // tile number (bits 0-9) + flip (bits 14/15); palette/priority dropped.
                let pattern_key = entry_word & 0xC3FF;
                let cell_id = *cell_ids.entry((chr_base, pattern_key)).or_insert_with(|| {
                    // Bake flip into the cell: the dungeon composite samples without flip.
                    let indices = decode_snes_4bpp_tile_indices(frame.vram, chr_base, pattern_key);
                    let id = cells.len() as u32;
                    cells.push(ModernIndexTile { id, indices });
                    id
                });
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id,
                        screen_x: (tx * 8) as i16 - h_scroll as i16,
                        screen_y: (ty * 8) as i16 - v_scroll as i16,
                        palette: ((entry_word >> 10) & 7) as u8,
                        hflip: false,
                        vflip: false,
                    });
            }
        }
    }
    (modern, cells)
}

/// Decode OAM into palette-index sprite-tile instances AND their UNIQUE 8×8 tile
/// patterns decoded from LIVE VRAM, mirroring the per-sprite, per-8×8-tile
/// ENUMERATION of `extract_modern_sprites` (same culling, sizes, flip, bank, and
/// tile addressing).
///
/// Unlike the static-atlas variant, this resolves each sprite tile's pixels from
/// the current frame's sprite CHR (`frame.obj.tile_adr1` / `tile_adr2`, selected
/// by the OAM bank bit) in `frame.vram`. Sprite CHR is DMA'd per frame, so this is
/// the only way modern rendering can follow animations (Link's poses, dynamic
/// sprites) — a fixed `(context, tile)` atlas cannot.
///
/// Returns `(cells, instances)`: `cells` is the deduplicated set of UNFLIPPED 8×8
/// patterns (sequential ids, ready for `draw_modern_sprites_indexed`); each
/// instance references its pattern by `cell_id` and carries the OAM
/// palette/priority/flip and on-screen position. Fully transparent (all-zero)
/// patterns are skipped.
pub fn extract_modern_sprites_from_vram(
    frame: &GpuFrame<'_>,
) -> (Vec<ModernIndexTile>, Vec<ModernIndexSpriteInstance>) {
    use std::collections::HashMap;

    let oam = frame.oam;
    let obj = &frame.obj;
    let mut cells: Vec<ModernIndexTile> = Vec::new();
    let mut pattern_ids: HashMap<[u8; 64], u32> = HashMap::new();
    let mut out = Vec::new();

    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);

        // Off-screen sentinel: the game parks hidden sprites at y == 0xf0.
        let y_byte = ((oam0 >> 8) & 0xff) as i32;
        if y_byte == 0xf0 {
            continue;
        }
        let top_y = ((y_byte + 1) & 0xff) - 1;

        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let size = SPRITE_SIZES[(obj.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize] as i32;

        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        if object_x > 256 && object_x + size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let palette = ((oam1 & 0x0e00) >> 9) as u8;
        let priority = ((oam1 & 0x3000) >> 12) as u8;
        // Decode from the correct CHR page directly (the OAM bank bit selects the
        // word base), so `used_tile` stays 0..255 within that page.
        let bank_base = if oam1 & 0x0100 != 0 {
            obj.tile_adr2
        } else {
            obj.tile_adr1
        };
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;

        let tiles_per_side = size / 8;
        for sty in 0..tiles_per_side {
            for stx in 0..tiles_per_side {
                let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                let used_tile = (((tile_row_base + src_row_tile) << 4)
                    | ((tile_col_base + src_col_tile) & 0x0f))
                    as u16;

                // Decode the UNFLIPPED 8×8 pattern (no flip bits): the renderer
                // applies hflip/vflip per instance when sampling the cell.
                let indices =
                    decode_snes_4bpp_tile_indices(frame.vram, bank_base as usize, used_tile);

                // Skip fully transparent tiles to avoid cluttering the cell set.
                if indices.iter().all(|&i| i == 0) {
                    continue;
                }

                let cell_id = *pattern_ids.entry(indices).or_insert_with(|| {
                    let id = cells.len() as u32;
                    cells.push(ModernIndexTile { id, indices });
                    id
                });

                out.push(ModernIndexSpriteInstance {
                    cell_id,
                    screen_x: (x + stx * 8) as i16,
                    screen_y: (top_y + sty * 8) as i16,
                    palette,
                    priority,
                    hflip,
                    vflip,
                });
            }
        }
    }

    (cells, out)
}

/// Decoded visual fields from a single SNES BG tilemap entry (u16).
///
/// Bits [9:0]  → tile_number
/// Bits [12:10] → palette
/// Bit  13     → priority
/// Bit  14     → hflip
/// Bit  15     → vflip
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileFields {
    pub tile_number: u16,
    pub palette: u8,
    pub priority: bool,
    pub hflip: bool,
    pub vflip: bool,
}

/// Decode one SNES BG tilemap entry word into its visual sub-fields.
pub fn decode_snes_tilemap_entry(entry: u16) -> ModernTileFields {
    ModernTileFields {
        tile_number: entry & 0x03ff,
        palette: ((entry >> 10) & 0x07) as u8,
        priority: entry & 0x2000 != 0,
        hflip: entry & 0x4000 != 0,
        vflip: entry & 0x8000 != 0,
    }
}

/// Extract frame-level visual state and BG tile instances from a `GpuFrame` into a `ModernFrame`,
/// mapping each tilemap entry to its atlas tile via `atlas_entry_for_tilemap_entry`.
///
/// Loops layers 0..3, gates on the main-screen enable bit, reads VRAM tilemap entries,
/// skips zeroes, looks up the atlas entry, and pushes a `ModernTileInstance` for each hit.
/// Screen position is `col*8 - h_scroll` / `row*8 - v_scroll`.
pub fn extract_modern_frame_with_atlas(
    frame: &GpuFrame<'_>,
    atlas: &ModernTileAtlasAsset,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].enabled_main = enabled_main;
        modern.bg_layers[layer_index].enabled_sub =
            frame.screen_enabled[1] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled_main {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        for row in 0..32usize {
            for col in 0..32usize {
                let entry_word = *frame.vram.get(base + row * 32 + col).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(atlas_entry) = atlas_entry_for_tilemap_entry(atlas, entry_word) else {
                    continue;
                };
                let fields = decode_snes_tilemap_entry(entry_word);
                let scale = atlas.atlas_scale.max(1);
                modern.bg_layers[layer_index]
                    .tiles
                    .push(ModernTileInstance {
                        atlas_id: atlas_entry.id,
                        atlas_x_px: atlas_entry.atlas_x_px,
                        atlas_y_px: atlas_entry.atlas_y_px,
                        atlas_width_px: atlas_entry.atlas_width_px,
                        atlas_height_px: atlas_entry.atlas_height_px,
                        screen_width_px: atlas_entry.atlas_width_px / scale,
                        screen_height_px: atlas_entry.atlas_height_px / scale,
                        screen_x: (col * 8) as i16 - frame.bg[layer_index].h_scroll as i16,
                        screen_y: (row * 8) as i16 - frame.bg[layer_index].v_scroll as i16,
                        palette: fields.palette,
                        priority: u8::from(fields.priority),
                        // atlas bakes the word's flip into the cell appearance; do not re-apply
                        // flip here — doing so would double-flip asymmetric tiles.
                        hflip: false,
                        vflip: false,
                        transparent_color_zero: true,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state and indexed BG tile instances from a `GpuFrame`
/// into a `ModernFrame`, using a palette-index atlas.
///
/// Sets `cgram_rgba` from `frame.cgram`. For layers 0..3 enabled on the main screen,
/// reads the 32×32 tilemap from `frame.bg[layer].tilemap_adr`, looks up each nonzero
/// word in the index atlas (by `word & 0xC3FF`), and pushes a `ModernIndexTileInstance`
/// with the per-word palette, screen position, and `hflip/vflip` fixed false (the atlas
/// index pattern already bakes flip).
pub fn extract_modern_frame_with_index_atlas(
    frame: &GpuFrame<'_>,
    atlas: &ModernIndexAtlas,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].enabled_main = enabled_main;
        modern.bg_layers[layer_index].enabled_sub =
            frame.screen_enabled[1] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled_main {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        for row in 0..32usize {
            for col in 0..32usize {
                let entry_word = *frame.vram.get(base + row * 32 + col).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(cell) = index_cell_for_tilemap_entry(atlas, entry_word) else {
                    continue;
                };
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id: cell.id,
                        screen_x: (col * 8) as i16 - h_scroll as i16,
                        screen_y: (row * 8) as i16 - v_scroll as i16,
                        palette: ((entry_word >> 10) & 7) as u8,
                        hflip: false,
                        vflip: false,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state and indexed BG tile instances from a `GpuFrame`
/// into a `ModernFrame`, using a dungeon palette-index atlas keyed by `(theme, graphics_key)`.
///
/// Sets `cgram_rgba` from `frame.cgram`. For layers 0..3 enabled on the main screen OR
/// subscreen (dungeon BG1 walls/statues use color-math / subscreen), reads the full SNES
/// tilemap honoring `tilemap_wider`/`tilemap_higher` (up to 64×64 tiles) with correct
/// four-quadrant VRAM addressing (each 32×32 block at base + quadrant × 0x400), looks up
/// each nonzero word in the dungeon atlas via `dungeon_index_cell(atlas, theme, word)`, and
/// pushes a `ModernIndexTileInstance` with the per-word palette, screen position, and
/// `hflip/vflip` fixed false (the atlas index pattern already bakes flip).
pub fn extract_modern_frame_with_dungeon_atlas(
    frame: &GpuFrame<'_>,
    atlas: &crate::modern_dungeon_atlas::ModernDungeonIndexAtlas,
    theme: u16,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    modern.cgram_rgba = crate::modern_palette::cgram_words_to_rgba256(frame.cgram);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        let enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        // Render if enabled on main OR sub — dungeon BG1 (the room floor/walls) is on the
        // subscreen for color-math and is NOT set on the main screen. The simplified indexed
        // renderer only draws layers whose `enabled_main` is true, so persist the combined
        // (main || sub) visibility into `enabled_main`; otherwise the subscreen floor is
        // extracted but never drawn, leaving the room black behind the HUD.
        let enabled = enabled_main || enabled_sub;
        modern.bg_layers[layer_index].enabled_main = enabled;
        modern.bg_layers[layer_index].enabled_sub = enabled_sub;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        let h_scroll = frame.bg[layer_index].h_scroll;
        let v_scroll = frame.bg[layer_index].v_scroll;
        // Dungeon BG tilemaps are 64×64 (four 32×32 quadrants at base+0x000/0x400/0x800/0xC00).
        // Quadrant index = (right_half ? 1 : 0) + (bottom_half ? (wide ? 2 : 1) : 0).
        let wide = frame.bg[layer_index].tilemap_wider;
        let tall = frame.bg[layer_index].tilemap_higher;
        let cols = if wide { 64usize } else { 32 };
        let rows = if tall { 64usize } else { 32 };
        for ty in 0..rows {
            for tx in 0..cols {
                let q = (if wide && tx >= 32 { 1 } else { 0 })
                    + (if tall && ty >= 32 {
                        if wide {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    });
                let within = (ty % 32) * 32 + (tx % 32);
                let addr = base + q * 0x400 + within;
                let entry_word = *frame.vram.get(addr).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(cell) =
                    crate::modern_dungeon_atlas::dungeon_index_cell(atlas, theme, entry_word)
                else {
                    continue;
                };
                modern.bg_layers[layer_index]
                    .index_tiles
                    .push(ModernIndexTileInstance {
                        cell_id: cell.id,
                        screen_x: (tx * 8) as i16 - h_scroll as i16,
                        screen_y: (ty * 8) as i16 - v_scroll as i16,
                        palette: ((entry_word >> 10) & 7) as u8,
                        hflip: false,
                        vflip: false,
                    });
            }
        }
    }
    modern
}

/// Extract frame-level visual state from a `GpuFrame` into a `ModernFrame`.
///
/// This function copies brightness and forced-blank from the GPU frame.
/// BG layer tile extraction will be added in a subsequent task.
pub fn extract_modern_frame(frame: &GpuFrame<'_>) -> ModernFrame {
    let mut modern = ModernFrame::empty();
    modern.brightness = frame.brightness;
    modern.forced_blank = frame.forced_blank;
    // Raw screen-enable bits + color-math registers for the full software path.
    modern.screen_enabled_main = frame.screen_enabled[0];
    modern.screen_enabled_sub = frame.screen_enabled[1];
    modern.math_enabled = frame.math_enabled;
    modern.subtract_color = frame.subtract_color;
    modern.half_color = frame.half_color;
    modern.fixed_color_r = frame.fixed_color_r;
    modern.fixed_color_g = frame.fixed_color_g;
    modern.fixed_color_b = frame.fixed_color_b;
    modern.add_subscreen = frame.add_subscreen;
    modern
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_frame::{GpuFrame, ScanlineRegs};
    use crate::modern_assets::{ModernTileAtlasAsset, ModernTileAtlasEntry};
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_palette::snes_cgram_to_rgba;

    #[test]
    fn decode_snes_tilemap_entry_splits_visual_fields() {
        let fields = decode_snes_tilemap_entry(0xed23);

        assert_eq!(fields.tile_number, 0x0123);
        assert_eq!(fields.palette, 3);
        assert!(fields.priority);
        assert!(fields.hflip);
        assert!(fields.vflip);
    }

    #[test]
    fn extract_modern_frame_copies_frame_level_visual_state() {
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let frame = test_gpu_frame(&vram, &cgram, &oam, 9, true);

        let modern = extract_modern_frame(&frame);

        assert_eq!(modern.width, 256);
        assert_eq!(modern.height, 224);
        assert_eq!(modern.brightness, 9);
        assert!(modern.forced_blank);
    }

    #[test]
    fn extract_modern_frame_maps_bg_tilemap_entry_to_atlas_tile() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root)
            .expect("atlas should load");
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = 2218;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0x2000;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert_eq!(modern.bg_layers[0].tiles[0].atlas_x_px, 1);
        // Real atlas entry is a 32px source cell at atlas_scale 4, so the on-screen
        // footprint downsamples to the true 8x8 tile size.
        assert_eq!(modern.bg_layers[0].tiles[0].atlas_width_px, 32);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_width_px, 8);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_height_px, 8);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_x, 0);
        assert_eq!(modern.bg_layers[0].tiles[0].screen_y, 0);
    }

    /// Builds a minimal synthetic atlas with a single entry keyed by the given tilemap word.
    fn synthetic_atlas(tilemap_entry: u16) -> ModernTileAtlasAsset {
        ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: 8,
            height_px: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![ModernTileAtlasEntry {
                id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
                tilemap_entry,
                tilemap_variants: vec![tilemap_entry],
            }],
        }
    }

    /// The atlas bakes flip into its cell pixels; re-applying the word's flip bits on the
    /// emitted ModernTileInstance would double-flip asymmetric tiles.  Expect hflip==false
    /// and vflip==false regardless of what the tilemap word's flip bits say.
    #[test]
    fn atlas_sourced_tile_does_not_re_apply_hflip() {
        let hflip_word: u16 = 0x4001; // bit 14 set = hflip
        let atlas = synthetic_atlas(hflip_word);
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = hflip_word;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert!(
            !modern.bg_layers[0].tiles[0].hflip,
            "hflip must be false: atlas bakes flip, re-applying would double-flip"
        );
        assert!(
            !modern.bg_layers[0].tiles[0].vflip,
            "vflip must be false: atlas bakes flip"
        );
    }

    #[test]
    fn atlas_sourced_tile_does_not_re_apply_vflip() {
        let vflip_word: u16 = 0x8001; // bit 15 set = vflip
        let atlas = synthetic_atlas(vflip_word);
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = vflip_word;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let modern = extract_modern_frame_with_atlas(&frame, &atlas);

        assert_eq!(modern.bg_layers[0].tiles.len(), 1);
        assert!(
            !modern.bg_layers[0].tiles[0].hflip,
            "hflip must be false: atlas bakes flip"
        );
        assert!(
            !modern.bg_layers[0].tiles[0].vflip,
            "vflip must be false: atlas bakes flip, re-applying would double-flip"
        );
    }

    /// WORD = 0x0C01: palette=3 (bits [12:10] = 3 = 0b011), tile=1 (bit 0 set), no flip.
    /// graphics_key = 0x0C01 & 0xC3FF = 0x0001 (palette/priority bits stripped).
    #[test]
    fn extract_indexed_frame_emits_index_tile_and_populates_cgram_rgba() {
        // palette=3, tile=1 → word=0x0C01; graphics_key=0x0001
        const WORD: u16 = (3u16 << 10) | 1u16; // 0x0C01
        const GRAPHICS_KEY: u16 = WORD & 0xC3FF; // 0x0001

        let cell = ModernIndexTile {
            id: 0,
            indices: [0u8; 64],
        };
        // Build an atlas that maps GRAPHICS_KEY → cell index 0
        let atlas = ModernIndexAtlas::from_keyed_cells_for_test(vec![cell], &[(GRAPHICS_KEY, 0)]);

        let mut vram = vec![0u16; 0x8000];
        // Set cgram[0]=0x001F (R=31 → [248,0,0,0xff]), cgram[1]=0x7C00 (B=31 → [0,0,248,0xff])
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F;
        cgram[1] = 0x7C00;
        let oam = vec![0u16; 0x110];

        vram[0] = WORD; // tilemap entry at row=0, col=0
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // only BG0 enabled on main

        let modern = extract_modern_frame_with_index_atlas(&frame, &atlas);

        // One indexed tile on layer 0
        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.cell_id, 0);
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, 0);
        assert!(!inst.hflip);
        assert!(!inst.vflip);

        // No RGBA atlas tiles emitted
        assert!(modern.bg_layers[0].tiles.is_empty());

        // Layers 1 and 2 are disabled → no tiles
        assert!(modern.bg_layers[1].index_tiles.is_empty());
        assert!(modern.bg_layers[2].index_tiles.is_empty());

        // cgram_rgba populated from frame.cgram
        assert_eq!(modern.cgram_rgba[0], snes_cgram_to_rgba(0x001F)); // [248,0,0,0xff]
        assert_eq!(modern.cgram_rgba[1], snes_cgram_to_rgba(0x7C00)); // [0,0,248,0xff]
                                                                      // Slots beyond the supplied cgram default to black opaque
        assert_eq!(modern.cgram_rgba[255], [0, 0, 0, 0xff]);
    }

    /// WORD = (3<<10)|0x012 → palette=3, tile=0x12, no flip.
    /// graphics_key = WORD & 0xC3FF = 0x0012 (palette bits stripped).
    /// packed key  = ((THEME as u32)<<16) | 0x0012.
    #[test]
    fn extract_dungeon_frame_emits_index_tile_by_theme_and_populates_cgram_rgba() {
        const THEME: u16 = 4;
        const WORD: u16 = (3u16 << 10) | 0x012; // palette=3, tile=0x12 → 0x0C12
        const GKEY: u32 = ((THEME as u32) << 16) | ((WORD & 0xC3FF) as u32);

        let cell = ModernIndexTile {
            id: 42,
            indices: [0u8; 64],
        };
        let atlas = crate::modern_dungeon_atlas::ModernDungeonIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![(GKEY, 0)],
        );

        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F; // R=31 → [248, 0, 0, 255]
        cgram[1] = 0x7C00; // B=31 → [0, 0, 248, 255]
        let oam = vec![0u16; 0x110];

        vram[0] = WORD; // tilemap entry at row=0, col=0
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // only BG0 on main

        let modern =
            crate::modern_extract::extract_modern_frame_with_dungeon_atlas(&frame, &atlas, THEME);

        // One indexed tile on layer 0
        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.cell_id, 42);
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, 0);
        assert!(!inst.hflip);
        assert!(!inst.vflip);

        // No RGBA atlas tiles emitted on any layer
        assert!(modern.bg_layers[0].tiles.is_empty());
        assert!(modern.bg_layers[1].index_tiles.is_empty());
        assert!(modern.bg_layers[2].index_tiles.is_empty());

        // cgram_rgba populated from frame.cgram
        assert_eq!(modern.cgram_rgba[0], snes_cgram_to_rgba(0x001F));
        assert_eq!(modern.cgram_rgba[1], snes_cgram_to_rgba(0x7C00));
        assert_eq!(modern.cgram_rgba[255], [0, 0, 0, 0xff]);

        // Wrong theme → zero tiles (key won't resolve)
        let modern_wrong_theme = crate::modern_extract::extract_modern_frame_with_dungeon_atlas(
            &frame,
            &atlas,
            THEME + 1,
        );
        assert!(
            modern_wrong_theme.bg_layers[0].index_tiles.is_empty(),
            "wrong theme must yield zero tiles"
        );
    }

    #[test]
    fn decode_2bpp_tile_reads_two_bitplanes() {
        // tile#0 at chr_base 0: row0 word = planes 0 (low) + 1 (high). Set both bit7
        // (pixel x=0) → index 3; leave the rest 0.
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x8080; // bp0 bit7 + bp1 bit7
        let out = decode_snes_2bpp_tile_indices(&vram, 0, 0);
        assert_eq!(out[0], 3, "pixel (0,0) = 2bpp index 3");
        assert_eq!(out[1], 0, "pixel (1,0) untouched");
        assert_eq!(out[8], 0, "pixel (0,1) untouched");
    }

    #[test]
    fn extract_dungeon_from_vram_decodes_live_chr_and_backdrop() {
        // CHR base 0, tile#1 → tile_base 16. Set pixel (0,0) to 4bpp index 10
        // (planes 1 and 3 → w01 high byte + w23 high byte have bit7).
        let mut vram = vec![0u16; 0x8000];
        vram[16] = 0x8000; // w01: bp1 bit7  (plane 1)
        vram[24] = 0x8000; // w23: bp3 bit7  (plane 3)  => index 0b1010 = 10
        vram[0x1000] = (3u16 << 10) | 1; // tilemap entry: palette 3, tile 1, no flip

        let mut cgram = vec![0u16; 0x100];
        cgram[0] = 0x001F; // backdrop R=31 → [248,0,0,255]
        let oam = vec![0u16; 0x110];

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0x1000;
        frame.bg[0].tile_adr = 0;
        frame.screen_enabled = [0x01, 0x00]; // BG1 on main only

        let (modern, cells) = crate::modern_extract::extract_modern_dungeon_frame_from_vram(&frame);

        assert_eq!(modern.bg_layers[0].index_tiles.len(), 1);
        let inst = &modern.bg_layers[0].index_tiles[0];
        assert_eq!(inst.palette, 3);
        assert_eq!(inst.screen_x, 0);
        assert_eq!(inst.screen_y, 0);
        let cell = &cells[inst.cell_id as usize];
        assert_eq!(cell.indices[0], 10, "live-VRAM decoded index at (0,0)");
        assert_eq!(cell.indices[1], 0, "neighbour pixel transparent");
        // Backdrop comes from CGRAM[0] (the classic main color-math operand).
        assert_eq!(modern.backdrop_color_rgba, snes_cgram_to_rgba(0x001F));
        // BG3 (layer 2) is intentionally not decoded from VRAM.
        assert!(modern.bg_layers[2].index_tiles.is_empty());
    }

    /// Craft an OAM with ONE 8×8 sprite and assert `extract_modern_sprites`
    /// resolves it to a single instance with the right cell/palette/priority/pos.
    /// For an 8×8 sprite, `effective_tile == oam1 & 0xff` (bank 0, single tile).
    #[test]
    fn extract_modern_sprites_decodes_one_8x8_sprite() {
        const CONTEXT: u64 = 21;
        const TILE: u16 = 5;
        let palette: u16 = 2;
        let priority: u16 = 1;
        let x: u16 = 40;
        let y: u16 = 50;

        let cell = ModernIndexTile {
            id: 99,
            indices: [0u8; 64],
        };
        let atlas = ModernSpriteIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![((CONTEXT, TILE), 0)],
        );

        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (y << 8) | x;
        oam[1] = (palette << 9) | (priority << 12) | TILE;
        // hi-word (oam[0x100]) left 0 → size bit 0 (small=8), x-hi 0.

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0; // SPRITE_SIZES[0] = [8, 16] → small = 8

        let sprites = extract_modern_sprites(&frame, &atlas, CONTEXT);

        assert_eq!(sprites.len(), 1);
        let s = &sprites[0];
        assert_eq!(s.cell_id, 99);
        assert_eq!(s.palette, 2);
        assert_eq!(s.priority, 1);
        assert_eq!(s.screen_x, 40);
        assert_eq!(s.screen_y, 50);
        assert!(!s.hflip);
        assert!(!s.vflip);

        // Wrong context → no resolution.
        assert!(extract_modern_sprites(&frame, &atlas, CONTEXT + 1).is_empty());
    }

    /// hflip on an 8×8 sprite resolves the same single tile but propagates the flag.
    #[test]
    fn extract_modern_sprites_propagates_hflip() {
        const CONTEXT: u64 = 21;
        const TILE: u16 = 5;

        let cell = ModernIndexTile {
            id: 7,
            indices: [0u8; 64],
        };
        let atlas = ModernSpriteIndexAtlas::from_keyed_cells_for_test(
            vec![cell],
            vec![((CONTEXT, TILE), 0)],
        );

        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (50u16 << 8) | 40u16;
        oam[1] = 0x4000 | TILE; // bit 14 = hflip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0;

        let sprites = extract_modern_sprites(&frame, &atlas, CONTEXT);

        assert_eq!(sprites.len(), 1);
        assert_eq!(sprites[0].cell_id, 7);
        assert!(sprites[0].hflip);
        assert!(!sprites[0].vflip);
    }

    /// Live-VRAM sprite decode: an 8×8 sprite using tile T from CHR page
    /// `tile_adr1 = B` must yield a cell whose pattern equals
    /// `decode_snes_4bpp_tile_indices` for the same words, and one instance with
    /// the matching cell_id/palette/screen position.
    #[test]
    fn extract_modern_sprites_from_vram_decodes_live_chr() {
        const B: usize = 0x1000; // CHR page word base (tile_adr1)
        const TILE: u16 = 3;
        let palette: u16 = 4;
        let priority: u16 = 2;
        let x: u16 = 40;
        let y: u16 = 50;

        let mut vram = vec![0u16; 0x8000];
        // Encode a known pattern for tile T at base B:
        //   row 0: bp0 = 0xFF → all 8 pixels index bit0 set (index 1)
        //   row 3: bp0/bp1 = 0xFF/0xFF → all 8 pixels index 3
        let tile_base = B + (TILE as usize) * 16;
        vram[tile_base] = 0x00FF; // y=0: bp0=0xFF, bp1=0x00
        vram[tile_base + 3] = 0xFFFF; // y=3: bp0=0xFF, bp1=0xFF

        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        oam[0] = (y << 8) | x;
        oam[1] = (palette << 9) | (priority << 12) | TILE; // bank 0, no flip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0; // small = 8×8
        frame.obj.tile_adr1 = B as u16;
        frame.obj.tile_adr2 = 0;

        let (cells, sprites) = extract_modern_sprites_from_vram(&frame);

        assert_eq!(sprites.len(), 1, "one visible 8×8 sprite tile");
        let s = &sprites[0];
        assert_eq!(s.palette, 4);
        assert_eq!(s.priority, 2);
        assert_eq!(s.screen_x, 40);
        assert_eq!(s.screen_y, 50);
        assert!(!s.hflip);
        assert!(!s.vflip);

        // The referenced cell's pattern equals the unflipped decode of the words.
        let expected = decode_snes_4bpp_tile_indices(&vram, B, TILE);
        assert_ne!(expected, [0u8; 64], "pattern must be non-trivial");
        let cell = &cells[s.cell_id as usize];
        assert_eq!(cell.indices, expected);
        // Spot-check the crafted rows.
        assert!(cell.indices[0..8].iter().all(|&i| i == 1));
        assert!(cell.indices[24..32].iter().all(|&i| i == 3));
    }

    /// hflip propagates as a flag; the decoded cell stays UNFLIPPED (so a second
    /// hflipped sprite using the same tile dedups to the same cell).
    #[test]
    fn extract_modern_sprites_from_vram_propagates_hflip_keeps_cell_unflipped() {
        const B: usize = 0x2000;
        const TILE: u16 = 7;

        let mut vram = vec![0u16; 0x8000];
        let tile_base = B + (TILE as usize) * 16;
        vram[tile_base] = 0x00FF; // row 0 index 1

        let cgram = vec![0u16; 0x100];
        let mut oam = vec![0u16; 0x110];
        // Sprite 0: no flip. Sprite 1: hflip, same tile.
        oam[0] = (50u16 << 8) | 40u16;
        oam[1] = TILE;
        oam[2] = (60u16 << 8) | 80u16;
        oam[3] = 0x4000 | TILE; // bit 14 = hflip

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.obj.obj_size = 0;
        frame.obj.tile_adr1 = B as u16;

        let (cells, sprites) = extract_modern_sprites_from_vram(&frame);

        assert_eq!(sprites.len(), 2);
        // Both reference the SAME (unflipped) cell — hflip is a per-instance flag.
        assert_eq!(sprites[0].cell_id, sprites[1].cell_id);
        assert_eq!(cells.len(), 1, "identical tiles dedup to one cell");
        assert!(!sprites[0].hflip);
        assert!(sprites[1].hflip);

        let expected = decode_snes_4bpp_tile_indices(&vram, B, TILE);
        assert_eq!(cells[sprites[1].cell_id as usize].indices, expected);
    }

    fn test_gpu_frame<'a>(
        vram: &'a [u16],
        cgram: &'a [u16],
        oam: &'a [u16],
        brightness: u8,
        forced_blank: bool,
    ) -> GpuFrame<'a> {
        GpuFrame {
            vram,
            cgram,
            oam,
            mode: 1,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness,
            forced_blank,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([ScanlineRegs::default(); 224]),
        }
    }
}
