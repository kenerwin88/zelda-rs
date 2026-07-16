#!/usr/bin/env python3

import tempfile
import unittest
from pathlib import Path

from scripts.input_script_tools import numeric_input_history, numeric_input_script, parse_buttons


class InputScriptToolsTests(unittest.TestCase):
    def test_named_buttons_match_snes_serial_bits(self) -> None:
        self.assertEqual(parse_buttons("START"), 0x0008)
        self.assertEqual(parse_buttons("A+RIGHT"), 0x0180)
        self.assertEqual(parse_buttons("B,Y|SELECT"), 0x0007)
        self.assertEqual(parse_buttons("NONE"), 0)

    def test_numeric_script_expands_includes_and_preserves_override_order(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "base.txt").write_text("10 START\n12 NONE\n", encoding="utf-8")
            (root / "route.txt").write_text(
                "include base.txt\n12 A+RIGHT\n20..22 LEFT\n", encoding="utf-8"
            )
            self.assertEqual(
                numeric_input_script(root / "route.txt"),
                "10 0x0008\n12 0x0000\n12 0x0180\n20..22 0x0040\n",
            )

    def test_numeric_history_compresses_runs_and_omits_idle_input(self) -> None:
        self.assertEqual(
            numeric_input_history(
                [(0, 0), (1, 0x80), (2, 0x80), (3, 0), (4, 0x100)]
            ),
            "# Deterministic controller stream captured once per game frame.\n"
            "1..2 0x0080\n"
            "4 0x0100\n",
        )


if __name__ == "__main__":
    unittest.main()
