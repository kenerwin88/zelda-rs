use std::env;
use std::fmt::Write;

use crate::modern_gpu::{modern_gpu_path_fallback_reason, ModernGpuPathFallback};
use crate::modern_software::VariantAtlasRenderStats;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexComparePixelDiff {
    pub first_x: usize,
    pub first_y: usize,
    pub classic_rgb: (u8, u8, u8),
    pub modern_rgb: (u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernIndexCompareFrameDiff {
    pub mismatch: u32,
    pub diff: Option<ModernIndexComparePixelDiff>,
}

#[derive(Clone, Copy)]
pub struct ModernIndexCompareFrameLine<'a> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub ppu_mode: u8,
    pub mismatch: u32,
    pub via: &'a str,
    pub variant_stats: Option<&'a VariantAtlasRenderStats>,
    pub diff: Option<ModernIndexComparePixelDiff>,
}

#[derive(Clone, Copy)]
pub struct ModernIndexCompareFrameRecord<'a> {
    pub frame: u32,
    pub mode_label: &'a str,
    pub ppu_mode: u8,
    pub via: &'a str,
    pub variant_stats: Option<&'a VariantAtlasRenderStats>,
    pub comparison: ModernIndexCompareFrameDiff,
    pub require_modern_index_parity: bool,
    pub require_full_gpu_path: bool,
    pub include_diff_in_frame_line: bool,
}

pub struct ModernIndexCompareFrameReport {
    pub mismatch: u32,
    pub parity_failure_line: Option<String>,
    pub full_gpu_failure_line: Option<String>,
    pub frame_line: Option<String>,
    pub progress_line: Option<String>,
}

pub fn compare_modern_index_rgba(
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
}

impl ModernIndexCompareStats {
    pub fn from_env() -> Self {
        Self {
            summary_enabled: env::var("ZELDA3_MODERN_INDEX_COMPARE_SUMMARY").is_ok(),
            progress_interval: env::var("ZELDA3_MODERN_INDEX_COMPARE_PROGRESS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
            ..Self::default()
        }
    }

    pub fn summary_enabled(&self) -> bool {
        self.summary_enabled
    }

    pub fn should_print_frame(&self, mismatch: u32) -> bool {
        !self.summary_enabled || mismatch != 0
    }

    pub fn record(
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

    pub fn full_gpu_fallback(
        &self,
        via: &str,
        variant_stats: Option<&VariantAtlasRenderStats>,
    ) -> Option<ModernGpuPathFallback> {
        modern_gpu_path_fallback_reason(via, variant_stats)
    }

    pub fn progress_line(&self, frame: u32) -> Option<String> {
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

    pub fn record_frame(
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
        let parity_failure_line =
            (record.require_modern_index_parity && mismatch != 0).then(|| self.mismatch_line(line));
        let full_gpu_failure_line = if record.require_full_gpu_path {
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

    pub fn frame_line(&self, line: ModernIndexCompareFrameLine<'_>) -> String {
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

    pub fn summary_line(&self) -> String {
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
            require_modern_index_parity: true,
            require_full_gpu_path: true,
            include_diff_in_frame_line: false,
        });

        assert_eq!(report.mismatch, 3);
        assert_eq!(
            report.parity_failure_line.as_deref(),
            Some(
                "modern_index_mismatch frame=42 mode=ow ppumode=1 mismatch_px=3 via=sources first_mismatch=(4, 5) classic_rgb=(1,2,3) modern_rgb=(4,5,6)"
            )
        );
        assert_eq!(
            report.full_gpu_failure_line.as_deref(),
            Some(
                "gpu_path_unsupported frame=42 mode=ow ppumode=1 via=sources reason=sources-cpu count=1 mismatch_px=3"
            )
        );
        assert_eq!(
            report.frame_line.as_deref(),
            Some("modern_index_compare frame=42 mode=ow ppumode=1 mismatch_px=3 via=sources")
        );
        assert!(report.progress_line.is_none());
        assert!(stats.summary_line().contains("compare_count=1"));
        assert!(stats.summary_line().contains("bad_pixels=3"));
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
            require_modern_index_parity: false,
            require_full_gpu_path: true,
            include_diff_in_frame_line: true,
        });

        assert!(report.parity_failure_line.is_none());
        assert!(report.full_gpu_failure_line.is_none());
        assert!(report
            .frame_line
            .as_deref()
            .is_some_and(|line| line.contains("first_mismatch=(2, 3)")));
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
