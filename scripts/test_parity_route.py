#!/usr/bin/env python3

import importlib.util
import json
import sys
import unittest
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR))


def load_script(name: str):
    path = SCRIPT_DIR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class ParityRouteTests(unittest.TestCase):
    def test_all_parity_entry_points_select_the_authoritative_existing_route(self):
        recorder = load_script("snes9x_route_recorder")
        precommit = load_script("precommit_snes9x_parity_gate")
        probe = load_script("parity_probe")

        expected = (SCRIPT_DIR.parent / "routes" / "crystal4_II").resolve()
        self.assertEqual(recorder.DEFAULT_PROJECT.resolve(), expected)
        self.assertEqual(precommit.DEFAULT_PROJECT.resolve(), expected)
        self.assertEqual((probe.ROOT / probe.DEFAULT_PROJECT).resolve(), expected)
        self.assertEqual(
            json.loads((expected / "manifest.json").read_text())["kind"],
            "zelda3_snes9x_route_recording_v1",
        )


if __name__ == "__main__":
    unittest.main()
