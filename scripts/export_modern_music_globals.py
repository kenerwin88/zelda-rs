#!/usr/bin/env python3
"""Export track-relative DSP global automation from focused music traces."""

from __future__ import annotations

import argparse
from pathlib import Path

from extract_modern_music_catalog import extract_music, load_frames


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("traces", nargs="+", type=Path)
    parser.add_argument("--tsv-out", required=True, type=Path)
    parser.add_argument(
        "--update-existing",
        action="store_true",
        help="replace captured tracks while retaining all other tracks in the output TSV",
    )
    args = parser.parse_args()

    candidates: dict[int, list[list[dict]]] = {}
    for trace in args.traces:
        catalog = extract_music(load_frames([trace]))
        for track in catalog["tracks"]:
            events = track.get("global_events", [])
            if events:
                candidates.setdefault(int(track["track"]), []).append(events)

    by_track: dict[int, list[str]] = {}
    for track, sequences in sorted(candidates.items()):
        # Focused parity traces are bounded to one track lifecycle. Prefer the
        # richest capture when an older/current duplicate is supplied.
        events = max(sequences, key=len)
        by_track[track] = [
                f"{track:02x}\t{event['start_frame']}\t{event['sample_offset']}\t"
                f"{event['register']:02x}\t{event['value']}"
                for event in events
            ]

    if args.update_existing and args.tsv_out.exists():
        for raw in args.tsv_out.read_text(encoding="utf-8").splitlines():
            if not raw or raw.startswith("#"):
                continue
            track = int(raw.split("\t", 1)[0], 16)
            if track not in candidates:
                by_track.setdefault(track, []).append(raw)

    lines = [
        "# track\tstart_frame\tsample_offset\tregister\tvalue",
        *(line for track in sorted(by_track) for line in by_track[track]),
    ]

    args.tsv_out.parent.mkdir(parents=True, exist_ok=True)
    args.tsv_out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"modern music globals: tracks={len(candidates)} events={len(lines) - 1}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
