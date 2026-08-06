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

    def test_stale_binary_override_is_dry_run_only(self) -> None:
        parity_probe.validate_stale_override(True, True)
        with self.assertRaisesRegex(SystemExit, "restricted to --dry-run"):
            parity_probe.validate_stale_override(True, False)

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
