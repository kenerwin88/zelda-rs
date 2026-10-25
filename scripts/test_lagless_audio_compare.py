import unittest
from contextlib import redirect_stdout
import io
import json
from pathlib import Path
import tempfile
import numpy as np
from lagless_audio_compare import align, compare, extract, command_alignment


class AlignmentTests(unittest.TestCase):
    def test_command_anchor_exposes_batched_delivery(self):
        native = [dict(cycle=1000, port=p, value=int(p == 0)) for p in range(4)]
        source = [dict(cycle=2000+d, port=p, value=int(p == 0))
                  for p, d in enumerate((0, 15, 21, 23))]
        result = command_alignment(native, source)
        self.assertEqual(result['status'], 'command-anchored')
        anchor = result['anchors'][0]
        self.assertEqual(anchor['matching_store_prefix'], 4)
        self.assertEqual(anchor['first_relative_timing_difference']['difference_cycles'], -15)
        self.assertIsNone(anchor['first_store_difference'])

    def test_repeated_commands_and_clears_do_not_resolve_identity(self):
        for values in ((0, 0), (1, 1)):
            rows = [dict(cycle=i*100, port=0, value=v) for i, v in enumerate(values)]
            self.assertEqual(command_alignment(rows, rows)['status'], 'unresolved')

    def test_cache_guard_precedes_directory_creation(self):
        with tempfile.TemporaryDirectory() as directory:
            cache = Path(directory)
            output = cache / 'must-not-exist'
            with self.assertRaisesRegex(ValueError, 'read-only'):
                extract(cache, output)
            self.assertFalse(output.exists())

    def test_signed_offset_and_full_stereo_equality(self):
        rng = np.random.default_rng(7)
        source = rng.integers(-20000, 20000, (1200, 2), dtype=np.int16)
        for offset in (-80, 0, 63):
            # Positive offset: native contains an earlier source slice.
            result = align(source, source[100-offset:1100-offset], 100)
            self.assertEqual(result['offset_samples'], offset)
            self.assertTrue(result['identical'])
            self.assertEqual(result['status'], 'matched')
            json.dumps(result)

    def test_residual_and_silence_are_not_equivalence(self):
        rng = np.random.default_rng(9)
        source = rng.integers(-10000, 10000, (1200, 2), dtype=np.int16)
        native = source[100:1100].copy()
        native[400, 1] += 1
        result = align(source, native, 100)
        self.assertEqual(result['offset_samples'], 0)
        self.assertFalse(result['identical'])
        self.assertEqual(align(source, np.zeros((1000, 2), dtype=np.int16), 100)['status'], 'silent')

    def test_constant_reference_is_not_a_correlated_onset(self):
        rng = np.random.default_rng(3)
        signal = rng.integers(-1000, 1000, (1000, 2), dtype=np.int16)
        result = align(np.zeros((1200, 2), dtype=np.int16), signal, 100)
        self.assertEqual(result['status'], 'weak')
        self.assertLessEqual(abs(result['correlation']), 1)

    def test_repeated_waveform_cannot_prove_unique_onset(self):
        period = np.random.default_rng(5).integers(-1000, 1000, (32, 2), dtype=np.int16)
        source = np.tile(period, (128, 1))
        result = align(source, source[1024:3072], 1024)
        self.assertTrue(result['identical'])
        self.assertTrue(result['ambiguous'])

    def test_missing_held_frame_retains_denominator_and_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = np.zeros((534 * 3, 2), dtype='<i2')
            source.tofile(root / 'oracle.pcm')
            np.array([0, 534, 1068, 1602], dtype='<u8').tofile(root / 'oracle.offsets')
            source[:1068].tofile(root / 'native.pcm')
            rows = []
            for i, (start, end) in enumerate(((0, 1), (1, None))):
                rows.append(dict(iteration=i, sample_start=i*534, sample_end=(i+1)*534,
                    source_start=start, source_end=end, callbacks=1, reseeded=False,
                    held=0, lag_nmis=i, masked=0, module=7, submodule=0, room=42, song=1))
            (root / 'map.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in rows))
            with redirect_stdout(io.StringIO()):
                compare(root, root/'native.pcm', root/'map.jsonl', root/'result.json',
                        stride=1, radius=20, window=50)
            result = json.loads((root/'result.json').read_text())
            self.assertEqual(result['counts']['source_exact_iterations'], 1)
            self.assertEqual(result['counts']['frame_comparisons'], 3)
            self.assertEqual(result['counts']['source_exact_frames'], 2)
            self.assertEqual(result['final_drift_samples'], -534)
            self.assertEqual(result['first_difference']['iteration'], 1)
            self.assertEqual(result['alignment'].get('identical_after_alignment', 0), 0)


if __name__ == '__main__':
    unittest.main()
