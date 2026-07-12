#!/usr/bin/env python3
"""Decode candidate Zelda 3 audio bytecode streams from the ROM audio catalog.

This is an offline import aid, not a runtime audio path. It consumes the
reviewable source catalog from `extract_rom_audio_catalog.py`, reconstructs the
SPC RAM images from the extracted asset pack, and performs a bounded bytecode
walk over candidate sequence targets to separate plausible streams from pointer
tables or raw data.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

import extract_rom_audio_catalog as source_catalog


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT_DIR = ROOT / "target" / "rom-audio-catalog"
CONTROL_ARITY = {
    0xE0: 1,
    0xE1: 1,
    0xE2: 1,
    0xE3: 3,
    0xE4: 0,
    0xE5: 1,
    0xE6: 0,
    0xE7: 1,
    0xE8: 1,
    0xE9: 2,
    0xEA: 1,
    0xEB: 2,
    0xEC: 0,
    0xED: 1,
    0xEE: 0,
    0xEF: 3,
    0xF0: 0,
    0xF1: 3,
    0xF2: 3,
    0xF3: 1,
    0xF4: 1,
    0xF5: 2,
    0xF6: 0,
    0xF7: 3,
    0xF8: 2,
    0xF9: 3,
    0xFA: 1,
    0xFB: 0,
    0xFC: 0,
    0xFD: 0,
    0xFE: 0,
    0xFF: 0,
}


@dataclass(frozen=True)
class DecodedBank:
    catalog_bank: dict
    ram: bytearray
    written_ranges: list[tuple[int, int]]


def read_u16(data: bytes | bytearray, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 2], "little")


def resolve_path(path_text: str, catalog_path: Path) -> Path:
    path = Path(path_text)
    if path.is_absolute():
        return path
    cwd_path = Path.cwd() / path
    if cwd_path.exists():
        return cwd_path
    return catalog_path.parent / path


def load_decoded_banks(catalog: dict, catalog_path: Path) -> list[DecodedBank]:
    asset_pack = resolve_path(catalog["source"]["asset_pack"], catalog_path)
    assets = source_catalog.parse_asset_pack(asset_pack)
    by_index = {asset.index: asset for asset in assets}
    banks = []
    for bank in catalog.get("sound_banks", []):
        if bank.get("status") != "parsed":
            continue
        asset = by_index.get(bank["asset_index"])
        if asset is None:
            continue
        ram = reconstruct_ram(asset.payload)
        ranges = [
            (item["start"], item["end_exclusive"])
            for item in bank.get("written_ranges", [])
            if item["start"] <= item["end_exclusive"]
        ]
        banks.append(DecodedBank(catalog_bank=bank, ram=ram, written_ranges=ranges))
    return banks


def reconstruct_ram(payload: bytes) -> bytearray:
    ram = bytearray(0x10000)
    cursor = 0
    while cursor + 4 <= len(payload):
        length = read_u16(payload, cursor)
        cursor += 2
        if length == 0:
            break
        target = read_u16(payload, cursor)
        cursor += 2
        block = payload[cursor : cursor + length]
        for index, value in enumerate(block):
            ram[(target + index) & 0xFFFF] = value
        cursor += length
    return ram


def in_ranges(address: int, ranges: list[tuple[int, int]]) -> bool:
    return any(start <= address < end for start, end in ranges)


def pointer_table_like(ram: bytearray, address: int, ranges: list[tuple[int, int]]) -> dict:
    values = []
    for offset in range(0, 24, 2):
        if address + offset + 2 > len(ram):
            break
        value = read_u16(ram, address + offset)
        if value == 0 or not in_ranges(value, ranges):
            continue
        values.append(value)
    monotonic_pairs = sum(1 for left, right in zip(values, values[1:]) if left <= right)
    return {
        "score": len(values),
        "monotonic_pairs": monotonic_pairs,
        "targets": values[:8],
        "is_pointer_like": len(values) >= 4 and monotonic_pairs >= max(2, len(values) - 2),
    }


def is_route_provenance_candidate(candidate: dict) -> bool:
    return candidate.get("confidence") == "route_sequence_provenance_target"


def decode_candidate(
    ram: bytearray,
    ranges: list[tuple[int, int]],
    candidate: dict,
    *,
    max_bytes: int,
    max_events: int,
) -> dict:
    address = int(candidate["address"])
    pointer_like = pointer_table_like(ram, address, ranges)
    if is_route_provenance_candidate(candidate):
        sfx_decoded = decode_sfx_candidate(
            ram,
            candidate,
            pointer_like,
            max_bytes=max_bytes,
            max_events=max_events,
        )
        if sfx_decoded["status"] == "decoded":
            return sfx_decoded
    if pointer_like["is_pointer_like"]:
        return rejected_candidate(candidate, "pointer_table_like", pointer_like)

    cursor = address
    end = min(len(ram), address + max_bytes)
    events = []
    counts = {"notes": 0, "rests": 0, "controls": 0, "terminators": 0}
    status = "unterminated"
    while cursor < end and len(events) < max_events:
        opcode = ram[cursor]
        event_address = cursor
        cursor += 1
        if opcode == 0x00:
            counts["terminators"] += 1
            events.append({"address": event_address, "kind": "terminator", "opcode": opcode})
            status = "terminated"
            break
        if opcode < 0x80:
            counts["notes"] += 1
            events.append({"address": event_address, "kind": "note_or_duration", "opcode": opcode})
            continue
        if opcode < 0xE0:
            counts["notes"] += 1
            events.append({"address": event_address, "kind": "note_or_duration", "opcode": opcode})
            continue
        arity = CONTROL_ARITY.get(opcode, 1)
        if cursor + arity > end:
            status = "truncated_control"
            events.append(
                {
                    "address": event_address,
                    "kind": "control",
                    "opcode": opcode,
                    "args": list(ram[cursor:end]),
                    "truncated": True,
                }
            )
            break
        args = list(ram[cursor : cursor + arity])
        cursor += arity
        counts["controls"] += 1
        events.append({"address": event_address, "kind": "control", "opcode": opcode, "args": args})
        if opcode in {0xF6, 0xFF}:
            status = "loop_or_stop"
            break

    confidence, reasons = classify_decoded_stream(status, counts, events)
    return {
        "technical_name": candidate["technical_name"],
        "address": address,
        "source_table": candidate["source_table"],
        "status": "decoded" if confidence != "rejected" else "rejected",
        "confidence": confidence,
        "reject_reasons": reasons,
        "bytes_consumed": max(0, cursor - address),
        "event_counts": counts,
        "events_preview": events[:32],
        "pointer_like": pointer_like,
        "decoder_kind": "music_table",
        "sha1_64": candidate.get("sha1_64"),
    }


def decode_sfx_candidate(
    ram: bytearray,
    candidate: dict,
    pointer_like: dict,
    *,
    max_bytes: int,
    max_events: int,
) -> dict:
    address = int(candidate["address"])
    cursor = address
    end = min(len(ram), address + max_bytes)
    events = []
    counts = {"notes": 0, "rests": 0, "controls": 0, "terminators": 0}
    status = "unterminated"
    malformed = False

    while cursor < end and len(events) < max_events:
        opcode = ram[cursor]
        event_address = cursor
        cursor += 1
        if opcode == 0x00:
            counts["terminators"] += 1
            events.append({"address": event_address, "kind": "terminator", "opcode": opcode})
            status = "terminated"
            break
        if opcode < 0x80:
            counts["controls"] += 1
            event = {"address": event_address, "kind": "sfx_length", "opcode": opcode}
            if cursor < end and ram[cursor] < 0x80:
                event["volume"] = ram[cursor]
                cursor += 1
                counts["controls"] += 1
            events.append(event)
            continue
        if opcode == 0xE0:
            if cursor >= end:
                malformed = True
                status = "truncated_control"
                events.append(
                    {
                        "address": event_address,
                        "kind": "sfx_instrument",
                        "opcode": opcode,
                        "args": [],
                        "truncated": True,
                    }
                )
                break
            instrument = ram[cursor]
            cursor += 1
            counts["controls"] += 1
            events.append(
                {
                    "address": event_address,
                    "kind": "sfx_instrument",
                    "opcode": opcode,
                    "instrument": instrument,
                }
            )
            continue
        if opcode == 0xF9:
            arity = 4
            if cursor + arity > end:
                malformed = True
                status = "truncated_control"
                events.append(
                    {
                        "address": event_address,
                        "kind": "sfx_pitch_slide",
                        "opcode": opcode,
                        "args": list(ram[cursor:end]),
                        "truncated": True,
                    }
                )
                break
            note, delay, length, target = ram[cursor : cursor + arity]
            cursor += arity
            counts["notes"] += 1
            counts["controls"] += 1
            events.append(
                {
                    "address": event_address,
                    "kind": "sfx_pitch_slide",
                    "opcode": opcode,
                    "note": note,
                    "delay": delay,
                    "length": length,
                    "target": target,
                }
            )
            status = "effect_started"
            break
        if opcode == 0xF1:
            arity = 3
            if cursor + arity > end:
                malformed = True
                status = "truncated_control"
                events.append(
                    {
                        "address": event_address,
                        "kind": "sfx_pitch_slide",
                        "opcode": opcode,
                        "args": list(ram[cursor:end]),
                        "truncated": True,
                    }
                )
                break
            delay, length, target = ram[cursor : cursor + arity]
            cursor += arity
            counts["controls"] += 1
            events.append(
                {
                    "address": event_address,
                    "kind": "sfx_pitch_slide",
                    "opcode": opcode,
                    "delay": delay,
                    "length": length,
                    "target": target,
                }
            )
            status = "effect_started"
            break
        if opcode == 0xFF:
            counts["controls"] += 1
            events.append({"address": event_address, "kind": "sfx_loop", "opcode": opcode})
            status = "loop_or_stop"
            break
        counts["notes"] += 1
        events.append({"address": event_address, "kind": "sfx_note", "opcode": opcode})

    confidence, reasons = classify_sfx_stream(status, counts, events, malformed)
    return {
        "technical_name": candidate["technical_name"],
        "address": address,
        "source_table": candidate["source_table"],
        "status": "decoded" if confidence != "rejected" else "rejected",
        "confidence": confidence,
        "reject_reasons": reasons,
        "bytes_consumed": max(0, cursor - address),
        "event_counts": counts,
        "events_preview": events[:32],
        "pointer_like": pointer_like,
        "decoder_kind": "sfx_compact",
        "sha1_64": candidate.get("sha1_64"),
    }


def rejected_candidate(candidate: dict, reason: str, pointer_like: dict) -> dict:
    return {
        "technical_name": candidate["technical_name"],
        "address": candidate["address"],
        "source_table": candidate["source_table"],
        "status": "rejected",
        "confidence": "rejected",
        "reject_reasons": [reason],
        "bytes_consumed": 0,
        "event_counts": {"notes": 0, "rests": 0, "controls": 0, "terminators": 0},
        "events_preview": [],
        "pointer_like": pointer_like,
        "decoder_kind": "none",
        "sha1_64": candidate.get("sha1_64"),
    }


def classify_decoded_stream(status: str, counts: dict, events: list[dict]) -> tuple[str, list[str]]:
    reasons = []
    if not events:
        return "rejected", ["empty_stream"]
    if counts["notes"] + counts["controls"] < 4:
        reasons.append("too_few_events")
    if counts["controls"] == 0 and counts["notes"] < 8:
        reasons.append("no_controls_and_short")
    if status == "truncated_control":
        reasons.append("truncated_control")
    if reasons:
        return "rejected", reasons
    if status in {"terminated", "loop_or_stop"} and counts["controls"] >= 1:
        return "high", []
    if counts["controls"] >= 2 and counts["notes"] >= 4:
        return "medium", []
    return "low", []


def classify_sfx_stream(
    status: str,
    counts: dict,
    events: list[dict],
    malformed: bool,
) -> tuple[str, list[str]]:
    reasons = []
    if not events:
        return "rejected", ["empty_stream"]
    if malformed:
        reasons.append(status)
    if counts["notes"] == 0:
        reasons.append("no_notes")
    if status == "unterminated":
        reasons.append("unterminated")
    if reasons:
        return "rejected", reasons
    if status == "terminated" and counts["controls"] >= 2:
        return "high", []
    if status in {"effect_started", "loop_or_stop"} and counts["controls"] >= 1:
        return "medium", []
    if status == "terminated" and counts["notes"] >= 1:
        return "low", []
    return "rejected", ["weak_sfx_shape"]


def decode_catalog(catalog: dict, catalog_path: Path, args: argparse.Namespace) -> dict:
    decoded_banks = load_decoded_banks(catalog, catalog_path)
    banks = []
    for decoded_bank in decoded_banks:
        decoded = [
            decode_candidate(
                decoded_bank.ram,
                decoded_bank.written_ranges,
                candidate,
                max_bytes=args.max_bytes,
                max_events=args.max_events,
            )
            for candidate in decoded_bank.catalog_bank.get("candidate_sequences", [])
        ]
        banks.append(
            {
                "asset_index": decoded_bank.catalog_bank["asset_index"],
                "asset_name": decoded_bank.catalog_bank["asset_name"],
                "role": decoded_bank.catalog_bank["role"],
                "candidate_count": len(decoded),
                "decoded_count": sum(1 for item in decoded if item["status"] == "decoded"),
                "high_confidence_count": sum(1 for item in decoded if item["confidence"] == "high"),
                "medium_confidence_count": sum(1 for item in decoded if item["confidence"] == "medium"),
                "low_confidence_count": sum(1 for item in decoded if item["confidence"] == "low"),
                "rejected_count": sum(1 for item in decoded if item["status"] == "rejected"),
                "sequences": decoded,
            }
        )
    route_links = route_cross_links(catalog, banks)
    return {
        "format": "zelda3_decoded_audio_sequences_v1",
        "source_catalog": str(catalog_path),
        "runtime_dependency": False,
        "decoder": {
            "max_bytes": args.max_bytes,
            "max_events": args.max_events,
            "note": "bounded offline bytecode walk; not SPC/APU emulation",
        },
        "banks": banks,
        "route_cross_links": route_links,
        "coverage": {
            "candidate_sequences": sum(bank["candidate_count"] for bank in banks),
            "decoded_sequences": sum(bank["decoded_count"] for bank in banks),
            "high_confidence": sum(bank["high_confidence_count"] for bank in banks),
            "medium_confidence": sum(bank["medium_confidence_count"] for bank in banks),
            "low_confidence": sum(bank["low_confidence_count"] for bank in banks),
            "rejected": sum(bank["rejected_count"] for bank in banks),
            "route_programs": len(route_links),
            "direct_route_links": sum(1 for link in route_links if link["status"] == "linked"),
        },
    }


def route_cross_links(catalog: dict, banks: list[dict]) -> list[dict]:
    by_address = {}
    by_raw_address: dict[int, list[dict]] = {}
    for bank in banks:
        for sequence in bank["sequences"]:
            if sequence["status"] == "decoded":
                by_address[(bank["asset_index"], sequence["address"])] = sequence
                by_raw_address.setdefault(sequence["address"], []).append(
                    {**sequence, "sound_bank_asset_index": bank["asset_index"]}
                )
    links = []
    route = catalog.get("route_harvest")
    if route is None:
        return links
    for program in route.get("programs", []):
        direct_addresses = []
        for key in ("rom_sequence_address", "sequence_address"):
            if isinstance(program.get(key), int):
                bank_index = program.get("sound_bank_asset_index", program.get("bank"))
                if isinstance(bank_index, int):
                    direct_addresses.append((bank_index, program[key]))
        for item in program.get("sequence_provenance", []):
            if not isinstance(item, dict):
                continue
            address = item.get("rom_sequence_address", item.get("sequence_address"))
            if not isinstance(address, int):
                continue
            bank_index = item.get("sound_bank_asset_index")
            if isinstance(bank_index, int):
                direct_addresses.append((bank_index, address))
            else:
                direct_addresses.append((None, address))
        matches = []
        for bank_index, address in direct_addresses:
            candidates = []
            if bank_index is None:
                candidates = by_raw_address.get(address, [])
            else:
                sequence = by_address.get((bank_index, address))
                if sequence is not None:
                    candidates = [{**sequence, "sound_bank_asset_index": bank_index}]
            for sequence in candidates:
                matches.append(
                    {
                        "sound_bank_asset_index": sequence["sound_bank_asset_index"],
                        "address": address,
                        "technical_name": sequence["technical_name"],
                        "confidence": sequence["confidence"],
                    }
                )
        unique_matches = []
        seen = set()
        for match in matches:
            key = (match["sound_bank_asset_index"], match["address"], match["technical_name"])
            if key in seen:
                continue
            seen.add(key)
            unique_matches.append(match)
        if unique_matches:
            notes = []
        elif direct_addresses:
            notes = [
                "route harvest carried sequence addresses, but no decoded sequence matched them; "
                "check rejected route-provenance candidates before promotion"
            ]
        else:
            notes = [
                "route harvest does not yet carry ROM sequence addresses; "
                "decoded sequence mapping requires provenance from the audio command loader"
            ]
        links.append(
            {
                "bank": program.get("bank"),
                "id": program.get("id"),
                "name": program.get("name"),
                "variant_count": program.get("variant_count", 0),
                "status": "linked" if unique_matches else "unlinked",
                "matches": unique_matches,
                "notes": notes,
            }
        )
    return links


def render_markdown(decoded: dict) -> str:
    lines = [
        "# Decoded ROM Audio Sequences",
        "",
        f"- Source catalog: `{decoded['source_catalog']}`",
        "- Runtime dependency: `false`",
        "- Decoder: bounded offline bytecode walk; not SPC/APU emulation",
        "",
        "| Asset | Role | Candidates | Decoded | High | Medium | Low | Rejected |",
        "|---:|---|---:|---:|---:|---:|---:|---:|",
    ]
    for bank in decoded["banks"]:
        lines.append(
            f"| {bank['asset_index']} | {bank['role']} | {bank['candidate_count']} | "
            f"{bank['decoded_count']} | {bank['high_confidence_count']} | "
            f"{bank['medium_confidence_count']} | {bank['low_confidence_count']} | "
            f"{bank['rejected_count']} |"
        )
    lines.extend(["", "## Top Decoded Sequences", ""])
    for bank in decoded["banks"]:
        lines.append(f"### {bank['asset_index']} {bank['role']}")
        lines.append("")
        lines.append("| Name | Address | Confidence | Events | Source table |")
        lines.append("|---|---:|---|---:|---:|")
        shown = 0
        for sequence in bank["sequences"]:
            if sequence["status"] != "decoded":
                continue
            events = sequence["event_counts"]["notes"] + sequence["event_counts"]["controls"]
            lines.append(
                f"| `{sequence['technical_name']}` | 0x{sequence['address']:04x} | "
                f"{sequence['confidence']} | {events} | 0x{sequence['source_table']:04x} |"
            )
            shown += 1
            if shown >= 12:
                break
        if shown == 0:
            lines.append("|  |  | no decoded sequences |  |  |")
        lines.append("")
    if decoded["route_cross_links"]:
        lines.extend(
            [
                "## Route Harvest Links",
                "",
                "| Bank | Id | Name | Variants | Status | Notes |",
                "|---:|---:|---|---:|---|---|",
            ]
        )
        for link in decoded["route_cross_links"]:
            notes = "; ".join(link.get("notes", []))
            lines.append(
                f"| {link['bank']} | 0x{link['id']:02x} | `{link['name']}` | "
                f"{link['variant_count']} | {link['status']} | {notes} |"
            )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog_json", type=Path)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--report-out", type=Path)
    parser.add_argument("--max-bytes", type=int, default=192)
    parser.add_argument("--max-events", type=int, default=96)
    args = parser.parse_args(argv)
    if not args.catalog_json.is_file():
        parser.error(f"catalog_json does not exist: {args.catalog_json}")
    if args.max_bytes <= 0:
        parser.error("--max-bytes must be greater than zero")
    if args.max_events <= 0:
        parser.error("--max-events must be greater than zero")
    if args.json_out is None:
        args.json_out = args.output_dir / "decoded-sequences.json"
    if args.report_out is None:
        args.report_out = args.output_dir / "decoded-sequences.md"
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        catalog = json.loads(args.catalog_json.read_text(encoding="utf-8"))
        decoded = decode_catalog(catalog, args.catalog_json, args)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(decoded, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.report_out.parent.mkdir(parents=True, exist_ok=True)
    args.report_out.write_text(render_markdown(decoded), encoding="utf-8")
    coverage = decoded["coverage"]
    print(
        "decoded ROM audio sequences: "
        f"decoded={coverage['decoded_sequences']}/{coverage['candidate_sequences']} "
        f"high={coverage['high_confidence']} medium={coverage['medium_confidence']} "
        f"low={coverage['low_confidence']} rejected={coverage['rejected']}"
    )
    print(f"json: {args.json_out}")
    print(f"report: {args.report_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
