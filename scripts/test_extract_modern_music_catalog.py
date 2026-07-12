#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "extract_modern_music_catalog.py"
SPEC = importlib.util.spec_from_file_location("extract_modern_music_catalog", SCRIPT)
extractor = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = extractor
SPEC.loader.exec_module(extractor)


class ExtractModernMusicCatalogTests(unittest.TestCase):
    def test_leaves_notes_without_an_observed_keyoff_unbounded(self) -> None:
        frames = [
            {
                "frame": 10,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    [0x02, 0, 12, 0],
                    [0x03, 8, 12, 0],
                    [0x4C, 0x01, 12, 0],
                ],
            }
        ]

        note = extractor.extract_music(frames)["tracks"][0]["notes"][0]

        self.assertEqual(note["duration_frames"], 0)
        self.assertEqual(note["keyoff_sample_offset"], 0)

    def test_preserves_same_frame_keyoff_as_zero_frame_duration(self) -> None:
        frames = [
            {
                "frame": 10,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    [0x02, 0, 12, 0],
                    [0x03, 8, 12, 0],
                    [0x4C, 0x01, 12, 0],
                    [0x5C, 0x01, 448, 0],
                ],
            }
        ]

        note = extractor.extract_music(frames)["tracks"][0]["notes"][0]

        self.assertEqual(note["duration_frames"], 0)
        self.assertEqual(note["sample_offset"], 12)
        self.assertEqual(note["keyoff_sample_offset"], 448)

    def test_lifts_polyphonic_keyons_and_keyoff_durations(self) -> None:
        frames = [
            {
                "frame": 10,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    [0x00, 40, 0, 0],
                    [0x01, 20, 0, 0],
                    [0x02, 0x00, 0, 0],
                    [0x03, 0x10, 0, 0],
                    [0x04, 3, 0, 0],
                    [0x10, 20, 0, 0],
                    [0x11, 40, 0, 0],
                    [0x12, 0x80, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 4, 0, 0],
                    [0x4C, 0x03, 12, 0],
                ],
            },
            {
                "frame": 14,
                "music": [0, 0, 1],
                "dsp_write_events": [[0x5C, 0x01, 0, 0]],
            },
            {
                "frame": 16,
                "music": [0, 0, 1],
                "dsp_write_events": [[0x5C, 0x02, 0, 0]],
            },
        ]

        catalog = extractor.extract_music(frames)

        self.assertEqual(len(catalog["tracks"]), 1)
        notes = catalog["tracks"][0]["notes"]
        self.assertEqual([note["voice"] for note in notes], [0, 1])
        self.assertEqual([note["duration_frames"] for note in notes], [4, 6])
        self.assertEqual([note["pan"] for note in notes], [-64, 64])
        self.assertEqual([note["start_frame"] for note in notes], [0, 0])
        self.assertEqual(notes[0]["pitch"], 32)
        self.assertEqual(notes[1]["pitch"], 17)

    def test_excludes_keyons_owned_by_active_sfx_channels(self) -> None:
        frames = [
            {
                "frame": 20,
                "music": [0, 0, 11],
                "sfx_channels": [{"voice": 1, "sound": 0x2C}],
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x12, 0, 0, 0],
                    [0x13, 8, 0, 0],
                    [0x4C, 0x03, 0, 0],
                ],
            }
        ]

        notes = extractor.extract_music(frames)["tracks"][0]["notes"]

        self.assertEqual([note["voice"] for note in notes], [0])

    def test_excludes_keyons_owned_by_nested_queue_sfx_channels(self) -> None:
        frames = [
            {
                "frame": 20,
                "music": [0, 0, 11],
                "queue": {"sfx_channels": [{"voice": 1, "sound": 0x2C}]},
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x12, 0, 0, 0],
                    [0x13, 8, 0, 0],
                    [0x4C, 0x03, 0, 0],
                ],
            }
        ]

        notes = extractor.extract_music(frames)["tracks"][0]["notes"]

        self.assertEqual([note["voice"] for note in notes], [0])

    def test_annotates_notes_near_sfx_commands_without_dropping_them(self) -> None:
        frames = [
            {
                "frame": 30,
                "music": [0, 0, 3],
                "modern_sfx_unknown": 1,
                "dsp_write_events": [],
            },
            {
                "frame": 32,
                "music": [0, 0, 3],
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x4C, 0x01, 0, 0],
                ],
            },
        ]

        notes = extractor.extract_music(frames)["tracks"][0]["notes"]

        self.assertEqual(len(notes), 1)
        self.assertEqual(notes[0]["preceding_sfx_command_frame"], 30)
        self.assertEqual(notes[0]["frames_after_sfx_command"], 2)


if __name__ == "__main__":
    unittest.main()
