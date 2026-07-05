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

pub type RawScanlineRegs = (u8, u8, u8, u8, u8, [u16; 4], [u16; 4], [i16; 8]);
pub type RawScanlineFrame = [RawScanlineRegs; 224];

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

/// Raw register/slice snapshot used to construct a [`GpuFrame`] without tying
/// the renderer crate to an emulator-state type.
#[derive(Clone, Copy)]
pub struct GpuFrameRegisterSnapshot<'a> {
    pub vram: &'a [u16],
    pub oam: &'a [u16],
    pub mode: u8,
    pub bg: [BgLayerRegs; 4],
    pub obj: ObjRegs,
    pub mosaic_enabled: u8,
    pub mosaic_size: u8,
    pub extra_left_right: u8,
    pub mode7: Mode7Regs,
    pub screen_enabled: [u8; 2],
    pub screen_windowed: [u8; 2],
    pub brightness: u8,
    pub forced_blank: bool,
    pub math_enabled: u8,
    pub subtract_color: bool,
    pub half_color: bool,
    pub fixed_color_r: u8,
    pub fixed_color_g: u8,
    pub fixed_color_b: u8,
    pub add_subscreen: bool,
    pub clip_mode: u8,
    pub prevent_math_mode: u8,
    pub windowsel: u32,
}

pub struct GpuFrameCaptureInput<'a> {
    pub registers: GpuFrameRegisterSnapshot<'a>,
    pub cgram: &'a [u16],
    pub raw_scanlines: &'a RawScanlineFrame,
}

impl<'a> GpuFrame<'a> {
    pub fn scanlines_from_raw(raw: &RawScanlineFrame) -> Box<[ScanlineRegs; 224]> {
        let mut result = Box::new([ScanlineRegs::default(); 224]);
        for (dst, &(w1l, w1r, w2l, w2r, tm, bg_h_scroll, bg_v_scroll, mode7_matrix)) in
            result.iter_mut().zip(raw.iter())
        {
            dst.window1_left = w1l;
            dst.window1_right = w1r;
            dst.window2_left = w2l;
            dst.window2_right = w2r;
            dst.screen_enabled_main = tm;
            dst.bg_h_scroll = bg_h_scroll;
            dst.bg_v_scroll = bg_v_scroll;
            dst.mode7_matrix = mode7_matrix;
        }
        result
    }

    pub fn from_source_and_raw_scanlines<S>(
        source: &S,
        cgram: &'a [u16],
        raw_scanlines: &RawScanlineFrame,
    ) -> Self
    where
        S: GpuFrameSource<'a> + ?Sized,
    {
        Self::from_source(source, cgram, Self::scanlines_from_raw(raw_scanlines))
    }

    pub fn from_capture_input(input: GpuFrameCaptureInput<'a>) -> Self {
        let registers = input.registers;
        Self {
            vram: registers.vram,
            cgram: input.cgram,
            oam: registers.oam,
            mode: registers.mode,
            bg: registers.bg,
            obj: registers.obj,
            mosaic_enabled: registers.mosaic_enabled,
            mosaic_size: registers.mosaic_size,
            extra_left_right: registers.extra_left_right,
            mode7: registers.mode7,
            screen_enabled: registers.screen_enabled,
            screen_windowed: registers.screen_windowed,
            brightness: registers.brightness,
            forced_blank: registers.forced_blank,
            math_enabled: registers.math_enabled,
            subtract_color: registers.subtract_color,
            half_color: registers.half_color,
            fixed_color_r: registers.fixed_color_r,
            fixed_color_g: registers.fixed_color_g,
            fixed_color_b: registers.fixed_color_b,
            add_subscreen: registers.add_subscreen,
            clip_mode: registers.clip_mode,
            prevent_math_mode: registers.prevent_math_mode,
            windowsel_cm: ((registers.windowsel >> 20) & 0xF) as u8,
            windowsel: registers.windowsel,
            scanlines: Self::scanlines_from_raw(input.raw_scanlines),
        }
    }

    pub fn from_source<S>(source: &S, cgram: &'a [u16], scanlines: Box<[ScanlineRegs; 224]>) -> Self
    where
        S: GpuFrameSource<'a> + ?Sized,
    {
        Self {
            vram: source.vram(),
            cgram,
            oam: source.oam(),
            mode: source.mode(),
            bg: std::array::from_fn(|layer| BgLayerRegs {
                h_scroll: source.bg_h_scroll(layer),
                v_scroll: source.bg_v_scroll(layer),
                tilemap_wider: source.bg_tilemap_wider(layer),
                tilemap_higher: source.bg_tilemap_higher(layer),
                tilemap_adr: source.bg_tilemap_adr(layer),
                tile_adr: source.bg_tile_adr(layer),
            }),
            obj: ObjRegs {
                tile_adr1: source.obj_tile_adr1(),
                tile_adr2: source.obj_tile_adr2(),
                obj_size: source.obj_size(),
            },
            mosaic_enabled: source.mosaic_enabled(),
            mosaic_size: source.mosaic_size(),
            extra_left_right: source.extra_left_right(),
            mode7: Mode7Regs {
                matrix: source.mode7_matrix(),
                large_field: source.mode7_large_field(),
                char_fill: source.mode7_char_fill(),
                x_flip: source.mode7_x_flip(),
                y_flip: source.mode7_y_flip(),
                ext_bg_always_zero: source.mode7_ext_bg_always_zero(),
            },
            screen_enabled: source.screen_enabled(),
            screen_windowed: source.screen_windowed(),
            brightness: source.brightness(),
            forced_blank: source.forced_blank(),
            math_enabled: source.math_enabled(),
            subtract_color: source.subtract_color(),
            half_color: source.half_color(),
            fixed_color_r: source.fixed_color_r(),
            fixed_color_g: source.fixed_color_g(),
            fixed_color_b: source.fixed_color_b(),
            add_subscreen: source.add_subscreen(),
            clip_mode: source.clip_mode(),
            prevent_math_mode: source.prevent_math_mode(),
            windowsel_cm: ((source.windowsel() >> 20) & 0xF) as u8,
            windowsel: source.windowsel(),
            scanlines,
        }
    }
}

pub trait GpuFrameSource<'a> {
    fn vram(&self) -> &'a [u16];
    fn oam(&self) -> &'a [u16];
    fn mode(&self) -> u8;
    fn bg_h_scroll(&self, layer: usize) -> u16;
    fn bg_v_scroll(&self, layer: usize) -> u16;
    fn bg_tilemap_wider(&self, layer: usize) -> bool;
    fn bg_tilemap_higher(&self, layer: usize) -> bool;
    fn bg_tilemap_adr(&self, layer: usize) -> u16;
    fn bg_tile_adr(&self, layer: usize) -> u16;
    fn obj_tile_adr1(&self) -> u16;
    fn obj_tile_adr2(&self) -> u16;
    fn obj_size(&self) -> u8;
    fn mosaic_enabled(&self) -> u8;
    fn mosaic_size(&self) -> u8;
    fn extra_left_right(&self) -> u8;
    fn mode7_matrix(&self) -> [i16; 8];
    fn mode7_large_field(&self) -> bool;
    fn mode7_char_fill(&self) -> bool;
    fn mode7_x_flip(&self) -> bool;
    fn mode7_y_flip(&self) -> bool;
    fn mode7_ext_bg_always_zero(&self) -> bool;
    fn screen_enabled(&self) -> [u8; 2];
    fn screen_windowed(&self) -> [u8; 2];
    fn brightness(&self) -> u8;
    fn forced_blank(&self) -> bool;
    fn math_enabled(&self) -> u8;
    fn subtract_color(&self) -> bool;
    fn half_color(&self) -> bool;
    fn fixed_color_r(&self) -> u8;
    fn fixed_color_g(&self) -> u8;
    fn fixed_color_b(&self) -> u8;
    fn add_subscreen(&self) -> bool;
    fn clip_mode(&self) -> u8;
    fn prevent_math_mode(&self) -> u8;
    fn windowsel(&self) -> u32;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestFrameSource<'a> {
        vram: &'a [u16],
        oam: &'a [u16],
    }

    impl<'a> GpuFrameSource<'a> for TestFrameSource<'a> {
        fn vram(&self) -> &'a [u16] {
            self.vram
        }

        fn oam(&self) -> &'a [u16] {
            self.oam
        }

        fn mode(&self) -> u8 {
            7
        }

        fn bg_h_scroll(&self, layer: usize) -> u16 {
            10 + layer as u16
        }

        fn bg_v_scroll(&self, layer: usize) -> u16 {
            20 + layer as u16
        }

        fn bg_tilemap_wider(&self, layer: usize) -> bool {
            layer == 1
        }

        fn bg_tilemap_higher(&self, layer: usize) -> bool {
            layer == 2
        }

        fn bg_tilemap_adr(&self, layer: usize) -> u16 {
            0x1000 + layer as u16
        }

        fn bg_tile_adr(&self, layer: usize) -> u16 {
            0x2000 + layer as u16
        }

        fn obj_tile_adr1(&self) -> u16 {
            0x3000
        }

        fn obj_tile_adr2(&self) -> u16 {
            0x4000
        }

        fn obj_size(&self) -> u8 {
            2
        }

        fn mosaic_enabled(&self) -> u8 {
            0x03
        }

        fn mosaic_size(&self) -> u8 {
            4
        }

        fn extra_left_right(&self) -> u8 {
            8
        }

        fn mode7_matrix(&self) -> [i16; 8] {
            [1, 2, 3, 4, 5, 6, 7, 8]
        }

        fn mode7_large_field(&self) -> bool {
            true
        }

        fn mode7_char_fill(&self) -> bool {
            false
        }

        fn mode7_x_flip(&self) -> bool {
            true
        }

        fn mode7_y_flip(&self) -> bool {
            false
        }

        fn mode7_ext_bg_always_zero(&self) -> bool {
            true
        }

        fn screen_enabled(&self) -> [u8; 2] {
            [0x11, 0x22]
        }

        fn screen_windowed(&self) -> [u8; 2] {
            [0x33, 0x44]
        }

        fn brightness(&self) -> u8 {
            15
        }

        fn forced_blank(&self) -> bool {
            true
        }

        fn math_enabled(&self) -> u8 {
            0x2f
        }

        fn subtract_color(&self) -> bool {
            true
        }

        fn half_color(&self) -> bool {
            true
        }

        fn fixed_color_r(&self) -> u8 {
            1
        }

        fn fixed_color_g(&self) -> u8 {
            2
        }

        fn fixed_color_b(&self) -> u8 {
            3
        }

        fn add_subscreen(&self) -> bool {
            true
        }

        fn clip_mode(&self) -> u8 {
            1
        }

        fn prevent_math_mode(&self) -> u8 {
            2
        }

        fn windowsel(&self) -> u32 {
            0x0050_0000
        }
    }

    #[test]
    fn gpu_frame_from_source_owns_register_assembly() {
        let vram = [0x1111, 0x2222];
        let cgram = [0x3333, 0x4444];
        let oam = [0x5555, 0x6666];
        let source = TestFrameSource {
            vram: &vram,
            oam: &oam,
        };
        let frame =
            GpuFrame::from_source(&source, &cgram, Box::new([ScanlineRegs::default(); 224]));

        assert_eq!(frame.vram, &vram);
        assert_eq!(frame.cgram, &cgram);
        assert_eq!(frame.oam, &oam);
        assert_eq!(frame.mode, 7);
        assert_eq!(frame.bg[2].h_scroll, 12);
        assert_eq!(frame.bg[2].v_scroll, 22);
        assert!(frame.bg[1].tilemap_wider);
        assert!(frame.bg[2].tilemap_higher);
        assert_eq!(frame.bg[3].tilemap_adr, 0x1003);
        assert_eq!(frame.bg[3].tile_adr, 0x2003);
        assert_eq!(frame.obj.tile_adr1, 0x3000);
        assert_eq!(frame.obj.tile_adr2, 0x4000);
        assert_eq!(frame.obj.obj_size, 2);
        assert_eq!(frame.mode7.matrix, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(frame.mode7.large_field);
        assert!(frame.mode7.x_flip);
        assert!(frame.mode7.ext_bg_always_zero);
        assert_eq!(frame.screen_enabled, [0x11, 0x22]);
        assert_eq!(frame.screen_windowed, [0x33, 0x44]);
        assert_eq!(frame.brightness, 15);
        assert!(frame.forced_blank);
        assert_eq!(frame.math_enabled, 0x2f);
        assert!(frame.subtract_color);
        assert!(frame.half_color);
        assert_eq!(frame.fixed_color_r, 1);
        assert_eq!(frame.fixed_color_g, 2);
        assert_eq!(frame.fixed_color_b, 3);
        assert!(frame.add_subscreen);
        assert_eq!(frame.clip_mode, 1);
        assert_eq!(frame.prevent_math_mode, 2);
        assert_eq!(frame.windowsel_cm, 5);
        assert_eq!(frame.windowsel, 0x0050_0000);
    }

    #[test]
    fn gpu_frame_from_source_and_raw_scanlines_owns_scanline_conversion() {
        let vram = [0x1111, 0x2222];
        let cgram = [0x3333, 0x4444];
        let oam = [0x5555, 0x6666];
        let source = TestFrameSource {
            vram: &vram,
            oam: &oam,
        };
        let mut raw = [(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8]); 224];
        raw[7] = (
            1,
            2,
            3,
            4,
            0x1f,
            [10, 11, 12, 13],
            [20, 21, 22, 23],
            [30, 31, 32, 33, 34, 35, 36, 37],
        );

        let frame = GpuFrame::from_source_and_raw_scanlines(&source, &cgram, &raw);

        assert_eq!(frame.scanlines[7].window1_left, 1);
        assert_eq!(frame.scanlines[7].window1_right, 2);
        assert_eq!(frame.scanlines[7].window2_left, 3);
        assert_eq!(frame.scanlines[7].window2_right, 4);
        assert_eq!(frame.scanlines[7].screen_enabled_main, 0x1f);
        assert_eq!(frame.scanlines[7].bg_h_scroll, [10, 11, 12, 13]);
        assert_eq!(frame.scanlines[7].bg_v_scroll, [20, 21, 22, 23]);
        assert_eq!(
            frame.scanlines[7].mode7_matrix,
            [30, 31, 32, 33, 34, 35, 36, 37]
        );
        assert_eq!(frame.vram, &vram);
        assert_eq!(frame.cgram, &cgram);
        assert_eq!(frame.oam, &oam);
    }

    #[test]
    fn gpu_frame_from_capture_input_owns_register_snapshot_adaptation() {
        let vram = [0x1111, 0x2222];
        let cgram = [0x3333, 0x4444];
        let oam = [0x5555, 0x6666];
        let mut raw = [(0, 0, 0, 0, 0, [0; 4], [0; 4], [0; 8]); 224];
        raw[3] = (
            9,
            10,
            11,
            12,
            0x12,
            [100, 101, 102, 103],
            [200, 201, 202, 203],
            [-1, -2, -3, -4, -5, -6, -7, -8],
        );
        let registers = GpuFrameRegisterSnapshot {
            vram: &vram,
            oam: &oam,
            mode: 7,
            bg: [
                BgLayerRegs {
                    h_scroll: 1,
                    v_scroll: 2,
                    tilemap_wider: true,
                    tilemap_higher: false,
                    tilemap_adr: 0x0400,
                    tile_adr: 0x1000,
                },
                BgLayerRegs::default(),
                BgLayerRegs::default(),
                BgLayerRegs::default(),
            ],
            obj: ObjRegs {
                tile_adr1: 0x2000,
                tile_adr2: 0x3000,
                obj_size: 5,
            },
            mosaic_enabled: 0x7,
            mosaic_size: 3,
            extra_left_right: 16,
            mode7: Mode7Regs {
                matrix: [1, 2, 3, 4, 5, 6, 7, 8],
                large_field: true,
                char_fill: true,
                x_flip: true,
                y_flip: false,
                ext_bg_always_zero: true,
            },
            screen_enabled: [0x15, 0x04],
            screen_windowed: [0x01, 0x02],
            brightness: 14,
            forced_blank: true,
            math_enabled: 0x2f,
            subtract_color: true,
            half_color: true,
            fixed_color_r: 3,
            fixed_color_g: 4,
            fixed_color_b: 5,
            add_subscreen: true,
            clip_mode: 2,
            prevent_math_mode: 1,
            windowsel: 0x0030_0000,
        };

        let frame = GpuFrame::from_capture_input(GpuFrameCaptureInput {
            registers,
            cgram: &cgram,
            raw_scanlines: &raw,
        });

        assert_eq!(frame.vram, &vram);
        assert_eq!(frame.cgram, &cgram);
        assert_eq!(frame.oam, &oam);
        assert_eq!(frame.mode, 7);
        assert_eq!(frame.bg[0].tilemap_adr, 0x0400);
        assert_eq!(frame.obj.tile_adr2, 0x3000);
        assert!(frame.mode7.large_field);
        assert_eq!(frame.screen_enabled, [0x15, 0x04]);
        assert_eq!(frame.brightness, 14);
        assert_eq!(frame.windowsel_cm, 3);
        assert_eq!(frame.scanlines[3].window1_left, 9);
        assert_eq!(frame.scanlines[3].bg_h_scroll, [100, 101, 102, 103]);
        assert_eq!(
            frame.scanlines[3].mode7_matrix,
            [-1, -2, -3, -4, -5, -6, -7, -8]
        );
    }
}
