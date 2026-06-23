use super::*;

#[test]
fn dash_dust_motive_expires_out_of_range_frame() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
    state.ancilla_slot_view_mut(0).set_timer(1);
    state.ancilla_slot_view_mut(0).set_item_to_link(3);

    state.dash_dust_motive(0);

    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
}

#[test]
fn sword_wall_hit_does_not_write_terminal_animation_counter() {
    let mut state = ZeldaState::new();
    state.ancilla_slot_view_mut(4).set_ancilla_type(0x1b);
    state.ancilla_slot_view_mut(4).set_item_to_link(7);
    state.ancilla_slot_view_mut(4).set_aux_timer(0);

    state.ancilla_sword_wall_hit(4);

    let slot = state.ancilla_slot_view(4);
    assert_eq!(slot.ancilla_type(), 0);
    assert_eq!(slot.item_to_link(), 7);
    assert_eq!(slot.aux_timer(), 0xff);
}
