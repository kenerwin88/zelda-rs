#!/usr/bin/env python3
"""Whole-replay divergence map: every (frame, page) that ever diverges, in two runs.

Per-frame probing (stable_page_diff / first_diverging_frame) re-replays from the start
for each frame you check, so finding ALL remaining divergences across ~170k frames is
prohibitively slow. This instead runs each binary ONCE to an end frame with
`ZELDA3_REPLAY_FRAME_PAGE_DUMP=<path>`, which streams a fixed record per frame:

    [frame:u32 LE][128 × page_fnv32 LE]   (one FNV-1a checksum per 1KB WRAM page)

then diffs the two streams frame-by-frame and lists every (frame, page) whose checksum
differs. That is the complete divergence map — the input to the parity loop — for the cost
of two replays.

    scripts/full_divergence_scan.py 170000              # run both, full map
    scripts/full_divergence_scan.py 170000 --page 0x0000 # only this page's diverging frames
    scripts/full_divergence_scan.py 170000 --reuse       # reuse existing dumps, don't re-run
    scripts/full_divergence_scan.py 170000 --first       # just the first diverging frame per page

Pages are 1KB; page P covers WRAM 0x{P*0x400:05x}. Workflow: this finds the (frame, page);
then stable_page_diff.py <frame> pins the bytes, step_diff.py <frame> <addr> pins the step.
"""
from __future__ import annotations
import os, struct, subprocess, sys, pathlib, tempfile

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
RECORD = 4 + 128 * 4  # frame u32 + 128 page checksums


def run(binary: pathlib.Path, frame: int, out: pathlib.Path) -> None:
    env = os.environ.copy(); env.update(HACKS)
    env["ZELDA3_REPLAY_FRAME_PAGE_DUMP"] = str(out)
    subprocess.run([str(binary), "--replay-save", str(ROM), str(SAVE), str(frame)],
                   cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                   stderr=subprocess.DEVNULL, check=False)


def load(path: pathlib.Path) -> dict[int, tuple]:
    data = path.read_bytes()
    out = {}
    for off in range(0, len(data) - RECORD + 1, RECORD):
        frame = struct.unpack_from("<I", data, off)[0]
        sums = struct.unpack_from("<128I", data, off + 4)
        out[frame] = sums
    return out


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__); return 2
    end = int(args[0], 0)
    page_filter = int(args[args.index("--page") + 1], 0) >> 10 if "--page" in args else None
    reuse = "--reuse" in args
    first_only = "--first" in args

    nbin = pathlib.Path(tempfile.gettempdir()) / "fdiv_new.bin"
    obin = pathlib.Path(tempfile.gettempdir()) / "fdiv_old.bin"
    if not reuse:
        print(f"running NEW to frame {end} ...", flush=True); run(NEW, end, nbin)
        print(f"running OLD to frame {end} ...", flush=True); run(OLD, end, obin)
    if not nbin.exists() or not obin.exists():
        print("error: dumps missing — rebuild both binaries with the "
              "ZELDA3_REPLAY_FRAME_PAGE_DUMP hook", file=sys.stderr)
        return 2

    new, old = load(nbin), load(obin)
    frames = sorted(set(new) & set(old))
    print(f"compared {len(frames)} frames (new={len(new)} old={len(old)})\n")

    # (page -> sorted list of diverging frames)
    by_page: dict[int, list[int]] = {}
    for f in frames:
        ns, os_ = new[f], old[f]
        for p in range(128):
            if page_filter is not None and p != page_filter:
                continue
            if ns[p] != os_[p]:
                by_page.setdefault(p, []).append(f)

    if not by_page:
        print("NO DIVERGENCE — every page matches at every compared frame. PERFECT PARITY.")
        return 0

    total_pairs = sum(len(v) for v in by_page.values())
    print(f"DIVERGENCES: {len(by_page)} page(s), {total_pairs} (frame,page) pairs\n")
    for p in sorted(by_page):
        fr = by_page[p]
        addr = p * 0x400
        if first_only:
            print(f"  page 0x{addr:05x}: first@{fr[0]}  ({len(fr)} frames)")
        else:
            # compress consecutive runs for readability
            runs = []
            s = e = fr[0]
            for x in fr[1:]:
                if x == e + 1:
                    e = x
                else:
                    runs.append((s, e)); s = e = x
            runs.append((s, e))
            shown = ", ".join(f"{a}" if a == b else f"{a}-{b}" for a, b in runs[:12])
            extra = "" if len(runs) <= 12 else f" … (+{len(runs)-12} more runs)"
            print(f"  page 0x{addr:05x}: {len(fr)} frames [{shown}{extra}]")
    return 0


if __name__ == "__main__":
    sys.exit(main())
