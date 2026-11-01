import unittest
import tempfile
from pathlib import Path
from pricing_coverage import census, native_touch, partition, source_rows, scalar_touch, verify_source_totals


class CoverageTests(unittest.TestCase):
    def test_missing_source_instruction_cannot_conserve_by_construction(self):
        rows = {1: {(0x8000, 0x8000, 1, 1): (1, 14)}}
        verify_source_totals(rows, {1: 14})
        with self.assertRaises(ValueError):
            verify_source_totals(rows, {1: 28})
        with self.assertRaises(ValueError):
            verify_source_totals(rows, {})

    def test_scalar_footprint_requires_native_site_scope_and_contract(self):
        native = {'scalar_sites': [([0x841e], 'site', [5300, 1])]}
        catalog = {'site': (0x841e, 5300, [(0x841e, True, True)])}
        touched, clocks = scalar_touch(native, catalog)
        self.assertEqual(clocks, 5300)
        rows, _ = partition({(0x841e, 0x841e, 1, 1): (1, 16)}, set(), set(), 100, touched)
        self.assertEqual(rows, {'scalar_touched': 16})
        native['scalar_sites'][0][0][0] = 0x9000
        with self.assertRaises(ValueError):
            scalar_touch(native, catalog)

    def test_overlapping_witnesses_must_agree(self):
        with tempfile.TemporaryDirectory() as directory:
            a, b = (Path(directory) / name for name in ('a.csv', 'b.csv'))
            header = 'iteration,routine,pc,m8,x8,count,master\n'
            a.write_text(header + '1,008034,008038,1,1,1,14\n')
            b.write_text(a.read_text())
            self.assertEqual(len(source_rows([a, b], {1})), 1)
            b.write_text(header + '1,008034,008038,1,1,1,16\n')
            with self.assertRaises(ValueError):
                source_rows([a, b], {1})
            b.write_text(a.read_text() + '1,008034,008038,1,1,1,14\n')
            with self.assertRaises(ValueError):
                source_rows([b], {1})

    def test_opaque_scalar_is_not_free_work(self):
        source = {(0x8000, 0x8001, 1, 1): (2, 28), (0x9000, 0x9001, 1, 1): (1, 14)}
        result, owners = partition(source, {(0x8001, True, True)}, set(), 1)
        self.assertEqual(result, dict(marker_touched=28, unresolved_alias=14))
        self.assertEqual(owners[0x9000]['uncovered'], 0)
        result, _ = partition(source, set(), {0x8001}, 0)
        self.assertEqual(result, dict(model_touched=28, uncovered=14))

    def test_scalar_and_model_conservation_and_missing_rows(self):
        native = dict(iteration=1, ledger_master=30, blocks=[], model_sites=[([], 'model', [20, 1])],
                      model_steps=[([], 'model', 0x9000, False, [15, 1])], scalar_sites=[([], 'scalar', [10, 1])])
        self.assertEqual(native_touch(native, {}), (set(), {0x9000}, 15))
        report = census({1: {(0x9000, 0x9000, 1, 1): (1, 15)}}, [native], {1, 2}, {})
        self.assertEqual(report['missing_source'], [2])
        self.assertEqual(report['missing_native'], [2])
        self.assertEqual(report['iterations'][0]['signed_difference'], -15)
        prediction = {1: {'vwf_master': '6', 'main_master': '36'}}
        report = census({1: {(0x9000, 0x9000, 1, 1): (1, 15)}}, [native], {1}, {}, prediction)
        self.assertEqual(report['native_main'], 36)
        self.assertEqual(report['iterations'][0]['opaque_native'], 21)
        prediction[1]['main_master'] = '37'
        with self.assertRaises(ValueError):
            census({1: {}}, [native], {1}, {}, prediction)
        with self.assertRaises(ValueError):
            census({}, [native, native], {1}, {})
        native['ledger_master'] += 1
        with self.assertRaises(ValueError):
            native_touch(native, {})

    def test_marker_width_identity(self):
        native = dict(iteration=1, ledger_master=14, blocks=[([], 0x8000, 0, [True, True], [14, 1])],
                      model_sites=[], model_steps=[], scalar_sites=[])
        self.assertEqual(native_touch(native, {0x8000: [(0x8000, True, True)]})[0], {(0x8000, True, True)})
        with self.assertRaises(ValueError):
            native_touch(native, {0x8000: [(0x8000, False, True)]})


if __name__ == '__main__':
    unittest.main()
