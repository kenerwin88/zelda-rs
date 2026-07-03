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
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Pattern


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WINDOWS = REPO_ROOT / "docs" / "porting" / "oracle_windows.tsv"
DEFAULT_CHECKPOINTS = REPO_ROOT / "docs" / "porting" / "oracle_checkpoints.tsv"
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
    r"(?: stable_preview_draws=(\d+) stable_effect_draws=(\d+) dynamic_material_draws=(\d+) "
    r"missing_art_draws=(\d+) unkeyed_fallback_draws=(\d+))?"
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


@dataclass(frozen=True)
class OracleCheckpoint:
    name: str
    frame: int
    checkpoint_path: str
    input_script: str
    wram_digest: str
    notes: str


@dataclass(frozen=True)
class RunItem:
    window: OracleWindow
    checkpoint: OracleCheckpoint | None
    tail_frames: int


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


def run_items_for_windows(
    windows: list[OracleWindow],
    checkpoints_by_name: dict[str, list[OracleCheckpoint]],
    fast: bool,
    frame_limit: int | None = None,
) -> list[RunItem]:
    items = []
    for window in windows:
        checkpoint = (
            best_checkpoint_for(window, checkpoints_by_name.get(window.name, []))
            if fast
            else None
        )
        tail_frames = window.frames
        if checkpoint is not None:
            tail_frames = window.frames - checkpoint.frame
        if frame_limit is not None:
            tail_frames = min(tail_frames, frame_limit)
        items.append(RunItem(window=window, checkpoint=checkpoint, tail_frames=tail_frames))
    return items


def selected_windows(
    windows: list[OracleWindow],
    only: list[str],
    max_frames: int | None,
    include_sram_windows: bool,
    limit: int | None = None,
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
    if limit is not None:
        selected = selected[:limit]
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
    frames: int | None = None,
    load_state: str | None = None,
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
            str(frames if frames is not None else window.frames),
            "--stride",
            str(stride),
        ]
    )
    if renderer == "assets-variant-gpu":
        command.extend(["--modern-index-compare", str(stride)])
    if window.input_script:
        command.extend(["--input-script", window.input_script])
        sidecar = sram_sidecar(window)
        if load_state is None and sidecar is not None and sidecar.exists():
            command.extend(["--load-sram", str(sidecar)])
    if load_state is not None:
        command.extend(["--load-state", load_state])
    return command


def command_for_run_item(
    item: RunItem,
    rom: Path,
    stride: int,
    release: bool,
    renderer: str | None,
) -> list[str]:
    load_state = (
        item.checkpoint.checkpoint_path if item.checkpoint is not None else None
    )
    return command_for(
        item.window,
        rom,
        stride,
        release,
        renderer,
        frames=item.tail_frames,
        load_state=load_state,
    )


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


def ensure_required_stable_draws(
    stable_preview_draws: int,
    stable_effect_draws: int,
) -> None:
    if stable_preview_draws + stable_effect_draws == 0:
        raise SystemExit(
            "required stable source-art/effect draws, but selected windows drew zero"
        )


def run_window(
    window: OracleWindow,
    rom: Path,
    stride: int,
    release: bool,
    renderer: str | None,
    progress_every: int,
    checkpoint: OracleCheckpoint | None,
    tail_frames: int | None = None,
) -> tuple[int, str, int, tuple[int, int, int, int]]:
    run_frames = window.frames
    load_state = None
    if checkpoint is not None:
        run_frames = window.frames - checkpoint.frame
        load_state = checkpoint.checkpoint_path
    if tail_frames is not None:
        run_frames = tail_frames
    command = command_for(
        window,
        rom,
        stride,
        release,
        renderer,
        frames=run_frames,
        load_state=load_state,
    )
    prefix = f"ZELDA3_RENDERER={renderer} " if renderer else ""
    if checkpoint is None:
        print(f"running {window.name}: {prefix}{' '.join(command)}", flush=True)
    else:
        print(
            f"running {window.name} from checkpoint frame {checkpoint.frame} "
            f"({run_frames} tail frame(s)): {prefix}{' '.join(command)}",
            flush=True,
        )
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
    start_frame = int(match.group(2))
    last_hash = match.group(4)
    mismatched_pixels = int(match.group(5))
    if checkpoint is not None and start_frame != checkpoint.frame:
        raise SystemExit(
            f"{window.name}: checkpoint start mismatch: expected {checkpoint.frame}, got {start_frame}"
        )
    if mismatched_pixels != 0:
        raise SystemExit(f"{window.name}: reported {mismatched_pixels} mismatched pixels")
    variant_stats = (0, 0, 0, 0, 0, 0, 0, 0, 0)
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
            int(modern_match.group(11) or 0),
            int(modern_match.group(12) or 0),
            int(modern_match.group(13) or 0),
            int(modern_match.group(14) or 0),
            int(modern_match.group(15) or 0),
        )
    print(
        f"{window.name}: compared={compared} frames={window.frames} "
        f"start_frame={start_frame} "
        f"last_hash={last_hash} mismatched_pixels=0 "
        f"variant_draws={variant_stats[0]} "
        f"fallback_draws={variant_stats[1]} "
        f"dynamic_palette_draws={variant_stats[2]} "
        f"missing_variant_draws={variant_stats[3]} "
        f"stable_preview_draws={variant_stats[4]} "
        f"stable_effect_draws={variant_stats[5]} "
        f"dynamic_material_draws={variant_stats[6]} "
        f"missing_art_draws={variant_stats[7]} "
        f"unkeyed_fallback_draws={variant_stats[8]}"
    )
    return compared, last_hash, mismatched_pixels, variant_stats


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--windows", type=Path, default=DEFAULT_WINDOWS)
    parser.add_argument("--checkpoints", type=Path, default=DEFAULT_CHECKPOINTS)
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
    parser.add_argument(
        "--limit",
        type=int,
        help="run only the first N selected passing windows after filters",
    )
    parser.add_argument(
        "--frames",
        type=int,
        help="cap each selected run to N frames after frame 0 or the selected checkpoint",
    )
    parser.add_argument(
        "--require-stable-draws",
        action="store_true",
        help=(
            "fail unless selected assets-variant-gpu windows draw at least one "
            "stable preview/effect-backed source-art tile"
        ),
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--jobs",
        type=int,
        default=1,
        help="run up to N selected windows in parallel; default 1",
    )
    parser.set_defaults(fast=True)
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
        help="run each GPU oracle window from frame 0 instead of resuming from checkpoints",
    )
    args = parser.parse_args()
    if args.stride <= 0:
        raise SystemExit("--stride must be greater than zero")
    if args.max_frames is not None and args.max_frames <= 0:
        raise SystemExit("--max-frames must be greater than zero")
    if args.progress_every < 0:
        raise SystemExit("--progress-every must be zero or greater")
    if args.limit is not None and args.limit <= 0:
        raise SystemExit("--limit must be greater than zero")
    if args.frames is not None and args.frames <= 0:
        raise SystemExit("--frames must be greater than zero")
    if args.require_stable_draws and args.renderer != "assets-variant-gpu":
        raise SystemExit("--require-stable-draws requires --renderer assets-variant-gpu")
    if args.jobs <= 0:
        raise SystemExit("--jobs must be greater than zero")
    if not args.rom.exists():
        raise SystemExit(f"ROM does not exist: {args.rom}")
    if not args.windows.exists():
        raise SystemExit(f"window table does not exist: {args.windows}")
    if not args.checkpoints.exists():
        raise SystemExit(f"checkpoint table does not exist: {args.checkpoints}")
    return args


def main() -> None:
    args = parse_args()
    checkpoints_by_name = group_checkpoints(load_checkpoints(args.checkpoints))
    windows = selected_windows(
        load_windows(args.windows),
        args.only,
        args.max_frames,
        args.include_sram_windows,
        args.limit,
    )
    if not windows:
        raise SystemExit("no windows selected")
    run_items = run_items_for_windows(
        windows,
        checkpoints_by_name,
        args.fast,
        args.frames,
    )

    total_compared = 0
    total_variant_draws = 0
    total_fallback_draws = 0
    total_dynamic_palette_draws = 0
    total_missing_variant_draws = 0
    total_stable_preview_draws = 0
    total_stable_effect_draws = 0
    total_dynamic_material_draws = 0
    total_missing_art_draws = 0
    total_unkeyed_fallback_draws = 0
    if args.dry_run:
        for item in run_items:
            prefix = f"ZELDA3_RENDERER={args.renderer} " if args.renderer else ""
            print(
                prefix
                + " ".join(
                    command_for_run_item(
                        item,
                        args.rom,
                        args.stride,
                        args.release,
                        args.renderer,
                    )
                )
            )
        return

    def run_item(item: RunItem) -> tuple[int, str, int, tuple[int, ...]]:
        return run_window(
            item.window,
            args.rom,
            args.stride,
            args.release,
            args.renderer,
            args.progress_every,
            item.checkpoint,
            item.tail_frames,
        )

    if args.jobs == 1:
        results = [run_item(item) for item in run_items]
    else:
        results = []
        with ThreadPoolExecutor(max_workers=args.jobs) as executor:
            futures = [executor.submit(run_item, item) for item in run_items]
            for future in as_completed(futures):
                results.append(future.result())

    for compared, _, _, variant_stats in results:
        total_compared += compared
        total_variant_draws += variant_stats[0]
        total_fallback_draws += variant_stats[1]
        total_dynamic_palette_draws += variant_stats[2]
        total_missing_variant_draws += variant_stats[3]
        total_stable_preview_draws += variant_stats[4]
        total_stable_effect_draws += variant_stats[5]
        total_dynamic_material_draws += variant_stats[6]
        total_missing_art_draws += variant_stats[7]
        total_unkeyed_fallback_draws += variant_stats[8]

    if args.require_stable_draws:
        ensure_required_stable_draws(
            stable_preview_draws=total_stable_preview_draws,
            stable_effect_draws=total_stable_effect_draws,
        )

    print(
        "gpu-render-oracle-windows completed "
        f"windows={len(windows)} compared={total_compared} stride={args.stride} "
        "mismatched_pixels=0 "
        f"variant_draws={total_variant_draws} "
        f"fallback_draws={total_fallback_draws} "
        f"dynamic_palette_draws={total_dynamic_palette_draws} "
        f"missing_variant_draws={total_missing_variant_draws} "
        f"stable_preview_draws={total_stable_preview_draws} "
        f"stable_effect_draws={total_stable_effect_draws} "
        f"dynamic_material_draws={total_dynamic_material_draws} "
        f"missing_art_draws={total_missing_art_draws} "
        f"unkeyed_fallback_draws={total_unkeyed_fallback_draws}"
    )


if __name__ == "__main__":
    main()
