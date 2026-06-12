use crate::game_state::constants::{
    MAZE_GAME_TIMER_HI, MAZE_GAME_TIMER_LO, MAZE_GAME_TIMER_SNAPSHOT_HI,
    MAZE_GAME_TIMER_SNAPSHOT_LO,
};
use crate::types::{read_le_u16, write_le_u16};

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteState {
    pub(crate) maze_game_timer: MazeGameTimerState,
}

impl SpriteState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            maze_game_timer: MazeGameTimerState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.maze_game_timer.write_to_ram(ram);
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
