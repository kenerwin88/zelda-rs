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
from dataclasses import dataclass, field, replace
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
VOICE_PARAMETER_NAMES = [
    "volume_left",
    "volume_right",
    "pitch_low",
    "pitch_high",
    "source",
    "adsr1",
    "adsr2",
    "gain",
]
STRONG_OWNERSHIP_PARAMETERS = {"source", "adsr1", "adsr2", "gain"}


@dataclass(frozen=True)
class SfxOccurrence:
    bank: int
    sfx_id: int
    voice_hint: int
    frame: int
    frame_index: int
    source: str
    next_command_frame: int | None = None


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
    start_sample_offset: int
    scheduler_tick_index: int
    state: VoiceState
    noise_enabled: bool
    echo_enabled: bool
    ownership: str
    owned_parameters: list[str]
    command_frame: int
    voice_hint: int
    pitch_changes: list[tuple[int, int]] = field(default_factory=list)
    keyoff_frame: int | None = None
    keyoff_sample_offset: int | None = None


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
        # APUI00 is music port 0, not a seventh SFX bank. Keep the slot for
        # compatibility with sequencer checkpoint/state layouts.
        0,
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
    previous_occurrence_by_bank: dict[int, int] = {}
    for index, occurrence in enumerate(occurrences):
        previous_index = previous_occurrence_by_bank.get(occurrence.bank)
        if previous_index is not None:
            occurrences[previous_index] = replace(
                occurrences[previous_index], next_command_frame=occurrence.frame
            )
        previous_occurrence_by_bank[occurrence.bank] = index
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
        "context_only": sum(1 for program in programs if program["status"] == "context_only"),
    }
    coverage["gaps"] = (
        coverage["ambiguous"]
        + coverage["missing_dsp_events"]
        + coverage["no_key_on"]
        + coverage["context_only"]
    )
    return {"coverage": coverage, "programs": programs}


def lift_occurrence(
    frames: list[dict], occurrence: SfxOccurrence, window_frames: int
) -> dict:
    end_frame = occurrence.frame + window_frames
    if occurrence.next_command_frame is not None:
        end_frame = min(end_frame, occurrence.next_command_frame - 1)
    window = [
        frame
        for frame in frames[occurrence.frame_index :]
        if occurrence.frame <= int(frame.get("frame", 0)) <= end_frame
    ]
    writes = list(iter_dsp_writes(window))
    sequence_provenance = sequence_provenance_for_occurrence(window, occurrence)
    base = {
        "bank": occurrence.bank,
        "id": occurrence.sfx_id,
        "name": generated_program_name(occurrence.bank, occurrence.sfx_id),
        "source": occurrence.source,
        "first_frame": occurrence.frame,
        "window_frames": window_frames,
        "trace_frames": [int(frame.get("frame", 0)) for frame in window],
        "dsp_write_events": len(writes),
        "sequence_provenance": sequence_provenance,
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

    provenance_voices = {
        int(item["voice"])
        for item in sequence_provenance
        if isinstance(item.get("voice"), int) and 0 <= int(item["voice"]) < 8
    }
    owned_voice_mask = sum(1 << voice for voice in provenance_voices)
    lifted = lift_steps_from_writes(
        writes,
        occurrence.frame,
        occurrence.voice_hint,
        owned_voice_mask=owned_voice_mask,
        frame_timer_cycles={
            int(frame.get("frame", 0)): int((frame.get("queue") or {}).get("timer") or 0)
            for frame in window
        },
    )
    steps = lifted["steps"]
    context_steps = lifted["context_steps"]
    if not steps:
        if context_steps:
            return {
                **base,
                "status": "context_only",
                "steps": [],
                "context_steps": context_steps,
                "voice_ownership": lifted["voice_ownership"],
                "notes": [
                    "dsp_write_events only produced carried-over or weakly-owned voices; "
                    "no command-owned SFX program was lifted"
                ],
            }
        return {
            **base,
            "status": "no_key_on",
            "steps": [],
            "context_steps": [],
            "voice_ownership": lifted["voice_ownership"],
            "notes": ["dsp_write_events were present, but no KON voice bit was observed"],
        }
    notes = []
    if context_steps:
        notes.append("ignored carried-over or weakly-owned context voices")
    return {
        **base,
        "status": "lifted",
        "steps": steps,
        "context_steps": context_steps,
        "voice_ownership": lifted["voice_ownership"],
        "notes": notes,
    }


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


def sequence_provenance_for_occurrence(frames: Iterable[dict], occurrence: SfxOccurrence) -> list[dict]:
    explicit = []
    matched_channels = []
    target_sound = occurrence.sfx_id & 0x3F
    seen: set[tuple[int | None, int, int]] = set()
    for frame in frames:
        frame_no = int(frame.get("frame", 0))
        for item in frame.get("sequence_provenance", []):
            if not isinstance(item, dict):
                continue
            bank = item.get("bank", item.get("command_bank"))
            command_id = item.get("id", item.get("command_id"))
            address = item.get("rom_sequence_address", item.get("sequence_address"))
            if not (
                isinstance(bank, int)
                and bank == occurrence.bank
                and isinstance(command_id, int)
                and (command_id & 0xFF) == occurrence.sfx_id
                and isinstance(address, int)
            ):
                continue
            key = (item.get("sound_bank_asset_index"), frame_no, address)
            if key in seen:
                continue
            seen.add(key)
            explicit.append(
                {
                    "frame": frame_no,
                    "source": occurrence.source,
                    "bank": occurrence.bank,
                    "id": occurrence.sfx_id,
                    "sound_bank_asset_index": item.get("sound_bank_asset_index"),
                    "rom_sequence_address": address & 0xFFFF,
                    "voice": item.get("voice"),
                    "sound_id": item.get("sound_id", target_sound),
                    "source_kind": "explicit_trace",
                }
            )
        queue = frame.get("queue")
        channels = frame.get("sfx_channels")
        if not isinstance(channels, list) and isinstance(queue, dict):
            channels = queue.get("sfx_channels")
        if not isinstance(channels, list):
            continue
        for channel in channels:
            if not isinstance(channel, dict):
                continue
            sound = channel.get("sound")
            address = channel.get("sound_ptr")
            if not (
                isinstance(sound, int)
                and (sound & 0x3F) == target_sound
                and isinstance(address, int)
                and address != 0
            ):
                continue
            key = (None, frame_no, address & 0xFFFF)
            if key in seen:
                continue
            seen.add(key)
            matched_channels.append(
                {
                    "frame": frame_no,
                    "source": occurrence.source,
                    "bank": occurrence.bank,
                    "id": occurrence.sfx_id,
                    "sound_bank_asset_index": None,
                    "rom_sequence_address": address & 0xFFFF,
                    "voice": channel.get("voice"),
                    "sound_id": sound & 0x3F,
                    "source_kind": "spc_sfx_channel",
                }
            )
    return explicit or matched_channels[:4]


def lift_steps_from_writes(
    writes: list[tuple[int, int, int, int, int]],
    command_frame: int = 0,
    voice_hint: int = 0,
    owned_voice_mask: int = 0,
    frame_timer_cycles: dict[int, int] | None = None,
) -> dict:
    voices = [VoiceState() for _ in range(8)]
    write_frames: list[dict[str, int]] = [dict() for _ in range(8)]
    noise_mask = 0
    echo_mask = 0
    active: dict[int, ActiveStep] = {}
    completed: list[ActiveStep] = []
    ownership_counts = {"owned_by_command": 0, "weak_update": 0, "carried_over": 0}

    for frame, addr, value, sample_offset, _timer_cycles in writes:
        reg = addr & 0x7F
        if reg == 0x3D:
            noise_mask = value
            continue
        if reg == 0x4D:
            echo_mask = value
            continue
        if reg == 0x4C:
            for voice in voices_from_mask(value):
                owned_parameters = command_owned_parameters(write_frames[voice], command_frame, frame)
                ownership = classify_voice_ownership(
                    owned_parameters,
                    voice,
                    owned_voice_mask,
                )
                ownership_counts[ownership] += 1
                active[voice] = ActiveStep(
                    voice=voice,
                    start_frame=frame,
                    start_sample_offset=sample_offset,
                    scheduler_tick_index=scheduler_tick_index(
                        frame,
                        sample_offset,
                        frame_timer_cycles or {},
                    ),
                    state=copy_voice_state(voices[voice]),
                    noise_enabled=bool(noise_mask & (1 << voice)),
                    echo_enabled=bool(echo_mask & (1 << voice)),
                    ownership=ownership,
                    owned_parameters=owned_parameters,
                    command_frame=command_frame,
                    voice_hint=voice_hint,
                )
            continue
        if reg == 0x5C:
            for voice in voices_from_mask(value):
                step = active.pop(voice, None)
                if step is not None:
                    step.keyoff_frame = frame
                    step.keyoff_sample_offset = sample_offset
                    completed.append(step)
            continue
        if (reg & 0x0F) <= 0x07:
            voice = reg >> 4
            parameter = reg & 0x0F
            update_voice_state(voices[voice], parameter, value)
            write_frames[voice][VOICE_PARAMETER_NAMES[parameter]] = frame
            if voice in active and parameter in (0x02, 0x03):
                active[voice].pitch_changes.append((frame, voice_pitch_word(voices[voice])))

    completed.extend(active.values())
    steps = [modern_step_from_active_step(step) for step in completed]
    return {
        "steps": [step for step in steps if step["ownership"] == "owned_by_command"],
        "context_steps": [step for step in steps if step["ownership"] != "owned_by_command"],
        "voice_ownership": ownership_counts,
    }


def command_owned_parameters(write_frames: dict[str, int], command_frame: int, keyon_frame: int) -> list[str]:
    return [
        name
        for name in VOICE_PARAMETER_NAMES
        if command_frame <= write_frames.get(name, -1) <= keyon_frame
    ]


def classify_voice_ownership(
    owned_parameters: list[str], voice: int = 0, owned_voice_mask: int = 0
) -> str:
    if owned_voice_mask and owned_voice_mask & (1 << voice) == 0:
        return "weak_update" if owned_parameters else "carried_over"
    if STRONG_OWNERSHIP_PARAMETERS.intersection(owned_parameters):
        return "owned_by_command"
    if owned_parameters:
        return "weak_update"
    return "carried_over"


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
        "pan": modern_pan(step.state),
        "echo": step.echo_enabled,
        "sample_offset": step.start_sample_offset,
        "command_delay_frames": max(0, step.start_frame - step.command_frame),
        "scheduler_tick_index": step.scheduler_tick_index,
        "volume_left": signed8(step.state.volume_left),
        "volume_right": signed8(step.state.volume_right),
        "dsp_pitch": pitch_word,
        "dsp_adsr1": step.state.adsr1,
        "dsp_adsr2": step.state.adsr2,
        "dsp_gain": step.state.gain,
        "envelope": modern_envelope(step.state),
        "duration_frames": max(1, duration_end - step.start_frame),
        "pitch_slide": pitch_slide,
        "ownership": step.ownership,
        "owned_parameters": step.owned_parameters,
        "evidence": {
            "start_frame": step.start_frame,
            "keyoff_frame": step.keyoff_frame,
            "command_frame": step.command_frame,
            "voice_hint": step.voice_hint,
            "dsp_pitch": pitch_word,
            "dsp_adsr1": step.state.adsr1,
            "dsp_adsr2": step.state.adsr2,
            "dsp_gain": step.state.gain,
            "sample_offset": step.start_sample_offset,
            "keyoff_sample_offset": step.keyoff_sample_offset,
        },
    }


def voice_pitch_word(state: VoiceState) -> int:
    return ((state.pitch_high & 0x3F) << 8) | state.pitch_low


def scheduler_tick_index(
    frame: int, sample_offset: int, frame_timer_cycles: dict[int, int]
) -> int:
    # Each game frame advances the DSP by 534 samples, or 22 modulo the
    # driver's 64-sample timer. Trace state is captured after that advance.
    post_frame_timer = frame_timer_cycles.get(frame, 22) & 0x3F
    start_timer = (post_frame_timer - (534 & 0x3F)) & 0x3F
    first_boundary = (-start_timer) & 0x3F
    if sample_offset < first_boundary:
        return 0
    return (sample_offset - first_boundary) // 64


def modern_pitch_from_dsp_pitch(pitch_word: int) -> int:
    if pitch_word <= 0:
        return 0
    return max(0, min(127, int(round(pitch_word / 32.0))))


def modern_volume(state: VoiceState) -> int:
    return max(abs(signed8(state.volume_left)), abs(signed8(state.volume_right)))


def modern_pan(state: VoiceState) -> int:
    left = abs(signed8(state.volume_left))
    right = abs(signed8(state.volume_right))
    strongest = max(left, right)
    if strongest == 0:
        return 0
    return max(-127, min(127, int(round((right - left) * 127 / strongest))))


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
    sequence_provenance = merge_sequence_provenance(variants)
    base = {
        "bank": bank,
        "id": sfx_id,
        "name": generated_program_name(bank, sfx_id),
        "occurrences": len(variants),
        "first_frames": [variant["first_frame"] for variant in variants],
        "sequence_provenance": sequence_provenance,
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
        variant_programs = []
        for variant_id, (signature, variant) in enumerate(
            sorted(signatures.items(), key=lambda item: item[1]["first_frame"])
        ):
            variant_programs.append(
                {
                    "variant": variant_id,
                    "variant_hash": stable_variant_hash(signature),
                    "name": generated_variant_program_name(bank, sfx_id, variant_id),
                    "first_frame": variant["first_frame"],
                    "first_frames": [
                        item["first_frame"]
                        for item in lifted
                        if json.dumps(
                            [step_signature(step) for step in item["steps"]],
                            sort_keys=True,
                            separators=(",", ":"),
                        )
                        == signature
                    ],
                    "steps": variant["steps"],
                    "context_steps": variant.get("context_steps", []),
                    "context_signature": context_signature_for_variant(variant),
                    "dsp_write_events": variant["dsp_write_events"],
                    "sequence_provenance": variant.get("sequence_provenance", []),
                }
            )
        return {
            **base,
            "status": "lifted",
            "steps": [],
            "variant_count": len(variant_programs),
            "variants": variant_programs,
            "notes": ["same SFX command produced multiple command-owned variants"],
        }
    variant = next(iter(signatures.values()))
    notes = []
    context_steps = []
    voice_ownership = {"owned_by_command": 0, "weak_update": 0, "carried_over": 0}
    for item in lifted:
        notes.extend(item.get("notes", []))
        context_steps.extend(item.get("context_steps", []))
        for key, count in item.get("voice_ownership", {}).items():
            voice_ownership[key] = voice_ownership.get(key, 0) + count
    return {
        **base,
        "status": "lifted",
        "source": variant["source"],
        "steps": variant["steps"],
        "variant_count": 1,
        "context_steps": context_steps,
        "voice_ownership": voice_ownership,
        "dsp_write_events": sum(item["dsp_write_events"] for item in lifted),
        "notes": sorted(set(notes)),
    }


def merge_sequence_provenance(variants: list[dict]) -> list[dict]:
    merged = []
    seen: set[tuple[int | None, int, int, int]] = set()
    for variant in variants:
        for item in variant.get("sequence_provenance", []):
            if not isinstance(item, dict):
                continue
            address = item.get("rom_sequence_address", item.get("sequence_address"))
            frame = item.get("frame", variant.get("first_frame", 0))
            if not isinstance(address, int) or not isinstance(frame, int):
                continue
            key = (
                item.get("sound_bank_asset_index"),
                address & 0xFFFF,
                int(item.get("voice", -1)) if isinstance(item.get("voice"), int) else -1,
                frame,
            )
            if key in seen:
                continue
            seen.add(key)
            merged.append(
                {
                    **item,
                    "frame": frame,
                    "rom_sequence_address": address & 0xFFFF,
                }
            )
    return merged


def generated_program_name(bank: int, sfx_id: int) -> str:
    return f"trace_sfx_{bank:02x}_{sfx_id:02x}"


def generated_variant_program_name(bank: int, sfx_id: int, variant_id: int) -> str:
    return f"{generated_program_name(bank, sfx_id)}_v{variant_id:02d}"


def stable_variant_hash(signature: str) -> int:
    hash_value = 2166136261
    for byte in signature.encode("utf-8"):
        hash_value = ((hash_value ^ byte) * 16777619) & 0xFFFFFFFF
    return hash_value


def context_signature_for_variant(variant: dict) -> dict:
    voice_mask = 0
    for step in variant["steps"]:
        voice_mask |= 1 << int(step["voice"])
    context_voice_mask = 0
    for step in variant.get("context_steps", []):
        context_voice_mask |= 1 << int(step["voice"])
    return {
        "source": variant["source"],
        "voice_mask": voice_mask,
        "context_voice_mask": context_voice_mask,
        "step_count": len(variant["steps"]),
    }


def step_signature(step: dict) -> dict:
    return {
        "voice": step["voice"],
        "pitch": step["pitch"],
        "instrument": step["instrument"],
        "waveform": step["waveform"],
        "volume": step["volume"],
        "pan": step["pan"],
        "echo": step["echo"],
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
        renderable = renderable_variants(program)
        for variant in renderable:
            render_rust_program(lines, program, variant)
    return "\n".join(lines)


def renderable_variants(program: dict) -> list[dict]:
    return program.get("variants") or [
        {
            "variant": 0,
            "variant_hash": stable_variant_hash(
                json.dumps(
                    [step_signature(step) for step in program["steps"]],
                    sort_keys=True,
                    separators=(",", ":"),
                )
            ),
            "name": program["name"],
            "steps": program["steps"],
            "context_signature": {
                "source": program.get("source", SFX_SLOT_SOURCES[program["bank"]]),
                "voice_mask": voice_mask_for_steps(program["steps"]),
                "context_voice_mask": voice_mask_for_steps(program.get("context_steps", [])),
                "step_count": len(program["steps"]),
            },
        }
    ]


def render_rust_module(programs: list[dict], array_name: str) -> str:
    definitions = [
        "// Generated by scripts/extract_modern_sfx_catalog.py.",
        "use super::{ModernSfxContextSignature, ModernSfxEnvelope, ModernSfxPitchSlide, ModernSfxProgram, ModernSfxStep, ModernSfxWaveform};",
        "",
    ]
    entries: list[str] = []
    for original in programs:
        if original["status"] not in {"lifted", "context_only", "no_key_on"}:
            continue
        program = original
        if original["status"] != "lifted":
            program = {
                **original,
                "steps": [],
                "context_steps": [],
                "source": SFX_SLOT_SOURCES[original["bank"]],
            }
        for variant in renderable_variants(program):
            render_rust_program(definitions, program, variant, entries)
    definitions.append(f"pub(super) const {array_name}: &[ModernSfxProgram] = &[")
    definitions.extend(f"    {line}" if line else "" for line in entries)
    definitions.append("];\n")
    return "\n".join(definitions)


def render_rust_program(
    lines: list[str], program: dict, variant: dict, program_lines: list[str] | None = None
) -> None:
    const_name = variant["name"].upper()
    lines.append(f"const {const_name}_STEPS: &[ModernSfxStep] = &[")
    for step in variant["steps"]:
        envelope = step["envelope"]
        lines.extend(
            [
                "    ModernSfxStep {",
                f"        voice: {step['voice']},",
                f"        pitch: {step['pitch']},",
                f"        instrument: {step['instrument']},",
                f"        waveform: ModernSfxWaveform::{step['waveform']},",
                f"        volume: {step['volume']},",
                f"        pan: {step['pan']},",
                f"        echo: {str(step['echo']).lower()},",
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
    context = variant["context_signature"]
    target = lines if program_lines is None else program_lines
    target.extend(
        [
            "ModernSfxProgram {",
            f"    bank: {program['bank']},",
            f"    id: 0x{program['id']:02x},",
            f"    variant: {variant['variant']},",
            f"    variant_hash: 0x{variant['variant_hash']:08x},",
            f"    name: \"{variant['name']}\",",
            "    context: ModernSfxContextSignature {",
            f"        source_slot: {program['bank']},",
            f"        voice_mask: 0x{context['voice_mask']:02x},",
            f"        context_voice_mask: 0x{context['context_voice_mask']:02x},",
            f"        step_count: {context['step_count']},",
            "    },",
            f"    steps: {const_name}_STEPS,",
            "},",
            "",
        ]
    )


def voice_mask_for_steps(steps: list[dict]) -> int:
    mask = 0
    for step in steps:
        mask |= 1 << int(step["voice"])
    return mask


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
