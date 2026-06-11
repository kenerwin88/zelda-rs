use super::*;

pub(crate) struct PpuScrollCopyView<'a> {
    ram: &'a [u8],
}

impl<'a> PpuScrollCopyView<'a> {
    pub(crate) fn new(ram: &'a [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn bg2_h_copy2_offset() -> usize {
        BG2_X_SCROLL
    }

    pub(crate) fn bg1_h_high(&self) -> u8 {
        byte(self.ram, BG1_H_SCROLL_COPY + 1)
    }

    pub(crate) fn bg1_h_copy(&self) -> u16 {
        word(self.ram, BG1_H_SCROLL_COPY)
    }

    pub(crate) fn bg1_h_copy_low(&self) -> u8 {
        byte(self.ram, BG1_H_SCROLL_COPY)
    }

    pub(crate) fn bg1_v_high(&self) -> u8 {
        byte(self.ram, BG1_V_SCROLL_COPY + 1)
    }

    pub(crate) fn bg1_v_copy(&self) -> u16 {
        word(self.ram, BG1_V_SCROLL_COPY)
    }

    pub(crate) fn bg1_v_copy_low(&self) -> u8 {
        byte(self.ram, BG1_V_SCROLL_COPY)
    }

    pub(crate) fn bg2_h_high(&self) -> u8 {
        byte(self.ram, BG2_H_SCROLL_COPY + 1)
    }

    pub(crate) fn bg2_h_copy(&self) -> u16 {
        word(self.ram, BG2_H_SCROLL_COPY)
    }

    pub(crate) fn bg2_h_copy_low(&self) -> u8 {
        byte(self.ram, BG2_H_SCROLL_COPY)
    }

    pub(crate) fn bg2_v_high(&self) -> u8 {
        byte(self.ram, BG2_V_SCROLL_COPY + 1)
    }

    pub(crate) fn bg2_v_copy(&self) -> u16 {
        word(self.ram, BG2_V_SCROLL_COPY)
    }

    pub(crate) fn bg2_v_copy_low(&self) -> u8 {
        byte(self.ram, BG2_V_SCROLL_COPY)
    }

    pub(crate) fn bg1_h_copy2(&self) -> u16 {
        word(self.ram, BG1_X_SCROLL)
    }

    pub(crate) fn bg1_v_copy2(&self) -> u16 {
        word(self.ram, BG1_Y_SCROLL)
    }

    pub(crate) fn bg2_h_copy2(&self) -> u16 {
        word(self.ram, BG2_X_SCROLL)
    }

    pub(crate) fn bg2_v_copy2(&self) -> u16 {
        word(self.ram, BG2_Y_SCROLL)
    }

    pub(crate) fn bg2_copy2_for_axis(&self, vertical: bool) -> u16 {
        if vertical {
            self.bg2_v_copy2()
        } else {
            self.bg2_h_copy2()
        }
    }

    pub(crate) fn bg3_h_high(&self) -> u8 {
        byte(self.ram, BG3_H_SCROLL_COPY2 + 1)
    }

    pub(crate) fn bg3_h_copy2(&self) -> u16 {
        word(self.ram, BG3_H_SCROLL_COPY2)
    }

    pub(crate) fn bg3_h_copy2_low(&self) -> u8 {
        byte(self.ram, BG3_H_SCROLL_COPY2)
    }

    pub(crate) fn bg3_v_high(&self) -> u8 {
        byte(self.ram, BG3_V_SCROLL_COPY2 + 1)
    }

    pub(crate) fn bg3_v_copy2(&self) -> u16 {
        word(self.ram, BG3_V_SCROLL_COPY2)
    }

    pub(crate) fn bg3_v_copy2_low(&self) -> u8 {
        byte(self.ram, BG3_V_SCROLL_COPY2)
    }

    pub(crate) fn bg2_h_copy2_cached(&self) -> u16 {
        word(self.ram, BG2_H_SCROLL_COPY2_CACHED)
    }

    pub(crate) fn bg2_v_copy2_cached(&self) -> u16 {
        word(self.ram, BG2_V_SCROLL_COPY2_CACHED)
    }

    pub(crate) fn map_backup_bg1_h_copy2(&self) -> u16 {
        word(self.ram, MAP_BACKUP_BG1_H_SCROLL_COPY2)
    }

    /// Backup of the main screen layer designation (TM) taken when the
    /// overworld map opens.
    pub(crate) fn mapbak_tm(&self) -> u8 {
        byte(self.ram, MAPBAK_TM)
    }

    pub(crate) fn map_backup_bg2_h_copy2(&self) -> u16 {
        word(self.ram, MAP_BACKUP_BG2_H_SCROLL_COPY2)
    }

    pub(crate) fn map_backup_bg1_v_copy2(&self) -> u16 {
        word(self.ram, MAP_BACKUP_BG1_V_SCROLL_COPY2)
    }

    pub(crate) fn map_backup_bg2_v_copy2(&self) -> u16 {
        word(self.ram, MAP_BACKUP_BG2_V_SCROLL_COPY2)
    }

    pub(crate) fn special_exit_bg2_h_copy2(&self) -> u16 {
        word(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT)
    }

    pub(crate) fn special_exit_bg2_v_copy2(&self) -> u16 {
        word(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT)
    }

    pub(crate) fn exit_bg2_h_copy2(&self) -> u16 {
        word(self.ram, BG2_H_SCROLL_COPY2_EXIT)
    }

    pub(crate) fn exit_bg2_v_copy2(&self) -> u16 {
        word(self.ram, BG2_V_SCROLL_COPY2_EXIT)
    }

    pub(crate) fn mode7_center_x_high(&self) -> u8 {
        byte(self.ram, MODE7_CENTER_X_COPY + 1)
    }

    pub(crate) fn mode7_center_x(&self) -> u16 {
        word(self.ram, MODE7_CENTER_X_COPY)
    }

    pub(crate) fn mode7_center_y_high(&self) -> u8 {
        byte(self.ram, MODE7_CENTER_Y_COPY + 1)
    }

    pub(crate) fn mode7_center_y(&self) -> u16 {
        word(self.ram, MODE7_CENTER_Y_COPY)
    }

    pub(crate) fn bg1_h_subpixel(&self) -> u16 {
        word(self.ram, BG1_H_SCROLL_SUBPIXEL)
    }

    pub(crate) fn bg1_v_subpixel(&self) -> u16 {
        word(self.ram, BG1_V_SCROLL_SUBPIXEL)
    }

    pub(crate) fn mapbak_main_tile_theme_index(&self) -> u8 {
        byte(self.ram, MAPBAK_MAIN_TILE_THEME_INDEX)
    }

    pub(crate) fn mapbak_sprite_graphics_index(&self) -> u8 {
        byte(self.ram, MAPBAK_SPRITE_GRAPHICS_INDEX)
    }

    pub(crate) fn mapbak_aux_tile_theme_index(&self) -> u8 {
        byte(self.ram, MAPBAK_AUX_TILE_THEME_INDEX)
    }

    pub(crate) fn mapbak_tm_word(&self) -> u16 {
        word(self.ram, MAPBAK_TM)
    }

    pub(crate) fn mapbak_ts(&self) -> u8 {
        byte(self.ram, MAPBAK_TS)
    }

    pub(crate) fn mapbak_bg1_x_offset(&self) -> u16 {
        word(self.ram, MAPBAK_BG1_X_OFFSET)
    }

    pub(crate) fn mapbak_bg1_y_offset(&self) -> u16 {
        word(self.ram, MAPBAK_BG1_Y_OFFSET)
    }

    pub(crate) fn mapbak_cgwsel(&self) -> u8 {
        byte(self.ram, MAPBAK_CGWSEL)
    }

    pub(crate) fn mapbak_cgwsel_word(&self) -> u16 {
        word(self.ram, MAPBAK_CGWSEL)
    }

    pub(crate) fn mapbak_hdmaen(&self) -> u8 {
        byte(self.ram, MAPBAK_HDMAEN)
    }

    pub(crate) fn mapbak_palette_slice(&self) -> &[u8] {
        &self.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 0x200]
    }
}

pub(crate) struct PpuScrollCopyViewMut<'a> {
    ram: &'a mut [u8],
}

impl<'a> PpuScrollCopyViewMut<'a> {
    pub(crate) fn set_mapbak_tm(&mut self, value: u8) {
        self.ram[MAPBAK_TM] = value;
    }

    pub(crate) fn set_mapbak_ts(&mut self, value: u8) {
        self.ram[MAPBAK_TS] = value;
    }

    pub(crate) fn set_mapbak_tm_word(&mut self, value: u16) {
        write_le_u16(self.ram, MAPBAK_TM, value);
    }

    pub(crate) fn new(ram: &'a mut [u8]) -> Self {
        Self { ram }
    }

    pub(crate) fn set_bg1_h_high(&mut self, value: u8) {
        self.ram[BG1_X_SCROLL + 1] = value;
    }

    pub(crate) fn set_bg1_h_copy(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_H_SCROLL_COPY, value);
    }

    pub(crate) fn set_bg1_v_copy(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_V_SCROLL_COPY, value);
    }

    pub(crate) fn set_bg2_h_copy(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_H_SCROLL_COPY, value);
    }

    pub(crate) fn set_bg2_v_copy(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_V_SCROLL_COPY, value);
    }

    pub(crate) fn set_bg1_h_copy_low(&mut self, value: u8) {
        self.ram[BG1_H_SCROLL_COPY] = value;
    }

    pub(crate) fn set_bg1_v_copy_low(&mut self, value: u8) {
        self.ram[BG1_V_SCROLL_COPY] = value;
    }

    pub(crate) fn set_bg2_h_copy_low(&mut self, value: u8) {
        self.ram[BG2_H_SCROLL_COPY] = value;
    }

    pub(crate) fn set_bg2_v_copy_low(&mut self, value: u8) {
        self.ram[BG2_V_SCROLL_COPY] = value;
    }

    pub(crate) fn set_bg1_h_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_X_SCROLL, value);
    }

    pub(crate) fn set_bg1_v_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_Y_SCROLL, value);
    }

    pub(crate) fn set_bg2_h_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_X_SCROLL, value);
    }

    pub(crate) fn set_bg2_v_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_Y_SCROLL, value);
    }

    pub(crate) fn set_bg3_h_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG3_H_SCROLL_COPY2, value);
    }

    pub(crate) fn set_bg3_v_copy2(&mut self, value: u16) {
        write_le_u16(self.ram, BG3_V_SCROLL_COPY2, value);
    }

    pub(crate) fn set_bg3_v_copy2_low(&mut self, value: u8) {
        self.ram[BG3_V_SCROLL_COPY2] = value;
    }

    pub(crate) fn set_mode7_center_x(&mut self, value: u16) {
        write_le_u16(self.ram, MODE7_CENTER_X_COPY, value);
    }

    pub(crate) fn set_mode7_center_y(&mut self, value: u16) {
        write_le_u16(self.ram, MODE7_CENTER_Y_COPY, value);
    }

    pub(crate) fn set_mode7_center(&mut self, x: u16, y: u16) {
        self.set_mode7_center_x(x);
        self.set_mode7_center_y(y);
    }

    pub(crate) fn set_bg1_h_live_and_copy(&mut self, value: u16) {
        self.set_bg1_h_copy2(value);
        self.set_bg1_h_copy(value);
    }

    pub(crate) fn set_bg1_v_live_and_copy(&mut self, value: u16) {
        self.set_bg1_v_copy2(value);
        self.set_bg1_v_copy(value);
    }

    pub(crate) fn set_bg2_h_live_and_copy(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
        self.set_bg2_h_copy(value);
    }

    pub(crate) fn set_bg2_v_live_and_copy(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
        self.set_bg2_v_copy(value);
    }

    pub(crate) fn set_bg1_bg2_h_live_and_copy(&mut self, value: u16) {
        self.set_bg2_h_live_and_copy(value);
        self.set_bg1_h_live_and_copy(value);
    }

    pub(crate) fn set_bg1_bg2_v_live_and_copy(&mut self, value: u16) {
        self.set_bg2_v_live_and_copy(value);
        self.set_bg1_v_live_and_copy(value);
    }

    pub(crate) fn set_bg1_bg2_live_and_copy(
        &mut self,
        bg2_h: u16,
        bg2_v: u16,
        bg1_h: u16,
        bg1_v: u16,
    ) {
        self.set_bg2_h_live_and_copy(bg2_h);
        self.set_bg2_v_live_and_copy(bg2_v);
        self.set_bg1_h_live_and_copy(bg1_h);
        self.set_bg1_v_live_and_copy(bg1_v);
    }

    pub(crate) fn set_bg2_h_copy2_cached(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_H_SCROLL_COPY2_CACHED, value);
    }

    pub(crate) fn set_bg2_v_copy2_cached(&mut self, value: u16) {
        write_le_u16(self.ram, BG2_V_SCROLL_COPY2_CACHED, value);
    }

    pub(crate) fn cache_bg2_live_scroll(&mut self) {
        copy_word(self.ram, BG2_H_SCROLL_COPY2_CACHED, BG2_X_SCROLL);
        copy_word(self.ram, BG2_V_SCROLL_COPY2_CACHED, BG2_Y_SCROLL);
    }

    pub(crate) fn save_special_exit_bg2_live_scroll(&mut self) {
        copy_word(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT, BG2_X_SCROLL);
        copy_word(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT, BG2_Y_SCROLL);
    }

    pub(crate) fn save_exit_bg2_live_scroll(&mut self) {
        copy_word(self.ram, BG2_H_SCROLL_COPY2_EXIT, BG2_X_SCROLL);
        copy_word(self.ram, BG2_V_SCROLL_COPY2_EXIT, BG2_Y_SCROLL);
    }

    pub(crate) fn restore_special_exit_bg2_scroll_to_all_layers(&mut self) {
        let h = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_SPECIAL_EXIT);
        let v = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_SPECIAL_EXIT);
        self.set_all_layer_h_scrolls(h);
        self.set_all_layer_v_scrolls(v);
    }

    pub(crate) fn restore_exit_bg2_scroll_to_all_layers(&mut self) {
        let h = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_EXIT);
        let v = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_EXIT);
        self.set_all_layer_h_scrolls(h);
        self.set_all_layer_v_scrolls(v);
    }

    pub(crate) fn set_all_layer_h_scrolls(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
        self.set_bg2_h_copy(value);
        self.set_bg1_h_copy2(value);
        self.set_bg1_h_copy(value);
    }

    pub(crate) fn set_all_layer_v_scrolls(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
        self.set_bg2_v_copy(value);
        self.set_bg1_v_copy2(value);
        self.set_bg1_v_copy(value);
    }

    pub(crate) fn set_map_backup_scrolls(
        &mut self,
        bg1_h: u16,
        bg2_h: u16,
        bg1_v: u16,
        bg2_v: u16,
    ) {
        write_le_u16(self.ram, MAP_BACKUP_BG1_H_SCROLL_COPY2, bg1_h);
        write_le_u16(self.ram, MAP_BACKUP_BG2_H_SCROLL_COPY2, bg2_h);
        write_le_u16(self.ram, MAP_BACKUP_BG1_V_SCROLL_COPY2, bg1_v);
        write_le_u16(self.ram, MAP_BACKUP_BG2_V_SCROLL_COPY2, bg2_v);
    }

    pub(crate) fn clear_bg3_h_copy2(&mut self) {
        self.set_bg3_h_copy2(0);
    }

    pub(crate) fn clear_bg3_v_copy2(&mut self) {
        self.set_bg3_v_copy2(0);
    }

    pub(crate) fn add_bg1_h_copy_low(&mut self, value: u8) {
        self.ram[BG1_H_SCROLL_COPY] = self.ram[BG1_H_SCROLL_COPY].wrapping_add(value);
    }

    pub(crate) fn add_bg1_v_copy_low(&mut self, value: u8) {
        self.ram[BG1_V_SCROLL_COPY] = self.ram[BG1_V_SCROLL_COPY].wrapping_add(value);
    }

    pub(crate) fn add_bg2_v_copy_low(&mut self, value: u8) {
        self.ram[BG2_V_SCROLL_COPY] = self.ram[BG2_V_SCROLL_COPY].wrapping_add(value);
    }

    pub(crate) fn subtract_bg2_h_copy_low(&mut self, value: u8) {
        self.ram[BG2_H_SCROLL_COPY] = self.ram[BG2_H_SCROLL_COPY].wrapping_sub(value);
    }

    pub(crate) fn add_bg2_h_copy2_signed(&mut self, value: i8) {
        let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add(value as i16 as u16);
        write_le_u16(self.ram, BG2_X_SCROLL, next);
    }

    pub(crate) fn add_bg2_v_copy2_signed(&mut self, value: i8) {
        let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add(value as i16 as u16);
        write_le_u16(self.ram, BG2_Y_SCROLL, next);
    }

    pub(crate) fn add_bg3_v_copy2_signed(&mut self, value: i8) {
        let next = read_le_u16(self.ram, BG3_V_SCROLL_COPY2).wrapping_add(value as i16 as u16);
        write_le_u16(self.ram, BG3_V_SCROLL_COPY2, next);
    }

    fn add_subpixel_scroll(&mut self, subpixel_addr: usize, scroll_addr: usize, value: u32) {
        let current = (read_le_u16(self.ram, subpixel_addr) as u32)
            | ((read_le_u16(self.ram, scroll_addr) as u32) << 16);
        let next = current.wrapping_add(value);
        write_le_u16(self.ram, subpixel_addr, next as u16);
        write_le_u16(self.ram, scroll_addr, (next >> 16) as u16);
    }

    fn subtract_subpixel_scroll(&mut self, subpixel_addr: usize, scroll_addr: usize, value: u32) {
        let current = (read_le_u16(self.ram, subpixel_addr) as u32)
            | ((read_le_u16(self.ram, scroll_addr) as u32) << 16);
        let next = current.wrapping_sub(value);
        write_le_u16(self.ram, subpixel_addr, next as u16);
        write_le_u16(self.ram, scroll_addr, (next >> 16) as u16);
    }

    pub(crate) fn clear_bg1_scroll_subpixels(&mut self) {
        write_le_u16(self.ram, BG1_H_SCROLL_SUBPIXEL, 0);
        write_le_u16(self.ram, BG1_V_SCROLL_SUBPIXEL, 0);
    }

    pub(crate) fn add_bg1_h_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_subpixel_scroll(
            BG1_H_SCROLL_SUBPIXEL,
            BG1_X_SCROLL,
            (subpixel as u32) | ((scroll as u32) << 16),
        );
    }

    pub(crate) fn add_bg1_v_live_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_subpixel_scroll(
            BG1_V_SCROLL_SUBPIXEL,
            BG1_Y_SCROLL,
            (subpixel as u32) | ((scroll as u32) << 16),
        );
    }

    pub(crate) fn subtract_bg1_v_live_subpixel(&mut self, value: u32) {
        self.subtract_subpixel_scroll(BG1_V_SCROLL_SUBPIXEL, BG1_Y_SCROLL, value);
    }

    pub(crate) fn add_bg1_h_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_bg1_h_live_subpixel(subpixel, scroll);
    }

    pub(crate) fn add_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.add_bg1_v_live_subpixel(subpixel, scroll);
    }

    pub(crate) fn subtract_bg1_v_copy2_subpixel(&mut self, subpixel: u16, scroll: u16) {
        self.subtract_subpixel_scroll(
            BG1_V_SCROLL_SUBPIXEL,
            BG1_Y_SCROLL,
            (subpixel as u32) | ((scroll as u32) << 16),
        );
    }

    pub(crate) fn set_bg1_h_subpixel(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_H_SCROLL_SUBPIXEL, value);
    }

    pub(crate) fn set_bg1_v_subpixel(&mut self, value: u16) {
        write_le_u16(self.ram, BG1_V_SCROLL_SUBPIXEL, value);
    }

    pub(crate) fn step_bg2_h_copy2_toward_cached(&mut self) {
        let h = read_le_u16(self.ram, BG2_X_SCROLL);
        let cached = read_le_u16(self.ram, BG2_H_SCROLL_COPY2_CACHED);
        if h != cached {
            write_le_u16(
                self.ram,
                BG2_X_SCROLL,
                if h < cached {
                    h.wrapping_add(1)
                } else {
                    h.wrapping_sub(1)
                },
            );
        }
    }

    pub(crate) fn step_bg2_v_copy2_toward_cached(&mut self) {
        let v = read_le_u16(self.ram, BG2_Y_SCROLL);
        let cached = read_le_u16(self.ram, BG2_V_SCROLL_COPY2_CACHED);
        if v != cached {
            write_le_u16(
                self.ram,
                BG2_Y_SCROLL,
                if v < cached {
                    v.wrapping_add(1)
                } else {
                    v.wrapping_sub(1)
                },
            );
        }
    }

    pub(crate) fn add_bg2_h_copy2(&mut self, value: u16) {
        let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add(value);
        write_le_u16(self.ram, BG2_X_SCROLL, next);
    }

    pub(crate) fn add_bg2_v_copy2(&mut self, value: u16) {
        let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add(value);
        write_le_u16(self.ram, BG2_Y_SCROLL, next);
    }

    pub(crate) fn add_bg2_copy2_for_axis_signed(&mut self, vertical: bool, value: i16) {
        if vertical {
            let next = read_le_u16(self.ram, BG2_Y_SCROLL).wrapping_add_signed(value);
            write_le_u16(self.ram, BG2_Y_SCROLL, next);
        } else {
            let next = read_le_u16(self.ram, BG2_X_SCROLL).wrapping_add_signed(value);
            write_le_u16(self.ram, BG2_X_SCROLL, next);
        }
    }

    pub(crate) fn copy_bg1_live_to_ppu_copy(&mut self) {
        copy_word(self.ram, BG1_H_SCROLL_COPY, BG1_X_SCROLL);
        copy_word(self.ram, BG1_V_SCROLL_COPY, BG1_Y_SCROLL);
    }

    pub(crate) fn copy_bg2_live_to_ppu_copy(&mut self) {
        copy_word(self.ram, BG2_H_SCROLL_COPY, BG2_X_SCROLL);
        copy_word(self.ram, BG2_V_SCROLL_COPY, BG2_Y_SCROLL);
    }

    pub(crate) fn copy_live_to_ppu_copy(&mut self) {
        self.copy_bg1_live_to_ppu_copy();
        self.copy_bg2_live_to_ppu_copy();
    }

    pub(crate) fn copy_bg2_live_to_bg1_live(&mut self) {
        copy_word(self.ram, BG1_X_SCROLL, BG2_X_SCROLL);
        copy_word(self.ram, BG1_Y_SCROLL, BG2_Y_SCROLL);
    }

    pub(crate) fn copy_bg2_h_live_to_bg1_h_live(&mut self) {
        copy_word(self.ram, BG1_X_SCROLL, BG2_X_SCROLL);
    }

    pub(crate) fn copy_bg2_v_live_to_bg1_v_live(&mut self) {
        copy_word(self.ram, BG1_Y_SCROLL, BG2_Y_SCROLL);
    }

    pub(crate) fn set_mapbak_main_tile_theme_index(&mut self, value: u8) {
        self.ram[MAPBAK_MAIN_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_mapbak_sprite_graphics_index(&mut self, value: u8) {
        self.ram[MAPBAK_SPRITE_GRAPHICS_INDEX] = value;
    }

    pub(crate) fn set_mapbak_aux_tile_theme_index(&mut self, value: u8) {
        self.ram[MAPBAK_AUX_TILE_THEME_INDEX] = value;
    }

    pub(crate) fn set_mapbak_bg1_x_offset(&mut self, value: u16) {
        write_le_u16(self.ram, MAPBAK_BG1_X_OFFSET, value);
    }

    pub(crate) fn set_mapbak_bg1_y_offset(&mut self, value: u16) {
        write_le_u16(self.ram, MAPBAK_BG1_Y_OFFSET, value);
    }

    pub(crate) fn set_mapbak_cgwsel(&mut self, value: u8) {
        self.ram[MAPBAK_CGWSEL] = value;
    }

    pub(crate) fn set_mapbak_cgwsel_word(&mut self, value: u16) {
        write_le_u16(self.ram, MAPBAK_CGWSEL, value);
    }

    pub(crate) fn set_mapbak_hdmaen(&mut self, value: u8) {
        self.ram[MAPBAK_HDMAEN] = value;
    }

    pub(crate) fn mapbak_palette_slice_mut(&mut self) -> &mut [u8] {
        &mut self.ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 0x200]
    }
}
