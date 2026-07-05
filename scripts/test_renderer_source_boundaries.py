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
