use super::*;

/// Tile-revert memory used when secrets/liftable tiles are uncovered:
/// parallel arrays of tilemap addresses and original tile values, plus a
/// running byte count.
pub(crate) struct MemorizedTileView<'a> {
    ram: &'a [u8],
}

impl<'a> MemorizedTileView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn count(&self) -> u16 {
        word(self.ram, NUM_MEMORIZED_TILES)
    }

    pub(crate) fn entry_addr(&self, index: usize) -> u16 {
        word(self.ram, MEMORIZED_TILE_ADDR + index * 2)
    }

    pub(crate) fn entry_value(&self, index: usize) -> u16 {
        word(self.ram, MEMORIZED_TILE_VALUE + index * 2)
    }
}

pub(crate) struct MemorizedTileViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MemorizedTileViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_count(&mut self, value: u16) {
        write_le_u16(self.ram, NUM_MEMORIZED_TILES, value);
    }

    pub(crate) fn clear_count(&mut self) {
        write_le_u16(self.ram, NUM_MEMORIZED_TILES, 0);
    }

    pub(crate) fn set_entry_addr(&mut self, offset: usize, pos: u16) {
        write_le_u16(self.ram, MEMORIZED_TILE_ADDR + offset, pos);
    }

    pub(crate) fn set_entry_value(&mut self, offset: usize, tile: u16) {
        write_le_u16(self.ram, MEMORIZED_TILE_VALUE + offset, tile);
    }

    pub(crate) fn append_entry(&mut self, pos: u16, tile: u16) {
        let offset = word(self.ram, NUM_MEMORIZED_TILES) as usize;
        self.set_entry_value(offset, tile);
        self.set_entry_addr(offset, pos);
        self.set_count(offset as u16 + 2);
    }
}

pub(crate) struct MinigameStateView<'a> {
    ram: &'a [u8],
}

impl<'a> MinigameStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn is_archer_or_shovel_game(&self) -> u8 {
        byte(self.ram, IS_ARCHER_OR_SHOVEL_GAME)
    }

    pub(crate) fn credits(&self) -> u8 {
        byte(self.ram, MINIGAME_CREDITS)
    }

    pub(crate) fn flag_boomerang_in_place(&self) -> u8 {
        byte(self.ram, FLAG_FOR_BOOMERANG_IN_PLACE)
    }

    pub(crate) fn boomerang_temp_x(&self) -> u16 {
        word(self.ram, BOOMERANG_TEMP_X)
    }

    pub(crate) fn boomerang_temp_y(&self) -> u16 {
        word(self.ram, BOOMERANG_TEMP_Y)
    }
}

pub(crate) struct MinigameStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> MinigameStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_is_archer_or_shovel_game(&mut self, value: u8) {
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = value;
    }

    pub(crate) fn clear_is_archer_or_shovel_game(&mut self) {
        self.ram[IS_ARCHER_OR_SHOVEL_GAME] = 0;
    }

    pub(crate) fn set_credits(&mut self, value: u8) {
        self.ram[MINIGAME_CREDITS] = value;
    }

    pub(crate) fn clear_flag_boomerang_in_place(&mut self) {
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 0;
    }

    pub(crate) fn set_flag_boomerang_in_place(&mut self, value: u8) {
        self.ram[FLAG_FOR_BOOMERANG_IN_PLACE] = value;
    }

    pub(crate) fn set_boomerang_temp_x(&mut self, value: u16) {
        write_le_u16(self.ram, BOOMERANG_TEMP_X, value);
    }

    pub(crate) fn set_boomerang_temp_y(&mut self, value: u16) {
        write_le_u16(self.ram, BOOMERANG_TEMP_Y, value);
    }
}

pub(crate) struct SpriteBattleView<'a> {
    ram: &'a [u8],
}

impl<'a> SpriteBattleView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn sprites_killed(&self) -> u8 {
        byte(self.ram, NUM_SPRITES_KILLED)
    }

    pub(crate) fn times_hurt_by_sprites(&self) -> u8 {
        byte(self.ram, TIMES_HURT_BY_SPRITES)
    }

    pub(crate) fn item_drop_luck(&self) -> u8 {
        byte(self.ram, ITEM_DROP_LUCK)
    }

    pub(crate) fn luck_kill_counter(&self) -> u8 {
        byte(self.ram, LUCK_KILL_COUNTER)
    }

    pub(crate) fn item_drop_counter(&self) -> u8 {
        byte(self.ram, ITEM_DROP_COUNTER)
    }

    pub(crate) fn damage_type_determiner(&self) -> u8 {
        byte(self.ram, DAMAGE_TYPE_DETERMINER)
    }

    pub(crate) fn damaging_enemies_timer(&self) -> u8 {
        byte(self.ram, SET_WHEN_DAMAGING_ENEMIES)
    }
}

pub(crate) struct SpriteBattleViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> SpriteBattleViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_sprites_killed(&mut self) {
        self.ram[NUM_SPRITES_KILLED] = 0;
    }

    pub(crate) fn clear_times_hurt_by_sprites(&mut self) {
        self.ram[TIMES_HURT_BY_SPRITES] = 0;
    }

    pub(crate) fn increment_times_hurt_by_sprites(&mut self) {
        self.ram[TIMES_HURT_BY_SPRITES] = self.ram[TIMES_HURT_BY_SPRITES].wrapping_add(1);
    }

    pub(crate) fn set_item_drop_luck(&mut self, value: u8) {
        self.ram[ITEM_DROP_LUCK] = value;
    }

    pub(crate) fn clear_luck_kill_counter(&mut self) {
        self.ram[LUCK_KILL_COUNTER] = 0;
    }

    pub(crate) fn clear_item_drop_counter(&mut self) {
        self.ram[ITEM_DROP_COUNTER] = 0;
    }

    pub(crate) fn increment_sprites_killed(&mut self) {
        self.ram[NUM_SPRITES_KILLED] = self.ram[NUM_SPRITES_KILLED].wrapping_add(1);
    }

    pub(crate) fn increment_luck_kill_counter(&mut self) {
        self.ram[LUCK_KILL_COUNTER] = self.ram[LUCK_KILL_COUNTER].wrapping_add(1);
    }

    pub(crate) fn set_damage_type_determiner(&mut self, value: u8) {
        self.ram[DAMAGE_TYPE_DETERMINER] = value;
    }

    pub(crate) fn set_damaging_enemies_timer(&mut self, value: u8) {
        self.ram[SET_WHEN_DAMAGING_ENEMIES] = value;
    }

    pub(crate) fn clear_damaging_enemies_timer(&mut self) {
        self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
    }

    pub(crate) fn tick_damaging_enemies_timer(&mut self) {
        if self.ram[SET_WHEN_DAMAGING_ENEMIES] & 0x7f != 0 {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] =
                self.ram[SET_WHEN_DAMAGING_ENEMIES].wrapping_sub(1);
        } else {
            self.ram[SET_WHEN_DAMAGING_ENEMIES] = 0;
        }
    }

    pub(crate) fn increment_item_drop_counter(&mut self) -> u8 {
        let v = self.ram[ITEM_DROP_COUNTER].wrapping_add(1);
        self.ram[ITEM_DROP_COUNTER] = v;
        v
    }
}

pub(crate) struct IntroSwordView<'a> {
    ram: &'a [u8],
}

impl<'a> IntroSwordView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn ypos(&self) -> u16 {
        word(self.ram, INTRO_SWORD_YPOS)
    }

    pub(crate) fn sparkle_timer(&self) -> u8 {
        byte(self.ram, INTRO_SWORD_SPARKLE_TIMER)
    }

    pub(crate) fn sparkle_step(&self) -> u8 {
        byte(self.ram, INTRO_SWORD_SPARKLE_STEP)
    }

    pub(crate) fn anim_phase(&self) -> u8 {
        byte(self.ram, INTRO_SWORD_ANIM_STEP) >> 1
    }

    pub(crate) fn anim_step_raw(&self) -> u8 {
        byte(self.ram, INTRO_SWORD_ANIM_STEP)
    }

    pub(crate) fn sparkle_y_offset(&self) -> u8 {
        byte(self.ram, INTRO_SWORD_SPARKLE_Y_OFFSET)
    }

    pub(crate) fn flash_rgb_channel(&self) -> usize {
        byte(self.ram, INTRO_SWORD_FLASH_RGB_CHANNEL) as usize
    }
}

pub(crate) struct IntroSwordViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> IntroSwordViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn reset_sword_state(&mut self) {
        self.ram[INTRO_SWORD_SPARKLE_STEP] = 7;
        self.ram[INTRO_SWORD_ANIM_STEP] = 0;
        self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 0;
        write_le_u16(self.ram, INTRO_SWORD_YPOS, (-130i16) as u16);
    }

    pub(crate) fn set_ypos(&mut self, value: u16) {
        write_le_u16(self.ram, INTRO_SWORD_YPOS, value);
    }

    pub(crate) fn advance_ypos(&mut self) {
        let y = read_le_u16(self.ram, INTRO_SWORD_YPOS).wrapping_add(16);
        write_le_u16(self.ram, INTRO_SWORD_YPOS, y);
    }

    pub(crate) fn decrement_sparkle_timer(&mut self) {
        self.ram[INTRO_SWORD_SPARKLE_TIMER] = self.ram[INTRO_SWORD_SPARKLE_TIMER].wrapping_sub(1);
    }

    pub(crate) fn set_sparkle_timer(&mut self, value: u8) {
        self.ram[INTRO_SWORD_SPARKLE_TIMER] = value;
    }

    pub(crate) fn set_sparkle_step(&mut self, value: u8) {
        self.ram[INTRO_SWORD_SPARKLE_STEP] = value;
    }

    pub(crate) fn decrement_sparkle_step_check_negative(&mut self) -> bool {
        self.ram[INTRO_SWORD_SPARKLE_STEP] = self.ram[INTRO_SWORD_SPARKLE_STEP].wrapping_sub(1);
        (self.ram[INTRO_SWORD_SPARKLE_STEP] as i8) < 0
    }

    pub(crate) fn advance_anim_step(&mut self) {
        self.ram[INTRO_SWORD_ANIM_STEP] = self.ram[INTRO_SWORD_ANIM_STEP].wrapping_add(2);
    }

    pub(crate) fn set_sparkle_y_offset(&mut self, value: u8) {
        self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = value;
    }

    pub(crate) fn advance_sparkle_y_offset(&mut self) {
        self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET] =
            self.ram[INTRO_SWORD_SPARKLE_Y_OFFSET].wrapping_add(4);
    }

    pub(crate) fn set_flash_rgb_channel_word(&mut self, value: u16) {
        write_le_u16(self.ram, INTRO_SWORD_FLASH_RGB_CHANNEL, value);
    }

    pub(crate) fn cycle_flash_rgb_channel(&mut self) {
        self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL] = if self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL] == 2 {
            0
        } else {
            self.ram[INTRO_SWORD_FLASH_RGB_CHANNEL].wrapping_add(1)
        };
    }
}
