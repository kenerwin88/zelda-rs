#!/usr/bin/env python3
"""Run full-GPU modern-index parity checks over saved replay checkpoints."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_BINARY = REPO_ROOT / "target" / "parity" / "zelda3"
DEFAULT_ROM = REPO_ROOT / "saves" / "zelda3.sfc"
DEFAULT_REPLAY = REPO_ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_STATE_DIR = REPO_ROOT / ".cache" / "replay-bisect"
CHECKPOINT_RE = re.compile(r"^rust-frame-(\d+)\.sav$")
SUMMARY_RE = re.compile(r"^modern_index_compare_summary\b.*$", re.MULTILINE)


@dataclass(frozen=True)
class ReplayCheckpoint:
    frame: int
    path: Path


def parse_key_value_stats(line: str) -> dict[str, int]:
    stats: dict[str, int] = {}
    for token in line.split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        if value.isdigit():
            stats[key] = int(value)
    return stats


def modern_index_summary_stats(output: str) -> dict[str, int] | None:
    match = SUMMARY_RE.search(output)
    if match is None:
        return None
    return parse_key_value_stats(match.group(0))


def discover_checkpoints(state_dir: Path) -> list[ReplayCheckpoint]:
    checkpoints: list[ReplayCheckpoint] = []
    if not state_dir.exists():
        return checkpoints
    for path in state_dir.iterdir():
        match = CHECKPOINT_RE.match(path.name)
        if match is None:
            continue
        checkpoints.append(ReplayCheckpoint(frame=int(match.group(1)), path=path))
    return sorted(checkpoints, key=lambda checkpoint: checkpoint.frame)


def select_checkpoints(
    checkpoints: list[ReplayCheckpoint],
    start_frame: int | None,
    end_frame: int | None,
    limit: int | None,
) -> list[ReplayCheckpoint]:
    selected = [
        checkpoint
        for checkpoint in checkpoints
        if (start_frame is None or checkpoint.frame >= start_frame)
        and (end_frame is None or checkpoint.frame <= end_frame)
    ]
    if limit is not None:
        selected = selected[:limit]
    return selected


def command_for_checkpoint(
    checkpoint: ReplayCheckpoint,
    binary: Path,
    rom: Path,
    replay: Path,
    frames: int,
    modern_index_compare: int,
) -> list[str]:
    return [
        str(binary),
        "--replay-save",
        str(rom),
        str(replay),
        str(checkpoint.frame + frames),
        "--load-state",
        str(checkpoint.path),
        "--modern-index-compare",
        str(modern_index_compare),
        "--require-full-gpu-path",
        "--require-modern-index-parity",
    ]


def env_for_run(base_env: dict[str, str], renderer: str | None) -> dict[str, str]:
    env = base_env.copy()
    env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"] = "1"
    if renderer:
        env["ZELDA3_RENDERER"] = renderer
    return env


def run_command(
    command: list[str],
    cwd: Path,
    env: dict[str, str],
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


CommandRunner = Callable[[list[str], Path, dict[str, str]], subprocess.CompletedProcess[str]]


def validate_summary_stats(checkpoint: ReplayCheckpoint, stats: dict[str, int]) -> None:
    bad_count = stats.get("bad_count", 0)
    bad_pixels = stats.get("bad_pixels", 0)
    cpu_count = stats.get("cpu_count", 0)
    compared = stats.get("compare_count", 0)
    gpu_count = stats.get("gpu_count", 0) + stats.get("mode7_gpu_count", 0)
    if bad_count != 0 or bad_pixels != 0:
        raise SystemExit(
            f"checkpoint {checkpoint.frame}: modern-index parity failed "
            f"bad_count={bad_count} bad_pixels={bad_pixels}"
        )
    if cpu_count != 0:
        raise SystemExit(
            f"checkpoint {checkpoint.frame}: full-GPU path violated cpu_count={cpu_count}"
        )
    if compared != 0 and gpu_count != compared:
        raise SystemExit(
            f"checkpoint {checkpoint.frame}: compared={compared} but gpu_count={gpu_count}"
        )


def run_sweep(
    checkpoints: list[ReplayCheckpoint],
    binary: Path,
    rom: Path,
    replay: Path,
    frames: int,
    modern_index_compare: int,
    renderer: str | None,
    dry_run: bool,
    runner: CommandRunner = run_command,
) -> int:
    env = env_for_run(os.environ, renderer)
    total_compared = 0
    last_frame = None
    for checkpoint in checkpoints:
        command = command_for_checkpoint(
            checkpoint,
            binary,
            rom,
            replay,
            frames,
            modern_index_compare,
        )
        print(f"checkpoint {checkpoint.frame} -> {checkpoint.frame + frames}", flush=True)
        if dry_run:
            prefix = f"ZELDA3_RENDERER={renderer} " if renderer else ""
            print(prefix + " ".join(command), flush=True)
            continue
        result = runner(command, REPO_ROOT, env)
        if result.returncode != 0:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
            return result.returncode
        stats = modern_index_summary_stats(result.stdout)
        if stats is None:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
            raise SystemExit(f"checkpoint {checkpoint.frame}: missing modern-index summary")
        validate_summary_stats(checkpoint, stats)
        total_compared += stats.get("compare_count", 0)
        last_frame = checkpoint.frame

    print(
        "checkpoint_sweep_passed "
        f"count={len(checkpoints)} compared={total_compared} "
        f"last={last_frame if last_frame is not None else 'none'}",
        flush=True,
    )
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--replay", type=Path, default=DEFAULT_REPLAY)
    parser.add_argument("--state-dir", type=Path, default=DEFAULT_STATE_DIR)
    parser.add_argument(
        "--frames",
        type=int,
        default=3,
        help="number of frames to run after each checkpoint frame",
    )
    parser.add_argument("--modern-index-compare", type=int, default=1)
    parser.add_argument("--start-frame", type=int)
    parser.add_argument("--end-frame", type=int)
    parser.add_argument("--limit", type=int)
    parser.add_argument(
        "--renderer",
        help=(
            "optionally set ZELDA3_RENDERER; omitted by default so the sweep "
            "proves the runtime default renderer"
        ),
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="build the parity-profile zelda3 binary before running",
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.frames <= 0:
        raise SystemExit("--frames must be greater than zero")
    if args.modern_index_compare <= 0:
        raise SystemExit("--modern-index-compare must be greater than zero")
    if args.limit is not None and args.limit <= 0:
        raise SystemExit("--limit must be greater than zero")
    if args.start_frame is not None and args.end_frame is not None:
        if args.start_frame > args.end_frame:
            raise SystemExit("--start-frame must be <= --end-frame")
    return args


def main() -> None:
    args = parse_args()
    if args.build:
        build = subprocess.run(
            ["cargo", "build", "--profile", "parity", "-p", "zelda3-bin"],
            cwd=REPO_ROOT,
            check=False,
        )
        if build.returncode != 0:
            raise SystemExit(build.returncode)
    checkpoints = select_checkpoints(
        discover_checkpoints(args.state_dir),
        args.start_frame,
        args.end_frame,
        args.limit,
    )
    if not checkpoints:
        raise SystemExit(f"no numeric rust-frame checkpoints found in {args.state_dir}")
    raise SystemExit(
        run_sweep(
            checkpoints,
            args.binary,
            args.rom,
            args.replay,
            args.frames,
            args.modern_index_compare,
            args.renderer,
            args.dry_run,
        )
    )


if __name__ == "__main__":
    main()
