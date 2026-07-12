#!/usr/bin/env python3
"""Add exact route-observed DSP pitch words to the compact music catalog."""

from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from pathlib import Path


MATCH_FIELDS = (
    ("voice", 2),
    ("pitch", 3),
    ("instrument", 4),
    ("volume", 5),
    ("pan", 6),
    ("start_frame", 7),
    ("duration_frames", 8),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", type=Path)
    parser.add_argument("traces", nargs="+", type=Path)
    args = parser.parse_args()

    notes: dict[int, list[dict]] = defaultdict(list)
    for path in args.traces:
        value = json.loads(path.read_text(encoding="utf-8"))
        for track in value.get("tracks", []):
            notes[int(track["track"])].extend(track["notes"])

    rows = list(csv.reader(args.catalog.read_text(encoding="utf-8").splitlines(), delimiter="\t"))
    output = ["# track\tlead_in\tvoice\tpitch\tinstrument\tvolume\tpan\tstart_frame\tduration_frames\tdsp_pitch\tsample_offset"]
    for row in rows:
        if not row or row[0].startswith("#"):
            continue
        if len(row) == 11:
            output.append("\t".join(row))
            continue
        if len(row) not in (9, 10):
            raise ValueError(f"expected 9, 10, or 11 fields, got {len(row)}: {row}")
        track = int(row[0], 16)
        matches = [
            note
            for note in notes[track]
            if all(int(note[field]) == int(row[index]) for field, index in MATCH_FIELDS)
        ]
        if len(row) == 10:
            matches = [note for note in matches if int(note["dsp_pitch"]) == int(row[9])]
        pitch_words = {int(note["dsp_pitch"]) for note in matches}
        if len(pitch_words) != 1:
            raise ValueError(
                f"track {track:02x} note {row[2:]} maps to pitch words {sorted(pitch_words)}"
            )
        sample_offsets = {int(note["sample_offset"]) for note in matches}
        if len(sample_offsets) != 1:
            raise ValueError(
                f"track {track:02x} note {row[2:]} maps to sample offsets {sorted(sample_offsets)}"
            )
        enriched = row if len(row) == 10 else [*row, str(pitch_words.pop())]
        output.append("\t".join([*enriched, str(sample_offsets.pop())]))

    args.catalog.write_text("\n".join(output) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
