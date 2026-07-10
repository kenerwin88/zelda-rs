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
        self.assertEqual(step["envelope"], {"attack": 15, "decay": 0, "sustain": 7, "release": 4})
        self.assertEqual(step["duration_frames"], 5)
        self.assertEqual(step["pitch_slide"]["target_pitch"], 80)
        self.assertEqual(step["pitch_slide"]["frames"], 3)

    def test_reports_missing_dsp_events_for_unfocused_trace_window(self) -> None:
        trace = [frame(50, queue_input=[0, 0x88, 0, 0])]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["missing_dsp_events"], 1)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "missing_dsp_events")
        self.assertIn("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME=50", program["notes"][0])

    def test_marks_same_command_with_distinct_lifted_programs_ambiguous(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
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
                    [0x4C, 0x02, 0, 0],
                    [0x5C, 0x02, 0, 0],
                ],
            ),
        ]

        catalog = extract_modern_sfx_catalog.extract_catalog(trace, window_frames=4)

        self.assertEqual(catalog["coverage"]["ambiguous"], 1)
        program = catalog["programs"][0]
        self.assertEqual(program["status"], "ambiguous")
        self.assertEqual(len(program["variants"]), 2)

    def test_repeated_same_program_at_different_frames_is_not_ambiguous(self) -> None:
        trace = [
            frame(
                10,
                queue_input=[0, 0x22, 0, 0],
                dsp_write_events=[
                    [0x12, 0x00, 0, 0],
                    [0x13, 0x08, 0, 0],
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
    return value


if __name__ == "__main__":
    unittest.main()
