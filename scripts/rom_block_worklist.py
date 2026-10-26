#!/usr/bin/env python3
"""Rank executed ROM blocks, preserving source routine and M/X ownership.

Needs instruction-profile.csv from logical_frame_ledger with
ZELDA3_LEDGER_BLOCK_PROFILE=1 and the SAME ZELDA3_LEDGER_PROFILE selection.
Use --cohort batch-22/decomposition.json to freeze the missing cohort, including
subsequently closed rows. Refuses missing/incomplete receipts; routine totals
alone cannot recover block execution. Native markers are declarations unless
--native supplies the measured --routine-profile companion *.blocks.csv.
Legacy scalar prices are not block-resolved: absence of a marker does NOT
mean the block is unpriced. source_master ranks executed work, not a fitted
deficit; use routine coverage alongside it when choosing a conversion.
"""
import argparse
import csv
import json
import re
from collections import defaultdict
from pathlib import Path

from rom_block_costs import DEFAULT_ROM, decode
from routine_coverage_diff import load


def rank(rom, receipts, routines, cohort, declared, native):
    observed = defaultdict(lambda: [0, 0])
    totals = defaultdict(lambda: defaultdict(int))
    seen = set()
    dynamic_leaders = set()
    for row in receipts:
        index = int(row['iteration'])
        if index not in cohort:
            continue
        owner, pc = (int(row[k], 16) & 0x7fffff for k in ('routine', 'pc'))
        m8, x8 = (bool(int(row[k])) for k in ('m8', 'x8'))
        key = (owner, pc, m8, x8)
        if (index, key) in seen:
            raise ValueError(f'duplicate source PC receipt {index}/{key}')
        seen.add((index, key))
        count, master = int(row['count']), int(row['master'])
        entries = int(row['entries'])
        if entries < 0 or entries > count:
            raise ValueError('invalid source block-entry count')
        if entries:
            dynamic_leaders.add(pc)
        if count <= 0 or master <= 0:
            raise ValueError('source receipts require positive count/master')
        observed[key][0] += count
        observed[key][1] += master
        totals[index][f'{owner:06x}'] += master
    if set(totals) != cohort or set(routines) != cohort:
        raise ValueError('source PC receipts/routines do not cover the complete frozen cohort')
    for index in cohort:
        expected = {pc: v[2] for pc, v in routines[index].items() if pc != 'total' and v[2]}
        if dict(totals[index]) != expected:
            raise ValueError(f'iteration {index}: PC receipts do not reconcile to exclusive routine clocks')

    # Canonical leaders include every observed branch destination, call return
    # and routine entry. Different entry widths stay distinct in the ranking.
    leaders = {key[0] for key in observed} | dynamic_leaders
    decoded = {}
    for key in observed:
        _, pc, m8, x8 = key
        ins = decode(rom, pc, m8, x8)
        if ins is None:
            raise ValueError(f'{pc:06x}: cannot decode observed PC')
        decoded[key] = ins
        if ins.kind == 'CTL':
            leaders.add(pc + ins.length)
            if ins.mode in ('rel', 'rel16') or ins.name in ('JMP', 'JML', 'JSR', 'JSL'):
                leaders.add(ins.operand)
    # Walk observed straight-line edges; a width-changing instruction has a
    # width-correct successor, and a control instruction always ends its block.
    successor = {}
    predecessor = {}
    for key, ins in decoded.items():
        owner, pc, m8, x8 = key
        if ins.kind == 'CTL':
            continue
        if ins.name in ('REP', 'SEP'):
            if ins.operand & 0x20: m8 = ins.name == 'SEP'
            if ins.operand & 0x10: x8 = ins.name == 'SEP'
        nxt = (owner, pc + ins.length, m8, x8)
        if nxt in observed and nxt[1] not in leaders:
            if nxt in predecessor:
                raise ValueError(f'ambiguous straight-line predecessor at {nxt}')
            successor[key] = nxt
            predecessor[nxt] = key
    rows = []
    consumed = set()
    for key in sorted(observed):
        if key in predecessor:
            continue
        chain = [key]
        while chain[-1] in successor:
            chain.append(successor[chain[-1]])
        consumed.update(chain)
        owner, pc, m8, x8 = key
        end = chain[-1][1] + decoded[chain[-1]].length
        measured = None if native is None else sum(v for (frame, addr, kind), v in native.items()
            if frame == owner and ((kind == 0 and addr == pc) or (kind != 0 and pc <= addr < end)))
        rows.append(dict(routine=f'{owner:06x}', block=f'{pc:06x}', end=f'{end:06x}',
            m8=m8, x8=x8, count=observed[key][0],
            source_master=sum(observed[k][1] for k in chain),
            marker_declared=pc in declared, native_marker_master=measured))
    if consumed != set(observed):
        raise ValueError('unpartitioned source instructions')
    variants = defaultdict(int)
    for row in rows:
        variants[(row['routine'], row['block'])] += 1
    for row in rows:
        # Native receipts name table addresses, not a live CPU flags register.
        # Never assign one native charge twice to distinct source M/X entries.
        ambiguous = variants[(row['routine'], row['block'])] > 1
        row['native_width_ambiguous'] = ambiguous
        if ambiguous and row['native_marker_master']:
            row['native_marker_master'] = None
    return sorted(rows, key=lambda r: (-r['source_master'], r['block'], r['routine']))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('instructions', type=Path)
    parser.add_argument('routines', type=Path)
    parser.add_argument('--cohort', required=True, type=Path)
    parser.add_argument('--rom', type=Path, default=DEFAULT_ROM)
    parser.add_argument('--native', type=Path)
    parser.add_argument('--json', required=True, type=Path)
    parser.add_argument('--show', type=int, default=30)
    args = parser.parse_args()
    try:
        cohort = set(map(int, json.loads(args.cohort.read_text())['rows']))
        routines = load(args.routines, [(i, i) for i in cohort])
        declared = set()
        for path in Path('crates/zelda3/src').rglob('*.rs'):
            if path.name.endswith('_tests.rs') or path.name == 'cycle_ledger.rs':
                continue
            declared.update(int(pc.replace('_', ''), 16) for pc in
                re.findall(r'\bblocks\.block\(0x([0-9a-fA-F_]+)\)', path.read_text()))
        native = None
        if args.native:
            native = defaultdict(int)
            with args.native.open() as source:
                for row in csv.DictReader(source):
                    if int(row['iteration']) in cohort:
                        native[(int(row['routine'], 16), int(row['pc'], 16), int(row['kind']))] += int(row['master'])
        rom = args.rom.read_bytes()
        if len(rom) % 0x400 == 0x200: rom = rom[0x200:]
        with args.instructions.open() as source:
            rows = rank(rom, csv.DictReader(source), routines, cohort, declared, native)
        args.json.write_text(json.dumps(dict(iterations=len(cohort), blocks=rows), indent=2) + '\n')
        for row in rows[:args.show]:
            print('{block} M8={m8} X8={x8} owner={routine} source={source_master} '
                  'marker={marker_declared} native={native_marker_master}'.format(**row))
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))


if __name__ == '__main__':
    main()
