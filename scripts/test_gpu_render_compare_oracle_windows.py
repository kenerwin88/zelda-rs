#!/usr/bin/env python3
"""Tests for oracle-window render compare helpers."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
from contextlib import redirect_stdout
from io import StringIO

from gpu_render_compare_oracle_windows import (
    MODERN_INDEX_PROGRESS_RE,
    MODERN_INDEX_SUMMARY_RE,
    OracleCheckpoint,
    RunItem,
    OracleWindow,
    best_checkpoint_for,
    command_for,
    command_for_run_item,
    env_for_renderer,
    ensure_required_stable_draws,
    run_items_for_windows,
    run_command_capture_output,
    selected_windows,
)


class GpuRenderCompareOracleWindowsTests(unittest.TestCase):
    def test_capture_output_does_not_wait_for_stdout_inheriting_grandchild(self) -> None:
        code = (
            "import subprocess, sys; "
            "subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(2)']); "
            "print('done')"
        )

        started = time.monotonic()
        result = run_command_capture_output(
            [sys.executable, "-c", code],
            cwd=Path.cwd(),
            env=os.environ.copy(),
        )
        elapsed = time.monotonic() - started

        self.assertEqual(result.returncode, 0)
        self.assertIn("done", result.stdout)
        self.assertLess(elapsed, 1.5)

    def test_capture_output_can_stream_progress_from_tempfile(self) -> None:
        code = (
            "print('hidden noise', flush=True); "
            "print('modern_index_compare_progress compare_count=10 frame=20 bad_count=0', flush=True); "
            "print('done', flush=True)"
        )

        live = StringIO()
        with redirect_stdout(live):
            result = run_command_capture_output(
                [sys.executable, "-c", code],
                cwd=Path.cwd(),
                env=os.environ.copy(),
                live_patterns=(MODERN_INDEX_PROGRESS_RE,),
                poll_seconds=0.01,
            )

        self.assertEqual(result.returncode, 0)
        self.assertIn("hidden noise", result.stdout)
        self.assertIn("done", result.stdout)
        self.assertIn("modern_index_compare_progress compare_count=10", live.getvalue())
        self.assertNotIn("hidden noise", live.getvalue())

    def test_variant_gpu_env_enables_summary_and_progress(self) -> None:
        env = env_for_renderer(
            {"KEEP": "1"},
            renderer="assets-variant-gpu",
            progress_every=2500,
        )

        self.assertEqual(env["KEEP"], "1")
        self.assertEqual(env["ZELDA3_RENDERER"], "assets-variant-gpu")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"], "1")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_PROGRESS"], "2500")

    def test_progress_can_be_disabled(self) -> None:
        env = env_for_renderer(
            {},
            renderer="assets-variant-gpu",
            progress_every=0,
        )

        self.assertEqual(env["ZELDA3_RENDERER"], "assets-variant-gpu")
        self.assertEqual(env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"], "1")
        self.assertNotIn("ZELDA3_MODERN_INDEX_COMPARE_PROGRESS", env)

    def test_modern_index_summary_regex_accepts_modern_draw_mix(self) -> None:
        line = (
            "modern_index_compare_summary compare_count=7 bad_count=0 bad_pixels=0 "
            "gpu_count=5 mode7_gpu_count=1 cpu_count=1 variant_draws=11 "
            "fallback_draws=13 dynamic_palette_draws=17 missing_variant_draws=19 "
            "stable_preview_draws=2 stable_effect_draws=3 dynamic_material_draws=5 "
            "missing_art_draws=7 unkeyed_fallback_draws=11 mixed_overlay_bg_effect_draws=13 "
            "mixed_overlay_bg_effect_candidates=17 "
            "mixed_overlay_bg_effect_reject_complex_frame=19 "
            "mixed_overlay_bg_effect_reject_complex_brightness=23 "
            "mixed_overlay_bg_effect_reject_complex_invalid_layer=29 "
            "mixed_overlay_bg_effect_reject_complex_mosaic=31 "
            "mixed_overlay_bg_effect_reject_complex_sub_window=37 "
            "mixed_overlay_bg_effect_reject_complex_effect_bounds=41 "
            "mixed_overlay_bg_effect_reject_complex_scanline_main=43 "
            "mixed_overlay_bg_effect_reject_complex_layer_window=47 "
            "mixed_overlay_bg_effect_reject_complex_color_math=53 "
            "mixed_overlay_bg_effect_reject_complex_color_math_clip=57 "
            "mixed_overlay_bg_effect_reject_complex_color_math_subscreen=58 "
            "mixed_overlay_bg_effect_reject_complex_color_math_fixed_color=59 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch=61 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=67 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg=71 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj=73 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain=74 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front=75 "
            "mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order=76 "
            "mixed_overlay_bg_effect_reject_cgram_mismatch=79 "
            "mixed_overlay_bg_effect_reject_overlap=83"
        )

        match = MODERN_INDEX_SUMMARY_RE.search(line)

        self.assertIsNotNone(match)
        self.assertEqual(match.group(7), "11")
        self.assertEqual(match.group(11), "2")
        self.assertEqual(match.group(15), "11")
        self.assertEqual(match.group(16), "13")
        self.assertEqual(match.group(17), "17")
        self.assertEqual(match.group(18), "19")
        self.assertEqual(match.group(19), "23")
        self.assertEqual(match.group(20), "29")
        self.assertEqual(match.group(21), "31")
        self.assertEqual(match.group(26), "53")
        self.assertEqual(match.group(27), "57")
        self.assertEqual(match.group(28), "58")
        self.assertEqual(match.group(29), "59")
        self.assertEqual(match.group(30), "61")
        self.assertEqual(match.group(31), "67")
        self.assertEqual(match.group(32), "71")
        self.assertEqual(match.group(33), "73")
        self.assertEqual(match.group(34), "74")
        self.assertEqual(match.group(35), "75")
        self.assertEqual(match.group(36), "76")
        self.assertEqual(match.group(37), "79")
        self.assertEqual(match.group(38), "83")

    def test_modern_index_summary_regex_accepts_legacy_draw_mix(self) -> None:
        line = (
            "modern_index_compare_summary compare_count=7 bad_count=0 bad_pixels=0 "
            "gpu_count=5 mode7_gpu_count=1 cpu_count=1 variant_draws=11 "
            "fallback_draws=13 dynamic_palette_draws=17 missing_variant_draws=19"
        )

        match = MODERN_INDEX_SUMMARY_RE.search(line)

        self.assertIsNotNone(match)
        self.assertEqual(match.group(7), "11")
        self.assertIsNone(match.group(11))

    def test_no_input_windows_are_not_treated_as_sram_windows(self) -> None:
        windows = [
            OracleWindow(
                name="no-input-intro",
                status="pass",
                frames=1,
                input_script="",
                coverage="intro",
                notes="",
            )
        ]

        selected = selected_windows(
            windows,
            only=[],
            max_frames=None,
            include_sram_windows=False,
        )

        self.assertEqual([window.name for window in selected], ["no-input-intro"])

    def test_selected_windows_can_limit_after_filters(self) -> None:
        windows = [
            OracleWindow("first", "pass", 100, "", "", ""),
            OracleWindow("skip", "fail", 100, "", "", ""),
            OracleWindow("second", "pass", 100, "", "", ""),
        ]

        selected = selected_windows(
            windows,
            only=[],
            max_frames=None,
            include_sram_windows=False,
            limit=1,
        )

        self.assertEqual([window.name for window in selected], ["first"])

    def test_required_stable_draws_rejects_zero_source_art(self) -> None:
        with self.assertRaisesRegex(SystemExit, "stable source-art/effect draws"):
            ensure_required_stable_draws(
                stable_preview_draws=0,
                stable_effect_draws=0,
            )

    def test_required_stable_draws_accepts_preview_or_effect_art(self) -> None:
        ensure_required_stable_draws(stable_preview_draws=1, stable_effect_draws=0)
        ensure_required_stable_draws(stable_preview_draws=0, stable_effect_draws=1)

    def test_command_for_checkpoint_uses_tail_frames_and_load_state(self) -> None:
        window = OracleWindow(
            name="file-select-button-taps",
            status="pass",
            frames=112000,
            input_script="scripts/inputs/file-select-enter-game-button-taps.txt",
            coverage="buttons",
            notes="",
        )

        command = command_for(
            window,
            Path("/rom.sfc"),
            stride=1,
            release=True,
            renderer="assets-variant-gpu",
            frames=5000,
            load_state="target/lockstep-checkpoints/file-select.bin",
        )

        self.assertEqual(
            command[:8],
            [
                "cargo",
                "run",
                "--release",
                "-q",
                "-p",
                "zelda3-bin",
                "--",
                "--play-gpu-render-compare",
            ],
        )
        self.assertEqual(command[8:10], ["/rom.sfc", "5000"])
        self.assertIn("--modern-index-compare", command)
        self.assertIn("--load-state", command)
        self.assertEqual(
            command[command.index("--load-state") + 1],
            "target/lockstep-checkpoints/file-select.bin",
        )
        self.assertIn("--input-script", command)
        self.assertNotIn("--load-sram", command)

    def test_best_checkpoint_uses_newest_existing_matching_checkpoint(self) -> None:
        window = OracleWindow(
            name="route",
            status="pass",
            frames=200,
            input_script="scripts/inputs/route.txt",
            coverage="",
            notes="",
        )
        existing = Path("target/test-existing-checkpoint.sav")
        existing.parent.mkdir(parents=True, exist_ok=True)
        existing.write_bytes(b"state")
        self.addCleanup(lambda: existing.unlink(missing_ok=True))
        checkpoints = [
            OracleCheckpoint(
                "route", 50, "target/missing.sav", "scripts/inputs/route.txt", "a", ""
            ),
            OracleCheckpoint(
                "route", 150, str(existing), "scripts/inputs/route.txt", "b", ""
            ),
            OracleCheckpoint(
                "route", 175, str(existing), "scripts/inputs/other.txt", "c", ""
            ),
            OracleCheckpoint(
                "route", 250, str(existing), "scripts/inputs/route.txt", "d", ""
            ),
        ]

        checkpoint = best_checkpoint_for(window, checkpoints)

        self.assertIsNotNone(checkpoint)
        self.assertEqual(checkpoint.frame, 150)

    def test_run_items_preserve_window_order_and_checkpoint_selection(self) -> None:
        first = OracleWindow("first", "pass", 100, "scripts/inputs/first.txt", "", "")
        second = OracleWindow("second", "pass", 200, "scripts/inputs/second.txt", "", "")
        checkpoint_path = Path("target/test-run-items-checkpoint.sav")
        checkpoint_path.parent.mkdir(parents=True, exist_ok=True)
        checkpoint_path.write_bytes(b"state")
        self.addCleanup(lambda: checkpoint_path.unlink(missing_ok=True))
        checkpoints = {
            "second": [
                OracleCheckpoint(
                    "second",
                    175,
                    str(checkpoint_path),
                    "scripts/inputs/second.txt",
                    "digest",
                    "",
                )
            ]
        }

        items = run_items_for_windows([first, second], checkpoints, fast=True)

        self.assertEqual([item.window.name for item in items], ["first", "second"])
        self.assertIsNone(items[0].checkpoint)
        self.assertIsNotNone(items[1].checkpoint)
        self.assertEqual(items[1].tail_frames, 25)

    def test_run_items_can_cap_tail_frames_for_short_proofs(self) -> None:
        window = OracleWindow("short-proof", "pass", 5000, "scripts/inputs/short.txt", "", "")

        items = run_items_for_windows(
            [window],
            checkpoints_by_name={},
            fast=False,
            frame_limit=120,
        )

        self.assertEqual(items[0].tail_frames, 120)

    def test_command_for_run_item_uses_capped_tail_frames(self) -> None:
        window = OracleWindow("short-proof", "pass", 5000, "scripts/inputs/short.txt", "", "")
        item = RunItem(window=window, checkpoint=None, tail_frames=120)

        command = command_for_run_item(
            item,
            Path("/rom.sfc"),
            stride=1,
            release=True,
            renderer="assets-variant-gpu",
        )

        self.assertEqual(command[8:10], ["/rom.sfc", "120"])


if __name__ == "__main__":
    unittest.main()
