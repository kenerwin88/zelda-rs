#!/usr/bin/env python3
"""Tests for ROM-derived editable CHR sheet helpers."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import chr_editable_sheets


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


def write_palette_json(path: Path, name: str, colors: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(
            {
                "asset": name,
                "colors": [
                    {"index": index, "rgb888": color, "snes_bgr15": "0x0000"}
                    for index, color in enumerate(colors)
                ],
            }
        )
    )


class ChrEditableSheetsTests(unittest.TestCase):
    def test_decode_2bpp_single_tile_returns_indices(self) -> None:
        data = bytes([0x80, 0x00] + [0x00, 0x00] * 7)

        tiles = chr_editable_sheets.decode_planar_tile_indices(data, 2)

        self.assertEqual(len(tiles), 1)
        self.assertEqual(tiles[0][0], 1)
        self.assertEqual(tiles[0][1:], bytes([0] * 63))

    def test_pack_tiles_to_sheet_places_tiles_left_to_right(self) -> None:
        tiles = [bytes([1] * 64), bytes([2] * 64)]

        width, height, pixels = chr_editable_sheets.pack_tiles_to_sheet(tiles, columns=2)

        self.assertEqual((width, height), (16, 8))
        self.assertEqual(pixels[:8], bytes([1] * 8))
        self.assertEqual(pixels[8:16], bytes([2] * 8))

    def test_build_editable_chr_sheets_uses_cgx_names(self) -> None:
        packs = [
            chr_editable_sheets.DecodedPack(
                kind="sprite",
                pack_index=i,
                bpp=3,
                tiles=[bytes([i % 8] * 64)] * 64,
            )
            for i in range(222)
        ]

        sheets = chr_editable_sheets.build_editable_chr_sheets(packs, [])

        self.assertEqual([sheet.name for sheet in sheets[:3]], ["2m-2q", "2r-2w", "a-h"])

    def test_build_editable_chr_sheets_preserves_block_provenance(self) -> None:
        packs = [
            chr_editable_sheets.DecodedPack(
                kind="sprite",
                pack_index=i,
                bpp=3,
                tiles=[bytes([i % 8] * 64)] * 64,
            )
            for i in range(222)
        ]

        sheet = chr_editable_sheets.build_editable_chr_sheets(packs, [])[1]

        self.assertEqual(sheet.name, "2r-2w")
        self.assertEqual(sheet.blocks[0]["source_kind"], "sprite")
        self.assertEqual(sheet.blocks[0]["source_pack"], 1)
        self.assertEqual(sheet.blocks[0]["block"], "2r-2w.DAT1")

    def test_write_editable_chr_sheets_writes_png_and_sidecar(self) -> None:
        raw_pack = bytes([0] * 1536)
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)] * 210

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(pack_arrays([compressed_literal(raw_pack)]))

            written = chr_editable_sheets.write_editable_chr_sheets(asset_dir)

            self.assertIn(asset_dir / "assets_src/chr/a-h.png", written)
            self.assertTrue((asset_dir / "assets_src/chr/a-h.json").is_file())

    def test_write_editable_chr_sheets_uses_rom_extracted_sprite_palette(self) -> None:
        from PIL import Image

        raw_pack = bytes([0] * 1536)
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)] * 210
        preview_colors = [
            "#010203",
            "#111213",
            "#212223",
            "#313233",
            "#414243",
            "#515253",
            "#616263",
            "#717273",
        ]

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(pack_arrays([compressed_literal(raw_pack)]))
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                "kPalette_MainSpr",
                preview_colors,
            )

            chr_editable_sheets.write_editable_chr_sheets(asset_dir)

            image = Image.open(asset_dir / "assets_src/chr/a-h.png")
            sidecar = json.loads((asset_dir / "assets_src/chr/a-h.json").read_text())
            self.assertEqual(
                image.getpalette()[:24],
                [1, 2, 3, 17, 18, 19, 33, 34, 35, 49, 50, 51, 65, 66, 67, 81, 82, 83, 97, 98, 99, 113, 114, 115],
            )
            self.assertEqual(sidecar["palette"]["preview"], "palette_main_spr")
            self.assertEqual(sidecar["palette"]["index_to_rgb"][0], [1, 2, 3])

    def test_write_chr_sheet_png_keeps_indexed_pixels_with_preview_palette(self) -> None:
        from PIL import Image

        tiles = [bytes([0, 1, 2, 3, 4, 5, 6, 7] * 8)]

        with TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "sheet.png"
            chr_editable_sheets.write_chr_sheet_png(path, tiles, columns=1)

            image = Image.open(path)

            self.assertEqual(image.mode, "P")
            self.assertEqual(list(image.tobytes())[:8], [0, 1, 2, 3, 4, 5, 6, 7])
            self.assertEqual(image.getpalette()[:24], chr_editable_sheets.preview_palette()[:24])

    def test_palette_bytes_limits_palette_to_png_index_capacity(self) -> None:
        colors = [[index % 256, index % 256, index % 256] for index in range(300)]

        palette = chr_editable_sheets.palette_bytes(colors)

        self.assertEqual(len(palette), 256 * 3)
        self.assertEqual(palette[-3:], [255, 255, 255])

    def test_sidecar_declares_indexed_png_contract_and_preview_colors(self) -> None:
        sheet = chr_editable_sheets.EditableChrSheet(
            name="example",
            tiles=[bytes([0] * 64)],
            blocks=[],
        )

        manifest = chr_editable_sheets.sidecar_for_sheet(Path("assets"), sheet, columns=16)

        self.assertEqual(manifest["palette"]["mode"], "indexed_png")
        self.assertEqual(manifest["palette"]["index_to_rgb"][0], [0, 0, 0])
        self.assertEqual(manifest["palette"]["index_to_rgb"][1], [255, 255, 255])


if __name__ == "__main__":
    unittest.main()
