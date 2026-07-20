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

import navigation_json
import palette_json
import tilemap_json
import dialogue_catalog


REPO_ROOT = Path(__file__).resolve().parents[1]
ASSET_COUNT = 165
DEFAULT_C_SOURCE = REPO_ROOT.parent / "zelda3"
DEFAULT_OUT_DIR = Path("generated/zelda3_assets")
ASSET_SIGNATURE_PREFIX = b"Zelda3_v0     \n\0"
SPC_DRIVER_UPLOAD_ADDRESS = 0x998000
SPC_DRIVER_LOAD_ADDRESS = 0x0800
SPC_DRIVER_EXPECTED_LENGTH = 0x0F9E
SPC_DRIVER_FILE = "spc_driver.bin"
READABLE_ASSET_SOURCES = {
    "kLightOverworldTilemap": {
        "format": tilemap_json.FORMAT_BYTE_TILEMAP,
        "file": "assets/tilemaps/light_overworld_tilemap.json",
        "width": 64,
        "height": 64,
    },
    "kDarkOverworldTilemap": {
        "format": tilemap_json.FORMAT_BYTE_TILEMAP,
        "file": "assets/tilemaps/dark_overworld_tilemap.json",
        "width": 32,
        "height": 32,
    },
    "kBgTilemap_0": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_0.json",
    },
    "kBgTilemap_1": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_1.json",
    },
    "kBgTilemap_2": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_2.json",
    },
    "kBgTilemap_3": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_3.json",
    },
    "kBgTilemap_4": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_4.json",
    },
    "kBgTilemap_5": {
        "format": tilemap_json.FORMAT_BYTE_STREAM_TILEMAP,
        "file": "assets/tilemaps/bg_tilemap_5.json",
    },
    "kPalette_DungBgMain": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_dung_bg_main.json",
    },
    "kPalette_MainSpr": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_main_spr.json",
    },
    "kPalette_ArmorAndGloves": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_armor_and_gloves.json",
    },
    "kPalette_Sword": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_sword.json",
    },
    "kPalette_Shield": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_shield.json",
    },
    "kPalette_SpriteAux3": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_sprite_aux3.json",
    },
    "kPalette_MiscSprite_Indoors": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_misc_sprite_indoors.json",
    },
    "kPalette_SpriteAux1": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_sprite_aux1.json",
    },
    "kPalette_OverworldBgMain": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_overworld_bg_main.json",
    },
    "kPalette_OverworldBgAux12": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_overworld_bg_aux12.json",
    },
    "kPalette_OverworldBgAux3": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_overworld_bg_aux3.json",
    },
    "kPalette_PalaceMapBg": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_palace_map_bg.json",
    },
    "kPalette_PalaceMapSpr": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/palette_palace_map_spr.json",
    },
    "kHudPalData": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/hud_pal_data.json",
    },
    "kOverworldMapPaletteData": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/overworld_map_palette_data.json",
    },
    "kOverworldBgPalettes": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/overworld_bg_palettes.json",
    },
    "kOverworldSpritePalettes": {
        "format": palette_json.FORMAT_SNES_PALETTE,
        "file": "assets/palettes/overworld_sprite_palettes.json",
    },
}

NAVIGATION_SOURCE_GROUPS = [
    {
        "format": navigation_json.FORMAT_DUNGEON_ENTRANCES,
        "file": "assets/navigation/dungeon_entrances.json",
        "asset_index_range": [11, 27],
        "asset_names": [field[0] for field in navigation_json.ENTRANCE_FIELDS],
        "source_from_assets": navigation_json.entrance_source_from_assets,
        "source_kwargs": {"asset": "kEntranceData"},
    },
    {
        "format": navigation_json.FORMAT_STARTING_POINTS,
        "file": "assets/navigation/starting_points.json",
        "asset_index_range": [28, 45],
        "asset_names": [field[0] for field in navigation_json.STARTING_POINT_FIELDS],
        "source_from_assets": navigation_json.starting_point_source_from_assets,
        "source_kwargs": {},
    },
    {
        "format": navigation_json.FORMAT_OVERWORLD_EXITS,
        "file": "assets/navigation/overworld_exits.json",
        "asset_index_range": [130, 142],
        "asset_names": [field[0] for field in navigation_json.EXIT_FIELDS],
        "source_from_assets": navigation_json.exit_source_from_assets,
        "source_kwargs": {},
    },
    {
        "format": navigation_json.FORMAT_SPECIAL_EXITS,
        "file": "assets/navigation/special_exits.json",
        "asset_index_range": [143, 156],
        "asset_names": [field[0] for field in navigation_json.SPECIAL_EXIT_FIELDS],
        "source_from_assets": navigation_json.special_exit_source_from_assets,
        "source_kwargs": {},
    },
]

NAVIGATION_ASSET_SOURCES = {
    asset_name: group
    for group in NAVIGATION_SOURCE_GROUPS
    for asset_name in group["asset_names"]
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
    parser.add_argument(
        "--write-diagnostic-variants",
        action="store_true",
        help=(
            "Also emit atlas/tile_variants.{png,json}; this brute-force oracle "
            "is large and is not needed for the default compact asset path."
        ),
    )
    parser.add_argument(
        "--write-base-effect-atlas",
        action="store_true",
        help=(
            "Also emit legacy atlas/base_tiles.{png,json}. The default runtime "
            "uses atlas/art_tiles.{png,json} plus atlas/tile_effects.json."
        ),
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


def lorom_offset(address: int) -> int:
    """Translate a headerless LoROM CPU address to a file offset."""
    bank = (address >> 16) & 0x7F
    offset = address & 0xFFFF
    if offset < 0x8000:
        raise ValueError(f"LoROM address ${address:06x} is outside the ROM window")
    return bank * 0x8000 + (offset - 0x8000)


def next_lorom_address(address: int) -> int:
    address += 1
    if address & 0xFFFF < 0x8000:
        address += 0x8000
    return address


def rom_byte(rom: bytes, address: int) -> int:
    offset = lorom_offset(address)
    if offset >= len(rom):
        raise RuntimeError(f"ROM address ${address:06x} extends past the input ROM")
    return rom[offset]


def read_rom_word(rom: bytes, address: int) -> tuple[int, int]:
    low = rom_byte(rom, address)
    address = next_lorom_address(address)
    high = rom_byte(rom, address)
    return low | high << 8, next_lorom_address(address)


def extract_spc_driver_program(
    rom_data: bytes,
    upload_address: int = SPC_DRIVER_UPLOAD_ADDRESS,
) -> bytes:
    """Extract the executable SPC program from the ROM's intro upload stream.

    The authored sound-bank asset intentionally omits this block because the
    translated driver owns sequencing.  The absolute event clock needs the
    original instructions only to reproduce driver poll timing; it never uses
    them to synthesize samples.
    """
    rom = rom_data[512:] if len(rom_data) % 0x8000 == 512 else rom_data
    address = upload_address
    driver = None
    for _ in range(256):
        length, address = read_rom_word(rom, address)
        target, address = read_rom_word(rom, address)
        if length == 0:
            if target != SPC_DRIVER_LOAD_ADDRESS:
                raise RuntimeError(
                    "intro SPC upload entry point is "
                    f"${target:04x}, expected $0800"
                )
            if driver is None:
                raise RuntimeError("intro SPC upload has no $0800 driver block")
            return driver

        payload = bytearray()
        for _ in range(length):
            payload.append(rom_byte(rom, address))
            address = next_lorom_address(address)
        if target == SPC_DRIVER_LOAD_ADDRESS:
            if driver is not None:
                raise RuntimeError("intro SPC upload has multiple $0800 driver blocks")
            driver = bytes(payload)

    raise RuntimeError("intro SPC upload did not terminate")


def write_spc_driver_program(out_dir: Path, rom: Path) -> dict[str, object]:
    driver = extract_spc_driver_program(rom.read_bytes())
    if len(driver) != SPC_DRIVER_EXPECTED_LENGTH:
        raise RuntimeError(
            f"SPC driver is {len(driver)} bytes, expected {SPC_DRIVER_EXPECTED_LENGTH}"
        )
    path = out_dir / SPC_DRIVER_FILE
    path.write_bytes(driver)
    return {
        "file": path.name,
        "load_address": f"0x{SPC_DRIVER_LOAD_ADDRESS:04x}",
        "rom_upload_address": f"0x{SPC_DRIVER_UPLOAD_ADDRESS:06x}",
        "size": len(driver),
    }


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
    atlas_dir = out_dir / "atlas"
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
    if atlas_dir.exists():
        shutil.rmtree(atlas_dir)
    old_pack = out_dir / "zelda3_assets.dat"
    if old_pack.exists():
        old_pack.unlink()
    old_spc_driver = out_dir / SPC_DRIVER_FILE
    if old_spc_driver.exists():
        old_spc_driver.unlink()
    return assets_dir


def preserve_palette_usage(out_dir: Path) -> bytes | None:
    path = out_dir / "atlas" / "palette_usage.json"
    if not path.is_file():
        return None
    return path.read_bytes()


def restore_palette_usage(out_dir: Path, data: bytes | None) -> None:
    if data is None:
        return
    path = out_dir / "atlas" / "palette_usage.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def bootstrap_source_file(source_root: Path, source_file: str, write) -> None:
    """Write a tracked readable-source file, bootstrap-only.

    `source_file` is a repo-relative path like `assets/palettes/foo.json`; it is
    resolved against `source_root` (the repo root in production, a temp fixture
    in tests). When the tracked file already exists it is the authority (possibly
    user-edited, guarded by its parity lock) and extraction must NOT overwrite
    it — mirrors `write_chr_source_sheets`.
    """
    target = source_root / source_file
    if target.exists():
        return
    target.parent.mkdir(parents=True, exist_ok=True)
    write(target)


def write_asset_output(
    out_dir: Path,
    *,
    index: int,
    name: str,
    payload: bytes,
    source_root: Path | None = None,
) -> dict[str, object]:
    source_root = source_root or REPO_ROOT
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

    if source["format"] == tilemap_json.FORMAT_BYTE_TILEMAP:
        tilemap = tilemap_json.tilemap_from_bytes(
            payload,
            asset=name,
            asset_index=index,
            width=int(source["width"]),
            height=int(source["height"]),
        )
    elif source["format"] == tilemap_json.FORMAT_BYTE_STREAM_TILEMAP:
        tilemap = tilemap_json.tilemap_stream_from_bytes(
            payload,
            asset=name,
            asset_index=index,
        )
    elif source["format"] == palette_json.FORMAT_SNES_PALETTE:
        palette = palette_json.palette_from_bytes(
            payload,
            asset=name,
            asset_index=index,
        )
        source_file = str(source["file"])
        bootstrap_source_file(
            source_root,
            source_file,
            lambda path: palette_json.write_palette_json(path, palette),
        )
        manifest_asset["source_file"] = source_file
        manifest_asset["source_format"] = source["format"]
        return manifest_asset
    else:
        raise RuntimeError(f"unsupported readable asset source format: {source['format']}")
    source_file = str(source["file"])
    bootstrap_source_file(
        source_root,
        source_file,
        lambda path: tilemap_json.write_tilemap_json(path, tilemap),
    )
    manifest_asset["source_file"] = source_file
    manifest_asset["source_format"] = source["format"]
    return manifest_asset


def write_asset_outputs(
    out_dir: Path,
    assets: list[tuple[str, bytes]],
    source_root: Path | None = None,
) -> list[dict[str, object]]:
    source_root = source_root or REPO_ROOT
    navigation_groups = {}
    assets_by_name = dict(assets)
    for group in NAVIGATION_SOURCE_GROUPS:
        if all(asset_name in assets_by_name for asset_name in group["asset_names"]):
            group_assets = {
                asset_name: assets_by_name[asset_name]
                for asset_name in group["asset_names"]
            }
            source = group["source_from_assets"](
                group_assets,
                asset_index_range=group["asset_index_range"],
                **group["source_kwargs"],
            )
            source_file = str(group["file"])
            bootstrap_source_file(
                source_root,
                source_file,
                lambda path, source=source: navigation_json.write_navigation_json(path, source),
            )
            navigation_groups[group["format"]] = source

    manifest_assets = []
    for index, (name, payload) in enumerate(assets):
        group = NAVIGATION_ASSET_SOURCES.get(name)
        if group is not None and group["format"] in navigation_groups:
            source_file = str(group["file"])
            manifest_assets.append(
                {
                    "index": index,
                    "name": name,
                    "size": len(payload),
                    "sha1": hashlib.sha1(payload).hexdigest(),
                    "source_file": source_file,
                    "source_format": group["format"],
                }
            )
            continue
        manifest_asset = write_asset_output(
            out_dir, index=index, name=name, payload=payload, source_root=source_root
        )
        if name == "kDialogue":
            # The tracked authority; the build compiles kDialogue from it directly.
            manifest_asset["source_file"] = "assets/dialogue/messages.toml"
            manifest_asset["source_format"] = dialogue_catalog.FORMAT_DIALOGUE_SOURCE
        manifest_assets.append(manifest_asset)
    return manifest_assets


def write_chr_source_sheets(out_dir: Path) -> list[dict[str, str]]:
    """Bootstrap the tracked `assets/chr/` sheet authority from the ROM.

    When the tracked sheets already exist they are the authority (possibly
    user-edited, guarded by chr.sha1) and extraction must not overwrite them;
    the manifest entries then just record the tracked files.
    """
    import chr_editable_sheets

    authority = chr_sheet_authority_dir(out_dir)
    if not authority.is_dir():
        try:
            chr_editable_sheets.write_editable_chr_sheets(out_dir, authority)
            print(
                f"bootstrapped tracked CHR sheet authority at {authority}; "
                "run `zelda3 --bless-chr` and commit it",
                file=sys.stderr,
            )
        except FileNotFoundError as exc:
            print(f"skipping editable CHR sheets: missing {exc.filename}", file=sys.stderr)
            return []

    entries = []
    for png_path in sorted(authority.glob("*.png")):
        manifest_path = png_path.with_suffix(".json")
        if not manifest_path.is_file():
            continue
        entries.append(
            {
                "sheet": png_path.stem,
                "image_file": png_path.relative_to(REPO_ROOT).as_posix(),
                "manifest_file": manifest_path.relative_to(REPO_ROOT).as_posix(),
                "source_format": "zelda3_editable_chr_sheet_v2",
            }
        )
    return entries


def write_rgba_variant_atlas(
    out_dir: Path,
    *,
    write_diagnostic_variants: bool = False,
    palettes_dir: Path | None = None,
) -> list[dict[str, str]]:
    if not write_diagnostic_variants:
        return []

    import rgba_variant_atlas

    try:
        written = rgba_variant_atlas.write_rom_variant_atlas(out_dir, palettes_dir=palettes_dir)
    except FileNotFoundError as exc:
        print(f"skipping RGBA variant atlas: missing {exc.filename}", file=sys.stderr)
        return []

    if not written:
        return []
    return [
        {
            "image_file": "atlas/tile_variants.png",
            "manifest_file": "atlas/tile_variants.json",
            "source_format": "zelda3_rgba_variant_atlas_v1",
        }
    ]


def write_base_effect_atlas(
    out_dir: Path, palettes_dir: Path | None = None
) -> list[dict[str, str]]:
    import rgba_variant_atlas

    try:
        written = rgba_variant_atlas.write_base_effect_atlas(
            out_dir,
            source_tiles_dir=REPO_ROOT / "zelda3-bin/developer_tilesets",
            palettes_dir=palettes_dir,
        )
    except FileNotFoundError as exc:
        print(f"skipping base/effect atlas: missing {exc.filename}", file=sys.stderr)
        return []

    if not written:
        return []
    return [
        {
            "image_file": "atlas/base_tiles.png",
            "manifest_file": "atlas/base_tiles.json",
            "effects_file": "atlas/tile_effects.json",
            "source_format": "zelda3_base_art_atlas_v1",
        }
    ]


def write_tile_effect_table(
    out_dir: Path, palettes_dir: Path | None = None
) -> list[dict[str, str]]:
    import rgba_variant_atlas

    try:
        written = rgba_variant_atlas.write_tile_effect_table(out_dir, palettes_dir=palettes_dir)
    except FileNotFoundError as exc:
        raise RuntimeError(
            f"required tile effect table input missing: {exc.filename}"
        ) from exc

    if not written:
        raise RuntimeError("required tile effect table was not written")
    return [
        {
            "effects_file": "atlas/tile_effects.json",
            "source_format": "zelda3_tile_effect_table_v1",
        }
    ]


def chr_sheet_authority_dir(out_dir: Path) -> Path:
    """The editable CHR sheets everything builds from: tracked `assets/chr/`.

    `write_chr_source_sheets` bootstraps the directory from the ROM on a
    fresh clone; after that the tracked sheets are the sole authority
    (`out_dir` is unused but kept for call-site stability).
    """
    del out_dir
    return REPO_ROOT / "assets/chr"


def write_canonical_art_atlas(
    out_dir: Path,
    chr_sheet_dir: Path | None = None,
    palettes_dir: Path | None = None,
) -> list[dict[str, str]]:
    import rgba_variant_atlas

    try:
        written = rgba_variant_atlas.write_canonical_art_atlas(
            out_dir,
            source_tiles_dir=REPO_ROOT / "zelda3-bin/developer_tilesets",
            chr_sheet_dir=chr_sheet_dir or chr_sheet_authority_dir(out_dir),
            palettes_dir=palettes_dir,
        )
    except FileNotFoundError as exc:
        raise RuntimeError(
            f"required canonical art atlas input missing: {exc.filename}"
        ) from exc

    if not written:
        raise RuntimeError("required canonical art atlas was not written")
    artifacts = [
        {
            "image_file": "atlas/art_tiles.png",
            "manifest_file": "atlas/art_tiles.json",
            "source_format": "zelda3_canonical_art_atlas_v1",
        },
        {
            "image_file": "atlas/dynamic_bg3_tiles.png",
            "manifest_file": "atlas/dynamic_bg3_tiles.json",
            "source_format": "zelda3_dynamic_bg3_art_atlas_v1",
        },
    ]
    if out_dir / "atlas/dialogue_glyph_tiles.json" in written:
        artifacts.append(
            {
                "image_file": "atlas/dialogue_glyph_tiles.png",
                "manifest_file": "atlas/dialogue_glyph_tiles.json",
                "source_format": "zelda3_dialogue_glyph_source_atlas_v1",
            }
        )
    if out_dir / "atlas/dialogue_vwf_font.json" in written:
        artifacts.append(
            {
                "manifest_file": "atlas/dialogue_vwf_font.json",
                "source_format": "zelda3_dialogue_vwf_font_v1",
            }
        )
    if out_dir / "atlas/dialogue_vwf_glyphs.json" in written:
        artifacts.append(
            {
                "image_file": "atlas/dialogue_vwf_glyphs.png",
                "manifest_file": "atlas/dialogue_vwf_glyphs.json",
                "source_format": "zelda3_dialogue_vwf_glyph_atlas_v1",
            }
        )
    return artifacts


def validate_canonical_art_atlas(out_dir: Path) -> dict[str, object]:
    import variant_atlas_summary

    atlas_dir = out_dir / "atlas"
    if not (atlas_dir / "art_tiles.json").is_file():
        raise RuntimeError(
            f"required canonical art atlas missing: {atlas_dir / 'art_tiles.json'}"
        )
    if not (atlas_dir / "tile_effects.json").is_file():
        raise RuntimeError(
            f"required tile effect table missing: {atlas_dir / 'tile_effects.json'}"
        )

    summary = variant_atlas_summary.summarize_variant_atlas(atlas_dir)
    errors = variant_atlas_summary.coverage_errors(summary)
    if errors:
        joined = "; ".join(errors)
        raise RuntimeError(f"canonical art atlas validation failed: {joined}")
    return summary


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
                # Junk sheets (0..11) contain copy commands past the produced
                # output; the runtime resolves those against its live WRAM
                # buffer, tools substitute zero like the port's Vec-based path.
                result.append(result[src] if src < len(result) else 0)
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
    palette_usage = preserve_palette_usage(out_dir)
    clean_generated_output(out_dir)
    restore_palette_usage(out_dir, palette_usage)
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

    manifest_assets = write_asset_outputs(out_dir, assets)
    spc_driver_program = write_spc_driver_program(out_dir, rom)
    chr_source_sheets = write_chr_source_sheets(out_dir)
    tile_effect_table = write_tile_effect_table(out_dir)
    base_effect_atlas = (
        write_base_effect_atlas(out_dir) if args.write_base_effect_atlas else []
    )
    canonical_art_atlas = write_canonical_art_atlas(out_dir)
    canonical_art_atlas_summary = validate_canonical_art_atlas(out_dir)
    rgba_variant_atlas = write_rgba_variant_atlas(
        out_dir,
        write_diagnostic_variants=args.write_diagnostic_variants,
    )
    manifest.write_text(
        json.dumps(
            {
                "asset_count": len(assets),
                "asset_key_signature": key_signature_path.name,
                "asset_signature": signature_path.name,
                "assets": manifest_assets,
                "base_effect_atlas": base_effect_atlas,
                "canonical_art_atlas": canonical_art_atlas,
                "canonical_art_atlas_summary": canonical_art_atlas_summary,
                "chr_source_sheets": chr_source_sheets,
                "rgba_variant_atlas": rgba_variant_atlas,
                "restool_pack_sha1": sha1(source_pack),
                "rom_sha1": sha1(rom),
                "source_tool": str(restool),
                "spc_driver_program": spc_driver_program,
                "tile_effect_table": tile_effect_table,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )
    bin_count = sum(1 for asset in manifest_assets if "file" in asset)
    source_count = sum(1 for asset in manifest_assets if "source_file" in asset)
    source_file_count = len(
        {asset["source_file"] for asset in manifest_assets if "source_file" in asset}
    )
    print(f"wrote {bin_count} binary asset files to {assets_dir}")
    if source_count:
        print(
            f"tracked {source_count} readable asset entries "
            f"across {source_file_count} source files under {REPO_ROOT / 'assets'} "
            "(bootstrap-only; existing tracked files preserved)"
        )
    if chr_source_sheets:
        print(f"tracked CHR sheet authority: {len(chr_source_sheets)} sheets at {REPO_ROOT / 'assets/chr'}")
    if base_effect_atlas:
        print(f"wrote legacy base/effect atlas to {out_dir / 'atlas'}")
    elif tile_effect_table:
        print(f"wrote tile effect table to {out_dir / 'atlas'}")
    if rgba_variant_atlas:
        print(f"wrote RGBA variant atlas to {out_dir / 'atlas'}")
    print(f"wrote {manifest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
