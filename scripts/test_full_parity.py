import argparse
import importlib.util
import os
import pathlib
import sys
import tempfile
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "full_parity.py"


def load_module():
    spec = importlib.util.spec_from_file_location("full_parity", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class Snes9xParityGateTests(unittest.TestCase):
    def test_finds_explicit_and_environment_snes9x_core(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            core = pathlib.Path(temp) / "snes9x_libretro.dylib"
            core.write_bytes(b"core")
            self.assertEqual(module.find_snes9x_core(str(core)), core)
            with mock.patch.dict(os.environ, {"SNES9X_LIBRETRO_CORE": str(core)}):
                self.assertEqual(module.find_snes9x_core(None), core)

    def test_snes9x_gate_uses_fixed_alignment_and_writes_a_session(self):
        module = load_module()
        args = argparse.Namespace(
            rom=pathlib.Path("/tmp/zelda3.sfc"),
            frames=321,
            release=False,
            input_script="route.txt",
            load_sram="save.srm",
            snes9x_skip=0,
            snes9x_core="/tmp/snes9x_libretro.dylib",
            no_install_snes9x=True,
            snes9x_url="unused",
            work_dir=pathlib.Path("/tmp/parity"),
        )
        result = module.CommandResult([], 0, "passed", "")
        with (
            mock.patch.object(
                module,
                "resolve_snes9x_core",
                return_value=pathlib.Path(args.snes9x_core),
            ),
            mock.patch.object(module, "run_command", return_value=result) as run,
        ):
            module.run_snes9x_gate(args)

        command = run.call_args.args[0]
        self.assertIn("--compare-snes9x-oracle", command)
        self.assertIn("--session-dir", command)
        self.assertIn("--audio-comparison", command)
        self.assertEqual(command[command.index("--audio-comparison") + 1], "exact")
        self.assertNotIn("--audio-timing-tolerance-ms", command)
        self.assertNotIn("--auto-align-video", command)

    def test_modern_audio_gate_uses_exact_snes9x_waveform_over_replay_save(self):
        module = load_module()
        args = argparse.Namespace(
            rom=pathlib.Path("/tmp/zelda3.sfc"),
            replay_save=pathlib.Path("/tmp/full-route.sav"),
            route_frames=1_073_092,
            release=True,
            snes9x_skip=0,
            snes9x_core="/tmp/snes9x_libretro.dylib",
            no_install_snes9x=True,
            snes9x_url="unused",
            work_dir=pathlib.Path("/tmp/parity"),
        )
        result = module.CommandResult([], 0, "passed", "")
        with (
            mock.patch.object(
                module,
                "resolve_snes9x_core",
                return_value=pathlib.Path(args.snes9x_core),
            ),
            mock.patch.object(module, "run_command", return_value=result) as run,
        ):
            module.run_snes9x_modern_audio_gate(args)

        command = run.call_args.args[0]
        self.assertIn("--compare-snes9x-oracle", command)
        self.assertIn("--replay-save", command)
        self.assertIn(str(args.replay_save), command)
        self.assertIn("--ignore-video", command)
        self.assertEqual(command[command.index("--audio-comparison") + 1], "exact")
        self.assertNotIn("--rust-audio-backend", command)

    def test_main_rejects_an_empty_gate_set(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            rom = pathlib.Path(temp) / "zelda3.sfc"
            rom.write_bytes(b"rom")
            argv = [
                str(SCRIPT),
                "--rom",
                str(rom),
            ]
            with mock.patch.object(sys, "argv", argv):
                self.assertEqual(module.main(), 2)


if __name__ == "__main__":
    unittest.main()
