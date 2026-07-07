#!/usr/bin/env python3
"""Tests for semantic dialogue catalog extraction."""

from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import dialogue_catalog


def pack_arrays(arrays: list[bytes]) -> bytes:
    use_wide_offsets = sum(len(item) for item in arrays[:-1]) >= 65536
    offset_size = 4 if use_wide_offsets else 2
    offsets = []
    offset = 0
    for item in arrays[:-1]:
        offset += len(item)
        offsets.append(offset.to_bytes(offset_size, "little"))
    marker = 8192 + len(arrays) - 1 if use_wide_offsets else len(arrays) - 1
    return b"".join([*offsets, *arrays, marker.to_bytes(2, "little")])


class DialogueCatalogTests(unittest.TestCase):
    def test_normalizes_trailing_padding_before_memblk_lookup(self) -> None:
        data = pack_arrays([b"dict", b"messages"]) + b"\0\0"

        normalized = dialogue_catalog.normalize_memblk(data, min_items=2)

        self.assertEqual(dialogue_catalog.find_index_in_memblk(normalized, 0), b"dict")
        self.assertEqual(dialogue_catalog.find_index_in_memblk(normalized, 1), b"messages")

    def test_expands_dictionary_and_parses_us_commands(self) -> None:
        dictionary = pack_arrays([bytes([0, 1])])
        raw_message = bytes(
            [
                0x88,
                2,
                0x78,
                3,
                0x7F,
            ]
        )

        entry = dialogue_catalog.message_catalog_entry(
            0,
            raw_message,
            dictionary,
            flags=0,
        )

        self.assertEqual(entry["expanded_bytes"], ["00", "01", "02", "78", "03", "7f"])
        self.assertEqual(entry["preview_text"], "ABC[wait 03]")
        self.assertEqual(
            [op["op"] for op in entry["ops"]],
            ["glyph", "glyph", "glyph", "wait", "end_message"],
        )
        self.assertEqual(entry["ops"][3]["param"], 3)
        self.assertEqual(entry["dictionary_expansions"][0]["dictionary_index"], 0)
        self.assertEqual(entry["source_text"], "ABC[wait 03][end_message]")

    def test_compiles_source_text_to_message_bytecode(self) -> None:
        bytecode = dialogue_catalog.compile_source_text(
            "Hi[...][line1][color 02][A][end_message]"
        )

        self.assertEqual(
            bytecode,
            bytes([7, 34, 67, 0x74, 0x77, 2, 91, 0x7F]),
        )

    def test_dialogue_source_compiles_to_uncompressed_asset(self) -> None:
        source = {
            "format": dialogue_catalog.FORMAT_DIALOGUE_SOURCE,
            "messages": [
                {"id": 0, "source_text": "AB[end_message]"},
                {"id": 1, "source_text": "C"},
            ],
        }

        asset = dialogue_catalog.asset_from_dialogue_source(source)
        dictionary, dialogue = dialogue_catalog.dialogue_blocks(asset)
        messages = dialogue_catalog.dialogue_messages(dialogue)

        self.assertEqual(dialogue_catalog.dictionary_entries(dictionary), [b""])
        self.assertEqual(messages, [bytes([0, 1, 0x7F]), bytes([2])])

    def test_dialogue_source_generation_requires_exact_source_roundtrip(self) -> None:
        catalog = {
            "language": {
                "language": "us",
                "dialogue_pack": 0,
                "font_pack": 0,
                "flags": 0,
                "raw_config": ["00", "00", "00"],
            },
            "messages": [
                {
                    "id": 0,
                    "expanded_bytes": ["00", "7f"],
                    "source_text": "B[end_message]",
                }
            ],
        }

        with self.assertRaisesRegex(ValueError, "message 0 source_text"):
            dialogue_catalog.dialogue_source_from_catalog(catalog)

    def test_catalog_from_assets_uses_language_map_and_message_table(self) -> None:
        dictionary = pack_arrays([bytes([0, 1])])
        messages = pack_arrays(
            [
                bytes([0x88, 0x7F]),
                bytes([52, 53, 0x7F]),
            ]
        )
        dialogue_asset = pack_arrays([dictionary, messages]) + b"\0\0"
        language_asset = pack_arrays([b"us", bytes([0, 0, 0])]) + b"\0\0"

        catalog = dialogue_catalog.catalog_from_assets(dialogue_asset, language_asset)

        self.assertEqual(catalog["format"], dialogue_catalog.FORMAT_DIALOGUE_CATALOG)
        self.assertEqual(catalog["language"]["language"], "us")
        self.assertEqual(catalog["message_count"], 2)
        self.assertEqual(catalog["messages"][0]["preview_text"], "AB")
        self.assertEqual(catalog["messages"][1]["preview_text"], "01")

    def test_writes_dialogue_catalog_for_asset_dir(self) -> None:
        dictionary = pack_arrays([bytes([0])])
        messages = pack_arrays([bytes([0x88, 0x7F])])
        dialogue_asset = pack_arrays([dictionary, messages]) + b"\0\0"
        language_asset = pack_arrays([b"us", bytes([0, 0, 0])]) + b"\0\0"

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets = asset_dir / "assets"
            assets.mkdir()
            (assets / "094-kDialogue.bin").write_bytes(dialogue_asset)
            (assets / "096-kDialogueMap.bin").write_bytes(language_asset)

            path = dialogue_catalog.write_dialogue_catalog_for_asset_dir(asset_dir)

            self.assertEqual(
                path,
                asset_dir / "assets_src/dialogue/dialogue_catalog.json",
            )
            payload = json.loads(path.read_text())
            self.assertEqual(payload["message_count"], 1)
            self.assertEqual(payload["messages"][0]["preview_text"], "A")

    def test_writes_dialogue_source_for_asset_dir(self) -> None:
        dictionary = pack_arrays([bytes([0])])
        messages = pack_arrays([bytes([0x88, 0x7F])])
        dialogue_asset = pack_arrays([dictionary, messages]) + b"\0\0"
        language_asset = pack_arrays([b"us", bytes([0, 0, 0])]) + b"\0\0"

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir)
            assets = asset_dir / "assets"
            assets.mkdir()
            (assets / "094-kDialogue.bin").write_bytes(dialogue_asset)
            (assets / "096-kDialogueMap.bin").write_bytes(language_asset)

            paths = dialogue_catalog.write_dialogue_sources_for_asset_dir(asset_dir)

            self.assertEqual(
                paths,
                [
                    asset_dir / "assets_src/dialogue/dialogue_catalog.json",
                    asset_dir / "assets_src/dialogue/dialogue_source.json",
                ],
            )
            source = json.loads(paths[1].read_text())
            self.assertEqual(source["format"], dialogue_catalog.FORMAT_DIALOGUE_SOURCE)
            self.assertEqual(source["messages"][0]["source_text"], "A[end_message]")


if __name__ == "__main__":
    unittest.main()
