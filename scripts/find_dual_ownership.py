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

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ram_ref as ref

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
# Reference array layout from this repo's WRAM constant map for the
# undersized-table lint. Each `const NAME: usize = 0xADDR;` marks a variable's
# base; a variable's span = distance to the next constant. A native range-write
# narrower than that span is likely truncated — exactly the class of the
# presence-table bug (0x200 Vec for a 0x1000 array).
# ----------------------------------------------------------------------------
def ref_array_spans():
    """Return {base_addr: (name, span)} from this repo's address map."""
    defs = ref.address_defs()
    if not defs:
        return {}
    spans = {}
    for i, (name, base) in enumerate(defs):
        nxt = defs[i + 1][1] if i + 1 < len(defs) else base + 0x10000
        # keep the first name seen at a base (mode-reuse aliases share an addr)
        spans.setdefault(base, (name, nxt - base))
    return spans


def report_clear_coherence(byte_owners, consts):
    """Audit `clear_room_parser_words(&[ADDRS])`-style bulk RAM clears.

    That bridge zeroes RAM and reloads ONLY DungeonRoomParserState. Any cleared
    address actually owned (in write_to_ram) by a DIFFERENT native state keeps a
    STALE native field, which then re-projects over the just-cleared RAM at frame
    end — the misc_object_index(0x42c) / cur_door_idx(0x460) class of bug that the
    write_to_ram-overlap scan can't see (the clear is a direct ram[] write, not a
    write_to_ram). Reports each cleared address's real owner.
    """
    src = (CRATE_SRC / "dungeon.rs").read_text(errors="replace")
    print("\n################  ROOM-LOAD CLEAR COHERENCE  ################")
    print("(addresses cleared via clear_room_parser_words but owned by another "
          "native state → stale field re-projects)\n")
    # find the literal address array(s) feeding clear_room_parser_words
    flagged = False
    for m in re.finditer(r"for\s+&offset\s+in\s+&\[([^\]]*)\]", src):
        if "clear_room_parser_words" not in src[m.end():m.end() + 200]:
            continue
        addrs = [parse_int(t) for t in m.group(1).split(",")]
        addrs = [a for a in addrs if a is not None]
        for a in addrs:
            owners = byte_owners.get(a, {})
            non_parser = {s: v for s, v in owners.items()
                          if "RoomParser" not in s}
            if non_parser:
                flagged = True
                for s, (label, fname) in non_parser.items():
                    print(f"  0x{a:05x} cleared via clear_room_parser_words but "
                          f"owned by {s} ({label}) [{fname}]")
    if not flagged:
        print("  none (all cleared addresses owned by the room parser)\n")
    else:
        print()


def collect_tuple_array_ranges(consts: dict[str, int]) -> dict[str, list[tuple[int, int]]]:
    """Resolve `const NAME: ... = [(BASE, COUNT), ...];` to [(base_addr, count)] ranges.
    Captures the per-slot field-range tables (e.g. SPRITE_SLOTS_FIELD_RANGES) that a
    state's write_to_ram iterates via a helper — opaque to the per-write parser, so the
    owner map would otherwise not know that state owns those bytes."""
    out: dict[str, list[tuple[int, int]]] = {}
    decl_re = re.compile(
        r"const\s+([A-Z][A-Z0-9_]*)\s*:[^=]*=\s*&?\s*\[(.*?)\]\s*;", re.DOTALL)
    pair_re = re.compile(r"\(\s*([A-Za-z0-9_]+)\s*,\s*([A-Za-z0-9_]+)\s*\)")
    for path in CRATE_SRC.rglob("*.rs"):
        for m in decl_re.finditer(path.read_text(errors="replace")):
            pairs = []
            for pm in pair_re.finditer(m.group(2)):
                base = addr_of(pm.group(1), consts)
                count = addr_of(pm.group(2), consts)
                if base is not None and count is not None:
                    pairs.append((base, count))
            if pairs:
                out.setdefault(m.group(1), pairs)
    return out


def collect_array_constants(consts: dict[str, int]) -> dict[str, list[int]]:
    """Resolve `const NAME: [usize; N] = [A, B, ...];` element addresses (for the
    field-array RAM copies like CACHED_SPRITE_LIVE_FIELDS / CACHED_SPRITE_ALT_FIELDS)."""
    arrays: dict[str, list[int]] = {}
    arr_re = re.compile(
        r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*\[\s*usize\s*;[^\]]*\]\s*=\s*\[([^\]]*)\]")
    for path in CRATE_SRC.rglob("*.rs"):
        for m in arr_re.finditer(path.read_text(errors="replace")):
            elems = [e.strip() for e in m.group(2).split(",") if e.strip()]
            addrs = [addr_of(e, consts) for e in elems]
            arrays.setdefault(m.group(1), [a for a in addrs if a is not None])
    return arrays


BRIDGE_STRUCT_RE = re.compile(
    r"struct\s+(Native\w*BridgeMut)\s*<[^>]*>\s*\{([^}]*)\}")
# fields like `state: &'a mut FooState,` / `sprite_slots: &'a mut SpriteSlotsState,`
STATE_FIELD_RE = re.compile(r"&\s*'?\w*\s*mut\s+([A-Za-z0-9_]+State)\b")


def collect_containment() -> dict[str, set]:
    """Map each composite `*State` struct to the `*State` sub-states it owns by value,
    so a bridge holding a parent (e.g. DisplayState) counts as holding its leaves
    (PaletteBufferState, ...). Lets the foreign-write lint avoid flagging a bridge that
    writes a leaf it transitively models."""
    cont: dict[str, set] = defaultdict(set)
    struct_re = re.compile(r"struct\s+([A-Za-z0-9_]+State)\s*\{([^}]*)\}")
    field_re = re.compile(r":\s*\[?\s*([A-Za-z0-9_]+State)\b")
    for path in NATIVE_DIR.glob("*.rs"):
        text = path.read_text(errors="replace")
        for m in struct_re.finditer(text):
            for ft in field_re.findall(m.group(2)):
                if ft != m.group(1):
                    cont[m.group(1)].add(ft)
    return cont


def expand_held(held: set, cont: dict[str, set]) -> set:
    """Transitively add every sub-state contained in the held composite states."""
    out, stack = set(held), list(held)
    while stack:
        cur = stack.pop()
        for child in cont.get(cur, ()):
            if child not in out:
                out.add(child)
                stack.append(child)
    return out


def report_bridge_foreign_writes(byte_owners, consts, arrays):
    """Flag a *Bridge* method that writes a WRAM address owned (in write_to_ram) by a
    native state the bridge does NOT hold. Such a write updates RAM but leaves that
    other state's native model stale, so a later native read sees the wrong value —
    the cached-sprite-uncache / native↔RAM coherence bug class. The fix is to resync
    the foreign owner from RAM after the write (or route through its own bridge)."""
    print("\n################  BRIDGE WRITES IT DOESN'T MODEL  ################")
    print("(a bridge writes RAM owned by a native state it doesn't hold → that state's "
          "native model goes stale unless resynced)\n")
    findings = []
    seen = set()
    cont = collect_containment()
    lhs_array_re = re.compile(r"\bram\s*\[\s*([A-Z][A-Z0-9_]*)\s*\[[^=]*=")
    for path in sorted(NATIVE_DIR.glob("*.rs")):
        text = path.read_text(errors="replace")
        bridges = {}
        for m in BRIDGE_STRUCT_RE.finditer(text):
            bridges[m.group(1)] = expand_held(
                set(STATE_FIELD_RE.findall(m.group(2))), cont)
        for im in re.finditer(r"impl(?:<[^>]*>)?\s+(Native\w*BridgeMut)\b", text):
            bname = im.group(1)
            if bname not in bridges:
                continue
            held = bridges[bname]
            impl_body = brace_body(text, text.index("{", im.end()))
            for fm in re.finditer(r"fn\s+(\w+)\s*\(", impl_body):
                mopen = impl_body.find("{", fm.end())
                if mopen < 0:
                    continue
                mbody = brace_body(impl_body, mopen)
                written = []
                for (s, _e, label) in extract_writes(mbody, consts, []):
                    written.append((s, label))
                for am in lhs_array_re.finditer(mbody):  # ram[ARRAY[..]..] = ...
                    for a in arrays.get(am.group(1), []):
                        written.append((a, f"array {am.group(1)}[]"))
                for addr, label in written:
                    owners = set(byte_owners.get(addr, {}))
                    if owners and not (owners & held):
                        for owner in sorted(owners):
                            key = (bname, fm.group(1), owner)
                            if key not in seen:
                                seen.add(key)
                                findings.append(
                                    (bname, fm.group(1), addr, owner, label, path.name))
    if not findings:
        print("  none\n")
        return
    for (bname, method, addr, owner, label, fname) in sorted(findings, key=lambda x: x[0]):
        print(f"  {bname}::{method} writes 0x{addr:05x} (via {label}) owned by {owner}; "
              f"bridge does not hold {owner}  [{fname}]")
    print()


def report_undersized_tables(owner_intervals):
    arrays = ref_array_spans()
    if not arrays:
        print("\n(no WRAM constant map found — skipping undersized-table lint)\n")
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
    print("(native range-write narrower than the const-map array span at the same base)\n")
    if not findings:
        print("  none\n")
        return
    for (s, struct, fname, label, span, cname, cspan) in sorted(findings):
        print(f"  0x{s:05x} {struct} [{fname}]: writes 0x{span:x} bytes but array "
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
    structs_with_write_to_ram: set[str] = set()

    files = sorted(NATIVE_DIR.glob("*.rs")) + [CRATE_SRC / "game_state" / "native.rs"]
    for path in files:
        text = path.read_text(errors="replace")
        for fnm in re.finditer(r"fn\s+write_to_ram\s*\([^)]*\)\s*\{", text):
            open_idx = text.index("{", fnm.start())
            body = brace_body(text, open_idx)
            struct = enclosing_struct(text, fnm.start())
            structs_with_write_to_ram.add(struct)
            unresolved = []
            for (s, e, label) in extract_writes(body, consts, unresolved):
                owner_intervals[struct].append((s, e, label, path.name))
            for u in unresolved:
                unresolved_all.append((struct, path.name, u))

    # Second pass: attribute per-slot field-range tables (tuple arrays) to any struct
    # whose impl references one — covers write_to_ram that iterates ranges via a helper
    # (e.g. SpriteSlotsState::field_offsets -> SPRITE_SLOTS_FIELD_RANGES), which the
    # per-write parser above can't see.
    tuple_arrays = collect_tuple_array_ranges(consts)
    if tuple_arrays:
        projected = structs_with_write_to_ram
        for path in files:
            text = path.read_text(errors="replace")
            for im in re.finditer(r"\bimpl(?:<[^>]*>)?\s+([A-Za-z0-9_]+)", text):
                struct = im.group(1)
                if struct not in projected:
                    continue
                impl_body = brace_body(text, text.index("{", im.end()))
                for name, ranges in tuple_arrays.items():
                    if re.search(rf"\b{name}\b", impl_body):
                        for (base, count) in ranges:
                            owner_intervals[struct].append(
                                (base, base + count, f"field-range {name}", path.name))

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

    report_clear_coherence(byte_owners, consts)
    report_bridge_foreign_writes(byte_owners, consts, collect_array_constants(consts))
    report_undersized_tables(owner_intervals)

    if args.verbose and unresolved_all:
        print("\nUnresolved writes (address could not be computed statically):")
        for struct, fname, label in unresolved_all:
            print(f"  {struct} [{fname}]: {label}")

    return 1 if overlap_bytes else 0


if __name__ == "__main__":
    sys.exit(main())
