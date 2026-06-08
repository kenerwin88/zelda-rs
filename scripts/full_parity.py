#!/usr/bin/env python3
"""Run the automatic Zelda 3 parity gates and report first useful diffs."""

from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_REPO = Path(os.environ.get("ZELDA3_C_REPO", str(REPO_ROOT.parent / "zelda3")))
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(DEFAULT_C_REPO / "zelda3.sfc")))
DEFAULT_MESEN_RUNNER = REPO_ROOT / "external" / "mesen2-oracle" / "run_trace.sh"
DEFAULT_BSNES_LOCAL = REPO_ROOT / "external" / "bsnes-libretro" / "local" / "bsnes_libretro.dylib"
DEFAULT_BSNES_URL = os.environ.get(
    "BSNES_LIBRETRO_URL",
    "https://buildbot.libretro.com/nightly/apple/osx/arm64/latest/bsnes_libretro.dylib.zip",
)


class GateFailure(RuntimeError):
    pass


@dataclass(frozen=True)
class CommandResult:
    command: list[str]
    returncode: int
    stdout: str
    stderr: str


def run_command(command: list[str]) -> CommandResult:
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    return CommandResult(command, result.returncode, result.stdout, result.stderr)


def command_text(command: list[str]) -> str:
    return " ".join(command)


def cargo_zelda(args: list[str], release: bool) -> list[str]:
    command = ["cargo", "run", "-q"]
    if release:
        command.append("--release")
    command.extend(["-p", "zelda3-bin", "--"])
    command.extend(args)
    return command


def require_success(result: CommandResult, gate: str) -> None:
    if result.returncode == 0:
        return
    tail = (result.stderr or result.stdout).strip().splitlines()[-40:]
    raise GateFailure(
        f"{gate} failed with exit code {result.returncode}\n"
        f"command: {command_text(result.command)}\n"
        + "\n".join(tail)
    )


def find_bsnes_core(explicit: str | None) -> Path | None:
    if explicit:
        path = Path(explicit).expanduser()
        return path if path.exists() else None
    env_path = os.environ.get("BSNES_LIBRETRO_CORE")
    if env_path:
        path = Path(env_path).expanduser()
        if path.exists():
            return path
    candidates = [
        DEFAULT_BSNES_LOCAL,
        REPO_ROOT / "external" / "bsnes-libretro" / "bsnes_libretro.dylib",
        Path("/private/tmp/bsnes_libretro/bsnes_libretro.dylib"),
        Path.home() / "Library/Application Support/RetroArch/cores/bsnes_libretro.dylib",
        Path("/Applications/RetroArch.app/Contents/Resources/cores/bsnes_libretro.dylib"),
    ]
    for path in candidates:
        if path.exists():
            return path
    return None


def download_bsnes_core(url: str) -> Path:
    if platform.system() != "Darwin" or platform.machine() not in {"arm64", "aarch64"}:
        raise GateFailure(
            "automatic bsnes download is currently configured for macOS arm64. "
            "Pass --bsnes-core or set BSNES_LIBRETRO_CORE for this platform."
        )
    DEFAULT_BSNES_LOCAL.parent.mkdir(parents=True, exist_ok=True)
    zip_path = DEFAULT_BSNES_LOCAL.with_suffix(".dylib.zip")
    print(f"downloading bsnes libretro core: {url}", flush=True)
    urllib.request.urlretrieve(url, zip_path)
    with zipfile.ZipFile(zip_path) as archive:
        member = next(
            (name for name in archive.namelist() if name.endswith("bsnes_libretro.dylib")),
            None,
        )
        if member is None:
            raise GateFailure(f"downloaded archive does not contain bsnes_libretro.dylib: {zip_path}")
        with archive.open(member) as source, DEFAULT_BSNES_LOCAL.open("wb") as dest:
            dest.write(source.read())
    DEFAULT_BSNES_LOCAL.chmod(0o755)
    return DEFAULT_BSNES_LOCAL


def resolve_bsnes_core(args: argparse.Namespace) -> Path | None:
    core = find_bsnes_core(args.bsnes_core)
    if core is not None or args.no_install_bsnes:
        return core
    return download_bsnes_core(args.bsnes_url)


def run_lockstep_gate(args: argparse.Namespace) -> None:
    command = cargo_zelda(
        ["--compare-lockstep-render", str(args.rom), str(args.lockstep_frames)],
        args.release,
    )
    if args.input_script:
        command.extend(["--input-script", args.input_script])
    if args.load_sram:
        command.extend(["--load-sram", args.load_sram])
    if args.load_state:
        command.extend(["--load-state", args.load_state])
    result = run_command(command)
    require_success(result, "lockstep behavior/render/audio")
    print(result.stdout.strip(), flush=True)


def run_bsnes_gate(args: argparse.Namespace) -> None:
    core = resolve_bsnes_core(args)
    if core is None:
        raise GateFailure(
            "bsnes libretro core not found. Set BSNES_LIBRETRO_CORE or pass "
            "--bsnes-core /path/to/bsnes_libretro.dylib. Full A/V parity cannot run without it."
        )
    command = cargo_zelda(
        [
            "--compare-bsnes-oracle",
            str(core),
            str(args.rom),
            str(args.frames),
            "--skip-bsnes-frames",
            str(args.bsnes_skip),
        ],
        args.release,
    )
    if args.input_script:
        command.extend(["--input-script", args.input_script])
    result = run_command(command)
    require_success(result, "bsnes exact audio/video")
    print(result.stdout.strip(), flush=True)


def run_c_audio_gate(args: argparse.Namespace) -> None:
    command = [
        sys.executable,
        str(REPO_ROOT / "scripts" / "compare_c_audio.py"),
        "--rom",
        str(args.rom),
        "--frames",
        str(args.frames),
        "--c-repo",
        str(args.c_repo),
        "--c-bin",
        str(args.c_bin),
        "--work-dir",
        str(args.work_dir / "audio-c-oracle"),
    ]
    if args.release:
        command.append("--release")
    result = run_command(command)
    require_success(result, "C oracle audio")
    print(result.stdout.strip(), flush=True)


def first_mesen_apui(trace_path: Path, addr: int, value: int) -> dict | None:
    with trace_path.open() as fh:
        for line in fh:
            event = json.loads(line)
            if (
                event.get("kind") == "apui_write"
                and event.get("addr") == addr
                and event.get("value") == value
            ):
                return event
    return None


def count_mesen_events(trace_path: Path, kind: str) -> int:
    count = 0
    with trace_path.open() as fh:
        for line in fh:
            if json.loads(line).get("kind") == kind:
                count += 1
    return count


def write_rust_startup_trace(args: argparse.Namespace, output: Path) -> list[dict]:
    command = cargo_zelda(
        ["--trace-startup-audio", str(args.rom), str(args.frames), "--jsonl"],
        args.release,
    )
    result = run_command(command)
    require_success(result, "rust startup audio trace")
    output.write_text(result.stdout)
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]


def first_rust_port3_sfx(events: list[dict], value: int) -> dict | None:
    for event in events:
        ports = event.get("ports") or []
        if len(ports) >= 4 and ports[3] == value:
            return event
    return None


def run_mesen_gate(args: argparse.Namespace) -> None:
    if not args.mesen_runner.exists():
        raise GateFailure(f"Mesen runner not found: {args.mesen_runner}")
    args.work_dir.mkdir(parents=True, exist_ok=True)
    mesen_trace = args.work_dir / "mesen-startup-apui-dsp.jsonl"
    rust_trace = args.work_dir / "rust-startup-audio.jsonl"
    command = [
        str(args.mesen_runner),
        str(args.rom),
        str(args.frames),
        str(mesen_trace),
    ]
    result = run_command(command)
    require_success(result, "Mesen2 APUI/DSP trace")
    if not mesen_trace.exists():
        raise GateFailure(f"Mesen2 did not write trace: {mesen_trace}")

    rust_events = write_rust_startup_trace(args, rust_trace)
    mesen_sfx = first_mesen_apui(mesen_trace, 0x2143, 0x0A)
    rust_sfx = first_rust_port3_sfx(rust_events, 0x0A)
    dsp_addr_count = count_mesen_events(mesen_trace, "spc_dsp_addr")
    dsp_data_count = count_mesen_events(mesen_trace, "spc_dsp_data")
    if dsp_addr_count == 0 or dsp_data_count == 0:
        raise GateFailure(
            f"Mesen2 trace has no DSP activity: spc_dsp_addr={dsp_addr_count} "
            f"spc_dsp_data={dsp_data_count}; trace={mesen_trace}"
        )
    if mesen_sfx is None or rust_sfx is None:
        raise GateFailure(
            f"missing startup SFX $2143=$0a event: mesen={mesen_sfx} rust={rust_sfx}; "
            f"mesen_trace={mesen_trace} rust_trace={rust_trace}"
        )
    expected_mesen_frame = rust_sfx["frame"] + args.mesen_startup_offset
    if mesen_sfx["frame"] != expected_mesen_frame:
        raise GateFailure(
            "APUI command timing divergence for startup SFX $2143=$0a\n"
            f"mesen_frame={mesen_sfx['frame']} mesen_event={mesen_sfx.get('event')} "
            f"rust_frame={rust_sfx['frame']} expected_mesen_frame={expected_mesen_frame} "
            f"startup_offset={args.mesen_startup_offset} rust_ports={rust_sfx.get('ports')} "
            f"rust_peak={rust_sfx.get('peak')}\n"
            f"mesen_trace={mesen_trace}\nrust_trace={rust_trace}"
        )
    print(
        "Mesen2 APUI/DSP trace matched startup SFX timing: "
        f"mesen_frame={mesen_sfx['frame']} rust_frame={rust_sfx['frame']} "
        f"startup_offset={args.mesen_startup_offset} dsp_addr={dsp_addr_count} "
        f"dsp_data={dsp_data_count}",
        flush=True,
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--frames", type=int, default=180, help="startup frames for audio/video oracle traces")
    parser.add_argument("--lockstep-frames", type=int, default=300)
    parser.add_argument("--c-repo", type=Path, default=DEFAULT_C_REPO)
    parser.add_argument("--c-bin", type=Path, default=DEFAULT_C_REPO / "zelda3")
    parser.add_argument("--input-script")
    parser.add_argument("--load-sram")
    parser.add_argument("--load-state")
    parser.add_argument("--bsnes-core")
    parser.add_argument("--bsnes-url", default=DEFAULT_BSNES_URL)
    parser.add_argument("--bsnes-skip", type=int, default=83)
    parser.add_argument("--mesen-startup-offset", type=int, default=82)
    parser.add_argument("--mesen-runner", type=Path, default=DEFAULT_MESEN_RUNNER)
    parser.add_argument("--work-dir", type=Path, default=REPO_ROOT / "target" / "parity")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--no-lockstep", action="store_true")
    parser.add_argument("--no-c-audio", action="store_true")
    parser.add_argument("--no-bsnes", action="store_true")
    parser.add_argument("--with-bsnes", action="store_true")
    parser.add_argument("--no-install-bsnes", action="store_true")
    parser.add_argument("--no-mesen", action="store_true")
    parser.add_argument("--with-mesen", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    args.rom = args.rom.expanduser()
    args.c_repo = args.c_repo.expanduser()
    args.c_bin = args.c_bin.expanduser()
    args.mesen_runner = args.mesen_runner.expanduser()
    args.work_dir = args.work_dir.expanduser()
    if not args.rom.exists():
        print(f"ROM does not exist: {args.rom}", file=sys.stderr)
        return 2

    gates: list[tuple[str, object]] = []
    if not args.no_lockstep:
        gates.append(("lockstep behavior/render/audio", run_lockstep_gate))
    if not args.no_c_audio:
        gates.append(("C oracle audio", run_c_audio_gate))
    if args.with_mesen and not args.no_mesen:
        gates.append(("Mesen2 APUI/DSP timing", run_mesen_gate))
    if args.with_bsnes and not args.no_bsnes:
        gates.append(("bsnes exact audio/video", run_bsnes_gate))

    failures: list[str] = []
    for name, gate in gates:
        print(f"\n== {name} ==", flush=True)
        try:
            gate(args)
        except GateFailure as exc:
            failures.append(f"[{name}] {exc}")
            print(failures[-1], file=sys.stderr, flush=True)

    if failures:
        print("\nparity failed:", file=sys.stderr, flush=True)
        for failure in failures:
            print(f"\n{failure}", file=sys.stderr, flush=True)
        return 1
    print("\nall enabled parity gates passed", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
