import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("snes9x_route_recorder.py")
SPEC = importlib.util.spec_from_file_location("snes9x_route_recorder", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class Snes9xRouteRecorderTests(unittest.TestCase):
    def test_no_arguments_default_to_tui(self):
        self.assertEqual(MODULE.parse_cli_args([]).action, "tui")

    def project(self, root: Path) -> Path:
        project = root / "route"
        (project / "boundaries/0000").mkdir(parents=True)
        (project / "boundaries/0001").mkdir(parents=True)
        (project / "takes/0000").mkdir(parents=True)
        (project / "boundaries/0000/oracle.state").write_bytes(b"state0")
        (project / "boundaries/0001/oracle.state").write_bytes(b"state1")
        (project / "boundaries/0000/sram.bin").write_bytes(b"sram0")
        (project / "boundaries/0001/sram.bin").write_bytes(b"sram1")
        (project / "takes/0000/input.txt").write_text("0 0x0080\n")
        (project / "manifest.json").write_text(
            json.dumps(
                {
                    "kind": "zelda3_snes9x_route_recording_v1",
                    "identity": {
                        "core_sha256": "11" * 32,
                        "rom_sha256": "22" * 32,
                    },
                    "boundaries": [
                        {
                            "id": 0,
                            "reset_start": True,
                            "state_path": "boundaries/0000/oracle.state",
                            "sram_path": "boundaries/0000/sram.bin",
                            "telemetry": {"main": 0, "health": 0},
                        },
                        {
                            "id": 1,
                            "reset_start": False,
                            "state_path": "boundaries/0001/oracle.state",
                            "sram_path": "boundaries/0001/sram.bin",
                            "telemetry": {"main": 7, "health": 24},
                        },
                    ],
                    "takes": [
                        {
                            "id": 0,
                            "start_boundary": 1,
                            "frames": 12,
                            "input_path": "takes/0000/input.txt",
                        }
                    ],
                }
            )
        )
        return project

    def test_pairing_records_hash_without_copying_or_converting_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            rust_state = root / "rust.z3state"
            rust_state.write_bytes(b"rust-state")

            MODULE.pair_boundary(project, 1, rust_state)

            pairings = json.loads((project / "pairings.json").read_text())
            self.assertEqual(pairings["kind"], "zelda3_snes9x_rust_boundary_pairings_v1")
            self.assertEqual(pairings["boundaries"]["1"]["rust_state"], str(rust_state.resolve()))
            self.assertFalse(pairings["boundaries"]["1"]["converted_to_snes9x"])
            self.assertEqual(rust_state.read_bytes(), b"rust-state")

    def test_compare_command_uses_take_start_boundary_and_exact_modern_lanes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            rust_state = root / "rust.z3state"
            rust_state.write_bytes(b"rust-state")
            MODULE.pair_boundary(project, 1, rust_state)

            command = MODULE.compare_command(
                binary=Path("zelda3"),
                core=Path("core.dylib"),
                rom=Path("rom.sfc"),
                project=project,
                take_id=0,
                session_dir=root / "comparison",
            )

            self.assertIn("--resume-rust-state", command)
            self.assertIn(str(rust_state.resolve()), command)
            self.assertIn("--resume-oracle-state", command)
            self.assertIn(str(project / "boundaries/0001/oracle.state"), command)
            self.assertIn("--resume-oracle-sram", command)
            self.assertIn(str(project / "boundaries/0001/sram.bin"), command)
            self.assertIn("--audio-comparison", command)
            self.assertIn("exact", command)
            self.assertIn("--rust-audio-backend", command)
            self.assertIn("modern", command)
            self.assertIn("--rust-audio-sequencer", command)
            self.assertIn("native", command)
            self.assertIn("--scan-all", command)

    def test_only_reset_or_paired_nonempty_takes_are_matrix_comparable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            manifest = json.loads((project / "manifest.json").read_text())
            manifest["takes"].extend(
                [
                    {
                        "id": 1,
                        "start_boundary": 0,
                        "frames": 3,
                        "input_path": "takes/0000/input.txt",
                    },
                    {
                        "id": 2,
                        "start_boundary": 1,
                        "frames": 0,
                        "input_path": "takes/0000/input.txt",
                    },
                ]
            )
            (project / "manifest.json").write_text(json.dumps(manifest))

            self.assertEqual(MODULE.comparable_take_ids(project), [1])

            rust_state = root / "rust.z3state"
            rust_state.write_bytes(b"rust-state")
            MODULE.pair_boundary(project, 1, rust_state)
            self.assertEqual(MODULE.comparable_take_ids(project), [0, 1])

    def test_unpaired_nonempty_takes_are_reported_as_excluded(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            self.assertEqual(
                MODULE.excluded_nonempty_takes(project),
                [
                    {
                        "take": 0,
                        "start_boundary": 1,
                        "frames": 12,
                        "reason": "start boundary has no paired Rust-native state",
                    }
                ],
            )

    def test_boundaries_can_be_named_and_resolved_case_insensitively(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            MODULE.name_boundary(project, 1, "Eastern Palace entrance")

            self.assertEqual(
                MODULE.resolve_start_boundary(project, "eastern palace ENTRANCE"),
                "1",
            )
            labels = json.loads((project / "labels.json").read_text())
            self.assertEqual(labels["boundaries"]["1"], "Eastern Palace entrance")

    def test_boundary_archive_is_reversible_and_preserves_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            MODULE.set_boundary_archived(project, 1, True)
            self.assertEqual(MODULE.load_labels(project)["archived_boundaries"], [1])
            self.assertTrue((project / "boundaries/0001/oracle.state").is_file())

            MODULE.set_boundary_archived(project, 1, False)
            self.assertEqual(MODULE.load_labels(project)["archived_boundaries"], [])

    def test_project_archive_is_reversible_and_preserves_recording(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            MODULE.set_project_archived(project, True)
            self.assertTrue(MODULE.load_labels(project)["archived_project"])
            self.assertTrue((project / "manifest.json").is_file())

            MODULE.set_project_archived(project, False)
            self.assertFalse(MODULE.load_labels(project)["archived_project"])

    def test_discarded_take_is_preserved_but_omitted_from_comparison(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            MODULE.set_take_discarded(project, 0, True)

            manifest = MODULE.load_manifest(project)
            self.assertEqual(manifest["takes"][0]["status"], "discarded")
            self.assertTrue((project / "takes/0000/input.txt").is_file())
            self.assertEqual(MODULE.comparable_take_ids(project), [])
            self.assertEqual(MODULE.excluded_nonempty_takes(project), [])

    def test_existing_project_seeds_from_selected_boundary_sram(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            seed = MODULE.prepare_recording_sram(
                project, "1", Path(tmp) / "unrelated.srm", False
            )

            self.assertEqual(seed, project / "boundaries/0001/sram.bin")

    def test_new_project_records_initial_sram_provenance(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = root / "new-route"
            sram = root / "chosen.srm"
            sram.write_bytes(b"chosen-sram")

            seed = MODULE.prepare_recording_sram(project, "latest", sram, False)

            self.assertEqual(seed, sram.resolve())
            origin = json.loads((project / "sram-origin.json").read_text())
            self.assertEqual(origin["filename"], "chosen.srm")
            self.assertNotIn("path", origin)
            self.assertEqual(origin["sha256"], MODULE.sha256(sram))

    def test_repository_sram_origin_uses_relative_path(self):
        self.assertEqual(
            MODULE.portable_source_path(MODULE.DEFAULT_SRAM),
            {"path": "saves/sram.dat"},
        )

    def test_continuous_chain_covers_adjacent_takes_from_reset(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))
            manifest = MODULE.load_manifest(project)
            manifest["takes"] = [
                {
                    "id": 0,
                    "start_boundary": 0,
                    "end_boundary": 1,
                    "frames": 12,
                    "input_path": "takes/0000/input.txt",
                    "status": "complete",
                },
                {
                    "id": 1,
                    "start_boundary": 1,
                    "end_boundary": 2,
                    "frames": 3,
                    "input_path": "takes/0001/input.txt",
                    "status": "complete",
                },
            ]
            manifest["boundaries"].append(
                {
                    "id": 2,
                    "reset_start": False,
                    "state_path": "boundaries/0002/oracle.state",
                    "sram_path": "boundaries/0002/sram.bin",
                }
            )
            (project / "manifest.json").write_text(json.dumps(manifest))

            self.assertEqual(MODULE.continuous_take_ids(project), [0, 1])

    def test_combined_input_offsets_each_take_frame_range(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            (project / "takes/0000/input.txt").write_text(
                "# take zero\n0..2 0x0008\n4 0x0010\n"
            )
            (project / "takes/0001").mkdir()
            (project / "takes/0001/input.txt").write_text(
                "# take one\n0..1 0x0040\n"
            )
            manifest = MODULE.load_manifest(project)
            manifest["takes"] = [
                {
                    "id": 0,
                    "start_boundary": 0,
                    "end_boundary": 1,
                    "frames": 5,
                    "input_path": "takes/0000/input.txt",
                    "status": "complete",
                },
                {
                    "id": 1,
                    "start_boundary": 1,
                    "end_boundary": None,
                    "frames": 2,
                    "input_path": "takes/0001/input.txt",
                    "status": "complete",
                },
            ]
            (project / "manifest.json").write_text(json.dumps(manifest))

            output = root / "combined.txt"
            frames = MODULE.write_continuous_input(project, [0, 1], output)

            self.assertEqual(frames, 7)
            self.assertEqual(
                output.read_text(),
                "# Continuous Snes9x route assembled from takes 0, 1.\n"
                "0..2 0x0008\n"
                "4 0x0010\n"
                "5..6 0x0040\n",
            )


if __name__ == "__main__":
    unittest.main()
