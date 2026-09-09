use super::*;
use crate::game_state::constants::{
    AUX_TILE_THEME_INDEX, MAIN_TILE_THEME_INDEX, OVERWORLD_TILE_THEME_INDEX, SPRITE_GRAPHICS_INDEX,
};

#[test]
fn overworld_restore_reads_the_entrance_backup_owner() {
    let mut state = ZeldaState::new();
    state
        .world_palette_theme_mut()
        .set_overworld_tile_theme_index(0x21);
    state
        .world_palette_theme_mut()
        .set_main_tile_theme_index(0x22);
    state
        .world_palette_theme_mut()
        .set_aux_tile_theme_index(0x23);
    state.sprite_system_mut().set_graphics_index(0x24);
    // Dungeon_LoadEntrance saves the live themes through the backup owner.
    state.dungeon_entrance_backup_mut().cache_exit_tile_themes();
    // The dungeon then changes the live themes without any full re-import.
    state
        .world_palette_theme_mut()
        .set_overworld_tile_theme_index(0x31);
    state
        .world_palette_theme_mut()
        .set_main_tile_theme_index(0x32);
    state
        .world_palette_theme_mut()
        .set_aux_tile_theme_index(0x33);
    state.sprite_system_mut().set_graphics_index(0x34);

    state.LoadCachedEntranceProperties();

    assert_eq!(state.ram[OVERWORLD_TILE_THEME_INDEX], 0x21);
    assert_eq!(state.ram[MAIN_TILE_THEME_INDEX], 0x22);
    assert_eq!(state.ram[AUX_TILE_THEME_INDEX], 0x23);
    assert_eq!(state.ram[SPRITE_GRAPHICS_INDEX], 0x24);
    assert_eq!(
        state.game_state.world.palette_theme.main_tile_theme_index(),
        0x22
    );
    assert_eq!(
        state.game_state.world.palette_theme.aux_tile_theme_index(),
        0x23
    );
    assert_eq!(state.game_state.sprites.system.graphics_index(), 0x24);
}
