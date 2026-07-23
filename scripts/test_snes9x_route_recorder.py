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
            self.assertEqual(
                pairings["kind"], "zelda3_snes9x_rust_boundary_pairings_v1"
            )
            self.assertEqual(
                pairings["boundaries"]["1"]["rust_state"], str(rust_state.resolve())
            )
            self.assertFalse(pairings["boundaries"]["1"]["converted_to_snes9x"])
            self.assertEqual(rust_state.read_bytes(), b"rust-state")

    def test_exact_pass_promotes_rust_checkpoint_and_receipt_to_end_boundary(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            manifest = MODULE.load_manifest(project)
            manifest["takes"][0].update(
                {"start_boundary": 0, "end_boundary": 1, "frames": 12}
            )
            (project / "manifest.json").write_text(json.dumps(manifest))
            session = project / "comparisons/take-0000"
            session.mkdir(parents=True)
            (session / "rust_final.z3state").write_bytes(b"final-rust-state")
            (session / "result.json").write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "parity_eligible": True,
                        "frames_completed": 12,
                        "video": {"matched": True},
                        "audio": {"matched": True, "mode": "exact"},
                    }
                )
            )
            (session / "manifest.json").write_text(
                json.dumps(
                    {
                        "core": {"sha256": "11" * 32},
                        "rom": {"sha256": "22" * 32},
                    }
                )
            )
            (session / "frame_receipts.jsonl").write_text(
                json.dumps({"oracle_engine": {"main_module": 7}}) + "\n"
            )

            receipt = MODULE.promote_passed_take(project, 0, session)

            promoted = project / "boundaries/0001/rust.z3state"
            self.assertEqual(promoted.read_bytes(), b"final-rust-state")
            self.assertEqual(receipt["status"], "exact_av_verified")
            self.assertEqual(receipt["take"], 0)
            self.assertEqual(receipt["end_boundary"], 1)
            self.assertEqual(receipt["audio_comparison"], "exact")
            self.assertEqual(receipt["frames_verified"], 12)
            self.assertEqual(receipt["rust_state_sha256"], MODULE.sha256(promoted))
            self.assertEqual(
                receipt["oracle_state_sha256"],
                MODULE.sha256(project / "boundaries/0001/oracle.state"),
            )
            saved = json.loads(
                (project / "boundaries/0001/parity.json").read_text()
            )
            self.assertEqual(saved, receipt)
            pairing = MODULE.load_pairings(project)["boundaries"]["1"]
            self.assertEqual(pairing["rust_state"], "boundaries/0001/rust.z3state")
            self.assertEqual(pairing["verified_by"], "boundaries/0001/parity.json")
            self.assertFalse(pairing["converted_from_snes9x"])

    def test_exact_pass_cannot_promote_a_mislabeled_endpoint(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))
            manifest = MODULE.load_manifest(project)
            manifest["takes"][0].update(
                {"start_boundary": 0, "end_boundary": 1, "frames": 12}
            )
            (project / "manifest.json").write_text(json.dumps(manifest))
            session = project / "comparisons/take-0000"
            session.mkdir(parents=True)
            (session / "rust_final.z3state").write_bytes(b"wrong-endpoint")
            (session / "result.json").write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "parity_eligible": True,
                        "frames_completed": 12,
                        "video": {"matched": True},
                        "audio": {"matched": True, "mode": "exact"},
                    }
                )
            )
            (session / "manifest.json").write_text(
                json.dumps(
                    {
                        "core": {"sha256": "11" * 32},
                        "rom": {"sha256": "22" * 32},
                    }
                )
            )
            (session / "frame_receipts.jsonl").write_text(
                json.dumps({"oracle_engine": {"main_module": 14}}) + "\n"
            )

            with self.assertRaisesRegex(
                SystemExit,
                "take 0 endpoint does not match boundary 1: main: comparison=14 recorded=7",
            ):
                MODULE.promote_passed_take(project, 0, session)

            self.assertFalse((project / "boundaries/0001/rust.z3state").exists())
            self.assertFalse((project / "boundaries/0001/parity.json").exists())
            self.assertNotIn("1", MODULE.load_pairings(project)["boundaries"])

    def test_non_exact_or_incomplete_result_cannot_promote_boundary(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))
            manifest = MODULE.load_manifest(project)
            manifest["takes"][0].update(
                {"start_boundary": 0, "end_boundary": 1, "frames": 12}
            )
            (project / "manifest.json").write_text(json.dumps(manifest))
            session = project / "comparisons/take-0000"
            session.mkdir(parents=True)
            (session / "rust_final.z3state").write_bytes(b"must-not-promote")
            (session / "manifest.json").write_text("{}")
            diagnostic = {
                "status": "diagnostic_passed",
                "parity_eligible": False,
                "frames_completed": 12,
                "video": {"matched": True},
                "audio": {"matched": True, "mode": "timing"},
            }
            (session / "result.json").write_text(json.dumps(diagnostic))

            with self.assertRaisesRegex(SystemExit, "not an exact A/V parity pass"):
                MODULE.promote_passed_take(project, 0, session)

            self.assertFalse((project / "boundaries/0001/rust.z3state").exists())
            self.assertFalse((project / "boundaries/0001/parity.json").exists())
            self.assertNotIn("1", MODULE.load_pairings(project)["boundaries"])

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

    def test_reset_comparison_loads_the_recorded_boundary_sram(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            project = self.project(root)
            manifest = MODULE.load_manifest(project)
            manifest["takes"][0]["start_boundary"] = 0
            (project / "manifest.json").write_text(json.dumps(manifest))

            command = MODULE.compare_command(
                binary=Path("zelda3"),
                core=Path("core.dylib"),
                rom=Path("rom.sfc"),
                project=project,
                take_id=0,
                session_dir=root / "comparison",
            )

            load_index = command.index("--load-sram")
            self.assertEqual(
                command[load_index + 1],
                str(project / "boundaries/0000/sram.bin"),
            )

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
                    {
                        "id": 3,
                        "start_boundary": 0,
                        "frames": 2,
                        "input_path": "takes/0000/input.txt",
                        "status": "recovered_after_interruption",
                    },
                ]
            )
            (project / "manifest.json").write_text(json.dumps(manifest))

            self.assertEqual(MODULE.comparable_take_ids(project), [1, 3])

            rust_state = root / "rust.z3state"
            rust_state.write_bytes(b"rust-state")
            MODULE.pair_boundary(project, 1, rust_state)
            self.assertEqual(MODULE.comparable_take_ids(project), [0, 1, 3])

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
            (project / "takes/0001/input.txt").write_text("# take one\n0..1 0x0040\n")
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

    def test_merge_across_boundary_combines_adjacent_takes_and_preserves_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))
            (project / "takes/0000/input.txt").write_text("0..1 0x0008\n")
            (project / "takes/0000/frame_receipts.jsonl").write_text(
                '{"frame":0,"input":"0x0008","telemetry":{"health":24}}\n'
                '{"frame":1,"input":"0x0008","telemetry":{"health":23}}\n'
            )
            (project / "takes/0001").mkdir()
            (project / "takes/0001/input.txt").write_text("0 0x0010\n2 0x0020\n")
            (project / "takes/0001/frame_receipts.jsonl").write_text(
                '{"frame":0,"input":"0x0010","telemetry":{"health":23}}\n'
                '{"frame":2,"input":"0x0020","telemetry":{"health":22}}\n'
            )
            manifest = MODULE.load_manifest(project)
            manifest["boundaries"].append(
                {
                    "id": 2,
                    "reset_start": False,
                    "state_path": "boundaries/0002/oracle.state",
                    "sram_path": "boundaries/0002/sram.bin",
                }
            )
            manifest["takes"] = [
                {
                    "id": 0,
                    "start_boundary": 0,
                    "end_boundary": 1,
                    "frames": 4,
                    "input_path": "takes/0000/input.txt",
                    "receipts_path": "takes/0000/frame_receipts.jsonl",
                    "status": "complete",
                },
                {
                    "id": 1,
                    "start_boundary": 1,
                    "end_boundary": 2,
                    "frames": 3,
                    "input_path": "takes/0001/input.txt",
                    "receipts_path": "takes/0001/frame_receipts.jsonl",
                    "status": "complete",
                },
            ]
            (project / "manifest.json").write_text(json.dumps(manifest))

            merged = MODULE.merge_takes_across_boundary(project, 1)

            self.assertEqual(merged["id"], 2)
            self.assertEqual(merged["start_boundary"], 0)
            self.assertEqual(merged["end_boundary"], 2)
            self.assertEqual(merged["frames"], 7)
            self.assertEqual(merged["merged_from_takes"], [0, 1])
            self.assertEqual(merged["merged_across_boundary"], 1)
            self.assertEqual(
                (project / merged["input_path"]).read_text(),
                "# Continuous Snes9x route assembled from takes 0, 1.\n"
                "0..1 0x0008\n"
                "4 0x0010\n"
                "6 0x0020\n",
            )
            self.assertEqual(merged["receipts_path"], "takes/0002/frame_receipts.jsonl")
            merged_receipts = [
                json.loads(line)
                for line in (project / merged["receipts_path"]).read_text().splitlines()
            ]
            self.assertEqual(
                [receipt["frame"] for receipt in merged_receipts], [0, 1, 4, 6]
            )
            self.assertEqual(
                [receipt["telemetry"]["health"] for receipt in merged_receipts],
                [24, 23, 23, 22],
            )
            updated = MODULE.load_manifest(project)
            self.assertEqual(
                [take["status"] for take in updated["takes"][:2]], ["merged", "merged"]
            )
            self.assertEqual(MODULE.load_labels(project)["archived_boundaries"], [1])
            self.assertEqual(MODULE.continuous_take_ids(project), [2])
            self.assertEqual(MODULE.comparable_take_ids(project), [2])
            with self.assertRaisesRegex(SystemExit, "cannot be restored independently"):
                MODULE.set_take_discarded(project, 0, False)
            self.assertTrue((project / "takes/0000/input.txt").is_file())
            self.assertTrue((project / "takes/0001/input.txt").is_file())

    def test_merge_across_boundary_rejects_route_endpoints(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = self.project(Path(tmp))

            with self.assertRaisesRegex(
                SystemExit, "exactly one active incoming and outgoing"
            ):
                MODULE.merge_takes_across_boundary(project, 0)


if __name__ == "__main__":
    unittest.main()
