use crate::game_state::constants::{
    ACTIVE_OVERLORD_INDEX, ALT_SPRITES_FLAG, ALT_SPRITE_GRAPHICS, ALT_SPRITE_SPAWNED_FLAG,
    ALT_SPRITE_STATE, ALT_SPRITE_TYPE, ALT_SPRITE_X_HI, ALT_SPRITE_X_LO, ALT_SPRITE_Y_HI,
    ALT_SPRITE_Y_LO, ANCILLA_ALLOC_ROTATE, ANCILLA_AUX_TIMER, ANCILLA_ITEM_TO_LINK, ANCILLA_STEP,
    ANCILLA_TIMER, ANCILLA_X_HI, ANCILLA_X_LO, ANCILLA_Y_HI, ANCILLA_Y_LO, AUX_TILE_THEME_INDEX,
    BLIND_HEAD_ANIM_COUNTER, CACHED_SPRITE_ALT_FIELDS, CACHED_SPRITE_LIVE_FIELDS,
    CHAIN_CHOMP_HISTORY_X, CHAIN_CHOMP_HISTORY_Y, CUR_OBJECT_INDEX, CUR_SPRITE_X, CUR_SPRITE_Y,
    DRAW_WORK_FLAGS_HI, DRAW_WORK_POSITION_X, DRAW_WORK_POSITION_Y, DUAL_LAYER_TILE_CACHE,
    ENEMY_DAMAGE_DATA, ETHER_ANGLE, ETHER_BEAM_TOP_BUCKET, ETHER_BEAM_Y, ETHER_ORBIT_X,
    ETHER_ORBIT_Y, ETHER_ORB_X, ETHER_ORB_Y, ETHER_RADIUS, ETHER_SPIN_COUNTDOWN, FOLLOWER_DROPPED,
    FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX, FOLLOWER_INDICATOR, FOLLOWER_JUMP_TIMER,
    FOLLOWER_KIKI_ANIM_COUNTER, FOLLOWER_PALETTE_SWAP_FLAG, FOLLOWER_SAVED_FLOOR,
    FOLLOWER_SAVED_INDOORS, FOLLOWER_SAVED_X, FOLLOWER_SAVED_Y, FOLLOWER_TAIL_WRITE_INDEX,
    GARNISH_ACTIVE, HAUNTED_GROVE_FLUTE_EVENT_LATCH, HITBOX_WORK_X_OFFSET, HITBOX_WORK_Y_OFFSET,
    MAIN_TILE_THEME_INDEX, MAZE_GAME_TIMER_HI, MAZE_GAME_TIMER_LO, MAZE_GAME_TIMER_SNAPSHOT_HI,
    MAZE_GAME_TIMER_SNAPSHOT_LO, MISC_SPRITES_GRAPHICS_INDEX, OVERWORLD_BOULDER_TRAP_COUNT,
    OVERWORLD_BOULDER_TRAP_TIMER, OVERWORLD_SPRITE_PRESENCE, OVERWORLD_SPRITE_WAS_LOADED,
    PRIZE_DROP_CYCLE, REPULSESPARK_ANIM_DELAY, REPULSESPARK_FLOOR_STATUS, REPULSESPARK_TIMER,
    REPULSESPARK_X_LO, REPULSESPARK_Y_LO, SPRCOLL_X_BASE, SPRCOLL_X_SIZE, SPRCOLL_Y_BASE,
    SPRCOLL_Y_SIZE, SPRITE_ALERT_FLAG, SPRITE_CHR_HALFSLOT_STATE, SPRITE_DRAW_PRIORITY_OVERRIDE,
    SPRITE_GFX_SUBSET_0, SPRITE_GRAPHICS_INDEX, SPRITE_GRAPHICS_INDEX_EXIT,
    SPRITE_GRAPHICS_INDEX_SPEXIT, SPRITE_LAST_GARNISH_INDEX, SPRITE_LIMIT_INSTANCE,
    SPRITE_LOAD_BLOCK_STATE, SPRITE_OAM_PREP_X, SPRITE_OAM_PREP_Y, SPRITE_PICKUP_SLOT_CACHE,
    SPRITE_RESET_WORK_A, SPRITE_RESET_WORK_B, SPRITE_ROOM_ORIGIN_X_HI, SPRITE_ROOM_ORIGIN_Y_HI,
    SPRITE_SHARED_WORK_A, SPRITE_STATE, SPRITE_TILETYPE, SPRITE_WHERE_IN_ROOM, SPRITE_Y_LO,
    SPR_RANGED_BASED_TOGGLER, TAGALONG_ANIM_FRAME_COUNTER, TAGALONG_APPEARANCE_NONE_FLAG,
    TAGALONG_DATA_INDEX, TAGALONG_EVENT_FLAGS, TAGALONG_HOOKSHOT_INTERLOCK, TAGALONG_LAYERBITS,
    TAGALONG_SHARED_STATE_A, TAGALONG_X_HI, TAGALONG_X_LO, TAGALONG_Y_HI, TAGALONG_Y_LO,
    TAGALONG_Z, TIMER_TAGALONG_REACQUIRE, ZELDA_RESCUE_CUTSCENE_STATE,
};
use crate::types::{read_le_u16, write_le_u16};

const SPRITE_SLOT_COUNT: usize = 16;
const TAGALONG_SLOT_COUNT: usize = 20;
const CHAIN_CHOMP_HISTORY_LEN: usize = 0x80;
const ETHER_ANGLE_COUNT: usize = 8;
const ENEMY_DAMAGE_SUBCLASS_COUNT: usize = 0x1000;
const OVERWORLD_SPRITE_FLAG_COUNT: usize = 0x200;
const SPRITE_GRAPHICS_SUBSET_COUNT: usize = 4;
const SPRITE_ZERO_PAGE_WORK_COUNT: usize = 16;
const SPRITE_WHERE_IN_ROOM_BYTES: usize = 0x1000;
const CACHED_SPRITE_SLOT_COUNT: usize = 0x1b;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteState {
    pub(crate) system: SpriteSystemState,
    pub(crate) workspace: SpriteWorkspaceState,
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
}

impl SpriteState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            system: SpriteSystemState::load_from_ram(ram),
            workspace: SpriteWorkspaceState::load_from_ram(ram),
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
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.system.write_to_ram(ram);
        self.workspace.write_to_ram(ram);
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
        self.failed_spin_sparkle_spawn.write_to_ram(ram);
        self.garnish_runtime.write_to_ram(ram);
        self.follower_runtime.write_to_ram(ram);
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
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> NativeCachedSpriteBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut CachedSpritesState, ram: &'a mut [u8], slot: usize) -> Self {
        *state = CachedSpritesState::load_from_ram(ram);
        Self { state, ram, slot }
    }

    fn reload(&mut self) {
        *self.state = CachedSpritesState::load_from_ram(self.ram);
    }

    pub(crate) fn clear_state(&mut self) {
        self.ram[ALT_SPRITE_STATE + self.slot] = 0;
        self.reload();
    }

    pub(crate) fn initialize_trinexx_component(&mut self) {
        self.ram[ALT_SPRITE_TYPE + self.slot] = 0x40;
        self.ram[ALT_SPRITE_X_HI + self.slot] = 0;
        self.ram[ALT_SPRITE_Y_HI + self.slot] = 0;
        self.reload();
    }

    pub(crate) fn set_type_byte(&mut self, value: u8) {
        self.ram[ALT_SPRITE_TYPE + self.slot] = value;
        self.reload();
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[ALT_SPRITE_Y_HI + self.slot] = value;
        self.reload();
    }

    pub(crate) fn cache_sprite_header(
        &mut self,
        sprite_type: u8,
        x_low: u8,
        x_high: u8,
        y_low: u8,
        y_high: u8,
        graphics: u8,
    ) {
        self.ram[ALT_SPRITE_STATE + self.slot] = 0;
        self.ram[ALT_SPRITE_TYPE + self.slot] = sprite_type;
        self.ram[ALT_SPRITE_X_LO + self.slot] = x_low;
        self.ram[ALT_SPRITE_X_HI + self.slot] = x_high;
        self.ram[ALT_SPRITE_Y_LO + self.slot] = y_low;
        self.ram[ALT_SPRITE_Y_HI + self.slot] = y_high;
        self.ram[ALT_SPRITE_GRAPHICS + self.slot] = graphics;
        self.reload();
    }

    pub(crate) fn cache_live_fields(&mut self) {
        for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
            self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
        }
        self.reload();
    }

    pub(crate) fn load_cached_into_live(&mut self, backup: &mut [u8; 24]) {
        for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
            backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
        }
        self.reload();
    }

    pub(crate) fn restore_live_from_backup(&mut self, backup: &[u8; 24]) {
        for i in (0..CACHED_SPRITE_LIVE_FIELDS.len()).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
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
    ancilla_alloc_rotate: u8,
    alt_sprites_flag: u8,
    ranged_based_toggler: u8,
    main_tile_theme: u8,
    aux_tile_theme: u8,
    misc_sprites_graphics_index: u8,
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
            ancilla_alloc_rotate: ram.get(ANCILLA_ALLOC_ROTATE).copied().unwrap_or(0),
            alt_sprites_flag: ram.get(ALT_SPRITES_FLAG).copied().unwrap_or(0),
            ranged_based_toggler: ram.get(SPR_RANGED_BASED_TOGGLER).copied().unwrap_or(0),
            main_tile_theme: ram.get(MAIN_TILE_THEME_INDEX).copied().unwrap_or(0),
            aux_tile_theme: ram.get(AUX_TILE_THEME_INDEX).copied().unwrap_or(0),
            misc_sprites_graphics_index: ram.get(MISC_SPRITES_GRAPHICS_INDEX).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[SPRITE_LIMIT_INSTANCE] = self.limit_instance;
        ram[BLIND_HEAD_ANIM_COUNTER] = self.blind_head_anim_counter;
        ram[SPRITE_CHR_HALFSLOT_STATE] = self.chr_halfslot_state;
        ram[SPRITE_ALERT_FLAG] = self.alert_flag;
        ram[SPRITE_GRAPHICS_INDEX] = self.graphics_index;
        ram[SPRITE_GRAPHICS_INDEX_SPEXIT] = self.saved_special_exit_graphics_index;
        ram[SPRITE_GRAPHICS_INDEX_EXIT] = self.saved_exit_graphics_index;
        ram[ALT_SPRITE_SPAWNED_FLAG] = self.alt_sprite_spawned_flag;
        ram[CUR_OBJECT_INDEX] = self.cur_object_index;
        ram[ANCILLA_ALLOC_ROTATE] = self.ancilla_alloc_rotate;
        ram[ALT_SPRITES_FLAG] = self.alt_sprites_flag;
        ram[SPR_RANGED_BASED_TOGGLER] = self.ranged_based_toggler;
        ram[MAIN_TILE_THEME_INDEX] = self.main_tile_theme;
        ram[AUX_TILE_THEME_INDEX] = self.aux_tile_theme;
        ram[MISC_SPRITES_GRAPHICS_INDEX] = self.misc_sprites_graphics_index;
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

    pub(crate) fn saved_special_exit_graphics_index(&self) -> u8 {
        self.saved_special_exit_graphics_index
    }

    pub(crate) fn saved_exit_graphics_index(&self) -> u8 {
        self.saved_exit_graphics_index
    }

    pub(crate) fn alt_sprite_spawned_flag(&self) -> u8 {
        self.alt_sprite_spawned_flag
    }

    pub(crate) fn cur_object_index(&self) -> u8 {
        self.cur_object_index
    }

    pub(crate) fn ancilla_alloc_rotate(&self) -> u8 {
        self.ancilla_alloc_rotate
    }

    pub(crate) fn alt_sprites_flag(&self) -> u8 {
        self.alt_sprites_flag
    }

    pub(crate) fn ranged_based_toggler(&self) -> u8 {
        self.ranged_based_toggler
    }

    pub(crate) fn main_tile_theme(&self) -> u8 {
        self.main_tile_theme
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

    fn set_main_tile_theme(&mut self, value: u8) {
        self.main_tile_theme = value;
    }

    fn set_aux_tile_theme(&mut self, value: u8) {
        self.aux_tile_theme = value;
    }

    fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.misc_sprites_graphics_index = value;
    }

    fn set_cur_object_index(&mut self, value: u8) {
        self.cur_object_index = value;
    }

    fn set_ancilla_alloc_rotate(&mut self, value: u8) {
        self.ancilla_alloc_rotate = value;
    }

    fn decrement_ancilla_alloc_rotate(&mut self) -> u8 {
        self.ancilla_alloc_rotate = self.ancilla_alloc_rotate.wrapping_sub(1);
        self.ancilla_alloc_rotate
    }

    fn clear_ancilla_alloc_rotate(&mut self) {
        self.ancilla_alloc_rotate = 0;
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
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteSystemBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteSystemState, ram: &'a mut [u8]) -> Self {
        *state = SpriteSystemState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SpriteSystemState::load_from_ram(self.ram));
    }

    pub(crate) fn set_limit_instance(&mut self, value: u8) {
        self.state.set_limit_instance(value);
        self.sync();
    }

    pub(crate) fn set_blind_head_anim_counter(&mut self, value: u8) {
        self.state.set_blind_head_anim_counter(value);
        self.sync();
    }

    pub(crate) fn increment_blind_head_anim_counter(&mut self) {
        self.state.increment_blind_head_anim_counter();
        self.sync();
    }

    pub(crate) fn increment_limit_instance(&mut self) -> u8 {
        let value = self.state.increment_limit_instance();
        self.sync();
        value
    }

    pub(crate) fn decrement_limit_instance(&mut self) -> u8 {
        let value = self.state.decrement_limit_instance();
        self.sync();
        value
    }

    pub(crate) fn set_chr_halfslot_state(&mut self, value: u8) {
        self.state.set_chr_halfslot_state(value);
        self.sync();
    }

    pub(crate) fn set_alert_flag(&mut self, value: u8) {
        self.state.set_alert_flag(value);
        self.sync();
    }

    pub(crate) fn decrement_alert_flag(&mut self) -> u8 {
        let value = self.state.decrement_alert_flag();
        self.sync();
        value
    }

    pub(crate) fn set_graphics_index(&mut self, value: u8) {
        self.state.set_graphics_index(value);
        self.sync();
    }

    pub(crate) fn save_special_exit_graphics_index(&mut self) {
        self.state.save_special_exit_graphics_index();
        self.sync();
    }

    pub(crate) fn restore_special_exit_graphics_index(&mut self) {
        self.state.restore_special_exit_graphics_index();
        self.sync();
    }

    pub(crate) fn restore_exit_graphics_index(&mut self) {
        self.state.restore_exit_graphics_index();
        self.sync();
    }

    pub(crate) fn fill_live_states(&mut self, value: u8) {
        self.ram[SPRITE_STATE..SPRITE_STATE + SPRITE_SLOT_COUNT].fill(value);
    }

    pub(crate) fn clear_live_table_pages(&mut self) {
        self.ram[SPRITE_Y_LO..SPRITE_Y_LO + 256 * 3].fill(0);
    }

    pub(crate) fn set_alt_sprite_spawned_flag(&mut self, value: u8) {
        self.state.set_alt_sprite_spawned_flag(value);
        self.sync();
    }

    pub(crate) fn set_main_tile_theme(&mut self, value: u8) {
        self.state.set_main_tile_theme(value);
        self.sync();
    }

    pub(crate) fn set_aux_tile_theme(&mut self, value: u8) {
        self.state.set_aux_tile_theme(value);
        self.sync();
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.state.set_misc_sprites_graphics_index(value);
        self.sync();
    }

    pub(crate) fn set_cur_object_index(&mut self, value: u8) {
        self.state.set_cur_object_index(value);
        self.sync();
    }

    pub(crate) fn set_ancilla_alloc_rotate(&mut self, value: u8) {
        self.state.set_ancilla_alloc_rotate(value);
        self.sync();
    }

    pub(crate) fn decrement_ancilla_alloc_rotate(&mut self) -> u8 {
        let value = self.state.decrement_ancilla_alloc_rotate();
        self.sync();
        value
    }

    pub(crate) fn clear_ancilla_alloc_rotate(&mut self) {
        self.state.clear_ancilla_alloc_rotate();
        self.sync();
    }

    pub(crate) fn set_alt_sprites_flag(&mut self, value: u8) {
        self.state.set_alt_sprites_flag(value);
        self.sync();
    }

    pub(crate) fn clear_alt_sprites_flag(&mut self) {
        self.state.clear_alt_sprites_flag();
        self.sync();
    }

    pub(crate) fn increment_ranged_based_toggler(&mut self) {
        self.state.increment_ranged_based_toggler();
        self.sync();
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
        ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + self.where_in_room.len()]
            .copy_from_slice(&self.where_in_room);
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

    pub(crate) fn reset_scratch_a(&self) -> u8 {
        self.reset_scratch_a
    }

    pub(crate) fn armos_knight_remaining_count(&self) -> u8 {
        self.reset_scratch_a
    }

    pub(crate) fn reset_scratch_b(&self) -> u8 {
        self.reset_scratch_b
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

    fn set_current_sprite_position(&mut self, x: u16, y: u16) {
        self.current_sprite_x = x;
        self.current_sprite_y = y;
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
    ram: &'a mut [u8],
}

impl<'a> NativeSpriteWorkspaceBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut SpriteWorkspaceState, ram: &'a mut [u8]) -> Self {
        *state = SpriteWorkspaceState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, SpriteWorkspaceState::load_from_ram(self.ram));
    }

    pub(crate) fn set_room_origin_x_high(&mut self, value: u8) {
        self.state.set_room_origin_x_high(value);
        self.sync();
    }

    pub(crate) fn set_room_origin_y_high(&mut self, value: u8) {
        self.state.set_room_origin_y_high(value);
        self.sync();
    }

    pub(crate) fn set_pickup_slot_cache(&mut self, value: u8) {
        self.state.set_pickup_slot_cache(value);
        self.sync();
    }

    pub(crate) fn set_shared_scratch_a(&mut self, value: u8) {
        self.state.set_shared_scratch_a(value);
        self.sync();
    }

    pub(crate) fn set_room_marker_word(&mut self, slot: usize, value: u16) {
        write_le_u16(
            self.ram,
            crate::game_state::constants::SPRITE_ROOM_MARKER_WORD + slot * 2,
            value,
        );
    }

    pub(crate) fn set_tile_type(&mut self, value: u8) {
        self.state.set_tile_type(value);
        self.sync();
    }

    pub(crate) fn set_prep_shared_counter(&mut self, value: u8) {
        self.state.set_prep_shared_counter(value);
        self.sync();
    }

    pub(crate) fn increment_prep_shared_counter(&mut self) -> u8 {
        let value = self.state.increment_prep_shared_counter();
        self.sync();
        value
    }

    pub(crate) fn decrement_prep_shared_counter(&mut self) -> u8 {
        let value = self.state.decrement_prep_shared_counter();
        self.sync();
        value
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

    pub(crate) fn set_reset_scratch_a(&mut self, value: u8) {
        self.state.set_reset_scratch_a(value);
        self.sync();
    }

    pub(crate) fn clear_agahnim_phase_scratch(&mut self) {
        self.state.set_reset_scratch_a(0);
        self.sync();
    }

    pub(crate) fn set_reset_scratch_b(&mut self, value: u8) {
        self.state.set_reset_scratch_b(value);
        self.sync();
    }

    pub(crate) fn set_graphics_subset(&mut self, slot: usize, value: u8) {
        self.state.set_graphics_subset(slot, value);
        self.sync();
    }

    pub(crate) fn clear_where_in_room(&mut self) {
        self.state.clear_where_in_room();
        self.sync();
    }

    pub(crate) fn clear_draw_priority_override(&mut self) {
        self.state.clear_draw_priority_override();
        self.sync();
    }

    pub(crate) fn set_draw_priority_override_low(&mut self, value: u8) {
        self.state.set_draw_priority_override_low(value);
        self.sync();
    }

    pub(crate) fn set_current_sprite_x(&mut self, value: u16) {
        self.state.set_current_sprite_x(value);
        self.sync();
    }

    pub(crate) fn set_current_sprite_x_low(&mut self, value: u8) {
        self.state.set_current_sprite_x_low(value);
        self.sync();
    }

    pub(crate) fn add_current_sprite_x_low(&mut self, value: u8) {
        self.state.add_current_sprite_x_low(value);
        self.sync();
    }

    pub(crate) fn set_current_sprite_y(&mut self, value: u16) {
        self.state.set_current_sprite_y(value);
        self.sync();
    }

    pub(crate) fn set_current_sprite_y_low(&mut self, value: u8) {
        self.state.set_current_sprite_y_low(value);
        self.sync();
    }

    pub(crate) fn add_current_sprite_y_low(&mut self, value: u8) {
        self.state.add_current_sprite_y_low(value);
        self.sync();
    }

    pub(crate) fn subtract_current_sprite_y_low(&mut self, value: u8) {
        self.state.subtract_current_sprite_y_low(value);
        self.sync();
    }

    pub(crate) fn set_current_sprite_position(&mut self, x: u16, y: u16) {
        self.state.set_current_sprite_position(x, y);
        self.sync();
    }

    pub(crate) fn set_oam_prep_coords(&mut self, x: u16, y: u16) {
        self.state.set_oam_prep_coords(x, y);
        self.sync();
    }

    pub(crate) fn set_killed_sprite_load_block(&mut self, block: u16) {
        self.state.set_killed_sprite_load_block(block);
        self.sync();
    }

    pub(crate) fn set_last_garnish_index(&mut self, index: i32) {
        self.state.set_last_garnish_index(index);
        self.sync();
    }

    pub(crate) fn set_where_in_room(&mut self, room: usize, value: u16) {
        self.state.set_where_in_room(room, value);
        self.sync();
    }
}

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
    kiki_anim_counter: u8,
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
            kiki_anim_counter: ram.get(FOLLOWER_KIKI_ANIM_COUNTER).copied().unwrap_or(0),
            palette_swap_flag: ram.get(FOLLOWER_PALETTE_SWAP_FLAG).copied().unwrap_or(0),
            zelda_rescue_cutscene_state: ram.get(ZELDA_RESCUE_CUTSCENE_STATE).copied().unwrap_or(0),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        ram[FOLLOWER_INDICATOR] = self.indicator;
        ram[TAGALONG_DATA_INDEX] = self.data_index;
        ram[TAGALONG_APPEARANCE_NONE_FLAG] = self.appearance_none_flag;
        ram[FOLLOWER_DROPPED] = self.dropped;
        ram[TAGALONG_HOOKSHOT_INTERLOCK] = self.hookshot_interlock;
        ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX] = self.hookshot_release_tail_index;
        ram[FOLLOWER_TAIL_WRITE_INDEX] = self.tail_write_index;
        ram[TAGALONG_EVENT_FLAGS] = self.event_flags;
        ram[TIMER_TAGALONG_REACQUIRE] = self.reacquire_timer_low;
        ram[TAGALONG_SHARED_STATE_A] = self.shared_state_a;
        ram[TAGALONG_ANIM_FRAME_COUNTER] = self.draw_anim_frame;
        ram[FOLLOWER_JUMP_TIMER] = self.jump_timer;
        write_le_u16(ram, FOLLOWER_SAVED_Y, self.saved_y);
        write_le_u16(ram, FOLLOWER_SAVED_X, self.saved_x);
        ram[FOLLOWER_SAVED_INDOORS] = self.saved_indoor_flag;
        ram[FOLLOWER_SAVED_FLOOR] = self.saved_floor;
        ram[FOLLOWER_KIKI_ANIM_COUNTER] = self.kiki_anim_counter;
        ram[FOLLOWER_PALETTE_SWAP_FLAG] = self.palette_swap_flag;
        ram[ZELDA_RESCUE_CUTSCENE_STATE] = self.zelda_rescue_cutscene_state;
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

    pub(crate) fn set_hookshot_release_tail_index(&mut self, value: u8) {
        self.hookshot_release_tail_index = value;
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

    pub(crate) fn clear_kiki_anim_counter(&mut self) {
        self.kiki_anim_counter = 0;
    }

    pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
        self.zelda_rescue_cutscene_state = value;
    }
}

pub(crate) struct NativeFollowerRuntimeBridgeMut<'a> {
    state: &'a mut FollowerRuntimeState,
    ram: &'a mut [u8],
}

impl<'a> NativeFollowerRuntimeBridgeMut<'a> {
    pub(crate) fn new(state: &'a mut FollowerRuntimeState, ram: &'a mut [u8]) -> Self {
        *state = FollowerRuntimeState::load_from_ram(ram);
        Self { state, ram }
    }

    fn sync(&mut self) {
        self.state.write_to_ram(self.ram);
        self.debug_assert_matches_ram();
    }

    fn debug_assert_matches_ram(&self) {
        debug_assert_eq!(*self.state, FollowerRuntimeState::load_from_ram(self.ram));
    }

    pub(crate) fn clear_palette_swap_flag(&mut self) {
        self.state.clear_palette_swap_flag();
        self.sync();
    }

    pub(crate) fn set_palette_swap_flag(&mut self, value: u8) {
        self.state.set_palette_swap_flag(value);
        self.sync();
    }

    pub(crate) fn set_indicator(&mut self, value: u8) {
        self.state.set_indicator(value);
        self.sync();
    }

    pub(crate) fn set_data_index(&mut self, value: u8) {
        self.state.set_data_index(value);
        self.sync();
    }

    pub(crate) fn advance_data_index_wrapping_at_20(&mut self) {
        self.state.advance_data_index_wrapping_at_20();
        self.sync();
    }

    pub(crate) fn xor_indicator(&mut self, value: u8) {
        self.state.xor_indicator(value);
        self.sync();
    }

    pub(crate) fn set_appearance_none_flag(&mut self, value: u8) {
        self.state.set_appearance_none_flag(value);
        self.sync();
    }

    pub(crate) fn set_dropped(&mut self, value: u8) {
        self.state.set_dropped(value);
        self.sync();
    }

    pub(crate) fn clear_hookshot_interlock(&mut self) {
        self.state.clear_hookshot_interlock();
        self.sync();
    }

    pub(crate) fn set_hookshot_interlock(&mut self) {
        self.state.set_hookshot_interlock();
        self.sync();
    }

    pub(crate) fn clear_event_flags(&mut self) {
        self.state.clear_event_flags();
        self.sync();
    }

    pub(crate) fn or_event_flags(&mut self, value: u8) {
        self.state.or_event_flags(value);
        self.sync();
    }

    pub(crate) fn and_event_flags(&mut self, value: u8) {
        self.state.and_event_flags(value);
        self.sync();
    }

    pub(crate) fn set_hookshot_release_tail_index_from_tail_write_index(&mut self) {
        self.state
            .set_hookshot_release_tail_index_from_tail_write_index();
        self.sync();
    }

    pub(crate) fn set_tail_write_index(&mut self, value: u8) {
        self.state.set_tail_write_index(value);
        self.sync();
    }

    pub(crate) fn increment_tail_write_index(&mut self) {
        self.state.increment_tail_write_index();
        self.sync();
    }

    pub(crate) fn set_hookshot_release_tail_index(&mut self, value: u8) {
        self.state.set_hookshot_release_tail_index(value);
        self.sync();
    }

    pub(crate) fn set_reacquire_timer_low(&mut self, value: u8) {
        self.state.set_reacquire_timer_low(value);
        self.sync();
    }

    pub(crate) fn decrement_reacquire_timer_low(&mut self) {
        self.state.decrement_reacquire_timer_low();
        self.sync();
    }

    pub(crate) fn set_reacquire_timer(&mut self, value: u16) {
        self.state.set_reacquire_timer(value);
        self.sync();
    }

    pub(crate) fn clear_tagalong_shared_state_a(&mut self) {
        self.state.clear_tagalong_shared_state_a();
        self.sync();
    }

    pub(crate) fn clear_draw_anim_frame(&mut self) {
        self.state.clear_draw_anim_frame();
        self.sync();
    }

    pub(crate) fn increment_and_cycle_draw_anim_frame(&mut self) {
        self.state.increment_and_cycle_draw_anim_frame();
        self.sync();
    }

    pub(crate) fn clear_jump_timer(&mut self) {
        self.state.clear_jump_timer();
        self.sync();
    }

    pub(crate) fn set_saved_y(&mut self, value: u16) {
        self.state.set_saved_y(value);
        self.sync();
    }

    pub(crate) fn set_saved_x(&mut self, value: u16) {
        self.state.set_saved_x(value);
        self.sync();
    }

    pub(crate) fn set_saved_indoor_flag(&mut self, value: u8) {
        self.state.set_saved_indoor_flag(value);
        self.sync();
    }

    pub(crate) fn set_saved_floor(&mut self, value: u8) {
        self.state.set_saved_floor(value);
        self.sync();
    }

    pub(crate) fn clear_kiki_anim_counter(&mut self) {
        self.state.clear_kiki_anim_counter();
        self.sync();
    }

    pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
        self.state.set_zelda_rescue_cutscene_state(value);
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
        *state = GarnishRuntimeState::load_from_ram(ram);
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
        *state = FailedSpinSparkleSpawnState::load_from_ram(ram);
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
