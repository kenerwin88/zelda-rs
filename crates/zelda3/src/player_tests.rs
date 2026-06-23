use super::*;
use crate::game_state::constants::{
    ANCILLA_TYPE, BUTTON_B_FRAMES, LINK_DELAY_TIMER_SPIN_ATTACK, LINK_HANDLER_STATE,
    LINK_IS_BUNNY_MIRROR, LINK_IS_IN_DEEP_WATER,
};

#[test]
fn link_splash_upon_landing_suppresses_recoil_other_deep_water_splash() {
    let mut state = ZeldaState::new();
    state.ram[LINK_HANDLER_STATE] = 6;
    state.ram[LINK_IS_IN_DEEP_WATER] = 1;
    state.ram[LINK_IS_BUNNY_MIRROR] = 0;
    state.sync_native_game_state_from_ram();

    state.link_splash_upon_landing();

    assert_eq!(state.ram[LINK_HANDLER_STATE], 4);
    assert_eq!(state.ram[ANCILLA_TYPE + 4], 0);
}

#[test]
fn hookshot_missing_ancilla_preserves_underflowed_spin_timer() {
    let mut state = ZeldaState::new();
    state.ram[LINK_HANDLER_STATE] = 19;
    state.ram[LINK_DELAY_TIMER_SPIN_ATTACK] = 0;
    state.ram[BUTTON_B_FRAMES] = 10;
    state.sync_native_game_state_from_ram();

    state.link_state_hookshotting();

    assert_eq!(state.ram[LINK_HANDLER_STATE], 0);
    assert_eq!(state.ram[LINK_DELAY_TIMER_SPIN_ATTACK], 0xff);
    assert_eq!(state.ram[BUTTON_B_FRAMES], 9);
}
