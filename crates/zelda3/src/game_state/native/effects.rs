use crate::game_state::constants::{
    BEAMOS_LASER_HISTORY_X_HI, BEAMOS_LASER_HISTORY_X_LO, BEAMOS_LASER_HISTORY_Y_HI,
    BEAMOS_LASER_HISTORY_Y_LO, BLAST_WALL_CENTER_X, BLAST_WALL_CENTER_Y, BLAST_WALL_DIRECTION,
    BLAST_WALL_ENTRY_STATE, BLAST_WALL_EXPLOSION_PHASE, BLAST_WALL_EXPLOSION_TIMER,
    BLAST_WALL_FIREBALL_TIMER, BLAST_WALL_FRAGMENT_X, BLAST_WALL_FRAGMENT_Y,
    BLAST_WALL_SECONDARY_STATE, BOMBOS_BLAST_PHASE, BOMBOS_BLAST_RELEASE_COUNTDOWN,
    BOMBOS_BLAST_RELEASE_LOCKED, BOMBOS_BLAST_TIMER, BOMBOS_BLAST_X, BOMBOS_BLAST_Y,
    BOMBOS_FIRE_COLUMN_PHASE, BOMBOS_FIRE_COLUMN_RADIAL_ANGLE, BOMBOS_FIRE_COLUMN_RADIUS,
    BOMBOS_FIRE_COLUMN_SEED_X, BOMBOS_FIRE_COLUMN_SEED_Y, BOMBOS_FIRE_COLUMN_TIMER,
    BOMBOS_FIRE_COLUMN_X_HI, BOMBOS_FIRE_COLUMN_X_LO, BOMBOS_FIRE_COLUMN_Y_HI,
    BOMBOS_FIRE_COLUMN_Y_LO, BOMBOS_MODE, DIGGING_GAME_PRIZE_ATTEMPTS, DIGGING_GAME_PRIZE_SPAWNED,
    DOOR_DEBRIS_DIRECTION, DOOR_DEBRIS_X, DOOR_DEBRIS_Y, EFFECT_ANGLE_WORK, HAPPINESS_POND_ACTIVE,
    HAPPINESS_POND_ITEM_TO_LINK, HAPPINESS_POND_STEP, HAPPINESS_POND_TIMER, HAPPINESS_POND_X_HI,
    HAPPINESS_POND_X_LO, HAPPINESS_POND_X_SUBPIXEL, HAPPINESS_POND_X_VEL, HAPPINESS_POND_Y_HI,
    HAPPINESS_POND_Y_LO, HAPPINESS_POND_Y_SUBPIXEL, HAPPINESS_POND_Y_VEL, HAPPINESS_POND_Z,
    HAPPINESS_POND_Z_SUBPIXEL, HAPPINESS_POND_Z_VEL, MOLDORM_HISTORY_X_HI, MOLDORM_HISTORY_X_LO,
    MOLDORM_HISTORY_Y_HI, MOLDORM_HISTORY_Y_LO, QUAKE_ACTIVE_BOLT_LIMIT, QUAKE_BOLT_PHASE,
    QUAKE_BOLT_TIMER, QUAKE_ORIGIN_X, QUAKE_ORIGIN_Y, QUAKE_PENDING_STEP, QUAKE_SCREEN_SHAKE_Y,
    SKULL_WOODS_FIRE_INNER_X, SKULL_WOODS_FIRE_INNER_Y, SKULL_WOODS_FIRE_OUTER_X,
    SKULL_WOODS_FIRE_OUTER_Y, SKULL_WOODS_FIRE_STARTED, SWAMOLA_HISTORY_X_HI, SWAMOLA_HISTORY_X_LO,
    SWAMOLA_HISTORY_Y_HI, SWAMOLA_HISTORY_Y_LO, SWAMOLA_TARGET_X_HI, SWAMOLA_TARGET_X_LO,
    SWAMOLA_TARGET_Y_HI, SWAMOLA_TARGET_Y_LO, TOWER_SEAL_BASE_SPARKLE_X_HI,
    TOWER_SEAL_BASE_SPARKLE_X_LO, TOWER_SEAL_BASE_SPARKLE_Y_HI, TOWER_SEAL_BASE_SPARKLE_Y_LO,
    TOWER_SEAL_CENTER_X, TOWER_SEAL_CENTER_Y, TOWER_SEAL_ORBIT_ANGLE, TOWER_SEAL_RING_RADIUS,
    TOWER_SEAL_SPARKLE_PHASE, TOWER_SEAL_SPARKLE_TIMER, TOWER_SEAL_SPARKLE_X_HI,
    TOWER_SEAL_SPARKLE_X_LO, TOWER_SEAL_SPARKLE_Y_HI, TOWER_SEAL_SPARKLE_Y_LO,
    TOWER_SEAL_WAIT_COUNTDOWN, WEATHERVANE_ANIM_TIMER, WEATHERVANE_DRAW_STATE, WEATHERVANE_X_HI,
    WEATHERVANE_X_LO, WEATHERVANE_X_VELOCITY, WEATHERVANE_Y_HI, WEATHERVANE_Y_LO,
    WEATHERVANE_Y_VELOCITY, WEATHERVANE_Z, WEATHERVANE_Z_VELOCITY,
};
use crate::types::{read_le_u16, write_le_u16};

const DOOR_DEBRIS_BANK_LEN: usize = 10;
const BOMBOS_FIRE_COLUMN_SLOTS: usize = 16;
const BOMBOS_FIRE_COLUMN_SEED_SLOTS: usize = 4;
const BOMBOS_FIRE_COLUMN_Y_LOW_RADIAL_OFFSET: usize =
    BOMBOS_FIRE_COLUMN_Y_LO - BOMBOS_FIRE_COLUMN_RADIAL_ANGLE;
const BOMBOS_BLAST_SLOTS: usize = 16;
const QUAKE_BOLT_SLOTS: usize = 5;
const TOWER_SEAL_ORBIT_SLOTS: usize = 8;
const TOWER_SEAL_SPARKLE_SLOTS: usize = 24;
const HAPPINESS_POND_RUPEE_SLOTS: usize = 10;
const WEATHER_VANE_DEBRIS_SLOTS: usize = 12;
const MOLDORM_HISTORY_SLOTS: usize = 128;
const SWAMOLA_TARGET_SLOTS: usize = 6;
const SWAMOLA_HISTORY_SLOTS: usize = 192;
const BEAMOS_LASER_HISTORY_SLOTS: usize = 128;
const LANMOLA_SEGMENT_MOTION_SLOTS: usize = 192;
const ENTRANCE_EFFECT_PHASE_SLOTS: usize = 8;
const ENTRANCE_EFFECT_POSITION_SLOTS: usize = 8;
const BLAST_WALL_FIREBALL_SLOTS: usize = 16;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EffectState {
    pub(crate) door_debris: DoorDebrisState,
    pub(crate) angle_scratch: EffectAngleScratchState,
    pub(crate) quake_spell: QuakeSpellState,
    pub(crate) quake_bolts: QuakeBoltState,
    pub(crate) bombos_spell: BombosSpellState,
    pub(crate) tower_seal: TowerSealState,
    pub(crate) happiness_pond_rupees: HappinessPondRupeesState,
    pub(crate) weather_vane_debris: WeatherVaneDebrisState,
    pub(crate) sprite_histories: SpriteHistoryScratchState,
    pub(crate) entrance_effects: EntranceEffectState,
    pub(crate) digging_game_prize: DiggingGamePrizeState,
}

impl EffectState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            door_debris: DoorDebrisState::load_from_ram(ram),
            angle_scratch: EffectAngleScratchState::load_from_ram(ram),
            quake_spell: QuakeSpellState::load_from_ram(ram),
            quake_bolts: QuakeBoltState::load_from_ram(ram),
            bombos_spell: BombosSpellState::load_from_ram(ram),
            tower_seal: TowerSealState::load_from_ram(ram),
            happiness_pond_rupees: HappinessPondRupeesState::load_from_ram(ram),
            weather_vane_debris: WeatherVaneDebrisState::load_from_ram(ram),
            sprite_histories: SpriteHistoryScratchState::load_from_ram(ram),
            entrance_effects: EntranceEffectState::load_from_ram(ram),
            digging_game_prize: DiggingGamePrizeState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.door_debris.write_to_ram(ram);
        self.angle_scratch.write_to_ram(ram);
        self.quake_spell.write_to_ram(ram);
        self.quake_bolts.write_to_ram(ram);
        self.bombos_spell.write_to_ram(ram);
        self.tower_seal.write_to_ram(ram);
        self.happiness_pond_rupees.write_to_ram(ram);
        self.weather_vane_debris.write_to_ram(ram);
        // Sprite-history scratch banks overlap entrance/digging effect scratch banks.
        // Project them explicitly through SpriteHistoryScratchState when that domain is active.
        self.entrance_effects.write_to_ram(ram);
        self.digging_game_prize.write_to_ram(ram);
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
pub(crate) struct QuakeBoltState {
    timers: [u8; QUAKE_BOLT_SLOTS],
    phases: [u8; QUAKE_BOLT_SLOTS],
}

impl QuakeBoltState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..QUAKE_BOLT_SLOTS {
            state.timers[slot] = ram.get(QUAKE_BOLT_TIMER + slot).copied().unwrap_or(0);
            state.phases[slot] = ram.get(QUAKE_BOLT_PHASE + slot).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[QUAKE_BOLT_TIMER..QUAKE_BOLT_TIMER + QUAKE_BOLT_SLOTS].copy_from_slice(&self.timers);
        ram[QUAKE_BOLT_PHASE..QUAKE_BOLT_PHASE + QUAKE_BOLT_SLOTS].copy_from_slice(&self.phases);
    }

    pub(crate) fn slot(&self, slot: usize) -> QuakeBoltSlotState {
        QuakeBoltSlotState {
            timer: self.timer(slot),
            phase: self.phase(slot),
        }
    }

    fn timer(&self, slot: usize) -> u8 {
        self.timers.get(slot).copied().unwrap_or(0)
    }

    fn phase(&self, slot: usize) -> u8 {
        self.phases.get(slot).copied().unwrap_or(0)
    }

    fn set_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_timer(&mut self, slot: usize) -> u8 {
        let value = self.timer(slot).wrapping_sub(1);
        self.set_timer(slot, value);
        value
    }

    fn set_phase(&mut self, slot: usize, value: u8) {
        if let Some(phase) = self.phases.get_mut(slot) {
            *phase = value;
        }
    }

    fn advance_phase(&mut self, slot: usize) -> u8 {
        let value = self.phase(slot).wrapping_add(1);
        self.set_phase(slot, value);
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct QuakeBoltSlotState {
    timer: u8,
    phase: u8,
}

impl QuakeBoltSlotState {
    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }

    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }
}

pub(crate) struct NativeQuakeBoltBridgeMut<'a> {
    state: &'a mut QuakeBoltState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeQuakeBoltBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut QuakeBoltState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, QuakeBoltState::load_from_ram(self.ram));
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.state.tick_timer(self.slot);
        self.sync();
        value
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.state.advance_phase(self.slot);
        self.sync();
        value
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
    fire_column_timers: [u8; BOMBOS_FIRE_COLUMN_SLOTS],
    fire_column_phases: [u8; BOMBOS_FIRE_COLUMN_SLOTS],
    fire_column_radial_angles: [u8; BOMBOS_FIRE_COLUMN_SLOTS],
    fire_column_x: [u16; BOMBOS_FIRE_COLUMN_SLOTS],
    fire_column_y: [u16; BOMBOS_FIRE_COLUMN_SLOTS],
    blast_phases: [u8; BOMBOS_BLAST_SLOTS],
    blast_timers: [u8; BOMBOS_BLAST_SLOTS],
    fire_column_seed_x: [u16; BOMBOS_FIRE_COLUMN_SEED_SLOTS],
    fire_column_seed_y: [u16; BOMBOS_FIRE_COLUMN_SEED_SLOTS],
    blast_x: [u16; BOMBOS_BLAST_SLOTS],
    blast_y: [u16; BOMBOS_BLAST_SLOTS],
}

impl BombosSpellState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self {
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
            ..Self::default()
        };
        for slot in 0..BOMBOS_FIRE_COLUMN_SLOTS {
            state.fire_column_timers[slot] = ram
                .get(BOMBOS_FIRE_COLUMN_TIMER + slot)
                .copied()
                .unwrap_or(0);
            state.fire_column_phases[slot] = ram
                .get(BOMBOS_FIRE_COLUMN_PHASE + slot)
                .copied()
                .unwrap_or(0);
            state.fire_column_radial_angles[slot] = ram
                .get(BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + slot)
                .copied()
                .unwrap_or(0);
            state.fire_column_x[slot] =
                read_split_u16(ram, BOMBOS_FIRE_COLUMN_X_LO, BOMBOS_FIRE_COLUMN_X_HI, slot);
            state.fire_column_y[slot] =
                read_split_u16(ram, BOMBOS_FIRE_COLUMN_Y_LO, BOMBOS_FIRE_COLUMN_Y_HI, slot);
        }
        for slot in 0..BOMBOS_BLAST_SLOTS {
            state.blast_phases[slot] = ram.get(BOMBOS_BLAST_PHASE + slot).copied().unwrap_or(0);
            state.blast_timers[slot] = ram.get(BOMBOS_BLAST_TIMER + slot).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[BOMBOS_MODE] = self.mode;
        ram[BOMBOS_FIRE_COLUMN_RADIUS] = self.fire_column_radius;
        ram[BOMBOS_BLAST_RELEASE_LOCKED] = self.blast_release_locked;
        ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = self.blast_release_countdown;
        for slot in 0..BOMBOS_FIRE_COLUMN_SLOTS {
            ram[BOMBOS_FIRE_COLUMN_TIMER + slot] = self.fire_column_timers[slot];
            ram[BOMBOS_FIRE_COLUMN_PHASE + slot] = self.fire_column_phases[slot];
            ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + slot] = self.fire_column_radial_angles[slot];
            write_split_u16(
                ram,
                BOMBOS_FIRE_COLUMN_X_LO,
                BOMBOS_FIRE_COLUMN_X_HI,
                slot,
                self.fire_column_x[slot],
            );
            write_split_u16(
                ram,
                BOMBOS_FIRE_COLUMN_Y_LO,
                BOMBOS_FIRE_COLUMN_Y_HI,
                slot,
                self.fire_column_y[slot],
            );
        }
        for slot in 0..BOMBOS_BLAST_SLOTS {
            ram[BOMBOS_BLAST_PHASE + slot] = self.blast_phases[slot];
            ram[BOMBOS_BLAST_TIMER + slot] = self.blast_timers[slot];
        }
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

    pub(crate) fn fire_column(&self, slot: usize) -> BombosFireColumnState {
        BombosFireColumnState {
            timer: self.fire_column_timers.get(slot).copied().unwrap_or(0),
            phase: self.fire_column_phases.get(slot).copied().unwrap_or(0),
            radial_angle: self
                .fire_column_radial_angles
                .get(slot)
                .copied()
                .unwrap_or(0),
            x: self.fire_column_x.get(slot).copied().unwrap_or(0),
            y: self.fire_column_y.get(slot).copied().unwrap_or(0),
        }
    }

    pub(crate) fn blast(&self, slot: usize) -> BombosBlastState {
        BombosBlastState {
            phase: self.blast_phases.get(slot).copied().unwrap_or(0),
            timer: self.blast_timers.get(slot).copied().unwrap_or(0),
        }
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

    fn set_fire_column_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.fire_column_timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_fire_column_timer(&mut self, slot: usize) -> u8 {
        let value = self
            .fire_column_timers
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_sub(1);
        self.set_fire_column_timer(slot, value);
        value
    }

    fn set_fire_column_phase(&mut self, slot: usize, value: u8) {
        if let Some(phase) = self.fire_column_phases.get_mut(slot) {
            *phase = value;
        }
    }

    fn advance_fire_column_phase(&mut self, slot: usize) -> u8 {
        let value = self
            .fire_column_phases
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_add(1);
        self.set_fire_column_phase(slot, value);
        value
    }

    fn set_fire_column_radial_angle(&mut self, slot: usize, value: u8) {
        if let Some(angle) = self.fire_column_radial_angles.get_mut(slot) {
            *angle = value;
        }
        if let Some(position_slot) = slot.checked_sub(BOMBOS_FIRE_COLUMN_Y_LOW_RADIAL_OFFSET) {
            if let Some(y) = self.fire_column_y.get_mut(position_slot) {
                *y = (*y & 0xff00) | u16::from(value);
            }
        }
    }

    fn add_fire_column_radial_angle(&mut self, slot: usize, value: u8) -> u8 {
        let angle = self
            .fire_column_radial_angles
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_add(value);
        self.set_fire_column_radial_angle(slot, angle);
        angle
    }

    fn set_fire_column_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(position) = self.fire_column_x.get_mut(slot) {
            *position = x;
        }
        if let Some(position) = self.fire_column_y.get_mut(slot) {
            *position = y;
        }
        let radial_slot = slot + BOMBOS_FIRE_COLUMN_Y_LOW_RADIAL_OFFSET;
        if let Some(angle) = self.fire_column_radial_angles.get_mut(radial_slot) {
            *angle = y as u8;
        }
    }

    fn set_blast_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.blast_timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_blast_timer(&mut self, slot: usize) -> u8 {
        let value = self
            .blast_timers
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_sub(1);
        self.set_blast_timer(slot, value);
        value
    }

    fn set_blast_phase(&mut self, slot: usize, value: u8) {
        if let Some(phase) = self.blast_phases.get_mut(slot) {
            *phase = value;
        }
    }

    fn advance_blast_phase(&mut self, slot: usize) -> u8 {
        let value = self
            .blast_phases
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_add(1);
        self.set_blast_phase(slot, value);
        value
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BombosFireColumnState {
    timer: u8,
    phase: u8,
    radial_angle: u8,
    x: u16,
    y: u16,
}

impl BombosFireColumnState {
    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }

    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }

    pub(crate) fn radial_angle(&self) -> u8 {
        self.radial_angle
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BombosBlastState {
    phase: u8,
    timer: u8,
}

impl BombosBlastState {
    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }

    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }
}

pub(crate) struct NativeBombosSpellBridgeMut<'a> {
    state: &'a mut BombosSpellState,
    ram: &'a mut [u8],
}

impl<'a> NativeBombosSpellBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut BombosSpellState, ram: &'a mut [u8]) -> Self {
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

pub(crate) struct NativeBombosFireColumnBridgeMut<'a> {
    state: &'a mut BombosSpellState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBombosFireColumnBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut BombosSpellState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, BombosSpellState::load_from_ram(self.ram));
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_fire_column_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.state.tick_fire_column_timer(self.slot);
        self.sync();
        value
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_fire_column_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.state.advance_fire_column_phase(self.slot);
        self.sync();
        value
    }

    pub(crate) fn add_radial_angle(&mut self, value: u8) -> u8 {
        let angle = self.state.add_fire_column_radial_angle(self.slot, value);
        self.sync();
        angle
    }

    pub(crate) fn set_radial_angle(&mut self, value: u8) {
        self.state.set_fire_column_radial_angle(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_fire_column_position(self.slot, x, y);
        self.sync();
    }
}

pub(crate) struct NativeBombosBlastBridgeMut<'a> {
    state: &'a mut BombosSpellState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBombosBlastBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut BombosSpellState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, BombosSpellState::load_from_ram(self.ram));
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_blast_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let value = self.state.advance_blast_phase(self.slot);
        self.sync();
        value
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_blast_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.state.tick_blast_timer(self.slot);
        self.sync();
        value
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HappinessPondRupeesState {
    active: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    y_low: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    y_high: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    x_low: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    x_high: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    z: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    y_velocity: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    x_velocity: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    z_velocity: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    y_subpixel: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    x_subpixel: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    z_subpixel: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    item_to_link: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    timer: [u8; HAPPINESS_POND_RUPEE_SLOTS],
    step: [u8; HAPPINESS_POND_RUPEE_SLOTS],
}

impl HappinessPondRupeesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..HAPPINESS_POND_RUPEE_SLOTS {
            state.active[slot] = ram.get(HAPPINESS_POND_ACTIVE + slot).copied().unwrap_or(0);
            state.y_low[slot] = ram.get(HAPPINESS_POND_Y_LO + slot).copied().unwrap_or(0);
            state.y_high[slot] = ram.get(HAPPINESS_POND_Y_HI + slot).copied().unwrap_or(0);
            state.x_low[slot] = ram.get(HAPPINESS_POND_X_LO + slot).copied().unwrap_or(0);
            state.x_high[slot] = ram.get(HAPPINESS_POND_X_HI + slot).copied().unwrap_or(0);
            state.z[slot] = ram.get(HAPPINESS_POND_Z + slot).copied().unwrap_or(0);
            state.y_velocity[slot] = ram.get(HAPPINESS_POND_Y_VEL + slot).copied().unwrap_or(0);
            state.x_velocity[slot] = ram.get(HAPPINESS_POND_X_VEL + slot).copied().unwrap_or(0);
            state.z_velocity[slot] = ram.get(HAPPINESS_POND_Z_VEL + slot).copied().unwrap_or(0);
            state.y_subpixel[slot] = ram
                .get(HAPPINESS_POND_Y_SUBPIXEL + slot)
                .copied()
                .unwrap_or(0);
            state.x_subpixel[slot] = ram
                .get(HAPPINESS_POND_X_SUBPIXEL + slot)
                .copied()
                .unwrap_or(0);
            state.z_subpixel[slot] = ram
                .get(HAPPINESS_POND_Z_SUBPIXEL + slot)
                .copied()
                .unwrap_or(0);
            state.item_to_link[slot] = ram
                .get(HAPPINESS_POND_ITEM_TO_LINK + slot)
                .copied()
                .unwrap_or(0);
            state.timer[slot] = ram.get(HAPPINESS_POND_TIMER + slot).copied().unwrap_or(0);
            state.step[slot] = ram.get(HAPPINESS_POND_STEP + slot).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for slot in 0..HAPPINESS_POND_RUPEE_SLOTS {
            ram[HAPPINESS_POND_ACTIVE + slot] = self.active[slot];
            ram[HAPPINESS_POND_Y_LO + slot] = self.y_low[slot];
            ram[HAPPINESS_POND_Y_HI + slot] = self.y_high[slot];
            ram[HAPPINESS_POND_X_LO + slot] = self.x_low[slot];
            ram[HAPPINESS_POND_X_HI + slot] = self.x_high[slot];
            ram[HAPPINESS_POND_Z + slot] = self.z[slot];
            ram[HAPPINESS_POND_Y_VEL + slot] = self.y_velocity[slot];
            ram[HAPPINESS_POND_X_VEL + slot] = self.x_velocity[slot];
            ram[HAPPINESS_POND_Z_VEL + slot] = self.z_velocity[slot];
            ram[HAPPINESS_POND_Y_SUBPIXEL + slot] = self.y_subpixel[slot];
            ram[HAPPINESS_POND_X_SUBPIXEL + slot] = self.x_subpixel[slot];
            ram[HAPPINESS_POND_Z_SUBPIXEL + slot] = self.z_subpixel[slot];
            ram[HAPPINESS_POND_ITEM_TO_LINK + slot] = self.item_to_link[slot];
            ram[HAPPINESS_POND_TIMER + slot] = self.timer[slot];
            ram[HAPPINESS_POND_STEP + slot] = self.step[slot];
        }
    }

    pub(crate) fn rupee(&self, slot: usize) -> HappinessPondRupeeSlotState {
        HappinessPondRupeeSlotState {
            active: self.active.get(slot).copied().unwrap_or(0),
            snapshot: self.snapshot(slot),
        }
    }

    fn clear(&mut self, slot: usize) {
        if let Some(active) = self.active.get_mut(slot) {
            *active = 0;
        }
    }

    fn initialize(
        &mut self,
        slot: usize,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
    ) {
        if slot >= HAPPINESS_POND_RUPEE_SLOTS {
            return;
        }
        self.active[slot] = 1;
        self.z_velocity[slot] = z_velocity;
        self.y_velocity[slot] = y_velocity;
        self.x_velocity[slot] = x_velocity;
        self.z[slot] = 0;
        self.step[slot] = 0;
        self.timer[slot] = 16;
        self.item_to_link[slot] = 53;
        self.x_low[slot] = x as u8;
        self.x_high[slot] = (x >> 8) as u8;
        self.y_low[slot] = y as u8;
        self.y_high[slot] = (y >> 8) as u8;
    }

    fn store_snapshot(&mut self, slot: usize, state: HappinessPondRupeeSnapshot) {
        if slot >= HAPPINESS_POND_RUPEE_SLOTS {
            return;
        }
        self.y_low[slot] = state.y_low;
        self.y_high[slot] = state.y_high;
        self.x_low[slot] = state.x_low;
        self.x_high[slot] = state.x_high;
        self.z[slot] = state.z;
        self.y_velocity[slot] = state.y_velocity;
        self.x_velocity[slot] = state.x_velocity;
        self.z_velocity[slot] = state.z_velocity;
        self.y_subpixel[slot] = state.y_subpixel;
        self.x_subpixel[slot] = state.x_subpixel;
        self.z_subpixel[slot] = state.z_subpixel;
        self.item_to_link[slot] = state.item_to_link;
        self.timer[slot] = state.timer;
        self.step[slot] = state.step;
    }

    fn snapshot(&self, slot: usize) -> HappinessPondRupeeSnapshot {
        HappinessPondRupeeSnapshot {
            y_low: self.y_low.get(slot).copied().unwrap_or(0),
            y_high: self.y_high.get(slot).copied().unwrap_or(0),
            x_low: self.x_low.get(slot).copied().unwrap_or(0),
            x_high: self.x_high.get(slot).copied().unwrap_or(0),
            z: self.z.get(slot).copied().unwrap_or(0),
            y_velocity: self.y_velocity.get(slot).copied().unwrap_or(0),
            x_velocity: self.x_velocity.get(slot).copied().unwrap_or(0),
            z_velocity: self.z_velocity.get(slot).copied().unwrap_or(0),
            y_subpixel: self.y_subpixel.get(slot).copied().unwrap_or(0),
            x_subpixel: self.x_subpixel.get(slot).copied().unwrap_or(0),
            z_subpixel: self.z_subpixel.get(slot).copied().unwrap_or(0),
            item_to_link: self.item_to_link.get(slot).copied().unwrap_or(0),
            timer: self.timer.get(slot).copied().unwrap_or(0).saturating_sub(1),
            step: self.step.get(slot).copied().unwrap_or(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HappinessPondRupeeSlotState {
    active: u8,
    snapshot: HappinessPondRupeeSnapshot,
}

impl HappinessPondRupeeSlotState {
    pub(crate) fn is_active(&self) -> bool {
        self.active != 0
    }

    pub(crate) fn step(&self) -> u8 {
        self.snapshot.step
    }

    pub(crate) fn snapshot(&self) -> HappinessPondRupeeSnapshot {
        self.snapshot
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HappinessPondRupeeSnapshot {
    pub(crate) y_low: u8,
    pub(crate) y_high: u8,
    pub(crate) x_low: u8,
    pub(crate) x_high: u8,
    pub(crate) z: u8,
    pub(crate) y_velocity: u8,
    pub(crate) x_velocity: u8,
    pub(crate) z_velocity: u8,
    pub(crate) y_subpixel: u8,
    pub(crate) x_subpixel: u8,
    pub(crate) z_subpixel: u8,
    pub(crate) item_to_link: u8,
    pub(crate) timer: u8,
    pub(crate) step: u8,
}

pub(crate) struct NativeHappinessPondRupeeBridgeMut<'a> {
    state: &'a mut HappinessPondRupeesState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeHappinessPondRupeeBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut HappinessPondRupeesState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            HappinessPondRupeesState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear(&mut self) {
        self.state.clear(self.slot);
        self.sync();
    }

    pub(crate) fn initialize(
        &mut self,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
    ) {
        self.state
            .initialize(self.slot, x, y, x_velocity, y_velocity, z_velocity);
        self.sync();
    }

    pub(crate) fn store_snapshot(&mut self, state: HappinessPondRupeeSnapshot) {
        self.state.store_snapshot(self.slot, state);
        self.sync();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WeatherVaneDebrisState {
    y: [u16; WEATHER_VANE_DEBRIS_SLOTS],
    x: [u16; WEATHER_VANE_DEBRIS_SLOTS],
    z: [u8; WEATHER_VANE_DEBRIS_SLOTS],
    y_velocity: [u8; WEATHER_VANE_DEBRIS_SLOTS],
    x_velocity: [u8; WEATHER_VANE_DEBRIS_SLOTS],
    z_velocity: [u8; WEATHER_VANE_DEBRIS_SLOTS],
    animation_timer: [u8; WEATHER_VANE_DEBRIS_SLOTS],
    draw_state: [u8; WEATHER_VANE_DEBRIS_SLOTS],
}

impl WeatherVaneDebrisState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..WEATHER_VANE_DEBRIS_SLOTS {
            state.y[slot] = read_split_u16(ram, WEATHERVANE_Y_LO, WEATHERVANE_Y_HI, slot);
            state.x[slot] = read_split_u16(ram, WEATHERVANE_X_LO, WEATHERVANE_X_HI, slot);
            state.z[slot] = ram.get(WEATHERVANE_Z + slot).copied().unwrap_or(0);
            state.y_velocity[slot] = ram.get(WEATHERVANE_Y_VELOCITY + slot).copied().unwrap_or(0);
            state.x_velocity[slot] = ram.get(WEATHERVANE_X_VELOCITY + slot).copied().unwrap_or(0);
            state.z_velocity[slot] = ram.get(WEATHERVANE_Z_VELOCITY + slot).copied().unwrap_or(0);
            state.animation_timer[slot] =
                ram.get(WEATHERVANE_ANIM_TIMER + slot).copied().unwrap_or(0);
            state.draw_state[slot] = ram.get(WEATHERVANE_DRAW_STATE + slot).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for slot in 0..WEATHER_VANE_DEBRIS_SLOTS {
            write_split_u16(ram, WEATHERVANE_Y_LO, WEATHERVANE_Y_HI, slot, self.y[slot]);
            write_split_u16(ram, WEATHERVANE_X_LO, WEATHERVANE_X_HI, slot, self.x[slot]);
            ram[WEATHERVANE_Z + slot] = self.z[slot];
            ram[WEATHERVANE_Y_VELOCITY + slot] = self.y_velocity[slot];
            ram[WEATHERVANE_X_VELOCITY + slot] = self.x_velocity[slot];
            ram[WEATHERVANE_Z_VELOCITY + slot] = self.z_velocity[slot];
            ram[WEATHERVANE_ANIM_TIMER + slot] = self.animation_timer[slot];
            ram[WEATHERVANE_DRAW_STATE + slot] = self.draw_state[slot];
        }
    }

    pub(crate) fn debris(&self, slot: usize) -> WeatherVaneDebrisSlotState {
        WeatherVaneDebrisSlotState {
            snapshot: self.snapshot(slot),
        }
    }

    fn initialize(
        &mut self,
        slot: usize,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
        z: u8,
        draw_state: u8,
    ) {
        if slot >= WEATHER_VANE_DEBRIS_SLOTS {
            return;
        }
        self.y_velocity[slot] = y_velocity;
        self.x_velocity[slot] = x_velocity;
        self.z_velocity[slot] = z_velocity;
        self.y[slot] = y;
        self.x[slot] = x;
        self.z[slot] = z;
        self.animation_timer[slot] = 1;
        self.draw_state[slot] = draw_state;
    }

    fn tick_animation(&mut self, slot: usize) -> u8 {
        if slot >= WEATHER_VANE_DEBRIS_SLOTS {
            return 0;
        }
        let timer = self.animation_timer[slot].wrapping_sub(1);
        self.animation_timer[slot] = timer;
        if (timer as i8).is_negative() {
            self.animation_timer[slot] = 1;
            self.draw_state[slot] ^= 1;
        }
        self.draw_state[slot]
    }

    fn tick_z_velocity(&mut self, slot: usize) -> u8 {
        if slot >= WEATHER_VANE_DEBRIS_SLOTS {
            return 0;
        }
        self.z_velocity[slot] = self.z_velocity[slot].wrapping_sub(1);
        self.z_velocity[slot]
    }

    fn mark_finished_if_landed(&mut self, slot: usize, z: u8) {
        if slot < WEATHER_VANE_DEBRIS_SLOTS && z >= 0xf0 {
            self.draw_state[slot] = 0xff;
        }
    }

    fn save_position(&mut self, slot: usize, x: u16, y: u16, z: u8) {
        if slot >= WEATHER_VANE_DEBRIS_SLOTS {
            return;
        }
        self.y[slot] = y;
        self.x[slot] = x;
        self.z[slot] = z;
    }

    fn snapshot(&self, slot: usize) -> WeatherVaneDebrisSnapshot {
        WeatherVaneDebrisSnapshot {
            y: self.y.get(slot).copied().unwrap_or(0),
            x: self.x.get(slot).copied().unwrap_or(0),
            z: self.z.get(slot).copied().unwrap_or(0),
            y_velocity: self.y_velocity.get(slot).copied().unwrap_or(0),
            x_velocity: self.x_velocity.get(slot).copied().unwrap_or(0),
            z_velocity: self.z_velocity.get(slot).copied().unwrap_or(0),
            draw_state: self.draw_state.get(slot).copied().unwrap_or(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WeatherVaneDebrisSlotState {
    snapshot: WeatherVaneDebrisSnapshot,
}

impl WeatherVaneDebrisSlotState {
    pub(crate) fn is_finished(&self) -> bool {
        self.snapshot.draw_state == 0xff
    }

    pub(crate) fn snapshot(&self) -> WeatherVaneDebrisSnapshot {
        self.snapshot
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WeatherVaneDebrisSnapshot {
    pub(crate) y: u16,
    pub(crate) x: u16,
    pub(crate) z: u8,
    pub(crate) y_velocity: u8,
    pub(crate) x_velocity: u8,
    pub(crate) z_velocity: u8,
    pub(crate) draw_state: u8,
}

pub(crate) struct NativeWeatherVaneDebrisBridgeMut<'a> {
    state: &'a mut WeatherVaneDebrisState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeWeatherVaneDebrisBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut WeatherVaneDebrisState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, WeatherVaneDebrisState::load_from_ram(self.ram));
    }

    pub(crate) fn initialize(
        &mut self,
        x: u16,
        y: u16,
        x_velocity: u8,
        y_velocity: u8,
        z_velocity: u8,
        z: u8,
        draw_state: u8,
    ) {
        self.state.initialize(
            self.slot, x, y, x_velocity, y_velocity, z_velocity, z, draw_state,
        );
        self.sync();
    }

    pub(crate) fn tick_animation(&mut self) -> u8 {
        let draw_state = self.state.tick_animation(self.slot);
        self.sync();
        draw_state
    }

    pub(crate) fn tick_z_velocity(&mut self) -> u8 {
        let z_velocity = self.state.tick_z_velocity(self.slot);
        self.sync();
        z_velocity
    }

    pub(crate) fn mark_finished_if_landed(&mut self, z: u8) {
        self.state.mark_finished_if_landed(self.slot, z);
        self.sync();
    }

    pub(crate) fn save_position(&mut self, x: u16, y: u16, z: u8) {
        self.state.save_position(self.slot, x, y, z);
        self.sync();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteHistoryScratchState {
    moldorm_x: Vec<u16>,
    moldorm_y: Vec<u16>,
    swamola_target_x: Vec<u16>,
    swamola_target_y: Vec<u16>,
    swamola_history_x: Vec<u16>,
    swamola_history_y: Vec<u16>,
    beamos_laser_x: Vec<u16>,
    beamos_laser_y: Vec<u16>,
    lanmola_z_offsets: Vec<u8>,
    lanmola_directions: Vec<u8>,
}

impl SpriteHistoryScratchState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            moldorm_x: read_split_word_bank(
                ram,
                MOLDORM_HISTORY_X_LO,
                MOLDORM_HISTORY_X_HI,
                MOLDORM_HISTORY_SLOTS,
            ),
            moldorm_y: read_split_word_bank(
                ram,
                MOLDORM_HISTORY_Y_LO,
                MOLDORM_HISTORY_Y_HI,
                MOLDORM_HISTORY_SLOTS,
            ),
            swamola_target_x: read_split_word_bank(
                ram,
                SWAMOLA_TARGET_X_LO,
                SWAMOLA_TARGET_X_HI,
                SWAMOLA_TARGET_SLOTS,
            ),
            swamola_target_y: read_split_word_bank(
                ram,
                SWAMOLA_TARGET_Y_LO,
                SWAMOLA_TARGET_Y_HI,
                SWAMOLA_TARGET_SLOTS,
            ),
            swamola_history_x: read_split_word_bank(
                ram,
                SWAMOLA_HISTORY_X_LO,
                SWAMOLA_HISTORY_X_HI,
                SWAMOLA_HISTORY_SLOTS,
            ),
            swamola_history_y: read_split_word_bank(
                ram,
                SWAMOLA_HISTORY_Y_LO,
                SWAMOLA_HISTORY_Y_HI,
                SWAMOLA_HISTORY_SLOTS,
            ),
            beamos_laser_x: read_split_word_bank(
                ram,
                BEAMOS_LASER_HISTORY_X_LO,
                BEAMOS_LASER_HISTORY_X_HI,
                BEAMOS_LASER_HISTORY_SLOTS,
            ),
            beamos_laser_y: read_split_word_bank(
                ram,
                BEAMOS_LASER_HISTORY_Y_LO,
                BEAMOS_LASER_HISTORY_Y_HI,
                BEAMOS_LASER_HISTORY_SLOTS,
            ),
            lanmola_z_offsets: read_byte_bank(
                ram,
                BEAMOS_LASER_HISTORY_X_HI,
                LANMOLA_SEGMENT_MOTION_SLOTS,
            ),
            lanmola_directions: read_byte_bank(
                ram,
                BEAMOS_LASER_HISTORY_Y_HI,
                LANMOLA_SEGMENT_MOTION_SLOTS,
            ),
        }
    }

    pub(crate) fn write_moldorm_history_to_ram(&self, ram: &mut [u8]) {
        write_split_word_bank(
            ram,
            MOLDORM_HISTORY_X_LO,
            MOLDORM_HISTORY_X_HI,
            &self.moldorm_x,
        );
        write_split_word_bank(
            ram,
            MOLDORM_HISTORY_Y_LO,
            MOLDORM_HISTORY_Y_HI,
            &self.moldorm_y,
        );
    }

    pub(crate) fn write_swamola_target_to_ram(&self, ram: &mut [u8]) {
        write_split_word_bank(
            ram,
            SWAMOLA_TARGET_X_LO,
            SWAMOLA_TARGET_X_HI,
            &self.swamola_target_x,
        );
        write_split_word_bank(
            ram,
            SWAMOLA_TARGET_Y_LO,
            SWAMOLA_TARGET_Y_HI,
            &self.swamola_target_y,
        );
    }

    pub(crate) fn write_swamola_history_to_ram(&self, ram: &mut [u8]) {
        write_split_word_bank(
            ram,
            SWAMOLA_HISTORY_X_LO,
            SWAMOLA_HISTORY_X_HI,
            &self.swamola_history_x,
        );
        write_split_word_bank(
            ram,
            SWAMOLA_HISTORY_Y_LO,
            SWAMOLA_HISTORY_Y_HI,
            &self.swamola_history_y,
        );
    }

    pub(crate) fn write_beamos_laser_history_to_ram(&self, ram: &mut [u8]) {
        write_split_word_bank(
            ram,
            BEAMOS_LASER_HISTORY_X_LO,
            BEAMOS_LASER_HISTORY_X_HI,
            &self.beamos_laser_x,
        );
        write_split_word_bank(
            ram,
            BEAMOS_LASER_HISTORY_Y_LO,
            BEAMOS_LASER_HISTORY_Y_HI,
            &self.beamos_laser_y,
        );
    }

    pub(crate) fn write_lanmola_segment_motion_to_ram(&self, ram: &mut [u8]) {
        write_byte_bank(ram, BEAMOS_LASER_HISTORY_X_HI, &self.lanmola_z_offsets);
        write_byte_bank(ram, BEAMOS_LASER_HISTORY_Y_HI, &self.lanmola_directions);
    }

    pub(crate) fn moldorm_history(&self, slot: usize) -> HistoryPositionState {
        history_position(&self.moldorm_x, &self.moldorm_y, slot)
    }

    pub(crate) fn swamola_target(&self, slot: usize) -> HistoryPositionState {
        history_position(&self.swamola_target_x, &self.swamola_target_y, slot)
    }

    pub(crate) fn swamola_history(&self, slot: usize) -> HistoryPositionState {
        history_position(&self.swamola_history_x, &self.swamola_history_y, slot)
    }

    pub(crate) fn beamos_laser_history(&self, slot: usize) -> HistoryPositionState {
        history_position(&self.beamos_laser_x, &self.beamos_laser_y, slot)
    }

    pub(crate) fn lanmola_segment_motion(&self, slot: usize) -> LanmolaSegmentMotionState {
        LanmolaSegmentMotionState {
            z_offset: self.lanmola_z_offsets.get(slot).copied().unwrap_or(0),
            direction: self.lanmola_directions.get(slot).copied().unwrap_or(0),
        }
    }

    fn set_moldorm_position(&mut self, slot: usize, x: u16, y: u16) {
        set_history_position(&mut self.moldorm_x, &mut self.moldorm_y, slot, x, y);
    }

    fn set_moldorm_low_position(&mut self, slot: usize, x_low: u8, y_low: u8) {
        set_history_low_position(&mut self.moldorm_x, &mut self.moldorm_y, slot, x_low, y_low);
    }

    fn set_swamola_target_position(&mut self, slot: usize, x: u16, y: u16) {
        set_history_position(
            &mut self.swamola_target_x,
            &mut self.swamola_target_y,
            slot,
            x,
            y,
        );
    }

    fn set_swamola_target_x_low(&mut self, slot: usize, value: u8) {
        set_word_low_byte(&mut self.swamola_target_x, slot, value);
    }

    fn set_swamola_target_y_low(&mut self, slot: usize, value: u8) {
        set_word_low_byte(&mut self.swamola_target_y, slot, value);
    }

    fn set_swamola_history_position(&mut self, slot: usize, x: u16, y: u16) {
        set_history_position(
            &mut self.swamola_history_x,
            &mut self.swamola_history_y,
            slot,
            x,
            y,
        );
    }

    fn set_beamos_laser_position(&mut self, slot: usize, x: u16, y: u16) {
        set_history_position(
            &mut self.beamos_laser_x,
            &mut self.beamos_laser_y,
            slot,
            x,
            y,
        );
        if let Some(value) = self.lanmola_z_offsets.get_mut(slot) {
            *value = (x >> 8) as u8;
        }
        if let Some(value) = self.lanmola_directions.get_mut(slot) {
            *value = (y >> 8) as u8;
        }
    }

    fn set_lanmola_z_offset(&mut self, slot: usize, value: u8) {
        if let Some(z_offset) = self.lanmola_z_offsets.get_mut(slot) {
            *z_offset = value;
        }
        set_word_high_byte(&mut self.beamos_laser_x, slot, value);
    }

    fn set_lanmola_direction(&mut self, slot: usize, value: u8) {
        if let Some(direction) = self.lanmola_directions.get_mut(slot) {
            *direction = value;
        }
        set_word_high_byte(&mut self.beamos_laser_y, slot, value);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HistoryPositionState {
    x: u16,
    y: u16,
}

impl HistoryPositionState {
    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.x as u8
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.y as u8
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LanmolaSegmentMotionState {
    z_offset: u8,
    direction: u8,
}

impl LanmolaSegmentMotionState {
    pub(crate) fn z_offset(&self) -> u8 {
        self.z_offset
    }

    pub(crate) fn direction(&self) -> u8 {
        self.direction
    }
}

fn history_position(xs: &[u16], ys: &[u16], slot: usize) -> HistoryPositionState {
    HistoryPositionState {
        x: xs.get(slot).copied().unwrap_or(0),
        y: ys.get(slot).copied().unwrap_or(0),
    }
}

fn set_history_position(xs: &mut [u16], ys: &mut [u16], slot: usize, x: u16, y: u16) {
    if let Some(value) = xs.get_mut(slot) {
        *value = x;
    }
    if let Some(value) = ys.get_mut(slot) {
        *value = y;
    }
}

fn set_history_low_position(xs: &mut [u16], ys: &mut [u16], slot: usize, x_low: u8, y_low: u8) {
    set_word_low_byte(xs, slot, x_low);
    set_word_low_byte(ys, slot, y_low);
}

fn set_word_low_byte(values: &mut [u16], slot: usize, low: u8) {
    if let Some(value) = values.get_mut(slot) {
        *value = (*value & 0xff00) | u16::from(low);
    }
}

fn set_word_high_byte(values: &mut [u16], slot: usize, high: u8) {
    if let Some(value) = values.get_mut(slot) {
        *value = (*value & 0x00ff) | (u16::from(high) << 8);
    }
}

pub(crate) struct NativeMoldormHistoryBridgeMut<'a> {
    state: &'a mut SpriteHistoryScratchState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeMoldormHistoryBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteHistoryScratchState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        let position = self.state.moldorm_history(self.slot);
        write_split_u16(
            self.ram,
            MOLDORM_HISTORY_X_LO,
            MOLDORM_HISTORY_X_HI,
            self.slot,
            position.x(),
        );
        write_split_u16(
            self.ram,
            MOLDORM_HISTORY_Y_LO,
            MOLDORM_HISTORY_Y_HI,
            self.slot,
            position.y(),
        );
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.state.moldorm_history(self.slot),
            SpriteHistoryScratchState::load_from_ram(self.ram).moldorm_history(self.slot)
        );
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_moldorm_position(self.slot, x, y);
        self.sync();
    }

    pub(crate) fn set_low_position(&mut self, x_low: u8, y_low: u8) {
        self.state.set_moldorm_low_position(self.slot, x_low, y_low);
        // C writes ONLY the low byte of a trail entry (ram[MOLDORM_HISTORY_X_LO + slot]). The
        // full-u16 sync would also write ram[MOLDORM_HISTORY_X_HI + slot], but the lanmola packs
        // 192 flat single-byte slots over the moldorm 128-slot lo/hi split, so X_HI[slot] aliases
        // a DIFFERENT trail slot's low byte (slot 0's hi == slot 128's lo). Syncing the high byte
        // here clobbered a neighboring head's recorded position (f220638: k=0's sync re-stamped
        // slot 0's stale high over k=2's slot-128 low). Write low bytes only, like C.
        if let Some(b) = self.ram.get_mut(MOLDORM_HISTORY_X_LO + self.slot) {
            *b = x_low;
        }
        if let Some(b) = self.ram.get_mut(MOLDORM_HISTORY_Y_LO + self.slot) {
            *b = y_low;
        }
    }
}

pub(crate) struct NativeSwamolaTargetBridgeMut<'a> {
    state: &'a mut SpriteHistoryScratchState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeSwamolaTargetBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteHistoryScratchState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        let position = self.state.swamola_target(self.slot);
        write_split_u16(
            self.ram,
            SWAMOLA_TARGET_X_LO,
            SWAMOLA_TARGET_X_HI,
            self.slot,
            position.x(),
        );
        write_split_u16(
            self.ram,
            SWAMOLA_TARGET_Y_LO,
            SWAMOLA_TARGET_Y_HI,
            self.slot,
            position.y(),
        );
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.state.swamola_target(self.slot),
            SpriteHistoryScratchState::load_from_ram(self.ram).swamola_target(self.slot)
        );
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_swamola_target_position(self.slot, x, y);
        self.sync();
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.state.set_swamola_target_x_low(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.state.set_swamola_target_y_low(self.slot, value);
        self.sync();
    }
}

pub(crate) struct NativeSwamolaHistoryBridgeMut<'a> {
    state: &'a mut SpriteHistoryScratchState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeSwamolaHistoryBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteHistoryScratchState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        let position = self.state.swamola_history(self.slot);
        write_split_u16(
            self.ram,
            SWAMOLA_HISTORY_X_LO,
            SWAMOLA_HISTORY_X_HI,
            self.slot,
            position.x(),
        );
        write_split_u16(
            self.ram,
            SWAMOLA_HISTORY_Y_LO,
            SWAMOLA_HISTORY_Y_HI,
            self.slot,
            position.y(),
        );
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.state.swamola_history(self.slot),
            SpriteHistoryScratchState::load_from_ram(self.ram).swamola_history(self.slot)
        );
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_swamola_history_position(self.slot, x, y);
        self.sync();
    }
}

pub(crate) struct NativeBeamosLaserHistoryBridgeMut<'a> {
    state: &'a mut SpriteHistoryScratchState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBeamosLaserHistoryBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteHistoryScratchState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        let position = self.state.beamos_laser_history(self.slot);
        write_split_u16(
            self.ram,
            BEAMOS_LASER_HISTORY_X_LO,
            BEAMOS_LASER_HISTORY_X_HI,
            self.slot,
            position.x(),
        );
        write_split_u16(
            self.ram,
            BEAMOS_LASER_HISTORY_Y_LO,
            BEAMOS_LASER_HISTORY_Y_HI,
            self.slot,
            position.y(),
        );
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.state.beamos_laser_history(self.slot),
            SpriteHistoryScratchState::load_from_ram(self.ram).beamos_laser_history(self.slot)
        );
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_beamos_laser_position(self.slot, x, y);
        self.sync();
    }
}

pub(crate) struct NativeLanmolaSegmentMotionBridgeMut<'a> {
    state: &'a mut SpriteHistoryScratchState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeLanmolaSegmentMotionBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteHistoryScratchState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        let motion = self.state.lanmola_segment_motion(self.slot);
        self.ram[BEAMOS_LASER_HISTORY_X_HI + self.slot] = motion.z_offset();
        self.ram[BEAMOS_LASER_HISTORY_Y_HI + self.slot] = motion.direction();
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            self.state.lanmola_segment_motion(self.slot),
            SpriteHistoryScratchState::load_from_ram(self.ram).lanmola_segment_motion(self.slot)
        );
    }

    pub(crate) fn set_z_offset(&mut self, value: u8) {
        self.state.set_lanmola_z_offset(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.state.set_lanmola_direction(self.slot, value);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TowerSealState {
    ring_radius: u8,
    center_x: u16,
    center_y: u16,
    wait_countdown: u8,
    orbit_angles: [u8; TOWER_SEAL_ORBIT_SLOTS],
    base_sparkle_x: [u16; TOWER_SEAL_ORBIT_SLOTS],
    base_sparkle_y: [u16; TOWER_SEAL_ORBIT_SLOTS],
    sparkle_phases: [u8; TOWER_SEAL_SPARKLE_SLOTS],
    sparkle_timers: [u8; TOWER_SEAL_SPARKLE_SLOTS],
    sparkle_x: [u16; TOWER_SEAL_SPARKLE_SLOTS],
    sparkle_y: [u16; TOWER_SEAL_SPARKLE_SLOTS],
}

impl TowerSealState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self {
            ring_radius: ram.get(TOWER_SEAL_RING_RADIUS).copied().unwrap_or(0),
            center_x: read_le_u16(ram, TOWER_SEAL_CENTER_X),
            center_y: read_le_u16(ram, TOWER_SEAL_CENTER_Y),
            wait_countdown: ram.get(TOWER_SEAL_WAIT_COUNTDOWN).copied().unwrap_or(0),
            ..Self::default()
        };
        for slot in 0..TOWER_SEAL_ORBIT_SLOTS {
            state.orbit_angles[slot] = ram.get(TOWER_SEAL_ORBIT_ANGLE + slot).copied().unwrap_or(0);
            state.base_sparkle_x[slot] = read_split_u16(
                ram,
                TOWER_SEAL_BASE_SPARKLE_X_LO,
                TOWER_SEAL_BASE_SPARKLE_X_HI,
                slot,
            );
            state.base_sparkle_y[slot] = read_split_u16(
                ram,
                TOWER_SEAL_BASE_SPARKLE_Y_LO,
                TOWER_SEAL_BASE_SPARKLE_Y_HI,
                slot,
            );
        }
        for slot in 0..TOWER_SEAL_SPARKLE_SLOTS {
            state.sparkle_phases[slot] = ram
                .get(TOWER_SEAL_SPARKLE_PHASE + slot)
                .copied()
                .unwrap_or(0);
            state.sparkle_timers[slot] = ram
                .get(TOWER_SEAL_SPARKLE_TIMER + slot)
                .copied()
                .unwrap_or(0);
            state.sparkle_x[slot] =
                read_split_u16(ram, TOWER_SEAL_SPARKLE_X_LO, TOWER_SEAL_SPARKLE_X_HI, slot);
            state.sparkle_y[slot] =
                read_split_u16(ram, TOWER_SEAL_SPARKLE_Y_LO, TOWER_SEAL_SPARKLE_Y_HI, slot);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[TOWER_SEAL_RING_RADIUS] = self.ring_radius;
        write_le_u16(ram, TOWER_SEAL_CENTER_X, self.center_x);
        write_le_u16(ram, TOWER_SEAL_CENTER_Y, self.center_y);
        ram[TOWER_SEAL_WAIT_COUNTDOWN] = self.wait_countdown;
    }

    fn write_transient_slots_to_ram(&self, ram: &mut [u8]) {
        for slot in 0..TOWER_SEAL_ORBIT_SLOTS {
            ram[TOWER_SEAL_ORBIT_ANGLE + slot] = self.orbit_angles[slot];
            write_split_u16(
                ram,
                TOWER_SEAL_BASE_SPARKLE_X_LO,
                TOWER_SEAL_BASE_SPARKLE_X_HI,
                slot,
                self.base_sparkle_x[slot],
            );
            write_split_u16(
                ram,
                TOWER_SEAL_BASE_SPARKLE_Y_LO,
                TOWER_SEAL_BASE_SPARKLE_Y_HI,
                slot,
                self.base_sparkle_y[slot],
            );
        }
        for slot in 0..TOWER_SEAL_SPARKLE_SLOTS {
            ram[TOWER_SEAL_SPARKLE_PHASE + slot] = self.sparkle_phases[slot];
            ram[TOWER_SEAL_SPARKLE_TIMER + slot] = self.sparkle_timers[slot];
            write_split_u16(
                ram,
                TOWER_SEAL_SPARKLE_X_LO,
                TOWER_SEAL_SPARKLE_X_HI,
                slot,
                self.sparkle_x[slot],
            );
            write_split_u16(
                ram,
                TOWER_SEAL_SPARKLE_Y_LO,
                TOWER_SEAL_SPARKLE_Y_HI,
                slot,
                self.sparkle_y[slot],
            );
        }
    }

    pub(crate) fn ring_radius(&self) -> u8 {
        self.ring_radius
    }

    pub(crate) fn center_x(&self) -> u16 {
        self.center_x
    }

    pub(crate) fn center_y(&self) -> u16 {
        self.center_y
    }

    pub(crate) fn orbit(&self, slot: usize) -> TowerSealOrbitState {
        TowerSealOrbitState {
            angle: self.orbit_angle(slot),
        }
    }

    pub(crate) fn sparkle(&self, slot: usize) -> TowerSealSparkleState {
        TowerSealSparkleState {
            phase: self.sparkle_phase(slot),
            x: self.sparkle_x.get(slot).copied().unwrap_or(0),
            y: self.sparkle_y.get(slot).copied().unwrap_or(0),
        }
    }

    pub(crate) fn set_ring_radius(&mut self, value: u8) {
        self.ring_radius = value;
    }

    pub(crate) fn set_center(&mut self, x: u16, y: u16) {
        self.center_x = x;
        self.center_y = y;
    }

    pub(crate) fn tick_wait_countdown(&mut self) -> u8 {
        self.wait_countdown = self.wait_countdown.wrapping_sub(1);
        self.wait_countdown
    }

    pub(crate) fn set_wait_countdown(&mut self, value: u8) {
        self.wait_countdown = value;
    }

    fn orbit_angle(&self, slot: usize) -> u8 {
        self.orbit_angles.get(slot).copied().unwrap_or(0)
    }

    fn set_orbit_angle(&mut self, slot: usize, value: u8) {
        if let Some(angle) = self.orbit_angles.get_mut(slot) {
            *angle = value;
        }
    }

    fn advance_orbit_angle_mod64(&mut self, slot: usize) -> u8 {
        let angle = self.orbit_angle(slot).wrapping_add(1) & 0x3f;
        self.set_orbit_angle(slot, angle);
        angle
    }

    fn set_base_sparkle_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(base_x) = self.base_sparkle_x.get_mut(slot) {
            *base_x = x;
        }
        if let Some(base_y) = self.base_sparkle_y.get_mut(slot) {
            *base_y = y;
        }
    }

    fn base_sparkle_position(&self, base: usize) -> (u16, u16) {
        (
            self.base_sparkle_x.get(base).copied().unwrap_or(0),
            self.base_sparkle_y.get(base).copied().unwrap_or(0),
        )
    }

    fn sparkle_phase(&self, slot: usize) -> u8 {
        self.sparkle_phases.get(slot).copied().unwrap_or(0)
    }

    fn set_sparkle_phase(&mut self, slot: usize, value: u8) {
        if let Some(phase) = self.sparkle_phases.get_mut(slot) {
            *phase = value;
        }
    }

    fn set_sparkle_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.sparkle_timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_sparkle_timer(&mut self, slot: usize) -> u8 {
        let timer = self
            .sparkle_timers
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_sub(1);
        self.set_sparkle_timer(slot, timer);
        timer
    }

    fn advance_sparkle_phase(&mut self, slot: usize) -> u8 {
        let phase = self.sparkle_phase(slot).wrapping_add(1);
        self.set_sparkle_phase(slot, phase);
        phase
    }

    fn set_sparkle_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(sparkle_x) = self.sparkle_x.get_mut(slot) {
            *sparkle_x = x;
        }
        if let Some(sparkle_y) = self.sparkle_y.get_mut(slot) {
            *sparkle_y = y;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TowerSealOrbitState {
    angle: u8,
}

impl TowerSealOrbitState {
    pub(crate) fn angle(&self) -> u8 {
        self.angle
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TowerSealSparkleState {
    phase: u8,
    x: u16,
    y: u16,
}

impl TowerSealSparkleState {
    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }

    pub(crate) fn is_free(&self) -> bool {
        self.phase == 0xff
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }
}

pub(crate) struct NativeTowerSealBridgeMut<'a> {
    state: &'a mut TowerSealState,
    ram: &'a mut [u8],
}

impl<'a> NativeTowerSealBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TowerSealState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TowerSealState::load_from_ram(self.ram));
    }

    pub(crate) fn set_ring_radius(&mut self, value: u8) {
        self.state.set_ring_radius(value);
        self.sync();
    }

    pub(crate) fn set_center(&mut self, x: u16, y: u16) {
        self.state.set_center(x, y);
        self.sync();
    }

    pub(crate) fn tick_wait_countdown(&mut self) -> u8 {
        let value = self.state.tick_wait_countdown();
        self.sync();
        value
    }

    pub(crate) fn set_wait_countdown(&mut self, value: u8) {
        self.state.set_wait_countdown(value);
        self.sync();
    }
}

pub(crate) struct NativeTowerSealOrbitBridgeMut<'a> {
    state: &'a mut TowerSealState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeTowerSealOrbitBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TowerSealState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_transient_slots_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TowerSealState::load_from_ram(self.ram));
    }

    pub(crate) fn set_angle(&mut self, value: u8) {
        self.state.set_orbit_angle(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_angle_mod64(&mut self) -> u8 {
        let angle = self.state.advance_orbit_angle_mod64(self.slot);
        self.sync();
        angle
    }

    pub(crate) fn set_base_sparkle_position(&mut self, x: u16, y: u16) {
        self.state.set_base_sparkle_position(self.slot, x, y);
        self.sync();
    }
}

pub(crate) struct NativeTowerSealSparkleBridgeMut<'a> {
    state: &'a mut TowerSealState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeTowerSealSparkleBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TowerSealState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_transient_slots_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TowerSealState::load_from_ram(self.ram));
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_sparkle_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_sparkle_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.state.tick_sparkle_timer(self.slot);
        self.sync();
        timer
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.state.advance_sparkle_phase(self.slot);
        self.sync();
        phase
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_sparkle_position(self.slot, x, y);
        self.sync();
    }

    pub(crate) fn base_sparkle_position(&self, base: usize) -> (u16, u16) {
        self.state.base_sparkle_position(base)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EntranceEffectState {
    phases: [u8; ENTRANCE_EFFECT_PHASE_SLOTS],
    timers: [u8; ENTRANCE_EFFECT_PHASE_SLOTS],
    state: u8,
    secondary_state: u8,
    center_y: u16,
    center_x: u16,
    direction: u8,
    y_positions: [u16; ENTRANCE_EFFECT_POSITION_SLOTS],
    x_positions: [u16; ENTRANCE_EFFECT_POSITION_SLOTS],
    fireball_timers: [u8; BLAST_WALL_FIREBALL_SLOTS],
}

impl EntranceEffectState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self {
            state: ram.get(BLAST_WALL_ENTRY_STATE).copied().unwrap_or(0),
            secondary_state: ram.get(BLAST_WALL_SECONDARY_STATE).copied().unwrap_or(0),
            center_y: read_le_u16(ram, BLAST_WALL_CENTER_Y),
            center_x: read_le_u16(ram, BLAST_WALL_CENTER_X),
            direction: ram.get(BLAST_WALL_DIRECTION).copied().unwrap_or(0),
            ..Self::default()
        };
        for slot in 0..ENTRANCE_EFFECT_PHASE_SLOTS {
            state.phases[slot] = ram
                .get(BLAST_WALL_EXPLOSION_PHASE + slot)
                .copied()
                .unwrap_or(0);
            state.timers[slot] = ram
                .get(BLAST_WALL_EXPLOSION_TIMER + slot)
                .copied()
                .unwrap_or(0);
            state.y_positions[slot] = read_le_u16(ram, BLAST_WALL_FRAGMENT_Y + slot * 2);
            state.x_positions[slot] = read_le_u16(ram, BLAST_WALL_FRAGMENT_X + slot * 2);
        }
        for slot in 0..BLAST_WALL_FIREBALL_SLOTS {
            state.fireball_timers[slot] = ram
                .get(BLAST_WALL_FIREBALL_TIMER + slot)
                .copied()
                .unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for slot in 0..ENTRANCE_EFFECT_PHASE_SLOTS {
            ram[BLAST_WALL_EXPLOSION_PHASE + slot] = self.phases[slot];
            ram[BLAST_WALL_EXPLOSION_TIMER + slot] = self.timers[slot];
            write_le_u16(
                ram,
                BLAST_WALL_FRAGMENT_Y + slot * 2,
                self.y_positions[slot],
            );
            write_le_u16(
                ram,
                BLAST_WALL_FRAGMENT_X + slot * 2,
                self.x_positions[slot],
            );
        }
        ram[BLAST_WALL_ENTRY_STATE] = self.state;
        ram[BLAST_WALL_SECONDARY_STATE] = self.secondary_state;
        write_le_u16(ram, BLAST_WALL_CENTER_Y, self.center_y);
        write_le_u16(ram, BLAST_WALL_CENTER_X, self.center_x);
        ram[BLAST_WALL_DIRECTION] = self.direction;
        for slot in 0..BLAST_WALL_FIREBALL_SLOTS {
            ram[BLAST_WALL_FIREBALL_TIMER + slot] = self.fireball_timers[slot];
        }
    }

    pub(crate) fn skull_woods_fire(&self) -> SkullWoodsFireState {
        SkullWoodsFireState {
            entrance_opening_started: self.state,
            inner_x: self.center_x,
            inner_y: self.center_y,
            outer_x: self.x_positions[3],
            outer_y: self.y_positions[3],
        }
    }

    pub(crate) fn blast_wall(&self) -> BlastWallState {
        BlastWallState {
            entry_state: self.state,
            secondary_state: self.secondary_state,
            direction: self.direction,
            center_x: self.center_x,
            center_y: self.center_y,
        }
    }

    pub(crate) fn blast_wall_direction(&self) -> u8 {
        self.direction
    }

    pub(crate) fn skull_woods_fire_has_started_entrance_opening(&self) -> bool {
        self.state != 0
    }

    pub(crate) fn skull_woods_fire_inner_x(&self) -> u16 {
        self.center_x
    }

    pub(crate) fn skull_woods_fire_slot(&self, slot: usize) -> SkullWoodsFireSlotState {
        SkullWoodsFireSlotState {
            phase: self.phases.get(slot).copied().unwrap_or(0),
            x: self.x_positions.get(slot).copied().unwrap_or(0),
            y: self.y_positions.get(slot).copied().unwrap_or(0),
        }
    }

    pub(crate) fn blast_wall_explosion_slot(&self, slot: usize) -> BlastWallExplosionSlotState {
        BlastWallExplosionSlotState {
            phase: self.phases.get(slot).copied().unwrap_or(0),
            timer: self.timers.get(slot).copied().unwrap_or(0),
        }
    }

    pub(crate) fn blast_wall_fragment_slot(&self, slot: usize) -> BlastWallFragmentSlotState {
        BlastWallFragmentSlotState {
            x: self.x_positions.get(slot).copied().unwrap_or(0),
            y: self.y_positions.get(slot).copied().unwrap_or(0),
        }
    }

    pub(crate) fn blast_wall_fireball_slot(&self, slot: usize) -> BlastWallFireballSlotState {
        BlastWallFireballSlotState {
            timer: self.fireball_timers.get(slot).copied().unwrap_or(0),
        }
    }

    fn set_phase(&mut self, slot: usize, value: u8) {
        if let Some(phase) = self.phases.get_mut(slot) {
            *phase = value;
        }
    }

    fn advance_phase(&mut self, slot: usize) -> u8 {
        let phase = self.phases.get(slot).copied().unwrap_or(0).wrapping_add(1);
        self.set_phase(slot, phase);
        phase
    }

    fn set_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_timer(&mut self, slot: usize) -> u8 {
        let timer = self.timers.get(slot).copied().unwrap_or(0).wrapping_sub(1);
        self.set_timer(slot, timer);
        timer
    }

    fn set_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(position) = self.x_positions.get_mut(slot) {
            *position = x;
        }
        if let Some(position) = self.y_positions.get_mut(slot) {
            *position = y;
        }
    }

    fn offset_position(&mut self, slot: usize, x_delta: i16, y_delta: i16) -> (u16, u16) {
        let x = self
            .x_positions
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_add(x_delta as u16);
        let y = self
            .y_positions
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_add(y_delta as u16);
        self.set_position(slot, x, y);
        (x, y)
    }

    fn set_fireball_timer(&mut self, slot: usize, value: u8) {
        if let Some(timer) = self.fireball_timers.get_mut(slot) {
            *timer = value;
        }
    }

    fn tick_fireball_timer(&mut self, slot: usize) -> u8 {
        let timer = self
            .fireball_timers
            .get(slot)
            .copied()
            .unwrap_or(0)
            .wrapping_sub(1);
        self.set_fireball_timer(slot, timer);
        timer
    }

    fn clear_entrance_opening_started(&mut self) {
        self.state = 0;
    }

    fn set_entrance_opening_started(&mut self) {
        self.state = 1;
    }

    fn set_inner_position(&mut self, x: u16, y: u16) {
        self.center_x = x;
        self.center_y = y;
    }

    fn set_outer_position(&mut self, x: u16, y: u16) {
        self.set_position(3, x, y);
    }

    fn retreat_inner_y(&mut self, value: u16) -> u16 {
        self.center_y = self.center_y.wrapping_sub(value);
        self.center_y
    }

    fn clear_entry_state(&mut self) {
        self.state = 0;
    }

    fn clear_secondary_state(&mut self) {
        self.secondary_state = 0;
    }

    fn offset_center(&mut self, x_delta: i8, y_delta: i8) -> (u16, u16) {
        self.center_y = self.center_y.wrapping_add(y_delta as i16 as u16);
        self.center_x = self.center_x.wrapping_add(x_delta as i16 as u16);
        (self.center_x, self.center_y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SkullWoodsFireState {
    entrance_opening_started: u8,
    inner_x: u16,
    inner_y: u16,
    outer_x: u16,
    outer_y: u16,
}

impl SkullWoodsFireState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            entrance_opening_started: ram.get(SKULL_WOODS_FIRE_STARTED).copied().unwrap_or(0),
            inner_x: read_le_u16(ram, SKULL_WOODS_FIRE_INNER_X),
            inner_y: read_le_u16(ram, SKULL_WOODS_FIRE_INNER_Y),
            outer_x: read_le_u16(ram, SKULL_WOODS_FIRE_OUTER_X),
            outer_y: read_le_u16(ram, SKULL_WOODS_FIRE_OUTER_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SKULL_WOODS_FIRE_STARTED] = self.entrance_opening_started;
        write_le_u16(ram, SKULL_WOODS_FIRE_INNER_X, self.inner_x);
        write_le_u16(ram, SKULL_WOODS_FIRE_INNER_Y, self.inner_y);
        write_le_u16(ram, SKULL_WOODS_FIRE_OUTER_X, self.outer_x);
        write_le_u16(ram, SKULL_WOODS_FIRE_OUTER_Y, self.outer_y);
    }

    pub(crate) fn has_started_entrance_opening(&self) -> bool {
        self.entrance_opening_started != 0
    }

    pub(crate) fn inner_x(&self) -> u16 {
        self.inner_x
    }

    pub(crate) fn inner_y(&self) -> u16 {
        self.inner_y
    }

    pub(crate) fn clear_entrance_opening_started(&mut self) {
        self.entrance_opening_started = 0;
    }

    pub(crate) fn set_entrance_opening_started(&mut self) {
        self.entrance_opening_started = 1;
    }

    pub(crate) fn set_inner_position(&mut self, x: u16, y: u16) {
        self.inner_x = x;
        self.inner_y = y;
    }

    pub(crate) fn set_outer_position(&mut self, x: u16, y: u16) {
        self.outer_x = x;
        self.outer_y = y;
    }

    pub(crate) fn retreat_inner_y(&mut self, value: u16) -> u16 {
        self.inner_y = self.inner_y.wrapping_sub(value);
        self.inner_y
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SkullWoodsFireSlotState {
    phase: u8,
    x: u16,
    y: u16,
}

impl SkullWoodsFireSlotState {
    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }

    pub(crate) fn is_finished(&self) -> bool {
        (self.phase() as i8).is_negative()
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }
}

pub(crate) struct NativeSkullWoodsFireBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
}

impl<'a> NativeSkullWoodsFireBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_entrance_opening_started(&mut self) {
        self.state.clear_entrance_opening_started();
        self.sync();
    }

    pub(crate) fn set_entrance_opening_started(&mut self) {
        self.state.set_entrance_opening_started();
        self.sync();
    }

    pub(crate) fn set_inner_position(&mut self, x: u16, y: u16) {
        self.state.set_inner_position(x, y);
        self.sync();
    }

    pub(crate) fn set_outer_position(&mut self, x: u16, y: u16) {
        self.state.set_outer_position(x, y);
        self.sync();
    }

    pub(crate) fn retreat_inner_y(&mut self, value: u16) -> u16 {
        let y = self.state.retreat_inner_y(value);
        self.sync();
        y
    }
}

pub(crate) struct NativeSkullWoodsFireSlotBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeSkullWoodsFireSlotBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.state.advance_phase(self.slot);
        self.sync();
        phase
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.state.tick_timer(self.slot);
        self.sync();
        timer
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_position(self.slot, x, y);
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BlastWallState {
    entry_state: u8,
    secondary_state: u8,
    direction: u8,
    center_x: u16,
    center_y: u16,
}

impl BlastWallState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            entry_state: ram.get(BLAST_WALL_ENTRY_STATE).copied().unwrap_or(0),
            secondary_state: ram.get(BLAST_WALL_SECONDARY_STATE).copied().unwrap_or(0),
            direction: ram.get(BLAST_WALL_DIRECTION).copied().unwrap_or(0),
            center_x: read_le_u16(ram, BLAST_WALL_CENTER_X),
            center_y: read_le_u16(ram, BLAST_WALL_CENTER_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[BLAST_WALL_ENTRY_STATE] = self.entry_state;
        ram[BLAST_WALL_SECONDARY_STATE] = self.secondary_state;
        ram[BLAST_WALL_DIRECTION] = self.direction;
        write_le_u16(ram, BLAST_WALL_CENTER_X, self.center_x);
        write_le_u16(ram, BLAST_WALL_CENTER_Y, self.center_y);
    }

    pub(crate) fn direction(&self) -> u8 {
        self.direction
    }

    pub(crate) fn center_x(&self) -> u16 {
        self.center_x
    }

    pub(crate) fn center_y(&self) -> u16 {
        self.center_y
    }

    pub(crate) fn clear_entry_state(&mut self) {
        self.entry_state = 0;
    }

    pub(crate) fn clear_secondary_state(&mut self) {
        self.secondary_state = 0;
    }

    pub(crate) fn offset_center(&mut self, x_delta: i8, y_delta: i8) -> (u16, u16) {
        self.center_y = self.center_y.wrapping_add(y_delta as i16 as u16);
        self.center_x = self.center_x.wrapping_add(x_delta as i16 as u16);
        (self.center_x, self.center_y)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BlastWallExplosionSlotState {
    phase: u8,
    timer: u8,
}

impl BlastWallExplosionSlotState {
    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }

    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BlastWallFragmentSlotState {
    x: u16,
    y: u16,
}

impl BlastWallFragmentSlotState {
    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BlastWallFireballSlotState {
    timer: u8,
}

impl BlastWallFireballSlotState {
    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }
}

pub(crate) struct NativeBlastWallBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
}

impl<'a> NativeBlastWallBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_entry_state(&mut self) {
        self.state.clear_entry_state();
        self.sync();
    }

    pub(crate) fn clear_secondary_state(&mut self) {
        self.state.clear_secondary_state();
        self.sync();
    }

    pub(crate) fn offset_center(&mut self, x_delta: i8, y_delta: i8) -> (u16, u16) {
        let center = self.state.offset_center(x_delta, y_delta);
        self.sync();
        center
    }
}

pub(crate) struct NativeBlastWallExplosionBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBlastWallExplosionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn set_phase(&mut self, value: u8) {
        self.state.set_phase(self.slot, value);
        self.sync();
    }

    pub(crate) fn advance_phase(&mut self) -> u8 {
        let phase = self.state.advance_phase(self.slot);
        self.sync();
        phase
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.state.tick_timer(self.slot);
        self.sync();
        timer
    }
}

pub(crate) struct NativeBlastWallFragmentBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBlastWallFragmentBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_position(self.slot, x, y);
        self.sync();
    }

    pub(crate) fn offset(&mut self, x_delta: i16, y_delta: i16) -> (u16, u16) {
        let position = self.state.offset_position(self.slot, x_delta, y_delta);
        self.sync();
        position
    }
}

pub(crate) struct NativeBlastWallFireballBridgeMut<'a> {
    state: &'a mut EntranceEffectState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeBlastWallFireballBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EntranceEffectState, ram: &'a mut [u8], slot: usize) -> Self {
        Self { state, ram, slot }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, EntranceEffectState::load_from_ram(self.ram));
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.state.set_fireball_timer(self.slot, value);
        self.sync();
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let timer = self.state.tick_fireball_timer(self.slot);
        self.sync();
        timer
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DiggingGamePrizeState {
    attempts: u8,
    spawned_marker: u8,
}

impl DiggingGamePrizeState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            attempts: ram.get(DIGGING_GAME_PRIZE_ATTEMPTS).copied().unwrap_or(0),
            spawned_marker: ram.get(DIGGING_GAME_PRIZE_SPAWNED).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[DIGGING_GAME_PRIZE_ATTEMPTS] = self.attempts;
        ram[DIGGING_GAME_PRIZE_SPAWNED] = self.spawned_marker;
    }

    pub(crate) fn attempts(&self) -> u8 {
        self.attempts
    }

    pub(crate) fn spawned_marker(&self) -> u8 {
        self.spawned_marker
    }

    pub(crate) fn increment_attempts(&mut self) {
        self.attempts = self.attempts.wrapping_add(1);
    }

    pub(crate) fn mark_spawned(&mut self) {
        self.spawned_marker = 0xeb;
    }

    pub(crate) fn clear_prize_spawned(&mut self) {
        self.spawned_marker = 0;
    }
}

pub(crate) struct NativeDiggingGamePrizeBridgeMut<'a> {
    state: &'a mut DiggingGamePrizeState,
    ram: &'a mut [u8],
}

impl<'a> NativeDiggingGamePrizeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DiggingGamePrizeState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, DiggingGamePrizeState::load_from_ram(self.ram));
    }

    pub(crate) fn increment_attempts(&mut self) {
        self.state.increment_attempts();
        self.sync();
    }

    pub(crate) fn mark_spawned(&mut self) {
        self.state.mark_spawned();
        self.sync();
    }

    pub(crate) fn clear_prize_spawned(&mut self) {
        self.state.clear_prize_spawned();
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

fn read_split_u16(ram: &[u8], low_base: usize, high_base: usize, slot: usize) -> u16 {
    u16::from(ram.get(low_base + slot).copied().unwrap_or(0))
        | (u16::from(ram.get(high_base + slot).copied().unwrap_or(0)) << 8)
}

fn read_split_word_bank(ram: &[u8], low_base: usize, high_base: usize, len: usize) -> Vec<u16> {
    let mut bank = Vec::with_capacity(len);
    for slot in 0..len {
        bank.push(read_split_u16(ram, low_base, high_base, slot));
    }
    bank
}

fn read_byte_bank(ram: &[u8], base: usize, len: usize) -> Vec<u8> {
    let mut bank = Vec::with_capacity(len);
    for slot in 0..len {
        bank.push(ram.get(base + slot).copied().unwrap_or(0));
    }
    bank
}

fn write_split_word_bank(ram: &mut [u8], low_base: usize, high_base: usize, values: &[u16]) {
    for (slot, value) in values.iter().copied().enumerate() {
        write_split_u16(ram, low_base, high_base, slot, value);
    }
}

fn write_byte_bank(ram: &mut [u8], base: usize, values: &[u8]) {
    for (slot, value) in values.iter().copied().enumerate() {
        ram[base + slot] = value;
    }
}

fn write_split_u16(ram: &mut [u8], low_base: usize, high_base: usize, slot: usize, value: u16) {
    ram[low_base + slot] = value as u8;
    ram[high_base + slot] = (value >> 8) as u8;
}

pub(crate) struct NativeDoorDebrisBridgeMut<'a> {
    state: &'a mut DoorDebrisState,
    ram: &'a mut [u8],
}

impl<'a> NativeDoorDebrisBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DoorDebrisState, ram: &'a mut [u8]) -> Self {
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
