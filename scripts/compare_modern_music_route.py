#!/usr/bin/env python3
"""Compare promoted modern music notes against focused DSP route evidence."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import extract_modern_music_catalog as extractor


PROMOTED_TRACK_INSTRUMENTS = {
    0x01: {15},
    0x02: {11, 17, 19},
    0x03: {10},
    0x04: {18},
    0x05: {9, 10, 22},
    0x07: {0, 9, 10, 22},
    0x08: {1, 6, 9, 10, 15},
    0x09: {9, 10, 17, 20, 24},
    0x0A: {9, 15},
    0x0B: {15},
    0x0C: {2, 11, 14, 17},
    0x0D: {2, 11, 17, 20},
    0x0E: {10, 18, 22, 24},
    0x10: {2, 10, 11, 12, 17, 22},
    0x11: {9, 10, 17},
    0x12: {2, 10, 16, 17},
    0x13: {2, 11, 15, 17},
    0x14: {21},
    0x15: {2, 11, 20},
    0x16: {9},
    0x17: {14},
    0x18: {2, 10, 16, 17},
    0x19: {10},
    0x1A: {10, 14},
    0x1B: {15},
    0x1C: {9},
    0x1D: {11},
    0x1E: {2, 9},
    0x1F: {2, 11, 17, 19, 24},
    0x20: {10, 24},
    0x21: {2, 9, 10, 11, 12, 15, 17, 18, 24},
    0x22: {9, 10, 11, 17, 18, 19, 22},
}


def note_key(frame: int, note: dict) -> tuple[int, int, int, int, int, int]:
    return (
        frame,
        int(note["voice"]),
        int(note["pitch"]),
        int(note["instrument"]),
        int(note["volume"]),
        int(note["pan"]),
    )


def compare_frames(frames: list[dict], tracks: set[int]) -> list[str]:
    oracle_catalog = extractor.extract_music(frames)
    oracle: dict[int, list[tuple[int, int, int, int, int, int]]] = {}
    for track in oracle_catalog["tracks"]:
        track_id = int(track["track"])
        if track_id not in tracks:
            continue
        instruments = PROMOTED_TRACK_INSTRUMENTS[track_id]
        oracle[track_id] = [
            note_key(int(note["frame"]), note)
            for note in track["notes"]
            if int(note["instrument"]) in instruments
        ]

    modern: dict[int, list[tuple[int, int, int, int, int, int]]] = {
        track: [] for track in tracks
    }
    for frame in frames:
        music = frame.get("music") or [0, 0, 0]
        track = int(music[2]) if len(music) >= 3 else 0
        if track not in tracks:
            continue
        instruments = PROMOTED_TRACK_INSTRUMENTS[track]
        modern[track].extend(
            note_key(int(frame["frame"]), note)
            for note in frame.get("modern_note_events") or []
            if note.get("origin") in (None, "music")
            and int(note["instrument"]) in instruments
        )

    failures: list[str] = []
    for track in sorted(tracks):
        expected = oracle.get(track, [])
        actual = modern.get(track, [])
        if expected == actual:
            continue
        mismatch = next(
            (
                index
                for index, (left, right) in enumerate(zip(expected, actual))
                if left != right
            ),
            min(len(expected), len(actual)),
        )
        failures.append(
            f"track=0x{track:02x} mismatch_index={mismatch} "
            f"oracle={expected[mismatch:mismatch + 3]} "
            f"modern={actual[mismatch:mismatch + 3]} "
            f"counts={len(expected)}/{len(actual)}"
        )
    return failures


def load_frames(paths: list[Path]) -> list[dict]:
    return extractor.load_frames(paths)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="+", type=Path)
    parser.add_argument("--track", action="append", type=lambda value: int(value, 0))
    args = parser.parse_args()
    tracks = set(args.track or PROMOTED_TRACK_INSTRUMENTS)
    unknown = tracks - PROMOTED_TRACK_INSTRUMENTS.keys()
    if unknown:
        parser.error(f"unpromoted tracks requested: {sorted(unknown)}")
    frames = load_frames(args.trace)
    failures = compare_frames(frames, tracks)
    if failures:
        for failure in failures:
            print(f"modern music route parity failed: {failure}")
        return 1
    print(
        "modern music route parity passed: "
        + ", ".join(f"track=0x{track:02x}" for track in sorted(tracks))
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
