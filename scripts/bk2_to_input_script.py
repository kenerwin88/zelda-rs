#!/usr/bin/env python3
"""Convert a SNES TAS movie into a zelda3-rs input script.

Supported inputs:
- BizHawk .bk2, including TASVideos gzip-wrapped downloads.
- Snes9x .smv, including TASVideos .smv.zip downloads.
"""

from __future__ import annotations

import argparse
import gzip
import io
import struct
import zipfile
import zlib
from pathlib import Path


P1_TO_ZELDA3_BITS = [
    ("UP", 1 << 4),
    ("DOWN", 1 << 5),
    ("LEFT", 1 << 6),
    ("RIGHT", 1 << 7),
    ("SELECT", 1 << 2),
    ("START", 1 << 3),
    ("Y", 1 << 1),
    ("B", 1 << 0),
    ("X", 1 << 9),
    ("A", 1 << 8),
    ("L", 1 << 10),
    ("R", 1 << 11),
]

SMV_TO_ZELDA3_BITS = [
    (0x0010, 1 << 11),  # R
    (0x0020, 1 << 10),  # L
    (0x0040, 1 << 9),  # X
    (0x0080, 1 << 8),  # A
    (0x0100, 1 << 7),  # Right
    (0x0200, 1 << 6),  # Left
    (0x0400, 1 << 5),  # Down
    (0x0800, 1 << 4),  # Up
    (0x1000, 1 << 3),  # Start
    (0x2000, 1 << 2),  # Select
    (0x4000, 1 << 1),  # Y
    (0x8000, 1 << 0),  # B
]


def read_movie_bytes(path: Path) -> bytes:
    data = path.read_bytes()
    if data.startswith(b"\x1f\x8b"):
        data = gzip.decompress(data)
    return data


def read_bk2_entry(path: Path, entry: str) -> str:
    data = read_movie_bytes(path)
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        return archive.read(entry).decode("utf-8-sig")


def parse_input_log(input_log: str) -> list[int]:
    frames: list[int] = []
    saw_input = False
    for raw_line in input_log.splitlines():
        if raw_line == "[Input]":
            saw_input = True
            continue
        if not saw_input or not raw_line or raw_line.startswith("LogKey:"):
            continue
        if not raw_line.startswith("|"):
            continue

        columns = raw_line.split("|")
        if len(columns) < 4:
            continue
        p1 = columns[2]
        if len(p1) != len(P1_TO_ZELDA3_BITS):
            raise ValueError(f"unexpected P1 field width {len(p1)} in line: {raw_line!r}")

        value = 0
        for pressed, (_name, bit) in zip(p1, P1_TO_ZELDA3_BITS):
            if pressed != ".":
                value |= bit
        frames.append(value)
    return frames


def read_smv(path: Path) -> tuple[str, bytes]:
    data = read_movie_bytes(path)
    if data.startswith(b"SMV\x1a"):
        return path.name, data
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        smv_names = [name for name in archive.namelist() if name.lower().endswith(".smv")]
        if len(smv_names) != 1:
            raise ValueError(f"expected one .smv in {path}, found {len(smv_names)}")
        return smv_names[0], archive.read(smv_names[0])


def le_u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def le_u16(data: bytes, offset: int) -> int:
    return struct.unpack_from("<H", data, offset)[0]


def extract_smv_sram(data: bytes, save_data_offset: int, controller_data_offset: int) -> bytes | None:
    if save_data_offset == 0 or save_data_offset >= controller_data_offset:
        return None
    payload = data[save_data_offset:controller_data_offset]
    if not payload:
        return None

    decompressor = zlib.decompressobj(16 + zlib.MAX_WBITS)
    try:
        sram = decompressor.decompress(payload) + decompressor.flush()
    except zlib.error as err:
        raise ValueError(f"failed to decompress SMV SRAM block: {err}") from err
    return sram or None


def parse_smv(path: Path, *, sram_output: Path | None = None) -> tuple[list[int], list[str]]:
    smv_name, data = read_smv(path)
    if len(data) < 0x20 or data[:4] != b"SMV\x1a":
        raise ValueError(f"{path} is not a Snes9x .smv movie")
    version = le_u32(data, 0x04)
    if version not in {1, 4, 5}:
        raise ValueError(f"unsupported SMV version {version}")

    rerecord_count = le_u32(data, 0x0C)
    frame_count = le_u32(data, 0x10)
    controller_mask = data[0x14]
    movie_options = data[0x15]
    sync1 = data[0x16]
    sync2 = data[0x17]
    save_data_offset = le_u32(data, 0x18)
    controller_data_offset = le_u32(data, 0x1C)
    if controller_data_offset > len(data):
        raise ValueError("SMV controller data offset extends past file")
    if not controller_mask & 0x01:
        raise ValueError("SMV does not include controller 1 input")

    controller_count = controller_mask.bit_count()
    sample_size = 2 * controller_count
    if version in {4, 5}:
        for port_type in data[0x24:0x26]:
            if port_type == 2:
                sample_size += 5
            elif port_type == 3:
                sample_size += 6
            elif port_type == 4:
                sample_size += 11

    available_samples = (len(data) - controller_data_offset) // sample_size
    samples_to_read = min(frame_count, available_samples)
    frames: list[int] = []
    for sample in range(samples_to_read):
        smv_value = le_u16(data, controller_data_offset + sample * sample_size)
        if smv_value == 0xFFFF:
            frames.append(0)
            continue
        value = 0
        for smv_bit, zelda3_bit in SMV_TO_ZELDA3_BITS:
            if smv_value & smv_bit:
                value |= zelda3_bit
        frames.append(value)

    metadata = [
        f"Source {smv_name}",
        f"MovieVersion Snes9x SMV version {version}",
        f"rerecordCount {rerecord_count}",
        f"frames {frame_count}",
        f"controllerMask 0x{controller_mask:02X}",
        f"movieOptions 0x{movie_options:02X}",
        f"syncOptions 0x{sync1:02X} 0x{sync2:02X}",
        f"saveDataOffset {save_data_offset}",
        f"controllerDataOffset {controller_data_offset}",
    ]

    sram = extract_smv_sram(data, save_data_offset, controller_data_offset)
    if sram is not None:
        if sram_output is not None:
            sram_output.parent.mkdir(parents=True, exist_ok=True)
            sram_output.write_bytes(sram)
            metadata.append(f"sramPath {sram_output}")
        metadata.append(f"sramBytes {len(sram)}")
    if samples_to_read != frame_count:
        metadata.append(f"warning converted {samples_to_read} samples, header requested {frame_count}")
    return frames, metadata


def coalesced_ranges(frames: list[int], *, offset: int) -> list[tuple[int, int, int]]:
    if not frames:
        return []
    ranges: list[tuple[int, int, int]] = []
    start = offset
    prev = offset
    value = frames[0]
    for frame, next_value in enumerate(frames[1:], start=offset + 1):
        if next_value == value:
            prev = frame
            continue
        if value != 0:
            ranges.append((start, prev, value))
        start = prev = frame
        value = next_value
    if value != 0:
        ranges.append((start, prev, value))
    return ranges


def format_range(start: int, end: int, value: int) -> str:
    frame = str(start) if start == end else f"{start}..{end}"
    return f"{frame} 0x{value:04X}"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("movie", type=Path)
    parser.add_argument("-o", "--output", type=Path, required=True)
    parser.add_argument(
        "--offset",
        type=int,
        default=0,
        help="add this many frames to every generated input range",
    )
    parser.add_argument(
        "--max-frames",
        type=int,
        help="only convert the first N movie frames",
    )
    parser.add_argument(
        "--extract-sram",
        nargs="?",
        const=True,
        default=None,
        help="extract an SMV embedded SRAM block; defaults to <output>.sram when no path is provided",
    )
    args = parser.parse_args()

    sram_output = None
    if args.extract_sram is True:
        sram_output = args.output.with_suffix(".sram")
    elif args.extract_sram is not None:
        sram_output = Path(args.extract_sram)

    movie_bytes = read_movie_bytes(args.movie)
    if movie_bytes.startswith(b"SMV\x1a") or zipfile.is_zipfile(io.BytesIO(movie_bytes)):
        try:
            frames, metadata = parse_smv(args.movie, sram_output=sram_output)
            source_kind = "Snes9x .smv controller data"
        except (KeyError, ValueError, zipfile.BadZipFile):
            header = read_bk2_entry(args.movie, "Header.txt")
            input_log = read_bk2_entry(args.movie, "Input Log.txt")
            frames = parse_input_log(input_log)
            metadata = header.splitlines()
            source_kind = "BizHawk .bk2 input log"
    else:
        header = read_bk2_entry(args.movie, "Header.txt")
        input_log = read_bk2_entry(args.movie, "Input Log.txt")
        frames = parse_input_log(input_log)
        metadata = header.splitlines()
        source_kind = "BizHawk .bk2 input log"

    if args.max_frames is not None:
        frames = frames[: args.max_frames]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="\n") as fh:
        fh.write(f"# Generated from {source_kind}.\n")
        for line in metadata:
            fh.write(f"# {line}\n")
        fh.write("#\n")
        fh.write("# Format: <frame-or-inclusive-range> <hex input word>\n")
        fh.write("# zelda3-rs input bits: B=0x0001 Y=0x0002 SELECT=0x0004 START=0x0008 ")
        fh.write("UP=0x0010 DOWN=0x0020 LEFT=0x0040 RIGHT=0x0080 A=0x0100 X=0x0200 L=0x0400 R=0x0800\n")
        for start, end, value in coalesced_ranges(frames, offset=args.offset):
            fh.write(format_range(start, end, value) + "\n")

    print(f"converted {len(frames)} frame(s) to {args.output}")


if __name__ == "__main__":
    main()
