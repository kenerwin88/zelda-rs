#!/usr/bin/env python3
"""Tests for RGBA variant atlas generation."""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from rgba_variant_atlas import (
    RgbaVariant,
    VariantKey,
    pack_rgba_variants,
    rgba_tile_from_indices,
    variant_id,
    write_rom_variant_atlas,
)


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


def write_palette_json(path: Path, colors: list[str]) -> None:
    import json

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            {
                "asset": "kPalette_MainSpr",
                "colors": [
                    {"index": index, "rgb888": color, "snes_bgr15": "0x0000"}
                    for index, color in enumerate(colors)
                ],
            }
        )
    )


def solid_rgba(value: int) -> bytes:
    return bytes([value, value, value, 255] * 64)


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

    def test_pack_rgba_variants_deduplicates_identical_pixels(self) -> None:
        key_a = VariantKey("sprite", "kSprGfx", 1, 2, 3, "palette_main_spr", 0)
        key_b = VariantKey("sprite", "kSprGfx", 1, 3, 3, "palette_main_spr", 0)

        width, height, pixels, entries = pack_rgba_variants(
            [RgbaVariant(key_a, solid_rgba(7)), RgbaVariant(key_b, solid_rgba(7))],
            columns=2,
        )

        self.assertEqual((width, height), (16, 8))
        self.assertEqual(len(pixels), 16 * 8 * 4)
        self.assertIsNone(entries[0].duplicate_of)
        self.assertEqual(entries[1].duplicate_of, entries[0].id)
        self.assertEqual(entries[1].rect, entries[0].rect)

    def test_write_rom_variant_atlas_uses_extracted_assets_and_palettes(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        palette_colors = [
            "#000000",
            "#102030",
            "#203040",
            "#304050",
            "#405060",
            "#506070",
            "#607080",
            "#708090",
        ] * 16

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(
                pack_arrays([compressed_literal(bytes(raw_pack))])
            )
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                palette_colors,
            )

            written = write_rom_variant_atlas(asset_dir)

            self.assertIn(asset_dir / "atlas/tile_variants.png", written)
            self.assertIn(asset_dir / "atlas/tile_variants.json", written)
            image = Image.open(asset_dir / "atlas/tile_variants.png")
            manifest = json.loads((asset_dir / "atlas/tile_variants.json").read_text())
            self.assertEqual(image.mode, "RGBA")
            self.assertEqual(image.getpixel((0, 0)), (0x10, 0x20, 0x30, 255))
            self.assertEqual(manifest["format"], "zelda3_rgba_variant_atlas_v1")
            self.assertTrue(
                any(entry["palette"] == "palette_main_spr" for entry in manifest["entries"])
            )


if __name__ == "__main__":
    unittest.main()
