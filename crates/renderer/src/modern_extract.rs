use crate::gpu_frame::GpuFrame;
use crate::modern_assets::{atlas_entry_for_tilemap_entry, ModernTileAtlasAsset};
use crate::modern_frame::{ModernFrame, ModernTileInstance};

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

/// Extract frame-level visual state from a `GpuFrame` into a `ModernFrame`.
///
/// This function copies brightness and forced-blank from the GPU frame.
/// BG layer tile extraction will be added in a subsequent task.
pub fn extract_modern_frame(frame: &GpuFrame<'_>) -> ModernFrame {
    let mut modern = ModernFrame::empty();
    modern.brightness = frame.brightness;
    modern.forced_blank = frame.forced_blank;
    modern
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_frame::{GpuFrame, ScanlineRegs};
    use crate::modern_assets::{ModernTileAtlasAsset, ModernTileAtlasEntry};

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
