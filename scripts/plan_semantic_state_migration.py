#!/usr/bin/env python3
"""Plan semantic GameState migration slices from current accessor usage.

This is a planning companion to check_ram_readability.py and
migrate_native_state_access.py. The checker answers "what still exists"; this
script answers "which slice is worth taking next, what methods does it need,
and which codemod command becomes valid once native ownership exists".
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

import check_ram_readability
import migrate_native_state_access


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"
CONSTANTS_RS = SRC_ROOT / "game_state" / "constants.rs"

SLOT_COUNTS_BY_VIEW_PREFIX = {
    "RamAncillaSlot": 10,
    "NativeAncillaSlot": 10,
    "RamGarnishSlot": 30,
    "NativeGarnishSlot": 30,
    "RamOverlordSlot": 16,
    "NativeOverlordSlot": 16,
    "RamSpriteSlot": 16,
    "NativeSpriteSlot": 16,
}


@dataclass(frozen=True)
class AccessorPlan:
    kind: str
    accessor: str
    return_type: str
    uses: int
    methods: Counter[str]
    files: Counter[str]
    first_path: str
    first_line: int


@dataclass(frozen=True)
class ProjectionField:
    name: str
    address: int
    uses: int
    slot_indexed: bool
    width: int


@dataclass(frozen=True)
class ProjectionPlan:
    view_type: str
    path: str
    line: int
    slot_count: int
    fields: list[ProjectionField]
    cluster_count: int
    covered_bytes: int
    span_bytes: int
    gap_bytes: int


@dataclass(frozen=True)
class MethodInfo:
    name: str
    signature: str
    body: str
    line: int


@dataclass(frozen=True)
class NativeFieldInfo:
    field: str
    width: int


@dataclass(frozen=True)
class NativeFieldDraft:
    field: str
    const: str
    width: int
    methods: tuple[str, ...]


@dataclass(frozen=True)
class BridgeReductionPlan:
    accessor: str
    return_type: str
    uses: int
    domain: str
    risk: str
    strategy: str
    first_path: str
    first_line: int
    methods: Counter[str]
    files: Counter[str]


LOW_RISK_BRIDGE_ACCESSORS = {
    "system_signals_mut",
    "palette_buffer_mut",
    "palette_filter_mut",
    "world_scroll_mut",
    "ppu_scroll_copy_mut",
    "display_state_bridge_mut",
    "attract_vram_destination_mut",
}

MEDIUM_RISK_BRIDGE_ACCESSORS = {
    "attract_scene_mut",
    "dungeon_doors_mut",
    "dungeon_room_load_mut",
    "follower_state_mut",
    "hud_state_mut",
    "inventory_items_mut",
    "oam_state_mut",
    "player_resources_mut",
    "tile_detect_position_mut",
    "world_region_mut",
    "world_transient_mut",
}

HIGH_RISK_BRIDGE_ACCESSORS = {
    "ancilla_slot_mut",
    "follower_link_state_mut",
    "garnish_slot_mut",
    "overlord_slot_mut",
    "sprite_slot_mut",
}


def selected_uses(
    kinds: list[str] | None,
    accessor_filter: str | None,
    exclude_accessor_filter: str | None,
    path_filter: str | None,
    exclude_path_filter: str | None,
) -> list[check_ram_readability.SemanticAccessorUse]:
    uses = check_ram_readability.semantic_accessor_uses()
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
        uses = [use for use in uses if path_re.search(relative(use.path))]
    if exclude_path_filter:
        exclude_path_re = re.compile(exclude_path_filter)
        uses = [use for use in uses if not exclude_path_re.search(relative(use.path))]
    return uses


def relative(path: Path) -> str:
    path = path.resolve()
    return str(path.relative_to(REPO_ROOT))


def constants_by_name() -> dict[str, int]:
    text = CONSTANTS_RS.read_text()
    constants: dict[str, int] = {}
    for match in re.finditer(
        r"^\s*pub(?:\([^)]*\))?\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*"
        r"(0x[0-9A-Fa-f]+|\d+)\s*;",
        text,
        re.MULTILINE,
    ):
        constants[match.group(1)] = int(match.group(2), 0)
    return constants


def view_type_name(return_type: str) -> str | None:
    match = re.search(r"\b([A-Z][A-Za-z0-9_]*(?:View|ViewMut|BridgeMut|State|Read))\b", return_type)
    return match.group(1) if match else None


def slot_count_for_view(view_type: str) -> int:
    for prefix, count in SLOT_COUNTS_BY_VIEW_PREFIX.items():
        if view_type.startswith(prefix):
            return count
    return 1


def find_matching_brace(text: str, open_brace: int) -> int | None:
    depth = 0
    for offset in range(open_brace, len(text)):
        char = text[offset]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return offset + 1
    return None


def find_impl_source(view_type: str) -> tuple[Path, int, str] | None:
    impl_re = re.compile(
        rf"\bimpl(?:<[^>]+>)?\s+{re.escape(view_type)}(?:<[^>]+>)?\s*\{{",
        re.MULTILINE,
    )
    for path in sorted((SRC_ROOT / "game_state").rglob("*.rs")):
        text = path.read_text()
        match = impl_re.search(text)
        if not match:
            continue
        open_brace = text.find("{", match.start())
        close = find_matching_brace(text, open_brace)
        if close is None:
            continue
        line = check_ram_readability.line_for_offset(text, match.start())
        return path, line, text[match.start() : close]
    return None


def parse_methods(impl_text: str, impl_start_line: int) -> dict[str, MethodInfo]:
    methods: dict[str, MethodInfo] = {}
    method_re = re.compile(r"(?m)^    (?:pub\(crate\) )?fn ([A-Za-z0-9_]+)\s*\(")
    for match in method_re.finditer(impl_text):
        name = match.group(1)
        signature_start = match.start()
        open_brace = impl_text.find("{", signature_start)
        if open_brace == -1:
            continue
        close_brace = find_matching_brace(impl_text, open_brace)
        if close_brace is None:
            continue
        signature = impl_text[signature_start:open_brace].strip()
        body = impl_text[open_brace + 1 : close_brace - 1]
        line = impl_start_line + impl_text[:signature_start].count("\n")
        methods[name] = MethodInfo(name=name, signature=signature, body=body, line=line)
    return methods


def impl_methods(view_type: str) -> tuple[Path, dict[str, MethodInfo]] | None:
    source = find_impl_source(view_type)
    if source is None:
        return None
    path, line, impl_text = source
    return path, parse_methods(impl_text, line)


def merge_ranges(ranges: list[tuple[int, int]]) -> list[tuple[int, int]]:
    if not ranges:
        return []
    merged: list[tuple[int, int]] = []
    for start, end in sorted(ranges):
        if not merged or start > merged[-1][1]:
            merged.append((start, end))
        else:
            old_start, old_end = merged[-1]
            merged[-1] = (old_start, max(old_end, end))
    return merged


def projection_plan(plan: AccessorPlan) -> ProjectionPlan | None:
    view_type = view_type_name(plan.return_type)
    if view_type is None:
        return None
    source = find_impl_source(view_type)
    if source is None:
        return None

    constants = constants_by_name()
    path, line, impl_text = source
    names = [name for name in re.findall(r"\b[A-Z][A-Z0-9_]{2,}\b", impl_text) if name in constants]
    counts = Counter(names)
    slot_count = slot_count_for_view(view_type)
    fields: list[ProjectionField] = []
    ranges: list[tuple[int, int]] = []
    for name, uses in counts.items():
        slot_stride_2 = bool(
            re.search(rf"\b{re.escape(name)}\s*\+\s*self\.slot\s*\*\s*2\b", impl_text)
            or re.search(rf"\b{re.escape(name)}\s*\+\s*2\s*\*\s*self\.slot\b", impl_text)
        )
        slot_indexed = slot_stride_2 or bool(
            re.search(rf"\b{re.escape(name)}\s*\+\s*self\.slot\b", impl_text)
            or re.search(rf"\bself\.slot\s*\+\s*{re.escape(name)}\b", impl_text)
        )
        width = slot_count * 2 if slot_stride_2 else slot_count if slot_indexed else 1
        address = constants[name]
        fields.append(ProjectionField(name, address, uses, slot_indexed, width))
        ranges.append((address, address + width))

    merged = merge_ranges(ranges)
    covered_bytes = sum(end - start for start, end in merged)
    span_bytes = 0
    if merged:
        span_bytes = merged[-1][1] - merged[0][0]
    return ProjectionPlan(
        view_type=view_type,
        path=relative(path),
        line=line,
        slot_count=slot_count,
        fields=sorted(fields, key=lambda field: (field.address, field.name)),
        cluster_count=len(merged),
        covered_bytes=covered_bytes,
        span_bytes=span_bytes,
        gap_bytes=max(0, span_bytes - covered_bytes),
    )


def accessor_plans(uses: list[check_ram_readability.SemanticAccessorUse]) -> list[AccessorPlan]:
    groups: dict[str, list[check_ram_readability.SemanticAccessorUse]] = defaultdict(list)
    for use in uses:
        groups[use.accessor.name].append(use)

    plans: list[AccessorPlan] = []
    for accessor, group in groups.items():
        first = sorted(group, key=lambda use: (relative(use.path), use.line))[0]
        plans.append(
            AccessorPlan(
                kind=first.accessor.kind,
                accessor=accessor,
                return_type=first.accessor.return_type,
                uses=len(group),
                methods=Counter(use.method for use in group),
                files=Counter(relative(use.path) for use in group),
                first_path=relative(first.path),
                first_line=first.line,
            )
        )
    priority = {
        "ram-backed-view": 0,
        "native-read-helper": 1,
        "native-copy-helper": 2,
        "native-bridge-mutator": 3,
    }
    return sorted(plans, key=lambda plan: (priority.get(plan.kind, 99), -plan.uses, plan.accessor))


def known_single_arg_accessors() -> set[str]:
    return {mapping.accessor for mapping in migrate_native_state_access.SINGLE_ARG_ACCESSORS}


def known_read_accessors() -> set[str]:
    return {mapping.accessor for mapping in migrate_native_state_access.MANUAL_ACCESSORS}


def migration_status(plan: AccessorPlan) -> str:
    if plan.accessor in known_single_arg_accessors():
        return "codemod-ready single-arg native accessor"
    if plan.accessor in known_read_accessors():
        return "codemod-ready native read accessor"
    if plan.kind == "native-read-helper":
        return "likely codemod-ready with --infer-direct-game-state"
    if plan.kind == "ram-backed-view":
        return "needs native owner before call-site codemod"
    if plan.kind == "native-bridge-mutator":
        return "keep until dual-write bridge can be removed"
    return "manual review"


def codemod_command(plan: AccessorPlan) -> str | None:
    escaped = plan.accessor.replace("\\", "\\\\").replace("'", "\\'")
    if plan.accessor in known_single_arg_accessors():
        return (
            "python3 scripts/migrate_native_state_access.py "
            "--include-single-arg-native-accessors "
            f"--accessor-regex '^{escaped}$' --apply"
        )
    if plan.accessor in known_read_accessors():
        return (
            "python3 scripts/migrate_native_state_access.py "
            f"--accessor-regex '^{escaped}$' --apply"
        )
    if plan.kind == "native-read-helper":
        return (
            "python3 scripts/migrate_native_state_access.py "
            "--infer-direct-game-state "
            f"--accessor-regex '^{escaped}$' --apply"
        )
    return None


def bridge_domain(accessor: str) -> str:
    if accessor.startswith("oam"):
        return "display"
    if accessor.startswith("tile_detect") or accessor.startswith("follower_state"):
        return "player"
    if accessor.startswith("sprite") or accessor in {"ancilla_slot_mut", "garnish_slot_mut", "overlord_slot_mut"}:
        return "sprites"
    if accessor.startswith("follower_link") or accessor.startswith("player"):
        return "player"
    if accessor.startswith("dungeon"):
        return "dungeon"
    if accessor.startswith("world") or accessor.startswith("overworld"):
        return "world"
    if accessor.startswith("palette") or accessor.startswith("ppu") or accessor.startswith("display"):
        return "display"
    if accessor.startswith("hud"):
        return "hud"
    if accessor.startswith("inventory"):
        return "inventory"
    if accessor.startswith("system"):
        return "system"
    if accessor.startswith("attract") or accessor.startswith("intro"):
        return "ending"
    return "other"


def bridge_risk(accessor: str) -> str:
    if accessor in HIGH_RISK_BRIDGE_ACCESSORS:
        return "high"
    if accessor in LOW_RISK_BRIDGE_ACCESSORS:
        return "low"
    if accessor in MEDIUM_RISK_BRIDGE_ACCESSORS:
        return "medium"
    domain = bridge_domain(accessor)
    if domain in {"display", "system", "world"}:
        return "low"
    if domain in {"sprites", "player"}:
        return "high"
    return "medium"


def bridge_strategy(accessor: str) -> str:
    risk = bridge_risk(accessor)
    domain = bridge_domain(accessor)
    if risk == "low":
        return (
            "promote the existing native owner to direct reads/writes, keep one bridge projection "
            "boundary, then codemod same-name call sites"
        )
    if domain == "sprites":
        return (
            "split slot state by lifecycle/draw/combat first; do not bulk-promote the full slot table "
            "as a typed RAM mirror"
        )
    if domain == "player":
        return (
            "continue behavior-shaped player methods; avoid one-field mirrors and verify with route "
            "parity after each coherent behavior cluster"
        )
    return (
        "promote a cohesive native sub-struct, add load/write parity asserts, then replace bridge "
        "call sites in a vertical slice"
    )


def bridge_reduction_plans(
    plans: list[AccessorPlan],
    max_risk: str | None,
    limit: int,
) -> list[BridgeReductionPlan]:
    rows = [
        BridgeReductionPlan(
            accessor=plan.accessor,
            return_type=plan.return_type,
            uses=plan.uses,
            domain=bridge_domain(plan.accessor),
            risk=bridge_risk(plan.accessor),
            strategy=bridge_strategy(plan.accessor),
            first_path=plan.first_path,
            first_line=plan.first_line,
            methods=plan.methods,
            files=plan.files,
        )
        for plan in plans
        if plan.kind == "native-bridge-mutator"
    ]
    risk_order = {"low": 0, "medium": 1, "high": 2}
    if max_risk is not None:
        max_rank = risk_order[max_risk]
        rows = [row for row in rows if risk_order.get(row.risk, 9) <= max_rank]
    rows = sorted(rows, key=lambda row: (risk_order.get(row.risk, 9), -row.uses, row.accessor))
    return rows if limit <= 0 else rows[:limit]


def print_bridge_reduction_text(
    plans: list[AccessorPlan],
    method_limit: int,
    file_limit: int,
    max_risk: str | None,
    limit: int,
) -> None:
    rows = bridge_reduction_plans(plans, max_risk, limit)
    total_uses = sum(row.uses for row in rows)
    suffix = f" (risk <= {max_risk})" if max_risk else ""
    print(
        f"bridge reduction plan{suffix}: "
        f"{total_uses} bridge mutator use(s), {len(rows)} accessor slice(s)"
    )
    if not rows:
        return
    by_risk = Counter(row.risk for row in rows)
    by_domain = Counter()
    for row in rows:
        by_domain[row.domain] += row.uses
    print(
        "risk slices: "
        + ", ".join(f"{risk}={count}" for risk, count in sorted(by_risk.items()))
    )
    print(
        "top domains by uses: "
        + ", ".join(f"{domain}={count}" for domain, count in by_domain.most_common(8))
    )
    for row in rows:
        print()
        print(f"{row.accessor} ({row.uses} use(s), domain={row.domain}, risk={row.risk})")
        print(f"  return:   {row.return_type}")
        print(f"  first:    {row.first_path}:{row.first_line}")
        print(f"  strategy: {row.strategy}")
        methods = ", ".join(
            f"{method} x{count}" for method, count in row.methods.most_common(method_limit)
        )
        files = ", ".join(f"{path} x{count}" for path, count in row.files.most_common(file_limit))
        print(f"  methods:  {methods or '(none)'}")
        print(f"  files:    {files or '(none)'}")


def print_bridge_reduction_json(
    plans: list[AccessorPlan],
    method_limit: int,
    file_limit: int,
    max_risk: str | None,
    limit: int,
) -> None:
    rows = bridge_reduction_plans(plans, max_risk, limit)
    payload = {
        "max_risk": max_risk,
        "total_bridge_mutator_uses": sum(row.uses for row in rows),
        "total_accessors": len(rows),
        "risk_slices": dict(Counter(row.risk for row in rows)),
        "domain_uses": {},
        "accessors": [
            {
                "accessor": row.accessor,
                "return_type": row.return_type,
                "uses": row.uses,
                "domain": row.domain,
                "risk": row.risk,
                "strategy": row.strategy,
                "first_path": row.first_path,
                "first_line": row.first_line,
                "top_methods": [
                    {"method": method, "uses": count}
                    for method, count in row.methods.most_common(method_limit)
                ],
                "top_files": [
                    {"path": path, "uses": count}
                    for path, count in row.files.most_common(file_limit)
                ],
            }
            for row in rows
        ],
    }
    domain_uses = Counter()
    for row in rows:
        domain_uses[row.domain] += row.uses
    payload["domain_uses"] = dict(domain_uses.most_common())
    print(json.dumps(payload, indent=2))


def print_projection_text(projection: ProjectionPlan, field_limit: int) -> None:
    print(
        f"  projection: {projection.view_type} at {projection.path}:{projection.line}; "
        f"{len(projection.fields)} constant field(s), {projection.cluster_count} cluster(s), "
        f"{projection.covered_bytes} covered byte(s), {projection.gap_bytes} gap byte(s) "
        f"across {projection.span_bytes} byte span"
    )
    shown = projection.fields if field_limit <= 0 else projection.fields[:field_limit]
    field_text = ", ".join(
        f"{field.name}@0x{field.address:04x}"
        f"{'[slot]' if field.slot_indexed else ''}"
        f"{f'[{field.width}]' if field.width != 1 else ''}"
        for field in shown
    )
    print(f"  fields: {field_text or '(none)'}")
    if len(shown) < len(projection.fields):
        print(f"  fields: ... {len(projection.fields) - len(shown)} more; pass --field-limit 0 to show all")


def projection_json(projection: ProjectionPlan, field_limit: int) -> dict[str, object]:
    fields = projection.fields if field_limit <= 0 else projection.fields[:field_limit]
    return {
        "view_type": projection.view_type,
        "path": projection.path,
        "line": projection.line,
        "slot_count": projection.slot_count,
        "field_count": len(projection.fields),
        "cluster_count": projection.cluster_count,
        "covered_bytes": projection.covered_bytes,
        "span_bytes": projection.span_bytes,
        "gap_bytes": projection.gap_bytes,
        "fields": [
            {
                "name": field.name,
                "address": field.address,
                "address_hex": f"0x{field.address:04x}",
                "uses": field.uses,
                "slot_indexed": field.slot_indexed,
                "width": field.width,
            }
            for field in fields
        ],
    }


def native_names_for_view(view_type: str) -> tuple[str, str, str]:
    base = re.sub(r"^(Ram|Native)", "", view_type)
    base = re.sub(r"(ViewMut|BridgeMut|View)$", "", base)
    if base == "PlayerState":
        return "FollowerLinkState", "FollowerLinkState", "NativeFollowerLinkBridgeMut"
    if base == "FrameState":
        return "FrameState", "NativeFrameStateView", "NativeFrameStateBridgeMut"
    if base.endswith("Slot"):
        state = f"{base}sState"
        view = f"Native{base}View"
        bridge = f"Native{base}BridgeMut"
    else:
        state = f"{base}NativeState"
        view = f"Native{base}View"
        bridge = f"Native{base}BridgeMut"
    return state, view, bridge


def native_api_type_for(view_type: str) -> str:
    _, view_name, bridge_name = native_names_for_view(view_type)
    if view_type.endswith("ViewMut") or view_type.endswith("BridgeMut"):
        return bridge_name
    return view_name


def api_method_sets(plan: AccessorPlan) -> tuple[str, dict[str, MethodInfo], str, dict[str, MethodInfo]] | None:
    view_type = view_type_name(plan.return_type)
    if view_type is None:
        return None
    source_methods = impl_methods(view_type)
    native_type = native_api_type_for(view_type)
    native_methods = impl_methods(native_type)
    if source_methods is None or native_methods is None:
        return None
    _, source = source_methods
    _, native = native_methods
    return view_type, source, native_type, native


def print_api_diff_for_methods(
    label: str,
    source_type: str,
    source_methods: dict[str, MethodInfo],
    native_type: str,
    native_methods: dict[str, MethodInfo],
    method_uses: Counter[str],
    method_limit: int,
) -> None:
    missing = sorted(set(source_methods) - set(native_methods))
    extra = sorted(set(native_methods) - set(source_methods))
    used_missing = (
        sorted(missing, key=lambda method: (-method_uses.get(method, 0), method))
        if method_uses
        else []
    )
    print()
    print(f"{label}: {source_type} -> {native_type}")
    print(
        f"  source methods: {len(source_methods)}; native methods: {len(native_methods)}; "
        f"missing: {len(missing)}; native-only: {len(extra)}"
    )
    if used_missing:
        shown = used_missing if method_limit <= 0 else used_missing[:method_limit]
        print(
            "  missing used methods: "
            + ", ".join(f"{method} x{method_uses.get(method, 0)}" for method in shown)
        )
        if len(shown) < len(used_missing):
            print(
                f"  missing used methods: ... {len(used_missing) - len(shown)} more; "
                "pass --method-limit 0 to show all"
            )
    unused_missing = [method for method in missing if method_uses.get(method, 0) == 0]
    if unused_missing:
        shown_unused = unused_missing if method_limit <= 0 else unused_missing[:method_limit]
        print("  missing unused methods: " + ", ".join(shown_unused))
        if len(shown_unused) < len(unused_missing):
            print(
                f"  missing unused methods: ... {len(unused_missing) - len(shown_unused)} more; "
                "pass --method-limit 0 to show all"
            )


def print_api_diff(plans: list[AccessorPlan], method_limit: int) -> int:
    for plan in plans:
        method_sets = api_method_sets(plan)
        if method_sets is None:
            print(f"{plan.accessor}: unable to compare API for {plan.return_type}")
            continue
        source_type, source_methods, native_type, native_methods = method_sets
        print_api_diff_for_methods(
            plan.accessor,
            source_type,
            source_methods,
            native_type,
            native_methods,
            plan.methods,
            method_limit,
        )
    return 0


def print_explicit_api_diff(source_type: str, native_type: str, method_limit: int) -> int:
    source = impl_methods(source_type)
    native = impl_methods(native_type)
    if source is None:
        print(f"unable to find impl for {source_type}", file=sys.stderr)
        return 1
    if native is None:
        print(f"unable to find impl for {native_type}", file=sys.stderr)
        return 1
    _, source_methods = source
    _, native_methods = native
    print_api_diff_for_methods(
        "explicit-api",
        source_type,
        source_methods,
        native_type,
        native_methods,
        Counter(),
        method_limit,
    )
    return 0


def print_method_codemod_map(
    plans: list[AccessorPlan],
    native_type: str,
    target_accessor: str,
    method_limit: int,
    output_path: Path | None,
) -> int:
    native = impl_methods(native_type)
    if native is None:
        print(f"unable to find impl for {native_type}", file=sys.stderr)
        return 1
    _, native_methods = native

    lines: list[str] = []
    for plan in plans:
        source_type = view_type_name(plan.return_type)
        if source_type is None:
            lines.append(f"# {plan.accessor}: unable to infer source type from {plan.return_type}")
            continue
        source = impl_methods(source_type)
        if source is None:
            lines.append(f"# {plan.accessor}: unable to find impl for {source_type}")
            continue
        _, source_methods = source
        candidates = [
            method
            for method, _ in plan.methods.most_common()
            if method in source_methods and method in native_methods
        ]
        if method_limit > 0:
            candidates = candidates[:method_limit]
        lines.append(f"# {plan.accessor}: {source_type} -> {target_accessor}: {native_type}")
        lines.append("# Verify target reads are native-authoritative or freshly synced before applying.")
        if not candidates:
            lines.append("# no same-name method mappings available")
            continue
        for method in candidates:
            uses = plan.methods.get(method, 0)
            lines.append(f"{plan.accessor}.{method}={target_accessor}.{method}  # {uses} use(s)")

    text = "\n".join(lines) + "\n"
    if output_path is not None:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(text)
        print(f"wrote method codemod map: {relative(output_path)}")
        print(
            "dry-run: python3 scripts/migrate_semantic_view_methods.py "
            f"--summary --map-file {relative(output_path)} crates/zelda3/src"
        )
        print(
            "apply:   python3 scripts/migrate_semantic_view_methods.py "
            f"--apply --map-file {relative(output_path)} crates/zelda3/src"
        )
    else:
        print(text, end="")
    return 0


def rust_const_name(name: str) -> str:
    snake = re.sub(r"(?<!^)([A-Z])", r"_\1", name).upper()
    return re.sub(r"__+", "_", snake)


def body_const_names(body: str) -> list[str]:
    return sorted(set(re.findall(r"\b[A-Z][A-Z0-9_]{2,}\b", body)))


def slot_byte_const(body: str) -> str | None:
    matches = re.findall(r"self\.ram\[\s*([A-Z][A-Z0-9_]+)\s*\+\s*self\.slot\s*\]", body)
    if len(set(matches)) == 1:
        return matches[0]
    return None


def indent_body(lines: list[str]) -> str:
    return "\n".join(f"        {line}" if line else "" for line in lines)


def native_fields_by_const(native_type: str) -> dict[str, NativeFieldInfo]:
    source = find_impl_source(native_type)
    if source is None:
        return {}
    _, _, impl_text = source
    fields: dict[str, NativeFieldInfo] = {}
    for field, const in re.findall(
        r"\b([a-z][A-Za-z0-9_]*)\s*:\s*ram_byte\(\s*ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)",
        impl_text,
    ):
        fields[const] = NativeFieldInfo(field=field, width=1)
    for field, const in re.findall(
        r"\b([a-z][A-Za-z0-9_]*)\s*:\s*read_le_u16\(\s*ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)",
        impl_text,
    ):
        fields[const] = NativeFieldInfo(field=field, width=2)
    return fields


def field_name_for_const(const: str) -> str:
    name = const.lower()
    prefixes = [
        "link_",
        "player_",
        "flag_",
        "eq_",
        "button_",
        "state_for_",
        "countdown_for_",
    ]
    for prefix in prefixes:
        if name.startswith(prefix):
            name = name[len(prefix) :]
            break
    suffixes = ["_lo", "_hi"]
    for suffix in suffixes:
        if name.endswith(suffix):
            name = name[: -len(suffix)]
            break
    return re.sub(r"[^a-z0-9_]+", "_", name).strip("_")


def method_direct_const_reads(method: MethodInfo) -> dict[str, int]:
    body = method.body
    reads: dict[str, int] = {}

    def add(const: str, width: int) -> None:
        reads[const] = max(reads.get(const, 0), width)

    for const in re.findall(r"\b(?:word|read_le_u16)\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)", body):
        add(const, 2)
    for const in re.findall(r"\bbyte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*(?:\+\s*1\s*)?\)", body):
        add(const, 1)
    return reads


def native_field_drafts_for_missing_methods(
    source_methods: dict[str, MethodInfo],
    native_methods: dict[str, MethodInfo],
    native_fields: dict[str, NativeFieldInfo],
    method_uses: Counter[str],
    method_limit: int,
) -> list[NativeFieldDraft]:
    missing = [
        method
        for method in sorted(set(source_methods) - set(native_methods), key=lambda name: (-method_uses.get(name, 0), name))
        if method_uses.get(method, 0) > 0
    ]
    if method_limit > 0:
        missing = missing[:method_limit]

    methods_by_const: dict[str, set[str]] = defaultdict(set)
    widths_by_const: dict[str, int] = {}
    for method_name in missing:
        for const, width in method_direct_const_reads(source_methods[method_name]).items():
            if const in native_fields:
                continue
            methods_by_const[const].add(method_name)
            widths_by_const[const] = max(widths_by_const.get(const, 0), width)

    drafts: list[NativeFieldDraft] = []
    used_fields: set[str] = set()
    for const in sorted(methods_by_const, key=lambda name: (-len(methods_by_const[name]), name)):
        field = field_name_for_const(const)
        if not field:
            field = const.lower()
        original = field
        index = 2
        while field in used_fields:
            field = f"{original}_{index}"
            index += 1
        used_fields.add(field)
        drafts.append(
            NativeFieldDraft(
                field=field,
                const=const,
                width=widths_by_const[const],
                methods=tuple(sorted(methods_by_const[const])),
            )
        )
    return drafts


def print_native_field_promotion_draft(plans: list[AccessorPlan], native_type: str, method_limit: int) -> int:
    if len(plans) != 1:
        print(f"--emit-native-field-promotion-draft requires exactly one selected accessor, got {len(plans)}", file=sys.stderr)
        return 1
    plan = plans[0]
    source_type = view_type_name(plan.return_type)
    if source_type is None:
        print(f"unable to infer source type from {plan.return_type}", file=sys.stderr)
        return 1
    source = impl_methods(source_type)
    native = impl_methods(native_type)
    if source is None:
        print(f"unable to find impl for {source_type}", file=sys.stderr)
        return 1
    if native is None:
        print(f"unable to find impl for {native_type}", file=sys.stderr)
        return 1
    _, source_methods = source
    _, native_methods = native
    native_fields = native_fields_by_const(native_type)
    drafts = native_field_drafts_for_missing_methods(
        source_methods,
        native_methods,
        native_fields,
        plan.methods,
        method_limit,
    )

    print(f"// Native field promotion draft for {source_type} -> {native_type}.")
    print("// Review semantic names and grouped byte fields before applying.")
    print("// This is intentionally a draft: non-contiguous LO/HI pairs may need one u16 field.")
    print()
    print("// struct fields")
    for draft in drafts:
        print(
            f"    {draft.field}: {'u16' if draft.width == 2 else 'u8'},"
            f" // {draft.const}; used by {', '.join(draft.methods)}"
        )
    print()
    print("// load_from_ram fields")
    for draft in drafts:
        if draft.width == 2:
            print(f"            {draft.field}: read_le_u16(ram, {draft.const}),")
        else:
            print(f"            {draft.field}: ram_byte(ram, {draft.const}),")
    print()
    print("// write_to_ram fields")
    for draft in drafts:
        if draft.width == 2:
            print(f"        write_le_u16(ram, {draft.const}, self.{draft.field});")
        else:
            print(f"        ram[{draft.const}] = self.{draft.field};")

    hypothetical_fields = dict(native_fields)
    for draft in drafts:
        hypothetical_fields[draft.const] = NativeFieldInfo(draft.field, draft.width)
    missing = [
        method
        for method in sorted(set(source_methods) - set(native_methods), key=lambda name: (-plan.methods.get(name, 0), name))
        if plan.methods.get(method, 0) > 0
    ]
    if method_limit > 0:
        missing = missing[:method_limit]
    print()
    print("// getter candidates that become available with the drafted fields")
    emitted = 0
    unsupported = 0
    for method_name in missing:
        header, body = translated_native_read_method(source_methods[method_name], hypothetical_fields)
        uses = plan.methods.get(method_name, 0)
        print()
        print(f"// {method_name}: {uses} current accessor use(s); source line {source_methods[method_name].line}")
        if header is None or body is None:
            unsupported += 1
            print(f"// TODO {method_name}: {body}")
            continue
        emitted += 1
        print(header)
        print(body)
    print()
    print(f"// drafted {len(drafts)} field(s); emitted {emitted} getter(s); {unsupported} getter(s) still need manual handling")
    return 0


def bridge_state_type(native_type: str) -> str | None:
    source = find_impl_source(native_type)
    if source is None:
        return None
    path, _, _ = source
    text = path.read_text()
    struct_match = re.search(
        rf"\bstruct\s+{re.escape(native_type)}(?:<[^>]+>)?\s*\{{(?P<body>[\s\S]*?)\n\}}",
        text,
    )
    if struct_match is None:
        return None
    state_match = re.search(
        r"\bstate\s*:\s*&(?:'a\s+)?mut\s+([A-Z][A-Za-z0-9_]*)",
        struct_match.group("body"),
    )
    return state_match.group(1) if state_match else None


def method_names_for_type(type_name: str | None) -> set[str]:
    if type_name is None:
        return set()
    methods = impl_methods(type_name)
    if methods is None:
        return set()
    _, method_map = methods
    return set(method_map)


def translated_native_read_method(
    method: MethodInfo,
    native_fields: dict[str, NativeFieldInfo],
) -> tuple[str | None, str | None]:
    body = method.body.strip()

    def field_for_exact(const: str, width: int) -> str | None:
        field = native_fields.get(const)
        if field is None:
            return None
        if field.width != width:
            return None
        return f"self.{field.field}"

    def field_for_byte(const: str) -> str | None:
        field = native_fields.get(const)
        if field is None:
            return None
        if field.width == 1:
            return f"self.{field.field}"
        if field.width == 2:
            return f"self.{field.field} as u8"
        return None

    def field_for_high_byte(const: str) -> str | None:
        field = native_fields.get(const)
        if field is None or field.width != 2:
            return None
        return f"(self.{field.field} >> 8) as u8"

    byte_read = re.fullmatch(r"byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)", body)
    if byte_read:
        field = field_for_byte(byte_read.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_read.group(1)} as a byte field"
        return method.signature + " {", indent_body([field]) + "\n    }"

    high_byte_read = re.fullmatch(
        r"byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\+\s*1\s*\)",
        body,
    )
    if high_byte_read:
        field = field_for_high_byte(high_byte_read.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {high_byte_read.group(1)} as a word field"
        return method.signature + " {", indent_body([field]) + "\n    }"

    word_read = re.fullmatch(
        r"(?:word|read_le_u16)\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)",
        body,
    )
    if word_read:
        field = field_for_exact(word_read.group(1), 2)
        if field is None:
            return None, f"unsupported: native target does not load {word_read.group(1)} as a word field"
        return method.signature + " {", indent_body([field]) + "\n    }"

    byte_bool = re.fullmatch(
        r"byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\s*!=\s*0",
        body,
    )
    if byte_bool:
        field = field_for_byte(byte_bool.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_bool.group(1)} as a byte field"
        return method.signature + " {", indent_body([f"{field} != 0"]) + "\n    }"

    byte_eq_value = re.fullmatch(
        r"byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\s*==\s*value",
        body,
    )
    if byte_eq_value:
        field = field_for_byte(byte_eq_value.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_eq_value.group(1)} as a byte field"
        return method.signature + " {", indent_body([f"{field} == value"]) + "\n    }"

    byte_matches = re.fullmatch(
        r"matches!\(\s*byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\),\s*([^)]+)\)",
        body,
    )
    if byte_matches:
        field = field_for_byte(byte_matches.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_matches.group(1)} as a byte field"
        return method.signature + " {", indent_body([f"matches!({field}, {byte_matches.group(2).strip()})"]) + "\n    }"

    byte_usize = re.fullmatch(
        r"usize::from\(\s*byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\s*\)",
        body,
    )
    if byte_usize:
        field = field_for_byte(byte_usize.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_usize.group(1)} as a byte field"
        return method.signature + " {", indent_body([f"usize::from({field})"]) + "\n    }"

    byte_shift_usize = re.fullmatch(
        r"usize::from\(\s*byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\s*>>\s*([0-9]+)\s*\)",
        body,
    )
    if byte_shift_usize:
        field = field_for_byte(byte_shift_usize.group(1))
        if field is None:
            return None, f"unsupported: native target does not load {byte_shift_usize.group(1)} as a byte field"
        return method.signature + " {", indent_body([f"usize::from({field} >> {byte_shift_usize.group(2)})"]) + "\n    }"

    byte_wrapping_sub = re.fullmatch(
        r"byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\.wrapping_sub\("
        r"\s*byte\(\s*self\.ram\s*,\s*([A-Z][A-Z0-9_]+)\s*\)\s*\)",
        body,
    )
    if byte_wrapping_sub:
        left = field_for_byte(byte_wrapping_sub.group(1))
        right = field_for_byte(byte_wrapping_sub.group(2))
        if left is None:
            return None, f"unsupported: native target does not load {byte_wrapping_sub.group(1)} as a byte field"
        if right is None:
            return None, f"unsupported: native target does not load {byte_wrapping_sub.group(2)} as a byte field"
        return method.signature + " {", indent_body([f"{left}.wrapping_sub({right})"]) + "\n    }"

    same_view_signed = re.fullmatch(r"self\.([a-z][A-Za-z0-9_]*)\(\)\s+as\s+i8", body)
    if same_view_signed:
        return method.signature + " {", indent_body([f"self.{same_view_signed.group(1)}() as i8"]) + "\n    }"

    return None, "unsupported: body shape needs manual native primitive"


def translated_bridge_method(
    method: MethodInfo,
    native_fields: dict[str, NativeFieldInfo] | None = None,
    bridge_state_methods: set[str] | None = None,
) -> tuple[str | None, str | None]:
    body = method.body.strip()
    const = slot_byte_const(body)

    if native_fields is not None:
        header, translated = translated_native_read_method(method, native_fields)
        if header is not None or translated != "unsupported: body shape needs manual native primitive":
            return header, translated

    direct_word_set = re.fullmatch(
        r"write_le_u16\(\s*self\.ram,\s*([A-Z][A-Z0-9_]+)\s*,\s*value\s*\)\s*;",
        body,
    )
    if direct_word_set:
        if bridge_state_methods is not None and method.name not in bridge_state_methods:
            return None, f"unsupported: native state is missing `{method.name}`"
        base = direct_word_set.group(1)
        lines = [
            f"self.state.{method.name}(value);",
            f"write_le_u16(self.ram, {base}, value);",
            "self.debug_assert_matches_ram();",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    direct_byte_set = re.fullmatch(
        r"self\.ram\[\s*([A-Z][A-Z0-9_]+)\s*\]\s*=\s*value\s*;",
        body,
    )
    if direct_byte_set:
        if bridge_state_methods is not None and method.name not in bridge_state_methods:
            return None, f"unsupported: native state is missing `{method.name}`"
        base = direct_byte_set.group(1)
        lines = [
            f"self.state.{method.name}(value);",
            f"self.ram[{base}] = value;",
            "self.debug_assert_matches_ram();",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.search(r"for base in \[[\s\S]*\]\s*\{", body) and "self.ram[base + self.slot] = 0" in body:
        constants = body_const_names(body)
        lines = ["for base in ["]
        lines.extend(f"            {name}," for name in constants)
        lines.extend(["        ] {", "            self.set_byte(base, 0);", "        }"])
        return method.signature + " {", indent_body(lines) + "\n    }"

    if const is None:
        position_match = re.fullmatch(
            r"write_position\(\s*self\.ram,\s*"
            r"([A-Z][A-Z0-9_]+)\s*\+\s*self\.slot,\s*"
            r"([A-Z][A-Z0-9_]+)\s*\+\s*self\.slot,\s*"
            r"value,\s*\)\s*;",
            body,
        )
        if position_match:
            low_offset, high_offset = position_match.groups()
            return (
                method.signature + " {",
                indent_body([f"self.set_position({low_offset}, {high_offset}, value);"]) + "\n    }",
            )
        word_match = re.fullmatch(
            r"write_le_u16\(\s*self\.ram,\s*([A-Z][A-Z0-9_]+)\s*\+\s*self\.slot\s*\*\s*2,\s*value\s*\)\s*;",
            body,
        )
        if word_match:
            base = word_match.group(1)
            return (
                method.signature + " {",
                indent_body([f"self.set_word_at({base} + self.slot * 2, value);"]) + "\n    }",
            )
        return None, "unsupported: method does not target exactly one slot byte"

    if re.fullmatch(rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]", body):
        return method.signature + " {", indent_body([f"self.state.byte(self.slot, {const})"]) + "\n    }"

    if re.fullmatch(rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*value\s*;", body):
        return method.signature + " {", indent_body([f"self.set_byte({const}, value);"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_add\(1\)\s*;",
        body,
    ):
        return method.signature + " {", indent_body([f"self.add_byte({const}, 1);"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_sub\(1\)\s*;",
        body,
    ):
        return method.signature + " {", indent_body([f"self.subtract_byte({const}, 1);"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_add\(value\)\s*;",
        body,
    ):
        return method.signature + " {", indent_body([f"self.add_byte({const}, value);"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_sub\(value\)\s*;",
        body,
    ):
        return method.signature + " {", indent_body([f"self.subtract_byte({const}, value);"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_add\(1\);\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]",
        body,
    ):
        return method.signature + " {", indent_body([f"self.add_byte({const}, 1)"]) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_sub\(1\);\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]",
        body,
    ):
        return method.signature + " {", indent_body([f"self.subtract_byte({const}, 1)"]) + "\n    }"

    for op, helper in [("^=", "xor_byte"), ("&=", "and_byte"), ("|=", "or_byte")]:
        if re.fullmatch(rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*{re.escape(op)}\s*value\s*;", body):
            return method.signature + " {", indent_body([f"self.{helper}({const}, value);"]) + "\n    }"

    if re.fullmatch(rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*&=\s*!mask\s*;", body):
        lines = [
            f"let next = self.state.byte(self.slot, {const}) & !mask;",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"\(self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*&\s*mask\)\s*\|\s*value\s*;",
        body,
    ):
        lines = [
            f"let next = (self.state.byte(self.slot, {const}) & mask) | value;",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_neg\(\)\s*;",
        body,
    ):
        lines = [
            f"let next = self.state.byte(self.slot, {const}).wrapping_neg();",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\.wrapping_shl\(amount\)\s*;",
        body,
    ):
        lines = [
            f"let next = self.state.byte(self.slot, {const}).wrapping_shl(amount);",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.fullmatch(
        rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*=\s*"
        rf"\(\(self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s+as\s+i8\)\s*>>\s*1\)\s*as\s+u8\s*;",
        body,
    ):
        lines = [
            f"let next = ((self.state.byte(self.slot, {const}) as i8) >> 1) as u8;",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    if re.fullmatch(rf"self\.ram\[\s*{const}\s*\+\s*self\.slot\s*\]\s*>>=\s*1\s*;", body):
        lines = [
            f"let next = self.state.byte(self.slot, {const}) >> 1;",
            f"self.set_byte({const}, next);",
        ]
        return method.signature + " {", indent_body(lines) + "\n    }"

    return None, "unsupported: body shape needs manual native primitive"


def emit_bridge_method_stubs(plans: list[AccessorPlan], method_limit: int) -> int:
    if len(plans) != 1:
        print(f"--emit-bridge-method-stubs requires exactly one selected accessor, got {len(plans)}", file=sys.stderr)
        return 1
    plan = plans[0]
    method_sets = api_method_sets(plan)
    if method_sets is None:
        print(f"unable to compare API for {plan.return_type}", file=sys.stderr)
        return 1
    source_type, source_methods, native_type, native_methods = method_sets
    state_type = bridge_state_type(native_type)
    native_fields = native_fields_by_const(state_type or native_type)
    bridge_state_methods = method_names_for_type(state_type) if state_type else None
    missing = sorted(set(source_methods) - set(native_methods), key=lambda method: (-plan.methods.get(method, 0), method))
    if method_limit > 0:
        missing = missing[:method_limit]

    print(f"// Generated bridge-method candidates from {source_type} for {native_type}.")
    if state_type:
        print(f"// Bridge state primitive target: {state_type}.")
    print("// Review before pasting; unsupported methods are listed as comments.")
    emitted = 0
    unsupported = 0
    for name in missing:
        method = source_methods[name]
        header, body = translated_bridge_method(method, native_fields, bridge_state_methods)
        uses = plan.methods.get(name, 0)
        print()
        print(f"// {name}: {uses} current accessor use(s); source line {method.line}")
        if header is None or body is None:
            unsupported += 1
            print(f"// TODO {name}: {body}")
            continue
        emitted += 1
        print(header)
        print(body)
    print()
    print(f"// emitted {emitted} method(s); {unsupported} unsupported method(s)")
    return 0


def emit_work_area_skeleton(plan: AccessorPlan, field_limit: int) -> int:
    projection = projection_plan(plan)
    if projection is None:
        print(f"no projection source found for {plan.accessor}", file=sys.stderr)
        return 1
    state_name, view_name, bridge_name = native_names_for_view(projection.view_type)
    const_prefix = rust_const_name(state_name.removesuffix("State"))
    fields = projection.fields if field_limit <= 0 else projection.fields[:field_limit]
    slot_shaped = projection.slot_count > 1 and any(field.slot_indexed for field in fields)
    if len(fields) != len(projection.fields):
        print(
            f"--emit-work-area-skeleton requires all fields; rerun with --field-limit 0 "
            f"({len(projection.fields) - len(fields)} field(s) hidden)",
            file=sys.stderr,
        )
        return 1

    print(f"// Generated from {projection.view_type} at {projection.path}:{projection.line}.")
    print("// Review names and method placement before committing generated Rust.")
    print(f"const {const_prefix}_FIELD_RANGES: &[(usize, usize)] = &[")
    for field in fields:
        print(f"    ({field.name}, {field.width}),")
    print("];")
    print(f"const {const_prefix}_WORK_BASE: usize = {fields[0].name};")
    print(
        f"const {const_prefix}_WORK_END: usize = "
        f"{fields[-1].name} + {fields[-1].width};"
    )
    print(
        f"const {const_prefix}_WORK_LEN: usize = "
        f"{const_prefix}_WORK_END - {const_prefix}_WORK_BASE;"
    )
    print()
    print("#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]")
    print(f"pub(crate) struct {state_name} {{")
    print("    work: Vec<u8>,")
    print("}")
    print()
    print(f"impl Default for {state_name} {{")
    print("    fn default() -> Self {")
    print(f"        Self {{ work: vec![0; {const_prefix}_WORK_LEN] }}")
    print("    }")
    print("}")
    print()
    print(f"impl {state_name} {{")
    print("    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {")
    print("        let mut state = Self::default();")
    print(f"        for offset in Self::field_offsets() {{")
    print("            let index = Self::work_index(offset);")
    print("            state.work[index] = ram.get(offset).copied().unwrap_or(0);")
    print("        }")
    print("        state")
    print("    }")
    print()
    print("    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {")
    print("        for offset in Self::field_offsets() {")
    print("            ram[offset] = self.byte_at(offset);")
    print("        }")
    print("    }")
    print()
    if slot_shaped:
        print(f"    pub(crate) fn slot(&self, slot: usize) -> {view_name}<'_> {{")
        print(f"        {view_name} {{ state: self, slot }}")
        print("    }")
        print()
        print(
            f"    pub(crate) fn slot_mut<'a>(&'a mut self, ram: &'a mut [u8], "
            f"slot: usize) -> {bridge_name}<'a> {{"
        )
        print(f"        {bridge_name} {{ state: self, ram, slot }}")
        print("    }")
        print()
    else:
        print(f"    pub(crate) fn view(&self) -> {view_name}<'_> {{")
        print(f"        {view_name} {{ state: self }}")
        print("    }")
        print()
        print(
            f"    pub(crate) fn bridge_mut<'a>(&'a mut self, ram: &'a mut [u8]) "
            f"-> {bridge_name}<'a> {{"
        )
        print(f"        {bridge_name} {{ state: self, ram }}")
        print("    }")
        print()
    print("    fn field_offsets() -> impl Iterator<Item = usize> {")
    print(f"        {const_prefix}_FIELD_RANGES")
    print("            .iter()")
    print("            .copied()")
    print("            .flat_map(|(base, width)| (0..width).map(move |offset| base + offset))")
    print("    }")
    print()
    print(f"    fn work_index(offset: usize) -> usize {{ offset - {const_prefix}_WORK_BASE }}")
    print("    fn byte_at(&self, offset: usize) -> u8 { self.work.get(Self::work_index(offset)).copied().unwrap_or(0) }")
    print("    fn set_byte_at(&mut self, offset: usize, value: u8) { self.work[Self::work_index(offset)] = value; }")
    if slot_shaped:
        print("    fn byte(&self, slot: usize, base: usize) -> u8 { self.byte_at(base + slot) }")
        print("    fn set_byte(&mut self, slot: usize, base: usize, value: u8) { self.set_byte_at(base + slot, value); }")
    else:
        print("    fn byte(&self, offset: usize) -> u8 { self.byte_at(offset) }")
        print("    fn set_byte(&mut self, offset: usize, value: u8) { self.set_byte_at(offset, value); }")
    print("}")
    print()
    print(f"pub(crate) struct {view_name}<'a> {{")
    print(f"    state: &'a {state_name},")
    if slot_shaped:
        print("    slot: usize,")
    print("}")
    print()
    print(f"pub(crate) struct {bridge_name}<'a> {{")
    print(f"    state: &'a mut {state_name},")
    print("    ram: &'a mut [u8],")
    if slot_shaped:
        print("    slot: usize,")
    print("}")
    print()
    print(f"impl<'a> {bridge_name}<'a> {{")
    print("    fn sync(&mut self) {")
    if slot_shaped:
        print(f"        for (base, width) in {const_prefix}_FIELD_RANGES.iter().copied() {{")
        print("            let offset = base + self.slot;")
        print("            if self.slot < width {")
        print("                self.ram[offset] = self.state.byte_at(offset);")
        print("            }")
        print("        }")
    else:
        print("        self.state.write_to_ram(self.ram);")
    print("        self.debug_assert_matches_ram();")
    print("    }")
    print()
    print("    fn debug_assert_matches_ram(&self) {")
    if slot_shaped:
        print(f"        for (base, width) in {const_prefix}_FIELD_RANGES.iter().copied() {{")
        print("            let offset = base + self.slot;")
        print("            if self.slot < width {")
        print("                debug_assert_eq!(self.state.byte_at(offset), self.ram[offset]);")
        print("            }")
        print("        }")
    else:
        print(f"        for (base, width) in {const_prefix}_FIELD_RANGES.iter().copied() {{")
        print("            for offset in base..base + width {")
        print("                debug_assert_eq!(self.state.byte_at(offset), self.ram[offset]);")
        print("            }")
        print("        }")
    print("    }")
    print("}")
    return 0


def print_text(
    plans: list[AccessorPlan],
    method_limit: int,
    file_limit: int,
    include_projection: bool,
    field_limit: int,
) -> None:
    total_uses = sum(plan.uses for plan in plans)
    print(f"semantic migration plan: {total_uses} use(s), {len(plans)} accessor slice(s)")
    for plan in plans:
        print()
        print(f"{plan.accessor} ({plan.kind}, {plan.uses} use(s))")
        print(f"  return: {plan.return_type}")
        print(f"  status: {migration_status(plan)}")
        print(f"  first:  {plan.first_path}:{plan.first_line}")
        methods = ", ".join(
            f"{method} x{count}" for method, count in plan.methods.most_common(method_limit)
        )
        files = ", ".join(f"{path} x{count}" for path, count in plan.files.most_common(file_limit))
        print(f"  methods: {methods or '(none)'}")
        print(f"  files:   {files or '(none)'}")
        command = codemod_command(plan)
        if command:
            print(f"  codemod: {command}")
        if include_projection:
            projection = projection_plan(plan)
            if projection:
                print_projection_text(projection, field_limit)


def print_json(
    plans: list[AccessorPlan],
    method_limit: int,
    file_limit: int,
    include_projection: bool,
    field_limit: int,
) -> None:
    payload = {
        "total_uses": sum(plan.uses for plan in plans),
        "total_accessors": len(plans),
        "accessors": [
            {
                "accessor": plan.accessor,
                "kind": plan.kind,
                "return_type": plan.return_type,
                "uses": plan.uses,
                "status": migration_status(plan),
                "first_path": plan.first_path,
                "first_line": plan.first_line,
                "top_methods": [
                    {"method": method, "uses": count}
                    for method, count in plan.methods.most_common(method_limit)
                ],
                "top_files": [
                    {"path": path, "uses": count}
                    for path, count in plan.files.most_common(file_limit)
                ],
                "codemod": codemod_command(plan),
                **(
                    {"projection": projection_json(projection, field_limit)}
                    if include_projection and (projection := projection_plan(plan))
                    else {}
                ),
            }
            for plan in plans
        ],
    }
    print(json.dumps(payload, indent=2))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--kind",
        action="append",
        choices=[
            "ram-backed-view",
            "native-read-helper",
            "native-copy-helper",
            "native-bridge-mutator",
        ],
        help="limit migration plans to one accessor kind; repeat for multiple kinds",
    )
    parser.add_argument("--accessor", help="regular expression for accessor/helper names to include")
    parser.add_argument("--exclude-accessor", help="regular expression for accessor/helper names to exclude")
    parser.add_argument("--path", help="regular expression for relative Rust file paths to include")
    parser.add_argument("--exclude-path", help="regular expression for relative Rust file paths to exclude")
    parser.add_argument("--limit", type=int, default=20, help="maximum accessor slices to print; use 0 for all")
    parser.add_argument("--method-limit", type=int, default=12, help="maximum methods to show per accessor")
    parser.add_argument("--file-limit", type=int, default=6, help="maximum files to show per accessor")
    parser.add_argument(
        "--projection-fields",
        action="store_true",
        help="include constants touched by the accessor's view implementation and sparse projection stats",
    )
    parser.add_argument(
        "--emit-work-area-skeleton",
        action="store_true",
        help="emit a Rust native work-area skeleton for exactly one selected accessor",
    )
    parser.add_argument(
        "--api-diff",
        action="store_true",
        help="compare the RAM-backed view API with the matching native view or bridge API",
    )
    parser.add_argument(
        "--source-type",
        help="source Rust type to compare with --api-diff, e.g. RamFrameStateViewMut",
    )
    parser.add_argument(
        "--native-type",
        help="native Rust type to compare with --api-diff, e.g. NativeFrameStateBridgeMut",
    )
    parser.add_argument(
        "--emit-bridge-method-stubs",
        action="store_true",
        help="emit native bridge method candidates for simple missing RAM view methods",
    )
    parser.add_argument(
        "--emit-native-field-promotion-draft",
        action="store_true",
        help="emit native field/load/write/getter candidates for used source methods missing on a native type",
    )
    parser.add_argument(
        "--emit-method-codemod-map",
        action="store_true",
        help="emit --map lines for same-name methods that exist on a target native accessor/type",
    )
    parser.add_argument(
        "--bridge-reduction-report",
        action="store_true",
        help="rank remaining native bridge mutator slices by domain, risk, and payoff",
    )
    parser.add_argument(
        "--bridge-max-risk",
        choices=["low", "medium", "high"],
        help="with --bridge-reduction-report, include only bridge slices at or below this risk",
    )
    parser.add_argument(
        "--target-accessor",
        help="target accessor name for --emit-method-codemod-map, e.g. follower_link_state",
    )
    parser.add_argument(
        "--target-type",
        help="target Rust type for --emit-method-codemod-map, e.g. FollowerLinkState",
    )
    parser.add_argument(
        "--write-method-codemod-map",
        type=Path,
        help="write --emit-method-codemod-map output to a file consumable by migrate_semantic_view_methods.py",
    )
    parser.add_argument("--field-limit", type=int, default=20, help="maximum projection fields to show per accessor")
    parser.add_argument("--format", choices=["text", "json"], default="text", help="output format")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if (args.source_type or args.native_type) and not (args.source_type and args.native_type):
        print("--source-type and --native-type must be passed together", file=sys.stderr)
        return 1
    if (args.source_type or args.native_type) and not args.api_diff:
        print("--source-type/--native-type are only supported with --api-diff", file=sys.stderr)
        return 1
    if args.source_type and args.native_type:
        return print_explicit_api_diff(args.source_type, args.native_type, args.method_limit)
    if (args.emit_method_codemod_map or args.write_method_codemod_map) and not (
        args.target_accessor and args.target_type
    ):
        print(
            "--emit-method-codemod-map/--write-method-codemod-map require "
            "--target-accessor and --target-type",
            file=sys.stderr,
        )
        return 1
    if args.emit_native_field_promotion_draft and not args.target_type:
        print("--emit-native-field-promotion-draft requires --target-type", file=sys.stderr)
        return 1

    uses = selected_uses(
        args.kind,
        args.accessor,
        args.exclude_accessor,
        args.path,
        args.exclude_path,
    )
    plans = accessor_plans(uses)

    if args.emit_work_area_skeleton:
        if args.limit > 0:
            plans = plans[: args.limit]
        if len(plans) != 1:
            print(
                f"--emit-work-area-skeleton requires exactly one selected accessor, got {len(plans)}",
                file=sys.stderr,
            )
            return 1
        return emit_work_area_skeleton(plans[0], args.field_limit)

    if args.api_diff:
        if args.limit > 0:
            plans = plans[: args.limit]
        return print_api_diff(plans, args.method_limit)

    if args.bridge_reduction_report:
        if args.format == "json":
            print_bridge_reduction_json(
                plans,
                args.method_limit,
                args.file_limit,
                args.bridge_max_risk,
                args.limit,
            )
        else:
            print_bridge_reduction_text(
                plans,
                args.method_limit,
                args.file_limit,
                args.bridge_max_risk,
                args.limit,
            )
        return 0

    if args.emit_bridge_method_stubs:
        if args.limit > 0:
            plans = plans[: args.limit]
        return emit_bridge_method_stubs(plans, args.method_limit)

    if args.emit_native_field_promotion_draft:
        if args.limit > 0:
            plans = plans[: args.limit]
        return print_native_field_promotion_draft(plans, args.target_type, args.method_limit)

    if args.emit_method_codemod_map or args.write_method_codemod_map:
        if args.limit > 0:
            plans = plans[: args.limit]
        return print_method_codemod_map(
            plans,
            args.target_type,
            args.target_accessor,
            args.method_limit,
            args.write_method_codemod_map,
        )

    if args.limit > 0:
        plans = plans[: args.limit]
    if args.format == "json":
        print_json(plans, args.method_limit, args.file_limit, args.projection_fields, args.field_limit)
    else:
        print_text(plans, args.method_limit, args.file_limit, args.projection_fields, args.field_limit)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
