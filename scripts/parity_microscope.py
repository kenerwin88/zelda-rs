#!/usr/bin/env python3
"""One provenance-safe entry point for the Zelda3 Snes9x parity frontier."""

from __future__ import annotations

import argparse
import bisect
import fcntl
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))

import parity_evidence as evidence  # noqa: E402
import parity_probe  # noqa: E402
import compare_snes9x_cpu_checkpoints as cpu_checkpoints  # noqa: E402
from extract_snes9x_rom_random import extract_samples, write_script  # noqa: E402


DEFAULT_PROJECT = ROOT / "routes" / "full_run"
DEFAULT_BINARY = ROOT / "target" / "parity" / "zelda3"
DEFAULT_ZPARITY = ROOT / "target" / "parity" / "zparity"
DEFAULT_STATE = ROOT / ".git" / "precommit-snes9x-parity-state.json"
DEFAULT_OUTPUT = ROOT / "target" / "parity-microscope"
DEFAULT_SYMBOLS = Path(
    os.environ.get("ZELDA3_ROM_SYMBOLS", "/Users/missingno/Documents/zelda3/other/names.txt")
)
SYMBOL_RE = re.compile(r"^0x([0-9a-fA-F]{4,6}):\s*(\S+)")
PC_RE = re.compile(r"^(?:\$|0x)?([0-9a-fA-F]{2})[:_]([0-9a-fA-F]{4})$")
WRAM_RE = re.compile(r"^(?:0x)?[0-9a-fA-F]{1,5}(?:-(?:0x)?[0-9a-fA-F]{1,5})?$")
FRAME_RANGE_RE = re.compile(r"^(\d+)-(\d+)$")
TRACE_EVENTS = frozenset({"frame", "nmi", "nmi-resume", "dma", "hdma", "ppu", "wram", "pc"})


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def read_local_state(path: Path = DEFAULT_STATE) -> dict[str, Any]:
    return evidence.load_json(path) if path.is_file() else {}


def newest_rust_evidence_source() -> tuple[float, Path]:
    sources = [ROOT / "crates" / "parity" / "Cargo.toml"]
    sources.extend((ROOT / "crates" / "parity" / "src").rglob("*.rs"))
    newest = max(sources, key=lambda path: path.stat().st_mtime)
    return newest.stat().st_mtime, newest


def rust_evidence_binary(
    *, required: bool = True, require_fresh: bool = True
) -> Path | None:
    candidates = [
        Path(os.environ["ZELDA3_ZPARITY"]) if "ZELDA3_ZPARITY" in os.environ else None,
        DEFAULT_ZPARITY,
        ROOT / "target" / "debug" / "zparity",
    ]
    stale: Path | None = None
    newest_mtime, newest_source = newest_rust_evidence_source()
    for candidate in candidates:
        if candidate is not None and candidate.is_file():
            if require_fresh and candidate.stat().st_mtime < newest_mtime:
                stale = candidate
                continue
            return candidate.resolve()
    if required:
        if stale is not None:
            raise SystemExit(
                f"parity microscope: Rust evidence engine is stale ({stale}); "
                f"newer source {newest_source.relative_to(ROOT)}; "
                "run cargo build --profile parity -p parity"
            )
        raise SystemExit(
            "parity microscope: Rust evidence engine is missing; "
            "run cargo build --profile parity -p parity"
        )
    return None


def build_rust_trace_index(
    session: Path, *, output: Path | None = None, required: bool = True
) -> tuple[Path, subprocess.CompletedProcess[str]] | None:
    engine = rust_evidence_binary(required=required)
    if engine is None:
        return None
    session = session.resolve()
    trace = session / "snes9x-trace.jsonl"
    manifest = session / "manifest.json"
    if not trace.is_file() or not manifest.is_file():
        raise SystemExit(
            f"parity microscope: {session} needs snes9x-trace.jsonl and manifest.json"
        )
    index = (output or session / "snes9x-trace.zpti").resolve()
    process = subprocess.run(
        [
            str(engine),
            "trace-index",
            str(trace),
            "--manifest",
            str(manifest),
            "--output",
            str(index),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    return index, process


def newest_result(project: Path) -> tuple[Path, dict[str, Any]] | None:
    results = list((project / "comparisons" / "precommit").glob("run-*/result.json"))
    if not results:
        return None
    path = max(results, key=lambda candidate: candidate.stat().st_mtime_ns)
    return path, evidence.load_json(path)


def active_change_scope() -> dict[str, Any]:
    names = set(evidence.git_output("diff", "--name-only", "HEAD").splitlines())
    names.update(
        evidence.git_output("ls-files", "--others", "--exclude-standard").splitlines()
    )
    paths = [Path(line) for line in sorted(names) if line]
    production = [
        path
        for path in paths
        if (
            (path.parts[:3] == ("crates", "zelda3", "src") or path.parts[:2] == ("zelda3-bin", "src"))
            and "tests" not in path.parts
            and not path.name.endswith("_tests.rs")
        )
    ]
    return {
        "changed_files": len(paths),
        "production_files": len(production),
        "production_paths": [str(path) for path in production],
        "within_one_root_cause_budget": len(production) <= 5,
    }


def default_frontier(state: dict[str, Any]) -> int:
    return max(1, int(state.get("last_checked_frame", 0)) + 1)


def canonical_lorom_pc(pc: int) -> int:
    bank = (pc >> 16) & 0xFF
    address = pc & 0xFFFF
    if address >= 0x8000:
        bank |= 0x80
    return (bank << 16) | address


def lorom_pc_aliases(pc: int) -> tuple[int, ...]:
    address = pc & 0xFFFF
    if address < 0x8000:
        return (pc,)
    bank = (pc >> 16) & 0xFF
    low = ((bank & 0x7F) << 16) | address
    high = ((bank | 0x80) << 16) | address
    return (low, high)


def format_pc(pc: int) -> str:
    return f"{(pc >> 16) & 0xff:02x}:{pc & 0xffff:04x}"


class SymbolTable:
    def __init__(self, path: Path = DEFAULT_SYMBOLS):
        self.path = path
        self.by_name: dict[str, int] = {}
        self.by_address: dict[int, str] = {}
        if path.is_file():
            for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
                match = SYMBOL_RE.match(line)
                if match is None:
                    continue
                address = canonical_lorom_pc(int(match.group(1), 16))
                name = match.group(2)
                self.by_name[name.lower()] = address
                self.by_address.setdefault(address, name)
        self.addresses = sorted(self.by_address)

    def parse(self, value: str) -> int:
        symbol = self.by_name.get(value.lower())
        if symbol is not None:
            return symbol
        match = PC_RE.fullmatch(value)
        if match is not None:
            return (int(match.group(1), 16) << 16) | int(match.group(2), 16)
        try:
            parsed = int(value, 0)
        except ValueError as error:
            location = f" in {self.path}" if self.path.is_file() else ""
            raise SystemExit(f"parity microscope: unknown PC or symbol {value!r}{location}") from error
        if not 0 <= parsed <= 0xFFFFFF:
            raise SystemExit(f"parity microscope: PC is outside the 24-bit address space: {value}")
        return parsed

    def trace_filter(self, values: list[str]) -> str:
        addresses: set[int] = set()
        for value in values:
            addresses.update(lorom_pc_aliases(self.parse(value)))
        return ",".join(format_pc(address) for address in sorted(addresses))

    def describe(self, pc: int) -> str:
        pc = canonical_lorom_pc(pc)
        exact = self.by_address.get(pc)
        if exact is not None:
            return exact
        index = bisect.bisect_right(self.addresses, pc) - 1
        if index < 0:
            return "unknown"
        start = self.addresses[index]
        if start >> 16 != pc >> 16 or pc - start > 0x2000:
            return "unknown"
        return f"{self.by_address[start]}+0x{pc - start:x}"


def validate_wram_filters(values: list[str]) -> str:
    invalid = [value for value in values if WRAM_RE.fullmatch(value) is None]
    if invalid:
        raise SystemExit(
            "parity microscope: invalid WRAM filter(s): " + ", ".join(invalid)
        )
    return ",".join(value.lower().removeprefix("0x") for value in values)


def parse_internal_frame_range(value: str) -> str:
    match = FRAME_RANGE_RE.fullmatch(value)
    if match is None or int(match.group(1)) > int(match.group(2)):
        raise argparse.ArgumentTypeError("expected ordered FIRST-LAST internal frames")
    return value


def parse_events(value: str, *, has_pc: bool, has_wram: bool) -> str:
    events = {item.strip() for item in value.split(",") if item.strip()}
    unknown = events - TRACE_EVENTS
    if unknown:
        raise SystemExit("parity microscope: unknown trace events: " + ", ".join(sorted(unknown)))
    if "pc" in events and not has_pc:
        raise SystemExit("parity microscope: refusing unfiltered PC tracing; pass at least one --pc")
    if "wram" in events and not has_wram:
        raise SystemExit("parity microscope: refusing unfiltered WRAM tracing; pass at least one --wram")
    if has_pc:
        events.add("pc")
    if has_wram:
        events.add("wram")
    events.update(("frame", "nmi", "nmi-resume"))
    return ",".join(sorted(events))


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    try:
        with path.open(encoding="utf-8") as stream:
            for line_number, line in enumerate(stream, 1):
                try:
                    value = json.loads(line)
                except json.JSONDecodeError as error:
                    raise SystemExit(
                        f"parity microscope: invalid JSONL {path}:{line_number}: {error}"
                    ) from error
                if not isinstance(value, dict):
                    raise SystemExit(
                        f"parity microscope: non-object JSONL record {path}:{line_number}"
                    )
                records.append(value)
    except OSError as error:
        raise SystemExit(f"parity microscope: cannot read {path}: {error}") from error
    return records


def cpu_checkpoint_correlation(session: Path) -> dict[str, Any]:
    rust_trace = session / "rust-cpu-checkpoints.jsonl"
    oracle_trace = session / "snes9x-trace.jsonl"
    manifest = session / "manifest.json"
    if not rust_trace.is_file() or rust_trace.stat().st_size == 0:
        return {"status": "not-observed", "records": 0}
    records = [
        record
        for record in read_jsonl(rust_trace)
        if record.get("event") == "rust-cpu-checkpoint"
    ]
    if not records:
        return {"status": "not-observed", "records": 0}
    manifest_value = evidence.load_json(manifest)
    timing = (
        manifest_value.get("timing")
        if isinstance(manifest_value.get("timing"), dict)
        else {}
    )
    start_frame = int(timing.get("start_frame", 0))
    legacy_resumed = next(
        (
            record
            for record in records
            if record.get("schema") == 1
            and record.get("coordinate") == cpu_checkpoints.LEGACY_RUST_COORDINATE
            and start_frame != 0
        ),
        None,
    )
    if legacy_resumed is not None:
        return {
            "status": "invalid",
            "records": len(records),
            "problem": (
                "legacy Rust checkpoint falsely claims a window-relative run during "
                f"checkpoint resume (manifest start_frame={start_frame}, "
                f"run={legacy_resumed.get('run')}); rebuild the parity binary so it "
                "emits schema-2 absolute host frames"
            ),
        }
    oracle_runs = {
        int(record["run"])
        for record in read_jsonl(oracle_trace)
        if isinstance(record.get("run"), int)
    }
    if not oracle_runs:
        return {
            "status": "invalid",
            "records": len(records),
            "problem": "oracle trace has no retro_run coordinates",
        }
    first_host_frame = start_frame + min(oracle_runs)
    last_host_frame = start_frame + max(oracle_runs)
    selected_records = [
        record
        for record in records
        if (
            isinstance(record.get("host_frame"), int)
            and first_host_frame <= int(record["host_frame"]) <= last_host_frame
        )
        or (
            record.get("schema") == 1
            and isinstance(record.get("run"), int)
            and first_host_frame <= int(record["run"]) <= last_host_frame
        )
    ]
    if not selected_records:
        return {
            "status": "not-observed",
            "records": len(records),
            "captured_host_range": [first_host_frame, last_host_frame],
        }
    pcs = {record.get("pc") for record in records if isinstance(record.get("pc"), int)}
    if len(pcs) != 1:
        return {
            "status": "invalid",
            "records": len(records),
            "problem": "Rust CPU checkpoint trace contains zero or multiple checkpoint PCs",
        }
    checkpoint_pc = int(next(iter(pcs)))
    oracle_checkpoint_runs = {
        int(record["run"])
        for record in read_jsonl(oracle_trace)
        if record.get("event") == "pc"
        and isinstance(record.get("pc"), int)
        and cpu_checkpoints.canonical_lorom_pc(int(record["pc"]))
        == cpu_checkpoints.canonical_lorom_pc(checkpoint_pc)
        and isinstance(record.get("run"), int)
    }
    matching_host_frames = sorted(
        {
            int(record.get("host_frame", record.get("run")))
            for record in selected_records
            if int(record.get("host_frame", record.get("run"))) - start_frame
            in oracle_checkpoint_runs
        }
    )
    unmatched_host_frames = sorted(
        {
            int(record.get("host_frame", record.get("run")))
            for record in selected_records
        }
        - set(matching_host_frames)
    )
    if not matching_host_frames:
        return {
            "status": "not-observed",
            "records": len(selected_records),
            "checkpoint_pc": f"0x{checkpoint_pc:06x}",
            "captured_host_range": [first_host_frame, last_host_frame],
            "rust_only_host_frames": unmatched_host_frames,
        }
    reports: list[dict[str, Any]] = []
    for host_frame in matching_host_frames:
        try:
            reports.append(
                cpu_checkpoints.compare(
                    oracle_trace,
                    rust_trace,
                    manifest,
                    checkpoint_pc,
                    host_frame,
                    host_frame,
                )
            )
        except SystemExit as error:
            return {
                "status": "invalid",
                "records": len(selected_records),
                "checkpoint_pc": f"0x{checkpoint_pc:06x}",
                "problem": str(error),
            }
    report = dict(reports[0])
    report["comparisons"] = [
        comparison
        for item in reports
        for comparison in item["comparisons"]
    ]
    report["rust_only_host_frames"] = unmatched_host_frames
    output = session / "cpu-checkpoint-correlation.json"
    write_json(output, report)
    deltas = [
        int(item["oracle_minus_rust_master_cycles"])
        for item in report["comparisons"]
    ]
    return {
        "status": "compared",
        "records": len(selected_records),
        "checkpoint_pc": f"0x{checkpoint_pc:06x}",
        "path": str(output),
        "host_frames": [
            int(item["host_frame"]) for item in report["comparisons"]
        ],
        "oracle_minus_rust_master_cycles": deltas,
        "rust_only_host_frames": unmatched_host_frames,
    }


def automatic_trace_frame_range(
    *,
    project: Path,
    frontier: int,
    tail_frames: int,
    checkpoint_dir: Path | None = None,
) -> str | None:
    if tail_frames <= 0:
        return None
    checkpoint = checkpoint_dir or parity_probe.default_frontier_checkpoint_dir(project)
    saved = parity_probe.saved_checkpoint_frame(checkpoint)
    if saved is None or saved >= frontier:
        return None
    distance = frontier - saved
    return f"{max(0, distance - tail_frames)}-{distance}"


def resolve_microscope_checkpoint(
    args: argparse.Namespace, *, project: Path
) -> tuple[Path | None, int | None]:
    """Resolve an explicit diagnostic checkpoint without any cold fallback.

    `--state` is the precommit frontier JSON, not a savestate.  Checkpoints have
    their own option so the selected generation and its frame cannot be silently
    ignored or inferred from a different project-scoped cache.
    """
    checkpoint_dir = (
        args.checkpoint_dir.resolve() if args.checkpoint_dir is not None else None
    )
    checkpoint_frame = args.checkpoint_frame
    explicit_checkpoint = checkpoint_dir is not None or checkpoint_frame is not None
    if args.cold and (explicit_checkpoint or args.trust_cross_build_checkpoint):
        raise SystemExit(
            "parity microscope: --cold cannot be combined with checkpoint options"
        )
    if args.trust_cross_build_checkpoint and checkpoint_dir is None:
        raise SystemExit(
            "parity microscope: --trust-cross-build-checkpoint requires --checkpoint-dir"
        )
    if checkpoint_dir is None:
        return None, checkpoint_frame
    saved_frame = parity_probe.saved_checkpoint_frame(checkpoint_dir)
    if saved_frame is None:
        raise SystemExit(
            f"parity microscope: --checkpoint-dir has no valid paired checkpoint: {checkpoint_dir}"
        )
    if checkpoint_frame is not None and checkpoint_frame != saved_frame:
        raise SystemExit(
            "parity microscope: --checkpoint-frame does not match the selected paired "
            f"checkpoint ({checkpoint_frame} != {saved_frame})"
        )
    return checkpoint_dir, saved_frame


def microscope_checkpoint_probe_args(
    checkpoint_dir: Path | None,
    checkpoint_frame: int | None,
    *,
    trust_cross_build: bool,
) -> list[str]:
    options: list[str] = []
    if checkpoint_frame is not None:
        options += ["--checkpoint-frame", str(checkpoint_frame)]
    if checkpoint_dir is not None:
        options += ["--checkpoint-dir", str(checkpoint_dir)]
    if trust_cross_build:
        options.append("--trust-cross-build-checkpoint")
    return options


def cached_diagnostic_rng(project: Path) -> Path | None:
    checkpoint = parity_probe.default_frontier_checkpoint_dir(project)
    script = checkpoint / "rom-random.txt"
    identity_path = checkpoint / parity_probe.IDENTITY_NAME
    if not script.is_file() or not identity_path.is_file():
        return None
    identity = evidence.load_json(identity_path)
    return script if identity.get("rom_random_sha256") == evidence.sha256_file(script) else None


def materialize_source_session_rng(source: Path, output: Path) -> tuple[Path, int] | None:
    """Resolve RNG only from artifacts cryptographically bound to `source`.

    A selected run may carry either a replay script or a live-oracle trace.  In
    the latter case we turn the trace into a stable, session-local script so a
    later replay cannot silently borrow the ledger from an unrelated checkpoint.
    """
    manifest = evidence.load_json(source / "manifest.json")
    replay = source / "rom-random.txt"
    replay_record = manifest.get("rom_random_replay")
    if replay.is_file():
        expected = replay_record.get("sha256") if isinstance(replay_record, dict) else None
        actual = evidence.sha256_file(replay)
        if expected != actual:
            raise SystemExit(
                f"parity microscope: source RNG replay is not manifest-bound: {replay}"
            )
        return replay, sum(
            1
            for line in replay.read_text(encoding="utf-8").splitlines()
            if line and not line.startswith("#")
        )

    authority = manifest.get("rom_random_authority")
    mode = authority.get("mode") if isinstance(authority, dict) else None
    artifact = authority.get("artifact") if isinstance(authority, dict) else None
    if mode != "live_oracle_trace":
        return None
    if not isinstance(artifact, str):
        raise SystemExit("parity microscope: live oracle RNG authority has no artifact")
    trace = source / artifact
    if not trace.is_file():
        raise SystemExit(f"parity microscope: source RNG trace is missing: {trace}")
    with trace.open(encoding="utf-8") as stream:
        samples = extract_samples(stream)
    if not samples:
        raise SystemExit(f"parity microscope: source RNG trace has no cartridge samples: {trace}")
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8") as stream:
        write_script(samples, stream)
    return output, len(samples)


def bind_live_rng_to_diagnostic_checkpoint(
    bootstrap_session: Path, project: Path
) -> tuple[Path, int]:
    bootstrap_identity = evidence.session_identity(bootstrap_session)
    if (
        bootstrap_identity["status"] != "passed"
        or not bootstrap_identity["parity_eligible"]
        or not isinstance(bootstrap_identity["video"], dict)
        or bootstrap_identity["video"].get("matched") is not True
    ):
        raise SystemExit(
            "parity microscope: refusing RNG materialization from an incomplete bootstrap"
        )
    trace = bootstrap_session / "oracle-rom-random.jsonl"
    if not trace.is_file():
        raise SystemExit(
            f"parity microscope: live-RNG bootstrap produced no oracle trace: {trace}"
        )
    with trace.open(encoding="utf-8") as stream:
        samples = extract_samples(stream)
    if not samples:
        raise SystemExit("parity microscope: live-RNG bootstrap produced no cartridge RNG samples")
    checkpoint = parity_probe.default_frontier_checkpoint_dir(project)
    identity_path = checkpoint / parity_probe.IDENTITY_NAME
    if not identity_path.is_file():
        raise SystemExit("parity microscope: diagnostic checkpoint has no identity sidecar")
    script = checkpoint / "rom-random.txt"
    temporary = checkpoint / ".rom-random.txt.tmp"
    with temporary.open("w", encoding="utf-8") as output:
        write_script(samples, output)
    os.replace(temporary, script)
    identity = evidence.load_json(identity_path)
    identity["rom_random_sha256"] = evidence.sha256_file(script)
    identity["rom_random_authority"] = "materialized from the checkpoint bootstrap's live oracle run"
    evidence.atomic_write_json(identity_path, identity)
    return script, len(samples)


def run_dir_context(
    *,
    project: Path,
    binary: Path,
    frontier: int,
    override: Path | None,
    require_recorded_rom_random: bool,
) -> dict[str, Any]:
    run_dir = parity_probe.resolve_run_dir(
        project,
        override,
        frontier,
        binary,
        require_recorded_rom_random=require_recorded_rom_random,
    )
    _, replay = parity_probe.parse_replay_script(run_dir / "replay.sh")
    core_sha = parity_probe.option_value(replay, "--expected-core-sha256")
    rom_sha = parity_probe.option_value(replay, "--expected-rom-sha256")
    state = read_local_state()
    route_signature = state.get("route_signature")
    if isinstance(route_signature, dict):
        if require_recorded_rom_random and route_signature.get("core_sha256") != core_sha:
            raise SystemExit(
                "parity microscope: selected replay core disagrees with the current route signature"
            )
        if route_signature.get("rom_sha256") != rom_sha:
            raise SystemExit(
                "parity microscope: selected replay ROM disagrees with the current route signature"
            )
    identity = evidence.session_identity(run_dir)
    if identity["core_sha256"] != core_sha or identity["rom_sha256"] != rom_sha:
        raise SystemExit("parity microscope: selected replay manifest disagrees with replay.sh")
    return {
        "run_dir": str(run_dir),
        "run_identity": identity,
        "route_signature": route_signature,
        "binary": {
            "path": str(binary.resolve()),
            "sha256": evidence.sha256_file(binary),
            "size": binary.stat().st_size,
        },
        "git": evidence.git_identity(),
    }


def _event_line(event: dict[str, Any], host_frame: int, symbols: SymbolTable) -> str:
    kind = str(event.get("event", "?"))
    pc = int(event.get("pc", 0))
    raster = f"{event.get('v', '?')}:{event.get('cycles', '?')}"
    module = (
        f"{int(event.get('main', 0)):02x}/"
        f"{int(event.get('sub', 0)):02x}/"
        f"{int(event.get('subsub', 0)):02x}"
    )
    detail = ""
    if kind == "dma":
        detail = (
            f" ch={event.get('channel')} src={int(event.get('source', 0)):06x}"
            f" bytes={event.get('bytes')} b={int(event.get('b_address', 0)):02x}"
        )
    elif kind == "wram":
        detail = (
            f" addr={int(event.get('address', 0)):05x}"
            f" value={int(event.get('value', 0)):02x}"
        )
    return (
        f"host={host_frame} run={event.get('run')} internal={event.get('frame')} "
        f"{kind:<10} raster={raster:<9} pc={format_pc(pc)} "
        f"{symbols.describe(pc)} module={module}{detail}"
    )


def build_timeline(
    trace: Path,
    *,
    start_frame: int,
    symbols: SymbolTable,
    internal_frame_filter: str | None = None,
) -> tuple[dict[str, Any], str]:
    by_run: dict[int, list[dict[str, Any]]] = defaultdict(list)
    with trace.open(encoding="utf-8") as stream:
        for line_number, line in enumerate(stream, 1):
            try:
                event = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(f"parity microscope: invalid trace {trace}:{line_number}: {error}") from error
            run = event.get("run")
            if not isinstance(run, int):
                raise SystemExit(f"parity microscope: trace event has no retro_run at line {line_number}")
            by_run[run].append(event)
    if not by_run:
        raise SystemExit(f"parity microscope: trace has no events: {trace}")

    rendered: list[str] = []
    runs: list[dict[str, Any]] = []
    incomplete: list[int] = []
    clipped_leading_runs: list[int] = []
    filter_start = (
        int(internal_frame_filter.split("-", 1)[0])
        if internal_frame_filter is not None
        else None
    )
    for run in sorted(by_run):
        events = by_run[run]
        host_frame = start_frame + run
        frame_events = [event for event in events if event.get("event") == "frame"]
        entries = [event for event in frame_events if event.get("stage") == "entry"]
        returns = [event for event in frame_events if event.get("stage") == "return"]
        internal_frames = sorted(
            {int(event["frame"]) for event in events if isinstance(event.get("frame"), int)}
        )
        clipped_leading = (
            run == min(by_run)
            and filter_start is not None
            and len(entries) == 0
            and len(returns) == 1
            and internal_frames
            and internal_frames[0] == filter_start
        )
        if clipped_leading:
            clipped_leading_runs.append(run)
        elif len(entries) != 1 or len(returns) != 1:
            incomplete.append(run)
        rendered.extend(_event_line(event, host_frame, symbols) for event in events)
        runs.append(
            {
                "run": run,
                "host_frame": host_frame,
                "internal_frames": internal_frames,
                "entry_events": len(entries),
                "return_events": len(returns),
                "events": len(events),
                "clipped_at_internal_filter_start": clipped_leading,
            }
        )
    report = {
        "schema": 1,
        "coordinate": "host_frame = comparison_start_frame + zero-based libretro retro_run",
        "comparison_start_frame": start_frame,
        "trace_sha256": evidence.sha256_file(trace),
        "internal_frame_filter": internal_frame_filter,
        "runs": runs,
        "incomplete_or_ambiguous_runs": incomplete,
        "clipped_leading_runs": clipped_leading_runs,
    }
    if incomplete:
        rendered.append(
            "WARNING incomplete/ambiguous retro_run frame boundaries: "
            + ", ".join(map(str, incomplete))
        )
    if clipped_leading_runs:
        rendered.append(
            "NOTE leading retro_run clipped at the explicit internal-frame filter: "
            + ", ".join(map(str, clipped_leading_runs))
        )
    return report, "\n".join(rendered) + "\n"


def first_cause(result: dict[str, Any]) -> dict[str, Any]:
    engine = result.get("engine_state")
    video = result.get("video")
    audio = result.get("audio")
    candidates: list[tuple[int, str, object]] = []
    for lane, value in (("engine_state", engine), ("video", video), ("audio", audio)):
        if not isinstance(value, dict) or value.get("first_mismatch") is None:
            continue
        mismatch = value.get("first_mismatch")
        frame = mismatch.get("frame") if isinstance(mismatch, dict) else None
        if not isinstance(frame, int):
            frame = int(result.get("frames_completed", 0))
        candidates.append((frame, lane, mismatch))
    if not candidates:
        return {"classification": "no-reported-lane-mismatch"}
    frame, lane, mismatch = min(candidates, key=lambda item: item[0])
    return {"classification": lane, "frame": frame, "receipt": mismatch}


def differing_byte_ranges(
    left: bytes, right: bytes, *, limit: int = 16
) -> tuple[int, list[dict[str, int]]]:
    if len(left) != len(right):
        raise SystemExit(
            f"parity microscope: buffer sizes differ ({len(left)} != {len(right)})"
        )
    ranges: list[dict[str, int]] = []
    count = 0
    start: int | None = None
    for index, (left_byte, right_byte) in enumerate(zip(left, right, strict=True)):
        differs = left_byte != right_byte
        count += int(differs)
        if differs and start is None:
            start = index
        elif not differs and start is not None:
            if len(ranges) < limit:
                ranges.append({"start_byte": start, "end_byte_exclusive": index})
            start = None
    if start is not None and len(ranges) < limit:
        ranges.append({"start_byte": start, "end_byte_exclusive": len(left)})
    return count, ranges


def build_frame_explanation(
    failure: Path, *, trace: Path | None = None, resume_frame: int = 0
) -> dict[str, Any]:
    failure = parity_probe.resolve_failure_dir(failure)
    receipt = evidence.load_json(failure / "diff.json")
    try:
        frame = int(receipt["frame"])
    except (KeyError, TypeError, ValueError) as error:
        raise SystemExit(f"parity microscope: invalid failure frame in {failure}") from error

    paths = {
        "oracle_scanout": failure / "oracle_before_vram.bin",
        "oracle_live": failure / "oracle_after_vram.bin",
        "rust_visible": failure / "rust_visible_vram.bin",
        "rust_live": failure / "rust_after_vram.bin",
    }
    missing = [str(path) for path in paths.values() if not path.is_file()]
    if missing:
        raise SystemExit(
            "parity microscope: failure is missing VRAM generation artifacts: "
            + ", ".join(missing)
        )
    buffers = {name: path.read_bytes() for name, path in paths.items()}
    hashes = {name: evidence.sha256_file(path) for name, path in paths.items()}
    identities = {
        rust_name: [
            oracle_name
            for oracle_name in ("oracle_scanout", "oracle_live")
            if buffers[rust_name] == buffers[oracle_name]
        ]
        for rust_name in ("rust_visible", "rust_live")
    }
    visible_diff_count, visible_ranges = differing_byte_ranges(
        buffers["rust_visible"], buffers["oracle_scanout"]
    )
    live_diff_count, live_ranges = differing_byte_ranges(
        buffers["rust_live"], buffers["oracle_live"]
    )
    oracle_changed = buffers["oracle_scanout"] != buffers["oracle_live"]
    if identities["rust_live"] != ["oracle_live"]:
        classification = "live-vram-divergence"
    elif identities["rust_visible"] == ["oracle_scanout"]:
        classification = "correct-scanout-generation"
    elif oracle_changed and identities["rust_visible"] == ["oracle_live"]:
        classification = "post-frame-vram-presented-early"
    else:
        classification = "unmatched-scanout-generation"

    report: dict[str, Any] = {
        "schema": 1,
        "failure": str(failure),
        "host_frame": frame,
        "classification": classification,
        "buffers": {
            name: {"path": str(paths[name]), "bytes": len(value), "sha256": hashes[name]}
            for name, value in buffers.items()
        },
        "exact_generation_matches": identities,
        "oracle_changed_during_frame": oracle_changed,
        "visible_vs_oracle_scanout": {
            "mismatched_bytes": visible_diff_count,
            "first_ranges": visible_ranges,
        },
        "live_vs_oracle_live": {
            "mismatched_bytes": live_diff_count,
            "first_ranges": live_ranges,
        },
    }

    if trace is not None:
        trace = trace.resolve()
        if not trace.is_file():
            raise SystemExit(f"parity microscope: missing oracle trace: {trace}")
        run = frame - resume_frame
        if run < 0:
            raise SystemExit(
                f"parity microscope: host frame {frame} precedes resume frame {resume_frame}"
            )
        events = []
        with trace.open(encoding="utf-8") as stream:
            for line_number, line in enumerate(stream, 1):
                try:
                    event = json.loads(line)
                except json.JSONDecodeError as error:
                    raise SystemExit(f"{trace}:{line_number}: invalid JSON: {error}") from error
                if event.get("run") == run:
                    events.append(event)
        entries = [
            event
            for event in events
            if event.get("event") == "frame" and event.get("stage") == "entry"
        ]
        returns = [
            event
            for event in events
            if event.get("event") == "frame" and event.get("stage") == "return"
        ]
        if len(entries) != 1 or len(returns) != 1:
            raise SystemExit(
                f"parity microscope: trace run {run} is missing or ambiguous "
                f"(entries={len(entries)} returns={len(returns)})"
            )
        first_mismatch_byte = (
            visible_ranges[0]["start_byte"] if visible_ranges else None
        )
        first_mismatch_word = (
            first_mismatch_byte // 2 if first_mismatch_byte is not None else None
        )
        dmas = []
        for event in events:
            if event.get("event") != "dma":
                continue
            b_address = int(event.get("b_address", -1))
            vram_address = (
                int(event.get("vram_address", 0)) if b_address in (0x18, 0x19) else None
            )
            dmas.append(
                {
                    "channel": event.get("channel"),
                    "raster": [event.get("v"), event.get("cycles")],
                    "pc": int(event.get("pc", 0)),
                    "source": int(event.get("source", 0)),
                    "bytes": int(event.get("bytes", 0)),
                    "mode": int(event.get("mode", 0)),
                    "b_address": b_address,
                    "vram_address": vram_address,
                    "starts_at_first_visible_mismatch_word": (
                        vram_address is not None and vram_address == first_mismatch_word
                    ),
                }
            )
        report["oracle_trace"] = {
            "path": str(trace),
            "sha256": evidence.sha256_file(trace),
            "resume_host_frame": resume_frame,
            "run": run,
            "internal_frames": [entries[0].get("frame"), returns[0].get("frame")],
            "first_visible_mismatch_byte": first_mismatch_byte,
            "first_visible_mismatch_word": first_mismatch_word,
            "dma": dmas,
        }
    return report


def render_frame_explanation(report: dict[str, Any]) -> str:
    lines = [
        f"frame {report['host_frame']}: {report['classification']}",
        "exact generation matches:",
    ]
    for rust_name, matches in report["exact_generation_matches"].items():
        lines.append(f"  {rust_name}: {', '.join(matches) if matches else '(none)'}")
    visible = report["visible_vs_oracle_scanout"]
    live = report["live_vs_oracle_live"]
    lines.append(
        f"visible mismatch: {visible['mismatched_bytes']} byte(s); "
        f"first ranges={visible['first_ranges']}"
    )
    lines.append(f"live mismatch: {live['mismatched_bytes']} byte(s)")
    trace = report.get("oracle_trace")
    if isinstance(trace, dict):
        lines.append(
            f"oracle coordinates: host={report['host_frame']} resume={trace['resume_host_frame']} "
            f"run={trace['run']} internal={trace['internal_frames']}"
        )
        matching = [
            dma for dma in trace["dma"] if dma["starts_at_first_visible_mismatch_word"]
        ]
        lines.append(f"DMA transfers in run: {len(trace['dma'])}")
        for dma in matching:
            lines.append(
                "  exact first-mismatch start: "
                f"pc=${dma['pc'] >> 16:02x}:{dma['pc'] & 0xffff:04x} "
                f"raster={dma['raster'][0]}:{dma['raster'][1]} "
                f"src=${dma['source'] >> 16:02x}:{dma['source'] & 0xffff:04x} "
                f"bytes={dma['bytes']} mode={dma['mode']} "
                f"dst_word=${dma['vram_address']:04x}"
            )
    return "\n".join(lines) + "\n"


def command_explain_frame(args: argparse.Namespace) -> int:
    report = build_frame_explanation(
        args.failure, trace=args.trace, resume_frame=args.resume_frame
    )
    if args.output is not None:
        write_json(args.output, report)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_frame_explanation(report), end="")
    return 0


def run_with_trace_limit(
    command: list[str],
    *,
    environment: dict[str, str],
    trace: Path,
    max_bytes: int,
) -> int:
    process = subprocess.Popen(command, cwd=ROOT, env=environment)
    while process.poll() is None:
        if trace.is_file() and trace.stat().st_size > max_bytes:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            print(
                f"parity microscope: stopped trace at {trace.stat().st_size / (1024 * 1024):.1f} MiB; "
                "narrow --pc/--wram or add --trace-internal-frames",
                file=sys.stderr,
            )
            return 75
        time.sleep(0.1)
    return int(process.returncode)


def command_status(args: argparse.Namespace) -> int:
    state = read_local_state(args.state)
    ledger = evidence.load_json(args.ledger)
    print(f"route: {ledger.get('project')}")
    promoted = ledger.get("promoted")
    if isinstance(promoted, dict):
        print(
            "durable promoted frontier: "
            f"{promoted.get('last_exact_video_frame')} at {promoted.get('commit')}"
        )
    else:
        print("durable promoted frontier: none")
    legacy = ledger.get("legacy_local_ratchet")
    if isinstance(legacy, dict):
        print(f"legacy local ratchet: {legacy.get('frame')} (not commit-bound)")
    print(f"local ratchet: {state.get('last_checked_frame', 0)}")
    newest = newest_result(args.project)
    if newest is not None:
        path, result = newest
        cause = first_cause(result)
        print(
            f"newest experiment: {path.parent.name} status={result.get('status')} "
            f"completed={result.get('frames_completed')} first_cause={cause.get('classification')}"
        )
    passes = evidence.load_cold_passes(args.pass_root)
    print(f"cold promotion receipts: {len(passes)}")
    caches = list(args.cache_root.glob("*/cache-manifest.json")) if args.cache_root.is_dir() else []
    print(f"immutable oracle cache entries: {len(caches)}")
    scope = active_change_scope()
    budget = "within" if scope["within_one_root_cause_budget"] else "OVER"
    print(
        f"active change scope: {scope['changed_files']} files, "
        f"{scope['production_files']} production files ({budget} five-file investigation budget)"
    )
    return 0


def _doctor_line(level: str, label: str, detail: str) -> None:
    print(f"{level:<4} {label:<22} {detail}")


def command_doctor(args: argparse.Namespace) -> int:
    """Validate the local iteration lane without launching either engine."""
    failures = 0
    warnings = 0

    def check(level: str, label: str, detail: str) -> None:
        nonlocal failures, warnings
        _doctor_line(level, label, detail)
        failures += level == "FAIL"
        warnings += level == "WARN"

    project = args.project.resolve()
    if project.is_dir():
        check("PASS", "route project", str(project.relative_to(ROOT)))
    else:
        check("FAIL", "route project", f"missing {project}")

    binary = args.binary.resolve()
    if not binary.is_file():
        check("FAIL", "parity binary", f"missing {binary}")
    else:
        newest_mtime, newest_path = parity_probe.newest_source_mtime()
        if binary.stat().st_mtime < newest_mtime:
            check(
                "FAIL",
                "parity binary",
                f"stale; newer source {newest_path.relative_to(ROOT) if newest_path else '?'}",
            )
        else:
            check(
                "PASS",
                "parity binary",
                f"{evidence.sha256_file(binary)[:16]} ({binary.stat().st_size / (1024 * 1024):.1f} MiB)",
            )

    if args.symbols.is_file():
        symbols = SymbolTable(args.symbols)
        check("PASS", "symbol table", f"{len(symbols.by_address)} symbols from {args.symbols}")
    else:
        check("WARN", "symbol table", f"missing {args.symbols}; numeric PCs still work")

    try:
        core_sha = parity_probe.validate_trace_core(parity_probe.TRACE_CORE)
    except SystemExit as error:
        check("FAIL", "trace core", str(error))
    else:
        check("PASS", "trace core", core_sha[:16])

    if project.is_dir() and binary.is_file():
        try:
            state = read_local_state(args.state)
            frontier = args.frontier or default_frontier(state)
            context = run_dir_context(
                project=project,
                binary=binary,
                frontier=frontier,
                override=args.run_dir,
                require_recorded_rom_random=False,
            )
        except SystemExit as error:
            check("FAIL", "replay provenance", str(error))
        else:
            identity = context["run_identity"]
            check(
                "PASS",
                "replay provenance",
                f"{Path(context['run_dir']).name} covers {identity['frames_completed']} frame(s)",
            )

    checkpoint = parity_probe.default_frontier_checkpoint_dir(project)
    saved = parity_probe.checkpoint_generation(checkpoint)
    identity_path = checkpoint / parity_probe.IDENTITY_NAME
    if saved is None:
        check("WARN", "paired checkpoint", "none; first microscope run will bootstrap one")
    elif not identity_path.is_file():
        check("FAIL", "paired checkpoint", f"frame {saved[0]} has no identity sidecar")
    else:
        try:
            identity = evidence.load_json(identity_path)
            rng = checkpoint / "rom-random.txt"
            rng_ok = (
                rng.is_file()
                and identity.get("rom_random_sha256") == evidence.sha256_file(rng)
            )
        except SystemExit as error:
            check("FAIL", "paired checkpoint", str(error))
        else:
            level = "PASS" if rng_ok else "WARN"
            check(level, "paired checkpoint", f"frame {saved[0]}; bound RNG={'yes' if rng_ok else 'no'}")

    try:
        cache = evidence.verify_oracle_cache_root(args.cache_root)
    except SystemExit as error:
        check("FAIL", "oracle cache", str(error))
    else:
        check(
            "PASS",
            "oracle cache",
            f"{cache['entries']} entries, {cache['artifacts']} artifacts, {cache['bytes'] / (1024 * 1024):.1f} MiB verified",
        )
        engine = rust_evidence_binary(required=False, require_fresh=False)
        if engine is None:
            check(
                "WARN",
                "Rust evidence engine",
                "missing; run cargo build --profile parity -p parity",
            )
        else:
            newest_mtime, newest_source = newest_rust_evidence_source()
            if engine.stat().st_mtime < newest_mtime:
                check(
                    "FAIL",
                    "Rust evidence engine",
                    f"stale; newer source {newest_source.relative_to(ROOT)}",
                )
                engine = None
        if engine is not None:
            rust_cache = subprocess.run(
                [str(engine), "cache-verify", str(args.cache_root.resolve()), "--json"],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            try:
                rust_inventory = json.loads(rust_cache.stdout)
            except json.JSONDecodeError:
                rust_inventory = None
            if rust_cache.returncode != 0 or rust_inventory != cache:
                detail = rust_cache.stderr.strip() or (
                    f"inventory disagrees with Python: Rust={rust_inventory!r} Python={cache!r}"
                )
                check("FAIL", "Rust evidence engine", detail)
            else:
                check(
                    "PASS",
                    "Rust evidence engine",
                    f"{engine}; cache contract agrees with Python",
                )

    usage = shutil.disk_usage(ROOT)
    free_gib = usage.free / (1024**3)
    check("PASS" if free_gib >= args.min_free_gib else "WARN", "free disk", f"{free_gib:.1f} GiB")

    lock_path = Path("/tmp/zelda3-snes9x-compare.lock")
    try:
        with lock_path.open("a+") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            fcntl.flock(lock, fcntl.LOCK_UN)
    except BlockingIOError:
        check("WARN", "GPU/oracle lock", "currently held by another comparison")
    except OSError as error:
        check("FAIL", "GPU/oracle lock", str(error))
    else:
        check("PASS", "GPU/oracle lock", "available")

    print(f"doctor: {failures} failure(s), {warnings} warning(s)")
    return 1 if failures else 0


def command_inspect(args: argparse.Namespace) -> int:
    session = args.session.resolve()
    identity = evidence.session_identity(session)
    manifest = evidence.load_json(session / "manifest.json")
    result = evidence.load_json(session / "result.json")
    timing = manifest.get("timing") if isinstance(manifest.get("timing"), dict) else {}
    start_frame = int(timing.get("start_frame", 0))
    print(f"session: {session}")
    print(
        f"result: status={identity['status']} completed={identity['frames_completed']} "
        f"start={start_frame} first_cause={first_cause(result).get('classification')}"
    )
    trace = session / "snes9x-trace.jsonl"
    if trace.is_file():
        plan_path = session / "microscope-plan.json"
        plan = evidence.load_json(plan_path) if plan_path.is_file() else {}
        internal_filter = (
            plan.get("trace", {}).get("internal_frame_filter")
            if isinstance(plan.get("trace"), dict)
            else None
        )
        timeline, _ = build_timeline(
            trace,
            start_frame=start_frame,
            symbols=SymbolTable(args.symbols),
            internal_frame_filter=internal_filter,
        )
        counts: dict[str, int] = defaultdict(int)
        for record in read_jsonl(trace):
            counts[str(record.get("event", "?"))] += 1
        busiest = sorted(timeline["runs"], key=lambda item: item["events"], reverse=True)[:3]
        print(
            "oracle trace: "
            + ", ".join(f"{name}={count}" for name, count in sorted(counts.items()))
        )
        print(
            "busiest host frames: "
            + ", ".join(f"{item['host_frame']} ({item['events']} events)" for item in busiest)
        )
        if timeline["incomplete_or_ambiguous_runs"]:
            print(
                "FAIL timeline has incomplete/ambiguous runs: "
                + ", ".join(map(str, timeline["incomplete_or_ambiguous_runs"]))
            )
            return 1

    correlation = cpu_checkpoint_correlation(session)
    print(f"CPU checkpoint correlation: {correlation['status']}")
    if correlation["status"] == "invalid":
        print(f"FAIL {correlation['problem']}")
        return 1
    if correlation["status"] == "compared":
        print(
            "  host frames: " + ", ".join(map(str, correlation["host_frames"]))
        )
        print(
            "  oracle-rust master-cycle deltas: "
            + ", ".join(map(str, correlation["oracle_minus_rust_master_cycles"]))
        )
        if correlation.get("rust_only_host_frames"):
            print(
                "  Rust-only checkpoints (oracle had not reached the PC in that host): "
                + ", ".join(map(str, correlation["rust_only_host_frames"]))
            )

    report_path = session / "report.json"
    if report_path.is_file():
        report = evidence.load_json(report_path)
        cache_path = report.get("oracle_cache", {}).get("path")
        if isinstance(cache_path, str):
            cache = Path(cache_path)
            manifest_path = cache / "cache-manifest.json"
            if not manifest_path.is_file():
                print(f"FAIL referenced oracle cache is missing: {cache}")
                return 1
            evidence.verify_oracle_cache_entry(cache, evidence.load_json(manifest_path))
            print(f"oracle cache: verified {cache.name}")
    return 0


def command_cache(args: argparse.Namespace) -> int:
    cache, reused = evidence.cache_oracle_session(args.session, cache_root=args.cache_root)
    print(f"oracle cache: {cache} ({'reused' if reused else 'created'})")
    return 0


def command_promote(args: argparse.Namespace) -> int:
    ledger = evidence.promote_frontier(
        ledger_path=args.ledger, binary=args.binary, pass_root=args.pass_root
    )
    promoted = ledger["promoted"]
    print(
        f"promoted exact A/V frontier {promoted['last_exact_video_frame']} "
        f"for commit {promoted['commit']}"
    )
    print(f"stage and commit {args.ledger}; it is a metadata-only promotion")
    return 0


def command_timeline(args: argparse.Namespace) -> int:
    symbols = SymbolTable(args.symbols)
    report, rendered = build_timeline(
        args.trace, start_frame=args.start_frame, symbols=symbols
    )
    if args.output:
        write_json(args.output.with_suffix(".json"), report)
        args.output.with_suffix(".txt").write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 1 if report["incomplete_or_ambiguous_runs"] and args.strict else 0


def command_pc(args: argparse.Namespace) -> int:
    symbols = SymbolTable(args.symbols)
    pc = symbols.parse(args.value)
    print(f"requested: {format_pc(pc)} {symbols.describe(pc)}")
    print("trace aliases: " + ",".join(format_pc(value) for value in lorom_pc_aliases(pc)))
    return 0


def command_trace_index(args: argparse.Namespace) -> int:
    built = build_rust_trace_index(args.session, output=args.output)
    assert built is not None
    index, process = built
    if process.stdout:
        print(process.stdout, end="")
    if process.stderr:
        print(process.stderr, end="", file=sys.stderr)
    if process.returncode == 0:
        print(f"parity microscope: Rust trace index {index}")
    return process.returncode


def command_trace_query(args: argparse.Namespace) -> int:
    engine = rust_evidence_binary()
    assert engine is not None
    target = args.target.resolve()
    index = target / "snes9x-trace.zpti" if target.is_dir() else target
    if not index.is_file():
        raise SystemExit(
            f"parity microscope: trace index is missing: {index}; "
            f"run ./parity trace-index {target}"
        )
    command = [str(engine), "trace-query", str(index)]
    for name in ("host_frame", "run", "internal_frame", "limit"):
        value = getattr(args, name)
        if value is not None:
            command += ["--" + name.replace("_", "-"), str(value)]
    if args.pc is not None:
        pc = SymbolTable(args.symbols).parse(args.pc)
        command += ["--pc", format_pc(pc)]
    if args.wram is not None:
        validate_wram_filters([args.wram])
        command += ["--wram", args.wram]
    for event in args.event:
        command += ["--event", event]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


def command_cache_verify(args: argparse.Namespace) -> int:
    engine = rust_evidence_binary()
    assert engine is not None
    return subprocess.run(
        [str(engine), "cache-verify", str(args.cache_root.resolve())],
        cwd=ROOT,
        check=False,
    ).returncode


def command_receipt_compare(args: argparse.Namespace) -> int:
    engine = rust_evidence_binary()
    assert engine is not None
    session = args.session.resolve()
    candidate = session / "frame_receipts.jsonl"
    if not candidate.is_file():
        raise SystemExit(f"parity microscope: missing candidate receipts: {candidate}")
    cache = args.cache.resolve() if args.cache is not None else None
    if cache is None:
        report_path = session / "report.json"
        if not report_path.is_file():
            raise SystemExit(
                "parity microscope: session has no report.json cache reference; pass --cache"
            )
        cache_path = evidence.load_json(report_path).get("oracle_cache", {}).get("path")
        if not isinstance(cache_path, str):
            raise SystemExit(
                "parity microscope: session report has no oracle cache reference; pass --cache"
            )
        cache = Path(cache_path).resolve()
    oracle = cache / "oracle-frame-receipts.jsonl"
    if not oracle.is_file():
        raise SystemExit(f"parity microscope: missing cached oracle receipts: {oracle}")
    command = [str(engine), "receipt-compare", str(candidate), str(oracle)]
    if args.json:
        command.append("--json")
    if args.allow_incomplete:
        command.append("--allow-incomplete")
    command += ["--max-differing-frames", str(args.max_differing_frames)]
    command += ["--max-differences-per-frame", str(args.max_differences_per_frame)]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


def command_av_compare(args: argparse.Namespace) -> int:
    engine = rust_evidence_binary()
    assert engine is not None
    session = args.session.resolve()
    candidate = session / "av_hashes.jsonl"
    if not candidate.is_file():
        raise SystemExit(f"parity microscope: missing candidate A/V hashes: {candidate}")
    cache = args.cache.resolve() if args.cache is not None else None
    if cache is None:
        report_path = session / "report.json"
        if not report_path.is_file():
            raise SystemExit(
                "parity microscope: session has no report.json cache reference; pass --cache"
            )
        cache_path = evidence.load_json(report_path).get("oracle_cache", {}).get("path")
        if not isinstance(cache_path, str):
            raise SystemExit(
                "parity microscope: session report has no oracle cache reference; pass --cache"
            )
        cache = Path(cache_path).resolve()
    oracle = cache / "oracle-av-hashes.jsonl"
    if not oracle.is_file():
        raise SystemExit(f"parity microscope: missing cached oracle A/V hashes: {oracle}")
    command = [str(engine), "av-compare", str(candidate), str(oracle)]
    if args.json:
        command.append("--json")
    if args.allow_incomplete:
        command.append("--allow-incomplete")
    command += ["--max-differing-frames", str(args.max_differing_frames)]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


def command_cached_av(args: argparse.Namespace) -> int:
    engine = rust_evidence_binary()
    assert engine is not None
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(
            f"parity microscope: missing {binary}; run cargo build --profile parity -p zelda3-bin"
        )
    cache = args.cache.resolve()
    oracle = cache / "oracle-av-hashes.jsonl"
    if not oracle.is_file():
        raise SystemExit(
            f"parity microscope: cache has no complete canonical A/V ledger: {oracle}"
        )
    output = args.output
    if output is None:
        output = (
            ROOT
            / "target"
            / "parity-cached-av"
            / f"run-{cache.name[:12]}-{time.strftime('%Y%m%d-%H%M%S')}"
        )
    output = output.resolve()
    started = time.monotonic()
    replay = subprocess.run(
        [
            str(binary),
            "--replay-cached-snes9x-av",
            str(cache),
            str(args.rom.resolve()),
            str(output),
        ],
        cwd=ROOT,
        check=False,
    )
    if replay.returncode not in (0, 1):
        return replay.returncode
    candidate = output / "av_hashes.jsonl"
    if not candidate.is_file():
        raise SystemExit(f"parity microscope: Rust replay wrote no candidate ledger: {candidate}")
    comparison = subprocess.run(
        [
            str(engine),
            "av-compare",
            str(candidate),
            str(oracle),
            "--max-differing-frames",
            str(args.max_differing_frames),
            "--candidate-stopped-at-first-mismatch",
        ],
        cwd=ROOT,
        check=False,
    )
    print(
        f"cached A/V replay: {output} ({time.monotonic() - started:.2f}s, no Snes9x core loaded)"
    )
    return comparison.returncode if comparison.returncode != 0 else replay.returncode


def command_oracle_av_capture(args: argparse.Namespace) -> int:
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(
            f"parity microscope: missing {binary}; run cargo build --profile parity -p zelda3-bin"
        )
    source = args.source_session.resolve()
    source_identity = evidence.session_identity(source)
    if args.frames <= 0 or args.frames > source_identity["frames_completed"]:
        raise SystemExit(
            f"parity microscope: --frames must be within source coverage "
            f"1..{source_identity['frames_completed']}"
        )
    manifest = evidence.load_json(source / "manifest.json")
    core_record = manifest.get("core")
    rom_record = manifest.get("rom")
    if not isinstance(core_record, dict) or not isinstance(rom_record, dict):
        raise SystemExit("parity microscope: source session has no core/ROM identity")
    core_sha = source_identity["core_sha256"]
    rom_sha = source_identity["rom_sha256"]
    core_value = args.core if args.core is not None else core_record.get("path")
    rom_value = args.rom if args.rom is not None else rom_record.get("path")
    if not isinstance(core_sha, str) or not isinstance(rom_sha, str):
        raise SystemExit("parity microscope: source session has no core/ROM hashes")
    if not isinstance(core_value, (str, Path)) or not isinstance(rom_value, (str, Path)):
        raise SystemExit("parity microscope: source session has no usable core/ROM paths")
    core = Path(core_value).resolve()
    rom = Path(rom_value).resolve()
    input_path = source / "input.txt"
    rng_path = source / "rom-random.txt"
    sram_path = source / "initial.srm"
    missing = [path for path in (core, rom, input_path, sram_path) if not path.is_file()]
    if missing:
        raise SystemExit(
            "parity microscope: oracle A/V capture source is missing: "
            + ", ".join(map(str, missing))
        )
    output = args.output
    if output is None:
        output = (
            ROOT
            / "target"
            / "parity-oracle-av"
            / f"capture-{args.frames}-{time.strftime('%Y%m%d-%H%M%S')}"
        )
    output = output.resolve()
    temporary_rng_path = None
    if not rng_path.is_file():
        authority = manifest.get("rom_random_authority")
        artifact = authority.get("artifact") if isinstance(authority, dict) else None
        mode = authority.get("mode") if isinstance(authority, dict) else None
        if mode != "live_oracle_trace" or not isinstance(artifact, str):
            raise SystemExit(
                "parity microscope: source has neither rom-random.txt nor a manifest-bound live oracle RNG trace"
            )
        trace = source / artifact
        if not trace.is_file():
            raise SystemExit(f"parity microscope: source RNG trace is missing: {trace}")
        with trace.open(encoding="utf-8") as stream:
            samples = extract_samples(stream)
        if not samples:
            raise SystemExit(f"parity microscope: source RNG trace has no cartridge samples: {trace}")
        temporary = tempfile.NamedTemporaryFile(
            mode="w",
            encoding="utf-8",
            prefix="zelda3-oracle-rng-",
            suffix=".txt",
            delete=False,
        )
        try:
            write_script(samples, temporary)
        finally:
            temporary.close()
        temporary_rng_path = Path(temporary.name)
        rng_path = temporary_rng_path
        print(
            f"parity microscope: materialized {len(samples)} RNG sample(s) from manifest-bound {trace}"
        )
    command = [
        str(binary),
        "--capture-snes9x-av",
        str(core),
        str(rom),
        str(args.frames),
        str(input_path),
        str(rng_path),
        str(sram_path),
        str(output),
        core_sha,
        rom_sha,
    ]
    started = time.monotonic()
    try:
        completed = subprocess.run(command, cwd=ROOT, check=False)
    finally:
        if temporary_rng_path is not None:
            temporary_rng_path.unlink(missing_ok=True)
    if completed.returncode != 0:
        return completed.returncode
    cache, reused = evidence.cache_oracle_session(output, cache_root=args.cache_root)
    print(
        f"oracle-only A/V capture: {output} ({time.monotonic() - started:.2f}s); "
        f"cache {cache} ({'reused' if reused else 'created'})"
    )
    return 0


def command_microscope(args: argparse.Namespace) -> int:
    if args.max_trace_mib < 1:
        raise SystemExit("parity microscope: --max-trace-mib must be positive")
    if args.trace_tail_frames < 0:
        raise SystemExit("parity microscope: --trace-tail-frames must be nonnegative")
    if args.state != DEFAULT_STATE and not args.state.is_file():
        raise SystemExit(
            "parity microscope: --state is a precommit frontier JSON file, not a "
            f"checkpoint directory; missing file: {args.state}"
        )
    state = read_local_state(args.state)
    frontier = args.frontier if args.frontier is not None else default_frontier(state)
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(
            f"parity microscope: missing {binary}; run cargo build --profile parity -p zelda3-bin"
        )
    project = args.project.resolve()
    checkpoint_dir, checkpoint_frame = resolve_microscope_checkpoint(
        args, project=project
    )
    context = run_dir_context(
        project=project,
        binary=binary,
        frontier=frontier,
        override=args.run_dir,
        require_recorded_rom_random=args.recorded_rng,
    )
    symbols = SymbolTable(args.symbols)
    requested_pcs = list(args.pc)
    if not args.no_cpu_checkpoint:
        requested_pcs.append(args.cpu_checkpoint_pc)
    pc_filter = symbols.trace_filter(requested_pcs) if requested_pcs else ""
    wram_filter = validate_wram_filters(args.wram) if args.wram else ""
    events = parse_events(args.events, has_pc=bool(pc_filter), has_wram=bool(wram_filter))
    high_volume = bool(args.pc or wram_filter)
    if args.cold and high_volume and args.trace_internal_frames is None:
        raise SystemExit(
            "parity microscope: cold PC/WRAM tracing requires --trace-internal-frames "
            "FIRST-LAST; bootstrap a diagnostic checkpoint instead of tracing the whole route"
        )

    stamp = time.strftime("%Y%m%d-%H%M%S")
    session = (args.session_dir or args.output_root / f"run-{frontier}-{stamp}").resolve()
    session.mkdir(parents=True, exist_ok=True)
    default_checkpoint_frame = parity_probe.saved_checkpoint_frame(
        parity_probe.default_frontier_checkpoint_dir(project)
    )
    force_cold_fallback = (
        not args.cold
        and checkpoint_frame is None
        and checkpoint_dir is None
        and default_checkpoint_frame is not None
        and default_checkpoint_frame >= frontier
    )
    trace = session / "snes9x-trace.jsonl"
    rust_trace = session / "rust-cpu-checkpoints.jsonl"
    trace_configuration = {
        "schema": 1,
        "events": events,
        "pc_filter": pc_filter,
        "wram_filter": wram_filter,
        "internal_frame_filter": args.trace_internal_frames,
        "automatic_tail_frames": args.trace_tail_frames,
        "frame_coordinate": "Snes9x internal frame; host identity uses retro_run",
    }
    plan = {
        "schema": 1,
        "kind": "zelda3-parity-microscope-plan",
        "frontier": frontier,
        "authoritative": False,
        "session": str(session),
        "selected_replay": context,
        "trace": trace_configuration,
        "max_trace_mib": args.max_trace_mib,
        "checkpoint": {
            "directory": str(checkpoint_dir) if checkpoint_dir is not None else None,
            "frame": checkpoint_frame,
            "trust_cross_build": args.trust_cross_build_checkpoint,
        },
        "evidence_tier": (
            "cold-from-zero"
            if args.cold
            else "diagnostic-cold-fallback"
            if force_cold_fallback
            else "diagnostic-checkpoint"
        ),
    }
    write_json(session / "microscope-plan.json", plan)

    source_rng = None
    source_rng_samples = None
    if not args.recorded_rng and not args.cold:
        resolved_source_rng = materialize_source_session_rng(
            Path(context["run_dir"]), session / "source-rom-random.txt"
        )
        if resolved_source_rng is not None:
            source_rng, source_rng_samples = resolved_source_rng
            plan["rom_random"] = {
                "authority": "selected replay session",
                "path": str(source_rng),
                "sha256": evidence.sha256_file(source_rng),
                "samples": source_rng_samples,
            }
            write_json(session / "microscope-plan.json", plan)
    cached_rng = (
        None
        if args.recorded_rng or args.cold or source_rng is not None
        else cached_diagnostic_rng(project)
    )

    def probe_command(
        output: Path, *, live_rng: bool, rom_random_script: Path | None = None
    ) -> list[str]:
        command = [
            sys.executable,
            str(SCRIPT_DIR / "parity_probe.py"),
            "--frontier",
            str(frontier),
            "--project",
            str(project),
            "--run-dir",
            str(context["run_dir"]),
            "--binary",
            str(binary),
            "--session-dir",
            str(output),
            "--core",
            "instrumented",
            "--no-frontier-capture",
            "--trace-only",
        ]
        if live_rng:
            command += [
                "--live-oracle-rng",
                "--engine-state-from-frame",
                str(max(200, int(state.get("last_checked_frame", 0)))),
            ]
        elif rom_random_script is not None:
            command += ["--rom-random-script", str(rom_random_script)]
        if args.cold or force_cold_fallback:
            command.append("--no-checkpoint")
        command += microscope_checkpoint_probe_args(
            checkpoint_dir,
            checkpoint_frame,
            trust_cross_build=args.trust_cross_build_checkpoint,
        )
        if args.dry_run:
            command.append("--dry-run")
        return command

    bootstrap_session = session / "checkpoint-bootstrap"
    checkpoint_rng_path = parity_probe.default_frontier_checkpoint_dir(project) / "rom-random.txt"
    bootstrap_required = (
        not args.cold
        and not args.recorded_rng
        and source_rng is None
        and cached_rng is None
    )
    bootstrap_command = probe_command(bootstrap_session, live_rng=True)
    command = probe_command(
        session,
        live_rng=bool(args.cold and not args.recorded_rng),
        rom_random_script=(
            source_rng or cached_rng or checkpoint_rng_path
            if not args.recorded_rng and not args.cold
            else None
        ),
    )

    trace_env: dict[str, str] = {
        "ZELDA3_SNES9X_TRACE": str(trace),
        "ZELDA3_SNES9X_TRACE_EVENTS": events,
    }
    if not args.no_cpu_checkpoint:
        trace_env["ZELDA3_CPU_CHECKPOINT_TRACE"] = str(rust_trace)
    if pc_filter:
        trace_env["ZELDA3_SNES9X_TRACE_PCS"] = pc_filter
    if wram_filter:
        trace_env["ZELDA3_SNES9X_TRACE_WRAM"] = wram_filter
    if args.trace_internal_frames is not None:
        trace_env["ZELDA3_SNES9X_TRACE_FRAMES"] = args.trace_internal_frames

    def write_replay_script() -> None:
        replay = "#!/bin/sh\nset -eu\ncd " + shlex.quote(str(ROOT)) + "\n"
        if bootstrap_required:
            replay += shlex.join(bootstrap_command) + "\n"
        replay += " ".join(
            [
                *(f"{name}={shlex.quote(value)}" for name, value in sorted(trace_env.items())),
                shlex.join(command),
            ]
        ) + "\n"
        (session / "replay-microscope.sh").write_text(replay, encoding="utf-8")
        os.chmod(session / "replay-microscope.sh", 0o755)

    if not bootstrap_required and args.trace_internal_frames is None and not args.cold:
        automatic_range = (
            f"{max(0, frontier - args.trace_tail_frames)}-{frontier}"
            if force_cold_fallback and args.trace_tail_frames > 0
            else automatic_trace_frame_range(
                project=project,
                frontier=frontier,
                tail_frames=args.trace_tail_frames,
                checkpoint_dir=checkpoint_dir,
            )
        )
        if automatic_range is not None:
            trace_env["ZELDA3_SNES9X_TRACE_FRAMES"] = automatic_range
            trace_configuration["internal_frame_filter"] = automatic_range
            trace_configuration["internal_frame_filter_authority"] = (
                "cold replay internal frames equal host frames"
                if force_cold_fallback
                else "derived from the hash-bound paired checkpoint frame"
            )
            write_json(session / "microscope-plan.json", plan)
    write_replay_script()

    print(f"parity microscope: frontier {frontier}")
    print(f"parity microscope: replay {context['run_dir']}")
    print(f"parity microscope: evidence tier {plan['evidence_tier']}")
    if force_cold_fallback:
        print(
            "parity microscope: checkpoint frame "
            f"{default_checkpoint_frame} is not before frontier {frontier}; "
            "forcing a cold diagnostic"
        )
    print(f"parity microscope: session {session}")
    if source_rng is not None:
        print(
            "parity microscope: RNG from selected replay "
            f"{source_rng} ({source_rng_samples} samples)"
        )
    print(f"parity microscope: trace PCs {pc_filter or '(none)'}")
    if trace_env.get("ZELDA3_SNES9X_TRACE_FRAMES"):
        print(
            "parity microscope: trace internal frames "
            + trace_env["ZELDA3_SNES9X_TRACE_FRAMES"]
        )
    if bootstrap_required:
        print(
            "parity microscope: checkpoint bootstrap "
            f"{shlex.join(bootstrap_command)}"
        )
    print(f"parity microscope: trace command {shlex.join(command)}")
    if args.dry_run:
        return 0

    if bootstrap_required:
        bootstrap_environment = {
            name: value
            for name, value in os.environ.items()
            if not name.startswith("ZELDA3_SNES9X_TRACE")
            and name != "ZELDA3_CPU_CHECKPOINT_TRACE"
        }
        bootstrap = subprocess.run(
            bootstrap_command, cwd=ROOT, env=bootstrap_environment, check=False
        )
        if bootstrap.returncode != 0:
            report = dict(plan)
            report["bootstrap"] = {
                "status": "failed",
                "returncode": bootstrap.returncode,
                "session": str(bootstrap_session),
            }
            write_json(session / "report.json", report)
            print(f"parity microscope: checkpoint bootstrap failed; report {session / 'report.json'}")
            return bootstrap.returncode
        materialized_rng, sample_count = bind_live_rng_to_diagnostic_checkpoint(
            bootstrap_session, project
        )
        print(
            f"parity microscope: materialized {sample_count} live oracle RNG sample(s) "
            f"for checkpoint replay at {materialized_rng}"
        )
        if args.trace_internal_frames is None and not args.cold:
            automatic_range = automatic_trace_frame_range(
                project=project,
                frontier=frontier,
                tail_frames=args.trace_tail_frames,
                checkpoint_dir=checkpoint_dir,
            )
            if automatic_range is not None:
                trace_env["ZELDA3_SNES9X_TRACE_FRAMES"] = automatic_range
                trace_configuration["internal_frame_filter"] = automatic_range
                trace_configuration["internal_frame_filter_authority"] = (
                    "derived from the hash-bound paired checkpoint frame"
                )
                plan["trace"] = trace_configuration
                write_json(session / "microscope-plan.json", plan)
                write_replay_script()
                print(
                    "parity microscope: auto-bounded trace to internal frames "
                    + automatic_range
                )

    environment = dict(os.environ)
    environment.update(trace_env)
    process_returncode = run_with_trace_limit(
        command,
        environment=environment,
        trace=trace,
        max_bytes=args.max_trace_mib * 1024 * 1024,
    )
    report: dict[str, Any] = dict(plan)
    result_path = session / "result.json"
    if result_path.is_file():
        result = evidence.load_json(result_path)
        report["result"] = {
            "status": result.get("status"),
            "frames_completed": result.get("frames_completed"),
            "first_cause": first_cause(result),
            "sha256": evidence.sha256_file(result_path),
        }
    if trace.is_file():
        manifest = evidence.load_json(session / "manifest.json")
        start_frame = int(manifest.get("timing", {}).get("start_frame", 0))
        timeline, rendered = build_timeline(
            trace,
            start_frame=start_frame,
            symbols=symbols,
            internal_frame_filter=trace_configuration.get("internal_frame_filter"),
        )
        write_json(session / "timeline.json", timeline)
        (session / "timeline.txt").write_text(rendered, encoding="utf-8")
        report["timeline"] = {
            "path": str(session / "timeline.json"),
            "incomplete_or_ambiguous_runs": timeline["incomplete_or_ambiguous_runs"],
        }
        indexed = build_rust_trace_index(session, required=False)
        if indexed is not None:
            index, index_process = indexed
            report["trace_index"] = {
                "path": str(index),
                "returncode": index_process.returncode,
                "stdout": index_process.stdout.strip(),
                "stderr": index_process.stderr.strip(),
            }
            if index_process.returncode == 0:
                print(f"parity microscope: Rust trace index {index}")
            else:
                print(
                    "parity microscope: Rust trace indexing failed: "
                    + index_process.stderr.strip(),
                    file=sys.stderr,
                )
                if process_returncode == 0:
                    process_returncode = 75
    correlation = cpu_checkpoint_correlation(session)
    report["cpu_checkpoint_correlation"] = correlation
    if correlation["status"] == "invalid":
        print(
            "parity microscope: invalid CPU checkpoint provenance: "
            + str(correlation["problem"]),
            file=sys.stderr,
        )
        if process_returncode == 0:
            process_returncode = 76
    elif correlation["status"] == "compared":
        print(
            "parity microscope: correlated CPU checkpoint host frame(s) "
            + ",".join(map(str, correlation["host_frames"]))
        )
        if correlation.get("rust_only_host_frames"):
            print(
                "parity microscope: Rust-only checkpoint host frame(s), not offset-guessed: "
                + ",".join(map(str, correlation["rust_only_host_frames"]))
            )
    manifest_path = session / "manifest.json"
    if manifest_path.is_file() and result_path.is_file():
        manifest = evidence.load_json(manifest_path)
        lanes = manifest.get("comparison_lanes")
        has_enabled_av_lane = isinstance(lanes, dict) and any(
            bool(lanes.get(name)) for name in ("video", "audio")
        )
        if has_enabled_av_lane:
            cache, reused = evidence.cache_oracle_session(
                session,
                cache_root=args.cache_root,
                trace_configuration=trace_configuration,
            )
            report["oracle_cache"] = {"path": str(cache), "reused": reused}
            print(
                f"parity microscope: oracle evidence cache {cache} "
                f"({'reused' if reused else 'created'})"
            )
        else:
            report["oracle_cache"] = {
                "skipped": "trace-only session has no enabled A/V lane"
            }
    write_json(session / "report.json", report)
    print(f"parity microscope: report {session / 'report.json'}")
    return process_returncode


def parser() -> argparse.ArgumentParser:
    command = argparse.ArgumentParser(description=__doc__)
    subcommands = command.add_subparsers(dest="command", required=True)

    status = subcommands.add_parser("status", help="show durable, local, and experimental frontiers")
    status.add_argument("--project", type=Path, default=DEFAULT_PROJECT)
    status.add_argument("--state", type=Path, default=DEFAULT_STATE)
    status.add_argument("--ledger", type=Path, default=evidence.DEFAULT_LEDGER)
    status.add_argument("--pass-root", type=Path, default=evidence.PASS_ROOT)
    status.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    status.set_defaults(handler=command_status)

    doctor = subcommands.add_parser(
        "doctor", help="validate binary, replay, checkpoint, cache, disk, and lock readiness"
    )
    doctor.add_argument("--project", type=Path, default=DEFAULT_PROJECT)
    doctor.add_argument("--state", type=Path, default=DEFAULT_STATE)
    doctor.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    doctor.add_argument("--run-dir", type=Path)
    doctor.add_argument("--frontier", type=int)
    doctor.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    doctor.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    doctor.add_argument("--min-free-gib", type=float, default=5.0)
    doctor.set_defaults(handler=command_doctor)

    inspect = subcommands.add_parser(
        "inspect", help="audit one completed session and correlate its coordinates"
    )
    inspect.add_argument("session", type=Path)
    inspect.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    inspect.set_defaults(handler=command_inspect)

    explain = subcommands.add_parser(
        "explain-frame",
        help="join failure VRAM generations with the exact oracle DMA run",
    )
    explain.add_argument("failure", type=Path)
    explain.add_argument("--trace", type=Path)
    explain.add_argument(
        "--resume-frame",
        type=int,
        default=0,
        help="absolute host frame represented by trace run 0",
    )
    explain.add_argument("--output", type=Path)
    explain.add_argument("--json", action="store_true")
    explain.set_defaults(handler=command_explain_frame)

    microscope = subcommands.add_parser("microscope", help="capture one provenance-safe frontier window")
    microscope.add_argument("--frontier", type=int)
    microscope.add_argument("--project", type=Path, default=DEFAULT_PROJECT)
    microscope.add_argument(
        "--state",
        type=Path,
        default=DEFAULT_STATE,
        help="precommit frontier state JSON (not an emulator checkpoint)",
    )
    microscope.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    microscope.add_argument("--run-dir", type=Path)
    microscope.add_argument("--session-dir", type=Path)
    microscope.add_argument("--output-root", type=Path, default=DEFAULT_OUTPUT)
    microscope.add_argument(
        "--max-trace-mib",
        type=int,
        default=128,
        help="stop a trace that grows past this size (default 128 MiB)",
    )
    microscope.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    microscope.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    microscope.add_argument("--pc", action="append", default=[], metavar="PC_OR_SYMBOL")
    microscope.add_argument("--wram", action="append", default=[], metavar="ADDR_OR_RANGE")
    microscope.add_argument("--events", default="frame,nmi,nmi-resume,dma")
    microscope.add_argument(
        "--cpu-checkpoint-pc",
        default="00:8051",
        metavar="PC_OR_SYMBOL",
        help="low-volume oracle PC paired with Rust CPU checkpoints (default 00:8051)",
    )
    microscope.add_argument(
        "--no-cpu-checkpoint",
        action="store_true",
        help="disable automatic Rust/oracle CPU-checkpoint correlation",
    )
    microscope.add_argument(
        "--trace-tail-frames",
        type=int,
        default=12,
        help="on checkpoint resume, trace only this many internal frames before the frontier; 0 disables (default 12)",
    )
    microscope.add_argument("--checkpoint-frame", type=int)
    microscope.add_argument(
        "--checkpoint-dir",
        type=Path,
        help="explicit paired checkpoint root or generation; fails if invalid",
    )
    microscope.add_argument(
        "--trust-cross-build-checkpoint",
        action="store_true",
        help="permit a different Rust build while still verifying replay provenance",
    )
    microscope.add_argument(
        "--trace-internal-frames",
        type=parse_internal_frame_range,
        metavar="FIRST-LAST",
        help="explicit Snes9x internal-frame filter; required for cold PC/WRAM traces",
    )
    microscope.add_argument("--cold", action="store_true", help="start from frame zero; diagnostic is default")
    microscope.add_argument(
        "--recorded-rng",
        action="store_true",
        help="use recorded RNG; same-run live oracle RNG is the safer diagnostic default",
    )
    microscope.add_argument("--dry-run", action="store_true")
    microscope.set_defaults(handler=command_microscope)

    cache = subcommands.add_parser("cache", help="cache oracle-only artifacts from a comparison session")
    cache.add_argument("session", type=Path)
    cache.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    cache.set_defaults(handler=command_cache)

    promote = subcommands.add_parser("promote", help="promote two cold exact passes into the frontier ledger")
    promote.add_argument("--ledger", type=Path, default=evidence.DEFAULT_LEDGER)
    promote.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    promote.add_argument("--pass-root", type=Path, default=evidence.PASS_ROOT)
    promote.set_defaults(handler=command_promote)

    timeline = subcommands.add_parser("timeline", help="map and symbolicate an existing trace")
    timeline.add_argument("trace", type=Path)
    timeline.add_argument("--start-frame", type=int, required=True)
    timeline.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    timeline.add_argument("--output", type=Path)
    timeline.add_argument("--strict", action="store_true")
    timeline.set_defaults(handler=command_timeline)

    trace_index = subcommands.add_parser(
        "trace-index", help="build a provenance-bound Rust seek index for a session trace"
    )
    trace_index.add_argument("session", type=Path)
    trace_index.add_argument("--output", type=Path)
    trace_index.set_defaults(handler=command_trace_index)

    trace_query = subcommands.add_parser(
        "trace-query", help="query original JSONL records through a verified Rust trace index"
    )
    trace_query.add_argument("target", type=Path, help="session directory or .zpti index")
    trace_query.add_argument("--host-frame", type=int)
    trace_query.add_argument("--run", type=int)
    trace_query.add_argument("--internal-frame", type=int)
    trace_query.add_argument("--pc")
    trace_query.add_argument("--wram")
    trace_query.add_argument("--event", action="append", default=[])
    trace_query.add_argument("--limit", type=int)
    trace_query.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    trace_query.set_defaults(handler=command_trace_query)

    cache_verify = subcommands.add_parser(
        "cache-verify", help="verify every oracle-cache identity and artifact in Rust"
    )
    cache_verify.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    cache_verify.set_defaults(handler=command_cache_verify)

    receipt_compare = subcommands.add_parser(
        "receipt-compare",
        help="stream-compare Rust semantic receipts with content-addressed oracle receipts",
    )
    receipt_compare.add_argument("session", type=Path)
    receipt_compare.add_argument("--cache", type=Path)
    receipt_compare.add_argument("--json", action="store_true")
    receipt_compare.add_argument("--allow-incomplete", action="store_true")
    receipt_compare.add_argument("--max-differing-frames", type=int, default=16)
    receipt_compare.add_argument("--max-differences-per-frame", type=int, default=64)
    receipt_compare.set_defaults(handler=command_receipt_compare)

    av_compare = subcommands.add_parser(
        "av-compare",
        help="stream-compare canonical Rust A/V hashes with content-addressed oracle hashes",
    )
    av_compare.add_argument("session", type=Path)
    av_compare.add_argument("--cache", type=Path)
    av_compare.add_argument("--json", action="store_true")
    av_compare.add_argument("--allow-incomplete", action="store_true")
    av_compare.add_argument("--max-differing-frames", type=int, default=16)
    av_compare.set_defaults(handler=command_av_compare)

    cached_av = subcommands.add_parser(
        "cached-av",
        help="replay Rust only against a verified cached Snes9x canonical A/V ledger",
    )
    cached_av.add_argument("cache", type=Path)
    cached_av.add_argument("--rom", type=Path, default=ROOT / "saves" / "zelda3.sfc")
    cached_av.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    cached_av.add_argument("--output", type=Path)
    cached_av.add_argument("--max-differing-frames", type=int, default=16)
    cached_av.set_defaults(handler=command_cached_av)

    oracle_av_capture = subcommands.add_parser(
        "oracle-av-capture",
        help="capture a complete pinned Snes9x A/V ledger once, without running Rust",
    )
    oracle_av_capture.add_argument("source_session", type=Path)
    oracle_av_capture.add_argument("--frames", type=int, required=True)
    oracle_av_capture.add_argument("--core", type=Path)
    oracle_av_capture.add_argument("--rom", type=Path)
    oracle_av_capture.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    oracle_av_capture.add_argument("--output", type=Path)
    oracle_av_capture.add_argument("--cache-root", type=Path, default=evidence.ORACLE_CACHE_ROOT)
    oracle_av_capture.set_defaults(handler=command_oracle_av_capture)

    pc = subcommands.add_parser("pc", help="show canonical and mirrored trace addresses")
    pc.add_argument("value")
    pc.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    pc.set_defaults(handler=command_pc)
    return command


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    return int(args.handler(args))


if __name__ == "__main__":
    raise SystemExit(main())
