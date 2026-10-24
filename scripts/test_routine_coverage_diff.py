"""Small deterministic checks for the diagnostic join and conservation gates."""
import tempfile
import unittest
from pathlib import Path

from routine_coverage_diff import compare, load, windows


class CoverageDiffTests(unittest.TestCase):
    def test_join_keeps_missing_zero_priced_and_overpriced_scopes(self):
        source = {7: {'total': (1, 30, 30), '018000': (1, 30, 10), '028000': (1, 20, 20)}}
        native = {7: {'total': (1, 30, 30), '018000': (1, 30, 30), '038000': (1, 0, 0)}}
        result = compare(source, native)
        self.assertEqual(result['difference'], 0)
        self.assertEqual(result['routines'][0]['routine'], '028000')
        self.assertTrue(result['routines'][0]['no_native_scope'])
        self.assertEqual(result['routines'][-1]['exclusive_difference'], -20)

    def test_join_rejects_different_iterations(self):
        with self.assertRaisesRegex(ValueError, 'iteration sets differ'):
            compare({1: {'total': (1, 0, 0)}}, {})
        with self.assertRaisesRegex(ValueError, 'no complete iterations'):
            compare({}, {})

    def test_reader_conserves_totals_and_normalizes_rom_mirrors(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'profile.csv'
            path.write_text('iteration,routine,calls,inclusive,exclusive\n'
                            '7,818000,1,20,20\n7,vwf,0,10,10\n7,total,1,30,30\n')
            self.assertIn('018000', load(path, windows('7'))[7])
            path.write_text(path.read_text().replace('7,total,1,30,30', '7,total,1,31,31'))
            with self.assertRaisesRegex(ValueError, 'exclusive sum'):
                load(path, [])

    def test_windows_validate_ranges(self):
        self.assertEqual(windows('0,2-4'), [(0, 0), (2, 4)])
        with self.assertRaises(ValueError):
            windows('4-2')


if __name__ == '__main__':
    unittest.main()
