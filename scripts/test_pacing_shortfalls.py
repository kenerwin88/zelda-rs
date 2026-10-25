import unittest

from pacing_shortfalls import FIELD, band, summarize, structural_costs, structural_summary


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

    def test_refresh_work_and_hdma_do_not_become_a_rounding_error(self):
        # Three refresh events accompany 2,000 CPU clocks plus 1,000 HDMA
        # clocks; native priced only 1,000 CPU clocks and one refresh event.
        source = dict(iteration=1, main_wall=3120, instructions=2000,
                      refresh_master=120, dma_master=0, interrupt_refresh_edges=0)
        predictor = dict(main_master='1000', vwf_master='1000', cost_refresh_main='40')
        row = structural_costs(source, predictor)
        self.assertEqual(row['refresh_gap'], 80)
        self.assertEqual(row['refresh_work_difference'], 40)
        self.assertEqual(row['refresh_hdma_difference'], 40)
        self.assertEqual(row['refresh_placement_upper'], 0)
        self.assertEqual(row['refresh_native_report_error'], 0)
        # main_master already contains VWF. Adding vwf_master again would
        # invent a second native refresh event in this control.

    def test_interrupt_edge_bounds_and_overlapping_declines_remain_visible(self):
        source = dict(iteration=2, main_wall=1440, instructions=1300,
                      refresh_master=40, dma_master=0, interrupt_refresh_edges=1)
        predictor = dict(category='07/00', main_master='1300', cost_refresh_main='40',
                         cost_hdma_decline=str(1 | 64))
        row = structural_costs(source, predictor)
        self.assertEqual((row['hdma_lower_bound'], row['hdma_upper_bound']), (100, 140))
        summary = structural_summary({2: row}, {2: predictor})
        self.assertEqual(summary['totals']['iterations'], 1)
        self.assertEqual(summary['hdma_decline_signatures']['vwf+program-write']['iterations'], 1)
        self.assertEqual(set(summary['hdma_decline_reasons']), {'vwf', 'program-write'})
        predictor['cost_hdma_decline'] = str(1 << 10)
        summary = structural_summary({2: row}, {2: predictor})
        self.assertEqual(summary['hdma_decline_signatures']['nmi-over-hdma-init']['iterations'], 1)
        predictor['cost_hdma_decline'] = str(1 << 11)
        with self.assertRaises(ValueError):
            structural_summary({2: row}, {2: predictor})


if __name__ == '__main__':
    unittest.main()
