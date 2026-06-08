/// Per-scanline HDMA-driven register values for the post-process color math pass.
///
/// Captured by running HDMA channels for all 224 scanlines before rendering.
/// Index 0 = topmost visible scanline.
#[derive(Clone, Copy, Default)]
pub struct ScanlineRegs {
    /// Window 1 left boundary (column index, inclusive).
    pub window1_left: u8,
    /// Window 1 right boundary (column index, inclusive).
    pub window1_right: u8,
    /// Window 2 left boundary (column index, inclusive).
    pub window2_left: u8,
    /// Window 2 right boundary (column index, inclusive).
    pub window2_right: u8,
    /// Main-screen layer-enable register (TM/$212C) for this scanline.
    /// Bits: 0=BG1, 1=BG2, 2=BG3, 3=BG4, 4=OBJ. Captured from HDMA simulation.
    pub screen_enabled_main: u8,
    /// Per-BG horizontal scroll captured after HDMA for this scanline.
    pub bg_h_scroll: [u16; 4],
    /// Per-BG vertical scroll captured after HDMA for this scanline.
    pub bg_v_scroll: [u16; 4],
    /// Mode 7 matrix captured after HDMA for this scanline.
    pub mode7_matrix: [i16; 8],
}

/// Data bundle for one GPU-rendered frame, borrowing directly from `PpuState`.
///
/// Constructed by the caller each frame; zero copy of VRAM/OAM/CGRAM.
pub struct GpuFrame<'a> {
    /// VRAM: 0x8000 u16 words (64 KB). Tile CHR data and tilemap entries.
    pub vram: &'a [u16],
    /// CGRAM: 0x100 u16 words (256 palette entries), 15-bit BGR each.
    pub cgram: &'a [u16],
    /// OAM: 0x110 u16 words (sprite table, 128 sprites × 4 bytes + 16-byte high table).
    pub oam: &'a [u16],
    /// PPU background mode (0–7).
    pub mode: u8,
    /// Per-layer scroll, tilemap address, and tile CHR base address.
    pub bg: [BgLayerRegs; 4],
    /// Sprite (OBJ) register state.
    pub obj: ObjRegs,
    /// Mosaic enable bits and size from MOSAIC/$2106.
    pub mosaic_enabled: u8,
    pub mosaic_size: u8,
    /// Extra horizontal pixels included in CPU sprite selection for wide renders.
    pub extra_left_right: u8,
    /// Mode 7 transform and wrapping state.
    pub mode7: Mode7Regs,
    /// Which layers are enabled on main screen [0] and sub screen [1].
    pub screen_enabled: [u8; 2],
    /// Which layers use window masking on main screen [0] and sub screen [1].
    pub screen_windowed: [u8; 2],
    /// Overall brightness (0 = off, 15 = full).
    pub brightness: u8,
    /// If true, output is forced to all black (INIDISP forced-blank).
    pub forced_blank: bool,
    /// Bitmask of which layers participate in color math (bits 0-5 = BG1-4, OBJ, backdrop).
    pub math_enabled: u8,
    /// If true, color math subtracts the fixed color instead of adding.
    pub subtract_color: bool,
    /// If true, the post-math result is halved before brightness scaling.
    pub half_color: bool,
    /// Fixed color R component (5-bit, 0-31).
    pub fixed_color_r: u8,
    /// Fixed color G component (5-bit, 0-31).
    pub fixed_color_g: u8,
    /// Fixed color B component (5-bit, 0-31).
    pub fixed_color_b: u8,
    /// If true, add the sub-screen pixel instead of the fixed color.
    pub add_subscreen: bool,
    /// Clip mode (0=never, 1=outside window, 2=inside window, 3=always).
    pub clip_mode: u8,
    /// Prevent-math mode (0=never prevent, 1=outside window, 2=inside window, 3=always).
    pub prevent_math_mode: u8,
    /// Window flags for layer 5 (color math window): bits 0-3 = W1inv, W1en, W2inv, W2en.
    /// Extracted as `(windowsel >> 20) & 0xF` from the PPU windowsel register.
    pub windowsel_cm: u8,
    /// Window flags for all layers; 4 bits per layer.
    pub windowsel: u32,
    /// Per-scanline window 1 boundaries, from HDMA pre-simulation (224 entries).
    pub scanlines: Box<[ScanlineRegs; 224]>,
}

/// Per-BG-layer register snapshot (mirrors `snes::ppu::BgLayer`).
#[derive(Clone, Copy, Default)]
pub struct BgLayerRegs {
    pub h_scroll: u16,
    pub v_scroll: u16,
    pub tilemap_wider: bool,
    pub tilemap_higher: bool,
    /// VRAM word address of the tilemap data.
    pub tilemap_adr: u16,
    /// VRAM word address of the tile CHR base (tile 0 of this layer).
    pub tile_adr: u16,
}

/// Sprite (OBJ) register snapshot.
#[derive(Clone, Copy, Default)]
pub struct ObjRegs {
    /// VRAM word address of the first sprite CHR page.
    pub tile_adr1: u16,
    /// VRAM word address of the second sprite CHR page.
    pub tile_adr2: u16,
    /// OBJ size selector (encodes small/large sprite size pair).
    pub obj_size: u8,
}

/// Mode 7 register snapshot.
#[derive(Clone, Copy, Default)]
pub struct Mode7Regs {
    pub matrix: [i16; 8],
    pub large_field: bool,
    pub char_fill: bool,
    pub x_flip: bool,
    pub y_flip: bool,
    pub ext_bg_always_zero: bool,
}
