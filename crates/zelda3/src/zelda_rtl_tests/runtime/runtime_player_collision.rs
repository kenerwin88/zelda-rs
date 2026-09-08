use super::*;
use crate::game_state::constants::{
    CHEAT_WALK_THROUGH_WALLS, DUNGEON_BG2_ATTR_TABLE, DUNG_HDR_COLLISION, IS_STANDING_IN_DOORWAY,
    LINK_IS_ON_LOWER_LEVEL, LINK_X_COORD, LINK_X_VELOCITY, LINK_Y_COORD, LINK_Y_VELOCITY,
    PLAYER_IS_INDOORS, TILEMAP_LOCATION_CALC_MASK,
};
use crate::game_state::{CollisionAxis, CollisionDirection, CollisionOrder, MovementProbeKind};
use sha2::{Digest, Sha256};

#[test]
fn player_collision_matches_frozen_runtime_effects() {
    let mut cases = String::new();
    for seed in 0..64u32 {
        for operation in 0..11 {
            let mut state = collision_test_state(seed, operation);
            let direction = (seed % 4) as u8;
            match operation {
                0 => state.detect_player_movement(
                    CollisionAxis::Vertical,
                    CollisionDirection::from_legacy(direction),
                    MovementProbeKind::Cardinal,
                ),
                1 => state.detect_player_movement(
                    CollisionAxis::Horizontal,
                    CollisionDirection::from_legacy(direction),
                    MovementProbeKind::Cardinal,
                ),
                2 => state.detect_player_movement(
                    CollisionAxis::Vertical,
                    CollisionDirection::from_legacy(direction),
                    MovementProbeKind::Slope,
                ),
                3 => state.detect_player_movement(
                    CollisionAxis::Horizontal,
                    CollisionDirection::from_legacy(direction),
                    MovementProbeKind::Slope,
                ),
                4 => state.player_tile_detect_nearby(),
                5 => state.tile_check_for_mirror_bonk(),
                6 => state.start_movement_collision_checks(CollisionAxis::Vertical),
                7 => state.start_movement_collision_checks(CollisionAxis::Horizontal),
                8 => state.run_slope_collision_checks(CollisionOrder::VerticalFirst),
                9 => state.run_slope_collision_checks(CollisionOrder::HorizontalFirst),
                10 => state.link_handle_cardinal_collision(),
                _ => unreachable!(),
            }
            let mut digest = Sha256::new();
            digest.update(&state.ram);
            let mut projected = vec![0xa5; WRAM_SIZE];
            state.game_state.write_to_ram(&mut projected);
            digest.update(&projected);
            cases.push_str(&format!("{seed}/{operation}: {:x}\n", digest.finalize()));
        }
    }
    assert_eq!(
        cases,
        include_str!("../../../testdata/player-collision-869fdb0c.txt")
    );
}

fn collision_test_state(seed: u32, operation: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    let mut rng = seed + 1;
    if operation < 6 {
        for byte in &mut state.ram {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            *byte = (rng >> 24) as u8;
        }
    }
    state.ram[PLAYER_IS_INDOORS] = 1;
    state.ram[LINK_IS_ON_LOWER_LEVEL] = (seed % 3) as u8;
    state.ram[CHEAT_WALK_THROUGH_WALLS] = 0;
    state.ram[DUNG_HDR_COLLISION] = (seed % 5) as u8;
    state.ram[IS_STANDING_IN_DOORWAY] = (seed % 3) as u8;
    state.ram[LINK_X_VELOCITY] = [0, 1, 0xff, 3][seed as usize % 4];
    state.ram[LINK_Y_VELOCITY] = [0xff, 0, 2, 1][seed as usize % 4];
    let positions = [0, 7, 8, 0x7f, 0xff, 0x1ff, 0xfff8, 0xffff];
    write_le_u16(&mut state.ram, LINK_X_COORD, positions[seed as usize % 8]);
    write_le_u16(&mut state.ram, LINK_Y_COORD, positions[(seed / 8) as usize]);
    write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x1ff);
    for (i, byte) in state.ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + 0x2000]
        .iter_mut()
        .enumerate()
    {
        *byte = [0, 1, 0x10, 0x11, 0x12, 0x13][(i + seed as usize) % 6];
    }
    state.sync_native_game_state_from_ram();
    state.game_state.enhanced_features = Default::default();
    state
}

#[test]
fn slope_checks_reconsider_the_second_axis_after_the_first() {
    let mut changed_eligibility = 0;
    for seed in 0..64 {
        for flags in [0, 0x10, 0x20, 0x30] {
            for order in [
                CollisionOrder::VerticalFirst,
                CollisionOrder::HorizontalFirst,
            ] {
                let mut expected = collision_test_state(seed, 10);
                expected
                    .follower_link_state_mut()
                    .set_moving_against_diag_tile(flags);
                let mut actual = expected.clone();
                let (first, second, first_mask, second_mask) = match order {
                    CollisionOrder::VerticalFirst => (
                        CollisionAxis::Vertical,
                        CollisionAxis::Horizontal,
                        0x20,
                        0x10,
                    ),
                    CollisionOrder::HorizontalFirst => (
                        CollisionAxis::Horizontal,
                        CollisionAxis::Vertical,
                        0x10,
                        0x20,
                    ),
                };
                // Frozen two-if dispatch from player.rs at 869fdb0c.
                if expected
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile()
                    & first_mask
                    == 0
                {
                    expected.start_movement_collision_checks(first);
                }
                let after_first = expected
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile();
                changed_eligibility += usize::from((flags ^ after_first) & second_mask != 0);
                if after_first & second_mask == 0 {
                    expected.start_movement_collision_checks(second);
                }
                actual.run_slope_collision_checks(order);
                assert_eq!(
                    actual.ram, expected.ram,
                    "seed={seed}, flags={flags}, order={order:?}"
                );
                assert_eq!(actual.game_state.player, expected.game_state.player);
            }
        }
    }
    assert!(
        changed_eligibility > 0,
        "exercise a slope that changes whether the second axis may run"
    );
}
