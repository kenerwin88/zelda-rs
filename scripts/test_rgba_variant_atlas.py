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
    build_base_effect_atlas,
    build_canonical_art_atlas,
    classify_palette_policy,
    rgba_tile_from_indices,
    variant_id,
    write_base_effect_atlas,
    write_canonical_art_atlas,
    write_rom_variant_atlas,
    write_tile_effect_table,
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
            (assets_dir / "095-kDialogueFont.bin").write_bytes(bytes(0x1000))
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

    def test_build_base_effect_atlas_uses_one_human_default_per_tile(self) -> None:
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
            (assets_dir / "095-kDialogueFont.bin").write_bytes(bytes(0x1000))
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                palette_colors,
            )

            width, height, pixels, entries, effects = build_base_effect_atlas(asset_dir)

            self.assertEqual((width, height), (256, 8))
            self.assertLess(len(entries), 2000)
            self.assertEqual(entries[0]["preview_palette"], "palette_main_spr")
            self.assertEqual(entries[0]["preview_palette_row"], 0)
            self.assertEqual(list(pixels[:4]), [0x10, 0x20, 0x30, 255])
            self.assertTrue(
                any(effect["id"] == "palette_main_spr:8color:row1" for effect in effects["effects"])
            )

    def test_base_effect_atlas_uses_palette_usage_map_for_preview_colors(self) -> None:
        import json

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
            "#000000",
            "#A0B0C0",
            "#A1B1C1",
            "#A2B2C2",
            "#A3B3C3",
            "#A4B4C4",
            "#A5B5C5",
            "#A6B6C6",
        ] * 8

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
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "sprite",
                                "asset": "kSprGfx",
                                "pack": 0,
                                "tile": 0,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 1,
                                "evidence_count": 7,
                            }
                        ],
                    }
                )
            )

            _width, _height, pixels, entries, _effects = build_base_effect_atlas(asset_dir)

            self.assertEqual(entries[0]["preview_palette"], "palette_main_spr")
            self.assertEqual(entries[0]["preview_palette_row"], 1)
            self.assertEqual(entries[0]["preview_source"], "palette_usage")
            self.assertEqual(entries[0]["palette_usage_evidence_count"], 7)
            self.assertEqual(list(pixels[:4]), [0xA0, 0xB0, 0xC0, 255])

    def test_base_effect_atlas_marks_source_default_stable_for_known_palette(self) -> None:
        raw_pack = bytearray(1536)
        raw_pack[16] = 0x80
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

            _width, _height, _pixels, entries, _effects = build_base_effect_atlas(asset_dir)

            self.assertEqual(entries[0]["preview_palette"], "palette_main_spr")
            self.assertEqual(entries[0]["preview_source"], "source_kind_default")
            self.assertEqual(entries[0]["dynamic_policy"], "stable")

    def test_base_effect_atlas_ingests_source_key_tiles_with_usage_palette(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
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
            "#000000",
            "#A0B0C0",
            "#A1B1C1",
            "#A2B2C2",
            "#A3B3C3",
            "#A4B4C4",
            "#A5B5C5",
            "#A6B6C6",
        ] * 8

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
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "bg",
                                "asset": "kBgGfx",
                                "pack": 32800,
                                "tile": 7,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 1,
                                "evidence_count": 11,
                            }
                        ],
                    }
                )
            )
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 1,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            }
                        ],
                    }
                )
            )
            image = Image.new("P", (8, 8))
            image.putdata([1] + [0] * 63)
            image.save(source_dir / "assets_by_source.png")

            width, _height, pixels, entries, _effects = build_base_effect_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            entry = next(
                entry
                for entry in entries
                if entry["id"] == "bg:kBgGfx:pack32800:tile7:3bpp"
            )
            self.assertEqual(entry["preview_palette"], "palette_main_spr")
            self.assertEqual(entry["preview_palette_row"], 1)
            self.assertEqual(entry["preview_source"], "palette_usage")
            self.assertEqual(entry["palette_usage_evidence_count"], 11)
            x, y, _w, _h = entry["rect"]
            pixel_offset = (y * width + x) * 4
            self.assertEqual(list(pixels[pixel_offset : pixel_offset + 4]), [0xA0, 0xB0, 0xC0, 255])

    def test_base_effect_atlas_keeps_one_canonical_source_tile_preview(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
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
            "#000000",
            "#A0B0C0",
            "#A1B1C1",
            "#A2B2C2",
            "#A3B3C3",
            "#A4B4C4",
            "#A5B5C5",
            "#A6B6C6",
        ] * 8

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
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "bg",
                                "asset": "kBgGfx",
                                "pack": 32800,
                                "tile": 7,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 1,
                                "evidence_count": 11,
                            }
                        ],
                    }
                )
            )
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 1,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            }
                        ],
                    }
                )
            )
            image = Image.new("P", (8, 8))
            image.putdata([1] + [0] * 63)
            image.save(source_dir / "assets_by_source.png")

            _width, _height, _pixels, entries, _effects = build_base_effect_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            source_entries = [
                entry
                for entry in entries
                if entry["source_kind"] == "bg"
                and entry["asset"] == "kBgGfx"
                and entry["pack"] == 32800
                and entry["tile"] == 7
                and entry["bpp"] == 3
            ]
            self.assertEqual(len(source_entries), 1)
            self.assertEqual(source_entries[0]["preview_source"], "palette_usage")

    def test_base_effect_atlas_loads_auxiliary_palette_from_usage_map(self) -> None:
        import json

        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        fallback_colors = [
            "#000000",
            "#102030",
            "#203040",
            "#304050",
            "#405060",
            "#506070",
            "#607080",
            "#708090",
        ] * 16
        aux_colors = [
            "#000000",
            "#010203",
            "#020304",
            "#030405",
            "#040506",
            "#050607",
            "#060708",
            "#070809",
            "#000000",
            "#D0E0F0",
            "#D1E1F1",
            "#D2E2F2",
            "#D3E3F3",
            "#D4E4F4",
            "#D5E5F5",
            "#D6E6F6",
        ] * 8

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
                fallback_colors,
            )
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_sprite_aux1.json",
                aux_colors,
            )
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "sprite",
                                "asset": "kSprGfx",
                                "pack": 0,
                                "tile": 0,
                                "bpp": 3,
                                "preview_palette": "palette_sprite_aux1",
                                "preview_palette_row": 1,
                            }
                        ],
                    }
                )
            )

            _width, _height, pixels, entries, effects = build_base_effect_atlas(asset_dir)

            self.assertEqual(entries[0]["preview_palette"], "palette_sprite_aux1")
            self.assertEqual(entries[0]["preview_palette_row"], 1)
            self.assertEqual(list(pixels[:4]), [0xD0, 0xE0, 0xF0, 255])
            self.assertTrue(
                any(
                    effect["id"] == "palette_sprite_aux1:8color:row1"
                    for effect in effects["effects"]
                )
            )

    def test_canonical_art_atlas_dedupes_raw_tiles_and_keeps_source_refs(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
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
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 2,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            },
                            {
                                "id": 1,
                                "key": (6 << 32) | (32801 << 16) | 9,
                                "kind": 6,
                                "pack": 32801,
                                "tile_off": 9,
                            },
                        ],
                    }
                )
            )
            image = Image.new("P", (16, 8))
            image.putdata(([1] + [0] * 7 + [1] + [0] * 7) * 8)
            image.save(source_dir / "assets_by_source.png")

            _width, _height, _pixels, arts = build_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            matching = [
                (art["art_id"], source)
                for art in arts
                for source in art["source_refs"]
                if source["pack"] in {32800, 32801}
            ]
            self.assertEqual(len(matching), 2)
            self.assertEqual({art_id for art_id, _source in matching}, {matching[0][0]})

    def test_canonical_art_atlas_collapses_flipped_tiles_into_source_ref_transform(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
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
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 2,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            },
                            {
                                "id": 1,
                                "key": (6 << 32) | (32801 << 16) | 9,
                                "kind": 6,
                                "pack": 32801,
                                "tile_off": 9,
                            },
                        ],
                    }
                )
            )
            row = [1, 2, 3, 4, 5, 6, 7, 0]
            flipped_row = list(reversed(row))
            image = Image.new("P", (16, 8))
            image.putdata((row + flipped_row) * 8)
            image.save(source_dir / "assets_by_source.png")

            _width, _height, _pixels, arts = build_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            matching = [
                (art["art_id"], source)
                for art in arts
                for source in art["source_refs"]
                if source["pack"] in {32800, 32801}
            ]
            self.assertEqual(len(matching), 2)
            self.assertEqual({art_id for art_id, _source in matching}, {matching[0][0]})
            transforms = {
                (source["pack"], source["hflip"], source["vflip"])
                for _art_id, source in matching
            }
            self.assertEqual(transforms, {(32800, False, False), (32801, True, False)})

    def test_canonical_art_atlas_prefers_real_palette_usage_for_preview(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
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
            "#000000",
            "#A0B0C0",
            "#A1B1C1",
            "#A2B2C2",
            "#A3B3C3",
            "#A4B4C4",
            "#A5B5C5",
            "#A6B6C6",
        ] * 8

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
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "bg",
                                "asset": "kBgGfx",
                                "pack": 32800,
                                "tile": 7,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 1,
                                "evidence_count": 11,
                            }
                        ],
                    }
                )
            )
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 1,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            }
                        ],
                    }
                )
            )
            image = Image.new("P", (8, 8))
            image.putdata([1] + [0] * 63)
            image.save(source_dir / "assets_by_source.png")

            width, _height, pixels, arts = build_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            art = next(
                art
                for art in arts
                if any(source["pack"] == 32800 and source["tile"] == 7 for source in art["source_refs"])
            )
            self.assertEqual(art["preview_source"], "palette_usage")
            self.assertEqual(art["preview_palette_row"], 1)
            x, y, _w, _h = art["rect"]
            pixel_offset = (y * width + x) * 4
            self.assertEqual(list(pixels[pixel_offset : pixel_offset + 4]), [0xA0, 0xB0, 0xC0, 255])

    def test_canonical_art_atlas_tracks_palette_rows_as_material_refs(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        palette_colors = [
            "#000000",
            "#101010",
            "#202020",
            "#303030",
            "#404040",
            "#505050",
            "#606060",
            "#707070",
            "#000000",
            "#A0A0A0",
            "#A1A1A1",
            "#A2A2A2",
            "#A3A3A3",
            "#A4A4A4",
            "#A5A5A5",
            "#A6A6A6",
            "#000000",
            "#B0B0B0",
            "#B1B1B1",
            "#B2B2B2",
            "#B3B3B3",
            "#B4B4B4",
            "#B5B5B5",
            "#B6B6B6",
        ] * 6

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
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True, exist_ok=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "bg",
                                "asset": "kBgGfx",
                                "pack": 32800,
                                "tile": 7,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 1,
                                "evidence_count": 3,
                            },
                            {
                                "source_kind": "bg",
                                "asset": "kBgGfx",
                                "pack": 32801,
                                "tile": 9,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 2,
                                "evidence_count": 2,
                            },
                        ],
                    }
                )
            )
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 2,
                        "cells": [
                            {
                                "id": 0,
                                "key": (6 << 32) | (32800 << 16) | 7,
                                "kind": 6,
                                "pack": 32800,
                                "tile_off": 7,
                            },
                            {
                                "id": 1,
                                "key": (6 << 32) | (32801 << 16) | 9,
                                "kind": 6,
                                "pack": 32801,
                                "tile_off": 9,
                            },
                        ],
                    }
                )
            )
            image = Image.new("P", (16, 8))
            image.putdata(([1] + [0] * 7 + [1] + [0] * 7) * 8)
            image.save(source_dir / "assets_by_source.png")

            _width, _height, _pixels, arts = build_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            matching = [
                (art["art_id"], source)
                for art in arts
                for source in art["source_refs"]
                if source["pack"] in {32800, 32801}
            ]
            self.assertEqual(len(matching), 2)
            self.assertEqual({art_id for art_id, _source in matching}, {matching[0][0]})
            self.assertEqual(
                {
                    (
                        source["preview_palette_row"],
                        source["runtime_material"],
                        source["runtime_material_policy"],
                        source["runtime_colors_per_row"],
                    )
                    for _art_id, source in matching
                },
                {
                    (1, "palette_lut", "stable", 8),
                    (2, "palette_lut", "stable", 8),
                },
            )

    def test_canonical_art_atlas_splits_dynamic_bg3_source_tiles(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        palette_colors = [f"#{index:02x}{index:02x}{index:02x}" for index in range(256)]

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(
                pack_arrays([compressed_literal(bytes(raw_pack))])
            )
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_overworld_bg_main.json",
                palette_colors,
            )
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 2,
                        "cells": [
                            {
                                "id": 0,
                                "key": (4 << 32) | (0x0C07 << 16),
                                "kind": 4,
                                "pack": 0x0C07,
                                "tile_off": 0,
                            },
                            {
                                "id": 1,
                                "key": (7 << 32) | (0x1234 << 16) | 0x5678,
                                "kind": 7,
                                "pack": 0x1234,
                                "tile_off": 0x5678,
                            },
                        ],
                    }
                )
            )
            image = Image.new("P", (16, 8))
            image.putdata(([14] + [0] * 63) * 2)
            image.save(source_dir / "assets_by_source.png")

            written = write_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            self.assertIn(asset_dir / "atlas/art_tiles.json", written)
            self.assertIn(asset_dir / "atlas/dynamic_bg3_tiles.json", written)
            main_manifest = json.loads((asset_dir / "atlas/art_tiles.json").read_text())
            dynamic_manifest = json.loads(
                (asset_dir / "atlas/dynamic_bg3_tiles.json").read_text()
            )

            refs = [
                source
                for art in main_manifest["arts"]
                for source in art["source_refs"]
                if source["source_kind"] == "bg3"
            ]
            self.assertEqual(len(refs), 1)
            self.assertEqual({ref["asset"] for ref in refs}, {"kBg3Gfx"})
            self.assertEqual(
                {(ref["pack"], ref["tile"]) for ref in refs},
                {(0x0C07, 0)},
            )
            self.assertEqual({ref["bpp"] for ref in refs}, {5})
            self.assertEqual({ref["runtime_colors_per_row"] for ref in refs}, {32})
            self.assertEqual({ref["runtime_material_policy"] for ref in refs}, {"stable"})

            dynamic_refs = [
                source
                for art in dynamic_manifest["arts"]
                for source in art["source_refs"]
                if source["source_kind"] == "bg3_dynamic"
            ]
            self.assertEqual(dynamic_manifest["format"], "zelda3_dynamic_bg3_art_atlas_v1")
            self.assertEqual(len(dynamic_refs), 1)
            self.assertEqual({ref["asset"] for ref in dynamic_refs}, {"kBg3TextGfx"})
            self.assertEqual(
                {(ref["pack"], ref["tile"]) for ref in dynamic_refs},
                {(0x1234, 0x5678)},
            )
            self.assertEqual({ref["bpp"] for ref in dynamic_refs}, {5})
            self.assertEqual({ref["runtime_colors_per_row"] for ref in dynamic_refs}, {32})
            self.assertEqual(
                {ref["runtime_material_policy"] for ref in dynamic_refs},
                {"requires_live_palette"},
            )

    def test_canonical_art_atlas_imports_mode7_asset66_as_direct_palette_art(self) -> None:
        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        palette_colors = [f"#{index:02x}{index:02x}{index:02x}" for index in range(256)]
        mode7_chars = bytearray(0x4000)
        mode7_chars[0] = 110
        mode7_chars[64] = 1

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(
                pack_arrays([compressed_literal(bytes(raw_pack))])
            )
            (assets_dir / "066-kOverworldMapGfx.bin").write_bytes(bytes(mode7_chars))
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_overworld_bg_main.json",
                palette_colors,
            )

            _width, _height, _pixels, arts = build_canonical_art_atlas(asset_dir)

            mode7_arts = [
                art
                for art in arts
                if any(source["source_kind"] == "mode7" for source in art["source_refs"])
            ]
            refs = [
                source
                for art in arts
                for source in art["source_refs"]
                if source["source_kind"] == "mode7"
            ]
            self.assertTrue(all("indices_hex" in art for art in mode7_arts))
            self.assertIn(mode7_chars[:64].hex(), {art["indices_hex"] for art in mode7_arts})
            self.assertEqual(len(refs), 256)
            self.assertEqual({ref["asset"] for ref in refs}, {"kOverworldMapGfx"})
            self.assertEqual({ref["pack"] for ref in refs}, {0})
            self.assertEqual({ref["bpp"] for ref in refs}, {8})
            self.assertEqual({ref["preview_palette"] for ref in refs}, {"palette_overworld_bg_main"})
            self.assertEqual({ref["runtime_material"] for ref in refs}, {"palette_lut"})
            self.assertEqual({ref["runtime_material_policy"] for ref in refs}, {"stable"})
            self.assertEqual({ref["runtime_colors_per_row"] for ref in refs}, {128})

    def test_canonical_art_atlas_imports_link_content_source_tiles(self) -> None:
        import json
        from PIL import Image

        palette_colors = [
            "#000000",
            "#101010",
            "#202020",
            "#303030",
            "#404040",
            "#505050",
            "#606060",
            "#707070",
        ] * 16
        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]

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
            source_dir = asset_dir / "developer_tilesets"
            source_dir.mkdir()
            (source_dir / "assets_by_source.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_assets_by_source_v2_png",
                        "cell_count": 1,
                        "cells": [
                            {
                                "id": 0,
                                "key": (8 << 32) | (0x1234 << 16) | 0x5678,
                                "kind": 8,
                                "pack": 0x1234,
                                "tile_off": 0x5678,
                            }
                        ],
                    }
                )
            )
            image = Image.new("P", (8, 8))
            image.putdata([7] + [0] * 63)
            image.save(source_dir / "assets_by_source.png")

            _width, _height, _pixels, arts = build_canonical_art_atlas(
                asset_dir,
                source_tiles_dir=source_dir,
            )

            refs = [
                source
                for art in arts
                for source in art["source_refs"]
                if source["source_kind"] == "link"
            ]
            self.assertEqual(len(refs), 1)
            self.assertEqual(refs[0]["asset"], "kLinkGfx")
            self.assertEqual(refs[0]["pack"], 0x1234)
            self.assertEqual(refs[0]["tile"], 0x5678)
            self.assertEqual(refs[0]["bpp"], 3)
            self.assertEqual(refs[0]["runtime_colors_per_row"], 8)
            self.assertEqual(refs[0]["runtime_material_policy"], "stable")

    def test_write_canonical_art_atlas_emits_art_tiles_manifest(self) -> None:
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

            written = write_canonical_art_atlas(asset_dir)

            self.assertIn(asset_dir / "atlas/art_tiles.png", written)
            self.assertIn(asset_dir / "atlas/art_tiles.json", written)
            self.assertIn(asset_dir / "atlas/dynamic_bg3_tiles.png", written)
            self.assertIn(asset_dir / "atlas/dynamic_bg3_tiles.json", written)
            with Image.open(asset_dir / "atlas/art_tiles.png") as image:
                self.assertEqual(image.mode, "RGBA")
            with Image.open(asset_dir / "atlas/dynamic_bg3_tiles.png") as image:
                self.assertEqual(image.mode, "RGBA")
            manifest = json.loads((asset_dir / "atlas/art_tiles.json").read_text())
            self.assertEqual(manifest["format"], "zelda3_canonical_art_atlas_v1")
            self.assertLess(manifest["art_count"], manifest["source_ref_count"])
            dynamic_manifest = json.loads(
                (asset_dir / "atlas/dynamic_bg3_tiles.json").read_text()
            )
            self.assertEqual(dynamic_manifest["format"], "zelda3_dynamic_bg3_art_atlas_v1")

    def test_write_canonical_art_atlas_emits_dialogue_glyph_source_when_available(self) -> None:
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
            (assets_dir / "095-kDialogueFont.bin").write_bytes(bytes(0x1000))
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                palette_colors,
            )
            chr_dir = asset_dir / "assets_src/chr"
            chr_dir.mkdir(parents=True)
            (chr_dir / "1w-2d.json").write_text(
                json.dumps(
                    {
                        "format": "zelda3_editable_chr_sheet_v1",
                        "layout": {
                            "columns": 2,
                            "rows": 2,
                            "tile_width": 8,
                            "tile_height": 8,
                        },
                        "blocks": [
                            {
                                "block": "1w-2d.DAT3N",
                                "source_bpp": 2,
                                "source_kind": "sprite",
                                "source_pack": 103,
                                "tile_count": 2,
                                "tile_start": 0,
                            },
                            {
                                "block": "1w-2d.DAT8N",
                                "source_bpp": 3,
                                "source_kind": "bg",
                                "source_pack": 0,
                                "tile_count": 2,
                                "tile_start": 2,
                            },
                        ],
                    }
                )
            )
            image = Image.new("P", (16, 16))
            image.putpalette([0, 0, 0, 255, 255, 255] + [0, 0, 0] * 254)
            image.putdata(([0] * 64) + ([1] * 64) + ([0] * 128))
            image.save(chr_dir / "1w-2d.png")

            written = write_canonical_art_atlas(asset_dir)

            self.assertIn(asset_dir / "atlas/dialogue_glyph_tiles.png", written)
            self.assertIn(asset_dir / "atlas/dialogue_glyph_tiles.json", written)
            self.assertIn(asset_dir / "atlas/dialogue_vwf_font.json", written)
            self.assertIn(asset_dir / "atlas/dialogue_font_tiles.png", written)
            self.assertIn(asset_dir / "atlas/dialogue_font_tiles.json", written)
            manifest = json.loads((asset_dir / "atlas/dialogue_glyph_tiles.json").read_text())
            self.assertEqual(manifest["format"], "zelda3_dialogue_glyph_source_atlas_v1")
            self.assertEqual(manifest["tile_count"], 2)
            self.assertEqual(
                [tile["source_block"] for tile in manifest["tiles"]],
                ["1w-2d.DAT3N", "1w-2d.DAT3N"],
            )
            vwf_manifest = json.loads((asset_dir / "atlas/dialogue_vwf_font.json").read_text())
            self.assertEqual(vwf_manifest["format"], "zelda3_dialogue_vwf_font_v1")
            self.assertEqual(vwf_manifest["glyph_count"], 0)
            font_tiles_manifest = json.loads(
                (asset_dir / "atlas/dialogue_font_tiles.json").read_text()
            )
            self.assertEqual(
                font_tiles_manifest["format"],
                "zelda3_dialogue_font_tile_atlas_v1",
            )
            self.assertEqual(font_tiles_manifest["tile_count"], 256)
            self.assertEqual(font_tiles_manifest["tiles"][0]["source_tile"], 0)

    def test_write_canonical_art_atlas_emits_vwf_font_metadata_when_available(self) -> None:
        import json
        from PIL import Image

        raw_pack = bytearray(1536)
        raw_pack[0] = 0x80
        sprite_items = [bytes(raw_pack)] * 12 + [compressed_literal(bytes(raw_pack))]
        palette_colors = ["#000000", "#ffffff"] * 64

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(
                pack_arrays([compressed_literal(bytes(raw_pack))])
            )
            (assets_dir / "095-kDialogueFont.bin").write_bytes(
                bytes(range(256)) * 16 + bytes([0, 3, 5, 8])
            )
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                palette_colors,
            )

            written = write_canonical_art_atlas(asset_dir)

            self.assertIn(asset_dir / "atlas/dialogue_vwf_font.json", written)
            self.assertIn(asset_dir / "atlas/dialogue_vwf_glyphs.png", written)
            self.assertIn(asset_dir / "atlas/dialogue_vwf_glyphs.json", written)
            self.assertIn(asset_dir / "atlas/dialogue_font_tiles.png", written)
            self.assertIn(asset_dir / "atlas/dialogue_font_tiles.json", written)
            manifest = json.loads((asset_dir / "atlas/dialogue_vwf_font.json").read_text())
            self.assertEqual(manifest["format"], "zelda3_dialogue_vwf_font_v1")
            self.assertEqual(manifest["font_data_size"], 0x1000)
            self.assertEqual(manifest["width_table_size"], 4)
            self.assertEqual(manifest["glyph_count"], 4)
            self.assertEqual(
                [glyph["width"] for glyph in manifest["glyphs"]],
                [0, 3, 5, 8],
            )
            self.assertEqual(manifest["glyphs"][2]["top_row_offset"], 32)
            self.assertEqual(manifest["glyphs"][2]["bottom_row_offset"], 288)
            glyph_manifest = json.loads(
                (asset_dir / "atlas/dialogue_vwf_glyphs.json").read_text()
            )
            self.assertEqual(
                glyph_manifest["format"], "zelda3_dialogue_vwf_glyph_atlas_v1"
            )
            self.assertEqual(glyph_manifest["glyph_count"], 4)
            self.assertEqual(glyph_manifest["width_table_size"], 4)
            self.assertEqual(glyph_manifest["width"], 256)
            self.assertEqual(glyph_manifest["height"], 16)
            self.assertEqual(glyph_manifest["glyphs"][2]["rect"], [32, 0, 16, 16])
            font_tiles_manifest = json.loads(
                (asset_dir / "atlas/dialogue_font_tiles.json").read_text()
            )
            self.assertEqual(font_tiles_manifest["tile_count"], 256)
            self.assertEqual(font_tiles_manifest["tiles"][2]["rect"], [16, 0, 8, 8])
            self.assertEqual(len(glyph_manifest["glyphs"][2]["indices_hex"]), 16 * 16 * 2)
            with Image.open(asset_dir / "atlas/dialogue_vwf_glyphs.png") as image:
                self.assertEqual(image.mode, "RGBA")
                self.assertEqual(image.size, (256, 16))

    def test_write_base_effect_atlas_emits_compact_art_and_effect_table(self) -> None:
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

            written = write_base_effect_atlas(asset_dir)

            self.assertIn(asset_dir / "atlas/base_tiles.png", written)
            self.assertIn(asset_dir / "atlas/base_tiles.json", written)
            self.assertIn(asset_dir / "atlas/tile_effects.json", written)
            with Image.open(asset_dir / "atlas/base_tiles.png") as image:
                self.assertEqual(image.mode, "RGBA")
            manifest = json.loads((asset_dir / "atlas/base_tiles.json").read_text())
            effects = json.loads((asset_dir / "atlas/tile_effects.json").read_text())
            self.assertEqual(manifest["format"], "zelda3_base_art_atlas_v1")
            self.assertEqual(effects["format"], "zelda3_tile_effect_table_v1")
            self.assertTrue(
                any(effect["colors_per_row"] == 32 for effect in effects["effects"])
            )
            self.assertLess(manifest["entry_count"], 2000)

    def test_write_tile_effect_table_omits_legacy_base_tiles(self) -> None:
        import json

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

            written = write_tile_effect_table(asset_dir)

            self.assertEqual(written, [asset_dir / "atlas/tile_effects.json"])
            self.assertFalse((asset_dir / "atlas/base_tiles.png").exists())
            self.assertFalse((asset_dir / "atlas/base_tiles.json").exists())
            effects = json.loads((asset_dir / "atlas/tile_effects.json").read_text())
            self.assertEqual(effects["format"], "zelda3_tile_effect_table_v1")
            self.assertTrue(
                any(effect["colors_per_row"] == 128 for effect in effects["effects"])
            )

    def test_known_static_palettes_are_stable(self) -> None:
        self.assertEqual(classify_palette_policy("palette_main_spr"), "stable")
        self.assertEqual(classify_palette_policy("palette_dung_bg_main"), "stable")

    def test_unknown_palette_requires_live_palette_until_proven(self) -> None:
        self.assertEqual(classify_palette_policy("palette_runtime_flash"), "requires_live_palette")


if __name__ == "__main__":
    unittest.main()
