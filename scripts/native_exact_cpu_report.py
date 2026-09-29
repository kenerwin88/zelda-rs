#!/usr/bin/env python3
"""Summarize the first retained-CPU mismatch and its state owner."""

import argparse
import re
from pathlib import Path


READ = re.compile(
    r"NativeExactCpuRead \{ pc: (\d+), address: (\d+), value: (\d+), width: (\d+), at: (\d+) \}"
)
NATIVE_WRITE = re.compile(
    r"native-exact-cpu-native-wram-write host=(\d+) address=([0-9a-f]+) "
)


def field(line: str, start: str, end: str | None = None) -> str:
    value = line.split(start, 1)[1]
    return value.split(end, 1)[0] if end else value.strip()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path, help="native_exact_cpu_diagnose.sh output")
    parser.add_argument(
        "--from-host", type=int, default=0,
        help="start at this host when an earlier checkpoint retains known mismatches",
    )
    args = parser.parse_args()

    lines = args.log.read_text(errors="replace").splitlines()
    mismatch = next(
        (
            line for line in lines
            if line.startswith("native-exact-cpu-ownership host=")
            and int(field(line, "host=", " ")) >= args.from_host
        ),
        None,
    )
    if mismatch is None:
        print("No differing CPU read was reported.")
        return

    host = int(field(mismatch, "host=", " "))
    source, trial = [tuple(map(int, match)) for match in READ.findall(mismatch)[:2]]
    pc, address, source_value, width, _ = source
    trial_value = trial[2]
    print(
        f"First read: host {host}, PC ${pc >> 16:02X}:{pc & 0xFFFF:04X}, "
        f"address ${address >> 16:02X}:{address & 0xFFFF:04X}, "
        f"source {source_value:0{width * 2}X}, trial {trial_value:0{width * 2}X}"
    )
    print(
        "Source CPU last writer: "
        + field(mismatch, "source_last_writer=", " trial_last_writer=")
    )
    print(
        "Trial CPU last writer: "
        + field(mismatch, "trial_last_writer=", " rebase_before_native=")
    )
    print(
        "Native state owner: "
        + field(mismatch, "native_state_owner=", " native_continuation=")
    )
    print(
        "Native continuation: "
        + field(mismatch, "native_continuation=", " cached_checkpoint=")
    )
    print(
        "Rebase before native frame: "
        + field(mismatch, "rebase_before_native=", " native_state_owner=")
    )

    native_address = f"{address:06x}"
    writers = [
        line for line in lines
        if (match := NATIVE_WRITE.match(line))
        and int(match.group(1)) < host
        and match.group(2) == native_address
    ]
    if writers:
        print("Native last writer: " + writers[-1])
    else:
        print(
            "Native last writer: not observed; replay with "
            f"ZELDA3_NATIVE_EXACT_CPU_WATCH_WRAM_ADDR={address & 0xFFFF:04x}"
        )

    schedule = next(
        (
            line for line in lines
            if line.startswith("native-exact-cpu-rebase-trial host=")
            and "same_schedule=false" in line
        ),
        None,
    )
    print(
        "First CPU schedule divergence: "
        + (field(schedule, "host=", " ") if schedule else "none in this log")
    )


if __name__ == "__main__":
    main()
