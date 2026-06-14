#!/usr/bin/env python3
"""Plan or apply the next safe semantic-view migration worklist.

By default this is an advisor. With --apply it repeatedly asks the single-slice
migration assistant for same-name methods that are already present on the
native target, excludes methods blocked by not-yet-native paired mutators, and
applies the existing semantic-view codemod in batches.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import assist_semantic_view_migration as assistant


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MAP_DIR = REPO_ROOT / "target" / "semantic-migration"


@dataclass(frozen=True)
class MigrationTarget:
    name: str
    source_accessor: str
    target_accessor: str
    target_type: str
    paired_mutator_accessor: str | None = None
    paired_target_type: str | None = None


TARGETS = [
    MigrationTarget(
        name="player-read",
        source_accessor="player_state",
        target_accessor="follower_link_state",
        target_type="FollowerLinkState",
        paired_mutator_accessor="player_state_mut",
        paired_target_type="NativeFollowerLinkBridgeMut",
    ),
    MigrationTarget(
        name="player-write",
        source_accessor="player_state_mut",
        target_accessor="follower_link_state_mut",
        target_type="NativeFollowerLinkBridgeMut",
    ),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--target",
        action="append",
        choices=[target.name for target in TARGETS],
        help="target slice to inspect; repeat for multiple slices; defaults to all known targets",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=16,
        help="number of top methods to include in each suggested batch",
    )
    parser.add_argument(
        "--min-uses",
        type=int,
        default=1,
        help="hide methods used fewer than this many times",
    )
    parser.add_argument(
        "--method",
        action="append",
        help="optional method regex forwarded to each slice recommendation",
    )
    parser.add_argument(
        "--exclude-method",
        action="append",
        help="optional method exclusion regex forwarded to each slice recommendation",
    )
    parser.add_argument(
        "--stubs",
        action="store_true",
        help="also print simple native bridge stubs for missing methods on each source accessor",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="apply automation-ready same-name method batches instead of only printing recommendations",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=4,
        help="maximum apply passes per target; each pass recomputes the current backlog",
    )
    parser.add_argument(
        "--method-limit",
        type=int,
        default=0,
        help="maximum mapped methods per apply pass; 0 means use --batch-size",
    )
    parser.add_argument(
        "--skip-final-checks",
        action="store_true",
        help="with --apply, skip fmt/readability/check follow-up gates",
    )
    parser.add_argument("--list-targets", action="store_true", help="list known targets and exit")
    return parser.parse_args()


def selected_targets(names: list[str] | None) -> list[MigrationTarget]:
    if not names:
        return TARGETS
    wanted = set(names)
    return [target for target in TARGETS if target.name in wanted]


def run(command: list[str]) -> int:
    printable = " ".join(command)
    print(f"$ {printable}", flush=True)
    completed = subprocess.run(command, cwd=REPO_ROOT, check=False)
    print(flush=True)
    return completed.returncode


def checked_run(command: list[str]) -> None:
    result = run(command)
    if result != 0:
        raise SystemExit(result)


def recommendation_command(target: MigrationTarget, args: argparse.Namespace) -> list[str]:
    command = [
        "python3",
        "scripts/assist_semantic_view_migration.py",
        "--source-accessor",
        target.source_accessor,
        "--target-accessor",
        target.target_accessor,
        "--target-type",
        target.target_type,
        "--recommend",
        "--batch-size",
        str(args.batch_size),
        "--min-uses",
        str(args.min_uses),
    ]
    if target.paired_mutator_accessor:
        command.extend(["--paired-mutator-accessor", target.paired_mutator_accessor])
    if target.paired_target_type:
        command.extend(["--paired-target-type", target.paired_target_type])
    for pattern in args.method or []:
        command.extend(["--method", pattern])
    for pattern in args.exclude_method or []:
        command.extend(["--exclude-method", pattern])
    return command


def stubs_command(target: MigrationTarget, args: argparse.Namespace) -> list[str]:
    return [
        "python3",
        "scripts/plan_semantic_state_migration.py",
        "--kind",
        "ram-backed-view",
        "--accessor",
        f"^{target.source_accessor}$",
        "--emit-bridge-method-stubs",
        "--method-limit",
        str(args.batch_size),
    ]


def apply_target_batch(target: MigrationTarget, args: argparse.Namespace, iteration: int) -> bool:
    plan = assistant.selected_plan(target.source_accessor)
    constants = assistant.planner.constants_by_name()
    include_patterns = assistant.compile_patterns(args.method)
    exclude_patterns = assistant.compile_patterns(args.exclude_method)
    blocker_args = argparse.Namespace(
        paired_mutator_accessor=target.paired_mutator_accessor,
        paired_target_type=target.paired_target_type,
    )
    blockers = assistant.write_blockers_by_const(blocker_args, constants)

    map_path = (
        DEFAULT_MAP_DIR
        / f"bulk-{target.name}-{target.source_accessor}-to-{target.target_accessor}-{iteration}.map"
    )
    method_limit = args.method_limit if args.method_limit > 0 else args.batch_size
    text, method_counts, blocked_counts = assistant.build_map(
        plan,
        target.target_accessor,
        target.target_type,
        include_patterns,
        exclude_patterns,
        method_limit,
        constants,
        blockers,
    )
    map_path.parent.mkdir(parents=True, exist_ok=True)
    map_path.write_text(text)

    if not method_counts:
        if blocked_counts:
            methods = ", ".join(
                f"{method} x{count}" for method, count in blocked_counts.most_common(args.batch_size)
            )
            print(f"{target.name}: no apply-ready methods; blocked by paired mutators: {methods}")
        else:
            print(f"{target.name}: no apply-ready methods")
        return False

    methods = ", ".join(f"{method} x{count}" for method, count in method_counts.most_common())
    print(f"{target.name}: applying {sum(method_counts.values())} use(s): {methods}", flush=True)
    checked_run(
        [
            "python3",
            "scripts/migrate_semantic_view_methods.py",
            "--apply",
            "--rewrite-safe-aliases",
            "--map-file",
            str(map_path.relative_to(REPO_ROOT)),
            "crates/zelda3/src",
        ]
    )
    return True


def apply_targets(args: argparse.Namespace) -> int:
    for target in selected_targets(args.target):
        print("=" * 80, flush=True)
        print(
            f"applying {target.name}: {target.source_accessor} -> "
            f"{target.target_accessor} ({target.target_type})",
            flush=True,
        )
        print("=" * 80, flush=True)
        changed = False
        for iteration in range(1, args.iterations + 1):
            if not apply_target_batch(target, args, iteration):
                break
            changed = True
        if not changed:
            print(f"{target.name}: nothing changed")

    checked_run(["python3", "scripts/migrate_native_state_access.py", "--apply", "--accessor-regex", "^follower_link_state$"])
    if not args.skip_final_checks:
        checked_run(["cargo", "fmt"])
        checked_run(
            [
                "python3",
                "-m",
                "py_compile",
                "scripts/check_ram_readability.py",
                "scripts/plan_semantic_state_migration.py",
                "scripts/assist_semantic_view_migration.py",
                "scripts/bulk_semantic_view_migration.py",
                "scripts/migrate_semantic_view_methods.py",
                "scripts/migrate_native_state_access.py",
            ]
        )
        checked_run(
            [
                "python3",
                "scripts/check_ram_readability.py",
                "--report-migration-progress",
                "--migration-progress-limit",
                "70",
            ]
        )
        checked_run(["cargo", "check", "-p", "zelda3", "--quiet"])
        checked_run(["cargo", "test", "-p", "zelda3", "--lib", "--no-run", "--quiet"])
    return 0


def main() -> int:
    args = parse_args()
    if args.list_targets:
        for target in TARGETS:
            print(
                f"{target.name}: {target.source_accessor} -> "
                f"{target.target_accessor} ({target.target_type})"
            )
        return 0
    if args.apply:
        return apply_targets(args)

    failures = 0
    for target in selected_targets(args.target):
        print("=" * 80, flush=True)
        print(
            f"{target.name}: {target.source_accessor} -> "
            f"{target.target_accessor} ({target.target_type})",
            flush=True,
        )
        print("=" * 80, flush=True)
        failures += int(run(recommendation_command(target, args)) != 0)
        if args.stubs:
            print(f"{target.name}: missing native bridge method candidates", flush=True)
            print("-" * 80, flush=True)
            failures += int(run(stubs_command(target, args)) != 0)
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
