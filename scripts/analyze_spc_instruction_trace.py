#!/usr/bin/env python3
"""Align Rust/oracle SPC instruction traces and expose the first clock drift.

The trace core and Rust recorder do not necessarily begin on the same SPC
instruction at a video-frame boundary.  Comparing list indexes therefore
produces noisy false leads.  This tool aligns complete CPU-state tokens first,
then reports contiguous regions where the timer-0 phase relationship changes.
Timer 0 is clocked from the same APU master cycle as the DSP, so its stage-1
phase is a cheap, stable witness for instruction-clock drift.
"""

from __future__ import annotations

import argparse
import difflib
import json
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


TIMER_PERIOD = 128


@dataclass(frozen=True)
class AlignedInstruction:
    rust_index: int
    oracle_index: int
    pc: int
    opcode: int
    timer_phase_delta: int
    rust_timer_divider: int
    oracle_timer_divider: int


def instruction_token(instruction: dict, *, rust: bool) -> tuple[int, ...]:
    """Return enough CPU state to disambiguate the driver's tight loops."""
    return (
        int(instruction["pc" if rust else "program_counter"]),
        int(instruction["opcode"]),
        int(instruction["a"]),
        int(instruction["x"]),
        int(instruction["y"]),
        int(instruction["sp" if rust else "stack_pointer"]),
    )


def direct_page_witness(instruction: dict, *, rust: bool) -> list[int] | None:
    if rust:
        low = instruction.get("direct_page_0_3")
        high = instruction.get("direct_page_8_11")
        if low is not None and high is not None:
            return [*map(int, low), *map(int, high)]
        return None
    values = instruction.get("direct_page_0_11")
    return None if values is None else [*map(int, values)]


def first_instruction_divergence(receipt: dict) -> dict | None:
    """Locate the first unmatched CPU-state region, not merely clock drift."""
    rust = receipt["rust_spc_instruction_trace"][1]
    oracle = receipt["oracle_smp_instructions"]
    rust_tokens = [instruction_token(item, rust=True) for item in rust]
    oracle_tokens = [instruction_token(item, rust=False) for item in oracle]
    matcher = difflib.SequenceMatcher(None, rust_tokens, oracle_tokens, autojunk=False)
    saw_aligned_region = False
    for tag, rust_start, rust_end, oracle_start, oracle_end in matcher.get_opcodes():
        if tag == "equal":
            saw_aligned_region = True
            continue
        # Different retro_run boundaries routinely contribute a short leading
        # tail/head. It is not a causal control-flow divergence.
        if not saw_aligned_region:
            continue
        rust_item = rust[rust_start] if rust_start < len(rust) else None
        oracle_item = oracle[oracle_start] if oracle_start < len(oracle) else None
        return {
            "tag": tag,
            "rust_range": (rust_start, rust_end),
            "oracle_range": (oracle_start, oracle_end),
            "rust_token": None if rust_item is None else rust_tokens[rust_start],
            "oracle_token": None if oracle_item is None else oracle_tokens[oracle_start],
            "rust_direct_page": None
            if rust_item is None
            else direct_page_witness(rust_item, rust=True),
            "oracle_direct_page": None
            if oracle_item is None
            else direct_page_witness(oracle_item, rust=False),
        }
    return None


def dsp_content_tokens(receipt: dict, *, rust: bool) -> list[tuple[int, int]]:
    if rust:
        return [
            (int(event["parity_dsp"]["addr"]), int(event["parity_dsp"]["value"]))
            for event in receipt["rust_audio_event_frame"]["events"]
            if event.get("parity_dsp") is not None
        ]
    return [
        (int(write["register"]), int(write["value"]))
        for write in receipt["oracle_dsp_register_writes"]
    ]


def first_dsp_content_divergence(receipts: list[dict]) -> dict | None:
    rust: list[tuple[int, int]] = []
    oracle: list[tuple[int, int]] = []
    for receipt in receipts:
        rust.extend(dsp_content_tokens(receipt, rust=True))
        oracle.extend(dsp_content_tokens(receipt, rust=False))
    matcher = difflib.SequenceMatcher(None, rust, oracle, autojunk=False)
    for tag, rust_start, rust_end, oracle_start, oracle_end in matcher.get_opcodes():
        if tag != "equal":
            return {
                "tag": tag,
                "rust_range": (rust_start, rust_end),
                "oracle_range": (oracle_start, oracle_end),
                "rust": rust[rust_start:rust_end],
                "oracle": oracle[oracle_start:oracle_end],
            }
    return None


def align_instructions(receipt: dict) -> list[AlignedInstruction]:
    rust = receipt["rust_spc_instruction_trace"][1]
    oracle = receipt["oracle_smp_instructions"]
    rust_tokens = [instruction_token(item, rust=True) for item in rust]
    oracle_tokens = [instruction_token(item, rust=False) for item in oracle]
    matcher = difflib.SequenceMatcher(
        None, rust_tokens, oracle_tokens, autojunk=False
    )
    aligned: list[AlignedInstruction] = []
    for rust_start, oracle_start, length in matcher.get_matching_blocks():
        for offset in range(length):
            rust_index = rust_start + offset
            oracle_index = oracle_start + offset
            rust_instruction = rust[rust_index]
            oracle_instruction = oracle[oracle_index]
            # Rust stores the remaining cycles until timer-0 stage 1 wraps;
            # Snes9x stores the elapsed stage-1 phase. Equal clocks sum to 128.
            phase_delta = (
                TIMER_PERIOD
                - int(rust_instruction["timer0_cycles"])
                - int(oracle_instruction["timer0_stage1"])
            ) % TIMER_PERIOD
            aligned.append(
                AlignedInstruction(
                    rust_index=rust_index,
                    oracle_index=oracle_index,
                    pc=rust_tokens[rust_index][0],
                    opcode=rust_tokens[rust_index][1],
                    timer_phase_delta=phase_delta,
                    rust_timer_divider=int(rust_instruction["timer0_divider"]),
                    oracle_timer_divider=int(oracle_instruction["timer0_stage2"]),
                )
            )
    return aligned


def phase_runs(aligned: list[AlignedInstruction]) -> list[tuple[int, int, int]]:
    """Return (start, end-exclusive, delta) runs in aligned instruction order."""
    if not aligned:
        return []
    runs: list[tuple[int, int, int]] = []
    start = 0
    phase = aligned[0].timer_phase_delta
    for index, instruction in enumerate(aligned[1:], 1):
        if instruction.timer_phase_delta != phase:
            runs.append((start, index, phase))
            start = index
            phase = instruction.timer_phase_delta
    runs.append((start, len(aligned), phase))
    return runs


def summarize(receipt: dict) -> dict:
    aligned = align_instructions(receipt)
    rust_count = len(receipt["rust_spc_instruction_trace"][1])
    oracle_count = len(receipt["oracle_smp_instructions"])
    phases = Counter(item.timer_phase_delta for item in aligned)
    divider_mismatches = sum(
        item.rust_timer_divider != item.oracle_timer_divider for item in aligned
    )
    return {
        "rust_count": rust_count,
        "oracle_count": oracle_count,
        "aligned_count": len(aligned),
        "coverage": len(aligned) / max(rust_count, oracle_count, 1),
        "phase_histogram": dict(sorted(phases.items())),
        "divider_mismatches": divider_mismatches,
        "aligned": aligned,
        "phase_runs": phase_runs(aligned),
        "first_instruction_divergence": first_instruction_divergence(receipt),
    }


def print_summary(path: Path, summary: dict, run_limit: int) -> None:
    print(
        f"{path}: aligned={summary['aligned_count']}/"
        f"{summary['rust_count']},{summary['oracle_count']} "
        f"coverage={summary['coverage']:.1%} "
        f"phase_histogram={summary['phase_histogram']} "
        f"divider_mismatches={summary['divider_mismatches']}"
    )
    aligned = summary["aligned"]
    runs = summary["phase_runs"]
    for start, end, phase in runs[:run_limit]:
        first = aligned[start]
        last = aligned[end - 1]
        print(
            f"  phase={phase:+d} matched[{start}:{end}] count={end - start} "
            f"rust[{first.rust_index}:{last.rust_index + 1}] "
            f"oracle[{first.oracle_index}:{last.oracle_index + 1}] "
            f"pc=${first.pc:04x}..${last.pc:04x}"
        )
    if len(runs) > run_limit:
        print(f"  ... {len(runs) - run_limit} additional phase runs")
    divergence = summary["first_instruction_divergence"]
    if divergence is not None:
        print(
            "  first_state_divergence "
            f"{divergence['tag']} rust{divergence['rust_range']} "
            f"oracle{divergence['oracle_range']} "
            f"rust_token={divergence['rust_token']} "
            f"oracle_token={divergence['oracle_token']}"
        )
        if divergence["rust_direct_page"] is not None or divergence["oracle_direct_page"] is not None:
            print(
                "    direct_page "
                f"rust={divergence['rust_direct_page']} "
                f"oracle={divergence['oracle_direct_page']}"
            )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", nargs="+", type=Path)
    parser.add_argument("--run-limit", type=int, default=12)
    args = parser.parse_args()
    receipts = []
    for path in args.trace:
        with path.open() as stream:
            receipt = json.load(stream)
        receipts.append(receipt)
        print_summary(path, summarize(receipt), args.run_limit)
    dsp_divergence = first_dsp_content_divergence(receipts)
    if dsp_divergence is not None:
        print(
            "first_dsp_content_divergence "
            f"{dsp_divergence['tag']} rust{dsp_divergence['rust_range']} "
            f"oracle{dsp_divergence['oracle_range']} "
            f"rust={dsp_divergence['rust']} oracle={dsp_divergence['oracle']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
