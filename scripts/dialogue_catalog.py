#!/usr/bin/env python3
"""Extract Zelda 3 dialogue bytecode into a semantic JSON catalog.

The game still consumes the original dialogue asset. This catalog is a readable
sidecar that preserves the ROM bytecode and exposes a parsed IR for tooling.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


FORMAT_DIALOGUE_CATALOG = "zelda3_dialogue_catalog_v1"
FORMAT_DIALOGUE_SOURCE = "zelda3_dialogue_source_v1"
TEXT_COMMAND_START_US = 0x67
TEXT_DICT_BASE = 0x88

US_ALPHABET = [
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "!",
    "?",
    "-",
    ".",
    ",",
    "[...]",
    ">",
    "(",
    ")",
    "[Ankh]",
    "[Waves]",
    "[Snake]",
    "[LinkL]",
    "[LinkR]",
    '"',
    "[Up]",
    "[Down]",
    "[Left]",
    "[Right]",
    "'",
    "[1HeartL]",
    "[1HeartR]",
    "[2HeartL]",
    "[3HeartL]",
    "[3HeartR]",
    "[4HeartL]",
    "[4HeartR]",
    " ",
    "<",
    "[A]",
    "[B]",
    "[X]",
    "[Y]",
]

US_COMMAND_NAMES = [
    "next_pic",
    "choose",
    "item",
    "player_name",
    "window",
    "number",
    "position",
    "scroll_speed",
    "selection_change",
    "unused_crash",
    "choose3",
    "choose2",
    "scroll",
    "line1",
    "line2",
    "line3",
    "color",
    "wait",
    "sound",
    "speed",
    "unused_mark",
    "unused_mark2",
    "unused_clear",
    "wait_key",
    "end_message",
]

US_COMMAND_PARAM_LENGTHS = [
    0,
    0,
    0,
    0,
    1,
    1,
    1,
    1,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    1,
    1,
    1,
    1,
    0,
    0,
    0,
    0,
    0,
]

US_COMMAND_TO_INDEX = {name: index for index, name in enumerate(US_COMMAND_NAMES)}
SINGLE_GLYPH_TEXT_TO_CODE = {
    text: code for code, text in enumerate(US_ALPHABET) if len(text) == 1
}
BRACKET_GLYPH_TAG_TO_CODE = {
    text[1:-1]: code
    for code, text in enumerate(US_ALPHABET)
    if text.startswith("[") and text.endswith("]")
}


def _memblk_index_bounds(data: bytes, index: int) -> tuple[int, int] | None:
    if len(data) < 2:
        return None
    end = len(data) - 2
    marker = int.from_bytes(data[end : end + 2], "little")
    if marker < 8192:
        count = marker
        offset_size = 2
    else:
        count = marker - 8192
        offset_size = 4
    if index > count or count * offset_size > end:
        return None
    left = (
        count * offset_size
        if index == 0
        else count * offset_size
        + int.from_bytes(data[index * offset_size - offset_size : index * offset_size], "little")
    )
    right = (
        end
        if index == count
        else count * offset_size
        + int.from_bytes(data[index * offset_size : index * offset_size + offset_size], "little")
    )
    if left > right or right > end:
        return None
    return left, right


def find_index_in_memblk(data: bytes, index: int) -> bytes:
    bounds = _memblk_index_bounds(data, index)
    if bounds is None:
        return b""
    left, right = bounds
    return data[left:right]


def memblk_item_count(data: bytes) -> int:
    if len(data) < 2:
        return 0
    marker = int.from_bytes(data[-2:], "little")
    count = marker - 8192 if marker >= 8192 else marker
    if _memblk_index_bounds(data, count) is None:
        return 0
    return count + 1


def normalize_memblk(data: bytes, *, min_items: int = 1) -> bytes:
    for trim in range(0, min(4, len(data)) + 1):
        candidate = data[: len(data) - trim] if trim else data
        if memblk_item_count(candidate) >= min_items:
            return candidate
    return data


def hex_bytes(data: bytes) -> list[str]:
    return [f"{byte:02x}" for byte in data]


def pack_arrays(arrays: list[bytes]) -> bytes:
    if not arrays:
        raise ValueError("memblk array must contain at least one item")
    use_wide_offsets = sum(len(item) for item in arrays[:-1]) >= 65536
    offset_size = 4 if use_wide_offsets else 2
    offsets = []
    offset = 0
    for item in arrays[:-1]:
        offset += len(item)
        offsets.append(offset.to_bytes(offset_size, "little"))
    marker = 8192 + len(arrays) - 1 if use_wide_offsets else len(arrays) - 1
    return b"".join([*offsets, *arrays, marker.to_bytes(2, "little")])


def decode_language_map(language_asset: bytes) -> dict[str, Any]:
    language_pack = normalize_memblk(language_asset, min_items=2)
    language = find_index_in_memblk(language_pack, 0).decode("ascii", errors="replace")
    config = find_index_in_memblk(language_pack, 1)
    dialogue_pack = config[0] if len(config) >= 1 else 0
    font_pack = config[1] if len(config) >= 2 else 0
    flags = config[2] if len(config) >= 3 else 0
    return {
        "language": language,
        "dialogue_pack": dialogue_pack,
        "font_pack": font_pack,
        "flags": flags,
        "raw_config": hex_bytes(config),
    }


def dialogue_blocks(dialogue_asset: bytes, dialogue_pack: int = 0) -> tuple[bytes, bytes]:
    dialogue_packs = normalize_memblk(dialogue_asset, min_items=dialogue_pack + 1)
    dialogue_pack_bytes = find_index_in_memblk(dialogue_packs, dialogue_pack)
    if not dialogue_pack_bytes:
        raise ValueError(f"dialogue pack {dialogue_pack} is not present")
    dialogue_pack_bytes = normalize_memblk(dialogue_pack_bytes, min_items=2)
    dictionary = find_index_in_memblk(dialogue_pack_bytes, 0)
    dialogue = find_index_in_memblk(dialogue_pack_bytes, 1)
    if not dialogue:
        raise ValueError(f"dialogue pack {dialogue_pack} has no message block")
    return dictionary, dialogue


def dictionary_entries(dictionary: bytes) -> list[bytes]:
    count = memblk_item_count(dictionary)
    return [find_index_in_memblk(dictionary, index) for index in range(count)]


def dialogue_messages(dialogue: bytes) -> list[bytes]:
    count = memblk_item_count(dialogue)
    return [find_index_in_memblk(dialogue, index) for index in range(count)]


def expand_dictionary(raw: bytes, dictionary: bytes) -> tuple[bytes, list[dict[str, Any]]]:
    expanded = bytearray()
    expansions = []
    entries = dictionary_entries(dictionary)
    for source_offset, byte in enumerate(raw):
        if byte < TEXT_DICT_BASE:
            expanded.append(byte)
            continue
        entry_index = byte - TEXT_DICT_BASE
        entry = entries[entry_index] if entry_index < len(entries) else b""
        destination_offset = len(expanded)
        expanded.extend(entry)
        expansions.append(
            {
                "source_offset": source_offset,
                "dictionary_code": f"0x{byte:02x}",
                "dictionary_index": entry_index,
                "expanded_offset": destination_offset,
                "expanded_length": len(entry),
            }
        )
    return bytes(expanded), expansions


def glyph_text(code: int) -> str:
    if 0 <= code < len(US_ALPHABET):
        return US_ALPHABET[code]
    return f"[glyph 0x{code:02x}]"


def parse_us_ir(decoded: bytes) -> list[dict[str, Any]]:
    ops = []
    offset = 0
    while offset < len(decoded):
        byte = decoded[offset]
        raw = [byte]
        op_offset = offset
        offset += 1
        if byte < TEXT_COMMAND_START_US or byte >= 0x80:
            code = 26 if byte >= 0x80 else byte
            ops.append(
                {
                    "op": "glyph",
                    "offset": op_offset,
                    "raw": hex_bytes(bytes(raw)),
                    "code": code,
                    "text": glyph_text(code),
                }
            )
            continue

        command = byte - TEXT_COMMAND_START_US
        if command >= len(US_COMMAND_NAMES):
            ops.append(
                {
                    "op": "unknown_command",
                    "offset": op_offset,
                    "raw": hex_bytes(bytes(raw)),
                    "command": command,
                }
            )
            continue

        param_length = US_COMMAND_PARAM_LENGTHS[command]
        params = list(decoded[offset : offset + param_length])
        raw.extend(params)
        offset += len(params)
        entry: dict[str, Any] = {
            "op": US_COMMAND_NAMES[command],
            "offset": op_offset,
            "raw": hex_bytes(bytes(raw)),
            "command": command,
        }
        if param_length:
            entry["param"] = params[0] if params else None
        if entry["op"] in {"line1", "line2", "line3"}:
            entry["line"] = command - 12
        if len(params) < param_length:
            entry["truncated"] = True
        ops.append(entry)
        if command == 24:
            break
    return ops


def plain_text_lossy(ops: list[dict[str, Any]]) -> str:
    chunks = []
    for op in ops:
        kind = op["op"]
        if kind == "glyph":
            chunks.append(str(op["text"]))
        elif kind.startswith("line") or kind == "scroll":
            chunks.append("\n")
        elif kind == "end_message":
            break
        elif "param" in op:
            chunks.append(f"[{kind} {int(op['param']):02x}]")
        else:
            chunks.append(f"[{kind}]")
    return "".join(chunks)


def source_text_from_ops(ops: list[dict[str, Any]]) -> str:
    chunks = []
    for op in ops:
        kind = str(op["op"])
        if kind == "glyph":
            chunks.append(str(op["text"]))
        elif "param" in op:
            chunks.append(f"[{kind} {int(op['param']):02x}]")
        else:
            chunks.append(f"[{kind}]")
    return "".join(chunks)


def parse_byte_param(value: str) -> int:
    base = 16
    text = value
    if text.startswith("0x"):
        text = text[2:]
    elif any(ch not in "0123456789abcdefABCDEF" for ch in text):
        raise ValueError(f"invalid byte parameter {value!r}")
    parsed = int(text, base)
    if parsed < 0 or parsed > 0xFF:
        raise ValueError(f"byte parameter {value!r} is outside 00..ff")
    return parsed


def encode_source_tag(tag: str) -> bytes:
    if tag in BRACKET_GLYPH_TAG_TO_CODE:
        return bytes([BRACKET_GLYPH_TAG_TO_CODE[tag]])
    parts = tag.split()
    if not parts:
        raise ValueError("empty source tag")
    if parts[0] == "glyph" and len(parts) == 2:
        return bytes([parse_byte_param(parts[1])])
    command = US_COMMAND_TO_INDEX.get(parts[0])
    if command is None:
        raise ValueError(f"unknown dialogue source tag [{tag}]")
    param_length = US_COMMAND_PARAM_LENGTHS[command]
    if param_length == 0:
        if len(parts) != 1:
            raise ValueError(f"[{tag}] does not take a parameter")
        return bytes([TEXT_COMMAND_START_US + command])
    if len(parts) != 2:
        raise ValueError(f"[{tag}] needs one byte parameter")
    return bytes([TEXT_COMMAND_START_US + command, parse_byte_param(parts[1])])


def compile_source_text(source_text: str, *, append_end_message: bool = False) -> bytes:
    out = bytearray()
    offset = 0
    while offset < len(source_text):
        ch = source_text[offset]
        if ch == "[":
            end = source_text.find("]", offset + 1)
            if end < 0:
                raise ValueError("unterminated dialogue source tag")
            out.extend(encode_source_tag(source_text[offset + 1 : end]))
            offset = end + 1
            continue
        code = SINGLE_GLYPH_TEXT_TO_CODE.get(ch)
        if code is None:
            raise ValueError(f"cannot encode dialogue character {ch!r}")
        out.append(code)
        offset += 1
    if append_end_message and (not out or out[-1] != TEXT_COMMAND_START_US + 24):
        out.append(TEXT_COMMAND_START_US + 24)
    return bytes(out)


def message_catalog_entry(
    message_id: int,
    raw: bytes,
    dictionary: bytes,
    *,
    flags: int,
) -> dict[str, Any]:
    expanded, expansions = expand_dictionary(raw, dictionary)
    if flags & 1:
        ops = [
            {
                "op": "unsupported_language_mode",
                "offset": 0,
                "flags": flags,
                "raw": hex_bytes(expanded),
            }
        ]
    else:
        ops = parse_us_ir(expanded)
    return {
        "id": message_id,
        "raw_bytes": hex_bytes(raw),
        "expanded_bytes": hex_bytes(expanded),
        "dictionary_expansions": expansions,
        "preview_text": plain_text_lossy(ops),
        "source_text": source_text_from_ops(ops),
        "ops": ops,
    }


def catalog_from_assets(dialogue_asset: bytes, language_asset: bytes) -> dict[str, Any]:
    language = decode_language_map(language_asset)
    dictionary, dialogue = dialogue_blocks(
        dialogue_asset,
        dialogue_pack=int(language["dialogue_pack"]),
    )
    messages = dialogue_messages(dialogue)
    entries = [
        message_catalog_entry(
            index,
            raw,
            dictionary,
            flags=int(language["flags"]),
        )
        for index, raw in enumerate(messages)
    ]
    return {
        "format": FORMAT_DIALOGUE_CATALOG,
        "source_assets": [
            {
                "asset": "kDialogue",
                "index": 94,
                "sha1": hashlib.sha1(dialogue_asset).hexdigest(),
            },
            {
                "asset": "kDialogueMap",
                "index": 96,
                "sha1": hashlib.sha1(language_asset).hexdigest(),
            },
        ],
        "language": language,
        "dictionary": {
            "entry_count": len(dictionary_entries(dictionary)),
            "sha1": hashlib.sha1(dictionary).hexdigest(),
        },
        "message_count": len(entries),
        "messages": entries,
    }


def dialogue_source_from_catalog(catalog: dict[str, Any]) -> dict[str, Any]:
    messages = []
    for message in catalog["messages"]:
        expanded = bytes(int(byte, 16) for byte in message["expanded_bytes"])
        source_text = str(message["source_text"])
        compiled = compile_source_text(source_text)
        if compiled != expanded:
            raise ValueError(
                f"message {int(message['id'])} source_text does not compile back to expanded bytes"
            )
        messages.append(
            {
                "id": int(message["id"]),
                "source_text": source_text,
                "expanded_sha1": hashlib.sha1(expanded).hexdigest(),
            }
        )
    return {
        "format": FORMAT_DIALOGUE_SOURCE,
        "language": catalog["language"],
        "message_count": len(messages),
        "encoding": {
            "mode": "us",
            "control_tags": "commands use [name] or [name xx]; bracket glyphs keep their glyph names",
            "dictionary_strategy": "compile_uncompressed_messages",
        },
        "messages": messages,
    }


def dialogue_source_from_assets(dialogue_asset: bytes, language_asset: bytes) -> dict[str, Any]:
    return dialogue_source_from_catalog(catalog_from_assets(dialogue_asset, language_asset))


def asset_from_dialogue_source(source: dict[str, Any]) -> bytes:
    if source.get("format") != FORMAT_DIALOGUE_SOURCE:
        raise ValueError(f"dialogue source is not {FORMAT_DIALOGUE_SOURCE}")
    messages = source.get("messages")
    if not isinstance(messages, list):
        raise ValueError("dialogue source messages must be an array")
    compiled_messages = []
    for expected_id, message in enumerate(messages):
        if not isinstance(message, dict):
            raise ValueError(f"message {expected_id} must be an object")
        if message.get("id") != expected_id:
            raise ValueError(f"message {expected_id} id mismatch")
        source_text = message.get("source_text")
        if not isinstance(source_text, str):
            raise ValueError(f"message {expected_id} source_text must be a string")
        compiled_messages.append(compile_source_text(source_text))
    empty_dictionary = pack_arrays([b""])
    dialogue_pack = pack_arrays([empty_dictionary, pack_arrays(compiled_messages)])
    return pack_arrays([dialogue_pack])


def write_dialogue_catalog(
    path: Path,
    *,
    dialogue_asset: bytes,
    language_asset: bytes,
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    catalog = catalog_from_assets(dialogue_asset, language_asset)
    path.write_text(json.dumps(catalog, indent=2, sort_keys=True) + "\n", encoding="utf8")


def write_dialogue_source(
    path: Path,
    *,
    dialogue_asset: bytes,
    language_asset: bytes,
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    source = dialogue_source_from_assets(dialogue_asset, language_asset)
    path.write_text(json.dumps(source, indent=2, sort_keys=True) + "\n", encoding="utf8")


def write_dialogue_catalog_for_asset_dir(asset_dir: Path) -> Path:
    dialogue_path = asset_dir / "assets/094-kDialogue.bin"
    language_path = asset_dir / "assets/096-kDialogueMap.bin"
    if not dialogue_path.is_file():
        raise FileNotFoundError(dialogue_path)
    if not language_path.is_file():
        raise FileNotFoundError(language_path)
    output_path = asset_dir / "assets_src/dialogue/dialogue_catalog.json"
    write_dialogue_catalog(
        output_path,
        dialogue_asset=dialogue_path.read_bytes(),
        language_asset=language_path.read_bytes(),
    )
    return output_path


def write_dialogue_sources_for_asset_dir(asset_dir: Path) -> list[Path]:
    dialogue_path = asset_dir / "assets/094-kDialogue.bin"
    language_path = asset_dir / "assets/096-kDialogueMap.bin"
    if not dialogue_path.is_file():
        raise FileNotFoundError(dialogue_path)
    if not language_path.is_file():
        raise FileNotFoundError(language_path)

    dialogue_asset = dialogue_path.read_bytes()
    language_asset = language_path.read_bytes()
    catalog_path = asset_dir / "assets_src/dialogue/dialogue_catalog.json"
    source_path = asset_dir / "assets_src/dialogue/dialogue_source.json"
    write_dialogue_catalog(
        catalog_path,
        dialogue_asset=dialogue_asset,
        language_asset=language_asset,
    )
    write_dialogue_source(
        source_path,
        dialogue_asset=dialogue_asset,
        language_asset=language_asset,
    )
    return [catalog_path, source_path]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--asset-dir",
        type=Path,
        default=Path("generated/zelda3_assets"),
        help="Generated asset directory containing assets/094 and assets/096",
    )
    parser.add_argument(
        "--out",
        type=Path,
        help="Output catalog path; defaults to assets_src/dialogue/dialogue_catalog.json",
    )
    parser.add_argument(
        "--source-out",
        type=Path,
        help="Optional editable source output path",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    asset_dir = args.asset_dir
    if args.out is None:
        outputs = write_dialogue_sources_for_asset_dir(asset_dir)
    else:
        dialogue_asset = (asset_dir / "assets/094-kDialogue.bin").read_bytes()
        language_asset = (asset_dir / "assets/096-kDialogueMap.bin").read_bytes()
        write_dialogue_catalog(
            args.out,
            dialogue_asset=dialogue_asset,
            language_asset=language_asset,
        )
        outputs = [args.out]
        if args.source_out is not None:
            write_dialogue_source(
                args.source_out,
                dialogue_asset=dialogue_asset,
                language_asset=language_asset,
            )
            outputs.append(args.source_out)
    for output in outputs:
        print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
