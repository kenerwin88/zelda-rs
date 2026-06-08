#!/usr/bin/env python3

from __future__ import annotations

import argparse
import collections
import datetime as dt
import json
import os
import pathlib
import re
import subprocess
from typing import Any


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE_ROOT = pathlib.Path(os.environ.get("ZELDA3_C_SRC", str(REPO_ROOT.parent / "zelda3" / "src")))
DEFAULT_LEDGER = pathlib.Path("docs/porting/c_function_ledger.json")
VALID_STATUSES = {"open", "partial", "implemented", "verified", "fixed", "not_applicable"}
DONE_STATUSES = {"verified", "fixed", "not_applicable"}


def iter_c_sources(source_root: pathlib.Path) -> list[pathlib.Path]:
    return sorted(
        path
        for path in source_root.rglob("*")
        if path.suffix in {".c", ".h"} and path.is_file()
    )


def count_lines(path: pathlib.Path) -> int:
    with path.open("r", encoding="utf-8", errors="replace") as handle:
        return sum(1 for _ in handle)


def load_ledger(path: pathlib.Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def prior_function_map(ledger: dict[str, Any] | None) -> dict[tuple[str, str], dict[str, Any]]:
    if not ledger:
        return {}
    result: dict[tuple[str, str], dict[str, Any]] = {}
    for file_entry in ledger.get("files", []):
        file_path = file_entry.get("path")
        for fn in file_entry.get("functions", []):
            name = fn.get("name")
            if file_path and name:
                result[(file_path, name)] = fn
    return result


def extract_functions(files: list[pathlib.Path]) -> dict[str, list[dict[str, Any]]]:
    if not files:
        return {}

    proc = subprocess.run(
        ["ctags", "-x", *map(str, files)],
        check=True,
        capture_output=True,
        text=True,
    )

    by_file: dict[str, list[tuple[int, str, str]]] = collections.defaultdict(list)
    for raw_line in proc.stdout.splitlines():
        parts = raw_line.split(None, 3)
        if len(parts) < 4:
            continue
        name, line_s, file_path, signature = parts
        if not line_s.isdigit():
            continue
        if not re.match(r"^[A-Za-z_]\w*$", name):
            continue
        if "(" not in signature or ")" not in signature:
            continue
        if signature.lstrip().startswith("typedef "):
            continue
        by_file[file_path].append((int(line_s), name, signature))

    result: dict[str, list[dict[str, Any]]] = {}
    for path in files:
        raw_functions = sorted(by_file.get(str(path), []), key=lambda item: (item[0], item[1]))
        file_line_count = count_lines(path)
        functions: list[dict[str, Any]] = []
        for idx, (start_line, name, signature) in enumerate(raw_functions):
            next_line = raw_functions[idx + 1][0] - 1 if idx + 1 < len(raw_functions) else file_line_count
            functions.append(
                {
                    "name": name,
                    "start_line": start_line,
                    "end_line": max(start_line, next_line),
                    "signature": signature,
                }
            )
        result[str(path)] = functions
    return result


def build_ledger(source_root: pathlib.Path, previous: dict[str, Any] | None) -> dict[str, Any]:
    files = iter_c_sources(source_root)
    extracted = extract_functions(files)
    prior = prior_function_map(previous)

    file_entries: list[dict[str, Any]] = []
    for path in files:
        file_path = str(path)
        functions: list[dict[str, Any]] = []
        for fn in extracted.get(file_path, []):
            old = prior.get((file_path, fn["name"]), {})
            functions.append(
                {
                    **fn,
                    "status": old.get("status", "open"),
                    "rust_path": old.get("rust_path"),
                    "rust_symbol": old.get("rust_symbol"),
                    "notes": old.get("notes", []),
                    "verified_at": old.get("verified_at"),
                }
            )
        file_entries.append(
            {
                "path": file_path,
                "total_lines": count_lines(path),
                "functions": functions,
            }
        )

    total_functions = sum(len(entry["functions"]) for entry in file_entries)
    by_status = collections.Counter(
        fn["status"] for entry in file_entries for fn in entry["functions"]
    )
    return {
        "schema_version": 1,
        "source_root": str(source_root),
        "generated_at": dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat(),
        "counts": {
            "files": len(file_entries),
            "files_with_functions": sum(1 for entry in file_entries if entry["functions"]),
            "functions": total_functions,
            "by_status": dict(sorted(by_status.items())),
        },
        "status_values": sorted(VALID_STATUSES),
        "files": file_entries,
    }


def write_ledger(path: pathlib.Path, ledger: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(ledger, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def find_function(
    ledger: dict[str, Any],
    file_query: str,
    function_name: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    matches: list[tuple[dict[str, Any], dict[str, Any]]] = []
    for file_entry in ledger["files"]:
        path = file_entry["path"]
        if path == file_query or pathlib.Path(path).name == file_query or path.endswith(file_query):
            for fn in file_entry["functions"]:
                if fn["name"] == function_name:
                    matches.append((file_entry, fn))
    if not matches:
        raise SystemExit(f"No ledger entry found for {file_query}:{function_name}")
    if len(matches) > 1:
        joined = ", ".join(f"{file_entry['path']}:{fn['name']}" for file_entry, fn in matches)
        raise SystemExit(f"Ambiguous ledger entry for {file_query}:{function_name}: {joined}")
    return matches[0]


def cmd_generate(args: argparse.Namespace) -> int:
    previous = load_ledger(args.ledger)
    ledger = build_ledger(args.source_root, previous)
    write_ledger(args.ledger, ledger)
    counts = ledger["counts"]
    print(
        f"Wrote {args.ledger}: {counts['functions']} functions across "
        f"{counts['files']} files ({counts['files_with_functions']} with functions)."
    )
    return 0


def cmd_mark(args: argparse.Namespace) -> int:
    ledger = load_ledger(args.ledger)
    if ledger is None:
        raise SystemExit(f"Ledger does not exist: {args.ledger}. Run generate first.")
    if args.status not in VALID_STATUSES:
        raise SystemExit(f"Invalid status {args.status!r}; expected one of {sorted(VALID_STATUSES)}")

    _, fn = find_function(ledger, args.file, args.function)
    fn["status"] = args.status
    if args.rust_path is not None:
        fn["rust_path"] = args.rust_path
    if args.rust_symbol is not None:
        fn["rust_symbol"] = args.rust_symbol
    if args.clear_rust:
        fn["rust_path"] = None
        fn["rust_symbol"] = None
    if args.clear_notes:
        fn["notes"] = []
    if args.note:
        fn.setdefault("notes", []).append(args.note)
    if args.status in {"implemented", "verified", "fixed", "not_applicable"}:
        fn["verified_at"] = dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat()
    else:
        fn["verified_at"] = None

    ledger["counts"]["by_status"] = dict(
        sorted(
            collections.Counter(
                item["status"] for entry in ledger["files"] for item in entry["functions"]
            ).items()
        )
    )
    write_ledger(args.ledger, ledger)
    print(
        f"Marked {args.file}:{args.function} as {args.status} "
        f"(C lines {fn['start_line']}-{fn['end_line']})."
    )
    return 0


def cmd_summary(args: argparse.Namespace) -> int:
    ledger = load_ledger(args.ledger)
    if ledger is None:
        raise SystemExit(f"Ledger does not exist: {args.ledger}. Run generate first.")
    counts = dict(ledger["counts"])
    total = counts["functions"]
    by_status = counts.get("by_status", {})
    done = sum(by_status.get(status, 0) for status in DONE_STATUSES)
    counts["done"] = done
    counts["done_percent"] = round(pct(done, total), 1)
    counts["by_status_percent"] = {
        status: round(pct(count, total), 1)
        for status, count in sorted(by_status.items())
    }
    print(json.dumps(counts, indent=2, sort_keys=True))
    return 0


def status_counts(functions: list[dict[str, Any]]) -> collections.Counter[str]:
    return collections.Counter(fn["status"] for fn in functions)


def pct(count: int, total: int) -> float:
    return (count / total) * 100 if total else 0.0


def cmd_progress(args: argparse.Namespace) -> int:
    ledger = load_ledger(args.ledger)
    if ledger is None:
        raise SystemExit(f"Ledger does not exist: {args.ledger}. Run generate first.")

    rows: list[dict[str, Any]] = []
    for file_entry in ledger["files"]:
        if not file_matches(file_entry["path"], args.file):
            continue
        functions = file_entry["functions"]
        if not functions:
            continue
        counts = status_counts(functions)
        done = sum(counts[status] for status in DONE_STATUSES)
        total = len(functions)
        rows.append(
            {
                "path": file_entry["path"],
                "name": pathlib.Path(file_entry["path"]).name,
                "done": done,
                "open": counts["open"],
                "partial": counts["partial"],
                "implemented": counts["implemented"],
                "total": total,
                "pct": pct(done, total),
                "counts": counts,
            }
        )

    if args.sort == "open":
        rows.sort(key=lambda row: (-row["open"], row["name"]))
    elif args.sort == "done":
        rows.sort(key=lambda row: (-row["pct"], row["name"]))
    else:
        rows.sort(key=lambda row: row["path"])

    total_functions = sum(row["total"] for row in rows)
    total_done = sum(row["done"] for row in rows)
    total_open = sum(row["open"] for row in rows)
    aggregate_counts: collections.Counter[str] = collections.Counter()
    for row in rows:
        aggregate_counts.update(row["counts"])
    total_pct = pct(total_done, total_functions)
    total_remaining = total_functions - total_done
    print(
        f"summary: done={total_done}/{total_functions} ({total_pct:.1f}%) "
        f"remaining={total_remaining}/{total_functions} ({pct(total_remaining, total_functions):.1f}%) "
        f"open={total_open}/{total_functions} ({pct(total_open, total_functions):.1f}%) "
        f"files={len(rows)}"
    )
    breakdown = [
        f"{status}={aggregate_counts[status]}/{total_functions} "
        f"({pct(aggregate_counts[status], total_functions):.1f}%)"
        for status in sorted(VALID_STATUSES)
        if aggregate_counts[status]
    ]
    if breakdown:
        print("status: " + ", ".join(breakdown))
    print("done%    done/open/total  file")
    print("------   ---------------  ----")
    for row in rows[: args.limit or None]:
        extra: list[str] = []
        if row["partial"]:
            extra.append(f"partial={row['partial']}")
        if row["implemented"]:
            extra.append(f"implemented={row['implemented']}")
        suffix = f" ({', '.join(extra)})" if extra else ""
        print(
            f"{row['pct']:6.1f}   "
            f"{row['done']:4}/{row['open']:4}/{row['total']:<4}  "
            f"{row['name']}{suffix}"
        )
    return 0


def file_matches(path: str, query: str | None) -> bool:
    if not query:
        return True
    return path == query or pathlib.Path(path).name == query or path.endswith(query)


def cmd_list(args: argparse.Namespace) -> int:
    ledger = load_ledger(args.ledger)
    if ledger is None:
        raise SystemExit(f"Ledger does not exist: {args.ledger}. Run generate first.")

    shown = 0
    for file_entry in ledger["files"]:
        if not file_matches(file_entry["path"], args.file):
            continue
        for fn in file_entry["functions"]:
            if args.status and fn["status"] != args.status:
                continue
            rust = ""
            if fn.get("rust_path") or fn.get("rust_symbol"):
                rust = f" -> {fn.get('rust_path') or '?'}"
                if fn.get("rust_symbol"):
                    rust += f"::{fn['rust_symbol']}"
            print(
                f"{fn['status']:14} {pathlib.Path(file_entry['path']).name}:"
                f"{fn['start_line']}-{fn['end_line']} {fn['name']}{rust}"
            )
            shown += 1
            if args.limit and shown >= args.limit:
                return 0
    return 0


def cmd_show(args: argparse.Namespace) -> int:
    ledger = load_ledger(args.ledger)
    if ledger is None:
        raise SystemExit(f"Ledger does not exist: {args.ledger}. Run generate first.")
    file_entry, fn = find_function(ledger, args.file, args.function)
    print(
        json.dumps(
            {
                "file": file_entry["path"],
                **fn,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Maintain the C-to-Rust parity ledger.")
    parser.add_argument("--ledger", type=pathlib.Path, default=DEFAULT_LEDGER)
    sub = parser.add_subparsers(dest="command", required=True)

    generate = sub.add_parser("generate", help="Regenerate function/line inventory while preserving statuses.")
    generate.add_argument("--source-root", type=pathlib.Path, default=DEFAULT_SOURCE_ROOT)
    generate.set_defaults(func=cmd_generate)

    mark = sub.add_parser("mark", help="Mark one C function's parity status.")
    mark.add_argument("--file", required=True, help="C file path, suffix, or basename.")
    mark.add_argument("--function", required=True, help="C function name.")
    mark.add_argument("--status", required=True, choices=sorted(VALID_STATUSES))
    mark.add_argument("--rust-path")
    mark.add_argument("--rust-symbol")
    mark.add_argument("--note")
    mark.add_argument("--clear-notes", action="store_true")
    mark.add_argument("--clear-rust", action="store_true")
    mark.set_defaults(func=cmd_mark)

    summary = sub.add_parser("summary", help="Print compact ledger counts.")
    summary.set_defaults(func=cmd_summary)

    progress = sub.add_parser("progress", help="Print file-by-file parity progress.")
    progress.add_argument("--file", help="C file path, suffix, or basename.")
    progress.add_argument("--limit", type=int, default=80)
    progress.add_argument("--sort", choices=["path", "open", "done"], default="path")
    progress.set_defaults(func=cmd_progress)

    list_cmd = sub.add_parser("list", help="Print compact function rows without dumping the full JSON.")
    list_cmd.add_argument("--file", help="C file path, suffix, or basename.")
    list_cmd.add_argument("--status", choices=sorted(VALID_STATUSES))
    list_cmd.add_argument("--limit", type=int, default=50)
    list_cmd.set_defaults(func=cmd_list)

    show = sub.add_parser("show", help="Print one function's ledger entry.")
    show.add_argument("--file", required=True, help="C file path, suffix, or basename.")
    show.add_argument("--function", required=True, help="C function name.")
    show.set_defaults(func=cmd_show)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
