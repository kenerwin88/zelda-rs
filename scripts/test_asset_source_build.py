#!/usr/bin/env python3
"""Build integration test for readable asset sources."""

from __future__ import annotations

import os
import shutil
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory


REPO = Path(__file__).resolve().parents[1]
GENERATED_ASSETS = REPO / "generated/zelda3_assets"


class AssetSourceBuildTests(unittest.TestCase):
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
                REPO / "assets_src/tilemaps/light_overworld_tilemap.json",
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
