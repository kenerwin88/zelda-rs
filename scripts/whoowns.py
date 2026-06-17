#!/usr/bin/env python3
"""Given a WRAM address, print everything you need to root-cause a divergence:
  * the OLD-clone (zelda3-rs-old) constant covering it, + offset + next-def span
  * OLD-clone read/write sites for that variable (the parity-correct reference)
  * the Rust constant(s) at that address in THIS repo (game_state/constants.rs)
  * the native *State struct(s) here that load/write it, with file:line

The old Rust clone (commit 1183dee) has perfect parity with the C oracle, so it —
not the C source — is the reference for "what is this address" and "who touches it".
Collapses the address -> semantics -> native-owner chain that every fix repeats.

    python3 scripts/whoowns.py 0x1ea10
    python3 scripts/whoowns.py 0xc198

Override the clone with ZELDA3_OLD_REPO (default: ~/Documents/zelda3-rs-old).
"""
from __future__ import annotations
import pathlib, re, subprocess, sys

# allow `import old_rust_ref` regardless of cwd
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import old_rust_ref as oldref

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

    # 1. OLD-clone constant covering this address (the reference semantics)
    if not oldref.available():
        print(f"OLD clone not found at {oldref.old_repo()} (set ZELDA3_OLD_REPO)")
        cover = None
    else:
        cover = oldref.covering(addr)
        if cover:
            name, base, nxt = cover
            span = (f", next def at 0x{nxt:05x} (=0x{nxt-base:x} wide)"
                    if nxt else "")
            off = "" if addr == base else f" + 0x{addr-base:x}"
            print(f"OLD const:   {name}{off}  (base 0x{base:05x}{span})")
            ds = oldref.def_site(name)
            if ds:
                print(f"  defined:   {ds[0]}:{ds[1]}")
            exact = oldref.names_at(addr)
            extra = [n for n in exact if n != name]
            if extra:
                print(f"  aliases @0x{addr:05x}: {', '.join(extra)}  (SNES byte-reuse)")
        else:
            print("OLD const:   (none found in zelda3-rs-old)")

    # 2. OLD-clone read/write sites for the covering var(s) — parity-correct behavior
    if cover:
        for name in dict.fromkeys([cover[0], *oldref.names_at(addr)]):
            sites = oldref.grep_sites(name, limit=25)
            if sites:
                print(f"\nOLD sites for {name}:")
                for s in sites:
                    print(f"  {s}")
                total = len(oldref.grep_sites(name, limit=10_000))
                if total > len(sites):
                    print(f"  ... +{total-len(sites)} more")

    # 3. Rust constants at this exact address in THIS repo
    consts_rs = CRATE / "game_state" / "constants.rs"
    rust_consts = []
    if consts_rs.exists():
        for m in re.finditer(
                r"const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9a-fA-F]+)\s*;",
                consts_rs.read_text()):
            if int(m.group(2), 16) == addr:
                rust_consts.append(m.group(1))
    print(f"\nThis-repo constant(s) at 0x{addr:05x}: "
          + (", ".join(rust_consts) if rust_consts else "(none exact)"))

    # 4. Native states (this repo) that load/write any constant at this address
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
