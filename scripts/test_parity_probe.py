#!/usr/bin/env python3
"""Regression tests for the focused parity probe's evidence selection."""

from __future__ import annotations

import importlib.util
import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("parity_probe.py")
SPEC = importlib.util.spec_from_file_location("parity_probe", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
parity_probe = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(parity_probe)


class ParityProbeTest(unittest.TestCase):
    def test_explicit_capture_range_is_inclusive_and_ordered(self) -> None:
        self.assertEqual(parity_probe.parse_frame_range("24068-24220"), (24068, 24220))
        with self.assertRaisesRegex(Exception, "START must not exceed END"):
            parity_probe.parse_frame_range("24220-24068")
        with self.assertRaisesRegex(Exception, "expected START-END"):
            parity_probe.parse_frame_range("24068")

    def test_staleness_guard_ignores_test_only_rust_sources(self) -> None:
        self.assertFalse(
            parity_probe.rust_source_affects_runtime_binary(
                Path("crates/zelda3/src/zelda_rtl_tests/display_publication.rs")
            )
        )
        self.assertFalse(
            parity_probe.rust_source_affects_runtime_binary(
                Path("crates/zelda3/tests/display_publication.rs")
            )
        )
        self.assertTrue(
            parity_probe.rust_source_affects_runtime_binary(
                Path("crates/zelda3/src/zelda_rtl.rs")
            )
        )

    def write_trace_fixture(
        self, root: Path, *, source_revision: str = "revision", patch_sha: str | None = None
    ) -> tuple[Path, Path, Path]:
        core = root / "trace.dylib"
        patch = root / "trace.patch"
        lock = root / "oracle-lock.json"
        core.write_bytes(b"binary-zelda3_snes9x_debug_ppu_value")
        patch.write_bytes(b"patch")
        lock_payload = {
            "core_name": "Snes9x",
            "core_version": "1.63",
            "source_tag": "1.63",
            "source_url": "https://example.test/snes9x.git",
            "source_revision": "revision",
        }
        lock.write_text(json.dumps(lock_payload), encoding="utf-8")
        receipt = {
            "schema": 1,
            "variant": "trace",
            **lock_payload,
            "source_revision": source_revision,
            "patch_sha256": patch_sha or hashlib.sha256(patch.read_bytes()).hexdigest(),
            "core_sha256": hashlib.sha256(core.read_bytes()).hexdigest(),
        }
        core.with_suffix(core.suffix + ".json").write_text(
            json.dumps(receipt), encoding="utf-8"
        )
        return core, patch, lock

    def write_binary(self, directory: Path, mtime: int) -> Path:
        binary = directory / "zelda3"
        binary.write_bytes(b"binary")
        os.utime(binary, (mtime, mtime))
        return binary

    def write_run(self, precommit: Path, frame: int, mtime: int) -> Path:
        run_dir = precommit / f"run-{frame}"
        run_dir.mkdir(parents=True)
        (run_dir / "replay.sh").write_text(
            "cargo run -q -p zelda3-bin -- --compare-snes9x-oracle core rom "
            f"{frame}\n",
            encoding="utf-8",
        )
        (run_dir / "input.txt").write_text("", encoding="utf-8")
        os.utime(run_dir, (mtime, mtime))
        return run_dir

    def test_resolve_run_dir_prefers_coverage_over_newest_mtime(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            precommit = project / "comparisons" / "precommit"
            binary = self.write_binary(project, 100)
            self.write_run(precommit, 10_000, 200)
            sufficient = self.write_run(precommit, 12_296, 100)

            selected = parity_probe.resolve_run_dir(project, None, 11_753, binary)

            self.assertEqual(selected, sufficient.resolve())

    def test_explicit_run_dir_must_cover_probe_window(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            binary = self.write_binary(project, 100)
            run_dir = self.write_run(project, 10_000, 100)

            with self.assertRaisesRegex(SystemExit, "covers only 10000"):
                parity_probe.resolve_run_dir(project, run_dir, 11_753, binary)

    def test_authoritative_probe_skips_resumed_gate_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            precommit = project / "comparisons" / "precommit"
            binary = self.write_binary(project, 100)
            rust_state = project / "rust.z3state"
            oracle_state = project / "oracle.state"
            for path in (rust_state, oracle_state):
                path.write_bytes(b"state")
                os.utime(path, (200, 200))
            resumed = self.write_run(precommit, 12_000, 200)
            (resumed / "replay.sh").write_text(
                "cargo run -q -p zelda3-bin -- --compare-snes9x-oracle core rom 12000 "
                f"--resume-rust-state {rust_state} --resume-oracle-state {oracle_state} "
                "--compare-from-frame 10000\n",
                encoding="utf-8",
            )
            cold = self.write_run(precommit, 13_000, 100)

            selected = parity_probe.resolve_run_dir(
                project, None, 11_753, binary, require_cold=True
            )

            self.assertEqual(selected, cold.resolve())

    def test_authoritative_probe_rejects_explicit_resumed_run(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            binary = self.write_binary(project, 100)
            rust_state = project / "rust.z3state"
            oracle_state = project / "oracle.state"
            for path in (rust_state, oracle_state):
                path.write_bytes(b"state")
                os.utime(path, (200, 200))
            resumed = self.write_run(project, 12_000, 200)
            (resumed / "replay.sh").write_text(
                "cargo run -q -p zelda3-bin -- --compare-snes9x-oracle core rom 12000 "
                f"--resume-rust-state {rust_state} --resume-oracle-state {oracle_state} "
                "--compare-from-frame 10000\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "authoritative proof"):
                parity_probe.resolve_run_dir(
                    project, resumed, 11_753, binary, require_cold=True
                )

    def test_stale_binary_override_is_dry_run_only(self) -> None:
        parity_probe.validate_stale_override(True, True)
        with self.assertRaisesRegex(SystemExit, "restricted to --dry-run"):
            parity_probe.validate_stale_override(True, False)

    def test_probe_retention_prunes_only_old_generated_sessions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            generated = []
            for index in range(5):
                session = root / f"probe-{index}"
                session.mkdir()
                (session / "result.json").write_text("{}", encoding="utf-8")
                os.utime(session, (index, index))
                generated.append(session)
            checkpoint = root / "checkpoints"
            checkpoint.mkdir()
            manual = root / "manual-evidence"
            manual.mkdir()

            removed = parity_probe.prune_probe_sessions(root, 2)

            self.assertEqual(removed, [generated[2], generated[1], generated[0]])
            self.assertEqual(
                sorted(path.name for path in root.iterdir()),
                ["checkpoints", "manual-evidence", "probe-3", "probe-4"],
            )

    def test_zero_probe_retention_disables_pruning(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = root / "probe-1"
            session.mkdir()

            self.assertEqual(parity_probe.prune_probe_sessions(root, 0), [])
            self.assertTrue(session.is_dir())

    def test_failure_focus_extracts_frontier_frame_and_first_bad_pixel(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            failure = Path(directory)
            (failure / "diff.json").write_text(
                json.dumps(
                    {
                        "frame": 23934,
                        "message": "video divergence: first_mismatch=(203, 40) rust=black",
                    }
                ),
                encoding="utf-8",
            )

            focus = parity_probe.load_failure_focus(failure)

            self.assertEqual(focus.frame, 23934)
            self.assertEqual(focus.pixel, (203, 40))
            self.assertEqual(focus.directory, failure.resolve())

    def test_live_oam_is_not_classified_as_a_completed_scanout_cause(self) -> None:
        causal, post_frame_only = parity_probe.split_display_domains(
            ["live_oam", "presented_oam", "cgram"]
        )

        self.assertEqual(causal, ["presented_oam", "cgram"])
        self.assertEqual(post_frame_only, ["live_oam"])

    def test_valid_decoded_obj_cache_is_a_scanout_causal_domain(self) -> None:
        rust = {"presented_obj_tile_cache": [0] * 128}
        oracle = {
            "presented_obj_tile_cache": [0] * 128,
            "presented_obj_tile_cache_valid": [0, 1],
        }
        oracle["presented_obj_tile_cache"][64 + 7] = 4

        self.assertEqual(
            parity_probe.valid_obj_tile_cache_differences(rust, oracle),
            ([71], 64),
        )

    def test_unsupported_decoded_obj_cache_receipt_is_ignored(self) -> None:
        rust = {"presented_obj_tile_cache": [0] * 64}
        oracle = {
            "presented_obj_tile_cache": [-1] * 64,
            "presented_obj_tile_cache_valid": [-1],
        }

        self.assertIsNone(parity_probe.valid_obj_tile_cache_differences(rust, oracle))

    def test_cross_build_checkpoint_trust_still_requires_a_complete_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory)
            wanted: dict[str, object] = {}
            self.assertEqual(
                parity_probe.checkpoint_reuse_problem(
                    checkpoint, wanted, trust_cross_build=True
                ),
                "no saved checkpoint generation",
            )
            generation = checkpoint / "frame-00023012"
            generation.mkdir()
            (generation / "manifest.json").write_text(
                json.dumps(
                    {"rust_state": "rust.z3state", "oracle_state": "oracle.state"}
                ),
                encoding="utf-8",
            )
            (generation / "rust.z3state").write_bytes(b"rust")
            (generation / "oracle.state").write_bytes(b"oracle")
            (checkpoint / "latest.json").write_text(
                json.dumps({"frame": 23012, "checkpoint": generation.name}),
                encoding="utf-8",
            )

            self.assertIsNone(
                parity_probe.checkpoint_reuse_problem(
                    checkpoint, wanted, trust_cross_build=True
                )
            )

    def test_cross_build_checkpoint_trust_accepts_an_exact_generation_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory)
            (checkpoint / "manifest.json").write_text(
                json.dumps(
                    {
                        "frame": 13586,
                        "rust_state": "rust.z3state",
                        "oracle_state": "oracle.state",
                    }
                ),
                encoding="utf-8",
            )
            (checkpoint / "rust.z3state").write_bytes(b"rust")
            (checkpoint / "oracle.state").write_bytes(b"oracle")

            self.assertIsNone(
                parity_probe.checkpoint_reuse_problem(
                    checkpoint, {}, trust_cross_build=True
                )
            )
            self.assertEqual(parity_probe.saved_checkpoint_frame(checkpoint), 13586)

    def test_start_mode_description_exposes_checkpoint_or_cold_replay(self) -> None:
        self.assertIn(
            "paired states",
            parity_probe.source_start_description(["--resume-rust-state", "rust"]),
        )
        self.assertIn(
            "cold replay",
            parity_probe.source_start_description(["--load-sram", "initial.srm"]),
        )

    def test_resumed_gate_reuses_its_fresh_paired_start_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "zelda3"
            rust_state = root / "rust_initial.z3state"
            oracle_state = root / "oracle_initial.state"
            for path in (binary, rust_state, oracle_state):
                path.write_bytes(b"state")
            os.utime(binary, (100, 100))
            os.utime(rust_state, (200, 200))
            os.utime(oracle_state, (200, 200))
            replay_argv = [
                "--compare-snes9x-oracle",
                "core",
                "rom",
                "12296",
                "--resume-rust-state",
                str(rust_state),
                "--resume-oracle-state",
                str(oracle_state),
                "--compare-from-frame",
                "10000",
            ]

            self.assertEqual(
                parity_probe.source_start_arguments(replay_argv, binary),
                [
                    "--resume-rust-state",
                    str(rust_state),
                    "--resume-oracle-state",
                    str(oracle_state),
                    "--compare-from-frame",
                    "10000",
                ],
            )

    def test_half_paired_source_state_is_rejected(self) -> None:
        replay_argv = [
            "--compare-snes9x-oracle",
            "core",
            "rom",
            "12296",
            "--resume-rust-state",
            "rust_initial.z3state",
        ]

        with self.assertRaisesRegex(SystemExit, "only one half"):
            parity_probe.source_start_arguments(replay_argv, Path("binary"))

    def test_resolve_run_dir_skips_stale_paired_start(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            precommit = project / "comparisons" / "precommit"
            binary = self.write_binary(project, 200)
            rust_state = project / "rust.z3state"
            oracle_state = project / "oracle.state"
            for path in (rust_state, oracle_state):
                path.write_bytes(b"state")
            os.utime(rust_state, (100, 100))
            os.utime(oracle_state, (100, 100))
            stale = self.write_run(precommit, 12_296, 200)
            (stale / "replay.sh").write_text(
                "cargo run -q -p zelda3-bin -- --compare-snes9x-oracle core rom 12296 "
                f"--resume-rust-state {rust_state} --resume-oracle-state {oracle_state} "
                "--compare-from-frame 10000\n",
                encoding="utf-8",
            )
            cold = self.write_run(precommit, 13_680, 100)

            self.assertEqual(
                parity_probe.resolve_run_dir(project, None, 11_789, binary),
                cold.resolve(),
            )

    def test_trace_core_receipt_pins_core_source_and_patch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            core, patch, lock = self.write_trace_fixture(Path(directory))

            self.assertEqual(
                parity_probe.validate_trace_core(core, lock_path=lock, patch_path=patch),
                hashlib.sha256(core.read_bytes()).hexdigest(),
            )

    def test_trace_core_rejects_stale_patch_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            core, patch, lock = self.write_trace_fixture(
                Path(directory), patch_sha="0" * 64
            )

            with self.assertRaisesRegex(SystemExit, "predates the current trace patch"):
                parity_probe.validate_trace_core(core, lock_path=lock, patch_path=patch)

    def test_trace_core_rejects_wrong_source_revision(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            core, patch, lock = self.write_trace_fixture(
                Path(directory), source_revision="obsolete"
            )

            with self.assertRaisesRegex(SystemExit, "source_revision does not match"):
                parity_probe.validate_trace_core(core, lock_path=lock, patch_path=patch)


if __name__ == "__main__":
    unittest.main()
