#!/usr/bin/env python3
"""Collapse an instrumented Snes9x PC/NMI trace into a semantic CPU ledger.

This is an offline extraction tool. It reads a trace produced from the pinned
ROM/core, verifies the comparison manifest, and emits compact per-retro_run
phase spans. The Rust game runtime does not read either the ROM or this ledger;
the ledger is reference evidence for implementing and testing native cycle
models.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import OrderedDict
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
from snes9x_trace_format import iter_events  # noqa: E402


SCHEMA = 1
MASTER_CYCLES_PER_SCANLINE = 1364
NTSC_SCANLINES_PER_FIELD = 262
DEFAULT_MARKERS = OrderedDict(
    [
        ("main_wait", 0x00_8034),
        ("main_entry", 0x00_8051),
        ("module_7", 0x02_87A2),
        ("supertile_transition", 0x02_8A26),
        ("faded_filter", 0x02_8B92),
        ("palette_filter", 0x00_E9E4),
        ("link_oam", 0x0D_A18E),
        ("prepare_sprites", 0x00_85FC),
    ]
)
STATE_FIELDS = (
    "frame",
    "v",
    "cycles",
    "pc",
    "s",
    "main",
    "sub",
    "subsub",
    "frame_counter",
    "room",
    "lights_out",
    "palette_countdown",
    "palette_direction",
    "mosaic_target",
    "nmi_latch",
    "nmi_disable",
    "nmi_pending",
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


def parse_marker(value: str) -> tuple[str, int]:
    try:
        label, raw_address = value.split("=", 1)
        address = int(raw_address, 0)
    except ValueError as error:
        raise argparse.ArgumentTypeError(
            f"marker must be LABEL=0xADDRESS, got {value!r}"
        ) from error
    if not label or not 0 <= address <= 0xFF_FFFF:
        raise argparse.ArgumentTypeError(f"invalid marker {value!r}")
    return label, address


def state(event: dict[str, Any]) -> dict[str, int]:
    missing = [field for field in STATE_FIELDS if field not in event]
    if missing:
        raise SystemExit(
            f"trace event {event.get('event')!r} run={event.get('run')!r} "
            f"is missing fields: {', '.join(missing)}"
        )
    snapshot = {field: int(event[field]) for field in STATE_FIELDS}
    snapshot["master_clock"] = (
        (snapshot["frame"] * NTSC_SCANLINES_PER_FIELD + snapshot["v"])
        * MASTER_CYCLES_PER_SCANLINE
        + snapshot["cycles"]
    )
    return snapshot


def same_pc_span(segment: dict[str, Any], label: str, event: dict[str, Any]) -> bool:
    return (
        segment.get("kind") == "pc_span"
        and segment.get("label") == label
        and segment["last"]["pc"] == int(event["pc"])
    )


def append_event(
    segments: list[dict[str, Any]], event: dict[str, Any], address_labels: dict[int, str]
) -> None:
    kind = event.get("event")
    if kind == "pc":
        label = address_labels.get(int(event["pc"]))
        if label is None:
            return
        snapshot = state(event)
        if segments and same_pc_span(segments[-1], label, event):
            segments[-1]["count"] += 1
            segments[-1]["last"] = snapshot
        else:
            segments.append(
                {
                    "kind": "pc_span",
                    "label": label,
                    "count": 1,
                    "first": snapshot,
                    "last": snapshot,
                }
            )
        return
    if kind in {"nmi", "nmi-resume", "frame"}:
        segment: dict[str, Any] = {"kind": kind, "state": state(event)}
        if kind == "frame" and "stage" in event:
            segment["stage"] = event["stage"]
        segments.append(segment)


def extract(
    trace_path: Path,
    manifest_path: Path,
    first_run: int,
    last_run: int,
    markers: OrderedDict[str, int],
) -> dict[str, Any]:
    if first_run < 0 or last_run < first_run:
        raise SystemExit("run range must satisfy 0 <= FIRST <= LAST")
    if len(set(markers.values())) != len(markers):
        raise SystemExit("semantic PC marker addresses must be unique")

    manifest = load_manifest(manifest_path)
    address_labels = {address: label for label, address in markers.items()}
    runs: OrderedDict[int, list[dict[str, Any]]] = OrderedDict()
    malformed_line = None
    try:
        for line_number, event in enumerate(iter_events(trace_path), 1):
            run = event.get("run")
            if not isinstance(run, int) or not first_run <= run <= last_run:
                continue
            if event.get("event") not in {"pc", "nmi", "nmi-resume", "frame"}:
                continue
            segments = runs.setdefault(run, [])
            append_event(segments, event, address_labels)
    except ValueError as error:
        malformed_line = str(error)
    if malformed_line is not None:
        raise SystemExit(f"trace {trace_path} contains an invalid record: {malformed_line}")
    if not runs:
        raise SystemExit(
            f"trace {trace_path} contains no CPU events for runs {first_run}..{last_run}"
        )

    return {
        "schema": SCHEMA,
        "provenance": {
            "trace": str(trace_path.resolve()),
            "trace_sha256": sha256(trace_path),
            "manifest": str(manifest_path.resolve()),
            "manifest_sha256": sha256(manifest_path),
            "core_sha256": manifest["core"]["sha256"],
            "rom_sha256": manifest["rom"]["sha256"],
        },
        "coordinate": "zero-based libretro retro_run",
        "run_range": [first_run, last_run],
        "markers": {label: f"0x{address:06x}" for label, address in markers.items()},
        "runs": [
            {"run": run, "segments": segments}
            for run, segments in runs.items()
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="instrumented Snes9x JSONL trace")
    parser.add_argument("--manifest", type=Path, help="comparison manifest.json")
    parser.add_argument("--first-run", type=int, required=True)
    parser.add_argument("--last-run", type=int, required=True)
    parser.add_argument(
        "--marker",
        action="append",
        type=parse_marker,
        default=[],
        metavar="LABEL=0xADDRESS",
        help="add a semantic PC marker (repeatable)",
    )
    parser.add_argument("--output", type=Path, help="write JSON here instead of stdout")
    args = parser.parse_args()

    trace_path = args.trace.resolve()
    manifest_path = (args.manifest or trace_path.with_name("manifest.json")).resolve()
    markers = DEFAULT_MARKERS.copy()
    for label, address in args.marker:
        markers[label] = address

    ledger = extract(
        trace_path,
        manifest_path,
        args.first_run,
        args.last_run,
        markers,
    )
    rendered = json.dumps(ledger, indent=2, sort_keys=False) + "\n"
    if args.output:
        args.output.write_text(rendered)
    else:
        sys.stdout.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
