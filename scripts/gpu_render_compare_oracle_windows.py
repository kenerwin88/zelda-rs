#!/usr/bin/env python3
"""Run GPU-vs-CPU render compares for recorded oracle input windows."""

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
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT.parent / "zelda3" / "zelda3.sfc")))
SUMMARY_RE = re.compile(
    r"play-gpu-render-compare completed compared=(\d+) start_frame=(\d+) "
    r"last_frame=(\d+) last_hash=(0x[0-9a-fA-F]{8}) mismatched_pixels=(\d+)"
)
MODERN_INDEX_SUMMARY_RE = re.compile(
    r"modern_index_compare_summary compare_count=(\d+) bad_count=(\d+) bad_pixels=(\d+) "
    r"gpu_count=(\d+) mode7_gpu_count=(\d+) cpu_count=(\d+) "
    r"variant_draws=(\d+) dynamic_palette_draws=(\d+) missing_variant_draws=(\d+)"
)


@dataclass(frozen=True)
class OracleWindow:
    name: str
    status: str
    frames: int
    input_script: str
    coverage: str
    notes: str


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
        selected = [window for window in selected if not sram_sidecar(window).exists()]
    return selected


def sram_sidecar(window: OracleWindow) -> Path:
    if not window.input_script:
        return Path("")
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
        if sidecar.exists():
            command.extend(["--load-sram", str(sidecar)])
    return command


def run_window(
    window: OracleWindow,
    rom: Path,
    stride: int,
    release: bool,
    renderer: str | None,
) -> tuple[int, str, int, tuple[int, int, int]]:
    command = command_for(window, rom, stride, release, renderer)
    prefix = f"ZELDA3_RENDERER={renderer} " if renderer else ""
    print(f"running {window.name}: {prefix}{' '.join(command)}", flush=True)
    env = os.environ.copy()
    if renderer:
        env["ZELDA3_RENDERER"] = renderer
    if renderer == "assets-variant-gpu":
        env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"] = "1"
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
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
    variant_stats = (0, 0, 0)
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
        )
    print(
        f"{window.name}: compared={compared} frames={window.frames} "
        f"last_hash={last_hash} mismatched_pixels=0 "
        f"variant_draws={variant_stats[0]} "
        f"dynamic_palette_draws={variant_stats[1]} "
        f"missing_variant_draws={variant_stats[2]}"
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
    parser.add_argument("--include-sram-windows", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.stride <= 0:
        raise SystemExit("--stride must be greater than zero")
    if args.max_frames is not None and args.max_frames <= 0:
        raise SystemExit("--max-frames must be greater than zero")
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
            window, args.rom, args.stride, args.release, args.renderer
        )
        total_compared += compared
        total_variant_draws += variant_stats[0]
        total_dynamic_palette_draws += variant_stats[1]
        total_missing_variant_draws += variant_stats[2]

    if not args.dry_run:
        print(
            "gpu-render-oracle-windows completed "
            f"windows={len(windows)} compared={total_compared} stride={args.stride} "
            "mismatched_pixels=0 "
            f"variant_draws={total_variant_draws} "
            f"dynamic_palette_draws={total_dynamic_palette_draws} "
            f"missing_variant_draws={total_missing_variant_draws}"
        )


if __name__ == "__main__":
    main()
