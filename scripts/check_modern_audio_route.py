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
    frames: list[dict],
    *,
    require_zero_unknown_sfx: bool = False,
    require_sfx_lockstep: bool = False,
) -> tuple[list[str], str]:
    failures, digest, _ = check_frame_stream(
        frames,
        require_zero_unknown_sfx=require_zero_unknown_sfx,
        require_sfx_lockstep=require_sfx_lockstep,
    )
    return failures, digest


def check_frame_stream(
    frames: Iterable[dict],
    *,
    require_zero_unknown_sfx: bool = False,
    require_sfx_lockstep: bool = False,
) -> tuple[list[str], str, int]:
    failures: list[str] = []
    digest = hashlib.sha256()
    count = 0
    active_silence_start: int | None = None
    overlong_voice_runs: dict[int, tuple[int, int]] = {}
    reported_overlong_voices: set[int] = set()
    classic_voices_seen_active: set[int] = set()
    strict_sfx_keyed_mask = 0
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
        if require_sfx_lockstep:
            if "classic_sfx_events" not in frame or "modern_sfx_events" not in frame:
                failures.append(f"frame={frame_no} missing exact SFX lockstep receipts")
            else:
                classic_events = frame["classic_sfx_events"]
                modern_events = frame["modern_sfx_events"]
                if classic_events != modern_events:
                    def describe(event: object) -> str:
                        if not isinstance(event, list) or len(event) < 3:
                            return repr(event)
                        return f"kind={event[0]} offset={event[1]} voice={event[2]} data={event[3:]}"

                    mismatch = next(
                        (
                            index
                            for index, (classic, modern) in enumerate(
                                zip(classic_events, modern_events)
                            )
                            if classic != modern
                        ),
                        min(len(classic_events), len(modern_events)),
                    )
                    classic = (
                        describe(classic_events[mismatch])
                        if mismatch < len(classic_events)
                        else "<missing>"
                    )
                    modern = (
                        describe(modern_events[mismatch])
                        if mismatch < len(modern_events)
                        else "<missing>"
                    )
                    failures.append(
                        f"frame={frame_no} SFX event lockstep mismatch index={mismatch} "
                        f"classic=({classic}) modern=({modern})"
                    )
                classic_mask = int(frame.get("classic_sfx_voice_mask") or 0)
                modern_mask = int(frame.get("modern_sfx_voice_mask") or 0)
                if classic_mask != modern_mask:
                    failures.append(
                        f"frame={frame_no} SFX ownership mismatch "
                        f"classic=0x{classic_mask:02x} modern=0x{modern_mask:02x}"
                    )
                for event in classic_events:
                    if isinstance(event, list) and len(event) >= 3 and event[0] == "on":
                        strict_sfx_keyed_mask |= 1 << int(event[2])
                strict_voice_mask = classic_mask & modern_mask & strict_sfx_keyed_mask
                classic_voices = frame.get("dsp_voices") or []
                modern_voices = frame.get("modern_voices") or []
                for voice in range(min(len(classic_voices), len(modern_voices), 8)):
                    if strict_voice_mask & (1 << voice) == 0:
                        continue
                    classic_voice = classic_voices[voice]
                    modern_voice = modern_voices[voice]
                    for field in ("pitch", "gain", "state", "rate_counter", "volume"):
                        if classic_voice.get(field) == modern_voice.get(field):
                            continue
                        failures.append(
                            f"frame={frame_no} voice={voice} SFX DSP state mismatch "
                            f"field={field} classic={classic_voice.get(field)} "
                            f"modern={modern_voice.get(field)}"
                        )
                if classic_mask == 0:
                    strict_sfx_keyed_mask = 0
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
        classic_voices = frame.get("dsp_voices") or []
        modern_voices = frame.get("modern_voices") or []
        voice_count = min(len(classic_voices), len(modern_voices))
        for voice in range(voice_count):
            classic = classic_voices[voice]
            modern = modern_voices[voice]
            classic_finished = (
                int(classic.get("gain", 0)) == 0
                and int(classic.get("sample", 0)) == 0
            )
            if not classic_finished:
                classic_voices_seen_active.add(voice)
            modern_audible = (
                bool(modern.get("active"))
                and (
                    int(modern.get("gain", 0)) != 0
                    or int(modern.get("sample", 0)) != 0
                )
            )
            if (
                voice in classic_voices_seen_active
                and classic_finished
                and modern_audible
            ):
                start, run = overlong_voice_runs.get(voice, (frame_no, 0))
                run += 1
                overlong_voice_runs[voice] = (start, run)
                if run >= 3 and voice not in reported_overlong_voices:
                    failures.append(
                        f"frame={start} voice={voice} remained audible after C key-off"
                    )
                    reported_overlong_voices.add(voice)
            else:
                overlong_voice_runs.pop(voice, None)
                reported_overlong_voices.discard(voice)
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
    parser.add_argument("--require-sfx-lockstep", action="store_true")
    args = parser.parse_args()
    failures, digest, frame_count = check_frame_stream(
        iter_frames(args.trace),
        require_zero_unknown_sfx=args.require_zero_unknown_sfx,
        require_sfx_lockstep=args.require_sfx_lockstep,
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
