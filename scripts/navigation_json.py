#!/usr/bin/env python3
"""Convert grouped navigation tables between raw asset bytes and JSON."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


FORMAT_DUNGEON_ENTRANCES = "zelda3_dungeon_entrances_v1"
FORMAT_STARTING_POINTS = "zelda3_starting_points_v1"
FORMAT_OVERWORLD_EXITS = "zelda3_overworld_exits_v1"
FORMAT_SPECIAL_EXITS = "zelda3_special_exits_v1"


ENTRANCE_FIELDS = [
    ("kEntranceData_rooms", "room", "u16", 1),
    ("kEntranceData_relativeCoords", "relative_coords", "u8", 8),
    ("kEntranceData_scrollX", "scroll_x", "u16", 1),
    ("kEntranceData_scrollY", "scroll_y", "u16", 1),
    ("kEntranceData_playerX", "player_x", "u16", 1),
    ("kEntranceData_playerY", "player_y", "u16", 1),
    ("kEntranceData_cameraX", "camera_x", "u16", 1),
    ("kEntranceData_cameraY", "camera_y", "u16", 1),
    ("kEntranceData_blockset", "blockset", "u8", 1),
    ("kEntranceData_floor", "floor", "i8", 1),
    ("kEntranceData_palace", "palace", "i8", 1),
    ("kEntranceData_doorwayOrientation", "doorway_orientation", "u8", 1),
    ("kEntranceData_startingBg", "starting_bg", "u8", 1),
    ("kEntranceData_quadrant1", "quadrant1", "u8", 1),
    ("kEntranceData_quadrant2", "quadrant2", "u8", 1),
    ("kEntranceData_doorSettings", "door_settings", "u16", 1),
    ("kEntranceData_musicTrack", "music_track", "u8", 1),
]

STARTING_POINT_FIELDS = [
    ("kStartingPoint_rooms", "room", "u16", 1),
    ("kStartingPoint_relativeCoords", "relative_coords", "u8", 8),
    ("kStartingPoint_scrollX", "scroll_x", "u16", 1),
    ("kStartingPoint_scrollY", "scroll_y", "u16", 1),
    ("kStartingPoint_playerX", "player_x", "u16", 1),
    ("kStartingPoint_playerY", "player_y", "u16", 1),
    ("kStartingPoint_cameraX", "camera_x", "u16", 1),
    ("kStartingPoint_cameraY", "camera_y", "u16", 1),
    ("kStartingPoint_blockset", "blockset", "u8", 1),
    ("kStartingPoint_floor", "floor", "i8", 1),
    ("kStartingPoint_palace", "palace", "i8", 1),
    ("kStartingPoint_doorwayOrientation", "doorway_orientation", "u8", 1),
    ("kStartingPoint_startingBg", "starting_bg", "u8", 1),
    ("kStartingPoint_quadrant1", "quadrant1", "u8", 1),
    ("kStartingPoint_quadrant2", "quadrant2", "u8", 1),
    ("kStartingPoint_doorSettings", "door_settings", "u16", 1),
    ("kStartingPoint_entrance", "entrance", "u8", 1),
    ("kStartingPoint_musicTrack", "music_track", "u8", 1),
]

EXIT_FIELDS = [
    ("kExitData_ScreenIndex", "screen_index", "u8", 1),
    ("kExitDataRooms", "room", "u16", 1),
    ("kExitData_Map16LoadSrcOff", "map16_load_src_off", "u16", 1),
    ("kExitData_ScrollX", "scroll_x", "u16", 1),
    ("kExitData_ScrollY", "scroll_y", "u16", 1),
    ("kExitData_XCoord", "x_coord", "u16", 1),
    ("kExitData_YCoord", "y_coord", "u16", 1),
    ("kExitData_CameraXScroll", "camera_x_scroll", "u16", 1),
    ("kExitData_CameraYScroll", "camera_y_scroll", "u16", 1),
    ("kExitData_NormalDoor", "normal_door", "u16", 1),
    ("kExitData_FancyDoor", "fancy_door", "u16", 1),
    ("kExitData_Unk1", "unk1", "i8", 1),
    ("kExitData_Unk3", "unk3", "i8", 1),
]

SPECIAL_EXIT_FIELDS = [
    ("kSpExit_Top", "top", "u16", 1),
    ("kSpExit_Bottom", "bottom", "u16", 1),
    ("kSpExit_Left", "left", "u16", 1),
    ("kSpExit_Right", "right", "u16", 1),
    ("kSpExit_Tab4", "tab4", "i16", 1),
    ("kSpExit_Tab5", "tab5", "i16", 1),
    ("kSpExit_Tab6", "tab6", "i16", 1),
    ("kSpExit_Tab7", "tab7", "i16", 1),
    ("kSpExit_LeftEdgeOfMap", "left_edge_of_map", "u16", 1),
    ("kSpExit_Dir", "dir", "u8", 1),
    ("kSpExit_SprGfx", "spr_gfx", "u8", 1),
    ("kSpExit_AuxGfx", "aux_gfx", "u8", 1),
    ("kSpExit_PalBg", "pal_bg", "u8", 1),
    ("kSpExit_PalSpr", "pal_spr", "u8", 1),
]

FIELDS_BY_FORMAT = {
    FORMAT_DUNGEON_ENTRANCES: ENTRANCE_FIELDS,
    FORMAT_STARTING_POINTS: STARTING_POINT_FIELDS,
    FORMAT_OVERWORLD_EXITS: EXIT_FIELDS,
    FORMAT_SPECIAL_EXITS: SPECIAL_EXIT_FIELDS,
}


def entrance_source_from_assets(
    assets: dict[str, bytes], *, asset: str, asset_index_range: list[int]
) -> dict[str, Any]:
    return source_from_assets(
        assets,
        source_format=FORMAT_DUNGEON_ENTRANCES,
        asset=asset,
        asset_index_range=asset_index_range,
        fields=ENTRANCE_FIELDS,
    )


def starting_point_source_from_assets(
    assets: dict[str, bytes], *, asset_index_range: list[int]
) -> dict[str, Any]:
    return source_from_assets(
        assets,
        source_format=FORMAT_STARTING_POINTS,
        asset="kStartingPoint",
        asset_index_range=asset_index_range,
        fields=STARTING_POINT_FIELDS,
    )


def exit_source_from_assets(
    assets: dict[str, bytes], *, asset_index_range: list[int]
) -> dict[str, Any]:
    return source_from_assets(
        assets,
        source_format=FORMAT_OVERWORLD_EXITS,
        asset="kExitData",
        asset_index_range=asset_index_range,
        fields=EXIT_FIELDS,
    )


def special_exit_source_from_assets(
    assets: dict[str, bytes], *, asset_index_range: list[int]
) -> dict[str, Any]:
    return source_from_assets(
        assets,
        source_format=FORMAT_SPECIAL_EXITS,
        asset="kSpExit",
        asset_index_range=asset_index_range,
        fields=SPECIAL_EXIT_FIELDS,
    )


def source_from_assets(
    assets: dict[str, bytes],
    *,
    source_format: str,
    asset: str,
    asset_index_range: list[int],
    fields: list[tuple[str, str, str, int]],
) -> dict[str, Any]:
    record_count = determine_record_count(assets, fields)
    records = []
    for index in range(record_count):
        record: dict[str, Any] = {"index": index}
        for asset_name, field_name, value_type, values_per_record in fields:
            values = read_values(assets[asset_name], value_type)
            start = index * values_per_record
            field_values = values[start : start + values_per_record]
            record[field_name] = (
                field_values[0] if values_per_record == 1 else field_values
            )
        records.append(record)

    return {
        "format": source_format,
        "asset": asset,
        "asset_index_range": asset_index_range,
        "field_encoding": "parallel legacy asset arrays; numeric values pack little-endian",
        "canonical_sha1": canonical_group_sha1(assets, fields),
        "records": records,
    }


def determine_record_count(
    assets: dict[str, bytes], fields: list[tuple[str, str, str, int]]
) -> int:
    counts = {}
    for asset_name, _field_name, value_type, values_per_record in fields:
        if asset_name not in assets:
            continue
        values = read_values(assets[asset_name], value_type)
        if len(values) % values_per_record != 0:
            raise ValueError(f"{asset_name} has a partial record")
        counts[asset_name] = len(values) // values_per_record
    if len(counts) != len(fields):
        missing = sorted({asset_name for asset_name, *_ in fields} - set(counts))
        raise ValueError(f"missing assets: {', '.join(missing)}")
    unique_counts = set(counts.values())
    if len(unique_counts) != 1:
        raise ValueError(f"parallel table record count mismatch: {counts}")
    return unique_counts.pop()


def bytes_for_asset(source: dict[str, Any], asset_name: str) -> bytes:
    source_format = source.get("format")
    fields = FIELDS_BY_FORMAT.get(source_format)
    if fields is None:
        raise ValueError(f"unsupported navigation source format: {source_format!r}")
    field = next((field for field in fields if field[0] == asset_name), None)
    if field is None:
        raise ValueError(f"{asset_name} is not part of {source_format}")
    _asset_name, field_name, value_type, values_per_record = field

    records = source.get("records")
    if not isinstance(records, list):
        raise ValueError("records is not a list")

    values = []
    for expected_index, record in enumerate(records):
        if not isinstance(record, dict):
            raise ValueError(f"record {expected_index} is not an object")
        actual_index = record.get("index")
        if actual_index != expected_index:
            raise ValueError(f"record index is {actual_index!r}, expected {expected_index}")
        value = record.get(field_name)
        if values_per_record == 1:
            values.append(value)
        else:
            if not isinstance(value, list) or len(value) != values_per_record:
                raise ValueError(
                    f"record {expected_index} {field_name} must have "
                    f"{values_per_record} values"
                )
            values.extend(value)
    return write_values(values, value_type)


def read_values(data: bytes, value_type: str) -> list[int]:
    width = value_width(value_type)
    if len(data) % width != 0:
        raise ValueError(f"{value_type} payload has a partial value")
    return [
        int.from_bytes(data[start : start + width], "little", signed=is_signed(value_type))
        for start in range(0, len(data), width)
    ]


def write_values(values: list[Any], value_type: str) -> bytes:
    width = value_width(value_type)
    lower, upper = value_range(value_type)
    data = bytearray()
    for index, value in enumerate(values):
        if not isinstance(value, int):
            raise ValueError(f"value {index} is {type(value).__name__}, expected int")
        if value < lower or value > upper:
            raise ValueError(f"value {index} is {value}, expected {lower}..{upper}")
        data.extend(value.to_bytes(width, "little", signed=is_signed(value_type)))
    return bytes(data)


def value_width(value_type: str) -> int:
    if value_type in ("u8", "i8"):
        return 1
    if value_type in ("u16", "i16"):
        return 2
    raise ValueError(f"unsupported value type: {value_type}")


def value_range(value_type: str) -> tuple[int, int]:
    if value_type == "u8":
        return 0, 0xFF
    if value_type == "i8":
        return -0x80, 0x7F
    if value_type == "u16":
        return 0, 0xFFFF
    if value_type == "i16":
        return -0x8000, 0x7FFF
    raise ValueError(f"unsupported value type: {value_type}")


def is_signed(value_type: str) -> bool:
    return value_type.startswith("i")


def canonical_group_sha1(
    assets: dict[str, bytes], fields: list[tuple[str, str, str, int]]
) -> str:
    digest = hashlib.sha1()
    for asset_name, *_ in fields:
        digest.update(asset_name.encode("ascii"))
        digest.update(b"\0")
        digest.update(assets[asset_name])
        digest.update(b"\0")
    return digest.hexdigest()


def read_navigation_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf8") as f:
        payload = json.load(f)
    if not isinstance(payload, dict):
        raise ValueError(f"{path} root is {type(payload).__name__}, expected object")
    return payload


def write_navigation_json(path: Path, source: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(format_navigation_json(source), encoding="utf8")


def format_navigation_json(source: dict[str, Any]) -> str:
    records = source.get("records")
    if not isinstance(records, list):
        raise ValueError("records is not a list")
    header = {key: value for key, value in source.items() if key != "records"}
    lines = ["{"]
    for key in sorted(header):
        value = json.dumps(header[key], sort_keys=True)
        lines.append(f'  "{key}": {value},')
    lines.append('  "records": [')
    for index, record in enumerate(records):
        suffix = "," if index + 1 < len(records) else ""
        lines.append(f"    {json.dumps(record, sort_keys=True)}{suffix}")
    lines.append("  ]")
    lines.append("}")
    return "\n".join(lines) + "\n"
