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
        let live_present_input = renderer::ModernAssetFrameLivePresentInput {};
        frontend.present_modern_asset_live_frame_from_entries(live_present_input);
        frontend.set_renderer_mode(renderer::RendererMode::from_effective_env());
        let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(0);
        let stats = renderer::ModernAssetLiveStats::from_env();
        let compare = renderer::ModernIndexCompareStats::from_env();
        let compare_config = renderer::ModernIndexCompareRunConfig::default();
        let frame_record = renderer::ModernIndexCompareFrameOutputInput {};
        let compare_resources = compare_config.load_resources_from_env();
        let compare_output = modern_index_compare_stats.render_compare_frame_output_from_entries(frame_record);
        let output_stream = renderer::ModernIndexCompareOutputStream::Stdout;
        let maybe_summary = modern_index_compare_stats.summary_line_if_enabled(true);
        let atlas_compare_resources = renderer::ModernAtlasCompareResources::load();
        let atlas_compare = atlas_compare_resources.compare_frame_rgba(0, &gpu_frame, &classic_rgba);
        let source_table = renderer::source_table_from_entries(&[(0, 0, 0)]);
        let gpu_diff = renderer::compare_gpu_render_frame_bgra_to_rgba(0, &[], &[]);
        let render_hash = renderer::render_hash_frame_bgra(0, &[]);
        let gpu_render_hash = renderer::gpu_render_hash_frame_rgba(0, &[]);
        let render_hash_pair = renderer::render_hash_pair_bgra_rgba(&[], &[]);
        let fingerprint_leaf = renderer::render_fingerprint_leaf_bgra(&[]);
        let frame_input = renderer::GpuFrameCaptureInput {};
        let frame_snapshot = renderer::GpuFrameRegisterSnapshot {};
        let frame = renderer::GpuFrame::from_capture_input(frame_input);
        report.failure_line();
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
            fn live_gpu_backend_site() {
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
                frontend.present_modern_asset_frame();
                frontend.present_gpu_frame_with_context(&gpu_frame, presentation);
                renderer::GpuFrame::from_source_and_raw_scanlines(source, cgram, scanlines);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 8)
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
                variant_live_stats.record_present_result(&present_result);
                variant_live_stats.record_present_output(&present, &modern_assets);
                self.modern_assets.gpu_asset_mode();
                self.modern_assets.unhandled_gpu_asset_frame_line();
                report.full_gpu_failure_line;
                report.fallback_presentation_context();
                if present.result.is_presented() { return; }
                let presentation = renderer::PresentationContext { in_dungeon: present.in_dungeon };
                eprintln!("gpu_path_unsupported_live reason={} count={}", reason, count);
                eprintln!("modern asset renderer did not handle a GPU asset frame");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 15)
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
                let input = renderer::ModernAtlasCompareFrameInput {};
                modern_atlas_compare_resources.compare_frame(input);
                if let Some(atlas) = modern_atlas_compare_resources.atlas() {
                    let compare = renderer::modern_gpu::compare_modern_atlas_to_rgba(&classic, &frame, atlas);
                }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 4)
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
                let direct_diff = report.comparison.diff;
                let line = report.divergence_line.as_deref();
                eprintln!("gpu-render-divergence frame={frames}");
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 5)
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

    def test_rejects_replay_fingerprint_render_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_replay_save() {
                render_standard_play_frame_bgra(&mut game, frame);
                let fp_render_leaf = render_fingerprint_leaf_bgra(frame);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all(
                "replay fingerprint render escaped play_renderer boundary" in error
                for error in errors
            )
        )
        self.assertTrue(all("run_replay_save" in error for error in errors))

    def test_rejects_play_renderer_classic_frame_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn render_replay_dump_frame_rgba() {
                crate::play_renderer::render_play_frame_bgra(&mut game, frame, pitch, PpuRenderFlags::empty());
                crate::play_renderer::render_standard_play_frame_bgra(&mut game, frame);
                crate::play_renderer::run_play_frame_bgra(&mut game, input, frame, PpuRenderFlags::empty());
                crate::play_renderer::run_play_frame_with_run_what_bgra(&mut game, input, run_what, frame, PpuRenderFlags::empty());
            }
            """
        )

        errors = module.check_source_text(source)

        classic_errors = [
            error
            for error in errors
            if "classic frame render escaped classic_frame_renderer boundary" in error
        ]
        self.assertEqual(len(classic_errors), 4)
        self.assertTrue(
            any("replay fingerprint render escaped play_renderer boundary" in error for error in errors)
        )

    def test_rejects_classic_backend_in_play_renderer(self):
        module = load_module()
        source = """
            struct CpuPlayRenderer;

            impl PlayRendererBackend for CpuPlayRenderer {
                fn present_frame(&mut self) {
                    render_play_frame_bgra(game, frame, 256 * 4, render_flags);
                }
            }
            """

        errors = module.check_play_renderer_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all(
                "classic CPU backend escaped classic_frame_renderer boundary" in error
                for error in errors
            )
        )

    def test_rejects_main_play_render_calls(self):
        module = load_module()
        source = """
            fn write_lockstep_parity_failure_artifacts() {
                render_play_frame_bgra(&mut rust_state, &mut rust_frame, pitch, PpuRenderFlags::empty());
            }

            fn dump_overworld_screen_site() {
                render_play_frame_bgra(&mut game, &mut frame, pitch, PpuRenderFlags::empty());
            }

            fn compare_oracle_render_site() {
                render_play_frame_bgra(&mut game_state, game_frame, pitch, PpuRenderFlags::empty());
            }

            fn render_lockstep_frames_in_place() {
                render_play_frame_bgra(&mut oracle.game, game_frame, pitch, PpuRenderFlags::empty());
            }
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 4)
        self.assertTrue(
            all(
                "play render escaped play_renderer boundary" in error
                for error in errors
            )
        )

    def test_rejects_main_classic_run_calls(self):
        module = load_module()
        source = """
            fn run_smoke_render() {
                run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
                run_play_frame_with_run_what_bgra(&mut game, input, run_what, &mut frame, render_flags);
            }
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all(
                "classic play-frame run escaped render_diagnostics boundary" in error
                for error in errors
            )
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

    def test_rejects_main_render_hash_helper_calls(self):
        module = load_module()
        source = """
            fn run_replay_save() {
                println!("{}", render_hash_frame_bgra_line(frames, frame));
            }
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 1)
        self.assertIn("render hash helper escaped gpu_capture boundary", errors[0])
        self.assertIn("run_replay_save", errors[0])

    def test_rejects_main_replay_play_render_helper_calls(self):
        module = load_module()
        source = """
            fn run_replay_save() {
                render_replay_projection_bgra(&mut game, &mut scratch);
                let fp_render_leaf = render_replay_fingerprint_leaf_bgra(&mut game, frame);
            }
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all(
                "replay play-render helper escaped gpu_capture boundary" in error
                for error in errors
            )
        )
        self.assertTrue(all("run_replay_save" in error for error in errors))

    def test_rejects_gpu_compare_command_ownership_in_main(self):
        module = load_module()
        source = """
            fn run_play_gpu_render_compare(args: &[String]) {}
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 1)
        self.assertIn("GPU compare command ownership escaped gpu_compare boundary", errors[0])

    def test_rejects_replay_classic_helpers_in_gpu_capture(self):
        module = load_module()
        source = """
            pub(crate) fn replay_projection_bgra(game: &mut ZeldaState, frame: &mut [u8]) {}

            pub(crate) fn replay_fingerprint_leaf_bgra(game: &mut ZeldaState, frame: &mut [u8]) -> u32 {
                0
            }
            """

        errors = module.check_gpu_capture_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 2)
        self.assertTrue(
            all(
                "replay classic frame helper escaped render_diagnostics boundary" in error
                for error in errors
            )
        )

    def test_rejects_readback_ownership_in_gpu_capture(self):
        module = load_module()
        source = """
            pub(crate) struct GpuReadbackRenderer;
            pub(crate) struct GpuRgbaReadbackFrame;
            pub(crate) struct OptionalGpuReadbackRenderer;
            pub(crate) struct ReplayRenderHashCapture;
            pub(crate) struct ReplayRenderHashGpuReadback;

            impl GpuReadbackRenderer {}
            impl OptionalGpuReadbackRenderer {}
            impl ReplayRenderHashCapture {}
            impl ReplayRenderHashGpuReadback {}
            """

        errors = module.check_gpu_capture_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 9)
        self.assertTrue(
            all(
                "GPU readback ownership escaped gpu_readback boundary" in error
                for error in errors
            )
        )

    def test_rejects_compare_ownership_in_gpu_capture(self):
        module = load_module()
        source = """
            pub(crate) struct ModernCompareModeDefaults;
            pub(crate) struct ModernIndexCompareRun;
            pub(crate) struct GpuRenderCompareRun;
            struct ModernAtlasCompareRun;
            pub(crate) struct PlayGpuRenderCompareSession;

            pub(crate) fn modern_compare_mode_defaults_from_env() {}
            pub(crate) fn modern_index_compare_run_from_env() {}
            pub(crate) fn gpu_render_compare_run() {}
            pub(crate) fn play_gpu_render_compare_session() {}
            fn compare_gpu_render_current_frame() {}
            fn emit_modern_index_compare_output_lines() {}
            fn cgram_match() {}
            """

        errors = module.check_gpu_capture_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 12)
        self.assertTrue(
            all(
                "GPU compare ownership escaped gpu_compare boundary" in error
                for error in errors
            )
        )

    def test_rejects_main_play_renderer_diagnostic_calls(self):
        module = load_module()
        source = """
            fn write_lockstep_parity_failure_artifacts() {
                render_lockstep_artifact_frame_bgra(&mut rust_state, &mut rust_frame);
            }

            fn dump_overworld_screen_site() {
                render_overworld_screen_dump_bgra(&mut game, &mut frame);
            }

            fn compare_oracle_render_site() {
                render_oracle_compare_frames_bgra(oracle, game_frame, snes_frame, pitch);
            }

            fn run_play_lockstep() {
                render_lockstep_oracle_frames_in_place(&mut oracle, &mut game_frame, &mut snes_frame, pitch);
            }
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 4)
        self.assertTrue(
            all(
                "diagnostic play-render helper escaped render_diagnostics boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_render_diagnostic_ownership_in_main(self):
        module = load_module()
        source = """
            struct RenderDiff {}
            fn compare_oracle_render_frame() {}
            fn format_render_ppu_summary() {}
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all(
                "render diagnostic ownership escaped render_diagnostics boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_frame_dump_command_ownership_in_main(self):
        module = load_module()
        source = """
            fn run_dump_frame() {}
            fn run_dump_overworld_screen() {}
            fn run_scan_replay_checkpoints() {}
            fn run_dump_replay_checkpoint_ppu() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 4)
        self.assertTrue(
            all(
                "frame dump command ownership escaped frame_dump_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_route_coverage_ownership_in_main(self):
        module = load_module()
        source = """
            fn route_coverage_frame_from_game() {}
            fn write_route_coverage_log_or_exit() {}
            fn run_coverage_probe() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all(
                "route coverage ownership escaped route_coverage_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_play_command_ownership_in_main(self):
        module = load_module()
        source = """
            fn run_frontend_smoke() {}
            fn run_play() {}
            fn run_standalone_play() {}
            fn run_play_with_state() {}
            fn apply_host_menu_action_for_test() {}
            mod host_menu_play_tests {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all(
                "play command ownership escaped play_commands boundary" in error
                for error in errors
            )
        )

    def test_rejects_image_output_ownership_in_main(self):
        module = load_module()
        source = """
            const ASSETS_PNG_COLUMNS: usize = 128;
            fn write_argb_frame_png() {}
            fn write_assets_index_png() {}
            fn write_rgba_frame_png() {}
            fn decode_rgba_png() {}
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 5)
        self.assertTrue(
            all(
                "image output ownership escaped image_output boundary" in error
                for error in errors
            )
        )

    def test_rejects_hd_authoring_command_ownership_in_main(self):
        module = load_module()
        source = """
            fn write_reference_palette_png() {}
            fn run_dump_hd_capture() {}
            fn run_slice_hd_cells() {}
            """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all(
                "HD authoring command ownership escaped hd_authoring_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_asset_palette_command_ownership_in_main(self):
        module = load_module()
        source = """
            fn run_dump_reference_palette() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 1)
        self.assertIn(
            "asset palette command ownership escaped asset_palette_commands boundary",
            errors[0],
        )

    def test_rejects_asset_source_dump_command_ownership_in_main(self):
        module = load_module()
        source = """
            struct AssetsBySourceManifest {}
            struct AssetsBySourceCell {}
            struct PaletteUsageKey {}
            struct PaletteUsageManifest {}
            struct PaletteUsageEntry {}
            fn palette_usage_key_from_chr_source() {}
            fn record_palette_usage_count() {}
            fn palette_usage_entries_from_counts() {}
            mod palette_usage_tests {}
            fn run_dump_assets_by_source() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 10)
        self.assertTrue(
            all(
                "asset source dump command ownership escaped asset_source_dump_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_developer_room_command_ownership_in_main(self):
        module = load_module()
        source = """
            struct DeveloperSandboxTilemapManifest {}
            struct DeveloperTilesetManifest {}
            struct DeveloperTilesetEntry {}
            fn current_developer_location_from_ram() {}
            fn load_developer_route_bookmark() {}
            fn load_developer_destination() {}
            fn developer_sandbox_tilemap_manifest() {}
            fn developer_kakariko_tileset_manifest() {}
            fn load_developer_synthetic_room() {}
            fn load_developer_room_theme_source() {}
            fn write_developer_room_visuals_to_ppu() {}
            fn write_developer_room_palette_from_source() {}
            fn copy_developer_room_chr_from_source() {}
            fn developer_room_kakariko_sample_origin() {}
            fn developer_room_kakariko_visible_cell() {}
            fn run_dump_developer_destination() {}
            fn run_dump_developer_tileset() {}
            const DEVELOPER_ROOM_BG_TILE_BASE: u16 = 0x2000;
            const DEVELOPER_ROOM_SOURCE_BG_LAYER: usize = 1;
            const DEV_TOWN_ROOF: u16 = 224;
            const DEV_TOWN_WALL: u16 = 225;
            const DEV_TOWN_DOOR: u16 = 226;
            const DEV_TOWN_GRASS: u16 = 227;
            const DEV_TOWN_PATH: u16 = 228;
            const DEV_TOWN_FENCE: u16 = 229;
            const DEV_TOWN_SHRUB: u16 = 230;
            const DEV_TOWN_SIGN: u16 = 231;
            const DEV_TOWN_TREE: u16 = 232;
            const DEV_TOWN_CLIFF_TOP: u16 = 233;
            const DEV_TOWN_CLIFF_FACE: u16 = 234;
            const DEV_TOWN_FLOWERS: u16 = 235;
            const DEV_TOWN_STONE: u16 = 236;
            const DEV_TOWN_HEDGE: u16 = 237;
            const DEVELOPER_ROOM_KAKARIKO_MUSIC: u8 = 0x07;
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 34)
        self.assertTrue(
            all(
                "developer room command ownership escaped developer_room_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_sheet_dump_command_ownership_in_main(self):
        module = load_module()
        source = """
            struct SpriteSheetPngManifest {}
            struct SpriteSheetPngCell {}
            fn run_dump_sprite_sheet_png() {}
            struct DungeonSheetPngManifest {}
            struct DungeonSheetPngCell {}
            fn run_dump_dungeon_sheet_png() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 6)
        self.assertTrue(
            all(
                "sheet dump command ownership escaped sheet_dump_commands boundary" in error
                for error in errors
            )
        )

    def test_rejects_index_dump_command_ownership_in_main(self):
        module = load_module()
        source = """
            struct DungeonIndexTileAtlasManifest {}
            struct DungeonIndexTileCellManifest {}
            struct SpriteIndexTileAtlasManifest {}
            struct SpriteIndexTileCellManifest {}
            struct SpriteIndexKey {}
            fn run_dump_dungeon_index_tiles() {}
            fn run_dump_sprite_index_tiles() {}
            fn dungeon_room_index_probe() {}
            const OBJ_SIZE_TABLE: [[u32; 2]; 8] = [[8, 16]; 8];
            struct SpriteTileProbe {}
            fn sprite_index_probe() {}
            fn decode_snes_4bpp_tile_indices() {}
            const DUNGEON_BG_CHR_BASE: usize = 0x2000;
            const DUNGEON_BG1_TILEMAP_WORDS: usize = 0x1000;
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 14)
        self.assertTrue(
            all(
                "index dump command ownership escaped index_dump_commands boundary" in error
                for error in errors
            )
        )

    def test_rejects_overworld_dump_command_ownership_in_main(self):
        module = load_module()
        source = """
            struct UniqueOverworldCellAtlasManifest {}
            struct UniqueOverworldCellManifestEntry {}
            struct UniqueOverworldCellSource {}
            struct UniqueOverworldCell {}
            struct UniqueOverworldCellCollector {}
            impl UniqueOverworldCellCollector {}
            struct UniqueOverworldTileAtlasManifest {}
            struct UniqueOverworldTileManifestEntry {}
            struct DecodedTilemapEntry {}
            struct UniqueOverworldTile {}
            struct UniqueOverworldTileCollector {}
            impl UniqueOverworldTileCollector {}
            struct OverworldIndexTile {}
            struct OverworldIndexTileCollector {}
            impl OverworldIndexTileCollector {}
            struct OverworldIndexTileAtlasManifest {}
            struct OverworldIndexTileCellManifest {}
            fn decode_tilemap_entry() {}
            fn collect_unique_overworld_cells_from_built_bg2_map() {}
            fn collect_unique_overworld_tiles_from_built_bg2_map() {}
            fn render_snes_4bpp_cell_to_rgba() {}
            fn render_snes_4bpp_tile_to_rgba() {}
            fn render_unique_overworld_cell_atlas() {}
            fn render_unique_overworld_tile_atlas() {}
            fn blit_scaled_rgba_cell() {}
            fn fnv32_bytes() {}
            fn run_dump_unique_overworld_cells() {}
            fn run_dump_unique_overworld_tiles() {}
            const OVERWORLD_BG_CHR_BASE: usize = 0x2000;
            const UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT: usize = 32;
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 30)
        self.assertTrue(
            all(
                "overworld dump command ownership escaped overworld_dump_commands boundary"
                in error
                for error in errors
            )
        )

    def test_rejects_input_script_ownership_in_main(self):
        module = load_module()
        source = """
            struct InputScript {}
            impl InputScript {}
            struct InputRule {}
            fn parse_frame_spec() {}
            fn parse_buttons() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 5)
        self.assertTrue(
            all(
                "input script ownership escaped input_script boundary" in error
                for error in errors
            )
        )

    def test_rejects_replay_save_config_ownership_in_main(self):
        module = load_module()
        source = '''
            struct ReplaySaveConfig {}
            fn parse_replay_save_args_or_exit() {}
            const USAGE: &str = "usage: zelda3 --replay-save <path-to-rom.sfc>";
        '''

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 3)
        self.assertTrue(
            all(
                "replay-save config ownership escaped replay_save_config boundary" in error
                for error in errors
            )
        )

    def test_rejects_replay_diagnostic_ownership_in_main(self):
        module = load_module()
        source = """
            fn replay_sram_checksum_ok() {}
            fn replay_checksum_bytes() {}
            fn replay_checksum_ram_range() {}
            fn replay_save_sprite_dump() {}
            fn replay_save_palette_dump() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 5)
        self.assertTrue(
            all(
                "replay diagnostic ownership escaped replay_diagnostics boundary" in error
                for error in errors
            )
        )

    def test_rejects_audio_trace_ownership_in_main(self):
        module = load_module()
        source = """
            struct AudioFrameStats {}
            fn print_replay_audio_trace() {}
            fn replay_dsp_write_events_json() {}
            fn replay_checksum_samples() {}
            fn replay_checksum_dsp_writes() {}
            fn replay_checksum_dsp_write_values() {}
            fn fingerprint_audio_hash() {}
            fn should_write_fingerprint() {}
            fn first_peak_frame() {}
            fn max_peak_frame() {}
            fn print_audio_window() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 11)
        self.assertTrue(
            all(
                "audio trace ownership escaped audio_trace boundary" in error
                for error in errors
            )
        )

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
            "missing renderer-owned source API call: present_modern_asset_live_frame_from_entries",
            errors,
        )

    def test_rejects_live_gpu_backend_ownership_in_main(self):
        module = load_module()
        source = """
            fn play_renderer_from_env() {
                let backend_env = std::env::var("ZELDA3_RENDER_BACKEND");
                let assets = renderer::ModernAssetFrameResources::load_from_env(Path::new("."));
                let report = frontend.present_modern_asset_live_frame_from_entries(input);
                frontend.set_renderer_mode(renderer::RendererMode::from_effective_env());
            }
            struct CpuPlayRenderer;
            fn draw_play_ppu_frame() {}
            fn live_gpu_backend_site() {
                draw_play_ppu_frame(&mut game, &mut frame, 256 * 4, render_flags);
                gpu_frame_from_ppu(&game.ppu, &cgram, &scanlines);
                renderer::ModernIndexCompareFrameOutputInput {
                    frame,
                    gpu_frame: &gpu_frame,
                    classic_rgba: &classic_rgba,
                };
                modern_index_compare_stats.render_compare_frame_output_from_entries(frame_record);
                fn emit(output: &renderer::ModernIndexCompareOutputLines) {}
                emit_modern_index_compare_output_lines(&output_lines);
                if output_lines.has_failure {}
                let stream = renderer::ModernIndexCompareOutputStream::Stdout;
                modern_atlas_compare_resources.compare_frame_rgba(frame, &gpu_frame, &classic_rgba);
                modern_atlas_compare.render_report_from_capture(&gpu_capture, &classic_rgba, frame);
                modern_index_compare.render_output_from_capture(&gpu_capture, &classic_rgba, frame, true);
                renderer::ModernAtlasCompareResources::load(enabled, root);
                let _atlas_resources = load_modern_atlas_compare_resources(enabled, root);
                let _atlas_report = render_modern_atlas_compare_report_from_capture();
                let modern_atlas_compare = modern_atlas_compare_run(modern_render_compare, Path::new("."));
                let _offscreen: renderer::OffscreenRenderer;
                let mut offscreen = pollster::block_on(OffscreenRenderer::new(256, 224));
                offscreen.upload_bgra_frame(frame);
                offscreen.render_to_rgba();
                let mut gpu_readback = if render_hash_log != 0 { Some(new_gpu_readback_renderer(256, 224)) } else { None };
                let mut gpu_readback = new_gpu_readback_renderer(256, 224);
                let mut gpu_readback = new_gpu_readback_renderer(width, height);
                let mut gpu_readback = optional_gpu_readback_renderer(required, 256, 224);
                let mut render_frame = vec![0u8; 256 * 224 * 4];
                let gpu_capture = capture_gpu_frame_from_game(&mut game);
                let gpu_readback = gpu_readback.as_mut().expect("GPU readback renderer allocated");
                gpu_readback.required();
                let _hash_rgba = gpu_readback.render_cpu_bgra_frame_rgba(frame);
                let _cpu_rgba = gpu_readback.render_bgra_frame_to_rgba(frame);
                let _gpu_rgba = gpu_readback.render_gpu_capture_rgba(&gpu_capture);
                let _dump_rgba = gpu_readback.render_cpu_bgra_frame_rgba(&frame);
                let _live_rgba = gpu_readback.render_live_gpu_capture_rgba(&gpu_capture);
                let _hashes = gpu_rgba.hash_pair_with_cpu_bgra(frame);
                println!("{}", gpu_rgba.render_hash_line(frames));
                let _debug_frame = render_hash_capture.gpu_frame();
                let _debug_cgram = render_hash_capture.cgram();
                let _debug_scanlines = render_hash_capture.raw_scanlines();
                let _debug_color = render_hash_capture.cgram_color(7);
                modern_index_compare.load_resources_from_env(root, false);
                let (mut renderer, mut frontend) = play_renderer::configured_from_env();
                let mut renderer = play_renderer::from_env();
                renderer.configure_frontend(&mut frontend);
                renderer.frontend().quit_requested();
                renderer.frontend_mut().poll_input();
                renderer.present_frame(&mut game, &mut frontend, &mut frame, render_flags);
                let renderer_mode = renderer::RendererMode::parse(std::env::var("ZELDA3_RENDERER").ok().as_deref());
                let _modern_compare = renderer::RendererMode::ModernCompare;
                if renderer_mode == renderer::RendererMode::Modern {}
                let source_table = renderer::source_table_from_entries(gpu_capture.source_entries());
                render_hd_capture_from_gpu_capture(&gpu_capture, &atlas);
                renderer::hd_authoring::render_hd_capture_from_sources(&gpu_frame, &source_table, &atlas);
                renderer::compare_gpu_render_frame_bgra_to_rgba(frames, frame, &gpu_rgba);
                gpu_render_compare.compare_current_frame(&mut game, &mut gpu_readback, &mut render_frame, completed_frame);
                gpu_render_compare.compare_current_frame_with_optional_readback(&mut game, &mut gpu_readback, &mut render_frame, completed_frame);
                modern_atlas_compare.render_report_from_game(&mut game, &mut gpu_readback, completed_frame);
                modern_index_compare.render_output_from_game(&mut game, &mut gpu_readback, completed_frame, true);
                gpu_render_compare.summary_line_if_quiet();
                modern_index_compare.summary_line_if_enabled();
                compare_session.modern_index_summary_line_if_enabled();
                compare_session.play_summary_line(start_frame);
                if gpu_render_compare != 0 && frames % gpu_render_compare == 0 {}
                gpu_render_compare_count = gpu_render_compare_count.wrapping_add(1);
                gpu_render_compare_last_frame = frames;
                gpu_render_compare_last_hash = cpu_hash;
                eprintln!("gpu-render-state frame={frames}");
                println!("{}", renderer::render_hash_frame_bgra(frames, frame).line);
                println!("{}", renderer::gpu_render_hash_frame_rgba(frames, &gpu_rgba).line);
                let _hashes = renderer::render_hash_pair_bgra_rgba(frame, &gpu_rgba);
                println!("{}", render_gpu_hash_frame_rgba_line(frames, &gpu_rgba));
                let _wrapper_hashes = render_hash_pair_bgra_rgba(frame, &gpu_rgba);
                let _leaf = renderer::render_fingerprint_leaf_bgra(frame);
                let _config = renderer::ModernIndexCompareRunConfig::default();
                let _stats = renderer::ModernIndexCompareStats::from_env();
                let _typed: renderer::ModernIndexCompareResources;
                let modern_index_compare_resources = _typed;
                let _resources = load_modern_index_compare_resources(_config, root, false);
                let _output = render_modern_index_compare_output_from_capture();
            }
            fn compare_gpu_render_current_frame() {}
            fn cgram_match() {}
        """

        errors = module.check_main_text(textwrap.dedent(source))

        self.assertEqual(len(errors), 89)
        self.assertTrue(
            all("live GPU play backend ownership escaped gpu_capture boundary" in error for error in errors)
        )


if __name__ == "__main__":
    unittest.main()
