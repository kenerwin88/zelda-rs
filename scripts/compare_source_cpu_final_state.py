#!/usr/bin/env python3
"""Compare a completed source CPU checkpoint with the pinned oracle snapshot.

The route's presented-OAM comparison is per host. This checks the final CPU,
WRAM, and SRAM state that OAM alone cannot establish. It intentionally does
not claim PPU or APU state equivalence.
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def snapshot_block(snapshot: bytes, wanted: bytes) -> bytes:
    if not snapshot.startswith(b"#!s9xsnp:"):
        raise ValueError("not an uncompressed Snes9x snapshot")
    cursor = snapshot.index(b"\n") + 1
    while cursor < len(snapshot):
        header = snapshot[cursor : cursor + 11]
        if len(header) != 11 or header[3:4] != b":" or header[10:11] != b":":
            raise ValueError(f"invalid snapshot block at byte {cursor}")
        length = int(header[4:10])
        cursor += 11
        block = snapshot[cursor : cursor + length]
        if len(block) != length:
            raise ValueError(f"truncated snapshot block at byte {cursor}")
        if header[:3] == wanted:
            return block
        cursor += length
    raise ValueError(f"missing snapshot block {wanted.decode()}")


def oracle_registers(block: bytes) -> dict[str, int]:
    # snapshot.cpp:SnapRegisters, network-order integer fields.
    if len(block) != 16:
        raise ValueError(f"expected 16 REG bytes, received {len(block)}")
    fields = {"k": block[0], "db": block[1]}
    for name, start in (
        ("p", 2),
        ("a", 4),
        ("dp", 6),
        ("sp", 8),
        ("x", 10),
        ("y", 12),
        ("pc", 14),
    ):
        fields[name] = int.from_bytes(block[start : start + 2], "big")
    return fields


def source_registers(cpu: dict) -> dict[str, int]:
    flags = (
        (int(cpu["e"]) << 8)
        | (int(cpu["n"]) << 7)
        | (int(cpu["v"]) << 6)
        | (int(cpu["mf"]) << 5)
        | (int(cpu["xf"]) << 4)
        | (int(cpu["d"]) << 3)
        | (int(cpu["i"]) << 2)
        | (int(cpu["z"]) << 1)
        | int(cpu["c"])
    )
    return {
        name: cpu[name] for name in ("k", "db", "a", "dp", "sp", "x", "y", "pc")
    } | {"p": flags}


def byte_differences(source: bytes, oracle: bytes) -> tuple[int, list[str]]:
    if len(source) != len(oracle):
        raise ValueError(
            f"byte lengths differ: source={len(source)} oracle={len(oracle)}"
        )
    differences = [
        (index, left, right)
        for index, (left, right) in enumerate(zip(source, oracle))
        if left != right
    ]
    return len(differences), [
        f"${index:05x}: source=${left:02x} oracle=${right:02x}"
        for index, left, right in differences[:12]
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_checkpoint", type=Path)
    parser.add_argument("oracle_cache", type=Path)
    args = parser.parse_args()

    cache = args.oracle_cache
    manifest = json.loads((cache / "cache-manifest.json").read_text())
    identity = manifest["cache_identity"]
    checkpoint = json.loads(args.source_checkpoint.read_text())
    expected_host = identity["frames_requested"] - 1
    if checkpoint["completed_host"] != expected_host:
        raise ValueError(
            f"source checkpoint host {checkpoint['completed_host']} is not oracle final host {expected_host}"
        )
    for source_key, manifest_key in (
        ("rom_sha256", "rom_sha256"),
        ("sram_sha256", "source_artifact_sha256.initial.srm"),
        ("input_sha256", "source_artifact_sha256.input.txt"),
    ):
        expected = identity
        for part in manifest_key.split(".", 1):
            expected = expected[part]
        if checkpoint[source_key] != expected:
            raise ValueError(
                f"source checkpoint {source_key} does not match oracle cache"
            )

    snapshot = (cache / "oracle_final.state").read_bytes()
    if sha256(snapshot) != manifest["artifact_sha256"]["oracle_final.state"]:
        raise ValueError("oracle final snapshot hash does not match cache manifest")
    source = checkpoint["cpu"]["snes"]
    expected_regs = oracle_registers(snapshot_block(snapshot, b"REG"))
    actual_regs = source_registers(source["cpu"])
    mismatched_regs = {
        name: (actual_regs[name], expected_regs[name])
        for name in expected_regs
        if actual_regs[name] != expected_regs[name]
    }
    wram = bytes(source["ram"])
    oracle_wram = snapshot_block(snapshot, b"RAM")
    wram_count, wram_first = byte_differences(wram, oracle_wram)
    sram = bytes(source["cart"]["ram"])
    oracle_sram = snapshot_block(snapshot, b"SRA")[: len(sram)]
    sram_count, sram_first = byte_differences(sram, oracle_sram)

    print(
        f"host={expected_host} cpu_register_differences={len(mismatched_regs)} "
        + f"wram_differences={wram_count} sram_differences={sram_count}"
    )
    for name, (actual, expected) in mismatched_regs.items():
        print(f"cpu {name}: source=${actual:04x} oracle=${expected:04x}")
    for detail in wram_first:
        print(f"wram {detail}")
    for detail in sram_first:
        print(f"sram {detail}")
    return int(bool(mismatched_regs or wram_count or sram_count))


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ValueError) as error:
        print(f"state comparison failed: {error}", file=sys.stderr)
        raise SystemExit(2) from None
