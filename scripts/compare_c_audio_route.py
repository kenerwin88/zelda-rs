#!/usr/bin/env python3
"""Compare full-route replay audio against the translated C engine oracle."""

from __future__ import annotations

import argparse
import json
import os
import queue
import subprocess
import sys
import threading
from pathlib import Path
from typing import TextIO


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(ROOT.parent / "zelda3")))
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(DEFAULT_C_ROOT / "zelda3.sfc")))
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_RUST_BIN = ROOT / "target" / "release" / "zelda3"
DEFAULT_FINAL_FRAME = 1_073_092

AUDIO_FIELDS = [
    "samples",
    "channels",
    "peak",
    "first_nonzero",
    "mean_abs",
    "hash",
    "apui",
    "music",
    "main",
    "sub",
    "subsub",
    "inidisp",
]
STATE_FIELDS = [
    "apui",
    "music",
    "main",
    "sub",
    "subsub",
    "inidisp",
]


class AudioRouteFailure(RuntimeError):
    pass


def start_process(name: str, command: list[str], cwd: Path) -> subprocess.Popen[str]:
    env = os.environ.copy()
    env.setdefault("SDL_VIDEODRIVER", "dummy")
    env.setdefault("SDL_AUDIODRIVER", "dummy")
    env.setdefault("SDL_RENDER_DRIVER", "software")
    try:
        return subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=1,
        )
    except OSError as exc:
        raise AudioRouteFailure(f"failed to start {name}: {exc}") from exc


def read_jsonl(
    name: str,
    stream: TextIO,
    out: queue.Queue[dict | str | None],
    trace: TextIO | None,
) -> None:
    try:
        for line in stream:
            if trace is not None:
                trace.write(line)
            stripped = line.strip()
            if not stripped or not stripped.startswith("{"):
                continue
            try:
                out.put(json.loads(stripped))
            except json.JSONDecodeError as exc:
                out.put(f"{name} emitted invalid JSON: {exc}: {stripped}")
                return
    finally:
        out.put(None)


def read_stderr(name: str, stream: TextIO, lines: list[str]) -> None:
    for line in stream:
        lines.append(line.rstrip())
        del lines[:-40]


def compare_event(c_event: dict, rust_event: dict, fields: list[str]) -> str | None:
    if c_event.get("frame") != rust_event.get("frame"):
        return f"frame mismatch: c={c_event.get('frame')} rust={rust_event.get('frame')}"
    for field in fields:
        if c_event.get(field) != rust_event.get(field):
            return (
                f"audio route mismatch frame={c_event.get('frame')} field={field}\n"
                f"c={c_event}\n"
                f"rust={rust_event}"
            )
    return None


def terminate(processes: list[subprocess.Popen[str]]) -> None:
    for process in processes:
        if process.poll() is None:
            process.terminate()
    for process in processes:
        if process.poll() is None:
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()


def run_compare(args: argparse.Namespace) -> int:
    trace_dir = args.trace_dir
    trace_dir.mkdir(parents=True, exist_ok=True)
    c_trace = None if args.no_trace_files else (trace_dir / "c-audio-route.jsonl").open("w")
    rust_trace = None if args.no_trace_files else (trace_dir / "rust-audio-route.jsonl").open("w")
    try:
        c_command = [
            str(args.c_bin),
            "--config",
            str(args.c_root / "other" / "headless_replay.ini"),
            "--replay-save",
            str(args.save),
            "--smv-test-frames",
            str(args.frames),
            "--audio-trace-log",
            str(args.stride),
        ]
        rust_command = [
            str(args.rust_bin),
            "--replay-save",
            str(args.rom),
            str(args.save),
            str(args.frames),
            "--audio-trace-log",
            str(args.stride),
        ]
        print("C command:", " ".join(c_command), flush=True)
        print("Rust command:", " ".join(rust_command), flush=True)

        c_process = start_process("c", c_command, args.c_root)
        rust_process = start_process("rust", rust_command, ROOT)
        processes = [c_process, rust_process]
        c_events: queue.Queue[dict | str | None] = queue.Queue()
        rust_events: queue.Queue[dict | str | None] = queue.Queue()
        stderr_lines = {"c": [], "rust": []}
        threads = [
            threading.Thread(target=read_jsonl, args=("c", c_process.stdout, c_events, c_trace), daemon=True),
            threading.Thread(target=read_jsonl, args=("rust", rust_process.stdout, rust_events, rust_trace), daemon=True),
            threading.Thread(target=read_stderr, args=("c", c_process.stderr, stderr_lines["c"]), daemon=True),
            threading.Thread(target=read_stderr, args=("rust", rust_process.stderr, stderr_lines["rust"]), daemon=True),
        ]
        for thread in threads:
            thread.start()

        compared = 0
        last_frame = 0
        while True:
            c_event = c_events.get()
            rust_event = rust_events.get()
            if c_event is None or rust_event is None:
                if c_event != rust_event:
                    raise AudioRouteFailure(
                        f"trace ended early: c_done={c_event is None} rust_done={rust_event is None}"
                    )
                break
            if isinstance(c_event, str) or isinstance(rust_event, str):
                raise AudioRouteFailure(str(c_event if isinstance(c_event, str) else rust_event))
            mismatch = compare_event(c_event, rust_event, STATE_FIELDS if args.state_only else AUDIO_FIELDS)
            if mismatch:
                terminate(processes)
                raise AudioRouteFailure(mismatch)
            compared += 1
            last_frame = int(c_event["frame"])
            if args.progress and compared % args.progress == 0:
                print(f"audio route progress compared={compared} frame={last_frame}", flush=True)

        c_status = c_process.wait()
        rust_status = rust_process.wait()
        if c_status != 0 or rust_status != 0:
            raise AudioRouteFailure(
                f"process failed: c={c_status} rust={rust_status}\n"
                f"c stderr tail:\n" + "\n".join(stderr_lines["c"]) + "\n"
                f"rust stderr tail:\n" + "\n".join(stderr_lines["rust"])
            )
        print(
            f"C audio route parity passed: compared={compared} last_frame={last_frame} stride={args.stride}",
            flush=True,
        )
        return 0
    finally:
        if c_trace is not None:
            c_trace.close()
        if rust_trace is not None:
            rust_trace.close()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", type=int, default=DEFAULT_FINAL_FRAME)
    parser.add_argument("--stride", type=int, default=1)
    parser.add_argument("--progress", type=int, default=10_000)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--c-root", type=Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--c-bin", type=Path, default=DEFAULT_C_ROOT / "zelda3")
    parser.add_argument("--rust-bin", type=Path, default=DEFAULT_RUST_BIN)
    parser.add_argument("--trace-dir", type=Path, default=ROOT / "target" / "parity" / "audio-route")
    parser.add_argument("--no-trace-files", action="store_true")
    parser.add_argument("--state-only", action="store_true", help="compare music/SFX command state but ignore rendered sample stats")
    args = parser.parse_args()
    if args.stride <= 0:
        parser.error("--stride must be greater than zero")
    return args


def main() -> int:
    try:
        return run_compare(parse_args())
    except AudioRouteFailure as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
