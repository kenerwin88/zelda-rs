#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import tempfile
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

    def test_event_sfx_mask_takes_precedence_over_frame_ownership(self) -> None:
        frames = [
            {
                "frame": 20,
                "music": [0, 0, 11],
                "dsp_write_events": [
                    [0x02, 0, 0, 0, 0],
                    [0x03, 8, 0, 0, 0],
                    [0x12, 0, 0, 0, 0x02],
                    [0x13, 8, 0, 0, 0x02],
                    [0x4C, 0x03, 0, 0, 0x02],
                ],
            }
        ]

        notes = extractor.extract_music(frames)["tracks"][0]["notes"]

        self.assertEqual([note["voice"] for note in notes], [0])

    def test_updates_only_selected_global_track(self) -> None:
        catalog = {
            "tracks": [
                {
                    "track": 1,
                    "global_events": [
                        {
                            "dsp_cycle": 112_436,
                            "register": 0x4D,
                            "value": 2,
                        }
                    ],
                }
            ]
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "globals.tsv"
            path.write_text(
                "# track\tstart_frame\tsample_offset\tregister\tvalue\n"
                "01\t0\t0\t4d\t0\n"
                "02\t3\t4\t2c\t5\n",
                encoding="utf-8",
            )

            extractor.update_globals_tsv(path, catalog)

            self.assertEqual(
                path.read_text(encoding="utf-8"),
                "# track\tdsp_cycle\tregister\tvalue\n"
                "01\t112436\t4d\t2\n"
                "02\t51392\t2c\t5\n",
            )

    def test_global_cycles_start_at_the_music_command_frame(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "music": [1, 0, 0],
                "dsp_write_events": [],
            },
            {
                "frame": 11,
                "audio_sample_frames": 534,
                "music": [0, 0, 1],
                "dsp_write_events": [[0x3C, 21, 10, 20]],
            },
        ]

        event = extractor.extract_music(frames)["tracks"][0]["global_events"][0]

        self.assertEqual(event["dsp_cycle"], (533 + 10) * 32 + 20)
        self.assertNotIn("start_frame", event)
        self.assertNotIn("sample_offset", event)
        self.assertNotIn("dsp_phase", event)

    def test_sfx_owned_echo_bit_does_not_pollute_music_globals(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 534,
                "music": [1, 0, 0],
                "dsp_write_events": [
                    {
                        "register": 0x4D,
                        "value": 0xFF,
                        "legacy_sample_offset": 0,
                        "phase": 0,
                        "sfx_voice_mask": 0,
                    }
                ],
            },
            {
                "frame": 11,
                "audio_sample_frames": 534,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x4D,
                        "value": 0x7F,
                        "legacy_sample_offset": 10,
                        "phase": 27,
                        "sfx_voice_mask": 0x80,
                    }
                ],
            },
            {
                "frame": 12,
                "audio_sample_frames": 534,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x4D,
                        "value": 0xFF,
                        "legacy_sample_offset": 20,
                        "phase": 27,
                        "sfx_voice_mask": 0,
                    }
                ],
            },
        ]

        events = extractor.extract_music(frames)["tracks"][0]["global_events"]

        self.assertEqual(
            [(event["register"], event["value"]) for event in events],
            [(0x4D, 0xFF)],
        )

    def test_global_cycles_use_monotonic_dsp_clock_across_frame_boundaries(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "dsp_clock": {
                    "first_output_cycle": 10_027,
                    "last_output_cycle": 27_051,
                    "output_count": 533,
                },
                "music": [1, 0, 0],
                "dsp_write_events": [],
            },
            {
                "frame": 11,
                "audio_sample_frames": 534,
                "dsp_clock": {
                    "first_output_cycle": 27_083,
                    "last_output_cycle": 44_139,
                    "output_count": 534,
                },
                "music": [0, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x3C,
                        "value": 21,
                        # This legacy counter is deliberately wrong. The
                        # monotonic clock is the authoritative coordinate.
                        "legacy_sample_offset": 99,
                        "phase": 20,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 27_396,
                    }
                ],
            },
        ]

        event = extractor.extract_music(frames)["tracks"][0]["global_events"][0]

        self.assertEqual(event["dsp_cycle"], 543 * 32 + 20)

    def test_monotonic_dsp_clock_rejects_inconsistent_hardware_phase(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "dsp_clock": {
                    "first_output_cycle": 10_027,
                    "last_output_cycle": 27_051,
                    "output_count": 533,
                },
                "music": [1, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x3C,
                        "value": 21,
                        "legacy_sample_offset": 0,
                        "phase": 20,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 10_021,
                    }
                ],
            }
        ]

        with self.assertRaisesRegex(ValueError, "DSP event clock/phase mismatch"):
            extractor.extract_music(frames)

    def test_callback_origin_uses_delivered_stream_when_raw_dsp_is_buffered(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "dsp_clock": {
                    "first_output_cycle": 10_027,
                    "last_output_cycle": 27_115,
                    "output_count": 535,
                },
                "music": [1, 0, 0],
                "dsp_write_events": [],
            },
            {
                "frame": 11,
                "audio_sample_frames": 533,
                "dsp_clock": {
                    "first_output_cycle": 27_147,
                    "last_output_cycle": 44_107,
                    "output_count": 531,
                },
                "music": [0, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x3C,
                        "value": 21,
                        "legacy_sample_offset": 0,
                        "phase": 20,
                        "sfx_voice_mask": 0,
                        # The write occurs after the two raw samples buffered
                        # at the end of frame 10.
                        "absolute_cycle": 27_172,
                    }
                ],
            },
        ]

        event = extractor.extract_music(frames)["tracks"][0]["global_events"][0]

        self.assertEqual(event["dsp_cycle"], 536 * 32 + 20)

    def test_note_keyon_and_keyoff_use_monotonic_dsp_samples(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "dsp_clock": {
                    "first_output_cycle": 10_027,
                    "last_output_cycle": 27_051,
                    "output_count": 533,
                },
                "music": [1, 0, 1],
                "dsp_write_events": [
                    {
                        "register": 0x02,
                        "value": 0,
                        "legacy_sample_offset": 90,
                        "phase": 4,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 10_004,
                    },
                    {
                        "register": 0x03,
                        "value": 8,
                        "legacy_sample_offset": 91,
                        "phase": 5,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 10_005,
                    },
                    {
                        "register": 0x4C,
                        "value": 1,
                        "legacy_sample_offset": 99,
                        "phase": 7,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 10_327,
                    },
                    {
                        "register": 0x5C,
                        "value": 1,
                        "legacy_sample_offset": 199,
                        "phase": 9,
                        "sfx_voice_mask": 0,
                        "absolute_cycle": 10_649,
                    },
                ],
            }
        ]

        note = extractor.extract_music(frames)["tracks"][0]["notes"][0]

        self.assertEqual(note["sample_offset"], 10)
        self.assertEqual(note["keyoff_sample_offset"], 20)

    def test_variable_audio_windows_become_continuous_sample_positions(self) -> None:
        frames = [
            {
                "frame": 10,
                "audio_sample_frames": 533,
                "music": [0, 0, 1],
                "dsp_write_events": [],
            },
            {
                "frame": 11,
                "audio_sample_frames": 534,
                "music": [0, 0, 1],
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x4C, 1, 10, 0],
                ],
            },
        ]

        track = extractor.extract_music(frames)["tracks"][0]
        note = track["notes"][0]

        self.assertEqual(track["lead_in_frames"], 1)
        self.assertEqual(note["start_frame"], 0)
        self.assertEqual(note["sample_offset"], 9)

    def test_ignores_keyons_before_the_music_track_is_known(self) -> None:
        frames = [
            {
                "frame": 10,
                "music": [0, 0, 0],
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x4C, 1, 10, 0],
                ],
            },
            {
                "frame": 11,
                "music": [1, 0, 1],
                "dsp_write_events": [
                    [0x02, 0, 0, 0],
                    [0x03, 8, 0, 0],
                    [0x4C, 1, 20, 0],
                ],
            },
        ]

        catalog = extractor.extract_music(frames)

        self.assertEqual([track["track"] for track in catalog["tracks"]], [1])
        self.assertEqual(len(catalog["tracks"][0]["notes"]), 1)
        self.assertEqual(catalog["tracks"][0]["notes"][0]["sample_offset"], 20)

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
