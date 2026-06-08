#!/usr/bin/env python3
"""Run checkpointed CPU/GPU render parity windows over a replay-save route."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT.parent / "zelda3" / "zelda3.sfc")))
DEFAULT_SAVE = REPO_ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_CHECKPOINT_DIR = REPO_ROOT / "target" / "gpu-render-checkpoints"
COMPARE_RE = re.compile(
    r"gpu-render-compare completed compared=(\d+) last_frame=(\d+) "
    r"last_hash=(0x[0-9a-fA-F]{8}) mismatched_pixels=(\d+)"
)
SAVED_RE = re.compile(r"saved replay-save checkpoint frame=(\d+) to (.+)")
SUMMARY_PREFIXES = (
    "gpu-render-compare completed ",
    "saved replay-save checkpoint ",
    "gpu-render-window-compare completed ",
)


def print_success_summary(output: str) -> None:
    for line in output.splitlines():
        if line.startswith(SUMMARY_PREFIXES):
            print(line)


def run(command: list[str]) -> str:
    print("+ " + " ".join(command), flush=True)
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        raise SystemExit(result.returncode)
    print_success_summary(result.stdout)
    return result.stdout


def cargo_prefix(release: bool) -> list[str]:
    command = ["cargo", "run"]
    if release:
        command.append("--release")
    command.extend(["-q", "-p", "zelda3-bin", "--"])
    return command


def checkpoint_path(checkpoint_dir: Path, frame: int) -> Path:
    return checkpoint_dir / f"rust-frame-{frame:06d}.sav"


def replay_command(
    *,
    rom: Path,
    save: Path,
    frames: int,
    release: bool,
    load_state: Path | None = None,
    save_state: Path | None = None,
    compare_stride: int | None = None,
) -> list[str]:
    command = [
        *cargo_prefix(release),
        "--replay-save",
        str(rom),
        str(save),
        str(frames),
    ]
    if load_state is not None:
        command.extend(["--load-state", str(load_state)])
    if save_state is not None:
        command.extend(["--save-state", str(save_state)])
    if compare_stride is not None:
        command.extend(
            [
                "--gpu-render-compare",
                str(compare_stride),
                "--gpu-render-compare-quiet",
            ]
        )
    return command


def nearest_checkpoint(checkpoint_dir: Path, frame: int) -> tuple[int, Path] | None:
    usable: list[tuple[int, Path]] = []
    for path in checkpoint_dir.glob("rust-frame-*.sav"):
        try:
            checkpoint_frame = int(path.stem.rsplit("-", 1)[1])
        except (IndexError, ValueError):
            continue
        if checkpoint_frame <= frame:
            usable.append((checkpoint_frame, path))
    if not usable:
        return None
    return max(usable, key=lambda item: item[0])


def ensure_checkpoint(
    *,
    rom: Path,
    save: Path,
    checkpoint_dir: Path,
    frame: int,
    release: bool,
) -> Path | None:
    if frame == 0:
        return None
    checkpoint_dir.mkdir(parents=True, exist_ok=True)
    wanted = checkpoint_path(checkpoint_dir, frame)
    if wanted.exists():
        print(f"checkpoint frame {frame} exists: {wanted}")
        return wanted

    nearest = nearest_checkpoint(checkpoint_dir, frame)
    if nearest is None or nearest[0] == 0:
        output = run(
            replay_command(
                rom=rom,
                save=save,
                frames=frame,
                release=release,
                save_state=wanted,
            )
        )
    else:
        nearest_frame, nearest_path = nearest
        output = run(
            replay_command(
                rom=rom,
                save=save,
                frames=frame,
                release=release,
                load_state=nearest_path,
                save_state=wanted,
            )
        )
        print(f"advanced checkpoint {nearest_frame} -> {frame}")

    match = SAVED_RE.search(output)
    if not match:
        raise SystemExit(f"missing checkpoint save confirmation for frame {frame}")
    actual_frame = int(match.group(1))
    if actual_frame != frame:
        raise SystemExit(f"checkpoint frame mismatch: expected {frame}, got {actual_frame}")
    if not wanted.exists():
        raise SystemExit(f"checkpoint was not created: {wanted}")
    return wanted


def compare_window(
    *,
    rom: Path,
    save: Path,
    checkpoint: Path | None,
    save_checkpoint: Path | None,
    start: int,
    end: int,
    stride: int,
    release: bool,
) -> tuple[int, int, str]:
    output = run(
        replay_command(
            rom=rom,
            save=save,
            frames=end,
            release=release,
            load_state=checkpoint,
            save_state=save_checkpoint,
            compare_stride=stride,
        )
    )
    match = COMPARE_RE.search(output)
    if not match:
        raise SystemExit(f"missing gpu-render-compare completion for window {start}..{end}")
    compared = int(match.group(1))
    last_frame = int(match.group(2))
    last_hash = match.group(3)
    mismatched_pixels = int(match.group(4))
    expected_min = max(0, end - start)
    if stride == 1 and compared != expected_min:
        raise SystemExit(
            f"window {start}..{end}: expected {expected_min} comparisons, got {compared}"
        )
    if mismatched_pixels != 0:
        raise SystemExit(
            f"window {start}..{end}: compare reported {mismatched_pixels} mismatched pixels"
        )
    if save_checkpoint is not None and not save_checkpoint.exists():
        raise SystemExit(f"end checkpoint was not created: {save_checkpoint}")
    return compared, last_frame, last_hash


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--checkpoint-dir", type=Path, default=DEFAULT_CHECKPOINT_DIR)
    parser.add_argument("--start", type=int, default=0)
    parser.add_argument("--end", type=int, required=True)
    parser.add_argument("--window-size", type=int, default=10_000)
    parser.add_argument("--stride", type=int, default=1)
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--no-save-end-checkpoints",
        action="store_true",
        help="do not save each window's ending frame as the next reusable checkpoint",
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.start < 0 or args.end <= args.start:
        raise SystemExit("--end must be greater than --start, and --start must be non-negative")
    if args.window_size <= 0:
        raise SystemExit("--window-size must be greater than zero")
    if args.stride <= 0:
        raise SystemExit("--stride must be greater than zero")
    if not args.rom.exists():
        raise SystemExit(f"ROM does not exist: {args.rom}")
    if not args.save.exists():
        raise SystemExit(f"replay save does not exist: {args.save}")
    return args


def main() -> None:
    args = parse_args()
    total_compared = 0
    last_frame = args.start
    last_hash = "0x00000000"

    for start in range(args.start, args.end, args.window_size):
        end = min(start + args.window_size, args.end)
        checkpoint = checkpoint_path(args.checkpoint_dir, start) if start > 0 else None
        save_checkpoint = (
            None
            if args.no_save_end_checkpoints
            else checkpoint_path(args.checkpoint_dir, end)
        )
        if args.dry_run:
            if checkpoint is not None:
                print(f"ensure checkpoint {start}: {checkpoint}")
            if save_checkpoint is None:
                print(f"compare window {start}..{end} stride={args.stride}")
            else:
                print(
                    f"compare window {start}..{end} stride={args.stride} "
                    f"save_checkpoint={save_checkpoint}"
                )
            continue

        checkpoint = ensure_checkpoint(
            rom=args.rom,
            save=args.save,
            checkpoint_dir=args.checkpoint_dir,
            frame=start,
            release=args.release,
        )
        compared, last_frame, last_hash = compare_window(
            rom=args.rom,
            save=args.save,
            checkpoint=checkpoint,
            save_checkpoint=save_checkpoint,
            start=start,
            end=end,
            stride=args.stride,
            release=args.release,
        )
        total_compared += compared

    if not args.dry_run:
        print(
            "gpu-render-window-compare completed "
            f"start={args.start} end={args.end} stride={args.stride} "
            f"compared={total_compared} last_frame={last_frame} last_hash={last_hash} "
            "mismatched_pixels=0"
        )


if __name__ == "__main__":
    main()
