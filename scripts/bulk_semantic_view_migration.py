#!/usr/bin/env python3
"""Print the next safe semantic-view migration worklist.

This is intentionally an advisor, not an aggressive rewriter. It runs the
single-slice migration assistant for the known semantic/native cutover targets
and keeps the actual codemod behind the generated per-slice commands.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


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


def main() -> int:
    args = parse_args()
    if args.list_targets:
        for target in TARGETS:
            print(
                f"{target.name}: {target.source_accessor} -> "
                f"{target.target_accessor} ({target.target_type})"
            )
        return 0

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
