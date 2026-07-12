#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "build_modern_sound_index.py"
SPEC = importlib.util.spec_from_file_location("build_modern_sound_index", SCRIPT)
build_modern_sound_index = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = build_modern_sound_index
SPEC.loader.exec_module(build_modern_sound_index)


class BuildModernSoundIndexTests(unittest.TestCase):
    def test_cli_links_harvested_route_program_to_decoded_sequence(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            decoded = tmp_path / "decoded-sequences.json"
            harvest = tmp_path / "modern-sfx-harvest.json"
            out = tmp_path / "out"
            decoded.write_text(json.dumps(decoded_fixture()), encoding="utf-8")
            harvest.write_text(json.dumps(harvest_fixture()), encoding="utf-8")

            status = build_modern_sound_index.main(
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
            index = json.loads((out / "modern-sound-index.json").read_text(encoding="utf-8"))
            self.assertEqual(index["format"], "zelda3_modern_sound_index_v1")
            self.assertFalse(index["runtime_dependency"])
            self.assertEqual(index["coverage"]["linked_programs"], 1)
            self.assertEqual(index["coverage"]["unlinked_programs"], 1)
            linked, unlinked = index["sounds"]
            self.assertEqual(linked["status"], "linked")
            self.assertEqual(
                linked["sequence_links"][0]["matches"][0]["technical_name"],
                "spc_seq_2100",
            )
            self.assertEqual(unlinked["status"], "unlinked")

            report = (out / "modern-sound-index.md").read_text(encoding="utf-8")
            self.assertIn("Modern Sound Index", report)
            self.assertIn("spc_seq_2100@0x2100", report)
            self.assertIn("no provenance", report)


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
                        "event_counts": {"notes": 4, "controls": 2},
                        "sha1_64": "hash",
                    }
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
                "id": 0x34,
                "name": "trace_sfx_00_34",
                "status": "lifted",
                "variant_count": 1,
                "first_frames": [10],
                "sequence_provenance": [
                    {
                        "frame": 12,
                        "bank": 0,
                        "id": 0x34,
                        "rom_sequence_address": 0x2100,
                        "voice": 1,
                        "source_kind": "spc_sfx_channel",
                    }
                ],
            },
            {
                "bank": 1,
                "id": 0x2C,
                "name": "trace_sfx_01_2c",
                "status": "lifted",
                "variant_count": 1,
                "first_frames": [20],
                "sequence_provenance": [],
            },
        ],
    }


if __name__ == "__main__":
    unittest.main()
