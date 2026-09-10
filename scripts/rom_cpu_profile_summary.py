#!/usr/bin/env python3
"""Summarize ROM-CPU shadow-run profiles written by ZELDA3_DEBUG_ROM_CPU_PROFILE.

Each JSON file is one shadow run (one measured timing plan on one host).
The summary groups runs by plan entry PC and reports how many hosts ran the
plan, the spread of its total master cycles, and the subroutines that
dominate it, named through the ROM symbol table when available.

usage: rom_cpu_profile_summary.py <profile-dir> [--symbols names.txt] [--top N]
"""
import argparse
import bisect
import json
import os
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

DEFAULT_SYMBOLS = Path(
    os.environ.get("ZELDA3_ROM_SYMBOLS", "/Users/missingno/Documents/zelda3/other/names.txt")
)
SYMBOL_RE = re.compile(r"^0x([0-9a-fA-F]{4,6}):\s*(\S+)")


def canonical_pc(pc: int) -> int:
    return ((pc >> 16) & 0x7F) << 16 | (pc & 0xFFFF)


class Symbols:
    def __init__(self, path: Path):
        self.by_address: dict[int, str] = {}
        if path.is_file():
            for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
                match = SYMBOL_RE.match(line)
                if match:
                    self.by_address.setdefault(canonical_pc(int(match.group(1), 16)), match.group(2))
        self.addresses = sorted(self.by_address)

    def describe(self, pc: int) -> str:
        pc = canonical_pc(pc)
        if not self.addresses:
            return f"{pc:06x}"
        index = bisect.bisect_right(self.addresses, pc) - 1
        if index < 0:
            return f"{pc:06x}"
        base = self.addresses[index]
        name = self.by_address[base]
        if base == pc:
            return f"{pc:06x} {name}"
        if (pc >> 16) != (base >> 16) or pc - base > 0x800:
            return f"{pc:06x}"
        return f"{pc:06x} {name}+{pc - base:x}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("profile_dir", type=Path)
    parser.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    parser.add_argument("--top", type=int, default=12)
    args = parser.parse_args()
    symbols = Symbols(args.symbols)

    runs_by_plan: dict[str, list[dict]] = defaultdict(list)
    for path in sorted(args.profile_dir.glob("host-*.json")):
        try:
            run = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        runs_by_plan[run["entry_pc"]].append(run)
    if not runs_by_plan:
        print(f"no profiles under {args.profile_dir}", file=sys.stderr)
        return 1

    for entry_pc, runs in sorted(runs_by_plan.items(), key=lambda item: -len(item[1])):
        totals = [run["total_master"] for run in runs]
        hosts = sorted({run["host"] for run in runs})
        print(f"plan {symbols.describe(int(entry_pc, 16))} -> stop {symbols.describe(int(runs[0]['stop_pc'], 16))}")
        print(
            f"  runs={len(runs)} hosts={len(hosts)} ({hosts[0]}..{hosts[-1]})  "
            f"total_master min={min(totals)} median={int(statistics.median(totals))} max={max(totals)}  "
            f"distinct_totals={len(set(totals))}  "
            f"nmi_entries mean={statistics.mean(run['nmi_entries'] for run in runs):.2f}  "
            f"dma_master mean={statistics.mean(run['dma_master'] for run in runs):.0f}"
        )
        inclusive: dict[str, list[int]] = defaultdict(list)
        calls: dict[str, int] = defaultdict(int)
        for run in runs:
            for sub in run["subroutines"]:
                inclusive[sub["pc"]].append(sub["inclusive_master"])
                calls[sub["pc"]] += sub["calls"]
        ranked = sorted(inclusive.items(), key=lambda item: -sum(item[1]))[: args.top]
        mean_total = statistics.mean(totals) or 1
        for pc, values in ranked:
            share = statistics.mean(values) / mean_total * 100 * len(values) / len(runs)
            print(
                f"    {symbols.describe(int(pc, 16)):40s} in {len(values):5d} runs  calls={calls[pc]:7d}  "
                f"inclusive mean={statistics.mean(values):10.0f} min={min(values):9d} max={max(values):9d}  ~{share:5.1f}% of plan"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
