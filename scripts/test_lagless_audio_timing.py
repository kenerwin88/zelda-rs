import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import lagless_audio_timing as timing


def command(value, iteration=0, frame=0, epoch=0, port=2):
    return dict(value=value, port=port, iteration=iteration, frame=frame,
                frame_upper=frame, epoch=epoch, epoch_frame=frame,
                context=[7, 0, 90, 19], kind='sfx1')


class TimingTests(unittest.TestCase):
    def test_dungeon_entry_tracks_include_first_post_load_request(self):
        rows = []
        for i, module in enumerate((6, 7, 7)):
            value = 16 if i == 1 else 0
            rows.append(dict(schema=1, iteration=i, lost_pushes=0, callbacks=1,
                held=0, masked=0, source_start=100+i, source_end=101+i,
                native_start=i, source_context=[module, 0, 1, 0],
                native_context=[module, 0, 1, 0], reseeded=False,
                source_writes=[[0, value], [1, 0], [2, 0], [3, 0]],
                native_pushes=[[i, [value, 0, 0, 0]]], source_reads=[], native_polls=[]))
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'trace.zst'
            path.write_bytes(subprocess.check_output(['zstd', '-q', '-c'],
                input=''.join(json.dumps(r) + '\n' for r in rows).encode()))
            result = timing.analyze(path, Path(d) / 'out')
            self.assertEqual(result['transition_groups']['dungeon_entry_track_request']['matched'], 1)

    def test_insert_delete_and_repeated_content_preserve_order(self):
        source = [command(v, i) for i, v in enumerate([5, 5, 6, 7])]
        native = [command(v, i) for i, v in enumerate([5, 8, 5, 7])]
        pairs = list(timing.align(source, native))
        self.assertEqual([(x and x['value'], y and y['value']) for x, y in pairs],
                         [(5, 5), (None, 8), (5, 5), (6, None), (7, 7)])
        self.assertEqual(sum(x is not None for x, _ in pairs), len(source))
        self.assertEqual(sum(y is not None for _, y in pairs), len(native))

    def test_never_match_across_reseed_or_distant_identical_effect(self):
        for other in (command(1, epoch=1), command(1, iteration=100)):
            pairs = list(timing.align([command(1)], [other]))
            self.assertFalse(any(x and y for x, y in pairs))

    def test_upload_protocol_does_not_discard_long_lag_handler_stream(self):
        normal = [[1, 0], [2, 0], [3, 10]] * 100
        upload = [[2, 0], [3, 208], [1, 1], [0, 204],
                  [0, 0], [1, 99], [0, 1], [1, 77],
                  [2, 0], [3, 8], [1, 0], [0, 5]]
        tail = [[0, 19], [1, 0], [2, 0], [3, 0]]
        writes = normal + [[0, 255]] + upload + tail
        commands, transfers = timing.source_commands(writes)
        self.assertEqual(len(commands), len(normal) + 1 + len(tail))
        self.assertEqual(transfers, [(301, 313)])
        self.assertEqual(commands[-4], (313, 0, 19))
        self.assertEqual(timing.source_commands(normal)[1], [])
        with self.assertRaisesRegex(ValueError, 'unterminated'):
            timing.source_commands(upload[:-4])

    def test_signed_bounds_and_nonoverlapping_distribution(self):
        a = command(1, frame=10)
        a['frame_upper'] = 14
        b = command(1, frame=12)
        result = timing.comparison(a, b)
        self.assertEqual(result['offset_bounds_frames'], [-2, 2])
        self.assertAlmostEqual(result['offset_ms'], 1000 / 30)
        summary = timing.summarize([result, timing.comparison(a, None)])
        self.assertEqual(summary['two_to_five'], 1)
        self.assertEqual(summary['bounds_include_zero'], 1)
        self.assertEqual(summary['source_only'], 1)
        self.assertEqual([timing.bucket(x) for x in [0, -1, 2, -5, 6]],
                         ['exact', 'one_frame', 'two_to_five', 'two_to_five', 'over_five'])

    def test_bounded_matcher_uses_nearest_edit_diagonal(self):
        # A closer-in-time match may require more skipped commands. The
        # documented rule minimizes skips first, consistently across runs.
        source = [command(1, 0), command(2, 1), command(3, 2)]
        native = [command(9, 0), command(1, 9), command(2, 1), command(3, 2)]
        pairs = list(timing.align(source, native))
        self.assertEqual([(x and x['value'], y and y['value']) for x, y in pairs],
                         [(None, 9), (1, 1), (2, 2), (3, 3)])

    def test_request_relative_duration_preserves_source_uncertainty(self):
        source, native = command(0, frame=10), command(0, frame=9)
        source['frame_upper'] = 14
        source['request'] = command(19, frame=0)
        source['request']['frame_upper'] = 2
        native['request'] = command(19, frame=0)
        result = timing.comparison(source, native)
        self.assertEqual(result['request_relative_offset_frames'], -1)
        self.assertEqual(result['request_relative_bounds_frames'], [-5, 1])

    def test_trace_accounting_normalization_epochs_and_early_end(self):
        def row(i, value, polls, reseeded=False):
            return dict(schema=1, iteration=i, lost_pushes=0, callbacks=1, held=0, masked=0,
                        source_start=100 + i * 2, source_end=102 + i * 2,
                        native_start=i, source_context=[7, 0, 90, 19],
                        native_context=[7, 0, 90, 19], reseeded=reseeded,
                        source_writes=[[0, value], [1, 0], [2, 0], [3, 0]],
                        native_pushes=[[i, [value, 0, 0, 0]]],
                        source_reads=[[0, polls[0]]], native_polls=[polls[1]])
        rows = [row(0, 19, [19, 1]), row(1, 19, [19, 0]),
                row(2, 0, [0, 0]), row(3, 19, [0, 0], True)]
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'trace.zst'
            path.write_bytes(subprocess.check_output(['zstd', '-q', '-c'],
                input=''.join(json.dumps(r) + '\n' for r in rows).encode()))
            result = timing.analyze(path, Path(d) / 'out')
            self.assertEqual(result['commands']['matched'], 2)
            self.assertEqual(result['totals']['source_nonzero_repeats'], 1)
            self.assertEqual(result['raw_nonzero_writes']['matched'], 3)
            self.assertEqual(len(result['epochs']), 2)
            self.assertEqual(result['song_ends']['matched'], 1)
            self.assertEqual(result['song_ends']['min_offset_frames'], -3)
            self.assertEqual(result['commands']['epoch_exact'], 2)
            rows[1]['lost_pushes'] = 1
            path.write_bytes(subprocess.check_output(['zstd', '-q', '-c'],
                input=''.join(json.dumps(r) + '\n' for r in rows).encode()))
            with self.assertRaisesRegex(ValueError, 'intermediate'):
                timing.analyze(path, Path(d) / 'bad')


if __name__ == '__main__':
    unittest.main()
