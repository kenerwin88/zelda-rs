#!/usr/bin/env python3
"""Compare native-state CPU timing and exact CPU WRAM-write ownership by NMI."""

import argparse
import re
from pathlib import Path


ENTRY = re.compile(r"native-tileset-shadow-entry host=(\d+)")
RETURN = re.compile(r"native-tileset-shadow-return host=(\d+) complete=(true|false) nmis=(\d+)")
SOURCE = re.compile(r"native-tileset-source-writes host=(\d+) region=\$([0-9a-f]+) stores=(\d+) address_hash=([0-9a-f]{16})")
SHADOW = re.compile(r"native-tileset-shadow-writes host=(\d+) slice=(\d+) region=\$([0-9a-f]+) stores=(\d+) address_hash=([0-9a-f]{16})")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path)
    args = parser.parse_args()
    lines = args.log.read_text(errors="replace").splitlines()
    entry = next((int(match[1]) for line in lines if (match := ENTRY.search(line))), None)
    result = next((match for line in lines if (match := RETURN.search(line))), None)
    if entry is None or result is None or result[2] != "true" or int(result[1]) != entry:
        parser.error("a completed native tileset timing run is required")
    last_host = entry + int(result[3])
    source: dict[tuple[int, int], tuple[int, int]] = {}
    shadow: dict[tuple[int, int], tuple[int, int]] = {}
    for line in lines:
        if match := SOURCE.search(line):
            host, region, stores, address_hash = int(match[1]), int(match[2], 16), int(match[3]), int(match[4], 16)
            if entry <= host <= last_host:
                if (host, region) in source:
                    parser.error(f"duplicate source host={host} region=${region:05x}")
                source[(host, region)] = (stores, address_hash)
        if match := SHADOW.search(line):
            host, slice_number, region, stores, address_hash = (
                int(match[1]), int(match[2]), int(match[3], 16), int(match[4]), int(match[5], 16)
            )
            if host != entry + slice_number:
                parser.error(f"shadow slice {slice_number} claims host {host}, expected {entry + slice_number}")
            if (host, region) in shadow:
                parser.error(f"duplicate shadow host={host} region=${region:05x}")
            shadow[(host, region)] = (stores, address_hash)
    differences = [(key, source.get(key), shadow.get(key)) for key in sorted(source.keys() | shadow.keys())
                   if source.get(key) != shadow.get(key)]
    print(f"entry_host={entry} nmis={int(result[3])} "
          f"source_regions={len(source)} shadow_regions={len(shadow)} "
          f"source_stores={sum(value[0] for value in source.values())} "
          f"shadow_stores={sum(value[0] for value in shadow.values())} "
          f"differences={len(differences)}")
    for (host, region), source_writes, shadow_writes in differences[:20]:
        print(f"host={host} region=${region:05x} source={source_writes} shadow={shadow_writes}")
    if differences:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
