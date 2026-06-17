#!/usr/bin/env python3
"""Given a WRAM address, print everything you need to root-cause a divergence:
  * the C variable (#define in ../zelda3/src/variables.h) covering it, + offset
  * C read/write sites for that variable
  * the Rust constant(s) at that address (game_state/constants.rs)
  * the native *State struct(s) that load/write it, with file:line

Collapses the address -> C-semantics -> native-owner grep chain that every fix
in this migration repeats.

    python3 scripts/whoowns.py 0x1ea10
    python3 scripts/whoowns.py 0xc198
"""
from __future__ import annotations
import pathlib, re, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates" / "zelda3" / "src"
CSRC = ROOT.parent / "zelda3"
VARS_H = CSRC / "src" / "variables.h"

DEF_RE = re.compile(
    r"#define\s+(\w+)\s+\(?\(*\s*(?:\*\s*)?\(\s*u?int\d+\s*\*\s*\)\s*\(\s*g_ram\s*\+\s*(0x[0-9A-Fa-f]+)")


def c_defines():
    out = []
    if VARS_H.exists():
        for m in DEF_RE.finditer(VARS_H.read_text(errors="replace")):
            out.append((m.group(1), int(m.group(2), 16)))
    return sorted(out, key=lambda x: x[1])


def grep(pattern, paths, flags="-rn"):
    try:
        r = subprocess.run(["grep", flags, "-E", pattern, *[str(p) for p in paths]],
                           capture_output=True, text=True)
        return [l for l in r.stdout.splitlines() if l.strip()]
    except Exception:
        return []


def main():
    if len(sys.argv) < 2:
        sys.exit("usage: whoowns.py <addr>")
    addr = int(sys.argv[1], 0)
    print(f"=== 0x{addr:05x} ===\n")

    # 1. C variable covering this address
    defs = c_defines()
    cover = [(n, a) for (n, a) in defs if a <= addr]
    cname = None
    if cover:
        cname, cbase = cover[-1]
        off = addr - cbase
        nxt = next((a for (_, a) in defs if a > cbase), None)
        span = f", next def at 0x{nxt:05x} (=0x{nxt-cbase:x} wide)" if nxt else ""
        exact = "" if off == 0 else f" + 0x{off:x}"
        print(f"C variable:  {cname}{exact}  (base 0x{cbase:05x}{span})")
    else:
        print("C variable:  (none found in variables.h)")

    # 2. C read/write sites
    if cname:
        sites = grep(rf"\b{re.escape(cname)}\b", sorted(CSRC.glob("src/*.c")))
        if sites:
            print("\nC sites:")
            for s in sites[:25]:
                rel = s.split("/zelda3/", 1)[-1]
                print(f"  {rel}")
            if len(sites) > 25:
                print(f"  ... +{len(sites)-25} more")

    # 3. Rust constants at this exact address
    consts_rs = CRATE / "game_state" / "constants.rs"
    rust_consts = []
    if consts_rs.exists():
        for m in re.finditer(r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9a-fA-F]+)\s*;",
                             consts_rs.read_text()):
            if int(m.group(2), 16) == addr:
                rust_consts.append(m.group(1))
    print(f"\nRust constant(s) at 0x{addr:05x}: "
          + (", ".join(rust_consts) if rust_consts else "(none exact)"))

    # 4. Native states that load/write any constant at this address
    if rust_consts:
        native = CRATE / "game_state" / "native"
        files = sorted(native.glob("*.rs")) + [CRATE / "game_state" / "native.rs"]
        for c in rust_consts:
            hits = grep(rf"\b{c}\b", files)
            owners = [h for h in hits if "read_le" in h or "ram[" in h or "write_le" in h
                      or ".copy_from_slice" in h or "ram_byte" in h or ".fill(" in h]
            if owners:
                print(f"\nNative load/write of {c}:")
                for h in owners[:20]:
                    rel = h.split("/src/", 1)[-1]
                    print(f"  {rel}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
