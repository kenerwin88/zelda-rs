#!/usr/bin/env python3
"""Tests for JSON tilemap authoring helpers."""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import tilemap_json


class TilemapJsonTests(unittest.TestCase):
    def test_round_trips_byte_grid_tilemap(self) -> None:
        source = bytes([0x28, 0x2C, 0x09, 0x08, 0xA5, 0x00])

        tilemap = tilemap_json.tilemap_from_bytes(
            source,
            asset="kLightOverworldTilemap",
            asset_index=67,
            width=3,
            height=2,
        )

        self.assertEqual(tilemap["format"], "zelda3_byte_tilemap_v1")
        self.assertEqual(tilemap["asset"], "kLightOverworldTilemap")
        self.assertEqual(tilemap["asset_index"], 67)
        self.assertEqual(tilemap["width"], 3)
        self.assertEqual(tilemap["height"], 2)
        self.assertEqual(tilemap["rows"], [[0x28, 0x2C, 0x09], [0x08, 0xA5, 0x00]])
        self.assertEqual(tilemap_json.bytes_from_tilemap(tilemap), source)

    def test_rejects_row_width_mismatch(self) -> None:
        tilemap = {
            "format": "zelda3_byte_tilemap_v1",
            "asset": "kLightOverworldTilemap",
            "asset_index": 67,
            "width": 2,
            "height": 1,
            "rows": [[1, 2, 3]],
        }

        with self.assertRaisesRegex(ValueError, "row 0 has 3 entries, expected 2"):
            tilemap_json.bytes_from_tilemap(tilemap)

    def test_rejects_out_of_range_tile_values(self) -> None:
        tilemap = {
            "format": "zelda3_byte_tilemap_v1",
            "asset": "kLightOverworldTilemap",
            "asset_index": 67,
            "width": 1,
            "height": 1,
            "rows": [[256]],
        }

        with self.assertRaisesRegex(ValueError, "row 0 column 0 is 256, expected 0..255"):
            tilemap_json.bytes_from_tilemap(tilemap)

    def test_writes_each_tilemap_row_on_one_line(self) -> None:
        tilemap = tilemap_json.tilemap_from_bytes(
            bytes([1, 2, 3, 4]),
            asset="kLightOverworldTilemap",
            asset_index=67,
            width=2,
            height=2,
        )

        with TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "tilemap.json"
            tilemap_json.write_tilemap_json(path, tilemap)

            text = path.read_text(encoding="utf8")

        self.assertIn("    [1, 2],\n", text)
        self.assertIn("    [3, 4]\n", text)


if __name__ == "__main__":
    unittest.main()
