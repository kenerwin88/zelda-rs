pub const MODERN_FRAME_WIDTH: u16 = 256;
pub const MODERN_FRAME_HEIGHT: u16 = 224;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernFrame {
    pub width: u16,
    pub height: u16,
    pub bg_layers: Vec<ModernBgLayer>,
    pub sprites: Vec<ModernSpriteInstance>,
    pub index_sprites: Vec<ModernIndexSpriteInstance>,
    pub backdrop_color_rgba: [u8; 4],
    pub brightness: u8,
    pub forced_blank: bool,
    pub cgram_rgba: [[u8; 4]; 256],
}

impl ModernFrame {
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
            backdrop_color_rgba: [0, 0, 0, 0xff],
            brightness: 15,
            forced_blank: false,
            cgram_rgba: [[0, 0, 0, 0xff]; 256],
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
            tiles: Vec::new(),
            index_tiles: Vec::new(),
            blend_mode: ModernBlendMode::Opaque,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexTileInstance {
    pub cell_id: u32,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub hflip: bool,
    pub vflip: bool,
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
            screen_x: -8,
            screen_y: 16,
            palette: 3,
            hflip: true,
            vflip: false,
        };
        assert_eq!(inst.cell_id, 42);
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
