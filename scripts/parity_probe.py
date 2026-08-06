#!/usr/bin/env python3
"""Replay one Snes9x parity divergence window from a nearby paired checkpoint.

The pre-commit gate leaves a full set of route inputs behind in
routes/<project>/comparisons/precommit/run-<target>/ (input.txt, rom-random.txt,
initial.srm, replay.sh). This tool reuses those inputs to re-run a short window
around a suspect frame, resuming from a paired Rust+oracle checkpoint saved just
before the window so the probe costs the window instead of the whole frontier.

    python3 scripts/parity_probe.py --around 17213 --capture

The first run for a given checkpoint frame replays from 0 and saves the
checkpoint; later runs resume from it. A checkpoint is only reused when the
zelda3 binary that produced it is byte-identical to the current one, because a
rebuilt binary invalidates the Rust half of the pair.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PROJECT = "routes/clean"
DEFAULT_BINARY = ROOT / "target" / "parity" / "zelda3"
PROBE_ROOT = ROOT / "target" / "parity-probes"
CHECKPOINT_ROOT = PROBE_ROOT / "checkpoints"
TRACE_CORE = ROOT / "external" / "snes9x-libretro" / "local" / "snes9x_libretro_trace.dylib"
TRACE_PATCH = ROOT / "external" / "snes9x-libretro" / "patches" / "zelda3-trace.patch"
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


def newest_source_mtime() -> tuple[float, Path | None]:
    newest = 0.0
    newest_path: Path | None = None
    for name in SOURCE_DIRS:
        for path in (ROOT / name).rglob("*.rs"):
            if "/target/" in str(path):
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


def resolve_run_dir(
    project: Path, override: Path | None, required_frame: int, binary: Path
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
    ]
    if not usable:
        raise SystemExit(
            "parity-probe: all sufficiently long runs have stale or incomplete paired starts; "
            "run the pre-commit gate with this binary or keep a cold run with --load-sram"
        )
    closest_frame = min(frame for frame, _ in usable)
    closest = [path for frame, path in usable if frame == closest_frame]
    return max(closest, key=lambda path: path.stat().st_mtime).resolve()


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


def validate_trace_core(
    core: Path,
    *,
    lock_path: Path = ORACLE_LOCK,
    patch_path: Path = TRACE_PATCH,
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

    actual_patch_sha = sha256_file(patch_path)
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
    checkpoint_dir: Path, wanted: dict[str, object]
) -> str | None:
    """Return why `checkpoint_dir` cannot be resumed, or None when it can."""
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


def saved_checkpoint_frame(checkpoint_dir: Path) -> int | None:
    latest = latest_generation(checkpoint_dir)
    return None if latest is None else latest[0]


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
    return headline, detail


def report_display_oracle(session_dir: Path) -> None:
    path = session_dir / "display_oracle.jsonl"
    if not path.is_file():
        return
    receipts = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not receipts:
        print(f"display-oracle: {path} is empty (no capture frames were reached)")
        return
    print(f"\ndisplay-oracle receipts ({len(receipts)}) from {path}:")
    detailed = 0
    for receipt in receipts:
        headline, detail = summarize_receipt(receipt)
        stage, frame = receipt["stage"], receipt["frame"]
        if not headline:
            print(f"  frame {frame} [{stage}]: all display domains exact")
            continue
        print(f"  frame {frame} [{stage}]: {', '.join(headline)}")
        if detailed < MAX_DETAIL_RECEIPTS:
            print("\n".join(detail))
        elif detailed == MAX_DETAIL_RECEIPTS:
            print("    (per-domain detail shown for the first diverging receipts only)")
        detailed += 1


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


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--around", type=int, required=True, help="suspect frame to probe")
    parser.add_argument("--window", type=int, default=40, help="frames to run past --around (default 40)")
    parser.add_argument("--project", default=DEFAULT_PROJECT, help="route project (default routes/clean)")
    parser.add_argument("--run-dir", type=Path, help="reuse this precommit run dir instead of the newest")
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--capture", action="store_true", help="write display_oracle.jsonl around --around")
    parser.add_argument(
        "--core",
        choices=("pinned", "instrumented"),
        help="core lane (default pinned, or instrumented with --capture)",
    )
    parser.add_argument("--core-path", type=Path, help="explicit core dylib (overrides --core)")
    parser.add_argument("--checkpoint-frame", type=int, help="paired checkpoint frame (default around-60)")
    parser.add_argument("--checkpoint-dir", type=Path, help="explicit paired checkpoint dir")
    parser.add_argument("--no-checkpoint", action="store_true", help="always replay from frame 0")
    parser.add_argument("--session-dir", type=Path, help="explicit session dir for this probe")
    parser.add_argument(
        "--allow-stale",
        action="store_true",
        help="skip the binary-vs-sources staleness guard for --dry-run only",
    )
    parser.add_argument("--dry-run", action="store_true", help="print the command without running it")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    validate_stale_override(args.allow_stale, args.dry_run)
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
    target_frame = args.around + max(0, args.window)
    run_dir = resolve_run_dir(project, args.run_dir, target_frame, binary)
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
    if args.capture or args.core == "instrumented":
        core_sha = validate_trace_core(core)
    else:
        core_sha = pinned_core_sha if str(core) == pinned_core else sha256_file(core)

    input_path = Path(option_value(replay_argv, "--input-script") or run_dir / "input.txt")
    rom_random_path = option_value(replay_argv, "--rom-random-script")
    rom_random = Path(rom_random_path) if rom_random_path else None
    source_start = source_start_arguments(replay_argv, binary)

    stamp = time.strftime("%Y%m%d-%H%M%S")
    session_dir = args.session_dir or PROBE_ROOT / f"probe-{args.around}-{stamp}"
    session_dir = session_dir if session_dir.is_absolute() else ROOT / session_dir

    checkpoint_frame = args.checkpoint_frame
    if checkpoint_frame is None:
        checkpoint_frame = max(0, args.around - 60)
    checkpoint_dir: Path | None = None
    resume_dir: Path | None = None
    if not args.no_checkpoint and checkpoint_frame > 0:
        if 2 * checkpoint_frame <= target_frame:
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
            problem = checkpoint_reuse_problem(checkpoint_dir, wanted)
            if problem is None:
                saved = saved_checkpoint_frame(checkpoint_dir)
                if saved is not None and saved >= args.around:
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
    command += ["--input-script", str(input_path)]
    if rom_random:
        command += ["--rom-random-script", str(rom_random)]
    if resume_dir is not None:
        # `--load-sram` and paired resume are mutually exclusive: the resumed
        # states already carry the SRAM as it stood at the checkpoint frame.
        command += ["--resume-paired", str(resume_dir)]
    else:
        command += source_start
        if checkpoint_dir is not None:
            # Rolling capture rather than --save-paired-resume-at: a fixed frame
            # can land inside an unserialized ROM-call continuation, which aborts
            # the whole run, while the rolling saver waits for the next quiescent
            # boundary at or after the interval.
            command += ["--save-rolling-paired-resume", str(checkpoint_frame), str(checkpoint_dir)]
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
    command += ["--session-dir", str(session_dir), "--scan-all"]

    env = dict(os.environ)
    for name, value in script_env.items():
        env.setdefault(name, value)
    capture_env: dict[str, str] = {}
    if args.capture:
        frames = f"{max(0, args.around - 3)}-{args.around + 3}"
        capture_env["ZELDA3_CAPTURE_DISPLAY_ORACLE_FRAMES"] = frames
        capture_env["ZELDA3_CAPTURE_DISPLAY_ORACLE_BEFORE_FRAMES"] = frames
        env.update(capture_env)

    prefix = " ".join(f"{name}={shlex.quote(value)}" for name, value in sorted(capture_env.items()))
    printable = f"{prefix} {shlex.join(command)}".strip()
    print(f"parity-probe: run dir {run_dir}")
    start_description = (
        f"binary-hash-matched probe checkpoint {resume_dir}"
        if resume_dir is not None
        else source_start_description(source_start)
    )
    print(f"parity-probe: start mode {start_description}")
    print(f"parity-probe: session dir {session_dir}")
    print(f"parity-probe: command\n  {printable}")
    if args.dry_run:
        return 0

    session_dir.mkdir(parents=True, exist_ok=True)
    process = subprocess.run(command, cwd=ROOT, env=env, check=False)
    if checkpoint_dir is not None and resume_dir is None and (checkpoint_dir / "latest.json").is_file():
        wanted["saved_frame"] = saved_checkpoint_frame(checkpoint_dir)
        (checkpoint_dir / IDENTITY_NAME).write_text(
            json.dumps(wanted, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"parity-probe: saved paired checkpoint {checkpoint_dir} (frame {wanted['saved_frame']})")

    report_result(session_dir)
    report_display_oracle(session_dir)
    return process.returncode


if __name__ == "__main__":
    raise SystemExit(main())
