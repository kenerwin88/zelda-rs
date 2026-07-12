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
    voices = [VoiceState() for _ in range(8)]
    active: dict[int, dict] = {}
    tracks: dict[int, list[dict]] = {}
    track_origins: dict[int, int] = {}
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
        reported_track = int(music[2]) if len(music) >= 3 else 0
        if 0 < reported_track < 0xF0:
            active_track = reported_track
        track = active_track if reported_track >= 0xF0 else reported_track
        if 0 < track < 0xF0:
            track_origins.setdefault(track, frame_no)
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
            if len(raw) < 2:
                continue
            addr, value = int(raw[0]), int(raw[1])
            sample_offset = int(raw[2]) if len(raw) >= 3 else 0
            reg = addr & 0x7F
            if reg in (0x0C, 0x1C, 0x2C, 0x3C, 0x0D, 0x2D, 0x3D, 0x4D, 0x6C, 0x6D, 0x7D) or reg & 0x0F == 0x0F:
                changed = last_global_values.get(reg) != value
                last_global_values[reg] = value
                if changed and 0 < track < 0xF0:
                    global_events.setdefault(track, []).append(
                        {
                            "frame": frame_no,
                            "sample_offset": sample_offset,
                            "register": reg,
                            "value": value,
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
                if changed and 0 < track < 0xF0 and sfx_voice_mask & (1 << voice) == 0:
                    global_events.setdefault(track, []).append(
                        {
                            "frame": frame_no,
                            "sample_offset": sample_offset,
                            "register": reg,
                            "value": value,
                        }
                    )
                continue
            if reg == 0x4D:
                echo_enable_mask = value
                continue
            if reg == 0x4C:
                for voice in range(8):
                    if value & (1 << voice) == 0:
                        continue
                    if sfx_voice_mask & (1 << voice) != 0:
                        continue
                    state = voices[voice]
                    pitch_word = (state.pitch_high << 8) | state.pitch_low
                    note = {
                        "track": track,
                        "voice": voice,
                        "frame": frame_no,
                        "sample_offset": sample_offset,
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
                        note["duration_frames"] = frame_no - note["frame"]
                        note["keyoff_sample_offset"] = sample_offset

    for notes in tracks.values():
        if not notes:
            continue
        origin = notes[0]["frame"]
        for note in notes:
            note["start_frame"] = note["frame"] - origin
    for track, events in global_events.items():
        origin = track_origins[track]
        for event in events:
            event["start_frame"] = event["frame"] - origin
    return {
        "format": "zelda3-modern-music-trace-v1",
        "tracks": [
            {
                "track": track,
                "lead_in_frames": notes[0]["frame"] - track_origins[track],
                "notes": notes,
                "global_events": global_events.get(track, []),
            }
            for track, notes in sorted(tracks.items())
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="+", type=Path)
    parser.add_argument("--json-out", type=Path, required=True)
    parser.add_argument("--update-tsv", type=Path)
    args = parser.parse_args()
    catalog = extract_music(load_frames(args.trace))
    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(catalog, indent=2, sort_keys=True) + "\n")
    if args.update_tsv is not None:
        update_tsv(args.update_tsv, catalog)
    print(
        "modern music catalog: "
        f"tracks={len(catalog['tracks'])} "
        f"notes={sum(len(track['notes']) for track in catalog['tracks'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
