use crate::gpu_frame::GpuFrame;
use crate::modern_assets::{atlas_entry_for_tilemap_entry, ModernTileAtlasAsset};
use crate::modern_frame::{ModernFrame, ModernIndexTileInstance, ModernTileInstance};
use crate::modern_index_atlas::{index_cell_for_tilemap_entry, ModernIndexAtlas};

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
