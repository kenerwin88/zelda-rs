use super::*;
use crate::game_state::constants::LINK_ELECTROCUTE_ON_TOUCH;

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

