#!/usr/bin/env python3
"""CPU-only census runner and strict shard-equivalence audit.

Clean periodic boundaries CONTINUE the native engine; --start forces a seed.
Consequently neither current lane is known to be independently shardable.
Normal execution conservatively runs one process. --audit-shards measures and
compares speculative shards, publishing them only if log, predictor and gate
are byte-identical to a fresh control. It never edits the replay or its seeds.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
import difflib
import hashlib
import itertools
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
SEED_EVERY = 2000


@dataclass(frozen=True, order=True)
class Shard:
    start: int
    end: int  # exclusive; --frames is an absolute end, not a count


def default_shards():
    """Leave two performance cores free; fall back to one if unavailable."""
    try:
        cores = int(subprocess.check_output(
            ['sysctl', '-n', 'hw.perflevel0.physicalcpu'], stderr=subprocess.DEVNULL))
    except (OSError, ValueError, subprocess.CalledProcessError):
        return 1
    return max(1, cores - 2)


def plan_shards(iterations, count, resync=()):
    """Balance periodic segments, including the last partial segment.

    A resync file lists *conditional* post-polygon seeds, not unconditional
    split points. Do not round to these or force an extra seed there. Each
    worker uses the original replay's resync handling inside its range.
    """
    if iterations < 1 or count < 1:
        raise ValueError('iterations and shards must be positive')
    if any(point < 0 or point >= iterations for point in resync):
        raise ValueError('resync outside route')
    segments = (iterations + SEED_EVERY - 1) // SEED_EVERY
    count = min(count, segments)
    cuts = [min((i * segments // count) * SEED_EVERY, iterations)
            for i in range(count)] + [iterations]
    return [Shard(a, b) for a, b in zip(cuts, cuts[1:])]


def ordered_shards(shards, iterations):
    ordered = sorted(shards)
    end = 0
    for shard in ordered:
        if shard.start != end or shard.end <= shard.start:
            raise ValueError('duplicate, overlapping or missing shard range')
        end = shard.end
    if end != iterations:
        raise ValueError('incomplete shard route')
    return ordered


def merge_csv(parts, output, iterations):
    """Preserve every CSV byte; reject header drift, duplicates and gaps."""
    ordered = ordered_shards(parts, iterations)
    header = None
    expected = 0
    with Path(output).open('wb') as dest:
        for shard in ordered:
            with Path(parts[shard]).open('rb') as source:
                current = source.readline()
                if not current.startswith(b'iteration,') or not current.endswith(b'\n'):
                    raise ValueError('invalid predictor header')
                if header is None:
                    header = current
                    dest.write(header)
                elif current != header:
                    raise ValueError('predictor headers differ')
                for row in source:
                    index = int(row.split(b',', 1)[0])
                    if index != expected or not shard.start <= index < shard.end:
                        raise ValueError(f'duplicate, unordered or missing predictor iteration: {index}; expected {expected}')
                    if not row.endswith(b'\n') or row.count(b',') != header.count(b','):
                        raise ValueError('truncated predictor row')
                    dest.write(row)
                    expected += 1
                if expected != shard.end:
                    raise ValueError('incomplete predictor shard')
    if expected != iterations:
        raise ValueError('incomplete predictor route')


def fields(line):
    return dict(re.findall(rb'([\w]+)=([^ ]+)', line.rstrip()))


def merge_logs(parts, output, iterations):
    """Join segment blocks in order and add process-local footer counters.

    No semantic lines/fields are normalized away. Empty startup segment blocks
    in nonzero-start processes are omitted; category totals and the final clean
    count are summed. Each process's stderr remains a separate artifact.
    """
    ordered = ordered_shards(parts, iterations)
    categories = {}
    clean = 0
    first = None
    seen = set()
    next_start = 0
    with Path(output).open('wb') as dest:
        for shard in ordered:
            lines = Path(parts[shard]).read_bytes().splitlines(keepends=True)
            empty_starts = {int(fields(line)[b'start']) for line in lines
                            if line.startswith(b'logical-segment ') and fields(line).get(b'iterations') == b'0'}
            footer_seen = False
            segment_lengths = {}
            pacing_lengths = {}
            song_starts = set()
            for line in lines:
                if line.startswith(b'logical-segment'):
                    values = fields(line)
                    start = int(values[b'start'])
                    if start in empty_starts:
                        continue
                    key = (line.split(b' ', 1)[0], start)
                    if key in seen:
                        raise ValueError('duplicate segment record')
                    seen.add(key)
                    if not shard.start <= start < shard.end:
                        raise ValueError('segment outside shard')
                    if line.startswith(b'logical-segment '):
                        length = int(values[b'iterations'])
                        if start != next_start or start + length > shard.end:
                            raise ValueError('unordered, overlapping or missing segment')
                        next_start = start + length
                        segment_lengths[start] = length
                    elif line.startswith(b'logical-segment-pacing '):
                        pacing_lengths[start] = int(values[b'iterations'])
                    elif line.startswith(b'logical-segment-song '):
                        song_starts.add(start)
                    dest.write(line)
                elif line.startswith(b'logical-pacing-category '):
                    values = fields(line)
                    module = values.pop(b'module', b'')
                    if not re.fullmatch(rb'[0-9a-f]{2}/[0-9a-f]{2}', module) or set(values) != {
                        b'iterations', b'equal', b'held', b'lag', b'elapsed'
                    }:
                        raise ValueError('invalid pacing category')
                    totals = categories.setdefault(module, {})
                    for key, value in values.items():
                        totals[key] = totals.get(key, 0) + int(value)
                elif line.startswith(b'logical replay: '):
                    if footer_seen:
                        raise ValueError('duplicate replay footer')
                    footer_seen = True
                    match = re.fullmatch(rb'logical replay: (\d+) clean iteration\(s\); first WRAM divergence (None|Some\((\d+)\))\n', line)
                    if not match:
                        raise ValueError('invalid replay footer')
                    clean += int(match[1])
                    if match[3] is not None:
                        first = min(first, int(match[3])) if first is not None else int(match[3])
                else:
                    # Diagnostics are evidence: preserve them, even duplicates.
                    # A repeated deferred-work diagnostic can itself disprove
                    # process independence and must not be hidden by merging.
                    dest.write(line)
            if not footer_seen or next_start != shard.end:
                raise ValueError('incomplete log shard')
            if segment_lengths != pacing_lengths or song_starts != set(segment_lengths):
                raise ValueError('missing or inconsistent song/pacing segment records')
        for module, totals in sorted(categories.items()):
            dest.write(b'logical-pacing-category module=' + module + b''.join(
                b' ' + key + b'=' + str(totals[key]).encode()
                for key in (b'iterations', b'equal', b'held', b'lag', b'elapsed')) + b'\n')
        divergence = 'None' if first is None else f'Some({first})'
        dest.write(f'logical replay: {clean} clean iteration(s); first WRAM divergence {divergence}\n'.encode())


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def first_difference(a, b):
    with Path(a).open('rb') as left, Path(b).open('rb') as right:
        for number, (x, y) in enumerate(itertools.zip_longest(left, right), 1):
            if x != y:
                return dict(line=number, control=x.decode(errors='replace') if x else None,
                            sharded=y.decode(errors='replace') if y else None)
    return None


def compare_outputs(control, merged, iterations):
    """Retain an exact line diff, plus streaming predictor mismatch counts."""
    differences = {name: first_difference(control / name, merged / name)
                   for name in ('stdout.log', 'predictor.csv', 'gate.txt')}
    with (merged / 'stdout.diff').open('w') as diff:
        diff.writelines(difflib.unified_diff(
            (control / 'stdout.log').read_text().splitlines(keepends=True),
            (merged / 'stdout.log').read_text().splitlines(keepends=True),
            fromfile='control/stdout.log', tofile='merged/stdout.log'))
    counts = {}
    changed = 0
    with (control / 'predictor.csv').open() as left, (merged / 'predictor.csv').open() as right:
        header = left.readline().strip().split(',')
        if right.readline().strip().split(',') != header:
            raise ValueError('control/merged predictor headers differ')
        for a, b in itertools.zip_longest(left, right):
            if a != b:
                changed += 1
                if a is None or b is None:
                    counts['missing_rows'] = counts.get('missing_rows', 0) + 1
                    continue
                for name, x, y in zip(header, a.strip().split(','), b.strip().split(',')):
                    if x != y:
                        counts[name] = counts.get(name, 0) + 1
    (merged / 'predictor-differences.json').write_text(json.dumps(
        dict(iterations=iterations, differing_rows=changed, columns=counts), indent=2) + '\n')
    return differences


def run_one(binary, ledger, sram, shard, directory, live):
    directory.mkdir()
    command = [str(binary), '--logical-replay', str(ledger), str(sram),
               '--report', '0', '--seed-every', str(SEED_EVERY),
               '--start', str(shard.start), '--frames', str(shard.end),
               '--pacing-predictor', str(directory / 'predictor.csv')]
    if live:
        command.append('--live-apu-reads')
    # Match a canonical invocation even if the caller has unrelated debug vars.
    env = {k: v for k, v in os.environ.items() if not k.startswith('ZELDA3_')}
    env['ZELDA3_SAVE_DIR'] = str(directory / 'save')
    started = time.monotonic()
    with (directory / 'stdout.log').open('wb') as out, (directory / 'stderr.log').open('wb') as err:
        with subprocess.Popen(command, stdout=out, stderr=err, env=env, cwd=ROOT) as proc:
            try:
                code = proc.wait()
            except BaseException:
                proc.terminate()
                proc.wait()
                raise
    receipt = dict(command=command, seconds=time.monotonic() - started, exit=code)
    (directory / 'run.json').write_text(json.dumps(receipt, indent=2) + '\n')
    if code:
        raise RuntimeError(f'replay exited {code}: {directory / "stderr.log"}')
    return receipt


def gate(log, output, iterations):
    result = subprocess.run([sys.executable, str(ROOT / 'scripts/logical_census_summary.py'),
                             str(log), '--gate', '--expect-iterations', str(iterations)],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    output.write_bytes(result.stdout)
    return result.returncode


def safe_output(output, ledger, sram):
    output = output.resolve()
    ledger, sram = ledger.resolve(), sram.resolve()
    if any(name in output.parts for name in ('parity-oracle-cache', 'logical-ledger')):
        raise ValueError('oracle cache and logical-ledger are read-only')
    if output == sram or output == ledger or output.is_relative_to(ledger) or sram.is_relative_to(output):
        raise ValueError('output overlaps replay inputs')
    if output.exists():
        raise ValueError('output already exists; choose a fresh directory')
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('ledger', type=Path)
    parser.add_argument('sram', type=Path)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/parity/zelda3')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--shards', type=int, default=default_shards())
    parser.add_argument('--frames', type=int, help='optional prefix length; retain the full ledger for input lookahead')
    parser.add_argument('--live-apu-reads', action='store_true')
    parser.add_argument('--audit-shards', action='store_true', help='test speculative shards against a fresh unsharded control; retain the control on any difference')
    args = parser.parse_args()
    try:
        ledger, sram, binary = args.ledger.resolve(), args.sram.resolve(), args.binary.resolve()
        output = safe_output(args.output, ledger, sram)
        # Validate the complete route, not map_while's silent partial-line prefix.
        iterations = 0
        with (ledger / 'iterations.jsonl').open() as records:
            for index, line in enumerate(records):
                if json.loads(line)['index'] != index:
                    raise ValueError('ledger indices must be contiguous from zero')
                iterations += 1
        if args.frames is not None:
            if not 1 <= args.frames <= iterations:
                raise ValueError('frames must be within the complete ledger')
            iterations = args.frames
        resync_file = ledger / 'seeds/resync.txt'
        resync = [int(x) for x in resync_file.read_text().splitlines()] if resync_file.exists() else []
        resync = [point for point in resync if point < iterations]
        shards = plan_shards(iterations, args.shards, resync)
        if args.audit_shards:
            for shard in shards[1:]:
                if not (ledger / f'seeds/iter-{shard.start}.bin').is_file():
                    raise ValueError(f'missing seed at {shard.start}')
        output.mkdir(parents=True)
        print('WARNING: clean boundaries retain native state in both lanes; using an unsharded control.', file=sys.stderr, flush=True)
        control = output / 'control'
        receipt = run_one(binary, ledger, sram, Shard(0, iterations), control, args.live_apu_reads)
        if gate(control / 'stdout.log', control / 'gate.txt', iterations):
            raise RuntimeError('unsharded control gate failed')
        verdict = dict(lane='live' if args.live_apu_reads else 'rom', iterations=iterations,
                       binary_sha256=digest(binary), sram_sha256=digest(sram),
                       ledger_sha256=digest(ledger / 'iterations.jsonl'),
                       control_seconds=receipt['seconds'], selected='control',
                       plan=[vars(s) for s in shards], ignored_stdout_lines=[])
        if args.audit_shards and len(shards) > 1:
            started = time.monotonic()
            try:
                with ThreadPoolExecutor(max_workers=len(shards)) as pool:
                    futures = [pool.submit(run_one, binary, ledger, sram, shard,
                                           output / f'shard-{shard.start:07}', args.live_apu_reads)
                               for shard in shards]
                    for future in futures:
                        future.result()
                verdict['sharded_seconds'] = time.monotonic() - started
                merged = output / 'merged'
                merged.mkdir()
                merge_csv({s: output / f'shard-{s.start:07}/predictor.csv' for s in shards}, merged / 'predictor.csv', iterations)
                merge_logs({s: output / f'shard-{s.start:07}/stdout.log' for s in shards}, merged / 'stdout.log', iterations)
                verdict['sharded_gate_exit'] = gate(merged / 'stdout.log', merged / 'gate.txt', iterations)
                verdict['differences'] = compare_outputs(control, merged, iterations)
                if not any(verdict['differences'].values()) and verdict['sharded_gate_exit'] == 0:
                    verdict['selected'] = 'merged'
            except (OSError, ValueError, RuntimeError) as error:
                verdict['audit_error'] = str(error)
            verdict['audit_seconds'] = time.monotonic() - started
            if verdict['selected'] == 'control':
                print('WARNING: shard equivalence failed; publishing the unsharded control. See verdict.json.', file=sys.stderr)
        selected = output / verdict['selected']
        for name in ('stdout.log', 'predictor.csv', 'gate.txt'):
            # Hard links avoid doubling the full predictors' disk use.
            os.link(selected / name, output / name)
        (output / 'verdict.json').write_text(json.dumps(verdict, indent=2) + '\n')
        print(json.dumps(verdict, indent=2))
    except (OSError, ValueError, RuntimeError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
