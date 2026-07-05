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
        let scene = renderer::ModernAssetFrameScene::from_in_dungeon(false);
        let stats = renderer::ModernAssetLiveStats::from_env();
        let compare = renderer::ModernIndexCompareStats::from_env();
        let source_table = renderer::MappedSourceTableView::new(&[(0, 0, 0)], |src: &(u8, u16, u16)| *src);
        let diff = renderer::compare_modern_index_rgba(&[], &[]);
        let gpu_diff = renderer::compare_gpu_render_bgra_to_rgba(&[], &[]);
        let frame = renderer::GpuFrame::from_source_and_raw_scanlines();
        let atlas_render = renderer::modern_gpu::render_modern_atlas_compare_frame();
        frontend.present_modern_asset_frame();
        renderer::modern_gpu::render_modern_index_compare_frame();
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
                if let Some(_) = variant_trace_pixel_env() {
                    renderer::modern_extract::extract_modern_frame_from_sources();
                    renderer::modern_variant_draw::trace_variant_plan_pixel();
                }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("run_replay_save", errors[0])

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
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
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
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 4)
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

    def test_rejects_source_table_view_adapter_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            struct VramChrSourceTableView {}
            impl renderer::modern_extract::SourceTableView for VramChrSourceTableView {
                fn get(&self, slot: usize) -> (u8, u16, u16) { (0, 0, 0) }
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 2)
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
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("modern index compare diff assembly escaped renderer boundary", errors[0])

    def test_rejects_direct_gpu_render_compare_diff_assembly_calls(self):
        module = load_module()
        source = source_with_required_calls(
            """
            fn run_play_with_state() {
                let diff = renderer::compare_bgra_to_rgba(&cpu, &gpu);
            }
            """
        )

        errors = module.check_source_text(source)

        self.assertEqual(len(errors), 1)
        self.assertIn("gpu render compare diff assembly escaped renderer boundary", errors[0])

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
