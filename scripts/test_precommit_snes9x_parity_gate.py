#!/usr/bin/env python3

import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import precommit_snes9x_parity_gate as gate


class CleanEnvironmentTests(unittest.TestCase):
    def test_hostile_parent_environment_is_removed_and_empty_config_is_explicit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inherited_config = root / "inherited.toml"
            inherited_config.write_text("[[cpu_phase_rules]]\n", encoding="utf-8")
            parent = {
                "DISPLAY": ":77",
                "GIT_DIR": "/hostile/git",
                "RUST_LOG": "trace",
                "SNES9X_TRACE_ALL": "1",
                "ZELDA3_PARITY_RUNTIME_CONFIG": str(inherited_config),
                "ZELDA3_TIMING_DIAGNOSTICS": "1",
            }
            with mock.patch.dict(os.environ, parent, clear=True):
                child, normalized = gate._clean_child_environment(root / "child")

            self.assertNotIn("GIT_DIR", child)
            self.assertNotIn("RUST_LOG", child)
            self.assertNotIn("SNES9X_TRACE_ALL", child)
            self.assertNotIn("ZELDA3_TIMING_DIAGNOSTICS", child)
            runtime_config = Path(child["ZELDA3_PARITY_RUNTIME_CONFIG"])
            self.assertNotEqual(runtime_config, inherited_config)
            self.assertEqual(runtime_config.read_bytes(), b"")
            self.assertEqual(
                normalized["runtime_config"]["sha256"],
                gate.parity_evidence.EMPTY_RUNTIME_CONFIG_SHA256,
            )
            self.assertEqual(runtime_config.stat().st_mode & 0o777, 0o444)
            self.assertEqual(child["DISPLAY"], ":77")

    def test_manual_and_hook_parent_noise_have_the_same_normalized_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manual = {"DISPLAY": ":77", "ZELDA3_DEBUG": "manual", "PWD": "/one"}
            hook = {"DISPLAY": ":77", "SNES9X_DEBUG": "hook", "GIT_INDEX_FILE": "/two"}
            with mock.patch.dict(os.environ, manual, clear=True):
                _, manual_contract = gate._clean_child_environment(root / "manual")
            with mock.patch.dict(os.environ, hook, clear=True):
                _, hook_contract = gate._clean_child_environment(root / "hook")

            self.assertEqual(manual_contract, hook_contract)


class AuthoritativePolicyTests(unittest.TestCase):
    def _command(self, root: Path) -> tuple[list[str], dict[str, Path]]:
        paths = {
            name: root / name
            for name in (
                "zelda3",
                "core.dylib",
                "zelda3.sfc",
                "input.txt",
                "rom-random.txt",
                "initial.srm",
                "session",
            )
        }
        for name, path in paths.items():
            if name != "session":
                path.write_bytes(name.encode())
        command = [
            str(paths["zelda3"]),
            "--compare-snes9x-oracle",
            str(paths["core.dylib"]),
            str(paths["zelda3.sfc"]),
            "100",
            "--expected-core-sha256",
            gate.recorder.sha256(paths["core.dylib"]),
            "--expected-rom-sha256",
            gate.recorder.sha256(paths["zelda3.sfc"]),
            "--input-script",
            str(paths["input.txt"]),
            "--rom-random-script",
            str(paths["rom-random.txt"]),
            "--load-sram",
            str(paths["initial.srm"]),
            "--audio-comparison",
            "exact",
            "--session-dir",
            str(paths["session"]),
            "--cold-evidence-invocation-id",
            gate.COLD_EVIDENCE_INVOCATION_ID_PLACEHOLDER,
        ]
        return command, paths

    def _normalize(self, command: list[str], paths: dict[str, Path]) -> dict:
        return gate._normalized_authoritative_policy(
            command,
            binary=paths["zelda3"],
            core=paths["core.dylib"],
            rom=paths["zelda3.sfc"],
            input_path=paths["input.txt"],
            rom_random_path=paths["rom-random.txt"],
            initial_sram_path=paths["initial.srm"],
            session_path=paths["session"],
            requested_frames=100,
        )

    def test_normalized_policy_uses_roles_not_incidental_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = root / "first"
            second = root / "second"
            first.mkdir()
            second.mkdir()
            first_command, first_paths = self._command(first)
            second_command, second_paths = self._command(second)

            self.assertEqual(
                self._normalize(first_command, first_paths),
                self._normalize(second_command, second_paths),
            )

    def test_forbidden_flag_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            command, paths = self._command(root)
            command.extend(["--fixed-oracle-startup-skip-frames", "1"])

            with self.assertRaisesRegex(SystemExit, "forbidden"):
                self._normalize(command, paths)

    def test_unnormalized_source_path_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            command, paths = self._command(root)
            paths["input.txt"] = root / "nested" / ".." / "input.txt"
            command[command.index("--input-script") + 1] = str(paths["input.txt"])

            with self.assertRaisesRegex(SystemExit, "not normalized"):
                self._normalize(command, paths)


class GateReuseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.project = self.root / "route"
        self.project.mkdir()
        self.binary = self.root / "zelda3"
        self.core = self.root / "core.dylib"
        self.rom = self.root / "zelda3.sfc"
        self.binary.write_bytes(b"binary")
        self.core.write_bytes(b"core")
        self.rom.write_bytes(b"rom")
        self.state_path = self.root / "state.json"
        self.manifest = {
            "takes": [{"id": 1, "frames": 100, "start_boundary": 0}]
        }
        self.signature = {"schema": 1, "route": "cold", "frames": 100}
        self.staged_source = {
            "identity": {"schema": 1, "content": "source"},
            "identity_sha256": "a" * 64,
            "build_binding": {"schema": 1, "profile": "parity"},
            "build_binding_sha256": "b" * 64,
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _environment(self) -> dict[str, str]:
        return {
            "ZELDA3_PRECOMMIT_PROJECT": str(self.project),
            "ZELDA3_PRECOMMIT_BINARY": str(self.binary),
            "ZELDA3_PRECOMMIT_ROM": str(self.rom),
            "ZELDA3_PRECOMMIT_VIDEO_PREFLIGHT": "0",
            "ZELDA3_PRECOMMIT_RESUME": "0",
            "ZELDA3_PRECOMMIT_TARGET_FRAME": "100",
            "ZELDA3_PRECOMMIT_MAX_FRAMES": "100",
            "ZELDA3_CHILD_SHOULD_NOT_INHERIT": "1",
            "SNES9X_CHILD_SHOULD_NOT_INHERIT": "1",
            "GIT_DIR": "/hostile/git",
        }

    @staticmethod
    def _write_input(
        _project: Path,
        _take_ids: list[int],
        output: Path,
        *,
        takes_by_id: dict,
    ) -> int:
        del takes_by_id
        output.write_text("0 0x0000\n", encoding="utf-8")
        return 100

    @staticmethod
    def _write_rng(
        _project: Path,
        _take_ids: list[int],
        output: Path,
        *,
        takes_by_id: dict,
    ) -> int:
        del takes_by_id
        output.write_text("9 0xa5 carry=1\n", encoding="utf-8")
        return 1

    def _build_command(self, **arguments: object) -> list[str]:
        input_path = Path(arguments["input_path"])
        random_value = arguments["rom_random_path"]
        random_path = Path(random_value) if random_value is not None else None
        session = Path(arguments["session_dir"])
        sram = input_path.parent / "initial.srm"
        sram.write_bytes(b"sram")
        command = [
            str(arguments["binary"]),
            "--compare-snes9x-oracle",
            str(arguments["core"]),
            str(arguments["rom"]),
            str(arguments["requested_frames"]),
            "--expected-core-sha256",
            gate.recorder.sha256(Path(arguments["core"])),
            "--expected-rom-sha256",
            gate.recorder.sha256(Path(arguments["rom"])),
            "--input-script",
            str(input_path),
            "--load-sram",
            str(sram),
            "--audio-comparison",
            "exact",
            "--session-dir",
            str(session),
        ]
        if random_path is not None:
            session_index = command.index("--load-sram")
            command[session_index:session_index] = [
                "--rom-random-script",
                str(random_path),
            ]
        invocation_id = arguments.get("cold_evidence_invocation_id")
        if invocation_id is not None:
            command.extend(["--cold-evidence-invocation-id", str(invocation_id)])
        return command

    def _common_patches(self):
        return (
            mock.patch.object(gate, "STATE_PATH", self.state_path),
            mock.patch.object(gate, "_stale_binary_source", return_value=None),
            mock.patch.object(gate.recorder, "load_manifest", return_value=self.manifest),
            mock.patch.object(gate.recorder, "continuous_take_ids", return_value=[1]),
            mock.patch.object(gate, "_route_signature", return_value=self.signature),
            mock.patch.object(gate, "_extract_identity", return_value=(self.core, {})),
            mock.patch.object(
                gate.recorder,
                "write_continuous_input",
                side_effect=self._write_input,
            ),
            mock.patch.object(
                gate.recorder,
                "write_continuous_rom_random",
                side_effect=self._write_rng,
            ),
            mock.patch.object(gate, "_build_check_command", side_effect=self._build_command),
            mock.patch.object(
                gate.parity_evidence,
                "staged_source_authority",
                return_value=self.staged_source,
            ),
        )

    def test_exact_reuse_hit_skips_session_run_and_new_receipt(self) -> None:
        reusable = {
            "receipt_path": str(self.root / "existing-proof.json"),
            "receipt_sha256": "c" * 64,
        }
        environment = self._environment()
        environment["ZELDA3_PRECOMMIT_VIDEO_PREFLIGHT"] = "1"
        environment["ZELDA3_PRECOMMIT_TRACE_CORE"] = str(self.core)

        def restore_rng(
            _signature: dict,
            _requested_frames: int,
            _trace_core_sha256: str,
            output: Path,
        ) -> int:
            output.write_text("9 0xa5 carry=1\n", encoding="utf-8")
            return 1

        with contextlib.ExitStack() as stack:
            for patch in self._common_patches():
                stack.enter_context(patch)
            stack.enter_context(
                mock.patch.object(
                    gate,
                    "validate_trace_core",
                    return_value=gate.recorder.sha256(self.core),
                )
            )
            stack.enter_context(
                mock.patch.object(gate, "_restore_rng_cache", side_effect=restore_rng)
            )
            stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "find_reusable_cold_pass",
                    return_value=reusable,
                )
            )
            reserve = stack.enter_context(
                mock.patch.object(gate, "_reserve_precommit_session_paths")
            )
            run = stack.enter_context(mock.patch.object(gate.subprocess, "run"))
            record = stack.enter_context(
                mock.patch.object(gate.parity_evidence, "record_cold_pass")
            )
            stack.enter_context(
                mock.patch.dict(os.environ, environment, clear=True)
            )
            self.assertEqual(gate.run_snes9x_gate(), 0)

        reserve.assert_not_called()
        run.assert_not_called()
        record.assert_not_called()
        state = json.loads(self.state_path.read_text(encoding="utf-8"))
        self.assertEqual(state["last_cold_receipt_path"], reusable["receipt_path"])
        self.assertEqual(state["last_cold_receipt_sha256"], reusable["receipt_sha256"])

    def test_calibration_then_reuse_removes_only_empty_exact_reservation(self) -> None:
        reusable = {
            "receipt_path": str(self.root / "existing-proof.json"),
            "receipt_sha256": "c" * 64,
        }
        environment = self._environment()
        environment["ZELDA3_PRECOMMIT_VIDEO_PREFLIGHT"] = "1"
        environment["ZELDA3_PRECOMMIT_TRACE_CORE"] = str(self.core)
        reservations = []
        real_reserve = gate._reserve_precommit_session_paths

        def reserve(project: Path, requested: int):
            paths = real_reserve(project, requested)
            reservations.append(paths)
            return paths

        def calibrate(command: list[str], **_arguments: object):
            session = Path(command[command.index("--session-dir") + 1])
            session.mkdir(parents=True)
            (session / "result.json").write_text(
                json.dumps({"status": "passed", "parity_eligible": True}),
                encoding="utf-8",
            )
            return mock.Mock(returncode=0, stdout="", stderr="")

        def write_rng(_trace: Path, output: Path) -> int:
            output.write_text("9 0xa5 carry=1\n", encoding="utf-8")
            return 1

        with contextlib.ExitStack() as stack:
            for patch in self._common_patches():
                stack.enter_context(patch)
            stack.enter_context(
                mock.patch.object(
                    gate,
                    "validate_trace_core",
                    return_value=gate.recorder.sha256(self.core),
                )
            )
            stack.enter_context(
                mock.patch.object(gate, "_restore_rng_cache", return_value=None)
            )
            stack.enter_context(
                mock.patch.object(
                    gate, "_write_live_oracle_rng_script", side_effect=write_rng
                )
            )
            stack.enter_context(mock.patch.object(gate, "_store_rng_cache"))
            stack.enter_context(
                mock.patch.object(
                    gate, "_reserve_precommit_session_paths", side_effect=reserve
                )
            )
            execute = stack.enter_context(
                mock.patch.object(gate.subprocess, "run", side_effect=calibrate)
            )
            stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "find_reusable_cold_pass",
                    return_value=reusable,
                )
            )
            record = stack.enter_context(
                mock.patch.object(gate.parity_evidence, "record_cold_pass")
            )
            stack.enter_context(mock.patch.dict(os.environ, environment, clear=True))
            self.assertEqual(gate.run_snes9x_gate(), 0)

        self.assertEqual(execute.call_count, 1, "only RNG calibration should run")
        record.assert_not_called()
        self.assertEqual(len(reservations), 1)
        self.assertFalse(reservations[0].exact.exists())
        self.assertTrue((reservations[0].rng_calibration / "result.json").is_file())

    def test_late_reuse_refuses_to_remove_nonempty_exact_reservation(self) -> None:
        exact = self.root / "reserved-exact"
        exact.mkdir()
        marker = exact / "unexpected"
        marker.write_text("preserve", encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "refusing to remove nonempty"):
            gate._remove_empty_reserved_exact_session(exact)
        self.assertEqual(marker.read_text(encoding="utf-8"), "preserve")

    def test_miss_runs_once_in_clean_environment_and_records_schema2(self) -> None:
        receipt = self.root / "new-proof.json"
        receipt.write_bytes(b"receipt")
        observed_environments = []
        observed_runtime_configs = []
        observed_invocation_ids = []

        def run(command: list[str], **arguments: object):
            observed_environments.append(dict(arguments["env"]))
            observed_runtime_configs.append(
                Path(arguments["env"]["ZELDA3_PARITY_RUNTIME_CONFIG"]).read_bytes()
            )
            observed_invocation_ids.append(
                command[command.index("--cold-evidence-invocation-id") + 1]
            )
            session = Path(command[command.index("--session-dir") + 1])
            (session / "result.json").write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "parity_eligible": True,
                        "video": {"matched": True},
                        "audio": {"matched": True},
                    }
                ),
                encoding="utf-8",
            )
            return mock.Mock(returncode=0, stdout="", stderr="")

        with contextlib.ExitStack() as stack:
            for patch in self._common_patches():
                stack.enter_context(patch)
            stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "find_reusable_cold_pass",
                    return_value=None,
                )
            )
            stack.enter_context(
                mock.patch.object(gate.subprocess, "run", side_effect=run)
            )
            record = stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "record_cold_pass",
                    return_value=receipt,
                )
            )
            stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "list_verified_cold_passes",
                    side_effect=lambda: [
                        {
                            "receipt_path": str(receipt.resolve()),
                            "receipt_sha256": gate.parity_evidence.sha256_file(
                                receipt
                            ),
                            "invocation_id": record.call_args.kwargs[
                                "invocation_id"
                            ],
                        }
                    ],
                )
            )
            stack.enter_context(
                mock.patch.dict(os.environ, self._environment(), clear=True)
            )
            self.assertEqual(gate.run_snes9x_gate(), 0)

        self.assertEqual(len(observed_environments), 1)
        child = observed_environments[0]
        self.assertNotIn("ZELDA3_CHILD_SHOULD_NOT_INHERIT", child)
        self.assertNotIn("SNES9X_CHILD_SHOULD_NOT_INHERIT", child)
        self.assertNotIn("GIT_DIR", child)
        self.assertEqual(observed_runtime_configs, [b""])
        call = record.call_args.kwargs
        self.assertEqual(call["authority"]["invocation"]["execution_policy"], "clean_env_v1")
        self.assertRegex(call["invocation_id"], r"^[A-Za-z0-9_.-]+$")
        self.assertEqual(observed_invocation_ids, [call["invocation_id"]])
        self.assertNotEqual(
            observed_invocation_ids[0],
            gate.COLD_EVIDENCE_INVOCATION_ID_PLACEHOLDER,
        )

    def test_force_confirmation_bypasses_reuse_and_records_independent_proof(self) -> None:
        receipt = self.root / "forced-proof.json"
        receipt.write_bytes(b"receipt")
        observed_environments = []

        def run(command: list[str], **arguments: object):
            observed_environments.append(dict(arguments["env"]))
            session = Path(command[command.index("--session-dir") + 1])
            (session / "result.json").write_text(
                json.dumps(
                    {
                        "status": "passed",
                        "parity_eligible": True,
                        "video": {"matched": True},
                        "audio": {"matched": True},
                    }
                ),
                encoding="utf-8",
            )
            return mock.Mock(returncode=0, stdout="", stderr="")

        environment = self._environment()
        environment["ZELDA3_PRECOMMIT_FORCE_COLD_CONFIRMATION"] = "1"
        with contextlib.ExitStack() as stack:
            for patch in self._common_patches():
                stack.enter_context(patch)
            find = stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "find_reusable_cold_pass",
                    return_value={
                        "receipt_path": str(self.root / "existing-proof.json"),
                        "receipt_sha256": "c" * 64,
                    },
                )
            )
            execute = stack.enter_context(
                mock.patch.object(gate.subprocess, "run", side_effect=run)
            )
            record = stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "record_cold_pass",
                    return_value=receipt,
                )
            )
            stack.enter_context(
                mock.patch.object(
                    gate.parity_evidence,
                    "list_verified_cold_passes",
                    side_effect=lambda: [
                        {
                            "receipt_path": str(receipt.resolve()),
                            "receipt_sha256": gate.parity_evidence.sha256_file(
                                receipt
                            ),
                            "invocation_id": record.call_args.kwargs[
                                "invocation_id"
                            ],
                        }
                    ],
                )
            )
            stack.enter_context(mock.patch.dict(os.environ, environment, clear=True))
            self.assertEqual(gate.run_snes9x_gate(), 0)

        find.assert_not_called()
        execute.assert_called_once()
        record.assert_called_once()
        self.assertNotIn(
            "ZELDA3_PRECOMMIT_FORCE_COLD_CONFIRMATION",
            observed_environments[0],
        )

    def test_force_confirmation_flag_fails_closed_on_non_boolean_value(self) -> None:
        with mock.patch.dict(
            os.environ,
            {"ZELDA3_PRECOMMIT_FORCE_COLD_CONFIRMATION": "yes"},
            clear=True,
        ):
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                with self.assertRaises(SystemExit) as raised:
                    gate.run_snes9x_gate()
            self.assertEqual(raised.exception.code, 2)
            self.assertIn("expected exactly 0 or 1", stderr.getvalue())

    def test_new_receipt_must_be_accepted_by_rust_before_state_advances(self) -> None:
        receipt = self.root / "new-proof.json"
        receipt.write_bytes(b"receipt")
        with mock.patch.object(
            gate.parity_evidence, "list_verified_cold_passes", return_value=[]
        ):
            with self.assertRaisesRegex(SystemExit, "did not verify"):
                gate._require_rust_verified_cold_receipt(receipt, "invocation-1")


if __name__ == "__main__":
    unittest.main()
