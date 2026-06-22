#!/usr/bin/env python3
"""Fast standard-route C/R parity windows for inner-loop commits.

This is not the exhaustive route proof. It checks representative route frames by
reusing cached C/R checkpoints from ``scripts/replay_bisect.py`` and running only
the short window from the nearest checkpoint to the target frame. That keeps
late-route coverage cheap while preserving a live C/R comparison at each target.

Run ``scripts/test_standard_replay_parity.py`` for the full standard-route gate.
"""

from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = ROOT.parent / "zelda3"
DEFAULT_ROM = DEFAULT_C_ROOT / "zelda3.sfc"
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_RUST_BIN = ROOT / "target" / "parity" / "zelda3"
DEFAULT_CHECKPOINT_DIR = ROOT / ".cache" / "replay-bisect"

DEFAULT_FRAMES = (
    1_000,
    12_000,
    42_998,
    80_000,
    112_078,
    180_000,
    202_254,
    202_255,
    350_000,
    700_000,
    1_045_814,
    1_073_092,
)

DEFAULT_FIELDS = "ramhash,ram0,ram1,ram2,ram3,ram4,ram5,ram6,ram7,sramhash,hp,state,speed,lspeed,rng,roommask,hist,histmask,main,sub,subsub,saved,indoors,room,ow"


def parse_frames(text: str | None) -> list[int]:
    if not text:
        return list(DEFAULT_FRAMES)
    return sorted({int(chunk.strip(), 0) for chunk in text.split(",") if chunk.strip()})


def checkpoint_pairs(checkpoint_dir: pathlib.Path) -> list[tuple[int, pathlib.Path, pathlib.Path]]:
    pairs: list[tuple[int, pathlib.Path, pathlib.Path]] = []
    for c_path in checkpoint_dir.glob("c-frame-*.sav"):
        stem = c_path.stem.removeprefix("c-frame-")
        try:
            frame = int(stem)
        except ValueError:
            continue
        rust_path = checkpoint_dir / f"rust-frame-{frame}.sav"
        if rust_path.exists():
            pairs.append((frame, c_path, rust_path))
    return sorted(pairs)


def nearest_checkpoint(
    pairs: list[tuple[int, pathlib.Path, pathlib.Path]], frame: int
) -> tuple[int, pathlib.Path, pathlib.Path] | None:
    best = None
    for pair in pairs:
        if pair[0] < frame:
            best = pair
        else:
            break
    return best


def run(cmd: list[str]) -> int:
    print("+ " + " ".join(cmd), flush=True)
    return subprocess.run(cmd, cwd=ROOT).returncode


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", help="comma-separated absolute frames to check")
    parser.add_argument("--fields", default=DEFAULT_FIELDS)
    parser.add_argument("--c-root", type=pathlib.Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--rom", type=pathlib.Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=pathlib.Path, default=DEFAULT_SAVE)
    parser.add_argument("--rust-bin", type=pathlib.Path, default=DEFAULT_RUST_BIN)
    parser.add_argument("--checkpoint-dir", type=pathlib.Path, default=DEFAULT_CHECKPOINT_DIR)
    parser.add_argument(
        "--max-from-reset",
        type=int,
        default=20_000,
        help="run from reset instead of a checkpoint for frames up to this value",
    )
    parser.add_argument(
        "--no-isolate-runtime-cwd",
        action="store_true",
        help="let replay processes use the repo cwd instead of temp runtime cwds",
    )
    args = parser.parse_args()

    args.c_root = args.c_root.resolve()
    args.rom = args.rom.resolve()
    args.save = args.save.resolve()
    args.rust_bin = args.rust_bin.resolve()
    args.checkpoint_dir = args.checkpoint_dir.resolve()

    missing = [
        path
        for path in [args.c_root / "zelda3", args.c_root / "other" / "headless_replay.ini", args.rom, args.save, args.rust_bin]
        if not path.exists()
    ]
    if missing:
        print("missing required fast parity artifact(s):", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 2

    pairs = checkpoint_pairs(args.checkpoint_dir)
    failures = 0
    for frame in parse_frames(args.frames):
        cmd = [
            sys.executable,
            "scripts/replay_bisect.py",
            "--check-frame",
            str(frame),
            "--fields",
            args.fields,
            "--c-root",
            str(args.c_root),
            "--rom",
            str(args.rom),
            "--save",
            str(args.save),
            "--rust-bin",
            str(args.rust_bin),
            "--no-save-local-checkpoint",
        ]
        if not args.no_isolate_runtime_cwd:
            cmd.append("--isolate-runtime-cwd")
        if frame > args.max_from_reset:
            ck = nearest_checkpoint(pairs, frame)
            if ck is None:
                print(
                    f"frame {frame}: no cached checkpoint below frame; refusing slow from-reset probe",
                    file=sys.stderr,
                )
                failures += 1
                continue
            ck_frame, c_checkpoint, rust_checkpoint = ck
            cmd.extend(
                [
                    "--checkpoint-frame",
                    str(ck_frame),
                    "--c-checkpoint",
                    str(c_checkpoint),
                    "--rust-checkpoint",
                    str(rust_checkpoint),
                ]
            )
        failures += 1 if run(cmd) != 0 else 0

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
