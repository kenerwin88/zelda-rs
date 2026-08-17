import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("compare_snes9x_cpu_checkpoints.py")
SPEC = importlib.util.spec_from_file_location("compare_snes9x_cpu_checkpoints", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CompareSnes9xCpuCheckpointsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.manifest = self.root / "manifest.json"
        self.manifest.write_text(
            json.dumps(
                {
                    "core": {"sha256": "a" * 64},
                    "rom": {"sha256": "b" * 64},
                    "timing": {"start_frame": 0},
                }
            )
        )
        self.state = {
            "main": 7,
            "sub": 2,
            "subsub": 12,
            "frame_counter": 9,
            "room": 114,
            "lights_out": 0,
            "palette_countdown": 3,
            "palette_direction": 2,
            "link_y": 0x215a,
            "link_x": 0x0937,
            "bg2_v": 0x2110,
            "bg2_h": 0x0900,
        }

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def write_jsonl(self, name: str, records: list[dict]) -> Path:
        path = self.root / name
        path.write_text("".join(json.dumps(record) + "\n" for record in records))
        return path

    def rust_checkpoint(self, run: int = 42) -> dict:
        return {
            "schema": 2,
            "event": "rust-cpu-checkpoint",
            "coordinate": "absolute comparison host frame",
            "host_frame": run,
            "pc": 0x8051,
            "v": 100,
            "cycles": 20,
            **self.state,
        }

    def oracle_checkpoint(self, run: int = 42) -> dict:
        return {
            "event": "pc",
            "run": run,
            "pc": 0x8051,
            "v": 100,
            "cycles": 32,
            **self.state,
        }

    def test_joins_only_by_explicit_run_and_reports_cycle_delta(self) -> None:
        rust = self.write_jsonl("rust.jsonl", [self.rust_checkpoint()])
        oracle = self.write_jsonl("oracle.jsonl", [self.oracle_checkpoint()])

        report = MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)

        self.assertEqual(len(report["comparisons"]), 1)
        self.assertEqual(
            report["comparisons"][0]["oracle_minus_rust_master_cycles"], 12
        )
        self.assertEqual(report["comparisons"][0]["run"], 42)
        self.assertEqual(report["comparisons"][0]["host_frame"], 42)

    def test_checkpoint_resume_maps_absolute_host_frame_through_manifest(self) -> None:
        self.manifest.write_text(
            json.dumps(
                {
                    "core": {"sha256": "a" * 64},
                    "rom": {"sha256": "b" * 64},
                    "timing": {"start_frame": 31_200},
                }
            )
        )
        rust = self.write_jsonl("rust.jsonl", [self.rust_checkpoint(run=31_286)])
        oracle = self.write_jsonl("oracle.jsonl", [self.oracle_checkpoint(run=86)])

        report = MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)

        self.assertEqual(report["comparisons"][0]["run"], 86)
        self.assertEqual(report["comparisons"][0]["host_frame"], 31_286)
        self.assertEqual(report["coordinate"]["comparison_start_frame"], 31_200)

    def test_checkpoint_resume_rejects_legacy_mislabeled_run(self) -> None:
        self.manifest.write_text(
            json.dumps(
                {
                    "core": {"sha256": "a" * 64},
                    "rom": {"sha256": "b" * 64},
                    "timing": {"start_frame": 31_200},
                }
            )
        )
        legacy = self.rust_checkpoint(run=31_286)
        legacy.update(
            {
                "schema": 1,
                "coordinate": "zero-based libretro retro_run",
                "run": legacy.pop("host_frame"),
            }
        )
        rust = self.write_jsonl("rust.jsonl", [legacy])
        oracle = self.write_jsonl("oracle.jsonl", [self.oracle_checkpoint(run=86)])

        with self.assertRaisesRegex(SystemExit, "falsely claims.*rebuild"):
            MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)

    def test_lorom_mirror_pc_is_joined_canonically(self) -> None:
        rust_record = self.rust_checkpoint()
        rust_record["pc"] = 0x00_8051
        oracle_record = self.oracle_checkpoint()
        oracle_record["pc"] = 0x80_8051
        rust = self.write_jsonl("rust.jsonl", [rust_record])
        oracle = self.write_jsonl("oracle.jsonl", [oracle_record])

        report = MODULE.compare(oracle, rust, self.manifest, 0x00_8051, None, None)

        self.assertEqual(len(report["comparisons"]), 1)

    def test_rejects_missing_oracle_checkpoint_instead_of_offset_guessing(self) -> None:
        rust = self.write_jsonl("rust.jsonl", [self.rust_checkpoint()])
        oracle = self.write_jsonl("oracle.jsonl", [self.oracle_checkpoint(run=43)])

        with self.assertRaisesRegex(SystemExit, "no selected checkpoint.*42"):
            MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)

    def test_rejects_state_mismatch_at_same_run(self) -> None:
        rust = self.write_jsonl("rust.jsonl", [self.rust_checkpoint()])
        oracle_record = self.oracle_checkpoint()
        oracle_record["room"] = 115
        oracle = self.write_jsonl("oracle.jsonl", [oracle_record])

        with self.assertRaisesRegex(SystemExit, "state provenance mismatch.*room"):
            MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)

    def test_rejects_a_different_rust_checkpoint_pc(self) -> None:
        rust_record = self.rust_checkpoint()
        rust_record["pc"] = 0x8034
        rust = self.write_jsonl("rust.jsonl", [rust_record])
        oracle = self.write_jsonl("oracle.jsonl", [self.oracle_checkpoint()])

        with self.assertRaisesRegex(SystemExit, "Rust checkpoint PC.*does not match"):
            MODULE.compare(oracle, rust, self.manifest, 0x8051, None, None)


if __name__ == "__main__":
    unittest.main()
