#!/usr/bin/env python3

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(*command: str) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def binary_symbols(binary: Path) -> str:
    return subprocess.run(
        ("nm", str(binary)),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Verify modern-only and audio-oracle Cargo build boundaries."
    )
    parser.add_argument(
        "--binary",
        type=Path,
        default=ROOT / "target" / "debug" / "zelda3",
        help="modern-only executable to inspect after the default build",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    run("cargo", "build", "-p", "zelda3-bin", "--features", "audio-oracle")
    oracle_symbols = binary_symbols(args.binary)
    if "spc_player_create" not in oracle_symbols:
        raise SystemExit("audio-oracle executable is missing SpcPlayer runtime symbols")
    run(str(args.binary), "--standalone-smoke", "2")

    run("cargo", "build", "-p", "zelda3-bin", "--no-default-features")
    symbols = binary_symbols(args.binary)
    forbidden = tuple(
        symbol for symbol in ("spc_player", "SpcPlayer") if symbol in symbols
    )
    if forbidden:
        raise SystemExit(
            "modern-only executable contains legacy audio symbols: "
            + ", ".join(forbidden)
        )

    print("audio build modes verified; modern-only executable has no SpcPlayer symbols")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
