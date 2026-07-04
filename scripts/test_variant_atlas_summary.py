#!/usr/bin/env python3
"""Tests for compact variant atlas coverage summaries."""

from __future__ import annotations

import json
import struct
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from variant_atlas_summary import (
    coverage_errors,
    format_summary,
    load_manifest_summary,
    summarize_variant_atlas,
)


def write_json(path: Path, data: object) -> None:
    path.write_text(json.dumps(data))


def write_png_header(path: Path, width: int, height: int) -> None:
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + struct.pack(">I", 13)
        + b"IHDR"
        + struct.pack(">II", width, height)
        + b"\x08\x06\x00\x00\x00"
    )


class VariantAtlasSummaryTests(unittest.TestCase):
    def test_summary_matches_loader_stable_effect_rule(self) -> None:
        with TemporaryDirectory() as temp_dir:
            atlas_dir = Path(temp_dir)
            write_json(
                atlas_dir / "art_tiles.json",
                {
                    "format": "zelda3_canonical_art_atlas_v1",
                    "width": 16,
                    "height": 8,
                    "art_count": 2,
                    "source_ref_count": 3,
                    "arts": [
                        {
                            "art_id": "art:a",
                            "bpp": 3,
                            "rect": [0, 0, 8, 8],
                            "preview_palette": "palette_main_spr",
                            "preview_palette_row": 0,
                            "preview_source": "source_kind_default",
                            "source_refs": [
                                {
                                    "source_kind": "sprite",
                                    "bpp": 3,
                                    "preview_palette": "palette_main_spr",
                                    "preview_palette_row": 0,
                                    "preview_source": "source_kind_default",
                                },
                                {
                                    "source_kind": "sprite",
                                    "bpp": 3,
                                    "preview_palette": "palette_live_flash",
                                    "preview_palette_row": 0,
                                    "preview_source": "source_kind_default",
                                },
                            ],
                        },
                        {
                            "art_id": "art:b",
                            "bpp": 2,
                            "rect": [8, 0, 8, 8],
                            "preview_palette": "palette_bg",
                            "preview_palette_row": 1,
                            "preview_source": "palette_usage",
                            "source_refs": [
                                {
                                    "source_kind": "bg",
                                    "bpp": 2,
                                    "preview_palette": "palette_bg",
                                    "preview_palette_row": 1,
                                    "preview_source": "palette_usage",
                                }
                            ],
                        },
                    ],
                },
            )
            write_png_header(atlas_dir / "art_tiles.png", 16, 8)
            write_json(
                atlas_dir / "tile_effects.json",
                {
                    "format": "zelda3_tile_effect_table_v1",
                    "effects": [
                        {
                            "palette": "palette_main_spr",
                            "palette_row": 0,
                            "colors_per_row": 8,
                            "dynamic_policy": "stable",
                        },
                        {
                            "palette": "palette_live_flash",
                            "palette_row": 0,
                            "colors_per_row": 8,
                            "dynamic_policy": "requires_live_palette",
                        },
                    ],
                },
            )

            summary = summarize_variant_atlas(atlas_dir)

            self.assertEqual(summary["source_refs"], 3)
            self.assertEqual(summary["counted_art_entries"], 2)
            self.assertEqual(summary["manifest_width"], 16)
            self.assertEqual(summary["manifest_height"], 8)
            self.assertEqual(summary["art_png_width"], 16)
            self.assertEqual(summary["art_png_height"], 8)
            self.assertEqual(summary["manifest_source_refs"], 3)
            self.assertEqual(summary["stable_by_loader_rule"], 2)
            self.assertEqual(summary["material_effect_refs"], 1)
            self.assertEqual(summary["stable_preview_refs"], 1)
            self.assertEqual(summary["requires_live_material_refs"], 1)
            self.assertEqual(summary["missing_effect_refs"], 1)
            self.assertEqual(summary["invalid_rect_count"], 0)
            self.assertEqual(summary["invalid_rects"], [])
            self.assertEqual(summary["stable_by_kind"], {"bg": 1, "sprite": 1})
            self.assertEqual(summary["source_refs_by_kind"], {"bg": 1, "sprite": 2})
            self.assertEqual(
                summary["preview_sources"],
                {"palette_usage": 1, "source_kind_default": 2},
            )

    def test_format_summary_is_stable_for_cli_use(self) -> None:
        text = format_summary(
            {
                "art_count": 2,
                "counted_art_entries": 2,
                "manifest_width": 16,
                "manifest_height": 8,
                "art_png_width": 16,
                "art_png_height": 8,
                "source_refs": 3,
                "manifest_source_refs": 3,
                "stable_by_loader_rule": 2,
                "material_effect_refs": 1,
                "stable_preview_refs": 1,
                "requires_live_material_refs": 1,
                "missing_effect_refs": 1,
                "invalid_rect_count": 0,
                "stable_by_kind": {"bg": 1, "sprite": 1},
                "source_refs_by_kind": {"bg": 1, "sprite": 2},
                "preview_sources": {"palette_usage": 1, "source_kind_default": 2},
            }
        )

        self.assertIn("counted_art_entries=2", text)
        self.assertIn("manifest_size=16x8", text)
        self.assertIn("art_png_size=16x8", text)
        self.assertIn("invalid_rect_count=0", text)
        self.assertIn("source_refs=3", text)
        self.assertIn("material_effect_refs=1", text)
        self.assertIn("stable_preview_refs=1", text)
        self.assertIn("requires_live_material_refs=1", text)
        self.assertIn("stable_by_kind bg=1 sprite=1", text)
        self.assertIn("preview_sources palette_usage=1 source_kind_default=2", text)

    def test_summary_uses_runtime_colors_per_row_metadata(self) -> None:
        with TemporaryDirectory() as temp_dir:
            atlas_dir = Path(temp_dir)
            write_json(
                atlas_dir / "art_tiles.json",
                {
                    "format": "zelda3_canonical_art_atlas_v1",
                    "width": 8,
                    "height": 8,
                    "art_count": 1,
                    "source_ref_count": 1,
                    "arts": [
                        {
                            "art_id": "art:bg",
                            "bpp": 3,
                            "rect": [0, 0, 8, 8],
                            "preview_palette": "palette_dung_bg_main",
                            "preview_palette_row": 2,
                            "preview_source": "source_kind_default",
                            "source_refs": [
                                {
                                    "source_kind": "bg",
                                    "bpp": 3,
                                    "preview_palette": "palette_dung_bg_main",
                                    "preview_palette_row": 2,
                                    "preview_source": "source_kind_default",
                                    "runtime_material": "palette_lut",
                                    "runtime_material_policy": "stable",
                                    "runtime_colors_per_row": 16,
                                }
                            ],
                        }
                    ],
                },
            )
            write_png_header(atlas_dir / "art_tiles.png", 8, 8)
            write_json(
                atlas_dir / "tile_effects.json",
                {
                    "format": "zelda3_tile_effect_table_v1",
                    "effects": [
                        {
                            "palette": "palette_dung_bg_main",
                            "palette_row": 2,
                            "colors_per_row": 16,
                            "dynamic_policy": "stable",
                        }
                    ],
                },
            )

            summary = summarize_variant_atlas(atlas_dir)

            self.assertEqual(summary["stable_by_loader_rule"], 1)
            self.assertEqual(summary["material_effect_refs"], 1)
            self.assertEqual(summary["missing_effect_refs"], 0)

    def test_summary_honors_runtime_material_policy_override(self) -> None:
        with TemporaryDirectory() as temp_dir:
            atlas_dir = Path(temp_dir)
            write_json(
                atlas_dir / "art_tiles.json",
                {
                    "format": "zelda3_canonical_art_atlas_v1",
                    "width": 8,
                    "height": 8,
                    "art_count": 1,
                    "source_ref_count": 1,
                    "arts": [
                        {
                            "art_id": "art:live",
                            "bpp": 3,
                            "rect": [0, 0, 8, 8],
                            "preview_palette": "palette_live_flash",
                            "preview_palette_row": 0,
                            "preview_source": "source_kind_default",
                            "source_refs": [
                                {
                                    "source_kind": "sprite",
                                    "bpp": 3,
                                    "preview_palette": "palette_live_flash",
                                    "preview_palette_row": 0,
                                    "preview_source": "source_kind_default",
                                    "runtime_material": "palette_lut",
                                    "runtime_material_policy": "requires_live_palette",
                                    "runtime_colors_per_row": 8,
                                }
                            ],
                        }
                    ],
                },
            )
            write_png_header(atlas_dir / "art_tiles.png", 8, 8)
            write_json(
                atlas_dir / "tile_effects.json",
                {
                    "format": "zelda3_tile_effect_table_v1",
                    "effects": [
                        {
                            "palette": "palette_live_flash",
                            "palette_row": 0,
                            "colors_per_row": 8,
                            "dynamic_policy": "stable",
                        }
                    ],
                },
            )

            summary = summarize_variant_atlas(atlas_dir)

            self.assertEqual(summary["stable_by_loader_rule"], 0)
            self.assertEqual(summary["material_effect_refs"], 0)
            self.assertEqual(summary["requires_live_material_refs"], 1)
            self.assertEqual(summary["missing_effect_refs"], 1)

    def test_coverage_errors_require_manifest_match_and_full_stable_coverage(self) -> None:
        valid_summary = {
            "source_refs": 3,
            "art_count": 2,
            "counted_art_entries": 2,
            "manifest_width": 16,
            "manifest_height": 8,
            "art_png_width": 16,
            "art_png_height": 8,
            "manifest_source_refs": 3,
            "missing_effect_refs": 0,
            "invalid_rect_count": 0,
            "invalid_rects": [],
        }
        self.assertEqual(
            coverage_errors(valid_summary, manifest_summary=valid_summary.copy()),
            [],
        )
        self.assertEqual(
            coverage_errors(
                {
                    "source_refs": 3,
                    "art_count": 2,
                    "counted_art_entries": 1,
                    "manifest_width": 16,
                    "manifest_height": 8,
                    "art_png_width": 24,
                    "art_png_height": 8,
                    "manifest_source_refs": 4,
                    "missing_effect_refs": 1,
                    "invalid_rect_count": 2,
                    "invalid_rects": ["art:a:[12, 0, 8, 8]", "art:b:['bad']"],
                }
            ),
            [
                "art_tiles.png size does not match art_tiles.json: 24x8 != 16x8",
                "manifest art_count does not match counted arts: 2 != 1",
                "art rects outside art_tiles.png bounds or malformed: 2 example(s): art:a:[12, 0, 8, 8], art:b:['bad']",
                "manifest source_ref_count does not match counted source_refs: 4 != 3",
                "canonical source refs without stable preview/effect coverage: 1",
            ],
        )
        manifest_summary = valid_summary.copy()
        manifest_summary["source_refs"] = 4
        self.assertEqual(
            coverage_errors(valid_summary, manifest_summary=manifest_summary),
            ["manifest canonical_art_atlas_summary does not match recomputed summary"],
        )

    def test_load_manifest_summary_reads_extraction_summary(self) -> None:
        with TemporaryDirectory() as temp_dir:
            manifest_path = Path(temp_dir) / "manifest.json"
            manifest_path.write_text(
                json.dumps(
                    {
                        "canonical_art_atlas_summary": {
                            "art_count": 2,
                            "source_refs": 3,
                        }
                    }
                )
            )

            self.assertEqual(
                load_manifest_summary(manifest_path),
                {"art_count": 2, "source_refs": 3},
            )


if __name__ == "__main__":
    unittest.main()
