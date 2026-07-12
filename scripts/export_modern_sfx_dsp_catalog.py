#!/usr/bin/env python3
"""Export unambiguous exact DSP parameters from reviewed SFX trace catalogs."""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path

from extract_modern_sfx_catalog import stable_variant_hash, step_signature


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalogs", nargs="+", type=Path)
    parser.add_argument("--tsv-out", required=True, type=Path)
    args = parser.parse_args()

    variants: dict[tuple[int, int, int], set[tuple[tuple[int, ...], ...]]] = defaultdict(set)
    for path in args.catalogs:
        catalog = json.loads(path.read_text(encoding="utf-8"))
        for program in catalog["programs"]:
            for variant in program.get("variants") or [program]:
                steps = tuple(
                    (
                        int(step["voice"]),
                        int(step["pitch"]),
                        int(step["instrument"]),
                        int(step["volume"]),
                        int(step["pan"]),
                        int(step["duration_frames"]),
                        int(bool(step["echo"])),
                        int(step["command_delay_frames"]),
                        int(step["scheduler_tick_index"]),
                        int(step["dsp_pitch"]),
                        int(step["volume_left"]),
                        int(step["volume_right"]),
                        int(step["dsp_adsr1"]),
                        int(step["dsp_adsr2"]),
                        int(step["dsp_gain"]),
                        # Key-on offset is live scheduler phase, not an SFX
                        # program invariant. Runtime scheduling must derive it.
                        0,
                    )
                    for step in variant.get("steps", [])
                )
                if steps:
                    variant_hash = variant.get("variant_hash")
                    if variant_hash is None:
                        signature = json.dumps(
                            [step_signature(step) for step in variant.get("steps", [])],
                            sort_keys=True,
                            separators=(",", ":"),
                        )
                        variant_hash = stable_variant_hash(signature)
                    key = (
                        int(program["bank"]),
                        int(program["id"]),
                        int(variant_hash),
                    )
                    variants[key].add(steps)

    lines = [
        "# bank\tid\tvariant_hash\tstep\tvoice\tpitch\tinstrument\tvolume\tpan\t"
        "duration_frames\techo\tcommand_delay_frames\tscheduler_tick_index\t"
        "dsp_pitch\tvolume_left\tvolume_right\t"
        "adsr1\tadsr2\tgain\tsample_offset"
    ]
    ambiguous = 0
    for (bank, sfx_id, variant_hash), candidates in sorted(variants.items()):
        if len(candidates) != 1:
            ambiguous += 1
            continue
        for step_index, values in enumerate(next(iter(candidates))):
            lines.append(
                "\t".join(
                    [
                        str(bank),
                        f"{sfx_id:02x}",
                        f"{variant_hash:08x}",
                        str(step_index),
                        *(str(value) for value in values),
                    ]
                )
            )

    args.tsv_out.parent.mkdir(parents=True, exist_ok=True)
    args.tsv_out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(
        f"modern SFX DSP catalog: variants={len(variants) - ambiguous} "
        f"ambiguous_skipped={ambiguous} steps={len(lines) - 1}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
