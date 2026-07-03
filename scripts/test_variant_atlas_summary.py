#!/usr/bin/env python3
"""Tests for compact variant atlas coverage summaries."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from variant_atlas_summary import format_summary, summarize_variant_atlas


def write_json(path: Path, data: object) -> None:
    path.write_text(json.dumps(data))


class VariantAtlasSummaryTests(unittest.TestCase):
    def test_summary_matches_loader_stable_effect_rule(self) -> None:
        with TemporaryDirectory() as temp_dir:
            atlas_dir = Path(temp_dir)
            write_json(
                atlas_dir / "art_tiles.json",
                {
                    "format": "zelda3_canonical_art_atlas_v1",
                    "art_count": 2,
                    "source_ref_count": 3,
                    "arts": [
                        {
                            "art_id": "art:a",
                            "bpp": 3,
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
            self.assertEqual(summary["manifest_source_refs"], 3)
            self.assertEqual(summary["stable_by_loader_rule"], 2)
            self.assertEqual(summary["missing_effect_refs"], 1)
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
                "source_refs": 3,
                "manifest_source_refs": 3,
                "stable_by_loader_rule": 2,
                "missing_effect_refs": 1,
                "stable_by_kind": {"bg": 1, "sprite": 1},
                "source_refs_by_kind": {"bg": 1, "sprite": 2},
                "preview_sources": {"palette_usage": 1, "source_kind_default": 2},
            }
        )

        self.assertIn("source_refs=3", text)
        self.assertIn("stable_by_kind bg=1 sprite=1", text)
        self.assertIn("preview_sources palette_usage=1 source_kind_default=2", text)


if __name__ == "__main__":
    unittest.main()
