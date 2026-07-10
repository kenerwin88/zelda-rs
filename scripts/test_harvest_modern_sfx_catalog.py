#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import contextlib
import io
import json
import os
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
for path in [SCRIPT_DIR]:
    if str(path) not in sys.path:
        sys.path.insert(0, str(path))

SCRIPT = SCRIPT_DIR / "harvest_modern_sfx_catalog.py"
SPEC = importlib.util.spec_from_file_location("harvest_modern_sfx_catalog", SCRIPT)
harvest_modern_sfx_catalog = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = harvest_modern_sfx_catalog
SPEC.loader.exec_module(harvest_modern_sfx_catalog)


class HarvestModernSfxCatalogTests(unittest.TestCase):
    def test_runs_broad_and_focused_replays_and_writes_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            fake_replay = write_fake_replay(tmp_path)
            output_dir = tmp_path / "harvest"

            status = run_harvester(
                "--rust-bin",
                str(fake_replay),
                "--rom",
                str(tmp_path / "zelda3.sfc"),
                "--save",
                str(tmp_path / "route.sav"),
                "--frames",
                "32",
                "--output-dir",
                str(output_dir),
            )

            self.assertEqual(status, 0)
            result = json.loads((output_dir / "modern-sfx-harvest.json").read_text())
            self.assertEqual(result["coverage"]["discovered_commands"], 1)
            self.assertEqual(result["coverage"]["focused_commands"], 1)
            self.assertEqual(result["coverage"]["lifted"], 1)
            self.assertEqual(result["coverage"]["gaps"], 0)
            self.assertEqual(result["coverage"]["focused_command_gaps"], 0)
            self.assertEqual(result["coverage"]["program_gaps"], 0)
            self.assertEqual(result["focused_traces"][0]["frame"], 10)
            self.assertTrue(Path(result["focused_traces"][0]["path"]).exists())
            self.assertIn("TRACE_SFX_00_34_STEPS", (output_dir / "modern-sfx-candidates.rs").read_text())
            self.assertIn("0x34", (output_dir / "modern-sfx-harvest.md").read_text())

    def test_reads_existing_broad_trace_and_marks_skipped_focus_as_gap(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            broad = tmp_path / "broad.jsonl"
            output_dir = tmp_path / "harvest"
            broad.write_text(
                json.dumps(trace_frame(25, queue_input=[0, 0x88, 0, 0])) + "\n",
                encoding="utf-8",
            )

            status = run_harvester(
                "--broad-trace-jsonl",
                str(broad),
                "--skip-focused-runs",
                "--output-dir",
                str(output_dir),
                "--fail-on-gaps",
            )

            self.assertEqual(status, 1)
            result = json.loads((output_dir / "modern-sfx-harvest.json").read_text())
            self.assertEqual(result["coverage"]["missing_focused_trace"], 1)
            self.assertEqual(result["coverage"]["gaps"], 1)
            self.assertEqual(result["coverage"]["focused_command_gaps"], 1)
            self.assertEqual(result["coverage"]["program_gaps"], 1)
            self.assertEqual(result["programs"][0]["status"], "missing_focused_trace")

    def test_reuses_cached_focused_trace_unless_forced(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            fake_replay = write_fake_replay(tmp_path)
            output_dir = tmp_path / "harvest"
            focused_dir = output_dir / "focused"
            focused_dir.mkdir(parents=True)
            cached = focused_dir / "frame-00000010-sfx-00-34.jsonl"
            cached.write_text(json.dumps(focused_trace_frame(10, volume=0x20)) + "\n", encoding="utf-8")

            status = run_harvester(
                "--rust-bin",
                str(fake_replay),
                "--rom",
                str(tmp_path / "zelda3.sfc"),
                "--save",
                str(tmp_path / "route.sav"),
                "--frames",
                "32",
                "--output-dir",
                str(output_dir),
            )

            self.assertEqual(status, 0)
            result = json.loads((output_dir / "modern-sfx-harvest.json").read_text())
            self.assertTrue(result["focused_traces"][0]["reused"])
            self.assertEqual(result["programs"][0]["steps"][0]["volume"], 32)


def write_fake_replay(tmp_path: Path) -> Path:
    script = tmp_path / "fake_zelda3.py"
    script.write_text(
        textwrap.dedent(
            """\
            #!/usr/bin/env python3
            import json
            import os
            target = os.environ.get("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME")
            target_range = os.environ.get("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME_RANGE")
            if target is None and target_range is None:
                frames = [
                    {"frame": 8, "apui": [0, 0, 0, 0], "music": [0, 0, 0], "queue": {"input": [0, 0, 0, 0]}},
                    {"frame": 10, "apui": [0, 0, 0, 0], "music": [0, 0, 0], "queue": {"input": [0, 52, 0, 0]}},
                    {"frame": 11, "apui": [0, 0, 0, 0], "music": [0, 0, 0], "queue": {"input": [0, 52, 0, 0]}},
                    {"frame": 12, "apui": [0, 0, 0, 0], "music": [0, 0, 0], "queue": {"input": [0, 0, 0, 0]}},
                ]
            else:
                if target_range is not None:
                    start, end = [int(part) for part in target_range.split(":")]
                else:
                    start = int(target)
                    end = start
                frames = [
                    {
                        "frame": start,
                        "apui": [0, 0, 0, 0],
                        "music": [0, 0, 0],
                        "queue": {"input": [0, 52, 0, 0]},
                        "dsp_write_events": [
                            [16, 64, 0, 0],
                            [17, 192, 0, 0],
                            [18, 0, 0, 0],
                            [19, 8, 0, 0],
                            [20, 2, 0, 0],
                            [21, 143, 0, 0],
                            [22, 228, 0, 0],
                            [61, 2, 0, 0],
                            [76, 2, 0, 0],
                        ],
                    },
                    {
                        "frame": min(start + 5, end),
                        "apui": [0, 0, 0, 0],
                        "music": [0, 0, 0],
                        "queue": {"input": [0, 52, 0, 0]},
                        "dsp_write_events": [[92, 2, 0, 0]],
                    },
                ]
            for frame in frames:
                print(json.dumps(frame))
            """
        ),
        encoding="utf-8",
    )
    script.chmod(0o755)
    return script


def run_harvester(*args: str) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return harvest_modern_sfx_catalog.main(list(args))


def trace_frame(frame_no: int, *, queue_input: list[int]) -> dict:
    return {
        "frame": frame_no,
        "apui": [0, 0, 0, 0],
        "music": [0, 0, 0],
        "queue": {"input": queue_input},
    }


def focused_trace_frame(frame_no: int, *, volume: int) -> dict:
    return {
        "frame": frame_no,
        "apui": [0, 0, 0, 0],
        "music": [0, 0, 0],
        "queue": {"input": [0, 0x34, 0, 0]},
        "dsp_write_events": [
            [0x10, volume, 0, 0],
            [0x11, 0, 0, 0],
            [0x12, 0, 0, 0],
            [0x13, 8, 0, 0],
            [0x4C, 2, 0, 0],
        ],
    }


if __name__ == "__main__":
    unittest.main()
