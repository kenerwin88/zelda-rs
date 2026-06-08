#!/usr/bin/env python3
"""Mine NES_Ver2 source labels and crosswalk them to Rust RAM constants."""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE_ROOT = Path.home() / "Documents" / "NES_Ver2"
OUT_DIR = REPO_ROOT / "docs" / "nes-ver2"
SRC_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"

RUST_PRIVATE_CONST_RE = re.compile(
    r"^\s*const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9A-Fa-f]+)\s*;",
    re.MULTILINE,
)
RUST_RAM_MODULE_CONST_RE = re.compile(
    r"^\s*pub(?:\([^)]*\))?\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*usize\s*=\s*(0x[0-9A-Fa-f]+)\s*;",
    re.MULTILINE,
)
EQU_RE = re.compile(
    r"^\s*(?P<label>[A-Za-z_][A-Za-z0-9_.$?]*)\s+EQU\s+(?P<expr>[^;]+?)(?P<tail>\s*(?:;.*)?)$"
)
HEX_RE = re.compile(r"^[0-9A-Fa-f]+H$")
DEC_RE = re.compile(r"^[0-9]+$")
IDENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.$?]*$")
ADD_SUB_RE = re.compile(
    r"^(?P<base>[A-Za-z_][A-Za-z0-9_.$?]*|[0-9A-Fa-f]+H|[0-9]+)\s*(?P<op>[+-])\s*(?P<delta>[0-9A-Fa-f]+H|[0-9]+)$"
)
LINE_LABEL_RE = re.compile(r"^\s*(?P<label>[A-Za-z_][A-Za-z0-9_.$?]*)\s+EQU\s+\$")
COMMENT_RE = re.compile(r";+\s*(?P<comment>.*)$")
MAP_MODULE_RE = re.compile(
    r"^\s*(?P<file>\S+\.rel)\s+(?P<module>\S+)\s+(?P<section>\S+)\s+"
    r"(?P<start>[0-9A-Fa-f]{2}:[0-9A-Fa-f]{4})\s+"
    r"(?P<end>[0-9A-Fa-f]{2}:[0-9A-Fa-f]{4})\s+"
    r"(?P<size>[0-9A-Fa-f]{4})\s*$"
)

TEXT_SUFFIXES = {
    ".asm",
    ".inc",
    ".lst",
    ".txt",
    ".tbl",
    ".cnf",
    ".lnk",
    ".map",
    ".make",
    ".sdm",
    ".c",
    ".h",
}
WEAK_NAME_RE = re.compile(r"(?:^|_)(UNK|SOME|VAR|TMP|SCRATCH)(?:_|$)")
COMMENT_TRANSLATIONS = [
    (re.compile(r"\bflem\b", re.IGNORECASE), "frame"),
    (re.compile(r"\bfram\b", re.IGNORECASE), "frame"),
    (re.compile(r"\bfrcnt\b", re.IGNORECASE), "frame count"),
    (re.compile(r"\bchenge\b", re.IGNORECASE), "change"),
    (re.compile(r"\bchara\b", re.IGNORECASE), "character"),
    (re.compile(r"\bchar\b", re.IGNORECASE), "character"),
    (re.compile(r"\bpos\.\b", re.IGNORECASE), "position"),
    (re.compile(r"\bpos\b", re.IGNORECASE), "position"),
    (re.compile(r"\bhi\b", re.IGNORECASE), "high byte"),
    (re.compile(r"\blow\b", re.IGNORECASE), "low byte"),
    (re.compile(r"\bhozon\b", re.IGNORECASE), "saved"),
    (re.compile(r"\bmuki\b", re.IGNORECASE), "direction"),
    (re.compile(r"\bidou\b", re.IGNORECASE), "movement"),
    (re.compile(r"\bhoukou\b", re.IGNORECASE), "direction"),
    (re.compile(r"\bken\b", re.IGNORECASE), "sword"),
    (re.compile(r"\bkaidan\b", re.IGNORECASE), "stairs"),
    (re.compile(r"\bkihon\b", re.IGNORECASE), "base"),
    (re.compile(r"\biti\b", re.IGNORECASE), "position"),
    (re.compile(r"\bdyoo\b", re.IGNORECASE), "state"),
    (re.compile(r"\bhosei\b", re.IGNORECASE), "correction"),
    (re.compile(r"\bkotei\b", re.IGNORECASE), "fixed"),
    (re.compile(r"\bhankan\b", re.IGNORECASE), "collision"),
    (re.compile(r"\btobiori\b", re.IGNORECASE), "jump-down"),
    (re.compile(r"\bkakuremino\b", re.IGNORECASE), "cape"),
    (re.compile(r"\byuka\b", re.IGNORECASE), "floor"),
    (re.compile(r"\bkui\b", re.IGNORECASE), "peg"),
    (re.compile(r"\bras?en\b", re.IGNORECASE), "spiral stairs"),
    (re.compile(r"\bdouzou\b", re.IGNORECASE), "statue"),
    (re.compile(r"\bhiku\b", re.IGNORECASE), "pull"),
    (re.compile(r"\byomu\b", re.IGNORECASE), "read"),
    (re.compile(r"\binoru\b", re.IGNORECASE), "pray"),
    (re.compile(r"\bkatsug[ui]\b", re.IGNORECASE), "carry"),
    (re.compile(r"\bshouji\b", re.IGNORECASE), "appearance"),
    (re.compile(r"\bnashi\b", re.IGNORECASE), "none"),
    (re.compile(r"\busagi\b", re.IGNORECASE), "bunny"),
    (re.compile(r"\bteki\b", re.IGNORECASE), "enemy"),
    (re.compile(r"\buusen\b", re.IGNORECASE), "priority"),
    (re.compile(r"\bjuni\b", re.IGNORECASE), "order"),
    (re.compile(r"\bconbear\b", re.IGNORECASE), "conveyor"),
    (re.compile(r"\bkaunto\b", re.IGNORECASE), "count"),
    (re.compile(r"\bcaunto\b", re.IGNORECASE), "count"),
    (re.compile(r"\bscrll\b", re.IGNORECASE), "scroll"),
    (re.compile(r"\badress\b", re.IGNORECASE), "address"),
    (re.compile(r"\bmesseg\b", re.IGNORECASE), "message"),
    (re.compile(r"\bmoji\b", re.IGNORECASE), "text character"),
    (re.compile(r"\baitem\b", re.IGNORECASE), "item"),
    (re.compile(r"\bmochi\b", re.IGNORECASE), "holding"),
    (re.compile(r"\btakara bako\b", re.IGNORECASE), "treasure chest"),
    (re.compile(r"\bhaka\b", re.IGNORECASE), "grave"),
    (re.compile(r"\brouya\b", re.IGNORECASE), "prison"),
    (re.compile(r"\bdanjyon\b", re.IGNORECASE), "dungeon"),
    (re.compile(r"\byoko\b", re.IGNORECASE), "horizontal"),
    (re.compile(r"\btate\b", re.IGNORECASE), "vertical"),
]


@dataclass(frozen=True)
class RustConst:
    name: str
    address: int
    path: Path
    line: int
    subsystem: str


@dataclass(frozen=True)
class SourceSymbol:
    label: str
    address: int
    path: Path
    line: int
    comment: str


@dataclass(frozen=True)
class RawEqu:
    label: str
    expr: str
    path: Path
    line: int
    comment: str


def rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def read_text(path: Path) -> str | None:
    try:
        data = path.read_bytes()
    except OSError:
        return None
    if b"\x00" in data[:4096]:
        return None
    for encoding in ("utf-8", "shift_jis", "latin-1"):
        try:
            return data.decode(encoding)
        except UnicodeDecodeError:
            continue
    return data.decode("latin-1", errors="replace")


def clean_source_comment(comment: str) -> str:
    return re.sub(r"\s+", " ", comment.strip().strip('"').strip())


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def module_for_line(text: str, offset: int) -> str | None:
    before = text[:offset]
    module_match = None
    for match in re.finditer(r"^\s*pub(?:\([^)]*\))?\s+mod\s+([a-z][a-z0-9_]*)\s*\{", before, re.MULTILINE):
        module_match = match
    return module_match.group(1) if module_match else None


def rust_subsystem(path: Path, text: str, offset: int, name: str) -> str:
    if path.name == "ram.rs":
        return module_for_line(text, offset) or "ram"
    prefix = name.split("_", 1)[0].lower()
    if prefix in {"link", "player", "swim", "swimcoll", "tiledetect"}:
        return "player"
    if prefix in {"sprite", "overlord", "garnish"}:
        return "sprite"
    if prefix in {"dung", "dungeon", "door"}:
        return "dungeon"
    if prefix in {"message", "messaging", "dialogue", "text", "vwf"}:
        return "messaging"
    if prefix in {"overworld", "ow"}:
        return "overworld"
    return path.stem if path.stem != "zelda_rtl" else "shared"


def scan_rust_consts() -> list[RustConst]:
    constants: list[RustConst] = []
    for path in sorted(SRC_ROOT.glob("*.rs")):
        text = path.read_text()
        patterns = [RUST_PRIVATE_CONST_RE]
        if path.name == "ram.rs":
            patterns.append(RUST_RAM_MODULE_CONST_RE)
        for pattern in patterns:
            for match in pattern.finditer(text):
                constants.append(
                    RustConst(
                        name=match.group(1),
                        address=int(match.group(2), 16),
                        path=path,
                        line=line_for_offset(text, match.start()),
                        subsystem=rust_subsystem(path, text, match.start(), match.group(1)),
                    )
                )
    return sorted(constants, key=lambda c: (c.address, rel(c.path, REPO_ROOT), c.name))


def parse_number(value: str) -> int | None:
    value = value.strip()
    if HEX_RE.match(value):
        return int(value[:-1], 16)
    if DEC_RE.match(value):
        return int(value, 10)
    return None


def resolve_expr(expr: str, labels: dict[str, int]) -> int | None:
    expr = expr.strip()
    number = parse_number(expr)
    if number is not None:
        return number
    if IDENT_RE.match(expr):
        return labels.get(expr)
    match = ADD_SUB_RE.match(expr)
    if not match:
        return None
    base_token = match.group("base")
    base = parse_number(base_token)
    if base is None:
        base = labels.get(base_token)
    delta = parse_number(match.group("delta"))
    if base is None or delta is None:
        return None
    if match.group("op") == "-":
        return base - delta
    return base + delta


def resolve_file_equ_symbols(raw_equs: list[RawEqu]) -> list[SourceSymbol]:
    labels: dict[str, int] = {}
    unresolved = list(raw_equs)

    for _ in range(20):
        changed = False
        next_unresolved: list[RawEqu] = []
        for equ in unresolved:
            address = resolve_expr(equ.expr, labels)
            if address is None:
                next_unresolved.append(equ)
                continue
            labels[equ.label] = address
            changed = True
        unresolved = next_unresolved
        if not changed or not unresolved:
            break

    symbols: list[SourceSymbol] = []
    for equ in raw_equs:
        address = resolve_expr(equ.expr, labels)
        if address is None or address < 0 or address > 0x1FFFF:
            continue
        symbols.append(
            SourceSymbol(
                label=equ.label,
                address=address,
                path=equ.path,
                line=equ.line,
                comment=equ.comment,
            )
        )
    return symbols


def scan_source_symbols(source_root: Path) -> tuple[list[SourceSymbol], list[dict[str, object]], Counter[str], Counter[str]]:
    symbols: list[SourceSymbol] = []
    files: list[dict[str, object]] = []
    suffix_counts: Counter[str] = Counter()
    top_dir_counts: Counter[str] = Counter()

    for path in sorted(source_root.rglob("*")):
        if not path.is_file():
            continue
        suffix = path.suffix.lower() or "<none>"
        suffix_counts[suffix] += 1
        try:
            top_dir = path.relative_to(source_root).parts[0]
        except ValueError:
            top_dir = "."
        top_dir_counts[top_dir] += 1
        size = path.stat().st_size
        text = read_text(path) if suffix in TEXT_SUFFIXES or suffix == "<none>" else None
        files.append(
            {
                "path": rel(path, source_root),
                "size": size,
                "suffix": suffix,
                "text": text is not None,
            }
        )
        if text is None:
            continue
        raw_equs: list[RawEqu] = []
        for line_no, line in enumerate(text.splitlines(), 1):
            match = EQU_RE.match(line)
            if not match:
                continue
            comment_match = COMMENT_RE.search(match.group("tail"))
            raw_equs.append(
                RawEqu(
                    label=match.group("label"),
                    expr=match.group("expr").strip(),
                    path=path,
                    line=line_no,
                    comment=clean_source_comment(comment_match.group("comment")) if comment_match else "",
                )
            )
        symbols.extend(resolve_file_equ_symbols(raw_equs))
    return sorted(symbols, key=lambda s: (s.address, rel(s.path, source_root), s.label)), files, suffix_counts, top_dir_counts


def scan_linker_maps(source_root: Path) -> list[dict[str, object]]:
    modules: list[dict[str, object]] = []
    for path in sorted(source_root.rglob("*.map")):
        text = read_text(path)
        if text is None:
            continue
        for line_no, line in enumerate(text.splitlines(), 1):
            match = MAP_MODULE_RE.match(line)
            if not match:
                continue
            start_bank, start_addr = match.group("start").split(":")
            end_bank, end_addr = match.group("end").split(":")
            modules.append(
                {
                    "map_path": rel(path, source_root),
                    "line": line_no,
                    "file": match.group("file"),
                    "module": match.group("module"),
                    "section": match.group("section"),
                    "start": match.group("start"),
                    "end": match.group("end"),
                    "size": int(match.group("size"), 16),
                    "start_bank": int(start_bank, 16),
                    "start_addr": int(start_addr, 16),
                    "end_bank": int(end_bank, 16),
                    "end_addr": int(end_addr, 16),
                }
            )
    return sorted(modules, key=lambda m: (str(m["map_path"]), int(m["start_bank"]), int(m["start_addr"])))


def cryptic_source_label(label: str) -> bool:
    if re.match(r"^C_[0-9A-F][A-Z][0-9A-F]$", label):
        return True
    if re.match(r"^MS_[0-9A-F]{3,5}$", label):
        return True
    if re.match(r"^(rm|dr)[0-9A-F]{3,5}$", label):
        return True
    return False


def label_is_intuitive(label: str) -> bool:
    if cryptic_source_label(label):
        return False
    if "_" in label:
        return True
    if len(label) >= 10:
        return True
    if re.search(r"(MSG|MES|MOJI|VWF|VRAM|DMA|RAM|MAP|LINK|DUNG|ENMY|SCR|CNT|WAIT|SPEED|PAL|OAM)", label):
        return True
    return False


def weak_rust_name(name: str) -> bool:
    return bool(WEAK_NAME_RE.search(name))


def translate_comment(comment: str) -> str:
    translated = comment.strip().strip('"').strip()
    if not translated:
        return ""
    for pattern, replacement in COMMENT_TRANSLATIONS:
        translated = pattern.sub(replacement, translated)
    translated = re.sub(r"\s+", " ", translated).strip()
    return translated


def comment_is_useful(comment: str) -> bool:
    translated = translate_comment(comment)
    if not translated:
        return False
    if translated in {"''", '"', "-"}:
        return False
    return bool(re.search(r"[A-Za-z]{4,}", translated))


def generic_source_label(label: str) -> bool:
    return bool(re.match(r"^(Z?WORK[0-9A-F]?|[PCI]WORK[0-9A-F]?|BFP[0-9A-FZ]{3})$", label))


def const_symbol_affinity(const: RustConst, symbol: SourceSymbol) -> int:
    rust = const.name
    source = f"{symbol.label} {symbol.comment}".upper()
    score = 0

    if "ATTRACT" in rust and ("TITLE DEMO" in source or symbol.label == "ZWORK"):
        score += 12
    if "LINK" in rust or "PLAYER" in rust:
        if re.match(r"^(PL|PY|PX|SPY|SPX)", symbol.label):
            score += 6
    if ("INDOORS" in rust or "DUNGEON" in rust) and re.search(r"(GMMODE|DJFLG|DANJYON|DUNGEON)", source):
        score += 12
    for axis in ("X", "Y", "Z"):
        if f"{axis}_COORD" in rust or f"VEL_{axis}" in rust or f"SUBPIXEL_{axis}" in rust:
            if f"{axis}-POS" in source or f"PL{axis}" in symbol.label or f"P{axis}" in symbol.label:
                score += 12
    if ("MESSAGE" in rust or "DIALOGUE" in rust or "TEXT" in rust) and re.search(r"(MSG|MES|MOJI|MESSAGE)", source):
        score += 10
    if ("NMI" in rust or "VBLANK" in rust) and re.search(r"(NMI|VMA|VRAM|DMA|BG|CG)", source):
        score += 8
    return score


def source_file_priority(symbol: SourceSymbol, source_root: Path) -> int:
    path = rel(symbol.path, source_root)
    if path == "us_asm/zel_ram.asm":
        return 100
    if path == "us_asm/zel_ram1.asm":
        return 95
    if path.startswith("us_asm/zel_char") or path.startswith("us_asm/tes_char"):
        return 5
    if path.startswith("us_asm/tes_"):
        return 10
    if path.startswith("us_asm/zel_"):
        return 50
    return 25


def source_symbol_priority(
    const: RustConst, symbol: SourceSymbol, source_root: Path
) -> tuple[int, int, bool, bool, bool, int, str]:
    return (
        const_symbol_affinity(const, symbol),
        source_file_priority(symbol, source_root),
        not cryptic_source_label(symbol.label),
        label_is_intuitive(symbol.label),
        not generic_source_label(symbol.label),
        len(symbol.comment),
        symbol.label,
    )


def crosswalk(
    rust_consts: list[RustConst], source_symbols: list[SourceSymbol], source_root: Path
) -> list[dict[str, object]]:
    by_addr: dict[int, list[SourceSymbol]] = defaultdict(list)
    for symbol in source_symbols:
        by_addr[symbol.address].append(symbol)

    rows: list[dict[str, object]] = []
    for const in rust_consts:
        matches = by_addr.get(const.address, [])
        if not matches:
            continue
        best = sorted(matches, key=lambda s: source_symbol_priority(const, s, source_root), reverse=True)[0]
        rows.append(
            {
                "address": const.address,
                "rust_name": const.name,
                "rust_path": rel(const.path, REPO_ROOT),
                "rust_line": const.line,
                "subsystem": const.subsystem,
                "source_label": best.label,
                "source_path": rel(best.path, source_root),
                "source_line": best.line,
                "source_comment": best.comment,
                "source_comment_en": translate_comment(best.comment),
                "source_label_intuitive": label_is_intuitive(best.label),
                "source_label_cryptic": cryptic_source_label(best.label),
                "source_comment_useful": comment_is_useful(best.comment),
                "rust_name_weak": weak_rust_name(const.name),
                "match_count": len(matches),
            }
        )
    return sorted(rows, key=lambda r: (int(r["address"]), str(r["rust_path"]), str(r["rust_name"])))


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def markdown_table(headers: list[str], rows: list[list[str]]) -> list[str]:
    lines = ["| " + " | ".join(headers) + " |", "|" + "|".join("---" for _ in headers) + "|"]
    for row in rows:
        lines.append("| " + " | ".join(cell.replace("\n", " ") for cell in row) + " |")
    return lines


def write_inventory(
    source_root: Path,
    files: list[dict[str, object]],
    suffix_counts: Counter[str],
    top_dir_counts: Counter[str],
    source_symbols: list[SourceSymbol],
) -> None:
    lines = [
        "# NES_Ver2 Source Inventory",
        "",
        f"Source root: `{source_root}`",
        f"Files scanned: {len(files)}",
        f"Text-like files parsed for symbols: {sum(1 for f in files if f['text'])}",
        f"Address-like `EQU` symbols mined: {len(source_symbols)}",
        "",
        "## Top-Level Directories",
        "",
    ]
    lines.extend(markdown_table(["Directory", "Files"], [[k, str(v)] for k, v in top_dir_counts.most_common()]))
    lines.extend(["", "## File Types", ""])
    lines.extend(markdown_table(["Suffix", "Files"], [[k, str(v)] for k, v in suffix_counts.most_common()]))
    lines.extend(["", "## High-Value Files", ""])
    interesting = [
        "us_asm/zel_ram.asm",
        "us_asm/zel_ram1.asm",
        "us_asm/zel_play.asm",
        "us_asm/zel_pysb.asm",
        "us_asm/zel_msge0.asm",
        "us_asm/zel_msge1.asm",
        "us_asm/zel_msge2.asm",
        "us_asm/zel_msge3.asm",
        "us_asm/zel_gmap.asm",
        "us_msg/bun/msg.DAT",
        "us_asm/zel_main.map",
        "us_asm/zel_main.sym",
    ]
    by_path = {str(f["path"]): f for f in files}
    rows = []
    for item in interesting:
        file = by_path.get(item)
        rows.append([f"`{item}`", "yes" if file else "missing", str(file["size"]) if file else ""])
    lines.extend(markdown_table(["Path", "Present", "Bytes"], rows))
    (OUT_DIR / "source_inventory.md").write_text("\n".join(lines) + "\n")


def write_crosswalk(rows: list[dict[str, object]]) -> None:
    lines = [
        "# NES_Ver2 RAM Symbol Crosswalk",
        "",
        "Generated by `python3 scripts/mine_nes_ver2_symbols.py --write`.",
        "NES_Ver2 labels and comments are treated as authority for meaning and provenance. Prefer the existing Rust name when it is clearer US-English; cryptic source labels remain evidence, not automatic Rust names.",
        "",
    ]
    table_rows = []
    for row in rows:
        table_rows.append(
            [
                f"`0x{int(row['address']):05x}`",
                f"`{row['rust_name']}`",
                str(row["subsystem"]),
                f"`{row['source_label']}`",
                str(row["source_comment"]),
                str(row["source_comment_en"]),
                f"`{row['source_path']}:{row['source_line']}`",
            ]
        )
    lines.extend(
        markdown_table(
            ["Address", "Rust", "Subsystem", "NES_Ver2 label", "Original comment", "US-English hint", "Source"],
            table_rows,
        )
    )
    (OUT_DIR / "ram_symbol_crosswalk.md").write_text("\n".join(lines) + "\n")


def write_findings(rows: list[dict[str, object]], source_symbols: list[SourceSymbol]) -> None:
    source_by_addr: dict[int, list[SourceSymbol]] = defaultdict(list)
    for symbol in source_symbols:
        source_by_addr[symbol.address].append(symbol)

    source_evidence = [
        row
        for row in rows
        if (
            not row["source_label_cryptic"]
            and not generic_source_label(str(row["source_label"]))
            and (row["source_label_intuitive"] or row["source_comment_useful"])
            and row["source_label"] != row["rust_name"]
            and not str(row["source_label"]).startswith("MS_")
        )
    ]
    weak_with_source = [
        row for row in rows if row["rust_name_weak"] and (row["source_label_intuitive"] or row["source_comment_useful"])
    ]
    ambiguous = [row for row in rows if int(row["match_count"]) > 1][:50]

    lines = [
        "# NES_Ver2 Naming Evidence",
        "",
        "This is the review surface for checking Rust RAM names against original NES_Ver2 labels and comments.",
        "",
        "## Source Evidence Review Queue",
        "",
        "These rows have useful NES_Ver2 evidence that differs from the Rust name. Original comments are preserved, and the US-English hint is a conservative glossary translation. Keep the Rust name when it is clearer; use this table to confirm meaning or catch names that are genuinely wrong.",
        "",
    ]
    lines.extend(
        markdown_table(
            ["Address", "Rust", "NES_Ver2", "Original comment", "US-English hint", "Source"],
            [
                [
                    f"`0x{int(r['address']):05x}`",
                    f"`{r['rust_name']}`",
                    f"`{r['source_label']}`",
                    str(r["source_comment"]),
                    str(r["source_comment_en"]),
                    f"`{r['source_path']}:{r['source_line']}`",
                ]
                for r in source_evidence[:200]
            ],
        )
    )
    lines.extend(["", "## Weak Rust Names With Source Evidence", ""])
    lines.extend(
        markdown_table(
            ["Address", "Rust", "NES_Ver2", "Original comment", "US-English hint", "Source"],
            [
                [
                    f"`0x{int(r['address']):05x}`",
                    f"`{r['rust_name']}`",
                    f"`{r['source_label']}`",
                    str(r["source_comment"]),
                    str(r["source_comment_en"]),
                    f"`{r['source_path']}:{r['source_line']}`",
                ]
                for r in weak_with_source
            ],
        )
    )
    lines.extend(["", "## Ambiguous Address Matches", ""])
    lines.extend(
        markdown_table(
            ["Address", "Rust", "Chosen label", "Match count"],
            [
                [
                    f"`0x{int(r['address']):05x}`",
                    f"`{r['rust_name']}`",
                    f"`{r['source_label']}`",
                    str(r["match_count"]),
                ]
                for r in ambiguous
            ],
        )
    )
    lines.extend(["", "## Source Coverage", ""])
    lines.append(f"- Source addresses mined: {len(source_by_addr)} unique offsets")
    lines.append(f"- Rust constants with at least one source match: {len(rows)}")
    lines.append(f"- Source evidence review rows shown: {min(len(source_evidence), 200)} of {len(source_evidence)}")
    lines.append(f"- Weak Rust names with source evidence: {len(weak_with_source)}")
    (OUT_DIR / "valuable_findings.md").write_text("\n".join(lines) + "\n")


def write_linker_maps(modules: list[dict[str, object]]) -> None:
    lines = [
        "# NES_Ver2 Linker Map Modules",
        "",
        "Generated from readable `*.map` files under NES_Ver2. This is useful for cross-referencing ROM bank ownership while porting routines.",
        "",
    ]
    rows = [
        [
            f"`{m['map_path']}`",
            f"`{m['module']}`",
            f"`{m['file']}`",
            str(m["section"]),
            f"`{m['start']}`",
            f"`{m['end']}`",
            f"`0x{int(m['size']):04x}`",
        ]
        for m in modules
    ]
    lines.extend(markdown_table(["Map", "Module", "Object", "Section", "Start", "End", "Size"], rows))
    (OUT_DIR / "linker_map_modules.md").write_text("\n".join(lines) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, default=DEFAULT_SOURCE_ROOT)
    parser.add_argument("--write", action="store_true", help=f"write docs under {OUT_DIR.relative_to(REPO_ROOT)}")
    args = parser.parse_args()

    source_root = args.source_root.expanduser().resolve()
    rust_consts = scan_rust_consts()
    source_symbols, files, suffix_counts, top_dir_counts = scan_source_symbols(source_root)
    linker_modules = scan_linker_maps(source_root)
    rows = crosswalk(rust_consts, source_symbols, source_root)

    summary = {
        "source_root": str(source_root),
        "files": len(files),
        "text_files": sum(1 for file in files if file["text"]),
        "source_symbols": len(source_symbols),
        "rust_ram_constants": len(rust_consts),
        "crosswalk_rows": len(rows),
        "linker_map_modules": len(linker_modules),
    }

    if args.write:
        OUT_DIR.mkdir(parents=True, exist_ok=True)
        write_json(OUT_DIR / "ram_symbol_crosswalk.json", rows)
        write_json(
            OUT_DIR / "source_symbol_index.json",
            [
                {
                    "label": s.label,
                    "address": s.address,
                    "path": rel(s.path, source_root),
                    "line": s.line,
                    "comment": s.comment,
                }
                for s in source_symbols
            ],
        )
        write_json(OUT_DIR / "inventory.json", {"summary": summary, "files": files})
        write_json(OUT_DIR / "linker_map_modules.json", linker_modules)
        write_inventory(source_root, files, suffix_counts, top_dir_counts, source_symbols)
        write_crosswalk(rows)
        write_findings(rows, source_symbols)
        write_linker_maps(linker_modules)

    print(json.dumps(summary, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
