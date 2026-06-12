use super::*;

pub(crate) struct DisplayNmiView<'a> {
    ram: &'a [u8],
}

impl<'a> DisplayNmiView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn hud_tile_indices_buffer(&self) -> &[u8] {
        &self.ram[HUD_TILE_INDICES_BUFFER..]
    }

    pub(crate) fn oam_buf(&self) -> &[u8] {
        &self.ram[OAM_BUF..]
    }

    pub(crate) fn tilemap_upload_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER..]
    }

    pub(crate) fn stripe_buffer_021b(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B..]
    }

    pub(crate) fn vram_upload_tile_buf(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF..]
    }

    pub(crate) fn bg1_wall_top_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER..]
    }

    pub(crate) fn bg1_wall_bottom_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER..]
    }

    pub(crate) fn bg_char_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER..]
    }

    pub(crate) fn bg_char_buffer_1(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1..]
    }

    pub(crate) fn bg_char_half_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER..]
    }

    pub(crate) fn game_over_text_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER..]
    }

    pub(crate) fn game_over_text_tail_buffer(&self) -> &[u8] {
        &self.ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER..]
    }

    pub(crate) fn polyhedral_buffer(&self) -> &[u8] {
        &self.ram[POLYHEDRAL_BUFFER..]
    }

    pub(crate) fn arbitrary_tilemap_dst(&self, slot: usize) -> u16 {
        word(
            self.ram,
            crate::game_state::constants::nmi::ARBITRARY_TILEMAP_DST_BUFFER + slot * 2,
        )
    }

    pub(crate) fn dungeon_bg2_attr_table(&self) -> &[u8] {
        &self.ram[DUNGEON_BG2_ATTR_TABLE..]
    }

    pub(crate) fn dungeon_bg1_attr_table(&self) -> &[u8] {
        &self.ram[DUNGEON_BG1_ATTR_TABLE..]
    }

    pub(crate) fn ram_slice_at(&self, addr: usize, len: usize) -> &[u8] {
        let start = addr.min(self.ram.len());
        let end = (addr + len).min(self.ram.len());
        &self.ram[start..end]
    }
}

pub(crate) struct DisplayNmiViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> DisplayNmiViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn tilemap_upload_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.ram[crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER..]
    }
}

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

pub(crate) struct PaletteFilterView<'a> {
    ram: &'a [u8],
}

impl<'a> PaletteFilterView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn countdown(&self) -> u8 {
        byte(self.ram, PALETTE_FILTER_COUNTDOWN)
    }

    pub(crate) fn countdown_word(&self) -> u16 {
        word(self.ram, PALETTE_FILTER_COUNTDOWN)
    }

    pub(crate) fn darkening_or_lightening_screen(&self) -> u8 {
        byte(self.ram, DARKENING_OR_LIGHTENING_SCREEN)
    }

    pub(crate) fn darkening_or_lightening_screen_word(&self) -> u16 {
        word(self.ram, DARKENING_OR_LIGHTENING_SCREEN)
    }

    pub(crate) fn color_window_selection(&self) -> u8 {
        byte(self.ram, CGWSEL_COPY)
    }

    pub(crate) fn color_window_and_math_word(&self) -> u16 {
        word(self.ram, CGWSEL_COPY)
    }

    pub(crate) fn color_math_control(&self) -> u8 {
        byte(self.ram, CGADSUB_COPY)
    }

    pub(crate) fn color_math_control_word(&self) -> u16 {
        word(self.ram, CGADSUB_COPY)
    }

    pub(crate) fn fixed_color_red(&self) -> u8 {
        byte(self.ram, COLDATA_COPY0)
    }

    pub(crate) fn fixed_color_green(&self) -> u8 {
        byte(self.ram, COLDATA_COPY1)
    }

    pub(crate) fn fixed_color_blue(&self) -> u8 {
        byte(self.ram, COLDATA_COPY2)
    }

    pub(crate) fn fixed_color_component(&self, index: usize) -> u8 {
        byte(self.ram, COLDATA_COPY0 + index)
    }
}

pub(crate) struct PaletteFilterViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> PaletteFilterViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_countdown(&mut self, value: u8) {
        self.ram[PALETTE_FILTER_COUNTDOWN] = value;
    }

    pub(crate) fn increment_countdown(&mut self) {
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
    }

    pub(crate) fn decrement_countdown(&mut self) {
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_sub(1);
    }

    pub(crate) fn set_countdown_word(&mut self, value: u16) {
        write_le_u16(self.ram, PALETTE_FILTER_COUNTDOWN, value);
    }

    pub(crate) fn set_darkening_or_lightening_screen(&mut self, value: u8) {
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = value;
    }

    pub(crate) fn xor_darkening_or_lightening_screen(&mut self, value: u8) {
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] ^= value;
    }

    pub(crate) fn set_darkening_or_lightening_screen_word(&mut self, value: u16) {
        write_le_u16(self.ram, DARKENING_OR_LIGHTENING_SCREEN, value);
    }

    pub(crate) fn set_color_window_selection(&mut self, value: u8) {
        self.ram[CGWSEL_COPY] = value;
    }

    pub(crate) fn set_color_window_and_math_word(&mut self, value: u16) {
        write_le_u16(self.ram, CGWSEL_COPY, value);
    }

    pub(crate) fn set_color_math_control(&mut self, value: u8) {
        self.ram[CGADSUB_COPY] = value;
    }

    pub(crate) fn set_fixed_color_red(&mut self, value: u8) {
        self.ram[COLDATA_COPY0] = value;
    }

    pub(crate) fn or_fixed_color_red(&mut self, value: u8) {
        self.ram[COLDATA_COPY0] |= value;
    }

    pub(crate) fn subtract_fixed_color_red(&mut self, value: u8) {
        self.ram[COLDATA_COPY0] = self.ram[COLDATA_COPY0].wrapping_sub(value);
    }

    pub(crate) fn set_fixed_color_green(&mut self, value: u8) {
        self.ram[COLDATA_COPY1] = value;
    }

    pub(crate) fn or_fixed_color_green(&mut self, value: u8) {
        self.ram[COLDATA_COPY1] |= value;
    }

    pub(crate) fn subtract_fixed_color_green(&mut self, value: u8) {
        self.ram[COLDATA_COPY1] = self.ram[COLDATA_COPY1].wrapping_sub(value);
    }

    pub(crate) fn set_fixed_color_blue(&mut self, value: u8) {
        self.ram[COLDATA_COPY2] = value;
    }

    pub(crate) fn or_fixed_color_blue(&mut self, value: u8) {
        self.ram[COLDATA_COPY2] |= value;
    }

    pub(crate) fn subtract_fixed_color_blue(&mut self, value: u8) {
        self.ram[COLDATA_COPY2] = self.ram[COLDATA_COPY2].wrapping_sub(value);
    }

    pub(crate) fn set_fixed_color_component(&mut self, index: usize, value: u8) {
        self.ram[COLDATA_COPY0 + index] = value;
    }

    pub(crate) fn or_fixed_color_component(&mut self, index: usize, value: u8) {
        self.ram[COLDATA_COPY0 + index] |= value;
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

pub(crate) struct HudInventoryOrderView<'a> {
    ram: &'a [u8],
}

impl<'a> HudInventoryOrderView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn is_custom(&self) -> bool {
        byte(self.ram, HUD_INVENTORY_ORDER) != 0
    }

    pub(crate) fn item(&self, index: usize) -> u8 {
        byte(self.ram, HUD_INVENTORY_ORDER + index)
    }
}

pub(crate) struct HudInventoryOrderViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> HudInventoryOrderViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn initialize_default_order(&mut self, count: usize) {
        for i in 0..count {
            self.ram[HUD_INVENTORY_ORDER + i] = i as u8 + 1;
        }
    }

    pub(crate) fn swap_items(&mut self, old_pos: usize, new_pos: usize) {
        self.ram
            .swap(HUD_INVENTORY_ORDER + old_pos, HUD_INVENTORY_ORDER + new_pos);
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

pub(crate) struct VramLoadStateView<'a> {
    ram: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinkDmaSourceSlot {
    BodyTop,
    BodyBottom,
    HeadTop,
    HeadBottom,
    HandLeft,
    HandRight,
    SwordUpper,
    SwordLower,
    ShieldUpper,
    ShieldLower,
    AuxUpper,
    AuxLower,
    PushUpper,
    PushLower,
    AnimatedTileUpper,
    AnimatedTileLower,
    HeadPointerUpper,
    HeadPointerLower,
    BodyPointerUpper,
    BodyPointerLower,
    TravelBirdUpper,
    TravelBirdLower,
}

impl LinkDmaSourceSlot {
    fn address(self) -> usize {
        match self {
            Self::BodyTop => DMA_SOURCE_ADDR_3,
            Self::BodyBottom => DMA_SOURCE_ADDR_0,
            Self::HeadTop => DMA_SOURCE_ADDR_4,
            Self::HeadBottom => DMA_SOURCE_ADDR_1,
            Self::HandLeft => DMA_SOURCE_ADDR_5,
            Self::HandRight => DMA_SOURCE_ADDR_2,
            Self::SwordUpper => DMA_SOURCE_ADDR_6,
            Self::SwordLower => DMA_SOURCE_ADDR_11,
            Self::ShieldUpper => DMA_SOURCE_ADDR_7,
            Self::ShieldLower => DMA_SOURCE_ADDR_12,
            Self::AuxUpper => DMA_SOURCE_ADDR_8,
            Self::AuxLower => DMA_SOURCE_ADDR_13,
            Self::PushUpper => DMA_SOURCE_ADDR_10,
            Self::PushLower => DMA_SOURCE_ADDR_15,
            Self::AnimatedTileUpper => DMA_SOURCE_ADDR_9,
            Self::AnimatedTileLower => DMA_SOURCE_ADDR_14,
            Self::HeadPointerUpper => DMA_SOURCE_ADDR_16,
            Self::HeadPointerLower => DMA_SOURCE_ADDR_18,
            Self::BodyPointerUpper => DMA_SOURCE_ADDR_17,
            Self::BodyPointerLower => DMA_SOURCE_ADDR_19,
            Self::TravelBirdUpper => DMA_SOURCE_ADDR_20,
            Self::TravelBirdLower => DMA_SOURCE_ADDR_21,
        }
    }
}

impl<'a> VramLoadStateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn incremental_counter(&self) -> u8 {
        byte(self.ram, INCREMENTAL_COUNTER_FOR_VRAM)
    }

    pub(crate) fn link_dma_source(&self, slot: LinkDmaSourceSlot) -> u16 {
        word(self.ram, slot.address())
    }
}
pub(crate) struct VramLoadStateViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> VramLoadStateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn reset_incremental_counter(&mut self) {
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0;
    }
    pub(crate) fn increment_counter(&mut self) {
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] =
            self.ram[INCREMENTAL_COUNTER_FOR_VRAM].wrapping_add(1);
    }
    pub(crate) fn set_nmi_tilemap_dst(&mut self, value: u8) {
        self.ram[NMI_UPDATE_TILEMAP_DST] = value;
    }
    pub(crate) fn set_nmi_tilemap_src(&mut self, value: u16) {
        write_le_u16(self.ram, NMI_UPDATE_TILEMAP_SRC, value);
    }

    pub(crate) fn set_link_dma_source(&mut self, addr: usize, value: u16) {
        write_le_u16(self.ram, addr, value);
    }

    pub(crate) fn set_link_body_dma_sources(&mut self, top: u16, bottom: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_3, top);
        self.set_link_dma_source(DMA_SOURCE_ADDR_0, bottom);
    }

    pub(crate) fn set_link_head_dma_sources(&mut self, top: u16, bottom: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_4, top);
        self.set_link_dma_source(DMA_SOURCE_ADDR_1, bottom);
    }

    pub(crate) fn set_link_hand_dma_sources(&mut self, left: u16, right: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_5, left);
        self.set_link_dma_source(DMA_SOURCE_ADDR_2, right);
    }

    pub(crate) fn set_link_sword_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_6, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_11, lower);
    }

    pub(crate) fn set_link_shield_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_7, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_12, lower);
    }

    pub(crate) fn set_link_aux_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_8, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_13, lower);
    }

    pub(crate) fn set_link_push_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_10, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_15, lower);
    }

    pub(crate) fn set_link_animated_tile_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_9, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_14, lower);
    }

    pub(crate) fn set_link_head_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_16, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_18, lower);
    }

    pub(crate) fn set_link_body_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_17, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_19, lower);
    }

    pub(crate) fn set_travel_bird_dma_sources(&mut self, upper: u16, lower: u16) {
        self.set_link_dma_source(DMA_SOURCE_ADDR_20, upper);
        self.set_link_dma_source(DMA_SOURCE_ADDR_21, lower);
    }

    pub(crate) fn reset_bg_tile_animation_countdown(&mut self, value: u16) {
        write_le_u16(self.ram, BG_TILE_ANIMATION_COUNTDOWN, value);
    }
}

pub(crate) struct OverworldTileUpdateView<'a> {
    ram: &'a [u8],
}

impl<'a> OverworldTileUpdateView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn tile_attr_word(&self, index: usize) -> u16 {
        word(
            self.ram,
            crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2,
        )
    }
}

pub(crate) struct OverworldTileUpdateViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> OverworldTileUpdateViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_tile_attr_word(&mut self, index: usize, value: u16) {
        write_le_u16(
            self.ram,
            crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2,
            value,
        );
    }

    pub(crate) fn set_upload_word(&mut self, index: usize, value: u16) {
        write_le_u16(
            self.ram,
            crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + index * 2,
            value,
        );
    }

    pub(crate) fn terminate_upload_words(&mut self, index: usize) {
        self.set_upload_word(index, 0xffff);
    }
}

pub(crate) struct StarTileView<'a> {
    ram: &'a [u8],
}
impl<'a> StarTileView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn restore_phase(&self) -> u8 {
        byte(self.ram, STAR_TILE_RESTORE_PHASE)
    }
}
pub(crate) struct StarTileViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> StarTileViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn clear_restore_phase(&mut self) {
        self.ram[STAR_TILE_RESTORE_PHASE] = 0;
    }
}

pub(crate) struct TrinexxPaletteView<'a> {
    ram: &'a [u8],
}
impl<'a> TrinexxPaletteView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn red_shell_delay(&self) -> u8 {
        byte(self.ram, TRINEXX_RED_SHELL_PALETTE_DELAY)
    }
    pub(crate) fn blue_shell_delay(&self) -> u8 {
        byte(self.ram, TRINEXX_BLUE_SHELL_PALETTE_DELAY)
    }
    pub(crate) fn red_shell_step(&self) -> u8 {
        byte(self.ram, TRINEXX_RED_SHELL_PALETTE_STEP)
    }
    pub(crate) fn blue_shell_step(&self) -> u8 {
        byte(self.ram, TRINEXX_BLUE_SHELL_PALETTE_STEP)
    }
}
pub(crate) struct TrinexxPaletteViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> TrinexxPaletteViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn set_red_shell_delay(&mut self, v: u8) {
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = v;
    }
    pub(crate) fn set_blue_shell_delay(&mut self, v: u8) {
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = v;
    }
    pub(crate) fn set_red_shell_step(&mut self, v: u8) {
        self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] = v;
    }
    pub(crate) fn set_blue_shell_step(&mut self, v: u8) {
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = v;
    }
    pub(crate) fn decrement_red_shell_delay(&mut self) {
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY].wrapping_sub(1);
    }
    pub(crate) fn decrement_blue_shell_delay(&mut self) {
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY].wrapping_sub(1);
    }
    pub(crate) fn increment_red_shell_step(&mut self) -> u8 {
        self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_STEP].wrapping_add(1);
        self.ram[TRINEXX_RED_SHELL_PALETTE_STEP]
    }
    pub(crate) fn increment_blue_shell_step(&mut self) -> u8 {
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP].wrapping_add(1);
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP]
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

pub(crate) struct WaterHdmaWindowView<'a> {
    ram: &'a [u8],
}
impl<'a> WaterHdmaWindowView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn window_x(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_X)
    }
    pub(crate) fn window_y(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_Y)
    }
    pub(crate) fn window_y_radius(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_Y_RADIUS)
    }
    pub(crate) fn window_x_radius(&self) -> u16 {
        word(self.ram, WATER_HDMA_WINDOW_X_RADIUS)
    }
    pub(crate) fn watergate_spotlight_y_upper(&self) -> u16 {
        word(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER)
    }
    pub(crate) fn watergate_pointer(&self) -> u8 {
        byte(self.ram, WATERGATE_POINTER)
    }
    pub(crate) fn watergate_tilemap_pos_x2(&self) -> u16 {
        word(self.ram, WATERGATE_POS)
    }
}
pub(crate) struct WaterHdmaWindowViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> WaterHdmaWindowViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn decrement_watergate_spotlight_y_upper(&mut self) -> u16 {
        let v = word(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER).wrapping_sub(1);
        write_le_u16(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, v);
        v
    }

    pub(crate) fn set_watergate_pointer(&mut self, value: u8) {
        self.ram[WATERGATE_POINTER] = value;
    }

    pub(crate) fn increment_watergate_pointer(&mut self) -> u8 {
        self.ram[WATERGATE_POINTER] = self.ram[WATERGATE_POINTER].wrapping_add(1);
        self.ram[WATERGATE_POINTER]
    }

    pub(crate) fn set_watergate_tilemap_pos_x2(&mut self, value: u16) {
        write_le_u16(self.ram, WATERGATE_POS, value);
    }

    pub(crate) fn set_window_x(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X, value);
    }

    pub(crate) fn set_window_y(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_Y, value);
    }

    pub(crate) fn set_window_x_radius(&mut self, value: u16) {
        write_le_u16(self.ram, WATER_HDMA_WINDOW_X_RADIUS, value);
    }

    pub(crate) fn set_window_y_radius_byte(&mut self, value: u8) {
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] = value;
    }

    pub(crate) fn increment_window_y_radius_byte(&mut self) -> u8 {
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] = self.ram[WATER_HDMA_WINDOW_Y_RADIUS].wrapping_add(1);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS]
    }

    pub(crate) fn set_watergate_spotlight_y_upper(&mut self, value: u16) {
        write_le_u16(self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, value);
    }

    pub(crate) fn copy_watergate_spotlight_to_spotlight_upper(&mut self) {
        self.ram[SPOTLIGHT_Y_UPPER] = self.ram[WATERGATE_SPOTLIGHT_Y_UPPER];
    }

    pub(crate) fn advance_watergate_window_y_radius(&mut self) -> u8 {
        self.copy_watergate_spotlight_to_spotlight_upper();
        self.ram[SPOTLIGHT_WINDOW_Y_BUFFER] = self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(1);
        let x_radius_minus_margin = self.ram[WATER_HDMA_WINDOW_X_RADIUS].wrapping_sub(8);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS] =
            self.ram[SPOTLIGHT_WINDOW_Y_BUFFER].wrapping_add(x_radius_minus_margin);
        self.ram[WATER_HDMA_WINDOW_Y_RADIUS]
    }
}

pub(crate) struct MosaicDirectionView<'a> {
    ram: &'a [u8],
}
impl<'a> MosaicDirectionView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn inc_or_dec(&self) -> u8 {
        byte(self.ram, MOSAIC_INC_OR_DEC)
    }
}
pub(crate) struct MosaicDirectionViewMut<'a> {
    ram: &'a mut [u8],
}
impl<'a> MosaicDirectionViewMut<'a> {
    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }
    pub(crate) fn set_inc_or_dec(&mut self, v: u8) {
        self.ram[MOSAIC_INC_OR_DEC] = v;
    }
    pub(crate) fn clear(&mut self) {
        self.ram[MOSAIC_INC_OR_DEC] = 0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_dma_source_slots_read_named_source_addresses() {
        let mut ram = vec![0; 0x10000];
        let slots = [
            (LinkDmaSourceSlot::BodyTop, DMA_SOURCE_ADDR_3),
            (LinkDmaSourceSlot::BodyBottom, DMA_SOURCE_ADDR_0),
            (LinkDmaSourceSlot::HeadTop, DMA_SOURCE_ADDR_4),
            (LinkDmaSourceSlot::HeadBottom, DMA_SOURCE_ADDR_1),
            (LinkDmaSourceSlot::HandLeft, DMA_SOURCE_ADDR_5),
            (LinkDmaSourceSlot::HandRight, DMA_SOURCE_ADDR_2),
            (LinkDmaSourceSlot::SwordUpper, DMA_SOURCE_ADDR_6),
            (LinkDmaSourceSlot::SwordLower, DMA_SOURCE_ADDR_11),
            (LinkDmaSourceSlot::ShieldUpper, DMA_SOURCE_ADDR_7),
            (LinkDmaSourceSlot::ShieldLower, DMA_SOURCE_ADDR_12),
            (LinkDmaSourceSlot::AuxUpper, DMA_SOURCE_ADDR_8),
            (LinkDmaSourceSlot::AuxLower, DMA_SOURCE_ADDR_13),
            (LinkDmaSourceSlot::PushUpper, DMA_SOURCE_ADDR_10),
            (LinkDmaSourceSlot::PushLower, DMA_SOURCE_ADDR_15),
            (LinkDmaSourceSlot::AnimatedTileUpper, DMA_SOURCE_ADDR_9),
            (LinkDmaSourceSlot::AnimatedTileLower, DMA_SOURCE_ADDR_14),
            (LinkDmaSourceSlot::HeadPointerUpper, DMA_SOURCE_ADDR_16),
            (LinkDmaSourceSlot::HeadPointerLower, DMA_SOURCE_ADDR_18),
            (LinkDmaSourceSlot::BodyPointerUpper, DMA_SOURCE_ADDR_17),
            (LinkDmaSourceSlot::BodyPointerLower, DMA_SOURCE_ADDR_19),
            (LinkDmaSourceSlot::TravelBirdUpper, DMA_SOURCE_ADDR_20),
            (LinkDmaSourceSlot::TravelBirdLower, DMA_SOURCE_ADDR_21),
        ];

        for (index, (_, address)) in slots.iter().copied().enumerate() {
            write_le_u16(&mut ram, address, 0x9000 + index as u16);
        }

        let view = VramLoadStateView::new(&ram);
        for (index, (slot, _)) in slots.iter().copied().enumerate() {
            assert_eq!(view.link_dma_source(slot), 0x9000 + index as u16);
        }
    }
}
