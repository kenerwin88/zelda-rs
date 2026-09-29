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
WITNESS = re.compile(
    r"host: (\d+), write: NativeExactCpuWrite \{ pc: (\d+), "
    r"address: (\d+), value: (\d+)"
)
TILESET = re.compile(
    r"TilesetDecompressionOutput\(slot=(\d+), "
    r"source_sheet=Some\((\d+)\), native_sheet=(\d+), "
    r"subset_source_writer=(.*?), buffer=([0-9a-f]+), "
    r"offset=([0-9a-f]+), source_output=(.*?), native_output=(.*?)\)$"
)
SHEET_OUTPUT = re.compile(r"Some\(\(Some\((\d+)\), Some\((\d+)\)\)\)")


def field(line: str, start: str, end: str | None = None) -> str:
    value = line.split(start, 1)[1]
    return value.split(end, 1)[0] if end else value.strip()


def witness_summary(value: str) -> str:
    match = WITNESS.search(value)
    if not match:
        return value
    host, pc, address, byte = map(int, match.groups())
    return (
        f"host {host}, PC ${pc >> 16:02X}:{pc & 0xFFFF:04X}, "
        f"address ${address >> 16:02X}:{address & 0xFFFF:04X}, byte {byte:02X}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path, help="native_exact_cpu_diagnose.sh output")
    parser.add_argument(
        "--from-host", type=int, default=0,
        help="start at this host when an earlier checkpoint retains known mismatches",
    )
    parser.add_argument(
        "--address-only", action="store_true",
        help="print the first differing WRAM offset for a watched replay",
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
        if not args.address_only:
            print("No differing CPU read was reported.")
        return

    host = int(field(mismatch, "host=", " "))
    source, trial = [tuple(map(int, match)) for match in READ.findall(mismatch)[:2]]
    pc, address, source_value, width, _ = source
    trial_value = trial[2]
    if args.address_only:
        print(f"{address & 0xFFFF:04x}")
        return
    print(
        f"First read: host {host}, PC ${pc >> 16:02X}:{pc & 0xFFFF:04X}, "
        f"address ${address >> 16:02X}:{address & 0xFFFF:04X}, "
        f"source {source_value:0{width * 2}X}, trial {trial_value:0{width * 2}X}"
    )
    print(
        "Source CPU last writer: "
        + witness_summary(field(mismatch, "source_last_writer=", " trial_last_writer="))
    )
    print(
        "Trial CPU last writer: "
        + witness_summary(field(mismatch, "trial_last_writer=", " rebase_before_native="))
    )
    owner = field(mismatch, "native_state_owner=", " native_continuation=")
    tileset = TILESET.fullmatch(owner)
    if tileset:
        slot, source_sheet, native_sheet, subset_writer, buffer, offset, source, native = (
            tileset.groups()
        )
        print(f"Native state owner: TilesetDecompressionOutput, slot {slot}")
        print(
            f"Sheet selection: source {source_sheet} ({witness_summary(subset_writer)}); "
            f"native retained {native_sheet}"
        )
        source_output = SHEET_OUTPUT.fullmatch(source)
        native_output = SHEET_OUTPUT.fullmatch(native)
        if source_output and native_output:
            source_byte, source_cycles = map(int, source_output.groups())
            native_byte, native_cycles = map(int, native_output.groups())
            print(
                f"Asset output at ${buffer}+${offset}: source sheet {source_byte:02X} "
                f"after {source_cycles} decompressor master cycles; native sheet "
                f"{native_byte:02X} after {native_cycles} cycles"
            )
    else:
        print("Native state owner: " + owner)
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
        (int(match.group(1)), line)
        for line in lines
        if (match := NATIVE_WRITE.match(line))
        and match.group(2) == native_address
    ]
    previous = [line for write_host, line in writers if write_host < host]
    following = [line for write_host, line in writers if write_host >= host]
    if previous:
        print("Native last writer: " + previous[-1])
    else:
        print("Native last writer: not observed by the enabled WRAM write hooks")
    if following:
        print("Native next writer: " + following[0])

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
