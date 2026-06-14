#!/usr/bin/env python3
"""Check and document Zelda RAM naming readability.

This guard is intentionally lexical. It prevents the easy regressions that made
the port hard to read: address-derived constant names and direct hex RAM slots.
It also generates a lightweight RAM map from the constants that remain in the
Rust port.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

from migrate_native_state_access import (
    MIGRATED_NATIVE_READ_ACCESSORS,
    MIGRATED_SINGLE_ARG_NATIVE_ACCESSORS,
)


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"
DEFAULT_DOC = REPO_ROOT / "docs" / "ram-map.md"
DEFAULT_WEAK_DOC = REPO_ROOT / "docs" / "ram-weak-names.md"
DEFAULT_CANDIDATE_DOC = REPO_ROOT / "docs" / "ram-source-backed-candidates.md"
DEFAULT_NES_VER2_CROSSWALK = REPO_ROOT / "docs" / "nes-ver2" / "ram_symbol_crosswalk.json"

ADDRESS_NAME_RE = re.compile(r"\b(?:BYTE|WORD|DWORD)_7E[0-9A-Fa-f][A-Za-z0-9_]*\b")
C_ADDRESS_NAME_RE = re.compile(r"\b(?:byte|word|dword)_7E[0-9A-Fa-f][A-Za-z0-9_]*\b")
DIRECT_RAM_RE = re.compile(
    r"(?:self\.ram\[\s*0x[0-9A-Fa-f]+|"
    r"read_le_u16\(\s*&self\.ram\s*,\s*0x[0-9A-Fa-f]+|"
    r"write_le_u16\(\s*&mut self\.ram\s*,\s*0x[0-9A-Fa-f]+)"
)
DIRECT_RAM_ANY_RE = re.compile(
    r"(?:\bself\.ram\[\s*[^]\n]+|"
    r"\bread_le_u16\(\s*&self\.ram\s*,\s*[^)\n]+|"
    r"\bwrite_le_u16\(\s*&mut self\.ram\s*,\s*[^)\n]+)"
)
NATIVE_BRIDGE_RE = re.compile(
    r"\bnative_ram_bridge_view(?:_mut)?\(\)\s*\.(?:"
    r"byte_at|word_at|long_at|range|watched_byte|"
    r"set_byte_at|set_word_at|set_long_at|range_mut|copy_to|fill|"
    r"move_link_axis_by_velocity|move_link_axis_by_subpixel_delta"
    r")\b"
)
NATIVE_TRANSITION_MUT_RE = re.compile(r"\bNative[A-Za-z0-9]*Mut\b")
ALLOWED_NATIVE_TRANSITION_MUT_NAMES = {
    "NativeRamBridgeViewMut",
    "NativeRamBridgeMut",
}
NATIVE_READ_ACCESSOR_RE = re.compile(
    r"\b(?:"
    + "|".join(re.escape(accessor) for accessor in MIGRATED_NATIVE_READ_ACCESSORS)
    + r")\(\)"
)
MIGRATED_SINGLE_ARG_NATIVE_ACCESSOR_RE = re.compile(
    r"(?:\.\s*|\bfn\s+)(?:"
    + "|".join(re.escape(accessor) for accessor in MIGRATED_SINGLE_ARG_NATIVE_ACCESSORS)
    + r")\s*\("
)
FRAME_RAM_VIEW_RE = re.compile(r"\bRamFrameStateView(?:Mut)?\b")
FRAME_CONTROL_ADDRESSES = {
    0x0010: "MAIN_MODULE",
    0x0011: "SUBMODULE",
    0x00B0: "SUBSUBMODULE",
    0x001A: "FRAME_COUNTER",
    0x010C: "SAVED_MODULE_FOR_MENU",
    0x0FC1: "MODAL_PAUSE_FLAG",
}
WORLD_LOCATION_ADDRESSES = {
    0x001B: "PLAYER_IS_INDOORS",
    0x008A: "OVERWORLD_SCREEN_INDEX",
    0x00A0: "DUNGEON_ROOM",
}
DISPLAY_CORE_ADDRESSES = {
    0x0013: "INIDISP_COPY",
    0x001C: "TM_COPY",
    0x001D: "TS_COPY",
    0x001E: "TMW_COPY",
    0x001F: "TSW_COPY",
    0x0094: "BGMODE_COPY",
    0x0095: "MOSAIC_COPY",
    0x0096: "W12SEL_COPY",
    0x0097: "W34SEL_COPY",
    0x0098: "WOBJSEL_COPY",
}
DISPLAY_UPLOAD_METADATA_ADDRESSES = {
    0x0116: "NMI_LOAD_TARGET_ADDR",
    0x0412: "INCREMENTAL_COUNTER_FOR_VRAM",
    0x1000: "VRAM_UPLOAD_OFFSET",
}
DISPLAY_CONTROL_ADDRESSES = {
    0x0012: "NMI_BOOLEAN",
    0x0017: "NMI_SUBROUTINE_INDEX",
    0x0019: "NMI_UPDATE_TILEMAP_DST",
    0x009B: "HDMAEN_COPY",
    0x00FF: "VIRQ_TRIGGER",
    0x0118: "NMI_UPDATE_TILEMAP_SRC",
    0x0128: "IRQ_FLAG",
    0x012A: "NMI_THREAD_ACTIVE",
    0x0134: "ANIMATED_TILE_VRAM_ADDR",
    0x0649: "CRYSTAL_ROTATION_COUNTER",
    0x0710: "NMI_DISABLE_CORE_UPDATES",
    0x0AAA: "LOAD_CHR_HALFSLOT_EVEN_ODD",
    0x0ADC: "ANIMATED_TILE_DATA_SRC",
    0x0AE8: "DMA_HEAD_POINTER",
    0x0AEA: "DMA_BODY_POINTER",
    0x0C00B: "MOSAIC_TARGET_LEVEL",
    0x0C011: "MOSAIC_LEVEL",
    0x1F0A: "POLY_THREAD_STACK",
    0x1F0C: "NMI_FLAG_UPDATE_POLYHEDRAL",
}
DISPLAY_PALETTE_FILTER_ADDRESSES = {
    0x0099: "CGWSEL_COPY",
    0x009A: "CGADSUB_COPY",
    0x009C: "COLDATA_COPY0",
    0x009D: "COLDATA_COPY1",
    0x009E: "COLDATA_COPY2",
    0x0C007: "PALETTE_FILTER_COUNTDOWN",
    0x0C009: "DARKENING_OR_LIGHTENING_SCREEN",
}
DISPLAY_LINK_DMA_SOURCE_ADDRESSES = {
    0x0AC0: "DMA_SOURCE_ADDR_6",
    0x0AC2: "DMA_SOURCE_ADDR_11",
    0x0AC4: "DMA_SOURCE_ADDR_7",
    0x0AC6: "DMA_SOURCE_ADDR_12",
    0x0AC8: "DMA_SOURCE_ADDR_8",
    0x0ACA: "DMA_SOURCE_ADDR_13",
    0x0ACC: "DMA_SOURCE_ADDR_3",
    0x0ACE: "DMA_SOURCE_ADDR_0",
    0x0AD0: "DMA_SOURCE_ADDR_4",
    0x0AD2: "DMA_SOURCE_ADDR_1",
    0x0AD4: "DMA_SOURCE_ADDR_5",
    0x0AD6: "DMA_SOURCE_ADDR_2",
    0x0AD8: "DMA_SOURCE_ADDR_10",
    0x0ADA: "DMA_SOURCE_ADDR_15",
    0x0AE0: "DMA_SOURCE_ADDR_9",
    0x0AE2: "DMA_SOURCE_ADDR_14",
    0x0AEC: "DMA_SOURCE_ADDR_16",
    0x0AEE: "DMA_SOURCE_ADDR_18",
    0x0AF0: "DMA_SOURCE_ADDR_17",
    0x0AF2: "DMA_SOURCE_ADDR_19",
    0x0AF6: "DMA_SOURCE_ADDR_20",
    0x0AF8: "DMA_SOURCE_ADDR_21",
}
DISPLAY_ANIMATION_MESSAGE_ADDRESSES = {
    0x0219: "MESSAGE_DMA_DST_ADDR",
    0x021D: "MESSAGE_DMA_TILE_BASE",
    0x021F: "MESSAGE_DMA_TILE_LIMIT",
    0x0221: "MESSAGE_DMA_TILE_SENTINEL",
    0x0AF4: "FLAG_TRAVEL_BIRD",
    0x0C00D: "BG_TILE_ANIMATION_COUNTDOWN",
}
WEAK_NAME_RE = re.compile(r"(?:^|_)(UNK\d*|SOME|VAR\d*|TMP|SCRATCH)(?:_|$)")
REVIEWED_WEAK_RAM_NAMES = {
    "ATTRACT_VAR7": "write-only attract scene work RAM; source label is the unrelated shared PYFLCH alias",
    "DUNGMAP_VAR7": "shared DMWRK0 work RAM used by dungeon-map and sprite drawing code",
    "SCRATCH_0": "shared zero-page scratch reused across unrelated player/overworld/tile code",
    "SCRATCH_A": "shared zero-page scratch reused across unrelated player/overworld/tile code",
    "SCRATCH_1": "shared zero-page scratch reused across unrelated player/overworld/tile code",
    "SCRATCH_0_ANCILLA": "single-use ancilla coordinate scratch; source BMWORK is broader beam/work RAM",
    "SCRATCH_1_ANCILLA": "single-use ancilla coordinate scratch; source CRTNR alias is not behavior-specific here",
}
PRIVATE_CONST_RE = re.compile(
    r"^\s*const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9A-Fa-f]+)\s*;",
    re.MULTILINE,
)
RAM_MODULE_CONST_RE = re.compile(
    r"^\s*pub(?:\([^)]*\))?\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9A-Fa-f]+)\s*;",
    re.MULTILINE,
)


@dataclass(frozen=True)
class Finding:
    path: Path
    line: int
    message: str


@dataclass(frozen=True)
class RamConst:
    path: Path
    line: int
    name: str
    address: int


@dataclass(frozen=True)
class SemanticAccessor:
    name: str
    return_type: str
    kind: str


@dataclass(frozen=True)
class SemanticAccessorUse:
    path: Path
    line: int
    accessor: SemanticAccessor
    method: str


def rust_files() -> list[Path]:
    return sorted(SRC_ROOT.rglob("*.rs"))


def is_semantic_view_source(path: Path) -> bool:
    return path.is_relative_to(SRC_ROOT / "game_state" / "view")


def is_game_state_access_layer(path: Path) -> bool:
    return (
        is_semantic_view_source(path)
        or path == SRC_ROOT / "game_state" / "native.rs"
        or path.is_relative_to(SRC_ROOT / "game_state" / "native")
    )


def is_public_ram_constant_registry(path: Path) -> bool:
    return path == SRC_ROOT / "game_state" / "constants.rs"


def is_frame_view_source(path: Path) -> bool:
    return path == SRC_ROOT / "game_state" / "view" / "frame.rs"


def is_frame_projection_source(path: Path) -> bool:
    return (
        is_frame_view_source(path)
        or path == SRC_ROOT / "game_state" / "native" / "frame.rs"
        or path == SRC_ROOT / "game_state" / "constants.rs"
    )


def is_world_location_projection_source(path: Path) -> bool:
    return (
        path == SRC_ROOT / "game_state" / "native" / "world.rs"
        or path == SRC_ROOT / "game_state" / "constants.rs"
    )


def is_display_core_projection_source(path: Path) -> bool:
    return (
        path == SRC_ROOT / "game_state" / "native" / "display.rs"
        or path == SRC_ROOT / "game_state" / "constants.rs"
    )


def is_display_projection_source(path: Path) -> bool:
    return is_display_core_projection_source(path)


def is_non_address_sized_constant(name: str) -> bool:
    return name.endswith("_BYTES") or name.endswith("_COUNT") or name.endswith("_COUNT_LIMIT")


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def is_inside_simple_string_literal(text: str, offset: int) -> bool:
    line_start = text.rfind("\n", 0, offset) + 1
    prefix = text[line_start:offset]
    escaped = False
    in_string = False
    for char in prefix:
        if escaped:
            escaped = False
        elif char == "\\":
            escaped = True
        elif char == '"':
            in_string = not in_string
    return in_string


def enclosing_function_name(text: str, offset: int) -> str | None:
    before = text[:offset]
    match = None
    for match in re.finditer(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?fn\s+([A-Za-z0-9_]+)\s*\(",
        before,
        re.MULTILINE,
    ):
        pass
    return match.group(1) if match else None


def check_file(path: Path) -> list[Finding]:
    text = path.read_text()
    findings: list[Finding] = []
    checks = [
        (ADDRESS_NAME_RE, "address-derived RAM constant name"),
        (C_ADDRESS_NAME_RE, "C-style address-derived RAM name in Rust source"),
    ]
    if not is_game_state_access_layer(path):
        checks.append((DIRECT_RAM_RE, "direct hex RAM access; use a named constant"))
    for pattern, message in checks:
        for match in pattern.finditer(text):
            findings.append(Finding(path, line_for_offset(text, match.start()), message))
    for match in NATIVE_TRANSITION_MUT_RE.finditer(text):
        name = match.group(0)
        if name in ALLOWED_NATIVE_TRANSITION_MUT_NAMES or name.endswith("BridgeMut"):
            continue
        findings.append(
            Finding(
                path,
                line_for_offset(text, match.start()),
                f"native transition wrapper should be named BridgeMut: {name}",
            )
        )
    for match in NATIVE_READ_ACCESSOR_RE.finditer(text):
        findings.append(
            Finding(
                path,
                line_for_offset(text, match.start()),
                "native-owned state read accessor; read the game_state path directly",
            )
        )
    for match in MIGRATED_SINGLE_ARG_NATIVE_ACCESSOR_RE.finditer(text):
        findings.append(
            Finding(
                path,
                line_for_offset(text, match.start()),
                "migrated native slot accessor shim; use the game_state slot path directly",
            )
        )
    if not is_frame_view_source(path):
        for match in FRAME_RAM_VIEW_RE.finditer(text):
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    "byte-backed frame view is compatibility-only; use game_state.frame or the native frame bridge",
                )
            )
    if not is_frame_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            frame_name = FRAME_CONTROL_ADDRESSES.get(address)
            if frame_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private frame-control RAM alias {name}@0x{address:04x}; "
                    f"use game_state.frame or canonical {frame_name}",
                )
            )
    if not is_world_location_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            world_name = WORLD_LOCATION_ADDRESSES.get(address)
            if world_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private world-location RAM alias {name}@0x{address:04x}; "
                    f"use game_state.world.location or canonical {world_name}",
                )
            )
    if not is_display_core_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_CORE_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display-core RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display or canonical {display_name}",
                )
            )
    if not is_display_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_UPLOAD_METADATA_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display upload-metadata RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display or canonical {display_name}",
                )
            )
    if not is_display_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_CONTROL_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display-control RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display or canonical {display_name}",
                )
            )
    if not is_display_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_PALETTE_FILTER_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display palette-filter RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display.palette_filter or canonical {display_name}",
                )
            )
    if not is_display_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_LINK_DMA_SOURCE_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display Link DMA source RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display.link_dma_source or canonical {display_name}",
                )
            )
    if not is_display_projection_source(path):
        for match in PRIVATE_CONST_RE.finditer(text):
            name = match.group(1)
            address = int(match.group(2), 16)
            if is_non_address_sized_constant(name):
                continue
            display_name = DISPLAY_ANIMATION_MESSAGE_ADDRESSES.get(address)
            if display_name is None:
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    f"private display animation/message RAM alias {name}@0x{address:04x}; "
                    f"use game_state.display or canonical {display_name}",
                )
            )
    return findings


def direct_ram_findings() -> list[Finding]:
    findings: list[Finding] = []
    for path in rust_files():
        if is_game_state_access_layer(path):
            continue
        text = path.read_text()
        for match in DIRECT_RAM_ANY_RE.finditer(text):
            if is_inside_simple_string_literal(text, match.start()):
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    "direct RAM access outside semantic view layer",
                )
            )
    return findings


ALLOWED_NATIVE_BRIDGE_FUNCTIONS = {
    "set_rom_startup_timing",
    "run_frame_internal",
    "emu_sync_memory_region",
    "load_snes_state",
    "save_snes_state",
    "state_recorder_read_next_replay_state",
    "zelda_run_frame",
    "state_recoder_multi_patch_patch",
    "zelda_initialization_code",
    "startup_initialize_memory",
    "intro_clear1kb_blocks_of_wram",
    "move_link_coord",
    "move_link_coord_subpixel_delta",
    "decrement_word",
    "ram_byte",
    "set_ram_byte",
    "copy_to_ram",
    "fill_ram",
    "read_u8_ram",
    "write_u8_ram",
    "read_u16_ram",
    "write_u16_ram",
    "read_u32_ram",
    "write_u32_ram",
}


def is_allowed_native_bridge_use(path: Path, text: str, offset: int) -> bool:
    if is_game_state_access_layer(path):
        return True
    if is_inside_simple_string_literal(text, offset):
        return True
    fn_name = enclosing_function_name(text, offset)
    if path.name != "zelda_rtl.rs" and fn_name != "intro_clear1kb_blocks_of_wram":
        return False
    return fn_name in ALLOWED_NATIVE_BRIDGE_FUNCTIONS


def native_bridge_findings() -> list[Finding]:
    findings: list[Finding] = []
    for path in rust_files():
        text = path.read_text()
        for match in NATIVE_BRIDGE_RE.finditer(text):
            if is_allowed_native_bridge_use(path, text, match.start()):
                continue
            findings.append(
                Finding(
                    path,
                    line_for_offset(text, match.start()),
                    "native RAM bridge use outside approved bridge boundary",
                )
            )
    return findings


def classify_semantic_accessor(return_type: str) -> str | None:
    compact = " ".join(return_type.split())
    if compact.startswith("Ram"):
        return "ram-backed-view"
    if compact.startswith("Native") and "BridgeMut" in compact:
        return "native-bridge-mutator"
    if compact.startswith("&") and ("State" in compact or "Read" in compact):
        return "native-read-helper"
    if compact.endswith("State"):
        return "native-copy-helper"
    return None


def semantic_accessors() -> dict[str, SemanticAccessor]:
    text = (SRC_ROOT / "zelda_rtl.rs").read_text()
    accessors: dict[str, SemanticAccessor] = {}
    pattern = re.compile(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+([a-z][A-Za-z0-9_]*)\s*"
        r"\([^)]*\)\s*->\s*([^{]+?)\s*\{",
        re.MULTILINE | re.DOTALL,
    )
    for match in pattern.finditer(text):
        name = match.group(1)
        return_type = " ".join(match.group(2).split())
        kind = classify_semantic_accessor(return_type)
        if kind is None:
            continue
        accessors[name] = SemanticAccessor(name, return_type, kind)
    return accessors


def semantic_accessor_uses() -> list[SemanticAccessorUse]:
    accessors = semantic_accessors()
    if not accessors:
        return []
    accessor_names = "|".join(re.escape(name) for name in sorted(accessors, key=len, reverse=True))
    chained_pattern = re.compile(
        rf"\bself\s*\.\s*({accessor_names})\s*"
        r"\([^;\n]*?\)\s*\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        re.MULTILINE,
    )
    alias_pattern = re.compile(
        rf"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"
        rf"self\s*\.\s*({accessor_names})\s*\([^;\n]*\)\s*;",
        re.MULTILINE,
    )
    uses: list[SemanticAccessorUse] = []
    for path in rust_files():
        if is_game_state_access_layer(path):
            continue
        text = path.read_text()
        for match in chained_pattern.finditer(text):
            if is_inside_simple_string_literal(text, match.start()):
                continue
            accessor = accessors[match.group(1)]
            uses.append(
                SemanticAccessorUse(
                    path,
                    line_for_offset(text, match.start()),
                    accessor,
                    match.group(2),
                )
            )
        for match in alias_pattern.finditer(text):
            if is_inside_simple_string_literal(text, match.start()):
                continue
            accessor = accessors[match.group(2)]
            uses.append(
                SemanticAccessorUse(
                    path,
                    line_for_offset(text, match.start()),
                    accessor,
                    "<alias>",
                )
            )
    return sorted(
        uses,
        key=lambda use: (use.accessor.kind, use.accessor.name, use.method, str(use.path), use.line),
    )


def native_read_helper_findings() -> list[SemanticAccessorUse]:
    return [use for use in semantic_accessor_uses() if use.accessor.kind == "native-read-helper"]


def native_copy_helper_findings() -> list[SemanticAccessorUse]:
    return [use for use in semantic_accessor_uses() if use.accessor.kind == "native-copy-helper"]


def non_player_ram_backed_view_findings() -> list[SemanticAccessorUse]:
    allowed_player_views = {"player_state", "player_state_mut"}
    return [
        use
        for use in semantic_accessor_uses()
        if use.accessor.kind == "ram-backed-view" and use.accessor.name not in allowed_player_views
    ]


def print_native_read_helper_failures(uses: list[SemanticAccessorUse], limit: int = 40) -> None:
    for use in uses[:limit]:
        rel = use.path.relative_to(REPO_ROOT)
        print(
            f"{rel}:{use.line}: native-owned state read helper "
            f"{use.accessor.name}().{use.method}(); read the game_state path directly",
            file=sys.stderr,
        )
    if len(uses) > limit:
        print(
            f"... {len(uses) - limit} more native-owned state read helper use(s); "
            "rerun with --report-migration-candidates --migration-candidate-kind native-read-helper",
            file=sys.stderr,
        )


def print_native_copy_helper_failures(uses: list[SemanticAccessorUse], limit: int = 40) -> None:
    for use in uses[:limit]:
        rel = use.path.relative_to(REPO_ROOT)
        print(
            f"{rel}:{use.line}: native-owned copy helper "
            f"{use.accessor.name}().{use.method}(); use native game_state ownership directly",
            file=sys.stderr,
        )
    if len(uses) > limit:
        print(
            f"... {len(uses) - limit} more native-owned copy helper use(s); "
            "rerun with --report-migration-candidates --migration-candidate-kind native-copy-helper",
            file=sys.stderr,
        )


def print_non_player_ram_backed_view_failures(uses: list[SemanticAccessorUse], limit: int = 40) -> None:
    for use in uses[:limit]:
        rel = use.path.relative_to(REPO_ROOT)
        print(
            f"{rel}:{use.line}: non-player byte-backed semantic view "
            f"{use.accessor.name}().{use.method}(); use native game_state or a native bridge",
            file=sys.stderr,
        )
    if len(uses) > limit:
        print(
            f"... {len(uses) - limit} more non-player byte-backed semantic view use(s); "
            "rerun with --report-migration-candidates --migration-candidate-kind ram-backed-view",
            file=sys.stderr,
        )


def print_semantic_migration_candidates(
    limit: int,
    kinds: list[str] | None,
    accessor_filter: str | None,
    exclude_accessor_filter: str | None,
    path_filter: str | None,
    exclude_path_filter: str | None,
    output_format: str,
) -> int:
    uses = semantic_accessor_uses()
    if kinds:
        allowed_kinds = set(kinds)
        uses = [use for use in uses if use.accessor.kind in allowed_kinds]
    if accessor_filter:
        accessor_re = re.compile(accessor_filter)
        uses = [use for use in uses if accessor_re.search(use.accessor.name)]
    if exclude_accessor_filter:
        exclude_accessor_re = re.compile(exclude_accessor_filter)
        uses = [use for use in uses if not exclude_accessor_re.search(use.accessor.name)]
    if path_filter:
        path_re = re.compile(path_filter)
        uses = [use for use in uses if path_re.search(str(use.path.relative_to(REPO_ROOT)))]
    if exclude_path_filter:
        exclude_path_re = re.compile(exclude_path_filter)
        uses = [use for use in uses if not exclude_path_re.search(str(use.path.relative_to(REPO_ROOT)))]

    by_accessor: dict[tuple[str, str, str], list[SemanticAccessorUse]] = {}
    for use in uses:
        key = (use.accessor.kind, use.accessor.name, use.method)
        by_accessor.setdefault(key, []).append(use)

    priority = {
        "ram-backed-view": 0,
        "native-read-helper": 1,
        "native-copy-helper": 2,
        "native-bridge-mutator": 3,
    }
    rows = sorted(
        by_accessor.items(),
        key=lambda item: (
            priority.get(item[0][0], 99),
            -len(item[1]),
            item[0][1],
            item[0][2],
        ),
    )
    total = sum(len(group) for _, group in rows)
    filters = []
    if kinds:
        filters.append(f"kind={','.join(kinds)}")
    if accessor_filter:
        filters.append(f"accessor=/{accessor_filter}/")
    if exclude_accessor_filter:
        filters.append(f"exclude-accessor=/{exclude_accessor_filter}/")
    if path_filter:
        filters.append(f"path=/{path_filter}/")
    if exclude_path_filter:
        filters.append(f"exclude-path=/{exclude_path_filter}/")
    suffix = f" ({'; '.join(filters)})" if filters else ""

    if output_format == "json":
        shown = rows if limit <= 0 else rows[:limit]
        payload = {
            "filters": {
                "kinds": kinds or [],
                "accessor": accessor_filter,
                "exclude_accessor": exclude_accessor_filter,
                "path": path_filter,
                "exclude_path": exclude_path_filter,
            },
            "total_uses": total,
            "total_groups": len(rows),
            "groups": [
                {
                    "uses": len(group),
                    "kind": kind,
                    "accessor": accessor,
                    "method": method,
                    "return_type": group[0].accessor.return_type,
                    "first_path": str(group[0].path.relative_to(REPO_ROOT)),
                    "first_line": group[0].line,
                }
                for (kind, accessor, method), group in shown
            ],
        }
        print(json.dumps(payload, indent=2))
        return total

    print(f"semantic migration candidates{suffix}: {total} accessor use(s), {len(rows)} grouped call pattern(s)")
    if not rows:
        return 0
    shown = rows if limit <= 0 else rows[:limit]
    for (kind, accessor, method), group in shown:
        sample = group[0]
        rel = sample.path.relative_to(REPO_ROOT)
        print(f"{len(group):4} {kind:22} {accessor}().{method}() first={rel}:{sample.line}")
    if len(shown) < len(rows):
        print(f"... {len(rows) - len(shown)} more grouped pattern(s); pass --migration-candidate-limit 0 to show all")
    return total


def print_top_accessor_methods(
    title: str,
    uses: list[SemanticAccessorUse],
    limit: int,
) -> None:
    print(f"{title}: {len(uses)} use(s)")
    if not uses:
        return
    grouped: dict[tuple[str, str, str], list[SemanticAccessorUse]] = {}
    for use in uses:
        grouped.setdefault((use.accessor.kind, use.accessor.name, use.method), []).append(use)
    rows = sorted(
        grouped.items(),
        key=lambda item: (-len(item[1]), item[0][1], item[0][2]),
    )
    shown = rows if limit <= 0 else rows[:limit]
    for (_kind, accessor, method), group in shown:
        sample = group[0]
        rel = sample.path.relative_to(REPO_ROOT)
        print(f"  {len(group):4} {accessor}().{method}() first={rel}:{sample.line}")
    if len(shown) < len(rows):
        print(f"  ... {len(rows) - len(shown)} more grouped pattern(s); pass --migration-progress-limit 0")


def print_semantic_migration_progress(limit: int) -> int:
    """Summarize migration state by authority class.

    This intentionally does not treat dual-write bridge mutators as the same
    kind of work as byte-backed views. Bridge use is expected while RAM is still
    being projected for parity; native read/copy helpers and non-player
    byte-backed views are regressions or cleanup targets.
    """

    uses = semantic_accessor_uses()
    native_reads = [use for use in uses if use.accessor.kind == "native-read-helper"]
    native_copies = [use for use in uses if use.accessor.kind == "native-copy-helper"]
    ram_backed = [use for use in uses if use.accessor.kind == "ram-backed-view"]
    non_player_ram = [
        use for use in ram_backed if use.accessor.name not in {"player_state", "player_state_mut"}
    ]
    player_ram = [
        use for use in ram_backed if use.accessor.name in {"player_state", "player_state_mut"}
    ]
    bridge_mutators = [use for use in uses if use.accessor.kind == "native-bridge-mutator"]

    print("semantic migration progress")
    print(f"  native read helper cleanup: {len(native_reads)} use(s)")
    print(f"  native copy helper cleanup: {len(native_copies)} use(s)")
    print(f"  non-player byte-backed views: {len(non_player_ram)} use(s)")
    print(f"  player byte-backed compatibility views: {len(player_ram)} use(s)")
    print(f"  dual-write native bridge mutators: {len(bridge_mutators)} use(s)")
    print()
    print_top_accessor_methods("player byte-backed compatibility backlog", player_ram, limit)
    print()
    print_top_accessor_methods("dual-write bridge usage, expected during transition", bridge_mutators, limit)
    return len(uses)


def print_actionable_semantic_migration(limit: int, output_format: str) -> int:
    """Print migration work that should be actively driven down now.

    Dual-write native bridge mutators are intentionally excluded. They are the
    transition mechanism while native state still projects to RAM; call sites
    using those APIs are not byte-backed-view debt.
    """

    uses = semantic_accessor_uses()
    actionable = [
        use
        for use in uses
        if use.accessor.kind
        in {
            "native-read-helper",
            "native-copy-helper",
            "ram-backed-view",
        }
    ]

    by_accessor: dict[tuple[str, str, str], list[SemanticAccessorUse]] = {}
    for use in actionable:
        key = (use.accessor.kind, use.accessor.name, use.method)
        by_accessor.setdefault(key, []).append(use)

    priority = {
        "native-read-helper": 0,
        "native-copy-helper": 1,
        "ram-backed-view": 2,
    }
    rows = sorted(
        by_accessor.items(),
        key=lambda item: (
            priority.get(item[0][0], 99),
            -len(item[1]),
            item[0][1],
            item[0][2],
        ),
    )
    total = sum(len(group) for _, group in rows)

    if output_format == "json":
        shown = rows if limit <= 0 else rows[:limit]
        payload = {
            "total_uses": total,
            "total_groups": len(rows),
            "groups": [
                {
                    "uses": len(group),
                    "kind": kind,
                    "accessor": accessor,
                    "method": method,
                    "return_type": group[0].accessor.return_type,
                    "first_path": str(group[0].path.relative_to(REPO_ROOT)),
                    "first_line": group[0].line,
                }
                for (kind, accessor, method), group in shown
            ],
        }
        print(json.dumps(payload, indent=2))
        return total

    print(
        "actionable semantic migration backlog "
        f"(excludes expected dual-write bridge mutators): {total} accessor use(s), "
        f"{len(rows)} grouped call pattern(s)"
    )
    shown = rows if limit <= 0 else rows[:limit]
    for (kind, accessor, method), group in shown:
        sample = group[0]
        rel = sample.path.relative_to(REPO_ROOT)
        print(f"{len(group):4} {kind:22} {accessor}().{method}() first={rel}:{sample.line}")
    if len(shown) < len(rows):
        print(
            f"... {len(rows) - len(shown)} more grouped pattern(s); "
            "pass --migration-candidate-limit 0 to show all"
        )
    return total


def actionable_semantic_migration_uses() -> list[SemanticAccessorUse]:
    return [
        use
        for use in semantic_accessor_uses()
        if use.accessor.kind
        in {
            "native-read-helper",
            "native-copy-helper",
            "ram-backed-view",
        }
    ]


def scan_consts() -> list[RamConst]:
    constants: list[RamConst] = []
    for path in rust_files():
        text = path.read_text()
        patterns = [PRIVATE_CONST_RE]
        if is_public_ram_constant_registry(path):
            patterns.append(RAM_MODULE_CONST_RE)
        for pattern in patterns:
            for match in pattern.finditer(text):
                constants.append(
                    RamConst(
                        path=path,
                        line=line_for_offset(text, match.start()),
                        name=match.group(1),
                        address=int(match.group(2), 16),
                    )
                )
    return sorted(constants, key=lambda c: (c.address, str(c.path), c.name))


def const_offset(path: Path, name: str) -> int | None:
    text = path.read_text()
    patterns = [PRIVATE_CONST_RE]
    if is_public_ram_constant_registry(path):
        patterns.append(RAM_MODULE_CONST_RE)
    for pattern in patterns:
        for match in pattern.finditer(text):
            if match.group(1) == name:
                return match.start()
    return None


def module_for_line(text: str, offset: int) -> str | None:
    before = text[:offset]
    module_match = None
    for match in re.finditer(r"^\s*pub(?:\([^)]*\))?\s+mod\s+([a-z][a-z0-9_]*)\s*\{", before, re.MULTILINE):
        module_match = match
    return module_match.group(1) if module_match else None


def subsystem_from_name(name: str, default: str = "game_state") -> str:
    prefix = name.split("_", 1)[0].lower()
    if prefix in {"link", "player", "swim", "swimcoll", "tiledetect", "button", "cape", "pit"}:
        return "player"
    if prefix in {"sprite", "overlord", "garnish", "oam", "alt", "cached", "enemy", "prize"}:
        return "sprite"
    if prefix in {"dung", "dungeon", "door", "room", "crush", "invisible", "big"}:
        return "dungeon"
    if prefix in {"message", "messaging", "dialogue", "text", "vwf", "select"}:
        return "messaging"
    if prefix in {"overworld", "ow", "map16", "camera", "quadrant", "bird", "bg1", "bg2"}:
        return "overworld"
    if prefix in {"nmi", "tm", "ts", "tmw", "tsw", "bgmode", "mosaic", "w12sel", "w34sel", "wobjsel", "hdmaen", "vram", "palette", "cgram", "hud", "spotlight", "water"}:
        return "display"
    if prefix == "poly":
        return "poly"
    if prefix in {"attract", "intro", "ending"}:
        return "attract"
    return default


def subsystem_for(path: Path, name: str) -> str:
    stem = path.stem
    if is_public_ram_constant_registry(path):
        text = path.read_text()
        offset = const_offset(path, name)
        if offset is not None and offset < text.find("// Source addresses"):
            return module_for_line(text, offset) or "game_state"
        return subsystem_from_name(name)
    if stem == "zelda_rtl":
        return subsystem_from_name(name, default="shared")
    return stem


def markdown_cell(value: object) -> str:
    return str(value).replace("\n", " ").replace("|", "\\|")


def markdown_link_to_source(path: Path, line: int) -> str:
    rel = path.relative_to(REPO_ROOT)
    target = f"../{rel}#L{line}"
    return f"[`{rel}:{line}`]({target})"


def load_source_crosswalk(path: Path = DEFAULT_NES_VER2_CROSSWALK) -> dict[tuple[str, str, int], dict[str, object]]:
    if not path.exists():
        return {}
    rows = json.loads(path.read_text())
    crosswalk: dict[tuple[str, str, int], dict[str, object]] = {}
    for row in rows:
        key = (str(row["rust_path"]), str(row["rust_name"]), int(row["address"]))
        crosswalk[key] = row
    return crosswalk


def confidence_for(const: RamConst, source: dict[str, object], aliases_by_addr: dict[int, int]) -> str:
    if not source:
        return "no-source"
    name = const.name
    source_label = str(source.get("source_label", ""))
    source_comment = str(source.get("source_comment_en", ""))
    source_label_intuitive = bool(source.get("source_label_intuitive"))
    source_label_cryptic = bool(source.get("source_label_cryptic"))
    source_comment_useful = bool(source.get("source_comment_useful"))
    weak = bool(WEAK_NAME_RE.search(name))

    if weak and (source_label_intuitive or source_comment_useful):
        return "source-backed-weak"
    if name == source_label:
        return "exact-source"
    if source_comment and source_comment.replace("-", "_").upper() in name:
        return "exact-source"
    if aliases_by_addr[const.address] > 1:
        return "source-backed-contextual"
    if source_label_cryptic:
        return "good"
    return "good"


def ram_table(constants: list[RamConst], source_crosswalk: dict[tuple[str, str, int], dict[str, object]], aliases_by_addr: dict[int, int]) -> list[str]:
    lines = [
        "| Address | Rust name | Subsystem | Confidence | Defined in | NES_Ver2 label | Original comment | US-English hint | Source |",
        "|---:|---|---|---|---|---|---|---|---|",
    ]
    for const in constants:
        rel = const.path.relative_to(REPO_ROOT)
        subsystem = subsystem_for(const.path, const.name)
        source = source_crosswalk.get((str(rel), const.name, const.address), {})
        confidence = confidence_for(const, source, aliases_by_addr)
        source_label = source.get("source_label", "")
        source_comment = source.get("source_comment", "")
        source_comment_en = source.get("source_comment_en", "")
        source_path = ""
        if source:
            source_path = f"{source['source_path']}:{source['source_line']}"
        lines.append(
            "| "
            + " | ".join(
                [
                    f"`0x{const.address:05x}`",
                    f"`{const.name}`",
                    markdown_cell(subsystem),
                    confidence,
                    markdown_link_to_source(const.path, const.line),
                    f"`{markdown_cell(source_label)}`" if source_label else "",
                    markdown_cell(source_comment),
                    markdown_cell(source_comment_en),
                    f"`{markdown_cell(source_path)}`" if source_path else "",
                ]
            )
            + " |"
        )
    lines.append("")
    return lines


def generate_doc(constants: list[RamConst]) -> str:
    source_crosswalk = load_source_crosswalk()
    aliases_by_addr: dict[int, int] = {}
    for const in constants:
        aliases_by_addr[const.address] = aliases_by_addr.get(const.address, 0) + 1
    constants_by_subsystem: dict[str, list[RamConst]] = {}
    for const in constants:
        constants_by_subsystem.setdefault(subsystem_for(const.path, const.name), []).append(const)

    lines = [
        "# Zelda RAM Map",
        "",
        "Generated by `python3 scripts/check_ram_readability.py --write-doc`.",
        "Addresses are byte offsets into `ZeldaState::ram` unless the name says otherwise.",
        "NES_Ver2 evidence is joined from `docs/nes-ver2/ram_symbol_crosswalk.json`; regenerate it with `python3 scripts/mine_nes_ver2_symbols.py --write` after changing RAM names.",
        "",
        "## Confidence Legend",
        "",
        "- `exact-source`: Rust name directly matches or clearly restates source evidence.",
        "- `good`: Rust name is readable and source evidence does not suggest a better one.",
        "- `source-backed-contextual`: address has multiple Rust aliases; context decides the best name.",
        "- `source-backed-weak`: weak Rust name has useful source evidence and should be manually reviewed.",
        "- `no-source`: no NES_Ver2 evidence was found for this Rust constant.",
        "",
        "## Grouped by Subsystem",
        "",
    ]
    for subsystem in sorted(constants_by_subsystem):
        lines.extend([f"### {subsystem}", ""])
        lines.extend(ram_table(constants_by_subsystem[subsystem], source_crosswalk, aliases_by_addr))
    lines.extend(["## Full Address Order", ""])
    lines.extend(ram_table(constants, source_crosswalk, aliases_by_addr))
    return "\n".join(lines)


def weak_name_findings(constants: list[RamConst]) -> list[Finding]:
    findings: list[Finding] = []
    for const in constants:
        if const.name in REVIEWED_WEAK_RAM_NAMES:
            continue
        if WEAK_NAME_RE.search(const.name):
            findings.append(Finding(const.path, const.line, f"weak RAM name: {const.name}"))
    return findings


def generate_weak_doc(findings: list[Finding]) -> str:
    lines = [
        "# Weak RAM Name Backlog",
        "",
        "Generated by `python3 scripts/check_ram_readability.py --write-weak-doc`.",
        "These names are warnings, not failures. Rename one only when surrounding behavior or source context proves a better name.",
        "",
        "| File | Line | Name |",
        "|---|---:|---|",
    ]
    for finding in findings:
        rel = finding.path.relative_to(REPO_ROOT)
        name = finding.message.removeprefix("weak RAM name: ")
        lines.append(f"| `{rel}` | {finding.line} | `{name}` |")
    lines.append("")
    return "\n".join(lines)


def usage_count(const: RamConst) -> int:
    pattern = re.compile(rf"\b{re.escape(const.name)}\b")
    count = 0
    for path in rust_files():
        count += len(pattern.findall(path.read_text()))
    return max(0, count - 1)


def candidate_sort_key(row: tuple[RamConst, dict[str, object], str]) -> tuple[str, int, str]:
    const, source, confidence = row
    priority = 0 if confidence == "source-backed-weak" else 1
    return (subsystem_for(const.path, const.name), priority, const.address, const.name)


def source_backed_candidates(constants: list[RamConst]) -> list[tuple[RamConst, dict[str, object], str]]:
    source_crosswalk = load_source_crosswalk()
    aliases_by_addr: dict[int, int] = {}
    for const in constants:
        aliases_by_addr[const.address] = aliases_by_addr.get(const.address, 0) + 1

    rows: list[tuple[RamConst, dict[str, object], str]] = []
    for const in constants:
        if const.name in REVIEWED_WEAK_RAM_NAMES:
            continue
        if not WEAK_NAME_RE.search(const.name):
            continue
        rel = const.path.relative_to(REPO_ROOT)
        source = source_crosswalk.get((str(rel), const.name, const.address), {})
        if not source:
            continue
        if not (source.get("source_label_intuitive") or source.get("source_comment_useful")):
            continue
        rows.append((const, source, confidence_for(const, source, aliases_by_addr)))
    return sorted(rows, key=candidate_sort_key)


def reviewed_source_backed_weak_names(
    constants: list[RamConst],
) -> list[tuple[RamConst, dict[str, object], str]]:
    source_crosswalk = load_source_crosswalk()
    aliases_by_addr: dict[int, int] = {}
    for const in constants:
        aliases_by_addr[const.address] = aliases_by_addr.get(const.address, 0) + 1

    rows: list[tuple[RamConst, dict[str, object], str]] = []
    for const in constants:
        if const.name not in REVIEWED_WEAK_RAM_NAMES:
            continue
        rel = const.path.relative_to(REPO_ROOT)
        source = source_crosswalk.get((str(rel), const.name, const.address), {})
        if not source:
            continue
        rows.append((const, source, confidence_for(const, source, aliases_by_addr)))
    return sorted(rows, key=candidate_sort_key)


def generate_candidate_doc(constants: list[RamConst]) -> str:
    rows = source_backed_candidates(constants)
    reviewed_rows = reviewed_source_backed_weak_names(constants)
    lines = [
        "# Source-Backed RAM Rename Candidates",
        "",
        "Generated by `python3 scripts/check_ram_readability.py --write-candidate-doc`.",
        "These are weak Rust RAM names where NES_Ver2 provides a useful label or comment.",
        "Rename only after checking call sites and confirming the source evidence matches actual Rust usage.",
        "",
        "| Address | Rust name | Subsystem | Confidence | Uses | Defined in | NES_Ver2 label | US-English hint | Source |",
        "|---:|---|---|---|---:|---|---|---|---|",
    ]
    for const, source, confidence in rows:
        source_path = f"{source['source_path']}:{source['source_line']}"
        lines.append(
            "| "
            + " | ".join(
                [
                    f"`0x{const.address:05x}`",
                    f"`{const.name}`",
                    markdown_cell(subsystem_for(const.path, const.name)),
                    confidence,
                    str(usage_count(const)),
                    markdown_link_to_source(const.path, const.line),
                    f"`{markdown_cell(source.get('source_label', ''))}`",
                    markdown_cell(source.get("source_comment_en", "")),
                    f"`{markdown_cell(source_path)}`",
                ]
            )
            + " |"
        )
    lines.append("")
    lines.append(f"Total candidates: {len(rows)}")
    if reviewed_rows:
        lines.extend(
            [
                "",
                "## Reviewed and Intentionally Left Weak",
                "",
                "These names were checked against call sites and NES_Ver2 evidence. They remain weak because a more specific name would overstate what the shared work RAM means.",
                "",
                "| Address | Rust name | Subsystem | Uses | Defined in | NES_Ver2 label | US-English hint | Reason |",
                "|---:|---|---|---:|---|---|---|---|",
            ]
        )
        for const, source, _confidence in reviewed_rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        f"`0x{const.address:05x}`",
                        f"`{const.name}`",
                        markdown_cell(subsystem_for(const.path, const.name)),
                        str(usage_count(const)),
                        markdown_link_to_source(const.path, const.line),
                        f"`{markdown_cell(source.get('source_label', ''))}`",
                        markdown_cell(source.get("source_comment_en", "")),
                        markdown_cell(REVIEWED_WEAK_RAM_NAMES[const.name]),
                    ]
                )
                + " |"
            )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write-doc", action="store_true", help=f"write {DEFAULT_DOC.relative_to(REPO_ROOT)}")
    parser.add_argument("--doc", type=Path, default=DEFAULT_DOC, help="RAM map output path")
    parser.add_argument(
        "--write-weak-doc",
        action="store_true",
        help=f"write {DEFAULT_WEAK_DOC.relative_to(REPO_ROOT)}",
    )
    parser.add_argument("--weak-doc", type=Path, default=DEFAULT_WEAK_DOC, help="weak-name backlog output path")
    parser.add_argument(
        "--write-candidate-doc",
        action="store_true",
        help=f"write {DEFAULT_CANDIDATE_DOC.relative_to(REPO_ROOT)}",
    )
    parser.add_argument("--candidate-doc", type=Path, default=DEFAULT_CANDIDATE_DOC, help="source-backed candidate output path")
    parser.add_argument(
        "--warn-weak-names",
        action="store_true",
        help="print non-failing warnings for UNK/SOME/VAR/TMP/SCRATCH RAM names",
    )
    parser.add_argument(
        "--report-direct-ram",
        action="store_true",
        help="print non-failing direct RAM access sites outside game_state/view",
    )
    parser.add_argument(
        "--report-native-bridge",
        action="store_true",
        help="print non-failing NativeRamBridgeView use outside approved runtime bridge boundaries",
    )
    parser.add_argument(
        "--report-migration-candidates",
        action="store_true",
        help="print grouped semantic accessor uses that are candidates for native GameState migration",
    )
    parser.add_argument(
        "--report-migration-progress",
        action="store_true",
        help="summarize semantic migration progress by authority class",
    )
    parser.add_argument(
        "--report-actionable-migration",
        action="store_true",
        help="print native migration backlog excluding expected dual-write bridge mutators",
    )
    parser.add_argument(
        "--fail-on-actionable-migration",
        action="store_true",
        help="fail when actionable native migration debt exists; expected dual-write bridge mutators are ignored",
    )
    parser.add_argument(
        "--migration-candidate-limit",
        type=int,
        default=40,
        help="maximum grouped migration candidate rows to print; use 0 for all",
    )
    parser.add_argument(
        "--migration-progress-limit",
        type=int,
        default=12,
        help="maximum grouped rows per migration progress section; use 0 for all",
    )
    parser.add_argument(
        "--migration-candidate-kind",
        action="append",
        choices=[
            "ram-backed-view",
            "native-read-helper",
            "native-copy-helper",
            "native-bridge-mutator",
        ],
        help="limit migration candidates to one accessor kind; repeat for multiple kinds",
    )
    parser.add_argument(
        "--migration-candidate-accessor",
        help="regular expression for accessor/helper names to include",
    )
    parser.add_argument(
        "--migration-candidate-exclude-accessor",
        help="regular expression for accessor/helper names to exclude",
    )
    parser.add_argument(
        "--migration-candidate-path",
        help="regular expression for relative Rust file paths to include",
    )
    parser.add_argument(
        "--migration-candidate-exclude-path",
        help="regular expression for relative Rust file paths to exclude",
    )
    parser.add_argument(
        "--migration-candidate-format",
        choices=["text", "json"],
        default="text",
        help="migration candidate report format",
    )
    args = parser.parse_args()

    findings = [finding for path in rust_files() for finding in check_file(path)]
    if findings:
        for finding in findings:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: {finding.message}", file=sys.stderr)
        return 1

    native_read_helper_uses = native_read_helper_findings()
    if native_read_helper_uses and not args.report_migration_candidates:
        print_native_read_helper_failures(native_read_helper_uses)
        return 1
    native_copy_helper_uses = native_copy_helper_findings()
    if native_copy_helper_uses and not args.report_migration_candidates:
        print_native_copy_helper_failures(native_copy_helper_uses)
        return 1
    non_player_ram_backed_view_uses = non_player_ram_backed_view_findings()
    if non_player_ram_backed_view_uses and not args.report_migration_candidates:
        print_non_player_ram_backed_view_failures(non_player_ram_backed_view_uses)
        return 1

    constants = scan_consts()
    weak_findings = weak_name_findings(constants)
    if args.warn_weak_names and weak_findings:
        for finding in weak_findings:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: warning: {finding.message}", file=sys.stderr)

    direct_findings = direct_ram_findings() if args.report_direct_ram else []
    if direct_findings:
        for finding in direct_findings:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: note: {finding.message}", file=sys.stderr)

    native_bridge_findings_list = native_bridge_findings() if args.report_native_bridge else []
    if native_bridge_findings_list:
        for finding in native_bridge_findings_list:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: note: {finding.message}", file=sys.stderr)

    migration_candidate_count = (
        print_semantic_migration_candidates(
            args.migration_candidate_limit,
            args.migration_candidate_kind,
            args.migration_candidate_accessor,
            args.migration_candidate_exclude_accessor,
            args.migration_candidate_path,
            args.migration_candidate_exclude_path,
            args.migration_candidate_format,
        )
        if args.report_migration_candidates
        else 0
    )
    migration_progress_count = (
        print_semantic_migration_progress(args.migration_progress_limit)
        if args.report_migration_progress
        else 0
    )
    actionable_migration_count = (
        print_actionable_semantic_migration(
            args.migration_candidate_limit,
            args.migration_candidate_format,
        )
        if args.report_actionable_migration
        else 0
    )
    if args.fail_on_actionable_migration:
        actionable_uses = actionable_semantic_migration_uses()
        actionable_migration_count = len(actionable_uses)
        if actionable_uses:
            if not args.report_actionable_migration:
                print_actionable_semantic_migration(
                    args.migration_candidate_limit,
                    args.migration_candidate_format,
                )
            print(
                "actionable semantic migration debt remains; "
                "migrate native read/copy helpers and byte-backed semantic views",
                file=sys.stderr,
            )
            return 1

    if args.write_doc:
        doc_path = args.doc if args.doc.is_absolute() else REPO_ROOT / args.doc
        doc_path.parent.mkdir(parents=True, exist_ok=True)
        doc_path.write_text(generate_doc(constants))
        print(f"wrote {doc_path.relative_to(REPO_ROOT)} ({len(constants)} RAM constants)")

    if args.write_weak_doc:
        weak_doc_path = args.weak_doc if args.weak_doc.is_absolute() else REPO_ROOT / args.weak_doc
        weak_doc_path.parent.mkdir(parents=True, exist_ok=True)
        weak_doc_path.write_text(generate_weak_doc(weak_findings))
        print(f"wrote {weak_doc_path.relative_to(REPO_ROOT)} ({len(weak_findings)} weak-name warning(s))")

    if args.write_candidate_doc:
        candidate_doc_path = args.candidate_doc if args.candidate_doc.is_absolute() else REPO_ROOT / args.candidate_doc
        candidate_doc_path.parent.mkdir(parents=True, exist_ok=True)
        candidate_doc_path.write_text(generate_candidate_doc(constants))
        print(f"wrote {candidate_doc_path.relative_to(REPO_ROOT)} ({len(source_backed_candidates(constants))} source-backed candidate(s))")

    if args.write_doc or args.write_weak_doc or args.write_candidate_doc:
        return 0

    message = f"ram readability ok ({len(constants)} RAM constants)"
    if args.warn_weak_names:
        message += f"; {len(weak_findings)} weak-name warning(s)"
    if args.report_direct_ram:
        message += f"; {len(direct_findings)} non-view direct RAM access note(s)"
    if args.report_native_bridge:
        message += f"; {len(native_bridge_findings_list)} non-boundary native bridge note(s)"
    if args.report_migration_candidates:
        message += f"; {migration_candidate_count} semantic migration candidate accessor use(s)"
    if args.report_migration_progress:
        message += f"; {migration_progress_count} semantic migration accessor use(s)"
    if args.report_actionable_migration:
        message += f"; {actionable_migration_count} actionable semantic migration accessor use(s)"
    if args.fail_on_actionable_migration:
        message += f"; actionable semantic migration guard passed"
    print(message)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
