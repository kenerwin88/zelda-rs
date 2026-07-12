#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "extract_modern_sfx_catalog.py"
SPEC = importlib.util.spec_from_file_location("extract_modern_sfx_catalog", SCRIPT)
extract_modern_sfx_catalog = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = extract_modern_sfx_catalog
SPEC.loader.exec_module(extract_modern_sfx_catalog)


class ExtractModernSfxCatalogTests(unittest.TestCase):
    def test_repeated_commands_are_bounded_before_the_next_transition(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0, 0x0C, 0],
                dsp_write_events=[
                    [0x52, 0x40, 0, 0],
                    [0x53, 0x05, 0, 0],
                    [0x54, 0x0D, 0, 0],
                    [0x55, 0xF9, 0, 0],
                    [0x56, 0x6E, 0, 0],
                    [0x57, 0xB8, 0, 0],
                    [0x4C, 0x20, 0, 0],
                ],
            ),
            frame(11, queue_input=[0, 0, 0, 0], dsp_write_events=[]),
            frame(
                13,
                queue_input=[0, 0, 0x0C, 0],
                dsp_write_events=[
                    [0x52, 0x80, 0, 0],
                    [0x53, 0x05, 0, 0],
                    [0x54, 0x0D, 0, 0],
                    [0x55, 0xF9, 0, 0],
                    [0x56, 0x6E, 0, 0],
                    [0x57, 0xB8, 0, 0],
                    [0x4C, 0x20, 0, 0],
                ],
            ),
        ]

        occurrences = extract_modern_sfx_catalog.discover_sfx_occurrences(trace)
        first = extract_modern_sfx_catalog.lift_occurrence(trace, occurrences[0], 20)

        self.assertEqual(occurrences[0].next_command_frame, 13)
        self.assertEqual(first["trace_frames"], [10, 11])
        self.assertEqual(len(first["steps"]), 1)

    def test_lifts_sfx_command_dsp_writes_into_modern_program_step(self) -> None:
        trace = [
            frame(
                100,
                queue_input=[0, 0x34, 0, 0],
                dsp_write_events=[
                    [0x10, 0x40, 0, 0],
                    [0x11, 0xC0, 0, 0],
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x3D, 0x02, 0, 0],
                    [0x4D, 0x02, 0, 0],
                    [0x4C, 0x02, 0, 0],
                ],
            ),
            frame(
                103,
                queue_input=[0, 0x34, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x0A, 0, 0],
                ],
            ),
            frame(
                105,
                queue_input=[0, 0x34, 0, 0],
                dsp_write_events=[[0x5C, 0x02, 0, 0]],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=8)

        self.assertEqual(catalog["coverage"]["lifted"], 1)
        self.assertEqual(catalog["coverage"]["missing_dsp_events"], 0)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "lifted")
        self.assertEqual(program["bank"], 0)
        self.assertEqual(program["id"], 0x34)
        self.assertEqual(program["source"], "queue.input[1]")
        self.assertEqual(program["first_frames"], [100])
        step = program["steps"][0]
        self.assertEqual(step["voice"], 1)
        self.assertEqual(step["pitch"], 64)
        self.assertEqual(step["instrument"], 2)
        self.assertEqual(step["waveform"], "Noise")
        self.assertEqual(step["volume"], 64)
        self.assertEqual(step["pan"], 0)
        self.assertTrue(step["echo"])
        self.assertEqual(step["envelope"], {"attack": 15, "decay": 0, "sustain": 7, "release": 4})
        self.assertEqual(step["duration_frames"], 5)
        self.assertEqual(step["pitch_slide"]["target_pitch"], 80)
        self.assertEqual(step["pitch_slide"]["frames"], 3)
        self.assertEqual(step["ownership"], "owned_by_command")
        self.assertIn("source", step["owned_parameters"])
        self.assertEqual(program["sequence_provenance"], [])

    def test_modern_pan_preserves_route_derived_stereo_position(self) -> None:
        hard_left = extract_modern_sfx_catalog.VoiceState(volume_left=50, volume_right=0)
        center = extract_modern_sfx_catalog.VoiceState(volume_left=50, volume_right=206)
        hard_right = extract_modern_sfx_catalog.VoiceState(volume_left=0, volume_right=50)

        self.assertEqual(extract_modern_sfx_catalog.modern_pan(hard_left), -127)
        self.assertEqual(extract_modern_sfx_catalog.modern_pan(center), 0)
        self.assertEqual(extract_modern_sfx_catalog.modern_pan(hard_right), 127)

    def test_lifts_sequence_provenance_from_spc_sfx_channel_snapshot(self) -> None:
        trace = [
            frame(10, queue_input=[0, 0x34, 0, 0]),
            frame(
                12,
                queue_input=[0, 0x34, 0, 0],
                dsp_write_events=[
                    [0x10, 0x40, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x4C, 0x02, 0, 0],
                ],
                sfx_channels=[
                    {"voice": 1, "sound": 0x34, "sound_ptr": 0x2100, "pan": 0},
                ],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)
        program = catalog["programs"][0]

        self.assertEqual(program["sequence_provenance"][0]["rom_sequence_address"], 0x2100)
        self.assertEqual(program["sequence_provenance"][0]["source_kind"], "spc_sfx_channel")
        self.assertEqual(program["sequence_provenance"][0]["voice"], 1)

    def test_spc_channel_voice_excludes_strong_writes_from_other_voices(self) -> None:
        trace = [
            frame(10, queue_input=[0, 0x34, 0, 0]),
            frame(
                12,
                queue_input=[0, 0x34, 0, 0],
                dsp_write_events=[
                    [0x14, 2, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x4C, 0x01, 0, 0],
                    [0x54, 20, 0, 0],
                    [0x55, 0x8E, 0, 0],
                    [0x56, 0xE0, 0, 0],
                    [0x4C, 0x20, 0, 0],
                ],
                sfx_channels=[
                    {"voice": 5, "sound": 0x34, "sound_ptr": 0x2100, "pan": 0},
                ],
            ),
        ]

        program = extract_modern_sfx_catalog.extract_catalog(trace, 4)["programs"][0]

        self.assertEqual([step["voice"] for step in program["steps"]], [5])
        self.assertEqual([step["voice"] for step in program["context_steps"]], [0])

    def test_reports_missing_dsp_events_for_unfocused_trace_window(self) -> None:
        trace = [frame(50, queue_input=[0, 0x88, 0, 0])]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["missing_dsp_events"], 1)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "missing_dsp_events")
        self.assertIn("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME=50", program["notes"][0])

    def test_groups_same_command_with_distinct_lifted_programs_as_variants(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x02, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
            frame(12, queue_input=[0, 0, 0, 0], dsp_write_events=[]),
            frame(
                20,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x0A, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x02, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["ambiguous"], 0)
        self.assertEqual(catalog["coverage"]["lifted"], 1)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "lifted")
        self.assertEqual(program["variant_count"], 2)
        self.assertEqual(len(program["variants"]), 2)
        self.assertNotEqual(program["variants"][0]["variant_hash"], program["variants"][1]["variant_hash"])
        self.assertEqual(program["variants"][0]["name"], "trace_sfx_00_22_v00")
        self.assertEqual(program["variants"][1]["name"], "trace_sfx_00_22_v01")

    def test_repeated_same_program_at_different_frames_is_not_ambiguous(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x02, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
            frame(12, queue_input=[0, 0, 0, 0], dsp_write_events=[]),
            frame(
                20,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x02, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["ambiguous"], 0)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "lifted")
        self.assertEqual(program["occurrences"], 2)
        self.assertEqual(program["first_frames"], [10, 20])

    def test_carried_over_context_voices_do_not_make_program_ambiguous(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x44, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x0A, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
            frame(12, queue_input=[0, 0, 0, 0], dsp_write_events=[]),
            frame(
                20,
                queue_input=[0, 0x44, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x14, 0x02, 0, 0],
                    [0x15, 0x8F, 0, 0],
                    [0x16, 0xE4, 0, 0],
                    [0x17, 0xB8, 0, 0],
                    [0x4C, 0x12, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["ambiguous"], 0)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "lifted")
        self.assertEqual(len(program["steps"]), 1)
        self.assertEqual(len(program["context_steps"]), 2)
        self.assertEqual(program["voice_ownership"]["carried_over"], 2)

    def test_key_on_without_owned_setup_is_context_only(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x55, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
                    [0x4C, 0x02, 0, 0],
                ],
            )
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["context_only"], 1)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "context_only")
        self.assertEqual(program["steps"], [])

    def test_cli_writes_json_and_reviewable_rust_snippets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            trace_path = tmp_path / "trace.jsonl"
            json_path = tmp_path / "catalog.json"
            rust_path = tmp_path / "catalog.rs"
            trace_path.write_text(
                json.dumps(
                    frame(
                        7,
                        queue_input=[0, 0x01, 0, 0],
                        dsp_write_events=[
                            [0x12, 0x00, 0, 0],
                            [0x13, 0x08, 0, 0],
                            [0x14, 0x02, 0, 0],
                            [0x15, 0x8F, 0, 0],
                            [0x16, 0xE4, 0, 0],
                            [0x17, 0xB8, 0, 0],
                            [0x4C, 0x02, 0, 0],
                        ],
                    )
                )
                + "\n",
                encoding="utf-8",
            )

            status = extract_modern_sfx_catalog.main(
                [
                    str(trace_path),
                    "--json-out",
                    str(json_path),
                    "--rust-out",
                    str(rust_path),
                ]
            )

            self.assertEqual(status, 0)
            self.assertEqual(json.loads(json_path.read_text(encoding="utf-8"))["coverage"]["lifted"], 1)
            rust = rust_path.read_text(encoding="utf-8")
            self.assertIn("TRACE_SFX_00_01_STEPS", rust)
            self.assertIn("ModernSfxProgram", rust)


def frame(
    frame_no: int,
    *,
    queue_input: list[int],
    dsp_write_events: list[list[int]] | None = None,
    sfx_channels: list[dict] | None = None,
    sequence_provenance: list[dict] | None = None,
) -> dict:
    value = {
        "frame": frame_no,
        "apui": [0, 0, 0, 0],
        "music": [0, 0, 0],
        "queue": {
            "pos": 0,
            "count": 0,
            "total": 0,
            "write": [0, 0, 0, 0],
            "pending": [0, 0, 0, 0],
            "input": queue_input,
        },
    }
    if dsp_write_events is not None:
        value["dsp_write_events"] = dsp_write_events
    if sfx_channels is not None:
        value["queue"]["sfx_channels"] = sfx_channels
    if sequence_provenance is not None:
        value["sequence_provenance"] = sequence_provenance
    return value


if __name__ == "__main__":
    unittest.main()
