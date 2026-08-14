#!/usr/bin/env python3

import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "snes9x_dma_receipt.py"
SPEC = importlib.util.spec_from_file_location("snes9x_dma_receipt", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
RECEIPT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RECEIPT)


class Snes9xDmaReceiptTests(unittest.TestCase):
    def test_maps_absolute_host_frame_to_checkpoint_run(self) -> None:
        events = [
            {
                "event": "frame",
                "stage": "entry",
                "run": 4,
                "frame": 12,
                "v": 225,
                "cycles": 2,
                "pc": 0x008036,
                "main": 7,
                "sub": 0x12,
                "subsub": 15,
                "frame_counter": 0xcc,
                "nmi_latch": 0,
            },
            {
                "event": "nmi",
                "run": 4,
                "frame": 12,
                "v": 225,
                "cycles": 90,
                "pc": 0x0080C9,
                "nmi_latch": 0,
                "nmi_disable": 0,
                "nmi_pending": 1,
            },
            {
                "event": "dma",
                "run": 4,
                "frame": 12,
                "v": 227,
                "cycles": 1136,
                "pc": 0x008a5c,
                "channel": 0,
                "source": 0x108840,
                "b_address": 0x18,
                "bytes": 64,
                "mode": 1,
                "vram_address": 0x4000,
            },
            {
                "event": "frame",
                "stage": "return",
                "run": 4,
                "frame": 13,
                "v": 225,
                "cycles": 4,
                "pc": 0x00ea20,
                "frame_counter": 0xcd,
                "nmi_latch": 1,
            },
        ]
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace.jsonl"
            trace.write_text("".join(f"{__import__('json').dumps(event)}\n" for event in events))
            report = RECEIPT.summarize(trace, host_frame=29_014, resume_frame=29_010)

        self.assertIn("host frame 29014 = trace run 4", report)
        self.assertIn("internal frames 12->13", report)
        self.assertIn("NMI entries: 1", report)
        self.assertIn("pc=$00:80c9 nmi_latch=0", report)
        self.assertIn("src=$10:8840", report)
        self.assertIn("dst_word=$4000", report)


if __name__ == "__main__":
    unittest.main()
