#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("prune_parity_artifacts.py")
SPEC = importlib.util.spec_from_file_location("prune_parity_artifacts", SCRIPT)
prune_parity_artifacts = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(prune_parity_artifacts)

class PruneParityArtifactsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.target = Path(self.temp_dir.name) / "target"
        self.target.mkdir()
        self.stale_trace = self.target / "parity-stale-trace"
        self.stale_trace.mkdir()
        (self.stale_trace / "trace.bin").write_bytes(b"trace")
        self.checkpoints = self.target / "parity-checkpoints"
        self.checkpoints.mkdir()
        self.stale_checkpoint = self.checkpoints / "old-frontier"
        self.stale_checkpoint.mkdir()
        (self.stale_checkpoint / "state.bin").write_bytes(b"state")
        self.retained = []
        for name in prune_parity_artifacts.RETAINED_CHECKPOINT_LINEAGES:
            path = self.checkpoints / name
            path.mkdir()
            (path / "manifest.json").write_text("{}")
            self.retained.append(path)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_dry_run_preserves_every_artifact(self) -> None:
        count, reclaimed = prune_parity_artifacts.prune(self.target, apply=False)

        self.assertEqual(count, 2)
        self.assertGreater(reclaimed, 0)
        self.assertTrue(self.stale_trace.exists())
        self.assertTrue(self.stale_checkpoint.exists())
        self.assertTrue(all(path.exists() for path in self.retained))

    def test_apply_removes_only_superseded_artifacts(self) -> None:
        count, reclaimed = prune_parity_artifacts.prune(self.target, apply=True)

        self.assertEqual(count, 2)
        self.assertGreater(reclaimed, 0)
        self.assertFalse(self.stale_trace.exists())
        self.assertFalse(self.stale_checkpoint.exists())
        self.assertTrue(all(path.exists() for path in self.retained))

    def test_refuses_non_target_directory(self) -> None:
        with self.assertRaisesRegex(ValueError, "non-target directory"):
            prune_parity_artifacts.prune(Path(self.temp_dir.name), apply=False)


if __name__ == "__main__":
    unittest.main()
