use super::*;

#[test]
fn pre_overworld_music_selection_preserves_f2_goto_setsong_path() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0xf2, 2, 0, 0x40, 1);

    assert_eq!(xt, 0xf3);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn pre_overworld_music_selection_applies_darkworld_override_without_f2() {
    let (xt, ow_anim_tiles) = pre_overworld_music_selection(0x6c, 0x1c, 0x09, 2, 0, 0x40, 1);

    assert_eq!(xt, 9);
    assert_eq!(ow_anim_tiles, 0x5a);
}

#[test]
fn turtle_rock_vram_common_terminates_nmi_upload_data() {
    let mut state = ZeldaState::new();
    state.write_vram_upload_buffer_byte(6, 0);
    state.ram[UVRAM_DATA_OVERWORLD + 6] = 0;

    state.turtle_rock_vram_common(0x10);

    assert_eq!(state.vram_upload_buffer_byte(6), 0xff);
    assert_eq!(state.ram[UVRAM_DATA_OVERWORLD + 6], 0);
}
