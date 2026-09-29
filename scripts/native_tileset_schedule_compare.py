#!/usr/bin/env python3
"""Compare native-state CPU timing and exact CPU WRAM-write ownership by NMI."""

import argparse
import re
from collections import Counter
from pathlib import Path


ENTRY = re.compile(r"native-tileset-shadow-entry host=(\d+)")
RETURN = re.compile(r"native-tileset-shadow-return host=(\d+) complete=(true|false) nmis=(\d+)")
SOURCE = re.compile(r"native-tileset-source-writes host=(\d+) region=\$([0-9a-f]+) stores=(\d+)")
SHADOW = re.compile(r"native-tileset-shadow-writes host=(\d+) slice=(\d+) region=\$([0-9a-f]+) stores=(\d+)")


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
    source: Counter[tuple[int, int]] = Counter()
    shadow: Counter[tuple[int, int]] = Counter()
    for line in lines:
        if match := SOURCE.search(line):
            host, region, stores = int(match[1]), int(match[2], 16), int(match[3])
            if entry <= host <= last_host:
                source[(host, region)] += stores
        if match := SHADOW.search(line):
            host, slice_number, region, stores = (
                int(match[1]), int(match[2]), int(match[3], 16), int(match[4])
            )
            if host != entry + slice_number:
                parser.error(f"shadow slice {slice_number} claims host {host}, expected {entry + slice_number}")
            shadow[(host, region)] += stores
    differences = [(key, source[key], shadow[key]) for key in sorted(source.keys() | shadow.keys())
                   if source[key] != shadow[key]]
    print(f"entry_host={entry} nmis={int(result[3])} "
          f"source_regions={len(source)} shadow_regions={len(shadow)} "
          f"source_stores={source.total()} shadow_stores={shadow.total()} "
          f"differences={len(differences)}")
    for (host, region), source_count, shadow_count in differences[:20]:
        print(f"host={host} region=${region:05x} source={source_count} shadow={shadow_count}")
    if differences:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
