#!/usr/bin/env python3
"""Summarize `zelda3 --logical-replay` segment lines into a worklist.

Usage: scripts/logical_census_summary.py <replay output>
See docs/parity/lagless-logical-parity.md.
"""
import argparse, collections, pathlib, re, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ram_ref

# The boot intro is part of the exact iteration gate. Later attract scenes
# remain separately reported; all segments still contribute gate failures.
TIMING_DEFINED_MODULES = {'14'}


def parse_segment(line):
    """`logical-segment ...` -> dict. Repeated keys are prefixed by their group."""
    fields = {}
    group = ''
    for token in line.split()[1:]:
        if '=' not in token:
            group = token + '.'
            continue
        key, value = token.split('=', 1)
        if key in fields or (group and key in ('unmatched', 'extra', 'unconsumed', 'incidental', 'engine', 'rom')):
            key = group + key
        fields[key] = value
    return fields


def num(fields, key):
    value = fields.get(key, '0')
    return int(value.split('/')[0]) if value else 0


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('log', type=pathlib.Path)
parser.add_argument('--gate', action='store_true', help='fail on unexplained state, RNG, commands or panics')
parser.add_argument('--expect-iterations', type=int, help='require this many iterations, including intro/attract')
args = parser.parse_args()
lines = args.log.read_text().splitlines()
segments = [parse_segment(line) for line in lines if line.startswith('logical-segment ')]
song = {int(row['start']): row for row in
        (parse_segment(line) for line in lines if line.startswith('logical-segment-song '))}
poly = {int(m.group(1)): (int(m.group(2)), int(m.group(3) or 0)) for m in
        (re.match(r'logical-segment-poly start=(\d+) iterations=(\d+)(?: wram_diverged=(\d+))?', line)
         for line in lines) if m}
poly_fields = {int(row['start']): row for row in
               (parse_segment(line) for line in lines if line.startswith('logical-segment-poly '))}
pacing = [parse_segment(line) for line in lines if line.startswith('logical-segment-pacing ')]
tot = collections.Counter()
by_first = collections.Counter()
examples = {}
worst = []
next_start = 0
contiguous = True
for seg in segments:
    iterations = num(seg, 'iterations')
    if iterations == 0:
        continue
    start = int(seg['start'])
    contiguous &= start == next_start
    next_start = start + iterations
    # Reporting the intro apart must not hide a new ordinary divergence,
    # panic, RNG miss or command mismatch anywhere in that segment.
    tot['all_iterations'] += iterations
    tot['gate_failures'] += num(seg, 'wram_diverged') + ('panicked' in seg)
    for key in ('rng.unconsumed', 'rng.extra', 'apu_reads.unconsumed', 'apu_reads.extra',
                'engine_unmatched', 'rom_unmatched'):
        tot['gate_failures'] += num(seg, key)
    if start < 2000:
        # The producer must not hide boot state differences in timing lanes.
        tot['gate_failures'] += poly.get(start, (0, 0))[1] + num(seg, 'wram_lag')
        tot['gate_failures'] += num(song.get(start, {}), 'wram_diverged')
        for key in ('rng_unconsumed', 'rng_extra'):
            tot['gate_failures'] += num(song.get(start, {}), key)
            tot['gate_failures'] += num(poly_fields.get(start, {}), key)
    if seg.get('start_module') in TIMING_DEFINED_MODULES:
        tot['timing_defined_segments'] += 1
        tot['timing_defined_iterations'] += iterations
        continue
    tot['segments'] += 1
    tot['iterations'] += iterations
    tot['poly_iterations'] += poly.get(start, (0, 0))[0]
    tot['poly_wram_diverged'] += poly.get(start, (0, 0))[1]
    tot['poly_rng_unconsumed'] += num(poly_fields.get(start, {}), 'rng_unconsumed')
    tot['poly_rng_extra'] += num(poly_fields.get(start, {}), 'rng_extra')
    if 'panicked' in seg:
        tot['panicked_segments'] += 1
    wram_clean = 'wram' in seg and seg['wram'] == 'clean'
    tot['wram_clean_segments'] += wram_clean
    tot['wram_lag'] += num(seg, 'wram_lag')
    for key in ('wram_diverged', 'rng_unconsumed', 'rng_extra'):
        tot['song_' + key] += num(song.get(start, {}), key)
    tot['wram_unexplained'] += num(seg, 'wram_diverged')
    video = seg.get('video', '0/0').split('/')
    matched, compared = int(video[0]), int(video[1])
    content = num(seg, 'content')
    for key in ('early', 'late', 'content_lag', 'content_poly', 'content_tear'):
        tot['video_' + key] += num(seg, key)
    tot['video_match'] += matched
    tot['video_compared'] += compared
    tot['video_content'] += content
    for key in ('engine', 'engine_unmatched', 'rom', 'rom_unmatched', 'unmatched_lag', 'unmatched_poly', 'unmatched_song', 'rearmed'):
        tot['apu_' + key] += num(seg, key) + num(seg, 'apu_commands.' + key)
    tot['rng_unconsumed'] += num(seg, 'rng.unconsumed')
    tot['rng_extra'] += num(seg, 'rng.extra')
    tot['apu_reads_unconsumed'] += num(seg, 'apu_reads.unconsumed')
    tot['apu_reads_extra'] += num(seg, 'apu_reads.extra')
    tot['apu_reads_incidental'] += num(seg, 'apu_reads.incidental')
    clean = (wram_clean and content == 0 and num(seg, 'early') + num(seg, 'late') <= 2
             and num(seg, 'engine_unmatched') + num(seg, 'rom_unmatched') == 0
             and num(seg, 'apu_reads.unconsumed') + num(seg, 'apu_reads.extra') == 0
             and num(seg, 'rng.extra') == 0 and num(seg, 'rng.unconsumed') == 0)
    tot['fully_clean_segments'] += clean
    if not wram_clean and 'address' in seg:
        address = int(seg['address'], 16)
        cover = ram_ref.covering(address)
        key = (seg.get('module', '?'), cover[0] if cover else hex(address))
        by_first[key] += 1
        examples.setdefault(key, start)
    worst.append((content, start, matched, compared, seg.get('module')))

vc = tot['video_compared'] or 1
if pacing:
    expected = {num(row, 'start'): num(row, 'iterations') for row in segments}
    observed = {num(row, 'start'): num(row, 'iterations') for row in pacing}
    tot['gate_failures'] += int(expected != observed or len(pacing) != len(segments))
    tot['gate_failures'] += sum(not (0 <= num(row, 'equal') <= num(row, 'iterations'))
                                or num(row, 'held') < 0 or num(row, 'lag') < 0 for row in pacing)
    print('native pacing: iterations={} equal={} held={} ledger_lag={}'.format(
        *(sum(num(row, key) for row in pacing) for key in ('iterations', 'equal', 'held', 'lag'))))
print(f"timing-defined attract segments: {tot['timing_defined_segments']} "
      f"({tot['timing_defined_iterations']} iterations), reported apart")
print(f"segments {tot['segments']}  iterations {tot['iterations']}  fully clean {tot['fully_clean_segments']}  "
      f"wram-clean {tot['wram_clean_segments']}  panicked {tot['panicked_segments']}  "
      f"polygon-thread iterations {tot['poly_iterations']} (wram differed {tot['poly_wram_diverged']})  "
      f"lag-caused wram differences (iterations) {tot['wram_lag']}")
print(f"song-end-poll wram differences (iterations) {tot['song_wram_diverged']}  "
      f"rng unconsumed {tot['song_rng_unconsumed']} extra {tot['song_rng_extra']}  "
      f"commands {tot['apu_unmatched_song']}  unexplained wram iterations {tot['wram_unexplained']}")
print(f"polygon RNG alignment: unconsumed {tot['poly_rng_unconsumed']} extra {tot['poly_rng_extra']}")
print(f"video {tot['video_match']}/{tot['video_compared']} ({100 * tot['video_match'] / vc:.2f}%)  "
      f"early {tot['video_early']}  late {tot['video_late']}  during-lag {tot['video_content_lag']}  "
      f"polygon-thread {tot['video_content_poly']}  torn {tot['video_content_tear']}  content {tot['video_content']} "
      f"({100 * tot['video_content'] / vc:.2f}%)")
print(f"rng unconsumed {tot['rng_unconsumed']}  extra {tot['rng_extra']}  |  "
      f"apu reads unconsumed {tot['apu_reads_unconsumed']}  extra {tot['apu_reads_extra']}  "
      f"incidental iris reads {tot['apu_reads_incidental']}")
print(f"apu commands engine {tot['apu_engine']} (unmatched {tot['apu_engine_unmatched']})  "
      f"rom {tot['apu_rom']} (unmatched {tot['apu_rom_unmatched']})  "
      f"lag-only {tot['apu_unmatched_lag']}  polygon-thread {tot['apu_unmatched_poly']}  "
      f"held re-sends {tot['apu_rearmed']}")
print('first WRAM divergence by (module, variable):')
for (module, name), count in by_first.most_common(40):
    print(f'  {count:4d}  module {module}  {name}  e.g. segment {examples[(module, name)]}')
print('most video content misses:')
for content, start, matched, compared, module in sorted(worst, reverse=True)[:20]:
    print(f'  segment {start} content={content} match={matched}/{compared} wram_first_module={module}')

if args.gate:
    failures = tot['gate_failures']
    complete = any(line.startswith('logical replay:') for line in lines) and bool(segments) and contiguous
    if args.expect_iterations is not None:
        complete &= tot['all_iterations'] == args.expect_iterations
    print(f"census gate: {'PASS' if complete and failures == 0 else 'FAIL'} (unexplained={failures}, complete={complete})")
    sys.exit(0 if complete and failures == 0 else 1)
