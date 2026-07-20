import importlib.util
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "diagnose_snes9x_state.py"


def load_module():
    spec = importlib.util.spec_from_file_location("diagnose_snes9x_state", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class DiagnoseSnes9xStateTests(unittest.TestCase):
    def test_groups_contiguous_ranges(self):
        module = load_module()
        self.assertEqual(
            module.differences(bytes([0, 0, 0, 0, 0]), bytes([0, 1, 2, 0, 3])),
            [module.Difference(1, 3), module.Difference(4, 5)],
        )

    def test_wram_label_marks_intro_block(self):
        module = load_module()
        self.assertIn("intro state block", module.label_for("wram", 0x1E10, 0x1E12))

    def test_changed_bytes_ignores_an_unchanged_reset_difference(self):
        module = load_module()
        self.assertEqual(module.changed_bytes(bytes([0, 0]), bytes([0, 3])), bytes([0, 3]))
        self.assertEqual(module.differences(bytes([0, 3]), bytes([0, 3])), [])

    def test_capture_path_uses_comparator_artifact_names(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            self.assertEqual(
                module.capture_path(pathlib.Path(temp), "oracle", "vram", 81).name,
                "oracle_vram_frame_81.bin",
            )
