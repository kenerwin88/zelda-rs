import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "classify_display_oracle.py"


def load_module():
    spec = importlib.util.spec_from_file_location("classify_display_oracle", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ClassifyDisplayOracleTests(unittest.TestCase):
    def test_classifies_brightness_and_finds_exact_candidate(self):
        module = load_module()
        report = module.analyze_record(
            {
                "frame": 27011,
                "stage": "after",
                "oracle": {"brightness": 0, "cgram": [1, 2]},
                "rust": {"brightness": 15, "cgram": [1, 2]},
                "rust_candidates": [
                    {"name": "selected", "brightness": 15},
                    {"name": "captured_before_nmi", "brightness": 0},
                ],
                "rust_context": {"entry_frame": [18, 3, 0, 121]},
            }
        )

        self.assertEqual(report["classification"], "active-display-blanking")
        self.assertEqual(report["field_mismatches"], {"brightness": 1})
        self.assertEqual(
            report["exact_candidates"]["brightness"], ["captured_before_nmi"]
        )

    def test_counts_nested_array_mismatches_and_classifies_cgram(self):
        module = load_module()
        self.assertEqual(module.mismatch_count([[1, 2], [3]], [[1, 9], [4]]), 2)
        report = module.analyze_record(
            {
                "oracle": {"cgram": [1, 2, 3]},
                "rust": {"cgram": [1, 9, 8]},
                "rust_candidates": [
                    {"name": "before_nmi_upload", "cgram": [1, 2, 3]}
                ],
            }
        )
        self.assertEqual(report["classification"], "cgram-generation")
        self.assertEqual(report["field_mismatches"], {"cgram": 2})

    def test_blackout_mismatch_suppresses_hidden_display_domains(self):
        module = load_module()
        report = module.analyze_record(
            {
                "oracle": {
                    "brightness": 0,
                    "forced_blank": True,
                    "presented_oam": [1],
                    "mode7": [-1],
                },
                "rust": {
                    "brightness": 15,
                    "forced_blank": False,
                    "presented_oam": [2],
                    "mode7": [9],
                },
            }
        )
        self.assertEqual(report["classification"], "active-display-blanking")
        self.assertEqual(
            report["field_mismatches"], {"brightness": 1, "forced_blank": 1}
        )
        self.assertEqual(
            report["suppressed_mismatches"], {"presented_oam": 1, "mode7": 1}
        )

    def test_loads_a_frame_from_a_session_directory(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            path = pathlib.Path(temp) / "display_oracle.jsonl"
            path.write_text(
                "\n".join(
                    json.dumps(
                        {
                            "frame": frame,
                            "oracle": {"presented_oam": [1]},
                            "rust": {"presented_oam": [value]},
                        }
                    )
                    for frame, value in [(10, 2), (11, 1)]
                )
            )
            record = module.load_record(pathlib.Path(temp), 11)
        self.assertEqual(record["frame"], 11)

    def test_ignores_unavailable_diagnostic_fields(self):
        module = load_module()
        report = module.analyze_record(
            {
                "oracle": {
                    "brightness_white": -1,
                    "presented_clip": [1, 2],
                    "presented_oam": [1],
                    "presented_obj_tile_cache_valid": [1],
                    "presented_obj_tile_cache": [7],
                },
                "rust": {
                    "brightness_white": 31,
                    "presented_clip": None,
                    "presented_oam": [2],
                    "presented_obj_tile_cache_valid": None,
                    "presented_obj_tile_cache": [9],
                },
            }
        )
        self.assertEqual(report["classification"], "multi-domain-publication")
        self.assertEqual(
            report["field_mismatches"],
            {"presented_oam": 1, "presented_obj_tile_cache": 1},
        )

    def test_obj_cache_compares_only_oracle_valid_tiles(self):
        module = load_module()
        oracle = {
            "presented_obj_tile_cache_valid": [1, 0],
            "presented_obj_tile_cache": [1] * 64 + [2] * 64,
        }
        rust = {
            "presented_obj_tile_cache_valid": None,
            "presented_obj_tile_cache": [1] * 64 + [9] * 64,
        }
        report = module.analyze_record(
            {"oracle": oracle, "rust": rust, "rust_candidates": []}
        )
        self.assertEqual(report["classification"], "exact")
        self.assertEqual(report["field_mismatches"], {})


if __name__ == "__main__":
    unittest.main()
