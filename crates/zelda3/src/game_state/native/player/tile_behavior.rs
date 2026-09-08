//! Attribute decoding and native probe-result accumulation. Publication stays
//! at the tile-detection bridge; no additional result state is stored here.

use super::{NativeTileDetectionBridgeMut, TileDetectionState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileResult {
    Collision,
    Normal,
    DeepWater,
    ShallowWater,
    MovingFloor,
    Ice,
    WaterStaircase,
    Grass,
    Stair,
    InRoomStaircase,
    Pit,
    Misc,
    Slope,
    Diagonal,
    VerticalLedge,
    HorizontalLedge,
    DownwardSideLedge,
    DiagonalLedge,
    SpikeTrigger,
    SpikeCactus,
    Aftermath,
    Readable,
    Dashable,
    Chest,
    KeyLockGravestone,
    Block,
}

impl TileDetectionState {
    fn accumulate_result(&mut self, result: TileResult, bits: u16) {
        use TileResult::*;
        match result {
            Collision => self.collision_bits |= bits,
            Normal => self.normal_tiles |= bits,
            DeepWater => self.deepwater |= bits,
            ShallowWater => self.shallow_water |= bits,
            MovingFloor => self.moving_floor_tiles |= bits,
            Ice => self.icy_floor |= bits,
            WaterStaircase => self.water_staircase |= bits,
            Grass => self.thick_grass |= bits,
            Stair => self.stair_tile |= bits as u8,
            InRoomStaircase => self.inroom_staircase |= bits,
            Pit => self.or_pit_tile(bits as u8),
            Misc => self.misc_tiles |= bits,
            Slope => self.slope_collision_bits |= bits,
            Diagonal => self.diagonal_tile |= bits,
            VerticalLedge => self.vertical_ledge |= bits as u8,
            HorizontalLedge => self.horizontal_ledge |= bits as u8,
            DownwardSideLedge => self.ledges_down_leftright |= bits as u8,
            DiagonalLedge => self.diagonal_ledge_tiles |= bits as u8,
            SpikeTrigger => self.spike_floor_and_triggers |= bits as u8,
            SpikeCactus => self.or_spike_cactus_tiles(bits as u8),
            Aftermath => self.destruction_aftermath |= bits,
            Readable => self.read_something |= bits,
            Dashable => self.dashable_tiles |= bits as u8,
            Chest => self.chest |= bits,
            KeyLockGravestone => self.or_key_lock_gravestones(bits as u8),
            Block => self.block_flags |= bits,
        }
    }
}

impl NativeTileDetectionBridgeMut<'_> {
    pub(crate) fn accumulate_result(&mut self, result: TileResult, bits: u16) {
        self.state.accumulate_result(result, bits);
        self.sync();
    }

    pub(crate) fn reset_probe_results(&mut self) {
        // Retain each intermediate projection, including the shared key-lock/spike
        // word's two clears. Scratch positions and interaction identity survive.
        self.clear_slope_collision_bits();
        self.clear_collision_bits();
        self.clear_diagonal_tile();
        self.clear_stair_tile();
        self.clear_pit_tile();
        self.clear_inroom_staircase();
        self.clear_block_flags();
        self.clear_door_direction_flags();
        self.clear_moving_floor_tiles();
        self.clear_deepwater();
        self.clear_normal_tiles();
        self.clear_icy_floor();
        self.clear_water_staircase();
        self.clear_thick_grass();
        self.clear_shallow_water();
        self.clear_destruction_aftermath();
        self.clear_read_something();
        self.clear_vertical_ledge();
        self.clear_horizontal_ledge();
        self.clear_ledges_down_leftright();
        self.clear_diagonal_ledge_tiles();
        self.clear_chest();
        self.clear_key_lock_gravestones();
        self.clear_spike_cactus_tiles();
        self.clear_spike_floor_and_triggers();
        self.clear_dashable_tiles();
        self.clear_misc_tiles();
    }
}
