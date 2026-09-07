//! `ZeldaState` methods split out of `zelda_rtl.rs` by topic (scroll).
//! Mechanical move: bodies are unchanged; private methods became
//! `pub(super)` so the parent module and its siblings can call them.

use super::*;

impl ZeldaState {
    /// Materialize the hardware HDMA enable register for an isolated ROM run.
    ///
    /// The translated game owns the C `HDMAEN_copy` value in native display
    /// state, while `DmaState` retains the channel descriptors configured by
    /// `HdmaSetup`. Render-time HDMA already combines those two owners. ROM CPU
    /// timing shadows must do the same or they omit each enabled scanline's bus
    /// stall even though the C code and Snes9x both execute the transfer.
    pub(super) fn dma_with_native_hdma_enable(&self) -> DmaState {
        let mut dma = self.dma.clone();
        for (index, channel) in dma.channel.iter_mut().enumerate() {
            channel.hdma_active = self.game_state.display.is_hdma_channel_enabled(index);
        }
        dma
    }

    pub(crate) fn bg1_move_calc(&self) -> &Bg1MovementAccumulatorState {
        &self.game_state.player.bg1_movement_accumulator
    }

    // BG scroll copy2 (0xe0/0xe2/0xe6/0xe8) is owned solely by PpuScrollCopyState; these
    // legacy `set_bgN_{x,y}` names delegate to it so the ~80 callers stay unchanged.
    pub(crate) fn set_bg1_x(&mut self, value: u16) {
        self.set_bg1_h_copy2(value);
    }

    pub(crate) fn set_bg1_x_low(&mut self, value: u8) {
        let hi = self.game_state.display.ppu_scroll_copy.bg1_h_copy2() & 0xff00;
        self.set_bg1_h_copy2(hi | u16::from(value));
    }

    pub(crate) fn set_bg1_y(&mut self, value: u16) {
        self.set_bg1_v_copy2(value);
    }

    pub(crate) fn set_bg1_y_low(&mut self, value: u8) {
        let hi = self.game_state.display.ppu_scroll_copy.bg1_v_copy2() & 0xff00;
        self.set_bg1_v_copy2(hi | u16::from(value));
    }

    pub(crate) fn set_bg2_x(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
    }

    pub(crate) fn set_bg2_y(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
    }

    pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
        self.world_scroll_mut().set_bg1_x_offset(value);
    }

    pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
        self.world_scroll_mut().set_bg1_y_offset(value);
    }

    pub(crate) fn world_camera_boundaries_mut(
        &mut self,
    ) -> NativeWorldCameraBoundariesBridgeMut<'_> {
        NativeWorldCameraBoundariesBridgeMut::new(
            &mut self.game_state.world.camera_boundaries,
            &mut self.ram,
        )
    }

    pub(crate) fn set_allow_scroll_z(&mut self, value: u8) {
        self.world_transient_mut().set_allow_scroll_z(value);
    }

    pub(crate) fn set_mode7_zoom_step_counter(&mut self, value: u8) {
        self.overworld_map_zoom_mut().set_step_counter(value);
    }

    pub(crate) fn mode7_zoom_timer(&self) -> u8 {
        self.game_state.world.overworld.map_zoom.timer
    }

    pub(crate) fn set_mode7_zoom_timer(&mut self, value: u8) {
        self.overworld_map_zoom_mut().set_timer(value);
    }

    pub(crate) fn overworld_right_bottom_scroll_bound(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .screen_size
            .right_bottom_bound_word()
    }

    pub(crate) fn overworld_vertical_scroll_delta_low(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .vertical_delta_low_byte()
    }

    pub(crate) fn overworld_horizontal_scroll_delta_low(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .horizontal_delta_low_byte()
    }

    pub(crate) fn overworld_vertical_scroll_delta(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .vertical_delta_word()
    }

    pub(crate) fn set_overworld_vertical_scroll_delta_low(&mut self, value: u8) {
        self.overworld_scroll_delta_mut()
            .set_vertical_delta_low_byte(value);
    }

    pub(crate) fn set_overworld_horizontal_scroll_delta_low(&mut self, value: u8) {
        self.overworld_scroll_delta_mut()
            .set_horizontal_delta_low_byte(value);
    }

    pub(crate) fn set_overworld_vertical_scroll_delta(&mut self, value: u16) {
        self.overworld_scroll_delta_mut()
            .set_vertical_delta_word(value);
    }

    pub(crate) fn set_overworld_horizontal_scroll_delta(&mut self, value: u16) {
        self.overworld_scroll_delta_mut()
            .set_horizontal_delta_word(value);
    }

    pub(crate) fn clear_overworld_vertical_scroll_delta_low(&mut self) {
        self.overworld_scroll_delta_mut()
            .clear_vertical_delta_low_byte();
    }

    pub(crate) fn set_bg12_window_selection(&mut self, value: u8) {
        self.display_core_mut().set_bg12_window_selection(value);
    }

    pub(crate) fn set_bg34_window_selection(&mut self, value: u8) {
        self.display_core_mut().set_bg34_window_selection(value);
    }

    pub(crate) fn set_object_color_window_selection(&mut self, value: u8) {
        self.display_core_mut()
            .set_object_color_window_selection(value);
    }

    pub(crate) fn set_main_screen_window_layers(&mut self, value: u8) {
        self.display_core_mut().set_main_screen_window_layers(value);
    }

    pub(crate) fn set_sub_screen_window_layers(&mut self, value: u8) {
        self.display_core_mut().set_sub_screen_window_layers(value);
    }

    pub(crate) fn set_window_layer_masks(
        &mut self,
        bg12_window_selection: u8,
        bg34_window_selection: u8,
        object_color_window_selection: u8,
        main_screen_window_layers: u8,
        sub_screen_window_layers: u8,
    ) {
        self.display_core_mut().set_window_layer_masks(
            bg12_window_selection,
            bg34_window_selection,
            object_color_window_selection,
            main_screen_window_layers,
            sub_screen_window_layers,
        );
    }

    pub(crate) fn clear_window_layer_masks(&mut self) {
        self.display_core_mut().clear_window_layer_masks();
    }

    pub(crate) fn clear_window_main_sub_masks(&mut self) {
        self.display_core_mut().clear_window_main_sub_masks();
    }

    pub(crate) fn set_hdma_enable_mask(&mut self, value: u8) {
        self.display_core_mut().set_hdma_enable_mask(value);
    }

    pub(crate) fn clear_hdma_enable_mask(&mut self) {
        self.display_core_mut().clear_hdma_enable_mask();
    }

    pub(crate) fn clear_dungeon_map_floor_scroll_step(&mut self) {
        self.dungeon_map_mut().clear_dungmap_floor_scroll_step();
    }

    pub(crate) fn increment_dungeon_map_floor_scroll_step(&mut self) {
        self.dungeon_map_mut().increment_dungmap_floor_scroll_step();
    }

    pub(crate) fn set_dungeon_map_scroll_draw_offset(&mut self, value: u16) {
        self.dungeon_map_mut().set_scroll_draw_offset(value);
    }

    pub(crate) fn set_dungeon_map_scroll_input(&mut self, value: u16) {
        self.dungeon_map_mut().set_scroll_input(value);
    }

    pub(crate) fn set_dungeon_map_scroll_target_y(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_scroll_target_y(value);
    }

    pub(crate) fn clear_dungeon_map_scroll_state(&mut self) {
        self.dungeon_map_mut().clear_scroll_state();
    }

    pub(crate) fn set_color_window_selection(&mut self, value: u8) {
        self.palette_filter_mut().set_color_window_selection(value);
    }

    pub(crate) fn set_color_window_and_math_word(&mut self, value: u16) {
        self.palette_filter_mut()
            .set_color_window_and_math_word(value);
    }

    pub(crate) fn dungeon_bg2_attribute_table(&self) -> &[u8] {
        self.game_state
            .display
            .dungeon_bg2_attribute_table(&self.ram)
    }

    pub(crate) fn dungeon_bg1_attribute_table(&self) -> &[u8] {
        self.game_state
            .display
            .dungeon_bg1_attribute_table(&self.ram)
    }

    pub(crate) fn set_select_file_name_scroll_x(&mut self, value: u16) {
        self.select_file_menu_mut().set_name_scroll_x(value);
    }

    pub(crate) fn set_select_file_name_scroll_x_step(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_scroll_x_step(value);
    }

    pub(crate) fn advance_select_file_name_scroll_x_step_by(&mut self, value: u8) -> u8 {
        self.select_file_menu_mut()
            .advance_name_scroll_x_step_by(value)
    }

    pub(crate) fn clear_select_file_name_scroll_y_step(&mut self) {
        self.select_file_menu_mut().clear_name_scroll_y_step();
    }

    pub(crate) fn increment_select_file_name_scroll_y_step(&mut self) -> u8 {
        self.select_file_menu_mut().increment_name_scroll_y_step()
    }

    pub(crate) fn set_select_file_name_scroll_x_direction(&mut self, value: u8) {
        self.select_file_menu_mut()
            .set_name_scroll_x_direction(value);
    }

    /// Single-byte HDMA scratch (0x654) that the C-style snapshot save/restore
    /// also leaves divergent on resume (a known tiny artifact). Restoring it keeps
    /// resume byte-faithful; it is otherwise transient HDMA scratch.
    /// Returns the pristine scratch bytes: the contiguous 0x1b00 region followed by
    /// one trailing byte for 0x654.
    pub fn saveload_hdma_scratch_bytes(&self) -> Vec<u8> {
        SpotlightHdmaState::saveload_scratch_bytes(&self.ram)
    }

    pub fn hdma_dynamic_table_bytes(&self) -> Vec<u8> {
        SpotlightHdmaState::dynamic_table_bytes(&self.ram)
    }

    pub(super) fn publish_bg_scroll_for_following_scanout(
        &mut self,
        scroll: BgScrollRegisterScanout,
    ) {
        scroll.publish_to(&mut self.ppu);
    }

    pub(super) fn simple_hdma_get_ptr(&self, p: u32) -> Option<Vec<u8>> {
        Self::simple_hdma_get_ptr_from_ram(&self.ram, p)
    }

    pub(super) fn simple_hdma_get_ptr_from_ram(ram: &[u8], p: u32) -> Option<Vec<u8>> {
        match p {
            0x0cfa87 => Some(ATTRACT_BG_DMA_SETUP.to_vec()),
            0x0cfa94 => Some(ATTRACT_TILEMAP_DMA_SETUP.to_vec()),
            0x0ebd53 => Some(ENDING_HDMA_SETUP.to_vec()),
            0x00f2fb => Some(SPOTLIGHT_INDIRECT_HDMA_SETUP.to_vec()),
            0x0abdcf => Some(MAP_MODE_HDMA_SETUP_NEAR.to_vec()),
            0x0abdd6 => Some(MAP_MODE_HDMA_SETUP_FAR.to_vec()),
            0x0abddd => Some(ATTRACT_INDIRECT_HDMA_SETUP.to_vec()),
            0x02c80c => Some(PRAYING_SCENE_HDMA_SETUP.to_vec()),
            0x001b00 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC, 0x1e0)),
            0x001be0 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC + 0xe0, 0x100)),
            0x001bf0 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC + 0xf0, 0xf0)),
            0x0add27 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_NEAR, 0)),
            0x0ade07 => Some(Self::u16_table_bytes(
                &MAP_MODE_PERSPECTIVE_ZOOMS_NEAR,
                0xe0,
            )),
            0x0adee7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0)),
            0x0adfc7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0xe0)),
            0x000600 => Some(Self::ram_bytes_from(ram, DEBUG_ROOM_BOUNDS_TOP, 2)),
            0x000602 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_Y_END, 2)),
            0x000604 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_X_START, 2)),
            0x000606 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_X_END, 2)),
            0x0000e2 => Some(Self::ram_bytes_from(
                ram,
                PpuScrollCopyState::bg2_h_copy2_offset(),
                2,
            )),
            _ => None,
        }
    }

    pub(super) fn simple_hdma_init(&self, c: &mut SimpleHdma, dc: &DmaChannel) {
        Self::simple_hdma_init_from_ram(&self.ram, c, dc);
    }

    pub(super) fn simple_hdma_init_from_ram(ram: &[u8], c: &mut SimpleHdma, dc: &DmaChannel) {
        if !dc.hdma_active {
            c.table = None;
            return;
        }
        c.table =
            Self::simple_hdma_get_ptr_from_ram(ram, dc.a_adr as u32 | ((dc.a_bank as u32) << 16));
        c.table_pos = 0;
        c.indir.clear();
        c.indir_pos = 0;
        c.rep_count = 0;
        c.mode = dc.mode | ((dc.indirect as u8) << 6);
        c.ppu_addr = dc.b_adr;
        c.indir_bank = dc.ind_bank;
    }

    pub(super) fn simple_hdma_table_byte(c: &mut SimpleHdma) -> Option<u8> {
        let table = c.table.as_ref()?;
        let value = table.get(c.table_pos).copied()?;
        c.table_pos += 1;
        Some(value)
    }

    pub(super) fn simple_hdma_line_writes(
        ram: &[u8],
        c: &mut SimpleHdma,
    ) -> ([(u32, u8); 4], usize) {
        let mut writes = [(0, 0); 4];
        if c.table.is_none() {
            return (writes, 0);
        }

        let mut do_transfer = false;
        if c.rep_count & 0x7f == 0 {
            let Some(rep_count) = Self::simple_hdma_table_byte(c) else {
                c.table = None;
                return (writes, 0);
            };
            c.rep_count = rep_count;
            if c.rep_count == 0 {
                c.table = None;
                return (writes, 0);
            }
            if c.mode & 0x40 != 0 {
                let Some(lo) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return (writes, 0);
                };
                let Some(hi) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return (writes, 0);
                };
                c.indir = Self::simple_hdma_get_ptr_from_ram(
                    ram,
                    ((c.indir_bank as u32) << 16) | lo as u32 | ((hi as u32) << 8),
                )
                .unwrap_or_default();
                c.indir_pos = 0;
            }
            do_transfer = true;
        }

        let mut write_count = 0;
        if do_transfer || c.rep_count & 0x80 != 0 {
            for j in 0..SIMPLE_HDMA_TRANSFER_LENGTH[(c.mode & 7) as usize] {
                let value = if c.mode & 0x40 != 0 {
                    let value = c.indir.get(c.indir_pos).copied().unwrap_or(0);
                    c.indir_pos += 1;
                    value
                } else {
                    Self::simple_hdma_table_byte(c).unwrap_or(0)
                };
                let offset = SIMPLE_HDMA_B_ADR_OFFSETS[(c.mode & 7) as usize][j];
                let adr = 0x2100 + c.ppu_addr.wrapping_add(offset) as u32;
                writes[write_count] = (adr, value);
                write_count += 1;
            }
        }
        c.rep_count = c.rep_count.wrapping_sub(1);
        (writes, write_count)
    }

    pub(super) fn simple_hdma_do_line(&mut self, c: &mut SimpleHdma) {
        let (writes, write_count) = Self::simple_hdma_line_writes(&self.ram, c);
        for (address, value) in writes.into_iter().take(write_count) {
            self.zelda_ppu_write(address, value);
        }
    }

    /// Simulate all 8 HDMA channels for 224 scanlines and capture window 1
    /// left/right boundaries per scanline.
    ///
    /// Used by the GPU renderer to reconstruct the HDMA-driven spotlight oval.
    /// Saves and restores all PPU state so the actual render call is unaffected.
    /// Simulate 224 HDMA scanlines and capture per-scanline window boundaries and
    /// main-screen layer-enable register (TM / screen_enabled[0]).
    ///
    /// Returns `(window1_left, window1_right, window2_left, window2_right,
    /// screen_enabled_main, bg_h_scroll, bg_v_scroll, mode7_matrix)` per scanline.
    /// ALttP writes TM via HDMA to enable/disable layers (OBJ, BG3, etc.) on a
    /// per-scanline basis, and can update BG scroll during rendering; the GPU
    /// uses this to match the CPU's per-row rendering.
    ///
    /// PPU and DMA latches are restored after capture, but a one-shot V-counter
    /// IRQ is consumed just as it is by `zelda_draw_ppu_frame` and real hardware.
    pub fn ppu_scanline_windows(
        &mut self,
    ) -> Box<[(u8, u8, u8, u8, u8, [u16; 4], [u16; 4], [i16; 8], bool); 224]> {
        let saved_channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.game_state.display.is_hdma_channel_enabled(i);
        }

        let saved_cgram = self.ppu.cgram.clone();
        let saved_cgram_pointer = self.ppu.cgram_pointer;
        let saved_cgram_second_write = self.ppu.cgram_second_write;
        let saved_cgram_buffer = self.ppu.cgram_buffer;
        let saved_forced_blank = self.ppu.forced_blank;
        let saved_brightness = self.ppu.brightness;
        let saved_screen_enabled = self.ppu.screen_enabled;
        let saved_window1_left = self.ppu.window1_left;
        let saved_window1_right = self.ppu.window1_right;
        let saved_window2_left = self.ppu.window2_left;
        let saved_window2_right = self.ppu.window2_right;
        let saved_scroll_prev = self.ppu.scroll_prev;
        let saved_scroll_prev2 = self.ppu.scroll_prev2;
        let saved_bg_scrolls: [(u16, u16); 4] =
            std::array::from_fn(|i| (self.ppu.bg_layer[i].h_scroll, self.ppu.bg_layer[i].v_scroll));
        let saved_m7_matrix = self.ppu.m7_matrix;
        let saved_m7_prev = self.ppu.m7_prev;

        let mut hdma_chans = [SimpleHdma::default(), SimpleHdma::default()];
        self.simple_hdma_init(&mut hdma_chans[0], &self.dma.channel[6]);
        self.simple_hdma_init(&mut hdma_chans[1], &self.dma.channel[7]);

        let mut result = Box::new(
            [(
                0u8, 0u8, 0u8, 0u8, 0u8, [0u16; 4], [0u16; 4], [0i16; 8], false,
            ); 224],
        );
        for line in 0..=224usize {
            if line == 128 && self.game_state.display.has_irq_control_flag() {
                let name_scroll_x = self.game_state.messaging.select_file_menu.name_scroll_x();
                self.zelda_ppu_write(0x2111, name_scroll_x as u8);
                self.zelda_ppu_write(0x2111, (name_scroll_x >> 8) as u8);
                self.zelda_ppu_write(0x2112, 0);
                self.zelda_ppu_write(0x2112, 0);
            }

            if (1..=224).contains(&line) {
                result[line - 1] = (
                    self.ppu.window1_left,
                    self.ppu.window1_right,
                    self.ppu.window2_left,
                    self.ppu.window2_right,
                    self.ppu.screen_enabled[0],
                    std::array::from_fn(|i| self.ppu.bg_layer[i].h_scroll),
                    std::array::from_fn(|i| self.ppu.bg_layer[i].v_scroll),
                    self.ppu.m7_matrix,
                    line - 1 < usize::from(self.ppu.forced_blank_scanlines)
                        || self
                            .ppu
                            .forced_blank_from_scanline
                            .is_some_and(|start| line > usize::from(start))
                        || (self.ppu.forced_blank
                            && self.ppu.forced_blank_from_scanline.is_none()
                            && self.ppu.forced_blank_scanlines == 0),
                );
            }

            self.simple_hdma_do_line(&mut hdma_chans[0]);
            self.simple_hdma_do_line(&mut hdma_chans[1]);
        }

        self.dma.channel = saved_channels;
        self.ppu.cgram = saved_cgram;
        self.ppu.cgram_pointer = saved_cgram_pointer;
        self.ppu.cgram_second_write = saved_cgram_second_write;
        self.ppu.cgram_buffer = saved_cgram_buffer;
        self.ppu.forced_blank = saved_forced_blank;
        self.ppu.brightness = saved_brightness;
        self.ppu.screen_enabled = saved_screen_enabled;
        self.ppu.window1_left = saved_window1_left;
        self.ppu.window1_right = saved_window1_right;
        self.ppu.window2_left = saved_window2_left;
        self.ppu.window2_right = saved_window2_right;
        self.ppu.scroll_prev = saved_scroll_prev;
        self.ppu.scroll_prev2 = saved_scroll_prev2;
        for (i, &(h_scroll, v_scroll)) in saved_bg_scrolls.iter().enumerate() {
            self.ppu.bg_layer[i].h_scroll = h_scroll;
            self.ppu.bg_layer[i].v_scroll = v_scroll;
        }
        self.ppu.m7_matrix = saved_m7_matrix;
        self.ppu.m7_prev = saved_m7_prev;

        if let Some(authority) = self.active_presented_bg_scroll.as_ref() {
            let mut mismatched_scanline_layers = 0;
            let mut first_mismatch = None;
            for (line, (native, authority)) in
                result.iter_mut().zip(authority.scanlines()).enumerate()
            {
                for layer in 0..crate::PresentedBgScroll::LAYER_COUNT {
                    if [native.5[layer], native.6[layer]] != authority[layer] {
                        mismatched_scanline_layers += 1;
                        first_mismatch.get_or_insert((line as u16, layer as u8));
                    }
                    native.5[layer] = authority[layer][0];
                    native.6[layer] = authority[layer][1];
                }
            }
            self.original_timing_bg_scroll_shadow_result =
                Some(crate::OriginalTimingBgScrollShadowResult {
                    compared_scanline_layers: crate::PresentedBgScroll::VISIBLE_LINES
                        * crate::PresentedBgScroll::LAYER_COUNT,
                    mismatched_scanline_layers,
                    first_mismatch,
                });
        }

        if let Some(authority) = self.active_presented_mode7_transform.as_ref() {
            let mut mismatched_scanline_fields = 0;
            let mut first_mismatch = None;
            for (line, (native, authority)) in
                result.iter_mut().zip(authority.scanlines()).enumerate()
            {
                for field in 0..crate::PresentedMode7Transform::FIELD_COUNT {
                    if native.7[field] != authority[field] {
                        mismatched_scanline_fields += 1;
                        first_mismatch.get_or_insert((line as u16, field as u8));
                    }
                }
                native.7 = *authority;
            }
            self.original_timing_mode7_transform_shadow_result =
                Some(crate::OriginalTimingMode7TransformShadowResult {
                    compared_scanline_fields: crate::PresentedMode7Transform::VISIBLE_LINES
                        * crate::PresentedMode7Transform::FIELD_COUNT,
                    mismatched_scanline_fields,
                    first_mismatch,
                });
        }

        if let Some(authority) = self.active_presented_window_mask.as_ref() {
            let mut mismatched_scanline_windows = 0;
            let mut first_mismatch = None;
            for (line, (native, authority)) in
                result.iter_mut().zip(authority.scanlines()).enumerate()
            {
                let native_windows = [[native.0, native.1], [native.2, native.3]];
                for window in 0..crate::PresentedWindowMask::WINDOW_COUNT {
                    if native_windows[window] != authority[window] {
                        mismatched_scanline_windows += 1;
                        first_mismatch.get_or_insert((line as u16, window as u8));
                    }
                }
                native.0 = authority[0][0];
                native.1 = authority[0][1];
                native.2 = authority[1][0];
                native.3 = authority[1][1];
            }
            let mismatched_screen_masks = self
                .original_timing_window_mask_shadow_result
                .map_or(0, |shadow| shadow.mismatched_screen_masks);
            self.original_timing_window_mask_shadow_result =
                Some(crate::OriginalTimingWindowMaskShadowResult {
                    compared_scanline_windows: crate::PresentedWindowMask::VISIBLE_LINES
                        * crate::PresentedWindowMask::WINDOW_COUNT,
                    mismatched_scanline_windows,
                    mismatched_screen_masks,
                    first_mismatch,
                });
        }

        result
    }

    pub(super) fn set_mode7_perspective_correction(&mut self, low: u16, high: u16) {
        self.ppu.mode7_perspective_low = if low != 0 { 1.0 / low as f32 } else { 0.0 };
        self.ppu.mode7_perspective_high = if high != 0 { 1.0 / high as f32 } else { 0.0 };
    }

    pub(super) fn hdma_setup(
        &mut self,
        addr6: u32,
        addr7: u32,
        transfer_unit: u8,
        reg6: u8,
        reg7: u8,
        indirect_bank: u8,
    ) {
        if addr6 != 0 {
            let ch = &mut self.dma.channel[6];
            ch.mode = transfer_unit & 7;
            ch.fixed = transfer_unit & 8 != 0;
            ch.decrement = transfer_unit & 0x10 != 0;
            ch.unused_bit = transfer_unit & 0x20 != 0;
            ch.indirect = transfer_unit & 0x40 != 0;
            ch.from_b = transfer_unit & 0x80 != 0;
            ch.b_adr = reg6;
            ch.a_adr = addr6 as u16;
            ch.a_bank = (addr6 >> 16) as u8;
            ch.ind_bank = indirect_bank;
        }

        let ch = &mut self.dma.channel[7];
        ch.mode = transfer_unit & 7;
        ch.fixed = transfer_unit & 8 != 0;
        ch.decrement = transfer_unit & 0x10 != 0;
        ch.unused_bit = transfer_unit & 0x20 != 0;
        ch.indirect = transfer_unit & 0x40 != 0;
        ch.from_b = transfer_unit & 0x80 != 0;
        ch.b_adr = reg7;
        ch.a_adr = addr7 as u16;
        ch.a_bank = (addr7 >> 16) as u8;
        ch.ind_bank = indirect_bank;
    }

    pub(super) fn finish_ground_movement_camera_tail(&mut self) {
        self.follower_link_state_mut().clear_pit_correction();
        if self.apply_links_movement_to_camera_called && self.game_state.enhanced_features.has(4096)
        {
            return;
        }
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn cache_camera_properties_for_player(&mut self) {
        let bg2_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.cache_bg2_live_scroll_from(bg2_x, bg2_y);
        self.follower_link_state_mut().cache_current_position();
        let y_start = self.game_state.world.room_bounds.y_bound(0);
        let y_end = self.game_state.world.room_bounds.y_bound(2);
        let x_start = self.game_state.world.room_bounds.x_bound(0);
        let x_end = self.game_state.world.room_bounds.x_bound(2);
        self.set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.cache_scroll_targets();
        self.cache_camera_scroll();
        self.cache_quadrant_fullsize_state();
        self.follower_link_state_mut().cache_current_quadrants();
        self.follower_link_state_mut().cache_facing();
        self.follower_link_state_mut().cache_lower_level_states();
        let doorway_state = self.game_state.player.follower_link.doorway_state();
        self.cache_standing_in_doorway(doorway_state);
        self.dungeon_stair_movement_mut().cache_current_floor();
    }

    pub fn mode7_character_source(&self) -> Option<&[u8]> {
        self.asset_raw(66)
    }
}
