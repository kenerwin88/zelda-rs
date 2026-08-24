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


REPO_ROOT = Path(__file__).resolve().parents[1]
LOCK_PATH = REPO_ROOT / "external" / "snes9x-libretro" / "oracle-lock.json"
PATCH_DIR = REPO_ROOT / "external" / "snes9x-libretro" / "patches"
TRACE_PATCHES = (
    PATCH_DIR / "zelda3-trace.patch",
    PATCH_DIR / "zelda3-trace-obj-cache.patch",
    PATCH_DIR / "zelda3-spc-opcode-ledger.patch",
    PATCH_DIR / "zelda3-dsp-phase-ledger.patch",
)
LOCK = json.loads(LOCK_PATH.read_text())
VERSION = LOCK["core_version"]
SOURCE_TAG = LOCK["source_tag"]
REVISION = LOCK["source_revision"]
SOURCE_URL = LOCK["source_url"]
EXPECTED_PATCH_PATHS = frozenset(
    {
        "apu/apu.cpp",
        "apu/apu.h",
        "apu/bapu/dsp/SPC_DSP.cpp",
        "apu/bapu/dsp/SPC_DSP.h",
        "apu/bapu/smp/core.cpp",
        "apu/bapu/smp/memory.cpp",
        "apu/bapu/smp/smp.cpp",
        "cpuexec.cpp",
        "cpuops.cpp",
        "dma.cpp",
        "getset.h",
        "gfx.cpp",
        "libretro/Makefile.common",
        "libretro/libretro.cpp",
        "ppu.cpp",
        "tileimpl.h",
        "tileimpl-n1x1.cpp",
        "tileimpl-n2x1.cpp",
        "zelda3_trace.cpp",
        "zelda3_trace.h",
    }
)
IGNORED_LOCAL_PATH_PREFIXES = ("target/",)


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


def patch_set_sha256(patches: tuple[Path, ...]) -> str | None:
    if not patches:
        return None
    if len(patches) == 1:
        return sha256(patches[0])
    digest = hashlib.sha256()
    for patch in patches:
        digest.update(patch.name.encode("utf-8"))
        digest.update(b"\0")
        digest.update(patch.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def checkout_source(source: Path) -> None:
    if not (source / ".git").exists():
        source.parent.mkdir(parents=True, exist_ok=True)
        run("git", "clone", "--no-checkout", SOURCE_URL, str(source))
        run("git", "checkout", "--detach", REVISION, cwd=source)
    revision = run("git", "rev-parse", "HEAD", cwd=source, capture=True)
    if revision != REVISION:
        if changed_paths(source):
            raise RuntimeError(
                "Snes9x source has local changes at the wrong revision; "
                f"expected {REVISION}, got {revision}"
            )
        run("git", "checkout", "--detach", REVISION, cwd=source)
    tagged_revision = run(
        "git",
        "rev-parse",
        f"refs/tags/{SOURCE_TAG}^{{commit}}",
        cwd=source,
        capture=True,
    )
    if tagged_revision != REVISION:
        raise RuntimeError(
            f"Snes9x tag {SOURCE_TAG} resolves to {tagged_revision}, expected {REVISION}"
        )
    version_header = (source / "snes9x.h").read_text()
    if f'#define VERSION\t"{VERSION}"' not in version_header:
        raise RuntimeError(
            f"Snes9x source revision {REVISION} does not declare version {VERSION}"
        )


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
        if not any(path.startswith(prefix) for prefix in IGNORED_LOCAL_PATH_PREFIXES):
            paths.add(path)
    return paths


def apply_trace_patches(source: Path, patches: tuple[Path, ...]) -> None:
    changes = changed_paths(source)
    if changes:
        raise RuntimeError("Snes9x trace patches require a pristine source checkout")
    for patch in patches:
        run("git", "apply", "--recount", "--check", str(patch), cwd=source)
        run("git", "apply", "--recount", str(patch), cwd=source)
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

def restore_stock_source(source: Path, patches: tuple[Path, ...]) -> None:
    changes = changed_paths(source)
    if not changes:
        return
    if changes != EXPECTED_PATCH_PATHS:
        unexpected = ", ".join(sorted(changes - EXPECTED_PATCH_PATHS))
        raise RuntimeError(
            "Snes9x source has changes outside the trace patch"
            + (f": {unexpected}" if unexpected else "")
        )
    for patch in reversed(patches):
        run("git", "apply", "--recount", "--reverse", "--check", str(patch), cwd=source)
        run("git", "apply", "--recount", "--reverse", str(patch), cwd=source)
    remaining = changed_paths(source)
    if remaining:
        raise RuntimeError(
            "failed to restore pristine Snes9x source: "
            + ", ".join(sorted(remaining))
        )


def build_settings() -> tuple[str, str]:
    if sys.platform == "darwin":
        return "osx", "snes9x_libretro.dylib"
    if sys.platform.startswith("linux"):
        return "unix", "snes9x_libretro.so"
    raise RuntimeError(f"unsupported traced-oracle build platform: {sys.platform}")


def write_receipt(
    output: Path,
    *,
    variant: str,
    patches: tuple[Path, ...],
) -> Path:
    receipt = output.with_suffix(output.suffix + ".json")
    payload = {
        "schema": 1,
        "core_name": LOCK["core_name"],
        "core_version": VERSION,
        "source_tag": SOURCE_TAG,
        "source_url": SOURCE_URL,
        "source_revision": REVISION,
        "oracle_lock_sha256": sha256(LOCK_PATH),
        "variant": variant,
        "patch": str(patches[0]) if patches else None,
        "patches": [str(patch) for patch in patches],
        "patch_sha256s": [sha256(patch) for patch in patches],
        "patch_sha256": patch_set_sha256(patches),
        "core": str(output),
        "core_sha256": sha256(output),
    }
    receipt.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    return receipt


def main() -> int:
    repo_root = REPO_ROOT
    default_source = repo_root / "external" / "snes9x-libretro" / "source"
    default_output_dir = repo_root / "external" / "snes9x-libretro" / "local"
    patches = tuple(patch.resolve() for patch in TRACE_PATCHES)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=default_source)
    parser.add_argument("--output", type=Path, help="instrumented core output")
    parser.add_argument("--stock-output", type=Path, help="stock core output")
    parser.add_argument(
        "--prepare-only",
        action="store_true",
        help="clone, pin, and patch the source without compiling",
    )
    args = parser.parse_args()

    source = args.source.resolve()
    checkout_source(source)
    restore_stock_source(source, patches)
    print(f"prepared stock Snes9x {VERSION} ({SOURCE_TAG}, {REVISION})")
    if args.prepare_only:
        apply_trace_patches(source, patches)
        for patch in patches:
            print(f"applied {patch.relative_to(repo_root)}")
        return 0

    platform, artifact_name = build_settings()
    jobs = str(max(1, os.cpu_count() or 1))
    build_arguments = [
        "make",
        "-C",
        "libretro",
        f"platform={platform}",
        "LTO=",
    ]

    def build(output: Path, *, variant: str, receipt_patches: tuple[Path, ...]) -> None:
        run(*build_arguments, "clean", cwd=source)
        run(*build_arguments, f"-j{jobs}", artifact_name, cwd=source)
        artifact = source / "libretro" / artifact_name
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(artifact, output)
        receipt = write_receipt(
            output,
            variant=variant,
            patches=receipt_patches,
        )
        print(f"{variant} core: {output}")
        print(f"{variant} build receipt: {receipt}")

    stock_output = (
        args.stock_output or (default_output_dir / artifact_name)
    ).resolve()
    build(stock_output, variant="stock", receipt_patches=())

    apply_trace_patches(source, patches)
    traced_artifact_name = artifact_name.replace("_libretro.", "_libretro_trace.")
    output = (args.output or (default_output_dir / traced_artifact_name)).resolve()
    build(output, variant="trace", receipt_patches=patches)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError, RuntimeError) as error:
        raise SystemExit(f"error: {error}") from error
