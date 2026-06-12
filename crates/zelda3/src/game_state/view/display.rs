use super::*;

pub(crate) struct PaletteBufferView<'a> {
    ram: &'a [u8],
}

impl<'a> PaletteBufferView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn main_color(&self, index: usize) -> u16 {
        word(self.ram, MAIN_PALETTE_BUFFER + index * 2)
    }

    pub(crate) fn aux_color(&self, index: usize) -> u16 {
        word(self.ram, AUX_PALETTE_BUFFER + index * 2)
    }

    pub(crate) fn aux_visible_slice(&self) -> &[u8] {
        &self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 256]
    }

    pub(crate) fn main_full_slice(&self) -> &[u8] {
        &self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 512]
    }

    pub(crate) fn aux_full_slice(&self) -> &[u8] {
        &self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 512]
    }

    pub(crate) fn overworld_palette_backup(&self) -> &[u8] {
        &self.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 512]
    }

    pub(crate) fn overworld_aux_or_main_offset(&self) -> u16 {
        word(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN)
    }
}

pub(crate) struct PaletteBufferViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> PaletteBufferViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn clear_aux_visible_subpalettes(&mut self) {
        self.ram[AUX_PALETTE_BUFFER + 32 * 2..AUX_PALETTE_BUFFER + 32 * 2 + 192].fill(0);
    }

    pub(crate) fn clear_main_visible_subpalettes(&mut self) {
        self.ram[MAIN_PALETTE_BUFFER + 32 * 2..MAIN_PALETTE_BUFFER + 32 * 2 + 192].fill(0);
    }

    pub(crate) fn clear_aux_sprite_subpalettes(&mut self) {
        self.ram[AUX_PALETTE_BUFFER + 0x180..AUX_PALETTE_BUFFER + 0x200].fill(0);
    }

    pub(crate) fn set_main_color(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, MAIN_PALETTE_BUFFER + index * 2, value);
    }

    pub(crate) fn set_aux_color(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, AUX_PALETTE_BUFFER + index * 2, value);
    }

    pub(crate) fn set_overworld_aux_or_main_offset(&mut self, value: u16) {
        write_le_u16(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, value);
    }

    pub(crate) fn clear_overworld_aux_or_main_offset(&mut self) {
        self.set_overworld_aux_or_main_offset(0);
    }

    pub(crate) fn select_overworld_aux_palette_offset(&mut self) {
        self.set_overworld_aux_or_main_offset(0x0200);
    }

    pub(crate) fn keep_overworld_aux_or_main_low_byte(&mut self) {
        let value = word(self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) & 0x00ff;
        self.set_overworld_aux_or_main_offset(value);
    }

    pub(crate) fn clear_main_full(&mut self) {
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 0x200].fill(0);
    }

    pub(crate) fn copy_aux_visible_from(&mut self, palette: &[u8]) {
        self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 256].copy_from_slice(palette);
    }

    pub(crate) fn copy_aux_full_from(&mut self, palette: &[u8]) {
        self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 512].copy_from_slice(palette);
    }

    pub(crate) fn backup_overworld_palette_from(&mut self, palette: &[u8]) {
        self.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 512].copy_from_slice(palette);
    }

    pub(crate) fn copy_main_full_from(&mut self, palette: &[u8]) {
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 512].copy_from_slice(palette);
    }

    pub(crate) fn copy_main_palette_bytes(&mut self, src: &[u8], len: usize) {
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + len].copy_from_slice(&src[..len]);
    }

    pub(crate) fn set_sp0l(&mut self, value: u8) {
        self.ram[PALETTE_SP0L] = value;
    }

    pub(crate) fn set_sp5l(&mut self, value: u8) {
        self.ram[PALETTE_SP5L] = value;
    }

    pub(crate) fn set_sp6l(&mut self, value: u8) {
        self.ram[PALETTE_SP6L] = value;
    }

    pub(crate) fn set_palette_main_indoors(&mut self, value: u8) {
        self.ram[PALETTE_MAIN_INDOORS] = value;
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.ram[HUD_PALETTE] = value;
    }

    pub(crate) fn set_sp6r_indoors(&mut self, value: u8) {
        self.ram[PALETTE_SP6R_INDOORS] = value;
    }

    pub(crate) fn set_overworld_palette_aux2_hi(&mut self, value: u8) {
        self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = value;
    }

    pub(crate) fn set_overworld_palette_aux3_lo(&mut self, value: u8) {
        self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = value;
    }

    pub(crate) fn set_bg_tile_animation_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN, value);
    }

    pub(crate) fn set_overworld_palette_mode(&mut self, value: u8) {
        self.ram[OVERWORLD_PALETTE_MODE] = value;
    }
}

pub(crate) struct HudStateView<'a> {
    ram: &'a [u8],
}

impl<'a> HudStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn floor_changed_timer_low(&self) -> u8 {
        byte(self.ram, HUD_FLOOR_CHANGED_TIMER)
    }

    pub(crate) fn super_bomb_indicator_timer(&self) -> u8 {
        byte(self.ram, SUPER_BOMB_INDICATOR_TIMER)
    }

    pub(crate) fn super_bomb_indicator_counter(&self) -> u8 {
        byte(self.ram, SUPER_BOMB_INDICATOR_COUNTER)
    }

    pub(crate) fn rupee_sfx_sound_delay(&self) -> u8 {
        byte(self.ram, RUPEE_SFX_SOUND_DELAY)
    }

    pub(crate) fn is_doing_heart_animation(&self) -> bool {
        byte(self.ram, IS_DOING_HEART_ANIMATION) != 0
    }

    pub(crate) fn is_doing_heart_animation_raw(&self) -> u8 {
        byte(self.ram, IS_DOING_HEART_ANIMATION)
    }

    pub(crate) fn heart_refill_countdown(&self) -> u8 {
        byte(self.ram, HEART_REFILL_COUNTDOWN)
    }

    pub(crate) fn heart_refill_anim_subpos(&self) -> u8 {
        byte(self.ram, HEART_REFILL_ANIM_SUBPOS)
    }

    pub(crate) fn flashing_circle_timer(&self) -> u8 {
        byte(self.ram, FLASHING_CIRCLE_TIMER)
    }

    pub(crate) fn prev_joypad_h(&self) -> u8 {
        byte(self.ram, MENU_PREV_JOYPAD_H)
    }

    pub(crate) fn equipment_menu_exit_state(&self) -> u8 {
        byte(self.ram, EQUIPMENT_MENU_EXIT_STATE)
    }

    pub(crate) fn bottle_menu_row(&self) -> u8 {
        byte(self.ram, BOTTLE_MENU_ROW)
    }

    pub(crate) fn dungeon_dark_with_lantern(&self) -> bool {
        byte(self.ram, HDR_DUNGEON_DARK_WITH_LANTERN) != 0
    }

    pub(crate) fn tick_counter(&self) -> u8 {
        byte(self.ram, HUD_MODULE_TICK_COUNTER)
    }

    pub(crate) fn tile_word(&self, tile: usize) -> u16 {
        word(self.ram, HUD_TILE_INDICES_BUFFER + tile * 2)
    }
}

pub(crate) struct HudStateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> HudStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_floor_changed_timer(&mut self, value: u16) {
        write_le_u16(self.ram, HUD_FLOOR_CHANGED_TIMER, value);
    }

    pub(crate) fn set_super_bomb_indicator_timer(&mut self, value: u8) {
        self.ram[SUPER_BOMB_INDICATOR_TIMER] = value;
    }

    pub(crate) fn set_super_bomb_indicator_counter(&mut self, value: u8) {
        self.ram[SUPER_BOMB_INDICATOR_COUNTER] = value;
    }

    pub(crate) fn set_rupee_sfx_sound_delay(&mut self, value: u8) {
        self.ram[RUPEE_SFX_SOUND_DELAY] = value;
    }

    pub(crate) fn set_is_doing_heart_animation(&mut self, value: u8) {
        self.ram[IS_DOING_HEART_ANIMATION] = value;
    }

    pub(crate) fn set_tile_word(&mut self, tile: usize, value: u16) {
        write_le_u16(self.ram, HUD_TILE_INDICES_BUFFER + tile * 2, value);
    }

    pub(crate) fn clear_is_doing_heart_animation(&mut self) {
        self.ram[IS_DOING_HEART_ANIMATION] = 0;
    }

    pub(crate) fn set_heart_refill_countdown(&mut self, value: u8) {
        self.ram[HEART_REFILL_COUNTDOWN] = value;
    }

    pub(crate) fn set_heart_refill_anim_subpos(&mut self, value: u8) {
        self.ram[HEART_REFILL_ANIM_SUBPOS] = value;
    }

    pub(crate) fn set_flashing_circle_timer(&mut self, value: u8) {
        self.ram[FLASHING_CIRCLE_TIMER] = value;
    }

    pub(crate) fn set_prev_joypad_h(&mut self, value: u8) {
        self.ram[MENU_PREV_JOYPAD_H] = value;
    }

    pub(crate) fn clear_prev_joypad_h(&mut self) {
        self.ram[MENU_PREV_JOYPAD_H] = 0;
    }

    pub(crate) fn set_equipment_menu_exit_state(&mut self, value: u8) {
        self.ram[EQUIPMENT_MENU_EXIT_STATE] = value;
    }

    pub(crate) fn set_bottle_menu_row(&mut self, value: u8) {
        self.ram[BOTTLE_MENU_ROW] = value;
    }

    pub(crate) fn decrement_bottle_menu_row(&mut self) -> u8 {
        self.ram[BOTTLE_MENU_ROW] = self.ram[BOTTLE_MENU_ROW].wrapping_sub(1);
        self.ram[BOTTLE_MENU_ROW]
    }

    pub(crate) fn set_dungeon_dark_with_lantern(&mut self) {
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
    }

    pub(crate) fn set_tick_counter(&mut self, value: u8) {
        self.ram[HUD_MODULE_TICK_COUNTER] = value;
    }

    pub(crate) fn clear_floor_changed_timer_low(&mut self) {
        self.ram[HUD_FLOOR_CHANGED_TIMER] = 0;
    }
}

pub(crate) struct GraphicsScratchViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> GraphicsScratchViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_aux_bg_subset_pack(&mut self, index: usize, value: u8) {
        self.ram[AUX_BG_SUBSET_0 + index] = value;
    }

    pub(crate) fn primary_decomp_buffer_offset() -> usize {
        PRIMARY_DECOMP_BUFFER_LOAD_GFX
    }

    pub(crate) fn secondary_decomp_buffer_offset() -> usize {
        SECONDARY_DECOMP_BUFFER_LOAD_GFX
    }

    pub(crate) fn primary_decomp_buffer(&self, len: usize) -> Vec<u8> {
        self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len].to_vec()
    }

    pub(crate) fn combined_decomp_buffers(&self) -> Vec<u8> {
        self.primary_decomp_buffer(0x0c00)
    }

    pub(crate) fn copy_to_primary_decomp_buffer(&mut self, data: &[u8]) {
        let len = data.len().min(
            self.ram
                .len()
                .saturating_sub(PRIMARY_DECOMP_BUFFER_LOAD_GFX),
        );
        self.ram[PRIMARY_DECOMP_BUFFER_LOAD_GFX..PRIMARY_DECOMP_BUFFER_LOAD_GFX + len]
            .copy_from_slice(&data[..len]);
    }

    pub(crate) fn copy_decompressed_graphics_to(&mut self, dst: usize, data: &[u8]) -> usize {
        let len = data.len().min(self.ram.len().saturating_sub(dst));
        self.ram[dst..dst + len].copy_from_slice(&data[..len]);
        len
    }

    pub(crate) fn rotate_animated_dungeon_tile_planes(&mut self) {
        for i in 0..256 {
            let base = 0x9000 + i * 2;
            let x = word(self.ram, base + 0x1880);
            let a = word(self.ram, base + 0x1c80);
            let b = word(self.ram, base + 0x1e80);
            let c = word(self.ram, base + 0x1a80);
            write_le_u16(self.ram, base + 0x1880, a);
            write_le_u16(self.ram, base + 0x1c80, b);
            write_le_u16(self.ram, base + 0x1e80, c);
            write_le_u16(self.ram, base + 0x1a80, x);
        }
    }

    pub(crate) fn write_expanded_tile_row(
        &mut self,
        dst: usize,
        low_plane: u8,
        high_plane: u8,
        upper_plane: u8,
        composite_plane: u8,
    ) {
        self.ram[dst] = low_plane;
        self.ram[dst + 1] = high_plane;
        self.ram[dst + 0x10] = upper_plane;
        self.ram[dst + 0x11] = composite_plane;
    }

    pub(crate) fn set_dungeon_line_pointer_row0(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, DUNG_LINE_PTRS_ROW0 + index * 2, value);
    }

    pub(crate) fn copy_message_rows(&mut self, dst: usize, src0: usize, src1: usize, len: usize) {
        for i in 0..len {
            self.ram[MESSAGING_RENDER_BUFFER + dst + i] = self.ram[src0 + i];
            self.ram[MESSAGING_RENDER_BUFFER + dst + len + i] = self.ram[src1 + i];
        }
    }

    pub(crate) fn copy_peg_tile_graphics_to_message_buffer(&mut self, first: usize, second: usize) {
        for i in 0..64 {
            let color = word(self.ram, PEG_TILE_GFX_BUFFER + (first >> 1) * 2 + i * 2);
            write_le_u16(self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = word(self.ram, PEG_TILE_GFX_BUFFER + (second >> 1) * 2 + i * 2);
            write_le_u16(self.ram, MESSAGING_BUF_LOAD_GFX + (64 + i) * 2, color);
        }
    }

    pub(crate) fn copy_dynamic_hdma_table_to_reserved(&mut self, count: usize) {
        let bytes = count * 2;
        let src = self.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + bytes].to_vec();
        self.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + bytes].copy_from_slice(&src);
    }

    pub(crate) fn clear_agahnim_palette_settings(&mut self, len: usize) {
        self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + len].fill(0);
    }

    pub(crate) fn agahnim_palette_word(&self, index: usize) -> u16 {
        word(self.ram, AGAHNIM_PAL_SETTING + index * 2)
    }

    pub(crate) fn set_agahnim_palette_word(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, AGAHNIM_PAL_SETTING + index * 2, value);
    }

    pub(crate) fn sprite_decomp_buffer_tail(&self) -> Vec<u8> {
        self.ram[SPRITE_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }

    pub(crate) fn staged_bg_and_sprite_decomp_buffers(&self) -> Vec<u8> {
        self.ram[BG_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec()
    }
}

pub(crate) struct SpotlightHdmaView<'a> {
    ram: &'a [u8],
}
impl<'a> SpotlightHdmaView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn y_lower(&self) -> u16 {
        word(self.ram, SPOTLIGHT_Y_LOWER)
    }
    pub(crate) fn y_upper(&self) -> u16 {
        word(self.ram, SPOTLIGHT_Y_UPPER)
    }
    pub(crate) fn window_x_center(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_X_CENTER)
    }
    pub(crate) fn window_state(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_STATE)
    }
    pub(crate) fn window_state_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_STATE)
    }
    pub(crate) fn window_radius(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_RADIUS)
    }
    pub(crate) fn window_radius_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_RADIUS)
    }
    pub(crate) fn window_y_buffer(&self) -> u16 {
        word(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER)
    }
    pub(crate) fn window_y_buffer_byte(&self) -> u8 {
        byte(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER)
    }
    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        word(self.ram, HDMA_TABLE_DYNAMIC + index * 2)
    }
}
pub(crate) struct SpotlightHdmaViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> SpotlightHdmaViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn set_y_lower(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_Y_LOWER, v);
    }
    pub(crate) fn set_y_upper(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_Y_UPPER, v);
    }
    pub(crate) fn set_window_x_center(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_X_CENTER, v);
    }
    pub(crate) fn set_window_state(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_STATE, v);
    }
    pub(crate) fn set_window_radius(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_RADIUS, v);
    }
    pub(crate) fn set_window_y_buffer(&mut self, v: u16) {
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, v);
    }
    pub(crate) fn decrement_window_y_buffer(&mut self) -> u16 {
        let v = word(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER).wrapping_sub(1);
        write_le_u16(self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, v);
        v
    }

    pub(crate) fn set_window_radius_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] = v;
    }

    pub(crate) fn set_window_state_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_STATE] = v;
    }

    pub(crate) fn set_window_y_buffer_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = v;
    }

    pub(crate) fn increment_window_y_buffer_byte(&mut self) {
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(1);
    }

    pub(crate) fn shr_window_radius_byte(&mut self, shift: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] >>= shift;
    }

    pub(crate) fn add_window_radius_byte(&mut self, v: u8) {
        self.ram[SPOTLIGHT_WINDOW_RADIUS] = self.ram[SPOTLIGHT_WINDOW_RADIUS].wrapping_add(v);
    }

    pub(crate) fn hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        word(self.ram, HDMA_TABLE_DYNAMIC + index * 2)
    }

    pub(crate) fn set_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + index * 2, value);
    }

    pub(crate) fn clear_hdma_table_dynamic(&mut self, count: usize) {
        for i in 0..count {
            write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
    }

    pub(crate) fn clear_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        for i in start..start + count {
            write_le_u16(self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
    }
}

pub(crate) struct OverworldPaletteBackupViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> OverworldPaletteBackupViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn set_main_indoors_backup(&mut self, v: u8) {
        self.ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = v;
    }
    pub(crate) fn set_aux3_bp7_backup(&mut self, v: u8) {
        self.ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = v;
    }
    pub(crate) fn set_main_indoors_copy_backup(&mut self, v: u8) {
        self.ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = v;
    }
}
