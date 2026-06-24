#!/usr/bin/env python3
"""Convert SNES palette bytes between raw data and readable JSON."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


FORMAT_SNES_PALETTE = "zelda3_snes_palette_v1"


def palette_from_bytes(data: bytes, *, asset: str, asset_index: int) -> dict[str, Any]:
    if len(data) % 2 != 0:
        raise ValueError(f"{asset} has {len(data)} bytes, expected an even byte count")

    colors = []
    for index in range(len(data) // 2):
        word = int.from_bytes(data[index * 2 : index * 2 + 2], "little")
        colors.append(
            {
                "index": index,
                "snes_bgr15": f"0x{word:04x}",
                "rgb888": rgb888_hex(word),
            }
        )
    return {
        "format": FORMAT_SNES_PALETTE,
        "asset": asset,
        "asset_index": asset_index,
        "color_encoding": "SNES BGR555 little-endian",
        "canonical_sha1": hashlib.sha1(data).hexdigest(),
        "colors": colors,
    }


def bytes_from_palette(palette: dict[str, Any]) -> bytes:
    require_value(palette, "format", FORMAT_SNES_PALETTE)
    colors = require_list(palette, "colors")
    data = bytearray()
    for expected_index, color in enumerate(colors):
        if not isinstance(color, dict):
            raise ValueError(f"color {expected_index} is {type(color).__name__}, expected object")
        actual_index = color.get("index")
        if actual_index != expected_index:
            raise ValueError(f"color index is {actual_index!r}, expected {expected_index}")
        word = parse_snes_word(color.get("snes_bgr15"), expected_index)
        data.extend(word.to_bytes(2, "little"))
    return bytes(data)


def read_palette_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf8") as f:
        payload = json.load(f)
    if not isinstance(payload, dict):
        raise ValueError(f"{path} root is {type(payload).__name__}, expected object")
    return payload


def write_palette_json(path: Path, palette: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(palette, indent=2, sort_keys=True) + "\n", encoding="utf8")


def rgb888_hex(word: int) -> str:
    red = scale_5bit_to_8bit(word & 0x1F)
    green = scale_5bit_to_8bit((word >> 5) & 0x1F)
    blue = scale_5bit_to_8bit((word >> 10) & 0x1F)
    return f"#{red:02x}{green:02x}{blue:02x}"


def scale_5bit_to_8bit(value: int) -> int:
    return (value << 3) | (value >> 2)


def parse_snes_word(value: Any, index: int) -> int:
    if not isinstance(value, str):
        raise ValueError(f"color {index} is {type(value).__name__}, expected hex string")
    try:
        word = int(value, 16)
    except ValueError as exc:
        raise ValueError(f"color {index} is {value!r}, expected hex string") from exc
    if word < 0 or word > 0x7FFF:
        raise ValueError(f"color {index} is 0x{word:04x}, expected 0x0000..0x7fff")
    return word


def require_value(palette: dict[str, Any], key: str, expected: Any) -> None:
    actual = palette.get(key)
    if actual != expected:
        raise ValueError(f"{key} is {actual!r}, expected {expected!r}")


def require_list(palette: dict[str, Any], key: str) -> list[Any]:
    value = palette.get(key)
    if not isinstance(value, list):
        raise ValueError(f"{key} is {type(value).__name__}, expected list")
    return value
