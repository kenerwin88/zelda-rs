#!/usr/bin/env python3
"""Add exact DSP mixing/envelope parameters to the compact music catalog."""

from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from pathlib import Path

from extract_modern_music_catalog import extract_music, load_frames


CATALOG_FIELDS = (
    ("voice", 2),
    ("pitch", 3),
    ("instrument", 4),
    ("volume", 5),
    ("pan", 6),
    ("start_frame", 7),
    ("duration_frames", 8),
    ("dsp_pitch", 9),
    ("sample_offset", 10),
)
DSP_FIELDS = ("volume_left", "volume_right", "adsr1", "adsr2", "gain", "echo_send")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", type=Path)
    parser.add_argument("traces", nargs="+", type=Path)
    args = parser.parse_args()

    notes: dict[int, list[dict]] = defaultdict(list)
    for trace in args.traces:
        extracted = extract_music(load_frames([trace]))
        for track in extracted["tracks"]:
            notes[int(track["track"])].extend(track["notes"])

    rows = list(csv.reader(args.catalog.read_text(encoding="utf-8").splitlines(), delimiter="\t"))
    output = [
        "# track\tlead_in\tvoice\tpitch\tinstrument\tvolume\tpan\tstart_frame\t"
        "duration_frames\tdsp_pitch\tsample_offset\tvolume_left\tvolume_right\t"
        "adsr1\tadsr2\tgain\techo_send\tkeyoff_sample_offset"
    ]
    for row in rows:
        if not row or row[0].startswith("#"):
            continue
        if len(row) == 18:
            output.append("\t".join(row))
            continue
        if len(row) not in (11, 17):
            raise ValueError(f"expected 11, 17, or 18 fields, got {len(row)}: {row}")
        track = int(row[0], 16)
        matches = [
            note
            for note in notes[track]
            if all(int(note[field]) == int(row[index]) for field, index in CATALOG_FIELDS)
        ]
        if len(row) == 17:
            matches = [
                note
                for note in matches
                if all(int(note[field]) == int(row[11 + index]) for index, field in enumerate(DSP_FIELDS))
            ]
        parameter_sets = {
            tuple(int(note[field]) for field in DSP_FIELDS)
            for note in matches
        }
        if len(parameter_sets) != 1:
            raise ValueError(
                f"track {track:02x} note {row[2:]} maps to DSP parameters {sorted(parameter_sets)}"
            )
        keyoff_offsets = {int(note["keyoff_sample_offset"]) for note in matches}
        if len(keyoff_offsets) != 1:
            raise ValueError(
                f"track {track:02x} note {row[2:]} maps to keyoff offsets {sorted(keyoff_offsets)}"
            )
        enriched = row if len(row) == 17 else [*row, *(str(value) for value in parameter_sets.pop())]
        output.append("\t".join([*enriched, str(keyoff_offsets.pop())]))

    args.catalog.write_text("\n".join(output) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
