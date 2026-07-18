import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "snes9x_segment_matrix.py"


def load_module():
    spec = importlib.util.spec_from_file_location("snes9x_segment_matrix", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class Snes9xSegmentMatrixTests(unittest.TestCase):
    def test_rejects_a_capture_that_is_not_route_eligible(self):
        module = load_module()
        manifest = {
            "kind": "zelda3_snes9x_native_segment_matrix_capture_v1",
            "continuous_playthrough": False,
            "segments": [],
            "summary": {"eligible_for_segmented_output_parity": False},
        }
        with self.assertRaisesRegex(ValueError, "not eligible"):
            module.validate_capture_manifest(manifest)

    def test_builds_exact_modern_video_audio_commands_for_all_segments(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            segments = []
            for index in range(1, 14):
                segment_dir = root / f"segment-{index:02}"
                segment_dir.mkdir()
                rust = segment_dir / "rust_start.z3state"
                oracle = segment_dir / "oracle_start.state"
                rust.write_bytes(b"rust")
                oracle.write_bytes(b"oracle")
                rust_sha256 = module.sha256_file(rust)
                oracle_sha256 = module.sha256_file(oracle)
                segments.append(
                    {
                        "segment": index,
                        "frames": index * 10,
                        "eligible_for_output_parity": True,
                        "paired_starts": {
                            "rust": {"path": str(rust), "sha256": rust_sha256},
                            "oracle": {
                                "path": str(oracle),
                                "sha256": oracle_sha256,
                                "converted_from_rust": False,
                            },
                        },
                    }
                )
            manifest = {
                "kind": "zelda3_snes9x_native_segment_matrix_capture_v1",
                "continuous_playthrough": False,
                "core": {"path": "/core.dylib", "sha256": "corehash"},
                "rom": {"path": "/rom.sfc", "sha256": "romhash"},
                "segments": segments,
                "summary": {
                    "eligible_for_segmented_output_parity": True,
                    "aggregate_input_frames": 910,
                    "created_native_boundary_states": 12,
                },
            }
            commands = module.comparison_commands(
                manifest, pathlib.Path("/zelda3"), root / "results"
            )

        self.assertEqual(len(commands), 13)
        self.assertIn("--audio-comparison", commands[0])
        self.assertIn("exact", commands[0])
        self.assertIn("--rust-audio-backend", commands[0])
        self.assertIn("modern", commands[0])
        self.assertIn("--rust-audio-sequencer", commands[0])
        self.assertIn("native", commands[0])
        self.assertIn("--scan-all", commands[0])
        self.assertEqual(commands[-1][commands[-1].index("--resume-oracle-state") + 1], str(oracle))


if __name__ == "__main__":
    unittest.main()
