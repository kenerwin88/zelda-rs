use super::*;

const BOMB: u8 = 0x07;
const FALLING_PRIZE: u8 = 0x29;

fn prize_slot(state: &ZeldaState) -> Option<usize> {
    (0..crate::game_state::ANCILLA_SLOT_COUNT)
        .find(|&k| state.ancilla_slot_view(k).ancilla_type() == FALLING_PRIZE)
}

/// The original room tag disarms itself even when the prize ancilla cannot be
/// allocated, so a boss killed with five non-evictable ancillae alive never drops
/// its heart container. The decompilation keeps the tag armed and retries; the
/// port inherits that fix.
#[test]
fn boss_prize_tag_retries_until_an_ancilla_slot_frees() {
    let mut state = ZeldaState::new();
    state
        .dungeon_savegame_state_mut()
        .set_savegame_state_high_bits(0x80);
    state.dungeon_header_mut().set_header_tag(0, 0x27);
    for k in 0..5 {
        state.ancilla_slot_view_mut(k).set_ancilla_type(BOMB);
    }

    state.RoomTag_GetHeartForPrize(0);
    assert_eq!(prize_slot(&state), None);
    assert_ne!(state.game_state.dungeon.header.header_tag(0), 0);

    state.ancilla_slot_view_mut(2).set_ancilla_type(0);
    state.RoomTag_GetHeartForPrize(0);
    assert_eq!(prize_slot(&state), Some(2));
    assert_eq!(state.game_state.dungeon.header.header_tag(0), 0);
}
