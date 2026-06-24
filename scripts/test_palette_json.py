#!/usr/bin/env python3
"""Tests for SNES palette JSON authoring helpers."""

from __future__ import annotations

import unittest

import palette_json


class PaletteJsonTests(unittest.TestCase):
    def test_round_trips_snes_palette_words(self) -> None:
        source = bytes([0x00, 0x00, 0xFF, 0x7F, 0x1F, 0x00, 0xE0, 0x03, 0x00, 0x7C])

        palette = palette_json.palette_from_bytes(
            source,
            asset="kPalette_Test",
            asset_index=1,
        )

        self.assertEqual(palette["format"], "zelda3_snes_palette_v1")
        self.assertEqual(palette["asset"], "kPalette_Test")
        self.assertEqual(palette["asset_index"], 1)
        self.assertEqual(
            palette["colors"],
            [
                {"index": 0, "snes_bgr15": "0x0000", "rgb888": "#000000"},
                {"index": 1, "snes_bgr15": "0x7fff", "rgb888": "#ffffff"},
                {"index": 2, "snes_bgr15": "0x001f", "rgb888": "#ff0000"},
                {"index": 3, "snes_bgr15": "0x03e0", "rgb888": "#00ff00"},
                {"index": 4, "snes_bgr15": "0x7c00", "rgb888": "#0000ff"},
            ],
        )
        self.assertEqual(palette_json.bytes_from_palette(palette), source)

    def test_rejects_odd_length_palette_bytes(self) -> None:
        with self.assertRaisesRegex(ValueError, "has 3 bytes, expected an even byte count"):
            palette_json.palette_from_bytes(
                b"\x00\x01\x02",
                asset="kPalette_Test",
                asset_index=1,
            )

    def test_rejects_out_of_range_snes_color_words(self) -> None:
        palette = {
            "format": "zelda3_snes_palette_v1",
            "asset": "kPalette_Test",
            "asset_index": 1,
            "colors": [
                {"index": 0, "snes_bgr15": "0x8000", "rgb888": "#000000"},
            ],
        }

        with self.assertRaisesRegex(ValueError, "color 0 is 0x8000, expected 0x0000..0x7fff"):
            palette_json.bytes_from_palette(palette)


if __name__ == "__main__":
    unittest.main()
