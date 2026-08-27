//! PPU state + register R/W. Port of `zelda3/snes/ppu.c`.
//!
//! The scanline renderer covers the C new-renderer path, including mode 1
//! backgrounds, sprites, windows, color math, mosaic, and mode 7.

use crate::consts::{PPU_EXTRA_LEFT_RIGHT, PPU_X_PIXELS};

bitflags::bitflags! {
    /// Mirrors the `kPpuRenderFlags_*` enum from `snes/ppu.h`.
    #[derive(Default, Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct PpuRenderFlags: u32 {
        const NEW_RENDERER      = 1;
        const MODE7_4X4        = 2;
        const HEIGHT_240       = 4;
        const NO_SPRITE_LIMITS = 8;
    }
}

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BgLayer {
    pub h_scroll: u16,
    pub v_scroll: u16,
    pub tilemap_wider: bool,
    pub tilemap_higher: bool,
    pub tilemap_adr: u16,
    pub tile_adr: u16,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PpuPixelPrioBufs {
    pub data: Vec<u16>, // length PPU_X_PIXELS
}

impl Default for PpuPixelPrioBufs {
    fn default() -> Self {
        Self {
            data: vec![0; PPU_X_PIXELS],
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PpuState {
    pub line_has_sprites: bool,
    pub last_brightness_mult: u8,
    pub last_mosaic_modulo: u8,
    pub render_flags: PpuRenderFlags,
    pub render_pitch: u32,
    pub render_buffer: Option<Vec<u8>>,
    pub extra_left_cur: u8,
    pub extra_right_cur: u8,
    pub extra_left_right: u8,
    pub extra_bottom_cur: u8,
    pub mode7_perspective_low: f32,
    pub mode7_perspective_high: f32,

    pub screen_enabled: [u8; 2],
    pub screen_windowed: [u8; 2],
    pub mosaic_enabled: u8,
    pub mosaic_size: u8,

    pub obj_tile_adr1: u16,
    pub obj_tile_adr2: u16,
    pub obj_size: u8,

    pub window1_left: u8,
    pub window1_right: u8,
    pub window2_left: u8,
    pub window2_right: u8,
    pub windowsel: u32,

    pub clip_mode: u8,
    pub prevent_math_mode: u8,
    pub add_subscreen: bool,
    pub subtract_color: bool,
    pub half_color: bool,
    pub math_enabled: u8,
    pub fixed_color_r: u8,
    pub fixed_color_g: u8,
    pub fixed_color_b: u8,

    pub forced_blank: bool,
    /// Visible scanline prefix rendered while INIDISP remains forced blank
    /// because the preceding VBlank workload overran into active display.
    #[serde(default)]
    pub forced_blank_scanlines: u8,
    /// First visible scanline forced blank by an active-display INIDISP write.
    /// `None` means the frame has no mid-frame transition into forced blank.
    #[serde(default)]
    pub forced_blank_from_scanline: Option<u8>,
    /// The visible prefix predates an NMI display-memory publication and must
    /// be retained from the preceding physical scanout surface. A main-thread
    /// INIDISP write can also produce `forced_blank_from_scanline`, but does
    /// not require this retention because its prefix uses the current frame's
    /// already-published VRAM and registers.
    #[serde(default)]
    pub retain_active_display_history: bool,
    pub brightness: u8,
    /// One-frame master-brightness generation for an active scanout whose
    /// visible rows precede the final INIDISP register write. This is
    /// presentation metadata, never a VRAM/CGRAM composition path.
    #[serde(default)]
    pub scanout_brightness_override: Option<u8>,
    /// Top rows removed when the completed hardware scanout is published as
    /// the fixed 224-line host surface. This is presentation geometry rather
    /// than a live PPU register.
    #[serde(skip)]
    pub scanout_top_crop: u8,
    /// Raw low nibble of BGMODE/$2105. Bits 0..2 select the background mode;
    /// the exact value $09 raises BG3 priority in Mode 1. Retaining bit 3 in
    /// this existing byte preserves positional save-state compatibility.
    pub mode: u8,

    pub vram_pointer: u16,
    pub vram_increment: u16,
    pub vram_increment_on_high: bool,
    pub cgram_pointer: u8,
    pub cgram_second_write: bool,
    pub cgram_buffer: u8,
    pub oam_adr: u16,
    pub oam_second_write: bool,
    pub oam_buffer: u8,

    /// Hardware H/V beam counters captured by a read from SLHV ($2137).
    /// These latches are CPU-visible PPU state, independent of rendering.
    #[serde(default)]
    h_counter_latched: u16,
    #[serde(default)]
    v_counter_latched: u16,
    #[serde(default)]
    h_counter_high_byte: bool,
    #[serde(default)]
    v_counter_high_byte: bool,

    pub bg_layer: [BgLayer; 4],
    pub scroll_prev: u8,
    pub scroll_prev2: u8,

    pub m7_matrix: [i16; 8],
    pub m7_prev: u8,
    pub m7_large_field: bool,
    pub m7_char_fill: bool,
    pub m7_x_flip: bool,
    pub m7_y_flip: bool,
    pub m7_ext_bg_always_zero: bool,
    pub m7_start_x: i32,
    pub m7_start_y: i32,

    pub oam: Vec<u16>,                 // 0x110
    pub brightness_mult: Vec<u8>,      // 32 + 31
    pub brightness_mult_half: Vec<u8>, // 32 * 2
    pub cgram: Vec<u16>,               // 0x100
    pub mosaic_modulo: Vec<u8>,        // PPU_X_PIXELS
    pub color_map_rgb: Vec<u32>,       // 256
    pub bg_buffers: [PpuPixelPrioBufs; 2],
    pub obj_buffer: PpuPixelPrioBufs,
    pub obj_fetch_buffer: PpuPixelPrioBufs,
    pub obj_fetch_has_sprites: bool,
    /// Optional decoded BG-CHR generation. An exact NMI DMA receipt can update
    /// the emulator's decoder cache at its owner boundary while raw VRAM for
    /// the composed field remains independently owned.
    #[serde(default)]
    pub bg_vram_latch: Option<Vec<u16>>,
    pub obj_vram_latch: Option<Vec<u16>>,
    pub obj_previous_frame_vram: Option<Vec<u16>>,
    pub vram: Vec<u16>, // 0x8000
}

#[derive(Clone, Copy, Debug)]
struct PpuWindows {
    edges: [i16; 6],
    nr: u8,
    bits: u8,
}

const WINDOW_1_INVERSED: u32 = 1;
const WINDOW_1_ENABLED: u32 = 2;
const WINDOW_2_INVERSED: u32 = 4;
const WINDOW_2_ENABLED: u32 = 8;
const SPRITE_SIZES: [[u8; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];
const BIT_DEPTHS_PER_MODE: [[usize; 4]; 10] = [
    [2, 2, 2, 2],
    [4, 4, 2, 5],
    [4, 4, 5, 5],
    [8, 4, 5, 5],
    [8, 2, 5, 5],
    [4, 2, 5, 5],
    [4, 5, 5, 5],
    [8, 5, 5, 5],
    [4, 4, 2, 5],
    [8, 7, 5, 5],
];
const LAYERS_PER_MODE: [[usize; 12]; 10] = [
    [4, 0, 1, 4, 0, 1, 4, 2, 3, 4, 2, 3],
    [4, 0, 1, 4, 0, 1, 4, 2, 4, 2, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5],
    [4, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5, 5],
    [2, 4, 0, 1, 4, 0, 1, 4, 4, 2, 5, 5],
    [4, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5, 5],
];
const PRIORITIES_PER_MODE: [[usize; 12]; 10] = [
    [3, 1, 1, 2, 0, 0, 1, 1, 1, 0, 0, 0],
    [3, 1, 1, 2, 0, 0, 1, 1, 0, 0, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5],
    [3, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5, 5],
    [1, 3, 1, 1, 2, 0, 0, 1, 0, 0, 5, 5],
    [3, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5, 5],
];
const LAYER_COUNT_PER_MODE: [usize; 10] = [12, 10, 8, 8, 8, 8, 6, 5, 10, 7];

impl Default for PpuState {
    fn default() -> Self {
        Self {
            line_has_sprites: false,
            last_brightness_mult: 0,
            last_mosaic_modulo: 0,
            render_flags: PpuRenderFlags::default(),
            render_pitch: 0,
            render_buffer: None,
            extra_left_cur: 0,
            extra_right_cur: 0,
            extra_left_right: PPU_EXTRA_LEFT_RIGHT as u8,
            extra_bottom_cur: 0,
            mode7_perspective_low: 0.0,
            mode7_perspective_high: 0.0,
            screen_enabled: [0; 2],
            screen_windowed: [0; 2],
            mosaic_enabled: 0,
            mosaic_size: 0,
            obj_tile_adr1: 0,
            obj_tile_adr2: 0,
            obj_size: 0,
            window1_left: 1,
            window1_right: 0,
            window2_left: 1,
            window2_right: 0,
            windowsel: 0,
            clip_mode: 0,
            prevent_math_mode: 0,
            add_subscreen: false,
            subtract_color: false,
            half_color: false,
            math_enabled: 0,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            forced_blank: false,
            forced_blank_scanlines: 0,
            forced_blank_from_scanline: None,
            retain_active_display_history: false,
            brightness: 0,
            scanout_brightness_override: None,
            scanout_top_crop: 0,
            mode: 0,
            vram_pointer: 0,
            vram_increment: 1,
            vram_increment_on_high: false,
            cgram_pointer: 0,
            cgram_second_write: false,
            cgram_buffer: 0,
            oam_adr: 0,
            oam_second_write: false,
            oam_buffer: 0,
            h_counter_latched: 0,
            v_counter_latched: 0,
            h_counter_high_byte: false,
            v_counter_high_byte: false,
            bg_layer: [BgLayer::default(); 4],
            scroll_prev: 0,
            scroll_prev2: 0,
            m7_matrix: [0; 8],
            m7_prev: 0,
            m7_large_field: false,
            m7_char_fill: false,
            m7_x_flip: false,
            m7_y_flip: false,
            m7_ext_bg_always_zero: false,
            m7_start_x: 0,
            m7_start_y: 0,
            oam: vec![0; 0x110],
            brightness_mult: vec![0; 32 + 31],
            brightness_mult_half: vec![0; 32 * 2],
            cgram: vec![0; 0x100],
            mosaic_modulo: vec![0; PPU_X_PIXELS],
            color_map_rgb: vec![0; 256],
            bg_buffers: [PpuPixelPrioBufs::default(), PpuPixelPrioBufs::default()],
            obj_buffer: PpuPixelPrioBufs::default(),
            obj_fetch_buffer: PpuPixelPrioBufs::default(),
            obj_fetch_has_sprites: false,
            bg_vram_latch: None,
            obj_vram_latch: None,
            obj_previous_frame_vram: None,
            vram: vec![0; 0x8000],
        }
    }
}

impl PpuState {
    pub const C_SAVELOAD_SIZE: usize = 0x8000 * 2 + 10 + 512 + 556 + 520 + 4 * 12 + 123;

    pub fn new() -> Self {
        Self::default()
    }

    /// Hardware background mode selected by BGMODE bits 0..2.
    pub fn bg_mode(&self) -> u8 {
        self.mode & 0x07
    }

    /// Snes9x raises BG3 priority only for the exact BGMODE low nibble `$09`.
    pub fn mode1_bg3_priority(&self) -> bool {
        self.mode == 0x09
    }

    /// `ppu_reset` — zero VRAM/CGRAM/OAM, set objTileAdrs, force blank.
    pub fn reset(&mut self) {
        for v in &mut self.vram {
            *v = 0;
        }
        for v in &mut self.cgram {
            *v = 0;
        }
        for v in &mut self.oam {
            *v = 0;
        }
        for v in &mut self.obj_buffer.data {
            *v = 0;
        }
        for v in &mut self.obj_fetch_buffer.data {
            *v = 0;
        }
        self.obj_fetch_has_sprites = false;
        self.bg_vram_latch = None;
        self.obj_vram_latch = None;
        self.obj_previous_frame_vram = None;
        self.last_brightness_mult = 0xff;
        self.last_mosaic_modulo = 0xff;
        self.extra_left_cur = 0;
        self.extra_right_cur = 0;
        self.extra_bottom_cur = 0;
        self.vram_pointer = 0;
        self.vram_increment_on_high = false;
        self.vram_increment = 1;
        self.cgram_pointer = 0;
        self.cgram_second_write = false;
        self.cgram_buffer = 0;
        self.oam_adr = 0;
        self.oam_second_write = false;
        self.oam_buffer = 0;
        self.h_counter_latched = 0;
        self.v_counter_latched = 0;
        self.h_counter_high_byte = false;
        self.v_counter_high_byte = false;
        self.obj_tile_adr1 = 0x4000;
        self.obj_tile_adr2 = 0x5000;
        self.obj_size = 0;
        for bg in &mut self.bg_layer {
            *bg = BgLayer::default();
        }
        self.scroll_prev = 0;
        self.scroll_prev2 = 0;
        self.mosaic_size = 1;
        self.screen_enabled = [0; 2];
        self.screen_windowed = [0; 2];
        self.m7_matrix = [0; 8];
        self.m7_prev = 0;
        self.m7_large_field = true;
        self.m7_char_fill = false;
        self.m7_x_flip = false;
        self.m7_y_flip = false;
        self.m7_ext_bg_always_zero = false;
        self.m7_start_x = 0;
        self.m7_start_y = 0;
        self.windowsel = 0;
        // Snes9x initializes both windows to the canonical empty interval.
        // The game can enable a window before ever writing its edges.
        self.window1_left = 1;
        self.window1_right = 0;
        self.window2_left = 1;
        self.window2_right = 0;
        self.clip_mode = 0;
        self.prevent_math_mode = 0;
        self.add_subscreen = false;
        self.subtract_color = false;
        self.half_color = false;
        self.math_enabled = 0;
        self.fixed_color_r = 0;
        self.fixed_color_g = 0;
        self.fixed_color_b = 0;
        self.forced_blank = true;
        self.brightness = 0;
        self.mode = 0;
    }

    /// Byte layout used by C `ppu_saveload`.
    ///
    /// C persists VRAM, CGRAM, and the four `BgLayer` tilemap snapshot fields
    /// with fixed padding blocks in between. Other renderer/register state is
    /// intentionally not part of that C saveload surface.
    pub fn save_c_saveload(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::C_SAVELOAD_SIZE);
        for i in 0..0x8000 {
            out.extend_from_slice(&self.vram.get(i).copied().unwrap_or(0).to_le_bytes());
        }
        out.extend_from_slice(&[0; 10]);
        for i in 0..0x100 {
            out.extend_from_slice(&self.cgram.get(i).copied().unwrap_or(0).to_le_bytes());
        }
        out.extend_from_slice(&[0; 556]);
        out.extend_from_slice(&[0; 520]);
        for bg in &self.bg_layer {
            out.extend_from_slice(&[0; 4]);
            out.push(bg.tilemap_wider as u8);
            out.push(bg.tilemap_higher as u8);
            out.extend_from_slice(&bg.tilemap_adr.to_le_bytes());
            out.extend_from_slice(&[0; 4]);
        }
        out.extend_from_slice(&[0; 123]);
        debug_assert_eq!(out.len(), Self::C_SAVELOAD_SIZE);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != Self::C_SAVELOAD_SIZE {
            return Err(format!(
                "invalid PPU saveload size {}, expected {}",
                data.len(),
                Self::C_SAVELOAD_SIZE
            ));
        }
        let mut pos = 0usize;
        if self.vram.len() != 0x8000 {
            self.vram.resize(0x8000, 0);
        }
        for v in &mut self.vram[..0x8000] {
            *v = u16::from_le_bytes([data[pos], data[pos + 1]]);
            pos += 2;
        }
        pos += 10;
        if self.cgram.len() != 0x100 {
            self.cgram.resize(0x100, 0);
        }
        for c in &mut self.cgram[..0x100] {
            *c = u16::from_le_bytes([data[pos], data[pos + 1]]);
            pos += 2;
        }
        pos += 556;
        pos += 520;
        for bg in &mut self.bg_layer {
            pos += 4;
            bg.tilemap_wider = data[pos] != 0;
            bg.tilemap_higher = data[pos + 1] != 0;
            bg.tilemap_adr = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            pos += 8;
        }
        Ok(())
    }

    pub fn latch_counters(&mut self, h_counter: u16, v_counter: u16) {
        self.h_counter_latched = h_counter;
        self.v_counter_latched = v_counter;
    }

    fn read_latched_counter(value: u16, high_byte: &mut bool) -> u8 {
        let result = if *high_byte {
            ((value >> 8) & 1) as u8
        } else {
            value as u8
        };
        *high_byte = !*high_byte;
        result
    }

    /// `ppu_read` ($2134..$213f), including the CPU-visible beam-counter
    /// latches used by raster synchronization loops.
    pub fn read(&mut self, adr: u8) -> u8 {
        match adr {
            0x34 | 0x35 | 0x36 => {
                let result =
                    (self.m7_matrix[0] as i32).wrapping_mul((self.m7_matrix[1] >> 8) as i32);
                ((result as u32) >> (8 * (adr - 0x34) as u32) & 0xff) as u8
            }
            0x3c => {
                Self::read_latched_counter(self.h_counter_latched, &mut self.h_counter_high_byte)
            }
            0x3d => {
                Self::read_latched_counter(self.v_counter_latched, &mut self.v_counter_high_byte)
            }
            0x3f => {
                self.h_counter_high_byte = false;
                self.v_counter_high_byte = false;
                0xff
            }
            _ => 0xff,
        }
    }

    /// `ppu_write` ($2100..$2133). Every match arm mirrors the C switch.
    pub fn write(&mut self, adr: u8, val: u8) {
        match adr {
            0x00 => {
                // INIDISP
                self.brightness = val & 0xf;
                self.forced_blank = val & 0x80 != 0;
            }
            0x01 => {
                // OBJ tile bit 8 always advances to the second 256-tile page;
                // OBSEL's name-select field adds a further configurable offset.
                // Convert both byte-addressed hardware fields to VRAM words.
                self.obj_tile_adr1 = u16::from(val & 3) << 13;
                self.obj_tile_adr2 = self
                    .obj_tile_adr1
                    .wrapping_add(u16::from(((val >> 3) & 3) + 1) << 12);
                self.obj_size = (val >> 5) & 7;
            }
            0x02 => {
                self.oam_adr = (self.oam_adr & 0xff00) | val as u16;
                self.oam_second_write = false;
            }
            0x03 => {
                debug_assert_eq!(val & 0x80, 0);
                self.oam_adr = (self.oam_adr & 0x00ff) | (((val & 1) as u16) << 8);
                self.oam_second_write = false;
            }
            0x04 => {
                if !self.oam_second_write {
                    self.oam_buffer = val;
                } else {
                    if (self.oam_adr as usize) < 0x110 {
                        self.oam[self.oam_adr as usize] =
                            ((val as u16) << 8) | self.oam_buffer as u16;
                        self.oam_adr = self.oam_adr.wrapping_add(1);
                    }
                }
                self.oam_second_write = !self.oam_second_write;
            }
            0x05 => {
                // BGMODE
                self.mode = val & 0x0f;
                // The title route begins in the reset-time mode 0 before it
                // switches to the game modes. All 3-bit hardware modes are
                // valid register values; rendering chooses the supported
                // composition path later.
            }
            0x06 => {
                // MOSAIC
                self.mosaic_size = (val >> 4) + 1;
                self.mosaic_enabled = if self.mosaic_size > 1 { val } else { 0 };
            }
            0x07 | 0x08 | 0x09 | 0x0a => {
                let i = (adr - 0x07) as usize;
                self.bg_layer[i].tilemap_wider = val & 0x1 != 0;
                self.bg_layer[i].tilemap_higher = val & 0x2 != 0;
                self.bg_layer[i].tilemap_adr = ((val & 0xfc) as u16) << 8;
            }
            0x0b => {
                self.bg_layer[0].tile_adr = ((val & 0x0f) as u16) << 12;
                self.bg_layer[1].tile_adr = ((val & 0xf0) as u16) << 8;
            }
            0x0c => {
                self.bg_layer[2].tile_adr = ((val & 0x0f) as u16) << 12;
                self.bg_layer[3].tile_adr = ((val & 0xf0) as u16) << 8;
            }
            0x0d => {
                // BG1HOFS + M7HOFS
                self.m7_matrix[6] = (((val as i32) << 8) | self.m7_prev as i32) as i16 & 0x1fff;
                self.m7_prev = val;
                // fallthrough to BG-HOFS write
                self.write_bg_hofs(adr, val);
            }
            0x0f | 0x11 | 0x13 => {
                self.write_bg_hofs(adr, val);
            }
            0x0e => {
                // BG1VOFS + M7VOFS
                self.m7_matrix[7] = (((val as i32) << 8) | self.m7_prev as i32) as i16 & 0x1fff;
                self.m7_prev = val;
                self.write_bg_vofs(adr, val);
            }
            0x10 | 0x12 | 0x14 => {
                self.write_bg_vofs(adr, val);
            }
            0x15 => {
                // VMAIN
                self.vram_increment = match val & 3 {
                    0 => 1,
                    1 => 32,
                    _ => 128,
                };
                debug_assert_eq!((val & 0xc) >> 2, 0, "VMAIN remap-mode != 0");
                self.vram_increment_on_high = val & 0x80 != 0;
            }
            0x16 => {
                self.vram_pointer = (self.vram_pointer & 0xff00) | val as u16;
            }
            0x17 => {
                self.vram_pointer = (self.vram_pointer & 0x00ff) | ((val as u16) << 8);
            }
            0x18 => {
                let idx = (self.vram_pointer & 0x7fff) as usize;
                self.vram[idx] = (self.vram[idx] & 0xff00) | val as u16;
                if !self.vram_increment_on_high {
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
            }
            0x19 => {
                let idx = (self.vram_pointer & 0x7fff) as usize;
                self.vram[idx] = (self.vram[idx] & 0x00ff) | ((val as u16) << 8);
                if self.vram_increment_on_high {
                    self.vram_pointer = self.vram_pointer.wrapping_add(self.vram_increment);
                }
            }
            0x1a => {
                debug_assert_eq!(val, 0x80, "M7SEL expected 0x80");
                self.m7_large_field = val & 0x80 != 0;
                self.m7_char_fill = val & 0x40 != 0;
                self.m7_y_flip = val & 0x2 != 0;
                self.m7_x_flip = val & 0x1 != 0;
            }
            0x1b | 0x1c | 0x1d | 0x1e => {
                let i = (adr - 0x1b) as usize;
                self.m7_matrix[i] = (((val as i32) << 8) | self.m7_prev as i32) as i16;
                self.m7_prev = val;
            }
            0x1f | 0x20 => {
                let i = (adr - 0x1b) as usize;
                self.m7_matrix[i] = (((val as i32) << 8) | self.m7_prev as i32) as i16 & 0x1fff;
                self.m7_prev = val;
            }
            0x21 => {
                self.cgram_pointer = val;
                self.cgram_second_write = false;
            }
            0x22 => {
                if !self.cgram_second_write {
                    self.cgram_buffer = val;
                } else {
                    self.cgram[self.cgram_pointer as usize] =
                        ((val as u16) << 8) | self.cgram_buffer as u16;
                    self.cgram_pointer = self.cgram_pointer.wrapping_add(1);
                }
                self.cgram_second_write = !self.cgram_second_write;
            }
            0x23 => {
                self.windowsel = (self.windowsel & !0xff) | val as u32;
            }
            0x24 => {
                self.windowsel = (self.windowsel & !0xff00) | ((val as u32) << 8);
            }
            0x25 => {
                self.windowsel = (self.windowsel & !0xff0000) | ((val as u32) << 16);
            }
            0x26 => self.window1_left = val,
            0x27 => self.window1_right = val,
            0x28 => self.window2_left = val,
            0x29 => self.window2_right = val,
            0x2a => debug_assert_eq!(val, 0, "WBGLOG nonzero"),
            0x2b => debug_assert_eq!(val, 0, "WOBJLOG nonzero"),
            0x2c => self.screen_enabled[0] = val,
            0x2d => self.screen_enabled[1] = val,
            0x2e => self.screen_windowed[0] = val,
            0x2f => self.screen_windowed[1] = val,
            0x30 => {
                debug_assert_eq!(val & 1, 0, "CGWSEL directColor expected 0");
                self.add_subscreen = val & 0x2 != 0;
                self.prevent_math_mode = (val & 0x30) >> 4;
                self.clip_mode = (val & 0xc0) >> 6;
            }
            0x31 => {
                self.subtract_color = val & 0x80 != 0;
                self.half_color = val & 0x40 != 0;
                self.math_enabled = val & 0x3f;
            }
            0x32 => {
                if val & 0x80 != 0 {
                    self.fixed_color_b = val & 0x1f;
                }
                if val & 0x40 != 0 {
                    self.fixed_color_g = val & 0x1f;
                }
                if val & 0x20 != 0 {
                    self.fixed_color_r = val & 0x1f;
                }
            }
            0x33 => {
                debug_assert_eq!(val, 0, "SETINI expected 0");
                self.m7_ext_bg_always_zero = val & 0x40 != 0;
            }
            _ => {}
        }
    }

    fn write_bg_hofs(&mut self, adr: u8, val: u8) {
        let i = ((adr - 0xd) / 2) as usize;
        self.bg_layer[i].h_scroll = ((val as u16) << 8)
            | (self.scroll_prev as u16 & 0xf8)
            | (self.scroll_prev2 as u16 & 0x7);
        self.scroll_prev = val;
        self.scroll_prev2 = val;
    }

    fn write_bg_vofs(&mut self, adr: u8, val: u8) {
        let i = ((adr - 0xe) / 2) as usize;
        self.bg_layer[i].v_scroll = ((val as u16) << 8) | self.scroll_prev as u16;
        self.scroll_prev = val;
    }

    pub fn current_render_scale(&self, render_flags: PpuRenderFlags) -> i32 {
        let hq = self.bg_mode() == 7
            && !self.forced_blank
            && render_flags.contains(PpuRenderFlags::MODE7_4X4 | PpuRenderFlags::NEW_RENDERER);
        if hq {
            4
        } else {
            1
        }
    }

    /// Refresh the INIDISP lookup table used by every scanout consumer.
    pub fn refresh_brightness_cache(&mut self) {
        let brightness = self.scanout_brightness();
        if brightness == self.last_brightness_mult {
            return;
        }
        self.last_brightness_mult = brightness;
        for i in 0..32 {
            // INIDISP is a linear 0..15 master level: zero is black and 15 is
            // the unattenuated five-bit component. Round to the nearest DAC
            // step so the lower half of the fade does not remain one level too
            // bright (for example, white at level 7 is component 14, not 15).
            let scaled = (i * brightness as usize + 7) / 15;
            let value = ((scaled << 3) | (scaled >> 2)) as u8;
            self.brightness_mult[i] = value;
            self.brightness_mult_half[i * 2] = value;
            self.brightness_mult_half[i * 2 + 1] = value;
        }
        for i in 32..self.brightness_mult.len() {
            self.brightness_mult[i] = self.brightness_mult[31];
        }
    }

    pub fn scanout_brightness(&self) -> u8 {
        self.scanout_brightness_override.unwrap_or(self.brightness)
    }

    fn snes9x_gamma_component(component: usize) -> u8 {
        let expanded = ((component << 3) | (component >> 2)) as u16;
        let wide = (expanded << 8) | expanded;
        if wide > 0x7fff {
            return (wide >> 8) as u8;
        }
        let corrected = 32767.0 * ((wide as f64) / 32767.0).powf(1.5);
        ((corrected as u16) >> 8) as u8
    }

    pub fn begin_drawing(
        &mut self,
        pixel_buffer: &mut [u8],
        pitch: usize,
        render_flags: PpuRenderFlags,
    ) {
        self.render_flags = render_flags;
        self.render_pitch = pitch as u32;
        let mut render_buffer = pixel_buffer.to_vec();
        for pixel in render_buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&0xff000000u32.to_le_bytes());
        }
        self.render_buffer = Some(render_buffer);
        self.refresh_brightness_cache();

        if self.current_render_scale(self.render_flags) == 4 {
            for i in 0..256 {
                let color = self.cgram[i] as usize;
                self.color_map_rgb[i] = (self.brightness_mult[color & 0x1f] as u32) << 16
                    | (self.brightness_mult[(color >> 5) & 0x1f] as u32) << 8
                    | self.brightness_mult[(color >> 10) & 0x1f] as u32;
            }
        }
    }

    pub fn finish_drawing(&mut self) {
        self.obj_previous_frame_vram = Some(
            self.obj_vram_latch
                .take()
                .unwrap_or_else(|| self.vram.clone()),
        );
    }

    fn refresh_mosaic_modulo(&mut self) {
        if self.mosaic_size == self.last_mosaic_modulo {
            return;
        }
        let modulo = self.mosaic_size.max(1);
        self.last_mosaic_modulo = modulo;
        let mut j = 0u8;
        for i in 0..self.mosaic_modulo.len() {
            self.mosaic_modulo[i] = (i as u8).wrapping_sub(j);
            j = if j.wrapping_add(1) == modulo {
                0
            } else {
                j.wrapping_add(1)
            };
        }
    }

    fn clear_backdrop(&mut self) {
        self.obj_buffer.data.fill(0x0500);
    }

    fn backdrop_rgb(&self) -> u32 {
        let color = self.cgram[0] as usize;
        let r = self.brightness_mult[color & 0x1f] as u32;
        let g = self.brightness_mult[(color >> 5) & 0x1f] as u32;
        let b = self.brightness_mult[(color >> 10) & 0x1f] as u32;
        (r << 16) | (g << 8) | b
    }

    fn bgr555_to_rgb(&self, color: u16) -> u32 {
        let color = color as usize;
        let r = self.brightness_mult[color & 0x1f] as u32;
        let g = self.brightness_mult[(color >> 5) & 0x1f] as u32;
        let b = self.brightness_mult[(color >> 10) & 0x1f] as u32;
        (r << 16) | (g << 8) | b
    }

    fn fill_render_line(&mut self, line: usize, color: u32) {
        let pitch = self.render_pitch as usize;
        if pitch == 0 {
            return;
        }
        let width = 256usize + self.extra_left_right as usize * 2;
        let Some(buffer) = self.render_buffer.as_mut() else {
            return;
        };
        let start = line.saturating_mul(pitch);
        let end = start.saturating_add(width.saturating_mul(4));
        if end > buffer.len() {
            return;
        }
        let bytes = color.to_le_bytes();
        for pixel in buffer[start..end].chunks_exact_mut(4) {
            pixel.copy_from_slice(&bytes);
        }
    }

    fn screen_enabled_for(&self, sub: bool, layer: usize) -> bool {
        self.screen_enabled[sub as usize] & (1 << layer) != 0
    }

    fn ppu_windows_clear(&self, layer: usize) -> PpuWindows {
        let mut win = PpuWindows {
            edges: [0; 6],
            nr: 1,
            bits: 0,
        };
        if layer == 2 {
            win.edges[0] = 0;
            win.edges[1] = 256;
        } else {
            win.edges[0] = -(self.extra_left_cur as i16);
            win.edges[1] = 256 + self.extra_right_cur as i16;
        }
        win
    }

    fn insert_window_edge(win: &mut PpuWindows, edge: i16, start: usize, nr: &mut usize) -> usize {
        let mut i = start;
        while i <= *nr {
            if edge == win.edges[i] {
                return i;
            }
            if edge < win.edges[i] {
                let mut j = *nr;
                *nr += 1;
                loop {
                    win.edges[j + 1] = win.edges[j];
                    if j == i {
                        break;
                    }
                    j -= 1;
                }
                win.edges[i] = edge;
                return i;
            }
            i += 1;
        }
        i
    }

    fn ppu_windows_calc(&self, layer: usize) -> PpuWindows {
        let mut win = self.ppu_windows_clear(layer);
        let winflags = self.windowsel >> (layer * 4);
        let mut nr = 1usize;
        let window_right = win.edges[1];

        let w1_enabled =
            winflags & WINDOW_1_ENABLED != 0 && self.window1_left <= self.window1_right;
        if w1_enabled {
            let left = self.window1_left as i16;
            let right = self.window1_right as i16 + 1;
            if left > win.edges[0] {
                win.edges[nr] = left;
                nr += 1;
                win.edges[nr] = window_right;
            }
            if right < window_right {
                win.edges[nr] = right;
                nr += 1;
                win.edges[nr] = window_right;
            }
        }

        let w2_enabled =
            winflags & WINDOW_2_ENABLED != 0 && self.window2_left <= self.window2_right;
        if w2_enabled {
            let i = Self::insert_window_edge(&mut win, self.window2_left as i16, 0, &mut nr);
            Self::insert_window_edge(&mut win, self.window2_right as i16 + 1, i, &mut nr);
        }

        win.nr = nr as u8;

        let mut w1_bits = 0u8;
        let mut w2_bits = 0u8;
        if w1_enabled {
            let left = self.window1_left as i16;
            let right = self.window1_right as i16 + 1;
            let i = win.edges[..=nr]
                .iter()
                .position(|&edge| edge == left)
                .unwrap_or(0);
            let j = win.edges[..=nr]
                .iter()
                .position(|&edge| edge == right)
                .unwrap_or(i);
            w1_bits = (((1u16 << (j - i)) - 1) << i) as u8;
        }
        if winflags & (WINDOW_1_ENABLED | WINDOW_1_INVERSED) == WINDOW_1_ENABLED | WINDOW_1_INVERSED
        {
            w1_bits = !w1_bits;
        }
        if w2_enabled {
            let left = self.window2_left as i16;
            let right = self.window2_right as i16 + 1;
            let i = win.edges[..=nr]
                .iter()
                .position(|&edge| edge == left)
                .unwrap_or(0);
            let j = win.edges[..=nr]
                .iter()
                .position(|&edge| edge == right)
                .unwrap_or(i);
            w2_bits = (((1u16 << (j - i)) - 1) << i) as u8;
        }
        if winflags & (WINDOW_2_ENABLED | WINDOW_2_INVERSED) == WINDOW_2_ENABLED | WINDOW_2_INVERSED
        {
            w2_bits = !w2_bits;
        }
        win.bits = w1_bits | w2_bits;
        win
    }

    fn windows_for_layer(&self, sub: bool, layer: usize) -> PpuWindows {
        if self.screen_windowed[sub as usize] & (1 << layer) != 0 {
            self.ppu_windows_calc(layer)
        } else {
            self.ppu_windows_clear(layer)
        }
    }

    fn sprite_prio_to_prio(prio: u16, level6: bool) -> u16 {
        ((prio * 4 + 2) * 16) + 4 + u16::from(level6) * 2
    }

    fn bg_tile_entry(&self, layer: usize, x: u16, y: u16) -> u16 {
        let bg = self.bg_layer[layer];
        let mut sc_offs = bg.tilemap_adr.wrapping_add(((y >> 3) & 0x1f) << 5);
        if y & 0x100 != 0 && bg.tilemap_higher {
            sc_offs = sc_offs.wrapping_add(if bg.tilemap_wider { 0x800 } else { 0x400 });
        }
        if x & 0x100 != 0 && bg.tilemap_wider {
            sc_offs = sc_offs.wrapping_add(0x400);
        }
        let index = sc_offs.wrapping_add((x >> 3) & 0x1f) & 0x7fff;
        self.vram[index as usize]
    }

    fn pixel_from_4bpp(bits: u32, col: u16, hflip: bool) -> u16 {
        let i = if hflip { col } else { 7 - col };
        ((bits >> i) & 1 | (bits >> (7 + i)) & 2 | (bits >> (14 + i)) & 4 | (bits >> (21 + i)) & 8)
            as u16
    }

    fn pixel_from_2bpp(bits: u16, col: u16, hflip: bool) -> u16 {
        let i = if hflip { col } else { 7 - col };
        ((bits >> i) & 1 | (bits >> (7 + i)) & 2) as u16
    }

    fn mosaic_modulo_at(&self, x: i32) -> u8 {
        self.mosaic_modulo.get(x as usize).copied().unwrap_or(0)
    }

    fn draw_background_4bpp(&mut self, y: u16, sub: bool, layer: usize, zhi: u16, zlo: u16) {
        if !self.screen_enabled_for(sub, layer) {
            return;
        }

        let bg = self.bg_layer[layer];
        let sy = y.wrapping_add(bg.v_scroll);
        let win = self.windows_for_layer(sub, layer);

        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            for screen_x in win.edges[windex] as i32..win.edges[windex + 1] as i32 {
                let sx = (screen_x as u16).wrapping_add(bg.h_scroll);
                let tile = self.bg_tile_entry(layer, sx, sy);
                let tile_y = if tile & 0x8000 != 0 {
                    7 - (sy & 7)
                } else {
                    sy & 7
                };
                let tile_addr = bg
                    .tile_adr
                    .wrapping_add((tile & 0x03ff).wrapping_mul(16))
                    .wrapping_add(tile_y)
                    & 0x7fff;
                let bg_vram = self.bg_vram_latch.as_deref().unwrap_or(&self.vram);
                let bits = bg_vram[tile_addr as usize] as u32
                    | ((bg_vram[tile_addr.wrapping_add(8) as usize & 0x7fff] as u32) << 16);
                if bits == 0 {
                    continue;
                }
                let pixel = Self::pixel_from_4bpp(bits, sx & 7, tile & 0x4000 != 0);
                if pixel == 0 {
                    continue;
                }
                let z =
                    (if tile & 0x2000 != 0 { zhi } else { zlo }).wrapping_add((tile & 0x1c00) >> 6);
                let value = z.wrapping_add(pixel);
                let dst = (screen_x + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                let buffer = &mut self.bg_buffers[sub as usize].data;
                if dst < buffer.len() && z > buffer[dst] {
                    buffer[dst] = value;
                }
            }
        }
    }

    fn draw_background_2bpp(&mut self, y: u16, sub: bool, layer: usize, zhi: u16, zlo: u16) {
        if !self.screen_enabled_for(sub, layer) {
            return;
        }

        let bg = self.bg_layer[layer];
        let sy = y.wrapping_add(bg.v_scroll);
        let win = self.windows_for_layer(sub, layer);

        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            for screen_x in win.edges[windex] as i32..win.edges[windex + 1] as i32 {
                let sx = (screen_x as u16).wrapping_add(bg.h_scroll);
                let tile = self.bg_tile_entry(layer, sx, sy);
                let tile_y = if tile & 0x8000 != 0 {
                    7 - (sy & 7)
                } else {
                    sy & 7
                };
                let tile_addr = bg
                    .tile_adr
                    .wrapping_add((tile & 0x03ff).wrapping_mul(8))
                    .wrapping_add(tile_y)
                    & 0x7fff;
                let bits = self.bg_vram_latch.as_deref().unwrap_or(&self.vram)[tile_addr as usize];
                if bits == 0 {
                    continue;
                }
                let pixel = Self::pixel_from_2bpp(bits, sx & 7, tile & 0x4000 != 0);
                if pixel == 0 {
                    continue;
                }
                let z =
                    (if tile & 0x2000 != 0 { zhi } else { zlo }).wrapping_add((tile & 0x1c00) >> 8);
                let value = z.wrapping_add(pixel);
                let dst = (screen_x + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                let buffer = &mut self.bg_buffers[sub as usize].data;
                if dst < buffer.len() && z > buffer[dst] {
                    buffer[dst] = value;
                }
            }
        }
    }

    fn draw_background_4bpp_mosaic(&mut self, y: u16, sub: bool, layer: usize, zhi: u16, zlo: u16) {
        if !self.screen_enabled_for(sub, layer) {
            return;
        }

        let bg = self.bg_layer[layer];
        let sy = (self.mosaic_modulo[y as usize] as u16).wrapping_add(bg.v_scroll);
        let win = self.windows_for_layer(sub, layer);

        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            let mut screen_x = win.edges[windex] as i32;
            let end_x = win.edges[windex + 1] as i32;
            let mut sx = (screen_x as u16).wrapping_add(bg.h_scroll);
            let mut w = self
                .mosaic_size
                .wrapping_sub(screen_x.wrapping_sub(self.mosaic_modulo_at(screen_x) as i32) as u8)
                as i32;
            while screen_x < end_x {
                w = w.min(end_x - screen_x);
                let tile = self.bg_tile_entry(layer, sx, sy);
                let tile_y = if tile & 0x8000 != 0 {
                    7 - (sy & 7)
                } else {
                    sy & 7
                };
                let tile_addr = bg
                    .tile_adr
                    .wrapping_add((tile & 0x03ff).wrapping_mul(16))
                    .wrapping_add(tile_y)
                    & 0x7fff;
                let bg_vram = self.bg_vram_latch.as_deref().unwrap_or(&self.vram);
                let mut bits = bg_vram[tile_addr as usize] as u32
                    | ((bg_vram[tile_addr.wrapping_add(8) as usize & 0x7fff] as u32) << 16);
                let x_in_tile = sx & 7;
                let pixel = if tile & 0x4000 != 0 {
                    bits >>= x_in_tile;
                    ((bits & 1) | ((bits >> 7) & 2) | ((bits >> 14) & 4) | ((bits >> 21) & 8))
                        as u16
                } else {
                    bits <<= x_in_tile;
                    (((bits >> 7) & 1)
                        | ((bits >> 14) & 2)
                        | ((bits >> 21) & 4)
                        | ((bits >> 28) & 8)) as u16
                };
                if pixel != 0 {
                    let z = if tile & 0x2000 != 0 { zhi } else { zlo };
                    let value = z.wrapping_add((tile & 0x1c00) >> 6).wrapping_add(pixel);
                    for i in 0..w {
                        let dst = (screen_x + i + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                        let buffer = &mut self.bg_buffers[sub as usize].data;
                        if dst < buffer.len() && z > buffer[dst] {
                            buffer[dst] = value;
                        }
                    }
                }
                screen_x += w;
                sx = sx.wrapping_add(w as u16);
                w = self.mosaic_size as i32;
            }
        }
    }

    fn draw_background_2bpp_mosaic(&mut self, y: u16, sub: bool, layer: usize, zhi: u16, zlo: u16) {
        if !self.screen_enabled_for(sub, layer) {
            return;
        }

        let bg = self.bg_layer[layer];
        let sy = (self.mosaic_modulo[y as usize] as u16).wrapping_add(bg.v_scroll);
        let win = self.windows_for_layer(sub, layer);

        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            let mut screen_x = win.edges[windex] as i32;
            let end_x = win.edges[windex + 1] as i32;
            let mut sx = (screen_x as u16).wrapping_add(bg.h_scroll);
            let mut w = self
                .mosaic_size
                .wrapping_sub(screen_x.wrapping_sub(self.mosaic_modulo_at(screen_x) as i32) as u8)
                as i32;
            while screen_x < end_x {
                w = w.min(end_x - screen_x);
                let tile = self.bg_tile_entry(layer, sx, sy);
                let tile_y = if tile & 0x8000 != 0 {
                    7 - (sy & 7)
                } else {
                    sy & 7
                };
                let tile_addr = bg
                    .tile_adr
                    .wrapping_add((tile & 0x03ff).wrapping_mul(8))
                    .wrapping_add(tile_y)
                    & 0x7fff;
                let mut bits =
                    self.bg_vram_latch.as_deref().unwrap_or(&self.vram)[tile_addr as usize] as u32;
                let x_in_tile = sx & 7;
                let pixel = if tile & 0x4000 != 0 {
                    bits >>= x_in_tile;
                    ((bits & 1) | ((bits >> 7) & 2)) as u16
                } else {
                    bits <<= x_in_tile;
                    (((bits >> 7) & 1) | ((bits >> 14) & 2)) as u16
                };
                if pixel != 0 {
                    let z = if tile & 0x2000 != 0 { zhi } else { zlo };
                    let value = z.wrapping_add((tile & 0x1c00) >> 8).wrapping_add(pixel);
                    for i in 0..w {
                        let dst = (screen_x + i + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                        let buffer = &mut self.bg_buffers[sub as usize].data;
                        if dst < buffer.len() && z > buffer[dst] {
                            buffer[dst] = value;
                        }
                    }
                }
                screen_x += w;
                sx = sx.wrapping_add(w as u16);
                w = self.mosaic_size as i32;
            }
        }
    }

    fn expand_m7_13(value: i16) -> i32 {
        ((value << 3) as i16 >> 3) as i32
    }

    fn clip_m7_offset(value: i32) -> i32 {
        if value & 0x2000 != 0 {
            value | !1023
        } else {
            value & 1023
        }
    }

    fn draw_background_mode7(&mut self, mut y: u16, sub: bool, z: u16) {
        let layer = 0usize;
        if !self.screen_enabled_for(sub, layer) {
            return;
        }
        let win = self.windows_for_layer(sub, layer);

        let h_scroll = Self::expand_m7_13(self.m7_matrix[6]);
        let v_scroll = Self::expand_m7_13(self.m7_matrix[7]);
        let x_center = Self::expand_m7_13(self.m7_matrix[4]);
        let y_center = Self::expand_m7_13(self.m7_matrix[5]);
        let clipped_h = Self::clip_m7_offset(h_scroll - x_center);
        let clipped_v = Self::clip_m7_offset(v_scroll - y_center);

        let mosaic_enabled = self.mosaic_enabled & 1 != 0;
        if mosaic_enabled {
            y = self.mosaic_modulo[y as usize] as u16;
        }
        let ry = if self.m7_y_flip {
            255u32.wrapping_sub(y as u32)
        } else {
            y as u32
        };
        let m7start_x = (((self.m7_matrix[0] as i32 * clipped_h) & !63)
            .wrapping_add((self.m7_matrix[1] as i32 * ry as i32) & !63)
            .wrapping_add((self.m7_matrix[1] as i32 * clipped_v) & !63)
            .wrapping_add(x_center << 8)) as u32;
        let m7start_y = (((self.m7_matrix[2] as i32 * clipped_h) & !63)
            .wrapping_add((self.m7_matrix[3] as i32 * ry as i32) & !63)
            .wrapping_add((self.m7_matrix[3] as i32 * clipped_v) & !63)
            .wrapping_add(y_center << 8)) as u32;
        let dx = if self.m7_x_flip {
            (-(self.m7_matrix[0] as i32)) as u32
        } else {
            self.m7_matrix[0] as i32 as u32
        };
        let dy = if self.m7_x_flip {
            (-(self.m7_matrix[2] as i32)) as u32
        } else {
            self.m7_matrix[2] as i32 as u32
        };
        let outside_value = if self.m7_large_field {
            0x3ffff
        } else {
            0xffffffff
        };

        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            let mut x = win.edges[windex] as i32;
            let x2 = win.edges[windex + 1] as i32;
            let rx = if self.m7_x_flip {
                255u32.wrapping_sub(x as u32)
            } else {
                x as u32
            };
            let mut xpos =
                m7start_x.wrapping_add((self.m7_matrix[0] as i32 as u32).wrapping_mul(rx));
            let mut ypos =
                m7start_y.wrapping_add((self.m7_matrix[2] as i32 as u32).wrapping_mul(rx));

            if mosaic_enabled {
                let mut w = self
                    .mosaic_size
                    .wrapping_sub(x.wrapping_sub(self.mosaic_modulo_at(x) as i32) as u8)
                    as i32;
                while x < x2 {
                    w = w.min(x2 - x);
                    let tile = if (xpos | ypos) > outside_value {
                        if !self.m7_char_fill {
                            x += w;
                            xpos = xpos.wrapping_add(dx.wrapping_mul(w as u32));
                            ypos = ypos.wrapping_add(dy.wrapping_mul(w as u32));
                            w = self.mosaic_size as i32;
                            continue;
                        }
                        0usize
                    } else {
                        (self.vram[((ypos >> 11 & 0x7f) * 128 + (xpos >> 11 & 0x7f)) as usize]
                            & 0xff) as usize
                    };
                    let pixel = (self.vram
                        [tile * 64 + ((ypos >> 8 & 7) * 8 + (xpos >> 8 & 7)) as usize]
                        >> 8) as u16;
                    if pixel != 0 {
                        for i in 0..w {
                            let dst = (x + i + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                            if dst < self.bg_buffers[sub as usize].data.len() {
                                self.bg_buffers[sub as usize].data[dst] = pixel + z;
                            }
                        }
                    }
                    x += w;
                    xpos = xpos.wrapping_add(dx.wrapping_mul(w as u32));
                    ypos = ypos.wrapping_add(dy.wrapping_mul(w as u32));
                    w = self.mosaic_size as i32;
                }
            } else {
                while x < x2 {
                    let tile = if (xpos | ypos) > outside_value {
                        if !self.m7_char_fill {
                            x += 1;
                            xpos = xpos.wrapping_add(dx);
                            ypos = ypos.wrapping_add(dy);
                            continue;
                        }
                        0usize
                    } else {
                        (self.vram[((ypos >> 11 & 0x7f) * 128 + (xpos >> 11 & 0x7f)) as usize]
                            & 0xff) as usize
                    };
                    let pixel = (self.vram
                        [tile * 64 + ((ypos >> 8 & 7) * 8 + (xpos >> 8 & 7)) as usize]
                        >> 8) as u16;
                    if pixel != 0 {
                        let dst = (x + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                        if dst < self.bg_buffers[sub as usize].data.len() {
                            self.bg_buffers[sub as usize].data[dst] = pixel + z;
                        }
                    }
                    x += 1;
                    xpos = xpos.wrapping_add(dx);
                    ypos = ypos.wrapping_add(dy);
                }
            }
        }
    }

    fn draw_backgrounds(&mut self, y: u16, sub: bool) {
        if self.bg_mode() == 1 {
            if self.line_has_sprites {
                self.draw_sprites(sub, true);
            }
            if self.mosaic_enabled & 1 == 0 {
                self.draw_background_4bpp(y, sub, 0, 0xc000, 0x8000);
            } else {
                self.draw_background_4bpp_mosaic(y, sub, 0, 0xc000, 0x8000);
            }
            if self.mosaic_enabled & 2 == 0 {
                self.draw_background_4bpp(y, sub, 1, 0xb100, 0x7100);
            } else {
                self.draw_background_4bpp_mosaic(y, sub, 1, 0xb100, 0x7100);
            }
            let bg3_high = if self.mode1_bg3_priority() {
                0xf200
            } else {
                0x7200
            };
            if self.mosaic_enabled & 4 == 0 {
                self.draw_background_2bpp(y, sub, 2, bg3_high, 0x1200);
            } else {
                self.draw_background_2bpp_mosaic(y, sub, 2, bg3_high, 0x1200);
            }
        } else {
            self.draw_background_mode7(y, sub, 0xc000);
            if self.line_has_sprites {
                self.draw_sprites(sub, false);
            }
        }
    }

    fn float_interpolate(x: f32, xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> f32 {
        ymin + (ymax - ymin) * (x - xmin) * (1.0 / (xmax - xmin))
    }

    fn draw_mode7_upsampled(&mut self, y: usize) {
        let x_center = Self::expand_m7_13(self.m7_matrix[4]) as u32;
        let y_center = Self::expand_m7_13(self.m7_matrix[5]) as u32;
        let clipped_h = (Self::expand_m7_13(self.m7_matrix[6]) as u32).wrapping_sub(x_center);
        let clipped_v = (Self::expand_m7_13(self.m7_matrix[7]) as u32).wrapping_sub(y_center);

        let mut m0v = [0i32; 4];
        if self.mode7_perspective_low.to_bits() == 0 {
            let value = (self.m7_matrix[0] as i32) << 12;
            m0v = [value; 4];
        } else {
            const OFFSETS: [f32; 4] = [-1.0, -0.75, -0.5, -0.25];
            for i in 0..4 {
                m0v[i] = (4096.0
                    / Self::float_interpolate(
                        y as f32 + OFFSETS[i],
                        0.0,
                        223.0,
                        self.mode7_perspective_low,
                        self.mode7_perspective_high,
                    )) as i32;
            }
        }

        let pitch = self.render_pitch as usize;
        if pitch == 0 {
            return;
        }
        let Some(mut buffer) = self.render_buffer.take() else {
            return;
        };
        let render_start = (y - 1).saturating_mul(4).saturating_mul(pitch);
        let dst_start = render_start + (self.extra_left_right - self.extra_left_cur) as usize * 16;
        let draw_width = 256usize + self.extra_left_cur as usize + self.extra_right_cur as usize;
        let m1 = (self.m7_matrix[1] as i32 as u32) << 12;
        let m2 = (self.m7_matrix[2] as i32 as u32) << 12;

        for j in 0..4usize {
            let m0 = m0v[j] as u32;
            let m3 = m0;
            let mut xpos = m0
                .wrapping_mul(clipped_h)
                .wrapping_add(m1.wrapping_mul(clipped_v.wrapping_add(y as u32)))
                .wrapping_add(x_center << 20);
            let mut ypos = m2
                .wrapping_mul(clipped_h)
                .wrapping_add(m3.wrapping_mul(clipped_v.wrapping_add(y as u32)))
                .wrapping_add(y_center << 20);
            xpos = xpos.wrapping_sub((m0.wrapping_add(m1)) >> 1);
            ypos = ypos.wrapping_sub((m2.wrapping_add(m3)) >> 1);
            let mut xcur = (xpos << 2).wrapping_add((j as u32).wrapping_mul(m1));
            let mut ycur = (ypos << 2).wrapping_add((j as u32).wrapping_mul(m3));
            xcur = xcur.wrapping_sub(
                (self.extra_left_cur as u32)
                    .wrapping_mul(4)
                    .wrapping_mul(m0),
            );
            ycur = ycur.wrapping_sub(
                (self.extra_left_cur as u32)
                    .wrapping_mul(4)
                    .wrapping_mul(m2),
            );

            let mut dst = dst_start + j * pitch;
            for _ in 0..draw_width * 4 {
                if dst + 4 > buffer.len() {
                    break;
                }
                let tile =
                    self.vram[((ycur >> 25 & 0x7f) * 128 + (xcur >> 25 & 0x7f)) as usize] & 0xff;
                let mut pixel = (self.vram
                    [(tile as usize) * 64 + ((ycur >> 22 & 7) * 8 + (xcur >> 22 & 7)) as usize]
                    >> 8) as usize;
                if xcur & 0x80000000 != 0 {
                    pixel = 0;
                }
                let mut color = self.color_map_rgb[pixel];
                if self.half_color {
                    color = (color & 0xfefefe) >> 1;
                }
                buffer[dst..dst + 4].copy_from_slice(&color.to_le_bytes());
                xcur = xcur.wrapping_add(m0);
                ycur = ycur.wrapping_add(m2);
                dst += 4;
            }
        }

        if self.line_has_sprites {
            let mut dst = dst_start;
            let pixel_start = PPU_EXTRA_LEFT_RIGHT.saturating_sub(self.extra_left_cur as usize);
            for i in 0..draw_width {
                let pixel = (self.obj_buffer.data[pixel_start + i] & 0xff) as usize;
                if pixel != 0 {
                    let color = self.color_map_rgb[pixel].to_le_bytes();
                    for row in 0..4 {
                        for col in 0..4 {
                            let off = dst + row * pitch + col * 4;
                            if off + 4 <= buffer.len() {
                                buffer[off..off + 4].copy_from_slice(&color);
                            }
                        }
                    }
                }
                dst += 16;
            }
        }

        let clear_left = self.extra_left_right.saturating_sub(self.extra_left_cur) as usize;
        if clear_left != 0 {
            let n = 16 * clear_left;
            for row in 0..4 {
                let off = render_start + row * pitch;
                if off + n <= buffer.len() {
                    buffer[off..off + n].fill(0);
                }
            }
        }
        let clear_right = self.extra_left_right.saturating_sub(self.extra_right_cur) as usize;
        if clear_right != 0 {
            let n = 16 * clear_right;
            let x = 256 + self.extra_left_right as usize * 2 - clear_right;
            for row in 0..4 {
                let off = render_start + row * pitch + x * 16;
                if off + n <= buffer.len() {
                    buffer[off..off + n].fill(0);
                }
            }
        }
        self.render_buffer = Some(buffer);
    }

    fn sprite_buffer_for_line(&self, line: i32) -> (bool, PpuPixelPrioBufs) {
        #[derive(Clone, Copy)]
        struct SpriteLine {
            index: usize,
            row: i32,
            x: i32,
            sprite_size: i32,
        }

        let mut index = 0usize;
        let index_end = index;
        let mut buffer = PpuPixelPrioBufs::default();
        buffer.data.fill(0x0500);
        let mut sprites_left = 33i32;
        let mut tiles_left = 35i32;
        let sprite_sizes = SPRITE_SIZES[self.obj_size as usize & 7];
        let extra_left_right = self.extra_left_right as i32;
        if self.render_flags.contains(PpuRenderFlags::NO_SPRITE_LIMITS) {
            sprites_left = 1024;
            tiles_left = 1024;
        }
        let tiles_left_org = tiles_left;
        let mut sprites = Vec::with_capacity(32);

        loop {
            let yy = ((self.oam[index] >> 8) as i32 + 1) & 0xff;
            if yy != 0xf0 {
                let row = (line - yy) & 0xff;
                let high_oam = (self.oam[0x100 + (index >> 4)] >> (index & 15)) as i32;
                let sprite_size = sprite_sizes[((high_oam >> 1) & 1) as usize] as i32;
                if row < sprite_size {
                    let object_x = (self.oam[index] & 0xff) as i32 + (high_oam & 1) * 256;
                    if object_x > 256 && object_x + sprite_size - 1 < 512 {
                        index = (index + 2) & 0xff;
                        if index == index_end {
                            break;
                        }
                        continue;
                    }
                    let mut x = object_x;
                    if x >= 256 + extra_left_right {
                        x -= 512;
                    }
                    if x > -(sprite_size + extra_left_right) {
                        sprites_left -= 1;
                        if sprites_left == 0 {
                            break;
                        }

                        sprites.push(SpriteLine {
                            index,
                            row,
                            x,
                            sprite_size,
                        });
                    }
                }
            }

            index = (index + 2) & 0xff;
            if index == index_end {
                break;
            }
        }

        if std::env::var_os("TRACE_OBJ_EDGE").is_some()
            && (line == 0 || line == 1 || line == 185 || line == 211 || (216..=223).contains(&line))
        {
            for sprite in &sprites {
                let high_oam =
                    (self.oam[0x100 + (sprite.index >> 4)] >> (sprite.index & 15)) as i32;
                let oam1 = self.oam[sprite.index + 1];
                eprintln!(
                    "obj-edge line={} idx={} x={} rawx={:03x} y={} row={} size={} high={:02x} oam0={:04x} oam1={:04x}",
                    line,
                    sprite.index / 2,
                    sprite.x,
                    (self.oam[sprite.index] & 0xff) as i32 + (high_oam & 1) * 256,
                    (self.oam[sprite.index] >> 8) as i32,
                    sprite.row,
                    sprite.sprite_size,
                    high_oam,
                    self.oam[sprite.index],
                    oam1,
                );
            }
        }

        for sprite in &sprites {
            let mut row = sprite.row;
            let oam1 = self.oam[sprite.index + 1];
            let obj_addr = if oam1 & 0x0100 != 0 {
                self.obj_tile_adr2
            } else {
                self.obj_tile_adr1
            };
            if oam1 & 0x8000 != 0 {
                row = sprite.sprite_size - 1 - row;
            }
            let palette_base = 0x80 + 16 * ((oam1 & 0x0e00) >> 9);
            let prio = Self::sprite_prio_to_prio((oam1 & 0x3000) >> 12, oam1 & 0x0800 == 0);
            let z = palette_base + (prio << 8);

            let mut col = 0i32;
            while col < sprite.sprite_size {
                if col + sprite.x > -8 - extra_left_right && col + sprite.x < 256 + extra_left_right
                {
                    tiles_left -= 1;
                    if tiles_left == 0 {
                        return (true, buffer);
                    }

                    let used_col = if oam1 & 0x4000 != 0 {
                        sprite.sprite_size - 1 - col
                    } else {
                        col
                    };
                    let used_tile = ((((oam1 & 0xff) >> 4) as i32 + (row >> 3)) << 4)
                        | ((((oam1 & 0x0f) as i32) + (used_col >> 3)) & 0x0f);
                    let addr = obj_addr
                        .wrapping_add((used_tile as u16).wrapping_mul(16))
                        .wrapping_add((row & 7) as u16)
                        & 0x7fff;
                    let obj_vram = self.obj_vram_latch.as_deref().unwrap_or(&self.vram);
                    let plane = obj_vram[addr as usize] as u32
                        | ((obj_vram[addr.wrapping_add(8) as usize & 0x7fff] as u32) << 16);
                    let px_left = (-(col + sprite.x + PPU_EXTRA_LEFT_RIGHT as i32)).max(0);
                    let px_right = (256 + PPU_EXTRA_LEFT_RIGHT as i32 - (col + sprite.x)).min(8);
                    for px in px_left..px_right {
                        let shift = if oam1 & 0x4000 != 0 { px } else { 7 - px };
                        let bits = plane >> shift;
                        let pixel = (bits & 1)
                            | ((bits >> 7) & 2)
                            | ((bits >> 14) & 4)
                            | ((bits >> 21) & 8);
                        let dst = (col + sprite.x + px + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                        if std::env::var_os("TRACE_OBJ_EDGE").is_some()
                            && (((line == 0 && dst == 124 + PPU_EXTRA_LEFT_RIGHT)
                                || (line == 1 && dst == 127 + PPU_EXTRA_LEFT_RIGHT)
                                || (line == 185 && dst == 78 + PPU_EXTRA_LEFT_RIGHT))
                                || (line == 210 && dst == PPU_EXTRA_LEFT_RIGHT + 11)
                                || (line == 211 && dst == 11 + PPU_EXTRA_LEFT_RIGHT)
                                || (line == 217 && dst == PPU_EXTRA_LEFT_RIGHT + 1)
                                || (line == 218 && dst == PPU_EXTRA_LEFT_RIGHT + 11)
                                || (line == 220 && dst == PPU_EXTRA_LEFT_RIGHT + 10))
                        {
                            eprintln!(
                                "obj-pixel line={} dst={} idx={} addr={:04x}/{:04x} plane={:08x} px={} pixel={} oam0={:04x} oam1={:04x}",
                                line,
                                dst,
                                sprite.index / 2,
                                addr,
                                addr.wrapping_add(8) & 0x7fff,
                                plane,
                                px,
                                pixel,
                                self.oam[sprite.index],
                                oam1,
                            );
                        }
                        if pixel != 0 && dst < buffer.data.len() && buffer.data[dst] & 0xff == 0 {
                            buffer.data[dst] = z + pixel as u16;
                        }
                    }
                }
                col += 8;
            }
        }

        (tiles_left != tiles_left_org, buffer)
    }

    fn fetch_sprites_for_line(&mut self, line: i32) {
        let (has_sprites, buffer) = self.sprite_buffer_for_line(line);
        self.obj_fetch_has_sprites = has_sprites;
        self.obj_fetch_buffer = buffer;
    }

    fn draw_sprites(&mut self, sub: bool, clear_backdrop: bool) {
        let layer = 4usize;
        if !self.screen_enabled_for(sub, layer) {
            return;
        }
        let win = self.windows_for_layer(sub, layer);
        for windex in 0..win.nr as usize {
            if win.bits & (1 << windex) != 0 {
                continue;
            }
            for screen_x in win.edges[windex] as i32..win.edges[windex + 1] as i32 {
                let dst = (screen_x + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                if dst >= self.obj_buffer.data.len()
                    || dst >= self.bg_buffers[sub as usize].data.len()
                {
                    continue;
                }
                let src = self.obj_buffer.data[dst];
                let bg = &mut self.bg_buffers[sub as usize].data[dst];
                if clear_backdrop || src > *bg {
                    *bg = src;
                }
                if std::env::var_os("TRACE_OBJ_EDGE").is_some()
                    && !sub
                    && screen_x == 78
                    && src != 0x0500
                {
                    eprintln!(
                        "obj-compose sub={} clear={} screen_x={} dst={} src={:04x} bg={:04x} enabled={:02x}/{:02x}",
                        sub,
                        clear_backdrop,
                        screen_x,
                        dst,
                        src,
                        *bg,
                        self.screen_enabled[0],
                        self.screen_enabled[1],
                    );
                }
            }
        }
    }

    fn get_window_state(&self, layer: usize, x: i32) -> bool {
        let winflags = self.windowsel >> (layer * 4);
        let w1_enabled = winflags & WINDOW_1_ENABLED != 0;
        let w2_enabled = winflags & WINDOW_2_ENABLED != 0;
        if !w1_enabled && !w2_enabled {
            return false;
        }
        if w1_enabled && !w2_enabled {
            let test = x >= self.window1_left as i32 && x <= self.window1_right as i32;
            return if winflags & WINDOW_1_INVERSED != 0 {
                !test
            } else {
                test
            };
        }
        if !w1_enabled && w2_enabled {
            let test = x >= self.window2_left as i32 && x <= self.window2_right as i32;
            return if winflags & WINDOW_2_INVERSED != 0 {
                !test
            } else {
                test
            };
        }
        let mut test1 = x >= self.window1_left as i32 && x <= self.window1_right as i32;
        let mut test2 = x >= self.window2_left as i32 && x <= self.window2_right as i32;
        if winflags & WINDOW_1_INVERSED != 0 {
            test1 = !test1;
        }
        if winflags & WINDOW_2_INVERSED != 0 {
            test2 = !test2;
        }
        test1 || test2
    }

    fn calculate_mode7_starts(&mut self, mut y: i32) {
        let h_scroll = Self::expand_m7_13(self.m7_matrix[6]);
        let v_scroll = Self::expand_m7_13(self.m7_matrix[7]);
        let x_center = Self::expand_m7_13(self.m7_matrix[4]);
        let y_center = Self::expand_m7_13(self.m7_matrix[5]);
        let clipped_h = Self::clip_m7_offset(h_scroll - x_center);
        let clipped_v = Self::clip_m7_offset(v_scroll - y_center);
        if self.mosaic_enabled & 1 != 0 {
            y -= (y - 1) % self.mosaic_size as i32;
        }
        let ry = if self.m7_y_flip { 255 - y } else { y };
        self.m7_start_x = ((self.m7_matrix[0] as i32 * clipped_h) & !63)
            + ((self.m7_matrix[1] as i32 * ry) & !63)
            + ((self.m7_matrix[1] as i32 * clipped_v) & !63)
            + (x_center << 8);
        self.m7_start_y = ((self.m7_matrix[2] as i32 * clipped_h) & !63)
            + ((self.m7_matrix[3] as i32 * ry) & !63)
            + ((self.m7_matrix[3] as i32 * clipped_v) & !63)
            + (y_center << 8);
    }

    fn pixel_for_mode7_old(&self, mut x: i32, layer: usize, priority: usize) -> u16 {
        if self.mosaic_enabled & (1 << layer) != 0 {
            x -= x % self.mosaic_size as i32;
        }
        let rx = if self.m7_x_flip { 255 - x } else { x };
        let mut x_pos = (self.m7_start_x + self.m7_matrix[0] as i32 * rx) >> 8;
        let mut y_pos = (self.m7_start_y + self.m7_matrix[2] as i32 * rx) >> 8;
        let mut outside_map = x_pos < 0 || x_pos >= 1024 || y_pos < 0 || y_pos >= 1024;
        x_pos &= 0x3ff;
        y_pos &= 0x3ff;
        if !self.m7_large_field {
            outside_map = false;
        }
        let tile = if outside_map {
            0
        } else {
            self.vram[((y_pos >> 3) * 128 + (x_pos >> 3)) as usize] & 0xff
        } as usize;
        let pixel = if outside_map && !self.m7_char_fill {
            0
        } else {
            self.vram[tile * 64 + ((y_pos & 7) * 8 + (x_pos & 7)) as usize] >> 8
        } as u16;
        if layer == 1 {
            if ((pixel & 0x80) != 0) != (priority != 0) {
                return 0;
            }
            return pixel & 0x7f;
        }
        pixel
    }

    fn pixel_for_bg_layer_old(&self, x: i32, y: i32, layer: usize, priority: usize) -> u16 {
        let bg = self.bg_layer[layer];
        let mode = self.bg_mode();
        let wide_tiles = mode == 5 || mode == 6;
        let tile_bits_x = if wide_tiles { 4 } else { 3 };
        let tile_high_bit_x = if wide_tiles { 0x200 } else { 0x100 };
        let tile_bits_y = 3;
        let tile_high_bit_y = 0x100;
        let mut tilemap_adr = bg.tilemap_adr as i32
            + ((((y >> tile_bits_y) & 0x1f) << 5) | ((x >> tile_bits_x) & 0x1f));
        if x & tile_high_bit_x != 0 && bg.tilemap_wider {
            tilemap_adr += 0x400;
        }
        if y & tile_high_bit_y != 0 && bg.tilemap_higher {
            tilemap_adr += if bg.tilemap_wider { 0x800 } else { 0x400 };
        }
        let tile = self.vram[(tilemap_adr & 0x7fff) as usize];
        if ((tile & 0x2000) != 0) != (priority != 0) {
            return 0;
        }
        let mut palette_num = (tile & 0x1c00) >> 10;
        let row = if tile & 0x8000 != 0 {
            7 - (y & 7)
        } else {
            y & 7
        } as u16;
        let col = if tile & 0x4000 != 0 {
            x & 7
        } else {
            7 - (x & 7)
        } as u16;
        let mut tile_num = tile & 0x03ff;
        if wide_tiles && ((x & 8 != 0) != (tile & 0x4000 != 0)) {
            tile_num = tile_num.wrapping_add(1);
        }
        let bit_depth = BIT_DEPTHS_PER_MODE[mode as usize][layer];
        if mode == 0 {
            palette_num += 8 * layer as u16;
        }
        let tile_base = bg
            .tile_adr
            .wrapping_add((tile_num & 0x03ff).wrapping_mul(4 * bit_depth as u16));
        let plane1 = self.vram[tile_base.wrapping_add(row) as usize & 0x7fff];
        let mut pixel = ((plane1 >> col) & 1) | (((plane1 >> (8 + col)) & 1) << 1);
        let mut palette_size = 4u16;
        if bit_depth > 2 {
            palette_size = 16;
            let plane2 = self.vram[tile_base.wrapping_add(8).wrapping_add(row) as usize & 0x7fff];
            pixel |= ((plane2 >> col) & 1) << 2;
            pixel |= ((plane2 >> (8 + col)) & 1) << 3;
        }
        if bit_depth > 4 {
            palette_size = 256;
            let plane3 = self.vram[tile_base.wrapping_add(16).wrapping_add(row) as usize & 0x7fff];
            pixel |= ((plane3 >> col) & 1) << 4;
            pixel |= ((plane3 >> (8 + col)) & 1) << 5;
            let plane4 = self.vram[tile_base.wrapping_add(24).wrapping_add(row) as usize & 0x7fff];
            pixel |= ((plane4 >> col) & 1) << 6;
            pixel |= ((plane4 >> (8 + col)) & 1) << 7;
        }
        if pixel == 0 {
            0
        } else {
            palette_size * palette_num + pixel
        }
    }

    fn get_pixel_old(&self, x: i32, y: i32, sub: bool) -> (usize, u16) {
        let mode = self.bg_mode();
        let mut act_mode = if mode == 1 { 8 } else { mode as usize };
        if mode == 7 && self.m7_ext_bg_always_zero {
            act_mode = 9;
        }
        let mut layer = 5usize;
        let mut pixel = 0u16;
        for i in 0..LAYER_COUNT_PER_MODE[act_mode] {
            let cur_layer = LAYERS_PER_MODE[act_mode][i];
            let cur_priority = PRIORITIES_PER_MODE[act_mode][i];
            let layer_active = if !sub {
                self.screen_enabled_for(false, cur_layer)
                    && (self.screen_windowed[0] & (1 << cur_layer) == 0
                        || !self.get_window_state(cur_layer, x))
            } else {
                self.screen_enabled_for(true, cur_layer)
                    && (self.screen_windowed[1] & (1 << cur_layer) == 0
                        || !self.get_window_state(cur_layer, x))
            };
            if layer_active {
                if cur_layer < 4 {
                    let mut lx = x;
                    let mut ly = y;
                    if self.mosaic_enabled & (1 << cur_layer) != 0 {
                        lx -= lx % self.mosaic_size as i32;
                        ly -= (ly - 1) % self.mosaic_size as i32;
                    }
                    if mode == 7 {
                        pixel = self.pixel_for_mode7_old(lx, cur_layer, cur_priority);
                    } else {
                        lx += self.bg_layer[cur_layer].h_scroll as i32;
                        ly += self.bg_layer[cur_layer].v_scroll as i32;
                        pixel = self.pixel_for_bg_layer_old(
                            lx & 0x3ff,
                            ly & 0x3ff,
                            cur_layer,
                            cur_priority,
                        );
                    }
                } else {
                    pixel = 0;
                    let src = self
                        .obj_buffer
                        .data
                        .get(x as usize + PPU_EXTRA_LEFT_RIGHT)
                        .copied()
                        .unwrap_or(0);
                    if (src >> 12) as usize == cur_priority * 4 + 2 {
                        pixel = src & 0xff;
                    }
                }
            }
            if pixel > 0 {
                layer = cur_layer;
                break;
            }
        }
        if layer == 4 && pixel < 0xc0 {
            layer = 6;
        }
        (layer, self.cgram[(pixel & 0xff) as usize])
    }

    pub fn debug_pixel_old_summary(&mut self, x: i32, y: i32, sub: bool) -> String {
        let (has_sprites, buffer) = self.sprite_buffer_for_line(y);
        self.line_has_sprites = has_sprites;
        self.obj_buffer = buffer;
        let (layer, color) = self.get_pixel_old(x, y, sub);
        format!(
            "old layer={} color={:04x} screen={:02x}/{:02x} scroll=bg1({:04x},{:04x}) bg2({:04x},{:04x}) bg3({:04x},{:04x}) obj={}",
            layer,
            color,
            self.screen_enabled[0],
            self.screen_enabled[1],
            self.bg_layer[0].h_scroll,
            self.bg_layer[0].v_scroll,
            self.bg_layer[1].h_scroll,
            self.bg_layer[1].v_scroll,
            self.bg_layer[2].h_scroll,
            self.bg_layer[2].v_scroll,
            has_sprites,
        )
    }

    fn handle_pixel_old(&mut self, x: i32, y: i32) {
        let mut color = 0u16;
        let trace_pixel = std::env::var_os("TRACE_OBJ_EDGE").is_some()
            && ((x == 127 && (y == 56 || y == 57)) || (x == 40 && (y == 40 || y == 41)));
        // `run_line` and the legacy renderer number visible lines 1..=224,
        // while presentation receipts number their output rows 0..223.
        // Keep the conversion here, at the only legacy pixel entry point, so
        // a force-blank edge beginning at receipt row 1 does not erase row 0.
        if !self.forced_blank_on_line((y - 1) as usize) {
            let (found_main_layer, main_color) = self.get_pixel_old(x, y, false);
            let main_layer = found_main_layer;
            color = main_color;
            let color_window_state = self.get_window_state(5, x);
            if self.clip_mode == 3
                || (self.clip_mode == 2 && color_window_state)
                || (self.clip_mode == 1 && !color_window_state)
            {
                color = 0;
            }
            let math_enabled = main_layer < 6
                && (self.math_enabled & (1 << main_layer)) != 0
                && !(self.prevent_math_mode == 3
                    || (self.prevent_math_mode == 2 && color_window_state)
                    || (self.prevent_math_mode == 1 && !color_window_state));
            let mut second_layer = 5usize;
            let mut color2 = 0u16;
            let mode = self.bg_mode();
            if (math_enabled && self.add_subscreen) || mode == 5 || mode == 6 {
                let (layer2, sub_color) = self.get_pixel_old(x, y, true);
                second_layer = layer2;
                color2 = sub_color;
            }
            if trace_pixel {
                eprintln!(
                    "old-compose-pre x={} y={} main_layer={} main_color={:04x} math={} second_layer={} color2={:04x} screen={:02x}/{:02x} math_enabled={:02x} add_sub={} clip={} prevent={}",
                    x,
                    y,
                    main_layer,
                    color,
                    math_enabled,
                    second_layer,
                    color2,
                    self.screen_enabled[0],
                    self.screen_enabled[1],
                    self.math_enabled,
                    self.add_subscreen,
                    self.clip_mode,
                    self.prevent_math_mode,
                );
            }
            if math_enabled {
                let mut r = (color & 0x1f) as i32;
                let mut g = ((color >> 5) & 0x1f) as i32;
                let mut b = ((color >> 10) & 0x1f) as i32;
                let r2 = if self.add_subscreen && second_layer != 5 {
                    (color2 & 0x1f) as i32
                } else {
                    self.fixed_color_r as i32
                };
                let g2 = if self.add_subscreen && second_layer != 5 {
                    ((color2 >> 5) & 0x1f) as i32
                } else {
                    self.fixed_color_g as i32
                };
                let b2 = if self.add_subscreen && second_layer != 5 {
                    ((color2 >> 10) & 0x1f) as i32
                } else {
                    self.fixed_color_b as i32
                };
                if self.subtract_color {
                    r -= r2;
                    g -= g2;
                    b -= b2;
                } else {
                    r += r2;
                    g += g2;
                    b += b2;
                }
                if self.half_color && (second_layer != 5 || !self.add_subscreen) {
                    r >>= 1;
                    g >>= 1;
                    b >>= 1;
                }
                color = r.clamp(0, 31) as u16
                    | ((g.clamp(0, 31) as u16) << 5)
                    | ((b.clamp(0, 31) as u16) << 10);
            }
        }
        if trace_pixel {
            eprintln!("old-compose-post x={} y={} color={:04x}", x, y, color);
        }

        let row = (y - 1) as usize;
        let dst = row
            .saturating_mul(self.render_pitch as usize)
            .saturating_add((x as usize + self.extra_left_right as usize) * 4);
        let Some(buffer) = self.render_buffer.as_mut() else {
            return;
        };
        if dst + 3 >= buffer.len() {
            return;
        }
        buffer[dst] = self.brightness_mult[((color >> 10) & 0x1f) as usize];
        buffer[dst + 1] = self.brightness_mult[((color >> 5) & 0x1f) as usize];
        buffer[dst + 2] = self.brightness_mult[(color & 0x1f) as usize];
        buffer[dst + 3] = 0;
    }

    fn draw_whole_line(&mut self, line: usize) {
        if self.forced_blank_on_line(line.saturating_sub(1)) {
            self.fill_render_line(line - 1, 0);
            return;
        }

        self.bg_buffers[0].data.fill(0x0500);

        if self.bg_mode() == 7 && self.render_flags.contains(PpuRenderFlags::MODE7_4X4) {
            self.draw_mode7_upsampled(line);
            return;
        }

        self.draw_backgrounds(line as u16, false);

        let math_enabled = self.math_enabled as u32;
        let mut rendered_subscreen = false;
        if self.prevent_math_mode != 3 && self.add_subscreen && math_enabled != 0 {
            self.bg_buffers[1].data.fill(0x0500);
            if self.screen_enabled[1] != 0 {
                self.draw_backgrounds(line as u16, true);
                rendered_subscreen = true;
            }
        }

        let cwin = self.ppu_windows_calc(5);
        const CW_BITS_MOD: [u8; 8] = [0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00];
        let mut cw_clip_math = (((cwin.bits & CW_BITS_MOD[self.clip_mode as usize]) as u32)
            ^ CW_BITS_MOD[self.clip_mode as usize + 4] as u32)
            | ((((cwin.bits & CW_BITS_MOD[self.prevent_math_mode as usize]) as u32)
                ^ CW_BITS_MOD[self.prevent_math_mode as usize + 4] as u32)
                << 8);
        let fixed_color = self.fixed_color_r as u16
            | ((self.fixed_color_g as u16) << 5)
            | ((self.fixed_color_b as u16) << 10);

        let pitch = self.render_pitch as usize;
        if pitch == 0 {
            return;
        }
        let Some(mut buffer) = self.render_buffer.take() else {
            return;
        };
        let start = (line - 1).saturating_mul(pitch);
        let mut output_x = self.extra_left_right.saturating_sub(self.extra_left_cur) as usize;

        for windex in 0..cwin.nr as usize {
            let left = (cwin.edges[windex] as i32 + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
            let right = (cwin.edges[windex + 1] as i32 + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
            let clip_color_mask = if cw_clip_math & 1 != 0 { 0x1f } else { 0 };
            let mut math_enabled_cur = if cw_clip_math & 0x100 != 0 {
                math_enabled
            } else {
                0
            };

            if math_enabled_cur == 0
                || (fixed_color == 0 && !self.half_color && !rendered_subscreen)
            {
                for i in left..right.min(PPU_X_PIXELS) {
                    let dst = start + output_x * 4;
                    if dst + 4 > buffer.len() {
                        break;
                    }
                    let color = self.cgram[(self.bg_buffers[0].data[i] & 0xff) as usize];
                    let r = self.brightness_mult[(color as usize) & clip_color_mask];
                    let g = self.brightness_mult[((color as usize) >> 5) & clip_color_mask];
                    let b = self.brightness_mult[((color as usize) >> 10) & clip_color_mask];
                    let rgb = 0xff000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    if std::env::var_os("TRACE_OBJ_EDGE").is_some()
                        && ((line == 185 && i == 78 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 1 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 56 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 57 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 40 && i == 40 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 41 && i == 40 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 211 && i == 11 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 210 && i == 11 + PPU_EXTRA_LEFT_RIGHT))
                    {
                        eprintln!(
                            "line-compose line={} i={} outx={} dst={} main={:04x} cgram={:04x} rgb={:08x} screen={:02x}/{:02x} extra={}/{}/{} pitch={} start={}",
                            line,
                            i,
                            output_x,
                            dst,
                            self.bg_buffers[0].data[i],
                            color,
                            rgb,
                            self.screen_enabled[0],
                            self.screen_enabled[1],
                            self.extra_left_right,
                            self.extra_left_cur,
                            self.extra_right_cur,
                            pitch,
                            start,
                        );
                    }
                    buffer[dst..dst + 4].copy_from_slice(&rgb.to_le_bytes());
                    output_x += 1;
                }
            } else {
                math_enabled_cur |= (self.add_subscreen as u32) << 8;
                math_enabled_cur |= (self.subtract_color as u32) << 9;
                for i in left..right.min(PPU_X_PIXELS) {
                    let dst = start + output_x * 4;
                    if dst + 4 > buffer.len() {
                        break;
                    }
                    let color = self.cgram[(self.bg_buffers[0].data[i] & 0xff) as usize];
                    let main_layer = ((self.bg_buffers[0].data[i] >> 8) & 0x0f) as u32;
                    let mut r = (color as usize) & clip_color_mask;
                    let mut g = ((color as usize) >> 5) & clip_color_mask;
                    let mut b = ((color as usize) >> 10) & clip_color_mask;
                    let mut use_half_map = false;

                    if math_enabled_cur & (1 << main_layer) != 0 {
                        let color2 = if math_enabled_cur & 0x100 != 0 {
                            let sub_color_index = self.bg_buffers[1].data[i] & 0xff;
                            if sub_color_index != 0 {
                                use_half_map = self.half_color;
                                self.cgram[sub_color_index as usize]
                            } else {
                                fixed_color
                            }
                        } else {
                            use_half_map = self.half_color;
                            fixed_color
                        };
                        let r2 = (color2 & 0x001f) as usize;
                        let g2 = ((color2 >> 5) & 0x001f) as usize;
                        let b2 = ((color2 >> 10) & 0x001f) as usize;
                        if math_enabled_cur & 0x200 != 0 {
                            r = r.saturating_sub(r2);
                            g = g.saturating_sub(g2);
                            b = b.saturating_sub(b2);
                        } else {
                            r += r2;
                            g += g2;
                            b += b2;
                        }
                    }

                    let color_map = if use_half_map {
                        &self.brightness_mult_half
                    } else {
                        &self.brightness_mult
                    };
                    let rgb = color_map[b] as u32
                        | ((color_map[g] as u32) << 8)
                        | ((color_map[r] as u32) << 16)
                        | 0xff000000;
                    if std::env::var_os("TRACE_OBJ_EDGE").is_some()
                        && ((line == 185 && i == 78 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 1 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 56 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 57 && i == 127 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 40 && i == 40 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 41 && i == 40 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 211 && i == 11 + PPU_EXTRA_LEFT_RIGHT)
                            || (line == 210 && i == 11 + PPU_EXTRA_LEFT_RIGHT))
                    {
                        eprintln!(
                            "line-compose line={} i={} outx={} dst={} main={:04x} sub={:04x} cgram={:04x} rgb={:08x} math={:03x} screen={:02x}/{:02x} extra={}/{}/{} pitch={} start={}",
                            line,
                            i,
                            output_x,
                            dst,
                            self.bg_buffers[0].data[i],
                            self.bg_buffers[1].data[i],
                            color,
                            rgb,
                            math_enabled_cur,
                            self.screen_enabled[0],
                            self.screen_enabled[1],
                            self.extra_left_right,
                            self.extra_left_cur,
                            self.extra_right_cur,
                            pitch,
                            start,
                        );
                    }
                    buffer[dst..dst + 4].copy_from_slice(&rgb.to_le_bytes());
                    output_x += 1;
                }
            }

            cw_clip_math >>= 1;
        }

        let clear_left = self.extra_left_right.saturating_sub(self.extra_left_cur) as usize;
        let clear_right = self.extra_left_right.saturating_sub(self.extra_right_cur) as usize;
        for x in 0..clear_left.min(PPU_X_PIXELS) {
            let dst = start + x * 4;
            if dst + 4 <= buffer.len() {
                buffer[dst..dst + 4].fill(0);
            }
        }
        let right_start = PPU_X_PIXELS.saturating_sub(clear_right);
        for x in right_start..PPU_X_PIXELS {
            let dst = start + x * 4;
            if dst + 4 <= buffer.len() {
                buffer[dst..dst + 4].fill(0);
            }
        }
        self.render_buffer = Some(buffer);
    }

    pub fn debug_pixel_compose_summary(&self, x: usize) -> String {
        let cwin = self.ppu_windows_calc(5);
        const CW_BITS_MOD: [u8; 8] = [0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00];
        let mut cw_clip_math = (((cwin.bits & CW_BITS_MOD[self.clip_mode as usize]) as u32)
            ^ CW_BITS_MOD[self.clip_mode as usize + 4] as u32)
            | ((((cwin.bits & CW_BITS_MOD[self.prevent_math_mode as usize]) as u32)
                ^ CW_BITS_MOD[self.prevent_math_mode as usize + 4] as u32)
                << 8);
        let ppu_x = x.saturating_add(self.extra_left_right as usize);
        let fixed_color = self.fixed_color_r as u16
            | ((self.fixed_color_g as u16) << 5)
            | ((self.fixed_color_b as u16) << 10);
        let clear_left = self.extra_left_right.saturating_sub(self.extra_left_cur) as usize;
        let clear_right = self.extra_left_right.saturating_sub(self.extra_right_cur) as usize;
        let right_start = PPU_X_PIXELS.saturating_sub(clear_right);

        for windex in 0..cwin.nr as usize {
            let left = (cwin.edges[windex] as i32 + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
            let right = (cwin.edges[windex + 1] as i32 + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
            if ppu_x >= left && ppu_x < right {
                let clip_color_mask = if cw_clip_math & 1 != 0 { 0x1f } else { 0 };
                let mut math_enabled_cur = if cw_clip_math & 0x100 != 0 {
                    self.math_enabled as u32
                } else {
                    0
                };
                let main = self.bg_buffers[0].data[ppu_x];
                let sub = self.bg_buffers[1].data[ppu_x];
                let main_layer = ((main >> 8) & 0x0f) as u32;
                let main_color = self.cgram[(main & 0xff) as usize];
                let mut color2 = fixed_color;
                let mut use_half_map = false;
                let mut math_applies = false;
                if math_enabled_cur != 0
                    && (fixed_color != 0 || self.half_color || self.add_subscreen)
                {
                    math_enabled_cur |= (self.add_subscreen as u32) << 8;
                    math_enabled_cur |= (self.subtract_color as u32) << 9;
                    math_applies = math_enabled_cur & (1 << main_layer) != 0;
                    if math_applies {
                        if math_enabled_cur & 0x100 != 0 {
                            let sub_color_index = sub & 0xff;
                            if sub_color_index != 0 {
                                use_half_map = self.half_color;
                                color2 = self.cgram[sub_color_index as usize];
                            }
                        } else {
                            use_half_map = self.half_color;
                        }
                    }
                }

                return format!(
                    "cwin=nr:{}/bits:{:02x}/edge:{}..{}/cw:{:03x} clip:{:02x} math_cur:{:03x} math_applies:{} half:{} sub_add:{} subtract:{} main:{:04x}/layer:{}/color:{:04x} sub:{:04x}/color2:{:04x} fixed:{:04x} clear:l{} r{} rs{} ppu_x{} halfmap:{}",
                    cwin.nr,
                    cwin.bits,
                    cwin.edges[windex],
                    cwin.edges[windex + 1],
                    cw_clip_math,
                    clip_color_mask,
                    math_enabled_cur,
                    math_applies,
                    self.half_color,
                    self.add_subscreen,
                    self.subtract_color,
                    main,
                    main_layer,
                    main_color,
                    sub,
                    color2,
                    fixed_color,
                    clear_left,
                    clear_right,
                    right_start,
                    ppu_x,
                    use_half_map,
                );
            }
            cw_clip_math >>= 1;
        }

        format!(
            "cwin=nr:{}/bits:{:02x}/miss ppu_x{} clear:l{} r{}",
            cwin.nr, cwin.bits, ppu_x, clear_left, clear_right
        )
    }

    /// `ppu_runLine`.
    pub fn run_line(&mut self, line: i32) {
        if line == 0 {
            self.refresh_brightness_cache();
            self.refresh_mosaic_modulo();
            self.clear_backdrop();
            self.line_has_sprites = false;
            return;
        }

        self.refresh_brightness_cache();
        self.refresh_mosaic_modulo();

        let line_index = (line - 1) as usize;
        if line >= 225 + self.extra_bottom_cur as i32 {
            self.fill_render_line(line_index, 0);
            return;
        }

        if line_index < usize::from(self.forced_blank_scanlines) {
            self.clear_backdrop();
            self.line_has_sprites = false;
            self.obj_fetch_buffer.data.fill(0x0500);
            self.obj_fetch_has_sprites = false;
            self.fill_render_line(line_index, 0);
            return;
        }

        let forced_blank_for_line = self.forced_blank_on_line(line_index);

        if self.render_flags.contains(PpuRenderFlags::NEW_RENDERER) && forced_blank_for_line {
            self.clear_backdrop();
            self.line_has_sprites = false;
            self.obj_fetch_buffer.data.fill(0x0500);
            self.obj_fetch_has_sprites = false;
            self.fill_render_line(line_index, 0);
            return;
        }

        if self.render_flags.contains(PpuRenderFlags::NEW_RENDERER) {
            self.fetch_sprites_for_line(line);
            self.line_has_sprites = self.obj_fetch_has_sprites;
            self.obj_buffer.clone_from(&self.obj_fetch_buffer);

            self.draw_whole_line(line as usize);

            self.obj_fetch_buffer.data.fill(0x0500);
            self.obj_fetch_has_sprites = false;
        } else {
            self.clear_backdrop();
            let final_forced_blank = self.forced_blank;
            self.forced_blank = forced_blank_for_line;
            if forced_blank_for_line {
                self.line_has_sprites = false;
            } else {
                let (has_sprites, buffer) = self.sprite_buffer_for_line(line);
                self.line_has_sprites = has_sprites;
                self.obj_buffer = buffer;
            }
            if self.bg_mode() == 7 {
                self.calculate_mode7_starts(line);
            }
            for x in 0..256 {
                self.handle_pixel_old(x, line);
            }
            self.forced_blank = final_forced_blank;
            if self.extra_left_right != 0 {
                let row_start = line_index.saturating_mul(self.render_pitch as usize);
                let Some(buffer) = self.render_buffer.as_mut() else {
                    return;
                };
                let left_bytes = self.extra_left_right as usize * 4;
                if row_start + left_bytes <= buffer.len() {
                    buffer[row_start..row_start + left_bytes].fill(0);
                }
                let right_start = row_start + (256 + self.extra_left_right as usize) * 4;
                if right_start + left_bytes <= buffer.len() {
                    buffer[right_start..right_start + left_bytes].fill(0);
                }
            }
        }
    }

    fn forced_blank_on_line(&self, line_index: usize) -> bool {
        self.forced_blank_from_scanline
            .is_some_and(|start| line_index >= usize::from(start))
            || (self.forced_blank && self.forced_blank_from_scanline.is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_display_force_blank_starts_at_its_owned_scanline() {
        let mut ppu = PpuState::new();
        ppu.forced_blank = true;
        ppu.forced_blank_from_scanline = Some(17);

        for line_index in 0..17 {
            assert!(!ppu.forced_blank_on_line(line_index));
        }
        assert!(ppu.forced_blank_on_line(17));
    }

    #[test]
    fn scanout_brightness_override_keeps_the_active_register_generation() {
        let mut ppu = PpuState::new();
        ppu.brightness = 0;
        ppu.scanout_brightness_override = Some(15);
        ppu.refresh_brightness_cache();

        assert_eq!(ppu.last_brightness_mult, 15);
        assert_eq!(ppu.brightness_mult[31], 255);
    }

    #[test]
    fn window_edges_start_as_snes9x_empty_intervals() {
        let mut ppu = PpuState::new();
        assert_eq!(
            (
                ppu.window1_left,
                ppu.window1_right,
                ppu.window2_left,
                ppu.window2_right
            ),
            (1, 0, 1, 0)
        );

        ppu.window1_left = 10;
        ppu.window1_right = 20;
        ppu.window2_left = 30;
        ppu.window2_right = 40;
        ppu.reset();
        assert_eq!(
            (
                ppu.window1_left,
                ppu.window1_right,
                ppu.window2_left,
                ppu.window2_right
            ),
            (1, 0, 1, 0)
        );
    }

    #[test]
    fn vmdatal_writes_low_byte() {
        let mut ppu = PpuState::new();
        ppu.write(0x16, 0x00);
        ppu.write(0x17, 0x00);
        ppu.write(0x15, 0x00); // increment after VMDATAL
        ppu.write(0x18, 0xaa);
        assert_eq!(ppu.vram[0], 0x00aa);
        assert_eq!(ppu.vram_pointer, 1);
    }

    #[test]
    fn vmdatah_writes_high_byte() {
        let mut ppu = PpuState::new();
        ppu.write(0x16, 0x00);
        ppu.write(0x17, 0x00);
        ppu.write(0x15, 0x80); // increment after VMDATAH
        ppu.write(0x19, 0x55);
        assert_eq!(ppu.vram[0], 0x5500);
        assert_eq!(ppu.vram_pointer, 1);
    }

    #[test]
    fn cgram_paired_writes() {
        let mut ppu = PpuState::new();
        ppu.write(0x21, 0); // CGADD = 0
        ppu.write(0x22, 0x34); // lo
        ppu.write(0x22, 0x12); // hi
        assert_eq!(ppu.cgram[0], 0x1234);
        assert_eq!(ppu.cgram_pointer, 1);
    }

    #[test]
    fn bg_scroll_registers_retain_full_latched_offsets() {
        let mut ppu = PpuState::new();

        ppu.write(0x0d, 0x91);
        ppu.write(0x0d, 0x24);
        ppu.write(0x0e, 0x00);
        ppu.write(0x0e, 0x41);

        assert_eq!(ppu.bg_layer[0].h_scroll, 0x2491);
        assert_eq!(ppu.bg_layer[0].v_scroll, 0x4100);
    }

    #[test]
    fn inidisp_force_blank() {
        let mut ppu = PpuState::new();
        ppu.write(0x00, 0x8f); // forced blank, brightness 15
        assert!(ppu.forced_blank);
        assert_eq!(ppu.brightness, 0xf);
    }

    #[test]
    fn master_brightness_scales_across_fifteen_intervals() {
        let mut ppu = PpuState::new();
        ppu.write(0x00, 0x0e);
        ppu.refresh_brightness_cache();
        assert_eq!(ppu.brightness_mult[31], 239);

        ppu.write(0x00, 0x0b);
        ppu.refresh_brightness_cache();
        assert_eq!(ppu.brightness_mult[31], 189);
    }

    #[test]
    fn master_brightness_rounds_low_levels_on_the_full_fifteen_step_scale() {
        let mut ppu = PpuState::new();
        ppu.write(0x00, 0x07);
        ppu.refresh_brightness_cache();

        assert_eq!(ppu.brightness_mult[31], 115);
    }

    #[test]
    fn run_line_clears_forced_blank_line() {
        let mut ppu = PpuState::new();
        let width = 256 + ppu.extra_left_right as usize * 2;
        ppu.render_pitch = (width * 4) as u32;
        ppu.render_buffer = Some(vec![0xff; width * 4]);
        ppu.forced_blank = true;
        ppu.run_line(1);
        assert!(ppu
            .render_buffer
            .as_ref()
            .unwrap()
            .iter()
            .all(|&byte| byte == 0));
    }

    #[test]
    fn run_line_blanks_only_the_published_active_display_prefix() {
        let mut ppu = PpuState::new();
        let width = 256 + ppu.extra_left_right as usize * 2;
        ppu.render_pitch = (width * 4) as u32;
        ppu.render_buffer = Some(vec![0xff; width * 2 * 4]);
        ppu.write(0x00, 0x0f);
        ppu.cgram[0] = 0x001f;
        ppu.forced_blank_scanlines = 1;

        ppu.run_line(1);
        ppu.run_line(2);

        let buffer = ppu.render_buffer.as_ref().unwrap();
        assert!(buffer[..width * 4].iter().all(|&byte| byte == 0));
        let second_line_pixel = width * 4 + ppu.extra_left_right as usize * 4;
        assert_eq!(
            &buffer[second_line_pixel..second_line_pixel + 4],
            &[0x00, 0x00, 0xff, 0x00]
        );
        assert_eq!(ppu.forced_blank_scanlines, 1);
    }

    #[test]
    fn legacy_renderer_blanks_only_the_published_active_display_suffix() {
        let mut ppu = PpuState::new();
        let width = 256 + ppu.extra_left_right as usize * 2;
        ppu.render_pitch = (width * 4) as u32;
        ppu.render_buffer = Some(vec![0; width * 2 * 4]);
        ppu.write(0x00, 0x0f);
        ppu.cgram[0] = 0x001f;
        ppu.forced_blank = false;
        ppu.forced_blank_from_scanline = Some(1);

        ppu.run_line(1);
        ppu.run_line(2);

        let buffer = ppu.render_buffer.as_ref().unwrap();
        let first_line_pixel = ppu.extra_left_right as usize * 4;
        assert_eq!(
            &buffer[first_line_pixel..first_line_pixel + 4],
            &[0x00, 0x00, 0xff, 0x00]
        );
        assert!(buffer[width * 4..width * 2 * 4]
            .iter()
            .all(|&byte| byte == 0));
    }

    #[test]
    fn run_line_draws_backdrop_when_visible() {
        let mut ppu = PpuState::new();
        let width = 256 + ppu.extra_left_right as usize * 2;
        ppu.render_pitch = (width * 4) as u32;
        ppu.render_buffer = Some(vec![0; width * 4]);
        ppu.write(0x00, 0x0f);
        ppu.cgram[0] = 0x001f;
        ppu.run_line(1);
        let pixel = ppu.extra_left_right as usize * 4;
        assert_eq!(
            &ppu.render_buffer.as_ref().unwrap()[pixel..pixel + 4],
            &[0x00, 0x00, 0xff, 0x00]
        );
    }

    #[test]
    fn obsel_decodes_base_name_offset_and_size_in_vram_words() {
        let mut ppu = PpuState::new();

        ppu.write(0x01, 0b1011_1010);

        assert_eq!(ppu.obj_tile_adr1, 0x4000);
        assert_eq!(ppu.obj_tile_adr2, 0x8000);
        assert_eq!(ppu.obj_size, 5);

        ppu.write(0x01, 2);
        assert_eq!(ppu.obj_tile_adr1, 0x4000);
        assert_eq!(ppu.obj_tile_adr2, 0x5000);
        assert_eq!(ppu.obj_size, 0);
    }

    #[test]
    fn bgmode_tracks_snes9x_mode1_bg3_priority_bit() {
        let mut ppu = PpuState::new();

        ppu.write(0x05, 0x01);
        assert_eq!(ppu.mode, 1);
        assert_eq!(ppu.bg_mode(), 1);
        assert!(!ppu.mode1_bg3_priority());

        ppu.write(0x05, 0x09);
        assert_eq!(ppu.mode, 9);
        assert_eq!(ppu.bg_mode(), 1);
        assert!(ppu.mode1_bg3_priority());

        ppu.write(0x05, 0x0a);
        assert_eq!(ppu.mode, 10);
        assert_eq!(ppu.bg_mode(), 2);
        assert!(!ppu.mode1_bg3_priority());
    }

    #[test]
    fn run_line_draws_reset_obj_from_tile_zero() {
        let mut ppu = PpuState::new();
        let width = 256 + ppu.extra_left_right as usize * 2;
        ppu.render_pitch = (width * 4) as u32;
        ppu.render_buffer = Some(vec![0; width * 224 * 4]);
        ppu.write(0x00, 0x0f);
        ppu.render_flags = PpuRenderFlags::NEW_RENDERER;
        ppu.mode = 1;
        ppu.screen_enabled[0] = 0x10;
        ppu.obj_tile_adr1 = 0;
        ppu.obj_tile_adr2 = 0x1000;
        ppu.oam[0] = 0x5555;
        ppu.vram[0] = 0x8000;
        ppu.cgram[0x82] = 0x5555;

        ppu.run_line(86);
        ppu.run_line(1);

        let pixel = 85 * width * 4 + (85 + ppu.extra_left_right as usize) * 4;
        assert_eq!(
            &ppu.render_buffer.as_ref().unwrap()[pixel..pixel + 4],
            &[0xad, 0x52, 0xad, 0xff]
        );
        let origin = ppu.extra_left_right as usize * 4;
        assert_eq!(
            &ppu.render_buffer.as_ref().unwrap()[origin..origin + 4],
            &[0xad, 0x52, 0xad, 0xff]
        );
    }

    #[test]
    fn begin_drawing_refreshes_mode7_color_map() {
        let mut ppu = PpuState::new();
        ppu.mode = 7;
        ppu.forced_blank = false;
        ppu.brightness = 0x0f;
        ppu.cgram[3] = 0x7c1f;
        let mut buffer = vec![0; 256 * 4];

        ppu.begin_drawing(
            &mut buffer,
            256 * 4,
            PpuRenderFlags::MODE7_4X4 | PpuRenderFlags::NEW_RENDERER,
        );

        assert_eq!(ppu.color_map_rgb[3], 0x00ff00ff);
    }

    #[test]
    fn c_saveload_layout_matches_ppu_c_byte_count() {
        let mut ppu = PpuState::new();
        ppu.vram[0] = 0x1234;
        ppu.vram[0x7fff] = 0xabcd;
        ppu.cgram[0] = 0x4567;
        ppu.bg_layer[2].tilemap_wider = true;
        ppu.bg_layer[2].tilemap_higher = true;
        ppu.bg_layer[2].tilemap_adr = 0x789a;
        ppu.bg_layer[2].tile_adr = 0xeeee;

        let bytes = ppu.save_c_saveload();
        assert_eq!(bytes.len(), PpuState::C_SAVELOAD_SIZE);
        assert_eq!(&bytes[0..2], &0x1234u16.to_le_bytes());
        assert_eq!(&bytes[0xfffe..0x10000], &0xabcdu16.to_le_bytes());
        let cgram_start = 0x8000 * 2 + 10;
        assert_eq!(
            &bytes[cgram_start..cgram_start + 2],
            &0x4567u16.to_le_bytes()
        );
        let bg2_start = cgram_start + 512 + 556 + 520 + 2 * 12;
        assert_eq!(bytes[bg2_start + 4], 1);
        assert_eq!(bytes[bg2_start + 5], 1);
        assert_eq!(
            &bytes[bg2_start + 6..bg2_start + 8],
            &0x789au16.to_le_bytes()
        );

        let mut loaded = PpuState::new();
        loaded.load_c_saveload(&bytes).unwrap();
        assert_eq!(loaded.vram[0], 0x1234);
        assert_eq!(loaded.vram[0x7fff], 0xabcd);
        assert_eq!(loaded.cgram[0], 0x4567);
        assert!(loaded.bg_layer[2].tilemap_wider);
        assert!(loaded.bg_layer[2].tilemap_higher);
        assert_eq!(loaded.bg_layer[2].tilemap_adr, 0x789a);
        assert_eq!(loaded.bg_layer[2].tile_adr, 0);
    }

    #[test]
    fn beam_counter_reads_follow_the_latched_raster_position() {
        let mut ppu = PpuState::new();
        ppu.latch_counters(0x0123, 0x00c0);

        // STAT78 resets the high/low read toggles. Zelda's spotlight wait
        // reads it between SLHV and OPVCT, then compares OPVCT's low byte.
        assert_eq!(ppu.read(0x3f), 0xff);
        assert_eq!(ppu.read(0x3d), 0xc0);
        assert_eq!(ppu.read(0x3d), 0x00);
        assert_eq!(ppu.read(0x3c), 0x23);
        assert_eq!(ppu.read(0x3c), 0x01);
    }
}
