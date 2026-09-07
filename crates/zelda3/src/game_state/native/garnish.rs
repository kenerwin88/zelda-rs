//! Garnish effect slots and shared runtime state.

use crate::game_state::constants::{
    ACTIVE_OVERLORD_INDEX, GARNISH_ACTIVE, GARNISH_COUNTDOWN, GARNISH_FLOOR, GARNISH_OAM_FLAGS,
    GARNISH_SPRITE, GARNISH_TYPE, GARNISH_X_HI, GARNISH_X_LO, GARNISH_X_SUBPIXEL,
    GARNISH_X_VELOCITY, GARNISH_Y_HI, GARNISH_Y_LO, GARNISH_Y_SUBPIXEL, GARNISH_Y_VELOCITY,
    HAUNTED_GROVE_FLUTE_EVENT_LATCH, OVERWORLD_BOULDER_TRAP_COUNT, OVERWORLD_BOULDER_TRAP_TIMER,
    REPULSESPARK_ANIM_DELAY, REPULSESPARK_FLOOR_STATUS, REPULSESPARK_TIMER, REPULSESPARK_X_LO,
    REPULSESPARK_Y_LO, SPRCOLL_X_BASE, SPRCOLL_X_SIZE, SPRCOLL_Y_BASE, SPRCOLL_Y_SIZE,
};
use crate::types::{read_le_u16, write_le_u16};

const GARNISH_SLOT_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GarnishSlotState {
    garnish_type: u8,
    x_low: u8,
    x_high: u8,
    y_low: u8,
    y_high: u8,
    x_velocity: u8,
    y_velocity: u8,
    x_subpixel: u8,
    y_subpixel: u8,
    countdown: u8,
    sprite: u8,
    floor: u8,
    oam_flags: u8,
}

impl GarnishSlotState {
    fn load_from_ram(ram: &[u8], slot: usize) -> Self {
        Self {
            garnish_type: ram.get(GARNISH_TYPE + slot).copied().unwrap_or(0),
            x_low: ram.get(GARNISH_X_LO + slot).copied().unwrap_or(0),
            x_high: ram.get(GARNISH_X_HI + slot).copied().unwrap_or(0),
            y_low: ram.get(GARNISH_Y_LO + slot).copied().unwrap_or(0),
            y_high: ram.get(GARNISH_Y_HI + slot).copied().unwrap_or(0),
            x_velocity: ram.get(GARNISH_X_VELOCITY + slot).copied().unwrap_or(0),
            y_velocity: ram.get(GARNISH_Y_VELOCITY + slot).copied().unwrap_or(0),
            x_subpixel: ram.get(GARNISH_X_SUBPIXEL + slot).copied().unwrap_or(0),
            y_subpixel: ram.get(GARNISH_Y_SUBPIXEL + slot).copied().unwrap_or(0),
            countdown: ram.get(GARNISH_COUNTDOWN + slot).copied().unwrap_or(0),
            sprite: ram.get(GARNISH_SPRITE + slot).copied().unwrap_or(0),
            floor: ram.get(GARNISH_FLOOR + slot).copied().unwrap_or(0),
            oam_flags: ram.get(GARNISH_OAM_FLAGS + slot).copied().unwrap_or(0),
        }
    }

    fn write_to_ram(&self, ram: &mut [u8], slot: usize) {
        ram[GARNISH_TYPE + slot] = self.garnish_type;
        ram[GARNISH_X_LO + slot] = self.x_low;
        ram[GARNISH_X_HI + slot] = self.x_high;
        ram[GARNISH_Y_LO + slot] = self.y_low;
        ram[GARNISH_Y_HI + slot] = self.y_high;
        ram[GARNISH_X_VELOCITY + slot] = self.x_velocity;
        ram[GARNISH_Y_VELOCITY + slot] = self.y_velocity;
        ram[GARNISH_X_SUBPIXEL + slot] = self.x_subpixel;
        ram[GARNISH_Y_SUBPIXEL + slot] = self.y_subpixel;
        ram[GARNISH_COUNTDOWN + slot] = self.countdown;
        crate::types::ww_check(
            GARNISH_SPRITE + slot,
            1,
            "GarnishSlotState::write_to_ram.sprite",
            self.sprite as u32,
        );
        ram[GARNISH_SPRITE + slot] = self.sprite;
        ram[GARNISH_FLOOR + slot] = self.floor;
        ram[GARNISH_OAM_FLAGS + slot] = self.oam_flags;
    }

    pub(crate) fn garnish_type(&self) -> u8 {
        self.garnish_type
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.garnish_type == 0
    }

    pub(crate) fn x(&self) -> u16 {
        u16::from(self.x_low) | (u16::from(self.x_high) << 8)
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.x_low
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.x_high
    }

    pub(crate) fn y(&self) -> u16 {
        u16::from(self.y_low) | (u16::from(self.y_high) << 8)
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.y_low
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.y_high
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.x_velocity
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.y_velocity
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.x_subpixel
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.y_subpixel
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.countdown
    }

    pub(crate) fn sprite(&self) -> u8 {
        self.sprite
    }

    pub(crate) fn floor(&self) -> u8 {
        self.floor
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        self.oam_flags
    }

    pub(crate) fn set_garnish_type(&mut self, value: u8) {
        self.garnish_type = value;
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.x_low = value as u8;
        self.x_high = (value >> 8) as u8;
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.x_low = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.x_high = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.y_low = value as u8;
        self.y_high = (value >> 8) as u8;
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.y_low = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.y_high = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.x_velocity = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.y_velocity = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.x_subpixel = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.y_subpixel = value;
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.countdown = value;
    }

    pub(crate) fn set_sprite(&mut self, value: u8) {
        self.sprite = value;
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.floor = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.oam_flags = value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GarnishSlotsState {
    slots: [GarnishSlotState; GARNISH_SLOT_COUNT],
}

impl GarnishSlotsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..GARNISH_SLOT_COUNT {
            state.slots[slot] = GarnishSlotState::load_from_ram(ram, slot);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for slot in 0..GARNISH_SLOT_COUNT {
            self.slots[slot].write_to_ram(ram, slot);
        }
    }

    pub(crate) fn slot(&self, slot: usize) -> NativeGarnishSlotView<'_> {
        NativeGarnishSlotView {
            state: &self.slots[slot],
        }
    }

    pub(crate) fn slot_mut<'a>(
        &'a mut self,
        ram: &'a mut [u8],
        slot: usize,
    ) -> NativeGarnishSlotBridgeMut<'a> {
        NativeGarnishSlotBridgeMut {
            state: &mut self.slots[slot],
            ram,
            slot,
        }
    }
}

pub(crate) struct NativeGarnishSlotView<'a> {
    state: &'a GarnishSlotState,
}

impl<'a> NativeGarnishSlotView<'a> {
    pub(crate) fn garnish_type(&self) -> u8 {
        self.state.garnish_type()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub(crate) fn x(&self) -> u16 {
        self.state.x()
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.state.x_low()
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.state.x_high()
    }

    pub(crate) fn y(&self) -> u16 {
        self.state.y()
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.state.y_low()
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.state.y_high()
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.state.x_velocity()
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.state.y_velocity()
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.state.x_subpixel()
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.state.y_subpixel()
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.state.countdown()
    }

    pub(crate) fn sprite(&self) -> u8 {
        self.state.sprite()
    }

    pub(crate) fn floor(&self) -> u8 {
        self.state.floor()
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        self.state.oam_flags()
    }
}

pub(crate) struct NativeGarnishSlotBridgeMut<'a> {
    state: &'a mut GarnishSlotState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeGarnishSlotBridgeMut<'a> {
    fn sync(&mut self) {
        self.state.write_to_ram(self.ram, self.slot);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            GarnishSlotState::load_from_ram(self.ram, self.slot)
        );
    }

    pub(crate) fn set_garnish_type(&mut self, value: u8) {
        self.state.set_garnish_type(value);
        self.sync();
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state.set_x(value);
        self.sync();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.state.set_x_low(value);
        self.sync();
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.state.set_x_high(value);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state.set_y(value);
        self.sync();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.state.set_y_low(value);
        self.sync();
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.state.set_y_high(value);
        self.sync();
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.state.set_x_velocity(value);
        self.sync();
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.state.set_y_velocity(value);
        self.sync();
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.state.set_x_subpixel(value);
        self.sync();
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.state.set_y_subpixel(value);
        self.sync();
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.state.set_countdown(value);
        self.sync();
    }

    pub(crate) fn set_sprite(&mut self, value: u8) {
        self.state.set_sprite(value);
        self.sync();
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.state.set_floor(value);
        self.sync();
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.state.set_oam_flags(value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GarnishRuntimeState {
    active_type: u8,
    boulder_trap_count: u8,
    boulder_trap_timer: u8,
    sprite_collision_x_size: u16,
    sprite_collision_y_size: u16,
    sprite_collision_x_base: u16,
    sprite_collision_y_base: u16,
    active_overlord_index: u8,
    haunted_grove_flute_event_latch: u8,
    repulsespark_timer: u8,
    repulsespark_anim_delay: u8,
    repulsespark_floor_status: u8,
    repulsespark_x_low: u8,
    repulsespark_y_low: u8,
}

impl GarnishRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            active_type: ram.get(GARNISH_ACTIVE).copied().unwrap_or(0),
            boulder_trap_count: ram.get(OVERWORLD_BOULDER_TRAP_COUNT).copied().unwrap_or(0),
            boulder_trap_timer: ram.get(OVERWORLD_BOULDER_TRAP_TIMER).copied().unwrap_or(0),
            sprite_collision_x_size: read_le_u16(ram, SPRCOLL_X_SIZE),
            sprite_collision_y_size: read_le_u16(ram, SPRCOLL_Y_SIZE),
            sprite_collision_x_base: read_le_u16(ram, SPRCOLL_X_BASE),
            sprite_collision_y_base: read_le_u16(ram, SPRCOLL_Y_BASE),
            active_overlord_index: ram.get(ACTIVE_OVERLORD_INDEX).copied().unwrap_or(0),
            haunted_grove_flute_event_latch: ram
                .get(HAUNTED_GROVE_FLUTE_EVENT_LATCH)
                .copied()
                .unwrap_or(0),
            repulsespark_timer: ram.get(REPULSESPARK_TIMER).copied().unwrap_or(0),
            repulsespark_anim_delay: ram.get(REPULSESPARK_ANIM_DELAY).copied().unwrap_or(0),
            repulsespark_floor_status: ram.get(REPULSESPARK_FLOOR_STATUS).copied().unwrap_or(0),
            repulsespark_x_low: ram.get(REPULSESPARK_X_LO).copied().unwrap_or(0),
            repulsespark_y_low: ram.get(REPULSESPARK_Y_LO).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[GARNISH_ACTIVE] = self.active_type;
        ram[OVERWORLD_BOULDER_TRAP_COUNT] = self.boulder_trap_count;
        ram[OVERWORLD_BOULDER_TRAP_TIMER] = self.boulder_trap_timer;
        write_le_u16(ram, SPRCOLL_X_SIZE, self.sprite_collision_x_size);
        write_le_u16(ram, SPRCOLL_Y_SIZE, self.sprite_collision_y_size);
        write_le_u16(ram, SPRCOLL_X_BASE, self.sprite_collision_x_base);
        write_le_u16(ram, SPRCOLL_Y_BASE, self.sprite_collision_y_base);
        ram[ACTIVE_OVERLORD_INDEX] = self.active_overlord_index;
        ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = self.haunted_grove_flute_event_latch;
        ram[REPULSESPARK_TIMER] = self.repulsespark_timer;
        ram[REPULSESPARK_ANIM_DELAY] = self.repulsespark_anim_delay;
        ram[REPULSESPARK_FLOOR_STATUS] = self.repulsespark_floor_status;
        ram[REPULSESPARK_X_LO] = self.repulsespark_x_low;
        ram[REPULSESPARK_Y_LO] = self.repulsespark_y_low;
    }

    pub(crate) fn active_type(&self) -> u8 {
        self.active_type
    }

    pub(crate) fn boulder_trap_count(&self) -> u8 {
        self.boulder_trap_count
    }

    pub(crate) fn boulder_trap_timer(&self) -> u8 {
        self.boulder_trap_timer
    }

    pub(crate) fn sprcoll_y_hi(&self) -> u8 {
        (self.sprite_collision_y_base >> 8) as u8
    }

    pub(crate) fn sprcoll_x_word(&self) -> u16 {
        self.sprite_collision_x_base
    }

    pub(crate) fn sprcoll_y_word(&self) -> u16 {
        self.sprite_collision_y_base
    }

    pub(crate) fn active_overlord_index(&self) -> u8 {
        self.active_overlord_index
    }

    pub(crate) fn haunted_grove_flute_event_latch(&self) -> u8 {
        self.haunted_grove_flute_event_latch
    }

    pub(crate) fn repulsespark_timer(&self) -> u8 {
        self.repulsespark_timer
    }

    pub(crate) fn repulsespark_anim_delay(&self) -> u8 {
        self.repulsespark_anim_delay
    }

    pub(crate) fn repulsespark_floor_status(&self) -> u8 {
        self.repulsespark_floor_status
    }

    pub(crate) fn repulsespark_x_lo(&self) -> u8 {
        self.repulsespark_x_low
    }

    pub(crate) fn repulsespark_y_lo(&self) -> u8 {
        self.repulsespark_y_low
    }

    pub(crate) fn sprcoll_x_size(&self) -> u16 {
        self.sprite_collision_x_size
    }

    pub(crate) fn sprcoll_y_size(&self) -> u16 {
        self.sprite_collision_y_size
    }

    pub(crate) fn set_active_type(&mut self, value: u8) {
        self.active_type = value;
    }

    pub(crate) fn clear_active_type(&mut self) {
        self.active_type = 0;
    }

    pub(crate) fn increment_boulder_trap_timer(&mut self) -> u8 {
        self.boulder_trap_timer = self.boulder_trap_timer.wrapping_add(1);
        self.boulder_trap_timer
    }

    pub(crate) fn set_active_overlord_index(&mut self, value: u8) {
        self.active_overlord_index = value;
    }

    pub(crate) fn increment_haunted_grove_flute_event_latch(&mut self) {
        self.haunted_grove_flute_event_latch = self.haunted_grove_flute_event_latch.wrapping_add(1);
    }

    pub(crate) fn set_repulsespark_timer(&mut self, value: u8) {
        self.repulsespark_timer = value;
    }

    pub(crate) fn clear_repulsespark_timer(&mut self) {
        self.repulsespark_timer = 0;
    }

    pub(crate) fn decrement_repulsespark_timer(&mut self) {
        self.repulsespark_timer = self.repulsespark_timer.wrapping_sub(1);
    }

    pub(crate) fn set_repulsespark_anim_delay(&mut self, value: u8) {
        self.repulsespark_anim_delay = value;
    }

    pub(crate) fn decrement_repulsespark_anim_delay(&mut self) -> u8 {
        self.repulsespark_anim_delay = self.repulsespark_anim_delay.wrapping_sub(1);
        self.repulsespark_anim_delay
    }

    pub(crate) fn set_repulsespark_x_lo(&mut self, value: u8) {
        self.repulsespark_x_low = value;
    }

    pub(crate) fn set_repulsespark_y_lo(&mut self, value: u8) {
        self.repulsespark_y_low = value;
    }

    pub(crate) fn set_sprcoll_x_size(&mut self, value: u16) {
        self.sprite_collision_x_size = value;
    }

    pub(crate) fn set_sprcoll_y_size(&mut self, value: u16) {
        self.sprite_collision_y_size = value;
    }

    pub(crate) fn set_sprcoll_x_base(&mut self, value: u16) {
        self.sprite_collision_x_base = value;
    }

    pub(crate) fn set_sprcoll_y_base(&mut self, value: u16) {
        self.sprite_collision_y_base = value;
    }

    pub(crate) fn set_repulsespark_floor_status(&mut self, value: u8) {
        self.repulsespark_floor_status = value;
    }

    pub(crate) fn clear_boulder_trap_count(&mut self) {
        self.boulder_trap_count = 0;
    }

    pub(crate) fn increment_boulder_trap_count(&mut self) {
        self.boulder_trap_count = self.boulder_trap_count.wrapping_add(1);
    }

    pub(crate) fn clear_haunted_grove_flute_event_latch(&mut self) {
        self.haunted_grove_flute_event_latch = 0;
    }
}

pub(crate) struct NativeGarnishRuntimeBridgeMut<'a> {
    state: &'a mut GarnishRuntimeState,
    ram: &'a mut [u8],
}

impl<'a> NativeGarnishRuntimeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut GarnishRuntimeState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, GarnishRuntimeState::load_from_ram(self.ram));
    }

    pub(crate) fn set_active_type(&mut self, value: u8) {
        self.state.set_active_type(value);
        self.sync();
    }

    pub(crate) fn clear_active_type(&mut self) {
        self.state.clear_active_type();
        self.sync();
    }

    pub(crate) fn increment_boulder_trap_timer(&mut self) -> u8 {
        let value = self.state.increment_boulder_trap_timer();
        self.sync();
        value
    }

    pub(crate) fn set_active_overlord_index(&mut self, value: u8) {
        self.state.set_active_overlord_index(value);
        self.sync();
    }

    pub(crate) fn increment_haunted_grove_flute_event_latch(&mut self) {
        self.state.increment_haunted_grove_flute_event_latch();
        self.sync();
    }

    pub(crate) fn set_repulsespark_timer(&mut self, value: u8) {
        self.state.set_repulsespark_timer(value);
        self.sync();
    }

    pub(crate) fn clear_repulsespark_timer(&mut self) {
        self.state.clear_repulsespark_timer();
        self.sync();
    }

    pub(crate) fn decrement_repulsespark_timer(&mut self) {
        self.state.decrement_repulsespark_timer();
        self.sync();
    }

    pub(crate) fn set_repulsespark_anim_delay(&mut self, value: u8) {
        self.state.set_repulsespark_anim_delay(value);
        self.sync();
    }

    pub(crate) fn decrement_repulsespark_anim_delay(&mut self) -> u8 {
        let value = self.state.decrement_repulsespark_anim_delay();
        self.sync();
        value
    }

    pub(crate) fn set_repulsespark_x_lo(&mut self, value: u8) {
        self.state.set_repulsespark_x_lo(value);
        self.sync();
    }

    pub(crate) fn set_repulsespark_y_lo(&mut self, value: u8) {
        self.state.set_repulsespark_y_lo(value);
        self.sync();
    }

    pub(crate) fn set_sprcoll_x_size(&mut self, value: u16) {
        self.state.set_sprcoll_x_size(value);
        self.sync();
    }

    pub(crate) fn set_sprcoll_y_size(&mut self, value: u16) {
        self.state.set_sprcoll_y_size(value);
        self.sync();
    }

    pub(crate) fn set_sprcoll_x_base(&mut self, value: u16) {
        self.state.set_sprcoll_x_base(value);
        self.sync();
    }

    pub(crate) fn set_sprcoll_y_base(&mut self, value: u16) {
        self.state.set_sprcoll_y_base(value);
        self.sync();
    }

    pub(crate) fn set_repulsespark_floor_status(&mut self, value: u8) {
        self.state.set_repulsespark_floor_status(value);
        self.sync();
    }

    pub(crate) fn clear_boulder_trap_count(&mut self) {
        self.state.clear_boulder_trap_count();
        self.sync();
    }

    pub(crate) fn increment_boulder_trap_count(&mut self) {
        self.state.increment_boulder_trap_count();
        self.sync();
    }

    pub(crate) fn clear_haunted_grove_flute_event_latch(&mut self) {
        self.state.clear_haunted_grove_flute_event_latch();
        self.sync();
    }
}
