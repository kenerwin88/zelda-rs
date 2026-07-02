#!/usr/bin/env python3
"""Tests for readable asset source emission during extraction."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import extract_assets
import navigation_json
import palette_json
import tilemap_json


def pack_arrays(arrays: list[bytes]) -> bytes:
    use_wide_offsets = sum(len(item) for item in arrays[:-1]) >= 65536
    offset_size = 4 if use_wide_offsets else 2
    offsets = []
    offset = 0
    for item in arrays[:-1]:
        offset += len(item)
        offsets.append(offset.to_bytes(offset_size, "little"))
    marker = 8192 + len(arrays) - 1 if use_wide_offsets else len(arrays) - 1
    return b"".join([*offsets, *arrays, marker.to_bytes(2, "little")])


def compressed_literal(payload: bytes) -> bytes:
    out = bytearray()
    offset = 0
    while offset < len(payload):
        chunk = payload[offset : offset + 1024]
        out.append(0xE0 | ((len(chunk) - 1) >> 8))
        out.append((len(chunk) - 1) & 0xFF)
        out.extend(chunk)
        offset += len(chunk)
    out.append(0xFF)
    return bytes(out)


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

    def test_writes_chr_source_sheets_from_extracted_graphics_assets(self) -> None:
        raw_pack = bytes([0] * 1536)
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)] * 17
        assets = [(f"kUnused_{i}", b"x") for i in range(64)]
        assets.append(("kSprGfx", pack_arrays(sprite_items)))
        assets.append(("kBgGfx", pack_arrays([compressed_literal(raw_pack)])))

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            extract_assets.write_asset_outputs(out_dir, assets)

            sheets = extract_assets.write_chr_source_sheets(out_dir)

            self.assertIn(
                {
                    "sheet": "a-h",
                    "image_file": "assets_src/chr/a-h.png",
                    "manifest_file": "assets_src/chr/a-h.json",
                    "source_format": "zelda3_editable_chr_sheet_v1",
                },
                sheets,
            )
            self.assertTrue((out_dir / "assets_src/chr/a-h.png").is_file())

    def test_writes_rgba_variant_atlas_from_extracted_graphics_assets(self) -> None:
        raw_pack = bytes([0] * 1536)
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)]
        assets = [(f"kUnused_{i}", b"x") for i in range(64)]
        assets.append(("kSprGfx", pack_arrays(sprite_items)))
        assets.append(("kBgGfx", pack_arrays([compressed_literal(raw_pack)])))

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            extract_assets.write_asset_outputs(out_dir, assets)
            palette_path = out_dir / "assets_src/palettes/palette_main_spr.json"
            palette_path.parent.mkdir(parents=True, exist_ok=True)
            palette_path.write_text(
                json.dumps(
                    {
                        "asset": "kPalette_MainSpr",
                        "colors": [
                            {
                                "index": index,
                                "rgb888": "#000000",
                                "snes_bgr15": "0x0000",
                            }
                            for index in range(64)
                        ],
                    }
                )
            )

            atlas = extract_assets.write_rgba_variant_atlas(out_dir)

            self.assertEqual(
                atlas,
                [
                    {
                        "image_file": "atlas/tile_variants.png",
                        "manifest_file": "atlas/tile_variants.json",
                        "source_format": "zelda3_rgba_variant_atlas_v1",
                    }
                ],
            )
            self.assertTrue((out_dir / "atlas/tile_variants.png").is_file())
            self.assertTrue((out_dir / "atlas/tile_variants.json").is_file())


if __name__ == "__main__":
    unittest.main()
