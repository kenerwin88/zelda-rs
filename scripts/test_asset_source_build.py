#!/usr/bin/env python3
"""Build integration test for readable asset sources."""

from __future__ import annotations

import os
import json
import re
import shutil
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import dialogue_catalog


REPO = Path(__file__).resolve().parents[1]
GENERATED_ASSETS = REPO / "generated/zelda3_assets"


def built_asset_pack() -> Path:
    candidates = sorted(
        REPO.glob("target/debug/build/zelda3-bin-*/out/zelda3_assets.dat"),
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    if not candidates:
        raise AssertionError("cargo build did not produce zelda3_assets.dat")
    return candidates[0]


def split_built_asset_pack(asset_pack: Path) -> list[tuple[str, bytes]]:
    data = asset_pack.read_bytes()
    if len(data) < 88 or data[:16] != b"Zelda3_v0     \n\0":
        raise AssertionError(f"{asset_pack} is not a zelda3 asset pack")
    count = int.from_bytes(data[80:84], "little")
    key_signature_len = int.from_bytes(data[84:88], "little")
    sizes_start = 88
    key_signature_start = sizes_start + count * 4
    payload_offset = key_signature_start + key_signature_len
    names = (
        data[key_signature_start:payload_offset]
        .rstrip(b"\0")
        .decode("utf8")
        .split("\0")
    )
    assets = []
    offset = payload_offset
    for index, name in enumerate(names):
        size = int.from_bytes(data[sizes_start + index * 4 : sizes_start + index * 4 + 4], "little")
        offset = (offset + 3) & ~3
        end = offset + size
        assets.append((name, data[offset:end]))
        offset = end
    return assets


class AssetSourceBuildTests(unittest.TestCase):
    def test_build_packs_spc_driver_as_timing_only_sidecar(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        result = subprocess.run(
            ["cargo", "build", "-p", "zelda3-bin"],
            cwd=REPO,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.assertEqual(result.returncode, 0, result.stdout)

        assets = split_built_asset_pack(built_asset_pack())
        sidecars = [
            payload for name, payload in assets if name == "kSpcDriverTimingProgram"
        ]
        self.assertEqual(len(sidecars), 1)
        self.assertEqual(len(sidecars[0]), 0x0F9E)
        self.assertEqual(sidecars[0][:4], b"\x20\xcd\xcf\xbd")

    def test_build_packs_light_overworld_tilemap_from_json_without_bin(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir) / "zelda3_assets"
            shutil.copytree(GENERATED_ASSETS, asset_dir)
            (asset_dir / "assets/067-kLightOverworldTilemap.bin").unlink(missing_ok=True)
            source_dir = asset_dir / "assets_src/tilemaps"
            source_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(
                GENERATED_ASSETS / "assets_src/tilemaps/light_overworld_tilemap.json",
                source_dir / "light_overworld_tilemap.json",
            )

            result = subprocess.run(
                ["cargo", "build", "-p", "zelda3-bin"],
                cwd=REPO,
                env={**os.environ, "ZELDA3_ASSETS_DIR": str(asset_dir)},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )

        self.assertEqual(result.returncode, 0, result.stdout)

    def _vanilla_dialogue_source(self) -> dict:
        """Vanilla dialogue source (JSON-shaped) extracted from the generated assets."""
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir) / "zelda3_assets"
            shutil.copytree(GENERATED_ASSETS, asset_dir)
            source_result = subprocess.run(
                [
                    "python3",
                    "scripts/dialogue_catalog.py",
                    "--asset-dir",
                    str(asset_dir),
                ],
                cwd=REPO,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            self.assertEqual(source_result.returncode, 0, source_result.stdout)
            source_path = asset_dir / "assets_src/dialogue/dialogue_source.json"
            return json.loads(source_path.read_text())

    def test_build_packs_dialogue_from_committed_messages_toml(self) -> None:
        """The tracked assets/dialogue/messages.toml is the kDialogue authority and
        compiles byte-identically to the vanilla asset (parity lock satisfied)."""
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")
        self.assertTrue((REPO / "assets/dialogue/messages.toml").is_file())

        expected = dialogue_catalog.asset_from_dialogue_source(
            self._vanilla_dialogue_source()
        )

        result = subprocess.run(
            ["cargo", "build", "-p", "zelda3-bin"],
            cwd=REPO,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        self.assertEqual(result.returncode, 0, result.stdout)

        assets = split_built_asset_pack(built_asset_pack())
        self.assertEqual(assets[94][0], "kDialogue")
        self.assertEqual(assets[94][1], expected)
        sidecars = [payload for name, payload in assets if name == "kDialogueSourceSemantic"]
        self.assertEqual(len(sidecars), 1)
        self.assertTrue(sidecars[0].startswith(b"Z3DLGSRCv1\0\0\0\0\0\0"))

    def test_dialogue_messages_override_rebuilds_pack(self) -> None:
        """ZELDA3_DIALOGUE_MESSAGES points the build at an alternate (unlocked) source
        without touching the tracked authority; edits change the packed asset."""
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        vanilla = dialogue_catalog.asset_from_dialogue_source(
            self._vanilla_dialogue_source()
        )

        committed = (REPO / "assets/dialogue/messages.toml").read_text()
        override_text, replaced = re.subn(
            r"(\[\[message\]\]\nid = 1\ntext = )'''.*?'''",
            r"\1'''Changed dialogue for the parity test.'''",
            committed,
            count=1,
        )
        self.assertEqual(replaced, 1, "expected to rewrite message id 1 in messages.toml")

        with TemporaryDirectory() as temp_dir:
            override_path = Path(temp_dir) / "messages.toml"
            override_path.write_text(override_text)
            result = subprocess.run(
                ["cargo", "build", "-p", "zelda3-bin"],
                cwd=REPO,
                env={**os.environ, "ZELDA3_DIALOGUE_MESSAGES": str(override_path)},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            self.assertEqual(result.returncode, 0, result.stdout)

            assets = split_built_asset_pack(built_asset_pack())

        self.assertEqual(assets[94][0], "kDialogue")
        self.assertNotEqual(assets[94][1], vanilla)
        sidecars = [
            payload for name, payload in assets if name == "kDialogueSourceSemantic"
        ]
        self.assertEqual(len(sidecars), 1)
        self.assertTrue(sidecars[0].startswith(b"Z3DLGSRCv1\0\0\0\0\0\0"))

    def test_generated_assets_store_all_tilemaps_as_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        tilemap_bins = sorted((GENERATED_ASSETS / "assets").glob("*Tilemap*.bin"))
        tilemap_sources = sorted((GENERATED_ASSETS / "assets_src/tilemaps").glob("*.json"))

        self.assertEqual(tilemap_bins, [])
        self.assertEqual(
            [path.name for path in tilemap_sources],
            [
                "bg_tilemap_0.json",
                "bg_tilemap_1.json",
                "bg_tilemap_2.json",
                "bg_tilemap_3.json",
                "bg_tilemap_4.json",
                "bg_tilemap_5.json",
                "dark_overworld_tilemap.json",
                "light_overworld_tilemap.json",
            ],
        )

    def test_generated_assets_store_all_palettes_as_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        palette_bins = sorted((GENERATED_ASSETS / "assets").glob("*Palette*.bin"))
        hud_palette_bin = GENERATED_ASSETS / "assets/092-kHudPalData.bin"
        palette_sources = sorted((GENERATED_ASSETS / "assets_src/palettes").glob("*.json"))

        self.assertEqual(palette_bins, [])
        self.assertFalse(hud_palette_bin.exists())
        self.assertEqual(len(palette_sources), 17)

    def test_generated_assets_store_navigation_tables_as_grouped_json_sources(self) -> None:
        if not GENERATED_ASSETS.is_dir():
            self.skipTest(f"missing generated assets: {GENERATED_ASSETS}")

        navigation_bins = []
        for start, end in [(11, 45), (130, 156)]:
            for index in range(start, end + 1):
                navigation_bins.extend((GENERATED_ASSETS / "assets").glob(f"{index:03d}-*.bin"))
        navigation_sources = sorted((GENERATED_ASSETS / "assets_src/navigation").glob("*.json"))

        self.assertEqual(navigation_bins, [])
        self.assertEqual(
            [path.name for path in navigation_sources],
            [
                "dungeon_entrances.json",
                "overworld_exits.json",
                "special_exits.json",
                "starting_points.json",
            ],
        )

    def test_ci_placeholder_assets_still_build(self) -> None:
        with TemporaryDirectory() as temp_dir:
            asset_dir = Path(temp_dir) / "ci-assets/zelda3_assets"
            create_result = subprocess.run(
                [
                    "python3",
                    "scripts/create_ci_assets.py",
                    "--out-dir",
                    str(asset_dir),
                ],
                cwd=REPO,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )
            self.assertEqual(create_result.returncode, 0, create_result.stdout)

            build_result = subprocess.run(
                ["cargo", "check", "-p", "zelda3-bin"],
                cwd=REPO,
                env={**os.environ, "ZELDA3_ASSETS_DIR": str(asset_dir)},
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
            )

        self.assertEqual(build_result.returncode, 0, build_result.stdout)


if __name__ == "__main__":
    unittest.main()
