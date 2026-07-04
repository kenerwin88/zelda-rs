//! Offline HD-art authoring helpers (not on the render/parity path). Builds the
//! per-frame placement map (source key -> screen rect) used to slice HD cells out
//! of super-resolved frames.
use serde::{Deserialize, Serialize};

use crate::modern_frame::ModernFrame;
use crate::modern_hd_overrides::NO_SOURCE_KEY;
use crate::modern_index_atlas::ModernIndexTile;

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
    use crate::modern_frame::ModernFrame;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;
    use crate::modern_index_atlas::ModernIndexTile;

    fn cell(id: u32, source_key: u64) -> ModernIndexTile {
        ModernIndexTile {
            id,
            indices: [0u8; 64],
            source_key,
            hflip: false,
            vflip: false,
        }
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
}
