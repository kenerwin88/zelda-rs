#!/usr/bin/env python3
"""Classify a captured display-oracle mismatch by publication domain.

The trace comparator can emit ``display_oracle.jsonl`` with the authoritative
Snes9x scanout, the selected Rust scanout, and alternate Rust publication
candidates.  This tool reduces that large record to the mismatched domains and
the candidate generation that most closely matches each one.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


FIELDS = (
    "mode",
    "brightness",
    "forced_blank",
    "brightness_white",
    "mode7_scanout_brightness_override",
    "fixed_color",
    "display_control",
    "bg_scroll",
    "cgram",
    "presented_oam",
    "window_scanlines",
    "presented_clip",
    "mode7",
    "mode7_scanlines",
    "presented_obj_tile_cache_valid",
    "presented_obj_tile_cache",
)

DOMAIN_FIELDS = {
    "display-registers": (
        "mode",
        "brightness",
        "forced_blank",
        "brightness_white",
        "mode7_scanout_brightness_override",
        "fixed_color",
        "display_control",
        "bg_scroll",
    ),
    "cgram": ("cgram",),
    "presented-oam": ("presented_oam",),
    "window-raster": ("window_scanlines", "presented_clip"),
    "mode7-raster": ("mode7", "mode7_scanlines"),
    "obj-cache": ("presented_obj_tile_cache_valid", "presented_obj_tile_cache"),
}


def field_is_available(field: str, value: Any) -> bool:
    if value is None:
        return False
    if field == "brightness_white" and value == -1:
        return False
    return True


def mismatch_count(left: Any, right: Any) -> int:
    if isinstance(left, list) and isinstance(right, list):
        overlap = sum(mismatch_count(a, b) for a, b in zip(left, right))
        return overlap + abs(len(left) - len(right))
    if isinstance(left, dict) and isinstance(right, dict):
        keys = set(left) | set(right)
        return sum(
            1
            if key not in left or key not in right
            else mismatch_count(left[key], right[key])
            for key in keys
        )
    return int(left != right)


def field_mismatch_count(
    field: str, left: Any, right: Any, oracle: dict[str, Any]
) -> int:
    if field != "presented_obj_tile_cache":
        return mismatch_count(left, right)
    valid = oracle.get("presented_obj_tile_cache_valid")
    if not isinstance(left, list) or not isinstance(right, list) or not isinstance(valid, list):
        return mismatch_count(left, right)
    if not valid or any(value not in (0, 1) for value in valid):
        return 0
    return sum(
        mismatch_count(left[tile * 64 : (tile + 1) * 64], right[tile * 64 : (tile + 1) * 64])
        for tile, is_valid in enumerate(valid)
        if is_valid == 1
    )


def classify_fields(field_mismatches: dict[str, int]) -> str:
    changed = {field for field, count in field_mismatches.items() if count}
    if not changed:
        return "exact"
    if changed <= {"brightness", "forced_blank", "brightness_white"}:
        return "active-display-blanking"
    if changed == {"cgram"}:
        return "cgram-generation"
    if changed == {"presented_oam"}:
        return "presented-oam-generation"
    if changed <= {"window_scanlines", "presented_clip"}:
        return "window-raster-generation"
    if changed <= {"mode7", "mode7_scanlines"}:
        return "mode7-raster-generation"
    if changed <= {"presented_obj_tile_cache_valid", "presented_obj_tile_cache"}:
        return "obj-cache-generation"
    if changed <= set(DOMAIN_FIELDS["display-registers"]):
        return "display-register-generation"
    return "multi-domain-publication"


def effective_mismatches(
    raw: dict[str, int], oracle: dict[str, Any], rust: dict[str, Any]
) -> tuple[dict[str, int], dict[str, int]]:
    """Remove domains that cannot affect the selected active scanout."""
    effective = dict(raw)
    oracle_black = oracle.get("forced_blank") is True or oracle.get("brightness") == 0
    rust_black = rust.get("forced_blank") is True or rust.get("brightness") == 0
    blanking_fields = {"brightness", "forced_blank", "brightness_white"}
    if oracle_black != rust_black and any(field in effective for field in blanking_fields):
        effective = {
            field: count
            for field, count in effective.items()
            if field in blanking_fields
        }
    else:
        if oracle.get("mode") != 7 and rust.get("mode") != 7:
            effective.pop("mode7", None)
            effective.pop("mode7_scanlines", None)
        oracle_obj_valid = oracle.get("presented_obj_tile_cache_valid")
        if not isinstance(oracle_obj_valid, list) or not any(oracle_obj_valid):
            effective.pop("presented_obj_tile_cache_valid", None)
            effective.pop("presented_obj_tile_cache", None)
    suppressed = {field: count for field, count in raw.items() if field not in effective}
    return effective, suppressed


def analyze_record(record: dict[str, Any]) -> dict[str, Any]:
    oracle = record["oracle"]
    rust = record["rust"]
    candidates = record.get("rust_candidates", [])
    field_mismatches = {
        field: field_mismatch_count(field, rust.get(field), oracle.get(field), oracle)
        for field in FIELDS
        if field in rust
        and field in oracle
        and field_is_available(field, rust[field])
        and field_is_available(field, oracle[field])
    }
    raw_changed_fields = {
        field: count for field, count in field_mismatches.items() if count
    }
    changed_fields, suppressed_mismatches = effective_mismatches(
        raw_changed_fields, oracle, rust
    )
    exact_candidates: dict[str, list[str]] = {}
    for field in changed_fields:
        exact_candidates[field] = [
            candidate.get("name", "unnamed")
            for candidate in candidates
            if field in candidate
            and field_mismatch_count(field, candidate[field], oracle[field], oracle) == 0
        ]

    best_candidates: dict[str, list[dict[str, Any]]] = {}
    for domain, fields in DOMAIN_FIELDS.items():
        if not any(field in changed_fields for field in fields):
            continue
        scored = []
        for candidate in candidates:
            covered = [field for field in fields if field in candidate and field in oracle]
            if not covered:
                continue
            score = sum(
                field_mismatch_count(field, candidate[field], oracle[field], oracle)
                for field in covered
            )
            scored.append(
                {
                    "name": candidate.get("name", "unnamed"),
                    "mismatches": score,
                    "covered_fields": covered,
                }
            )
        if scored:
            lowest = min(item["mismatches"] for item in scored)
            best_candidates[domain] = [
                item for item in scored if item["mismatches"] == lowest
            ]

    return {
        "frame": record.get("frame"),
        "stage": record.get("stage"),
        "classification": classify_fields(changed_fields),
        "field_mismatches": changed_fields,
        "suppressed_mismatches": suppressed_mismatches,
        "exact_candidates": exact_candidates,
        "best_candidates": best_candidates,
        "rust_context": record.get("rust_context"),
    }


def load_record(path: Path, frame: int | None) -> dict[str, Any]:
    if path.is_dir():
        path = path / "display_oracle.jsonl"
    records = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if frame is not None:
        records = [record for record in records if record.get("frame") == frame]
    if not records:
        raise ValueError(f"no display-oracle record found in {path}")
    return records[0]


def format_report(report: dict[str, Any]) -> str:
    lines = [
        f"display_mismatch frame={report['frame']} stage={report['stage']}",
        f"classification={report['classification']}",
    ]
    fields = " ".join(
        f"{name}={count}" for name, count in report["field_mismatches"].items()
    )
    lines.append(f"mismatches {fields or 'none'}")
    if report["suppressed_mismatches"]:
        suppressed = " ".join(
            f"{name}={count}"
            for name, count in report["suppressed_mismatches"].items()
        )
        lines.append(f"suppressed_by_scanout {suppressed}")
    for field, names in report["exact_candidates"].items():
        if names:
            lines.append(f"exact_candidate {field}={','.join(names)}")
    for domain, candidates in report["best_candidates"].items():
        rendered = ",".join(
            f"{item['name']}:{item['mismatches']}"
            f"/{'+'.join(item['covered_fields'])}"
            for item in candidates
        )
        lines.append(f"best_candidate {domain}={rendered}")
    if report.get("rust_context") is not None:
        lines.append(
            "context=" + json.dumps(report["rust_context"], separators=(",", ":"))
        )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path, help="display_oracle.jsonl or its session")
    parser.add_argument("--frame", type=int)
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    args = parser.parse_args()
    try:
        report = analyze_record(load_record(args.capture, args.frame))
    except (OSError, ValueError, json.JSONDecodeError) as error:
        parser.error(str(error))
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(format_report(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
