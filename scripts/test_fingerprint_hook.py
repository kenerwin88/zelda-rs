#!/usr/bin/env python3
"""The Rust --fingerprint-log stream is well-formed and its WRAM page column
matches the page hashes derived from a full WRAM dump at the same frame."""
import os, subprocess, sys, tempfile, struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
BIN = REPO / "target" / "parity" / "zelda3"
ROM = REPO / "saves" / "zelda3.sfc"
SAVE = REPO / "saves" / "zelda3-combined-route.sav"
RECORD_LEN = 788
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ("SELECT_FILE","LOADFILE","DUNGEON","OVERWORLD","MESSAGING","DEATH_INTRO","DEATH_RELOAD")}
MASK = {0x654}

def fnv_page(buf, start):
    h = 2166136261
    for off in range(start, start + 0x400):
        b = 0 if off in MASK else buf[off]
        h = ((h ^ b) * 16777619) & 0xffffffff
    return h

def main():
    frames = 300
    with tempfile.TemporaryDirectory() as td:
        fp = Path(td) / "fp.bin"
        wram = Path(td) / "w.bin"
        env = {**os.environ, **HACKS,
               "ZELDA3_REPLAY_WRAM_DUMP": str(wram)}
        subprocess.run([str(BIN), "--replay-save", str(ROM), str(SAVE), str(frames),
                        "--fingerprint-log", str(fp)], cwd=REPO, env=env, check=True,
                       capture_output=True, text=True)
        data = fp.read_bytes()
        assert len(data) == frames * RECORD_LEN, (len(data), frames * RECORD_LEN)
        # Last record's wram column == page hashes of the final WRAM dump.
        last = data[(frames - 1) * RECORD_LEN:]
        w = wram.read_bytes()
        for p in range(128):
            got = struct.unpack_from("<I", last, 4 + p * 4)[0]
            exp = fnv_page(w, p * 0x400)
            assert got == exp, f"page {p}: fp=0x{got:08x} dump=0x{exp:08x}"
        # frame field of last record == frames
        assert struct.unpack_from("<I", last, 0)[0] == frames
    print("fingerprint hook OK")

if __name__ == "__main__":
    sys.exit(main())
