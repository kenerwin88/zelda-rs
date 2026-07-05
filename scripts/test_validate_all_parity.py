import importlib.util
import pathlib
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_all_parity.py"
PRE_COMMIT = ROOT / ".githooks" / "pre-commit"


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

    def test_gpu_checkpoint_sweep_runs_after_successful_zparity_when_requested(self):
        module = load_module()
        calls = []

        def fake_run(cmd, **kwargs):
            calls.append(cmd)
            return mock.Mock(returncode=0)

        with mock.patch.object(module.subprocess, "run", side_effect=fake_run):
            self.assertEqual(module.main(["--frames", "12000", "--gpu-checkpoint-sweep"]), 0)

        self.assertEqual(
            calls,
            [
                ["cargo", "run", "-p", "parity", "--", "check", "--frames", "12000"],
                [
                    module.sys.executable,
                    "scripts/gpu_render_compare_checkpoint_sweep.py",
                    "--build",
                    "--require-mode7",
                ],
            ],
        )

    def test_gpu_checkpoint_sweep_does_not_run_after_failed_zparity(self):
        module = load_module()
        calls = []

        def fake_run(cmd, **kwargs):
            calls.append(cmd)
            return mock.Mock(returncode=1)

        with mock.patch.object(module.subprocess, "run", side_effect=fake_run):
            self.assertEqual(module.main(["--gpu-checkpoint-sweep"]), 1)

        self.assertEqual(len(calls), 1)

    def test_pre_commit_uses_golden_checker_not_live_c_parity(self):
        hook = PRE_COMMIT.read_text()

        self.assertIn("scripts/validate_all_parity.py --full", hook)
        self.assertIn('scripts/validate_all_parity.py --frames "$PARITY_FRAMES"', hook)
        self.assertIn("scripts/variant_atlas_summary.py --require-full-stable", hook)
        self.assertIn("--manifest generated/zelda3_assets/manifest.json", hook)
        self.assertNotIn("scripts/test_standard_replay_parity.py", hook)
        self.assertNotIn("scripts/fast_standard_replay_parity.py", hook)
        self.assertNotIn("../zelda3/zelda3", hook)
        self.assertNotIn("--smv-test-frames", hook)


if __name__ == "__main__":
    unittest.main()
