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
            // The frozen projection predates ownership of the item index's
            // high byte. Check it explicitly, retaining every old hash byte.
            let high = crate::game_state::constants::HUD_CUR_ITEM_HIGH;
            assert_eq!(projected[high], state.ram[high]);
            projected[high] = 0xa5;
            digest.update(&projected);
            cases.push_str(&format!("{seed}/{operation}: {:x}\n", digest.finalize()));
        }
    }
    assert_eq!(
        cases,
        include_str!("../../../testdata/player-collision-zero-page-scratch.txt")
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

#[test]
fn native_completed_vertical_axis_handoff_runs_only_the_remaining_horizontal_axis() {
    use crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail;
    let mut duplicate_axis_changes = 0;
    for seed in 0..64 {
        let mut expected = collision_test_state(seed, 10);
        let mut actual = expected.clone();
        let mut duplicate = expected.clone();
        expected.start_movement_collision_checks(CollisionAxis::Horizontal);
        expected.finish_native_cardinal_collision(CardinalCollisionTail::MainHandler);
        actual.finish_native_cardinal_collision(CardinalCollisionTail::SlopeChecks {
            primary_layer: false,
            horizontal_first: false,
            vertical_completed: true,
        });
        assert_eq!(
            actual.ram, expected.ram,
            "completed vertical axis seed={seed}"
        );
        assert_eq!(actual.game_state.player, expected.game_state.player);
        duplicate.run_slope_collision_checks(CollisionOrder::VerticalFirst);
        duplicate.finish_native_cardinal_collision(CardinalCollisionTail::MainHandler);
        duplicate_axis_changes += usize::from(duplicate.ram != expected.ram);
    }
    assert!(
        duplicate_axis_changes > 0,
        "exercise a state where repeating the vertical axis changes the result"
    );
}

#[test]
fn native_completed_main_tile_probe_preserves_classification_and_reset_neighbors() {
    use crate::cycle_models::{main_tile_probe::MainTileHandoff,tile_classifier::TileClassifierTail};
    for seed in 0..64 {
        let mut state=collision_test_state(seed,4);
        // Item five has no response body. Its completed native classifier
        // must survive the semantic return without another reset or lookup.
        state.early_main_tile_probe=Some(MainTileHandoff {item:5,x:seed as u16,y:0,classifier:TileClassifierTail::Complete});
        let before=state.ram.clone();
        let mut projected_before=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_before);
        state.tile_detect_main_handler(5);
        assert!(state.early_main_tile_probe.is_none());
        assert_eq!(state.ram,before,"seed={seed}");
        let mut projected_after=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_after);
        assert_eq!(projected_after,projected_before,"seed={seed}");
    }
}

#[test]
fn native_completed_main_tile_response_and_cardinal_return_do_not_replay_effects() {
    use crate::cycle_models::{main_tile_probe::MainTileHandoff,main_tile_response::MainTileResponseTail,tile_classifier::TileClassifierTail,cardinal_collision_prefix::CardinalCollisionTail};
    for seed in 0..64 {
        let mut state=collision_test_state(seed,4);
        state.early_main_tile_probe=Some(MainTileHandoff {item:0,x:seed as u16,y:0,classifier:TileClassifierTail::Complete});
        state.early_main_tile_response=Some(MainTileResponseTail::Returned);
        let before=state.ram.clone();
        let mut projected_before=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_before);
        state.finish_native_cardinal_collision(CardinalCollisionTail::MainHandler);
        assert!(state.early_main_tile_probe.is_none());
        assert!(state.early_main_tile_response.is_none());
        assert_eq!(state.ram,before,"seed={seed}");
        let mut projected_after=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_after);
        assert_eq!(projected_after,projected_before,"seed={seed}");
    }
}

#[test]
fn native_ground_animation_return_skips_animation_receipt_and_completed_cleanup() {
    use crate::cycle_models::{ground_velocity::GroundVelocityTail,cardinal_collision_prefix::CardinalCollisionTail,main_tile_response::MainTileResponseTail,main_tile_probe::MainTileHandoff,tile_classifier::TileClassifierTail};
    for seed in 0..64 {
        let mut state=collision_test_state(seed,4);
        let address=crate::game_state::constants::ENHANCED_FEATURE_FLAGS;
        state.ram[address..address+4].copy_from_slice(&4096u32.to_le_bytes());
        state.sync_native_game_state_from_ram();
        state.apply_links_movement_to_camera_called=true;
        state.early_ground_velocity_tail=Some(GroundVelocityTail::Complete);
        state.early_cardinal_collision_tail=Some(CardinalCollisionTail::MainHandler);
        state.early_main_tile_probe=Some(MainTileHandoff {item:0,x:0,y:0,classifier:TileClassifierTail::Complete});
        state.early_main_tile_response=Some(MainTileResponseTail::Returned);
        state.early_ground_animation_return=true;
        let before=state.ram.clone();
        let mut projected_before=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_before);
        let cycles_before=crate::cycle_ledger::master();
        state.finish_ground_movement_after_diagonal(true);
        assert!(!state.early_ground_animation_return);
        assert_eq!(state.ram,before,"seed={seed}");
        let mut projected_after=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_after);
        assert_eq!(projected_after,projected_before,"seed={seed}");
        assert_eq!(crate::cycle_ledger::master(),cycles_before,"no animation timing receipt seed={seed}");
    }
}

#[test]
fn native_ground_camera_return_skips_owned_camera_and_ground_effects() {
    use crate::cycle_models::{ground_velocity::GroundVelocityTail,cardinal_collision_prefix::CardinalCollisionTail,main_tile_response::MainTileResponseTail,main_tile_probe::MainTileHandoff,tile_classifier::TileClassifierTail};
    for seed in 0..64 {
        let mut state=collision_test_state(seed,4);
        state.ram[0x1b]=1;
        state.ram[0x6c]=0;
        state.sync_native_game_state_from_ram();
        state.apply_links_movement_to_camera_called=false;
        state.early_ground_camera_tail=Some(crate::cycle_models::ground_camera_return::GroundCameraTail::Returned);
        state.early_ground_velocity_tail=Some(GroundVelocityTail::Complete);
        state.early_cardinal_collision_tail=Some(CardinalCollisionTail::MainHandler);
        state.early_main_tile_probe=Some(MainTileHandoff {item:0,x:0,y:0,classifier:TileClassifierTail::Complete});
        state.early_main_tile_response=Some(MainTileResponseTail::Returned);
        state.early_ground_animation_return=true;
        let before=state.ram.clone();
        let mut projected_before=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_before);
        let cycles_before=crate::cycle_ledger::master();
        state.finish_ground_movement_after_diagonal(true);
        assert!(!state.early_ground_animation_return);
        assert!(state.early_ground_camera_tail.is_none());
        assert!(state.apply_links_movement_to_camera_called);
        assert_eq!(state.ram,before,"seed={seed}");
        let mut projected_after=vec![0xa5;WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected_after);
        assert_eq!(projected_after,projected_before,"seed={seed}");
        assert_eq!(crate::cycle_ledger::master(),cycles_before,"no animation timing receipt seed={seed}");
    }
}

#[test]
fn native_player_ancilla_return_preserves_completed_scan_without_receipts() {
    for seed in 0..64 {
        let mut state=collision_test_state(seed,4);
        state.early_player_ancilla_tail=Some(crate::cycle_models::player_ancilla_return::PlayerAncillaTail::Returned);
        let before=state.ram.clone();let cycles=crate::cycle_ledger::master();
        let mut typed=vec![0xa5;WRAM_SIZE];state.game_state.write_to_ram(&mut typed);
        state.handle_somaria_and_graves();
        assert!(state.early_player_ancilla_tail.is_none());assert_eq!(state.ram,before);
        let mut after=vec![0xa5;WRAM_SIZE];state.game_state.write_to_ram(&mut after);
        assert_eq!(typed,after);assert_eq!(crate::cycle_ledger::master(),cycles);
    }
}

#[test]
fn native_dungeon_push_caller_return_skips_owned_reset_loop_and_receipts() {
    use crate::cycle_models::dungeon_push_caller_return::PushCallerTail;
    for case in 0..64usize {
        for native_loop in [false,true] {
        let prepare=|| {
            let mut state=ZeldaState::new();
            state.ram[0x10]=7;state.ram[0x11]=15;state.ram[0x1b]=1;
            let end=(case%17*2) as u16;
            state.ram[0x478..0x47a].copy_from_slice(&end.to_le_bytes());
            for i in 0..16 {
                let record=[0u16,3,5,0x1111,0xffff,0x8000][(case+i)%6];
                state.ram[0x500+i*2..0x502+i*2].copy_from_slice(&record.to_le_bytes());
            }
            state.sync_native_game_state_from_ram();state
        };
        let mut expected=prepare();let mut actual=prepare();
        let end=actual.game_state.dungeon.torch.torches_start_index();
        actual.dungeon_object_tracking_mut().set_misc_object_index(if native_loop {end} else {0});
        actual.early_dungeon_push_caller_tail=Some(if native_loop {PushCallerTail::Returned} else {PushCallerTail::Handler});
        let before=crate::cycle_ledger::master();
        expected.complete_module07_dungeon_after_submodule();
        let expected_cycles=crate::cycle_ledger::master()-before;
        let before=crate::cycle_ledger::master();
        actual.complete_module07_dungeon_after_submodule();
        let actual_cycles=crate::cycle_ledger::master()-before;
        assert!(actual.early_dungeon_push_caller_tail.is_none());
        assert_eq!(actual.ram,expected.ram,"case={case}");
        let mut a=vec![0xa5;WRAM_SIZE];let mut b=a.clone();
        actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
        assert_eq!(a,b,"typed case={case}");
        let loop_cycles=(0..usize::from(end/2)).map(|i| {
            if expected.game_state.dungeon.object_tracking.object_record(i)==crate::game_state::ObjectRecord::IDLE_PUSH_BLOCK {316u64} else {448}
        }).sum::<u64>()+184;
        assert_eq!(expected_cycles-actual_cycles,94+if native_loop {loop_cycles} else {0},"owned caller and handler receipts case={case}");
    }
    }
}

#[test]
fn native_active_push_blocks_resume_room_child_without_replaying_prior_blocks_or_motion() {
    use crate::cycle_models::dungeon_push_caller_return::PushCallerTail;
    use crate::game_state::ObjectRecord;
    let mut cases=0;
    for target in [0x02b62e,0x02b6cd,0x02b76e,0x02b81c] {
        for arrived in [false,true] {
            for tile in [0u8,0x20,0x23] {
                for case in 0..8u16 {
                    let prepare=|| {
                        let mut state=ZeldaState::new();
                        state.ram[0x10]=7;state.ram[0x11]=0;state.ram[0x1b]=1;
                        state.ram[0xa0..0xa2].copy_from_slice(&31u16.to_le_bytes());
                        state.ram[0xa9]=u8::from(target==0x02b6cd);
                        state.ram[0xaa]=if target==0x02b81c {2} else {0};
                        state.ram[0x20..0x22].copy_from_slice(&(0x0200+case).to_le_bytes());
                        state.ram[0x22..0x24].copy_from_slice(&(0x0100+case).to_le_bytes());
                        state.ram[0x42c..0x42e].copy_from_slice(&2u16.to_le_bytes());
                        state.ram[0x478..0x47a].copy_from_slice(&6u16.to_le_bytes());
                        // The earlier falling block has already run; replay
                        // would decrement the shared animation timer again.
                        state.ram[0x500..0x502].copy_from_slice(&4u16.to_le_bytes());
                        state.ram[0x502..0x504].copy_from_slice(&(if arrived {3u16} else {2}).to_le_bytes());
                        state.ram[0x504..0x506].fill(0);
                        state.ram[0x542..0x544].copy_from_slice(&0x0100u16.to_le_bytes());
                        state.ram[0x12080]=tile;
                        state.ram[0x5fc]=1;state.ram[0x5fd]=2;
                        state.ram[0x2c3]=0;state.ram[0x2c4]=9;
                        for (i,byte) in state.ram[0x5e0..0x5fc].iter_mut().enumerate() {*byte=(i as u8).wrapping_mul(7).wrapping_add(case as u8);}
                        state.sync_native_game_state_from_ram();state
                    };
                    let mut expected=prepare();let mut actual=prepare();
                    let before=actual.game_state.player.pushed_block;
                    match target {
                        0x02b62e => expected.handle_edge_transition_movement_east_right_by8(),
                        0x02b6cd => expected.dung_start_inter_room_trans_left_plus(),
                        0x02b76e => expected.handle_edge_transition_movement_south_down_by16(),
                        0x02b81c => expected.dungeon_start_inter_room_trans_up(),
                        _ => unreachable!(),
                    }
                    if arrived {
                        expected.push_block_check_for_pit(2);
                        expected.dungeon_object_tracking_mut().advance_object_record(1);
                    }
                    expected.dungeon_object_tracking_mut().set_misc_object_index(4);
                    expected.dungeon_push_block_handler_until(u16::MAX);
                    actual.finish_native_dungeon_push_handler(PushCallerTail::RoomTransition(target));
                    assert_eq!(actual.ram,expected.ram,"target={target:06x} arrived={arrived} tile={tile:02x} case={case}");
                    let mut a=vec![0xa5;WRAM_SIZE];let mut b=a.clone();
                    actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
                    assert_eq!(a,b,"typed target={target:06x} arrived={arrived} tile={tile:02x} case={case}");
                    assert_eq!(actual.game_state.player.pushed_block,before,"native motion or animation replayed target={target:06x}");
                    assert_eq!(actual.game_state.dungeon.object_tracking.object_record(0),ObjectRecord::FALLING);
                    assert_eq!(actual.game_state.dungeon.object_tracking.misc_object_index(),6);
                    cases+=1;
                }
            }
        }
    }
    println!("native active push blocks: {cases} room-child continuations, all four directions, sliding/arrived states, pit/plate/floor arrivals, no prior-block or motion replay");
}

#[test]
fn native_player_ancilla_return_resumes_selected_child_and_remaining_slots() {
    use crate::cycle_models::player_ancilla_return::PlayerAncillaTail;
    for pattern in 0..243usize {
        for indoors in [0u8,1] {
            let prepare=|| {
                let mut state=ZeldaState::new();
                state.ram[0x11]=(pattern&1) as u8;state.ram[0xfa0]=0xa5;state.ram[0x2f]=4;
                state.ram[0x1b]=indoors;state.ram[0x3e9]=1;
                let mut p=pattern;
                for slot in 0..5 {
                    state.ram[0xc4a+slot]=[0,0x24,0x2c][p%3];p/=3;
                    state.ram[0x394+slot]=1;
                    state.ram[0xbfa+slot]=100;state.ram[0xc04+slot]=100;
                }
                state.sync_native_game_state_from_ram();state
            };
            let mut expected=prepare();let mut actual=prepare();
            let grave=if indoors==0 {(0..5).rev().find(|i|actual.ram[0xc4a+i]==0x24)} else {None};
            let somaria=(0..5).rev().find(|i|actual.ram[0xc4a+i]==0x2c);
            let tail=grave.map(|i|PlayerAncillaTail::Grave(i as u8)).or_else(||somaria.map(|i|PlayerAncillaTail::Somaria(i as u8))).unwrap_or(PlayerAncillaTail::Returned);
            expected.handle_somaria_and_graves();
            actual.early_player_ancilla_tail=Some(tail);actual.handle_somaria_and_graves();
            assert!(actual.early_player_ancilla_tail.is_none());
            assert_eq!(actual.ram,expected.ram,"pattern={pattern} indoors={indoors} tail={tail:?}");
            let mut a=vec![0xa5;WRAM_SIZE];let mut b=a.clone();
            actual.game_state.write_to_ram(&mut a);expected.game_state.write_to_ram(&mut b);
            assert_eq!(a,b,"typed pattern={pattern} indoors={indoors} tail={tail:?}");
        }
    }
}

#[test]
fn native_dungeon_after_push_handoffs_skip_owned_guards_half_slot_and_camera() {
    use crate::cycle_models::dungeon_after_push::DungeonAfterPushTail;
    let mut count = 0;
    for case in 0..32usize {
        for tail in [
            DungeonAfterPushTail::Lamp,
            DungeonAfterPushTail::HalfSlotBody,
            DungeonAfterPushTail::AfterCamera,
        ] {
            let prepare = || {
                let mut state = ZeldaState::new();
                state.ram[0x10] = 7;
                state.ram[0x1b] = 1;
                state.ram[0x11] = if tail == DungeonAfterPushTail::Lamp {
                    15
                } else {
                    0
                };
                crate::cycle_models::dungeon_camera::seed_case(
                    &mut state.ram,
                    [0, 1, 3, 253, 255][case % 5],
                    [255, 253, 3, 1, 0][case % 5],
                    case % 24,
                );
                state.ram[0xaaa] = if tail == DungeonAfterPushTail::HalfSlotBody {
                    19 + (case % 2) as u8
                } else {
                    0
                };
                state.sync_native_game_state_from_ram();
                state
            };
            let mut expected = prepare();
            let mut actual = prepare();
            let owned = match tail {
                DungeonAfterPushTail::MainWait => panic!("after-push fixture stops before the main return"),
                DungeonAfterPushTail::SpritePrefix {..} | DungeonAfterPushTail::AfterDrawing(_) | DungeonAfterPushTail::BeforeDrawing(_) | DungeonAfterPushTail::AfterLamp | DungeonAfterPushTail::RoomTags(_) | DungeonAfterPushTail::Interactions(_) | DungeonAfterPushTail::Edge(_)=>unreachable!(),
                DungeonAfterPushTail::Lamp => 46,
                DungeonAfterPushTail::HalfSlotBody => 40 + 62 + 48,
                DungeonAfterPushTail::AfterCamera => {
                    actual.graphics_load_chr_half_slot();
                    let before = crate::cycle_ledger::master();
                    actual.dungeon_handle_camera();
                    40 + 62 + 98 + 46 + (crate::cycle_ledger::master() - before)
                }
            };
            actual.early_dungeon_after_push_tail = Some(tail);
            let before = crate::cycle_ledger::master();
            expected.complete_module07_dungeon_after_push_block_handler(true);
            let expected_cycles = crate::cycle_ledger::master() - before;
            let before = crate::cycle_ledger::master();
            actual.complete_module07_dungeon_after_push_block_handler(true);
            let actual_cycles = crate::cycle_ledger::master() - before;
            assert!(actual.early_dungeon_after_push_tail.is_none());
            assert_eq!(actual.ram, expected.ram, "case={case} tail={tail:?}");
            let mut a = vec![0xa5; WRAM_SIZE];
            let mut b = a.clone();
            actual.game_state.write_to_ram(&mut a);
            expected.game_state.write_to_ram(&mut b);
            assert_eq!(a, b, "native projection case={case} tail={tail:?}");
            assert_eq!(
                expected_cycles - actual_cycles,
                owned,
                "owned receipts case={case} tail={tail:?}"
            );
            count += 1;
        }
    }
    println!(
        "native dungeon after push semantic handoffs: {count} cases, no owned guards, conversion or camera replay"
    );
}

#[test]
fn native_room_tag_handoffs_skip_owned_probe_and_selected_calls() {
    use crate::cycle_models::{
        dungeon_after_push::DungeonAfterPushTail, dungeon_room_tags::RoomTagTail,
    };
    let mut count = 0;
    for case in 0..32usize {
        for kind in 0..4 {
            let pos = (((0x0100u16) & 0x1f8) << 3) | ((0x0200 & 0x1f8) >> 3);
            let landing = 0x30 + (case % 4) as u8;
            let head = [0x26u8, 0x38, 0x39, 0x5e, 0x5f][case % 5];
            let prepare = || {
                let mut state = ZeldaState::new();
                state.ram[0x10] = 7;
                state.ram[0x11] = 0;
                state.ram[0x1b] = 1;
                state.ram[0xa0..0xa2].copy_from_slice(&0x31u16.to_le_bytes());
                state.ram[0x20..0x24].copy_from_slice(&[0, 1, 0, 2]);
                state.ram[0x0f] = 0x7a;
                state.ram[0x67] = if kind == 3 { 8 } else { 0 };
                state.ram[0x4c7] = if kind == 0 && case & 1 != 0 {
                    case as u8
                } else {
                    0
                };
                state.ram[0xae] = if kind == 1 {
                    0x34 + (case % 4) as u8
                } else {
                    0
                };
                state.ram[0xaf] = if kind == 2 {
                    0x34 + (case % 4) as u8
                } else {
                    0
                };
                if kind == 3 {
                    state.ram[0x12000 + usize::from(pos)] = head;
                    state.ram[0x12000 + usize::from(pos) + 64] = landing;
                    for i in 0..4 {
                        state.ram[0xc001 + i] = 0x40 + i as u8;
                        state.ram[0x63c + i] = 0x30 + ((case + i) % 2 * 4) as u8;
                    }
                }
                state.sync_native_game_state_from_ram();
                state
            };
            let mut expected = prepare();
            let mut actual = prepare();
            actual.graphics_load_chr_half_slot();
            let before = crate::cycle_ledger::master();
            actual.dungeon_handle_camera();
            let camera_cycles = crate::cycle_ledger::master() - before;
            let camera_owned = 40 + 62 + 98 + 46 + camera_cycles;
            let (tail, tag_owned) = match kind {
                0 => {
                    let before = crate::cycle_ledger::master();
                    actual.dungeon_handle_room_tags();
                    let cycles = crate::cycle_ledger::master() - before;
                    (RoomTagTail::Returned, 40 + 62 + cycles)
                }
                1 | 2 => {
                    actual
                        .tile_detect_position_mut()
                        .set_collision_bits_low_byte(if kind == 1 { 0 } else { 1 });
                    (
                        RoomTagTail::Tag {
                            index: if kind == 1 { 0 } else { 1 },
                            target: 0xcc08 + ((case % 4) * 4) as u16,
                        },
                        40 + 62 + 48 + 68 + 164 + 138 + if kind == 2 { 42 + 154 } else { 0 },
                    )
                }
                _ => {
                    actual
                        .tile_detect_position_mut()
                        .set_collision_bits_low_byte(head);
                    (
                        RoomTagTail::Staircase {
                            head,
                            landing: (pos & 0xff00) | u16::from(landing),
                            pos,
                        },
                        40 + 62
                            + 48
                            + 68
                            + 94
                            + 400
                            + 6
                            + 138
                            + 6
                            + 130
                            + match head {
                                0x26 => 6,
                                0x38 => 38,
                                0x39 => 70,
                                0x5e => 102,
                                _ => 128,
                            }
                            + 148
                            + 48
                            + 6,
                    )
                }
            };
            actual.early_dungeon_after_push_tail = Some(DungeonAfterPushTail::RoomTags(tail));
            let before = crate::cycle_ledger::master();
            expected.complete_module07_dungeon_after_push_block_handler(true);
            let expected_cycles = crate::cycle_ledger::master() - before;
            let before = crate::cycle_ledger::master();
            actual.complete_module07_dungeon_after_push_block_handler(true);
            let actual_cycles = crate::cycle_ledger::master() - before;
            let diffs = actual
                .ram
                .iter()
                .zip(expected.ram.iter())
                .enumerate()
                .filter_map(|(i, (a, b))| (a != b).then_some((i, *a, *b)))
                .take(16)
                .collect::<Vec<_>>();
            assert!(
                diffs.is_empty(),
                "case={case} kind={kind} RAM differences {diffs:x?}"
            );
            let mut a = vec![0xa5; WRAM_SIZE];
            let mut b = a.clone();
            actual.game_state.write_to_ram(&mut a);
            expected.game_state.write_to_ram(&mut b);
            assert_eq!(a, b, "native case={case} kind={kind}");
            assert_eq!(
                expected_cycles - actual_cycles,
                camera_owned + tag_owned,
                "owned work case={case} kind={kind}"
            );
            count += 1;
        }
    }
    println!(
        "native room tag semantic handoffs: {count} cases, no probe, first-tag or caller replay"
    );
}
