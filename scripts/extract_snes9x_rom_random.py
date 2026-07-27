#!/usr/bin/env python3
"""Extract a host-run-keyed Zelda RNG replay script from a Snes9x JSONL trace."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import TextIO


RNG_SEED_ADDRESS = 0x0FA1


def extract_samples(lines: TextIO) -> list[tuple[int, int]]:
    samples: list[tuple[int, int]] = []
    for line_number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"line {line_number}: invalid JSON: {error}") from error
        if event.get("event") != "rng-write" or event.get("address") != RNG_SEED_ADDRESS:
            continue
        run = event.get("run")
        value = event.get("value")
        if not isinstance(run, int) or run < 0:
            raise ValueError(
                f"line {line_number}: RNG write is missing a non-negative integer run"
            )
        if not isinstance(value, int) or not 0 <= value <= 0xFF:
            raise ValueError(
                f"line {line_number}: RNG write has invalid byte value {value!r}"
            )
        if samples and run < samples[-1][0]:
            raise ValueError(
                f"line {line_number}: retro_run index {run} precedes {samples[-1][0]}"
            )
        samples.append((run, value))
    return samples


def write_script(samples: list[tuple[int, int]], output: TextIO) -> None:
    output.write(
        "# Cartridge $8dba71 beam-counter RNG outputs, keyed by zero-based retro_run.\n"
        "# Generated from the trace event's explicit run index; Snes9x's completed-frame\n"
        "# counter can advance inside one retro_run and must not be used as this coordinate.\n"
    )
    for run, value in samples:
        output.write(f"{run} 0x{value:02x}\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="Snes9x JSONL trace")
    parser.add_argument("--output", type=Path, help="write here instead of stdout")
    args = parser.parse_args()

    with args.trace.open() as trace:
        samples = extract_samples(trace)
    if not samples:
        raise ValueError("trace contains no cartridge RNG writes")

    if args.output is None:
        write_script(samples, sys.stdout)
    else:
        with args.output.open("w") as output:
            write_script(samples, output)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        raise SystemExit(f"error: {error}") from error
