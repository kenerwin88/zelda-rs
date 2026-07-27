#!/usr/bin/env python3

import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("prepare_snes9x_trace_oracle.py")
SPEC = importlib.util.spec_from_file_location("prepare_snes9x_trace_oracle", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class PrepareSnes9xTraceOracleTests(unittest.TestCase):
    def test_patch_scope_is_small_and_explicit(self) -> None:
        self.assertEqual(
            MODULE.EXPECTED_PATCH_PATHS,
            {
                "cpuexec.cpp",
                "dma.cpp",
                "getset.h",
                "libretro/Makefile.common",
                "libretro/libretro.cpp",
                "ppu.cpp",
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


if __name__ == "__main__":
    unittest.main()
