#!/usr/bin/env python3
"""Drive one semantic-view-to-native migration slice.

This wraps the existing planner and codemod so a safe slice can be migrated
with one repeatable command:

    python3 scripts/assist_semantic_view_migration.py \
        --source-accessor player_state_mut \
        --target-accessor follower_link_state_mut \
        --target-type NativeFollowerLinkBridgeMut \
        --method '^(clear_speed_modifier|set_speed_modifier)$'

By default this is a dry run: it writes a generated method map under
target/semantic-migration, prints the matching direct/alias callsites, and
prints the follow-up apply command. Pass --apply once the native target really
owns the underlying RAM fields.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

import plan_semantic_state_migration as planner


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MAP_DIR = REPO_ROOT / "target" / "semantic-migration"


def relative(path: Path) -> str:
    path = path.resolve()
    return str(path.relative_to(REPO_ROOT))


def shell_quote(value: str) -> str:
    return "'" + value.replace("'", "'\\''") + "'"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source-accessor",
        required=True,
        help="RAM-backed accessor to migrate, e.g. player_state_mut",
    )
    parser.add_argument(
        "--target-accessor",
        required=True,
        help="native accessor to rewrite to, e.g. follower_link_state_mut",
    )
    parser.add_argument(
        "--target-type",
        required=True,
        help="native Rust API type that must contain mapped methods",
    )
    parser.add_argument(
        "--method",
        action="append",
        help="regex for source methods to include; repeat to OR several patterns",
    )
    parser.add_argument(
        "--exclude-method",
        action="append",
        help="regex for source methods to exclude",
    )
    parser.add_argument(
        "--method-limit",
        type=int,
        default=0,
        help="maximum mapped methods to emit by current use count; 0 means all",
    )
    parser.add_argument(
        "--recommend",
        action="store_true",
        help="print the next automation-friendly slice instead of writing a codemod map",
    )
    parser.add_argument(
        "--paired-mutator-accessor",
        help=(
            "with --recommend, RAM-backed mutator accessor to inspect for write-side blockers, "
            "e.g. player_state_mut"
        ),
    )
    parser.add_argument(
        "--paired-target-type",
        help=(
            "with --recommend and --paired-mutator-accessor, native mutator API type; "
            "methods missing here are treated as write-side blockers"
        ),
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=12,
        help="with --recommend, how many ready methods to include in the suggested batch",
    )
    parser.add_argument(
        "--min-uses",
        type=int,
        default=1,
        help="with --recommend, hide ready methods used fewer than this many times",
    )
    parser.add_argument(
        "--map-file",
        type=Path,
        help="where to write the generated map; defaults under target/semantic-migration",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="rewrite files in place and run lightweight verification",
    )
    parser.add_argument(
        "--rewrite-safe-aliases",
        action="store_true",
        help="also let the codemod rewrite aliases whose whole block is mapped",
    )
    parser.add_argument(
        "--rewrite-partial-read-aliases",
        action="store_true",
        help="also rewrite mapped immutable read-alias method calls even when other alias uses remain",
    )
    parser.add_argument(
        "--include-native-alias-methods",
        action="store_true",
        help=(
            "include same-name source/native methods even if they only appear behind aliases; "
            "useful with --rewrite-partial-read-aliases"
        ),
    )
    parser.add_argument(
        "--path",
        action="append",
        type=Path,
        help="file or directory to scan/rewrite; defaults to crates/zelda3/src",
    )
    parser.add_argument(
        "--skip-checks",
        action="store_true",
        help="with --apply, skip fmt/check/readability follow-up commands",
    )
    return parser.parse_args()


def selected_plan(source_accessor: str) -> planner.AccessorPlan:
    uses = planner.selected_uses(
        ["ram-backed-view"],
        f"^{re.escape(source_accessor)}$",
        None,
        None,
        None,
    )
    plans = planner.accessor_plans(uses)
    if len(plans) != 1:
        raise SystemExit(f"expected one plan for {source_accessor}, found {len(plans)}")
    return plans[0]


def compile_patterns(patterns: list[str] | None) -> list[re.Pattern[str]]:
    return [re.compile(pattern) for pattern in patterns or []]


def allowed_method(
    method: str,
    include_patterns: list[re.Pattern[str]],
    exclude_patterns: list[re.Pattern[str]],
) -> bool:
    if include_patterns and not any(pattern.search(method) for pattern in include_patterns):
        return False
    return not any(pattern.search(method) for pattern in exclude_patterns)


def generated_map_path(source_accessor: str, target_accessor: str) -> Path:
    name = f"{source_accessor}_to_{target_accessor}.map"
    return DEFAULT_MAP_DIR / name


def build_map(
    plan: planner.AccessorPlan,
    target_accessor: str,
    target_type: str,
    include_patterns: list[re.Pattern[str]],
    exclude_patterns: list[re.Pattern[str]],
    method_limit: int,
    constants: dict[str, int],
    blocker_methods_by_const: dict[str, list[tuple[str, int]]],
    include_native_alias_methods: bool = False,
) -> tuple[str, Counter[str], Counter[str]]:
    source_type = planner.view_type_name(plan.return_type)
    if source_type is None:
        raise SystemExit(f"unable to infer source type from {plan.return_type}")
    source = planner.impl_methods(source_type)
    native = planner.impl_methods(target_type)
    if source is None:
        raise SystemExit(f"unable to find impl for {source_type}")
    if native is None:
        raise SystemExit(f"unable to find impl for {target_type}")

    _, source_methods = source
    _, native_methods = native
    blocked: Counter[str] = Counter()

    candidates = [
        method
        for method, _ in plan.methods.most_common()
        if method in source_methods
        and method in native_methods
        and allowed_method(method, include_patterns, exclude_patterns)
    ]
    if include_native_alias_methods:
        seen_candidates = set(candidates)
        for method in sorted(set(source_methods) & set(native_methods)):
            if method in seen_candidates:
                continue
            if not allowed_method(method, include_patterns, exclude_patterns):
                continue
            candidates.append(method)
            seen_candidates.add(method)
    unblocked_candidates = []
    for method in candidates:
        source_method = source_methods[method]
        if any(
            const in blocker_methods_by_const
            for const in method_constants(source_method, constants, source_methods)
        ):
            blocked[method] = plan.methods[method]
            continue
        unblocked_candidates.append(method)
    candidates = unblocked_candidates
    if method_limit > 0:
        candidates = candidates[:method_limit]

    lines = [
        f"# Generated from {plan.accessor}: {source_type} -> {target_accessor}: {target_type}",
        "# Review before applying. The native target must own every mapped field.",
    ]
    for method in candidates:
        uses = plan.methods.get(method, 0)
        lines.append(f"{plan.accessor}.{method}={target_accessor}.{method}  # {uses} use(s)")
    if blocked:
        lines.append("# blocked by paired mutators that are not native-owned yet:")
        for method, uses in blocked.most_common():
            lines.append(f"# {plan.accessor}.{method} skipped  # {uses} use(s)")
    return (
        "\n".join(lines) + "\n",
        Counter({method: plan.methods.get(method, 0) for method in candidates}),
        blocked,
    )


def source_native_methods(
    plan: planner.AccessorPlan,
    target_type: str,
) -> tuple[str, dict[str, planner.MethodInfo], dict[str, planner.MethodInfo]]:
    source_type = planner.view_type_name(plan.return_type)
    if source_type is None:
        raise SystemExit(f"unable to infer source type from {plan.return_type}")
    source = planner.impl_methods(source_type)
    native = planner.impl_methods(target_type)
    if source is None:
        raise SystemExit(f"unable to find impl for {source_type}")
    if native is None:
        raise SystemExit(f"unable to find impl for {target_type}")
    _, source_methods = source
    _, native_methods = native
    return source_type, source_methods, native_methods


def method_constants(
    method: planner.MethodInfo,
    constants: dict[str, int],
    methods: dict[str, planner.MethodInfo] | None = None,
    seen: set[str] | None = None,
) -> tuple[str, ...]:
    names = {name for name in planner.body_const_names(method.body) if name in constants}
    if methods is not None:
        seen = set(seen or set())
        seen.add(method.name)
        for called in re.findall(r"\bself\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(", method.body):
            if called in seen:
                continue
            called_method = methods.get(called)
            if called_method is None:
                continue
            names.update(method_constants(called_method, constants, methods, seen))
    return tuple(sorted(names))


def constant_labels(names: tuple[str, ...], constants: dict[str, int]) -> str:
    if not names:
        return "(no RAM constants detected)"
    return ", ".join(f"{name}@0x{constants[name]:04x}" for name in names)


def blocker_summary(
    method_names: list[str],
    source_methods: dict[str, planner.MethodInfo],
    constants: dict[str, int],
    blocker_methods_by_const: dict[str, list[tuple[str, int]]],
    limit: int,
) -> dict[str, list[tuple[str, int]]]:
    summaries: dict[str, list[tuple[str, int]]] = {}
    for method_name in method_names:
        source_method = source_methods.get(method_name)
        if source_method is None:
            continue
        blockers: dict[str, int] = {}
        for const in method_constants(source_method, constants, source_methods):
            for blocker, uses in blocker_methods_by_const.get(const, []):
                blockers[blocker] = max(blockers.get(blocker, 0), uses)
        if blockers:
            summaries[method_name] = sorted(blockers.items(), key=lambda item: (-item[1], item[0]))[:limit]
    return summaries


def write_blockers_by_const(
    args: argparse.Namespace,
    constants: dict[str, int],
) -> dict[str, list[tuple[str, int]]]:
    if not args.paired_mutator_accessor:
        return {}
    if not args.paired_target_type:
        raise SystemExit("--paired-mutator-accessor requires --paired-target-type")

    paired_plan = selected_plan(args.paired_mutator_accessor)
    _, paired_source_methods, paired_native_methods = source_native_methods(
        paired_plan,
        args.paired_target_type,
    )
    blockers: dict[str, list[tuple[str, int]]] = {}
    for method_name, uses in paired_plan.methods.items():
        if method_name == "<alias>" or method_name in paired_native_methods:
            continue
        source_method = paired_source_methods.get(method_name)
        if source_method is None:
            continue
        for const in method_constants(source_method, constants, paired_source_methods):
            blockers.setdefault(const, []).append((method_name, uses))
    for const, methods in blockers.items():
        blockers[const] = sorted(methods, key=lambda item: (-item[1], item[0]))
    return blockers


def print_method_evidence(
    title: str,
    methods: Counter[str],
    source_methods: dict[str, planner.MethodInfo],
    constants: dict[str, int],
    method_limit: int,
) -> None:
    if not methods:
        return
    print(f"  {title}:")
    for method_name, uses in methods.most_common(method_limit):
        method = source_methods.get(method_name)
        if method is None:
            print(f"    {method_name} x{uses}")
            continue
        print(
            f"    {method_name} x{uses}; source line {method.line}; "
            f"fields: {constant_labels(method_constants(method, constants, source_methods), constants)}"
        )


def print_recommendation(args: argparse.Namespace, plan: planner.AccessorPlan) -> int:
    source_type, source_methods, native_methods = source_native_methods(plan, args.target_type)
    constants = planner.constants_by_name()
    include_patterns = compile_patterns(args.method)
    exclude_patterns = compile_patterns(args.exclude_method)

    ready = Counter(
        {
            method: count
            for method, count in plan.methods.items()
            if method != "<alias>"
            and count >= args.min_uses
            and method in source_methods
            and method in native_methods
            and allowed_method(method, include_patterns, exclude_patterns)
        }
    )
    missing = Counter(
        {
            method: count
            for method, count in plan.methods.items()
            if method != "<alias>"
            and method in source_methods
            and method not in native_methods
            and allowed_method(method, include_patterns, exclude_patterns)
        }
    )
    blocker_methods_by_const = write_blockers_by_const(args, constants)
    ready_blockers = blocker_summary(
        [method for method, _ in ready.most_common()],
        source_methods,
        constants,
        blocker_methods_by_const,
        args.batch_size,
    )
    blocked_ready = Counter({method: ready[method] for method in ready_blockers})
    unblocked_ready = Counter(
        {method: count for method, count in ready.items() if method not in ready_blockers}
    )
    alias_uses = plan.methods.get("<alias>", 0)
    batch_methods = [method for method, _ in unblocked_ready.most_common(args.batch_size)]

    print(f"semantic migration recommendation for {plan.accessor} -> {args.target_accessor}")
    print(f"  source API: {source_type}")
    print(f"  target API: {args.target_type}")
    print(f"  total uses: {plan.uses}")
    print(f"  alias-backed uses needing scoped rewrite/manual review: {alias_uses}")
    if args.paired_mutator_accessor:
        print(
            f"  paired mutator blocker check: {args.paired_mutator_accessor} -> "
            f"{args.paired_target_type}"
        )
    print(
        f"  same-name methods present on native target: "
        f"{sum(ready.values())} use(s), {len(ready)} method(s)"
    )
    if blocked_ready:
        print(
            f"  write-side blocked ready methods: "
            f"{sum(blocked_ready.values())} use(s), {len(blocked_ready)} method(s)"
        )
    print(
        f"  automation-ready after blocker check: "
        f"{sum(unblocked_ready.values())} use(s), {len(unblocked_ready)} method(s)"
    )
    if unblocked_ready:
        print(
            "  top automation-ready methods: "
            + ", ".join(
                f"{method} x{count}" for method, count in unblocked_ready.most_common(args.batch_size)
            )
        )
    elif ready and args.paired_mutator_accessor:
        print("  top automation-ready methods: (none; add paired native mutators first)")
    print(f"  used source methods missing on native target: {sum(missing.values())} use(s), {len(missing)} method(s)")
    if missing:
        print(
            "  top missing methods: "
            + ", ".join(f"{method} x{count}" for method, count in missing.most_common(args.batch_size))
        )
    print_method_evidence(
        "field evidence for automation-ready methods",
        unblocked_ready,
        source_methods,
        constants,
        args.batch_size,
    )
    if blocked_ready:
        print_method_evidence(
            "field evidence for blocked methods",
            blocked_ready,
            source_methods,
            constants,
            args.batch_size,
        )
        print("  write-side blockers:")
        for method_name, blockers in sorted(
            ready_blockers.items(),
            key=lambda item: (-ready[item[0]], item[0]),
        )[: args.batch_size]:
            print(
                f"    {method_name}: "
                + ", ".join(f"{blocker} x{uses}" for blocker, uses in blockers)
            )
    print_method_evidence(
        "field evidence for top missing methods",
        Counter(dict(missing.most_common(args.batch_size))),
        source_methods,
        constants,
        args.batch_size,
    )

    if batch_methods:
        method_regex = "^(" + "|".join(re.escape(method) for method in batch_methods) + ")$"
        base_command = [
            "python3",
            "scripts/assist_semantic_view_migration.py",
            "--source-accessor",
            args.source_accessor,
            "--target-accessor",
            args.target_accessor,
            "--target-type",
            args.target_type,
            "--method",
            method_regex,
            "--rewrite-safe-aliases",
        ]
        print()
        print("next dry-run command:")
        print("  " + " ".join(shell_quote(part) if " " in part else part for part in base_command))
        print("apply after native ownership is verified:")
        print(
            "  "
            + " ".join(
                shell_quote(part) if " " in part else part
                for part in [*base_command, "--apply", "--skip-checks"]
            )
        )

    print()
    print("native API gap report:")
    print(
        "  python3 scripts/plan_semantic_state_migration.py --api-diff "
        f"--source-type {source_type} --native-type {args.target_type} --method-limit {args.batch_size}"
    )
    print("candidate bridge stubs for simple missing methods:")
    print(
        "  python3 scripts/plan_semantic_state_migration.py --kind ram-backed-view "
        f"--accessor '^{re.escape(args.source_accessor)}$' "
        f"--emit-bridge-method-stubs --method-limit {args.batch_size}"
    )
    return 0


def run(command: list[str], *, check: bool = True) -> int:
    printable = " ".join(shell_quote(part) if " " in part else part for part in command)
    print(f"$ {printable}")
    completed = subprocess.run(command, cwd=REPO_ROOT, check=False)
    if check and completed.returncode != 0:
        raise SystemExit(completed.returncode)
    return completed.returncode


def codemod_command(args: argparse.Namespace, map_path: Path) -> list[str]:
    command = [
        "python3",
        "scripts/migrate_semantic_view_methods.py",
        "--map-file",
        relative(map_path),
    ]
    if args.apply:
        command.append("--apply")
    else:
        command.append("--summary")
    if args.rewrite_safe_aliases:
        command.append("--rewrite-safe-aliases")
    if args.rewrite_partial_read_aliases:
        command.append("--rewrite-partial-read-aliases")
    command.extend(str(path) for path in (args.path or [Path("crates/zelda3/src")]))
    return command


def print_next_commands(args: argparse.Namespace, map_path: Path) -> None:
    apply_command = [
        "python3",
        "scripts/assist_semantic_view_migration.py",
        "--source-accessor",
        args.source_accessor,
        "--target-accessor",
        args.target_accessor,
        "--target-type",
        args.target_type,
        "--map-file",
        relative(map_path),
        "--apply",
    ]
    for pattern in args.method or []:
        apply_command.extend(["--method", pattern])
    for pattern in args.exclude_method or []:
        apply_command.extend(["--exclude-method", pattern])
    if args.rewrite_safe_aliases:
        apply_command.append("--rewrite-safe-aliases")
    if args.rewrite_partial_read_aliases:
        apply_command.append("--rewrite-partial-read-aliases")
    if args.include_native_alias_methods:
        apply_command.append("--include-native-alias-methods")
    print()
    print("apply after native ownership is in place:")
    print("  " + " ".join(shell_quote(part) if " " in part else part for part in apply_command))


def run_apply_checks(args: argparse.Namespace) -> None:
    if args.skip_checks:
        return
    run(["cargo", "fmt"])
    run(["cargo", "check", "-p", "zelda3", "--quiet"])
    run(["python3", "scripts/check_ram_readability.py"])
    run(
        [
            "python3",
            "scripts/plan_semantic_state_migration.py",
            "--kind",
            "ram-backed-view",
            "--accessor",
            f"^{args.source_accessor}$",
            "--limit",
            "0",
            "--method-limit",
            "40",
            "--file-limit",
            "8",
        ]
    )


def main() -> int:
    args = parse_args()
    plan = selected_plan(args.source_accessor)
    if args.recommend:
        return print_recommendation(args, plan)

    map_path = (args.map_file or generated_map_path(args.source_accessor, args.target_accessor)).resolve()
    map_path.parent.mkdir(parents=True, exist_ok=True)

    constants = planner.constants_by_name()
    blocker_methods_by_const = write_blockers_by_const(args, constants)
    text, method_counts, blocked_counts = build_map(
        plan,
        args.target_accessor,
        args.target_type,
        compile_patterns(args.method),
        compile_patterns(args.exclude_method),
        args.method_limit,
        constants,
        blocker_methods_by_const,
        args.include_native_alias_methods,
    )
    map_path.write_text(text)

    print(f"wrote semantic migration map: {relative(map_path)}", flush=True)
    if method_counts:
        methods = ", ".join(f"{method} x{count}" for method, count in method_counts.most_common())
        print(f"mapped methods: {methods}", flush=True)
    if blocked_counts:
        methods = ", ".join(f"{method} x{count}" for method, count in blocked_counts.most_common())
        print(f"blocked by paired mutators: {methods}", flush=True)

    if not method_counts:
        print("mapped methods: none")
        if blocked_counts:
            print(
                "all matching methods are blocked by paired mutators that are not native-owned yet",
                file=sys.stderr,
            )
        else:
            print("no same-name source/native methods matched the selected filters", file=sys.stderr)
        return 1

    run(codemod_command(args, map_path), check=args.apply)
    if args.apply:
        run_apply_checks(args)
    else:
        print_next_commands(args, map_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
