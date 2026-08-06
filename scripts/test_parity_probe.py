#!/usr/bin/env python3
"""Regression tests for the focused parity probe's evidence selection."""

from __future__ import annotations

import importlib.util
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
            self.write_run(precommit, 10_000, 200)
            sufficient = self.write_run(precommit, 12_296, 100)

            selected = parity_probe.resolve_run_dir(project, None, 11_753)

            self.assertEqual(selected, sufficient.resolve())

    def test_explicit_run_dir_must_cover_probe_window(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_dir = self.write_run(Path(directory), 10_000, 100)

            with self.assertRaisesRegex(SystemExit, "covers only 10000"):
                parity_probe.resolve_run_dir(Path(directory), run_dir, 11_753)

    def test_stale_binary_override_is_dry_run_only(self) -> None:
        parity_probe.validate_stale_override(True, True)
        with self.assertRaisesRegex(SystemExit, "restricted to --dry-run"):
            parity_probe.validate_stale_override(True, False)


if __name__ == "__main__":
    unittest.main()
