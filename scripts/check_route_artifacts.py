#!/usr/bin/env python3
"""Validate that committed route packages are portable and reasonably sized."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ROUTES = ROOT / "routes"
MAX_VERSIONED_FILE_BYTES = 10 * 1024 * 1024
MAX_VERSIONED_PROJECT_BYTES = 50 * 1024 * 1024
RECORDING_KIND = "zelda3_snes9x_route_recording_v1"
BOUNDARY_FILES = {
    "state_path": "state_sha256",
    "wram_path": "wram_sha256",
    "vram_path": "vram_sha256",
    "sram_path": "sram_sha256",
    "screenshot_path": "screenshot_sha256",
}


def is_generated_large_artifact(path: Path, routes: Path = ROUTES) -> bool:
    try:
        relative = path.relative_to(routes)
    except ValueError:
        return False
    parts = relative.parts
    return (
        "comparisons" in parts
        or path.name == "frame_receipts.jsonl"
        or path.name == "matrix-result.json"
        or path.suffix == ".tmp"
    )


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def safe_project_path(project: Path, relative: str, label: str) -> tuple[Path | None, str | None]:
    value = Path(relative)
    if value.is_absolute() or ".." in value.parts:
        return None, f"{label} must be project-relative: {relative}"
    resolved = (project / value).resolve()
    try:
        resolved.relative_to(project.resolve())
    except ValueError:
        return None, f"{label} escapes project: {relative}"
    return resolved, None


def validate_project(project: Path) -> list[str]:
    errors = []
    manifest_path = project / "manifest.json"
    try:
        manifest = json.loads(manifest_path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        return [f"{manifest_path}: {error}"]
    if manifest.get("kind") != RECORDING_KIND:
        return [f"{manifest_path}: unsupported kind {manifest.get('kind')!r}"]

    for required in ("labels.json", "sram-origin.json"):
        if not (project / required).is_file():
            errors.append(f"{project / required}: missing versioned route metadata")

    origin_path = project / "sram-origin.json"
    if origin_path.is_file():
        try:
            origin = json.loads(origin_path.read_text())
            source_path = origin.get("path")
            if source_path and Path(source_path).is_absolute():
                errors.append(f"{origin_path}: SRAM source path must be portable")
        except json.JSONDecodeError as error:
            errors.append(f"{origin_path}: {error}")

    for boundary in manifest.get("boundaries", []):
        boundary_id = boundary.get("id", "?")
        for path_key, hash_key in BOUNDARY_FILES.items():
            relative = boundary.get(path_key)
            if not relative:
                errors.append(f"{manifest_path}: boundary {boundary_id} lacks {path_key}")
                continue
            artifact, error = safe_project_path(
                project, relative, f"boundary {boundary_id} {path_key}"
            )
            if error:
                errors.append(f"{manifest_path}: {error}")
                continue
            if not artifact.is_file():
                errors.append(f"{artifact}: missing boundary artifact")
                continue
            expected = boundary.get(hash_key)
            if expected and sha256(artifact) != expected:
                errors.append(f"{artifact}: SHA-256 does not match {hash_key}")

    for take in manifest.get("takes", []):
        relative = take.get("input_path")
        take_id = take.get("id", "?")
        if not relative:
            errors.append(f"{manifest_path}: take {take_id} lacks input_path")
            continue
        artifact, error = safe_project_path(project, relative, f"take {take_id} input_path")
        if error:
            errors.append(f"{manifest_path}: {error}")
        elif not artifact.is_file():
            errors.append(f"{artifact}: missing compact input stream")
    return errors


def validate_routes(
    routes: Path = ROUTES,
    max_file_bytes: int = MAX_VERSIONED_FILE_BYTES,
    max_project_bytes: int = MAX_VERSIONED_PROJECT_BYTES,
) -> list[str]:
    if not routes.is_dir():
        return [f"{routes}: routes directory does not exist"]
    errors = []
    for path in routes.rglob("*"):
        if not path.is_file() or is_generated_large_artifact(path, routes):
            continue
        if path.stat().st_size > max_file_bytes:
            errors.append(
                f"{path}: {path.stat().st_size} bytes exceeds the "
                f"{max_file_bytes}-byte versioned-file limit"
            )
    for manifest in routes.glob("*/manifest.json"):
        project = manifest.parent
        project_bytes = sum(
            path.stat().st_size
            for path in project.rglob("*")
            if path.is_file() and not is_generated_large_artifact(path, routes)
        )
        if project_bytes > max_project_bytes:
            errors.append(
                f"{project}: {project_bytes} bytes exceeds the "
                f"{max_project_bytes}-byte versioned-project limit"
            )
        errors.extend(validate_project(project))
    return errors


def main() -> None:
    errors = validate_routes()
    if errors:
        for error in errors:
            print(f"route artifact error: {error}", file=sys.stderr)
        raise SystemExit(1)
    projects = len(list(ROUTES.glob("*/manifest.json")))
    print(
        f"route artifacts valid: projects={projects} "
        f"max_file_bytes={MAX_VERSIONED_FILE_BYTES} "
        f"max_project_bytes={MAX_VERSIONED_PROJECT_BYTES}"
    )


if __name__ == "__main__":
    main()
