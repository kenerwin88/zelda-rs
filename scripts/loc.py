#!/usr/bin/env python3
"""Count tracked source lines at a git revision, to track cleanup over time.

    scripts/loc.py                 # HEAD
    scripts/loc.py REV [REV ...]   # one row per revision, oldest first as given

Rust is split into production and test files by name (`*_tests.rs`,
`tests.rs`, anything under a `tests/` or `*_tests/` directory); inline
`#[cfg(test)]` modules count as production. Blank lines are excluded.
"""

import subprocess
import sys

KINDS = {".rs": "rust", ".py": "python", ".wgsl": "wgsl", ".sh": "shell"}


def is_test(path):
    parts = path.split("/")
    name = parts[-1]
    return (
        name == "tests.rs"
        or name.endswith("_tests.rs")
        or any(p == "tests" or p.endswith("_tests") for p in parts[:-1])
    )


def count(rev):
    tree = subprocess.run(
        ["git", "ls-tree", "-r", rev], capture_output=True, text=True, check=True
    ).stdout
    blobs = []
    for line in tree.splitlines():
        meta, path = line.split("\t", 1)
        _, kind, sha = meta.split()
        ext = path[path.rfind("."):] if "." in path else ""
        if kind == "blob" and ext in KINDS:
            blobs.append((sha, path, KINDS[ext]))
    out = subprocess.run(
        ["git", "cat-file", "--batch"],
        input="".join(f"{sha}\n" for sha, _, _ in blobs).encode(),
        capture_output=True,
        check=True,
    ).stdout
    totals = {"rust": 0, "rust_test": 0, "python": 0, "wgsl": 0, "shell": 0}
    files = {"rust": 0, "rust_test": 0}
    pos = 0
    for _, path, kind in blobs:
        header_end = out.index(b"\n", pos)
        size = int(out[pos:header_end].split()[2])
        body = out[header_end + 1 : header_end + 1 + size]
        pos = header_end + 1 + size + 1
        lines = sum(1 for l in body.splitlines() if l.strip())
        if kind == "rust" and is_test(path):
            kind = "rust_test"
        totals[kind] += lines
        if kind in files:
            files[kind] += 1
    return totals, files


def main():
    revs = sys.argv[1:] or ["HEAD"]
    print(f"{'rev':<12} {'date':<10} {'rust':>9} {'files':>5} {'rust tests':>10} {'files':>5} {'python':>7} {'wgsl':>6} {'shell':>6} {'total':>9}")
    for rev in revs:
        short, date = subprocess.run(
            ["git", "log", "-1", "--format=%h %cs", rev], capture_output=True, text=True, check=True
        ).stdout.split()
        t, f = count(rev)
        total = sum(t.values())
        print(f"{short:<12} {date:<10} {t['rust']:>9,} {f['rust']:>5} {t['rust_test']:>10,} {f['rust_test']:>5} {t['python']:>7,} {t['wgsl']:>6,} {t['shell']:>6,} {total:>9,}")


if __name__ == "__main__":
    main()
