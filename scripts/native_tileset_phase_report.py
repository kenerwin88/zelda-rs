#!/usr/bin/env python3
"""Report CPU-work offsets of one InitializeTilesets call from an opt-in replay log."""

import argparse
import re
from pathlib import Path


WORK = re.compile(r"native-tileset-work host=(\d+) master_cycles=(\d+)")
PHASE = re.compile(
    r"native-tileset-phase host=(\d+) phase=([^ ]+) pc=([0-9a-f]+) "
    r"at=(\d+) work_before=(\d+)"
)
SELECTION = re.compile(
    r"native-tileset-selection host=(\d+) slot=(\d+) sheet=(\d+) "
    r"pc=([0-9a-f]+) at=(\d+) work_before=(\d+)"
)
STORES = re.compile(
    r"native-tileset-stores host=(\d+) slot=(\d+) "
    r"first_pc=[0-9a-f]+ first_address=([0-9a-f]+) first_at=\d+ "
    r"first_work=(\d+) last_address=([0-9a-f]+) last_at=\d+ last_work=(\d+)"
)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    parser.add_argument("--from-host", type=int, default=0)
    args = parser.parse_args()

    lines = args.log.read_text(errors="replace").splitlines()
    work = {
        int(match[1]): int(match[2])
        for line in lines
        if (match := WORK.search(line))
    }
    entries = [
        (int(match[1]), match[2], int(match[3], 16), int(match[5]))
        for line in lines
        if (match := PHASE.search(line))
        and match[2] == "initialize-tilesets"
        and int(match[1]) >= args.from_host
    ]
    if not entries:
        parser.error("no InitializeTilesets entry found in this log")
    start = entries[0][0]
    next_entry = next((host for host, *_ in entries[1:] if host > start), None)
    end = next_entry if next_entry is not None else max(work, default=start) + 1
    prefix = {}
    elapsed = 0
    for host in range(start, end):
        prefix[host] = elapsed
        elapsed += work.get(host, 0)

    def offset(host: int, local: int) -> int:
        return prefix[host] + local

    events = []
    for line in lines:
        if match := PHASE.search(line):
            host, phase, pc, _, local = match.groups()
            host = int(host)
            if start <= host < end:
                events.append((
                    offset(host, int(local)),
                    f"host {host}  {phase}  PC ${int(pc, 16):06X}",
                ))
        elif match := SELECTION.search(line):
            host, slot, sheet, pc, _, local = match.groups()
            host = int(host)
            if start <= host < end:
                events.append((
                    offset(host, int(local)),
                    f"host {host}  select slot {slot} sheet {sheet}  PC ${int(pc, 16):06X}",
                ))
        elif match := STORES.search(line):
            host, slot, first_address, first_work, last_address, last_work = match.groups()
            host = int(host)
            if start <= host < end:
                first = offset(host, int(first_work))
                last = offset(host, int(last_work))
                events.append((
                    first,
                    f"host {host}  slot {slot} stores ${int(first_address, 16):06X}.."
                    f"${int(last_address, 16):06X} (through work {last})",
                ))
    print(f"InitializeTilesets entry: host {start}; CPU work offsets in master cycles")
    for at, description in sorted(events):
        print(f"{at:>9}  {description}")


if __name__ == "__main__":
    main()
