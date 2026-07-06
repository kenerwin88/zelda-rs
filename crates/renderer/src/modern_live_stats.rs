use std::env;

use crate::modern_gpu::{modern_gpu_path_fallback_reason, ModernGpuPathFallback};
use crate::modern_software::VariantAtlasRenderStats;

#[derive(Default)]
pub struct ModernAssetLiveStats {
    enabled: bool,
    require_full_gpu_path: bool,
    log_every_frames: u64,
    frames: u64,
    mode7_source_gpu_frames: u64,
    stable_draws: u64,
    fallback_draws: u64,
    live_index_draws: u64,
    live_index_bg_draws: u64,
    live_index_bg12_draws: u64,
    live_index_bg3_draws: u64,
    live_index_sprite_draws: u64,
    gpu_prefinal_base_frames: u64,
    gpu_screen_builder_frames: u64,
    cpu_prefinal_composite_frames: u64,
    cpu_prefinal_overlay_frames: u64,
    dynamic_palette_draws: u64,
    missing_variant_draws: u64,
    stable_preview_draws: u64,
    stable_effect_draws: u64,
    dynamic_material_draws: u64,
    effect_material_draws: u64,
    dynamic_material_fallback_draws: u64,
    dynamic_material_fallback_instance_source_draws: u64,
    dynamic_material_fallback_brightness_draws: u64,
    dynamic_material_fallback_policy_draws: u64,
    dynamic_material_fallback_missing_effect_draws: u64,
    dynamic_material_fallback_unsupported_draws: u64,
    unsupported_material_draws: u64,
    missing_art_draws: u64,
    unkeyed_fallback_draws: u64,
    unkeyed_bg_fallback_draws: u64,
    unkeyed_bg12_fallback_draws: u64,
    unkeyed_bg3_fallback_draws: u64,
    unkeyed_sprite_fallback_draws: u64,
    mixed_overlay_bg_effect_draws: u64,
    mixed_overlay_bg_effect_candidates: u64,
    mixed_overlay_bg_effect_culled_invisible_main: u64,
    mixed_overlay_bg_effect_reject_complex_frame: u64,
    mixed_overlay_bg_effect_reject_complex_brightness: u64,
    mixed_overlay_bg_effect_reject_complex_invalid_layer: u64,
    mixed_overlay_bg_effect_reject_complex_mosaic: u64,
    mixed_overlay_bg_effect_reject_complex_sub_window: u64,
    mixed_overlay_bg_effect_reject_complex_effect_bounds: u64,
    mixed_overlay_bg_effect_reject_complex_scanline_main: u64,
    mixed_overlay_bg_effect_reject_complex_layer_window: u64,
    mixed_overlay_bg_effect_reject_complex_color_math: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_clip: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_subscreen: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_fixed_color: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain: u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front:
        u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order:
        u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect:
        u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex:
        u64,
    mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch:
        u64,
    mixed_overlay_bg_effect_reject_cgram_mismatch: u64,
    mixed_overlay_bg_effect_reject_overlap: u64,
}

pub struct ModernAssetLiveFrameReport {
    failure_line: Option<String>,
    fallback_presentation_context: Option<crate::PresentationContext>,
}

impl ModernAssetLiveFrameReport {
    pub fn failure_line(&self) -> Option<&str> {
        self.failure_line.as_deref()
    }

    pub fn fallback_presentation_context(&self) -> Option<crate::PresentationContext> {
        self.fallback_presentation_context
    }
}

impl ModernAssetLiveStats {
    pub fn from_env() -> Self {
        let enabled = env::var_os("ZELDA3_VARIANT_LIVE_STATS").is_some();
        let log_every_frames = env::var("ZELDA3_VARIANT_LIVE_STATS_EVERY")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value != 0)
            .unwrap_or(300);
        let require_full_gpu_path =
            env_flag_default_true(env::var("ZELDA3_REQUIRE_FULL_GPU_PATH").ok().as_deref());
        Self {
            enabled,
            require_full_gpu_path,
            log_every_frames,
            ..Self::default()
        }
    }

    pub fn record_variant_stats(
        &mut self,
        stats: VariantAtlasRenderStats,
    ) -> Option<ModernGpuPathFallback> {
        if let Some(fallback) = self.full_gpu_violation(&stats) {
            return Some(fallback);
        }
        if !self.enabled {
            return None;
        }
        self.frames += 1;
        self.stable_draws += u64::from(stats.stable_draws);
        self.fallback_draws += u64::from(stats.fallback_draws);
        self.live_index_draws += u64::from(stats.live_index_draws);
        self.live_index_bg_draws += u64::from(stats.live_index_bg_draws);
        self.live_index_bg12_draws += u64::from(stats.live_index_bg12_draws);
        self.live_index_bg3_draws += u64::from(stats.live_index_bg3_draws);
        self.live_index_sprite_draws += u64::from(stats.live_index_sprite_draws);
        self.gpu_prefinal_base_frames += u64::from(stats.gpu_prefinal_base_frames);
        self.gpu_screen_builder_frames += u64::from(stats.gpu_screen_builder_frames);
        self.cpu_prefinal_composite_frames += u64::from(stats.cpu_prefinal_composite_frames);
        self.cpu_prefinal_overlay_frames += u64::from(stats.cpu_prefinal_overlay_frames);
        self.dynamic_palette_draws += u64::from(stats.dynamic_palette_draws);
        self.missing_variant_draws += u64::from(stats.missing_variant_draws);
        self.stable_preview_draws += u64::from(stats.stable_preview_draws);
        self.stable_effect_draws += u64::from(stats.stable_effect_draws);
        self.dynamic_material_draws += u64::from(stats.dynamic_material_draws);
        self.effect_material_draws += u64::from(stats.effect_material_draws);
        self.dynamic_material_fallback_draws += u64::from(stats.dynamic_material_fallback_draws);
        self.dynamic_material_fallback_instance_source_draws +=
            u64::from(stats.dynamic_material_fallback_instance_source_draws);
        self.dynamic_material_fallback_brightness_draws +=
            u64::from(stats.dynamic_material_fallback_brightness_draws);
        self.dynamic_material_fallback_policy_draws +=
            u64::from(stats.dynamic_material_fallback_policy_draws);
        self.dynamic_material_fallback_missing_effect_draws +=
            u64::from(stats.dynamic_material_fallback_missing_effect_draws);
        self.dynamic_material_fallback_unsupported_draws +=
            u64::from(stats.dynamic_material_fallback_unsupported_draws);
        self.unsupported_material_draws += u64::from(stats.unsupported_material_draws);
        self.missing_art_draws += u64::from(stats.missing_art_draws);
        self.unkeyed_fallback_draws += u64::from(stats.unkeyed_fallback_draws);
        self.unkeyed_bg_fallback_draws += u64::from(stats.unkeyed_bg_fallback_draws);
        self.unkeyed_bg12_fallback_draws += u64::from(stats.unkeyed_bg12_fallback_draws);
        self.unkeyed_bg3_fallback_draws += u64::from(stats.unkeyed_bg3_fallback_draws);
        self.unkeyed_sprite_fallback_draws += u64::from(stats.unkeyed_sprite_fallback_draws);
        self.mixed_overlay_bg_effect_draws += u64::from(stats.mixed_overlay_bg_effect_draws);
        self.mixed_overlay_bg_effect_candidates +=
            u64::from(stats.mixed_overlay_bg_effect_candidates);
        self.mixed_overlay_bg_effect_culled_invisible_main +=
            u64::from(stats.mixed_overlay_bg_effect_culled_invisible_main);
        self.mixed_overlay_bg_effect_reject_complex_frame +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_frame);
        self.mixed_overlay_bg_effect_reject_complex_brightness +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_brightness);
        self.mixed_overlay_bg_effect_reject_complex_invalid_layer +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_invalid_layer);
        self.mixed_overlay_bg_effect_reject_complex_mosaic +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_mosaic);
        self.mixed_overlay_bg_effect_reject_complex_sub_window +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_sub_window);
        self.mixed_overlay_bg_effect_reject_complex_effect_bounds +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_effect_bounds);
        self.mixed_overlay_bg_effect_reject_complex_scanline_main +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_scanline_main);
        self.mixed_overlay_bg_effect_reject_complex_layer_window +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_layer_window);
        self.mixed_overlay_bg_effect_reject_complex_color_math +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math);
        self.mixed_overlay_bg_effect_reject_complex_color_math_clip +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_clip);
        self.mixed_overlay_bg_effect_reject_complex_color_math_subscreen +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen);
        self.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color);
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap);
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg);
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj +=
            u64::from(stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj);
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex,
        );
        self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch += u64::from(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch,
        );
        self.mixed_overlay_bg_effect_reject_cgram_mismatch +=
            u64::from(stats.mixed_overlay_bg_effect_reject_cgram_mismatch);
        self.mixed_overlay_bg_effect_reject_overlap +=
            u64::from(stats.mixed_overlay_bg_effect_reject_overlap);
        if self.should_log_summary() {
            self.log_summary();
        }
        None
    }

    pub fn record_present_output(
        &mut self,
        output: &crate::ModernAssetFramePresentOutput,
        resources: &crate::ModernAssetFrameResources,
    ) -> ModernAssetLiveFrameReport {
        let mut report = self.record_present_result(&output.result);
        if report.failure_line.is_none() && !output.result.is_presented() {
            report.failure_line = resources
                .unhandled_gpu_asset_frame_line()
                .map(ToString::to_string);
        }
        if report.failure_line.is_none() && !output.result.is_presented() {
            report.fallback_presentation_context = Some(crate::PresentationContext {
                in_dungeon: output.in_dungeon,
            });
        }
        report
    }

    fn record_present_result(
        &mut self,
        result: &crate::ModernAssetFramePresentResult,
    ) -> ModernAssetLiveFrameReport {
        let failure_line = match result {
            crate::ModernAssetFramePresentResult::Presented { via, variant_stats } => {
                if let Some(fallback) = self.full_gpu_route_violation(via, variant_stats.as_ref()) {
                    Some(format_live_full_gpu_failure_line(fallback))
                } else if let Some(stats) = variant_stats {
                    self.record_variant_stats(*stats)
                        .map(|fallback| format_live_full_gpu_failure_line(fallback))
                } else {
                    self.record_non_variant_gpu_route(via);
                    None
                }
            }
            _ => None,
        };
        ModernAssetLiveFrameReport {
            failure_line,
            fallback_presentation_context: None,
        }
    }

    fn full_gpu_violation(&self, stats: &VariantAtlasRenderStats) -> Option<ModernGpuPathFallback> {
        self.full_gpu_route_violation("variant-gpu", Some(stats))
    }

    fn full_gpu_route_violation(
        &self,
        via: &str,
        stats: Option<&VariantAtlasRenderStats>,
    ) -> Option<ModernGpuPathFallback> {
        self.require_full_gpu_path
            .then(|| modern_gpu_path_fallback_reason(via, stats))
            .flatten()
    }

    fn record_non_variant_gpu_route(&mut self, via: &str) {
        if !self.enabled {
            return;
        }
        if via == "mode7-source-gpu" {
            self.frames += 1;
            self.mode7_source_gpu_frames += 1;
            if self.should_log_summary() {
                self.log_summary();
            }
        }
    }

    fn should_log_summary(&self) -> bool {
        self.log_every_frames != 0 && self.frames % self.log_every_frames == 0
    }

    fn log_summary(&self) {
        eprintln!(
            "variant_live_summary frames={} mode7_source_gpu_frames={} variant_draws={} fallback_draws={} live_index_draws={} live_index_bg_draws={} live_index_bg12_draws={} live_index_bg3_draws={} live_index_sprite_draws={} gpu_prefinal_base_frames={} gpu_screen_builder_frames={} cpu_prefinal_composite_frames={} cpu_prefinal_overlay_frames={} dynamic_palette_draws={} missing_variant_draws={} stable_preview_draws={} stable_effect_draws={} dynamic_material_draws={} effect_material_draws={} dynamic_material_fallback_draws={} dynamic_material_fallback_instance_source_draws={} dynamic_material_fallback_brightness_draws={} dynamic_material_fallback_policy_draws={} dynamic_material_fallback_missing_effect_draws={} dynamic_material_fallback_unsupported_draws={} unsupported_material_draws={} missing_art_draws={} unkeyed_fallback_draws={} unkeyed_bg_fallback_draws={} unkeyed_bg12_fallback_draws={} unkeyed_bg3_fallback_draws={} unkeyed_sprite_fallback_draws={} mixed_overlay_bg_effect_draws={} mixed_overlay_bg_effect_candidates={} mixed_overlay_bg_effect_culled_invisible_main={} mixed_overlay_bg_effect_reject_complex_frame={} mixed_overlay_bg_effect_reject_complex_brightness={} mixed_overlay_bg_effect_reject_complex_invalid_layer={} mixed_overlay_bg_effect_reject_complex_mosaic={} mixed_overlay_bg_effect_reject_complex_sub_window={} mixed_overlay_bg_effect_reject_complex_effect_bounds={} mixed_overlay_bg_effect_reject_complex_scanline_main={} mixed_overlay_bg_effect_reject_complex_layer_window={} mixed_overlay_bg_effect_reject_complex_color_math={} mixed_overlay_bg_effect_reject_complex_color_math_clip={} mixed_overlay_bg_effect_reject_complex_color_math_subscreen={} mixed_overlay_bg_effect_reject_complex_color_math_fixed_color={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex={} mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch={} mixed_overlay_bg_effect_reject_cgram_mismatch={} mixed_overlay_bg_effect_reject_overlap={}",
            self.frames,
            self.mode7_source_gpu_frames,
            self.stable_draws,
            self.fallback_draws,
            self.live_index_draws,
            self.live_index_bg_draws,
            self.live_index_bg12_draws,
            self.live_index_bg3_draws,
            self.live_index_sprite_draws,
            self.gpu_prefinal_base_frames,
            self.gpu_screen_builder_frames,
            self.cpu_prefinal_composite_frames,
            self.cpu_prefinal_overlay_frames,
            self.dynamic_palette_draws,
            self.missing_variant_draws,
            self.stable_preview_draws,
            self.stable_effect_draws,
            self.dynamic_material_draws,
            self.effect_material_draws,
            self.dynamic_material_fallback_draws,
            self.dynamic_material_fallback_instance_source_draws,
            self.dynamic_material_fallback_brightness_draws,
            self.dynamic_material_fallback_policy_draws,
            self.dynamic_material_fallback_missing_effect_draws,
            self.dynamic_material_fallback_unsupported_draws,
            self.unsupported_material_draws,
            self.missing_art_draws,
            self.unkeyed_fallback_draws,
            self.unkeyed_bg_fallback_draws,
            self.unkeyed_bg12_fallback_draws,
            self.unkeyed_bg3_fallback_draws,
            self.unkeyed_sprite_fallback_draws,
            self.mixed_overlay_bg_effect_draws,
            self.mixed_overlay_bg_effect_candidates,
            self.mixed_overlay_bg_effect_culled_invisible_main,
            self.mixed_overlay_bg_effect_reject_complex_frame,
            self.mixed_overlay_bg_effect_reject_complex_brightness,
            self.mixed_overlay_bg_effect_reject_complex_invalid_layer,
            self.mixed_overlay_bg_effect_reject_complex_mosaic,
            self.mixed_overlay_bg_effect_reject_complex_sub_window,
            self.mixed_overlay_bg_effect_reject_complex_effect_bounds,
            self.mixed_overlay_bg_effect_reject_complex_scanline_main,
            self.mixed_overlay_bg_effect_reject_complex_layer_window,
            self.mixed_overlay_bg_effect_reject_complex_color_math,
            self.mixed_overlay_bg_effect_reject_complex_color_math_clip,
            self.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            self.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex,
            self.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch,
            self.mixed_overlay_bg_effect_reject_cgram_mismatch,
            self.mixed_overlay_bg_effect_reject_overlap
        );
    }
}

fn env_flag_default_true(value: Option<&str>) -> bool {
    match value.map(str::trim) {
        Some("0") => false,
        Some(value)
            if value.eq_ignore_ascii_case("false")
                || value.eq_ignore_ascii_case("off")
                || value.eq_ignore_ascii_case("no") =>
        {
            false
        }
        _ => true,
    }
}

fn format_live_full_gpu_failure_line(fallback: ModernGpuPathFallback) -> String {
    format!(
        "gpu_path_unsupported_live reason={} count={}",
        fallback.reason, fallback.count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_gpu_live_guard_defaults_on_with_explicit_opt_out() {
        assert!(env_flag_default_true(None));
        assert!(env_flag_default_true(Some("1")));
        assert!(env_flag_default_true(Some("true")));
        assert!(env_flag_default_true(Some("yes")));
        assert!(!env_flag_default_true(Some("0")));
        assert!(!env_flag_default_true(Some("false")));
        assert!(!env_flag_default_true(Some("OFF")));
        assert!(!env_flag_default_true(Some(" no ")));
    }

    #[test]
    fn full_gpu_live_guard_reports_cpu_prefinal_violation() {
        let stats = VariantAtlasRenderStats {
            cpu_prefinal_composite_frames: 1,
            ..Default::default()
        };
        let strict = ModernAssetLiveStats {
            require_full_gpu_path: true,
            ..Default::default()
        };
        assert_eq!(
            strict.full_gpu_violation(&stats),
            Some(ModernGpuPathFallback {
                reason: "prefinal-composite-cpu",
                count: 1,
            })
        );

        let opt_out = ModernAssetLiveStats {
            require_full_gpu_path: false,
            ..Default::default()
        };
        assert_eq!(opt_out.full_gpu_violation(&stats), None);
    }

    #[test]
    fn full_gpu_live_guard_reports_source_art_violation() {
        let stats = VariantAtlasRenderStats {
            unsupported_material_draws: 2,
            live_index_draws: 3,
            ..Default::default()
        };
        let strict = ModernAssetLiveStats {
            require_full_gpu_path: true,
            ..Default::default()
        };

        assert_eq!(
            strict.full_gpu_violation(&stats),
            Some(ModernGpuPathFallback {
                reason: "unsupported-material",
                count: 2,
            })
        );
    }

    fn live_resources(gpu_asset_mode: bool) -> crate::ModernAssetFrameResources {
        crate::ModernAssetFrameResources {
            source_atlas: None,
            variant_atlas: None,
            hd_overrides: None,
            mode7_source_chars: None,
            gpu_asset_mode,
            variant_gpu_mode: gpu_asset_mode,
        }
    }

    #[test]
    fn record_present_output_owns_live_gpu_failure_line() {
        let mut live = ModernAssetLiveStats {
            require_full_gpu_path: true,
            ..Default::default()
        };
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Presented {
                via: "variant-gpu",
                variant_stats: Some(VariantAtlasRenderStats {
                    cpu_prefinal_overlay_frames: 2,
                    ..Default::default()
                }),
            },
            in_dungeon: false,
        };

        let report = live.record_present_output(&output, &live_resources(false));

        assert_eq!(
            report.failure_line(),
            Some("gpu_path_unsupported_live reason=prefinal-overlay-cpu count=2")
        );
        assert_eq!(report.fallback_presentation_context(), None);
    }

    #[test]
    fn record_present_output_rejects_mode7_live_vram_route() {
        let mut live = ModernAssetLiveStats {
            require_full_gpu_path: true,
            ..Default::default()
        };
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Presented {
                via: "mode7-gpu",
                variant_stats: None,
            },
            in_dungeon: false,
        };

        let report = live.record_present_output(&output, &live_resources(true));

        assert_eq!(
            report.failure_line(),
            Some("gpu_path_unsupported_live reason=mode7-live-vram count=1")
        );
        assert_eq!(report.fallback_presentation_context(), None);
    }

    #[test]
    fn record_present_output_accepts_source_backed_mode7_route() {
        let mut live = ModernAssetLiveStats {
            enabled: true,
            require_full_gpu_path: true,
            ..Default::default()
        };
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Presented {
                via: "mode7-source-gpu",
                variant_stats: None,
            },
            in_dungeon: false,
        };

        let report = live.record_present_output(&output, &live_resources(true));

        assert_eq!(report.failure_line(), None);
        assert_eq!(report.fallback_presentation_context(), None);
        assert_eq!(live.frames, 1);
        assert_eq!(live.mode7_source_gpu_frames, 1);
    }

    #[test]
    fn record_present_output_rejects_vram_gpu_route() {
        let mut live = ModernAssetLiveStats {
            require_full_gpu_path: true,
            ..Default::default()
        };
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Presented {
                via: "vram-gpu",
                variant_stats: None,
            },
            in_dungeon: false,
        };

        let report = live.record_present_output(&output, &live_resources(true));

        assert_eq!(
            report.failure_line(),
            Some("gpu_path_unsupported_live reason=vram-live-gpu count=1")
        );
        assert_eq!(report.fallback_presentation_context(), None);
    }

    #[test]
    fn record_present_output_owns_unhandled_gpu_asset_failure_line() {
        let mut live = ModernAssetLiveStats::default();
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Unhandled,
            in_dungeon: true,
        };

        let report = live.record_present_output(&output, &live_resources(true));

        assert_eq!(
            report.failure_line(),
            Some("modern asset renderer did not handle a GPU asset frame")
        );
        assert_eq!(report.fallback_presentation_context(), None);
    }

    #[test]
    fn record_present_output_owns_fallback_context() {
        let mut live = ModernAssetLiveStats::default();
        let output = crate::ModernAssetFramePresentOutput {
            result: crate::ModernAssetFramePresentResult::Unhandled,
            in_dungeon: true,
        };

        let report = live.record_present_output(&output, &live_resources(false));

        assert_eq!(report.failure_line(), None);
        assert_eq!(
            report.fallback_presentation_context(),
            Some(crate::PresentationContext { in_dungeon: true })
        );
    }
}
