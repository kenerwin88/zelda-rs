use super::*;
use crate::game_state::constants::{
    FLAG_TRAVEL_BIRD, HUD_CUR_ITEM_X, OVERWORLD_HOLE_SCAN_STEP, TM_COPY, WHICH_ENTRANCE,
};

#[test]
fn hud_item_slot_survives_the_master_projection() {
    let mut state = ZeldaState::new();
    state.save_progress_mut().set_hud_current_item_slot(1, 0x2b);
    assert_eq!(state.ram[HUD_CUR_ITEM_X], 0x2b);
    state.project_native_game_state_to_ram();
    assert_eq!(
        state.ram[HUD_CUR_ITEM_X], 0x2b,
        "no second owner may re-stamp the save progress HUD slot"
    );
    assert_eq!(
        state
            .game_state
            .inventory
            .save_progress
            .hud_current_item_slot(1),
        0x2b
    );
}

#[test]
fn travel_bird_and_layer_masks_have_one_owner() {
    let mut state = ZeldaState::new();
    state.set_travel_bird_tile_offset(0x12);
    state.set_layer_masks_word(0x1716);
    state.project_native_game_state_to_ram();
    assert_eq!(state.ram[FLAG_TRAVEL_BIRD], 0x12);
    assert_eq!(read_le_u16(&state.ram, TM_COPY), 0x1716);

    // The exit backups keep the display's masks and restore them there.
    state.save_exit_tm_copy();
    state.set_layer_masks_word(0);
    state.restore_exit_layer_masks();
    assert_eq!(state.game_state.display.layer_masks_word(), 0x1716);
    assert_eq!(read_le_u16(&state.ram, TM_COPY), 0x1716);
    state.save_spexit_tm_copy();
    state.set_layer_masks_word(0x0101);
    state.restore_spexit_layer_masks();
    assert_eq!(state.game_state.display.layer_masks_word(), 0x1716);
}

#[test]
fn ending_entrance_word_writes_both_owners() {
    let mut state = ZeldaState::new();
    state.set_which_entrance_word(0x0342);
    assert_eq!(state.ram[WHICH_ENTRANCE], 0x42);
    assert_eq!(state.ram[OVERWORLD_HOLE_SCAN_STEP], 0x03);
    assert_eq!(state.game_state.world.region.which_entrance(), 0x42);
    assert_eq!(
        state.game_state.world.transient.overworld_hole_scan_step,
        0x03
    );
    state.project_native_game_state_to_ram();
    assert_eq!(state.ram[WHICH_ENTRANCE], 0x42);
    assert_eq!(state.ram[OVERWORLD_HOLE_SCAN_STEP], 0x03);
}
