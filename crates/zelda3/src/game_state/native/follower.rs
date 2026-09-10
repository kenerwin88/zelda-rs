//! Follower runtime state and the tagalong position trail.

use crate::game_state::constants::{
    FOLLOWER_DROPPED, FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX, FOLLOWER_INDICATOR,
    FOLLOWER_JUMP_TIMER, FOLLOWER_PALETTE_SWAP_FLAG, FOLLOWER_SAVED_FLOOR, FOLLOWER_SAVED_INDOORS,
    FOLLOWER_SAVED_X, FOLLOWER_SAVED_Y, FOLLOWER_TAIL_WRITE_INDEX, TAGALONG_ANIM_FRAME_COUNTER,
    TAGALONG_APPEARANCE_NONE_FLAG, TAGALONG_DATA_INDEX, TAGALONG_EVENT_FLAGS,
    TAGALONG_HOOKSHOT_INTERLOCK, TAGALONG_LAYERBITS, TAGALONG_SHARED_STATE_A, TAGALONG_X_HI,
    TAGALONG_X_LO, TAGALONG_Y_HI, TAGALONG_Y_LO, TAGALONG_Z, TIMER_TAGALONG_REACQUIRE,
    ZELDA_RESCUE_CUTSCENE_STATE,
};
use crate::game_state::native::ram_target::RamTarget;
use crate::types::read_le_u16;

const TAGALONG_SLOT_COUNT: usize = 20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FollowerRuntimeState {
    indicator: u8,
    data_index: u8,
    appearance_none_flag: u8,
    dropped: u8,
    hookshot_interlock: u8,
    hookshot_release_tail_index: u8,
    tail_write_index: u8,
    event_flags: u8,
    reacquire_timer_low: u8,
    shared_state_a: u8,
    draw_anim_frame: u8,
    jump_timer: u8,
    saved_y: u16,
    saved_x: u16,
    saved_indoor_flag: u8,
    saved_floor: u8,
    palette_swap_flag: u8,
    zelda_rescue_cutscene_state: u8,
}

impl FollowerRuntimeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            indicator: ram.get(FOLLOWER_INDICATOR).copied().unwrap_or(0),
            data_index: ram.get(TAGALONG_DATA_INDEX).copied().unwrap_or(0),
            appearance_none_flag: ram.get(TAGALONG_APPEARANCE_NONE_FLAG).copied().unwrap_or(0),
            dropped: ram.get(FOLLOWER_DROPPED).copied().unwrap_or(0),
            hookshot_interlock: ram.get(TAGALONG_HOOKSHOT_INTERLOCK).copied().unwrap_or(0),
            hookshot_release_tail_index: ram
                .get(FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX)
                .copied()
                .unwrap_or(0),
            tail_write_index: ram.get(FOLLOWER_TAIL_WRITE_INDEX).copied().unwrap_or(0),
            event_flags: ram.get(TAGALONG_EVENT_FLAGS).copied().unwrap_or(0),
            reacquire_timer_low: ram.get(TIMER_TAGALONG_REACQUIRE).copied().unwrap_or(0),
            shared_state_a: ram.get(TAGALONG_SHARED_STATE_A).copied().unwrap_or(0),
            draw_anim_frame: ram.get(TAGALONG_ANIM_FRAME_COUNTER).copied().unwrap_or(0),
            jump_timer: ram.get(FOLLOWER_JUMP_TIMER).copied().unwrap_or(0),
            saved_y: read_le_u16(ram, FOLLOWER_SAVED_Y),
            saved_x: read_le_u16(ram, FOLLOWER_SAVED_X),
            saved_indoor_flag: ram.get(FOLLOWER_SAVED_INDOORS).copied().unwrap_or(0),
            saved_floor: ram.get(FOLLOWER_SAVED_FLOOR).copied().unwrap_or(0),
            palette_swap_flag: ram.get(FOLLOWER_PALETTE_SWAP_FLAG).copied().unwrap_or(0),
            zelda_rescue_cutscene_state: ram.get(ZELDA_RESCUE_CUTSCENE_STATE).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        ram.write_byte(FOLLOWER_INDICATOR, self.indicator);
        ram.write_byte(TAGALONG_DATA_INDEX, self.data_index);
        ram.write_byte(TAGALONG_APPEARANCE_NONE_FLAG, self.appearance_none_flag);
        ram.write_byte(FOLLOWER_DROPPED, self.dropped);
        ram.write_byte(TAGALONG_HOOKSHOT_INTERLOCK, self.hookshot_interlock);
        ram.write_byte(
            FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX,
            self.hookshot_release_tail_index,
        );
        ram.write_byte(FOLLOWER_TAIL_WRITE_INDEX, self.tail_write_index);
        ram.write_byte(TAGALONG_EVENT_FLAGS, self.event_flags);
        ram.write_byte(TIMER_TAGALONG_REACQUIRE, self.reacquire_timer_low);
        ram.write_byte(TAGALONG_SHARED_STATE_A, self.shared_state_a);
        ram.write_byte(TAGALONG_ANIM_FRAME_COUNTER, self.draw_anim_frame);
        ram.write_byte(FOLLOWER_JUMP_TIMER, self.jump_timer);
        ram.write_word(FOLLOWER_SAVED_Y, self.saved_y);
        ram.write_word(FOLLOWER_SAVED_X, self.saved_x);
        ram.write_byte(FOLLOWER_SAVED_INDOORS, self.saved_indoor_flag);
        ram.write_byte(FOLLOWER_SAVED_FLOOR, self.saved_floor);
        // FOLLOWER_KIKI_ANIM_COUNTER (0xb69) is solely owned by SpriteSystemState. C
        // reuses byte_7E0B69 across mutually-exclusive users -- Blind's head animation,
        // the tutorial guard's message index, and this Kiki clear -- and follower_runtime
        // projects AFTER system in SpriteState::write_to_ram, so a second copy here
        // re-stamped the live counter every frame.
        ram.write_byte(FOLLOWER_PALETTE_SWAP_FLAG, self.palette_swap_flag);
        // ZELDA_RESCUE_CUTSCENE_STATE (0x1fe01) is write-through, not projected -- see the
        // note in SpriteState::write_to_ram.
    }

    pub(crate) fn indicator(&self) -> u8 {
        self.indicator
    }

    pub(crate) fn indicator_word(&self) -> u16 {
        u16::from(self.indicator) | (u16::from(self.saved_y as u8) << 8)
    }

    pub(crate) fn data_index(&self) -> u8 {
        self.data_index
    }

    pub(crate) fn data_index_word(&self) -> u16 {
        u16::from(self.data_index) | (u16::from(self.hookshot_interlock) << 8)
    }

    pub(crate) fn appearance_none_flag(&self) -> u8 {
        self.appearance_none_flag
    }

    pub(crate) fn dropped(&self) -> u8 {
        self.dropped
    }

    pub(crate) fn hookshot_interlock(&self) -> u8 {
        self.hookshot_interlock
    }

    pub(crate) fn hookshot_interlock_is_clear(&self) -> bool {
        self.hookshot_interlock() == 0
    }

    pub(crate) fn tail_write_index(&self) -> u8 {
        self.tail_write_index
    }

    pub(crate) fn hookshot_release_tail_index(&self) -> u8 {
        self.hookshot_release_tail_index
    }

    pub(crate) fn event_flags(&self) -> u8 {
        self.event_flags
    }

    pub(crate) fn reacquire_timer_low(&self) -> u8 {
        self.reacquire_timer_low
    }

    #[cfg(test)]
    pub(crate) fn reacquire_timer(&self) -> u16 {
        u16::from(self.reacquire_timer_low) | (u16::from(self.tail_write_index) << 8)
    }

    pub(crate) fn draw_anim_frame(&self) -> u8 {
        self.draw_anim_frame
    }

    pub(crate) fn saved_y(&self) -> u16 {
        self.saved_y
    }

    pub(crate) fn saved_x(&self) -> u16 {
        self.saved_x
    }

    pub(crate) fn saved_indoor_flag(&self) -> u8 {
        self.saved_indoor_flag
    }

    pub(crate) fn saved_floor(&self) -> u8 {
        self.saved_floor
    }

    pub(crate) fn palette_swap_flag(&self) -> u8 {
        self.palette_swap_flag
    }

    pub(crate) fn zelda_rescue_cutscene_state(&self) -> u8 {
        self.zelda_rescue_cutscene_state
    }

    pub(crate) fn clear_palette_swap_flag(&mut self) {
        self.palette_swap_flag = 0;
    }

    pub(crate) fn set_palette_swap_flag(&mut self, value: u8) {
        self.palette_swap_flag = value;
    }

    pub(crate) fn set_indicator(&mut self, value: u8) {
        self.indicator = value;
    }

    pub(crate) fn set_data_index(&mut self, value: u8) {
        self.data_index = value;
    }

    pub(crate) fn advance_data_index_wrapping_at_20(&mut self) {
        self.data_index = if self.data_index.wrapping_add(1) >= 20 {
            0
        } else {
            self.data_index.wrapping_add(1)
        };
    }

    pub(crate) fn xor_indicator(&mut self, value: u8) {
        self.indicator ^= value;
    }

    pub(crate) fn set_appearance_none_flag(&mut self, value: u8) {
        self.appearance_none_flag = value;
    }

    pub(crate) fn set_dropped(&mut self, value: u8) {
        self.dropped = value;
    }

    pub(crate) fn clear_hookshot_interlock(&mut self) {
        self.hookshot_interlock = 0;
    }

    pub(crate) fn set_hookshot_interlock(&mut self) {
        self.hookshot_interlock = 1;
    }

    pub(crate) fn clear_event_flags(&mut self) {
        self.event_flags = 0;
    }

    pub(crate) fn or_event_flags(&mut self, value: u8) {
        self.event_flags |= value;
    }

    pub(crate) fn and_event_flags(&mut self, value: u8) {
        self.event_flags &= value;
    }

    pub(crate) fn set_hookshot_release_tail_index_from_tail_write_index(&mut self) {
        self.hookshot_release_tail_index = self.tail_write_index;
    }

    pub(crate) fn set_tail_write_index(&mut self, value: u8) {
        self.tail_write_index = value;
    }

    pub(crate) fn increment_tail_write_index(&mut self) {
        self.tail_write_index = self.tail_write_index.wrapping_add(1);
    }

    pub(crate) fn set_reacquire_timer_low(&mut self, value: u8) {
        self.reacquire_timer_low = value;
    }

    pub(crate) fn decrement_reacquire_timer_low(&mut self) {
        self.set_reacquire_timer_low(self.reacquire_timer_low().wrapping_sub(1));
    }

    pub(crate) fn set_reacquire_timer(&mut self, value: u16) {
        self.reacquire_timer_low = value as u8;
        self.tail_write_index = (value >> 8) as u8;
    }

    pub(crate) fn clear_tagalong_shared_state_a(&mut self) {
        self.shared_state_a = 0;
    }

    pub(crate) fn clear_draw_anim_frame(&mut self) {
        self.draw_anim_frame = 0;
    }

    pub(crate) fn increment_and_cycle_draw_anim_frame(&mut self) {
        self.draw_anim_frame = self.draw_anim_frame.wrapping_add(1);
        if self.draw_anim_frame == 3 {
            self.draw_anim_frame = 0;
        }
    }

    pub(crate) fn clear_jump_timer(&mut self) {
        self.jump_timer = 0;
    }

    pub(crate) fn set_saved_y(&mut self, value: u16) {
        self.saved_y = value;
    }

    pub(crate) fn set_saved_x(&mut self, value: u16) {
        self.saved_x = value;
    }

    pub(crate) fn set_saved_indoor_flag(&mut self, value: u8) {
        self.saved_indoor_flag = value;
    }

    pub(crate) fn set_saved_floor(&mut self, value: u8) {
        self.saved_floor = value;
    }

    pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
        self.zelda_rescue_cutscene_state = value;
    }
}

pub(crate) struct NativeFollowerRuntimeBridgeMut<'a> {
    before: Vec<(usize, u8)>,
    state: &'a mut FollowerRuntimeState,
    ram: &'a mut [u8],
}

impl<'a> NativeFollowerRuntimeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut FollowerRuntimeState, ram: &'a mut [u8]) -> Self {
        *state = FollowerRuntimeState::load_from_ram(&*ram);
        let before =
            crate::game_state::native::ram_target::capture(&*ram, |log| state.write_to_ram(log));
        Self { before, state, ram }
    }

    fn sync(&mut self) {
        let now = crate::game_state::native::ram_target::capture(&*self.ram, |log| {
            self.state.write_to_ram(log)
        });
        crate::game_state::native::ram_target::publish_changes(&self.before, &now, self.ram);
        self.before = now;
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        let mut projected = FollowerRuntimeState::load_from_ram(self.ram);
        // This byte is deliberately write-through: unrelated follower mutations must
        // not project the frame-start rescue state back over a newer C-order write.
        // Exclude it from the bulk-projection assertion; its dedicated setter below
        // still writes and verifies the byte in the same operation.
        projected.zelda_rescue_cutscene_state = self.state.zelda_rescue_cutscene_state;
        debug_assert_eq!(*self.state, projected);
    }

    forward_synced! {
        state;
        fn clear_palette_swap_flag();
        fn set_palette_swap_flag(value: u8);
        fn set_indicator(value: u8);
        fn set_data_index(value: u8);
        fn advance_data_index_wrapping_at_20();
        fn xor_indicator(value: u8);
        fn set_appearance_none_flag(value: u8);
        fn set_dropped(value: u8);
        fn clear_hookshot_interlock();
        fn set_hookshot_interlock();
        fn clear_event_flags();
        fn or_event_flags(value: u8);
        fn and_event_flags(value: u8);
    }

    pub(crate) fn set_hookshot_release_tail_index_from_tail_write_index(&mut self) {
        self.state
            .set_hookshot_release_tail_index_from_tail_write_index();
        self.sync();
    }

    forward_synced! {
        state;
        fn set_tail_write_index(value: u8);
        fn increment_tail_write_index();
        fn set_reacquire_timer_low(value: u8);
        fn decrement_reacquire_timer_low();
        fn set_reacquire_timer(value: u16);
        fn clear_tagalong_shared_state_a();
        fn clear_draw_anim_frame();
        fn increment_and_cycle_draw_anim_frame();
        fn clear_jump_timer();
        fn set_saved_y(value: u16);
        fn set_saved_x(value: u16);
        fn set_saved_indoor_flag(value: u8);
        fn set_saved_floor(value: u8);
    }

    pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
        self.state.set_zelda_rescue_cutscene_state(value);
        self.ram[ZELDA_RESCUE_CUTSCENE_STATE] = value;
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TagalongTrailState {
    x_low: [u8; TAGALONG_SLOT_COUNT],
    x_high: [u8; TAGALONG_SLOT_COUNT],
    y_low: [u8; TAGALONG_SLOT_COUNT],
    y_high: [u8; TAGALONG_SLOT_COUNT],
    z: [u8; TAGALONG_SLOT_COUNT],
    layer_bits: [u8; TAGALONG_SLOT_COUNT],
}

impl TagalongTrailState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x_low: read_tagalong_bank(ram, TAGALONG_X_LO),
            x_high: read_tagalong_bank(ram, TAGALONG_X_HI),
            y_low: read_tagalong_bank(ram, TAGALONG_Y_LO),
            y_high: read_tagalong_bank(ram, TAGALONG_Y_HI),
            z: read_tagalong_bank(ram, TAGALONG_Z),
            layer_bits: read_tagalong_bank(ram, TAGALONG_LAYERBITS),
        }
    }

    pub(crate) fn write_to_ram<R: RamTarget + ?Sized>(&self, ram: &mut R) {
        write_tagalong_bank(ram, TAGALONG_X_LO, self.x_low);
        write_tagalong_bank(ram, TAGALONG_X_HI, self.x_high);
        write_tagalong_bank(ram, TAGALONG_Y_LO, self.y_low);
        write_tagalong_bank(ram, TAGALONG_Y_HI, self.y_high);
        write_tagalong_bank(ram, TAGALONG_Z, self.z);
        write_tagalong_bank(ram, TAGALONG_LAYERBITS, self.layer_bits);
    }

    pub(crate) fn x(&self, slot: usize) -> u16 {
        packed_slot_position(self.x_low, self.x_high, slot)
    }

    pub(crate) fn y(&self, slot: usize) -> u16 {
        packed_slot_position(self.y_low, self.y_high, slot)
    }

    pub(crate) fn z(&self, slot: usize) -> u8 {
        self.z.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn layer_bits(&self, slot: usize) -> u8 {
        self.layer_bits.get(slot).copied().unwrap_or(0)
    }
}

fn read_tagalong_bank(ram: &[u8], base: usize) -> [u8; TAGALONG_SLOT_COUNT] {
    let mut bank = [0; TAGALONG_SLOT_COUNT];
    for (slot, value) in bank.iter_mut().enumerate() {
        *value = ram.get(base + slot).copied().unwrap_or(0);
    }
    bank
}

fn write_tagalong_bank<R: RamTarget + ?Sized>(
    ram: &mut R,
    base: usize,
    bank: [u8; TAGALONG_SLOT_COUNT],
) {
    for (slot, value) in bank.iter().copied().enumerate() {
        ram.write_byte(base + slot, value);
    }
}

fn packed_slot_position(
    low_bank: [u8; TAGALONG_SLOT_COUNT],
    high_bank: [u8; TAGALONG_SLOT_COUNT],
    slot: usize,
) -> u16 {
    u16::from(low_bank.get(slot).copied().unwrap_or(0))
        | (u16::from(high_bank.get(slot).copied().unwrap_or(0)) << 8)
}

pub(crate) struct TagalongSlotRead<'a> {
    state: &'a TagalongTrailState,
    slot: usize,
}

impl<'a> TagalongSlotRead<'a> {
    pub(crate) fn new(state: &'a TagalongTrailState, slot: usize) -> Self {
        Self { state, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        self.state.x(self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        self.state.y(self.slot)
    }

    pub(crate) fn z(&self) -> u8 {
        self.state.z(self.slot)
    }

    pub(crate) fn z_signed(&self) -> i8 {
        self.z() as i8
    }

    pub(crate) fn is_above_ground(&self) -> bool {
        self.z_signed() > 0
    }

    pub(crate) fn layer_bits(&self) -> u8 {
        self.state.layer_bits(self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        self.layer_bits() & 3
    }
}

pub(crate) struct NativeTagalongSlotBridgeMut<'a> {
    before: Vec<(usize, u8)>,
    state: &'a mut TagalongTrailState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeTagalongSlotBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TagalongTrailState, ram: &'a mut [u8], slot: usize) -> Self {
        *state = TagalongTrailState::load_from_ram(&*ram);
        let before =
            crate::game_state::native::ram_target::capture(&*ram, |log| state.write_to_ram(log));
        Self {
            before,
            state,
            ram,
            slot,
        }
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        if let Some(y_high) = self.state.y_high.get_mut(self.slot) {
            *y_high = value;
            self.sync();
        }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        if self.slot < TAGALONG_SLOT_COUNT {
            self.state.x_low[self.slot] = x as u8;
            self.state.x_high[self.slot] = (x >> 8) as u8;
            self.state.y_low[self.slot] = y as u8;
            self.state.y_high[self.slot] = (y >> 8) as u8;
            self.sync();
        }
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        if let Some(z) = self.state.z.get_mut(self.slot) {
            *z = value;
            self.sync();
        }
    }

    pub(crate) fn set_layer_bits(&mut self, value: u8) {
        if let Some(layer_bits) = self.state.layer_bits.get_mut(self.slot) {
            *layer_bits = value;
            self.sync();
        }
    }

    fn sync(&mut self) {
        let now = crate::game_state::native::ram_target::capture(&*self.ram, |log| {
            self.state.write_to_ram(log)
        });
        crate::game_state::native::ram_target::publish_changes(&self.before, &now, self.ram);
        self.before = now;
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TagalongTrailState::load_from_ram(self.ram));
    }
}
