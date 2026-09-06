#!/usr/bin/env python3
"""Derive per-take cartridge RNG scripts from a continuous trace-core replay.

The recorder's live traces are keyed by the RECORDING SESSION's retro_run
index, which counts menu, resume, and re-record frames (take 4 shipped with an
entry at 20889 despite being 11,735 frames long). Replaying the assembled
continuous input through the instrumented trace core instead yields rng-write
events whose run index IS the route frame:

    ZELDA3_SNES9X_TRACE=<out.jsonl> ZELDA3_SNES9X_TRACE_EVENTS=rng \
      zelda3 --compare-snes9x-oracle <trace-core.dylib> <rom> <frames> \
        --input-script .../continuous-input.txt --load-sram .../initial.srm \
        --ignore-video --ignore-audio ...

This tool splits that JSONL into per-take `rom-random.txt` files (take-relative
frames) and registers them in the project manifest so both the continuous
assembly and per-take compares consume the corrected coordinates.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from snes9x_route_recorder import continuous_take_ids, load_manifest  # noqa: E402
from snes9x_trace_format import iter_events  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="rng-write JSONL from the trace core")
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument(
        "--dry-run", action="store_true", help="print the split without writing"
    )
    args = parser.parse_args()

    samples = []
    for line_number, event in enumerate(iter_events(args.trace), 1):
        if event.get("event") != "rng-write":
            continue
        run, value, carry = event["run"], event["value"], event["carry"]
        if samples and run < samples[-1][0]:
            raise SystemExit(f"line {line_number}: run {run} out of order")
        samples.append((int(run), int(value), int(carry)))

    manifest = load_manifest(args.project)
    takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    take_ids = continuous_take_ids(args.project)

    start = 0
    consumed = 0
    for take_id in take_ids:
        take = takes_by_id[take_id]
        frames = int(take["frames"])
        window = [s for s in samples if start <= s[0] < start + frames]
        rel_path = f"takes/{take_id:04}/rom-random.txt"
        lines = [
            "# Cartridge $8dba71 RNG outputs, keyed by take-relative route frame.",
            "# Derived from a continuous trace-core replay of the assembled route",
            "# input (scripts/derive_route_rom_random.py), where the trace event's",
            "# retro_run index equals the route frame.",
        ]
        lines += [
            f"{run - start} 0x{value:02x} carry={carry}" for run, value, carry in window
        ]
        print(f"take {take_id}: {len(window)} sample(s) -> {rel_path}")
        if not args.dry_run:
            (args.project / rel_path).write_text("\n".join(lines) + "\n")
            take["rom_random_path"] = rel_path
        consumed += len(window)
        start += frames

    if consumed != len(samples):
        print(
            f"warning: {len(samples) - consumed} sample(s) beyond the route window",
            file=sys.stderr,
        )
    if not args.dry_run:
        (args.project / "manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
