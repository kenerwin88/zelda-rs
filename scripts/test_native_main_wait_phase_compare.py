#!/usr/bin/env python3
"""Regression checks for CPU phase evidence and baseline protection."""

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

import native_main_wait_phase_compare as phases
from snes9x_trace_format import encode_events


def capture(host, pc, v=225, h=24, owner=None):
    owner = host if owner is None else owner
    return (f"native-main-phase-capture frame={host} cpu_host={host} trailing=false "
            f"retained=Some(({owner}, {pc}, CpuRasterPosition {{ scanline: {v}, "
            f"master_cycle: {h} }}))\n")


class NativePhaseTests(unittest.TestCase):
    def test_same_raster_different_instruction_is_a_mismatch(self):
        rows = phases.compare({17: [0x8036, 225, 24]}, {17: [[0x8034, 225, 24]]})
        self.assertEqual(rows[0]["status"], "mismatch")

    def test_missing_and_multiple_source_acceptances_are_not_matches(self):
        rows = phases.compare({17: [0x8034, 225, 24], 18: [0x8034, 225, 24]},
                              {18: [[0x8034, 225, 24], [0x8034, 225, 24]]})
        self.assertEqual([row["status"] for row in rows],
                         ["source_missing", "source_ambiguous"])

    def test_binary_and_json_source_use_field_frame_rather_than_retro_run(self):
        events = [{"event": "nmi", "frame": 16, "run": 3,
                   "pc": 0x8034, "v": 225, "cycles": 24}]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "trace.bin"
            text = root / "trace.jsonl"
            binary.write_bytes(encode_events(events))
            text.write_text(json.dumps(events[0]) + "\n")
            self.assertEqual(phases.source_acceptances(binary),
                             phases.source_acceptances(text))
            self.assertEqual(phases.source_acceptances(binary), {17: [[0x8034, 225, 24]]})

    def test_other_host_owners_are_skipped_and_conflicting_samples_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.log"
            path.write_text(capture(17, 0x8034, owner=18) + capture(17, 0x8036))
            self.assertEqual(phases.read_native_phases(path), {17: [0x8036, 225, 24]})
            path.write_text(capture(17, 0x8034) + capture(17, 0x8036))
            with self.assertRaisesRegex(ValueError, "conflicting"):
                phases.read_native_phases(path)
            path.write_text("native-main-phase-capture frame=17 cpu_host=17 retained=Some(bad)\n")
            with self.assertRaisesRegex(ValueError, "malformed"):
                phases.read_native_phases(path)

    def test_later_gains_cannot_hide_an_earlier_regression_or_missing_sample(self):
        before = [{"host": 17, "status": "match"}, {"host": 18, "status": "mismatch"}]
        after = [{"host": 18, "status": "match"}]
        self.assertEqual(phases.phase_changes(before, after),
                         {"gained_exact_hosts": [18], "lost_exact_hosts": [17]})

    def test_unowned_captures_are_visible_and_can_gain_a_same_host_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "native.log"
            missing = "native-main-phase-capture frame=17 cpu_host=17 retained=None\n"
            path.write_text(missing + capture(18, 0x8034, owner=19))
            observed = phases.read_native_phases(path)
            self.assertEqual(observed, {17: None, 18: None})
            rows = phases.compare(observed, {17: [[0x8034, 225, 24]]})
            self.assertEqual([row["status"] for row in rows],
                             ["native_missing", "source_missing"])
            path.write_text(missing + capture(17, 0x8034) + missing)
            self.assertEqual(phases.read_native_phases(path), {17: [0x8034, 225, 24]})

    def test_exact_retained_sample_does_not_hide_an_unowned_source_wait(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native, source, report = [root / name for name in
                                      ("native.log", "source.bin", "report.json")]
            native.write_text(capture(17, 0x8034) +
                              "native-main-phase-capture frame=18 cpu_host=18 retained=None\n")
            source.write_bytes(encode_events([{"event": "nmi", "frame": frame,
                "pc": 0x8034, "v": 225, "cycles": 24} for frame in [16, 17]]))
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(phases.main([str(native), str(source),
                                             "--output", str(report)]), 1)
            data = json.loads(report.read_text())
            self.assertEqual(data["exact_hosts"], 1)
            self.assertEqual(data["sampled_hosts"], 2)
            self.assertEqual(data["retained_hosts"], 1)
            self.assertEqual(data["missing_native_hosts"], [18])
            self.assertEqual(data["first_unowned_source_main_wait"]["host"], 18)
            self.assertIsNone(data["first_retained_mismatch"])

    def test_cli_rejects_regression_and_writes_replayable_file_identities(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline, candidate, source, report = [root / name for name in
                ("baseline.log", "candidate.log", "source.bin", "report.json")]
            baseline.write_text(capture(17, 0x8034) + capture(18, 0x8036))
            candidate.write_text(capture(17, 0x8036) + capture(18, 0x8034))
            source.write_bytes(encode_events([{"event": "nmi", "frame": frame,
                "pc": 0x8034, "v": 225, "cycles": 24} for frame in [16, 17]]))
            with contextlib.redirect_stdout(io.StringIO()):
                status = phases.main([str(candidate), str(source), "--baseline-native-log",
                                      str(baseline), "--output", str(report)])
            self.assertEqual(status, 1)
            data = json.loads(report.read_text())
            self.assertEqual(data["gained_exact_hosts"], [18])
            self.assertEqual(data["lost_exact_hosts"], [17])
            self.assertEqual(data["source_trace"], phases.identity(source))

    def test_empty_observations_do_not_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            native, source = root / "native.log", root / "source.bin"
            native.write_text("")
            source.write_bytes(encode_events([]))
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(phases.main([str(native), str(source)]), 1)


if __name__ == "__main__":
    unittest.main()
