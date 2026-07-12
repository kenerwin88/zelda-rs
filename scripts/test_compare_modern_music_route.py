#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
SPEC = importlib.util.spec_from_file_location(
    "compare_modern_music_route", SCRIPT_DIR / "compare_modern_music_route.py"
)
comparator = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(comparator)


class CompareModernMusicRouteTests(unittest.TestCase):
    def test_reports_first_note_mismatch_and_accepts_exact_match(self) -> None:
        frame = {
            "frame": 10,
            "music": [0, 0, 1],
            "dsp_write_events": [
                [0x00, 23, 0, 0],
                [0x01, 23, 0, 0],
                [0x02, 0x80, 0, 0],
                [0x03, 0x1A, 0, 0],
                [0x04, 15, 0, 0],
                [0x4C, 1, 0, 0],
            ],
            "modern_note_events": [
                {"voice": 0, "pitch": 53, "instrument": 15, "volume": 23, "pan": 0}
            ],
        }

        self.assertEqual(comparator.compare_frames([frame], {1}), [])

        frame["modern_note_events"][0]["pitch"] = 52
        failures = comparator.compare_frames([frame], {1})
        self.assertEqual(len(failures), 1)
        self.assertIn("mismatch_index=0", failures[0])


if __name__ == "__main__":
    unittest.main()
