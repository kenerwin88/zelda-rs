#!/usr/bin/env python3
"""Turn an instrumented Snes9x run into a ROM-symbolized boot contract.

The Snes9x trace is behavioral evidence from the original ROM.  This script
does not copy its PPU state into zelda3-rs.  It records the sequence of ROM
work-buffer writes and hardware transfers, then attaches the checked-in ROM
decompilation symbols to every WRAM address it can identify.

Expected trace records (emitted by the exact Snes9x 1.63 trace core):

  <frame> state-before <main> <sub> <nmi> <inidisp> <vram> <cgram> <hud> <nmi-op>
  <frame> dma <channel> <bank>:<source> <b-address> <bytes> <mode> <fixed> <decrement> <vma>
  <frame> dma_pc <pc>
  <frame> wram <address> <value> <pc>
  <frame> oam <address> <value> dma=<0|1> channel=<n>
  <frame> 2118|2119 <vma> <value> <address> <vma-flags> <dma> <channel>
  <frame> state-after ...

The output is deterministic JSON.  It is intended to be checked into a route
receipt and compared before rendered RGBA pixels.  Unknown symbols remain
unknown evidence; this tool never invents a source value or a fallback.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any


STATE_RE = re.compile(r"^(?P<frame>\d+) state-(?P<stage>before|after) (?P<values>(?:[0-9a-fA-F]{2} ?){8})$")
DMA_RE = re.compile(
    r"^(?P<frame>\d+) dma (?P<channel>\d+) (?P<bank>[0-9a-fA-F]{2}):(?P<source>[0-9a-fA-F]{4}) "
    r"(?P<port>[0-9a-fA-F]{2}) (?P<count>\d+) (?P<mode>\d+) (?P<fixed>\d+) (?P<decrement>\d+) (?P<vma>\d+)$"
)
DMA_PC_RE = re.compile(r"^(?P<frame>\d+) dma_pc (?P<pc>[0-9a-fA-F]{6})$")
WRAM_RE = re.compile(r"^(?P<frame>\d+) wram (?P<address>[0-9a-fA-F]{4}) (?P<value>[0-9a-fA-F]{2}) (?P<pc>[0-9a-fA-F]{6})$")
OAM_RE = re.compile(r"^(?P<frame>\d+) oam (?P<address>[0-9a-fA-F]{4}) (?P<value>[0-9a-fA-F]{2}) dma=(?P<dma>\d+) channel=(?P<channel>\d+)$")
VRAM_RE = re.compile(
    r"^(?P<frame>\d+) (?P<port>2118|2119) (?P<vma>[0-9a-fA-F]{4}) (?P<value>[0-9a-fA-F]{2}) "
    r"(?P<address>[0-9a-fA-F]{4}) (?P<flags>[0-9a-fA-F]{2}) (?P<dma>\d+) (?P<channel>\d+)$"
)

STATE_FIELDS = ("main_module", "submodule", "nmi_latch", "inidisp", "bg_vram_load", "cgram_upload", "hud_upload", "nmi_subroutine")


def parse_int(value: str) -> int:
    return int(value, 16)


def load_symbols(path: Path) -> dict[int, list[dict[str, Any]]]:
    by_address: dict[int, list[dict[str, Any]]] = defaultdict(list)
    for item in json.loads(path.read_text()):
        by_address[int(item["address"])].append(
            {
                "rust": item["rust_name"],
                "source": item["source_label"],
                "source_path": item["source_path"],
                "source_line": item["source_line"],
                "subsystem": item["subsystem"],
            }
        )
    return dict(by_address)


def symbolize(address: int, symbols: dict[int, list[dict[str, Any]]]) -> list[dict[str, Any]]:
    """Use exact RAM labels only; proximity would be an unsupported inference."""
    return symbols.get(address, [])


def event_sort_key(event: dict[str, Any]) -> tuple[int, int]:
    order = {"state": 0, "wram_write": 1, "dma": 2, "oam_write": 3, "vram_write": 4}
    return event["line"], order[event["kind"]]


def parse_trace(trace: Path, symbols: dict[int, list[dict[str, Any]]], frame_limit: int | None) -> dict[str, Any]:
    frames: dict[int, list[dict[str, Any]]] = defaultdict(list)
    last_dma_by_frame: dict[int, dict[str, Any]] = {}
    ignored = 0
    # The first ad-hoc trace core emitted VRAM-delta records with a literal
    # ``\\n`` suffix.  Normalize that producer defect here so a following
    # state boundary cannot be swallowed into the delta line.  This preserves
    # the event text; it does not synthesize an event or a state value.
    normalized = trace.read_text().replace(r"\n", "\n")
    for line_number, raw in enumerate(normalized.splitlines(), start=1):
        text = raw.strip()
        match = STATE_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame_limit is not None and frame > frame_limit:
                continue
            values = [parse_int(value) for value in match["values"].split()]
            frames[frame].append({"line": line_number, "kind": "state", "stage": match["stage"], "values": dict(zip(STATE_FIELDS, values, strict=True))})
            continue
        match = DMA_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame_limit is not None and frame > frame_limit:
                continue
            event = {"line": line_number, "kind": "dma", "channel": int(match["channel"]), "source": f"{match['bank'].lower()}:{match['source'].lower()}", "port": f"21{match['port'].lower()}", "bytes": int(match["count"]), "mode": int(match["mode"]), "fixed": bool(int(match["fixed"])), "decrement": bool(int(match["decrement"])), "vma": int(match["vma"]), "pc": None}
            frames[frame].append(event)
            last_dma_by_frame[frame] = event
            continue
        match = DMA_PC_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame in last_dma_by_frame:
                last_dma_by_frame[frame]["pc"] = f"{match['pc'].lower()}"
            continue
        match = WRAM_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame_limit is not None and frame > frame_limit:
                continue
            address = parse_int(match["address"])
            frames[frame].append({"line": line_number, "kind": "wram_write", "address": f"{address:04x}", "value": f"{parse_int(match['value']):02x}", "pc": match["pc"].lower(), "symbols": symbolize(address, symbols)})
            continue
        match = OAM_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame_limit is not None and frame > frame_limit:
                continue
            frames[frame].append({"line": line_number, "kind": "oam_write", "address": match["address"].lower(), "value": match["value"].lower(), "dma": bool(int(match["dma"])), "channel": int(match["channel"])})
            continue
        match = VRAM_RE.match(text)
        if match:
            frame = int(match["frame"])
            if frame_limit is not None and frame > frame_limit:
                continue
            frames[frame].append({"line": line_number, "kind": "vram_write", "port": match["port"], "vma": match["vma"].lower(), "address": match["address"].lower(), "value": match["value"].lower(), "flags": match["flags"].lower(), "dma": bool(int(match["dma"])), "channel": int(match["channel"])})
            continue
        if text:
            ignored += 1
    return {"schema": "zelda3.snes9x-boot-contract/v1", "trace": str(trace), "frames": [{"frame": frame, "events": sorted(events, key=event_sort_key)} for frame, events in sorted(frames.items())], "ignored_trace_lines": ignored}


def validate_contract(contract: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for boundary in contract["frames"]:
        events = boundary["events"]
        stages = [event.get("stage") for event in events if event["kind"] == "state"]
        if stages != ["before", "after"]:
            errors.append(f"frame {boundary['frame']}: expected state-before then state-after, saw {stages}")
        for event in events:
            if event["kind"] == "dma" and event["pc"] is None:
                errors.append(f"frame {boundary['frame']}: DMA lacks its ROM program counter")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="ZELDA3_SNES9X_VRAM_TRACE output")
    parser.add_argument("--symbols", type=Path, default=Path("docs/nes-ver2/ram_symbol_crosswalk.json"))
    parser.add_argument("--frames", type=int, default=None, help="include frames through this frame")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--validate", action="store_true", help="fail if the trace cannot establish complete frame boundaries")
    args = parser.parse_args()

    contract = parse_trace(args.trace, load_symbols(args.symbols), args.frames)
    args.output.write_text(json.dumps(contract, indent=2, sort_keys=True) + "\n")
    if args.validate:
        errors = validate_contract(contract)
        if errors:
            for error in errors:
                print(error)
            return 1
    print(f"wrote {args.output}: frames={len(contract['frames'])} ignored_trace_lines={contract['ignored_trace_lines']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
