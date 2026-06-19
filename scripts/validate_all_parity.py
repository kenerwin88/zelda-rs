#!/usr/bin/env python3
"""Validate every parity layer of THIS repo against the C oracle — the regression gate.

The C checkout (`../zelda3`, override ZELDA3_C_REPO) is the ground-truth oracle.
This repo's parity binary is run alongside it over the canonical replay route and
every layer that has ever diverged is compared, byte-for-byte where possible:

  * WRAM   — full 128KB main memory (ZELDA3_REPLAY_WRAM_DUMP), byte-equal
             + ramhash / ram0..ram7 from the completion line
  * VRAM   — full 64KB PPU tile/tilemap memory (ZELDA3_VRAM_DUMP), byte-equal
  * SRAM   — battery-save hash (sramhash)
  * RENDER — CPU PPU rendered-frame hash, every frame (--render-hash-log 1):
             transitively covers CGRAM, OAM, color-math, windows — any pixel change
  * AUDIO  — APU/DSP per-frame trace JSON (--audio-trace-log): DSP pre/post hashes,
             write hashes, sample hash, music/queue state

The C oracle and the Rust binary emit byte-identical dumps/traces for matching
state, so comparison is exact. Any mismatch fails the gate (exit 1) with the first
diverging frame/offset.

Requires the C oracle hooks committed in ../zelda3 (audio-trace default freq, the
WRAM/VRAM dump env vars). Rebuild the oracle with `make -C ../zelda3 zelda3`.

Usage:
  scripts/validate_all_parity.py                 # fast smoke test (3000 frames)
  scripts/validate_all_parity.py --frames 12000
  scripts/validate_all_parity.py --full          # full route ~1,073,092 frames (exhaustive)
  scripts/validate_all_parity.py --build         # cargo build the parity binary first
  scripts/validate_all_parity.py --no-audio      # skip the audio layer
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(REPO.parent / "zelda3")))
C_BIN = C_ROOT / "zelda3"
C_INI = C_ROOT / "other" / "headless_replay.ini"
ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO / "saves" / "zelda3.sfc")))
SAVE = Path(os.environ.get("ZELDA3_REPLAY_SAVE", str(REPO / "saves" / "zelda3-combined-route.sav")))


def _pick_new_bin() -> Path:
    override = os.environ.get("ZELDA3_NEW_BIN")
    if override:
        return Path(override)
    parity, release = REPO / "target" / "parity" / "zelda3", REPO / "target" / "release" / "zelda3"
    if parity.exists() and release.exists():
        return parity if parity.stat().st_mtime >= release.stat().st_mtime else release
    return parity if parity.exists() else release


NEW_BIN = _pick_new_bin()

TIMING_HACKS = {
    f"ZELDA3_SMV_{k}_TIMING_HACKS": "1"
    for k in ("SELECT_FILE", "LOADFILE", "DUNGEON", "OVERWORLD", "MESSAGING", "DEATH_INTRO", "DEATH_RELOAD")
}
SDL_DUMMY = {"SDL_VIDEODRIVER": "dummy", "SDL_AUDIODRIVER": "dummy", "SDL_RENDER_DRIVER": "software"}
HASH_FIELDS = ["ramhash", "ram0", "ram1", "ram2", "ram3", "ram4", "ram5", "ram6", "ram7", "sramhash"]
# Canonical final frame of the combined replay route (test_standard_replay_parity.py).
FULL_ROUTE_FRAMES = 1_073_092


def parse_fields(line: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for tok in line.split():
        if "=" in tok:
            k, v = tok.split("=", 1)
            out[k] = v
    return out


def run(cmd, cwd, env, wram, vram):
    env = {**os.environ, **env, "ZELDA3_REPLAY_WRAM_DUMP": str(wram), "ZELDA3_VRAM_DUMP": str(vram)}
    proc = subprocess.run(cmd, cwd=cwd, env=env, text=True, capture_output=True)
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr[-2000:])
        raise SystemExit(f"FAIL: {cmd[0]} exited {proc.returncode}")
    state, render, audio = "", [], []
    for line in proc.stdout.splitlines():
        if line.startswith("render-hash "):
            render.append(line)
        elif line.startswith('{"frame"'):
            audio.append(line)
        elif line.startswith("smv-test frame=") or line.startswith("replay-save completed"):
            state = line
    return state, render, audio


def first_byte_diff(a: Path, b: Path):
    da, db = a.read_bytes(), b.read_bytes()
    n = min(len(da), len(db))
    for i in range(n):
        if da[i] != db[i]:
            return (i, da[i], db[i])
    return None if len(da) == len(db) else (n, -1, -1)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--frames", type=int, default=3000)
    # Canonical end of the combined replay route (matches test_standard_replay_parity.py).
    ap.add_argument("--full", action="store_true", help="full route (~1,073,092 frames)")
    ap.add_argument("--render-stride", type=int, default=1)
    ap.add_argument("--audio-stride", type=int, default=1)
    ap.add_argument("--no-audio", action="store_true")
    ap.add_argument("--build", action="store_true")
    args = ap.parse_args()
    frames = FULL_ROUTE_FRAMES if args.full else args.frames

    if args.build:
        subprocess.run(["cargo", "build", "--profile", "parity", "-p", "zelda3-bin"], cwd=REPO, check=True)

    for path, what in [(C_BIN, "C oracle binary"), (NEW_BIN, "parity binary"), (ROM, "ROM"), (SAVE, "replay save")]:
        if not path.exists():
            raise SystemExit(f"missing {what}: {path}")
    c_main = C_ROOT / "src" / "main.c"
    if c_main.exists() and c_main.stat().st_mtime > C_BIN.stat().st_mtime:
        sys.stderr.write(f"WARNING: {C_BIN} older than src/main.c — rebuild: make -C {C_ROOT} zelda3\n")

    astride = 0 if args.no_audio else args.audio_stride
    print(f"validate_all_parity: THIS repo vs C oracle over {frames} frames")

    with tempfile.TemporaryDirectory() as td:
        rw, rv = Path(td) / "r.wram", Path(td) / "r.vram"
        cw, cv = Path(td) / "c.wram", Path(td) / "c.vram"
        rust_cmd = [str(NEW_BIN), "--replay-save", str(ROM), str(SAVE), str(frames),
                    "--render-hash-log", str(args.render_stride)]
        c_cmd = [str(C_BIN), "--config", str(C_INI), "--replay-save", str(SAVE),
                 "--smv-test-frames", str(frames), "--render-hash-log", str(args.render_stride)]
        if astride:
            rust_cmd += ["--audio-trace-log", str(astride)]
            c_cmd += ["--audio-trace-log", str(astride)]
        r_state, r_render, r_audio = run(rust_cmd, REPO, TIMING_HACKS, rw, rv)
        c_state, c_render, c_audio = run(c_cmd, C_ROOT, SDL_DUMMY, cw, cv)

        failures: list[str] = []
        rf, cf = parse_fields(r_state), parse_fields(c_state)

        # WRAM byte-equal + hashes
        d = first_byte_diff(cw, rw)
        wram_hash_bad = [f for f in HASH_FIELDS[:-1] if rf.get(f) != cf.get(f)]
        if d is None and not wram_hash_bad:
            print("  [PASS] WRAM   128KB byte-identical (+ ramhash/ram0..7)")
        else:
            if d:
                failures.append(f"WRAM byte 0x{d[0]:05x}: C=0x{d[1]:02x} RUST=0x{d[2]:02x}")
                print(f"  [FAIL] WRAM   {failures[-1]}")
            for f in wram_hash_bad:
                failures.append(f"{f}: C={cf.get(f)} RUST={rf.get(f)}")
                print(f"  [FAIL] WRAM   {failures[-1]}")

        # VRAM byte-equal
        d = first_byte_diff(cv, rv)
        if d is None:
            print("  [PASS] VRAM   64KB byte-identical")
        else:
            failures.append(f"VRAM byte 0x{d[0]:05x}: C=0x{d[1]:02x} RUST=0x{d[2]:02x}")
            print(f"  [FAIL] VRAM   {failures[-1]}")

        # SRAM
        if rf.get("sramhash") == cf.get("sramhash"):
            print("  [PASS] SRAM   sramhash match")
        else:
            failures.append(f"sramhash: C={cf.get('sramhash')} RUST={rf.get('sramhash')}")
            print(f"  [FAIL] SRAM   {failures[-1]}")

        # RENDER
        rd = next((i for i in range(min(len(r_render), len(c_render))) if r_render[i] != c_render[i]), None)
        if rd is None and len(r_render) == len(c_render):
            print(f"  [PASS] RENDER {len(r_render)} frame hashes match")
        else:
            msg = (f"first diff RUST[{rd}]={r_render[rd]} C={c_render[rd]}" if rd is not None
                   else f"line-count RUST={len(r_render)} C={len(c_render)}")
            failures.append(f"RENDER {msg}")
            print(f"  [FAIL] RENDER {msg}")

        # AUDIO
        if astride:
            ad = next((i for i in range(min(len(r_audio), len(c_audio))) if r_audio[i] != c_audio[i]), None)
            if ad is None and len(r_audio) == len(c_audio):
                print(f"  [PASS] AUDIO  {len(r_audio)} frame traces match")
            else:
                msg = (f"first diff at trace {ad}" if ad is not None
                       else f"line-count RUST={len(r_audio)} C={len(c_audio)}")
                failures.append(f"AUDIO {msg}")
                print(f"  [FAIL] AUDIO  {msg}")
        else:
            print("  [SKIP] AUDIO  (--no-audio)")

    if failures:
        print(f"\nPARITY GATE FAILED — {len(failures)} mismatch(es) vs C oracle.")
        return 1
    print("\nALL LAYERS MATCH C ORACLE — perfect parity.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
