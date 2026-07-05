use std::path::Path;
use std::process;

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
    main_module: u8,
    player_indoors: u8,
}

impl LiveGpuFrameCapture {
    pub fn from_game(game: &mut ZeldaState) -> Self {
        let cgram = game.cgram_after_first_hdma_line();
        let raw_scanlines = game.ppu_scanline_windows();
        let ppu = game.ppu.clone();
        let source_entries = game.vram_chr_source().as_slice().to_vec();
        let main_module = game.ram[MAIN_MODULE_INDEX];
        let player_indoors = game.ram[PLAYER_IS_INDOORS];
        Self {
            ppu,
            cgram,
            raw_scanlines,
            source_entries,
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

    pub fn modern_asset_present_input<'a>(
        &'a self,
        resources: &'a renderer::ModernAssetFrameResources,
        stats: &'a mut renderer::ModernAssetLiveStats,
    ) -> renderer::ModernAssetFrameLivePresentInput<'a, 'a, zelda3::LogicalChrSrc> {
        renderer::ModernAssetFrameLivePresentInput {
            frame: self.capture_input(),
            source_entries: &self.source_entries,
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

pub(crate) struct ModernCompareModeDefaults {
    pub(crate) enable_modern_render_compare: bool,
    pub(crate) note: Option<&'static str>,
}

pub(crate) struct ModernIndexCompareRun {
    config: renderer::ModernIndexCompareRunConfig,
    stats: renderer::ModernIndexCompareStats,
    resources: Option<renderer::ModernIndexCompareResources>,
    allow_source_cpu_fallback: bool,
}

pub(crate) struct GpuRenderCompareRun {
    stride: u32,
    quiet: bool,
    compared: u32,
    last_frame: u32,
    last_hash: u32,
}

struct ModernAtlasCompareRun {
    stride: u32,
    resources: renderer::ModernAtlasCompareResources,
}

pub(crate) struct PlayGpuRenderCompareSession {
    readback: GpuReadbackRenderer,
    render_frame: Vec<u8>,
    gpu_render_compare: GpuRenderCompareRun,
    modern_atlas_compare: ModernAtlasCompareRun,
    modern_index_compare: ModernIndexCompareRun,
}

pub(crate) struct GpuReadbackRenderer {
    offscreen: renderer::OffscreenRenderer,
}

pub(crate) struct GpuRgbaReadbackFrame {
    rgba: Vec<u8>,
}

pub(crate) struct OptionalGpuReadbackRenderer {
    renderer: Option<GpuReadbackRenderer>,
}

pub(crate) struct ReplayRenderHashCapture {
    capture: LiveGpuFrameCapture,
}

pub(crate) struct ReplayRenderHashGpuReadback {
    frame: GpuRgbaReadbackFrame,
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

fn new_gpu_readback_renderer(width: u32, height: u32) -> GpuReadbackRenderer {
    GpuReadbackRenderer {
        offscreen: pollster::block_on(renderer::OffscreenRenderer::new(width, height)),
    }
}

fn optional_gpu_readback_renderer(
    required: bool,
    width: u32,
    height: u32,
) -> OptionalGpuReadbackRenderer {
    OptionalGpuReadbackRenderer {
        renderer: required.then(|| new_gpu_readback_renderer(width, height)),
    }
}

pub(crate) fn replay_optional_gpu_readback_renderer(
    render_hash_log: u32,
    gpu_render_compare: &GpuRenderCompareRun,
    render_hash_dump_enabled: bool,
    dump_frame_enabled: bool,
    modern_index_compare: &ModernIndexCompareRun,
) -> OptionalGpuReadbackRenderer {
    optional_gpu_readback_renderer(
        render_hash_log != 0
            || gpu_render_compare.enabled()
            || render_hash_dump_enabled
            || dump_frame_enabled
            || modern_index_compare.enabled(),
        256,
        224,
    )
}

pub(crate) fn render_live_game_gpu_frame_rgba(
    game: &mut ZeldaState,
    width: u32,
    height: u32,
) -> GpuRgbaReadbackFrame {
    let capture = capture_gpu_frame_from_game(game);
    let mut readback = new_gpu_readback_renderer(width, height);
    readback.render_live_gpu_capture_rgba(&capture)
}

pub(crate) fn modern_compare_mode_defaults_from_env() -> ModernCompareModeDefaults {
    let renderer_mode =
        renderer::RendererMode::parse(std::env::var("ZELDA3_RENDERER").ok().as_deref());
    let enable_modern_render_compare = renderer_mode == renderer::RendererMode::ModernCompare
        || renderer_mode == renderer::RendererMode::Modern;
    let note = if renderer_mode == renderer::RendererMode::Modern {
        Some(
            "note: ZELDA3_RENDERER=modern is experimental; modern path cannot render most content — running as modern-compare",
        )
    } else {
        None
    };
    ModernCompareModeDefaults {
        enable_modern_render_compare,
        note,
    }
}

pub(crate) fn modern_index_compare_run_from_env() -> ModernIndexCompareRun {
    ModernIndexCompareRun {
        config: renderer::ModernIndexCompareRunConfig::default(),
        stats: renderer::ModernIndexCompareStats::from_env(),
        resources: None,
        allow_source_cpu_fallback: false,
    }
}

pub(crate) fn gpu_render_compare_run(stride: u32, quiet: bool) -> GpuRenderCompareRun {
    GpuRenderCompareRun {
        stride,
        quiet,
        compared: 0,
        last_frame: 0,
        last_hash: 0,
    }
}

fn modern_atlas_compare_run(stride: u32, root: &Path) -> Result<ModernAtlasCompareRun, String> {
    let resources = renderer::ModernAtlasCompareResources::load(stride != 0, root)?;
    Ok(ModernAtlasCompareRun { stride, resources })
}

pub(crate) fn play_gpu_render_compare_session(
    stride: u32,
    modern_render_compare: u32,
    mut modern_index_compare: ModernIndexCompareRun,
    root: &Path,
) -> Result<PlayGpuRenderCompareSession, String> {
    let modern_atlas_compare = modern_atlas_compare_run(modern_render_compare, root)
        .map_err(|e| format!("modern atlas compare resources load failed: {e}"))?;
    modern_index_compare
        .load_resources(root, false)
        .map_err(|e| format!("modern index compare resources load failed: {e}"))?;
    Ok(PlayGpuRenderCompareSession {
        readback: new_gpu_readback_renderer(256, 224),
        render_frame: vec![0u8; 256 * 224 * 4],
        gpu_render_compare: gpu_render_compare_run(stride, true),
        modern_atlas_compare,
        modern_index_compare,
    })
}

impl ModernAtlasCompareRun {
    fn should_compare_frame(&self, frame: u32) -> bool {
        self.stride != 0 && frame % self.stride == 0
    }

    fn render_report_from_game(
        &self,
        game: &mut ZeldaState,
        readback: &mut GpuReadbackRenderer,
        frame: u32,
    ) -> Option<renderer::ModernAtlasCompareFrameReport> {
        let capture = capture_gpu_frame_from_game(game);
        let classic_rgba = readback.render_live_gpu_capture_rgba(&capture);
        self.render_report_from_capture(&capture, classic_rgba.as_slice(), frame)
    }

    fn render_report_from_capture(
        &self,
        capture: &LiveGpuFrameCapture,
        classic_rgba: &[u8],
        frame: u32,
    ) -> Option<renderer::ModernAtlasCompareFrameReport> {
        let gpu_frame = capture.gpu_frame();
        self.resources
            .compare_frame_rgba(frame, &gpu_frame, classic_rgba)
    }
}

impl GpuRenderCompareRun {
    pub(crate) fn set_stride(&mut self, stride: u32) -> bool {
        if stride == 0 {
            return false;
        }
        self.stride = stride;
        true
    }

    pub(crate) fn set_quiet(&mut self) {
        self.quiet = true;
    }

    pub(crate) fn enabled(&self) -> bool {
        self.stride != 0
    }

    pub(crate) fn should_compare_frame(&self, frame: u32) -> bool {
        self.stride != 0 && frame % self.stride == 0
    }

    pub(crate) fn compare_current_frame(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut GpuReadbackRenderer,
        frame_bgra: &mut [u8],
        frame: u32,
    ) -> Option<Option<String>> {
        let cpu_hash = compare_gpu_render_current_frame(game, readback, frame_bgra, frame)?;
        self.compared = self.compared.wrapping_add(1);
        self.last_frame = frame;
        self.last_hash = cpu_hash;
        Some((!self.quiet).then(|| {
            format!("gpu-render-compare frame={frame} hash=0x{cpu_hash:08x} mismatched_pixels=0")
        }))
    }

    fn compare_current_frame_with_optional_readback(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut OptionalGpuReadbackRenderer,
        frame_bgra: &mut [u8],
        frame: u32,
    ) -> Option<Option<String>> {
        self.compare_current_frame(game, readback.required(), frame_bgra, frame)
    }

    fn summary_line_if_quiet(&self) -> Option<String> {
        (self.enabled() && self.quiet).then(|| {
            format!(
                "gpu-render-compare completed compared={} last_frame={} last_hash=0x{:08x} mismatched_pixels=0",
                self.compared, self.last_frame, self.last_hash
            )
        })
    }

    pub(crate) fn emit_current_frame_with_optional_readback(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut OptionalGpuReadbackRenderer,
        frame_bgra: &mut [u8],
        frame: u32,
    ) -> bool {
        let Some(line) =
            self.compare_current_frame_with_optional_readback(game, readback, frame_bgra, frame)
        else {
            return false;
        };
        if let Some(line) = line {
            println!("{line}");
        }
        true
    }

    pub(crate) fn emit_summary_line_if_quiet(&self) {
        if let Some(line) = self.summary_line_if_quiet() {
            println!("{line}");
        }
    }

    pub(crate) fn play_summary_line(&self, start_frame: u32) -> String {
        let last_frame = if self.compared == 0 {
            start_frame
        } else {
            self.last_frame
        };
        format!(
            "play-gpu-render-compare completed compared={} start_frame={} last_frame={} last_hash=0x{:08x} mismatched_pixels=0",
            self.compared, start_frame, last_frame, self.last_hash
        )
    }
}

impl ModernIndexCompareRun {
    pub(crate) fn set_stride(&mut self, stride: u32) -> bool {
        self.config.set_stride(stride).is_ok()
    }

    pub(crate) fn set_require_full_gpu_path(&mut self) {
        self.config.set_require_full_gpu_path();
    }

    pub(crate) fn set_require_modern_index_parity(&mut self) {
        self.config.set_require_modern_index_parity();
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        self.config.validate().map_err(|e| e.to_string())
    }

    pub(crate) fn enabled(&self) -> bool {
        self.config.enabled()
    }

    pub(crate) fn should_compare_frame(&self, frame: u32) -> bool {
        self.config.should_compare_frame(frame)
    }

    pub(crate) fn load_resources(
        &mut self,
        root: &Path,
        allow_source_cpu_fallback: bool,
    ) -> Result<(), String> {
        self.resources = Some(
            self.config
                .load_resources_from_env(root, allow_source_cpu_fallback)?,
        );
        self.allow_source_cpu_fallback = allow_source_cpu_fallback;
        Ok(())
    }

    fn render_output_from_capture(
        &mut self,
        capture: &LiveGpuFrameCapture,
        classic_rgba: &[u8],
        frame: u32,
        include_diff_in_frame_line: bool,
    ) -> renderer::ModernIndexCompareOutputLines {
        let gpu_frame = capture.gpu_frame();
        let resources = self
            .resources
            .as_ref()
            .expect("modern index compare resources loaded");
        self.stats.render_compare_frame_output_from_entries(
            renderer::ModernIndexCompareFrameOutputInput {
                frame,
                main_module: capture.main_module,
                player_indoors: capture.player_indoors,
                gpu_frame: &gpu_frame,
                source_entries: capture.source_entries(),
                resources,
                classic_rgba,
                allow_source_cpu_fallback: self.allow_source_cpu_fallback,
                run_config: self.config,
                include_diff_in_frame_line,
            },
        )
    }

    fn render_output_from_game(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut GpuReadbackRenderer,
        frame: u32,
        include_diff_in_frame_line: bool,
    ) -> renderer::ModernIndexCompareOutputLines {
        let capture = capture_gpu_frame_from_game(game);
        let classic_rgba = readback.render_live_gpu_capture_rgba(&capture);
        self.render_output_from_capture(
            &capture,
            classic_rgba.as_slice(),
            frame,
            include_diff_in_frame_line,
        )
    }

    fn render_output_from_game_with_optional_readback(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut OptionalGpuReadbackRenderer,
        frame: u32,
        include_diff_in_frame_line: bool,
    ) -> renderer::ModernIndexCompareOutputLines {
        self.render_output_from_game(game, readback.required(), frame, include_diff_in_frame_line)
    }

    pub(crate) fn summary_line_if_enabled(&self) -> Option<String> {
        self.stats.summary_line_if_enabled(self.enabled())
    }

    pub(crate) fn emit_summary_line_if_enabled(&self) {
        if let Some(line) = self.summary_line_if_enabled() {
            println!("{line}");
        }
    }

    pub(crate) fn emit_compare_from_game_with_optional_readback(
        &mut self,
        game: &mut ZeldaState,
        readback: &mut OptionalGpuReadbackRenderer,
        frame: u32,
        include_diff_in_frame_line: bool,
    ) -> bool {
        let output_lines = self.render_output_from_game_with_optional_readback(
            game,
            readback,
            frame,
            include_diff_in_frame_line,
        );
        emit_modern_index_compare_output_lines(&output_lines);
        !output_lines.has_failure
    }
}

impl PlayGpuRenderCompareSession {
    pub(crate) fn compare_frame(&mut self, game: &mut ZeldaState, completed_frame: u32) -> bool {
        let should_compare_stride = self
            .gpu_render_compare
            .should_compare_frame(completed_frame);
        let should_compare_modern = self
            .modern_atlas_compare
            .should_compare_frame(completed_frame);
        let should_compare_modern_index = self
            .modern_index_compare
            .should_compare_frame(completed_frame);
        if !should_compare_stride && !should_compare_modern && !should_compare_modern_index {
            return true;
        }
        if should_compare_stride {
            let Some(line) = self.gpu_render_compare.compare_current_frame(
                game,
                &mut self.readback,
                &mut self.render_frame,
                completed_frame,
            ) else {
                return false;
            };
            if let Some(line) = line {
                println!("{line}");
            }
        }
        if should_compare_modern {
            if let Some(report) = self.modern_atlas_compare.render_report_from_game(
                game,
                &mut self.readback,
                completed_frame,
            ) {
                println!("{}", report.line);
            }
        }
        if should_compare_modern_index {
            let output_lines = self.modern_index_compare.render_output_from_game(
                game,
                &mut self.readback,
                completed_frame,
                true,
            );
            emit_modern_index_compare_output_lines(&output_lines);
            if output_lines.has_failure {
                return false;
            }
        }
        true
    }

    pub(crate) fn emit_summaries(&self, start_frame: u32) {
        println!("{}", self.gpu_render_compare.play_summary_line(start_frame));
        self.modern_index_compare.emit_summary_line_if_enabled();
    }
}

fn compare_gpu_render_current_frame(
    game: &mut ZeldaState,
    readback: &mut GpuReadbackRenderer,
    frame: &mut [u8],
    frames: u32,
) -> Option<u32> {
    let width = 256u32;
    let gpu_capture = capture_gpu_frame_from_game(game);
    crate::play_renderer::render_play_frame_bgra(
        game,
        frame,
        width as usize * 4,
        PpuRenderFlags::empty(),
    );
    let gpu_rgba = readback.render_live_gpu_capture_rgba(&gpu_capture);
    let render_comparison =
        renderer::compare_gpu_render_frame_bgra_to_rgba(frames, frame, gpu_rgba.as_slice());
    if let Some(diff) = render_comparison.diff() {
        let gpu_ppu = gpu_capture.ppu();
        let scanlines_raw = gpu_capture.raw_scanlines();
        let hdma_cgram = gpu_capture.cgram();
        if let Some(line) = render_comparison.divergence_line() {
            eprintln!("{line}");
        }
        eprintln!(
            "gpu-render-state frame={frames} forced_blank={} brightness={} screen={:02x}/{:02x} windowed={:02x}/{:02x} windowsel={:08x} math={:02x} add_sub={} subtract={} half={} fixed=({},{},{}) clip={} prevent={} extra=({},{},{}) win0=({},{},{},{}) win128=({},{},{},{}) scanline_tm0={:02x} scanline_tm128={:02x} mode={} cgram0={:04x}",
            gpu_ppu.forced_blank,
            gpu_ppu.brightness,
            gpu_ppu.screen_enabled[0],
            gpu_ppu.screen_enabled[1],
            gpu_ppu.screen_windowed[0],
            gpu_ppu.screen_windowed[1],
            gpu_ppu.windowsel,
            gpu_ppu.math_enabled,
            gpu_ppu.add_subscreen,
            gpu_ppu.subtract_color,
            gpu_ppu.half_color,
            gpu_ppu.fixed_color_r,
            gpu_ppu.fixed_color_g,
            gpu_ppu.fixed_color_b,
            gpu_ppu.clip_mode,
            gpu_ppu.prevent_math_mode,
            gpu_ppu.extra_left_cur,
            gpu_ppu.extra_right_cur,
            gpu_ppu.extra_bottom_cur,
            scanlines_raw[0].0,
            scanlines_raw[0].1,
            scanlines_raw[0].2,
            scanlines_raw[0].3,
            scanlines_raw[128].0,
            scanlines_raw[128].1,
            scanlines_raw[128].2,
            scanlines_raw[128].3,
            scanlines_raw[0].4,
            scanlines_raw[128].4,
            gpu_ppu.mode,
            hdma_cgram[0]
        );
        eprintln!(
            "gpu-render-captured-compose frame={frames} x{} {}",
            diff.first_x,
            game.ppu.debug_pixel_compose_summary(diff.first_x)
        );
        eprintln!(
            "gpu-render-cgram-match frame={frames} cpu={} gpu={}",
            cgram_match(hdma_cgram, diff.cpu_rgb),
            cgram_match(hdma_cgram, diff.gpu_rgb)
        );
        let mut cpu_probe_ppu = gpu_ppu.clone();
        let probe_sl = &scanlines_raw[diff.first_y];
        cpu_probe_ppu.window1_left = probe_sl.0;
        cpu_probe_ppu.window1_right = probe_sl.1;
        cpu_probe_ppu.window2_left = probe_sl.2;
        cpu_probe_ppu.window2_right = probe_sl.3;
        cpu_probe_ppu.screen_enabled[0] = probe_sl.4;
        for layer in 0..4 {
            cpu_probe_ppu.bg_layer[layer].h_scroll = probe_sl.5[layer];
            cpu_probe_ppu.bg_layer[layer].v_scroll = probe_sl.6[layer];
        }
        eprintln!(
            "gpu-render-old-probe frame={frames} {}",
            cpu_probe_ppu.debug_pixel_old_summary(
                diff.first_x as i32,
                diff.first_y as i32 + 1,
                false
            )
        );
        return None;
    }

    Some(render_comparison.cpu_hash())
}

impl GpuReadbackRenderer {
    fn render_gpu_capture_rgba(&mut self, capture: &LiveGpuFrameCapture) -> GpuRgbaReadbackFrame {
        GpuRgbaReadbackFrame {
            rgba: self.offscreen.render_gpu_frame(&capture.gpu_frame()),
        }
    }

    fn render_bgra_frame_to_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.offscreen.upload_bgra_frame(frame);
        GpuRgbaReadbackFrame {
            rgba: self.offscreen.render_to_rgba(),
        }
    }

    pub(crate) fn render_live_gpu_capture_rgba(
        &mut self,
        capture: &LiveGpuFrameCapture,
    ) -> GpuRgbaReadbackFrame {
        self.render_gpu_capture_rgba(capture)
    }

    pub(crate) fn render_cpu_bgra_frame_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.render_bgra_frame_to_rgba(frame)
    }
}

impl OptionalGpuReadbackRenderer {
    pub(crate) fn required(&mut self) -> &mut GpuReadbackRenderer {
        self.renderer
            .as_mut()
            .expect("GPU readback renderer allocated")
    }

    pub(crate) fn render_live_gpu_capture_rgba(
        &mut self,
        capture: &LiveGpuFrameCapture,
    ) -> GpuRgbaReadbackFrame {
        self.required().render_live_gpu_capture_rgba(capture)
    }

    pub(crate) fn render_cpu_bgra_frame_rgba(&mut self, frame: &[u8]) -> GpuRgbaReadbackFrame {
        self.required().render_cpu_bgra_frame_rgba(frame)
    }

    pub(crate) fn capture_replay_render_hash_frame(
        &self,
        game: &mut ZeldaState,
    ) -> ReplayRenderHashCapture {
        ReplayRenderHashCapture {
            capture: capture_gpu_frame_from_game(game),
        }
    }

    pub(crate) fn render_replay_hash_cpu_frame_rgba(
        &mut self,
        game: &mut ZeldaState,
        frame: &mut [u8],
    ) -> GpuRgbaReadbackFrame {
        let width = 256usize;
        crate::play_renderer::render_play_frame_bgra(
            game,
            frame,
            width * 4,
            PpuRenderFlags::empty(),
        );
        self.render_cpu_bgra_frame_rgba(frame)
    }

    pub(crate) fn render_replay_dump_frame_rgba(
        &mut self,
        game: &ZeldaState,
    ) -> GpuRgbaReadbackFrame {
        let width = 256usize;
        let height = 224usize;
        let mut frame = vec![0u8; width * height * 4];
        let mut render_game = game.clone();
        crate::play_renderer::render_play_frame_bgra(
            &mut render_game,
            &mut frame,
            width * 4,
            PpuRenderFlags::empty(),
        );
        self.render_cpu_bgra_frame_rgba(&frame)
    }
}

impl ReplayRenderHashCapture {
    pub(crate) fn render_gpu_rgba(
        &self,
        readback: &mut OptionalGpuReadbackRenderer,
    ) -> ReplayRenderHashGpuReadback {
        ReplayRenderHashGpuReadback {
            frame: readback.render_live_gpu_capture_rgba(&self.capture),
        }
    }

    fn cgram_color(&self, index: usize) -> u16 {
        self.capture.cgram().get(index).copied().unwrap_or(0)
    }

    pub(crate) fn cgram_color_hex(&self, index: usize) -> String {
        format!("{:#06x}", self.cgram_color(index))
    }

    pub(crate) fn debug_frame_800_scanline_screen_enabled_main_line(&self) -> String {
        let values = self.capture.raw_scanlines()[60..70]
            .iter()
            .map(|e| e.4)
            .collect::<Vec<_>>();
        format!("[gpu-dbg] f800 scanlines[60..70] screen_enabled_main: {values:?}")
    }

    pub(crate) fn debug_cgram_render_diff_lines(
        &self,
        frame: u32,
        post_cgram: &[u16],
    ) -> Vec<String> {
        let diffs = self
            .capture
            .cgram()
            .iter()
            .enumerate()
            .zip(post_cgram.iter())
            .filter(|((_, &h), &p)| h != p)
            .map(|((i, &h), &p)| (i, h, p))
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(1 + diffs.len().min(20));
        lines.push(format!(
            "[gpu-dbg] frame={frame} CGRAM changes during render: {} entries differ",
            diffs.len()
        ));
        lines.extend(diffs.iter().take(20).map(|(i, before, after)| {
            format!("[gpu-dbg]   cgram[{i}]: {before:#06x} → {after:#06x}")
        }));
        lines
    }

    pub(crate) fn debug_cgram_value_lines(
        &self,
        frame: u32,
        label: &str,
        value: u16,
    ) -> Vec<String> {
        self.capture
            .cgram()
            .iter()
            .enumerate()
            .filter_map(|(index, &cgram_value)| {
                (cgram_value == value)
                    .then(|| format!("[gpu-dbg] frame={frame} {label}[{index}]={value:#06x}"))
            })
            .collect()
    }

    pub(crate) fn debug_math_state_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] math_enabled={:#04x} subtract={} half={} fixed_rgb=({},{},{}) add_sub={} clip_mode={} prevent_math={} windowsel_cm={:#04x} brightness={}",
            gpu_frame.math_enabled,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            gpu_frame.add_subscreen,
            gpu_frame.clip_mode,
            gpu_frame.prevent_math_mode,
            gpu_frame.windowsel_cm,
            gpu_frame.brightness
        )
    }

    pub(crate) fn debug_frame_332_math_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] frame=332 math_enabled={:#04x} subtract={} half={} fixed=({},{},{}) clip_mode={} prevent_math={} windowsel_cm={:#04x} add_sub={}",
            gpu_frame.math_enabled,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            gpu_frame.clip_mode,
            gpu_frame.prevent_math_mode,
            gpu_frame.windowsel_cm,
            gpu_frame.add_subscreen
        )
    }

    pub(crate) fn debug_frame_332_scanline_window_line(&self) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] frame=332 scanline[0]: w1l={} w1r={}",
            gpu_frame.scanlines[0].window1_left, gpu_frame.scanlines[0].window1_right
        )
    }

    pub(crate) fn debug_effect_math_line(
        &self,
        frame: u32,
        bg1_hscroll: u16,
        irq_flag: u8,
    ) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] f{frame} math={:#04x} add_sub={} subtract={} half={} fixed_r={} fixed_g={} fixed_b={} bg1_hscroll={} irq_flag={}",
            gpu_frame.math_enabled,
            gpu_frame.add_subscreen,
            gpu_frame.subtract_color,
            gpu_frame.half_color,
            gpu_frame.fixed_color_r,
            gpu_frame.fixed_color_g,
            gpu_frame.fixed_color_b,
            bg1_hscroll,
            irq_flag
        )
    }

    pub(crate) fn debug_scanline_tm_probe_line(&self, frame: u32, cy: i32) -> String {
        let gpu_frame = self.capture.gpu_frame();
        format!(
            "[gpu-dbg] f{frame} scanline_tm row{}={:#04x} row{}={:#04x}",
            cy,
            gpu_frame.scanlines[cy as usize].screen_enabled_main,
            cy + 1,
            gpu_frame.scanlines[(cy + 1) as usize].screen_enabled_main
        )
    }
}

impl GpuRgbaReadbackFrame {
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.rgba
    }

    fn render_hash_line(&self, frame: u32) -> String {
        renderer::gpu_render_hash_frame_rgba(frame, &self.rgba).line
    }

    fn hash_pair_with_cpu_bgra(&self, cpu_bgra: &[u8]) -> renderer::RenderHashPair {
        renderer::render_hash_pair_bgra_rgba(cpu_bgra, &self.rgba)
    }
}

pub(crate) fn replay_cpu_bgra_hash_line(frame: u32, cpu_bgra: &[u8]) -> String {
    renderer::render_hash_frame_bgra(frame, cpu_bgra).line
}

pub(crate) fn replay_projection_bgra(game: &mut ZeldaState, frame: &mut [u8]) {
    crate::play_renderer::render_standard_play_frame_bgra(game, frame);
}

pub(crate) fn replay_fingerprint_leaf_bgra(game: &mut ZeldaState, frame: &mut [u8]) -> u32 {
    replay_projection_bgra(game, frame);
    renderer::render_fingerprint_leaf_bgra(frame)
}

impl ReplayRenderHashGpuReadback {
    pub(crate) fn gpu_render_hash_log_line(&self, frame: u32) -> String {
        self.frame.render_hash_line(frame)
    }

    pub(crate) fn debug_hash_line_with_cpu_bgra(&self, frame: u32, cpu_bgra: &[u8]) -> String {
        let hashes = self.frame.hash_pair_with_cpu_bgra(cpu_bgra);
        format!(
            "[gpu-dbg] frame={frame} cpu_hash={:#010x} gpu_hash={:#010x}",
            hashes.cpu_hash, hashes.gpu_hash
        )
    }
}

impl std::ops::Deref for GpuRgbaReadbackFrame {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.rgba
    }
}

impl std::ops::Deref for ReplayRenderHashGpuReadback {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.frame.as_slice()
    }
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

fn emit_modern_index_compare_output_lines(output: &renderer::ModernIndexCompareOutputLines) {
    for output_line in &output.lines {
        match output_line.stream {
            renderer::ModernIndexCompareOutputStream::Stdout => println!("{}", output_line.line),
            renderer::ModernIndexCompareOutputStream::Stderr => eprintln!("{}", output_line.line),
        }
    }
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

fn cgram_match(cgram: &[u16], rgb: (u8, u8, u8)) -> String {
    cgram
        .iter()
        .enumerate()
        .find_map(|(i, &entry)| {
            let r5 = (entry & 0x1f) as u8;
            let g5 = ((entry >> 5) & 0x1f) as u8;
            let b5 = ((entry >> 10) & 0x1f) as u8;
            let r = (r5 << 3) | (r5 >> 2);
            let g = (g5 << 3) | (g5 >> 2);
            let b = (b5 << 3) | (b5 >> 2);
            if (r, g, b) == rgb {
                Some(format!("{i:02x}:{entry:04x}"))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "none".to_string())
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

#[cfg(test)]
mod tests {
    use super::gpu_render_compare_run;

    #[test]
    fn quiet_summary_requires_enabled_compare() {
        let compare = gpu_render_compare_run(0, true);

        assert_eq!(compare.summary_line_if_quiet(), None);
    }

    #[test]
    fn play_summary_uses_start_frame_until_first_compare() {
        let compare = gpu_render_compare_run(10, true);

        assert_eq!(
            compare.play_summary_line(1234),
            "play-gpu-render-compare completed compared=0 start_frame=1234 last_frame=1234 last_hash=0x00000000 mismatched_pixels=0"
        );
    }
}
