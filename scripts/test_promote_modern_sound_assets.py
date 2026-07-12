#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "promote_modern_sound_assets.py"
SPEC = importlib.util.spec_from_file_location("promote_modern_sound_assets", SCRIPT)
promote_modern_sound_assets = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = promote_modern_sound_assets
SPEC.loader.exec_module(promote_modern_sound_assets)


class PromoteModernSoundAssetsTests(unittest.TestCase):
    def test_cli_selects_primary_sequences_and_retains_alternates_per_variant(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            decoded = tmp_path / "decoded-sequences.json"
            harvest = tmp_path / "modern-sfx-harvest.json"
            out = tmp_path / "out"
            decoded.write_text(json.dumps(decoded_fixture()), encoding="utf-8")
            harvest.write_text(json.dumps(harvest_fixture()), encoding="utf-8")

            status = promote_modern_sound_assets.main(
                [
                    "--decoded-sequences-json",
                    str(decoded),
                    "--modern-sfx-harvest-json",
                    str(harvest),
                    "--output-dir",
                    str(out),
                ]
            )

            self.assertEqual(status, 0)
            manifest = json.loads((out / "modern-sound-assets.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["format"], "zelda3_modern_sound_assets_v1")
            self.assertFalse(manifest["runtime_dependency"])
            self.assertEqual(manifest["coverage"]["assets"], 3)
            self.assertEqual(manifest["coverage"]["primary_assets"], 3)
            self.assertEqual(manifest["coverage"]["review_ready_assets"], 2)
            self.assertEqual(manifest["coverage"]["needs_review_assets"], 1)
            self.assertEqual(manifest["coverage"]["blocked_assets"], 0)

            by_id = {asset["asset_id"]: asset for asset in manifest["assets"]}
            single = by_id["sfx_00_03"]
            self.assertEqual(single["promotion_status"], "review_ready")
            self.assertEqual(single["evidence"]["primary_sequence"]["technical_name"], "spc_seq_2100")
            self.assertEqual(len(single["evidence"]["alternate_sequences"]), 1)
            self.assertEqual(single["modern_program"]["context"]["source_slot"], 1)
            self.assertEqual(single["modern_program"]["context"]["voice_mask"], 0x80)

            variant_zero = by_id["sfx_01_2b_v00"]
            self.assertEqual(variant_zero["promotion_status"], "review_ready")
            self.assertEqual(variant_zero["modern_program"]["context"]["source_slot"], 2)

            variant = by_id["sfx_01_2b_v01"]
            self.assertEqual(variant["promotion_status"], "needs_review")
            self.assertEqual(variant["evidence"]["primary_sequence"]["technical_name"], "spc_seq_2130")
            self.assertIn("low confidence", variant["notes"][0])

            report = (out / "modern-sound-assets.md").read_text(encoding="utf-8")
            self.assertIn("Modern Sound Assets", report)
            self.assertIn("spc_seq_2100@0x2100", report)
            self.assertIn("sfx_01_2b_v01", report)


def decoded_fixture() -> dict:
    return {
        "format": "zelda3_decoded_audio_sequences_v1",
        "runtime_dependency": False,
        "banks": [
            {
                "asset_index": 0,
                "role": "intro_overworld",
                "sequences": [
                    {
                        "technical_name": "spc_seq_2100",
                        "address": 0x2100,
                        "status": "decoded",
                        "confidence": "high",
                        "decoder_kind": "sfx_compact",
                        "bytes_consumed": 8,
                        "event_counts": {"notes": 2, "controls": 2, "terminators": 1},
                    },
                    {
                        "technical_name": "spc_seq_2110",
                        "address": 0x2110,
                        "status": "decoded",
                        "confidence": "medium",
                        "decoder_kind": "sfx_compact",
                        "bytes_consumed": 16,
                        "event_counts": {"notes": 5, "controls": 4, "terminators": 0},
                    },
                    {
                        "technical_name": "spc_seq_2120",
                        "address": 0x2120,
                        "status": "decoded",
                        "confidence": "medium",
                        "decoder_kind": "sfx_compact",
                        "bytes_consumed": 6,
                        "event_counts": {"notes": 1, "controls": 2, "terminators": 0},
                    },
                    {
                        "technical_name": "spc_seq_2130",
                        "address": 0x2130,
                        "status": "decoded",
                        "confidence": "low",
                        "decoder_kind": "sfx_compact",
                        "bytes_consumed": 2,
                        "event_counts": {"notes": 1, "controls": 0, "terminators": 1},
                    },
                ],
            }
        ],
    }


def harvest_fixture() -> dict:
    return {
        "coverage": {"programs": 2},
        "programs": [
            {
                "bank": 0,
                "id": 0x03,
                "name": "trace_sfx_00_03",
                "status": "lifted",
                "source": "queue.input[1]",
                "steps": [step(7)],
                "sequence_provenance": [
                    provenance(40, 0, 0x03, 0x2110),
                    provenance(41, 0, 0x03, 0x2110),
                    provenance(42, 0, 0x03, 0x2100),
                ],
            },
            {
                "bank": 1,
                "id": 0x2B,
                "name": "trace_sfx_01_2b",
                "status": "lifted",
                "variant_count": 2,
                "variants": [
                    {
                        "name": "trace_sfx_01_2b_v00",
                        "context_signature": {
                            "source": "queue.input[2]",
                            "voice_mask": 0x80,
                            "context_voice_mask": 0,
                            "step_count": 1,
                        },
                        "steps": [step(7)],
                        "sequence_provenance": [provenance(50, 1, 0x2B, 0x2120)],
                    },
                    {
                        "name": "trace_sfx_01_2b_v01",
                        "context_signature": {
                            "source": "queue.input[2]",
                            "voice_mask": 0x40,
                            "context_voice_mask": 0,
                            "step_count": 1,
                        },
                        "steps": [step(6)],
                        "sequence_provenance": [provenance(60, 1, 0x2B, 0x2130)],
                    },
                ],
            },
        ],
    }


def step(voice: int) -> dict:
    return {
        "voice": voice,
        "pitch": 64,
        "instrument": 2,
        "waveform": "Pulse",
        "volume": 96,
        "envelope": {"attack": 1, "decay": 2, "sustain": 8, "release": 2},
        "duration_frames": 4,
        "pitch_slide": None,
        "evidence": {"command_frame": 1},
    }


def provenance(frame: int, bank: int, sound_id: int, address: int) -> dict:
    return {
        "frame": frame,
        "bank": bank,
        "id": sound_id,
        "rom_sequence_address": address,
        "sound_bank_asset_index": None,
        "source": "queue.input[1]",
        "source_kind": "spc_sfx_channel",
        "voice": 7,
    }


if __name__ == "__main__":
    unittest.main()
