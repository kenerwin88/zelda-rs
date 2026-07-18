#!/usr/bin/env python3
"""Run verified video/audio parity over a Snes9x-native segmented route matrix."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
CAPTURE_KIND = "zelda3_snes9x_native_segment_matrix_capture_v1"
RESULT_KIND = "zelda3_snes9x_segmented_output_parity_v1"


def artifact_path(value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else REPO_ROOT / path


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_capture_manifest(manifest: dict[str, Any]) -> None:
    if manifest.get("kind") != CAPTURE_KIND:
        raise ValueError(f"capture kind must be {CAPTURE_KIND}")
    if manifest.get("continuous_playthrough") is not False:
        raise ValueError("capture must explicitly declare continuous_playthrough=false")
    summary = manifest.get("summary")
    if not isinstance(summary, dict):
        raise ValueError("capture summary is missing")
    if summary.get("eligible_for_segmented_output_parity") is not True:
        reason = summary.get("stopped_reason", "route milestone validation failed")
        raise ValueError(f"capture is not eligible for segmented output parity: {reason}")
    if summary.get("created_native_boundary_states") != 12:
        raise ValueError("capture must contain exactly 12 Snes9x-native boundary states")

    segments = manifest.get("segments")
    if not isinstance(segments, list) or len(segments) != 13:
        raise ValueError("capture must contain exactly 13 segments")
    total_frames = 0
    for index, segment in enumerate(segments, 1):
        if segment.get("segment") != index:
            raise ValueError(f"segment {index} has the wrong index")
        if segment.get("eligible_for_output_parity") is not True:
            raise ValueError(f"segment {index} is not eligible for output parity")
        frames = segment.get("frames")
        if not isinstance(frames, int) or frames <= 0:
            raise ValueError(f"segment {index} has an invalid frame count")
        total_frames += frames
        starts = segment.get("paired_starts", {})
        for lane, key in (("Rust", "rust"), ("Snes9x", "oracle")):
            entry = starts.get(key, {})
            value = entry.get("path")
            path = artifact_path(value) if isinstance(value, str) else None
            if path is None or not path.is_file():
                raise ValueError(f"segment {index} {lane} start state is missing: {value}")
            expected_sha256 = entry.get("sha256")
            actual_sha256 = sha256_file(path)
            if expected_sha256 != actual_sha256:
                raise ValueError(
                    f"segment {index} {lane} start state hash mismatch: "
                    f"expected {expected_sha256}, got {actual_sha256}"
                )
        oracle = starts.get("oracle", {})
        if oracle.get("converted_from_rust") is not False:
            raise ValueError(f"segment {index} lacks no-Rust-conversion provenance")
    if summary.get("aggregate_input_frames") != total_frames:
        raise ValueError(
            "capture aggregate frame count does not equal the sum of its segments"
        )
    for label in ("core", "rom"):
        entry = manifest.get(label)
        if not isinstance(entry, dict) or not entry.get("path") or not entry.get("sha256"):
            raise ValueError(f"capture {label} provenance is incomplete")


def comparison_commands(
    manifest: dict[str, Any], binary: Path, output_dir: Path
) -> list[list[str]]:
    validate_capture_manifest(manifest)
    core = manifest["core"]
    rom = manifest["rom"]
    commands: list[list[str]] = []
    for segment in manifest["segments"]:
        index = int(segment["segment"])
        starts = segment["paired_starts"]
        commands.append(
            [
                str(binary),
                "--compare-snes9x-oracle",
                str(artifact_path(core["path"])),
                str(artifact_path(rom["path"])),
                str(segment["frames"]),
                "--resume-rust-state",
                str(artifact_path(starts["rust"]["path"])),
                "--resume-oracle-state",
                str(artifact_path(starts["oracle"]["path"])),
                "--expected-core-sha256",
                str(core["sha256"]),
                "--expected-rom-sha256",
                str(rom["sha256"]),
                "--audio-comparison",
                "exact",
                "--rust-audio-backend",
                "modern",
                "--rust-audio-sequencer",
                "native",
                "--session-dir",
                str(output_dir / f"segment-{index:02}"),
                "--scan-all",
            ]
        )
    return commands


def run_matrix(capture_path: Path, binary: Path, output_dir: Path) -> int:
    manifest = json.loads(capture_path.read_text(encoding="utf-8"))
    validate_capture_manifest(manifest)
    output_dir.mkdir(parents=True, exist_ok=True)
    commands = comparison_commands(manifest, binary, output_dir)
    environment = os.environ.copy()
    asset_pack = REPO_ROOT / "zelda3_assets.dat"
    if asset_pack.is_file():
        environment.setdefault("ZELDA3_ASSET_PACK", str(asset_pack))

    receipts: list[dict[str, Any]] = []
    verified_frames = 0
    for segment, command in zip(manifest["segments"], commands, strict=True):
        index = int(segment["segment"])
        print(
            f"segmented parity {index:02}/13 frames={segment['frames']} "
            f"cumulative={segment['cumulative_frames']}",
            flush=True,
        )
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=environment,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        segment_dir = output_dir / f"segment-{index:02}"
        segment_dir.mkdir(parents=True, exist_ok=True)
        (segment_dir / "command.log").write_text(completed.stdout, encoding="utf-8")
        result_path = segment_dir / "result.json"
        result = (
            json.loads(result_path.read_text(encoding="utf-8"))
            if result_path.is_file()
            else None
        )
        passed = completed.returncode == 0 and result is not None and result.get("status") == "passed"
        if passed:
            verified_frames += int(segment["frames"])
        receipts.append(
            {
                "segment": index,
                "frames": segment["frames"],
                "returncode": completed.returncode,
                "passed": passed,
                "session_dir": str(segment_dir),
                "result": result,
            }
        )
        print(f"  {'passed' if passed else 'FAILED'}", flush=True)

    all_passed = all(receipt["passed"] for receipt in receipts)
    aggregate = {
        "schema": 1,
        "kind": RESULT_KIND,
        "coverage_label": "segmented coverage",
        "continuous_playthrough": False,
        "capture_manifest": str(capture_path),
        "core": manifest["core"],
        "rom": manifest["rom"],
        "comparison": {
            "video": "exact completed-frame RGBA",
            "audio": "exact continuous PCM",
            "rust_audio_backend": "modern",
            "rust_audio_sequencer": "native",
            "dynamic_alignment": False,
        },
        "segments": receipts,
        "summary": {
            "segment_count": len(receipts),
            "aggregate_input_frames": manifest["summary"]["aggregate_input_frames"],
            "verified_video_audio_parity_frames": verified_frames,
            "all_segments_passed": all_passed,
            "final_segment_reached_ending": (
                all_passed
                and manifest["segments"][-1]["milestone"]["actual"].get("ending") == "1"
            ),
        },
    }
    result_path = output_dir / "matrix-result.json"
    result_path.write_text(json.dumps(aggregate, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"segmented parity result: {result_path}")
    return 0 if all_passed else 1


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture_manifest", type=Path)
    parser.add_argument("--binary", type=Path, default=REPO_ROOT / "target/release/zelda3")
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=REPO_ROOT / "target/parity/snes9x-segment-matrix-results",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if not args.binary.is_file():
            raise ValueError(f"zelda3 binary does not exist: {args.binary}")
        return run_matrix(args.capture_manifest, args.binary, args.output_dir)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(error, file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
