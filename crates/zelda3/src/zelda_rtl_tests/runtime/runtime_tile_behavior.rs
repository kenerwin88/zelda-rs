use super::*;
use crate::game_state::constants::{
    CHEAT_WALK_THROUGH_WALLS, DUNGEON_BG2_ATTR_TABLE, DUNG_CHEST_LOCATIONS,
    DUNG_SAVEGAME_STATE_BITS, FLAG_BLOCK_LINK_MENU, PLAYER_ON_SOMARIA_PLATFORM,
};
use sha2::{Digest, Sha256};

#[test]
fn tile_attributes_match_frozen_runtime_effects() {
    let mut cases = String::new();
    for indoors in [false, true] {
        for tile in 0..=u8::MAX {
            let mut digest = Sha256::new();
            for context in 0..32u32 {
                let mut state = tile_behavior_test_state(context);
                let initial_ram = state.ram.clone();
                let initial_native = state.game_state.clone();
                for bits in [0, 1, 2, 4, 8, 0x0f, 0xf0, 0xffff] {
                    state.ram.copy_from_slice(&initial_ram);
                    state.game_state = initial_native.clone();
                    state.tile_detect_execute_inner(tile, 0x100, bits, indoors);
                    digest.update(&state.ram);
                    let mut projected = vec![0xa5; WRAM_SIZE];
                    state.game_state.write_to_ram(&mut projected);
                    digest.update(&projected);
                }
            }
            cases.push_str(&format!("{indoors}/{tile:02x}: {:x}\n", digest.finalize()));
        }
    }
    for seed in 0..32 {
        let mut state = tile_behavior_test_state(seed);
        state.tile_detect_reset_state();
        let mut digest = Sha256::new();
        digest.update(&state.ram);
        let mut projected = vec![0xa5; WRAM_SIZE];
        state.game_state.write_to_ram(&mut projected);
        digest.update(&projected);
        cases.push_str(&format!("reset/{seed}: {:x}\n", digest.finalize()));
    }
    assert_eq!(
        cases,
        include_str!("../../../testdata/tile-behavior-7ad23477.txt")
    );
}

fn tile_behavior_test_state(context: u32) -> ZeldaState {
    let mut state = ZeldaState::new();
    let mut rng = context + 1;
    for byte in &mut state.ram {
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        *byte = (rng >> 24) as u8;
    }
    state.ram[CHEAT_WALK_THROUGH_WALLS] = u8::from(context & 16 != 0);
    state.ram[DUNGEON_BG2_ATTR_TABLE + 0x140] = if context & 8 != 0 { 0x60 } else { 0 };
    for chest in 0..6 {
        write_le_u16(
            &mut state.ram,
            DUNG_CHEST_LOCATIONS + chest * 2,
            if context & 8 != 0 { 0x8000 } else { 0x7fff },
        );
    }
    state.ram[FLAG_BLOCK_LINK_MENU] = u8::from(context & 1 != 0);
    state.ram[PLAYER_ON_SOMARIA_PLATFORM] = u8::from(context & 4 != 0);
    write_le_u16(
        &mut state.ram,
        DUNG_SAVEGAME_STATE_BITS,
        if context & 2 != 0 { 0x8000 } else { 0 },
    );
    state.sync_native_game_state_from_ram();
    state
}
