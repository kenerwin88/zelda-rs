use super::*;
use crate::game_state::constants::WATER_HDMA_WINDOW_Y_RADIUS;
use crate::types::read_le_u16;

/// The swamp drain and the dam flood share the water HDMA window words. The dungeon
/// environment used to hold a second copy that its bridge re-projected on every
/// unrelated setter; the display's water window is the only owner now.
#[test]
fn environment_setters_do_not_restamp_the_water_hdma_window() {
    let mut state = ZeldaState::new();
    state.water_hdma_window_mut().set_window_y_radius(0x30);
    assert_eq!(
        state.game_state.display.water_hdma_window.window_y_radius(),
        0x30
    );
    assert_eq!(read_le_u16(&state.ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x30);

    // An unrelated dungeon environment mutation in the same frame.
    state.dungeon_environment_mut().set_trapdoors_down(1);
    assert_eq!(read_le_u16(&state.ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x30);

    state.project_native_game_state_to_ram();
    assert_eq!(read_le_u16(&state.ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x30);
    assert_eq!(
        state.game_state.display.water_hdma_window.window_y_radius(),
        0x30
    );
}
