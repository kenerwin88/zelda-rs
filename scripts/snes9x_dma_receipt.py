#!/usr/bin/env python3
"""Summarize one host frame from a narrow instrumented-Snes9x trace.

Checkpointed Snes9x states restore ``IPPU.TotalEmulatedFrames`` independently
of the route's absolute frame number.  The trace's ``run`` field is therefore
the authoritative mapping back to a comparator host frame:

    trace run = requested host frame - resumed host frame

This tool makes that mapping explicit and refuses to silently select an empty
or ambiguous run.  It reports the exact DMA operands and raster position plus
the decoded OBJ tile/cache provenance captured for a requested pixel.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Iterable


DMA_DOMAINS = {
    0x04: "OAM",
    0x18: "VRAM-low/mode",
    0x19: "VRAM-high",
    0x22: "CGRAM",
    0x80: "WRAM",
}


def hex_address(value: int) -> str:
    return f"${value >> 16:02x}:{value & 0xffff:04x}"


def pc_address(value: int) -> str:
    return hex_address(value)


def load_run(path: Path, run: int) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    with path.open() as stream:
        for line_number, line in enumerate(stream, 1):
            try:
                event = json.loads(line)
            except json.JSONDecodeError as error:
                raise SystemExit(f"{path}:{line_number}: invalid JSON: {error}") from error
            if event.get("run") == run:
                events.append(event)
    return events


def frame_boundaries(events: Iterable[dict[str, Any]]) -> list[dict[str, Any]]:
    return [event for event in events if event.get("event") == "frame"]


def summarize(path: Path, host_frame: int, resume_frame: int) -> str:
    run = host_frame - resume_frame
    if run < 0:
        raise SystemExit(
            f"host frame {host_frame} precedes resumed frame {resume_frame}"
        )
    events = load_run(path, run)
    if not events:
        raise SystemExit(
            f"no trace events for run {run}; do not use the trace's internal frame "
            "counter as a route frame selector"
        )

    boundaries = frame_boundaries(events)
    entries = [event for event in boundaries if event.get("stage") == "entry"]
    returns = [event for event in boundaries if event.get("stage") == "return"]
    if len(entries) != 1 or len(returns) != 1:
        raise SystemExit(
            f"run {run} is incomplete or ambiguous: entries={len(entries)} "
            f"returns={len(returns)}"
        )

    entry = entries[0]
    returned = returns[0]
    lines = [
        f"host frame {host_frame} = trace run {run} "
        f"(internal frames {entry.get('frame')}->{returned.get('frame')})",
        (
            "entry "
            f"raster={entry.get('v')}:{entry.get('cycles')} "
            f"pc={pc_address(int(entry.get('pc', 0)))} "
            f"module={int(entry.get('main', 0)):02x}/"
            f"{int(entry.get('sub', 0)):02x}/"
            f"{int(entry.get('subsub', 0)):02x} "
            f"frame_counter={int(entry.get('frame_counter', 0)):02x} "
            f"nmi_latch={entry.get('nmi_latch')}"
        ),
    ]

    nmis = [event for event in events if event.get("event") == "nmi"]
    lines.append(f"NMI entries: {len(nmis)}")
    for nmi in nmis:
        lines.append(
            "  "
            f"raster={nmi.get('v')}:{nmi.get('cycles')} "
            f"pc={pc_address(int(nmi.get('pc', 0)))} "
            f"nmi_latch={nmi.get('nmi_latch')} "
            f"nmi_disable={nmi.get('nmi_disable')} "
            f"nmi_pending={nmi.get('nmi_pending')}"
        )

    dmas = [event for event in events if event.get("event") == "dma"]
    lines.append(f"DMA transfers: {len(dmas)}")
    for index, dma in enumerate(dmas):
        b_address = int(dma.get("b_address", -1))
        domain = DMA_DOMAINS.get(b_address, f"PPU ${0x2100 + b_address:04x}")
        destination = ""
        if b_address in (0x18, 0x19):
            destination = f" dst_word=${int(dma.get('vram_address', 0)):04x}"
        lines.append(
            f"  {index:02d} {domain:<14} raster={dma.get('v')}:{dma.get('cycles')} "
            f"pc={pc_address(int(dma.get('pc', 0)))} "
            f"ch={dma.get('channel')} src={hex_address(int(dma.get('source', 0)))} "
            f"bytes={dma.get('bytes')} mode={dma.get('mode')}{destination}"
        )

    pixels = [event for event in events if event.get("event") == "pixel-write"]
    lines.append(f"captured pixel writes: {len(pixels)}")
    for pixel in pixels:
        lines.append(
            "  "
            f"palette={pixel.get('pix')} obj_tile=${int(pixel.get('obj_tile', 0)) & 0xffff:04x} "
            f"obj_line={pixel.get('obj_line')} origin=({pixel.get('obj_x')},"
            f"{pixel.get('obj_y')}) decoded_tile={pixel.get('obj_tile_number')} "
            f"cache_valid={pixel.get('obj_cache_valid')} "
            f"cache_hash={int(pixel.get('obj_cache_hash', 0)):08x} "
            f"pixel={pixel.get('obj_cache')}"
        )

    lines.append(
        "return "
        f"raster={returned.get('v')}:{returned.get('cycles')} "
        f"pc={pc_address(int(returned.get('pc', 0)))} "
        f"frame_counter={int(returned.get('frame_counter', 0)):02x} "
        f"nmi_latch={returned.get('nmi_latch')}"
    )
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", type=Path, help="instrumented oracle JSONL trace")
    parser.add_argument("--host-frame", type=int, required=True)
    parser.add_argument(
        "--resume-frame",
        type=int,
        default=0,
        help="absolute host frame represented by trace run 0 (default: 0)",
    )
    args = parser.parse_args()
    print(summarize(args.trace, args.host_frame, args.resume_frame))


if __name__ == "__main__":
    main()
