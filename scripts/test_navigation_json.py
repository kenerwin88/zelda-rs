#!/usr/bin/env python3
"""Tests for grouped navigation table JSON sources."""

from __future__ import annotations

import unittest

import navigation_json


class NavigationJsonTests(unittest.TestCase):
    def test_entrance_records_round_trip_each_legacy_asset(self) -> None:
        assets = {
            "kEntranceData_rooms": (0x1234).to_bytes(2, "little"),
            "kEntranceData_relativeCoords": bytes([1, 2, 3, 4, 5, 6, 7, 8]),
            "kEntranceData_scrollX": (0x2345).to_bytes(2, "little"),
            "kEntranceData_scrollY": (0x3456).to_bytes(2, "little"),
            "kEntranceData_playerX": (0x4567).to_bytes(2, "little"),
            "kEntranceData_playerY": (0x5678).to_bytes(2, "little"),
            "kEntranceData_cameraX": (0x6789).to_bytes(2, "little"),
            "kEntranceData_cameraY": (0x789A).to_bytes(2, "little"),
            "kEntranceData_blockset": bytes([9]),
            "kEntranceData_floor": (-2).to_bytes(1, "little", signed=True),
            "kEntranceData_palace": (-1).to_bytes(1, "little", signed=True),
            "kEntranceData_doorwayOrientation": bytes([10]),
            "kEntranceData_startingBg": bytes([11]),
            "kEntranceData_quadrant1": bytes([12]),
            "kEntranceData_quadrant2": bytes([13]),
            "kEntranceData_doorSettings": (0x8004).to_bytes(2, "little"),
            "kEntranceData_musicTrack": bytes([14]),
        }

        source = navigation_json.entrance_source_from_assets(
            assets,
            asset="kEntranceData",
            asset_index_range=[11, 27],
        )

        self.assertEqual(source["format"], navigation_json.FORMAT_DUNGEON_ENTRANCES)
        self.assertEqual(source["records"][0]["floor"], -2)
        self.assertEqual(source["records"][0]["relative_coords"], [1, 2, 3, 4, 5, 6, 7, 8])
        for asset, payload in assets.items():
            self.assertEqual(navigation_json.bytes_for_asset(source, asset), payload)

    def test_exit_records_round_trip_signed_fields(self) -> None:
        assets = {
            "kExitData_ScreenIndex": bytes([0x3F]),
            "kExitDataRooms": (0x0102).to_bytes(2, "little"),
            "kExitData_Map16LoadSrcOff": (0x0304).to_bytes(2, "little"),
            "kExitData_ScrollX": (0x0506).to_bytes(2, "little"),
            "kExitData_ScrollY": (0x0708).to_bytes(2, "little"),
            "kExitData_XCoord": (0x090A).to_bytes(2, "little"),
            "kExitData_YCoord": (0x0B0C).to_bytes(2, "little"),
            "kExitData_CameraXScroll": (0x0D0E).to_bytes(2, "little"),
            "kExitData_CameraYScroll": (0x0F10).to_bytes(2, "little"),
            "kExitData_NormalDoor": (0x1112).to_bytes(2, "little"),
            "kExitData_FancyDoor": (0x1314).to_bytes(2, "little"),
            "kExitData_Unk1": (-12).to_bytes(1, "little", signed=True),
            "kExitData_Unk3": (-34).to_bytes(1, "little", signed=True),
        }

        source = navigation_json.exit_source_from_assets(assets, asset_index_range=[130, 142])

        self.assertEqual(source["format"], navigation_json.FORMAT_OVERWORLD_EXITS)
        self.assertEqual(source["records"][0]["unk1"], -12)
        self.assertEqual(source["records"][0]["unk3"], -34)
        for asset, payload in assets.items():
            self.assertEqual(navigation_json.bytes_for_asset(source, asset), payload)

    def test_special_exit_records_round_trip_signed_words(self) -> None:
        assets = {
            "kSpExit_Top": (0).to_bytes(2, "little"),
            "kSpExit_Bottom": (0x0120).to_bytes(2, "little"),
            "kSpExit_Left": (0x0200).to_bytes(2, "little"),
            "kSpExit_Right": (0x0300).to_bytes(2, "little"),
            "kSpExit_Tab4": (-224).to_bytes(2, "little", signed=True),
            "kSpExit_Tab5": (-4).to_bytes(2, "little", signed=True),
            "kSpExit_Tab6": (0x0400).to_bytes(2, "little", signed=True),
            "kSpExit_Tab7": (0x0500).to_bytes(2, "little", signed=True),
            "kSpExit_LeftEdgeOfMap": (0x0600).to_bytes(2, "little"),
            "kSpExit_Dir": bytes([4]),
            "kSpExit_SprGfx": bytes([0x0E]),
            "kSpExit_AuxGfx": bytes([0x2F]),
            "kSpExit_PalBg": bytes([0x0A]),
            "kSpExit_PalSpr": bytes([8]),
        }

        source = navigation_json.special_exit_source_from_assets(
            assets,
            asset_index_range=[143, 156],
        )

        self.assertEqual(source["format"], navigation_json.FORMAT_SPECIAL_EXITS)
        self.assertEqual(source["records"][0]["tab4"], -224)
        for asset, payload in assets.items():
            self.assertEqual(navigation_json.bytes_for_asset(source, asset), payload)

    def test_rejects_mismatched_parallel_table_lengths(self) -> None:
        assets = {
            "kExitData_ScreenIndex": bytes([1, 2]),
            "kExitDataRooms": (3).to_bytes(2, "little"),
            "kExitData_Map16LoadSrcOff": (3).to_bytes(2, "little"),
            "kExitData_ScrollX": (3).to_bytes(2, "little"),
            "kExitData_ScrollY": (3).to_bytes(2, "little"),
            "kExitData_XCoord": (3).to_bytes(2, "little"),
            "kExitData_YCoord": (3).to_bytes(2, "little"),
            "kExitData_CameraXScroll": (3).to_bytes(2, "little"),
            "kExitData_CameraYScroll": (3).to_bytes(2, "little"),
            "kExitData_NormalDoor": (3).to_bytes(2, "little"),
            "kExitData_FancyDoor": (3).to_bytes(2, "little"),
            "kExitData_Unk1": bytes([3]),
            "kExitData_Unk3": bytes([3]),
        }

        with self.assertRaisesRegex(ValueError, "record count"):
            navigation_json.exit_source_from_assets(assets, asset_index_range=[130, 142])


if __name__ == "__main__":
    unittest.main()
