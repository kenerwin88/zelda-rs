#!/usr/bin/env python3
"""Oracle-seeded route segments: march every route take independently, in parallel.

Each active take of the continuous route starts at a Snes9x boundary state the
human recorder saved. This driver seeds Rust from that oracle state's memory
(`--seed-rust-from-oracle-state`, development evidence only) and runs the
renderless live-RNG engine-state comparison for the take's frames. Segments are
independent of every earlier segment's Rust behavior, so the whole route's
semantic divergence census comes back in one parallel pass.

    python3 scripts/segment_march.py --out /tmp/segments --concurrency 8
    python3 scripts/segment_march.py --out /tmp/segments --segments 10,11,12

Renderless runs take no compare lock. Session directories are deleted after each
segment is summarized (only the log and the summary survive) because a live-RNG
session writes ~50KB of oracle trace per frame.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import snes9x_route_recorder as recorder  # noqa: E402

DEFAULT_BINARY = ROOT / "target" / "parity" / "zelda3"
TRACE_CORE = ROOT / "external" / "snes9x-libretro" / "local" / "snes9x_libretro_trace.dylib"
DEFAULT_ROM = ROOT / "saves" / "zelda3.sfc"
ASSET_PACK = ROOT / "zelda3_assets.dat"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def segments_for(project: Path) -> list[dict]:
    take_ids = recorder.continuous_take_ids(project)
    ranges = recorder.continuous_take_ranges(project, take_ids)
    manifest = recorder.load_manifest(project)
    takes = {int(t["id"]): t for t in manifest["takes"]}
    boundaries = {int(b["id"]): b for b in manifest["boundaries"]}
    out = []
    for r in ranges:
        take = takes[r["take"]]
        b = boundaries[int(take["start_boundary"])]
        out.append(
            {
                "take": r["take"],
                "boundary": int(take["start_boundary"]),
                "name": r["name"],
                "start_frame": r["start_frame"],
                "end_frame": r["end_frame_exclusive"],
                "frames": r["frames"],
                "reset_start": bool(b.get("reset_start", False)),
                "oracle_state": str(project / b["state_path"]),
                "sram": str(project / b["sram_path"]),
            }
        )
    return out, take_ids


PANIC_RE = re.compile(r"panicked at [^\n]*\n([^\n]*)")
HOST_RE = re.compile(r"host=(\d+)")
RCPT_RE = re.compile(r"\[RCPT-RING\] host_call=(\d+)")
ENGINE_RE = re.compile(r"engine-state (?:mismatch|divergence)[^\n]*?frame (\d+)", re.I)
FRAME_AT_RE = re.compile(r"at frame (\d+)")


def summarize(log_text: str, seg: dict, returncode: int, end_requested: int | None = None) -> dict:
    lines = [l for l in log_text.splitlines() if not l.startswith("engine-state transient")]
    transients = len(re.findall(r"engine-state transient begins", log_text))
    result = {"returncode": returncode, "transients": transients}
    m = PANIC_RE.search(log_text)
    if m:
        result["kind"] = "panic"
        result["message"] = m.group(1).strip()[:400]
        loc = re.search(r"panicked at ([^:\n]+:\d+)", log_text)
        result["where"] = loc.group(1) if loc else None
        hosts = RCPT_RE.findall(log_text)
        h = HOST_RE.search(m.group(1))
        if h:
            result["frame"] = int(h.group(1))
        elif hosts:
            result["frame"] = int(hosts[-1]) + 1
        else:
            fa = FRAME_AT_RE.search(m.group(1))
            result["frame"] = int(fa.group(1)) if fa else None
    else:
        divergence = [l for l in lines if ("divergence" in l or "mismatch" in l) and "transient" not in l]
        if divergence:
            result["kind"] = "divergence"
            result["message"] = divergence[0][:400]
            fa = FRAME_AT_RE.search(divergence[0])
            result["frame"] = int(fa.group(1)) if fa else None
        elif returncode == 0:
            result["kind"] = "clean"
            result["frame"] = end_requested if end_requested is not None else seg["end_frame"]
        else:
            result["kind"] = "error"
            result["message"] = (lines[-1] if lines else "")[:400]
            result["frame"] = None
    if result.get("frame") is not None:
        result["progress"] = result["frame"] - seg["start_frame"]
    return result


def run_segment(seg: dict, args, input_path: Path, core_sha: str, rom_sha: str, out: Path, lock) -> dict:
    tag = f"seg-{seg['boundary']:02d}-t{seg['take']:02d}"
    log_path = out / f"{tag}.log"
    session = out / f"{tag}-session"
    end = seg["end_frame"]
    if args.frames_cap:
        end = min(end, seg["start_frame"] + args.frames_cap)
    cmd = [
        str(args.binary), "--compare-snes9x-oracle", str(TRACE_CORE), str(args.rom), str(end),
        "--expected-core-sha256", core_sha, "--expected-rom-sha256", rom_sha,
        "--input-script", str(input_path), "--live-oracle-rng",
    ]
    if seg["reset_start"]:
        cmd += ["--load-sram", seg["sram"]]
    else:
        cmd += [
            "--seed-rust-from-oracle-state", str(seg["start_frame"]),
            "--resume-oracle-state", seg["oracle_state"],
            "--resume-oracle-sram", seg["sram"],
        ]
    cmd += [
        "--skip-oracle-frames", "0", "--compare-from-frame", "0",
        "--compare-engine-state-from-frame", str(max(seg["start_frame"], args.engine_floor)),
        "--ignore-video", "--ignore-audio", "--session-dir", str(session / "replay"),
    ]
    env = dict(os.environ)
    env["ZELDA3_ASSET_PACK"] = str(ASSET_PACK)
    started = time.monotonic()
    with lock:
        print(f"start {tag}: frames {seg['start_frame']}..{end} ({seg['name']})", flush=True)
    with log_path.open("w") as log:
        log.write("+ " + " ".join(cmd) + "\n")
        log.flush()
        proc = subprocess.run(cmd, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, text=True)
    elapsed = time.monotonic() - started
    text = log_path.read_text(errors="replace")
    summary = summarize(text, seg, proc.returncode, end)
    summary.update({"tag": tag, "elapsed_s": round(elapsed, 1), "log": str(log_path), **seg, "end_requested": end})
    if not args.keep_sessions:
        shutil.rmtree(session, ignore_errors=True)
    with lock:
        print(
            f"done  {tag}: {summary['kind']} frame={summary.get('frame')} "
            f"progress={summary.get('progress')}/{end - seg['start_frame']} "
            f"{summary.get('elapsed_s')}s :: {summary.get('message', '')[:140]}",
            flush=True,
        )
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--project", type=Path, default=ROOT / "routes" / "full_run")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--concurrency", type=int, default=6)
    parser.add_argument("--segments", type=str, default=None, help="comma-separated boundary ids (default: all)")
    parser.add_argument("--frames-cap", type=int, default=0, help="cap frames per segment (smoke tests)")
    parser.add_argument("--engine-floor", type=int, default=0)
    parser.add_argument("--keep-sessions", action="store_true")
    parser.add_argument("--list", action="store_true")
    args = parser.parse_args()

    project = args.project if args.project.is_absolute() else ROOT / args.project
    segments, take_ids = segments_for(project)
    if args.list:
        for s in segments:
            print(f"boundary {s['boundary']:3d} take {s['take']:3d} frames {s['start_frame']:8d}..{s['end_frame']:8d} (+{s['frames']:6d}) reset={s['reset_start']} {s['name']}")
        return 0
    if args.segments:
        wanted = {int(x) for x in args.segments.split(",")}
        segments = [s for s in segments if s["boundary"] in wanted]
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    input_path = out / "input.txt"
    recorder.write_continuous_input(project, take_ids, input_path)
    core_sha = sha256_file(TRACE_CORE)
    rom_sha = sha256_file(args.rom)
    lock = threading.Lock()
    results = []
    # Longest segments first so the tail of the schedule is short.
    order = sorted(segments, key=lambda s: -s["frames"])
    with ThreadPoolExecutor(max_workers=args.concurrency) as pool:
        futures = [pool.submit(run_segment, s, args, input_path, core_sha, rom_sha, out, lock) for s in order]
        for f in futures:
            results.append(f.result())
    results.sort(key=lambda r: r["start_frame"])
    (out / "summary.json").write_text(json.dumps(results, indent=1))
    print("\n=== segment census ===")
    total_clean = 0
    for r in results:
        flag = "OK " if r["kind"] == "clean" else "XX "
        total_clean += r["kind"] == "clean"
        print(f"{flag} b{r['boundary']:02d} {r['start_frame']:8d}..{r['end_frame']:8d} {r['kind']:10s} frame={str(r.get('frame')):>8} +{str(r.get('progress')):>7}  {r.get('message','')[:110]}")
    print(f"{total_clean}/{len(results)} segments clean; summary at {out / 'summary.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
