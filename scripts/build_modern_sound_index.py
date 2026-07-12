#!/usr/bin/env python3
"""Build a reviewable modern sound index from decoded sequences and route harvests.

This is an offline authoring aid. It joins `decoded-sequences.json` with
`modern-sfx-harvest.json` sequence provenance and emits a stable index of
route commands to decoded ROM sequence candidates. The modern runtime should
consume reviewed/generated modern assets, not this ROM-derived index directly.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DECODED = ROOT / "target" / "rom-audio-catalog" / "decoded-sequences.json"
DEFAULT_HARVEST = ROOT / "target" / "modern-sfx-harvest-variants-40" / "modern-sfx-harvest.json"
DEFAULT_OUTPUT_DIR = ROOT / "target" / "modern-sound-index"


def decoded_sequence_lookup(decoded: dict) -> dict:
    by_asset_and_address = {}
    by_address: dict[int, list[dict]] = {}
    for bank in decoded.get("banks", []):
        asset_index = bank.get("asset_index")
        if not isinstance(asset_index, int):
            continue
        for sequence in bank.get("sequences", []):
            if sequence.get("status") != "decoded":
                continue
            address = sequence.get("address")
            if not isinstance(address, int):
                continue
            record = {
                "sound_bank_asset_index": asset_index,
                "sound_bank_role": bank.get("role"),
                "technical_name": sequence.get("technical_name"),
                "address": address,
                "confidence": sequence.get("confidence"),
                "event_counts": sequence.get("event_counts", {}),
                "sha1_64": sequence.get("sha1_64"),
            }
            by_asset_and_address[(asset_index, address)] = record
            by_address.setdefault(address, []).append(record)
    return {
        "by_asset_and_address": by_asset_and_address,
        "by_address": by_address,
    }


def program_provenance(program: dict) -> list[dict]:
    values = []
    values.extend(valid_provenance_items(program.get("sequence_provenance", [])))
    for variant in program.get("variants", []):
        if isinstance(variant, dict):
            values.extend(valid_provenance_items(variant.get("sequence_provenance", [])))
    return dedupe_provenance(values)


def valid_provenance_items(items: object) -> list[dict]:
    if not isinstance(items, list):
        return []
    values = []
    for item in items:
        if not isinstance(item, dict):
            continue
        address = item.get("rom_sequence_address", item.get("sequence_address"))
        if not isinstance(address, int):
            continue
        values.append({**item, "rom_sequence_address": address & 0xFFFF})
    return values


def dedupe_provenance(items: list[dict]) -> list[dict]:
    deduped = []
    seen: set[tuple[int | None, int, int | None, int | None]] = set()
    for item in items:
        key = (
            item.get("sound_bank_asset_index"),
            item["rom_sequence_address"],
            item.get("voice") if isinstance(item.get("voice"), int) else None,
            item.get("frame") if isinstance(item.get("frame"), int) else None,
        )
        if key in seen:
            continue
        seen.add(key)
        deduped.append(item)
    return deduped


def link_provenance(item: dict, lookup: dict) -> dict:
    address = item["rom_sequence_address"]
    asset_index = item.get("sound_bank_asset_index")
    if isinstance(asset_index, int):
        sequence = lookup["by_asset_and_address"].get((asset_index, address))
        if sequence is None:
            return {"status": "unmatched_asset_address", "provenance": item, "matches": []}
        return {"status": "linked", "provenance": item, "matches": [sequence]}

    matches = lookup["by_address"].get(address, [])
    if len(matches) == 1:
        return {"status": "linked", "provenance": item, "matches": matches}
    if len(matches) > 1:
        return {"status": "ambiguous_address", "provenance": item, "matches": matches}
    return {"status": "unmatched_address", "provenance": item, "matches": []}


def build_index(decoded: dict, harvest: dict, decoded_path: Path, harvest_path: Path) -> dict:
    lookup = decoded_sequence_lookup(decoded)
    sounds = []
    for program in harvest.get("programs", []):
        if not isinstance(program, dict):
            continue
        provenance = program_provenance(program)
        links = [link_provenance(item, lookup) for item in provenance]
        linked = [link for link in links if link["status"] == "linked"]
        ambiguous = [link for link in links if link["status"] == "ambiguous_address"]
        sounds.append(
            {
                "name": program.get("name"),
                "bank": program.get("bank"),
                "id": program.get("id"),
                "status": "linked"
                if linked
                else "ambiguous"
                if ambiguous
                else "unlinked",
                "program_status": program.get("status"),
                "variant_count": program.get("variant_count", 0),
                "first_frames": program.get("first_frames", []),
                "sequence_links": links,
            }
        )
    return {
        "format": "zelda3_modern_sound_index_v1",
        "runtime_dependency": False,
        "source": {
            "decoded_sequences": str(decoded_path),
            "modern_sfx_harvest": str(harvest_path),
            "boundary": "offline authoring index only; modern runtime consumes reviewed generated assets",
        },
        "coverage": {
            "programs": len(sounds),
            "linked_programs": sum(1 for sound in sounds if sound["status"] == "linked"),
            "ambiguous_programs": sum(1 for sound in sounds if sound["status"] == "ambiguous"),
            "unlinked_programs": sum(1 for sound in sounds if sound["status"] == "unlinked"),
            "sequence_links": sum(
                1
                for sound in sounds
                for link in sound["sequence_links"]
                if link["status"] == "linked"
            ),
        },
        "sounds": sounds,
    }


def render_markdown(index: dict) -> str:
    coverage = index["coverage"]
    lines = [
        "# Modern Sound Index",
        "",
        f"- Decoded sequences: `{index['source']['decoded_sequences']}`",
        f"- Modern SFX harvest: `{index['source']['modern_sfx_harvest']}`",
        "- Runtime dependency: `false`",
        f"- Linked programs: {coverage['linked_programs']}/{coverage['programs']}",
        f"- Ambiguous programs: {coverage['ambiguous_programs']}",
        f"- Unlinked programs: {coverage['unlinked_programs']}",
        "",
        "| Status | Bank | Id | Name | Variants | Frames | Linked sequences |",
        "|---|---:|---:|---|---:|---|---|",
    ]
    for sound in index["sounds"]:
        linked = [
            match
            for link in sound["sequence_links"]
            if link["status"] == "linked"
            for match in link["matches"]
        ]
        linked_text = ", ".join(
            f"{match['technical_name']}@0x{match['address']:04x} ({match['confidence']})"
            for match in linked[:4]
        )
        if not linked_text:
            statuses = sorted({link["status"] for link in sound["sequence_links"]})
            linked_text = ", ".join(statuses) if statuses else "no provenance"
        frames = ", ".join(str(frame) for frame in sound.get("first_frames", []))
        lines.append(
            f"| {sound['status']} | {sound['bank']} | 0x{sound['id']:02x} | "
            f"`{sound['name']}` | {sound['variant_count']} | {frames} | {linked_text} |"
        )
    return "\n".join(lines) + "\n"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--decoded-sequences-json", type=Path, default=DEFAULT_DECODED)
    parser.add_argument("--modern-sfx-harvest-json", type=Path, default=DEFAULT_HARVEST)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--report-out", type=Path)
    args = parser.parse_args(argv)
    if not args.decoded_sequences_json.is_file():
        parser.error(f"--decoded-sequences-json does not exist: {args.decoded_sequences_json}")
    if not args.modern_sfx_harvest_json.is_file():
        parser.error(f"--modern-sfx-harvest-json does not exist: {args.modern_sfx_harvest_json}")
    if args.json_out is None:
        args.json_out = args.output_dir / "modern-sound-index.json"
    if args.report_out is None:
        args.report_out = args.output_dir / "modern-sound-index.md"
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        decoded = json.loads(args.decoded_sequences_json.read_text(encoding="utf-8"))
        harvest = json.loads(args.modern_sfx_harvest_json.read_text(encoding="utf-8"))
        index = build_index(decoded, harvest, args.decoded_sequences_json, args.modern_sfx_harvest_json)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.report_out.parent.mkdir(parents=True, exist_ok=True)
    args.report_out.write_text(render_markdown(index), encoding="utf-8")
    coverage = index["coverage"]
    print(
        "modern sound index: "
        f"linked={coverage['linked_programs']}/{coverage['programs']} "
        f"ambiguous={coverage['ambiguous_programs']} unlinked={coverage['unlinked_programs']} "
        f"sequence_links={coverage['sequence_links']}"
    )
    print(f"json: {args.json_out}")
    print(f"report: {args.report_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
