//! Room scrolling, edge transitions, warps, and graphics handoffs.

use super::*;
use crate::game_state::constants::{COMPOSITE_OF_LAYOUT_AND_QUADRANT, DUNG_BLASTWALL_FLAG_Y, LINK_QUADRANT_X, LINK_QUADRANT_Y, QUADRANT_FULLSIZE_X, QUADRANT_FULLSIZE_Y};

impl ZeldaState {

    /// ROM: `ResetTransitionPropsAndAdvance_ResetInterface`
    pub(in crate::zelda_rtl) fn reset_transition_props_and_advance_reset_interface(&mut self) {
        self.set_overworld_map_state(0);
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    pub(super) fn ResetTransitionPropsAndAdvanceSubmodule(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_8cac);
        crate::cycle_ledger::charge(398); // $8CAC-$8CCD, through the darkness branch.
        self.clear_mosaic_level_word();
        self.set_darkening_or_lightening_screen(0);
        self.set_countdown(0);
        self.set_mosaic_target_level(31);
        self.dungeon_room_runtime_mut().clear_reserved_gfx_config();
        self.dungeon_torch_mut().clear_lit_torches();
        if self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            crate::cycle_ledger::charge(56);
            self.set_color_window_selection(0x02);
            self.set_color_math_control(0xb3);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(22 + 32 + 46); // SEP; STZ darkness; JSR cleanup.
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        self.dungeon_reset_torch_background_and_player_inner();
        crate::cycle_ledger::charge(46); // JSR palette cache.
        self.overworld_copy_palettes_to_cache();
        crate::cycle_ledger::charge(38 + 42); // INC state; RTS.
        self.increment_subsubmodule();
    }

    /// ROM: `RoomBounds_AddX_A`
    pub(super) fn room_bounds_add_x_a(&mut self) {
        self.room_bounds_mut().add_x_bounds_a(0x0100);
    }

    /// ROM: `RoomBounds_AddX_B`
    pub(super) fn room_bounds_add_x_b(&mut self) {
        self.room_bounds_mut().add_x_bounds_b(0x0200);
    }

    /// ROM: `RoomBounds_SubX_B`
    pub(super) fn room_bounds_sub_x_b(&mut self) {
        self.room_bounds_mut().add_x_bounds_b(0xfe00);
    }

    /// ROM: `RoomBounds_SubX_A`
    pub(super) fn room_bounds_sub_x_a(&mut self) {
        self.room_bounds_mut().add_x_bounds_a(0xff00);
    }

    /// ROM: `RoomBounds_AddY_A`
    pub(super) fn room_bounds_add_y_a(&mut self) {
        self.room_bounds_mut().add_y_bounds_a(0x0100);
    }

    /// ROM: `RoomBounds_AddY_B`
    pub(super) fn room_bounds_add_y_b(&mut self) {
        self.room_bounds_mut().add_y_bounds_b(0x0200);
    }

    /// ROM: `RoomBounds_SubY_B`
    pub(super) fn room_bounds_sub_y_b(&mut self) {
        self.room_bounds_mut().add_y_bounds_b(0xfe00);
    }

    /// ROM: `RoomBounds_SubY_A`
    pub(super) fn room_bounds_sub_y_a(&mut self) {
        self.room_bounds_mut().add_y_bounds_a(0xff00);
    }

    /// ROM: `AdjustQuadrantAndCamera_right`
    pub(in crate::zelda_rtl) fn adjust_quadrant_and_camera_right(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_x();
        self.dungeon_adjust_quadrant();
        self.room_bounds_add_x_a();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `AdjustQuadrantAndCamera_left`
    pub(in crate::zelda_rtl) fn adjust_quadrant_and_camera_left(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_x();
        self.dungeon_adjust_quadrant();
        self.room_bounds_sub_x_a();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `AdjustQuadrantAndCamera_down`
    pub(in crate::zelda_rtl) fn adjust_quadrant_and_camera_down(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_y();
        self.dungeon_adjust_quadrant();
        self.room_bounds_add_y_a();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `AdjustQuadrantAndCamera_up`
    pub(in crate::zelda_rtl) fn adjust_quadrant_and_camera_up(&mut self) {
        self.follower_link_state_mut().toggle_quadrant_y();
        self.dungeon_adjust_quadrant();
        self.room_bounds_sub_y_a();
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `Dungeon_AdjustQuadrant`
    pub(super) fn dungeon_adjust_quadrant(&mut self) {
        let quadrant_y = self.game_state.player.follower_link.quadrant_y();
        let quadrant_x = self.game_state.player.follower_link.quadrant_x();
        // dung_layout_and_starting_quadrant is owned solely by DungeonRoomParserState
        // (it is a uint16 set by the room-object parser, C dungeon.c:2661); the low
        // byte feeds the composite key (C dungeon.c:8239).
        let layout = self
            .game_state
            .dungeon
            .room_parser
            .room_layout_and_starting_quadrant() as u8;
        self.dungeon_room_load_mut()
            .update_layout_quadrant_key(layout, quadrant_y, quadrant_x);
    }

    /// ROM: `Dungeon_AdjustForRoomLayout`
    pub(super) fn dungeon_adjust_for_room_layout(&mut self) {
        self.dungeon_adjust_quadrant();
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let horizontal_mask = self.game_state.player.follower_link.quadrant_x_mask();
        let vertical_mask = self.game_state.player.follower_link.quadrant_y_mask();
        let blast_wall_x_open = self.game_state.dungeon.room_effects.blast_wall_x_open();
        let blast_wall_y_open = self.game_state.dungeon.room_effects.blast_wall_y_open();
        self.apply_dungeon_layout_quadrant_fullsize(
            flags,
            horizontal_mask,
            vertical_mask,
            blast_wall_x_open,
            blast_wall_y_open,
        );
        let reset_xy_flags = self.game_state.dungeon.door_setup.reset_xy_check_flags();
        self.apply_reset_xy_quadrant_overrides(reset_xy_flags);
        if crate::debug_env::var_os!("ZELDA3_DEBUG_QUADRANT").is_some() {
            eprintln!(
                "[QUADRANT-LAYOUT] host={} room={:#x} key={:#x} ram_aa={:#x} flags={:#x} x_mask={:#x} y_mask={:#x} ram_a8={:#x} ram_a9={:#x} blast=({},{}) reset_xy={:#x} -> fullsize=({},{})",
                self.frame_ctr_dbg,
                self.game_state.world.location.dungeon_room(),
                self.game_state.dungeon.room_load.layout_quadrant_key(),
                self.ram[LINK_QUADRANT_Y],
                flags,
                horizontal_mask,
                vertical_mask,
                self.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT],
                self.ram[LINK_QUADRANT_X],
                blast_wall_x_open,
                blast_wall_y_open,
                reset_xy_flags,
                self.ram[QUADRANT_FULLSIZE_X],
                self.ram[QUADRANT_FULLSIZE_Y],
            );
        }
    }

    /// ROM: `Dung_SaveDataForCurrentRoom`
    pub(super) fn dung_save_data_for_current_room(&mut self) {
        let saved = (self.game_state.dungeon.savegame_state.savegame_state_bits() >> 4)
            | (self.game_state.dungeon.doors.opened_doors() & 0xf000)
            | self.game_state.dungeon.room_load.quadrants_visited();
        let room = self.game_state.world.location.dungeon_room() as usize;
        self.set_saved_room_flags(room, saved);
    }

    pub(super) fn SaveQuadrantsToSram(&mut self) {
        let room = self.game_state.world.location.dungeon_room() as usize;
        let saved =
            self.saved_room_flags(room) | self.game_state.dungeon.room_load.quadrants_visited();
        self.set_saved_room_flags(room, saved);
    }

    /// ROM: `Dung_HandleExitToOverworld`
    pub(super) fn dung_handle_exit_to_overworld(&mut self) {
        self.SaveDungeonKeys();
        self.SaveQuadrantsToSram();
        self.set_saved_module_for_menu(8);
        self.set_main_module(15);
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.dungeon_reset_torch_background_and_player_inner();
    }

    /// ROM: `Dungeon_FlagRoomData_Quadrants`
    pub(in crate::zelda_rtl) fn dungeon_flag_room_data_quadrants(&mut self) {
        let player_quadrant_y = self.game_state.player.follower_link.quadrant_y();
        let player_quadrant_x = self.game_state.player.follower_link.quadrant_x();
        let index = self
            .game_state
            .world
            .transient
            .dungeon_quadrant_visit_index(player_quadrant_y, player_quadrant_x);
        self.dungeon_room_load_mut()
            .or_quadrants_visited(DUNGEON_QUADRANT_VISITING_FLAGS[index]);
        self.dung_save_data_for_current_room();
    }

    /// ROM: `DungeonTransition_AdjustCamera_X`
    pub(super) fn dungeon_transition_adjust_camera_x(&mut self, arg: u8) {
        let index = arg as usize * 2;
        self.set_left_right_scroll_target(
            DUNGEON_TRANSITION_ADJUST_CAMERA_X_UP_DOWN_SCROLL_VALUES[index],
        );
        self.set_left_right_scroll_target_end(
            DUNGEON_TRANSITION_ADJUST_CAMERA_X_UP_DOWN_SCROLL_VALUES[index + 1],
        );
    }

    /// ROM: `DungeonTransition_AdjustCamera_Y`
    pub(super) fn dungeon_transition_adjust_camera_y(&mut self, arg: u8) {
        let index = arg as usize;
        self.set_up_down_scroll_target(
            DUNGEON_TRANSITION_ADJUST_CAMERA_Y_UP_DOWN_SCROLL_VALUES[index],
        );
        self.set_up_down_scroll_target_end(
            DUNGEON_TRANSITION_ADJUST_CAMERA_Y_UP_DOWN_SCROLL_VALUES[index + 1],
        );
    }

    /// ROM: `HandleEdgeTransition_AdjustCameraBoundaries`
    pub(super) fn handle_edge_transition_adjust_camera_boundaries(&mut self, arg: u8) {
        self.set_screen_transition(arg);
        if self.game_state.player.follower_link.direction() & 3 != 0 {
            let mut index = if self.game_state.player.follower_link.direction() & 1 != 0 {
                0
            } else {
                2
            };
            if self.game_state.player.follower_link.quadrant_x() != 0 {
                index += 1;
            }
            self.set_camera_x_coord_scroll_low(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_X_BOUNDS[index],
            );
            self.set_camera_x_coord_scroll_hi(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_X_BOUNDS[index]
                    .wrapping_add(2),
            );
        } else {
            let mut index = if self.game_state.player.follower_link.direction() & 4 != 0 {
                0
            } else {
                2
            };
            if self.game_state.player.follower_link.quadrant_y() != 0 {
                index += 1;
            }
            self.set_camera_y_coord_scroll_low(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_Y_BOUNDS[index],
            );
            self.set_camera_y_coord_scroll_hi(
                HANDLE_EDGE_TRANSITION_ADJUST_CAMERA_BOUNDARIES_CAMERA_Y_BOUNDS[index]
                    .wrapping_add(2),
            );
        }
    }

    /// ROM: `Dungeon_StartInterRoomTrans_Left`
    pub(super) fn dungeon_start_inter_room_trans_left(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_x = self.follower_link_state_mut().toggle_quadrant_x();
        self.dungeon_adjust_quadrant();
        self.room_bounds_sub_x_a();
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_x(quadrant_x ^ 1);
        self.handle_edge_transition_adjust_camera_boundaries(3);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_x() != 0 {
            self.room_bounds_sub_x_b();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() & 0xcf == 0x89 {
                let room = self.game_state.dungeon.header.travel_destination(3);
                self.set_dungeon_room_index(room);
                self.dungeon_adjust_for_teleport_doors(
                    self.game_state
                        .world
                        .location
                        .dungeon_room_index()
                        .wrapping_add(1),
                    0xff,
                );
            } else {
                if self.game_state.world.location.dungeon_room_index()
                    != self.game_state.dungeon.room_tracking.room_index2()
                {
                    let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(previous_room);
                    self.dungeon_adjust_after_spiral_stairs();
                }
                self.decrement_dungeon_room_index_by(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_y_after_transition();
    }

    /// ROM: `Dung_StartInterRoomTrans_Left_Plus`
    pub(in crate::zelda_rtl) fn dung_start_inter_room_trans_left_plus(&mut self) {
        let x = self.game_state.player.follower_link.x().wrapping_sub(8);
        self.follower_link_state_mut().set_x(x);
        self.dungeon_start_inter_room_trans_left();
    }

    /// ROM: `Dungeon_StartInterRoomTrans_Right`
    pub(super) fn dungeon_start_inter_room_trans_right(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_x = self.follower_link_state_mut().toggle_quadrant_x();
        self.dungeon_adjust_quadrant();
        self.room_bounds_add_x_a();
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_x(quadrant_x);
        self.handle_edge_transition_adjust_camera_boundaries(2);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_x() == 0 {
            self.room_bounds_add_x_b();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() & 0xcf == 0x89 {
                let room = self.game_state.dungeon.header.travel_destination(4);
                self.set_dungeon_room_index(room);
                self.dungeon_adjust_for_teleport_doors(
                    self.game_state
                        .world
                        .location
                        .dungeon_room_index()
                        .wrapping_sub(1),
                    1,
                );
            } else {
                if self.game_state.world.location.dungeon_room_index()
                    != self.game_state.dungeon.room_tracking.room_index2()
                {
                    let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(previous_room);
                    self.dungeon_adjust_after_spiral_stairs();
                }
                self.increment_dungeon_room_index_by(1);
            }
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_y_after_transition();
    }

    /// ROM: `Dungeon_StartInterRoomTrans_Up`
    pub(in crate::zelda_rtl) fn dungeon_start_inter_room_trans_up(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_y = self.follower_link_state_mut().toggle_quadrant_y();
        self.dungeon_adjust_quadrant();
        self.room_bounds_sub_y_a();
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_y(quadrant_y ^ 2);
        self.handle_edge_transition_adjust_camera_boundaries(1);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_y() != 0 {
            self.room_bounds_sub_y_b();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() == 0x8e {
                self.dung_handle_exit_to_overworld();
                return;
            }
            if self.game_state.world.location.dungeon_room_index() == 0 {
                self.SaveDungeonKeys();
                self.set_main_module(25);
                self.set_submodule(0);
                self.set_subsubmodule(0);
                return;
            }
            // B86E branches past adjustment when the room indices match.
            // The C translation inverted this condition; follow the ROM.
            if self.game_state.dungeon.room_tracking.room_index2()
                != self.game_state.world.location.dungeon_room_index()
            {
                let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                self.dungeon_room_tracking_mut()
                    .set_room_index_prev(previous_room);
                self.dungeon_adjust_after_spiral_stairs();
            }
            self.decrement_dungeon_room_index_by(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_x_after_transition();
    }

    /// ROM: `Dungeon_StartInterRoomTrans_Down`
    pub(super) fn dungeon_start_inter_room_trans_down(&mut self) {
        assert_eq!(self.game_state.frame.submodule, 0);
        let quadrant_y = self.follower_link_state_mut().toggle_quadrant_y();
        self.dungeon_adjust_quadrant();
        self.room_bounds_add_y_a();
        self.dung_save_data_for_current_room();
        self.dungeon_transition_adjust_camera_y(quadrant_y);
        self.handle_edge_transition_adjust_camera_boundaries(0);
        self.set_submodule(1);
        if self.game_state.player.follower_link.quadrant_y() == 0 {
            self.room_bounds_add_y_b();
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            if self.game_state.player.follower_link.tile_below() == 0x8e {
                self.dung_handle_exit_to_overworld();
                return;
            }
            if self.game_state.world.location.dungeon_room_index()
                != self.game_state.dungeon.room_tracking.room_index2()
            {
                let previous_room = self.game_state.dungeon.room_tracking.room_index2();
                self.dungeon_room_tracking_mut()
                    .set_room_index_prev(previous_room);
                self.dungeon_adjust_after_spiral_stairs();
            }
            self.increment_dungeon_room_index_by(0x10);
            self.finish_super_tile_transition_room_side_effects();
        }
        self.set_room_transitioning_flags(0);
        self.update_quadrant_fullsize_x_after_transition();
    }

    fn finish_super_tile_transition_room_side_effects(&mut self) {
        self.set_submodule(2);
        if self.game_state.world.transient.room_transitioning_flags() & 1 != 0 {
            self.follower_link_state_mut().toggle_lower_level_state();
            self.follower_link_state_mut().mirror_lower_level_state();
        }
        if self.game_state.world.transient.room_transitioning_flags() & 2 != 0 {
            self.save_progress_mut().xor_palace_index_x2(2);
        }
    }

    fn update_quadrant_fullsize_x_after_transition(&mut self) {
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let mask = self.game_state.player.follower_link.quadrant_x_mask();
        let blast_wall_x_open = self.game_state.dungeon.room_effects.blast_wall_x_open();
        self.apply_dungeon_layout_horizontal_fullsize(flags, mask, blast_wall_x_open);
    }

    fn update_quadrant_fullsize_y_after_transition(&mut self) {
        let flags =
            LAYOUT_QUADRANT_FLAGS[self.game_state.dungeon.room_load.layout_quadrant_key() as usize];
        let mask = self.game_state.player.follower_link.quadrant_y_mask();
        let blast_wall_y_open = self.game_state.dungeon.room_effects.blast_wall_y_open();
        if crate::debug_env::var_os!("ZELDA3_DEBUG_QUADRANT").is_some() {
            eprintln!(
                "[QUADRANT] host={} room={:#x} key={:#x} ram_aa={:#x} flags={:#x} y_mask={:#x} ram_a9={:#x} blast_y={} ram_453={:#x} -> fullsize_y={}",
                self.frame_ctr_dbg,
                self.game_state.world.location.dungeon_room(),
                self.game_state.dungeon.room_load.layout_quadrant_key(),
                self.ram[LINK_QUADRANT_Y],
                flags,
                mask,
                self.ram[LINK_QUADRANT_X],
                blast_wall_y_open,
                self.ram[DUNG_BLASTWALL_FLAG_Y],
                u8::from(blast_wall_y_open || flags & mask == 0) * 2,
            );
        }
        self.apply_dungeon_layout_vertical_fullsize(flags, mask, blast_wall_y_open);
    }

    /// ROM: `HandleEdgeTransitionMovementEast_RightBy8`
    pub(in crate::zelda_rtl) fn handle_edge_transition_movement_east_right_by8(&mut self) {
        let x = self.game_state.player.follower_link.x().wrapping_add(8);
        self.follower_link_state_mut().set_x(x);
        self.dungeon_start_inter_room_trans_right();
    }

    /// ROM: `HandleEdgeTransitionMovementSouth_DownBy16`
    pub(in crate::zelda_rtl) fn handle_edge_transition_movement_south_down_by16(&mut self) {
        let y = self.game_state.player.follower_link.y().wrapping_add(16);
        self.follower_link_state_mut().set_y(y);
        self.dungeon_start_inter_room_trans_down();
    }

    /// ROM: `Module07_15_WarpPad`
    pub(in crate::zelda_rtl) fn module07_15_warp_pad(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            self.graphics_incremental_vram_upload();
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::WarpPad,
            ) {
                return;
            }
            self.dungeon_load_attribute_selectable();
        }
        self.complete_module07_15_warp_pad_after_attribute_loader();
    }

    pub(in crate::zelda_rtl) fn complete_module07_15_warp_pad_after_attribute_loader(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => self.module07_15_01_apply_mosaic_and_filter(),
            2 => {
                // The warp-pad room initialization is the same long
                // Dungeon_InitializeRoomFromSpecial call as the falling
                // entrance's; it can hold the translated caller across the
                // wire-held vblank (route host 92533).
                if !self.begin_dungeon_falling_room_initialization_work() {
                    self.dungeon_initialize_room_from_special();
                }
            }
            3 => self.dungeon_transition_load_sprite_gfx(),
            4 => self.module07_15_04_sync_room_props_and_build_overlay(),
            5 => self.dungeon_inter_room_trans_state4(),
            6 => self.dungeon_inter_room_trans_not_dark_room(),
            7 => self.dungeon_inter_room_trans_state4(),
            8 => self.dungeon_inter_room_trans_not_dark_room(),
            9 => self.dungeon_inter_room_trans_state4(),
            10 => self.dungeon_inter_room_trans_not_dark_room(),
            11 => self.dungeon_inter_room_trans_state4(),
            12 => self.dungeon_staircase14(),
            13 => self.module07_15_0e_fade_in_from_warp(),
            14 => self.module07_15_0f_finalize_and_cache_entry(),
            other => panic!("invalid Module07_15_WarpPad subsubmodule_index {other}"),
        }
    }

    /// ROM: `Dung_TagRoutine_TrapdoorsUp`
    pub(super) fn dung_tag_routine_trapdoors_up(&mut self) {
        if self.game_state.dungeon.environment.trapdoors_down() != 0 {
            self.dungeon_environment_mut().clear_trapdoors_down();
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_sound_effect_2(0x1b);
            self.set_submodule(5);
        }
    }

    pub(in crate::zelda_rtl) fn CalculateTransitionLanding(&mut self) -> u8 {
        let mut pos = ((self.game_state.player.follower_link.y().wrapping_add(12) & 0x01f8) << 3)
            | ((self.game_state.player.follower_link.x().wrapping_add(8) & 0x01f8) >> 3);
        pos |= self
            .game_state
            .player
            .follower_link
            .lower_level_tilemap_offset();

        let result = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(pos as usize)
            .transition_landing();

        self.dungeon_room_runtime_mut().set_landing_class(result);
        result
    }

    pub(super) fn MirrorBg1Bg2Offs(&mut self) {
        let h = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let v = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.set_bg1_x(h);
        self.set_bg1_y(v);
    }

    /// ROM: `Dungeon_InterRoomTrans_State13`
    pub(in crate::zelda_rtl) fn dungeon_inter_room_trans_state13(&mut self) {
        self.stage_room_72_supertile_landing_obj_scanout();
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.apply_palette_filter_bounce();
        }
        self.dungeon_intra_room_trans_state5();
        if let Some(advance) = self
            .take_dungeon_landing_cpu_advance()
            .map(|advance| self.supersede_dungeon_sprite_main_estimate_by_wire(advance))
        {
            match advance.phase {
                ModuleCpuPhase::InterruptedInNmiPrepareSprites
                    if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) =>
                {
                    // Sprite_Main, LinkOam and HUD have already returned.
                    // Only preparation resumes after this held handler; the
                    // next Open NMI owns its queued palette and OBJ uploads.
                    self.dungeon_nmi_prepare_sprites_return_pending = true;
                }
                ModuleCpuPhase::InterruptedInNmiPrepareSprites
                | ModuleCpuPhase::InterruptedAfterModule => {
                    if self.original_timing_owes_sprite_main_return() {
                        // The live wire still owes this host its Sprite_Main
                        // return, so the caller cannot suspend before that
                        // body. Its typed interruption receipt (for host
                        // 27210, SpritePreparation) names the real suspension
                        // point and owns it through its own consumer; the
                        // estimate advance stands down entirely.
                    } else {
                        let scheduled = self.begin_dungeon_supertile_transition_work(
                            DungeonSupertileTransitionWork::State13CallerReturn,
                        );
                        debug_assert!(scheduled);
                        self.dungeon_state_13_recurring_main_publication_host_frame =
                            Some(self.frame_ctr_dbg);
                    }
                }
                ModuleCpuPhase::InterruptedBeforeSpriteMain
                | ModuleCpuPhase::InterruptedInSpriteMain => {
                    if self.original_timing_owes_sprite_main_return() {
                        // The live wire proves this host's Sprite_Main return
                        // crossed before the held acceptance; the estimate's
                        // raster phase drifted into the body. Run the body
                        // whole and suspend at the post-return boundary
                        // instead (route host 92969).
                        self.dungeon_post_sprite_main_return_pending = true;
                    } else {
                        let armed = self.arm_dungeon_sprite_main_cpu_continuation(advance);
                        debug_assert!(armed);
                    }
                }
                ModuleCpuPhase::InterruptedAfterSpriteMain
                | ModuleCpuPhase::InterruptedInLinkOam => {
                    // Let the translated common suffix execute Sprite_Main
                    // exactly once, then suspend at the measured post-sprite
                    // boundary before resuming Link/OAM work after NMI.
                    self.dungeon_post_sprite_main_return_pending = true;
                }
                ModuleCpuPhase::CompleteBeforeNmi => {
                    self.dungeon_state_13_atomic_caller_return_publication_host_frame =
                        Some(self.frame_ctr_dbg);
                }
                ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites => {
                    // The interrupt landed after the module body and its
                    // Sprite_Main but before NMI_PrepareSprites began. The
                    // live host plan's armed pending common suffix owns the
                    // deferred prep; no scheduled work is needed (route host
                    // 33322).
                    if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
                        self.dungeon_state_12_caller_suffix_nmi_pending = true;
                    }
                }
                phase => panic!(
                    "state-13 vblank reached {phase:?}; a semantic continuation is required \
                     (host={} room={:04x} countdown={} sprite_boundary={:?})",
                    self.frame_ctr_dbg,
                    self.game_state.world.location.dungeon_room(),
                    self.game_state.display.palette_filter.countdown(),
                    advance.sprite_main_boundary,
                ),
            }
            debug_assert_eq!(advance.subsubmodule, self.game_state.frame.subsubmodule);
            debug_assert_eq!(
                advance.palette_countdown,
                self.game_state.display.palette_filter.countdown(),
            );
        }
    }

    /// ROM: `Module07_01_SubtileTransition`
    pub(in crate::zelda_rtl) fn module07_01_subtile_transition(&mut self) {
        self.follower_link_state_mut()
            .cache_previous_position_from_current();
        self.link_handle_moving_animation_full_long_entry();
        match self.game_state.frame.subsubmodule {
            0 => self.dungeon_transition_subtile_prep_transition(),
            1 => self.dungeon_transition_subtile_apply_filter(),
            2 => self.dungeon_transition_subtile_reset_shutters(),
            3 => self.dungeon_transition_scroll_room(),
            4 => self.dungeon_transition_find_subtile_landing(),
            5 => self.dungeon_intra_room_trans_state5(),
            6 => self.dungeon_transition_subtile_apply_filter(),
            7 => self.dungeon_transition_subtile_trigger_shutters(),
            _ => panic!("invalid dungeon subtile transition index"),
        }
    }

    /// ROM: `Module07_02_SupertileTransition`
    pub(in crate::zelda_rtl) fn module07_02_supertile_transition(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_8a26);
        crate::cycle_ledger::charge(228); // Cache position and test transition state.
        let entry_subsubmodule = self.game_state.frame.subsubmodule;
        let state_12_cpu_advance =
            (self.rom_startup_timing() && entry_subsubmodule == 12).then(|| {
                let captured = self.take_dungeon_landing_cpu_advance();

                captured.unwrap_or_else(|| begin_dungeon_supertile_state_12_cpu_advance(self))
            });
        let quadrant_cpu_advance = (self.rom_startup_timing()
            && matches!(entry_subsubmodule, 4..=7))
        .then(|| self.take_dungeon_landing_cpu_advance())
        .flatten();
        self.follower_link_state_mut()
            .cache_previous_position_from_current();
        if self.rom_startup_timing() && self.game_state.frame.subsubmodule == 1 {
            // The ROM timing checkpoint is the Module 7 dispatcher entry,
            // before its shared attribute-loader prefix mutates $0200 and the
            // room-attribute cursors. Capture that exact state once; cloning
            // inside Module07_02_01 would execute the prefix twice in the
            // isolated instruction stream.
            let measured_entry = self.native_dungeon_room_load_cpu_entry.take();
            let schedule = dungeon_room_load_cpu_schedule(self, measured_entry);
            if crate::debug_env::var_os!("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
                eprintln!(
                    "dungeon_room_load_cpu_schedule host={} room={:04x} room_load_nmis={} auxiliary_graphics_nmis={} caller_nmis={} prefix_nmis={} sprite_main_nmis={} suffix_nmis={} boundary={:?}",
                    self.frame_ctr_dbg,
                    self.game_state.world.location.dungeon_room_index(),
                    schedule.room_load_nmis,
                    schedule.auxiliary_graphics_nmis,
                    schedule.caller_nmis,
                    schedule.caller_prefix_nmis,
                    schedule.caller_sprite_main_nmis,
                    schedule.caller_suffix_nmis,
                    schedule.sprite_main_boundary,
                );
            }
            self.dungeon_room_load_cpu_schedule = Some(schedule);
        }
        if self.game_state.frame.subsubmodule != 0 {
            crate::cycle_ledger::charge(32); // CMP #7; BCC.
            if self.game_state.frame.subsubmodule >= 7 {
                crate::cycle_ledger::charge(62);
                self.graphics_incremental_vram_upload();
            } else {
                crate::cycle_ledger::charge(6);
            }
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::SupertileTransition {
                    state_12_cpu_advance,
                    quadrant_cpu_advance,
                },
            ) {
                return;
            }
            crate::cycle_ledger::charge(62); // JSL attribute loader.
            self.dungeon_load_attribute_selectable();
        } else {
            crate::cycle_ledger::charge(6); // BEQ skips the attribute loader.
        }
        self.complete_module07_02_supertile_transition_after_attribute_loader(
            state_12_cpu_advance,
            quadrant_cpu_advance,
        );
    }

    pub(in crate::zelda_rtl) fn complete_module07_02_supertile_transition_after_attribute_loader(
        &mut self,
        state_12_cpu_advance: Option<DungeonModuleCpuAdvance>,
        quadrant_cpu_advance: Option<DungeonModuleCpuAdvance>,
    ) {
        crate::cycle_ledger::charge(62); // $8A44 JSL animation.
        self.link_handle_moving_animation_full_long_entry();
        crate::cycle_ledger::charge(98); // LDA state; ASL; TAX; JMP (table,X).
        match self.game_state.frame.subsubmodule {
            0 => self.module07_02_00_initialize_transition(),
            1 => self.module07_02_01_load_next_room(),
            2 => self.module07_02_faded_filter(),
            3 => self.dungeon_inter_room_trans_state3(),
            4 => self.dungeon_inter_room_trans_state4(),
            5 => self.dungeon_inter_room_trans_not_dark_room(),
            6 => self.dungeon_inter_room_trans_state4(),
            7 => self.dungeon_inter_room_trans_state7(),
            8 => {
                self.stage_room_72_supertile_scroll_obj_scanout();
                self.dungeon_transition_scroll_room();
            }
            9 => self.dungeon_inter_room_trans_state9(),
            10 => self.dungeon_inter_room_trans_state10(),
            11 => self.dungeon_inter_room_trans_state9(),
            12 => {
                let caller_suffix_crosses_nmi = match state_12_cpu_advance
                    .map(|advance| advance.phase)
                {
                    Some(ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites) => true,
                    Some(ModuleCpuPhase::InterruptedInNmiPrepareSprites) => {
                        // The state body and common Module 7 suffix have
                        // already returned. Preserve the existing semantic
                        // NMI_PrepareSprites continuation so the dispatcher
                        // resumes only that interrupted caller after NMI.
                        self.dungeon_nmi_prepare_sprites_return_pending = true;
                        false
                    }
                    Some(
                        ModuleCpuPhase::InterruptedAfterSpriteMain
                        | ModuleCpuPhase::InterruptedInLinkOam,
                    ) => {
                        // As in state 13: the translated common suffix
                        // executes Sprite_Main exactly once, then suspends
                        // at the measured post-sprite boundary before
                        // resuming Link/OAM work after NMI (route: the
                        // warp-pad landing following host 92533).
                        self.dungeon_post_sprite_main_return_pending = true;
                        false
                    }
                    Some(
                        ModuleCpuPhase::InterruptedBeforeSpriteMain
                        | ModuleCpuPhase::InterruptedInSpriteMain,
                    ) => {
                        // As in state 13 (route host 408408: the host ends
                        // inside Sprite_Main after slot 1 with no NMI).
                        if self.original_timing_owes_sprite_main_return() {
                            self.dungeon_post_sprite_main_return_pending = true;
                        } else {
                            let armed = self.arm_dungeon_sprite_main_cpu_continuation(
                                state_12_cpu_advance
                                    .expect("state-12 Sprite_Main phase requires its advance"),
                            );
                            debug_assert!(armed);
                        }
                        false
                    }
                    Some(
                        phase @ (ModuleCpuPhase::InterruptedBeforeSubmodule
                        | ModuleCpuPhase::InterruptedInSubmodule
                        | ModuleCpuPhase::InterruptedAfterModule),
                    ) => panic!(
                        "state-12 vblank reached {phase:?}; a semantic continuation is required (host={} room={:04x})",
                        self.frame_ctr_dbg,
                        self.game_state.world.location.dungeon_room(),
                    ),
                    Some(ModuleCpuPhase::CompleteBeforeNmi) | None => false,
                };
                self.dungeon_inter_room_trans_state12();
                self.dungeon_state_12_caller_suffix_nmi_pending = caller_suffix_crosses_nmi;
            }
            13 => self.dungeon_inter_room_trans_state13(),
            14 => {
                self.stage_room_72_supertile_landing_obj_scanout();
                self.module07_02_faded_filter();
            }
            15 => self.dungeon_inter_room_trans_state15(),
            _ => panic!("invalid dungeon supertile transition index"),
        }
        if let Some(advance) = quadrant_cpu_advance {
            self.apply_dungeon_quadrant_cpu_advance(advance);
        }
    }

    /// ROM: `Module07_02_00_InitializeTransition`
    pub(super) fn module07_02_00_initialize_transition(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_8a4f);
        crate::cycle_ledger::charge(32 + 22 + 46); // Preserve darkness; call shared reset.
        self.dungeon_post_landing_leading_nmi_room = None;
        let bak = self
            .game_state
            .dungeon
            .torch
            .dungeon_dark_with_lantern_raw();
        self.ResetTransitionPropsAndAdvanceSubmodule();
        crate::cycle_ledger::charge(28 + 32 + 42); // Restore darkness; RTS.
        self.dungeon_torch_mut()
            .set_dungeon_dark_with_lantern_raw(bak);
    }

    /// ROM: `Module07_02_01_LoadNextRoom`
    pub(super) fn module07_02_01_load_next_room(&mut self) {
        if self.native_exact_cpu_owner.is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL").is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_OBJECT_DRAW_LIVE_TRIAL").is_some()
        {
            // Dungeon_LoadRoom's floor and object loops cross multiple CPU
            // hosts. Retain their translated cursors until the source-owned
            // statement completions, as for the entrance room loader.
            self.begin_live_dungeon_supertile_room_load();
        } else {
            self.dungeon_load_room();
        }
        if self.begin_dungeon_supertile_transition_work(DungeonSupertileTransitionWork::RoomLoad) {
            return;
        }
        self.finish_live_dungeon_supertile_room_load();
        self.continue_module07_02_01_after_room_load();
    }

    /// Begin the shared Dungeon_LoadRoom draw without publishing its floor,
    /// object, or post-object work before that work's continuation advances.
    pub(super) fn begin_live_dungeon_supertile_room_load(&mut self) {
        self.dungeon_load_header();
        self.dungeon_load_room_before_floors();
        assert!(self.active_early_dungeon_floor_draw.is_none());
        self.active_early_dungeon_floor_draw = self.begin_dungeon_floor_draw_current_room();
        assert!(self.active_early_dungeon_floor_draw.is_some());
    }

    pub(in crate::zelda_rtl) fn finish_live_dungeon_supertile_room_load(&mut self) {
        let Some(floor) = self.active_early_dungeon_floor_draw.take() else {
            return;
        };
        assert_eq!(floor.completed_words(), DungeonFloorDrawWork::TOTAL_WORDS,
            "supertile room load returned before its floor draw completed");
        if let Some(mut objects) = self.active_early_dungeon_object_draw.take() {
            // The retained CPU has returned from Dungeon_LoadRoom, so all of
            // its remaining object statements have executed.
            while self.advance_dungeon_object_draw_one(&mut objects) {}
            assert!(objects.finished());
            self.dungeon_load_room_after_objects();
        } else if self.active_early_dungeon_room_upload.is_none() {
            // The source already crossed the post-object torch-index stores
            // on an earlier host. Only its remaining torch stream is owed.
            self.dungeon_load_room_after_object_prefix();
        }
    }

    pub(in crate::zelda_rtl) fn continue_module07_02_01_after_room_load(&mut self) {
        self.ResetStarTileGraphics();
        self.load_trans_aux_gfx_sprite();
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
        ) {
            return;
        }
        self.complete_module07_02_01_after_auxiliary_sprite_graphics();
    }

    pub(in crate::zelda_rtl) fn complete_module07_02_01_after_auxiliary_sprite_graphics(&mut self) {
        self.complete_module07_02_01_before_dungeon_reset_sprites();
        self.complete_module07_02_01_from_dungeon_reset_sprites();
    }

    pub(in crate::zelda_rtl) fn complete_module07_02_01_before_dungeon_reset_sprites(&mut self) {
        self.increment_subsubmodule();
        self.set_overworld_map_state(0);
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
    }

    pub(in crate::zelda_rtl) fn complete_module07_02_01_from_dungeon_reset_sprites(&mut self) {
        self.dungeon_reset_sprites();
        self.complete_module07_02_01_after_dungeon_reset_sprites();
    }

    pub(in crate::zelda_rtl) fn complete_module07_02_01_after_dungeon_reset_sprites(&mut self) {
        if !self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            self.MirrorBg1Bg2Offs();
        }
        self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
    }

    /// ROM: `Dungeon_InterRoomTrans_State3`
    pub(super) fn dungeon_inter_room_trans_state3(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            self.set_sub_screen_layers(0);
        }
        self.dungeon_adjust_for_room_layout();
        self.LoadNewSpriteGFXSet();
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpriteConversion,
        ) {
            return;
        }
        self.complete_dungeon_inter_room_transition_state3_after_sprite_conversion();
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_inter_room_transition_state3_after_sprite_conversion(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.water_flood_build_one_quadrant_for_vram();
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_InterRoomTrans_State10`
    pub(super) fn dungeon_inter_room_trans_state10(&mut self) {
        let palette_filter_active = self.game_state.dungeon.torch.any_lights_out_request() != 0;
        let palette_filter_loop_master_cycles = palette_filter_active
            .then(|| crate::zelda_rtl::palette_filter_bounce_loop_master_cycles(self));
        if palette_filter_active {
            self.apply_palette_filter_bounce();
        }
        self.dungeon_inter_room_trans_not_dark_room_with_cpu_timing(palette_filter_loop_master_cycles);
    }

    /// ROM: `Dungeon_SpiralStaircase11`
    pub(super) fn dungeon_spiral_staircase11(&mut self) {
        if self.lag_free {
            crate::cycle_ledger::charge(62); // $02:8AAF JSL palette filter.
        }
        self.apply_palette_filter_bounce();
        if self.suspend_spiral_staircase_palette_filter(
            SpiralStaircasePaletteTail::BuildQuadrantForVram,
        ) {
            return;
        }
        self.complete_dungeon_spiral_staircase_palette_filter(
            SpiralStaircasePaletteTail::BuildQuadrantForVram,
        );
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_spiral_staircase_palette_filter(
        &mut self,
        tail: SpiralStaircasePaletteTail,
    ) {
        match tail {
            SpiralStaircasePaletteTail::FallingFadeInAfterDirectionToggle => {
                self.complete_apply_palette_filter_bounce_after_direction_toggle();
                self.complete_falling_fade_in_after_palette();
                return;
            }
            SpiralStaircasePaletteTail::BuildQuadrantForVram => {
                if self.lag_free { crate::cycle_ledger::charge(62 + 38 + 42); } // $02:8AB3..8AB9.
                self.water_flood_build_one_quadrant_for_vram();
            }
            SpiralStaircasePaletteTail::PrepareNextQuadrant => {
                if self.lag_free { crate::cycle_ledger::charge(62 + 38 + 42); } // $02:8AC8..8ACE.
                self.dungeon_prepare_next_room_quadrant_upload();
            }
        }
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_InterRoomTrans_notDarkRoom`
    pub(in crate::zelda_rtl) fn dungeon_inter_room_trans_not_dark_room(&mut self) {
        self.dungeon_inter_room_trans_not_dark_room_with_cpu_timing(None);
    }

    /// ROM: `Dungeon_InterRoomTrans_notDarkRoom_with_cpu_timing`
    fn dungeon_inter_room_trans_not_dark_room_with_cpu_timing(
        &mut self,
        palette_filter_loop_master_cycles: Option<u32>,
    ) {
        if palette_filter_loop_master_cycles.is_some()
            && self.game_state.frame.submodule == 2
            && self.game_state.frame.subsubmodule == 10
            && self.begin_dungeon_supertile_transition_work_with_palette(
                DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild,
                palette_filter_loop_master_cycles,
            )
        {
            return;
        }
        self.complete_dungeon_inter_room_transition_not_dark_room();
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_inter_room_transition_not_dark_room(&mut self) {
        self.water_flood_build_one_quadrant_for_vram();
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_InterRoomTrans_State9`
    pub(in crate::zelda_rtl) fn dungeon_inter_room_trans_state9(&mut self) {
        let palette_filter_active = self.game_state.dungeon.torch.any_lights_out_request() != 0;
        let palette_filter_loop_master_cycles = palette_filter_active
            .then(|| crate::zelda_rtl::palette_filter_bounce_loop_master_cycles(self));
        if palette_filter_active {
            self.apply_palette_filter_bounce();
        }
        self.dungeon_inter_room_trans_state4_with_cpu_timing(palette_filter_loop_master_cycles);
    }

    /// ROM: `Dungeon_SpiralStaircase12`
    pub(super) fn dungeon_spiral_staircase12(&mut self) {
        if self.lag_free {
            crate::cycle_ledger::charge(62); // $02:8AC4 JSL palette filter.
        }
        self.apply_palette_filter_bounce();
        if self.suspend_spiral_staircase_palette_filter(
            SpiralStaircasePaletteTail::PrepareNextQuadrant,
        ) {
            return;
        }
        self.complete_dungeon_spiral_staircase_palette_filter(
            SpiralStaircasePaletteTail::PrepareNextQuadrant,
        );
    }

    /// ROM: `Dungeon_InterRoomTrans_State4`
    pub(super) fn dungeon_inter_room_trans_state4(&mut self) {
        self.dungeon_inter_room_trans_state4_with_cpu_timing(None);
    }

    /// ROM: `Dungeon_InterRoomTrans_State4_with_cpu_timing`
    fn dungeon_inter_room_trans_state4_with_cpu_timing(
        &mut self,
        palette_filter_loop_master_cycles: Option<u32>,
    ) {
        if self.game_state.frame.subsubmodule == 6 {
            self.stage_dungeon_supertile_quadrant_upload_obj_scanout();
        }
        if let Some((words, boundary)) = self.take_native_exact_cpu_quadrant_copy_boundary() {
            let quadrant = usize::from(self.game_state.dungeon.room_load.quadrant_upload_index());
            let mut copy = self.begin_dungeon_quadrant_upload(ROOM_BG2_TILEMAP_BASE, quadrant, 0);
            self.advance_dungeon_quadrant_upload(&mut copy, words);
            let work = GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::QuadrantUploadCopy { copy },
            };
            match boundary {
                OriginalTimingBoundary::HostReturn => self.game_execution_scheduler.schedule_work(work, 1),
                OriginalTimingBoundary::NmiAccepted => self.game_execution_scheduler.schedule_after_current_trailing_nmi(work),
            }
            return;
        }
        self.dungeon_prepare_next_room_quadrant_upload();
        if self.rom_startup_timing()
            && self.game_state.frame.submodule == 2
            && self.game_state.frame.subsubmodule == 11
            && palette_filter_loop_master_cycles.is_some()
        {
            // The filtered quadrant work reaches vblank before state 11 can
            // advance. Keep the translated call stack suspended at the body
            // boundary selected by the CPU phase budget.
            let scheduled = self.begin_dungeon_supertile_transition_work_with_palette(
                DungeonSupertileTransitionWork::QuadrantUploadCallerReturn,
                palette_filter_loop_master_cycles,
            );
            debug_assert!(scheduled);
            return;
        }
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_InterRoomTrans_State12`
    pub(super) fn dungeon_inter_room_trans_state12(&mut self) {
        self.stage_room_72_supertile_landing_obj_scanout();
        if self.game_state.frame.submodule == 2 {
            if self.overworld_map_state() != 5 {
                return;
            }
            self.SubtileTransitionCalculateLanding();
            if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
                self.apply_palette_filter_bounce();
            }
        }
        self.increment_subsubmodule();
        self.dungeon_reset_torch_background_and_player();
    }

    /// ROM: `Dungeon_Staircase14`
    pub(super) fn dungeon_staircase14(&mut self) {
        self.increment_subsubmodule();
        self.dungeon_reset_torch_background_and_player();
    }

    /// ROM: `Dungeon_InterRoomTrans_State7`
    pub(super) fn dungeon_inter_room_trans_state7(&mut self) {
        self.MirrorBg1Bg2Offs();
        if self.game_state.world.location.dungeon_room() != 54
            && self.game_state.world.location.dungeon_room() != 56
        {
            let y = if SPIRAL_SUBSCREEN_LAYER_BY_BG2
                [self.game_state.dungeon.room_load.bg2_properties() as usize]
                != 0
            {
                0x0116
            } else {
                0x0016
            };
            let tm_ts = self.game_state.display.main_screen_layers as u16
                | ((self.game_state.display.sub_screen_layers as u16) << 8);
            if y != tm_ts
                && (self.game_state.display.main_screen_layers == 0x17
                    || (self.game_state.display.main_screen_layers
                        | self.game_state.display.sub_screen_layers)
                        != 0x17)
            {
                self.set_main_screen_layers(y as u8);
                self.set_sub_screen_layers((y >> 8) as u8);
            }
        }
        self.dungeon_transition_run_filtering();
    }

    /// ROM: `DungeonTransition_FindSubtileLanding`
    pub(super) fn dungeon_transition_find_subtile_landing(&mut self) {
        self.dungeon_reset_torch_background_and_player_inner();
        self.SubtileTransitionCalculateLanding();
        self.increment_subsubmodule();
        let room = self.game_state.world.location.dungeon_room() as usize;
        let visited = self.game_state.dungeon.room_load.quadrants_visited();
        self.add_saved_room_flags(room, visited);
    }

    pub(super) fn SubtileTransitionCalculateLanding(&mut self) {
        let st = self.screen_transition();
        let mut a = self.CalculateTransitionLanding();
        if a == 2 {
            a = 1;
        } else if a == 4 {
            a = 2;
        }
        let index = a as usize + self.screen_transition() as usize * 5;
        let mut v = STAIRCASE_LANDING_COORDINATES[index];
        if v < 0 {
            v = v.wrapping_add(8);
        } else {
            v = v.wrapping_sub(8);
        }
        if st & 2 != 0 {
            self.follower_link_state_mut().set_x_low(v as u8);
        } else {
            self.follower_link_state_mut().set_y_low(v as u8);
        }
        self.follower_link_state_mut().set_visibility_status(0);
    }

    /// ROM: `Dungeon_IntraRoomTrans_State5`
    pub(super) fn dungeon_intra_room_trans_state5(&mut self) {
        self.link_handle_moving_animation_full_long_entry();
        if !self.dungeon_transition_move_link_out_door() {
            return;
        }
        if self.game_state.dungeon.room_runtime.landing_class_is_pit() {
            self.follower_link_state_mut().clear_doorway_state();
        }
        self.follower_link_state_mut()
            .set_force_move_any_direction(0);
        self.dungeon_room_runtime_mut().clear_landing_class();
        self.clear_screen_transition();
        self.increment_subsubmodule();
    }

    /// ROM: `DungeonTransition_MoveLinkOutDoor`
    pub(super) fn dungeon_transition_move_link_out_door(&mut self) -> bool {
        let index = self.game_state.dungeon.room_runtime.landing_class() as usize
            + self.screen_transition() as usize * 5;
        let target = STAIRCASE_LANDING_COORDINATES[index] as u8;
        let step = if self.screen_transition() & 1 != 0 {
            (-2i16) as u16
        } else {
            2
        };
        if self.screen_transition() & 2 == 0 {
            let y = self.game_state.player.follower_link.y().wrapping_add(step);
            self.follower_link_state_mut().set_y(y);
            (y as u8 & 0xfe) == target
        } else {
            let x = self.game_state.player.follower_link.x().wrapping_add(step);
            self.follower_link_state_mut().set_x(x);
            (x as u8 & 0xfe) == target
        }
    }

    /// ROM: `DungeonTransition_Subtile_PrepTransition`
    pub(super) fn dungeon_transition_subtile_prep_transition(&mut self) {
        self.set_darkening_or_lightening_screen_word(0);
        self.set_countdown_word(0);
        self.set_mosaic_target_level(31);
        self.dungeon_room_runtime_mut().clear_reserved_gfx_config();
        self.dungeon_environment_mut()
            .clear_somaria_block_switch_counter();
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        self.increment_subsubmodule();
    }

    /// ROM: `DungeonTransition_Subtile_ApplyFilter`
    pub(super) fn dungeon_transition_subtile_apply_filter(&mut self) {
        if self.game_state.dungeon.torch.wants_lights_out() == 0 {
            self.increment_subsubmodule();
            self.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
            return;
        }
        self.apply_palette_filter_bounce();
        if !self.rom_startup_timing() && self.game_state.display.palette_filter.countdown() != 0 {
            self.apply_palette_filter_bounce();
        }
        self.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
    }

    /// ROM: `DungeonTransition_Subtile_ResetShutters`
    pub(super) fn dungeon_transition_subtile_reset_shutters(&mut self) {
        self.dungeon_environment_mut().set_trapdoors_down_low(0);
        self.dungeon_doors_mut().set_door_animation_step_low(7);
        let bak = self.game_state.frame.submodule;
        self.OperateShutterDoors();
        self.set_submodule(bak);
        self.set_countdown(31);
        self.clear_mosaic_target_level();
        self.increment_subsubmodule();
    }

    /// ROM: `DungeonTransition_Subtile_TriggerShutters`
    pub(super) fn dungeon_transition_subtile_trigger_shutters(&mut self) {
        self.ResetThenCacheRoomEntryProperties();
        if self.game_state.dungeon.environment.trapdoors_down_low() == 0 {
            self.dungeon_environment_mut().set_trapdoors_down_low(1);
            self.dungeon_doors_mut().clear_current_door_pos();
            self.dungeon_doors_mut().clear_door_animation_step();
            self.set_submodule(5);
        }
    }

    /// ROM: `DungeonTransition_RunFiltering`
    pub(super) fn dungeon_transition_run_filtering(&mut self) {
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            let torch = if self.game_state.dungeon.torch.wants_lights_out() != 0 {
                self.game_state.dungeon.torch.lit_torches() as usize
            } else {
                3
            };
            self.dungeon_room_effects_mut()
                .set_fixed_color_plusminus(DUNGEON_LIT_TORCH_COLOR_PLUS[torch]);
            self.dungeon_approach_fixed_color_variable(
                self.game_state.dungeon.room_effects.fixed_color_plusminus(),
            );
            self.clear_mosaic_target_level();
        }
        self.dungeon_handle_translucency_and_palette();
    }

    /// The isolated ROM CPU estimate may place this host's vblank inside
    /// Sprite_Main while the live wire has already published the iteration's
    /// `SpriteMainReturned` with no suspension checkpoint: the raster phase
    /// drifted into the body. The wire then also settles where the caller
    /// stopped — a completed shared suffix means the iteration finished, and
    /// an outstanding suffix means the trailing acceptance landed after
    /// Sprite_Main returned (route host 205020, the state-14 faded filter).
    fn supersede_dungeon_sprite_main_estimate_by_wire(
        &self,
        advance: DungeonModuleCpuAdvance,
    ) -> DungeonModuleCpuAdvance {
        let estimate_stops_in_sprite_main = matches!(
            advance.phase,
            ModuleCpuPhase::InterruptedBeforeSpriteMain | ModuleCpuPhase::InterruptedInSpriteMain
        );
        let estimate_stops_after_sprite_main = matches!(
            advance.phase,
            ModuleCpuPhase::InterruptedAfterSpriteMain
                | ModuleCpuPhase::InterruptedInLinkOam
                | ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites
                | ModuleCpuPhase::InterruptedInNmiPrepareSprites
        );
        if !matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !(estimate_stops_in_sprite_main || estimate_stops_after_sprite_main)
            || !self.original_timing_owes_sprite_main_return()
            || self.original_timing_owes_sprite_main_progress()
            // A cached-sprite execution checkpoint names the exact suspended
            // statement inside the module body; that receipt owns the
            // continuation through `arm_dungeon_sprite_main_cpu_continuation`
            // (route host 27133).
            || self.original_timing_semantic_receipts.as_ref().is_some_and(|receipts| {
                receipts.semantic().iter().any(|receipt| {
                    matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(_)
                    )
                })
            })
        {
            return advance;
        }
        // The fresh iteration's return timeline may already have consumed
        // its suffix receipts before the module body runs, so decide from the
        // still-installed facts: a trailing Held acceptance with no completed
        // suffix leaves the caller stopped after Sprite_Main; anything else
        // proves the iteration finished in-host (route hosts 205020, 27139).
        let semantic = self
            .original_timing_semantic_receipts
            .as_ref()
            .map(|receipts| receipts.semantic().to_vec())
            .unwrap_or_default();
        let gates = &self.original_timing_expected_nmi_update_gates;
        let trailing_held = gates.contains(&NmiUpdateGate::LatchHeld)
            || semantic.iter().any(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld)
                )
            });
        let completed = gates.contains(&NmiUpdateGate::Open)
            || semantic.iter().any(|receipt| {
                matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted
                        | OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open)
                )
            });
        let phase = if estimate_stops_after_sprite_main {
            // A post-Sprite_Main estimate stands unless the wire proves the
            // iteration completed in-host (route host 85259, the state-14
            // faded filter's NMI_PrepareSprites estimate against a completed
            // suffix and trailing Open acceptance).
            if completed {
                ModuleCpuPhase::CompleteBeforeNmi
            } else {
                return advance;
            }
        } else if trailing_held && !completed {
            ModuleCpuPhase::InterruptedAfterSpriteMain
        } else {
            ModuleCpuPhase::CompleteBeforeNmi
        };
        DungeonModuleCpuAdvance {
            phase,
            sprite_main_boundary: None,
            ..advance
        }
    }

    /// ROM: `Module07_02_FadedFilter`
    pub(in crate::zelda_rtl) fn module07_02_faded_filter(&mut self) {
        let entry_subsubmodule = self.game_state.frame.subsubmodule;
        if self.game_state.dungeon.torch.any_lights_out_request() != 0 {
            if self.rom_startup_timing() {
                // The interrupt uploads the palette generation that entered
                // this walk. Preserve it explicitly before the translated
                // filter advances the main palette buffer atomically.
                self.retain_palette_filter_input_cgram_on_next_display_capture();
            }
            self.apply_palette_filter_bounce();
            let cpu_advance = self
                .take_dungeon_landing_cpu_advance()
                .map(|advance| self.supersede_dungeon_sprite_main_estimate_by_wire(advance));
            let sprite_main_continuation_armed = cpu_advance
                .is_some_and(|advance| self.arm_dungeon_sprite_main_cpu_continuation(advance));
            if self.game_state.display.palette_filter.countdown() != 0 {
                if self.rom_startup_timing() {
                    let Some(advance) = cpu_advance else {
                        panic!(
                            "timed faded-filter entry {entry_subsubmodule} has no CPU continuation provenance"
                        );
                    };
                    if advance.phase == ModuleCpuPhase::InterruptedInSubmodule {
                        // The CPU completed the first palette walk and reached
                        // vblank during the second. The translated first pass
                        // is therefore the exact continuation boundary.
                        assert_eq!(advance.subsubmodule, self.game_state.frame.subsubmodule);
                        assert_eq!(
                            advance.palette_countdown,
                            self.game_state.display.palette_filter.countdown(),
                        );
                        self.stage_dungeon_faded_filter_first_palette_scanout();
                        self.schedule_pre_main_caller_continuation(
                            PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
                                resumed_phase: advance.resumed_phase.expect(
                                    "interrupted palette pass must measure its resumed CPU phase",
                                ),
                            },
                        );
                        return;
                    }
                    assert_eq!(
                        advance.phase,
                        ModuleCpuPhase::CompleteBeforeNmi,
                        "faded-filter second-pass continuation is not implemented for {:?}",
                        advance.phase,
                    );
                }
                self.complete_dungeon_faded_filter_second_palette_pass();
            } else if self.rom_startup_timing() {
                let caller_return_crosses_nmi = match cpu_advance {
                    Some(advance) if advance.phase == ModuleCpuPhase::InterruptedAfterModule => {
                        true
                    }
                    Some(advance) if advance.phase == ModuleCpuPhase::InterruptedInSubmodule => {
                        panic!(
                            "state-14 vblank interrupted the first palette pass; a range \
                            continuation is required before executing it"
                        )
                    }
                    Some(advance)
                        if advance.phase == ModuleCpuPhase::InterruptedInNmiPrepareSprites =>
                    {
                        self.dungeon_nmi_prepare_sprites_return_pending = true;
                        false
                    }
                    Some(_) | None => false,
                };
                if matches!(
                    cpu_advance,
                    Some(DungeonModuleCpuAdvance {
                        phase: ModuleCpuPhase::InterruptedAfterSpriteMain
                            | ModuleCpuPhase::InterruptedInLinkOam,
                        ..
                    })
                ) {
                    self.stage_dungeon_faded_filter_first_palette_scanout();
                    self.dungeon_post_sprite_main_return_pending = true;
                }
                if entry_subsubmodule == 2
                    && !sprite_main_continuation_armed
                    && !self.dungeon_nmi_prepare_sprites_return_pending
                    // A live host whose wire leaves the shared suffix
                    // outstanding (Sprite_Main returned, no suffix, no
                    // trailing acceptance; route host 256363) has not reached
                    // the caller's return: the next host's continued return
                    // owns the suffix, not a leading-NMI wait.
                    && !self.original_timing_live_fresh_iteration_suffix_outstanding()
                {
                    // The room-load walk returns immediately before a leading
                    // hardware NMI. Preserve that host boundary explicitly:
                    // the next fresh module iteration does not begin until the
                    // following host frame.
                    self.schedule_dungeon_faded_filter_completion_nmi();
                } else if entry_subsubmodule == 14 {
                    // The direction-toggle pass advances to state 15 and
                    // reaches a leading OAM DMA; a penultimate zero-count pass
                    // remains interrupted in state 14 and retains the older
                    // resident generation.
                    if self.game_state.frame.subsubmodule != entry_subsubmodule {
                        self.stage_dungeon_faded_filter_completion_scanout();
                    } else {
                        self.stage_dungeon_faded_filter_first_palette_scanout();
                    }
                    self.dungeon_faded_filter_palette_completion_host_frame =
                        Some(self.frame_ctr_dbg);
                    if caller_return_crosses_nmi {
                        let work = if self.game_state.frame.subsubmodule != entry_subsubmodule {
                            DungeonSupertileTransitionWork::FadedFilterCallerReturn
                        } else {
                            DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn
                        };
                        let scheduled = self.begin_dungeon_supertile_transition_work(work);
                        debug_assert!(scheduled);
                    }
                }
            }
        } else {
            self.increment_subsubmodule();
            let cpu_advance = self
                .take_dungeon_landing_cpu_advance()
                .map(|advance| self.supersede_dungeon_sprite_main_estimate_by_wire(advance));
            if let Some(advance) = cpu_advance {
                self.arm_dungeon_sprite_main_cpu_continuation(advance);
            }
            let caller_return_crosses_nmi = match cpu_advance {
                Some(advance) if advance.phase == ModuleCpuPhase::InterruptedAfterModule => true,
                Some(advance) if advance.phase == ModuleCpuPhase::InterruptedInSubmodule => {
                    panic!(
                        "state-14 vblank interrupted the translated module body; a body \
                         continuation is required before executing it"
                    )
                }
                Some(advance)
                    if matches!(
                        advance.phase,
                        ModuleCpuPhase::InterruptedAfterSpriteMain
                            | ModuleCpuPhase::InterruptedInLinkOam
                    ) =>
                {
                    self.stage_dungeon_faded_filter_first_palette_scanout();
                    self.dungeon_post_sprite_main_return_pending = true;
                    false
                }
                Some(advance)
                    if advance.phase == ModuleCpuPhase::InterruptedInNmiPrepareSprites =>
                {
                    self.dungeon_nmi_prepare_sprites_return_pending = true;
                    false
                }
                Some(_) | None => false,
            };
            if self.rom_startup_timing() && entry_subsubmodule == 14 && caller_return_crosses_nmi {
                // The shadow CPU reached state 15 but not the Module 7 caller
                // return before vblank. Suspend that real call-stack phase so
                // state 15 cannot start one host frame early.
                self.stage_dungeon_faded_filter_completion_scanout();
                self.dungeon_faded_filter_palette_completion_host_frame = Some(self.frame_ctr_dbg);
                let scheduled = self.begin_dungeon_supertile_transition_work(
                    DungeonSupertileTransitionWork::FadedFilterCallerReturn,
                );
                debug_assert!(scheduled);
            }
        }
    }

    pub(in crate::zelda_rtl) fn arm_dungeon_sprite_main_cpu_continuation(
        &mut self,
        advance: DungeonModuleCpuAdvance,
    ) -> bool {
        let live_authority = matches!(self.original_timing_owner, OriginalTimingOwnerState::Live);
        let cached_sprite_interruption =
            self.take_authoritative_cached_sprite_interruption(advance.cached_sprite_interruption);
        if let Some((boundary, authority_boundary)) = cached_sprite_interruption {
            if !live_authority {
                assert_eq!(
                    advance.phase,
                    ModuleCpuPhase::InterruptedInSpriteMain,
                    "cached-sprite copy boundary must interrupt Sprite_Main",
                );
            }
            assert!(
                self.dungeon_cached_sprite_cpu_interruption_pending
                    .is_none(),
                "cached-sprite CPU continuation was already armed",
            );
            self.dungeon_cached_sprite_cpu_interruption_pending = Some(boundary);
            self.dungeon_cached_sprite_cpu_interruption_boundary = authority_boundary;
            return true;
        }
        let Some(boundary) = advance.sprite_main_boundary else {
            return false;
        };
        assert!(
            matches!(
                advance.phase,
                ModuleCpuPhase::InterruptedBeforeSpriteMain
                    | ModuleCpuPhase::InterruptedInSpriteMain
            ),
            "Sprite_Main boundary requires a returned Module 7 submodule",
        );
        assert!(
            self.sprite_main_cpu_boundary.is_none(),
            "Sprite_Main CPU continuation was already armed",
        );
        self.sprite_main_cpu_boundary = Some(boundary);
        // This CPU advance stops at the first NMI after the submodule return.
        self.sprite_main_cpu_nmi_slices = 1;
        true
    }

    /// Transfer a source-observed `UncacheAndExecuteSprite` interruption into
    /// the translated Sprite_Main continuation before consulting the legacy
    /// quadrant timing reconstruction.
    ///
    /// The receipt describes the C statement boundary directly, so a
    /// contradictory synthetic `ModuleCpuPhase` cannot demote it. This is the
    /// authority handoff which lets a later native timing backend replace
    /// Snes9x without exposing a CPU PC or raster selector to gameplay code.
    pub(super) fn arm_live_cached_sprite_main_cpu_continuation(&mut self) -> bool {
        let wire_receipt = self.take_original_timing_cached_sprite_execution_progress();
        // The submodule dispatcher and the outer Module07 caller can both
        // reach this arming point. A retained CPU receipt transfers once,
        // just like the wire receipt, rather than rearming the same caller.
        let native_receipt = self.native_exact_cpu_host_trace.as_mut()
            .and_then(|trace| trace.cached_sprite_progress.take());
        let receipt = wire_receipt.or(native_receipt);
        let Some(receipt) = receipt else {
            return false;
        };
        assert!(
            self.dungeon_cached_sprite_cpu_interruption_pending
                .replace(receipt.progress.into())
                .is_none(),
            "cached-sprite semantic continuation was already armed",
        );
        assert!(
            self.dungeon_cached_sprite_cpu_interruption_boundary
                .replace(receipt.boundary)
                .is_none(),
            "cached-sprite semantic boundary was already armed",
        );
        self.dungeon_quadrant_cpu_continuation_active = true;
        true
    }

    pub(in crate::zelda_rtl) fn apply_dungeon_quadrant_cpu_advance(&mut self, advance: DungeonModuleCpuAdvance) {
        if crate::debug_env::var_os!("ZELDA3_DEBUG_DUNGEON_CPU_SCHEDULE").is_some() {
            eprintln!(
                "dungeon_quadrant_cpu_apply host={} phase={:?} active_return={}",
                self.frame_ctr_dbg,
                advance.phase,
                self.active_dungeon_sprite_main_return.is_some(),
            );
        }
        debug_assert_eq!(advance.subsubmodule, self.game_state.frame.subsubmodule);
        debug_assert_eq!(
            advance.palette_countdown,
            self.game_state.display.palette_filter.countdown(),
        );
        if self.arm_live_cached_sprite_main_cpu_continuation() {
            return;
        }
        // The wire's Sprite_Main return and completed suffix supersede a
        // post-Sprite_Main estimate as well (route host 97801, the state-7
        // NMI_PrepareSprites estimate against a completed iteration).
        let advance = self.supersede_dungeon_sprite_main_estimate_by_wire(advance);
        match advance.phase {
            ModuleCpuPhase::CompleteBeforeNmi | ModuleCpuPhase::InterruptedAfterModule => {}
            ModuleCpuPhase::InterruptedBeforeSpriteMain
            | ModuleCpuPhase::InterruptedInSpriteMain => {
                if self.original_timing_owes_sprite_main_return()
                    && !self.original_timing_owes_sprite_main_progress()
                {
                    // The live wire still owes this fresh iteration's
                    // `SpriteMainReturned` claim with no suspension
                    // checkpoint: the ROM ran Sprite_Main to completion
                    // within this host, so the legacy quadrant timing
                    // reconstruction's interruption estimate is superseded
                    // (route host 132399, the post-room-load quadrant
                    // upload's slot-0 estimate).
                    return;
                }
                self.dungeon_quadrant_cpu_continuation_active = true;
                let armed = self.arm_dungeon_sprite_main_cpu_continuation(advance);
                assert!(armed, "Sprite_Main interruption has no semantic boundary");
            }
            ModuleCpuPhase::InterruptedAfterSpriteMain | ModuleCpuPhase::InterruptedInLinkOam => {
                self.dungeon_post_sprite_main_return_pending = true;
            }
            ModuleCpuPhase::InterruptedBeforeNmiPrepareSprites => {
                self.dungeon_state_12_caller_suffix_nmi_pending = true;
            }
            ModuleCpuPhase::InterruptedInNmiPrepareSprites => {
                self.dungeon_nmi_prepare_sprites_return_pending = true;
            }
            phase @ (ModuleCpuPhase::InterruptedBeforeSubmodule
            | ModuleCpuPhase::InterruptedInSubmodule) => {
                panic!("dungeon quadrant dispatcher ran despite a {phase:?} CPU continuation");
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_faded_filter_second_palette_pass(&mut self) {
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 2);
        debug_assert!(matches!(self.game_state.frame.subsubmodule, 2 | 14));
        self.apply_palette_filter_bounce();
    }

    /// ROM: `Dungeon_InterRoomTrans_State15`
    pub(in crate::zelda_rtl) fn dungeon_inter_room_trans_state15(&mut self) {
        // The supertile dispatcher jumps here; charge into its call frame.
        crate::cycle_ledger::charge(46); // $02:8BAE: JSR reset/cache.
        self.ResetThenCacheRoomEntryProperties();
        crate::cycle_ledger::charge(48); // $8BB1..$8BB4: trapdoor test.
        if self.game_state.dungeon.environment.trapdoors_down_low() != 0 {
            crate::cycle_ledger::charge(6); // $8BB4: taken BNE to the music tail.
        } else {
            crate::cycle_ledger::charge(56); // $8BB6..$8BBA: room comparison.
            if self.game_state.world.location.dungeon_room_index() != 172 {
                crate::cycle_ledger::charge(6); // $8BBA: taken BNE.
            } else {
                crate::cycle_ledger::charge(64); // $8BBC..$8BC1: first save bit.
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x2000 != 0 {
                    crate::cycle_ledger::charge(6); // $8BC1: taken BNE.
                } else {
                    crate::cycle_ledger::charge(64); // $8BC3..$8BC8: second save bit.
                    if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x1000 == 0 {
                        crate::cycle_ledger::charge(6); // $8BC8: taken BEQ to the music tail.
                    }
                }
            }
        }
        if self.game_state.dungeon.environment.trapdoors_down_low() == 0
            && (self.game_state.world.location.dungeon_room_index() != 172
                || self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x3000 != 0)
        {
            crate::cycle_ledger::charge(150); // $8BCA..$8BD5: trapdoor, door bytes, submodule.
            self.dungeon_environment_mut().set_trapdoors_down_low(1);
            // C writes `ram[DUNG_CUR_DOOR_POS_DUNGEON] = 0` as a byte here (0x68e only),
            // leaving the high byte 0x68f intact (a stale leftover).
            self.dungeon_doors_mut().clear_current_door_pos_low_byte();
            // $02:8BD0 is also an 8-bit STZ; $0691 remains live scratch.
            self.dungeon_doors_mut().set_door_animation_step_low(0);
            self.set_submodule(5);
        }
        self.dungeon_play_music_if_defeated();
    }

    /// ROM: `DungeonTransition_LoadSpriteGFX`
    pub(in crate::zelda_rtl) fn dungeon_transition_load_sprite_gfx(&mut self) {
        self.LoadNewSpriteGFXSet();
        // The warp-pad transition ($15) shares this exact sprite-graphics
        // reload with the spiral-stairs transition ($0e); both can hold the
        // translated caller across measured NMI slices (route host 92533).
        if self.game_state.frame.main_module == 7 {
            let work = match self.game_state.frame.submodule {
                6 | 7 => Some(DungeonSupertileTransitionWork::FallingSpriteGraphics),
                0x0e | 0x15 => Some(DungeonSupertileTransitionWork::SpiralSpriteGraphics),
                _ => None,
            };
            if work.is_some_and(|work| self.begin_dungeon_supertile_transition_work(work)) {
                return;
            }
        }
        self.complete_dungeon_transition_load_sprite_gfx();
    }

    pub(in crate::zelda_rtl) fn complete_dungeon_transition_load_sprite_gfx(&mut self) {
        self.dungeon_reset_sprites();
        self.dungeon_transition_run_filtering();
    }

    /// ROM: `Module07_04_UnlockDoor`
    pub(in crate::zelda_rtl) fn module07_04_unlock_door(&mut self) {
        self.dungeon_opening_locked_door_combined(false);
    }

    /// ROM: `Module07_03_OverlayChange`
    pub(in crate::zelda_rtl) fn module07_03_overlay_change(&mut self) {
        let overlay_index = self.game_state.dungeon.room_load.overlay_to_load() as usize;
        let overlay_offs = self.asset_u16(49, overlay_index) as usize;
        let overlay_data = self
            .asset_raw(48)
            .expect("missing dungeon room overlay asset")
            .to_vec();
        let overlay = &overlay_data[overlay_offs..];
        self.dungeon_draw_room_overlay(overlay);
        let mut dst_pos = 0usize;
        let mut offset = 0usize;
        loop {
            let marker = overlay[offset] as u16 | ((overlay[offset + 1] as u16) << 8);
            if marker == 0xffff {
                break;
            }
            let p =
                ((overlay[offset] as u16 >> 2) | ((overlay[offset + 1] as u16 >> 2) << 6)) as usize;
            dst_pos = self.dungeon_prep_overlay_dma_next_prep(dst_pos, (p * 2) as u16);
            self.dungeon_draw_room_overlay_apply(p);
            offset += 3;
        }
        self.request_nmi_copy_packets();
        self.set_submodule(0);
    }

    /// ROM: `Module07_05_ControlShutters`
    pub(in crate::zelda_rtl) fn module07_05_control_shutters(&mut self) {
        self.OperateShutterDoors();
    }

    /// ROM: `Module07_06_FatInterRoomStairs`
    pub(in crate::zelda_rtl) fn module07_06_fat_inter_room_stairs(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::FatInterRoomStairs,
            ) {
                return;
            }
            self.dungeon_load_attribute_selectable();
        }
        self.complete_module07_06_fat_inter_room_stairs_after_attribute_loader();
    }

    pub(in crate::zelda_rtl) fn complete_module07_06_fat_inter_room_stairs_after_attribute_loader(&mut self) {
        if self.game_state.frame.subsubmodule >= 13 {
            self.graphics_incremental_vram_upload();
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 0
            {
                self.module07_06_fat_inter_room_stairs_dispatch();
                return;
            }
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 0x10
            {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            let direction = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                4
            } else {
                8
            };
            self.follower_link_state_mut().set_direction(direction);
            self.link_handle_velocity();
            self.dungeon_handle_camera();
        }

        self.link_handle_moving_animation_full_long_entry();
        self.module07_06_fat_inter_room_stairs_dispatch();
    }

    /// ROM: `Module07_06_FatInterRoomStairs_dispatch`
    fn module07_06_fat_inter_room_stairs_dispatch(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.reset_transition_props_and_advance_reset_interface(),
            1 => {
                self.apply_palette_filter_bounce();
                if self.game_state.display.palette_filter.countdown() != 0 {
                    self.apply_palette_filter_bounce();
                }
            }
            2 => {
                // The fat inter-room stairs share the long
                // Dungeon_InitializeRoomFromSpecial call; the live wire can
                // hold it across vblank the same way (route host 402660).
                if !self.begin_dungeon_falling_room_initialization_work() {
                    self.dungeon_initialize_room_from_special();
                }
            }
            3 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            4 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            5 => self.dungeon_transition_load_sprite_gfx(),
            6 => self.dungeon_transition_adjust_for_fat_stair_scroll(),
            7 => self.dungeon_inter_room_trans_state4(),
            8 => self.dungeon_inter_room_trans_not_dark_room(),
            9 => self.dungeon_inter_room_trans_state4(),
            10 => self.dungeon_spiral_staircase11(),
            11 => self.dungeon_spiral_staircase12(),
            12 => self.dungeon_spiral_staircase11(),
            13 => self.dungeon_spiral_staircase12(),
            14 => self.dungeon_double_apply_and_increment_grayscale(),
            15 => self.dungeon_staircase14(),
            16 => {
                if (self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen()
                    | self.game_state.display.palette_filter.countdown())
                    == 0
                    && self.overworld_map_state() == 5
                {
                    self.ResetThenCacheRoomEntryProperties();
                }
            }
            _ => panic!("invalid fat inter-room stair index"),
        }
    }

    /// ROM: `Dungeon_InitializeRoomFromSpecial`
    pub(super) fn dungeon_initialize_room_from_special(&mut self) {
        self.dungeon_adjust_after_spiral_stairs();
        self.dungeon_initialize_room_from_special_after_adjust();
    }

    /// The room load and tileset tail of `Dungeon_InitializeRoomFromSpecial`.
    /// Under ROM timing the falling transition's wire-held work runs the
    /// spiral-stairs adjust at the scheduling host (Link's coordinates and
    /// camera land immediately, route host 91638) and only this tail at the
    /// terminal completion.
    pub(in crate::zelda_rtl) fn dungeon_initialize_room_from_special_after_adjust(&mut self) {
        if self.native_exact_cpu_owner.is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL").is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_OBJECT_DRAW_LIVE_TRIAL").is_some()
            && self.game_state.frame.main_module == 7
            && self.game_state.frame.submodule == 0x0e
        {
            // The spiral-stair caller enters the same Dungeon_LoadRoom loop as
            // Module07_02_01. Its floor and object writes belong to the CPU
            // continuation, not to the host that entered this function.
            self.dungeon_load_header();
            self.dungeon_load_room_before_floors();
            assert!(self.active_early_dungeon_floor_draw.is_none());
            self.active_early_dungeon_floor_draw = self.begin_dungeon_floor_draw_current_room();
            assert!(self.active_early_dungeon_floor_draw.is_some());
            if self.begin_dungeon_supertile_transition_work(
                DungeonSupertileTransitionWork::SpiralRoomInitialization,
            ) {
                return;
            }
            self.finish_live_dungeon_supertile_room_load();
            self.complete_spiral_room_initialization_after_room_load(false);
            self.increment_subsubmodule();
        } else {
            self.dungeon_load_room();
            self.complete_spiral_room_initialization_after_room_load(true);
        }
    }

    pub(in crate::zelda_rtl) fn complete_spiral_room_initialization_after_room_load(&mut self, schedule: bool) {
        if self.native_transition_aux_graphics_progress.is_none() {
            self.ResetStarTileGraphics();
        }
        self.LoadTransAuxGFX();
        self.complete_spiral_room_initialization_after_aux_graphics(schedule);
    }

    /// Run `ResetStarTileGraphics` and `LoadTransAuxGFX` with translated
    /// decoders, then withhold the decompressed WRAM so the source store
    /// cursor can publish it in order.
    pub(in crate::zelda_rtl) fn begin_spiral_aux_graphics_publication(&mut self) -> AuxGraphicsPublication {
        self.ResetStarTileGraphics();
        let before = self.ram[AuxGraphicsPublication::RANGE].to_vec();
        self.LoadTransAuxGFX();
        let bytes = self.ram[AuxGraphicsPublication::RANGE].to_vec();
        self.ram[AuxGraphicsPublication::RANGE].copy_from_slice(&before);
        AuxGraphicsPublication { bytes, published: 0 }
    }

    pub(in crate::zelda_rtl) fn complete_spiral_room_initialization_after_aux_graphics(&mut self, schedule: bool) {
        self.dungeon_load_custom_tile_attr();
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.follower_initialize();
        if schedule
            && self.game_state.frame.main_module == 7
            && self.game_state.frame.submodule == 0x0e
            && self.begin_dungeon_supertile_transition_work(
                DungeonSupertileTransitionWork::SpiralRoomInitialization,
            )
        {
            return;
        }
        if schedule {
            self.increment_subsubmodule();
        }
    }

    /// ROM: `DungeonTransition_AdjustForFatStairScroll`
    pub(super) fn dungeon_transition_adjust_for_fat_stair_scroll(&mut self) {
        self.MirrorBg1Bg2Offs();
        self.dungeon_adjust_for_room_layout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);

        self.follower_link_state_mut().set_speed_modifier(1);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.dungeon_stair_movement_mut().decrement_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(32);
            self.set_sound_effect_1(0x19);
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(48);
            self.set_sound_effect_1(0x17);
        }
        self.set_sound_effect_2(0x24);
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.dungeon_inter_room_trans_not_dark_room();
    }

    /// ROM: `DungeonTransition_ScrollRoom`
    pub(super) fn dungeon_transition_scroll_room(&mut self) {
        self.increment_transition_counter();
        let i = self.screen_transition() as usize;
        self.set_bg1_y_offset(0);
        self.set_bg1_x_offset(0);
        let delta = DUNGEON_TRANSITION_SCROLL_DELTAS[i] as i16 as u16;

        let t = if i >= 2 {
            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add(delta)
                & !1;
            self.set_bg2_x(t);
            self.set_bg1_x(t);
            if self.transition_counter() >= DUNGEON_TRANSITION_PLAYER_MOVE_FRAMES[i] {
                let x = self.game_state.player.follower_link.x().wrapping_add(delta);
                self.follower_link_state_mut().set_x(x);
            }
            t
        } else {
            let t = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_add(delta)
                & !1;
            self.set_bg2_y(t);
            self.set_bg1_y(t);
            if self.transition_counter() >= DUNGEON_TRANSITION_PLAYER_MOVE_FRAMES[i] {
                let y = self.game_state.player.follower_link.y().wrapping_add(delta);
                self.follower_link_state_mut().set_y(y);
            }
            t
        };

        if (t & 0x01fc)
            == self
                .game_state
                .world
                .camera_boundaries
                .up_down_scroll_target(i)
        {
            self.SetAndSaveVisitedQuadrantFlags();
            self.increment_subsubmodule();
            self.set_transition_counter(0);
            if self.game_state.frame.submodule == 2 {
                self.water_flood_build_one_quadrant_for_vram();
            }
        }
    }

    /// ROM: `DungeonTransition_TriggerBGC34UpdateAndAdvance`
    pub(in crate::zelda_rtl) fn dungeon_transition_trigger_bgc34_update_and_advance(&mut self) {
        self.PrepTransAuxGfx();
        // The conversion can span several CPU hosts. The ROM publishes the
        // DMA request only after it finishes, so the source CPU receipt owns
        // these two stores while the translated conversion is suspended.
        if self.native_exact_cpu_owner.is_none()
            || crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_BG_CHARS_GATE_LIVE_TRIAL")
                .is_none()
        {
            self.set_pending_nmi_subroutine(9);
            self.set_core_update_disable_flag(9);
        }
        if self.game_state.frame.main_module == 7 {
            let work = match self.game_state.frame.submodule {
                // Fat stairs and falling transitions call the same helper;
                // neither caller may advance while its conversion is suspended.
                6 | 7 => Some(DungeonSupertileTransitionWork::FallingBgCharacters34),
                0x0e => Some(DungeonSupertileTransitionWork::SpiralBgCharacters34),
                _ => None,
            };
            if work.is_some_and(|work| self.begin_dungeon_supertile_transition_work(work)) {
                return;
            }
        }
        self.increment_subsubmodule();
    }

    /// ROM: `DungeonTransition_TriggerBGC56UpdateAndAdvance`
    pub(super) fn dungeon_transition_trigger_bgc56_update_and_advance(&mut self) {
        if self.native_exact_cpu_owner.is_none()
            || crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_BG_CHARS_GATE_LIVE_TRIAL")
                .is_none()
        {
            self.set_pending_nmi_subroutine(10);
            self.set_core_update_disable_flag(10);
        }
        self.increment_subsubmodule();
    }

    /// ROM: `Dungeon_TryScreenEdgeTransition`
    pub(in crate::zelda_rtl) fn dungeon_try_screen_edge_transition(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_885e);
        crate::cycle_ledger::charge(94); // REP, Y-velocity test.
        let mut dir = None;

        if self.game_state.player.follower_link.y_velocity() != 0 {
            crate::cycle_ledger::charge(200); // Axis direction, masked Y, north comparison.
            let y = self.game_state.player.follower_link.y() & 0x01ff;
            if y < 4 {
                crate::cycle_ledger::charge(6);
                dir = Some(3);
            } else {
                crate::cycle_ledger::charge(56); // South comparison.
                if y >= 476 {
                    crate::cycle_ledger::charge(6);
                    dir = Some(2);
                }
            }
        } else {
            crate::cycle_ledger::charge(6);
        }

        if dir.is_none() {
            crate::cycle_ledger::charge(72); // X-velocity test.
            if self.game_state.player.follower_link.x_velocity() != 0 {
                crate::cycle_ledger::charge(200); // Axis direction, masked X, west comparison.
                let x = self.game_state.player.follower_link.x() & 0x01ff;
                if x < 8 {
                    crate::cycle_ledger::charge(6);
                    dir = Some(1);
                } else {
                    crate::cycle_ledger::charge(56); // East comparison.
                    if x >= 489 {
                        dir = Some(0);
                    } else {
                        crate::cycle_ledger::charge(6);
                    }
                }
            } else {
                crate::cycle_ledger::charge(6);
            }
        }

        let Some(dir) = dir else {
            crate::cycle_ledger::charge(64); // Shared SEP : RTS.
            return;
        };

        crate::cycle_ledger::charge(84); // SEP : JSL predicate.
        let blocked = self.link_check_for_edge_screen_transition();
        crate::cycle_ledger::charge(16); // BCS after the predicate returns.
        if blocked {
            crate::cycle_ledger::charge(6);
        } else {
            crate::cycle_ledger::charge(56); // Main-module gate.
            if self.game_state.frame.main_module == 7 {
                crate::cycle_ledger::charge(62);
                self.dungeon_handle_edge_transition_movement(dir);
                self.dungeon_finish_screen_edge_after_movement();
                return;
            } else {
                crate::cycle_ledger::charge(6);
            }
        }
        crate::cycle_ledger::charge(64);
    }

    pub(super) fn dungeon_finish_screen_edge_after_movement(&mut self) {
        crate::cycle_ledger::charge(56); // Recheck after movement (which can exit the dungeon).
        if self.game_state.frame.main_module == 7 {
            crate::cycle_ledger::charge(40);
            self.set_submodule(2);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(64);
    }
    /// ROM: `Dungeon_HandleEdgeTransitionMovement`
    pub(super) fn dungeon_handle_edge_transition_movement(&mut self, dir: u8) {
        let _scope = crate::cycle_ledger::routine(0x02_88c5);
        self.follower_link_state_mut().mask_direction(
            DUNGEON_HANDLE_EDGE_TRANSITION_MOVEMENT_LIMIT_DIRECTION_ON_ONE_AXIS[dir as usize],
        );
        match dir {
            0 => self.dungeon_start_inter_room_trans_right(),
            1 => self.dungeon_start_inter_room_trans_left(),
            2 => self.dungeon_start_inter_room_trans_down(),
            3 => self.dungeon_start_inter_room_trans_up(),
            _ => unreachable!(),
        }
    }

    /// ROM: `Dungeon_AdjustAfterSpiralStairs`
    pub(in crate::zelda_rtl) fn dungeon_adjust_after_spiral_stairs(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let prev = self
            .game_state
            .dungeon
            .room_tracking
            .previous_room_index_word();
        let xd = ((room & 0x000f) as i32 - (prev & 0x000f) as i32) * 0x200;
        self.add_dungeon_room_delta_x(xd as i16 as u16);

        let yd = (((room & 0x00f0) >> 4) as i32 - ((prev & 0x00f0) >> 4) as i32) * 0x200;
        self.add_dungeon_room_delta_y(yd as i16 as u16);
    }

    /// ROM: `Dungeon_AdjustForTeleportDoors`
    pub(super) fn dungeon_adjust_for_teleport_doors(&mut self, room: u8, flag: u8) {
        self.dungeon_room_tracking_mut()
            .set_room_index2_word(room as u16);
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(room as u16);

        let link_x_hi = self.game_state.player.follower_link.x() >> 8;
        let xx = ((room & 0x0f) as u16)
            .wrapping_mul(2)
            .wrapping_sub(link_x_hi)
            .wrapping_add(flag as u16);
        self.add_dungeon_room_delta_x(xx << 8);

        let link_y_hi = self.game_state.player.follower_link.y() >> 8;
        let yy = (((room & 0xf0) >> 3) as u16).wrapping_sub(link_y_hi);
        self.add_dungeon_room_delta_y(yy << 8);

        let y_hi = (self.game_state.player.follower_link.y() >> 8) as u8;
        for i in 0..20 {
            self.tagalong_slot_mut(i).set_y_high(y_hi);
        }
    }

    fn add_dungeon_room_delta_x(&mut self, delta: u16) {
        let link_x = self.game_state.player.follower_link.x().wrapping_add(delta);
        self.follower_link_state_mut().set_x(link_x);
        self.add_bg2_h_copy2(delta);
        for index in 0..4 {
            let value = self
                .game_state
                .world
                .room_bounds
                .x_bound(index)
                .wrapping_add(delta);
            self.room_bounds_mut().set_x_bound(index, value);
        }
    }

    fn add_dungeon_room_delta_y(&mut self, delta: u16) {
        let link_y = self.game_state.player.follower_link.y().wrapping_add(delta);
        self.follower_link_state_mut().set_y(link_y);
        self.add_bg2_v_copy2(delta);
        for index in 0..4 {
            let value = self
                .game_state
                .world
                .room_bounds
                .y_bound(index)
                .wrapping_add(delta);
            self.room_bounds_mut().set_y_bound(index, value);
        }
    }
}
