#!/usr/bin/env python3
"""Rewrite native-owned state read accessors to direct GameState paths.

This is intentionally narrow. It only handles accessors whose backing state is
already native-owned and dual-synced to RAM. Bridge-backed mutation helpers are
left alone because they still project updates into RAM during the transition.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"


@dataclass(frozen=True)
class AccessorMapping:
    accessor: str
    game_state_path: str
    borrowed_alias: bool = True


@dataclass(frozen=True)
class SingleArgAccessorMapping:
    accessor: str
    game_state_path: str
    native_method: str
    mutable: bool = False


MANUAL_ACCESSORS: tuple[AccessorMapping, ...] = (
    AccessorMapping("frame_state", "game_state.frame"),
    AccessorMapping("world_location_state", "game_state.world.location"),
    AccessorMapping("display_state", "game_state.display"),
    AccessorMapping("intro_scene_state", "game_state.ending.intro_scene"),
    AccessorMapping("weather_vane_state", "game_state.world.overworld.weather_vane"),
    AccessorMapping("trinexx_palette_state", "game_state.display.trinexx_palette"),
    AccessorMapping("overworld_map16_load_state", "game_state.world.overworld.map16.active_load", borrowed_alias=False),
    AccessorMapping("overworld_prev_map16_load_state", "game_state.world.overworld.map16.previous_load", borrowed_alias=False),
    AccessorMapping("world_scroll", "game_state.world.scroll"),
    AccessorMapping("world_camera_boundaries", "game_state.world.camera_boundaries"),
    AccessorMapping("world_palette_theme", "game_state.world.palette_theme"),
    AccessorMapping("world_region", "game_state.world.region"),
    AccessorMapping("world_transient", "game_state.world.transient"),
    AccessorMapping("enhanced_features", "game_state.enhanced_features"),
    AccessorMapping("system_signals", "game_state.system_signals"),
    AccessorMapping("ppu_scroll_copy", "game_state.display.ppu_scroll_copy"),
    AccessorMapping("attract_scene", "game_state.ending.attract_scene"),
    AccessorMapping("dialogue_message_index", "game_state.messaging.dialogue_message_index"),
    AccessorMapping("save_progress", "game_state.inventory.save_progress"),
    AccessorMapping("temp_counter", "game_state.scratch_counter"),
    AccessorMapping("palette_buffer", "game_state.display.palette_buffer"),
    AccessorMapping("palette_filter", "game_state.display.palette_filter"),
    AccessorMapping("mirror_warp_scratch", "game_state.inventory.mirror_warp"),
    AccessorMapping("overworld_event_info", "game_state.world.overworld.event_info"),
    AccessorMapping("hud_inventory_order_state", "game_state.display.hud_inventory_order"),
    AccessorMapping("room_bounds", "game_state.world.room_bounds"),
    AccessorMapping("inventory_items", "game_state.inventory.items"),
    AccessorMapping("player_resources", "game_state.inventory.player_resources"),
    AccessorMapping("select_file_scratch", "game_state.messaging.select_file_menu"),
    AccessorMapping("minigame_state", "game_state.minigame"),
    AccessorMapping("shared_message_timer_state", "game_state.messaging.shared_message_timer"),
    AccessorMapping("ending_credit_state", "game_state.ending.credits"),
    AccessorMapping("messaging_state", "game_state.messaging.runtime"),
    AccessorMapping("archery_game", "game_state.archery_game"),
    AccessorMapping("intro_sword", "game_state.intro_sword"),
    AccessorMapping("memorized_tile", "game_state.memorized_tiles"),
    AccessorMapping("dialogue_number", "game_state.messaging.dialogue_number"),
    AccessorMapping("messaging_text", "game_state.messaging.decoded_text"),
    AccessorMapping("spotlight_hdma", "game_state.display.spotlight_hdma"),
    AccessorMapping("water_hdma_window", "game_state.display.water_hdma_window"),
    AccessorMapping("messaging_render_buffer", "game_state.messaging.render_buffer"),
    AccessorMapping("vwf_glyph_spacing", "game_state.messaging.vwf_render"),
    AccessorMapping("poly_runtime", "game_state.poly.runtime"),
    AccessorMapping("poly_projected_vertex", "game_state.poly.projected_vertices"),
    AccessorMapping("poly_face_coords", "game_state.poly.face_coords"),
    AccessorMapping("poly_raster_edge", "game_state.poly.raster_edge"),
    AccessorMapping("effect_angle_scratch", "game_state.effects.angle_scratch"),
    AccessorMapping("quake_spell_scratch", "game_state.effects.quake_spell"),
    AccessorMapping("bombos_spell_scratch", "game_state.effects.bombos_spell"),
    AccessorMapping("tower_seal_scratch", "game_state.effects.tower_seal"),
    AccessorMapping("special_exit_position", "game_state.player.special_exit_position"),
    AccessorMapping("pushed_block", "game_state.player.pushed_block"),
    AccessorMapping("player_tile_attributes", "game_state.player.tile_attributes"),
    AccessorMapping("dungeon_key_slots", "game_state.inventory.dungeon_key_slots"),
    AccessorMapping("ending_scratch", "game_state.dungeon.scratch_word"),
    AccessorMapping("save_load_scratch", "game_state.save_load_transfer"),
    AccessorMapping("dungeon_map_scratch", "game_state.dungeon_map_display"),
    AccessorMapping("dungeon_secret_scratch", "game_state.dungeon_secret"),
    AccessorMapping("sprite_battle", "game_state.sprite_battle"),
    AccessorMapping("door_debris", "game_state.effects.door_debris"),
    AccessorMapping("digging_game_prize", "game_state.effects.digging_game_prize"),
    AccessorMapping("chain_chomp_history", "game_state.sprites.chain_chomp_history"),
    AccessorMapping("maze_game_timer", "game_state.sprites.maze_game_timer"),
    AccessorMapping("enemy_damage_subclass_table", "game_state.sprites.enemy_damage_subclasses"),
    AccessorMapping("ether_orbit", "game_state.sprites.ether_orbit"),
    AccessorMapping("dual_layer_tile_cache", "game_state.sprites.dual_layer_tile_cache"),
    AccessorMapping("dungeon_header", "game_state.dungeon.header"),
    AccessorMapping("dungeon_moving_floor", "game_state.dungeon.moving_floor"),
    AccessorMapping("dungeon_room_tracking", "game_state.dungeon.room_tracking"),
    AccessorMapping("dungeon_environment", "game_state.dungeon.environment"),
    AccessorMapping("dungeon_room_items", "game_state.dungeon.room_items"),
    AccessorMapping("dungeon_room_effects", "game_state.dungeon.room_effects"),
    AccessorMapping("dungeon_room_doors", "game_state.dungeon.door_setup"),
    AccessorMapping("dungeon_room_runtime", "game_state.dungeon.room_runtime"),
    AccessorMapping("dungeon_movable_blocks", "game_state.dungeon.movable_blocks"),
    AccessorMapping("scratch_word", "game_state.dungeon.scratch_word"),
    AccessorMapping("draw_scratch_position", "game_state.sprites.draw_hitbox_work"),
    AccessorMapping("hitbox_scratch_offset", "game_state.sprites.draw_hitbox_work"),
    AccessorMapping("overworld_sprite_presence", "game_state.sprites.overworld_sprite_presence"),
    AccessorMapping("overworld_sprite_loaded", "game_state.sprites.overworld_sprite_loaded"),
    AccessorMapping("dungeon_bg2_attributes", "game_state.dungeon.bg2_attributes"),
    AccessorMapping("dungeon_savegame_state", "game_state.dungeon.savegame_state"),
    AccessorMapping("dungeon_stair_movement", "game_state.dungeon.stair_movement"),
    AccessorMapping("dungeon_torch_state", "game_state.dungeon.torch"),
    AccessorMapping("dungeon_doors", "game_state.dungeon.doors"),
    AccessorMapping("dungeon_object_tracking", "game_state.dungeon.object_tracking"),
    AccessorMapping("dungeon_room_load", "game_state.dungeon.room_load"),
    AccessorMapping("dungeon_stair_lists", "game_state.dungeon.stair_lists"),
    AccessorMapping("dungeon_room_tilemaps", "game_state.dungeon.room_tilemaps"),
    AccessorMapping("dungeon_room_parser", "game_state.dungeon.room_parser"),
    AccessorMapping("swim_acceleration", "game_state.player.swim_acceleration"),
    AccessorMapping("tile_detect_position", "game_state.player.tile_detection"),
    AccessorMapping("follower_link_state", "game_state.player.follower_link"),
    AccessorMapping("follower_state", "game_state.sprites.follower_runtime"),
    AccessorMapping("garnish_state", "game_state.sprites.garnish_runtime"),
    AccessorMapping("sprite_system", "game_state.sprites.system"),
    AccessorMapping("oam_state", "game_state.oam"),
    AccessorMapping("sprite_workspace", "game_state.sprites.workspace"),
)


SINGLE_ARG_ACCESSORS: tuple[SingleArgAccessorMapping, ...] = (
    SingleArgAccessorMapping("ancilla_slot", "game_state.sprites.ancilla_slots", "slot"),
    SingleArgAccessorMapping(
        "ancilla_slot_mut",
        "game_state.sprites.ancilla_slots",
        "slot_mut",
        mutable=True,
    ),
    SingleArgAccessorMapping("garnish_slot", "game_state.sprites.garnish_slots", "slot"),
    SingleArgAccessorMapping(
        "garnish_slot_mut",
        "game_state.sprites.garnish_slots",
        "slot_mut",
        mutable=True,
    ),
    SingleArgAccessorMapping("overlord_slot", "game_state.sprites.overlord_slots", "slot"),
    SingleArgAccessorMapping(
        "overlord_slot_mut",
        "game_state.sprites.overlord_slots",
        "slot_mut",
        mutable=True,
    ),
)


MIGRATED_NATIVE_READ_ACCESSORS: tuple[str, ...] = tuple(
    mapping.accessor for mapping in MANUAL_ACCESSORS
)
MIGRATED_SINGLE_ARG_NATIVE_ACCESSORS: tuple[str, ...] = tuple(
    mapping.accessor for mapping in SINGLE_ARG_ACCESSORS
)

DIRECT_GAME_STATE_ACCESSOR_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([a-z][A-Za-z0-9_]*)\s*"
    r"\(\s*&self\s*\)\s*->\s*([^{}]+?)\s*\{\s*"
    r"(&?)\s*self\s*\.\s*game_state((?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)+)\s*"
    r"\}",
    re.MULTILINE | re.DOTALL,
)
DIRECT_SINGLE_ARG_GAME_STATE_ACCESSOR_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([a-z][A-Za-z0-9_]*)\s*"
    r"\(\s*&self\s*,\s*([a-z][A-Za-z0-9_]*)\s*:\s*usize\s*\)\s*->\s*([^{}]+?)\s*\{\s*"
    r"self\s*\.\s*game_state((?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*)+)"
    r"\s*\.\s*([a-z][A-Za-z0-9_]*)\s*\(\s*\2\s*\)\s*"
    r"\}",
    re.MULTILINE | re.DOTALL,
)


def default_paths() -> list[Path]:
    return sorted(path for path in SRC_ROOT.glob("*.rs") if path.is_file())


def is_game_state_access_layer(path: Path) -> bool:
    path = path.resolve()
    return path.is_relative_to(SRC_ROOT / "game_state")


def relative(path: Path) -> str:
    path = path.resolve()
    return str(path.relative_to(REPO_ROOT))


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def normalized_game_state_path(field_path: str) -> str:
    fields = [part.strip() for part in field_path.split(".") if part.strip()]
    return ".".join(["game_state", *fields])


def inferred_direct_game_state_accessors() -> tuple[AccessorMapping, ...]:
    """Find read helpers that are already just `&self.game_state.foo`.

    These are the safest helpers to rewrite mechanically: the helper body adds
    no behavior, does not compose multiple native structs, and does not touch
    RAM. Composite read wrappers and bridge mutators are intentionally skipped.
    """

    text = (SRC_ROOT / "zelda_rtl.rs").read_text()
    accessors: list[AccessorMapping] = []
    for match in DIRECT_GAME_STATE_ACCESSOR_RE.finditer(text):
        accessor = match.group(1)
        return_type = " ".join(match.group(2).split())
        body_borrow = match.group(3) == "&"
        if accessor.endswith("_mut"):
            continue
        if not return_type.startswith("&") and body_borrow:
            continue
        if return_type.startswith("&") != body_borrow:
            continue
        if "State" not in return_type and "Read" not in return_type:
            continue
        accessors.append(
            AccessorMapping(
                accessor,
                normalized_game_state_path(match.group(4)),
                borrowed_alias=return_type.startswith("&"),
            )
        )
    return tuple(accessors)


def inferred_single_arg_game_state_accessors() -> tuple[SingleArgAccessorMapping, ...]:
    """Find `fn foo(&self, slot: usize) { self.game_state.bar.baz(slot) }`.

    These helpers return copied native slot/scratch state and add no behavior.
    They can be rewritten to call the native subsystem method directly.
    """

    text = (SRC_ROOT / "zelda_rtl.rs").read_text()
    accessors: list[SingleArgAccessorMapping] = []
    for match in DIRECT_SINGLE_ARG_GAME_STATE_ACCESSOR_RE.finditer(text):
        accessor = match.group(1)
        return_type = " ".join(match.group(3).split())
        if accessor.endswith("_mut"):
            continue
        if "State" not in return_type and "Read" not in return_type:
            continue
        accessors.append(
            SingleArgAccessorMapping(
                accessor,
                normalized_game_state_path(match.group(4)),
                match.group(5),
            )
        )
    return tuple(accessors)


def selected_accessors(
    include_inferred: bool,
    accessor_regex: str | None,
    exclude_accessor_regex: str | None,
) -> tuple[AccessorMapping, ...]:
    mappings = list(MANUAL_ACCESSORS)
    if include_inferred:
        known = {mapping.accessor for mapping in mappings}
        mappings.extend(
            mapping
            for mapping in inferred_direct_game_state_accessors()
            if mapping.accessor not in known
        )
    if accessor_regex:
        pattern = re.compile(accessor_regex)
        mappings = [mapping for mapping in mappings if pattern.search(mapping.accessor)]
    if exclude_accessor_regex:
        pattern = re.compile(exclude_accessor_regex)
        mappings = [mapping for mapping in mappings if not pattern.search(mapping.accessor)]
    return tuple(sorted(mappings, key=lambda mapping: mapping.accessor))


def selected_single_arg_accessors(
    include_single_arg: bool,
    include_inferred_single_arg: bool,
    accessor_regex: str | None,
    exclude_accessor_regex: str | None,
) -> tuple[SingleArgAccessorMapping, ...]:
    if not include_single_arg and not include_inferred_single_arg:
        return ()
    mappings = list(SINGLE_ARG_ACCESSORS) if include_single_arg else []
    if include_inferred_single_arg:
        known = {mapping.accessor for mapping in mappings}
        mappings.extend(
            mapping
            for mapping in inferred_single_arg_game_state_accessors()
            if mapping.accessor not in known
        )
    if accessor_regex:
        pattern = re.compile(accessor_regex)
        mappings = [mapping for mapping in mappings if pattern.search(mapping.accessor)]
    if exclude_accessor_regex:
        pattern = re.compile(exclude_accessor_regex)
        mappings = [mapping for mapping in mappings if not pattern.search(mapping.accessor)]
    return tuple(sorted(mappings, key=lambda mapping: mapping.accessor))


def rewrite_text(
    text: str,
    accessors: tuple[AccessorMapping, ...],
    single_arg_accessors: tuple[SingleArgAccessorMapping, ...],
) -> str:
    for mapping in accessors:
        accessor = re.escape(mapping.accessor)
        path = mapping.game_state_path

        # receiver.accessor().method_or_field, allowing rustfmt line breaks.
        text = re.sub(
            rf"\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*{accessor}\(\)\s*\.",
            rf"\1.{path}.",
            text,
        )

        # let local = receiver.accessor(); for read aliases.
        alias_prefix = r"&" if mapping.borrowed_alias else ""
        text = re.sub(
            rf"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\.{accessor}\(\);",
            rf"let \1 = {alias_prefix}\2.{path};",
            text,
        )

        # let local = *receiver.accessor(); for copy aliases.
        text = re.sub(
            rf"\blet\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\*([A-Za-z_][A-Za-z0-9_]*)\.{accessor}\(\);",
            rf"let \1 = \2.{path};",
            text,
        )
    for mapping in single_arg_accessors:
        accessor = re.escape(mapping.accessor)
        path = mapping.game_state_path
        method = mapping.native_method
        arg = r"([^);\n]+?)"

        if mapping.mutable:
            replacement = rf"self.{path}.{method}(&mut self.ram, \1)."
            alias_replacement = rf"let \1\2 = self.{path}.{method}(&mut self.ram, \3);"
        else:
            replacement = rf"self.{path}.{method}(\1)."
            alias_replacement = rf"let \1\2 = self.{path}.{method}(\3);"

        text = re.sub(
            rf"\bself\s*\.\s*{accessor}\(\s*{arg}\s*\)\s*\.",
            replacement,
            text,
        )
        text = re.sub(
            rf"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"
            rf"self\.{accessor}\(\s*{arg}\s*\)\s*;",
            alias_replacement,
            text,
        )
    return text


def findings(
    path: Path,
    text: str,
    accessors: tuple[AccessorMapping, ...],
    single_arg_accessors: tuple[SingleArgAccessorMapping, ...],
) -> list[str]:
    names = [re.escape(mapping.accessor) for mapping in accessors]
    lines = []
    if names:
        pattern = re.compile(rf"\b(?:{'|'.join(names)})\(")
        for match in pattern.finditer(text):
            line_start = text.rfind("\n", 0, match.start()) + 1
            if re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+", text[line_start:match.start()]):
                continue
            lines.append(f"{relative(path)}:{line_for_offset(text, match.start())}: {match.group(0)}")
    single_arg_names = [re.escape(mapping.accessor) for mapping in single_arg_accessors]
    if single_arg_names:
        pattern = re.compile(rf"\bself\s*\.\s*(?:{'|'.join(single_arg_names)})\(")
        for match in pattern.finditer(text):
            lines.append(f"{relative(path)}:{line_for_offset(text, match.start())}: {match.group(0)}")
    return lines


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--apply",
        action="store_true",
        help="rewrite files in place; without this, only report remaining accessor reads",
    )
    parser.add_argument(
        "--infer-direct-game-state",
        action="store_true",
        help="also target read helpers whose body is a direct self.game_state path",
    )
    parser.add_argument(
        "--include-single-arg-native-accessors",
        action="store_true",
        help="also target native-backed single-argument accessors such as overlord_slot(k)",
    )
    parser.add_argument(
        "--infer-direct-single-arg-game-state",
        action="store_true",
        help="also target single-argument helpers whose body is a direct self.game_state method call",
    )
    parser.add_argument(
        "--accessor-regex",
        help="limit targeted accessors by regular expression",
    )
    parser.add_argument(
        "--exclude-accessor-regex",
        help="exclude targeted accessors by regular expression",
    )
    parser.add_argument(
        "--list-mappings",
        action="store_true",
        help="print selected accessor-to-game_state mappings before scanning",
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help="Rust files to scan or rewrite; defaults to crates/zelda3/src/*.rs",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    accessors = selected_accessors(
        args.infer_direct_game_state,
        args.accessor_regex,
        args.exclude_accessor_regex,
    )
    single_arg_accessors = selected_single_arg_accessors(
        args.include_single_arg_native_accessors,
        args.infer_direct_single_arg_game_state,
        args.accessor_regex,
        args.exclude_accessor_regex,
    )
    if args.list_mappings:
        for mapping in accessors:
            mode = "borrow" if mapping.borrowed_alias else "copy"
            print(f"{mapping.accessor} -> {mapping.game_state_path} ({mode})")
        for mapping in single_arg_accessors:
            mode = "mut" if mapping.mutable else "read"
            print(
                f"{mapping.accessor}(arg) -> "
                f"{mapping.game_state_path}.{mapping.native_method}(arg) ({mode})"
            )
        if not args.apply and not args.paths:
            return 0

    paths = args.paths or default_paths()
    changed: list[Path] = []
    all_findings: list[str] = []

    for path in paths:
        if path.is_dir():
            files = sorted(path.rglob("*.rs"))
        else:
            files = [path]
        for file_path in files:
            if is_game_state_access_layer(file_path):
                continue
            text = file_path.read_text()
            if args.apply:
                next_text = rewrite_text(text, accessors, single_arg_accessors)
                if next_text != text:
                    file_path.write_text(next_text)
                    changed.append(file_path)
                    text = next_text
            all_findings.extend(findings(file_path, text, accessors, single_arg_accessors))

    if args.apply and changed:
        print("rewrote native state accessors:")
        for path in changed:
            print(f"  {relative(path)}")

    if all_findings:
        print("remaining native read accessor call(s):")
        for finding in all_findings:
            print(f"  {finding}")
        return 1

    print("native state accessors ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
