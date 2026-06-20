#!/usr/bin/env python3
"""C oracle and Rust binary emit byte-identical fingerprint streams."""
import os, subprocess, sys, tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(REPO.parent / "zelda3")))
C_BIN = C_ROOT / "zelda3"
C_INI = C_ROOT / "other" / "headless_replay.ini"
NEW = REPO / "target" / "parity" / "zelda3"
ROM = REPO / "saves" / "zelda3.sfc"
SAVE = REPO / "saves" / "zelda3-combined-route.sav"
HACKS = {f"ZELDA3_SMV_{k}_TIMING_HACKS": "1" for k in
         ("SELECT_FILE","LOADFILE","DUNGEON","OVERWORLD","MESSAGING","DEATH_INTRO","DEATH_RELOAD")}
SDL = {"SDL_VIDEODRIVER":"dummy","SDL_AUDIODRIVER":"dummy","SDL_RENDER_DRIVER":"software"}

def main():
    frames = 500
    with tempfile.TemporaryDirectory() as td:
        rfp, cfp = Path(td)/"r.bin", Path(td)/"c.bin"
        subprocess.run([str(NEW), "--replay-save", str(ROM), str(SAVE), str(frames),
                        "--fingerprint-log", str(rfp)], cwd=REPO,
                       env={**os.environ, **HACKS}, check=True, capture_output=True, text=True)
        subprocess.run([str(C_BIN), "--config", str(C_INI), "--replay-save", str(SAVE),
                        "--smv-test-frames", str(frames), "--fingerprint-log", str(cfp)],
                       cwd=C_ROOT, env={**os.environ, **SDL}, check=True, capture_output=True, text=True)
        a, b = rfp.read_bytes(), cfp.read_bytes()
        assert len(a) == len(b), f"length mismatch: Rust={len(a)} C={len(b)}"
        for i in range(0, len(a), 788):
            if a[i:i+788] != b[i:i+788]:
                frame = i // 788
                ar, br = a[i:i+788], b[i:i+788]
                for off in range(0, 788, 4):
                    rv = int.from_bytes(ar[off:off+4], 'little')
                    cv = int.from_bytes(br[off:off+4], 'little')
                    if rv != cv:
                        print(f"  first diff at record offset {off} (0x{off:x}): Rust=0x{rv:08x} C=0x{cv:08x}")
                        break
                raise SystemExit(f"FAIL: fingerprint diverges at frame {frame} (record offset {i})")
    print("C/Rust fingerprint streams byte-identical")

if __name__ == "__main__":
    sys.exit(main())
