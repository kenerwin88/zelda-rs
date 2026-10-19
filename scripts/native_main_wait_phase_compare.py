#!/usr/bin/env python3
"""Compare retained native wait phases with pinned Snes9x NMI acceptances.

Capture ZELDA3_DEBUG_DIALOGUE_CPU_PHASE over the same host windows in both
builds. Source frame N belongs to engine host N+1; retro_run is not a clock.
Unowned captures remain visible; they do not establish clock agreement.
Without a baseline, every sampled phase must match. With --baseline-native-log,
the check protects every previously exact host, including missing observations.
This is a CPU phase diagnostic, not an A/V parity receipt.

    python3 scripts/native_main_wait_phase_compare.py CANDIDATE.log SOURCE.bin \
        --baseline-native-log BASELINE.log --output target/native-phase-report.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

from snes9x_trace_format import iter_events


CAPTURE = re.compile(r"^native-main-phase-capture frame=(\d+) cpu_host=(\d+) ")
RETAINED = re.compile(
    r"retained=Some\(\((\d+), (\d+), CpuRasterPosition \{ "
    r"scanline: (\d+), master_cycle: (\d+) \}"
)


def read_native_phases(path: Path) -> dict[int, list[int] | None]:
    phases = {}
    with path.open() as stream:
        for number, line in enumerate(stream, 1):
            capture = CAPTURE.match(line)
            if capture is None:
                continue
            captured_host = int(capture[2])
            # Keep the observation even when this caller has no clock owner.
            # A later capture in the same host may establish the owner.
            phases.setdefault(captured_host, None)
            retained = RETAINED.search(line)
            if retained is None:
                if "retained=None" not in line:
                    raise ValueError(f"{path}:{number}: malformed retained CPU phase")
                continue
            host, pc, v, h = map(int, retained.groups())
            if host != int(capture[2]):
                continue  # A future or expired owner is not this host's phase.
            phase = [pc, v, h]
            if phases[host] is not None and phases[host] != phase:
                raise ValueError(f"{path}:{number}: conflicting phases for host {host}")
            phases[host] = phase
    return phases


def source_acceptances(path: Path) -> dict[int, list[list[int]]]:
    phases = {}
    for event in iter_events(path):
        if event["event"] == "nmi":
            host = int(event["frame"]) + 1
            phases.setdefault(host, []).append(
                [int(event["pc"]), int(event["v"]), int(event["cycles"])]
            )
    return phases


def compare(phases: dict, source: dict) -> list[dict]:
    rows = []
    for host, native in sorted(phases.items()):
        acceptances = source.get(host, [])
        expected = acceptances[0] if len(acceptances) == 1 else None
        status = (
            "source_missing" if not acceptances else
            "source_ambiguous" if len(acceptances) != 1 else
            "native_missing" if native is None else
            "match" if native == expected else "mismatch"
        )
        rows.append({"host": host, "native": native, "source": expected,
                     "source_acceptances": len(acceptances), "status": status})
    return rows


def phase_changes(baseline: list[dict], candidate: list[dict]) -> dict:
    before = {row["host"] for row in baseline if row["status"] == "match"}
    after = {row["host"] for row in candidate if row["status"] == "match"}
    return {"gained_exact_hosts": sorted(after - before),
            "lost_exact_hosts": sorted(before - after)}


def identity(path: Path) -> dict:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return {"path": str(path.resolve()), "sha256": digest.hexdigest()}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("native_log", type=Path)
    parser.add_argument("source_trace", type=Path, help="Z3TRACE1 binary or JSON Lines")
    parser.add_argument("--baseline-native-log", type=Path,
                        help="check for lost exact hosts rather than require full agreement")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    try:
        source = source_acceptances(args.source_trace)
        rows = compare(read_native_phases(args.native_log), source)
        report = {"schema": 1, "kind": "native-main-wait-phase-comparison",
                  "native_log": identity(args.native_log),
                  "source_trace": identity(args.source_trace), "rows": rows,
                  "sampled_hosts": len(rows),
                  "retained_hosts": sum(row["native"] is not None for row in rows),
                  "missing_native_hosts": [row["host"] for row in rows
                                           if row["native"] is None],
                  "first_retained_mismatch": next((row for row in rows
                                                   if row["status"] == "mismatch"), None),
                  "first_unowned_source_main_wait": next((row for row in rows
                      if row["status"] == "native_missing"
                      and row["source"][0] in (0x8034, 0x8036)), None),
                  "exact_hosts": sum(row["status"] == "match" for row in rows)}
        failed = not rows or report["exact_hosts"] != len(rows)
        if args.baseline_native_log is not None:
            baseline = compare(read_native_phases(args.baseline_native_log), source)
            if not any(row["status"] == "match" for row in baseline):
                raise ValueError("baseline has no exact CPU phase observations")
            changes = phase_changes(baseline, rows)
            report.update(changes)
            report["baseline_native_log"] = identity(args.baseline_native_log)
            failed = not rows or bool(changes["lost_exact_hosts"])
        report["check_passed"] = not failed
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, indent=2) + "\n")
        print(f"exact CPU phases: {report['exact_hosts']}/{len(rows)}")
        if args.baseline_native_log is not None:
            print(f"gained: {report['gained_exact_hosts']}; lost: {report['lost_exact_hosts']}")
        else:
            first = next((row for row in rows if row["status"] != "match"), None)
            if first is not None:
                print(f"first disagreement: {json.dumps(first)}")
        return int(failed)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(2, f"native phase comparison: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
