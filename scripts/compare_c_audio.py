#!/usr/bin/env python3
"""Compare Rust startup audio against the translated C engine oracle."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_REPO = Path(os.environ.get("ZELDA3_C_REPO", str(REPO_ROOT.parent / "zelda3")))
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(DEFAULT_C_REPO / "zelda3.sfc")))
DEFAULT_WORK_DIR = REPO_ROOT / "target" / "parity" / "audio-c-oracle"


class AudioParityFailure(RuntimeError):
    pass


def run_command(command: list[str], cwd: Path) -> str:
    result = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    if result.returncode != 0:
        tail = (result.stderr or result.stdout).strip().splitlines()[-40:]
        raise AudioParityFailure(
            f"command failed with exit code {result.returncode}\n"
            f"command: {' '.join(command)}\n"
            + "\n".join(tail)
        )
    return result.stdout


def cargo_zelda(args: list[str], release: bool) -> list[str]:
    command = ["cargo", "run", "-q"]
    if release:
        command.append("--release")
    command.extend(["-p", "zelda3-bin", "--"])
    command.extend(args)
    return command


def parse_jsonl(text: str, label: str) -> list[dict]:
    events = []
    for line_no, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            events.append(json.loads(line))
        except json.JSONDecodeError as exc:
            raise AudioParityFailure(f"{label} line {line_no} is not JSON: {exc}") from exc
    return events


def write_trace(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def run_c_trace(args: argparse.Namespace) -> list[dict]:
    command = [str(args.c_bin), "--trace-audio", str(args.frames)]
    stdout = run_command(command, args.c_repo)
    write_trace(args.work_dir / "c-audio.jsonl", stdout)
    return parse_jsonl(stdout, "C trace")


def run_rust_trace(args: argparse.Namespace) -> list[dict]:
    command = cargo_zelda(
        ["--trace-startup-audio", str(args.rom), str(args.frames), "--jsonl", "--c-oracle"],
        args.release,
    )
    stdout = run_command(command, REPO_ROOT)
    write_trace(args.work_dir / "rust-audio.jsonl", stdout)
    return parse_jsonl(stdout, "Rust trace")


def compare_events(c_events: list[dict], rust_events: list[dict], work_dir: Path) -> None:
    if len(c_events) != len(rust_events):
        raise AudioParityFailure(f"trace length mismatch: c={len(c_events)} rust={len(rust_events)}")
    fields = [
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
    for frame, (c_event, rust_event) in enumerate(zip(c_events, rust_events)):
        for field in fields:
            if c_event.get(field) != rust_event.get(field):
                raise AudioParityFailure(
                    f"audio parity mismatch frame={frame} field={field}\n"
                    f"c={c_event}\n"
                    f"rust={rust_event}\n"
                    f"traces: {work_dir / 'c-audio.jsonl'} {work_dir / 'rust-audio.jsonl'}"
                )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", type=int, default=120)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--c-repo", type=Path, default=DEFAULT_C_REPO)
    parser.add_argument("--c-bin", type=Path, default=DEFAULT_C_REPO / "zelda3")
    parser.add_argument("--work-dir", type=Path, default=DEFAULT_WORK_DIR)
    parser.add_argument("--release", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        c_events = run_c_trace(args)
        rust_events = run_rust_trace(args)
        compare_events(c_events, rust_events, args.work_dir)
    except AudioParityFailure as exc:
        print(str(exc), file=sys.stderr)
        return 1
    print(
        f"C audio oracle parity passed for {args.frames} frames; "
        f"traces: {args.work_dir / 'c-audio.jsonl'} {args.work_dir / 'rust-audio.jsonl'}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
