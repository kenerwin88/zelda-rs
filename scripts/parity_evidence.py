#!/usr/bin/env python3
"""Shared provenance, promotion, and immutable-oracle evidence helpers.

This module deliberately lives in verification tooling.  The Zelda runtime never
reads a route, ROM, trace, or cached oracle artifact.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PROJECT = ROOT / "routes" / "full_run"
DEFAULT_LEDGER = DEFAULT_PROJECT / "parity-frontier.json"
PASS_ROOT = ROOT / ".git" / "parity-cold-passes"
ORACLE_CACHE_ROOT = ROOT / ".git" / "parity-oracle-cache"
ZPARITY = ROOT / "target" / "parity" / "zparity"
LEGACY_PASS_SCHEMA = 1
PASS_SCHEMA = 2
COLD_EVIDENCE_KIND = "zelda3-cold-parity-pass"
COLD_EVIDENCE_REQUEST_KIND = "zelda3-cold-parity-reuse-request"
CLEAN_ENV_EXECUTION_POLICY = "clean_env_v1"
EMPTY_RUNTIME_CONFIG_SHA256 = hashlib.sha256(b"").hexdigest()
CACHE_SCHEMA = 1
FRONTIER_SCHEMA = 1
PARITY_BUILD_COMMAND = (
    "cargo",
    "build",
    "--profile",
    "parity",
    "-p",
    "zelda3-bin",
    "-p",
    "parity",
)

ORACLE_SESSION_FILES = (
    "oracle_initial.state",
    "oracle_last_before.state",
    "oracle_final.state",
    "semantic-trace-final.checkpoint.json",
    "snes9x-trace.jsonl",
    "display_oracle.jsonl",
    "obj_state_ledger.jsonl",
    "original-timing-host-receipts.jsonl.zst",
)
REPLAY_SOURCE_FILES = ("input.txt", "rom-random.txt", "initial.srm")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def stable_hash(value: object) -> str:
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"parity evidence: cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise SystemExit(f"parity evidence: {path} is not a JSON object")
    return value


def atomic_write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            json.dump(value, stream, indent=2, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()


def git_output(*arguments: str) -> str:
    return subprocess.run(
        ["git", *arguments],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    ).stdout.strip()


def git_identity() -> dict[str, Any]:
    status = git_output("status", "--porcelain=v1", "--untracked-files=all")
    diff = subprocess.run(
        ["git", "diff", "--binary", "HEAD"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    ).stdout
    return {
        "head": git_output("rev-parse", "HEAD"),
        "describe": git_output("describe", "--always", "--dirty"),
        "clean": not status,
        "status_sha256": hashlib.sha256(status.encode("utf-8")).hexdigest(),
        "tracked_diff_sha256": hashlib.sha256(diff).hexdigest(),
    }


def _command_bytes(command: list[str], label: str) -> bytes:
    process = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    if process.returncode != 0:
        detail = process.stderr.decode("utf-8", errors="replace").strip()
        raise SystemExit(f"parity evidence: {label} failed: {detail}")
    return process.stdout


def _workspace_content_identity() -> dict[str, Any]:
    """Hash every tracked or nonignored untracked build/runtime input."""
    paths = _command_bytes(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        "git source inventory",
    ).split(b"\0")
    paths = sorted(path for path in paths if path)
    digest = hashlib.sha256()
    root_bytes = os.fsencode(ROOT)
    for relative in paths:
        absolute = os.path.join(root_bytes, relative)
        try:
            metadata = os.lstat(absolute)
        except OSError as error:
            raise SystemExit(
                "parity evidence: source inventory changed while hashing "
                f"{os.fsdecode(relative)}: {error}"
            ) from error
        digest.update(len(relative).to_bytes(8, "big"))
        digest.update(relative)
        digest.update((metadata.st_mode & 0o177777).to_bytes(4, "big"))
        if stat.S_ISREG(metadata.st_mode):
            content = hashlib.sha256()
            with open(absolute, "rb") as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                    content.update(chunk)
            current = os.lstat(absolute)
            if (
                metadata.st_dev,
                metadata.st_ino,
                metadata.st_size,
                metadata.st_mtime_ns,
            ) != (
                current.st_dev,
                current.st_ino,
                current.st_size,
                current.st_mtime_ns,
            ):
                raise SystemExit(
                    "parity evidence: source inventory changed while hashing "
                    f"{os.fsdecode(relative)}"
                )
            payload_hash = content.digest()
        elif stat.S_ISLNK(metadata.st_mode):
            target = os.readlink(absolute)
            payload_hash = hashlib.sha256(os.fsencode(target)).digest()
        elif stat.S_ISDIR(metadata.st_mode):
            # Gitlinks appear as directory entries in the worktree. Their exact
            # staged object is separately bound by the index inventory below.
            payload_hash = hashlib.sha256(b"gitlink-directory").digest()
        else:
            raise SystemExit(
                "parity evidence: unsupported source input type: "
                f"{os.fsdecode(relative)}"
            )
        digest.update(payload_hash)
    return {
        "schema": 1,
        "head": git_output("rev-parse", "HEAD"),
        "file_count": len(paths),
        "content_inventory_sha256": digest.hexdigest(),
        "index_inventory_sha256": hashlib.sha256(
            _command_bytes(["git", "ls-files", "--stage", "-z"], "git index inventory")
        ).hexdigest(),
        "status_sha256": hashlib.sha256(
            _command_bytes(
                ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
                "git status inventory",
            )
        ).hexdigest(),
    }


def staged_source_authority() -> dict[str, Any]:
    identity = _workspace_content_identity()
    build_environment_names = (
        "AR",
        "CC",
        "CFLAGS",
        "CXX",
        "CXXFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "MACOSX_DEPLOYMENT_TARGET",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTFLAGS",
        "SDKROOT",
        "ZELDA3_EMBEDDED_ASSETS",
    )
    build_binding = {
        "schema": 1,
        "profile": "parity",
        "command": list(PARITY_BUILD_COMMAND),
        "rustc_verbose_version": _command_bytes(
            ["rustc", "-vV"], "rustc version"
        ).decode("utf-8", errors="strict").strip(),
        "cargo_version": _command_bytes(
            ["cargo", "--version", "--verbose"], "cargo version"
        ).decode("utf-8", errors="strict").strip(),
        "build_environment": {
            name: {
                "present": name in os.environ,
                "value_sha256": hashlib.sha256(
                    os.environ.get(name, "").encode("utf-8", errors="surrogateescape")
                ).hexdigest()
                if name in os.environ
                else None,
            }
            for name in build_environment_names
        },
    }
    return {
        "identity": identity,
        "identity_sha256": stable_hash(identity),
        "build_binding": build_binding,
        "build_binding_sha256": stable_hash(build_binding),
    }


def artifact_hashes(directory: Path, names: Iterable[str]) -> dict[str, str]:
    return {
        name: sha256_file(directory / name)
        for name in names
        if (directory / name).is_file()
    }


def _required_hash(directory: Path, name: str) -> str:
    path = directory / name
    if not path.is_file():
        raise SystemExit(f"parity evidence: required artifact is missing: {path}")
    return sha256_file(path)


def session_identity(session: Path) -> dict[str, Any]:
    manifest = load_json(session / "manifest.json")
    result = load_json(session / "result.json")
    core_sha = manifest.get("core", {}).get("sha256")
    rom_sha = manifest.get("rom", {}).get("sha256")
    if not isinstance(core_sha, str) or len(core_sha) != 64:
        raise SystemExit(f"parity evidence: {session}/manifest.json has no core hash")
    if not isinstance(rom_sha, str) or len(rom_sha) != 64:
        raise SystemExit(f"parity evidence: {session}/manifest.json has no ROM hash")
    rng = manifest.get("rom_random_replay")
    if isinstance(rng, dict) and (session / "rom-random.txt").is_file():
        actual = sha256_file(session / "rom-random.txt")
        if rng.get("sha256") != actual:
            raise SystemExit(
                f"parity evidence: {session}/rom-random.txt does not match its manifest"
            )
    timing = manifest.get("timing") if isinstance(manifest.get("timing"), dict) else {}
    comparison_lanes = (
        manifest.get("comparison_lanes")
        if isinstance(manifest.get("comparison_lanes"), dict)
        else {}
    )
    return {
        "core_sha256": core_sha,
        "rom_sha256": rom_sha,
        "frames_requested": int(timing.get("frames_requested", result.get("frames_completed", 0))),
        "frames_completed": int(result.get("frames_completed", manifest.get("frames_completed", 0))),
        "start_frame": int(timing.get("start_frame", 0)),
        "compare_from_frame": int(timing.get("compare_from_frame", 0)),
        "comparison_lanes": {
            "video": bool(comparison_lanes.get("video", False)),
            "audio": bool(comparison_lanes.get("audio", False)),
        },
        "parity_eligible": bool(result.get("parity_eligible")),
        "status": result.get("status"),
        "video": result.get("video"),
        "audio": result.get("audio"),
        "source_artifact_sha256": artifact_hashes(session, REPLAY_SOURCE_FILES),
        "manifest_sha256": sha256_file(session / "manifest.json"),
        "result_sha256": sha256_file(session / "result.json"),
    }


def _lane_matched(value: object) -> bool:
    return isinstance(value, dict) and value.get("matched") is True


def schema2_session_identity(session: Path) -> dict[str, Any]:
    return {
        "manifest_sha256": _required_hash(session, "manifest.json"),
        "result_sha256": _required_hash(session, "result.json"),
        "source_artifact_sha256": {
            name: _required_hash(session, name) for name in REPLAY_SOURCE_FILES
        },
    }


def record_cold_pass(
    *,
    session: Path,
    route_signature: dict[str, Any],
    binary: Path,
    output_root: Path = PASS_ROOT,
    authority: dict[str, Any] | None = None,
    invocation_id: str | None = None,
) -> Path:
    """Record one exact cold A/V pass after the gate has accepted it."""
    identity = session_identity(session)
    if identity["status"] != "passed" or not identity["parity_eligible"]:
        raise SystemExit("parity evidence: refusing to record a non-passing session")
    if identity["start_frame"] != 0:
        raise SystemExit("parity evidence: refusing to record a resumed session as cold proof")
    if not _lane_matched(identity["video"]) or not _lane_matched(identity["audio"]):
        raise SystemExit("parity evidence: cold promotion requires exact video and audio lanes")
    if not binary.is_file():
        raise SystemExit(f"parity evidence: parity binary is missing: {binary}")
    created_ns = time.time_ns()
    if authority is not None:
        if not invocation_id:
            raise SystemExit("parity evidence: schema 2 receipt requires an invocation ID")
        manifest = load_json(session / "manifest.json")
        if manifest.get("cold_evidence_invocation_id") != invocation_id:
            raise SystemExit(
                "parity evidence: session invocation ID does not match the receipt"
            )
        run_nonce = manifest.get("cold_evidence_run_nonce")
        if (
            not isinstance(run_nonce, str)
            or len(run_nonce) != 64
            or any(character not in "0123456789abcdef" for character in run_nonce)
        ):
            raise SystemExit(
                "parity evidence: session has no valid runner-authored cold run nonce"
            )
        if authority.get("target_frames") != identity["frames_completed"]:
            raise SystemExit(
                "parity evidence: schema 2 authority target does not match the session"
            )
        receipt = {
            "schema": PASS_SCHEMA,
            "kind": COLD_EVIDENCE_KIND,
            "created_unix_ns": created_ns,
            "invocation_id": invocation_id,
            "run_nonce": run_nonce,
            "authority": authority,
            "session": str(session.resolve()),
            "session_identity": schema2_session_identity(session),
        }
        receipt_hash = stable_hash(receipt)
        output = output_root / (
            f"{created_ns}-{identity['frames_completed']}-{receipt_hash[:12]}.json"
        )
        atomic_write_json(output, receipt)
        return output

    # Historical callers can still write their legacy diagnostic receipts.
    # Rust's schema-2 verifier rejects these for both reuse and promotion.
    receipt: dict[str, Any] = {
        "schema": LEGACY_PASS_SCHEMA,
        "kind": COLD_EVIDENCE_KIND,
        "created_unix_ns": created_ns,
        "route_signature": route_signature,
        "route_signature_sha256": stable_hash(route_signature),
        "binary": {
            "path": str(binary.resolve()),
            "sha256": sha256_file(binary),
            "size": binary.stat().st_size,
        },
        "git": git_identity(),
        "proof": {
            "cold": True,
            "exact_video": True,
            "exact_audio": True,
            "frames": identity["frames_completed"],
        },
        "session": str(session.resolve()),
        "session_identity": identity,
    }
    receipt_hash = stable_hash(receipt)
    output = output_root / (
        f"{created_ns}-{identity['frames_completed']}-{receipt_hash[:12]}.json"
    )
    atomic_write_json(output, receipt)
    return output


def load_cold_passes(root: Path = PASS_ROOT) -> list[tuple[Path, dict[str, Any]]]:
    receipts: list[tuple[Path, dict[str, Any]]] = []
    if not root.is_dir():
        return receipts
    for path in sorted(root.glob("*.json")):
        receipt = load_json(path)
        if receipt.get("schema") in (LEGACY_PASS_SCHEMA, PASS_SCHEMA) and receipt.get(
            "kind"
        ) == COLD_EVIDENCE_KIND:
            receipts.append((path, receipt))
    return receipts


def _run_zparity_cold_evidence(
    arguments: list[str], *, zparity: Path = ZPARITY
) -> dict[str, Any]:
    if not zparity.is_file():
        raise SystemExit(
            f"parity evidence: verifier missing ({zparity}); build target/parity/zparity first"
        )
    process = subprocess.run(
        [str(zparity), "cold-evidence", *arguments],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if process.returncode != 0:
        raise SystemExit(
            "parity evidence: zparity cold-evidence verification failed: "
            f"{process.stderr.strip()}"
        )
    try:
        output = json.loads(process.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(
            f"parity evidence: invalid zparity cold-evidence output: {error}"
        ) from error
    if not isinstance(output, dict) or output.get("schema") != PASS_SCHEMA:
        raise SystemExit("parity evidence: zparity returned an invalid schema")
    return output


def find_reusable_cold_pass(
    authority: dict[str, Any],
    *,
    pass_root: Path = PASS_ROOT,
    zparity: Path = ZPARITY,
) -> dict[str, Any] | None:
    request = {
        "schema": PASS_SCHEMA,
        "kind": COLD_EVIDENCE_REQUEST_KIND,
        "authority": authority,
    }
    with tempfile.TemporaryDirectory(prefix="zparity-cold-request-") as directory:
        request_path = Path(directory) / "request.json"
        atomic_write_json(request_path, request)
        output = _run_zparity_cold_evidence(
            ["find", str(pass_root), str(request_path)], zparity=zparity
        )
    if output.get("mode") != "find" or not isinstance(output.get("receipts"), list):
        raise SystemExit("parity evidence: zparity returned an invalid find response")
    receipts = output["receipts"]
    if output.get("reusable") is not bool(receipts):
        raise SystemExit("parity evidence: zparity returned an inconsistent find response")
    return receipts[-1] if receipts else None


def list_verified_cold_passes(
    *,
    pass_root: Path = PASS_ROOT,
    zparity: Path = ZPARITY,
) -> list[dict[str, Any]]:
    output = _run_zparity_cold_evidence(["list", str(pass_root)], zparity=zparity)
    if output.get("mode") != "list" or not isinstance(output.get("receipts"), list):
        raise SystemExit("parity evidence: zparity returned an invalid list response")
    return output["receipts"]


def _proof_fingerprint(authority: dict[str, Any]) -> str:
    return stable_hash(
        {
            "authority": authority,
            "proof": {
                "cold": True,
                "exact_video": True,
                "exact_audio": True,
                "engine_state": False,
                "frames": authority.get("target_frames"),
            },
        }
    )


def _committed_source_projection(identity: object) -> dict[str, object] | None:
    """Return source facts which survive the pre-commit -> commit transition.

    A successful pre-commit run necessarily records the old HEAD plus a dirty
    index/status.  After that exact content is committed, HEAD/index/status
    change even though every build/runtime input is byte-identical.  The
    content inventory and its cardinality are the stable authority across that
    transition; all other source-identity fields remain receipt provenance.
    """
    if not isinstance(identity, dict):
        return None
    schema = identity.get("schema")
    file_count = identity.get("file_count")
    content_hash = identity.get("content_inventory_sha256")
    if (
        schema != 1
        or not isinstance(file_count, int)
        or file_count < 0
        or not isinstance(content_hash, str)
        or len(content_hash) != 64
    ):
        return None
    return {
        "schema": schema,
        "file_count": file_count,
        "content_inventory_sha256": content_hash,
    }


def _authority_matches_current_build(
    authority: object,
    current_source: dict[str, Any],
    binary: Path,
) -> bool:
    if not isinstance(authority, dict):
        return False
    receipt_binary = authority.get("binary")
    receipt_source = authority.get("staged_source")
    if not isinstance(receipt_binary, dict) or not isinstance(receipt_source, dict):
        return False
    current_identity = _committed_source_projection(current_source.get("identity"))
    receipt_identity = _committed_source_projection(receipt_source.get("identity"))
    return (
        receipt_binary.get("sha256") == sha256_file(binary)
        and receipt_binary.get("size") == binary.stat().st_size
        and receipt_identity is not None
        and receipt_identity == current_identity
        and receipt_source.get("build_binding") == current_source.get("build_binding")
        and receipt_source.get("build_binding_sha256")
        == current_source.get("build_binding_sha256")
    )


def _valid_run_nonce(value: object) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


def promote_frontier(
    *,
    ledger_path: Path = DEFAULT_LEDGER,
    binary: Path = ROOT / "target" / "parity" / "zelda3",
    pass_root: Path = PASS_ROOT,
    zparity: Path = ZPARITY,
) -> dict[str, Any]:
    """Promote the newest twice-proven binary into the tracked frontier ledger."""
    git = git_identity()
    if not git["clean"]:
        raise SystemExit(
            "parity evidence: promotion requires a clean committed tree; "
            "commit the proven implementation first"
        )
    if not binary.is_file():
        raise SystemExit(f"parity evidence: parity binary is missing: {binary}")
    binary_sha = sha256_file(binary)
    current_source = staged_source_authority()
    compatible = [
        receipt
        for receipt in list_verified_cold_passes(pass_root=pass_root, zparity=zparity)
        if _authority_matches_current_build(
            receipt.get("authority"), current_source, binary
        )
    ]
    grouped: dict[str, list[dict[str, Any]]] = {}
    for receipt in compatible:
        authority = receipt.get("authority")
        if not isinstance(authority, dict):
            continue
        grouped.setdefault(_proof_fingerprint(authority), []).append(receipt)
    candidates: list[list[dict[str, Any]]] = []
    for receipts in grouped.values():
        ordered = sorted(
            receipts, key=lambda item: str(item.get("receipt_path", "")), reverse=True
        )
        invocation_counts: dict[str, int] = {}
        session_counts: dict[str, int] = {}
        run_nonce_counts: dict[str, int] = {}
        for receipt in ordered:
            invocation = receipt.get("invocation_id")
            session = receipt.get("session_path")
            run_nonce = receipt.get("run_nonce")
            if (
                isinstance(invocation, str)
                and isinstance(session, str)
                and _valid_run_nonce(run_nonce)
            ):
                invocation_counts[invocation] = invocation_counts.get(invocation, 0) + 1
                session_counts[session] = session_counts.get(session, 0) + 1
                run_nonce_counts[run_nonce] = run_nonce_counts.get(run_nonce, 0) + 1
        independent = [
            receipt
            for receipt in ordered
            if isinstance(receipt.get("invocation_id"), str)
            and isinstance(receipt.get("session_path"), str)
            and _valid_run_nonce(receipt.get("run_nonce"))
            and invocation_counts[receipt["invocation_id"]] == 1
            and session_counts[receipt["session_path"]] == 1
            and run_nonce_counts[receipt["run_nonce"]] == 1
        ]
        if len(independent) >= 2:
            candidates.append(independent)
    if not candidates:
        raise SystemExit(
            "parity evidence: this binary has fewer than two independent cold exact A/V passes"
        )
    selected = max(
        candidates,
        key=lambda items: int(items[0]["authority"]["target_frames"]),
    )
    first, second = selected[:2]
    promoted_frame = min(int(item["target_frames"]) for item in (first, second))
    ledger = load_json(ledger_path) if ledger_path.is_file() else {
        "schema": FRONTIER_SCHEMA,
        "project": "routes/full_run",
        "policy": {"required_cold_confirmations": 2},
    }
    if ledger.get("schema") != FRONTIER_SCHEMA:
        raise SystemExit(f"parity evidence: unsupported frontier ledger: {ledger_path}")
    previous_engine_state = (
        ledger.get("promoted", {}).get("last_exact_engine_state_frame", 0)
        if isinstance(ledger.get("promoted"), dict)
        else 0
    )
    authority = first["authority"]
    ledger["promoted"] = {
        "commit": git["head"],
        "binary_sha256": binary_sha,
        "route_signature": authority["route_signature"],
        "route_signature_sha256": authority["route_signature_sha256"],
        "authority_sha256": stable_hash(authority),
        "proof_fingerprint": _proof_fingerprint(authority),
        "last_exact_engine_state_frame": previous_engine_state,
        "last_exact_video_frame": promoted_frame,
        "last_exact_audio_frame": promoted_frame,
        "cold_confirmation_receipts": [
            {
                "path": item["receipt_path"],
                "sha256": item["receipt_sha256"],
                "invocation_id": item["invocation_id"],
                "run_nonce": item["run_nonce"],
                "session": item["session_path"],
                "session_result_sha256": sha256_file(
                    Path(item["session_path"]) / "result.json"
                ),
                "frames": item["target_frames"],
            }
            for item in (first, second)
        ],
    }
    atomic_write_json(ledger_path, ledger)
    return ledger


def _oracle_receipt(record: dict[str, Any]) -> dict[str, Any]:
    keep = {
        "frame": record.get("frame"),
        "input": record.get("input"),
        "oracle_audio_sample_frames": record.get("oracle_audio_sample_frames"),
        "oracle_video_width": record.get("oracle_video_width"),
        "oracle_video_height": record.get("oracle_video_height"),
        "oracle_engine": record.get("oracle_engine"),
    }
    vram = record.get("vram")
    if isinstance(vram, dict):
        keep["oracle_vram_words"] = vram.get("oracle_words")
        keep["oracle_vram_sha256"] = vram.get("oracle_sha256")
    return keep


def _extract_oracle_frame_receipts(source: Path, destination: Path) -> int:
    count = 0
    with source.open(encoding="utf-8") as input_stream, destination.open(
        "w", encoding="utf-8"
    ) as output_stream:
        for line_number, line in enumerate(input_stream, 1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(
                    f"parity evidence: invalid frame receipt {source}:{line_number}: {error}"
                ) from error
            if not isinstance(record, dict) or not isinstance(record.get("oracle_engine"), dict):
                raise SystemExit(
                    f"parity evidence: {source}:{line_number} has no oracle engine receipt"
                )
            output_stream.write(json.dumps(_oracle_receipt(record), sort_keys=True) + "\n")
            count += 1
    return count


def _extract_oracle_av_hashes(source: Path, destination: Path) -> int:
    """Strip Rust hashes while preserving the frame/input provenance join key."""
    count = 0
    previous_frame: int | None = None
    with source.open(encoding="utf-8") as input_stream, destination.open(
        "w", encoding="utf-8"
    ) as output_stream:
        for line_number, line in enumerate(input_stream, 1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(
                    f"parity evidence: invalid A/V hash record {source}:{line_number}: {error}"
                ) from error
            frame = record.get("frame") if isinstance(record, dict) else None
            input_value = record.get("input") if isinstance(record, dict) else None
            if (
                not isinstance(record, dict)
                or record.get("schema") != 1
                or not isinstance(frame, int)
                or not isinstance(input_value, str)
                or (previous_frame is not None and frame <= previous_frame)
            ):
                raise SystemExit(
                    f"parity evidence: malformed or non-monotonic A/V hash record "
                    f"{source}:{line_number}"
                )
            oracle_record: dict[str, Any] = {
                "schema": 1,
                "frame": frame,
                "input": input_value,
                "oracle_audio_sample_frames": record.get("oracle_audio_sample_frames"),
                "video": None,
                "audio": None,
            }
            enabled_lanes = 0
            for lane in ("video", "audio"):
                lane_record = record.get(lane)
                if lane_record is None:
                    continue
                oracle = lane_record.get("oracle") if isinstance(lane_record, dict) else None
                if not isinstance(oracle, dict) or not isinstance(oracle.get("sha256"), str):
                    raise SystemExit(
                        f"parity evidence: {source}:{line_number} has no oracle {lane} digest"
                    )
                oracle_record[lane] = oracle
                enabled_lanes += 1
            if enabled_lanes == 0:
                raise SystemExit(
                    f"parity evidence: {source}:{line_number} has no enabled A/V lane"
                )
            output_stream.write(json.dumps(oracle_record, sort_keys=True) + "\n")
            previous_frame = frame
            count += 1
    return count


def _av_hash_evidence_schema(source: Path) -> int:
    if not source.is_file():
        return 0
    try:
        with source.open(encoding="utf-8") as stream:
            first = json.loads(stream.readline())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"parity evidence: cannot inspect A/V ledger {source}: {error}") from error
    if not isinstance(first, dict):
        raise SystemExit(f"parity evidence: malformed A/V ledger: {source}")
    return 2 if isinstance(first.get("oracle_audio_sample_frames"), int) else 1


def verify_oracle_cache_entry(cache: Path, manifest: dict[str, Any] | None = None) -> None:
    if manifest is None:
        manifest = load_json(cache / "cache-manifest.json")
    for name, expected in manifest.get("artifact_sha256", {}).items():
        relative = Path(name)
        if relative.is_absolute() or not relative.parts or any(
            part in {"", ".", ".."} for part in relative.parts
        ):
            raise SystemExit(f"parity evidence: unsafe artifact name {name!r}")
        path = cache / name
        current = cache
        safe = True
        for index, part in enumerate(relative.parts):
            current = current / part
            if current.is_symlink():
                safe = False
                break
            if index + 1 == len(relative.parts):
                safe = current.is_file()
            else:
                safe = current.is_dir()
            if not safe:
                break
        if not safe or sha256_file(path) != expected:
            raise SystemExit(f"parity evidence: immutable oracle cache is corrupt: {path}")


def verify_oracle_cache_root(cache_root: Path = ORACLE_CACHE_ROOT) -> dict[str, int]:
    """Verify every immutable cache entry and return an inventory summary."""
    if not cache_root.exists():
        return {"entries": 0, "artifacts": 0, "bytes": 0}
    if not cache_root.is_dir():
        raise SystemExit(f"parity evidence: oracle cache root is not a directory: {cache_root}")
    entries = artifacts = total_bytes = 0
    for cache in sorted(path for path in cache_root.iterdir() if path.is_dir()):
        manifest_path = cache / "cache-manifest.json"
        if not manifest_path.is_file():
            raise SystemExit(f"parity evidence: incomplete oracle cache entry: {cache}")
        manifest = load_json(manifest_path)
        if manifest.get("schema") != CACHE_SCHEMA or manifest.get("kind") != "zelda3-content-addressed-oracle-evidence":
            raise SystemExit(f"parity evidence: unsupported oracle cache manifest: {manifest_path}")
        if manifest.get("cache_key") != cache.name:
            raise SystemExit(f"parity evidence: oracle cache directory/key mismatch: {cache}")
        if stable_hash(manifest.get("cache_identity")) != cache.name:
            raise SystemExit(f"parity evidence: oracle cache identity hash mismatch: {cache}")
        verify_oracle_cache_entry(cache, manifest)
        hashes = manifest.get("artifact_sha256", {})
        if not isinstance(hashes, dict):
            raise SystemExit(f"parity evidence: invalid artifact inventory: {manifest_path}")
        entries += 1
        artifacts += len(hashes)
        total_bytes += sum((cache / name).stat().st_size for name in hashes)
    return {"entries": entries, "artifacts": artifacts, "bytes": total_bytes}


def cache_oracle_session(
    session: Path,
    *,
    cache_root: Path = ORACLE_CACHE_ROOT,
    trace_configuration: dict[str, Any] | None = None,
) -> tuple[Path, bool]:
    """Extract oracle-only artifacts into a tamper-evident content-addressed cache."""
    session = session.resolve()
    identity = session_identity(session)
    frame_receipts = session / "frame_receipts.jsonl"
    av_hashes = session / "av_hashes.jsonl"
    timing_host_receipts = session / "original-timing-host-receipts.jsonl.zst"
    semantic_trace_checkpoint = session / "semantic-trace-final.checkpoint.json"
    oracle_checkpoints = session / "oracle-checkpoints"
    session_manifest = load_json(session / "manifest.json")
    timing_host_receipts_schema = 0
    if timing_host_receipts.is_file():
        timing_host_receipts_schema = (
            session_manifest.get("original_timing_host_receipts", {}).get("schema")
        )
        if not isinstance(timing_host_receipts_schema, int) or timing_host_receipts_schema <= 0:
            raise SystemExit(
                "parity evidence: source timing receipts require a positive manifest schema"
            )
    semantic_trace_checkpoint_schema = 0
    if semantic_trace_checkpoint.is_file():
        semantic_trace_checkpoint_schema = load_json(semantic_trace_checkpoint).get("schema")
        if (
            not isinstance(semantic_trace_checkpoint_schema, int)
            or semantic_trace_checkpoint_schema <= 0
        ):
            raise SystemExit(
                "parity evidence: semantic trace checkpoint requires a positive schema"
            )
    oracle_checkpoint_artifacts = {
        str(path.relative_to(session)): sha256_file(path)
        for path in sorted(oracle_checkpoints.rglob("*"))
        if path.is_file()
    } if oracle_checkpoints.is_dir() else {}
    cache_identity = {
        "schema": CACHE_SCHEMA,
        "core_sha256": identity["core_sha256"],
        "rom_sha256": identity["rom_sha256"],
        "frames_requested": identity["frames_requested"],
        "start_frame": identity["start_frame"],
        "compare_from_frame": identity["compare_from_frame"],
        "comparison_lanes": identity["comparison_lanes"],
        "source_artifact_sha256": identity["source_artifact_sha256"],
        "oracle_initial_state_sha256": (
            sha256_file(session / "oracle_initial.state")
            if (session / "oracle_initial.state").is_file()
            else None
        ),
        "trace_configuration": trace_configuration or {},
        "oracle_evidence": {
            "semantic_receipts_schema": 1 if frame_receipts.is_file() else 0,
            "timing_host_receipts_schema": timing_host_receipts_schema,
            "semantic_trace_checkpoint_schema": semantic_trace_checkpoint_schema,
            "canonical_av_hash_ledger_schema": _av_hash_evidence_schema(av_hashes),
            "oracle_checkpoint_schema": (
                session_manifest.get("oracle_checkpoints", {}).get("schema", 0)
            ),
            "oracle_checkpoint_interval": (
                session_manifest.get("oracle_checkpoints", {}).get("interval")
            ),
            "oracle_checkpoint_artifact_sha256": oracle_checkpoint_artifacts,
        },
    }
    key = stable_hash(cache_identity)
    cache = cache_root / key
    manifest_path = cache / "cache-manifest.json"
    if manifest_path.is_file():
        manifest = load_json(manifest_path)
        if manifest.get("cache_identity") != cache_identity:
            raise SystemExit(f"parity evidence: cache key collision at {cache}")
        verify_oracle_cache_entry(cache, manifest)
        return cache, True

    temporary = Path(tempfile.mkdtemp(prefix=f".{key}.", dir=cache_root.mkdir(parents=True, exist_ok=True) or cache_root))
    try:
        for name in (*REPLAY_SOURCE_FILES, *ORACLE_SESSION_FILES):
            source = session / name
            if source.is_file():
                shutil.copy2(source, temporary / name)
        if oracle_checkpoints.is_dir():
            shutil.copytree(oracle_checkpoints, temporary / "oracle-checkpoints")
        receipt_count = 0
        if frame_receipts.is_file():
            receipt_count = _extract_oracle_frame_receipts(
                frame_receipts, temporary / "oracle-frame-receipts.jsonl"
            )
        av_hash_count = 0
        if av_hashes.is_file():
            av_hash_count = _extract_oracle_av_hashes(
                av_hashes, temporary / "oracle-av-hashes.jsonl"
            )
        copied = sorted(
            str(path.relative_to(temporary))
            for path in temporary.rglob("*")
            if path.is_file()
        )
        if not any(name.startswith("oracle_") or name.startswith("oracle-") or name == "snes9x-trace.jsonl" for name in copied):
            raise SystemExit(f"parity evidence: {session} contains no reusable oracle artifacts")
        manifest = {
            "schema": CACHE_SCHEMA,
            "kind": "zelda3-content-addressed-oracle-evidence",
            "cache_key": key,
            "cache_identity": cache_identity,
            "source_session": str(session),
            "oracle_frame_receipts": receipt_count,
            "oracle_av_hash_frames": av_hash_count,
            "artifact_sha256": artifact_hashes(temporary, copied),
        }
        atomic_write_json(temporary / "cache-manifest.json", manifest)
        try:
            os.replace(temporary, cache)
        except FileExistsError:
            shutil.rmtree(temporary)
        manifest = load_json(manifest_path)
        verify_oracle_cache_entry(cache, manifest)
        return cache, False
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)
