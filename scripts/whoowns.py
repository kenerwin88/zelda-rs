#!/usr/bin/env python3
"""Given a WRAM address, print everything you need to root-cause a divergence:
  * the constant covering it, + offset + next-def span (this repo's address map)
  * read/write sites for that variable in this repo
  * the native *State struct(s) here that load/write it, with file:line

The address map is scanned from THIS repo's `crates/zelda3/src/*.rs`
(`const NAME: usize = 0xADDR;`). Pair it with `zparity drill <frame>`: drill
gives you a diverging WRAM page/address, whoowns tells you what that address is
and who touches it.

    python3 scripts/whoowns.py 0x1ea10
    python3 scripts/whoowns.py 0xcf8
"""
from __future__ import annotations
import pathlib, subprocess, sys

# allow `import ram_ref` regardless of cwd
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ram_ref as ref

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates" / "zelda3" / "src"


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

    # 1. Constant covering this address (the reference semantics)
    cover = ref.covering(addr)
    if cover:
        name, base, nxt = cover
        span = (f", next def at 0x{nxt:05x} (=0x{nxt-base:x} wide)" if nxt else "")
        off = "" if addr == base else f" + 0x{addr-base:x}"
        print(f"const:   {name}{off}  (base 0x{base:05x}{span})")
        ds = ref.def_site(name)
        if ds:
            print(f"  defined:   {ds[0]}:{ds[1]}")
        exact = ref.names_at(addr)
        extra = [n for n in exact if n != name]
        if extra:
            print(f"  aliases @0x{addr:05x}: {', '.join(extra)}  (SNES byte-reuse)")
    else:
        print("const:   (none found at/below this address)")

    # 2. Read/write sites for the covering var(s)
    if cover:
        for name in dict.fromkeys([cover[0], *ref.names_at(addr)]):
            sites = ref.grep_sites(name, limit=25)
            if sites:
                print(f"\nsites for {name}:")
                for s in sites:
                    print(f"  {s}")
                total = len(ref.grep_sites(name, limit=10_000))
                if total > len(sites):
                    print(f"  ... +{total-len(sites)} more")

    # 3. Native states (this repo) that load/write any constant at this address
    names_here = ref.names_at(addr)
    print(f"\nConstant(s) at 0x{addr:05x}: "
          + (", ".join(names_here) if names_here else "(none exact)"))
    if names_here:
        native = CRATE / "game_state" / "native"
        files = sorted(native.glob("*.rs")) + [CRATE / "game_state" / "native.rs"]
        for c in names_here:
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
