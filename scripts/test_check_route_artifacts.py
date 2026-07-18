import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("check_route_artifacts.py")
SPEC = importlib.util.spec_from_file_location("check_route_artifacts", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class RouteArtifactPolicyTests(unittest.TestCase):
    def test_generated_per_frame_and_comparison_outputs_are_not_versioned(self):
        root = Path("routes")
        self.assertTrue(
            MODULE.is_generated_large_artifact(
                root / "route/takes/0000/frame_receipts.jsonl", root
            )
        )
        self.assertTrue(
            MODULE.is_generated_large_artifact(
                root / "route/comparisons/take-0000/result.json", root
            )
        )
        self.assertFalse(
            MODULE.is_generated_large_artifact(
                root / "route/boundaries/0000/oracle.state", root
            )
        )

    def test_oversized_versioned_candidate_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            routes = Path(tmp) / "routes"
            route = routes / "large-route"
            route.mkdir(parents=True)
            oversized = route / "large.state"
            oversized.write_bytes(b"x" * (MODULE.MAX_VERSIONED_FILE_BYTES + 1))

            errors = MODULE.validate_routes(routes)

            self.assertEqual(len(errors), 1)
            self.assertIn("versioned-file limit", errors[0])

    def test_oversized_total_project_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            routes = Path(tmp) / "routes"
            route = routes / "wide-route"
            route.mkdir(parents=True)
            (route / "manifest.json").write_text(
                json.dumps(
                    {
                        "kind": MODULE.RECORDING_KIND,
                        "boundaries": [],
                        "takes": [],
                    }
                )
            )
            (route / "labels.json").write_text("{}")
            (route / "sram-origin.json").write_text("{}")
            (route / "one.bin").write_bytes(b"123456")
            (route / "two.bin").write_bytes(b"123456")

            errors = MODULE.validate_routes(
                routes, max_file_bytes=1000, max_project_bytes=10
            )

            self.assertTrue(any("versioned-project limit" in error for error in errors))

    def test_absolute_sram_origin_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            route = Path(tmp) / "route"
            route.mkdir()
            (route / "manifest.json").write_text(
                json.dumps(
                    {
                        "kind": MODULE.RECORDING_KIND,
                        "boundaries": [],
                        "takes": [],
                    }
                )
            )
            (route / "labels.json").write_text("{}")
            (route / "sram-origin.json").write_text(
                json.dumps({"source": "file", "path": "/private/save.srm"})
            )

            errors = MODULE.validate_project(route)

            self.assertEqual(len(errors), 1)
            self.assertIn("portable", errors[0])

    def test_repository_routes_pass_policy(self):
        self.assertEqual(MODULE.validate_routes(), [])


if __name__ == "__main__":
    unittest.main()
