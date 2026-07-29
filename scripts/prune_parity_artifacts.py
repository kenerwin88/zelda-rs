#!/usr/bin/env python3
"""Prune superseded local parity artifacts while retaining canonical checkpoints."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_TARGET = ROOT / "target"
CHECKPOINT_ROOT_NAME = "parity-checkpoints"
RETAINED_CHECKPOINT_LINEAGES = frozenset(
    {
        "canonical-main-window-reset",
        "canonical-main-7653d6a2",
        "canonical-full-7653d6a2",
    }
)


def artifact_size(path: Path) -> int:
    if path.is_symlink() or path.is_file():
        return path.lstat().st_size
    return sum(
        child.lstat().st_size
        for child in path.rglob("*")
        if child.is_file() or child.is_symlink()
    )


def removable_artifacts(target_dir: Path) -> list[Path]:
    artifacts = [
        path
        for path in target_dir.glob("parity-*")
        if path.name != CHECKPOINT_ROOT_NAME
    ]
    checkpoint_root = target_dir / CHECKPOINT_ROOT_NAME
    if checkpoint_root.is_dir():
        artifacts.extend(
            path
            for path in checkpoint_root.iterdir()
            if path.name not in RETAINED_CHECKPOINT_LINEAGES
        )
    return sorted(artifacts)


def remove_artifact(path: Path) -> None:
    if path.is_symlink() or path.is_file():
        path.unlink()
    else:
        shutil.rmtree(path)


def prune(target_dir: Path, *, apply: bool) -> tuple[int, int]:
    target_dir = target_dir.resolve()
    if target_dir.name != "target":
        raise ValueError(f"refusing to prune non-target directory: {target_dir}")

    artifacts = removable_artifacts(target_dir)
    reclaimed_bytes = 0
    action = "remove" if apply else "would remove"
    for path in artifacts:
        size = artifact_size(path)
        reclaimed_bytes += size
        print(f"{action}: {path} ({size / 1024 / 1024:.1f} MiB)")
        if apply:
            remove_artifact(path)

    mode = "removed" if apply else "would remove"
    print(
        f"{mode} {len(artifacts)} artifacts "
        f"({reclaimed_bytes / 1024 / 1024 / 1024:.1f} GiB); "
        f"retained checkpoints: {', '.join(sorted(RETAINED_CHECKPOINT_LINEAGES))}"
    )
    return len(artifacts), reclaimed_bytes


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--apply",
        action="store_true",
        help="delete the listed artifacts; the default is a dry run",
    )
    parser.add_argument(
        "--target-dir",
        type=Path,
        default=DEFAULT_TARGET,
        help=argparse.SUPPRESS,
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    prune(args.target_dir, apply=args.apply)


if __name__ == "__main__":
    main()
