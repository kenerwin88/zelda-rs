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

MANUAL_EXTRACT = "extract_modern_frame_from_sources"
REQUIRED_RENDERER_OWNED_CALLS = (
    "present_modern_asset_frame",
    "ModernAssetFrameScene",
    "ModernIndexCompareScene",
    "ModernAssetLiveStats",
    "ModernIndexCompareStats",
    "ModernIndexCompareFrameRecord",
    "MappedSourceTableView",
    "MappedSourceTableView::from_entries",
    "compare_modern_index_rgba",
    "compare_gpu_render_bgra_to_rgba",
    "GpuFrame::from_source_and_raw_scanlines",
    "compare_modern_atlas_to_rgba",
    "render_modern_index_compare_frame",
    "record_frame",
    "from_main_module_and_player_indoors_flag",
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
    "present_modern_variant_gpu_from_sources",
    "present_modern_gpu_from_sources",
    "present_modern_gpu_from_vram",
    "present_modern_frame_from_sources",
    "present_modern_mode7_gpu",
)

FORBIDDEN_ASSET_POLICY_CALLS = (
    "source_atlas_renderer_mode",
    "variant_atlas_renderer_mode",
    "load_source_atlas_for_mode",
    "load_variant_atlas_for_mode",
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
)

FORBIDDEN_MODERN_INDEX_COMPARE_POLICY_CALLS = (
    "ZELDA3_MODERN_INDEX_COMPARE_SUMMARY",
    "ZELDA3_MODERN_INDEX_COMPARE_PROGRESS",
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
)

FORBIDDEN_DIRECT_GPU_RENDER_COMPARE_CALLS = (
    "renderer::compare_bgra_to_rgba(",
)

FORBIDDEN_DIRECT_MODERN_ATLAS_COMPARE_HASH_CALLS = (
    "renderer::render_frame_rgb_hash_rgba(&classic_rgba)",
    "renderer::render_frame_rgb_hash_rgba(&modern_render.rgba)",
)

FORBIDDEN_GPU_FRAME_ASSEMBLY_CALLS = (
    "GpuFrame {",
)

FORBIDDEN_GPU_SCANLINE_CAPTURE_CALLS = (
    "scanlines_from_raw",
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
        for forbidden in FORBIDDEN_DIRECT_GPU_RENDER_COMPARE_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "gpu render compare diff assembly escaped renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_DIRECT_MODERN_ATLAS_COMPARE_HASH_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "modern atlas compare hash assembly escaped renderer boundary at "
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


def main() -> int:
    source = MAIN_RS.read_text()
    errors = check_source_text(source)
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
