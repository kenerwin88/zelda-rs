#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "test_standard_replay_parity.py"
SPEC = importlib.util.spec_from_file_location("standard_replay_parity", SCRIPT)
parity = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(parity)


class StandardReplayParityHelperTests(unittest.TestCase):
    def test_menu_render_hash_samples_cover_transition_and_final_frame(self) -> None:
        self.assertEqual(parity.MENU_RENDER_HASH_FRAMES[0], 356_446)
        self.assertIn(356_458, parity.MENU_RENDER_HASH_FRAMES)
        self.assertEqual(parity.MENU_RENDER_HASH_FRAMES[-1], parity.MENU_DISPLAY_FRAME)
        self.assertEqual(len(set(parity.MENU_RENDER_HASH_FRAMES)), 9)

    def test_inventory_gate_requires_a_fully_open_equipment_menu(self) -> None:
        valid = (
            "replay-save completed frames=356535 main=14 sub=1 map=4 "
            "bg3v=0xff18 saved=9\n"
        )
        self.assertEqual(parity.inventory_menu_state_failures(valid, "Rust"), [])

        dialogue = (
            "replay-save completed frames=700090 main=14 sub=2 map=0 "
            "bg3v=0x0000 saved=7\n"
        )
        failures = parity.inventory_menu_state_failures(dialogue, "Rust")
        self.assertTrue(any("sub expected=1" in failure for failure in failures))
        self.assertTrue(any("map expected=4" in failure for failure in failures))
        self.assertTrue(any("bg3v expected=65304" in failure for failure in failures))

    def test_title_palette_gate_rejects_red_subtitle(self) -> None:
        width, height, channels = 256, 224, 4
        pixels = bytearray(width * height * channels)
        for index, (x, y) in enumerate(
            (x, y) for y in range(135, 148) for x in range(90, 210)
        ):
            if index >= 224:
                break
            offset = (y * width + x) * channels
            pixels[offset : offset + 4] = bytes((255, 255, 255, 255))
        self.assertIsNone(
            parity.title_subtitle_palette_failure(
                "Rust", width, height, bytes(pixels), channels
            )
        )

        for y in range(135, 148):
            for x in range(90, 210):
                offset = (y * width + x) * channels
                pixels[offset : offset + 4] = bytes((173, 24, 24, 255))
        failure = parity.title_subtitle_palette_failure(
            "Rust", width, height, bytes(pixels), channels
        )
        self.assertIsNotNone(failure)
        self.assertIn("expected white text", failure)

    def test_menu_vertical_coverage_rejects_an_upper_half_only_panel(self) -> None:
        width, height = 256, 224
        pixels = bytearray(width * height * 4)
        for y in range(height // 2):
            for x in range(width):
                offset = (y * width + x) * 4
                pixels[offset : offset + 4] = bytes((40, 80, 40, 255))
        failure = parity.menu_vertical_coverage_failure(
            "live GPU", width, height, bytes(pixels)
        )
        self.assertIsNotNone(failure)
        self.assertIn("did not extend through the lower half", failure)

        for y in range(height // 2, height):
            for x in range(100):
                offset = (y * width + x) * 4
                pixels[offset : offset + 4] = bytes((40, 80, 40, 255))
        self.assertIsNone(
            parity.menu_vertical_coverage_failure(
                "live GPU", width, height, bytes(pixels)
            )
        )

    def test_inventory_panel_gate_requires_both_lower_panel_interiors(self) -> None:
        width, height = 256, 224
        pixels = bytearray(width * height * 4)
        for y in range(height):
            for x in range(width):
                offset = (y * width + x) * 4
                pixels[offset : offset + 4] = bytes((45, 95, 55, 255))
        failure = parity.inventory_menu_panel_geometry_failure(
            "live GPU", width, height, bytes(pixels)
        )
        self.assertIsNotNone(failure)
        self.assertIn("lower panels are missing or clipped", failure)

        for left, top, right, bottom in ((16, 152, 152, 208), (176, 152, 240, 208)):
            for y in range(top, bottom):
                for x in range(left, right):
                    offset = (y * width + x) * 4
                    pixels[offset : offset + 4] = bytes((0, 0, 0, 255))
        self.assertIsNone(
            parity.inventory_menu_panel_geometry_failure(
                "live GPU", width, height, bytes(pixels)
            )
        )


if __name__ == "__main__":
    unittest.main()
