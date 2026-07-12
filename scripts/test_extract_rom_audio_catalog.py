#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parent / "extract_rom_audio_catalog.py"
SPEC = importlib.util.spec_from_file_location("extract_rom_audio_catalog", SCRIPT)
extract_rom_audio_catalog = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = extract_rom_audio_catalog
SPEC.loader.exec_module(extract_rom_audio_catalog)


class ExtractRomAudioCatalogTests(unittest.TestCase):
    def test_parses_song_bank_blocks_and_pointer_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            pack = tmp_path / "zelda3_assets.dat"
            write_asset_pack(
                pack,
                [
                    ("kSoundBank_intro", song_bank_payload(0x2000, pointer_table_payload())),
                    ("kSoundBank_indoor", song_bank_payload(0x3000, b"\x01\x02\x03\x04")),
                    ("kSoundBank_ending", song_bank_payload(0x4000, b"\x05\x06\x07\x08")),
                    ("kOther", b"ignored"),
                ],
            )

            args = extract_rom_audio_catalog.parse_args(["--asset-pack", str(pack)])
            catalog = extract_rom_audio_catalog.build_catalog(args)

            self.assertEqual(catalog["format"], "zelda3_modern_audio_source_catalog_v1")
            self.assertFalse(catalog["source"]["runtime_dependency"])
            self.assertEqual(catalog["asset_pack"]["asset_count"], 4)
            intro = catalog["sound_banks"][0]
            self.assertEqual(intro["asset_name"], "kSoundBank_intro")
            self.assertEqual(len(intro["blocks"]), 1)
            self.assertEqual(intro["blocks"][0]["target"], 0x2000)
            self.assertGreaterEqual(len(intro["pointer_tables"]), 1)
            self.assertTrue(
                any(candidate["address"] == 0x2100 for candidate in intro["candidate_sequences"])
            )

    def test_cli_writes_json_and_markdown_with_route_crosslink(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            pack = tmp_path / "zelda3_assets.dat"
            route = tmp_path / "harvest.json"
            out = tmp_path / "out"
            write_asset_pack(
                pack,
                [
                    ("kSoundBank_intro", song_bank_payload(0x2000, pointer_table_payload())),
                    ("kSoundBank_indoor", song_bank_payload(0x3000, b"\0" * 8)),
                    ("kSoundBank_ending", song_bank_payload(0x4000, b"\0" * 8)),
                ],
            )
            route.write_text(
                json.dumps(
                    {
                        "coverage": {"focused_commands": 2, "programs": 1, "lifted": 1, "gaps": 0},
                        "programs": [
                            {
                                "bank": 1,
                                "id": 0x2C,
                                "status": "lifted",
                                "occurrences": 2,
                                "variant_count": 2,
                                "first_frames": [10, 20],
                                "name": "trace_sfx_01_2c",
                                "sound_bank_asset_index": 0,
                                "rom_sequence_address": 0x2100,
                                "sequence_provenance": [
                                    {
                                        "frame": 12,
                                        "rom_sequence_address": 0x2140,
                                        "source_kind": "spc_sfx_channel",
                                    }
                                ],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            status = extract_rom_audio_catalog.main(
                [
                    "--asset-pack",
                    str(pack),
                    "--route-harvest-json",
                    str(route),
                    "--output-dir",
                    str(out),
                ]
            )

            self.assertEqual(status, 0)
            catalog = json.loads((out / "rom-audio-catalog.json").read_text(encoding="utf-8"))
            self.assertEqual(catalog["route_harvest"]["coverage"]["gaps"], 0)
            program = catalog["route_harvest"]["programs"][0]
            self.assertEqual(program["sound_bank_asset_index"], 0)
            self.assertEqual(program["rom_sequence_address"], 0x2100)
            self.assertEqual(program["sequence_provenance"][0]["rom_sequence_address"], 0x2140)
            self.assertTrue(
                any(
                    candidate["address"] == 0x2140
                    and candidate["confidence"] == "route_sequence_provenance_target"
                    for candidate in catalog["sound_banks"][0]["candidate_sequences"]
                )
            )
            report = (out / "rom-audio-catalog.md").read_text(encoding="utf-8")
            self.assertIn("Route Harvest Cross-Link", report)
            self.assertIn("0x2c", report)


def song_bank_payload(target: int, block: bytes) -> bytes:
    return (
        len(block).to_bytes(2, "little")
        + target.to_bytes(2, "little")
        + block
        + b"\0\0"
    )


def pointer_table_payload() -> bytes:
    payload = bytearray(0x180)
    pointers = [0x2100, 0x2110, 0x2120, 0x2130]
    for index, pointer in enumerate(pointers):
        payload[index * 2 : index * 2 + 2] = pointer.to_bytes(2, "little")
    for pointer in pointers:
        offset = pointer - 0x2000
        payload[offset : offset + 4] = bytes([pointer & 0xFF, pointer >> 8, 0xAA, 0xBB])
    return bytes(payload)


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
