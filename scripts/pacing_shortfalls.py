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
HDMA_DECLINE_REASONS = (
    'vwf', 'main-dma', 'song-upload', 'raster-wait', 'worker-change',
    'timed-work', 'program-write', 'no-entry-program', 'program-mismatch', 'disabled',
)


def refresh(master):
    return (master + 1323) // 1324 * 40


def structural_costs(source, predictor):
    """Apply the same refresh formula to source and native bus occupancy.

    The residual at source work is placement/endpoint uncertainty, not an
    instruction price. Lower/upper HDMA bounds preserve interrupt-edge ambiguity.
    main_master already includes the separate VWF counter; never add it again.
    """
    row = dict(source)
    native_base = int(predictor['main_master']) + int(predictor.get('cost_main_dma', 0))
    native_hdma = int(predictor.get('cost_main_hdma', 0))
    source_base = row['instructions'] + row['dma_master']
    residual = row['main_wall'] - source_base - row['refresh_master']
    lower, upper = max(0, residual), residual + 40 * row['interrupt_refresh_edges']
    if upper < lower:
        raise ValueError(f"iteration {row['iteration']}: invalid source HDMA bounds")
    row.update(native_dma_master=int(predictor.get('cost_main_dma', 0)),
               native_refresh_master=int(predictor.get('cost_refresh_main', refresh(native_base))),
               native_hdma_master=native_hdma,
               native_raster_wait_master=int(predictor.get('cost_raster_wait', 0)),
               other_bus_stalls=residual, hdma_lower_bound=lower, hdma_upper_bound=upper,
               refresh_rounding_at_source_work=row['refresh_master'] - refresh(source_base),
               refresh_work_difference=refresh(source_base) - refresh(native_base),
               refresh_hdma_difference=(refresh(source_base + lower) - refresh(source_base)
                                        - refresh(native_base + native_hdma) + refresh(native_base)),
               refresh_placement_lower=row['refresh_master'] - refresh(source_base + upper),
               refresh_placement_upper=row['refresh_master'] - refresh(source_base + lower))
    row['refresh_gap'] = row['refresh_master'] - row['native_refresh_master']
    row['refresh_native_report_error'] = refresh(native_base + native_hdma) - row['native_refresh_master']
    assert row['refresh_gap'] == sum(row[k] for k in (
        'refresh_work_difference', 'refresh_hdma_difference',
        'refresh_placement_upper', 'refresh_native_report_error'))
    return row


def structural_summary(structural, predictors):
    """Disjoint signature groups plus explicitly overlapping decline reasons."""
    totals = Counter()
    categories, signatures, reasons = (defaultdict(Counter) for _ in range(3))
    for index, row in structural.items():
        values = {k: v for k, v in row.items() if k != 'iteration'}
        values['iterations'] = 1
        totals.update(values)
        categories[predictors[index]['category']].update(values)
        if 'cost_hdma_decline' in predictors[index]:
            bits = int(predictors[index]['cost_hdma_decline'])
            if bits >> len(HDMA_DECLINE_REASONS):
                raise ValueError(f'iteration {index}: unknown HDMA decline bit')
            names = [name for bit, name in enumerate(HDMA_DECLINE_REASONS) if bits & (1 << bit)]
            signature = '+'.join(names) or 'admitted'
            signatures[signature].update(values)
            for bit, name in enumerate(HDMA_DECLINE_REASONS):
                if bits & (1 << bit):
                    reasons[name].update(values)
    return dict(totals=dict(totals), categories=dict(categories),
                hdma_decline_signatures=dict(signatures), hdma_decline_reasons=dict(reasons),
                note='Signatures are disjoint; individual decline reasons overlap and must not be summed.')


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
        structural = {i: structural_costs(row, missing[i]) for i, row in structural.items()}
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
        summary = structural_summary(structural, missing)
        report['structural_totals'] = summary.pop('totals')
        report['structural_summary'] = summary
    args.json.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ('total', 'bands')}, indent=2))
    if 'ranking' in report:
        for row in report['ranking']['routines'][:30]:
            print(row['routine'], row['exclusive_difference'], row['source_calls'], row['native_calls'])


if __name__ == '__main__':
    main()
