use super::*;
use crate::game_state::constants::BOSS_PRIZE_GRAPHICS_COUNTDOWN;

/// The boss room tag starts the prize countdown and the prize ancilla counts it down
/// mid-frame. Room-effects setters used to re-project a stale second copy of the byte
/// over the live count; the world transient is now the only owner.
#[test]
fn room_effects_setters_do_not_restamp_the_boss_prize_countdown() {
    let mut state = ZeldaState::new();

    state.world_transient_mut().begin_boss_prize_graphics_countdown();
    assert_eq!(state.ram[BOSS_PRIZE_GRAPHICS_COUNTDOWN], 128);
    assert_eq!(
        state.game_state.world.transient.boss_prize_graphics_countdown(),
        128
    );

    state
        .world_transient_mut()
        .decrement_boss_prize_graphics_countdown();
    assert_eq!(state.ram[BOSS_PRIZE_GRAPHICS_COUNTDOWN], 127);

    // A room-effects mutation in the same frame re-projects that state's bytes.
    state
        .dungeon_room_effects_mut()
        .set_moving_wall_dot_pointer(1);
    assert_eq!(state.ram[BOSS_PRIZE_GRAPHICS_COUNTDOWN], 127);

    // The master projection publishes the owner's value, not a stale copy.
    state.project_native_game_state_to_ram();
    assert_eq!(state.ram[BOSS_PRIZE_GRAPHICS_COUNTDOWN], 127);
    assert_eq!(
        state.game_state.world.transient.boss_prize_graphics_countdown(),
        127
    );
}
