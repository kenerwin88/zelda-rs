#!/usr/bin/env python3
"""Validate rendered modern-audio quality and determinism in route traces."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections.abc import Iterable, Iterator
from pathlib import Path


def load_frames(paths: list[Path]) -> list[dict]:
    return list(iter_frames(paths))


def iter_frames(paths: list[Path]) -> Iterator[dict]:
    for path in paths:
        if str(path) == "-":
            lines: Iterable[str] = sys.stdin
        else:
            lines = path.open(encoding="utf-8")
        for line in lines:
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(value, dict) and "modern_audio" in value:
                yield value
        if str(path) != "-":
            lines.close()


def check_frames(
    frames: list[dict], *, require_zero_unknown_sfx: bool = False
) -> tuple[list[str], str]:
    failures, digest, _ = check_frame_stream(
        frames, require_zero_unknown_sfx=require_zero_unknown_sfx
    )
    return failures, digest


def check_frame_stream(
    frames: Iterable[dict], *, require_zero_unknown_sfx: bool = False
) -> tuple[list[str], str, int]:
    failures: list[str] = []
    digest = hashlib.sha256()
    count = 0
    active_silence_start: int | None = None
    for frame in frames:
        count += 1
        frame_no = int(frame["frame"])
        audio = frame["modern_audio"]
        digest.update(frame_no.to_bytes(4, "little"))
        digest.update(int(str(audio["hash"]), 0).to_bytes(4, "little"))
        if int(audio["ignored_events"]) != 0:
            failures.append(
                f"frame={frame_no} ignored_events={audio['ignored_events']}"
            )
        if require_zero_unknown_sfx and int(frame.get("modern_sfx_unknown") or 0) != 0:
            failures.append(
                f"frame={frame_no} unknown_sfx={frame['modern_sfx_unknown']} "
                f"programs={frame.get('modern_sfx_unknown_programs') or []}"
            )
        notes = frame.get("modern_note_events") or []
        oracle_audible = int(frame.get("peak", 1)) != 0
        if notes and oracle_audible and int(audio["peak"]) == 0:
            failures.append(f"frame={frame_no} note-on rendered silence")
        active_silent = (
            int(audio["active_voices"]) != 0
            and oracle_audible
            and int(audio["peak"]) == 0
        )
        if active_silent:
            if active_silence_start is None:
                active_silence_start = frame_no
            elif frame_no == active_silence_start + 1:
                failures.append(
                    f"frame={active_silence_start} active voices rendered sustained silence"
                )
        else:
            active_silence_start = None
        if (
            len(notes) == 1
            and int(audio["peak"]) != 0
            and int(audio["active_voices"]) == 1
        ):
            pan = int(notes[0]["pan"])
            left = int(audio["left_abs"])
            right = int(audio["right_abs"])
            if pan < 0 and left <= right:
                failures.append(
                    f"frame={frame_no} left pan has energy {left}/{right}"
                )
            elif pan > 0 and right <= left:
                failures.append(
                    f"frame={frame_no} right pan has energy {left}/{right}"
                )
    return failures, digest.hexdigest(), count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="+", type=Path)
    parser.add_argument("--expect-digest")
    parser.add_argument("--require-zero-unknown-sfx", action="store_true")
    args = parser.parse_args()
    failures, digest, frame_count = check_frame_stream(
        iter_frames(args.trace), require_zero_unknown_sfx=args.require_zero_unknown_sfx
    )
    if args.expect_digest and digest != args.expect_digest:
        failures.append(f"digest mismatch expected={args.expect_digest} actual={digest}")
    if failures:
        for failure in failures[:20]:
            print(f"modern audio route check failed: {failure}")
        return 1
    print(f"modern audio route check passed: frames={frame_count} digest={digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
