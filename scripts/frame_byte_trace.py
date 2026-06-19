#!/usr/bin/env python3
"""Per-frame byte tracer for timing/sequence divergences.

The per-frame WRAM-dump probes (first_diverging_frame, manual bisect) re-replay from the start
for each frame, so pinning *when* a counter/animation field desyncs old-vs-new costs one replay
per candidate frame. This instead runs each binary ONCE with
`ZELDA3_REPLAY_FRAME_BYTE_DUMP=<a0,a1,...>:<path>`, which streams one record per frame:

    [frame:u32 LE][one byte per watched addr]   (at the end-of-frame after-run-frame-internal)

then diffs the two streams and, for each watched address, reports the FIRST frame it diverges
plus a short trace of (frame, old, new) around it — turning "which frame did this byte desync
and what were the values either side" into a single pair of replays.

    scripts/frame_byte_trace.py <start> <end> 0x25,0xaf4,0xcd2          # trace these addrs
    scripts/frame_byte_trace.py <start> <end> 0x25 --context 8          # +/-8 frames of trace
    scripts/frame_byte_trace.py <start> <end> 0x25 --new-ckpt /tmp/n.sav --old-ckpt /tmp/o.sav
    scripts/frame_byte_trace.py <start> <end> 0x25 --reuse              # reuse /tmp dumps

Resume from a checkpoint near <start> for speed (see CLAUDE.md "Checkpoint resume").
"""
from __future__ import annotations
import os, struct, subprocess, sys, pathlib, argparse

ROOT = pathlib.Path(__file__).resolve().parents[1]
NEW = ROOT / "target" / "parity" / "zelda3"
OLD = pathlib.Path(os.environ.get("ZELDA3_OLD_REPO", str(ROOT.parent / "zelda3-rs-old"))) \
    / "target" / "release" / "zelda3"
ROM = pathlib.Path(os.environ.get("ZELDA3_ROM", str(ROOT / "saves" / "zelda3.sfc")))
SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ["SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING",
          "DEATH_INTRO", "DEATH_RELOAD"]}


def run(binary, end, path, addrs_spec, ckpt):
    env = os.environ.copy()
    env.update(HACKS)
    env["ZELDA3_REPLAY_FRAME_BYTE_DUMP"] = f"{addrs_spec}:{path}"
    cmd = [str(binary), "--replay-save", str(ROM), str(SAVE), str(end)]
    if ckpt:
        cmd += ["--load-state", str(ckpt)]
    subprocess.run(cmd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)


def load(path, n_addrs):
    rec = 4 + n_addrs
    out = {}
    data = pathlib.Path(path).read_bytes()
    for i in range(0, len(data) - rec + 1, rec):
        frame = struct.unpack_from("<I", data, i)[0]
        out[frame] = data[i + 4:i + 4 + n_addrs]
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("start", type=int)
    ap.add_argument("end", type=int)
    ap.add_argument("addrs", help="comma-separated hex addrs, e.g. 0x25,0xaf4")
    ap.add_argument("--context", type=int, default=5)
    ap.add_argument("--new-ckpt", default=None)
    ap.add_argument("--old-ckpt", default=None)
    ap.add_argument("--reuse", action="store_true")
    args = ap.parse_args()

    addr_list = [int(s.strip(), 16) for s in args.addrs.split(",")]
    n = len(addr_list)
    np_, op_ = "/tmp/frame_byte_new.bin", "/tmp/frame_byte_old.bin"
    # The dump's BufWriter isn't flushed on process exit, so the last ~150 frames are lost.
    # Replay a bit past the report window so the whole window lands in the flushed region.
    replay_end = args.end + 600
    if not args.reuse:
        print(f"running NEW to {replay_end} ...", flush=True)
        run(NEW, replay_end, np_, args.addrs, args.new_ckpt)
        print(f"running OLD to {replay_end} ...", flush=True)
        run(OLD, replay_end, op_, args.addrs, args.old_ckpt)

    new, old = load(np_, n), load(op_, n)
    frames = sorted(f for f in new.keys() & old.keys() if args.start <= f <= args.end)
    if not frames:
        print("no overlapping frames in range (did both dumps get written?)")
        return

    for ai, addr in enumerate(addr_list):
        first = None
        for f in frames:
            if new[f][ai] != old[f][ai]:
                first = f
                break
        if first is None:
            print(f"0x{addr:05x}: MATCHES across {frames[0]}..{frames[-1]}")
            continue
        print(f"0x{addr:05x}: first diverges at frame {first} "
              f"(OLD=0x{old[first][ai]:02x} NEW=0x{new[first][ai]:02x})")
        lo = max(0, frames.index(first) - args.context)
        hi = min(len(frames), frames.index(first) + args.context + 1)
        for f in frames[lo:hi]:
            mark = "  <-- diverge" if new[f][ai] != old[f][ai] else ""
            print(f"    f{f}: OLD=0x{old[f][ai]:02x} NEW=0x{new[f][ai]:02x}{mark}")


if __name__ == "__main__":
    main()
