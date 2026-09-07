//! Offline editor snapshots, compiled only with `map-preview`. This module
//! never receives a live game, runs frames, or reads/writes saves. It uses the
//! existing room, graphics, palette and attribute producers in a fresh state.
use super::ZeldaState;

pub struct DungeonSnapshot {
    pub room: u16,
    pub bg1: Vec<u16>,
    pub bg2: Vec<u16>,
    pub bg1_attributes: Vec<u8>,
    pub bg2_attributes: Vec<u8>,
    pub vram: Vec<u16>,
    pub palette: Vec<u16>,
}

/// Draw the entrance's initial room with unopened chests/doors and no saved
/// progress. This is an editor construction snapshot, never a gameplay receipt.
pub fn dungeon_snapshot(
    rom: &[u8],
    assets: &[u8],
    entrance: u8,
) -> Result<DungeonSnapshot, String> {
    let mut game = ZeldaState::new();
    game.set_rom(rom);
    game.set_assets(assets)?;
    game.parity_probe_dungeon_load_and_draw(u16::from(entrance));
    game.palette_load_dungeon_set();
    game.init_load_default_tile_attr();
    game.Dungeon_LoadCustomTileAttr();
    game.Dungeon_LoadAttributeTable();
    let tiles = &game.game_state.dungeon.room_tilemaps;
    let attributes = &game.game_state.dungeon.bg2_attributes;
    Ok(DungeonSnapshot {
        room: game.game_state.world.location.dungeon_room(),
        bg1: (0..4096).map(|i| tiles.bg1_tile(i)).collect(),
        bg2: (0..4096).map(|i| tiles.bg2_tile(i)).collect(),
        bg1_attributes: (0..4096).map(|i| attributes.bg1_attr(i)).collect(),
        bg2_attributes: (0..4096).map(|i| attributes.bg2_attr(i)).collect(),
        vram: game.ppu.vram.to_vec(),
        // The loader fills the target (auxiliary) palette; the live game fades
        // it into main over time. An unfaded editor view uses the target colors.
        palette: (0..256)
            .map(|i| game.game_state.display.palette_buffer.aux_color(i))
            .collect(),
    })
}
