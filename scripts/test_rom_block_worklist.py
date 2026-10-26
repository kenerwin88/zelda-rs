import unittest
from rom_block_worklist import rank


class Worklist(unittest.TestCase):
    def test_width_convergence_keeps_shared_tail_separate(self):
        rom = bytes([0xc2, 0x30, 0x60]) + bytes(0x8000 - 3)
        receipts = [dict(iteration='7', routine='008000', pc=pc,
                         m8=m8, x8=x8, count=str(count), master=str(master), entries='0')
                    for pc, m8, x8, count, master in [
                        ('008000', '1', '1', 1, 22),
                        ('008000', '0', '0', 1, 22),
                        ('008002', '0', '0', 2, 84)]]
        routines = {7: {'008000': (2, 128, 128), 'total': (1, 128, 128)}}
        rows = rank(rom, receipts, routines, {7}, set(), None)
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0]['block'], '008002')
        self.assertEqual(rows[0]['source_master'], 84)
        self.assertEqual(sum(r['source_master'] for r in rows), 128)
        self.assertTrue(all(r['end'] == '008002' for r in rows[1:]))

    def test_executed_branch_fragments_and_native_owner(self):
        rom = bytes([0xd0, 1, 0xea, 0x60]) + bytes(0x8000 - 4)
        receipts = [dict(iteration='7', routine='008000', pc=pc, m8='1', x8='1', count=str(count), master=str(master), entries='0')
                    for pc, count, master in [('008000', 2, 38), ('008002', 1, 14), ('008003', 2, 84)]]
        routines = {7: {'008000': (2, 136, 136), 'total': (1, 136, 136)}}
        rows = rank(rom, receipts, routines, {7}, {0x8000},
                    {(0x8000, 0x8000, 0): 32, (0x8000, 0x8000, 1): 6,
                     (0x9000, 0x8000, 0): 100})
        self.assertEqual([r['block'] for r in rows], ['008003', '008000', '008002'])
        self.assertEqual(rows[1]['native_marker_master'], 38)
        self.assertTrue(rows[1]['marker_declared'])
        self.assertEqual(sum(r['source_master'] for r in rows), 136)
        with self.assertRaisesRegex(ValueError, 'reconcile'):
            rank(rom, receipts[:-1], routines, {7}, set(), None)
        with self.assertRaisesRegex(ValueError, 'complete frozen cohort'):
            rank(rom, receipts, routines, {7, 8}, set(), None)
        # An observed indirect jump can enter the middle of a contiguous ROM
        # range without a statically known target. Its dynamic entry splits it.
        image = bytearray(0x8000)
        image[:3] = bytes([0xea, 0xea, 0x60])
        image[0x10:0x13] = bytes([0x6c, 0x00, 0x90])
        indirect = [dict(iteration='7', routine='008000', pc=pc, m8='1', x8='1',
                         count=str(count), master=str(master), entries=str(entries))
                    for pc, count, master, entries in [
                        ('008000', 1, 14, 1), ('008001', 2, 28, 1),
                        ('008002', 2, 84, 0), ('008010', 1, 40, 1)]]
        rows = rank(image, indirect, {7: {'008000': (2, 166, 166), 'total': (1, 166, 166)}},
                    {7}, set(), None)
        self.assertEqual([r['block'] for r in rows], ['008001', '008010', '008000'])
        self.assertEqual(rows[0]['source_master'], 112)
        # One native address-only receipt cannot be attributed to BOTH source
        # entry-width variants, even when their instruction costs happen to agree.
        receipts = [dict(iteration='7', routine='008000', pc='008000', m8=m8,
                         x8='1', count='1', master='42', entries='1') for m8 in ['0', '1']]
        routines = {7: {'008000': (2, 84, 84), 'total': (1, 84, 84)}}
        rows = rank(bytes([0x60]) + bytes(0x7fff), receipts, routines, {7}, {0x8000},
                    {(0x8000, 0x8000, 0): 42})
        self.assertEqual(len(rows), 2)
        self.assertTrue(all(r['native_width_ambiguous'] for r in rows))
        self.assertTrue(all(r['native_marker_master'] is None for r in rows))


if __name__ == '__main__':
    unittest.main()
