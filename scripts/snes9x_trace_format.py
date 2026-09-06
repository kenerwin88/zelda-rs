#!/usr/bin/env python3
"""Decode and encode the pinned Snes9x trace core's Z3TRACE1 binary records.

The instrumented core writes an 8-byte magic (``Z3TRACE1``) followed by
records: a little-endian ``u16`` body length, a fixed 106-byte machine-state
header, and a tag/length/value tail. This module mirrors
``crates/parity/src/trace_format.rs`` byte for byte and renders every record
as the same canonical JSON object the former JSON Lines trace used, so the
Python tooling keeps its field names.

``iter_events`` also accepts JSON Lines input (older traces and test
fixtures), detected by the absence of the magic.

    python3 scripts/snes9x_trace_format.py decode TRACE.bin [--run N] [--event NAME]
    python3 scripts/snes9x_trace_format.py encode INPUT.jsonl OUTPUT.bin
"""

from __future__ import annotations

import argparse
import io
import json
import struct
import sys
from pathlib import Path
from typing import Any, BinaryIO, Iterable, Iterator, TextIO

MAGIC = b"Z3TRACE1"
HEADER_LEN = 106
PPU_OPERAND_COUNT = 31

KIND_NAMES = {
    1: "frame",
    2: "video",
    3: "nmi",
    4: "nmi-resume",
    5: "pc",
    6: "dma",
    7: "hdma-start",
    8: "hdma-end",
    9: "rng-ppu-read",
    10: "ppu-read",
    11: "ppu-write",
    12: "rng-write",
    13: "wram-write",
    14: "pixel-write",
}
KIND_IDS = {name: kind for kind, name in KIND_NAMES.items()}
STAGE_NAMES = {1: "entry", 2: "return", 3: "presented"}
STAGE_IDS = {name: stage for stage, name in STAGE_NAMES.items()}

TAG_CHANNEL_STATE = 13
# tag -> (json key, signed, encoded width in bytes)
SCALAR_TAGS = {
    1: ("address", False, 4),
    2: ("value", False, 4),
    3: ("h_latched", False, 2),
    4: ("channel", False, 1),
    5: ("source", False, 4),
    6: ("b_address", False, 1),
    7: ("bytes", False, 4),
    8: ("mode", False, 1),
    9: ("fixed", False, 1),
    10: ("decrement", False, 1),
    11: ("vram_address", False, 2),
    12: ("channels", False, 1),
    20: ("pix", True, 4),
    21: ("z1", True, 4),
    22: ("z2", True, 4),
    23: ("tile", True, 4),
    24: ("tile_number", True, 4),
    25: ("tile_address", True, 4),
    26: ("tile_cache_valid", True, 4),
    27: ("tile_cache_hash", False, 4),
    28: ("obj_tile", True, 4),
    29: ("obj_line", True, 4),
    30: ("obj_x", True, 4),
    31: ("obj_y", True, 4),
    32: ("obj_cache", True, 4),
    33: ("obj_tile_number", False, 4),
    34: ("obj_cache_valid", True, 4),
    35: ("obj_cache_hash", False, 4),
}
SCALAR_TAG_BY_KEY = {key: (tag, signed, width) for tag, (key, signed, width) in SCALAR_TAGS.items()}

# The fixed header after kind/stage, in order: (json key, struct code).
HEADER_FIELDS = (
    ("run", "Q"),
    ("frame", "I"),
    ("v", "h"),
    ("cycles", "i"),
    ("pc", "I"),
    ("a", "H"),
    ("x", "H"),
    ("y", "H"),
    ("s", "H"),
    ("carry", "B"),
    ("p", "B"),
    ("main", "B"),
    ("sub", "B"),
    ("subsub", "B"),
    ("frame_counter", "B"),
    ("room", "H"),
    ("lights_out", "B"),
    ("palette_countdown", "B"),
    ("palette_direction", "B"),
    ("link_y", "H"),
    ("link_x", "H"),
    ("bg2_v", "H"),
    ("bg2_h", "H"),
    ("mosaic_target", "B"),
    ("spotlight_radius", "H"),
    ("spotlight_state", "H"),
    ("spotlight_var4_low", "B"),
    ("spotlight_lower_cursor", "H"),
    ("rng_seed", "B"),
    ("nmi_latch", "B"),
    ("nmi_disable", "B"),
    ("nmi_pending", "B"),
    ("joypad_high", "B"),
    ("joypad_low", "B"),
    ("joypad_high_filtered", "B"),
    ("joypad_low_filtered", "B"),
)
_HEADER_STRUCT = struct.Struct("<BB" + "".join(code for _, code in HEADER_FIELDS))
_TAIL_STRUCT = struct.Struct("<I4B")  # return_address, stack1..stack4
assert _HEADER_STRUCT.size + PPU_OPERAND_COUNT + _TAIL_STRUCT.size == HEADER_LEN


def decode_record(body: bytes) -> dict[str, Any]:
    """Decode one record body (everything after the ``u16`` length)."""
    if len(body) < HEADER_LEN:
        raise ValueError(f"trace record body is {len(body)} bytes; the header needs {HEADER_LEN}")
    values = _HEADER_STRUCT.unpack_from(body, 0)
    kind, stage = values[0], values[1]
    event: dict[str, Any] = {"event": KIND_NAMES.get(kind, f"unknown-{kind}")}
    for (key, _), value in zip(HEADER_FIELDS, values[2:]):
        event[key] = value
    offset = _HEADER_STRUCT.size
    event["nmi_ppu_register_operands"] = list(body[offset : offset + PPU_OPERAND_COUNT])
    offset += PPU_OPERAND_COUNT
    return_address, stack1, stack2, stack3, stack4 = _TAIL_STRUCT.unpack_from(body, offset)
    event["return_address"] = return_address
    event["stack1"], event["stack2"], event["stack3"], event["stack4"] = stack1, stack2, stack3, stack4
    if stage in STAGE_NAMES:
        event["stage"] = STAGE_NAMES[stage]
    offset = HEADER_LEN
    channel_state: list[dict[str, Any]] = []
    while offset < len(body):
        if offset + 2 > len(body):
            raise ValueError(f"truncated tag header at byte {offset}")
        tag, length = body[offset], body[offset + 1]
        payload = body[offset + 2 : offset + 2 + length]
        if len(payload) != length:
            raise ValueError(f"truncated tag {tag} payload at byte {offset}")
        offset += 2 + length
        if tag == TAG_CHANNEL_STATE:
            if len(payload) < 14:
                continue
            data_len = payload[13]
            channel_state.append(
                {
                    "channel": payload[0],
                    "source": int.from_bytes(payload[1:5], "little"),
                    "table_address": int.from_bytes(payload[5:7], "little"),
                    "indirect": payload[7],
                    "line_count": payload[8],
                    "repeat": payload[9],
                    "do_transfer": payload[10],
                    "b_address": payload[11],
                    "mode": payload[12],
                    "data": list(payload[14 : 14 + data_len]),
                }
            )
            continue
        scalar = SCALAR_TAGS.get(tag)
        if scalar is None or len(payload) not in (1, 2, 4):
            event[f"tag_{tag}"] = list(payload)
            continue
        key, signed, _ = scalar
        event[key] = int.from_bytes(payload, "little", signed=signed and len(payload) == 4)
    if kind in (KIND_IDS["hdma-start"], KIND_IDS["hdma-end"]):
        event["channel_state"] = channel_state
    return event


def encode_record(event: dict[str, Any]) -> bytes:
    """Encode one canonical JSON object as a framed record (``u16`` length + body)."""
    name = event.get("event")
    if name not in KIND_IDS:
        raise ValueError(f"unknown trace event {name!r}")
    stage = STAGE_IDS.get(event.get("stage"), 0)
    header_values = [KIND_IDS[name], stage]
    for key, code in HEADER_FIELDS:
        value = int(event.get(key, 0) or 0)
        if code == "h":
            value = ((value + 0x8000) & 0xFFFF) - 0x8000
        elif code == "i":
            value = ((value + 0x8000_0000) & 0xFFFF_FFFF) - 0x8000_0000
        else:
            value &= (1 << (8 * struct.calcsize(code))) - 1
        header_values.append(value)
    body = bytearray(_HEADER_STRUCT.pack(*header_values))
    operands = list(event.get("nmi_ppu_register_operands") or [])
    operands = (operands + [0] * PPU_OPERAND_COUNT)[:PPU_OPERAND_COUNT]
    body.extend(int(v) & 0xFF for v in operands)
    body.extend(
        _TAIL_STRUCT.pack(
            int(event.get("return_address", 0) or 0) & 0xFFFF_FFFF,
            *(int(event.get(f"stack{i}", 0) or 0) & 0xFF for i in (1, 2, 3, 4)),
        )
    )
    for key, (tag, signed, width) in SCALAR_TAG_BY_KEY.items():
        if key not in event:
            continue
        value = int(event[key])
        payload = value.to_bytes(width, "little", signed=signed) if signed else (value & ((1 << (8 * width)) - 1)).to_bytes(width, "little")
        body.extend((tag, width))
        body.extend(payload)
    for state in event.get("channel_state") or []:
        data = [int(v) & 0xFF for v in state.get("data", [])]
        payload = bytearray([int(state.get("channel", 0)) & 0xFF])
        payload.extend((int(state.get("source", 0)) & 0xFFFF_FFFF).to_bytes(4, "little"))
        payload.extend((int(state.get("table_address", 0)) & 0xFFFF).to_bytes(2, "little"))
        payload.extend(
            int(state.get(k, 0)) & 0xFF
            for k in ("indirect", "line_count", "repeat", "do_transfer", "b_address", "mode")
        )
        payload.append(len(data))
        payload.extend(data)
        body.extend((TAG_CHANNEL_STATE, len(payload)))
        body.extend(payload)
    return struct.pack("<H", len(body)) + bytes(body)


def encode_events(events: Iterable[dict[str, Any]]) -> bytes:
    """A complete binary trace: magic followed by framed records."""
    out = bytearray(MAGIC)
    for event in events:
        out.extend(encode_record(event))
    return bytes(out)


def _iter_binary(stream: BinaryIO) -> Iterator[dict[str, Any]]:
    number = 0
    while True:
        length_bytes = stream.read(2)
        if not length_bytes:
            return
        if len(length_bytes) != 2:
            raise ValueError(f"record {number + 1}: truncated length prefix")
        (length,) = struct.unpack("<H", length_bytes)
        body = stream.read(length)
        if len(body) != length:
            raise ValueError(f"record {number + 1}: promised {length} bytes but the file ended")
        number += 1
        try:
            yield decode_record(body)
        except ValueError as error:
            raise ValueError(f"record {number}: {error}") from error


def _iter_json_lines(lines: Iterable[str]) -> Iterator[dict[str, Any]]:
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"line {number}: invalid JSON: {error}") from error
        if not isinstance(value, dict):
            raise ValueError(f"line {number}: non-object trace record")
        yield value


def iter_events(source: Any) -> Iterator[dict[str, Any]]:
    """Yield canonical event dicts from a path, a binary stream, or JSON Lines text.

    A path or binary stream starting with the ``Z3TRACE1`` magic is decoded as
    the binary format; anything else is read as JSON Lines (older traces and
    fixtures). Text streams are always JSON Lines.
    """
    if isinstance(source, (str, Path)):
        with open(source, "rb") as stream:
            yield from iter_events(stream)
        return
    if isinstance(source, io.TextIOBase):
        yield from _iter_json_lines(source)
        return
    if isinstance(source, (bytes, bytearray)):
        yield from iter_events(io.BytesIO(source))
        return
    if hasattr(source, "read"):
        head = source.read(len(MAGIC))
        if head == MAGIC:
            yield from _iter_binary(source)
            return
        rest = source.read()
        text = (head + rest).decode("utf-8")
        yield from _iter_json_lines(text.splitlines())
        return
    # Any other iterable of already-decoded events or JSON lines.
    for item in source:
        if isinstance(item, dict):
            yield item
        else:
            yield from _iter_json_lines([item])


def is_binary_trace(path: Path) -> bool:
    try:
        with open(path, "rb") as stream:
            return stream.read(len(MAGIC)) == MAGIC
    except OSError:
        return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    decode = sub.add_parser("decode", help="render a trace as JSON Lines on stdout")
    decode.add_argument("trace", type=Path)
    decode.add_argument("--run", type=int, default=None, help="only this retro_run")
    decode.add_argument("--run-range", default=None, help="A-B inclusive retro_run range")
    decode.add_argument("--event", default=None, help="only this event kind")
    decode.add_argument("--limit", type=int, default=None)
    encode = sub.add_parser("encode", help="convert JSON Lines records into the binary format")
    encode.add_argument("input", type=Path)
    encode.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.command == "encode":
        with args.input.open(encoding="utf-8") as stream:
            args.output.write_bytes(encode_events(_iter_json_lines(stream)))
        return 0
    lo = hi = None
    if args.run is not None:
        lo = hi = args.run
    elif args.run_range:
        a, _, b = args.run_range.partition("-")
        lo, hi = int(a), int(b or a)
    emitted = 0
    out: TextIO = sys.stdout
    for event in iter_events(args.trace):
        run = event.get("run", 0)
        if lo is not None and run < lo:
            continue
        if hi is not None and run > hi:
            break
        if args.event and event.get("event") != args.event:
            continue
        out.write(json.dumps(event, separators=(",", ":")) + "\n")
        emitted += 1
        if args.limit and emitted >= args.limit:
            break
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
