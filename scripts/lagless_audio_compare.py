#!/usr/bin/env python3
"""Renderless PCM comparison. Extract pinned receipts, then compare mapped replay PCM.

Requires numpy and the zstd CLI. All sample coordinates count stereo pairs at
32 kHz. Positive alignment offset means the native signal occurs later than its
source-mapped position. Silence and weak/ambiguous matches are not onset proof.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

import numpy as np

RATE = 32000
FRAME = 534


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024*1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def extract(cache, output):
    """Decode only presented_audio; validate every host against canonical hashes."""
    if output.resolve().is_relative_to(cache.resolve()):
        raise ValueError('oracle cache is read-only')
    output.mkdir(parents=True, exist_ok=True)
    pattern = re.compile(rb'"presented_audio"\s*:\s*')
    decoder = json.JSONDecoder()
    count = pairs = 0
    with (output / 'oracle.pcm').open('xb') as pcm, (output / 'oracle.offsets').open('xb') as index:
        with subprocess.Popen(['zstd', '-dc', str(cache / 'original-timing-host-receipts.jsonl.zst')],
                              stdout=subprocess.PIPE) as process, (cache / 'oracle-av-hashes.jsonl').open() as hashes:
            index.write((0).to_bytes(8, 'little'))
            for line, canonical in zip(process.stdout, hashes, strict=True):
                # Decode the entire audio object but skip unrelated visual receipts.
                match = pattern.search(line)
                if match is None:
                    raise ValueError(f'no presented_audio at host {count}')
                audio, _ = decoder.raw_decode(line[match.end():].decode())
                values = audio['interleaved_stereo']
                if len(values) % 2:
                    raise ValueError('partial stereo pair')
                data = np.asarray(values, dtype='<i2').tobytes()
                expected = json.loads(canonical)
                if (expected['frame'] != count or expected['audio']['channels'] != 2
                        or expected['audio']['sample_frames'] != len(values) // 2
                        or expected['audio']['sha256'] != hashlib.sha256(data).hexdigest()):
                    raise ValueError(f'oracle hash/shape mismatch at host {count}')
                pcm.write(data)
                pairs += len(values) // 2
                index.write(pairs.to_bytes(8, 'little'))
                count += 1
                if count % 100000 == 0:
                    print(f'extracted {count} hosts', file=sys.stderr, flush=True)
            if process.wait() != 0:
                raise ValueError('zstd extraction failed')
    inputs = {name: sha256(cache/name) for name in
        ('cache-manifest.json', 'oracle-av-hashes.jsonl', 'original-timing-host-receipts.jsonl.zst')}
    manifest = json.loads((cache/'cache-manifest.json').read_text())
    for name in ('oracle-av-hashes.jsonl', 'original-timing-host-receipts.jsonl.zst'):
        if inputs[name] != manifest['artifact_sha256'][name]:
            raise ValueError(f'cache manifest hash mismatch: {name}')
    (output / 'oracle.json').write_text(json.dumps(dict(hosts=count, sample_pairs=pairs,
        rate=RATE, cache=str(cache.resolve()), hashes_verified=True,
        input_sha256=inputs), indent=2) + '\n')


def align(reference, signal, radius):
    """Best normalized stereo correlation across every integer sample offset.

    reference contains radius padding on both sides. Return the best offset,
    correlation, and full-window equality; do not infer causality from residuals.
    FFT computes all dot products, rolling sums normalize candidate energy.
    """
    n = len(signal)
    if n == 0 or len(reference) != n + 2 * radius:
        raise ValueError('alignment shape')
    x = reference.astype(np.float64)
    y = signal.astype(np.float64)
    y -= y.mean(axis=0)
    energy_y = np.sum(y * y)
    if energy_y == 0:
        return dict(offset_samples=None, correlation=None, identical=False, status='silent')
    size = 1 << (len(x) + n - 2).bit_length()
    dots = np.zeros(2 * radius + 1)
    energy_x = np.zeros_like(dots)
    for channel in range(2):
        conv = np.fft.irfft(np.fft.rfft(x[:, channel], size) *
                            np.fft.rfft(y[::-1, channel], size), size)
        dots += conv[n - 1:n + 2 * radius]
        sums = np.concatenate(([0.0], np.cumsum(x[:, channel])))
        squares = np.concatenate(([0.0], np.cumsum(x[:, channel] ** 2)))
        energy_x += squares[n:] - squares[:-n] - (sums[n:] - sums[:-n]) ** 2 / n
    valid = energy_x > max(1e-8, energy_y * 1e-12)
    score = np.full_like(dots, -1.0)
    score[valid] = np.clip(dots[valid] / np.sqrt(energy_x[valid] * energy_y), -1, 1)
    best = int(np.argmax(score))
    # Prefer the closest zero offset on an equal-correlation plateau.
    tied = np.flatnonzero(score >= score[best] - 1e-10)
    best = int(tied[np.argmin(abs(tied - radius))])
    offset = radius - best
    identical = bool(np.array_equal(reference[best:best+n], signal))
    error = reference[best:best+n].astype(np.float64) - signal
    independent = np.abs(np.arange(len(score)) - best) > FRAME
    competing = float(np.max(score[independent])) if np.any(independent) else -1.0
    return dict(offset_samples=offset, offset_ms=offset * 1000 / RATE,
                correlation=float(score[best]), identical=identical,
                rmse=float(np.sqrt(np.mean(error**2))), max_sample_error=float(np.max(abs(error))),
                status='matched' if score[best] >= 0.95 else 'weak',
                boundary=best in (0, 2 * radius),
                competing_peak_correlation=competing,
                ambiguous=bool(len(tied) > 1 or competing >= score[best] - 0.001))


def compare(oracle, pcm_path, map_path, output, stride=60, radius=3200, window=2048,
            expect_iterations=None):
    reference = np.memmap(oracle / 'oracle.pcm', dtype='<i2', mode='r').reshape(-1, 2)
    offsets = np.memmap(oracle / 'oracle.offsets', dtype='<u8', mode='r')
    signal = np.memmap(pcm_path, dtype='<i2', mode='r').reshape(-1, 2)
    counts = Counter()
    aligned = Counter()
    matched_offsets = []
    stream_offsets = []
    stream_bins = Counter()
    worst = []
    worst_timing = []
    worst_residual = []
    first = None
    first_sample = None
    first_residual = None
    drifts = []
    songs = []
    last_song = None
    song_drift_origin = 0
    sample_end = 0
    first_source = None
    first_source_host = None
    previous_iteration = None
    drift_min = drift_max = 0
    with map_path.open() as records, Path(str(output) + '.windows.jsonl').open('w') as windows:
        for line in records:
            row = json.loads(line)
            i = row['iteration']
            a, b = row['sample_start'], row['sample_end']
            if a < 0 or b > len(signal) or b-a != row['callbacks']*FRAME:
                raise ValueError('invalid callback/sample range')
            if previous_iteration is None and expect_iterations is not None and i != 0:
                raise ValueError('full route must start at iteration zero')
            if a != sample_end or (previous_iteration is not None and i != previous_iteration + 1):
                raise ValueError('non-contiguous replay mapping (panic or skipped window)')
            sample_end = b
            previous_iteration = i
            start, end = row['source_start'], row['source_end']
            if end is None:
                # The ledger has no next opening NMI at EOF. Report the entire
                # remaining captured suffix, explicitly counting this convention.
                end = len(offsets) - 1
                counts['terminal_spans_to_oracle_eof'] += 1
            if end < start or end >= len(offsets):
                raise ValueError('invalid source host range')
            ra, rb = int(offsets[start]), int(offsets[end])
            if first_source is None:
                first_source = ra
                first_source_host = start
            native = signal[a:b]
            expected = reference[ra:rb]
            equal = bool(np.array_equal(native, expected))
            overlap = min(len(native), len(expected))
            if first_sample is None and overlap:
                different = np.flatnonzero(np.any(native[:overlap] != expected[:overlap], axis=1))
                if len(different):
                    k = int(different[0])
                    first_sample = dict(iteration=i, source_host=start, pair_in_span=k,
                        native=native[k].tolist(), oracle=expected[k].tolist())
            counts['iterations'] += 1
            counts['source_exact_iterations'] += equal
            counts['source_exact_nonzero_iterations'] += equal and bool(np.any(native))
            counts['native_callbacks'] += row['callbacks']
            counts['source_hosts'] += end - start
            counts['reseeded'] += row['reseeded']
            counts['held'] += row['held']
            counts['source_lag_nmis'] += row['lag_nmis']
            counts['masked'] += row['masked']
            # Callback j is paired with source host open_host+j; excess callbacks
            # remain mismatches, never silently truncated from the denominator.
            for j in range(max(row['callbacks'], end - start)):
                counts['frame_comparisons'] += 1
                if j < row['callbacks'] and start+j < end:
                    h0, h1 = int(offsets[start+j]), int(offsets[start+j+1])
                    frame = signal[a+j*FRAME:a+(j+1)*FRAME]
                    exact = bool(np.array_equal(frame, reference[h0:h1]))
                    counts['source_exact_frames'] += exact
                    counts['source_exact_nonzero_frames'] += exact and bool(np.any(frame))
            drift = b - (rb - first_source)
            drift_before = a - (ra-first_source)
            if row['song'] != last_song or row['reseeded']:
                song_drift_origin = drift_before
            drift_min, drift_max = min(drift_min, drift), max(drift_max, drift)
            context = {k: row[k] for k in ('iteration', 'source_start', 'module', 'submodule', 'room', 'song', 'reseeded')}
            context.update(source_module=row.get('source_module'), source_submodule=row.get('source_submodule'))
            if first is None and not equal:
                overlap = min(len(native), len(expected))
                different = np.flatnonzero(np.any(native[:overlap] != expected[:overlap], axis=1))
                first = dict(context, native_pairs=b-a, source_pairs=rb-ra,
                             first_pair=int(different[0]) if len(different) else overlap)
            if i % 2000 == 0 or row['song'] != last_song or row['reseeded']:
                drifts.append(dict(context, drift_samples=drift, drift_ms=drift*1000/RATE))
            if row['song'] != last_song:
                songs.append(dict(context, drift_samples=drift, drift_ms=drift*1000/RATE))
            # Source-mapped windows, never pretend these are independently labelled
            # DSP onsets. Song transitions receive a window even off the stride.
            # Predict phase from independent sample budgets since the observed
            # music change. This recenters the finite search; it does not alter
            # either stream, and effects/loops may still defeat correspondence.
            centers = sorted(set((ra, ra + drift_before - song_drift_origin)))
            centers = [c for c in centers if c >= radius and c+window+radius <= len(reference)]
            if (i % stride == 0 or row['song'] != last_song) and centers and a+window <= len(signal):
                candidates = [(c, align(reference[c-radius:c+window+radius], signal[a:a+window], radius)) for c in centers]
                center, result = max(candidates, key=lambda pair: -2 if pair[1]['correlation'] is None else pair[1]['correlation'])
                for other_center, other in candidates:
                    if (other_center != center and other['correlation'] is not None
                            and result['correlation'] is not None
                            and other['correlation'] >= result['correlation'] - 0.001
                            and abs((other_center-other['offset_samples']) - (center-result['offset_samples'])) > FRAME):
                        result['ambiguous'] = True
                result['search_center_delta_samples'] = center-ra
                result.update(context, drift_samples=drift)
                if result['offset_samples'] is not None:
                    result['search_residual_samples'] = result['offset_samples']
                    result['offset_samples'] += ra-center
                    result['offset_ms'] = result['offset_samples']*1000/RATE
                    stream_offset = drift_before + result['offset_samples']
                    result.update(stream_offset_samples=stream_offset,
                                  stream_offset_ms=stream_offset*1000/RATE)
                windows.write(json.dumps(result) + '\n')
                aligned[result['status']] += 1
                aligned['windows'] += 1
                aligned['identical_any_best_shift'] += result['identical']
                if result['status'] == 'matched' and not result['boundary'] and not result['ambiguous']:
                    distance = abs(result['offset_samples'])
                    category = 'exact' if distance == 0 else 'within_1_frame' if distance <= FRAME else 'within_2_frames' if distance <= 2*FRAME else 'worse'
                    aligned[category] += 1
                    aligned['identical_after_alignment'] += result['identical']
                    matched_offsets.append(result['offset_samples'])
                    stream_offsets.append(stream_offset)
                    distance = abs(stream_offset)
                    stream_bins['exact' if distance == 0 else 'within_1_frame' if distance <= FRAME
                                else 'within_2_frames' if distance <= 2*FRAME else 'worse'] += 1
                    worst_timing.append(result)
                    worst_timing.sort(key=lambda r: abs(r['stream_offset_samples']), reverse=True)
                    del worst_timing[20:]
                    worst_residual.append(result)
                    worst_residual.sort(key=lambda r: r['rmse'], reverse=True)
                    del worst_residual[20:]
                    if first_residual is None and not result['identical']:
                        first_residual = result
                else:
                    aligned['unresolved'] += 1
                if result['correlation'] is not None:
                    worst.append(result)
                    worst.sort(key=lambda r: r['correlation'])
                    del worst[20:]
            last_song = row['song']
    if sample_end != len(signal):
        raise ValueError('PCM length differs from map')
    if expect_iterations is not None and counts['iterations'] != expect_iterations:
        raise ValueError(f"incomplete route: {counts['iterations']} != {expect_iterations} iterations")
    quantiles = dict(zip(('min', 'p05', 'median', 'p95', 'max'),
        np.percentile(matched_offsets, [0, 5, 50, 95, 100]).tolist())) if matched_offsets else {}
    stream_quantiles = dict(zip(('min', 'p05', 'median', 'p95', 'max'),
        np.percentile(stream_offsets, [0, 5, 50, 95, 100]).tolist())) if stream_offsets else {}
    summary = dict(method='source-host spans; stereo FFT normalized correlation; unresolved is not an onset',
        rate=RATE, frame_pairs=FRAME, window_pairs=window, radius_pairs=radius,
        stride_iterations=stride, counts=dict(counts), alignment=dict(aligned), first_difference=first,
        first_sample_difference=first_sample, drift_min_samples=drift_min, drift_max_samples=drift_max,
        first_correlated_residual=first_residual, final_drift_samples=drift,
        native_sample_pairs=sample_end, source_sample_pairs=rb-first_source,
        excluded_source_bootstrap_pairs=first_source, oracle_total_pairs=len(reference),
        excluded_source_bootstrap_hosts=first_source_host,
        full_capture_sample_difference=sample_end-len(reference),
        full_capture_difference_ms=(sample_end-len(reference))*1000/RATE,
        callback_count_component_samples=FRAME*(counts['native_callbacks']-counts['source_hosts']),
        per_host_budget_component_samples=FRAME*counts['source_hosts']-(rb-first_source),
        offset_quantiles_samples=quantiles,
        offset_quantiles_ms={key: value*1000/RATE for key, value in quantiles.items()},
        stream_offset_bins=dict(stream_bins), stream_offset_quantiles_samples=stream_quantiles,
        stream_offset_quantiles_ms={key: value*1000/RATE for key, value in stream_quantiles.items()},
        final_drift_ms=drift*1000/RATE, drift_checkpoints=drifts, song_changes=songs, worst_windows=worst,
        worst_timing_windows=worst_timing, worst_correlated_residual_windows=worst_residual,
        pcm=str(pcm_path.resolve()), mapping=str(map_path.resolve()))
    output.write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({k: v for k, v in summary.items() if k not in ('drift_checkpoints', 'song_changes', 'worst_windows', 'worst_timing_windows', 'worst_correlated_residual_windows')}, indent=2))


def modes(rom_pcm, rom_map, live_pcm, live_map, output):
    """Separate observed APU-policy effects from speculative emulator causes."""
    rom = np.memmap(rom_pcm, dtype='<i2', mode='r').reshape(-1, 2)
    live = np.memmap(live_pcm, dtype='<i2', mode='r').reshape(-1, 2)
    counts = Counter()
    first = first_poll = first_boolean_poll = None
    with rom_map.open() as a, live_map.open() as b:
        for x, y in zip(a, b, strict=True):
            x, y = json.loads(x), json.loads(y)
            if x['iteration'] != y['iteration']:
                raise ValueError('APU policy iteration coverage differs')
            equal = bool(np.array_equal(rom[x['sample_start']:x['sample_end']],
                                       live[y['sample_start']:y['sample_end']]))
            counts['iterations'] += 1
            counts['identical_pcm_iterations'] += equal
            counts['different_callback_counts'] += x['callbacks'] != y['callbacks']
            counts['different_poll_answers'] += x['polls'] != y['polls']
            boolean_poll_difference = [bool(v) for v in x['polls']] != [bool(v) for v in y['polls']]
            counts['different_boolean_poll_answers'] += boolean_poll_difference
            counts['rom_missing_holds'] += max(0, x['lag_nmis'] - x['held'])
            counts['live_missing_holds'] += max(0, y['lag_nmis'] - y['held'])
            if not equal and first is None:
                first = dict(rom=x, live=y)
            if x['polls'] != y['polls'] and first_poll is None:
                first_poll = dict(rom=x, live=y)
            if boolean_poll_difference and first_boolean_poll is None:
                first_boolean_poll = dict(rom=x, live=y)
    result = dict(counts=dict(counts), first_pcm_policy_difference=first,
                  first_poll_policy_difference=first_poll,
                  first_boolean_poll_policy_difference=first_boolean_poll,
                  residual_cause='unresolved: timing/commands/SPC history must be controlled before inferring an SPC/DSP bug')
    output.write_text(json.dumps(result, indent=2) + '\n')


def command_alignment(native, source):
    """Anchor unique nonzero port commands; repeated clears never identify an event.

    A shared command value identifies a candidate in these already route-matched
    short captures. It does not establish equal upload history or chip state.
    Compare the following raw stores and relative delivery cycles separately.
    """
    key = lambda row: (row['port'], row['value'])
    nc = Counter(key(row) for row in native if row['value'])
    sc = Counter(key(row) for row in source if row['value'])
    anchors = []
    for ni, row in enumerate(native):
        token = key(row)
        if not row['value'] or nc[token] != 1 or sc[token] != 1:
            continue
        si = next(i for i, item in enumerate(source) if key(item) == token)
        first_timing = first_store = None
        compared = 0
        for offset, (n, s) in enumerate(zip(native[ni:], source[si:])):
            if key(n) != key(s):
                first_store = dict(offset=offset, native=n, source=s)
                break
            compared += 1
            nd = n['cycle'] - row['cycle']
            sd = s['cycle'] - source[si]['cycle']
            if nd != sd and first_timing is None:
                first_timing = dict(offset=offset, native_delta=nd, source_delta=sd,
                                    difference_cycles=nd-sd)
        anchors.append(dict(port=row['port'], value=row['value'], native=row,
            source=source[si], matching_store_prefix=compared,
            first_relative_timing_difference=first_timing,
            first_store_difference=first_store,
            native_stores_remaining=len(native)-ni, source_stores_remaining=len(source)-si))
    return dict(status='command-anchored' if anchors else 'unresolved', anchors=anchors,
        native_stores=len(native), source_stores=len(source),
        limitation='short route-matched command identity; equal initial SPC/DSP state is not established')


def compare_commands(native_log, source_dir, output):
    pattern = re.compile(r'spc_port_write host=(\d+) cycle=(\d+) port=(\d+) value=(\d+) delivery=(\S+)')
    native = [dict(host=int(m[1]), cycle=int(m[2]), port=int(m[3]), value=int(m[4]),
                   delivery=m[5]) for m in pattern.finditer(native_log.read_text())]
    source = []
    pairs = 0
    previous = None
    for path in sorted(source_dir.glob('audio-*.json'), key=lambda p: int(p.stem.split('-')[1])):
        row = json.loads(path.read_text())
        if previous is not None and row['frame'] != previous + 1:
            raise ValueError('source command capture must contain contiguous hosts')
        previous = row['frame']
        for event in row['apu_port_writes']:
            if not event['is_read']:
                source.append(dict(host=row['frame'], cycle=pairs*32+event['apu_cycle_after'],
                    port=event['port'], value=event['value']))
        pairs += row['audio_digest']['sample_frames']
    if not native or previous is None:
        raise ValueError('both port traces are required')
    result = command_alignment(native, source)
    result.update(native_log=str(native_log.resolve()), source_dir=str(source_dir.resolve()))
    output.write_text(json.dumps(result, indent=2) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    p = commands.add_parser('extract')
    p.add_argument('cache', type=Path)
    p.add_argument('output', type=Path)
    p = commands.add_parser('compare')
    for name in ('oracle', 'pcm', 'mapping', 'output'):
        p.add_argument(name, type=Path)
    p.add_argument('--stride', type=int, default=60)
    p.add_argument('--radius', type=int, default=3200)
    p.add_argument('--window', type=int, default=2048)
    p.add_argument('--expect-iterations', type=int, help='fail closed on a truncated route')
    p = commands.add_parser('modes')
    for name in ('rom_pcm', 'rom_map', 'live_pcm', 'live_map', 'output'):
        p.add_argument(name, type=Path)
    p = commands.add_parser('commands')
    for name in ('native_log', 'source_dir', 'output'):
        p.add_argument(name, type=Path)
    args = parser.parse_args()
    if args.action == 'extract':
        extract(args.cache, args.output)
    elif args.action == 'modes':
        modes(args.rom_pcm, args.rom_map, args.live_pcm, args.live_map, args.output)
    elif args.action == 'commands':
        compare_commands(args.native_log, args.source_dir, args.output)
    else:
        if min(args.stride, args.radius, args.window) < 1:
            parser.error('stride, radius and window must be positive')
        compare(args.oracle, args.pcm, args.mapping, args.output, args.stride, args.radius, args.window,
                args.expect_iterations)


if __name__ == '__main__':
    main()
