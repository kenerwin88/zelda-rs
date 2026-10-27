//! Spiral and straight stairs, falling entrances, and landing wipes.

use super::*;
use crate::tile_definition::{DungeonRole, NativeTile};
use crate::zelda_rtl::misc::DUNG_ANIMATED_TILES;

impl ZeldaState {

    /// ROM: `Module07_0E_01_HandleMusicAndResetProps`
    pub(super) fn module07_0e_01_handle_music_and_reset_props(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        if (room == 7 || (room == 23 && !self.zelda_is_playing_music_track(17)))
            && self.game_state.inventory.player_resources.pendant_flags() & 1 == 0
        {
            self.set_music_control(0xf1);
        }
        let staircase_move_counter =
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                106
            } else {
                88
            };
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(staircase_move_counter);
        self.set_overworld_map_state(0);
        self.ResetTransitionPropsAndAdvanceSubmodule();
    }

    /// ROM: `Module07_0E_02_ApplyFilterIf`
    pub(in crate::zelda_rtl) fn module07_0e_02_apply_filter_if(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            < 9
        {
            self.apply_palette_filter_bounce();
            if self.game_state.display.palette_filter.countdown() != 0 {
                if self.rom_startup_timing() {
                    // The two full palette walks straddle vblank on hardware.
                    // Preserve the translated call stack here: the following
                    // host frame resumes at the second pass instead of running
                    // a fresh game-loop iteration.
                    self.schedule_pre_main_caller_continuation(
                        PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
                    );
                    return;
                }
                self.apply_palette_filter_bounce();
            }
        }
        self.complete_spiral_stairs_palette_filter_tail();
    }

    pub(in crate::zelda_rtl) fn complete_spiral_stairs_second_palette_filter(&mut self) {
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 0x0e);
        debug_assert_eq!(self.game_state.frame.subsubmodule, 2);
        self.apply_palette_filter_bounce();
        self.complete_spiral_stairs_palette_filter_tail();
    }

    fn complete_spiral_stairs_palette_filter_tail(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            return;
        }
        self.follower_state_mut().set_appearance_none_flag(12);
        self.follower_link_state_mut().set_visibility_status(12);
    }

    /// ROM: `Dungeon_AdvanceThenSetBossMusicUnorthodox`
    pub(super) fn dungeon_advance_then_set_boss_music_unorthodox(&mut self) {
        self.dungeon_reset_torch_background_and_player_inner();
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(0x38);
        self.increment_subsubmodule();
        self.dungeon_set_boss_music_unorthodox();
    }

    /// ROM: `Dungeon_SetBossMusicUnorthodox`
    pub(super) fn dungeon_set_boss_music_unorthodox(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        let mut x = 0x1c;
        if room != 16 {
            x = 0x15;
            if room != 7 {
                x = 0x11;
                if room != 23 || self.zelda_is_playing_music_track(17) {
                    return;
                }
            }
            if self.game_state.system_signals.current_music_control() != 0xf1
                && self.game_state.inventory.player_resources.pendant_flags() & 1 != 0
            {
                return;
            }
        }
        self.set_music_control(x);
    }

    /// ROM: `Module07_0E_SpiralStairs`
    pub(in crate::zelda_rtl) fn module07_0e_spiral_stairs(&mut self) {
        if self.lag_free {
            // $02:9054..906B: conditional upload/attribute JSLs, Link JSL,
            // state load and JumpTableLocal JSL. Table bytes are not code.
            crate::cycle_ledger::charge(56 + if self.game_state.frame.subsubmodule >= 7 { 124 } else { 6 });
        }
        if self.game_state.frame.subsubmodule >= 7 {
            self.graphics_incremental_vram_upload();
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::SpiralStairs,
            ) {
                return;
            }
            self.dungeon_load_attribute_selectable();
        }
        self.complete_module07_0e_spiral_stairs_after_attribute_loader();
    }

    pub(in crate::zelda_rtl) fn complete_module07_0e_spiral_stairs_after_attribute_loader(&mut self) {
        if self.lag_free {
            // $02:9062..906B and $00:8781..8799 JumpTableLocal's executed
            // return-address/table dispatch (414); no inline table bytes.
            crate::cycle_ledger::charge(62 + 24 + 62 + 414);
        }
        self.HandleLinkOnSpiralStairs();
        match self.game_state.frame.subsubmodule {
            0 => self.module07_0e_00_init_priority_and_screens(),
            1 => self.module07_0e_01_handle_music_and_reset_props(),
            2 => self.module07_0e_02_apply_filter_if(),
            3 => self.dungeon_initialize_room_from_special(),
            4 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            5 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            6 => self.dungeon_transition_load_sprite_gfx(),
            7 => self.dungeon_sync_backgrounds_from_spiral_stairs(),
            8 => self.dungeon_inter_room_trans_state4(),
            9 => self.dungeon_inter_room_trans_not_dark_room(),
            10 => self.dungeon_inter_room_trans_state4(),
            11 => self.dungeon_spiral_staircase11(),
            12 => self.dungeon_spiral_staircase12(),
            13 => self.dungeon_spiral_staircase11(),
            14 => self.dungeon_spiral_staircase12(),
            15 => self.dungeon_double_apply_and_increment_grayscale(),
            16 => self.dungeon_advance_then_set_boss_music_unorthodox(),
            17 => self.dungeon_spiral_staircase17(),
            18 => self.dungeon_spiral_staircase18(),
            19 => self.module07_0e_13_set_room_and_layer_and_cache(),
            other => panic!("invalid Module07_0E_SpiralStairs subsubmodule_index {other}"),
        }
    }

    /// ROM: `Dungeon_SyncBackgroundsFromSpiralStairs`
    pub(super) fn dungeon_sync_backgrounds_from_spiral_stairs(&mut self) {
        if self.game_state.sprites.follower_runtime.indicator() == 6
            && self.game_state.world.location.dungeon_room_index() == 100
        {
            self.follower_state_mut().set_indicator(0);
        }
        let bak = self.game_state.player.follower_link.lower_level_state();
        let y_delta = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            48
        } else {
            (-48i16) as u16
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta);
        self.follower_link_state_mut().set_y(y);
        let lower_level_state = TELEPORT_PIT_SECONDARY_LEVELS[self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane()
            as usize];
        self.follower_link_state_mut()
            .set_lower_level_state(lower_level_state);
        self.spiral_stairs_make_nearby_walls_high_priority_exiting();
        self.follower_link_state_mut().set_lower_level_state(bak);
        let y_delta = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            (-48i16) as u16
        } else {
            48
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta);
        self.follower_link_state_mut().set_y(y);
        self.copy_bg2_live_to_bg1_live();
        self.dungeon_adjust_for_room_layout();
        let mut ts = SPIRAL_SUBSCREEN_LAYER_BY_BG2
            [self.game_state.dungeon.room_load.bg2_properties() as usize];
        let mut tm = 0x16;
        if ts < 0 {
            tm = 0x17;
            ts = 0;
        }
        if self.game_state.dungeon.room_load.bg2_properties() == 2 {
            ts = 3;
        }
        self.set_main_screen_layers(tm);
        self.set_sub_screen_layers(ts as u8);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.dungeon_stair_movement_mut().decrement_current_floor();
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(24);
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.hud_restore_torch_background();
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpiralBackgroundSync,
        ) {
            return;
        }
        self.dungeon_inter_room_trans_not_dark_room();
    }

    /// ROM: `Dungeon_SpiralStaircase17`
    pub(super) fn dungeon_spiral_staircase17(&mut self) {
        self.spiral_stairs_find_landing_spot();
        self.dungeon_stair_movement_mut()
            .decrement_staircase_move_counter();
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            let staircase_move_counter =
                if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                    10
                } else {
                    24
                };
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(staircase_move_counter);
            self.increment_subsubmodule();
        }
    }

    /// ROM: `Dungeon_SpiralStaircase18`
    pub(super) fn dungeon_spiral_staircase18(&mut self) {
        self.spiral_stairs_find_landing_spot();
        self.dungeon_stair_movement_mut()
            .decrement_staircase_move_counter();
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            self.increment_subsubmodule();
            self.set_overworld_map_state(0);
        }
    }

    /// ROM: `Module07_0E_00_InitPriorityAndScreens`
    pub(super) fn module07_0e_00_init_priority_and_screens(&mut self) {
        self.spiral_stairs_make_nearby_walls_high_priority_entering();
        if self.game_state.player.follower_link.is_on_lower_level() {
            self.and_main_screen_layers(0x0f);
            self.or_sub_screen_layers(0x10);
            self.follower_link_state_mut().set_lower_level_state(3);
        }
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_0E_13_SetRoomAndLayerAndCache`
    pub(super) fn module07_0e_13_set_room_and_layer_and_cache(&mut self) {
        let reenter_player_control = if self.rom_startup_timing() {
            let schedule = self
                .dungeon_submodule_cpu_schedule
                .take()
                .expect("spiral return requires its pre-NMI CPU schedule");
            assert_eq!(
                schedule.submodule_nmis, 0,
                "spiral return body crossed NMI before its module transition"
            );
            schedule.reenters_main_loop_before_nmi
        } else {
            false
        };
        let plane = self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane() as usize;
        self.follower_link_state_mut().set_lower_level_states(
            TELEPORT_PIT_SECONDARY_LEVELS[plane],
            TELEPORT_PIT_PRIMARY_LEVELS[plane],
        );
        self.or_main_screen_layers(0x10);
        self.and_sub_screen_layers(0x0f);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 == 0 {
            self.spiral_stairs_make_nearby_walls_low_priority();
        }
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.ResetThenCacheRoomEntryProperties();
        if reenter_player_control {
            self.spiral_stair_return_player_control_pending = true;
        }
    }

    pub(super) fn RepositionLinkAfterSpiralStairs(&mut self) {
        self.follower_link_state_mut().set_visibility_status(0);
        self.follower_state_mut().set_appearance_none_flag(0);
        let mut i = if self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane()
            == 0
            && self
                .game_state
                .dungeon
                .stair_movement
                .staircase_lower_level_status()
                != 0
        {
            1usize
        } else {
            0usize
        };
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            i += 2;
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPIRAL_STAIRCASE_X_OFFSETS[i] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPIRAL_STAIRCASE_Y_OFFSETS[i] as i16 as u16);
        self.follower_link_state_mut().set_x(x);
        self.follower_link_state_mut().set_y(y);

        if self.game_state.display.main_screen_layers & 0x10 != 0 {
            if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                == 2
            {
                self.follower_link_state_mut().set_lower_level_state(3);
                self.and_main_screen_layers(0x0f);
                self.or_sub_screen_layers(0x10);
                if self
                    .game_state
                    .dungeon
                    .stair_movement
                    .staircase_lower_level_status()
                    != 2
                {
                    let y = self.game_state.player.follower_link.y().wrapping_add(24);
                    self.follower_link_state_mut().set_y(y);
                }
            }
            self.follower_initialize();
        } else {
            if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                != 2
            {
                self.or_main_screen_layers(0x10);
                self.and_sub_screen_layers(0x0f);
                if self
                    .game_state
                    .dungeon
                    .stair_movement
                    .staircase_lower_level_status()
                    != 2
                {
                    let y = self.game_state.player.follower_link.y().wrapping_sub(24);
                    self.follower_link_state_mut().set_y(y);
                }
            }
            self.follower_initialize();
        }
    }

    /// ROM: `Dungeon_PlayMusicIfDefeated`
    pub(super) fn dungeon_play_music_if_defeated(&mut self) {
        // $02:8BD7 is the preceding transition's tail, not another call.
        crate::cycle_ledger::charge(110); // $8BD7..$8BE0: widths, room and first branch.
        let room = self.game_state.world.location.dungeon_room();
        let mut x = 0x14;
        if room != 18 {
            crate::cycle_ledger::charge(56); // $8BE2..$8BE7: second room comparison.
            x = 0x10;
            if room != 2 {
                crate::cycle_ledger::charge(16); // $8BE9: LDX #$18.
                // The source scans twelve words backwards, including its
                // final DEX pair and taken BMI when no entry matches.
                for candidate in DUNGEON_BOSS_ROOMS.iter().rev() {
                    crate::cycle_ledger::charge(108); // $8BEB..$8BF3, branches not taken.
                    if *candidate == room { break; }
                    crate::cycle_ledger::charge(6); // $8BF3: taken BNE.
                }
                if !DUNGEON_BOSS_ROOMS.contains(&room) {
                    crate::cycle_ledger::charge(50 + 64); // $8BEB..$8BED taken BMI; $8C02..$8C04.
                    return;
                }
                crate::cycle_ledger::charge(84); // $8BF5..$8BF7: SEP, JSL.
                if self.sprite_check_if_screen_is_clear() {
                    crate::cycle_ledger::charge(22 + 64); // $8BFB taken BCS; $8C02..$8C04.
                    return;
                }
                crate::cycle_ledger::charge(16 + 16); // $8BFB BCS not taken; $8BFD LDX #$15.
                x = 0x15;
            } else {
                crate::cycle_ledger::charge(6); // $8BE7: taken BEQ.
            }
        } else {
            crate::cycle_ledger::charge(6); // $8BE0: taken BEQ.
        }
        crate::cycle_ledger::charge(32 + 64); // $8BFF store; $8C02..$8C04 SEP and RTS.
        self.set_music_control(x);
    }

    /// ROM: `Dungeon_DetectStaircase`
    pub(super) fn dungeon_detect_staircase(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x01_c329);
        crate::cycle_ledger::charge(94); // Direction mask and no-probe branch.
        let k = self.game_state.player.follower_link.direction() & 12;
        if k == 0 {
            crate::cycle_ledger::charge(6 + 64); // Taken branch and shared SEP/RTS.
            return;
        }

        crate::cycle_ledger::charge(400);
        let lookup = DUNGEON_DETECT_STAIRCASE_BUGGY_LOOKUP[(k >> 1) as usize] as i16 as u16;
        let mut pos = (self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(lookup)
            & 0x01f8)
            << 3;
        pos |= (self.game_state.player.follower_link.x() & 0x01f8) >> 3;
        pos |= self
            .game_state
            .player
            .follower_link
            .lower_level_tilemap_offset();
        crate::cycle_ledger::charge(
            if self.game_state.player.follower_link.lower_level_state() == 0 {
                6
            } else {
                24
            },
        );
        crate::cycle_ledger::charge(138 + if k == 4 { 52 } else { 6 } + 130);

        let at_pos = pos.wrapping_add(if k == 4 { 0x80 } else { 0 }) as usize;
        let head = self.game_state.dungeon.bg2_attributes.bg2_tile(at_pos);
        crate::cycle_ledger::charge(match head.cartridge_attribute() {
            0x26 => 6,
            0x38 => 32 + 6,
            0x39 => 64 + 6,
            0x5e => 96 + 6,
            0x5f => 128,
            _ => 128 + 6,
        });
        let _straight = match head.dungeon_role() {
            DungeonRole::StraightStairHead { descending } => Some(descending),
            DungeonRole::SpiralStairHead | DungeonRole::WallSpiralStairHead { .. } => None,
            _ => {
                crate::cycle_ledger::charge(64);
                return;
            }
        };

        crate::cycle_ledger::charge(148); // Save head, load landing and test attribute.
        let landing = self
            .game_state
            .dungeon
            .bg2_attributes
            .bg2_tile(pos as usize + xy(0, 1));
        if !matches!(landing.dungeon_role(), DungeonRole::StairLanding { .. }) {
            crate::cycle_ledger::charge(6 + 28 + 64); // Taken branch, PLA, shared return.
            return;
        }
        let attr2 = landing.cartridge_attribute();

        crate::cycle_ledger::charge(48); // Carry/lift sign test.
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
        {
            crate::cycle_ledger::charge(144 + 64); // Restore previous Y, shared return.
            self.follower_link_state_mut()
                .restore_y_from_previous_position();
            return;
        }

        crate::cycle_ledger::charge(6); // Taken BPL belongs to this semantic prefix.
        // $01:C386 `TAY` copies the accumulator's hidden high byte, last set
        // by the head lookup's 16-bit address (`ADC #$0080` when facing up).
        self.dungeon_detect_staircase_after_landing(head, u16::from(attr2) | (at_pos as u16 & 0xff00));
    }

    pub(super) fn dungeon_detect_staircase_after_landing(
        &mut self,
        head: NativeTile,
        landing: u16,
    ) {
        let straight = match head.dungeon_role() {
            DungeonRole::StraightStairHead { descending } => Some(descending),
            _ => None,
        };
        let _scope = crate::cycle_ledger::routine(0x01_c329);
        let attr2 = landing as u8;
        crate::cycle_ledger::charge(210); // Taken BPL, stair/room stores, quadrant JSL.
        self.dungeon_stair_movement_mut().set_staircase_index(attr2);
        self.dungeon_stair_movement_mut()
            .set_staircase_index_high((landing >> 8) as u8);
        let dungeon_room = self.game_state.world.location.dungeon_room();
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(dungeon_room);
        self.dungeon_flag_room_data_quadrants();

        crate::cycle_ledger::charge(
            78 + match head.cartridge_attribute() {
                0x38 => 6,
                0x39 => 32,
                _ => 32 + 6,
            },
        ); // Resume after quadrant call and test straight-stair heads.
        if let Some(descending) = straight {
            crate::cycle_ledger::charge(80);
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(0x20);
            if descending {
                crate::cycle_ledger::charge(6 + 62); // Taken south branch and JSL.
                self.dungeon_start_inter_room_trans_down();
            } else {
                crate::cycle_ledger::charge(62);
                self.dungeon_start_inter_room_trans_up();
                crate::cycle_ledger::charge(22); // BRA after north call returns.
            }
        }

        crate::cycle_ledger::charge(246); // Destination/plane lookup and lower-floor test.
        let j = (self.game_state.dungeon.stair_movement.staircase_index() & 3) as usize;
        let room = self.game_state.dungeon.header.travel_destination(j + 1);
        let plane = self.game_state.dungeon.header.staircase_plane(j);
        let lower_level_status = if self
            .game_state
            .player
            .follower_link
            .has_lower_level_state_or_mirror()
        {
            2
        } else {
            0
        };
        crate::cycle_ledger::charge(
            if self.game_state.player.follower_link.lower_level_state() != 0 {
                6
            } else {
                64 + if lower_level_status != 0 { 16 } else { 6 }
            },
        );
        crate::cycle_ledger::charge(292); // Clear action state, restore head, spiral test.
        self.set_dungeon_room_index(room);
        self.dungeon_stair_movement_mut()
            .set_current_staircase_plane(plane);
        self.dungeon_stair_movement_mut()
            .set_staircase_lower_level_status(lower_level_status);
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut().clear_direction_lock_bits(1);

        if head == NativeTile::SPIRAL_STAIR_HEAD {
            crate::cycle_ledger::charge(6 + 104);
            self.set_submodule(6);
            let sound_effect_1 = if self
                .game_state
                .dungeon
                .stair_movement
                .current_staircase_plane()
                < 0x34
            {
                crate::cycle_ledger::charge(6);
                22
            } else {
                crate::cycle_ledger::charge(16);
                24
            };
            crate::cycle_ledger::charge(74); // Sound store and RTS.
            self.set_sound_effect_1(sound_effect_1);
        } else if let Some(descending) = straight {
            crate::cycle_ledger::charge(48 + if descending { 48 } else { 0 } + 6 + 86);
            let submodule = if descending { 19 } else { 18 };
            self.set_submodule(submodule);
            // The source's LinkResetPushTimer is semantically inlined here.
            crate::cycle_ledger::charge_routine(0x07_f3f3, 192);
            self.follower_link_state_mut().set_push_fatigue_timer(7);
            crate::cycle_ledger::charge(42);
        } else {
            crate::cycle_ledger::charge(96 + 62); // Straight tests and wall-stair JSL.
            self.UsedForStraightInterRoomStaircase();
            crate::cycle_ledger::charge(82); // LDX/STX submodule and RTS after the call.
            self.set_submodule(14);
        }
    }

    pub(super) fn UsedForStraightInterRoomStaircase(&mut self) {
        for i in (0..=9).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 13 {
                self.ancilla_slot_view_mut(i).clear();
            }
        }
        self.follower_link_state_mut()
            .clear_animation_step_if_at_least(5);
        self.follower_link_state_mut().clear_movement_subpixels();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_push_fatigue_timer(28);
        self.dungeon_stair_movement_mut()
            .set_staircase_countdown(32);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.ancilla_sfx2_near(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                0x18
            } else {
                0x16
            },
        );

        let x = self.game_state.player.follower_link.x();
        let detect_x = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            x.wrapping_sub(15)
        } else {
            x.wrapping_add(16)
        };
        self.tile_detect_position_mut().set_x(detect_x);
        let link_y = self.game_state.player.follower_link.y();
        self.tile_detect_position_mut().set_y(link_y);
    }

    /// ROM: `Module07_07_FallingTransition`
    pub(in crate::zelda_rtl) fn module07_07_falling_transition(&mut self) {
        // $02:87AC's JSR ($876C,X), table word $877A, selects $8E27, M8/X8.
        // Price the prefix before the attribute loader can suspend it.
        {
            let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x028e27));
            let b = crate::cycle_ledger::blocks(self.lag_free);
            b.block(0x028e27);
            if self.game_state.frame.subsubmodule < 6 { b.taken(0x028e2b); }
            else { b.block(0x028e2d); }
        }
        if self.game_state.frame.subsubmodule >= 6 {
            self.graphics_incremental_vram_upload();
            {
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x028e27));
                crate::cycle_ledger::blocks(self.lag_free).block(0x028e31);
            }
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::FallingTransition,
            ) {
                return;
            }
            self.dungeon_load_attribute_selectable();
        }
        self.complete_module07_07_falling_transition_after_attribute_loader();
    }

    pub(in crate::zelda_rtl) fn complete_module07_07_falling_transition_after_attribute_loader(&mut self) {
        if self.game_state.frame.subsubmodule >= 6 {
            {
                let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x028e27));
                crate::cycle_ledger::blocks(self.lag_free).block(0x028e35);
            }
            self.apply_grayscale_fixed_incremental();
        }
        {
            let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x028e27));
            // $8E39 LDA/JSL; JumpTableLocal and selected bodies are children.
            crate::cycle_ledger::blocks(self.lag_free).block(0x028e39);
        }
        match self.game_state.frame.subsubmodule {
            0 => self.module07_07_00_handle_music_and_reset_room(),
            1 => self.apply_palette_filter_bounce(),
            2 => {
                if !self.begin_dungeon_falling_room_initialization_work() {
                    self.dungeon_initialize_room_from_special();
                }
            }
            3 => self.dungeon_transition_trigger_bgc34_update_and_advance(),
            4 => self.dungeon_transition_trigger_bgc56_update_and_advance(),
            5 => self.dungeon_transition_load_sprite_gfx(),
            6 => self.module07_07_06_sync_bg1and2(),
            7 => self.dungeon_inter_room_trans_state4(),
            8 => self.dungeon_inter_room_trans_not_dark_room(),
            9 => self.dungeon_inter_room_trans_state4(),
            10 => self.dungeon_inter_room_trans_not_dark_room(),
            11 => self.dungeon_inter_room_trans_state4(),
            12 => self.dungeon_inter_room_trans_not_dark_room(),
            13 => self.dungeon_inter_room_trans_state4(),
            14 => self.dungeon_staircase14(),
            15 => self.module07_07_0f_falling_fade_in(),
            16 => self.module07_07_10_land_link_from_falling(),
            17 => self.module07_07_11_cache_room_and_set_music(),
            other => panic!("invalid Module07_07_FallingTransition subsubmodule_index {other}"),
        }
    }

    /// ROM: `Module07_07_00_HandleMusicAndResetRoom`
    pub(super) fn module07_07_00_handle_music_and_reset_room(&mut self) {
        let room = self.game_state.world.location.dungeon_room();
        if room == 0x10 || room == 7 || room == 0x17 {
            self.set_music_control(0xf1);
        }
        self.reset_transition_props_and_advance_reset_interface();
    }

    /// ROM: `Module07_07_06_SyncBG1and2`
    pub(super) fn module07_07_06_sync_bg1and2(&mut self) {
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

    /// ROM: `Module07_07_0F_FallingFadeIn`
    pub(super) fn module07_07_0f_falling_fade_in(&mut self) {
        let mut direction_toggled = false;
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live) {
            if let Some(receipts) = self.original_timing_semantic_receipts.as_mut() {
                receipts.semantic.retain(|receipt| {
                    if *receipt == OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled {
                        assert!(!direction_toggled, "falling palette direction checkpoint replayed");
                        direction_toggled = true;
                        false
                    } else { true }
                });
            }
        }
        if direction_toggled {
            assert_eq!(
                (
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                    self.game_state.frame.subsubmodule
                ),
                (7, 7, 15)
            );
            self.apply_palette_filter_bounce_through_direction_toggle();
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishSpiralStaircasePaletteFilter {
                    tail: SpiralStaircasePaletteTail::FallingFadeInAfterDirectionToggle,
                    caller: InterruptedPaletteFilterCaller::FallingRoom,
                },
                1,
            );
            return;
        }
        self.apply_palette_filter_bounce();
        self.complete_falling_fade_in_after_palette();
    }

    pub(super) fn complete_falling_fade_in_after_palette(&mut self) {
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            != 0
        {
            return;
        }

        let link_y = self.game_state.player.follower_link.y();
        let detect_y = self.game_state.player.tile_detection.y();
        let high = ((link_y >> 8) as u8).wrapping_add(u8::from((link_y as u8) >= detect_y as u8));
        self.tile_detect_position_mut().set_y_high(high);
        self.dungeon_set_boss_music_unorthodox();

        let room = self.game_state.world.location.dungeon_room_index();
        if room == 0x89 || room == 0x4f {
            return;
        }
        if room == 0xa7 {
            self.clear_hud_floor_changed_timer();
            self.dungeon_stair_movement_mut().set_current_floor(1);
            return;
        }
        self.dungeon_stair_movement_mut().decrement_current_floor();
        self.dungeon_play_blip_and_cache_quadrant_visits();
    }

    /// ROM: `Module07_07_10_LandLinkFromFalling`
    pub(super) fn module07_07_10_land_link_from_falling(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.set_submodule(7);
        self.set_subsubmodule(17);
        self.set_chr_halfslot_request(1);
        self.graphics_load_chr_half_slot();
    }

    /// ROM: `Module07_07_11_CacheRoomAndSetMusic`
    pub(super) fn module07_07_11_cache_room_and_set_music(&mut self) {
        if self.overworld_map_state() == 5 {
            self.ResetThenCacheRoomEntryProperties();
            self.dungeon_play_music_if_defeated();
            self.graphics_load_chr_half_slot();
        }
    }

    /// ROM: `Module11_DungeonFallingEntrance`
    pub(in crate::zelda_rtl) fn module11_dungeon_falling_entrance(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => {
                let entrance_music = self.asset_raw(27).expect("missing entrance music asset")
                    [self.game_state.world.region.which_entrance() as usize];
                if entrance_music != 3
                    || self.game_state.inventory.save_progress.progress_indicator() >= 2
                {
                    self.set_music_control(0xf1);
                }
                self.reset_transition_props_and_advance_reset_interface();
            }
            1 => {
                if self.game_state.frame.frame_counter & 1 == 0 {
                    self.apply_palette_filter_bounce();
                }
            }
            2 => self.module11_02_load_entrance(),
            3 => {
                if !self
                    .begin_dungeon_falling_entrance_work(DungeonFallingEntranceWork::SpriteGraphics)
                {
                    self.dungeon_transition_load_sprite_gfx();
                }
            }
            4 => {
                let screen_brightness =
                    self.game_state.display.screen_brightness.wrapping_add(1) & 0x0f;
                self.set_screen_brightness(screen_brightness);
                if self.game_state.display.screen_brightness == 15 {
                    self.increment_subsubmodule();
                }
                self.module11_dungeon_falling_entrance_land();
            }
            5 => self.module11_dungeon_falling_entrance_land(),
            _ => {}
        }
    }

    /// ROM: `Module11_DungeonFallingEntrance_land`
    fn module11_dungeon_falling_entrance_land(&mut self) {
        self.handle_dungeon_landing_from_pit();
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.set_main_module(7);
        self.dungeon_room_runtime_mut().skip_room_tags_once();
        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.ResetThenCacheRoomEntryProperties();
        let queued_music_control = self.game_state.system_signals.queued_music_control();
        self.set_music_control(queued_music_control);
        self.save_current_music_as_last();
    }

    /// ROM: `Module11_02_LoadEntrance`
    pub(super) fn module11_02_load_entrance(&mut self) {
        self.EnableForceBlank();
        self.set_color_window_selection(2);
        if self.begin_dungeon_falling_entrance_work(DungeonFallingEntranceWork::RoomAndTilesets) {
            // C's Module11_02_LoadEntrance executes its quick entrance prefix
            // (Dungeon_LoadEntrance through the Link y adjust) before the first
            // heavy room-draw loop, so the room index, Link position, and
            // indoors flag land in WRAM at the FIRST interrupted slice — the
            // oracle flips them at route frame 8451 while the frame counter is
            // already held. Only the heavy tail belongs to the completion
            // slice. Bug class 5 (collapsed timed side-effect phase), the same
            // split as the f2235 selected-game-load room preload.
            self.module11_02_load_entrance_prefix();
            self.begin_module11_02_room_header_if_source_returned();
            return;
        }
        self.module11_02_load_entrance_prefix();
        self.complete_module11_02_load_entrance();
    }

    fn module11_02_load_entrance_prefix(&mut self) {
        self.dungeon_load_entrance();

        let dung = self.game_state.inventory.save_progress.palace_index_x2();
        let keys = if dung != 0xff {
            let palace_index_x2 = if dung == 2 { 0 } else { dung };
            self.game_state
                .inventory
                .dungeon_key_slots
                .keys_earned(palace_index_x2)
        } else {
            0xff
        };
        self.player_resources_mut().set_keys(keys);
        self.hud_rebuild();
        self.follower_link_state_mut().set_pit_data_index(4);
        self.follower_link_state_mut().set_near_pit_state(3);
        {
            self.follower_link_state_mut().set_visibility_status(12);
        }
        self.follower_link_state_mut().set_speed_modifier(16);

        let y = (self.game_state.player.follower_link.y() as u8)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_picking_throw_state();
        }
        self.follower_link_state_mut().set_y_button_action_timer(0);
        let dungeon_room = self.game_state.world.location.dungeon_room();
        self.dungeon_room_tracking_mut()
            .set_previous_room_index_word(dungeon_room);
        let link_y = self.game_state.player.follower_link.y();
        self.tile_detect_position_mut().set_y(link_y);
        let new_y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(u16::from(y).wrapping_add(16));
        self.follower_link_state_mut().set_y(new_y);
    }

    pub(in crate::zelda_rtl) fn complete_module11_02_load_entrance(&mut self) {
        if self.active_early_dungeon_floor_draw.is_none() {
            self.dungeon_torch_mut().clear_lit_torches();
            self.dungeon_torch_mut().clear_dungeon_dark_with_lantern();
        }
        if !self.finish_early_dungeon_room_load(EarlyDungeonRoomOwner::FallingEntrance) {
            self.dungeon_load_and_draw_room();
        }
        self.dungeon_load_custom_tile_attr();
        let animated = DUNG_ANIMATED_TILES
            [self.game_state.world.palette_theme.main_tile_theme_index() as usize];
        self.decompress_animated_dungeon_tiles(animated as usize);
        self.dungeon_load_attribute_table();
        // This is Module11 stage 2's literal `bak + 1` publication. In the
        // live-timing path the room-parser clear and this restore are exposed
        // separately by semantic receipts while the long call is suspended;
        // completing the translated body remains idempotent at the source's
        // terminal stage instead of reconstructing `bak` from transient RAM.
        self.set_subsubmodule(3);
        self.world_palette_theme_mut()
            .set_misc_sprites_graphics_index(10);
        self.initialize_tilesets();
        // PALETTE_SP6R_INDOORS (0xab1) is read by palette_load_sprite_environment_dungeon
        // off display.palette_buffer, not world.palette_theme. Set the display owner so
        // the immediately-following dungeon_load_palettes() loads the selector-10 sprite
        // environment palette instead of a stale selector (matches raw-RAM single-byte).
        self.set_sp6r_indoors(10);
        self.dungeon_load_palettes();
        self.hud_restore_torch_background();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().clear_button_b_frames();
        self.dungeon_reset_torch_background_and_player();
        if self.game_state.player.follower_link.is_bunny_mirror() {
            self.load_gear_palettes_bunny();
        }
        self.set_hdma_enable_mask(0x80);
        self.hud_refill_logic();
        self.module_pre_dungeon_set_ambient_sfx();
        self.set_submodule(7);
        let _ = self.dungeon_load_song_bank_if_needed();
    }

    fn set_spiral_stair_wall_priority(&mut self, pos: u16, high: bool) {
        let mask = if high { 0x2000 } else { 0xdfff };
        for i in 0..5usize {
            for y in 0..4usize {
                let index = (pos as usize + i + y * 64) as u16;
                // C reads/writes `dung_bg2[index]` flat; for `index >= 0x1000`
                // (a lower-level staircase wall) that spills into the contiguous
                // BG1 span. `bg2_tile`/`set_bg2_tile` would read 0 and drop the
                // OOB write, so the priority clear became a no-op and the wall
                // stayed high-priority. Route through the spill-aware path.
                let value = self
                    .game_state
                    .dungeon
                    .room_tilemaps
                    .room_tilemap_word(crate::game_state::constants::DUNG_BG2, index);
                let value = if high { value | mask } else { value & mask };
                self.dungeon_room_tilemaps_mut().set_room_tilemap_word(
                    crate::game_state::constants::DUNG_BG2,
                    index,
                    value,
                );
            }
        }
    }

    /// ROM: `SpiralStairs_MakeNearbyWallsHighPriority_Entering`
    pub(super) fn spiral_stairs_make_nearby_walls_high_priority_entering(&mut self) {
        let index = (self.game_state.dungeon.stair_movement.staircase_index() & 3) as usize;
        let pos = self
            .game_state
            .dungeon
            .stair_lists
            .inter_staircase_pos(index)
            .wrapping_sub(4);
        self.dungeon_stair_movement_mut()
            .set_staircase_tilemap_pos_x2(pos.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(pos, true);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.request_nmi_copy_packets();
    }

    /// ROM: `SpiralStairs_MakeNearbyWallsLowPriority`
    pub(super) fn spiral_stairs_make_nearby_walls_low_priority(&mut self) {
        let pos = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_tilemap_pos_x2()
            >> 1;
        self.set_spiral_stair_wall_priority(pos, false);
        let dma_ptr = self.dungeon_prep_overlay_dma_next_prep(0, pos.wrapping_mul(2));
        self.dungeon_prep_overlay_dma_next_prep(dma_ptr, pos.wrapping_mul(2).wrapping_add(8));
        self.request_nmi_copy_packets();
    }

    /// ROM: `SpiralStairs_MakeNearbyWallsHighPriority_Exiting`
    pub(super) fn spiral_stairs_make_nearby_walls_high_priority_exiting(&mut self) {
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            return;
        }
        let lf = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_tilemap_pos_x2()
            .wrapping_add(8)
            & 0x007f;
        let mut x = 0usize;
        let mut p;
        loop {
            p = self.game_state.dungeon.stair_lists.inter_staircase_pos(x);
            if (p.wrapping_mul(2) & 0x007f) == lf {
                break;
            }
            x += 1;
        }
        p = p.wrapping_sub(4);
        self.dungeon_stair_movement_mut()
            .set_staircase_tilemap_pos_x2(p.wrapping_mul(2));
        self.set_spiral_stair_wall_priority(p, true);
    }

    /// ROM: `Module07_0F_00_InitSpotlight`
    pub(super) fn module07_0f_00_init_spotlight(&mut self) {
        // Cycle ledger: Module07_0F_00_InitSpotlight ($02:932D).
        let _scope = crate::cycle_ledger::routine(0x02_932d);
        // $932D: JSL Spotlight_open (the callee charges itself).
        crate::cycle_ledger::charge(62);
        self.spotlight_open();
        // $9331-$9333: INC $B0, RTS.
        crate::cycle_ledger::charge(80);
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_0F_01_OperateSpotlight`
    pub(super) fn module07_0f_01_operate_spotlight(&mut self) {
        // Cycle ledger: Module07_0F_01_OperateSpotlight ($02:9334). The
        // suffix is charged by `complete_module07_0f_operate_spotlight_suffix`,
        // which a deferred goal transition also runs on a later host.
        let _scope = crate::cycle_ledger::routine(0x02_9334);
        // $9334: JSL Sprite_Main (the callee charges itself when annotated).
        crate::cycle_ledger::charge(62);
        self.sprite_main();
        // $9338: JSL IrisSpotlight_ConfigureTable (the callee charges itself).
        crate::cycle_ledger::charge(62);
        if let Some(progress) = self.native_exact_cpu_landing_spotlight_checkpoint() {
            self.suspend_landing_spotlight_table(progress);
            return;
        }
        self.iris_spotlight_configure_table();
        self.complete_module07_0f_operate_spotlight_suffix();
    }

    pub(in crate::zelda_rtl) fn suspend_landing_spotlight_table(
        &mut self,
        progress: crate::SpotlightTableBuildProgress,
    ) {
        assert!(self.native_exact_cpu_landing_spotlight_build.is_none());
        assert!(!self.main_loop_sprite_preparation_completed);
        self.native_exact_cpu_landing_spotlight_build =
            Some(self.begin_iris_spotlight_configure_table_at_progress(progress));
        // The table builder still owns the Module 7 stack. Its eventual
        // return also owes ZeldaRunGameLoop's sprite-DMA preparation and
        // latch clear, even when the host stops without accepting an NMI.
        self.pending_main_loop_common_suffix.get_or_insert(
            MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,
        );
    }

    /// The Module07_0F_01 suffix run after a deferred landing goal
    /// transition: the ROM is returning from IrisSpotlight_ConfigureTable
    /// into Module07_0F_01_OperateSpotlight, so it runs under that scope.
    pub(in crate::zelda_rtl) fn complete_deferred_module07_0f_operate_spotlight_suffix(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_9334);
        self.complete_module07_0f_operate_spotlight_suffix();
    }

    pub(super) fn complete_module07_0f_operate_spotlight_suffix(&mut self) {
        // Cycle ledger: while the landing goal transition is deferred the ROM
        // has not returned from IrisSpotlight_ConfigureTable yet, so this
        // inline pass (a no-op with submodule still $0F) charges nothing; the
        // deferred wrapper charges the suffix once when it really runs.
        let price = !self.dungeon_landing_goal_transition_pending;
        let charge = |master: u64| {
            if price {
                crate::cycle_ledger::charge(master);
            }
        };
        // $933C-$933E: LDA $11, BNE.
        charge(40);
        if self.game_state.frame.submodule == 0 {
            // $9340-$9351: STZ $96/$97/$98/$1E/$1F/$B0, LDA $0132, CMP #$FF,
            // BEQ.
            charge(208);
            self.clear_window_layer_masks();
            self.clear_window_main_sub_masks();
            self.set_subsubmodule(0);
            let queued_music_control = self.game_state.system_signals.queued_music_control();
            if queued_music_control != 0xff {
                // $9353: STA $012C.
                charge(32);
                self.set_music_control(queued_music_control);
            } else {
                // BEQ $9356 taken.
                charge(6);
            }
        } else {
            // BNE $9356 taken.
            charge(6);
        }
        // $9356: RTS.
        charge(42);
    }

    /// ROM: `Module07_0F_LandingWipe`
    pub(in crate::zelda_rtl) fn module07_0f_landing_wipe(&mut self) {
        // Cycle ledger: Module07_0F_LandingWipe ($02:931D).
        let _scope = crate::cycle_ledger::routine(0x02_931d);
        let cpu_advance = self.take_dungeon_landing_cpu_advance();
        let authoritative_link_oam_suspension =
            self.take_original_timing_main_loop_interruption(crate::MainLoopInterruption::LinkOam);
        if self.game_state.frame.subsubmodule == 0 {
            let entry_started_after_leading_nmi = self
                .game_execution_scheduler
                .current_main_iteration_follows_leading_nmi();
            self.begin_dungeon_landing_spotlight_publication(entry_started_after_leading_nmi);
        }
        // $931D-$9321: LDA $B0, ASL, TAX, JSR (kDungeon_Submodule_F,X); the
        // submodule charges itself.
        crate::cycle_ledger::charge(114);
        match self.game_state.frame.subsubmodule {
            0 => self.module07_0f_00_init_spotlight(),
            1 => self.module07_0f_01_operate_spotlight(),
            other => panic!("invalid Module07_0F_LandingWipe subsubmodule_index {other}"),
        }
        if self.native_exact_cpu_landing_spotlight_build.is_some() {
            // The retained CPU stopped inside the table builder. Keep the
            // native C caller suspended across that host boundary instead of
            // eagerly running its return and Link/OAM.
            self.latch_nmi_update();
            self.game_execution_scheduler
                .schedule_cpu_timed_work_from_current_main_iteration(
                    GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                    1,
                );
            return;
        }
        if !self.dungeon_landing_goal_transition_pending {
            // A deferred IrisSpotlight return still owns the submodule
            // publication. Animation and Link OAM must run after that
            // publication, under the resumed landing caller.
            self.complete_module07_0f_landing_wipe_suffix();
        }
        if authoritative_link_oam_suspension {
            // The temporary Live authority observed the host return while the
            // source was still inside this landing submodule's LinkOam_Main
            // call. Resume the existing semantic caller continuation on the
            // following host; do not enter Module 7's shared Sprite_Main
            // suffix early. CPU addresses remain private to the replaceable
            // oracle adapter.
            self.latch_nmi_update();
            self.game_execution_scheduler
                .schedule_cpu_timed_work_returning_on_later_host(
                    GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                    1,
                );
            return;
        }
        if let Some(advance) = cpu_advance {
            debug_assert_eq!(advance.subsubmodule, self.game_state.frame.subsubmodule);
            debug_assert_eq!(
                advance.palette_countdown,
                self.game_state.display.palette_filter.countdown(),
            );
            match advance.phase {
                ModuleCpuPhase::InterruptedInSubmodule => {
                    assert_ne!(
                        advance.submodule_nmi_slices, 0,
                        "interrupted landing-wipe submodule must cross an NMI",
                    );
                    // The ROM is still inside the indirect submodule call, so
                    // Main_PrepSpritesForNmi has not reached its final
                    // nmi_boolean = 0 store. Preserve that caller-owned latch:
                    // the interrupt publishes PPU registers but skips the
                    // NMI_DoUpdates DMA body.
                    self.latch_nmi_update();
                    // Spotlight_open/IrisSpotlight_ConfigureTable are
                    // synchronous C calls. An ordinary main iteration can
                    // spend its still-pending trailing NMI on the first
                    // crossing; an iteration which began after a leading NMI
                    // has no second boundary in this host callback. Derive
                    // that ownership from the scheduler's CPU phase instead
                    // of inferring a cadence from adjacent spotlight frames.
                    self.game_execution_scheduler
                        .schedule_cpu_timed_work_from_current_main_iteration(
                            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
                            advance.submodule_nmi_slices,
                        );
                }
                ModuleCpuPhase::CompleteBeforeNmi => {
                    debug_assert_eq!(advance.submodule_nmi_slices, 0);
                }
                phase => {
                    panic!("landing-wipe CPU continuation reached unsupported phase {phase:?}")
                }
            }
        }
    }

    pub(in crate::zelda_rtl) fn complete_deferred_module07_0f_landing_wipe_suffix(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x02_931d);
        self.complete_module07_0f_landing_wipe_suffix();
    }

    fn complete_module07_0f_landing_wipe_suffix(&mut self) {
        // The indirect spotlight call returns through this landing caller
        // before Module 7's shared tail. Retain both animation and Link OAM
        // when that call is suspended, including the animation wrap.
        // $9324/$9328: JSL animation, JSL LinkOam_Main; $932C: RTS.
        crate::cycle_ledger::charge(62);
        self.link_handle_moving_animation_full_long_entry();
        crate::cycle_ledger::charge(62);
        self.link_oam_main();
        crate::cycle_ledger::charge(42);
    }

    /// ROM: `Module07_10_SouthIntraRoomStairs`
    pub(in crate::zelda_rtl) fn module07_10_south_intra_room_stairs(&mut self) {
        let t = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter();
        if t != 0 {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            if t == 20 {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.module07_10_00_init_stairs(),
            1 => self.module07_10_01_climb_stairs(),
            other => panic!("invalid Module07_10_SouthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    /// ROM: `Module07_09_OpenCrackedDoor`
    pub(in crate::zelda_rtl) fn module07_09_open_cracked_door(&mut self) {
        self.OpenCrackedDoor();
    }

    /// ROM: `Module07_10_00_InitStairs`
    pub(super) fn module07_10_00_init_stairs(&mut self) {
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.game_state.player.follower_link.direction() & 4 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(v1);
        self.set_sound_effect_1(sfx);
        self.follower_link_state_mut().set_speed_modifier(1);
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_10_01_ClimbStairs`
    pub(super) fn module07_10_01_climb_stairs(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            return;
        }
        if self.game_state.player.follower_link.direction() & 8 != 0 {
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
        }
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `Module07_08_NorthIntraRoomStairs`
    pub(in crate::zelda_rtl) fn module07_08_north_intra_room_stairs(&mut self) {
        let t = self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter();
        if t != 0 {
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            if t == 20 {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.link_handle_velocity();
            self.apply_links_movement_to_camera();
            self.dungeon_handle_camera();
            self.link_handle_moving_animation_full_long_entry();
        }
        match self.game_state.frame.subsubmodule {
            0 => self.module07_08_00_init_stairs(),
            1 => self.module07_08_01_climb_stairs(),
            other => panic!("invalid Module07_08_NorthIntraRoomStairs subsubmodule_index {other}"),
        }
    }

    /// ROM: `Module07_08_00_InitStairs`
    pub(super) fn module07_08_00_init_stairs(&mut self) {
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        let mut v1 = 0x3c;
        let mut sfx = 25;
        if self.game_state.player.follower_link.direction() & 8 != 0 {
            v1 = 0x38;
            sfx = 23;
            self.follower_link_state_mut()
                .set_lower_level_mirror_state(0);
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().clear_lower_level();
            }
        }
        self.dungeon_stair_movement_mut()
            .set_staircase_move_counter(v1);
        self.set_sound_effect_1(sfx);
        self.follower_link_state_mut().set_speed_modifier(1);
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_08_01_ClimbStairs`
    pub(super) fn module07_08_01_climb_stairs(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            return;
        }
        if self.game_state.player.follower_link.direction() & 4 != 0 {
            self.follower_link_state_mut()
                .set_lower_level_mirror_state(1);
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().mark_lower_level();
            }
        }
        self.set_subsubmodule(0);
        self.clear_screen_transition();
        self.set_submodule(0);
        self.SetAndSaveVisitedQuadrantFlags();
    }

    /// ROM: `Module07_11_StraightInterroomStairs`
    pub(in crate::zelda_rtl) fn module07_11_straight_interroom_stairs(&mut self) {
        if self.game_state.frame.subsubmodule >= 3 {
            if self.suspend_dungeon_selectable_peg_attribute_flip(
                DungeonPegAttributeFlipCaller::StraightInterroomStairs,
            ) {
                return;
            }
            self.dungeon_load_attribute_selectable();
        }
        self.complete_module07_11_straight_interroom_stairs_after_attribute_loader();
    }

    pub(in crate::zelda_rtl) fn complete_module07_11_straight_interroom_stairs_after_attribute_loader(&mut self) {
        let fadeout_cpu_advance = (self.rom_startup_timing()
            && !matches!(self.original_timing_owner, crate::zelda_rtl::OriginalTimingOwnerState::Live)
            && self.game_state.frame.subsubmodule == 1)
            .then(|| self.take_dungeon_landing_cpu_advance()).flatten();
        if self.game_state.frame.subsubmodule >= 13 {
            self.graphics_incremental_vram_upload();
        }
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            != 0
        {
            if self
                .game_state
                .dungeon
                .stair_movement
                .staircase_move_counter()
                == 16
            {
                self.follower_link_state_mut().set_speed_modifier(2);
            }
            self.dungeon_stair_movement_mut()
                .decrement_staircase_move_counter();
            let direction = if self.game_state.frame.submodule == 18 {
                8
            } else {
                4
            };
            self.follower_link_state_mut().set_direction(direction);
            self.link_handle_velocity();
        }
        self.link_handle_moving_animation_full_long_entry();
        match self.game_state.frame.subsubmodule {
            0 => self.module07_11_00_prep_and_reset(),
            1 => self.module07_11_01_fade_out(),
            2 => self.module07_11_02_load_and_prep_room(),
            3 => self.module07_11_03_filter_and_load_bg_chars(),
            4 => self.module07_11_04_filter_do_bg_and_reset_sprites(),
            5 => self.dungeon_spiral_staircase11(),
            6 => self.dungeon_spiral_staircase12(),
            7 => self.dungeon_spiral_staircase11(),
            8 => self.dungeon_spiral_staircase12(),
            9 => self.module07_11_09_load_sprite_graphics(),
            10 => self.module07_11_0a_scroll_camera(),
            11 => self.module07_11_0b_prep_destination(),
            12 => self.dungeon_inter_room_trans_state4(),
            13 => self.dungeon_inter_room_trans_not_dark_room(),
            14 => self.dungeon_inter_room_trans_state4(),
            15 => self.dungeon_double_apply_and_increment_grayscale(),
            16 => self.module07_11_19_set_song_and_filter(),
            17 => self.module07_11_11_keep_sliding(),
            18 => self.ResetThenCacheRoomEntryProperties(),
            other => {
                panic!("invalid Module07_11_StraightInterroomStairs subsubmodule_index {other}")
            }
        }
        if let Some(advance) = fadeout_cpu_advance {
            self.apply_dungeon_quadrant_cpu_advance(advance);
        }
    }

    /// ROM: `Module07_11_00_PrepAndReset`
    pub(super) fn module07_11_00_prep_and_reset(&mut self) {
        if self.game_state.player.follower_link.is_running() {
            self.follower_link_state_mut().clear_running();
            self.follower_link_state_mut().set_speed_setting(2);
        }
        let sound_effect_1 = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            24
        } else {
            22
        };
        self.set_sound_effect_1(sound_effect_1);
        let room = self.game_state.world.location.dungeon_room();
        if room == 48 || room == 64 {
            self.set_music_control(0xf1);
        }
        self.reset_transition_props_and_advance_reset_interface();
    }

    /// ROM: `Module07_11_01_FadeOut`
    pub(super) fn module07_11_01_fade_out(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            < 9
        {
            // Capture the complete input palette before the atomic translation
            // advances it. On hardware the filter can span vblank: NMI uploads
            // this generation, then the 65816 resumes and finishes the next.
            self.retain_palette_filter_input_cgram_on_next_display_capture();
            self.apply_palette_filter_bounce();
            self.stage_straight_interroom_fadeout_obj_source();
            let palette_countdown = self.game_state.display.palette_filter.countdown();
            self.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(palette_countdown);
            if palette_countdown == 23 {
                self.increment_subsubmodule();
            }
        }
    }

    /// ROM: `Module07_11_02_LoadAndPrepRoom`
    pub(super) fn module07_11_02_load_and_prep_room(&mut self) {
        self.apply_palette_filter_bounce();
        let live_draw = self.native_exact_cpu_owner.is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL").is_some()
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_OBJECT_DRAW_LIVE_TRIAL").is_some();
        if live_draw {
            self.begin_live_dungeon_supertile_room_load();
        } else {
            self.dungeon_load_room();
            self.complete_straight_room_initialization_after_room_load();
        }
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
        ) {
            return;
        }
        if live_draw {
            self.finish_live_dungeon_supertile_room_load();
            self.complete_straight_room_initialization_after_room_load();
        }
        self.increment_subsubmodule();
    }

    pub(in crate::zelda_rtl) fn complete_straight_room_initialization_after_room_load(&mut self) {
        if self.native_transition_aux_graphics_progress.is_none() {
            self.dungeon_restore_star_tile_chr();
        }
        self.LoadTransAuxGFX();
        self.dungeon_load_custom_tile_attr();
        self.dungeon_adjust_for_room_layout();
        self.follower_initialize();
    }

    /// ROM: `Module07_11_03_FilterAndLoadBGChars`
    pub(super) fn module07_11_03_filter_and_load_bg_chars(&mut self) {
        self.apply_palette_filter_bounce();
        self.PrepTransAuxGfx();
        self.set_pending_nmi_subroutine(9);
        self.set_core_update_disable_flag(9);
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
        ) {
            return;
        }
        self.increment_subsubmodule();
    }

    /// ROM: `Module07_11_04_FilterDoBGAndResetSprites`
    pub(super) fn module07_11_04_filter_do_bg_and_reset_sprites(&mut self) {
        self.apply_palette_filter_bounce();
        self.dungeon_transition_trigger_bgc56_update_and_advance();
        if self.suspend_straight_interroom_sprite_reset_before_room_load() {
            return;
        }
        self.complete_straight_interroom_sprite_reset();
    }

    pub(in crate::zelda_rtl) fn complete_straight_interroom_sprite_reset(&mut self) {
        let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
        self.dungeon_room_tracking_mut()
            .set_room_index2(dungeon_room_index);
        self.dungeon_reset_sprites();
    }

    /// ROM: `Module07_11_09_LoadSpriteGraphics`
    pub(super) fn module07_11_09_load_sprite_graphics(&mut self) {
        self.apply_palette_filter_bounce();
        self.decrement_subsubmodule();
        self.LoadNewSpriteGFXSet();
        if self.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
        ) {
            return;
        }
        self.dungeon_handle_translucency_and_palette();
    }

    /// ROM: `Module07_11_0A_ScrollCamera`
    pub(super) fn module07_11_0a_scroll_camera(&mut self) {
        self.follower_link_state_mut().set_visibility_status(12);
        self.follower_state_mut().set_appearance_none_flag(12);
        let mut i = self.screen_transition() as usize;
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(DUNGEON_TRANSITION_SCROLL_DELTAS[i] as i16 as u16)
            & !3;
        self.set_bg1_y(y);
        self.set_bg2_y(y);
        if (y & 0x01fc)
            == self
                .game_state
                .world
                .camera_boundaries
                .up_down_scroll_target(i)
        {
            if self.game_state.frame.submodule >= 18 {
                i += 2;
            }
            let link_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(STAIRCASE_CAMERA_LINK_Y_ADJUSTMENTS[i] as i16 as u16);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().set_visibility_status(0);
            self.follower_state_mut().set_appearance_none_flag(0);
            self.increment_subsubmodule();
        }
    }

    /// ROM: `Module07_11_0B_PrepDestination`
    pub(super) fn module07_11_0b_prep_destination(&mut self) {
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
                .set_staircase_move_counter(0x32);
            self.set_sound_effect_1(25);
        } else {
            self.dungeon_stair_movement_mut().increment_current_floor();
            self.dungeon_stair_movement_mut()
                .set_staircase_move_counter(0x3c);
            self.set_sound_effect_1(23);
        }

        let mut r0 = 0u8;
        let y_delta = if self.game_state.frame.submodule == 18 {
            (-32i16) as u16
        } else {
            32
        };
        if self.game_state.player.follower_link.is_on_lower_level() {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y_delta);
            self.follower_link_state_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }
        let plane = self
            .game_state
            .dungeon
            .stair_movement
            .current_staircase_plane() as usize;
        self.follower_link_state_mut().set_lower_level_states(
            TELEPORT_PIT_SECONDARY_LEVELS[plane],
            TELEPORT_PIT_PRIMARY_LEVELS[plane],
        );
        if self.game_state.player.follower_link.is_on_lower_level() {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y_delta);
            self.follower_link_state_mut().set_y(y);
            r0 = r0.wrapping_add(1);
        }

        if r0 == 0 {
            let delta = if self.game_state.frame.submodule == 18 {
                if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                    (-24i16) as u16
                } else {
                    (-8i16) as u16
                }
            } else {
                12
            };
            let y = self.game_state.player.follower_link.y().wrapping_add(delta);
            self.follower_link_state_mut().set_y(y);
        }

        self.dungeon_play_blip_and_cache_quadrant_visits();
        self.hud_restore_torch_background();
        self.dungeon_inter_room_trans_not_dark_room();
    }

    /// ROM: `Module07_11_19_SetSongAndFilter`
    pub(super) fn module07_11_19_set_song_and_filter(&mut self) {
        if self.overworld_map_state() == 5
            && self
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen()
                == 0
        {
            self.increment_subsubmodule();
            let room = self.game_state.world.location.dungeon_room();
            if room == 48 {
                self.set_music_control(0x1c);
            } else if room == 64 {
                self.set_music_control(0x10);
            }
        }
        self.apply_grayscale_fixed_incremental();
    }

    /// ROM: `Module07_11_11_KeepSliding`
    pub(super) fn module07_11_11_keep_sliding(&mut self) {
        if self
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter()
            == 0
        {
            self.increment_subsubmodule();
        } else {
            self.apply_grayscale_fixed_incremental();
        }
    }

    /// ROM: `Module07_14_RecoverFromFall`
    pub(in crate::zelda_rtl) fn module07_14_recover_from_fall(&mut self) {
        match self.game_state.frame.subsubmodule {
            0 => self.module07_14_00_scroll_camera(),
            1 => self.RecoverPositionAfterDrowning(),
            _ => {}
        }
    }

    /// ROM: `Module07_14_00_ScrollCamera`
    pub(super) fn module07_14_00_scroll_camera(&mut self) {
        for _ in 0..2 {
            self.step_bg2_h_copy2_toward_cached();
            self.step_bg2_v_copy2_toward_cached();
        }
        if self.game_state.display.ppu_scroll_copy.bg2_h_copy2()
            == self.game_state.display.ppu_scroll_copy.bg2_h_copy2_cached()
            && self.game_state.display.ppu_scroll_copy.bg2_v_copy2()
                == self.game_state.display.ppu_scroll_copy.bg2_v_copy2_cached()
        {
            self.increment_subsubmodule();
        }
        if !self.game_state.dungeon.torch.dungeon_dark_with_lantern() {
            self.MirrorBg1Bg2Offs();
        }
    }

    pub(super) fn HandleLinkOnSpiralStairs(&mut self) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x07_f2c1));
        let lag_free = self.lag_free;
        let charge = |master| { if lag_free { crate::cycle_ledger::charge(master); } };
        // $07:F2C1..F2D2: cache XY and test the stair action step.
        charge(236);
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self.game_state.player.follower_link.y_button_action_step() != 0 {
            charge(44); // $07:F2D4 RTL.
            return;
        }
        // $07:F2D2 taken BEQ, $F2D5..F2E4 clear/DBR and direction test,
        // $F2E6..ED or $F2FC..F303 velocity/decrement. Direction-4 also BRA.
        let reverse = self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0;
        charge(6 + 216 + 102 + if reverse { 22 } else { 6 });
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_auxiliary_state();

        self.follower_link_state_mut()
            .set_actual_y_velocity((-2i8) as u8);
        let push_fatigue_timer = self
            .follower_link_state_mut()
            .decrement_push_fatigue_timer();
        if (push_fatigue_timer as i8).is_negative() {
            charge(112); // $07:F2EF..F2F8 or $F305..F30E underflow path.
            self.follower_link_state_mut().set_push_fatigue_timer(0);
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                self.follower_link_state_mut()
                    .set_actual_velocity_xy((-2i8) as u8, 0);
            } else {
                self.follower_link_state_mut()
                    .set_actual_velocity_xy(2, (-2i8) as u8);
            }
        } else {
            charge(6); // $07:F2ED/F303 taken BPL.
        }
        charge(172); // $07:F310..F31B: two JSLs and fatigue test.
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.game_state.player.follower_link.push_fatigue_timer() == 0 {
            charge(62); // $07:F31D..F320 countdown decrement/test.
            if self.dungeon_stair_movement_mut().decrement_staircase_countdown_underflowed() {
                charge(112 + 24 + if reverse { 6 } else { 16 }); // $F322..F330.
                let facing = if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 { 4 } else { 6 };
                self.follower_link_state_mut().set_facing(facing);
            } else {
                charge(6); // $07:F320 taken BPL.
            }
        } else {
            charge(6); // $07:F31B taken BNE.
        }
        charge(78 + 16); // $07:F332..F33C X distance and equality test.
        let mut xd = (self.game_state.player.follower_link.x() as u8)
            .wrapping_sub(self.game_state.player.tile_detection.x_low()) as i8;
        if xd < 0 {
            charge(30); // $07:F339..F33B absolute value.
            xd = xd.wrapping_neg();
        } else {
            charge(6); // $07:F337 taken BPL.
        }
        if xd != 0 {
            charge(6 + 94); // $07:F33C taken BNE, $F38D..F390 exit.
            return;
        }
        charge(162); // $07:F33E..F34A reposition JSL and follower test.
        self.RepositionLinkAfterSpiralStairs();
        if self.game_state.sprites.follower_runtime.indicator() != 0 {
            charge(62); // $07:F34C JSL follower (body belongs to child).
            self.follower_initialize();
        } else {
            charge(6); // $07:F34A taken BEQ.
        }
        // $07:F350..F38A: direction-dependent detect X and action/SFX setup.
        charge(144 + if reverse { 6 } else { 64 });
        charge(314 + if reverse { 6 + 62 } else { 84 });
        charge(94); // $07:F38D..F390 SEP/PLB/RTL.
        let detect_x = self.game_state.player.follower_link.x().wrapping_add(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                (-8i16) as u16
            } else {
                12
            },
        );
        self.tile_detect_position_mut().set_x(detect_x);
        self.follower_link_state_mut().set_y_button_action_step(1);
        self.dungeon_stair_movement_mut().set_staircase_countdown(6);
        self.ancilla_sfx2_near(
            if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
                25
            } else {
                23
            },
        );
    }

    /// ROM: `SpiralStairs_FindLandingSpot`
    pub(super) fn spiral_stairs_find_landing_spot(&mut self) {
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self
            .dungeon_stair_movement_mut()
            .decrement_staircase_countdown_underflowed()
        {
            self.follower_link_state_mut().set_facing(2);
        }

        self.follower_link_state_mut().set_actual_velocity_xy(4, 0);
        if self.game_state.dungeon.stair_movement.staircase_index() & 4 != 0 {
            self.follower_link_state_mut()
                .set_actual_velocity_xy((-4i8) as u8, 2);
        }
        if self.game_state.player.follower_link.y_button_action_step() == 2 {
            self.follower_link_state_mut().set_actual_velocity_xy(0, 16);
        }
        self.link_move_position();
        self.link_handle_moving_animation_start_with_dash();
        if self.game_state.player.follower_link.x() as u8
            == self.game_state.player.tile_detection.x_low()
        {
            self.follower_link_state_mut().set_y_button_action_step(2);
        }
    }
}
