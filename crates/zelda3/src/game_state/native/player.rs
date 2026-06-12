use super::ram_byte;
use crate::game_state::constants::{
    LINK_X_COORD, LINK_X_COORD_SPEXIT, LINK_Y_COORD, LINK_Y_COORD_SPEXIT, SWIM_ACCELERATION,
    SWIM_ACCELERATION_DIRECTION, SWIM_ACCELERATION_MODE, SWIM_MAX_SPEED, SWIM_SPEED_ACTIVE_FLAG,
};
use crate::types::{read_le_u16, write_le_u16};

const SWIM_AXIS_COUNT: usize = 2;

fn swim_axis_index(offset: usize) -> Option<usize> {
    match offset {
        0 => Some(0),
        2 => Some(1),
        _ => None,
    }
}

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
    pub(crate) swim_acceleration: SwimAccelerationState,
}

impl PlayerState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            special_exit_position: SpecialExitPositionState::load_from_ram(ram),
            swim_acceleration: SwimAccelerationState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.special_exit_position.write_to_ram(ram);
        self.swim_acceleration.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SwimAccelerationState {
    mode: [u16; SWIM_AXIS_COUNT],
    speed_active_flag: [u16; SWIM_AXIS_COUNT],
    max_speed: [u16; SWIM_AXIS_COUNT],
    acceleration_direction: [u16; SWIM_AXIS_COUNT],
    acceleration: [u16; SWIM_AXIS_COUNT],
}

impl SwimAccelerationState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            mode: read_axis_words(ram, SWIM_ACCELERATION_MODE),
            speed_active_flag: read_axis_words(ram, SWIM_SPEED_ACTIVE_FLAG),
            max_speed: read_axis_words(ram, SWIM_MAX_SPEED),
            acceleration_direction: read_axis_words(ram, SWIM_ACCELERATION_DIRECTION),
            acceleration: read_axis_words(ram, SWIM_ACCELERATION),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_axis_words(ram, SWIM_ACCELERATION_MODE, self.mode);
        write_axis_words(ram, SWIM_SPEED_ACTIVE_FLAG, self.speed_active_flag);
        write_axis_words(ram, SWIM_MAX_SPEED, self.max_speed);
        write_axis_words(
            ram,
            SWIM_ACCELERATION_DIRECTION,
            self.acceleration_direction,
        );
        write_axis_words(ram, SWIM_ACCELERATION, self.acceleration);
    }

    pub(crate) fn mode(&self, offset: usize) -> u16 {
        axis_word(self.mode, offset)
    }

    pub(crate) fn mode_low(&self, axis: usize) -> u8 {
        self.mode.get(axis).copied().unwrap_or(0) as u8
    }

    pub(crate) fn speed_active_flag(&self, offset: usize) -> u16 {
        axis_word(self.speed_active_flag, offset)
    }

    pub(crate) fn max_speed(&self, offset: usize) -> u16 {
        axis_word(self.max_speed, offset)
    }

    pub(crate) fn acceleration_direction(&self, offset: usize) -> u16 {
        axis_word(self.acceleration_direction, offset)
    }

    pub(crate) fn acceleration(&self, offset: usize) -> u16 {
        axis_word(self.acceleration, offset)
    }

    pub(crate) fn has_any_acceleration(&self) -> bool {
        self.acceleration[0] | self.acceleration[1] != 0
    }
}

fn read_axis_words(ram: &[u8], base: usize) -> [u16; SWIM_AXIS_COUNT] {
    [
        if base + 1 < ram.len() {
            read_le_u16(ram, base)
        } else {
            0
        },
        if base + 3 < ram.len() {
            read_le_u16(ram, base + 2)
        } else {
            0
        },
    ]
}

fn write_axis_words(ram: &mut [u8], base: usize, values: [u16; SWIM_AXIS_COUNT]) {
    write_le_u16(ram, base, values[0]);
    write_le_u16(ram, base + 2, values[1]);
}

fn axis_word(values: [u16; SWIM_AXIS_COUNT], offset: usize) -> u16 {
    swim_axis_index(offset)
        .and_then(|axis| values.get(axis).copied())
        .unwrap_or(0)
}

pub(crate) struct SwimAccelerationView<'a> {
    state: &'a SwimAccelerationState,
}

impl<'a> SwimAccelerationView<'a> {
    pub(crate) fn new(state: &'a SwimAccelerationState) -> Self {
        Self { state }
    }

    pub(crate) fn mode(&self, offset: usize) -> u16 {
        self.state.mode(offset)
    }

    pub(crate) fn mode_low(&self, axis: usize) -> u8 {
        self.state.mode_low(axis)
    }

    pub(crate) fn speed_active_flag(&self, offset: usize) -> u16 {
        self.state.speed_active_flag(offset)
    }

    pub(crate) fn max_speed(&self, offset: usize) -> u16 {
        self.state.max_speed(offset)
    }

    pub(crate) fn acceleration_direction(&self, offset: usize) -> u16 {
        self.state.acceleration_direction(offset)
    }

    pub(crate) fn acceleration(&self, offset: usize) -> u16 {
        self.state.acceleration(offset)
    }

    pub(crate) fn has_any_acceleration(&self) -> bool {
        self.state.has_any_acceleration()
    }
}

pub(crate) struct NativeSwimAccelerationBridgeMut<'a> {
    state: &'a mut SwimAccelerationState,
    ram: &'a mut [u8],
}

impl<'a> NativeSwimAccelerationBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SwimAccelerationState, ram: &'a mut [u8]) -> Self {
        *state = SwimAccelerationState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SwimAccelerationState::load_from_ram(self.ram));
    }

    pub(crate) fn set_mode(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.mode[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn clear_mode_low_axis(&mut self) {
        self.state.mode[0] = 0;
        self.sync();
    }

    pub(crate) fn set_speed_active_flag(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.speed_active_flag[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_max_speed(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.max_speed[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_max_speed_both_axes(&mut self, value: u16) {
        self.state.max_speed = [value; SWIM_AXIS_COUNT];
        self.sync();
    }

    pub(crate) fn set_acceleration_direction(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.acceleration_direction[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn set_acceleration(&mut self, offset: usize, value: u16) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.acceleration[axis] = value;
            self.sync();
        }
    }

    pub(crate) fn clear_axis_motion(&mut self, offset: usize) {
        if let Some(axis) = swim_axis_index(offset) {
            self.state.speed_active_flag[axis] = 0;
            self.state.mode[axis] = 0;
            self.state.acceleration[axis] = 0;
            self.state.max_speed[axis] = 0;
            self.sync();
        }
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
