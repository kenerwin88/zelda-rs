use super::*;

#[test]
fn ppu_side_space_reads_the_room_bounds_owner() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.set_indoor_flag(0);
    state.ppu_scroll_copy_mut().set_bg2_h_copy2(0x0210);
    state.ppu_scroll_copy_mut().set_bg2_v_copy2(0x0420);
    state
        .room_bounds_mut()
        .set_packed_bounds(0x0400, 0x0428, 0x0200, 0x0218);
    state.configure_ppu_side_space();
    assert_eq!(state.ppu.extra_left_cur, 0x10);
    assert_eq!(state.ppu.extra_right_cur, 0x08);
    assert_eq!(state.ppu.extra_bottom_cur, 0x08);
}
