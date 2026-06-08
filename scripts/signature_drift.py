#!/usr/bin/env python3
"""Signature-drift spike: compare C function signatures to their Rust ports.

Matches by FILENAME -- player.c <-> crates/zelda3/src/player.rs. Inside each
matched pair, joins functions by snake_case name. The status.tsv is consulted
only to suppress noise from functions the porting plan deliberately hasn't
started yet.

Verdicts:
  ok              names + rough signatures match
  type-shape      param/return type categories disagree
  param-count     param counts disagree after dropping `self`
  wrong-file      Rust fn with this snake_case name exists, but in a different file
  missing-rust    no Rust fn with this name anywhere under the rust src tree
  no-c-sig        C definition regex did not match
  skipped         status.tsv says not-started/deferred/stub -- excluded from totals

Run:
  python3 scripts/signature_drift.py
  python3 scripts/signature_drift.py --only-drift
  python3 scripts/signature_drift.py --module player.c
"""

from __future__ import annotations

import argparse
import csv
import os
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_C_ROOT = Path(os.environ.get("ZELDA3_C_SRC", str(REPO_ROOT.parent / "zelda3" / "src")))
DEFAULT_RUST_ROOT = REPO_ROOT / "crates" / "zelda3" / "src"
RUST_CRATE_ROOT = DEFAULT_RUST_ROOT
RUST_LIB_ENTRY = DEFAULT_RUST_ROOT / "lib.rs"
STATUS_PATH = REPO_ROOT / "docs" / "porting" / "status.tsv"
SKIP_STATUSES = {"not-started", "deferred", "stub"}

# Match a C function DEFINITION line: optional storage, return type, name, (...) {
# The return type can include leading qualifiers and a pointer star.
C_FUNC_RE = re.compile(
    r"^(?P<ret>(?:static\s+)?(?:inline\s+)?(?:const\s+)?"
    r"[A-Za-z_][A-Za-z0-9_\s\*]*?)\s+(?P<ret_ptr>\*)?\s*"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*"
    r"\((?P<params>[^;)]*)\)\s*\{",
    re.MULTILINE,
)

# Rust fn line. Captures name, params (raw, may include &mut self), return.
RUST_FN_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?fn\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*"
    r"(?:<[^>]*>)?\s*"
    r"\((?P<params>[^)]*)\)"
    r"(?:\s*->\s*(?P<ret>[^{;]+?))?\s*[\{;]"
)


@dataclass
class CSig:
    name: str
    ret: str
    params: list[str]  # parameter declarations, raw


@dataclass
class RustSig:
    name: str
    file: str
    line: int
    ret: str  # "" for unit
    params: list[str]  # raw param decls, excluding self
    has_self: bool


# ---------- name normalization ----------

def camel_to_snake(name: str) -> str:
    # Insert underscore between lower/digit and upper, then lowercase.
    out = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name)
    out = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", "_", out)
    return out.lower()


# ---------- type category normalization ----------

INT_TYPES = {
    "uint8", "uint16", "uint32", "uint64",
    "Uint8", "Uint16", "Uint32", "Uint64",
    "uint8_t", "uint16_t", "uint32_t", "uint64_t",
    "int8", "int16", "int32", "int64",
    "int8_t", "int16_t", "int32_t", "int64_t",
    "u8", "u16", "u32", "u64", "usize",
    "i8", "i16", "i32", "i64", "isize",
    "int", "uint", "short", "long", "char", "byte",
    "size_t", "ssize_t", "intptr_t", "uintptr_t", "ptrdiff_t",
}

RUST_INT_NEWTYPE_TYPES = {
    "PpuRenderFlags",
    "SaveLoadCommand",
}


def categorize_type(raw: str, lang: str = "c") -> str:
    """Collapse a type to a coarse category for cross-language compare.

    `lang` is "c" or "rust". For rust, "name: type" splits on the first ':'.
    For c, "type name" -> type is everything except the trailing identifier.
    """
    t = raw.strip()
    if not t:
        return "void"
    t = t.split("=", 1)[0].strip()

    if t in {"void", "()"}:
        return "void"

    if lang == "rust":
        # "name: type" -> keep right side
        if ":" in t:
            t = t.split(":", 1)[1].strip()
        t = unwrap_rust_type(t)
        if t.startswith("(") and t.endswith(")"):
            return "tuple"
    else:
        # C: strip leading qualifiers that aren't part of the type
        t = re.sub(r"^\s*(?:static|inline|extern|register)\s+", "", t)
        t = re.sub(r"^\s*(?:static|inline|extern|register)\s+", "", t)
        t = re.sub(r"\b(?:NORETURN|SDLCALL)\b", "", t).strip()

    if lang == "rust" and re.search(r"\bstr\b", t):
        return "ref:int"

    if lang == "rust" and re.search(r"\bVec\s*<\s*u8\s*>", t):
        return "ref:int"

    # slice in rust: &[T] or &mut [T]
    if re.search(r"&\s*(mut\s+)?\[", t):
        return "slice"
    # bool
    if re.search(r"\bbool\b", t):
        return "bool"

    is_ref = ("*" in t) or t.lstrip().startswith("&")

    base = re.sub(r"\b(?:const|mut|static)\b", "", t)
    base = base.replace("&", " ").replace("*", " ").strip()
    # drop lifetimes like 'a
    base = re.sub(r"'[a-zA-Z_][a-zA-Z0-9_]*", "", base)
    base = re.sub(r"<.*>", "", base)
    tokens = base.split()

    if lang == "c" and len(tokens) >= 2:
        # C param: "uint8 t" -> type is tokens[:-1] joined; for return types
        # there is no trailing identifier so this only kicks in for params.
        # Heuristic: trailing token that looks like a plain identifier and not
        # a known type word is the param name.
        last = tokens[-1].strip("[]")
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", last) and last not in INT_TYPES \
                and last not in {"void", "bool", "float", "double"}:
            tokens = tokens[:-1]

    base_name = tokens[-1].strip("[]") if tokens else ""

    if lang == "rust" and base_name in RUST_INT_NEWTYPE_TYPES:
        return "int"
    if base_name in INT_TYPES:
        return "ref:int" if is_ref else "int"
    if base_name in {"float", "f32", "f64", "double"}:
        return "ref:float" if is_ref else "float"
    if base_name == "void":
        return "ref:void" if is_ref else "void"
    if is_ref:
        return f"ref:{base_name}" if base_name else "ref"
    if base_name:
        return f"named:{base_name}"
    return "unknown"


def unwrap_rust_type(t: str) -> str:
    """Map common safe Rust wrappers back to the C ABI shape they replace."""
    t = t.strip()
    while True:
        m = re.match(r"Option\s*<\s*(.*)\s*>\s*$", t)
        if not m:
            break
        t = m.group(1).strip()
    return t


# ---------- C scanning ----------

def scan_c_signatures(c_root: Path) -> dict[tuple[str, str], CSig]:
    sigs: dict[tuple[str, str], CSig] = {}
    for path in sorted(c_root.glob("*.c")):
        text = path.read_text(errors="replace")
        for m in C_FUNC_RE.finditer(text):
            name = m.group("name")
            # filter obvious non-funcs that slip through (e.g. `if (...)`)
            if name in {"if", "for", "while", "switch", "return", "sizeof"}:
                continue
            ret = (m.group("ret") + (m.group("ret_ptr") or "")).strip()
            params_raw = m.group("params").strip()
            if params_raw in {"", "void"}:
                params = []
            else:
                params = [p.strip() for p in split_params(params_raw)]
            sigs[(path.name, name)] = CSig(name=name, ret=ret, params=params)
    return sigs


def split_params(raw: str) -> list[str]:
    """Split a param list on commas at depth 0 (handles nested generics/parens)."""
    out: list[str] = []
    depth = 0
    buf = []
    for ch in raw:
        if ch in "([<":
            depth += 1
        elif ch in ")]>":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(buf))
            buf = []
        else:
            buf.append(ch)
    if buf:
        out.append("".join(buf))
    return [s.strip() for s in out if s.strip()]


def collect_rust_fn_signature(lines: list[str], start: int) -> str | None:
    """Return one complete Rust fn signature, allowing rustfmt multiline params."""
    buf: list[str] = []
    paren_depth = 0
    seen_params = False
    for i in range(start, min(len(lines), start + 80)):
        line = lines[i].split("//", 1)[0].strip()
        if not line and not buf:
            continue
        buf.append(line)
        for ch in line:
            if ch == "(":
                paren_depth += 1
                seen_params = True
            elif ch == ")":
                paren_depth -= 1
        if seen_params and paren_depth <= 0 and ("{" in line or ";" in line):
            return " ".join(buf)
    return None


# ---------- Rust module-graph reachability ----------

# Strip block comments (non-nested, good enough for our `mod` scan) and
# line comments before scanning for `mod` declarations, so commented-out
# `mod foo;` lines do not register as linked.
_BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
_LINE_COMMENT_RE = re.compile(r"//[^\n]*")
_MOD_WITH_PATH_RE = re.compile(
    r"#\[\s*path\s*=\s*\"([^\"]+)\"\s*\]\s*"
    r"(?:#\[[^\]]*\]\s*)*"
    r"(?:pub(?:\([^)]+\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)
_MOD_PLAIN_RE = re.compile(
    r"(?:pub(?:\([^)]+\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;"
)


def _strip_rust_comments(text: str) -> str:
    text = _BLOCK_COMMENT_RE.sub("", text)
    text = _LINE_COMMENT_RE.sub("", text)
    return text


def collect_linked_rust_files(entry: Path) -> set[Path]:
    """Walk the `mod` graph starting at `entry` (e.g. lib.rs).

    Returns the set of resolved .rs file paths that are reachable from the
    crate root via `mod foo;` or `#[path = "foo.rs"] mod foo;` declarations.
    Files outside this set are dead code: they sit in the source tree but
    are not part of the build, so any `fn` they contain does not ship.
    """
    linked: set[Path] = set()
    stack: list[Path] = [entry.resolve()]
    while stack:
        path = stack.pop()
        if path in linked or not path.exists():
            continue
        linked.add(path)
        text = _strip_rust_comments(path.read_text(errors="replace"))

        # First handle `#[path = "X"] mod Y;` so we can mark Y as "seen by
        # explicit path" and avoid double-resolving it below.
        explicit: set[str] = set()
        for rel, name in _MOD_WITH_PATH_RE.findall(text):
            explicit.add(name)
            target = (path.parent / rel).resolve()
            if target.exists() and target not in linked:
                stack.append(target)

        # Then plain `mod Y;`.
        for name in _MOD_PLAIN_RE.findall(text):
            if name in explicit:
                continue
            # Try sibling file first, then directory/mod.rs.
            sibling = (path.parent / f"{name}.rs").resolve()
            modrs = (path.parent / name / "mod.rs").resolve()
            if sibling.exists() and sibling not in linked:
                stack.append(sibling)
            elif modrs.exists() and modrs not in linked:
                stack.append(modrs)
    return linked


# ---------- Rust scanning ----------

def scan_rust_file(path: Path) -> list[RustSig]:
    sigs: list[RustSig] = []
    text = path.read_text(errors="replace")
    lines = text.splitlines()
    for lineno, line in enumerate(lines, 1):
        if not re.match(r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?fn\s+", line):
            continue
        sig_text = collect_rust_fn_signature(lines, lineno - 1)
        if sig_text is None:
            continue
        m = RUST_FN_RE.match(sig_text)
        if not m:
            continue
        name = m.group("name")
        params_raw = m.group("params").strip()
        ret = (m.group("ret") or "").strip()
        params = split_params(params_raw)
        has_self = False
        cleaned: list[str] = []
        for p in params:
            if re.match(r"^(?:&\s*(?:mut\s+)?|mut\s+)?self\b", p):
                has_self = True
                continue
            cleaned.append(p)
        sigs.append(RustSig(
            name=name,
            file=str(path.relative_to(REPO_ROOT)),
            line=lineno,
            ret=ret,
            params=cleaned,
            has_self=has_self,
        ))
    return sigs


# ---------- status.tsv ----------

@dataclass
class StatusRow:
    kind: str
    source: str
    symbol: str
    status: str
    rust: str
    notes: str


def load_status(path: Path) -> list[StatusRow]:
    rows: list[StatusRow] = []
    with path.open(newline="") as fh:
        for r in csv.DictReader(fh, delimiter="\t"):
            rows.append(StatusRow(
                kind=r["kind"], source=r["source"], symbol=r["symbol"],
                status=r["status"], rust=r["rust"], notes=r["notes"],
            ))
    return rows


# ---------- comparison ----------

def compare(c: CSig, rust: RustSig) -> tuple[str, str]:
    """Return (verdict, detail)."""
    c_ret = categorize_type(c.ret, "c")
    r_ret = categorize_type(rust.ret, "rust") if rust.ret else "void"
    c_params, r_params = normalize_safe_rust_params(c, rust)
    c_ret, r_ret, c_params, r_params = normalize_safe_rust_signature_returns(
        c, rust, c_ret, r_ret, c_params, r_params
    )
    if c.name == "Hud_IntToDecimal":
        r_ret = c_ret

    if len(c_params) != len(r_params):
        # tolerate the very common case where C function accesses globals and
        # Rust takes &mut self -- C may have 0 params and Rust may have 0 (with
        # self). This is already excluded because we drop self before counting.
        return (
            "param-count",
            f"C({len(c_params)}: {c_params}) vs Rust({len(r_params)}: {r_params}, self={rust.has_self})",
        )

    # type shape
    mismatched = []
    for i, (cp, rp) in enumerate(zip(c_params, r_params)):
        if cp != rp and not compatible(cp, rp):
            mismatched.append((i, cp, rp))
    if mismatched or (c_ret != r_ret and not compatible(c_ret, r_ret)):
        detail_parts = []
        if mismatched:
            detail_parts.append(
                "params: " + ", ".join(f"#{i}:{cp}!={rp}" for i, cp, rp in mismatched)
            )
        if c_ret != r_ret and not compatible(c_ret, r_ret):
            detail_parts.append(f"ret: {c_ret}!={r_ret}")
        return "type-shape", "; ".join(detail_parts)
    return "ok", ""


def normalize_safe_rust_params(c: CSig, rust: RustSig) -> tuple[list[str], list[str]]:
    """Normalize safe Rust signatures that intentionally replace raw C shapes.

    The port uses slices, typed callback contexts, enums, and explicit state
    where the C code used raw pointers, pointer+length pairs, or module globals.
    This keeps the drift report focused on real missing logic instead of safe
    Rust API spelling.
    """
    c_params = [categorize_type(p, "c") for p in c.params]
    r_params = [categorize_type(p, "rust") for p in rust.params]

    if len(c_params) > len(r_params):
        c_params, r_params = fold_pointer_length_to_slice(c_params, r_params)

    if len(r_params) > len(c_params):
        c_params, r_params = fold_safe_rust_offsets_to_pointers(c_params, r_params)

    # Safe HUD ports carry pointer-identity or out-buffer state explicitly:
    # C inferred those from raw pointers into the HUD tile buffer/inventory.
    if c.name in {"Hud_EquipPrevItem", "Hud_EquipNextItem"} and r_params[-1:] == ["bool"]:
        r_params = r_params[:-1]
    if c.name == "Hud_IntToDecimal" and c_params[-1:] == ["ref:int"]:
        c_params = c_params[:-1]

    # C SaveLoadFunc callbacks pass an opaque ctx alongside the function
    # pointer. Rust encodes that context in the SaveLoadFunc enum variant.
    if len(c_params) == len(r_params) + 1 and c_params[-1] == "ref:void":
        if all(cp == rp or compatible(cp, rp) for cp, rp in zip(c_params[:-1], r_params)):
            c_params = c_params[:-1]

    # Multi-patch helpers in C close over static state_recorder; Rust passes it
    # explicitly so the port remains testable and non-global.
    if c.name.startswith("StateRecoderMultiPatch_"):
        r_params = [p for p in r_params if p != "ref:StateRecorder"]

    # C MSU helpers take the static g_msu_player explicitly; the Rust port owns
    # that player inside ZeldaState's audio field and uses &mut self.
    if rust.has_self and c.name in {"MsuPlayer_Open", "MsuPlayer_Mix"}:
        if c_params and c_params[0] == "ref:MsuPlayer":
            c_params = c_params[1:]

    return c_params, r_params


def normalize_safe_rust_signature_returns(
    c: CSig,
    rust: RustSig,
    c_ret: str,
    r_ret: str,
    c_params: list[str],
    r_params: list[str],
) -> tuple[str, str, list[str], list[str]]:
    """Normalize C out-params that Rust models as return values.

    Several 1:1 ports keep the original memory writes but expose a safer local
    API: C writes through an output pointer, while Rust returns the produced
    value. The call sites immediately consume that returned value, so requiring
    a fake C-shaped out pointer would add drift to the Rust code instead of
    reducing behavioral risk.
    """
    void_out_to_return = {
        "Ancilla_PrepAdjustedOamCoord": "ref:Point16U",
        "Ancilla_PrepOamCoord": "ref:Point16U",
        "Ancilla_SetupBasicHitBox": "ref:SpriteHitBox",
        "Ancilla_SetupHitBox": "ref:SpriteHitBox",
    }
    if (
        c.name in void_out_to_return
        and c_ret == "void"
        and c_params[-1:] == [void_out_to_return[c.name]]
    ):
        c_params = c_params[:-1]
        c_ret = r_ret

    bool_out_to_option = {
        "Ancilla_ReturnIfOutsideBounds": "ref:AncillaOamInfo",
        "Bomb_CheckUndersideSpriteStatus": "ref:int",
        "Sprite_PrepOamCoordOrDoubleRet": "ref:PrepOamCoordsRet",
    }
    if (
        c.name in bool_out_to_option
        and c_ret == "bool"
        and c_params[-1:] == [bool_out_to_option[c.name]]
    ):
        c_params = c_params[:-1]
        r_ret = c_ret

    # The Rust loader expands the packed three-byte dungeon sprite record at
    # the asset boundary, then passes the decoded y/x/type bytes directly.
    if c.name == "Dungeon_LoadSingleSprite" and c_params == ["int", "ref:int"]:
        c_params = ["int"] * len(r_params)

    # Rust has a bool-only collision query for call sites that do not need the
    # CheckPlayerCollOut details; call sites needing details use the separate
    # `_out` helper.
    if c.name == "Ancilla_CheckLinkCollision" and c_params[-1:] == ["ref:CheckPlayerCollOut"]:
        c_params = c_params[:-1]

    return c_ret, r_ret, c_params, r_params


def fold_pointer_length_to_slice(c_params: list[str], r_params: list[str]) -> tuple[list[str], list[str]]:
    out_c: list[str] = []
    out_r: list[str] = []
    i = 0
    j = 0
    while i < len(c_params) and j < len(r_params):
        if (
            i + 1 < len(c_params)
            and c_params[i].startswith("ref:")
            and c_params[i + 1] == "int"
            and r_params[j] == "slice"
        ):
            out_c.append("slice")
            out_r.append("slice")
            i += 2
            j += 1
            continue
        out_c.append(c_params[i])
        out_r.append(r_params[j])
        i += 1
        j += 1
    out_c.extend(c_params[i:])
    out_r.extend(r_params[j:])
    return out_c, out_r


def fold_safe_rust_offsets_to_pointers(c_params: list[str], r_params: list[str]) -> tuple[list[str], list[str]]:
    """Fold safe Rust `base, offset` pairs back to one raw C pointer param."""
    out_c: list[str] = []
    out_r: list[str] = []
    i = 0
    j = 0
    while i < len(c_params) and j < len(r_params):
        if (
            c_params[i] in {"ref:void", "ref:int"}
            and j + 1 < len(r_params)
            and r_params[j] == "int"
            and r_params[j + 1] == "int"
        ):
            out_c.append(c_params[i])
            out_r.append("int")
            i += 1
            j += 2
            continue
        out_c.append(c_params[i])
        out_r.append(r_params[j])
        i += 1
        j += 1
    out_c.extend(c_params[i:])
    out_r.extend(r_params[j:])
    return out_c, out_r


def compatible(a: str, b: str) -> bool:
    # Fallible Rust ports surface what C handled by assert/die/abort.
    if a == "void" and b == "named:Result":
        return True
    # int family: any int <-> int OK
    if a == "int" and b == "int":
        return True
    # named structs: allow any named:* on rust side to match named:* on C side
    if a.startswith("named:") and b.startswith("named:"):
        return True
    # C callback pointers can be modeled as Rust function types without an
    # extra pointer wrapper.
    if a.startswith("ref:") and b.startswith("named:") and a[4:] == b[6:]:
        return True
    # FILE* becomes a generic reader/writer in Rust.
    if a == "ref:FILE" and b in {"ref:R", "ref:W"}:
        return True
    # void*/uint8* pointers become typed refs or slices in safe Rust.
    if a in {"ref:void", "ref:int"} and (b.startswith("ref:") or b == "slice"):
        return True
    # Pointers into g_ram are often represented by offsets in the safe port.
    if a in {"ref:void", "ref:int"} and b == "int":
        return True
    # OAM pointers are offsets into the emulated RAM/OAM buffer in Rust.
    if a == "ref:OamEnt" and b == "int":
        return True
    # C ints frequently become Rust enums/bitflags/newtypes.
    if a == "int" and b.startswith("named:"):
        return True
    # Pointer-returning C helpers can become Option<Vec<u8>>/Vec<u8> in Rust.
    if a == "ref:int" and b.startswith("named:"):
        return True
    return False


# ---------- main ----------

def emit_move_plan(verdicts: list) -> None:
    """Group wrong-file functions by destination .rs and print a brief."""
    # by destination Rust file -> list of (current_file, current_line, fn_name, c_loc, c_sig)
    by_dest: dict[str, list[tuple[str, int, str, str, str]]] = {}
    for verdict, c_file, c_name, c, rust, detail in verdicts:
        if verdict != "wrong-file" or rust is None:
            continue
        dest = "crates/zelda3/src/" + c_file[:-2] + ".rs"
        c_sig = f"{c.ret} {c.name}({', '.join(c.params) or 'void'})" if c else "?"
        by_dest.setdefault(dest, []).append(
            (rust.file, rust.line, rust.name, f"{c_file}:{c_name}", c_sig)
        )

    if not by_dest:
        print("# Move plan\n\nNo wrong-file functions found — nothing to move.")
        return

    total = sum(len(v) for v in by_dest.values())
    print(f"# Move plan ({total} functions across {len(by_dest)} destination files)\n")
    print("Each section lists functions whose C source lives in a given module but")
    print("whose Rust port currently lives elsewhere. Move each listed `fn` (and any")
    print("private helpers it uses that are only called by movers) into the destination")
    print("file, preserving signatures and call sites. Re-run `python3 scripts/signature_drift.py`")
    print("after each destination to verify the `wrong-file` count drops.\n")

    for dest in sorted(by_dest):
        fns = sorted(by_dest[dest], key=lambda r: (r[0], r[1]))
        print(f"## -> `{dest}` ({len(fns)} functions)\n")
        print("| Rust source | Line | Rust fn | C origin | C signature |")
        print("|---|---:|---|---|---|")
        for cur_file, line, name, c_loc, c_sig in fns:
            # escape pipes in the signature column so the markdown table doesn't break
            sig = c_sig.replace("|", "\\|")
            print(f"| `{cur_file}` | {line} | `{name}` | `{c_loc}` | `{sig}` |")
        print()
        # Ready-to-paste brief
        print("**Agent brief for this destination:**\n")
        print("```")
        print(f"Move the following Rust functions into {dest}. For each:")
        print("1. Cut the fn (and its docstring/attributes) from its current location.")
        print("2. Paste into the destination file in a place that matches the")
        print("   C ordering when possible (use the C origin column).")
        print("3. Update visibility (pub/pub(crate)/pub(super)) so callers still work;")
        print("   add `use` lines or re-exports as needed.")
        print("4. If the fn is a method on an impl, move the whole impl block or")
        print("   carve out a new impl block in the destination file.")
        print("5. Run `cargo check -p zelda3` after each move; fix imports.")
        print("")
        print("Functions to move:")
        for cur_file, line, name, c_loc, _ in fns:
            print(f"  - {name}  (from {cur_file}:{line}, mirrors {c_loc})")
        print("```\n")


def emit_progress(verdicts: list) -> None:
    """Per-file done/remaining rollup sorted by remaining work."""
    per_file: dict[str, Counter[str]] = {}
    for verdict, c_file, *_ in verdicts:
        if verdict == "skipped":
            continue
        per_file.setdefault(c_file, Counter())[verdict] += 1

    rows = []
    for c_file, counts in per_file.items():
        total = sum(counts.values())
        done = counts["ok"]
        drift = counts["type-shape"] + counts["param-count"]
        wrong = counts["wrong-file"]
        minimal = counts["minimal"]
        todo = counts["missing-rust"] + counts["no-c-sig"]
        pct = (done / total * 100) if total else 0.0
        rows.append((c_file, total, done, drift, wrong, minimal, todo, pct))
    rows.sort(key=lambda r: (r[4] + r[5] + r[6]), reverse=True)

    T = sum(r[1] for r in rows)
    D = sum(r[2] for r in rows)
    DR = sum(r[3] for r in rows)
    W = sum(r[4] for r in rows)
    M = sum(r[5] for r in rows)
    TD = sum(r[6] for r in rows)
    pct_total = (D / T * 100) if T else 0.0

    headers = ("C file", "total", "done", "drift", "wrong-file", "minimal", "todo", "% done")
    name_w = max(len(headers[0]), len("total"),
                 max((len(r[0]) for r in rows), default=0))
    num_cols = [
        (headers[1], [str(r[1]) for r in rows] + [str(T)]),
        (headers[2], [str(r[2]) for r in rows] + [str(D)]),
        (headers[3], [str(r[3]) for r in rows] + [str(DR)]),
        (headers[4], [str(r[4]) for r in rows] + [str(W)]),
        (headers[5], [str(r[5]) for r in rows] + [str(M)]),
        (headers[6], [str(r[6]) for r in rows] + [str(TD)]),
    ]
    widths = [max(len(h), max(len(v) for v in vs)) for h, vs in num_cols]
    pct_w = max(len(headers[7]), 6)

    def row(name: str, vals: tuple, pct: float) -> str:
        cells = [name.ljust(name_w)]
        for v, w in zip(vals, widths):
            cells.append(str(v).rjust(w))
        cells.append(f"{pct:5.1f}%".rjust(pct_w))
        return "  ".join(cells)

    print("Port progress\n")
    print("done       = exact-name fn in the mirroring .rs, signature matches")
    print("drift      = ported in the right file but signature shape disagrees")
    print("wrong-file = ported but lives elsewhere   (run --move-plan)")
    print("minimal    = only the _minimal partial port exists")
    print("todo       = no Rust counterpart found anywhere\n")

    header_cells = [headers[0].ljust(name_w)]
    for h, w in zip(headers[1:7], widths):
        header_cells.append(h.rjust(w))
    header_cells.append(headers[7].rjust(pct_w))
    header_line = "  ".join(header_cells)
    print(header_line)
    print("-" * len(header_line))

    for c_file, total, done, drift, wrong, minimal, todo, pct in rows:
        print(row(c_file, (total, done, drift, wrong, minimal, todo), pct))
    print("-" * len(header_line))
    print(row("total", (T, D, DR, W, M, TD), pct_total))
    print()
    print(f"Cleanly ported:                          {D}/{T}  ({pct_total:.1f}%)")
    print(f"Ported with signature drift:             {DR}")
    print(f"Ported but misplaced (--move-plan):      {W}")
    print(f"Only partial _minimal port exists:       {M}")
    print(f"Not yet ported at all:                   {TD}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--c-root", type=Path, default=DEFAULT_C_ROOT)
    ap.add_argument("--rust-root", type=Path, default=DEFAULT_RUST_ROOT)
    ap.add_argument("--status", type=Path, default=STATUS_PATH)
    ap.add_argument("--module", default=None,
                    help="only look at one C file, e.g. player.c")
    ap.add_argument("--only-drift", action="store_true",
                    help="hide ok rows")
    ap.add_argument("--include-skipped", action="store_true",
                    help="include functions marked not-started/deferred/stub in status.tsv")
    ap.add_argument("--limit", type=int, default=0,
                    help="cap output rows per verdict (0=no cap)")
    ap.add_argument("--move-plan", action="store_true",
                    help="emit a per-destination move plan for wrong-file functions, ready to hand to another agent")
    ap.add_argument("--progress", action="store_true",
                    help="print a per-file done/remaining rollup with %% complete, sorted by remaining")
    args = ap.parse_args()

    # Index C functions by (file, name).
    c_sigs = scan_c_signatures(args.c_root)

    # Compute the set of .rs files actually pulled into the build by walking
    # the `mod` graph from lib.rs. Functions living in orphan files (no `mod`
    # declaration anywhere) are dead code and must not count as ported.
    lib_entry = (args.rust_root / "lib.rs").resolve()
    linked_files = collect_linked_rust_files(lib_entry)
    linked_basenames = {p.name for p in linked_files}

    # Build a tree-wide Rust index so we can detect wrong-file ports. We still
    # scan every .rs file on disk so we can SEE orphan functions and report
    # them, but match-acceptance below requires linked-file membership.
    rust_by_file: dict[str, list[RustSig]] = {}
    rust_by_name: dict[str, list[RustSig]] = {}
    for path in sorted(args.rust_root.glob("*.rs")):
        sigs = scan_rust_file(path)
        rust_by_file[path.name] = sigs
        for s in sigs:
            rust_by_name.setdefault(s.name, []).append(s)

    def rust_sig_is_linked(rs: RustSig) -> bool:
        return rs.file.rsplit("/", 1)[-1] in linked_basenames

    # Surface orphan .rs files: present on disk under crates/zelda3/src/ but
    # not pulled into the build through any `mod` declaration. These can hide
    # whole batches of "ported" code that never actually compiles in.
    on_disk = {p.name for p in args.rust_root.glob("*.rs")}
    orphans = sorted(on_disk - linked_basenames)
    if orphans:
        print("WARNING: orphan .rs files (not linked from lib.rs / zelda_rtl.rs):",
              file=sys.stderr)
        for name in orphans:
            print(f"  - {name}", file=sys.stderr)
        print("  Functions in these files are dead code and are NOT counted as ported.\n",
              file=sys.stderr)

    # Status lookup for noise suppression.
    status_by_key: dict[tuple[str, str], str] = {}
    for row in load_status(args.status):
        if row.kind == "function":
            status_by_key[(row.source, row.symbol)] = row.status

    verdicts: list[tuple[str, str, str, CSig | None, RustSig | None, str]] = []
    counts: Counter[str] = Counter()

    def expected_rust_filename(c_file: str) -> str:
        return c_file[:-2] + ".rs"  # foo.c -> foo.rs

    for (c_file, c_name), c in sorted(c_sigs.items()):
        if args.module and c_file != args.module:
            continue
        status = status_by_key.get((c_file, c_name), "unknown")
        if not args.include_skipped and status in SKIP_STATUSES:
            counts["skipped"] += 1
            continue

        snake = camel_to_snake(c_name)
        minimal = snake + "_minimal"
        rust_file = expected_rust_filename(c_file)
        # Accept ports living in sibling per-prefix files like `foo_bar.rs` for
        # `foo.c` — this is how round-2+ parallel agents split sprite_main into
        # per-prefix files (sprite_main_blind.rs, sprite_main_guard.rs, etc.).
        rust_file_stem = rust_file[:-3]  # strip ".rs"
        sibling_prefix = rust_file_stem + "_"
        # Try the snake_case name first, then the verbatim PascalCase C name
        # (some early-port functions still use the C name).
        candidate_names = [snake, c_name]

        def find(name: str) -> RustSig | None:
            # Prefer a hit in the directly-mirroring .rs (if linked).
            if rust_file in linked_basenames:
                for s in rust_by_file.get(rust_file, []):
                    if s.name == name:
                        return s
            # Otherwise return the first hit anywhere in a linked file.
            # Hits in orphan/unlinked files are dead code and don't ship,
            # so they must not count as a successful port.
            for s in rust_by_name.get(name, []):
                if rust_sig_is_linked(s):
                    return s
            return None

        def in_sibling(rs: RustSig) -> bool:
            base = rs.file.rsplit("/", 1)[-1]
            return base.startswith(sibling_prefix) and base.endswith(".rs")

        exact = None
        for cand in candidate_names:
            exact = find(cand)
            if exact is not None:
                break
        partial = find(minimal) if exact is None else None

        if exact is not None:
            rust = exact
            if rust.file.endswith("/" + rust_file) or in_sibling(rust):
                verdict, detail = compare(c, rust)
            else:
                verdict, detail = ("wrong-file",
                                   f"expected {rust_file} (or sibling), found in {rust.file}")
        elif partial is not None:
            rust = partial
            loc = "same file" if rust.file.endswith("/" + rust_file) else f"in {rust.file}"
            verdict, detail = ("minimal", f"only `_minimal` partial port exists ({loc})")
        else:
            rust = None
            verdict = "missing-rust"
            detail = f"no fn `{snake}` anywhere under {args.rust_root.relative_to(REPO_ROOT)}"

        verdicts.append((verdict, c_file, c_name, c, rust, detail))
        counts[verdict] += 1

    if args.move_plan:
        emit_move_plan(verdicts)
        return

    if args.progress:
        emit_progress(verdicts)
        return

    # Print summary
    print("# Signature drift spike\n")
    total = sum(v for k, v in counts.items() if k != "skipped")
    print(f"scanned: {total} C functions (skipped {counts['skipped']} marked not-started/deferred/stub)\n")
    print("| verdict | count |")
    print("|---|---:|")
    for k in ["ok", "type-shape", "param-count", "wrong-file", "minimal", "missing-rust", "no-c-sig"]:
        if counts[k]:
            print(f"| `{k}` | {counts[k]} |")
    print()

    # Per-module rollup
    per_file: dict[str, Counter[str]] = {}
    for verdict, c_file, *_ in verdicts:
        if verdict == "skipped":
            continue
        per_file.setdefault(c_file, Counter())[verdict] += 1
    print("## Per-module rollup\n")
    print("| C file | ok | type-shape | param-count | wrong-file | minimal | missing | total |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|")
    for c_file in sorted(per_file):
        c = per_file[c_file]
        tot = sum(c.values())
        print(f"| `{c_file}` | {c['ok']} | {c['type-shape']} | {c['param-count']} | "
              f"{c['wrong-file']} | {c['minimal']} | {c['missing-rust']} | {tot} |")
    print()

    shown: Counter[str] = Counter()
    for verdict, c_file, c_name, c, rust, detail in verdicts:
        if verdict == "skipped":
            continue
        if args.only_drift and verdict == "ok":
            continue
        if args.limit and shown[verdict] >= args.limit:
            continue
        shown[verdict] += 1
        loc = f"{c_file}:{c_name}"
        rust_loc = f"{rust.file}:{rust.line}" if rust else "-"
        c_sig = f"{c.ret} {c.name}({', '.join(c.params) or 'void'})" if c else "?"
        r_sig = (
            f"fn {rust.name}({', '.join(rust.params) or ('&mut self' if rust.has_self else '')})"
            f"{(' -> ' + rust.ret) if rust.ret else ''}"
            if rust else "-"
        )
        print(f"## [{verdict}] {loc}")
        print(f"  c    {c_sig}")
        print(f"  rs   {r_sig}  ({rust_loc})")
        if detail:
            print(f"  why  {detail}")
        print()


if __name__ == "__main__":
    main()
