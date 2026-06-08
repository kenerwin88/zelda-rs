#!/usr/bin/env python3
"""Rename a RAM constant and run the standard documentation/check workflow."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"
WEAK_DOC = REPO_ROOT / "docs" / "ram-weak-names.md"
WEAK_COUNT_RE = re.compile(r"wrote docs/ram-weak-names\.md \((\d+) weak-name warning\(s\)\)")
ORACLE_DIGEST_RE = re.compile(r"WRAM fnv1a64 = ([0-9a-f]+)")


def run(command: list[str]) -> str:
    print("+ " + " ".join(command))
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.stdout:
        print(completed.stdout, end="")
    return completed.stdout


def replace_identifier(old: str, new: str, dry_run: bool) -> list[Path]:
    pattern = re.compile(rf"\b{re.escape(old)}\b")
    changed: list[Path] = []
    for path in sorted(SRC_ROOT.glob("*.rs")):
        text = path.read_text()
        updated = pattern.sub(new, text)
        if updated == text:
            continue
        changed.append(path)
        if not dry_run:
            path.write_text(updated)
    return changed


def weak_count_from_doc() -> int | None:
    if not WEAK_DOC.exists():
        return None
    count = 0
    for line in WEAK_DOC.read_text().splitlines():
        if line.startswith("| `crates/"):
            count += 1
    return count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("old", help="old Rust RAM constant name")
    parser.add_argument("new", help="new Rust RAM constant name")
    parser.add_argument("--dry-run", action="store_true", help="show changed files without writing")
    parser.add_argument("--oracle", action="store_true", help="also run focused file-select-enter-game oracle")
    args = parser.parse_args()

    if not re.fullmatch(r"[A-Z][A-Z0-9_]*", args.old):
        parser.error("old name must look like an uppercase Rust constant")
    if not re.fullmatch(r"[A-Z][A-Z0-9_]*", args.new):
        parser.error("new name must look like an uppercase Rust constant")

    weak_before = weak_count_from_doc()
    changed = replace_identifier(args.old, args.new, args.dry_run)
    if not changed:
        print(f"no Rust source references found for {args.old}")
        return 1

    for path in changed:
        print(path.relative_to(REPO_ROOT))
    if args.dry_run:
        return 0

    command_outputs: list[str] = []
    command_outputs.append(run(["cargo", "fmt", "-p", "zelda3"]))
    run(["python3", "scripts/mine_nes_ver2_symbols.py", "--write"])
    command_outputs.append(
        run(
            [
                "python3",
                "scripts/check_ram_readability.py",
                "--write-doc",
                "--doc",
                "docs/ram-map.md",
                "--write-weak-doc",
                "--weak-doc",
                "docs/ram-weak-names.md",
                "--write-candidate-doc",
                "--candidate-doc",
                "docs/ram-source-backed-candidates.md",
            ]
        )
    )
    command_outputs.append(run(["python3", "scripts/check_ram_readability.py"]))
    command_outputs.append(run(["cargo", "check", "-p", "zelda3-bin"]))
    if args.oracle:
        command_outputs.append(run(["python3", "scripts/oracle_windows.py", "--run", "--only", "file-select-enter-game"]))

    weak_after = weak_count_from_doc()
    combined_output = "\n".join(command_outputs)
    oracle_digest = None
    digest_match = ORACLE_DIGEST_RE.search(combined_output)
    if digest_match:
        oracle_digest = digest_match.group(1)

    print("")
    print("rename proof:")
    print(f"- rename: {args.old} -> {args.new}")
    print("- changed files: " + ", ".join(str(path.relative_to(REPO_ROOT)) for path in changed))
    if weak_before is not None and weak_after is not None:
        print(f"- weak-name backlog: {weak_before} -> {weak_after}")
    if oracle_digest:
        print(f"- oracle WRAM fnv1a64: {oracle_digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
