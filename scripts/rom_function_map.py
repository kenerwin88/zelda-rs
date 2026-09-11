#!/usr/bin/env python3
"""Map ROM routine addresses to the translated engine's functions.

The C port names every translated function with its ROM address
(`void Sprite_Main() {  // 868328`). This script reads those, then finds the
same-named function (or its snake_case rename) in the Rust engine, and prints
one line per routine: address, C name, Rust file:line or `-`.

usage: rom_function_map.py [--c-src ~/Documents/zelda3/src] [--json out.json] [name-or-address ...]
"""
import argparse
import json
import re
import sys
from pathlib import Path

C_FN = re.compile(r"^[A-Za-z_][\w *]*?\b(\w+)\s*\([^;{]*\)\s*\{\s*//\s*([0-9a-fA-F]{6})\s*$")
RUST_FN = re.compile(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?fn\s+(\w+)\s*[<(]")


def snake(name):
    out = re.sub(r"(?<=[a-z0-9])([A-Z])", r"_\1", name)
    out = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", out)
    return out.lower()


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--c-src", type=Path, default=Path.home() / "Documents/zelda3/src")
    parser.add_argument("--rust-src", type=Path, default=Path("crates/zelda3/src"))
    parser.add_argument("--json", type=Path)
    parser.add_argument("queries", nargs="*")
    args = parser.parse_args()

    c_functions = {}
    for path in sorted(args.c_src.glob("*.c")):
        for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            match = C_FN.match(line)
            if match:
                c_functions[match.group(1)] = (int(match.group(2), 16) & 0x7FFFFF, f"{path.name}:{number}")
    rust_functions = {}
    for path in sorted(args.rust_src.rglob("*.rs")):
        for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            match = RUST_FN.match(line)
            if match:
                rust_functions.setdefault(match.group(1), f"{path.relative_to(args.rust_src)}:{number}")
    rows = []
    for name, (address, c_where) in sorted(c_functions.items(), key=lambda item: item[1][0]):
        rust = rust_functions.get(name) or rust_functions.get(snake(name)) or rust_functions.get(name.lower())
        rows.append({"address": f"{address:06x}", "c_name": name, "c_where": c_where, "rust": rust})
    if args.queries:
        wanted = {query.lower() for query in args.queries}
        rows = [row for row in rows if row["c_name"].lower() in wanted or row["address"] in wanted or snake(row["c_name"]) in wanted]
    if args.json:
        args.json.write_text(json.dumps(rows, indent=1))
    mapped = sum(1 for row in rows if row["rust"])
    for row in rows:
        print(f"{row['address']}  {row['c_name']:48s} {row['rust'] or '-'}")
    print(f"# {len(rows)} C routines with addresses, {mapped} found in the Rust engine", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
