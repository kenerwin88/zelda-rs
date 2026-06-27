#!/usr/bin/env python3
"""Route-based modern-vs-classic render parity gate.

Runs the harness in ``--play-gpu-render-compare`` mode for the first N frames
of the combined replay route and gates on the per-frame hash comparison between
the classic (old) renderer and the modern GPU renderer.

Expected state: RED (mismatched > 0) until Task 11 drives the modern renderer
to full coverage. Early intro frames are forced-blank and always match=true;
gameplay frames diverge once the modern atlas is incomplete.

Exit codes:
  0 - all compared frames matched (gate GREEN)
  1 - mismatch found, OR no compare lines emitted (wiring broken)
  2 - harness returned a nonzero exit code
"""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_ROM = ROOT / "saves" / "zelda3.sfc"

COMPARE_RE = re.compile(
    r"^modern_render_compare\s+frame=(\d+)\s+old=0x([0-9a-fA-F]+)\s+modern=0x([0-9a-fA-F]+)\s+match=(true|false)$"
)


def ensure_binary(profile: str) -> pathlib.Path:
    """Return the harness binary path, building it if necessary."""
    binary = ROOT / "target" / profile / "zelda3"
    if binary.exists():
        return binary
    print(f"binary not found at {binary}; building with --profile {profile} ...", flush=True)
    cmd = [
        "cargo",
        "build",
        "-p",
        "zelda3-bin",
        "--profile",
        profile,
    ]
    print("+ " + " ".join(cmd), flush=True)
    rc = subprocess.run(cmd, cwd=ROOT, check=False).returncode
    if rc != 0:
        print(f"cargo build failed (exit {rc})", file=sys.stderr)
        raise SystemExit(rc)
    return binary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rom",
        type=pathlib.Path,
        default=DEFAULT_ROM,
        help="path to the Zelda 3 ROM (default: saves/zelda3.sfc)",
    )
    parser.add_argument(
        "--frames",
        type=int,
        default=300,
        help="number of frames to replay (default: 300)",
    )
    parser.add_argument(
        "--stride",
        type=int,
        default=30,
        help="compare every Nth frame (default: 30)",
    )
    parser.add_argument(
        "--profile",
        default="parity",
        help="cargo profile for the harness binary (default: parity)",
    )
    args = parser.parse_args()

    if args.frames <= 0:
        parser.error("--frames must be positive")
    if args.stride <= 0:
        parser.error("--stride must be positive")

    rom = args.rom.expanduser().resolve()
    if not rom.exists():
        print(f"ROM not found: {rom}", file=sys.stderr)
        return 2

    binary = ensure_binary(args.profile)

    env = os.environ.copy()
    env.update(
        {
            "SDL_VIDEODRIVER": "dummy",
            "SDL_AUDIODRIVER": "dummy",
            "SDL_RENDER_DRIVER": "software",
        }
    )

    cmd = [
        str(binary),
        "--play-gpu-render-compare",
        str(rom),
        str(args.frames),
        "--stride",
        str(args.stride),
        "--modern-render-compare",
        str(args.stride),
    ]
    print("+ " + " ".join(cmd), flush=True)

    result = subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )

    output = result.stdout or ""
    if output:
        print(output, end="" if output.endswith("\n") else "\n")

    if result.returncode != 0:
        print(
            f"harness exited with code {result.returncode}",
            file=sys.stderr,
        )
        return result.returncode

    compared = 0
    matched = 0
    mismatched = 0
    first_mismatch_frame: int | None = None

    for line in output.splitlines():
        m = COMPARE_RE.match(line)
        if not m:
            continue
        frame = int(m.group(1))
        is_match = m.group(4) == "true"
        compared += 1
        if is_match:
            matched += 1
        else:
            mismatched += 1
            if first_mismatch_frame is None:
                first_mismatch_frame = frame

    if compared == 0:
        print(
            "error: no modern_render_compare lines found — "
            "is --modern-render-compare wired up in the harness?",
            file=sys.stderr,
        )
        return 1

    if mismatched > 0:
        summary = (
            f"modern parity: compared={compared} matched={matched} mismatched={mismatched}"
            f" (first mismatch frame={first_mismatch_frame})"
        )
    else:
        summary = f"modern parity: compared={compared} matched={matched} mismatched={mismatched}"
    print(summary)

    return 0 if mismatched == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
