#!/usr/bin/env python3
"""Summarize compact RGBA variant atlas coverage.

This is a deterministic manifest check. It does not run the game, replay frames,
or invoke the renderer; it answers whether canonical art source refs have a
stable preview/effect entry that the live variant loader can use.
"""

from __future__ import annotations

import argparse
import json
import struct
import sys
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


def load_manifest_summary(path: Path) -> dict[str, Any]:
    manifest = _load_json(path)
    summary = manifest.get("canonical_art_atlas_summary")
    if not isinstance(summary, dict):
        raise SystemExit(f"{path}: missing canonical_art_atlas_summary object")
    return summary


def _png_dimensions(path: Path) -> tuple[int, int]:
    try:
        header = path.read_bytes()[:24]
    except FileNotFoundError as exc:
        raise SystemExit(f"missing required PNG: {path}") from exc
    if len(header) < 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        raise SystemExit(f"not a PNG with an IHDR header: {path}")
    width, height = struct.unpack(">II", header[16:24])
    return width, height


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
    art_png_width, art_png_height = _png_dimensions(atlas_dir / "art_tiles.png")
    effect_policies = _effect_policy_by_key(effects_manifest)

    source_refs = 0
    stable_by_loader_rule = 0
    material_effect_refs = 0
    stable_preview_refs = 0
    requires_live_material_refs = 0
    missing_effect_refs = 0
    stable_by_kind: Counter[str] = Counter()
    source_refs_by_kind: Counter[str] = Counter()
    preview_sources: Counter[str] = Counter()
    counted_art_entries = 0
    invalid_rects: list[str] = []

    for art in art_manifest.get("arts", []):
        if not isinstance(art, dict):
            continue
        counted_art_entries += 1
        art_id = str(art.get("art_id", f"art_index:{counted_art_entries - 1}"))
        rect = art.get("rect")
        if not _rect_is_valid(rect, art_png_width, art_png_height):
            invalid_rects.append(f"{art_id}:{rect!r}")
        for ref in art.get("source_refs", []):
            if not isinstance(ref, dict):
                continue
            source_refs += 1
            source_kind = str(ref.get("source_kind", "unknown"))
            preview_source = str(ref.get("preview_source", art.get("preview_source", "unknown")))
            palette = str(ref.get("preview_palette", art.get("preview_palette", "")))
            palette_row = int(ref.get("preview_palette_row", art.get("preview_palette_row", 0)))
            bpp = int(ref.get("bpp", art.get("bpp", 0)))
            colors_per_row = int(ref.get("runtime_colors_per_row", 1 << bpp if bpp >= 0 else 0))
            source_refs_by_kind[source_kind] += 1
            preview_sources[preview_source] += 1

            effect_policy = effect_policies.get((palette, palette_row, colors_per_row))
            runtime_material = str(
                ref.get(
                    "runtime_material",
                    "palette_lut" if effect_policy is not None else "unknown",
                )
            )
            runtime_material_policy = ref.get("runtime_material_policy")
            if runtime_material == "palette_lut" and runtime_material_policy in {
                "stable",
                "requires_live_palette",
            }:
                stable = runtime_material_policy == "stable"
            else:
                stable = preview_source == "palette_usage" or effect_policy == "stable"
            if stable:
                stable_by_loader_rule += 1
                stable_by_kind[source_kind] += 1
                if runtime_material == "palette_lut" and effect_policy == "stable":
                    material_effect_refs += 1
                else:
                    stable_preview_refs += 1
            else:
                if runtime_material == "palette_lut":
                    requires_live_material_refs += 1
                missing_effect_refs += 1

    manifest_source_refs = art_manifest.get("source_ref_count")
    dynamic_bg3_summary = _summarize_dynamic_bg3_atlas(atlas_dir)
    dialogue_glyph_summary = _summarize_dialogue_glyph_atlas(atlas_dir)
    dialogue_vwf_summary = _summarize_dialogue_vwf_font(atlas_dir)
    dialogue_vwf_glyph_summary = _summarize_dialogue_vwf_glyph_atlas(atlas_dir)
    return {
        "art_count": art_manifest.get("art_count", len(art_manifest.get("arts", []))),
        "counted_art_entries": counted_art_entries,
        "manifest_width": art_manifest.get("width"),
        "manifest_height": art_manifest.get("height"),
        "art_png_width": art_png_width,
        "art_png_height": art_png_height,
        "source_refs": source_refs,
        "manifest_source_refs": manifest_source_refs,
        "stable_by_loader_rule": stable_by_loader_rule,
        "material_effect_refs": material_effect_refs,
        "stable_preview_refs": stable_preview_refs,
        "requires_live_material_refs": requires_live_material_refs,
        "missing_effect_refs": missing_effect_refs,
        "invalid_rect_count": len(invalid_rects),
        "invalid_rects": invalid_rects[:5],
        "stable_by_kind": dict(sorted(stable_by_kind.items())),
        "source_refs_by_kind": dict(sorted(source_refs_by_kind.items())),
        "preview_sources": dict(sorted(preview_sources.items())),
        "dynamic_bg3_art_count": dynamic_bg3_summary["art_count"],
        "dynamic_bg3_counted_art_entries": dynamic_bg3_summary["counted_art_entries"],
        "dynamic_bg3_source_refs": dynamic_bg3_summary["source_refs"],
        "dynamic_bg3_manifest_source_refs": dynamic_bg3_summary["manifest_source_refs"],
        "dynamic_bg3_png_width": dynamic_bg3_summary["png_width"],
        "dynamic_bg3_png_height": dynamic_bg3_summary["png_height"],
        "dynamic_bg3_manifest_width": dynamic_bg3_summary["manifest_width"],
        "dynamic_bg3_manifest_height": dynamic_bg3_summary["manifest_height"],
        "dynamic_bg3_invalid_rect_count": dynamic_bg3_summary["invalid_rect_count"],
        "dynamic_bg3_invalid_rects": dynamic_bg3_summary["invalid_rects"],
        "total_art_count": art_manifest.get("art_count", len(art_manifest.get("arts", [])))
        + dynamic_bg3_summary["art_count"],
        "total_source_refs": source_refs + dynamic_bg3_summary["source_refs"],
        "dialogue_glyph_tile_count": dialogue_glyph_summary["tile_count"],
        "dialogue_glyph_width": dialogue_glyph_summary["width"],
        "dialogue_glyph_height": dialogue_glyph_summary["height"],
        "dialogue_glyph_invalid_rect_count": dialogue_glyph_summary["invalid_rect_count"],
        "dialogue_glyph_invalid_rects": dialogue_glyph_summary["invalid_rects"],
        "dialogue_vwf_glyph_count": dialogue_vwf_summary["glyph_count"],
        "dialogue_vwf_width_table_size": dialogue_vwf_summary["width_table_size"],
        "dialogue_vwf_glyph_atlas_count": dialogue_vwf_glyph_summary["glyph_count"],
        "dialogue_vwf_glyph_atlas_width": dialogue_vwf_glyph_summary["width"],
        "dialogue_vwf_glyph_atlas_height": dialogue_vwf_glyph_summary["height"],
        "dialogue_vwf_glyph_atlas_invalid_rect_count": dialogue_vwf_glyph_summary[
            "invalid_rect_count"
        ],
        "dialogue_vwf_glyph_atlas_invalid_rects": dialogue_vwf_glyph_summary[
            "invalid_rects"
        ],
    }


def _summarize_dynamic_bg3_atlas(atlas_dir: Path) -> dict[str, Any]:
    manifest_path = atlas_dir / "dynamic_bg3_tiles.json"
    png_path = atlas_dir / "dynamic_bg3_tiles.png"
    if not manifest_path.is_file() and not png_path.is_file():
        return {
            "art_count": 0,
            "counted_art_entries": 0,
            "source_refs": 0,
            "manifest_source_refs": 0,
            "png_width": 0,
            "png_height": 0,
            "manifest_width": 0,
            "manifest_height": 0,
            "invalid_rect_count": 0,
            "invalid_rects": [],
        }
    manifest = _load_json(manifest_path)
    png_width, png_height = _png_dimensions(png_path)
    source_refs = 0
    counted_art_entries = 0
    invalid_rects: list[str] = []
    for art in manifest.get("arts", []):
        if not isinstance(art, dict):
            continue
        counted_art_entries += 1
        art_id = str(art.get("art_id", f"dynamic_bg3_art_index:{counted_art_entries - 1}"))
        rect = art.get("rect")
        if not _rect_is_valid(rect, png_width, png_height):
            invalid_rects.append(f"{art_id}:{rect!r}")
        for ref in art.get("source_refs", []):
            if isinstance(ref, dict):
                source_refs += 1
    return {
        "art_count": int(manifest.get("art_count", len(manifest.get("arts", [])))),
        "counted_art_entries": counted_art_entries,
        "source_refs": source_refs,
        "manifest_source_refs": int(manifest.get("source_ref_count", 0)),
        "png_width": png_width,
        "png_height": png_height,
        "manifest_width": int(manifest.get("width", 0)),
        "manifest_height": int(manifest.get("height", 0)),
        "invalid_rect_count": len(invalid_rects),
        "invalid_rects": invalid_rects[:5],
    }


def _summarize_dialogue_glyph_atlas(atlas_dir: Path) -> dict[str, Any]:
    manifest_path = atlas_dir / "dialogue_glyph_tiles.json"
    png_path = atlas_dir / "dialogue_glyph_tiles.png"
    if not manifest_path.is_file() and not png_path.is_file():
        return {
            "tile_count": 0,
            "width": 0,
            "height": 0,
            "invalid_rect_count": 0,
            "invalid_rects": [],
        }
    manifest = _load_json(manifest_path)
    png_width, png_height = _png_dimensions(png_path)
    invalid_rects: list[str] = []
    for index, tile in enumerate(manifest.get("tiles", [])):
        if not isinstance(tile, dict):
            continue
        rect = tile.get("rect")
        if not _rect_is_valid(rect, png_width, png_height):
            invalid_rects.append(f"{tile.get('id', f'tile:{index}')}:{rect!r}")
    return {
        "tile_count": int(manifest.get("tile_count", len(manifest.get("tiles", [])))),
        "width": int(manifest.get("width", 0)),
        "height": int(manifest.get("height", 0)),
        "invalid_rect_count": len(invalid_rects),
        "invalid_rects": invalid_rects[:5],
    }


def _summarize_dialogue_vwf_font(atlas_dir: Path) -> dict[str, Any]:
    manifest_path = atlas_dir / "dialogue_vwf_font.json"
    if not manifest_path.is_file():
        return {"glyph_count": 0, "width_table_size": 0}
    manifest = _load_json(manifest_path)
    return {
        "glyph_count": int(manifest.get("glyph_count", len(manifest.get("glyphs", [])))),
        "width_table_size": int(manifest.get("width_table_size", 0)),
    }


def _summarize_dialogue_vwf_glyph_atlas(atlas_dir: Path) -> dict[str, Any]:
    manifest_path = atlas_dir / "dialogue_vwf_glyphs.json"
    png_path = atlas_dir / "dialogue_vwf_glyphs.png"
    if not manifest_path.is_file() and not png_path.is_file():
        return {
            "glyph_count": 0,
            "width": 0,
            "height": 0,
            "invalid_rect_count": 0,
            "invalid_rects": [],
        }
    manifest = _load_json(manifest_path)
    png_width, png_height = _png_dimensions(png_path)
    invalid_rects: list[str] = []
    for index, glyph in enumerate(manifest.get("glyphs", [])):
        if not isinstance(glyph, dict):
            continue
        rect = glyph.get("rect")
        if not _rect_is_valid(rect, png_width, png_height):
            invalid_rects.append(f"{glyph.get('hex', f'glyph:{index}')}:{rect!r}")
    return {
        "glyph_count": int(manifest.get("glyph_count", len(manifest.get("glyphs", [])))),
        "width": int(manifest.get("width", 0)),
        "height": int(manifest.get("height", 0)),
        "invalid_rect_count": len(invalid_rects),
        "invalid_rects": invalid_rects[:5],
    }


def _format_counts(prefix: str, counts: dict[str, int]) -> str:
    if not counts:
        return f"{prefix} none"
    return f"{prefix} " + " ".join(f"{key}={value}" for key, value in counts.items())


def _rect_is_valid(rect: Any, width: int, height: int) -> bool:
    if not isinstance(rect, list) or len(rect) != 4:
        return False
    if not all(isinstance(value, int) for value in rect):
        return False
    x, y, w, h = rect
    if w <= 0 or h <= 0 or x < 0 or y < 0:
        return False
    return x + w <= width and y + h <= height


def format_summary(summary: dict[str, Any]) -> str:
    lines = [
        f"art_count={summary['art_count']}",
        f"counted_art_entries={summary['counted_art_entries']}",
        f"manifest_size={summary['manifest_width']}x{summary['manifest_height']}",
        f"art_png_size={summary['art_png_width']}x{summary['art_png_height']}",
        f"source_refs={summary['source_refs']}",
        f"manifest_source_refs={summary['manifest_source_refs']}",
        f"stable_by_loader_rule={summary['stable_by_loader_rule']}",
        f"material_effect_refs={summary['material_effect_refs']}",
        f"stable_preview_refs={summary['stable_preview_refs']}",
        f"requires_live_material_refs={summary['requires_live_material_refs']}",
        f"missing_effect_refs={summary['missing_effect_refs']}",
        f"invalid_rect_count={summary['invalid_rect_count']}",
        _format_counts("stable_by_kind", summary["stable_by_kind"]),
        _format_counts("source_refs_by_kind", summary["source_refs_by_kind"]),
        _format_counts("preview_sources", summary["preview_sources"]),
        f"dynamic_bg3_art_count={summary['dynamic_bg3_art_count']}",
        f"dynamic_bg3_source_refs={summary['dynamic_bg3_source_refs']}",
        f"dynamic_bg3_size={summary['dynamic_bg3_manifest_width']}x{summary['dynamic_bg3_manifest_height']}",
        f"total_art_count={summary['total_art_count']}",
        f"total_source_refs={summary['total_source_refs']}",
        f"dialogue_glyph_tile_count={summary['dialogue_glyph_tile_count']}",
        f"dialogue_glyph_size={summary['dialogue_glyph_width']}x{summary['dialogue_glyph_height']}",
        f"dialogue_vwf_glyph_count={summary['dialogue_vwf_glyph_count']}",
        f"dialogue_vwf_glyph_atlas_count={summary['dialogue_vwf_glyph_atlas_count']}",
        f"dialogue_vwf_glyph_atlas_size={summary['dialogue_vwf_glyph_atlas_width']}x{summary['dialogue_vwf_glyph_atlas_height']}",
    ]
    return "\n".join(lines)


def coverage_errors(
    summary: dict[str, Any],
    *,
    manifest_summary: dict[str, Any] | None = None,
) -> list[str]:
    errors = []
    if (
        summary["manifest_width"] != summary["art_png_width"]
        or summary["manifest_height"] != summary["art_png_height"]
    ):
        errors.append(
            "art_tiles.png size does not match art_tiles.json: "
            f"{summary['art_png_width']}x{summary['art_png_height']} != "
            f"{summary['manifest_width']}x{summary['manifest_height']}"
        )
    if summary["art_count"] != summary["counted_art_entries"]:
        errors.append(
            "manifest art_count does not match counted arts: "
            f"{summary['art_count']} != {summary['counted_art_entries']}"
        )
    if summary["invalid_rect_count"] != 0:
        examples = ", ".join(summary["invalid_rects"])
        errors.append(
            "art rects outside art_tiles.png bounds or malformed: "
            f"{summary['invalid_rect_count']} example(s): {examples}"
        )
    if summary["manifest_source_refs"] != summary["source_refs"]:
        errors.append(
            "manifest source_ref_count does not match counted source_refs: "
            f"{summary['manifest_source_refs']} != {summary['source_refs']}"
        )
    if summary["missing_effect_refs"] != 0:
        errors.append(
            "canonical source refs without stable preview/effect coverage: "
            f"{summary['missing_effect_refs']}"
        )
    if (
        summary["dynamic_bg3_manifest_width"] != summary["dynamic_bg3_png_width"]
        or summary["dynamic_bg3_manifest_height"] != summary["dynamic_bg3_png_height"]
    ):
        errors.append(
            "dynamic_bg3_tiles.png size does not match dynamic_bg3_tiles.json: "
            f"{summary['dynamic_bg3_png_width']}x{summary['dynamic_bg3_png_height']} != "
            f"{summary['dynamic_bg3_manifest_width']}x{summary['dynamic_bg3_manifest_height']}"
        )
    if summary["dynamic_bg3_art_count"] != summary["dynamic_bg3_counted_art_entries"]:
        errors.append(
            "dynamic BG3 manifest art_count does not match counted arts: "
            f"{summary['dynamic_bg3_art_count']} != {summary['dynamic_bg3_counted_art_entries']}"
        )
    if summary["dynamic_bg3_manifest_source_refs"] != summary["dynamic_bg3_source_refs"]:
        errors.append(
            "dynamic BG3 manifest source_ref_count does not match counted source_refs: "
            f"{summary['dynamic_bg3_manifest_source_refs']} != {summary['dynamic_bg3_source_refs']}"
        )
    if summary["dynamic_bg3_invalid_rect_count"] != 0:
        examples = ", ".join(summary["dynamic_bg3_invalid_rects"])
        errors.append(
            "dynamic BG3 art rects outside dynamic_bg3_tiles.png bounds or malformed: "
            f"{summary['dynamic_bg3_invalid_rect_count']} example(s): {examples}"
        )
    if summary["dialogue_glyph_invalid_rect_count"] != 0:
        examples = ", ".join(summary["dialogue_glyph_invalid_rects"])
        errors.append(
            "dialogue glyph rects outside dialogue_glyph_tiles.png bounds or malformed: "
            f"{summary['dialogue_glyph_invalid_rect_count']} example(s): {examples}"
        )
    if summary["dialogue_vwf_glyph_atlas_invalid_rect_count"] != 0:
        examples = ", ".join(summary["dialogue_vwf_glyph_atlas_invalid_rects"])
        errors.append(
            "dialogue VWF glyph rects outside dialogue_vwf_glyphs.png bounds or malformed: "
            f"{summary['dialogue_vwf_glyph_atlas_invalid_rect_count']} example(s): {examples}"
        )
    if manifest_summary is not None and manifest_summary != summary:
        errors.append("manifest canonical_art_atlas_summary does not match recomputed summary")
    return errors


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
    parser.add_argument(
        "--require-full-stable",
        action="store_true",
        help="exit nonzero unless every canonical source ref is stable-covered",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="also compare against manifest.json canonical_art_atlas_summary",
    )
    args = parser.parse_args()

    summary = summarize_variant_atlas(args.atlas_dir)
    manifest_summary = load_manifest_summary(args.manifest) if args.manifest else None
    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(format_summary(summary))
    if args.require_full_stable:
        errors = coverage_errors(summary, manifest_summary=manifest_summary)
        if errors:
            for error in errors:
                print(f"error: {error}", file=sys.stderr)
            raise SystemExit(1)


if __name__ == "__main__":
    main()
