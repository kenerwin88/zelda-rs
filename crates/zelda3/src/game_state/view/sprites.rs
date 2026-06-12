use super::*;

pub(crate) struct FollowerStateView<'a> {
    ram: &'a [u8],
}

impl<'a> FollowerStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn indicator(&self) -> u8 {
        byte(self.ram, FOLLOWER_INDICATOR)
    }

    pub(crate) fn indicator_word(&self) -> u16 {
        word(self.ram, FOLLOWER_INDICATOR)
    }

    pub(crate) fn data_index(&self) -> u8 {
        byte(self.ram, TAGALONG_DATA_INDEX)
    }

    pub(crate) fn data_index_word(&self) -> u16 {
        word(self.ram, TAGALONG_DATA_INDEX)
    }

    pub(crate) fn appearance_none_flag(&self) -> u8 {
        byte(self.ram, TAGALONG_APPEARANCE_NONE_FLAG)
    }

    pub(crate) fn dropped(&self) -> u8 {
        byte(self.ram, FOLLOWER_DROPPED)
    }

    pub(crate) fn hookshot_interlock(&self) -> u8 {
        byte(self.ram, TAGALONG_HOOKSHOT_INTERLOCK)
    }

    pub(crate) fn hookshot_interlock_is_clear(&self) -> bool {
        self.hookshot_interlock() == 0
    }

    pub(crate) fn tail_write_index(&self) -> u8 {
        byte(self.ram, FOLLOWER_TAIL_WRITE_INDEX)
    }

    pub(crate) fn hookshot_release_tail_index(&self) -> u8 {
        byte(self.ram, FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX)
    }

    pub(crate) fn event_flags(&self) -> u8 {
        byte(self.ram, TAGALONG_EVENT_FLAGS)
    }

    pub(crate) fn reacquire_timer_low(&self) -> u8 {
        byte(self.ram, TIMER_TAGALONG_REACQUIRE)
    }

    pub(crate) fn reacquire_timer(&self) -> u16 {
        word(self.ram, TIMER_TAGALONG_REACQUIRE)
    }

    pub(crate) fn draw_anim_frame(&self) -> u8 {
        byte(self.ram, TAGALONG_ANIM_FRAME_COUNTER)
    }

    pub(crate) fn saved_y(&self) -> u16 {
        word(self.ram, FOLLOWER_SAVED_Y)
    }

    pub(crate) fn saved_x(&self) -> u16 {
        word(self.ram, FOLLOWER_SAVED_X)
    }

    pub(crate) fn saved_indoor_flag(&self) -> u8 {
        byte(self.ram, FOLLOWER_SAVED_INDOORS)
    }

    pub(crate) fn saved_floor(&self) -> u8 {
        byte(self.ram, FOLLOWER_SAVED_FLOOR)
    }

    pub(crate) fn palette_swap_flag(&self) -> u8 {
        byte(self.ram, FOLLOWER_PALETTE_SWAP_FLAG)
    }

    /// Cutscene progress byte shared by the Priest and rescued Zelda
    /// during the opening rescue sequence.
    pub(crate) fn zelda_rescue_cutscene_state(&self) -> u8 {
        byte(self.ram, ZELDA_RESCUE_CUTSCENE_STATE)
    }
}

pub(crate) struct FollowerStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> FollowerStateViewMut<'a> {
    pub(crate) fn clear_palette_swap_flag(&mut self) {
        self.ram[FOLLOWER_PALETTE_SWAP_FLAG] = 0;
    }

    pub(crate) fn set_palette_swap_flag(&mut self, value: u8) {
        self.ram[FOLLOWER_PALETTE_SWAP_FLAG] = value;
    }

    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_indicator(&mut self, value: u8) {
        self.ram[FOLLOWER_INDICATOR] = value;
    }

    pub(crate) fn set_data_index(&mut self, value: u8) {
        self.ram[TAGALONG_DATA_INDEX] = value;
    }

    pub(crate) fn advance_data_index_wrapping_at_20(&mut self) {
        self.ram[TAGALONG_DATA_INDEX] = if self.ram[TAGALONG_DATA_INDEX].wrapping_add(1) >= 20 {
            0
        } else {
            self.ram[TAGALONG_DATA_INDEX].wrapping_add(1)
        };
    }

    pub(crate) fn xor_indicator(&mut self, value: u8) {
        self.ram[FOLLOWER_INDICATOR] ^= value;
    }

    pub(crate) fn set_appearance_none_flag(&mut self, value: u8) {
        self.ram[TAGALONG_APPEARANCE_NONE_FLAG] = value;
    }

    pub(crate) fn set_dropped(&mut self, value: u8) {
        self.ram[FOLLOWER_DROPPED] = value;
    }

    pub(crate) fn clear_hookshot_interlock(&mut self) {
        self.ram[TAGALONG_HOOKSHOT_INTERLOCK] = 0;
    }

    pub(crate) fn set_hookshot_interlock(&mut self) {
        self.ram[TAGALONG_HOOKSHOT_INTERLOCK] = 1;
    }

    pub(crate) fn clear_event_flags(&mut self) {
        self.ram[TAGALONG_EVENT_FLAGS] = 0;
    }

    pub(crate) fn or_event_flags(&mut self, value: u8) {
        self.ram[TAGALONG_EVENT_FLAGS] |= value;
    }

    pub(crate) fn and_event_flags(&mut self, value: u8) {
        self.ram[TAGALONG_EVENT_FLAGS] &= value;
    }

    pub(crate) fn set_hookshot_release_tail_index_from_tail_write_index(&mut self) {
        self.ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX] = self.ram[FOLLOWER_TAIL_WRITE_INDEX];
    }

    pub(crate) fn set_tail_write_index(&mut self, value: u8) {
        self.ram[FOLLOWER_TAIL_WRITE_INDEX] = value;
    }

    pub(crate) fn increment_tail_write_index(&mut self) {
        self.ram[FOLLOWER_TAIL_WRITE_INDEX] = self.ram[FOLLOWER_TAIL_WRITE_INDEX].wrapping_add(1);
    }

    pub(crate) fn set_hookshot_release_tail_index(&mut self, value: u8) {
        self.ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX] = value;
    }

    pub(crate) fn set_reacquire_timer_low(&mut self, value: u8) {
        self.ram[TIMER_TAGALONG_REACQUIRE] = value;
    }

    pub(crate) fn decrement_reacquire_timer_low(&mut self) {
        self.ram[TIMER_TAGALONG_REACQUIRE] = self.ram[TIMER_TAGALONG_REACQUIRE].wrapping_sub(1);
    }

    pub(crate) fn set_reacquire_timer(&mut self, value: u16) {
        write_le_u16(self.ram, TIMER_TAGALONG_REACQUIRE, value);
    }

    pub(crate) fn clear_tagalong_shared_state_a(&mut self) {
        self.ram[TAGALONG_SHARED_STATE_A] = 0;
    }

    pub(crate) fn clear_draw_anim_frame(&mut self) {
        self.ram[TAGALONG_ANIM_FRAME_COUNTER] = 0;
    }

    pub(crate) fn increment_and_cycle_draw_anim_frame(&mut self) {
        self.ram[TAGALONG_ANIM_FRAME_COUNTER] =
            self.ram[TAGALONG_ANIM_FRAME_COUNTER].wrapping_add(1);
        if self.ram[TAGALONG_ANIM_FRAME_COUNTER] == 3 {
            self.ram[TAGALONG_ANIM_FRAME_COUNTER] = 0;
        }
    }

    pub(crate) fn clear_jump_timer(&mut self) {
        self.ram[FOLLOWER_JUMP_TIMER] = 0;
    }

    pub(crate) fn set_saved_y(&mut self, value: u16) {
        write_le_u16(self.ram, FOLLOWER_SAVED_Y, value);
    }

    pub(crate) fn set_saved_x(&mut self, value: u16) {
        write_le_u16(self.ram, FOLLOWER_SAVED_X, value);
    }

    pub(crate) fn set_saved_indoor_flag(&mut self, value: u8) {
        self.ram[FOLLOWER_SAVED_INDOORS] = value;
    }

    pub(crate) fn set_saved_floor(&mut self, value: u8) {
        self.ram[FOLLOWER_SAVED_FLOOR] = value;
    }

    pub(crate) fn clear_kiki_anim_counter(&mut self) {
        self.ram[FOLLOWER_KIKI_ANIM_COUNTER] = 0;
    }

    pub(crate) fn set_zelda_rescue_cutscene_state(&mut self, value: u8) {
        self.ram[ZELDA_RESCUE_CUTSCENE_STATE] = value;
    }
}

pub(crate) struct ChainChompHistoryView<'a> {
    ram: &'a [u8],
}

pub(crate) struct AncillaSpawnScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> AncillaSpawnScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn write_failed_spin_sparkle(&mut self, step: u8, x: u16, y: u16) {
        self.ram[ANCILLA_ITEM_TO_LINK - 1] = 0;
        self.ram[ANCILLA_STEP - 1] = step;
        self.ram[ANCILLA_TIMER - 1] = 4;
        self.ram[ANCILLA_AUX_TIMER - 1] = 3;
        self.ram[ANCILLA_X_LO - 1] = x as u8;
        self.ram[ANCILLA_X_HI - 1] = (x >> 8) as u8;
        self.ram[ANCILLA_Y_LO - 1] = y as u8;
        self.ram[ANCILLA_Y_HI - 1] = (y >> 8) as u8;
    }
}

impl<'a> ChainChompHistoryView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn x(&self, position: usize) -> u16 {
        word(self.ram, CHAIN_CHOMP_HISTORY_X + position * 2)
    }

    pub(crate) fn y(&self, position: usize) -> u16 {
        word(self.ram, CHAIN_CHOMP_HISTORY_Y + position * 2)
    }
}

pub(crate) struct ChainChompHistoryViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> ChainChompHistoryViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_x(&mut self, position: usize, value: u16) {
        write_le_u16(self.ram, CHAIN_CHOMP_HISTORY_X + position * 2, value);
    }

    pub(crate) fn set_y(&mut self, position: usize, value: u16) {
        write_le_u16(self.ram, CHAIN_CHOMP_HISTORY_Y + position * 2, value);
    }
}

pub(crate) struct MazeGameTimerRawView<'a> {
    ram: &'a [u8],
}

impl<'a> MazeGameTimerRawView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn elapsed_low(&self) -> u16 {
        word(self.ram, MAZE_GAME_TIMER_LO)
    }

    pub(crate) fn elapsed_high(&self) -> u16 {
        word(self.ram, MAZE_GAME_TIMER_HI)
    }

    pub(crate) fn snapshot_low(&self) -> u16 {
        word(self.ram, MAZE_GAME_TIMER_SNAPSHOT_LO)
    }
}

pub(crate) struct MazeGameTimerRawViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MazeGameTimerRawViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_elapsed(&mut self) {
        write_le_u16(self.ram, MAZE_GAME_TIMER_LO, 0);
        write_le_u16(self.ram, MAZE_GAME_TIMER_HI, 0);
    }

    pub(crate) fn increment_elapsed_low(&mut self) -> u16 {
        let value = word(self.ram, MAZE_GAME_TIMER_LO).wrapping_add(1);
        write_le_u16(self.ram, MAZE_GAME_TIMER_LO, value);
        value
    }

    pub(crate) fn increment_elapsed_high(&mut self) -> u16 {
        let value = word(self.ram, MAZE_GAME_TIMER_HI).wrapping_add(1);
        write_le_u16(self.ram, MAZE_GAME_TIMER_HI, value);
        value
    }

    pub(crate) fn capture_snapshot(&mut self) {
        let low = word(self.ram, MAZE_GAME_TIMER_LO);
        let high = word(self.ram, MAZE_GAME_TIMER_HI);
        write_le_u16(self.ram, MAZE_GAME_TIMER_SNAPSHOT_LO, low);
        write_le_u16(self.ram, MAZE_GAME_TIMER_SNAPSHOT_HI, high);
    }
}

pub(crate) struct TagalongSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> TagalongSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            TAGALONG_X_LO + self.slot,
            TAGALONG_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            TAGALONG_Y_LO + self.slot,
            TAGALONG_Y_HI + self.slot,
        )
    }

    pub(crate) fn z(&self) -> u8 {
        byte(self.ram, TAGALONG_Z + self.slot)
    }

    pub(crate) fn z_signed(&self) -> i8 {
        self.z() as i8
    }

    pub(crate) fn is_above_ground(&self) -> bool {
        self.z_signed() > 0
    }

    pub(crate) fn layer_bits(&self) -> u8 {
        byte(self.ram, TAGALONG_LAYERBITS + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        self.layer_bits() & 3
    }
}

pub(crate) struct TagalongSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> TagalongSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            TAGALONG_X_LO + self.slot,
            TAGALONG_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            TAGALONG_Y_LO + self.slot,
            TAGALONG_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[TAGALONG_Y_HI + self.slot] = value;
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        self.set_x(x);
        self.set_y(y);
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.ram[TAGALONG_Z + self.slot] = value;
    }

    pub(crate) fn set_layer_bits(&mut self, value: u8) {
        self.ram[TAGALONG_LAYERBITS + self.slot] = value;
    }
}

pub(crate) struct ArrghusPuffHomeView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> ArrghusPuffHomeView<'a> {
    pub(crate) fn new(ram: &'a [u8], puff_slot: usize) -> Self {
        Self {
            ram,
            slot: puff_slot + 7,
        }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_X_LO + self.slot,
            OVERLORD_Y_LO + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN1 + self.slot,
            OVERLORD_GEN3 + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_LO + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN1 + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN3 + self.slot)
    }
}

pub(crate) struct ArmosKnightHomeView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> ArmosKnightHomeView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_X_HI + self.slot,
            OVERLORD_Y_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN2 + self.slot,
            OVERLORD_FLOOR + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, OVERLORD_X_HI + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN2 + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, OVERLORD_FLOOR + self.slot)
    }
}

pub(crate) struct ArmosKnightHomeViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> ArmosKnightHomeViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_position(&mut self, x: u16, y: u16) {
        write_position(
            self.ram,
            OVERLORD_X_HI + self.slot,
            OVERLORD_Y_HI + self.slot,
            x,
        );
        write_position(
            self.ram,
            OVERLORD_GEN2 + self.slot,
            OVERLORD_FLOOR + self.slot,
            y,
        );
    }
}

pub(crate) struct CachedSpriteSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> CachedSpriteSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn state(&self) -> u8 {
        byte(self.ram, ALT_SPRITE_STATE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.state() != 0
    }

    pub(crate) fn type_byte(&self) -> u8 {
        byte(self.ram, ALT_SPRITE_TYPE + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, ALT_SPRITE_Y_HI + self.slot)
    }
}

pub(crate) struct CachedSpriteSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> CachedSpriteSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn clear_state(&mut self) {
        self.ram[ALT_SPRITE_STATE + self.slot] = 0;
    }

    pub(crate) fn initialize_trinexx_component(&mut self) {
        self.ram[ALT_SPRITE_TYPE + self.slot] = 0x40;
        self.ram[ALT_SPRITE_X_HI + self.slot] = 0;
        self.ram[ALT_SPRITE_Y_HI + self.slot] = 0;
    }

    pub(crate) fn set_type_byte(&mut self, value: u8) {
        self.ram[ALT_SPRITE_TYPE + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[ALT_SPRITE_Y_HI + self.slot] = value;
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
    }

    pub(crate) fn cache_live_fields(&mut self) {
        for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
            self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
        }
    }

    pub(crate) fn load_cached_into_live(&mut self, backup: &mut [u8; 24]) {
        for i in 0..CACHED_SPRITE_LIVE_FIELDS.len() {
            backup[i] = self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot];
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] =
                self.ram[CACHED_SPRITE_ALT_FIELDS[i] + self.slot];
        }
    }

    pub(crate) fn restore_live_from_backup(&mut self, backup: &[u8; 24]) {
        for i in (0..CACHED_SPRITE_LIVE_FIELDS.len()).rev() {
            self.ram[CACHED_SPRITE_LIVE_FIELDS[i] + self.slot] = backup[i];
        }
    }
}

pub(crate) type AltSpriteSlotViewMut<'a> = CachedSpriteSlotViewMut<'a>;

pub(crate) struct SpriteSystemView<'a> {
    ram: &'a [u8],
}

impl<'a> SpriteSystemView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn limit_instance(&self) -> u8 {
        byte(self.ram, SPRITE_LIMIT_INSTANCE)
    }

    /// Animation counter shared by Blind's head and the tutorial guards.
    pub(crate) fn blind_head_anim_counter(&self) -> u8 {
        byte(self.ram, BLIND_HEAD_ANIM_COUNTER)
    }

    pub(crate) fn chr_halfslot_state(&self) -> u8 {
        byte(self.ram, SPRITE_CHR_HALFSLOT_STATE)
    }

    pub(crate) fn alert_flag(&self) -> u8 {
        byte(self.ram, SPRITE_ALERT_FLAG)
    }

    pub(crate) fn graphics_index(&self) -> u8 {
        byte(self.ram, SPRITE_GRAPHICS_INDEX)
    }

    pub(crate) fn saved_special_exit_graphics_index(&self) -> u8 {
        byte(self.ram, SPRITE_GRAPHICS_INDEX_SPEXIT)
    }

    pub(crate) fn saved_exit_graphics_index(&self) -> u8 {
        byte(self.ram, SPRITE_GRAPHICS_INDEX_EXIT)
    }

    pub(crate) fn alt_sprite_spawned_flag(&self) -> u8 {
        byte(self.ram, ALT_SPRITE_SPAWNED_FLAG)
    }

    pub(crate) fn cur_object_index(&self) -> u8 {
        byte(self.ram, CUR_OBJECT_INDEX)
    }

    pub(crate) fn ancilla_alloc_rotate(&self) -> u8 {
        byte(self.ram, ANCILLA_ALLOC_ROTATE)
    }

    pub(crate) fn alt_sprites_flag(&self) -> u8 {
        byte(self.ram, ALT_SPRITES_FLAG)
    }

    pub(crate) fn ranged_based_toggler(&self) -> u8 {
        byte(self.ram, SPR_RANGED_BASED_TOGGLER)
    }

    pub(crate) fn main_tile_theme(&self) -> u8 {
        byte(self.ram, MAIN_TILE_THEME_INDEX)
    }
}

pub(crate) struct SpriteSystemViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SpriteSystemViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_limit_instance(&mut self, value: u8) {
        self.ram[SPRITE_LIMIT_INSTANCE] = value;
    }

    pub(crate) fn set_blind_head_anim_counter(&mut self, value: u8) {
        self.ram[BLIND_HEAD_ANIM_COUNTER] = value;
    }

    pub(crate) fn increment_blind_head_anim_counter(&mut self) {
        self.ram[BLIND_HEAD_ANIM_COUNTER] = self.ram[BLIND_HEAD_ANIM_COUNTER].wrapping_add(1);
    }

    pub(crate) fn increment_limit_instance(&mut self) -> u8 {
        self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_add(1);
        self.ram[SPRITE_LIMIT_INSTANCE]
    }

    pub(crate) fn decrement_limit_instance(&mut self) -> u8 {
        self.ram[SPRITE_LIMIT_INSTANCE] = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_sub(1);
        self.ram[SPRITE_LIMIT_INSTANCE]
    }

    pub(crate) fn set_chr_halfslot_state(&mut self, value: u8) {
        self.ram[SPRITE_CHR_HALFSLOT_STATE] = value;
    }

    pub(crate) fn set_alert_flag(&mut self, value: u8) {
        self.ram[SPRITE_ALERT_FLAG] = value;
    }

    pub(crate) fn decrement_alert_flag(&mut self) -> u8 {
        self.ram[SPRITE_ALERT_FLAG] = self.ram[SPRITE_ALERT_FLAG].wrapping_sub(1);
        self.ram[SPRITE_ALERT_FLAG]
    }

    pub(crate) fn set_graphics_index(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS_INDEX] = value;
    }

    pub(crate) fn save_special_exit_graphics_index(&mut self) {
        self.ram[SPRITE_GRAPHICS_INDEX_SPEXIT] = self.ram[SPRITE_GRAPHICS_INDEX];
    }

    pub(crate) fn restore_special_exit_graphics_index(&mut self) {
        self.ram[SPRITE_GRAPHICS_INDEX] = self.ram[SPRITE_GRAPHICS_INDEX_SPEXIT];
    }

    pub(crate) fn restore_exit_graphics_index(&mut self) {
        self.ram[SPRITE_GRAPHICS_INDEX] = self.ram[SPRITE_GRAPHICS_INDEX_EXIT];
    }

    pub(crate) fn fill_live_states(&mut self, value: u8) {
        self.ram[SPRITE_STATE..SPRITE_STATE + 16].fill(value);
    }

    pub(crate) fn clear_live_table_pages(&mut self) {
        self.ram[SPRITE_Y_LO..SPRITE_Y_LO + 256 * 3].fill(0);
    }

    pub(crate) fn set_alt_sprite_spawned_flag(&mut self, value: u8) {
        self.ram[ALT_SPRITE_SPAWNED_FLAG] = value;
    }

    pub(crate) fn set_main_tile_theme(&mut self, value: u8) {
        self.ram[MAIN_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_aux_tile_theme(&mut self, value: u8) {
        self.ram[AUX_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_misc_sprites_graphics_index(&mut self, value: u8) {
        self.ram[MISC_SPRITES_GRAPHICS_INDEX] = value;
    }

    pub(crate) fn set_cur_object_index(&mut self, value: u8) {
        self.ram[CUR_OBJECT_INDEX] = value;
    }

    pub(crate) fn set_ancilla_alloc_rotate(&mut self, value: u8) {
        self.ram[ANCILLA_ALLOC_ROTATE] = value;
    }

    pub(crate) fn decrement_ancilla_alloc_rotate(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ALLOC_ROTATE].wrapping_sub(1);
        self.ram[ANCILLA_ALLOC_ROTATE] = value;
        value
    }

    pub(crate) fn clear_ancilla_alloc_rotate(&mut self) {
        self.ram[ANCILLA_ALLOC_ROTATE] = 0;
    }

    pub(crate) fn set_alt_sprites_flag(&mut self, value: u8) {
        self.ram[ALT_SPRITES_FLAG] = value;
    }

    pub(crate) fn clear_alt_sprites_flag(&mut self) {
        self.ram[ALT_SPRITES_FLAG] = 0;
    }

    pub(crate) fn increment_ranged_based_toggler(&mut self) {
        self.ram[SPR_RANGED_BASED_TOGGLER] = self.ram[SPR_RANGED_BASED_TOGGLER].wrapping_add(1);
    }
}

pub(crate) struct SpriteWorkspaceView<'a> {
    ram: &'a [u8],
}

impl<'a> SpriteWorkspaceView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn room_origin_x_high(&self) -> u8 {
        byte(self.ram, SPRITE_ROOM_ORIGIN_X_HI)
    }

    pub(crate) fn room_origin_y_high(&self) -> u8 {
        byte(self.ram, SPRITE_ROOM_ORIGIN_Y_HI)
    }

    pub(crate) fn pickup_slot_cache(&self) -> u8 {
        byte(self.ram, SPRITE_PICKUP_SLOT_CACHE)
    }

    pub(crate) fn shared_scratch_a(&self) -> u8 {
        byte(self.ram, SPRITE_SHARED_WORK_A)
    }

    pub(crate) fn tile_type(&self) -> u8 {
        byte(self.ram, SPRITE_TILETYPE)
    }

    pub(crate) fn prep_shared_counter(&self) -> u8 {
        byte(self.ram, SPRITE_RESET_WORK_A)
    }

    pub(crate) fn reset_scratch_a(&self) -> u8 {
        byte(self.ram, SPRITE_RESET_WORK_A)
    }

    /// Alias of `reset_scratch_a`: the Armos Knights fight reuses the
    /// shared scratch byte as the remaining-knight counter.
    pub(crate) fn armos_knight_remaining_count(&self) -> u8 {
        byte(self.ram, SPRITE_RESET_WORK_A)
    }

    pub(crate) fn reset_scratch_b(&self) -> u8 {
        byte(self.ram, SPRITE_RESET_WORK_B)
    }

    pub(crate) fn graphics_subset(&self, slot: usize) -> u8 {
        byte(self.ram, SPRITE_GFX_SUBSET_0 + slot)
    }

    pub(crate) fn draw_priority_override(&self) -> u16 {
        read_le_u16(self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE)
    }

    pub(crate) fn current_sprite_x(&self) -> u16 {
        read_le_u16(self.ram, CUR_SPRITE_X)
    }

    pub(crate) fn current_sprite_x_low(&self) -> u8 {
        byte(self.ram, CUR_SPRITE_X)
    }

    pub(crate) fn current_sprite_y(&self) -> u16 {
        read_le_u16(self.ram, CUR_SPRITE_Y)
    }

    pub(crate) fn current_sprite_y_low(&self) -> u8 {
        byte(self.ram, CUR_SPRITE_Y)
    }

    pub(crate) fn oam_prep_x(&self) -> u16 {
        read_le_u16(self.ram, SPRITE_OAM_PREP_X)
    }

    pub(crate) fn oam_prep_y(&self) -> u16 {
        read_le_u16(self.ram, SPRITE_OAM_PREP_Y)
    }

    pub(crate) fn where_in_room(&self, room: usize) -> u16 {
        read_le_u16(self.ram, SPRITE_WHERE_IN_ROOM + room * 2)
    }
}

pub(crate) struct SpriteWorkspaceViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SpriteWorkspaceViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_room_origin_x_high(&mut self, value: u8) {
        self.ram[SPRITE_ROOM_ORIGIN_X_HI] = value;
    }

    pub(crate) fn set_room_origin_y_high(&mut self, value: u8) {
        self.ram[SPRITE_ROOM_ORIGIN_Y_HI] = value;
    }

    pub(crate) fn set_pickup_slot_cache(&mut self, value: u8) {
        self.ram[SPRITE_PICKUP_SLOT_CACHE] = value;
    }

    pub(crate) fn set_shared_scratch_a(&mut self, value: u8) {
        self.ram[SPRITE_SHARED_WORK_A] = value;
    }

    pub(crate) fn set_room_marker_word(&mut self, slot: usize, value: u16) {
        write_le_u16(self.ram, SPRITE_ROOM_MARKER_WORD + slot * 2, value);
    }

    pub(crate) fn set_tile_type(&mut self, value: u8) {
        self.ram[SPRITE_TILETYPE] = value;
    }

    pub(crate) fn set_prep_shared_counter(&mut self, value: u8) {
        self.ram[SPRITE_RESET_WORK_A] = value;
    }

    pub(crate) fn increment_prep_shared_counter(&mut self) -> u8 {
        self.ram[SPRITE_RESET_WORK_A] = self.ram[SPRITE_RESET_WORK_A].wrapping_add(1);
        self.ram[SPRITE_RESET_WORK_A]
    }

    pub(crate) fn decrement_prep_shared_counter(&mut self) -> u8 {
        self.ram[SPRITE_RESET_WORK_A] = self.ram[SPRITE_RESET_WORK_A].wrapping_sub(1);
        self.ram[SPRITE_RESET_WORK_A]
    }

    /// Alias of `decrement_prep_shared_counter` for the Armos Knights
    /// fight, which reuses the shared scratch byte as the
    /// remaining-knight counter. Returns the new value.
    pub(crate) fn decrement_armos_knight_remaining_count(&mut self) -> u8 {
        self.ram[SPRITE_RESET_WORK_A] = self.ram[SPRITE_RESET_WORK_A].wrapping_sub(1);
        self.ram[SPRITE_RESET_WORK_A]
    }

    /// Alias of `set_prep_shared_counter(0)` for the Vitreous fight,
    /// which reuses the shared scratch byte as the eyeball release
    /// counter.
    pub(crate) fn clear_vitreous_eyeball_release_count(&mut self) {
        self.ram[SPRITE_RESET_WORK_A] = 0;
    }

    pub(crate) fn set_reset_scratch_a(&mut self, value: u8) {
        self.ram[SPRITE_RESET_WORK_A] = value;
    }

    pub(crate) fn clear_agahnim_phase_scratch(&mut self) {
        self.ram[SPRITE_RESET_WORK_A] = 0;
    }

    pub(crate) fn set_reset_scratch_b(&mut self, value: u8) {
        self.ram[SPRITE_RESET_WORK_B] = value;
    }

    pub(crate) fn set_graphics_subset(&mut self, slot: usize, value: u8) {
        self.ram[SPRITE_GFX_SUBSET_0 + slot] = value;
    }

    pub(crate) fn clear_where_in_room(&mut self) {
        self.ram[SPRITE_WHERE_IN_ROOM..SPRITE_WHERE_IN_ROOM + 0x1000].fill(0);
    }

    pub(crate) fn clear_draw_priority_override(&mut self) {
        write_le_u16(self.ram, SPRITE_DRAW_PRIORITY_OVERRIDE, 0);
    }

    pub(crate) fn set_draw_priority_override_low(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_PRIORITY_OVERRIDE] = value;
    }

    pub(crate) fn set_current_sprite_x(&mut self, value: u16) {
        write_le_u16(self.ram, CUR_SPRITE_X, value);
    }

    pub(crate) fn set_current_sprite_x_low(&mut self, value: u8) {
        self.ram[CUR_SPRITE_X] = value;
    }

    pub(crate) fn add_current_sprite_x_low(&mut self, value: u8) {
        self.ram[CUR_SPRITE_X] = self.ram[CUR_SPRITE_X].wrapping_add(value);
    }

    pub(crate) fn set_current_sprite_y(&mut self, value: u16) {
        write_le_u16(self.ram, CUR_SPRITE_Y, value);
    }

    pub(crate) fn set_current_sprite_y_low(&mut self, value: u8) {
        self.ram[CUR_SPRITE_Y] = value;
    }

    pub(crate) fn add_current_sprite_y_low(&mut self, value: u8) {
        self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_add(value);
    }

    pub(crate) fn subtract_current_sprite_y_low(&mut self, value: u8) {
        self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_sub(value);
    }

    pub(crate) fn set_current_sprite_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, CUR_SPRITE_X, x);
        write_le_u16(self.ram, CUR_SPRITE_Y, y);
    }

    pub(crate) fn set_oam_prep_coords(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, SPRITE_OAM_PREP_X, x);
        write_le_u16(self.ram, SPRITE_OAM_PREP_Y, y);
    }

    pub(crate) fn set_killed_sprite_load_block(&mut self, block: u16) {
        self.ram[SPRITE_LOAD_BLOCK_STATE] = block as u8;
        write_le_u16(
            self.ram,
            SPRITE_LOAD_BLOCK_STATE + 1,
            (block >> 3).wrapping_add(0xef80),
        );
    }

    pub(crate) fn set_last_garnish_index(&mut self, index: i32) {
        self.ram[SPRITE_LAST_GARNISH_INDEX] = index as u8;
    }

    pub(crate) fn set_where_in_room(&mut self, room: usize, value: u16) {
        write_le_u16(self.ram, SPRITE_WHERE_IN_ROOM + room * 2, value);
    }
}

pub(crate) struct EnemyDamageDataView<'a> {
    ram: &'a [u8],
}

impl<'a> EnemyDamageDataView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn entry(&self, index: usize) -> u8 {
        byte(self.ram, ENEMY_DAMAGE_DATA + index)
    }
}

pub(crate) struct EnemyDamageDataViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> EnemyDamageDataViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_entry(&mut self, index: usize, value: u8) {
        self.ram[ENEMY_DAMAGE_DATA + index] = value;
    }
}

pub(crate) struct EtherOrbitView<'a> {
    ram: &'a [u8],
}

impl<'a> EtherOrbitView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn angle(&self, slot: usize) -> u8 {
        byte(self.ram, ETHER_ANGLE + slot)
    }

    pub(crate) fn radius(&self) -> u8 {
        byte(self.ram, ETHER_RADIUS)
    }

    pub(crate) fn beam_top_bucket(&self) -> u8 {
        byte(self.ram, ETHER_BEAM_TOP_BUCKET)
    }

    pub(crate) fn beam_y(&self) -> u16 {
        read_le_u16(self.ram, ETHER_BEAM_Y)
    }

    pub(crate) fn orbit_x(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORBIT_X)
    }

    pub(crate) fn orbit_y(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORBIT_Y)
    }

    /// Sword beam scratch coordinates alias the ether orbit scratch words.
    pub(crate) fn swordbeam_temp_x(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORBIT_X)
    }

    pub(crate) fn swordbeam_temp_y(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORBIT_Y)
    }

    pub(crate) fn orb_x(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORB_X)
    }

    pub(crate) fn orb_y(&self) -> u16 {
        read_le_u16(self.ram, ETHER_ORB_Y)
    }
}

pub(crate) struct EtherOrbitViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> EtherOrbitViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_angle(&mut self, slot: usize, value: u8) {
        self.ram[ETHER_ANGLE + slot] = value;
    }

    pub(crate) fn advance_angle(&mut self, slot: usize) -> u8 {
        let next = self.ram[ETHER_ANGLE + slot].wrapping_add(1) & 0x3f;
        self.ram[ETHER_ANGLE + slot] = next;
        next
    }

    pub(crate) fn set_radius(&mut self, value: u8) {
        self.ram[ETHER_RADIUS] = value;
    }

    pub(crate) fn tick_spin_countdown(&mut self) -> u8 {
        let value = self.ram[ETHER_SPIN_COUNTDOWN].wrapping_sub(1);
        self.ram[ETHER_SPIN_COUNTDOWN] = value;
        value
    }

    pub(crate) fn set_spin_countdown(&mut self, value: u8) {
        self.ram[ETHER_SPIN_COUNTDOWN] = value;
    }

    pub(crate) fn set_beam_top_bucket(&mut self, value: u8) {
        self.ram[ETHER_BEAM_TOP_BUCKET] = value;
    }

    pub(crate) fn initialize_beam_adjusted_y(&mut self, value: u16) {
        write_le_u16(self.ram, ETHER_BEAM_TOP_BUCKET, value);
    }

    pub(crate) fn set_orb_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, ETHER_ORB_X, x);
        write_le_u16(self.ram, ETHER_ORB_Y, y);
    }

    pub(crate) fn set_orbit_position(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, ETHER_ORBIT_X, x);
        write_le_u16(self.ram, ETHER_ORBIT_Y, y);
    }

    /// Sword beam scratch coordinates alias the ether orbit scratch words.
    pub(crate) fn set_swordbeam_temp(&mut self, x: u16, y: u16) {
        write_le_u16(self.ram, ETHER_ORBIT_X, x);
        write_le_u16(self.ram, ETHER_ORBIT_Y, y);
    }

    pub(crate) fn set_beam_y(&mut self, value: u16) {
        write_le_u16(self.ram, ETHER_BEAM_Y, value);
    }
}

pub(crate) struct PrizeDropCycleRawViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> PrizeDropCycleRawViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn take_next_index(&mut self, slot: usize) -> u8 {
        let index = byte(self.ram, PRIZE_DROP_CYCLE + slot);
        self.ram[PRIZE_DROP_CYCLE + slot] = index.wrapping_add(1) & 7;
        index
    }
}

pub(crate) struct DualLayerTileCacheView<'a> {
    ram: &'a [u8],
}

impl<'a> DualLayerTileCacheView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn tile_type(&self, slot: usize) -> u8 {
        byte(self.ram, DUAL_LAYER_TILE_CACHE + slot)
    }
}

pub(crate) struct DualLayerTileCacheViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DualLayerTileCacheViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_tile_type(&mut self, slot: usize, value: u8) {
        self.ram[DUAL_LAYER_TILE_CACHE + slot] = value;
    }
}

pub(crate) struct SpriteSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> SpriteSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn slot(&self) -> u8 {
        self.slot as u8
    }

    pub(crate) fn sprite_type(&self) -> u8 {
        byte(self.ram, SPRITE_TYPE + self.slot)
    }

    pub(crate) fn state(&self) -> u8 {
        byte(self.ram, SPRITE_STATE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.sprite_type() != 0 || self.state() != 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, SPRITE_X_LO + self.slot, SPRITE_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, SPRITE_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, SPRITE_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, SPRITE_Y_LO + self.slot, SPRITE_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, SPRITE_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, SPRITE_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_Y_VELOCITY + self.slot)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        byte(self.ram, SPRITE_Z_VELOCITY + self.slot)
    }

    pub(crate) fn x_recoil(&self) -> u8 {
        byte(self.ram, SPRITE_X_RECOIL + self.slot)
    }

    pub(crate) fn y_recoil(&self) -> u8 {
        byte(self.ram, SPRITE_Y_RECOIL + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn z(&self) -> u8 {
        byte(self.ram, SPRITE_Z + self.slot)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        byte(self.ram, SPRITE_Z_SUBPIXEL + self.slot)
    }

    pub(crate) fn ai_state(&self) -> u8 {
        byte(self.ram, SPRITE_AI_STATE + self.slot)
    }

    pub(crate) fn a(&self) -> u8 {
        byte(self.ram, SPRITE_A + self.slot)
    }

    pub(crate) fn c(&self) -> u8 {
        byte(self.ram, SPRITE_C + self.slot)
    }

    pub(crate) fn b(&self) -> u8 {
        byte(self.ram, SPRITE_B + self.slot)
    }

    pub(crate) fn e(&self) -> u8 {
        byte(self.ram, SPRITE_E + self.slot)
    }

    pub(crate) fn f(&self) -> u8 {
        byte(self.ram, SPRITE_F + self.slot)
    }

    pub(crate) fn g(&self) -> u8 {
        byte(self.ram, SPRITE_G + self.slot)
    }

    pub(crate) fn graphics(&self) -> u8 {
        byte(self.ram, SPRITE_GRAPHICS + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, SPRITE_D + self.slot)
    }

    pub(crate) fn subtype(&self) -> u8 {
        byte(self.ram, SPRITE_SUBTYPE + self.slot)
    }

    pub(crate) fn delay_main(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_MAIN + self.slot)
    }

    pub(crate) fn delay_aux1(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX1 + self.slot)
    }

    pub(crate) fn delay_aux4(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX4 + self.slot)
    }

    pub(crate) fn delay_aux2(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX2 + self.slot)
    }

    pub(crate) fn flags2(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS2 + self.slot)
    }

    pub(crate) fn flags(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS + self.slot)
    }

    pub(crate) fn flags3(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS3 + self.slot)
    }

    pub(crate) fn wall_collision(&self) -> u8 {
        byte(self.ram, SPRITE_WALL_COLLISION + self.slot)
    }

    pub(crate) fn anim_clock(&self) -> u8 {
        byte(self.ram, SPRITE_ANIM_CLOCK + self.slot)
    }

    pub(crate) fn delay_aux3(&self) -> u8 {
        byte(self.ram, SPRITE_DELAY_AUX3 + self.slot)
    }

    pub(crate) fn flags4(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS4 + self.slot)
    }

    pub(crate) fn flags5(&self) -> u8 {
        byte(self.ram, SPRITE_FLAGS5 + self.slot)
    }

    pub(crate) fn health(&self) -> u8 {
        byte(self.ram, SPRITE_HEALTH + self.slot)
    }

    pub(crate) fn hit_timer(&self) -> u8 {
        byte(self.ram, SPRITE_HIT_TIMER + self.slot)
    }

    pub(crate) fn pause(&self) -> u8 {
        byte(self.ram, SPRITE_PAUSE + self.slot)
    }

    pub(crate) fn stunned(&self) -> u8 {
        byte(self.ram, SPRITE_STUNNED + self.slot)
    }

    pub(crate) fn ignore_projectile(&self) -> u8 {
        byte(self.ram, SPRITE_IGNORE_PROJECTILE + self.slot)
    }

    pub(crate) fn draw_work_byte_2(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_2 + self.slot)
    }

    pub(crate) fn n(&self) -> u8 {
        byte(self.ram, SPRITE_N + self.slot)
    }

    pub(crate) fn n_word(&self) -> u16 {
        read_le_u16(self.ram, SPRITE_N + self.slot * 2)
    }

    pub(crate) fn deflection_bits(&self) -> u8 {
        byte(self.ram, SPRITE_DEFL_BITS + self.slot)
    }

    pub(crate) fn bump_damage(&self) -> u8 {
        byte(self.ram, SPRITE_BUMP_DAMAGE + self.slot)
    }

    pub(crate) fn incoming_damage(&self) -> u8 {
        byte(self.ram, SPRITE_INCOMING_DAMAGE + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, SPRITE_FLOOR + self.slot)
    }

    pub(crate) fn room(&self) -> u8 {
        byte(self.ram, SPRITE_ROOM + self.slot)
    }

    pub(crate) fn die_action(&self) -> u8 {
        byte(self.ram, SPRITE_DIE_ACTION + self.slot)
    }

    pub(crate) fn draw_i(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_I + self.slot)
    }

    pub(crate) fn draw_work_byte_3(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_3 + self.slot)
    }

    pub(crate) fn draw_work_byte_4(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_4 + self.slot)
    }

    pub(crate) fn draw_work_byte_5(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_5 + self.slot)
    }

    pub(crate) fn draw_work_byte_1(&self) -> u8 {
        byte(self.ram, SPRITE_DRAW_WORK_BYTE_1 + self.slot)
    }

    pub(crate) fn head_direction(&self) -> u8 {
        byte(self.ram, SPRITE_HEAD_DIR + self.slot)
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        byte(self.ram, SPRITE_OAM_FLAGS + self.slot)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        byte(self.ram, SPRITE_OBJ_PRIO + self.slot)
    }

    pub(crate) fn subtype2(&self) -> u8 {
        byte(self.ram, SPRITE_SUBTYPE2 + self.slot)
    }
}

pub(crate) struct SpriteSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> SpriteSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_sprite_type(&mut self, value: u8) {
        self.ram[SPRITE_TYPE + self.slot] = value;
    }

    pub(crate) fn set_state(&mut self, value: u8) {
        self.ram[SPRITE_STATE + self.slot] = value;
    }

    pub(crate) fn increment_state(&mut self) {
        self.ram[SPRITE_STATE + self.slot] = self.ram[SPRITE_STATE + self.slot].wrapping_add(1);
    }

    pub(crate) fn clear(&mut self) {
        self.set_state(0);
    }

    pub(crate) fn clear_prep_runtime_state(&mut self) {
        for base in [
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
        ] {
            self.ram[base + self.slot] = 0;
        }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            SPRITE_X_LO + self.slot,
            SPRITE_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[SPRITE_X_HI + self.slot] = value;
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = self.ram[SPRITE_X_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.ram[SPRITE_X_LO + self.slot] = self.ram[SPRITE_X_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            SPRITE_Y_LO + self.slot,
            SPRITE_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[SPRITE_Y_HI + self.slot] = value;
    }

    pub(crate) fn add_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = self.ram[SPRITE_Y_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = self.ram[SPRITE_Y_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[SPRITE_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] = value;
    }

    pub(crate) fn decrement_z_velocity(&mut self) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_delay_aux4(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX4 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux1(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX1 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux2(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX2 + self.slot] = value;
    }

    pub(crate) fn set_delay_aux3(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_AUX3 + self.slot] = value;
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.ram[SPRITE_X_VELOCITY + self.slot]
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.ram[SPRITE_Y_VELOCITY + self.slot]
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn xor_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] ^= value;
    }

    pub(crate) fn xor_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] ^= value;
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn negate_z_velocity(&mut self) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn and_x_velocity(&mut self, value: u8) {
        self.ram[SPRITE_X_VELOCITY + self.slot] &= value;
    }

    pub(crate) fn and_y_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] &= value;
    }

    pub(crate) fn shift_x_velocity_left(&mut self, amount: u32) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_shl(amount);
    }

    pub(crate) fn shift_y_velocity_left(&mut self, amount: u32) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_shl(amount);
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            self.ram[SPRITE_X_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            self.ram[SPRITE_Y_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn subtract_z_velocity(&mut self, value: u8) {
        self.ram[SPRITE_Z_VELOCITY + self.slot] =
            self.ram[SPRITE_Z_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn halve_x_velocity(&mut self) {
        self.ram[SPRITE_X_VELOCITY + self.slot] =
            ((self.ram[SPRITE_X_VELOCITY + self.slot] as i8) >> 1) as u8;
    }

    pub(crate) fn halve_y_velocity(&mut self) {
        self.ram[SPRITE_Y_VELOCITY + self.slot] =
            ((self.ram[SPRITE_Y_VELOCITY + self.slot] as i8) >> 1) as u8;
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.ram[SPRITE_Z + self.slot] = value;
    }

    pub(crate) fn add_z(&mut self, value: u8) {
        self.ram[SPRITE_Z + self.slot] = self.ram[SPRITE_Z + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn negate_z_subpixel(&mut self) {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_neg();
    }

    pub(crate) fn increment_z_subpixel(&mut self) -> u8 {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_add(1);
        self.ram[SPRITE_Z_SUBPIXEL + self.slot]
    }

    pub(crate) fn decrement_z_subpixel(&mut self) -> u8 {
        self.ram[SPRITE_Z_SUBPIXEL + self.slot] =
            self.ram[SPRITE_Z_SUBPIXEL + self.slot].wrapping_sub(1);
        self.ram[SPRITE_Z_SUBPIXEL + self.slot]
    }

    pub(crate) fn set_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] = value;
    }

    pub(crate) fn increment_ai_state(&mut self) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(1);
    }

    pub(crate) fn decrement_ai_state(&mut self) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(1);
    }

    pub(crate) fn add_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_ai_state(&mut self, value: u8) {
        self.ram[SPRITE_AI_STATE + self.slot] =
            self.ram[SPRITE_AI_STATE + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = value;
    }

    pub(crate) fn increment_a(&mut self) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(1);
    }

    pub(crate) fn xor_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] ^= value;
    }

    pub(crate) fn add_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_a(&mut self) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(1);
    }

    pub(crate) fn subtract_a(&mut self, value: u8) {
        self.ram[SPRITE_A + self.slot] = self.ram[SPRITE_A + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = value;
    }

    pub(crate) fn increment_b(&mut self) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] = self.ram[SPRITE_B + self.slot].wrapping_sub(value);
    }

    pub(crate) fn xor_b(&mut self, value: u8) {
        self.ram[SPRITE_B + self.slot] ^= value;
    }

    pub(crate) fn set_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] = value;
    }

    pub(crate) fn increment_c(&mut self) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_add(value);
    }

    pub(crate) fn xor_c(&mut self, value: u8) {
        self.ram[SPRITE_C + self.slot] ^= value;
    }

    pub(crate) fn decrement_c(&mut self) {
        self.ram[SPRITE_C + self.slot] = self.ram[SPRITE_C + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_e(&mut self, value: u8) {
        self.ram[SPRITE_E + self.slot] = value;
    }

    pub(crate) fn increment_e(&mut self) -> u8 {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(1);
        self.ram[SPRITE_E + self.slot]
    }

    pub(crate) fn add_e(&mut self, value: u8) {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_e(&mut self) -> u8 {
        self.ram[SPRITE_E + self.slot] = self.ram[SPRITE_E + self.slot].wrapping_sub(1);
        self.ram[SPRITE_E + self.slot]
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] = value;
    }

    pub(crate) fn add_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(value);
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] &= value;
    }

    pub(crate) fn increment_direction(&mut self) {
        self.ram[SPRITE_D + self.slot] = self.ram[SPRITE_D + self.slot].wrapping_add(1);
    }

    pub(crate) fn xor_direction(&mut self, value: u8) {
        self.ram[SPRITE_D + self.slot] ^= value;
    }

    pub(crate) fn set_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] = value;
    }

    pub(crate) fn increment_delay_main(&mut self) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_delay_main(&mut self, value: u8) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] =
            self.ram[SPRITE_DELAY_MAIN + self.slot].wrapping_sub(value);
    }

    pub(crate) fn halve_delay_main(&mut self) {
        self.ram[SPRITE_DELAY_MAIN + self.slot] >>= 1;
    }

    pub(crate) fn set_head_direction(&mut self, value: u8) {
        self.ram[SPRITE_HEAD_DIR + self.slot] = value;
    }

    pub(crate) fn increment_head_direction(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_head_direction(&mut self, value: u8) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_head_direction(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_sub(1);
    }

    pub(crate) fn increment_head_direction_mod16(&mut self) {
        self.ram[SPRITE_HEAD_DIR + self.slot] =
            self.ram[SPRITE_HEAD_DIR + self.slot].wrapping_add(1) & 15;
    }

    pub(crate) fn set_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] = value;
    }

    pub(crate) fn increment_graphics(&mut self) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_graphics(&mut self) {
        self.ram[SPRITE_GRAPHICS + self.slot] =
            self.ram[SPRITE_GRAPHICS + self.slot].wrapping_sub(1);
    }

    pub(crate) fn xor_graphics(&mut self, value: u8) {
        self.ram[SPRITE_GRAPHICS + self.slot] ^= value;
    }

    pub(crate) fn set_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] = value;
    }

    pub(crate) fn add_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] =
            self.ram[SPRITE_FLAGS2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] =
            self.ram[SPRITE_FLAGS2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn and_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] &= value;
    }

    pub(crate) fn or_flags2(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] |= value;
    }

    pub(crate) fn masked_or_flags2(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS2 + self.slot] = (self.ram[SPRITE_FLAGS2 + self.slot] & mask) | value;
    }

    pub(crate) fn set_flags(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS + self.slot] = value;
    }

    pub(crate) fn masked_or_flags(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS + self.slot] = (self.ram[SPRITE_FLAGS + self.slot] & mask) | value;
    }

    pub(crate) fn set_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] = value;
    }

    pub(crate) fn and_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] &= value;
    }

    pub(crate) fn clear_flags3_bits(&mut self, mask: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] &= !mask;
    }

    pub(crate) fn set_flags4(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS4 + self.slot] = value;
    }

    pub(crate) fn increment_flags4(&mut self) {
        self.ram[SPRITE_FLAGS4 + self.slot] = self.ram[SPRITE_FLAGS4 + self.slot].wrapping_add(1);
    }

    pub(crate) fn or_flags4(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS4 + self.slot] |= value;
    }

    pub(crate) fn set_flags5(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS5 + self.slot] = value;
    }

    pub(crate) fn and_flags5(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS5 + self.slot] &= value;
    }

    pub(crate) fn or_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] |= value;
    }

    pub(crate) fn xor_flags3(&mut self, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] ^= value;
    }

    pub(crate) fn masked_or_flags3(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_FLAGS3 + self.slot] = (self.ram[SPRITE_FLAGS3 + self.slot] & mask) | value;
    }

    pub(crate) fn set_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] = value;
    }

    pub(crate) fn increment_subtype2(&mut self) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn decrement_subtype2(&mut self) -> u8 {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(1);
        self.ram[SPRITE_SUBTYPE2 + self.slot]
    }

    pub(crate) fn subtract_subtype2(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE2 + self.slot] =
            self.ram[SPRITE_SUBTYPE2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] = value;
    }

    pub(crate) fn increment_subtype(&mut self) {
        self.ram[SPRITE_SUBTYPE + self.slot] = self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] =
            self.ram[SPRITE_SUBTYPE + self.slot].wrapping_add(value);
    }

    pub(crate) fn and_subtype(&mut self, value: u8) {
        self.ram[SPRITE_SUBTYPE + self.slot] &= value;
    }

    pub(crate) fn decrement_subtype(&mut self) {
        self.ram[SPRITE_SUBTYPE + self.slot] = self.ram[SPRITE_SUBTYPE + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_f(&mut self, value: u8) {
        self.ram[SPRITE_F + self.slot] = value;
    }

    pub(crate) fn subtract_f(&mut self, value: u8) {
        self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(value);
    }

    pub(crate) fn decrement_f(&mut self) {
        self.ram[SPRITE_F + self.slot] = self.ram[SPRITE_F + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = value;
    }

    pub(crate) fn increment_g(&mut self) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(value);
    }

    pub(crate) fn decrement_g(&mut self) {
        self.ram[SPRITE_G + self.slot] = self.ram[SPRITE_G + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[SPRITE_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_room(&mut self, value: u8) {
        self.ram[SPRITE_ROOM + self.slot] = value;
    }

    pub(crate) fn set_x_recoil(&mut self, value: u8) {
        self.ram[SPRITE_X_RECOIL + self.slot] = value;
    }

    pub(crate) fn set_y_recoil(&mut self, value: u8) {
        self.ram[SPRITE_Y_RECOIL + self.slot] = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] = value;
    }

    pub(crate) fn and_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] &= value;
    }

    pub(crate) fn xor_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] ^= value;
    }

    pub(crate) fn or_oam_flags(&mut self, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] |= value;
    }

    pub(crate) fn masked_or_oam_flags(&mut self, mask: u8, value: u8) {
        self.ram[SPRITE_OAM_FLAGS + self.slot] =
            (self.ram[SPRITE_OAM_FLAGS + self.slot] & mask) | value;
    }

    pub(crate) fn set_stunned(&mut self, value: u8) {
        self.ram[SPRITE_STUNNED + self.slot] = value;
    }

    pub(crate) fn decrement_stunned(&mut self) {
        self.ram[SPRITE_STUNNED + self.slot] = self.ram[SPRITE_STUNNED + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_pause(&mut self, value: u8) {
        self.ram[SPRITE_PAUSE + self.slot] = value;
    }

    pub(crate) fn set_health(&mut self, value: u8) {
        self.ram[SPRITE_HEALTH + self.slot] = value;
    }

    pub(crate) fn decrement_health(&mut self) {
        self.ram[SPRITE_HEALTH + self.slot] = self.ram[SPRITE_HEALTH + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_ignore_projectile(&mut self, value: u8) {
        self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_2(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_2 + self.slot] = value;
    }

    pub(crate) fn set_draw_i(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_I + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_3(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] = value;
    }

    pub(crate) fn increment_draw_work_byte_3(&mut self) {
        self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot] =
            self.ram[SPRITE_DRAW_WORK_BYTE_3 + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_draw_work_byte_4(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_4 + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_5(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_5 + self.slot] = value;
    }

    pub(crate) fn set_draw_work_byte_1(&mut self, value: u8) {
        self.ram[SPRITE_DRAW_WORK_BYTE_1 + self.slot] = value;
    }

    pub(crate) fn increment_ignore_projectile(&mut self) {
        self.ram[SPRITE_IGNORE_PROJECTILE + self.slot] =
            self.ram[SPRITE_IGNORE_PROJECTILE + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] = value;
    }

    pub(crate) fn or_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] |= value;
    }

    pub(crate) fn clear_deflection_bits(&mut self, mask: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] &= !mask;
    }

    pub(crate) fn and_deflection_bits(&mut self, value: u8) {
        self.ram[SPRITE_DEFL_BITS + self.slot] &= value;
    }

    pub(crate) fn set_bump_damage(&mut self, value: u8) {
        self.ram[SPRITE_BUMP_DAMAGE + self.slot] = value;
    }

    pub(crate) fn and_bump_damage(&mut self, value: u8) {
        self.ram[SPRITE_BUMP_DAMAGE + self.slot] &= value;
    }

    pub(crate) fn set_incoming_damage(&mut self, value: u8) {
        self.ram[SPRITE_INCOMING_DAMAGE + self.slot] = value;
    }

    pub(crate) fn set_n(&mut self, value: u8) {
        self.ram[SPRITE_N + self.slot] = value;
    }

    pub(crate) fn set_n_word(&mut self, value: u16) {
        write_le_u16(self.ram, SPRITE_N + self.slot * 2, value);
    }

    pub(crate) fn set_die_action(&mut self, value: u8) {
        self.ram[SPRITE_DIE_ACTION + self.slot] = value;
    }

    pub(crate) fn increment_die_action(&mut self) {
        self.ram[SPRITE_DIE_ACTION + self.slot] =
            self.ram[SPRITE_DIE_ACTION + self.slot].wrapping_add(1);
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] = value;
    }

    pub(crate) fn clear_object_priority_bits(&mut self, mask: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] &= !mask;
    }

    pub(crate) fn or_object_priority_bits(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
    }

    pub(crate) fn or_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] |= value;
    }

    pub(crate) fn and_object_priority(&mut self, value: u8) {
        self.ram[SPRITE_OBJ_PRIO + self.slot] &= value;
    }

    pub(crate) fn set_hit_timer(&mut self, value: u8) {
        self.ram[SPRITE_HIT_TIMER + self.slot] = value;
    }

    pub(crate) fn or_hit_timer(&mut self, value: u8) {
        self.ram[SPRITE_HIT_TIMER + self.slot] |= value;
    }

    pub(crate) fn set_anim_clock(&mut self, value: u8) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] = value;
    }

    pub(crate) fn add_anim_clock(&mut self, value: u8) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(value);
    }

    pub(crate) fn increment_anim_clock(&mut self) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_add(1);
    }

    pub(crate) fn decrement_anim_clock(&mut self) {
        self.ram[SPRITE_ANIM_CLOCK + self.slot] =
            self.ram[SPRITE_ANIM_CLOCK + self.slot].wrapping_sub(1);
    }

    pub(crate) fn set_wall_collision(&mut self, value: u8) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] = value;
    }

    pub(crate) fn or_wall_collision(&mut self, value: u8) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] |= value;
    }

    pub(crate) fn decrement_wall_collision(&mut self) {
        self.ram[SPRITE_WALL_COLLISION + self.slot] =
            self.ram[SPRITE_WALL_COLLISION + self.slot].wrapping_sub(1);
    }

    pub(crate) fn move_x(&mut self) {
        if self.ram[SPRITE_X_VELOCITY + self.slot] == 0 {
            return;
        }
        move_axis24(
            self.ram,
            SPRITE_X_SUBPIXEL + self.slot,
            SPRITE_X_LO + self.slot,
            SPRITE_X_HI + self.slot,
            SPRITE_X_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_y(&mut self) {
        if self.ram[SPRITE_Y_VELOCITY + self.slot] == 0 {
            return;
        }
        move_axis24(
            self.ram,
            SPRITE_Y_SUBPIXEL + self.slot,
            SPRITE_Y_LO + self.slot,
            SPRITE_Y_HI + self.slot,
            SPRITE_Y_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_z(&mut self) {
        move_axis16(
            self.ram,
            SPRITE_Z_SUBPIXEL + self.slot,
            SPRITE_Z + self.slot,
            SPRITE_Z_VELOCITY + self.slot,
        );
    }
}

pub(crate) struct AncillaSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> AncillaSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn slot(&self) -> u8 {
        self.slot as u8
    }

    pub(crate) fn ancilla_type(&self) -> u8 {
        byte(self.ram, ANCILLA_TYPE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.ancilla_type() != 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, ANCILLA_X_LO + self.slot, ANCILLA_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, ANCILLA_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, ANCILLA_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, ANCILLA_Y_LO + self.slot, ANCILLA_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_VELOCITY + self.slot)
    }

    pub(crate) fn z_velocity(&self) -> u8 {
        byte(self.ram, ANCILLA_Z_VELOCITY + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn z(&self) -> u8 {
        byte(self.ram, ANCILLA_Z + self.slot)
    }

    pub(crate) fn z_subpixel(&self) -> u8 {
        byte(self.ram, ANCILLA_Z_SUBPIXEL_PLAYER + self.slot)
    }

    pub(crate) fn item_to_link(&self) -> u8 {
        byte(self.ram, ANCILLA_ITEM_TO_LINK + self.slot)
    }

    pub(crate) fn timer(&self) -> u8 {
        byte(self.ram, ANCILLA_TIMER + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, ANCILLA_FLOOR + self.slot)
    }

    pub(crate) fn floor2(&self) -> u8 {
        byte(self.ram, ANCILLA_FLOOR2 + self.slot)
    }

    pub(crate) fn object_priority(&self) -> u8 {
        byte(self.ram, ANCILLA_OBJPRIO + self.slot)
    }

    pub(crate) fn u(&self) -> u8 {
        byte(self.ram, ANCILLA_U + self.slot)
    }

    pub(crate) fn num_sprites(&self) -> u8 {
        byte(self.ram, ANCILLA_NUMSPR + self.slot)
    }

    pub(crate) fn direction(&self) -> u8 {
        byte(self.ram, ANCILLA_DIRECTION + self.slot)
    }

    pub(crate) fn tile_attribute(&self) -> u8 {
        byte(self.ram, ANCILLA_TILE_ATTRIBUTE + self.slot)
    }

    pub(crate) fn step(&self) -> u8 {
        byte(self.ram, ANCILLA_STEP + self.slot)
    }

    pub(crate) fn aux_timer(&self) -> u8 {
        byte(self.ram, ANCILLA_AUX_TIMER + self.slot)
    }

    pub(crate) fn work_byte_3(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_3 + self.slot)
    }

    pub(crate) fn work_byte_1(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_1 + self.slot)
    }

    pub(crate) fn s_player(&self) -> u8 {
        byte(self.ram, ANCILLA_S_PLAYER + self.slot)
    }

    pub(crate) fn t_player(&self) -> u8 {
        byte(self.ram, ANCILLA_T_PLAYER + self.slot)
    }

    pub(crate) fn a(&self) -> u8 {
        byte(self.ram, ANCILLA_A + self.slot)
    }

    pub(crate) fn b(&self) -> u8 {
        byte(self.ram, ANCILLA_B + self.slot)
    }

    pub(crate) fn ab_word(&self) -> u16 {
        u16::from(self.a()) | (u16::from(self.b()) << 8)
    }

    /// Word read at `a`; the high byte aliases the next slot's `a` byte,
    /// matching the original WORD(ancilla_A(k)) access.
    pub(crate) fn a_word(&self) -> u16 {
        word(self.ram, ANCILLA_A + self.slot)
    }

    pub(crate) fn l(&self) -> u8 {
        byte(self.ram, ANCILLA_L + self.slot)
    }

    pub(crate) fn h(&self) -> u8 {
        byte(self.ram, ANCILLA_H + self.slot)
    }

    pub(crate) fn k(&self) -> u8 {
        byte(self.ram, ANCILLA_K + self.slot)
    }

    pub(crate) fn g(&self) -> u8 {
        byte(self.ram, ANCILLA_G + self.slot)
    }

    pub(crate) fn r(&self) -> u8 {
        byte(self.ram, ANCILLA_R + self.slot)
    }

    pub(crate) fn work_byte_22(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_22 + self.slot)
    }

    pub(crate) fn work_byte_23(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_23 + self.slot)
    }

    pub(crate) fn work_byte_24(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_24 + self.slot)
    }

    pub(crate) fn work_byte_4(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_4 + self.slot)
    }

    pub(crate) fn work_byte_25(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_25 + self.slot)
    }

    pub(crate) fn work_byte_26(&self) -> u8 {
        byte(self.ram, ANCILLA_WORK_BYTE_26 + self.slot)
    }
}

pub(crate) struct AncillaSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> AncillaSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_ancilla_type(&mut self, value: u8) {
        self.ram[ANCILLA_TYPE + self.slot] = value;
    }

    pub(crate) fn increment_ancilla_type(&mut self) -> u8 {
        self.ram[ANCILLA_TYPE + self.slot] = self.ram[ANCILLA_TYPE + self.slot].wrapping_add(1);
        self.ram[ANCILLA_TYPE + self.slot]
    }

    pub(crate) fn clear(&mut self) {
        self.set_ancilla_type(0);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            ANCILLA_X_LO + self.slot,
            ANCILLA_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[ANCILLA_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[ANCILLA_X_HI + self.slot] = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            ANCILLA_Y_LO + self.slot,
            ANCILLA_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[ANCILLA_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[ANCILLA_Y_HI + self.slot] = value;
    }

    pub(crate) fn set_x_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_X_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_y_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_Y_SUBPIXEL + self.slot] = value;
    }

    pub(crate) fn set_x_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_X_VELOCITY + self.slot] = value;
    }

    pub(crate) fn set_y_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] = value;
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        self.ram[ANCILLA_X_VELOCITY + self.slot]
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        self.ram[ANCILLA_Y_VELOCITY + self.slot]
    }

    pub(crate) fn add_x_velocity(&mut self, value: u8) -> u8 {
        let velocity = self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_add(value);
        self.set_x_velocity(velocity);
        velocity
    }

    pub(crate) fn add_y_velocity(&mut self, value: u8) -> u8 {
        let velocity = self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_add(value);
        self.set_y_velocity(velocity);
        velocity
    }

    pub(crate) fn subtract_y_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] =
            self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_sub(value);
    }

    pub(crate) fn negate_x_velocity(&mut self) {
        self.ram[ANCILLA_X_VELOCITY + self.slot] =
            self.ram[ANCILLA_X_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn negate_y_velocity(&mut self) {
        self.ram[ANCILLA_Y_VELOCITY + self.slot] =
            self.ram[ANCILLA_Y_VELOCITY + self.slot].wrapping_neg();
    }

    pub(crate) fn set_z_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Z_VELOCITY + self.slot] = value;
    }

    pub(crate) fn add_z_velocity(&mut self, value: u8) {
        self.ram[ANCILLA_Z_VELOCITY + self.slot] =
            self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_z_velocity(&mut self) -> u8 {
        let value = self.ram[ANCILLA_Z_VELOCITY + self.slot].wrapping_sub(1);
        self.set_z_velocity(value);
        value
    }

    pub(crate) fn set_z(&mut self, value: u8) {
        self.ram[ANCILLA_Z + self.slot] = value;
    }

    pub(crate) fn set_z_subpixel(&mut self, value: u8) {
        self.ram[ANCILLA_Z_SUBPIXEL_PLAYER + self.slot] = value;
    }

    pub(crate) fn move_x(&mut self) {
        move_axis24(
            self.ram,
            ANCILLA_X_SUBPIXEL + self.slot,
            ANCILLA_X_LO + self.slot,
            ANCILLA_X_HI + self.slot,
            ANCILLA_X_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_y(&mut self) {
        move_axis24(
            self.ram,
            ANCILLA_Y_SUBPIXEL + self.slot,
            ANCILLA_Y_LO + self.slot,
            ANCILLA_Y_HI + self.slot,
            ANCILLA_Y_VELOCITY + self.slot,
        );
    }

    pub(crate) fn move_z(&mut self) {
        move_axis16(
            self.ram,
            ANCILLA_Z_SUBPIXEL_PLAYER + self.slot,
            ANCILLA_Z + self.slot,
            ANCILLA_Z_VELOCITY + self.slot,
        );
    }

    pub(crate) fn set_item_to_link(&mut self, value: u8) {
        self.ram[ANCILLA_ITEM_TO_LINK + self.slot] = value;
    }

    pub(crate) fn advance_item_to_link(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(1);
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn add_item_to_link(&mut self, value: u8) {
        self.ram[ANCILLA_ITEM_TO_LINK + self.slot] =
            self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_item_to_link(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot].wrapping_sub(1);
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn toggle_item_to_link_bit0(&mut self) -> u8 {
        let value = self.ram[ANCILLA_ITEM_TO_LINK + self.slot] ^ 1;
        self.set_item_to_link(value);
        value
    }

    pub(crate) fn set_timer(&mut self, value: u8) {
        self.ram[ANCILLA_TIMER + self.slot] = value;
    }

    pub(crate) fn tick_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_TIMER + self.slot].wrapping_sub(1);
        self.set_timer(value);
        value
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[ANCILLA_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_floor2(&mut self, value: u8) {
        self.ram[ANCILLA_FLOOR2 + self.slot] = value;
    }

    pub(crate) fn set_oam_index(&mut self, value: u8) {
        self.ram[ANCILLA_OAM_IDX + self.slot] = value;
    }

    pub(crate) fn set_num_sprites(&mut self, value: u8) {
        self.ram[ANCILLA_NUMSPR + self.slot] = value;
    }

    pub(crate) fn set_object_priority(&mut self, value: u8) {
        self.ram[ANCILLA_OBJPRIO + self.slot] = value;
    }

    pub(crate) fn xor_object_priority(&mut self, value: u8) {
        self.ram[ANCILLA_OBJPRIO + self.slot] ^= value;
    }

    pub(crate) fn set_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] = value;
    }

    pub(crate) fn or_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] |= value;
    }

    pub(crate) fn and_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] &= value;
    }

    pub(crate) fn add_direction(&mut self, value: u8) {
        self.ram[ANCILLA_DIRECTION + self.slot] =
            self.ram[ANCILLA_DIRECTION + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_tile_attribute(&mut self, value: u8) {
        self.ram[ANCILLA_TILE_ATTRIBUTE + self.slot] = value;
    }

    pub(crate) fn set_step(&mut self, value: u8) {
        self.ram[ANCILLA_STEP + self.slot] = value;
    }

    pub(crate) fn advance_step(&mut self) -> u8 {
        let value = self.ram[ANCILLA_STEP + self.slot].wrapping_add(1);
        self.set_step(value);
        value
    }

    pub(crate) fn add_step(&mut self, value: u8) {
        self.ram[ANCILLA_STEP + self.slot] = self.ram[ANCILLA_STEP + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_step(&mut self) -> u8 {
        let value = self.ram[ANCILLA_STEP + self.slot].wrapping_sub(1);
        self.set_step(value);
        value
    }

    pub(crate) fn set_aux_timer(&mut self, value: u8) {
        self.ram[ANCILLA_AUX_TIMER + self.slot] = value;
    }

    pub(crate) fn advance_aux_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(1);
        self.set_aux_timer(value);
        value
    }

    pub(crate) fn add_aux_timer(&mut self, value: u8) {
        self.ram[ANCILLA_AUX_TIMER + self.slot] =
            self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_aux_timer(&mut self) -> u8 {
        let value = self.ram[ANCILLA_AUX_TIMER + self.slot].wrapping_sub(1);
        self.set_aux_timer(value);
        value
    }

    pub(crate) fn set_work_byte_3(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_3 + self.slot] = value;
    }

    pub(crate) fn add_work_byte_3(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_3 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_1(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_1 + self.slot] = value;
    }

    pub(crate) fn subtract_work_byte_1(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_1 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_a(&mut self, value: u8) {
        self.ram[ANCILLA_A + self.slot] = value;
    }

    pub(crate) fn advance_a(&mut self) -> u8 {
        let value = self.ram[ANCILLA_A + self.slot].wrapping_add(1);
        self.set_a(value);
        value
    }

    /// Word write at `a`; the high byte aliases the next slot's `a` byte,
    /// matching the original WORD(ancilla_A(k)) access.
    pub(crate) fn set_a_word(&mut self, value: u16) {
        write_le_u16(self.ram, ANCILLA_A + self.slot, value);
    }

    pub(crate) fn set_b(&mut self, value: u8) {
        self.ram[ANCILLA_B + self.slot] = value;
    }

    pub(crate) fn set_l(&mut self, value: u8) {
        self.ram[ANCILLA_L + self.slot] = value;
    }

    pub(crate) fn advance_l(&mut self) -> u8 {
        let value = self.ram[ANCILLA_L + self.slot].wrapping_add(1);
        self.set_l(value);
        value
    }

    pub(crate) fn add_l(&mut self, value: u8) {
        self.ram[ANCILLA_L + self.slot] = self.ram[ANCILLA_L + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_l(&mut self) -> u8 {
        let value = self.ram[ANCILLA_L + self.slot].wrapping_sub(1);
        self.set_l(value);
        value
    }

    pub(crate) fn set_h(&mut self, value: u8) {
        self.ram[ANCILLA_H + self.slot] = value;
    }

    pub(crate) fn set_k(&mut self, value: u8) {
        self.ram[ANCILLA_K + self.slot] = value;
    }

    pub(crate) fn toggle_k_bit0(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot] ^ 1;
        self.set_k(value);
        value
    }

    pub(crate) fn advance_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_add(1);
        self.set_k(value);
        value
    }

    pub(crate) fn add_k(&mut self, value: u8) {
        self.ram[ANCILLA_K + self.slot] = self.ram[ANCILLA_K + self.slot].wrapping_add(value);
    }

    pub(crate) fn retreat_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
        self.set_k(value);
        value
    }

    pub(crate) fn tick_k(&mut self) -> u8 {
        let value = self.ram[ANCILLA_K + self.slot].wrapping_sub(1);
        self.set_k(value);
        value
    }

    pub(crate) fn set_g(&mut self, value: u8) {
        self.ram[ANCILLA_G + self.slot] = value;
    }

    pub(crate) fn subtract_g(&mut self, value: u8) {
        self.ram[ANCILLA_G + self.slot] = self.ram[ANCILLA_G + self.slot].wrapping_sub(value);
    }

    pub(crate) fn tick_g(&mut self) -> u8 {
        let value = self.ram[ANCILLA_G + self.slot].wrapping_sub(1);
        self.set_g(value);
        value
    }

    pub(crate) fn set_s_player(&mut self, value: u8) {
        self.ram[ANCILLA_S_PLAYER + self.slot] = value;
    }

    pub(crate) fn set_t_player(&mut self, value: u8) {
        self.ram[ANCILLA_T_PLAYER + self.slot] = value;
    }

    pub(crate) fn set_r(&mut self, value: u8) {
        self.ram[ANCILLA_R + self.slot] = value;
    }

    pub(crate) fn advance_r(&mut self) -> u8 {
        let value = self.ram[ANCILLA_R + self.slot].wrapping_add(1);
        self.set_r(value);
        value
    }

    pub(crate) fn add_r(&mut self, value: u8) {
        self.ram[ANCILLA_R + self.slot] = self.ram[ANCILLA_R + self.slot].wrapping_add(value);
    }

    pub(crate) fn tick_s_player(&mut self) -> u8 {
        let value = self.ram[ANCILLA_S_PLAYER + self.slot].wrapping_sub(1);
        self.set_s_player(value);
        value
    }

    pub(crate) fn set_u(&mut self, value: u8) {
        self.ram[ANCILLA_U + self.slot] = value;
    }

    pub(crate) fn subtract_u(&mut self, value: u8) {
        self.ram[ANCILLA_U + self.slot] = self.ram[ANCILLA_U + self.slot].wrapping_sub(value);
    }

    pub(crate) fn advance_work_byte_1_mod4(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(1) & 3;
        self.set_work_byte_1(value);
        value
    }

    pub(crate) fn add_work_byte_1_mod4(&mut self, value: u8) -> u8 {
        let next = self.ram[ANCILLA_WORK_BYTE_1 + self.slot].wrapping_add(value) & 3;
        self.set_work_byte_1(next);
        next
    }

    pub(crate) fn tick_work_byte_3(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_sub(1);
        self.set_work_byte_3(value);
        value
    }

    pub(crate) fn advance_work_byte_3(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_3 + self.slot].wrapping_add(1);
        self.set_work_byte_3(value);
        value
    }

    pub(crate) fn set_work_byte_4(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_4 + self.slot] = value;
    }

    pub(crate) fn subtract_work_byte_4(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_4 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_work_byte_22(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_22 + self.slot] = value;
    }

    pub(crate) fn tick_work_byte_22(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(1);
        self.set_work_byte_22(value);
        value
    }

    pub(crate) fn subtract_work_byte_22(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_22 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_22 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_work_byte_23(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_23 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_23(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(1);
        self.set_work_byte_23(value);
        value
    }

    pub(crate) fn add_work_byte_23(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_23 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_23 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_24(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_24 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_24(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(1);
        self.set_work_byte_24(value);
        value
    }

    pub(crate) fn add_work_byte_24(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_24 + self.slot] =
            self.ram[ANCILLA_WORK_BYTE_24 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_work_byte_25(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_25 + self.slot] = value;
    }

    pub(crate) fn set_work_byte_26(&mut self, value: u8) {
        self.ram[ANCILLA_WORK_BYTE_26 + self.slot] = value;
    }

    pub(crate) fn advance_work_byte_25(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_add(1);
        self.set_work_byte_25(value);
        value
    }

    pub(crate) fn retreat_work_byte_25(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_25 + self.slot].wrapping_sub(1);
        self.set_work_byte_25(value);
        value
    }

    pub(crate) fn advance_work_byte_4(&mut self) -> u8 {
        let value = self.ram[ANCILLA_WORK_BYTE_4 + self.slot].wrapping_add(1);
        self.set_work_byte_4(value);
        value
    }
}

pub(crate) struct OverlordSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> OverlordSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_X_LO + self.slot,
            OVERLORD_X_HI + self.slot,
        )
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_Y_LO + self.slot,
            OVERLORD_Y_HI + self.slot,
        )
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn adjacent_x_low_word(&self) -> u16 {
        word(self.ram, OVERLORD_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, OVERLORD_X_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, OVERLORD_Y_HI + self.slot)
    }

    pub(crate) fn overlord_type(&self) -> u8 {
        byte(self.ram, OVERLORD_TYPE + self.slot)
    }

    pub(crate) fn is_active(&self) -> bool {
        self.overlord_type() != 0
    }

    pub(crate) fn gen1(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN1 + self.slot)
    }

    pub(crate) fn gen1_word(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN1 + self.slot,
            OVERLORD_GEN1 + self.slot + 1,
        )
    }

    pub(crate) fn gen2(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN2 + self.slot)
    }

    pub(crate) fn gen2_word(&self) -> u16 {
        packed_position(
            self.ram,
            OVERLORD_GEN2 + self.slot,
            OVERLORD_GEN2 + self.slot + 1,
        )
    }

    pub(crate) fn gen3(&self) -> u8 {
        byte(self.ram, OVERLORD_GEN3 + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, OVERLORD_FLOOR + self.slot)
    }

    pub(crate) fn spawned_area(&self) -> u8 {
        byte(self.ram, OVERLORD_SPAWNED_AREA + self.slot)
    }

    pub(crate) fn sprite_block_pos(&self) -> u16 {
        word(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2)
    }
}

pub(crate) struct OverlordSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> OverlordSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            OVERLORD_X_LO + self.slot,
            OVERLORD_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] = value;
    }

    pub(crate) fn set_adjacent_x_low_word(&mut self, value: u16) {
        write_le_u16(self.ram, OVERLORD_X_LO + self.slot, value);
    }

    pub(crate) fn subtract_adjacent_x_low_word(&mut self, value: u16) -> u16 {
        let updated = word(self.ram, OVERLORD_X_LO + self.slot).wrapping_sub(value);
        self.set_adjacent_x_low_word(updated);
        updated
    }

    pub(crate) fn x_low(&self) -> u8 {
        self.ram[OVERLORD_X_LO + self.slot]
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[OVERLORD_X_HI + self.slot] = value;
    }

    pub(crate) fn increment_x_high(&mut self) {
        self.ram[OVERLORD_X_HI + self.slot] = self.ram[OVERLORD_X_HI + self.slot].wrapping_add(1);
    }

    pub(crate) fn add_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] =
            self.ram[OVERLORD_X_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_circle_x(&mut self, value: u16) {
        self.ram[OVERLORD_X_HI + self.slot] = value as u8;
        self.ram[OVERLORD_Y_HI + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_circle_y(&mut self, value: u16) {
        self.ram[OVERLORD_GEN2 + self.slot] = value as u8;
        self.ram[OVERLORD_FLOOR + self.slot] = (value >> 8) as u8;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            OVERLORD_Y_LO + self.slot,
            OVERLORD_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[OVERLORD_Y_LO + self.slot] = value;
    }

    pub(crate) fn set_y_high(&mut self, value: u8) {
        self.ram[OVERLORD_Y_HI + self.slot] = value;
    }

    pub(crate) fn subtract_x_low(&mut self, value: u8) {
        self.ram[OVERLORD_X_LO + self.slot] =
            self.ram[OVERLORD_X_LO + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_overlord_type(&mut self, value: u8) {
        self.ram[OVERLORD_TYPE + self.slot] = value;
    }

    pub(crate) fn clear(&mut self) {
        self.set_overlord_type(0);
    }

    pub(crate) fn set_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] = value;
    }

    pub(crate) fn add_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] =
            self.ram[OVERLORD_GEN1 + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_gen1_word(&mut self, value: u16) {
        let next = read_le_u16(self.ram, OVERLORD_GEN1 + self.slot).wrapping_add(value);
        write_le_u16(self.ram, OVERLORD_GEN1 + self.slot, next);
    }

    pub(crate) fn subtract_gen1(&mut self, value: u8) {
        self.ram[OVERLORD_GEN1 + self.slot] =
            self.ram[OVERLORD_GEN1 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] = value;
    }

    pub(crate) fn add_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] =
            self.ram[OVERLORD_GEN2 + self.slot].wrapping_add(value);
    }

    pub(crate) fn add_gen2_word(&mut self, value: u16) {
        let next = read_le_u16(self.ram, OVERLORD_GEN2 + self.slot).wrapping_add(value);
        write_le_u16(self.ram, OVERLORD_GEN2 + self.slot, next);
    }

    pub(crate) fn subtract_gen2(&mut self, value: u8) {
        self.ram[OVERLORD_GEN2 + self.slot] =
            self.ram[OVERLORD_GEN2 + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_gen3(&mut self, value: u8) {
        self.ram[OVERLORD_GEN3 + self.slot] = value;
    }

    pub(crate) fn add_gen3(&mut self, value: u8) {
        self.ram[OVERLORD_GEN3 + self.slot] =
            self.ram[OVERLORD_GEN3 + self.slot].wrapping_add(value);
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[OVERLORD_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_sprite_block_pos(&mut self, value: u16) {
        write_le_u16(self.ram, OVERLORD_OFFSET_SPRITE_POS + self.slot * 2, value);
    }

    pub(crate) fn set_spawned_area(&mut self, value: u8) {
        self.ram[OVERLORD_SPAWNED_AREA + self.slot] = value;
    }
}

pub(crate) struct OverworldSpritePresenceView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldSpritePresenceView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn marker(&self, index: usize) -> u8 {
        byte(self.ram, OVERWORLD_SPRITE_PRESENCE + index)
    }
}

pub(crate) struct OverworldSpritePresenceViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldSpritePresenceViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_marker(&mut self, index: usize, value: u8) {
        self.ram[OVERWORLD_SPRITE_PRESENCE + index] = value;
    }
}

pub(crate) struct OverworldSpriteLoadedView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldSpriteLoadedView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn is_loaded(&self, block: u16, loaded_mask: u8) -> bool {
        byte(
            self.ram,
            OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3),
        ) & loaded_mask
            != 0
    }
}

pub(crate) struct OverworldSpriteLoadedViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldSpriteLoadedViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.ram[OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)] &= !loaded_mask;
    }

    /// Same as `clear_loaded_mask`, but wraps the byte index to the low
    /// 128 KiB of RAM exactly like the original code did.
    pub(crate) fn clear_loaded_mask_wrapped(&mut self, block: u16, loaded_mask: u8) {
        self.ram[(OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)) & 0x1ffff] &= !loaded_mask;
    }

    pub(crate) fn set_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.ram[OVERWORLD_SPRITE_WAS_LOADED + usize::from(block >> 3)] |= loaded_mask;
    }

    pub(crate) fn clear_all(&mut self) {
        self.ram[OVERWORLD_SPRITE_WAS_LOADED..OVERWORLD_SPRITE_WAS_LOADED + 0x200].fill(0);
    }
}

pub(crate) struct GarnishSlotView<'a> {
    ram: &'a [u8],
    slot: usize,
}

impl<'a> GarnishSlotView<'a> {
    pub(crate) fn new(ram: &'a [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn garnish_type(&self) -> u8 {
        byte(self.ram, GARNISH_TYPE + self.slot)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.garnish_type() == 0
    }

    pub(crate) fn x(&self) -> u16 {
        packed_position(self.ram, GARNISH_X_LO + self.slot, GARNISH_X_HI + self.slot)
    }

    pub(crate) fn x_low(&self) -> u8 {
        byte(self.ram, GARNISH_X_LO + self.slot)
    }

    pub(crate) fn x_high(&self) -> u8 {
        byte(self.ram, GARNISH_X_HI + self.slot)
    }

    pub(crate) fn y(&self) -> u16 {
        packed_position(self.ram, GARNISH_Y_LO + self.slot, GARNISH_Y_HI + self.slot)
    }

    pub(crate) fn y_low(&self) -> u8 {
        byte(self.ram, GARNISH_Y_LO + self.slot)
    }

    pub(crate) fn y_high(&self) -> u8 {
        byte(self.ram, GARNISH_Y_HI + self.slot)
    }

    pub(crate) fn x_velocity(&self) -> u8 {
        byte(self.ram, GARNISH_X_VELOCITY + self.slot)
    }

    pub(crate) fn y_velocity(&self) -> u8 {
        byte(self.ram, GARNISH_Y_VELOCITY + self.slot)
    }

    pub(crate) fn x_subpixel(&self) -> u8 {
        byte(self.ram, GARNISH_X_SUBPIXEL + self.slot)
    }

    pub(crate) fn y_subpixel(&self) -> u8 {
        byte(self.ram, GARNISH_Y_SUBPIXEL + self.slot)
    }

    pub(crate) fn countdown(&self) -> u8 {
        byte(self.ram, GARNISH_COUNTDOWN + self.slot)
    }

    pub(crate) fn sprite(&self) -> u8 {
        byte(self.ram, GARNISH_SPRITE + self.slot)
    }

    pub(crate) fn floor(&self) -> u8 {
        byte(self.ram, GARNISH_FLOOR + self.slot)
    }

    pub(crate) fn oam_flags(&self) -> u8 {
        byte(self.ram, GARNISH_OAM_FLAGS + self.slot)
    }
}

pub(crate) struct GarnishSlotViewMut<'a> {
    ram: &'a mut [u8],
    slot: usize,
}

impl<'a> GarnishSlotViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8], slot: usize) -> Self {
        Self { ram, slot }
    }

    pub(crate) fn set_garnish_type(&mut self, value: u8) {
        self.ram[GARNISH_TYPE + self.slot] = value;
    }

    pub(crate) fn clear(&mut self) {
        self.set_garnish_type(0);
    }

    pub(crate) fn set_x(&mut self, value: u16) {
        write_position(
            self.ram,
            GARNISH_X_LO + self.slot,
            GARNISH_X_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_x_low(&mut self, value: u8) {
        self.ram[GARNISH_X_LO + self.slot] = value;
    }

    pub(crate) fn set_x_high(&mut self, value: u8) {
        self.ram[GARNISH_X_HI + self.slot] = value;
    }

    pub(crate) fn set_y(&mut self, value: u16) {
        write_position(
            self.ram,
            GARNISH_Y_LO + self.slot,
            GARNISH_Y_HI + self.slot,
            value,
        );
    }

    pub(crate) fn set_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = value;
    }

    pub(crate) fn add_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = self.ram[GARNISH_Y_LO + self.slot].wrapping_add(value);
    }

    pub(crate) fn subtract_y_low(&mut self, value: u8) {
        self.ram[GARNISH_Y_LO + self.slot] = self.ram[GARNISH_Y_LO + self.slot].wrapping_sub(value);
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

    pub(crate) fn add_y_velocity(&mut self, value: u8) {
        self.ram[GARNISH_Y_VELOCITY + self.slot] =
            self.ram[GARNISH_Y_VELOCITY + self.slot].wrapping_add(value);
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

    pub(crate) fn subtract_countdown(&mut self, value: u8) {
        self.ram[GARNISH_COUNTDOWN + self.slot] =
            self.ram[GARNISH_COUNTDOWN + self.slot].wrapping_sub(value);
    }

    pub(crate) fn set_sprite(&mut self, value: u8) {
        self.ram[GARNISH_SPRITE + self.slot] = value;
    }

    pub(crate) fn set_floor(&mut self, value: u8) {
        self.ram[GARNISH_FLOOR + self.slot] = value;
    }

    pub(crate) fn set_oam_flags(&mut self, value: u8) {
        self.ram[GARNISH_OAM_FLAGS + self.slot] = value;
    }
}

pub(crate) struct GarnishStateView<'a> {
    ram: &'a [u8],
}

impl<'a> GarnishStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn active_type(&self) -> u8 {
        byte(self.ram, GARNISH_ACTIVE)
    }

    pub(crate) fn boulder_trap_count(&self) -> u8 {
        byte(self.ram, OVERWORLD_BOULDER_TRAP_COUNT)
    }

    pub(crate) fn boulder_trap_timer(&self) -> u8 {
        byte(self.ram, OVERWORLD_BOULDER_TRAP_TIMER)
    }

    pub(crate) fn sprcoll_y_hi(&self) -> u8 {
        byte(self.ram, SPRCOLL_Y_BASE + 1)
    }

    pub(crate) fn sprcoll_x_word(&self) -> u16 {
        word(self.ram, SPRCOLL_X_BASE)
    }

    pub(crate) fn sprcoll_y_word(&self) -> u16 {
        word(self.ram, SPRCOLL_Y_BASE)
    }

    pub(crate) fn active_overlord_index(&self) -> u8 {
        byte(self.ram, ACTIVE_OVERLORD_INDEX)
    }

    pub(crate) fn haunted_grove_flute_event_latch(&self) -> u8 {
        byte(self.ram, HAUNTED_GROVE_FLUTE_EVENT_LATCH)
    }

    pub(crate) fn repulsespark_timer(&self) -> u8 {
        byte(self.ram, REPULSESPARK_TIMER)
    }

    pub(crate) fn repulsespark_anim_delay(&self) -> u8 {
        byte(self.ram, REPULSESPARK_ANIM_DELAY)
    }

    pub(crate) fn repulsespark_floor_status(&self) -> u8 {
        byte(self.ram, REPULSESPARK_FLOOR_STATUS)
    }

    pub(crate) fn repulsespark_x_lo(&self) -> u8 {
        byte(self.ram, REPULSESPARK_X_LO)
    }

    pub(crate) fn repulsespark_y_lo(&self) -> u8 {
        byte(self.ram, REPULSESPARK_Y_LO)
    }

    pub(crate) fn sprcoll_x_size(&self) -> u16 {
        word(self.ram, SPRCOLL_X_SIZE)
    }

    pub(crate) fn sprcoll_y_size(&self) -> u16 {
        word(self.ram, SPRCOLL_Y_SIZE)
    }
}

pub(crate) struct GarnishStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> GarnishStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_active_type(&mut self, value: u8) {
        self.ram[GARNISH_ACTIVE] = value;
    }

    pub(crate) fn clear_active_type(&mut self) {
        self.ram[GARNISH_ACTIVE] = 0;
    }

    pub(crate) fn increment_boulder_trap_timer(&mut self) -> u8 {
        let val = self.ram[OVERWORLD_BOULDER_TRAP_TIMER].wrapping_add(1);
        self.ram[OVERWORLD_BOULDER_TRAP_TIMER] = val;
        val
    }

    pub(crate) fn set_active_overlord_index(&mut self, value: u8) {
        self.ram[ACTIVE_OVERLORD_INDEX] = value;
    }

    pub(crate) fn increment_haunted_grove_flute_event_latch(&mut self) {
        self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] =
            self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH].wrapping_add(1);
    }

    pub(crate) fn set_repulsespark_timer(&mut self, value: u8) {
        self.ram[REPULSESPARK_TIMER] = value;
    }

    pub(crate) fn clear_repulsespark_timer(&mut self) {
        self.ram[REPULSESPARK_TIMER] = 0;
    }

    pub(crate) fn decrement_repulsespark_timer(&mut self) {
        self.ram[REPULSESPARK_TIMER] = self.ram[REPULSESPARK_TIMER].wrapping_sub(1);
    }

    pub(crate) fn set_repulsespark_anim_delay(&mut self, value: u8) {
        self.ram[REPULSESPARK_ANIM_DELAY] = value;
    }

    pub(crate) fn decrement_repulsespark_anim_delay(&mut self) -> u8 {
        let value = self.ram[REPULSESPARK_ANIM_DELAY].wrapping_sub(1);
        self.ram[REPULSESPARK_ANIM_DELAY] = value;
        value
    }

    pub(crate) fn set_repulsespark_x_lo(&mut self, value: u8) {
        self.ram[REPULSESPARK_X_LO] = value;
    }

    pub(crate) fn set_repulsespark_y_lo(&mut self, value: u8) {
        self.ram[REPULSESPARK_Y_LO] = value;
    }

    pub(crate) fn set_sprcoll_x_size(&mut self, value: u16) {
        write_le_u16(self.ram, SPRCOLL_X_SIZE, value);
    }

    pub(crate) fn set_sprcoll_y_size(&mut self, value: u16) {
        write_le_u16(self.ram, SPRCOLL_Y_SIZE, value);
    }

    pub(crate) fn set_sprcoll_x_base(&mut self, value: u16) {
        write_le_u16(self.ram, SPRCOLL_X_BASE, value);
    }

    pub(crate) fn set_sprcoll_y_base(&mut self, value: u16) {
        write_le_u16(self.ram, SPRCOLL_Y_BASE, value);
    }

    pub(crate) fn set_repulsespark_floor_status(&mut self, value: u8) {
        self.ram[REPULSESPARK_FLOOR_STATUS] = value;
    }

    pub(crate) fn clear_boulder_trap_count(&mut self) {
        self.ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0;
    }

    pub(crate) fn increment_boulder_trap_count(&mut self) {
        self.ram[OVERWORLD_BOULDER_TRAP_COUNT] =
            self.ram[OVERWORLD_BOULDER_TRAP_COUNT].wrapping_add(1);
    }

    pub(crate) fn clear_haunted_grove_flute_event_latch(&mut self) {
        self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 0;
    }
}
