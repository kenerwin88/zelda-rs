use super::*;
use crate::game_state::constants::DUNGEON_TORCH_ATTR;
use crate::tile_definition::{DungeonRole, NativeTile, TilePair};

fn tile(attribute: u8) -> NativeTile {
    NativeTile::from_cartridge(attribute)
}

/// The room logic's original mask arithmetic, written out per read point.
fn original_role(a: u8) -> DungeonRole {
    if (0x1d..=0x1f).contains(&a) {
        // Dungeon_LoadObjectAttribute: 0x1f1f type 0, 0x1e1e type 1, 0x1d1d type 2.
        DungeonRole::InRoomStaircase { kind: 0x1f - a }
    } else if (0x23..=0x25).contains(&a) {
        DungeonRole::FloorSwitch { variant: a - 0x23 }
    } else if a == 0x26 {
        DungeonRole::SpiralStairHead
    } else if (a & 0xf8) == 0x30 {
        DungeonRole::StairLanding {
            index: usize::from(a & 7),
        }
    } else if a == 0x38 || a == 0x39 {
        DungeonRole::StraightStairHead {
            descending: a == 0x39,
        }
    } else if a == 0x3a || a == 0x3b {
        DungeonRole::StarSwitch { toggled: a == 0x3a }
    } else if a == 0x5e || a == 0x5f {
        DungeonRole::WallSpiralStairHead { second: a == 0x5f }
    } else if a == 0x62 {
        DungeonRole::BombableFloor
    } else if a == 0x63 {
        DungeonRole::MinigameChest
    } else if (a & 0xfc) == 0x6c {
        DungeonRole::Curtain {
            panel: usize::from(a & 3),
        }
    } else if (a & 0xf0) == 0x70 {
        DungeonRole::TrackedObject {
            slot: usize::from(a & 0x0f),
        }
    } else if (a & 0xf0) == 0x80 {
        DungeonRole::OpenDoor
    } else if (a & 0xf0) == 0xb0 {
        DungeonRole::SomariaPipe
    } else if (a & 0xf0) == 0xc0 {
        DungeonRole::Torch {
            slot: usize::from(a & 0x0f),
        }
    } else if (a & 0xf0) == 0xf0 {
        DungeonRole::ClosedDoor {
            slot: usize::from(a & 0x0f),
        }
    } else {
        DungeonRole::None
    }
}

// PushBlock_AttemptToPushTheBlock's original accepted-target list.
const PUSH_BLOCK_ACCEPTED: [u8; 24] = [
    0, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 28, 32, 35, 36, 37, 58, 59, 64, 72, 74, 96, 97, 98,
];

#[test]
fn dungeon_roles_match_the_original_mask_arithmetic() {
    for a in 0..=u8::MAX {
        let t = tile(a);
        assert_eq!(t.dungeon_role(), original_role(a), "{a:#04x}");
        assert_eq!(t.object_slot(), usize::from(a & 0x0f));
        // CalculateTransitionLanding.
        let landing = if a == 0 || a == 9 {
            0
        } else {
            match a & 0x8e {
                0x80 => 1,
                0x82 => 2,
                0x84 | 0x88 => 3,
                0x86 => 4,
                _ => 2,
            }
        };
        assert_eq!(t.transition_landing(), landing, "{a:#04x}");
        assert_eq!(
            t.accepts_push_block(),
            PUSH_BLOCK_ACCEPTED.contains(&a) || a == 100,
            "{a:#04x}"
        );
        // Statue_CheckForSwitch accepts 0x23, 0x24, 0x25, and 0x3b.
        assert_eq!(t.is_floor_switch(), matches!(a, 0x23..=0x25 | 0x3b));
    }
    assert_eq!(NativeTile::torch(3).cartridge_attribute(), 0xc3);
    assert_eq!(NativeTile::stair_landing(5).cartridge_attribute(), 0x35);
    assert_eq!(
        TilePair::stair_landing(0).next_identity(),
        TilePair::stair_landing(1)
    );
    assert_eq!(TilePair::stair_landing(1).uniform(), Some(tile(0x31)));
    // The room recipes' original words, low cell first.
    let encoded = |pair: TilePair| u16::from_le_bytes(pair.0.map(NativeTile::cartridge_attribute));
    assert_eq!(encoded(TilePair::stair_landing(0)), 0x3030);
    assert_eq!(
        encoded(TilePair([NativeTile::LAYER_WALL, NativeTile::GROUND])),
        0x0003
    );
    assert_eq!(
        encoded(TilePair([NativeTile::GROUND, NativeTile::LAYER_WALL])),
        0x0300
    );
    assert_eq!(
        encoded(TilePair([
            NativeTile::LAYER_WALL,
            NativeTile::LAYER_LANDING
        ])),
        0x0a03
    );
    assert_eq!(
        encoded(TilePair([
            NativeTile::LAYER_LANDING,
            NativeTile::LAYER_WALL
        ])),
        0x030a
    );
    for (pair, word) in [
        (NativeTile::IN_ROOM_STAIR_PSEUDO_UP_NORTH, 0x1d1d),
        (NativeTile::SPIRAL_STAIR_HEAD, 0x2626),
        (NativeTile::STRAIGHT_STAIR_UP_HEAD, 0x3838),
        (NativeTile::STRAIGHT_STAIR_DOWN_HEAD, 0x3939),
        (NativeTile::STAR_SWITCH, 0x3b3b),
        (NativeTile::WALL_SPIRAL_STAIR_HEAD, 0x5e5e),
        (NativeTile::WALL_SPIRAL_STAIR_HEAD_2, 0x5f5f),
        (NativeTile::MINIGAME_CHEST, 0x6363),
    ] {
        assert_eq!(encoded(TilePair::repeated(pair)), word);
    }
    assert_eq!(TilePair([tile(0x31), tile(0x30)]).uniform(), None);
}

#[test]
fn transition_landing_reads_the_tile_under_the_player() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    state.follower_link_state_mut().set_position(0x100, 0x100);
    let pos = usize::from(((0x100u16 + 12) & 0x01f8) << 3) + usize::from((0x100u16 + 8) >> 3);
    for a in [0x00, 0x09, 0x80, 0x82, 0x84, 0x88, 0x86, 0xa0, 0x01, 0x8f] {
        state.dungeon_bg2_attributes_mut().set_bg2_attr(pos, a);
        assert_eq!(
            state.CalculateTransitionLanding(),
            tile(a).transition_landing(),
            "{a:#04x}"
        );
        assert_eq!(
            state.game_state.dungeon.room_runtime.landing_class(),
            tile(a).transition_landing()
        );
    }
}

#[test]
fn room_tag_switch_probes_require_a_uniform_cell() {
    fn probe_state(word: [u8; 2], below: [u8; 2]) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        state.follower_link_state_mut().set_position(0x100, 0x100);
        let p = state.RoomTag_GetTilemapCoords() as usize;
        let mut map = state.dungeon_bg2_attributes_mut();
        map.set_bg2_attr(p, word[0]);
        map.set_bg2_attr(p + 1, word[1]);
        map.set_bg2_attr(p + 64, below[0]);
        map.set_bg2_attr(p + 65, below[1]);
        state
    }
    assert_eq!(
        probe_state([0x23, 0x23], [0x23, 0x23]).RoomTag_MaybeCheckShutters(),
        Some(NativeTile::PRESSURE_PLATE)
    );
    assert_eq!(
        probe_state([0x24, 0x24], [0x24, 0x24]).RoomTag_MaybeCheckShutters(),
        Some(tile(0x24))
    );
    assert_eq!(
        probe_state([0x25, 0x25], [0x25, 0x25]).RoomTag_MaybeCheckShutters(),
        None,
        "the second held variant is not a shutter switch"
    );
    assert_eq!(
        probe_state([0x23, 0x24], [0x23, 0x24]).RoomTag_MaybeCheckShutters(),
        None,
        "mixed cells never match the original whole-word compare"
    );
    assert_eq!(
        probe_state([0x23, 0x23], [0x23, 0x00]).RoomTag_MaybeCheckShutters(),
        None,
        "the lower row must repeat the switch"
    );
    assert_eq!(
        probe_state([0x3b, 0x3b], [0x3b, 0x3b]).RoomTag_CheckForPressedSwitch(),
        Some(true)
    );
    assert_eq!(
        probe_state([0x3a, 0x3a], [0x3a, 0x3a]).RoomTag_CheckForPressedSwitch(),
        Some(false)
    );
    assert_eq!(
        probe_state([0x23, 0x23], [0x23, 0x23]).RoomTag_CheckForPressedSwitch(),
        Some(false)
    );
    assert_eq!(
        probe_state([0x24, 0x24], [0x24, 0x24]).RoomTag_CheckForPressedSwitch(),
        None
    );
}

#[test]
fn torch_target_keeps_its_byte_and_only_torches_light() {
    let mut state = ZeldaState::new();
    state.dungeon_torch_mut().set_target(NativeTile::torch(5));
    assert_eq!(state.ram[DUNGEON_TORCH_ATTR], 0xc5);
    assert!(state.game_state.dungeon.torch.targets_torch());
    assert_eq!(state.game_state.dungeon.torch.attr_index(), 5);
    state.dungeon_torch_mut().set_target(tile(0x75));
    assert_eq!(state.ram[DUNGEON_TORCH_ATTR], 0x75);
    assert!(!state.game_state.dungeon.torch.targets_torch());
    assert_eq!(state.game_state.dungeon.torch.attr_index(), 5);
    state.dungeon_torch_mut().clear_target();
    assert_eq!(state.ram[DUNGEON_TORCH_ATTR], 0);
    assert_eq!(state.game_state.dungeon.torch.target(), NativeTile::GROUND);
}
