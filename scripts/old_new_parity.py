#!/usr/bin/env python3
"""Old-vs-new Rust replay parity harness.

Replays the same input movie through a known-good reference build and the
current build, sweeping frames and comparing *everything* the binary exposes:
whole-WRAM hash + per-page hashes + SRAM (bytes), every semantic state field on
the replay-save line, and the rendered frame (graphics).

On the first divergence it binary-searches the exact first bad frame and writes
a self-contained bundle that pinpoints WHERE the problem is:
  * the exact diverging WRAM bytes, each annotated with the named constant it
    belongs to (so an address becomes e.g. `DUNGEON_BG1_ATTR_TABLE+0x12`);
  * which per-subsystem dump (sprites / dungmap / message / doors / dungeon
    attrs / palette / ...) differs, with the source file that owns it;
  * both screenshots, both save-states, and both full state lines.
A progress file records the furthest frame proven identical, so reruns resume.

Typical use:
    python3 scripts/old_new_parity.py --end 20000 --step 250

Build both binaries first (this harness does not build):
    cargo build --release -p zelda3-bin                              # new (this repo)
    (cd ../zelda3-rs-old && ZELDA3_ROM=/path/zelda3.sfc cargo build --release -p zelda3-bin)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
# Default to the fast 'parity' profile build (deterministic stand-in for release).
DEFAULT_NEW_BIN = ROOT / "target" / "parity" / "zelda3"
DEFAULT_OLD_BIN = ROOT.parent / "zelda3-rs-old" / "target" / "release" / "zelda3"
# Commit the good reference clone is built at; used to pickaxe culprit commits.
DEFAULT_GOOD_REF = "1183dee4b6f0ba9bc1db39b035a587cbcee861e7"
# Rust-vs-Rust only: ROM lives in this repo (saves/, gitignored), never ../zelda3.
DEFAULT_ROM = pathlib.Path(os.environ.get("ZELDA3_ROM", str(ROOT / "saves" / "zelda3.sfc")))
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_OUT = ROOT / ".cache" / "old-new-parity"
CONSTANTS_RS = ROOT / "crates" / "zelda3" / "src" / "game_state" / "constants.rs"

TIMING_HACK_ENV = {
    "ZELDA3_SMV_SELECT_FILE_TIMING_HACKS": "1",
    "ZELDA3_SMV_LOADFILE_TIMING_HACKS": "1",
    "ZELDA3_SMV_DUNGEON_TIMING_HACKS": "1",
    "ZELDA3_SMV_OVERWORLD_TIMING_HACKS": "1",
    "ZELDA3_SMV_MESSAGING_TIMING_HACKS": "1",
    "ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS": "1",
    "ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS": "1",
}

COMPLETED_RE = re.compile(r"replay-save completed frames=(\d+)\b")
PAGE_FIELDS = [f"ram{i}" for i in range(8)]
IGNORE_FIELDS = {"_ok", "_rc", "_raw", "frames"}

# Per-subsystem dumps: env var -> (line prefix, source files that own it).
# When one of these dumps differs across builds, the bug lives in that file.
SUBSYSTEM_DUMPS = {
    "ZELDA3_REPLAY_SPRITE_DUMP": ("sprites", "sprite*.rs  <- C sprite*.c"),
    "ZELDA3_REPLAY_DUNGMAP_DUMP": ("dungmap", "messaging.rs (dungeon map)  <- C messaging.c"),
    "ZELDA3_REPLAY_MESSAGE_DUMP": ("message", "messaging.rs (text)  <- C messaging.c"),
    "ZELDA3_REPLAY_DOOR_DUMP": ("doors", "dungeon.rs (doors)  <- C dungeon.c"),
    "ZELDA3_REPLAY_DUNGEON_ATTR_DUMP": ("dungeon-attrs", "dungeon.rs (attributes)  <- C dungeon.c"),
    "ZELDA3_REPLAY_DUNGEON_ATTR_STATE_DUMP": ("dungeon-attr-state", "game_state/native/dungeon.rs"),
    "ZELDA3_REPLAY_PALETTE_DUMP": ("palette", "game_state/native/display.rs  <- C palette code"),
    "ZELDA3_REPLAY_OVERLORD_DUMP": ("overlords", "overlord.rs  <- C overlord.c"),
    "ZELDA3_REPLAY_GARNISH_DUMP": ("garnish", "garnish/ancilla  <- C garnish.c"),
    "ZELDA3_REPLAY_ROOM_HISTORY_DUMP": ("room-history", "dungeon.rs (room history)"),
    "ZELDA3_REPLAY_ROOM_MASK_DUMP": ("room-masks", "sprite.rs (room death masks)"),
    "ZELDA3_REPLAY_RNG_DUMP": ("rng", "misc.rs get_random_number  <- C GetRandomNumber"),
}

# Coarse WRAM-region -> owning-subsystem hint, for diverging bytes that have no
# exact named constant. Keyed by (lo, hi) inclusive-exclusive byte range.
REGION_HINTS = [
    (0x0000, 0x0100, "link/player + DMA scratch (player.rs, link/*)"),
    (0x0100, 0x0500, "sprite slot arrays (sprite*.rs)"),
    (0x0c000, 0x0c200, "overworld/exit decode (overworld.rs)"),
    (0x0f800, 0x0fc00, "tile/draw scratch + memorized tiles (sprite_main_draw.rs, tile_detect.rs)"),
    (0x10000, 0x11000, "messaging render buffer (messaging.rs)"),
    (0x12000, 0x14000, "dungeon BG attribute tables (dungeon.rs / native/dungeon.rs)"),
]


def load_constants():
    """Parse constants.rs -> sorted [(addr, NAME)] for address annotation."""
    out = []
    if not CONSTANTS_RS.exists():
        return out
    pat = re.compile(r"const\s+([A-Z0-9_]+)\s*:\s*usize\s*=\s*0x([0-9a-fA-F]+)\s*;")
    for line in CONSTANTS_RS.read_text().splitlines():
        m = pat.search(line)
        if m:
            out.append((int(m.group(2), 16), m.group(1)))
    out.sort()
    return out


def name_for_addr(addr: int, consts) -> str:
    """Nearest named constant at or below addr (within a small window)."""
    best = None
    for caddr, name in consts:
        if caddr <= addr:
            best = (caddr, name)
        else:
            break
    if best is None:
        return region_hint(addr)
    caddr, name = best
    off = addr - caddr
    if off == 0:
        return name
    if off <= 0x40:
        return f"{name}+0x{off:x}"
    return region_hint(addr)


def region_hint(addr: int) -> str:
    for lo, hi, hint in REGION_HINTS:
        if lo <= addr < hi:
            return f"~{hint}"
    return "(unnamed)"


class Build:
    def __init__(self, name, binary, args, ckpt_root):
        self.name = name
        self.binary = binary
        self.args = args
        self.ckpt_dir = ckpt_root / name
        self.ckpt_dir.mkdir(parents=True, exist_ok=True)
        self.checkpoints: dict[int, pathlib.Path] = {}

    def _env(self, extra=None):
        env = os.environ.copy()
        env.update(TIMING_HACK_ENV)
        if extra:
            env.update(extra)
        return env

    def _nearest_ckpt(self, frame):
        best = None
        for cf in self.checkpoints:
            if cf <= frame and (best is None or cf > best):
                best = cf
        return best

    def run(self, frame, *, save_ckpt=False, dump_png=None, save_state=None, extra_env=None):
        cmd = [str(self.binary), "--replay-save", str(self.args.rom), str(self.args.save), str(frame)]
        near = self._nearest_ckpt(frame)
        if near is not None and near > 0:
            cmd += ["--load-state", str(self.checkpoints[near])]
        if dump_png is not None:
            cmd += ["--dump-frame", str(dump_png)]
        ckpt_target = save_state if save_state is not None else (
            self.ckpt_dir / f"f{frame}.sav" if save_ckpt else None)
        if ckpt_target is not None:
            cmd += ["--save-state", str(ckpt_target)]
        proc = subprocess.run(cmd, cwd=ROOT, env=self._env(extra_env), text=True,
                              stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
        state = parse_state(proc.stdout)
        state["_rc"] = proc.returncode
        state["_raw"] = proc.stdout
        if ckpt_target is not None and ckpt_target.exists() and save_state is None:
            self.checkpoints[frame] = ckpt_target
        return state

    def dump_lines(self, frame, env_var, prefix):
        """Run with a dump env var; return stdout lines starting with prefix."""
        raw = self.run(frame, extra_env={env_var: "1"}).get("_raw", "")
        return [ln for ln in raw.splitlines() if ln.startswith(prefix)]

    def page_checksums(self, frame):
        for ln in self.run(frame, extra_env={"ZELDA3_REPLAY_RAM_PAGE_DUMP": "1"}).get("_raw", "").splitlines():
            if ln.startswith("ram-pages"):
                return dict(re.findall(r"\[([0-9a-f]{5})=0x([0-9a-f]{8})\]", ln))
        return {}

    def page_bytes(self, frame, page_addr):
        env = {"ZELDA3_REPLAY_RAM_DUMP_PAGE": f"0x{page_addr:05x}"}
        for ln in self.run(frame, extra_env=env).get("_raw", "").splitlines():
            if ln.startswith("ram-page-bytes"):
                return dict(re.findall(r"\[([0-9a-f]{5})=0x([0-9a-f]{2})\]", ln))
        return {}


def parse_state(output):
    line = next((ln.strip() for ln in output.splitlines() if COMPLETED_RE.search(ln)), None)
    if line is None:
        return {"_ok": False}
    state = {"_ok": True}
    for tok in line.split():
        if "=" in tok:
            k, v = tok.split("=", 1)
            state[k] = v
    return state


def png_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()[:16] if path.exists() else None


def compare(old, new):
    if not old.get("_ok") or not new.get("_ok"):
        return [("_replay", f"ok={old.get('_ok')} rc={old.get('_rc')}",
                 f"ok={new.get('_ok')} rc={new.get('_rc')}")]
    return [(k, str(old.get(k)), str(new.get(k)))
            for k in sorted(set(old) | set(new))
            if k not in IGNORE_FIELDS and not k.startswith("_") and old.get(k) != new.get(k)]


def first_divergent_frame(old, new, good, bad):
    lo, hi = good, bad
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        hi, lo = (mid, lo) if compare(old.run(mid), new.run(mid)) else (hi, mid)
    return hi


def wram_byte_diff(old, new, frame, consts, limit=400):
    """Localize to 1KB chunks, then byte-diff each, annotated with constant names."""
    oc, nc = old.page_checksums(frame), new.page_checksums(frame)
    if not oc or not nc:
        return None, []
    chunks = sorted(int(k, 16) for k in (set(oc) | set(nc)) if oc.get(k) != nc.get(k))
    diffs = []
    for chunk in chunks:
        ob, nb = old.page_bytes(frame, chunk), new.page_bytes(frame, chunk)
        for a in sorted(set(ob) | set(nb), key=lambda x: int(x, 16)):
            if ob.get(a) != nb.get(a):
                addr = int(a, 16)
                diffs.append((addr, ob.get(a, "00"), nb.get(a, "00"), name_for_addr(addr, consts)))
                if len(diffs) >= limit:
                    return chunks, diffs
    return chunks, diffs


def subsystem_diff(old, new, frame):
    """Which per-subsystem dumps differ -> names the owning source file."""
    hits = []
    for env_var, (prefix, hint) in SUBSYSTEM_DUMPS.items():
        ol, nl = old.dump_lines(frame, env_var, prefix), new.dump_lines(frame, env_var, prefix)
        if ol != nl:
            sample_o = next((o for o, n in zip(ol, nl) if o != n), (ol[len(nl):] or [""])[0] if ol else "")
            sample_n = next((n for o, n in zip(ol, nl) if o != n), (nl[len(ol):] or [""])[0] if nl else "")
            hits.append({"subsystem": prefix, "source": hint,
                         "sample_old": sample_o[:200], "sample_new": sample_n[:200]})
    return hits


def pickaxe_culprits(names, good_ref, limit=10):
    """For each diverging named constant, find the commits in good_ref..HEAD that
    changed code mentioning it (git -S pickaxe). Replaces bisecting: points
    straight at the commit that altered the field's handling, with no rebuilds."""
    out = {}
    for name in list(dict.fromkeys(names))[:limit]:
        res = subprocess.run(
            ["git", "log", "--oneline", "-S", name, f"{good_ref}..HEAD", "--", "crates/zelda3/src"],
            cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, check=False)
        commits = [ln.strip() for ln in res.stdout.splitlines() if ln.strip()][:3]
        if commits:
            out[name] = commits
    return out


def capture(old, new, frame, out_dir, consts, good_ref):
    bundle = out_dir / f"divergence-f{frame}"
    if bundle.exists():
        shutil.rmtree(bundle)
    bundle.mkdir(parents=True)
    prev = max(frame - 1, 0)
    old_prev, new_prev = old.run(prev), new.run(prev)
    old_cur = old.run(frame, dump_png=bundle / "old.png", save_state=bundle / "old.sav")
    new_cur = new.run(frame, dump_png=bundle / "new.png", save_state=bundle / "new.sav")

    field_diffs = compare(old_cur, new_cur)
    chunks, byte_diffs = wram_byte_diff(old, new, frame, consts)
    subsystems = subsystem_diff(old, new, frame)
    png_old, png_new = png_hash(bundle / "old.png"), png_hash(bundle / "new.png")

    # Named constants behind the diverging bytes -> pickaxe their culprit commits.
    culprit_names = [nm.split("+")[0] for _, _, _, nm in byte_diffs
                     if not nm.startswith(("~", "("))]
    culprits = pickaxe_culprits(culprit_names, good_ref)

    for nm, st in (("old", old_cur), ("new", new_cur),
                   ("old.prev", old_prev), ("new.prev", new_prev)):
        (bundle / f"{nm}.state.txt").write_text(st.get("_raw", ""))

    report = {
        "first_divergent_frame": frame,
        "last_identical_frame": prev,
        "graphics_diff": png_old != png_new,
        "state_field_diffs": [{"field": k, "old": o, "new": n} for k, o, n in field_diffs],
        "diverging_1kb_chunks": [f"0x{c:05x}" for c in (chunks or [])],
        "wram_byte_diffs": [
            {"addr": f"0x{a:05x}", "name": nm, "old": f"0x{o}", "new": f"0x{n}"}
            for a, o, n, nm in byte_diffs],
        "diverging_subsystems": subsystems,
        "likely_culprit_commits": culprits,
    }
    (bundle / "report.json").write_text(json.dumps(report, indent=2))

    print(f"\n=== FIRST DIVERGENCE at frame {frame} (last identical: {prev}) ===")
    print(f"bundle: {bundle}")
    if png_old != png_new:
        print(f"GRAPHICS differ (old.png={png_old} new.png={png_new})")
    if subsystems:
        print("diverging SUBSYSTEMS (where to look):")
        for s in subsystems:
            print(f"  [{s['subsystem']}]  {s['source']}")
            if s["sample_old"] or s["sample_new"]:
                print(f"      old: {s['sample_old']}")
                print(f"      new: {s['sample_new']}")
    if culprits:
        print("LIKELY CULPRIT COMMITS (git -S pickaxe on diverging constants):")
        for name, commits in culprits.items():
            print(f"  {name}:")
            for c in commits:
                print(f"      {c}")
    if byte_diffs:
        print(f"diverging WRAM bytes ({len(byte_diffs)}; first 40):")
        for a, o, n, nm in byte_diffs[:40]:
            print(f"  0x{a:05x} [{nm}]: old=0x{o} new=0x{n}")
        if len(byte_diffs) > 40:
            print(f"  ... {len(byte_diffs) - 40} more (report.json)")
    if field_diffs:
        print("state field diffs (old -> new):")
        for k, o, n in field_diffs[:30]:
            print(f"  {k}: {o} -> {n}")
    return report


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--old-bin", type=pathlib.Path, default=DEFAULT_OLD_BIN)
    ap.add_argument("--new-bin", type=pathlib.Path, default=DEFAULT_NEW_BIN)
    ap.add_argument("--rom", type=pathlib.Path, default=DEFAULT_ROM)
    ap.add_argument("--save", type=pathlib.Path, default=DEFAULT_SAVE)
    ap.add_argument("--out-dir", type=pathlib.Path, default=DEFAULT_OUT)
    ap.add_argument("--start", type=int, default=0)
    ap.add_argument("--end", type=int, default=20000)
    ap.add_argument("--step", type=int, default=250)
    ap.add_argument("--checkpoint-interval", type=int, default=2000)
    ap.add_argument("--good-ref", default=DEFAULT_GOOD_REF,
                    help="commit the good reference clone is built at (for culprit pickaxe)")
    ap.add_argument("--semantic-only", action="store_true",
                    help="ignore raw WRAM/SRAM hashes; flag only semantic field (behavior) "
                         "divergences. Use to find the real behavior bug past benign RAM-shadow lag.")
    ap.add_argument("--resume", action="store_true")
    args = ap.parse_args()

    if args.semantic_only:
        IGNORE_FIELDS.update(PAGE_FIELDS)
        IGNORE_FIELDS.update({"ramhash", "sramhash"})

    for label, b in (("old", args.old_bin), ("new", args.new_bin)):
        if not b.exists():
            print(f"missing {label} binary: {b}\nbuild it first (see header docstring).", file=sys.stderr)
            return 2

    args.out_dir.mkdir(parents=True, exist_ok=True)
    progress_file = args.out_dir / "progress.json"
    consts = load_constants()
    high_water = args.start
    if args.resume and progress_file.exists():
        high_water = json.loads(progress_file.read_text()).get("high_water", args.start)
        print(f"resuming from high-water frame {high_water}")

    old = Build("old", args.old_bin, args, args.out_dir / "ckpt")
    new = Build("new", args.new_bin, args, args.out_dir / "ckpt")

    start = max(args.start, high_water)
    frame = start if start % args.step == 0 else start + (args.step - start % args.step)
    frame = max(frame, args.step)
    last_good = max(start - args.step, 0)

    print(f"sweeping {frame}..{args.end} step {args.step} ({len(consts)} constants loaded)")
    while frame <= args.end:
        is_ckpt = frame % args.checkpoint_interval == 0
        diffs = compare(old.run(frame, save_ckpt=is_ckpt), new.run(frame, save_ckpt=is_ckpt))
        if diffs:
            fields = ",".join(d[0] for d in diffs[:8])
            print(f"frame {frame}: DIVERGED ({fields}{'...' if len(diffs) > 8 else ''}) "
                  f"-- narrowing in ({last_good}, {frame}]")
            first = first_divergent_frame(old, new, last_good, frame)
            capture(old, new, first, args.out_dir, consts, args.good_ref)
            progress_file.write_text(json.dumps(
                {"high_water": max(first - 1, 0), "first_divergence": first}, indent=2))
            return 1
        last_good = high_water = frame
        progress_file.write_text(json.dumps({"high_water": high_water}, indent=2))
        print(f"frame {frame}: parity OK  (high-water {high_water})")
        frame += args.step

    print(f"\nNo divergence found through frame {args.end}. high-water={high_water}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
