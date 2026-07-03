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


if __name__ == "__main__":
    unittest.main()
