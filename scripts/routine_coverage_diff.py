#!/usr/bin/env python3
"""Rank source CPU minus lagless instruction coverage for identical iterations.

Inputs are logical_frame_ledger's routine-profile.csv and --logical-replay's
--routine-profile CSV. All clocks are master clocks, exclusive of interrupts,
polygon work, refresh and DMA. Inclusive rows overlap: never sum their deficits
to derive missing clocks or add a parent deficit on top of its children.
"""

import argparse
from bisect import bisect_right
import csv
import json
from collections import defaultdict
from pathlib import Path


def windows(text):
    result = []
    for item in text.split(','):
        lo, _, hi = item.partition('-')
        lo, hi = int(lo), int(hi or lo)
        if lo < 0 or hi < lo:
            raise ValueError('windows must be nonnegative and ordered')
        result.append((lo, hi))
    return result


def load(path, selected):
    # Merge ranges once: a sparse sample may select hundreds of windows in a
    # full-route CSV. Avoid scanning every window for every routine record.
    merged = []
    for lo, hi in sorted(selected):
        if merged and lo <= merged[-1][1] + 1:
            merged[-1][1] = max(hi, merged[-1][1])
        else:
            merged.append([lo, hi])
    starts = [lo for lo, _ in merged]
    rows = defaultdict(dict)
    with path.open() as source:
        for row in csv.DictReader(source):
            index = int(row['iteration'])
            if merged:
                window = bisect_right(starts, index) - 1
                if window < 0 or index > merged[window][1]:
                    continue
            address = row['routine'].lower()
            if address not in ('total', 'vwf'):
                address = f'{int(address, 16) & 0x7fffff:06x}'
            if address in rows[index]:
                raise ValueError(f'{path}: duplicate iteration/address {index}/{address}')
            rows[index][address] = tuple(int(row[k]) for k in ('calls', 'inclusive', 'exclusive'))
    for index, routines in rows.items():
        if 'total' not in routines:
            raise ValueError(f'{path}: incomplete iteration {index} (no total)')
        measured = sum(value[2] for key, value in routines.items() if key != 'total')
        if measured != routines['total'][2]:
            raise ValueError(f'{path}: iteration {index}: exclusive sum {measured} != total {routines["total"][2]}')
    return dict(rows)


def compare(source, native):
    if not source and not native:
        raise ValueError('no complete iterations selected')
    if source.keys() != native.keys():
        raise ValueError(f'iteration sets differ: source-only {sorted(source.keys() - native.keys())[:12]}, '
                         f'native-only {sorted(native.keys() - source.keys())[:12]}')
    totals = defaultdict(lambda: [0] * 6)
    iterations = []
    for index in sorted(source):
        s, n = source[index], native[index]
        iterations.append(dict(iteration=index, source=s['total'][2], native=n['total'][2],
                               difference=s['total'][2] - n['total'][2]))
        for address in s.keys() | n.keys():
            if address == 'total':
                continue
            a, b = s.get(address, (0, 0, 0)), n.get(address, (0, 0, 0))
            for k, value in enumerate(a + b):
                totals[address][k] += value
    ranking = []
    for address, (sc, si, se, nc, ni, ne) in totals.items():
        ranking.append(dict(routine=address, source_calls=sc, native_calls=nc,
                            source_inclusive=si, native_inclusive=ni, inclusive_difference=si-ni,
                            source_exclusive=se, native_exclusive=ne, exclusive_difference=se-ne,
                            no_native_scope=address not in ('000000', 'vwf') and sc > 0 and nc == 0))
    ranking.sort(key=lambda r: (-r['exclusive_difference'], r['routine']))
    return dict(iterations=iterations, source_total=sum(r['source'] for r in iterations),
                native_total=sum(r['native'] for r in iterations),
                difference=sum(r['difference'] for r in iterations), routines=ranking)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('native', type=Path)
    parser.add_argument('--windows', type=windows, default=[])
    parser.add_argument('--json', type=Path)
    parser.add_argument('--csv', type=Path)
    parser.add_argument('--show', type=int, default=30)
    parser.add_argument('--sort', choices=['exclusive', 'inclusive'], default='exclusive')
    args = parser.parse_args()
    try:
        report = compare(load(args.source, args.windows), load(args.native, args.windows))
    except ValueError as error:
        parser.error(str(error))
    report['routines'].sort(key=lambda row: (-row[args.sort + '_difference'], row['routine']))
    if args.json:
        args.json.write_text(json.dumps(report, indent=2) + '\n')
    if args.csv and report['routines']:
        with args.csv.open('w') as output:
            writer = csv.DictWriter(output, fieldnames=list(report['routines'][0]))
            writer.writeheader()
            writer.writerows(report['routines'])
    print(f"iterations={len(report['iterations'])} source={report['source_total']} "
          f"native={report['native_total']} difference={report['difference']}")
    print('routine  source_excl native_excl difference source_incl native_incl difference status')
    for row in report['routines'][:args.show]:
        print('{routine:8} {source_exclusive:11} {native_exclusive:11} {exclusive_difference:11} '
              '{source_inclusive:11} {native_inclusive:11} {inclusive_difference:11} {status}'.format(
                  **row, status='NO NATIVE SCOPE' if row['no_native_scope'] else ''))
    missing = [row for row in report['routines'] if row['no_native_scope']]
    print(f'source targets without any native scope: {len(missing)} (all retained in JSON/CSV)')


if __name__ == '__main__':
    main()
