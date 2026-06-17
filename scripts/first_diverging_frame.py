#!/usr/bin/env python3
"""Binary-search the first frame a specific WRAM address (or range) diverges
between the old (reference) and new builds.

Unlike old_new_parity.py's full-hash bisection (unreliable: benign scratch
flicker is non-monotonic, so it lands on arbitrary frames), this compares ONLY
the bytes you name. A single owned byte's divergence is effectively monotonic,
so the bisection is sound — and it pinpoints the exact frame a root-cause byte
first goes wrong, which is the most common manual hunt in this migration.

Requires both binaries to support ZELDA3_REPLAY_WRAM_DUMP (the new build and the
1183dee clone both do).

    python3 scripts/first_diverging_frame.py 0x1ea10               # 1 byte
    python3 scripts/first_diverging_frame.py 0x1ea10 --width 4     # 4 bytes
    python3 scripts/first_diverging_frame.py 0x0e2d --max 8000     # search bound
    python3 scripts/first_diverging_frame.py 0xc198-0xc19c         # explicit range
    python3 scripts/first_diverging_frame.py 0x1ea10 --linear      # no monotonic assumption
"""
from __future__ import annotations
import argparse, os, pathlib, subprocess, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
NEW = ROOT / "target" / "parity" / "zelda3"
OLD = pathlib.Path(
    os.environ.get("ZELDA3_OLD_REPO", str(ROOT.parent / "zelda3-rs-old"))
) / "target" / "release" / "zelda3"
# Reference only zelda3-rs (this repo) and zelda3-rs-old (perfect parity). The ROM lives
# in this repo (saves/, gitignored); never reach into the C repo. Override with ZELDA3_ROM.
ROM = pathlib.Path(os.environ.get("ZELDA3_ROM", str(ROOT / "saves" / "zelda3.sfc")))
SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ["SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING",
          "DEATH_INTRO", "DEATH_RELOAD"]}


def wram(binary, frame, tmp):
    env = os.environ.copy(); env.update(HACKS)
    env["ZELDA3_REPLAY_WRAM_DUMP"] = str(tmp)
    subprocess.run([str(binary), "--replay-save", str(ROM), str(SAVE), str(frame)],
                   cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                   stderr=subprocess.DEVNULL, check=False)
    return pathlib.Path(tmp).read_bytes()


def parse_range(tok):
    if "-" in tok:
        a, b = tok.split("-", 1)
        lo, hi = int(a, 0), int(b, 0)
        return lo, hi - lo
    return int(tok, 0), None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("addr", help="address e.g. 0x1ea10 or range 0xc198-0xc19c")
    ap.add_argument("--width", type=int, default=1)
    ap.add_argument("--max", type=int, default=8000)
    ap.add_argument("--linear", action="store_true",
                    help="scan every frame (no monotonic assumption)")
    args = ap.parse_args()

    lo, span = parse_range(args.addr)
    width = span if span is not None else args.width
    hi = lo + width

    if not OLD.exists():
        sys.exit(f"old clone binary missing: {OLD}")

    td = tempfile.mkdtemp(prefix="fdf-")
    nt, ot = f"{td}/n.bin", f"{td}/o.bin"

    def differs(frame):
        n = wram(NEW, frame, nt)[lo:hi]
        o = wram(OLD, frame, ot)[lo:hi]
        return n != o, o, n

    def fmt(b):
        return "0x" + b.hex() if b else "(empty)"

    print(f"searching 0x{lo:05x}..0x{hi:05x} ({width} byte(s)) over frames 0..{args.max}")

    if args.linear:
        prev = None
        for f in range(0, args.max + 1):
            d, o, n = differs(f)
            if d:
                print(f"FIRST DIVERGENCE at frame {f}: old={fmt(o)} new={fmt(n)}")
                return 0
        print("no divergence found")
        return 0

    d_hi, o_hi, n_hi = differs(args.max)
    if not d_hi:
        print(f"frame {args.max}: still matches (old={fmt(o_hi)}) — no divergence in range")
        return 0
    d_lo, _, _ = differs(0)
    if d_lo:
        print("frame 0 already diverges (search lower or check setup)")
        return 0

    glo, ghi = 0, args.max  # glo matches, ghi diverges
    while ghi - glo > 1:
        mid = (glo + ghi) // 2
        d, _, _ = differs(mid)
        print(f"  probe frame {mid}: {'DIFF' if d else 'match'}")
        if d:
            ghi = mid
        else:
            glo = mid
    _, o, n = differs(ghi)
    _, o0, n0 = differs(glo)
    print(f"\nFIRST DIVERGENCE at frame {ghi} (last identical: {glo})")
    print(f"  frame {glo}: old={fmt(o0)} new={fmt(n0)}")
    print(f"  frame {ghi}: old={fmt(o)} new={fmt(n)}")
    print("(binary search assumes monotonic divergence; pass --linear to verify)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
