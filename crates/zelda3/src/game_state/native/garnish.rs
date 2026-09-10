//! Garnish effect slots and shared runtime state.

use crate::game_state::constants::{
    ACTIVE_OVERLORD_INDEX, GARNISH_ACTIVE, GARNISH_COUNTDOWN, GARNISH_FLOOR, GARNISH_OAM_FLAGS,
    GARNISH_SPRITE, GARNISH_TYPE, GARNISH_X_HI, GARNISH_X_LO, GARNISH_X_SUBPIXEL,
    GARNISH_X_VELOCITY, GARNISH_Y_HI, GARNISH_Y_LO, GARNISH_Y_SUBPIXEL, GARNISH_Y_VELOCITY,
    HAUNTED_GROVE_FLUTE_EVENT_LATCH, OVERWORLD_BOULDER_TRAP_COUNT, OVERWORLD_BOULDER_TRAP_TIMER,
    REPULSESPARK_ANIM_DELAY, REPULSESPARK_FLOOR_STATUS, REPULSESPARK_TIMER, REPULSESPARK_X_LO,
    REPULSESPARK_Y_LO, SPRCOLL_X_BASE, SPRCOLL_X_SIZE, SPRCOLL_Y_BASE, SPRCOLL_Y_SIZE,
};
use crate::game_state::native::ram_target::RamTarget;
use crate::types::read_le_u16;

const GARNISH_SLOT_COUNT: usize = 30;

/// A named view of the live garnish bytes in WRAM. There is no second slot
/// bank to load, project, or reconcile; snapshots retain their own WRAM copy.
pub(crate) struct GarnishSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> GarnishSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        assert!(slot < GARNISH_SLOT_COUNT, "garnish slot out of bounds");
        Self { ram, slot }
    }

    pub(crate) fn garnish_type(&self) -> u8 {
        self.ram[GARNISH_TYPE + self.slot]
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.ram[GARNISH_X_LO + self.slot]
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.ram[GARNISH_X_HI + self.slot]
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.ram[GARNISH_Y_LO + self.slot]
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.ram[GARNISH_Y_HI + self.slot]
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.ram[GARNISH_X_VELOCITY + self.slot]
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.ram[GARNISH_Y_VELOCITY + self.slot]
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.ram[GARNISH_X_SUBPIXEL + self.slot]
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.ram[GARNISH_Y_SUBPIXEL + self.slot]
    }

    pub(crate) fn countdown(&self) -> u8 {
        self.ram[GARNISH_COUNTDOWN + self.slot]
    }

    pub(crate) fn sprite(&self) -> u8 {
        self.ram[GARNISH_SPRITE + self.slot]
    }

    pub(crate) fn floor(&self) -> u8 {
        self.ram[GARNISH_FLOOR + self.slot]
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        self.ram[GARNISH_OAM_FLAGS + self.slot]
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.garnish_type() == 0
    }

    pub(crate) fn x(&self) -> u16 {
        u16::from_le_bytes([self.x_low(), self.x_high()])
    }

    pub(crate) fn y(&self) -> u16 {
        u16::from_le_bytes([self.y_low(), self.y_high()])
    }
}

/// Mutates only the selected garnish field's bytes in the authoritative WRAM.
pub(crate) struct GarnishSlotMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> GarnishSlotMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        assert!(slot < GARNISH_SLOT_COUNT, "garnish slot out of bounds");
        Self { ram, slot }
    }

    pub(crate) fn set_garnish_type(&mut self, value: u8) {
        self.ram[GARNISH_TYPE + self.slot] = value;
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[GARNISH_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[GARNISH_X_HI + self.slot] = value;
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[GARNISH_Y_HI + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[GARNISH_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[GARNISH_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[GARNISH_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[GARNISH_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.ram[GARNISH_COUNTDOWN + self.slot] = value;
    }

    pub(crate) fn set_sprite(&mut self, value: u8) {
        crate::types::ww_check(
            GARNISH_SPRITE + self.slot,
            1,
            "GarnishSlotMut::set_sprite",
            value as u32,
        );
        self.ram[GARNISH_SPRITE + self.slot] = value;
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[GARNISH_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.ram[GARNISH_OAM_FLAGS + self.slot] = value;
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.set_x_low(value as u8);
        self.set_x_high((value >> 8) as u8);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.set_y_low(value as u8);
        self.set_y_high((value >> 8) as u8);
    }
}

#[cfg(test)]
#[path = "garnish_tests.rs"]
mod tests;

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

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(GARNISH_ACTIVE, self.active_type);
        ram.write_byte(OVERWORLD_BOULDER_TRAP_COUNT, self.boulder_trap_count);
        ram.write_byte(OVERWORLD_BOULDER_TRAP_TIMER, self.boulder_trap_timer);
        ram.write_word(SPRCOLL_X_SIZE, self.sprite_collision_x_size);
        ram.write_word(SPRCOLL_Y_SIZE, self.sprite_collision_y_size);
        ram.write_word(SPRCOLL_X_BASE, self.sprite_collision_x_base);
        ram.write_word(SPRCOLL_Y_BASE, self.sprite_collision_y_base);
        ram.write_byte(ACTIVE_OVERLORD_INDEX, self.active_overlord_index);
        ram.write_byte(
            HAUNTED_GROVE_FLUTE_EVENT_LATCH,
            self.haunted_grove_flute_event_latch,
        );
        ram.write_byte(REPULSESPARK_TIMER, self.repulsespark_timer);
        ram.write_byte(REPULSESPARK_ANIM_DELAY, self.repulsespark_anim_delay);
        ram.write_byte(REPULSESPARK_FLOOR_STATUS, self.repulsespark_floor_status);
        ram.write_byte(REPULSESPARK_X_LO, self.repulsespark_x_low);
        ram.write_byte(REPULSESPARK_Y_LO, self.repulsespark_y_low);
    }

    pub(crate) fn active_type(&self) -> u8 {
        self.active_type
    }

    pub(crate) fn boulder_trap_count(&self) -> u8 {
        self.boulder_trap_count
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

adopting_bridge!(NativeGarnishRuntimeBridgeMut, state: GarnishRuntimeState);

impl<'a> NativeGarnishRuntimeBridgeMut<'a> {
    forward_synced! {
        state;
        fn set_active_type(value: u8);
        fn clear_active_type();
        fn increment_boulder_trap_timer() -> u8;
        fn set_active_overlord_index(value: u8);
        fn increment_haunted_grove_flute_event_latch();
        fn set_repulsespark_timer(value: u8);
        fn clear_repulsespark_timer();
        fn decrement_repulsespark_timer();
        fn set_repulsespark_anim_delay(value: u8);
        fn decrement_repulsespark_anim_delay() -> u8;
        fn set_repulsespark_x_lo(value: u8);
        fn set_repulsespark_y_lo(value: u8);
        fn set_sprcoll_x_size(value: u16);
        fn set_sprcoll_y_size(value: u16);
        fn set_sprcoll_x_base(value: u16);
        fn set_sprcoll_y_base(value: u16);
        fn set_repulsespark_floor_status(value: u8);
        fn clear_boulder_trap_count();
        fn increment_boulder_trap_count();
        fn clear_haunted_grove_flute_event_latch();
    }
}
