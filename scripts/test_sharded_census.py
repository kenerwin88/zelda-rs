import json
import io
from contextlib import redirect_stdout, redirect_stderr
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from sharded_census import (Shard, default_shards, first_difference, merge_csv,
                            merge_logs, ordered_shards, plan_shards, safe_output, main)


def block(start, length, held=0):
    return (f'logical-segment-song start={start} wram_diverged=0 rng_unconsumed=0 rng_extra=0\n'
            f'logical-segment start={start} iterations={length} wram=clean seeded=false\n'
            f'logical-segment-pacing start={start} iterations={length} equal={length} held={held} lag=0 elapsed=0\n')


def log(start, length, held=0):
    return (block(start, length, held) +
            f'logical-pacing-category module=09/00 iterations={length} equal={length} held={held} lag=0 elapsed=0\n' +
            f'logical replay: {length} clean iteration(s); first WRAM divergence None\n')


class PlanningTests(unittest.TestCase):
    def test_balanced_aligned_complete(self):
        shards = plan_shards(1473123, 6)
        self.assertEqual(shards[0].start, 0)
        self.assertEqual(shards[-1].end, 1473123)
        self.assertTrue(all(s.start % 2000 == 0 for s in shards))
        self.assertTrue(all(a.end == b.start for a, b in zip(shards, shards[1:])))
        self.assertLessEqual(max(s.end-s.start for s in shards)-min(s.end-s.start for s in shards), 2000)

    def test_last_partial_segment(self):
        self.assertEqual(plan_shards(4001, 99), [Shard(0, 2000), Shard(2000, 4000), Shard(4000, 4001)])
        self.assertEqual(plan_shards(1, 9), [Shard(0, 1)])
        self.assertEqual(plan_shards(4000, 1), [Shard(0, 4000)])

    def test_post_polygon_resync_is_conditional(self):
        # 730 ends the intro. A clean engine continues at that resync;
        # forcing a new shard there changes native runtime ownership.
        self.assertEqual(plan_shards(8001, 4, [730, 3836, 4000, 8000]), plan_shards(8001, 4))
        self.assertNotIn(730, [s.start for s in plan_shards(8001, 4, [730])])

    def test_invalid_ranges_and_duplicates(self):
        for total, count in [(0, 1), (1, 0), (-1, 3)]:
            with self.assertRaises(ValueError):
                plan_shards(total, count)
        with self.assertRaises(ValueError):
            plan_shards(10, 1, [10])
        for shards in [[Shard(0, 2), Shard(0, 2)], [Shard(1, 2)], [Shard(0, 1)]]:
            with self.assertRaises(ValueError):
                ordered_shards(shards, 2)

    def test_performance_core_default_and_restricted_fallback(self):
        with patch('sharded_census.subprocess.check_output', return_value=b'10\n'):
            self.assertEqual(default_shards(), 8)
        with patch('sharded_census.subprocess.check_output', side_effect=OSError()):
            self.assertEqual(default_shards(), 1)


class MergeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.out = self.root / 'out'

    def put(self, name, content):
        path = self.root / name
        path.write_bytes(content.encode())
        return path

    def test_csv_orders_parts_single_header_exact_bytes(self):
        a = self.put('a', 'iteration,value\r\n0,01\r\n1,02\r\n')
        b = self.put('b', 'iteration,value\r\n2,03\r\n')
        merge_csv({Shard(2, 3): b, Shard(0, 2): a}, self.out, 3)
        self.assertEqual(self.out.read_bytes(), b'iteration,value\r\n0,01\r\n1,02\r\n2,03\r\n')

    def test_csv_rejects_duplicate_gap_wrong_header_and_truncation(self):
        a = self.put('a', 'iteration,value\n0,x\n')
        for content in ['iteration,value\n0,x\n', 'iteration,value\n2,x\n',
                        'iteration,other\n1,x\n', 'iteration,value\n1,x',
                        'iteration,value\n', 'iteration,value\n1,x,y\n',
                        'iteration,value\niteration,value\n']:
            with self.subTest(content=content), self.assertRaises(ValueError):
                merge_csv({Shard(0, 1): a, Shard(1, 2): self.put('b', content)}, self.out, 2)

    def test_logs_order_blocks_and_add_footers(self):
        a = self.put('a', log(0, 2, 1))
        b = self.put('b', block(0, 0) + log(2, 1, 2))
        merge_logs({Shard(2, 3): b, Shard(0, 2): a}, self.out, 3)
        expected = (block(0, 2, 1) + block(2, 1, 2) +
                    'logical-pacing-category module=09/00 iterations=3 equal=3 held=3 lag=0 elapsed=0\n' +
                    'logical replay: 3 clean iteration(s); first WRAM divergence None\n')
        self.assertEqual(self.out.read_bytes(), expected.encode())

    def test_logs_preserve_semantic_seed_fields_and_diagnostics(self):
        diagnostic = 'logical-deferred-work iteration=0 work=Pending { x: 1 }\n'
        a = self.put('a', diagnostic + log(0, 1).replace('seeded=false', 'seeded=true'))
        merge_logs({Shard(0, 1): a}, self.out, 1)
        self.assertEqual(self.out.read_bytes(), a.read_bytes())

    def test_logs_post_polygon_irregular_segment_and_last_partial(self):
        a = self.put('a', block(0, 730) +
                     'logical-segment-poly start=0 iterations=730 wram_diverged=0 rng_unconsumed=0 rng_extra=0\n' +
                     block(730, 1270) +
                     'logical replay: 2000 clean iteration(s); first WRAM divergence None\n')
        b = self.put('b', log(2000, 1))
        merge_logs({Shard(0, 2000): a, Shard(2000, 2001): b}, self.out, 2001)
        self.assertIn(b'logical-segment start=730 iterations=1270', self.out.read_bytes())
        self.assertIn(b'logical-segment start=2000 iterations=1', self.out.read_bytes())

    def test_logs_reject_duplicates_gaps_and_missing_footer(self):
        for content in [block(0, 1) + log(0, 1), log(1, 1), block(0, 1),
                        log(0, 1) + 'logical replay: 1 clean iteration(s); first WRAM divergence None\n']:
            with self.subTest(content=content), self.assertRaises(ValueError):
                merge_logs({Shard(0, 1): self.put('a', content)}, self.out, 1)

    def test_first_difference_including_eof(self):
        a, b = self.put('a', 'same\nleft\n'), self.put('b', 'same\n')
        self.assertEqual(first_difference(a, b), dict(line=2, control='left\n', sharded=None))
        self.assertIsNone(first_difference(a, a))

    def test_read_only_guards_before_creation(self):
        for path in [self.root / '.git/parity-oracle-cache/out',
                     self.root / 'target/logical-ledger/out', self.root / 'ledger/out']:
            with self.assertRaises(ValueError):
                safe_output(path, self.root / 'ledger', self.root / 'initial.srm')
            self.assertFalse(path.exists())


class RunnerTests(unittest.TestCase):
    def test_publish_only_identical_shards_otherwise_control(self):
        for changed in (False, True, 'failed'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                ledger = root / 'ledger'
                (ledger / 'seeds').mkdir(parents=True)
                (ledger / 'seeds/iter-2000.bin').touch()
                (ledger / 'iterations.jsonl').write_text(''.join(
                    json.dumps(dict(index=i)) + '\n' for i in range(2001)))
                binary, sram = root / 'binary', root / 'initial.srm'
                binary.write_bytes(b'binary')
                sram.write_bytes(b'sram')
                out = root / 'result'

                def replay(binary, ledger, sram, shard, directory, live):
                    if changed == 'failed' and shard.start:
                        raise RuntimeError('candidate replay failed')
                    directory.mkdir()
                    text = ''
                    for start in range(shard.start, shard.end, 2000):
                        text += block(start, min(start+2000, shard.end)-start)
                    length = shard.end-shard.start
                    text += (f'logical-pacing-category module=09/00 iterations={length} equal={length} held=0 lag=0 elapsed=0\n'
                             f'logical replay: {length} clean iteration(s); first WRAM divergence None\n')
                    if changed and shard.start:
                        text = text.replace('seeded=false', 'seeded=true')
                    (directory / 'stdout.log').write_text(text)
                    (directory / 'predictor.csv').write_text('iteration,value\n' + ''.join(
                        f'{i},0\n' for i in range(shard.start, shard.end)))
                    return dict(seconds=0.01, exit=0)

                def fake_gate(log, output, iterations):
                    output.write_bytes(b'census gate: PASS\n')
                    return 0

                argv = ['sharded_census.py', str(ledger), str(sram), '--binary', str(binary),
                        '--output', str(out), '--shards', '2', '--audit-shards']
                with patch('sys.argv', argv), patch('sharded_census.run_one', side_effect=replay), \
                     patch('sharded_census.gate', side_effect=fake_gate), \
                     redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                    main()
                verdict = json.loads((out / 'verdict.json').read_text())
                self.assertEqual(verdict['selected'], 'control' if changed else 'merged')
                for name in ('stdout.log', 'predictor.csv', 'gate.txt'):
                    self.assertEqual((out/name).read_bytes(), (out/'control'/name).read_bytes())
                if changed == 'failed':
                    self.assertIn('candidate replay failed', verdict['audit_error'])
                else:
                    self.assertEqual(bool((out/'merged/stdout.diff').read_bytes()), changed)


if __name__ == '__main__':
    unittest.main()
