#!/usr/bin/env python3
"""Run a self-ratcheting Snes9x parity gate for pre-commit.

The script keeps local state under .git/precommit-snes9x-parity-state.json. Each
run extends a committed frame frontier (or re-checks the existing frontier on
route/hash changes) so regressions are blocked before they land.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import snes9x_route_recorder as recorder

ROOT = Path(__file__).resolve().parents[1]
STATE_PATH = ROOT / ".git" / "precommit-snes9x-parity-state.json"
CHECKPOINT_PATH = ROOT / ".git" / "precommit-snes9x-parity-checkpoint"
DEFAULT_PROJECT = ROOT / "routes" / "crystal4_II"
STATE_SCHEMA = 1
# Paired-resume flags the checkpoint supersedes; the binary rejects them next to
# --resume-paired because a resumed pair already carries its own SRAM/boundary.
RESUME_CONFLICTING_OPTIONS = (
    "--load-sram",
    "--resume-rust-state",
    "--resume-oracle-state",
    "--resume-oracle-sram",
)


def env_int(name: str, default: int | None = None) -> int | None:
    if name not in os.environ:
        return default
    value = os.environ[name].strip()
    if not value:
        return default
    try:
        return int(value)
    except ValueError as error:
        print(
            f"pre-commit gate: invalid {name}={value!r}; expected an integer",
            file=sys.stderr,
        )
        raise SystemExit(2) from error


def _abs_path(value: str) -> Path:
    path = Path(os.path.expanduser(value))
    return path if path.is_absolute() else ROOT / path


def _load_json(path: Path, default: object | None = None) -> object | None:
    if not path.exists():
        return default
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        print(
            f"pre-commit gate: ignoring invalid state file {path}",
            file=sys.stderr,
        )
        return default


def _write_json(path: Path, data: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _route_signature(
    project: Path,
    manifest: dict,
    take_ids: list[int],
    total_frames: int,
) -> dict[str, object]:
    generation = recorder.oracle_generations(manifest)[-1]
    identity = generation["identity"]
    return {
        "project": str(project.relative_to(ROOT)),
        "generation_id": int(generation["id"]),
        "core_sha256": identity["core_sha256"],
        "rom_sha256": identity["rom_sha256"],
        "take_count": len(take_ids),
        "total_frames": total_frames,
        "schema": STATE_SCHEMA,
    }


def _resume_enabled() -> bool:
    return os.environ.get("ZELDA3_PRECOMMIT_RESUME", "").strip() not in ("", "0")


def _binary_identity(binary: Path) -> dict[str, object]:
    digest = hashlib.sha256()
    with binary.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    stat = binary.stat()
    return {"path": str(binary), "size": stat.st_size, "sha256": digest.hexdigest()}


def _latest_checkpoint(root: Path) -> tuple[int, Path] | None:
    latest = _load_json(root / "latest.json")
    if not isinstance(latest, dict) or latest.get("schema") != 1:
        return None
    frame = latest.get("frame")
    name = latest.get("checkpoint")
    if not isinstance(frame, int) or not isinstance(name, str):
        return None
    checkpoint = root / name
    return (frame, checkpoint) if (checkpoint / "manifest.json").is_file() else None


def _resume_checkpoint(
    state: dict,
    *,
    signature: dict,
    binary_identity: dict[str, object],
    requested: int,
) -> Path | None:
    """The recorded paired checkpoint this run may resume from, if any.

    The checkpoint's Rust half is only valid for the exact binary that wrote it,
    so any rebuild (or route change) falls back to a full replay from frame 0.
    The recorded generation is resumed by path rather than through the rolling
    `latest.json`, so a later failing run cannot move the gate's resume point.
    """
    frame = state.get("checkpoint_frame")
    directory = state.get("checkpoint_dir")
    if not isinstance(frame, int) or not isinstance(directory, str):
        return None
    if frame <= 0 or frame >= requested:
        return None
    if state.get("route_signature") != signature:
        return None
    if state.get("checkpoint_binary") != binary_identity:
        print(
            "pre-commit gate: parity binary changed since the resume checkpoint; replaying from frame 0",
            file=sys.stderr,
        )
        return None
    checkpoint = Path(directory)
    if not (checkpoint / "manifest.json").is_file():
        print(
            f"pre-commit gate: resume checkpoint {checkpoint} is gone; replaying from frame 0",
            file=sys.stderr,
        )
        return None
    return checkpoint


def _apply_resume_options(
    command: list[str],
    *,
    resume_dir: Path | None,
    rolling: tuple[int, Path] | None,
) -> list[str]:
    if resume_dir is None and rolling is None:
        return command
    result: list[str] = []
    index = 0
    while index < len(command):
        token = command[index]
        if resume_dir is not None and token in RESUME_CONFLICTING_OPTIONS:
            index += 2
            continue
        result.append(token)
        index += 1
    if resume_dir is not None:
        result.extend(["--resume-paired", str(resume_dir)])
    if rolling is not None:
        interval, root = rolling
        # Rolling rather than --save-paired-resume-at: a fixed frame can land
        # inside an unserialized ROM-call continuation, which aborts the run,
        # while the rolling saver waits for the next quiescent boundary.
        result.extend(["--save-rolling-paired-resume", str(interval), str(root)])
    return result


def _resolve_project(path: str) -> Path:
    project = Path(path)
    if not project.is_absolute():
        project = ROOT / project
    project = project.resolve()
    if not project.exists():
        raise SystemExit(f"pre-commit gate: parity project does not exist: {project}")
    return project


def _build_check_command(
    *,
    binary: Path,
    core: Path,
    rom: Path,
    project: Path,
    session_dir: Path,
    take_ids: list[int],
    start_boundary: int,
    requested_frames: int,
    input_path: Path,
    rom_random_path: Path | None,
    resume_dir: Path | None = None,
    rolling: tuple[int, Path] | None = None,
) -> list[str]:
    manifest = recorder.load_manifest(project)
    identity = recorder.oracle_generations(manifest)[-1]["identity"]
    takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    command = recorder.compare_input_command(
        binary=binary,
        core=core,
        rom=rom,
        project=project,
        boundary_id=start_boundary,
        frames=requested_frames,
        input_path=input_path,
        rom_random_path=rom_random_path,
        session_dir=session_dir,
        identity=identity,
    )
    return _apply_resume_options(command, resume_dir=resume_dir, rolling=rolling)


def _extract_identity(manifest: dict) -> tuple[Path, dict]:
    generation = recorder.oracle_generations(manifest)[-1]
    identity = generation["identity"]
    requested_core = _abs_path(os.environ.get("SNES9X_LIBRETRO_CORE", str(recorder.DEFAULT_CORE)))
    required_core = recorder.required_core(requested_core, identity)
    return required_core, identity


def run_snes9x_gate() -> int:
    project = _resolve_project(os.environ.get("ZELDA3_PRECOMMIT_PROJECT", str(DEFAULT_PROJECT)))
    binary = _abs_path(os.environ.get("ZELDA3_PRECOMMIT_BINARY", str(ROOT / "target" / "parity" / "zelda3")))
    if not binary.is_file():
        print(
            f"pre-commit gate: parity binary missing ({binary}); run the parity build first",
            file=sys.stderr,
        )
        return 1
    rom = _abs_path(os.environ.get("ZELDA3_PRECOMMIT_ROM", os.environ.get("ZELDA3_ROM", str(recorder.default_rom_path()))))
    if not rom.is_file():
        print(f"pre-commit gate: ROM does not exist: {rom}", file=sys.stderr)
        return 1

    manifest = recorder.load_manifest(project)
    take_ids = recorder.continuous_take_ids(project)
    if not take_ids:
        print(
            f"pre-commit gate: no takes in continuous route project {project}",
            file=sys.stderr,
        )
        return 1

    takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    total_frames = sum(int(takes_by_id[take_id]["frames"]) for take_id in take_ids)
    if total_frames <= 0:
        print(
            f"pre-commit gate: route project {project} has no recorded frame coverage",
            file=sys.stderr,
        )
        return 1

    step = env_int("ZELDA3_PRECOMMIT_STEP", 10000)
    if step is None:
        step = 10000
    if step < 0:
        print("pre-commit gate: ZELDA3_PRECOMMIT_STEP must be >= 0", file=sys.stderr)
        return 1
    initial_check = env_int("ZELDA3_PRECOMMIT_INITIAL_CHECK", max(1, step))
    if initial_check is None:
        initial_check = max(1, step)
    if initial_check <= 0:
        print("pre-commit gate: ZELDA3_PRECOMMIT_INITIAL_CHECK must be > 0", file=sys.stderr)
        return 1
    max_frames_env = env_int("ZELDA3_PRECOMMIT_MAX_FRAMES", total_frames)
    if max_frames_env is None:
        max_frames_env = total_frames
    max_frames = max(1, min(total_frames, max_frames_env))

    state = _load_json(STATE_PATH, {}) or {}
    signature = _route_signature(project, manifest, take_ids, total_frames)
    prior_signature = state.get("route_signature")
    prior_frame = state.get("last_checked_frame", 0)
    if not isinstance(prior_frame, int) or prior_frame < 0:
        prior_frame = 0

    if prior_signature != signature:
        print("pre-commit gate: route/signature changed, resetting ratchet", file=sys.stderr)
        prior_frame = 0

    explicit = env_int("ZELDA3_PRECOMMIT_TARGET_FRAME", 0)
    if explicit is not None and explicit > 0:
        requested = min(explicit, max_frames)
    elif prior_frame <= 0:
        requested = min(initial_check, max_frames)
    else:
        requested = min(prior_frame + step, max_frames)

    if requested <= 0:
        requested = min(1, max_frames)
    if requested > prior_frame and prior_frame >= max_frames:
        requested = prior_frame

    print(
        f"pre-commit: Snes9x parity gate target={requested} frame(s) "
        f"(last_checked={prior_frame}, step={step}, max={max_frames})",
    )

    required_core, _ = _extract_identity(manifest)
    if not required_core.exists():
        print(
            f"pre-commit gate: required Snes9x core not found: {required_core}",
            file=sys.stderr,
        )
        return 1

    resume_enabled = _resume_enabled()
    binary_identity = _binary_identity(binary) if resume_enabled else None

    session_dir = (project / "comparisons" / "precommit" / f"run-{requested}").resolve()
    with tempfile.TemporaryDirectory(prefix="zelda3-precommit-") as temp_dir:
        temp_dir = Path(temp_dir)
        input_path = temp_dir / "input.txt"
        input_frames = recorder.write_continuous_input(
            project,
            take_ids,
            input_path,
            takes_by_id=takes_by_id,
        )
        if input_frames < requested:
            requested = input_frames

        rom_random_path = temp_dir / "rom-random.txt"
        rom_random_count = recorder.write_continuous_rom_random(
            project,
            take_ids,
            rom_random_path,
            takes_by_id=takes_by_id,
        )
        if rom_random_count == 0 and rom_random_path.exists():
            rom_random_path.unlink()

        first_take = takes_by_id[take_ids[0]]
        start_boundary = int(first_take["start_boundary"])

        resume_dir = None
        rolling = None
        if resume_enabled:
            resume_dir = _resume_checkpoint(
                state,
                signature=signature,
                binary_identity=binary_identity or {},
                requested=requested,
            )
            interval = env_int("ZELDA3_PRECOMMIT_RESUME_INTERVAL", max(1, step)) or max(1, step)
            rolling = (interval, CHECKPOINT_PATH)
            if resume_dir is not None:
                print(f"pre-commit: resuming from paired checkpoint {resume_dir}")

        command = _build_check_command(
            binary=binary,
            core=required_core,
            rom=rom,
            project=project,
            session_dir=session_dir,
            take_ids=take_ids,
            start_boundary=start_boundary,
            requested_frames=requested,
            input_path=input_path,
            rom_random_path=rom_random_path if rom_random_count else None,
            resume_dir=resume_dir,
            rolling=rolling,
        )

        process = subprocess.run(
            [str(item) for item in command],
            cwd=ROOT,
            text=True,
            capture_output=True,
        )
    if process.stdout:
        print(process.stdout)
    if process.stderr:
        print(process.stderr, file=sys.stderr)

    result_path = session_dir / "result.json"
    if not result_path.exists():
        print("pre-commit gate: compare command did not produce a session result", file=sys.stderr)
        return 1
    result = json.loads(result_path.read_text(encoding="utf-8"))

    video = result.get("video", {})
    audio = result.get("audio", {})
    matched = (
        process.returncode == 0
        and result.get("status") == "passed"
        and bool(result.get("parity_eligible"))
        and video.get("matched") is True
        and audio.get("matched") is not False
    )

    if not matched:
        first_video = (video or {}).get("first_mismatch")
        first_audio = audio.get("first_mismatch") if isinstance(audio, dict) else None
        print(
            "pre-commit: parity failed at or before requested frame "
            f"{requested} on {project}; keeping ratchet at {prior_frame}",
            file=sys.stderr,
        )
        if first_video:
            print(f"pre-commit: first video mismatch: {first_video}", file=sys.stderr)
        if first_audio:
            print(f"pre-commit: first audio mismatch: {first_audio}", file=sys.stderr)
        print(f"pre-commit: replay artifacts: {session_dir}", file=sys.stderr)
        return 1

    state.update(
        {
            "schema": STATE_SCHEMA,
            "route_signature": signature,
            "last_checked_frame": max(prior_frame, requested),
            "last_checked_total_frames": max_frames,
            "binary": str(binary),
            "core": str(required_core),
            "rom": str(rom),
            "route_project": str(project),
        }
    )
    if resume_enabled:
        # Only a passing run may advance the resume point: the binary refuses to
        # write a checkpoint once a mismatch is seen, and a failing run returns
        # before this state is persisted.
        latest = _latest_checkpoint(CHECKPOINT_PATH)
        if latest is not None:
            checkpoint_frame, checkpoint_dir = latest
            state.update(
                {
                    "checkpoint_frame": checkpoint_frame,
                    "checkpoint_dir": str(checkpoint_dir),
                    "checkpoint_binary": binary_identity,
                }
            )
            print(f"pre-commit: paired resume checkpoint at frame {checkpoint_frame}")
    _write_json(STATE_PATH, state)

    checked = state["last_checked_frame"]
    if checked >= max_frames:
        print(f"pre-commit: parity passed at {checked} frame(s) (route cap reached)")
    else:
        print(
            f"pre-commit: parity passed to {checked} frame(s); "
            "commit will ratchet higher on next run",
        )
    return 0


def main() -> int:
    return run_snes9x_gate()


if __name__ == "__main__":
    raise SystemExit(main())
