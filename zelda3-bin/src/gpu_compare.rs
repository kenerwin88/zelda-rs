use std::path::Path;

use crate::gpu_capture::{capture_gpu_frame_from_game, LiveGpuFrameCapture};
use crate::gpu_readback::{GpuReadbackRenderer, OptionalGpuReadbackRenderer};
use snes::ppu::PpuRenderFlags;
use zelda3::ZeldaState;

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

pub(crate) fn modern_compare_mode_defaults_from_env() -> ModernCompareModeDefaults {
    let renderer_mode =
        renderer::RendererMode::parse(std::env::var("ZELDA3_RENDERER").ok().as_deref());
    let enable_modern_render_compare = renderer_mode == renderer::RendererMode::ModernCompare
        || renderer_mode == renderer::RendererMode::Modern;
    let note = if renderer_mode == renderer::RendererMode::Modern {
        Some(
            "note: ZELDA3_RENDERER=modern is experimental; modern path cannot render most content \u{2014} running as modern-compare",
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
        readback: GpuReadbackRenderer::new(256, 224),
        render_frame: vec![0u8; 256 * 224 * 4],
        gpu_render_compare: gpu_render_compare_run(stride, true),
        modern_atlas_compare,
        modern_index_compare,
    })
}

pub(crate) fn replay_optional_gpu_readback_renderer(
    render_hash_log: u32,
    gpu_render_compare: &GpuRenderCompareRun,
    render_hash_dump_enabled: bool,
    dump_frame_enabled: bool,
    modern_index_compare: &ModernIndexCompareRun,
) -> OptionalGpuReadbackRenderer {
    OptionalGpuReadbackRenderer::new(
        render_hash_log != 0
            || gpu_render_compare.enabled()
            || render_hash_dump_enabled
            || dump_frame_enabled
            || modern_index_compare.enabled(),
        256,
        224,
    )
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

    pub(crate) fn summary_line_if_quiet(&self) -> Option<String> {
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
                main_module: capture.main_module(),
                player_indoors: capture.player_indoors(),
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
    crate::classic_frame_renderer::render_play_frame_bgra(
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
                false,
            )
        );
        return None;
    }

    Some(render_comparison.cpu_hash())
}

pub(crate) fn replay_cpu_bgra_hash_line(frame: u32, cpu_bgra: &[u8]) -> String {
    renderer::render_hash_frame_bgra(frame, cpu_bgra).line
}

fn emit_modern_index_compare_output_lines(output: &renderer::ModernIndexCompareOutputLines) {
    for output_line in &output.lines {
        match output_line.stream {
            renderer::ModernIndexCompareOutputStream::Stdout => println!("{}", output_line.line),
            renderer::ModernIndexCompareOutputStream::Stderr => eprintln!("{}", output_line.line),
        }
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
