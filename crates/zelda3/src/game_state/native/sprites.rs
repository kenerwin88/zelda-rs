use crate::game_state::constants::{
    CHAIN_CHOMP_HISTORY_X, CHAIN_CHOMP_HISTORY_Y, DRAW_WORK_FLAGS_HI, DRAW_WORK_POSITION_X,
    DRAW_WORK_POSITION_Y, DUAL_LAYER_TILE_CACHE, ENEMY_DAMAGE_DATA, ETHER_ANGLE,
    ETHER_BEAM_TOP_BUCKET, ETHER_BEAM_Y, ETHER_ORBIT_X, ETHER_ORBIT_Y, ETHER_ORB_X, ETHER_ORB_Y,
    ETHER_RADIUS, ETHER_SPIN_COUNTDOWN, HITBOX_WORK_X_OFFSET, HITBOX_WORK_Y_OFFSET,
    MAZE_GAME_TIMER_HI, MAZE_GAME_TIMER_LO, MAZE_GAME_TIMER_SNAPSHOT_HI,
    MAZE_GAME_TIMER_SNAPSHOT_LO, OVERWORLD_SPRITE_PRESENCE, OVERWORLD_SPRITE_WAS_LOADED,
    PRIZE_DROP_CYCLE, TAGALONG_LAYERBITS, TAGALONG_X_HI, TAGALONG_X_LO, TAGALONG_Y_HI,
    TAGALONG_Y_LO, TAGALONG_Z,
};
use crate::types::{read_le_u16, write_le_u16};

const SPRITE_SLOT_COUNT: usize = 16;
const TAGALONG_SLOT_COUNT: usize = 20;
const CHAIN_CHOMP_HISTORY_LEN: usize = 0x80;
const ETHER_ANGLE_COUNT: usize = 8;
const ENEMY_DAMAGE_SUBCLASS_COUNT: usize = 0x1000;
const OVERWORLD_SPRITE_FLAG_COUNT: usize = 0x200;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteState {
    pub(crate) maze_game_timer: MazeGameTimerState,
    pub(crate) prize_drop_cycle: PrizeDropCycleState,
    pub(crate) dual_layer_tile_cache: DualLayerTileCacheState,
    pub(crate) draw_hitbox_work: SpriteDrawHitboxWorkState,
    pub(crate) enemy_damage_subclasses: EnemyDamageSubclassTableState,
    pub(crate) tagalong_trail: TagalongTrailState,
    pub(crate) chain_chomp_history: ChainChompHistoryState,
    pub(crate) ether_orbit: EtherOrbitState,
    pub(crate) overworld_sprite_presence: OverworldSpritePresenceState,
    pub(crate) overworld_sprite_loaded: OverworldSpriteLoadedState,
}

impl SpriteState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            maze_game_timer: MazeGameTimerState::load_from_ram(ram),
            prize_drop_cycle: PrizeDropCycleState::load_from_ram(ram),
            dual_layer_tile_cache: DualLayerTileCacheState::load_from_ram(ram),
            draw_hitbox_work: SpriteDrawHitboxWorkState::load_from_ram(ram),
            enemy_damage_subclasses: EnemyDamageSubclassTableState::load_from_ram(ram),
            tagalong_trail: TagalongTrailState::load_from_ram(ram),
            chain_chomp_history: ChainChompHistoryState::load_from_ram(ram),
            ether_orbit: EtherOrbitState::load_from_ram(ram),
            overworld_sprite_presence: OverworldSpritePresenceState::load_from_ram(ram),
            overworld_sprite_loaded: OverworldSpriteLoadedState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.maze_game_timer.write_to_ram(ram);
        self.prize_drop_cycle.write_to_ram(ram);
        self.dual_layer_tile_cache.write_to_ram(ram);
        self.draw_hitbox_work.write_to_ram(ram);
        self.enemy_damage_subclasses.write_to_ram(ram);
        self.tagalong_trail.write_to_ram(ram);
        self.chain_chomp_history.write_to_ram(ram);
        self.ether_orbit.write_to_ram(ram);
        self.overworld_sprite_presence.write_to_ram(ram);
        self.overworld_sprite_loaded.write_to_ram(ram);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldSpritePresenceState {
    markers: Vec<u8>,
}

impl Default for OverworldSpritePresenceState {
    fn default() -> Self {
        Self {
            markers: vec![0; OVERWORLD_SPRITE_FLAG_COUNT],
        }
    }
}

impl OverworldSpritePresenceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            markers: ram[OVERWORLD_SPRITE_PRESENCE
                ..OVERWORLD_SPRITE_PRESENCE + OVERWORLD_SPRITE_FLAG_COUNT]
                .to_vec(),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_SPRITE_PRESENCE..OVERWORLD_SPRITE_PRESENCE + OVERWORLD_SPRITE_FLAG_COUNT]
            .fill(0);
        let len = self.markers.len().min(OVERWORLD_SPRITE_FLAG_COUNT);
        ram[OVERWORLD_SPRITE_PRESENCE..OVERWORLD_SPRITE_PRESENCE + len]
            .copy_from_slice(&self.markers[..len]);
    }

    pub(crate) fn marker(&self, index: usize) -> u8 {
        self.markers.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn set_marker(&mut self, index: usize, value: u8) {
        if let Some(marker) = self.markers.get_mut(index) {
            *marker = value;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldSpriteLoadedState {
    flags: Vec<u8>,
}

impl Default for OverworldSpriteLoadedState {
    fn default() -> Self {
        Self {
            flags: vec![0; OVERWORLD_SPRITE_FLAG_COUNT],
        }
    }
}

impl OverworldSpriteLoadedState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            flags: ram[OVERWORLD_SPRITE_WAS_LOADED
                ..OVERWORLD_SPRITE_WAS_LOADED + OVERWORLD_SPRITE_FLAG_COUNT]
                .to_vec(),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[OVERWORLD_SPRITE_WAS_LOADED..OVERWORLD_SPRITE_WAS_LOADED + OVERWORLD_SPRITE_FLAG_COUNT]
            .fill(0);
        let len = self.flags.len().min(OVERWORLD_SPRITE_FLAG_COUNT);
        ram[OVERWORLD_SPRITE_WAS_LOADED..OVERWORLD_SPRITE_WAS_LOADED + len]
            .copy_from_slice(&self.flags[..len]);
    }

    pub(crate) fn is_loaded(&self, block: u16, loaded_mask: u8) -> bool {
        self.flags
            .get(usize::from(block >> 3))
            .is_some_and(|flag| flag & loaded_mask != 0)
    }

    pub(crate) fn clear_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        if let Some(flag) = self.flags.get_mut(usize::from(block >> 3)) {
            *flag &= !loaded_mask;
        }
    }

    pub(crate) fn set_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        if let Some(flag) = self.flags.get_mut(usize::from(block >> 3)) {
            *flag |= loaded_mask;
        }
    }

    pub(crate) fn clear_all(&mut self) {
        self.flags.fill(0);
    }
}

pub(crate) struct NativeOverworldSpritePresenceBridgeMut<'a> {
    state: &'a mut OverworldSpritePresenceState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldSpritePresenceBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut OverworldSpritePresenceState, ram: &'a mut [u8]) -> Self {
        *state = OverworldSpritePresenceState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            OverworldSpritePresenceState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_marker(&mut self, index: usize, value: u8) {
        self.state.set_marker(index, value);
        self.sync();
    }
}

pub(crate) struct NativeOverworldSpriteLoadedBridgeMut<'a> {
    state: &'a mut OverworldSpriteLoadedState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldSpriteLoadedBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut OverworldSpriteLoadedState, ram: &'a mut [u8]) -> Self {
        *state = OverworldSpriteLoadedState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            OverworldSpriteLoadedState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn clear_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.state.clear_loaded_mask(block, loaded_mask);
        self.sync();
    }

    pub(crate) fn clear_loaded_mask_wrapped(&mut self, block: u16, loaded_mask: u8) {
        let address = (OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)) & 0x1ffff;
        self.ram[address] &= !loaded_mask;
        if let Some(index) = address.checked_sub(OVERWORLD_SPRITE_WAS_LOADED) {
            if index < OVERWORLD_SPRITE_FLAG_COUNT {
                self.state.clear_loaded_mask(block, loaded_mask);
            }
        }
    }

    pub(crate) fn set_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.state.set_loaded_mask(block, loaded_mask);
        self.sync();
    }

    pub(crate) fn clear_all(&mut self) {
        self.state.clear_all();
        self.sync();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EtherOrbitState {
    angles: [u8; ETHER_ANGLE_COUNT],
    radius: u8,
    beam_y: u16,
    beam_adjusted_y: u16,
    orbit_x: u16,
    orbit_y: u16,
    spin_countdown: u8,
    orb_x: u16,
    orb_y: u16,
}

impl EtherOrbitState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut angles = [0; ETHER_ANGLE_COUNT];
        for (slot, angle) in angles.iter_mut().enumerate() {
            *angle = ram.get(ETHER_ANGLE + slot).copied().unwrap_or(0);
        }

        Self {
            angles,
            radius: ram.get(ETHER_RADIUS).copied().unwrap_or(0),
            beam_y: read_le_u16(ram, ETHER_BEAM_Y),
            beam_adjusted_y: read_le_u16(ram, ETHER_BEAM_TOP_BUCKET),
            orbit_x: read_le_u16(ram, ETHER_ORBIT_X),
            orbit_y: read_le_u16(ram, ETHER_ORBIT_Y),
            spin_countdown: ram.get(ETHER_SPIN_COUNTDOWN).copied().unwrap_or(0),
            orb_x: read_le_u16(ram, ETHER_ORB_X),
            orb_y: read_le_u16(ram, ETHER_ORB_Y),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[ETHER_ANGLE..ETHER_ANGLE + ETHER_ANGLE_COUNT].copy_from_slice(&self.angles);
        ram[ETHER_RADIUS] = self.radius;
        write_le_u16(ram, ETHER_BEAM_Y, self.beam_y);
        write_le_u16(ram, ETHER_BEAM_TOP_BUCKET, self.beam_adjusted_y);
        write_le_u16(ram, ETHER_ORBIT_X, self.orbit_x);
        write_le_u16(ram, ETHER_ORBIT_Y, self.orbit_y);
        ram[ETHER_SPIN_COUNTDOWN] = self.spin_countdown;
        write_le_u16(ram, ETHER_ORB_Y, self.orb_y);
        write_le_u16(ram, ETHER_ORB_X, self.orb_x);
    }

    pub(crate) fn angle(&self, slot: usize) -> u8 {
        self.angles.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn radius(&self) -> u8 {
        self.radius
    }

    pub(crate) fn beam_top_bucket(&self) -> u8 {
        self.beam_adjusted_y as u8
    }

    pub(crate) fn beam_y(&self) -> u16 {
        self.beam_y
    }

    pub(crate) fn orbit_x(&self) -> u16 {
        self.orbit_x
    }

    pub(crate) fn orbit_y(&self) -> u16 {
        self.orbit_y
    }

    pub(crate) fn swordbeam_temp_x(&self) -> u16 {
        self.orbit_x()
    }

    pub(crate) fn swordbeam_temp_y(&self) -> u16 {
        self.orbit_y()
    }

    pub(crate) fn orb_x(&self) -> u16 {
        self.orb_x
    }

    pub(crate) fn orb_y(&self) -> u16 {
        self.orb_y
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        if let Some(angle) = self.angles.get_mut(slot) {
            *angle = value;
        }
    }

    pub(crate) fn advance_angle(&mut self, slot: usize) -> u8 {
        let next = self.angle(slot).wrapping_add(1) & 0x3f;
        self.set_angle(slot, next);
        next
    }

    pub(crate) fn set_radius(&mut self, value: u8) {
        self.radius = value;
    }

    pub(crate) fn tick_spin_countdown(&mut self) -> u8 {
        self.spin_countdown = self.spin_countdown.wrapping_sub(1);
        self.spin_countdown
    }

    pub(crate) fn set_spin_countdown(&mut self, value: u8) {
        self.spin_countdown = value;
    }

    pub(crate) fn set_beam_top_bucket(&mut self, value: u8) {
        self.beam_adjusted_y = (self.beam_adjusted_y & 0xff00) | u16::from(value);
    }

    pub(crate) fn initialize_beam_adjusted_y(&mut self, value: u16) {
        self.beam_adjusted_y = value;
    }

    pub(crate) fn set_orb_position(&mut self, x: u16, y: u16) {
        self.orb_x = x;
        self.orb_y = y;
    }

    pub(crate) fn set_orbit_position(&mut self, x: u16, y: u16) {
        self.orbit_x = x;
        self.orbit_y = y;
    }

    pub(crate) fn set_swordbeam_temp(&mut self, x: u16, y: u16) {
        self.set_orbit_position(x, y);
    }

    pub(crate) fn set_beam_y(&mut self, value: u16) {
        self.beam_y = value;
    }
}

pub(crate) struct NativeEtherOrbitBridgeMut<'a> {
    orbit: &'a mut EtherOrbitState,
    ram: &'a mut [u8],
}

impl<'a> NativeEtherOrbitBridgeMut<'a> {
    pub(crate) fn new(orbit: &'a mut EtherOrbitState, ram: &'a mut [u8]) -> Self {
        *orbit = EtherOrbitState::load_from_ram(ram);
        Self { orbit, ram }
    }

    fn sync(&mut self) {
        self.orbit.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.orbit, EtherOrbitState::load_from_ram(self.ram));
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        self.orbit.set_angle(slot, value);
        self.sync();
    }

    pub(crate) fn advance_angle(&mut self, slot: usize) -> u8 {
        let next = self.orbit.advance_angle(slot);
        self.sync();
        next
    }

    pub(crate) fn set_radius(&mut self, value: u8) {
        self.orbit.set_radius(value);
        self.sync();
    }

    pub(crate) fn tick_spin_countdown(&mut self) -> u8 {
        let value = self.orbit.tick_spin_countdown();
        self.sync();
        value
    }

    pub(crate) fn set_spin_countdown(&mut self, value: u8) {
        self.orbit.set_spin_countdown(value);
        self.sync();
    }

    pub(crate) fn set_beam_top_bucket(&mut self, value: u8) {
        self.orbit.set_beam_top_bucket(value);
        self.sync();
    }

    pub(crate) fn initialize_beam_adjusted_y(&mut self, value: u16) {
        self.orbit.initialize_beam_adjusted_y(value);
        self.sync();
    }

    pub(crate) fn set_orb_position(&mut self, x: u16, y: u16) {
        self.orbit.set_orb_position(x, y);
        self.sync();
    }

    pub(crate) fn set_orbit_position(&mut self, x: u16, y: u16) {
        self.orbit.set_orbit_position(x, y);
        self.sync();
    }

    pub(crate) fn set_swordbeam_temp(&mut self, x: u16, y: u16) {
        self.orbit.set_swordbeam_temp(x, y);
        self.sync();
    }

    pub(crate) fn set_beam_y(&mut self, value: u16) {
        self.orbit.set_beam_y(value);
        self.sync();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ChainChompHistoryState {
    x_positions: Vec<u16>,
    y_positions: Vec<u16>,
}

impl Default for ChainChompHistoryState {
    fn default() -> Self {
        Self {
            x_positions: vec![0; CHAIN_CHOMP_HISTORY_LEN],
            y_positions: vec![0; CHAIN_CHOMP_HISTORY_LEN],
        }
    }
}

impl ChainChompHistoryState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut x_positions = vec![0; CHAIN_CHOMP_HISTORY_LEN];
        let mut y_positions = vec![0; CHAIN_CHOMP_HISTORY_LEN];
        for position in 0..CHAIN_CHOMP_HISTORY_LEN {
            x_positions[position] = read_le_u16(ram, CHAIN_CHOMP_HISTORY_X + position * 2);
            y_positions[position] = read_le_u16(ram, CHAIN_CHOMP_HISTORY_Y + position * 2);
        }
        Self {
            x_positions,
            y_positions,
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for position in 0..CHAIN_CHOMP_HISTORY_LEN {
            write_le_u16(
                ram,
                CHAIN_CHOMP_HISTORY_X + position * 2,
                self.x_positions[position],
            );
            write_le_u16(
                ram,
                CHAIN_CHOMP_HISTORY_Y + position * 2,
                self.y_positions[position],
            );
        }
    }

    pub(crate) fn x(&self, position: usize) -> u16 {
        self.x_positions.get(position).copied().unwrap_or(0)
    }

    pub(crate) fn y(&self, position: usize) -> u16 {
        self.y_positions.get(position).copied().unwrap_or(0)
    }

    pub(crate) fn set_x(&mut self, position: usize, value: u16) {
        if let Some(x) = self.x_positions.get_mut(position) {
            *x = value;
        }
    }

    pub(crate) fn set_y(&mut self, position: usize, value: u16) {
        if let Some(y) = self.y_positions.get_mut(position) {
            *y = value;
        }
    }
}

pub(crate) struct NativeChainChompHistoryBridgeMut<'a> {
    history: &'a mut ChainChompHistoryState,
    ram: &'a mut [u8],
}

impl<'a> NativeChainChompHistoryBridgeMut<'a> {
    pub(crate) fn new(history: &'a mut ChainChompHistoryState, ram: &'a mut [u8]) -> Self {
        *history = ChainChompHistoryState::load_from_ram(ram);
        Self { history, ram }
    }

    fn sync(&mut self) {
        self.history.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.history,
            ChainChompHistoryState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn set_x(&mut self, position: usize, value: u16) {
        self.history.set_x(position, value);
        self.sync();
    }

    pub(crate) fn set_y(&mut self, position: usize, value: u16) {
        self.history.set_y(position, value);
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

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
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

fn write_tagalong_bank(ram: &mut [u8], base: usize, bank: [u8; TAGALONG_SLOT_COUNT]) {
    for (slot, value) in bank.iter().copied().enumerate() {
        ram[base + slot] = value;
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

pub(crate) struct TagalongSlotView<'a> {
    state: &'a TagalongTrailState,
    slot: usize,
}

impl<'a> TagalongSlotView<'a> {
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
    state: &'a mut TagalongTrailState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeTagalongSlotBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut TagalongTrailState, ram: &'a mut [u8], slot: usize) -> Self {
        *state = TagalongTrailState::load_from_ram(ram);
        Self { state, ram, slot }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        if self.slot < TAGALONG_SLOT_COUNT {
            self.state.x_low[self.slot] = value as u8;
            self.state.x_high[self.slot] = (value >> 8) as u8;
            self.sync();
        }
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        if self.slot < TAGALONG_SLOT_COUNT {
            self.state.y_low[self.slot] = value as u8;
            self.state.y_high[self.slot] = (value >> 8) as u8;
            self.sync();
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
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, TagalongTrailState::load_from_ram(self.ram));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EnemyDamageSubclassTableState {
    subclasses: Vec<u8>,
}

impl Default for EnemyDamageSubclassTableState {
    fn default() -> Self {
        Self {
            subclasses: vec![0; ENEMY_DAMAGE_SUBCLASS_COUNT],
        }
    }
}

impl EnemyDamageSubclassTableState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut subclasses = vec![0; ENEMY_DAMAGE_SUBCLASS_COUNT];
        for (index, subclass) in subclasses.iter_mut().enumerate() {
            *subclass = ram.get(ENEMY_DAMAGE_DATA + index).copied().unwrap_or(0);
        }
        Self { subclasses }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for (index, subclass) in self.subclasses.iter().copied().enumerate() {
            if index >= ENEMY_DAMAGE_SUBCLASS_COUNT {
                break;
            }
            ram[ENEMY_DAMAGE_DATA + index] = subclass;
        }
    }

    pub(crate) fn entry(&self, index: usize) -> u8 {
        self.subclasses.get(index).copied().unwrap_or(0)
    }

    fn set_entry(&mut self, index: usize, value: u8) -> bool {
        let Some(entry) = self.subclasses.get_mut(index) else {
            return false;
        };
        *entry = value;
        true
    }

    fn load_from_packed_nibbles(&mut self, data: &[u8]) {
        for index in (0..ENEMY_DAMAGE_SUBCLASS_COUNT).step_by(2) {
            let packed = data.get(index >> 1).copied().unwrap_or(0);
            self.subclasses[index] = packed >> 4;
            self.subclasses[index + 1] = packed & 0x0f;
        }
    }
}

pub(crate) struct EnemyDamageSubclassTableView<'a> {
    state: &'a EnemyDamageSubclassTableState,
}

impl<'a> EnemyDamageSubclassTableView<'a> {
    pub(crate) fn new(state: &'a EnemyDamageSubclassTableState) -> Self {
        Self { state }
    }

    pub(crate) fn entry(&self, index: usize) -> u8 {
        self.state.entry(index)
    }
}

pub(crate) struct NativeEnemyDamageSubclassTableBridgeMut<'a> {
    state: &'a mut EnemyDamageSubclassTableState,
    ram: &'a mut [u8],
}

impl<'a> NativeEnemyDamageSubclassTableBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EnemyDamageSubclassTableState, ram: &'a mut [u8]) -> Self {
        *state = EnemyDamageSubclassTableState::load_from_ram(ram);
        Self { state, ram }
    }

    pub(crate) fn set_entry(&mut self, index: usize, value: u8) {
        if self.state.set_entry(index, value) {
            self.ram[ENEMY_DAMAGE_DATA + index] = value;
            self.debug_assert_entry_matches_ram(index);
        }
    }

    pub(crate) fn load_from_packed_nibbles(&mut self, data: &[u8]) {
        self.state.load_from_packed_nibbles(data);
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_entry_matches_ram(&self, index: usize) {
        debug_assert_eq!(
            self.state.entry(index),
            self.ram
                .get(ENEMY_DAMAGE_DATA + index)
                .copied()
                .unwrap_or(0)
        );
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            EnemyDamageSubclassTableState::load_from_ram(self.ram)
        );
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
