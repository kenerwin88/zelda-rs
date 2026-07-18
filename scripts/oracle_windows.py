#!/usr/bin/env python3
"""List, validate, and run recorded lockstep oracle windows."""

from __future__ import annotations

import argparse
import csv
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WINDOWS = REPO_ROOT / "docs" / "porting" / "oracle_windows.tsv"
DEFAULT_CHECKPOINTS = REPO_ROOT / "docs" / "porting" / "oracle_checkpoints.tsv"
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT.parent / "zelda3" / "zelda3.sfc")))
ALLOWED_STATUSES = {"pass", "fail", "stale"}
DIGEST_RE = re.compile(r"WRAM fnv1a64 = ([0-9a-fA-F]{16})")


@dataclass(frozen=True)
class OracleWindow:
    name: str
    status: str
    frames: int
    input_script: str
    wram_digest: str
    coverage: str
    last_verified: str
    notes: str

    def command(self, rom: Path) -> list[str]:
        return self.command_for(rom, self.frames)

    def command_for(
        self,
        rom: Path,
        frames: int,
        *,
        load_state: str | None = None,
        release: bool = False,
        render: bool = False,
        semantic: bool = False,
    ) -> list[str]:
        cmd = [
            "cargo",
            "run",
        ]
        if release:
            cmd.append("--release")
        cmd.extend(
            [
                "-p",
                "zelda3-bin",
                "--",
                "--lockstep",
                str(rom),
                str(frames),
            ]
        )
        if self.input_script:
            cmd.extend(["--input-script", self.input_script])
        if load_state:
            cmd.extend(["--load-state", load_state])
        if semantic and not render:
            cmd.append("--trace-semantic-state")
        return cmd


@dataclass(frozen=True)
class OracleCheckpoint:
    name: str
    frame: int
    checkpoint_path: str
    input_script: str
    wram_digest: str
    notes: str


def load_windows(path: Path) -> list[OracleWindow]:
    with path.open(newline="") as fh:
        return [
            OracleWindow(
                name=row["name"],
                status=row["status"],
                frames=int(row["frames"]),
                input_script=row["input_script"],
                wram_digest=row["wram_digest"],
                coverage=row["coverage"],
                last_verified=row["last_verified"],
                notes=row["notes"],
            )
            for row in csv.DictReader(fh, delimiter="\t")
        ]


def load_checkpoints(path: Path) -> list[OracleCheckpoint]:
    if not path.exists():
        return []
    with path.open(newline="") as fh:
        return [
            OracleCheckpoint(
                name=row["name"],
                frame=int(row["frame"]),
                checkpoint_path=row["checkpoint_path"],
                input_script=row["input_script"],
                wram_digest=row["wram_digest"],
                notes=row["notes"],
            )
            for row in csv.DictReader(fh, delimiter="\t")
        ]


def validate_windows(path: Path, windows: list[OracleWindow], rom: Path) -> list[str]:
    errors: list[str] = []
    seen: set[str] = set()
    if not rom.exists():
        errors.append(f"ROM does not exist: {rom}")
    for window in windows:
        if window.name in seen:
            errors.append(f"{path}: duplicate oracle window {window.name!r}")
        seen.add(window.name)
        if window.status not in ALLOWED_STATUSES:
            errors.append(f"{path}: {window.name}: unknown status {window.status!r}")
        if window.frames <= 0:
            errors.append(f"{path}: {window.name}: frames must be positive")
        if window.input_script and not (REPO_ROOT / window.input_script).exists():
            errors.append(f"{path}: {window.name}: input script does not exist: {window.input_script}")
        if window.status == "pass" and not window.wram_digest:
            errors.append(f"{path}: {window.name}: passing windows must record a WRAM digest")
    return errors


def validate_checkpoints(
    path: Path,
    checkpoints: list[OracleCheckpoint],
    windows_by_name: dict[str, OracleWindow],
    require_files: bool,
) -> list[str]:
    errors: list[str] = []
    seen: set[tuple[str, int]] = set()
    for checkpoint in checkpoints:
        key = (checkpoint.name, checkpoint.frame)
        if key in seen:
            errors.append(f"{path}: duplicate checkpoint {checkpoint.name!r} at frame {checkpoint.frame}")
        seen.add(key)
        window = windows_by_name.get(checkpoint.name)
        if window is None:
            errors.append(f"{path}: checkpoint references unknown window {checkpoint.name!r}")
            continue
        if checkpoint.frame <= 0:
            errors.append(f"{path}: {checkpoint.name}: checkpoint frame must be positive")
        if checkpoint.frame > window.frames:
            errors.append(
                f"{path}: {checkpoint.name}: checkpoint frame {checkpoint.frame} exceeds window frame {window.frames}"
            )
        if checkpoint.input_script != window.input_script:
            errors.append(
                f"{path}: {checkpoint.name}: checkpoint input script {checkpoint.input_script!r} does not match window {window.input_script!r}"
            )
        if checkpoint.input_script and not (REPO_ROOT / checkpoint.input_script).exists():
            errors.append(
                f"{path}: {checkpoint.name}: input script does not exist: {checkpoint.input_script}"
            )
        if not checkpoint.wram_digest:
            errors.append(f"{path}: {checkpoint.name}: checkpoint must record a WRAM digest")
        checkpoint_file = REPO_ROOT / checkpoint.checkpoint_path
        if require_files and not checkpoint_file.exists():
            errors.append(f"{path}: {checkpoint.name}: checkpoint file does not exist: {checkpoint.checkpoint_path}")
    return errors


def print_windows(
    windows: list[OracleWindow],
    checkpoints: list[OracleCheckpoint],
    rom: Path,
    release: bool,
    render: bool,
    semantic: bool,
) -> None:
    checkpoints_by_name = group_checkpoints(checkpoints)
    for window in windows:
        print(f"{window.name}\t{window.status}\t{window.frames}\t{window.wram_digest}")
        print(f"  coverage: {window.coverage}")
        print(
            f"  command: {' '.join(window.command_for(rom, window.frames, release=release, render=render, semantic=semantic))}"
        )
        checkpoint = best_checkpoint_for(window, checkpoints_by_name.get(window.name, []))
        if checkpoint is not None:
            tail_frames = window.frames - checkpoint.frame
            print(
                f"  fast: {' '.join(window.command_for(rom, tail_frames, load_state=checkpoint.checkpoint_path, release=release, render=render, semantic=semantic))}"
            )


def group_checkpoints(checkpoints: list[OracleCheckpoint]) -> dict[str, list[OracleCheckpoint]]:
    grouped: dict[str, list[OracleCheckpoint]] = {}
    for checkpoint in checkpoints:
        grouped.setdefault(checkpoint.name, []).append(checkpoint)
    return grouped


def best_checkpoint_for(
    window: OracleWindow,
    checkpoints: list[OracleCheckpoint],
) -> OracleCheckpoint | None:
    usable = [
        checkpoint
        for checkpoint in checkpoints
        if checkpoint.name == window.name
        and checkpoint.input_script == window.input_script
        and 0 < checkpoint.frame < window.frames
        and (REPO_ROOT / checkpoint.checkpoint_path).exists()
    ]
    if not usable:
        return None
    return max(usable, key=lambda checkpoint: checkpoint.frame)


def selected_windows(
    windows: list[OracleWindow],
    include_failing: bool,
    only: list[str],
) -> list[OracleWindow]:
    selected = windows if include_failing else [w for w in windows if w.status == "pass"]
    if only:
        wanted = set(only)
        selected = [window for window in selected if window.name in wanted]
        missing = wanted.difference(window.name for window in selected)
        if missing:
            raise SystemExit(f"unknown or unselected oracle window(s): {', '.join(sorted(missing))}")
    return selected


def run_window(
    window: OracleWindow,
    rom: Path,
    *,
    checkpoints: list[OracleCheckpoint],
    fast: bool,
    release: bool,
    render: bool,
    semantic: bool,
) -> None:
    checkpoint = best_checkpoint_for(window, checkpoints) if fast else None
    if checkpoint is None:
        print(f"running {window.name}")
        command = window.command_for(rom, window.frames, release=release, render=render, semantic=semantic)
    else:
        tail_frames = window.frames - checkpoint.frame
        print(
            f"running {window.name} from checkpoint frame {checkpoint.frame} ({tail_frames} tail frame(s))"
        )
        command = window.command_for(
            rom,
            tail_frames,
            load_state=checkpoint.checkpoint_path,
            release=release,
            render=render,
            semantic=semantic,
        )

    result = subprocess.run(command, cwd=REPO_ROOT, text=True, capture_output=True)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="")
    if result.returncode != 0:
        raise SystemExit(result.returncode)
    match = DIGEST_RE.search(result.stdout)
    if window.status == "pass" and window.wram_digest and not match:
        raise SystemExit(f"{window.name}: missing WRAM digest in oracle output")
    if window.status == "pass" and window.wram_digest and match:
        actual = match.group(1).lower()
        expected = window.wram_digest.lower()
        if actual != expected:
            raise SystemExit(
                f"{window.name}: digest mismatch: expected {expected}, got {actual}"
            )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--windows", type=Path, default=DEFAULT_WINDOWS)
    parser.add_argument("--checkpoints", type=Path, default=DEFAULT_CHECKPOINTS)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--check", action="store_true")
    parser.add_argument(
        "--check-checkpoints",
        action="store_true",
        help="also require checkpoint files listed in the checkpoint ledger to exist",
    )
    parser.add_argument("--run", action="store_true", help="run all passing oracle windows")
    parser.set_defaults(fast=True, release=True)
    parser.add_argument(
        "--fast",
        dest="fast",
        action="store_true",
        help="resume from the newest recorded checkpoint before each window's final frame; default",
    )
    parser.add_argument(
        "--cold",
        dest="fast",
        action="store_false",
        help="run each oracle window from frame 0 instead of resuming from checkpoints",
    )
    parser.add_argument(
        "--release",
        dest="release",
        action="store_true",
        help="run the oracle binary with Cargo's release profile; default",
    )
    parser.add_argument(
        "--debug",
        dest="release",
        action="store_false",
        help="run the oracle binary with Cargo's debug profile",
    )
    parser.add_argument(
        "--render",
        action="store_true",
        help="use render-aware lockstep comparison for windows that need visible-frame validation",
    )
    parser.add_argument(
        "--semantic",
        action="store_true",
        help="include --trace-semantic-state in lockstep route logs; ignored for --render",
    )
    parser.add_argument(
        "--only",
        action="append",
        default=[],
        metavar="NAME",
        help="run only the named oracle window; may be passed multiple times",
    )
    parser.add_argument(
        "--include-failing",
        action="store_true",
        help="include windows marked fail/stale when used with --run",
    )
    args = parser.parse_args()

    windows = load_windows(args.windows)
    checkpoints = load_checkpoints(args.checkpoints)
    windows_by_name = {window.name: window for window in windows}
    errors = validate_windows(args.windows, windows, args.rom)
    errors.extend(
        validate_checkpoints(
            args.checkpoints,
            checkpoints,
            windows_by_name,
            args.check_checkpoints,
        )
    )
    if errors:
        for error in errors:
            print(error)
        raise SystemExit(1)

    if args.check:
        print(f"{args.windows}: ok")
        return

    if args.run:
        checkpoints_by_name = group_checkpoints(checkpoints)
        for window in selected_windows(windows, args.include_failing, args.only):
            run_window(
                window,
                args.rom,
                checkpoints=checkpoints_by_name.get(window.name, []),
                fast=args.fast,
                release=args.release,
                render=args.render,
                semantic=args.semantic,
            )
        return

    print_windows(windows, checkpoints, args.rom, args.release, args.render, args.semantic)


if __name__ == "__main__":
    main()
