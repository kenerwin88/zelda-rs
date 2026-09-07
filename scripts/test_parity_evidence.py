import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import parity_evidence as evidence


class Schema2ColdEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.binary = self.root / "zelda3"
        self.binary.write_bytes(b"binary")
        self.session = self.root / "run-100-exact-invocation-a"
        self.session.mkdir()
        for name, contents in (
            ("input.txt", b"0 0x0000\n"),
            ("rom-random.txt", b"9 0xa5 carry=1\n"),
            ("initial.srm", b"sram"),
        ):
            (self.session / name).write_bytes(contents)
        core_sha = "c" * 64
        rom_sha = "d" * 64
        source_hashes = {
            name: evidence.sha256_file(self.session / name)
            for name in evidence.REPLAY_SOURCE_FILES
        }
        (self.session / "manifest.json").write_text(
            json.dumps(
                {
                    "cold_evidence_invocation_id": "invocation-a",
                    "cold_evidence_run_nonce": "a" * 64,
                    "status": "passed",
                    "parity_eligible": True,
                    "frames_completed": 100,
                    "core": {"sha256": core_sha},
                    "rom": {"sha256": rom_sha},
                    "timing": {"frames_requested": 100, "start_frame": 0},
                    "comparison_lanes": {"video": True, "audio": True},
                    "rom_random_replay": {
                        "sha256": source_hashes["rom-random.txt"]
                    },
                }
            ),
            encoding="utf-8",
        )
        (self.session / "result.json").write_text(
            json.dumps(
                {
                    "status": "passed",
                    "parity_eligible": True,
                    "frames_completed": 100,
                    "video": {"matched": True},
                    "audio": {"matched": True},
                }
            ),
            encoding="utf-8",
        )
        route = {"schema": 1, "project": "routes/full_run"}
        staged_identity = {
            "schema": 1,
            "head": "a" * 40,
            "file_count": 17,
            "content_inventory_sha256": "9" * 64,
            "index_inventory_sha256": "8" * 64,
            "status_sha256": "7" * 64,
        }
        build_binding = {"schema": 1, "profile": "parity"}
        policy = {"schema": 1, "command": {"cold": True}}
        self.authority = {
            "target_frames": 100,
            "route_signature": route,
            "route_signature_sha256": evidence.stable_hash(route),
            "binary": {
                "sha256": evidence.sha256_file(self.binary),
                "size": self.binary.stat().st_size,
            },
            "staged_source": {
                "identity": staged_identity,
                "identity_sha256": evidence.stable_hash(staged_identity),
                "build_binding": build_binding,
                "build_binding_sha256": evidence.stable_hash(build_binding),
            },
            "invocation": {
                "execution_policy": evidence.CLEAN_ENV_EXECUTION_POLICY,
                "policy": policy,
                "policy_sha256": evidence.stable_hash(policy),
                "environment_sha256": "e" * 64,
                "runtime_config_sha256": evidence.EMPTY_RUNTIME_CONFIG_SHA256,
            },
            "core_sha256": core_sha,
            "rom_sha256": rom_sha,
            "source_artifact_sha256": source_hashes,
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def current_source_authority(self) -> dict:
        recorded = self.authority["staged_source"]
        identity = dict(recorded["identity"])
        # A successful pre-commit proof is promoted after its exact content is
        # committed: these provenance fields legitimately change at that edge.
        identity.update(
            {
                "head": "f" * 40,
                "index_inventory_sha256": "0" * 64,
                "status_sha256": "0" * 64,
            }
        )
        return {
            "identity": identity,
            "identity_sha256": evidence.stable_hash(identity),
            "build_binding": recorded["build_binding"],
            "build_binding_sha256": recorded["build_binding_sha256"],
        }

    def copy_session(self, name: str, invocation_id: str, run_nonce: str) -> Path:
        destination = self.root / name
        shutil.copytree(self.session, destination)
        manifest_path = destination / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["cold_evidence_invocation_id"] = invocation_id
        manifest["cold_evidence_run_nonce"] = run_nonce
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        return destination

    def test_schema2_receipt_binds_invocation_session_and_content_filename(self) -> None:
        receipt = evidence.record_cold_pass(
            session=self.session,
            route_signature=self.authority["route_signature"],
            binary=self.binary,
            authority=self.authority,
            invocation_id="invocation-a",
            output_root=self.root / "passes",
        )

        raw = json.loads(receipt.read_text(encoding="utf-8"))
        self.assertEqual(raw["schema"], 2)
        self.assertEqual(raw["authority"], self.authority)
        self.assertEqual(raw["invocation_id"], "invocation-a")
        self.assertEqual(raw["run_nonce"], "a" * 64)
        self.assertTrue(receipt.name.endswith(f"-{evidence.stable_hash(raw)[:12]}.json"))

    def test_schema2_writer_rejects_caller_or_nonce_not_bound_by_the_runner(self) -> None:
        with self.assertRaisesRegex(SystemExit, "invocation ID does not match"):
            evidence.record_cold_pass(
                session=self.session,
                route_signature=self.authority["route_signature"],
                binary=self.binary,
                authority=self.authority,
                invocation_id="different-invocation",
                output_root=self.root / "passes",
            )

        manifest_path = self.session / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["cold_evidence_run_nonce"] = "caller-selected"
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "runner-authored cold run nonce"):
            evidence.record_cold_pass(
                session=self.session,
                route_signature=self.authority["route_signature"],
                binary=self.binary,
                authority=self.authority,
                invocation_id="invocation-a",
                output_root=self.root / "passes",
            )

    def test_legacy_receipt_remains_history_but_is_not_schema2(self) -> None:
        with mock.patch.object(evidence, "git_identity", return_value={"clean": True}):
            receipt = evidence.record_cold_pass(
                session=self.session,
                route_signature=self.authority["route_signature"],
                binary=self.binary,
                output_root=self.root / "passes",
            )
        self.assertEqual(json.loads(receipt.read_text())["schema"], 1)

    def test_find_uses_rust_verifier_and_copies_exact_authority(self) -> None:
        captured = {}

        def run(arguments: list[str], *, zparity: Path) -> dict:
            self.assertEqual(arguments[0], "find")
            captured.update(json.loads(Path(arguments[2]).read_text()))
            return {
                "schema": 2,
                "mode": "find",
                "reusable": True,
                "receipts": [
                    {
                        "receipt_path": "proof.json",
                        "receipt_sha256": "a" * 64,
                        "run_nonce": "a" * 64,
                    }
                ],
                "rejected": [],
            }

        with mock.patch.object(evidence, "_run_zparity_cold_evidence", side_effect=run):
            found = evidence.find_reusable_cold_pass(
                self.authority,
                pass_root=self.root / "passes",
                zparity=self.root / "zparity",
            )

        self.assertEqual(captured["schema"], 2)
        self.assertEqual(captured["authority"], self.authority)
        self.assertEqual(found["receipt_path"], "proof.json")

    def test_schema1_verifier_output_is_never_accepted_for_reuse(self) -> None:
        zparity = self.root / "zparity"
        zparity.write_bytes(b"binary")
        completed = mock.Mock(
            returncode=0,
            stdout=json.dumps(
                {"schema": 1, "mode": "find", "reusable": True, "receipts": []}
            ),
            stderr="",
        )
        with mock.patch.object(evidence.subprocess, "run", return_value=completed):
            with self.assertRaisesRegex(SystemExit, "invalid schema"):
                evidence._run_zparity_cold_evidence(
                    ["find", str(self.root), str(self.root / "request.json")],
                    zparity=zparity,
                )

    def test_promotion_requires_independent_evidence_and_does_not_add_engine_state(self) -> None:
        second_session = self.copy_session(
            "run-100-exact-invocation-b", "invocation-b", "b" * 64
        )
        receipts = [
            {
                "invocation_id": "invocation-a",
                "run_nonce": "a" * 64,
                "receipt_path": str(self.root / "a.json"),
                "receipt_sha256": "1" * 64,
                "session_path": str(self.session),
                "target_frames": 100,
                "authority": self.authority,
            },
            {
                "invocation_id": "invocation-b",
                "run_nonce": "b" * 64,
                "receipt_path": str(self.root / "b.json"),
                "receipt_sha256": "2" * 64,
                "session_path": str(second_session),
                "target_frames": 100,
                "authority": self.authority,
            },
        ]
        ledger_path = self.root / "frontier.json"
        ledger_path.write_text(
            json.dumps(
                {
                    "schema": 1,
                    "project": "routes/full_run",
                    "policy": {"required_cold_confirmations": 2},
                    "promoted": {"last_exact_engine_state_frame": 77},
                }
            ),
            encoding="utf-8",
        )
        with (
            mock.patch.object(
                evidence,
                "git_identity",
                return_value={"clean": True, "head": "f" * 40},
            ),
            mock.patch.object(
                evidence, "list_verified_cold_passes", return_value=receipts
            ),
            mock.patch.object(
                evidence,
                "staged_source_authority",
                return_value=self.current_source_authority(),
            ),
        ):
            ledger = evidence.promote_frontier(
                ledger_path=ledger_path,
                binary=self.binary,
                pass_root=self.root / "passes",
                zparity=self.root / "zparity",
            )

        confirmations = ledger["promoted"]["cold_confirmation_receipts"]
        self.assertEqual({item["invocation_id"] for item in confirmations}, {"invocation-a", "invocation-b"})
        self.assertEqual(ledger["promoted"]["last_exact_engine_state_frame"], 77)
        self.assertEqual(ledger["promoted"]["last_exact_video_frame"], 100)

    def test_promotion_rejects_copied_invocation_or_session_evidence(self) -> None:
        different_session = self.copy_session(
            "different-session", "invocation-a", "b" * 64
        )
        receipts = [
            {
                "invocation_id": "invocation-a",
                "run_nonce": "a" * 64,
                "receipt_path": str(self.root / "a.json"),
                "receipt_sha256": "1" * 64,
                "session_path": str(self.session),
                "target_frames": 100,
                "authority": self.authority,
            },
            {
                "invocation_id": "invocation-a",
                "run_nonce": "b" * 64,
                "receipt_path": str(self.root / "copied-invocation.json"),
                "receipt_sha256": "2" * 64,
                "session_path": str(different_session),
                "target_frames": 100,
                "authority": self.authority,
            },
        ]
        with (
            mock.patch.object(
                evidence,
                "git_identity",
                return_value={"clean": True, "head": "f" * 40},
            ),
            mock.patch.object(
                evidence, "list_verified_cold_passes", return_value=receipts
            ),
            mock.patch.object(
                evidence,
                "staged_source_authority",
                return_value=self.current_source_authority(),
            ),
        ):
            with self.assertRaisesRegex(SystemExit, "fewer than two independent"):
                evidence.promote_frontier(
                    ledger_path=self.root / "frontier.json",
                    binary=self.binary,
                    pass_root=self.root / "passes",
                    zparity=self.root / "zparity",
                )

    def test_promotion_rejects_reissued_copy_with_unique_strings(self) -> None:
        copied = self.root / "copied-session"
        shutil.copytree(self.session, copied)
        copied_manifest_path = copied / "manifest.json"
        copied_manifest = json.loads(
            copied_manifest_path.read_text(encoding="utf-8")
        )
        copied_manifest["cold_evidence_invocation_id"] = "invocation-reissued"
        # Copying a run cannot create a new runner-authored nonce.
        copied_manifest_path.write_text(
            json.dumps(copied_manifest), encoding="utf-8"
        )
        receipts = [
            {
                "invocation_id": invocation,
                "run_nonce": "a" * 64,
                "receipt_path": str(self.root / f"{invocation}.json"),
                "receipt_sha256": digest * 64,
                "session_path": str(session),
                "target_frames": 100,
                "authority": self.authority,
            }
            for invocation, digest, session in (
                ("invocation-a", "1", self.session),
                ("invocation-reissued", "2", copied),
            )
        ]
        with (
            mock.patch.object(
                evidence,
                "git_identity",
                return_value={"clean": True, "head": "f" * 40},
            ),
            mock.patch.object(
                evidence, "list_verified_cold_passes", return_value=receipts
            ),
            mock.patch.object(
                evidence,
                "staged_source_authority",
                return_value=self.current_source_authority(),
            ),
        ):
            with self.assertRaisesRegex(SystemExit, "fewer than two independent"):
                evidence.promote_frontier(
                    ledger_path=self.root / "frontier.json",
                    binary=self.binary,
                    pass_root=self.root / "passes",
                    zparity=self.root / "zparity",
                )

    def test_promotion_rejects_source_or_build_drift_with_same_binary(self) -> None:
        second_session = self.copy_session(
            "second-session", "invocation-b", "2" * 64
        )
        receipts = [
            {
                "invocation_id": invocation,
                "run_nonce": digest * 64,
                "receipt_path": str(self.root / f"{invocation}.json"),
                "receipt_sha256": digest * 64,
                "session_path": str(session),
                "target_frames": 100,
                "authority": self.authority,
            }
            for invocation, digest, session in (
                ("invocation-a", "1", self.session),
                ("invocation-b", "2", second_session),
            )
        ]
        current = self.current_source_authority()
        current["identity"]["content_inventory_sha256"] = "6" * 64
        current["identity_sha256"] = evidence.stable_hash(current["identity"])
        with (
            mock.patch.object(
                evidence,
                "git_identity",
                return_value={"clean": True, "head": "f" * 40},
            ),
            mock.patch.object(
                evidence, "list_verified_cold_passes", return_value=receipts
            ),
            mock.patch.object(
                evidence, "staged_source_authority", return_value=current
            ),
        ):
            with self.assertRaisesRegex(SystemExit, "fewer than two independent"):
                evidence.promote_frontier(
                    ledger_path=self.root / "frontier.json",
                    binary=self.binary,
                    pass_root=self.root / "passes",
                    zparity=self.root / "zparity",
                )

        current = self.current_source_authority()
        current["build_binding"] = {"schema": 1, "profile": "different"}
        current["build_binding_sha256"] = evidence.stable_hash(
            current["build_binding"]
        )
        with (
            mock.patch.object(
                evidence,
                "git_identity",
                return_value={"clean": True, "head": "f" * 40},
            ),
            mock.patch.object(
                evidence, "list_verified_cold_passes", return_value=receipts
            ),
            mock.patch.object(
                evidence, "staged_source_authority", return_value=current
            ),
        ):
            with self.assertRaisesRegex(SystemExit, "fewer than two independent"):
                evidence.promote_frontier(
                    ledger_path=self.root / "frontier.json",
                    binary=self.binary,
                    pass_root=self.root / "passes",
                    zparity=self.root / "zparity",
                )

    def test_workspace_inventory_binds_nonignored_file_content(self) -> None:
        source = self.root / "source.rs"
        source.write_bytes(b"before")

        def command(command: list[str], _label: str) -> bytes:
            if "--stage" in command:
                return b"100644 deadbeef 0\tsource.rs\0"
            if "status" in command:
                return b" M source.rs\0"
            return b"source.rs\0"

        with (
            mock.patch.object(evidence, "ROOT", self.root),
            mock.patch.object(evidence, "_command_bytes", side_effect=command),
            mock.patch.object(evidence, "git_output", return_value="a" * 40),
        ):
            before = evidence._workspace_content_identity()
            source.write_bytes(b"after")
            after = evidence._workspace_content_identity()

        self.assertNotEqual(
            before["content_inventory_sha256"], after["content_inventory_sha256"]
        )


if __name__ == "__main__":
    unittest.main()


class CachedAvPromotionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.binary = self.root / "zelda3"
        self.binary.write_bytes(b"binary")
        self.cache = self.root / "cache"
        self.cache.mkdir()
        self.cache_manifest = {
            "schema": 1,
            "kind": "zelda3-content-addressed-oracle-evidence",
            "cache_key": "k" * 64,
            "oracle_av_hash_frames": 100,
            "cache_identity": {
                "core_sha256": "c" * 64,
                "rom_sha256": "d" * 64,
                "oracle_initial_state_sha256": "e" * 64,
                "source_artifact_sha256": {
                    "initial.srm": "1" * 64,
                    "input.txt": "2" * 64,
                    "rom-random.txt": "3" * 64,
                },
            },
        }
        (self.cache / "cache-manifest.json").write_text(
            json.dumps(self.cache_manifest), encoding="utf-8"
        )
        self.run = self.root / "run-cached"
        self.run.mkdir()
        (self.run / "av_hashes.jsonl").write_text("{}\n" * 100, encoding="utf-8")
        self.ledger = self.root / "parity-frontier.json"
        self.ledger.write_text(
            json.dumps(
                {
                    "schema": evidence.FRONTIER_SCHEMA,
                    "project": "routes/full_run",
                    "policy": {"required_cold_confirmations": 2},
                    "promoted": {"last_exact_engine_state_frame": 7, "last_exact_video_frame": 50},
                }
            ),
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_run_manifest(self, **overrides) -> None:
        manifest = {
            "schema": 1,
            "kind": "zelda3-rust-only-cached-snes9x-av-replay",
            "binary_sha256": evidence.sha256_file(self.binary),
            "oracle_cache": str(self.cache),
            "oracle_cache_key": "k" * 64,
            "oracle_cache_manifest_sha256": evidence.sha256_file(
                self.cache / "cache-manifest.json"
            ),
            "rom": {"path": "rom.sfc", "sha256": "d" * 64},
            "start_frame": 0,
            "compare_from_frame": 0,
            "frames_completed": 100,
            "frames_compared": 100,
            "stop_before_frame": None,
            "resume_paired": None,
            "comparison_lanes": {"video": True, "audio": True},
            "matched": True,
            "first_rng_drift": None,
            "candidate_ledger": "av_hashes.jsonl",
        }
        manifest.update(overrides)
        (self.run / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    def promote(self):
        with mock.patch.object(
            evidence, "git_identity", return_value={"clean": True, "head": "h" * 40}
        ):
            return evidence.promote_frontier_from_cached_av(
                self.run, ledger_path=self.ledger, binary=self.binary
            )

    def test_full_route_cached_av_pass_promotes_video_and_audio(self) -> None:
        self.write_run_manifest()
        ledger = self.promote()
        promoted = ledger["promoted"]
        self.assertEqual(promoted["kind"], "cached_av_pass")
        self.assertEqual(promoted["last_exact_video_frame"], 99)
        self.assertEqual(promoted["last_exact_audio_frame"], 99)
        self.assertEqual(promoted["last_exact_engine_state_frame"], 7)
        self.assertEqual(promoted["binary_sha256"], evidence.sha256_file(self.binary))
        self.assertEqual(promoted["route_signature"]["core_sha256"], "c" * 64)
        self.assertEqual(promoted["cached_av_receipt"]["frames"], 100)
        receipt_copy = self.ledger.parent / promoted["cached_av_receipt"]["receipt_path"]
        self.assertTrue(receipt_copy.is_file())
        self.assertEqual(
            evidence.sha256_file(receipt_copy), promoted["cached_av_receipt"]["manifest_sha256"]
        )
        self.assertIn(
            "one full-route Rust-only cached Snes9x A/V pass (policy 2026-09-06)",
            ledger["policy"]["accepted_exact_av_receipts"],
        )
        self.assertEqual(json.loads(self.ledger.read_text())["promoted"]["kind"], "cached_av_pass")

    def test_partial_resumed_or_mismatched_runs_are_rejected(self) -> None:
        for overrides in (
            {"matched": False},
            {"start_frame": 5, "compare_from_frame": 5},
            {"resume_paired": "/somewhere/frame-00000005"},
            {"frames_compared": 99},
            {"stop_before_frame": 60},
            {"comparison_lanes": {"video": True, "audio": False}},
            {"binary_sha256": "f" * 64},
            {"rom": {"path": "rom.sfc", "sha256": "0" * 64}},
        ):
            self.write_run_manifest(**overrides)
            with self.assertRaises(SystemExit, msg=str(overrides)):
                self.promote()

    def test_dirty_tree_is_rejected(self) -> None:
        self.write_run_manifest()
        with mock.patch.object(evidence, "git_identity", return_value={"clean": False}):
            with self.assertRaises(SystemExit):
                evidence.promote_frontier_from_cached_av(
                    self.run, ledger_path=self.ledger, binary=self.binary
                )


class EngineStateFrontierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.session = self.root / "run-100-rng-calibration-x"
        self.session.mkdir()
        self.ledger = self.root / "parity-frontier.json"
        self.ledger.write_text(
            json.dumps(
                {
                    "schema": evidence.FRONTIER_SCHEMA,
                    "project": "routes/full_run",
                    "policy": {},
                    "promoted": {"last_exact_engine_state_frame": 7, "last_exact_video_frame": 99},
                }
            ),
            encoding="utf-8",
        )
        self.prefix = self.root / "prefix.json"
        self.prefix.write_text(
            json.dumps({"authority": {"target_frames": 40}}), encoding="utf-8"
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_session(self, *, status="passed", from_frame=40, completed=100, matched=True) -> None:
        (self.session / "manifest.json").write_text(
            json.dumps(
                {
                    "status": status,
                    "parity_eligible": True,
                    "frames_completed": completed,
                    "timing": {"frames_requested": 100},
                    "comparison_lanes": {
                        "engine_state": True,
                        "engine_state_from_frame": from_frame,
                        "video": False,
                        "audio": False,
                    },
                }
            ),
            encoding="utf-8",
        )
        (self.session / "result.json").write_text(
            json.dumps(
                {"status": status, "engine_state": {"matched": matched, "first_mismatch": None}}
            ),
            encoding="utf-8",
        )

    def test_passed_session_with_prefix_records_the_route_end(self) -> None:
        self.write_session()
        with mock.patch.object(
            evidence, "git_identity", return_value={"clean": True, "head": "h" * 40}
        ):
            ledger = evidence.record_engine_state_frontier(
                self.session, ledger_path=self.ledger, prefix_receipt=self.prefix
            )
        promoted = ledger["promoted"]
        self.assertEqual(promoted["last_exact_engine_state_frame"], 99)
        self.assertEqual(promoted["last_exact_video_frame"], 99)
        self.assertEqual(len(promoted["engine_state_receipts"]), 2)
        self.assertEqual(promoted["engine_state_receipts"][1]["frames"], 40)

    def test_continuation_without_prefix_or_failed_session_is_rejected(self) -> None:
        self.write_session()
        with mock.patch.object(
            evidence, "git_identity", return_value={"clean": True, "head": "h" * 40}
        ):
            with self.assertRaises(SystemExit):
                evidence.record_engine_state_frontier(self.session, ledger_path=self.ledger)
            self.write_session(matched=False)
            with self.assertRaises(SystemExit):
                evidence.record_engine_state_frontier(
                    self.session, ledger_path=self.ledger, prefix_receipt=self.prefix
                )
            self.write_session(completed=90)
            with self.assertRaises(SystemExit):
                evidence.record_engine_state_frontier(
                    self.session, ledger_path=self.ledger, prefix_receipt=self.prefix
                )
