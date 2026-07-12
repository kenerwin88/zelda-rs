#!/usr/bin/env python3
"""Extract reviewable modern-audio source facts from Zelda 3 audio assets.

This is an offline bridge: it reads the generated asset pack that was extracted
from the ROM and emits JSON/Markdown catalogs for review. The modern runtime
must consume generated modern assets, not ROM/SPC/DSP structures directly.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ASSET_PACK = ROOT / "zelda3_assets.dat"
DEFAULT_OUTPUT_DIR = ROOT / "target" / "rom-audio-catalog"
ASSET_SIGNATURE_PREFIX = b"Zelda3_v0     \n\0"
SOUND_BANK_ASSETS = [
    (0, "intro_overworld", "kSoundBank_intro"),
    (1, "indoor_dungeon", "kSoundBank_indoor"),
    (2, "ending_credits", "kSoundBank_ending"),
]


@dataclass(frozen=True)
class Asset:
    index: int
    name: str
    payload: bytes
    payload_offset: int


def read_u16(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little")


def read_u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little")


def sha1(data: bytes) -> str:
    return hashlib.sha1(data).hexdigest()


def parse_asset_pack(path: Path) -> list[Asset]:
    data = path.read_bytes()
    if len(data) < 88 or data[:16] != ASSET_SIGNATURE_PREFIX:
        raise ValueError(f"{path} is not a valid zelda3_assets.dat")
    count = read_u32(data, 80)
    key_signature_len = read_u32(data, 84)
    sizes_start = 88
    key_signature_start = sizes_start + count * 4
    payload_offset = key_signature_start + key_signature_len
    if payload_offset > len(data):
        raise ValueError(f"{path} asset header extends past end of file")
    names = data[key_signature_start:payload_offset].rstrip(b"\0").decode("utf-8").split("\0")
    if len(names) != count:
        raise ValueError(f"{path} has {len(names)} asset names, expected {count}")

    assets: list[Asset] = []
    offset = payload_offset
    for index, name in enumerate(names):
        size = read_u32(data, sizes_start + index * 4)
        offset = (offset + 3) & ~3
        end = offset + size
        if end > len(data):
            raise ValueError(f"{path} asset {index} extends past end of file")
        assets.append(Asset(index=index, name=name, payload=data[offset:end], payload_offset=offset))
        offset = end
    return assets


def parse_song_bank(asset: Asset, role: str) -> dict:
    ram = bytearray(0x10000)
    cursor = 0
    blocks = []
    written_ranges: list[tuple[int, int]] = []
    while cursor + 4 <= len(asset.payload):
        block_offset = cursor
        length = read_u16(asset.payload, cursor)
        cursor += 2
        if length == 0:
            break
        target = read_u16(asset.payload, cursor)
        cursor += 2
        end = cursor + length
        if end > len(asset.payload):
            raise ValueError(
                f"asset {asset.index} {asset.name} malformed block at {block_offset}: "
                f"target=${target:04x} length={length} remaining={len(asset.payload) - cursor}"
            )
        block = asset.payload[cursor:end]
        for pos, value in enumerate(block):
            ram[(target + pos) & 0xFFFF] = value
        written_start = target & 0xFFFF
        written_end = (target + length - 1) & 0xFFFF
        blocks.append(
            {
                "offset": block_offset,
                "target": target,
                "length": length,
                "end": written_end,
                "sha1": sha1(block),
            }
        )
        if target + length <= 0x10000:
            written_ranges.append((target, target + length))
        cursor = end

    merged_ranges = merge_ranges(written_ranges)
    pointer_tables = find_pointer_tables(ram, merged_ranges)
    candidate_sequences = sequence_candidates_from_tables(ram, pointer_tables)
    return {
        "asset_index": asset.index,
        "asset_name": asset.name,
        "role": role,
        "payload_size": len(asset.payload),
        "payload_sha1": sha1(asset.payload),
        "payload_offset": asset.payload_offset,
        "blocks": blocks,
        "parse_end": cursor,
        "written_ranges": [
            {"start": start, "end_exclusive": end, "size": end - start}
            for start, end in merged_ranges
        ],
        "reset_vector": read_u16(ram, 0xFFFE),
        "pointer_tables": pointer_tables,
        "candidate_sequences": candidate_sequences,
    }


def merge_ranges(ranges: list[tuple[int, int]]) -> list[tuple[int, int]]:
    if not ranges:
        return []
    merged = []
    for start, end in sorted(ranges):
        if not merged or start > merged[-1][1]:
            merged.append([start, end])
        else:
            merged[-1][1] = max(merged[-1][1], end)
    return [(start, end) for start, end in merged]


def in_ranges(value: int, ranges: list[tuple[int, int]]) -> bool:
    return any(start <= value < end for start, end in ranges)


def in_catalog_ranges(value: int, bank: dict) -> bool:
    return any(
        item["start"] <= value < item["end_exclusive"]
        for item in bank.get("written_ranges", [])
    )


def find_pointer_tables(
    ram: bytearray,
    written_ranges: list[tuple[int, int]],
    *,
    min_entries: int = 4,
    max_tables: int = 80,
) -> list[dict]:
    tables = []
    cursor = 0
    while cursor + min_entries * 2 <= len(ram):
        values = []
        pos = cursor
        while pos + 2 <= len(ram):
            value = read_u16(ram, pos)
            if value == 0 or not in_ranges(value, written_ranges):
                break
            values.append(value)
            pos += 2
        if len(values) >= min_entries:
            unique_values = sorted(set(values))
            tables.append(
                {
                    "address": cursor,
                    "entries": len(values),
                    "unique_entries": len(unique_values),
                    "first_targets": unique_values[:16],
                    "target_span": {
                        "min": min(unique_values),
                        "max": max(unique_values),
                    },
                }
            )
            cursor = pos
            if len(tables) >= max_tables:
                break
        else:
            cursor += 2
    return tables


def sequence_candidates_from_tables(ram: bytearray, tables: list[dict], *, max_per_table: int = 16) -> list[dict]:
    candidates = []
    seen: set[int] = set()
    for table in tables:
        for target in table["first_targets"][:max_per_table]:
            if target in seen:
                continue
            seen.add(target)
            window = bytes(ram[target : min(target + 64, len(ram))])
            candidates.append(
                {
                    "technical_name": f"spc_seq_{target:04x}",
                    "address": target,
                    "source_table": table["address"],
                    "preview": list(window[:24]),
                    "sha1_64": sha1(window),
                    "curated_name": None,
                    "confidence": "candidate_pointer_target",
                }
            )
    return candidates


def route_harvest_summary(path: Path | None) -> dict | None:
    if path is None:
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    programs = []
    for program in data.get("programs", []):
        summary = {
            "bank": program.get("bank"),
            "id": program.get("id"),
            "status": program.get("status"),
            "occurrences": program.get("occurrences"),
            "variant_count": program.get("variant_count", 1 if program.get("steps") else 0),
            "first_frames": program.get("first_frames", []),
            "name": program.get("name"),
        }
        for key in ("sound_bank_asset_index", "rom_sequence_address", "sequence_address"):
            if isinstance(program.get(key), int):
                summary[key] = program[key]
        if isinstance(program.get("sequence_provenance"), list):
            summary["sequence_provenance"] = program["sequence_provenance"]
        programs.append(summary)
    return {
        "source": str(path),
        "coverage": data.get("coverage", {}),
        "programs": programs,
    }


def add_route_provenance_candidates(banks: list[dict], route: dict | None) -> None:
    if route is None:
        return
    addresses = sorted(
        {
            item["rom_sequence_address"] & 0xFFFF
            for program in route.get("programs", [])
            for item in program.get("sequence_provenance", [])
            if isinstance(item, dict) and isinstance(item.get("rom_sequence_address"), int)
        }
    )
    if not addresses:
        return
    for bank in banks:
        if bank.get("status") != "parsed":
            continue
        existing = {candidate["address"] for candidate in bank.get("candidate_sequences", [])}
        for address in addresses:
            if address in existing or not in_catalog_ranges(address, bank):
                continue
            bank["candidate_sequences"].append(
                {
                    "technical_name": f"spc_seq_{address:04x}",
                    "address": address,
                    "source_table": address,
                    "preview": [],
                    "sha1_64": None,
                    "curated_name": None,
                    "confidence": "route_sequence_provenance_target",
                }
            )
            existing.add(address)


def build_catalog(args: argparse.Namespace) -> dict:
    assets = parse_asset_pack(args.asset_pack)
    by_index = {asset.index: asset for asset in assets}
    banks = []
    for index, role, expected_name in SOUND_BANK_ASSETS:
        asset = by_index.get(index)
        if asset is None:
            banks.append({"asset_index": index, "role": role, "status": "missing"})
            continue
        bank = parse_song_bank(asset, role)
        bank["expected_asset_name"] = expected_name
        bank["status"] = "parsed"
        banks.append(bank)

    route_harvest = route_harvest_summary(args.route_harvest_json)
    add_route_provenance_candidates(banks, route_harvest)

    catalog = {
        "format": "zelda3_modern_audio_source_catalog_v1",
        "source": {
            "asset_pack": str(args.asset_pack),
            "rom": str(args.rom) if args.rom is not None else None,
            "runtime_dependency": False,
            "boundary": "offline source extraction only; modern runtime consumes generated catalog assets",
        },
        "asset_pack": {
            "asset_count": len(assets),
            "sound_bank_asset_indices": [index for index, _, _ in SOUND_BANK_ASSETS],
        },
        "sound_banks": banks,
        "route_harvest": route_harvest,
    }
    return catalog


def render_markdown(catalog: dict) -> str:
    lines = [
        "# ROM Audio Source Catalog",
        "",
        f"- Asset pack: `{catalog['source']['asset_pack']}`",
        f"- ROM: `{catalog['source']['rom']}`",
        "- Runtime dependency: `false`",
        "- Boundary: offline source extraction only; modern runtime consumes generated catalog assets",
        "",
        "| Asset | Role | Payload bytes | Blocks | Written ranges | Pointer tables | Sequence candidates |",
        "|---:|---|---:|---:|---:|---:|---:|",
    ]
    for bank in catalog["sound_banks"]:
        if bank.get("status") != "parsed":
            lines.append(f"| {bank['asset_index']} | {bank['role']} | missing |  |  |  |  |")
            continue
        lines.append(
            f"| {bank['asset_index']} | {bank['role']} | {bank['payload_size']} | "
            f"{len(bank['blocks'])} | {len(bank['written_ranges'])} | "
            f"{len(bank['pointer_tables'])} | {len(bank['candidate_sequences'])} |"
        )
    route = catalog.get("route_harvest")
    if route is not None:
        coverage = route.get("coverage", {})
        lines.extend(
            [
                "",
                "## Route Harvest Cross-Link",
                "",
                f"- Source: `{route['source']}`",
                f"- Focused commands: {coverage.get('focused_commands', coverage.get('commands', 0))}",
                f"- Programs: {coverage.get('programs', 0)}",
                f"- Lifted: {coverage.get('lifted', 0)}",
                f"- Gaps: {coverage.get('gaps', 0)}",
                "",
                "| Bank | Id | Status | Occurrences | Variants | First frames |",
                "|---:|---:|---|---:|---:|---|",
            ]
        )
        for program in route.get("programs", []):
            frames = ", ".join(str(frame) for frame in program.get("first_frames", []))
            lines.append(
                f"| {program['bank']} | 0x{program['id']:02x} | {program['status']} | "
                f"{program['occurrences']} | {program['variant_count']} | {frames} |"
            )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-pack", type=Path, default=DEFAULT_ASSET_PACK)
    parser.add_argument("--rom", type=Path)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--report-out", type=Path)
    parser.add_argument("--route-harvest-json", type=Path)
    args = parser.parse_args(argv)
    if not args.asset_pack.is_file():
        parser.error(f"--asset-pack does not exist: {args.asset_pack}")
    if args.rom is not None and not args.rom.is_file():
        parser.error(f"--rom does not exist: {args.rom}")
    if args.route_harvest_json is not None and not args.route_harvest_json.is_file():
        parser.error(f"--route-harvest-json does not exist: {args.route_harvest_json}")
    if args.json_out is None:
        args.json_out = args.output_dir / "rom-audio-catalog.json"
    if args.report_out is None:
        args.report_out = args.output_dir / "rom-audio-catalog.md"
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        catalog = build_catalog(args)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(catalog, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.report_out.parent.mkdir(parents=True, exist_ok=True)
    args.report_out.write_text(render_markdown(catalog), encoding="utf-8")
    banks = [bank for bank in catalog["sound_banks"] if bank.get("status") == "parsed"]
    print(
        "rom audio catalog: "
        f"banks={len(banks)} "
        f"blocks={sum(len(bank['blocks']) for bank in banks)} "
        f"pointer_tables={sum(len(bank['pointer_tables']) for bank in banks)} "
        f"candidates={sum(len(bank['candidate_sequences']) for bank in banks)}"
    )
    print(f"json: {args.json_out}")
    print(f"report: {args.report_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
