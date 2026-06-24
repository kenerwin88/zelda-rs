#!/usr/bin/env python3
"""Tests for readable asset source emission during extraction."""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import extract_assets
import navigation_json
import palette_json
import tilemap_json


class ExtractAssetSourcesTests(unittest.TestCase):
    def test_writes_light_overworld_tilemap_as_json_source(self) -> None:
        payload = bytes(range(64)) * 64

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=67,
                name="kLightOverworldTilemap",
                payload=payload,
            )

            source_path = out_dir / "assets_src/tilemaps/light_overworld_tilemap.json"

            self.assertEqual(manifest["source_format"], "zelda3_byte_tilemap_v1")
            self.assertEqual(
                manifest["source_file"],
                "assets_src/tilemaps/light_overworld_tilemap.json",
            )
            self.assertFalse((out_dir / "assets/067-kLightOverworldTilemap.bin").exists())
            self.assertEqual(
                tilemap_json.bytes_from_tilemap(tilemap_json.read_tilemap_json(source_path)),
                payload,
            )

    def test_writes_dark_overworld_tilemap_as_json_source(self) -> None:
        payload = bytes(range(64)) * 16

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=68,
                name="kDarkOverworldTilemap",
                payload=payload,
            )

            source_path = out_dir / "assets_src/tilemaps/dark_overworld_tilemap.json"

            self.assertEqual(manifest["source_format"], "zelda3_byte_tilemap_v1")
            self.assertFalse((out_dir / "assets/068-kDarkOverworldTilemap.bin").exists())
            self.assertEqual(
                tilemap_json.bytes_from_tilemap(tilemap_json.read_tilemap_json(source_path)),
                payload,
            )

    def test_writes_bg_tilemaps_as_json_sources(self) -> None:
        payload = bytes([0x2A, 0x00, 0xFF, 0x80])

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=99,
                name="kBgTilemap_0",
                payload=payload,
            )

            source_path = out_dir / "assets_src/tilemaps/bg_tilemap_0.json"

            self.assertEqual(manifest["source_format"], "zelda3_byte_stream_tilemap_v1")
            self.assertFalse((out_dir / "assets/099-kBgTilemap_0.bin").exists())
            self.assertEqual(
                tilemap_json.bytes_from_tilemap(tilemap_json.read_tilemap_json(source_path)),
                payload,
            )

    def test_writes_palettes_as_json_sources(self) -> None:
        payload = bytes([0x00, 0x00, 0xFF, 0x7F])

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=80,
                name="kPalette_MainSpr",
                payload=payload,
            )

            source_path = out_dir / "assets_src/palettes/palette_main_spr.json"

            self.assertEqual(manifest["source_format"], "zelda3_snes_palette_v1")
            self.assertFalse((out_dir / "assets/080-kPalette_MainSpr.bin").exists())
            self.assertEqual(
                palette_json.bytes_from_palette(palette_json.read_palette_json(source_path)),
                payload,
            )

    def test_writes_hud_palette_data_as_json_source(self) -> None:
        payload = bytes([0x1F, 0x00, 0xE0, 0x03])

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=92,
                name="kHudPalData",
                payload=payload,
            )

            source_path = out_dir / "assets_src/palettes/hud_pal_data.json"

            self.assertEqual(manifest["source_format"], "zelda3_snes_palette_v1")
            self.assertFalse((out_dir / "assets/092-kHudPalData.bin").exists())
            self.assertEqual(
                palette_json.bytes_from_palette(palette_json.read_palette_json(source_path)),
                payload,
            )

    def test_writes_navigation_tables_as_grouped_json_sources(self) -> None:
        assets = [
            ("kEntranceData_rooms", (0x1234).to_bytes(2, "little")),
            ("kEntranceData_relativeCoords", bytes([1, 2, 3, 4, 5, 6, 7, 8])),
            ("kEntranceData_scrollX", (0).to_bytes(2, "little")),
            ("kEntranceData_scrollY", (1).to_bytes(2, "little")),
            ("kEntranceData_playerX", (2).to_bytes(2, "little")),
            ("kEntranceData_playerY", (3).to_bytes(2, "little")),
            ("kEntranceData_cameraX", (4).to_bytes(2, "little")),
            ("kEntranceData_cameraY", (5).to_bytes(2, "little")),
            ("kEntranceData_blockset", bytes([6])),
            ("kEntranceData_floor", (-1).to_bytes(1, "little", signed=True)),
            ("kEntranceData_palace", (-2).to_bytes(1, "little", signed=True)),
            ("kEntranceData_doorwayOrientation", bytes([7])),
            ("kEntranceData_startingBg", bytes([8])),
            ("kEntranceData_quadrant1", bytes([9])),
            ("kEntranceData_quadrant2", bytes([10])),
            ("kEntranceData_doorSettings", (0x8000).to_bytes(2, "little")),
            ("kEntranceData_musicTrack", bytes([11])),
        ]

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_outputs(out_dir, assets)

            source_path = out_dir / "assets_src/navigation/dungeon_entrances.json"

            self.assertTrue(source_path.is_file())
            self.assertFalse((out_dir / "assets/000-kEntranceData_rooms.bin").exists())
            self.assertEqual(
                manifest[0]["source_format"], navigation_json.FORMAT_DUNGEON_ENTRANCES
            )
            self.assertEqual(
                manifest[0]["source_file"],
                "assets_src/navigation/dungeon_entrances.json",
            )
            source = navigation_json.read_navigation_json(source_path)
            self.assertEqual(
                navigation_json.bytes_for_asset(source, "kEntranceData_floor"),
                (-1).to_bytes(1, "little", signed=True),
            )

    def test_keeps_unmigrated_assets_as_bin_files(self) -> None:
        payload = b"abc"

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=69,
                name="kPredefinedTileData",
                payload=payload,
            )

            self.assertNotIn("source_format", manifest)
            self.assertEqual(manifest["file"], "assets/069-kPredefinedTileData.bin")
            self.assertEqual(
                (out_dir / "assets/069-kPredefinedTileData.bin").read_bytes(),
                payload,
            )


if __name__ == "__main__":
    unittest.main()
