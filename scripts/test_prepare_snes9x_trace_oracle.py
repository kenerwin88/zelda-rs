#!/usr/bin/env python3

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("prepare_snes9x_trace_oracle.py")
SPEC = importlib.util.spec_from_file_location("prepare_snes9x_trace_oracle", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class PrepareSnes9xTraceOracleTests(unittest.TestCase):
    def test_obj_cache_patch_exports_addressed_active_name_pages(self) -> None:
        patch = MODULE.TRACE_PATCHES[1].read_text()

        self.assertIn("presented_obj_tile_slot_count = 512", patch)
        self.assertIn("presented_obj_tile_pixels = 64", patch)
        self.assertIn("case 29:", patch)
        self.assertIn("case 30:", patch)
        self.assertIn("case 45:", patch)
        self.assertIn("Zelda3TracePresentedObjTileCacheValue(2, index)", patch)
        self.assertIn("case 46:", patch)
        self.assertIn("Zelda3TracePresentedObjTileCacheValue(3, index)", patch)
        self.assertNotIn("case 47:", patch)
        self.assertNotIn("case 48:", patch)
        self.assertIn("PPU.OBJNameBase + 0x2000 + PPU.OBJNameSelect", patch)
        self.assertIn("& 0x7fff", patch)
        self.assertIn(
            "std::fill_n(presented_obj_tile_word_address,", patch
        )
        self.assertIn("captured_obj_tile[tile_number]", patch)
        self.assertIn("const int slot = page * 256 + tile", patch)
        self.assertIn("presented_obj_tile_word_address[slot] = word_address", patch)
        self.assertIn("case 0: return 1; // Addressed OBJ cache ABI version.", patch)
        self.assertIn("case 3: return presented_obj_tile_page_base[0]", patch)
        self.assertIn("case 4: return presented_obj_tile_page_base[1]", patch)
        self.assertNotIn("presented_obj_tile_count", patch)

    def test_trace_patch_stack_keeps_ledgers_before_presentation_receipts(self) -> None:
        self.assertEqual(
            MODULE.TRACE_PATCHES[-7].name,
            "zelda3-dsp-phase-ledger.patch",
        )
        patch = MODULE.TRACE_PATCHES[-7].read_text()
        self.assertIn("zelda3_snes9x_debug_dsp_ledger_abi_version", patch)
        self.assertIn("zelda3_ledger_copy_state", patch)
        self.assertIn("Zelda3TraceDspLedgerBranch", patch)
        self.assertEqual(MODULE.TRACE_PATCHES[-6].name, "zelda3-dma-ledger.patch")
        dma_patch = MODULE.TRACE_PATCHES[-6].read_text()
        self.assertIn("zelda3_snes9x_debug_dma_ledger_count", dma_patch)
        self.assertIn("Zelda3TraceDmaByteBegin", dma_patch)
        self.assertEqual(
            MODULE.TRACE_PATCHES[-5].name,
            "zelda3-trace-presented-cgram.patch",
        )
        cgram_patch = MODULE.TRACE_PATCHES[-5].read_text()
        self.assertIn("Zelda3TraceBeginPresentedPpuState", cgram_patch)
        self.assertIn("Zelda3TracePresentedCgramValue", cgram_patch)
        self.assertIn("std::memcmp(scanout_cgram, PPU.CGDATA", cgram_patch)
        self.assertIn("case 36: return Zelda3TracePresentedCgramValue", cgram_patch)
        self.assertEqual(
            MODULE.TRACE_PATCHES[-4].name,
            "zelda3-trace-presented-hud.patch",
        )
        hud_patch = MODULE.TRACE_PATCHES[-4].read_text()
        self.assertIn("Zelda3TracePresentedHudTilemapValue", hud_patch)
        self.assertIn("presented_hud_word_address = 0x6040", hud_patch)
        self.assertIn("case 37: return Zelda3TracePresentedHudTilemapValue", hud_patch)
        self.assertIn("Zelda3TraceCaptureRenderedPpuRange(GFX.StartY, GFX.EndY)", hud_patch)
        self.assertIn("scanout_hud_tilemap_valid", hud_patch)
        self.assertIn("presented_hud_first_line = 16", hud_patch)
        self.assertIn("case 38: return Zelda3TracePresentedInidispValue", hud_patch)
        self.assertIn("Zelda3TraceSetPresentedTopCrop(overscan_offset)", hud_patch)
        self.assertIn("case 40: return Zelda3TracePresentedAnimatedBgDestination", hud_patch)
        self.assertEqual(
            MODULE.TRACE_PATCHES[-3].name,
            "zelda3-trace-presented-bg-tilemaps.patch",
        )
        bg_patch = MODULE.TRACE_PATCHES[-3].read_text()
        self.assertIn("Zelda3TracePresentedBgTilemapMetaValue", bg_patch)
        self.assertIn("Zelda3TracePresentedBgTilemapWordValue", bg_patch)
        self.assertIn("std::memcmp(scanout_vram, Memory.VRAM", bg_patch)
        self.assertIn("case 41: return Zelda3TracePresentedBgTilemapMetaValue", bg_patch)
        self.assertEqual(
            MODULE.TRACE_PATCHES[-2].name,
            "zelda3-trace-presented-window-mask.patch",
        )
        window_patch = MODULE.TRACE_PATCHES[-2].read_text()
        self.assertIn("Zelda3TracePresentedScreenWindowedValue", window_patch)
        self.assertIn("Zelda3TracePresentedWindowPredicateValue", window_patch)
        self.assertIn("Memory.FillRAM[0x212e]", window_patch)
        self.assertIn("case 43: return Zelda3TracePresentedScreenWindowedValue", window_patch)
        self.assertIn("case 44: return Zelda3TracePresentedWindowPredicateValue", window_patch)
        self.assertIn("PPU.ClipWindowOverlapLogic[layer]", window_patch)
        self.assertEqual(
            MODULE.TRACE_PATCHES[-1].name,
            "zelda3-trace-joypad-publication.patch",
        )
        joypad_patch = MODULE.TRACE_PATCHES[-1].read_text()
        self.assertIn("joypad_high", joypad_patch)
        self.assertIn("joypad_low_filtered", joypad_patch)
        self.assertIn("Memory.RAM[0x00f0]", joypad_patch)
        self.assertIn("Memory.RAM[0x00f6]", joypad_patch)
        self.assertIn("spotlight_var4_low", joypad_patch)
        self.assertIn("spotlight_lower_cursor", joypad_patch)
        self.assertIn("Memory.RAM[0x067a]", joypad_patch)
        self.assertIn("Memory.RAM[0x0006]", joypad_patch)

    def test_lock_pins_the_official_snes9x_1_63_release(self) -> None:
        lock = json.loads(MODULE.LOCK_PATH.read_text())

        self.assertEqual(lock["core_name"], "Snes9x")
        self.assertEqual(lock["core_version"], "1.63")
        self.assertEqual(lock["source_tag"], "1.63")
        self.assertEqual(lock["source_url"], "https://github.com/snes9xgit/snes9x.git")
        self.assertEqual(
            lock["source_revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37",
        )
        self.assertEqual(MODULE.VERSION, lock["core_version"])
        self.assertEqual(MODULE.REVISION, lock["source_revision"])
        self.assertEqual(MODULE.SOURCE_URL, lock["source_url"])

    def test_patch_scope_is_small_and_explicit(self) -> None:
        self.assertEqual(
            MODULE.EXPECTED_PATCH_PATHS,
            {
                "apu/apu.cpp",
                "apu/apu.h",
                "apu/bapu/dsp/SPC_DSP.cpp",
                "apu/bapu/dsp/SPC_DSP.h",
                "apu/bapu/smp/core.cpp",
                "apu/bapu/smp/memory.cpp",
                "apu/bapu/smp/smp.cpp",
                "cpuexec.cpp",
                "cpuops.cpp",
                "dma.cpp",
                "getset.h",
                "gfx.cpp",
                "libretro/Makefile.common",
                "libretro/libretro.cpp",
                "ppu.cpp",
                "ppu.h",
                "tileimpl.h",
                "tileimpl-n1x1.cpp",
                "tileimpl-n2x1.cpp",
                "zelda3_trace.cpp",
                "zelda3_trace.h",
            },
        )

    def test_platform_build_uses_the_native_libretro_extension(self) -> None:
        platform, artifact = MODULE.build_settings()
        if MODULE.sys.platform == "darwin":
            self.assertEqual((platform, artifact), ("osx", "snes9x_libretro.dylib"))
        elif MODULE.sys.platform.startswith("linux"):
            self.assertEqual((platform, artifact), ("unix", "snes9x_libretro.so"))

    def test_changed_paths_preserves_porcelain_status_spacing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checkout = Path(directory)
            MODULE.run("git", "init", "-q", cwd=checkout)
            MODULE.run("git", "config", "user.email", "test@example.com", cwd=checkout)
            MODULE.run("git", "config", "user.name", "Test", cwd=checkout)
            tracked = checkout / "tracked.txt"
            tracked.write_text("before\n")
            MODULE.run("git", "add", "tracked.txt", cwd=checkout)
            MODULE.run("git", "commit", "-qm", "fixture", cwd=checkout)
            tracked.write_text("after\n")
            (checkout / "new.txt").write_text("new\n")

            self.assertEqual(MODULE.changed_paths(checkout), {"tracked.txt", "new.txt"})

    def test_changed_paths_ignores_local_parity_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checkout = Path(directory)
            MODULE.run("git", "init", "-q", cwd=checkout)
            artifact = checkout / "target/parity-failures/example/diff.json"
            artifact.parent.mkdir(parents=True)
            artifact.write_text("{}\n")

            self.assertEqual(MODULE.changed_paths(checkout), set())

    def test_receipts_distinguish_stock_and_trace_builds(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stock_core = root / "snes9x_libretro.dylib"
            trace_core = root / "snes9x_libretro_trace.dylib"
            patch = root / "trace.patch"
            stock_core.write_bytes(b"stable-core")
            trace_core.write_bytes(b"traced-stable-core")
            patch.write_bytes(b"trace-patch")

            stock_receipt = MODULE.write_receipt(
                stock_core,
                variant="stock",
                patches=(),
            )
            trace_receipt = MODULE.write_receipt(
                trace_core,
                variant="trace",
                patches=(patch,),
            )

            stock = json.loads(stock_receipt.read_text())
            trace = json.loads(trace_receipt.read_text())
            self.assertEqual(stock["core_version"], "1.63")
            self.assertEqual(stock["source_tag"], "1.63")
            self.assertEqual(stock["oracle_lock_sha256"], MODULE.sha256(MODULE.LOCK_PATH))
            self.assertEqual(stock["variant"], "stock")
            self.assertIsNone(stock["patch"])
            self.assertIsNone(stock["patch_sha256"])
            self.assertEqual(stock["patches"], [])
            self.assertEqual(stock["patch_sha256s"], [])
            self.assertEqual(trace["variant"], "trace")
            self.assertEqual(trace["patch"], str(patch))
            self.assertEqual(trace["patches"], [str(patch)])
            self.assertEqual(trace["patch_sha256s"], [MODULE.sha256(patch)])
            self.assertEqual(trace["patch_sha256"], MODULE.sha256(patch))

    def test_trace_patch_has_cartridge_only_rng_event(self) -> None:
        trace_patch = MODULE.TRACE_PATCHES[0].read_text()
        self.assertIn('has_token(events, "rom-rng")', trace_patch)
        self.assertIn("TRACE_ROM_RNG", trace_patch)
        self.assertIn("(Registers.PBPC & 0xffff) == 0xba7f", trace_patch)


if __name__ == "__main__":
    unittest.main()
