use crate::game_state::constants::{
    DRAW_WORK_FLAGS_HI, DRAW_WORK_POSITION_X, DRAW_WORK_POSITION_Y, DUAL_LAYER_TILE_CACHE,
    HITBOX_WORK_X_OFFSET, HITBOX_WORK_Y_OFFSET, MAZE_GAME_TIMER_HI, MAZE_GAME_TIMER_LO,
    MAZE_GAME_TIMER_SNAPSHOT_HI, MAZE_GAME_TIMER_SNAPSHOT_LO, PRIZE_DROP_CYCLE,
};
use crate::types::{read_le_u16, write_le_u16};

const SPRITE_SLOT_COUNT: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteState {
    pub(crate) maze_game_timer: MazeGameTimerState,
    pub(crate) prize_drop_cycle: PrizeDropCycleState,
    pub(crate) dual_layer_tile_cache: DualLayerTileCacheState,
    pub(crate) draw_hitbox_work: SpriteDrawHitboxWorkState,
}

impl SpriteState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            maze_game_timer: MazeGameTimerState::load_from_ram(ram),
            prize_drop_cycle: PrizeDropCycleState::load_from_ram(ram),
            dual_layer_tile_cache: DualLayerTileCacheState::load_from_ram(ram),
            draw_hitbox_work: SpriteDrawHitboxWorkState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.maze_game_timer.write_to_ram(ram);
        self.prize_drop_cycle.write_to_ram(ram);
        self.dual_layer_tile_cache.write_to_ram(ram);
        self.draw_hitbox_work.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteDrawHitboxWorkState {
    draw_position_x: u8,
    draw_position_y: u8,
    hitbox_y_offset: u8,
    draw_flags_or_hitbox_x_offset: u8,
}

impl SpriteDrawHitboxWorkState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        debug_assert_eq!(DRAW_WORK_FLAGS_HI, HITBOX_WORK_X_OFFSET);
        Self {
            draw_position_x: ram.get(DRAW_WORK_POSITION_X).copied().unwrap_or(0),
            draw_position_y: ram.get(DRAW_WORK_POSITION_Y).copied().unwrap_or(0),
            hitbox_y_offset: ram.get(HITBOX_WORK_Y_OFFSET).copied().unwrap_or(0),
            draw_flags_or_hitbox_x_offset: ram.get(DRAW_WORK_FLAGS_HI).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        debug_assert_eq!(DRAW_WORK_FLAGS_HI, HITBOX_WORK_X_OFFSET);
        ram[DRAW_WORK_POSITION_X] = self.draw_position_x;
        ram[DRAW_WORK_POSITION_Y] = self.draw_position_y;
        ram[HITBOX_WORK_Y_OFFSET] = self.hitbox_y_offset;
        ram[DRAW_WORK_FLAGS_HI] = self.draw_flags_or_hitbox_x_offset;
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.draw_position_x
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.draw_position_y
    }

    pub(crate) fn low_position_word(&self) -> u16 {
        u16::from(self.draw_position_x) | (u16::from(self.draw_position_y) << 8)
    }

    pub(crate) fn hitbox_x_high_offset(&self) -> u8 {
        self.draw_flags_or_hitbox_x_offset
    }

    pub(crate) fn hitbox_y_low_offset(&self) -> u8 {
        self.hitbox_y_offset
    }
}

pub(crate) struct SpriteDrawWorkPositionView<'a> {
    state: &'a SpriteDrawHitboxWorkState,
}

impl<'a> SpriteDrawWorkPositionView<'a> {
    pub(crate) fn new(state: &'a SpriteDrawHitboxWorkState) -> Self {
        Self { state }
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.state.x_low()
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.state.y_low()
    }

    pub(crate) fn low_position_word(&self) -> u16 {
        self.state.low_position_word()
    }
}

pub(crate) struct NativeSpriteDrawWorkPositionBridgeMut<'a> {
    state: &'a mut SpriteDrawHitboxWorkState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteDrawWorkPositionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteDrawHitboxWorkState, ram: &'a mut [u8]) -> Self {
        *state = SpriteDrawHitboxWorkState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_low_position(&mut self, x: u8, y: u8) {
        self.state.draw_position_x = x;
        self.state.draw_position_y = y;
        self.sync();
    }

    pub(crate) fn set_low_position_word(&mut self, value: u16) {
        self.state.draw_position_x = value as u8;
        self.state.draw_position_y = (value >> 8) as u8;
        self.sync();
    }

    pub(crate) fn set_word_bytes(&mut self, low: u8, high: u8) {
        self.set_low_position(low, high);
    }

    pub(crate) fn offset_low_position(&mut self, dx: u8, dy: u8) -> (u8, u8) {
        self.state.draw_position_x = self.state.draw_position_x.wrapping_add(dx);
        self.state.draw_position_y = self.state.draw_position_y.wrapping_add(dy);
        let position = (self.state.draw_position_x, self.state.draw_position_y);
        self.sync();
        position
    }

    pub(crate) fn set_flags_high(&mut self, value: u8) {
        self.state.draw_flags_or_hitbox_x_offset = value;
        self.sync();
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            SpriteDrawHitboxWorkState::load_from_ram(self.ram)
        );
    }
}

pub(crate) struct SpriteHitboxWorkOffsetView<'a> {
    state: &'a SpriteDrawHitboxWorkState,
}

impl<'a> SpriteHitboxWorkOffsetView<'a> {
    pub(crate) fn new(state: &'a SpriteDrawHitboxWorkState) -> Self {
        Self { state }
    }

    pub(crate) fn x_high_offset(&self) -> u8 {
        self.state.hitbox_x_high_offset()
    }

    pub(crate) fn y_low_offset(&self) -> u8 {
        self.state.hitbox_y_low_offset()
    }
}

pub(crate) struct NativeSpriteHitboxWorkOffsetBridgeMut<'a> {
    state: &'a mut SpriteDrawHitboxWorkState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteHitboxWorkOffsetBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteDrawHitboxWorkState, ram: &'a mut [u8]) -> Self {
        *state = SpriteDrawHitboxWorkState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_x_high_offset(&mut self, value: u8) {
        self.state.draw_flags_or_hitbox_x_offset = value;
        self.sync();
    }

    pub(crate) fn set_y_low_offset(&mut self, value: u8) {
        self.state.hitbox_y_offset = value;
        self.sync();
    }

    pub(crate) fn set_offsets(&mut self, y_low: u8, x_high: u8) {
        self.state.hitbox_y_offset = y_low;
        self.state.draw_flags_or_hitbox_x_offset = x_high;
        self.sync();
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            SpriteDrawHitboxWorkState::load_from_ram(self.ram)
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DualLayerTileCacheState {
    tile_types: [u8; SPRITE_SLOT_COUNT],
}

impl DualLayerTileCacheState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut tile_types = [0; SPRITE_SLOT_COUNT];
        for (slot, tile_type) in tile_types.iter_mut().enumerate() {
            *tile_type = ram.get(DUAL_LAYER_TILE_CACHE + slot).copied().unwrap_or(0);
        }
        Self { tile_types }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, tile_type) in self.tile_types.iter().copied().enumerate() {
            ram[DUAL_LAYER_TILE_CACHE + slot] = tile_type;
        }
    }

    pub(crate) fn tile_type(&self, slot: usize) -> u8 {
        self.tile_types.get(slot).copied().unwrap_or(0)
    }
}

pub(crate) struct DualLayerTileCacheView<'a> {
    state: &'a DualLayerTileCacheState,
}

impl<'a> DualLayerTileCacheView<'a> {
    pub(crate) fn new(state: &'a DualLayerTileCacheState) -> Self {
        Self { state }
    }

    pub(crate) fn tile_type(&self, slot: usize) -> u8 {
        self.state.tile_type(slot)
    }
}

pub(crate) struct NativeDualLayerTileCacheBridgeMut<'a> {
    state: &'a mut DualLayerTileCacheState,
    ram: &'a mut [u8],
}

impl<'a> NativeDualLayerTileCacheBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DualLayerTileCacheState, ram: &'a mut [u8]) -> Self {
        *state = DualLayerTileCacheState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_tile_type(&mut self, slot: usize, value: u8) {
        if let Some(tile_type) = self.state.tile_types.get_mut(slot) {
            *tile_type = value;
            self.sync();
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            DualLayerTileCacheState::load_from_ram(self.ram)
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PrizeDropCycleState {
    next_indices: [u8; SPRITE_SLOT_COUNT],
}

impl PrizeDropCycleState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut next_indices = [0; SPRITE_SLOT_COUNT];
        for (slot, index) in next_indices.iter_mut().enumerate() {
            *index = ram.get(PRIZE_DROP_CYCLE + slot).copied().unwrap_or(0);
        }
        Self { next_indices }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (slot, index) in self.next_indices.iter().copied().enumerate() {
            ram[PRIZE_DROP_CYCLE + slot] = index;
        }
    }

    pub(crate) fn next_index_for_slot(&self, slot: usize) -> u8 {
        self.next_indices.get(slot).copied().unwrap_or(0)
    }
}

pub(crate) struct NativePrizeDropCycleBridgeMut<'a> {
    state: &'a mut PrizeDropCycleState,
    ram: &'a mut [u8],
}

impl<'a> NativePrizeDropCycleBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PrizeDropCycleState, ram: &'a mut [u8]) -> Self {
        *state = PrizeDropCycleState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn take_next_index(&mut self, slot: usize) -> u8 {
        let Some(index) = self.state.next_indices.get_mut(slot) else {
            return 0;
        };
        let current = *index;
        *index = current.wrapping_add(1) & 7;
        self.sync();
        current
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, PrizeDropCycleState::load_from_ram(self.ram));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct MazeGameTimerState {
    elapsed_low: u16,
    elapsed_high: u16,
    snapshot_low: u16,
    snapshot_high: u16,
}

impl MazeGameTimerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            elapsed_low: read_le_u16(ram, MAZE_GAME_TIMER_LO),
            elapsed_high: read_le_u16(ram, MAZE_GAME_TIMER_HI),
            snapshot_low: read_le_u16(ram, MAZE_GAME_TIMER_SNAPSHOT_LO),
            snapshot_high: read_le_u16(ram, MAZE_GAME_TIMER_SNAPSHOT_HI),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, MAZE_GAME_TIMER_LO, self.elapsed_low);
        write_le_u16(ram, MAZE_GAME_TIMER_HI, self.elapsed_high);
        write_le_u16(ram, MAZE_GAME_TIMER_SNAPSHOT_LO, self.snapshot_low);
        write_le_u16(ram, MAZE_GAME_TIMER_SNAPSHOT_HI, self.snapshot_high);
    }

    pub(crate) fn elapsed_low(&self) -> u16 {
        self.elapsed_low
    }

    pub(crate) fn elapsed_high(&self) -> u16 {
        self.elapsed_high
    }

    pub(crate) fn snapshot_low(&self) -> u16 {
        self.snapshot_low
    }
}

pub(crate) struct MazeGameTimerView<'a> {
    state: &'a MazeGameTimerState,
}

impl<'a> MazeGameTimerView<'a> {
    pub(crate) fn new(state: &'a MazeGameTimerState) -> Self {
        Self { state }
    }

    pub(crate) fn elapsed_low(&self) -> u16 {
        self.state.elapsed_low()
    }

    pub(crate) fn elapsed_high(&self) -> u16 {
        self.state.elapsed_high()
    }

    pub(crate) fn snapshot_low(&self) -> u16 {
        self.state.snapshot_low()
    }
}

pub(crate) struct NativeMazeGameTimerBridgeMut<'a> {
    state: &'a mut MazeGameTimerState,
    ram: &'a mut [u8],
}

impl<'a> NativeMazeGameTimerBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut MazeGameTimerState, ram: &'a mut [u8]) -> Self {
        *state = MazeGameTimerState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn clear_elapsed(&mut self) {
        self.state.elapsed_low = 0;
        self.state.elapsed_high = 0;
        self.sync();
    }

    pub(crate) fn increment_elapsed_low(&mut self) -> u16 {
        self.state.elapsed_low = self.state.elapsed_low.wrapping_add(1);
        self.sync();
        self.state.elapsed_low
    }

    pub(crate) fn increment_elapsed_high(&mut self) -> u16 {
        self.state.elapsed_high = self.state.elapsed_high.wrapping_add(1);
        self.sync();
        self.state.elapsed_high
    }

    pub(crate) fn capture_snapshot(&mut self) {
        self.state.snapshot_low = self.state.elapsed_low;
        self.state.snapshot_high = self.state.elapsed_high;
        self.sync();
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, MazeGameTimerState::load_from_ram(self.ram));
    }
}
