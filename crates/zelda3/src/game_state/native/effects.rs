use crate::game_state::constants::{
    BOMBOS_BLAST_RELEASE_COUNTDOWN, BOMBOS_BLAST_RELEASE_LOCKED, BOMBOS_BLAST_X, BOMBOS_BLAST_Y,
    BOMBOS_FIRE_COLUMN_RADIUS, BOMBOS_FIRE_COLUMN_SEED_X, BOMBOS_FIRE_COLUMN_SEED_Y, BOMBOS_MODE,
    DOOR_DEBRIS_DIRECTION, DOOR_DEBRIS_X, DOOR_DEBRIS_Y, EFFECT_ANGLE_WORK,
    QUAKE_ACTIVE_BOLT_LIMIT, QUAKE_ORIGIN_X, QUAKE_ORIGIN_Y, QUAKE_PENDING_STEP,
    QUAKE_SCREEN_SHAKE_Y,
};
use crate::types::{read_le_u16, write_le_u16};

const DOOR_DEBRIS_BANK_LEN: usize = 10;
const BOMBOS_FIRE_COLUMN_SEED_SLOTS: usize = 4;
const BOMBOS_BLAST_SLOTS: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EffectState {
    pub(crate) door_debris: DoorDebrisState,
    pub(crate) angle_scratch: EffectAngleScratchState,
    pub(crate) quake_spell: QuakeSpellState,
    pub(crate) bombos_spell: BombosSpellState,
}

impl EffectState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            door_debris: DoorDebrisState::load_from_ram(ram),
            angle_scratch: EffectAngleScratchState::load_from_ram(ram),
            quake_spell: QuakeSpellState::load_from_ram(ram),
            bombos_spell: BombosSpellState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.door_debris.write_to_ram(ram);
        self.angle_scratch.write_to_ram(ram);
        self.quake_spell.write_to_ram(ram);
        self.bombos_spell.write_to_ram(ram);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EffectAngleScratchState {
    angles: [u8; EFFECT_ANGLE_WORK_LEN],
}

const EFFECT_ANGLE_WORK_LEN: usize = 9;

impl EffectAngleScratchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut angles = [0; EFFECT_ANGLE_WORK_LEN];
        for (slot, value) in angles.iter_mut().enumerate() {
            *value = ram.get(EFFECT_ANGLE_WORK + slot).copied().unwrap_or(0);
        }
        Self { angles }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[EFFECT_ANGLE_WORK..EFFECT_ANGLE_WORK + EFFECT_ANGLE_WORK_LEN]
            .copy_from_slice(&self.angles);
    }

    pub(crate) fn angle(&self, slot: usize) -> u8 {
        self.angles.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn trailing_angle(&self) -> u8 {
        self.angle(4)
    }

    pub(crate) fn radial_radius(&self) -> u8 {
        self.angle(8)
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        if let Some(angle) = self.angles.get_mut(slot) {
            *angle = value;
        }
    }

    pub(crate) fn set_angles4(&mut self, values: &[u8], start: usize) {
        for slot in 0..4 {
            self.set_angle(slot, values[start + slot]);
        }
    }

    pub(crate) fn add_angle_mod64(&mut self, slot: usize, value: u8) -> u8 {
        let angle = self.angle(slot).wrapping_add(value) & 0x3f;
        self.set_angle(slot, angle);
        angle
    }

    pub(crate) fn set_trailing_angle(&mut self, value: u8) {
        self.set_angle(4, value);
    }

    pub(crate) fn add_trailing_angle_mod64(&mut self, value: u8) -> u8 {
        self.add_angle_mod64(4, value)
    }

    pub(crate) fn set_radial_radius(&mut self, value: u8) {
        self.set_angle(8, value);
    }
}

pub(crate) struct NativeEffectAngleScratchBridgeMut<'a> {
    state: &'a mut EffectAngleScratchState,
    ram: &'a mut [u8],
}

impl<'a> NativeEffectAngleScratchBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EffectAngleScratchState, ram: &'a mut [u8]) -> Self {
        *state = EffectAngleScratchState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            EffectAngleScratchState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        self.state.set_angle(slot, value);
        self.sync();
    }

    pub(crate) fn set_angles4(&mut self, values: &[u8], start: usize) {
        self.state.set_angles4(values, start);
        self.sync();
    }

    pub(crate) fn add_angle_mod64(&mut self, slot: usize, value: u8) -> u8 {
        let angle = self.state.add_angle_mod64(slot, value);
        self.sync();
        angle
    }

    pub(crate) fn set_trailing_angle(&mut self, value: u8) {
        self.state.set_trailing_angle(value);
        self.sync();
    }

    pub(crate) fn add_trailing_angle_mod64(&mut self, value: u8) -> u8 {
        let angle = self.state.add_trailing_angle_mod64(value);
        self.sync();
        angle
    }

    pub(crate) fn set_radial_radius(&mut self, value: u8) {
        self.state.set_radial_radius(value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct QuakeSpellState {
    active_bolt_limit: u8,
    pending_step: u8,
    origin_x: u16,
    origin_y: u16,
    screen_shake_y: u16,
}

impl QuakeSpellState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            active_bolt_limit: ram.get(QUAKE_ACTIVE_BOLT_LIMIT).copied().unwrap_or(0),
            pending_step: ram.get(QUAKE_PENDING_STEP).copied().unwrap_or(0),
            origin_x: read_le_u16(ram, QUAKE_ORIGIN_X),
            origin_y: read_le_u16(ram, QUAKE_ORIGIN_Y),
            screen_shake_y: read_le_u16(ram, QUAKE_SCREEN_SHAKE_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[QUAKE_ACTIVE_BOLT_LIMIT] = self.active_bolt_limit;
        ram[QUAKE_PENDING_STEP] = self.pending_step;
        write_le_u16(ram, QUAKE_ORIGIN_X, self.origin_x);
        write_le_u16(ram, QUAKE_ORIGIN_Y, self.origin_y);
        write_le_u16(ram, QUAKE_SCREEN_SHAKE_Y, self.screen_shake_y);
    }

    pub(crate) fn active_bolt_limit(&self) -> u8 {
        self.active_bolt_limit
    }

    pub(crate) fn pending_step(&self) -> u8 {
        self.pending_step
    }

    pub(crate) fn origin_x(&self) -> u16 {
        self.origin_x
    }

    pub(crate) fn origin_y(&self) -> u16 {
        self.origin_y
    }

    pub(crate) fn screen_shake_y(&self) -> u16 {
        self.screen_shake_y
    }

    pub(crate) fn set_active_bolt_limit(&mut self, value: u8) {
        self.active_bolt_limit = value;
    }

    pub(crate) fn set_pending_step(&mut self, value: u8) {
        self.pending_step = value;
    }

    pub(crate) fn set_origin(&mut self, x: u16, y: u16) {
        self.origin_x = x;
        self.origin_y = y;
    }

    pub(crate) fn set_screen_shake_y(&mut self, value: u16) {
        self.screen_shake_y = value;
    }

    pub(crate) fn invert_screen_shake_y(&mut self) -> u16 {
        let value = self.screen_shake_y;
        self.screen_shake_y = 0u16.wrapping_sub(value);
        value
    }
}

pub(crate) struct NativeQuakeSpellBridgeMut<'a> {
    state: &'a mut QuakeSpellState,
    ram: &'a mut [u8],
}

impl<'a> NativeQuakeSpellBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut QuakeSpellState, ram: &'a mut [u8]) -> Self {
        *state = QuakeSpellState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, QuakeSpellState::load_from_ram(self.ram));
    }

    pub(crate) fn set_active_bolt_limit(&mut self, value: u8) {
        self.state.set_active_bolt_limit(value);
        self.sync();
    }

    pub(crate) fn set_pending_step(&mut self, value: u8) {
        self.state.set_pending_step(value);
        self.sync();
    }

    pub(crate) fn set_origin(&mut self, x: u16, y: u16) {
        self.state.set_origin(x, y);
        self.sync();
    }

    pub(crate) fn set_screen_shake_y(&mut self, value: u16) {
        self.state.set_screen_shake_y(value);
        self.sync();
    }

    pub(crate) fn invert_screen_shake_y(&mut self) -> u16 {
        let value = self.state.invert_screen_shake_y();
        self.sync();
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BombosSpellState {
    mode: u8,
    fire_column_radius: u8,
    blast_release_locked: u8,
    blast_release_countdown: u8,
    fire_column_seed_x: [u16; BOMBOS_FIRE_COLUMN_SEED_SLOTS],
    fire_column_seed_y: [u16; BOMBOS_FIRE_COLUMN_SEED_SLOTS],
    blast_x: [u16; BOMBOS_BLAST_SLOTS],
    blast_y: [u16; BOMBOS_BLAST_SLOTS],
}

impl BombosSpellState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            mode: ram.get(BOMBOS_MODE).copied().unwrap_or(0),
            fire_column_radius: ram.get(BOMBOS_FIRE_COLUMN_RADIUS).copied().unwrap_or(0),
            blast_release_locked: ram.get(BOMBOS_BLAST_RELEASE_LOCKED).copied().unwrap_or(0),
            blast_release_countdown: ram
                .get(BOMBOS_BLAST_RELEASE_COUNTDOWN)
                .copied()
                .unwrap_or(0),
            fire_column_seed_x: read_word_bank::<BOMBOS_FIRE_COLUMN_SEED_SLOTS>(
                ram,
                BOMBOS_FIRE_COLUMN_SEED_X,
            ),
            fire_column_seed_y: read_word_bank::<BOMBOS_FIRE_COLUMN_SEED_SLOTS>(
                ram,
                BOMBOS_FIRE_COLUMN_SEED_Y,
            ),
            blast_x: read_word_bank::<BOMBOS_BLAST_SLOTS>(ram, BOMBOS_BLAST_X),
            blast_y: read_word_bank::<BOMBOS_BLAST_SLOTS>(ram, BOMBOS_BLAST_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[BOMBOS_MODE] = self.mode;
        ram[BOMBOS_FIRE_COLUMN_RADIUS] = self.fire_column_radius;
        ram[BOMBOS_BLAST_RELEASE_LOCKED] = self.blast_release_locked;
        ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = self.blast_release_countdown;
        write_word_bank(ram, BOMBOS_FIRE_COLUMN_SEED_X, self.fire_column_seed_x);
        write_word_bank(ram, BOMBOS_FIRE_COLUMN_SEED_Y, self.fire_column_seed_y);
        write_word_bank(ram, BOMBOS_BLAST_X, self.blast_x);
        write_word_bank(ram, BOMBOS_BLAST_Y, self.blast_y);
    }

    pub(crate) fn mode(&self) -> u8 {
        self.mode
    }

    pub(crate) fn fire_column_radius(&self) -> u8 {
        self.fire_column_radius
    }

    pub(crate) fn blast_release_locked(&self) -> bool {
        self.blast_release_locked != 0
    }

    pub(crate) fn fire_column_seed_x(&self, slot: usize) -> u16 {
        self.fire_column_seed_x.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn fire_column_seed_y(&self, slot: usize) -> u16 {
        self.fire_column_seed_y.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn blast_x(&self, slot: usize) -> u16 {
        self.blast_x.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn blast_y(&self, slot: usize) -> u16 {
        self.blast_y.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn set_mode(&mut self, value: u8) {
        self.mode = value;
    }

    pub(crate) fn set_fire_column_radius(&mut self, value: u8) {
        self.fire_column_radius = value;
    }

    pub(crate) fn grow_fire_column_radius(&mut self, value: u8, limit: u8) -> u8 {
        let next = self.fire_column_radius.wrapping_add(value);
        let radius = if next >= limit { limit } else { next };
        self.fire_column_radius = radius;
        radius
    }

    pub(crate) fn set_blast_release_locked(&mut self, value: bool) {
        self.blast_release_locked = u8::from(value);
    }

    pub(crate) fn set_blast_release_countdown(&mut self, value: u8) {
        self.blast_release_countdown = value;
    }

    pub(crate) fn tick_blast_release_countdown(&mut self) -> u8 {
        self.blast_release_countdown = self.blast_release_countdown.wrapping_sub(1);
        self.blast_release_countdown
    }

    pub(crate) fn set_fire_column_seed_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(seed_x) = self.fire_column_seed_x.get_mut(slot) {
            *seed_x = x;
        }
        if let Some(seed_y) = self.fire_column_seed_y.get_mut(slot) {
            *seed_y = y;
        }
    }

    pub(crate) fn set_blast_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(blast_x) = self.blast_x.get_mut(slot) {
            *blast_x = x;
        }
        if let Some(blast_y) = self.blast_y.get_mut(slot) {
            *blast_y = y;
        }
    }
}

pub(crate) struct NativeBombosSpellBridgeMut<'a> {
    state: &'a mut BombosSpellState,
    ram: &'a mut [u8],
}

impl<'a> NativeBombosSpellBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut BombosSpellState, ram: &'a mut [u8]) -> Self {
        *state = BombosSpellState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, BombosSpellState::load_from_ram(self.ram));
    }

    pub(crate) fn set_mode(&mut self, value: u8) {
        self.state.set_mode(value);
        self.sync();
    }

    pub(crate) fn set_fire_column_radius(&mut self, value: u8) {
        self.state.set_fire_column_radius(value);
        self.sync();
    }

    pub(crate) fn grow_fire_column_radius(&mut self, value: u8, limit: u8) -> u8 {
        let radius = self.state.grow_fire_column_radius(value, limit);
        self.sync();
        radius
    }

    pub(crate) fn set_blast_release_locked(&mut self, value: bool) {
        self.state.set_blast_release_locked(value);
        self.sync();
    }

    pub(crate) fn set_blast_release_countdown(&mut self, value: u8) {
        self.state.set_blast_release_countdown(value);
        self.sync();
    }

    pub(crate) fn tick_blast_release_countdown(&mut self) -> u8 {
        let value = self.state.tick_blast_release_countdown();
        self.sync();
        value
    }

    pub(crate) fn set_fire_column_seed_position(&mut self, slot: usize, x: u16, y: u16) {
        self.state.set_fire_column_seed_position(slot, x, y);
        self.sync();
    }

    pub(crate) fn set_blast_position(&mut self, slot: usize, x: u16, y: u16) {
        self.state.set_blast_position(slot, x, y);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DoorDebrisState {
    x_bytes: [u8; DOOR_DEBRIS_BANK_LEN],
    y_bytes: [u8; DOOR_DEBRIS_BANK_LEN],
    directions: [u8; DOOR_DEBRIS_BANK_LEN],
}

impl DoorDebrisState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            x_bytes: read_bank(ram, DOOR_DEBRIS_X),
            y_bytes: read_bank(ram, DOOR_DEBRIS_Y),
            directions: read_bank(ram, DOOR_DEBRIS_DIRECTION),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        write_bank(ram, DOOR_DEBRIS_X, self.x_bytes);
        write_bank(ram, DOOR_DEBRIS_Y, self.y_bytes);
        write_bank(ram, DOOR_DEBRIS_DIRECTION, self.directions);
    }

    pub(crate) fn x(&self, slot: usize) -> u8 {
        self.x_bytes.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn y(&self, slot: usize) -> u8 {
        self.y_bytes.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn direction(&self, slot: usize) -> u8 {
        self.directions.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn x_word(&self, slot: usize) -> u16 {
        word_from_bank(self.x_bytes, slot)
    }

    pub(crate) fn y_word(&self, slot: usize) -> u16 {
        word_from_bank(self.y_bytes, slot)
    }
}

fn read_word_bank<const N: usize>(ram: &[u8], base: usize) -> [u16; N] {
    let mut bank = [0; N];
    for (slot, value) in bank.iter_mut().enumerate() {
        *value = read_le_u16(ram, base + slot * 2);
    }
    bank
}

fn write_word_bank<const N: usize>(ram: &mut [u8], base: usize, bank: [u16; N]) {
    for (slot, value) in bank.iter().copied().enumerate() {
        write_le_u16(ram, base + slot * 2, value);
    }
}

fn read_bank(ram: &[u8], base: usize) -> [u8; DOOR_DEBRIS_BANK_LEN] {
    let mut bank = [0; DOOR_DEBRIS_BANK_LEN];
    for (index, value) in bank.iter_mut().enumerate() {
        *value = ram.get(base + index).copied().unwrap_or(0);
    }
    bank
}

fn write_bank(ram: &mut [u8], base: usize, bank: [u8; DOOR_DEBRIS_BANK_LEN]) {
    for (index, value) in bank.iter().copied().enumerate() {
        ram[base + index] = value;
    }
}

fn word_from_bank(bank: [u8; DOOR_DEBRIS_BANK_LEN], slot: usize) -> u16 {
    if slot * 2 + 1 >= bank.len() {
        return 0;
    }
    let mut bytes = [0; DOOR_DEBRIS_BANK_LEN];
    bytes.copy_from_slice(&bank);
    read_le_u16(&bytes, slot * 2)
}

pub(crate) struct DoorDebrisView<'a> {
    state: &'a DoorDebrisState,
}

impl<'a> DoorDebrisView<'a> {
    pub(crate) fn new(state: &'a DoorDebrisState) -> Self {
        Self { state }
    }

    pub(crate) fn x(&self, slot: usize) -> u8 {
        self.state.x(slot)
    }

    pub(crate) fn y(&self, slot: usize) -> u8 {
        self.state.y(slot)
    }

    pub(crate) fn direction(&self, slot: usize) -> u8 {
        self.state.direction(slot)
    }

    pub(crate) fn x_word(&self, slot: usize) -> u16 {
        self.state.x_word(slot)
    }

    pub(crate) fn y_word(&self, slot: usize) -> u16 {
        self.state.y_word(slot)
    }
}

pub(crate) struct NativeDoorDebrisBridgeMut<'a> {
    state: &'a mut DoorDebrisState,
    ram: &'a mut [u8],
}

impl<'a> NativeDoorDebrisBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DoorDebrisState, ram: &'a mut [u8]) -> Self {
        *state = DoorDebrisState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_direction(&mut self, slot: usize, value: u8) {
        if let Some(direction) = self.state.directions.get_mut(slot) {
            *direction = value;
            self.sync();
        }
    }

    pub(crate) fn set_y_low_and_x_low_from_word(&mut self, slot: usize, value: u16) {
        if slot < DOOR_DEBRIS_BANK_LEN {
            self.state.y_bytes[slot] = value as u8;
            self.state.x_bytes[slot] = (value >> 8) as u8;
            self.sync();
        }
    }

    pub(crate) fn set_x_word(&mut self, slot: usize, value: u16) {
        self.write_word_to_x_bank(slot, value);
        self.sync();
    }

    pub(crate) fn set_y_word(&mut self, slot: usize, value: u16) {
        self.write_word_to_y_bank(slot, value);
        self.sync();
    }

    fn write_word_to_x_bank(&mut self, slot: usize, value: u16) {
        if slot * 2 + 1 < DOOR_DEBRIS_BANK_LEN {
            write_le_u16(&mut self.state.x_bytes, slot * 2, value);
        }
    }

    fn write_word_to_y_bank(&mut self, slot: usize, value: u16) {
        if slot * 2 + 1 < DOOR_DEBRIS_BANK_LEN {
            write_le_u16(&mut self.state.y_bytes, slot * 2, value);
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DoorDebrisState::load_from_ram(self.ram));
    }
}
