#!/usr/bin/env python3
"""Replay a captured live controller session through Rust and Snes9x together."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from input_script_tools import numeric_input_history, parse_buttons


DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT / "saves" / "zelda3.sfc")))


def session_frame_count(session_dir: Path) -> int:
    result_path = session_dir / "result.json"
    if result_path.exists():
        result = json.loads(result_path.read_text())
        for key in ("frames", "frames_completed"):
            if key in result:
                return int(result[key])
    events = session_dir / "live_inputs.jsonl"
    if events.exists():
        return len(captured_input_events(events))
    raise ValueError(f"cannot determine frame count from {session_dir}")


def captured_input_events(events_path: Path) -> list[tuple[int, int]]:
    raw_events = events_path.read_text()
    lines = raw_events.splitlines()
    events: list[tuple[int, int]] = []
    for line_number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
            frame = int(event["frame"])
            value = parse_buttons(str(event["input"]))
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            if line_number == len(lines) and not raw_events.endswith("\n"):
                break
            raise ValueError(f"{events_path}:{line_number}: invalid input event: {error}") from error
        if frame != len(events):
            raise ValueError(
                f"{events_path}:{line_number}: expected frame {len(events)}, found {frame}"
            )
        events.append((frame, value))
    if not events:
        raise ValueError(f"captured input event stream is empty: {events_path}")
    return events


def recover_input_script(session_dir: Path) -> Path:
    input_script = session_dir / "input.txt"
    if input_script.exists():
        return input_script

    events_path = session_dir / "live_inputs.jsonl"
    if not events_path.exists():
        raise ValueError(f"captured input script does not exist: {input_script}")
    events = captured_input_events(events_path)
    recovered = session_dir / "input.recovered.txt"
    recovered.write_text(numeric_input_history(events), encoding="utf-8")
    return recovered


def build_command(args: argparse.Namespace) -> list[str]:
    session_dir = args.session_dir.resolve()
    input_script = recover_input_script(session_dir)
    initial_sram = session_dir / "initial.srm"
    if not initial_sram.exists():
        raise ValueError(f"captured initial SRAM does not exist: {initial_sram}")
    frames = args.frames or session_frame_count(session_dir)
    command = [
        sys.executable,
        str(REPO_ROOT / "scripts" / "full_parity.py"),
        "--with-snes9x",
        "--no-lockstep",
        "--no-c-audio",
        "--rom",
        str(args.rom.resolve()),
        "--frames",
        str(frames),
        "--input-script",
        str(input_script),
        "--load-sram",
        str(initial_sram),
        "--work-dir",
        str((args.work_dir or session_dir / "snes9x-replay").resolve()),
    ]
    if args.snes9x_core:
        command.extend(["--snes9x-core", str(args.snes9x_core.resolve())])
    if args.no_install_snes9x:
        command.append("--no-install-snes9x")
    if args.no_exact_apu:
        command.append("--no-snes9x-exact-apu")
    if args.release:
        command.append("--release")
    return command


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("session_dir", type=Path)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--frames", type=int)
    parser.add_argument("--snes9x-core", type=Path)
    parser.add_argument("--no-install-snes9x", action="store_true")
    parser.add_argument("--no-exact-apu", action="store_true")
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--release", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if not args.session_dir.is_dir():
        print(f"session directory does not exist: {args.session_dir}", file=sys.stderr)
        return 2
    if not args.rom.exists():
        print(f"ROM does not exist: {args.rom}", file=sys.stderr)
        return 2
    try:
        command = build_command(args)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 2
    return subprocess.run(command, cwd=REPO_ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
