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
    def test_sfx_lockstep_requires_sample_events_and_voice_ownership(self) -> None:
        frame = {
            "frame": 356450,
            "classic_sfx_events": [
                ["on", 104, 7, 6, 254, 106, 112, 31, 31],
                ["pitch", 104, 7, 3564],
            ],
            "modern_sfx_events": [
                ["on", 360, 7, 6, 254, 106, 112, 31, 31],
                ["pitch", 360, 7, 3606],
            ],
            "classic_sfx_voice_mask": 0x80,
            "modern_sfx_voice_mask": 0x80,
            "modern_note_events": [],
            "modern_audio": {
                "peak": 1,
                "hash": "0x00000001",
                "active_voices": 1,
                "ignored_events": 0,
                "left_abs": 1,
                "right_abs": 1,
            },
        }

        failures, _ = checker.check_frames([frame], require_sfx_lockstep=True)
        self.assertEqual(len(failures), 1)
        self.assertIn("SFX event lockstep", failures[0])
        self.assertIn("offset=104", failures[0])
        self.assertIn("offset=360", failures[0])

        frame["modern_sfx_events"] = list(frame["classic_sfx_events"])
        failures, _ = checker.check_frames([frame], require_sfx_lockstep=True)
        self.assertEqual(failures, [])

        frame["modern_sfx_voice_mask"] = 0
        failures, _ = checker.check_frames([frame], require_sfx_lockstep=True)
        self.assertEqual(
            failures,
            ["frame=356450 SFX ownership mismatch classic=0x80 modern=0x00"],
        )

    def test_sfx_lockstep_rejects_traces_without_exact_receipts(self) -> None:
        frame = {
            "frame": 1,
            "modern_note_events": [],
            "modern_audio": {
                "peak": 0,
                "hash": "0x00000001",
                "active_voices": 0,
                "ignored_events": 0,
                "left_abs": 0,
                "right_abs": 0,
            },
        }

        failures, _ = checker.check_frames([frame], require_sfx_lockstep=True)
        self.assertEqual(
            failures,
            ["frame=1 missing exact SFX lockstep receipts"],
        )

    def test_sfx_lockstep_compares_audible_dsp_voice_state(self) -> None:
        frame = {
            "frame": 2,
            "classic_sfx_events": [["on", 10, 0, 6, 254, 106, 112, 31, 31]],
            "modern_sfx_events": [["on", 10, 0, 6, 254, 106, 112, 31, 31]],
            "classic_sfx_voice_mask": 1,
            "modern_sfx_voice_mask": 1,
            "dsp_voices": [
                {"pitch": 3525, "gain": 1215, "state": 1, "rate_counter": 1, "volume": [31, 31]}
            ],
            "modern_voices": [
                {"pitch": 3606, "gain": 1215, "state": 1, "rate_counter": 1, "volume": [31, 31]}
            ],
            "modern_note_events": [],
            "modern_audio": {
                "peak": 1,
                "hash": "0x00000001",
                "active_voices": 1,
                "ignored_events": 0,
                "left_abs": 1,
                "right_abs": 1,
            },
        }

        failures, _ = checker.check_frames([frame], require_sfx_lockstep=True)
        self.assertEqual(
            failures,
            [
                "frame=2 voice=0 SFX DSP state mismatch "
                "field=pitch classic=3525 modern=3606"
            ],
        )

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

    def test_rejects_voice_that_outlives_c_keyoff(self) -> None:
        def frame(number: int, classic_active: bool, modern_active: bool) -> dict:
            return {
                "frame": number,
                "peak": 0,
                "modern_note_events": [],
                "dsp_voices": [
                    {
                        "gain": 512 if classic_active else 0,
                        "sample": 20 if classic_active else 0,
                    },
                ],
                "modern_voices": [
                    {
                        "active": modern_active,
                        "gain": 512 if modern_active else 0,
                        "sample": 20 if modern_active else 0,
                    }
                ],
                "modern_audio": {
                    "peak": 20 if modern_active else 0,
                    "hash": "0x00000001",
                    "active_voices": int(modern_active),
                    "ignored_events": 0,
                    "left_abs": 20 if modern_active else 0,
                    "right_abs": 20 if modern_active else 0,
                },
            }

        failures, _ = checker.check_frames(
            [
                frame(1, True, True),
                frame(2, False, True),
                frame(3, False, True),
            ]
        )
        self.assertEqual(failures, [])

        failures, _ = checker.check_frames(
            [
                frame(1, True, True),
                frame(2, False, True),
                frame(3, False, True),
                frame(4, False, True),
            ]
        )
        self.assertEqual(
            failures,
            ["frame=2 voice=0 remained audible after C key-off"],
        )


if __name__ == "__main__":
    unittest.main()
