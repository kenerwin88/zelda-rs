#!/usr/bin/env python3
"""Price a ROM routine's basic blocks in master cycles, statically, for the
cycle ledger annotations in the translated engine.

The pricing follows the shadow CPU (`crates/snes`, a 65816 with slow ROM):
master = internal cycles x 6 + bus accesses x 8 (6 for the register window
$2000-$5FFF). Per instruction: the base cycle count of the opcode, +1 for
16-bit accumulator memory operands (+2 for read-modify-write), +1 for 16-bit
index operands, +1 for a taken branch, +1 on abs,X / abs,Y / (dp),Y reads
when the index registers are 16-bit or the index crosses a page (printed as
a variable cost when the index width is 8 bits), and the direct-page penalty
when DP's low byte is nonzero (assumed zero; use --dp to override).

The walk follows control flow from the entry, tracking the M and X flags
through REP/SEP on every path, and prints one line per instruction with its
master cycles, grouped into basic blocks with block totals. Calls (JSR/JSL)
are priced as the call instruction only; the callee is its own routine.

usage: rom_block_costs.py <entry> [--rom saves/zelda3.sfc] [--m8|--m16] [--x8|--x16]
       rom_block_costs.py --verify <profile-dir>   # check the pricing against profiles
"""
import argparse
import json
import os
import re
import sys
from collections import defaultdict
from pathlib import Path

DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", "saves/zelda3.sfc"))
DEFAULT_SYMBOLS = Path(
    os.environ.get("ZELDA3_ROM_SYMBOLS", "/Users/missingno/Documents/zelda3/other/names.txt")
)

CYCLES = [
    7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5,
    2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
    6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5,
    2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
    6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5,
    2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
    6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5,
    2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
    3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5,
    2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
    2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5,
    2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5,
    2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
    2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5,
    2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
]

# (mnemonic, addressing mode, operand kind)
# kinds: 'A' accumulator-width memory operand, 'X' index-width operand,
# 'RMW' read-modify-write, 'W' write-only (no page-cross penalty), 'S' stack,
# 'CTL' control flow, 'N' no memory operand.
MODES = {}


def _op(code, name, mode, kind):
    MODES[code] = (name, mode, kind)


for base, name in [(0x00, "ORA"), (0x20, "AND"), (0x40, "EOR"), (0x60, "ADC"), (0x80, "STA"), (0xA0, "LDA"), (0xC0, "CMP"), (0xE0, "SBC")]:
    kind = "W" if name == "STA" else "A"
    _op(base | 0x01, name, "(dp,x)", kind)
    _op(base | 0x03, name, "sr,s", kind)
    _op(base | 0x05, name, "dp", kind)
    _op(base | 0x07, name, "[dp]", kind)
    _op(base | 0x09, name, "#", kind if name != "STA" else "A")
    _op(base | 0x0D, name, "abs", kind)
    _op(base | 0x0F, name, "long", kind)
    _op(base | 0x11, name, "(dp),y", kind)
    _op(base | 0x12, name, "(dp)", kind)
    _op(base | 0x13, name, "(sr,s),y", kind)
    _op(base | 0x15, name, "dp,x", kind)
    _op(base | 0x17, name, "[dp],y", kind)
    _op(base | 0x19, name, "abs,y", kind)
    _op(base | 0x1D, name, "abs,x", kind)
    _op(base | 0x1F, name, "long,x", kind)
for code, name, mode, kind in [
    (0x06, "ASL", "dp", "RMW"), (0x0A, "ASL", "acc", "N"), (0x0E, "ASL", "abs", "RMW"), (0x16, "ASL", "dp,x", "RMW"), (0x1E, "ASL", "abs,x", "RMW"),
    (0x26, "ROL", "dp", "RMW"), (0x2A, "ROL", "acc", "N"), (0x2E, "ROL", "abs", "RMW"), (0x36, "ROL", "dp,x", "RMW"), (0x3E, "ROL", "abs,x", "RMW"),
    (0x46, "LSR", "dp", "RMW"), (0x4A, "LSR", "acc", "N"), (0x4E, "LSR", "abs", "RMW"), (0x56, "LSR", "dp,x", "RMW"), (0x5E, "LSR", "abs,x", "RMW"),
    (0x66, "ROR", "dp", "RMW"), (0x6A, "ROR", "acc", "N"), (0x6E, "ROR", "abs", "RMW"), (0x76, "ROR", "dp,x", "RMW"), (0x7E, "ROR", "abs,x", "RMW"),
    (0xC6, "DEC", "dp", "RMW"), (0x3A, "DEC", "acc", "N"), (0xCE, "DEC", "abs", "RMW"), (0xD6, "DEC", "dp,x", "RMW"), (0xDE, "DEC", "abs,x", "RMW"),
    (0xE6, "INC", "dp", "RMW"), (0x1A, "INC", "acc", "N"), (0xEE, "INC", "abs", "RMW"), (0xF6, "INC", "dp,x", "RMW"), (0xFE, "INC", "abs,x", "RMW"),
    (0x04, "TSB", "dp", "RMW"), (0x0C, "TSB", "abs", "RMW"), (0x14, "TRB", "dp", "RMW"), (0x1C, "TRB", "abs", "RMW"),
    (0x24, "BIT", "dp", "A"), (0x2C, "BIT", "abs", "A"), (0x34, "BIT", "dp,x", "A"), (0x3C, "BIT", "abs,x", "A"), (0x89, "BIT", "#", "A"),
    (0x64, "STZ", "dp", "W"), (0x74, "STZ", "dp,x", "W"), (0x9C, "STZ", "abs", "W"), (0x9E, "STZ", "abs,x", "W"),
    (0xA2, "LDX", "#", "X"), (0xA6, "LDX", "dp", "X"), (0xAE, "LDX", "abs", "X"), (0xB6, "LDX", "dp,y", "X"), (0xBE, "LDX", "abs,y", "X"),
    (0xA0, "LDY", "#", "X"), (0xA4, "LDY", "dp", "X"), (0xAC, "LDY", "abs", "X"), (0xB4, "LDY", "dp,x", "X"), (0xBC, "LDY", "abs,x", "X"),
    (0x86, "STX", "dp", "XW"), (0x8E, "STX", "abs", "XW"), (0x96, "STX", "dp,y", "XW"),
    (0x84, "STY", "dp", "XW"), (0x8C, "STY", "abs", "XW"), (0x94, "STY", "dp,x", "XW"),
    (0xE0, "CPX", "#", "X"), (0xE4, "CPX", "dp", "X"), (0xEC, "CPX", "abs", "X"),
    (0xC0, "CPY", "#", "X"), (0xC4, "CPY", "dp", "X"), (0xCC, "CPY", "abs", "X"),
    (0x10, "BPL", "rel", "CTL"), (0x30, "BMI", "rel", "CTL"), (0x50, "BVC", "rel", "CTL"), (0x70, "BVS", "rel", "CTL"),
    (0x90, "BCC", "rel", "CTL"), (0xB0, "BCS", "rel", "CTL"), (0xD0, "BNE", "rel", "CTL"), (0xF0, "BEQ", "rel", "CTL"),
    (0x80, "BRA", "rel", "CTL"), (0x82, "BRL", "rel16", "CTL"),
    (0x4C, "JMP", "abs", "CTL"), (0x5C, "JML", "long", "CTL"), (0x6C, "JMP", "(abs)", "CTL"), (0x7C, "JMP", "(abs,x)", "CTL"), (0xDC, "JML", "[abs]", "CTL"),
    (0x20, "JSR", "abs", "CTL"), (0x22, "JSL", "long", "CTL"), (0xFC, "JSR", "(abs,x)", "CTL"),
    (0x60, "RTS", "imp", "CTL"), (0x6B, "RTL", "imp", "CTL"), (0x40, "RTI", "imp", "CTL"),
    (0xC2, "REP", "#8", "N"), (0xE2, "SEP", "#8", "N"),
    (0x48, "PHA", "imp", "SA"), (0x68, "PLA", "imp", "SA"), (0xDA, "PHX", "imp", "SX"), (0xFA, "PLX", "imp", "SX"),
    (0x5A, "PHY", "imp", "SX"), (0x7A, "PLY", "imp", "SX"), (0x08, "PHP", "imp", "S"), (0x28, "PLP", "imp", "S"),
    (0x8B, "PHB", "imp", "S"), (0xAB, "PLB", "imp", "S"), (0x0B, "PHD", "imp", "S2"), (0x2B, "PLD", "imp", "S2"), (0x4B, "PHK", "imp", "S"),
    (0xF4, "PEA", "abs", "S2"), (0xD4, "PEI", "dp", "S2"), (0x62, "PER", "rel16", "S2"),
    (0x54, "MVN", "mv", "N"), (0x44, "MVP", "mv", "N"),
    (0x00, "BRK", "#8", "CTL"), (0x02, "COP", "#8", "CTL"), (0xCB, "WAI", "imp", "N"), (0xDB, "STP", "imp", "N"), (0x42, "WDM", "#8", "N"),
]:
    _op(code, name, mode, kind)
for code, name in [(0x18, "CLC"), (0x38, "SEC"), (0x58, "CLI"), (0x78, "SEI"), (0xB8, "CLV"), (0xD8, "CLD"), (0xF8, "SED"), (0xFB, "XCE"),
                   (0x88, "DEY"), (0xC8, "INY"), (0xCA, "DEX"), (0xE8, "INX"), (0xEA, "NOP"), (0xEB, "XBA"),
                   (0x8A, "TXA"), (0x98, "TYA"), (0x9A, "TXS"), (0x9B, "TXY"), (0xA8, "TAY"), (0xAA, "TAX"), (0xBA, "TSX"), (0xBB, "TYX"),
                   (0x1B, "TCS"), (0x3B, "TSC"), (0x5B, "TCD"), (0x7B, "TDC")]:
    _op(code, name, "imp", "N")

OPERAND_BYTES = {"imp": 0, "acc": 0, "#": 1, "#8": 1, "dp": 1, "dp,x": 1, "dp,y": 1, "(dp,x)": 1, "(dp),y": 1, "(dp)": 1, "[dp]": 1, "[dp],y": 1,
                 "sr,s": 1, "(sr,s),y": 1, "rel": 1, "rel16": 2, "abs": 2, "abs,x": 2, "abs,y": 2, "(abs)": 2, "(abs,x)": 2, "[abs]": 2,
                 "long": 3, "long,x": 3, "mv": 2}
POINTER_BYTES = {"(dp,x)": 2, "(dp),y": 2, "(dp)": 2, "[dp]": 3, "[dp],y": 3, "(sr,s),y": 2, "(abs)": 2, "(abs,x)": 2, "[abs]": 3}


def lorom(address):
    bank = (address >> 16) & 0x7F
    offset = address & 0xFFFF
    if offset < 0x8000:
        return None
    return bank * 0x8000 + (offset - 0x8000)


def access_cost(address):
    address &= 0xFFFFFF
    if address & 0x408000:
        return 8
    if (address + 0x6000) & 0x4000:
        return 8
    if (address - 0x4000) & 0x7E00:
        return 6
    return 12


class Instruction:
    def __init__(self, pc, opcode, name, mode, kind, operand, length, m8, x8):
        self.pc = pc
        self.opcode = opcode
        self.name = name
        self.mode = mode
        self.kind = kind
        self.operand = operand
        self.length = length
        self.m8 = m8
        self.x8 = x8

    def price(self, dp_low_nonzero=False):
        """Return (master_cycles, variable_extra, note). variable_extra is the
        master cycles a data-dependent page crossing would add (0 if none);
        for branches master is the not-taken cost and variable_extra the taken
        increment."""
        name, mode, kind = self.name, self.mode, self.kind
        cycles = CYCLES[self.opcode]
        bus = self.length  # opcode + operand fetches
        variable = 0
        note = ""
        wide_a = not self.m8
        wide_x = not self.x8
        if mode == "#" or mode == "#8":
            # A 16-bit immediate is one more fetch; `length` already counts it.
            if mode == "#" and ((kind in ("A",) and wide_a) or (kind == "X" and wide_x)):
                cycles += 1
        elif mode in ("dp", "dp,x", "dp,y", "(dp,x)", "(dp),y", "(dp)", "[dp]", "[dp],y") and dp_low_nonzero:
            cycles += 1
        if kind in ("A", "W", "X", "XW", "RMW") and mode not in ("#", "#8", "acc", "imp"):
            data = 1 + (1 if ((kind in ("A", "W", "RMW") and wide_a) or (kind in ("X", "XW") and wide_x)) else 0)
            if kind == "RMW":
                bus += 2 * data
                cycles += 1 if wide_a else 0
                cycles += 1 if wide_a else 0
            else:
                bus += data
                cycles += 1 if data == 2 else 0
            bus += POINTER_BYTES.get(mode, 0)
            if mode in ("abs,x", "abs,y", "(dp),y") and kind in ("A", "X"):
                if wide_x:
                    cycles += 1
                else:
                    variable = 6
                    note = "+6 if the index crosses a page"
            if mode in ("abs", "abs,x", "abs,y") and self.operand is not None \
                    and 0x2000 <= (self.operand & 0xFFFF) < 0x6000:
                # The data bank is unknown statically: with a system bank
                # this is a register-window access at 6 cycles each.
                accesses = data * (2 if kind == "RMW" else 1)
                note = f"-{2 * accesses} if the data bank is a system bank (register window)"
                return (cycles - bus) * 6 + bus * 8, variable, note
            if mode in ("long", "long,x") and self.operand is not None:
                # `abs` modes go through the data bank register, which the
                # static walk does not know; only long addresses are priced
                # by the bus rule (the register window costs 6 per access).
                per_access = access_cost(self.operand)
                if per_access != 8:
                    bus_delta = (per_access - 8) * data * (2 if kind == "RMW" else 1)
                    note = f"register window access ({per_access} each)"
                    return (cycles - bus) * 6 + bus * 8 + bus_delta, variable, note
        elif kind == "CTL":
            if mode == "rel":
                bus = 2
                if name == "BRA":
                    # Always taken: the table's 3 cycles already include it.
                    return (cycles - bus) * 6 + bus * 8, 0, ""
                return (cycles - bus) * 6 + bus * 8, 6, "taken +6"
            if mode == "rel16":
                bus = 3
            elif name == "JSR" and mode == "abs":
                bus = 3 + 2
            elif name == "JSR" and mode == "(abs,x)":
                bus = 3 + 2 + 2
            elif name == "JSL":
                bus = 4 + 3
            elif name == "RTS":
                bus = 1 + 2
            elif name == "RTL":
                bus = 1 + 3
            elif name == "RTI":
                # Native mode pulls P, PCL, PCH and PBR: one more cycle and
                # one more pull than the emulation-mode table entry.
                cycles += 1
                bus = 1 + 4
            elif name == "JMP" and mode == "abs":
                bus = 3
            elif name == "JML" and mode == "long":
                bus = 4
            elif name == "JMP" and mode == "(abs)":
                bus = 3 + 2
            elif name == "JMP" and mode == "(abs,x)":
                bus = 3 + 2
            elif name == "JML" and mode == "[abs]":
                bus = 3 + 3
        elif kind in ("SA", "SX", "S", "S2"):
            if kind == "SA" and wide_a or kind == "SX" and wide_x or kind == "S2":
                cycles += 1 if kind in ("SA", "SX") else 0
                bus = 1 + 2
            else:
                bus = 1 + 1
            if name in ("PEA", "PER"):
                bus = self.length + 2
            if name == "PEI":
                bus = self.length + 2 + 2
        elif kind == "N":
            if mode == "mv":
                bus = 3 + 2
                note = "per byte moved (7 cycles)"
        return (cycles - bus) * 6 + bus * 8, variable, note


class Symbols:
    def __init__(self, path):
        self.by_address = {}
        self.by_name = {}
        if path.is_file():
            for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
                match = re.match(r"^0x([0-9a-fA-F]{4,6}):\s*(\S+)", line)
                if match:
                    address = int(match.group(1), 16)
                    address = ((address >> 16) & 0x7F) << 16 | (address & 0xFFFF)
                    self.by_address.setdefault(address, match.group(2))
                    self.by_name.setdefault(match.group(2).lower(), address)
        self.addresses = sorted(self.by_address)

    def describe(self, pc):
        return self.by_address.get(pc, "")


def decode(rom, pc, m8, x8):
    offset = lorom(pc)
    if offset is None or offset >= len(rom):
        return None
    opcode = rom[offset]
    if opcode not in MODES:
        return None
    name, mode, kind = MODES[opcode]
    length = 1 + OPERAND_BYTES[mode]
    if mode == "#":
        length = 2 if ((kind == "A" and m8) or (kind == "X" and x8)) else 3
    raw = rom[offset + 1:offset + length]
    operand = None
    if mode == "rel":
        operand = (pc + 2 + (raw[0] - 256 if raw[0] > 127 else raw[0])) & 0xFFFFFF
    elif mode == "rel16":
        operand = (pc + 3 + int.from_bytes(raw, "little", signed=True)) & 0xFFFFFF
    elif mode in ("abs", "abs,x", "abs,y", "(abs)", "(abs,x)", "[abs]", "long", "long,x"):
        value = int.from_bytes(raw, "little")
        operand = value if mode.startswith("long") else ((pc & 0xFF0000) | value) if mode in ("abs", "(abs,x)") else value
    elif raw:
        operand = int.from_bytes(raw, "little")
    return Instruction(pc, opcode, name, mode, kind, operand, length, m8, x8)


def walk(rom, entry, m8, x8, limit=4000):
    """Follow control flow from entry; returns instructions keyed by pc with
    the flag state they were first reached in."""
    seen = {}
    work = [(entry, m8, x8)]
    order = []
    while work and len(seen) < limit:
        pc, m8, x8 = work.pop()
        if pc in seen:
            continue
        ins = decode(rom, pc, m8, x8)
        if ins is None:
            break
        seen[pc] = ins
        order.append(pc)
        nm8, nx8 = m8, x8
        if ins.name == "REP":
            nm8 = m8 and not (ins.operand & 0x20)
            nx8 = x8 and not (ins.operand & 0x10)
        elif ins.name == "SEP":
            nm8 = m8 or bool(ins.operand & 0x20)
            nx8 = x8 or bool(ins.operand & 0x10)
        nxt = pc + ins.length
        if ins.name in ("RTS", "RTL", "RTI", "STP"):
            continue
        if ins.name in ("JMP", "JML") and ins.mode in ("abs", "long"):
            work.append((ins.operand, nm8, nx8))
            continue
        if ins.name in ("JMP", "JML"):
            continue  # indirect: table dispatch
        if ins.name in ("BRA", "BRL"):
            work.append((ins.operand, nm8, nx8))
            continue
        if ins.mode == "rel":
            work.append((ins.operand, nm8, nx8))
        work.append((nxt, nm8, nx8))
    return seen, order


def format_operand(ins, symbols):
    if ins.operand is None:
        return ""
    if ins.mode in ("rel", "rel16", "abs", "long") and ins.kind == "CTL":
        name = symbols.describe(ins.operand)
        return f"${ins.operand:06x}" + (f" {name}" if name else "")
    if ins.mode in ("#", "#8"):
        return f"#${ins.operand:0{2 if ins.length == 2 else 4}x}"
    return f"{ins.mode.replace('dp', '$%02x' % ins.operand if ins.length == 2 else 'dp')}" if ins.mode.startswith(("dp", "(dp", "[dp")) else f"{ins.mode} ${ins.operand:x}"


def print_listing(rom, entry, m8, x8, symbols, dp_low_nonzero):
    seen, order = walk(rom, entry, m8, x8)
    leaders = {entry}
    for pc, ins in seen.items():
        if ins.kind == "CTL" and ins.mode in ("rel", "rel16") or ins.name in ("BRA", "BRL"):
            leaders.add(ins.operand)
            leaders.add(pc + ins.length)
        if ins.name in ("JMP", "JML", "RTS", "RTL", "RTI"):
            leaders.add(pc + ins.length)
    total = 0
    block = 0
    for pc in sorted(seen):
        ins = seen[pc]
        if pc in leaders:
            if block:
                print(f"        ; block total {block}")
            block = 0
            label = symbols.describe(pc)
            print(f"\n{pc:06x}: {label or ''}")
        master, variable, note = ins.price(dp_low_nonzero)
        block += master
        total += master
        extra = f" (+{variable}: {note})" if variable else (f" ({note})" if note else "")
        print(f"  {pc:06x}  {master:4d}{extra:34s} {ins.name} {format_operand(ins, symbols)}   [m{'8' if ins.m8 else '16'} x{'8' if ins.x8 else '16'}]")
    if block:
        print(f"        ; block total {block}")
    print(f"\n; {len(seen)} instructions reached from {entry:06x}; straight-line sum {total} (branches priced not taken)")


def verify(profile_dir, rom, symbols):
    """Compare static prices against the per-PC master/count of every profile."""
    observed = defaultdict(lambda: [0, 0])
    for path in Path(profile_dir).glob("host-*.json"):
        try:
            run = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        for row in run.get("instructions_by_pc", []):
            entry = observed[int(row["pc"], 16)]
            entry[0] += row["master"]
            entry[1] += row["count"]
    exact = 0
    variable = 0
    mismatches = []
    undecodable = 0
    for pc, (master, count) in sorted(observed.items()):
        # Decode under both flag widths; accept a price that matches either.
        candidates = set()
        for m8 in (True, False):
            for x8 in (True, False):
                ins = decode(rom, pc, m8, x8)
                if ins is None:
                    continue
                price, extra, note = ins.price()
                candidates.add(price)
                if extra:
                    candidates.add(price + extra)
                if note.startswith("-"):
                    candidates.add(price - int(note[1:].split()[0]))
        if not candidates:
            undecodable += 1
            continue
        per = master / count
        # An interrupt accepted at this address charges its entry cycles
        # here once; allow a small drift over many executions.
        if any(abs(per - candidate) < 0.05 for candidate in candidates):
            exact += 1
        elif min(candidates) <= per <= max(candidates) + 0.05:
            variable += 1
        else:
            mismatches.append((pc, per, sorted(candidates), count))
    print(f"pcs={len(observed)} exact={exact} mixed(branch/page)={variable} undecodable={undecodable} mismatches={len(mismatches)}")
    for pc, per, candidates, count in mismatches[:40]:
        ins = decode(rom, pc, True, True)
        print(f"  {pc:06x} {symbols.describe(pc):28s} observed={per:7.2f} x{count:6d} static={candidates} {ins.name if ins else '?'} {ins.mode if ins else ''}")
    return len(mismatches) == 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("entry", nargs="?")
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--symbols", type=Path, default=DEFAULT_SYMBOLS)
    parser.add_argument("--m16", action="store_true")
    parser.add_argument("--x16", action="store_true")
    parser.add_argument("--dp-nonzero", action="store_true")
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    rom = bytearray(args.rom.read_bytes())
    if len(rom) % 0x400 == 0x200:
        rom = rom[0x200:]
    symbols = Symbols(args.symbols)
    if args.verify:
        return 0 if verify(args.verify, rom, symbols) else 1
    if not args.entry:
        parser.error("an entry address or symbol is required")
    entry = symbols.by_name.get(args.entry.lower())
    if entry is None:
        entry = int(args.entry.replace("$", "").replace(":", ""), 16)
    print_listing(rom, entry, not args.m16, not args.x16, symbols, args.dp_nonzero)
    return 0


if __name__ == "__main__":
    sys.exit(main())
