#!/usr/bin/env python3
"""Emit ROM-derived editable CHR sheets for Zelda 3 graphics."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path

import extract_assets


@dataclass(frozen=True)
class DecodedPack:
    kind: str
    pack_index: int
    bpp: int
    tiles: list[bytes]


@dataclass(frozen=True)
class EditableChrSheet:
    name: str
    tiles: list[bytes]
    blocks: list[dict[str, object]]


@dataclass(frozen=True)
class PreviewPalette:
    name: str
    colors: list[list[int]]


CHR_SHEET_BLOCK_NUMBERS: list[tuple[str, list[int], bool]] = [
    ("2m-2q", [1], False),
    ("2r-2w", list(range(1, 13)), False),
    ("a-h", list(range(1, 17)), True),
    ("i-p", [1, 2, 3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15], True),
    ("q-x", list(range(3, 17)), True),
    ("y-1f", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16], True),
    ("1g-1n", [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16], True),
    ("1o-1v", list(range(2, 17)), True),
    ("1w-2d", list(range(1, 10)), True),
    ("2e-2l", list(range(1, 17)), True),
    ("2x-3b", list(range(1, 11)), True),
    ("3c-3j", list(range(1, 17)), True),
    ("3k-3r", list(range(1, 17)), True),
    ("3s-3z", [1, 2, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16], True),
    ("4a-4h", list(range(1, 17)), True),
    ("4i-4p", list(range(1, 16)), True),
    ("4q-4s", list(range(1, 7)), True),
    ("4t-4x", list(range(1, 6)), True),
]


def _block_name(sheet: str, block_number: int, has_n_suffix: bool) -> str:
    suffix = "N" if has_n_suffix else ""
    return f"{sheet}.DAT{block_number}{suffix}"


def build_editable_chr_sheets(
    sprite_packs: list[DecodedPack],
    bg_packs: list[DecodedPack],
) -> list[EditableChrSheet]:
    source_packs = sprite_packs + bg_packs
    source_index = 0
    sheets = []
    for sheet_name, block_numbers, has_n_suffix in CHR_SHEET_BLOCK_NUMBERS:
        sheet_tiles: list[bytes] = []
        blocks: list[dict[str, object]] = []
        for block_number in block_numbers:
            if source_index >= len(source_packs):
                break
            pack = source_packs[source_index]
            tile_start = len(sheet_tiles)
            sheet_tiles.extend(pack.tiles)
            blocks.append(
                {
                    "block": _block_name(sheet_name, block_number, has_n_suffix),
                    "source_kind": pack.kind,
                    "source_pack": pack.pack_index,
                    "source_bpp": pack.bpp,
                    "tile_start": tile_start,
                    "tile_count": len(pack.tiles),
                }
            )
            source_index += 1
        sheets.append(EditableChrSheet(name=sheet_name, tiles=sheet_tiles, blocks=blocks))
    return sheets


def decode_planar_tile_indices(data: bytes, bpp: int) -> list[bytes]:
    if bpp not in (2, 3, 4):
        raise ValueError(f"unsupported SNES tile bit depth: {bpp}")
    stride = {2: 16, 3: 24, 4: 32}[bpp]
    if len(data) % stride != 0:
        raise ValueError(f"{len(data)} bytes is not a multiple of {stride} for {bpp}bpp")

    tiles = []
    for tile_index in range(len(data) // stride):
        base = tile_index * stride
        pixels = bytearray()
        for y in range(8):
            plane0 = data[base + y * 2]
            plane1 = data[base + y * 2 + 1]
            plane2 = data[base + 16 + y] if bpp == 3 else 0
            plane2_4 = data[base + 16 + y * 2] if bpp == 4 else 0
            plane3 = data[base + 16 + y * 2 + 1] if bpp == 4 else 0
            for x in range(8):
                bit = 7 - x
                value = ((plane0 >> bit) & 1) | (((plane1 >> bit) & 1) << 1)
                if bpp == 3:
                    value |= ((plane2 >> bit) & 1) << 2
                elif bpp == 4:
                    value |= ((plane2_4 >> bit) & 1) << 2
                    value |= ((plane3 >> bit) & 1) << 3
                pixels.append(value)
        tiles.append(bytes(pixels))
    return tiles


def pack_tiles_to_sheet(tiles: list[bytes], columns: int) -> tuple[int, int, bytes]:
    if columns <= 0:
        raise ValueError("columns must be positive")
    if any(len(tile) != 64 for tile in tiles):
        raise ValueError("all CHR tiles must be 8x8 index data")

    rows = (len(tiles) + columns - 1) // columns
    width = columns * 8
    height = rows * 8
    pixels = bytearray(width * height)
    for tile_index, tile in enumerate(tiles):
        tile_x = tile_index % columns
        tile_y = tile_index // columns
        for y in range(8):
            dst = (tile_y * 8 + y) * width + tile_x * 8
            src = y * 8
            pixels[dst : dst + 8] = tile[src : src + 8]
    return width, height, bytes(pixels)


def preview_palette() -> list[int]:
    palette = palette_bytes(preview_palette_colors())
    return palette


def palette_bytes(colors: list[list[int]]) -> list[int]:
    palette = [channel for color in colors[:256] for channel in color]
    palette.extend([0] * (256 * 3 - len(palette)))
    return palette


def preview_palette_colors() -> list[list[int]]:
    return [
        [0, 0, 0],
        [255, 255, 255],
        [104, 176, 72],
        [24, 104, 56],
        [216, 64, 64],
        [120, 56, 40],
        [248, 184, 64],
        [96, 64, 160],
        [64, 136, 216],
        [40, 64, 112],
        [184, 184, 184],
        [96, 96, 96],
        [232, 136, 184],
        [32, 176, 184],
        [240, 216, 120],
        [32, 32, 32],
    ]


def _parse_rgb888(value: str) -> list[int]:
    if len(value) != 7 or not value.startswith("#"):
        raise ValueError(f"invalid rgb888 color: {value}")
    return [int(value[index : index + 2], 16) for index in (1, 3, 5)]


def _read_extracted_palette(asset_dir: Path, palette_name: str) -> PreviewPalette | None:
    path = asset_dir / "assets_src/palettes" / f"{palette_name}.json"
    if not path.is_file():
        return None
    data = json.loads(path.read_text())
    colors_by_index: dict[int, list[int]] = {}
    for color in data.get("colors", []):
        colors_by_index[int(color["index"])] = _parse_rgb888(str(color["rgb888"]))
    if not colors_by_index:
        return None
    color_count = min(max(colors_by_index) + 1, 256)
    colors = [colors_by_index.get(index, [0, 0, 0]) for index in range(color_count)]
    return PreviewPalette(name=palette_name, colors=colors)


def preview_palette_for_sheet(asset_dir: Path, sheet: EditableChrSheet) -> PreviewPalette:
    source_kinds = [str(block.get("source_kind")) for block in sheet.blocks]
    bg_count = source_kinds.count("bg")
    sprite_count = source_kinds.count("sprite")
    preferred = "palette_dung_bg_main" if bg_count > sprite_count else "palette_main_spr"
    return _read_extracted_palette(asset_dir, preferred) or PreviewPalette(
        name="developer_default",
        colors=preview_palette_colors(),
    )


def write_chr_sheet_png(
    path: Path,
    tiles: list[bytes],
    columns: int,
    palette: list[int] | None = None,
) -> None:
    from PIL import Image

    width, height, pixels = pack_tiles_to_sheet(tiles, columns)
    path.parent.mkdir(parents=True, exist_ok=True)
    image = Image.new("P", (width, height))
    image.putpalette(palette or preview_palette())
    image.putdata(pixels)
    image.save(path)


def write_chr_sheet_sidecar(path: Path, manifest: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


def _decoded_pack_bpp(data: bytes) -> int:
    if len(data) == 2048:
        return 2
    if len(data) % 24 == 0:
        return 3
    if len(data) % 32 == 0:
        return 4
    if len(data) % 16 == 0:
        return 2
    raise ValueError(f"cannot infer SNES CHR bit depth for {len(data)} bytes")


def _read_packed_chr_asset(
    path: Path,
    kind: str,
    uncompressed_prefix_count: int = 0,
) -> list[DecodedPack]:
    items = extract_assets.unpack_packed_arrays(path.read_bytes())
    packs = []
    for index, item in enumerate(items):
        data = item if index < uncompressed_prefix_count else extract_assets.decomp_asset(item)
        bpp = _decoded_pack_bpp(data)
        packs.append(
            DecodedPack(
                kind=kind,
                pack_index=index,
                bpp=bpp,
                tiles=decode_planar_tile_indices(data, bpp),
            )
        )
    return packs


def read_decoded_chr_packs(asset_dir: Path) -> tuple[list[DecodedPack], list[DecodedPack]]:
    assets_dir = asset_dir / "assets"
    sprite_path = assets_dir / "064-kSprGfx.bin"
    bg_path = assets_dir / "065-kBgGfx.bin"
    if not sprite_path.is_file():
        raise FileNotFoundError(sprite_path)
    if not bg_path.is_file():
        raise FileNotFoundError(bg_path)
    return (
        _read_packed_chr_asset(sprite_path, "sprite", uncompressed_prefix_count=12),
        _read_packed_chr_asset(bg_path, "bg"),
    )


def sidecar_for_sheet(
    asset_dir: Path,
    sheet: EditableChrSheet,
    columns: int,
    palette: PreviewPalette | None = None,
) -> dict[str, object]:
    rows = (len(sheet.tiles) + columns - 1) // columns
    palette = palette or PreviewPalette(name="developer_default", colors=preview_palette_colors())
    return {
        "format": "zelda3_editable_chr_sheet_v1",
        "sheet": sheet.name,
        "source": {
            "kind": "rom_extracted_assets",
            "asset_dir": str(asset_dir),
        },
        "layout": {
            "tile_width": 8,
            "tile_height": 8,
            "columns": columns,
            "rows": rows,
        },
        "palette": {
            "preview": palette.name,
            "mode": "indexed_png",
            "index_to_rgb": palette.colors,
        },
        "blocks": sheet.blocks,
    }


def write_editable_chr_sheets(asset_dir: Path, out_dir: Path | None = None) -> list[Path]:
    sprite_packs, bg_packs = read_decoded_chr_packs(asset_dir)
    sheets = build_editable_chr_sheets(sprite_packs, bg_packs)
    destination = out_dir or asset_dir / "assets_src/chr"
    written = []
    for sheet in sheets:
        if not sheet.tiles:
            continue
        png_path = destination / f"{sheet.name}.png"
        json_path = destination / f"{sheet.name}.json"
        palette = preview_palette_for_sheet(asset_dir, sheet)
        write_chr_sheet_png(png_path, sheet.tiles, columns=16, palette=palette_bytes(palette.colors))
        write_chr_sheet_sidecar(json_path, sidecar_for_sheet(asset_dir, sheet, columns=16, palette=palette))
        written.extend([png_path, json_path])
    return written


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-dir", required=True, type=Path)
    parser.add_argument("--out-dir", type=Path)
    args = parser.parse_args()
    for path in write_editable_chr_sheets(args.asset_dir, args.out_dir):
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
