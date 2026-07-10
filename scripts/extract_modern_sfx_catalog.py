#!/usr/bin/env python3
"""Lift modern SFX catalog candidates from Rust audio trace JSONL.

The Rust replay trace can include per-frame DSP writes by setting
`ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME` for focused captures. This tool consumes
those JSONL traces, detects SFX command transitions with the same slot mapping
as `ModernAudioSequencer`, and translates observed voice DSP writes into
reviewable modern SFX program candidates.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, TextIO


SFX_SLOT_SOURCES = [
    "queue.input[1]",
    "queue.input[2]",
    "queue.input[3]",
    "music.sound_effect_ambient",
    "music.sound_effect_1",
    "music.sound_effect_2",
    "music.apui00",
]

WAVEFORMS = ["Pulse", "Saw", "Triangle"]


@dataclass(frozen=True)
class SfxOccurrence:
    bank: int
    sfx_id: int
    voice_hint: int
    frame: int
    frame_index: int
    source: str


@dataclass
class VoiceState:
    volume_left: int = 0
    volume_right: int = 0
    pitch_low: int = 0
    pitch_high: int = 0
    source: int = 0
    adsr1: int = 0
    adsr2: int = 0
    gain: int = 0


@dataclass
class ActiveStep:
    voice: int
    start_frame: int
    state: VoiceState
    noise_enabled: bool
    pitch_changes: list[tuple[int, int]] = field(default_factory=list)
    keyoff_frame: int | None = None


def load_trace(paths: list[Path]) -> list[dict]:
    frames: list[dict] = []
    if not paths:
        frames.extend(load_trace_stream(sys.stdin, "<stdin>"))
    for path in paths:
        if str(path) == "-":
            frames.extend(load_trace_stream(sys.stdin, "<stdin>"))
        else:
            with path.open(encoding="utf-8") as stream:
                frames.extend(load_trace_stream(stream, str(path)))
    return sorted(frames, key=lambda frame: int(frame.get("frame", 0)))


def load_trace_stream(stream: TextIO, label: str) -> list[dict]:
    frames: list[dict] = []
    for line_no, line in enumerate(stream, start=1):
        stripped = line.strip()
        if not stripped or not stripped.startswith("{"):
            continue
        try:
            frame = json.loads(stripped)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{label}:{line_no}: invalid JSON: {exc}") from exc
        if "frame" in frame:
            frames.append(frame)
    return frames


def sfx_slot_values(frame: dict) -> list[int]:
    queue = frame.get("queue")
    queue_input = queue.get("input") if isinstance(queue, dict) else None
    if not is_u8_list(queue_input, 4):
        queue_input = [0, 0, 0, 0]

    apui = frame.get("apui")
    if not is_u8_list(apui, 4):
        apui = [0, 0, 0, 0]
    music = frame.get("music")
    if not is_u8_list(music, 3):
        music = [0, 0, 0]

    return [
        int(queue_input[1]),
        int(queue_input[2]),
        int(queue_input[3]),
        int(apui[2]),
        int(apui[3]),
        int(music[0]),
        int(apui[0]),
    ]


def is_u8_list(value: object, size: int) -> bool:
    return (
        isinstance(value, list)
        and len(value) == size
        and all(isinstance(item, int) and 0 <= item <= 0xFF for item in value)
    )


def discover_sfx_occurrences(frames: list[dict]) -> list[SfxOccurrence]:
    previous = [0] * len(SFX_SLOT_SOURCES)
    occurrences: list[SfxOccurrence] = []
    for index, frame in enumerate(frames):
        frame_no = int(frame.get("frame", 0))
        for slot, code in enumerate(sfx_slot_values(frame)):
            if code == previous[slot]:
                continue
            if code != 0:
                occurrences.append(
                    SfxOccurrence(
                        bank=slot,
                        sfx_id=code,
                        voice_hint=min(slot + 1, 7),
                        frame=frame_no,
                        frame_index=index,
                        source=SFX_SLOT_SOURCES[slot],
                    )
                )
            previous[slot] = code
    return occurrences


def extract_catalog(frames: list[dict], window_frames: int) -> dict:
    occurrences = discover_sfx_occurrences(frames)
    observed: dict[tuple[int, int], list[dict]] = {}
    for occurrence in occurrences:
        program = lift_occurrence(frames, occurrence, window_frames)
        observed.setdefault((occurrence.bank, occurrence.sfx_id), []).append(program)

    programs = []
    for (bank, sfx_id), variants in sorted(observed.items()):
        programs.append(merge_variants(bank, sfx_id, variants))

    coverage = {
        "commands": len(occurrences),
        "programs": len(programs),
        "lifted": sum(1 for program in programs if program["status"] == "lifted"),
        "ambiguous": sum(1 for program in programs if program["status"] == "ambiguous"),
        "missing_dsp_events": sum(
            1 for program in programs if program["status"] == "missing_dsp_events"
        ),
        "no_key_on": sum(1 for program in programs if program["status"] == "no_key_on"),
    }
    coverage["gaps"] = (
        coverage["ambiguous"] + coverage["missing_dsp_events"] + coverage["no_key_on"]
    )
    return {"coverage": coverage, "programs": programs}


def lift_occurrence(
    frames: list[dict], occurrence: SfxOccurrence, window_frames: int
) -> dict:
    end_frame = occurrence.frame + window_frames
    window = [
        frame
        for frame in frames[occurrence.frame_index :]
        if occurrence.frame <= int(frame.get("frame", 0)) <= end_frame
    ]
    writes = list(iter_dsp_writes(window))
    base = {
        "bank": occurrence.bank,
        "id": occurrence.sfx_id,
        "name": generated_program_name(occurrence.bank, occurrence.sfx_id),
        "source": occurrence.source,
        "first_frame": occurrence.frame,
        "window_frames": window_frames,
        "trace_frames": [int(frame.get("frame", 0)) for frame in window],
        "dsp_write_events": len(writes),
    }
    if not writes:
        return {
            **base,
            "status": "missing_dsp_events",
            "steps": [],
            "notes": [
                "trace window has no dsp_write_events; rerun with "
                f"ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME={occurrence.frame}"
            ],
        }

    steps = lift_steps_from_writes(writes)
    if not steps:
        return {
            **base,
            "status": "no_key_on",
            "steps": [],
            "notes": ["dsp_write_events were present, but no KON voice bit was observed"],
        }
    return {**base, "status": "lifted", "steps": steps, "notes": []}


def iter_dsp_writes(frames: Iterable[dict]) -> Iterable[tuple[int, int, int, int, int]]:
    for frame in frames:
        frame_no = int(frame.get("frame", 0))
        events = frame.get("dsp_write_events")
        if not isinstance(events, list):
            continue
        for event in events:
            if (
                isinstance(event, list)
                and len(event) == 4
                and isinstance(event[0], int)
                and isinstance(event[1], int)
                and isinstance(event[2], int)
                and isinstance(event[3], int)
            ):
                yield frame_no, event[0] & 0xFF, event[1] & 0xFF, event[2], event[3] & 0xFF


def lift_steps_from_writes(writes: list[tuple[int, int, int, int, int]]) -> list[dict]:
    voices = [VoiceState() for _ in range(8)]
    noise_mask = 0
    active: dict[int, ActiveStep] = {}
    completed: list[ActiveStep] = []

    for frame, addr, value, _sample_offset, _timer_cycles in writes:
        reg = addr & 0x7F
        if reg == 0x3D:
            noise_mask = value
            continue
        if reg == 0x4C:
            for voice in voices_from_mask(value):
                active[voice] = ActiveStep(
                    voice=voice,
                    start_frame=frame,
                    state=copy_voice_state(voices[voice]),
                    noise_enabled=bool(noise_mask & (1 << voice)),
                )
            continue
        if reg == 0x5C:
            for voice in voices_from_mask(value):
                step = active.pop(voice, None)
                if step is not None:
                    step.keyoff_frame = frame
                    completed.append(step)
            continue
        if (reg & 0x0F) <= 0x07:
            voice = reg >> 4
            parameter = reg & 0x0F
            update_voice_state(voices[voice], parameter, value)
            if voice in active and parameter in (0x02, 0x03):
                active[voice].pitch_changes.append((frame, voice_pitch_word(voices[voice])))

    completed.extend(active.values())
    return [modern_step_from_active_step(step) for step in completed]


def voices_from_mask(mask: int) -> Iterable[int]:
    for voice in range(8):
        if mask & (1 << voice):
            yield voice


def update_voice_state(state: VoiceState, parameter: int, value: int) -> None:
    if parameter == 0x00:
        state.volume_left = value
    elif parameter == 0x01:
        state.volume_right = value
    elif parameter == 0x02:
        state.pitch_low = value
    elif parameter == 0x03:
        state.pitch_high = value & 0x3F
    elif parameter == 0x04:
        state.source = value
    elif parameter == 0x05:
        state.adsr1 = value
    elif parameter == 0x06:
        state.adsr2 = value
    elif parameter == 0x07:
        state.gain = value


def copy_voice_state(state: VoiceState) -> VoiceState:
    return VoiceState(
        volume_left=state.volume_left,
        volume_right=state.volume_right,
        pitch_low=state.pitch_low,
        pitch_high=state.pitch_high,
        source=state.source,
        adsr1=state.adsr1,
        adsr2=state.adsr2,
        gain=state.gain,
    )


def modern_step_from_active_step(step: ActiveStep) -> dict:
    pitch_word = voice_pitch_word(step.state)
    pitch = modern_pitch_from_dsp_pitch(pitch_word)
    pitch_slide = None
    for frame, changed_pitch_word in step.pitch_changes:
        target = modern_pitch_from_dsp_pitch(changed_pitch_word)
        if target != pitch:
            pitch_slide = {
                "target_pitch": target,
                "frames": max(1, frame - step.start_frame),
                "dsp_target_pitch": changed_pitch_word,
            }
    duration_end = step.keyoff_frame if step.keyoff_frame is not None else step.start_frame + 1
    return {
        "voice": step.voice,
        "pitch": pitch,
        "instrument": step.state.source,
        "waveform": waveform_for_step(step),
        "volume": modern_volume(step.state),
        "envelope": modern_envelope(step.state),
        "duration_frames": max(1, duration_end - step.start_frame),
        "pitch_slide": pitch_slide,
        "evidence": {
            "start_frame": step.start_frame,
            "keyoff_frame": step.keyoff_frame,
            "dsp_pitch": pitch_word,
            "dsp_adsr1": step.state.adsr1,
            "dsp_adsr2": step.state.adsr2,
            "dsp_gain": step.state.gain,
        },
    }


def voice_pitch_word(state: VoiceState) -> int:
    return ((state.pitch_high & 0x3F) << 8) | state.pitch_low


def modern_pitch_from_dsp_pitch(pitch_word: int) -> int:
    if pitch_word <= 0:
        return 0
    return max(0, min(127, int(round(pitch_word / 32.0))))


def modern_volume(state: VoiceState) -> int:
    return max(abs(signed8(state.volume_left)), abs(signed8(state.volume_right)))


def signed8(value: int) -> int:
    return value - 256 if value >= 128 else value


def modern_envelope(state: VoiceState) -> dict:
    return {
        "attack": state.adsr1 & 0x0F,
        "decay": (state.adsr1 >> 4) & 0x07,
        "sustain": (state.adsr2 >> 5) & 0x07,
        "release": state.gain & 0x1F if state.gain else state.adsr2 & 0x1F,
    }


def waveform_for_step(step: ActiveStep) -> str:
    if step.noise_enabled:
        return "Noise"
    return WAVEFORMS[step.state.source % len(WAVEFORMS)]


def merge_variants(bank: int, sfx_id: int, variants: list[dict]) -> dict:
    lifted = [variant for variant in variants if variant["status"] == "lifted"]
    base = {
        "bank": bank,
        "id": sfx_id,
        "name": generated_program_name(bank, sfx_id),
        "occurrences": len(variants),
        "first_frames": [variant["first_frame"] for variant in variants],
    }
    if not lifted:
        statuses = sorted({variant["status"] for variant in variants})
        status = statuses[0] if len(statuses) == 1 else "missing_dsp_events"
        notes = []
        for variant in variants:
            notes.extend(variant.get("notes", []))
        return {**base, "status": status, "steps": [], "notes": sorted(set(notes))}

    signatures: dict[str, dict] = {}
    for variant in lifted:
        signature = json.dumps(
            [step_signature(step) for step in variant["steps"]],
            sort_keys=True,
            separators=(",", ":"),
        )
        signatures.setdefault(signature, variant)
    if len(signatures) > 1:
        return {
            **base,
            "status": "ambiguous",
            "steps": [],
            "variants": [
                {
                    "first_frame": variant["first_frame"],
                    "steps": variant["steps"],
                    "dsp_write_events": variant["dsp_write_events"],
                }
                for variant in signatures.values()
            ],
            "notes": ["same SFX command produced multiple distinct DSP programs"],
        }
    variant = next(iter(signatures.values()))
    return {
        **base,
        "status": "lifted",
        "source": variant["source"],
        "steps": variant["steps"],
        "dsp_write_events": sum(item["dsp_write_events"] for item in lifted),
        "notes": [],
    }


def generated_program_name(bank: int, sfx_id: int) -> str:
    return f"trace_sfx_{bank:02x}_{sfx_id:02x}"


def step_signature(step: dict) -> dict:
    return {
        "voice": step["voice"],
        "pitch": step["pitch"],
        "instrument": step["instrument"],
        "waveform": step["waveform"],
        "volume": step["volume"],
        "envelope": step["envelope"],
        "duration_frames": step["duration_frames"],
        "pitch_slide": step["pitch_slide"],
    }


def render_rust_catalog(programs: list[dict]) -> str:
    lines = [
        "// Generated by scripts/extract_modern_sfx_catalog.py.",
        "// Review before copying into crates/zelda3/src/modern_sfx_catalog.rs.",
        "",
    ]
    for program in programs:
        if program["status"] != "lifted":
            lines.append(
                f"// {program['name']}: skipped status={program['status']} frames={program['first_frames']}"
            )
            continue
        const_name = program["name"].upper()
        lines.append(f"const {const_name}_STEPS: &[ModernSfxStep] = &[")
        for step in program["steps"]:
            envelope = step["envelope"]
            lines.extend(
                [
                    "    ModernSfxStep {",
                    f"        voice: {step['voice']},",
                    f"        pitch: {step['pitch']},",
                    f"        instrument: {step['instrument']},",
                    f"        waveform: ModernSfxWaveform::{step['waveform']},",
                    f"        volume: {step['volume']},",
                    "        envelope: ModernSfxEnvelope {",
                    f"            attack: {envelope['attack']},",
                    f"            decay: {envelope['decay']},",
                    f"            sustain: {envelope['sustain']},",
                    f"            release: {envelope['release']},",
                    "        },",
                    f"        duration_frames: {step['duration_frames']},",
                ]
            )
            slide = step["pitch_slide"]
            if slide is None:
                lines.append("        pitch_slide: None,")
            else:
                lines.extend(
                    [
                        "        pitch_slide: Some(ModernSfxPitchSlide {",
                        f"            target_pitch: {slide['target_pitch']},",
                        f"            frames: {slide['frames']},",
                        "        }),",
                    ]
                )
            lines.append("    },")
        lines.append("];")
        lines.append("")
        lines.extend(
            [
                f"ModernSfxProgram {{",
                f"    bank: {program['bank']},",
                f"    id: 0x{program['id']:02x},",
                f"    name: \"{program['name']}\",",
                f"    steps: {const_name}_STEPS,",
                f"}},",
                "",
            ]
        )
    return "\n".join(lines)


def write_output(path: Path | None, content: str) -> None:
    if path is None:
        print(content)
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content + "\n", encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="*", type=Path, help="Rust audio trace JSONL path, or - for stdin")
    parser.add_argument("--window-frames", type=int, default=12)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--rust-out", type=Path)
    parser.add_argument(
        "--fail-on-gaps",
        action="store_true",
        help="exit non-zero when any discovered command is missing or ambiguous",
    )
    args = parser.parse_args(argv)
    if args.window_frames <= 0:
        parser.error("--window-frames must be greater than zero")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        catalog = extract_catalog(load_trace(args.trace), args.window_frames)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    json_content = json.dumps(catalog, indent=2, sort_keys=True)
    if args.json_out is None and args.rust_out is None:
        print(json_content)
    elif args.json_out is not None:
        write_output(args.json_out, json_content)
    if args.rust_out is not None:
        write_output(args.rust_out, render_rust_catalog(catalog["programs"]))
    if args.fail_on_gaps and catalog["coverage"]["gaps"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
