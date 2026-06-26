import importlib.util
import pathlib
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_all_parity.py"


def load_module():
    spec = importlib.util.spec_from_file_location("validate_all_parity", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ValidateAllParityTests(unittest.TestCase):
    def test_full_build_delegates_to_sharded_zparity_check_without_c_oracle(self):
        module = load_module()
        calls = []

        def fake_run(cmd, **kwargs):
            calls.append((cmd, kwargs))
            return mock.Mock(returncode=0)

        with mock.patch.object(module.subprocess, "run", side_effect=fake_run):
            self.assertEqual(
                module.main(["--full", "--build", "--shards", "8"]),
                0,
            )

        commands = [call[0] for call in calls]
        self.assertEqual(
            commands[0],
            ["cargo", "build", "--profile", "parity", "-p", "zelda3-bin"],
        )
        self.assertEqual(
            commands[1],
            [
                "cargo",
                "run",
                "-p",
                "parity",
                "--",
                "check",
                "--full",
                "--shards",
                "8",
            ],
        )
        flattened = " ".join(" ".join(command) for command in commands)
        self.assertNotIn("--smv-test-frames", flattened)
        self.assertNotIn("headless_replay.ini", flattened)

    def test_frame_count_delegates_to_zparity_frames(self):
        module = load_module()
        calls = []

        def fake_run(cmd, **kwargs):
            calls.append(cmd)
            return mock.Mock(returncode=0)

        with mock.patch.object(module.subprocess, "run", side_effect=fake_run):
            self.assertEqual(module.main(["--frames", "12000"]), 0)

        self.assertEqual(
            calls,
            [["cargo", "run", "-p", "parity", "--", "check", "--frames", "12000"]],
        )


if __name__ == "__main__":
    unittest.main()
