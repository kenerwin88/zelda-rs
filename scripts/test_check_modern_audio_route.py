#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "check_modern_audio_route.py"
SPEC = importlib.util.spec_from_file_location("check_modern_audio_route", SCRIPT)
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)


class CheckModernAudioRouteTests(unittest.TestCase):
    def test_accepts_audible_pan_and_reports_silence(self) -> None:
        frame = {
            "frame": 10,
            "modern_note_events": [
                {"voice": 0, "pitch": 40, "instrument": 15, "volume": 20, "pan": 127}
            ],
            "modern_audio": {
                "peak": 100,
                "hash": "0x12345678",
                "active_voices": 1,
                "ignored_events": 0,
                "left_abs": 0,
                "right_abs": 1000,
            },
        }

        failures, digest = checker.check_frames([frame])
        self.assertEqual(failures, [])
        self.assertEqual(len(digest), 64)

        frame["modern_audio"]["peak"] = 0
        failures, _ = checker.check_frames([frame])
        self.assertTrue(any("rendered silence" in failure for failure in failures))

        frame["peak"] = 0
        failures, _ = checker.check_frames([frame])
        self.assertEqual(failures, [])

    def test_can_require_complete_sfx_catalog_coverage(self) -> None:
        frame = {
            "frame": 20,
            "modern_sfx_unknown": 1,
            "modern_sfx_unknown_programs": [[2, 79]],
            "modern_note_events": [],
            "modern_audio": {
                "peak": 1,
                "hash": "0x00000001",
                "active_voices": 0,
                "ignored_events": 0,
                "left_abs": 0,
                "right_abs": 0,
            },
        }

        failures, _ = checker.check_frames(
            [frame], require_zero_unknown_sfx=True
        )

        self.assertEqual(
            failures,
            ["frame=20 unknown_sfx=1 programs=[[2, 79]]"],
        )

    def test_requires_sustained_silence_for_active_voice_failure(self) -> None:
        def frame(number: int, peak: int) -> dict:
            return {
                "frame": number,
                "peak": 100,
                "modern_note_events": [],
                "modern_audio": {
                    "peak": peak,
                    "hash": "0x00000001",
                    "active_voices": 1,
                    "ignored_events": 0,
                    "left_abs": 0,
                    "right_abs": 0,
                },
            }

        failures, _ = checker.check_frames([frame(1, 0), frame(2, 1)])
        self.assertEqual(failures, [])

        failures, _ = checker.check_frames([frame(1, 0), frame(2, 0)])
        self.assertTrue(any("sustained silence" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
