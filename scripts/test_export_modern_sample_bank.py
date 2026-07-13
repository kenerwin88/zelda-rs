import copy
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "export_modern_sample_bank.py"
SPEC = importlib.util.spec_from_file_location("export_modern_sample_bank", SCRIPT)
exporter = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(exporter)


class ExportModernSampleBankTests(unittest.TestCase):
    def test_checked_in_pack_is_valid_and_complete(self) -> None:
        manifest = exporter.DEFAULT_OUTPUT / "manifest.json"
        document = json.loads(manifest.read_text(encoding="utf-8"))

        self.assertEqual(exporter.validate_document(document, manifest.parent), [])
        self.assertEqual(len(document["banks"]), 3)
        self.assertEqual([len(bank["instruments"]) for bank in document["banks"]], [25, 25, 25])
        self.assertLess(len(document["samples"]), 25)

    def test_export_is_cumulative_and_deduplicates_samples(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            output = root / "output"
            source.mkdir()
            sample = bytes([0x03]) + bytes(8)
            directory_entry = (0x4000).to_bytes(2, "little") + (0x4000).to_bytes(2, "little")

            def upload(blocks: list[tuple[int, bytes]]) -> bytes:
                data = bytearray()
                for address, block in blocks:
                    data += len(block).to_bytes(2, "little")
                    data += address.to_bytes(2, "little")
                    data += block
                data += b"\0\0"
                return bytes(data)

            base = upload([(exporter.DIRECTORY, directory_entry * 25), (0x4000, sample), (0xD000, b"A")])
            overlay = upload([(0xD000, b"B")])
            for (_, _, filename), data in zip(exporter.BANK_FILES, (base, overlay, overlay)):
                (source / filename).write_bytes(data)

            document, files = exporter.build_document(source)
            exporter.write_export(document, files, output)

            self.assertEqual(exporter.validate_document(document, output), [])
            self.assertEqual(len(document["samples"]), 1)
            self.assertEqual(files[document["banks"][0]["echo_seed"]["file"]][0x800], ord("A"))
            self.assertEqual(files[document["banks"][1]["echo_seed"]["file"]][0x800], ord("B"))

    def test_validator_rejects_duplicate_source_and_corrupt_sample(self) -> None:
        manifest = exporter.DEFAULT_OUTPUT / "manifest.json"
        document = json.loads(manifest.read_text(encoding="utf-8"))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path in exporter.DEFAULT_OUTPUT.glob("**/*"):
                if path.is_file():
                    destination = root / path.relative_to(exporter.DEFAULT_OUTPUT)
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(path.read_bytes())
            broken = copy.deepcopy(document)
            broken["banks"][0]["instruments"].append(copy.deepcopy(broken["banks"][0]["instruments"][0]))
            broken["banks"][0]["instruments"][1]["loop_offset"] = 1
            sample_path = root / broken["samples"][0]["file"]
            sample_path.write_bytes(sample_path.read_bytes()[:-1] + b"\xff")

            errors = exporter.validate_document(broken, root)

            self.assertTrue(any("duplicates source" in error for error in errors))
            self.assertTrue(any("loop_offset is invalid" in error for error in errors))
            self.assertTrue(any("sha256 mismatch" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
