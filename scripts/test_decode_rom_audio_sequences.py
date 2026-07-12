#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent


def load_script(name: str):
    script = SCRIPT_DIR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, script)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


extract_rom_audio_catalog = load_script("extract_rom_audio_catalog")
decode_rom_audio_sequences = load_script("decode_rom_audio_sequences")


class DecodeRomAudioSequencesTests(unittest.TestCase):
    def test_cli_decodes_streams_rejects_pointer_tables_and_links_route_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            pack = tmp_path / "zelda3_assets.dat"
            catalog_path = tmp_path / "rom-audio-catalog.json"
            out = tmp_path / "out"
            write_asset_pack(
                pack,
                [
                    ("kSoundBank_intro", song_bank_payload(0x2000, bank_payload())),
                    ("kSoundBank_indoor", song_bank_payload(0x3000, b"\0" * 8)),
                    ("kSoundBank_ending", song_bank_payload(0x4000, b"\0" * 8)),
                ],
            )
            catalog_path.write_text(json.dumps(catalog_fixture(pack)), encoding="utf-8")

            status = decode_rom_audio_sequences.main(
                [str(catalog_path), "--output-dir", str(out), "--max-bytes", "64"]
            )

            self.assertEqual(status, 0)
            decoded = json.loads((out / "decoded-sequences.json").read_text(encoding="utf-8"))
            self.assertEqual(decoded["format"], "zelda3_decoded_audio_sequences_v1")
            self.assertFalse(decoded["runtime_dependency"])
            self.assertEqual(decoded["coverage"]["candidate_sequences"], 3)
            self.assertEqual(decoded["coverage"]["decoded_sequences"], 2)
            self.assertEqual(decoded["coverage"]["rejected"], 1)
            self.assertEqual(decoded["coverage"]["direct_route_links"], 2)

            intro_sequences = decoded["banks"][0]["sequences"]
            by_address = {sequence["address"]: sequence for sequence in intro_sequences}
            self.assertEqual(by_address[0x2100]["status"], "decoded")
            self.assertEqual(by_address[0x2100]["confidence"], "high")
            self.assertEqual(by_address[0x2100]["decoder_kind"], "music_table")
            self.assertEqual(by_address[0x2120]["status"], "rejected")
            self.assertIn("pointer_table_like", by_address[0x2120]["reject_reasons"])
            self.assertEqual(by_address[0x2140]["status"], "decoded")
            self.assertEqual(by_address[0x2140]["decoder_kind"], "sfx_compact")
            self.assertTrue(by_address[0x2140]["pointer_like"]["is_pointer_like"])

            linked, route_linked = decoded["route_cross_links"]
            self.assertEqual(linked["status"], "linked")
            self.assertEqual(linked["matches"][0]["technical_name"], "spc_seq_2100")
            self.assertEqual(route_linked["status"], "linked")
            self.assertEqual(route_linked["matches"][0]["technical_name"], "spc_seq_2140")

            report = (out / "decoded-sequences.md").read_text(encoding="utf-8")
            self.assertIn("Decoded ROM Audio Sequences", report)
            self.assertIn("spc_seq_2100", report)
            self.assertIn("Route Harvest Links", report)


def catalog_fixture(asset_pack: Path) -> dict:
    return {
        "format": "zelda3_modern_audio_source_catalog_v1",
        "source": {
            "asset_pack": str(asset_pack),
            "rom": None,
            "runtime_dependency": False,
        },
        "sound_banks": [
            {
                "asset_index": 0,
                "asset_name": "kSoundBank_intro",
                "role": "intro_overworld",
                "status": "parsed",
                "written_ranges": [{"start": 0x2000, "end_exclusive": 0x2200, "size": 0x200}],
                "candidate_sequences": [
                    {
                        "technical_name": "spc_seq_2100",
                        "address": 0x2100,
                        "source_table": 0x2000,
                        "sha1_64": "decoded",
                    },
                    {
                        "technical_name": "spc_seq_2120",
                        "address": 0x2120,
                        "source_table": 0x2000,
                        "sha1_64": "pointer-like",
                    },
                    {
                        "technical_name": "spc_seq_2140",
                        "address": 0x2140,
                        "source_table": 0x2140,
                        "sha1_64": "route-sfx",
                        "confidence": "route_sequence_provenance_target",
                    },
                ],
            },
            {
                "asset_index": 1,
                "asset_name": "kSoundBank_indoor",
                "role": "indoor_dungeon",
                "status": "parsed",
                "written_ranges": [{"start": 0x3000, "end_exclusive": 0x3008, "size": 8}],
                "candidate_sequences": [],
            },
            {
                "asset_index": 2,
                "asset_name": "kSoundBank_ending",
                "role": "ending_credits",
                "status": "parsed",
                "written_ranges": [{"start": 0x4000, "end_exclusive": 0x4008, "size": 8}],
                "candidate_sequences": [],
            },
        ],
        "route_harvest": {
            "coverage": {"focused_commands": 2, "programs": 2, "lifted": 2, "gaps": 0},
            "programs": [
                {
                    "bank": 1,
                    "id": 0x2C,
                    "name": "trace_sfx_01_2c",
                    "variant_count": 2,
                    "sound_bank_asset_index": 0,
                    "rom_sequence_address": 0x2100,
                },
                {
                    "bank": 2,
                    "id": 0x0C,
                    "name": "trace_sfx_02_0c",
                    "variant_count": 3,
                    "sequence_provenance": [
                        {
                            "frame": 42,
                            "bank": 2,
                            "id": 0x0C,
                            "sound_bank_asset_index": 0,
                            "rom_sequence_address": 0x2140,
                            "voice": 3,
                            "source_kind": "spc_sfx_channel",
                        }
                    ],
                },
            ],
        },
    }


def bank_payload() -> bytes:
    payload = bytearray(0x200)
    pointer_targets = [0x2100, 0x2120, 0x2140, 0x2160]
    for index, pointer in enumerate(pointer_targets):
        payload[index * 2 : index * 2 + 2] = pointer.to_bytes(2, "little")

    sequence = bytes([0xE0, 0x01, 0x30, 0x31, 0x32, 0xED, 0x7F, 0x33, 0x00])
    payload[0x100 : 0x100 + len(sequence)] = sequence

    pointer_table = [0x2100, 0x2110, 0x2120, 0x2130, 0x2140, 0x2150]
    for index, pointer in enumerate(pointer_table):
        start = 0x120 + index * 2
        payload[start : start + 2] = pointer.to_bytes(2, "little")

    route_sfx = bytes([0xB0, 0x20, 0x10, 0x21, 0xB2, 0x21, 0xC0, 0x21, 0x00])
    payload[0x140 : 0x140 + len(route_sfx)] = route_sfx
    return bytes(payload)


def song_bank_payload(target: int, block: bytes) -> bytes:
    return (
        len(block).to_bytes(2, "little")
        + target.to_bytes(2, "little")
        + block
        + b"\0\0"
    )


def write_asset_pack(path: Path, assets: list[tuple[str, bytes]]) -> None:
    names = "\0".join(name for name, _payload in assets).encode("utf-8") + b"\0"
    data = bytearray(88 + len(assets) * 4 + len(names))
    data[:16] = extract_rom_audio_catalog.ASSET_SIGNATURE_PREFIX
    data[80:84] = len(assets).to_bytes(4, "little")
    data[84:88] = len(names).to_bytes(4, "little")
    sizes_start = 88
    for index, (_name, payload) in enumerate(assets):
        data[sizes_start + index * 4 : sizes_start + index * 4 + 4] = len(payload).to_bytes(
            4, "little"
        )
    key_start = sizes_start + len(assets) * 4
    data[key_start : key_start + len(names)] = names
    for _name, payload in assets:
        while len(data) % 4 != 0:
            data.append(0)
        data.extend(payload)
    path.write_bytes(bytes(data))


if __name__ == "__main__":
    unittest.main()
