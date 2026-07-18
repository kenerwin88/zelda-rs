import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REMOVED_ORACLE = b"bs" + b"nes"


class RemovedOracleReferenceTests(unittest.TestCase):
    def test_current_tracked_tree_has_no_removed_oracle_references(self) -> None:
        tracked = subprocess.run(
            ["git", "ls-files", "-z"],
            cwd=ROOT,
            check=True,
            capture_output=True,
        ).stdout.split(b"\0")
        offenders = []
        for encoded_path in tracked:
            if not encoded_path:
                continue
            path = ROOT / encoded_path.decode()
            if not path.is_file():
                continue
            if REMOVED_ORACLE in path.read_bytes().lower():
                offenders.append(path.relative_to(ROOT).as_posix())

        self.assertEqual(offenders, [])


if __name__ == "__main__":
    unittest.main()
