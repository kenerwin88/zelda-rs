#!/usr/bin/env python3
"""Run the automatic Zelda 3 parity gates and report first useful diffs."""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
import urllib.error
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT / "saves" / "zelda3.sfc")))
DEFAULT_MESEN_RUNNER = REPO_ROOT / "external" / "mesen2-oracle" / "run_trace.sh"
DEFAULT_SNES9X_LOCAL = (
    REPO_ROOT / "external" / "snes9x-libretro" / "local" / "snes9x_libretro.dylib"
)
DEFAULT_SNES9X_URL = os.environ.get(
    "SNES9X_LIBRETRO_URL",
    "https://buildbot.libretro.com/nightly/apple/osx/arm64/latest/snes9x_libretro.dylib.zip",
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
    env = os.environ.copy()
    repo_asset_pack = REPO_ROOT / "zelda3_assets.dat"
    if repo_asset_pack.exists():
        env.setdefault("ZELDA3_ASSET_PACK", str(repo_asset_pack))
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    return CommandResult(command, result.returncode, result.stdout, result.stderr)


def command_text(command: list[str]) -> str:
    return " ".join(command)


def cargo_zelda(
    args: list[str], release: bool
) -> list[str]:
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
        f"command: {command_text(result.command)}\n" + "\n".join(tail)
    )


def find_snes9x_core(explicit: str | None) -> Path | None:
    if explicit:
        path = Path(explicit).expanduser()
        return path if path.exists() else None
    env_path = os.environ.get("SNES9X_LIBRETRO_CORE")
    if env_path:
        path = Path(env_path).expanduser()
        if path.exists():
            return path
    candidates = [
        DEFAULT_SNES9X_LOCAL,
        REPO_ROOT / "external" / "snes9x-libretro" / "snes9x_libretro.dylib",
        Path("/private/tmp/snes9x_libretro/snes9x_libretro.dylib"),
        Path.home()
        / "Library/Application Support/RetroArch/cores/snes9x_libretro.dylib",
        Path(
            "/Applications/RetroArch.app/Contents/Resources/cores/snes9x_libretro.dylib"
        ),
    ]
    for path in candidates:
        if path.exists():
            return path
    return None


def download_snes9x_core(url: str) -> Path:
    if platform.system() != "Darwin" or platform.machine() not in {"arm64", "aarch64"}:
        raise GateFailure(
            "automatic Snes9x download is currently configured for macOS arm64. "
            "Pass --snes9x-core or set SNES9X_LIBRETRO_CORE for this platform."
        )
    DEFAULT_SNES9X_LOCAL.parent.mkdir(parents=True, exist_ok=True)
    zip_path = DEFAULT_SNES9X_LOCAL.with_suffix(".dylib.zip")
    core_tmp = DEFAULT_SNES9X_LOCAL.with_suffix(".dylib.tmp")
    print(f"downloading Snes9x libretro core: {url}", flush=True)
    try:
        with (
            urllib.request.urlopen(url, timeout=60) as source,
            zip_path.open("wb") as dest,
        ):
            shutil.copyfileobj(source, dest)
        with zipfile.ZipFile(zip_path) as archive:
            member = next(
                (
                    name
                    for name in archive.namelist()
                    if name.endswith("snes9x_libretro.dylib")
                ),
                None,
            )
            if member is None:
                raise GateFailure(
                    "downloaded archive does not contain snes9x_libretro.dylib: "
                    f"{zip_path}"
                )
            with archive.open(member) as source, core_tmp.open("wb") as dest:
                shutil.copyfileobj(source, dest)
        core_tmp.replace(DEFAULT_SNES9X_LOCAL)
    except (OSError, urllib.error.URLError, zipfile.BadZipFile) as error:
        raise GateFailure(
            f"failed to download Snes9x libretro core from {url}: {error}"
        ) from error
    finally:
        zip_path.unlink(missing_ok=True)
        core_tmp.unlink(missing_ok=True)
    DEFAULT_SNES9X_LOCAL.chmod(0o755)
    return DEFAULT_SNES9X_LOCAL


def resolve_snes9x_core(args: argparse.Namespace) -> Path | None:
    core = find_snes9x_core(args.snes9x_core)
    if core is not None or args.no_install_snes9x:
        return core
    return download_snes9x_core(args.snes9x_url)


def run_snes9x_gate(args: argparse.Namespace) -> None:
    core = resolve_snes9x_core(args)
    if core is None:
        raise GateFailure(
            "Snes9x libretro core not found. Set SNES9X_LIBRETRO_CORE or pass "
            "--snes9x-core /path/to/snes9x_libretro.dylib. Live A/V parity cannot run without it."
        )
    session_dir = args.work_dir / "snes9x-session"
    command = cargo_zelda(
        [
            "--compare-snes9x-oracle",
            str(core),
            str(args.rom),
            str(args.frames),
            "--skip-oracle-frames",
            str(args.snes9x_skip),
            "--audio-comparison",
            "exact",
            "--session-dir",
            str(session_dir),
            "--scan-all",
        ],
        args.release,
    )
    if args.input_script:
        command.extend(["--input-script", args.input_script])
    if args.load_sram:
        command.extend(["--load-sram", args.load_sram])
    result = run_command(command)
    require_success(result, "Snes9x live audio/video")
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
    parser.add_argument(
        "--frames",
        type=int,
        default=180,
        help="startup frames for audio/video oracle traces",
    )
    parser.add_argument("--input-script")
    parser.add_argument("--load-sram")
    parser.add_argument("--load-state")
    parser.add_argument("--snes9x-core")
    parser.add_argument("--snes9x-url", default=DEFAULT_SNES9X_URL)
    parser.add_argument("--snes9x-skip", type=int, default=0)
    parser.add_argument("--mesen-startup-offset", type=int, default=82)
    parser.add_argument("--mesen-runner", type=Path, default=DEFAULT_MESEN_RUNNER)
    parser.add_argument(
        "--work-dir", type=Path, default=REPO_ROOT / "target" / "parity"
    )
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--no-snes9x", action="store_true")
    parser.add_argument("--with-snes9x", action="store_true")
    parser.add_argument("--no-install-snes9x", action="store_true")
    parser.add_argument("--no-mesen", action="store_true")
    parser.add_argument("--with-mesen", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    args.rom = args.rom.expanduser()
    args.mesen_runner = args.mesen_runner.expanduser()
    args.work_dir = args.work_dir.expanduser()
    if not args.rom.exists():
        print(f"ROM does not exist: {args.rom}", file=sys.stderr)
        return 2

    gates: list[tuple[str, object]] = []
    if args.with_mesen and not args.no_mesen:
        gates.append(("Mesen2 APUI/DSP timing", run_mesen_gate))
    if args.with_snes9x and not args.no_snes9x:
        gates.append(("Snes9x live audio/video", run_snes9x_gate))

    if not gates:
        print("no parity gates are enabled", file=sys.stderr)
        return 2

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
