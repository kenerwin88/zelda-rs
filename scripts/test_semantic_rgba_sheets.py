#!/usr/bin/env python3
"""Tests for semantic RGBA sheet export and compilation."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image

from semantic_rgba_sheets import compile_semantic_sheets
from semantic_rgba_sheets import SemanticCoverageError
from semantic_rgba_sheets import write_initial_semantic_sheets


def write_test_atlas(asset_dir: Path, entry_count: int = 2) -> None:
    atlas_dir = asset_dir / "atlas"
    atlas_dir.mkdir(parents=True)
    image = Image.new("RGBA", (entry_count * 8, 8))
    for index in range(entry_count):
        color = (10, 20, 30, 255) if index == 0 else (40, 50, 60, 255)
        for y in range(8):
            for x in range(8):
                image.putpixel((index * 8 + x, y), color)
    image.save(atlas_dir / "tile_variants.png")
    entries = []
    for index in range(entry_count):
        entries.append(
            {
                "id": (
                    f"sprite:kSprGfx:pack12:tile{index + 3}:3bpp:"
                    f"palette_main_spr:row{index}"
                ),
                "source_kind": "sprite",
                "asset": "kSprGfx",
                "pack": 12,
                "tile": index + 3,
                "bpp": 3,
                "palette": "palette_main_spr",
                "palette_row": index,
                "rect": [index * 8, 0, 8, 8],
                "sha1": f"{index:03x}",
                "duplicate_of": None,
                "dynamic_policy": "stable",
            }
        )
    (atlas_dir / "tile_variants.json").write_text(
        json.dumps(
            {
                "format": "zelda3_rgba_variant_atlas_v1",
                "tile_width": 8,
                "tile_height": 8,
                "width": entry_count * 8,
                "height": 8,
                "entry_count": entry_count,
                "entries": entries,
            }
        )
        + "\n"
    )


class SemanticRgbaSheetsTests(unittest.TestCase):
    def test_write_initial_semantic_sheets_groups_sprite_variants_with_emits(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_test_atlas(asset_dir)

            written = write_initial_semantic_sheets(asset_dir)

            png_path = asset_dir / "assets_src/semantic/sprites/sprite_kSprGfx_pack12.png"
            json_path = asset_dir / "assets_src/semantic/sprites/sprite_kSprGfx_pack12.json"
            self.assertEqual(written, [png_path, json_path])
            with Image.open(png_path) as image:
                self.assertEqual(image.mode, "RGBA")
                self.assertEqual(image.size, (16, 8))
                self.assertEqual(image.getpixel((0, 0)), (10, 20, 30, 255))
                self.assertEqual(image.getpixel((8, 0)), (40, 50, 60, 255))

            manifest = json.loads(json_path.read_text())
            self.assertEqual(manifest["format"], "zelda3_semantic_rgba_sheet_v1")
            self.assertEqual(manifest["source_kind"], "sprite")
            self.assertEqual(manifest["asset"], "kSprGfx")
            self.assertEqual(manifest["pack"], 12)
            self.assertEqual(
                manifest["frames"],
                [
                    {
                        "id": "sprite_kSprGfx_pack12_tile3_palette_main_spr_row0",
                        "source_rect": [0, 0, 8, 8],
                        "emits": [
                            "sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"
                        ],
                    },
                    {
                        "id": "sprite_kSprGfx_pack12_tile4_palette_main_spr_row1",
                        "source_rect": [8, 0, 8, 8],
                        "emits": [
                            "sprite:kSprGfx:pack12:tile4:3bpp:palette_main_spr:row1"
                        ],
                    },
                ],
            )

    def test_cli_exports_initial_semantic_sheets(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_test_atlas(asset_dir)

            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/semantic_rgba_sheets.py",
                    "--asset-dir",
                    str(asset_dir),
                ],
                cwd=Path(__file__).resolve().parents[1],
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(
                (asset_dir / "assets_src/semantic/sprites/sprite_kSprGfx_pack12.png").is_file()
            )

    def test_write_initial_semantic_sheets_wraps_large_groups(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_test_atlas(asset_dir, entry_count=130)

            write_initial_semantic_sheets(asset_dir)

            png_path = asset_dir / "assets_src/semantic/sprites/sprite_kSprGfx_pack12.png"
            json_path = asset_dir / "assets_src/semantic/sprites/sprite_kSprGfx_pack12.json"
            with Image.open(png_path) as image:
                self.assertEqual(image.size, (1024, 16))
                self.assertEqual(image.getpixel((0, 8)), (40, 50, 60, 255))
            manifest = json.loads(json_path.read_text())
            self.assertEqual(manifest["frames"][128]["source_rect"], [0, 8, 8, 8])

    def test_compile_semantic_sheets_emits_rgba_variant_with_original_key(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_test_atlas(asset_dir, entry_count=1)
            semantic_dir = asset_dir / "assets_src/semantic"
            sheet_dir = semantic_dir / "sprites"
            sheet_dir.mkdir(parents=True)
            image = Image.new("RGBA", (8, 8), (90, 80, 70, 255))
            image.save(sheet_dir / "sprite_kSprGfx_pack12.png")
            (sheet_dir / "sprite_kSprGfx_pack12.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_semantic_rgba_sheet_v1",
                        "source_kind": "sprite",
                        "asset": "kSprGfx",
                        "pack": 12,
                        "tile_width": 8,
                        "tile_height": 8,
                        "image_file": "sprite_kSprGfx_pack12.png",
                        "frames": [
                            {
                                "id": "edited_sprite",
                                "source_rect": [0, 0, 8, 8],
                                "emits": [
                                    "sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"
                                ],
                            }
                        ],
                    }
                )
                + "\n"
            )

            variants = compile_semantic_sheets(asset_dir, semantic_dir)

            self.assertEqual(len(variants), 1)
            self.assertEqual(variants[0].key.source_kind, "sprite")
            self.assertEqual(variants[0].key.asset, "kSprGfx")
            self.assertEqual(variants[0].key.pack, 12)
            self.assertEqual(variants[0].key.tile, 3)
            self.assertEqual(variants[0].key.palette, "palette_main_spr")
            self.assertEqual(variants[0].key.palette_row, 0)
            self.assertEqual(variants[0].pixels[:4], bytes([90, 80, 70, 255]))

    def test_compile_semantic_sheets_reports_missing_duplicate_and_out_of_bounds(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_test_atlas(asset_dir, entry_count=2)
            semantic_dir = asset_dir / "assets_src/semantic"
            sheet_dir = semantic_dir / "sprites"
            sheet_dir.mkdir(parents=True)
            Image.new("RGBA", (8, 8), (90, 80, 70, 255)).save(
                sheet_dir / "sprite_kSprGfx_pack12.png"
            )
            (sheet_dir / "sprite_kSprGfx_pack12.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_semantic_rgba_sheet_v1",
                        "source_kind": "sprite",
                        "asset": "kSprGfx",
                        "pack": 12,
                        "tile_width": 8,
                        "tile_height": 8,
                        "image_file": "sprite_kSprGfx_pack12.png",
                        "frames": [
                            {
                                "id": "first",
                                "source_rect": [0, 0, 8, 8],
                                "emits": [
                                    "sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"
                                ],
                            },
                            {
                                "id": "duplicate",
                                "source_rect": [0, 0, 8, 8],
                                "emits": [
                                    "sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"
                                ],
                            },
                            {
                                "id": "bounds",
                                "source_rect": [7, 7, 8, 8],
                                "emits": [
                                    "sprite:kSprGfx:pack12:tile4:3bpp:palette_main_spr:row1"
                                ],
                            },
                        ],
                    }
                )
                + "\n"
            )

            with self.assertRaises(SemanticCoverageError) as raised:
                compile_semantic_sheets(asset_dir, semantic_dir)

            self.assertEqual(
                raised.exception.missing_variant_ids,
                ["sprite:kSprGfx:pack12:tile4:3bpp:palette_main_spr:row1"],
            )
            self.assertEqual(
                raised.exception.duplicate_variant_ids,
                ["sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"],
            )
            self.assertEqual(
                raised.exception.rect_out_of_bounds,
                ["sprite:kSprGfx:pack12:tile4:3bpp:palette_main_spr:row1"],
            )


if __name__ == "__main__":
    unittest.main()
