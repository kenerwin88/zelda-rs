#!/usr/bin/env python3
"""Rewrite semantic view method calls to a native semantic accessor.

This is a deliberately narrow codemod for the RAM-to-native GameState cutover.
It rewrites direct chains such as:

    self.player_state().x()

to a mapped native chain such as:

    self.follower_link_state().x()

Aliases are reported, not rewritten, because safely replacing local borrowed
views requires scope-aware edits. Pass --rewrite-safe-aliases to also rewrite
simple local aliases when every use in the lexical block maps cleanly to the
same target accessor and the alias is not otherwise used.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"


@dataclass(frozen=True)
class MethodMapping:
    source_accessor: str
    source_method: str
    target_accessor: str
    target_method: str


def default_paths() -> list[Path]:
    return sorted(path for path in SRC_ROOT.glob("*.rs") if path.is_file())


def relative(path: Path) -> str:
    path = path.resolve()
    return str(path.relative_to(REPO_ROOT))


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def brace_depth_at(text: str, offset: int) -> int:
    depth = 0
    for char in text[:offset]:
        if char == "{":
            depth += 1
        elif char == "}":
            depth = max(0, depth - 1)
    return depth


def block_end_for_offset(text: str, offset: int) -> int:
    depth = brace_depth_at(text, offset)
    for index in range(offset, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth < brace_depth_at(text, offset):
                return index
    return len(text)


def parse_mapping(value: str) -> MethodMapping:
    match = re.fullmatch(
        r"([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)="
        r"([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)",
        value.strip(),
    )
    if not match:
        raise argparse.ArgumentTypeError(
            "expected SOURCE_ACCESSOR.SOURCE_METHOD=TARGET_ACCESSOR.TARGET_METHOD"
        )
    return MethodMapping(*match.groups())


def mappings_from_file(path: Path) -> list[MethodMapping]:
    mappings: list[MethodMapping] = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("--map "):
            line = line.removeprefix("--map ").strip()
        try:
            mappings.append(parse_mapping(line))
        except argparse.ArgumentTypeError as error:
            raise SystemExit(f"{path}:{line_number}: {error}") from error
    return mappings


def rewrite_direct_calls(text: str, mappings: list[MethodMapping]) -> tuple[str, list[tuple[int, str]]]:
    rewrites: list[tuple[int, str]] = []
    for mapping in mappings:
        source_accessor = re.escape(mapping.source_accessor)
        source_method = re.escape(mapping.source_method)
        pattern = re.compile(
            rf"\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*"
            rf"{source_accessor}\(\)\s*\.\s*{source_method}\s*\("
        )

        def replace(match: re.Match[str]) -> str:
            rewrites.append((match.start(), f"{mapping.source_accessor}.{mapping.source_method}"))
            receiver = match.group(1)
            return f"{receiver}.{mapping.target_accessor}().{mapping.target_method}("

        text = pattern.sub(replace, text)
    return text, rewrites


def mapping_by_accessor_method(mappings: list[MethodMapping]) -> dict[tuple[str, str], MethodMapping]:
    return {(mapping.source_accessor, mapping.source_method): mapping for mapping in mappings}


def rewrite_safe_aliases(text: str, mappings: list[MethodMapping]) -> tuple[str, list[tuple[int, str]], list[tuple[int, str]]]:
    source_accessors = sorted({mapping.source_accessor for mapping in mappings})
    if not source_accessors:
        return text, [], []

    by_method = mapping_by_accessor_method(mappings)
    source_pattern = "|".join(re.escape(accessor) for accessor in source_accessors)
    alias_re = re.compile(
        rf"(?m)^(?P<indent>\s*)let\s+(?:mut\s+)?(?P<alias>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*"
        rf"(?P<receiver>[A-Za-z_][A-Za-z0-9_]*)\.({source_pattern})\(\)\s*;\n"
    )

    replacements: list[tuple[int, int, str]] = []
    rewrites: list[tuple[int, str]] = []
    rejected: list[tuple[int, str]] = []

    for alias_match in alias_re.finditer(text):
        alias = alias_match.group("alias")
        receiver = alias_match.group("receiver")
        accessor = alias_match.group(4)
        scope_start = alias_match.end()
        scope_end = block_end_for_offset(text, scope_start)
        scope = text[scope_start:scope_end]

        if re.search(rf"(?m)^\s*let\s+(?:mut\s+)?{re.escape(alias)}\b", scope):
            rejected.append((alias_match.start(), f"alias `{alias}` is shadowed later in the block"))
            continue
        if re.search(rf"\b{re.escape(alias)}\s*=", scope):
            rejected.append((alias_match.start(), f"alias `{alias}` is assigned later in the block"))
            continue

        uses = list(re.finditer(rf"\b{re.escape(alias)}\b", scope))
        if not uses:
            replacements.append((alias_match.start(), alias_match.end(), ""))
            rewrites.append((alias_match.start(), f"removed unused alias `{alias}` from {accessor}()"))
            continue

        alias_replacements: list[tuple[int, int, str, str]] = []
        ok = True
        for use in uses:
            after = scope[use.end() :]
            method_match = re.match(r"\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(", after)
            if method_match is None:
                rejected.append((alias_match.start(), f"alias `{alias}` has a non-method use"))
                ok = False
                break
            method = method_match.group(1)
            mapping = by_method.get((accessor, method))
            if mapping is None:
                rejected.append((alias_match.start(), f"alias `{alias}` uses unmapped method `{method}`"))
                ok = False
                break
            method_start = scope_start + use.end() + method_match.start(1)
            method_end = scope_start + use.end() + method_match.end(1)
            alias_abs_start = scope_start + use.start()
            alias_abs_end = scope_start + use.end()
            alias_replacements.append(
                (
                    alias_abs_start,
                    alias_abs_end,
                    f"{receiver}.{mapping.target_accessor}()",
                    f"{accessor}.{method}",
                )
            )
            alias_replacements.append((method_start, method_end, mapping.target_method, f"{accessor}.{method}"))

        if not ok:
            continue

        replacements.append((alias_match.start(), alias_match.end(), ""))
        rewritten_methods = sorted({label for _, _, _, label in alias_replacements})
        rewrites.append(
            (
                alias_match.start(),
                f"rewrote alias `{alias}` from {accessor}(): {', '.join(rewritten_methods)}",
            )
        )
        for start, end, replacement, _ in alias_replacements:
            replacements.append((start, end, replacement))

    if not replacements:
        return text, rewrites, rejected

    next_text = text
    for start, end, replacement in sorted(replacements, reverse=True):
        next_text = next_text[:start] + replacement + next_text[end:]
    return next_text, rewrites, rejected


def rewrite_partial_read_aliases(text: str, mappings: list[MethodMapping]) -> tuple[str, list[tuple[int, str]]]:
    """Rewrite mapped method calls behind immutable read aliases.

    This deliberately leaves the alias binding in place. It is useful when a
    block mixes already-native methods with not-yet-migrated methods: mapped
    reads can move to native state without pretending the whole alias is safe to
    remove. Mutable aliases are skipped because replacing only part of a mutable
    borrow can create overlapping `&mut self` borrows.
    """

    read_mappings = [
        mapping
        for mapping in mappings
        if not mapping.source_accessor.endswith("_mut")
        and not mapping.target_accessor.endswith("_mut")
    ]
    source_accessors = sorted({mapping.source_accessor for mapping in read_mappings})
    if not source_accessors:
        return text, []

    by_method = mapping_by_accessor_method(read_mappings)
    source_pattern = "|".join(re.escape(accessor) for accessor in source_accessors)
    alias_re = re.compile(
        rf"(?m)^(?P<indent>\s*)let\s+(?P<mut>mut\s+)?(?P<alias>[A-Za-z_][A-Za-z0-9_]*)\s*=\s*"
        rf"(?P<receiver>[A-Za-z_][A-Za-z0-9_]*)\.({source_pattern})\(\)\s*;\n"
    )

    replacements: list[tuple[int, int, str]] = []
    rewrites: list[tuple[int, str]] = []
    for alias_match in alias_re.finditer(text):
        if alias_match.group("mut"):
            continue
        alias = alias_match.group("alias")
        receiver = alias_match.group("receiver")
        accessor = alias_match.group(5)
        scope_start = alias_match.end()
        scope_end = block_end_for_offset(text, scope_start)
        scope = text[scope_start:scope_end]

        for use in re.finditer(rf"\b{re.escape(alias)}\b", scope):
            after = scope[use.end() :]
            method_match = re.match(r"\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(", after)
            if method_match is None:
                continue
            method = method_match.group(1)
            mapping = by_method.get((accessor, method))
            if mapping is None:
                continue
            alias_abs_start = scope_start + use.start()
            alias_abs_end = scope_start + use.end()
            method_start = scope_start + use.end() + method_match.start(1)
            method_end = scope_start + use.end() + method_match.end(1)
            replacements.append((alias_abs_start, alias_abs_end, f"{receiver}.{mapping.target_accessor}()"))
            replacements.append((method_start, method_end, mapping.target_method))
            rewrites.append((alias_abs_start, f"{accessor}.{method}"))

    if not replacements:
        return text, []

    next_text = text
    for start, end, replacement in sorted(replacements, reverse=True):
        next_text = next_text[:start] + replacement + next_text[end:]
    return next_text, rewrites


def alias_findings(text: str, mappings: list[MethodMapping]) -> list[tuple[int, str]]:
    source_accessors = sorted({mapping.source_accessor for mapping in mappings})
    if not source_accessors:
        return []
    source_pattern = "|".join(re.escape(accessor) for accessor in source_accessors)
    alias_re = re.compile(
        rf"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"
        rf"([A-Za-z_][A-Za-z0-9_]*)\.({source_pattern})\(\)\s*;"
    )
    method_by_accessor: dict[str, set[str]] = {}
    for mapping in mappings:
        method_by_accessor.setdefault(mapping.source_accessor, set()).add(mapping.source_method)

    findings: list[tuple[int, str]] = []
    for alias_match in alias_re.finditer(text):
        alias = alias_match.group(1)
        accessor = alias_match.group(3)
        methods = method_by_accessor.get(accessor, set())
        if not methods:
            continue
        method_pattern = "|".join(re.escape(method) for method in sorted(methods))
        after = text[alias_match.end() :]
        use_match = re.search(rf"\b{re.escape(alias)}\s*\.\s*({method_pattern})\s*\(", after)
        if use_match:
            findings.append(
                (
                    alias_match.start(),
                    f"alias `{alias}` from {accessor}() has mapped method `{use_match.group(1)}`",
                )
            )
    return findings


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="rewrite files in place")
    parser.add_argument(
        "--map",
        dest="mappings",
        action="append",
        type=parse_mapping,
        default=[],
        help="method mapping: source_accessor.method=target_accessor.method",
    )
    parser.add_argument(
        "--map-file",
        action="append",
        type=Path,
        default=[],
        help="file containing one --map-compatible mapping per line; comments are allowed",
    )
    parser.add_argument(
        "--rewrite-safe-aliases",
        action="store_true",
        help=(
            "rewrite simple local aliases when every alias use in the block is a mapped method call; "
            "unsafe aliases are reported"
        ),
    )
    parser.add_argument(
        "--rewrite-partial-read-aliases",
        action="store_true",
        help=(
            "rewrite mapped method calls behind immutable read aliases even when other alias uses "
            "remain; mutable aliases are skipped"
        ),
    )
    parser.add_argument(
        "--fail-on-rejected-alias",
        action="store_true",
        help="return non-zero if --rewrite-safe-aliases rejects any alias candidate",
    )
    parser.add_argument(
        "--fail-on-alias",
        action="store_true",
        help="return non-zero if mapped methods remain behind local semantic view aliases",
    )
    parser.add_argument(
        "--summary",
        action="store_true",
        help="only print counts by mapping and alias category",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=200,
        help="maximum detailed findings to print per section; use 0 for all",
    )
    parser.add_argument("paths", nargs="*", type=Path, help="Rust files or directories to scan")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    mappings: list[MethodMapping] = list(args.mappings)
    for path in args.map_file:
        mappings.extend(mappings_from_file(path))
    if not mappings:
        print("no method mappings supplied", file=sys.stderr)
        return 2

    changed: list[Path] = []
    direct_hits: list[str] = []
    alias_hits: list[str] = []
    alias_rewrites: list[str] = []
    partial_alias_rewrites: list[str] = []
    rejected_alias_hits: list[str] = []
    direct_counts: Counter[str] = Counter()
    alias_counts: Counter[str] = Counter()
    alias_rewrite_counts: Counter[str] = Counter()
    partial_alias_rewrite_counts: Counter[str] = Counter()
    rejected_alias_counts: Counter[str] = Counter()
    paths = args.paths or default_paths()
    for path in paths:
        files = sorted(path.rglob("*.rs")) if path.is_dir() else [path]
        for file_path in files:
            text = file_path.read_text()
            next_text, rewrites = rewrite_direct_calls(text, mappings)
            if args.rewrite_safe_aliases:
                next_text, safe_alias_rewrites, rejected_aliases = rewrite_safe_aliases(next_text, mappings)
            else:
                safe_alias_rewrites = []
                rejected_aliases = []
            if args.rewrite_partial_read_aliases:
                next_text, partial_read_alias_rewrites = rewrite_partial_read_aliases(next_text, mappings)
            else:
                partial_read_alias_rewrites = []
            aliases = alias_findings(next_text, mappings)
            for offset, label in rewrites:
                direct_hits.append(f"{relative(file_path)}:{line_for_offset(text, offset)}: {label}")
                direct_counts[label] += 1
            for offset, label in safe_alias_rewrites:
                alias_rewrites.append(f"{relative(file_path)}:{line_for_offset(text, offset)}: {label}")
                alias_rewrite_counts[label] += 1
            for offset, label in partial_read_alias_rewrites:
                partial_alias_rewrites.append(f"{relative(file_path)}:{line_for_offset(text, offset)}: {label}")
                partial_alias_rewrite_counts[label] += 1
            for offset, label in rejected_aliases:
                rejected_alias_hits.append(f"{relative(file_path)}:{line_for_offset(text, offset)}: {label}")
                rejected_alias_counts[label] += 1
            for offset, label in aliases:
                alias_hits.append(f"{relative(file_path)}:{line_for_offset(next_text, offset)}: {label}")
                alias_counts[label] += 1
            if args.apply and next_text != text:
                file_path.write_text(next_text)
                changed.append(file_path)

    if args.apply and changed:
        print("rewrote semantic view method calls:")
        for path in changed:
            print(f"  {relative(path)}")
    elif direct_hits:
        print("direct semantic view method call(s) that can be rewritten:")
        for label, count in direct_counts.most_common():
            print(f"  {label}: {count}")
        if not args.summary:
            shown = direct_hits if args.limit <= 0 else direct_hits[: args.limit]
            for hit in shown:
                print(f"  {hit}")
            if len(shown) < len(direct_hits):
                print(f"  ... {len(direct_hits) - len(shown)} more; pass --limit 0 to show all")
    else:
        print("no direct semantic view method calls matched")

    if alias_rewrites:
        print("safe alias semantic view method call(s) rewritten:")
        for label, count in alias_rewrite_counts.most_common():
            print(f"  {label}: {count}")
        if not args.summary:
            shown = alias_rewrites if args.limit <= 0 else alias_rewrites[: args.limit]
            for hit in shown:
                print(f"  {hit}")
            if len(shown) < len(alias_rewrites):
                print(f"  ... {len(alias_rewrites) - len(shown)} more; pass --limit 0 to show all")

    if partial_alias_rewrites:
        print("partial read-alias semantic view method call(s) rewritten:")
        for label, count in partial_alias_rewrite_counts.most_common():
            print(f"  {label}: {count}")
        if not args.summary:
            shown = partial_alias_rewrites if args.limit <= 0 else partial_alias_rewrites[: args.limit]
            for hit in shown:
                print(f"  {hit}")
            if len(shown) < len(partial_alias_rewrites):
                print(f"  ... {len(partial_alias_rewrites) - len(shown)} more; pass --limit 0 to show all")

    if rejected_alias_hits:
        print("semantic view alias candidate(s) rejected as unsafe:")
        for label, count in rejected_alias_counts.most_common():
            print(f"  {label}: {count}")
        if not args.summary:
            shown = rejected_alias_hits if args.limit <= 0 else rejected_alias_hits[: args.limit]
            for hit in shown:
                print(f"  {hit}")
            if len(shown) < len(rejected_alias_hits):
                print(f"  ... {len(rejected_alias_hits) - len(shown)} more; pass --limit 0 to show all")
        if args.fail_on_rejected_alias:
            return 1

    if alias_hits:
        print("alias-backed semantic view method call(s) need manual or scoped migration:")
        for label, count in alias_counts.most_common():
            print(f"  {label}: {count}")
        if not args.summary:
            shown = alias_hits if args.limit <= 0 else alias_hits[: args.limit]
            for hit in shown:
                print(f"  {hit}")
            if len(shown) < len(alias_hits):
                print(f"  ... {len(alias_hits) - len(shown)} more; pass --limit 0 to show all")
        if args.fail_on_alias:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
