import importlib.util
import pathlib
import sys
import textwrap
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "check_renderer_source_boundaries.py"


def load_module():
    spec = importlib.util.spec_from_file_location("check_renderer_source_boundaries", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def source_with_required_calls(body: str) -> str:
    required = """
    fn run_play_with_state() {
        let assets = renderer::ModernAssetFrameResources::load_from_env();
        frontend.set_renderer_mode(renderer::RendererMode::from_effective_env());
        let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(0);
        let stats = renderer::ModernAssetLiveStats::from_env();
        variant_live_stats.record_present_result(&present_result);
        modern_assets.unhandled_gpu_asset_frame_line();
        let compare = renderer::ModernIndexCompareStats::from_env();
        let compare_config = renderer::ModernIndexCompareRunConfig::default();
        let frame_record = renderer::ModernIndexCompareFrameOutputInput {};
        let compare_resources = compare_config.load_resources_from_env();
        let compare_output = modern_index_compare_stats.render_compare_frame_output_from_entries(frame_record);
        let output_stream = renderer::ModernIndexCompareOutputStream::Stdout;
        let maybe_summary = modern_index_compare_stats.summary_line_if_enabled(true);
        let atlas_compare_resources = renderer::ModernAtlasCompareResources::load();
        let atlas_frame_record = renderer::ModernAtlasCompareFrameInput {};
        let source_table = renderer::source_table_from_entries(&[(0, 0, 0)]);
        let gpu_diff = renderer::compare_gpu_render_frame_bgra_to_rgba(0, &[], &[]);
        let render_hash = renderer::render_hash_frame_bgra(0, &[]);
        let gpu_render_hash = renderer::gpu_render_hash_frame_rgba(0, &[]);
        let render_hash_pair = renderer::render_hash_pair_bgra_rgba(&[], &[]);
        let fingerprint_leaf = renderer::render_fingerprint_leaf_bgra(&[]);
        let frame = renderer::GpuFrame::from_source_and_raw_scanlines();
        frontend.present_modern_asset_frame();
        report.failure_line();
        atlas_compare_resources.compare_frame(atlas_frame_record);
        renderer::hd_authoring::render_hd_capture_from_sources();
    }
    """
    return textwrap.dedent(required + body)


class RendererSourceBoundaryTests(unittest.TestCase):
    def test_allows_renderer_owned_source_render_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_dump_hd_capture() {
                renderer::hd_authoring::render_hd_capture_from_sources();
            }
            """
        )

        self.assertEqual(module.check_source_text(source), [])

    def test_rejects_hd_authoring_manual_extraction(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_dump_hd_capture() {
                use renderer::hd_authoring::build_hd_placement_map;
                renderer::modern_extract::extract_modern_frame_from_sources();
                let _map = build_hd_placement_map();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("run_dump_hd_capture", errors[0])

    def test_rejects_trace_path_manual_extraction(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_replay_save() {
                if trace_pixel_enabled {
                    renderer::modern_extract::extract_modern_frame_from_sources();
                    renderer::modern_variant_draw::trace_variant_plan_pixel();
                }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("run_replay_save", errors[0])

    def test_rejects_modern_index_trace_env_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_replay_save() {
                let _ = "ZELDA3_VARIANT_TRACE_PIXEL";
                variant_trace_pixel_env();
                parse_variant_trace_pixel("1:2:3");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all("modern index compare stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_trace_format_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_replay_save() {
                print_variant_pixel_traces(frame, x, y, &rendered.variant_traces);
                if let Some(trace_pixel) = rendered.trace_pixel {
                    eprintln!("variant_pixel_trace frame={frame}");
                }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 4)
        self.assertTrue(
            all("modern index compare stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_rendered_report_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_replay_save() {
                for line in &rendered.trace_lines {}
                let report = rendered.report;
                rendered.output_lines();
                modern_index_compare_stats.render_compare_frame(frame_record);
                modern_index_compare_stats.render_compare_frame_output(frame_record);
                let compare_scene = renderer::ModernIndexCompareScene::from_main_module_and_player_indoors_flag(9, 0);
                if let Some(line) = report.frame_line() {}
                if let Some(line) = report.progress_line() {}
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all("modern index compare stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_run_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                if modern_index_compare != 0 && frames % modern_index_compare == 0 {}
                if modern_index_compare != 0 && completed_frame % modern_index_compare == 0 {}
                if require_full_gpu_path && modern_index_compare == 0 {}
                if require_modern_index_parity && modern_index_compare == 0 {}
                modern_index_compare.require_full_gpu_path();
                modern_index_compare.require_modern_index_parity();
                let _input = ModernIndexCompareFrameRenderInput {
                    require_full_gpu_path: true,
                    require_modern_index_parity: true,
                };
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 11)
        self.assertTrue(
            all("modern index compare stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_default_path_manual_extraction(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::modern_extract::extract_modern_frame_from_sources();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("run_play_with_state", errors[0])

    def test_rejects_low_level_source_render_call(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::modern_extract::render_modern_frame_full_scaled_from_sources();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("low-level source render escaped renderer boundary", errors[0])
        self.assertIn("run_play_with_state", errors[0])

    def test_rejects_modern_atlas_compare_render_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::modern_extract::extract_modern_frame_with_atlas();
                renderer::modern_software::render_modern_frame_software();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all("modern atlas compare render escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_live_vram_extraction(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::modern_extract::extract_modern_frame_from_vram();
                renderer::modern_extract::extract_modern_sprites_from_vram();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all("manual VRAM extraction escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_granular_live_present_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                frontend.present_modern_variant_gpu_from_sources();
                frontend.present_modern_gpu_from_sources();
                frontend.present_modern_gpu_from_vram();
                frontend.present_modern_frame_from_sources();
                frontend.present_modern_mode7_gpu();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 5)
        self.assertTrue(
            all("granular live present escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_asset_loading_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::source_atlas_renderer_mode("assets-anim-gpu");
                renderer::variant_atlas_renderer_mode("assets-variant-gpu");
                effective_renderer_mode_from_env_value(renderer_env.as_deref());
                renderer::EffectiveRendererMode::from_env_value(renderer_env.as_deref(), variant_env.as_deref());
                let _ = "ZELDA3_VARIANT_ATLAS";
                renderer::ModernAssetFrameResources::load_for_mode(mode, root);
                renderer::ModernIndexCompareResources::load_for_mode(enabled, mode, root, true);
                renderer::modern_assets::load_modern_overworld_tile_atlas(root);
                let renderer_env = renderer::EffectiveRendererMode::from_env();
                renderer_env.uses_source_atlas();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 10)
        self.assertTrue(
            all("modern asset loading policy escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_hd_override_loading_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                renderer::modern_hd_overrides::ModernHdOverrides::from_env();
                renderer::modern_hd_overrides::HdOverrideCtx::new(&store);
                renderer::modern_hd_overrides::HdOverrideCtx::disabled();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all("HD override loading policy escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_live_stats_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            struct VariantLiveStats {}
            fn env_flag_default_true(value: Option<&str>) -> bool { true }
            fn run_play_with_state() {
                let _ = "ZELDA3_VARIANT_LIVE_STATS";
                let _ = "ZELDA3_REQUIRE_FULL_GPU_PATH";
                variant_live_stats.record_variant_stats(stats);
                self.modern_assets.gpu_asset_mode();
                report.full_gpu_failure_line;
                eprintln!("gpu_path_unsupported_live reason={} count={}", reason, count);
                eprintln!("modern asset renderer did not handle a GPU asset frame");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 9)
        self.assertTrue(
            all("live modern asset stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_compare_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let modern_index_compare_count = 0;
                let modern_index_compare_bad_count = 0;
                let modern_index_compare_variant_draws = 0;
                let _ = "ZELDA3_MODERN_INDEX_COMPARE_SUMMARY";
                let _ = "ZELDA3_MODERN_INDEX_COMPARE_PROGRESS";
                renderer::modern_gpu::modern_gpu_path_fallback_reason("variant-gpu", None);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all("modern index compare stats policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_frame_report_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                modern_index_compare_stats.record(via, mismatch, variant_stats.as_ref());
                modern_index_compare_stats.full_gpu_fallback(via, variant_stats.as_ref());
                modern_index_compare_stats.should_print_frame(mismatch);
                modern_index_compare_stats.frame_line(renderer::ModernIndexCompareFrameLine {});
                modern_index_compare_stats.progress_line(frames);
                modern_index_compare_stats.record_frame(frame_record);
                modern_index_compare_stats.summary_enabled();
                modern_index_compare_stats.summary_line();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 8)
        self.assertTrue(
            all("modern index frame report policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_dump_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let dump = "ZELDA3_MODERN_INDEX_DUMP_FRAME";
                let classic_path = format!("/tmp/classic_{frame}.png");
                let modern_path = format!("/tmp/modern_index_{frame}.png");
                println!("dumped classic frame to {classic_path}");
                println!("dumped modern_index frame to {modern_path}");
                let paths = renderer::ModernIndexCompareDumpPaths {};
                modern_index_compare_stats.dump_paths_for_frame(frame);
                modern_index_compare_stats.write_dump_for_frame(frame, &classic, &modern);
                write_modern_index_compare_dump(&paths, &classic, &modern);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 9)
        self.assertTrue(
            all("modern index dump policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_index_resource_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let atlas_gpu_compare = mode.name() == "assets-anim-gpu";
                let variant_gpu_compare = mode.uses_variant_atlas();
                let modern_gpu_headless = Some(renderer::ModernGpuHeadless::new());
                let modern_variant_headless = variant_atlas.as_ref().map(renderer::ModernGpuVariantHeadless::new);
                let variant_atlas = if compare_enabled { None } else { None };
                let source_atlas = if compare_enabled { None } else { None };
                renderer::modern_index_atlas::load_modern_overworld_index_atlas(root);
                renderer::modern_dungeon_atlas::load_modern_dungeon_index_atlas(root);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all("modern index resource policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_modern_scene_policy_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let scene = renderer::ModernAssetFrameScene::from_in_dungeon(game.ram[PLAYER_IS_INDOORS] != 0);
                let mode_str: Option<String> = match module {
                    9 | 11 => Some("ow".to_string()),
                    7 | 16 => Some("dungeon".to_string()),
                    m => Some(format!("mod{m}")),
                };
                let mode_label = match module {
                    9 | 11 => "ow".to_string(),
                    7 | 16 => "dungeon".to_string(),
                    m => format!("mod{m}"),
                };
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all("modern scene policy escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_source_table_view_adapter_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            struct VramChrSourceTableView {}
            impl renderer::modern_extract::SourceTableView for VramChrSourceTableView {
                fn get(&self, slot: usize) -> (u8, u16, u16) { (0, 0, 0) }
            }
            fn vram_chr_source_table_view() {}
            fn logical_chr_src_tuple() {}
            fn run_play_with_state() {
                renderer::MappedSourceTableView::new(table.as_slice(), logical_chr_src_tuple);
                renderer::MappedSourceTableView::from_entries(table.as_slice());
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all("source table view adapter escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_frame_compare_helper_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            struct GpuRenderDiff {}
            fn compare_bgra_to_rgba(cpu_bgra: &[u8], gpu_rgba: &[u8]) {}
            fn compare_rgba_to_rgba(classic_rgba: &[u8], modern_rgba: &[u8]) {}
            fn render_frame_rgb_hash_bgra(frame: &[u8]) {}
            fn render_frame_rgb_hash_rgba(frame: &[u8]) {}
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 5)
        self.assertTrue(
            all("frame compare helper escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_direct_modern_index_compare_diff_assembly_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let diff = renderer::compare_rgba_to_rgba(&classic, &modern);
                let index_diff = renderer::compare_modern_index_rgba(&classic, &modern);
                let modern = renderer::modern_gpu::render_modern_index_compare_frame();
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 3)
        self.assertIn("modern index compare diff assembly escaped renderer boundary", errors[0])

    def test_rejects_direct_modern_atlas_compare_execution_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                if let Some(atlas) = modern_atlas_compare_resources.atlas() {
                    let compare = renderer::modern_gpu::compare_modern_atlas_to_rgba(&classic, &frame, atlas);
                }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all("modern atlas compare execution escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_direct_gpu_render_compare_diff_assembly_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let diff = renderer::compare_bgra_to_rgba(&cpu, &gpu);
                let comparison = renderer::compare_gpu_render_bgra_to_rgba(&cpu, &gpu);
                eprintln!("gpu-render-divergence frame={frames}");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all("gpu render compare diff assembly escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_render_hash_report_assembly_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                println!("render-hash frame={frames} hash=0x12345678");
                println!("gpu-render-hash frame={frames} hash=0x12345678");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all("render hash report escaped renderer boundary" in error for error in errors)
        )

    def test_rejects_raw_render_hash_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let cpu_hash = renderer::render_frame_rgb_hash_bgra(&classic_bgra);
                let gpu_hash = renderer::render_frame_rgb_hash_rgba(&modern_rgba);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all("raw render hash escaped renderer boundary" in error for error in errors)
        )
        self.assertTrue(all("run_play_with_state" in error for error in errors))

    def test_rejects_gpu_frame_assembly_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let frame = renderer::GpuFrame {
                    vram: &[],
                };
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("gpu frame assembly escaped renderer boundary", errors[0])

    def test_rejects_gpu_scanline_capture_conversion_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let scanlines = renderer::scanlines_from_raw(&scanlines_raw);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("gpu scanline capture conversion escaped renderer boundary", errors[0])

    def test_rejects_missing_renderer_owned_api_call(self):
        module = load_module()
        source = "fn run_play_with_state() {}"

        errors = module.check_source_text(source)

        self.assertIn(
            "missing renderer-owned source API call: present_modern_asset_frame",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
