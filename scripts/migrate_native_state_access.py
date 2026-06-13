#!/usr/bin/env python3
"""Rewrite native-owned state read accessors to direct GameState paths.

This is intentionally narrow. It only handles accessors whose backing state is
already native-owned and dual-synced to RAM. Bridge-backed mutation helpers are
left alone because they still project updates into RAM during the transition.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"


@dataclass(frozen=True)
class AccessorMapping:
    accessor: str
    game_state_path: str
    borrowed_alias: bool = True


ACCESSORS: tuple[AccessorMapping, ...] = (
    AccessorMapping("frame_state", "game_state.frame"),
    AccessorMapping("world_location_state", "game_state.world.location"),
    AccessorMapping("display_state", "game_state.display"),
    AccessorMapping("intro_scene_state", "game_state.ending.intro_scene"),
    AccessorMapping("weather_vane_state", "game_state.world.overworld.weather_vane"),
    AccessorMapping("trinexx_palette_state", "game_state.display.trinexx_palette"),
    AccessorMapping("overworld_map16_load_state", "game_state.world.overworld.map16.active_load", borrowed_alias=False),
    AccessorMapping("overworld_prev_map16_load_state", "game_state.world.overworld.map16.previous_load", borrowed_alias=False),
)


def default_paths() -> list[Path]:
    return sorted(path for path in SRC_ROOT.glob("*.rs") if path.is_file())


def relative(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def rewrite_text(text: str) -> str:
    for mapping in ACCESSORS:
        accessor = re.escape(mapping.accessor)
        path = mapping.game_state_path

        # receiver.accessor().method_or_field, allowing rustfmt line breaks.
        text = re.sub(
            rf"\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*{accessor}\(\)\s*\.",
            rf"\1.{path}.",
            text,
        )

        # let local = receiver.accessor(); for read aliases.
        alias_prefix = r"&" if mapping.borrowed_alias else ""
        text = re.sub(
            rf"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\.{accessor}\(\);",
            rf"let \1 = {alias_prefix}\2.{path};",
            text,
        )

        # let local = *receiver.accessor(); for copy aliases.
        text = re.sub(
            rf"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\*([A-Za-z_][A-Za-z0-9_]*)\.{accessor}\(\);",
            rf"let \1 = \2.{path};",
            text,
        )
    return text


def findings(path: Path, text: str) -> list[str]:
    names = "|".join(re.escape(mapping.accessor) for mapping in ACCESSORS)
    pattern = re.compile(rf"\b(?:{names})\(\)")
    lines = []
    for match in pattern.finditer(text):
        lines.append(f"{relative(path)}:{line_for_offset(text, match.start())}: {match.group(0)}")
    return lines


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--apply",
        action="store_true",
        help="rewrite files in place; without this, only report remaining accessor reads",
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Rust files to scan or rewrite; defaults to crates/zelda3/src/*.rs",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    paths = args.paths or default_paths()
    changed: list[Path] = []
    all_findings: list[str] = []

    for path in paths:
        if path.is_dir():
            files = sorted(path.rglob("*.rs"))
        else:
            files = [path]
        for file_path in files:
            text = file_path.read_text()
            if args.apply:
                next_text = rewrite_text(text)
                if next_text != text:
                    file_path.write_text(next_text)
                    changed.append(file_path)
                    text = next_text
            all_findings.extend(findings(file_path, text))

    if args.apply and changed:
        print("rewrote native state accessors:")
        for path in changed:
            print(f"  {relative(path)}")

    if all_findings:
        print("remaining native read accessor call(s):")
        for finding in all_findings:
            print(f"  {finding}")
        return 1

    print("native state accessors ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
