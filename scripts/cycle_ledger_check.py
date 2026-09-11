#!/usr/bin/env python3
"""Check cycle-ledger charges against shadow-CPU profiles, host by host.

`ZELDA3_DEBUG_CYCLE_LEDGER=<dir>` writes `ledger.csv` (host, ROM address,
master cycles) for every annotated routine the translated engine ran;
`ZELDA3_DEBUG_ROM_CPU_PROFILE=<dir>` writes one JSON per shadow run with
each subroutine's inclusive and interrupt cycles. For every host that has
both, this compares the ledger's total per routine with the profile's
`inclusive - interrupt` total per routine and reports exact hosts,
mismatches, and the routines annotated so far with their coverage of the
profiled cycles.

usage: cycle_ledger_check.py <ledger-dir> <profile-dir> [--symbols names.txt]
"""
import argparse
import bisect
import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

DEFAULT_SYMBOLS = Path(
    os.environ.get("ZELDA3_ROM_SYMBOLS", "/Users/missingno/Documents/zelda3/other/names.txt")
)


# Annotated regions the profile cannot isolate: the main-loop iteration (the
# plans start inside it and its wait spin is not work) and tail-jump
# dispatchers whose frame stays open through the code they jump to.
SKIP_CHECK = {
    0x008034,  # ZeldaRunGameLoop iteration
    0x0080b5,  # Module_MainRouting (JML into the module)
    0x008781,  # JumpTableLocal (JML into the table entry)
}


def load_symbols(path):
    table = {}
    if path.is_file():
        for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
            match = re.match(r"^0x([0-9a-fA-F]{4,6}):\s*(\S+)", line)
            if match:
                address = int(match.group(1), 16)
                table.setdefault(((address >> 16) & 0x7F) << 16 | (address & 0xFFFF), match.group(2))
    return table


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("ledger_dir", type=Path)
    parser.add_argument("profile_dir", type=Path)
    parser.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    parser.add_argument("--show", type=int, default=20)
    parser.add_argument("--window", type=int, default=6, help="hosts of slack for deferred calls")
    args = parser.parse_args()
    symbols = load_symbols(args.symbols)
    name = lambda address: f"{address:06x} {symbols.get(address, '')}".strip()

    ledger = defaultdict(lambda: defaultdict(int))
    ledger_calls = defaultdict(lambda: defaultdict(int))
    for line in (args.ledger_dir / "ledger.csv").read_text().splitlines():
        host, address, master = line.split(",")
        ledger[int(host)][int(address, 16)] += int(master)
        ledger_calls[int(host)][int(address, 16)] += 1

    # A routine's own cost: the profile's inclusive cycles minus the inclusive
    # cycles of the frames it called (interrupt handlers included), which is
    # what its annotation charges since annotated callees charge themselves
    # and the ledger records self charges too.
    annotated = sorted({address for host in ledger.values() for address in host})
    profile = defaultdict(lambda: defaultdict(int))
    profile_calls = defaultdict(lambda: defaultdict(int))
    profiled_total = defaultdict(int)
    # A plan may run twice on one host (both ends of an entry envelope);
    # count each (host, plan) once.
    seen_plans = set()
    for path in sorted(args.profile_dir.glob("host-*.json")):
        try:
            run = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        host = run["host"]
        if (host, run["entry_pc"]) in seen_plans:
            continue
        seen_plans.add((host, run["entry_pc"]))
        profiled_total[host] += run["total_master"]
        for sub in run["subroutines"]:
            address = int(sub["pc"], 16)
            profile[host][address] += sub["inclusive_master"] - sub.get("callee_master", 0)
            profile_calls[host][address] += sub["calls"]
    exact = 0
    mismatches = []
    compared = 0
    covered = 0
    total = 0
    # Per host first; a routine the ROM plan ran on host h that the
    # translation charged within `--window` hosts of h (a deferred or
    # scheduled call) is matched there before counting a mismatch.
    ledger_hosts = sorted(ledger)
    matched_ledger = set()
    for host in sorted(profile):
        for address in annotated:
            if address in SKIP_CHECK:
                continue
            measured = profile[host].get(address)
            if measured is None:
                continue
            compared += 1
            charged = ledger[host].get(address)
            if charged == measured and (host, address) not in matched_ledger:
                matched_ledger.add((host, address))
                exact += 1
                continue
            found = None
            for near in range(host - args.window, host + args.window + 1):
                if near != host and (near, address) not in matched_ledger and ledger.get(near, {}).get(address) == measured:
                    found = near
                    break
            if found is not None:
                matched_ledger.add((found, address))
                exact += 1
                continue
            mismatches.append((host, address, charged, measured, ledger_calls[host][address], profile_calls[host][address]))
    # Run totals per routine: the strongest single number per routine.
    # (over the hosts the profile covers, with the same window of slack)
    totals = {}
    for address in annotated:
        if address in SKIP_CHECK:
            continue
        profile_sum = sum(hosts.get(address, 0) for hosts in profile.values())
        covered_hosts = set()
        for host in profile:
            if address in profile[host]:
                covered_hosts.update(range(host - args.window, host + args.window + 1))
        ledger_sum = sum(ledger[host].get(address, 0) for host in covered_hosts if host in ledger)
        totals[address] = (ledger_sum, profile_sum)
    print(f"annotated routines: {len(annotated)}")
    for address in annotated:
        hosts = sum(1 for host in ledger if address in ledger[host])
        if address in SKIP_CHECK:
            print(f"  {name(address):40s} charged on {hosts} hosts (not checkable against the profile)")
            continue
        ledger_sum, profile_sum = totals[address]
        verdict = "EXACT" if ledger_sum == profile_sum else f"ledger {ledger_sum} vs profile {profile_sum} ({(ledger_sum - profile_sum) / profile_sum * 100:+.2f}%)" if profile_sum else "no profile"
        print(f"  {name(address):40s} charged on {hosts} hosts; run total {verdict}")
    print(f"host x routine comparisons: {compared}, exact: {exact}, mismatched: {len(mismatches)}")
    for host, address, charged, measured, calls, pcalls in mismatches[: args.show]:
        print(f"  host {host} {name(address)}: ledger {charged} ({calls} calls) vs profile {measured} ({pcalls} calls)")
    if total:
        print(f"profiled cycles covered by annotated routines: {covered / total * 100:.2f}% of {total}")
    return 0 if not mismatches else 1


if __name__ == "__main__":
    sys.exit(main())
