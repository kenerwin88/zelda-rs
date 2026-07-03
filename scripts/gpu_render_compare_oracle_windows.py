#!/usr/bin/env python3
"""Run GPU-vs-CPU render compares for recorded oracle input windows."""

from __future__ import annotations

import argparse
import csv
import os
import re
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Pattern


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WINDOWS = REPO_ROOT / "docs" / "porting" / "oracle_windows.tsv"
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT.parent / "zelda3" / "zelda3.sfc")))
DEFAULT_PROGRESS_EVERY = 10_000
SUMMARY_RE = re.compile(
    r"play-gpu-render-compare completed compared=(\d+) start_frame=(\d+) "
    r"last_frame=(\d+) last_hash=(0x[0-9a-fA-F]{8}) mismatched_pixels=(\d+)"
)
MODERN_INDEX_SUMMARY_RE = re.compile(
    r"modern_index_compare_summary compare_count=(\d+) bad_count=(\d+) bad_pixels=(\d+) "
    r"gpu_count=(\d+) mode7_gpu_count=(\d+) cpu_count=(\d+) "
    r"variant_draws=(\d+) fallback_draws=(\d+) dynamic_palette_draws=(\d+) missing_variant_draws=(\d+)"
)
MODERN_INDEX_PROGRESS_RE = re.compile(
    r"modern_index_compare_progress compare_count=(\d+) frame=(\d+) bad_count=(\d+)"
)


@dataclass(frozen=True)
class OracleWindow:
    name: str
    status: str
    frames: int
    input_script: str
    coverage: str
    notes: str


def run_command_capture_output(
    command: list[str],
    cwd: Path,
    env: dict[str, str],
    live_patterns: tuple[Pattern[str], ...] = (),
    poll_seconds: float = 1.0,
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryFile("w+", encoding="utf-8") as stdout:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            text=True,
            stdout=stdout,
            stderr=subprocess.STDOUT,
        )
        read_offset = 0
        pending = ""
        while process.poll() is None:
            if live_patterns:
                read_offset, pending = print_matching_live_lines(
                    stdout,
                    read_offset,
                    pending,
                    live_patterns,
                )
            time.sleep(poll_seconds)
        if live_patterns:
            read_offset, pending = print_matching_live_lines(
                stdout,
                read_offset,
                pending,
                live_patterns,
            )
            if pending and any(pattern.search(pending) for pattern in live_patterns):
                print(pending, flush=True)
        stdout.seek(0)
        return subprocess.CompletedProcess(
            args=command,
            returncode=process.returncode,
            stdout=stdout.read(),
        )


def print_matching_live_lines(
    stdout,
    read_offset: int,
    pending: str,
    live_patterns: tuple[Pattern[str], ...],
) -> tuple[int, str]:
    stdout.flush()
    stdout.seek(read_offset)
    chunk = stdout.read()
    read_offset = stdout.tell()
    if not chunk:
        return read_offset, pending
    pending += chunk
    lines = pending.splitlines(keepends=True)
    if lines and not lines[-1].endswith(("\n", "\r")):
        pending = lines.pop()
    else:
        pending = ""
    for line in lines:
        text = line.rstrip("\r\n")
        if any(pattern.search(text) for pattern in live_patterns):
            print(text, flush=True)
    return read_offset, pending


def load_windows(path: Path) -> list[OracleWindow]:
    with path.open(newline="") as fh:
        return [
            OracleWindow(
                name=row["name"],
                status=row["status"],
                frames=int(row["frames"]),
                input_script=row["input_script"],
                coverage=row["coverage"],
                notes=row["notes"],
            )
            for row in csv.DictReader(fh, delimiter="\t")
        ]


def selected_windows(
    windows: list[OracleWindow],
    only: list[str],
    max_frames: int | None,
    include_sram_windows: bool,
) -> list[OracleWindow]:
    selected = [window for window in windows if window.status == "pass"]
    if only:
        wanted = set(only)
        selected = [window for window in selected if window.name in wanted]
        missing = wanted.difference(window.name for window in selected)
        if missing:
            raise SystemExit(f"unknown passing window(s): {', '.join(sorted(missing))}")
    if max_frames is not None:
        selected = [window for window in selected if window.frames <= max_frames]
    if not include_sram_windows:
        selected = [
            window
            for window in selected
            if (sidecar := sram_sidecar(window)) is None or not sidecar.exists()
        ]
    return selected


def sram_sidecar(window: OracleWindow) -> Path | None:
    if not window.input_script:
        return None
    return (REPO_ROOT / window.input_script).with_suffix(".sram")


def command_for(
    window: OracleWindow,
    rom: Path,
    stride: int,
    release: bool,
    renderer: str | None = None,
) -> list[str]:
    command = ["cargo", "run"]
    if release:
        command.append("--release")
    command.extend(
        [
            "-q",
            "-p",
            "zelda3-bin",
            "--",
            "--play-gpu-render-compare",
            str(rom),
            str(window.frames),
            "--stride",
            str(stride),
        ]
    )
    if renderer == "assets-variant-gpu":
        command.extend(["--modern-index-compare", str(stride)])
    if window.input_script:
        command.extend(["--input-script", window.input_script])
        sidecar = sram_sidecar(window)
        if sidecar is not None and sidecar.exists():
            command.extend(["--load-sram", str(sidecar)])
    return command


def env_for_renderer(
    base_env: dict[str, str],
    renderer: str | None,
    progress_every: int,
) -> dict[str, str]:
    env = base_env.copy()
    if renderer:
        env["ZELDA3_RENDERER"] = renderer
    if renderer == "assets-variant-gpu":
        env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"] = "1"
        if progress_every > 0:
            env["ZELDA3_MODERN_INDEX_COMPARE_PROGRESS"] = str(progress_every)
    return env


def run_window(
    window: OracleWindow,
    rom: Path,
    stride: int,
    release: bool,
    renderer: str | None,
    progress_every: int,
) -> tuple[int, str, int, tuple[int, int, int, int]]:
    command = command_for(window, rom, stride, release, renderer)
    prefix = f"ZELDA3_RENDERER={renderer} " if renderer else ""
    print(f"running {window.name}: {prefix}{' '.join(command)}", flush=True)
    env = env_for_renderer(os.environ, renderer, progress_every)
    live_patterns = (
        (MODERN_INDEX_PROGRESS_RE,)
        if renderer == "assets-variant-gpu" and progress_every > 0
        else ()
    )
    result = run_command_capture_output(
        command,
        cwd=REPO_ROOT,
        env=env,
        live_patterns=live_patterns,
    )
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        raise SystemExit(result.returncode)
    match = SUMMARY_RE.search(result.stdout)
    if not match:
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        raise SystemExit(f"{window.name}: missing play-gpu-render-compare summary")
    compared = int(match.group(1))
    last_hash = match.group(4)
    mismatched_pixels = int(match.group(5))
    if mismatched_pixels != 0:
        raise SystemExit(f"{window.name}: reported {mismatched_pixels} mismatched pixels")
    variant_stats = (0, 0, 0, 0)
    modern_match = MODERN_INDEX_SUMMARY_RE.search(result.stdout)
    if renderer == "assets-variant-gpu":
        if not modern_match:
            if result.stdout:
                print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
            raise SystemExit(f"{window.name}: missing modern-index compare summary")
        modern_bad_pixels = int(modern_match.group(3))
        if modern_bad_pixels != 0:
            raise SystemExit(f"{window.name}: reported {modern_bad_pixels} modern-index bad pixels")
        variant_stats = (
            int(modern_match.group(7)),
            int(modern_match.group(8)),
            int(modern_match.group(9)),
            int(modern_match.group(10)),
        )
    print(
        f"{window.name}: compared={compared} frames={window.frames} "
        f"last_hash={last_hash} mismatched_pixels=0 "
        f"variant_draws={variant_stats[0]} "
        f"fallback_draws={variant_stats[1]} "
        f"dynamic_palette_draws={variant_stats[2]} "
        f"missing_variant_draws={variant_stats[3]}"
    )
    return compared, last_hash, mismatched_pixels, variant_stats


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--windows", type=Path, default=DEFAULT_WINDOWS)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--only", action="append", default=[], metavar="NAME")
    parser.add_argument("--max-frames", type=int)
    parser.add_argument("--stride", type=int, default=1)
    parser.add_argument(
        "--renderer",
        help="set ZELDA3_RENDERER for oracle window compares, e.g. assets-variant-gpu",
    )
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--progress-every",
        type=int,
        default=DEFAULT_PROGRESS_EVERY,
        help=(
            "print live modern-index progress every N compared frames for "
            "assets-variant-gpu; use 0 to disable"
        ),
    )
    parser.add_argument("--include-sram-windows", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.stride <= 0:
        raise SystemExit("--stride must be greater than zero")
    if args.max_frames is not None and args.max_frames <= 0:
        raise SystemExit("--max-frames must be greater than zero")
    if args.progress_every < 0:
        raise SystemExit("--progress-every must be zero or greater")
    if not args.rom.exists():
        raise SystemExit(f"ROM does not exist: {args.rom}")
    if not args.windows.exists():
        raise SystemExit(f"window table does not exist: {args.windows}")
    return args


def main() -> None:
    args = parse_args()
    windows = selected_windows(
        load_windows(args.windows),
        args.only,
        args.max_frames,
        args.include_sram_windows,
    )
    if not windows:
        raise SystemExit("no windows selected")

    total_compared = 0
    total_variant_draws = 0
    total_fallback_draws = 0
    total_dynamic_palette_draws = 0
    total_missing_variant_draws = 0
    for window in windows:
        if args.dry_run:
            prefix = f"ZELDA3_RENDERER={args.renderer} " if args.renderer else ""
            print(
                prefix
                + " ".join(command_for(window, args.rom, args.stride, args.release, args.renderer))
            )
            continue
        compared, _, _, variant_stats = run_window(
            window,
            args.rom,
            args.stride,
            args.release,
            args.renderer,
            args.progress_every,
        )
        total_compared += compared
        total_variant_draws += variant_stats[0]
        total_fallback_draws += variant_stats[1]
        total_dynamic_palette_draws += variant_stats[2]
        total_missing_variant_draws += variant_stats[3]

    if not args.dry_run:
        print(
            "gpu-render-oracle-windows completed "
            f"windows={len(windows)} compared={total_compared} stride={args.stride} "
            "mismatched_pixels=0 "
            f"variant_draws={total_variant_draws} "
            f"fallback_draws={total_fallback_draws} "
            f"dynamic_palette_draws={total_dynamic_palette_draws} "
            f"missing_variant_draws={total_missing_variant_draws}"
        )


if __name__ == "__main__":
    main()
