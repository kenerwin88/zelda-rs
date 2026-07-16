"""Shared input-script parsing for cross-engine parity runners.

The Rust runner accepts symbolic SNES button names and includes.  The C runner
accepts only numeric input words, so cross-engine harnesses must normalize the
script once and pass the exact same expanded rules to both processes.
"""

from __future__ import annotations

from pathlib import Path


BUTTON_BITS = {
    "B": 1 << 0,
    "Y": 1 << 1,
    "SELECT": 1 << 2,
    "START": 1 << 3,
    "UP": 1 << 4,
    "DOWN": 1 << 5,
    "LEFT": 1 << 6,
    "RIGHT": 1 << 7,
    "A": 1 << 8,
    "X": 1 << 9,
    "L": 1 << 10,
    "R": 1 << 11,
    "NONE": 0,
}


def parse_buttons(spec: str) -> int:
    spec = spec.strip()
    if not spec or spec.upper() == "NONE":
        return 0
    if spec.lower().startswith("0x"):
        value = int(spec, 16)
        if not 0 <= value <= 0xFFFF:
            raise ValueError(f"input word out of range: {spec}")
        return value
    value = 0
    for token in spec.replace(",", "+").replace("|", "+").split("+"):
        name = token.strip().upper()
        if not name:
            continue
        try:
            value |= BUTTON_BITS[name]
        except KeyError as error:
            raise ValueError(f"unknown button: {name}") from error
    return value


def _expanded_rules(path: Path, stack: tuple[Path, ...]) -> list[tuple[str, int]]:
    path = path.resolve()
    if path in stack:
        chain = " -> ".join(str(entry) for entry in (*stack, path))
        raise ValueError(f"recursive input script include: {chain}")
    rules: list[tuple[str, int]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if parts[0].lower() == "include":
            if len(parts) != 2:
                raise ValueError(f"{path}:{line_number}: include takes one path")
            rules.extend(_expanded_rules(path.parent / parts[1], (*stack, path)))
            continue
        if len(parts) < 2:
            raise ValueError(f"{path}:{line_number}: missing input buttons")
        frame_spec = parts[0]
        button_spec = "+".join(parts[1:])
        try:
            value = parse_buttons(button_spec)
        except ValueError as error:
            raise ValueError(f"{path}:{line_number}: {error}") from error
        rules.append((frame_spec, value))
    return rules


def numeric_input_script(path: Path) -> str:
    """Return an include-free script accepted identically by C and Rust."""
    return "".join(
        f"{frame_spec} 0x{value:04x}\n"
        for frame_spec, value in _expanded_rules(path, ())
    )


def numeric_input_history(events: list[tuple[int, int]]) -> str:
    """Compress a captured per-frame controller stream into Rust's script format."""
    text = "# Deterministic controller stream captured once per game frame.\n"
    if not events:
        return text

    start, value = events[0]
    end = start
    runs: list[tuple[int, int, int]] = []
    for frame, next_value in events[1:]:
        if frame == end + 1 and next_value == value:
            end = frame
            continue
        runs.append((start, end, value))
        start = end = frame
        value = next_value
    runs.append((start, end, value))

    for start, end, value in runs:
        if value == 0:
            continue
        frame_spec = str(start) if start == end else f"{start}..{end}"
        text += f"{frame_spec} 0x{value:04x}\n"
    return text
