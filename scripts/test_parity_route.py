#!/usr/bin/env python3

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))


def load_script(name: str):
    path = SCRIPT_DIR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class ParityRouteTests(unittest.TestCase):
    def test_all_parity_entry_points_select_the_authoritative_existing_route(self):
        recorder = load_script("snes9x_route_recorder")
        precommit = load_script("precommit_snes9x_parity_gate")
        probe = load_script("parity_probe")

        expected = (SCRIPT_DIR.parent / "routes" / "crystal4_II").resolve()
        self.assertEqual(recorder.DEFAULT_PROJECT.resolve(), expected)
        self.assertEqual(precommit.DEFAULT_PROJECT.resolve(), expected)
        self.assertEqual((probe.ROOT / probe.DEFAULT_PROJECT).resolve(), expected)
        self.assertEqual(
            json.loads((expected / "manifest.json").read_text())["kind"],
            "zelda3_snes9x_route_recording_v1",
        )

    def test_precommit_gate_rejects_a_binary_older_than_runtime_sources(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "zelda3"
            binary.write_bytes(b"binary")
            source = Path(directory) / "runtime.rs"
            with mock.patch.object(
                precommit,
                "newest_source_mtime",
                return_value=(binary.stat().st_mtime + 1, source),
            ):
                self.assertEqual(precommit._stale_binary_source(binary), source)

    def test_precommit_route_source_hash_changes_with_input_contents(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            input_path = project / "input.txt"
            input_path.write_text("0 0x0000\n")
            takes = {4: {"id": 4, "frames": 10, "input_path": "input.txt"}}

            before = precommit._take_file_chain_sha256(
                project, takes, [4], "input_path"
            )
            input_path.write_text("0 0x0080\n")
            after = precommit._take_file_chain_sha256(
                project, takes, [4], "input_path"
            )

            self.assertNotEqual(before, after)

    def test_precommit_gate_stops_at_the_first_mismatch(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        identity = {"core_sha256": "core", "rom_sha256": "rom"}
        with (
            mock.patch.object(precommit.recorder, "load_manifest", return_value={}),
            mock.patch.object(
                precommit.recorder,
                "oracle_generations",
                return_value=[{"identity": identity}],
            ),
            mock.patch.object(
                precommit.recorder,
                "compare_input_command",
                return_value=["zelda3", "--scan-all", "--session-dir", "/tmp/session"],
            ),
        ):
            command = precommit._build_check_command(
                binary=Path("zelda3"),
                core=Path("core"),
                rom=Path("rom"),
                project=Path("project"),
                session_dir=Path("session"),
                take_ids=[0],
                start_boundary=0,
                requested_frames=100,
                input_path=Path("input"),
                rom_random_path=None,
            )

        self.assertNotIn("--scan-all", command)

    def test_precommit_gate_video_preflight_disables_only_audio(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        identity = {"core_sha256": "core", "rom_sha256": "rom"}
        with (
            mock.patch.object(precommit.recorder, "load_manifest", return_value={}),
            mock.patch.object(
                precommit.recorder,
                "oracle_generations",
                return_value=[{"identity": identity}],
            ),
            mock.patch.object(
                precommit.recorder,
                "compare_input_command",
                return_value=["zelda3", "--scan-all", "--session-dir", "/tmp/session"],
            ),
        ):
            command = precommit._build_check_command(
                binary=Path("zelda3"),
                core=Path("core"),
                rom=Path("rom"),
                project=Path("project"),
                session_dir=Path("session"),
                take_ids=[0],
                start_boundary=0,
                requested_frames=100,
                input_path=Path("input"),
                rom_random_path=None,
                rolling=(1000, Path("video-checkpoints")),
                ignore_audio=True,
            )

        self.assertNotIn("--scan-all", command)
        self.assertIn("--ignore-audio", command)
        self.assertNotIn("--ignore-video", command)
        self.assertEqual(
            command[-3:],
            ["--save-rolling-paired-resume", "1000", "video-checkpoints"],
        )

    def test_precommit_live_rng_preflight_uses_trace_core_without_resume(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        identity = {"core_sha256": "stock", "rom_sha256": "rom"}
        with (
            mock.patch.object(precommit.recorder, "load_manifest", return_value={}),
            mock.patch.object(
                precommit.recorder,
                "oracle_generations",
                return_value=[{"identity": identity}],
            ),
            mock.patch.object(
                precommit.recorder,
                "compare_input_command",
                return_value=[
                    "zelda3",
                    "--expected-core-sha256",
                    "stock",
                    "--session-dir",
                    "/tmp/session",
                ],
            ),
        ):
            command = precommit._build_check_command(
                binary=Path("zelda3"),
                core=Path("trace-core"),
                rom=Path("rom"),
                project=Path("project"),
                session_dir=Path("session"),
                take_ids=[0],
                start_boundary=0,
                requested_frames=100,
                input_path=Path("input"),
                rom_random_path=None,
                resume_dir=Path("diagnostic-checkpoint"),
                rolling=(1000, Path("fresh-checkpoints")),
                ignore_audio=True,
                ignore_video=True,
                live_oracle_rng=True,
                expected_core_sha256="trace",
            )

        self.assertEqual(
            command[command.index("--expected-core-sha256") + 1], "trace"
        )
        self.assertIn("--live-oracle-rng", command)
        self.assertIn("--ignore-audio", command)
        self.assertIn("--ignore-video", command)
        self.assertNotIn("--rom-random-script", command)
        self.assertNotIn("--resume-paired", command)
        self.assertNotIn("--save-rolling-paired-resume", command)

    def test_precommit_materializes_only_cartridge_rng_writes(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace.jsonl"
            output = Path(directory) / "rom-random.txt"
            trace.write_text(
                "\n".join(
                    [
                        '{"event":"rng-write","run":7,"pc":57291,"address":4001,"value":36,"carry":0}',
                        '{"event":"rng-write","run":9,"pc":899711,"address":4001,"value":165,"carry":1}',
                    ]
                )
                + "\n"
            )

            count = precommit._write_live_oracle_rng_script(trace, output)

            self.assertEqual(count, 1)
            self.assertIn("9 0xa5 carry=1", output.read_text())
            self.assertNotIn("7 0x24", output.read_text())

    def test_precommit_rng_cache_is_content_addressed_and_tamper_evident(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        signature = {
            "input_sha256": "input",
            "initial_sram_sha256": "sram",
            "rom_sha256": "rom",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            script = root / "fresh.txt"
            restored = root / "restored.txt"
            script.write_text("9 0xa5 carry=1\n")
            with mock.patch.object(precommit, "RNG_CACHE_PATH", root / "cache"):
                precommit._store_rng_cache(signature, 100, "trace", script, 1)
                self.assertEqual(
                    precommit._restore_rng_cache(
                        signature, 100, "trace", restored
                    ),
                    1,
                )
                self.assertEqual(restored.read_text(), script.read_text())
                cache = next((root / "cache").iterdir()) / "rom-random.txt"
                cache.write_text("corrupt\n")
                self.assertIsNone(
                    precommit._restore_rng_cache(
                        signature, 100, "trace", restored
                    )
                )

            changed = dict(signature, input_sha256="changed")
            self.assertNotEqual(
                precommit._rng_cache_key(signature, 100, "trace"),
                precommit._rng_cache_key(changed, 100, "trace"),
            )

    def test_precommit_gate_authoritative_command_always_starts_cold(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        identity = {"core_sha256": "core", "rom_sha256": "rom"}
        with (
            mock.patch.object(precommit.recorder, "load_manifest", return_value={}),
            mock.patch.object(
                precommit.recorder,
                "oracle_generations",
                return_value=[{"identity": identity}],
            ),
            mock.patch.object(
                precommit.recorder,
                "compare_input_command",
                return_value=["zelda3", "--session-dir", "/tmp/session"],
            ),
        ):
            command = precommit._build_check_command(
                binary=Path("zelda3"),
                core=Path("core"),
                rom=Path("rom"),
                project=Path("project"),
                session_dir=Path("session"),
                take_ids=[0],
                start_boundary=0,
                requested_frames=100,
                input_path=Path("input"),
                rom_random_path=None,
                resume_dir=Path("diagnostic-checkpoint"),
                rolling=(1000, Path("fresh-checkpoints")),
                authoritative=True,
            )

        self.assertNotIn("--resume-paired", command)
        self.assertNotIn("--save-rolling-paired-resume", command)

    def test_precommit_gate_keeps_dense_diagnostic_checkpoints_by_default(self):
        precommit = load_script("precommit_snes9x_parity_gate")
        with mock.patch.dict(os.environ, {}, clear=True):
            self.assertTrue(precommit._resume_enabled())
            self.assertTrue(precommit._video_preflight_enabled())
        with mock.patch.dict(os.environ, {"ZELDA3_PRECOMMIT_RESUME": "0"}):
            self.assertFalse(precommit._resume_enabled())
        with mock.patch.dict(
            os.environ, {"ZELDA3_PRECOMMIT_VIDEO_PREFLIGHT": "false"}
        ):
            self.assertFalse(precommit._video_preflight_enabled())
        self.assertEqual(precommit._default_resume_interval(10_000), 1_000)
        self.assertEqual(precommit._default_resume_interval(250), 250)


if __name__ == "__main__":
    unittest.main()
