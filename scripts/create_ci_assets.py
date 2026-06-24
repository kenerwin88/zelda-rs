#!/usr/bin/env python3
"""Create placeholder assets for ROM-free binary build checks."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path


ASSET_COUNT = 165
DEFAULT_ASSET_SIZE = 16 * 1024
SIGNATURE_PREFIX = b"Zelda3_v0     \n\0"
SIGNATURE_SIZE = 48


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Create a generated/zelda3_assets-compatible directory for CI. "
            "The files are zero-filled placeholders, not playable game assets."
        )
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("target/ci-assets/zelda3_assets"),
        help="directory to create, replacing it if it already exists",
    )
    parser.add_argument(
        "--asset-size",
        type=int,
        default=DEFAULT_ASSET_SIZE,
        help="bytes to write for each placeholder asset",
    )
    return parser.parse_args()


def write_signature(out_dir: Path) -> None:
    if len(SIGNATURE_PREFIX) > SIGNATURE_SIZE:
        raise ValueError("signature prefix exceeds fixed signature size")
    padding = bytes(SIGNATURE_SIZE - len(SIGNATURE_PREFIX))
    (out_dir / "asset_signature.bin").write_bytes(SIGNATURE_PREFIX + padding)


def write_key_signature(out_dir: Path, names: list[str]) -> None:
    payload = b"\0".join(name.encode("ascii") for name in names) + b"\0"
    (out_dir / "asset_key_signature.bin").write_bytes(payload)


def create_assets(out_dir: Path, asset_size: int) -> None:
    if asset_size <= 0:
        raise ValueError("--asset-size must be positive")

    if out_dir.exists():
        shutil.rmtree(out_dir)

    assets_dir = out_dir / "assets"
    assets_dir.mkdir(parents=True)
    write_signature(out_dir)

    names = [f"ci_asset_{index:03d}" for index in range(ASSET_COUNT)]
    write_key_signature(out_dir, names)

    payload = bytes(asset_size)
    manifest_assets = []
    for index, name in enumerate(names):
        (assets_dir / f"{index:03d}-{name}.bin").write_bytes(payload)
        manifest_assets.append(
            {
                "index": index,
                "name": name,
                "file": f"assets/{index:03d}-{name}.bin",
                "size": len(payload),
                "sha1": hashlib.sha1(payload).hexdigest(),
            }
        )

    (out_dir / "manifest.json").write_text(
        json.dumps(
            {
                "asset_count": ASSET_COUNT,
                "asset_key_signature": "asset_key_signature.bin",
                "asset_signature": "asset_signature.bin",
                "assets": manifest_assets,
                "image_previews": [],
                "source_tool": "scripts/create_ci_assets.py",
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )


def main() -> None:
    args = parse_args()
    create_assets(args.out_dir, args.asset_size)
    print(f"created {ASSET_COUNT} placeholder assets in {args.out_dir}")


if __name__ == "__main__":
    main()
