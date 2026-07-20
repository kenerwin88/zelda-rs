#!/usr/bin/env python3
"""Focused regression tests for the Snes9x boot-contract extractor."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("snes9x_boot_contract.py")
SPEC = importlib.util.spec_from_file_location("snes9x_boot_contract", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class BootContractTests(unittest.TestCase):
    def test_keeps_event_order_and_attaches_exact_decomp_symbols(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            trace = root / "trace.txt"
            trace.write_text(
                "82 state-before 00 00 01 80 00 00 00 00\n"
                "82 wram 0012 01 008b20\n"
                "82 dma 0 7e:0000 18 1024 1 0 0 0\n"
                "82 dma_pc 008b67\n"
                "82 oam 0000 55 dma=1 channel=0\n"
                "82 delta 0001 00 80\\n82 state-after 00 00 00 80 00 00 00 00\n"
            )
            symbols = root / "symbols.json"
            symbols.write_text(json.dumps([{"address": 0x12, "rust_name": "NMI_BOOLEAN", "source_label": "NMIFLG", "source_path": "us_asm/zel_ram.asm", "source_line": 25, "subsystem": "shared"}]))

            contract = MODULE.parse_trace(trace, MODULE.load_symbols(symbols), None)

        events = contract["frames"][0]["events"]
        self.assertEqual([event["kind"] for event in events], ["state", "wram_write", "dma", "oam_write", "state"])
        self.assertEqual(events[1]["symbols"][0]["rust"], "NMI_BOOLEAN")
        self.assertEqual(events[2]["pc"], "008b67")
        self.assertEqual(MODULE.validate_contract(contract), [])

    def test_rejects_a_dma_without_its_rom_pc(self) -> None:
        contract = {"frames": [{"frame": 9, "events": [{"kind": "state", "stage": "before"}, {"kind": "dma", "pc": None}, {"kind": "state", "stage": "after"}]}]}
        self.assertEqual(MODULE.validate_contract(contract), ["frame 9: DMA lacks its ROM program counter"])


if __name__ == "__main__":
    unittest.main()
