use super::ram_byte;
use crate::game_state::constants::{
    LINK_X_COORD, LINK_X_COORD_SPEXIT, LINK_Y_COORD, LINK_Y_COORD_SPEXIT,
};
use crate::types::{read_le_u16, write_le_u16};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpecialExitPositionState {
    x: u16,
    y: u16,
}

impl SpecialExitPositionState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x: if LINK_X_COORD_SPEXIT + 1 < ram.len() {
                read_le_u16(ram, LINK_X_COORD_SPEXIT)
            } else {
                0
            },
            y: if LINK_Y_COORD_SPEXIT + 1 < ram.len() {
                read_le_u16(ram, LINK_Y_COORD_SPEXIT)
            } else {
                0
            },
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_le_u16(ram, LINK_X_COORD_SPEXIT, self.x);
        write_le_u16(ram, LINK_Y_COORD_SPEXIT, self.y);
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn map_zoom_y(&self) -> u16 {
        ((self.y >> 4).wrapping_sub(0x48)) & !1
    }

    pub(crate) fn map_zoom_x_offset(&self) -> u16 {
        (self.x >> 4).wrapping_sub(0x80)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlayerState {
    pub(crate) special_exit_position: SpecialExitPositionState,
}

impl PlayerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            special_exit_position: SpecialExitPositionState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.special_exit_position.write_to_ram(ram);
    }
}

pub(crate) struct SpecialExitPositionView<'a> {
    state: &'a SpecialExitPositionState,
}

impl<'a> SpecialExitPositionView<'a> {
    pub(crate) fn new(state: &'a SpecialExitPositionState) -> Self {
        Self { state }
    }

    pub(crate) fn x(&self) -> u16 {
        self.state.x()
    }

    pub(crate) fn y(&self) -> u16 {
        self.state.y()
    }

    pub(crate) fn map_zoom_y(&self) -> u16 {
        self.state.map_zoom_y()
    }

    pub(crate) fn map_zoom_x_offset(&self) -> u16 {
        self.state.map_zoom_x_offset()
    }
}

pub(crate) struct NativeSpecialExitPositionBridgeMut<'a> {
    state: &'a mut SpecialExitPositionState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpecialExitPositionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpecialExitPositionState, ram: &'a mut [u8]) -> Self {
        *state = SpecialExitPositionState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            SpecialExitPositionState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.x = value;
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.y = value;
        self.sync();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.x = x;
        self.state.y = y;
        self.sync();
    }

    pub(crate) fn offset_position(&mut self, x_delta: u16, y_delta: u16) {
        self.state.x = self.state.x.wrapping_add(x_delta);
        self.state.y = self.state.y.wrapping_add(y_delta);
        self.sync();
    }

    pub(crate) fn store_from_player(&mut self) {
        self.state.x = u16::from(ram_byte(self.ram, LINK_X_COORD))
            | (u16::from(ram_byte(self.ram, LINK_X_COORD + 1)) << 8);
        self.state.y = u16::from(ram_byte(self.ram, LINK_Y_COORD))
            | (u16::from(ram_byte(self.ram, LINK_Y_COORD + 1)) << 8);
        self.sync();
    }

    pub(crate) fn restore_player_position(&mut self) {
        write_le_u16(self.ram, LINK_X_COORD, self.state.x);
        write_le_u16(self.ram, LINK_Y_COORD, self.state.y);
    }
}
