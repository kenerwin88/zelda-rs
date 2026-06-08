#!/usr/bin/env python3
"""Small targeted 65c816 disassembler for the local Zelda 3 ROM.

This intentionally reuses the opcode formatting tables from the C port's
tracing.c so targeted ROM listings match the emulator trace format.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
from pathlib import Path


def parse_hex(value: str) -> int:
    value = value.strip().replace("$", "0x")
    return int(value, 16 if value.lower().startswith("0x") else 10)


def parse_string_array(text: str, name: str) -> list[str | None]:
    match = re.search(
        rf"static const char\* {re.escape(name)}\[256\] = \{{(.*?)\}};",
        text,
        re.S,
    )
    if not match:
        raise ValueError(f"missing {name} in tracing.c")
    values: list[str | None] = []
    for token in re.findall(r'"(?:\\.|[^"\\])*"|NULL', match.group(1)):
        if token == "NULL":
            values.append(None)
        else:
            values.append(bytes(token[1:-1], "utf-8").decode("unicode_escape"))
    if len(values) != 256:
        raise ValueError(f"{name} has {len(values)} entries, expected 256")
    return values


def parse_int_array(text: str, name: str) -> list[int]:
    match = re.search(
        rf"static const int {re.escape(name)}\[256\] = \{{(.*?)\}};",
        text,
        re.S,
    )
    if not match:
        raise ValueError(f"missing {name} in tracing/cpu source")
    values = [int(v) for v in re.findall(r"-?\d+", match.group(1))]
    if len(values) != 256:
        raise ValueError(f"{name} has {len(values)} entries, expected 256")
    return values


def load_labels(path: Path | None) -> dict[int, str]:
    if path is None or not path.exists():
        return {}
    labels: dict[int, str] = {}
    for line in path.read_text().splitlines():
        match = re.match(r"0x([0-9a-fA-F]+):\s+(\S+)", line)
        if match:
            labels[int(match.group(1), 16)] = match.group(2)
    return labels


def lorom_offset(addr: int) -> int:
    bank = (addr >> 16) & 0xFF
    offset = addr & 0xFFFF
    if offset < 0x8000:
        raise ValueError(f"${addr:06x} is not in LoROM mapped ROM space")
    return (bank & 0x7F) * 0x8000 + (offset & 0x7FFF)


def signed8(value: int) -> int:
    return value - 0x100 if value & 0x80 else value


def signed16(value: int) -> int:
    return value - 0x10000 if value & 0x8000 else value


def apply_format(template: str, *values: int) -> str:
    return (template % values).rstrip()


def disassemble(
    rom: bytes,
    opcode_names: list[str | None],
    opcode_names_sp: list[str | None],
    opcode_type: list[int],
    cycles: list[int] | None,
    labels: dict[int, str],
    start: int,
    end: int | None,
    count: int | None,
    m_flag: bool,
    x_flag: bool,
) -> None:
    addr = start
    emitted = 0
    while True:
        if count is not None and emitted >= count:
            break
        if end is not None and addr >= end:
            break

        if addr in labels:
            print(f"\n{labels[addr]}:")

        off = lorom_offset(addr)
        opcode = rom[off]
        byte = rom[off + 1] if off + 1 < len(rom) else 0
        byte2 = rom[off + 2] if off + 2 < len(rom) else 0
        word = byte | (byte2 << 8)
        longv = word | ((rom[off + 3] if off + 3 < len(rom) else 0) << 16)
        low_pc = addr & 0xFFFF
        kind = opcode_type[opcode]
        size = 1
        mnemonic = ""

        if kind == 0:
            mnemonic = apply_format(opcode_names[opcode] or "???")
        elif kind == 1:
            size = 2
            mnemonic = apply_format(opcode_names[opcode] or "???", byte)
        elif kind == 2:
            size = 3
            mnemonic = apply_format(opcode_names[opcode] or "???", word)
        elif kind == 3:
            size = 4
            mnemonic = apply_format(opcode_names[opcode] or "???", longv)
        elif kind == 4:
            if m_flag:
                size = 2
                mnemonic = apply_format(opcode_names_sp[opcode] or opcode_names[opcode] or "???", byte)
            else:
                size = 3
                mnemonic = apply_format(opcode_names[opcode] or "???", word)
        elif kind == 5:
            if x_flag:
                size = 2
                mnemonic = apply_format(opcode_names_sp[opcode] or opcode_names[opcode] or "???", byte)
            else:
                size = 3
                mnemonic = apply_format(opcode_names[opcode] or "???", word)
        elif kind == 6:
            size = 2
            target = (low_pc + 2 + signed8(byte)) & 0xFFFF
            mnemonic = apply_format(opcode_names[opcode] or "???", target)
        elif kind == 7:
            size = 3
            target = (low_pc + 3 + signed16(word)) & 0xFFFF
            mnemonic = apply_format(opcode_names[opcode] or "???", target)
        elif kind == 8:
            size = 3
            mnemonic = apply_format(opcode_names[opcode] or "???", byte2, byte)
        else:
            raise ValueError(f"unknown opcode type {kind}")

        raw = rom[off : off + size]
        cycle_text = f" {cycles[opcode]:2d}c" if cycles else ""
        print(f"{addr:06x}: {' '.join(f'{b:02x}' for b in raw):<11} {mnemonic:<16}{cycle_text}")

        if opcode in (0xC2, 0xE2):
            set_bits = opcode == 0xE2
            if byte & 0x20:
                m_flag = set_bits
            if byte & 0x10:
                x_flag = set_bits

        addr += size
        emitted += 1


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    c_root = Path(os.environ.get("ZELDA3_C_REPO", str(repo_root.parent / "zelda3")))
    parser = argparse.ArgumentParser()
    parser.add_argument("--rom", default=str(Path(os.environ.get("ZELDA3_ROM", str(c_root / "zelda3.sfc")))))
    parser.add_argument("--tracing", default=str(c_root / "snes" / "tracing.c"))
    parser.add_argument("--cpu", default=str(c_root / "snes" / "cpu.c"))
    parser.add_argument("--names", default=str(c_root / "other" / "names.txt"))
    parser.add_argument("--start", required=True, type=parse_hex)
    parser.add_argument("--end", type=parse_hex)
    parser.add_argument("--count", type=int)
    parser.add_argument("--m", choices=("8", "16"), default="8")
    parser.add_argument("--x", choices=("8", "16"), default="8")
    parser.add_argument("--cycles", action="store_true")
    args = parser.parse_args()

    tracing = Path(args.tracing).read_text()
    cpu_source = Path(args.cpu).read_text() if args.cycles else ""
    disassemble(
        Path(args.rom).read_bytes(),
        parse_string_array(tracing, "opcodeNames"),
        parse_string_array(tracing, "opcodeNamesSp"),
        parse_int_array(tracing, "opcodeType"),
        parse_int_array(cpu_source, "cyclesPerOpcode") if args.cycles else None,
        load_labels(Path(args.names) if args.names else None),
        args.start,
        args.end,
        args.count,
        args.m == "8",
        args.x == "8",
    )


if __name__ == "__main__":
    main()
