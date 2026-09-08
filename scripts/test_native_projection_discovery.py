import pathlib
import tempfile
import unittest

import find_dual_ownership as ownership


class ProjectionDiscoveryTests(unittest.TestCase):
    def test_nested_codecs_and_publication_helpers_remain_visible(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "player").mkdir()
            (root / "native_tests").mkdir()
            (root / "native_tests" / "fixture.rs").write_text("struct Ignored {}")
            (root / "state.rs").write_text("""
                struct GameState {
                    player: PlayerState,
                }
                struct PlayerState {
                    magic: MagicState,
                }
                struct MagicState {}
                impl GameState {
                    fn write_to_ram(&self, ram: &mut [u8]) {
                        self.player.write_to_ram(ram);
                    }
                }
            """)
            (root / "player" / "codec.rs").write_text("""
                impl PlayerState {
                    fn write_to_ram(&self, ram: &mut [u8]) {
                        self.publish_resources(ram);
                        self.magic.publish_amount(ram);
                    }
                    fn publish_resources(&self, ram: &mut [u8]) {
                        ram[RESOURCE] = 1;
                    }
                }
                impl MagicState {
                    fn publish_amount(&self, ram: &mut [u8]) {
                        ram[MAGIC] = 2;
                    }
                }
            """)
            files = ownership.native_state_files(root)
            self.assertEqual(len(files), 2)
            expand = ownership.projection_helper_expander(files)
            writes = list(ownership.extract_writes(
                expand("PlayerState", "write_to_ram"), {"RESOURCE": 0x12, "MAGIC": 0x13}, []))
            self.assertEqual({(start, end) for start, end, _ in writes}, {(0x12, 0x13), (0x13, 0x14)})
            # Child projections remain separate owners, rather than duplicated
            # as writes owned by both GameState and PlayerState.
            self.assertEqual(list(ownership.extract_writes(expand("GameState", "write_to_ram"), {}, [])), [])
            self.assertEqual(ownership.collect_projection_reachable(files), {"GameState", "PlayerState"})


if __name__ == "__main__":
    unittest.main()
