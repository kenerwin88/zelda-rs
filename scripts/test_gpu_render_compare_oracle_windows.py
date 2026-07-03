#!/usr/bin/env python3
"""Tests for oracle-window render compare helpers."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import time
import unittest

from gpu_render_compare_oracle_windows import run_command_capture_output


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


if __name__ == "__main__":
    unittest.main()
