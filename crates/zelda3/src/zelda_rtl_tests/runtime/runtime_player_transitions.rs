use super::*;
use crate::game_state::constants::{
    ABOUT_TO_JUMP_OFF_LEDGE, BUTTON_MASK_B_Y, LINK_ELECTROCUTE_ON_TOUCH, LINK_IS_ON_LOWER_LEVEL,
    PLAYER_DEFENSE_FLAGS,
};
use sha2::{Digest, Sha256};

#[test]
fn action_reset_updates_native_fields_without_a_repairing_import() {
    use crate::game_state::constants::{LINK_CAPE_MODE, RELATED_TO_HOOKSHOT};
    let mut state = ZeldaState::new();
    for address in [
        LINK_ELECTROCUTE_ON_TOUCH,
        LINK_CAPE_MODE,
        RELATED_TO_HOOKSHOT,
    ] {
        state.ram[address] = 0xff;
    }
    state.follower_link_state_mut().reset_action_state();
    for address in [
        LINK_ELECTROCUTE_ON_TOUCH,
        LINK_CAPE_MODE,
        RELATED_TO_HOOKSHOT,
    ] {
        assert_eq!(state.ram[address], 0, "reset C's original byte store");
    }
    assert_eq!(
        state.game_state.player.follower_link,
        crate::game_state::FollowerLinkState::load_from_ram(&state.ram),
        "the transition must model every player byte it publishes"
    );
}

#[test]
fn player_transitions_match_frozen_runtime_effects() {
    let mut cases = String::new();
    for seed in 0..32u32 {
        for transition in 0..9 {
            let mut state = ZeldaState::new();
            let mut rng = seed + 1;
            for byte in &mut state.ram {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                *byte = (rng >> 24) as u8;
            }
            state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 0;
            state.ram[LINK_IS_ON_LOWER_LEVEL] = (seed % 3) as u8;
            state.sync_native_game_state_from_ram();
            state.game_state.enhanced_features = Default::default();
            // Legacy writers can leave native fields stale at transition entry.
            if seed & 1 != 0 {
                state.ram[PLAYER_DEFENSE_FLAGS] ^= 0xa5;
                state.ram[BUTTON_MASK_B_Y] ^= 0x81;
                state.ram[LINK_ELECTROCUTE_ON_TOUCH] ^= 0x5a;
            }
            match transition {
                0 => state.finish_recoil_landing(),
                1 => state.link_reset_sword_and_item_usage(),
                2 => state.link_initialize(),
                3 => state.link_reset_properties_a(),
                4 => state.link_reset_properties_b(),
                5 => state.link_reset_properties_c(),
                6 => state.link_reset_swimming_state(),
                7 => state.reset_all_acceleration(),
                8 => state.link_force_unequip_cape_quietly(),
                _ => unreachable!(),
            }
            let mut digest = Sha256::new();
            digest.update(&state.ram);
            let mut projected = vec![0xa5; WRAM_SIZE];
            state.game_state.write_to_ram(&mut projected);
            digest.update(&projected);
            cases.push_str(&format!("{seed}/{transition}: {:x}\n", digest.finalize()));
        }
    }
    assert_eq!(
        cases,
        include_str!("../../../testdata/player-transitions-de9235e8.txt")
    );
}
