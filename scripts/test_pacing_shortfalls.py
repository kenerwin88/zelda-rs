import unittest

from pacing_shortfalls import FIELD, band, summarize


class ShortfallsTest(unittest.TestCase):
    def test_strict_band_boundaries(self):
        self.assertEqual(band(FIELD // 20), 'near-miss')
        self.assertEqual(band(FIELD // 20 + 1), 'moderate')
        self.assertEqual(band(FIELD // 2), 'large')

    def test_only_missing_holds_contribute_and_frames_are_not_iterations(self):
        def row(index, lag, holds, shortfall):
            return dict(iteration=str(index), module='07', submodule='00',
                        lag_nmis=str(lag), cost_p2=str(holds), native_held=str(holds),
                        cost_elapsed_p2=str(holds * FIELD), cost_shortfall=str(shortfall))
        missing, report = summarize([row(0, 3, 1, 200000), row(1, 1, 1, 0),
                                     row(2, 0, 0, 0), row(3, 1, 0, 10)])
        self.assertEqual(set(missing), {0, 3})
        self.assertEqual(report['total'], dict(iterations=2, missing=3, shortfall_master=200010))

    def test_stale_inverse_model_is_rejected(self):
        with self.assertRaises(ValueError):
            summarize([dict(iteration='4', lag_nmis='1', cost_p2='0',
                            native_held='0', cost_elapsed_p2=str(FIELD))])


if __name__ == '__main__':
    unittest.main()
