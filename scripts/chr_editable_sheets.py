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
class SheetPaletteRow:
    """One palette row shown by the sheet PNG.

    `base` is the PNG palette slot of the row's color 0: a tile assigned to this
    row stores pixel values `base + raw_chr_index`, so decoding back to CHR
    indices never depends on the colors themselves.
    """

    id: int
    palette: str
    palette_row: int
    colors_per_row: int
    base: int
    index_to_rgb: list[list[int]]
    preview_source: str


@dataclass(frozen=True)
class SheetPalettePlan:
    rows: list[SheetPaletteRow]
    tile_row_ids: list[int]

    def combined_colors(self) -> list[list[int]]:
        colors: list[list[int]] = []
        for row in self.rows:
            colors.extend(row.index_to_rgb)
        return colors

    def transparency_bytes(self) -> bytes:
        alphas = bytearray([255] * 256)
        for row in self.rows:
            alphas[row.base] = 0
        return bytes(alphas)


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


def read_palette_colors(path: Path) -> list[list[int]]:
    data = json.loads(path.read_text())
    colors_by_index: dict[int, list[int]] = {}
    for color in data.get("colors", []):
        colors_by_index[int(color["index"])] = _parse_rgb888(str(color["rgb888"]))
    if not colors_by_index:
        raise ValueError(f"{path}: palette has no colors")
    return [colors_by_index.get(index, [0, 0, 0]) for index in range(max(colors_by_index) + 1)]


def _default_palette_names(asset_dir: Path) -> list[str]:
    palettes_dir = asset_dir / "assets_src/palettes"
    preferred = ["palette_main_spr", "palette_dung_bg_main", "palette_overworld_bg_main"]
    available = sorted(path.stem for path in palettes_dir.glob("*.json"))
    ordered = [name for name in preferred if name in available]
    ordered.extend(name for name in available if name not in ordered)
    return ordered


def _rows_for_bpp(bpp: int) -> tuple[int, int]:
    if bpp == 2:
        return 4, 4
    if bpp == 3:
        return 8, 8
    if bpp == 4:
        return 16, 16
    if bpp == 5:
        return 8, 32
    if bpp == 8:
        return 2, 128
    raise ValueError(f"unsupported SNES tile bit depth: {bpp}")


def _default_preview_palette(
    kind: str,
    available_palette_names: list[str],
) -> tuple[str, int]:
    preferred = {
        "sprite": "palette_main_spr",
        "link": "palette_main_spr",
        "bg": "palette_overworld_bg_main",
        "bg3": "palette_overworld_bg_main",
        "bg3_dynamic": "palette_overworld_bg_main",
        "mode7": "palette_overworld_bg_main",
    }.get(kind)
    if preferred in available_palette_names:
        return preferred, 0
    if "palette_dung_bg_main" in available_palette_names and kind in ("bg", "bg3"):
        return "palette_dung_bg_main", 0
    if available_palette_names:
        return available_palette_names[0], 0
    raise FileNotFoundError("no extracted palettes available for base atlas")


def _asset_name_for_kind(kind: str) -> str:
    if kind == "sprite":
        return "kSprGfx"
    if kind == "bg":
        return "kBgGfx"
    if kind == "mode7":
        return "kOverworldMapGfx"
    raise ValueError(f"unknown decoded CHR pack kind: {kind}")


def _tile_usage_key(
    kind: str,
    asset: str,
    pack: int,
    tile: int,
    bpp: int,
) -> tuple[str, str, int, int, int]:
    return kind, asset, pack, tile, bpp


def _palette_usage_paths(asset_dir: Path) -> list[Path]:
    return [
        asset_dir / "atlas/palette_usage.json",
        asset_dir / "assets_src/palette_usage.json",
    ]


def read_palette_usage_map(asset_dir: Path) -> dict[tuple[str, str, int, int, int], dict[str, object]]:
    for path in _palette_usage_paths(asset_dir):
        if not path.is_file():
            continue
        data = json.loads(path.read_text())
        if data.get("format") != "zelda3_palette_usage_v1":
            raise ValueError(f"{path}: unsupported palette usage format {data.get('format')!r}")
        usage = {}
        for entry in data.get("entries", []):
            source_kind = str(entry["source_kind"])
            asset = str(entry["asset"])
            pack = int(entry["pack"])
            tile = int(entry["tile"])
            bpp = int(entry["bpp"])
            usage[_tile_usage_key(source_kind, asset, pack, tile, bpp)] = entry
        return usage
    return {}


def _preview_palette_for_tile(
    *,
    kind: str,
    asset: str,
    pack: int,
    tile: int,
    bpp: int,
    colors_per_row: int,
    available_palette_names: list[str],
    palettes: dict[str, list[list[int]]],
    palette_usage: dict[tuple[str, str, int, int, int], dict[str, object]],
) -> tuple[str, int, str, dict[str, object] | None]:
    usage = palette_usage.get(_tile_usage_key(kind, asset, pack, tile, bpp))
    if usage is not None:
        usage_palette = str(usage.get("preview_palette", ""))
        usage_row = int(usage.get("preview_palette_row", -1))
        if usage_palette in palettes and usage_row >= 0:
            colors = palettes[usage_palette]
            if (usage_row + 1) * colors_per_row <= len(colors):
                return usage_palette, usage_row, "palette_usage", usage

    preview_palette, preview_row = _default_preview_palette(kind, available_palette_names)
    colors = palettes[preview_palette]
    if (preview_row + 1) * colors_per_row > len(colors):
        preview_row = 0
    return preview_palette, preview_row, "source_kind_default", None


def _row_colors(
    palettes: dict[str, list[list[int]]],
    palette_name: str,
    palette_row: int,
    colors_per_row: int,
) -> list[list[int]]:
    colors = palettes[palette_name]
    row = [list(color) for color in colors[palette_row * colors_per_row : (palette_row + 1) * colors_per_row]]
    row.extend([[0, 0, 0]] * (colors_per_row - len(row)))
    return row


def _read_sheet_palettes(asset_dir: Path) -> tuple[list[str], dict[str, list[list[int]]]]:
    palettes_dir = asset_dir / "assets_src/palettes"
    available = _default_palette_names(asset_dir) if palettes_dir.is_dir() else []
    palettes = {
        name: read_palette_colors(palettes_dir / f"{name}.json") for name in available
    }
    if not palettes:
        available = ["developer_default"]
        palettes = {"developer_default": preview_palette_colors()}
    return available, palettes


def compute_sheet_palette_plan(asset_dir: Path, sheet: EditableChrSheet) -> SheetPalettePlan:
    """Assign every sheet tile its usual in-game palette row.

    Row choice mirrors the canonical-art atlas (`palette_usage.json` evidence,
    then per-kind defaults). All rows share one <=256-slot PNG palette; when the
    combined rows overflow it, the least-evidenced rows are demoted to the
    tile's per-kind default row — decode never depends on the demotion because
    the raw CHR index is always `pixel - base`.
    """
    available, palettes = _read_sheet_palettes(asset_dir)
    palette_usage = read_palette_usage_map(asset_dir)

    row_key_of_tile: list[tuple[str, int, int]] = []
    default_key_of_tile: list[tuple[str, int, int]] = []
    row_meta: dict[tuple[str, int, int], dict[str, object]] = {}

    def note_row(key: tuple[str, int, int], source: str, evidence: int, order: int) -> None:
        meta = row_meta.setdefault(
            key,
            {"source": source, "evidence": evidence, "order": order, "default": False},
        )
        meta["evidence"] = max(int(meta["evidence"]), evidence)
        if source == "palette_usage":
            meta["source"] = "palette_usage"

    order = 0
    for block in sheet.blocks:
        kind = str(block["source_kind"])
        asset = _asset_name_for_kind(kind)
        pack = int(block["source_pack"])
        bpp = int(block["source_bpp"])
        tile_count = int(block["tile_count"])
        _, colors_per_row = _rows_for_bpp(bpp)
        default_palette, default_row = _default_preview_palette(kind, available)
        if (default_row + 1) * colors_per_row > len(palettes[default_palette]):
            default_row = 0
        default_key = (default_palette, default_row, colors_per_row)
        row_meta.setdefault(
            default_key,
            {"source": "source_kind_default", "evidence": 0, "order": order, "default": True},
        )["default"] = True
        for tile in range(tile_count):
            palette_name, palette_row, source, usage = _preview_palette_for_tile(
                kind=kind,
                asset=asset,
                pack=pack,
                tile=tile,
                bpp=bpp,
                colors_per_row=colors_per_row,
                available_palette_names=available,
                palettes=palettes,
                palette_usage=palette_usage,
            )
            evidence = int(usage.get("evidence_count", 0)) if usage is not None else 0
            key = (palette_name, palette_row, colors_per_row)
            order += 1
            note_row(key, source, evidence, order)
            row_key_of_tile.append(key)
            default_key_of_tile.append(default_key)

    # Allocate PNG palette slots: per-kind default rows first (demotion targets
    # must always fit), then observed rows by evidence.
    defaults = [key for key, meta in row_meta.items() if meta["default"]]
    defaults.sort(key=lambda key: int(row_meta[key]["order"]))
    observed = [key for key, meta in row_meta.items() if not meta["default"]]
    observed.sort(key=lambda key: (-int(row_meta[key]["evidence"]), int(row_meta[key]["order"])))

    rows: list[SheetPaletteRow] = []
    row_id_by_key: dict[tuple[str, int, int], int] = {}
    base = 0
    for key in defaults + observed:
        palette_name, palette_row, colors_per_row = key
        if base + colors_per_row > 256:
            continue
        row_id_by_key[key] = len(rows)
        rows.append(
            SheetPaletteRow(
                id=len(rows),
                palette=palette_name,
                palette_row=palette_row,
                colors_per_row=colors_per_row,
                base=base,
                index_to_rgb=_row_colors(palettes, palette_name, palette_row, colors_per_row),
                preview_source=str(row_meta[key]["source"]),
            )
        )
        base += colors_per_row

    tile_row_ids = [
        row_id_by_key.get(key, row_id_by_key[default_key])
        for key, default_key in zip(row_key_of_tile, default_key_of_tile)
    ]
    return SheetPalettePlan(rows=rows, tile_row_ids=tile_row_ids)


def write_chr_sheet_png(
    path: Path,
    tiles: list[bytes],
    columns: int,
    palette: list[int] | None = None,
    transparency: bytes | None = None,
) -> None:
    from PIL import Image

    width, height, pixels = pack_tiles_to_sheet(tiles, columns)
    path.parent.mkdir(parents=True, exist_ok=True)
    image = Image.new("P", (width, height))
    image.putpalette(palette or preview_palette())
    image.putdata(pixels)
    if transparency is None:
        image.save(path)
    else:
        image.save(path, transparency=transparency)


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
    plan: SheetPalettePlan,
) -> dict[str, object]:
    rows = (len(sheet.tiles) + columns - 1) // columns
    blocks = []
    for block in sheet.blocks:
        tile_start = int(block["tile_start"])
        tile_count = int(block["tile_count"])
        block_v2 = dict(block)
        block_v2["tile_palette_rows"] = plan.tile_row_ids[tile_start : tile_start + tile_count]
        blocks.append(block_v2)
    return {
        "format": "zelda3_editable_chr_sheet_v2",
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
            "mode": "indexed_png_per_tile_rows",
            "index_to_rgb": plan.combined_colors(),
        },
        "palette_rows": [
            {
                "id": row.id,
                "palette": row.palette,
                "palette_row": row.palette_row,
                "colors_per_row": row.colors_per_row,
                "base": row.base,
                "index_to_rgb": row.index_to_rgb,
                "preview_source": row.preview_source,
            }
            for row in plan.rows
        ],
        "blocks": blocks,
    }


def remap_tiles_for_plan(sheet: EditableChrSheet, plan: SheetPalettePlan) -> list[bytes]:
    remapped = []
    for tile, row_id in zip(sheet.tiles, plan.tile_row_ids):
        base = plan.rows[row_id].base
        remapped.append(bytes(base + value for value in tile))
    return remapped


def write_editable_chr_sheets(asset_dir: Path, out_dir: Path) -> list[Path]:
    sprite_packs, bg_packs = read_decoded_chr_packs(asset_dir)
    sheets = build_editable_chr_sheets(sprite_packs, bg_packs)
    destination = out_dir
    written = []
    for sheet in sheets:
        if not sheet.tiles:
            continue
        png_path = destination / f"{sheet.name}.png"
        json_path = destination / f"{sheet.name}.json"
        plan = compute_sheet_palette_plan(asset_dir, sheet)
        write_chr_sheet_png(
            png_path,
            remap_tiles_for_plan(sheet, plan),
            columns=16,
            palette=palette_bytes(plan.combined_colors()),
            transparency=plan.transparency_bytes(),
        )
        write_chr_sheet_sidecar(json_path, sidecar_for_sheet(asset_dir, sheet, columns=16, plan=plan))
        written.extend([png_path, json_path])
    return written


def _sheet_tiles_from_png(png_path: Path, columns: int, tile_total: int) -> list[bytes]:
    from PIL import Image

    with Image.open(png_path) as image:
        if image.mode != "P":
            raise ValueError(f"{png_path}: editable CHR sheets must be indexed (P-mode) PNGs")
        width, height = image.size
        if width != columns * 8:
            raise ValueError(f"{png_path}: expected width {columns * 8}, found {width}")
        pixels = image.tobytes()
    tiles = []
    for tile_index in range(tile_total):
        tile_x = tile_index % columns
        tile_y = tile_index // columns
        tile = bytearray()
        for y in range(8):
            src = (tile_y * 8 + y) * width + tile_x * 8
            tile.extend(pixels[src : src + 8])
        tiles.append(bytes(tile))
    return tiles


def read_editable_chr_sheet(png_path: Path, json_path: Path) -> EditableChrSheet:
    """Decode an editable sheet PNG back to raw CHR indices per tile.

    Supports v1 (pixels are raw indices) and v2 (pixels are `base + index` per
    the sidecar's palette-row assignment). Pixels outside a tile's assigned row
    are hard errors — they mark an edit that crossed palette-row boundaries.
    """
    manifest = json.loads(json_path.read_text())
    fmt = manifest.get("format")
    if fmt not in ("zelda3_editable_chr_sheet_v1", "zelda3_editable_chr_sheet_v2"):
        raise ValueError(f"{json_path}: unsupported editable CHR sheet format {fmt!r}")
    columns = int(manifest["layout"]["columns"])
    blocks = list(manifest["blocks"])
    tile_total = max(
        int(block["tile_start"]) + int(block["tile_count"]) for block in blocks
    )
    raw_tiles = _sheet_tiles_from_png(png_path, columns, tile_total)

    if fmt == "zelda3_editable_chr_sheet_v1":
        tiles = raw_tiles
    else:
        rows_by_id = {int(row["id"]): row for row in manifest["palette_rows"]}
        tiles = list(raw_tiles)
        for block in blocks:
            tile_start = int(block["tile_start"])
            tile_count = int(block["tile_count"])
            row_ids = block["tile_palette_rows"]
            if len(row_ids) != tile_count:
                raise ValueError(
                    f"{json_path}: block {block['block']} tile_palette_rows length "
                    f"{len(row_ids)} != tile_count {tile_count}"
                )
            for offset, row_id in enumerate(row_ids):
                row = rows_by_id[int(row_id)]
                base = int(row["base"])
                colors_per_row = int(row["colors_per_row"])
                decoded = bytearray()
                for value in raw_tiles[tile_start + offset]:
                    index = value - base
                    if not 0 <= index < colors_per_row:
                        raise ValueError(
                            f"{png_path}: tile {tile_start + offset} pixel value {value} is "
                            f"outside its palette row (base {base}, {colors_per_row} colors); "
                            "edits must stay within the tile's assigned row"
                        )
                    decoded.append(index)
                tiles[tile_start + offset] = bytes(decoded)

    return EditableChrSheet(
        name=str(manifest["sheet"]),
        tiles=tiles,
        blocks=blocks,
    )


def read_decoded_chr_packs_from_sheets(
    asset_dir: Path,
    sheet_dir: Path,
) -> tuple[list[DecodedPack], list[DecodedPack]]:
    """Reconstruct the decoded CHR packs with the editable sheets as authority.

    Packs covered by no sheet (bg pack 114) are read from the packed binaries.
    """
    bin_sprite_packs, bin_bg_packs = read_decoded_chr_packs(asset_dir)
    packs_by_key: dict[tuple[str, int], DecodedPack] = {}
    for sheet_name, _block_numbers, _has_n_suffix in CHR_SHEET_BLOCK_NUMBERS:
        json_path = sheet_dir / f"{sheet_name}.json"
        png_path = sheet_dir / f"{sheet_name}.png"
        if not json_path.is_file() or not png_path.is_file():
            raise FileNotFoundError(json_path if not json_path.is_file() else png_path)
        sheet = read_editable_chr_sheet(png_path, json_path)
        for block in sheet.blocks:
            kind = str(block["source_kind"])
            pack_index = int(block["source_pack"])
            bpp = int(block["source_bpp"])
            tile_start = int(block["tile_start"])
            tile_count = int(block["tile_count"])
            packs_by_key[(kind, pack_index)] = DecodedPack(
                kind=kind,
                pack_index=pack_index,
                bpp=bpp,
                tiles=sheet.tiles[tile_start : tile_start + tile_count],
            )

    def merged(bin_packs: list[DecodedPack]) -> list[DecodedPack]:
        return [
            packs_by_key.get((pack.kind, pack.pack_index), pack) for pack in bin_packs
        ]

    return merged(bin_sprite_packs), merged(bin_bg_packs)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-dir", required=True, type=Path)
    parser.add_argument(
        "--out-dir",
        required=True,
        type=Path,
        help="Sheet destination (the tracked authority is assets/chr)",
    )
    args = parser.parse_args()
    for path in write_editable_chr_sheets(args.asset_dir, args.out_dir):
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
