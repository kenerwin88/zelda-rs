#!/usr/bin/env python3
"""Report the first raw Snes9x-vs-Rust state difference from a compare session.

The direct comparator can capture one or more frame boundaries with:

  ZELDA3_DEBUG_WRAM_FRAMES=80-84 \\
  ZELDA3_DEBUG_VRAM_FRAMES=80-84 \\
  target/debug/zelda3 --compare-snes9x-oracle ... --session-dir /tmp/session

This tool intentionally compares emulated memory, not rendered pixels.  Its
output identifies the earliest captured frame and contiguous dirty ranges so a
renderer symptom cannot hide the state producer that first went wrong.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Difference:
    start: int
    end: int

    @property
    def length(self) -> int:
        return self.end - self.start


WRAM_LABELS = (
    (0x0010, 0x0018, "main-loop/NMI flags"),
    (0x00f0, 0x0100, "controller publication"),
    (0x0134, 0x0136, "animated-tile VRAM destination"),
    (0x0adc, 0x0ade, "animated-tile DMA source"),
    (0x1e00, 0x1e70, "intro state block"),
)


def parse_frames(value: str) -> list[int]:
    frames: set[int] = set()
    for item in value.split(","):
        item = item.strip()
        if not item:
            continue
        if "-" in item:
            start, end = item.split("-", 1)
            frames.update(range(int(start), int(end) + 1))
        else:
            frames.add(int(item))
    return sorted(frames)


def differences(left: bytes, right: bytes) -> list[Difference]:
    result: list[Difference] = []
    start: int | None = None
    for index in range(max(len(left), len(right))):
        changed = (left[index] if index < len(left) else None) != (
            right[index] if index < len(right) else None
        )
        if changed and start is None:
            start = index
        elif not changed and start is not None:
            result.append(Difference(start, index))
            start = None
    if start is not None:
        result.append(Difference(start, max(len(left), len(right))))
    return result


def label_for(domain: str, start: int, end: int) -> str:
    if domain != "wram":
        return ""
    labels = [label for low, high, label in WRAM_LABELS if start < high and end > low]
    return f" ({', '.join(labels)})" if labels else ""


def preview(data: bytes, offset: int, count: int = 16) -> str:
    return " ".join(f"{value:02x}" for value in data[offset : offset + count])


def capture_path(session: Path, side: str, domain: str, frame: int) -> Path:
    return session / f"{side}_{domain}_frame_{frame}.bin"


def changed_bytes(previous: bytes, current: bytes) -> bytes:
    """Encode a state transition so unchanged, mismatched reset bytes disappear."""
    length = max(len(previous), len(current))
    return bytes(
        (previous[index] if index < len(previous) else 0)
        ^ (current[index] if index < len(current) else 0)
        for index in range(length)
    )


def report_domain(
    session: Path, domain: str, frames: list[int], limit: int, mode: str
) -> bool:
    previous_rust: bytes | None = None
    previous_oracle: bytes | None = None
    for frame in frames:
        rust_path = capture_path(session, "rust", domain, frame)
        oracle_path = capture_path(session, "oracle", domain, frame)
        if not rust_path.exists() or not oracle_path.exists():
            continue
        rust, oracle = rust_path.read_bytes(), oracle_path.read_bytes()
        if mode == "delta":
            if previous_rust is None or previous_oracle is None:
                previous_rust, previous_oracle = rust, oracle
                continue
            compared_rust = changed_bytes(previous_rust, rust)
            compared_oracle = changed_bytes(previous_oracle, oracle)
        else:
            compared_rust, compared_oracle = rust, oracle
        previous_rust, previous_oracle = rust, oracle
        changed = differences(compared_rust, compared_oracle)
        if not changed:
            continue
        print(f"first_{domain}_{mode}_divergence frame={frame} ranges={len(changed)}")
        for item in changed[:limit]:
            print(
                f"  ${item.start:04x}-${item.end - 1:04x} length={item.length}"
                f" rust=[{preview(compared_rust, item.start)}]"
                f" oracle=[{preview(compared_oracle, item.start)}]"
                f"{label_for(domain, item.start, item.end)}"
            )
        if len(changed) > limit:
            print(f"  ... {len(changed) - limit} additional range(s)")
        return True
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("session", type=Path)
    parser.add_argument("--frames", default="0-1000000", help="captured frames, e.g. 80-84")
    parser.add_argument("--domain", choices=("wram", "vram", "all"), default="all")
    parser.add_argument(
        "--mode",
        choices=("delta", "snapshot"),
        default="delta",
        help="compare frame-to-frame writes (default) or absolute post-frame state",
    )
    parser.add_argument("--limit", type=int, default=8)
    args = parser.parse_args()

    domains = ("wram", "vram") if args.domain == "all" else (args.domain,)
    found_capture = False
    found_difference = False
    frames = parse_frames(args.frames)
    for domain in domains:
        paths_exist = any(
            capture_path(args.session, "rust", domain, frame).exists()
            for frame in frames
        )
        found_capture |= paths_exist
        found_difference |= report_domain(args.session, domain, frames, args.limit, args.mode)
    if not found_capture:
        parser.error("no matching captures; enable the matching ZELDA3_DEBUG_*_FRAMES variable")
    if not found_difference:
        print("no raw state divergence in captured frames")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
