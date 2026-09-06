#!/usr/bin/env python3
"""Small curses dashboard for a running Snes9x pre-commit parity gate.

Reads only what the gate already writes: the precommit session directories
under routes/full_run/comparisons/precommit (one line of av_hashes.jsonl per
compared frame), the gate state file, the disk, and an optional gate log.

    python3 scripts/parity_gate_tui.py [--log PATH] [--target FRAMES]

Keys: q quits.
"""

from __future__ import annotations

import argparse
import curses
import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRECOMMIT = ROOT / "routes" / "full_run" / "comparisons" / "precommit"
STATE_PATH = ROOT / ".git" / "precommit-snes9x-parity-state.json"
LOG_TAIL_LINES = 10
STAGES = ("rng-calibration", "video-preflight", "exact")
STAGE_TITLES = {
    "rng-calibration": "1. renderless live-RNG calibration",
    "video-preflight": "2. recorded-RNG video preflight",
    "exact": "3. cold exact A/V certification",
}


def count_lines(path: Path, cache: dict) -> int:
    """Count newline-terminated lines, re-reading only appended bytes."""
    try:
        size = path.stat().st_size
    except OSError:
        return 0
    known_size, known_lines = cache.get(path, (0, 0))
    if size < known_size:
        known_size, known_lines = 0, 0
    if size == known_size:
        return known_lines
    lines = known_lines
    with path.open("rb") as handle:
        handle.seek(known_size)
        remaining = size - known_size
        while remaining > 0:
            chunk = handle.read(min(remaining, 1 << 20))
            if not chunk:
                break
            remaining -= len(chunk)
            lines += chunk.count(b"\n")
    cache[path] = (size, lines)
    return lines


def newest_invocation(target: int) -> str | None:
    """Invocation id of the newest session directory for `target` frames."""
    best: tuple[float, str] | None = None
    pattern = re.compile(rf"run-{target}-(?:{'|'.join(STAGES)})-([0-9a-z_]+)$")
    for entry in PRECOMMIT.glob(f"run-{target}-*"):
        match = pattern.match(entry.name)
        if not match:
            continue
        mtime = entry.stat().st_mtime
        if best is None or mtime > best[0]:
            best = (mtime, match.group(1))
    return best[1] if best else None


def human_bytes(value: float) -> str:
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if abs(value) < 1024:
            return f"{value:.1f} {unit}"
        value /= 1024
    return f"{value:.1f} PiB"


def human_duration(seconds: float | None) -> str:
    if seconds is None or seconds < 0 or seconds != seconds:
        return "--"
    seconds = int(seconds)
    hours, rest = divmod(seconds, 3600)
    minutes, secs = divmod(rest, 60)
    if hours:
        return f"{hours}h{minutes:02d}m"
    if minutes:
        return f"{minutes}m{secs:02d}s"
    return f"{secs}s"


def gate_processes() -> list[str]:
    try:
        out = subprocess.run(
            ["ps", "-axo", "pid,etime,%cpu,rss,command"],
            capture_output=True,
            text=True,
            check=False,
        ).stdout
    except OSError:
        return []
    rows = []
    for line in out.splitlines()[1:]:
        if "precommit_snes9x_parity_gate.py" in line or "--compare-snes9x-oracle" in line:
            if "parity_gate_tui" in line:
                continue
            fields = line.split(None, 4)
            if len(fields) == 5:
                pid, etime, cpu, rss, command = fields
                name = "gate.py" if "precommit_snes9x" in command else "zelda3 compare"
                rows.append(f"{name:<15} pid {pid:>6}  up {etime:>11}  cpu {cpu:>5}%  rss {human_bytes(int(rss) * 1024)}")
    return rows


class Tracker:
    def __init__(self, target: int, log: Path | None):
        self.target = target
        self.log = log
        self.line_cache: dict = {}
        self.history: dict[str, list[tuple[float, int]]] = {}
        self.started = time.time()

    def stage_progress(self, invocation: str | None):
        result = []
        for stage in STAGES:
            info = {"stage": stage, "frames": 0, "status": "pending", "dir": None, "rate": None, "eta": None, "trace": 0}
            if invocation is None:
                result.append(info)
                continue
            directory = PRECOMMIT / f"run-{self.target}-{stage}-{invocation}"
            if not directory.is_dir():
                result.append(info)
                continue
            info["dir"] = directory
            frames = count_lines(directory / "av_hashes.jsonl", self.line_cache)
            info["frames"] = frames
            result_path = directory / "result.json"
            if result_path.exists():
                try:
                    status = json.loads(result_path.read_text()).get("status", "?")
                except (OSError, ValueError):
                    status = "?"
                info["status"] = status
            elif frames > 0:
                info["status"] = "running"
            else:
                info["status"] = "starting" if any(directory.iterdir()) else "reserved"
            trace = directory / "oracle-rom-random.jsonl"
            if trace.exists():
                info["trace"] = trace.stat().st_size
            history = self.history.setdefault(stage, [])
            now = time.time()
            history.append((now, frames))
            del history[:-60]
            if len(history) >= 2 and history[-1][1] > history[0][1]:
                dt = history[-1][0] - history[0][0]
                df = history[-1][1] - history[0][1]
                if dt > 0:
                    rate = df / dt
                    info["rate"] = rate
                    info["eta"] = (self.target - frames) / rate if rate > 0 else None
            result.append(info)
        return result

    def log_tail(self, lines: int) -> list[str]:
        if self.log is None or not self.log.exists():
            return []
        try:
            data = self.log.read_bytes()[-65536:].decode("utf-8", "replace")
        except OSError:
            return []
        return [line.rstrip() for line in data.splitlines() if line.strip()][-lines:]


def draw(stdscr, tracker: Tracker):
    curses.curs_set(0)
    stdscr.nodelay(True)
    stdscr.timeout(2000)
    while True:
        key = stdscr.getch()
        if key in (ord("q"), ord("Q")):
            return
        stdscr.erase()
        height, width = stdscr.getmaxyx()
        row = 0

        def put(text: str, attr=0):
            nonlocal row
            if row < height - 1:
                stdscr.addnstr(row, 0, text, width - 1, attr)
                row += 1

        invocation = newest_invocation(tracker.target)
        procs = gate_processes()
        alive = bool(procs)
        put(f" zelda3 Snes9x parity gate  target {tracker.target:,} frames  invocation {invocation or '--'}  {'RUNNING' if alive else 'NO GATE PROCESS'}", curses.A_BOLD)
        put(f" {time.strftime('%Y-%m-%d %H:%M:%S')}   watching for {human_duration(time.time() - tracker.started)}")
        put("")
        stages = tracker.stage_progress(invocation)
        bar_width = max(10, min(60, width - 60))
        for info in stages:
            frames = info["frames"]
            fraction = min(1.0, frames / tracker.target) if tracker.target else 0
            filled = int(fraction * bar_width)
            bar = "#" * filled + "-" * (bar_width - filled)
            status = info["status"]
            attr = curses.A_BOLD if status == "running" else 0
            if status == "passed":
                attr = curses.A_DIM
            put(f" {STAGE_TITLES[info['stage']]:<40} {status:<9}", attr)
            put(f"   [{bar}] {fraction * 100:5.1f}%  {frames:>9,}/{tracker.target:,}")
            details = []
            if info["rate"]:
                details.append(f"{info['rate']:.0f} frames/s")
            if info["eta"] is not None and status == "running":
                details.append(f"ETA {human_duration(info['eta'])}")
            if info["trace"]:
                per_frame = info["trace"] / frames if frames else 0
                projected = per_frame * tracker.target
                details.append(f"trace {human_bytes(info['trace'])} (~{human_bytes(projected)} at full route)")
            if details:
                put("   " + "   ".join(details))
            put("")
        usage = shutil.disk_usage(str(ROOT))
        put(f" disk free {human_bytes(usage.free)} of {human_bytes(usage.total)}", curses.A_BOLD if usage.free < 20 << 30 else 0)
        try:
            state = json.loads(STATE_PATH.read_text())
            put(f" ratchet last_checked {state.get('last_checked_frame', 0):,} / {state.get('last_checked_total_frames', 0):,}   receipt {Path(str(state.get('last_cold_receipt_path', ''))).name}")
        except (OSError, ValueError):
            put(" ratchet state unavailable")
        put("")
        put(" processes", curses.A_UNDERLINE)
        for line in procs or ["   (none)"]:
            put("   " + line)
        put("")
        remaining = height - row - 2
        if remaining > 2 and tracker.log is not None:
            put(f" gate log tail (last {LOG_TAIL_LINES})  {tracker.log}", curses.A_UNDERLINE)
            for line in tracker.log_tail(min(LOG_TAIL_LINES, remaining - 1)):
                put("   " + line)
        stdscr.addnstr(height - 1, 0, " q quit   refresh 2s ", width - 1, curses.A_REVERSE)
        stdscr.refresh()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--log", type=Path, default=None, help="gate log file to tail")
    parser.add_argument("--target", type=int, default=None, help="target frame count (default: from the gate state file)")
    args = parser.parse_args()
    target = args.target
    if target is None:
        try:
            target = int(json.loads(STATE_PATH.read_text()).get("last_checked_total_frames", 0))
        except (OSError, ValueError):
            target = 0
        newest = sorted(PRECOMMIT.glob("run-*-exact-*"), key=lambda p: p.stat().st_mtime)
        if newest:
            match = re.match(r"run-(\d+)-", newest[-1].name)
            if match:
                target = int(match.group(1))
    if not target:
        parser.error("could not infer the target frame count; pass --target")
    tracker = Tracker(target, args.log)
    curses.wrapper(draw, tracker)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
