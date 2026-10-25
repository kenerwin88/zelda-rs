#!/usr/bin/env python3
"""Summarize P2 shortfalls and rank instruction deficits only on missing holds.

The replay CSV records an inverse of the completed native model: the minimum
extra main clocks appended after its existing ownership checkpoints, with its
worker share and raster wait held fixed. This is a diagnostic counterfactual,
not a price, a measured source instruction deficit, or a runtime input.
"""

import argparse
from collections import Counter, defaultdict
import csv
import json
from pathlib import Path

from routine_coverage_diff import compare, load

FIELD = 1364 * 262


def band(shortfall):
    if shortfall * 20 < FIELD:
        return 'near-miss'
    return 'moderate' if shortfall * 2 < FIELD else 'large'


def summarize(rows):
    missing = {}
    categories = defaultdict(Counter)
    bands = defaultdict(Counter)
    total = Counter()
    for row in rows:
        index, lag, holds = (int(row[k]) for k in ('iteration', 'lag_nmis', 'cost_p2'))
        if int(row['native_held']) != holds:
            raise ValueError(f'iteration {index}: runtime holds differ from P2')
        if int(row['cost_elapsed_p2']) // FIELD != holds:
            raise ValueError(f'iteration {index}: diagnostic model differs from completed P2')
        if holds >= lag:
            continue
        shortfall = int(row['cost_shortfall'])
        if shortfall <= 0 or index in missing:
            raise ValueError(f'iteration {index}: invalid or duplicate shortfall')
        category = row['module'] + '/' + row['submodule']
        label = band(shortfall)
        missing[index] = dict(row, category=category, band=label)
        for counter in (total, categories[category], bands[label]):
            counter.update(iterations=1, missing=lag-holds, shortfall_master=shortfall)
        categories[category][label] += 1
    return missing, dict(field_master=FIELD,
                         bands_definition={'near-miss': '<5% field',
                                           'moderate': '5% to <50% field',
                                           'large': '>=50% field'},
                         total=dict(total), bands=dict(bands), categories=dict(categories))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('predictor', type=Path)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--native', type=Path)
    parser.add_argument('--structural', type=Path)
    parser.add_argument('--ledger', type=Path)
    parser.add_argument('--json', type=Path, required=True)
    args = parser.parse_args()
    with args.predictor.open() as f:
        missing, report = summarize(csv.DictReader(f))
    report['missing_iterations'] = missing
    if bool(args.source) != bool(args.native):
        parser.error('--source and --native must be supplied together')
    if args.source:
        windows = [(i, i) for i in missing]
        source, native = (load(args.source, windows), load(args.native, windows)) if missing else ({}, {})
        ranking = compare(source, native) if missing else dict(
            iterations=[], source_total=0, native_total=0, difference=0, routines=[])
        ranking['population'] = len(missing)
        ranking['profiled_missing_iterations'] = len(source)
        ranking['unprofiled_iterations'] = sorted(missing.keys() - source.keys())
        ranking['note'] = ('Signed exclusive deficits on profiled missing-hold iterations only. '
                           'Aliases, differing work and separately owned SPC polling require '
                           'manual resolution; inclusive deficits overlap and are not additive.')
        report['ranking'] = ranking
    if args.structural:
        with args.structural.open() as f:
            structural = {int(r['iteration']): {k: int(v) for k, v in r.items()}
                          for r in csv.DictReader(f) if int(r['iteration']) in missing}
        for i, row in structural.items():
            predictor = missing[i]
            row['native_dma_master'] = int(predictor.get('cost_main_dma', 0))
            row['native_refresh_master'] = int(predictor.get('cost_refresh_main',
                ((int(predictor['main_master']) + row['native_dma_master'] + 1323) // 1324) * 40))
            if 'cost_raster_wait' in predictor:
                row['native_raster_wait_master'] = int(predictor['cost_raster_wait'])
            row['other_bus_stalls'] = (row['main_wall'] - row['instructions']
                                       - row['refresh_master'] - row['dma_master'])
            row['hdma_lower_bound'] = max(0, row['other_bus_stalls'])
            row['hdma_upper_bound'] = (row['other_bus_stalls']
                + 40 * row['interrupt_refresh_edges'])
            row['refresh_rounding_at_source_work'] = (row['refresh_master']
                - (row['instructions'] + row['dma_master'] + 1323) // 1324 * 40)
        if args.ledger:
            with args.ledger.open() as f:
                for line in f:
                    entry = json.loads(line)
                    row = structural.get(entry['index'])
                    if row is not None:
                        row['source_host_width'] = entry['close_host'] - entry['open_host']
                        row['source_host_wall_master'] = row['source_host_width'] * FIELD
                        row['source_irqs'] = entry['irqs']
                        row['source_host_width_minus_lag_fields'] = row['source_host_width'] - entry['lag_nmis'] - 1
        report['structural'] = structural
        totals = Counter()
        for row in structural.values():
            totals.update({k: v for k, v in row.items() if k != 'iteration'})
        report['structural_totals'] = dict(totals)
    args.json.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ('total', 'bands')}, indent=2))
    if 'ranking' in report:
        for row in report['ranking']['routines'][:30]:
            print(row['routine'], row['exclusive_difference'], row['source_calls'], row['native_calls'])


if __name__ == '__main__':
    main()
