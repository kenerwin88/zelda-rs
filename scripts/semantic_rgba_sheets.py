#!/usr/bin/env python3
"""Export and compile semantic RGBA sheets for Zelda 3 atlas variants."""

from __future__ import annotations

from dataclasses import dataclass
import argparse
import json
from pathlib import Path


SEMANTIC_SHEET_COLUMNS = 128


@dataclass(frozen=True)
class SemanticFrame:
    id: str
    source_rect: tuple[int, int, int, int]
    emits: list[str]


@dataclass(frozen=True)
class AtlasVariantEntry:
    id: str
    source_kind: str
    asset: str
    pack: int
    tile: int
    bpp: int
    palette: str
    palette_row: int
    rect: tuple[int, int, int, int]


def _atlas_paths(asset_dir: Path) -> tuple[Path, Path]:
    return asset_dir / "atlas/tile_variants.json", asset_dir / "atlas/tile_variants.png"


def _read_variant_entries(asset_dir: Path) -> list[AtlasVariantEntry]:
    manifest_path, _image_path = _atlas_paths(asset_dir)
    data = json.loads(manifest_path.read_text())
    if data.get("format") != "zelda3_rgba_variant_atlas_v1":
        raise ValueError(f"{manifest_path}: unsupported atlas format {data.get('format')!r}")

    entries: list[AtlasVariantEntry] = []
    for entry in data.get("entries", []):
        rect = entry["rect"]
        entries.append(
            AtlasVariantEntry(
                id=str(entry["id"]),
                source_kind=str(entry["source_kind"]),
                asset=str(entry["asset"]),
                pack=int(entry["pack"]),
                tile=int(entry["tile"]),
                bpp=int(entry["bpp"]),
                palette=str(entry["palette"]),
                palette_row=int(entry["palette_row"]),
                rect=(int(rect[0]), int(rect[1]), int(rect[2]), int(rect[3])),
            )
        )
    return entries


def _semantic_subdir(source_kind: str) -> str:
    if source_kind == "sprite":
        return "sprites"
    if source_kind == "bg":
        return "backgrounds"
    return source_kind


def _safe_name(value: str) -> str:
    return "".join(ch if ch.isalnum() else "_" for ch in value).strip("_")


def _sheet_stem(source_kind: str, asset: str, pack: int) -> str:
    return f"{_safe_name(source_kind)}_{_safe_name(asset)}_pack{pack}"


def _frame_id(entry: AtlasVariantEntry) -> str:
    return (
        f"{_sheet_stem(entry.source_kind, entry.asset, entry.pack)}_"
        f"tile{entry.tile}_{_safe_name(entry.palette)}_row{entry.palette_row}"
    )


def _group_entries(
    entries: list[AtlasVariantEntry],
) -> dict[tuple[str, str, int], list[AtlasVariantEntry]]:
    grouped: dict[tuple[str, str, int], list[AtlasVariantEntry]] = {}
    for entry in entries:
        grouped.setdefault((entry.source_kind, entry.asset, entry.pack), []).append(entry)
    for group in grouped.values():
        group.sort(key=lambda entry: (entry.tile, entry.palette, entry.palette_row, entry.id))
    return dict(sorted(grouped.items()))


def write_initial_semantic_sheets(asset_dir: Path, out_dir: Path | None = None) -> list[Path]:
    from PIL import Image

    entries = _read_variant_entries(asset_dir)
    _manifest_path, image_path = _atlas_paths(asset_dir)
    with Image.open(image_path) as image:
        atlas = image.convert("RGBA")
        destination = out_dir or asset_dir / "assets_src/semantic"
        written: list[Path] = []
        for (source_kind, asset, pack), group in _group_entries(entries).items():
            columns = min(SEMANTIC_SHEET_COLUMNS, max(1, len(group)))
            rows = (len(group) + columns - 1) // columns
            width = columns * 8
            height = rows * 8
            sheet = Image.new("RGBA", (width, height))
            frames: list[SemanticFrame] = []
            for index, entry in enumerate(group):
                src_x, src_y, src_w, src_h = entry.rect
                dst_x = (index % columns) * 8
                dst_y = (index // columns) * 8
                crop = atlas.crop((src_x, src_y, src_x + src_w, src_y + src_h))
                sheet.paste(crop, (dst_x, dst_y))
                frames.append(
                    SemanticFrame(
                        id=_frame_id(entry),
                        source_rect=(dst_x, dst_y, src_w, src_h),
                        emits=[entry.id],
                    )
                )

            group_dir = destination / _semantic_subdir(source_kind)
            group_dir.mkdir(parents=True, exist_ok=True)
            stem = _sheet_stem(source_kind, asset, pack)
            png_path = group_dir / f"{stem}.png"
            json_path = group_dir / f"{stem}.json"
            sheet.save(png_path)
            manifest = {
                "format": "zelda3_semantic_rgba_sheet_v1",
                "source_kind": source_kind,
                "asset": asset,
                "pack": pack,
                "tile_width": 8,
                "tile_height": 8,
                "image_file": png_path.name,
                "frames": [
                    {
                        "id": frame.id,
                        "source_rect": list(frame.source_rect),
                        "emits": frame.emits,
                    }
                    for frame in frames
                ],
            }
            json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
            written.extend([png_path, json_path])
        return written


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--asset-dir",
        required=True,
        type=Path,
        help="Extracted asset directory containing atlas/tile_variants.json and .png",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="Semantic sheet output directory; defaults to ASSET_DIR/assets_src/semantic",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    written = write_initial_semantic_sheets(args.asset_dir, out_dir=args.out_dir)
    for path in written:
        print(path)


if __name__ == "__main__":
    main()
