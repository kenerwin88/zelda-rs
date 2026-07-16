#!/usr/bin/env python3
"""Require sample-exact modern audio across the standard replay route.

The replay binary renders the legacy SPC/DSP path and ModernAudioEngine from
the same frame. This gate streams JSON trace records across the entire
requested route and reports representative stereo PCM mismatches without
materializing a multi-gigabyte trace file. Use --fail-fast for diagnosis.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from collections.abc import Iterable, Iterator


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(ROOT.parent / "zelda3")))
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(DEFAULT_C_ROOT / "zelda3.sfc")))
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_RUST_BIN = ROOT / "target" / "release" / "zelda3"
DEFAULT_ASSET_PACK = ROOT / "zelda3_assets.dat"
DEFAULT_FINAL_FRAME = 1_073_092


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
    timeline = WaveformTimeline()
    for frame in frames:
        count += 1
        frame_no = int(frame["frame"])
        audio = frame["modern_audio"]
        if (
            "samples" in frame
            and "channels" in frame
            and "oracle_exact_samples" in audio
        ):
            timeline_mismatch = timeline.compare(frame)
            if timeline_mismatch is not None:
                failures.append(timeline_mismatch)
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


class WaveformTimeline:
    """Tracks byte-exact waveform parity on one continuous sample clock."""

    def __init__(self) -> None:
        self.frames = 0
        self.sample_frames = 0
        self.interleaved_samples = 0

    def compare(self, event: dict) -> str | None:
        frame = int(event["frame"])
        samples = int(event["samples"])
        channels = int(event["channels"])
        frame_interleaved = samples * channels
        modern = event["modern_audio"]
        exact = int(modern["oracle_exact_samples"])
        first_mismatch = modern.get("oracle_first_mismatch_sample")
        if exact != frame_interleaved:
            relative = int(first_mismatch) if first_mismatch is not None else 0
            absolute = self.interleaved_samples + relative
            sample_frame, channel = divmod(absolute, channels)
            return (
                "continuous waveform mismatch "
                f"frame={frame} frame_sample={relative} absolute_interleaved={absolute} "
                f"absolute_sample_frame={sample_frame} channel={channel} "
                f"exact={exact}/{frame_interleaved}"
            )
        self.frames += 1
        self.sample_frames += samples
        self.interleaved_samples += frame_interleaved
        return None


def append_frame_range(ranges: list[list[int]], frame: int) -> None:
    if ranges and frame == ranges[-1][1] + 1:
        ranges[-1][1] = frame
    else:
        ranges.append([frame, frame])


def format_frame_ranges(ranges: list[list[int]], limit: int = 20) -> str:
    def render(frame_range: list[int]) -> str:
        start, end = frame_range
        return str(start) if start == end else f"{start}..{end}"

    selected: list[list[int] | None] = list(ranges)
    if len(selected) > limit:
        side = limit // 2
        selected = [*selected[:side], None, *selected[-side:]]
    return ", ".join("..." if item is None else render(item) for item in selected)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="*", type=Path)
    parser.add_argument("--expect-digest")
    parser.add_argument("--require-zero-unknown-sfx", action="store_true")
    parser.add_argument("--require-sfx-lockstep", action="store_true")
    parser.add_argument("--frames", type=int, default=DEFAULT_FINAL_FRAME)
    parser.add_argument("--progress", type=int, default=10_000)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--rust-bin", type=Path, default=DEFAULT_RUST_BIN)
    parser.add_argument("--asset-pack", type=Path, default=DEFAULT_ASSET_PACK)
    parser.add_argument("--input-script", type=Path)
    parser.add_argument("--load-sram", type=Path)
    parser.add_argument("--load-state", type=Path)
    parser.add_argument("--stop-replay-after-load", action="store_true")
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="stop at the first mismatch instead of scanning the full route",
    )
    parser.add_argument(
        "--sequencer",
        choices=("native", "exact-spc-driver"),
        default="native",
        help="modern sequencer under test (default: production native path)",
    )
    args = parser.parse_args()
    if args.frames <= 0:
        parser.error("--frames must be greater than zero")
    if args.progress < 0:
        parser.error("--progress must be non-negative")
    return args


def fail(process: subprocess.Popen[str], message: str) -> int:
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
    print(message, file=sys.stderr)
    return 1


def run(args: argparse.Namespace) -> int:
    command = build_replay_command(args)
    env = os.environ.copy()
    env.update(
        {
            "ZELDA3_AUDIO_SEQUENCER": args.sequencer,
            "ZELDA3_ASSET_PACK": str(args.asset_pack),
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )
    print("Modern audio route command:", " ".join(command), flush=True)
    with tempfile.TemporaryFile(mode="w+t", encoding="utf-8") as stderr:
        process = subprocess.Popen(
            command,
            cwd=ROOT,
            env=env,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=stderr,
            bufsize=1,
        )
        assert process.stdout is not None
        compared = 0
        last_frame = 0
        mismatch_count = 0
        mismatch_examples: list[str] = []
        mismatch_ranges: list[list[int]] = []
        timeline = WaveformTimeline()
        for line in process.stdout:
            if not line.startswith("{"):
                continue
            try:
                event = json.loads(line)
            except json.JSONDecodeError as error:
                return fail(process, f"invalid replay JSON: {error}: {line[:240]}")
            modern = event["modern_audio"]
            total_samples = int(event["samples"]) * int(event["channels"])
            exact_samples = int(modern["oracle_exact_samples"])
            classic_hash = event["hash"]
            modern_hash = modern["hash"]
            ignored_events = int(modern["ignored_events"])
            timeline_mismatch = timeline.compare(event)
            if (
                exact_samples != total_samples
                or modern_hash != classic_hash
                or ignored_events != 0
            ):
                mismatch_count += 1
                append_frame_range(mismatch_ranges, int(event["frame"]))
                mismatch = (
                    (timeline_mismatch or "modern audio route mismatch ")
                    + " "
                    f"frame={event['frame']} exact={exact_samples}/{total_samples} "
                    f"classic_hash={classic_hash} modern_hash={modern_hash} "
                    f"max_abs_diff={modern['oracle_max_abs_diff']} "
                    f"mean_abs_diff={modern['oracle_mean_abs_diff']} "
                    f"ignored_events={ignored_events} dsp_writes={event['dsp_writes']}"
                )
                if len(mismatch_examples) < 20:
                    mismatch_examples.append(mismatch)
                if args.fail_fast:
                    return fail(process, mismatch)
            compared += 1
            last_frame = int(event["frame"])
            if args.progress and compared % args.progress == 0:
                print(
                    f"modern audio route progress compared={compared} frame={last_frame}",
                    flush=True,
                )

        status = process.wait()
        if status != 0:
            stderr.seek(0)
            tail = stderr.read().splitlines()[-40:]
            print(
                f"replay process failed status={status}\n" + "\n".join(tail),
                file=sys.stderr,
            )
            return 1
        expected_frames = args.frames
        if args.load_state is not None:
            # A checkpoint resumes on its absolute replay clock. The trace
            # count is therefore the requested end frame minus that clock and
            # cannot be inferred from CLI arguments alone.
            expected_frames = compared
        if compared != expected_frames:
            print(
                f"trace ended early: compared={compared} expected={expected_frames} "
                f"last_frame={last_frame}",
                file=sys.stderr,
            )
            return 1
        if mismatch_count:
            print(
                f"modern audio route mismatched on {mismatch_count}/{compared} frames; "
                f"ranges: {format_frame_ranges(mismatch_ranges)}; "
                f"showing first {len(mismatch_examples)}:",
                file=sys.stderr,
            )
            for mismatch in mismatch_examples:
                print(mismatch, file=sys.stderr)
            return 1
        print(
            f"Modern audio route parity passed: compared={compared} "
            f"last_frame={last_frame} sample_frames={timeline.sample_frames} "
            f"interleaved_samples={timeline.interleaved_samples} "
            "first_mismatch=none",
            flush=True,
        )
        return 0


def build_replay_command(args: argparse.Namespace) -> list[str]:
    command = [
        str(args.rust_bin),
        "--replay-save",
        str(args.rom),
        str(args.save),
        str(args.frames),
        "--audio-trace-log",
        "1",
    ]
    if args.input_script is not None:
        command.extend(["--input-script", str(args.input_script)])
    if args.load_sram is not None:
        command.extend(["--load-sram", str(args.load_sram)])
    if args.load_state is not None:
        command.extend(["--load-state", str(args.load_state)])
    if args.stop_replay_after_load:
        command.append("--stop-replay-after-load")
    return command


def main() -> int:
    args = parse_args()
    if not args.trace:
        return run(args)
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
