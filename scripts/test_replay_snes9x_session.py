import argparse
import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "replay_snes9x_session.py"


def load_module():
    spec = importlib.util.spec_from_file_location("replay_snes9x_session", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReplaySnes9xSessionTests(unittest.TestCase):
    def test_builds_full_live_and_exact_replay_from_recording(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            session = pathlib.Path(temp)
            (session / "input.txt").write_text("10 A\n")
            (session / "initial.srm").write_bytes(b"sram")
            (session / "result.json").write_text(json.dumps({"frames": 123}))
            args = argparse.Namespace(
                session_dir=session,
                rom=pathlib.Path("/tmp/zelda3.sfc"),
                frames=None,
                snes9x_core=None,
                no_install_snes9x=False,
                no_exact_apu=False,
                work_dir=None,
                release=False,
            )
            command = module.build_command(args)

        self.assertIn("--with-snes9x", command)
        self.assertIn("--input-script", command)
        self.assertIn("--load-sram", command)
        self.assertEqual(command[command.index("--frames") + 1], "123")
        self.assertNotIn("--no-snes9x-exact-apu", command)

    def test_recovers_script_from_crash_resilient_events(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            session = pathlib.Path(temp)
            (session / "initial.srm").write_bytes(b"sram")
            (session / "live_inputs.jsonl").write_text(
                '{"frame":0,"input":"0x0000"}\n'
                '{"frame":1,"input":"0x0080"}\n'
                '{"frame":2,"input":"0x0080"}\n'
            )
            args = argparse.Namespace(
                session_dir=session,
                rom=pathlib.Path("/tmp/zelda3.sfc"),
                frames=None,
                snes9x_core=None,
                no_install_snes9x=False,
                no_exact_apu=False,
                work_dir=None,
                release=False,
            )

            command = module.build_command(args)
            recovered = session / "input.recovered.txt"

            self.assertTrue(recovered.exists())
            self.assertIn(str(recovered.resolve()), command)
            self.assertEqual(
                recovered.read_text(),
                "# Deterministic controller stream captured once per game frame.\n"
                "1..2 0x0080\n",
            )

    def test_recovery_ignores_only_a_truncated_final_event(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as temp:
            events = pathlib.Path(temp) / "live_inputs.jsonl"
            events.write_text(
                '{"frame":0,"input":"0x0000"}\n'
                '{"frame":1,"input":"0x0080"}\n'
                '{"frame":2,"inp'
            )

            self.assertEqual(module.captured_input_events(events), [(0, 0), (1, 0x80)])


if __name__ == "__main__":
    unittest.main()
