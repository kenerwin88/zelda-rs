#!/usr/bin/env python3
"""Export ROM-derived song-bank uploads into reviewed modern sample assets.

The generated asset pack is an offline authoring input. Runtime code consumes the
checked-in manifest and BRR/echo files; it never reads these upload blobs or a ROM.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "generated" / "zelda3_assets" / "assets"
DEFAULT_OUTPUT = ROOT / "assets" / "audio" / "modern_samples"
FORMAT = "zelda3_modern_sample_bank_v1"
DIRECTORY = 0x3C00
SOURCE_COUNT = 25
ECHO_SEED_START = 0xC800
BANK_FILES = (
    (0, "overworld", "000-kSoundBank_intro.bin"),
    (1, "dungeon", "001-kSoundBank_indoor.bin"),
    (2, "credits", "002-kSoundBank_ending.bin"),
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def apply_upload(ram: bytearray, upload: bytes) -> None:
    cursor = 0
    while cursor + 2 <= len(upload):
        length = int.from_bytes(upload[cursor : cursor + 2], "little")
        cursor += 2
        if length == 0:
            return
        if cursor + 2 > len(upload):
            raise ValueError("upload is truncated before target address")
        target = int.from_bytes(upload[cursor : cursor + 2], "little")
        cursor += 2
        end = cursor + length
        if end > len(upload):
            raise ValueError("upload block extends past input")
        for value in upload[cursor:end]:
            ram[target] = value
            target = (target + 1) & 0xFFFF
        cursor = end
    raise ValueError("upload is missing its zero-length terminator")


def extract_brr(ram: bytes, start: int) -> bytes:
    if start == 0 or start + 9 > len(ram):
        raise ValueError(f"invalid BRR start address 0x{start:04x}")
    address = start
    for _ in range(4096):
        if address + 9 > len(ram):
            raise ValueError(f"BRR sample at 0x{start:04x} crosses RAM boundary")
        address += 9
        if ram[address - 9] & 1:
            return ram[start:address]
    raise ValueError(f"BRR sample at 0x{start:04x} has no end block")


def build_document(source_dir: Path) -> tuple[dict[str, object], dict[str, bytes]]:
    ram = bytearray(0x10000)
    files: dict[str, bytes] = {}
    samples: dict[str, dict[str, object]] = {}
    banks: list[dict[str, object]] = []

    for bank_id, name, filename in BANK_FILES:
        upload = (source_dir / filename).read_bytes()
        apply_upload(ram, upload)
        instruments: list[dict[str, object]] = []
        for source in range(SOURCE_COUNT):
            entry = DIRECTORY + source * 4
            start = int.from_bytes(ram[entry : entry + 2], "little")
            loop_address = int.from_bytes(ram[entry + 2 : entry + 4], "little")
            raw = extract_brr(ram, start)
            digest = sha256(raw)
            sample_id = f"brr_{digest[:16]}"
            sample_file = f"samples/{sample_id}.brr"
            files.setdefault(sample_file, raw)
            samples.setdefault(
                sample_id,
                {
                    "id": sample_id,
                    "file": sample_file,
                    "sha256": digest,
                    "blocks": len(raw) // 9,
                },
            )
            loop_offset = loop_address - start
            if not 0 <= loop_offset < len(raw):
                loop_offset = 0
            instruments.append(
                {
                    "source": source,
                    "sample": sample_id,
                    "loop_offset": loop_offset,
                }
            )

        echo_seed = bytes(ram[ECHO_SEED_START:])
        echo_file = f"echo/{bank_id:02d}-{name}.bin"
        files[echo_file] = echo_seed
        banks.append(
            {
                "id": bank_id,
                "name": name,
                "instruments": instruments,
                "echo_seed": {
                    "start_address": ECHO_SEED_START,
                    "file": echo_file,
                    "sha256": sha256(echo_seed),
                },
            }
        )

    document: dict[str, object] = {
        "format": FORMAT,
        "sample_rate": 32000,
        "samples": sorted(samples.values(), key=lambda sample: str(sample["id"])),
        "banks": banks,
    }
    return document, files


def validate_document(document: object, base: Path) -> list[str]:
    errors: list[str] = []
    if not isinstance(document, dict):
        return ["document must be an object"]
    if document.get("format") != FORMAT:
        errors.append(f"format must be {FORMAT!r}")
    if document.get("sample_rate") != 32000:
        errors.append("sample_rate must be 32000")
    samples = document.get("samples")
    banks = document.get("banks")
    if not isinstance(samples, list) or not samples:
        errors.append("samples must be a non-empty array")
        return errors
    if not isinstance(banks, list) or not banks:
        errors.append("banks must be a non-empty array")
        return errors

    sample_ids: set[str] = set()
    sample_lengths: dict[str, int] = {}
    for index, sample in enumerate(samples):
        label = f"samples[{index}]"
        if not isinstance(sample, dict):
            errors.append(f"{label} must be an object")
            continue
        sample_id = sample.get("id")
        if not isinstance(sample_id, str) or not sample_id:
            errors.append(f"{label}.id must be non-empty")
            continue
        if sample_id in sample_ids:
            errors.append(f"{label} duplicates sample id {sample_id!r}")
        sample_ids.add(sample_id)
        path = base / str(sample.get("file", ""))
        try:
            raw = path.read_bytes()
        except OSError as exc:
            errors.append(f"{label} cannot read {path}: {exc}")
            continue
        if len(raw) == 0 or len(raw) % 9 != 0 or raw[-9] & 1 == 0:
            errors.append(f"{label} is not a complete BRR stream")
        if sha256(raw) != sample.get("sha256"):
            errors.append(f"{label} sha256 mismatch")
        if sample.get("blocks") != len(raw) // 9:
            errors.append(f"{label}.blocks does not match file")
        sample_lengths[sample_id] = len(raw)

    bank_ids: set[int] = set()
    for index, bank in enumerate(banks):
        label = f"banks[{index}]"
        if not isinstance(bank, dict):
            errors.append(f"{label} must be an object")
            continue
        bank_id = bank.get("id")
        if not isinstance(bank_id, int) or not 0 <= bank_id <= 255:
            errors.append(f"{label}.id must be in 0..255")
        elif bank_id in bank_ids:
            errors.append(f"{label} duplicates bank id {bank_id}")
        else:
            bank_ids.add(bank_id)
        sources: set[int] = set()
        instruments = bank.get("instruments")
        if not isinstance(instruments, list):
            errors.append(f"{label}.instruments must be an array")
            continue
        for instrument_index, instrument in enumerate(instruments):
            instrument_label = f"{label}.instruments[{instrument_index}]"
            if not isinstance(instrument, dict):
                errors.append(f"{instrument_label} must be an object")
                continue
            source = instrument.get("source")
            if not isinstance(source, int) or not 0 <= source <= 255:
                errors.append(f"{instrument_label}.source must be in 0..255")
            elif source in sources:
                errors.append(f"{instrument_label} duplicates source {source}")
            else:
                sources.add(source)
            if instrument.get("sample") not in sample_ids:
                errors.append(f"{instrument_label} references unknown sample")
            else:
                loop_offset = instrument.get("loop_offset")
                sample_length = sample_lengths.get(str(instrument.get("sample")), 0)
                if (
                    not isinstance(loop_offset, int)
                    or loop_offset < 0
                    or loop_offset % 9 != 0
                    or loop_offset >= sample_length
                ):
                    errors.append(f"{instrument_label}.loop_offset is invalid")
        if sources != set(range(SOURCE_COUNT)):
            errors.append(f"{label} must map sources 0..{SOURCE_COUNT - 1}")
        echo = bank.get("echo_seed")
        if not isinstance(echo, dict):
            errors.append(f"{label}.echo_seed must be an object")
            continue
        path = base / str(echo.get("file", ""))
        try:
            raw = path.read_bytes()
        except OSError as exc:
            errors.append(f"{label}.echo_seed cannot read {path}: {exc}")
            continue
        if sha256(raw) != echo.get("sha256"):
            errors.append(f"{label}.echo_seed sha256 mismatch")
        start = echo.get("start_address")
        if not isinstance(start, int) or not 0 <= start <= 0xFFFF or start + len(raw) > 0x10000:
            errors.append(f"{label}.echo_seed range is invalid")
    return errors


def write_export(document: dict[str, object], files: dict[str, bytes], output: Path) -> None:
    output.mkdir(parents=True, exist_ok=True)
    expected = {output / relative for relative in files}
    for path in output.glob("**/*"):
        if path.is_file() and path.name != "manifest.json" and path not in expected:
            path.unlink()
    for relative, data in files.items():
        path = output / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    (output / "manifest.json").write_text(
        json.dumps(document, indent=2) + "\n", encoding="utf-8"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    if args.validate_only:
        document = json.loads((args.output / "manifest.json").read_text(encoding="utf-8"))
    else:
        document, files = build_document(args.source)
        write_export(document, files, args.output)
    errors = validate_document(document, args.output)
    if errors:
        for error in errors:
            print(error)
        return 1
    print(
        f"modern sample bank valid: samples={len(document['samples'])} "
        f"banks={len(document['banks'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
