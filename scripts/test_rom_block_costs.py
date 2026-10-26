"""Generator contracts; ROM equality skips on machines without the cartridge."""
import json
import unittest
from pathlib import Path

import rom_block_costs as costs


class BlockCosts(unittest.TestCase):
    def test_checked_in_table_matches_rom(self):
        if not costs.DEFAULT_ROM.is_file():
            self.skipTest('ROM absent')
        rom = costs.DEFAULT_ROM.read_bytes()
        if len(rom) % 0x400 == 0x200:
            rom = rom[0x200:]
        entries = json.loads(Path('scripts/rom_block_entries.json').read_text())
        self.assertEqual(costs.emit_rust(rom, entries),
                         Path('crates/zelda3/src/rom_block_costs.rs').read_text())

    def fixture(self, code):
        rom = bytearray(0x8000)
        rom[:len(code)] = bytes(code)
        return costs.generated_blocks(rom, [dict(entry='008000', end=f'{0x8000 + len(code):06x}')])

    def test_branch_target_splits_block_and_retains_extra(self):
        blocks = self.fixture([0xd0, 1, 0xea, 0x60])
        self.assertEqual([[i.pc for i in b] for b in blocks], [[0x8000], [0x8002], [0x8003]])
        self.assertEqual(blocks[0][0].price()[:2], (16, 6))

    def test_calls_split_and_rep_changes_immediate_width(self):
        blocks = self.fixture([0x20, 0x00, 0x90, 0xc2, 0x20, 0xa9, 1, 2, 0x60])
        self.assertEqual(len(blocks), 2)
        self.assertEqual(blocks[1][1].length, 3)
        self.assertFalse(blocks[1][1].m8)

    def test_dynamic_width_and_conflicts_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'dynamic state'):
            self.fixture([0x28, 0x60])
        with self.assertRaisesRegex(ValueError, 'conflicting M/X'):
            self.fixture([0xd0, 2, 0xc2, 0x20, 0x60])

    def test_paired_status_restore_tracks_widths_and_rejects_stack_aliases(self):
        blocks = self.fixture([0x08, 0xc2, 0x20, 0x28, 0xa9, 1, 0x60])
        instructions = [i for b in blocks for i in b]
        self.assertFalse(instructions[2].m8)
        self.assertTrue(instructions[3].m8)
        self.assertEqual(instructions[3].length, 2)
        with self.assertRaisesRegex(ValueError, 'stack mutation'):
            self.fixture([0x08, 0x48, 0x28, 0x60])
        with self.assertRaisesRegex(ValueError, 'unbalanced saved status'):
            self.fixture([0x08, 0x60])

    def test_explicit_data_bank_prices_register_bus_and_rejects_conflicts(self):
        rom = bytearray(0x8000)
        rom[:4] = bytes([0xad, 0x17, 0x42, 0x60])
        spec = dict(entry='008000', end='008004')
        unknown = costs.generated_blocks(rom, [spec])[0][0]
        self.assertIn('data bank', unknown.price()[2])
        for bank, clocks in [(0x1e, 30), (0x7e, 32), (0x9e, 30)]:
            ins = costs.generated_blocks(rom, [dict(spec, data_bank=bank)])[0][0]
            self.assertEqual(ins.price(), (clocks, 0, ''))
        # A word straddling $5FFF/$6000 has one register and one RAM access.
        rom[:4] = bytes([0xad, 0xff, 0x5f, 0x60])
        wide = dict(spec, m8=False)
        unknown = costs.generated_blocks(rom, [wide])[0][0].price()[0]
        known = costs.generated_blocks(rom, [dict(wide, data_bank=0x1e)])[0][0]
        self.assertEqual(known.price(), (unknown - 2, 0, ''))
        with self.assertRaisesRegex(ValueError, 'conflicting data banks'):
            costs.generated_blocks(rom, [dict(spec, data_bank=0x1e), spec])
        with self.assertRaisesRegex(ValueError, 'invalid data bank'):
            costs.generated_blocks(rom, [dict(spec, data_bank=256)])

    def test_terminal_indirect_dispatch_requires_an_explicit_boundary(self):
        rom=bytearray(0x8000);rom[:3]=bytes([0xdc,0,0])
        spec=dict(entry='008000',end='008003')
        with self.assertRaisesRegex(ValueError,'explicit entries'):
            costs.generated_blocks(rom,[spec])
        blocks=costs.generated_blocks(rom,[dict(spec,terminal_dispatch=True)])
        self.assertEqual([i.name for b in blocks for i in b],['JML'])


if __name__ == '__main__':
    unittest.main()
