#!/usr/bin/env python3
"""Compare a run's WRAM dumps against the committed goldens (Rust-vs-Rust).

The goldens under `routes/full_run/golden/wram/frame-<N>.bin` are the 128 KiB
WRAM images the promoted binary produced at those route frames during a
full-route exact cached-av pass. Any later binary replaying the same route must
reproduce them byte for byte unless a change is intentional (in which case
re-bless with `--bless`).

Produce the dumps with the cached-av lane, e.g.

    ZELDA3_DEBUG_WRAM_FRAMES=60000,150470,500000,732000 \\
        ./parity cached-av <cache> --frames 732001

then

    scripts/wram_golden_check.py <run-dir>              # compare
    scripts/wram_golden_check.py <run-dir> --bless      # replace the goldens

No Snes9x is needed; the check takes milliseconds.
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GOLDEN = ROOT / "routes" / "full_run" / "golden" / "wram"
DUMP_PREFIX = "rust_wram_frame_"


def ranges(indexes: list[int]) -> list[tuple[int, int]]:
    out: list[tuple[int, int]] = []
    for index in indexes:
        if out and index <= out[-1][1] + 1:
            out[-1] = (out[-1][0], index)
        else:
            out.append((index, index))
    return out


def compare(run_dir: Path, limit: int) -> int:
    goldens = sorted(GOLDEN.glob("frame-*.bin"))
    if not goldens:
        print(f"no goldens under {GOLDEN}", file=sys.stderr)
        return 2
    problems = 0
    checked = 0
    for golden in goldens:
        frame = int(golden.stem.split("-", 1)[1])
        dump = run_dir / f"{DUMP_PREFIX}{frame}.bin"
        if not dump.is_file():
            print(f"frame {frame}: no dump in {run_dir} (skipped)")
            continue
        checked += 1
        expected = golden.read_bytes()
        actual = dump.read_bytes()
        if expected == actual:
            print(f"frame {frame}: MATCH")
            continue
        problems += 1
        if len(expected) != len(actual):
            print(f"frame {frame}: SIZE {len(actual)} vs golden {len(expected)}")
            continue
        diffs = [i for i in range(len(expected)) if expected[i] != actual[i]]
        print(f"frame {frame}: {len(diffs)} differing bytes in {len(ranges(diffs))} ranges")
        for lo, hi in ranges(diffs)[:limit]:
            print(
                f"  0x{lo:05x}-0x{hi:05x} golden={expected[lo:hi + 1][:8].hex()}"
                f" run={actual[lo:hi + 1][:8].hex()}"
            )
    if checked == 0:
        print("no dumps matched any golden frame", file=sys.stderr)
        return 2
    return 1 if problems else 0


def bless(run_dir: Path) -> int:
    dumps = sorted(run_dir.glob(f"{DUMP_PREFIX}*.bin"))
    if not dumps:
        print(f"no {DUMP_PREFIX}*.bin dumps in {run_dir}", file=sys.stderr)
        return 2
    GOLDEN.mkdir(parents=True, exist_ok=True)
    for dump in dumps:
        frame = int(dump.stem[len(DUMP_PREFIX):])
        target = GOLDEN / f"frame-{frame}.bin"
        shutil.copyfile(dump, target)
        print(f"blessed {target.relative_to(ROOT)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("run_dir", type=Path)
    parser.add_argument("--bless", action="store_true", help="replace the goldens with this run's dumps")
    parser.add_argument("--limit", type=int, default=40, help="max differing ranges to print per frame")
    args = parser.parse_args()
    if args.bless:
        return bless(args.run_dir)
    return compare(args.run_dir, args.limit)


if __name__ == "__main__":
    raise SystemExit(main())
