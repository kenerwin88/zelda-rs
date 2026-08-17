#!/usr/bin/env python3
"""Join Rust CPU-checkpoint predictions to a pinned Snes9x PC trace safely.

This tool is intentionally strict. It rejects ambiguous checkpoints and state
disagreement instead of guessing a frame offset. Rust records absolute host
frames because it cannot know a checkpointed comparison's window origin. The
manifest supplies that origin, and only then is the record joined to Snes9x's
window-relative `retro_run`. The Rust trace comes from
ZELDA3_CPU_CHECKPOINT_TRACE; the oracle trace must contain the selected `pc`
event from the instrumented, manifest-recorded Snes9x core.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import defaultdict
from collections.abc import Iterator
from pathlib import Path
from typing import Any


SCHEMA = 2
MASTER_CYCLES_PER_SCANLINE = 1364
RUST_COORDINATE = "absolute comparison host frame"
LEGACY_RUST_COORDINATE = "zero-based libretro retro_run"
ORACLE_COORDINATE = "zero-based libretro retro_run"
STATE_FIELDS = (
    "main",
    "sub",
    "subsub",
    "frame_counter",
    "room",
    "lights_out",
    "palette_countdown",
    "palette_direction",
    "link_y",
    "link_x",
    "bg2_v",
    "bg2_h",
)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        manifest = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"failed to read comparison manifest {path}: {error}") from error
    for section in ("core", "rom"):
        value = manifest.get(section, {}).get("sha256")
        if not isinstance(value, str) or len(value) != 64:
            raise SystemExit(f"manifest {path} has no valid {section}.sha256 provenance")
    return manifest


def read_jsonl(path: Path) -> Iterator[dict[str, Any]]:
    try:
        with path.open() as stream:
            for line_number, line in enumerate(stream, 1):
                try:
                    record = json.loads(line)
                except json.JSONDecodeError as error:
                    raise SystemExit(
                        f"invalid JSON in {path} at line {line_number}: {error}"
                    ) from error
                if not isinstance(record, dict):
                    raise SystemExit(f"non-object JSON in {path} at line {line_number}")
                yield record
    except OSError as error:
        raise SystemExit(f"failed to read {path}: {error}") from error


def checked_int(record: dict[str, Any], field: str, source: str) -> int:
    value = record.get(field)
    if not isinstance(value, int):
        raise SystemExit(f"{source} record is missing integer field {field!r}: {record}")
    return value


def comparison_start_frame(manifest: dict[str, Any]) -> int:
    timing = manifest.get("timing")
    if timing is None:
        return 0
    if not isinstance(timing, dict) or not isinstance(timing.get("start_frame", 0), int):
        raise SystemExit("comparison manifest has no valid timing.start_frame coordinate")
    start_frame = int(timing.get("start_frame", 0))
    if start_frame < 0:
        raise SystemExit("comparison manifest timing.start_frame must be nonnegative")
    return start_frame


def canonical_lorom_pc(pc: int) -> int:
    bank = (pc >> 16) & 0xFF
    address = pc & 0xFFFF
    if address >= 0x8000:
        bank |= 0x80
    return (bank << 16) | address


def load_rust(
    path: Path,
    start_frame: int,
    first_host_frame: int | None,
    last_host_frame: int | None,
) -> dict[int, dict[str, Any]]:
    by_run: dict[int, dict[str, Any]] = {}
    for record in read_jsonl(path):
        if record.get("event") != "rust-cpu-checkpoint":
            continue
        schema = record.get("schema")
        coordinate = record.get("coordinate")
        if schema == SCHEMA and coordinate == RUST_COORDINATE:
            host_frame = checked_int(record, "host_frame", "Rust")
        elif schema == 1 and coordinate == LEGACY_RUST_COORDINATE:
            legacy_run = checked_int(record, "run", "Rust")
            if start_frame != 0:
                raise SystemExit(
                    "legacy Rust checkpoint falsely claims a window-relative run during "
                    f"checkpoint resume (manifest start_frame={start_frame}, run={legacy_run}); "
                    "rebuild the parity binary so it emits schema-2 absolute host frames"
                )
            host_frame = legacy_run
        else:
            raise SystemExit(f"unsupported Rust checkpoint schema/coordinate: {record}")
        if host_frame < start_frame:
            raise SystemExit(
                f"Rust checkpoint host frame {host_frame} precedes manifest start frame {start_frame}"
            )
        if first_host_frame is not None and host_frame < first_host_frame:
            continue
        if last_host_frame is not None and host_frame > last_host_frame:
            continue
        run = host_frame - start_frame
        if run in by_run:
            raise SystemExit(
                f"ambiguous Rust checkpoint: host frame {host_frame} / oracle run {run} "
                "occurs more than once"
            )
        by_run[run] = {**record, "host_frame": host_frame, "oracle_run": run}
    if not by_run:
        raise SystemExit(f"no Rust CPU checkpoints selected from {path}")
    return by_run


def load_oracle(
    path: Path, selected_runs: set[int], checkpoint_pc: int
) -> dict[int, dict[str, Any]]:
    matches: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for record in read_jsonl(path):
        pc = record.get("pc")
        if (
            record.get("event") != "pc"
            or not isinstance(pc, int)
            or canonical_lorom_pc(pc) != canonical_lorom_pc(checkpoint_pc)
        ):
            continue
        run = checked_int(record, "run", "oracle")
        if run in selected_runs:
            matches[run].append(record)
    missing = sorted(selected_runs - matches.keys())
    if missing:
        raise SystemExit(
            "oracle trace has no selected checkpoint for Rust run(s): "
            + ", ".join(map(str, missing))
        )
    ambiguous = {run: len(records) for run, records in matches.items() if len(records) != 1}
    if ambiguous:
        detail = ", ".join(f"{run} ({count} records)" for run, count in sorted(ambiguous.items()))
        raise SystemExit(f"ambiguous oracle checkpoints: {detail}")
    return {run: records[0] for run, records in matches.items()}


def compare(
    oracle_path: Path,
    rust_path: Path,
    manifest_path: Path,
    checkpoint_pc: int,
    first_host_frame: int | None,
    last_host_frame: int | None,
) -> dict[str, Any]:
    manifest = load_manifest(manifest_path)
    start_frame = comparison_start_frame(manifest)
    rust = load_rust(
        rust_path, start_frame, first_host_frame, last_host_frame
    )
    oracle = load_oracle(oracle_path, set(rust), checkpoint_pc)
    comparisons = []
    for run in sorted(rust):
        predicted = rust[run]
        actual = oracle[run]
        predicted_pc = checked_int(predicted, "pc", "Rust")
        if canonical_lorom_pc(predicted_pc) != canonical_lorom_pc(checkpoint_pc):
            raise SystemExit(
                f"run {run} Rust checkpoint PC 0x{predicted_pc:06x} does not match "
                f"selected oracle PC 0x{checkpoint_pc:06x}"
            )
        mismatches = {
            field: {"rust": predicted.get(field), "oracle": actual.get(field)}
            for field in STATE_FIELDS
            if predicted.get(field) != actual.get(field)
        }
        if mismatches:
            raise SystemExit(
                f"run {run} state provenance mismatch at checkpoint 0x{checkpoint_pc:06x}: "
                + json.dumps(mismatches, sort_keys=True)
            )
        predicted_clock = (
            checked_int(predicted, "v", "Rust") * MASTER_CYCLES_PER_SCANLINE
            + checked_int(predicted, "cycles", "Rust")
        )
        actual_clock = (
            checked_int(actual, "v", "oracle") * MASTER_CYCLES_PER_SCANLINE
            + checked_int(actual, "cycles", "oracle")
        )
        comparisons.append(
            {
                "run": run,
                "host_frame": predicted["host_frame"],
                "state": {field: predicted[field] for field in STATE_FIELDS},
                "rust": {"v": predicted["v"], "cycles": predicted["cycles"]},
                "oracle": {"v": actual["v"], "cycles": actual["cycles"]},
                "oracle_minus_rust_master_cycles": actual_clock - predicted_clock,
            }
        )
    artifact_hashes = {}
    for name in ("input.txt", "initial.srm"):
        artifact = manifest_path.with_name(name)
        if artifact.is_file():
            artifact_hashes[name] = sha256(artifact)
    return {
        "schema": SCHEMA,
        "coordinate": {
            "oracle_run": ORACLE_COORDINATE,
            "rust": RUST_COORDINATE,
            "comparison_start_frame": start_frame,
            "mapping": "oracle_run = host_frame - comparison_start_frame",
        },
        "checkpoint_pc": f"0x{checkpoint_pc:06x}",
        "provenance": {
            "oracle_trace": str(oracle_path.resolve()),
            "oracle_trace_sha256": sha256(oracle_path),
            "rust_trace": str(rust_path.resolve()),
            "rust_trace_sha256": sha256(rust_path),
            "manifest": str(manifest_path.resolve()),
            "manifest_sha256": sha256(manifest_path),
            "core_sha256": manifest["core"]["sha256"],
            "rom_sha256": manifest["rom"]["sha256"],
            "session_artifact_sha256": artifact_hashes,
        },
        "comparisons": comparisons,
    }


def parse_address(value: str) -> int:
    try:
        address = int(value, 0)
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"invalid address {value!r}") from error
    if not 0 <= address <= 0xFF_FFFF:
        raise argparse.ArgumentTypeError(f"address is outside 24-bit range: {value!r}")
    return address


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("oracle_trace", type=Path)
    parser.add_argument("rust_trace", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--checkpoint-pc", type=parse_address, default=0x00_8051)
    parser.add_argument("--first-host-frame", type=int)
    parser.add_argument("--last-host-frame", type=int)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.first_host_frame is not None and args.first_host_frame < 0:
        parser.error("--first-host-frame must be nonnegative")
    if (
        args.first_host_frame is not None
        and args.last_host_frame is not None
        and args.last_host_frame < args.first_host_frame
    ):
        parser.error("--last-host-frame must not precede --first-host-frame")
    oracle_path = args.oracle_trace.resolve()
    rust_path = args.rust_trace.resolve()
    manifest_path = (args.manifest or oracle_path.with_name("manifest.json")).resolve()
    report = compare(
        oracle_path,
        rust_path,
        manifest_path,
        args.checkpoint_pc,
        args.first_host_frame,
        args.last_host_frame,
    )
    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.write_text(rendered)
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
