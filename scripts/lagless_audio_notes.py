#!/usr/bin/env python3
"""Compare live DSP note writes from --render-audio-cpu captures.

This does not infer note onsets from mixed PCM. Raw KON/KOFF writes, including
repeated KOFF masks, are preserved. An alignment is a convention, not proof of
the identity of repeated notes. Unanchored notes remain unanchored.
"""
import argparse
import bisect
from collections import Counter, defaultdict
import difflib
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import wave


def sha256(path):
    digest = hashlib.sha256()
    with open(path, 'rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def verify_reference(capture, oracle):
    complete = json.loads((capture / 'complete.json').read_text())
    if not complete['reference_verified'] or complete['lag_free']:
        raise ValueError('capture is not a successfully verified original')
    count = samples = 0
    canonical = iter(rows(oracle))
    for row in rows(capture / 'hashes.jsonl.zst'):
        expected = next(canonical)
        if (row['host'] != count or expected['frame'] != count or
                row['sample_frames'] != expected['audio']['sample_frames'] or
                row['sha256'] != expected['audio']['sha256'] or
                expected['audio']['channels'] != 2 or not row['oracle_match']):
            raise ValueError(f'original hash proof failed at host {count}')
        samples += row['sample_frames']
        count += 1
    if count != complete['frames'] or samples != complete['sample_frames']:
        raise ValueError('original proof does not cover the completed capture')
    return dict(frames=count, sample_frames=samples, full_oracle_route=next(canonical, None) is None,
                oracle_sha256=sha256(oracle), capture_hashes_sha256=sha256(capture / 'hashes.jsonl.zst'))


def rows(path):
    if str(path).endswith('.zst'):
        with subprocess.Popen(['zstd', '-dc', str(path)], stdout=subprocess.PIPE,
                              stderr=subprocess.DEVNULL) as proc:
            for line in proc.stdout:
                yield json.loads(line)
            if proc.wait():
                raise ValueError(f'incomplete compressed capture: {path}')
    else:
        with (gzip.open(path, 'rt') if str(path).endswith('.gz') else open(path)) as stream:
            for line in stream:
                yield json.loads(line)


def signed(value):
    return value if value < 128 else value - 256


def extract_notes(frames, registers=None, observed=None, contours=True):
    """Read every DSP write in its original order, snapshot registers at KON/KOFF.

    A phase is 1/32 sample (one SPC clock). Release is the first subsequent
    KOFF write for that voice; retrigger and capture-end are separate endings.
    Writes after a KON are retained as the active note's register contour.
    """
    regs = registers if registers is not None else [0] * 128
    seen = observed if observed is not None else [True] * 128
    active = {}
    notes, edges = [], []
    for frame in frames:
        if frame.get('reset'):
            for old in active.values():
                old['end'] = frame['sample']
                old['end_kind'] = 'reset'
            active.clear()
            regs[:] = [0] * 128
            seen[:] = [False] * 128
        for sample, phase, address, value in frame['writes']:
            if address >= 128:
                continue
            regs[address] = value
            seen[address] = True
            at = sample + phase / 32
            if address not in (0x4c, 0x5c):
                voice, register = divmod(address, 16)
                if contours and register <= 7 and voice in active:
                    note = active[voice]
                    note['contour'].append([round(at - note['time'], 5), register, value])
                continue
            for voice in range(8):
                if not value & (1 << voice):
                    continue
                base = voice * 16
                edge = dict(kind='on' if address == 0x4c else 'off', voice=voice,
                            source=regs[base + 4], pitch=regs[base + 2] | (regs[base + 3] & 63) << 8,
                            volume=[signed(regs[base]), signed(regs[base + 1])],
                            adsr=regs[base + 5:base + 7], gain=regs[base + 7],
                            time=at, host=frame['host'], iteration=frame['iteration'],
                            context=frame['context'], music=frame['music'])
                edge['observed_fields'] = [name for name, offsets in
                    (('source', [4]), ('pitch', [2, 3]), ('volume', [0, 1]), ('adsr', [5, 6]), ('gain', [7]))
                    if all(seen[base + offset] for offset in offsets)]
                edges.append(edge)
                old = active.pop(voice, None)
                if old is not None:
                    old['end'] = at
                    old['end_kind'] = 'off' if address == 0x5c else 'retrigger'
                if address == 0x4c:
                    note = dict(edge, contour=[], end=None, end_kind='capture_end')
                    notes.append(note)
                    active[voice] = note
    return notes, edges


def token(note):
    return note['voice'], note['source'], note['pitch']


def verify_emitted_edges(path, edges):
    """Independently check the Rust snapshots against the complete raw DSP stream."""
    count = 0
    for emitted, rebuilt in zip(rows(path), edges, strict=True):
        for field in ('host', 'iteration', 'kind', 'voice', 'source', 'pitch', 'volume', 'adsr', 'gain'):
            if emitted[field] != rebuilt[field]:
                raise ValueError(f'note snapshot mismatch at edge {count}: {field}')
        if emitted['sample'] + emitted['phase'] / 32 != rebuilt['time']:
            raise ValueError(f'note timestamp mismatch at edge {count}')
        count += 1
    return count


def verify_live_edges(capture):
    """Streaming capture validation without retaining a route of register contours."""
    registers = [0] * 128
    emitted = iter(rows(capture / 'notes.jsonl.zst'))
    counts = Counter()
    writes = 0
    last_host = -1
    last_dsp_host = -1
    contexts = Counter()
    context_bounds = {}
    songs = Counter()
    for frame in rows(capture / 'dsp.jsonl.zst'):
        if frame['host'] <= last_host:
            raise ValueError('nonmonotone DSP host stream')
        last_host = frame['host']
        if frame.get('reset'):
            registers = [0] * 128
        if frame['writes']:
            last_dsp_host = last_host
            context = tuple(frame['context'])
            contexts[context] += 1
            context_bounds.setdefault(context, [last_host, last_host])[1] = last_host
            songs[frame['music'].get('last_music_control', 0)] += 1
        for sample, phase, address, value in frame['writes']:
            writes += 1
            if address >= 128:
                continue
            registers[address] = value
            if address not in (0x4c, 0x5c):
                continue
            for voice in range(8):
                if not value & (1 << voice):
                    continue
                base = voice * 16
                expected = dict(host=frame['host'], iteration=frame['iteration'], sample=sample, phase=phase,
                                kind='on' if address == 0x4c else 'off', voice=voice,
                                source=registers[base+4], pitch=registers[base+2] | (registers[base+3] & 63) << 8,
                                volume=[signed(registers[base]), signed(registers[base+1])],
                                adsr=registers[base+5:base+7], gain=registers[base+7])
                actual = next(emitted, None)
                if actual != expected:
                    raise ValueError(f'live edge mismatch at host {last_host}, edge {sum(counts.values())}')
                counts[expected['kind']] += 1
    if next(emitted, None) is not None:
        raise ValueError('extra emitted edges after DSP EOF')
    return dict(dsp_writes=writes, edges=dict(counts), last_dsp_host=last_dsp_host,
                dsp_active_contexts=[dict(context=list(context), frames=count,
                                         first_host=context_bounds[context][0], last_host=context_bounds[context][1])
                                     for context, count in contexts.most_common()],
                dsp_active_song_latches=dict(songs))


def align(left, right):
    """Ordered exact anchors, then edit alignment inside unmatched spans.

    Exact voice/source/pitch anchors prevent a single omitted note from
    shifting all subsequent identities. Substitutions cost less than a
    deletion plus insertion; large ambiguous spans stay unpaired.
    """
    matcher = difflib.SequenceMatcher(None, [token(n) for n in left],
                                     [token(n) for n in right], autojunk=False)
    result = []
    for op, a, b, c, d in matcher.get_opcodes():
        if op == 'equal':
            result.extend((i, j, 'exact') for i, j in zip(range(a, b), range(c, d)))
        elif op == 'replace' and (b - a) * (d - c) <= 65536:
            n, m = b - a, d - c
            score = [[0] * (m + 1) for _ in range(n + 1)]
            for i in range(n + 1): score[i][0] = i * 2
            for j in range(m + 1): score[0][j] = j * 2
            for i in range(1, n + 1):
                for j in range(1, m + 1):
                    cost = 0 if token(left[a+i-1]) == token(right[c+j-1]) else 3
                    score[i][j] = min(score[i-1][j] + 2, score[i][j-1] + 2,
                                      score[i-1][j-1] + cost)
            i, j = n, m
            tail = []
            while i or j:
                cost = 0 if i and j and token(left[a+i-1]) == token(right[c+j-1]) else 3
                if i and j and score[i][j] == score[i-1][j-1] + cost:
                    tail.append((a+i-1, c+j-1, 'exact' if cost == 0 else 'substitution'))
                    i -= 1; j -= 1
                elif i and score[i][j] == score[i-1][j] + 2:
                    tail.append((a+i-1, None, 'missing')); i -= 1
                else:
                    tail.append((None, c+j-1, 'extra')); j -= 1
            result.extend(reversed(tail))
        else:
            result.extend((i, None, 'missing') for i in range(a, b))
            result.extend((None, j, 'extra') for j in range(c, d))
    return result


def compare(left, right, anchors):
    """Partition by matched command anchors, preserving voice and monotonic order.

    Anchors contain original/lagless sample times for matched song commands.
    Absolute shift and shift relative to the first matched note in a song
    are separate fields. The same song origin is used across all eight voices.
    Without voice ownership, these timing fields are measurements, not audible
    music/SFX classifications. No threshold is applied to absolute route lead.
    """
    boundaries = [[x[side] for x in anchors] for side in ('original', 'lagless')]
    groups = [defaultdict(list), defaultdict(list)]
    for side, notes in enumerate((left, right)):
        for note in notes:
            segment = bisect.bisect_right(boundaries[side], note['time'])
            groups[side][segment, note['voice']].append(note)
    output = []
    for key in sorted(groups[0].keys() | groups[1].keys()):
        a, b = groups[0][key], groups[1][key]
        paired = align(a, b)
        for i, j, status in paired:
            x, y = a[i] if i is not None else None, b[j] if j is not None else None
            row = dict(status=status, segment=key[0], voice=key[1], original=x, lagless=y)
            if x and y:
                shift = (y['time'] - x['time']) / 32
                row.update(onset_shift_ms=shift,
                           mismatches=[field for field in ('source', 'pitch', 'volume', 'adsr', 'gain')
                                       if field in x.get('observed_fields', ('source', 'pitch', 'volume', 'adsr', 'gain'))
                                       and field in y.get('observed_fields', ('source', 'pitch', 'volume', 'adsr', 'gain'))
                                       and x[field] != y[field]],
                           release_shift_ms=((y['end'] - y['time']) - (x['end'] - x['time'])) / 32
                               if x['end'] is not None and y['end'] is not None else None)
            output.append(row)
    origins = {}
    for row in sorted((r for r in output if r['status'] == 'exact'),
                      key=lambda r: r['original']['time']):
        origins.setdefault(row['segment'], row['onset_shift_ms'])
    for row in output:
        if 'onset_shift_ms' not in row:
            continue
        segment = row['segment']
        origin = origins.get(segment)
        row['relative_first_note_ms'] = row['onset_shift_ms'] - origin if origin is not None else None
        row['uniform_shift_ms'] = origin
        if segment:
            anchor = anchors[segment - 1]
            row['command_relative_onset_shift_ms'] = row['onset_shift_ms'] - (anchor['lagless'] - anchor['original']) / 32
    return output


def compare_segments(original, lagless, boundaries, output, require_full=True):
    """Bounded-memory comparison with explicit matched-command/reset boundaries.

    Each boundary supplies original_host/lagless_host and corresponding sample
    coordinates. Register values carry across cuts; active notes are censored
    at cuts, never fabricated as fresh KONs. A reset segment remains a harness
    observation until a later matched song request establishes a common origin.
    Category ownership is deliberately not guessed from a fixed voice number.
    """
    if len(boundaries) < 2:
        raise ValueError('need a start and an end boundary')
    for side in ('original', 'lagless'):
        if boundaries[0][side + '_host'] != 0:
            raise ValueError('segmented capture must start at host zero')
        if any(a[side + '_host'] >= b[side + '_host'] for a, b in zip(boundaries, boundaries[1:])):
            raise ValueError('nonmonotone segment boundaries')
    streams = [iter(rows(path / 'dsp.jsonl.zst')) for path in (original, lagless)]
    pending = [next(stream, None) for stream in streams]
    registers = [[0] * 128 for _ in streams]
    observed = [[True] * 128 for _ in streams]
    totals, continuous, restarted, resets, groups = Counter(), Counter(), Counter(), 0, []
    output.mkdir(parents=True, exist_ok=False)
    seeded = False
    with gzip.open(output / 'alignment.jsonl.gz', 'wt') as target:
        for index, (start, end) in enumerate(zip(boundaries, boundaries[1:])):
            if start.get('kind') == 'reset':
                seeded = True
                resets += 1
            elif start.get('kind') == 'song':
                seeded = False
            notes = []
            for side, name in enumerate(('original', 'lagless')):
                def frames():
                    while pending[side] is not None and pending[side]['host'] < end[name + '_host']:
                        item = pending[side]
                        pending[side] = next(streams[side], None)
                        yield item
                on, _ = extract_notes(frames(), registers[side], observed[side], contours=False)
                notes.append(on)
            anchor = dict(original=start['original_sample'], lagless=start['lagless_sample'])
            counts = Counter()
            comparable = []
            for name, on in zip(('original', 'lagless'), notes):
                comparable.append([note for note in on if {'source', 'pitch'} <= set(note['observed_fields'])])
                counts['unobservable_identity_' + name] = len(on) - len(comparable[-1])
            result = compare(*comparable, [anchor])
            for row in result:
                row['route_segment'] = index
                row['boundary'] = start
                row['seed_restart_contamination'] = seeded
                row['category'] = 'unattributed'
                counts[row['status']] += 1
                for field in row.get('mismatches', []):
                    counts['mismatch_' + field] += 1
                for field in ('onset_shift_ms', 'command_relative_onset_shift_ms', 'relative_first_note_ms'):
                    if row.get(field) is not None:
                        for threshold in (20, 45):
                            counts[f'{field}_over_{threshold}'] += abs(row[field]) > threshold
                for name in ('original', 'lagless'):
                    if row[name] is not None:
                        # Raw register streams remain the contour authority;
                        # avoid duplicating hundreds of millions of writes.
                        row[name] = {k: v for k, v in row[name].items() if k != 'contour'}
                target.write(json.dumps(row, separators=(',', ':')) + '\n')
            groups.append(dict(index=index, start=start, end=end, seeded=seeded,
                               notes=list(map(len, notes)), counts=dict(counts)))
            totals.update(counts)
            (restarted if seeded else continuous).update(counts)
    full = not any(item is not None for item in pending)
    if require_full and not full:
        raise ValueError('segment endpoints do not cover the captures')
    summary = dict(counts=dict(totals), resets=resets, segments=groups, full_capture_coverage=full,
                   continuous_counts=dict(continuous), seed_restart_counts=dict(restarted),
                   limitation='KON snapshot comparison, release censored at segment cuts; '
                   'voice ownership and audibility are not inferred. Reset-contaminated '
                   'segments must be reported separately from continuous-play findings.')
    (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    return summary


def read_pcm_window(capture, start, length):
    """Read a sample interval from sparse PCM; refuse padding or capture gaps."""
    metadata = list(rows(capture / 'frames.jsonl'))
    if start < 0 or length <= 0:
        raise ValueError('invalid listening interval')
    end, cursor, chunks = start + length, start, []
    with (capture / 'windows.s16le').open('rb') as pcm:
        for frame in metadata:
            lo, hi = frame['sample'], frame['sample'] + frame['sample_frames']
            if hi <= cursor:
                continue
            if lo > cursor:
                raise ValueError('PCM window crosses an uncaptured gap')
            pcm.seek(frame['byte_offset'])
            data = pcm.read(frame['sample_frames'] * 4)
            if len(data) != frame['sample_frames'] * 4 or hashlib.sha256(data).hexdigest() != frame['sha256']:
                raise ValueError('selected PCM callback hash mismatch')
            stop = min(hi, end)
            chunks.append(data[(cursor - lo) * 4:(stop - lo) * 4])
            cursor = stop
            if cursor == end:
                return b''.join(chunks)
    raise ValueError('PCM window extends past the capture')


def render_listen_windows(original, lagless, plan, output):
    """Render exactly three seconds per side with the plan's common alignment.

    The caller supplies the category and selection rationale; this function
    never ranks mixed-waveform error or calls a sample difference audible.
    """
    output.mkdir(parents=True, exist_ok=False)
    index = []
    for entry in plan:
        row = dict(entry)
        for name, capture in (('original', original), ('lagless', lagless)):
            data = read_pcm_window(capture, entry[name + '_sample_start'], 96000)
            path = output / f"{entry['id']}-{name}.wav"
            with wave.open(str(path), 'wb') as wav:
                wav.setnchannels(2)
                wav.setsampwidth(2)
                wav.setframerate(32000)
                wav.writeframes(data)
            row[name + '_wav'] = path.name
            row[name + '_sha256'] = sha256(path)
        index.append(row)
    (output / 'index.json').write_text(json.dumps(index, indent=2) + '\n')
    return index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('original', type=Path)
    parser.add_argument('lagless', type=Path)
    parser.add_argument('output', type=Path)
    alignment = parser.add_mutually_exclusive_group(required=True)
    alignment.add_argument('--anchors', type=Path,
                        help='JSON list of matched song commands with original/lagless sample times')
    alignment.add_argument('--segments', type=Path,
                           help='JSON matched song/reset host and sample boundaries; bounded memory')
    parser.add_argument('--oracle', type=Path, required=True, help='read-only oracle-av-hashes.jsonl')
    args = parser.parse_args()
    complete = [json.loads((path / 'complete.json').read_text()) for path in (args.original, args.lagless)]
    proof = verify_reference(args.original, args.oracle)
    if args.segments:
        checks = {name: verify_live_edges(path) for name, path in
                  (('original', args.original), ('lagless', args.lagless))}
        result = compare_segments(args.original, args.lagless,
                                  json.loads(args.segments.read_text()), args.output)
        result.update(reference_proof=proof, live_note_verification=checks,
                      boundaries_sha256=sha256(args.segments), captures=complete)
        (args.output / 'summary.json').write_text(json.dumps(result, indent=2) + '\n')
        return
    args.output.mkdir(parents=True, exist_ok=False)
    streams = [extract_notes(rows(path / 'dsp.jsonl.zst')) for path in (args.original, args.lagless)]
    for name, (notes, edges) in zip(('original', 'lagless'), streams):
        source = args.original if name == 'original' else args.lagless
        if (source / 'notes.jsonl.zst').exists():
            verify_emitted_edges(source / 'notes.jsonl.zst', edges)
        for kind, data in (('notes', notes), ('edges', edges)):
            with gzip.open(args.output / f'{name}-{kind}.jsonl.gz', 'wt') as target:
                for row in data: target.write(json.dumps(row, separators=(',', ':')) + '\n')
    anchors = json.loads(args.anchors.read_text())
    for side in ('original', 'lagless'):
        if any(a[side] >= b[side] for a, b in zip(anchors, anchors[1:])):
            raise ValueError('command anchors must be strictly monotone')
    result = compare(streams[0][0], streams[1][0], anchors)
    with gzip.open(args.output / 'alignment.jsonl.gz', 'wt') as target:
        for row in result: target.write(json.dumps(row, separators=(',', ':')) + '\n')
    summary = dict(captures=complete, reference_proof=proof, note_counts=[len(s[0]) for s in streams],
                   edge_counts=[dict(Counter(e['kind'] for e in s[1])) for s in streams],
                   alignment=dict(Counter(r['status'] for r in result)),
                   mismatch_fields=dict(Counter(f for r in result for f in r.get('mismatches', []))),
                   relative_over_20_ms=sum(abs(r.get('relative_first_note_ms') or 0) > 20 for r in result),
                   relative_over_45_ms=sum(abs(r.get('relative_first_note_ms') or 0) > 45 for r in result),
                   limitation='Voice ownership/category and command-anchor provenance require separate validation; '
                              'Relative timing uses the earliest exact note pair across voices in each segment; '
                              'these counts are not audibility findings without validated category attribution.')
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')


if __name__ == '__main__':
    main()
