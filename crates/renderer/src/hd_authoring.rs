//! Offline HD-art authoring helpers (not on the render/parity path). Builds the
//! per-frame placement map (source key -> screen rect) used to slice HD cells out
//! of super-resolved frames.
use serde::{Deserialize, Serialize};

use crate::gpu_frame::GpuFrame;
use crate::modern_frame::ModernFrame;
use crate::modern_hd_overrides::NO_SOURCE_KEY;
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_source_atlas::ModernSourceAtlas;

/// One drawn 8×8 cell occurrence: its source key and native-pixel screen rect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HdPlacement {
    /// Source key as hex string `0x{:016x}` (matches the atlas dump + manifest).
    pub key: String,
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

/// Native-frame HD-authoring capture assembled from a source-atlas GPU frame.
pub struct HdCaptureFrame {
    /// Native 256×224 RGBA8 frame, rendered at scale 1 with HD overrides disabled.
    pub rgba: Vec<u8>,
    /// Source-key placements for slicing generated HD cells.
    pub placements: Vec<HdPlacement>,
    /// 256-entry CGRAM palette converted to RGBA, used as the reference palette.
    pub cgram_rgba: [[u8; 4]; 256],
}

/// Build the native RGBA frame and placement map for offline HD-art authoring.
///
/// This mirrors the source-atlas modern path but returns authoring metadata in
/// one renderer-owned bundle so callers do not assemble intermediate modern
/// frames themselves.
pub fn render_hd_capture_from_sources<S: crate::modern_extract::SourceTableView + ?Sized>(
    frame: &GpuFrame<'_>,
    src_table: &S,
    atlas: &ModernSourceAtlas,
) -> HdCaptureFrame {
    let (mut modern, bg_cells) =
        crate::modern_extract::extract_modern_frame_from_sources(frame, src_table, atlas);
    let (sprite_cells, sprites) =
        crate::modern_extract::extract_modern_sprites_from_sources(frame, src_table, atlas);
    modern.index_sprites = sprites;

    let ctx = crate::modern_hd_overrides::HdOverrideCtx::disabled();
    let rgba = crate::modern_software::render_modern_frame_full_scaled(
        &modern,
        &bg_cells,
        &sprite_cells,
        &ctx,
        1,
    );
    let placements = build_hd_placement_map(&modern, &bg_cells, &sprite_cells);
    HdCaptureFrame {
        rgba,
        placements,
        cgram_rgba: modern.cgram_rgba,
    }
}

/// Enumerate every drawn tile/sprite instance that has a real source key, with its
/// native screen position. Cells are 8×8. `NO_SOURCE_KEY` cells are skipped.
pub fn build_hd_placement_map(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<HdPlacement> {
    let mut out = Vec::new();
    for layer in &frame.bg_layers {
        for inst in &layer.index_tiles {
            if let Some(c) = bg_cells.get(inst.cell_id as usize) {
                if c.source_key != NO_SOURCE_KEY {
                    out.push(HdPlacement {
                        key: format!("0x{:016x}", c.source_key),
                        x: inst.screen_x,
                        y: inst.screen_y,
                        w: 8,
                        h: 8,
                    });
                }
            }
        }
    }
    for inst in &frame.index_sprites {
        if let Some(c) = sprite_cells.get(inst.cell_id as usize) {
            if c.source_key != NO_SOURCE_KEY {
                out.push(HdPlacement {
                    key: format!("0x{:016x}", c.source_key),
                    x: inst.screen_x,
                    y: inst.screen_y,
                    w: 8,
                    h: 8,
                });
            }
        }
    }
    out
}

/// Crop one cell's HD pixels from a super-resolved frame. `sr` is row-major RGBA8
/// of `sr_w × sr_h`. The cell footprint is `w×h` native px at native `(x,y)`,
/// upscaled by `scale` (so the crop is `(w*scale)×(h*scale)`). Returns `None` if
/// the upscaled crop is not fully inside the frame (partial/negative → skip).
pub fn slice_hd_cell(
    sr: &[u8],
    sr_w: u32,
    sr_h: u32,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
    scale: u32,
) -> Option<Vec<u8>> {
    if x < 0 || y < 0 {
        return None;
    }
    let ow = w as u32 * scale;
    let oh = h as u32 * scale;
    let ox = x as u32 * scale;
    let oy = y as u32 * scale;
    if ox + ow > sr_w || oy + oh > sr_h {
        return None;
    }
    let row_bytes = (ow * 4) as usize;
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for row in 0..oh {
        let src = (((oy + row) * sr_w + ox) * 4) as usize;
        let dst = (row * ow * 4) as usize;
        out[dst..dst + row_bytes].copy_from_slice(&sr[src..src + row_bytes]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_frame::{GpuFrame, ScanlineRegs};
    use crate::modern_extract::{
        extract_modern_frame_from_sources, extract_modern_sprites_from_sources,
    };
    use crate::modern_frame::ModernFrame;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_source_atlas::{modern_source_key, ModernSourceAtlas};

    fn cell(id: u32, source_key: u64) -> ModernIndexTile {
        ModernIndexTile {
            id,
            indices: [0u8; 64],
            source_key,
            hflip: false,
            vflip: false,
        }
    }

    fn test_gpu_frame<'a>(
        vram: &'a [u16],
        cgram: &'a [u16],
        oam: &'a [u16],
        brightness: u8,
        forced_blank: bool,
    ) -> GpuFrame<'a> {
        GpuFrame {
            hardware_startup_transient: None,
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
            mode7_scanout_brightness_override: None,
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
            bg3_source_tiles: &[],
            bg3_vwf_glyph_runs: &[],
            dialogue_message_id: None,
            source_dialogue_ir: &[],
            dialogue_ir: &[],
            dialogue_layout: &[],
            dialogue_layout_origin_tile_number: None,
            cgram_provenance: None,
        }
    }

    fn content_hash32_slot_for_test(vram: &[u16], slot: usize) -> u32 {
        let base = slot * 16;
        let mut hash: u32 = 0x811c_9dc5;
        for off in 0..16 {
            let word = *vram.get(base + off).unwrap_or(&0);
            for byte in [(word & 0xff) as u8, (word >> 8) as u8] {
                hash ^= byte as u32;
                hash = hash.wrapping_mul(0x0100_0193);
            }
        }
        hash
    }

    #[test]
    fn placement_map_records_keyed_bg_and_sprite_positions_and_skips_unkeyed() {
        use crate::modern_frame::{ModernIndexSpriteInstance, ModernIndexTileInstance};
        let mut frame = ModernFrame::empty();
        // BG tile: keyed cell 0 at (16,24); unkeyed cell 1 at (0,0) -> skipped.
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 16,
                screen_y: 24,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 1,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let bg_cells = vec![cell(0, 0xABCD), cell(1, NO_SOURCE_KEY)];
        // Sprite: keyed cell 0 at (32,40).
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 40,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let sprite_cells = vec![cell(0, 0x1234)];

        let map = build_hd_placement_map(&frame, &bg_cells, &sprite_cells);
        assert_eq!(
            map,
            vec![
                HdPlacement {
                    key: "0x000000000000abcd".into(),
                    x: 16,
                    y: 24,
                    w: 8,
                    h: 8
                },
                HdPlacement {
                    key: "0x0000000000001234".into(),
                    x: 32,
                    y: 40,
                    w: 8,
                    h: 8
                },
            ]
        );
    }

    #[test]
    fn slice_extracts_scaled_region_and_skips_offscreen() {
        // 4x2 native SR frame at scale 2 -> sr is 8x4 RGBA. Fill each pixel R=x, G=y.
        let (sr_w, sr_h, scale) = (8u32, 4u32, 2u32);
        let mut sr = vec![0u8; (sr_w * sr_h * 4) as usize];
        for py in 0..sr_h {
            for px in 0..sr_w {
                let i = ((py * sr_w + px) * 4) as usize;
                sr[i] = px as u8;
                sr[i + 1] = py as u8;
                sr[i + 3] = 0xff;
            }
        }
        // Native cell 1x1 footprint at native (1,0), scale 2 -> crop sr region x=2..4, y=0..2.
        let got = slice_hd_cell(&sr, sr_w, sr_h, 1, 0, 1, 1, scale).expect("on-screen");
        assert_eq!(got.len(), (2 * 2 * 4) as usize);
        assert_eq!(&got[0..4], &[2, 0, 0, 0xff]); // sr (2,0)
        assert_eq!(&got[4..8], &[3, 0, 0, 0xff]); // sr (3,0)
        assert_eq!(&got[8..12], &[2, 1, 0, 0xff]); // sr (2,1)
                                                   // Negative and overhanging placements skip.
        assert!(slice_hd_cell(&sr, sr_w, sr_h, -1, 0, 1, 1, scale).is_none());
        assert!(slice_hd_cell(&sr, sr_w, sr_h, 4, 0, 1, 1, scale).is_none()); // x*scale=8 >= sr_w
    }

    #[test]
    fn capture_from_sources_matches_manual_authoring_assembly() {
        const CHR_KIND_BG_STREAM: u8 = 6;

        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        cgram[7] = 0x001f;
        for (i, word) in vram[0x2040..0x2050].iter_mut().enumerate() {
            *word = 0x1000u16.wrapping_add(i as u16);
        }
        vram[0] = 4;
        let hash = content_hash32_slot_for_test(&vram, 0x204);
        let source_key = modern_source_key(
            CHR_KIND_BG_STREAM,
            (hash >> 16) as u16,
            (hash & 0xffff) as u16,
        );

        let mut indices = [0u8; 64];
        indices[0] = 7;
        let atlas = ModernSourceAtlas::from_keyed_cells_for_test(
            vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }],
            &[(
                CHR_KIND_BG_STREAM,
                (hash >> 16) as u16,
                (hash & 0xffff) as u16,
                0,
            )],
        );
        let table = |slot: usize| -> (u8, u16, u16) {
            if slot == 0x200 + 4 {
                (CHR_KIND_BG_STREAM, 9, 99)
            } else {
                (0, 0, 0)
            }
        };

        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.bg[0].tile_adr = 0x2000;
        frame.screen_enabled = [0x01, 0x00];

        let (mut modern, bg_cells) = extract_modern_frame_from_sources(&frame, &table, &atlas);
        let (sprite_cells, sprites) = extract_modern_sprites_from_sources(&frame, &table, &atlas);
        modern.index_sprites = sprites;
        let ctx = crate::modern_hd_overrides::HdOverrideCtx::disabled();
        let manual_rgba = crate::modern_software::render_modern_frame_full_scaled(
            &modern,
            &bg_cells,
            &sprite_cells,
            &ctx,
            1,
        );
        let manual_map = build_hd_placement_map(&modern, &bg_cells, &sprite_cells);

        let capture = render_hd_capture_from_sources(&frame, &table, &atlas);

        assert_eq!(capture.rgba, manual_rgba);
        assert_eq!(capture.placements, manual_map);
        assert_eq!(capture.cgram_rgba, modern.cgram_rgba);
        assert_eq!(
            capture.placements,
            vec![HdPlacement {
                key: format!("0x{source_key:016x}"),
                x: 0,
                y: -1,
                w: 8,
                h: 8,
            }]
        );
    }
}
