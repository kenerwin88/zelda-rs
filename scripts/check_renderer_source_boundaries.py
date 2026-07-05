#!/usr/bin/env python3
"""Guard renderer-owned source rendering boundaries.

The live/default GPU paths should hand a `GpuFrame` plus source table to the
renderer crate. The binary may manually extract source-backed modern frames only
when it needs intermediate draw data for diagnostics or authoring metadata.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MAIN_RS = REPO / "zelda3-bin" / "src" / "main.rs"

MANUAL_EXTRACT = "extract_modern_frame_from_sources"
REQUIRED_RENDERER_OWNED_CALLS = (
    "present_modern_variant_gpu_from_sources",
    "present_modern_gpu_from_sources",
    "render_rgba_with_live_index_base_from_sources_traced",
    "render_rgba_with_live_index_base_from_sources",
    "render_rgba_from_sources",
    "render_modern_frame_full_scaled_from_sources",
)


@dataclass(frozen=True)
class Occurrence:
    line_number: int
    function: str | None
    line: str
    context: str


def enclosing_function(lines: list[str], index: int) -> str | None:
    fn_re = re.compile(r"^\s*fn\s+([A-Za-z0-9_]+)\b")
    for line in reversed(lines[: index + 1]):
        match = fn_re.match(line)
        if match:
            return match.group(1)
    return None


def context_window(lines: list[str], index: int, radius: int = 45) -> str:
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)
    return "\n".join(lines[start:end])


def manual_extract_occurrences(source: str) -> list[Occurrence]:
    lines = source.splitlines()
    occurrences: list[Occurrence] = []
    for index, line in enumerate(lines):
        if MANUAL_EXTRACT not in line:
            continue
        stripped = line.lstrip()
        if stripped.startswith("//") or stripped.startswith("///"):
            continue
        occurrences.append(
            Occurrence(
                line_number=index + 1,
                function=enclosing_function(lines, index),
                line=line.strip(),
                context=context_window(lines, index),
            )
        )
    return occurrences


def allowed_manual_extract(occurrence: Occurrence) -> bool:
    if occurrence.function == "run_dump_hd_capture":
        return "build_hd_placement_map" in occurrence.context
    return False


def check_source_text(source: str) -> list[str]:
    errors: list[str] = []
    for required in REQUIRED_RENDERER_OWNED_CALLS:
        if required not in source:
            errors.append(f"missing renderer-owned source API call: {required}")

    occurrences = manual_extract_occurrences(source)
    for occurrence in occurrences:
        if not allowed_manual_extract(occurrence):
            fn = occurrence.function or "<module>"
            errors.append(
                "manual source extraction escaped renderer boundary at "
                f"zelda3-bin/src/main.rs:{occurrence.line_number} "
                f"in {fn}: {occurrence.line}"
            )
    return errors


def main() -> int:
    source = MAIN_RS.read_text()
    errors = check_source_text(source)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(
        "renderer source boundary ok "
        f"manual_extracts={len(manual_extract_occurrences(source))} "
        f"renderer_owned_apis={len(REQUIRED_RENDERER_OWNED_CALLS)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
