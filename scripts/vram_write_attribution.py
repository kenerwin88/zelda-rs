#!/usr/bin/env python3
"""Attribute a Snes9x VRAM/WRAM divergence to the transfer that caused it.

Given a `target/parity-failures/<run>/` artifact directory, this answers the
question a raw byte diff cannot: *which side performed a transfer the other did
not, and with which operand generation?*

It works from three write sets:

  O = words the ORACLE changed during the frame (oracle_before vs oracle_after)
  L = words where RUST's live VRAM differs from the oracle's after-frame VRAM
  P = words where RUST's presented (scanout) VRAM differs from the same

The classification of `L` against `O` is the decisive signal:

  * `L` inside `O`   -> both sides wrote the same words with different data
                        (operand *values* differ, or a source buffer differs).
  * `L` disjoint from `O` -> one side performed a transfer the other did not, or
                        performed it with a different source pointer. Every word
                        the oracle wrote matched, so the engine is fine and the
                        divergence is a DMA scheduling/operand-generation bug.
  * in `L` but not `P` -> a publication-generation choice, not wrong content.

For the Link OBJ CHR block it goes one step further: each divergent destination
is resolved against the `LinkDmaSourceSlot` table (scraped from the Rust
sources, so it cannot silently drift), and rust's bytes are compared against
`ram[pointer .. pointer+len]` using both the frame's entry pointer and its exit
pointer. When those two differ, the side whose bytes match names the operand
generation each emulator used -- which is exactly the input to
`rom_graphics_dma_plan`'s `link_obj_operands` decision.

Usage:
    python3 scripts/vram_write_attribution.py target/parity-failures/<run>
    python3 scripts/vram_write_attribution.py --latest
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FAILURE_ROOT = ROOT / "target" / "parity-failures"

# VRAM regions worth naming in the report. (first_word, last_word, label)
VRAM_REGIONS: list[tuple[int, int, str]] = [
    (0x0000, 0x0FFF, "BG2 tilemap / low tilemap"),
    (0x1000, 0x1FFF, "BG1 tilemap"),
    (0x2000, 0x3AFF, "BG CHR"),
    (0x3B00, 0x3CFF, "animated BG CHR block"),
    (0x3D00, 0x3FFF, "BG CHR (peg/star tiles)"),
    (0x4000, 0x43FF, "Link OBJ CHR"),
    (0x4400, 0x4FFF, "OBJ CHR"),
    (0x5000, 0x5FFF, "OBJ CHR (name select)"),
    (0x6000, 0x6FFF, "BG3 tilemap"),
    (0x7000, 0x7FFF, "BG3 CHR"),
]


def region_of(word: int) -> str:
    for first, last, label in VRAM_REGIONS:
        if first <= word <= last:
            return label
    return "unmapped"


def read(path: Path) -> bytes | None:
    try:
        return path.read_bytes()
    except OSError:
        return None


def words(buf: bytes) -> list[int]:
    return [int.from_bytes(buf[i : i + 2], "little") for i in range(0, len(buf), 2)]


def diff_words(a: bytes, b: bytes) -> list[int]:
    wa, wb = words(a), words(b)
    return [i for i in range(min(len(wa), len(wb))) if wa[i] != wb[i]]


def to_runs(indices: list[int]) -> list[tuple[int, int]]:
    runs: list[list[int]] = []
    for i in indices:
        if runs and i == runs[-1][1] + 1:
            runs[-1][1] = i
        else:
            runs.append([i, i])
    return [(a, b) for a, b in runs]


# ---------------------------------------------------------------------------
# Link OBJ DMA table, scraped from the Rust sources so it cannot drift silently.
# ---------------------------------------------------------------------------

SLOT_CONST_RE = re.compile(r"Self::(\w+)\s*=>\s*(DMA_SOURCE_ADDR_\d+)\s*,")
CONST_RE = re.compile(r"const\s+(DMA_SOURCE_ADDR_\d+)\s*:\s*usize\s*=\s*(0x[0-9a-fA-F]+)\s*;")
TRANSFER_RE = re.compile(
    r"\(\s*(0x[0-9a-fA-F]+)\s*,\s*LinkDmaSourceSlot::(\w+)\s*,\s*(0x[0-9a-fA-F]+)\s*\)"
)


def load_link_dma_table() -> tuple[list[tuple[int, str, int, int]], list[str]]:
    """Return [(vram_dest_word, slot_name, ram_pointer_addr, byte_len)] + warnings."""
    warnings: list[str] = []
    display = read(ROOT / "crates/zelda3/src/game_state/native/display.rs")
    constants = read(ROOT / "crates/zelda3/src/game_state/constants.rs")
    rtl = read(ROOT / "crates/zelda3/src/zelda_rtl.rs")
    if display is None or constants is None or rtl is None:
        return [], ["could not read the Rust sources; Link OBJ attribution skipped"]

    const_addr = {
        name: int(value, 16)
        for name, value in CONST_RE.findall(constants.decode("utf-8", "replace"))
    }
    slot_const = dict(SLOT_CONST_RE.findall(display.decode("utf-8", "replace")))
    table: dict[int, tuple[int, str, int, int]] = {}
    for dest, slot, length in TRANSFER_RE.findall(rtl.decode("utf-8", "replace")):
        const_name = slot_const.get(slot)
        if const_name is None or const_name not in const_addr:
            warnings.append(f"no RAM pointer resolved for LinkDmaSourceSlot::{slot}")
            continue
        table[int(dest, 16)] = (
            int(dest, 16),
            slot,
            const_addr[const_name],
            int(length, 16),
        )
    if not table:
        warnings.append("no Link OBJ DMA transfers scraped; table regexes need updating")
    return sorted(table.values()), warnings


def u16(buf: bytes, addr: int) -> int | None:
    if addr + 1 >= len(buf):
        return None
    return buf[addr] | (buf[addr + 1] << 8)


def vram_bytes(buf: bytes, dest_word: int, byte_len: int) -> bytes:
    return buf[dest_word * 2 : dest_word * 2 + byte_len]


def attribute_link_obj(
    divergent: set[int],
    rust_after_vram: bytes,
    oracle_after_vram: bytes,
    rust_before_ram: bytes,
    rust_after_ram: bytes,
) -> None:
    table, warnings = load_link_dma_table()
    for warning in warnings:
        print(f"  ! {warning}")
    hits = [
        entry
        for entry in table
        if any(entry[0] <= w < entry[0] + entry[3] // 2 for w in divergent)
    ]
    if not hits:
        return
    print()
    print("  Link OBJ CHR operand attribution")
    print("  (does each side's VRAM match ram[pointer..] at frame entry or at frame exit?)")
    for dest, slot, ptr_addr, byte_len in hits:
        entry_ptr = u16(rust_before_ram, ptr_addr)
        exit_ptr = u16(rust_after_ram, ptr_addr)
        rust_words = vram_bytes(rust_after_vram, dest, byte_len)
        oracle_words = vram_bytes(oracle_after_vram, dest, byte_len)
        print(
            f"    ${dest:04x} {slot:<18} pointer ${ptr_addr:04x}: "
            f"entry={entry_ptr:04x} exit={exit_ptr:04x}"
            + ("  (advanced during the frame)" if entry_ptr != exit_ptr else "")
        )
        for label, ptr in (("entry", entry_ptr), ("exit", exit_ptr)):
            if ptr is None:
                continue
            source = rust_after_ram[ptr : ptr + byte_len]
            if len(source) < byte_len:
                continue
            tags = []
            if rust_words == source:
                tags.append("rust")
            if oracle_words == source:
                tags.append("oracle")
            if tags:
                generation = (
                    "HostBoundaryBeforeMain" if label == "entry" else "LiveAfterMain"
                )
                print(
                    f"      {'+'.join(tags):<13} == ram[${ptr:04x}..+0x{byte_len:x}]"
                    f"  -> {generation}"
                )


SAVESTATE_CHUNK_RE = re.compile(rb"([A-Z0-9]{3,4}):(\d{6}):")
OAM_BYTES = 544


def oracle_oam_from_savestate(state: bytes, rust_oam: bytes) -> tuple[bytes, int, int] | None:
    """Locate Snes9x's OAM inside a savestate's PPU chunk.

    The PPU chunk is an opaque struct dump, so the OAM is found by sliding a
    544-byte window and taking the best match against rust's OAM. That is only
    trustworthy when the winner is far clear of the runner-up, so the separation
    is returned and checked by the caller.
    """
    for match in SAVESTATE_CHUNK_RE.finditer(state[:4096]):
        if match.group(1) != b"PPU":
            continue
        start = match.end()
        chunk = state[start : start + int(match.group(2))]
        scored = sorted(
            (
                sum(1 for i in range(OAM_BYTES) if chunk[offset + i] != rust_oam[i]),
                offset,
            )
            for offset in range(len(chunk) - OAM_BYTES + 1)
        )
        if len(scored) < 2:
            return None
        (best, offset), (runner_up, _) = scored[0], scored[1]
        return chunk[offset : offset + OAM_BYTES], best, runner_up
    return None


OAM_FIELDS = ("x", "y", "tile", "attr")


def report_oam(directory: Path, rust_oam: bytes) -> None:
    state = read(directory / "oracle_after.state")
    if state is None:
        return
    found = oracle_oam_from_savestate(state, rust_oam)
    if found is None:
        print("  ! no PPU chunk found in oracle_after.state; OAM diff skipped")
        return
    oracle_oam, best, runner_up = found
    print()
    if best and runner_up < best * 4:
        print(
            f"  ! OAM window match is ambiguous (best {best} vs runner-up {runner_up}"
            " differing bytes); treat the slots below as unverified"
        )
    print(f"OAM (rust presented vs oracle savestate): {best} byte(s) differ")
    if best == 0:
        return
    for i in range(OAM_BYTES):
        if oracle_oam[i] == rust_oam[i]:
            continue
        if i < 512:
            print(
                f"  slot {i // 4:3d} {OAM_FIELDS[i % 4]:<4} "
                f"rust={rust_oam[i]:02x} oracle={oracle_oam[i]:02x}"
                f"  (delta {rust_oam[i] - oracle_oam[i]:+d})"
            )
        else:
            print(
                f"  high byte {i - 512:2d} (slots {(i - 512) * 4}..{(i - 512) * 4 + 3}) "
                f"rust={rust_oam[i]:02x} oracle={oracle_oam[i]:02x}"
            )
    print(
        "  Small position deltas on several slots mean the presented OAM sits on the\n"
        "  wrong side of the main slice that moved them -- an `oam_operands` /\n"
        "  `oam_scanout` generation bug, not wrong sprite data."
    )


def report(directory: Path) -> int:
    files = {
        name: read(directory / name)
        for name in (
            "oracle_before_vram.bin",
            "oracle_after_vram.bin",
            "rust_after_vram.bin",
            "rust_visible_vram.bin",
            "rust_before_ram.bin",
            "rust_after_ram.bin",
            "oracle_before_ram.bin",
            "oracle_after_ram.bin",
        )
    }
    missing = [name for name, data in files.items() if data is None]
    required = ("oracle_before_vram.bin", "oracle_after_vram.bin", "rust_after_vram.bin")
    if any(files[name] is None for name in required):
        print(f"missing required artifacts in {directory}: {missing}", file=sys.stderr)
        return 2

    print(f"=== {directory} ===")

    oracle_wrote = set(diff_words(files["oracle_before_vram.bin"], files["oracle_after_vram.bin"]))
    live = diff_words(files["rust_after_vram.bin"], files["oracle_after_vram.bin"])
    presented = (
        diff_words(files["rust_visible_vram.bin"], files["oracle_after_vram.bin"])
        if files["rust_visible_vram.bin"] is not None
        else []
    )
    print(f"oracle wrote {len(oracle_wrote)} VRAM word(s) during the frame")
    print(f"live divergence      : {len(live)} word(s)")
    print(f"presented divergence : {len(presented)} word(s)")
    if not live and not presented:
        print("VRAM matches in both generations.")
    else:
        inside = [w for w in live if w in oracle_wrote]
        outside = [w for w in live if w not in oracle_wrote]
        print()
        print(f"  {len(inside)} divergent word(s) are inside the oracle's write set")
        print(f"  {len(outside)} divergent word(s) are OUTSIDE it")
        if outside and not inside:
            print(
                "  => every word the oracle wrote MATCHES. One side ran a transfer the\n"
                "     other did not, or ran it with a different source pointer:\n"
                "     a DMA scheduling / operand-generation bug, not an engine bug."
            )
        elif inside and not outside:
            print(
                "  => both sides wrote the same words with different data:\n"
                "     the operand VALUES or the source buffer differ."
            )
        presented_set = set(presented)
        live_only = [w for w in live if w not in presented_set]
        if live_only:
            print(
                f"  {len(live_only)} word(s) diverge LIVE ONLY (held back in the presented\n"
                "     generation) -- an early upload, not the visible bug."
            )
        for label, indices in (("live", live), ("presented", presented)):
            if not indices:
                continue
            print()
            print(f"  {label} divergent word runs:")
            for first, last in to_runs(indices):
                span = f"${first:04x}-${last:04x}" if first != last else f"${first:04x}"
                extra = ""
                if 0x4000 <= first <= 0x43FF:
                    extra = f", Link tile ${(first - 0x4000) // 16:02x}"
                print(f"    {span:<15} {region_of(first)}{extra}")

        if any(0x4000 <= w <= 0x43FF for w in live) and (
            files["rust_before_ram.bin"] is not None
            and files["rust_after_ram.bin"] is not None
        ):
            attribute_link_obj(
                set(live),
                files["rust_after_vram.bin"],
                files["oracle_after_vram.bin"],
                files["rust_before_ram.bin"],
                files["rust_after_ram.bin"],
            )

    rust_oam = read(directory / "rust_visible_oam.bin")
    if rust_oam is not None and len(rust_oam) == OAM_BYTES:
        report_oam(directory, rust_oam)

    if all(
        files[name] is not None
        for name in ("rust_before_ram.bin", "oracle_before_ram.bin", "rust_after_ram.bin", "oracle_after_ram.bin")
    ):
        before = {
            i
            for i in range(min(len(files["rust_before_ram.bin"]), len(files["oracle_before_ram.bin"])))
            if files["rust_before_ram.bin"][i] != files["oracle_before_ram.bin"][i]
        }
        after = {
            i
            for i in range(min(len(files["rust_after_ram.bin"]), len(files["oracle_after_ram.bin"])))
            if files["rust_after_ram.bin"][i] != files["oracle_after_ram.bin"][i]
        }
        newly = sorted(after - before)
        healed = sorted(before - after)
        print()
        print(
            f"WRAM: {len(before)} byte(s) already divergent at frame entry, "
            f"{len(newly)} newly divergent, {len(healed)} healed"
        )
        if newly:
            print("  newly divergent WRAM:")
            for first, last in to_runs(newly):
                span = f"0x{first:05x}-0x{last:05x}" if first != last else f"0x{first:05x}"
                rust = files["rust_after_ram.bin"][first : min(last + 1, first + 8)].hex()
                oracle = files["oracle_after_ram.bin"][first : min(last + 1, first + 8)].hex()
                print(f"    {span:<17} rust={rust} oracle={oracle}")
        else:
            print(
                "  no newly divergent WRAM: the engine agrees and the divergence is\n"
                "  confined to display transfers/publication."
            )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", nargs="?", help="parity-failures artifact directory")
    parser.add_argument(
        "--latest",
        action="store_true",
        help=f"use the newest run directory under {FAILURE_ROOT}",
    )
    args = parser.parse_args()

    if args.latest or args.directory is None:
        candidates = [
            path
            for path in FAILURE_ROOT.glob("*")
            if path.is_dir() and (path / "oracle_after_vram.bin").exists()
        ]
        if not candidates:
            print(f"no usable artifact directories under {FAILURE_ROOT}", file=sys.stderr)
            return 2
        directory = max(candidates, key=lambda path: path.stat().st_mtime)
    else:
        directory = Path(args.directory)
    return report(directory)


if __name__ == "__main__":
    raise SystemExit(main())
