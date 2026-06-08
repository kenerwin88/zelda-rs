#!/usr/bin/env python3
"""Collect C/Rust function coverage from replay-save routes.

Coverage answers a different question than replay parity:

* coverage: which C and Rust functions did this route execute?
* parity: did the executed code produce identical state and output?

Use this script to find blind spots, then use replay_bisect.py/checkpoints to
prove behavior.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = ROOT.parent / "zelda3"
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", DEFAULT_C_ROOT / "zelda3.sfc"))
DEFAULT_SAVE = ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_OUTPUT = ROOT / "target" / "replay-coverage"
DEFAULT_FRAMES = 1_073_092
TIMING_HACK_ENV = {
    "ZELDA3_SMV_SELECT_FILE_TIMING_HACKS": "1",
    "ZELDA3_SMV_LOADFILE_TIMING_HACKS": "1",
    "ZELDA3_SMV_DUNGEON_TIMING_HACKS": "1",
    "ZELDA3_SMV_OVERWORLD_TIMING_HACKS": "1",
    "ZELDA3_SMV_MESSAGING_TIMING_HACKS": "1",
    "ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS": "1",
    "ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS": "1",
}
HEADLESS_ENV = {
    "SDL_VIDEODRIVER": "dummy",
    "SDL_AUDIODRIVER": "dummy",
    "SDL_RENDER_DRIVER": "software",
}


@dataclass(frozen=True)
class Window:
    name: str
    frames: int


def run(
    command: list[str],
    *,
    cwd: Path,
    env: dict[str, str] | None = None,
    check: bool = True,
    stdout: object | None = None,
) -> subprocess.CompletedProcess[str]:
    print("+ " + " ".join(command), flush=True)
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=stdout if stdout is not None else subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if check and result.returncode != 0:
        if stdout is None:
            print("\n".join((result.stdout or "").splitlines()[-80:]), file=sys.stderr)
        raise SystemExit(f"command failed with exit code {result.returncode}: {' '.join(command)}")
    return result


def resolve_tool(name: str) -> str:
    path = shutil.which(name)
    if path:
        return path
    xcrun = shutil.which("xcrun")
    if xcrun:
        result = subprocess.run(
            [xcrun, "--find", name],
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip()
    raise SystemExit(f"required LLVM tool not found: {name}")


def parse_window(text: str) -> Window:
    if ":" not in text:
        return Window("route", int(text, 0))
    name, frames = text.split(":", 1)
    if not name:
        raise argparse.ArgumentTypeError("window name must not be empty")
    return Window(name, int(frames, 0))


def rust_binary_path(target_dir: Path, release: bool) -> Path:
    profile = "release" if release else "debug"
    return target_dir / profile / "zelda3"


def build_rust(args: argparse.Namespace, rust_target: Path) -> Path:
    env = os.environ.copy()
    env["RUSTFLAGS"] = " ".join(
        part for part in [env.get("RUSTFLAGS", ""), "-Cinstrument-coverage"] if part
    )
    env["CARGO_TARGET_DIR"] = str(rust_target)
    command = ["cargo", "build", "-p", "zelda3-bin"]
    if args.release:
        command.append("--release")
    run(command, cwd=ROOT, env=env)
    binary = rust_binary_path(rust_target, args.release)
    if not binary.exists():
        raise SystemExit(f"Rust coverage binary not found after build: {binary}")
    return binary


def build_c(args: argparse.Namespace) -> Path:
    clang = shutil.which("clang") or "clang"
    sdl_cflags = subprocess.run(
        ["sdl2-config", "--cflags"],
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if sdl_cflags.returncode != 0:
        raise SystemExit("sdl2-config --cflags failed; C coverage build needs SDL2 development flags")
    cflags = (
        "-O0 -g -fprofile-instr-generate -fcoverage-mapping "
        "-Wno-error -I . "
        f"{sdl_cflags.stdout.strip()} "
        "-DSYSTEM_VOLUME_MIXER_AVAILABLE=0"
    )
    ldflags = "-fprofile-instr-generate -fcoverage-mapping"
    if args.rebuild_c:
        run(["make", "clean_obj"], cwd=args.c_root)
    run(
        [
            "make",
            f"CC={clang}",
            f"CFLAGS={cflags}",
            f"LDFLAGS={ldflags}",
            "-j8",
        ],
        cwd=args.c_root,
    )
    binary = args.c_root / "zelda3"
    if not binary.exists():
        raise SystemExit(f"C coverage binary not found after build: {binary}")
    return binary


def run_c_window(args: argparse.Namespace, binary: Path, window: Window, profraw_dir: Path) -> None:
    env = os.environ.copy()
    env.update(HEADLESS_ENV)
    env["LLVM_PROFILE_FILE"] = str(profraw_dir / f"c-{window.name}-%m-%p.profraw")
    run(
        [
            str(binary),
            "--config",
            str(args.c_root / "other" / "headless_replay.ini"),
            "--replay-save",
            str(args.save),
            "--smv-test-frames",
            str(window.frames),
        ],
        cwd=args.c_root,
        env=env,
    )


def run_rust_window(args: argparse.Namespace, binary: Path, window: Window, profraw_dir: Path) -> None:
    env = os.environ.copy()
    env.update(TIMING_HACK_ENV)
    env["LLVM_PROFILE_FILE"] = str(profraw_dir / f"rust-{window.name}-%m-%p.profraw")
    run(
        [
            str(binary),
            "--replay-save",
            str(args.rom),
            str(args.save),
            str(window.frames),
        ],
        cwd=ROOT,
        env=env,
    )


def merge_profiles(profdata: str, profraw_dir: Path, prefix: str, output: Path) -> bool:
    raw_files = sorted(profraw_dir.glob(f"{prefix}-*.profraw"))
    if not raw_files:
        print(f"no {prefix} profraw files found in {profraw_dir}", file=sys.stderr)
        return False
    run([profdata, "merge", "-sparse", *map(str, raw_files), "-o", str(output)], cwd=ROOT)
    return True


def export_coverage(
    llvm_cov: str,
    binary: Path,
    profdata: Path,
    output_dir: Path,
    ignore_regex: str,
) -> Path:
    html_dir = output_dir / "html"
    html_dir.mkdir(parents=True, exist_ok=True)
    run(
        [
            llvm_cov,
            "show",
            str(binary),
            f"-instr-profile={profdata}",
            "--format=html",
            f"--output-dir={html_dir}",
            f"--ignore-filename-regex={ignore_regex}",
        ],
        cwd=ROOT,
    )
    lcov_path = output_dir / "coverage.lcov"
    with lcov_path.open("w") as fh:
        run(
            [
                llvm_cov,
                "export",
                str(binary),
                f"-instr-profile={profdata}",
                "--format=lcov",
                f"--ignore-filename-regex={ignore_regex}",
            ],
            cwd=ROOT,
            stdout=fh,
        )
    json_path = output_dir / "coverage.json"
    with json_path.open("w") as fh:
        run(
            [
                llvm_cov,
                "export",
                str(binary),
                f"-instr-profile={profdata}",
                "--format=text",
                f"--ignore-filename-regex={ignore_regex}",
            ],
            cwd=ROOT,
            stdout=fh,
        )
    return json_path


def function_entries(json_path: Path) -> dict[str, int]:
    try:
        payload = json.loads(json_path.read_text())
    except (OSError, json.JSONDecodeError):
        return {}
    entries: dict[str, int] = {}
    for data in payload.get("data", []):
        for file_entry in data.get("files", []):
            for function in file_entry.get("functions", []):
                name = function.get("name")
                if not name:
                    continue
                count = int(function.get("count") or 0)
                entries[name] = entries.get(name, 0) + count
    return entries


def normalize_function(name: str) -> str:
    name = re.sub(r"^.*::", "", name)
    name = re.sub(r"^<.* as .*>::", "", name)
    name = re.sub(r"[^A-Za-z0-9]", "", name)
    return name.lower()


def write_function_tsv(path: Path, functions: dict[str, int]) -> None:
    with path.open("w") as fh:
        fh.write("covered\tcount\tfunction\n")
        for name, count in sorted(functions.items(), key=lambda item: (item[1] == 0, item[0])):
            fh.write(f"{1 if count else 0}\t{count}\t{name}\n")


def write_matrix(
    output: Path,
    windows: list[Window],
    c_functions: dict[str, int],
    rust_functions: dict[str, int],
) -> None:
    c_by_norm: dict[str, list[tuple[str, int]]] = {}
    rust_by_norm: dict[str, list[tuple[str, int]]] = {}
    for name, count in c_functions.items():
        c_by_norm.setdefault(normalize_function(name), []).append((name, count))
    for name, count in rust_functions.items():
        rust_by_norm.setdefault(normalize_function(name), []).append((name, count))

    c_covered = {key for key, values in c_by_norm.items() if any(count > 0 for _, count in values)}
    rust_covered = {key for key, values in rust_by_norm.items() if any(count > 0 for _, count in values)}
    paired = sorted(set(c_by_norm) & set(rust_by_norm))
    c_only_covered = sorted(c_covered - set(rust_by_norm))
    rust_only_covered = sorted(rust_covered - set(c_by_norm))
    mismatched = [
        key
        for key in paired
        if (key in c_covered) != (key in rust_covered)
    ]

    def display(values: list[tuple[str, int]]) -> str:
        covered = [f"{name}({count})" for name, count in values if count > 0]
        return ", ".join(covered[:3]) if covered else values[0][0]

    matrix = output / "parity_matrix.md"
    with matrix.open("w") as fh:
        fh.write("# Replay Coverage Matrix\n\n")
        fh.write("Coverage shows executed code, not parity. Use replay_bisect.py to prove matching state.\n\n")
        fh.write("## Replay Windows\n\n")
        fh.write("| Window | Frames |\n|---|---:|\n")
        for window in windows:
            fh.write(f"| {window.name} | {window.frames} |\n")
        fh.write("\n## Summary\n\n")
        fh.write(f"- C functions in report: {len(c_functions)}\n")
        fh.write(f"- Rust functions in report: {len(rust_functions)}\n")
        fh.write(f"- mapped functions with coverage mismatch: {len(mismatched)}\n")
        fh.write(f"- covered C functions without normalized Rust match: {len(c_only_covered)}\n")
        fh.write(f"- covered Rust functions without normalized C match: {len(rust_only_covered)}\n\n")

        fh.write("## Mapped Coverage Mismatches\n\n")
        fh.write("| Normalized function | C | Rust |\n|---|---|---|\n")
        for key in mismatched[:200]:
            fh.write(f"| {key} | {display(c_by_norm[key])} | {display(rust_by_norm[key])} |\n")
        if not mismatched:
            fh.write("| none |  |  |\n")

        fh.write("\n## Covered C Without Rust Match\n\n")
        fh.write("| Normalized function | C function |\n|---|---|\n")
        for key in c_only_covered[:200]:
            fh.write(f"| {key} | {display(c_by_norm[key])} |\n")
        if not c_only_covered:
            fh.write("| none |  |\n")

        fh.write("\n## Covered Rust Without C Match\n\n")
        fh.write("| Normalized function | Rust function |\n|---|---|\n")
        for key in rust_only_covered[:200]:
            fh.write(f"| {key} | {display(rust_by_norm[key])} |\n")
        if not rust_only_covered:
            fh.write("| none |  |\n")


def write_readme(output: Path, args: argparse.Namespace, windows: list[Window]) -> None:
    readme = output / "README.md"
    with readme.open("w") as fh:
        fh.write("# Replay Coverage Output\n\n")
        fh.write(f"- C root: `{args.c_root}`\n")
        fh.write(f"- Rust root: `{ROOT}`\n")
        fh.write(f"- ROM: `{args.rom}`\n")
        fh.write(f"- Replay save: `{args.save}`\n")
        fh.write("- Windows: " + ", ".join(f"{w.name}:{w.frames}" for w in windows) + "\n\n")
        fh.write("Open `c/html/index.html` and `rust/html/index.html` for source coverage.\n")
        fh.write("Use `parity_matrix.md` for a first pass at mapped function blind spots.\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--c-root", type=Path, default=DEFAULT_C_ROOT)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--window", action="append", type=parse_window, help="name:frames or frames; may repeat")
    parser.add_argument("--frames", type=int, default=DEFAULT_FRAMES, help="default route frames when --window is omitted")
    parser.add_argument("--release", action="store_true", help="build Rust coverage with release profile")
    parser.add_argument("--rebuild-c", action="store_true", help="run make clean_obj before C coverage build")
    parser.add_argument("--skip-c", action="store_true")
    parser.add_argument("--skip-rust", action="store_true")
    parser.add_argument("--no-run", action="store_true", help="build/export setup only; do not execute replay windows")
    args = parser.parse_args()

    args.c_root = args.c_root.expanduser().resolve()
    args.rom = args.rom.expanduser().resolve()
    args.save = args.save.expanduser().resolve()
    args.output_dir = args.output_dir.expanduser().resolve()
    windows = args.window or [Window("combined-route", args.frames)]
    args.output_dir.mkdir(parents=True, exist_ok=True)
    profraw_dir = args.output_dir / "profraw"
    profraw_dir.mkdir(parents=True, exist_ok=True)

    profdata = resolve_tool("llvm-profdata")
    llvm_cov = resolve_tool("llvm-cov")

    c_json = None
    rust_json = None
    if not args.skip_c:
        c_binary = build_c(args)
        if not args.no_run:
            for window in windows:
                run_c_window(args, c_binary, window, profraw_dir)
        c_profdata = args.output_dir / "c.profdata"
        if merge_profiles(profdata, profraw_dir, "c", c_profdata):
            c_json = export_coverage(
                llvm_cov,
                c_binary,
                c_profdata,
                args.output_dir / "c",
                r"third_party|snes|/usr/include|SDL",
            )

    if not args.skip_rust:
        rust_target = args.output_dir / "rust-target"
        rust_binary = build_rust(args, rust_target)
        if not args.no_run:
            for window in windows:
                run_rust_window(args, rust_binary, window, profraw_dir)
        rust_profdata = args.output_dir / "rust.profdata"
        if merge_profiles(profdata, profraw_dir, "rust", rust_profdata):
            rust_json = export_coverage(
                llvm_cov,
                rust_binary,
                rust_profdata,
                args.output_dir / "rust",
                r"/rustc/|/.cargo/|registry/src",
            )

    c_functions = function_entries(c_json) if c_json else {}
    rust_functions = function_entries(rust_json) if rust_json else {}
    write_function_tsv(args.output_dir / "c-functions.tsv", c_functions)
    write_function_tsv(args.output_dir / "rust-functions.tsv", rust_functions)
    write_matrix(args.output_dir, windows, c_functions, rust_functions)
    write_readme(args.output_dir, args, windows)

    print(f"coverage output: {args.output_dir}")
    print(f"matrix: {args.output_dir / 'parity_matrix.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
