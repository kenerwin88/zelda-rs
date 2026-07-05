#!/usr/bin/env python3
"""Guard renderer-owned source rendering boundaries.

The live/default GPU paths should hand a `GpuFrame` plus source table to the
renderer crate. The binary may manually extract source-backed modern frames only
when it needs intermediate draw data for diagnostics or authoring metadata.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MAIN_RS = REPO / "zelda3-bin" / "src" / "main.rs"
BOUNDARY_SOURCE_FILES = (
    MAIN_RS,
    REPO / "zelda3-bin" / "src" / "gpu_capture.rs",
    REPO / "zelda3-bin" / "src" / "play_renderer.rs",
)

MANUAL_EXTRACT = "extract_modern_frame_from_sources"
REQUIRED_RENDERER_OWNED_CALLS = (
    "ModernAssetFrameLivePresentInput",
    "present_modern_asset_live_frame_from_entries",
    "ModernAssetFrameResources::load_from_env",
    "RendererMode::from_effective_env",
    "ModernAssetFrameScene",
    "ModernAssetLiveStats",
    "ModernIndexCompareStats",
    "ModernIndexCompareRunConfig",
    "ModernIndexCompareFrameOutputInput",
    "load_resources_from_env",
    "render_compare_frame_output_from_entries",
    "ModernIndexCompareOutputStream",
    "summary_line_if_enabled",
    "failure_line()",
    "ModernAtlasCompareResources",
    "compare_frame_rgba",
    "source_table_from_entries",
    "compare_gpu_render_frame_bgra_to_rgba",
    "render_hash_frame_bgra",
    "gpu_render_hash_frame_rgba",
    "render_hash_pair_bgra_rgba",
    "render_fingerprint_leaf_bgra",
    "GpuFrameCaptureInput",
    "GpuFrameRegisterSnapshot",
    "GpuFrame::from_capture_input",
    "from_player_indoors_flag",
    "render_hd_capture_from_sources",
)

FORBIDDEN_SOURCE_RENDER_CALLS = (
    "render_modern_frame_full_scaled_from_sources",
)

FORBIDDEN_MODERN_ATLAS_COMPARE_CALLS = (
    "extract_modern_frame_with_atlas",
    "render_modern_frame_software",
)

FORBIDDEN_VRAM_EXTRACT_CALLS = (
    "extract_modern_frame_from_vram",
    "extract_modern_sprites_from_vram",
)

FORBIDDEN_GRANULAR_LIVE_PRESENT_CALLS = (
    "frontend.present_modern_asset_frame(",
    "present_modern_variant_gpu_from_sources",
    "present_modern_gpu_from_sources",
    "present_modern_gpu_from_vram",
    "present_modern_frame_from_sources",
    "present_modern_mode7_gpu",
    "frontend.present_gpu_frame_with_context(&gpu_frame",
    "GpuFrame::from_source_and_raw_scanlines",
)

FORBIDDEN_ASSET_POLICY_CALLS = (
    "source_atlas_renderer_mode",
    "variant_atlas_renderer_mode",
    "load_source_atlas_for_mode",
    "load_variant_atlas_for_mode",
    "effective_renderer_mode_from_env_value",
    "EffectiveRendererMode::from_env_value",
    "ZELDA3_VARIANT_ATLAS",
    "ModernAssetFrameResources::load_for_mode(",
    "ModernIndexCompareResources::load_for_mode(",
    "load_modern_overworld_tile_atlas(",
    "EffectiveRendererMode::from_env()",
    "uses_source_atlas()",
)

FORBIDDEN_HD_OVERRIDE_CALLS = (
    "ModernHdOverrides::from_env",
    "HdOverrideCtx::new",
    "HdOverrideCtx::disabled",
)

FORBIDDEN_LIVE_STATS_POLICY_CALLS = (
    "struct VariantLiveStats",
    "env_flag_default_true",
    "ZELDA3_VARIANT_LIVE_STATS",
    "ZELDA3_REQUIRE_FULL_GPU_PATH",
    "variant_live_stats.record_variant_stats(",
    "variant_live_stats.record_present_result(",
    "variant_live_stats.record_present_output(",
    "modern_assets.gpu_asset_mode()",
    "modern_assets.unhandled_gpu_asset_frame_line()",
    ".full_gpu_failure_line",
    ".fallback_presentation_context()",
    "present.result.is_presented()",
    "in_dungeon: present.in_dungeon",
    "gpu_path_unsupported_live reason={}",
    "modern asset renderer did not handle a GPU asset frame",
)

FORBIDDEN_MODERN_INDEX_COMPARE_POLICY_CALLS = (
    "ModernIndexCompareScene",
    "ModernIndexCompareFrameRenderInput",
    "ZELDA3_MODERN_INDEX_COMPARE_SUMMARY",
    "ZELDA3_MODERN_INDEX_COMPARE_PROGRESS",
    "ZELDA3_VARIANT_TRACE_PIXEL",
    "variant_trace_pixel_env(",
    "parse_variant_trace_pixel(",
    "print_variant_pixel_traces(",
    "variant_pixel_trace frame=",
    ".variant_traces",
    ".trace_pixel",
    ".trace_lines",
    "rendered.report",
    "rendered.output_lines()",
    "render_compare_frame(",
    "render_compare_frame_output(",
    "modern_index_compare != 0",
    "frames % modern_index_compare",
    "completed_frame % modern_index_compare",
    "require_full_gpu_path && modern_index_compare",
    "require_modern_index_parity && modern_index_compare",
    ".require_full_gpu_path()",
    ".require_modern_index_parity()",
    "require_full_gpu_path:",
    "require_modern_index_parity:",
    "modern_index_compare_count",
    "modern_index_compare_bad_count",
    "modern_index_compare_variant_draws",
    "modern_gpu_path_fallback_reason(",
)

FORBIDDEN_MODERN_INDEX_FRAME_REPORT_CALLS = (
    "modern_index_compare_stats.record(",
    "modern_index_compare_stats.full_gpu_fallback(",
    "modern_index_compare_stats.should_print_frame(",
    "modern_index_compare_stats.frame_line(",
    "modern_index_compare_stats.progress_line(",
    "modern_index_compare_stats.record_frame(",
    "modern_index_compare_stats.summary_enabled(",
    "modern_index_compare_stats.summary_line(",
)

FORBIDDEN_MODERN_INDEX_DUMP_POLICY_CALLS = (
    "ModernIndexCompareDumpPaths",
    "dump_paths_for_frame(",
    "write_dump_for_frame(",
    "write_modern_index_compare_dump(",
    "ZELDA3_MODERN_INDEX_DUMP_FRAME",
    "/tmp/classic_",
    "/tmp/modern_index_",
    "dumped classic frame to ",
    "dumped modern_index frame to ",
)

FORBIDDEN_MODERN_INDEX_RESOURCE_POLICY_CALLS = (
    "atlas_gpu_compare",
    "variant_gpu_compare",
    "let modern_gpu_headless",
    "let modern_variant_headless",
    "let variant_atlas = if modern_index_compare",
    "let source_atlas = if modern_index_compare",
    "load_modern_overworld_index_atlas",
    "load_modern_dungeon_index_atlas",
)

FORBIDDEN_MODERN_SCENE_POLICY_CALLS = (
    "renderer::ModernAssetFrameScene::from_in_dungeon(",
    "mode_str: Option<String> = match module",
    "mode_label = match module",
)

FORBIDDEN_SOURCE_TABLE_VIEW_CALLS = (
    "struct VramChrSourceTableView",
    "impl renderer::modern_extract::SourceTableView for VramChrSourceTableView",
    "fn vram_chr_source_table_view(",
    "fn logical_chr_src_tuple(",
    "renderer::MappedSourceTableView::new(",
    "renderer::MappedSourceTableView::from_entries(",
)

FORBIDDEN_FRAME_COMPARE_CALLS = (
    "struct GpuRenderDiff",
    "fn compare_bgra_to_rgba",
    "fn compare_rgba_to_rgba",
    "fn render_frame_rgb_hash_bgra",
    "fn render_frame_rgb_hash_rgba",
)

FORBIDDEN_DIRECT_MODERN_INDEX_COMPARE_CALLS = (
    "renderer::compare_rgba_to_rgba(",
    "renderer::compare_modern_index_rgba(",
    "renderer::modern_gpu::render_modern_index_compare_frame(",
)

FORBIDDEN_DIRECT_MODERN_ATLAS_COMPARE_CALLS = (
    "ModernAtlasCompareFrameInput",
    "modern_atlas_compare_resources.compare_frame(",
    "renderer::modern_gpu::compare_modern_atlas_to_rgba(",
    "modern_atlas_compare_resources.atlas()",
)

FORBIDDEN_DIRECT_GPU_RENDER_COMPARE_CALLS = (
    "renderer::compare_bgra_to_rgba(",
    "renderer::compare_gpu_render_bgra_to_rgba(",
    ".comparison.",
    ".divergence_line.",
    "gpu-render-divergence frame=",
)

FORBIDDEN_RENDER_HASH_REPORT_CALLS = (
    '"render-hash frame=',
    '"gpu-render-hash frame=',
)

FORBIDDEN_RAW_RENDER_HASH_CALLS = (
    "renderer::render_frame_rgb_hash_bgra(",
    "renderer::render_frame_rgb_hash_rgba(",
)

FORBIDDEN_GPU_FRAME_ASSEMBLY_CALLS = (
    "GpuFrame {",
)

FORBIDDEN_GPU_SCANLINE_CAPTURE_CALLS = (
    "scanlines_from_raw",
)

FORBIDDEN_MAIN_GPU_PLAY_BACKEND_CALLS = (
    "trait PlayRendererBackend",
    "struct CpuPlayRenderer",
    "fn play_renderer_from_env",
    "draw_play_ppu_frame(",
    "gpu_frame_from_ppu(",
    "ModernIndexCompareFrameOutputInput",
    "render_compare_frame_output_from_entries(",
    "ModernIndexCompareOutputLines",
    "ModernIndexCompareOutputStream",
    "emit_modern_index_compare_output_lines(",
    "output_lines.has_failure",
    "compare_frame_rgba(",
    "modern_atlas_compare.render_report_from_capture(",
    "modern_index_compare.render_output_from_capture(",
    "ModernAtlasCompareResources::load(",
    "load_modern_atlas_compare_resources",
    "render_modern_atlas_compare_report_from_capture",
    "modern_atlas_compare_run(",
    "renderer::OffscreenRenderer",
    "OffscreenRenderer::new(",
    "let mut gpu_readback = if render_hash_log != 0",
    "new_gpu_readback_renderer(",
    "let mut render_frame = vec![0u8; 256 * 224 * 4]",
    "GPU readback renderer allocated",
    "gpu_readback.required()",
    ".render_gpu_capture_rgba(",
    ".render_bgra_frame_to_rgba(",
    ".upload_bgra_frame(",
    ".render_to_rgba(",
    "load_resources_from_env(",
    "play_renderer::from_env(",
    "renderer.configure_frontend(",
    "renderer.frontend().",
    "renderer.frontend_mut().",
    "let (mut renderer, mut frontend)",
    "renderer.present_frame(&mut game, &mut frontend",
    "ZELDA3_RENDER_BACKEND",
    "ZELDA3_RENDERER",
    "struct GpuPlayRenderer",
    "impl PlayRendererBackend for GpuPlayRenderer",
    "ModernAssetFrameResources::load_from_env",
    "ModernAssetLiveStats::from_env()",
    "LiveGpuFrameCapture::from_game(",
    "present_modern_asset_live_frame_from_entries(",
    "source_table_from_entries(",
    "render_hd_capture_from_gpu_capture(",
    "render_hd_capture_from_sources(",
    "compare_gpu_render_frame_bgra_to_rgba(",
    "compare_gpu_render_current_frame(",
    ".compare_current_frame(",
    ".render_report_from_game(",
    ".render_output_from_game(",
    "gpu_render_compare != 0",
    "frames % gpu_render_compare",
    "gpu_render_compare_count",
    "gpu_render_compare_last_frame",
    "gpu_render_compare_last_hash",
    "fn cgram_match(",
    "gpu-render-state frame=",
    "renderer::render_hash_frame_bgra(",
    "renderer::gpu_render_hash_frame_rgba(",
    "renderer::render_hash_pair_bgra_rgba(",
    "render_gpu_hash_frame_rgba_line(",
    " render_hash_pair_bgra_rgba(",
    "render_hash_pair_bgra_rgba,",
    "renderer::render_fingerprint_leaf_bgra(",
    "renderer::ModernIndexCompareRunConfig",
    "renderer::ModernIndexCompareStats",
    "renderer::ModernIndexCompareResources",
    "let modern_index_compare_resources",
    "load_modern_index_compare_resources",
    "render_modern_index_compare_output_from_capture",
    "RendererMode::parse(",
    "RendererMode::ModernCompare",
    "RendererMode::Modern {",
    "RendererMode::from_effective_env()",
    "modern asset load failed",
)


@dataclass(frozen=True)
class Occurrence:
    line_number: int
    function: str | None
    line: str
    context: str


def enclosing_function(lines: list[str], index: int) -> str | None:
    fn_re = re.compile(r"^\s*fn\s+([A-Za-z0-9_]+)\b")
    for line in reversed(lines[: index + 1]):
        match = fn_re.match(line)
        if match:
            return match.group(1)
    return None


def context_window(lines: list[str], index: int, radius: int = 45) -> str:
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)
    return "\n".join(lines[start:end])


def manual_extract_occurrences(source: str) -> list[Occurrence]:
    lines = source.splitlines()
    occurrences: list[Occurrence] = []
    for index, line in enumerate(lines):
        if MANUAL_EXTRACT not in line:
            continue
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        occurrences.append(
            Occurrence(
                line_number=index + 1,
                function=enclosing_function(lines, index),
                line=line.strip(),
                context=context_window(lines, index),
            )
        )
    return occurrences


def check_source_text(source: str) -> list[str]:
    errors: list[str] = []
    for required in REQUIRED_RENDERER_OWNED_CALLS:
        if required not in source:
            errors.append(f"missing renderer-owned source API call: {required}")

    occurrences = manual_extract_occurrences(source)
    for occurrence in occurrences:
        fn = occurrence.function or "<module>"
        errors.append(
            "manual source extraction escaped renderer boundary at "
            f"zelda3-bin/src/main.rs:{occurrence.line_number} "
            f"in {fn}: {occurrence.line}"
        )
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for forbidden in FORBIDDEN_SOURCE_RENDER_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "low-level source render escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_ATLAS_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern atlas compare render escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_VRAM_EXTRACT_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "manual VRAM extraction escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_GRANULAR_LIVE_PRESENT_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "granular live present escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_ASSET_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern asset loading policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_HD_OVERRIDE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "HD override loading policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_LIVE_STATS_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "live modern asset stats policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_INDEX_COMPARE_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern index compare stats policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_INDEX_FRAME_REPORT_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern index frame report policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_INDEX_DUMP_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern index dump policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_INDEX_RESOURCE_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern index resource policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MODERN_SCENE_POLICY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern scene policy escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_SOURCE_TABLE_VIEW_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "source table view adapter escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_FRAME_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "frame compare helper escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_DIRECT_MODERN_INDEX_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern index compare diff assembly escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_DIRECT_MODERN_ATLAS_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern atlas compare execution escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_DIRECT_GPU_RENDER_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "gpu render compare diff assembly escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_RENDER_HASH_REPORT_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "render hash report escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_RAW_RENDER_HASH_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "raw render hash escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_GPU_FRAME_ASSEMBLY_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "gpu frame assembly escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_GPU_SCANLINE_CAPTURE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "gpu scanline capture conversion escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_main_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for forbidden in FORBIDDEN_MAIN_GPU_PLAY_BACKEND_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "live GPU play backend ownership escaped gpu_capture boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def boundary_source_text() -> str:
    return "\n".join(path.read_text() for path in BOUNDARY_SOURCE_FILES)


def main() -> int:
    source = boundary_source_text()
    errors = check_source_text(source)
    errors.extend(check_main_text(MAIN_RS.read_text()))
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(
        "renderer source boundary ok "
        f"manual_extracts={len(manual_extract_occurrences(source))} "
        f"renderer_owned_apis={len(REQUIRED_RENDERER_OWNED_CALLS)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
