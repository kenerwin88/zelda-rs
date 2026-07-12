#!/usr/bin/env python3
"""Promote harvested SFX evidence into a stable modern sound asset manifest.

This is an offline authoring tool. It may reference decoded ROM sequence
evidence to choose and document a primary source for each harvested modern SFX
program, but the emitted `modern_program` payload is the modern-owned asset
shape. Runtime code should consume reviewed modern assets, not ROM/SPC/DSP
structures.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DECODED = ROOT / "target" / "rom-audio-catalog" / "decoded-sequences.json"
DEFAULT_HARVEST = ROOT / "target" / "modern-sfx-harvest-variants-40" / "modern-sfx-harvest.json"
DEFAULT_OUTPUT_DIR = ROOT / "target" / "modern-sound-assets"

CONFIDENCE_SCORE = {
    "high": 3000,
    "medium": 2000,
    "low": 1000,
}


def decoded_sequence_lookup(decoded: dict) -> dict:
    by_asset_and_address = {}
    by_address: dict[int, list[dict]] = defaultdict(list)
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
                "decoder_kind": sequence.get("decoder_kind"),
                "event_counts": sequence.get("event_counts", {}),
                "bytes_consumed": sequence.get("bytes_consumed"),
                "sha1_64": sequence.get("sha1_64"),
            }
            by_asset_and_address[(asset_index, address)] = record
            by_address[address].append(record)
    return {
        "by_asset_and_address": by_asset_and_address,
        "by_address": dict(by_address),
    }


def source_slot_from_text(source: object, fallback: int) -> int:
    if isinstance(source, str):
        match = re.search(r"\[(\d+)\]", source)
        if match:
            return int(match.group(1))
    return fallback


def voice_mask_from_steps(steps: list[dict]) -> int:
    mask = 0
    for step in steps:
        voice = step.get("voice")
        if isinstance(voice, int) and 0 <= voice < 8:
            mask |= 1 << voice
    return mask


def clean_step(step: dict) -> dict:
    return {
        "voice": step.get("voice"),
        "pitch": step.get("pitch"),
        "instrument": step.get("instrument"),
        "waveform": step.get("waveform"),
        "volume": step.get("volume"),
        "envelope": step.get("envelope", {}),
        "duration_frames": step.get("duration_frames"),
        "pitch_slide": step.get("pitch_slide"),
    }


def valid_provenance(items: object) -> list[dict]:
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


def harvest_units(program: dict) -> list[dict]:
    variants = [item for item in program.get("variants", []) if isinstance(item, dict)]
    if not variants:
        return [
            {
                "program": program,
                "unit": program,
                "variant": 0,
                "has_variants": False,
                "name": program.get("name"),
                "variant_name": None,
            }
        ]
    units = []
    for index, variant in enumerate(variants):
        units.append(
            {
                "program": program,
                "unit": variant,
                "variant": index,
                "has_variants": True,
                "name": variant.get("name", f"{program.get('name')}_v{index:02d}"),
                "variant_name": variant.get("name"),
            }
        )
    return units


def link_provenance(item: dict, lookup: dict) -> list[dict]:
    address = item["rom_sequence_address"]
    asset_index = item.get("sound_bank_asset_index")
    if isinstance(asset_index, int):
        sequence = lookup["by_asset_and_address"].get((asset_index, address))
        return [] if sequence is None else [sequence]
    return lookup["by_address"].get(address, [])


def aggregate_sequence_evidence(provenance: list[dict], lookup: dict) -> list[dict]:
    grouped: dict[tuple[int, int, str], dict] = {}
    for item in provenance:
        matches = link_provenance(item, lookup)
        for match in matches:
            key = (
                int(match["sound_bank_asset_index"]),
                int(match["address"]),
                str(match["technical_name"]),
            )
            entry = grouped.setdefault(
                key,
                {
                    **match,
                    "occurrences": 0,
                    "frames": set(),
                    "voices": set(),
                    "sources": Counter(),
                },
            )
            entry["occurrences"] += 1
            frame = item.get("frame")
            if isinstance(frame, int):
                entry["frames"].add(frame)
            voice = item.get("voice")
            if isinstance(voice, int):
                entry["voices"].add(voice)
            source = item.get("source")
            if isinstance(source, str):
                entry["sources"][source] += 1
    evidence = []
    for entry in grouped.values():
        sources = [
            {"source": source, "occurrences": count}
            for source, count in sorted(entry.pop("sources").items())
        ]
        evidence.append(
            {
                **entry,
                "frames": sorted(entry["frames"]),
                "voices": sorted(entry["voices"]),
                "sources": sources,
            }
        )
    return sorted(evidence, key=sequence_sort_key)


def sequence_score(sequence: dict) -> int:
    counts = sequence.get("event_counts", {})
    controls = int(counts.get("controls", 0))
    notes = int(counts.get("notes", 0))
    return (
        CONFIDENCE_SCORE.get(sequence.get("confidence"), 0)
        + int(sequence.get("occurrences", 0)) * 100
        + controls * 25
        + notes * 10
        + int(sequence.get("bytes_consumed") or 0)
    )


def sequence_sort_key(sequence: dict) -> tuple[int, int, str]:
    return (-sequence_score(sequence), int(sequence.get("address", 0)), str(sequence.get("technical_name")))


def reviewed_status(primary: dict | None, steps: list[dict]) -> str:
    if primary is None or not steps:
        return "blocked"
    if primary.get("confidence") in {"high", "medium"}:
        return "review_ready"
    return "needs_review"


def stable_asset_id(program: dict, variant: int, has_variants: bool) -> str:
    base = f"sfx_{int(program.get('bank', 0)):02x}_{int(program.get('id', 0)):02x}"
    if has_variants:
        return f"{base}_v{variant:02d}"
    return base


def build_asset(unit_record: dict, lookup: dict) -> dict:
    program = unit_record["program"]
    unit = unit_record["unit"]
    variant = unit_record["variant"]
    steps = [clean_step(step) for step in unit.get("steps", []) if isinstance(step, dict)]
    context = unit.get("context_signature") if isinstance(unit.get("context_signature"), dict) else {}
    source = unit.get("source", context.get("source", program.get("source")))
    source_slot = source_slot_from_text(source, int(program.get("bank", 0)))
    provenance = valid_provenance(unit.get("sequence_provenance", []))
    if not provenance and unit is not program:
        provenance = valid_provenance(program.get("sequence_provenance", []))
    sequence_evidence = aggregate_sequence_evidence(provenance, lookup)
    primary = sequence_evidence[0] if sequence_evidence else None
    alternates = sequence_evidence[1:]
    status = reviewed_status(primary, steps)
    return {
        "asset_id": stable_asset_id(program, variant, bool(unit_record["has_variants"])),
        "name": unit_record["name"],
        "source_command": {
            "bank": program.get("bank"),
            "id": program.get("id"),
            "variant": variant,
            "source": source,
        },
        "promotion_status": status,
        "modern_program": {
            "steps": steps,
            "context": {
                "source_slot": source_slot,
                "voice_mask": context.get("voice_mask", voice_mask_from_steps(steps)),
                "context_voice_mask": context.get("context_voice_mask", 0),
                "step_count": context.get("step_count", len(steps)),
            },
        },
        "evidence": {
            "primary_sequence": primary,
            "alternate_sequences": alternates,
            "sequence_candidates": len(sequence_evidence),
            "route_occurrences": sum(item.get("occurrences", 0) for item in sequence_evidence),
            "provenance_records": len(provenance),
        },
        "notes": review_notes(status, primary, alternates),
    }


def review_notes(status: str, primary: dict | None, alternates: list[dict]) -> list[str]:
    notes = []
    if status == "blocked":
        notes.append("no promoted modern steps or decoded primary sequence evidence")
    elif status == "needs_review":
        notes.append("primary sequence is low confidence; keep as review evidence before runtime promotion")
    if alternates:
        notes.append("alternate decoded sequence evidence retained for reviewer comparison")
    if primary and primary.get("decoder_kind") == "sfx_compact":
        notes.append("primary sequence came from route-gated compact SFX decoding")
    return notes


def build_assets(decoded: dict, harvest: dict, decoded_path: Path, harvest_path: Path) -> dict:
    lookup = decoded_sequence_lookup(decoded)
    assets = []
    for program in harvest.get("programs", []):
        if not isinstance(program, dict):
            continue
        for unit in harvest_units(program):
            assets.append(build_asset(unit, lookup))
    status_counts = Counter(asset["promotion_status"] for asset in assets)
    alternate_count = sum(len(asset["evidence"]["alternate_sequences"]) for asset in assets)
    return {
        "format": "zelda3_modern_sound_assets_v1",
        "runtime_dependency": False,
        "source": {
            "decoded_sequences": str(decoded_path),
            "modern_sfx_harvest": str(harvest_path),
            "boundary": "offline promotion artifact; modern_program is runtime-owned, evidence may reference ROM/SPC/DSP diagnostics",
        },
        "coverage": {
            "assets": len(assets),
            "primary_assets": sum(1 for asset in assets if asset["evidence"]["primary_sequence"] is not None),
            "review_ready_assets": status_counts["review_ready"],
            "needs_review_assets": status_counts["needs_review"],
            "blocked_assets": status_counts["blocked"],
            "alternate_sequences": alternate_count,
        },
        "assets": assets,
    }


def render_markdown(manifest: dict) -> str:
    coverage = manifest["coverage"]
    lines = [
        "# Modern Sound Assets",
        "",
        f"- Decoded sequences: `{manifest['source']['decoded_sequences']}`",
        f"- Modern SFX harvest: `{manifest['source']['modern_sfx_harvest']}`",
        "- Runtime dependency: `false`",
        f"- Assets: {coverage['assets']}",
        f"- Primary sequence evidence: {coverage['primary_assets']}/{coverage['assets']}",
        f"- Review-ready assets: {coverage['review_ready_assets']}",
        f"- Needs review: {coverage['needs_review_assets']}",
        f"- Blocked: {coverage['blocked_assets']}",
        "",
        "| Status | Asset | Command | Steps | Primary sequence | Alternates | Notes |",
        "|---|---|---|---:|---|---:|---|",
    ]
    for asset in manifest["assets"]:
        command = asset["source_command"]
        primary = asset["evidence"]["primary_sequence"]
        if primary is None:
            primary_text = "none"
        else:
            primary_text = (
                f"{primary['technical_name']}@0x{primary['address']:04x} "
                f"({primary['confidence']}, x{primary['occurrences']})"
            )
        notes = "; ".join(asset.get("notes", []))
        lines.append(
            f"| {asset['promotion_status']} | `{asset['asset_id']}` / `{asset['name']}` | "
            f"{command['bank']}:0x{command['id']:02x} v{command['variant']} | "
            f"{len(asset['modern_program']['steps'])} | {primary_text} | "
            f"{len(asset['evidence']['alternate_sequences'])} | {notes} |"
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
        args.json_out = args.output_dir / "modern-sound-assets.json"
    if args.report_out is None:
        args.report_out = args.output_dir / "modern-sound-assets.md"
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        decoded = json.loads(args.decoded_sequences_json.read_text(encoding="utf-8"))
        harvest = json.loads(args.modern_sfx_harvest_json.read_text(encoding="utf-8"))
        manifest = build_assets(decoded, harvest, args.decoded_sequences_json, args.modern_sfx_harvest_json)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.report_out.parent.mkdir(parents=True, exist_ok=True)
    args.report_out.write_text(render_markdown(manifest), encoding="utf-8")
    coverage = manifest["coverage"]
    print(
        "modern sound assets: "
        f"assets={coverage['assets']} primary={coverage['primary_assets']} "
        f"review_ready={coverage['review_ready_assets']} needs_review={coverage['needs_review_assets']} "
        f"blocked={coverage['blocked_assets']} alternates={coverage['alternate_sequences']}"
    )
    print(f"json: {args.json_out}")
    print(f"report: {args.report_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
