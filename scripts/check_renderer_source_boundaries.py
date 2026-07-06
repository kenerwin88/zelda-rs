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
ASSET_PALETTE_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "asset_palette_commands.rs"
ASSET_SOURCE_DUMP_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "asset_source_dump_commands.rs"
AUDIO_TRACE_RS = REPO / "zelda3-bin" / "src" / "audio_trace.rs"
DEVELOPER_ROOM_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "developer_room_commands.rs"
FRAME_DUMP_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "frame_dump_commands.rs"
SHEET_DUMP_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "sheet_dump_commands.rs"
INDEX_DUMP_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "index_dump_commands.rs"
OVERWORLD_DUMP_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "overworld_dump_commands.rs"
ROUTE_COVERAGE_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "route_coverage_commands.rs"
GPU_COMPARE_RS = REPO / "zelda3-bin" / "src" / "gpu_compare.rs"
GPU_CAPTURE_RS = REPO / "zelda3-bin" / "src" / "gpu_capture.rs"
HD_AUTHORING_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "hd_authoring_commands.rs"
IMAGE_OUTPUT_RS = REPO / "zelda3-bin" / "src" / "image_output.rs"
INPUT_SCRIPT_RS = REPO / "zelda3-bin" / "src" / "input_script.rs"
GPU_READBACK_RS = REPO / "zelda3-bin" / "src" / "gpu_readback.rs"
PLAY_RENDERER_RS = REPO / "zelda3-bin" / "src" / "play_renderer.rs"
PLAY_COMMANDS_RS = REPO / "zelda3-bin" / "src" / "play_commands.rs"
REPLAY_DIAGNOSTICS_RS = REPO / "zelda3-bin" / "src" / "replay_diagnostics.rs"
REPLAY_SAVE_CONFIG_RS = REPO / "zelda3-bin" / "src" / "replay_save_config.rs"
CLASSIC_FRAME_RENDERER_RS = REPO / "zelda3-bin" / "src" / "classic_frame_renderer.rs"
BOUNDARY_SOURCE_FILES = (
    MAIN_RS,
    ASSET_PALETTE_COMMANDS_RS,
    ASSET_SOURCE_DUMP_COMMANDS_RS,
    AUDIO_TRACE_RS,
    DEVELOPER_ROOM_COMMANDS_RS,
    SHEET_DUMP_COMMANDS_RS,
    INDEX_DUMP_COMMANDS_RS,
    OVERWORLD_DUMP_COMMANDS_RS,
    ROUTE_COVERAGE_COMMANDS_RS,
    CLASSIC_FRAME_RENDERER_RS,
    GPU_COMPARE_RS,
    REPO / "zelda3-bin" / "src" / "gpu_capture.rs",
    HD_AUTHORING_COMMANDS_RS,
    GPU_READBACK_RS,
    IMAGE_OUTPUT_RS,
    INPUT_SCRIPT_RS,
    REPO / "zelda3-bin" / "src" / "play_renderer.rs",
    PLAY_COMMANDS_RS,
    REPLAY_DIAGNOSTICS_RS,
    REPLAY_SAVE_CONFIG_RS,
    REPO / "zelda3-bin" / "src" / "render_diagnostics.rs",
)

MANUAL_EXTRACT = "extract_modern_frame_from_sources"
REQUIRED_RENDERER_OWNED_CALLS = (
    "ModernAssetFrameLivePresentInput",
    "present_modern_asset_live_frame_from_entries",
    "ModernAssetFrameResources::load_live_gpu_from_env",
    "ModernIndexCompareResources::load_live_gpu_from_env",
    "RendererMode::from_effective_mode",
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
    "ModernIndexCompareResources::load_from_env(",
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

FORBIDDEN_REPLAY_FINGERPRINT_RENDER_CALLS = (
    "render_standard_play_frame_bgra(&mut game",
    "fp_render_leaf = render_fingerprint_leaf_bgra(",
)

FORBIDDEN_PLAY_RENDERER_CLASSIC_FRAME_CALLS = (
    "crate::play_renderer::render_play_frame_bgra(",
    "crate::play_renderer::render_standard_play_frame_bgra(",
    "crate::play_renderer::run_play_frame_bgra(",
    "crate::play_renderer::run_play_frame_with_run_what_bgra(",
)

FORBIDDEN_MAIN_PLAY_RENDER_CALLS = (
    "render_play_frame_bgra(",
)

FORBIDDEN_MAIN_CLASSIC_RUN_CALLS = (
    "run_play_frame_bgra(",
    "run_play_frame_with_run_what_bgra(",
)

FORBIDDEN_MAIN_RENDER_HASH_HELPER_CALLS = (
    "render_hash_frame_bgra_line(",
)

FORBIDDEN_MAIN_REPLAY_PLAY_RENDER_HELPER_CALLS = (
    "render_replay_projection_bgra(",
    "render_replay_fingerprint_leaf_bgra(",
)

FORBIDDEN_MAIN_GPU_COMPARE_COMMAND_OWNERSHIP = (
    "fn run_play_gpu_render_compare",
)

FORBIDDEN_GPU_CAPTURE_REPLAY_CLASSIC_HELPERS = (
    "pub(crate) fn replay_projection_bgra",
    "pub(crate) fn replay_fingerprint_leaf_bgra",
)

FORBIDDEN_GPU_CAPTURE_READBACK_OWNERSHIP = (
    "struct GpuReadbackRenderer",
    "struct GpuRgbaReadbackFrame",
    "struct OptionalGpuReadbackRenderer",
    "struct ReplayRenderHashCapture",
    "struct ReplayRenderHashGpuReadback",
    "impl GpuReadbackRenderer",
    "impl OptionalGpuReadbackRenderer",
    "impl ReplayRenderHashCapture",
    "impl ReplayRenderHashGpuReadback",
)

FORBIDDEN_GPU_CAPTURE_COMPARE_OWNERSHIP = (
    "struct ModernCompareModeDefaults",
    "struct ModernIndexCompareRun",
    "struct GpuRenderCompareRun",
    "struct ModernAtlasCompareRun",
    "struct PlayGpuRenderCompareSession",
    "fn modern_compare_mode_defaults_from_env",
    "fn modern_index_compare_run_from_env",
    "fn gpu_render_compare_run",
    "fn play_gpu_render_compare_session",
    "fn compare_gpu_render_current_frame",
    "fn emit_modern_index_compare_output_lines",
    "fn cgram_match",
)

FORBIDDEN_PLAY_RENDERER_CLASSIC_BACKEND_CALLS = (
    "struct CpuPlayRenderer",
    "impl PlayRendererBackend for CpuPlayRenderer",
    "render_play_frame_bgra(",
)

FORBIDDEN_CLASSIC_FRAME_RENDERER_LIVE_BACKEND = (
    "struct CpuPlayRenderer",
    "impl crate::play_renderer::PlayRendererBackend",
    "impl PlayRendererBackend for",
    "fn new_cpu_play_renderer",
)

FORBIDDEN_MAIN_PLAY_RENDERER_DIAGNOSTIC_CALLS = (
    "render_lockstep_artifact_frame_bgra(",
    "render_overworld_screen_dump_bgra(",
    "render_oracle_compare_frames_bgra(",
    "render_lockstep_oracle_frames_in_place(",
)

FORBIDDEN_MAIN_RENDER_DIAGNOSTIC_OWNERSHIP = (
    "struct RenderDiff",
    "fn compare_oracle_render_frame",
    "fn format_render_ppu_summary",
)

FORBIDDEN_MAIN_FRAME_DUMP_COMMAND_OWNERSHIP = (
    "fn run_dump_frame",
    "fn run_dump_overworld_screen",
    "fn run_scan_replay_checkpoints",
    "fn run_dump_replay_checkpoint_ppu",
)

GPU_ASSET_FRAME_DUMP_FUNCTIONS = {
    "run_dump_frame",
    "run_dump_overworld_screen",
    "run_dump_replay_checkpoint_ppu",
}

GPU_ASSET_MAIN_RENDER_FUNCTIONS = {
    "run_smoke_render",
}

GPU_ASSET_PLAY_LOCKSTEP_FUNCTIONS = {
    "run_play_lockstep",
}

FORBIDDEN_PLAY_LOCKSTEP_CLASSIC_PRESENT_CALLS = (
    "NativeFrontend::new_with_options(",
    "std::slice::from_raw_parts(",
    "frontend.present_frame(",
)

GPU_ASSET_LIBRETRO_VIDEO_FUNCTION_FORBIDDEN_FRAMES = {
    "run_compare_libretro_oracle": ("&rust_frame,",),
    "run_play_lockstep": ("&game_frame,",),
}

GPU_ASSET_LOCKSTEP_ARTIFACT_FUNCTIONS = {
    "write_lockstep_parity_failure_artifacts",
}

FORBIDDEN_LOCKSTEP_ARTIFACT_CLASSIC_RUST_FRAME_CALLS = (
    "render_diagnostic_lockstep_artifact_frame_bgra(&mut rust_state",
    'write_argb_frame_png(&dir.join("rust_frame.png")',
)

FORBIDDEN_FRAME_DUMP_CLASSIC_DEFAULT_CALLS = (
    "run_diagnostic_play_frame_bgra(",
    "render_diagnostic_overworld_screen_bgra(",
    "write_argb_frame_png(",
)

GPU_ASSET_DEVELOPER_DESTINATION_FUNCTIONS = {
    "run_dump_developer_destination",
}

FORBIDDEN_DEVELOPER_DESTINATION_CLASSIC_DEFAULT_CALLS = (
    "run_diagnostic_play_frame_bgra(&mut game",
    "write_argb_frame_png(&out_path",
    "<cpu-out.png>",
    "[--gpu",
)

FORBIDDEN_MAIN_GPU_ASSET_RENDER_CALLS = (
    "run_diagnostic_play_frame_bgra(",
    "write_argb_frame_png(",
)

FORBIDDEN_MAIN_ROUTE_COVERAGE_OWNERSHIP = (
    "fn route_coverage_frame_from_game",
    "fn write_route_coverage_log_or_exit",
    "fn run_coverage_probe",
)

FORBIDDEN_MAIN_PLAY_COMMAND_OWNERSHIP = (
    "fn run_frontend_smoke",
    "fn run_play(",
    "fn run_standalone_play",
    "fn run_play_with_state",
    "fn apply_host_menu_action_for_test",
    "mod host_menu_play_tests",
)

FORBIDDEN_MAIN_INPUT_SCRIPT_OWNERSHIP = (
    "struct InputScript",
    "struct InputRule",
    "impl InputScript",
    "fn parse_frame_spec",
    "fn parse_buttons",
)

FORBIDDEN_MAIN_REPLAY_SAVE_CONFIG_OWNERSHIP = (
    "struct ReplaySaveConfig",
    "fn parse_replay_save_args_or_exit",
    "usage: zelda3 --replay-save",
)

FORBIDDEN_MAIN_REPLAY_DIAGNOSTIC_OWNERSHIP = (
    "fn replay_sram_checksum_ok",
    "fn replay_checksum_bytes",
    "fn replay_checksum_ram_range",
    "fn replay_save_ancilla_dump",
    "fn replay_save_ram_page_dump",
    "fn replay_save_ram0400_dump",
    "fn replay_save_ram0000_dump",
    "fn replay_save_requested_ram_page_dump",
    "fn replay_save_room_mask",
    "fn replay_save_garnish_dump",
    "fn replay_save_room_history_dump",
    "fn replay_save_room_mask_dump",
    "fn replay_save_overlord_dump",
    "fn replay_save_sprite_dump",
    "fn replay_save_door_dump",
    "fn replay_save_dungeon_attr_dump",
    "fn replay_save_dungmap_dump",
    "fn replay_save_message_dump",
    "fn replay_save_palette_dump",
)

FORBIDDEN_MAIN_AUDIO_TRACE_OWNERSHIP = (
    "struct AudioFrameStats",
    "fn print_replay_audio_trace",
    "fn replay_dsp_write_events_json",
    "fn replay_checksum_samples",
    "fn replay_checksum_dsp_writes",
    "fn replay_checksum_dsp_write_values",
    "fn fingerprint_audio_hash",
    "fn should_write_fingerprint",
    "fn first_peak_frame",
    "fn max_peak_frame",
    "fn print_audio_window",
)

AUDIO_TRACE_ADVANCE_FUNCTIONS = {
    "run_compare_bootstrap_apu_startup",
    "run_trace_bootstrap_apu_direct_frame",
    "run_trace_startup_audio",
    "run_trace_bsnes_audio",
    "run_compare_bsnes_startup_audio",
    "run_compare_startup_apu_impls",
}

FORBIDDEN_AUDIO_TRACE_CPU_RENDER_CALLS = (
    "run_diagnostic_play_frame_bgra(",
)

FORBIDDEN_MAIN_IMAGE_OUTPUT_OWNERSHIP = (
    "fn write_argb_frame_png",
    "fn write_assets_index_png",
    "fn write_rgba_frame_png",
    "fn decode_rgba_png",
    "const ASSETS_PNG_COLUMNS",
)

FORBIDDEN_MAIN_HD_AUTHORING_COMMAND_OWNERSHIP = (
    "fn write_reference_palette_png",
    "fn run_dump_hd_capture",
    "fn run_slice_hd_cells",
)

FORBIDDEN_MAIN_ASSET_PALETTE_COMMAND_OWNERSHIP = (
    "fn run_dump_reference_palette",
)

FORBIDDEN_MAIN_ASSET_SOURCE_DUMP_COMMAND_OWNERSHIP = (
    "struct AssetsBySourceManifest",
    "struct AssetsBySourceCell",
    "struct PaletteUsageKey",
    "struct PaletteUsageManifest",
    "struct PaletteUsageEntry",
    "fn palette_usage_key_from_chr_source",
    "fn record_palette_usage_count",
    "fn content_hash_source_key",
    "fn index_pattern_hash32",
    "fn bg3_content_source_key",
    "fn palette_usage_entries_from_counts",
    "mod palette_usage_tests",
    "fn run_dump_assets_by_source",
)

FORBIDDEN_MAIN_DEVELOPER_ROOM_COMMAND_OWNERSHIP = (
    "struct DeveloperSandboxTilemapManifest {",
    "struct DeveloperTilesetManifest {",
    "struct DeveloperTilesetEntry {",
    "fn current_developer_location_from_ram(",
    "fn load_developer_route_bookmark(",
    "fn load_developer_destination(",
    "fn developer_sandbox_tilemap_manifest(",
    "fn developer_kakariko_tileset_manifest(",
    "fn load_developer_synthetic_room(",
    "fn load_developer_room_theme_source(",
    "fn write_developer_room_visuals_to_ppu(",
    "fn write_developer_room_palette_from_source(",
    "fn copy_developer_room_chr_from_source(",
    "fn developer_room_kakariko_sample_origin(",
    "fn developer_room_kakariko_visible_cell(",
    "fn run_dump_developer_destination(",
    "fn run_dump_developer_tileset(",
    "const DEVELOPER_ROOM_BG_TILE_BASE:",
    "const DEVELOPER_ROOM_SOURCE_BG_LAYER:",
    "const DEV_TOWN_ROOF:",
    "const DEV_TOWN_WALL:",
    "const DEV_TOWN_DOOR:",
    "const DEV_TOWN_GRASS:",
    "const DEV_TOWN_PATH:",
    "const DEV_TOWN_FENCE:",
    "const DEV_TOWN_SHRUB:",
    "const DEV_TOWN_SIGN:",
    "const DEV_TOWN_TREE:",
    "const DEV_TOWN_CLIFF_TOP:",
    "const DEV_TOWN_CLIFF_FACE:",
    "const DEV_TOWN_FLOWERS:",
    "const DEV_TOWN_STONE:",
    "const DEV_TOWN_HEDGE:",
    "const DEVELOPER_ROOM_KAKARIKO_MUSIC:",
)

FORBIDDEN_MAIN_SHEET_DUMP_COMMAND_OWNERSHIP = (
    "struct SpriteSheetPngManifest",
    "struct SpriteSheetPngCell",
    "fn run_dump_sprite_sheet_png",
    "struct DungeonSheetPngManifest",
    "struct DungeonSheetPngCell",
    "fn run_dump_dungeon_sheet_png",
)

FORBIDDEN_MAIN_INDEX_DUMP_COMMAND_OWNERSHIP = (
    "struct DungeonIndexTileAtlasManifest",
    "struct DungeonIndexTileCellManifest",
    "struct SpriteIndexTileAtlasManifest",
    "struct SpriteIndexTileCellManifest",
    "struct SpriteIndexKey",
    "fn run_dump_dungeon_index_tiles",
    "fn run_dump_sprite_index_tiles",
    "fn dungeon_room_index_probe",
    "const OBJ_SIZE_TABLE",
    "struct SpriteTileProbe",
    "fn sprite_index_probe",
    "fn decode_snes_4bpp_tile_indices",
    "const DUNGEON_BG_CHR_BASE",
    "const DUNGEON_BG1_TILEMAP_WORDS",
)

FORBIDDEN_MAIN_OVERWORLD_DUMP_COMMAND_OWNERSHIP = (
    "struct UniqueOverworldCellAtlasManifest {",
    "struct UniqueOverworldCellManifestEntry {",
    "struct UniqueOverworldCellSource {",
    "struct UniqueOverworldCell {",
    "struct UniqueOverworldCellCollector {",
    "impl UniqueOverworldCellCollector {",
    "struct UniqueOverworldTileAtlasManifest {",
    "struct UniqueOverworldTileManifestEntry {",
    "struct DecodedTilemapEntry {",
    "struct UniqueOverworldTile {",
    "struct UniqueOverworldTileCollector {",
    "impl UniqueOverworldTileCollector {",
    "struct OverworldIndexTile {",
    "struct OverworldIndexTileCollector {",
    "impl OverworldIndexTileCollector {",
    "struct OverworldIndexTileAtlasManifest {",
    "struct OverworldIndexTileCellManifest {",
    "fn decode_tilemap_entry(",
    "fn collect_unique_overworld_cells_from_built_bg2_map(",
    "fn collect_unique_overworld_tiles_from_built_bg2_map(",
    "fn render_snes_4bpp_cell_to_rgba(",
    "fn render_snes_4bpp_tile_to_rgba(",
    "fn render_unique_overworld_cell_atlas(",
    "fn render_unique_overworld_tile_atlas(",
    "fn blit_scaled_rgba_cell(",
    "fn fnv32_bytes(",
    "fn run_dump_unique_overworld_cells(",
    "fn run_dump_unique_overworld_tiles(",
    "const OVERWORLD_BG_CHR_BASE:",
    "const UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT:",
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
    " optional_gpu_readback_renderer(",
    "let mut render_frame = vec![0u8; 256 * 224 * 4]",
    "GPU readback renderer allocated",
    "gpu_readback.required()",
    "capture_gpu_frame_from_game(&mut game)",
    ".render_gpu_capture_rgba(",
    ".render_bgra_frame_to_rgba(",
    ".render_cpu_bgra_frame_rgba(frame)",
    ".render_cpu_bgra_frame_rgba(&frame)",
    ".render_live_gpu_capture_rgba(&gpu_capture)",
    ".hash_pair_with_cpu_bgra(",
    ".render_hash_line(frames)",
    "render_hash_capture.gpu_frame()",
    "render_hash_capture.cgram()",
    "render_hash_capture.raw_scanlines()",
    "render_hash_capture.cgram_color(",
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
    "ModernAssetFrameResources::load_live_gpu_from_env",
    "ModernIndexCompareResources::load_live_gpu_from_env",
    "ModernAssetLiveStats::from_env()",
    "LiveGpuFrameCapture::from_game(",
    "present_modern_asset_live_frame_from_entries(",
    "source_table_from_entries(",
    "render_hd_capture_from_gpu_capture(",
    "render_hd_capture_from_sources(",
    "compare_gpu_render_frame_bgra_to_rgba(",
    "compare_gpu_render_current_frame(",
    ".compare_current_frame(",
    ".compare_current_frame_with_optional_readback(",
    ".render_report_from_game(",
    ".render_output_from_game(",
    ".summary_line_if_quiet(",
    ".summary_line_if_enabled(",
    ".modern_index_summary_line_if_enabled(",
    ".play_summary_line(",
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
    "RendererMode::from_effective_mode(",
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
        for forbidden in FORBIDDEN_REPLAY_FINGERPRINT_RENDER_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "replay fingerprint render escaped play_renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_PLAY_RENDERER_CLASSIC_FRAME_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "classic frame render escaped classic_frame_renderer boundary at "
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
        for forbidden in FORBIDDEN_MAIN_PLAY_RENDER_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "play render escaped play_renderer boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_CLASSIC_RUN_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "classic play-frame run escaped render_diagnostics boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_RENDER_HASH_HELPER_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "render hash helper escaped gpu_capture boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_REPLAY_PLAY_RENDER_HELPER_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "replay play-render helper escaped gpu_capture boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_GPU_COMPARE_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "GPU compare command ownership escaped gpu_compare boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_PLAY_RENDERER_DIAGNOSTIC_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "diagnostic play-render helper escaped render_diagnostics boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_RENDER_DIAGNOSTIC_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "render diagnostic ownership escaped render_diagnostics boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        fn = enclosing_function(lines, index) or "<module>"
        if fn in GPU_ASSET_MAIN_RENDER_FUNCTIONS:
            for forbidden in FORBIDDEN_MAIN_GPU_ASSET_RENDER_CALLS:
                if forbidden in line:
                    errors.append(
                        "default render smoke escaped PNG-backed GPU path at "
                        f"zelda3-bin/src/main.rs:{index + 1} "
                        f"in {fn}: {line.strip()}"
                    )
        if fn in GPU_ASSET_PLAY_LOCKSTEP_FUNCTIONS:
            for forbidden in FORBIDDEN_PLAY_LOCKSTEP_CLASSIC_PRESENT_CALLS:
                if forbidden in line:
                    errors.append(
                        "play-lockstep presentation escaped PNG-backed GPU path at "
                        f"zelda3-bin/src/main.rs:{index + 1} "
                        f"in {fn}: {line.strip()}"
                    )
        for forbidden in GPU_ASSET_LIBRETRO_VIDEO_FUNCTION_FORBIDDEN_FRAMES.get(fn, ()):
            if forbidden in line:
                errors.append(
                    "libretro video comparison escaped PNG-backed GPU path at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        if fn in GPU_ASSET_LOCKSTEP_ARTIFACT_FUNCTIONS:
            for forbidden in FORBIDDEN_LOCKSTEP_ARTIFACT_CLASSIC_RUST_FRAME_CALLS:
                if forbidden in line:
                    errors.append(
                        "lockstep rust frame artifact escaped PNG-backed GPU path at "
                        f"zelda3-bin/src/main.rs:{index + 1} "
                        f"in {fn}: {line.strip()}"
                    )
        for forbidden in FORBIDDEN_MAIN_FRAME_DUMP_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "frame dump command ownership escaped frame_dump_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_ROUTE_COVERAGE_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "route coverage ownership escaped route_coverage_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_PLAY_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "play command ownership escaped play_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_INPUT_SCRIPT_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "input script ownership escaped input_script boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_REPLAY_SAVE_CONFIG_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "replay-save config ownership escaped replay_save_config boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_REPLAY_DIAGNOSTIC_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "replay diagnostic ownership escaped replay_diagnostics boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_AUDIO_TRACE_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "audio trace ownership escaped audio_trace boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        if fn in AUDIO_TRACE_ADVANCE_FUNCTIONS:
            for forbidden in FORBIDDEN_AUDIO_TRACE_CPU_RENDER_CALLS:
                if forbidden in line:
                    errors.append(
                        "audio trace frame advance escaped CPU-free path at "
                        f"zelda3-bin/src/main.rs:{index + 1} "
                        f"in {fn}: {line.strip()}"
                    )
        for forbidden in FORBIDDEN_MAIN_IMAGE_OUTPUT_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "image output ownership escaped image_output boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_HD_AUTHORING_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "HD authoring command ownership escaped hd_authoring_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_ASSET_PALETTE_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "asset palette command ownership escaped asset_palette_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_ASSET_SOURCE_DUMP_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "asset source dump command ownership escaped asset_source_dump_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_DEVELOPER_ROOM_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "developer room command ownership escaped developer_room_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_SHEET_DUMP_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "sheet dump command ownership escaped sheet_dump_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_INDEX_DUMP_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "index dump command ownership escaped index_dump_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_MAIN_OVERWORLD_DUMP_COMMAND_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "overworld dump command ownership escaped overworld_dump_commands boundary at "
                    f"zelda3-bin/src/main.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_play_renderer_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for forbidden in FORBIDDEN_PLAY_RENDERER_CLASSIC_BACKEND_CALLS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "classic CPU backend escaped classic_frame_renderer boundary at "
                    f"zelda3-bin/src/play_renderer.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_frame_dump_commands_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        fn = enclosing_function(lines, index)
        if fn not in GPU_ASSET_FRAME_DUMP_FUNCTIONS:
            continue
        for forbidden in FORBIDDEN_FRAME_DUMP_CLASSIC_DEFAULT_CALLS:
            if forbidden in line:
                errors.append(
                    "default frame dump escaped PNG-backed GPU path at "
                    f"zelda3-bin/src/frame_dump_commands.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_developer_room_commands_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        fn = enclosing_function(lines, index)
        if fn not in GPU_ASSET_DEVELOPER_DESTINATION_FUNCTIONS:
            continue
        for forbidden in FORBIDDEN_DEVELOPER_DESTINATION_CLASSIC_DEFAULT_CALLS:
            if forbidden in line:
                errors.append(
                    "default developer destination dump escaped PNG-backed GPU path at "
                    f"zelda3-bin/src/developer_room_commands.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_classic_frame_renderer_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for forbidden in FORBIDDEN_CLASSIC_FRAME_RENDERER_LIVE_BACKEND:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "live CPU play backend escaped explicit diagnostic boundary at "
                    f"zelda3-bin/src/classic_frame_renderer.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def check_gpu_capture_text(source: str) -> list[str]:
    errors: list[str] = []
    lines = source.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        for forbidden in FORBIDDEN_GPU_CAPTURE_REPLAY_CLASSIC_HELPERS:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "replay classic frame helper escaped render_diagnostics boundary at "
                    f"zelda3-bin/src/gpu_capture.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_GPU_CAPTURE_READBACK_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "GPU readback ownership escaped gpu_readback boundary at "
                    f"zelda3-bin/src/gpu_capture.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
        for forbidden in FORBIDDEN_GPU_CAPTURE_COMPARE_OWNERSHIP:
            if forbidden in line:
                fn = enclosing_function(lines, index) or "<module>"
                errors.append(
                    "GPU compare ownership escaped gpu_compare boundary at "
                    f"zelda3-bin/src/gpu_capture.rs:{index + 1} "
                    f"in {fn}: {line.strip()}"
                )
    return errors


def boundary_source_text() -> str:
    return "\n".join(path.read_text() for path in BOUNDARY_SOURCE_FILES)


def main() -> int:
    source = boundary_source_text()
    errors = check_source_text(source)
    errors.extend(check_main_text(MAIN_RS.read_text()))
    errors.extend(check_frame_dump_commands_text(FRAME_DUMP_COMMANDS_RS.read_text()))
    errors.extend(check_developer_room_commands_text(DEVELOPER_ROOM_COMMANDS_RS.read_text()))
    errors.extend(check_gpu_capture_text(GPU_CAPTURE_RS.read_text()))
    errors.extend(check_play_renderer_text(PLAY_RENDERER_RS.read_text()))
    errors.extend(check_classic_frame_renderer_text(CLASSIC_FRAME_RENDERER_RS.read_text()))
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
