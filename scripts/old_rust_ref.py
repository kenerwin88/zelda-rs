#!/usr/bin/env python3
"""Shared reference data from the known-good OLD Rust clone (`zelda3-rs-old`).

The old clone at commit 1183dee has perfect parity with the C oracle, so it — not
the C source — is the authoritative reference for "what is this WRAM address" and
"who reads/writes it". The old clone predates the native-state migration, so every
WRAM variable is a `const NAME: usize = 0xADDR;` accessed as `ram[NAME]`; that gives
a clean address->name map plus parity-correct read/write sites.

Override the clone location with ZELDA3_OLD_REPO (default: ~/Documents/zelda3-rs-old).

Used by whoowns.py and find_dual_ownership.py. Run directly to dump the map:
    python3 scripts/old_rust_ref.py 0x45c
"""
from __future__ import annotations
import functools, os, pathlib, re, subprocess, sys

DEFAULT_OLD = pathlib.Path.home() / "Documents" / "zelda3-rs-old"


def old_repo() -> pathlib.Path:
    return pathlib.Path(os.environ.get("ZELDA3_OLD_REPO", str(DEFAULT_OLD)))


def old_src() -> pathlib.Path:
    return old_repo() / "crates" / "zelda3" / "src"


def available() -> bool:
    return old_src().is_dir()


CONST_RE = re.compile(
    r"(?:pub\s+)?const\s+([A-Z_][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9A-Fa-f]+)\s*;")

# Contexts that prove a constant is used as a WRAM index (filters out size consts
# like SRAM_SIZE / VRAM_WORDS that share the const-usize-hex shape).
RAM_INDEX_RES = [
    re.compile(r"ram\s*\[\s*([A-Z_][A-Z0-9_]*)"),
    re.compile(
        r"(?:read_le_u\d+|write_le_u\d+|ram_byte|load24|read_word_from_slice)"
        r"\s*\(\s*(?:&\s*mut\s+|&\s+)?[\w.]*ram[\w.]*\s*,\s*([A-Z_][A-Z0-9_]*)"),
]


@functools.lru_cache(maxsize=1)
def _scan():
    """Return (defs, ram_names).

    defs: list of (name, addr, relpath, line) for every usize=0x const.
    ram_names: set of names that appear in a WRAM-index context.
    """
    defs = []
    ram_names: set[str] = set()
    src = old_src()
    if not src.is_dir():
        return defs, ram_names
    for f in sorted(src.glob("*.rs")):
        text = f.read_text(errors="replace")
        rel = str(f.relative_to(old_repo()))
        for i, line in enumerate(text.splitlines(), 1):
            m = CONST_RE.search(line)
            if m:
                defs.append((m.group(1), int(m.group(2), 16), rel, i))
        for rx in RAM_INDEX_RES:
            for m in rx.finditer(text):
                ram_names.add(m.group(1))
    return defs, ram_names


def address_defs():
    """WRAM address constants only (ram-indexed), sorted by address.

    Returns list of (name, addr). A constant is treated as a WRAM address if it is
    used as a ram index *somewhere*, or (fallback) its value is below the 128 KiB
    WRAM size and it is not an obvious size/count constant.
    """
    defs, ram_names = _scan()
    out = []
    seen = set()
    for name, addr, _rel, _line in defs:
        if name in seen:
            continue
        is_addr = name in ram_names or (
            addr < 0x20000 and not re.search(r"_(SIZE|COUNT|WORDS|BYTES|LEN|MAX|SLOTS)$", name))
        if is_addr:
            out.append((name, addr))
            seen.add(name)
    return sorted(out, key=lambda x: x[1])


def names_at(addr: int):
    """All ram-indexed constant names whose address == addr."""
    return [n for (n, a) in address_defs() if a == addr]


def covering(addr: int):
    """(name, base, next_addr) for the nearest ram-index const at/below addr."""
    defs = address_defs()
    cover = [(n, a) for (n, a) in defs if a <= addr]
    if not cover:
        return None
    name, base = cover[-1]
    nxt = next((a for (_, a) in defs if a > base), None)
    return name, base, nxt


def def_site(name: str):
    """(relpath, line) where OLD defines `const name`."""
    defs, _ = _scan()
    for n, _a, rel, line in defs:
        if n == name:
            return rel, line
    return None


def grep_sites(name: str, limit: int = 40):
    """OLD-clone source lines referencing `name` (the parity-correct read/write sites)."""
    src = old_src()
    if not src.is_dir():
        return []
    try:
        r = subprocess.run(
            ["grep", "-rn", "-E", rf"\b{re.escape(name)}\b",
             *[str(p) for p in sorted(src.glob("*.rs"))]],
            capture_output=True, text=True)
    except Exception:
        return []
    rel = []
    for line in r.stdout.splitlines():
        if not line.strip():
            continue
        # strip the const definition line itself (kept separately as def_site)
        if re.search(rf"const\s+{re.escape(name)}\s*:", line):
            continue
        rel.append(line.replace(str(old_repo()) + "/", ""))
    return rel[:limit]


def main():
    if len(sys.argv) < 2:
        sys.exit("usage: old_rust_ref.py <addr>")
    if not available():
        sys.exit(f"OLD clone not found at {old_repo()} (set ZELDA3_OLD_REPO)")
    addr = int(sys.argv[1], 0)
    cov = covering(addr)
    if cov:
        name, base, nxt = cov
        span = f", next def 0x{nxt:05x} (=0x{nxt-base:x} wide)" if nxt else ""
        off = "" if addr == base else f" + 0x{addr-base:x}"
        print(f"OLD const: {name}{off}  (base 0x{base:05x}{span})")
    exact = names_at(addr)
    if exact:
        print("exact:", ", ".join(exact))
    for n in (exact or ([cov[0]] if cov else [])):
        ds = def_site(n)
        print(f"\n{n}" + (f"  (defined {ds[0]}:{ds[1]})" if ds else ""))
        for s in grep_sites(n, 15):
            print(f"  {s}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
