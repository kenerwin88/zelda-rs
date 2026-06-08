#!/usr/bin/env python3
"""Report zelda3 C-to-Rust porting status.

The status TSV is intentionally small and hand-edited. This script scans the
upstream C tree to keep the "everything left" map current without maintaining a
huge checklist by hand.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = Path(os.environ.get("ZELDA3_C_SRC", str(REPO_ROOT.parent / "zelda3" / "src")))
STATUS_PATH = REPO_ROOT / "docs" / "porting" / "status.tsv"
ALLOWED_KINDS = {"module", "function", "table"}
ALLOWED_STATUSES = {"done", "partial", "seed", "stub", "deferred", "not-started"}

FUNC_RE = re.compile(
    r"^(?:static\s+)?(?:inline\s+)?[A-Za-z_][A-Za-z0-9_\s\*]*?\s+\*?\s*"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\([^;]*\)\s*\{"
)


@dataclass(frozen=True)
class Status:
    kind: str
    source: str
    symbol: str
    status: str
    rust: str
    notes: str


@dataclass(frozen=True)
class CFunction:
    source: str
    line: int
    name: str


def load_statuses(path: Path) -> tuple[dict[str, Status], dict[tuple[str, str], Status]]:
    module_status: dict[str, Status] = {}
    symbol_status: dict[tuple[str, str], Status] = {}
    with path.open(newline="") as fh:
        for row in csv.DictReader(fh, delimiter="\t"):
            status = Status(
                kind=row["kind"],
                source=row["source"],
                symbol=row["symbol"],
                status=row["status"],
                rust=row["rust"],
                notes=row["notes"],
            )
            if status.kind == "module":
                module_status[status.source] = status
            elif status.kind in {"function", "table"}:
                symbol_status[(status.source, status.symbol)] = status
    return module_status, symbol_status


def validate_statuses(
    path: Path,
    c_root: Path,
    module_status: dict[str, Status],
    symbol_status: dict[tuple[str, str], Status],
    functions: list[CFunction],
) -> list[str]:
    errors: list[str] = []
    c_functions = {(function.source, function.name) for function in functions}
    seen: set[tuple[str, str, str]] = set()

    with path.open(newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        expected = ["kind", "source", "symbol", "status", "rust", "notes"]
        if reader.fieldnames != expected:
            errors.append(f"{path}: header must be {'/'.join(expected)}")
            return errors

        for rownum, row in enumerate(reader, 2):
            kind = row["kind"]
            source = row["source"]
            symbol = row["symbol"]
            status = row["status"]
            rust = row["rust"]

            key = (kind, source, symbol)
            if key in seen:
                errors.append(f"{path}:{rownum}: duplicate row for {kind} {source}:{symbol}")
            seen.add(key)

            if kind not in ALLOWED_KINDS:
                errors.append(f"{path}:{rownum}: unknown kind {kind!r}")
            if status not in ALLOWED_STATUSES:
                errors.append(f"{path}:{rownum}: unknown status {status!r}")
            if not source:
                errors.append(f"{path}:{rownum}: source is required")
            elif not (c_root / source).exists():
                errors.append(f"{path}:{rownum}: source {source!r} does not exist under {c_root}")
            if kind == "module" and symbol != "*":
                errors.append(f"{path}:{rownum}: module rows must use symbol '*'")
            if kind == "function" and source.endswith(".c") and (source, symbol) not in c_functions:
                errors.append(f"{path}:{rownum}: function {source}:{symbol} was not found in C source")
            if kind == "table" and source and symbol and (c_root / source).exists():
                source_text = (c_root / source).read_text(errors="replace")
                if symbol not in source_text:
                    errors.append(f"{path}:{rownum}: table {source}:{symbol} was not found in C source")
            if rust and rust != "TBD":
                rust_path = rust.split(":", 1)[0]
                if not (REPO_ROOT / rust_path).exists():
                    errors.append(f"{path}:{rownum}: rust target {rust_path!r} does not exist")

    for source, status in module_status.items():
        if status.kind != "module":
            errors.append(f"internal error: module_status contains non-module row {source}")
    for (source, symbol), status in symbol_status.items():
        if status.kind not in {"function", "table"}:
            errors.append(f"internal error: symbol_status contains {status.kind} row {source}:{symbol}")

    return errors


def iter_c_files(c_root: Path) -> list[Path]:
    return sorted(c_root.glob("*.c"))


def scan_functions(c_root: Path) -> list[CFunction]:
    functions: list[CFunction] = []
    for path in iter_c_files(c_root):
        with path.open(errors="replace") as fh:
            for lineno, line in enumerate(fh, 1):
                match = FUNC_RE.match(line)
                if match:
                    functions.append(CFunction(path.name, lineno, match.group("name")))
    return functions


def c_line_counts(c_root: Path) -> dict[str, int]:
    counts: dict[str, int] = {}
    for path in iter_c_files(c_root):
        with path.open(errors="replace") as fh:
            counts[path.name] = sum(1 for _ in fh)
    return counts


def module_rollup(
    functions: list[CFunction], module_status: dict[str, Status], line_counts: dict[str, int]
) -> list[dict[str, object]]:
    by_source: dict[str, int] = {}
    for function in functions:
        by_source[function.source] = by_source.get(function.source, 0) + 1
    rows = []
    for source in sorted(line_counts):
        status = module_status.get(source)
        rows.append(
            {
                "source": source,
                "lines": line_counts[source],
                "functions": by_source.get(source, 0),
                "status": status.status if status else "not-started",
                "rust": status.rust if status else "",
                "notes": status.notes if status else "",
            }
        )
    return rows


def print_markdown(
    module_rows: list[dict[str, object]],
    functions: list[CFunction],
    symbol_status: dict[tuple[str, str], Status],
    list_functions: bool,
) -> None:
    totals: dict[str, int] = {}
    for row in module_rows:
        totals[row["status"]] = totals.get(row["status"], 0) + int(row["functions"])

    print("# Porting Inventory\n")
    print("| Status | Approx functions |")
    print("|---|---:|")
    for status in ["done", "partial", "seed", "stub", "deferred", "not-started"]:
        if status in totals:
            print(f"| `{status}` | {totals[status]} |")

    print("\n## Modules\n")
    print("| C file | Lines | Approx funcs | Status | Rust target |")
    print("|---|---:|---:|---|---|")
    for row in module_rows:
        print(
            f"| `{row['source']}` | {row['lines']} | {row['functions']} | "
            f"`{row['status']}` | {row['rust']} |"
        )

    tracked = [status for status in symbol_status.values()]
    if tracked:
        print("\n## Tracked Symbols\n")
        print("| C symbol | Status | Rust target | Notes |")
        print("|---|---|---|---|")
        for status in sorted(tracked, key=lambda item: (item.source, item.symbol)):
            print(
                f"| `{status.source}:{status.symbol}` | `{status.status}` | "
                f"{status.rust} | {status.notes} |"
            )

    if list_functions:
        print("\n## Full Function List\n")
        print("| C symbol | Status | Rust target |")
        print("|---|---|---|")
        for function in functions:
            status = symbol_status.get((function.source, function.name))
            label = status.status if status else "not-started"
            rust = status.rust if status else ""
            print(f"| `{function.source}:{function.line}:{function.name}` | `{label}` | {rust} |")


def print_tsv(
    module_rows: list[dict[str, object]],
    functions: list[CFunction],
    symbol_status: dict[tuple[str, str], Status],
    list_functions: bool,
) -> None:
    print("kind\tsource\tline\tsymbol\tstatus\trust\tnotes")
    for row in module_rows:
        print(
            f"module\t{row['source']}\t\t*\t{row['status']}\t{row['rust']}\t{row['notes']}"
        )
    if list_functions:
        for function in functions:
            status = symbol_status.get((function.source, function.name))
            print(
                "function\t"
                f"{function.source}\t{function.line}\t{function.name}\t"
                f"{status.status if status else 'not-started'}\t"
                f"{status.rust if status else ''}\t{status.notes if status else ''}"
            )


def status_totals(
    module_rows: list[dict[str, object]],
    functions: list[CFunction],
    symbol_status: dict[tuple[str, str], Status],
) -> dict[str, object]:
    module_status_counts: dict[str, int] = {}
    approximate_function_counts: dict[str, int] = {}
    tracked_symbol_counts: dict[str, int] = {}
    tracked_function_counts: dict[str, int] = {}
    tracked_table_counts: dict[str, int] = {}
    untracked_by_source: dict[str, int] = {}
    c_function_keys = {(function.source, function.name) for function in functions}

    for row in module_rows:
        status = str(row["status"])
        module_status_counts[status] = module_status_counts.get(status, 0) + 1
        approximate_function_counts[status] = (
            approximate_function_counts.get(status, 0) + int(row["functions"])
        )

    for status in symbol_status.values():
        tracked_symbol_counts[status.status] = tracked_symbol_counts.get(status.status, 0) + 1
        if status.kind == "function" and (status.source, status.symbol) in c_function_keys:
            tracked_function_counts[status.status] = (
                tracked_function_counts.get(status.status, 0) + 1
            )
        elif status.kind == "table":
            tracked_table_counts[status.status] = tracked_table_counts.get(status.status, 0) + 1

    for function in functions:
        if (function.source, function.name) not in symbol_status:
            untracked_by_source[function.source] = untracked_by_source.get(function.source, 0) + 1

    total_functions = len(functions)
    explicit_done_functions = tracked_function_counts.get("done", 0)
    explicit_incomplete_functions = sum(
        count for status, count in tracked_function_counts.items() if status != "done"
    )
    untracked_functions = sum(untracked_by_source.values())
    explicit_left_functions = total_functions - explicit_done_functions

    return {
        "completion_summary": {
            "total_c_functions": total_functions,
            "explicit_done_functions": explicit_done_functions,
            "explicit_incomplete_functions": explicit_incomplete_functions,
            "untracked_functions": untracked_functions,
            "estimated_left_functions": explicit_left_functions,
            "tracked_c_functions_by_status": tracked_function_counts,
            "tracked_tables_by_status": tracked_table_counts,
        },
        "modules": module_status_counts,
        "approximate_functions_by_module_status": approximate_function_counts,
        "tracked_symbols": tracked_symbol_counts,
        "untracked_functions_by_source": dict(sorted(untracked_by_source.items())),
    }


def print_summary_json(
    module_rows: list[dict[str, object]],
    functions: list[CFunction],
    symbol_status: dict[tuple[str, str], Status],
) -> None:
    print(json.dumps(status_totals(module_rows, functions, symbol_status), indent=2, sort_keys=True))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--c-root", type=Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--status", type=Path, default=STATUS_PATH)
    parser.add_argument("--format", choices=["markdown", "tsv", "json"], default="markdown")
    parser.add_argument("--list-functions", action="store_true")
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate the editable status TSV against upstream C and Rust targets",
    )
    args = parser.parse_args()

    module_status, symbol_status = load_statuses(args.status)
    functions = scan_functions(args.c_root)
    module_rows = module_rollup(functions, module_status, c_line_counts(args.c_root))

    if args.check:
        errors = validate_statuses(args.status, args.c_root, module_status, symbol_status, functions)
        if errors:
            for error in errors:
                print(error, file=sys.stderr)
            raise SystemExit(1)
        print(f"{args.status}: ok")
        return

    if args.format == "markdown":
        print_markdown(module_rows, functions, symbol_status, args.list_functions)
    elif args.format == "tsv":
        print_tsv(module_rows, functions, symbol_status, args.list_functions)
    else:
        print_summary_json(module_rows, functions, symbol_status)


if __name__ == "__main__":
    main()
