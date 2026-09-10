use super::*;
use crate::game_state::constants::DOOR_ANIMATION_STEP_INDICATOR;
use crate::types::read_le_u16;

/// The original has one door animation step (0x690) that the dungeon doors and the
/// overworld's big entrance doors share. The port kept a second copy in the world
/// transient, so a step set through the dungeon door owner was invisible to the
/// overworld's big-door module until the next full import.
#[test]
fn overworld_big_door_reads_the_dungeon_door_owner() {
    let mut state = ZeldaState::new();
    state.dungeon_doors_mut().set_door_animation_step(3);
    assert_eq!(read_le_u16(&state.ram, DOOR_ANIMATION_STEP_INDICATOR), 3);

    state.Module09_09_OpenBigDoorFromExiting();

    // Step 3 completes the door: the module arms the walk-out countdown and advances.
    assert_eq!(state.game_state.frame.submodule, 1);
}
