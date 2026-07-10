#!/usr/bin/env python3
"""Harvest trace-backed modern SFX candidates from replay routes.

This is the orchestration layer above `extract_modern_sfx_catalog.py`:

1. Run or read a broad Rust audio trace.
2. Detect SFX command transitions.
3. Re-run focused trace windows with `ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME`.
4. Lift focused DSP writes into modern SFX candidate programs.
5. Write coverage, JSON candidates, and optional Rust snippets for review.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import replace
from pathlib import Path
from typing import Iterable

import extract_modern_sfx_catalog as extractor


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = Path(os.environ.get("ZELDA3_C_REPO", str(ROOT.parent / "zelda3")))
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(DEFAULT_C_ROOT / "zelda3.sfc")))
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_RUST_BIN = ROOT / "target" / "release" / "zelda3"
DEFAULT_FINAL_FRAME = 1_073_092
DEFAULT_OUTPUT_DIR = ROOT / "target" / "modern-sfx-harvest"


class HarvestFailure(RuntimeError):
    pass


def run_replay_trace(
    *,
    rust_bin: Path,
    rom: Path,
    save: Path,
    frames: int,
    audio_trace_log: int,
    dsp_writes_frame: int | None = None,
    dsp_writes_frame_range: tuple[int, int] | None = None,
) -> list[dict]:
    command = [
        str(rust_bin),
        "--replay-save",
        str(rom),
        str(save),
        str(frames),
        "--audio-trace-log",
        str(audio_trace_log),
    ]
    env = os.environ.copy()
    env.setdefault("SDL_VIDEODRIVER", "dummy")
    env.setdefault("SDL_AUDIODRIVER", "dummy")
    env.setdefault("SDL_RENDER_DRIVER", "software")
    if dsp_writes_frame is not None:
        env["ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME"] = str(dsp_writes_frame)
    if dsp_writes_frame_range is not None:
        start, end = dsp_writes_frame_range
        env["ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME_RANGE"] = f"{start}:{end}"

    try:
        result = subprocess.run(
            command,
            cwd=ROOT,
            env=env,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as exc:
        raise HarvestFailure(f"failed to start replay trace: {exc}") from exc
    if result.returncode != 0:
        stderr_tail = "\n".join(result.stderr.splitlines()[-30:])
        raise HarvestFailure(
            f"replay trace failed with exit {result.returncode}: {' '.join(command)}\n{stderr_tail}"
        )
    return extractor.load_trace_stream(result.stdout.splitlines(), "<replay stdout>")


def write_jsonl(path: Path, frames: Iterable[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as stream:
        for frame in frames:
            stream.write(json.dumps(frame, sort_keys=True, separators=(",", ":")) + "\n")


def read_or_run_broad_trace(args: argparse.Namespace) -> tuple[list[dict], Path]:
    broad_path = args.output_dir / "broad-audio-trace.jsonl"
    if args.broad_trace_jsonl is not None:
        frames = extractor.load_trace([args.broad_trace_jsonl])
        write_jsonl(broad_path, frames)
        return frames, broad_path

    frames = run_replay_trace(
        rust_bin=args.rust_bin,
        rom=args.rom,
        save=args.save,
        frames=args.frames,
        audio_trace_log=args.audio_trace_log,
    )
    write_jsonl(broad_path, frames)
    return frames, broad_path


def occurrence_focus_frames(
    occurrences: list[extractor.SfxOccurrence],
    max_occurrences: int | None,
) -> list[extractor.SfxOccurrence]:
    selected = occurrences
    if max_occurrences is not None:
        selected = selected[:max_occurrences]
    return selected


def focused_trace_path(output_dir: Path, occurrence: extractor.SfxOccurrence) -> Path:
    return (
        output_dir
        / "focused"
        / f"frame-{occurrence.frame:08d}-sfx-{occurrence.bank:02x}-{occurrence.sfx_id:02x}.jsonl"
    )


def load_or_run_focused_trace(
    args: argparse.Namespace,
    occurrence: extractor.SfxOccurrence,
) -> tuple[list[dict], Path, bool]:
    path = focused_trace_path(args.output_dir, occurrence)
    if path.exists() and not args.force:
        return extractor.load_trace([path]), path, True
    if args.skip_focused_runs:
        return [], path, False

    frames = run_replay_trace(
        rust_bin=args.rust_bin,
        rom=args.rom,
        save=args.save,
        frames=occurrence.frame + args.window_frames,
        audio_trace_log=1,
        dsp_writes_frame_range=(occurrence.frame, occurrence.frame + args.window_frames),
    )
    write_jsonl(path, frames)
    return frames, path, False


def lift_from_focused_trace(
    frames: list[dict],
    occurrence: extractor.SfxOccurrence,
    window_frames: int,
) -> dict:
    if not frames:
        return {
            "bank": occurrence.bank,
            "id": occurrence.sfx_id,
            "name": extractor.generated_program_name(occurrence.bank, occurrence.sfx_id),
            "source": occurrence.source,
            "first_frame": occurrence.frame,
            "window_frames": window_frames,
            "trace_frames": [],
            "dsp_write_events": 0,
            "status": "missing_focused_trace",
            "steps": [],
            "notes": ["focused replay was skipped or produced no trace frames"],
        }
    focus_index = next(
        (
            index
            for index, frame in enumerate(frames)
            if int(frame.get("frame", -1)) >= occurrence.frame
        ),
        None,
    )
    if focus_index is None:
        return {
            "bank": occurrence.bank,
            "id": occurrence.sfx_id,
            "name": extractor.generated_program_name(occurrence.bank, occurrence.sfx_id),
            "source": occurrence.source,
            "first_frame": occurrence.frame,
            "window_frames": window_frames,
            "trace_frames": [int(frame.get("frame", 0)) for frame in frames],
            "dsp_write_events": 0,
            "status": "missing_focused_frame",
            "steps": [],
            "notes": [f"focused trace did not include frame {occurrence.frame}"],
        }
    focused_occurrence = replace(occurrence, frame_index=focus_index)
    return extractor.lift_occurrence(frames, focused_occurrence, window_frames)


def merge_harvested_programs(variants: list[dict]) -> list[dict]:
    observed: dict[tuple[int, int], list[dict]] = {}
    for variant in variants:
        observed.setdefault((variant["bank"], variant["id"]), []).append(variant)
    return [
        merge_harvested_variants(bank, sfx_id, values)
        for (bank, sfx_id), values in sorted(observed.items())
    ]


def merge_harvested_variants(bank: int, sfx_id: int, variants: list[dict]) -> dict:
    supported = [
        variant
        for variant in variants
        if variant["status"] in {"lifted", "ambiguous", "missing_dsp_events", "no_key_on"}
    ]
    if supported:
        return extractor.merge_variants(bank, sfx_id, supported)
    notes = []
    for variant in variants:
        notes.extend(variant.get("notes", []))
    return {
        "bank": bank,
        "id": sfx_id,
        "name": extractor.generated_program_name(bank, sfx_id),
        "occurrences": len(variants),
        "first_frames": [variant["first_frame"] for variant in variants],
        "status": variants[0]["status"] if variants else "missing_focused_trace",
        "steps": [],
        "notes": sorted(set(notes)),
    }


def coverage_for(
    programs: list[dict],
    occurrences: list[extractor.SfxOccurrence],
    variants: list[dict],
) -> dict:
    program_gaps = sum(1 for program in programs if program["status"] != "lifted")
    focused_command_gaps = sum(1 for variant in variants if variant["status"] != "lifted")
    coverage = {
        "commands": len(occurrences),
        "programs": len(programs),
        "lifted": sum(1 for program in programs if program["status"] == "lifted"),
        "ambiguous": sum(1 for program in programs if program["status"] == "ambiguous"),
        "missing_dsp_events": sum(
            1 for program in programs if program["status"] == "missing_dsp_events"
        ),
        "no_key_on": sum(1 for program in programs if program["status"] == "no_key_on"),
        "missing_focused_trace": sum(
            1 for program in programs if program["status"] == "missing_focused_trace"
        ),
        "missing_focused_frame": sum(
            1 for program in programs if program["status"] == "missing_focused_frame"
        ),
        "focused_lifted_commands": sum(
            1 for variant in variants if variant["status"] == "lifted"
        ),
        "focused_command_gaps": focused_command_gaps,
        "program_gaps": program_gaps,
    }
    coverage["gaps"] = max(program_gaps, focused_command_gaps)
    return coverage


def harvest(args: argparse.Namespace) -> dict:
    args.output_dir.mkdir(parents=True, exist_ok=True)
    broad_frames, broad_path = read_or_run_broad_trace(args)
    occurrences = extractor.discover_sfx_occurrences(broad_frames)
    selected = occurrence_focus_frames(occurrences, args.max_occurrences)
    variants = []
    focused = []

    for occurrence in selected:
        frames, path, reused = load_or_run_focused_trace(args, occurrence)
        variant = lift_from_focused_trace(frames, occurrence, args.window_frames)
        variants.append(variant)
        focused.append(
            {
                "frame": occurrence.frame,
                "bank": occurrence.bank,
                "id": occurrence.sfx_id,
                "source": occurrence.source,
                "path": str(path),
                "reused": reused,
                "status": variant["status"],
            }
        )

    programs = merge_harvested_programs(variants)
    coverage = coverage_for(programs, selected, variants)
    coverage["discovered_commands"] = len(occurrences)
    coverage["focused_commands"] = len(selected)
    return {
        "broad_trace": str(broad_path),
        "focused_traces": focused,
        "coverage": coverage,
        "programs": programs,
    }


def render_report(result: dict) -> str:
    coverage = result["coverage"]
    lines = [
        "# Modern SFX Harvest Report",
        "",
        f"- Broad trace: `{result['broad_trace']}`",
        f"- Discovered commands: {coverage['discovered_commands']}",
        f"- Focused commands: {coverage['focused_commands']}",
        f"- Programs: {coverage['programs']}",
        f"- Lifted: {coverage['lifted']}",
        f"- Gaps: {coverage['gaps']}",
        "",
        "| Status | Bank | Id | Occurrences | Frames | Notes |",
        "|---|---:|---:|---:|---|---|",
    ]
    for program in result["programs"]:
        notes = "; ".join(program.get("notes", []))
        frames = ", ".join(str(frame) for frame in program.get("first_frames", []))
        lines.append(
            f"| {program['status']} | {program['bank']} | 0x{program['id']:02x} | "
            f"{program.get('occurrences', 0)} | {frames} | {notes} |"
        )
    return "\n".join(lines) + "\n"


def default_output_paths(args: argparse.Namespace) -> None:
    if args.json_out is None:
        args.json_out = args.output_dir / "modern-sfx-harvest.json"
    if args.rust_out is None:
        args.rust_out = args.output_dir / "modern-sfx-candidates.rs"
    if args.report_out is None:
        args.report_out = args.output_dir / "modern-sfx-harvest.md"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--broad-trace-jsonl", type=Path)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--json-out", type=Path)
    parser.add_argument("--rust-out", type=Path)
    parser.add_argument("--report-out", type=Path)
    parser.add_argument("--rust-bin", type=Path, default=DEFAULT_RUST_BIN)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--frames", type=int, default=DEFAULT_FINAL_FRAME)
    parser.add_argument("--audio-trace-log", type=int, default=1)
    parser.add_argument("--window-frames", type=int, default=12)
    parser.add_argument("--max-occurrences", type=int)
    parser.add_argument("--force", action="store_true", help="re-run focused traces even when cached traces exist")
    parser.add_argument(
        "--skip-focused-runs",
        action="store_true",
        help="only analyze the broad trace and mark focused captures as gaps",
    )
    parser.add_argument("--fail-on-gaps", action="store_true")
    args = parser.parse_args(argv)
    if args.frames <= 0:
        parser.error("--frames must be greater than zero")
    if args.audio_trace_log <= 0:
        parser.error("--audio-trace-log must be greater than zero")
    if args.window_frames <= 0:
        parser.error("--window-frames must be greater than zero")
    if args.max_occurrences is not None and args.max_occurrences < 0:
        parser.error("--max-occurrences must be zero or greater")
    default_output_paths(args)
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        result = harvest(args)
    except (HarvestFailure, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 2

    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.rust_out.parent.mkdir(parents=True, exist_ok=True)
    args.rust_out.write_text(
        extractor.render_rust_catalog(result["programs"]) + "\n",
        encoding="utf-8",
    )
    args.report_out.parent.mkdir(parents=True, exist_ok=True)
    args.report_out.write_text(render_report(result), encoding="utf-8")

    coverage = result["coverage"]
    print(
        "modern SFX harvest: "
        f"commands={coverage['focused_commands']}/{coverage['discovered_commands']} "
        f"programs={coverage['programs']} lifted={coverage['lifted']} gaps={coverage['gaps']}"
    )
    print(f"json: {args.json_out}")
    print(f"rust: {args.rust_out}")
    print(f"report: {args.report_out}")
    if args.fail_on_gaps and coverage["gaps"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
