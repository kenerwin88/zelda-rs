#!/usr/bin/env python3
"""Summarize `zelda3 --logical-replay` segment lines into a worklist.

Usage: scripts/logical_census_summary.py <replay output>
See docs/parity/lagless-logical-parity.md.
"""
import collections, pathlib, re, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import ram_ref

# Intro and attract modes run the polygon thread / attract scenes on the NMI
# clock; they are timing-defined and reported apart.
TIMING_DEFINED_MODULES = {'00', '14'}


def parse_segment(line):
    """`logical-segment ...` -> dict. Repeated keys are prefixed by their group."""
    fields = {}
    group = ''
    for token in line.split()[1:]:
        if '=' not in token:
            group = token + '.'
            continue
        key, value = token.split('=', 1)
        if key in fields or (group and key in ('unmatched', 'extra', 'unconsumed', 'engine', 'rom')):
            key = group + key
        fields[key] = value
    return fields


def num(fields, key):
    value = fields.get(key, '0')
    return int(value.split('/')[0]) if value else 0


segments = [parse_segment(line) for line in open(sys.argv[1]) if line.startswith('logical-segment ')]
poly = {int(m.group(1)): (int(m.group(2)), int(m.group(3) or 0)) for m in
        (re.match(r'logical-segment-poly start=(\d+) iterations=(\d+)(?: wram_diverged=(\d+))?', line)
         for line in open(sys.argv[1])) if m}
tot = collections.Counter()
by_first = collections.Counter()
examples = {}
worst = []
for seg in segments:
    iterations = num(seg, 'iterations')
    if iterations == 0:
        continue
    start = int(seg['start'])
    if seg.get('start_module') in TIMING_DEFINED_MODULES:
        tot['timing_defined_segments'] += 1
        tot['timing_defined_iterations'] += iterations
        continue
    tot['segments'] += 1
    tot['iterations'] += iterations
    tot['poly_iterations'] += poly.get(start, (0, 0))[0]
    tot['poly_wram_diverged'] += poly.get(start, (0, 0))[1]
    if 'panicked' in seg:
        tot['panicked_segments'] += 1
    wram_clean = 'wram' in seg and seg['wram'] == 'clean'
    tot['wram_clean_segments'] += wram_clean
    video = seg.get('video', '0/0').split('/')
    matched, compared = int(video[0]), int(video[1])
    content = num(seg, 'content')
    for key in ('early', 'late', 'content_lag', 'content_poly', 'content_tear'):
        tot['video_' + key] += num(seg, key)
    tot['video_match'] += matched
    tot['video_compared'] += compared
    tot['video_content'] += content
    for key in ('apu_commands.rom', 'apu_commands.unmatched', 'apu_commands.engine'):
        tot[key] += num(seg, key)
    tot['apu_rom_unmatched'] += num(seg, 'unmatched') if 'apu_commands.unmatched' not in seg else 0
    tot['rng_unconsumed'] += num(seg, 'rng.unconsumed')
    tot['rng_extra'] += num(seg, 'rng.extra')
    tot['apu_reads_unconsumed'] += num(seg, 'apu_reads.unconsumed')
    tot['apu_reads_extra'] += num(seg, 'apu_reads.extra')
    clean = (wram_clean and content == 0 and num(seg, 'early') + num(seg, 'late') <= 2
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
print(f"timing-defined (intro/attract) segments: {tot['timing_defined_segments']} "
      f"({tot['timing_defined_iterations']} iterations), reported apart")
print(f"segments {tot['segments']}  iterations {tot['iterations']}  fully clean {tot['fully_clean_segments']}  "
      f"wram-clean {tot['wram_clean_segments']}  panicked {tot['panicked_segments']}  "
      f"polygon-thread iterations {tot['poly_iterations']} (wram differed {tot['poly_wram_diverged']})")
print(f"video {tot['video_match']}/{tot['video_compared']} ({100 * tot['video_match'] / vc:.2f}%)  "
      f"early {tot['video_early']}  late {tot['video_late']}  during-lag {tot['video_content_lag']}  "
      f"polygon-thread {tot['video_content_poly']}  torn {tot['video_content_tear']}  content {tot['video_content']} "
      f"({100 * tot['video_content'] / vc:.2f}%)")
print(f"rng unconsumed {tot['rng_unconsumed']}  extra {tot['rng_extra']}  |  "
      f"apu reads unconsumed {tot['apu_reads_unconsumed']}  extra {tot['apu_reads_extra']}")
print('first WRAM divergence by (module, variable):')
for (module, name), count in by_first.most_common(40):
    print(f'  {count:4d}  module {module}  {name}  e.g. segment {examples[(module, name)]}')
print('most video content misses:')
for content, start, matched, compared, module in sorted(worst, reverse=True)[:20]:
    print(f'  segment {start} content={content} match={matched}/{compared} wram_first_module={module}')
