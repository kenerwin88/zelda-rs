use super::*;

/// `Sprite_SetSpawnedCoordinates` copies the parent's x, y, and z from the spawn record
/// into the child. The port's per-module spawn adapters returned only x and y, and their
/// coordinate adapters rebuilt a record with z zero, so every child spawned through them
/// landed on the floor regardless of the parent's height.
#[test]
fn spawned_foliage_inherits_the_parent_height() {
    let mut state = ZeldaState::new();
    let parent = 3;
    {
        let mut sprite = state.sprite_slot_view_mut(parent);
        sprite.set_state(9);
        sprite.set_x(0x0345);
        sprite.set_y(0x0678);
        sprite.set_z(5);
    }

    state.bush_guard_spawn_foliage(parent);

    let child = (0..16)
        .find(|&j| j != parent && state.sprite_slot_view(j).state() != 0)
        .expect("the foliage spawned");
    assert_eq!(state.sprite_slot_view(child).x(), 0x0345);
    assert_eq!(state.sprite_slot_view(child).y(), 0x0678);
    assert_eq!(state.sprite_slot_view(child).z(), 5);
}
