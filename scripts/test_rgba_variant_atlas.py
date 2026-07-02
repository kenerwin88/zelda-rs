#!/usr/bin/env python3
"""Tests for RGBA variant atlas generation."""

from __future__ import annotations

import unittest

from rgba_variant_atlas import VariantKey, rgba_tile_from_indices, variant_id


class RgbaVariantAtlasTests(unittest.TestCase):
    def test_rgba_tile_from_indices_applies_palette_row(self) -> None:
        indices = bytes([0, 1, 2, 3] + [0] * 60)
        colors = [[index, index + 1, index + 2] for index in range(32)]

        rgba = rgba_tile_from_indices(indices, colors, palette_row=1, colors_per_row=16)

        self.assertEqual(
            list(rgba[:16]),
            [
                16,
                17,
                18,
                0,
                17,
                18,
                19,
                255,
                18,
                19,
                20,
                255,
                19,
                20,
                21,
                255,
            ],
        )

    def test_variant_id_is_stable_and_readable(self) -> None:
        key = VariantKey("sprite", "kSprGfx", 12, 37, 3, "palette_main_spr", 3)

        self.assertEqual(
            variant_id(key),
            "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3",
        )


if __name__ == "__main__":
    unittest.main()
