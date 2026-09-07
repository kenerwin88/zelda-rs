use crate::game_state::constants::{
    ALT_SPRITES_FLAG, ALT_SPRITE_GRAPHICS, ALT_SPRITE_SPAWNED_FLAG, ALT_SPRITE_STATE,
    ALT_SPRITE_TYPE, ALT_SPRITE_X_HI, ALT_SPRITE_X_LO, ALT_SPRITE_Y_HI, ALT_SPRITE_Y_LO,
    ANCILLA_AUX_TIMER, ANCILLA_ITEM_TO_LINK, ANCILLA_STEP, ANCILLA_TIMER, ANCILLA_X_HI,
    ANCILLA_X_LO, ANCILLA_Y_HI, ANCILLA_Y_LO, BLIND_HEAD_ANIM_COUNTER, CACHED_SPRITE_ALT_FIELDS,
    CACHED_SPRITE_LIVE_FIELDS, CHAIN_CHOMP_HISTORY_X, CHAIN_CHOMP_HISTORY_Y, CUR_OBJECT_INDEX,
    CUR_SPRITE_X, CUR_SPRITE_Y, DRAW_WORK_FLAGS_HI, DRAW_WORK_POSITION_X, DRAW_WORK_POSITION_Y,
    DUAL_LAYER_TILE_CACHE, ENEMY_DAMAGE_DATA, ETHER_ANGLE, ETHER_BEAM_TOP_BUCKET, ETHER_BEAM_Y,
    ETHER_ORBIT_X, ETHER_ORBIT_Y, ETHER_ORB_X, ETHER_ORB_Y, ETHER_RADIUS, ETHER_SPIN_COUNTDOWN,
    HITBOX_WORK_X_OFFSET, HITBOX_WORK_Y_OFFSET, MAZE_GAME_TIMER_HI, MAZE_GAME_TIMER_LO,
    MAZE_GAME_TIMER_SNAPSHOT_HI, MAZE_GAME_TIMER_SNAPSHOT_LO, OVERLORD_FLOOR, OVERLORD_GEN1,
    OVERLORD_GEN2, OVERLORD_GEN3, OVERLORD_X_HI, OVERLORD_X_LO, OVERLORD_Y_HI, OVERLORD_Y_LO,
    OVERWORLD_SPRITE_PRESENCE, OVERWORLD_SPRITE_WAS_LOADED, PLAYER_IS_INDOORS, PRIZE_DROP_CYCLE,
    SPRITE_A, SPRITE_AI_STATE, SPRITE_ALERT_FLAG, SPRITE_ANIM_CLOCK, SPRITE_B, SPRITE_BUMP_DAMAGE,
    SPRITE_C, SPRITE_CHR_HALFSLOT_STATE, SPRITE_D, SPRITE_DEFL_BITS, SPRITE_DELAY_AUX1,
    SPRITE_DELAY_AUX2, SPRITE_DELAY_AUX3, SPRITE_DELAY_AUX4, SPRITE_DELAY_MAIN, SPRITE_DIE_ACTION,
    SPRITE_DRAW_I, SPRITE_DRAW_PRIORITY_OVERRIDE, SPRITE_DRAW_WORK_BYTE_1, SPRITE_DRAW_WORK_BYTE_2,
    SPRITE_DRAW_WORK_BYTE_3, SPRITE_DRAW_WORK_BYTE_4, SPRITE_DRAW_WORK_BYTE_5, SPRITE_E, SPRITE_F,
    SPRITE_FLAGS, SPRITE_FLAGS2, SPRITE_FLAGS3, SPRITE_FLAGS4, SPRITE_FLAGS5, SPRITE_FLOOR,
    SPRITE_G, SPRITE_GFX_SUBSET_0, SPRITE_GRAPHICS, SPRITE_GRAPHICS_INDEX,
    SPRITE_GRAPHICS_INDEX_EXIT, SPRITE_GRAPHICS_INDEX_SPEXIT, SPRITE_HEAD_DIR, SPRITE_HEALTH,
    SPRITE_HIT_TIMER, SPRITE_IGNORE_PROJECTILE, SPRITE_INCOMING_DAMAGE, SPRITE_LAST_GARNISH_INDEX,
    SPRITE_LIMIT_INSTANCE, SPRITE_LOAD_BLOCK_STATE, SPRITE_N, SPRITE_OAM_FLAGS, SPRITE_OAM_PREP_X,
    SPRITE_OAM_PREP_Y, SPRITE_OBJ_PRIO, SPRITE_PAUSE, SPRITE_PICKUP_SLOT_CACHE,
    SPRITE_RESET_WORK_A, SPRITE_RESET_WORK_B, SPRITE_ROOM, SPRITE_ROOM_ORIGIN_X_HI,
    SPRITE_ROOM_ORIGIN_Y_HI, SPRITE_SHARED_WORK_A, SPRITE_STATE, SPRITE_STUNNED, SPRITE_SUBTYPE,
    SPRITE_SUBTYPE2, SPRITE_TILETYPE, SPRITE_TYPE, SPRITE_WALL_COLLISION, SPRITE_WHERE_IN_ROOM,
    SPRITE_X_HI, SPRITE_X_LO, SPRITE_X_RECOIL, SPRITE_X_SUBPIXEL, SPRITE_X_VELOCITY, SPRITE_Y_HI,
    SPRITE_Y_LO, SPRITE_Y_RECOIL, SPRITE_Y_SUBPIXEL, SPRITE_Y_VELOCITY, SPRITE_Z,
    SPRITE_Z_SUBPIXEL, SPRITE_Z_VELOCITY, SPR_RANGED_BASED_TOGGLER,
};
use crate::types::{read_le_u16, write_le_u16};

pub(crate) use super::ancilla::{
    AncillaSlotsState, NativeAncillaSlotBridgeMut, NativeAncillaSlotView, ANCILLA_SLOT_COUNT,
};
pub(crate) use super::follower::{
    FollowerRuntimeState, NativeFollowerRuntimeBridgeMut, NativeTagalongSlotBridgeMut,
    TagalongSlotRead, TagalongTrailState,
};
pub(crate) use super::garnish::{
    GarnishRuntimeState, GarnishSlotsState, NativeGarnishRuntimeBridgeMut,
    NativeGarnishSlotBridgeMut, NativeGarnishSlotView,
};
pub(crate) use super::overlord::{
    NativeOverlordSlotBridgeMut, NativeOverlordSlotView, OverlordSlotsState,
};

const SPRITE_SLOT_COUNT: usize = 16;
const CHAIN_CHOMP_HISTORY_LEN: usize = 0x80;
const ETHER_ANGLE_COUNT: usize = 8;
const ENEMY_DAMAGE_SUBCLASS_COUNT: usize = 0x1000;
pub(crate) const OVERWORLD_SPRITE_FLAG_COUNT: usize = 0x200;

/// One WRAM statement in `Dungeon_CacheTransSprites`, in exact C source order.
/// `StateClear` and `State` intentionally share the same destination: the C
/// routine clears the cache slot before conditionally copying the live state.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum CachedSpriteCacheField {
    StateClear,
    Type,
    XLow,
    Graphics,
    XHigh,
    YLow,
    YHigh,
    State,
    A,
    HeadDirection,
    OamFlags,
    ObjPriority,
    D,
    Flags2,
    Floor,
    SpawnedFlag,
    Flags3,
    B,
    C,
    E,
    Subtype2,
    HeightAboveShadow,
    DelayMain,
    I,
    IgnoreProjectile,
}

impl CachedSpriteCacheField {
    pub(crate) const C_SOURCE_ORDER: [Self; 25] = [
        Self::StateClear,
        Self::Type,
        Self::XLow,
        Self::Graphics,
        Self::XHigh,
        Self::YLow,
        Self::YHigh,
        Self::State,
        Self::A,
        Self::HeadDirection,
        Self::OamFlags,
        Self::ObjPriority,
        Self::D,
        Self::Flags2,
        Self::Floor,
        Self::SpawnedFlag,
        Self::Flags3,
        Self::B,
        Self::C,
        Self::E,
        Self::Subtype2,
        Self::HeightAboveShadow,
        Self::DelayMain,
        Self::I,
        Self::IgnoreProjectile,
    ];

    const fn live_field_index(self) -> Option<usize> {
        Some(match self {
            Self::StateClear => return None,
            Self::State => 0,
            Self::Type => 1,
            Self::XLow => 2,
            Self::XHigh => 3,
            Self::YLow => 4,
            Self::YHigh => 5,
            Self::Graphics => 6,
            Self::A => 7,
            Self::HeadDirection => 8,
            Self::OamFlags => 9,
            Self::ObjPriority => 10,
            Self::D => 11,
            Self::Flags2 => 12,
            Self::Floor => 13,
            Self::SpawnedFlag => 14,
            Self::Flags3 => 15,
            Self::B => 16,
            Self::C => 17,
            Self::E => 18,
            Self::Subtype2 => 19,
            Self::HeightAboveShadow => 20,
            Self::DelayMain => 21,
            Self::I => 22,
            Self::IgnoreProjectile => 23,
        })
    }

    pub(crate) const fn alt_address(self) -> usize {
        match self.live_field_index() {
            Some(index) => CACHED_SPRITE_ALT_FIELDS[index],
            None => ALT_SPRITE_STATE,
        }
    }

    const fn live_address(self) -> Option<usize> {
        match self.live_field_index() {
            Some(index) => Some(CACHED_SPRITE_LIVE_FIELDS[index]),
            None => None,
        }
    }
}
// `sprite_where_in_overworld` (presence markers, 0x1df80) is indexed by `blk`
// DIRECTLY (C: `sprite_where_in_overworld[blk]`), and blk spans 0..0xfff, so the
// table is 0x1000 bytes (0x1df80..0x1ef80, ending exactly at the was-loaded
// bitmask). The 0x200 FLAG_COUNT above is only correct for the was-loaded table,
// which is indexed by `blk >> 3`.
pub(crate) const OVERWORLD_SPRITE_PRESENCE_COUNT: usize = 0x1000;
const SPRITE_GRAPHICS_SUBSET_COUNT: usize = 4;
const SPRITE_ZERO_PAGE_WORK_COUNT: usize = 16;
const SPRITE_WHERE_IN_ROOM_BYTES: usize = 0x1000;
const CACHED_SPRITE_SLOT_COUNT: usize = 0x1b;
const BOSS_HOME_POSITION_COUNT: usize = 0x1b;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteState {
    pub(crate) system: SpriteSystemState,
    pub(crate) workspace: SpriteWorkspaceState,
    pub(crate) sprite_slots: SpriteSlotsState,
    pub(crate) ancilla_slots: AncillaSlotsState,
    pub(crate) overlord_slots: OverlordSlotsState,
    pub(crate) garnish_slots: GarnishSlotsState,
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
    pub(crate) failed_spin_sparkle_spawn: FailedSpinSparkleSpawnState,
    pub(crate) garnish_runtime: GarnishRuntimeState,
    pub(crate) follower_runtime: FollowerRuntimeState,
    pub(crate) cached_sprites: CachedSpritesState,
    pub(crate) boss_home_positions: BossHomePositionsState,
}

impl SpriteState {
    /// Resync the ancilla slot models from RAM. Used after code writes WRAM that the
    /// ancilla model also covers but doesn't go through the ancilla bridge — e.g. the
    /// boomerang temp coords (BOOMERANG_TEMP_X/Y at 0x399-0x39c) alias ANCILLA_G[5..9],
    /// so writing them leaves the ancilla model stale and its bulk projection would
    /// re-stamp the frame-start value over the fresh temp write (f370034).
    pub(crate) fn reload_ancilla_slots_from_ram(&mut self, ram: &[u8]) {
        self.ancilla_slots = AncillaSlotsState::load_from_ram(ram);
    }

    pub(crate) fn reload_overlord_slots_from_ram(&mut self, ram: &[u8]) {
        self.overlord_slots = OverlordSlotsState::load_from_ram(ram);
    }

    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            system: SpriteSystemState::load_from_ram(ram),
            workspace: SpriteWorkspaceState::load_from_ram(ram),
            sprite_slots: SpriteSlotsState::load_from_ram(ram),
            ancilla_slots: AncillaSlotsState::load_from_ram(ram),
            overlord_slots: OverlordSlotsState::load_from_ram(ram),
            garnish_slots: GarnishSlotsState::load_from_ram(ram),
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
            failed_spin_sparkle_spawn: FailedSpinSparkleSpawnState::load_from_ram(ram),
            garnish_runtime: GarnishRuntimeState::load_from_ram(ram),
            follower_runtime: FollowerRuntimeState::load_from_ram(ram),
            cached_sprites: CachedSpritesState::load_from_ram(ram),
            boss_home_positions: BossHomePositionsState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.system.write_to_ram(ram);
        self.workspace.write_to_ram(ram);
        self.sprite_slots.write_to_ram(ram);
        self.ancilla_slots.write_to_ram(ram);
        self.overlord_slots.write_to_ram(ram);
        self.garnish_slots.write_to_ram(ram);
        // The 0x1fe00 window is C's `beamos_x_hi[]` sprite-history bank, borrowed by
        // mutually-exclusive systems: word_7FFE00 (maze-race timer), byte_7FFE01 (Zelda
        // rescue cutscene state) and beamos_x_hi[0]/[1] (digging-game prize spawn and
        // attempt count). C never re-stamps it -- whichever system is running owns it.
        // Bulk-projecting all three every frame made it last-writer-wins, the same shape
        // as the $7F58xx ancilla scratch. Write-through instead; the sprite-history bank
        // at the same base already is.
        self.prize_drop_cycle.write_to_ram(ram);
        self.dual_layer_tile_cache.write_to_ram(ram);
        self.draw_hitbox_work.write_to_ram(ram);
        self.enemy_damage_subclasses.write_to_ram(ram);
        self.tagalong_trail.write_to_ram(ram);
        self.chain_chomp_history.write_to_ram(ram);
        // ether_orbit models part of the C-aliased $7F58xx ancilla scratch (it owns the
        // swordbeam_temp_x/y alias too) and is write-through, not bulk-projected — see the
        // note in EffectsState::write_to_ram.
        self.overworld_sprite_presence.write_to_ram(ram);
        self.overworld_sprite_loaded.write_to_ram(ram);
        self.failed_spin_sparkle_spawn.write_to_ram(ram);
        self.garnish_runtime.write_to_ram(ram);
        self.follower_runtime.write_to_ram(ram);
    }

    /// Leaf-level coherence drill-down (see GameState::report_incoherent_with_ram):
    /// returns "sprites.<leaf>" for each sub-state that has drifted out of sync with RAM.
    pub(crate) fn report_incoherent_with_ram(&self, ram: &[u8]) -> Vec<&'static str> {
        let fresh = Self::load_from_ram(ram);
        let mut out = Vec::new();
        macro_rules! check {
            ($field:ident) => {
                if self.$field != fresh.$field {
                    out.push(concat!("sprites.", stringify!($field)));
                }
            };
        }
        check!(system);
        check!(workspace);
        check!(sprite_slots);
        check!(ancilla_slots);
        check!(overlord_slots);
        check!(garnish_slots);
        check!(maze_game_timer);
        check!(prize_drop_cycle);
        check!(dual_layer_tile_cache);
        check!(draw_hitbox_work);
        check!(enemy_damage_subclasses);
        check!(tagalong_trail);
        check!(chain_chomp_history);
        check!(ether_orbit);
        check!(overworld_sprite_presence);
        check!(overworld_sprite_loaded);
        check!(failed_spin_sparkle_spawn);
        check!(garnish_runtime);
        check!(follower_runtime);
        check!(cached_sprites);
        check!(boss_home_positions);
        out
    }
}

const SPRITE_SLOTS_FIELD_RANGES: &[(usize, usize)] = &[
    (SPRITE_STUNNED, SPRITE_SLOT_COUNT),
    (SPRITE_FLAGS, SPRITE_SLOT_COUNT),
    (SPRITE_OBJ_PRIO, SPRITE_SLOT_COUNT),
    (SPRITE_IGNORE_PROJECTILE, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_WORK_BYTE_2, SPRITE_SLOT_COUNT),
    (SPRITE_N, SPRITE_SLOT_COUNT * 2),
    (SPRITE_FLAGS5, SPRITE_SLOT_COUNT),
    (SPRITE_ROOM, SPRITE_SLOT_COUNT),
    (SPRITE_DEFL_BITS, SPRITE_SLOT_COUNT),
    (SPRITE_DIE_ACTION, SPRITE_SLOT_COUNT),
    (SPRITE_BUMP_DAMAGE, SPRITE_SLOT_COUNT),
    (SPRITE_INCOMING_DAMAGE, SPRITE_SLOT_COUNT),
    (SPRITE_Y_LO, SPRITE_SLOT_COUNT),
    (SPRITE_X_LO, SPRITE_SLOT_COUNT),
    (SPRITE_Y_HI, SPRITE_SLOT_COUNT),
    (SPRITE_X_HI, SPRITE_SLOT_COUNT),
    (SPRITE_Y_VELOCITY, SPRITE_SLOT_COUNT),
    (SPRITE_X_VELOCITY, SPRITE_SLOT_COUNT),
    (SPRITE_Y_SUBPIXEL, SPRITE_SLOT_COUNT),
    (SPRITE_X_SUBPIXEL, SPRITE_SLOT_COUNT),
    (SPRITE_AI_STATE, SPRITE_SLOT_COUNT),
    (SPRITE_A, SPRITE_SLOT_COUNT),
    (SPRITE_B, SPRITE_SLOT_COUNT),
    (SPRITE_C, SPRITE_SLOT_COUNT),
    (SPRITE_GRAPHICS, SPRITE_SLOT_COUNT),
    (SPRITE_STATE, SPRITE_SLOT_COUNT),
    (SPRITE_D, SPRITE_SLOT_COUNT),
    (SPRITE_DELAY_MAIN, SPRITE_SLOT_COUNT),
    (SPRITE_DELAY_AUX1, SPRITE_SLOT_COUNT),
    (SPRITE_DELAY_AUX2, SPRITE_SLOT_COUNT),
    (SPRITE_TYPE, SPRITE_SLOT_COUNT),
    (SPRITE_SUBTYPE, SPRITE_SLOT_COUNT),
    (SPRITE_FLAGS2, SPRITE_SLOT_COUNT),
    (SPRITE_HEALTH, SPRITE_SLOT_COUNT),
    (SPRITE_FLAGS3, SPRITE_SLOT_COUNT),
    (SPRITE_WALL_COLLISION, SPRITE_SLOT_COUNT),
    (SPRITE_SUBTYPE2, SPRITE_SLOT_COUNT),
    (SPRITE_E, SPRITE_SLOT_COUNT),
    (SPRITE_F, SPRITE_SLOT_COUNT),
    (SPRITE_HEAD_DIR, SPRITE_SLOT_COUNT),
    (SPRITE_ANIM_CLOCK, SPRITE_SLOT_COUNT),
    (SPRITE_G, SPRITE_SLOT_COUNT),
    (SPRITE_DELAY_AUX3, SPRITE_SLOT_COUNT),
    (SPRITE_HIT_TIMER, SPRITE_SLOT_COUNT),
    (SPRITE_PAUSE, SPRITE_SLOT_COUNT),
    (SPRITE_DELAY_AUX4, SPRITE_SLOT_COUNT),
    (SPRITE_FLOOR, SPRITE_SLOT_COUNT),
    (SPRITE_Y_RECOIL, SPRITE_SLOT_COUNT),
    (SPRITE_X_RECOIL, SPRITE_SLOT_COUNT),
    (SPRITE_OAM_FLAGS, SPRITE_SLOT_COUNT),
    (SPRITE_FLAGS4, SPRITE_SLOT_COUNT),
    (SPRITE_Z, SPRITE_SLOT_COUNT),
    (SPRITE_Z_VELOCITY, SPRITE_SLOT_COUNT),
    (SPRITE_Z_SUBPIXEL, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_I, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_WORK_BYTE_3, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_WORK_BYTE_4, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_WORK_BYTE_5, SPRITE_SLOT_COUNT),
    (SPRITE_DRAW_WORK_BYTE_1, SPRITE_SLOT_COUNT),
];
const SPRITE_SLOTS_WORK_BASE: usize = SPRITE_STUNNED;
const SPRITE_SLOTS_WORK_END: usize = SPRITE_DRAW_WORK_BYTE_1 + SPRITE_SLOT_COUNT;
const SPRITE_SLOTS_WORK_LEN: usize = SPRITE_SLOTS_WORK_END - SPRITE_SLOTS_WORK_BASE;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteSlotsState {
    work: Vec<u8>,
}

/// Byte-addressed slot bank shared by the sprite and ancilla slot states: the
/// 8.8 / 8.16 fixed-point axis moves the ROM performs with `ADC` chains.
pub(super) trait SlotBankBytes {
    fn bank_byte(&self, slot: usize, offset: usize) -> u8;
    fn set_bank_byte(&mut self, slot: usize, offset: usize, value: u8);

    fn move_axis24(
        &mut self,
        slot: usize,
        subpixel_offset: usize,
        low_offset: usize,
        high_offset: usize,
        velocity_offset: usize,
    ) {
        let pos = u32::from(self.bank_byte(slot, subpixel_offset))
            | (u32::from(self.bank_byte(slot, low_offset)) << 8)
            | (u32::from(self.bank_byte(slot, high_offset)) << 16);
        let delta = ((self.bank_byte(slot, velocity_offset) as i8 as i32) << 4) as u32;
        let moved = pos.wrapping_add(delta);
        self.set_bank_byte(slot, subpixel_offset, moved as u8);
        self.set_bank_byte(slot, low_offset, (moved >> 8) as u8);
        self.set_bank_byte(slot, high_offset, (moved >> 16) as u8);
    }

    fn move_axis16(
        &mut self,
        slot: usize,
        subpixel_offset: usize,
        offset: usize,
        velocity_offset: usize,
    ) {
        let pos = (u16::from(self.bank_byte(slot, offset)) << 8)
            | u16::from(self.bank_byte(slot, subpixel_offset));
        let delta = ((self.bank_byte(slot, velocity_offset) as i8 as i32) << 4) as u16;
        let moved = pos.wrapping_add(delta);
        self.set_bank_byte(slot, subpixel_offset, moved as u8);
        self.set_bank_byte(slot, offset, (moved >> 8) as u8);
    }
}

impl SlotBankBytes for SpriteSlotsState {
    fn bank_byte(&self, slot: usize, offset: usize) -> u8 {
        self.byte(slot, offset)
    }
    fn set_bank_byte(&mut self, slot: usize, offset: usize, value: u8) {
        self.set_byte(slot, offset, value);
    }
}

impl Default for SpriteSlotsState {
    fn default() -> Self {
        Self {
            work: vec![0; SPRITE_SLOTS_WORK_LEN],
        }
    }
}

impl SpriteSlotsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for offset in Self::field_offsets() {
            let index = Self::work_index(offset);
            state.work[index] = ram.get(offset).copied().unwrap_or(0);
        }
        state
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        for offset in Self::field_offsets() {
            ram[offset] = self.byte_at(offset);
        }
    }

    pub(crate) fn copy_slot_from(&mut self, source: &Self, ram: &mut [u8], slot: usize) {
        assert!(slot < SPRITE_SLOT_COUNT);
        for &(base, width) in SPRITE_SLOTS_FIELD_RANGES {
            let offsets = if width == SPRITE_SLOT_COUNT {
                [base + slot, base + slot]
            } else {
                assert_eq!(width, SPRITE_SLOT_COUNT * 2);
                [base + slot * 2, base + slot * 2 + 1]
            };
            let count = if width == SPRITE_SLOT_COUNT { 1 } else { 2 };
            for &offset in &offsets[..count] {
                let value = source.byte_at(offset);
                self.set_byte_at(offset, value);
                ram[offset] = value;
            }
        }
    }

    pub(crate) fn slot(&self, slot: usize) -> NativeSpriteSlotView<'_> {
        NativeSpriteSlotView { state: self, slot }
    }

    pub(crate) fn slot_mut<'a>(
        &'a mut self,
        ram: &'a mut [u8],
        slot: usize,
    ) -> NativeSpriteSlotBridgeMut<'a> {
        NativeSpriteSlotBridgeMut {
            state: self,
            ram,
            slot,
        }
    }

    fn field_offsets() -> impl Iterator<Item = usize> {
        SPRITE_SLOTS_FIELD_RANGES
            .iter()
            .copied()
            .flat_map(|(base, width)| (0..width).map(move |offset| base + offset))
    }

    fn work_index(offset: usize) -> usize {
        offset - SPRITE_SLOTS_WORK_BASE
    }

    fn byte_at(&self, offset: usize) -> u8 {
        self.work
            .get(Self::work_index(offset))
            .copied()
            .unwrap_or(0)
    }

    fn set_byte_at(&mut self, offset: usize, value: u8) {
        crate::types::ww_check(offset, 1, "SpriteSlotsState::set_byte_at", u32::from(value));
        self.work[Self::work_index(offset)] = value;
    }

    fn byte(&self, slot: usize, base: usize) -> u8 {
        self.byte_at(base + slot)
    }

    fn set_byte(&mut self, slot: usize, base: usize, value: u8) {
        self.set_byte_at(base + slot, value);
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
}

pub(crate) struct NativeSpriteSlotView<'a> {
    state: &'a SpriteSlotsState,
    slot: usize,
}

impl<'a> NativeSpriteSlotView<'a> {
    pub(crate) fn sprite_type(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_TYPE)
    }

    pub(crate) fn state(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_STATE)
    }

    pub(crate) fn x(&self) -> u16 {
        self.state
            .packed_position(self.slot, SPRITE_X_LO, SPRITE_X_HI)
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_LO)
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_HI)
    }

    pub(crate) fn y(&self) -> u16 {
        self.state
            .packed_position(self.slot, SPRITE_Y_LO, SPRITE_Y_HI)
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_LO)
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_HI)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_VELOCITY)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_VELOCITY)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Z_VELOCITY)
    }

    pub(crate) fn x_recoil(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_RECOIL)
    }

    pub(crate) fn y_recoil(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_RECOIL)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_SUBPIXEL)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_SUBPIXEL)
    }

    pub(crate) fn z(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Z)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Z_SUBPIXEL)
    }

    pub(crate) fn ai_state(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_AI_STATE)
    }

    pub(crate) fn a(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_A)
    }

    pub(crate) fn c(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_C)
    }

    pub(crate) fn b(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_B)
    }

    pub(crate) fn e(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_E)
    }

    pub(crate) fn f(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_F)
    }

    pub(crate) fn g(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_G)
    }

    pub(crate) fn graphics(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_GRAPHICS)
    }

    pub(crate) fn direction(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_D)
    }

    pub(crate) fn subtype(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_SUBTYPE)
    }

    pub(crate) fn delay_main(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DELAY_MAIN)
    }

    pub(crate) fn delay_aux1(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DELAY_AUX1)
    }

    pub(crate) fn delay_aux4(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DELAY_AUX4)
    }

    pub(crate) fn delay_aux2(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DELAY_AUX2)
    }

    pub(crate) fn flags2(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLAGS2)
    }

    pub(crate) fn flags(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLAGS)
    }

    pub(crate) fn flags3(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLAGS3)
    }

    pub(crate) fn wall_collision(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_WALL_COLLISION)
    }

    pub(crate) fn anim_clock(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_ANIM_CLOCK)
    }

    pub(crate) fn delay_aux3(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DELAY_AUX3)
    }

    pub(crate) fn flags4(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLAGS4)
    }

    pub(crate) fn flags5(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLAGS5)
    }

    pub(crate) fn health(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_HEALTH)
    }

    pub(crate) fn hit_timer(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_HIT_TIMER)
    }

    pub(crate) fn pause(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_PAUSE)
    }

    pub(crate) fn stunned(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_STUNNED)
    }

    pub(crate) fn ignore_projectile(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_IGNORE_PROJECTILE)
    }

    pub(crate) fn draw_work_byte_2(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_WORK_BYTE_2)
    }

    pub(crate) fn n(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_N)
    }

    pub(crate) fn n_word(&self) -> u16 {
        self.state.word_at(SPRITE_N + self.slot * 2)
    }

    pub(crate) fn deflection_bits(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DEFL_BITS)
    }

    pub(crate) fn bump_damage(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_BUMP_DAMAGE)
    }

    pub(crate) fn incoming_damage(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_INCOMING_DAMAGE)
    }

    pub(crate) fn floor(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_FLOOR)
    }

    pub(crate) fn room(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_ROOM)
    }

    pub(crate) fn die_action(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DIE_ACTION)
    }

    pub(crate) fn draw_i(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_I)
    }

    pub(crate) fn draw_work_byte_3(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_WORK_BYTE_3)
    }

    pub(crate) fn draw_work_byte_4(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_WORK_BYTE_4)
    }

    pub(crate) fn draw_work_byte_5(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_WORK_BYTE_5)
    }

    pub(crate) fn draw_work_byte_1(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_DRAW_WORK_BYTE_1)
    }

    pub(crate) fn head_direction(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_HEAD_DIR)
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_OAM_FLAGS)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_OBJ_PRIO)
    }

    pub(crate) fn subtype2(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_SUBTYPE2)
    }
}

const SPRITE_PREP_RUNTIME_STATE_SOURCE_ORDER: [usize; 40] = [
    SPRITE_PAUSE,
    SPRITE_E,
    SPRITE_X_VELOCITY,
    SPRITE_Y_VELOCITY,
    SPRITE_Z_VELOCITY,
    SPRITE_X_SUBPIXEL,
    SPRITE_Y_SUBPIXEL,
    SPRITE_Z_SUBPIXEL,
    SPRITE_AI_STATE,
    SPRITE_GRAPHICS,
    SPRITE_D,
    SPRITE_DELAY_MAIN,
    SPRITE_DELAY_AUX1,
    SPRITE_DELAY_AUX2,
    SPRITE_DELAY_AUX4,
    SPRITE_HEAD_DIR,
    SPRITE_ANIM_CLOCK,
    SPRITE_G,
    SPRITE_HIT_TIMER,
    SPRITE_WALL_COLLISION,
    SPRITE_Z,
    SPRITE_HEALTH,
    SPRITE_F,
    SPRITE_X_RECOIL,
    SPRITE_Y_RECOIL,
    SPRITE_A,
    SPRITE_B,
    SPRITE_C,
    SPRITE_DRAW_WORK_BYTE_2,
    SPRITE_SUBTYPE2,
    SPRITE_IGNORE_PROJECTILE,
    SPRITE_OBJ_PRIO,
    SPRITE_OAM_FLAGS,
    SPRITE_STUNNED,
    SPRITE_INCOMING_DAMAGE,
    SPRITE_DRAW_WORK_BYTE_3,
    SPRITE_DRAW_WORK_BYTE_4,
    SPRITE_DRAW_WORK_BYTE_5,
    SPRITE_DRAW_WORK_BYTE_1,
    SPRITE_DRAW_I,
];

pub(crate) struct NativeSpriteSlotBridgeMut<'a> {
    state: &'a mut SpriteSlotsState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeSpriteSlotBridgeMut<'a> {
    fn sync(&mut self) {
        for (base, width) in SPRITE_SLOTS_FIELD_RANGES.iter().copied() {
            let offset = base + self.slot;
            if self.slot < width {
                self.ram[offset] = self.state.byte_at(offset);
            }
        }
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        for (base, width) in SPRITE_SLOTS_FIELD_RANGES.iter().copied() {
            let offset = base + self.slot;
            if self.slot < width {
                debug_assert_eq!(self.state.byte_at(offset), self.ram[offset]);
            }
        }
    }

    fn set_byte(&mut self, base: usize, value: u8) {
        self.state.set_byte(self.slot, base, value);
        self.sync();
    }

    fn set_position(&mut self, low_offset: usize, high_offset: usize, value: u16) {
        self.state
            .set_position(self.slot, low_offset, high_offset, value);
        self.sync();
    }

    fn set_word_at(&mut self, offset: usize, value: u16) {
        self.state.set_word_at(offset, value);
        self.ram[offset] = self.state.byte_at(offset);
        self.ram[offset + 1] = self.state.byte_at(offset + 1);
        debug_assert_eq!(self.state.byte_at(offset), self.ram[offset]);
        debug_assert_eq!(self.state.byte_at(offset + 1), self.ram[offset + 1]);
    }

    fn move_axis24(
        &mut self,
        subpixel_offset: usize,
        low_offset: usize,
        high_offset: usize,
        velocity_offset: usize,
    ) {
        self.state.move_axis24(
            self.slot,
            subpixel_offset,
            low_offset,
            high_offset,
            velocity_offset,
        );
        self.sync();
    }

    fn move_axis16(&mut self, subpixel_offset: usize, offset: usize, velocity_offset: usize) {
        self.state
            .move_axis16(self.slot, subpixel_offset, offset, velocity_offset);
        self.sync();
    }

    fn add_byte(&mut self, base: usize, value: u8) -> u8 {
        let next = self.state.byte(self.slot, base).wrapping_add(value);
        self.set_byte(base, next);
        next
    }

    fn subtract_byte(&mut self, base: usize, value: u8) -> u8 {
        let next = self.state.byte(self.slot, base).wrapping_sub(value);
        self.set_byte(base, next);
        next
    }

    fn xor_byte(&mut self, base: usize, value: u8) {
        let next = self.state.byte(self.slot, base) ^ value;
        self.set_byte(base, next);
    }

    fn and_byte(&mut self, base: usize, value: u8) {
        let next = self.state.byte(self.slot, base) & value;
        self.set_byte(base, next);
    }

    fn or_byte(&mut self, base: usize, value: u8) {
        let next = self.state.byte(self.slot, base) | value;
        self.set_byte(base, next);
    }

    pub(crate) fn set_sprite_type(&mut self, value: u8) {
        self.set_byte(SPRITE_TYPE, value);
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.set_byte(SPRITE_STATE, value);
    }

    pub(crate) fn increment_state(&mut self) {
        self.add_byte(SPRITE_STATE, 1);
    }

    pub(crate) fn clear(&mut self) {
        self.set_state(0);
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.set_byte(SPRITE_X_VELOCITY, value);
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.set_byte(SPRITE_Y_VELOCITY, value);
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.set_byte(SPRITE_Z_VELOCITY, value);
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_X_VELOCITY)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.state.byte(self.slot, SPRITE_Y_VELOCITY)
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) {
        self.add_byte(SPRITE_X_VELOCITY, value);
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.add_byte(SPRITE_Y_VELOCITY, value);
    }

    pub(crate) fn subtract_x_velocity(&mut self, value: u8) {
        self.subtract_byte(SPRITE_X_VELOCITY, value);
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.subtract_byte(SPRITE_Y_VELOCITY, value);
    }

    pub(crate) fn xor_x_velocity(&mut self, value: u8) {
        self.xor_byte(SPRITE_X_VELOCITY, value);
    }

    pub(crate) fn xor_y_velocity(&mut self, value: u8) {
        self.xor_byte(SPRITE_Y_VELOCITY, value);
    }

    pub(crate) fn set_ai_state(&mut self, value: u8) {
        self.set_byte(SPRITE_AI_STATE, value);
    }

    pub(crate) fn increment_ai_state(&mut self) {
        self.add_byte(SPRITE_AI_STATE, 1);
    }

    pub(crate) fn decrement_ai_state(&mut self) {
        self.subtract_byte(SPRITE_AI_STATE, 1);
    }

    pub(crate) fn add_ai_state(&mut self, value: u8) {
        self.add_byte(SPRITE_AI_STATE, value);
    }

    pub(crate) fn set_delay_main(&mut self, value: u8) {
        self.set_byte(SPRITE_DELAY_MAIN, value);
    }

    pub(crate) fn increment_delay_main(&mut self) {
        self.add_byte(SPRITE_DELAY_MAIN, 1);
    }

    pub(crate) fn add_delay_main(&mut self, value: u8) {
        self.add_byte(SPRITE_DELAY_MAIN, value);
    }

    pub(crate) fn subtract_delay_main(&mut self, value: u8) {
        self.subtract_byte(SPRITE_DELAY_MAIN, value);
    }

    pub(crate) fn set_graphics(&mut self, value: u8) {
        self.set_byte(SPRITE_GRAPHICS, value);
    }

    pub(crate) fn increment_graphics(&mut self) {
        self.add_byte(SPRITE_GRAPHICS, 1);
    }

    pub(crate) fn add_graphics(&mut self, value: u8) {
        self.add_byte(SPRITE_GRAPHICS, value);
    }

    pub(crate) fn decrement_graphics(&mut self) {
        self.subtract_byte(SPRITE_GRAPHICS, 1);
    }

    pub(crate) fn xor_graphics(&mut self, value: u8) {
        self.xor_byte(SPRITE_GRAPHICS, value);
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.set_byte(SPRITE_D, value);
    }

    pub(crate) fn add_direction(&mut self, value: u8) {
        self.add_byte(SPRITE_D, value);
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.and_byte(SPRITE_D, value);
    }

    pub(crate) fn increment_direction(&mut self) {
        self.add_byte(SPRITE_D, 1);
    }

    pub(crate) fn xor_direction(&mut self, value: u8) {
        self.xor_byte(SPRITE_D, value);
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.set_byte(SPRITE_OAM_FLAGS, value);
    }

    pub(crate) fn and_oam_flags(&mut self, value: u8) {
        self.and_byte(SPRITE_OAM_FLAGS, value);
    }

    pub(crate) fn xor_oam_flags(&mut self, value: u8) {
        self.xor_byte(SPRITE_OAM_FLAGS, value);
    }

    pub(crate) fn or_oam_flags(&mut self, value: u8) {
        self.or_byte(SPRITE_OAM_FLAGS, value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.set_byte(SPRITE_A, value);
    }

    pub(crate) fn increment_a(&mut self) {
        self.add_byte(SPRITE_A, 1);
    }

    pub(crate) fn add_a(&mut self, value: u8) {
        self.add_byte(SPRITE_A, value);
    }

    pub(crate) fn decrement_a(&mut self) {
        self.subtract_byte(SPRITE_A, 1);
    }

    pub(crate) fn subtract_a(&mut self, value: u8) {
        self.subtract_byte(SPRITE_A, value);
    }

    pub(crate) fn xor_a(&mut self, value: u8) {
        self.xor_byte(SPRITE_A, value);
    }

    pub(crate) fn set_head_direction(&mut self, value: u8) {
        self.set_byte(SPRITE_HEAD_DIR, value);
    }

    pub(crate) fn increment_head_direction(&mut self) {
        self.add_byte(SPRITE_HEAD_DIR, 1);
    }

    pub(crate) fn add_head_direction(&mut self, value: u8) {
        self.add_byte(SPRITE_HEAD_DIR, value);
    }

    pub(crate) fn decrement_head_direction(&mut self) {
        self.subtract_byte(SPRITE_HEAD_DIR, 1);
    }

    pub(crate) fn increment_head_direction_mod16(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_HEAD_DIR).wrapping_add(1) & 15;
        self.set_byte(SPRITE_HEAD_DIR, next);
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.set_byte(SPRITE_Z, value);
    }

    pub(crate) fn set_ignore_projectile(&mut self, value: u8) {
        self.set_byte(SPRITE_IGNORE_PROJECTILE, value);
    }

    pub(crate) fn set_subtype2(&mut self, value: u8) {
        self.set_byte(SPRITE_SUBTYPE2, value);
    }

    pub(crate) fn set_flags2(&mut self, value: u8) {
        self.set_byte(SPRITE_FLAGS2, value);
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.set_byte(SPRITE_X_LO, value);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        self.set_position(SPRITE_X_LO, SPRITE_X_HI, value);
    }

    pub(crate) fn set_c(&mut self, value: u8) {
        self.set_byte(SPRITE_C, value);
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.set_byte(SPRITE_B, value);
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.set_byte(SPRITE_Y_LO, value);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        self.set_position(SPRITE_Y_LO, SPRITE_Y_HI, value);
    }

    pub(crate) fn set_delay_aux1(&mut self, value: u8) {
        self.set_byte(SPRITE_DELAY_AUX1, value);
    }

    pub(crate) fn add_subtype2(&mut self, value: u8) {
        self.add_byte(SPRITE_SUBTYPE2, value);
    }

    pub(crate) fn increment_subtype2(&mut self) {
        self.add_byte(SPRITE_SUBTYPE2, 1);
    }

    pub(crate) fn increment_ignore_projectile(&mut self) {
        self.add_byte(SPRITE_IGNORE_PROJECTILE, 1);
    }

    pub(crate) fn set_flags4(&mut self, value: u8) {
        self.set_byte(SPRITE_FLAGS4, value);
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.set_byte(SPRITE_G, value);
    }

    pub(crate) fn subtract_z_velocity(&mut self, value: u8) {
        self.subtract_byte(SPRITE_Z_VELOCITY, value);
    }

    pub(crate) fn set_hit_timer(&mut self, value: u8) {
        self.set_byte(SPRITE_HIT_TIMER, value);
    }

    pub(crate) fn set_flags3(&mut self, value: u8) {
        self.set_byte(SPRITE_FLAGS3, value);
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.set_byte(SPRITE_X_HI, value);
    }

    pub(crate) fn set_e(&mut self, value: u8) {
        self.set_byte(SPRITE_E, value);
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.set_byte(SPRITE_Y_HI, value);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.set_byte(SPRITE_FLOOR, value);
    }

    pub(crate) fn set_deflection_bits(&mut self, value: u8) {
        self.set_byte(SPRITE_DEFL_BITS, value);
    }

    pub(crate) fn set_delay_aux4(&mut self, value: u8) {
        self.set_byte(SPRITE_DELAY_AUX4, value);
    }

    pub(crate) fn set_health(&mut self, value: u8) {
        self.set_byte(SPRITE_HEALTH, value);
    }

    pub(crate) fn set_subtype(&mut self, value: u8) {
        self.set_byte(SPRITE_SUBTYPE, value);
    }

    pub(crate) fn set_delay_aux2(&mut self, value: u8) {
        self.set_byte(SPRITE_DELAY_AUX2, value);
    }

    pub(crate) fn set_f(&mut self, value: u8) {
        self.set_byte(SPRITE_F, value);
    }

    pub(crate) fn set_anim_clock(&mut self, value: u8) {
        self.set_byte(SPRITE_ANIM_CLOCK, value);
    }

    pub(crate) fn set_bump_damage(&mut self, value: u8) {
        self.set_byte(SPRITE_BUMP_DAMAGE, value);
    }

    pub(crate) fn set_flags5(&mut self, value: u8) {
        self.set_byte(SPRITE_FLAGS5, value);
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.set_byte(SPRITE_OBJ_PRIO, value);
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.add_byte(SPRITE_X_LO, value);
    }

    pub(crate) fn or_flags3(&mut self, value: u8) {
        self.or_byte(SPRITE_FLAGS3, value);
    }

    pub(crate) fn add_b(&mut self, value: u8) {
        self.add_byte(SPRITE_B, value);
    }

    pub(crate) fn set_stunned(&mut self, value: u8) {
        self.set_byte(SPRITE_STUNNED, value);
    }

    pub(crate) fn add_g(&mut self, value: u8) {
        self.add_byte(SPRITE_G, value);
    }

    pub(crate) fn and_flags3(&mut self, value: u8) {
        self.and_byte(SPRITE_FLAGS3, value);
    }

    pub(crate) fn set_incoming_damage(&mut self, value: u8) {
        self.set_byte(SPRITE_INCOMING_DAMAGE, value);
    }

    pub(crate) fn set_n(&mut self, value: u8) {
        self.set_byte(SPRITE_N, value);
    }

    pub(crate) fn set_n_word(&mut self, value: u16) {
        self.set_word_at(SPRITE_N + self.slot * 2, value);
    }

    pub(crate) fn or_object_priority(&mut self, value: u8) {
        self.or_byte(SPRITE_OBJ_PRIO, value);
    }

    pub(crate) fn set_x_recoil(&mut self, value: u8) {
        self.set_byte(SPRITE_X_RECOIL, value);
    }

    pub(crate) fn set_y_recoil(&mut self, value: u8) {
        self.set_byte(SPRITE_Y_RECOIL, value);
    }

    pub(crate) fn add_flags2(&mut self, value: u8) {
        self.add_byte(SPRITE_FLAGS2, value);
    }

    pub(crate) fn add_y_low(&mut self, value: u8) {
        self.add_byte(SPRITE_Y_LO, value);
    }

    pub(crate) fn set_die_action(&mut self, value: u8) {
        self.set_byte(SPRITE_DIE_ACTION, value);
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.add_byte(SPRITE_Z_VELOCITY, value);
    }

    pub(crate) fn add_c(&mut self, value: u8) {
        self.add_byte(SPRITE_C, value);
    }

    pub(crate) fn or_deflection_bits(&mut self, value: u8) {
        self.or_byte(SPRITE_DEFL_BITS, value);
    }

    pub(crate) fn set_delay_aux3(&mut self, value: u8) {
        self.set_byte(SPRITE_DELAY_AUX3, value);
    }

    pub(crate) fn and_flags2(&mut self, value: u8) {
        self.and_byte(SPRITE_FLAGS2, value);
    }

    pub(crate) fn increment_g(&mut self) {
        self.add_byte(SPRITE_G, 1);
    }

    pub(crate) fn set_pause(&mut self, value: u8) {
        self.set_byte(SPRITE_PAUSE, value);
    }

    pub(crate) fn set_room(&mut self, value: u8) {
        self.set_byte(SPRITE_ROOM, value);
    }

    pub(crate) fn set_wall_collision(&mut self, value: u8) {
        self.set_byte(SPRITE_WALL_COLLISION, value);
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.set_byte(SPRITE_Z_SUBPIXEL, value);
    }

    pub(crate) fn subtract_y_low(&mut self, value: u8) {
        self.subtract_byte(SPRITE_Y_LO, value);
    }

    pub(crate) fn add_z(&mut self, value: u8) {
        self.add_byte(SPRITE_Z, value);
    }

    pub(crate) fn clear_flags3_bits(&mut self, mask: u8) {
        let next = self.state.byte(self.slot, SPRITE_FLAGS3) & !mask;
        self.set_byte(SPRITE_FLAGS3, next);
    }

    pub(crate) fn decrement_subtype2(&mut self) -> u8 {
        self.subtract_byte(SPRITE_SUBTYPE2, 1)
    }

    pub(crate) fn or_flags2(&mut self, value: u8) {
        self.or_byte(SPRITE_FLAGS2, value);
    }

    pub(crate) fn or_object_priority_bits(&mut self, value: u8) {
        self.or_byte(SPRITE_OBJ_PRIO, value);
    }

    pub(crate) fn subtract_flags2(&mut self, value: u8) {
        self.subtract_byte(SPRITE_FLAGS2, value);
    }

    pub(crate) fn add_subtype(&mut self, value: u8) {
        self.add_byte(SPRITE_SUBTYPE, value);
    }

    pub(crate) fn decrement_z_velocity(&mut self) {
        self.subtract_byte(SPRITE_Z_VELOCITY, 1);
    }

    pub(crate) fn increment_anim_clock(&mut self) {
        self.add_byte(SPRITE_ANIM_CLOCK, 1);
    }

    pub(crate) fn increment_b(&mut self) {
        self.add_byte(SPRITE_B, 1);
    }

    pub(crate) fn increment_c(&mut self) {
        self.add_byte(SPRITE_C, 1);
    }

    pub(crate) fn increment_e(&mut self) -> u8 {
        self.add_byte(SPRITE_E, 1)
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_X_VELOCITY).wrapping_neg();
        self.set_byte(SPRITE_X_VELOCITY, next);
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_Y_VELOCITY).wrapping_neg();
        self.set_byte(SPRITE_Y_VELOCITY, next);
    }

    pub(crate) fn increment_flags4(&mut self) {
        self.add_byte(SPRITE_FLAGS4, 1);
    }

    pub(crate) fn set_flags(&mut self, value: u8) {
        self.set_byte(SPRITE_FLAGS, value);
    }

    pub(crate) fn and_deflection_bits(&mut self, value: u8) {
        self.and_byte(SPRITE_DEFL_BITS, value);
    }

    pub(crate) fn and_flags5(&mut self, value: u8) {
        self.and_byte(SPRITE_FLAGS5, value);
    }

    pub(crate) fn decrement_g(&mut self) {
        self.subtract_byte(SPRITE_G, 1);
    }

    pub(crate) fn masked_or_oam_flags(&mut self, mask: u8, value: u8) {
        let next = (self.state.byte(self.slot, SPRITE_OAM_FLAGS) & mask) | value;
        self.set_byte(SPRITE_OAM_FLAGS, next);
    }

    pub(crate) fn or_wall_collision(&mut self, value: u8) {
        self.or_byte(SPRITE_WALL_COLLISION, value);
    }

    pub(crate) fn set_draw_i(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_I, value);
    }

    pub(crate) fn set_draw_work_byte_3(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_WORK_BYTE_3, value);
    }

    pub(crate) fn subtract_b(&mut self, value: u8) {
        self.subtract_byte(SPRITE_B, value);
    }

    pub(crate) fn and_object_priority(&mut self, value: u8) {
        self.and_byte(SPRITE_OBJ_PRIO, value);
    }

    pub(crate) fn decrement_c(&mut self) {
        self.subtract_byte(SPRITE_C, 1);
    }

    pub(crate) fn decrement_subtype(&mut self) {
        self.subtract_byte(SPRITE_SUBTYPE, 1);
    }

    pub(crate) fn increment_subtype(&mut self) {
        self.add_byte(SPRITE_SUBTYPE, 1);
    }

    pub(crate) fn masked_or_flags2(&mut self, mask: u8, value: u8) {
        let next = (self.state.byte(self.slot, SPRITE_FLAGS2) & mask) | value;
        self.set_byte(SPRITE_FLAGS2, next);
    }

    pub(crate) fn masked_or_flags3(&mut self, mask: u8, value: u8) {
        let next = (self.state.byte(self.slot, SPRITE_FLAGS3) & mask) | value;
        self.set_byte(SPRITE_FLAGS3, next);
    }

    pub(crate) fn or_hit_timer(&mut self, value: u8) {
        self.or_byte(SPRITE_HIT_TIMER, value);
    }

    pub(crate) fn set_draw_work_byte_1(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_WORK_BYTE_1, value);
    }

    pub(crate) fn set_draw_work_byte_2(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_WORK_BYTE_2, value);
    }

    pub(crate) fn set_draw_work_byte_4(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_WORK_BYTE_4, value);
    }

    pub(crate) fn set_draw_work_byte_5(&mut self, value: u8) {
        self.set_byte(SPRITE_DRAW_WORK_BYTE_5, value);
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.subtract_byte(SPRITE_X_LO, value);
    }

    pub(crate) fn xor_flags3(&mut self, value: u8) {
        self.xor_byte(SPRITE_FLAGS3, value);
    }

    pub(crate) fn add_e(&mut self, value: u8) {
        self.add_byte(SPRITE_E, value);
    }

    pub(crate) fn and_bump_damage(&mut self, value: u8) {
        self.and_byte(SPRITE_BUMP_DAMAGE, value);
    }

    pub(crate) fn and_subtype(&mut self, value: u8) {
        self.and_byte(SPRITE_SUBTYPE, value);
    }

    pub(crate) fn clear_deflection_bits(&mut self, mask: u8) {
        let next = self.state.byte(self.slot, SPRITE_DEFL_BITS) & !mask;
        self.set_byte(SPRITE_DEFL_BITS, next);
    }

    pub(crate) fn clear_object_priority_bits(&mut self, mask: u8) {
        let next = self.state.byte(self.slot, SPRITE_OBJ_PRIO) & !mask;
        self.set_byte(SPRITE_OBJ_PRIO, next);
    }

    pub(crate) fn clear_prep_runtime_state_prefix(&mut self, completed_stores: u8) {
        let completed_stores = usize::from(completed_stores);
        assert!(
            completed_stores <= SPRITE_PREP_RUNTIME_STATE_SOURCE_ORDER.len(),
            "SpritePrep_ResetProperties prefix exceeds its source store count",
        );
        for &base in &SPRITE_PREP_RUNTIME_STATE_SOURCE_ORDER[..completed_stores] {
            self.set_byte(base, 0);
        }
    }

    pub(crate) fn clear_prep_runtime_state_from(&mut self, completed_stores: u8) {
        const TOTAL_STORES: u8 = SPRITE_PREP_RUNTIME_STATE_SOURCE_ORDER.len() as u8;
        assert!(
            completed_stores <= TOTAL_STORES,
            "SpritePrep_ResetProperties continuation exceeds its source store count",
        );
        for &base in &SPRITE_PREP_RUNTIME_STATE_SOURCE_ORDER[usize::from(completed_stores)..] {
            self.set_byte(base, 0);
        }
    }

    pub(crate) fn clear_prep_runtime_state(&mut self) {
        self.clear_prep_runtime_state_prefix(40);
    }

    pub(crate) fn decrement_anim_clock(&mut self) {
        self.subtract_byte(SPRITE_ANIM_CLOCK, 1);
    }

    pub(crate) fn decrement_f(&mut self) {
        self.subtract_byte(SPRITE_F, 1);
    }

    pub(crate) fn decrement_health(&mut self) {
        self.subtract_byte(SPRITE_HEALTH, 1);
    }

    pub(crate) fn decrement_z_subpixel(&mut self) -> u8 {
        self.subtract_byte(SPRITE_Z_SUBPIXEL, 1)
    }

    pub(crate) fn increment_die_action(&mut self) {
        self.add_byte(SPRITE_DIE_ACTION, 1);
    }

    pub(crate) fn increment_draw_work_byte_3(&mut self) {
        self.add_byte(SPRITE_DRAW_WORK_BYTE_3, 1);
    }

    pub(crate) fn masked_or_flags(&mut self, mask: u8, value: u8) {
        let next = (self.state.byte(self.slot, SPRITE_FLAGS) & mask) | value;
        self.set_byte(SPRITE_FLAGS, next);
    }

    pub(crate) fn negate_z_subpixel(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_Z_SUBPIXEL).wrapping_neg();
        self.set_byte(SPRITE_Z_SUBPIXEL, next);
    }

    pub(crate) fn negate_z_velocity(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_Z_VELOCITY).wrapping_neg();
        self.set_byte(SPRITE_Z_VELOCITY, next);
    }

    pub(crate) fn or_flags4(&mut self, value: u8) {
        self.or_byte(SPRITE_FLAGS4, value);
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.set_byte(SPRITE_X_SUBPIXEL, value);
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.set_byte(SPRITE_Y_SUBPIXEL, value);
    }

    pub(crate) fn subtract_f(&mut self, value: u8) {
        self.subtract_byte(SPRITE_F, value);
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.subtract_byte(SPRITE_G, value);
    }

    pub(crate) fn subtract_subtype2(&mut self, value: u8) {
        self.subtract_byte(SPRITE_SUBTYPE2, value);
    }

    pub(crate) fn xor_b(&mut self, value: u8) {
        self.xor_byte(SPRITE_B, value);
    }

    pub(crate) fn xor_c(&mut self, value: u8) {
        self.xor_byte(SPRITE_C, value);
    }

    pub(crate) fn and_x_velocity(&mut self, value: u8) {
        self.and_byte(SPRITE_X_VELOCITY, value);
    }

    pub(crate) fn and_y_velocity(&mut self, value: u8) {
        self.and_byte(SPRITE_Y_VELOCITY, value);
    }

    pub(crate) fn decrement_e(&mut self) -> u8 {
        self.subtract_byte(SPRITE_E, 1)
    }

    pub(crate) fn decrement_stunned(&mut self) {
        self.subtract_byte(SPRITE_STUNNED, 1);
    }

    pub(crate) fn decrement_wall_collision(&mut self) {
        self.subtract_byte(SPRITE_WALL_COLLISION, 1);
    }

    pub(crate) fn halve_delay_main(&mut self) {
        let next = self.state.byte(self.slot, SPRITE_DELAY_MAIN) >> 1;
        self.set_byte(SPRITE_DELAY_MAIN, next);
    }

    pub(crate) fn halve_x_velocity(&mut self) {
        let next = ((self.state.byte(self.slot, SPRITE_X_VELOCITY) as i8) >> 1) as u8;
        self.set_byte(SPRITE_X_VELOCITY, next);
    }

    pub(crate) fn halve_y_velocity(&mut self) {
        let next = ((self.state.byte(self.slot, SPRITE_Y_VELOCITY) as i8) >> 1) as u8;
        self.set_byte(SPRITE_Y_VELOCITY, next);
    }

    pub(crate) fn increment_z_subpixel(&mut self) -> u8 {
        self.add_byte(SPRITE_Z_SUBPIXEL, 1)
    }

    pub(crate) fn shift_x_velocity_left(&mut self, amount: u32) {
        let next = self
            .state
            .byte(self.slot, SPRITE_X_VELOCITY)
            .wrapping_shl(amount);
        self.set_byte(SPRITE_X_VELOCITY, next);
    }

    pub(crate) fn shift_y_velocity_left(&mut self, amount: u32) {
        let next = self
            .state
            .byte(self.slot, SPRITE_Y_VELOCITY)
            .wrapping_shl(amount);
        self.set_byte(SPRITE_Y_VELOCITY, next);
    }

    pub(crate) fn move_x(&mut self) {
        if self.state.byte(self.slot, SPRITE_X_VELOCITY) == 0 {
            return;
        }
        self.move_axis24(
            SPRITE_X_SUBPIXEL,
            SPRITE_X_LO,
            SPRITE_X_HI,
            SPRITE_X_VELOCITY,
        );
    }

    pub(crate) fn move_y(&mut self) {
        if self.state.byte(self.slot, SPRITE_Y_VELOCITY) == 0 {
            return;
        }
        self.move_axis24(
            SPRITE_Y_SUBPIXEL,
            SPRITE_Y_LO,
            SPRITE_Y_HI,
            SPRITE_Y_VELOCITY,
        );
    }

    pub(crate) fn move_z(&mut self) {
        self.move_axis16(SPRITE_Z_SUBPIXEL, SPRITE_Z, SPRITE_Z_VELOCITY);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct BossHomePosition {
    x_low: u8,
    x_high: u8,
    y_low: u8,
    y_high: u8,
}

impl BossHomePosition {
    fn x(&self) -> u16 {
        u16::from(self.x_low) | (u16::from(self.x_high) << 8)
    }

    fn y(&self) -> u16 {
        u16::from(self.y_low) | (u16::from(self.y_high) << 8)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BossHomePositionsState {
    arrghus_puff_home_positions: [BossHomePosition; BOSS_HOME_POSITION_COUNT],
    armos_knight_home_positions: [BossHomePosition; BOSS_HOME_POSITION_COUNT],
}

impl BossHomePositionsState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..BOSS_HOME_POSITION_COUNT {
            let puff_overlord_slot = slot + 7;
            state.arrghus_puff_home_positions[slot] = BossHomePosition {
                x_low: ram
                    .get(OVERLORD_X_LO + puff_overlord_slot)
                    .copied()
                    .unwrap_or(0),
                x_high: ram
                    .get(OVERLORD_Y_LO + puff_overlord_slot)
                    .copied()
                    .unwrap_or(0),
                y_low: ram
                    .get(OVERLORD_GEN1 + puff_overlord_slot)
                    .copied()
                    .unwrap_or(0),
                y_high: ram
                    .get(OVERLORD_GEN3 + puff_overlord_slot)
                    .copied()
                    .unwrap_or(0),
            };
            state.armos_knight_home_positions[slot] = BossHomePosition {
                x_low: ram.get(OVERLORD_X_HI + slot).copied().unwrap_or(0),
                x_high: ram.get(OVERLORD_Y_HI + slot).copied().unwrap_or(0),
                y_low: ram.get(OVERLORD_GEN2 + slot).copied().unwrap_or(0),
                y_high: ram.get(OVERLORD_FLOOR + slot).copied().unwrap_or(0),
            };
        }
        state
    }

    pub(crate) fn arrghus_puff_home_position(&self, puff_slot: usize) -> BossHomePositionRead {
        BossHomePositionRead {
            position: self
                .arrghus_puff_home_positions
                .get(puff_slot)
                .copied()
                .unwrap_or_default(),
        }
    }

    pub(crate) fn armos_knight_home_position(&self, slot: usize) -> BossHomePositionRead {
        BossHomePositionRead {
            position: self
                .armos_knight_home_positions
                .get(slot)
                .copied()
                .unwrap_or_default(),
        }
    }

    fn project_arrghus_puff_home_position_to_wram(&self, ram: &mut [u8], puff_slot: usize) {
        if let Some(position) = self.arrghus_puff_home_positions.get(puff_slot).copied() {
            let overlord_slot = puff_slot + 7;
            ram[OVERLORD_X_LO + overlord_slot] = position.x_low;
            ram[OVERLORD_Y_LO + overlord_slot] = position.x_high;
            ram[OVERLORD_GEN1 + overlord_slot] = position.y_low;
            ram[OVERLORD_GEN3 + overlord_slot] = position.y_high;
        }
    }

    fn project_armos_knight_home_position_to_wram(&self, ram: &mut [u8], slot: usize) {
        if let Some(position) = self.armos_knight_home_positions.get(slot).copied() {
            ram[OVERLORD_X_HI + slot] = position.x_low;
            ram[OVERLORD_Y_HI + slot] = position.x_high;
            ram[OVERLORD_GEN2 + slot] = position.y_low;
            ram[OVERLORD_FLOOR + slot] = position.y_high;
        }
    }

    fn set_armos_knight_home_position(&mut self, slot: usize, x: u16, y: u16) {
        if let Some(position) = self.armos_knight_home_positions.get_mut(slot) {
            *position = BossHomePosition {
                x_low: x as u8,
                x_high: (x >> 8) as u8,
                y_low: y as u8,
                y_high: (y >> 8) as u8,
            };
        }
    }

    fn set_arrghus_puff_home_position(&mut self, puff_slot: usize, x: u16, y: u16) {
        if let Some(position) = self.arrghus_puff_home_positions.get_mut(puff_slot) {
            *position = BossHomePosition {
                x_low: x as u8,
                x_high: (x >> 8) as u8,
                y_low: y as u8,
                y_high: (y >> 8) as u8,
            };
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BossHomePositionRead {
    position: BossHomePosition,
}

impl BossHomePositionRead {
    /// Build a home from four raw bytes (x_low, x_high, y_low, y_high). Used to read the
    /// armos-knight home out of the overlord slot array, where the armos coordinator
    /// stores it by SNES byte reuse (C reads `dung_overlord_x_hi[k] | dung_overlord_y_hi[k]<<8`
    /// for x, `gen2[k] | floor[k]<<8` for y).
    pub(crate) fn from_xy_bytes(x_low: u8, x_high: u8, y_low: u8, y_high: u8) -> Self {
        Self {
            position: BossHomePosition {
                x_low,
                x_high,
                y_low,
                y_high,
            },
        }
    }

    pub(crate) fn x(&self) -> u16 {
        self.position.x()
    }

    pub(crate) fn y(&self) -> u16 {
        self.position.y()
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.position.x_low
    }

    pub(crate) fn x_high(&self) -> u8 {
        self.position.x_high
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.position.y_low
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.position.y_high
    }
}

pub(crate) struct NativeArrghusPuffHomePositionBridgeMut<'a> {
    state: &'a mut BossHomePositionsState,
    ram: &'a mut [u8],
    puff_slot: usize,
}

impl<'a> NativeArrghusPuffHomePositionBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut BossHomePositionsState,
        ram: &'a mut [u8],
        puff_slot: usize,
    ) -> Self {
        Self {
            state,
            ram,
            puff_slot,
        }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state
            .set_arrghus_puff_home_position(self.puff_slot, x, y);
        self.state
            .project_arrghus_puff_home_position_to_wram(self.ram, self.puff_slot);
        debug_assert_eq!(
            self.state.arrghus_puff_home_position(self.puff_slot),
            BossHomePositionsState::load_from_ram(self.ram)
                .arrghus_puff_home_position(self.puff_slot),
        );
    }
}

pub(crate) struct NativeArmosKnightHomePositionBridgeMut<'a> {
    state: &'a mut BossHomePositionsState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeArmosKnightHomePositionBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut BossHomePositionsState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self { state, ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.state.set_armos_knight_home_position(self.slot, x, y);
        self.state
            .project_armos_knight_home_position_to_wram(self.ram, self.slot);
        debug_assert_eq!(
            self.state.armos_knight_home_position(self.slot),
            BossHomePositionsState::load_from_ram(self.ram).armos_knight_home_position(self.slot),
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CachedSpriteSlotState {
    state: u8,
    type_byte: u8,
    x_low: u8,
    x_high: u8,
    y_low: u8,
    y_high: u8,
    graphics: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedSpritesState {
    slots: [CachedSpriteSlotState; CACHED_SPRITE_SLOT_COUNT],
}

impl CachedSpritesState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for slot in 0..CACHED_SPRITE_SLOT_COUNT {
            state.slots[slot] = CachedSpriteSlotState {
                state: ram.get(ALT_SPRITE_STATE + slot).copied().unwrap_or(0),
                type_byte: ram.get(ALT_SPRITE_TYPE + slot).copied().unwrap_or(0),
                x_low: ram.get(ALT_SPRITE_X_LO + slot).copied().unwrap_or(0),
                x_high: ram.get(ALT_SPRITE_X_HI + slot).copied().unwrap_or(0),
                y_low: ram.get(ALT_SPRITE_Y_LO + slot).copied().unwrap_or(0),
                y_high: ram.get(ALT_SPRITE_Y_HI + slot).copied().unwrap_or(0),
                graphics: ram.get(ALT_SPRITE_GRAPHICS + slot).copied().unwrap_or(0),
            };
        }
        state
    }

    pub(crate) fn slot(&self, slot: usize) -> CachedSpriteRead {
        CachedSpriteRead {
            slot: self.slots.get(slot).copied().unwrap_or_default(),
        }
    }

    fn slot_mut(&mut self, slot: usize) -> Option<&mut CachedSpriteSlotState> {
        self.slots.get_mut(slot)
    }

    fn clear_state(&mut self, slot: usize) {
        if let Some(cached) = self.slot_mut(slot) {
            cached.state = 0;
        }
    }

    fn initialize_trinexx_component(&mut self, slot: usize) {
        if let Some(cached) = self.slot_mut(slot) {
            cached.type_byte = 0x40;
            cached.x_high = 0;
            cached.y_high = 0;
        }
    }

    fn set_type_byte(&mut self, slot: usize, value: u8) {
        if let Some(cached) = self.slot_mut(slot) {
            cached.type_byte = value;
        }
    }

    fn set_y_high(&mut self, slot: usize, value: u8) {
        if let Some(cached) = self.slot_mut(slot) {
            cached.y_high = value;
        }
    }

    fn sync_slot_from_ram(&mut self, ram: &[u8], slot: usize) {
        if let Some(cached) = self.slot_mut(slot) {
            *cached = CachedSpriteSlotState {
                state: ram.get(ALT_SPRITE_STATE + slot).copied().unwrap_or(0),
                type_byte: ram.get(ALT_SPRITE_TYPE + slot).copied().unwrap_or(0),
                x_low: ram.get(ALT_SPRITE_X_LO + slot).copied().unwrap_or(0),
                x_high: ram.get(ALT_SPRITE_X_HI + slot).copied().unwrap_or(0),
                y_low: ram.get(ALT_SPRITE_Y_LO + slot).copied().unwrap_or(0),
                y_high: ram.get(ALT_SPRITE_Y_HI + slot).copied().unwrap_or(0),
                graphics: ram.get(ALT_SPRITE_GRAPHICS + slot).copied().unwrap_or(0),
            };
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedSpriteRead {
    slot: CachedSpriteSlotState,
}

impl CachedSpriteRead {
    pub(crate) fn state(&self) -> u8 {
        self.slot.state
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state() != 0
    }

    pub(crate) fn type_byte(&self) -> u8 {
        self.slot.type_byte
    }

    pub(crate) fn y_high(&self) -> u8 {
        self.slot.y_high
    }
}

pub(crate) struct NativeCachedSpriteBridgeMut<'a> {
    state: &'a mut CachedSpritesState,
    sprite_slots: &'a mut SpriteSlotsState,
    system: &'a mut SpriteSystemState,
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeCachedSpriteBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut CachedSpritesState,
        sprite_slots: &'a mut SpriteSlotsState,
        system: &'a mut SpriteSystemState,
        ram: &'a mut [u8],
        slot: usize,
    ) -> Self {
        Self {
            state,
            sprite_slots,
            system,
            ram,
            slot,
        }
    }

    /// The cache copy/restore routines write the live sprite arrays directly in
    /// RAM (ALT_SPRITE_* <-> SPRITE_*), matching C's raw walks. Adopt the
    /// just-written bytes into the live-slot native model so a later slot sync
    /// cannot re-stamp the stale pre-copy values and native reads see the
    /// uncached sprite.
    fn adopt_live_slots_from_ram(&mut self) {
        *self.sprite_slots = SpriteSlotsState::load_from_ram(self.ram);
    }

    fn sync_slot_from_ram(&mut self) {
        self.state.sync_slot_from_ram(self.ram, self.slot);
        self.debug_assert_slot_matches_ram();
    }

    fn debug_assert_slot_matches_ram(&self) {
        debug_assert_eq!(
            self.state.slot(self.slot),
            CachedSpritesState::load_from_ram(self.ram).slot(self.slot),
        );
    }

    pub(crate) fn clear_state(&mut self) {
        self.state.clear_state(self.slot);
        self.ram[ALT_SPRITE_STATE + self.slot] = 0;
    }

    pub(crate) fn initialize_trinexx_component(&mut self) {
        self.state.initialize_trinexx_component(self.slot);
        self.ram[ALT_SPRITE_TYPE + self.slot] = 0x40;
        self.ram[ALT_SPRITE_X_HI + self.slot] = 0;
        self.ram[ALT_SPRITE_Y_HI + self.slot] = 0;
    }

    pub(crate) fn set_type_byte(&mut self, value: u8) {
        self.state.set_type_byte(self.slot, value);
        self.ram[ALT_SPRITE_TYPE + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.state.set_y_high(self.slot, value);
        self.ram[ALT_SPRITE_Y_HI + self.slot] = value;
    }

    pub(crate) fn cache_field_from_live(&mut self, field: CachedSpriteCacheField) {
        let value = field
            .live_address()
            .map(|address| self.ram[address + self.slot])
            .unwrap_or(0);
        self.write_cache_field(field, value);
    }

    fn write_cache_field(&mut self, field: CachedSpriteCacheField, value: u8) {
        self.ram[field.alt_address() + self.slot] = value;
        self.sync_slot_from_ram();
        if field == CachedSpriteCacheField::SpawnedFlag {
            // Slot 0's ai-state cache byte IS alt_sprite_spawned_flag[0]
            // (0x1de0), the byte SpriteSystemState models for the damage
            // tracker (sprite_main.c:25815).
            self.system.adopt_alt_sprite_spawned_flag_from_ram(self.ram);
        }
    }

    pub(crate) fn load_cached_into_live(&mut self, backup: &mut [u8; 24]) {
        for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
            backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
        }
        self.sync_slot_from_ram();
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn load_cached_fields_into_live_before_nmi(
        &mut self,
        backup: &mut [u8; 24],
        copied_fields: usize,
    ) {
        // The CPU cycle budget may suspend the reverse restore walk at any
        // field boundary. Fields below the cut remain cached at NMI.
        debug_assert!(copied_fields <= CACHED_SPRITE_LIVE_FIELDS.len());
        for i in 0..copied_fields {
            backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
        }
        self.sync_slot_from_ram();
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn complete_cached_dynamic_fields_into_live_after_nmi(
        &mut self,
        backup: &mut [u8; 24],
        copied_fields: usize,
    ) {
        debug_assert!(copied_fields <= CACHED_SPRITE_LIVE_FIELDS.len());
        for i in copied_fields..CACHED_SPRITE_LIVE_FIELDS.len() {
            backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
        }
        self.sync_slot_from_ram();
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn restore_live_suffix_from_backup_before_nmi(
        &mut self,
        backup: &[u8; 24],
        live_fields: usize,
    ) {
        debug_assert!(live_fields <= CACHED_SPRITE_LIVE_FIELDS.len());
        for i in (live_fields..CACHED_SPRITE_LIVE_FIELDS.len()).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn restore_live_prefix_from_backup_after_nmi(
        &mut self,
        backup: &[u8; 24],
        live_fields: usize,
    ) {
        debug_assert!(live_fields <= CACHED_SPRITE_LIVE_FIELDS.len());
        for i in (0..live_fields).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn restore_live_range_from_backup(
        &mut self,
        backup: &[u8; 24],
        remaining_before: usize,
        remaining_after: usize,
    ) {
        assert!(remaining_after <= remaining_before && remaining_before <= backup.len());
        for i in (remaining_after..remaining_before).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
        self.adopt_live_slots_from_ram();
    }

    pub(crate) fn restore_live_from_backup(&mut self, backup: &[u8; 24]) {
        for i in (0..CACHED_SPRITE_LIVE_FIELDS.len()).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
        self.adopt_live_slots_from_ram();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteSystemState {
    limit_instance: u8,
    blind_head_anim_counter: u8,
    chr_halfslot_state: u8,
    alert_flag: u8,
    graphics_index: u8,
    saved_special_exit_graphics_index: u8,
    saved_exit_graphics_index: u8,
    alt_sprite_spawned_flag: u8,
    cur_object_index: u8,
    alt_sprites_flag: u8,
    ranged_based_toggler: u8,
}

impl SpriteSystemState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            limit_instance: ram.get(SPRITE_LIMIT_INSTANCE).copied().unwrap_or(0),
            blind_head_anim_counter: ram.get(BLIND_HEAD_ANIM_COUNTER).copied().unwrap_or(0),
            chr_halfslot_state: ram.get(SPRITE_CHR_HALFSLOT_STATE).copied().unwrap_or(0),
            alert_flag: ram.get(SPRITE_ALERT_FLAG).copied().unwrap_or(0),
            graphics_index: ram.get(SPRITE_GRAPHICS_INDEX).copied().unwrap_or(0),
            saved_special_exit_graphics_index: ram
                .get(SPRITE_GRAPHICS_INDEX_SPEXIT)
                .copied()
                .unwrap_or(0),
            saved_exit_graphics_index: ram.get(SPRITE_GRAPHICS_INDEX_EXIT).copied().unwrap_or(0),
            alt_sprite_spawned_flag: ram.get(ALT_SPRITE_SPAWNED_FLAG).copied().unwrap_or(0),
            cur_object_index: ram.get(CUR_OBJECT_INDEX).copied().unwrap_or(0),
            alt_sprites_flag: ram.get(ALT_SPRITES_FLAG).copied().unwrap_or(0),
            ranged_based_toggler: ram.get(SPR_RANGED_BASED_TOGGLER).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SPRITE_LIMIT_INSTANCE] = self.limit_instance;
        ram[BLIND_HEAD_ANIM_COUNTER] = self.blind_head_anim_counter;
        ram[SPRITE_CHR_HALFSLOT_STATE] = self.chr_halfslot_state;
        ram[SPRITE_ALERT_FLAG] = self.alert_flag;
        ram[SPRITE_GRAPHICS_INDEX] = self.graphics_index;
        ram[SPRITE_GRAPHICS_INDEX_SPEXIT] = self.saved_special_exit_graphics_index;
        // NOTE: SPRITE_GRAPHICS_INDEX_EXIT (0xc167) is owned and projected by
        // DungeonEntranceBackupState (the `Dungeon_LoadEntrance` save, C
        // dungeon.c:8477). `saved_exit_graphics_index` is a load-only mirror read
        // by the restore (`graphics_index = saved_exit_graphics_index`); projecting
        // it here would clobber the authoritative save.
        ram[ALT_SPRITE_SPAWNED_FLAG] = self.alt_sprite_spawned_flag;
        ram[CUR_OBJECT_INDEX] = self.cur_object_index;
        ram[ALT_SPRITES_FLAG] = self.alt_sprites_flag;
        ram[SPR_RANGED_BASED_TOGGLER] = self.ranged_based_toggler;
    }

    /// Adopt the alt_sprite_spawned_flag byte (0x1de0) straight from RAM after the
    /// cached-sprite copy wrote it raw (it doubles as slot 0's ai-state cache).
    pub(crate) fn adopt_alt_sprite_spawned_flag_from_ram(&mut self, ram: &[u8]) {
        self.alt_sprite_spawned_flag = ram.get(ALT_SPRITE_SPAWNED_FLAG).copied().unwrap_or(0);
    }

    pub(crate) fn limit_instance(&self) -> u8 {
        self.limit_instance
    }

    pub(crate) fn blind_head_anim_counter(&self) -> u8 {
        self.blind_head_anim_counter
    }

    pub(crate) fn chr_halfslot_state(&self) -> u8 {
        self.chr_halfslot_state
    }

    pub(crate) fn alert_flag(&self) -> u8 {
        self.alert_flag
    }

    pub(crate) fn graphics_index(&self) -> u8 {
        self.graphics_index
    }

    pub(crate) fn alt_sprite_spawned_flag(&self) -> u8 {
        self.alt_sprite_spawned_flag
    }

    pub(crate) fn cur_object_index(&self) -> u8 {
        self.cur_object_index
    }

    pub(crate) fn alt_sprites_flag(&self) -> u8 {
        self.alt_sprites_flag
    }

    pub(crate) fn ranged_based_toggler(&self) -> u8 {
        self.ranged_based_toggler
    }

    fn set_limit_instance(&mut self, value: u8) {
        self.limit_instance = value;
    }

    fn set_blind_head_anim_counter(&mut self, value: u8) {
        self.blind_head_anim_counter = value;
    }

    fn increment_blind_head_anim_counter(&mut self) {
        self.blind_head_anim_counter = self.blind_head_anim_counter.wrapping_add(1);
    }

    fn increment_limit_instance(&mut self) -> u8 {
        self.limit_instance = self.limit_instance.wrapping_add(1);
        self.limit_instance
    }

    fn decrement_limit_instance(&mut self) -> u8 {
        self.limit_instance = self.limit_instance.wrapping_sub(1);
        self.limit_instance
    }

    fn set_chr_halfslot_state(&mut self, value: u8) {
        self.chr_halfslot_state = value;
    }

    fn set_alert_flag(&mut self, value: u8) {
        self.alert_flag = value;
    }

    fn decrement_alert_flag(&mut self) -> u8 {
        self.alert_flag = self.alert_flag.wrapping_sub(1);
        self.alert_flag
    }

    fn set_graphics_index(&mut self, value: u8) {
        self.graphics_index = value;
    }

    fn save_special_exit_graphics_index(&mut self) {
        self.saved_special_exit_graphics_index = self.graphics_index;
    }

    fn restore_special_exit_graphics_index(&mut self) {
        self.graphics_index = self.saved_special_exit_graphics_index;
    }

    fn restore_exit_graphics_index(&mut self) {
        self.graphics_index = self.saved_exit_graphics_index;
    }

    fn set_alt_sprite_spawned_flag(&mut self, value: u8) {
        self.alt_sprite_spawned_flag = value;
    }

    fn set_cur_object_index(&mut self, value: u8) {
        self.cur_object_index = value;
    }

    fn set_alt_sprites_flag(&mut self, value: u8) {
        self.alt_sprites_flag = value;
    }

    fn clear_alt_sprites_flag(&mut self) {
        self.alt_sprites_flag = 0;
    }

    fn increment_ranged_based_toggler(&mut self) {
        self.ranged_based_toggler = self.ranged_based_toggler.wrapping_add(1);
    }
}

pub(crate) struct NativeSpriteSystemBridgeMut<'a> {
    state: &'a mut SpriteSystemState,
    sprite_slots: &'a mut SpriteSlotsState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteSystemBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteSystemState,
        sprite_slots: &'a mut SpriteSlotsState,
        ram: &'a mut [u8],
    ) -> Self {
        Self {
            state,
            sprite_slots,
            ram,
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        let mut fresh = SpriteSystemState::load_from_ram(self.ram);
        fresh.saved_exit_graphics_index = self.state.saved_exit_graphics_index;
        debug_assert_eq!(*self.state, fresh);
    }

    forward_synced! {
        state;
        fn set_limit_instance(value: u8);
        fn set_blind_head_anim_counter(value: u8);
        fn increment_blind_head_anim_counter();
        fn increment_limit_instance() -> u8;
        fn decrement_limit_instance() -> u8;
        fn set_chr_halfslot_state(value: u8);
        fn set_alert_flag(value: u8);
        fn decrement_alert_flag() -> u8;
        fn set_graphics_index(value: u8);
        fn save_special_exit_graphics_index();
        fn restore_special_exit_graphics_index();
        fn restore_exit_graphics_index();
    }

    pub(crate) fn fill_live_states(&mut self, value: u8) {
        self.ram[SPRITE_STATE..SPRITE_STATE + SPRITE_SLOT_COUNT].fill(value);
        *self.sprite_slots = SpriteSlotsState::load_from_ram(self.ram);
    }

    pub(crate) fn clear_live_table_pages(&mut self) {
        self.ram[SPRITE_Y_LO..SPRITE_Y_LO + 256 * 3].fill(0);
        *self.sprite_slots = SpriteSlotsState::load_from_ram(self.ram);
    }

    forward_synced! {
        state;
        fn set_alt_sprite_spawned_flag(value: u8);
        fn set_cur_object_index(value: u8);
        fn set_alt_sprites_flag(value: u8);
        fn clear_alt_sprites_flag();
        fn increment_ranged_based_toggler();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteWorkspaceState {
    room_origin_x_high: u8,
    room_origin_y_high: u8,
    pickup_slot_cache: u8,
    shared_scratch_a: u8,
    tile_type: u8,
    reset_scratch_a: u8,
    reset_scratch_b: u8,
    graphics_subsets: [u8; SPRITE_GRAPHICS_SUBSET_COUNT],
    draw_priority_override: u16,
    current_sprite_x: u16,
    current_sprite_y: u16,
    low_scratch: [u8; SPRITE_ZERO_PAGE_WORK_COUNT],
    where_in_room: Vec<u8>,
}

impl Default for SpriteWorkspaceState {
    fn default() -> Self {
        Self {
            room_origin_x_high: 0,
            room_origin_y_high: 0,
            pickup_slot_cache: 0,
            shared_scratch_a: 0,
            tile_type: 0,
            reset_scratch_a: 0,
            reset_scratch_b: 0,
            graphics_subsets: [0; SPRITE_GRAPHICS_SUBSET_COUNT],
            draw_priority_override: 0,
            current_sprite_x: 0,
            current_sprite_y: 0,
            low_scratch: [0; SPRITE_ZERO_PAGE_WORK_COUNT],
            where_in_room: vec![0; SPRITE_WHERE_IN_ROOM_BYTES],
        }
    }
}

impl SpriteWorkspaceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        let mut graphics_subsets = [0; SPRITE_GRAPHICS_SUBSET_COUNT];
        for (slot, value) in graphics_subsets.iter_mut().enumerate() {
            *value = ram.get(SPRITE_GFX_SUBSET_0 + slot).copied().unwrap_or(0);
        }

        let mut low_scratch = [0; SPRITE_ZERO_PAGE_WORK_COUNT];
        for (offset, value) in low_scratch.iter_mut().enumerate() {
            *value = ram.get(offset).copied().unwrap_or(0);
        }

        Self {
            room_origin_x_high: ram.get(SPRITE_ROOM_ORIGIN_X_HI).copied().unwrap_or(0),
            room_origin_y_high: ram.get(SPRITE_ROOM_ORIGIN_Y_HI).copied().unwrap_or(0),
            pickup_slot_cache: ram.get(SPRITE_PICKUP_SLOT_CACHE).copied().unwrap_or(0),
            shared_scratch_a: ram.get(SPRITE_SHARED_WORK_A).copied().unwrap_or(0),
            tile_type: ram.get(SPRITE_TILETYPE).copied().unwrap_or(0),
            reset_scratch_a: ram.get(SPRITE_RESET_WORK_A).copied().unwrap_or(0),
            reset_scratch_b: ram.get(SPRITE_RESET_WORK_B).copied().unwrap_or(0),
            graphics_subsets,
            draw_priority_override: read_le_u16(ram, SPRITE_DRAW_PRIORITY_OVERRIDE),
            current_sprite_x: read_le_u16(ram, CUR_SPRITE_X),
            current_sprite_y: read_le_u16(ram, CUR_SPRITE_Y),
            low_scratch,
            // SPRITE_WHERE_IN_ROOM (0x1df80) is the dungeon per-room sprite-kill
            // bitmask; in the OVERWORLD the same WRAM is `sprite_where_in_overworld`
            // (the proximity-spawn presence table, owned by OverworldSpritePresence-
            // State). ALWAYS load the real region so the mirror holds the true bytes
            // (incl. overworld-presence leftover that persists into a dungeon); the
            // *projection* is what's gated on indoors (see write_to_ram). Loading an
            // all-zero buffer outdoors was wrong on the overworld->dungeon transition
            // frame, where player_is_indoors flips 0->1 mid-frame: the empty buffer
            // then got projected and wiped the presence leftover.
            where_in_room: ram
                [SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + SPRITE_WHERE_IN_ROOM_BYTES]
                .to_vec(),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SPRITE_ROOM_ORIGIN_X_HI] = self.room_origin_x_high;
        ram[SPRITE_ROOM_ORIGIN_Y_HI] = self.room_origin_y_high;
        ram[SPRITE_PICKUP_SLOT_CACHE] = self.pickup_slot_cache;
        ram[SPRITE_SHARED_WORK_A] = self.shared_scratch_a;
        ram[SPRITE_TILETYPE] = self.tile_type;
        ram[SPRITE_RESET_WORK_A] = self.reset_scratch_a;
        ram[SPRITE_RESET_WORK_B] = self.reset_scratch_b;
        ram[SPRITE_GFX_SUBSET_0..SPRITE_GFX_SUBSET_0 + SPRITE_GRAPHICS_SUBSET_COUNT]
            .copy_from_slice(&self.graphics_subsets);
        write_le_u16(
            ram,
            SPRITE_DRAW_PRIORITY_OVERRIDE,
            self.draw_priority_override,
        );
        write_le_u16(ram, CUR_SPRITE_X, self.current_sprite_x);
        write_le_u16(ram, CUR_SPRITE_Y, self.current_sprite_y);
        ram[..SPRITE_ZERO_PAGE_WORK_COUNT].copy_from_slice(&self.low_scratch);
        // Only project the dungeon per-room kill bitmask while indoors; in the
        // overworld this WRAM is the proximity-spawn presence table (see
        // load_from_ram), and projecting stale all-zero room data would wipe the
        // markers OverworldSpritePresenceState just filled.
        if ram[PLAYER_IS_INDOORS] != 0 {
            ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + self.where_in_room.len()]
                .copy_from_slice(&self.where_in_room);
        }
    }

    pub(crate) fn room_origin_x_high(&self) -> u8 {
        self.room_origin_x_high
    }

    pub(crate) fn room_origin_y_high(&self) -> u8 {
        self.room_origin_y_high
    }

    pub(crate) fn pickup_slot_cache(&self) -> u8 {
        self.pickup_slot_cache
    }

    pub(crate) fn shared_scratch_a(&self) -> u8 {
        self.shared_scratch_a
    }

    pub(crate) fn tile_type(&self) -> u8 {
        self.tile_type
    }

    pub(crate) fn prep_shared_counter(&self) -> u8 {
        self.reset_scratch_a
    }

    pub(crate) fn armos_knight_remaining_count(&self) -> u8 {
        self.reset_scratch_a
    }

    pub(crate) fn graphics_subset(&self, slot: usize) -> u8 {
        self.graphics_subsets.get(slot).copied().unwrap_or(0)
    }

    pub(crate) fn draw_priority_override(&self) -> u16 {
        self.draw_priority_override
    }

    pub(crate) fn current_sprite_x(&self) -> u16 {
        self.current_sprite_x
    }

    pub(crate) fn current_sprite_x_low(&self) -> u8 {
        self.current_sprite_x as u8
    }

    pub(crate) fn current_sprite_y(&self) -> u16 {
        self.current_sprite_y
    }

    pub(crate) fn current_sprite_y_low(&self) -> u8 {
        self.current_sprite_y as u8
    }

    pub(crate) fn oam_prep_x(&self) -> u16 {
        u16::from(self.low_scratch[SPRITE_OAM_PREP_X])
            | (u16::from(self.low_scratch[SPRITE_OAM_PREP_X + 1]) << 8)
    }

    pub(crate) fn oam_prep_y(&self) -> u16 {
        u16::from(self.low_scratch[SPRITE_OAM_PREP_Y])
            | (u16::from(self.low_scratch[SPRITE_OAM_PREP_Y + 1]) << 8)
    }

    pub(crate) fn where_in_room(&self, room: usize) -> u16 {
        let offset = room * 2;
        let low = self.where_in_room.get(offset).copied().unwrap_or(0);
        let high = self.where_in_room.get(offset + 1).copied().unwrap_or(0);
        u16::from(low) | (u16::from(high) << 8)
    }

    fn set_room_origin_x_high(&mut self, value: u8) {
        self.room_origin_x_high = value;
    }

    fn set_room_origin_y_high(&mut self, value: u8) {
        self.room_origin_y_high = value;
    }

    fn set_pickup_slot_cache(&mut self, value: u8) {
        self.pickup_slot_cache = value;
    }

    fn set_shared_scratch_a(&mut self, value: u8) {
        self.shared_scratch_a = value;
    }

    fn set_tile_type(&mut self, value: u8) {
        self.tile_type = value;
    }

    fn set_prep_shared_counter(&mut self, value: u8) {
        self.reset_scratch_a = value;
    }

    fn increment_prep_shared_counter(&mut self) -> u8 {
        self.reset_scratch_a = self.reset_scratch_a.wrapping_add(1);
        self.reset_scratch_a
    }

    fn decrement_prep_shared_counter(&mut self) -> u8 {
        self.reset_scratch_a = self.reset_scratch_a.wrapping_sub(1);
        self.reset_scratch_a
    }

    fn set_reset_scratch_a(&mut self, value: u8) {
        self.reset_scratch_a = value;
    }

    fn set_reset_scratch_b(&mut self, value: u8) {
        self.reset_scratch_b = value;
    }

    fn set_graphics_subset(&mut self, slot: usize, value: u8) {
        if let Some(subset) = self.graphics_subsets.get_mut(slot) {
            *subset = value;
        }
    }

    fn clear_where_in_room(&mut self) {
        self.where_in_room.fill(0);
    }

    fn clear_draw_priority_override(&mut self) {
        self.draw_priority_override = 0;
    }

    fn set_draw_priority_override_low(&mut self, value: u8) {
        self.draw_priority_override = (self.draw_priority_override & 0xff00) | u16::from(value);
    }

    fn set_current_sprite_x(&mut self, value: u16) {
        self.current_sprite_x = value;
    }

    fn set_current_sprite_x_low(&mut self, value: u8) {
        self.current_sprite_x = (self.current_sprite_x & 0xff00) | u16::from(value);
    }

    fn add_current_sprite_x_low(&mut self, value: u8) {
        self.set_current_sprite_x_low(self.current_sprite_x_low().wrapping_add(value));
    }

    fn set_current_sprite_y(&mut self, value: u16) {
        self.current_sprite_y = value;
    }

    fn set_current_sprite_y_low(&mut self, value: u8) {
        self.current_sprite_y = (self.current_sprite_y & 0xff00) | u16::from(value);
    }

    fn add_current_sprite_y_low(&mut self, value: u8) {
        self.set_current_sprite_y_low(self.current_sprite_y_low().wrapping_add(value));
    }

    fn subtract_current_sprite_y_low(&mut self, value: u8) {
        self.set_current_sprite_y_low(self.current_sprite_y_low().wrapping_sub(value));
    }

    fn set_oam_prep_coords(&mut self, x: u16, y: u16) {
        self.low_scratch[SPRITE_OAM_PREP_X] = x as u8;
        self.low_scratch[SPRITE_OAM_PREP_X + 1] = (x >> 8) as u8;
        self.low_scratch[SPRITE_OAM_PREP_Y] = y as u8;
        self.low_scratch[SPRITE_OAM_PREP_Y + 1] = (y >> 8) as u8;
    }

    fn set_killed_sprite_load_block(&mut self, block: u16) {
        self.low_scratch[SPRITE_LOAD_BLOCK_STATE] = block as u8;
        let pointer = (block >> 3).wrapping_add(0xef80);
        self.low_scratch[SPRITE_LOAD_BLOCK_STATE + 1] = pointer as u8;
        self.low_scratch[SPRITE_LOAD_BLOCK_STATE + 2] = (pointer >> 8) as u8;
    }

    fn set_last_garnish_index(&mut self, index: i32) {
        self.low_scratch[SPRITE_LAST_GARNISH_INDEX] = index as u8;
    }

    fn set_where_in_room(&mut self, room: usize, value: u16) {
        let offset = room * 2;
        if offset + 1 < self.where_in_room.len() {
            self.where_in_room[offset] = value as u8;
            self.where_in_room[offset + 1] = (value >> 8) as u8;
        }
    }
}

pub(crate) struct NativeSpriteWorkspaceBridgeMut<'a> {
    state: &'a mut SpriteWorkspaceState,
    overworld_sprite_presence: &'a mut OverworldSpritePresenceState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteWorkspaceBridgeMut<'a> {
    pub(crate) fn new(
        state: &'a mut SpriteWorkspaceState,
        overworld_sprite_presence: &'a mut OverworldSpritePresenceState,
        ram: &'a mut [u8],
    ) -> Self {
        Self {
            state,
            overworld_sprite_presence,
            ram,
        }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        let mut fresh = SpriteWorkspaceState::load_from_ram(self.ram);
        if self.ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0) == 0 {
            fresh.where_in_room.clone_from(&self.state.where_in_room);
        }
        debug_assert_eq!(*self.state, fresh);
    }

    forward_synced! {
        state;
        fn set_room_origin_x_high(value: u8);
        fn set_room_origin_y_high(value: u8);
        fn set_pickup_slot_cache(value: u8);
        fn set_shared_scratch_a(value: u8);
    }

    pub(crate) fn set_room_marker_word(&mut self, slot: usize, value: u16) {
        write_le_u16(
            self.ram,
            crate::game_state::constants::SPRITE_ROOM_MARKER_WORD + slot * 2,
            value,
        );
    }

    forward_synced! {
        state;
        fn set_tile_type(value: u8);
        fn set_prep_shared_counter(value: u8);
        fn increment_prep_shared_counter() -> u8;
        fn decrement_prep_shared_counter() -> u8;
    }

    pub(crate) fn decrement_armos_knight_remaining_count(&mut self) -> u8 {
        let value = self.state.decrement_prep_shared_counter();
        self.sync();
        value
    }

    pub(crate) fn clear_vitreous_eyeball_release_count(&mut self) {
        self.state.set_prep_shared_counter(0);
        self.sync();
    }

    forward_synced! { state; fn set_reset_scratch_a(value: u8); }

    pub(crate) fn clear_agahnim_phase_scratch(&mut self) {
        self.state.set_reset_scratch_a(0);
        self.sync();
    }

    forward_synced! {
        state;
        fn set_reset_scratch_b(value: u8);
        fn set_graphics_subset(slot: usize, value: u8);
    }

    pub(crate) fn clear_where_in_room(&mut self) {
        self.state.clear_where_in_room();
        // The legacy sprite reset path (sprite_reset_all_no_disable) memsets the full
        // 0x1000 region in RAM unconditionally. Our mirror's projection is gated on
        // indoors (see write_to_ram), so an OUTDOORS reset would otherwise leave RAM
        // untouched and the overworld presence table (sprite_where_in_overworld, the
        // same WRAM) would keep stale markers across an area reload. Clear RAM directly
        // so both modes match the oracle, and clear the presence mirror that models the
        // same region (its projection is outdoors-gated) so it cannot re-project the
        // stale markers.
        self.ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + SPRITE_WHERE_IN_ROOM_BYTES].fill(0);
        self.overworld_sprite_presence.clear_all();
        self.sync();
    }

    /// Publish a byte range of the source's descending reset loop, keeping
    /// both native interpretations of the shared WRAM region coherent.
    pub(crate) fn clear_where_in_room_range(&mut self, range: std::ops::Range<usize>) {
        assert!(range.start <= range.end && range.end <= SPRITE_WHERE_IN_ROOM_BYTES);
        self.state.where_in_room[range.clone()].fill(0);
        self.overworld_sprite_presence.markers[range.clone()].fill(0);
        self.ram[SPRITE_WHERE_IN_ROOM + range.start..SPRITE_WHERE_IN_ROOM + range.end].fill(0);
        self.sync();
    }

    forward_synced! {
        state;
        fn clear_draw_priority_override();
        fn set_draw_priority_override_low(value: u8);
        fn set_current_sprite_x(value: u16);
        fn set_current_sprite_x_low(value: u8);
        fn add_current_sprite_x_low(value: u8);
        fn set_current_sprite_y(value: u16);
        fn set_current_sprite_y_low(value: u8);
        fn add_current_sprite_y_low(value: u8);
        fn subtract_current_sprite_y_low(value: u8);
        fn set_oam_prep_coords(x: u16, y: u16);
        fn set_killed_sprite_load_block(block: u16);
        fn set_last_garnish_index(index: i32);
        fn set_where_in_room(room: usize, value: u16);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FailedSpinSparkleSpawnState {
    item_to_link: u8,
    step: u8,
    timer: u8,
    aux_timer: u8,
    x: u16,
    y: u16,
}

impl FailedSpinSparkleSpawnState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            item_to_link: ram.get(ANCILLA_ITEM_TO_LINK - 1).copied().unwrap_or(0),
            step: ram.get(ANCILLA_STEP - 1).copied().unwrap_or(0),
            timer: ram.get(ANCILLA_TIMER - 1).copied().unwrap_or(0),
            aux_timer: ram.get(ANCILLA_AUX_TIMER - 1).copied().unwrap_or(0),
            x: u16::from(ram.get(ANCILLA_X_LO - 1).copied().unwrap_or(0))
                | (u16::from(ram.get(ANCILLA_X_HI - 1).copied().unwrap_or(0)) << 8),
            y: u16::from(ram.get(ANCILLA_Y_LO - 1).copied().unwrap_or(0))
                | (u16::from(ram.get(ANCILLA_Y_HI - 1).copied().unwrap_or(0)) << 8),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[ANCILLA_ITEM_TO_LINK - 1] = self.item_to_link;
        ram[ANCILLA_STEP - 1] = self.step;
        ram[ANCILLA_TIMER - 1] = self.timer;
        ram[ANCILLA_AUX_TIMER - 1] = self.aux_timer;
        ram[ANCILLA_X_LO - 1] = self.x as u8;
        ram[ANCILLA_X_HI - 1] = (self.x >> 8) as u8;
        ram[ANCILLA_Y_LO - 1] = self.y as u8;
        ram[ANCILLA_Y_HI - 1] = (self.y >> 8) as u8;
    }

    pub(crate) fn write_failed_spin_sparkle(&mut self, step: u8, x: u16, y: u16) {
        self.item_to_link = 0;
        self.step = step;
        self.timer = 4;
        self.aux_timer = 3;
        self.x = x;
        self.y = y;
    }

    pub(crate) fn step(&self) -> u8 {
        self.step
    }

    pub(crate) fn timer(&self) -> u8 {
        self.timer
    }

    pub(crate) fn aux_timer(&self) -> u8 {
        self.aux_timer
    }

    pub(crate) fn x(&self) -> u16 {
        self.x
    }

    pub(crate) fn y(&self) -> u16 {
        self.y
    }
}

pub(crate) struct NativeFailedSpinSparkleSpawnBridgeMut<'a> {
    state: &'a mut FailedSpinSparkleSpawnState,
    ram: &'a mut [u8],
}

impl<'a> NativeFailedSpinSparkleSpawnBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut FailedSpinSparkleSpawnState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(
            *self.state,
            FailedSpinSparkleSpawnState::load_from_ram(self.ram)
        );
    }

    pub(crate) fn write_failed_spin_sparkle(&mut self, step: u8, x: u16, y: u16) {
        self.state.write_failed_spin_sparkle(step, x, y);
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldSpritePresenceState {
    markers: Vec<u8>,
}

impl Default for OverworldSpritePresenceState {
    fn default() -> Self {
        Self {
            markers: vec![0; OVERWORLD_SPRITE_PRESENCE_COUNT],
        }
    }
}

impl OverworldSpritePresenceState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            markers: ram[OVERWORLD_SPRITE_PRESENCE
                ..OVERWORLD_SPRITE_PRESENCE + OVERWORLD_SPRITE_PRESENCE_COUNT]
                .to_vec(),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        // sprite_where_in_overworld (0x1df80) shares WRAM with the dungeon
        // sprite_where_in_room bitmask (SpriteWorkspaceState.where_in_room). It is an
        // OVERWORLD-only table, so only project it outdoors; indoors where_in_room owns
        // the region (projecting these markers there would clobber the dungeon kill
        // bitmask).
        if ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0) != 0 {
            return;
        }
        ram[OVERWORLD_SPRITE_PRESENCE..OVERWORLD_SPRITE_PRESENCE + OVERWORLD_SPRITE_PRESENCE_COUNT]
            .fill(0);
        let len = self.markers.len().min(OVERWORLD_SPRITE_PRESENCE_COUNT);
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

    pub(crate) fn clear_all(&mut self) {
        self.markers.fill(0);
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

    forward_synced! { state; fn set_marker(index: usize, value: u8); }
}

pub(crate) struct NativeOverworldSpriteLoadedBridgeMut<'a> {
    state: &'a mut OverworldSpriteLoadedState,
    ram: &'a mut [u8],
}

impl<'a> NativeOverworldSpriteLoadedBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut OverworldSpriteLoadedState, ram: &'a mut [u8]) -> Self {
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

    forward_synced! {
        state;
        fn clear_loaded_mask(block: u16, loaded_mask: u8);
        fn set_loaded_mask(block: u16, loaded_mask: u8);
        fn clear_all();
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
        // The $7F58xx ancilla scratch is C-aliased across mutually-exclusive effects and is
        // no longer bulk-projected (see EffectsState::write_to_ram). Re-read from RAM before
        // mutating so a setter composes with whichever effect wrote the window last.
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

    forward_synced! {
        orbit;
        fn set_angle(slot: usize, value: u8);
        fn advance_angle(slot: usize) -> u8;
        fn set_radius(value: u8);
        fn tick_spin_countdown() -> u8;
        fn set_spin_countdown(value: u8);
        fn set_beam_top_bucket(value: u8);
        fn initialize_beam_adjusted_y(value: u16);
        fn set_orb_position(x: u16, y: u16);
        fn set_orbit_position(x: u16, y: u16);
    }

    pub(crate) fn set_swordbeam_temp(&mut self, x: u16, y: u16) {
        // SWORDBEAM_TEMP aliases ETHER_ORBIT_X/Y, but the rest of EtherOrbitState — the
        // angles at ETHER_ANGLE (0x15800) and radius at ETHER_RADIUS (0x15808) — is
        // mode-reused with the sword-beam EFFECT_ANGLE_WORK (0x15800-0x15808) the caller
        // (add_sword_beam) just wrote via the effect-angle-scratch bridge. A plain sync
        // would re-stamp this state's stale frame-start ether angles/radius over that fresh
        // sword-beam write. Reload from RAM first so the sync writes those overlapping bytes
        // back unchanged and only the orbit x/y move (f372737; matches the old clone's raw
        // SWORDBEAM_TEMP_X/Y store).
        *self.orbit = EtherOrbitState::load_from_ram(self.ram);
        self.orbit.set_swordbeam_temp(x, y);
        self.sync();
    }

    forward_synced! { orbit; fn set_beam_y(value: u16); }
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

    forward_synced! {
        history;
        fn set_x(position: usize, value: u16);
        fn set_y(position: usize, value: u16);
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

pub(crate) struct NativeEnemyDamageSubclassTableBridgeMut<'a> {
    state: &'a mut EnemyDamageSubclassTableState,
    ram: &'a mut [u8],
}

impl<'a> NativeEnemyDamageSubclassTableBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut EnemyDamageSubclassTableState, ram: &'a mut [u8]) -> Self {
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

    pub(crate) fn set_low_position(&mut self, x: u8, y: u8) {
        self.draw_position_x = x;
        self.draw_position_y = y;
    }

    pub(crate) fn set_low_position_word(&mut self, value: u16) {
        self.set_low_position(value as u8, (value >> 8) as u8);
    }

    pub(crate) fn offset_low_position(&mut self, dx: u8, dy: u8) -> (u8, u8) {
        self.draw_position_x = self.draw_position_x.wrapping_add(dx);
        self.draw_position_y = self.draw_position_y.wrapping_add(dy);
        (self.draw_position_x, self.draw_position_y)
    }

    pub(crate) fn set_flags_high(&mut self, value: u8) {
        self.draw_flags_or_hitbox_x_offset = value;
    }

    pub(crate) fn set_x_high_offset(&mut self, value: u8) {
        self.draw_flags_or_hitbox_x_offset = value;
    }

    pub(crate) fn set_offsets(&mut self, y_low: u8, x_high: u8) {
        self.hitbox_y_offset = y_low;
        self.draw_flags_or_hitbox_x_offset = x_high;
    }
}

pub(crate) struct NativeSpriteDrawWorkPositionBridgeMut<'a> {
    state: &'a mut SpriteDrawHitboxWorkState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteDrawWorkPositionBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteDrawHitboxWorkState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    forward_synced! {
        state;
        fn set_low_position(x: u8, y: u8);
        fn set_low_position_word(value: u16);
    }

    pub(crate) fn set_word_bytes(&mut self, low: u8, high: u8) {
        self.set_low_position(low, high);
    }

    forward_synced! {
        state;
        fn offset_low_position(dx: u8, dy: u8) -> (u8, u8);
        fn set_flags_high(value: u8);
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

pub(crate) struct NativeSpriteHitboxWorkOffsetBridgeMut<'a> {
    state: &'a mut SpriteDrawHitboxWorkState,
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteHitboxWorkOffsetBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteDrawHitboxWorkState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    forward_synced! {
        state;
        fn set_x_high_offset(value: u8);
        fn set_offsets(y_low: u8, x_high: u8);
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

    pub(crate) fn set_tile_type(&mut self, slot: usize, value: u8) -> bool {
        let Some(tile_type) = self.tile_types.get_mut(slot) else {
            return false;
        };
        *tile_type = value;
        true
    }
}

pub(crate) struct NativeDualLayerTileCacheBridgeMut<'a> {
    state: &'a mut DualLayerTileCacheState,
    ram: &'a mut [u8],
}

impl<'a> NativeDualLayerTileCacheBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut DualLayerTileCacheState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    pub(crate) fn set_tile_type(&mut self, slot: usize, value: u8) {
        if self.state.set_tile_type(slot, value) {
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

    pub(crate) fn take_next_index(&mut self, slot: usize) -> u8 {
        let Some(index) = self.next_indices.get_mut(slot) else {
            return 0;
        };
        let current = *index;
        *index = current.wrapping_add(1) & 7;
        current
    }
}

pub(crate) struct NativePrizeDropCycleBridgeMut<'a> {
    state: &'a mut PrizeDropCycleState,
    ram: &'a mut [u8],
}

impl<'a> NativePrizeDropCycleBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut PrizeDropCycleState, ram: &'a mut [u8]) -> Self {
        Self { state, ram }
    }

    forward_synced! { state; fn take_next_index(slot: usize) -> u8; }

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

    pub(crate) fn snapshot_high(&self) -> u16 {
        self.snapshot_high
    }

    pub(crate) fn clear_elapsed(&mut self) {
        self.elapsed_low = 0;
        self.elapsed_high = 0;
    }

    pub(crate) fn increment_elapsed_low(&mut self) -> u16 {
        self.elapsed_low = self.elapsed_low.wrapping_add(1);
        self.elapsed_low
    }

    pub(crate) fn increment_elapsed_high(&mut self) -> u16 {
        self.elapsed_high = self.elapsed_high.wrapping_add(1);
        self.elapsed_high
    }

    pub(crate) fn capture_snapshot(&mut self) {
        self.snapshot_low = self.elapsed_low;
        self.snapshot_high = self.elapsed_high;
    }
}

pub(crate) struct NativeMazeGameTimerBridgeMut<'a> {
    state: &'a mut MazeGameTimerState,
    ram: &'a mut [u8],
}

impl<'a> NativeMazeGameTimerBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut MazeGameTimerState, ram: &'a mut [u8]) -> Self {
        // Shared 0x1fe00 window (see SpriteState::write_to_ram): re-read before mutating
        // so a setter composes with whichever system wrote it last.
        *state = MazeGameTimerState::load_from_ram(ram);
        Self { state, ram }
    }

    forward_synced! {
        state;
        fn clear_elapsed();
        fn increment_elapsed_low() -> u16;
        fn increment_elapsed_high() -> u16;
        fn capture_snapshot();
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, MazeGameTimerState::load_from_ram(self.ram));
    }
}
