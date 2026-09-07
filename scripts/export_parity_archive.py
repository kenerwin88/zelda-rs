#!/usr/bin/env python3
"""Export everything needed to re-prove the promoted full-route parity result.

The proof of parity lives in artifacts that git does not carry: the Snes9x
oracle A/V cache under `.git/parity-oracle-cache/`, the pinned instrumented
core, the promoted binary and its WRAM goldens. This script gathers them, plus
a complete `git bundle` of the repository, into one self-describing directory
with a SHA-256 manifest so the whole result can be copied off the machine and
re-verified later with `--verify`.

    scripts/export_parity_archive.py [--dest DIR] [--binary PATH]
    scripts/export_parity_archive.py --verify ARCHIVE_DIR

The ROM is deliberately NOT copied (only its SHA-256 is recorded); it must be
supplied separately when re-verifying.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "routes" / "full_run" / "parity-frontier.json"
CACHE_ROOT = ROOT / ".git" / "parity-oracle-cache"
CORE_DIRS = (
    ROOT / "external" / "snes9x-libretro" / "local",
    ROOT / "external" / "snes9x-libretro" / "source" / "libretro",
)
GOLDEN_WRAM = ROOT / "routes" / "full_run" / "golden" / "wram"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def copy_tree(source: Path, dest: Path) -> None:
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(source, dest)


def find_core(sha: str) -> Path | None:
    for directory in CORE_DIRS:
        if not directory.is_dir():
            continue
        for candidate in sorted(directory.glob("*.dylib")) + sorted(directory.glob("*.so")):
            if sha256_file(candidate) == sha:
                return candidate
    return None


def export(dest_root: Path, binary: Path | None, label: str | None) -> Path:
    if not LEDGER.is_file():
        raise SystemExit(f"parity archive: ledger is missing: {LEDGER}")
    ledger = json.loads(LEDGER.read_text(encoding="utf-8"))
    promoted = ledger.get("promoted") or {}
    receipt = promoted.get("cached_av_receipt") or {}
    commit = str(promoted.get("commit") or git("rev-parse", "HEAD"))
    cache_key = receipt.get("oracle_cache_key")
    if not cache_key:
        raise SystemExit("parity archive: the ledger has no promoted cached-av receipt")
    cache_dir = CACHE_ROOT / cache_key
    if not cache_dir.is_dir():
        raise SystemExit(f"parity archive: oracle cache is missing: {cache_dir}")
    cache_manifest = json.loads((cache_dir / "cache-manifest.json").read_text(encoding="utf-8"))
    if sha256_file(cache_dir / "cache-manifest.json") != receipt.get("oracle_cache_manifest_sha256"):
        raise SystemExit("parity archive: the oracle cache manifest no longer matches the ledger")
    identity = cache_manifest.get("cache_identity") or {}

    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    name = label or f"{stamp}-{commit[:12]}"
    archive = dest_root / name
    if archive.exists():
        raise SystemExit(f"parity archive: destination already exists: {archive}")
    archive.mkdir(parents=True)
    notes: list[str] = []

    # 1. The oracle cache: pinned Snes9x per-frame RGB/audio hashes, input, SRAM,
    #    recorded cartridge RNG, host receipts. This is the ground truth.
    copy_tree(cache_dir, archive / "oracle-cache" / cache_key)

    # 2. Complete repository history (every branch and tag).
    subprocess.run(
        ["git", "bundle", "create", str(archive / "repo.bundle"), "--all"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )

    # 3. Route project as committed (boundaries, takes, ledger, receipts).
    with (archive / "routes-full_run.tar").open("wb") as handle:
        subprocess.run(
            ["git", "archive", "--format=tar", commit, "routes/full_run"],
            cwd=ROOT,
            check=True,
            stdout=handle,
        )
    shutil.copyfile(LEDGER, archive / "parity-frontier.json")

    # 4. The pinned core that produced the cache.
    core_sha = identity.get("core_sha256")
    core = find_core(core_sha) if core_sha else None
    if core is None:
        notes.append(f"pinned core {core_sha} was not found locally; the cache still binds it")
    else:
        shutil.copyfile(core, archive / f"snes9x_libretro_trace-{core_sha[:12]}{core.suffix}")

    # 5. The promoted binary.
    binary_sha = promoted.get("binary_sha256")
    if binary is not None:
        if binary_sha and sha256_file(binary) != binary_sha:
            raise SystemExit(
                f"parity archive: {binary} is not the promoted binary {binary_sha[:12]}"
            )
        shutil.copyfile(binary, archive / "zelda3-promoted")
        (archive / "zelda3-promoted").chmod(0o755)
    else:
        notes.append("promoted binary not supplied (--binary); rebuild from the commit")

    # 6. WRAM goldens (Rust-only regression baseline).
    if GOLDEN_WRAM.is_dir():
        copy_tree(GOLDEN_WRAM, archive / "golden-wram")

    manifest = {
        "kind": "zelda3-rs-parity-archive",
        "schema": 1,
        "created_utc": stamp,
        "commit": commit,
        "head_at_export": git("rev-parse", "HEAD"),
        "promoted": promoted,
        "oracle_cache_key": cache_key,
        "cache_identity": identity,
        "rom": {
            "included": False,
            "sha256": identity.get("rom_sha256"),
            "note": "supply saves/zelda3.sfc with this SHA-256 to re-verify",
        },
        "reverify": [
            "git clone repo.bundle zelda3-rs && cd zelda3-rs && git checkout <commit>",
            "mkdir -p .git/parity-oracle-cache && cp -R <archive>/oracle-cache/* .git/parity-oracle-cache/",
            "cargo build --profile parity -p zelda3-bin",
            "./parity cached-av .git/parity-oracle-cache/<cache_key>   # must match all frames",
        ],
        "notes": notes,
    }
    (archive / "MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    write_sums(archive)
    return archive


def write_sums(archive: Path) -> None:
    lines = []
    for path in sorted(p for p in archive.rglob("*") if p.is_file() and p.name != "SHA256SUMS"):
        lines.append(f"{sha256_file(path)}  {path.relative_to(archive).as_posix()}")
    (archive / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")


def verify(archive: Path) -> int:
    sums = archive / "SHA256SUMS"
    if not sums.is_file():
        print(f"missing {sums}", file=sys.stderr)
        return 2
    bad = 0
    listed: set[str] = set()
    for line in sums.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        digest, rel = line.split("  ", 1)
        listed.add(rel)
        path = archive / rel
        if not path.is_file():
            print(f"MISSING {rel}")
            bad += 1
        elif sha256_file(path) != digest:
            print(f"CORRUPT {rel}")
            bad += 1
    extra = {
        p.relative_to(archive).as_posix()
        for p in archive.rglob("*")
        if p.is_file() and p.name != "SHA256SUMS"
    } - listed
    for rel in sorted(extra):
        print(f"UNLISTED {rel}")
    print(f"verified {len(listed)} files, {bad} problems, {len(extra)} unlisted")
    return 1 if bad else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--dest", type=Path, default=Path.home() / "zelda3-parity-archive")
    parser.add_argument("--binary", type=Path, help="the promoted parity binary (sha must match the ledger)")
    parser.add_argument("--label", help="archive directory name (default: <utc stamp>-<commit>)")
    parser.add_argument("--verify", type=Path, metavar="ARCHIVE_DIR", help="re-check an exported archive")
    args = parser.parse_args()
    if args.verify:
        return verify(args.verify)
    archive = export(args.dest, args.binary, args.label)
    size = sum(p.stat().st_size for p in archive.rglob("*") if p.is_file())
    print(f"exported {archive} ({size / (1 << 30):.2f} GiB)")
    return verify(archive)


if __name__ == "__main__":
    raise SystemExit(main())
