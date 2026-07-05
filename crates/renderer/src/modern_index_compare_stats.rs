use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::modern_gpu::{modern_gpu_path_fallback_reason, ModernGpuPathFallback};
use crate::modern_software::VariantAtlasRenderStats;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernIndexCompareRunConfigError {
    ZeroStride,
    RequireFullGpuPathWithoutCompare,
    RequireModernIndexParityWithoutCompare,
}

impl std::fmt::Display for ModernIndexCompareRunConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroStride => write!(f, "--modern-index-compare must be greater than zero"),
            Self::RequireFullGpuPathWithoutCompare => {
                write!(f, "--require-full-gpu-path requires --modern-index-compare")
            }
            Self::RequireModernIndexParityWithoutCompare => {
                write!(
                    f,
                    "--require-modern-index-parity requires --modern-index-compare"
                )
            }
        }
    }
}

impl std::error::Error for ModernIndexCompareRunConfigError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModernIndexCompareRunConfig {
    stride: u32,
    require_full_gpu_path: bool,
    require_modern_index_parity: bool,
}

impl ModernIndexCompareRunConfig {
    pub fn set_stride(&mut self, stride: u32) -> Result<(), ModernIndexCompareRunConfigError> {
        if stride == 0 {
            return Err(ModernIndexCompareRunConfigError::ZeroStride);
        }
        self.stride = stride;
        Ok(())
    }

    pub fn set_require_full_gpu_path(&mut self) {
        self.require_full_gpu_path = true;
    }

    pub fn set_require_modern_index_parity(&mut self) {
        self.require_modern_index_parity = true;
    }

    pub fn validate(self) -> Result<(), ModernIndexCompareRunConfigError> {
        if self.stride == 0 && self.require_full_gpu_path {
            return Err(ModernIndexCompareRunConfigError::RequireFullGpuPathWithoutCompare);
        }
        if self.stride == 0 && self.require_modern_index_parity {
            return Err(ModernIndexCompareRunConfigError::RequireModernIndexParityWithoutCompare);
        }
        Ok(())
    }

    pub fn enabled(self) -> bool {
        self.stride != 0
    }

    pub fn should_compare_frame(self, frame: u32) -> bool {
        self.stride != 0 && frame % self.stride == 0
    }

    pub fn load_resources_from_env(
        self,
        root: &std::path::Path,
        allow_source_cpu_fallback: bool,
    ) -> Result<crate::ModernIndexCompareResources, String> {
        crate::ModernIndexCompareResources::load_from_env(
            self.enabled(),
            root,
            allow_source_cpu_fallback,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ModernIndexComparePixelDiff {
    pub first_x: usize,
    pub first_y: usize,
    pub classic_rgb: (u8, u8, u8),
    pub modern_rgb: (u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ModernIndexCompareFrameDiff {
    pub mismatch: u32,
    pub diff: Option<ModernIndexComparePixelDiff>,
}

#[derive(Clone, Copy)]
struct ModernIndexCompareFrameLine<'a> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub ppu_mode: u8,
    pub mismatch: u32,
    pub via: &'a str,
    pub variant_stats: Option<&'a VariantAtlasRenderStats>,
    pub diff: Option<ModernIndexComparePixelDiff>,
}

#[derive(Clone, Copy)]
struct ModernIndexCompareFrameRecord<'a> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub ppu_mode: u8,
    pub via: &'a str,
    pub variant_stats: Option<&'a VariantAtlasRenderStats>,
    pub comparison: ModernIndexCompareFrameDiff,
    pub run_config: ModernIndexCompareRunConfig,
    pub include_diff_in_frame_line: bool,
}

pub struct ModernIndexCompareFrameReport {
    mismatch: u32,
    parity_failure_line: Option<String>,
    full_gpu_failure_line: Option<String>,
    frame_line: Option<String>,
    progress_line: Option<String>,
}

impl ModernIndexCompareFrameReport {
    pub fn mismatch(&self) -> u32 {
        self.mismatch
    }

    pub fn failure_line(&self) -> Option<&str> {
        self.parity_failure_line
            .as_deref()
            .or(self.full_gpu_failure_line.as_deref())
    }

    pub fn frame_line(&self) -> Option<&str> {
        self.frame_line.as_deref()
    }

    pub fn progress_line(&self) -> Option<&str> {
        self.progress_line.as_deref()
    }
}

pub struct ModernIndexCompareFrameRenderInput<
    'a,
    'frame,
    S: crate::modern_extract::SourceTableView + ?Sized,
> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub gpu_frame: &'a crate::gpu_frame::GpuFrame<'frame>,
    pub src_table: Option<&'a S>,
    pub resources: &'a crate::ModernIndexCompareResources,
    pub scene: crate::ModernAssetFrameScene,
    pub classic_rgba: &'a [u8],
    pub allow_source_cpu_fallback: bool,
    pub run_config: ModernIndexCompareRunConfig,
    pub include_diff_in_frame_line: bool,
}

struct ModernIndexCompareFrameRenderedRecord<'a> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub ppu_mode: u8,
    pub classic_rgba: &'a [u8],
    pub modern_render: crate::modern_gpu::ModernIndexCompareRender,
    pub trace_pixel: Option<ModernIndexCompareTracePixel>,
    pub run_config: ModernIndexCompareRunConfig,
    pub include_diff_in_frame_line: bool,
}

struct ModernIndexCompareFrameRenderedReport {
    pub report: ModernIndexCompareFrameReport,
    pub modern_rgba: Vec<u8>,
    pub trace_lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernIndexCompareOutputStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernIndexCompareOutputLine {
    pub stream: ModernIndexCompareOutputStream,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernIndexCompareOutputLines {
    pub lines: Vec<ModernIndexCompareOutputLine>,
    pub has_failure: bool,
}

impl ModernIndexCompareFrameRenderedReport {
    fn output_lines(&self) -> ModernIndexCompareOutputLines {
        let mut lines = Vec::new();
        lines.extend(
            self.trace_lines
                .iter()
                .cloned()
                .map(ModernIndexCompareOutputLine::stderr),
        );

        if let Some(line) = self.report.failure_line() {
            lines.push(ModernIndexCompareOutputLine::stderr(line));
            return ModernIndexCompareOutputLines {
                lines,
                has_failure: true,
            };
        }

        if let Some(line) = self.report.frame_line() {
            lines.push(ModernIndexCompareOutputLine::stdout(line));
        }
        if let Some(line) = self.report.progress_line() {
            lines.push(ModernIndexCompareOutputLine::stderr(line));
        }

        ModernIndexCompareOutputLines {
            lines,
            has_failure: false,
        }
    }
}

impl ModernIndexCompareOutputLine {
    fn stdout(line: impl Into<String>) -> Self {
        Self {
            stream: ModernIndexCompareOutputStream::Stdout,
            line: line.into(),
        }
    }

    fn stderr(line: impl Into<String>) -> Self {
        Self {
            stream: ModernIndexCompareOutputStream::Stderr,
            line: line.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexCompareTracePixel {
    pub frame: u32,
    pub x: i16,
    pub y: i16,
}

struct ModernIndexCompareDumpPaths {
    pub classic_path: PathBuf,
    pub modern_path: PathBuf,
    pub classic_dumped_line: String,
    pub modern_dumped_line: String,
}

fn compare_modern_index_rgba(
    classic_rgba: &[u8],
    modern_rgba: &[u8],
) -> ModernIndexCompareFrameDiff {
    let generic_diff = crate::frame_compare::compare_rgba_to_rgba(classic_rgba, modern_rgba);
    let mismatch = generic_diff
        .as_ref()
        .map(|diff| diff.mismatched_pixels as u32)
        .unwrap_or(0);
    let diff = generic_diff.map(|diff| ModernIndexComparePixelDiff {
        first_x: diff.first_x,
        first_y: diff.first_y,
        classic_rgb: diff.cpu_rgb,
        modern_rgb: diff.gpu_rgb,
    });
    ModernIndexCompareFrameDiff { mismatch, diff }
}

#[derive(Default)]
pub struct ModernIndexCompareStats {
    summary_enabled: bool,
    progress_interval: u64,
    compare_count: u64,
    bad_count: u64,
    bad_pixels: u64,
    gpu_count: u64,
    mode7_gpu_count: u64,
    cpu_count: u64,
    variant_totals: VariantAtlasRenderTotals,
    dump_frame: Option<u32>,
    trace_pixel: Option<ModernIndexCompareTracePixel>,
}

impl ModernIndexCompareStats {
    pub fn from_env() -> Self {
        Self {
            summary_enabled: env::var("ZELDA3_MODERN_INDEX_COMPARE_SUMMARY").is_ok(),
            progress_interval: env::var("ZELDA3_MODERN_INDEX_COMPARE_PROGRESS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
            dump_frame: env::var("ZELDA3_MODERN_INDEX_DUMP_FRAME")
                .ok()
                .and_then(|s| s.parse::<u32>().ok()),
            trace_pixel: env::var("ZELDA3_VARIANT_TRACE_PIXEL")
                .ok()
                .and_then(|value| parse_trace_pixel(&value)),
            ..Self::default()
        }
    }

    fn should_print_frame(&self, mismatch: u32) -> bool {
        !self.summary_enabled || mismatch != 0
    }

    fn record(
        &mut self,
        via: &str,
        mismatch: u32,
        variant_stats: Option<&VariantAtlasRenderStats>,
    ) {
        self.compare_count += 1;
        match via {
            "gpu" | "variant-gpu" => self.gpu_count += 1,
            "mode7-gpu" => self.mode7_gpu_count += 1,
            "mode7-cpu" | "sources" | "vram" => self.cpu_count += 1,
            _ => {}
        }
        if mismatch != 0 {
            self.bad_count += 1;
            self.bad_pixels += u64::from(mismatch);
        }
        if let Some(stats) = variant_stats {
            self.variant_totals.add(stats);
        }
    }

    fn full_gpu_fallback(
        &self,
        via: &str,
        variant_stats: Option<&VariantAtlasRenderStats>,
    ) -> Option<ModernGpuPathFallback> {
        modern_gpu_path_fallback_reason(via, variant_stats)
    }

    fn progress_line(&self, frame: u32) -> Option<String> {
        (self.summary_enabled
            && self.progress_interval != 0
            && self.compare_count % self.progress_interval == 0)
            .then(|| {
                format!(
                    "modern_index_compare_progress compare_count={} frame={} bad_count={}",
                    self.compare_count, frame, self.bad_count
                )
            })
    }

    fn dump_paths_for_frame(&self, frame: u32) -> Option<ModernIndexCompareDumpPaths> {
        (self.dump_frame == Some(frame)).then(|| {
            let classic_path = PathBuf::from(format!("/tmp/classic_{frame}.png"));
            let modern_path = PathBuf::from(format!("/tmp/modern_index_{frame}.png"));
            ModernIndexCompareDumpPaths {
                classic_dumped_line: format!("dumped classic frame to {}", classic_path.display()),
                modern_dumped_line: format!(
                    "dumped modern_index frame to {}",
                    modern_path.display()
                ),
                classic_path,
                modern_path,
            }
        })
    }

    fn write_dump_for_frame(
        &self,
        frame: u32,
        classic_rgba: &[u8],
        modern_rgba: &[u8],
    ) -> ModernIndexCompareOutputLines {
        let mut lines = Vec::new();
        if let Some(paths) = self.dump_paths_for_frame(frame) {
            write_dump_png(
                &mut lines,
                &paths.classic_path,
                paths.classic_dumped_line,
                classic_rgba,
            );
            write_dump_png(
                &mut lines,
                &paths.modern_path,
                paths.modern_dumped_line,
                modern_rgba,
            );
        }
        ModernIndexCompareOutputLines {
            lines,
            has_failure: false,
        }
    }

    fn trace_pixel_for_frame(&self, frame: u32) -> Option<ModernIndexCompareTracePixel> {
        self.trace_pixel.filter(|trace| trace.frame == frame)
    }

    fn record_frame(
        &mut self,
        record: ModernIndexCompareFrameRecord<'_>,
    ) -> ModernIndexCompareFrameReport {
        let mismatch = record.comparison.mismatch;
        self.record(record.via, mismatch, record.variant_stats);

        let line = ModernIndexCompareFrameLine {
            frame: record.frame,
            mode_label: record.mode_label,
            ppu_mode: record.ppu_mode,
            mismatch,
            via: record.via,
            variant_stats: record.variant_stats,
            diff: record.comparison.diff,
        };
        let parity_failure_line = (record.run_config.require_modern_index_parity && mismatch != 0)
            .then(|| self.mismatch_line(line));
        let full_gpu_failure_line = if record.run_config.require_full_gpu_path {
            self.full_gpu_fallback(record.via, record.variant_stats)
                .map(|fallback| {
                    format!(
                        "gpu_path_unsupported frame={} mode={} ppumode={} via={} reason={} count={} mismatch_px={}",
                        record.frame,
                        record.mode_label,
                        record.ppu_mode,
                        record.via,
                        fallback.reason,
                        fallback.count,
                        mismatch
                    )
                })
        } else {
            None
        };
        let frame_line = self.should_print_frame(mismatch).then(|| {
            self.frame_line(ModernIndexCompareFrameLine {
                diff: record
                    .include_diff_in_frame_line
                    .then_some(record.comparison.diff)
                    .flatten(),
                ..line
            })
        });
        let progress_line = self.progress_line(record.frame);

        ModernIndexCompareFrameReport {
            mismatch,
            parity_failure_line,
            full_gpu_failure_line,
            frame_line,
            progress_line,
        }
    }

    fn record_rendered_frame(
        &mut self,
        record: ModernIndexCompareFrameRenderedRecord<'_>,
    ) -> ModernIndexCompareFrameRenderedReport {
        let via = record.modern_render.via;
        let variant_stats = record.modern_render.variant_stats;
        let comparison = compare_modern_index_rgba(record.classic_rgba, &record.modern_render.rgba);
        let report = self.record_frame(ModernIndexCompareFrameRecord {
            frame: record.frame,
            mode_label: record.mode_label,
            ppu_mode: record.ppu_mode,
            via,
            variant_stats: variant_stats.as_ref(),
            comparison,
            run_config: record.run_config,
            include_diff_in_frame_line: record.include_diff_in_frame_line,
        });

        ModernIndexCompareFrameRenderedReport {
            report,
            modern_rgba: record.modern_render.rgba,
            trace_lines: trace_lines(record.trace_pixel, &record.modern_render.variant_traces),
        }
    }

    fn render_compare_frame<S: crate::modern_extract::SourceTableView + ?Sized>(
        &mut self,
        input: ModernIndexCompareFrameRenderInput<'_, '_, S>,
    ) -> ModernIndexCompareFrameRenderedReport {
        let trace_pixel = self.trace_pixel_for_frame(input.frame);
        let modern_render = crate::modern_gpu::render_modern_index_compare_frame(
            input.gpu_frame,
            input.src_table,
            input.resources.source_atlas(),
            input.resources.gpu_headless(),
            input.resources.variant_headless(),
            input.scene,
            trace_pixel.map(|trace| (trace.x, trace.y)),
            input.allow_source_cpu_fallback,
        );
        self.record_rendered_frame(ModernIndexCompareFrameRenderedRecord {
            frame: input.frame,
            mode_label: input.mode_label,
            ppu_mode: input.gpu_frame.mode,
            classic_rgba: input.classic_rgba,
            modern_render,
            trace_pixel,
            run_config: input.run_config,
            include_diff_in_frame_line: input.include_diff_in_frame_line,
        })
    }

    pub fn render_compare_frame_output<S: crate::modern_extract::SourceTableView + ?Sized>(
        &mut self,
        input: ModernIndexCompareFrameRenderInput<'_, '_, S>,
    ) -> ModernIndexCompareOutputLines {
        let frame = input.frame;
        let classic_rgba = input.classic_rgba;
        let rendered = self.render_compare_frame(input);
        self.output_lines_for_rendered_frame(frame, classic_rgba, rendered)
    }

    fn output_lines_for_rendered_frame(
        &self,
        frame: u32,
        classic_rgba: &[u8],
        rendered: ModernIndexCompareFrameRenderedReport,
    ) -> ModernIndexCompareOutputLines {
        let mut output = rendered.output_lines();
        if !output.has_failure {
            output.lines.extend(
                self.write_dump_for_frame(frame, classic_rgba, &rendered.modern_rgba)
                    .lines,
            );
        }
        output
    }

    fn frame_line(&self, line: ModernIndexCompareFrameLine<'_>) -> String {
        self.format_frame_line("modern_index_compare", line)
    }

    fn mismatch_line(&self, line: ModernIndexCompareFrameLine<'_>) -> String {
        self.format_frame_line("modern_index_mismatch", line)
    }

    fn format_frame_line(&self, prefix: &str, line: ModernIndexCompareFrameLine<'_>) -> String {
        let mut out = format!(
            "{prefix} frame={} mode={} ppumode={} mismatch_px={} via={}",
            line.frame, line.mode_label, line.ppu_mode, line.mismatch, line.via
        );
        if let Some(stats) = line.variant_stats {
            append_variant_stats_fields(&mut out, stats);
        }
        if let Some(diff) = line.diff {
            let _ = write!(
                out,
                " first_mismatch=({}, {}) classic_rgb=({},{},{}) modern_rgb=({},{},{})",
                diff.first_x,
                diff.first_y,
                diff.classic_rgb.0,
                diff.classic_rgb.1,
                diff.classic_rgb.2,
                diff.modern_rgb.0,
                diff.modern_rgb.1,
                diff.modern_rgb.2
            );
        }
        out
    }

    fn summary_line(&self) -> String {
        let mut out = format!(
            "modern_index_compare_summary compare_count={} bad_count={} bad_pixels={} gpu_count={} mode7_gpu_count={} cpu_count={}",
            self.compare_count,
            self.bad_count,
            self.bad_pixels,
            self.gpu_count,
            self.mode7_gpu_count,
            self.cpu_count
        );
        self.variant_totals.append_fields(&mut out);
        out
    }

    pub fn summary_line_if_enabled(&self, compare_enabled: bool) -> Option<String> {
        (compare_enabled && self.summary_enabled).then(|| self.summary_line())
    }
}

fn parse_trace_pixel(value: &str) -> Option<ModernIndexCompareTracePixel> {
    let mut parts = value.split([':', ',']);
    let frame = parts.next()?.parse().ok()?;
    let x = parts.next()?.parse().ok()?;
    let y = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(ModernIndexCompareTracePixel { frame, x, y })
}

fn trace_lines(
    trace_pixel: Option<ModernIndexCompareTracePixel>,
    traces: &[crate::modern_variant_draw::VariantPixelTrace],
) -> Vec<String> {
    let Some(trace_pixel) = trace_pixel else {
        return Vec::new();
    };
    if traces.is_empty() {
        return vec![format!(
            "variant_pixel_trace frame={} pixel=({}, {}) hits=0",
            trace_pixel.frame, trace_pixel.x, trace_pixel.y
        )];
    }

    traces
        .iter()
        .map(|trace| {
            format!(
                "variant_pixel_trace frame={} pixel=({}, {}) {}",
                trace_pixel.frame,
                trace_pixel.x,
                trace_pixel.y,
                trace.describe()
            )
        })
        .collect()
}

fn write_dump_png(
    output: &mut Vec<ModernIndexCompareOutputLine>,
    path: &std::path::Path,
    success_line: String,
    rgba: &[u8],
) {
    match write_rgba_frame_png(path, rgba, 256, 224) {
        Ok(()) => output.push(ModernIndexCompareOutputLine::stdout(success_line)),
        Err(error) => output.push(ModernIndexCompareOutputLine::stderr(format!(
            "failed to write {}: {error}",
            path.display()
        ))),
    }
}

fn write_rgba_frame_png(
    path: &std::path::Path,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png = encoder.write_header().map_err(|error| error.to_string())?;
    png.write_image_data(rgba)
        .map_err(|error| error.to_string())
}

macro_rules! variant_stat_fields {
    ($macro:ident) => {
        $macro! {
            (stable_draws, "variant_draws"),
            (fallback_draws, "fallback_draws"),
            (live_index_draws, "live_index_draws"),
            (live_index_bg_draws, "live_index_bg_draws"),
            (live_index_bg12_draws, "live_index_bg12_draws"),
            (live_index_bg3_draws, "live_index_bg3_draws"),
            (live_index_sprite_draws, "live_index_sprite_draws"),
            (gpu_prefinal_base_frames, "gpu_prefinal_base_frames"),
            (gpu_screen_builder_frames, "gpu_screen_builder_frames"),
            (cpu_prefinal_composite_frames, "cpu_prefinal_composite_frames"),
            (cpu_prefinal_overlay_frames, "cpu_prefinal_overlay_frames"),
            (dynamic_palette_draws, "dynamic_palette_draws"),
            (missing_variant_draws, "missing_variant_draws"),
            (stable_preview_draws, "stable_preview_draws"),
            (stable_effect_draws, "stable_effect_draws"),
            (dynamic_material_draws, "dynamic_material_draws"),
            (effect_material_draws, "effect_material_draws"),
            (dynamic_material_fallback_draws, "dynamic_material_fallback_draws"),
            (dynamic_material_fallback_instance_source_draws, "dynamic_material_fallback_instance_source_draws"),
            (dynamic_material_fallback_brightness_draws, "dynamic_material_fallback_brightness_draws"),
            (dynamic_material_fallback_policy_draws, "dynamic_material_fallback_policy_draws"),
            (dynamic_material_fallback_missing_effect_draws, "dynamic_material_fallback_missing_effect_draws"),
            (dynamic_material_fallback_unsupported_draws, "dynamic_material_fallback_unsupported_draws"),
            (unsupported_material_draws, "unsupported_material_draws"),
            (missing_art_draws, "missing_art_draws"),
            (unkeyed_fallback_draws, "unkeyed_fallback_draws"),
            (unkeyed_bg_fallback_draws, "unkeyed_bg_fallback_draws"),
            (unkeyed_bg12_fallback_draws, "unkeyed_bg12_fallback_draws"),
            (unkeyed_bg3_fallback_draws, "unkeyed_bg3_fallback_draws"),
            (unkeyed_sprite_fallback_draws, "unkeyed_sprite_fallback_draws"),
            (mixed_overlay_bg_effect_draws, "mixed_overlay_bg_effect_draws"),
            (mixed_overlay_bg_effect_candidates, "mixed_overlay_bg_effect_candidates"),
            (mixed_overlay_bg_effect_culled_invisible_main, "mixed_overlay_bg_effect_culled_invisible_main"),
            (mixed_overlay_bg_effect_reject_complex_frame, "mixed_overlay_bg_effect_reject_complex_frame"),
            (mixed_overlay_bg_effect_reject_complex_brightness, "mixed_overlay_bg_effect_reject_complex_brightness"),
            (mixed_overlay_bg_effect_reject_complex_invalid_layer, "mixed_overlay_bg_effect_reject_complex_invalid_layer"),
            (mixed_overlay_bg_effect_reject_complex_mosaic, "mixed_overlay_bg_effect_reject_complex_mosaic"),
            (mixed_overlay_bg_effect_reject_complex_sub_window, "mixed_overlay_bg_effect_reject_complex_sub_window"),
            (mixed_overlay_bg_effect_reject_complex_effect_bounds, "mixed_overlay_bg_effect_reject_complex_effect_bounds"),
            (mixed_overlay_bg_effect_reject_complex_scanline_main, "mixed_overlay_bg_effect_reject_complex_scanline_main"),
            (mixed_overlay_bg_effect_reject_complex_layer_window, "mixed_overlay_bg_effect_reject_complex_layer_window"),
            (mixed_overlay_bg_effect_reject_complex_color_math, "mixed_overlay_bg_effect_reject_complex_color_math"),
            (mixed_overlay_bg_effect_reject_complex_color_math_clip, "mixed_overlay_bg_effect_reject_complex_color_math_clip"),
            (mixed_overlay_bg_effect_reject_complex_color_math_subscreen, "mixed_overlay_bg_effect_reject_complex_color_math_subscreen"),
            (mixed_overlay_bg_effect_reject_complex_color_math_fixed_color, "mixed_overlay_bg_effect_reject_complex_color_math_fixed_color"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex"),
            (mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch, "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch"),
            (mixed_overlay_bg_effect_reject_cgram_mismatch, "mixed_overlay_bg_effect_reject_cgram_mismatch"),
            (mixed_overlay_bg_effect_reject_overlap, "mixed_overlay_bg_effect_reject_overlap"),
        }
    };
}

macro_rules! define_variant_totals {
    ($(($field:ident, $label:literal)),+ $(,)?) => {
        #[derive(Default)]
        struct VariantAtlasRenderTotals {
            $($field: u64,)+
        }

        impl VariantAtlasRenderTotals {
            fn add(&mut self, stats: &VariantAtlasRenderStats) {
                $(self.$field += u64::from(stats.$field);)+
            }

            fn append_fields(&self, out: &mut String) {
                $(let _ = write!(out, concat!(" ", $label, "={}"), self.$field);)+
                let _ = write!(
                    out,
                    " direct_gpu_fallback_frames={}",
                    self.gpu_prefinal_base_frames
                );
            }
        }

        fn append_variant_stats_fields(out: &mut String, stats: &VariantAtlasRenderStats) {
            $(let _ = write!(out, concat!(" ", $label, "={}"), stats.$field);)+
            let _ = write!(
                out,
                " direct_gpu_fallback_frames={}",
                stats.gpu_prefinal_base_frames
            );
        }
    };
}

variant_stat_fields!(define_variant_totals);

#[cfg(test)]
mod tests {
    use super::*;

    fn run_config_with_requirements(
        require_modern_index_parity: bool,
        require_full_gpu_path: bool,
    ) -> ModernIndexCompareRunConfig {
        let mut config = ModernIndexCompareRunConfig::default();
        config.set_stride(1).expect("valid compare stride");
        if require_modern_index_parity {
            config.set_require_modern_index_parity();
        }
        if require_full_gpu_path {
            config.set_require_full_gpu_path();
        }
        config
    }

    #[test]
    fn records_compare_counts_and_variant_totals() {
        let mut stats = ModernIndexCompareStats::default();
        let variant = VariantAtlasRenderStats {
            stable_draws: 2,
            fallback_draws: 3,
            gpu_prefinal_base_frames: 1,
            cpu_prefinal_composite_frames: 1,
            mixed_overlay_bg_effect_reject_overlap: 4,
            ..Default::default()
        };

        stats.record("variant-gpu", 5, Some(&variant));
        stats.record("mode7-gpu", 0, None);
        stats.record("sources", 7, None);

        let summary = stats.summary_line();
        assert!(summary.contains("compare_count=3"));
        assert!(summary.contains("bad_count=2"));
        assert!(summary.contains("bad_pixels=12"));
        assert!(summary.contains("gpu_count=1"));
        assert!(summary.contains("mode7_gpu_count=1"));
        assert!(summary.contains("cpu_count=1"));
        assert!(summary.contains("variant_draws=2"));
        assert!(summary.contains("fallback_draws=3"));
        assert!(summary.contains("gpu_prefinal_base_frames=1"));
        assert!(summary.contains("direct_gpu_fallback_frames=1"));
        assert!(summary.contains("cpu_prefinal_composite_frames=1"));
        assert!(summary.contains("mixed_overlay_bg_effect_reject_overlap=4"));
    }

    #[test]
    fn summary_line_if_enabled_owns_print_gating() {
        let mut stats = ModernIndexCompareStats {
            summary_enabled: true,
            ..Default::default()
        };
        stats.record("gpu", 0, None);

        assert!(stats.summary_line_if_enabled(false).is_none());
        let summary = stats
            .summary_line_if_enabled(true)
            .expect("enabled summary returns a line");
        assert!(summary.starts_with(
            "modern_index_compare_summary compare_count=1 bad_count=0 bad_pixels=0 gpu_count=1 mode7_gpu_count=0 cpu_count=0"
        ));
        assert!(summary.contains("direct_gpu_fallback_frames=0"));

        let disabled = ModernIndexCompareStats::default();
        assert!(disabled.summary_line_if_enabled(true).is_none());
    }

    #[test]
    fn parses_trace_pixel_config_and_filters_by_frame() {
        assert_eq!(
            parse_trace_pixel("175:102:104"),
            Some(ModernIndexCompareTracePixel {
                frame: 175,
                x: 102,
                y: 104
            })
        );
        assert_eq!(
            parse_trace_pixel("175,102,104"),
            Some(ModernIndexCompareTracePixel {
                frame: 175,
                x: 102,
                y: 104
            })
        );
        assert_eq!(parse_trace_pixel("175:102"), None);
        assert_eq!(parse_trace_pixel("175:102:104:1"), None);

        let stats = ModernIndexCompareStats {
            trace_pixel: parse_trace_pixel("175:102:104"),
            ..Default::default()
        };
        assert_eq!(
            stats.trace_pixel_for_frame(175),
            Some(ModernIndexCompareTracePixel {
                frame: 175,
                x: 102,
                y: 104
            })
        );
        assert_eq!(stats.trace_pixel_for_frame(174), None);
    }

    #[test]
    fn run_config_owns_compare_cadence_and_require_validation() {
        let mut config = ModernIndexCompareRunConfig::default();
        assert!(!config.enabled());
        assert!(!config.should_compare_frame(10));
        assert_eq!(
            config.set_stride(0),
            Err(ModernIndexCompareRunConfigError::ZeroStride)
        );

        config.set_stride(5).expect("valid stride");
        assert!(config.enabled());
        assert!(config.should_compare_frame(10));
        assert!(!config.should_compare_frame(11));

        let mut invalid_full_gpu = ModernIndexCompareRunConfig::default();
        invalid_full_gpu.set_require_full_gpu_path();
        assert_eq!(
            invalid_full_gpu.validate(),
            Err(ModernIndexCompareRunConfigError::RequireFullGpuPathWithoutCompare)
        );

        let mut invalid_parity = ModernIndexCompareRunConfig::default();
        invalid_parity.set_require_modern_index_parity();
        assert_eq!(
            invalid_parity.validate(),
            Err(ModernIndexCompareRunConfigError::RequireModernIndexParityWithoutCompare)
        );

        config.set_require_full_gpu_path();
        config.set_require_modern_index_parity();
        assert!(config.validate().is_ok());
        assert!(config.should_compare_frame(10));
    }

    #[test]
    fn formats_frame_lines_with_optional_stats_and_diff() {
        let stats = ModernIndexCompareStats::default();
        let variant = VariantAtlasRenderStats {
            stable_draws: 1,
            fallback_draws: 2,
            ..Default::default()
        };
        let line = stats.frame_line(ModernIndexCompareFrameLine {
            frame: 42,
            mode_label: "ow",
            ppu_mode: 1,
            mismatch: 3,
            via: "variant-gpu",
            variant_stats: Some(&variant),
            diff: Some(ModernIndexComparePixelDiff {
                first_x: 4,
                first_y: 5,
                classic_rgb: (1, 2, 3),
                modern_rgb: (4, 5, 6),
            }),
        });

        assert!(line.starts_with(
            "modern_index_compare frame=42 mode=ow ppumode=1 mismatch_px=3 via=variant-gpu"
        ));
        assert!(line.contains("variant_draws=1"));
        assert!(line.contains("fallback_draws=2"));
        assert!(line.contains("first_mismatch=(4, 5)"));
        assert!(line.contains("classic_rgb=(1,2,3)"));
        assert!(line.contains("modern_rgb=(4,5,6)"));
    }

    #[test]
    fn record_frame_owns_compare_reporting_and_failure_lines() {
        let mut stats = ModernIndexCompareStats::default();
        let comparison = ModernIndexCompareFrameDiff {
            mismatch: 3,
            diff: Some(ModernIndexComparePixelDiff {
                first_x: 4,
                first_y: 5,
                classic_rgb: (1, 2, 3),
                modern_rgb: (4, 5, 6),
            }),
        };

        let report = stats.record_frame(ModernIndexCompareFrameRecord {
            frame: 42,
            mode_label: "ow",
            ppu_mode: 1,
            via: "sources",
            variant_stats: None,
            comparison,
            run_config: run_config_with_requirements(true, true),
            include_diff_in_frame_line: false,
        });

        assert_eq!(report.mismatch(), 3);
        assert_eq!(
            report.failure_line(),
            Some(
                "modern_index_mismatch frame=42 mode=ow ppumode=1 mismatch_px=3 via=sources first_mismatch=(4, 5) classic_rgb=(1,2,3) modern_rgb=(4,5,6)"
            )
        );
        assert_eq!(
            report.frame_line(),
            Some("modern_index_compare frame=42 mode=ow ppumode=1 mismatch_px=3 via=sources")
        );
        assert!(report.progress_line().is_none());
        assert!(stats.summary_line().contains("compare_count=1"));
        assert!(stats.summary_line().contains("bad_pixels=3"));
    }

    #[test]
    fn report_failure_line_uses_full_gpu_failure_when_parity_passes() {
        let mut stats = ModernIndexCompareStats::default();
        let report = stats.record_frame(ModernIndexCompareFrameRecord {
            frame: 42,
            mode_label: "ow",
            ppu_mode: 1,
            via: "sources",
            variant_stats: None,
            comparison: ModernIndexCompareFrameDiff {
                mismatch: 0,
                diff: None,
            },
            run_config: run_config_with_requirements(true, true),
            include_diff_in_frame_line: false,
        });

        assert_eq!(
            report.failure_line(),
            Some(
                "gpu_path_unsupported frame=42 mode=ow ppumode=1 via=sources reason=sources-cpu count=1 mismatch_px=0"
            )
        );
    }

    #[test]
    fn record_frame_can_include_diff_in_compare_line() {
        let mut stats = ModernIndexCompareStats::default();
        let comparison = ModernIndexCompareFrameDiff {
            mismatch: 1,
            diff: Some(ModernIndexComparePixelDiff {
                first_x: 2,
                first_y: 3,
                classic_rgb: (7, 8, 9),
                modern_rgb: (10, 11, 12),
            }),
        };

        let report = stats.record_frame(ModernIndexCompareFrameRecord {
            frame: 99,
            mode_label: "dungeon",
            ppu_mode: 7,
            via: "mode7-gpu",
            variant_stats: None,
            comparison,
            run_config: run_config_with_requirements(false, true),
            include_diff_in_frame_line: true,
        });

        assert!(report.failure_line().is_none());
        assert!(report
            .frame_line()
            .is_some_and(|line| line.contains("first_mismatch=(2, 3)")));
    }

    #[test]
    fn dump_paths_for_frame_owns_env_dump_paths_and_lines() {
        let stats = ModernIndexCompareStats {
            dump_frame: Some(42),
            ..Default::default()
        };

        let paths = stats
            .dump_paths_for_frame(42)
            .expect("configured frame gets dump paths");

        assert_eq!(
            paths.classic_path,
            std::path::PathBuf::from("/tmp/classic_42.png")
        );
        assert_eq!(
            paths.modern_path,
            std::path::PathBuf::from("/tmp/modern_index_42.png")
        );
        assert_eq!(
            paths.classic_dumped_line,
            "dumped classic frame to /tmp/classic_42.png"
        );
        assert_eq!(
            paths.modern_dumped_line,
            "dumped modern_index frame to /tmp/modern_index_42.png"
        );
        assert!(stats.dump_paths_for_frame(43).is_none());

        let classic_path = std::path::PathBuf::from("/tmp/classic_42.png");
        let modern_path = std::path::PathBuf::from("/tmp/modern_index_42.png");
        let _ = std::fs::remove_file(&classic_path);
        let _ = std::fs::remove_file(&modern_path);
        let classic = vec![0u8; 256 * 224 * 4];
        let modern = vec![0xffu8; 256 * 224 * 4];

        let output = stats.write_dump_for_frame(42, &classic, &modern);

        assert_eq!(
            output,
            ModernIndexCompareOutputLines {
                lines: vec![
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line: "dumped classic frame to /tmp/classic_42.png".to_string(),
                    },
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line: "dumped modern_index frame to /tmp/modern_index_42.png".to_string(),
                    },
                ],
                has_failure: false,
            }
        );
        assert!(classic_path.exists());
        assert!(modern_path.exists());
        let _ = std::fs::remove_file(classic_path);
        let _ = std::fs::remove_file(modern_path);

        assert_eq!(
            stats.write_dump_for_frame(43, &classic, &modern),
            ModernIndexCompareOutputLines {
                lines: Vec::new(),
                has_failure: false,
            }
        );
    }

    #[test]
    fn record_rendered_frame_owns_compare_diff_and_report() {
        let mut stats = ModernIndexCompareStats::default();
        let classic = vec![
            1, 2, 3, 0xff, //
            4, 5, 6, 0xff,
        ];
        let modern = vec![
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let rendered = stats.record_rendered_frame(ModernIndexCompareFrameRenderedRecord {
            frame: 77,
            mode_label: "ow",
            ppu_mode: 1,
            classic_rgba: &classic,
            modern_render: crate::modern_gpu::ModernIndexCompareRender {
                rgba: modern.clone(),
                via: "gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            },
            trace_pixel: Some(ModernIndexCompareTracePixel {
                frame: 77,
                x: 1,
                y: 0,
            }),
            run_config: run_config_with_requirements(true, true),
            include_diff_in_frame_line: true,
        });

        assert_eq!(rendered.modern_rgba, modern);
        assert_eq!(
            rendered.trace_lines,
            vec!["variant_pixel_trace frame=77 pixel=(1, 0) hits=0"]
        );
        assert_eq!(rendered.report.mismatch(), 1);
        assert_eq!(
            rendered.report.failure_line(),
            Some(
                "modern_index_mismatch frame=77 mode=ow ppumode=1 mismatch_px=1 via=gpu first_mismatch=(1, 0) classic_rgb=(4,5,6) modern_rgb=(4,7,6)"
            )
        );
        assert_eq!(
            rendered.output_lines(),
            ModernIndexCompareOutputLines {
                lines: vec![
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stderr,
                        line: "variant_pixel_trace frame=77 pixel=(1, 0) hits=0".to_string(),
                    },
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stderr,
                        line: "modern_index_mismatch frame=77 mode=ow ppumode=1 mismatch_px=1 via=gpu first_mismatch=(1, 0) classic_rgb=(4,5,6) modern_rgb=(4,7,6)".to_string(),
                    },
                ],
                has_failure: true,
            }
        );
        assert!(rendered
            .report
            .frame_line()
            .is_some_and(|line| line.contains("first_mismatch=(1, 0)")));
    }

    #[test]
    fn rendered_output_lines_preserve_nonfatal_frame_and_progress_order() {
        let mut stats = ModernIndexCompareStats {
            summary_enabled: true,
            progress_interval: 1,
            ..Default::default()
        };
        let classic = vec![
            1, 2, 3, 0xff, //
            4, 5, 6, 0xff,
        ];
        let modern = vec![
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let rendered = stats.record_rendered_frame(ModernIndexCompareFrameRenderedRecord {
            frame: 78,
            mode_label: "ow",
            ppu_mode: 1,
            classic_rgba: &classic,
            modern_render: crate::modern_gpu::ModernIndexCompareRender {
                rgba: modern,
                via: "gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            },
            trace_pixel: None,
            run_config: run_config_with_requirements(false, false),
            include_diff_in_frame_line: false,
        });

        assert_eq!(
            rendered.output_lines(),
            ModernIndexCompareOutputLines {
                lines: vec![
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line:
                            "modern_index_compare frame=78 mode=ow ppumode=1 mismatch_px=1 via=gpu"
                                .to_string(),
                    },
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stderr,
                        line: "modern_index_compare_progress compare_count=1 frame=78 bad_count=1"
                            .to_string(),
                    },
                ],
                has_failure: false,
            }
        );
    }

    #[test]
    fn output_lines_for_rendered_frame_appends_dumps_after_successful_compare_output() {
        let mut stats = ModernIndexCompareStats {
            dump_frame: Some(80),
            ..Default::default()
        };
        let classic_path = std::path::PathBuf::from("/tmp/classic_80.png");
        let modern_path = std::path::PathBuf::from("/tmp/modern_index_80.png");
        let _ = std::fs::remove_file(&classic_path);
        let _ = std::fs::remove_file(&modern_path);
        let classic = vec![0u8; 256 * 224 * 4];
        let modern = classic.clone();

        let rendered = stats.record_rendered_frame(ModernIndexCompareFrameRenderedRecord {
            frame: 80,
            mode_label: "ow",
            ppu_mode: 1,
            classic_rgba: &classic,
            modern_render: crate::modern_gpu::ModernIndexCompareRender {
                rgba: modern,
                via: "gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            },
            trace_pixel: None,
            run_config: ModernIndexCompareRunConfig::default(),
            include_diff_in_frame_line: false,
        });

        let output = stats.output_lines_for_rendered_frame(80, &classic, rendered);

        assert_eq!(
            output,
            ModernIndexCompareOutputLines {
                lines: vec![
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line:
                            "modern_index_compare frame=80 mode=ow ppumode=1 mismatch_px=0 via=gpu"
                                .to_string(),
                    },
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line: "dumped classic frame to /tmp/classic_80.png".to_string(),
                    },
                    ModernIndexCompareOutputLine {
                        stream: ModernIndexCompareOutputStream::Stdout,
                        line: "dumped modern_index frame to /tmp/modern_index_80.png".to_string(),
                    },
                ],
                has_failure: false,
            }
        );
        assert!(classic_path.exists());
        assert!(modern_path.exists());
        let _ = std::fs::remove_file(classic_path);
        let _ = std::fs::remove_file(modern_path);
    }

    #[test]
    fn output_lines_for_rendered_frame_skips_dumps_after_failure() {
        let mut stats = ModernIndexCompareStats {
            dump_frame: Some(81),
            ..Default::default()
        };
        let classic_path = std::path::PathBuf::from("/tmp/classic_81.png");
        let modern_path = std::path::PathBuf::from("/tmp/modern_index_81.png");
        let _ = std::fs::remove_file(&classic_path);
        let _ = std::fs::remove_file(&modern_path);
        let classic = vec![0u8; 256 * 224 * 4];
        let modern = vec![0xffu8; 256 * 224 * 4];

        let rendered = stats.record_rendered_frame(ModernIndexCompareFrameRenderedRecord {
            frame: 81,
            mode_label: "ow",
            ppu_mode: 1,
            classic_rgba: &classic,
            modern_render: crate::modern_gpu::ModernIndexCompareRender {
                rgba: modern,
                via: "gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            },
            trace_pixel: None,
            run_config: run_config_with_requirements(true, false),
            include_diff_in_frame_line: false,
        });

        let output = stats.output_lines_for_rendered_frame(81, &classic, rendered);

        assert_eq!(output.lines.len(), 1);
        assert!(output.has_failure);
        assert!(output.lines[0].line.starts_with("modern_index_mismatch "));
        assert!(!classic_path.exists());
        assert!(!modern_path.exists());
    }

    #[test]
    fn compare_modern_index_rgba_maps_generic_diff_to_index_diff() {
        let classic = [
            1, 2, 3, 0xff, //
            4, 5, 6, 0xff,
        ];
        let modern = [
            1, 2, 3, 0xff, //
            4, 7, 6, 0xff,
        ];

        let comparison = compare_modern_index_rgba(&classic, &modern);

        assert_eq!(comparison.mismatch, 1);
        assert_eq!(
            comparison.diff,
            Some(ModernIndexComparePixelDiff {
                first_x: 1,
                first_y: 0,
                classic_rgb: (4, 5, 6),
                modern_rgb: (4, 7, 6),
            })
        );
    }

    #[test]
    fn reports_full_gpu_fallback_reason() {
        let stats = ModernIndexCompareStats::default();
        let variant = VariantAtlasRenderStats {
            cpu_prefinal_composite_frames: 1,
            ..Default::default()
        };

        assert_eq!(
            stats.full_gpu_fallback("variant-gpu", Some(&variant)),
            Some(ModernGpuPathFallback {
                reason: "prefinal-composite-cpu",
                count: 1,
            })
        );
    }
}
