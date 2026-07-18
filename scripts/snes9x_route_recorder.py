#!/usr/bin/env python3
"""Record, resume, pair, and compare human-played Snes9x oracle routes."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = ROOT / "target/release/zelda3"
DEFAULT_CORE = ROOT / "external/snes9x-libretro/local/snes9x_libretro.dylib"
DEFAULT_ROM = ROOT / "target/parity/audio-oracle-fixture/zelda3.sfc"
DEFAULT_PROJECT_ROOT = ROOT / "routes"
DEFAULT_PROJECT = DEFAULT_PROJECT_ROOT / "default"
DEFAULT_SRAM = ROOT / "saves/sram.dat"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def portable_source_path(path: Path) -> dict[str, str]:
    resolved = path.resolve()
    try:
        return {"path": resolved.relative_to(ROOT).as_posix()}
    except ValueError:
        return {"filename": resolved.name}


def load_manifest(project: Path) -> dict:
    path = project / "manifest.json"
    try:
        manifest = json.loads(path.read_text())
    except FileNotFoundError as error:
        raise SystemExit(f"recorder project does not exist: {path}") from error
    if manifest.get("kind") != "zelda3_snes9x_route_recording_v1":
        raise SystemExit(f"unsupported recorder manifest kind in {path}")
    return manifest


def load_pairings(project: Path) -> dict:
    path = project / "pairings.json"
    if not path.exists():
        return {
            "kind": "zelda3_snes9x_rust_boundary_pairings_v1",
            "oracle_independence": "paths and hashes only; no state conversion",
            "boundaries": {},
        }
    pairings = json.loads(path.read_text())
    if pairings.get("kind") != "zelda3_snes9x_rust_boundary_pairings_v1":
        raise SystemExit(f"unsupported pairing manifest kind in {path}")
    return pairings


def load_labels(project: Path) -> dict:
    path = project / "labels.json"
    if not path.exists():
        return {
            "kind": "zelda3_snes9x_boundary_labels_v1",
            "boundaries": {},
            "archived_boundaries": [],
            "archived_project": False,
        }
    labels = json.loads(path.read_text())
    if labels.get("kind") != "zelda3_snes9x_boundary_labels_v1":
        raise SystemExit(f"unsupported boundary labels kind in {path}")
    labels.setdefault("archived_boundaries", [])
    labels.setdefault("archived_project", False)
    return labels


def name_boundary(project: Path, boundary: int, label: str) -> None:
    manifest = load_manifest(project)
    boundaries = manifest.get("boundaries", [])
    if boundary < 0 or boundary >= len(boundaries):
        raise SystemExit(f"unknown boundary {boundary}; project has {len(boundaries)}")
    label = label.strip()
    if not label:
        raise SystemExit("boundary name cannot be empty")
    labels = load_labels(project)
    duplicate = next(
        (
            key
            for key, value in labels["boundaries"].items()
            if value.casefold() == label.casefold() and key != str(boundary)
        ),
        None,
    )
    if duplicate is not None:
        raise SystemExit(
            f"boundary name {label!r} is already used by boundary {duplicate}"
        )
    labels["boundaries"][str(boundary)] = label
    (project / "labels.json").write_text(json.dumps(labels, indent=2) + "\n")


def set_boundary_archived(project: Path, boundary: int, archived: bool) -> None:
    manifest = load_manifest(project)
    boundaries = manifest.get("boundaries", [])
    if boundary < 0 or boundary >= len(boundaries):
        raise SystemExit(f"unknown boundary {boundary}; project has {len(boundaries)}")
    labels = load_labels(project)
    archived_ids = {int(value) for value in labels.get("archived_boundaries", [])}
    if archived:
        archived_ids.add(boundary)
    else:
        archived_ids.discard(boundary)
    labels["archived_boundaries"] = sorted(archived_ids)
    (project / "labels.json").write_text(json.dumps(labels, indent=2) + "\n")


def set_project_archived(project: Path, archived: bool) -> None:
    load_manifest(project)
    labels = load_labels(project)
    labels["archived_project"] = archived
    (project / "labels.json").write_text(json.dumps(labels, indent=2) + "\n")


def resolve_start_boundary(project: Path, value: str) -> str:
    if value == "latest" or value.isdecimal():
        return value
    labels = load_labels(project)["boundaries"]
    matches = [
        key for key, label in labels.items() if label.casefold() == value.casefold()
    ]
    if not matches:
        raise SystemExit(
            f"unknown boundary name {value!r}; use the list command to see saved boundaries"
        )
    return matches[0]


def prepare_recording_sram(
    project: Path, resolved_start: str, requested_sram: Path, blank_sram: bool
) -> Path | None:
    manifest_path = project / "manifest.json"
    if manifest_path.exists():
        manifest = load_manifest(project)
        boundaries = manifest.get("boundaries", [])
        boundary_id = (
            len(boundaries) - 1 if resolved_start == "latest" else int(resolved_start)
        )
        if boundary_id < 0 or boundary_id >= len(boundaries):
            raise SystemExit(
                f"unknown boundary {boundary_id}; project has {len(boundaries)}"
            )
        # Existing projects are initialized from their selected boundary, not
        # from a mutable external .srm file. The native restore writes it again.
        return project / boundaries[boundary_id]["sram_path"]

    project.mkdir(parents=True, exist_ok=True)
    origin = {
        "kind": "zelda3_snes9x_sram_origin_v1",
        "source": "blank" if blank_sram else "file",
    }
    if blank_sram:
        origin["description"] = "blank SRAM initialized by the pinned libretro core"
        seed = None
    else:
        requested_sram = requested_sram.resolve()
        if not requested_sram.is_file():
            raise SystemExit(f"SRAM file does not exist: {requested_sram}")
        origin.update(
            {
                "sha256": sha256(requested_sram),
                "bytes": requested_sram.stat().st_size,
                **portable_source_path(requested_sram),
            }
        )
        seed = requested_sram
    (project / "sram-origin.json").write_text(json.dumps(origin, indent=2) + "\n")
    return seed


def set_take_discarded(project: Path, take_id: int, discarded: bool) -> None:
    manifest = load_manifest(project)
    take = next(
        (take for take in manifest.get("takes", []) if int(take["id"]) == take_id),
        None,
    )
    if take is None:
        raise SystemExit(f"unknown take {take_id}")
    if not discarded and take.get("status") == "merged":
        raise SystemExit(
            f"take {take_id} is merged provenance and cannot be restored independently"
        )
    take["status"] = "discarded" if discarded else "complete"
    (project / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    _write_take_status(project, take)


def _write_take_status(project: Path, take: dict) -> None:
    take_id = int(take["id"])
    status_path = project / f"takes/{take_id:04}/status.json"
    if status_path.parent.exists():
        status_path.write_text(
            json.dumps(
                {
                    "status": take["status"],
                    "id": take_id,
                    "start_boundary": take["start_boundary"],
                    "end_boundary": take.get("end_boundary"),
                    "frames": take.get("frames", 0),
                },
                indent=2,
            )
            + "\n"
        )


def take_is_active(take: dict) -> bool:
    return take.get("status", "complete") in {
        "complete",
        "recovered_after_interruption",
    }


def merge_takes_across_boundary(project: Path, boundary_id: int) -> dict:
    """Replace two adjacent active takes with one non-destructive merged take."""
    manifest = load_manifest(project)
    boundary = next(
        (
            boundary
            for boundary in manifest.get("boundaries", [])
            if int(boundary["id"]) == boundary_id
        ),
        None,
    )
    if boundary is None:
        raise SystemExit(f"unknown boundary {boundary_id}")
    takes = manifest.get("takes", [])
    incoming = [
        take
        for take in takes
        if take_is_active(take)
        and take.get("end_boundary") is not None
        and int(take["end_boundary"]) == boundary_id
    ]
    outgoing = [
        take
        for take in takes
        if take_is_active(take) and int(take["start_boundary"]) == boundary_id
    ]
    if len(incoming) != 1 or len(outgoing) != 1 or incoming[0] is outgoing[0]:
        raise SystemExit(
            f"save #{boundary_id} must have exactly one active incoming and outgoing "
            f"take; found incoming={len(incoming)} outgoing={len(outgoing)}"
        )

    before, after = incoming[0], outgoing[0]
    source_ids = [int(before["id"]), int(after["id"])]
    new_id = max((int(take["id"]) for take in takes), default=-1) + 1
    input_path = Path(f"takes/{new_id:04}/input.txt")
    receipts_path = Path(f"takes/{new_id:04}/frame_receipts.jsonl")
    takes_by_id = {int(take["id"]): take for take in takes}
    frames = write_continuous_input(
        project, source_ids, project / input_path, takes_by_id=takes_by_id
    )
    merged = {
        "id": new_id,
        "start_boundary": int(before["start_boundary"]),
        "end_boundary": (
            None if after.get("end_boundary") is None else int(after["end_boundary"])
        ),
        "frames": frames,
        "input_path": str(input_path),
        "status": "complete",
        "merged_from_takes": source_ids,
        "merged_across_boundary": boundary_id,
    }
    if write_continuous_receipts(
        project, source_ids, project / receipts_path, takes_by_id=takes_by_id
    ):
        merged["receipts_path"] = str(receipts_path)
    before["status"] = "merged"
    after["status"] = "merged"
    takes.append(merged)
    (project / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for take in (before, after, merged):
        _write_take_status(project, take)

    set_boundary_archived(project, boundary_id, True)
    return merged


def pair_boundary(project: Path, boundary: int, rust_state: Path) -> None:
    manifest = load_manifest(project)
    boundaries = manifest.get("boundaries", [])
    if boundary < 0 or boundary >= len(boundaries):
        raise SystemExit(f"unknown boundary {boundary}; project has {len(boundaries)}")
    rust_state = rust_state.resolve()
    if not rust_state.is_file():
        raise SystemExit(f"Rust state does not exist: {rust_state}")
    pairings = load_pairings(project)
    pairings["boundaries"][str(boundary)] = {
        "rust_state": str(rust_state),
        "rust_state_sha256": sha256(rust_state),
        "oracle_state": str((project / boundaries[boundary]["state_path"]).resolve()),
        "oracle_state_sha256": boundaries[boundary].get("state_sha256"),
        "converted_to_snes9x": False,
        "converted_from_snes9x": False,
    }
    (project / "pairings.json").write_text(json.dumps(pairings, indent=2) + "\n")


def resolve_project_path(project: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else project / path


def promote_passed_take(project: Path, take_id: int, session_dir: Path) -> dict:
    """Persist an independently verified Rust endpoint beside its Snes9x boundary."""
    manifest = load_manifest(project)
    takes = manifest.get("takes", [])
    if take_id < 0 or take_id >= len(takes):
        raise SystemExit(f"unknown take {take_id}; project has {len(takes)}")
    take = takes[take_id]
    end_boundary = take.get("end_boundary")
    if end_boundary is None:
        raise SystemExit(f"take {take_id} has no end boundary to promote")
    end_boundary = int(end_boundary)
    boundaries = manifest.get("boundaries", [])
    if end_boundary < 0 or end_boundary >= len(boundaries):
        raise SystemExit(f"take {take_id} ends at unknown boundary {end_boundary}")

    result_path = session_dir / "result.json"
    session_manifest_path = session_dir / "manifest.json"
    rust_final_path = session_dir / "rust_final.z3state"
    try:
        result = json.loads(result_path.read_text())
        session_manifest = json.loads(session_manifest_path.read_text())
    except FileNotFoundError as error:
        raise SystemExit(f"comparison artifact is missing: {error.filename}") from error
    exact_pass = (
        result.get("status") == "passed"
        and result.get("parity_eligible") is True
        and result.get("video", {}).get("matched") is True
        and result.get("audio", {}).get("matched") is True
        and result.get("audio", {}).get("mode") == "exact"
        and int(result.get("frames_completed", -1)) == int(take["frames"])
    )
    if not exact_pass:
        raise SystemExit(
            f"take {take_id} result is not an exact A/V parity pass; boundary was not promoted"
        )
    if not rust_final_path.is_file():
        raise SystemExit(f"comparison did not produce Rust endpoint: {rust_final_path}")

    identity = manifest["identity"]
    core_sha256 = session_manifest.get("core", {}).get("sha256")
    rom_sha256 = session_manifest.get("rom", {}).get("sha256")
    if core_sha256 != identity["core_sha256"] or rom_sha256 != identity["rom_sha256"]:
        raise SystemExit("comparison core/ROM identity does not match the recorded route")

    boundary = boundaries[end_boundary]
    oracle_state = project / boundary["state_path"]
    oracle_sram = project / boundary["sram_path"]
    if not oracle_state.is_file() or not oracle_sram.is_file():
        raise SystemExit(f"Snes9x-native boundary {end_boundary} is incomplete")

    boundary_dir = project / f"boundaries/{end_boundary:04}"
    promoted_state = boundary_dir / "rust.z3state"
    shutil.copyfile(rust_final_path, promoted_state)
    promoted_relative = promoted_state.relative_to(project).as_posix()
    receipt_path = boundary_dir / "parity.json"
    receipt_relative = receipt_path.relative_to(project).as_posix()
    receipt = {
        "kind": "zelda3_snes9x_boundary_exact_parity_v1",
        "status": "exact_av_verified",
        "oracle": "Snes9x 1.63 libretro only",
        "production_renderer": "modern Rust",
        "production_audio_backend": "modern",
        "production_audio_sequencer": "native",
        "audio_comparison": "exact",
        "take": take_id,
        "start_boundary": int(take["start_boundary"]),
        "end_boundary": end_boundary,
        "frames_verified": int(take["frames"]),
        "input_sha256": sha256(project / take["input_path"]),
        "core_sha256": core_sha256,
        "rom_sha256": rom_sha256,
        "rust_state": promoted_relative,
        "rust_state_sha256": sha256(promoted_state),
        "oracle_state": boundary["state_path"],
        "oracle_state_sha256": sha256(oracle_state),
        "oracle_sram": boundary["sram_path"],
        "oracle_sram_sha256": sha256(oracle_sram),
        "comparison_result_sha256": sha256(result_path),
        "oracle_independence": {
            "state_conversion_used": False,
            "snes9x_state_created_by": "Snes9x retro_serialize",
            "rust_state_created_by": "zelda3-rs replay",
        },
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")
    pairings = load_pairings(project)
    pairings["boundaries"][str(end_boundary)] = {
        "rust_state": promoted_relative,
        "rust_state_sha256": receipt["rust_state_sha256"],
        "oracle_state": boundary["state_path"],
        "oracle_state_sha256": receipt["oracle_state_sha256"],
        "verified_by": receipt_relative,
        "converted_to_snes9x": False,
        "converted_from_snes9x": False,
    }
    (project / "pairings.json").write_text(json.dumps(pairings, indent=2) + "\n")
    return receipt


def compare_command(
    *,
    binary: Path,
    core: Path,
    rom: Path,
    project: Path,
    take_id: int,
    session_dir: Path,
) -> list[str]:
    manifest = load_manifest(project)
    takes = manifest.get("takes", [])
    if take_id < 0 or take_id >= len(takes):
        raise SystemExit(f"unknown take {take_id}; project has {len(takes)}")
    take = takes[take_id]
    boundary_id = int(take["start_boundary"])
    return compare_input_command(
        binary=binary,
        core=core,
        rom=rom,
        project=project,
        boundary_id=boundary_id,
        frames=int(take["frames"]),
        input_path=project / take["input_path"],
        session_dir=session_dir,
    )


def compare_input_command(
    *,
    binary: Path,
    core: Path,
    rom: Path,
    project: Path,
    boundary_id: int,
    frames: int,
    input_path: Path,
    session_dir: Path,
) -> list[str]:
    manifest = load_manifest(project)
    boundaries = manifest.get("boundaries", [])
    boundary = boundaries[boundary_id]
    command = [
        str(binary),
        "--compare-snes9x-oracle",
        str(core),
        str(rom),
        str(frames),
        "--expected-core-sha256",
        manifest["identity"]["core_sha256"],
        "--expected-rom-sha256",
        manifest["identity"]["rom_sha256"],
        "--input-script",
        str(input_path),
    ]
    if not boundary.get("reset_start", False):
        pairings = load_pairings(project)
        pairing = pairings["boundaries"].get(str(boundary_id))
        if pairing is None:
            raise SystemExit(
                f"boundary {boundary_id} has no Rust pairing; run the pair command first"
            )
        rust_state = resolve_project_path(project, pairing["rust_state"])
        if sha256(rust_state) != pairing["rust_state_sha256"]:
            raise SystemExit(f"Rust state hash changed: {rust_state}")
        command.extend(
            [
                "--resume-rust-state",
                str(rust_state),
                "--resume-oracle-state",
                str(project / boundary["state_path"]),
                "--resume-oracle-sram",
                str(project / boundary["sram_path"]),
            ]
        )
    command.extend(
        [
            "--audio-comparison",
            "exact",
            "--rust-audio-backend",
            "modern",
            "--rust-audio-sequencer",
            "native",
            "--session-dir",
            str(session_dir),
            "--scan-all",
        ]
    )
    return command


def comparable_take_ids(project: Path) -> list[int]:
    manifest = load_manifest(project)
    pairings = load_pairings(project)["boundaries"]
    boundaries = manifest.get("boundaries", [])
    result = []
    for take in manifest.get("takes", []):
        if not take_is_active(take):
            continue
        boundary = boundaries[int(take["start_boundary"])]
        if int(take.get("frames", 0)) <= 0:
            continue
        if boundary.get("reset_start", False) or str(boundary["id"]) in pairings:
            result.append(int(take["id"]))
    return result


def excluded_nonempty_takes(project: Path) -> list[dict]:
    manifest = load_manifest(project)
    pairings = load_pairings(project)["boundaries"]
    boundaries = manifest.get("boundaries", [])
    result = []
    for take in manifest.get("takes", []):
        if not take_is_active(take):
            continue
        if int(take.get("frames", 0)) <= 0:
            continue
        boundary_id = int(take["start_boundary"])
        boundary = boundaries[boundary_id]
        if not boundary.get("reset_start", False) and str(boundary_id) not in pairings:
            result.append(
                {
                    "take": int(take["id"]),
                    "start_boundary": boundary_id,
                    "frames": int(take["frames"]),
                    "reason": "start boundary has no paired Rust-native state",
                }
            )
    return result


def continuous_take_ids(project: Path) -> list[int]:
    manifest = load_manifest(project)
    boundaries = manifest.get("boundaries", [])
    reset_boundaries = [
        int(boundary["id"])
        for boundary in boundaries
        if boundary.get("reset_start", False)
    ]
    if len(reset_boundaries) != 1:
        raise SystemExit(
            f"continuous comparison requires one reset boundary; found {reset_boundaries}"
        )
    active = [
        take
        for take in manifest.get("takes", [])
        if int(take.get("frames", 0)) > 0 and take_is_active(take)
    ]
    chain: list[int] = []
    current_boundary = reset_boundaries[0]
    remaining = {int(take["id"]): take for take in active}
    while remaining:
        candidates = [
            take
            for take in remaining.values()
            if int(take["start_boundary"]) == current_boundary
        ]
        if not candidates:
            break
        if len(candidates) != 1:
            ids = sorted(int(take["id"]) for take in candidates)
            raise SystemExit(
                f"continuous route branches at boundary {current_boundary}: takes {ids}"
            )
        take = candidates[0]
        take_id = int(take["id"])
        chain.append(take_id)
        del remaining[take_id]
        end_boundary = take.get("end_boundary")
        if end_boundary is None:
            break
        current_boundary = int(end_boundary)
    if remaining:
        raise SystemExit(
            "recorded takes are not one continuous reset-started route; "
            f"disconnected takes={sorted(remaining)}"
        )
    return chain


def _offset_input_selector(selector: str, offset: int) -> str:
    if ".." in selector:
        start, end = selector.split("..", 1)
        return f"{int(start) + offset}..{int(end) + offset}"
    return str(int(selector) + offset)


def write_continuous_input(
    project: Path,
    take_ids: list[int],
    output: Path,
    *,
    takes_by_id: dict[int, dict] | None = None,
) -> int:
    if takes_by_id is None:
        manifest = load_manifest(project)
        takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    output.parent.mkdir(parents=True, exist_ok=True)
    offset = 0
    with output.open("w") as destination:
        source_list = ", ".join(str(take_id) for take_id in take_ids)
        destination.write(
            f"# Continuous Snes9x route assembled from takes {source_list}.\n"
        )
        for take_id in take_ids:
            take = takes_by_id[take_id]
            path = project / take["input_path"]
            with path.open() as source:
                for line_no, raw in enumerate(source, start=1):
                    stripped = raw.strip()
                    if not stripped or stripped.startswith("#"):
                        continue
                    parts = stripped.split()
                    if len(parts) != 2:
                        raise SystemExit(
                            f"unsupported recorder input at {path}:{line_no}: {stripped}"
                        )
                    selector = _offset_input_selector(parts[0], offset)
                    destination.write(f"{selector} {parts[1]}\n")
            offset += int(take["frames"])
    return offset


def write_continuous_receipts(
    project: Path,
    take_ids: list[int],
    output: Path,
    *,
    takes_by_id: dict[int, dict],
) -> bool:
    sources = []
    for take_id in take_ids:
        sources.extend(_receipt_source_takes(takes_by_id[take_id], takes_by_id))

    receipt_files = []
    for take in sources:
        relative_path = take.get("receipts_path")
        if not relative_path:
            return False
        path = project / relative_path
        if not path.is_file():
            return False
        receipt_files.append((take, path))

    output.parent.mkdir(parents=True, exist_ok=True)
    offset = 0
    with output.open("w") as destination:
        for take, path in receipt_files:
            with path.open() as source:
                for line_no, raw in enumerate(source, start=1):
                    if not raw.strip():
                        continue
                    try:
                        receipt = json.loads(raw)
                        receipt["frame"] = int(receipt["frame"]) + offset
                    except (
                        json.JSONDecodeError,
                        KeyError,
                        TypeError,
                        ValueError,
                    ) as error:
                        raise SystemExit(
                            f"unsupported recorder receipt at {path}:{line_no}: {error}"
                        ) from error
                    destination.write(json.dumps(receipt, separators=(",", ":")) + "\n")
            offset += int(take["frames"])
    return True


def _receipt_source_takes(
    take: dict, takes_by_id: dict[int, dict], ancestors: frozenset[int] = frozenset()
) -> list[dict]:
    take_id = int(take["id"])
    if take_id in ancestors:
        raise SystemExit(f"merged take provenance contains a cycle at take {take_id}")
    source_ids = take.get("merged_from_takes")
    if not source_ids:
        return [take]
    next_ancestors = ancestors | {take_id}
    return [
        source
        for source_id in source_ids
        for source in _receipt_source_takes(
            takes_by_id[int(source_id)], takes_by_id, next_ancestors
        )
    ]


def compare_all(
    *, binary: Path, core: Path, rom: Path, project: Path
) -> tuple[dict, int]:
    manifest = load_manifest(project)
    take_ids = comparable_take_ids(project)
    excluded_takes = excluded_nonempty_takes(project)
    nonempty_take_ids = [
        int(take["id"])
        for take in manifest.get("takes", [])
        if int(take.get("frames", 0)) > 0 and take_is_active(take)
    ]
    summaries = []
    requested_frames = 0
    passed_frames = 0
    # A matrix with skipped recorded gameplay is incomplete, even if every
    # currently comparable take passes.
    exit_code = 1 if excluded_takes else 0
    takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    for take_id in take_ids:
        take = takes_by_id[take_id]
        requested_frames += int(take["frames"])
        session_dir = project / "comparisons" / f"take-{take_id:04}"
        command = compare_command(
            binary=binary,
            core=core,
            rom=rom,
            project=project,
            take_id=take_id,
            session_dir=session_dir,
        )
        completed = subprocess.run(command, cwd=ROOT)
        result_path = session_dir / "result.json"
        result = json.loads(result_path.read_text()) if result_path.exists() else {}
        passed = (
            completed.returncode == 0
            and result.get("status") == "passed"
            and result.get("parity_eligible") is True
            and result.get("video", {}).get("matched") is True
            and result.get("audio", {}).get("matched") is True
            and result.get("audio", {}).get("mode") == "exact"
            and int(result.get("frames_completed", 0)) == int(take["frames"])
        )
        if passed:
            receipt = promote_passed_take(project, take_id, session_dir)
            passed_frames += receipt["frames_verified"]
        else:
            exit_code = 1
        summaries.append(
            {
                "take": take_id,
                "start_boundary": take["start_boundary"],
                "frames_requested": take["frames"],
                "frames_completed": result.get("frames_completed", 0),
                "passed": passed,
                "result": str(result_path.relative_to(project)),
            }
        )
    matrix = {
        "kind": "zelda3_snes9x_human_route_matrix_v1",
        "coverage_label": "segmented human-recorded coverage",
        "continuous_playthrough": False,
        "oracle": "Snes9x 1.63 libretro only",
        "production_renderer": "modern Rust",
        "production_audio_backend": "modern",
        "production_audio_sequencer": "native",
        "nonempty_recorded_takes": nonempty_take_ids,
        "comparable_takes": take_ids,
        "excluded_takes": excluded_takes,
        "frames_requested": requested_frames,
        "frames_verified_video_and_audio": passed_frames,
        "all_recorded_takes_comparable": not excluded_takes,
        "all_comparable_takes_passed": bool(take_ids)
        and all(summary["passed"] for summary in summaries),
        "all_recorded_takes_passed": bool(nonempty_take_ids)
        and not excluded_takes
        and all(summary["passed"] for summary in summaries),
        "takes": summaries,
    }
    output = project / "matrix-result.json"
    output.write_text(json.dumps(matrix, indent=2) + "\n")
    return matrix, exit_code


def compare_continuous(
    *, binary: Path, core: Path, rom: Path, project: Path
) -> tuple[dict, int]:
    take_ids = continuous_take_ids(project)
    if not take_ids:
        raise SystemExit("continuous route has no nonempty takes")
    session_dir = project / "comparisons/continuous"
    input_path = session_dir / "continuous-input.txt"
    frames = write_continuous_input(project, take_ids, input_path)
    manifest = load_manifest(project)
    takes = {int(take["id"]): take for take in manifest.get("takes", [])}
    start_boundary = int(takes[take_ids[0]]["start_boundary"])
    command = compare_input_command(
        binary=binary,
        core=core,
        rom=rom,
        project=project,
        boundary_id=start_boundary,
        frames=frames,
        input_path=input_path,
        session_dir=session_dir,
    )
    completed = subprocess.run(command, cwd=ROOT)
    result_path = session_dir / "result.json"
    result = json.loads(result_path.read_text()) if result_path.exists() else {}
    passed = (
        completed.returncode == 0
        and result.get("status") == "passed"
        and result.get("video", {}).get("matched") is True
        and result.get("audio", {}).get("matched") is True
        and int(result.get("frames_completed", 0)) == frames
    )
    summary = {
        "kind": "zelda3_snes9x_continuous_route_result_v1",
        "coverage_label": "continuous human-recorded coverage",
        "continuous_playthrough": True,
        "oracle": "Snes9x 1.63 libretro only",
        "production_renderer": "modern Rust",
        "production_audio_backend": "modern",
        "production_audio_sequencer": "native",
        "takes": take_ids,
        "frames_requested": frames,
        "frames_completed": int(result.get("frames_completed", 0)),
        "video_matched": result.get("video", {}).get("matched") is True,
        "audio_matched": result.get("audio", {}).get("matched") is True,
        "passed": passed,
        "result": str(result_path.relative_to(project)),
    }
    (project / "continuous-result.json").write_text(
        json.dumps(summary, indent=2) + "\n"
    )
    return summary, 0 if passed else 1


def build(binary: Path) -> None:
    subprocess.run(
        ["cargo", "build", "--release", "-p", "zelda3-bin"],
        cwd=ROOT,
        check=True,
    )
    if not binary.is_file():
        raise SystemExit(f"recorder binary was not produced: {binary}")


def print_project(project: Path) -> None:
    manifest = load_manifest(project)
    pairings = load_pairings(project)["boundaries"]
    label_data = load_labels(project)
    labels = label_data["boundaries"]
    archived = {int(value) for value in label_data["archived_boundaries"]}
    print(f"project: {project}")
    print(
        f"oracle: {manifest['identity'].get('core_name')} "
        f"{manifest['identity'].get('core_version')}"
    )
    print("boundaries:")
    for boundary in manifest.get("boundaries", []):
        telemetry = boundary.get("telemetry", {})
        if boundary.get("reset_start", False):
            parity = "reset-ready"
        elif str(boundary["id"]) in pairings:
            parity = "rust-paired"
        else:
            parity = "rust-needed"
        status = "archived" if int(boundary["id"]) in archived else "active"
        main = telemetry.get("main")
        submodule = telemetry.get("sub")
        module = (
            f"{int(main):02x}/{int(submodule):02x}"
            if main is not None and submodule is not None
            else "??/??"
        )
        print(
            f"  {boundary['id']:4} parity={parity:11} "
            f"status={status:8} "
            f"name={labels.get(str(boundary['id']), '-')!r} "
            f"module={module} "
            f"room={telemetry.get('room', '?')} hp={telemetry.get('health', '?')} "
            f"{boundary.get('screenshot_path', '')}"
        )
    print("takes:")
    for take in manifest.get("takes", []):
        print(
            f"  {take['id']:4} start={take['start_boundary']} "
            f"end={take.get('end_boundary')} frames={take['frames']} "
            f"status={take.get('status', '?')}"
        )


def parser() -> argparse.ArgumentParser:
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--project", type=Path, default=DEFAULT_PROJECT)
    common.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    common.add_argument("--core", type=Path, default=DEFAULT_CORE)
    common.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    common.add_argument("--no-build", action="store_true")

    result = argparse.ArgumentParser(description=__doc__)
    sub = result.add_subparsers(dest="action", required=True)
    record = sub.add_parser("record", parents=[common])
    record.add_argument("--start", default="latest", help="boundary number or latest")
    record.add_argument("--sram", type=Path, default=DEFAULT_SRAM)
    record.add_argument("--blank-sram", action="store_true")
    record.add_argument("--max-frames", type=int)
    sub.add_parser("list", parents=[common])
    pair = sub.add_parser("pair", parents=[common])
    pair.add_argument("boundary", type=int)
    pair.add_argument("rust_state", type=Path)
    name = sub.add_parser("name", parents=[common])
    name.add_argument("boundary", type=int)
    name.add_argument("label")
    discard = sub.add_parser("discard-take", parents=[common])
    discard.add_argument("take", type=int)
    restore = sub.add_parser("restore-take", parents=[common])
    restore.add_argument("take", type=int)
    tui = sub.add_parser("tui", parents=[common])
    tui.add_argument(
        "--project-root",
        type=Path,
        default=DEFAULT_PROJECT_ROOT,
        help="directory containing recorder projects",
    )
    compare = sub.add_parser("compare", parents=[common])
    compare.add_argument("take", type=int)
    compare.add_argument("--session-dir", type=Path)
    sub.add_parser("compare-all", parents=[common])
    sub.add_parser("compare-route", parents=[common])
    return result


def parse_cli_args(argv: list[str] | None = None):
    values = sys.argv[1:] if argv is None else argv
    return parser().parse_args(values or ["tui"])


def main() -> None:
    args = parse_cli_args()
    if args.action == "list":
        print_project(args.project)
        return
    if args.action == "pair":
        pair_boundary(args.project, args.boundary, args.rust_state)
        print(f"paired boundary {args.boundary} with {args.rust_state.resolve()}")
        return
    if args.action == "name":
        name_boundary(args.project, args.boundary, args.label)
        print(f"named boundary {args.boundary}: {args.label.strip()}")
        return
    if args.action in {"discard-take", "restore-take"}:
        discarded = args.action == "discard-take"
        set_take_discarded(args.project, args.take, discarded)
        action = "discarded" if discarded else "restored"
        print(f"{action} take {args.take}; recorded files were preserved")
        return
    if args.action == "tui":
        import snes9x_route_tui

        snes9x_route_tui.run(args)
        return
    if not args.no_build:
        build(args.binary)
    if args.action == "record":
        resolved_start = resolve_start_boundary(args.project, args.start)
        seed_sram = prepare_recording_sram(
            args.project, resolved_start, args.sram, args.blank_sram
        )
        command = [
            str(args.binary),
            "--record-snes9x-route",
            str(args.core),
            str(args.rom),
            str(args.project),
            "--start-boundary",
            resolved_start,
            "--expected-core-sha256",
            sha256(args.core),
            "--expected-rom-sha256",
            sha256(args.rom),
        ]
        if seed_sram is not None:
            command.extend(["--load-sram", str(seed_sram)])
        if args.max_frames is not None:
            command.extend(["--max-frames", str(args.max_frames)])
        raise SystemExit(subprocess.run(command, cwd=ROOT).returncode)
    if args.action == "compare":
        session_dir = args.session_dir or (
            args.project / "comparisons" / f"take-{args.take:04}"
        )
        command = compare_command(
            binary=args.binary,
            core=args.core,
            rom=args.rom,
            project=args.project,
            take_id=args.take,
            session_dir=session_dir,
        )
        completed = subprocess.run(command, cwd=ROOT)
        if completed.returncode == 0:
            receipt = promote_passed_take(args.project, args.take, session_dir)
            print(
                f"saved exact A/V parity through boundary {receipt['end_boundary']}: "
                f"{receipt['frames_verified']} frames"
            )
        raise SystemExit(completed.returncode)
    if args.action == "compare-all":
        matrix, exit_code = compare_all(
            binary=args.binary,
            core=args.core,
            rom=args.rom,
            project=args.project,
        )
        print(
            "human route matrix: "
            f"takes={len(matrix['comparable_takes'])} "
            f"verified={matrix['frames_verified_video_and_audio']}/"
            f"{matrix['frames_requested']} frames "
            f"excluded={len(matrix['excluded_takes'])}"
        )
        raise SystemExit(exit_code)
    if args.action == "compare-route":
        summary, exit_code = compare_continuous(
            binary=args.binary,
            core=args.core,
            rom=args.rom,
            project=args.project,
        )
        print(
            "continuous human route: "
            f"takes={summary['takes']} "
            f"completed={summary['frames_completed']}/{summary['frames_requested']} "
            f"video={'pass' if summary['video_matched'] else 'FAIL'} "
            f"audio={'pass' if summary['audio_matched'] else 'FAIL'}"
        )
        raise SystemExit(exit_code)
    raise AssertionError(args.action)


if __name__ == "__main__":
    main()
