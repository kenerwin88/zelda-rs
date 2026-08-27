#!/usr/bin/env python3
"""Replay one Snes9x parity divergence window against the pinned oracle.

The pre-commit gate leaves a full set of route inputs behind in
routes/<project>/comparisons/precommit/run-<target>/ (input.txt, rom-random.txt,
initial.srm, replay.sh). This tool reuses those inputs to re-run a short window
around a suspect frame. The default path starts cold because checkpoint resumes
can mask timing divergences. Pass --use-checkpoint for a faster diagnostic-only
resume from a paired Rust+oracle checkpoint saved just before the window.

    python3 scripts/parity_probe.py --around 17213 --capture

For the fastest edit/diagnose loop, point directly at the newest failure. This
derives the frame and first bad pixel, compares only the frontier window, and
labels post-frame-only differences that cannot explain the completed scanout:

    python3 scripts/parity_probe.py --from-failure --use-checkpoint

Focused probes start cold by default. ``--frontier`` instead maintains a
project-scoped rolling paired checkpoint and deliberately reuses it across Rust
builds for the fast diagnostic loop. Every such result is labeled diagnostic
only; the cold exact-A/V gate remains the sole parity proof and commit ratchet.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import NamedTuple

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PROJECT = "routes/full_run"
DEFAULT_BINARY = ROOT / "target" / "parity" / "zelda3"
PROBE_ROOT = ROOT / "target" / "parity-probes"
CHECKPOINT_ROOT = PROBE_ROOT / "checkpoints"
AUTOMATIC_ROLLING_CHECKPOINT_INTERVAL = 600
TRACE_CORE = ROOT / "external" / "snes9x-libretro" / "local" / "snes9x_libretro_trace.dylib"
TRACE_PATCHES = (
    ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-trace.patch",
    ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-trace-obj-cache.patch",
    ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-spc-opcode-ledger.patch",
    ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-dsp-phase-ledger.patch",
    ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-dma-ledger.patch",
    ROOT
    / "external"
    / "snes9x-libretro"
    / "patches"
    / "zelda3-trace-presented-cgram.patch",
    ROOT
    / "external"
    / "snes9x-libretro"
    / "patches"
    / "zelda3-trace-presented-hud.patch",
    ROOT
    / "external"
    / "snes9x-libretro"
    / "patches"
    / "zelda3-trace-presented-bg-tilemaps.patch",
    ROOT
    / "external"
    / "snes9x-libretro"
    / "patches"
    / "zelda3-trace-presented-window-mask.patch",
)
ORACLE_LOCK = ROOT / "external" / "snes9x-libretro" / "oracle-lock.json"
IDENTITY_NAME = "probe-identity.json"
INSTRUMENTED_SYMBOL = b"zelda3_snes9x_debug_ppu_value"
SOURCE_DIRS = ("crates", "zelda3-bin")

# `display_register_values` in zelda3-bin/src/snes9x_compare.rs flattens the
# register domain in this order.
REGISTER_LABELS = (
    ["mode", "brightness", "forced_blank"]
    + [f"fixed_color[{i}]" for i in range(3)]
    + [f"display_control[{i}]" for i in range(6)]
    + [f"bg_scroll[{i}]" for i in range(8)]
)
# Low OAM table is 128 slots x 4 bytes; the 32-byte high table follows it.
OAM_LOW_VALUES = 512
MAX_DETAIL_LINES = 4
MAX_DETAIL_RECEIPTS = 3
POST_FRAME_ONLY_DISPLAY_DOMAINS = frozenset({"live_oam"})
FRONTIER_PROVENANCE_FIELDS = (
    "main_module",
    "submodule",
    "subsubmodule",
    "frame_counter",
    "nmi_update_latch",
    "nmi_bg_vram_load_mode",
    "nmi_subroutine_index",
    "pending_tilemap_destination_page",
    "pending_tilemap_source_offset",
    "pending_tilemap_source_hash",
    "incremental_vram_upload_counter",
    "palette_filter_countdown",
    "screen_brightness",
    "link_dma_countdown",
    "link_dma_source_offset",
    "link_dma_tile_offset",
    "ppu_oam_dma_shadow_hash",
)
FIRST_MISMATCH_RE = re.compile(r"first_mismatch=\((\d+),\s*(\d+)\)")
FRAME_RANGE_RE = re.compile(r"(\d+)-(\d+)")


class FailureFocus(NamedTuple):
    directory: Path
    frame: int
    pixel: tuple[int, int] | None


def parse_frame_range(value: str) -> tuple[int, int]:
    match = FRAME_RANGE_RE.fullmatch(value)
    if match is None:
        raise argparse.ArgumentTypeError("expected START-END")
    start, end = map(int, match.groups())
    if start > end:
        raise argparse.ArgumentTypeError("START must not exceed END")
    return start, end


def resolve_failure_dir(value: Path) -> Path:
    if str(value) != "latest":
        directory = value if value.is_absolute() else ROOT / value
        if directory.name == "diff.json":
            directory = directory.parent
        if not (directory / "diff.json").is_file():
            raise SystemExit(f"parity-probe: {directory} has no diff.json")
        return directory.resolve()
    root = ROOT / "target" / "parity-failures"
    candidates = (
        [path for path in root.iterdir() if (path / "diff.json").is_file()]
        if root.is_dir()
        else []
    )
    if not candidates:
        raise SystemExit(f"parity-probe: no failure receipts under {root}")
    return max(candidates, key=lambda path: path.stat().st_mtime).resolve()


def prune_probe_sessions(probe_root: Path, keep: int) -> list[Path]:
    """Remove only reproducible probe-* scratch sessions beyond `keep` newest."""
    if keep < 1 or not probe_root.is_dir():
        return []
    candidates = sorted(
        (
            path
            for path in probe_root.iterdir()
            if path.is_dir() and path.name.startswith("probe-")
        ),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    removed = candidates[keep:]
    for path in removed:
        shutil.rmtree(path)
    return removed


def load_failure_focus(value: Path) -> FailureFocus:
    directory = resolve_failure_dir(value)
    try:
        receipt = json.loads((directory / "diff.json").read_text(encoding="utf-8"))
        frame = int(receipt["frame"])
        message = str(receipt.get("message") or "")
    except (OSError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        raise SystemExit(f"parity-probe: invalid failure receipt in {directory}: {error}") from error
    match = FIRST_MISMATCH_RE.search(message)
    pixel = (int(match.group(1)), int(match.group(2))) if match else None
    return FailureFocus(directory=directory, frame=frame, pixel=pixel)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_output(*args: str) -> str:
    try:
        return subprocess.run(
            ["git", *args],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        ).stdout.strip()
    except OSError:
        return ""


def working_tree_identity() -> dict[str, str]:
    return {
        "head": git_output("rev-parse", "HEAD"),
        "describe": git_output("describe", "--always", "--dirty"),
        "diff_stat_sha256": hashlib.sha256(
            git_output("diff", "--stat").encode("utf-8")
        ).hexdigest(),
    }


def binary_identity(binary: Path) -> dict[str, object]:
    stat = binary.stat()
    return {
        "path": str(binary),
        "size": stat.st_size,
        "mtime_ns": stat.st_mtime_ns,
        "sha256": sha256_file(binary),
    }


def rust_source_affects_runtime_binary(path: Path) -> bool:
    """Exclude Rust sources that Cargo only compiles into test artifacts."""
    try:
        repo_path = path.relative_to(ROOT)
    except ValueError:
        repo_path = path
    return not (
        repo_path.parts[:2] == ("crates", "parity")
        or repo_path.name.endswith("_tests.rs")
        or "tests" in repo_path.parts
        or any(part.endswith("_tests") for part in repo_path.parts)
    )


def newest_source_mtime() -> tuple[float, Path | None]:
    newest = 0.0
    newest_path: Path | None = None
    for name in SOURCE_DIRS:
        for path in (ROOT / name).rglob("*.rs"):
            if "/target/" in str(path) or not rust_source_affects_runtime_binary(path):
                continue
            mtime = path.stat().st_mtime
            if mtime > newest:
                newest, newest_path = mtime, path
    return newest, newest_path


def validate_stale_override(allow_stale: bool, dry_run: bool) -> None:
    if allow_stale and not dry_run:
        raise SystemExit(
            "parity-probe: --allow-stale is restricted to --dry-run; "
            "rebuild the parity binary before collecting evidence"
        )


def replay_target_frame(run_dir: Path) -> int:
    _, replay_argv = parse_replay_script(run_dir / "replay.sh")
    if len(replay_argv) < 4:
        raise SystemExit(
            f"parity-probe: unexpected compare invocation in {run_dir / 'replay.sh'}"
        )
    try:
        return int(replay_argv[3])
    except ValueError as error:
        raise SystemExit(
            f"parity-probe: invalid target frame {replay_argv[3]!r} in {run_dir / 'replay.sh'}"
        ) from error


def replay_covered_frame(run_dir: Path, replay_argv: list[str] | None = None) -> int:
    """Return the replay bundle's proven coverage, not merely its requested target."""
    if replay_argv is None:
        requested = replay_target_frame(run_dir)
    else:
        try:
            requested = int(replay_argv[3])
        except (IndexError, ValueError) as error:
            raise SystemExit("parity-probe: malformed replay argv while checking bundle coverage") from error
    manifest_path = run_dir / "manifest.json"
    if not manifest_path.is_file():
        return requested
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        raw_completed = manifest.get("frames_completed")
        if raw_completed is None and (run_dir / "result.json").is_file():
            result = json.loads((run_dir / "result.json").read_text(encoding="utf-8"))
            raw_completed = result.get("frames_completed")
        # A session whose manifest was initialized but never finalized is not a
        # reusable replay bundle. Keep it out of automatic selection.
        completed = 0 if raw_completed is None else int(raw_completed)
    except (OSError, json.JSONDecodeError, TypeError, ValueError) as error:
        raise SystemExit(
            f"parity-probe: invalid completed-frame provenance in {manifest_path}: {error}"
        ) from error
    if completed < 0 or completed > requested:
        raise SystemExit(
            f"parity-probe: impossible completed-frame provenance in {manifest_path}: "
            f"completed={completed} requested={requested}"
        )
    return completed


def source_input_problem(run_dir: Path, required_frame: int) -> str | None:
    """Reject legacy failed sessions that replaced full input with a prefix."""
    completed = replay_covered_frame(run_dir)
    if completed >= required_frame:
        return None
    manifest_path = run_dir / "manifest.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return f"cannot verify the failed session's complete input artifact: {error}"
    receipt = manifest.get("input_replay")
    if not isinstance(receipt, dict) or receipt.get("mode") != "source_script":
        return (
            f"session completed only {completed} frame(s) and does not prove that "
            "input.txt still contains the complete source script"
        )
    input_path = run_dir / str(receipt.get("artifact", "input.txt"))
    expected_sha = receipt.get("sha256")
    if not input_path.is_file() or not isinstance(expected_sha, str):
        return "session input provenance receipt is incomplete"
    if sha256_file(input_path) != expected_sha:
        return "session input.txt does not match its complete-source provenance receipt"
    return None


def source_start_problem(run_dir: Path, binary: Path) -> str | None:
    _, replay_argv = parse_replay_script(run_dir / "replay.sh")
    rust_state = option_value(replay_argv, "--resume-rust-state")
    oracle_state = option_value(replay_argv, "--resume-oracle-state")
    if (rust_state is None) != (oracle_state is None):
        return "source replay has only one half of its paired resume state"
    if rust_state is None or oracle_state is None:
        return None
    paths = (Path(rust_state), Path(oracle_state))
    if not all(path.is_file() for path in paths):
        return "source replay's paired start states are missing"
    if min(path.stat().st_mtime for path in paths) < binary.stat().st_mtime:
        return "source replay's paired start predates the parity binary"
    return None


def source_start_is_cold(run_dir: Path) -> bool:
    _, replay_argv = parse_replay_script(run_dir / "replay.sh")
    return option_value(replay_argv, "--resume-rust-state") is None


def recorded_rom_random_problem(run_dir: Path) -> str | None:
    """Return why a gate replay cannot reproduce cartridge RNG for video parity."""
    _, replay_argv = parse_replay_script(run_dir / "replay.sh")
    value = option_value(replay_argv, "--rom-random-script")
    if value is None:
        return "replay has no recorded --rom-random-script"
    path = Path(value)
    path = path if path.is_absolute() else ROOT / path
    if not path.is_file():
        return f"recorded ROM-random stream is missing: {path}"
    return None


def resolve_run_dir(
    project: Path,
    override: Path | None,
    required_frame: int,
    binary: Path,
    *,
    require_cold: bool = False,
    require_recorded_rom_random: bool = False,
) -> Path:
    if override is not None:
        run_dir = override if override.is_absolute() else ROOT / override
        if not (run_dir / "replay.sh").is_file():
            raise SystemExit(f"parity-probe: {run_dir} has no replay.sh")
        covered_frame = replay_target_frame(run_dir)
        if covered_frame < required_frame:
            raise SystemExit(
                f"parity-probe: {run_dir} covers only {covered_frame} frame(s), "
                f"but this probe needs {required_frame}"
            )
        if problem := source_start_problem(run_dir, binary):
            raise SystemExit(f"parity-probe: cannot use {run_dir}: {problem}")
        if problem := source_input_problem(run_dir, required_frame):
            raise SystemExit(f"parity-probe: cannot use {run_dir}: {problem}")
        if require_cold and not source_start_is_cold(run_dir):
            raise SystemExit(
                f"parity-probe: cannot use {run_dir} for authoritative proof: "
                "its replay starts from paired states; select a cold run or pass "
                "--use-checkpoint for diagnostic-only probing"
            )
        if require_recorded_rom_random and (
            problem := recorded_rom_random_problem(run_dir)
        ):
            raise SystemExit(
                f"parity-probe: cannot use {run_dir} for video parity: {problem}; "
                "select a recorded-RNG video-preflight run"
            )
        return run_dir.resolve()
    precommit = project / "comparisons" / "precommit"
    candidates = [
        path
        for path in precommit.glob("run-*")
        if (path / "replay.sh").is_file() and (path / "input.txt").is_file()
    ]
    if not candidates:
        raise SystemExit(
            f"parity-probe: no usable run dir under {precommit}; run the pre-commit gate once first"
        )
    covered = [(replay_target_frame(path), path) for path in candidates]
    sufficient = [(frame, path) for frame, path in covered if frame >= required_frame]
    if not sufficient:
        available = max(frame for frame, _ in covered)
        raise SystemExit(
            f"parity-probe: newest precommit inputs cover only {available} frame(s), "
            f"but this probe needs {required_frame}; run the pre-commit gate farther first"
        )
    usable = [
        (frame, path)
        for frame, path in sufficient
        if source_start_problem(path, binary) is None
        and source_input_problem(path, required_frame) is None
        and (not require_cold or source_start_is_cold(path))
        and (
            not require_recorded_rom_random
            or recorded_rom_random_problem(path) is None
        )
    ]
    if not usable:
        raise SystemExit(
            "parity-probe: no sufficiently long run has an eligible start; "
            "run the pre-commit gate with this binary or keep a cold recorded-RNG "
            "video-preflight run with --load-sram"
        )
    # A shorter sufficient run is not necessarily compatible with the current
    # binary's control flow: recorded cartridge RNG is call-order sensitive.
    # Prefer the newest eligible gate receipt, which is the one most likely to
    # have been produced by the current build, instead of silently selecting an
    # older run merely because its target is closer to the requested window.
    return max((path for _, path in usable), key=lambda path: path.stat().st_mtime).resolve()


def parse_replay_script(path: Path) -> tuple[dict[str, str], list[str]]:
    """Return (env assignments, compare argv) from a session replay.sh."""
    command_line = next(
        (
            line
            for line in path.read_text(encoding="utf-8").splitlines()
            if "--compare-snes9x-oracle" in line
        ),
        None,
    )
    if command_line is None:
        raise SystemExit(f"parity-probe: {path} has no --compare-snes9x-oracle command")
    tokens = shlex.split(command_line)
    env: dict[str, str] = {}
    while tokens and "=" in tokens[0] and not tokens[0].startswith("-"):
        name, _, value = tokens[0].partition("=")
        env[name] = value
        tokens.pop(0)
    if "--" not in tokens:
        raise SystemExit(f"parity-probe: {path} is not a `cargo run -- ...` invocation")
    return env, tokens[tokens.index("--") + 1 :]


def option_value(argv: list[str], name: str) -> str | None:
    for index, token in enumerate(argv):
        if token == name and index + 1 < len(argv):
            return argv[index + 1]
    return None


def source_start_arguments(replay_argv: list[str], binary: Path) -> list[str]:
    load_sram = option_value(replay_argv, "--load-sram")
    rust_state = option_value(replay_argv, "--resume-rust-state")
    oracle_state = option_value(replay_argv, "--resume-oracle-state")
    if (rust_state is None) != (oracle_state is None):
        raise SystemExit(
            "parity-probe: source replay has only one half of its paired resume state"
        )
    if rust_state is not None and oracle_state is not None:
        if load_sram is not None:
            raise SystemExit(
                "parity-probe: source replay combines paired resume states with --load-sram"
            )
        compare_from = option_value(replay_argv, "--compare-from-frame")
        if compare_from is None:
            raise SystemExit(
                "parity-probe: resumed source replay has no --compare-from-frame boundary"
            )
        rust_state_path = Path(rust_state)
        oracle_state_path = Path(oracle_state)
        if not rust_state_path.is_file() or not oracle_state_path.is_file():
            raise SystemExit("parity-probe: source replay's paired start states are missing")
        if min(rust_state_path.stat().st_mtime, oracle_state_path.stat().st_mtime) < binary.stat().st_mtime:
            raise SystemExit(
                "parity-probe: source replay's paired start predates the parity binary"
            )
        arguments = [
            "--resume-rust-state",
            rust_state,
            "--resume-oracle-state",
            oracle_state,
        ]
        oracle_sram = option_value(replay_argv, "--resume-oracle-sram")
        if oracle_sram is not None:
            arguments += ["--resume-oracle-sram", oracle_sram]
        arguments += ["--compare-from-frame", compare_from]
        return arguments
    return ["--load-sram", load_sram] if load_sram is not None else []


def source_start_description(arguments: list[str]) -> str:
    if "--resume-rust-state" in arguments:
        return "binary-matched paired states from the selected gate run"
    if "--load-sram" in arguments:
        return "cold replay from the selected gate run's SRAM"
    return "cold replay from the emulator default state"


def cold_replay_bundle_available(
    run_dir: Path,
    replay_argv: list[str],
    *,
    target_frame: int,
    input_overridden: bool,
    resuming: bool,
) -> bool:
    """Return whether one run directory can supply the whole cold replay."""
    return (
        not input_overridden
        and not resuming
        and replay_covered_frame(run_dir, replay_argv) >= target_frame
        and option_value(replay_argv, "--load-sram") is not None
        and all(
            (run_dir / name).is_file()
            for name in ("manifest.json", "input.txt", "rom-random.txt", "initial.srm")
        )
    )


def validate_trace_core(
    core: Path,
    *,
    lock_path: Path = ORACLE_LOCK,
    patch_path: Path | None = None,
) -> str:
    """Return the verified core hash or reject non-reproducible trace evidence."""
    receipt_path = core.with_suffix(core.suffix + ".json")
    if not core.is_file():
        raise SystemExit(f"parity-probe: instrumented core is missing: {core}")
    if not receipt_path.is_file():
        raise SystemExit(
            f"parity-probe: instrumented core has no build receipt: {receipt_path}; "
            "rebuild it with scripts/prepare_snes9x_trace_oracle.py"
        )
    try:
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        lock = json.loads(lock_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"parity-probe: cannot verify instrumented core receipt: {error}") from error

    if receipt.get("schema") != 1 or receipt.get("variant") != "trace":
        raise SystemExit(
            f"parity-probe: {receipt_path} is not a schema-1 trace-core receipt"
        )
    for field in (
        "core_name",
        "core_version",
        "source_tag",
        "source_url",
        "source_revision",
    ):
        if receipt.get(field) != lock.get(field):
            raise SystemExit(
                f"parity-probe: instrumented core {field} does not match oracle-lock.json; "
                "rebuild the trace core"
            )

    patches = (patch_path,) if patch_path is not None else TRACE_PATCHES
    if len(patches) == 1:
        actual_patch_sha = sha256_file(patches[0])
    else:
        digest = hashlib.sha256()
        for patch in patches:
            digest.update(patch.name.encode("utf-8"))
            digest.update(b"\0")
            digest.update(patch.read_bytes())
            digest.update(b"\0")
        actual_patch_sha = digest.hexdigest()
    if receipt.get("patch_sha256") != actual_patch_sha:
        raise SystemExit(
            "parity-probe: instrumented core predates the current trace patch; "
            "rebuild it with scripts/prepare_snes9x_trace_oracle.py"
        )
    actual_core_sha = sha256_file(core)
    if receipt.get("core_sha256") != actual_core_sha:
        raise SystemExit(
            f"parity-probe: instrumented core hash does not match {receipt_path}"
        )
    if INSTRUMENTED_SYMBOL not in core.read_bytes():
        raise SystemExit(
            "parity-probe: receipt claims a trace core, but the core does not export "
            f"{INSTRUMENTED_SYMBOL.decode()}"
        )
    return actual_core_sha


def instrumented_core() -> Path:
    validate_trace_core(TRACE_CORE)
    return TRACE_CORE


def checkpoint_identity(
    *,
    frame: int,
    binary: Path,
    core_sha: str,
    rom_sha: str | None,
    input_path: Path,
    rom_random_path: Path | None,
) -> dict[str, object]:
    return {
        "schema": 1,
        "requested_frame": frame,
        "binary": binary_identity(binary),
        "git": working_tree_identity(),
        "core_sha256": core_sha,
        "rom_sha256": rom_sha,
        "input_sha256": sha256_file(input_path),
        "rom_random_sha256": sha256_file(rom_random_path) if rom_random_path else None,
        "created": time.strftime("%Y-%m-%dT%H:%M:%S"),
    }


def checkpoint_reuse_problem(
    checkpoint_dir: Path,
    wanted: dict[str, object],
    *,
    trust_cross_build: bool = False,
) -> str | None:
    """Return why `checkpoint_dir` cannot be resumed, or None when it can."""
    if trust_cross_build:
        saved = checkpoint_generation(checkpoint_dir)
        if saved is None:
            return "no saved checkpoint generation"
        _, generation = saved
        try:
            manifest = json.loads((generation / "manifest.json").read_text(encoding="utf-8"))
            members = [str(manifest[name]) for name in ("rust_state", "oracle_state")]
        except (OSError, json.JSONDecodeError, KeyError, TypeError):
            return "invalid paired checkpoint manifest"
        if any(Path(member).name != member for member in members):
            return "paired checkpoint manifest contains an unsafe state path"
        if not all((generation / member).is_file() for member in members):
            return "paired checkpoint is missing a state file"
        # Cross-build trust permits a different Rust binary, not a different
        # replay. Input and recorded cartridge RNG are part of the savestate's
        # causal history; mixing them can look like a parity regression many
        # frames after the resume point. Exact precommit checkpoints record
        # those hashes in their own manifest, while rolling probe checkpoints
        # also have an identity sidecar.
        identity: dict[str, object] = {}
        identity_path = checkpoint_dir / IDENTITY_NAME
        if identity_path.is_file():
            try:
                identity = json.loads(identity_path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                return "unreadable probe identity"
        provenance = {
            "rom_sha256": (manifest.get("rom") or {}).get("sha256"),
            "input_sha256": (manifest.get("input_script") or {}).get("sha256"),
            "rom_random_sha256": (manifest.get("rom_random_script") or {}).get("sha256"),
        }
        for key, label in (
            ("rom_sha256", "rom"),
            ("input_sha256", "input"),
            ("rom_random_sha256", "rom_random"),
        ):
            if key not in wanted:
                continue
            has_identity_value = key in identity
            recorded = identity.get(key) if has_identity_value else provenance[key]
            if recorded is None and not has_identity_value:
                return f"paired checkpoint has no recorded {label} provenance"
            if recorded != wanted[key]:
                return f"{label} changed since the checkpoint was saved"
        return None
    identity_path = checkpoint_dir / IDENTITY_NAME
    if not identity_path.is_file():
        return "no probe identity recorded"
    if latest_generation(checkpoint_dir) is None:
        return "no saved checkpoint generation"
    try:
        stored = json.loads(identity_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return "unreadable probe identity"
    if stored.get("schema") != wanted["schema"]:
        return "probe identity schema changed"
    stored_binary = stored.get("binary") or {}
    wanted_binary = wanted["binary"]
    assert isinstance(wanted_binary, dict)
    if stored_binary.get("sha256") != wanted_binary["sha256"]:
        return "zelda3 binary changed since the checkpoint was saved"
    for key in ("core_sha256", "rom_sha256", "input_sha256", "rom_random_sha256"):
        if stored.get(key) != wanted[key]:
            return f"{key.removesuffix('_sha256')} changed since the checkpoint was saved"
    return None


def latest_generation(checkpoint_dir: Path) -> tuple[int, Path] | None:
    """The newest paired generation a rolling checkpoint dir actually holds."""
    try:
        latest = json.loads((checkpoint_dir / "latest.json").read_text(encoding="utf-8"))
        frame, name = int(latest["frame"]), str(latest["checkpoint"])
    except (OSError, json.JSONDecodeError, KeyError, TypeError, ValueError):
        return None
    generation = checkpoint_dir / name
    return (frame, generation) if (generation / "manifest.json").is_file() else None


def checkpoint_generation(checkpoint_dir: Path) -> tuple[int, Path] | None:
    """Resolve either an exact paired checkpoint or a rolling checkpoint root."""
    manifest_path = checkpoint_dir / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            frame = int(manifest["frame"])
        except (OSError, json.JSONDecodeError, KeyError, TypeError, ValueError):
            return None
        return frame, checkpoint_dir
    return latest_generation(checkpoint_dir)


def saved_checkpoint_frame(checkpoint_dir: Path) -> int | None:
    saved = checkpoint_generation(checkpoint_dir)
    return None if saved is None else saved[0]


def default_frontier_checkpoint_dir(project: Path) -> Path:
    """Stable project-scoped cache path for repeated frontier investigations."""
    slug = re.sub(r"[^A-Za-z0-9_.-]+", "-", project.name).strip("-") or "project"
    identity = hashlib.sha256(str(project.resolve()).encode("utf-8")).hexdigest()[:8]
    return CHECKPOINT_ROOT / f"frontier-{slug}-{identity}"


def checkpoint_policy(
    *,
    frontier_mode: bool,
    no_checkpoint: bool,
    explicit_use: bool,
    checkpoint_frame: int | None,
    checkpoint_dir: Path | None,
    trust_cross_build: bool,
    project: Path,
) -> tuple[bool, bool, Path | None]:
    """Resolve fast diagnostic defaults without weakening focused/cold probes."""
    explicitly_requested = (
        explicit_use
        or checkpoint_frame is not None
        or checkpoint_dir is not None
        or trust_cross_build
    )
    if no_checkpoint:
        return False, False, checkpoint_dir
    if frontier_mode and not explicitly_requested:
        return True, True, default_frontier_checkpoint_dir(project)
    return explicitly_requested, trust_cross_build, checkpoint_dir


def checkpoint_result_is_promotable(returncode: int, result: dict) -> bool:
    """Only a successful cold exact-A/V comparison may become reusable state."""
    video = result.get("video") or {}
    audio = result.get("audio") or {}
    return (
        returncode == 0
        and result.get("status") == "passed"
        and result.get("parity_eligible") is True
        and video.get("matched") is True
        and audio.get("matched") is True
    )


def checkpoint_result_is_diagnostic(returncode: int, result: dict) -> bool:
    """A renderless exact-engine pass may seed only the diagnostic checkpoint cache."""
    engine = result.get("engine_state") or {}
    return (
        returncode == 0
        and result.get("status") == "passed"
        and result.get("parity_eligible") is True
        and engine.get("matched") is True
    )


def diagnostic_checkpoint_before_failure(
    candidate_dir: Path | None, mismatch_frame: int | None
) -> Path | None:
    """Return an isolated pre-failure candidate suitable only for trace replay."""
    if candidate_dir is None or mismatch_frame is None:
        return None
    saved = saved_checkpoint_frame(candidate_dir)
    return candidate_dir if saved is not None and saved < mismatch_frame else None


def checkpoint_is_impractically_early(
    *, automatic_rolling: bool, checkpoint_frame: int, target_frame: int
) -> bool:
    """The legacy fixed-frame heuristic does not apply to periodic captures."""
    return not automatic_rolling and 2 * checkpoint_frame <= target_frame


def checkpoint_interval(frontier_mode: bool, around: int, requested: int | None) -> int:
    if requested is not None:
        return requested
    if frontier_mode:
        return max(1, min(AUTOMATIC_ROLLING_CHECKPOINT_INTERVAL, around - 60))
    return max(0, around - 60)


def should_stage_rolling_checkpoint(
    checkpoint_dir: Path | None, resume_dir: Path | None, frontier_mode: bool
) -> bool:
    return checkpoint_dir is not None and (resume_dir is None or frontier_mode)


def promote_checkpoint_candidate(
    candidate_dir: Path,
    checkpoint_dir: Path,
    wanted: dict,
    stamp: str,
) -> tuple[int, Path | None]:
    """Atomically trust a verified candidate, preserving any replaced cache."""
    saved_frame = saved_checkpoint_frame(candidate_dir)
    if saved_frame is None:
        raise ValueError(f"checkpoint candidate has no complete generation: {candidate_dir}")
    quarantined: Path | None = None
    if checkpoint_dir.exists():
        quarantined = checkpoint_dir.with_name(f"rejected-{checkpoint_dir.name}-{stamp}")
        suffix = 1
        while quarantined.exists():
            quarantined = checkpoint_dir.with_name(
                f"rejected-{checkpoint_dir.name}-{stamp}-{suffix}"
            )
            suffix += 1
        shutil.move(str(checkpoint_dir), str(quarantined))
    checkpoint_dir.parent.mkdir(parents=True, exist_ok=True)
    shutil.move(str(candidate_dir), str(checkpoint_dir))
    identity = dict(wanted)
    identity["saved_frame"] = saved_frame
    (checkpoint_dir / IDENTITY_NAME).write_text(
        json.dumps(identity, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return saved_frame, quarantined


def registers(probe: dict) -> list[int]:
    return [
        int(probe["mode"]),
        int(probe["brightness"]),
        int(bool(probe["forced_blank"])),
        *(int(value) for value in probe["fixed_color"]),
        *(int(value) for value in probe["display_control"]),
        *(int(value) for value in probe["bg_scroll"]),
    ]


def window_column(probe: dict, window: int) -> list[int]:
    return [
        int(value)
        for scanline in probe.get("window_scanlines", [])
        for value in scanline[window * 2 : window * 2 + 2]
    ]


def differing_indices(rust: list[int], oracle: list[int]) -> list[int]:
    shared = [
        index
        for index, (left, right) in enumerate(zip(rust, oracle))
        if left != right
    ]
    shared.extend(range(min(len(rust), len(oracle)), max(len(rust), len(oracle))))
    return shared


def oam_slot(values: list[int], slot: int) -> str:
    base = slot * 4
    if base + 3 >= len(values):
        return "absent"
    x, y, tile, attr = values[base : base + 4]
    return f"x={x:3d} y={y:3d} tile=0x{tile:02x} attr=0x{attr:02x}"


def oam_high_bits(values: list[int], slot: int) -> str:
    index = OAM_LOW_VALUES + slot // 4
    if index >= len(values):
        return "absent"
    bits = (values[index] >> ((slot % 4) * 2)) & 0b11
    return f"x8={bits & 1} size={(bits >> 1) & 1}"


def summarize_oam(label: str, rust: list[int], oracle: list[int]) -> list[str]:
    slots = sorted({index // 4 for index in differing_indices(rust, oracle) if index < OAM_LOW_VALUES})
    high_slots = sorted(
        {
            (index - OAM_LOW_VALUES) * 4 + offset
            for index in differing_indices(rust, oracle)
            if index >= OAM_LOW_VALUES
            for offset in range(4)
        }
    )
    lines = []
    for slot in slots[:MAX_DETAIL_LINES]:
        lines.append(
            f"    {label} slot {slot:3d}: rust {oam_slot(rust, slot)} | oracle {oam_slot(oracle, slot)}"
        )
    if len(slots) > MAX_DETAIL_LINES:
        lines.append(f"    {label}: +{len(slots) - MAX_DETAIL_LINES} more slots")
    shown_high = [
        slot
        for slot in high_slots
        if oam_high_bits(rust, slot) != oam_high_bits(oracle, slot)
    ]
    for slot in shown_high[:MAX_DETAIL_LINES]:
        lines.append(
            f"    {label} high slot {slot:3d}: rust {oam_high_bits(rust, slot)} "
            f"| oracle {oam_high_bits(oracle, slot)}"
        )
    if len(shown_high) > MAX_DETAIL_LINES:
        lines.append(f"    {label} high: +{len(shown_high) - MAX_DETAIL_LINES} more slots")
    return lines


def flatten_mode7_scanlines(probe: dict) -> list[int]:
    return [int(value) for scanline in probe.get("mode7_scanlines", []) for value in scanline]


def valid_obj_tile_cache_differences(
    rust: dict, oracle: dict
) -> tuple[list[int], int] | None:
    """Compare only OBJ tiles the enhanced oracle says completed scanout decoded.

    The stock instrumented core predates this receipt and returns ``-1`` for
    every validity entry. Treat that as unavailable instead of manufacturing a
    permanent cache mismatch from unsupported fields.
    """
    left = rust.get("presented_obj_tile_cache")
    right = oracle.get("presented_obj_tile_cache")
    valid = oracle.get("presented_obj_tile_cache_valid")
    if not isinstance(left, list) or not isinstance(right, list) or not isinstance(valid, list):
        return None
    if not valid or any(value not in (0, 1) for value in valid):
        return None
    differing: list[int] = []
    compared = 0
    for tile, is_valid in enumerate(valid):
        if not is_valid:
            continue
        start, end = tile * 64, (tile + 1) * 64
        left_tile, right_tile = left[start:end], right[start:end]
        compared += max(len(left_tile), len(right_tile))
        differing.extend(start + index for index in differing_indices(left_tile, right_tile))
    return differing, compared


def candidate_publication_matches(receipt: dict) -> list[str]:
    """Rank existing Rust publication generations against the oracle.

    The candidate matrix is diagnostic-only. Exact matches identify an already
    captured generation that the resolver selected incorrectly; nearest matches
    keep the next investigation bounded when the required generation is mixed.
    """
    rust, oracle = receipt["rust"], receipt["oracle"]
    candidates = receipt.get("rust_candidates", [])
    if not isinstance(candidates, list):
        return []
    recommendations: list[str] = []

    selected_cgram = list(rust.get("cgram", []))
    oracle_cgram = list(oracle.get("cgram", []))
    if differing_indices(selected_cgram, oracle_cgram):
        scores: list[tuple[int, str]] = []
        for candidate in candidates:
            values = candidate.get("cgram") if isinstance(candidate, dict) else None
            if isinstance(values, list):
                scores.append((len(differing_indices(values, oracle_cgram)), str(candidate["name"])))
        if scores:
            scores.sort()
            count, name = scores[0]
            qualifier = "exact" if count == 0 else f"nearest, {count} color(s) differ"
            recommendations.append(f"cgram=>{name} ({qualifier})")

    selected_oam = list(rust.get("presented_oam", []))
    oracle_oam = list(oracle.get("presented_oam", []))
    if differing_indices(selected_oam, oracle_oam):
        scores: list[tuple[int, str]] = []
        for candidate in candidates:
            values = candidate.get("presented_oam") if isinstance(candidate, dict) else None
            if isinstance(values, list):
                scores.append((len(differing_indices(values, oracle_oam)), str(candidate["name"])))
        if scores:
            scores.sort()
            count, name = scores[0]
            qualifier = "exact" if count == 0 else f"nearest, {count} byte(s) differ"
            recommendations.append(f"presented_oam=>{name} ({qualifier})")

    selected_cache = valid_obj_tile_cache_differences(rust, oracle)
    if selected_cache is not None and selected_cache[0]:
        valid = oracle.get("presented_obj_tile_cache_valid", [])
        oracle_cache = oracle.get("presented_obj_tile_cache", [])
        scores = []
        for candidate in candidates:
            values = (
                candidate.get("presented_obj_tile_cache")
                if isinstance(candidate, dict)
                else None
            )
            if not isinstance(values, list):
                continue
            mismatches = 0
            for tile, is_valid in enumerate(valid):
                if is_valid != 1:
                    continue
                start, end = tile * 64, (tile + 1) * 64
                mismatches += len(
                    differing_indices(values[start:end], oracle_cache[start:end])
                )
            scores.append((mismatches, str(candidate["name"])))
        if scores:
            scores.sort()
            count, name = scores[0]
            qualifier = "exact" if count == 0 else f"nearest, {count} pixel index(es) differ"
            recommendations.append(f"presented_obj_tile_cache=>{name} ({qualifier})")
    return recommendations


def oracle_previous_frame_holds(receipt: dict, previous_oracle: dict | None) -> list[str]:
    """Identify cascades caused by failing to retain an oracle generation.

    Rust's ``last_presented`` candidate becomes contaminated after the first
    wrong selection, so per-frame candidate ranking cannot recognize a long
    hold chain. The oracle's own preceding receipt remains authoritative and
    exposes that chain directly.
    """
    if not isinstance(previous_oracle, dict):
        return []
    rust, oracle = receipt["rust"], receipt["oracle"]
    domains = [
        ("registers", registers(rust), registers(oracle), registers(previous_oracle)),
        (
            "cgram",
            list(rust.get("cgram", [])),
            list(oracle.get("cgram", [])),
            list(previous_oracle.get("cgram", [])),
        ),
        (
            "presented_oam",
            list(rust.get("presented_oam", [])),
            list(oracle.get("presented_oam", [])),
            list(previous_oracle.get("presented_oam", [])),
        ),
    ]
    return [
        f"{name}=>oracle_previous_frame_chain (exact)"
        for name, selected, current, previous in domains
        if differing_indices(selected, current) and current == previous
    ]


def format_frame_ranges(frames: list[int]) -> str:
    ordered = sorted(set(frames))
    if not ordered:
        return ""
    ranges: list[tuple[int, int]] = []
    for frame in ordered:
        if ranges and frame == ranges[-1][1] + 1:
            ranges[-1] = (ranges[-1][0], frame)
        else:
            ranges.append((frame, frame))
    return ", ".join(
        str(start) if start == end else f"{start}..{end}"
        for start, end in ranges
    )


def format_publication_context(receipt: dict) -> str | None:
    context = receipt.get("rust_context")
    if not isinstance(context, dict):
        return None
    entry = context.get("entry_frame", [])
    following = context.get("following_frame", [])
    if len(entry) != 4 or len(following) != 4:
        return None
    phase = lambda values: f"{values[0]:02x}/{values[1]:02x}/{values[2]:02x}/fc{values[3]:02x}"
    return (
        f"entry={phase(entry)} following={phase(following)} "
        f"room={int(context.get('dungeon_room', 0)):02x} "
        f"stair={int(context.get('staircase_index', 0)):02x} "
        f"palette={int(context.get('palette_filter_countdown', 0)):04x} "
        f"nmi={int(context.get('nmi_update_latch', 0)):02x} "
        f"oam={context.get('oam_scanout_source')} "
        f"retain={context.get('retain_captured_oam')} "
        f"link={context.get('link_obj_scanout_generation')}/"
        f"{context.get('link_obj_source_generation')} "
        f"captured_host_diff={int(context.get('captured_to_host_oam_mismatches', 0))}"
    )


def summarize_receipt(receipt: dict) -> tuple[list[str], list[str]]:
    rust, oracle = receipt["rust"], receipt["oracle"]
    domains: list[tuple[str, list[int], list[int]]] = [
        ("registers", registers(rust), registers(oracle)),
        ("cgram", rust["cgram"], oracle["cgram"]),
        ("live_oam", rust["oam"], oracle["oam"]),
        ("presented_oam", rust["presented_oam"], oracle["presented_oam"]),
        ("window1_scanlines", window_column(rust, 0), window_column(oracle, 0)),
        ("window2_scanlines", window_column(rust, 1), window_column(oracle, 1)),
    ]
    if int(rust["mode"]) == 7 or int(oracle["mode"]) == 7:
        domains.append(("mode7", rust["mode7"], oracle["mode7"]))
        domains.append(
            (
                "mode7_scanlines",
                flatten_mode7_scanlines(rust),
                flatten_mode7_scanlines(oracle),
            )
        )

    headline, detail = [], []
    for name, left, right in domains:
        indices = differing_indices(list(left), list(right))
        if not indices:
            continue
        headline.append(f"{name} {len(indices)}/{max(len(left), len(right))}")
        if name == "registers":
            for index in indices[:MAX_DETAIL_LINES]:
                label = REGISTER_LABELS[index] if index < len(REGISTER_LABELS) else f"[{index}]"
                detail.append(f"    registers {label}: rust={left[index]} oracle={right[index]}")
        elif name == "cgram":
            shown = ", ".join(
                f"{index}: 0x{left[index]:04x}/0x{right[index]:04x}" for index in indices[:MAX_DETAIL_LINES]
            )
            more = f" (+{len(indices) - MAX_DETAIL_LINES} more)" if len(indices) > MAX_DETAIL_LINES else ""
            detail.append(f"    cgram rust/oracle {shown}{more}")
        elif name in ("live_oam", "presented_oam"):
            detail.extend(summarize_oam(name, list(left), list(right)))
        else:
            shown = ", ".join(str(index) for index in indices[:MAX_DETAIL_LINES])
            more = f" (+{len(indices) - MAX_DETAIL_LINES} more)" if len(indices) > MAX_DETAIL_LINES else ""
            detail.append(f"    {name} first differing indices: {shown}{more}")
    obj_cache = valid_obj_tile_cache_differences(rust, oracle)
    if obj_cache is not None:
        indices, compared = obj_cache
        if indices:
            headline.append(f"presented_obj_tile_cache {len(indices)}/{compared}")
            tiles = sorted({index // 64 for index in indices})
            shown = ", ".join(f"0x{tile:02x}" for tile in tiles[:MAX_DETAIL_LINES])
            more = f" (+{len(tiles) - MAX_DETAIL_LINES} more)" if len(tiles) > MAX_DETAIL_LINES else ""
            detail.append(f"    presented_obj_tile_cache differing tiles: {shown}{more}")
    return headline, detail


def split_display_domains(domain_names: list[str]) -> tuple[list[str], list[str]]:
    causal = [name for name in domain_names if name not in POST_FRAME_ONLY_DISPLAY_DOMAINS]
    post_frame_only = [name for name in domain_names if name in POST_FRAME_ONLY_DISPLAY_DOMAINS]
    return causal, post_frame_only


def mismatch_count(domain: object) -> int:
    return int(domain.get("mismatched_values", 0)) if isinstance(domain, dict) else 0


def report_failure_focus(focus: FailureFocus) -> None:
    print(f"parity-probe: failure receipt {focus.directory} (frame {focus.frame})")
    if focus.pixel is not None:
        print(f"parity-probe: first bad pixel {focus.pixel[0]},{focus.pixel[1]}")

    display_path = focus.directory / "display_oracle.json"
    if display_path.is_file():
        try:
            differences = json.loads(display_path.read_text(encoding="utf-8"))["differences"]
            divergent = [str(name) for name in differences.get("divergent_domains", [])]
        except (OSError, json.JSONDecodeError, KeyError, TypeError):
            differences, divergent = {}, []
        causal, post_frame_only = split_display_domains(divergent)
        if causal:
            print(f"parity-probe: scanout-causal display candidates: {', '.join(causal)}")
        if post_frame_only:
            print(
                "parity-probe: post-frame-only differences: "
                f"{', '.join(post_frame_only)} (do not use these to explain completed pixels)"
            )
        if not causal and mismatch_count(differences.get("presented_oam")) == 0:
            print("parity-probe: presented OAM is exact; skip OAM generation hypotheses")

    generations_path = focus.directory / "vram_generations.json"
    if generations_path.is_file():
        try:
            generations = json.loads(generations_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            generations = {}
        visible = mismatch_count(generations.get("visible_scanout"))
        live = mismatch_count(generations.get("live_after_frame"))
        if visible:
            print(
                f"parity-probe: visible VRAM differs in {visible} byte(s); "
                "trace the first bad pixel's tile/source generation"
            )
        elif live:
            print(
                f"parity-probe: only live post-frame VRAM differs ({live} byte(s)); "
                "do not attribute the completed scanout to it"
            )


def report_display_oracle(session_dir: Path) -> None:
    path = session_dir / "display_oracle.jsonl"
    if not path.is_file():
        return
    receipts = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not receipts:
        print(f"display-oracle: {path} is empty (no capture frames were reached)")
        return
    print(f"\ndisplay-oracle receipts ({len(receipts)}) from {path}:")
    causal_detailed = 0
    post_frame_detailed = 0
    candidate_clusters: dict[tuple[str, ...], list[int]] = {}
    candidate_contexts: list[tuple[int, str, tuple[str, ...]]] = []
    after_oracles = {
        int(receipt["frame"]): receipt["oracle"]
        for receipt in receipts
        if receipt.get("stage") == "after" and isinstance(receipt.get("oracle"), dict)
    }
    for receipt in receipts:
        headline, detail = summarize_receipt(receipt)
        candidate_matches = candidate_publication_matches(receipt)
        stage, frame = receipt["stage"], receipt["frame"]
        if stage == "after":
            candidate_matches.extend(
                oracle_previous_frame_holds(receipt, after_oracles.get(int(frame) - 1))
            )
        if not headline:
            print(f"  frame {frame} [{stage}]: all display domains exact")
            continue
        names = [domain.split(" ", 1)[0] for domain in headline]
        causal, post_frame_only = split_display_domains(names)
        classification = []
        if causal:
            classification.append(f"scanout-causal={','.join(causal)}")
        if post_frame_only:
            classification.append(f"post-frame-only={','.join(post_frame_only)}")
        suffix = f" ({'; '.join(classification)})" if classification else ""
        print(f"  frame {frame} [{stage}]: {', '.join(headline)}{suffix}")
        if candidate_matches:
            print(f"    candidate match: {'; '.join(candidate_matches)}")
            if stage == "after":
                candidate_clusters.setdefault(tuple(candidate_matches), []).append(int(frame))
                context = format_publication_context(receipt)
                if context is not None and any("(exact)" in match for match in candidate_matches):
                    candidate_contexts.append((int(frame), context, tuple(candidate_matches)))
        if causal and causal_detailed < MAX_DETAIL_RECEIPTS:
            print("\n".join(detail))
            causal_detailed += 1
        elif causal and causal_detailed == MAX_DETAIL_RECEIPTS:
            print("    (per-domain detail shown for the first diverging receipts only)")
            causal_detailed += 1
        elif post_frame_only and post_frame_detailed == 0:
            print("\n".join(detail))
            post_frame_detailed += 1
    if candidate_clusters:
        print("\npublication-candidate clusters:")
        for signature, frames in candidate_clusters.items():
            print(f"  {format_frame_ranges(frames)}: {'; '.join(signature)}")
    if candidate_contexts:
        print("\nexact-candidate timing contexts:")
        for frame, context, signature in candidate_contexts:
            print(f"  {frame}: {'; '.join(signature)} | {context}")


def report_display_classification(session_dir: Path) -> None:
    capture = session_dir / "display_oracle.jsonl"
    classifier = ROOT / "scripts" / "classify_display_oracle.py"
    if not capture.is_file() or not classifier.is_file():
        return
    completed = subprocess.run(
        [sys.executable, str(classifier), str(capture)],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode == 0:
        print("\nautomatic display root-cause classification:")
        print(completed.stdout.rstrip())
    elif completed.stderr:
        print(
            f"parity-probe: display classifier failed: {completed.stderr.strip()}",
            file=sys.stderr,
        )


def report_result(session_dir: Path) -> None:
    path = session_dir / "result.json"
    if not path.is_file():
        print(f"parity-probe: no result.json in {session_dir}", file=sys.stderr)
        return
    result = json.loads(path.read_text(encoding="utf-8"))
    video = result.get("video") or {}
    audio = result.get("audio") or {}
    print(
        f"\nresult: status={result.get('status')} parity_eligible={result.get('parity_eligible')} "
        f"video_matched={video.get('matched')} audio_matched={audio.get('matched')}"
    )
    if video.get("first_mismatch"):
        print(f"  first video mismatch: {video['first_mismatch']}")
    if audio.get("first_mismatch"):
        print(f"  first audio mismatch: {audio['first_mismatch']}")


def first_video_mismatch_frame(result: dict) -> int | None:
    video = result.get("video") or {}
    ranges = video.get("mismatch_ranges") or []
    try:
        return int(ranges[0][0]) if ranges else None
    except (IndexError, TypeError, ValueError):
        return None


def frontier_provenance_onsets(receipts: list[dict]) -> list[tuple[int, str, object, object]]:
    """Return the onset of divergence chains still active at the frontier.

    Checkpoint state can contain known diagnostic-only differences, so a
    frontier report deliberately ignores domains that never became exact and
    transient mismatches that recovered before the visible failure.
    """
    if not receipts:
        return []
    found: list[tuple[int, str, object, object]] = []
    final = receipts[-1]
    final_rust = final.get("rust_engine") or {}
    final_oracle = final.get("oracle_engine") or {}
    for field in FRONTIER_PROVENANCE_FIELDS:
        if final_rust.get(field) == final_oracle.get(field):
            continue
        onset_index = len(receipts) - 1
        while onset_index > 0:
            previous = receipts[onset_index - 1]
            previous_rust = previous.get("rust_engine") or {}
            previous_oracle = previous.get("oracle_engine") or {}
            if previous_rust.get(field) == previous_oracle.get(field):
                break
            onset_index -= 1
        if onset_index == 0:
            continue
        onset = receipts[onset_index]
        rust = onset.get("rust_engine") or {}
        oracle = onset.get("oracle_engine") or {}
        found.append((int(onset.get("frame", 0)), field, rust.get(field), oracle.get(field)))

    final_vram = final.get("vram") or {}
    if int(final_vram.get("mismatched_words", 0)):
        onset_index = len(receipts) - 1
        while onset_index > 0 and int(
            (receipts[onset_index - 1].get("vram") or {}).get("mismatched_words", 0)
        ):
            onset_index -= 1
        if onset_index > 0:
            onset_vram = receipts[onset_index].get("vram") or {}
            found.append(
                (
                    int(receipts[onset_index].get("frame", 0)),
                    "vram",
                    onset_vram.get("first_rust_word"),
                    onset_vram.get("first_oracle_word"),
                )
            )
    return sorted(found, key=lambda item: (item[0], item[1]))


def newest_failure_since(started_ns: int) -> FailureFocus | None:
    root = ROOT / "target" / "parity-failures"
    candidates = (
        [
            path
            for path in root.iterdir()
            if (path / "diff.json").is_file() and path.stat().st_mtime_ns >= started_ns
        ]
        if root.is_dir()
        else []
    )
    if not candidates:
        return None
    return load_failure_focus(max(candidates, key=lambda path: path.stat().st_mtime_ns))


def report_frontier_provenance(session_dir: Path, through_frame: int | None) -> None:
    path = session_dir / "frame_receipts.jsonl"
    if not path.is_file():
        return
    receipts = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            receipt = json.loads(line)
        except json.JSONDecodeError:
            # A comparator interrupted at the frontier can leave one partial
            # trailing JSONL record. Preserve every complete receipt and let
            # the failure artifact remain authoritative for the bad frame.
            continue
        if through_frame is None or int(receipt.get("frame", 0)) <= through_frame:
            receipts.append(receipt)
    onsets = frontier_provenance_onsets(receipts)
    if not onsets:
        print("frontier provenance: no newly divergent tracked state before the video frontier")
        return
    print("\nfrontier provenance (new divergence after an exact receipt baseline):")
    for frame, field, rust, oracle in onsets:
        print(f"  frame {frame}: {field} rust={rust!r} oracle={oracle!r}")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    target = parser.add_mutually_exclusive_group(required=True)
    target.add_argument("--around", type=int, help="suspect frame to probe")
    target.add_argument(
        "--from-failure",
        type=Path,
        nargs="?",
        const=Path("latest"),
        help=(
            "derive the frame, first bad pixel, and causal domains from a failure dir "
            "(default latest)"
        ),
    )
    target.add_argument(
        "--frontier",
        type=int,
        help=(
            "scan through this frame, report newly divergent state provenance, and "
            "automatically capture the first video mismatch with the trace core; "
            "uses a rolling diagnostic checkpoint unless --no-checkpoint is passed"
        ),
    )
    parser.add_argument(
        "--window",
        type=int,
        help="frames to run past the suspect frame (default 40, or 2 with --from-failure)",
    )
    parser.add_argument(
        "--project",
        default=DEFAULT_PROJECT,
        help=f"route project (default {DEFAULT_PROJECT})",
    )
    parser.add_argument("--run-dir", type=Path, help="reuse this precommit run dir instead of the newest")
    parser.add_argument(
        "--input-script",
        type=Path,
        help="override the selected gate run's input.txt with an explicitly preserved input stream",
    )
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--capture", action="store_true", help="write display_oracle.jsonl around --around")
    parser.add_argument(
        "--capture-range",
        type=parse_frame_range,
        metavar="START-END",
        help="capture an explicit inclusive receipt range instead of only around --around",
    )
    parser.add_argument(
        "--core",
        choices=("pinned", "instrumented"),
        help="core lane (default pinned, or instrumented with --capture)",
    )
    parser.add_argument("--core-path", type=Path, help="explicit core dylib (overrides --core)")
    parser.add_argument(
        "--use-checkpoint",
        action="store_true",
        help="use a paired checkpoint for diagnostic-only probing",
    )
    parser.add_argument(
        "--checkpoint-frame",
        type=int,
        help="paired checkpoint frame (implies --use-checkpoint; default around-60)",
    )
    parser.add_argument(
        "--checkpoint-dir",
        type=Path,
        help="explicit paired checkpoint dir (implies --use-checkpoint)",
    )
    parser.add_argument(
        "--trust-cross-build-checkpoint",
        action="store_true",
        help="reuse a paired checkpoint after code changes for diagnostics only; cold proof is still required",
    )
    parser.add_argument(
        "--no-checkpoint",
        action="store_true",
        help="force a cold frontier replay (required for authoritative proof)",
    )
    parser.add_argument("--session-dir", type=Path, help="explicit session dir for this probe")
    parser.add_argument(
        "--keep-probe-sessions",
        type=int,
        default=12,
        help="retain this many newest generated probe-* sessions (default 12; 0 disables pruning)",
    )
    parser.add_argument(
        "--allow-stale",
        action="store_true",
        help="skip the binary-vs-sources staleness guard for --dry-run only",
    )
    parser.add_argument("--dry-run", action="store_true", help="print the command without running it")
    parser.add_argument(
        "--no-frontier-capture",
        action="store_true",
        help="with --frontier, report the first mismatch without launching the trace-core rerun",
    )
    parser.add_argument(
        "--video-only",
        action="store_true",
        help="disable audio comparison for diagnostic checkpoint/trace replays",
    )
    parser.add_argument(
        "--trace-only",
        action="store_true",
        help="disable video and audio while retaining engine-state/trace comparison; diagnostic only",
    )
    parser.add_argument(
        "--live-oracle-rng",
        action="store_true",
        help="source cartridge RNG from this same instrumented oracle run; diagnostic only",
    )
    parser.add_argument(
        "--rom-random-script",
        type=Path,
        help="override the selected run's recorded RNG script with an explicitly materialized one",
    )
    parser.add_argument(
        "--engine-state-from-frame",
        type=int,
        default=200,
        help="first frame for live-RNG engine-state comparison (default 200)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.trace_only and args.video_only:
        raise SystemExit("parity-probe: --trace-only cannot be combined with --video-only")
    if args.live_oracle_rng and args.rom_random_script is not None:
        raise SystemExit(
            "parity-probe: --live-oracle-rng cannot be combined with --rom-random-script"
        )
    if args.keep_probe_sessions < 0:
        raise SystemExit("parity-probe: --keep-probe-sessions must be non-negative")
    validate_stale_override(args.allow_stale, args.dry_run)
    failure_focus = load_failure_focus(args.from_failure) if args.from_failure else None
    if failure_focus is not None:
        args.around = failure_focus.frame
        args.capture = True
        report_failure_focus(failure_focus)
    if args.capture_range is not None:
        args.capture = True
    frontier_mode = args.frontier is not None
    if frontier_mode:
        args.around = args.frontier
        args.window = 0
        if args.core is None and args.core_path is None:
            args.core = "pinned"
    assert args.around is not None
    if args.window is None:
        args.window = 2 if failure_focus is not None else 40
    binary = args.binary if args.binary.is_absolute() else ROOT / args.binary
    if not binary.is_file():
        print(
            f"parity-probe: parity binary missing ({binary}); build it with "
            "`cargo build --profile parity -p zelda3-bin`",
            file=sys.stderr,
        )
        return 1
    if not args.allow_stale:
        newest, newest_path = newest_source_mtime()
        if newest > binary.stat().st_mtime:
            print(
                f"parity-probe: {binary} is older than {newest_path}; rebuild the parity binary",
                file=sys.stderr,
            )
            return 1

    project = Path(args.project)
    project = project if project.is_absolute() else ROOT / project
    target_frame = max(
        args.around + max(0, args.window),
        args.capture_range[1] if args.capture_range is not None else 0,
    )
    use_checkpoint, trust_cross_build, automatic_checkpoint_dir = checkpoint_policy(
        frontier_mode=frontier_mode,
        no_checkpoint=args.no_checkpoint,
        explicit_use=args.use_checkpoint,
        checkpoint_frame=args.checkpoint_frame,
        checkpoint_dir=args.checkpoint_dir,
        trust_cross_build=args.trust_cross_build_checkpoint,
        project=project,
    )
    explicitly_requested_checkpoint = (
        args.use_checkpoint
        or args.checkpoint_frame is not None
        or args.checkpoint_dir is not None
        or args.trust_cross_build_checkpoint
    )
    if args.no_checkpoint and explicitly_requested_checkpoint:
        raise SystemExit(
            "parity-probe: --no-checkpoint cannot be combined with checkpoint options"
        )
    args.trust_cross_build_checkpoint = trust_cross_build
    if args.checkpoint_dir is None:
        args.checkpoint_dir = automatic_checkpoint_dir
    run_dir = resolve_run_dir(
        project,
        args.run_dir,
        target_frame,
        binary,
        # Capture mode is diagnostic by definition. It may safely start from a
        # paired gate receipt, and doing so keeps its input/RNG provenance tied
        # to the newest current-build failure instead of falling back to an old
        # cold run with a now-incompatible RNG call order.
        require_cold=not (use_checkpoint or args.capture),
        require_recorded_rom_random=(
            not args.live_oracle_rng and args.rom_random_script is None
        ),
    )
    script_env, replay_argv = parse_replay_script(run_dir / "replay.sh")

    if len(replay_argv) < 4 or replay_argv[0] != "--compare-snes9x-oracle":
        raise SystemExit(
            f"parity-probe: unexpected compare invocation in {run_dir / 'replay.sh'}"
        )
    pinned_core, rom = replay_argv[1], replay_argv[2]
    pinned_core_sha = option_value(replay_argv, "--expected-core-sha256")
    if args.core_path is not None:
        core = (args.core_path if args.core_path.is_absolute() else ROOT / args.core_path).resolve()
    elif (args.core or ("instrumented" if args.capture else "pinned")) == "instrumented":
        core = instrumented_core()
    else:
        core = Path(pinned_core)
    core_is_instrumented = core.is_file() and INSTRUMENTED_SYMBOL in core.read_bytes()
    if core_is_instrumented:
        core_sha = validate_trace_core(core)
    else:
        core_sha = pinned_core_sha if str(core) == pinned_core else sha256_file(core)

    input_path = args.input_script or Path(
        option_value(replay_argv, "--input-script") or run_dir / "input.txt"
    )
    input_path = input_path if input_path.is_absolute() else ROOT / input_path
    if not input_path.is_file():
        raise SystemExit(
            f"parity-probe: input stream is missing: {input_path}; "
            "pass --input-script with the preserved authoritative stream"
        )
    rom_random_path = None if args.live_oracle_rng else (
        str(args.rom_random_script.resolve())
        if args.rom_random_script is not None
        else option_value(replay_argv, "--rom-random-script")
    )
    rom_random = Path(rom_random_path) if rom_random_path else None
    if rom_random is not None and not rom_random.is_file():
        raise SystemExit(f"parity-probe: ROM-random stream is missing: {rom_random}")
    source_start = source_start_arguments(replay_argv, binary)

    stamp = time.strftime("%Y%m%d-%H%M%S")
    session_dir = args.session_dir or PROBE_ROOT / f"probe-{args.around}-{stamp}"
    session_dir = session_dir if session_dir.is_absolute() else ROOT / session_dir

    checkpoint_frame = checkpoint_interval(frontier_mode, args.around, args.checkpoint_frame)
    checkpoint_dir: Path | None = None
    resume_dir: Path | None = None
    wanted: dict | None = None
    if use_checkpoint and checkpoint_frame > 0:
        print(
            "parity-probe: CHECKPOINT MODE IS DIAGNOSTIC ONLY; a clean result "
            "does not establish parity"
        )
        if checkpoint_is_impractically_early(
            automatic_rolling=frontier_mode and args.checkpoint_frame is None,
            checkpoint_frame=checkpoint_frame,
            target_frame=target_frame,
        ):
            print(
                f"parity-probe: checkpoint frame {checkpoint_frame} is too close to the target "
                f"{target_frame} to save safely; replaying from frame 0",
            )
        else:
            suffix = "" if core_sha == pinned_core_sha else f"-{(core_sha or 'unknown')[:8]}"
            checkpoint_dir = args.checkpoint_dir or CHECKPOINT_ROOT / f"ck-{checkpoint_frame}{suffix}"
            checkpoint_dir = checkpoint_dir if checkpoint_dir.is_absolute() else ROOT / checkpoint_dir
            wanted = checkpoint_identity(
                frame=checkpoint_frame,
                binary=binary,
                core_sha=core_sha or "",
                rom_sha=option_value(replay_argv, "--expected-rom-sha256"),
                input_path=input_path,
                rom_random_path=rom_random,
            )
            problem = checkpoint_reuse_problem(
                checkpoint_dir,
                wanted,
                trust_cross_build=args.trust_cross_build_checkpoint,
            )
            if args.checkpoint_dir is not None and problem is not None:
                raise SystemExit(
                    "parity-probe: explicitly selected checkpoint cannot be resumed: "
                    f"{problem}: {checkpoint_dir}"
                )
            if problem is None:
                if args.trust_cross_build_checkpoint:
                    print(
                        "parity-probe: TRUSTED CROSS-BUILD CHECKPOINT; this can mask earlier "
                        "state changes and is never authoritative proof"
                    )
                saved = saved_checkpoint_frame(checkpoint_dir)
                if (
                    args.checkpoint_dir is not None
                    and args.checkpoint_frame is not None
                    and saved != args.checkpoint_frame
                ):
                    raise SystemExit(
                        "parity-probe: explicitly selected checkpoint frame does not "
                        f"match its manifest ({args.checkpoint_frame} != {saved})"
                    )
                if saved is not None and saved >= args.around:
                    if args.checkpoint_dir is not None:
                        raise SystemExit(
                            "parity-probe: explicitly selected checkpoint is not before "
                            f"the probed frame ({saved} >= {args.around})"
                        )
                    print(
                        f"parity-probe: checkpoint {checkpoint_dir} is at frame {saved}, past the probed "
                        f"frame {args.around}; replaying from frame 0 (pass an earlier --checkpoint-frame)",
                    )
                    checkpoint_dir = None
                else:
                    resume_dir = checkpoint_dir
                    print(f"parity-probe: resuming from paired checkpoint {checkpoint_dir} (frame {saved})")
            else:
                print(f"parity-probe: not resuming ({problem}); replaying from 0 and re-saving the checkpoint")

    checkpoint_candidate_dir = (
        session_dir / "checkpoint-candidate"
        if should_stage_rolling_checkpoint(checkpoint_dir, resume_dir, frontier_mode)
        else None
    )
    command: list[str] = [
        str(binary),
        "--compare-snes9x-oracle",
        str(core),
        rom,
        str(target_frame),
    ]
    if core_sha:
        command += ["--expected-core-sha256", core_sha]
    rom_sha = option_value(replay_argv, "--expected-rom-sha256")
    if rom_sha:
        command += ["--expected-rom-sha256", rom_sha]
    use_replay_bundle = cold_replay_bundle_available(
        run_dir,
        replay_argv,
        target_frame=target_frame,
        input_overridden=args.input_script is not None or args.live_oracle_rng,
        resuming=resume_dir is not None,
    )
    if use_replay_bundle:
        command += ["--replay-bundle", str(run_dir)]
    else:
        command += ["--input-script", str(input_path)]
        if rom_random:
            command += ["--rom-random-script", str(rom_random)]
            if args.rom_random_script is not None:
                # The override is bound to the paired checkpoint by SHA-256 in
                # probe-identity.json. Its different parent directory is an
                # intentional diagnostic layout, not accidental source mixing.
                command.append("--allow-mixed-replay-provenance")
        if args.live_oracle_rng:
            command += [
                "--live-oracle-rng",
                "--compare-engine-state-from-frame",
                str(args.engine_state_from_frame),
            ]
    if resume_dir is not None:
        # `--load-sram` and paired resume are mutually exclusive: the resumed
        # states already carry the SRAM as it stood at the checkpoint frame.
        command += ["--resume-paired", str(resume_dir)]
    elif not use_replay_bundle:
        command += source_start
    if checkpoint_candidate_dir is not None:
        # Rolling capture rather than --save-paired-resume-at: a fixed frame can
        # land inside an unserialized ROM-call continuation. Resumed frontier
        # scans continue rolling into a session-local candidate so the next
        # trace starts near the failure instead of at the original resume point.
        command += [
            "--save-rolling-paired-resume",
            str(checkpoint_frame),
            str(checkpoint_candidate_dir),
        ]
    diagnostic_video_only = args.video_only or (frontier_mode and resume_dir is not None)
    if args.trace_only:
        command.extend(("--ignore-video", "--ignore-audio"))
    elif diagnostic_video_only:
        command.append("--ignore-audio")
    else:
        for option in (
            "--audio-comparison",
            "--audio-window-ms",
            "--audio-silence-threshold",
            "--audio-timing-tolerance-ms",
            "--audio-envelope-tolerance",
        ):
            value = option_value(replay_argv, option)
            if value is not None:
                command += [option, value]
    command += ["--session-dir", str(session_dir)]
    if not frontier_mode:
        command.append("--scan-all")
    if failure_focus is not None:
        # A failure-focused replay is diagnostic: skip thousands of already-known
        # exact comparisons and ask both renderers about only the bad pixel at the
        # frontier. The normal cold gate remains the authoritative proof lane.
        command += ["--compare-from-frame", str(max(0, args.around - 1))]
        if failure_focus.pixel is not None and core_is_instrumented:
            command += [
                "--trace-video-pixel",
                str(failure_focus.pixel[0]),
                str(failure_focus.pixel[1]),
            ]

    env = dict(os.environ)
    for name, value in script_env.items():
        env.setdefault(name, value)
    capture_env: dict[str, str] = {}
    if args.capture and core_is_instrumented:
        frames = (
            f"{args.capture_range[0]}-{args.capture_range[1]}"
            if args.capture_range is not None
            else f"{max(0, args.around - 3)}-{args.around + 3}"
        )
        capture_env["ZELDA3_CAPTURE_DISPLAY_ORACLE_FRAMES"] = frames
        capture_env["ZELDA3_CAPTURE_DISPLAY_ORACLE_BEFORE_FRAMES"] = frames
        capture_env["ZELDA3_CAPTURE_DISPLAY_CANDIDATES"] = "1"
    if failure_focus is not None and failure_focus.pixel is not None and core_is_instrumented:
        capture_env["ZELDA3_SNES9X_TRACE"] = str(session_dir / "snes9x-trace.jsonl")
        # A pixel proves what was presented; NMI and DMA events prove which
        # hardware transfer produced it. The event volume is tiny compared
        # with instruction tracing and can be converted directly into a
        # scripts/snes9x_dma_receipt.py report.
        capture_env["ZELDA3_SNES9X_TRACE_EVENTS"] = "frame,nmi,dma"
        capture_env["ZELDA3_SNES9X_TRACE_PIXEL"] = ",".join(
            str(value) for value in failure_focus.pixel
        )
    env.update(capture_env)

    prefix = " ".join(f"{name}={shlex.quote(value)}" for name, value in sorted(capture_env.items()))
    printable = f"{prefix} {shlex.join(command)}".strip()
    print(f"parity-probe: run dir {run_dir}")
    start_description = (
        (
            f"trusted cross-build diagnostic checkpoint {resume_dir}"
            if args.trust_cross_build_checkpoint
            else f"binary-hash-matched probe checkpoint {resume_dir}"
        )
        if resume_dir is not None
        else (
            f"atomic replay bundle {run_dir}"
            if use_replay_bundle
            else source_start_description(source_start)
        )
    )
    print(f"parity-probe: start mode {start_description}")
    print(f"parity-probe: session dir {session_dir}")
    print(f"parity-probe: command\n  {printable}")
    if args.dry_run:
        return 0

    removed_sessions = prune_probe_sessions(PROBE_ROOT, args.keep_probe_sessions)
    if removed_sessions:
        print(
            f"parity-probe: pruned {len(removed_sessions)} old generated probe session(s); "
            f"retaining the newest {args.keep_probe_sessions}"
        )
    session_dir.mkdir(parents=True, exist_ok=True)
    process_started_ns = time.time_ns()
    process = subprocess.run(command, cwd=ROOT, env=env, check=False)
    result_path = session_dir / "result.json"
    result = (
        json.loads(result_path.read_text(encoding="utf-8"))
        if result_path.is_file()
        else {}
    )
    checkpoint_promoted = False
    if checkpoint_candidate_dir is not None and checkpoint_candidate_dir.exists():
        authoritative_checkpoint = checkpoint_result_is_promotable(process.returncode, result)
        diagnostic_checkpoint = args.trace_only and checkpoint_result_is_diagnostic(
            process.returncode, result
        )
        if (
            checkpoint_dir is not None
            and wanted is not None
            and resume_dir is None
            and (authoritative_checkpoint or diagnostic_checkpoint)
        ):
            promoted_identity = dict(wanted)
            promoted_identity["diagnostic_only"] = not authoritative_checkpoint
            saved_frame, quarantined = promote_checkpoint_candidate(
                checkpoint_candidate_dir, checkpoint_dir, promoted_identity, stamp
            )
            checkpoint_promoted = True
            authority = "verified" if authoritative_checkpoint else "diagnostic-only"
            print(
                f"parity-probe: promoted {authority} paired checkpoint "
                f"{checkpoint_dir} (frame {saved_frame})"
            )
            if quarantined is not None:
                print(f"parity-probe: preserved replaced checkpoint at {quarantined}")
        else:
            print(
                "parity-probe: checkpoint candidate was not promoted because neither a cold "
                "exact-A/V pass nor a renderless exact-engine diagnostic pass completed; "
                f"evidence remains in {checkpoint_candidate_dir}"
            )

    report_result(session_dir)
    report_display_oracle(session_dir)
    report_display_classification(session_dir)
    if frontier_mode:
        mismatch_frame = first_video_mismatch_frame(result)
        generated_failure = newest_failure_since(process_started_ns)
        if (
            generated_failure is not None
            and mismatch_frame is not None
            and generated_failure.frame != mismatch_frame
        ):
            generated_failure = None
        if generated_failure is not None and mismatch_frame is None:
            mismatch_frame = generated_failure.frame
            print(
                f"frontier: recovered first mismatch from {generated_failure.directory} "
                "after the comparison exited without result.json"
            )
        report_frontier_provenance(session_dir, mismatch_frame)
        if (
            mismatch_frame is not None
            and not args.no_frontier_capture
            and not core_is_instrumented
        ):
            capture_dir = session_dir.with_name(f"{session_dir.name}-capture-{mismatch_frame}")
            capture_command = [
                sys.executable,
                str(Path(__file__).resolve()),
            ]
            if generated_failure is not None:
                capture_command += ["--from-failure", str(generated_failure.directory)]
            else:
                capture_command += [
                    "--around",
                    str(mismatch_frame),
                    "--window",
                    "3",
                    "--capture-range",
                    f"{max(0, mismatch_frame - 3)}-{mismatch_frame + 3}",
                ]
            capture_command += [
                "--core",
                "instrumented",
                "--video-only",
                "--binary",
                str(binary),
                "--run-dir",
                str(run_dir),
                "--input-script",
                str(input_path),
                "--session-dir",
                str(capture_dir),
            ]
            capture_checkpoint_dir = diagnostic_checkpoint_before_failure(
                checkpoint_candidate_dir, mismatch_frame
            )
            if capture_checkpoint_dir is None and checkpoint_dir is not None and (
                resume_dir is not None or checkpoint_promoted
            ):
                capture_checkpoint_dir = checkpoint_dir
            if capture_checkpoint_dir is not None:
                capture_command += ["--checkpoint-dir", str(capture_checkpoint_dir)]
            if args.trust_cross_build_checkpoint or capture_checkpoint_dir == checkpoint_candidate_dir:
                capture_command.append("--trust-cross-build-checkpoint")
            print(
                f"\nfrontier: launching trace-core candidate capture at frame {mismatch_frame}\n"
                f"  {shlex.join(capture_command)}"
            )
            capture_process = subprocess.run(capture_command, cwd=ROOT, check=False)
            if capture_process.returncode not in (0, 1):
                return capture_process.returncode
    return process.returncode


if __name__ == "__main__":
    raise SystemExit(main())
