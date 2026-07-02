#!/usr/bin/env python3
"""Build ROM-traceable RGBA tile variants for Zelda 3 graphics."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib


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


def variant_id(key: VariantKey) -> str:
    return (
        f"{key.source_kind}:{key.asset}:pack{key.pack}:tile{key.tile}:"
        f"{key.bpp}bpp:{key.palette}:row{key.palette_row}"
    )


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
        if digest in sha_to_rect:
            original_id, rect = sha_to_rect[digest]
            entries.append(AtlasEntry(entry_id, variant.key, rect, digest, original_id))
            continue

        unique_index = len(unique_pixels)
        x = (unique_index % columns) * 8
        y = (unique_index // columns) * 8
        rect = (x, y, 8, 8)
        unique_pixels.append(variant.pixels)
        sha_to_rect[digest] = (entry_id, rect)
        entries.append(AtlasEntry(entry_id, variant.key, rect, digest, None))

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
