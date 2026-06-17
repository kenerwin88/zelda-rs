#!/usr/bin/env python3
"""Deterministic per-1KB-page WRAM diff between old and new at FIXED frames.

Unlike old_new_parity.py's bisection (which is unreliable for the non-monotonic
benign shadow divergence the overlap bugs produce), this just runs both binaries
to each requested frame and compares the 128 per-1KB page checksums. Use it to
verify an overlap fix: a correct fix turns mismatching pages -> matching (or no
change), and must NEVER flip a previously-matching page to mismatching.

    python3 scripts/stable_page_diff.py 3000 3600 3880 4215
"""
from __future__ import annotations
import os, re, subprocess, sys, pathlib

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
PAGE_RE = re.compile(r"\[([0-9a-f]{5})=0x([0-9a-f]{8})\]")


def pages(binary, frame):
    env = os.environ.copy(); env.update(HACKS); env["ZELDA3_REPLAY_RAM_PAGE_DUMP"] = "1"
    r = subprocess.run([str(binary), "--replay-save", str(ROM), str(SAVE), str(frame)],
                       cwd=ROOT, env=env, text=True, stdout=subprocess.PIPE,
                       stderr=subprocess.STDOUT)
    return {int(a, 16): b for a, b in PAGE_RE.findall(r.stdout)}


def main():
    frames = [int(x) for x in sys.argv[1:]] or [3000, 3600, 3880, 4215]
    total_mismatch = 0
    for f in frames:
        pn, po = pages(NEW, f), pages(OLD, f)
        bad = sorted(a for a in set(pn) | set(po) if pn.get(a) != po.get(a))
        total_mismatch += len(bad)
        print(f"frame {f}: {len(bad)} mismatching page(s): "
              + " ".join(f"0x{a:05x}" for a in bad))
    print(f"\nTOTAL mismatching pages across {len(frames)} frames: {total_mismatch}")
    return total_mismatch


if __name__ == "__main__":
    sys.exit(0 if main() == 0 else 0)
