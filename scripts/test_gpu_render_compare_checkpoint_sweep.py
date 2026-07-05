#!/usr/bin/env python3

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import gpu_render_compare_checkpoint_sweep as sweep


class CheckpointSweepTests(unittest.TestCase):
    def test_discover_checkpoints_sorts_numeric_frames_and_skips_refresh_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            state_dir = Path(tmp)
            for name in [
                "rust-frame-1034152-refresh.sav",
                "rust-frame-20.sav",
                "rust-frame-after.sav",
                "rust-frame-3.sav",
            ]:
                (state_dir / name).write_bytes(b"state")

            checkpoints = sweep.discover_checkpoints(state_dir)

        self.assertEqual([checkpoint.frame for checkpoint in checkpoints], [3, 20])

    def test_select_checkpoints_filters_by_frame_bounds_and_limit(self) -> None:
        checkpoints = [
            sweep.ReplayCheckpoint(10, Path("a.sav")),
            sweep.ReplayCheckpoint(20, Path("b.sav")),
            sweep.ReplayCheckpoint(30, Path("c.sav")),
        ]

        selected = sweep.select_checkpoints(checkpoints, start_frame=15, end_frame=35, limit=1)

        self.assertEqual([checkpoint.frame for checkpoint in selected], [20])

    def test_command_for_checkpoint_enforces_full_gpu_modern_index_parity(self) -> None:
        checkpoint = sweep.ReplayCheckpoint(279401, Path(".cache/rust-frame-279401.sav"))

        command = sweep.command_for_checkpoint(
            checkpoint,
            Path("target/parity/zelda3"),
            Path("saves/zelda3.sfc"),
            Path("saves/route.sav"),
            frames=3,
            modern_index_compare=1,
        )

        self.assertEqual(command[:5], [
            "target/parity/zelda3",
            "--replay-save",
            "saves/zelda3.sfc",
            "saves/route.sav",
            "279404",
        ])
        self.assertIn("--load-state", command)
        self.assertIn("--modern-index-compare", command)
        self.assertIn("--require-full-gpu-path", command)
        self.assertIn("--require-modern-index-parity", command)

    def test_env_for_run_does_not_override_renderer_by_default(self) -> None:
        env = sweep.env_for_run({"PATH": "/bin"}, renderer=None)

        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"], "1")
        self.assertNotIn("ZELDA3_RENDERER", env)

    def test_validate_summary_stats_rejects_cpu_fallback(self) -> None:
        checkpoint = sweep.ReplayCheckpoint(100, Path("a.sav"))

        with self.assertRaises(SystemExit):
            sweep.validate_summary_stats(
                checkpoint,
                {
                    "compare_count": 3,
                    "bad_count": 0,
                    "bad_pixels": 0,
                    "gpu_count": 2,
                    "mode7_gpu_count": 0,
                    "cpu_count": 1,
                },
            )

    def test_run_sweep_reports_compared_frames_from_summaries(self) -> None:
        checkpoints = [
            sweep.ReplayCheckpoint(100, Path("a.sav")),
            sweep.ReplayCheckpoint(200, Path("b.sav")),
        ]
        commands: list[list[str]] = []

        def runner(
            command: list[str],
            cwd: Path,
            env: dict[str, str],
        ) -> subprocess.CompletedProcess[str]:
            commands.append(command)
            return subprocess.CompletedProcess(
                args=command,
                returncode=0,
                stdout=(
                    "modern_index_compare_summary compare_count=3 "
                    "bad_count=0 bad_pixels=0 gpu_count=3 cpu_count=0\n"
                ),
            )

        status = sweep.run_sweep(
            checkpoints,
            Path("target/parity/zelda3"),
            Path("saves/zelda3.sfc"),
            Path("saves/route.sav"),
            frames=3,
            modern_index_compare=1,
            renderer=None,
            require_mode7=False,
            dry_run=False,
            runner=runner,
        )

        self.assertEqual(status, 0)
        self.assertEqual([command[4] for command in commands], ["103", "203"])

    def test_run_sweep_can_require_mode7_gpu_coverage(self) -> None:
        checkpoints = [sweep.ReplayCheckpoint(100, Path("a.sav"))]

        def runner(
            command: list[str],
            cwd: Path,
            env: dict[str, str],
        ) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                args=command,
                returncode=0,
                stdout=(
                    "modern_index_compare_summary compare_count=3 "
                    "bad_count=0 bad_pixels=0 gpu_count=3 mode7_gpu_count=0 cpu_count=0\n"
                ),
            )

        with self.assertRaises(SystemExit):
            sweep.run_sweep(
                checkpoints,
                Path("target/parity/zelda3"),
                Path("saves/zelda3.sfc"),
                Path("saves/route.sav"),
                frames=3,
                modern_index_compare=1,
                renderer=None,
                require_mode7=True,
                dry_run=False,
                runner=runner,
            )


if __name__ == "__main__":
    unittest.main()
