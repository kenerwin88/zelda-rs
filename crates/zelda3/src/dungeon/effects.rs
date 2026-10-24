//! Dungeon layer effects, water, lighting, palettes, and maiden graphics.

use super::*;
use crate::tile_definition::{NativeTile, TilePair};
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

impl ZeldaState {

    pub(in crate::zelda_rtl) fn LoadOWMusicIfNeeded(&mut self) -> bool {
        if !self.resident_song_bank_is_dungeon() {
            return false;
        }
        self.select_overworld_song_bank();
        self.load_overworld_songs();
        true
    }

    /// ROM: `Dungeon_LoadSongBankIfNeeded`
    pub(super) fn dungeon_load_song_bank_if_needed(&mut self) -> bool {
        let queued = self.game_state.system_signals.queued_music_control();
        if queued == 0xff || queued == 0xf2 {
            return false;
        }
        if queued == 3 || queued == 7 || queued == 14 {
            self.LoadOWMusicIfNeeded()
        } else {
            if self.resident_song_bank_is_dungeon() {
                return false;
            }
            self.select_dungeon_song_bank();
            self.load_dungeon_songs();
            true
        }
    }

    /// ROM: `ApplyGrayscaleFixed_Incremental`
    pub(super) fn apply_grayscale_fixed_incremental(&mut self) {
        let mut a = self.game_state.display.palette_filter.fixed_color_red() & 0x1f;
        let target = self.game_state.dungeon.room_effects.fixed_color_plusminus();
        if a == target {
            return;
        }
        if a < target {
            a = a.wrapping_add(1);
        } else {
            a = a.wrapping_sub(1);
        }
        self.dungeon_approach_fixed_color_variable(a);
    }

    /// ROM: `Dungeon_ApproachFixedColor_variable`
    pub(in crate::zelda_rtl) fn dungeon_approach_fixed_color_variable(&mut self, a: u8) {
        self.set_fixed_color_red(a | 0x20);
        self.set_fixed_color_green(a | 0x40);
        self.set_fixed_color_blue(a | 0x80);
    }

    /// ROM: `Dungeon_DoubleApplyAndIncrementGrayscale`
    pub(in crate::zelda_rtl) fn dungeon_double_apply_and_increment_grayscale(&mut self) {
        self.apply_palette_filter_bounce();
        if self.rom_startup_timing() {
            self.schedule_pre_main_caller_continuation(
                PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
            );
            return;
        }
        self.complete_spiral_stairs_second_grayscale_palette_filter();
    }

    pub(in crate::zelda_rtl) fn complete_spiral_stairs_second_grayscale_palette_filter(&mut self) {
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        // This translated helper is shared by all three dungeon stair
        // transitions. Its caller may have advanced the state byte before the
        // pre-main return is observed, so the stable invariant is the caller
        // family rather than one historical subsubmodule value.
        debug_assert!(matches!(
            self.game_state.frame.submodule,
            0x0e | 0x11 | 0x12
        ));
        self.apply_palette_filter_bounce();
        self.apply_grayscale_fixed_incremental();
    }

    /// ROM: `Module07_0A_ChangeBrightness`
    pub(in crate::zelda_rtl) fn module07_0a_change_brightness(&mut self) {
        self.OrientLampLightCone();
        self.apply_grayscale_fixed_incremental();
        if self.game_state.display.palette_filter.fixed_color_red() & 0x1f
            != self.game_state.dungeon.room_effects.fixed_color_plusminus()
        {
            return;
        }
        self.set_submodule(0);
        self.set_subsubmodule(0);
    }

    /// ROM: `Module07_0B_DrainSwampPool`
    pub(in crate::zelda_rtl) fn module07_0b_drain_swamp_pool(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                if self
                    .game_state
                    .dungeon
                    .moving_floor
                    .water_transition_counter()
                    & 7
                    == 0
                {
                    let k = ((self
                        .game_state
                        .dungeon
                        .moving_floor
                        .water_transition_counter()
                        >> 2)
                        & 3) as usize;
                    if self.game_state.display.water_hdma_window.window_y_radius()
                        == self.game_state.display.water_hdma_window.window_y_target()
                    {
                        self.dungeon_set_attr_for_activated_water_off();
                        return;
                    }
                    let delta = MODULE07_0_B_DRAIN_SWAMP_POOL_SWAMP_DRAIN_WINDOW_RADIUS_DELTAS[k]
                        as i16 as u16;
                    let y_radius = self.game_state.display.water_hdma_window.window_y_radius()
                        .wrapping_add(delta);
                    let x_radius = self.game_state.display.water_hdma_window.window_x_radius()
                        .wrapping_add(delta);
                    self.water_hdma_window_mut().set_window_y_radius(y_radius);
                    self.water_hdma_window_mut().set_window_x_radius(x_radius);
                }
                self.dungeon_moving_floor_mut()
                    .increment_water_transition_counter();
                self.AdjustWaterHDMAWindow();
            }
            1 => {
                let tile = self.tile_word(0x01e0, 0);
                for i in 0..0x1000usize {
                    self.dungeon_room_tilemaps_mut().set_bg1_tile(i, tile);
                }
                self.dungeon_room_load_mut().clear_quadrant_upload_index();
                self.increment_subsubmodule();
            }
            2..=5 => self.dungeon_flood_swamp_water_prep_tile_map(),
            _ => {}
        }
    }

    /// ROM: `Module07_0C_FloodSwampWater`
    pub(in crate::zelda_rtl) fn module07_0c_flood_swamp_water(&mut self) {
        match self.game_state.frame.subsubmodule {
            0..=3 => self.dungeon_flood_swamp_water_prep_tile_map(),
            4..=8 => {
                if self
                    .dungeon_moving_floor_mut()
                    .decrement_water_transition_counter()
                    == 0
                {
                    self.dungeon_moving_floor_mut()
                        .set_water_transition_counter(4);
                    self.increment_subsubmodule();
                    let depth = i32::from(self.game_state.frame.subsubmodule) - 4;
                    self.water_hdma_window_mut().set_window_x_radius(8);
                    self.water_hdma_window_mut().set_window_y_radius_alt(0);
                    self.water_hdma_window_mut().set_window_y_radius(0x30);
                    self.dungeon_adjust_water_vomit(0x1654 + 0x10, depth);
                }
            }
            9 => {
                self.set_bg12_window_selection(3);
                self.set_bg34_window_selection(0);
                self.set_object_color_window_selection(0);
                self.set_main_screen_window_layers(22);
                self.set_sub_screen_window_layers(1);
                self.set_sub_screen_layers(1);
                self.set_color_window_selection(2);
                self.set_color_math_control(98);
                self.dungeon_moving_floor_mut()
                    .set_water_transition_counter(0);
                self.increment_subsubmodule();
                self.module07_0c_flood_swamp_water_raise_window(
                    MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_LEFT_DELTAS,
                    MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_RIGHT_DELTAS,
                );
            }
            10 => self.module07_0c_flood_swamp_water_raise_window(
                MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_LEFT_DELTAS,
                MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_WINDOW_RIGHT_DELTAS,
            ),
            11 => {
                if self
                    .game_state
                    .dungeon
                    .moving_floor
                    .water_transition_counter()
                    & 7
                    == 0
                {
                    let k = ((self
                        .game_state
                        .dungeon
                        .moving_floor
                        .water_transition_counter()
                        >> 2)
                        & 3) as usize;
                    if self.game_state.display.water_hdma_window.window_y_radius()
                        == self.game_state.display.water_hdma_window.window_y_target()
                    {
                        self.dungeon_set_attr_for_activated_water();
                        return;
                    }
                    let delta = MODULE07_0_C_FLOOD_SWAMP_WATER_SWAMP_FILL_FINAL_RADIUS_DELTAS[k]
                        as i16 as u16;
                    let y_radius = self.game_state.display.water_hdma_window.window_y_radius()
                        .wrapping_add(delta);
                    let x_radius = self.game_state.display.water_hdma_window.window_x_radius()
                        .wrapping_add(delta);
                    self.water_hdma_window_mut().set_window_y_radius(y_radius);
                    self.water_hdma_window_mut().set_window_x_radius(x_radius);

                    let a = self.game_state.display.water_hdma_window.window_y_target()
                        .wrapping_sub(y_radius);
                    if a == 0 || a == 8 {
                        self.dungeon_adjust_water_vomit(if a == 0 { 0x16b4 } else { 0x168c }, 5);
                    }
                }
                self.dungeon_moving_floor_mut()
                    .increment_water_transition_counter();
                self.AdjustWaterHDMAWindow();
            }
            _ => {}
        }
    }

    /// ROM: `Module07_0C_FloodSwampWater_raise_window`
    fn module07_0c_flood_swamp_water_raise_window(&mut self, tab0: [i8; 4], tab1: [i8; 4]) {
        let k = (self
            .game_state
            .dungeon
            .moving_floor
            .water_transition_counter()
            & 3) as usize;
        let r0 = 0x0688u16
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(0x24);
        let x_radius = self.game_state.display.water_hdma_window.window_x_radius()
            .wrapping_add(tab0[k] as i16 as u16);
        let y_span = self.game_state.display.water_hdma_window.window_y_radius_alt()
            .wrapping_add(tab1[k] as i16 as u16);
        self.water_hdma_window_mut().set_window_x_radius(x_radius);
        self.water_hdma_window_mut().set_window_y_radius_alt(y_span);
        if y_span >= r0 {
            self.dungeon_room_load_mut().set_bg2_properties(7);
            self.increment_subsubmodule();
        }
        self.dungeon_moving_floor_mut()
            .increment_water_transition_counter();
        let lower = 0x0688u16
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_sub(self.game_state.display.water_hdma_window.window_y_radius());
        self.set_spotlight_y_lower(lower);
        let upper = lower.wrapping_add(y_span);
        self.set_spotlight_y_upper(upper);
        self.adjust_water_hdma_window_x(upper);
    }

    /// ROM: `Module07_0D_FloodDam`
    pub(in crate::zelda_rtl) fn module07_0d_flood_dam(&mut self) {
        self.flood_dam_prep_flood_hdma();
        match self.game_state.frame.subsubmodule {
            0 => self.flood_dam_prep_tiles_init(),
            1..=3 => self.watergate_main_state1(),
            4 => self.flood_dam_expand(),
            5 => self.flood_dam_fill(),
            other => panic!("invalid Module07_0D_FloodDam subsubmodule_index {other}"),
        }
    }

    /// ROM: `WaterFlood_BuildOneQuadrantForVRAM`
    pub(in crate::zelda_rtl) fn water_flood_build_one_quadrant_for_vram(&mut self) {
        assert_ne!(self.game_state.dungeon.header.primary_header_tag(), 25);
        let _scope = crate::cycle_ledger::routine(0x00_91c4);
        // $00:91C4..91C8, BNE taken into TileMapPrep_NotWaterOnTag.
        crate::cycle_ledger::charge(56 + 6);
        self.tile_map_prep_not_water_on_tag_body();
    }

    /// ROM: `FloodDam_PrepTiles_init`
    pub(super) fn flood_dam_prep_tiles_init(&mut self) {
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        self.clear_screen_transition();
        self.water_flood_build_one_quadrant_for_vram();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
    }

    /// ROM: `Watergate_Main_State1`
    pub(super) fn watergate_main_state1(&mut self) {
        self.clear_screen_transition();
        self.water_flood_build_one_quadrant_for_vram();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_FloodSwampWater_PrepTileMap`
    pub(super) fn dungeon_flood_swamp_water_prep_tile_map(&mut self) {
        self.water_flood_build_one_quadrant_for_vram();
        self.dungeon_room_load_mut()
            .advance_quadrant_upload_index_by(4);
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule == 6 {
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
            self.set_subsubmodule(0);
            self.set_submodule(0);
        }
    }

    /// ROM: `Dungeon_AdjustWaterVomit`
    pub(super) fn dungeon_adjust_water_vomit(&mut self, src: usize, depth: i32) {
        let mut dsto = (self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_source_pos())
        .wrapping_add(xy(0, 2) as u16);
        let mut row = 0usize;
        let mut remaining = depth;
        loop {
            for x in 0..4 {
                let tile = self.tile_word(src, row * 4 + x);
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile(dsto as usize + x, tile);
            }
            dsto = dsto.wrapping_add(xy(0, 1) as u16);
            row += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }

        let base_dsto = (self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_source_pos())
        .wrapping_add(xy(0, 2) as u16);
        let mut upload = self.game_state.display.vram_upload_buffer_base();
        for i in 0..4u16 {
            let col = base_dsto.wrapping_add(i);
            let vram_addr = self.dungeon_map_vram_addr(col);
            self.write_vram_upload_absolute_word(upload, vram_addr);
            self.write_vram_upload_absolute_word(upload + 2, 0x0980);
            for y in 0..5usize {
                let tile = self
                    .game_state
                    .dungeon
                    .room_tilemaps
                    .bg2_tile(col as usize + y * 64);
                self.write_vram_upload_absolute_word(upload + 4 + y * 2, tile);
            }
            upload += 14;
        }
        self.write_vram_upload_absolute_word(upload, 0xffff);
        self.set_bg_vram_load_mode(1);
    }

    /// ROM: `FloodDam_Expand`
    pub(super) fn flood_dam_expand(&mut self) {
        let watergate_var1 = self.water_hdma_window_mut().increment_watergate_pointer();
        self.water_hdma_window_mut()
            .set_window_x_radius(u16::from(watergate_var1 >> 1));
        self.water_hdma_window_mut()
            .advance_watergate_window_y_radius();

        if watergate_var1 & 0x0f != 0 {
            return;
        }
        if watergate_var1 == 64 {
            self.increment_subsubmodule();
        }

        let src =
            FLOOD_DAM_EXPAND_WATERGATE_SRCS1[((watergate_var1 >> 4).wrapping_sub(1)) as usize];
        let dsto = self
            .game_state
            .display
            .water_hdma_window
            .watergate_tilemap_pos_x2()
            >> 1;
        for x in 0..10u16 {
            for y in 0..4u16 {
                let tile = self.tile_word(src, (x * 4 + y) as usize);
                self.room_write_bg2(dsto + x + y * 64, tile);
            }
        }

        let mut pos = self
            .game_state
            .display
            .water_hdma_window
            .watergate_tilemap_pos_x2();
        let mut dma_ptr = 0usize;
        for _ in 0..3 {
            dma_ptr = self.dungeon_prep_overlay_dma_watergate(dma_ptr, pos, 0x0881, 4);
            pos = pos.wrapping_add(6);
        }
        self.request_nmi_copy_packets();
    }

    /// ROM: `FloodDam_Fill`
    pub(super) fn flood_dam_fill(&mut self) {
        let t = self
            .water_hdma_window_mut()
            .increment_window_y_radius_byte()
            .wrapping_add(self.game_state.display.spotlight_hdma.y_upper() as u8);
        if t >= 225 {
            self.dungeon_room_load_mut().clear_quadrant_upload_index();
            self.set_submodule(0);
            self.set_subsubmodule(0);
            self.clear_window_main_sub_masks();
            self.iris_spotlight_reset_table();
        }
    }

    /// ROM: `TileMapPrep_NotWaterOnTag`
    pub(super) fn tile_map_prep_not_water_on_tag(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x00_91d3);
        self.tile_map_prep_not_water_on_tag_body();
    }

    fn tile_map_prep_not_water_on_tag_body(&mut self) {
        // $00:91D3..9251: the $00:913F copy over the BG1 tilemap, with the
        // 268-cycle tail (target page + $10) and its RTL.
        crate::cycle_ledger::charge(254 + 128 * 918 + 120 * 6 + 8 * 172 + 7 * 6 + 268);
        self.dungeon_prepare_next_room_quadrant_upload_from(
            ROOM_BG1_TILEMAP_BASE,
            self.game_state.dungeon.room_load.quadrant_upload_index() as usize,
            0x10,
        );
    }

    pub(in crate::zelda_rtl) fn OrientLampLightCone(&mut self) {
        // Cycle ledger: OrientLampLightCone $00:F567 (JSL target, m8 x8).
        // $00:F567 LDA $0458 : BEQ (48; taken +6 into the $00:F566 RTL, 44,
        // without the lantern), $00:F56C LDA $11 : CMP #$14 : BEQ (56; taken
        // +6 into the same RTL in submodule $14).
        let _scope = crate::cycle_ledger::routine(0x00_f567);
        let dark_with_lantern = self.game_state.dungeon.torch.dungeon_dark_with_lantern();
        let lamp_submodule = self.game_state.frame.submodule == 20;
        crate::cycle_ledger::charge(if !dark_with_lantern {
            48 + 6 + 44
        } else if lamp_submodule {
            48 + 56 + 6 + 44
        } else {
            48 + 56
        });
        if !dark_with_lantern || lamp_submodule {
            return;
        }

        // $00:F572-F581 REP #$30, the facing load into $00 and X, LDA $6C,
        // AND #$00FF, BEQ (196; taken +6 with no doorway).
        crate::cycle_ledger::charge(196);
        let a = self.game_state.player.follower_link.facing() >> 1;
        let mut idx = a;
        if self.game_state.player.follower_link.doorway_state() != 0 {
            idx = self.game_state.player.follower_link.doorway_state() & 0xfe;
            // $00:F583 AND #$00FE : ASL : TAX : BEQ (68; taken +6 for a
            // zero doorway index).
            if idx != 0 {
                // $00:F58A LDA $00 : CMP #$0004 : BCS (72); facing up/down
                // runs $00:F591-F59A the x + 8 low byte and BRA (116) into
                // $00:F5AB CMP #$0080 : BCC (40; taken +6 below $80, else
                // $00:F5B0 INX : INX, 28). Facing left/right takes the BCS
                // (+6) into $00:F59C TAX : BRA (36).
                if a < 2 {
                    let far = self.game_state.player.follower_link.x().wrapping_add(8) as u8 >= 0x80;
                    crate::cycle_ledger::charge(68 + 72 + 116 + 40 + if far { 28 } else { 6 });
                    idx = idx.wrapping_add(u8::from(far));
                } else {
                    crate::cycle_ledger::charge(68 + 72 + 6 + 36);
                    idx = a;
                }
            } else if a >= 2 {
                // BEQ taken (+6) into $00:F59F LDA $00 : CMP #$0004 : BCC
                // (72); facing left/right runs $00:F5A6 LDA $20 : AND #$00FF
                // (56) into the $00:F5AB test (40; +6 below $80, else 28).
                let far = self.game_state.player.follower_link.y() as u8 >= 0x80;
                crate::cycle_ledger::charge(68 + 6 + 72 + 56 + 40 + if far { 28 } else { 6 });
                idx = idx.wrapping_add(u8::from(far));
            } else {
                // The $00:F5A4 BCC taken (+6) into $00:F59C TAX : BRA (36).
                crate::cycle_ledger::charge(68 + 6 + 72 + 6 + 36);
                idx = a;
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        let idx = idx as usize;
        // $00:F5B2 CPX #$0004 : BCS (40; taken +6 into $00:F600 for a
        // left/right cone).
        crate::cycle_ledger::charge(40);
        if idx >= 4 {
            return;
        }

        // Either cone: the scroll arithmetic ending BPL (656; a negative
        // total runs LDA #$0000, 24, else the BPL is taken +6), CMP the
        // clamp : BCC (64; below the clamp takes it +6, else the clamp is
        // loaded, 48), then the baseline subtraction, the store, SEP #$30
        // and RTL (160): $00:F5B7-F5FF for the up/down cone, $00:F600-F648
        // for the left/right cone.
        let charge_clamp = |t: u16, clamp: u16| {
            let negative = (t as i16) < 0;
            let clamped_low = if negative { 0 } else { t };
            crate::cycle_ledger::charge(
                656 + if negative { 24 } else { 6 }
                    + 64
                    + if clamped_low < clamp { 6 } else { 48 }
                    + 160,
            );
        };
        if idx < 2 {
            let h = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(self.game_state.player.follower_link.x().wrapping_sub(0x77))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS[idx]);
            self.set_bg1_x(h);

            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(self.game_state.player.follower_link.y().wrapping_sub(0x58))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS[idx])
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS[idx] as u16)
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16);
            charge_clamp(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            let t = clamp_c_int16_to_u16(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            self.set_bg1_v_copy2(
                t.wrapping_sub(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16),
            );
        } else {
            crate::cycle_ledger::charge(6);
            let v = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(self.game_state.player.follower_link.y().wrapping_sub(0x72))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_Y_BASE_OFFSETS[idx]);
            self.set_bg1_y(v);

            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(self.game_state.player.follower_link.x().wrapping_sub(0x58))
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_BG1_X_BASE_OFFSETS[idx])
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_ADJUSTMENTS[idx] as u16)
                .wrapping_add(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16);
            charge_clamp(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            let t = clamp_c_int16_to_u16(t, ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_CLAMPS[idx]);
            self.set_bg1_h_copy2(
                t.wrapping_sub(ORIENT_LAMP_LIGHT_CONE_LAMP_CONE_SCROLL_BASELINES[idx] as u16),
            );
        }
    }

    /// ROM: `Module07_16_UpdatePegs`
    pub(in crate::zelda_rtl) fn module07_16_update_pegs(&mut self) {
        if self.lag_free {
            // $02:972A..9738: state increment/test; either early RTS $971F
            // or the shift/index/indirect dispatch. Children price their bodies.
            crate::cycle_ledger::charge(94
                + if self.game_state.frame.subsubmodule.wrapping_add(1) & 3 != 0 { 6 + 42 } else { 98 });
        }
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule & 3 != 0 {
            return;
        }
        match self.game_state.frame.subsubmodule >> 2 {
            0 | 1 => self.module07_16_update_pegs_step1(),
            2 => self.module07_16_update_pegs_step2(),
            3 => self.RecoverPegGFXFromMapping(),
            4 => {
                if self.lag_free {
                    crate::cycle_ledger::charge(152); // $02:97A9..97B1: JSL, clears, RTS.
                }
                if let Some(progress) =
                    self.take_original_timing_dungeon_peg_attribute_flip_progress()
                {
                    assert_eq!(
                        self.game_state.frame.subsubmodule, 0x10,
                        "peg-attribute flip progress requires Module07/$16 state $10",
                    );
                    self.dungeon_flip_crystal_peg_attribute_through_progress(progress);
                    assert!(
                        self.dungeon_peg_attribute_flip_pending
                            .replace(DungeonPegAttributeFlipContinuation {
                                caller: DungeonPegAttributeFlipCaller::UpdatePegs,
                                progress,
                            })
                            .is_none(),
                        "peg-attribute flip continuation was already pending",
                    );
                    match progress.boundary {
                        OriginalTimingBoundary::NmiAccepted => self
                            .game_execution_scheduler
                            .schedule_after_current_trailing_nmi(
                                GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                            ),
                        OriginalTimingBoundary::HostReturn => {
                            self.game_execution_scheduler.schedule_work(
                                GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                                1,
                            )
                        }
                    }
                    return;
                }
                self.dungeon_flip_crystal_peg_attribute();
                self.set_subsubmodule(0);
                self.set_submodule(0);
            }
            _ => {}
        }
    }

    /// ROM: `Module07_17_PressurePlate`
    pub(in crate::zelda_rtl) fn module07_17_pressure_plate(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule != 0 {
            return;
        }
        let link_y = self.game_state.player.follower_link.y().wrapping_sub(2);
        self.follower_link_state_mut().set_y(link_y);
        let pos = self
            .game_state
            .dungeon
            .room_items
            .replacement_tile_destination_x2();
        self.dungeon_update_tile_map_with_common_tile(
            (pos & 0x003f) << 3,
            (pos >> 3) & 0x01f8,
            0x0e,
        );
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_submodule(saved_module);
    }

    /// ROM: `Module07_18_RescuedMaiden`
    pub(in crate::zelda_rtl) fn module07_18_rescued_maiden(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_980a);
        // $02:980A LDA $B0, JSL JumpTableLocal; its PLY frame owns 52 cycles.
        crate::cycle_ledger::charge(24 + 62);
        crate::cycle_ledger::charge_routine(0x00_8781, 52);
        crate::cycle_ledger::charge(414 - 52);
        match self.game_state.frame.subsubmodule {
            0 => {
                // $02:9826..9840 PrepareForCrystalCutscene (JSL
                // PaletteFilter_RestoreBGSubstractiveStrict, colour copy, test).
                crate::cycle_ledger::charge(294);
                self.palette_filter_restore_bg_substractive_strict();
                self.copy_color(
                    (zelda3_palette::Bank::Main, 32),
                    (zelda3_palette::Bank::Main, 0),
                );
                if self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen()
                    != 0xff
                {
                    // BNE $9887 taken, RTS.
                    crate::cycle_ledger::charge(6 + 42);
                    return;
                }
                if let Some(progress) =
                    self.take_original_timing_rescued_maiden_tilemap_clear_progress()
                {
                    self.apply_rescued_maiden_tilemap_clear_stores(0, progress.completed_stores);
                    let work = GameWorkContinuation::FinishRescuedMaidenTilemapClear {
                        completed_stores: progress.completed_stores,
                    };
                    match progress.boundary {
                        crate::OriginalTimingBoundary::NmiAccepted => self
                            .game_execution_scheduler
                            .schedule_cpu_timed_work_returning_on_later_host(work, 1),
                        crate::OriginalTimingBoundary::HostReturn => {
                            self.game_execution_scheduler.schedule_work(work, 1);
                        }
                    }
                    return;
                }
                // $02:9842..9887: 1,024 passes of the eight-store clear (452,
                // BNE taken on all but the last), the scroll/quadrant resets
                // and the RTS.
                crate::cycle_ledger::charge(70 + 1_024 * 452 + 1_023 * 6 + 284 + 42);
                self.complete_rescued_maiden_tilemap_clear(0);
            }
            1 => {
                // $02:9888..9899 BuildCrystalCutsceneTilemap (JSL
                // PaletteFilter_Crystal).
                crate::cycle_ledger::charge(220);
                self.palette_filter_crystal();
                self.set_sub_screen_layers(1);
                self.follower_link_state_mut().set_immobilized_flag(2);
                let room = self.game_state.world.location.dungeon_room();
                let j = DUNGEON_BOSS_ROOMS
                    .iter()
                    .rposition(|&r| r == room)
                    .expect("rescued maiden room must be a boss room")
                    .checked_sub(4)
                    .expect("rescued maiden boss room index must select a crystal slot");
                // $02:989B scans the eight boss rooms at $02:895C from X=$0C
                // down (92 per pass, BNE taken until the match), then $98A3,
                // four rows of eight stores (56 + 8 x 288, BPL taken seven
                // times, 274, BNE taken three times) and $98E2's RTS.
                let scan = 7 - j as u64;
                crate::cycle_ledger::charge(
                    scan * 92 + (scan - 1) * 6 + 190 + 4 * (56 + 8 * 288 + 7 * 6 + 274) + 3 * 6 + 102,
                );
                let mut dsto = DUNGEON_CRYSTAL_CUTSCENE_TILE_BASES[j] >> 1;
                let mut tile = 0u16;
                for _ in 0..4 {
                    for x in 0..8u16 {
                        self.room_write_bg1(dsto + x, 0x1f80 | tile);
                        self.room_write_bg1(dsto + x + xy(0, 4) as u16, 0x1f88 | tile);
                        tile = tile.wrapping_add(1);
                    }
                    tile = tile.wrapping_add(8);
                    dsto = dsto.wrapping_add(xy(0, 1) as u16);
                }
                self.increment_subsubmodule();
            }
            2 | 4 | 6 | 8 => {
                // $02:8AB3 (JSL WaterFlood_BuildOneQuadrantForVRAM, INC $B0, RTS).
                crate::cycle_ledger::charge(142);
                self.dungeon_inter_room_trans_not_dark_room()
            }
            3 | 5 | 7 | 9 => {
                // $02:8AC8 (JSL Dungeon_PrepareNextRoomQuadrantUpload, INC $B0, RTS).
                crate::cycle_ledger::charge(142);
                self.dungeon_inter_room_trans_state4()
            }
            10 => {
                // $02:98E7 StartCrystalCutscene (INC $012A, two JSLs, RTS).
                crate::cycle_ledger::charge(260);
                if let Some(progress) =
                    self.take_original_timing_rescued_maiden_initialization_progress()
                {
                    assert_eq!(
                        progress.boundary,
                        crate::OriginalTimingBoundary::HostReturn,
                        "rescued-maiden initialization requires source host-return progress",
                    );
                    self.begin_rescued_maiden_initialization();
                    self.apply_follower_graphics_progress(None, progress.stage);
                    self.game_execution_scheduler
                        .schedule_cpu_timed_work_returning_on_later_host(
                            GameWorkContinuation::FinishRescuedMaidenInitialization {
                                stage: progress.stage,
                            },
                            1,
                        );
                    return;
                }
                self.begin_rescued_maiden_initialization();
                self.LoadFollowerGraphics();
                self.crystal_cutscene_initialize_after_follower_graphics();
                self.set_submodule(0);
                self.set_subsubmodule(0);
            }
            _ => {}
        }
    }

    pub(in crate::zelda_rtl) fn apply_rescued_maiden_tilemap_clear_stores(&mut self, start: u16, end: u16) {
        const STORES_PER_WORD: u16 = 8;
        const WORDS_PER_QUADRANT: usize = 0x400;
        const TOTAL_STORES: u16 = 0x2000;

        assert!(start <= end && end <= TOTAL_STORES);
        for ordinal in start..end {
            let word = usize::from(ordinal / STORES_PER_WORD);
            let region = usize::from(ordinal % STORES_PER_WORD);
            let tile_index = (region & 3) * WORDS_PER_QUADRANT + word;
            if region < 4 {
                self.dungeon_room_tilemaps_mut()
                    .set_bg2_tile(tile_index, 0x01ec);
            } else {
                self.dungeon_room_tilemaps_mut()
                    .set_bg1_tile(tile_index, 0x01ec);
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_rescued_maiden_tilemap_clear(&mut self, completed_stores: u16) {
        self.apply_rescued_maiden_tilemap_clear_stores(completed_stores, 0x2000);
        self.set_bg1_y_offset(0);
        self.set_bg1_x_offset(0);
        self.dungeon_moving_floor_mut().clear_floor_offsets();
        self.clear_screen_transition();
        self.dungeon_room_load_mut().clear_quadrant_upload_index();
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_19_MirrorFade`
    pub(in crate::zelda_rtl) fn module07_19_mirror_fade(&mut self) {
        self.overworld_reset_mosaic_always_increase();
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.set_main_module(5);
        self.set_submodule(0);
        self.clear_bg_vram_load_mode();
        self.save_current_music_as_last();
        if self.palette_swap_enabled() {
            self.palette_revert_translucency_swap();
        }
    }

    /// ROM: `Module07_1A_RoomDraw_OpenTriforceDoor_bounce`
    pub(in crate::zelda_rtl) fn module07_1a_room_draw_open_triforce_door_bounce(&mut self) {
        self.follower_link_state_mut().immobilize();
        if self.game_state.dungeon.scratch_word.word() != 0 {
            if self.scratch_word_mut().decrement_ganon_door_bounce_low() != 0 {
                return;
            }
            if self.scratch_word_mut().decrement_high() != 0 {
                return;
            }
            self.set_ambient_sound_effect(21);
            self.follower_link_state_mut().clear_force_hold_sword_up();
            self.follower_link_state_mut().clear_direction_lock();
        }
        self.follower_link_state_mut().clear_immobilized();
        self.increment_subsubmodule();
        if self.game_state.frame.subsubmodule & 3 != 0 {
            return;
        }

        let index = self.game_state.frame.subsubmodule.wrapping_sub(4) as usize >> 2;
        let src = MODULE07_1_A_ROOM_DRAW_OPEN_TRIFORCE_DOOR_BOUNCE_OPEN_GANON_DOOR_TILE_SOURCES
            [index] as usize;
        for i in 0..8u16 {
            self.room_write_bg(
                0x2000,
                xy(44, 3) as u16 + i,
                self.tile_word(src, (i * 4) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 4) as u16 + i,
                self.tile_word(src, (i * 4 + 1) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 5) as u16 + i,
                self.tile_word(src, (i * 4 + 2) as usize),
            );
            self.room_write_bg(
                0x2000,
                xy(44, 6) as u16 + i,
                self.tile_word(src, (i * 4 + 3) as usize),
            );
        }

        self.dungeon_prep_overlay_dma_watergate(0, 0x01d8, 0x0881, 8);
        if self.game_state.frame.subsubmodule == 16 {
            self.write_attr2(xy(44, 5), TilePair::repeated(NativeTile::SOLID_WALL));
            self.write_attr2(xy(44, 6), TilePair::repeated(NativeTile::SOLID_WALL));
            self.write_attr2(
                xy(50, 5),
                TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
            );
            self.write_attr2(
                xy(50, 6),
                TilePair([NativeTile::GROUND, NativeTile::SOLID_WALL]),
            );
            for i in (0..6).step_by(2) {
                for y in 0..7 {
                    self.write_attr2(xy(45 + i, y), TilePair::repeated(NativeTile::GROUND));
                }
            }
            self.room_bounds_mut().set_y_bound(0, (-64i16) as u16);
            self.set_submodule(0);
            self.set_subsubmodule(0);
        }
        self.request_nmi_copy_packets();
    }

    /// ROM: `Dungeon_ExtinguishTorch`
    pub(in crate::zelda_rtl) fn dungeon_extinguish_torch(&mut self) {
        let y = self.game_state.dungeon.torch.target().object_slot() * 2
            + self.game_state.dungeon.torch.torches_start_index() as usize;
        let idx = y >> 1;
        let mut r8 = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(idx)
            & 0x7fff;
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(idx, r8);

        let obj_pos = (self
            .game_state
            .dungeon
            .object_tracking
            .object_pos_in_objdata(idx)
            & 0x00ff)
            >> 1;
        self.dungeon_torch_mut()
            .set_torch_data_word_index(obj_pos as usize, r8);

        r8 &= 0x3fff;
        self.room_draw_adjust_torch_lighting_change(r8, 0x0ec2, r8);
        self.request_nmi_copy_packets();

        if self.game_state.dungeon.torch.wants_lights_out() != 0
            && self.game_state.dungeon.torch.lit_torches() != 0
        {
            self.dungeon_torch_mut().decrement_lit_torches();
            if self.game_state.dungeon.torch.lit_torches() < 3 {
                if self.game_state.dungeon.torch.lit_torches() == 0 {
                    self.set_sub_screen_layers(1);
                }
                let torch = self.game_state.dungeon.torch.lit_torches() as usize;
                self.dungeon_room_effects_mut()
                    .set_fixed_color_plusminus(DUNGEON_LIT_TORCH_COLOR_PLUS[torch]);
                self.set_submodule(10);
                self.set_subsubmodule(0);
            }
        }

        let torch_timer = self.game_state.dungeon.torch.target().object_slot();
        self.dungeon_torch_mut().clear_timer(torch_timer);
        self.dungeon_torch_mut().clear_target();
    }

    fn crystal_cutscene_initialize_before_follower_graphics(&mut self) {
        // CrystalCutscene_Initialize $1E:CCE3..CD47 (JSL target), all of it
        // here; the translation splits it at SpawnMaiden's
        // LoadFollowerGraphics, so the second half reopens the scope.
        let _scope = crate::cycle_ledger::routine(0x1e_cce3);
        crate::cycle_ledger::charge(1_296);
        self.set_color_math_control(0x33);
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.palette_assert_translucency_swap_with_cycle_charge();
        self.palette_filter_crystal();
        for (i, color) in CRYSTAL_CUTSCENE_INITIALIZE_CRYSTAL_MAIDEN_PAL
            .iter()
            .enumerate()
        {
            self.set_main_color_asset(112 + i, *color);
        }
        self.increment_cgram_update_flag();
        self.crystal_cutscene_spawn_maiden_before_follower_graphics();
    }

    fn crystal_cutscene_spawn_maiden_before_follower_graphics(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x1e_cd48);
        // $1E:CD48..CD90: clear the sixteen sprite states (68 per pass, BPL
        // taken fifteen times), spawn and set up the maiden; then $CD92 for
        // room $18 (palace index $18) or BNE taken into $CD9B.
        crate::cycle_ledger::charge(
            32 + 16 * 68 + 15 * 6 + 770
                + if self.game_state.inventory.save_progress.palace_index_x2() == 24 { 92 } else { 6 + 16 },
        );
        self.sprite_system_mut().fill_live_states(0);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(0, 0xab, &mut info);
        let j = j as usize;

        let player_x = self.game_state.player.follower_link.x();
        let player_y = self.game_state.player.follower_link.y();
        let maiden_a = self.ancilla_terminate_select_interactives(j as u8);
        let mut maiden = self.sprite_slot_view_mut(j);
        maiden.set_x((player_x & 0xff00) | 0x78);
        maiden.set_y((player_y & 0xff00) | 0x7c);
        maiden.set_direction(1);
        maiden.set_oam_flags(0x0b);
        maiden.set_subtype2(0);
        maiden.set_floor(0);
        maiden.set_a(maiden_a);
        self.follower_link_state_mut().set_item_receipt_method(0);

        if self.game_state.inventory.save_progress.palace_index_x2() == 24 {
            self.sprite_slot_view_mut(j).set_oam_flags(9);
            self.follower_state_mut().set_indicator(1);
        } else {
            self.follower_state_mut().set_indicator(6);
        }
    }

    fn crystal_cutscene_spawn_maiden_after_follower_graphics(&mut self) {
        // $1E:CD9D..CDD8: the follower-graphics call and the floor offsets.
        let _initialize = crate::cycle_ledger::routine(0x1e_cce3);
        let _scope = crate::cycle_ledger::routine(0x1e_cd48);
        crate::cycle_ledger::charge(752);
        self.follower_state_mut().set_indicator(0);

        let floor_x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(0x79);
        let floor_y =
            0x30u16.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2_low() as u16);
        self.dungeon_moving_floor_mut()
            .set_floor_offsets(floor_x, floor_y);
        self.dungeon_room_load_mut()
            .set_header_collision_2_mirror(1);
    }

    fn crystal_cutscene_initialize_after_follower_graphics(&mut self) {
        self.crystal_cutscene_spawn_maiden_after_follower_graphics();
        let _initialize = crate::cycle_ledger::routine(0x1e_cce3);
        self.crystal_cutscene_initialize_polyhedral();
    }

    pub(super) fn begin_rescued_maiden_initialization(&mut self) {
        self.activate_nmi_thread();
        self.polyhedral_initialize_thread();
        self.crystal_cutscene_initialize_before_follower_graphics();
    }

    pub(in crate::zelda_rtl) fn apply_follower_graphics_progress(
        &mut self,
        prior: Option<crate::RescuedMaidenInitializationStage>,
        next: crate::RescuedMaidenInitializationStage,
    ) {
        use crate::RescuedMaidenInitializationStage::{
            Conversion, FirstFollowerSheet, SecondFollowerSheet,
        };

        match (prior, next) {
            (
                None,
                FirstFollowerSheet {
                    completed_bytes: end,
                },
            ) => self.apply_follower_graphics_decompression_slice(true, 0, end),
            (
                None,
                SecondFollowerSheet {
                    completed_bytes: end,
                },
            ) => {
                self.apply_follower_graphics_decompression_slice(true, 0, 0x0600);
                self.apply_follower_graphics_decompression_slice(false, 0, end);
            }
            (None, Conversion { completed_stores }) => {
                self.apply_follower_graphics_decompression_slice(true, 0, 0x0600);
                self.apply_follower_graphics_decompression_slice(false, 0, 0x0600);
                self.apply_follower_graphics_conversion_slice(0, completed_stores);
            }
            (
                Some(FirstFollowerSheet {
                    completed_bytes: start,
                }),
                FirstFollowerSheet {
                    completed_bytes: end,
                },
            ) => {
                assert!(start <= end, "rescued-maiden first sheet moved backward");
                self.apply_follower_graphics_decompression_slice(true, start, end);
            }
            (
                Some(FirstFollowerSheet {
                    completed_bytes: start,
                }),
                SecondFollowerSheet {
                    completed_bytes: end,
                },
            ) => {
                self.apply_follower_graphics_decompression_slice(true, start, 0x0600);
                self.apply_follower_graphics_decompression_slice(false, 0, end);
            }
            (
                Some(SecondFollowerSheet {
                    completed_bytes: start,
                }),
                SecondFollowerSheet {
                    completed_bytes: end,
                },
            ) => {
                assert!(start <= end, "rescued-maiden second sheet moved backward");
                self.apply_follower_graphics_decompression_slice(false, start, end);
            }
            (
                Some(FirstFollowerSheet {
                    completed_bytes: start,
                }),
                Conversion { completed_stores },
            ) => {
                self.apply_follower_graphics_decompression_slice(true, start, 0x0600);
                self.apply_follower_graphics_decompression_slice(false, 0, 0x0600);
                self.apply_follower_graphics_conversion_slice(0, completed_stores);
            }
            (
                Some(SecondFollowerSheet {
                    completed_bytes: start,
                }),
                Conversion { completed_stores },
            ) => {
                self.apply_follower_graphics_decompression_slice(false, start, 0x0600);
                self.apply_follower_graphics_conversion_slice(0, completed_stores);
            }
            (
                Some(Conversion {
                    completed_stores: start,
                }),
                Conversion {
                    completed_stores: end,
                },
            ) => {
                assert!(start <= end, "follower-graphics conversion moved backward");
                self.apply_follower_graphics_conversion_slice(start, end);
            }
            (Some(SecondFollowerSheet { .. }), FirstFollowerSheet { .. }) => {
                panic!("rescued-maiden initialization returned to its first sheet")
            }
            (Some(Conversion { .. }), FirstFollowerSheet { .. } | SecondFollowerSheet { .. }) => {
                panic!("follower-graphics conversion returned to a decompression sheet")
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_rescued_maiden_initialization(
        &mut self,
        stage: crate::RescuedMaidenInitializationStage,
    ) {
        self.complete_follower_graphics_decompression(stage);
        self.crystal_cutscene_initialize_after_follower_graphics();
        self.set_submodule(0);
        self.set_subsubmodule(0);
    }

    /// ROM: `Dungeon_HandleLayerEffect`
    pub(in crate::zelda_rtl) fn dungeon_handle_layer_effect(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x0a_fe80);
        crate::cycle_ledger::charge(98); // LDA, ASL, TAX, indirect tail jump.
        match self.game_state.dungeon.room_load.header_collision_2() {
            0 | 1 => self.layer_effect_nothing(),
            2 => self.layer_effect_scroll(),
            3 => self.layer_effect_water_rapids(),
            4 => self.layer_effect_trinexx(),
            5 => self.layer_effect_agahnim2(),
            6 => self.layer_effect_invisible_floor(),
            7 => self.layer_effect_ganon(),
            _ => panic!("invalid dungeon layer effect index"),
        }
    }

    /// ROM: `LayerEffect_Nothing`
    pub(super) fn layer_effect_nothing(&mut self) {
        crate::cycle_ledger::charge(44); // Shared RTL at $0A:FE87.
    }

    /// ROM: `LayerEffect_Scroll`
    pub(super) fn layer_effect_scroll(&mut self) {
        crate::cycle_ledger::charge(64);
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0 {
            crate::cycle_ledger::charge(68);
            self.dungeon_room_load_mut().clear_header_collision_2();
            return;
        }
        crate::cycle_ledger::charge(6 + 172);
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
        let flags = self.game_state.dungeon.moving_floor.floor_move_flags();
        if flags & 1 != 0 {
            crate::cycle_ledger::charge(6 + 66);
            return;
        }
        crate::cycle_ledger::charge(248);
        let subpixel = self.bg1_move_calc_mut().advance_x_subpixel(0x80);
        let mut t = (subpixel >> 8) as i16;
        if flags & 2 != 0 {
            crate::cycle_ledger::charge(38);
            t = -t;
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(80);
        if flags < 4 {
            crate::cycle_ledger::charge(318);
            self.dungeon_moving_floor_mut()
                .set_floor_x_velocity(t as u16);
            let x_offs = self
                .game_state
                .dungeon
                .moving_floor
                .floor_x_offset()
                .wrapping_sub(t as u16);
            self.dungeon_moving_floor_mut().set_floor_x_offset(x_offs);
            let bg1 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(x_offs);
            self.set_bg1_x(bg1);
        } else {
            crate::cycle_ledger::charge(6 + 252 + 66);
            self.dungeon_moving_floor_mut()
                .set_floor_y_velocity(t as u16);
            let y_offs = self
                .game_state
                .dungeon
                .moving_floor
                .floor_y_offset()
                .wrapping_sub(t as u16);
            self.dungeon_moving_floor_mut().set_floor_y_offset(y_offs);
            let bg1 = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(y_offs);
            self.set_bg1_y(bg1);
        }
    }

    /// ROM: `LayerEffect_Trinexx`
    pub(super) fn layer_effect_trinexx(&mut self) {
        crate::cycle_ledger::charge(436);
        let x = self
            .game_state
            .dungeon
            .moving_floor
            .floor_x_offset()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_velocity());
        let y = self
            .game_state
            .dungeon
            .moving_floor
            .floor_y_offset()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
        self.dungeon_moving_floor_mut().set_floor_x_offset(x);
        self.dungeon_moving_floor_mut().set_floor_y_offset(y);
        self.dungeon_moving_floor_mut().set_floor_x_velocity(0);
        self.dungeon_moving_floor_mut().set_floor_y_velocity(0);
    }

    /// ROM: `LayerEffect_Agahnim2`
    pub(super) fn layer_effect_agahnim2(&mut self) {
        let j = self.game_state.frame.frame_counter & 0x7f;
        // The ROM tests 3, 5, 36, 38 in that order, sharing the two palette tails.
        crate::cycle_ledger::charge(72);
        if j != 3 {
            crate::cycle_ledger::charge(32);
            if j != 5 {
                crate::cycle_ledger::charge(32);
                if j != 36 { crate::cycle_ledger::charge(32); }
            }
        }
        if j != 38 { crate::cycle_ledger::charge(6); }
        if j == 3 || j == 36 {
            crate::cycle_ledger::charge(212 + 156);
            self.set_main_color_constant(0x6d, 0x1d59);
            self.set_main_color_constant(0x6e, 0x25ff);
            self.set_main_color_constant(0x6f, 0x001a);
            self.set_main_color_constant(0x77, 0x001a);
            self.increment_cgram_update_flag();
        } else if j == 5 || j == 38 {
            crate::cycle_ledger::charge(262 + 156);
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6d),
                (zelda3_palette::Bank::Main, 0x6d),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6e),
                (zelda3_palette::Bank::Main, 0x6e),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6f),
                (zelda3_palette::Bank::Main, 0x6f),
            );
            self.copy_color(
                (zelda3_palette::Bank::Aux, 0x6f),
                (zelda3_palette::Bank::Main, 0x77),
            );
            self.increment_cgram_update_flag();
        }
        crate::cycle_ledger::charge(84);
        self.set_sub_screen_layers(2);
    }

    /// ROM: `LayerEffect_InvisibleFloor`
    pub(super) fn layer_effect_invisible_floor(&mut self) {
        crate::cycle_ledger::charge(78);
        let mut count = 0;
        for i in 0..16 {
            crate::cycle_ledger::charge(76);
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                crate::cycle_ledger::charge(54);
                count += 1;
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(68 + if i != 15 { 6 } else { 0 });
        }
        crate::cycle_ledger::charge(96 + if count == 0 { 48 } else { 6 });
        let (x, y) = if count == 0 { (0, 0) } else { (0x2940, 0x4e60) };
        crate::cycle_ledger::charge(78);
        if self.game_state.display.palette_buffer.aux_color(0x7b) != x {
            crate::cycle_ledger::charge(260);
            self.set_main_color_constant(0x7b, x);
            self.set_aux_color_constant(0x7b, x);
            self.set_main_color_constant(0x7c, y);
            self.set_aux_color_constant(0x7c, y);
            // $0A:FF9B is INC $15 with a 16-bit accumulator. Carry belongs
            // to the adjacent HUD-update flag, not a discarded byte overflow.
            if self.increment_cgram_update_flag() == 0 {
                self.system_signals_mut().increment_hud_update_flag();
            }
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(106);
        self.set_sub_screen_layers(2);
    }

    /// ROM: `LayerEffect_Ganon`
    pub(super) fn layer_effect_ganon(&mut self) {
        crate::cycle_ledger::charge(78);
        let mut count = 0u8;
        // The source stops at byte offset 6: only the first three object words.
        for i in 0..3 {
            crate::cycle_ledger::charge(76);
            if self
                .game_state
                .dungeon
                .object_tracking
                .object_tilemap_pos(i)
                & 0x8000
                != 0
            {
                crate::cycle_ledger::charge(62);
                count = count.wrapping_add(1);
            } else {
                crate::cycle_ledger::charge(6);
            }
            crate::cycle_ledger::charge(68 + if i != 2 { 6 } else { 0 });
        }
        crate::cycle_ledger::charge(70);
        self.dungeon_torch_mut().set_ganon_torch_count(count);
        if count == 0 {
            crate::cycle_ledger::charge(108);
            self.set_sub_screen_layers(0);
            self.set_color_math_control(0xb3);
        } else if count == 1 {
            crate::cycle_ledger::charge(6 + 32 + 124);
            self.set_sub_screen_layers(2);
            self.set_color_math_control(0x70);
        } else {
            crate::cycle_ledger::charge(6 + 32 + 6 + 108);
            self.set_sub_screen_layers(0);
            self.set_color_math_control(0x70);
        }
    }

    /// ROM: `LayerEffect_WaterRapids`
    pub(super) fn layer_effect_water_rapids(&mut self) {
        crate::cycle_ledger::charge(372);
        let t = self.bg1_move_calc_mut().advance_x_subpixel(0x80);
        self.dungeon_moving_floor_mut()
            .set_floor_x_velocity((-(t as i16 >> 8)) as u16);
    }

    /// ROM: `Module07_15_01_ApplyMosaicAndFilter`
    pub(super) fn module07_15_01_apply_mosaic_and_filter(&mut self) {
        self.conditional_mosaic_control();
        self.set_mosaic_copy_from_level_or(3);
        self.apply_palette_filter_bounce();
    }

    /// ROM: `Module07_15_04_SyncRoomPropsAndBuildOverlay`
    pub(super) fn module07_15_04_sync_room_props_and_build_overlay(&mut self) {
        self.apply_grayscale_fixed_incremental();
        if self.game_state.world.location.dungeon_room() == 0x17 {
            self.dungeon_stair_movement_mut().set_current_floor(4);
        }
        self.MirrorBg1Bg2Offs();
        self.dungeon_adjust_for_room_layout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize]
            as u8;
        let mut tm = 0x16;
        if ts & 0x80 != 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts);
        self.water_flood_build_one_quadrant_for_vram();
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_15_0E_FadeInFromWarp`
    pub(super) fn module07_15_0e_fade_in_from_warp(&mut self) {
        if self.game_state.display.palette_filter.countdown() & 1 != 0
            && self.game_state.display.mosaic_level != 0
        {
            self.decrement_mosaic_level_by(0x10);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.apply_palette_filter_bounce();
    }

    /// ROM: `Module07_15_0F_FinalizeAndCacheEntry`
    pub(super) fn module07_15_0f_finalize_and_cache_entry(&mut self) {
        if self.overworld_map_state() == 5 {
            self.SetAndSaveVisitedQuadrantFlags();
            self.set_submodule(0);
            self.ResetThenCacheRoomEntryProperties();
        }
    }

    pub(in crate::zelda_rtl) fn orient_lamp_light_cone(&mut self) {
        // Not annotated yet: counted as a silent call for the ledger prefix.
        let _probe = crate::cycle_ledger::probe_annotation();
        self.OrientLampLightCone();
    }
}
