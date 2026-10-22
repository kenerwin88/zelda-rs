#!/usr/bin/env python3
"""Record/compare the native headless full route; never writes the oracle cache."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
CACHE = Path('/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18')


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['record', 'compare'])
    parser.add_argument('ledger', nargs='?', type=Path,
                        help='defaults to target/state-ledger/baseline-<mode>.jsonl.zst')
    parser.add_argument('--mode', choices=['default', 'lag-free'], default='default')
    parser.add_argument('--cache', type=Path, default=CACHE)
    parser.add_argument('--frames', type=int, default=1_581_079)
    parser.add_argument('--interval', type=int, default=1)
    parser.add_argument('--rgb', action='store_true', help='also hash CPU diagnostic RGB (slower)')
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/parity/zelda3')
    parser.add_argument('--rom', type=Path, help='optional ROM-backed smoke (requires zelda3_assets.dat)')
    args = parser.parse_args()
    if args.ledger is None:
        args.ledger = ROOT / f'target/state-ledger/baseline-{args.mode}.jsonl.zst'
    if args.frames < 1 or args.interval < 1:
        parser.error('frames and interval must be positive')
    args.ledger = args.ledger.resolve()
    cache = args.cache.resolve()
    if args.action == 'record':
        if args.ledger.is_relative_to(cache):
            parser.error('the oracle cache is read-only')
        if args.ledger.exists():
            parser.error('output exists; choose a new filename to re-bless explicitly')
        args.ledger.parent.mkdir(parents=True, exist_ok=True)
    inputs, sram = cache / 'oracle-av-hashes.jsonl', cache / 'initial.srm'
    # Pin all launch inputs alongside the result; the ledger is a streaming data file.
    provenance = {
        'schema': 2, 'mode': args.mode, 'frames': args.frames, 'interval': args.interval, 'rgb': args.rgb,
        'input_sha256': sha256(inputs), 'sram_sha256': sha256(sram),
        'rom_sha256': sha256(args.rom) if args.rom else None,
        'binary_sha256': sha256(args.binary),
        'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'git_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT)),
    }
    metadata = Path(str(args.ledger) + '.meta.json')
    if args.action == 'compare' and metadata.exists():
        original = json.loads(metadata.read_text())
        for key in ['schema', 'mode', 'frames', 'interval', 'rgb', 'input_sha256', 'sram_sha256', 'rom_sha256']:
            if original.get(key, False if key == 'rgb' else None) != provenance[key]:
                parser.error(f'baseline launch mismatch: {key}')
    env = {key: value for key, value in os.environ.items() if not key.startswith('ZELDA3_')}
    env.update(ZELDA3_SMOKE_INPUT_LEDGER=str(inputs), ZELDA3_SMOKE_SRAM=str(sram))
    if args.rom:
        env['ZELDA3_SMOKE_ROM'] = str(args.rom.resolve())
    flag = '--state-ledger' if args.action == 'record' else '--compare-state-ledger'
    command = [str(args.binary.resolve()), '--standalone-smoke', str(args.frames),
               flag, str(args.ledger), '--ledger-interval', str(args.interval)]
    if args.rgb:
        command.append('--ledger-rgb')
    if args.mode == 'lag-free':
        command.append('--lag-free')
    # Isolate any in-game SRAM persistence from the user's actual saves/cache.
    with tempfile.TemporaryDirectory(prefix='zelda3-state-ledger-') as save_dir:
        env['ZELDA3_SAVE_DIR'] = save_dir
        started = time.monotonic()
        result = subprocess.run(command, cwd=ROOT, env=env)
        seconds = time.monotonic() - started
    print(f'{args.action}: mode={args.mode} exit={result.returncode} frames={args.frames} '
          f'seconds={seconds:.3f}' + (f' fps={args.frames / seconds:.1f}' if result.returncode == 0 else ''), flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
    if args.action == 'record':
        provenance.update(seconds=seconds, fps=args.frames / seconds,
                          ledger_sha256=sha256(args.ledger))
        metadata.write_text(json.dumps(provenance, indent=2) + '\n')
    else:
        print('state ledger matches (all records and EOF)')


if __name__ == '__main__':
    main()
