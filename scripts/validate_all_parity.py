#!/usr/bin/env python3
"""Validate THIS repo against the captured C-oracle golden fingerprints.

This is the release/parity compatibility entrypoint. It intentionally does not
run the C oracle. The C-derived truth lives in `parity-golden/manifest.json`,
`rollup.bin`, and `merkle.bin`; `zparity check` compares Rust frame
fingerprints against that golden set.

Large checks are sharded by `zparity check` by default. It seeds compatible Rust
checkpoints once, then fans out replay shards across the available CPU cores.

Usage:
  scripts/validate_all_parity.py                 # fast smoke test (3000 frames)
  scripts/validate_all_parity.py --frames 12000
  scripts/validate_all_parity.py --full          # full route + GPU checkpoint sweep
  scripts/validate_all_parity.py --full --shards 8
  scripts/validate_all_parity.py --gpu-checkpoint-sweep
  scripts/validate_all_parity.py --full --skip-gpu-checkpoint-sweep
  scripts/validate_all_parity.py --build         # build target/parity/zelda3 first
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frames", type=int, default=3000)
    parser.add_argument("--full", action="store_true")
    parser.add_argument("--shards", type=int)
    parser.add_argument("--build", action="store_true")
    parser.add_argument(
        "--detail",
        action="store_true",
        help="pass through to zparity check for detailed failure guidance",
    )
    parser.add_argument(
        "--render-stride",
        type=int,
        default=1,
        help="accepted for legacy callers; zparity fingerprints every checked frame",
    )
    parser.add_argument(
        "--audio-stride",
        type=int,
        default=1,
        help="accepted for legacy callers; zparity fingerprints every checked frame",
    )
    parser.add_argument(
        "--no-audio",
        action="store_true",
        help="accepted for legacy callers; golden rollups include the fingerprint mask",
    )
    parser.add_argument(
        "--gpu-checkpoint-sweep",
        action="store_true",
        help=(
            "after zparity passes, run the self-seeding full-GPU modern-index "
            "checkpoint sweep, including Mode-7 coverage"
        ),
    )
    parser.add_argument(
        "--skip-gpu-checkpoint-sweep",
        action="store_true",
        help="with --full, run only the zparity rollup and skip the GPU checkpoint sweep",
    )
    return parser.parse_args(argv)


def zparity_check_command(args: argparse.Namespace) -> list[str]:
    cmd = ["cargo", "run", "-p", "parity", "--", "check"]
    if args.full:
        cmd.append("--full")
    else:
        cmd.extend(["--frames", str(args.frames)])
    if args.shards is not None:
        cmd.extend(["--shards", str(args.shards)])
    if args.detail:
        cmd.append("--detail")
    return cmd


def gpu_checkpoint_sweep_command() -> list[str]:
    return [
        sys.executable,
        "scripts/gpu_render_compare_checkpoint_sweep.py",
        "--build",
        "--require-mode7",
    ]


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.build:
        build = ["cargo", "build", "--profile", "parity", "-p", "zelda3-bin"]
        result = subprocess.run(build, cwd=REPO)
        if result.returncode != 0:
            return result.returncode

    if args.no_audio:
        print(
            "validate_all_parity: --no-audio is ignored; zparity checks the golden rollup",
            file=sys.stderr,
        )
    if args.render_stride != 1 or args.audio_stride != 1:
        print(
            "validate_all_parity: stride flags are ignored; zparity checks every emitted fingerprint",
            file=sys.stderr,
        )

    cmd = zparity_check_command(args)
    result = subprocess.run(cmd, cwd=REPO)
    if result.returncode != 0:
        return result.returncode
    run_gpu_checkpoint_sweep = args.gpu_checkpoint_sweep or (
        args.full and not args.skip_gpu_checkpoint_sweep
    )
    if run_gpu_checkpoint_sweep:
        result = subprocess.run(gpu_checkpoint_sweep_command(), cwd=REPO)
        return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
