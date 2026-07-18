#!/usr/bin/env python3
"""Lift route-observed music notes from focused Rust DSP audio traces.

This is an offline authoring bridge. It reconstructs voice parameters around
DSP key-on/key-off writes and emits backend-neutral note facts; the modern
runtime must consume reviewed generated data, not DSP registers.
"""

from __future__ import annotations

import argparse
import csv
import json
from dataclasses import dataclass
from pathlib import Path


@dataclass
class VoiceState:
    volume_left: int = 0
    volume_right: int = 0
    pitch_low: int = 0
    pitch_high: int = 0
    instrument: int = 0
    adsr1: int = 0
    adsr2: int = 0
    gain: int = 0


def signed8(value: int) -> int:
    return value - 256 if value >= 128 else value


def modern_pan(state: VoiceState) -> int:
    left = abs(signed8(state.volume_left))
    right = abs(signed8(state.volume_right))
    strongest = max(left, right)
    if strongest == 0:
        return 0
    return max(-127, min(127, int(round((right - left) * 127 / strongest))))


def load_frames(paths: list[Path]) -> list[dict]:
    frames: list[dict] = []
    for path in paths:
        for line in path.read_text(encoding="utf-8").splitlines():
            try:
                value = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(value, dict) and "frame" in value:
                frames.append(value)
    return sorted(frames, key=lambda frame: frame["frame"])


def extract_music(frames: list[dict]) -> dict:
    frame_start_samples: dict[int, int] = {}
    frame_dsp_origins: dict[int, int] = {}
    sample_cursor = 0
    previous_frame_no: int | None = None
    previous_dsp_frame_no: int | None = None
    previous_dsp_last_output: int | None = None
    dsp_stream_phase0_cycle: int | None = None
    for frame in frames:
        frame_no = int(frame["frame"])
        if previous_frame_no is not None and frame_no > previous_frame_no + 1:
            sample_cursor += (frame_no - previous_frame_no - 1) * 534
        frame_start_samples.setdefault(frame_no, sample_cursor)
        audio_sample_frames = int(frame.get("audio_sample_frames") or 534)
        sample_cursor += audio_sample_frames
        dsp_clock = frame.get("dsp_clock")
        if isinstance(dsp_clock, dict):
            output_count = int(dsp_clock.get("output_count") or 0)
            if output_count > 0:
                first_output = int(dsp_clock["first_output_cycle"])
                last_output = int(dsp_clock["last_output_cycle"])
                expected_span = (output_count - 1) * 32
                if last_output - first_output != expected_span:
                    raise ValueError(
                        f"frame {frame_no} DSP output clock is not 32-cycle periodic"
                    )
                if (
                    previous_dsp_frame_no is not None
                    and frame_no == previous_dsp_frame_no + 1
                    and first_output != previous_dsp_last_output + 32
                ):
                    raise ValueError(
                        f"frame {frame_no} DSP clock is discontinuous across callbacks"
                    )
                if dsp_stream_phase0_cycle is None:
                    # The Libretro callback can drain samples buffered by an
                    # earlier retro_run. Anchor the raw DSP stream once, then
                    # address callback starts by cumulative delivered samples.
                    dsp_stream_phase0_cycle = (
                        first_output - 27 - frame_start_samples[frame_no] * 32
                    )
                previous_dsp_frame_no = frame_no
                previous_dsp_last_output = last_output
            if dsp_stream_phase0_cycle is not None:
                frame_dsp_origins[frame_no] = (
                    dsp_stream_phase0_cycle + frame_start_samples[frame_no] * 32
                )
        previous_frame_no = frame_no

    voices = [VoiceState() for _ in range(8)]
    active: dict[int, dict] = {}
    tracks: dict[int, list[dict]] = {}
    track_origins: dict[int, int] = {}
    global_track_origins: dict[int, int] = {}
    global_events: dict[int, list[dict]] = {}
    last_global_values: dict[int, int] = {}
    last_voice_values: dict[tuple[int, int], int] = {}
    recent_sfx_command_frame: int | None = None
    echo_enable_mask = 0
    active_track = 0

    for frame in frames:
        frame_no = int(frame["frame"])
        if int(frame.get("modern_sfx_known") or 0) + int(
            frame.get("modern_sfx_unknown") or 0
        ) > 0:
            recent_sfx_command_frame = frame_no
        music = frame.get("music") or [0, 0, 0]
        command_track = int(music[0]) if music else 0
        reported_track = int(music[2]) if len(music) >= 3 else 0
        if 0 < command_track < 0xF0:
            active_track = command_track
            global_track_origins.setdefault(command_track, frame_no)
        if 0 < reported_track < 0xF0:
            active_track = reported_track
        track = active_track if reported_track >= 0xF0 else reported_track
        if track == 0 and 0 < command_track < 0xF0:
            track = command_track
        if 0 < track < 0xF0:
            track_origins.setdefault(track, frame_no)
            global_track_origins.setdefault(track, frame_no)
        sfx_voice_mask = 0
        channels = frame.get("sfx_channels")
        queue = frame.get("queue")
        if not isinstance(channels, list) and isinstance(queue, dict):
            channels = queue.get("sfx_channels")
        for channel in channels or []:
            voice = channel.get("voice")
            sound = int(channel.get("sound") or 0) & 0x3F
            if isinstance(voice, int) and 0 <= voice < 8 and sound != 0:
                sfx_voice_mask |= 1 << voice
        for raw in frame.get("dsp_write_events") or []:
            if isinstance(raw, dict):
                addr = int(raw["register"])
                value = int(raw["value"])
                sample_offset = int(raw.get("legacy_sample_offset") or 0)
                dsp_phase = int(raw.get("phase") or 0)
                event_sfx_voice_mask = int(
                    raw.get("sfx_voice_mask", sfx_voice_mask)
                ) & 0xFF
                absolute_cycle = raw.get("absolute_cycle")
                absolute_cycle = (
                    int(absolute_cycle) if absolute_cycle is not None else None
                )
            else:
                if len(raw) < 2:
                    continue
                addr, value = int(raw[0]), int(raw[1])
                sample_offset = int(raw[2]) if len(raw) >= 3 else 0
                dsp_phase = int(raw[3]) if len(raw) >= 4 else 0
                event_sfx_voice_mask = (
                    int(raw[4]) & 0xFF if len(raw) >= 5 else sfx_voice_mask
                )
                absolute_cycle = int(raw[5]) if len(raw) >= 6 else None
            if absolute_cycle is not None and dsp_stream_phase0_cycle is not None:
                stream_cycle = absolute_cycle - dsp_stream_phase0_cycle
                if stream_cycle < 0 or stream_cycle % 32 != dsp_phase:
                    raise ValueError(
                        f"frame {frame_no} DSP event clock/phase mismatch: "
                        f"cycle={stream_cycle} phase={dsp_phase}"
                    )
                absolute_event_sample = stream_cycle // 32
            else:
                absolute_event_sample = (
                    frame_start_samples[frame_no] + sample_offset
                )
            reg = addr & 0x7F
            if reg in (0x0C, 0x1C, 0x2C, 0x3C, 0x0D, 0x2D, 0x3D, 0x4D, 0x6C, 0x6D, 0x7D) or reg & 0x0F == 0x0F:
                catalog_value = value
                if reg in (0x2D, 0x3D, 0x4D) and event_sfx_voice_mask:
                    previous_music_value = last_global_values.get(reg, 0)
                    catalog_value = (
                        (value & ~event_sfx_voice_mask)
                        | (previous_music_value & event_sfx_voice_mask)
                    )
                changed = last_global_values.get(reg) != catalog_value
                last_global_values[reg] = catalog_value
                if changed and 0 < track < 0xF0:
                    global_events.setdefault(track, []).append(
                        {
                            "frame": frame_no,
                            "sample_offset": sample_offset,
                            "dsp_phase": dsp_phase,
                            "absolute_cycle": absolute_cycle,
                            "absolute_sample": absolute_event_sample,
                            "register": reg,
                            "value": catalog_value,
                        }
                    )
            if (reg & 0x0F) <= 0x07:
                voice = reg >> 4
                parameter = reg & 0x0F
                if parameter == 0:
                    voices[voice].volume_left = value
                elif parameter == 1:
                    voices[voice].volume_right = value
                elif parameter == 2:
                    voices[voice].pitch_low = value
                elif parameter == 3:
                    voices[voice].pitch_high = value & 0x3F
                elif parameter == 4:
                    voices[voice].instrument = value
                elif parameter == 5:
                    voices[voice].adsr1 = value
                elif parameter == 6:
                    voices[voice].adsr2 = value
                elif parameter == 7:
                    voices[voice].gain = value
                key = (voice, parameter)
                changed = last_voice_values.get(key) != value
                last_voice_values[key] = value
                if (
                    changed
                    and 0 < track < 0xF0
                    and event_sfx_voice_mask & (1 << voice) == 0
                ):
                    global_events.setdefault(track, []).append(
                        {
                            "frame": frame_no,
                            "sample_offset": sample_offset,
                            "dsp_phase": dsp_phase,
                            "absolute_cycle": absolute_cycle,
                            "absolute_sample": absolute_event_sample,
                            "register": reg,
                            "value": value,
                        }
                    )
                continue
            if reg == 0x4D:
                echo_enable_mask = value
                continue
            if reg == 0x4C:
                if not 0 < track < 0xF0:
                    continue
                for voice in range(8):
                    if value & (1 << voice) == 0:
                        continue
                    if event_sfx_voice_mask & (1 << voice) != 0:
                        continue
                    state = voices[voice]
                    pitch_word = (state.pitch_high << 8) | state.pitch_low
                    note = {
                        "track": track,
                        "voice": voice,
                        "frame": frame_no,
                        "sample_offset": sample_offset,
                        "absolute_sample": absolute_event_sample,
                        "kon_phase": dsp_phase,
                        "dsp_pitch": pitch_word,
                        "pitch": max(0, min(127, int(round(pitch_word / 128.0)))),
                        "instrument": state.instrument,
                        "volume_left": signed8(state.volume_left),
                        "volume_right": signed8(state.volume_right),
                        "adsr1": state.adsr1,
                        "adsr2": state.adsr2,
                        "gain": state.gain,
                        "echo_send": bool(echo_enable_mask & (1 << voice)),
                        "volume": max(
                            abs(signed8(state.volume_left)),
                            abs(signed8(state.volume_right)),
                        ),
                        "pan": modern_pan(state),
                        "duration_frames": 0,
                        "keyoff_sample_offset": 0,
                        "keyoff_phase": 0,
                    }
                    if (
                        recent_sfx_command_frame is not None
                        and 0 <= frame_no - recent_sfx_command_frame <= 12
                    ):
                        note["preceding_sfx_command_frame"] = recent_sfx_command_frame
                        note["frames_after_sfx_command"] = (
                            frame_no - recent_sfx_command_frame
                        )
                    tracks.setdefault(track, []).append(note)
                    active[voice] = note
                continue
            if reg == 0x5C:
                for voice in range(8):
                    if value & (1 << voice) == 0:
                        continue
                    note = active.pop(voice, None)
                    if note is not None:
                        note["keyoff_absolute_sample"] = absolute_event_sample
                        note["keyoff_phase"] = dsp_phase

    track_lead_frames: dict[int, int] = {}
    for track, notes in tracks.items():
        if not notes:
            continue
        track_origin_sample = frame_start_samples[track_origins[track]]
        first_relative_sample = notes[0]["absolute_sample"] - track_origin_sample
        lead_in_frames = max(0, first_relative_sample // 534)
        track_lead_frames[track] = lead_in_frames
        lead_in_samples = lead_in_frames * 534
        for note in notes:
            relative_sample = (
                note.pop("absolute_sample") - track_origin_sample - lead_in_samples
            )
            note["start_frame"], note["sample_offset"] = divmod(relative_sample, 534)
            keyoff_absolute = note.pop("keyoff_absolute_sample", None)
            if keyoff_absolute is None:
                note["duration_frames"] = 0
                note["keyoff_sample_offset"] = 0
            else:
                keyoff_relative = keyoff_absolute - track_origin_sample - lead_in_samples
                keyoff_frame, note["keyoff_sample_offset"] = divmod(keyoff_relative, 534)
                note["duration_frames"] = keyoff_frame - note["start_frame"]
    for track, events in global_events.items():
        origin_frame = global_track_origins[track]
        track_origin_sample = frame_start_samples[origin_frame]
        track_origin_cycle = frame_dsp_origins.get(origin_frame)
        for event in events:
            absolute_sample = event.pop("absolute_sample")
            absolute_cycle = event.pop("absolute_cycle")
            dsp_phase = event.pop("dsp_phase")
            event.pop("frame", None)
            event.pop("sample_offset", None)
            if absolute_cycle is not None and track_origin_cycle is not None:
                relative_cycle = absolute_cycle - track_origin_cycle
                if relative_cycle < 0:
                    raise ValueError(
                        f"track {track:02x} DSP event precedes its command origin"
                    )
                if relative_cycle % 32 != dsp_phase:
                    raise ValueError(
                        f"track {track:02x} DSP clock phase mismatch: "
                        f"cycle={relative_cycle} phase={dsp_phase}"
                    )
                event["dsp_cycle"] = relative_cycle
            else:
                relative_sample = absolute_sample - track_origin_sample
                event["dsp_cycle"] = relative_sample * 32 + dsp_phase
    catalog_tracks = sorted(set(tracks) | set(global_events))
    return {
        "format": "zelda3-modern-music-trace-v1",
        "tracks": [
            {
                "track": track,
                "lead_in_frames": track_lead_frames.get(track, 0),
                "notes": tracks.get(track, []),
                "global_events": global_events.get(track, []),
            }
            for track in catalog_tracks
            if 0 < track < 0xF0
        ],
    }


def update_tsv(path: Path, catalog: dict) -> None:
    """Replace only the extracted tracks in the packed catalog source."""
    replacements: dict[int, list[str]] = {}
    for track in catalog["tracks"]:
        track_id = int(track["track"])
        notes = track["notes"]
        if not notes:
            continue
        lead_in = int(track.get("lead_in_frames", 0))
        replacements[track_id] = [
            "\t".join(
                str(value)
                for value in (
                    f"{track_id:02x}", lead_in, note["voice"], note["pitch"],
                    note["instrument"], note["volume"], note["pan"],
                    note["start_frame"], note["duration_frames"], note["dsp_pitch"],
                    note["sample_offset"], note["volume_left"], note["volume_right"],
                    note["adsr1"], note["adsr2"], note["gain"],
                    int(note["echo_send"]), note["keyoff_sample_offset"],
                    note["kon_phase"], note["keyoff_phase"],
                )
            )
            for note in notes
        ]

    rows = list(csv.reader(path.read_text(encoding="utf-8").splitlines(), delimiter="\t"))
    retained: dict[int, list[str]] = {}
    header = next(("\t".join(row) for row in rows if row and row[0].startswith("#")), "")
    for row in rows:
        if not row or row[0].startswith("#"):
            continue
        track_id = int(row[0], 16)
        if track_id not in replacements:
            retained.setdefault(track_id, []).append("\t".join(row))
    retained.update(replacements)
    lines = [header, *(line for track_id in sorted(retained) for line in retained[track_id])]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def update_globals_tsv(path: Path, catalog: dict) -> None:
    """Replace only the extracted tracks in the packed global-event source."""
    replacements: dict[int, list[str]] = {}
    for track in catalog["tracks"]:
        track_id = int(track["track"])
        replacements[track_id] = [
            "\t".join(
                (
                    f"{track_id:02x}",
                    str(event["dsp_cycle"]),
                    f"{int(event['register']):02x}",
                    str(event["value"]),
                )
            )
            for event in track.get("global_events", [])
        ]

    rows = list(csv.reader(path.read_text(encoding="utf-8").splitlines(), delimiter="\t"))
    retained: dict[int, list[str]] = {}
    header = "# track\tdsp_cycle\tregister\tvalue"
    for row in rows:
        if not row or row[0].startswith("#"):
            continue
        track_id = int(row[0], 16)
        if track_id not in replacements:
            if len(row) == 4:
                normalized = row
            elif len(row) in (5, 6):
                phase = int(row[5]) if len(row) == 6 else 0
                dsp_cycle = (int(row[1]) * 534 + int(row[2])) * 32 + phase
                normalized = [row[0], str(dsp_cycle), row[3], row[4]]
            else:
                raise ValueError(f"invalid global catalog row: {row!r}")
            retained.setdefault(track_id, []).append("\t".join(normalized))
    retained.update(replacements)
    for track_rows in retained.values():
        track_rows.sort(key=lambda line: int(line.split("\t")[1]))
    lines = [header, *(line for track_id in sorted(retained) for line in retained[track_id])]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def update_tsv_phases(path: Path, catalog: dict) -> tuple[int, int]:
    """Add captured KON/KOFF phases without replacing the reviewed note set."""
    notes_by_track = {
        int(track["track"]): list(track.get("notes", []))
        for track in catalog["tracks"]
    }
    used: dict[int, set[int]] = {track: set() for track in notes_by_track}
    rows = list(csv.reader(path.read_text(encoding="utf-8").splitlines(), delimiter="\t"))
    output = [
        "# track\tlead_in\tvoice\tpitch\tinstrument\tvolume\tpan\tstart_frame"
        "\tduration_frames\tdsp_pitch\tsample_offset\tvolume_left\tvolume_right"
        "\tadsr1\tadsr2\tgain\techo_send\tkeyoff_sample_offset\tkon_phase\tkeyoff_phase"
    ]
    matched = 0
    selected_rows = 0

    for row in rows:
        if not row or row[0].startswith("#"):
            continue
        if len(row) not in (18, 20):
            raise ValueError(f"invalid music catalog row: {row!r}")
        track = int(row[0], 16)
        candidates = notes_by_track.get(track)
        if candidates is None:
            output.append("\t".join(row[:18] + (row[18:20] if len(row) == 20 else ["0", "0"])))
            continue

        selected_rows += 1
        semantic = (
            int(row[2]), int(row[3]), int(row[4]), int(row[5]), int(row[6]),
            int(row[9]), int(row[11]), int(row[12]), int(row[13]), int(row[14]),
            int(row[15]), bool(int(row[16])),
        )
        start = int(row[7]) * 534 + int(row[10])
        choices = []
        for index, note in enumerate(candidates):
            if index in used[track]:
                continue
            candidate_semantic = (
                int(note["voice"]), int(note["pitch"]), int(note["instrument"]),
                int(note["volume"]), int(note["pan"]), int(note["dsp_pitch"]),
                int(note["volume_left"]), int(note["volume_right"]), int(note["adsr1"]),
                int(note["adsr2"]), int(note["gain"]), bool(note["echo_send"]),
            )
            if candidate_semantic == semantic:
                candidate_start = int(note["start_frame"]) * 534 + int(note["sample_offset"])
                choices.append((abs(candidate_start - start), index, note))
        if choices:
            _, index, note = min(choices, key=lambda choice: (choice[0], choice[1]))
            used[track].add(index)
            phases = [str(int(note["kon_phase"])), str(int(note["keyoff_phase"]))]
            matched += 1
        else:
            phases = row[18:20] if len(row) == 20 else ["0", "0"]
        output.append("\t".join(row[:18] + phases))

    path.write_text("\n".join(output) + "\n", encoding="utf-8")
    return matched, selected_rows


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="+", type=Path)
    parser.add_argument("--json-out", type=Path, required=True)
    parser.add_argument("--update-tsv", type=Path)
    parser.add_argument("--update-tsv-phases", type=Path)
    parser.add_argument("--update-globals-tsv", type=Path)
    parser.add_argument("--track", action="append", type=lambda value: int(value, 0))
    args = parser.parse_args()
    catalog = extract_music(load_frames(args.trace))
    if args.track:
        selected = set(args.track)
        catalog["tracks"] = [
            track for track in catalog["tracks"] if int(track["track"]) in selected
        ]
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(catalog, indent=2, sort_keys=True) + "\n")
    if args.update_tsv is not None:
        update_tsv(args.update_tsv, catalog)
    if args.update_tsv_phases is not None:
        matched, selected_rows = update_tsv_phases(args.update_tsv_phases, catalog)
        print(f"modern music phase update: matched={matched}/{selected_rows}")
    if args.update_globals_tsv is not None:
        update_globals_tsv(args.update_globals_tsv, catalog)
    print(
        "modern music catalog: "
        f"tracks={len(catalog['tracks'])} "
        f"notes={sum(len(track['notes']) for track in catalog['tracks'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
