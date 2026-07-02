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
                        x: inst.screen_x, y: inst.screen_y, w: 8, h: 8,
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
                    x: inst.screen_x, y: inst.screen_y, w: 8, h: 8,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::ModernFrame;
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;

    fn cell(id: u32, source_key: u64) -> ModernIndexTile {
        ModernIndexTile { id, indices: [0u8; 64], source_key, hflip: false, vflip: false }
    }

    #[test]
    fn placement_map_records_keyed_bg_and_sprite_positions_and_skips_unkeyed() {
        use crate::modern_frame::{ModernIndexTileInstance, ModernIndexSpriteInstance};
        let mut frame = ModernFrame::empty();
        // BG tile: keyed cell 0 at (16,24); unkeyed cell 1 at (0,0) -> skipped.
        frame.bg_layers[0].index_tiles.push(ModernIndexTileInstance {
            cell_id: 0, screen_x: 16, screen_y: 24, palette: 0, hflip: false, vflip: false, priority: false,
        });
        frame.bg_layers[0].index_tiles.push(ModernIndexTileInstance {
            cell_id: 1, screen_x: 0, screen_y: 0, palette: 0, hflip: false, vflip: false, priority: false,
        });
        let bg_cells = vec![cell(0, 0xABCD), cell(1, NO_SOURCE_KEY)];
        // Sprite: keyed cell 0 at (32,40).
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0, screen_x: 32, screen_y: 40, palette: 0, priority: 0, hflip: false, vflip: false, row_mask: 0xff,
        });
        let sprite_cells = vec![cell(0, 0x1234)];

        let map = build_hd_placement_map(&frame, &bg_cells, &sprite_cells);
        assert_eq!(map, vec![
            HdPlacement { key: "0x000000000000abcd".into(), x: 16, y: 24, w: 8, h: 8 },
            HdPlacement { key: "0x0000000000001234".into(), x: 32, y: 40, w: 8, h: 8 },
        ]);
    }
}
