// This module provides a dev-tooling function wired up fully in Task 13.  The public API is
// intentionally unused by the binary for now; silence the lint rather than leaving warnings.
#![allow(dead_code)]

/// Builds a [`renderer::modern_frame::ModernFrame`] from a developer room's JSON tilemap and
/// the overworld tile atlas, without touching any SNES VRAM/CGRAM/OAM addresses.
///
/// # Mapping note (simplification)
/// The JSON tilemap stores Kakariko source-cell ids (small integers 0..N) in each cell.
/// These are mapped to overworld atlas entries by matching `atlas_entry.id == cell_value`.
/// This is a deliberate simplification: the cell ids are not genuine SNES tilemap words, so
/// the rendered colours will not be pixel-faithful. The purpose of Task 12 is to prove the
/// VRAM-free emission path (room JSON → ModernFrame), not pixel parity.  As long as some
/// cells resolve to atlas entries the frame contains tiles, which is all the test asserts.
use renderer::modern_assets::load_modern_overworld_tile_atlas;
use renderer::modern_frame::{ModernFrame, ModernTileInstance};
use std::path::Path;

#[derive(serde::Deserialize)]
struct RoomTilemap {
    // width and height are present in the JSON schema for documentation purposes but are
    // not used at runtime — the row/column iteration drives the screen positions directly.
    #[allow(dead_code)]
    width: u16,
    #[allow(dead_code)]
    height: u16,
    rows: Vec<Vec<u32>>,
}

/// Emit a [`ModernFrame`] from the developer room identified by `room_id`.
///
/// Uses no SNES VRAM, CGRAM, or OAM addresses — only the room's JSON tilemap and the
/// prebuilt overworld tile atlas on disk.
pub fn load_developer_modern_frame(room_id: &str) -> Result<ModernFrame, String> {
    let room = crate::developer_destinations::synthetic_room(room_id)
        .ok_or_else(|| format!("unknown developer room: {room_id}"))?;

    let tilemap: RoomTilemap = serde_json::from_str(room.tilemap_json)
        .map_err(|e| format!("failed to parse room tilemap JSON: {e}"))?;

    // env!("CARGO_MANIFEST_DIR") resolves to zelda3-bin at compile time; ".." is the
    // workspace root, which is where load_modern_overworld_tile_atlas expects it.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let atlas = load_modern_overworld_tile_atlas(&repo_root)
        .map_err(|e| format!("failed to load overworld tile atlas: {e}"))?;

    let scale = atlas.atlas_scale.max(1);
    let mut frame = ModernFrame::empty();
    // Enable layer 0 on the main screen so the test assertion (`any layer has tiles`) holds.
    frame.bg_layers[0].enabled_main = true;

    for (y, row) in tilemap.rows.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            // Match by atlas entry id.  Cells that have no corresponding entry are skipped
            // (the JSON may contain ids beyond the atlas range; skip silently per spec).
            let Some(entry) = atlas.entries.iter().find(|e| e.id == cell) else {
                continue;
            };
            frame.bg_layers[0].tiles.push(ModernTileInstance {
                atlas_id: entry.id,
                atlas_x_px: entry.atlas_x_px,
                atlas_y_px: entry.atlas_y_px,
                atlas_width_px: entry.atlas_width_px,
                atlas_height_px: entry.atlas_height_px,
                // Downsample from atlas pixels to true 8×8 screen pixels.
                screen_width_px: entry.atlas_width_px / scale,
                screen_height_px: entry.atlas_height_px / scale,
                screen_x: (x * 8) as i16,
                screen_y: (y * 8) as i16,
                palette: 0,
                priority: 0,
                hflip: false,
                vflip: false,
                transparent_color_zero: true,
            });
        }
    }

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn developer_sandbox_can_emit_modern_frame_without_vram_addresses() {
        let frame = load_developer_modern_frame("preset-dev-sandbox")
            .expect("sandbox should emit modern frame");
        assert_eq!(frame.width, 256);
        assert_eq!(frame.height, 224);
        assert!(frame.bg_layers.iter().any(|layer| !layer.tiles.is_empty()));
    }
}
