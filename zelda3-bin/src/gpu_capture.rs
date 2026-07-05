use std::path::Path;
use std::process;

use crate::gpu_readback::{GpuReadbackRenderer, GpuRgbaReadbackFrame};
use platform::NativeFrontend;
use renderer::{GpuFrame, RawScanlineFrame};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

const PLAYER_IS_INDOORS: usize = 0x001b;
const MAIN_MODULE_INDEX: usize = 0x10;

pub struct LiveGpuFrameCapture {
    ppu: snes::ppu::PpuState,
    cgram: Vec<u16>,
    raw_scanlines: Box<RawScanlineFrame>,
    source_entries: Vec<zelda3::LogicalChrSrc>,
    mode7_source_chars: Option<Vec<u8>>,
    main_module: u8,
    player_indoors: u8,
}

impl LiveGpuFrameCapture {
    pub fn from_game(game: &mut ZeldaState) -> Self {
        let cgram = game.cgram_after_first_hdma_line();
        let raw_scanlines = game.ppu_scanline_windows();
        let ppu = game.ppu.clone();
        let source_entries = game.vram_chr_source().as_slice().to_vec();
        let mode7_source_chars = game.mode7_character_source().map(<[u8]>::to_vec);
        let main_module = game.ram[MAIN_MODULE_INDEX];
        let player_indoors = game.ram[PLAYER_IS_INDOORS];
        Self {
            ppu,
            cgram,
            raw_scanlines,
            source_entries,
            mode7_source_chars,
            main_module,
            player_indoors,
        }
    }

    pub fn capture_input(&self) -> renderer::GpuFrameCaptureInput<'_> {
        gpu_frame_capture_from_ppu(&self.ppu, &self.cgram, self.raw_scanlines.as_ref())
    }

    pub fn gpu_frame(&self) -> GpuFrame<'_> {
        GpuFrame::from_capture_input(self.capture_input())
    }

    pub fn ppu(&self) -> &snes::ppu::PpuState {
        &self.ppu
    }

    pub fn cgram(&self) -> &[u16] {
        &self.cgram
    }

    pub fn raw_scanlines(&self) -> &RawScanlineFrame {
        self.raw_scanlines.as_ref()
    }

    pub fn source_entries(&self) -> &[zelda3::LogicalChrSrc] {
        &self.source_entries
    }

    pub fn mode7_source_chars(&self) -> Option<&[u8]> {
        self.mode7_source_chars.as_deref()
    }

    pub fn main_module(&self) -> u8 {
        self.main_module
    }

    pub fn player_indoors(&self) -> u8 {
        self.player_indoors
    }

    pub fn modern_asset_present_input<'a>(
        &'a self,
        resources: &'a renderer::ModernAssetFrameResources,
        stats: &'a mut renderer::ModernAssetLiveStats,
    ) -> renderer::ModernAssetFrameLivePresentInput<'a, 'a, zelda3::LogicalChrSrc> {
        renderer::ModernAssetFrameLivePresentInput {
            frame: self.capture_input(),
            source_entries: &self.source_entries,
            mode7_source_chars: self.mode7_source_chars(),
            resources,
            stats,
            player_indoors: self.player_indoors,
        }
    }
}

/// Modern asset resources + HD override store for the live present path, loaded
/// once. The renderer crate owns which atlases each renderer mode requires and
/// how they route through GPU/software presentation.
struct GpuPlayRenderer {
    modern_assets: renderer::ModernAssetFrameResources,
    variant_live_stats: renderer::ModernAssetLiveStats,
}

impl GpuPlayRenderer {
    fn new() -> Self {
        let modern_assets = renderer::ModernAssetFrameResources::load_from_env(Path::new("."))
            .unwrap_or_else(|e| {
                eprintln!("modern asset load failed: {e}");
                process::exit(2);
            });
        Self {
            modern_assets,
            variant_live_stats: renderer::ModernAssetLiveStats::from_env(),
        }
    }
}

impl crate::play_renderer::PlayRendererBackend for GpuPlayRenderer {
    fn name(&self) -> &'static str {
        "gpu_render"
    }

    fn configure_frontend(&self, frontend: &mut NativeFrontend) {
        // Default (unset) now selects `assets-variant-gpu`; `ZELDA3_VARIANT_ATLAS=off`
        // selects the older `assets-anim-gpu` path, and `ZELDA3_RENDERER=classic`
        // opts back into the wgpu PPU. `ZELDA3_RENDERER=modern`/`modern-compare`
        // route through the modern software live-VRAM path. Asset modes map to
        // Modern because `RendererMode::parse` only recognizes "modern" and
        // "modern-compare"; GPU asset modes intercept Mode 7 and source-atlas
        // misses below, so the default path does not need
        // `FrameRenderer::render_modern_frame`'s CPU compositor.
        frontend.set_renderer_mode(renderer::RendererMode::from_effective_env());
    }

    fn present_frame(
        &mut self,
        game: &mut ZeldaState,
        frontend: &mut NativeFrontend,
        _frame: &mut [u8],
        _render_flags: PpuRenderFlags,
    ) {
        let capture = LiveGpuFrameCapture::from_game(game);
        let report = frontend.present_modern_asset_live_frame_from_entries(
            capture.modern_asset_present_input(&self.modern_assets, &mut self.variant_live_stats),
        );
        if let Some(line) = report.failure_line() {
            eprintln!("{line}");
            process::exit(2);
        }
    }
}

pub(crate) fn new_gpu_play_renderer() -> Box<dyn crate::play_renderer::PlayRendererBackend> {
    Box::new(GpuPlayRenderer::new())
}

pub(crate) fn capture_gpu_frame_from_game(game: &mut ZeldaState) -> LiveGpuFrameCapture {
    LiveGpuFrameCapture::from_game(game)
}

pub(crate) fn render_live_game_gpu_frame_rgba(
    game: &mut ZeldaState,
    width: u32,
    height: u32,
) -> GpuRgbaReadbackFrame {
    let capture = capture_gpu_frame_from_game(game);
    let mut readback = GpuReadbackRenderer::new(width, height);
    readback.render_live_gpu_capture_rgba(&capture)
}

pub(crate) fn render_hd_capture_from_game(
    game: &mut ZeldaState,
    atlas: &renderer::modern_source_atlas::ModernSourceAtlas,
) -> Option<renderer::hd_authoring::HdCaptureFrame> {
    let capture = capture_gpu_frame_from_game(game);
    let gpu_frame = capture.gpu_frame();
    (gpu_frame.mode != 7).then(|| render_hd_capture_from_gpu_capture(&capture, atlas))
}

fn render_hd_capture_from_gpu_capture(
    capture: &LiveGpuFrameCapture,
    atlas: &renderer::modern_source_atlas::ModernSourceAtlas,
) -> renderer::hd_authoring::HdCaptureFrame {
    let gpu_frame = capture.gpu_frame();
    let source_table = renderer::source_table_from_entries(capture.source_entries());
    renderer::hd_authoring::render_hd_capture_from_sources(&gpu_frame, &source_table, atlas)
}

fn gpu_frame_capture_from_ppu<'a>(
    ppu: &'a snes::ppu::PpuState,
    cgram: &'a [u16],
    raw_scanlines: &'a RawScanlineFrame,
) -> renderer::GpuFrameCaptureInput<'a> {
    renderer::GpuFrameCaptureInput {
        registers: gpu_frame_register_snapshot_from_ppu(ppu),
        cgram,
        raw_scanlines,
    }
}

fn gpu_frame_register_snapshot_from_ppu<'a>(
    ppu: &'a snes::ppu::PpuState,
) -> renderer::GpuFrameRegisterSnapshot<'a> {
    renderer::GpuFrameRegisterSnapshot {
        vram: &ppu.vram,
        oam: &ppu.oam,
        mode: ppu.mode,
        bg: std::array::from_fn(|layer| renderer::BgLayerRegs {
            h_scroll: ppu.bg_layer[layer].h_scroll,
            v_scroll: ppu.bg_layer[layer].v_scroll,
            tilemap_wider: ppu.bg_layer[layer].tilemap_wider,
            tilemap_higher: ppu.bg_layer[layer].tilemap_higher,
            tilemap_adr: ppu.bg_layer[layer].tilemap_adr,
            tile_adr: ppu.bg_layer[layer].tile_adr,
        }),
        obj: renderer::ObjRegs {
            tile_adr1: ppu.obj_tile_adr1,
            tile_adr2: ppu.obj_tile_adr2,
            obj_size: ppu.obj_size,
        },
        mosaic_enabled: ppu.mosaic_enabled,
        mosaic_size: ppu.mosaic_size,
        extra_left_right: ppu.extra_left_right,
        mode7: renderer::Mode7Regs {
            matrix: ppu.m7_matrix,
            large_field: ppu.m7_large_field,
            char_fill: ppu.m7_char_fill,
            x_flip: ppu.m7_x_flip,
            y_flip: ppu.m7_y_flip,
            ext_bg_always_zero: ppu.m7_ext_bg_always_zero,
        },
        screen_enabled: ppu.screen_enabled,
        screen_windowed: ppu.screen_windowed,
        brightness: ppu.brightness,
        forced_blank: ppu.forced_blank,
        math_enabled: ppu.math_enabled,
        subtract_color: ppu.subtract_color,
        half_color: ppu.half_color,
        fixed_color_r: ppu.fixed_color_r,
        fixed_color_g: ppu.fixed_color_g,
        fixed_color_b: ppu.fixed_color_b,
        add_subscreen: ppu.add_subscreen,
        clip_mode: ppu.clip_mode,
        prevent_math_mode: ppu.prevent_math_mode,
        windowsel: ppu.windowsel,
    }
}
