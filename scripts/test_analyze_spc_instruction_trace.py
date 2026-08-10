#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))
SPEC = importlib.util.spec_from_file_location(
    "analyze_spc_instruction_trace",
    SCRIPT_DIR / "analyze_spc_instruction_trace.py",
)
analyzer = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = analyzer
SPEC.loader.exec_module(analyzer)


def rust(pc: int, *, remaining: int, a: int = 0) -> dict:
    return {
        "pc": pc,
        "opcode": pc & 0xFF,
        "a": a,
        "x": 1,
        "y": 2,
        "sp": 3,
        "timer0_cycles": remaining,
        "timer0_divider": 4,
        "direct_page_0_3": [0, 0, 0, 0],
        "direct_page_8_11": [0, 0, 0, 0],
    }


def oracle(pc: int, *, elapsed: int, a: int = 0) -> dict:
    return {
        "program_counter": pc,
        "opcode": pc & 0xFF,
        "a": a,
        "x": 1,
        "y": 2,
        "stack_pointer": 3,
        "timer0_stage1": elapsed,
        "timer0_stage2": 4,
        "direct_page_0_11": [0] * 12,
    }


class AnalyzeSpcInstructionTraceTests(unittest.TestCase):
    def test_alignment_ignores_different_frame_boundary_instruction(self) -> None:
        receipt = {
            "rust_spc_instruction_trace": [
                0,
                [
                    rust(0x1000, remaining=110),
                    rust(0x1001, remaining=106),
                    rust(0x1002, remaining=102),
                ],
            ],
            "oracle_smp_instructions": [
                oracle(0x0FFF, elapsed=14),
                oracle(0x1000, elapsed=18),
                oracle(0x1001, elapsed=22),
                oracle(0x1002, elapsed=26),
                oracle(0x1003, elapsed=30),
            ],
        }

        summary = analyzer.summarize(receipt)

        self.assertEqual(summary["aligned_count"], 3)
        self.assertEqual(summary["phase_histogram"], {0: 3})
        self.assertEqual(summary["phase_runs"], [(0, 3, 0)])

    def test_reports_first_clock_delta_without_index_noise(self) -> None:
        receipt = {
            "rust_spc_instruction_trace": [
                0,
                [
                    rust(0x1000, remaining=110),
                    rust(0x1001, remaining=105),
                    rust(0x1002, remaining=101),
                ],
            ],
            "oracle_smp_instructions": [
                oracle(0x1000, elapsed=18),
                oracle(0x1001, elapsed=22),
                oracle(0x1002, elapsed=26),
            ],
        }

        summary = analyzer.summarize(receipt)

        self.assertEqual(summary["phase_histogram"], {0: 1, 1: 2})
        self.assertEqual(summary["phase_runs"], [(0, 1, 0), (1, 3, 1)])

    def test_reports_first_control_flow_gap_with_direct_page_witness(self) -> None:
        receipt = {
            "rust_spc_instruction_trace": [
                0,
                [rust(0x1000, remaining=110), rust(0x1001, remaining=106), rust(0x2000, remaining=102)],
            ],
            "oracle_smp_instructions": [
                oracle(0x1000, elapsed=18), oracle(0x2000, elapsed=26),
            ],
        }

        divergence = analyzer.first_instruction_divergence(receipt)

        self.assertIsNotNone(divergence)
        self.assertEqual(divergence["tag"], "delete")
        self.assertEqual(divergence["rust_range"], (1, 2))
        self.assertEqual(divergence["oracle_range"], (1, 1))
        self.assertEqual(divergence["rust_direct_page"], [0] * 8)

    def test_reports_duplicate_dsp_write_across_receipts(self) -> None:
        def receipt(rust_writes: list[tuple[int, int]], oracle_writes: list[tuple[int, int]]) -> dict:
            return {
                "rust_audio_event_frame": {
                    "events": [
                        {"parity_dsp": {"addr": address, "value": value}}
                        for address, value in rust_writes
                    ]
                },
                "oracle_dsp_register_writes": [
                    {"register": address, "value": value}
                    for address, value in oracle_writes
                ],
            }

        divergence = analyzer.first_dsp_content_divergence(
            [receipt([(0x5C, 0x20), (0x5C, 0x20)], [(0x5C, 0x20)])]
        )

        self.assertIsNotNone(divergence)
        self.assertEqual(divergence["tag"], "delete")
        self.assertEqual(divergence["rust"], [(0x5C, 0x20)])


if __name__ == "__main__":
    unittest.main()
