use super::*;
use crate::game_state::constants::{BLAST_WALL_CENTER_X, BLAST_WALL_CENTER_Y, BLAST_WALL_DIRECTION};
use crate::types::read_le_u16;

/// The blast-wall room tag stores the direction and center into the dialogue buffer
/// words the entrance-effects state owns. The room-effects state used to hold a second
/// copy whose projection was gated on the wall being open, so the trigger's values never
/// reached RAM through it and the code wrote raw RAM and reloaded the owner instead.
#[test]
fn blast_wall_trigger_writes_reach_the_entrance_effects_owner() {
    let mut state = ZeldaState::new();
    state.blast_wall_scratch_mut().set_direction(4);
    state.blast_wall_scratch_mut().set_center(0x0123, 0x0045);

    assert_eq!(state.ram[BLAST_WALL_DIRECTION], 4);
    assert_eq!(read_le_u16(&state.ram, BLAST_WALL_CENTER_X), 0x0123);
    assert_eq!(read_le_u16(&state.ram, BLAST_WALL_CENTER_Y), 0x0045);
    assert_eq!(
        state.game_state.effects.entrance_effects.blast_wall_direction(),
        4
    );

    // An unrelated room-effects setter in the same frame re-projects that state's bytes.
    state
        .dungeon_room_effects_mut()
        .set_moving_wall_dot_pointer(1);
    assert_eq!(state.ram[BLAST_WALL_DIRECTION], 4);
    assert_eq!(read_le_u16(&state.ram, BLAST_WALL_CENTER_X), 0x0123);
    assert_eq!(read_le_u16(&state.ram, BLAST_WALL_CENTER_Y), 0x0045);
}
