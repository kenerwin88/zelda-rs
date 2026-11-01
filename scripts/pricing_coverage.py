#!/usr/bin/env python3
"""Per-iteration PC-touch census, NOT a retirement matcher or pricing waiver.

Consumes logical_frame_ledger instruction-profile/--coverage-source CSVs and
current --logical-replay ZELDA3_SHADOW_AUDIT_SUMMARY receipts. Generated block
membership is decoded from the ROM; model PCs come from executed witnesses.
Opaque scalar work makes unmatched PCs ambiguous, never proved uncharged.
Multiple source files must agree exactly on overlapping iterations. The output
includes every selected iteration missing source/native evidence. All inputs
are read-only; output creation is exclusive. No counts become runtime prices.
"""
import argparse
import csv
import hashlib
import json
import subprocess
from collections import Counter, defaultdict
from pathlib import Path

from rom_block_costs import generated_blocks


def source_rows(paths, cohort):
    merged = {}
    for path in paths:
        rows = defaultdict(dict)
        with path.open() as stream:
            for r in csv.DictReader(stream):
                i = int(r['iteration'])
                if i not in cohort:
                    continue
                key = (int(r['routine'], 16) & 0x7fffff,
                       int(r['pc'], 16) & 0x7fffff, int(r['m8']), int(r['x8']))
                if key in rows[i]:
                    raise ValueError(f'duplicate source PC {path}:{i}:{key}')
                value = (int(r['count']), int(r['master']))
                if min(value) <= 0:
                    raise ValueError('nonpositive source receipt')
                rows[i][key] = value
        for i, row in rows.items():
            if i in merged and merged[i] != row:
                raise ValueError(f'source witnesses disagree at iteration {i}')
            merged[i] = row
    return merged


def source_totals(paths, cohort):
    totals = {}
    for path in paths:
        if path.suffix == '.json':
            rows = ((r['iteration'], r['source_main'])
                    for r in json.loads(path.read_text())['iterations'])
        else:
            with path.open() as stream:
                rows = [(int(r['iteration']), int(r['exclusive']))
                        for r in csv.DictReader(stream) if r['routine'] == 'total']
        for i, clocks in rows:
            if i not in cohort:
                continue
            if i in totals and totals[i] != clocks:
                raise ValueError(f'independent source totals disagree at {i}')
            totals[i] = clocks
    return totals


def verify_source_totals(sources, totals):
    for i, rows in sources.items():
        if i not in totals or sum(value[1] for value in rows.values()) != totals[i]:
            raise ValueError(f'per-PC source does not reconcile to independent total at {i}')


def json_rows(path):
    if path.suffix != '.zst':
        with path.open() as f:
            yield from map(json.loads, f)
        return
    process = subprocess.Popen(['zstd', '-dc', str(path)], stdout=subprocess.PIPE, text=True)
    try:
        yield from map(json.loads, process.stdout)
    finally:
        process.stdout.close()
        if process.wait() != 0:
            raise ValueError(f'zstd decode failed: {path}')


def native_touch(row, blocks):
    """Presence only. No cost-based selection or claimed occurrence pairing."""
    markers, models = set(), set()
    marker_master = 0
    for _path, pc, kind, width, value in row['blocks']:
        clocks, count = value
        marker_master += clocks
        if kind == 0 and count:
            instructions = blocks[pc]
            if tuple(width) != instructions[0][1:]:
                raise ValueError(f'generated width mismatch at {pc:06x}')
            markers.update(instructions)
    model_master = sum(v[0] for _path, _site, v in row['model_sites'])
    mapped_model_master = 0
    for _path, _site, pc, child, value in row['model_steps']:
        if pc is not None:
            models.add(pc & 0x7fffff)
            mapped_model_master += value[0]
    scalar_master = sum(v[0] for _path, _site, v in row['scalar_sites'])
    if marker_master + model_master + scalar_master != row['ledger_master']:
        raise ValueError(f'native charge classes do not conserve iteration {row["iteration"]}')
    if mapped_model_master > model_master:
        raise ValueError('model instruction clocks exceed model charges')
    return markers, models, scalar_master + model_master - mapped_model_master


def scalar_touch(row, catalog):
    touched, master = set(), 0
    for path, site, (clocks, count) in row['scalar_sites']:
        if site not in catalog:
            continue
        owner, per_charge, instructions = catalog[site]
        if not path or path[-1] != owner or clocks != count * per_charge:
            raise ValueError(f'scalar footprint contract changed: {site}')
        if count:
            touched.update((owner, pc, m, x) for pc, m, x in instructions)
            master += clocks
    return touched, master


def partition(source, markers, models, opaque, scalars=()):
    rows = Counter()
    owners = defaultdict(Counter)
    for (owner, pc, m8, x8), (_count, clocks) in source.items():
        if (pc, bool(m8), bool(x8)) in markers:
            kind = 'marker_touched'
        elif pc in models:
            kind = 'model_touched'
        elif (owner, pc, bool(m8), bool(x8)) in scalars:
            kind = 'scalar_touched'
        elif opaque:
            kind = 'unresolved_alias'
        else:
            kind = 'uncovered'
        rows[kind] += clocks
        owners[owner][kind] += clocks
    return rows, owners


def identity(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return dict(path=str(path.resolve()), sha256=digest.hexdigest(), bytes=path.stat().st_size)


def census(sources, natives, cohort, blocks, predictors=None, scalar_catalog=None):
    totals, owners = Counter(), defaultdict(Counter)
    scalar_sites = defaultdict(Counter)
    iterations, seen = [], set()
    prefixes = {end: Counter() for end in (94334, 141568, 228499, 380243, 461989, 462627)}
    for n in natives:
        i = n['iteration']
        if i not in cohort:
            raise ValueError(f'unselected native iteration {i}')
        if i in seen:
            raise ValueError(f'duplicate native iteration {i}')
        seen.add(i)
        if i not in sources:
            continue
        markers, models, opaque = native_touch(n, blocks)
        scalars, known_scalar_master = scalar_touch(n, scalar_catalog or {})
        opaque -= known_scalar_master
        if opaque < 0:
            raise ValueError('scalar footprint exceeds opaque work')
        separate_vwf = 0
        if predictors is not None:
            prediction = predictors[i]
            separate_vwf = int(prediction['vwf_master'])
            if n['ledger_master'] + separate_vwf != int(prediction['main_master']):
                raise ValueError(f'native observer/predictor disagreement at {i}')
            opaque += separate_vwf
        for path, site, value in n['scalar_sites']:
            key = (tuple(path), site)
            scalar_sites[key]['native_master'] += value[0]
            scalar_sites[key]['charges'] += value[1]
            scalar_sites[key]['missing_rows'] += 1
            if i < 462627:
                scalar_sites[key]['before_462627_master'] += value[0]
        classes, by_owner = partition(sources[i], markers, models, opaque, scalars)
        source = sum(v[1] for v in sources[i].values())
        residual = source - sum(classes.values())
        if residual:
            raise ValueError('source classification conservation')
        native = n['ledger_master'] + separate_vwf
        iterations.append(dict(iteration=i, source_main=source, native_main=native,
            signed_difference=source-native, opaque_native=opaque, separate_vwf=separate_vwf,
            residual=residual, **classes))
        totals.update(classes)
        for end, prefix in prefixes.items():
            if i < end:
                prefix.update(classes)
                prefix['missing_rows'] += 1
        for owner, work in by_owner.items():
            owners[owner].update(work)
            if work['unresolved_alias'] or work['uncovered']:
                owners[owner]['missing_rows'] += 1
                if i < 462627:
                    owners[owner]['before_462627_rows'] += 1
                    owners[owner]['before_462627_unresolved'] += work['unresolved_alias']
                    owners[owner]['before_462627_uncovered'] += work['uncovered']
    ranked = [dict(routine=f'{owner:06x}', **work) for owner, work in owners.items()]
    ranked.sort(key=lambda r: (-r.get('uncovered', 0), -r.get('unresolved_alias', 0), r['routine']))
    return dict(schema=1, semantics='PC touch in the same iteration; not occurrence/retirement equivalence',
        scalar_rule='Any opaque native work leaves unmatched source PCs unresolved; no scope-only exclusion',
        selected_rows=len(cohort), analyzed_rows=len(iterations), missing_source=sorted(cohort-sources.keys()),
        missing_native=sorted(cohort-seen), totals=totals, prefixes=prefixes, iterations=iterations,
        source_main=sum(r['source_main'] for r in iterations),
        native_main=sum(r['native_main'] for r in iterations),
        signed_difference=sum(r['signed_difference'] for r in iterations),
        prefix_routines=sorted(ranked, key=lambda r: (-r.get('before_462627_uncovered', 0),
            -r.get('before_462627_unresolved', 0), r['routine'])),
        routines=ranked, scalar_sites=sorted([dict(path=[f'{pc:06x}' for pc in path],site=site,**v)
            for (path,site),v in scalar_sites.items()],key=lambda r:-r['native_master']), pricing_admitted=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, nargs='+', required=True)
    parser.add_argument('--source-totals', type=Path, nargs='+', required=True,
                        help='coverage-source receipt JSON and/or routine-profile CSV')
    parser.add_argument('--native', type=Path, nargs='+', required=True)
    parser.add_argument('--cohort', type=Path, required=True)
    parser.add_argument('--predictor', type=Path, nargs='+', required=True)
    parser.add_argument('--rom', type=Path, default=Path('saves/zelda3.sfc'))
    parser.add_argument('--entries', type=Path, default=Path('scripts/rom_block_entries.json'))
    parser.add_argument('--scalar-footprints', type=Path, default=Path('scripts/pricing_scalar_footprints.json'))
    parser.add_argument('--json', type=Path, required=True)
    args = parser.parse_args()
    cohort = set(json.loads(args.cohort.read_text())['rows'])
    rom = args.rom.read_bytes()
    blocks = {b[0].pc: [(x.pc, x.m8, x.x8) for x in b]
              for b in generated_blocks(rom, json.loads(args.entries.read_text()))}
    scalar_catalog = {}
    scalar_sources = []
    for entry in json.loads(args.scalar_footprints.read_text()):
        path = Path(entry['source'])
        if identity(path)['sha256'] != entry['source_sha256']:
            raise ValueError(f'scalar source changed; review footprint: {path}')
        scalar_sources.append(path)
        instructions = [x for block in generated_blocks(rom, [entry]) for x in block]
        if any(x.name in ('JSR', 'JSL') for x in instructions):
            raise ValueError('scalar leaf footprint cannot include children')
        if entry['site'] in scalar_catalog:
            raise ValueError('duplicate scalar footprint site')
        scalar_catalog[entry['site']] = (int(entry['routine'], 16), entry['master_per_charge'],
            [(x.pc, x.m8, x.x8) for x in instructions])
    predictors = {}
    for path in args.predictor:
        with path.open() as stream:
            for row in csv.DictReader(stream):
                i = int(row['iteration'])
                if i in cohort:
                    if i in predictors:
                        raise ValueError(f'duplicate predictor iteration {i}')
                    predictors[i] = row
    sources = source_rows(args.source, cohort)
    verify_source_totals(sources, source_totals(args.source_totals, cohort))
    report = census(sources,
        (r for p in args.native for r in json_rows(p)), cohort, blocks, predictors, scalar_catalog)
    report['independent_source_totals_verified'] = True
    report['identities'] = [identity(p) for p in args.source + args.source_totals + args.native + args.predictor +
        [args.cohort, args.rom, args.entries, args.scalar_footprints, Path(__file__)] + scalar_sources]
    with args.json.open('x') as out:
        json.dump(report, out, indent=2)
        out.write('\n')
    print(json.dumps({k: report[k] for k in ('selected_rows', 'analyzed_rows', 'totals')}))
    print('routine,uncovered,unresolved_alias,missing_rows,before_462627_unresolved')
    for row in report['routines'][:50]:
        print(','.join(str(row.get(k, 0)) for k in
            ('routine','uncovered','unresolved_alias','missing_rows','before_462627_unresolved')))


if __name__ == '__main__':
    main()
