import copy
import importlib.util
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_modern_sfx_assets.py"
SPEC = importlib.util.spec_from_file_location("validate_modern_sfx_assets", SCRIPT)
validator = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(validator)


class ValidateModernSfxAssetsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.document = json.loads(validator.DEFAULT_ASSETS.read_text(encoding="utf-8"))

    def test_checked_in_assets_are_valid_and_complete(self) -> None:
        self.assertEqual(validator.validate(self.document), [])
        self.assertEqual(len(self.document["programs"]), 342)
        self.assertEqual(len(self.document["exact_dsp_steps"]), 570)
        self.assertEqual(len(self.document["pitch_events"]), 80)

    def test_rejects_unreviewed_duplicate_and_invalid_step(self) -> None:
        document = copy.deepcopy(self.document)
        document["programs"][0]["promotion_status"] = "needs_review"
        document["programs"][0]["steps"][0]["voice"] = 8
        document["programs"][0]["steps"][0]["waveform"] = "PCM"
        document["programs"].append(copy.deepcopy(document["programs"][0]))

        errors = validator.validate(document)

        self.assertTrue(any("not review_ready" in error for error in errors))
        self.assertTrue(any("duplicates" in error for error in errors))
        self.assertTrue(any("voice must be" in error for error in errors))
        self.assertTrue(any("waveform is invalid" in error for error in errors))

    def test_rejects_unknown_and_duplicate_exact_records(self) -> None:
        document = copy.deepcopy(self.document)
        document["unexpected"] = True
        document["programs"][0]["steps"][0]["envelope"]["typo"] = 1
        document["exact_dsp_steps"].append(copy.deepcopy(document["exact_dsp_steps"][0]))
        document["pitch_events"].append(copy.deepcopy(document["pitch_events"][0]))

        errors = validator.validate(document)

        self.assertTrue(any("document has unknown fields" in error for error in errors))
        self.assertTrue(any("envelope has unknown fields" in error for error in errors))
        self.assertTrue(any("exact_dsp_steps" in error and "duplicates" in error for error in errors))
        self.assertTrue(any("pitch_events" in error and "duplicates" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
