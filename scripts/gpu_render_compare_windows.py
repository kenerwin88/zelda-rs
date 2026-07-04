#!/usr/bin/env python3
"""Run checkpointed CPU/GPU render parity windows over a replay-save route."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ROM = Path(os.environ.get("ZELDA3_ROM", str(REPO_ROOT.parent / "zelda3" / "zelda3.sfc")))
DEFAULT_SAVE = REPO_ROOT / "saves" / "zelda3-combined-route.sav"
DEFAULT_CHECKPOINT_DIR = REPO_ROOT / "target" / "gpu-render-checkpoints"
COMPARE_RE = re.compile(
    r"gpu-render-compare completed compared=(\d+) last_frame=(\d+) "
    r"last_hash=(0x[0-9a-fA-F]{8}) mismatched_pixels=(\d+)"
)
MODERN_INDEX_SUMMARY_RE = re.compile(
    r"modern_index_compare_summary compare_count=(\d+) bad_count=(\d+) bad_pixels=(\d+) "
    r"gpu_count=(\d+) mode7_gpu_count=(\d+) cpu_count=(\d+)"
    r"(?: variant_draws=(\d+)(?: fallback_draws=(\d+))? dynamic_palette_draws=(\d+) missing_variant_draws=(\d+)"
    r"(?: stable_preview_draws=(\d+) stable_effect_draws=(\d+) dynamic_material_draws=(\d+) "
    r"(?:unsupported_material_draws=\d+ )?"
    r"missing_art_draws=(\d+) unkeyed_fallback_draws=(\d+)"
    r"(?: unkeyed_bg_fallback_draws=\d+ unkeyed_sprite_fallback_draws=\d+)?"
    r"(?: mixed_overlay_bg_effect_draws=(\d+)"
    r"(?: mixed_overlay_bg_effect_candidates=(\d+) "
    r"(?:mixed_overlay_bg_effect_culled_invisible_main=\d+ )?"
    r"mixed_overlay_bg_effect_reject_complex_frame=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_brightness=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_invalid_layer=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_mosaic=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_sub_window=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_effect_bounds=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_scanline_main=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_layer_window=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_clip=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_subscreen=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_fixed_color=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch=(\d+) )?)?)?)?)?)?"
    r"mixed_overlay_bg_effect_reject_cgram_mismatch=(\d+) "
    r"mixed_overlay_bg_effect_reject_overlap=(\d+))?)?)?)?"
)
MODERN_INDEX_VARIANT_RE = re.compile(
    r"modern_index_compare frame=(\d+) .* via=variant-gpu "
    r"variant_draws=(\d+)(?: fallback_draws=(\d+))? dynamic_palette_draws=(\d+) missing_variant_draws=(\d+)"
    r"(?: stable_preview_draws=(\d+) stable_effect_draws=(\d+) dynamic_material_draws=(\d+) "
    r"(?:unsupported_material_draws=\d+ )?"
    r"missing_art_draws=(\d+) unkeyed_fallback_draws=(\d+)"
    r"(?: unkeyed_bg_fallback_draws=\d+ unkeyed_sprite_fallback_draws=\d+)?"
    r"(?: mixed_overlay_bg_effect_draws=(\d+)"
    r"(?: mixed_overlay_bg_effect_candidates=(\d+) "
    r"(?:mixed_overlay_bg_effect_culled_invisible_main=\d+ )?"
    r"mixed_overlay_bg_effect_reject_complex_frame=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_brightness=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_invalid_layer=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_mosaic=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_sub_window=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_effect_bounds=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_scanline_main=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_layer_window=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_clip=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_subscreen=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_fixed_color=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order=(\d+) "
    r"(?:mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex=(\d+) "
    r"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch=(\d+) )?)?)?)?)?)?"
    r"mixed_overlay_bg_effect_reject_cgram_mismatch=(\d+) "
    r"mixed_overlay_bg_effect_reject_overlap=(\d+))?)?"
)
SAVED_RE = re.compile(r"saved replay-save checkpoint frame=(\d+) to (.+)")
SUMMARY_PREFIXES = (
    "gpu-render-compare completed ",
    "modern_index_compare_summary ",
    "saved replay-save checkpoint ",
    "gpu-render-window-compare completed ",
)


def int_stat(text: str, name: str) -> int:
    match = re.search(rf"\b{re.escape(name)}=(\d+)", text)
    return int(match.group(1)) if match else 0


def ensure_no_unsupported_material_draws(unsupported_material_draws: int) -> None:
    if unsupported_material_draws != 0:
        raise SystemExit(
            "variant GPU proof hit unsupported runtime material fallback draws: "
            f"{unsupported_material_draws}"
        )


def print_success_summary(output: str) -> None:
    for line in output.splitlines():
        if line.startswith(SUMMARY_PREFIXES):
            print(line)


def run(command: list[str], *, renderer: str | None = None) -> str:
    prefix = f"ZELDA3_RENDERER={renderer} " if renderer else ""
    print("+ " + prefix + " ".join(command), flush=True)
    env = os.environ.copy()
    if renderer:
        env["ZELDA3_RENDERER"] = renderer
    if renderer == "assets-variant-gpu":
        env["ZELDA3_MODERN_INDEX_COMPARE_SUMMARY"] = "1"
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, end="" if result.stdout.endswith("\n") else "\n")
        raise SystemExit(result.returncode)
    print_success_summary(result.stdout)
    return result.stdout


def cargo_prefix(release: bool) -> list[str]:
    command = ["cargo", "run"]
    if release:
        command.append("--release")
    command.extend(["-q", "-p", "zelda3-bin", "--"])
    return command


def checkpoint_path(checkpoint_dir: Path, frame: int) -> Path:
    return checkpoint_dir / f"rust-frame-{frame:06d}.sav"


def replay_command(
    *,
    rom: Path,
    save: Path,
    frames: int,
    release: bool,
    load_state: Path | None = None,
    save_state: Path | None = None,
    compare_stride: int | None = None,
    compare_mode: str = "gpu-render",
) -> list[str]:
    command = [
        *cargo_prefix(release),
        "--replay-save",
        str(rom),
        str(save),
        str(frames),
    ]
    if load_state is not None:
        command.extend(["--load-state", str(load_state)])
    if save_state is not None:
        command.extend(["--save-state", str(save_state)])
    if compare_stride is not None:
        if compare_mode == "modern-index":
            command.extend(["--modern-index-compare", str(compare_stride)])
        else:
            command.extend(
                [
                    "--gpu-render-compare",
                    str(compare_stride),
                    "--gpu-render-compare-quiet",
                ]
            )
    return command


def nearest_checkpoint(checkpoint_dir: Path, frame: int) -> tuple[int, Path] | None:
    usable: list[tuple[int, Path]] = []
    for path in checkpoint_dir.glob("rust-frame-*.sav"):
        try:
            checkpoint_frame = int(path.stem.rsplit("-", 1)[1])
        except (IndexError, ValueError):
            continue
        if checkpoint_frame <= frame:
            usable.append((checkpoint_frame, path))
    if not usable:
        return None
    return max(usable, key=lambda item: item[0])


def ensure_checkpoint(
    *,
    rom: Path,
    save: Path,
    checkpoint_dir: Path,
    frame: int,
    release: bool,
) -> Path | None:
    if frame == 0:
        return None
    checkpoint_dir.mkdir(parents=True, exist_ok=True)
    wanted = checkpoint_path(checkpoint_dir, frame)
    if wanted.exists():
        print(f"checkpoint frame {frame} exists: {wanted}")
        return wanted

    nearest = nearest_checkpoint(checkpoint_dir, frame)
    if nearest is None or nearest[0] == 0:
        output = run(
            replay_command(
                rom=rom,
                save=save,
                frames=frame,
                release=release,
                save_state=wanted,
            )
        )
    else:
        nearest_frame, nearest_path = nearest
        output = run(
            replay_command(
                rom=rom,
                save=save,
                frames=frame,
                release=release,
                load_state=nearest_path,
                save_state=wanted,
            )
        )
        print(f"advanced checkpoint {nearest_frame} -> {frame}")

    match = SAVED_RE.search(output)
    if not match:
        raise SystemExit(f"missing checkpoint save confirmation for frame {frame}")
    actual_frame = int(match.group(1))
    if actual_frame != frame:
        raise SystemExit(f"checkpoint frame mismatch: expected {frame}, got {actual_frame}")
    if not wanted.exists():
        raise SystemExit(f"checkpoint was not created: {wanted}")
    return wanted


def compare_window(
    *,
    rom: Path,
    save: Path,
    checkpoint: Path | None,
    save_checkpoint: Path | None,
    start: int,
    end: int,
    stride: int,
    release: bool,
    renderer: str | None,
) -> tuple[int, int, str, int, tuple[int, int, int, int]]:
    compare_mode = "modern-index" if renderer == "assets-variant-gpu" else "gpu-render"
    output = run(
        replay_command(
            rom=rom,
            save=save,
            frames=end,
            release=release,
            load_state=checkpoint,
            save_state=save_checkpoint,
            compare_stride=stride,
            compare_mode=compare_mode,
        ),
        renderer=renderer,
    )
    if compare_mode == "modern-index":
        match = MODERN_INDEX_SUMMARY_RE.search(output)
        if not match:
            raise SystemExit(f"missing modern-index compare summary for window {start}..{end}")
        compared = int(match.group(1))
        bad_pixels = int(match.group(3))
        if match.group(7) is not None:
            variant_draws = int(match.group(7))
            fallback_draws = int(match.group(8) or 0)
            dynamic_palette_draws = int(match.group(9))
            missing_variant_draws = int(match.group(10))
            stable_preview_draws = int(match.group(11) or 0)
            stable_effect_draws = int(match.group(12) or 0)
            dynamic_material_draws = int(match.group(13) or 0)
            unsupported_material_draws = int_stat(match.group(0), "unsupported_material_draws")
            missing_art_draws = int(match.group(14) or 0)
            unkeyed_fallback_draws = int(match.group(15) or 0)
            unkeyed_bg_fallback_draws = int_stat(match.group(0), "unkeyed_bg_fallback_draws")
            unkeyed_sprite_fallback_draws = int_stat(
                match.group(0), "unkeyed_sprite_fallback_draws"
            )
            mixed_overlay_bg_effect_draws = int(match.group(16) or 0)
            mixed_overlay_bg_effect_candidates = int(match.group(17) or 0)
            mixed_overlay_bg_effect_culled_invisible_main = int_stat(
                match.group(0), "mixed_overlay_bg_effect_culled_invisible_main"
            )
            mixed_overlay_bg_effect_reject_complex_frame = int(match.group(18) or 0)
            mixed_overlay_bg_effect_reject_complex_brightness = int(match.group(19) or 0)
            mixed_overlay_bg_effect_reject_complex_invalid_layer = int(match.group(20) or 0)
            mixed_overlay_bg_effect_reject_complex_mosaic = int(match.group(21) or 0)
            mixed_overlay_bg_effect_reject_complex_sub_window = int(match.group(22) or 0)
            mixed_overlay_bg_effect_reject_complex_effect_bounds = int(match.group(23) or 0)
            mixed_overlay_bg_effect_reject_complex_scanline_main = int(match.group(24) or 0)
            mixed_overlay_bg_effect_reject_complex_layer_window = int(match.group(25) or 0)
            mixed_overlay_bg_effect_reject_complex_color_math = int(match.group(26) or 0)
            mixed_overlay_bg_effect_reject_complex_color_math_clip = int(match.group(27) or 0)
            mixed_overlay_bg_effect_reject_complex_color_math_subscreen = int(match.group(28) or 0)
            mixed_overlay_bg_effect_reject_complex_color_math_fixed_color = int(
                match.group(29) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch = int(
                match.group(30) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap = int(
                match.group(31) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg = int(
                match.group(32) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj = int(
                match.group(33) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain = int(
                match.group(34) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front = int(
                match.group(35) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order = int(
                match.group(36) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect = int(
                match.group(37) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex = int(
                match.group(38) or 0
            )
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch = int(
                match.group(39) or 0
            )
            mixed_overlay_bg_effect_reject_cgram_mismatch = int(match.group(40) or 0)
            mixed_overlay_bg_effect_reject_overlap = int(match.group(41) or 0)
        else:
            variant_draws = 0
            fallback_draws = 0
            dynamic_palette_draws = 0
            missing_variant_draws = 0
            stable_preview_draws = 0
            stable_effect_draws = 0
            dynamic_material_draws = 0
            unsupported_material_draws = 0
            missing_art_draws = 0
            unkeyed_fallback_draws = 0
            unkeyed_bg_fallback_draws = 0
            unkeyed_sprite_fallback_draws = 0
            mixed_overlay_bg_effect_draws = 0
            mixed_overlay_bg_effect_candidates = 0
            mixed_overlay_bg_effect_culled_invisible_main = 0
            mixed_overlay_bg_effect_reject_complex_frame = 0
            mixed_overlay_bg_effect_reject_complex_brightness = 0
            mixed_overlay_bg_effect_reject_complex_invalid_layer = 0
            mixed_overlay_bg_effect_reject_complex_mosaic = 0
            mixed_overlay_bg_effect_reject_complex_sub_window = 0
            mixed_overlay_bg_effect_reject_complex_effect_bounds = 0
            mixed_overlay_bg_effect_reject_complex_scanline_main = 0
            mixed_overlay_bg_effect_reject_complex_layer_window = 0
            mixed_overlay_bg_effect_reject_complex_color_math = 0
            mixed_overlay_bg_effect_reject_complex_color_math_clip = 0
            mixed_overlay_bg_effect_reject_complex_color_math_subscreen = 0
            mixed_overlay_bg_effect_reject_complex_color_math_fixed_color = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex = 0
            mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch = 0
            mixed_overlay_bg_effect_reject_cgram_mismatch = 0
            mixed_overlay_bg_effect_reject_overlap = 0
            for frame_match in MODERN_INDEX_VARIANT_RE.finditer(output):
                variant_draws += int(frame_match.group(2))
                fallback_draws += int(frame_match.group(3) or 0)
                dynamic_palette_draws += int(frame_match.group(4))
                missing_variant_draws += int(frame_match.group(5))
                stable_preview_draws += int(frame_match.group(6) or 0)
                stable_effect_draws += int(frame_match.group(7) or 0)
                dynamic_material_draws += int(frame_match.group(8) or 0)
                unsupported_material_draws += int_stat(
                    frame_match.group(0), "unsupported_material_draws"
                )
                missing_art_draws += int(frame_match.group(9) or 0)
                unkeyed_fallback_draws += int(frame_match.group(10) or 0)
                unkeyed_bg_fallback_draws += int_stat(
                    frame_match.group(0), "unkeyed_bg_fallback_draws"
                )
                unkeyed_sprite_fallback_draws += int_stat(
                    frame_match.group(0), "unkeyed_sprite_fallback_draws"
                )
                mixed_overlay_bg_effect_draws += int(frame_match.group(11) or 0)
                mixed_overlay_bg_effect_candidates += int(frame_match.group(12) or 0)
                mixed_overlay_bg_effect_culled_invisible_main += int_stat(
                    frame_match.group(0), "mixed_overlay_bg_effect_culled_invisible_main"
                )
                mixed_overlay_bg_effect_reject_complex_frame += int(frame_match.group(13) or 0)
                mixed_overlay_bg_effect_reject_complex_brightness += int(frame_match.group(14) or 0)
                mixed_overlay_bg_effect_reject_complex_invalid_layer += int(frame_match.group(15) or 0)
                mixed_overlay_bg_effect_reject_complex_mosaic += int(frame_match.group(16) or 0)
                mixed_overlay_bg_effect_reject_complex_sub_window += int(frame_match.group(17) or 0)
                mixed_overlay_bg_effect_reject_complex_effect_bounds += int(frame_match.group(18) or 0)
                mixed_overlay_bg_effect_reject_complex_scanline_main += int(frame_match.group(19) or 0)
                mixed_overlay_bg_effect_reject_complex_layer_window += int(frame_match.group(20) or 0)
                mixed_overlay_bg_effect_reject_complex_color_math += int(frame_match.group(21) or 0)
                mixed_overlay_bg_effect_reject_complex_color_math_clip += int(
                    frame_match.group(22) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_subscreen += int(
                    frame_match.group(23) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_fixed_color += int(
                    frame_match.group(24) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch += int(
                    frame_match.group(25) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap += int(
                    frame_match.group(26) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg += int(
                    frame_match.group(27) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj += int(
                    frame_match.group(28) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain += int(
                    frame_match.group(29) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front += int(
                    frame_match.group(30) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order += int(
                    frame_match.group(31) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect += int(
                    frame_match.group(32) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex += int(
                    frame_match.group(33) or 0
                )
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch += int(
                    frame_match.group(34) or 0
                )
                mixed_overlay_bg_effect_reject_cgram_mismatch += int(frame_match.group(35) or 0)
                mixed_overlay_bg_effect_reject_overlap += int(frame_match.group(36) or 0)
        print(
            f"modern-index window {start}..{end}: compared={compared} "
            f"bad_pixels={bad_pixels} renderer={renderer} "
            f"variant_draws={variant_draws} fallback_draws={fallback_draws} "
            f"dynamic_palette_draws={dynamic_palette_draws} "
            f"missing_variant_draws={missing_variant_draws} "
            f"stable_preview_draws={stable_preview_draws} "
            f"stable_effect_draws={stable_effect_draws} "
            f"dynamic_material_draws={dynamic_material_draws} "
            f"unsupported_material_draws={unsupported_material_draws} "
            f"missing_art_draws={missing_art_draws} "
            f"unkeyed_fallback_draws={unkeyed_fallback_draws} "
            f"unkeyed_bg_fallback_draws={unkeyed_bg_fallback_draws} "
            f"unkeyed_sprite_fallback_draws={unkeyed_sprite_fallback_draws} "
            f"mixed_overlay_bg_effect_draws={mixed_overlay_bg_effect_draws} "
            f"mixed_overlay_bg_effect_candidates={mixed_overlay_bg_effect_candidates} "
            f"mixed_overlay_bg_effect_culled_invisible_main={mixed_overlay_bg_effect_culled_invisible_main} "
            f"mixed_overlay_bg_effect_reject_complex_frame={mixed_overlay_bg_effect_reject_complex_frame} "
            f"mixed_overlay_bg_effect_reject_complex_brightness={mixed_overlay_bg_effect_reject_complex_brightness} "
            f"mixed_overlay_bg_effect_reject_complex_invalid_layer={mixed_overlay_bg_effect_reject_complex_invalid_layer} "
            f"mixed_overlay_bg_effect_reject_complex_mosaic={mixed_overlay_bg_effect_reject_complex_mosaic} "
            f"mixed_overlay_bg_effect_reject_complex_sub_window={mixed_overlay_bg_effect_reject_complex_sub_window} "
            f"mixed_overlay_bg_effect_reject_complex_effect_bounds={mixed_overlay_bg_effect_reject_complex_effect_bounds} "
            f"mixed_overlay_bg_effect_reject_complex_scanline_main={mixed_overlay_bg_effect_reject_complex_scanline_main} "
            f"mixed_overlay_bg_effect_reject_complex_layer_window={mixed_overlay_bg_effect_reject_complex_layer_window} "
            f"mixed_overlay_bg_effect_reject_complex_color_math={mixed_overlay_bg_effect_reject_complex_color_math} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_clip={mixed_overlay_bg_effect_reject_complex_color_math_clip} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_subscreen={mixed_overlay_bg_effect_reject_complex_color_math_subscreen} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_fixed_color={mixed_overlay_bg_effect_reject_complex_color_math_fixed_color} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch={mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_cgram_mismatch={mixed_overlay_bg_effect_reject_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_overlap={mixed_overlay_bg_effect_reject_overlap}"
        )
        if save_checkpoint is not None and not save_checkpoint.exists():
            raise SystemExit(f"end checkpoint was not created: {save_checkpoint}")
        return (
            compared,
            end,
            "0x00000000",
            bad_pixels,
            (
                variant_draws,
                fallback_draws,
                dynamic_palette_draws,
                missing_variant_draws,
                stable_preview_draws,
                stable_effect_draws,
                dynamic_material_draws,
                missing_art_draws,
                unkeyed_fallback_draws,
                mixed_overlay_bg_effect_draws,
                mixed_overlay_bg_effect_candidates,
                mixed_overlay_bg_effect_reject_complex_frame,
                mixed_overlay_bg_effect_reject_complex_brightness,
                mixed_overlay_bg_effect_reject_complex_invalid_layer,
                mixed_overlay_bg_effect_reject_complex_mosaic,
                mixed_overlay_bg_effect_reject_complex_sub_window,
                mixed_overlay_bg_effect_reject_complex_effect_bounds,
                mixed_overlay_bg_effect_reject_complex_scanline_main,
                mixed_overlay_bg_effect_reject_complex_layer_window,
                mixed_overlay_bg_effect_reject_complex_color_math,
                mixed_overlay_bg_effect_reject_complex_color_math_clip,
                mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
                mixed_overlay_bg_effect_reject_complex_color_math_fixed_color,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex,
                mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch,
                mixed_overlay_bg_effect_reject_cgram_mismatch,
                mixed_overlay_bg_effect_reject_overlap,
                mixed_overlay_bg_effect_culled_invisible_main,
                unkeyed_bg_fallback_draws,
                unkeyed_sprite_fallback_draws,
                unsupported_material_draws,
            ),
        )
    match = COMPARE_RE.search(output)
    if not match:
        raise SystemExit(f"missing gpu-render-compare completion for window {start}..{end}")
    compared = int(match.group(1))
    last_frame = int(match.group(2))
    last_hash = match.group(3)
    mismatched_pixels = int(match.group(4))
    expected_min = max(0, end - start)
    if stride == 1 and compared != expected_min:
        raise SystemExit(
            f"window {start}..{end}: expected {expected_min} comparisons, got {compared}"
        )
    if mismatched_pixels != 0:
        raise SystemExit(
            f"window {start}..{end}: compare reported {mismatched_pixels} mismatched pixels"
        )
    if save_checkpoint is not None and not save_checkpoint.exists():
        raise SystemExit(f"end checkpoint was not created: {save_checkpoint}")
    return compared, last_frame, last_hash, mismatched_pixels, (0, 0, 0, 0, 0, 0, 0, 0, 0)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", type=Path, default=DEFAULT_ROM)
    parser.add_argument("--save", type=Path, default=DEFAULT_SAVE)
    parser.add_argument("--checkpoint-dir", type=Path, default=DEFAULT_CHECKPOINT_DIR)
    parser.add_argument("--start", type=int, default=0)
    parser.add_argument("--end", type=int, required=True)
    parser.add_argument("--window-size", type=int, default=10_000)
    parser.add_argument("--max-windows", type=int, help="limit the number of windows to run")
    parser.add_argument("--stride", type=int, default=1)
    parser.add_argument(
        "--renderer",
        help="set ZELDA3_RENDERER for compare windows, e.g. assets-variant-gpu",
    )
    parser.add_argument("--release", action="store_true")
    parser.add_argument(
        "--no-save-end-checkpoints",
        action="store_true",
        help="do not save each window's ending frame as the next reusable checkpoint",
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.start < 0 or args.end <= args.start:
        raise SystemExit("--end must be greater than --start, and --start must be non-negative")
    if args.window_size <= 0:
        raise SystemExit("--window-size must be greater than zero")
    if args.stride <= 0:
        raise SystemExit("--stride must be greater than zero")
    if args.max_windows is not None and args.max_windows <= 0:
        raise SystemExit("--max-windows must be greater than zero")
    if not args.rom.exists():
        raise SystemExit(f"ROM does not exist: {args.rom}")
    if not args.save.exists():
        raise SystemExit(f"replay save does not exist: {args.save}")
    return args


def main() -> None:
    args = parse_args()
    total_compared = 0
    total_mismatched_pixels = 0
    total_variant_draws = 0
    total_fallback_draws = 0
    total_dynamic_palette_draws = 0
    total_missing_variant_draws = 0
    total_stable_preview_draws = 0
    total_stable_effect_draws = 0
    total_dynamic_material_draws = 0
    total_unsupported_material_draws = 0
    total_missing_art_draws = 0
    total_unkeyed_fallback_draws = 0
    total_unkeyed_bg_fallback_draws = 0
    total_unkeyed_sprite_fallback_draws = 0
    total_mixed_overlay_bg_effect_draws = 0
    total_mixed_overlay_bg_effect_candidates = 0
    total_mixed_overlay_bg_effect_culled_invisible_main = 0
    total_mixed_overlay_bg_effect_reject_complex_frame = 0
    total_mixed_overlay_bg_effect_reject_complex_brightness = 0
    total_mixed_overlay_bg_effect_reject_complex_invalid_layer = 0
    total_mixed_overlay_bg_effect_reject_complex_mosaic = 0
    total_mixed_overlay_bg_effect_reject_complex_sub_window = 0
    total_mixed_overlay_bg_effect_reject_complex_effect_bounds = 0
    total_mixed_overlay_bg_effect_reject_complex_scanline_main = 0
    total_mixed_overlay_bg_effect_reject_complex_layer_window = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_clip = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_subscreen = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_fixed_color = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex = 0
    total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch = 0
    total_mixed_overlay_bg_effect_reject_cgram_mismatch = 0
    total_mixed_overlay_bg_effect_reject_overlap = 0
    last_frame = args.start
    last_hash = "0x00000000"

    for window_index, start in enumerate(range(args.start, args.end, args.window_size)):
        if args.max_windows is not None and window_index >= args.max_windows:
            break
        end = min(start + args.window_size, args.end)
        checkpoint = checkpoint_path(args.checkpoint_dir, start) if start > 0 else None
        save_checkpoint = (
            None
            if args.no_save_end_checkpoints
            else checkpoint_path(args.checkpoint_dir, end)
        )
        if args.dry_run:
            if checkpoint is not None:
                print(f"ensure checkpoint {start}: {checkpoint}")
            if save_checkpoint is None:
                print(
                    f"compare window {start}..{end} stride={args.stride} "
                    f"renderer={args.renderer or '<default>'}"
                )
            else:
                print(
                    f"compare window {start}..{end} stride={args.stride} "
                    f"save_checkpoint={save_checkpoint} renderer={args.renderer or '<default>'}"
                )
            continue

        checkpoint = ensure_checkpoint(
            rom=args.rom,
            save=args.save,
            checkpoint_dir=args.checkpoint_dir,
            frame=start,
            release=args.release,
        )
        (
            compared,
            last_frame,
            last_hash,
            mismatched_pixels,
            variant_stats,
        ) = compare_window(
            rom=args.rom,
            save=args.save,
            checkpoint=checkpoint,
            save_checkpoint=save_checkpoint,
            start=start,
            end=end,
            stride=args.stride,
            release=args.release,
            renderer=args.renderer,
        )
        total_compared += compared
        total_mismatched_pixels += mismatched_pixels
        total_variant_draws += variant_stats[0]
        total_fallback_draws += variant_stats[1]
        total_dynamic_palette_draws += variant_stats[2]
        total_missing_variant_draws += variant_stats[3]
        total_stable_preview_draws += variant_stats[4]
        total_stable_effect_draws += variant_stats[5]
        total_dynamic_material_draws += variant_stats[6]
        total_unsupported_material_draws += variant_stats[38]
        total_missing_art_draws += variant_stats[7]
        total_unkeyed_fallback_draws += variant_stats[8]
        total_unkeyed_bg_fallback_draws += variant_stats[36]
        total_unkeyed_sprite_fallback_draws += variant_stats[37]
        total_mixed_overlay_bg_effect_draws += variant_stats[9]
        total_mixed_overlay_bg_effect_candidates += variant_stats[10]
        total_mixed_overlay_bg_effect_culled_invisible_main += variant_stats[35]
        total_mixed_overlay_bg_effect_reject_complex_frame += variant_stats[11]
        total_mixed_overlay_bg_effect_reject_complex_brightness += variant_stats[12]
        total_mixed_overlay_bg_effect_reject_complex_invalid_layer += variant_stats[13]
        total_mixed_overlay_bg_effect_reject_complex_mosaic += variant_stats[14]
        total_mixed_overlay_bg_effect_reject_complex_sub_window += variant_stats[15]
        total_mixed_overlay_bg_effect_reject_complex_effect_bounds += variant_stats[16]
        total_mixed_overlay_bg_effect_reject_complex_scanline_main += variant_stats[17]
        total_mixed_overlay_bg_effect_reject_complex_layer_window += variant_stats[18]
        total_mixed_overlay_bg_effect_reject_complex_color_math += variant_stats[19]
        total_mixed_overlay_bg_effect_reject_complex_color_math_clip += variant_stats[20]
        total_mixed_overlay_bg_effect_reject_complex_color_math_subscreen += variant_stats[21]
        total_mixed_overlay_bg_effect_reject_complex_color_math_fixed_color += variant_stats[22]
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch += (
            variant_stats[23]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap += variant_stats[
            24
        ]
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg += (
            variant_stats[25]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj += (
            variant_stats[26]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain += (
            variant_stats[27]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front += (
            variant_stats[28]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order += (
            variant_stats[29]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect += (
            variant_stats[30]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex += (
            variant_stats[31]
        )
        total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch += (
            variant_stats[32]
        )
        total_mixed_overlay_bg_effect_reject_cgram_mismatch += variant_stats[33]
        total_mixed_overlay_bg_effect_reject_overlap += variant_stats[34]

    if not args.dry_run:
        if args.renderer == "assets-variant-gpu":
            ensure_no_unsupported_material_draws(total_unsupported_material_draws)
        print(
            "gpu-render-window-compare completed "
            f"start={args.start} end={args.end} stride={args.stride} "
            f"compared={total_compared} last_frame={last_frame} last_hash={last_hash} "
            f"mismatched_pixels={total_mismatched_pixels} "
            f"variant_draws={total_variant_draws} "
            f"fallback_draws={total_fallback_draws} "
            f"dynamic_palette_draws={total_dynamic_palette_draws} "
            f"missing_variant_draws={total_missing_variant_draws} "
            f"stable_preview_draws={total_stable_preview_draws} "
            f"stable_effect_draws={total_stable_effect_draws} "
            f"dynamic_material_draws={total_dynamic_material_draws} "
            f"unsupported_material_draws={total_unsupported_material_draws} "
            f"missing_art_draws={total_missing_art_draws} "
            f"unkeyed_fallback_draws={total_unkeyed_fallback_draws} "
            f"unkeyed_bg_fallback_draws={total_unkeyed_bg_fallback_draws} "
            f"unkeyed_sprite_fallback_draws={total_unkeyed_sprite_fallback_draws} "
            f"mixed_overlay_bg_effect_draws={total_mixed_overlay_bg_effect_draws} "
            f"mixed_overlay_bg_effect_candidates={total_mixed_overlay_bg_effect_candidates} "
            f"mixed_overlay_bg_effect_culled_invisible_main={total_mixed_overlay_bg_effect_culled_invisible_main} "
            f"mixed_overlay_bg_effect_reject_complex_frame={total_mixed_overlay_bg_effect_reject_complex_frame} "
            f"mixed_overlay_bg_effect_reject_complex_brightness={total_mixed_overlay_bg_effect_reject_complex_brightness} "
            f"mixed_overlay_bg_effect_reject_complex_invalid_layer={total_mixed_overlay_bg_effect_reject_complex_invalid_layer} "
            f"mixed_overlay_bg_effect_reject_complex_mosaic={total_mixed_overlay_bg_effect_reject_complex_mosaic} "
            f"mixed_overlay_bg_effect_reject_complex_sub_window={total_mixed_overlay_bg_effect_reject_complex_sub_window} "
            f"mixed_overlay_bg_effect_reject_complex_effect_bounds={total_mixed_overlay_bg_effect_reject_complex_effect_bounds} "
            f"mixed_overlay_bg_effect_reject_complex_scanline_main={total_mixed_overlay_bg_effect_reject_complex_scanline_main} "
            f"mixed_overlay_bg_effect_reject_complex_layer_window={total_mixed_overlay_bg_effect_reject_complex_layer_window} "
            f"mixed_overlay_bg_effect_reject_complex_color_math={total_mixed_overlay_bg_effect_reject_complex_color_math} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_clip={total_mixed_overlay_bg_effect_reject_complex_color_math_clip} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_subscreen={total_mixed_overlay_bg_effect_reject_complex_color_math_subscreen} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_fixed_color={total_mixed_overlay_bg_effect_reject_complex_color_math_fixed_color} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex} "
            f"mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch={total_mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_cgram_mismatch={total_mixed_overlay_bg_effect_reject_cgram_mismatch} "
            f"mixed_overlay_bg_effect_reject_overlap={total_mixed_overlay_bg_effect_reject_overlap}"
        )


if __name__ == "__main__":
    main()
