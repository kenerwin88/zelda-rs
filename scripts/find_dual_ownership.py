#!/usr/bin/env python3
"""Static dual-ownership finder for the native game-state migration.

A "dual-ownership" bug is when two *different* native `*State` structs both
write the SAME WRAM byte from their `write_to_ram` (the full-state projection
used by every bridge `sync()`). Whichever projects last wins, so if the two
structs hold different values they mutually clobber that byte — the exact class
of overlap bug this migration keeps hitting (BG-scroll-copy2, attract-vram,
camera-scroll-cache, exit-tile-themes, ...).

This script parses every native state file, resolves the WRAM address(es) each
`write_to_ram` writes, and reports any address written by 2+ structs.

It is a *static* check: it does not run the game, so it finds latent overlaps
that the replay harness only surfaces once some code actually reads the
clobbered byte (and even then only via flaky raw-hash bisection). Run it after
each migration commit.

    python3 scripts/find_dual_ownership.py            # report overlaps
    python3 scripts/find_dual_ownership.py --verbose  # also list unresolved writes

Exit code 1 if any overlap is found (usable as a CI gate).
"""
from __future__ import annotations

import argparse
import pathlib
import re
import sys
from collections import defaultdict

ROOT = pathlib.Path(__file__).resolve().parents[1]
NATIVE_DIR = ROOT / "crates" / "zelda3" / "src" / "game_state" / "native"
CRATE_SRC = ROOT / "crates" / "zelda3" / "src"

# ----------------------------------------------------------------------------
# 1. Resolve all `const NAME: usize = <addr|expr>;` definitions to addresses.
# ----------------------------------------------------------------------------
CONST_RE = re.compile(
    r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*([^;]+);"
)
# array-length style: `const NAME: usize = 16;` also matches above (fine).


def parse_int(tok: str):
    tok = tok.strip()
    try:
        return int(tok, 0)
    except ValueError:
        return None


def collect_constants() -> dict[str, int]:
    raw: dict[str, str] = {}
    for path in CRATE_SRC.rglob("*.rs"):
        text = path.read_text(errors="replace")
        for m in CONST_RE.finditer(text):
            name, expr = m.group(1), m.group(2).strip()
            raw.setdefault(name, expr)

    resolved: dict[str, int] = {}

    def resolve(name: str, seen: frozenset) -> int | None:
        if name in resolved:
            return resolved[name]
        if name not in raw or name in seen:
            return None
        expr = raw[name]
        # direct literal
        val = parse_int(expr)
        if val is not None:
            resolved[name] = val
            return val
        # NAME, NAME + N, NAME - N  (single referenced const)
        m = re.fullmatch(r"([A-Z][A-Z0-9_]*)\s*([+\-]\s*(?:0x[0-9a-fA-F]+|\d+))?", expr)
        if m:
            base = resolve(m.group(1), seen | {name})
            if base is None:
                return None
            off = 0
            if m.group(2):
                off = parse_int(m.group(2).replace(" ", ""))
                off = off if off is not None else 0
            resolved[name] = base + off
            return resolved[name]
        return None

    for name in list(raw):
        resolve(name, frozenset())
    return resolved


# ----------------------------------------------------------------------------
# 2. Resolve simple length/count constants used in range writes (`+ N`).
# ----------------------------------------------------------------------------
def addr_of(tok: str, consts: dict[str, int]) -> int | None:
    tok = tok.strip()
    v = parse_int(tok)
    if v is not None:
        return v
    return consts.get(tok)


# helper write fns -> element count (each element is u16 = 2 bytes)
HELPER_SPANS = {
    "write_scroll_targets": ("SCROLL_TARGET_COUNT", 2),
    "write_scroll_counters": ("SCROLL_COUNTER_COUNT", 2),
}


# ----------------------------------------------------------------------------
# 3. Extract (start, end) write intervals from a write_to_ram body.
# ----------------------------------------------------------------------------
def extract_writes(body: str, consts: dict[str, int], unresolved: list):
    """Yield (start, end, label) byte intervals written in this body."""
    intervals = []

    def add(start, end, label):
        if start is None or end is None or end <= start:
            unresolved.append(label)
            return
        intervals.append((start, end, label))

    # write_le_u16(ram, NAME, ...) and write_le_u32
    for m in re.finditer(r"write_le_u16\s*\(\s*\w+\s*,\s*([A-Za-z0-9_]+)", body):
        a = addr_of(m.group(1), consts)
        add(a, a + 2 if a is not None else None, f"write_le_u16 {m.group(1)}")
    for m in re.finditer(r"write_le_u32\s*\(\s*\w+\s*,\s*([A-Za-z0-9_]+)", body):
        a = addr_of(m.group(1), consts)
        add(a, a + 4 if a is not None else None, f"write_le_u32 {m.group(1)}")

    # helper range writers
    for fn, (count_const, elem) in HELPER_SPANS.items():
        for m in re.finditer(fn + r"\s*\(\s*\w+\s*,\s*([A-Za-z0-9_]+)", body):
            a = addr_of(m.group(1), consts)
            n = addr_of(count_const, consts)
            span = (n * elem) if n is not None else None
            add(a, a + span if (a is not None and span is not None) else None,
                f"{fn} {m.group(1)}")

    # ram[A..B] = / .copy_from_slice  (slice ranges)
    for m in re.finditer(
        r"\w+\s*\[\s*([A-Za-z0-9_]+)\s*\.\.\s*([A-Za-z0-9_]+)\s*([+\-]\s*[A-Za-z0-9_]+)?\s*\]",
        body,
    ):
        a = addr_of(m.group(1), consts)
        b = addr_of(m.group(2), consts)
        if b is not None and m.group(3):
            sign = -1 if m.group(3).strip().startswith("-") else 1
            off = addr_of(m.group(3).strip()[1:].strip(), consts)
            if off is not None:
                b = b + sign * off
            else:
                b = None
        add(a, b, f"slice {m.group(1)}..{m.group(2)}{m.group(3) or ''}")

    # ram[NAME + idx] = ...  (indexed scalar/loop write) -> at least the base byte
    for m in re.finditer(
        r"\w+\s*\[\s*([A-Z][A-Za-z0-9_]*)\s*\+\s*([A-Za-z0-9_]+)\s*\]\s*=", body
    ):
        a = addr_of(m.group(1), consts)
        off = addr_of(m.group(2), consts)  # numeric/const offset if any
        if off is not None:
            add(a, a + off + 1 if a is not None else None,
                f"index {m.group(1)}+{m.group(2)}")
        else:
            # loop variable: record the base byte (range unknown)
            add(a, a + 1 if a is not None else None,
                f"index {m.group(1)}+<var>")

    # ram[NAME] = ...  (plain scalar byte)
    for m in re.finditer(r"\w+\s*\[\s*([A-Z][A-Za-z0-9_]*)\s*\]\s*=", body):
        a = addr_of(m.group(1), consts)
        add(a, a + 1 if a is not None else None, f"byte {m.group(1)}")

    return intervals


# ----------------------------------------------------------------------------
# Mode classification. The master `GameState::write_to_ram` (native.rs) projects
# EVERY state's write_to_ram unconditionally each full sync, in a fixed order, so
# last-writer-wins. An overlap is only a *real clobber bug* when both owners are
# live in the SAME game mode. SNES WRAM legitimately reuses bytes across mutually
# exclusive modes (attract vs gameplay vs menu) — those overlaps are by design.
#
# We tag each struct's mode by name. "CORE" = live during normal gameplay
# (overworld/dungeon/player/sprites/display). Mode-exclusive structs get their
# own tag; CORE-vs-CORE overlaps are the dangerous ones to audit first.
# ----------------------------------------------------------------------------
MODE_TAGS = [
    ("ATTRACT/ENDING", ("Attract", "Ending", "Credit", "PolyRenderer", "Poly")),
    ("MENU/SELECTFILE", ("SelectFile", "SaveLoad", "FileSelect")),
    ("INTRO", ("IntroSword", "Intro")),
    ("MINIGAME", ("Minigame", "Archery", "DiggingGame")),
    ("MAP-SCREEN", ("DungeonMapDisplay", "OverworldMap", "MapZoom", "MapUi")),
]


def mode_of(struct: str) -> str:
    for tag, keys in MODE_TAGS:
        if any(k in struct for k in keys):
            return tag
    return "CORE"


# ----------------------------------------------------------------------------
# 4. Find each `fn write_to_ram` and attribute it to its enclosing struct.
# ----------------------------------------------------------------------------
IMPL_RE = re.compile(r"\bimpl(?:<[^>]*>)?\s+([A-Za-z0-9_]+)")


def brace_body(text: str, open_idx: int) -> str:
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[open_idx : i + 1]
    return text[open_idx:]


def enclosing_struct(text: str, pos: int) -> str:
    last = None
    for m in IMPL_RE.finditer(text, 0, pos):
        last = m.group(1)
    return last or "<unknown>"


# ----------------------------------------------------------------------------
# C array layout (../zelda3/src/variables.h) for the undersized-table lint.
# Array defines look like `((uint8*)(g_ram+0xADDR))`; scalars have a leading
# deref `(*(uint16*)(g_ram+0xADDR))`. A native range-write whose span is smaller
# than the C array's span (distance to the next #define) is likely truncated —
# exactly the class of the presence-table bug (0x200 Vec for a 0x1000 C array).
# ----------------------------------------------------------------------------
C_VARS_H = ROOT.parent / "zelda3" / "src" / "variables.h"
C_ARRAY_RE = re.compile(
    r"#define\s+(\w+)\s+\(\s*\(\s*u?int\d+\s*\*\s*\)\s*\(\s*g_ram\s*\+\s*(0x[0-9A-Fa-f]+)")
C_ANY_RE = re.compile(r"g_ram\s*\+\s*(0x[0-9A-Fa-f]+)")


def c_array_spans():
    """Return {base_addr: (name, span)} for C array (pointer) defines."""
    if not C_VARS_H.exists():
        return {}
    text = C_VARS_H.read_text(errors="replace")
    all_addrs = sorted({int(m.group(1), 16) for m in C_ANY_RE.finditer(text)})
    arrays = {}
    for m in C_ARRAY_RE.finditer(text):
        base = int(m.group(2), 16)
        nxt = next((a for a in all_addrs if a > base), base + 0x10000)
        arrays[base] = (m.group(1), nxt - base)
    return arrays


def report_undersized_tables(owner_intervals):
    arrays = c_array_spans()
    if not arrays:
        print("\n(no variables.h found — skipping undersized-table lint)\n")
        return
    findings = []
    for struct, ivs in owner_intervals.items():
        for (s, e, label, fname) in ivs:
            # Only genuine table projections (slice / helper range-writers) can be
            # "truncated"; a single write_le_u16/byte to an array base is just a
            # member access and is expected to be < the array span.
            if not (label.startswith("slice") or label.startswith("write_scroll")):
                continue
            if s in arrays:
                cname, cspan = arrays[s]
                span = e - s
                if span < cspan:
                    findings.append((s, struct, fname, label, span, cname, cspan))
    print("\n################  UNDERSIZED NATIVE TABLES  ################")
    print("(native range-write narrower than the C array at the same base)\n")
    if not findings:
        print("  none\n")
        return
    for (s, struct, fname, label, span, cname, cspan) in sorted(findings):
        print(f"  0x{s:05x} {struct} [{fname}]: writes 0x{span:x} bytes but C array "
              f"`{cname}` spans 0x{cspan:x}  (via {label})")
    print()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--verbose", action="store_true",
                    help="list writes whose address couldn't be resolved")
    args = ap.parse_args()

    consts = collect_constants()

    # struct -> list of (start, end, label, file)
    owner_intervals: dict[str, list] = defaultdict(list)
    unresolved_all = []

    files = sorted(NATIVE_DIR.glob("*.rs")) + [CRATE_SRC / "game_state" / "native.rs"]
    for path in files:
        text = path.read_text(errors="replace")
        for fnm in re.finditer(r"fn\s+write_to_ram\s*\([^)]*\)\s*\{", text):
            open_idx = text.index("{", fnm.start())
            body = brace_body(text, open_idx)
            struct = enclosing_struct(text, fnm.start())
            unresolved = []
            for (s, e, label) in extract_writes(body, consts, unresolved):
                owner_intervals[struct].append((s, e, label, path.name))
            for u in unresolved:
                unresolved_all.append((struct, path.name, u))

    # Build byte -> set of structs (and remember a representative interval).
    byte_owners: dict[int, dict[str, tuple]] = defaultdict(dict)
    for struct, ivs in owner_intervals.items():
        for (s, e, label, fname) in ivs:
            for addr in range(s, e):
                if struct not in byte_owners[addr]:
                    byte_owners[addr][struct] = (label, fname)

    # Collapse overlapping bytes into contiguous ranges with the same owner-set.
    overlap_bytes = sorted(a for a, owners in byte_owners.items() if len(owners) >= 2)

    print(f"resolved {len(consts)} constants; "
          f"{len(owner_intervals)} structs have write_to_ram; "
          f"{len(overlap_bytes)} overlapping bytes\n")

    if not overlap_bytes:
        print("No dual-ownership overlaps found. ✅")
    else:
        # group contiguous bytes that share the identical owner set
        groups = []
        for addr in overlap_bytes:
            owners = frozenset(byte_owners[addr])
            if groups and groups[-1][1] == owners and addr == groups[-1][0][-1] + 1:
                groups[-1][0].append(addr)
            else:
                groups.append(([addr], owners))

        # Split into HIGH RISK (>=2 owners share the same mode) vs likely-reuse.
        high, reuse = [], []
        for addrs, owners in groups:
            modes = defaultdict(list)
            for s in owners:
                modes[mode_of(s)].append(s)
            same_mode = any(len(v) >= 2 for v in modes.values())
            (high if same_mode else reuse).append((addrs, owners, modes))

        def show(group_list):
            for addrs, owners, modes in group_list:
                lo, hi = addrs[0], addrs[-1]
                rng = f"0x{lo:05x}" if lo == hi else f"0x{lo:05x}-0x{hi:05x}"
                print(f"  {rng}  ({len(addrs)} byte(s)) owned by {len(owners)} structs:")
                for struct in sorted(owners):
                    label, fname = byte_owners[lo].get(
                        struct, byte_owners[addrs[0]][struct])
                    print(f"      - [{mode_of(struct):14s}] {struct:32s} via {label}  [{fname}]")
                print()

        print(f"################  HIGH RISK: {len(high)} same-mode overlap(s)  "
              f"################")
        print("(two states live in the same game mode both project this byte → clobber)\n")
        show(high)

        print(f"################  LIKELY SNES MODE-REUSE: {len(reuse)} cross-mode "
              f"overlap(s)  ################")
        print("(owners live in mutually exclusive modes — by-design byte reuse, "
              "verify if unsure)\n")
        show(reuse)

    report_undersized_tables(owner_intervals)

    if args.verbose and unresolved_all:
        print("\nUnresolved writes (address could not be computed statically):")
        for struct, fname, label in unresolved_all:
            print(f"  {struct} [{fname}]: {label}")

    return 1 if overlap_bytes else 0


if __name__ == "__main__":
    sys.exit(main())
