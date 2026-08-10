import importlib.util
import io
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("extract_snes9x_rom_random.py")
SPEC = importlib.util.spec_from_file_location("extract_snes9x_rom_random", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class ExtractSnes9xRomRandomTests(unittest.TestCase):
    def test_extracts_rng_writes_by_explicit_retro_run(self) -> None:
        trace = io.StringIO(
            "\n".join(
                [
                    '{"event":"frame","run":9,"frame":10}',
                    '{"event":"rng-ppu-read","run":9,"address":8508,"value":12}',
                    '{"event":"rng-write","run":9,"frame":10,"pc":899711,"address":4001,"value":5,"carry":0}',
                    '{"event":"rng-write","run":9,"frame":10,"pc":899711,"address":4001,"value":255,"carry":1}',
                    '{"event":"rng-write","run":9,"frame":10,"pc":57291,"address":4001,"value":36,"carry":0}',
                    '{"event":"wram-write","run":10,"address":32,"value":1}',
                ]
            )
        )

        samples = MODULE.extract_samples(trace)
        output = io.StringIO()
        MODULE.write_script(samples, output)

        self.assertEqual(samples, [(9, 5, False), (9, 255, True)])
        self.assertTrue(
            output.getvalue().endswith(
                "9 0x05 carry=0\n9 0xff carry=1\n"
            )
        )

    def test_rejects_rng_write_without_host_run(self) -> None:
        trace = io.StringIO(
            '{"event":"rng-write","frame":10,"pc":899711,"address":4001,"value":5,"carry":0}\n'
        )

        with self.assertRaisesRegex(ValueError, "missing.*run"):
            MODULE.extract_samples(trace)

    def test_rejects_rng_write_without_carry(self) -> None:
        trace = io.StringIO(
            '{"event":"rng-write","run":9,"pc":899711,"address":4001,"value":5}\n'
        )

        with self.assertRaisesRegex(ValueError, "invalid carry"):
            MODULE.extract_samples(trace)


if __name__ == "__main__":
    unittest.main()
