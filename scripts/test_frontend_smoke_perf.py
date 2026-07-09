#!/usr/bin/env python3
"""Opt-in live renderer performance gate for the PNG-driven GPU path."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


TIMING_RE = re.compile(r"(\w+)=([^\s]+)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frames", type=int, default=600)
    parser.add_argument("--max-bg-tiles", type=int, default=4096)
    parser.add_argument("--max-avg-extract-us", type=float, default=1800.0)
    parser.add_argument("--max-avg-render-us", type=float, default=1200.0)
    parser.add_argument("--max-render-us", type=float, default=6000.0)
    parser.add_argument(
        "--paced",
        action="store_true",
        help="leave frame pacing enabled; default disables pacing for a perf gate",
    )
    return parser.parse_args()


def timing_fields(line: str) -> dict[str, str]:
    return {key: value for key, value in TIMING_RE.findall(line)}


def main() -> int:
    args = parse_args()
    repo = Path(__file__).resolve().parents[1]
    cmd = [
        "cargo",
        "run",
        "-p",
        "zelda3-bin",
        "--quiet",
        "--",
        "--frontend-smoke",
        str(args.frames),
    ]
    if not args.paced:
        cmd.append("--no-frame-pacing")

    env = os.environ.copy()
    env["ZELDA3_RENDER_TIMINGS"] = "1"
    run = subprocess.run(
        cmd,
        cwd=repo,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if run.returncode != 0:
        sys.stdout.write(run.stdout)
        sys.stderr.write(run.stderr)
        return run.returncode

    frames = 0
    rendered = 0
    max_bg_tiles = 0
    total_extract_us = 0.0
    total_render_us = 0.0
    max_render_us = 0.0
    for line in run.stderr.splitlines():
        if "modern_live_timing" not in line:
            continue
        fields = timing_fields(line)
        if fields.get("rendered") != "true":
            continue
        frames += 1
        rendered += 1
        bg_tiles = int(fields.get("bg_tiles", "0"))
        extract_us = float(fields.get("extract_us", "0"))
        render_us = float(fields.get("render_us", "0"))
        max_bg_tiles = max(max_bg_tiles, bg_tiles)
        total_extract_us += extract_us
        total_render_us += render_us
        max_render_us = max(max_render_us, render_us)

    if frames == 0:
        sys.stderr.write("no rendered modern_live_timing lines captured\n")
        sys.stderr.write(run.stderr)
        return 1

    avg_extract_us = total_extract_us / frames
    avg_render_us = total_render_us / frames
    print(
        "frontend_smoke_perf "
        f"frames={frames} rendered={rendered} max_bg_tiles={max_bg_tiles} "
        f"avg_extract_us={avg_extract_us:.1f} avg_render_us={avg_render_us:.1f} "
        f"max_render_us={max_render_us:.1f}"
    )

    failures: list[str] = []
    if max_bg_tiles > args.max_bg_tiles:
        failures.append(f"max_bg_tiles {max_bg_tiles} > {args.max_bg_tiles}")
    if avg_extract_us > args.max_avg_extract_us:
        failures.append(f"avg_extract_us {avg_extract_us:.1f} > {args.max_avg_extract_us:.1f}")
    if avg_render_us > args.max_avg_render_us:
        failures.append(f"avg_render_us {avg_render_us:.1f} > {args.max_avg_render_us:.1f}")
    if max_render_us > args.max_render_us:
        failures.append(f"max_render_us {max_render_us:.1f} > {args.max_render_us:.1f}")
    if failures:
        for failure in failures:
            print(f"FAIL {failure}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
