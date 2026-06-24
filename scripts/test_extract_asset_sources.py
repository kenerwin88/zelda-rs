#!/usr/bin/env python3
"""Tests for readable asset source emission during extraction."""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import extract_assets
import tilemap_json


class ExtractAssetSourcesTests(unittest.TestCase):
    def test_writes_light_overworld_tilemap_as_json_source(self) -> None:
        payload = bytes(range(64)) * 64

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=67,
                name="kLightOverworldTilemap",
                payload=payload,
            )

            source_path = out_dir / "assets_src/tilemaps/light_overworld_tilemap.json"

            self.assertEqual(manifest["source_format"], "zelda3_byte_tilemap_v1")
            self.assertEqual(
                manifest["source_file"],
                "assets_src/tilemaps/light_overworld_tilemap.json",
            )
            self.assertFalse((out_dir / "assets/067-kLightOverworldTilemap.bin").exists())
            self.assertEqual(
                tilemap_json.bytes_from_tilemap(tilemap_json.read_tilemap_json(source_path)),
                payload,
            )

    def test_keeps_unmigrated_assets_as_bin_files(self) -> None:
        payload = b"abc"

        with TemporaryDirectory() as temp_dir:
            out_dir = Path(temp_dir)
            manifest = extract_assets.write_asset_output(
                out_dir,
                index=68,
                name="kDarkOverworldTilemap",
                payload=payload,
            )

            self.assertNotIn("source_format", manifest)
            self.assertEqual(manifest["file"], "assets/068-kDarkOverworldTilemap.bin")
            self.assertEqual(
                (out_dir / "assets/068-kDarkOverworldTilemap.bin").read_bytes(),
                payload,
            )


if __name__ == "__main__":
    unittest.main()
