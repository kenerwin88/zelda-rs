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
    raise ValueError(f"unsupported SNES tile bit depth: {bpp}")


def _default_preview_palette(
    kind: str,
    available_palette_names: list[str],
) -> tuple[str, int]:
    preferred = {
        "sprite": "palette_main_spr",
        "bg": "palette_dung_bg_main",
    }.get(kind)
    if preferred in available_palette_names:
        return preferred, 0
    if "palette_overworld_bg_main" in available_palette_names and kind == "bg":
        return "palette_overworld_bg_main", 0
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
    metadata: list[tuple[VariantKey, str, int, str, dict[str, object] | None]] = []
    for pack in [*sprite_packs, *bg_packs]:
        _rows, colors_per_row = _rows_for_bpp(pack.bpp)
        asset = _asset_name_for_kind(pack.kind)
        for tile_index, tile in enumerate(pack.tiles):
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
            metadata.append((key, preview_palette, preview_row, preview_source, usage))

    width, height, pixels, packed_entries = pack_rgba_variants(variants)
    entries = []
    for entry, (key, preview_palette, preview_row, preview_source, usage) in zip(
        packed_entries, metadata
    ):
        json_entry = {
            "id": _tile_entry_id(key.source_kind, key.asset, key.pack, key.tile, key.bpp),
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
        entries.append(json_entry)

    effects_by_id: dict[str, dict[str, object]] = {}
    for name, colors in palettes.items():
        for colors_per_row in (4, 8, 16):
            for effect in _effect_rows_for_palette(name, colors, colors_per_row):
                effects_by_id.setdefault(str(effect["id"]), effect)
    effects = {
        "format": "zelda3_tile_effect_table_v1",
        "strategy": "base_art_plus_shader_effects",
        "effects": list(effects_by_id.values()),
    }
    return width, height, pixels, entries, effects


def write_base_effect_atlas(asset_dir: Path, out_dir: Path | None = None) -> list[Path]:
    from PIL import Image

    destination = out_dir or asset_dir / "atlas"
    width, height, pixels, entries, effects = build_base_effect_atlas(asset_dir)
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
