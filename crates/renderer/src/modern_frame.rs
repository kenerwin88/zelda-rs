pub const MODERN_FRAME_WIDTH: u16 = 256;
pub const MODERN_FRAME_HEIGHT: u16 = 224;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernFrame {
    pub width: u16,
    pub height: u16,
    pub bg_layers: Vec<ModernBgLayer>,
    pub sprites: Vec<ModernSpriteInstance>,
    pub backdrop_color_rgba: [u8; 4],
    pub brightness: u8,
    pub forced_blank: bool,
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
            backdrop_color_rgba: [0, 0, 0, 0xff],
            brightness: 15,
            forced_blank: false,
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
            blend_mode: ModernBlendMode::Opaque,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileInstance {
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
        assert!(tile.hflip);
        assert!(tile.transparent_color_zero);
    }
}
