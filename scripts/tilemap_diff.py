#!/usr/bin/env python3
"""Visualize a BG tilemap divergence old-vs-new as a grid + decoded tile diff.

A handful of scattered tilemap addresses (e.g. the 19 staircase bytes) hides the shape
of the bug. This runs both binaries for one frame, reads the BG tilemap region, and
prints a grid (matching=., differing=#) plus, for each differing tile, the decoded SNES
tile word for OLD and NEW: index / palette / priority / h,v-flip. That turns
"0x4650 0x46d0 0x482c ..." into "the staircase rails are palette-3 in old vs palette-2+
priority in new", which names the object and points at the RoomDraw routine.

    scripts/tilemap_diff.py 37485                 # default DUNG_BG1 (0x4000, 64x64)
    scripts/tilemap_diff.py 37485 --base 0x2000   # DUNG_BG2
    scripts/tilemap_diff.py 37485 --cols 64 --rows 64

SNES tilemap word (little-endian, 16-bit): v h o pp p t t t t t t t t t t
  bit15 v-flip, bit14 h-flip, bit13 priority, bits12-10 palette, bits9-0 tile index.
"""
from __future__ import annotations
import os, subprocess, sys, pathlib, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
NEW = ROOT / "target" / "parity" / "zelda3"
OLD = pathlib.Path(
    os.environ.get("ZELDA3_OLD_REPO", str(ROOT.parent / "zelda3-rs-old"))
) / "target" / "release" / "zelda3"
ROM = pathlib.Path(os.environ.get("ZELDA3_ROM", str(ROOT / "saves" / "zelda3.sfc")))
SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ["SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING",
          "DEATH_INTRO", "DEATH_RELOAD"]}


def dump(binary: pathlib.Path, frame: int) -> bytes:
    tmp = pathlib.Path(tempfile.gettempdir()) / f"tm_{binary.parent.parent.name}.bin"
    env = os.environ.copy(); env.update(HACKS); env["ZELDA3_REPLAY_WRAM_DUMP"] = str(tmp)
    subprocess.run([str(binary), "--replay-save", str(ROM), str(SAVE), str(frame)],
                   cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                   stderr=subprocess.DEVNULL, check=False)
    return tmp.read_bytes()


def decode(word: int) -> str:
    return (f"t=0x{word & 0x3ff:03x} pal={(word >> 10) & 7} "
            f"pri={(word >> 13) & 1} h={(word >> 14) & 1} v={(word >> 15) & 1}")


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__); return 2
    frame = int(args[0], 0)
    base = int(args[args.index("--base") + 1], 0) if "--base" in args else 0x4000
    cols = int(args[args.index("--cols") + 1], 0) if "--cols" in args else 64
    rows = int(args[args.index("--rows") + 1], 0) if "--rows" in args else 64

    n, o = dump(NEW, frame), dump(OLD, frame)

    def word(buf, r, c):
        a = base + (r * cols + c) * 2
        return buf[a] | (buf[a + 1] << 8) if a + 1 < len(buf) else 0

    diffs = [(r, c) for r in range(rows) for c in range(cols)
             if word(n, r, c) != word(o, r, c)]
    if not diffs:
        print(f"frame {frame}: BG@0x{base:05x} matches (no tilemap divergence)"); return 0

    r0 = min(r for r, _ in diffs); r1 = max(r for r, _ in diffs)
    c0 = min(c for _, c in diffs); c1 = max(c for _, c in diffs)
    dset = set(diffs)
    print(f"frame {frame}: BG@0x{base:05x}  {len(diffs)} differing tiles, "
          f"bbox rows {r0}-{r1} cols {c0}-{c1}\n")
    # column header (tens)
    print("      " + "".join(str((c // 10) % 10) for c in range(c0, c1 + 1)))
    print("      " + "".join(str(c % 10) for c in range(c0, c1 + 1)))
    for r in range(r0, r1 + 1):
        row = "".join("#" if (r, c) in dset else "." for c in range(c0, c1 + 1))
        print(f"  r{r:2}: {row}")
    print("\n  differing tiles (addr: OLD | NEW):")
    for r, c in diffs:
        a = base + (r * cols + c) * 2
        print(f"   0x{a:05x} r{r:2}c{c:2}: {decode(word(o, r, c))}  |  {decode(word(n, r, c))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
