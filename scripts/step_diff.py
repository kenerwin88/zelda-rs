#!/usr/bin/env python3
"""Cross-binary per-step WRAM divergence localizer.

The dominant remaining parity bug class is "identical input, one WRAM byte ends up
different" (a room-object decode, a scratch register, a stale projection). A frame-end
WRAM dump tells you WHICH byte but not WHICH code wrote it. This does:

  Both this repo and zelda3-rs-old emit, under ZELDA3_REPLAY_STEP_DUMP=<frame>:<path>,
  a full 128KB WRAM snapshot at every labeled checkpoint (replay_trace_ram_watch(...))
  during the replay frame whose replay_frame_counter == <frame>. The labels/order match
  between the two binaries (same C-port checkpoints), so this script diffs the two
  "movies" checkpoint-by-checkpoint and reports the FIRST checkpoint after which a target
  address/page diverges, with old/new values.

That pins the divergence to one step == one function. Read that function; if it does
many writes, drop extra replay_trace_ram_watch("my-label") calls inside it and re-run to
bisect to the exact write. Keys on the same frame number used with scripts/replay.sh and
stable_page_diff.py (no frame_ctr_dbg/replay-count offset to reconcile).

    scripts/step_diff.py 38000 0x4650 0x43db     # track specific bytes
    scripts/step_diff.py 38000 --page 0x4000     # 1KB-page diff-count timeline
    scripts/step_diff.py 6557 0xc8               # the r16 scratch
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


def run_movie(binary: pathlib.Path, frame: int) -> list[tuple[str, bytes]]:
    """Run one binary for `frame` and parse its per-step WRAM movie."""
    tmp = pathlib.Path(tempfile.gettempdir()) / f"step_movie_{binary.parent.parent.name}.bin"
    if tmp.exists():
        tmp.unlink()
    env = os.environ.copy()
    env.update(HACKS)
    env["ZELDA3_REPLAY_STEP_DUMP"] = f"{frame}:{tmp}"
    subprocess.run([str(binary), "--replay-save", str(ROM), str(SAVE), str(frame)],
                   cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                   stderr=subprocess.DEVNULL, check=False)
    if not tmp.exists():
        return []
    data = tmp.read_bytes()
    records: list[tuple[str, bytes]] = []
    off = 0
    while off + 2 <= len(data):
        (llen,) = struct.unpack_from("<H", data, off); off += 2
        label = data[off:off + llen].decode("ascii", "replace"); off += llen
        (wlen,) = struct.unpack_from("<I", data, off); off += 4
        wram = data[off:off + wlen]; off += wlen
        records.append((label, wram))
    return records


def parse_target(tok: str) -> int:
    return int(tok, 0)


def main() -> int:
    args = sys.argv[1:]
    if not args:
        print(__doc__); return 2
    frame = int(args[0], 0)
    page_mode = "--page" in args
    targets = [parse_target(a) for a in args[1:] if not a.startswith("--")]
    if not targets:
        print("error: give at least one target address (e.g. 0x4650)", file=sys.stderr)
        return 2

    new = run_movie(NEW, frame)
    old = run_movie(OLD, frame)
    if not new or not old:
        print(f"error: no step movie captured (new={len(new)} old={len(old)} records). "
              f"Is replay_frame_counter == {frame} reached? Try frame+/-1, "
              f"and rebuild both binaries with the ZELDA3_REPLAY_STEP_DUMP hook.",
              file=sys.stderr)
        return 2

    n = min(len(new), len(old))
    if len(new) != len(old):
        print(f"!! step count differs: new={len(new)} old={len(old)} "
              f"(control flow diverged this frame) — comparing first {n} by index\n")
    # Warn at first label mismatch (control-flow divergence is itself the bug location).
    for i in range(n):
        if new[i][0] != old[i][0]:
            print(f"!! checkpoint #{i} label differs: new='{new[i][0]}' old='{old[i][0]}' "
                  f"— the control flow diverged here; that step is the bug.\n")
            break

    print(f"frame {frame}: {n} checkpoints captured\n")

    for tgt in targets:
        if page_mode:
            base = tgt & ~0x3FF
            print(f"=== page 0x{base:05x}-0x{base+0x3ff:05x} (1KB) diff-count timeline ===")
            first = None
            prev = -1
            for i in range(n):
                lbl, nw = new[i]
                ow = old[i][1]
                bad = [a for a in range(base, base + 0x400)
                       if a < len(nw) and a < len(ow) and nw[a] != ow[a]]
                if len(bad) != prev:
                    mark = ""
                    if first is None and bad:
                        first = (i, lbl, bad)
                        mark = "  <== FIRST DIVERGENCE"
                    print(f"  #{i:2} {lbl:<34} diffs={len(bad)}{mark}")
                    prev = len(bad)
            if first:
                i, lbl, bad = first
                shown = ", ".join(f"0x{a:05x}(N={new[i][1][a]:02x}/O={old[i][1][a]:02x})"
                                  for a in bad[:8])
                print(f"  --> first diverges AFTER step '{lbl}' (#{i}): {shown}")
            else:
                print("  (no divergence in this page on this frame)")
            print()
        else:
            print(f"=== byte 0x{tgt:05x} value timeline (only changes/diffs shown) ===")
            first = None
            prev = None
            for i in range(n):
                lbl, nw = new[i]
                ow = old[i][1]
                nv = nw[tgt] if tgt < len(nw) else None
                ov = ow[tgt] if tgt < len(ow) else None
                cur = (nv, ov)
                if cur != prev:
                    diff = "  <-- DIFFER" if nv != ov else ""
                    if first is None and nv != ov:
                        first = (i, lbl, nv, ov)
                        diff = "  <== FIRST DIVERGENCE"
                    print(f"  #{i:2} {lbl:<34} N=0x{nv:02x} O=0x{ov:02x}{diff}"
                          if nv is not None and ov is not None
                          else f"  #{i:2} {lbl:<34} N={nv} O={ov}")
                    prev = cur
            if first:
                i, lbl, nv, ov = first
                print(f"  --> 0x{tgt:05x} first diverges AFTER step '{lbl}' (#{i}): "
                      f"N=0x{nv:02x} O=0x{ov:02x}")
            else:
                print("  (matches at every checkpoint on this frame)")
            print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
