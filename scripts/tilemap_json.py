#!/usr/bin/env python3
"""Convert simple Zelda 3 tilemap assets between bytes and readable JSON."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


FORMAT_BYTE_TILEMAP = "zelda3_byte_tilemap_v1"
FORMAT_BYTE_STREAM_TILEMAP = "zelda3_byte_stream_tilemap_v1"


def tilemap_from_bytes(
    data: bytes,
    *,
    asset: str,
    asset_index: int,
    width: int,
    height: int,
) -> dict[str, Any]:
    expected_size = width * height
    if len(data) != expected_size:
        raise ValueError(f"{asset} is {len(data)} bytes, expected {expected_size}")

    rows = [
        list(data[row_start : row_start + width])
        for row_start in range(0, expected_size, width)
    ]
    return {
        "format": FORMAT_BYTE_TILEMAP,
        "asset": asset,
        "asset_index": asset_index,
        "width": width,
        "height": height,
        "tile_value": "raw 8-bit tile id",
        "canonical_sha1": hashlib.sha1(data).hexdigest(),
        "rows": rows,
    }


def tilemap_stream_from_bytes(
    data: bytes,
    *,
    asset: str,
    asset_index: int,
) -> dict[str, Any]:
    return {
        "format": FORMAT_BYTE_STREAM_TILEMAP,
        "asset": asset,
        "asset_index": asset_index,
        "tile_value": "raw variable-length tilemap byte stream",
        "canonical_sha1": hashlib.sha1(data).hexdigest(),
        "values": list(data),
    }


def bytes_from_tilemap(tilemap: dict[str, Any]) -> bytes:
    tilemap_format = tilemap.get("format")
    if tilemap_format == FORMAT_BYTE_STREAM_TILEMAP:
        return bytes_from_tilemap_stream(tilemap)
    require_value(tilemap, "format", FORMAT_BYTE_TILEMAP)
    width = require_int(tilemap, "width")
    height = require_int(tilemap, "height")
    rows = require_list(tilemap, "rows")
    if len(rows) != height:
        raise ValueError(f"tilemap has {len(rows)} rows, expected {height}")

    data = bytearray()
    for y, row in enumerate(rows):
        if not isinstance(row, list):
            raise ValueError(f"row {y} is {type(row).__name__}, expected list")
        if len(row) != width:
            raise ValueError(f"row {y} has {len(row)} entries, expected {width}")
        for x, value in enumerate(row):
            if not isinstance(value, int):
                raise ValueError(
                    f"row {y} column {x} is {type(value).__name__}, expected int"
                )
            if value < 0 or value > 0xFF:
                raise ValueError(f"row {y} column {x} is {value}, expected 0..255")
            data.append(value)
    return bytes(data)


def bytes_from_tilemap_stream(tilemap: dict[str, Any]) -> bytes:
    values = require_list(tilemap, "values")
    data = bytearray()
    for index, value in enumerate(flatten_stream_values(values)):
        append_stream_value(data, index, value)
    return bytes(data)


def flatten_stream_values(values: list[Any]) -> list[Any]:
    flattened = []
    for value in values:
        if isinstance(value, list):
            flattened.extend(value)
        else:
            flattened.append(value)
    return flattened


def append_stream_value(data: bytearray, index: int, value: Any) -> None:
    if not isinstance(value, int):
        raise ValueError(f"value {index} is {type(value).__name__}, expected int")
    if value < 0 or value > 0xFF:
        raise ValueError(f"value {index} is {value}, expected 0..255")
    data.append(value)


def read_tilemap_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf8") as f:
        payload = json.load(f)
    if not isinstance(payload, dict):
        raise ValueError(f"{path} root is {type(payload).__name__}, expected object")
    return payload


def write_tilemap_json(path: Path, tilemap: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(format_tilemap_json(tilemap), encoding="utf8")


def format_tilemap_json(tilemap: dict[str, Any]) -> str:
    if tilemap.get("format") == FORMAT_BYTE_STREAM_TILEMAP:
        return format_stream_tilemap_json(tilemap)

    rows = require_list(tilemap, "rows")
    header = {key: value for key, value in tilemap.items() if key != "rows"}
    lines = ["{"]
    for key in sorted(header):
        value = json.dumps(header[key], sort_keys=True)
        lines.append(f'  "{key}": {value},')
    lines.append('  "rows": [')
    for index, row in enumerate(rows):
        suffix = "," if index + 1 < len(rows) else ""
        lines.append(f"    {json.dumps(row)}{suffix}")
    lines.append("  ]")
    lines.append("}")
    return "\n".join(lines) + "\n"


def format_stream_tilemap_json(tilemap: dict[str, Any]) -> str:
    values = require_list(tilemap, "values")
    header = {key: value for key, value in tilemap.items() if key != "values"}
    lines = ["{"]
    for key in sorted(header):
        value = json.dumps(header[key], sort_keys=True)
        lines.append(f'  "{key}": {value},')
    lines.append('  "values": [')
    for start in range(0, len(values), 32):
        chunk = values[start : start + 32]
        suffix = "," if start + 32 < len(values) else ""
        lines.append(f"    {json.dumps(chunk)}{suffix}")
    lines.append("  ]")
    lines.append("}")
    return "\n".join(lines) + "\n"


def require_value(tilemap: dict[str, Any], key: str, expected: Any) -> None:
    actual = tilemap.get(key)
    if actual != expected:
        raise ValueError(f"{key} is {actual!r}, expected {expected!r}")


def require_int(tilemap: dict[str, Any], key: str) -> int:
    value = tilemap.get(key)
    if not isinstance(value, int):
        raise ValueError(f"{key} is {type(value).__name__}, expected int")
    if value <= 0:
        raise ValueError(f"{key} is {value}, expected positive int")
    return value


def require_list(tilemap: dict[str, Any], key: str) -> list[Any]:
    value = tilemap.get(key)
    if not isinstance(value, list):
        raise ValueError(f"{key} is {type(value).__name__}, expected list")
    return value


def export_tilemap(args: argparse.Namespace) -> None:
    data = args.input_bin.read_bytes()
    tilemap = tilemap_from_bytes(
        data,
        asset=args.asset,
        asset_index=args.asset_index,
        width=args.width,
        height=args.height,
    )
    write_tilemap_json(args.output_json, tilemap)
    print(f"wrote {args.output_json}")


def verify_tilemap(args: argparse.Namespace) -> None:
    tilemap = read_tilemap_json(args.input_json)
    packed = bytes_from_tilemap(tilemap)
    canonical = args.canonical_bin.read_bytes()
    if packed != canonical:
        packed_sha1 = hashlib.sha1(packed).hexdigest()
        canonical_sha1 = hashlib.sha1(canonical).hexdigest()
        raise SystemExit(
            "tilemap JSON does not match canonical bytes: "
            f"{args.input_json} sha1={packed_sha1} "
            f"{args.canonical_bin} sha1={canonical_sha1}"
        )
    print(f"verified {args.input_json} matches {args.canonical_bin}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    export_parser = subparsers.add_parser("export", help="write JSON from a binary tilemap")
    export_parser.add_argument("--input-bin", required=True, type=Path)
    export_parser.add_argument("--output-json", required=True, type=Path)
    export_parser.add_argument("--asset", required=True)
    export_parser.add_argument("--asset-index", required=True, type=int)
    export_parser.add_argument("--width", required=True, type=int)
    export_parser.add_argument("--height", required=True, type=int)
    export_parser.set_defaults(func=export_tilemap)

    verify_parser = subparsers.add_parser("verify", help="compare JSON against canonical bytes")
    verify_parser.add_argument("--input-json", required=True, type=Path)
    verify_parser.add_argument("--canonical-bin", required=True, type=Path)
    verify_parser.set_defaults(func=verify_tilemap)

    return parser.parse_args()


def main() -> None:
    args = parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
