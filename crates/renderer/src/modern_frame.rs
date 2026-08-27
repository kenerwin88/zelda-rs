pub const MODERN_FRAME_WIDTH: u16 = 256;
pub const MODERN_FRAME_HEIGHT: u16 = 224;

/// Apply the SNES INIDISP master-brightness DAC curve to one 5-bit channel.
/// Brightness zero is black and level 15 preserves the source component;
/// forced blank remains a separate display state.
#[inline]
pub(crate) fn apply_master_brightness(c5: u8, brightness: u8) -> u8 {
    let c5 = u32::from(c5.min(31));
    let scaled = (c5 * u32::from(brightness.min(15)) + 7) / 15;
    ((scaled << 3) | (scaled >> 2)) as u8
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernFrame {
    pub width: u16,
    pub height: u16,
    pub bg_layers: Vec<ModernBgLayer>,
    pub sprites: Vec<ModernSpriteInstance>,
    pub index_sprites: Vec<ModernIndexSpriteInstance>,
    /// A short-lived direct-color cell emitted by the semantic hardware-startup
    /// model. It is composited after the normal modern frame, because its color
    /// is Snes9x power-on residue rather than a game CGRAM entry.
    pub hardware_startup_transient: Option<crate::gpu_frame::HardwareStartupTransient>,
    pub bg3_vwf_glyph_runs: Vec<ModernVwfGlyphRun>,
    pub dialogue_message_id: Option<u16>,
    pub source_dialogue_ir: Vec<zelda3_dialogue::DialogueIrOp>,
    pub dialogue_ir: Vec<zelda3_dialogue::DialogueIrOp>,
    pub dialogue_layout: Vec<zelda3_dialogue::DialogueGlyphPlacement>,
    pub dialogue_layout_vwf_glyph_runs: Vec<ModernVwfGlyphRun>,
    /// 2bpp palette row carried by the message box's BG3 tilemap entries
    /// (bits 10-12 of the entry mapping the VWF origin tile). Semantic glyph
    /// cells must render through THIS row, not an assumed default: the intro
    /// telepathy box maps its text with palette 6 while ordinary boxes use 7.
    pub dialogue_box_tilemap_palette: Option<u8>,
    pub backdrop_color_rgba: [u8; 4],
    pub brightness: u8,
    /// One-frame Mode 7 master-brightness generation for the deferred
    /// world-map fade.
    pub scanout_brightness_override: Option<u8>,
    pub forced_blank: bool,
    /// Number of leading visible scanlines that began while INIDISP was still
    /// forced blank after an overlong VBlank workload.
    pub forced_blank_scanlines: u8,
    /// First scanline in a trailing forced-blank range caused by an INIDISP
    /// write during active display. `None` means there is no blank suffix.
    pub forced_blank_from_scanline: Option<u8>,
    /// Whether the visible gap between forced-blank ranges was scanned before
    /// an NMI replaced its display memory and must come from surface history.
    pub retain_active_display_history: bool,
    pub cgram_rgba: [[u8; 4]; 256],
    /// Provenance-clean CGRAM mirror committed at the last CGRAM upload —
    /// the zero-CGRAM color source that replaces `cgram_rgba` once complete.
    /// `None` on paths without game state (unit frames, dumps).
    pub cgram_provenance: Option<zelda3_palette::CgramProvenanceSnapshot>,
    /// Mode-1 BG3-high ordering selected by BGMODE bit 3 for this scanout.
    pub mode1_bg3_priority: bool,
    /// Raw main-screen layer-enable bits (TM/$212C): 0=BG1..3=BG4, 4=OBJ.
    /// Used by the full (color-math) software path to build the MAIN composite;
    /// distinct from the per-layer `enabled_main`, which the dungeon extractor
    /// pollutes with sub-screen layers so the simple renderer draws them.
    pub screen_enabled_main: u8,
    /// Raw sub-screen layer-enable bits (TS/$212D): 0=BG1..3=BG4, 4=OBJ.
    pub screen_enabled_sub: u8,
    /// Color-math participation bitmask (CGADSUB): bits 0-5 = BG1-4, OBJ, backdrop.
    pub math_enabled: u8,
    /// Color math subtracts the second operand instead of adding.
    pub subtract_color: bool,
    /// Halve the post-math channel before brightness scaling.
    pub half_color: bool,
    /// Fixed color (COLDATA), each channel 5-bit (0-31).
    pub fixed_color_r: u8,
    pub fixed_color_g: u8,
    pub fixed_color_b: u8,
    /// Use the sub-screen pixel as the second math operand instead of the fixed color.
    pub add_subscreen: bool,
    /// Clip mode (CGWSEL): 0=never clip, 1=outside window, 2=inside window, 3=always.
    pub clip_mode: u8,
    /// Prevent-math mode (CGWSEL): 0=never prevent, 1=outside, 2=inside, 3=always.
    pub prevent_math_mode: u8,
    /// Color-math window select (windowsel layer 5): bits 0-3 = W1inv, W1en, W2inv, W2en.
    pub windowsel_cm: u8,
    /// Full per-layer window-select register (PPU windowsel). Each layer L occupies
    /// nibble L: bit0=W1inv, bit1=W1en, bit2=W2inv, bit3=W2en. Layers 0-2 = BG1-3,
    /// layer 4 (>>16) = OBJ (layer 5 / >>20 = color-math, kept in `windowsel_cm`).
    /// Used by the main-screen window mask in the Mode-1 compositor. Default 0.
    pub windowsel: u32,
    /// Main-screen window-enable bits (TMW/$212E): bit L masks layer L's main pixels
    /// where the layer's window region is active. Default 0 (no masking).
    pub screen_windowed_main: u8,
    /// Sub-screen window-enable bits (TSW/$212F): bit L masks layer L's sub pixels
    /// where the layer's window region is active. The classic applies windows to the
    /// SUB screen too (it only skips the per-scanline TM check there, not the window),
    /// which is how the dungeon color-math floor iris masks to backdrop. Default 0.
    pub screen_windowed_sub: u8,
    /// Per-scanline color-window boundaries `[w1_left, w1_right, w2_left, w2_right]`
    /// (column indices, inclusive), 224 entries. Mirrors the GPU post-process pass.
    pub window_scanlines: Vec<[u8; 4]>,
    /// Per-scanline main-screen layer-enable (TM/$212C, HDMA-captured), 224 entries.
    /// A main BG/OBJ pixel shows only if its scanline has the layer bit set (BG L =
    /// `1<<L`, OBJ = `0x10`), matching the classic per-scanline TM check. Default
    /// `0xff` = all enabled (no gating) so unit frames are unaffected.
    pub main_tm_scanlines: Vec<u8>,
    /// SNES mosaic enable bits (MOSAIC/$2106 low nibble): bit L = BG layer L mosaiced.
    /// Default 0 (no layer mosaiced) so normal frames take the non-mosaic path.
    pub mosaic_enabled: u8,
    /// SNES mosaic block size N (1..16; N<=1 means mosaic off). Default 1.
    pub mosaic_size: u8,
    /// Per-scanline BG scroll captured after HDMA (mirrors the classic GPU's
    /// `bg_layer.wgsl` `scanline_scroll`): 224 entries, each `[[h, v]; 4]` indexed by
    /// BG layer 0-3. Empty = "no per-scanline data" → the compositor takes the fast
    /// path that bakes a single scroll into each tile's `screen_x`/`screen_y`. When a
    /// layer's per-scanline scroll varies from its base `scroll_x`/`scroll_y`, the
    /// Mode-1 compositor re-samples that layer per output row (the pyramid HDMA wave).
    pub bg_scroll_scanlines: Vec<[[u16; 2]; 4]>,
}

impl ModernFrame {
    pub(crate) fn scanout_brightness(&self) -> u8 {
        self.scanout_brightness_override.unwrap_or(self.brightness)
    }

    pub fn empty() -> Self {
        Self {
            width: MODERN_FRAME_WIDTH,
            height: MODERN_FRAME_HEIGHT,
            bg_layers: vec![
                ModernBgLayer::new(0),
                ModernBgLayer::new(1),
                ModernBgLayer::new(2),
                ModernBgLayer::new(3),
            ],
            sprites: Vec::new(),
            index_sprites: Vec::new(),
            hardware_startup_transient: None,
            bg3_vwf_glyph_runs: Vec::new(),
            dialogue_message_id: None,
            source_dialogue_ir: Vec::new(),
            dialogue_ir: Vec::new(),
            dialogue_layout: Vec::new(),
            dialogue_layout_vwf_glyph_runs: Vec::new(),
            dialogue_box_tilemap_palette: None,
            backdrop_color_rgba: [0, 0, 0, 0xff],
            brightness: 15,
            scanout_brightness_override: None,
            forced_blank: false,
            forced_blank_scanlines: 0,
            forced_blank_from_scanline: None,
            retain_active_display_history: false,
            cgram_rgba: [[0, 0, 0, 0xff]; 256],
            cgram_provenance: None,
            mode1_bg3_priority: false,
            screen_enabled_main: 0,
            screen_enabled_sub: 0,
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
            screen_windowed_main: 0,
            screen_windowed_sub: 0,
            window_scanlines: vec![[0u8; 4]; usize::from(MODERN_FRAME_HEIGHT)],
            main_tm_scanlines: vec![0xff; usize::from(MODERN_FRAME_HEIGHT)],
            mosaic_enabled: 0,
            mosaic_size: 1,
            bg_scroll_scanlines: Vec::new(),
        }
    }

    pub fn vwf_glyph_runs_for_draw(&self) -> &[ModernVwfGlyphRun] {
        &self.dialogue_layout_vwf_glyph_runs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernVwfGlyphRun {
    pub glyph_code: u16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub width: u8,
    pub dialogue_offset: Option<u16>,
    pub dialogue_ir_kind: Option<zelda3_dialogue::DialogueIrKind>,
    pub dialogue_color: Option<u8>,
}

impl ModernVwfGlyphRun {
    pub fn source_glyph_code(&self) -> u16 {
        match &self.dialogue_ir_kind {
            Some(zelda3_dialogue::DialogueIrKind::Glyph { code }) => u16::from(*code),
            _ => self.glyph_code,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernBgLayer {
    pub index: u8,
    pub enabled_main: bool,
    pub enabled_sub: bool,
    pub scroll_x: u16,
    pub scroll_y: u16,
    /// BG tilemap pixel dimensions (the SNES scroll torus period): 256 for a 32-tile
    /// map, 512 for a wide/tall (64-tile) map. Used by the per-scanline-scroll
    /// compositor to wrap-sample the layer. Default 256.
    pub wrap_w: u16,
    pub wrap_h: u16,
    pub tiles: Vec<ModernTileInstance>,
    pub index_tiles: Vec<ModernIndexTileInstance>,
    pub blend_mode: ModernBlendMode,
}

impl ModernBgLayer {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            enabled_main: false,
            enabled_sub: false,
            scroll_x: 0,
            scroll_y: 0,
            wrap_w: 256,
            wrap_h: 256,
            tiles: Vec::new(),
            index_tiles: Vec::new(),
            blend_mode: ModernBlendMode::Opaque,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexTileInstance {
    pub cell_id: u32,
    /// Logical source key for this draw instance when it is known.
    ///
    /// Cleaned/deduped BG cells can be shared by multiple ROM source tiles, so the
    /// source identity may need to live on the instance instead of the cell.
    pub source_key: u64,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub hflip: bool,
    pub vflip: bool,
    /// SNES BG per-tile priority bit (tilemap entry bit 13). Drives Mode 1
    /// compositing order: high-priority BG tiles render above lower-priority tiles
    /// of the other BG layers (and above the interleaved OBJ priorities).
    pub priority: bool,
}

/// A single palette-index OBJ (sprite) 8×8 tile instance, decoded from OAM.
///
/// `cell_id` indexes into the sprite index atlas (the UNFLIPPED pattern); the
/// renderer applies `hflip`/`vflip` to the 8×8 when drawing. Color comes from
/// `cgram_rgba[0x80 + palette*16 + index]`, index 0 transparent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexSpriteInstance {
    pub cell_id: u32,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,  // OBJ palette 0..7 (renderer maps to cgram 0x80 + palette*16)
    pub priority: u8, // 0..3 from OAM
    pub hflip: bool,
    pub vflip: bool,
    /// Per-output-row visibility under the SNES per-scanline OBJ range/time-over
    /// limit (bit r = row `screen_y + r` survives). The compositor skips rows whose
    /// bit is clear so it drops the same over-budget sprites the classic PPU does.
    /// `0xff` = all rows visible (the default when the budget is not modeled).
    pub row_mask: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileInstance {
    pub atlas_id: u32,
    /// Source rect in the atlas (in atlas pixels). For a scaled atlas this is the
    /// upscaled cell (e.g. 32x32 for an 8x8 tile at atlas_scale 4).
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    /// On-screen footprint of the tile (in screen pixels), distinct from the
    /// atlas source rect. Equals atlas_{width,height}_px / atlas_scale (e.g. 8x8).
    /// Renderers downsample the source rect into this footprint (nearest:
    /// block top-left), so a scaled atlas tile draws at its true size.
    pub screen_width_px: u16,
    pub screen_height_px: u16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub priority: u8,
    pub hflip: bool,
    pub vflip: bool,
    pub transparent_color_zero: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSpriteInstance {
    pub atlas_id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub priority: u8,
    pub hflip: bool,
    pub vflip: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernBlendMode {
    Opaque,
    Add,
    Subtract,
    HalfAdd,
    HalfSubtract,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn master_brightness_uses_the_full_fifteen_step_scale() {
        assert_eq!(apply_master_brightness(31, 15), 255);
        assert_eq!(apply_master_brightness(31, 7), 115);
        assert_eq!(apply_master_brightness(31, 0), 0);
    }

    #[test]
    fn indexed_tile_cgram_defaults_and_index_tiles_empty() {
        let frame = ModernFrame::empty();
        assert_eq!(frame.cgram_rgba[0], [0, 0, 0, 0xff]);
        assert_eq!(frame.cgram_rgba[255], [0, 0, 0, 0xff]);
        assert!(frame.bg_layers[0].index_tiles.is_empty());
    }

    #[test]
    fn empty_frame_has_no_index_sprites() {
        let frame = ModernFrame::empty();
        assert!(frame.index_sprites.is_empty());
    }

    #[test]
    fn modern_index_tile_instance_fields() {
        let inst = ModernIndexTileInstance {
            cell_id: 42,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: -8,
            screen_y: 16,
            palette: 3,
            hflip: true,
            vflip: false,
            priority: false,
        };
        assert_eq!(inst.cell_id, 42);
        assert_eq!(inst.source_key, crate::modern_hd_overrides::NO_SOURCE_KEY);
        assert_eq!(inst.screen_x, -8);
        assert_eq!(inst.palette, 3);
        assert!(inst.hflip);
        assert!(!inst.vflip);
    }

    #[test]
    fn modern_frame_defaults_to_fixed_game_resolution() {
        let frame = ModernFrame::empty();

        assert_eq!(frame.width, MODERN_FRAME_WIDTH);
        assert_eq!(frame.height, MODERN_FRAME_HEIGHT);
        assert_eq!(frame.bg_layers.len(), 4);
        assert!(frame.sprites.is_empty());
        assert_eq!(frame.backdrop_color_rgba, [0, 0, 0, 0xff]);
    }

    #[test]
    fn vwf_glyph_runs_for_draw_and_cull_never_fall_back_to_live_bg3_runs() {
        let mut frame = ModernFrame::empty();
        frame.bg3_vwf_glyph_runs.push(ModernVwfGlyphRun {
            glyph_code: 0x41,
            screen_x: 8,
            screen_y: 16,
            width: 8,
            dialogue_offset: None,
            dialogue_ir_kind: None,
            dialogue_color: None,
        });

        assert!(frame.vwf_glyph_runs_for_draw().is_empty());

        frame
            .dialogue_layout_vwf_glyph_runs
            .push(ModernVwfGlyphRun {
                glyph_code: 0x42,
                screen_x: 8,
                screen_y: 16,
                width: 8,
                dialogue_offset: Some(0),
                dialogue_ir_kind: Some(zelda3_dialogue::DialogueIrKind::Glyph { code: 0x42 }),
                dialogue_color: Some(2),
            });

        assert_eq!(frame.vwf_glyph_runs_for_draw()[0].source_glyph_code(), 0x42);
    }

    #[test]
    fn tile_instance_records_modern_render_inputs_without_snes_memory_addresses() {
        let tile = ModernTileInstance {
            atlas_id: 17,
            atlas_x_px: 64,
            atlas_y_px: 32,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 12,
            screen_y: 20,
            palette: 3,
            priority: 1,
            hflip: true,
            vflip: false,
            transparent_color_zero: true,
        };

        assert_eq!(tile.atlas_id, 17);
        assert_eq!(tile.screen_x, 12);
        assert_eq!(tile.screen_width_px, 8);
        assert!(tile.hflip);
        assert!(tile.transparent_color_zero);
    }
}
