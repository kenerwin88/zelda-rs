#!/usr/bin/env python3
"""Measure census command publication timing; run both lanes or analyze saved traces.

No renderer, source CPU, external emulator or oracle-cache writer is invoked.
Source writes have order but no individual bus timestamps: reported point offsets
use the iteration's opening host, and every match also carries conservative bounds.
"""
import argparse
import bisect
import collections
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FRAME_MS = 1000 / 60
LOOKAHEAD = 64
MAX_ITERATION_GAP = 32


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def read_trace(path):
    with subprocess.Popen(['zstd', '-dc', str(path)], stdout=subprocess.PIPE) as p:
        for line in p.stdout:
            yield json.loads(line)
        if p.wait():
            raise ValueError(f'incomplete or corrupt trace: {path}')


def source_commands(writes):
    """Keep runtime port writes; remove explicit IPL block transfers, not long NMIs.

The upload starts with ports 2,3,1,0 (destination, run flag, token CC).
It ends at the same header with run flag zero. Port-0 FF immediately before
it remains a stop/upload request. Full payload bytes remain in the input trace.
"""
    commands, transport = [], []
    i = 0
    while i < len(writes):
        header = writes[i:i + 4]
        if ([p for p, _ in header] == [2, 3, 1, 0]
                and header[2][1] != 0 and header[3][1] == 0xcc):
            start = i
            i += 4
            while i + 4 <= len(writes):
                h = writes[i:i + 4]
                if [p for p, _ in h] == [2, 3, 1, 0] and h[2][1] == 0:
                    i += 4
                    break
                i += 1
            else:
                raise ValueError('unterminated source bank transfer')
            transport.append((start, i))
        else:
            commands.append((i, *writes[i]))
            i += 1
    return commands, transport


def kind(port, value):
    if port == 0:
        return 'song_change' if value < 0xf0 else 'music_control'
    return ('ambient', 'sfx1', 'sfx2')[port - 1]


def transition_groups(e):
    """Source-grounded cue values and module contexts; groups may overlap."""
    groups = []
    if e['port'] == 0 and e['value'] == 0x13:
        groups.append('receipt_fanfare_13')
    # misc.rs::complete_ancilla_add_item_receipt. These values can also be
    # reused by other callers, so retain module/room context with every row.
    if (e['port'], e['value'] & 0x3f) in ((3, 0x0f), (2, 0x2c)):
        groups.append('item_get_cue_values')
    if e['port'] == 0:
        if e.get('dungeon_entry_track'):
            groups.append('dungeon_entry_track_request')
        if e['context'][0] == 0x06:
            groups.append('dungeon_entry_module_06')
        if e['context'][0] in (0x19, 0x1a):
            groups.append('ending_modules_19_1a')
    return groups


def event(row, side, port, value, frame, upper, epoch, origin, order):
    return dict(iteration=row['iteration'], port=port, value=value,
                kind=kind(port, value), frame=frame, frame_upper=upper,
                epoch=epoch, epoch_frame=frame - origin, order=order,
                context=row[f'{side}_context'])


def align(source, native):
    """Bounded, monotone, content-only pairing with an iteration-distance guard.

Equal (port,value) events pair first. At a mismatch choose the next equal
anchor requiring the fewest skips, then the smallest iteration distance.
Never cross a native reseed epoch. Unpaired events are retained, not forgiven.
Repeated equal commands do not have unique identities; this is an explicit
alignment convention, not proof that each duplicate is the same physical event.
"""
    a = b = 0
    positions = collections.defaultdict(list)
    iterations = collections.defaultdict(list)
    for j, y in enumerate(native):
        key = y['port'], y['value'], y['epoch']
        positions[key].append(j)
        iterations[key].append(y['iteration'])
    def equal(i, j):
        x, y = source[i], native[j]
        return ((x['port'], x['value'], x['epoch']) ==
                (y['port'], y['value'], y['epoch']) and
                abs(x['iteration'] - y['iteration']) <= MAX_ITERATION_GAP)
    while a < len(source) and b < len(native):
        if equal(a, b):
            yield source[a], native[b]
            a += 1
            b += 1
            continue
        candidates = []
        # For a fixed source index, the earliest eligible native occurrence
        # minimizes skips. Binary searches avoid repeatedly scanning persistent
        # equal-valued queue publications in the raw-write inventory.
        for i in range(a, min(a + LOOKAHEAD, len(source))):
            x = source[i]
            key = x['port'], x['value'], x['epoch']
            js, its = positions[key], iterations[key]
            k = max(bisect.bisect_left(js, b),
                    bisect.bisect_left(its, x['iteration'] - MAX_ITERATION_GAP))
            if k < len(js) and js[k] < b + LOOKAHEAD and its[k] <= x['iteration'] + MAX_ITERATION_GAP:
                j = js[k]
                candidates.append((i - a + j - b, abs(x['iteration'] - native[j]['iteration']), i, j))
        if candidates:
            _, _, i, j = min(candidates)
            while a < i:
                yield source[a], None
                a += 1
            while b < j:
                yield None, native[b]
                b += 1
        elif (source[a]['epoch'], source[a]['iteration']) <= (native[b]['epoch'], native[b]['iteration']):
            yield source[a], None
            a += 1
        else:
            yield None, native[b]
            b += 1
    for x in source[a:]:
        yield x, None
    for y in native[b:]:
        yield None, y


def bucket(offset):
    d = abs(offset)
    return 'exact' if d == 0 else 'one_frame' if d == 1 else 'two_to_five' if d <= 5 else 'over_five'


def comparison(x, y):
    if x is None or y is None:
        return dict(status='native_only' if x is None else 'source_only', source=x, native=y)
    d = y['frame'] - x['frame']
    result = dict(status='matched', source=x, native=y, offset_frames=d,
                offset_ms=d * FRAME_MS,
                offset_bounds_frames=[y['frame'] - x['frame_upper'], d],
                offset_bounds_ms=[(y['frame'] - x['frame_upper']) * FRAME_MS, d * FRAME_MS],
                epoch_offset_frames=y['epoch_frame'] - x['epoch_frame'],
                iteration_offset=y['iteration'] - x['iteration'])
    if x.get('request') is not None and y.get('request') is not None:
        sr, nr = x['request'], y['request']
        if sr['value'] == nr['value']:
            sd, nd = x['frame'] - sr['frame'], y['frame'] - nr['frame']
            result.update(source_request_to_end_frames=sd, native_request_to_end_frames=nd,
                          request_relative_offset_frames=nd - sd,
                          request_relative_offset_ms=(nd - sd) * FRAME_MS,
                          request_relative_bounds_frames=[nd - (x['frame_upper'] - sr['frame']),
                                                          nd - (x['frame'] - sr['frame_upper'])])
    return result


def summarize(matches):
    result = collections.Counter()
    worst = []
    for r in matches:
        result[r['status']] += 1
        if r['status'] != 'matched':
            continue
        d = r['offset_frames']
        result[bucket(d)] += 1
        result['early' if d < 0 else 'late' if d > 0 else 'on_anchor'] += 1
        result['epoch_' + bucket(r['epoch_offset_frames'])] += 1
        result['iteration_' + bucket(r['iteration_offset'])] += 1
        result['iteration_early' if r['iteration_offset'] < 0 else
               'iteration_late' if r['iteration_offset'] > 0 else 'iteration_equal'] += 1
        if 'request_relative_offset_frames' in r:
            duration = r['request_relative_offset_frames']
            result['duration_' + bucket(duration)] += 1
            result['duration_early' if duration < 0 else 'duration_late' if duration > 0 else 'duration_equal'] += 1
        if 'gap_offset_frames' in r:
            result['gap_' + bucket(r['gap_offset_frames'])] += 1
            lo, hi = r['gap_bounds_frames']
            result['gap_definitely_within_one'] += lo >= -1 and hi <= 1
        lo, hi = r['offset_bounds_frames']
        result['definitely_early' if hi < 0 else 'definitely_late' if lo > 0 else 'bounds_include_zero'] += 1
        result['uncertain_source_frame'] += lo != hi
        worst.append(r)
    for name in ('matched', 'source_only', 'native_only', 'exact', 'one_frame', 'two_to_five', 'over_five',
                 'early', 'late', 'on_anchor', 'definitely_early', 'definitely_late', 'bounds_include_zero'):
        result.setdefault(name, 0)
    result['within_one_inclusive'] = result['exact'] + result['one_frame']
    result['iteration_within_one_inclusive'] = result['iteration_exact'] + result['iteration_one_frame']
    result['epoch_within_one_inclusive'] = result['epoch_exact'] + result['epoch_one_frame']
    result['worst_five'] = sorted(worst, key=lambda r: abs(r['offset_frames']), reverse=True)[:5]
    if worst:
        result['min_offset_frames'] = min(r['offset_frames'] for r in worst)
        result['max_offset_frames'] = max(r['offset_frames'] for r in worst)
        absolute = sorted(abs(r['offset_frames']) for r in worst)
        result['absolute_p50_frames'] = absolute[(len(absolute) - 1) // 2]
        result['absolute_p95_frames'] = absolute[(len(absolute) - 1) * 95 // 100]
    return dict(result)


def write_jsonl(path, rows):
    with gzip.open(path, 'wt') as f:
        for row in rows:
            f.write(json.dumps(row, separators=(',', ':')) + '\n')


def analyze(path, output):
    source, native, source_raw, native_raw = [], [], [], []
    polls = {'source': [], 'native': []}
    previous = {'source': [0] * 4, 'native': [0] * 4}
    poll_active = {'source': False, 'native': False}
    requests = {'source': None, 'native': None}
    dungeon_entry = {'source': False, 'native': False}
    totals = collections.Counter()
    epochs, transfers = [], []
    origin = None
    expected = 0
    for row in read_trace(path):
        if row['schema'] != 1 or row['iteration'] != expected:
            raise ValueError('trace schema/iteration gap')
        expected += 1
        if row['lost_pushes']:
            raise ValueError('trace missed intermediate native pushes')
        if row['callbacks'] != 1 + row['held'] + row['masked']:
            raise ValueError('callback accounting mismatch')
        if origin is None:
            origin = row['source_start']
        sframe = row['source_start'] - origin
        end = row['source_end']
        if end is None:
            # Last source interval is right-censored. Do not invent its width.
            if any(v for _, v in row['source_writes']) or row['source_reads']:
                raise ValueError('last source interval has events but no closing host')
            end = row['source_start']
        upper = end - origin
        if not epochs or row['reseeded']:
            epochs.append(dict(iteration=row['iteration'], source_frame=sframe,
                               native_frame=row['native_start']))
            previous = {'source': [0] * 4, 'native': [0] * 4}
            poll_active = {'source': False, 'native': False}
            requests = {'source': None, 'native': None}
            dungeon_entry = {'source': False, 'native': False}
        for side in ('source', 'native'):
            module = row[f'{side}_context'][0]
            if module == 0x06:
                dungeon_entry[side] = True
            elif module != 0x07:
                dungeon_entry[side] = False
        epoch = len(epochs) - 1
        so, no = epochs[-1]['source_frame'], epochs[-1]['native_frame']
        totals['iterations'] += 1
        for key in ('callbacks', 'held', 'masked'):
            totals[key] += row[key]
        totals['source_writes'] += len(row['source_writes'])
        totals['native_pushes'] += len(row['native_pushes'])
        commands, transport = source_commands(row['source_writes'])
        entry_requests = requests.copy()
        # Only the opening NMI's initial 1/2/3 group (optional music first)
        # is individually located by this ledger's format. Later lag-handler
        # groups and main-thread writes retain the full interval bound.
        ports = [p for p, _ in row['source_writes'][:4]]
        opening_end = 4 if ports == [0, 1, 2, 3] else 3 if ports[:3] == [1, 2, 3] else 0
        for lo, hi in transport:
            transfers.append(dict(iteration=row['iteration'], source_context=row['source_context'],
                                  first_write=lo, end_write=hi, writes=hi-lo,
                                  sha256=hashlib.sha256(json.dumps(row['source_writes'][lo:hi]).encode()).hexdigest()))
            totals['transport_writes'] += hi - lo
        for order, port, value in commands:
            if value:
                x = event(row, 'source', port, value, sframe,
                          sframe if order < opening_end else upper, epoch, so, order)
                source_raw.append(x)
                if value != previous['source'][port]:
                    source.append(x)
                    if port == 0 and value < 0xf0:
                        requests['source'] = x
                        x['dungeon_entry_track'] = dungeon_entry['source']
                        dungeon_entry['source'] = False
                else:
                    totals['source_nonzero_repeats'] += 1
            else:
                totals['source_zero_clears'] += 1
            previous['source'][port] = value
        for at, ports in row['native_pushes']:
            for port, value in enumerate(ports):
                if value:
                    y = event(row, 'native', port, value, at, at, epoch, no, port)
                    native_raw.append(y)
                    if value != previous['native'][port]:
                        native.append(y)
                        if port == 0 and value < 0xf0:
                            requests['native'] = y
                            y['dungeon_entry_track'] = dungeon_entry['native']
                            dungeon_entry['native'] = False
                    else:
                        totals['native_nonzero_repeats'] += 1
                else:
                    totals['native_zero_clears'] += 1
                previous['native'][port] = value
        # The only gameplay song-end callers are pendant/crystal receipts
        # (ancilla/items.rs), waiting on fanfare $13. The ledger also contains
        # uploader/ACK reads with no PC, so do not call all port-0 zeroes song
        # endings. Retain those other reads in the trace and count them here.
        source_polls = [v for p, v in row['source_reads'] if p == 0]
        receipt_reads = (not transport and row['source_context'][3] == 0x13
                         and all(p == 0 and v in (0, 0x13) for p, v in row['source_reads']))
        if not receipt_reads:
            totals['other_source_port0_reads'] += len(source_polls)
        for side, values, at, hi, start in (
                ('source', source_polls if receipt_reads else [],
                 sframe, upper, so),
                ('native', row['native_polls'], row['native_start'], row['native_start'], no)):
            for value in values:
                totals[side + '_polls'] += 1
                if value:
                    poll_active[side] = True
                elif poll_active[side]:
                    x = event(row, side, 0, 0, at, hi, epoch, start, 0)
                    x['kind'] = 'song_end'
                    x['request'] = entry_requests[side]
                    polls[side].append(x)
                    poll_active[side] = False
        totals['source_elapsed_frames'] = end - origin
        totals['native_elapsed_frames'] = row['native_start'] + row['callbacks']
    matched = [comparison(x, y) for x, y in align(source, native)]
    last = None
    for r in matched:
        if r['status'] != 'matched':
            continue
        if last is not None and last['source']['epoch'] == r['source']['epoch']:
            r['gap_offset_frames'] = r['offset_frames'] - last['offset_frames']
            r['gap_offset_ms'] = r['gap_offset_frames'] * FRAME_MS
            r['gap_bounds_frames'] = [r['offset_bounds_frames'][0] - last['offset_bounds_frames'][1],
                                      r['offset_bounds_frames'][1] - last['offset_bounds_frames'][0]]
        last = r
    raw = [comparison(x, y) for x, y in align(source_raw, native_raw)]
    # End episodes pair by order, epoch and a bounded iteration gap; zero is
    # deliberately not a song command. Every candidate/unmatched end is kept.
    ends = [comparison(x, y) for x, y in align(polls['source'], polls['native'])]
    def identity(e):
        return tuple(e[k] for k in ('epoch', 'iteration', 'frame', 'order', 'port', 'value'))
    music_pairs = {(identity(r['source']), identity(r['native'])) for r in matched
                   if r['status'] == 'matched' and r['source']['kind'] == 'song_change'}
    for r in ends:
        if r['status'] != 'matched':
            continue
        sr, nr = r['source'].get('request'), r['native'].get('request')
        r['request_pair_matched'] = bool(sr and nr and (identity(sr), identity(nr)) in music_pairs)
        if not r['request_pair_matched']:
            for key in ('source_request_to_end_frames', 'native_request_to_end_frames',
                        'request_relative_offset_frames', 'request_relative_offset_ms',
                        'request_relative_bounds_frames'):
                r.pop(key, None)
    changes = [r for r in matched if (r['source'] or r['native'])['kind'] == 'song_change']
    controls = [r for r in matched if (r['source'] or r['native'])['kind'] == 'music_control']
    contexts = collections.defaultdict(list)
    songs = collections.defaultdict(list)
    kinds = collections.defaultdict(list)
    transitions = collections.defaultdict(list)
    for r in matched:
        e = r['source'] or r['native']
        m, s, _, song = e['context']
        contexts[f'{m:02x}/{s:02x}'].append(r)
        songs[f'{song:02x}'].append(r)
        kinds[e['kind']].append(r)
        for group in transition_groups(e):
            transitions[group].append(r)
    summary = dict(schema=1, trace_sha256=digest(path), totals=dict(totals), epochs=epochs,
                   commands=summarize(matched), raw_nonzero_writes=summarize(raw),
                   song_changes=summarize(changes), music_controls=summarize(controls),
                   song_ends=summarize(ends), by_context={k: summarize(v) for k, v in sorted(contexts.items())},
                   by_song={k: summarize(v) for k, v in sorted(songs.items())},
                   by_kind={k: summarize(v) for k, v in sorted(kinds.items())},
                   transition_groups={k: summarize(v) for k, v in sorted(transitions.items())},
                   method=dict(frame_ms=FRAME_MS, source_point='iteration opening host',
                               source_bounds='opening host through next opening host (conservative inclusive)',
                               native_point='queue publication callback; before that callback audio render',
                               epoch='actual foreign-state reseed; cumulative clock never reset',
                               matching=f'monotone content/order, lookahead {LOOKAHEAD}, iteration gap <= {MAX_ITERATION_GAP}',
                               normalization='nonzero transitions per port; clears and equal repeats in raw trace',
                               limitations='command publication, not PCM onset or exact bus time; seeded source RNG census, not cold quest proof'))
    output.mkdir(parents=True, exist_ok=True)
    write_jsonl(output / 'commands.jsonl.gz', matched)
    write_jsonl(output / 'raw-nonzero-writes.jsonl.gz', raw)
    write_jsonl(output / 'unmatched.jsonl.gz', (r for r in matched if r['status'] != 'matched'))
    write_jsonl(output / 'song-changes.jsonl.gz', changes)
    write_jsonl(output / 'music-controls.jsonl.gz', controls)
    write_jsonl(output / 'song-ends.jsonl.gz', ends)
    write_jsonl(output / 'transitions.jsonl.gz',
                (dict(r, groups=transition_groups(r['source'] or r['native'])) for r in matched
                 if transition_groups(r['source'] or r['native'])))
    write_jsonl(output / 'bank-transfers.jsonl.gz', transfers)
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    return summary


def markdown_report(summaries):
    lines = ['# Lagless audio command timing', '',
             'Offsets are native minus source, on a nominal 60 Hz presentation grid. '
             'Negative is early. Source points use the opening NMI host; per-event '
             'bounds retain source timing uncertainty. These are seeded census measurements, '
             'not PCM onset or a continuous native quest proof.', '',
             '| Lane / events | Matched | Exact | 1 frame | 2–5 | >5 | Source only | Native only |',
             '| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for lane, s in summaries.items():
        for label, stats in [('commands', s['commands']), ('song changes', s['song_changes']),
                             ('music controls', s['music_controls']), ('song ends', s['song_ends']),
                             *s['transition_groups'].items()]:
            lines.append(f"| {lane} / {label} | " + ' | '.join(str(stats[k]) for k in
                         ('matched', 'exact', 'one_frame', 'two_to_five', 'over_five', 'source_only', 'native_only')) + ' |')
    lines += ['', 'Machine-readable summaries also contain every module/submodule and song, '
              'epoch and logical-iteration distributions, signed early/late counts, source bounds '
              'and request-to-end duration offsets. All unpaired events remain in unmatched.jsonl.gz. '
              'Source bank-upload payloads and zero clears remain in the compressed observation trace.']
    return '\n'.join(lines) + '\n'


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, default=ROOT / 'target/parity/zelda3')
    p.add_argument('--ledger', type=Path)
    p.add_argument('--sram', type=Path)
    p.add_argument('--output', type=Path, default=ROOT / 'target/lagless-audio-timing')
    p.add_argument('--frames', type=int, help='optional prefix; omit for the full census')
    p.add_argument('--analyze-only', action='store_true')
    args = p.parse_args()
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / 'target'):
        p.error('output must be under this checkout target/')
    if not args.analyze_only and (args.ledger is None or args.sram is None):
        p.error('--ledger and --sram are required to run censuses')
    if args.frames is not None and args.frames < 1:
        p.error('--frames must be positive')
    output.mkdir(parents=True, exist_ok=True)
    provenance = dict(binary_sha256=digest(args.binary), frames=args.frames)
    if args.ledger:
        provenance['ledger_sha256'] = digest(args.ledger / 'iterations.jsonl')
    if args.sram:
        provenance['sram_sha256'] = digest(args.sram)
    provenance['source_sha256'] = {str(f.relative_to(ROOT)): digest(f) for f in
        (Path(__file__).resolve(), ROOT / 'zelda3-bin/src/logical_replay.rs')}
    expected = None
    if args.ledger:
        with (args.ledger / 'iterations.jsonl').open('rb') as f:
            expected = sum(1 for _ in f)
        if args.frames is not None:
            expected = min(expected, args.frames)
    summaries = {}
    for lane in ('rom', 'live'):
        trace = output / f'{lane}.jsonl.zst'
        if not args.analyze_only:
            if trace.exists():
                p.error(f'output exists: {trace}; use --analyze-only or a fresh output directory')
            env = {k: v for k, v in os.environ.items() if not k.startswith('ZELDA3_')}
            with tempfile.TemporaryDirectory(prefix='zelda3-audio-timing-') as save:
                env['ZELDA3_SAVE_DIR'] = save
                cmd = [str(args.binary.resolve()), '--logical-replay', str(args.ledger.resolve()),
                       str(args.sram.resolve()), '--seed-every', '2000', '--report', '0',
                       '--audio-timing-trace', str(trace)]
                if lane == 'live':
                    cmd.append('--live-apu-reads')
                if args.frames is not None:
                    cmd += ['--frames', str(args.frames)]
                with (output / f'{lane}.log').open('w') as log:
                    subprocess.run(cmd, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
        log_path = output / f'{lane}.log'
        if log_path.exists() and expected is not None:
            with (output / f'{lane}-census-summary.txt').open('w') as gate:
                subprocess.run([sys.executable, str(ROOT / 'scripts/logical_census_summary.py'),
                                str(log_path), '--gate', '--expect-iterations', str(expected)],
                               stdout=gate, stderr=subprocess.STDOUT, check=True)
        summary = analyze(trace, output / lane)
        if expected is not None and summary['totals']['iterations'] != expected:
            raise ValueError(f'{lane}: expected {expected} iterations')
        summaries[lane] = summary
        provenance[lane] = dict(trace_sha256=summary['trace_sha256'],
                                log_sha256=digest(log_path) if log_path.exists() else None,
                                iterations=summary['totals']['iterations'])
        print(lane, json.dumps({k: v for k, v in summary['commands'].items() if k != 'worst_five'}), flush=True)
    (output / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    (output / 'report.md').write_text(markdown_report(summaries))


if __name__ == '__main__':
    main()
