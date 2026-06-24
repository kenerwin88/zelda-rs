#!/usr/bin/env python3
"""Build integration test for readable asset sources."""

from __future__ import annotations

import os
import shutil
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


REPO = Path(__file__).resolve().parents[1]
GENERATED_ASSETS = REPO / "generated/zelda3_assets"


class AssetSourceBuildTests(unittest.TestCase):
    def test_build_packs_light_overworld_tilemap_from_json_without_bin(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir) / "zelda3_assets"
            shutil.copytree(GENERATED_ASSETS, asset_dir)
            (asset_dir / "assets/067-kLightOverworldTilemap.bin").unlink(missing_ok=True)
            source_dir = asset_dir / "assets_src/tilemaps"
            source_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(
                GENERATED_ASSETS / "assets_src/tilemaps/light_overworld_tilemap.json",
                source_dir / "light_overworld_tilemap.json",
            )

            result = subprocess.run(
                ["cargo", "build", "-p", "zelda3-bin"],
                cwd=REPO,
                env={**os.environ, "ZELDA3_ASSETS_DIR": str(asset_dir)},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )

        self.assertEqual(result.returncode, 0, result.stdout)

    def test_generated_assets_store_all_tilemaps_as_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        tilemap_bins = sorted((GENERATED_ASSETS / "assets").glob("*Tilemap*.bin"))
        tilemap_sources = sorted((GENERATED_ASSETS / "assets_src/tilemaps").glob("*.json"))

        self.assertEqual(tilemap_bins, [])
        self.assertEqual(
            [path.name for path in tilemap_sources],
            [
                "bg_tilemap_0.json",
                "bg_tilemap_1.json",
                "bg_tilemap_2.json",
                "bg_tilemap_3.json",
                "bg_tilemap_4.json",
                "bg_tilemap_5.json",
                "dark_overworld_tilemap.json",
                "light_overworld_tilemap.json",
            ],
        )

    def test_generated_assets_store_all_palettes_as_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        palette_bins = sorted((GENERATED_ASSETS / "assets").glob("*Palette*.bin"))
        hud_palette_bin = GENERATED_ASSETS / "assets/092-kHudPalData.bin"
        palette_sources = sorted((GENERATED_ASSETS / "assets_src/palettes").glob("*.json"))

        self.assertEqual(palette_bins, [])
        self.assertFalse(hud_palette_bin.exists())
        self.assertEqual(len(palette_sources), 17)

    def test_generated_assets_store_navigation_tables_as_grouped_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        navigation_bins = []
        for start, end in [(11, 45), (130, 156)]:
            for index in range(start, end + 1):
                navigation_bins.extend((GENERATED_ASSETS / "assets").glob(f"{index:03d}-*.bin"))
        navigation_sources = sorted((GENERATED_ASSETS / "assets_src/navigation").glob("*.json"))

        self.assertEqual(navigation_bins, [])
        self.assertEqual(
            [path.name for path in navigation_sources],
            [
                "dungeon_entrances.json",
                "overworld_exits.json",
                "special_exits.json",
                "starting_points.json",
            ],
        )

    def test_ci_placeholder_assets_still_build(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir) / "ci-assets/zelda3_assets"
            create_result = subprocess.run(
                [
                    "python3",
                    "scripts/create_ci_assets.py",
                    "--out-dir",
                    str(asset_dir),
                ],
                cwd=REPO,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            self.assertEqual(create_result.returncode, 0, create_result.stdout)

            build_result = subprocess.run(
                ["cargo", "check", "-p", "zelda3-bin"],
                cwd=REPO,
                env={**os.environ, "ZELDA3_ASSETS_DIR": str(asset_dir)},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )

        self.assertEqual(build_result.returncode, 0, build_result.stdout)


if __name__ == "__main__":
    unittest.main()
