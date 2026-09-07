//! Ancilla slot storage, WRAM aliases, and write-through access.

use super::sprites::SlotBankBytes;
use crate::game_state::constants::{
    ANCILLA_A, ANCILLA_AUX_TIMER, ANCILLA_B, ANCILLA_DIRECTION, ANCILLA_FLOOR, ANCILLA_FLOOR2,
    ANCILLA_G, ANCILLA_H, ANCILLA_ITEM_TO_LINK, ANCILLA_K, ANCILLA_L, ANCILLA_NUMSPR,
    ANCILLA_OAM_IDX, ANCILLA_OBJPRIO, ANCILLA_R, ANCILLA_STEP, ANCILLA_S_PLAYER,
    ANCILLA_TILE_ATTRIBUTE, ANCILLA_TIMER, ANCILLA_TYPE, ANCILLA_T_PLAYER, ANCILLA_U,
    ANCILLA_WORK_BYTE_1, ANCILLA_WORK_BYTE_22, ANCILLA_WORK_BYTE_23, ANCILLA_WORK_BYTE_24,
    ANCILLA_WORK_BYTE_25, ANCILLA_WORK_BYTE_26, ANCILLA_WORK_BYTE_3, ANCILLA_WORK_BYTE_4,
    ANCILLA_X_HI, ANCILLA_X_LO, ANCILLA_X_SUBPIXEL, ANCILLA_X_VELOCITY, ANCILLA_Y_HI, ANCILLA_Y_LO,
    ANCILLA_Y_SUBPIXEL, ANCILLA_Y_VELOCITY, ANCILLA_Z, ANCILLA_Z_SUBPIXEL_PLAYER,
    ANCILLA_Z_VELOCITY, DOOR_DEBRIS_DIRECTION, DOOR_DEBRIS_X, DOOR_DEBRIS_Y,
};

pub(crate) const ANCILLA_SLOT_COUNT: usize = 10;

const ANCILLA_WORK_BASE: usize = ANCILLA_OBJPRIO;
const ANCILLA_WORK_END: usize = ANCILLA_NUMSPR + ANCILLA_SLOT_COUNT;
const ANCILLA_WORK_LEN: usize = ANCILLA_WORK_END - ANCILLA_WORK_BASE;
const ANCILLA_FIELD_BASES: &[usize] = &[
    ANCILLA_OBJPRIO,
    ANCILLA_U,
    ANCILLA_Z_VELOCITY,
    ANCILLA_Z,
    ANCILLA_Z_SUBPIXEL_PLAYER,
    ANCILLA_K,
    ANCILLA_L,
    ANCILLA_A,
    ANCILLA_B,
    ANCILLA_G,
    ANCILLA_WORK_BYTE_3,
    ANCILLA_WORK_BYTE_1,
    ANCILLA_S_PLAYER,
    ANCILLA_AUX_TIMER,
    // Door debris lives inside the ancilla RAM window on the hardware
    // ($03B6/$03BA/$03BE) and overlaps the aux timers and arr26; the bank
    // owns those cells so every alias is one byte.
    DOOR_DEBRIS_X,
    DOOR_DEBRIS_Y,
    DOOR_DEBRIS_DIRECTION,
    ANCILLA_H,
    ANCILLA_FLOOR2,
    ANCILLA_WORK_BYTE_23,
    ANCILLA_T_PLAYER,
    ANCILLA_WORK_BYTE_24,
    ANCILLA_TILE_ATTRIBUTE,
    ANCILLA_R,
    ANCILLA_WORK_BYTE_26,
    ANCILLA_WORK_BYTE_25,
    ANCILLA_WORK_BYTE_22,
    ANCILLA_WORK_BYTE_4,
    ANCILLA_Y_LO,
    ANCILLA_X_LO,
    ANCILLA_Y_HI,
    ANCILLA_X_HI,
    ANCILLA_Y_VELOCITY,
    ANCILLA_X_VELOCITY,
    ANCILLA_Y_SUBPIXEL,
    ANCILLA_X_SUBPIXEL,
    ANCILLA_TYPE,
    ANCILLA_STEP,
    ANCILLA_ITEM_TO_LINK,
    ANCILLA_TIMER,
    ANCILLA_DIRECTION,
    ANCILLA_FLOOR,
    ANCILLA_OAM_IDX,
    ANCILLA_NUMSPR,
];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AncillaSlotsState {
    work: Vec<u8>,
}

impl Default for AncillaSlotsState {
    fn default() -> Self {
        Self {
            work: vec![0; ANCILLA_WORK_LEN],
        }
    }
}

/// ANCILLA_G slot 9 (0x394 + 9 = 0x39d) is the same byte as GAME_OVER_LETTER_CURSOR / the
/// hookshot effect index — a deliberate C memory overlap (ancilla_arr_g has 10 entries and the
/// last one doubles as the effect index). That byte is owned by messaging here (set via
/// set_effect_index, read raw at ancilla.rs during a boomerang/hookshot throw), so the ancilla
/// bulk projection must not touch it or it re-stamps a stale frame-start value over messaging's
/// mid-frame write (f358784).
const ANCILLA_G_SLOT9_HOOKSHOT_EFFECT: usize = ANCILLA_G + (ANCILLA_SLOT_COUNT - 1);

/// ANCILLA_R (0x3ea) is only a 5-slot array in the C (ancilla_arr_r[5], bounded below by
/// LINK_SOMETHING_WITH_HOOKSHOT 0x3e9 and above by LINK_FORCE_HOLD_SWORD_UP 0x3ef). Slots 5-9
/// (0x3ef-0x3f3) are FOREIGN Link/game vars — LINK_FORCE_HOLD_SWORD_UP, FLUTE_COUNTDOWN,
/// MOVING_FLOOR_BG_CHECK_FLAGS, and LINK_ON_CONVEYOR_BELT (0x3f3 == ANCILLA_R+9). Projecting our
/// stale R[5..9] over them re-stamps mid-frame writes (f843633: the spin-charge ancilla in slot 9
/// re-stamped 0x3f3 with the stale conveyor value after link-main cleared it). Same overlap class
/// as ANCILLA_G[9] (f358784).
const ANCILLA_R_REAL_SLOT_COUNT: usize = 5;
fn is_ancilla_r_overflow(offset: usize) -> bool {
    (ANCILLA_R + ANCILLA_R_REAL_SLOT_COUNT..ANCILLA_R + ANCILLA_SLOT_COUNT).contains(&offset)
}

impl AncillaSlotsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for offset in Self::field_offsets() {
            if offset == ANCILLA_G_SLOT9_HOOKSHOT_EFFECT || is_ancilla_r_overflow(offset) {
                continue;
            }
            let index = Self::work_index(offset);
            state.work[index] = ram.get(offset).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for offset in Self::field_offsets() {
            if offset == ANCILLA_G_SLOT9_HOOKSHOT_EFFECT || is_ancilla_r_overflow(offset) {
                continue;
            }
            ram[offset] = self.byte_at(offset);
        }
    }

    pub(crate) fn slot(&self, slot: usize) -> NativeAncillaSlotView<'_> {
        NativeAncillaSlotView { state: self, slot }
    }

    pub(crate) fn slot_mut<'a>(
        &'a mut self,
        ram: &'a mut [u8],
        slot: usize,
    ) -> NativeAncillaSlotBridgeMut<'a> {
        NativeAncillaSlotBridgeMut {
            state: self,
            ram,
            slot,
        }
    }

    fn work_index(offset: usize) -> usize {
        offset - ANCILLA_WORK_BASE
    }

    fn field_offsets() -> impl Iterator<Item = usize> {
        ANCILLA_FIELD_BASES
            .iter()
            .copied()
            .flat_map(|base| (0..ANCILLA_SLOT_COUNT).map(move |slot| base + slot))
    }

    fn byte_at(&self, offset: usize) -> u8 {
        self.work
            .get(Self::work_index(offset))
            .copied()
            .unwrap_or(0)
    }

    fn set_byte_at(&mut self, offset: usize, value: u8) {
        let index = Self::work_index(offset);
        if self.work.len() < ANCILLA_WORK_LEN {
            self.work.resize(ANCILLA_WORK_LEN, 0);
        }
        self.work[index] = value;
    }

    fn byte(&self, slot: usize, offset: usize) -> u8 {
        self.byte_at(offset + slot)
    }

    fn set_byte(&mut self, slot: usize, offset: usize, value: u8) {
        self.set_byte_at(offset + slot, value);
    }

    /// Write one bank byte that several hardware arrays share (currently
    /// `ancilla_alloc_rotate` at $03C4 == arr26[4] == arr25[2]). The caller
    /// writes the same byte through to RAM; the overlap is exact by
    /// construction because every alias is the same bank cell.
    pub(crate) fn set_shared_byte(&mut self, offset: usize, value: u8) {
        debug_assert!((ANCILLA_WORK_BASE..ANCILLA_WORK_END).contains(&offset));
        self.set_byte_at(offset, value);
    }

    fn word_at(&self, offset: usize) -> u16 {
        u16::from(self.byte_at(offset)) | (u16::from(self.byte_at(offset + 1)) << 8)
    }

    fn set_word_at(&mut self, offset: usize, value: u16) {
        self.set_byte_at(offset, value as u8);
        self.set_byte_at(offset + 1, (value >> 8) as u8);
    }

    fn packed_position(&self, slot: usize, low_offset: usize, high_offset: usize) -> u16 {
        u16::from(self.byte(slot, low_offset)) | (u16::from(self.byte(slot, high_offset)) << 8)
    }

    fn set_position(&mut self, slot: usize, low_offset: usize, high_offset: usize, value: u16) {
        self.set_byte(slot, low_offset, value as u8);
        self.set_byte(slot, high_offset, (value >> 8) as u8);
    }

    fn add_byte(&mut self, slot: usize, offset: usize, value: u8) -> u8 {
        let next = self.byte(slot, offset).wrapping_add(value);
        self.set_byte(slot, offset, next);
        next
    }

    fn subtract_byte(&mut self, slot: usize, offset: usize, value: u8) -> u8 {
        let next = self.byte(slot, offset).wrapping_sub(value);
        self.set_byte(slot, offset, next);
        next
    }

    fn xor_byte(&mut self, slot: usize, offset: usize, value: u8) -> u8 {
        let next = self.byte(slot, offset) ^ value;
        self.set_byte(slot, offset, next);
        next
    }

    fn or_byte(&mut self, slot: usize, offset: usize, value: u8) {
        let next = self.byte(slot, offset) | value;
        self.set_byte(slot, offset, next);
    }

    fn and_byte(&mut self, slot: usize, offset: usize, value: u8) {
        let next = self.byte(slot, offset) & value;
        self.set_byte(slot, offset, next);
    }

    fn negate_byte(&mut self, slot: usize, offset: usize) {
        let next = self.byte(slot, offset).wrapping_neg();
        self.set_byte(slot, offset, next);
    }
}

pub(crate) struct NativeAncillaSlotView<'a> {
    state: &'a AncillaSlotsState,
    slot: usize,
}

impl<'a> NativeAncillaSlotView<'a> {
    pub(crate) fn ancilla_type(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_TYPE)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.ancilla_type() != 0
    }

    pub(crate) fn x(&self) -> u16 {
        self.state
            .packed_position(self.slot, ANCILLA_X_LO, ANCILLA_X_HI)
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_X_LO)
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_X_HI)
    }

    pub(crate) fn y(&self) -> u16 {
        self.state
            .packed_position(self.slot, ANCILLA_Y_LO, ANCILLA_Y_HI)
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Y_LO)
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Y_HI)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_X_VELOCITY)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Y_VELOCITY)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Z_VELOCITY)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_X_SUBPIXEL)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Y_SUBPIXEL)
    }

    pub(crate) fn z(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Z)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Z_SUBPIXEL_PLAYER)
    }

    pub(crate) fn item_to_link(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_ITEM_TO_LINK)
    }

    pub(crate) fn timer(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_TIMER)
    }

    pub(crate) fn floor(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_FLOOR)
    }

    pub(crate) fn floor2(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_FLOOR2)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_OBJPRIO)
    }

    pub(crate) fn u(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_U)
    }

    pub(crate) fn num_sprites(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_NUMSPR)
    }

    pub(crate) fn direction(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_DIRECTION)
    }

    pub(crate) fn tile_attribute(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_TILE_ATTRIBUTE)
    }

    pub(crate) fn step(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_STEP)
    }

    pub(crate) fn aux_timer(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_AUX_TIMER)
    }

    pub(crate) fn work_byte_3(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_3)
    }

    pub(crate) fn work_byte_1(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_1)
    }

    pub(crate) fn s_player(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_S_PLAYER)
    }

    pub(crate) fn t_player(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_T_PLAYER)
    }

    pub(crate) fn a(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_A)
    }

    pub(crate) fn b(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_B)
    }

    pub(crate) fn ab_word(&self) -> u16 {
        u16::from(self.a()) | (u16::from(self.b()) << 8)
    }

    pub(crate) fn a_word(&self) -> u16 {
        self.state.word_at(ANCILLA_A + self.slot)
    }

    pub(crate) fn l(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_L)
    }

    pub(crate) fn h(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_H)
    }

    pub(crate) fn k(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_K)
    }

    pub(crate) fn g(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_G)
    }

    pub(crate) fn r(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_R)
    }

    pub(crate) fn work_byte_22(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_22)
    }

    pub(crate) fn work_byte_23(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_23)
    }

    pub(crate) fn work_byte_24(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_24)
    }

    pub(crate) fn work_byte_4(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_4)
    }

    pub(crate) fn work_byte_25(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_25)
    }

    pub(crate) fn work_byte_26(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_WORK_BYTE_26)
    }
}

pub(crate) struct NativeAncillaSlotBridgeMut<'a> {
    state: &'a mut AncillaSlotsState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeAncillaSlotBridgeMut<'a> {
    fn sync(&mut self) {
        for base in ANCILLA_FIELD_BASES.iter().copied() {
            let offset = base + self.slot;
            // ANCILLA_G slot 9 (0x39d) overlaps GAME_OVER_LETTER_CURSOR / the hookshot effect
            // index, owned by messaging — projecting our stale g[9] re-stamps it (f358784).
            if offset == ANCILLA_G_SLOT9_HOOKSHOT_EFFECT || is_ancilla_r_overflow(offset) {
                continue;
            }
            self.ram[offset] = self.state.byte_at(offset);
        }
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        for base in ANCILLA_FIELD_BASES.iter().copied() {
            let offset = base + self.slot;
            if offset == ANCILLA_G_SLOT9_HOOKSHOT_EFFECT || is_ancilla_r_overflow(offset) {
                continue;
            }
            debug_assert_eq!(self.state.byte_at(offset), self.ram[offset]);
        }
    }

    fn set_byte(&mut self, offset: usize, value: u8) {
        self.state.set_byte(self.slot, offset, value);
        self.sync();
    }

    fn add_byte(&mut self, offset: usize, value: u8) -> u8 {
        let next = self.state.add_byte(self.slot, offset, value);
        self.sync();
        next
    }

    fn subtract_byte(&mut self, offset: usize, value: u8) -> u8 {
        let next = self.state.subtract_byte(self.slot, offset, value);
        self.sync();
        next
    }

    pub(crate) fn set_ancilla_type(&mut self, value: u8) {
        self.set_byte(ANCILLA_TYPE, value);
    }

    pub(crate) fn increment_ancilla_type(&mut self) -> u8 {
        self.add_byte(ANCILLA_TYPE, 1)
    }

    pub(crate) fn clear(&mut self) {
        self.set_ancilla_type(0);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.state
            .set_position(self.slot, ANCILLA_X_LO, ANCILLA_X_HI, value);
        self.sync();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.set_byte(ANCILLA_X_LO, value);
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.set_byte(ANCILLA_X_HI, value);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.state
            .set_position(self.slot, ANCILLA_Y_LO, ANCILLA_Y_HI, value);
        self.sync();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.set_byte(ANCILLA_Y_LO, value);
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.set_byte(ANCILLA_Y_HI, value);
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.set_byte(ANCILLA_X_SUBPIXEL, value);
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.set_byte(ANCILLA_Y_SUBPIXEL, value);
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.set_byte(ANCILLA_X_VELOCITY, value);
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.set_byte(ANCILLA_Y_VELOCITY, value);
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_X_VELOCITY)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.state.byte(self.slot, ANCILLA_Y_VELOCITY)
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) -> u8 {
        self.add_byte(ANCILLA_X_VELOCITY, value)
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) -> u8 {
        self.add_byte(ANCILLA_Y_VELOCITY, value)
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_Y_VELOCITY, value);
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        self.state.negate_byte(self.slot, ANCILLA_X_VELOCITY);
        self.sync();
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        self.state.negate_byte(self.slot, ANCILLA_Y_VELOCITY);
        self.sync();
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.set_byte(ANCILLA_Z_VELOCITY, value);
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.add_byte(ANCILLA_Z_VELOCITY, value);
    }

    pub(crate) fn tick_z_velocity(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_Z_VELOCITY, 1)
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.set_byte(ANCILLA_Z, value);
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.set_byte(ANCILLA_Z_SUBPIXEL_PLAYER, value);
    }

    pub(crate) fn move_x(&mut self) {
        self.state.move_axis24(
            self.slot,
            ANCILLA_X_SUBPIXEL,
            ANCILLA_X_LO,
            ANCILLA_X_HI,
            ANCILLA_X_VELOCITY,
        );
        self.sync();
    }

    pub(crate) fn move_y(&mut self) {
        self.state.move_axis24(
            self.slot,
            ANCILLA_Y_SUBPIXEL,
            ANCILLA_Y_LO,
            ANCILLA_Y_HI,
            ANCILLA_Y_VELOCITY,
        );
        self.sync();
    }

    pub(crate) fn move_z(&mut self) {
        self.state.move_axis16(
            self.slot,
            ANCILLA_Z_SUBPIXEL_PLAYER,
            ANCILLA_Z,
            ANCILLA_Z_VELOCITY,
        );
        self.sync();
    }

    pub(crate) fn set_item_to_link(&mut self, value: u8) {
        self.set_byte(ANCILLA_ITEM_TO_LINK, value);
    }

    pub(crate) fn advance_item_to_link(&mut self) -> u8 {
        self.add_byte(ANCILLA_ITEM_TO_LINK, 1)
    }

    pub(crate) fn add_item_to_link(&mut self, value: u8) {
        self.add_byte(ANCILLA_ITEM_TO_LINK, value);
    }

    pub(crate) fn retreat_item_to_link(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_ITEM_TO_LINK, 1)
    }

    pub(crate) fn toggle_item_to_link_bit0(&mut self) -> u8 {
        let next = self.state.xor_byte(self.slot, ANCILLA_ITEM_TO_LINK, 1);
        self.sync();
        next
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.set_byte(ANCILLA_TIMER, value);
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_TIMER, 1)
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.set_byte(ANCILLA_FLOOR, value);
    }

    pub(crate) fn set_floor2(&mut self, value: u8) {
        self.set_byte(ANCILLA_FLOOR2, value);
    }

    pub(crate) fn set_oam_index(&mut self, value: u8) {
        self.set_byte(ANCILLA_OAM_IDX, value);
    }

    pub(crate) fn set_num_sprites(&mut self, value: u8) {
        self.set_byte(ANCILLA_NUMSPR, value);
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.set_byte(ANCILLA_OBJPRIO, value);
    }

    pub(crate) fn xor_object_priority(&mut self, value: u8) {
        self.state.xor_byte(self.slot, ANCILLA_OBJPRIO, value);
        self.sync();
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.set_byte(ANCILLA_DIRECTION, value);
    }

    pub(crate) fn or_direction(&mut self, value: u8) {
        self.state.or_byte(self.slot, ANCILLA_DIRECTION, value);
        self.sync();
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.state.and_byte(self.slot, ANCILLA_DIRECTION, value);
        self.sync();
    }

    pub(crate) fn set_tile_attribute(&mut self, value: u8) {
        self.set_byte(ANCILLA_TILE_ATTRIBUTE, value);
    }

    pub(crate) fn set_step(&mut self, value: u8) {
        self.set_byte(ANCILLA_STEP, value);
    }

    pub(crate) fn advance_step(&mut self) -> u8 {
        self.add_byte(ANCILLA_STEP, 1)
    }

    pub(crate) fn add_step(&mut self, value: u8) {
        self.add_byte(ANCILLA_STEP, value);
    }

    pub(crate) fn retreat_step(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_STEP, 1)
    }

    pub(crate) fn set_aux_timer(&mut self, value: u8) {
        self.set_byte(ANCILLA_AUX_TIMER, value);
    }

    pub(crate) fn add_aux_timer(&mut self, value: u8) {
        self.add_byte(ANCILLA_AUX_TIMER, value);
    }

    pub(crate) fn tick_aux_timer(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_AUX_TIMER, 1)
    }

    pub(crate) fn set_work_byte_3(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_3, value);
    }

    pub(crate) fn add_work_byte_3(&mut self, value: u8) {
        self.add_byte(ANCILLA_WORK_BYTE_3, value);
    }

    pub(crate) fn set_work_byte_1(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_1, value);
    }

    pub(crate) fn subtract_work_byte_1(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_WORK_BYTE_1, value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.set_byte(ANCILLA_A, value);
    }

    pub(crate) fn advance_a(&mut self) -> u8 {
        self.add_byte(ANCILLA_A, 1)
    }

    pub(crate) fn set_a_word(&mut self, value: u16) {
        self.state.set_word_at(ANCILLA_A + self.slot, value);
        self.sync();
        self.ram[ANCILLA_A + self.slot + 1] = self.state.byte_at(ANCILLA_A + self.slot + 1);
        debug_assert_eq!(
            self.state.byte_at(ANCILLA_A + self.slot + 1),
            self.ram[ANCILLA_A + self.slot + 1]
        );
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.set_byte(ANCILLA_B, value);
    }

    pub(crate) fn set_l(&mut self, value: u8) {
        self.set_byte(ANCILLA_L, value);
    }

    pub(crate) fn advance_l(&mut self) -> u8 {
        self.add_byte(ANCILLA_L, 1)
    }

    pub(crate) fn add_l(&mut self, value: u8) {
        self.add_byte(ANCILLA_L, value);
    }

    pub(crate) fn retreat_l(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_L, 1)
    }

    pub(crate) fn set_h(&mut self, value: u8) {
        self.set_byte(ANCILLA_H, value);
    }

    pub(crate) fn set_k(&mut self, value: u8) {
        self.set_byte(ANCILLA_K, value);
    }

    pub(crate) fn toggle_k_bit0(&mut self) -> u8 {
        let next = self.state.xor_byte(self.slot, ANCILLA_K, 1);
        self.sync();
        next
    }

    pub(crate) fn advance_k(&mut self) -> u8 {
        self.add_byte(ANCILLA_K, 1)
    }

    pub(crate) fn add_k(&mut self, value: u8) {
        self.add_byte(ANCILLA_K, value);
    }

    pub(crate) fn retreat_k(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_K, 1)
    }

    pub(crate) fn tick_k(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_K, 1)
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.set_byte(ANCILLA_G, value);
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_G, value);
    }

    pub(crate) fn tick_g(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_G, 1)
    }

    pub(crate) fn set_s_player(&mut self, value: u8) {
        self.set_byte(ANCILLA_S_PLAYER, value);
    }

    pub(crate) fn set_t_player(&mut self, value: u8) {
        self.set_byte(ANCILLA_T_PLAYER, value);
    }

    pub(crate) fn set_r(&mut self, value: u8) {
        self.set_byte(ANCILLA_R, value);
    }

    pub(crate) fn add_r(&mut self, value: u8) {
        self.add_byte(ANCILLA_R, value);
    }

    pub(crate) fn tick_s_player(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_S_PLAYER, 1)
    }

    pub(crate) fn set_u(&mut self, value: u8) {
        self.set_byte(ANCILLA_U, value);
    }

    pub(crate) fn subtract_u(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_U, value);
    }

    pub(crate) fn advance_work_byte_1_mod4(&mut self) -> u8 {
        let next = self.state.add_byte(self.slot, ANCILLA_WORK_BYTE_1, 1) & 3;
        self.state.set_byte(self.slot, ANCILLA_WORK_BYTE_1, next);
        self.sync();
        next
    }

    pub(crate) fn add_work_byte_1_mod4(&mut self, value: u8) -> u8 {
        let next = self.state.add_byte(self.slot, ANCILLA_WORK_BYTE_1, value) & 3;
        self.state.set_byte(self.slot, ANCILLA_WORK_BYTE_1, next);
        self.sync();
        next
    }

    pub(crate) fn tick_work_byte_3(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_WORK_BYTE_3, 1)
    }

    pub(crate) fn advance_work_byte_3(&mut self) -> u8 {
        self.add_byte(ANCILLA_WORK_BYTE_3, 1)
    }

    pub(crate) fn set_work_byte_4(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_4, value);
    }

    pub(crate) fn subtract_work_byte_4(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_WORK_BYTE_4, value);
    }

    pub(crate) fn set_work_byte_22(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_22, value);
    }

    pub(crate) fn subtract_work_byte_22(&mut self, value: u8) {
        self.subtract_byte(ANCILLA_WORK_BYTE_22, value);
    }

    pub(crate) fn set_work_byte_23(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_23, value);
    }

    pub(crate) fn add_work_byte_23(&mut self, value: u8) {
        self.add_byte(ANCILLA_WORK_BYTE_23, value);
    }

    pub(crate) fn set_work_byte_24(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_24, value);
    }

    pub(crate) fn add_work_byte_24(&mut self, value: u8) {
        self.add_byte(ANCILLA_WORK_BYTE_24, value);
    }

    pub(crate) fn set_work_byte_25(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_25, value);
    }

    pub(crate) fn set_work_byte_26(&mut self, value: u8) {
        self.set_byte(ANCILLA_WORK_BYTE_26, value);
    }

    pub(crate) fn advance_work_byte_25(&mut self) -> u8 {
        self.add_byte(ANCILLA_WORK_BYTE_25, 1)
    }

    pub(crate) fn retreat_work_byte_25(&mut self) -> u8 {
        self.subtract_byte(ANCILLA_WORK_BYTE_25, 1)
    }

    pub(crate) fn advance_work_byte_4(&mut self) -> u8 {
        self.add_byte(ANCILLA_WORK_BYTE_4, 1)
    }
}

impl SlotBankBytes for AncillaSlotsState {
    fn bank_byte(&self, slot: usize, offset: usize) -> u8 {
        self.byte(slot, offset)
    }
    fn set_bank_byte(&mut self, slot: usize, offset: usize, value: u8) {
        self.set_byte(slot, offset, value);
    }
}
