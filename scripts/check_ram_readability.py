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


def rust_files() -> list[Path]:
    return sorted(SRC_ROOT.rglob("*.rs"))


def is_public_ram_constant_registry(path: Path) -> bool:
    return path == SRC_ROOT / "game_state" / "constants.rs"


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def check_file(path: Path) -> list[Finding]:
    text = path.read_text()
    findings: list[Finding] = []
    checks = [
        (ADDRESS_NAME_RE, "address-derived RAM constant name"),
        (C_ADDRESS_NAME_RE, "C-style address-derived RAM name in Rust source"),
        (DIRECT_RAM_RE, "direct hex RAM access; use a named constant"),
    ]
    for pattern, message in checks:
        for match in pattern.finditer(text):
            findings.append(Finding(path, line_for_offset(text, match.start()), message))
    return findings


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
    args = parser.parse_args()

    findings = [finding for path in rust_files() for finding in check_file(path)]
    if findings:
        for finding in findings:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: {finding.message}", file=sys.stderr)
        return 1

    constants = scan_consts()
    weak_findings = weak_name_findings(constants)
    if args.warn_weak_names and weak_findings:
        for finding in weak_findings:
            rel = finding.path.relative_to(REPO_ROOT)
            print(f"{rel}:{finding.line}: warning: {finding.message}", file=sys.stderr)

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
    print(message)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
