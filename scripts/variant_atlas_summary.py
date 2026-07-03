#!/usr/bin/env python3
"""Summarize compact RGBA variant atlas coverage.

This is a deterministic manifest check. It does not run the game, replay frames,
or invoke the renderer; it answers whether canonical art source refs have a
stable preview/effect entry that the live variant loader can use.
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any


def _load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"missing required manifest: {path}") from exc
    if not isinstance(data, dict):
        raise SystemExit(f"manifest is not a JSON object: {path}")
    return data


def _effect_policy_by_key(effects: dict[str, Any]) -> dict[tuple[str, int, int], str]:
    policy_by_key: dict[tuple[str, int, int], str] = {}
    for effect in effects.get("effects", []):
        if not isinstance(effect, dict):
            continue
        palette = effect.get("palette")
        palette_row = effect.get("palette_row")
        colors_per_row = effect.get("colors_per_row")
        dynamic_policy = effect.get("dynamic_policy")
        if (
            isinstance(palette, str)
            and isinstance(palette_row, int)
            and isinstance(colors_per_row, int)
            and isinstance(dynamic_policy, str)
        ):
            policy_by_key[(palette, palette_row, colors_per_row)] = dynamic_policy
    return policy_by_key


def summarize_variant_atlas(atlas_dir: Path) -> dict[str, Any]:
    art_manifest = _load_json(atlas_dir / "art_tiles.json")
    effects_manifest = _load_json(atlas_dir / "tile_effects.json")
    effect_policies = _effect_policy_by_key(effects_manifest)

    source_refs = 0
    stable_by_loader_rule = 0
    missing_effect_refs = 0
    stable_by_kind: Counter[str] = Counter()
    source_refs_by_kind: Counter[str] = Counter()
    preview_sources: Counter[str] = Counter()

    for art in art_manifest.get("arts", []):
        if not isinstance(art, dict):
            continue
        for ref in art.get("source_refs", []):
            if not isinstance(ref, dict):
                continue
            source_refs += 1
            source_kind = str(ref.get("source_kind", "unknown"))
            preview_source = str(ref.get("preview_source", art.get("preview_source", "unknown")))
            palette = str(ref.get("preview_palette", art.get("preview_palette", "")))
            palette_row = int(ref.get("preview_palette_row", art.get("preview_palette_row", 0)))
            bpp = int(ref.get("bpp", art.get("bpp", 0)))
            colors_per_row = 1 << bpp if bpp >= 0 else 0
            source_refs_by_kind[source_kind] += 1
            preview_sources[preview_source] += 1

            effect_policy = effect_policies.get((palette, palette_row, colors_per_row))
            stable = preview_source == "palette_usage" or effect_policy == "stable"
            if stable:
                stable_by_loader_rule += 1
                stable_by_kind[source_kind] += 1
            else:
                missing_effect_refs += 1

    manifest_source_refs = art_manifest.get("source_ref_count")
    return {
        "art_count": art_manifest.get("art_count", len(art_manifest.get("arts", []))),
        "source_refs": source_refs,
        "manifest_source_refs": manifest_source_refs,
        "stable_by_loader_rule": stable_by_loader_rule,
        "missing_effect_refs": missing_effect_refs,
        "stable_by_kind": dict(sorted(stable_by_kind.items())),
        "source_refs_by_kind": dict(sorted(source_refs_by_kind.items())),
        "preview_sources": dict(sorted(preview_sources.items())),
    }


def _format_counts(prefix: str, counts: dict[str, int]) -> str:
    if not counts:
        return f"{prefix} none"
    return f"{prefix} " + " ".join(f"{key}={value}" for key, value in counts.items())


def format_summary(summary: dict[str, Any]) -> str:
    lines = [
        f"art_count={summary['art_count']}",
        f"source_refs={summary['source_refs']}",
        f"manifest_source_refs={summary['manifest_source_refs']}",
        f"stable_by_loader_rule={summary['stable_by_loader_rule']}",
        f"missing_effect_refs={summary['missing_effect_refs']}",
        _format_counts("stable_by_kind", summary["stable_by_kind"]),
        _format_counts("source_refs_by_kind", summary["source_refs_by_kind"]),
        _format_counts("preview_sources", summary["preview_sources"]),
    ]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "atlas_dir",
        nargs="?",
        default="generated/zelda3_assets/atlas",
        type=Path,
        help="directory containing art_tiles.json and tile_effects.json",
    )
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    args = parser.parse_args()

    summary = summarize_variant_atlas(args.atlas_dir)
    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(format_summary(summary))


if __name__ == "__main__":
    main()
