#!/usr/bin/env python3
"""Validate the checked-in modern SFX authoring source without building Rust."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ASSETS = ROOT / "assets" / "audio" / "modern_sfx.json"
FORMAT = "zelda3_modern_sfx_assets_v1"
WAVEFORMS = {"Pulse", "Saw", "Triangle", "Noise"}
TOP_LEVEL_FIELDS = {"format", "programs", "exact_dsp_steps", "pitch_events"}
PROGRAM_FIELDS = {
    "bank", "id", "variant", "variant_hash", "name", "promotion_status", "context", "steps"
}
CONTEXT_FIELDS = {"source_slot", "voice_mask", "context_voice_mask", "step_count"}
STEP_FIELDS = {
    "voice", "pitch", "instrument", "waveform", "volume", "pan", "echo", "envelope",
    "duration_frames", "pitch_slide",
}
ENVELOPE_FIELDS = {"attack", "decay", "sustain", "release"}
SLIDE_FIELDS = {"target_pitch", "frames"}
EXACT_DSP_FIELDS = {
    "bank", "id", "variant_hash", "step", "voice", "pitch", "instrument", "volume", "pan",
    "duration_frames", "echo", "command_delay_frames", "scheduler_tick_index", "dsp_pitch",
    "volume_left", "volume_right", "adsr1", "adsr2", "gain", "sample_offset", "duration_samples",
    "interrupt_voice", "interrupt_delay_frames", "interrupt_scheduler_tick_index",
    "ownership_duration_samples", "ownership_release_overflows", "volume_via_parameters",
}
PITCH_FIELDS = {"bank", "id", "variant_hash", "step", "relative_sample", "pitch_word"}


def reject_unknown_fields(
    value: object, allowed: set[str], label: str, errors: list[str]
) -> None:
    if isinstance(value, dict) and (unknown := set(value) - allowed):
        errors.append(f"{label} has unknown fields: {sorted(unknown)}")


def validate(document: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(document, dict):
        return ["document must be a JSON object"]
    reject_unknown_fields(document, TOP_LEVEL_FIELDS, "document", errors)
    if document.get("format") != FORMAT:
        errors.append(f"format must be {FORMAT!r}")
    programs = document.get("programs")
    if not isinstance(programs, list) or not programs:
        errors.append("programs must be a non-empty array")
        return errors

    identities: set[tuple[int, int, int]] = set()
    for index, program in enumerate(programs):
        label = f"programs[{index}]"
        if not isinstance(program, dict):
            errors.append(f"{label} must be an object")
            continue
        reject_unknown_fields(program, PROGRAM_FIELDS, label, errors)
        reject_unknown_fields(program.get("context"), CONTEXT_FIELDS, f"{label}.context", errors)
        identity = (program.get("bank"), program.get("id"), program.get("variant_hash"))
        if identity in identities:
            errors.append(f"{label} duplicates bank/id/variant_hash {identity}")
        identities.add(identity)
        if program.get("promotion_status") != "review_ready":
            errors.append(f"{label} is not review_ready")
        if not isinstance(program.get("name"), str) or not program["name"]:
            errors.append(f"{label}.name must be non-empty")
        steps = program.get("steps")
        if not isinstance(steps, list):
            errors.append(f"{label}.steps must be an array")
            continue
        for step_index, step in enumerate(steps):
            step_label = f"{label}.steps[{step_index}]"
            if not isinstance(step, dict):
                errors.append(f"{step_label} must be an object")
                continue
            reject_unknown_fields(step, STEP_FIELDS, step_label, errors)
            reject_unknown_fields(
                step.get("envelope"), ENVELOPE_FIELDS, f"{step_label}.envelope", errors
            )
            if step.get("pitch_slide") is not None:
                reject_unknown_fields(
                    step.get("pitch_slide"), SLIDE_FIELDS, f"{step_label}.pitch_slide", errors
                )
            if not isinstance(step.get("voice"), int) or not 0 <= step["voice"] < 8:
                errors.append(f"{step_label}.voice must be in 0..7")
            if step.get("waveform") not in WAVEFORMS:
                errors.append(f"{step_label}.waveform is invalid")

    exact_steps = document.get("exact_dsp_steps")
    if not isinstance(exact_steps, list):
        errors.append("exact_dsp_steps must be an array")
    else:
        exact_identities: set[tuple[int, int, int, int]] = set()
        for index, step in enumerate(exact_steps):
            if not isinstance(step, dict):
                errors.append(f"exact_dsp_steps[{index}] must be an object")
                continue
            reject_unknown_fields(step, EXACT_DSP_FIELDS, f"exact_dsp_steps[{index}]", errors)
            if not isinstance(step.get("voice"), int) or not 0 <= step["voice"] < 8:
                errors.append(f"exact_dsp_steps[{index}].voice must be in 0..7")
            identity = tuple(
                step.get(field) for field in ("bank", "id", "variant_hash", "step")
            )
            if identity in exact_identities:
                errors.append(f"exact_dsp_steps[{index}] duplicates identity {identity}")
            exact_identities.add(identity)

    pitch_events = document.get("pitch_events")
    if not isinstance(pitch_events, list):
        errors.append("pitch_events must be an array")
    else:
        pitch_identities: set[tuple[int, int, int, int, int]] = set()
        for index, event in enumerate(pitch_events):
            if not isinstance(event, dict):
                errors.append(f"pitch_events[{index}] must be an object")
                continue
            reject_unknown_fields(event, PITCH_FIELDS, f"pitch_events[{index}]", errors)
            identity = tuple(
                event.get(field)
                for field in ("bank", "id", "variant_hash", "step", "relative_sample")
            )
            if identity in pitch_identities:
                errors.append(f"pitch_events[{index}] duplicates identity {identity}")
            pitch_identities.add(identity)
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path, nargs="?", default=DEFAULT_ASSETS)
    args = parser.parse_args(argv)
    try:
        document = json.loads(args.assets.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"{args.assets}: {exc}", file=sys.stderr)
        return 2
    errors = validate(document)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(
        f"modern SFX assets valid: programs={len(document['programs'])} "
        f"exact_dsp_steps={len(document['exact_dsp_steps'])} "
        f"pitch_events={len(document['pitch_events'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
