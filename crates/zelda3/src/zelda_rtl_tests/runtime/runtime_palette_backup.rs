use super::*;
use crate::game_state::constants::MAPBAK_PALETTE;
use crate::game_state::PaletteSliceSource;

#[test]
fn palette_backup_survives_the_master_projection() {
    let mut state = ZeldaState::new();
    let palette: Vec<u8> = (0..512u32).map(|i| (i * 7 + 3) as u8).collect();
    state.backup_overworld_palette_from_tagged(&palette, PaletteSliceSource::Unannotated);
    assert_eq!(
        &state.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 512],
        &palette[..]
    );
    state.project_native_game_state_to_ram();
    assert_eq!(
        &state.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 512],
        &palette[..],
        "the projected owner must carry the game-over palette backup"
    );
    assert_eq!(
        state
            .game_state
            .display
            .palette_buffer
            .overworld_palette_backup(),
        &palette[..]
    );
}
