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
PASS_SCHEMA = 1
CACHE_SCHEMA = 1
FRONTIER_SCHEMA = 1

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


def record_cold_pass(
    *,
    session: Path,
    route_signature: dict[str, Any],
    binary: Path,
    output_root: Path = PASS_ROOT,
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
    receipt: dict[str, Any] = {
        "schema": PASS_SCHEMA,
        "kind": "zelda3-cold-parity-pass",
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
        if receipt.get("schema") == PASS_SCHEMA and receipt.get("kind") == "zelda3-cold-parity-pass":
            receipts.append((path, receipt))
    return receipts


def _proof_group(receipt: dict[str, Any]) -> tuple[str, str]:
    return (
        str(receipt.get("route_signature_sha256", "")),
        str(receipt.get("binary", {}).get("sha256", "")),
    )


def promote_frontier(
    *,
    ledger_path: Path = DEFAULT_LEDGER,
    binary: Path = ROOT / "target" / "parity" / "zelda3",
    pass_root: Path = PASS_ROOT,
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
    compatible = [
        (path, receipt)
        for path, receipt in load_cold_passes(pass_root)
        if receipt.get("binary", {}).get("sha256") == binary_sha
    ]
    grouped: dict[tuple[str, str], list[tuple[Path, dict[str, Any]]]] = {}
    for item in compatible:
        grouped.setdefault(_proof_group(item[1]), []).append(item)
    candidates = [items for items in grouped.values() if len(items) >= 2]
    if not candidates:
        raise SystemExit(
            "parity evidence: this binary has fewer than two independent cold exact A/V passes"
        )
    selected = max(
        candidates,
        key=lambda items: max(int(item[1]["created_unix_ns"]) for item in items),
    )
    selected.sort(key=lambda item: int(item[1]["created_unix_ns"]), reverse=True)
    first, second = selected[:2]
    if first[1].get("session") == second[1].get("session"):
        raise SystemExit("parity evidence: duplicate receipts for one session do not count as two cold runs")
    promoted_frame = min(
        int(first[1]["proof"]["frames"]), int(second[1]["proof"]["frames"])
    )
    ledger = load_json(ledger_path) if ledger_path.is_file() else {
        "schema": FRONTIER_SCHEMA,
        "project": "routes/full_run",
        "policy": {"required_cold_confirmations": 2},
    }
    if ledger.get("schema") != FRONTIER_SCHEMA:
        raise SystemExit(f"parity evidence: unsupported frontier ledger: {ledger_path}")
    ledger["promoted"] = {
        "commit": git["head"],
        "binary_sha256": binary_sha,
        "route_signature": first[1]["route_signature"],
        "route_signature_sha256": first[1]["route_signature_sha256"],
        "last_exact_engine_state_frame": promoted_frame,
        "last_exact_video_frame": promoted_frame,
        "last_exact_audio_frame": promoted_frame,
        "cold_confirmation_receipts": [
            {
                "path": str(path.relative_to(ROOT) if path.is_relative_to(ROOT) else path),
                "sha256": sha256_file(path),
                "session_result_sha256": receipt["session_identity"]["result_sha256"],
                "frames": receipt["proof"]["frames"],
            }
            for path, receipt in (first, second)
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
