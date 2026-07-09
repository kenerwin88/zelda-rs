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

            written = chr_editable_sheets.write_editable_chr_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )

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

            chr_editable_sheets.write_editable_chr_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )

            image = Image.open(asset_dir / "assets_src/chr/a-h.png")
            sidecar = json.loads((asset_dir / "assets_src/chr/a-h.json").read_text())
            self.assertEqual(
                image.getpalette()[:24],
                [1, 2, 3, 17, 18, 19, 33, 34, 35, 49, 50, 51, 65, 66, 67, 81, 82, 83, 97, 98, 99, 113, 114, 115],
            )
            self.assertEqual(sidecar["format"], "zelda3_editable_chr_sheet_v2")
            self.assertEqual(sidecar["palette"]["mode"], "indexed_png_per_tile_rows")
            self.assertEqual(sidecar["palette"]["index_to_rgb"][0], [1, 2, 3])
            self.assertEqual(sidecar["palette_rows"][0]["palette"], "palette_main_spr")
            self.assertEqual(sidecar["palette_rows"][0]["base"], 0)
            for block in sidecar["blocks"]:
                self.assertEqual(len(block["tile_palette_rows"]), block["tile_count"])

    def test_palette_usage_recolors_tile_with_observed_row(self) -> None:
        from PIL import Image

        # One sprite tile whose pixels are index 1 everywhere.
        tile_planar = bytearray(24)
        for y in range(8):
            tile_planar[y * 2] = 0xFF  # plane 0 set -> index 1
        raw_pack = bytes(tile_planar) * 64
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)] * 210
        palette_colors = [f"#{value:02x}0000" for value in range(32)]

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(pack_arrays([compressed_literal(raw_pack)]))
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                "kPalette_MainSpr",
                palette_colors,
            )
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True)
            usage_path.write_text(
                json.dumps(
                    {
                        "format": "zelda3_palette_usage_v1",
                        "entries": [
                            {
                                "source_kind": "sprite",
                                "asset": "kSprGfx",
                                "pack": 13,
                                "tile": 0,
                                "bpp": 3,
                                "preview_palette": "palette_main_spr",
                                "preview_palette_row": 2,
                                "evidence_count": 100,
                            }
                        ],
                    }
                )
            )

            chr_editable_sheets.write_editable_chr_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )

            sidecar = json.loads((asset_dir / "assets_src/chr/a-h.json").read_text())
            # Row 0 is the sheet default (palette row 0); the observed row 2
            # follows it at base 8.
            observed = [row for row in sidecar["palette_rows"] if row["palette_row"] == 2]
            self.assertEqual(len(observed), 1)
            self.assertEqual(observed[0]["preview_source"], "palette_usage")
            self.assertEqual(observed[0]["base"], 8)
            self.assertEqual(observed[0]["index_to_rgb"][1], [17, 0, 0])
            block = sidecar["blocks"][0]
            self.assertEqual(block["tile_palette_rows"][0], observed[0]["id"])
            self.assertEqual(block["tile_palette_rows"][1], 0)
            # Pixels of the remapped tile live at base 8 + index 1 = 9.
            image = Image.open(asset_dir / "assets_src/chr/a-h.png")
            self.assertEqual(image.tobytes()[0], 9)
            # And decode losslessly back to raw index 1.
            sheet = chr_editable_sheets.read_editable_chr_sheet(
                asset_dir / "assets_src/chr/a-h.png",
                asset_dir / "assets_src/chr/a-h.json",
            )
            self.assertEqual(sheet.tiles[0], bytes([1] * 64))
            self.assertEqual(sheet.tiles[1], bytes([1] * 64))

    def test_sheet_roundtrip_matches_packs(self) -> None:
        import random

        rng = random.Random(1234)
        packs = []
        for index in range(222):
            planar = bytearray()
            for _tile in range(2):
                planar.extend(rng.randrange(256) for _ in range(24))
            packs.append(bytes(planar))
        sprite_items = packs[:12] + [compressed_literal(item) for item in packs[12:108]]
        bg_items = [compressed_literal(item) for item in packs[108:]]

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(pack_arrays(bg_items))

            chr_editable_sheets.write_editable_chr_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )

            from_bins = chr_editable_sheets.read_decoded_chr_packs(asset_dir)
            from_sheets = chr_editable_sheets.read_decoded_chr_packs_from_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )
            for bin_packs, sheet_packs in zip(from_bins, from_sheets):
                self.assertEqual(len(bin_packs), len(sheet_packs))
                for bin_pack, sheet_pack in zip(bin_packs, sheet_packs):
                    self.assertEqual(bin_pack, sheet_pack)

    def test_palette_plan_demotes_overflow_rows_to_default(self) -> None:
        palette_colors = [f"#{value % 256:02x}00ff" for value in range(33 * 8)]
        usage_entries = [
            {
                "source_kind": "sprite",
                "asset": "kSprGfx",
                "pack": 13,
                "tile": tile,
                "bpp": 3,
                "preview_palette": "palette_main_spr",
                "preview_palette_row": tile % 33,
                "evidence_count": 1000 - tile,
            }
            for tile in range(64)
        ]

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            write_palette_json(
                asset_dir / "assets_src/palettes/palette_main_spr.json",
                "kPalette_MainSpr",
                palette_colors,
            )
            usage_path = asset_dir / "atlas/palette_usage.json"
            usage_path.parent.mkdir(parents=True)
            usage_path.write_text(
                json.dumps({"format": "zelda3_palette_usage_v1", "entries": usage_entries})
            )
            sheet = chr_editable_sheets.EditableChrSheet(
                name="a-h",
                tiles=[bytes([tile % 8] * 64) for tile in range(64)],
                blocks=[
                    {
                        "block": "a-h.DAT1N",
                        "source_kind": "sprite",
                        "source_pack": 13,
                        "source_bpp": 3,
                        "tile_start": 0,
                        "tile_count": 64,
                    }
                ],
            )

            plan = chr_editable_sheets.compute_sheet_palette_plan(asset_dir, sheet)

            # 33 distinct 8-color rows (264 colors) cannot all fit in 256
            # slots; the plan keeps the highest-evidence rows and demotes the
            # rest to the default row (id 0).
            self.assertLessEqual(sum(row.colors_per_row for row in plan.rows), 256)
            self.assertEqual(plan.rows[0].palette_row, 0)
            demoted = [
                tile
                for tile, row_id in enumerate(plan.tile_row_ids)
                if row_id == 0 and tile % 33 != 0
            ]
            self.assertTrue(demoted, "expected at least one overflow tile demoted to default")
            # Highest-evidence rows (low tile numbers) survive.
            self.assertNotEqual(plan.tile_row_ids[1], 0)

    def test_read_editable_chr_sheet_rejects_out_of_row_pixel(self) -> None:
        from PIL import Image

        raw_pack = bytes([0] * 1536)
        sprite_items = [raw_pack] * 12 + [compressed_literal(raw_pack)] * 210

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets_dir = asset_dir / "assets"
            assets_dir.mkdir()
            (assets_dir / "064-kSprGfx.bin").write_bytes(pack_arrays(sprite_items))
            (assets_dir / "065-kBgGfx.bin").write_bytes(pack_arrays([compressed_literal(raw_pack)]))

            chr_editable_sheets.write_editable_chr_sheets(
                asset_dir, asset_dir / "assets_src/chr"
            )

            png_path = asset_dir / "assets_src/chr/a-h.png"
            image = Image.open(png_path)
            pixels = bytearray(image.tobytes())
            pixels[0] = 200  # far outside the tile's assigned row
            edited = Image.new("P", image.size)
            edited.putpalette(image.getpalette())
            edited.putdata(bytes(pixels))
            edited.save(png_path)

            with self.assertRaises(ValueError):
                chr_editable_sheets.read_editable_chr_sheet(
                    png_path, asset_dir / "assets_src/chr/a-h.json"
                )

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

    def test_sidecar_declares_per_tile_row_contract(self) -> None:
        sheet = chr_editable_sheets.EditableChrSheet(
            name="example",
            tiles=[bytes([0] * 64)],
            blocks=[
                {
                    "block": "example.DAT1",
                    "source_kind": "sprite",
                    "source_pack": 0,
                    "source_bpp": 3,
                    "tile_start": 0,
                    "tile_count": 1,
                }
            ],
        )
        with TemporaryDirectory() as temp_dir:
            plan = chr_editable_sheets.compute_sheet_palette_plan(Path(temp_dir), sheet)

            manifest = chr_editable_sheets.sidecar_for_sheet(
                Path(temp_dir), sheet, columns=16, plan=plan
            )

        self.assertEqual(manifest["format"], "zelda3_editable_chr_sheet_v2")
        self.assertEqual(manifest["palette"]["mode"], "indexed_png_per_tile_rows")
        # No extracted palettes: developer default colors back the single row.
        self.assertEqual(manifest["palette_rows"][0]["palette"], "developer_default")
        self.assertEqual(manifest["palette"]["index_to_rgb"][0], [0, 0, 0])
        self.assertEqual(manifest["palette"]["index_to_rgb"][1], [255, 255, 255])
        self.assertEqual(manifest["blocks"][0]["tile_palette_rows"], [0])


if __name__ == "__main__":
    unittest.main()
