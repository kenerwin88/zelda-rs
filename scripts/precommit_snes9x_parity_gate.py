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
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import NamedTuple

import snes9x_route_recorder as recorder
import parity_evidence
from extract_snes9x_rom_random import extract_samples, write_script
from parity_probe import TRACE_CORE, newest_source_mtime, validate_trace_core

ROOT = Path(__file__).resolve().parents[1]
STATE_PATH = ROOT / ".git" / "precommit-snes9x-parity-state.json"
CHECKPOINT_PATH = ROOT / ".git" / "precommit-snes9x-parity-checkpoint"
RNG_CACHE_PATH = ROOT / ".git" / "precommit-snes9x-rom-random-cache"
DEFAULT_PROJECT = ROOT / "routes" / "full_run"
STATE_SCHEMA = 1
# The oracle boots from the ROM reset vector, so its WRAM reads 0x55 (and Rust's
# 0x00) until the game's init code settles. Engine-state comparison at the reset
# frame is therefore meaningless -- every field mismatches. On a cold ratchet
# (prior_frame below this, e.g. after a route-signature reset) start the live-oracle
# RNG calibration's engine-state comparison past boot instead of at frame 0.
ENGINE_STATE_COLD_START_FLOOR = 200
# Paired-resume flags the checkpoint supersedes; the binary rejects them next to
# --resume-paired because a resumed pair already carries its own SRAM/boundary.
RESUME_CONFLICTING_OPTIONS = (
    "--load-sram",
    "--resume-rust-state",
    "--resume-oracle-state",
    "--resume-oracle-sram",
)


class _PrecommitSessionPaths(NamedTuple):
    invocation_id: str
    exact: Path
    video_preflight: Path
    rng_calibration: Path


def _reserve_precommit_session_paths(
    project: Path,
    requested_frames: int,
) -> _PrecommitSessionPaths:
    """Reserve one invocation's exact session and name its diagnostic siblings.

    The exact directory is created atomically by ``mkdtemp``. Its unique suffix
    is then shared by every session produced by this gate invocation, so two
    concurrent or repeated checks of the same frame target cannot overwrite
    each other's receipts. Keep the sessions as direct ``run-*`` children so
    the existing parity probe and microscope discovery paths still find them.
    """
    root = (project / "comparisons" / "precommit").resolve()
    root.mkdir(parents=True, exist_ok=True)
    exact_prefix = f"run-{requested_frames}-exact-"
    exact = Path(tempfile.mkdtemp(prefix=exact_prefix, dir=root)).resolve()
    invocation_id = exact.name[len(exact_prefix) :]
    return _PrecommitSessionPaths(
        invocation_id=invocation_id,
        exact=exact,
        video_preflight=(
            root / f"run-{requested_frames}-video-preflight-{invocation_id}"
        ).resolve(),
        rng_calibration=(
            root / f"run-{requested_frames}-rng-calibration-{invocation_id}"
        ).resolve(),
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


def _stale_binary_source(binary: Path) -> Path | None:
    newest_mtime, newest_path = newest_source_mtime()
    return newest_path if newest_path is not None and newest_mtime > binary.stat().st_mtime else None


def _route_signature(
    project: Path,
    manifest: dict,
    take_ids: list[int],
    total_frames: int,
) -> dict[str, object]:
    generation = recorder.oracle_generations(manifest)[-1]
    identity = generation["identity"]
    takes_by_id = {int(take["id"]): take for take in manifest.get("takes", [])}
    start_boundary = int(takes_by_id[take_ids[0]]["start_boundary"])
    boundary = manifest["boundaries"][start_boundary]
    return {
        "project": str(project.relative_to(ROOT)),
        "generation_id": int(generation["id"]),
        "core_sha256": identity["core_sha256"],
        "rom_sha256": identity["rom_sha256"],
        "take_count": len(take_ids),
        "total_frames": total_frames,
        "input_sha256": _take_file_chain_sha256(
            project, takes_by_id, take_ids, "input_path"
        ),
        "recorded_rng_sha256": _take_file_chain_sha256(
            project, takes_by_id, take_ids, "rom_random_path"
        ),
        "initial_sram_sha256": recorder.sha256(
            recorder.resolve_project_path(project, boundary["sram_path"])
        ),
        "schema": STATE_SCHEMA,
    }


def _take_file_chain_sha256(
    project: Path,
    takes_by_id: dict[int, dict],
    take_ids: list[int],
    field: str,
) -> str:
    """Hash ordered route sources, including missing optional artifacts."""
    digest = hashlib.sha256()
    for take_id in take_ids:
        take = takes_by_id[take_id]
        relative_path = take.get(field)
        digest.update(f"{take_id}:{int(take['frames'])}:{relative_path or '-'}\0".encode())
        if relative_path:
            path = recorder.resolve_project_path(project, relative_path)
            digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def _resume_enabled() -> bool:
    return os.environ.get("ZELDA3_PRECOMMIT_RESUME", "1").strip().lower() not in (
        "0",
        "false",
        "no",
    )


def _video_preflight_enabled() -> bool:
    return os.environ.get("ZELDA3_PRECOMMIT_VIDEO_PREFLIGHT", "1").strip().lower() not in (
        "0",
        "false",
        "no",
    )


def _default_resume_interval(step: int) -> int:
    # A failed cold run should leave a diagnostic start no more than 1,000
    # frames behind its frontier. This costs only sparse checkpoint writes and
    # turns the usual trace rerun from minutes into seconds.
    return min(max(1, step), 1_000)


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


def _co_locate_cold_replay_sources(command: list[str], input_path: Path) -> list[str]:
    """Put the cold SRAM beside the generated input/RNG provenance bundle."""
    if "--load-sram" not in command:
        return command
    command = command.copy()
    load_index = command.index("--load-sram") + 1
    source_sram = Path(command[load_index])
    bundled_sram = input_path.parent / "initial.srm"
    if source_sram.resolve() != bundled_sram.resolve():
        shutil.copyfile(source_sram, bundled_sram)
    command[load_index] = str(bundled_sram)
    return command


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
    ignore_audio: bool = False,
    ignore_video: bool = False,
    authoritative: bool = False,
    live_oracle_rng: bool = False,
    engine_state_from_frame: int | None = None,
    expected_core_sha256: str | None = None,
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
    command = _co_locate_cold_replay_sources(command, input_path)
    # The authoritative ratchet needs the earliest failing boundary and its
    # finalized session, not every later symptom. The route recorder defaults
    # to --scan-all for broad reports, so explicitly narrow the pre-commit
    # command before adding checkpoint options.
    command = [item for item in command if item != "--scan-all"]
    if expected_core_sha256 is not None:
        hash_index = command.index("--expected-core-sha256") + 1
        command[hash_index] = expected_core_sha256
    if ignore_audio and "--ignore-audio" not in command:
        command.append("--ignore-audio")
    if ignore_video and "--ignore-video" not in command:
        command.append("--ignore-video")
    if live_oracle_rng:
        if rom_random_path is not None:
            raise ValueError("live oracle RNG cannot consume a recorded RNG script")
        command.append("--live-oracle-rng")
    if engine_state_from_frame is not None:
        if not live_oracle_rng:
            raise ValueError("engine-state comparison requires live oracle RNG")
        command.extend(
            ["--compare-engine-state-from-frame", str(engine_state_from_frame)]
        )
    # Paired checkpoints are an optimization aid, not parity authority: they
    # intentionally do not yet serialize every presentation/scheduler
    # transient, and writing the legacy save can perturb fields not covered by
    # its restoration trailer. A ratchet-advancing exact A/V pass must both
    # start cold and avoid checkpoint writes while it is running.
    return _apply_resume_options(
        command,
        resume_dir=None if authoritative or live_oracle_rng else resume_dir,
        rolling=None if authoritative or live_oracle_rng else rolling,
    )


def _write_live_oracle_rng_script(trace_path: Path, output_path: Path) -> int:
    """Materialize the cartridge-only RNG script from a live trace receipt."""
    if not trace_path.is_file():
        raise FileNotFoundError(f"live oracle RNG trace was not produced: {trace_path}")
    with trace_path.open(encoding="utf-8") as trace:
        samples = extract_samples(trace)
    with output_path.open("w", encoding="utf-8") as output:
        write_script(samples, output)
    return len(samples)


def _rng_cache_key(
    signature: dict[str, object], requested_frames: int, trace_core_sha256: str
) -> str:
    identity = {
        "schema": 1,
        "requested_frames": requested_frames,
        "input_sha256": signature["input_sha256"],
        "initial_sram_sha256": signature["initial_sram_sha256"],
        "rom_sha256": signature["rom_sha256"],
        "trace_core_sha256": trace_core_sha256,
    }
    return hashlib.sha256(
        json.dumps(identity, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def _restore_rng_cache(
    signature: dict[str, object],
    requested_frames: int,
    trace_core_sha256: str,
    output_path: Path,
) -> int | None:
    key = _rng_cache_key(signature, requested_frames, trace_core_sha256)
    cache_dir = RNG_CACHE_PATH / key
    metadata = _load_json(cache_dir / "metadata.json")
    script = cache_dir / "rom-random.txt"
    if not isinstance(metadata, dict) or not script.is_file():
        return None
    if metadata.get("script_sha256") != recorder.sha256(script):
        return None
    sample_count = metadata.get("sample_count")
    if not isinstance(sample_count, int) or sample_count < 0:
        return None
    shutil.copyfile(script, output_path)
    return sample_count


def _store_rng_cache(
    signature: dict[str, object],
    requested_frames: int,
    trace_core_sha256: str,
    script_path: Path,
    sample_count: int,
) -> None:
    key = _rng_cache_key(signature, requested_frames, trace_core_sha256)
    cache_dir = RNG_CACHE_PATH / key
    cache_dir.mkdir(parents=True, exist_ok=True)
    cached_script = cache_dir / "rom-random.txt"
    shutil.copyfile(script_path, cached_script)
    _write_json(
        cache_dir / "metadata.json",
        {
            "schema": 1,
            "requested_frames": requested_frames,
            "sample_count": sample_count,
            "script_sha256": recorder.sha256(cached_script),
        },
    )


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
    if stale_source := _stale_binary_source(binary):
        print(
            f"pre-commit gate: {binary} is older than {stale_source}; "
            "run `cargo build --profile parity -p zelda3-bin` first",
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
    authority_core = required_core
    authority_core_sha256: str | None = None
    if _video_preflight_enabled():
        authority_core = _abs_path(
            os.environ.get("ZELDA3_PRECOMMIT_TRACE_CORE", str(TRACE_CORE))
        )
        authority_core_sha256 = validate_trace_core(authority_core)

    resume_enabled = _resume_enabled()
    binary_identity = _binary_identity(binary) if resume_enabled else None
    if binary_identity is not None:
        binary_identity["authority_core_sha256"] = (
            authority_core_sha256 or recorder.sha256(authority_core)
        )

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

        sessions = _reserve_precommit_session_paths(project, requested)
        session_dir = sessions.exact

        rom_random_path = temp_dir / "rom-random.txt"
        rom_random_count = 0

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
            default_interval = _default_resume_interval(step)
            interval = (
                env_int("ZELDA3_PRECOMMIT_RESUME_INTERVAL", default_interval)
                or default_interval
            )
            rolling = (interval, CHECKPOINT_PATH)
            if resume_dir is not None:
                print(f"pre-commit: resuming from paired checkpoint {resume_dir}")

        if _video_preflight_enabled():
            trace_core = authority_core
            trace_core_sha256 = authority_core_sha256
            assert trace_core_sha256 is not None
            # Continuous Zelda-level timing receipts are emitted only by the
            # maintained trace build. Once the runtime consumes those receipts,
            # replaying their RNG stream against a stock core silently runs a
            # different scheduler and can shift source calls by a host frame.
            # Use one core identity for calibration, checkpointable preflight,
            # and cold exact certification so every tier observes the same
            # authoritative host intervals.
            cached_rng_count = _restore_rng_cache(
                signature,
                requested,
                trace_core_sha256,
                rom_random_path,
            )
            using_live_rng = cached_rng_count is None
            if cached_rng_count is not None:
                rom_random_count = cached_rng_count
            if using_live_rng:
                rng_session_dir = sessions.rng_calibration
                rng_command = _build_check_command(
                    binary=binary,
                    core=trace_core,
                    rom=rom,
                    project=project,
                    session_dir=rng_session_dir,
                    take_ids=take_ids,
                    start_boundary=start_boundary,
                    requested_frames=requested,
                    input_path=input_path,
                    rom_random_path=None,
                    ignore_audio=True,
                    ignore_video=True,
                    live_oracle_rng=True,
                    engine_state_from_frame=max(
                        prior_frame, ENGINE_STATE_COLD_START_FLOOR
                    ),
                    expected_core_sha256=trace_core_sha256,
                )
                print(
                    "pre-commit: RNG cache miss; running renderless live-oracle "
                    "calibration"
                )
                rng_started = time.monotonic()
                rng_process = subprocess.run(
                    [str(item) for item in rng_command],
                    cwd=ROOT,
                    text=True,
                    capture_output=True,
                )
                print(
                    "pre-commit: RNG calibration elapsed "
                    f"{time.monotonic() - rng_started:.1f}s"
                )
                if rng_process.stdout:
                    print(rng_process.stdout)
                if rng_process.stderr:
                    print(rng_process.stderr, file=sys.stderr)
                rng_result_path = rng_session_dir / "result.json"
                if not rng_result_path.exists():
                    print(
                        "pre-commit gate: RNG calibration did not produce a session result",
                        file=sys.stderr,
                    )
                    return 1
                rng_result = json.loads(rng_result_path.read_text(encoding="utf-8"))
                if not (
                    rng_process.returncode == 0
                    and rng_result.get("status") == "passed"
                ):
                    print(
                        "pre-commit gate: live-oracle RNG calibration failed",
                        file=sys.stderr,
                    )
                    print(
                        f"pre-commit: replay artifacts: {rng_session_dir}",
                        file=sys.stderr,
                    )
                    return 1
                try:
                    rom_random_count = _write_live_oracle_rng_script(
                        rng_session_dir / "oracle-rom-random.jsonl",
                        rom_random_path,
                    )
                    _store_rng_cache(
                        signature,
                        requested,
                        trace_core_sha256,
                        rom_random_path,
                        rom_random_count,
                    )
                except (OSError, ValueError) as error:
                    print(
                        f"pre-commit gate: could not materialize live oracle RNG: {error}",
                        file=sys.stderr,
                    )
                    return 1
                print(
                    "pre-commit: RNG calibration passed "
                    f"({rom_random_count} cartridge sample(s))"
                )
            else:
                print(
                    "pre-commit: RNG cache hit "
                    f"({rom_random_count} cartridge sample(s))"
                )
            preflight_session_dir = sessions.video_preflight
            preflight_command = _build_check_command(
                binary=binary,
                core=authority_core,
                rom=rom,
                project=project,
                session_dir=preflight_session_dir,
                take_ids=take_ids,
                start_boundary=start_boundary,
                requested_frames=requested,
                input_path=input_path,
                rom_random_path=rom_random_path if rom_random_count else None,
                resume_dir=resume_dir,
                rolling=(interval, CHECKPOINT_PATH) if resume_enabled else None,
                ignore_audio=True,
                expected_core_sha256=authority_core_sha256,
            )
            print(
                "pre-commit: running checkpointable timing-authority video preflight "
                "before exact A/V certification"
            )
            preflight_started = time.monotonic()
            preflight_process = subprocess.run(
                [str(item) for item in preflight_command],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
            print(
                "pre-commit: video preflight elapsed "
                f"{time.monotonic() - preflight_started:.1f}s"
            )
            if preflight_process.stdout:
                print(preflight_process.stdout)
            if preflight_process.stderr:
                print(preflight_process.stderr, file=sys.stderr)
            preflight_result_path = preflight_session_dir / "result.json"
            if not preflight_result_path.exists():
                print(
                    "pre-commit gate: video preflight did not produce a session result",
                    file=sys.stderr,
                )
                return 1
            preflight_result = json.loads(
                preflight_result_path.read_text(encoding="utf-8")
            )
            preflight_video = preflight_result.get("video", {})
            if not (
                preflight_process.returncode == 0
                and preflight_result.get("status") == "passed"
                and preflight_video.get("matched") is True
            ):
                first_video = (preflight_video or {}).get("first_mismatch")
                print(
                    "pre-commit: video preflight failed; skipping expensive exact audio pass",
                    file=sys.stderr,
                )
                if first_video:
                    print(
                        f"pre-commit: first video mismatch: {first_video}",
                        file=sys.stderr,
                    )
                print(
                    f"pre-commit: replay artifacts: {preflight_session_dir}",
                    file=sys.stderr,
                )
                return 1
            print(
                "pre-commit: RNG-verified video preflight passed "
                f"({rom_random_count} cartridge sample(s)); "
                "running cold timing-authority exact A/V certification"
            )
        else:
            rom_random_count = recorder.write_continuous_rom_random(
                project,
                take_ids,
                rom_random_path,
                takes_by_id=takes_by_id,
            )
            if rom_random_count == 0 and rom_random_path.exists():
                rom_random_path.unlink()

        command = _build_check_command(
            binary=binary,
            core=authority_core,
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
            authoritative=True,
            expected_core_sha256=authority_core_sha256,
        )

        exact_started = time.monotonic()
        process = subprocess.run(
            [str(item) for item in command],
            cwd=ROOT,
            text=True,
            capture_output=True,
        )
        print(
            "pre-commit: cold exact A/V elapsed "
            f"{time.monotonic() - exact_started:.1f}s"
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
    pass_receipt = parity_evidence.record_cold_pass(
        session=session_dir,
        route_signature=signature,
        binary=binary,
    )
    _write_json(STATE_PATH, state)
    print(f"pre-commit: cold proof receipt {pass_receipt}")

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
