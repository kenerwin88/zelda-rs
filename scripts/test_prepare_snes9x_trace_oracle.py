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
                "apu/bapu/dsp/SPC_DSP.cpp",
                "apu/bapu/dsp/SPC_DSP.h",
                "apu/bapu/smp/core.cpp",
                "apu/bapu/smp/smp.cpp",
                "cpuexec.cpp",
                "dma.cpp",
                "getset.h",
                "gfx.cpp",
                "libretro/Makefile.common",
                "libretro/libretro.cpp",
                "ppu.cpp",
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
                patch=None,
            )
            trace_receipt = MODULE.write_receipt(
                trace_core,
                variant="trace",
                patch=patch,
            )

            stock = json.loads(stock_receipt.read_text())
            trace = json.loads(trace_receipt.read_text())
            self.assertEqual(stock["core_version"], "1.63")
            self.assertEqual(stock["source_tag"], "1.63")
            self.assertEqual(stock["variant"], "stock")
            self.assertIsNone(stock["patch"])
            self.assertIsNone(stock["patch_sha256"])
            self.assertEqual(trace["variant"], "trace")
            self.assertEqual(trace["patch"], str(patch))
            self.assertEqual(trace["patch_sha256"], MODULE.sha256(patch))


if __name__ == "__main__":
    unittest.main()
