#!/usr/bin/env python3
"""Materialize the pinned, instrumented Snes9x libretro parity oracle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


REVISION = "185488cd83aaf274752a742c94d45561cbecb7af"
SOURCE_URL = "https://github.com/libretro/snes9x.git"
EXPECTED_PATCH_PATHS = frozenset(
    {
        "apu/bapu/dsp/SPC_DSP.cpp",
        "apu/bapu/dsp/SPC_DSP.h",
        "apu/bapu/smp/core.cpp",
        "apu/bapu/smp/smp.cpp",
        "cpuexec.cpp",
        "dma.cpp",
        "getset.h",
        "gfx.cpp",
        "libretro/Makefile.common",
        "libretro/libretro.cpp",
        "ppu.cpp",
        "tileimpl-n1x1.cpp",
        "tileimpl-n2x1.cpp",
        "zelda3_trace.cpp",
        "zelda3_trace.h",
    }
)


def run(*args: str, cwd: Path | None = None, capture: bool = False) -> str:
    result = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout.rstrip("\n") if capture else ""


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def checkout_source(source: Path) -> None:
    if not (source / ".git").exists():
        source.parent.mkdir(parents=True, exist_ok=True)
        run("git", "clone", "--no-checkout", SOURCE_URL, str(source))
        run("git", "checkout", "--detach", REVISION, cwd=source)
    revision = run("git", "rev-parse", "HEAD", cwd=source, capture=True)
    if revision != REVISION:
        raise RuntimeError(f"expected Snes9x revision {REVISION}, got {revision}")


def changed_paths(source: Path) -> set[str]:
    output = run(
        "git",
        "status",
        "--porcelain",
        "--untracked-files=all",
        cwd=source,
        capture=True,
    )
    paths: set[str] = set()
    for line in output.splitlines():
        if not line:
            continue
        path = line[3:]
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        paths.add(path)
    return paths


def apply_trace_patch(source: Path, patch: Path) -> None:
    changes = changed_paths(source)
    if not changes:
        run("git", "apply", "--check", str(patch), cwd=source)
        run("git", "apply", str(patch), cwd=source)
        changes = changed_paths(source)

    unexpected = changes - EXPECTED_PATCH_PATHS
    missing = EXPECTED_PATCH_PATHS - changes
    if unexpected or missing:
        details = []
        if unexpected:
            details.append(f"unexpected changes: {', '.join(sorted(unexpected))}")
        if missing:
            details.append(f"missing trace changes: {', '.join(sorted(missing))}")
        raise RuntimeError("Snes9x trace checkout is not canonical: " + "; ".join(details))

    run("git", "apply", "--reverse", "--check", str(patch), cwd=source)


def build_settings() -> tuple[str, str]:
    if sys.platform == "darwin":
        return "osx", "snes9x_libretro.dylib"
    if sys.platform.startswith("linux"):
        return "unix", "snes9x_libretro.so"
    raise RuntimeError(f"unsupported traced-oracle build platform: {sys.platform}")


def write_receipt(output: Path, patch: Path) -> Path:
    receipt = output.with_suffix(output.suffix + ".json")
    payload = {
        "schema": 1,
        "source_url": SOURCE_URL,
        "source_revision": REVISION,
        "patch": str(patch),
        "patch_sha256": sha256(patch),
        "core": str(output),
        "core_sha256": sha256(output),
    }
    receipt.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    return receipt


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    default_source = repo_root / "external" / "snes9x-libretro" / "source"
    default_output_dir = repo_root / "external" / "snes9x-libretro" / "local"
    patch = repo_root / "external" / "snes9x-libretro" / "patches" / "zelda3-trace.patch"

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=default_source)
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--prepare-only",
        action="store_true",
        help="clone, pin, and patch the source without compiling",
    )
    args = parser.parse_args()

    source = args.source.resolve()
    checkout_source(source)
    apply_trace_patch(source, patch.resolve())
    print(f"prepared Snes9x {REVISION} with {patch.relative_to(repo_root)}")
    if args.prepare_only:
        return 0

    platform, artifact_name = build_settings()
    jobs = str(max(1, os.cpu_count() or 1))
    run(
        "make",
        "-C",
        "libretro",
        f"platform={platform}",
        "LTO=",
        f"-j{jobs}",
        artifact_name,
        cwd=source,
    )
    artifact = source / "libretro" / artifact_name
    traced_artifact_name = artifact_name.replace("_libretro.", "_libretro_trace.")
    output = (args.output or (default_output_dir / traced_artifact_name)).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(artifact, output)
    receipt = write_receipt(output, patch.resolve())
    print(f"traced core: {output}")
    print(f"build receipt: {receipt}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError, RuntimeError) as error:
        raise SystemExit(f"error: {error}") from error
