#!/usr/bin/env python3
"""Tests for oracle-window render compare helpers."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
from contextlib import redirect_stdout
from io import StringIO

from gpu_render_compare_oracle_windows import (
    MODERN_INDEX_PROGRESS_RE,
    OracleWindow,
    env_for_renderer,
    run_command_capture_output,
    selected_windows,
)


class GpuRenderCompareOracleWindowsTests(unittest.TestCase):
    def test_capture_output_does_not_wait_for_stdout_inheriting_grandchild(self) -> None:
        code = (
            "import subprocess, sys; "
            "subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(2)']); "
            "print('done')"
        )

        started = time.monotonic()
        result = run_command_capture_output(
            [sys.executable, "-c", code],
            cwd=Path.cwd(),
            env=os.environ.copy(),
        )
        elapsed = time.monotonic() - started

        self.assertEqual(result.returncode, 0)
        self.assertIn("done", result.stdout)
        self.assertLess(elapsed, 1.5)

    def test_capture_output_can_stream_progress_from_tempfile(self) -> None:
        code = (
            "print('hidden noise', flush=True); "
            "print('modern_index_compare_progress compare_count=10 frame=20 bad_count=0', flush=True); "
            "print('done', flush=True)"
        )

        live = StringIO()
        with redirect_stdout(live):
            result = run_command_capture_output(
                [sys.executable, "-c", code],
                cwd=Path.cwd(),
                env=os.environ.copy(),
                live_patterns=(MODERN_INDEX_PROGRESS_RE,),
                poll_seconds=0.01,
            )

        self.assertEqual(result.returncode, 0)
        self.assertIn("hidden noise", result.stdout)
        self.assertIn("done", result.stdout)
        self.assertIn("modern_index_compare_progress compare_count=10", live.getvalue())
        self.assertNotIn("hidden noise", live.getvalue())

    def test_variant_gpu_env_enables_summary_and_progress(self) -> None:
        env = env_for_renderer(
            {"KEEP": "1"},
            renderer="assets-variant-gpu",
            progress_every=2500,
        )

        self.assertEqual(env["KEEP"], "1")
        self.assertEqual(env["ZELDA3_RENDERER"], "assets-variant-gpu")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"], "1")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_PROGRESS"], "2500")

    def test_progress_can_be_disabled(self) -> None:
        env = env_for_renderer(
            {},
            renderer="assets-variant-gpu",
            progress_every=0,
        )

        self.assertEqual(env["ZELDA3_RENDERER"], "assets-variant-gpu")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"], "1")
        self.assertNotIn("ZELDA3_MODERN_INDEX_COMPARE_PROGRESS", env)

    def test_no_input_windows_are_not_treated_as_sram_windows(self) -> None:
        windows = [
            OracleWindow(
                name="no-input-intro",
                status="pass",
                frames=1,
                input_script="",
                coverage="intro",
                notes="",
            )
        ]

        selected = selected_windows(
            windows,
            only=[],
            max_frames=None,
            include_sram_windows=False,
        )

        self.assertEqual([window.name for window in selected], ["no-input-intro"])


if __name__ == "__main__":
    unittest.main()
