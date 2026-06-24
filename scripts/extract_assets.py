#!/usr/bin/env python3
"""Generate ignored Zelda 3 runtime assets from a user-provided ROM.

This wrapper keeps generated output in this repo while delegating the asset pack
format to the original C project's restool.py, which remains the source of
truth for resource extraction. The generated repo-local output is a folder of
individual asset files, not the monolithic restool pack.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import tilemap_json


REPO_ROOT = Path(__file__).resolve().parents[1]
ASSET_COUNT = 165
DEFAULT_C_SOURCE = REPO_ROOT.parent / "zelda3"
DEFAULT_OUT_DIR = Path("generated/zelda3_assets")
ASSET_SIGNATURE_PREFIX = b"Zelda3_v0     \n\0"
PREVIEW_ASSETS = {
    "kLinkGraphics",
    "kSprGfx",
    "kBgGfx",
    "kOverworldMapGfx",
    "kGeneratedWishPondItem",
    "kGeneratedBombosArr",
    "kGeneratedEndSequence15",
}
READABLE_ASSET_SOURCES = {
    "kLightOverworldTilemap": {
        "format": tilemap_json.FORMAT_BYTE_TILEMAP,
        "file": "assets_src/tilemaps/light_overworld_tilemap.json",
        "width": 64,
        "height": 64,
    }
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", required=True, type=Path, help="Path to a USA Zelda 3 ROM")
    parser.add_argument(
        "--out-dir",
        default=DEFAULT_OUT_DIR,
        type=Path,
        help="Directory for generated ignored assets",
    )
    parser.add_argument(
        "--c-source",
        default=os.environ.get("ZELDA3_C_SOURCE", str(DEFAULT_C_SOURCE)),
        type=Path,
        help="Path to the original zelda3 C checkout containing assets/restool.py",
    )
    return parser.parse_args()


def sha1(path: Path) -> str:
    digest = hashlib.sha1()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little")


def split_asset_pack(asset_pack: Path) -> tuple[bytes, bytes, list[tuple[str, bytes]]]:
    data = asset_pack.read_bytes()
    if len(data) < 88 or data[:16] != ASSET_SIGNATURE_PREFIX:
        raise RuntimeError(f"{asset_pack} is not a valid zelda3_assets.dat")
    count = read_u32(data, 80)
    if count != ASSET_COUNT:
        raise RuntimeError(f"{asset_pack} has {count} assets, expected {ASSET_COUNT}")
    key_signature_len = read_u32(data, 84)
    sizes_start = 88
    key_signature_start = sizes_start + count * 4
    payload_offset = key_signature_start + key_signature_len
    key_signature = data[key_signature_start:payload_offset]
    names = key_signature.rstrip(b"\0").decode("utf8").split("\0")
    if len(names) != count:
        raise RuntimeError(f"{asset_pack} has {len(names)} asset names, expected {count}")

    assets: list[tuple[str, bytes]] = []
    offset = payload_offset
    for i, name in enumerate(names):
        size = read_u32(data, sizes_start + i * 4)
        offset = (offset + 3) & ~3
        end = offset + size
        if end > len(data):
            raise RuntimeError(f"{asset_pack} asset {i} extends past end of file")
        assets.append((name, data[offset:end]))
        offset = end
    return data[:48], key_signature, assets


def clean_generated_output(out_dir: Path) -> Path:
    assets_dir = out_dir / "assets"
    images_dir = out_dir / "images"
    assets_src_dir = out_dir / "assets_src"
    if assets_dir.exists():
        for path in assets_dir.iterdir():
            if path.is_file():
                path.unlink()
    else:
        assets_dir.mkdir(parents=True, exist_ok=True)
    if images_dir.exists():
        for path in images_dir.iterdir():
            if path.is_file():
                path.unlink()
    if assets_src_dir.exists():
        shutil.rmtree(assets_src_dir)
    old_pack = out_dir / "zelda3_assets.dat"
    if old_pack.exists():
        old_pack.unlink()
    return assets_dir


def write_asset_output(
    out_dir: Path, *, index: int, name: str, payload: bytes
) -> dict[str, object]:
    source = READABLE_ASSET_SOURCES.get(name)
    manifest_asset: dict[str, object] = {
        "index": index,
        "name": name,
        "size": len(payload),
        "sha1": hashlib.sha1(payload).hexdigest(),
    }
    if source is None:
        file_name = f"{index:03d}-{name}.bin"
        file_path = out_dir / "assets" / file_name
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_bytes(payload)
        manifest_asset["file"] = f"assets/{file_name}"
        return manifest_asset

    tilemap = tilemap_json.tilemap_from_bytes(
        payload,
        asset=name,
        asset_index=index,
        width=int(source["width"]),
        height=int(source["height"]),
    )
    source_file = str(source["file"])
    tilemap_json.write_tilemap_json(out_dir / source_file, tilemap)
    manifest_asset["source_file"] = source_file
    manifest_asset["source_format"] = source["format"]
    return manifest_asset


def decomp_asset(data: bytes) -> bytes:
    result = bytearray()
    offset = 0
    while True:
        control = data[offset]
        offset += 1
        if control == 0xFF:
            return bytes(result)
        if (control & 0xE0) != 0xE0:
            cmd = control & 0xE0
            length = control & 0x1F
        else:
            cmd = (control << 3) & 0xE0
            length = ((control & 3) << 8) | data[offset]
            offset += 1
        length += 1
        if cmd == 0x00:
            result.extend(data[offset : offset + length])
            offset += length
        elif cmd & 0x80:
            src = data[offset] | (data[offset + 1] << 8)
            offset += 2
            for _ in range(length):
                result.append(result[src])
                src += 1
        elif (cmd & 0x40) == 0:
            value = data[offset]
            offset += 1
            result.extend(bytes([value]) * length)
        elif (cmd & 0x20) == 0:
            first = data[offset]
            second = data[offset + 1]
            offset += 2
            while length:
                result.append(first)
                if length == 1:
                    break
                result.append(second)
                length -= 2
        else:
            value = data[offset]
            offset += 1
            for _ in range(length):
                result.append(value)
                value = (value + 1) & 0xFF


def unpack_packed_arrays(data: bytes) -> list[bytes]:
    marker = int.from_bytes(data[-2:], "little")
    if marker >= 8192:
        count = marker - 8192 + 1
        offset_size = 4
    else:
        count = marker + 1
        offset_size = 2

    offsets = []
    pos = 0
    for _ in range(count - 1):
        offsets.append(int.from_bytes(data[pos : pos + offset_size], "little"))
        pos += offset_size

    payload = data[pos:-2]
    starts = [0, *offsets]
    ends = [*offsets, len(payload)]
    return [payload[start:end] for start, end in zip(starts, ends)]


def decode_planar_tiles(data: bytes, bpp: int, columns: int = 16) -> tuple[int, int, bytearray]:
    bytes_per_tile = {2: 16, 3: 24, 4: 32}[bpp]
    tile_count = len(data) // bytes_per_tile
    rows = (tile_count + columns - 1) // columns
    width = columns * 8
    height = rows * 8
    pixels = bytearray(width * height)
    for tile in range(tile_count):
        tile_base = tile * bytes_per_tile
        tile_x = (tile % columns) * 8
        tile_y = (tile // columns) * 8
        for y in range(8):
            plane0 = data[tile_base + y * 2]
            plane1 = data[tile_base + y * 2 + 1]
            plane2 = data[tile_base + 16 + y] if bpp >= 3 else 0
            plane3 = data[tile_base + 16 + y * 2 + 1] if bpp >= 4 else 0
            for x in range(8):
                bit = 7 - x
                value = ((plane0 >> bit) & 1) | (((plane1 >> bit) & 1) << 1)
                value |= ((plane2 >> bit) & 1) << 2
                value |= ((plane3 >> bit) & 1) << 3
                pixels[(tile_y + y) * width + tile_x + x] = value
    return width, height, pixels


def preview_palette() -> list[int]:
    # Most generated graphics assets are raw SNES tile planes: they contain
    # palette indices, not the palette/attribute context needed for final
    # in-game colors. Use a neutral ramp so previews show shape without
    # pretending to be authoritative color renders.
    colors = [
        (0, 0, 0),
        (34, 34, 34),
        (51, 51, 51),
        (68, 68, 68),
        (85, 85, 85),
        (102, 102, 102),
        (119, 119, 119),
        (136, 136, 136),
        (153, 153, 153),
        (170, 170, 170),
        (187, 187, 187),
        (204, 204, 204),
        (221, 221, 221),
        (238, 238, 238),
        (255, 255, 255),
        (255, 255, 255),
    ]
    palette = [channel for color in colors for channel in color]
    palette.extend([0] * (256 * 3 - len(palette)))
    return palette


def link_palette() -> list[int]:
    snes_colors = [
        0,
        0x7FFF,
        0x237E,
        0x11B7,
        0x369E,
        0x14A5,
        0x01FF,
        0x1078,
        0x599D,
        0x3647,
        0x3B68,
        0x0A4A,
        0x12EF,
        0x2A5C,
        0x1571,
        0x7A18,
    ]
    palette = []
    for color in snes_colors:
        r = color & 0x1F
        g = (color >> 5) & 0x1F
        b = (color >> 10) & 0x1F
        palette.extend([r << 3 | r >> 2, g << 3 | g >> 2, b << 3 | b >> 2])
    palette.extend([0] * (256 * 3 - len(palette)))
    return palette


def choose_bpp(data: bytes) -> int:
    if len(data) % 32 == 0:
        return 4
    if len(data) % 24 == 0:
        return 3
    if len(data) % 16 == 0:
        return 2
    raise RuntimeError(f"cannot infer SNES tile bit depth for {len(data)} bytes")


def save_indexed_png(path: Path, size: tuple[int, int], pixels: bytes, palette: list[int]) -> None:
    from PIL import Image

    image = Image.new("P", size)
    image.putpalette(palette)
    image.putdata(pixels)
    image.save(path)


def save_planar_preview(path: Path, data: bytes, palette: list[int], bpp: int | None = None) -> None:
    width, height, pixels = decode_planar_tiles(data, bpp or choose_bpp(data))
    save_indexed_png(path, (width, height), pixels, palette)


def decode_packed_graphics(data: bytes, uncompressed_prefix_count: int = 0) -> bytearray:
    rows = []
    for index, item in enumerate(unpack_packed_arrays(data)):
        decoded = item if index < uncompressed_prefix_count else decomp_asset(item)
        width, height, pixels = decode_planar_tiles(decoded, choose_bpp(decoded))
        rows.append((width, height, pixels))
    width = max(row[0] for row in rows)
    height = sum(row[1] for row in rows)
    out = bytearray(width * height)
    y_offset = 0
    for row_width, row_height, pixels in rows:
        for y in range(row_height):
            dst = (y_offset + y) * width
            src = y * row_width
            out[dst : dst + row_width] = pixels[src : src + row_width]
        y_offset += row_height
    return width, height, out


def write_preview_images(out_dir: Path, assets: list[tuple[str, bytes]]) -> list[dict[str, str]]:
    try:
        from PIL import Image  # noqa: F401
    except ImportError:
        print("Pillow is not installed; skipping generated PNG previews", file=sys.stderr)
        return []

    images_dir = out_dir / "images"
    images_dir.mkdir(parents=True, exist_ok=True)
    previews = []
    generic_palette = preview_palette()
    for index, (name, payload) in enumerate(assets):
        if name not in PREVIEW_ASSETS:
            continue
        file_name = f"{index:03d}-{name}.png"
        path = images_dir / file_name
        if name == "kLinkGraphics":
            save_planar_preview(path, payload, link_palette(), bpp=4)
        elif name == "kSprGfx":
            width, height, pixels = decode_packed_graphics(payload, uncompressed_prefix_count=12)
            save_indexed_png(path, (width, height), pixels, generic_palette)
        elif name == "kBgGfx":
            width, height, pixels = decode_packed_graphics(payload)
            save_indexed_png(path, (width, height), pixels, generic_palette)
        else:
            try:
                save_planar_preview(path, payload, generic_palette)
            except RuntimeError as exc:
                print(f"skipping PNG preview for {name}: {exc}", file=sys.stderr)
                continue
        previews.append({"asset": name, "file": f"images/{file_name}"})
    return previews


def main() -> int:
    args = parse_args()
    rom = args.rom.expanduser().resolve()
    c_source = args.c_source.expanduser().resolve()
    out_dir = args.out_dir.resolve()
    restool = c_source / "assets" / "restool.py"
    source_pack = c_source / "zelda3_assets.dat"
    assets_dir = out_dir / "assets"
    manifest = out_dir / "manifest.json"
    signature_path = out_dir / "asset_signature.bin"
    key_signature_path = out_dir / "asset_key_signature.bin"

    if not rom.is_file():
        print(f"ROM not found: {rom}", file=sys.stderr)
        return 2
    if not restool.is_file():
        print(f"restool.py not found: {restool}", file=sys.stderr)
        print("Set ZELDA3_C_SOURCE or pass --c-source /path/to/zelda3.", file=sys.stderr)
        return 2

    out_dir.mkdir(parents=True, exist_ok=True)
    clean_generated_output(out_dir)
    if source_pack.exists():
        source_pack.unlink()

    subprocess.run(
        [sys.executable, str(restool), "--rom", str(rom)],
        cwd=c_source,
        check=True,
    )
    if not source_pack.is_file():
        raise RuntimeError(f"restool did not create {source_pack}")

    signature, key_signature, assets = split_asset_pack(source_pack)
    signature_path.write_bytes(signature)
    key_signature_path.write_bytes(key_signature)

    manifest_assets = []
    for index, (name, payload) in enumerate(assets):
        manifest_assets.append(
            write_asset_output(out_dir, index=index, name=name, payload=payload)
        )
    previews = write_preview_images(out_dir, assets)

    manifest.write_text(
        json.dumps(
            {
                "asset_count": len(assets),
                "asset_key_signature": key_signature_path.name,
                "asset_signature": signature_path.name,
                "assets": manifest_assets,
                "image_previews": previews,
                "restool_pack_sha1": sha1(source_pack),
                "rom_sha1": sha1(rom),
                "source_tool": str(restool),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )
    bin_count = sum(1 for asset in manifest_assets if "file" in asset)
    source_count = sum(1 for asset in manifest_assets if "source_file" in asset)
    print(f"wrote {bin_count} binary asset files to {assets_dir}")
    if source_count:
        print(f"wrote {source_count} readable asset sources to {out_dir / 'assets_src'}")
    if previews:
        print(f"wrote {len(previews)} PNG previews to {out_dir / 'images'}")
    print(f"wrote {manifest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
