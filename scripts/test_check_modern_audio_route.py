#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path
from types import SimpleNamespace


SCRIPT = Path(__file__).resolve().parent / "check_modern_audio_route.py"
SPEC = importlib.util.spec_from_file_location("check_modern_audio_route", SCRIPT)
checker = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(checker)


class CheckModernAudioRouteTests(unittest.TestCase):
    def test_mismatch_ranges_keep_late_route_failures_visible(self) -> None:
        ranges: list[list[int]] = []
        for frame in [272, 273, 3002, 3003, 3010]:
            checker.append_frame_range(ranges, frame)

        self.assertEqual(ranges, [[272, 273], [3002, 3003], [3010, 3010]])
        self.assertEqual(
            checker.format_frame_ranges(ranges),
            "272..273, 3002..3003, 3010",
        )

    def test_route_command_carries_file_select_inputs_and_sram(self) -> None:
        args = SimpleNamespace(
            rust_bin=Path("zelda3"),
            rom=Path("zelda3.sfc"),
            save=Path("route.sav"),
            frames=2260,
            input_script=Path("scripts/inputs/file-select-enter-game.txt"),
            load_sram=Path("saves/sram.dat"),
            load_state=None,
            stop_replay_after_load=True,
        )

        command = checker.build_replay_command(args)

        self.assertIn("--input-script", command)
        self.assertIn("scripts/inputs/file-select-enter-game.txt", command)
        self.assertIn("--load-sram", command)
        self.assertIn("saves/sram.dat", command)
        self.assertIn("--stop-replay-after-load", command)

    def test_continuous_waveform_timeline_reports_absolute_sample_position(self) -> None:
        timeline = checker.WaveformTimeline()
        exact = {
            "frame": 1,
            "samples": 735,
            "channels": 2,
            "modern_audio": {
                "oracle_exact_samples": 1470,
                "oracle_first_mismatch_sample": None,
            },
        }
        self.assertIsNone(timeline.compare(exact))

        mismatch = {
            "frame": 2,
            "samples": 735,
            "channels": 2,
            "modern_audio": {
                "oracle_exact_samples": 1469,
                "oracle_first_mismatch_sample": 5,
            },
        }
        failure = timeline.compare(mismatch)

        self.assertIn("absolute_interleaved=1475", failure)
        self.assertIn("absolute_sample_frame=737", failure)
        self.assertIn("channel=1", failure)

    def test_stream_gate_rejects_one_wrong_sample_on_the_continuous_clock(self) -> None:
        def event(number: int, exact: int, mismatch: int | None) -> dict:
            return {
                "frame": number,
                "samples": 2,
                "channels": 2,
                "modern_note_events": [],
                "modern_audio": {
                    "peak": 0,
                    "hash": "0x00000001",
                    "active_voices": 0,
                    "ignored_events": 0,
                    "left_abs": 0,
                    "right_abs": 0,
                    "oracle_exact_samples": exact,
                    "oracle_first_mismatch_sample": mismatch,
                },
            }

        failures, _, count = checker.check_frame_stream(
            [event(40, 4, None), event(41, 3, 2)]
        )

        self.assertEqual(count, 2)
        self.assertTrue(
            any("absolute_interleaved=6" in failure for failure in failures), failures
        )

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
