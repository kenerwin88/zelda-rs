#!/usr/bin/env python3
"""Build ROM-traceable RGBA tile variants for Zelda 3 graphics."""

from __future__ import annotations

from dataclasses import asdict
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path

import chr_editable_sheets


@dataclass(frozen=True)
class VariantKey:
    source_kind: str
    asset: str
    pack: int
    tile: int
    bpp: int
    palette: str
    palette_row: int


@dataclass(frozen=True)
class RgbaVariant:
    key: VariantKey
    pixels: bytes


@dataclass(frozen=True)
class AtlasEntry:
    id: str
    key: VariantKey
    rect: tuple[int, int, int, int]
    sha1: str
    duplicate_of: str | None
    dynamic_policy: str


def variant_id(key: VariantKey) -> str:
    return (
        f"{key.source_kind}:{key.asset}:pack{key.pack}:tile{key.tile}:"
        f"{key.bpp}bpp:{key.palette}:row{key.palette_row}"
    )


def classify_palette_policy(palette_name: str) -> str:
    stable_palettes = {
        "palette_main_spr",
        "palette_dung_bg_main",
        "palette_overworld_bg_main",
    }
    return "stable" if palette_name in stable_palettes else "requires_live_palette"


def _build_effect_table_from_palettes(
    palettes: dict[str, list[list[int]]],
) -> dict[str, object]:
    effects_by_id: dict[str, dict[str, object]] = {}
    for name, colors in palettes.items():
        for colors_per_row in (4, 8, 16, 32, 128):
            for effect in _effect_rows_for_palette(name, colors, colors_per_row):
                effects_by_id.setdefault(str(effect["id"]), effect)
    return {
        "format": "zelda3_tile_effect_table_v1",
        "strategy": "canonical_art_plus_shader_effects",
        "effects": list(effects_by_id.values()),
    }


def rgba_tile_from_indices(
    indices: bytes,
    palette_colors: list[list[int]],
    palette_row: int,
    colors_per_row: int,
) -> bytes:
    if len(indices) != 64:
        raise ValueError("RGBA variant tiles must be built from one 8x8 index tile")
    if palette_row < 0:
        raise ValueError("palette_row must be non-negative")
    if colors_per_row <= 0:
        raise ValueError("colors_per_row must be positive")

    base = palette_row * colors_per_row
    out = bytearray()
    for index in indices:
        color_index = base + index
        if color_index >= len(palette_colors):
            raise ValueError(
                f"palette index {color_index} outside palette with {len(palette_colors)} colors"
            )
        r, g, b = palette_colors[color_index]
        alpha = 0 if index == 0 else 255
        out.extend([r, g, b, alpha])
    return bytes(out)


def pack_rgba_variants(
    variants: list[RgbaVariant],
    columns: int = 32,
) -> tuple[int, int, bytes, list[AtlasEntry]]:
    if columns <= 0:
        raise ValueError("columns must be positive")

    unique_pixels: list[bytes] = []
    sha_to_rect: dict[str, tuple[str, tuple[int, int, int, int]]] = {}
    entries: list[AtlasEntry] = []
    for variant in variants:
        if len(variant.pixels) != 8 * 8 * 4:
            raise ValueError("each RGBA variant must be one 8x8 RGBA tile")
        digest = hashlib.sha1(variant.pixels).hexdigest()
        entry_id = variant_id(variant.key)
        dynamic_policy = classify_palette_policy(variant.key.palette)
        if digest in sha_to_rect:
            original_id, rect = sha_to_rect[digest]
            entries.append(
                AtlasEntry(entry_id, variant.key, rect, digest, original_id, dynamic_policy)
            )
            continue

        unique_index = len(unique_pixels)
        x = (unique_index % columns) * 8
        y = (unique_index // columns) * 8
        rect = (x, y, 8, 8)
        unique_pixels.append(variant.pixels)
        sha_to_rect[digest] = (entry_id, rect)
        entries.append(AtlasEntry(entry_id, variant.key, rect, digest, None, dynamic_policy))

    rows = max(1, (len(unique_pixels) + columns - 1) // columns)
    width = columns * 8
    height = rows * 8
    atlas = bytearray(width * height * 4)
    for unique_index, tile in enumerate(unique_pixels):
        x = (unique_index % columns) * 8
        y = (unique_index // columns) * 8
        for row in range(8):
            dst = ((y + row) * width + x) * 4
            src = row * 8 * 4
            atlas[dst : dst + 8 * 4] = tile[src : src + 8 * 4]
    return width, height, bytes(atlas), entries


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


def _asset_name_for_kind(kind: str) -> str:
    if kind == "sprite":
        return "kSprGfx"
    if kind == "bg":
        return "kBgGfx"
    if kind == "mode7":
        return "kOverworldMapGfx"
    raise ValueError(f"unknown decoded CHR pack kind: {kind}")


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


def _tile_entry_id(kind: str, asset: str, pack: int, tile: int, bpp: int) -> str:
    return f"{kind}:{asset}:pack{pack}:tile{tile}:{bpp}bpp"


def _tile_usage_key(
    kind: str,
    asset: str,
    pack: int,
    tile: int,
    bpp: int,
) -> tuple[str, str, int, int, int]:
    return kind, asset, pack, tile, bpp


def _source_tile_key_for_kind(
    kind: int,
    pack: int,
    tile: int,
) -> tuple[str, str, int, int, int] | None:
    if kind in (1, 5, 6):
        return "bg", "kBgGfx", pack, tile, 3
    if kind == 2:
        return "sprite", "kSprGfx", pack, tile, 3
    if kind == 4:
        return "bg3", "kBg3Gfx", pack, tile, 5
    if kind == 7:
        return "bg3_dynamic", "kBg3TextGfx", pack, tile, 5
    if kind == 8:
        return "link", "kLinkGfx", pack, tile, 3
    return None


def _source_tiles_paths(source_tiles_dir: Path) -> tuple[Path, Path]:
    return source_tiles_dir / "assets_by_source.json", source_tiles_dir / "assets_by_source.png"


def _read_source_tile_indices(source_tiles_dir: Path) -> list[tuple[tuple[str, str, int, int, int], bytes]]:
    from PIL import Image

    manifest_path, image_path = _source_tiles_paths(source_tiles_dir)
    if not manifest_path.is_file() or not image_path.is_file():
        return []

    data = json.loads(manifest_path.read_text())
    if data.get("format") not in {
        "zelda3_assets_by_source_v1",
        "zelda3_assets_by_source_v2_png",
    }:
        raise ValueError(f"{manifest_path}: unsupported source-tile format {data.get('format')!r}")
    with Image.open(image_path) as image:
        indexed = image.convert("P")
        width, height = indexed.size
        if width % 8 != 0 or height % 8 != 0:
            raise ValueError(f"{image_path}: source tile sheet size must be divisible by 8")
        columns = width // 8
        pixels = indexed.load()
        source_tiles: list[tuple[tuple[str, str, int, int, int], bytes]] = []
        for cell in data.get("cells", []):
            cell_id = int(cell["id"])
            x0 = (cell_id % columns) * 8
            y0 = (cell_id // columns) * 8
            if y0 + 8 > height:
                raise ValueError(f"{manifest_path}: cell {cell_id} is outside {image_path.name}")
            key = _source_tile_key_for_kind(
                int(cell["kind"]),
                int(cell["pack"]),
                int(cell["tile_off"]),
            )
            if key is None:
                continue
            indices = bytes(int(pixels[x0 + x, y0 + y]) for y in range(8) for x in range(8))
            source_tiles.append((key, indices))
    return source_tiles


def _read_mode7_source_tile_indices(
    asset_dir: Path,
) -> list[tuple[tuple[str, str, int, int, int], bytes]]:
    path = asset_dir / "assets" / "066-kOverworldMapGfx.bin"
    if not path.is_file():
        return []
    data = path.read_bytes()
    if len(data) % 64 != 0:
        raise ValueError(f"{path}: Mode 7 character source must be divisible by 64 bytes")
    tiles: list[tuple[tuple[str, str, int, int, int], bytes]] = []
    for tile_index in range(len(data) // 64):
        start = tile_index * 64
        tiles.append(
            (
                ("mode7", "kOverworldMapGfx", 0, tile_index, 8),
                data[start : start + 64],
            )
        )
    return tiles


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


def _indices_fit_palette(
    indices: bytes,
    colors: list[list[int]],
    palette_row: int,
    colors_per_row: int,
) -> bool:
    return palette_row >= 0 and palette_row * colors_per_row + max(indices, default=0) < len(colors)


def _source_ref(
    kind: str,
    asset: str,
    pack: int,
    tile: int,
    bpp: int,
    colors_per_row: int,
    preview_palette: str,
    preview_row: int,
    preview_source: str,
    usage: dict[str, object] | None,
    hflip: bool = False,
    vflip: bool = False,
) -> dict[str, object]:
    ref: dict[str, object] = {
        "source_kind": kind,
        "asset": asset,
        "pack": pack,
        "tile": tile,
        "bpp": bpp,
        "hflip": hflip,
        "vflip": vflip,
        "preview_palette": preview_palette,
        "preview_palette_row": preview_row,
        "preview_source": preview_source,
        "runtime_material": "palette_lut",
        "runtime_material_policy": classify_palette_policy(preview_palette),
        "runtime_colors_per_row": colors_per_row,
    }
    if usage is not None and "evidence_count" in usage:
        ref["palette_usage_evidence_count"] = int(usage["evidence_count"])
    return ref


def _preview_rank(preview_source: str, usage: dict[str, object] | None) -> tuple[int, int]:
    if preview_source != "palette_usage":
        return 0, 0
    evidence_count = int(usage.get("evidence_count", 0)) if usage is not None else 0
    return 1, evidence_count


def _transform_indices(indices: bytes, hflip: bool, vflip: bool) -> bytes:
    out = bytearray(64)
    for y in range(8):
        for x in range(8):
            src_x = 7 - x if hflip else x
            src_y = 7 - y if vflip else y
            out[y * 8 + x] = indices[src_y * 8 + src_x]
    return bytes(out)


def _normalize_indices_by_first_use(indices: bytes) -> tuple[bytes, list[int]]:
    index_map = {0: 0}
    next_index = 1
    normalized = bytearray()
    normalized_to_source = [0]
    for value in indices:
        if value not in index_map:
            index_map[value] = next_index
            normalized_to_source.append(value)
            next_index += 1
        normalized.append(index_map[value])
    return bytes(normalized), normalized_to_source


def _effect_rows_for_palette(
    palette_name: str,
    colors: list[list[int]],
    colors_per_row: int,
) -> list[dict[str, object]]:
    rows = len(colors) // colors_per_row
    return [
        {
            "id": f"{palette_name}:{colors_per_row}color:row{row}",
            "type": "palette_lut",
            "palette": palette_name,
            "palette_row": row,
            "colors_per_row": colors_per_row,
            "index_to_rgb": colors[row * colors_per_row : (row + 1) * colors_per_row],
            "dynamic_policy": classify_palette_policy(palette_name),
            "runtime": "shader_effect",
        }
        for row in range(rows)
    ]


def build_base_effect_atlas(
    asset_dir: Path,
    palette_names: list[str] | None = None,
    source_tiles_dir: Path | None = None,
) -> tuple[int, int, bytes, list[dict[str, object]], dict[str, object]]:
    palette_names = palette_names or _default_palette_names(asset_dir)
    if not palette_names:
        raise FileNotFoundError(asset_dir / "assets_src/palettes")

    palettes_dir = asset_dir / "assets_src/palettes"
    palettes = {
        name: read_palette_colors(palettes_dir / f"{name}.json")
        for name in palette_names
    }
    palette_usage = read_palette_usage_map(asset_dir)
    sprite_packs, bg_packs = chr_editable_sheets.read_decoded_chr_packs(asset_dir)
    variants: list[RgbaVariant] = []
    metadata: list[tuple[str, VariantKey, str, int, str, dict[str, object] | None]] = []
    seen_base_keys: set[tuple[str, str, int, int, int]] = set()
    for pack in [*sprite_packs, *bg_packs]:
        _rows, colors_per_row = _rows_for_bpp(pack.bpp)
        asset = _asset_name_for_kind(pack.kind)
        for tile_index, tile in enumerate(pack.tiles):
            base_key = _tile_usage_key(pack.kind, asset, pack.pack_index, tile_index, pack.bpp)
            preview_palette, preview_row, preview_source, usage = _preview_palette_for_tile(
                kind=pack.kind,
                asset=asset,
                pack=pack.pack_index,
                tile=tile_index,
                bpp=pack.bpp,
                colors_per_row=colors_per_row,
                available_palette_names=palette_names,
                palettes=palettes,
                palette_usage=palette_usage,
            )
            colors = palettes[preview_palette]
            key = VariantKey(
                pack.kind,
                asset,
                pack.pack_index,
                tile_index,
                pack.bpp,
                preview_palette,
                preview_row,
            )
            variants.append(
                RgbaVariant(
                    key,
                    rgba_tile_from_indices(tile, colors, preview_row, colors_per_row),
                )
            )
            metadata.append((
                _tile_entry_id(key.source_kind, key.asset, key.pack, key.tile, key.bpp),
                key,
                preview_palette,
                preview_row,
                preview_source,
                usage,
            ))
            seen_base_keys.add(base_key)

    if source_tiles_dir is not None:
        for (kind, asset, pack, tile, bpp), indices in _read_source_tile_indices(source_tiles_dir):
            base_key = _tile_usage_key(kind, asset, pack, tile, bpp)
            if base_key in seen_base_keys:
                continue
            _rows, colors_per_row = _rows_for_bpp(bpp)
            preview_palette, preview_row, preview_source, usage = _preview_palette_for_tile(
                kind=kind,
                asset=asset,
                pack=pack,
                tile=tile,
                bpp=bpp,
                colors_per_row=colors_per_row,
                available_palette_names=palette_names,
                palettes=palettes,
                palette_usage=palette_usage,
            )
            colors = palettes[preview_palette]
            if not _indices_fit_palette(indices, colors, preview_row, colors_per_row):
                continue
            key = VariantKey(
                kind,
                asset,
                pack,
                tile,
                bpp,
                preview_palette,
                preview_row,
            )
            variants.append(
                RgbaVariant(
                    key,
                    rgba_tile_from_indices(indices, colors, preview_row, colors_per_row),
                )
            )
            metadata.append((
                _tile_entry_id(key.source_kind, key.asset, key.pack, key.tile, key.bpp),
                key,
                preview_palette,
                preview_row,
                preview_source,
                usage,
            ))
            seen_base_keys.add(base_key)

    width, height, pixels, packed_entries = pack_rgba_variants(
        variants,
        columns=128 if source_tiles_dir is not None else 32,
    )
    entries = []
    for entry, (entry_id, key, preview_palette, preview_row, preview_source, usage) in zip(
        packed_entries, metadata
    ):
        json_entry = {
            "id": entry_id,
            "source_kind": key.source_kind,
            "asset": key.asset,
            "pack": key.pack,
            "tile": key.tile,
            "bpp": key.bpp,
            "preview_palette": preview_palette,
            "preview_palette_row": preview_row,
            "preview_source": preview_source,
            "rect": list(entry.rect),
            "sha1": entry.sha1,
            "duplicate_of": entry.duplicate_of,
        }
        if usage is not None and "evidence_count" in usage:
            json_entry["palette_usage_evidence_count"] = int(usage["evidence_count"])
        json_entry["dynamic_policy"] = classify_palette_policy(preview_palette)
        entries.append(json_entry)

    effects = _build_effect_table_from_palettes(palettes)
    return width, height, pixels, entries, effects


def build_tile_effect_table(
    asset_dir: Path,
    palette_names: list[str] | None = None,
) -> dict[str, object]:
    palette_names = palette_names or _default_palette_names(asset_dir)
    if not palette_names:
        raise FileNotFoundError(asset_dir / "assets_src/palettes")

    palettes_dir = asset_dir / "assets_src/palettes"
    palettes = {
        name: read_palette_colors(palettes_dir / f"{name}.json")
        for name in palette_names
    }
    return _build_effect_table_from_palettes(palettes)


def write_tile_effect_table(
    asset_dir: Path,
    out_dir: Path | None = None,
) -> list[Path]:
    destination = out_dir or asset_dir / "atlas"
    effects = build_tile_effect_table(asset_dir)
    destination.mkdir(parents=True, exist_ok=True)
    effects_path = destination / "tile_effects.json"
    effects_path.write_text(json.dumps(effects, indent=2, sort_keys=True) + "\n")
    return [effects_path]


def write_base_effect_atlas(
    asset_dir: Path,
    out_dir: Path | None = None,
    source_tiles_dir: Path | None = None,
) -> list[Path]:
    from PIL import Image

    destination = out_dir or asset_dir / "atlas"
    width, height, pixels, entries, effects = build_base_effect_atlas(
        asset_dir,
        source_tiles_dir=source_tiles_dir,
    )
    destination.mkdir(parents=True, exist_ok=True)
    png_path = destination / "base_tiles.png"
    json_path = destination / "base_tiles.json"
    effects_path = destination / "tile_effects.json"

    Image.frombytes("RGBA", (width, height), pixels).save(png_path)
    manifest = {
        "format": "zelda3_base_art_atlas_v1",
        "tile_width": 8,
        "tile_height": 8,
        "width": width,
        "height": height,
        "entry_count": len(entries),
        "entries": entries,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    effects_path.write_text(json.dumps(effects, indent=2, sort_keys=True) + "\n")
    return [png_path, json_path, effects_path]


def _pack_canonical_art_groups(
    groups: dict[str, dict[str, object]],
    palettes: dict[str, list[list[int]]],
) -> tuple[int, int, bytes, list[dict[str, object]]]:
    columns = 128
    art_groups = list(groups.values())
    rows = max(1, (len(art_groups) + columns - 1) // columns)
    width = columns * 8
    height = rows * 8
    pixels = bytearray(width * height * 4)
    arts: list[dict[str, object]] = []
    for index, group in enumerate(art_groups):
        x = (index % columns) * 8
        y = (index // columns) * 8
        bpp = int(group["bpp"])
        _rows, colors_per_row = _rows_for_bpp(bpp)
        preview_palette = str(group["preview_palette"])
        preview_row = int(group["preview_row"])
        tile_pixels = rgba_tile_from_indices(
            group["indices"],  # type: ignore[arg-type]
            palettes[preview_palette],
            preview_row,
            colors_per_row,
        )
        for row in range(8):
            dst = ((y + row) * width + x) * 4
            src = row * 8 * 4
            pixels[dst : dst + 8 * 4] = tile_pixels[src : src + 8 * 4]
        arts.append(
            {
                "art_id": f"art:{group['digest']}",
                "bpp": bpp,
                "indices_hex": bytes(group["indices"]).hex(),  # type: ignore[arg-type]
                "rect": [x, y, 8, 8],
                "sha1_indices": group["digest"],
                "preview_palette": preview_palette,
                "preview_palette_row": preview_row,
                "preview_source": group["preview_source"],
                "source_refs": group["source_refs"],
            }
        )
    return width, height, bytes(pixels), arts


def build_canonical_art_atlases(
    asset_dir: Path,
    palette_names: list[str] | None = None,
    source_tiles_dir: Path | None = None,
) -> tuple[
    tuple[int, int, bytes, list[dict[str, object]]],
    tuple[int, int, bytes, list[dict[str, object]]],
]:
    palette_names = palette_names or _default_palette_names(asset_dir)
    if not palette_names:
        raise FileNotFoundError(asset_dir / "assets_src/palettes")

    palettes_dir = asset_dir / "assets_src/palettes"
    palettes = {
        name: read_palette_colors(palettes_dir / f"{name}.json")
        for name in palette_names
    }
    palette_usage = read_palette_usage_map(asset_dir)
    sprite_packs, bg_packs = chr_editable_sheets.read_decoded_chr_packs(asset_dir)

    groups: dict[str, dict[str, object]] = {}
    dynamic_bg3_groups: dict[str, dict[str, object]] = {}
    seen_source_keys: set[tuple[str, str, int, int, int]] = set()

    def add_tile(kind: str, asset: str, pack: int, tile: int, bpp: int, indices: bytes) -> None:
        _rows, colors_per_row = _rows_for_bpp(bpp)
        preview_palette, preview_row, preview_source, usage = _preview_palette_for_tile(
            kind=kind,
            asset=asset,
            pack=pack,
            tile=tile,
            bpp=bpp,
            colors_per_row=colors_per_row,
            available_palette_names=palette_names,
            palettes=palettes,
            palette_usage=palette_usage,
        )
        colors = palettes[preview_palette]
        if not _indices_fit_palette(indices, colors, preview_row, colors_per_row):
            return
        rank = _preview_rank(preview_source, usage)

        if kind == "bg3_dynamic":
            digest = ""
            hflip = False
            vflip = False
            normalized_indices = b""
            index_map: list[int] = []
            for candidate_hflip, candidate_vflip in (
                (False, False),
                (True, False),
                (False, True),
                (True, True),
            ):
                transformed = _transform_indices(indices, candidate_hflip, candidate_vflip)
                candidate_indices, candidate_index_map = _normalize_indices_by_first_use(
                    transformed
                )
                candidate_digest = hashlib.sha1(
                    b"dynamic-bg3" + bytes([bpp]) + candidate_indices
                ).hexdigest()
                if not digest or candidate_digest in dynamic_bg3_groups:
                    digest = candidate_digest
                    hflip = candidate_hflip
                    vflip = candidate_vflip
                    normalized_indices = candidate_indices
                    index_map = candidate_index_map
                    if candidate_digest in dynamic_bg3_groups:
                        break
            ref = _source_ref(
                kind,
                asset,
                pack,
                tile,
                bpp,
                colors_per_row,
                preview_palette,
                preview_row,
                preview_source,
                usage,
                hflip=hflip,
                vflip=vflip,
            )
            ref["dynamic_shape_encoding"] = "first_use_index_normalized"
            ref["dynamic_index_map"] = index_map
            group = dynamic_bg3_groups.get(digest)
            if group is None:
                dynamic_bg3_groups[digest] = {
                    "digest": digest,
                    "bpp": bpp,
                    "indices": normalized_indices,
                    "preview_palette": preview_palette,
                    "preview_row": preview_row,
                    "preview_source": preview_source,
                    "preview_rank": rank,
                    "source_refs": [ref],
                }
                return
            group["source_refs"].append(ref)  # type: ignore[index]
            if rank > group["preview_rank"]:
                group["preview_palette"] = preview_palette
                group["preview_row"] = preview_row
                group["preview_source"] = preview_source
                group["preview_rank"] = rank
            return

        digest = hashlib.sha1(bytes([bpp]) + indices).hexdigest()
        hflip = False
        vflip = False
        for candidate_hflip, candidate_vflip in (
            (False, False),
            (True, False),
            (False, True),
            (True, True),
        ):
            transformed = _transform_indices(indices, candidate_hflip, candidate_vflip)
            transformed_digest = hashlib.sha1(bytes([bpp]) + transformed).hexdigest()
            if transformed_digest in groups:
                digest = transformed_digest
                hflip = candidate_hflip
                vflip = candidate_vflip
                break
        ref = _source_ref(
            kind,
            asset,
            pack,
            tile,
            bpp,
            colors_per_row,
            preview_palette,
            preview_row,
            preview_source,
            usage,
            hflip=hflip,
            vflip=vflip,
        )
        group = groups.get(digest)
        if group is None:
            groups[digest] = {
                "digest": digest,
                "bpp": bpp,
                "indices": indices,
                "preview_palette": preview_palette,
                "preview_row": preview_row,
                "preview_source": preview_source,
                "preview_rank": rank,
                "source_refs": [ref],
            }
            return
        group["source_refs"].append(ref)  # type: ignore[index]
        if rank > group["preview_rank"]:
            group["preview_palette"] = preview_palette
            group["preview_row"] = preview_row
            group["preview_source"] = preview_source
            group["preview_rank"] = rank

    for pack in [*sprite_packs, *bg_packs]:
        asset = _asset_name_for_kind(pack.kind)
        for tile_index, tile in enumerate(pack.tiles):
            source_key = _tile_usage_key(pack.kind, asset, pack.pack_index, tile_index, pack.bpp)
            add_tile(pack.kind, asset, pack.pack_index, tile_index, pack.bpp, tile)
            seen_source_keys.add(source_key)

    if source_tiles_dir is not None:
        for (kind, asset, pack, tile, bpp), indices in _read_source_tile_indices(source_tiles_dir):
            source_key = _tile_usage_key(kind, asset, pack, tile, bpp)
            if source_key in seen_source_keys:
                continue
            add_tile(kind, asset, pack, tile, bpp, indices)
            seen_source_keys.add(source_key)

    for (kind, asset, pack, tile, bpp), indices in _read_mode7_source_tile_indices(asset_dir):
        source_key = _tile_usage_key(kind, asset, pack, tile, bpp)
        if source_key in seen_source_keys:
            continue
        add_tile(kind, asset, pack, tile, bpp, indices)
        seen_source_keys.add(source_key)

    return (
        _pack_canonical_art_groups(groups, palettes),
        _pack_canonical_art_groups(dynamic_bg3_groups, palettes),
    )


def build_canonical_art_atlas(
    asset_dir: Path,
    palette_names: list[str] | None = None,
    source_tiles_dir: Path | None = None,
) -> tuple[int, int, bytes, list[dict[str, object]]]:
    main_atlas, _dynamic_bg3_atlas = build_canonical_art_atlases(
        asset_dir,
        palette_names=palette_names,
        source_tiles_dir=source_tiles_dir,
    )
    return main_atlas


def write_canonical_art_atlas(
    asset_dir: Path,
    out_dir: Path | None = None,
    source_tiles_dir: Path | None = None,
) -> list[Path]:
    from PIL import Image

    destination = out_dir or asset_dir / "atlas"
    (width, height, pixels, arts), (
        dynamic_width,
        dynamic_height,
        dynamic_pixels,
        dynamic_arts,
    ) = build_canonical_art_atlases(
        asset_dir,
        source_tiles_dir=source_tiles_dir,
    )
    destination.mkdir(parents=True, exist_ok=True)
    png_path = destination / "art_tiles.png"
    json_path = destination / "art_tiles.json"
    dynamic_png_path = destination / "dynamic_bg3_tiles.png"
    dynamic_json_path = destination / "dynamic_bg3_tiles.json"
    Image.frombytes("RGBA", (width, height), pixels).save(png_path)
    manifest = {
        "format": "zelda3_canonical_art_atlas_v1",
        "tile_width": 8,
        "tile_height": 8,
        "width": width,
        "height": height,
        "art_count": len(arts),
        "source_ref_count": sum(len(art["source_refs"]) for art in arts),
        "arts": arts,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    Image.frombytes("RGBA", (dynamic_width, dynamic_height), dynamic_pixels).save(
        dynamic_png_path
    )
    dynamic_manifest = {
        "format": "zelda3_dynamic_bg3_art_atlas_v1",
        "tile_width": 8,
        "tile_height": 8,
        "width": dynamic_width,
        "height": dynamic_height,
        "art_count": len(dynamic_arts),
        "source_ref_count": sum(len(art["source_refs"]) for art in dynamic_arts),
        "classification": "bg3_content_hash_text_like",
        "description": "Dynamic BG3/message/sign/content-hash tiles split from canonical editable art.",
        "arts": dynamic_arts,
    }
    dynamic_json_path.write_text(
        json.dumps(dynamic_manifest, indent=2, sort_keys=True) + "\n"
    )
    written = [png_path, json_path, dynamic_png_path, dynamic_json_path]
    written.extend(_write_dialogue_glyph_source_atlas(asset_dir, destination))
    written.extend(_write_dialogue_vwf_font_metadata(asset_dir, destination))
    written.extend(_write_dialogue_vwf_glyph_atlas(asset_dir, destination))
    written.extend(_write_dialogue_font_tile_atlas(asset_dir, destination))
    return written


def _find_index_in_memblk(data: bytes, index: int) -> bytes:
    if len(data) < 2:
        return b""
    end = len(data) - 2
    count = int.from_bytes(data[end : end + 2], "little")
    if count < 8192:
        if index > count or count * 2 > end:
            return b""
        left = (
            count * 2
            if index == 0
            else count * 2 + int.from_bytes(data[index * 2 - 2 : index * 2], "little")
        )
        right = (
            end
            if index == count
            else count * 2 + int.from_bytes(data[index * 2 : index * 2 + 2], "little")
        )
    else:
        count -= 8192
        if index > count or count * 4 > end:
            return b""
        left = (
            count * 4
            if index == 0
            else count * 4 + int.from_bytes(data[index * 4 - 4 : index * 4], "little")
        )
        right = (
            end
            if index == count
            else count * 4 + int.from_bytes(data[index * 4 : index * 4 + 4], "little")
        )
    if left > right or right > end:
        return b""
    return data[left:right]


def _dialogue_font_payloads(font_path: Path) -> tuple[bytes, bytes]:
    data = font_path.read_bytes()
    font_pack = _find_index_in_memblk(data, 0)
    if font_pack:
        font_bytes = _find_index_in_memblk(font_pack, 0)
        widths = _find_index_in_memblk(font_pack, 1)
        if len(font_bytes) >= 0x1000:
            return font_bytes[:0x1000], widths

    if len(data) < 0x1000:
        raise ValueError(f"{font_path}: dialogue font asset is smaller than 0x1000 bytes")
    return data[:0x1000], data[0x1000:]


def _dialogue_vwf_glyph_records(
    font_path: Path,
) -> tuple[bytes, bytes, list[dict[str, object]]]:
    font_bytes, widths = _dialogue_font_payloads(font_path)
    glyph_count = min(len(widths), 128)
    glyphs = []
    for code in range(glyph_count):
        width = int(widths[code])
        top_row_offset = (((code & 0x70) * 2) + (code & 0x0F)) * 16
        bottom_row_offset = top_row_offset + 16 * 16
        glyphs.append(
            {
                "code": code,
                "hex": f"0x{code:02x}",
                "width": width,
                "top_row_offset": top_row_offset,
                "bottom_row_offset": bottom_row_offset,
                "top_row_bytes": font_bytes[top_row_offset : top_row_offset + 16].hex(),
                "bottom_row_bytes": font_bytes[
                    bottom_row_offset : bottom_row_offset + 16
                ].hex(),
            }
        )
    return font_bytes, widths, glyphs


def _write_dialogue_vwf_font_metadata(asset_dir: Path, destination: Path) -> list[Path]:
    font_path = asset_dir / "assets/095-kDialogueFont.bin"
    if not font_path.is_file():
        return []
    font_bytes, widths, glyphs = _dialogue_vwf_glyph_records(font_path)

    json_path = destination / "dialogue_vwf_font.json"
    manifest = {
        "format": "zelda3_dialogue_vwf_font_v1",
        "source_asset": "assets/095-kDialogueFont.bin",
        "description": "Dialogue VWF font metadata using the same offsets and widths as VWF_RenderSingle.",
        "font_data_size": len(font_bytes),
        "width_table_size": len(widths),
        "glyph_count": len(glyphs),
        "runtime_offset_rule": "r10=((code&0x70)*2)+(code&0x0f); top=r10*16; bottom=top+0x100",
        "glyphs": glyphs,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return [json_path]


def _write_dialogue_vwf_glyph_atlas(asset_dir: Path, destination: Path) -> list[Path]:
    from PIL import Image

    font_path = asset_dir / "assets/095-kDialogueFont.bin"
    if not font_path.is_file():
        return []
    font_bytes, widths, glyphs = _dialogue_vwf_glyph_records(font_path)
    if not glyphs:
        return []

    cell_width = 16
    cell_height = 16
    columns = 16
    rows = max(1, (len(glyphs) + columns - 1) // columns)
    width = columns * cell_width
    height = rows * cell_height
    index_colors = {
        0: [0, 0, 0, 0],
        1: [248, 248, 248, 255],
        2: [88, 88, 88, 255],
        3: [184, 184, 184, 255],
    }
    pixels = bytearray(width * height * 4)
    for glyph_index, glyph in enumerate(glyphs):
        code = int(glyph["code"])
        glyph_indices = _render_dialogue_vwf_glyph_indices(
            font_bytes, code, int(glyph["width"])
        )
        x = (glyph_index % columns) * cell_width
        y = (glyph_index // columns) * cell_height
        glyph["rect"] = [x, y, cell_width, cell_height]
        glyph["indices_hex"] = glyph_indices.hex()
        for row in range(cell_height):
            for col in range(cell_width):
                color = index_colors[glyph_indices[row * cell_width + col]]
                dst = ((y + row) * width + x + col) * 4
                pixels[dst : dst + 4] = bytes(color)

    png_path = destination / "dialogue_vwf_glyphs.png"
    json_path = destination / "dialogue_vwf_glyphs.json"
    Image.frombytes("RGBA", (width, height), bytes(pixels)).save(png_path)
    manifest = {
        "format": "zelda3_dialogue_vwf_glyph_atlas_v1",
        "source_asset": "assets/095-kDialogueFont.bin",
        "description": "Editable RGBA source glyphs decoded from the dialogue VWF font using the same tile-byte writes as VWF_RenderSingle.",
        "encoding": "rgba_index_mask",
        "cell_width": cell_width,
        "cell_height": cell_height,
        "width": width,
        "height": height,
        "glyph_count": len(glyphs),
        "width_table_size": len(widths),
        "index_colors": index_colors,
        "glyphs": glyphs,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return [png_path, json_path]


def _decode_snes_2bpp_tile_indices(tile_bytes: bytes) -> bytes:
    if len(tile_bytes) != 16:
        raise ValueError(f"expected 16 bytes for one 2bpp tile, got {len(tile_bytes)}")
    indices = bytearray()
    for row in range(8):
        plane0 = tile_bytes[row * 2]
        plane1 = tile_bytes[row * 2 + 1]
        for x in range(8):
            bit = 7 - x
            indices.append(((plane0 >> bit) & 1) | (((plane1 >> bit) & 1) << 1))
    return bytes(indices)


def _write_dialogue_font_tile_atlas(asset_dir: Path, destination: Path) -> list[Path]:
    from PIL import Image

    font_path = asset_dir / "assets/095-kDialogueFont.bin"
    if not font_path.is_file():
        return []
    font_bytes, _widths = _dialogue_font_payloads(font_path)

    tile_width = 8
    tile_height = 8
    columns = 16
    tile_count = len(font_bytes) // 16
    rows = max(1, (tile_count + columns - 1) // columns)
    width = columns * tile_width
    height = rows * tile_height
    index_colors = {
        0: [0, 0, 0, 0],
        1: [248, 248, 248, 255],
        2: [88, 88, 88, 255],
        3: [184, 184, 184, 255],
    }
    pixels = bytearray(width * height * 4)
    tiles: list[dict[str, object]] = []
    for tile in range(tile_count):
        indices = _decode_snes_2bpp_tile_indices(font_bytes[tile * 16 : tile * 16 + 16])
        x = (tile % columns) * tile_width
        y = (tile // columns) * tile_height
        for row in range(tile_height):
            for col in range(tile_width):
                color = index_colors[indices[row * tile_width + col]]
                dst = ((y + row) * width + x + col) * 4
                pixels[dst : dst + 4] = bytes(color)
        tiles.append(
            {
                "id": f"kDialogueFont:tile{tile:02x}",
                "source_asset": "assets/095-kDialogueFont.bin",
                "source_kind": "dialogue_font",
                "source_pack": 0,
                "source_bpp": 2,
                "source_tile": tile,
                "rect": [x, y, tile_width, tile_height],
                "indices_hex": indices.hex(),
            }
        )

    png_path = destination / "dialogue_font_tiles.png"
    json_path = destination / "dialogue_font_tiles.json"
    Image.frombytes("RGBA", (width, height), bytes(pixels)).save(png_path)
    manifest = {
        "format": "zelda3_dialogue_font_tile_atlas_v1",
        "source_asset": "assets/095-kDialogueFont.bin",
        "description": "Editable RGBA source tiles for the raw dialogue font CHR copied to BG3 during ending credits.",
        "encoding": "snes_2bpp_tiles",
        "tile_width": tile_width,
        "tile_height": tile_height,
        "width": width,
        "height": height,
        "tile_count": tile_count,
        "index_colors": index_colors,
        "tiles": tiles,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return [png_path, json_path]


def _render_dialogue_vwf_glyph_indices(
    font_bytes: bytes, code: int, glyph_width: int
) -> bytes:
    masks = [0x80, 0x40, 0x20, 0x10, 8, 4, 2, 1]
    buffer = bytearray(0x150 + 32)

    def write_word(offset: int, value: int) -> None:
        if offset + 1 < len(buffer):
            buffer[offset] = value & 0xFF
            buffer[offset + 1] = (value >> 8) & 0xFF

    def render_half(r10: int, line_ptr: int) -> None:
        src = r10 * 16
        for i in range(0, 16, 2):
            if src + 1 >= len(font_bytes):
                return
            r4 = font_bytes[src] | (font_bytes[src + 1] << 8)
            src += 2
            y_base = line_ptr
            x = (y_base & 0xFF0) + i
            y = (y_base >> 1) & 7
            r3 = glyph_width
            while r3 != 0:
                if r4 & 0x0080:
                    buffer[x] ^= masks[y]
                else:
                    buffer[x] &= ~masks[y] & 0xFF
                if r4 & 0x8000:
                    buffer[x + 1] ^= masks[y]
                else:
                    buffer[x + 1] &= ~masks[y] & 0xFF
                r4 = (r4 & ~0x8080) << 1
                r3 -= 1
                y += 1
                if y == 8:
                    break
            x += 16
            if r4 != 0:
                write_word(x, r4)

    r10 = ((code & 0x70) * 2) + (code & 0x0F)
    render_half(r10, 0)
    render_half(r10 + 16, 0x150)

    indices = bytearray()
    for row in range(16):
        base = 0 if row < 8 else 0x150
        tile_row = row & 7
        for tile_col in range(2):
            tile_offset = base + tile_col * 16 + tile_row * 2
            plane0 = buffer[tile_offset]
            plane1 = buffer[tile_offset + 1]
            for x in range(8):
                bit = 7 - x
                indices.append(((plane0 >> bit) & 1) | (((plane1 >> bit) & 1) << 1))
    return bytes(indices)


def _write_dialogue_glyph_source_atlas(asset_dir: Path, destination: Path) -> list[Path]:
    from PIL import Image

    source_manifest_path = asset_dir / "assets_src/chr/1w-2d.json"
    source_png_path = asset_dir / "assets_src/chr/1w-2d.png"
    if not source_manifest_path.is_file() or not source_png_path.is_file():
        return []

    source_manifest = json.loads(source_manifest_path.read_text())
    blocks = [
        block
        for block in source_manifest.get("blocks", [])
        if str(block.get("block", "")).startswith("1w-2d.DAT")
        and int(block.get("source_bpp", 0)) == 2
        and str(block.get("source_kind", "")) == "sprite"
    ]
    if not blocks:
        return []

    with Image.open(source_png_path) as source_image:
        source_rgba = source_image.convert("RGBA")
        source_width, source_height = source_rgba.size
        layout = source_manifest.get("layout", {})
        tile_width = int(layout.get("tile_width", 8))
        tile_height = int(layout.get("tile_height", 8))
        source_columns = int(layout.get("columns", source_width // tile_width))
        if tile_width != 8 or tile_height != 8 or source_columns <= 0:
            raise ValueError(f"{source_manifest_path}: unsupported 1w-2d tile layout")

        tile_entries: list[dict[str, object]] = []
        tile_pixels: list[bytes] = []
        for block in blocks:
            tile_start = int(block["tile_start"])
            tile_count = int(block["tile_count"])
            for relative_tile in range(tile_count):
                source_tile = tile_start + relative_tile
                x = (source_tile % source_columns) * tile_width
                y = (source_tile // source_columns) * tile_height
                if x + tile_width > source_width or y + tile_height > source_height:
                    raise ValueError(
                        f"{source_manifest_path}: source tile {source_tile} is outside {source_png_path.name}"
                    )
                tile_pixels.append(
                    source_rgba.crop((x, y, x + tile_width, y + tile_height)).tobytes()
                )
                tile_entries.append(
                    {
                        "id": f"{block['block']}:tile{relative_tile}",
                        "source_sheet": "1w-2d",
                        "source_block": block["block"],
                        "source_kind": block["source_kind"],
                        "source_pack": int(block["source_pack"]),
                        "source_bpp": int(block["source_bpp"]),
                        "source_tile": source_tile,
                        "relative_tile": relative_tile,
                    }
                )

    columns = 16
    rows = max(1, (len(tile_pixels) + columns - 1) // columns)
    width = columns * 8
    height = rows * 8
    pixels = bytearray(width * height * 4)
    for index, tile in enumerate(tile_pixels):
        x = (index % columns) * 8
        y = (index // columns) * 8
        tile_entries[index]["rect"] = [x, y, 8, 8]
        for row in range(8):
            dst = ((y + row) * width + x) * 4
            src = row * 8 * 4
            pixels[dst : dst + 8 * 4] = tile[src : src + 8 * 4]

    png_path = destination / "dialogue_glyph_tiles.png"
    json_path = destination / "dialogue_glyph_tiles.json"
    Image.frombytes("RGBA", (width, height), bytes(pixels)).save(png_path)
    manifest = {
        "format": "zelda3_dialogue_glyph_source_atlas_v1",
        "source_sheet": "assets_src/chr/1w-2d.png",
        "description": "Proper source glyph/menu text blocks used as the target for replacing dynamic BG3 text chunks.",
        "tile_width": 8,
        "tile_height": 8,
        "width": width,
        "height": height,
        "tile_count": len(tile_entries),
        "blocks": blocks,
        "tiles": tile_entries,
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return [png_path, json_path]


def build_rom_variant_atlas(
    asset_dir: Path,
    palette_names: list[str] | None = None,
) -> tuple[int, int, bytes, list[AtlasEntry]]:
    palette_names = palette_names or _default_palette_names(asset_dir)
    if not palette_names:
        raise FileNotFoundError(asset_dir / "assets_src/palettes")

    palettes_dir = asset_dir / "assets_src/palettes"
    palettes = {
        name: read_palette_colors(palettes_dir / f"{name}.json")
        for name in palette_names
    }
    sprite_packs, bg_packs = chr_editable_sheets.read_decoded_chr_packs(asset_dir)
    variants: list[RgbaVariant] = []
    for pack in [*sprite_packs, *bg_packs]:
        rows, colors_per_row = _rows_for_bpp(pack.bpp)
        for palette_name, colors in palettes.items():
            for palette_row in range(rows):
                if (palette_row + 1) * colors_per_row > len(colors):
                    continue
                for tile_index, tile in enumerate(pack.tiles):
                    key = VariantKey(
                        pack.kind,
                        _asset_name_for_kind(pack.kind),
                        pack.pack_index,
                        tile_index,
                        pack.bpp,
                        palette_name,
                        palette_row,
                    )
                    variants.append(
                        RgbaVariant(
                            key,
                            rgba_tile_from_indices(tile, colors, palette_row, colors_per_row),
                        )
                    )
    return pack_rgba_variants(variants)


def _entry_to_json(entry: AtlasEntry) -> dict[str, object]:
    key = asdict(entry.key)
    return {
        "id": entry.id,
        **key,
        "rect": list(entry.rect),
        "sha1": entry.sha1,
        "duplicate_of": entry.duplicate_of,
        "dynamic_policy": entry.dynamic_policy,
    }


def write_rom_variant_atlas(asset_dir: Path, out_dir: Path | None = None) -> list[Path]:
    from PIL import Image

    destination = out_dir or asset_dir / "atlas"
    width, height, pixels, entries = build_rom_variant_atlas(asset_dir)
    destination.mkdir(parents=True, exist_ok=True)
    png_path = destination / "tile_variants.png"
    json_path = destination / "tile_variants.json"

    image = Image.frombytes("RGBA", (width, height), pixels)
    image.save(png_path)
    manifest = {
        "format": "zelda3_rgba_variant_atlas_v1",
        "tile_width": 8,
        "tile_height": 8,
        "width": width,
        "height": height,
        "entry_count": len(entries),
        "entries": [_entry_to_json(entry) for entry in entries],
    }
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return [png_path, json_path]
