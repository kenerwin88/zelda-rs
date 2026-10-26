//! World objects, switches, statues, barriers, and dash-triggered items.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;
use crate::types::sign8;
use crate::types::SpriteHitBox;

impl ZeldaState {
    // -----------------------------------------------------------------------
    // void SpriteDraw_LightFountain(int k) {  // 858a94 — sprite_main.c:2169
    //   static const DrawMultipleData kMasterSword_LightBall_Dmd[12] = {
    //     {-6, 4, 0x0082, 2}, {-6, 4, 0x4082, 2}, {-6, 4, 0xc082, 2},
    //     {-6, 4, 0x8082, 2}, {-6, 4, 0x00a0, 2}, {-6, 4, 0x40a0, 2},
    //     {-6, 4, 0xc0a0, 2}, {-6, 4, 0x80a0, 2}, {-6, 4, 0x0080, 2},
    //     {-6, 4, 0x4080, 2}, {-6, 4, 0xc080, 2}, {-6, 4, 0x8080, 2},
    //   };
    //   Oam_AllocateFromRegionC(4);
    //   Sprite_DrawMultiple(k, &kMasterSword_LightBall_Dmd[sprite_graphics[k] * 4 + sprite_D[k]], 1, NULL);
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_light_fountain(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        let idx = (self.sprite_slot_view(k).graphics() as usize) * 4
            + (self.sprite_slot_view(k).direction() as usize);
        self.sprite_draw_multiple(k, &MASTER_SWORD_LIGHT_BALL_DRAW_FRAMES[idx..idx + 1]);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_ThrownItem_Gigantic(int k) {  // 86ab76 — sprite_main.c:9629
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_thrown_item_gigantic(&mut self, k: usize) {
        let c_idx = usize::from(self.sprite_slot_view(k).c().wrapping_sub(6));
        let value = THROWABLE_SCENERY_LARGE_OAM_FLAGS[c_idx];
        self.sprite_slot_view_mut(k).set_oam_flags(value);

        let info = match self.sprite_prep_oam_coord_or_double_ret(k) {
            Some(p) => PrepOamCoordsRet::from_tuple(p),
            None => return,
        };

        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut i: i32 = 3;
        loop {
            let x = info
                .x
                .wrapping_add(THROWABLE_SCENERY_LARGE_X_OFFSETS[i as usize] as u16);
            let y = info
                .y
                .wrapping_add(THROWABLE_SCENERY_LARGE_Y_OFFSETS[i as usize] as u16);
            self.set_oam_helper0_at(
                oam,
                x,
                y,
                0x4a,
                THROWABLE_SCENERY_LARGE_DRAW_FLAGS[i as usize] | info.flags,
                2,
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.oam_allocate_from_region_b(12);
        oam = self.game_state.oam.current_pointer_usize();
        let info_y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let info_x = info.x;
        let mut i: i32 = 2;
        loop {
            let x = info_x.wrapping_add(THROWABLE_SCENERY_LARGE_EXTRA_X_OFFSETS[i as usize] as u16);
            let y = info_y.wrapping_add(12);
            self.set_oam_helper0_at(oam, x, y, 0x6c, 0x24, 2);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void ThrowableScenery_ScatterIntoDebris(int k) {  // 86ac41
    pub(in crate::zelda_rtl) fn throwable_scenery_scatter_into_debris(&mut self, k: usize) {
        if !sign8(self.sprite_slot_view(k).c()) && self.sprite_slot_view(k).c() >= 6 {
            for i in (0..4).rev() {
                let mut info = SpriteSpawnInfo::default();
                let j = self.sprite_spawn_dynamically(k, 0xec, &mut info);
                if j >= 0 {
                    let j = j as usize;
                    let value = self.sprite_slot_view(k).z();
                    self.sprite_slot_view_mut(j).set_z(value);
                    self.sprite_set_x(
                        j,
                        info.r0_x.wrapping_add(SCATTER_DEBRIS_X_OFFSETS[i] as u16),
                    );
                    self.sprite_set_y(
                        j,
                        info.r2_y.wrapping_add(SCATTER_DEBRIS_Y_OFFSETS[i] as u16),
                    );
                    let value = 1;
                    self.sprite_slot_view_mut(j).set_c(value);
                    self.sprite_schedule_for_breakage(j);
                    let value = if self.sprite_slot_view(k).c() < 7 {
                        12
                    } else {
                        0
                    };
                    self.sprite_slot_view_mut(j).set_oam_flags(value);
                }
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.throwable_scenery_scatter_after_state_clear(k);
        }
    }

    /// Resume the small-debris branch immediately after its terminal
    /// `sprite_state[k] = 0` publication. This source statement can be exposed
    /// at a host boundary before the OAM-coordinate helper returns.
    pub(in crate::zelda_rtl) fn throwable_scenery_scatter_after_state_clear(&mut self, k: usize) {
        let Some(t) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(t);
        let mut j: i32 = 29;
        while {
            let old = j as usize;
            let filled = self.garnish_slot_view(old).garnish_type() != 0;
            j -= 1;
            filled && j >= 0
        } {}
        j += 1;
        let j = j as usize;
        let value = 22;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(22);
        let value = self.sprite_slot_view(k).x_low();
        self.garnish_slot_view_mut(j).set_x_low(value);
        let value = self.sprite_slot_view(k).x_high();
        self.garnish_slot_view_mut(j).set_x_high(value);
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.sprite_slot_view(k).z() as u16)
            .wrapping_add(0x10);
        let value = y as u8;
        self.garnish_slot_view_mut(j).set_y_low(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(j).set_y_high(value);
        let value = info.flags;
        self.garnish_slot_view_mut(j).set_oam_flags(value);
        let value = self.sprite_slot_view(k).floor();
        self.garnish_slot_view_mut(j).set_floor(value);
        let value = 31;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let value = self.sprite_slot_view(k).c();
        self.garnish_slot_view_mut(j).set_sprite(value);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_OctorokStoneCrumbling(int k) {  // 86d643 — sprite_main.c:12032
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_octorok_stone_crumbling(&mut self, k: usize) {
        let info = match self.sprite_prep_oam_coord_or_double_ret(k) {
            Some(p) => PrepOamCoordsRet::from_tuple(p),
            None => return,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = (((self.sprite_slot_view(k).delay_main() >> 1) & 0xc) ^ 0xc) as usize;
        let mut i: i32 = 3;
        loop {
            let j = g + i as usize;
            let x = info
                .x
                .wrapping_add(OCTOSTONE_DRAW_X_OFFSETS[j] as i16 as u16);
            let y = info
                .y
                .wrapping_add(OCTOSTONE_DRAW_Y_OFFSETS[j] as i16 as u16);
            self.set_oam_helper0_at(oam, x, y, 0xbc, OCTOSTONE_DRAW_FLAGS[j] | 0x2d, 0);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void MedallionTablet_Draw(int k) {  // 8dd1e2
    //   Four deferred tablet tiles selected by sprite_graphics.
    // }
    pub(super) fn medallion_tablet_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &MEDALLION_TABLET_DRAW_FRAMES[base..base + 4],
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_1C_Statue(int k) {  // 86c0e8
    //   Movable statue collision, Link drag interaction, and switch sensing.
    // }
    pub(super) fn sprite_1_c_statue(&mut self, k: usize) {
        if self.sprite_slot_view(k).direction() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_direction(value);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_defense_flags();
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_direction(value);
            self.follower_link_state_mut().set_defense_flags(129);
            self.follower_link_state_mut().set_speed_setting(8);
        }
        self.movable_statue_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.statue_block_sprites(k);
        self.dungeon_environment_mut()
            .clear_water_puzzle_state_changed();
        if self.statue_check_for_switch(k) {
            self.dungeon_environment_mut()
                .set_water_puzzle_state_changed(1);
        }
        // ROM $06:C11C
        self.sprite_move_xy_marked(k, 0x06e93e);
        self.sprite_get16_bit_coords(k);
        self.sprite_check_tile_collision2(k);
        self.sprite_zero_velocity_xy(k);
        if self.sprite_check_damage_to_link_same_layer(k) {
            let value = 7;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_repel_dash();
            if self.sprite_slot_view(k).delay_aux1() != 0 {
                self.sprite_nullify_hookshot_drag();
                return;
            }
            let j = self.sprite_direction_to_face_link(k) as usize;
            let value = MOVABLE_STATUE_X_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = MOVABLE_STATUE_Y_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        } else {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 13;
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
            }
            let cur_x = self.game_state.sprites.workspace.current_sprite_x();
            let cur_y = self.game_state.sprites.workspace.current_sprite_y();
            let link_x = self.game_state.player.follower_link.x();
            let link_y = self.game_state.player.follower_link.y();
            if cur_x.wrapping_sub(link_x).wrapping_add(16) < 35
                && cur_y.wrapping_sub(link_y).wrapping_add(12) < 36
            {
                let j = self.sprite_direction_to_face_link(k) as usize;
                if self.game_state.player.follower_link.facing() == MOVABLE_STATUE_DIRECTIONS[j]
                    && !self.game_state.player.follower_link.is_running()
                {
                    self.follower_link_state_mut().mark_near_moveable_statue();
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if !self.game_state.player.follower_link.grabbing_wall_has(2)
                        || (MOVABLE_STATUE_JOYPAD_MASKS[j]
                            & self.game_state.player.follower_link.joypad1h_last())
                            == 0
                        || (self.game_state.player.follower_link.x_velocity()
                            | self.game_state.player.follower_link.y_velocity())
                            == 0
                    {
                        return;
                    }
                    let j = j ^ 1;
                    let value = MOVABLE_STATUE_X_VELOCITIES[j];
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = MOVABLE_STATUE_Y_VELOCITIES[j];
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                } else {
                    if self.sprite_slot_view(k).a() != 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_a(value);
                        self.follower_link_state_mut().set_speed_setting(0);
                        self.follower_link_state_mut().clear_grabbing_wall();
                        self.follower_link_state_mut().clear_near_moveable_statue();
                        self.follower_link_state_mut().clear_direction_lock_bits(1);
                    }
                    return;
                }
            } else {
                if self.sprite_slot_view(k).a() != 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_a(value);
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.follower_link_state_mut().clear_grabbing_wall();
                    self.follower_link_state_mut().clear_near_moveable_statue();
                    self.follower_link_state_mut().clear_direction_lock_bits(1);
                }
                return;
            }
        }
        if !self.game_state.player.follower_link.grabbing_wall_has(2) {
            self.sprite_nullify_hookshot_drag();
        }
        if (self.sprite_slot_view(k).wall_collision() & 15) == 0
            && self.sprite_slot_view(k).delay_aux4() == 0
        {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
            let value = 8;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
        }
    }

    // bool Statue_CheckForSwitch(int k) {  // 86c203
    //   Four-corner switch-tile probe.
    // }
    pub(super) fn statue_check_for_switch(&mut self, k: usize) -> bool {
        for j in (0..4).rev() {
            let x = self
                .sprite_get_x(k)
                .wrapping_add(u16::from(MOVABLE_STATUE_SWITCH_X_OFFSETS[j]));
            let y = self
                .sprite_get_y(k)
                .wrapping_add(u16::from(MOVABLE_STATUE_SWITCH_Y_OFFSETS[j]));
            let floor = self.sprite_slot_view(k).floor();
            // ROM $06:C229 JSL $06:E87B
            if !self.probe_entity_tile_long(floor, x, y).0.is_floor_switch() {
                return false;
            }
        }
        true
    }

    // -----------------------------------------------------------------------
    // void MovableStatue_Draw(int k) {  // 86c264
    //   Three deferred statue tiles.
    // }
    pub(super) fn movable_statue_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &MOVABLE_STATUE_DRAW_FRAMES);
    }

    // void Statue_BlockSprites(int k) {  // 86c277
    //   Repel overlapping active sprites away from the pushed statue.
    // }
    pub(super) fn statue_block_sprites(&mut self, k: usize) {
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        for j in (0..16).rev() {
            if self.sprite_slot_view(j).sprite_type() == 0x1c
                || j == k
                || (((j as u8) ^ self.game_state.frame.frame_counter) & 1) != 0
                || self.sprite_slot_view(j).state() < 9
            {
                continue;
            }
            let x = self.sprite_get_x(j);
            let y = self.sprite_get_y(j);
            if cur_x.wrapping_sub(x).wrapping_add(12) < 24
                && cur_y.wrapping_sub(y).wrapping_add(12) < 36
            {
                let value = 4;
                self.sprite_slot_view_mut(j).set_f(value);
                // ROM $06:C2D1
                let pt = self.sprite_project_speed_towards_location_marked(k, x, y, 32);
                let value = pt.y;
                self.sprite_slot_view_mut(j).set_y_recoil(value);
                let value = pt.x;
                self.sprite_slot_view_mut(j).set_x_recoil(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void MovableMantle_Draw(int k) {  // 9afcb3
    //   Six hand-written OAM tiles; no canonical DrawMultipleData table in C.
    // }
    pub(in crate::zelda_rtl) fn movable_mantle_draw(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x1a_fcb3);
        crate::cycle_ledger::charge(156); // $FCB3..FCBD, allocate + coordinate calls
        self.oam_allocate_from_region_b(0x20);
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Long) else {
            crate::cycle_ledger::charge(6 + 42); // taken BCS, $FCEC RTS
            return;
        };
        crate::cycle_ledger::charge(38); // $FCBF..FCC0
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..6).rev() {
            self.oam_state_mut().write_entry(
                oam,
                x.wrapping_add(MOVABLE_MANTLE_X_OFFSETS[i] as u16) as u8,
                y.wrapping_add(MOVABLE_MANTLE_Y_OFFSETS[i] as u16) as u8,
                MOVABLE_MANTLE_CHARS[i],
                MOVABLE_MANTLE_FLAGS[i],
            );
            oam += 4;
            crate::cycle_ledger::charge(474 + if i != 0 { 6 } else { 0 }); // $FCC2..FCE1
        }
        crate::cycle_ledger::charge(122); // $FCE3..FCE8, including correction JSL
        self.sprite_correct_oam_entries_long(k, 5, 2);
        crate::cycle_ledger::charge(42); // $FCEC RTS
    }

    // -----------------------------------------------------------------------
    // void Sprite_EE_MovableMantle(int k) {  // 85e819
    pub(in crate::zelda_rtl) fn sprite_ee_movable_mantle(&mut self, k: usize) {
        crate::cycle_ledger::charge_routine(0x1a_fc31, 190); // bank wrapper $FC31..FC38
        let _scope = crate::cycle_ledger::routine(0x1a_fc39);
        crate::cycle_ledger::charge(92); // $FC39..FC3C, drawing/inactive JSRs
        self.movable_mantle_draw(k);
        if self.sprite_return_if_inactive_bank1a(k) {
            return;
        }
        crate::cycle_ledger::charge(78); // $FC3F..FC43, damage JSL + BCC
        if !self.sprite_check_damage_to_link_same_layer(k) {
            crate::cycle_ledger::charge(6 + 42); // taken BCC, $FC9A RTS
            return;
        }
        crate::cycle_ledger::charge(124); // $FC45..FC49, hookshot/dash calls
        self.sprite_nullify_hookshot_drag();
        self.sprite_repel_dash();

        if { crate::cycle_ledger::charge(72); // $FC4D..FC53
            self.game_state.sprites.follower_runtime.indicator() != 1 }
            || { crate::cycle_ledger::charge(56); // $FC55..FC59
                self.game_state.inventory.items.torch() == 0 }
            || { crate::cycle_ledger::charge(48); // $FC5B..FC5E
                self.game_state.player.follower_link.is_running() }
            || { crate::cycle_ledger::charge(64); // $FC60..FC65
                self.sprite_slot_view(k).g() == 0x90 }
            || { crate::cycle_ledger::charge(56); // $FC67..FC6B
                sign8(
                self.game_state
                    .player
                    .follower_link
                    .actual_x_velocity()
                    .wrapping_sub(24),
            ) }
        {
            crate::cycle_ledger::charge(6 + 42); // taken guard, $FC9A RTS
            return;
        }

        crate::cycle_ledger::charge(172); // $FC6D..FC7B
        self.save_progress_mut().set_which_starting_point(4);
        self.sprite_slot_view_mut(k).add_subtype2(1);

        if (self.sprite_slot_view(k).subtype2() & 1) == 0 {
            crate::cycle_ledger::charge(52); // $FC7D INC
            self.sprite_slot_view_mut(k).add_g(1);
        } else {
            crate::cycle_ledger::charge(6); // $FC7B BNE taken
        }

        crate::cycle_ledger::charge(64); // $FC80..FC85
        if self.sprite_slot_view(k).g() < 8 {
            crate::cycle_ledger::charge(6 + 42); // taken BCC, $FC9A RTS
            return;
        }
        crate::cycle_ledger::charge(48); // $FC87..FC8A
        if self.game_state.system_signals.sound_effect_1() == 0 {
            crate::cycle_ledger::charge(48); // $FC8C..FC8E
            self.set_sound_effect_1(34);
        } else {
            crate::cycle_ledger::charge(6); // $FC8A BNE taken
        }
        crate::cycle_ledger::charge(116); // $FC91..FC96, including movement JSL
        let value = 2;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy(k);
        crate::cycle_ledger::charge(42); // $FC9A RTS
    }

    // -----------------------------------------------------------------------
    // void MedallionTablet_Main(int k) {  // 85f30c
    pub(super) fn medallion_tablet_main(&mut self, k: usize) {
        self.medallion_tablet_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.follower_link_state_mut()
            .clear_position_mode_bits(0x20);
        let value = 0;
        self.sprite_slot_view_mut(k).set_a(value);
        // ROM $05:F31E
        if self.sprite_check_damage_same_layer_long_marked(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
            self.sprite_slot_view_mut(k).add_a(1);
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.world.location.overworld_screen_index() != 3 {
                    self.bombos_tablet(k);
                } else {
                    self.ether_tablet(k);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 240;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let delay = self.sprite_slot_view(k).delay_main();
                    if delay == 0x20 || delay == 0x40 || delay == 0x60 {
                        let value = self.sprite_slot_view(k).graphics().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                    if (self.game_state.frame.frame_counter & 7) == 0 {
                        self.sprite_spawn_dust_cloud(k);
                    }
                }
            }
            3 => {
                let value = 4;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void BombosTablet(int k) {  // 85f355
    pub(super) fn bombos_tablet(&mut self, k: usize) {
        if self.game_state.player.follower_link.has_facing()
            || self.sprite_direction_to_face_link(k) != 2
        {
            return;
        }
        if self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add(16)
            < self.game_state.player.follower_link.y()
        {
            return;
        }
        self.medallion_tablet_activate(k, &BOMBOS_TABLET_MESSAGES, true);
    }

    // -----------------------------------------------------------------------
    // void EtherTablet(int k) {  // 85f3c4
    pub(super) fn ether_tablet(&mut self, k: usize) {
        if self.game_state.player.follower_link.has_facing()
            || self.sprite_direction_to_face_link(k) != 2
        {
            return;
        }
        if self.sprite_slot_view(k).y_low().wrapping_add(16)
            < self.game_state.player.follower_link.y() as u8
        {
            return;
        }
        self.medallion_tablet_activate(k, &ETHER_TABLET_MESSAGES, false);
    }

    fn medallion_tablet_activate(&mut self, k: usize, msg: &[u16; 2], bombos: bool) {
        if (self.game_state.player.follower_link.filtered_joypad_h() & 0x80) != 0
            && self.game_state.inventory.items.sword_type() == 2
        {
            return;
        }
        let j = if self.game_state.inventory.save_progress.hud_current_item()
            == HUD_ITEM_BOOK_MUDORA_DRAW
            && (self.game_state.player.follower_link.filtered_joypad_h() & 0x40) != 0
        {
            1usize
        } else if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) != 0 {
            0usize
        } else {
            return;
        };
        if j != 0 {
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_position_mode(32);
            self.set_sound_effect_1(0);
            if !sign8(self.game_state.inventory.items.sword_type())
                && self.game_state.inventory.items.sword_type() >= 2
            {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                if bombos {
                    self.bombos_tablet_start_cutscene();
                } else {
                    self.ether_tablet_start_cutscene();
                }
                let value = 64;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
        }
        self.sprite_show_message_unconditional(msg[j]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_PullSwitch_bounce(int k) {  // 85d6c2
    pub(super) fn sprite_pull_switch_bounce(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() == 5
            || self.sprite_slot_view(k).sprite_type() == 7
        {
            self.pull_switch_facing_up(k);
        } else {
            self.pull_switch_facing_down(k);
        }
    }

    // -----------------------------------------------------------------------
    // void PullSwitch_FacingUp(int k) {  // 85d6d4
    pub(super) fn pull_switch_facing_up(&mut self, k: usize) {
        self.pull_switch_handle_up_pulling(k);
        let mut j = self.sprite_slot_view(k).graphics();
        if j != 0 && j != 11 {
            self.follower_link_state_mut()
                .set_pull_action_state(UP_PULL_PLAYER_ACTION_STATES[usize::from(j - 1)]);
            let sprite_y = self.sprite_get_y(k);
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut()
                .set_y(sprite_y.wrapping_sub(19));
            self.follower_link_state_mut().set_x(sprite_x);
            if self.sprite_slot_view(k).delay_main() == 0 {
                j = j.wrapping_add(1);
                let value = j;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if j == 11 {
                    self.set_sound_effect_2(0x1b);
                    self.dungeon_environment_mut()
                        .set_water_puzzle_state_changed(1);
                }
                let value = UP_PULL_ANIMATION_DELAYS[usize::from(j - 2)];
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
        }
        if self.sprite_slot_view(k).sprite_type() != 7 {
            self.bad_pull_down_switch_draw(k);
        } else {
            self.bad_pull_up_switch_draw(k);
        }
    }

    // -----------------------------------------------------------------------
    // void PullSwitch_HandleUpPulling(int k) {  // 85d743
    pub(super) fn pull_switch_handle_up_pulling(&mut self, k: usize) {
        // ROM $05:D743
        if !self.sprite_check_damage_same_layer_long_marked(k) {
            return;
        }
        self.follower_link_state_mut().set_actual_velocity_xy(0, 0);
        self.sprite_repel_dash();
        self.follower_link_state_mut().clear_defense_flags();
        let y = (self.game_state.player.follower_link.y() as u8)
            .wrapping_sub(self.sprite_slot_view(k).y_low());
        if !sign8(y.wrapping_sub(2)) {
            let sprite_y = self.sprite_get_y(k);
            self.follower_link_state_mut()
                .set_y(sprite_y.wrapping_add(9));
        } else if sign8(y.wrapping_sub(244)) {
            self.follower_link_state_mut()
                .increment_player_pose_draw_counter();
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0
                && (self.game_state.player.follower_link.joypad1h_last() & 3) == 0
                && self.sprite_slot_view(k).graphics() == 0
            {
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 8;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
            }
            let sprite_y = self.sprite_get_y(k);
            self.follower_link_state_mut()
                .set_y(sprite_y.wrapping_sub(21));
        } else if sign8(
            (self.game_state.player.follower_link.x() as u8)
                .wrapping_sub(self.sprite_slot_view(k).x_low()),
        ) {
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut()
                .set_x(sprite_x.wrapping_sub(16));
        } else {
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut()
                .set_x(sprite_x.wrapping_add(14));
        }
    }

    // -----------------------------------------------------------------------
    // void PullSwitch_FacingDown(int k) {  // 85d8b5
    pub(super) fn pull_switch_facing_down(&mut self, k: usize) {
        self.pull_switch_handle_down_pulling(k);
        let mut j = self.sprite_slot_view(k).graphics();
        if j != 0 && j != 13 {
            self.follower_link_state_mut()
                .set_pull_action_state(DOWN_PULL_PLAYER_ACTION_STATES[usize::from(j - 1)]);
            let sprite_y = self.sprite_get_y(k);
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut().set_y(
                sprite_y.wrapping_add(u16::from(PULL_SWITCH_FACING_DOWN_YOFFS[usize::from(j - 1)])),
            );
            self.follower_link_state_mut().set_x(sprite_x);
            if self.sprite_slot_view(k).delay_main() == 0 {
                j = j.wrapping_add(1);
                let value = j;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if j == 13 {
                    if self.sprite_slot_view(k).sprite_type() == 6 {
                        self.dungeon_room_effects_mut()
                            .set_activate_bomb_trap_overlord(1);
                        self.set_sound_effect_1(0x3c);
                    } else {
                        self.dungeon_environment_mut()
                            .set_water_puzzle_state_changed(1);
                        self.set_sound_effect_2(0x1b);
                    }
                }
                let value = DOWN_PULL_ANIMATION_DELAYS[usize::from(j - 2)];
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
        }
        self.good_pull_switch_draw(k);
        if self.sprite_slot_view(k).pause() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void PullSwitch_HandleDownPulling(int k) {  // 85d999
    pub(super) fn pull_switch_handle_down_pulling(&mut self, k: usize) {
        // ROM $05:D999
        if !self.sprite_check_damage_same_layer_long_marked(k) {
            return;
        }
        self.follower_link_state_mut().set_actual_velocity_xy(0, 0);
        self.sprite_repel_dash();
        self.follower_link_state_mut().clear_defense_flags();
        let y = (self.game_state.player.follower_link.y() as u8)
            .wrapping_sub(self.sprite_slot_view(k).y_low());
        if !sign8(y.wrapping_sub(2)) {
            self.follower_link_state_mut()
                .increment_player_pose_draw_counter();
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0
                && (self.game_state.player.follower_link.joypad1h_last() & 3) == 0
            {
                self.follower_link_state_mut().increment_pull_action_state();
                if (self.game_state.player.follower_link.joypad1h_last() & 4) != 0
                    && self.sprite_slot_view(k).graphics() == 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = 12;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                }
            }
            let sprite_y = self.sprite_get_y(k);
            self.follower_link_state_mut()
                .set_y(sprite_y.wrapping_add(9));
        } else if sign8(y.wrapping_sub(244)) {
            let sprite_y = self.sprite_get_y(k);
            self.follower_link_state_mut()
                .set_y(sprite_y.wrapping_sub(21));
        } else if sign8(
            (self.game_state.player.follower_link.x() as u8)
                .wrapping_sub(self.sprite_slot_view(k).x_low()),
        ) {
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut()
                .set_x(sprite_x.wrapping_sub(16));
        } else {
            let sprite_x = self.sprite_get_x(k);
            self.follower_link_state_mut()
                .set_x(sprite_x.wrapping_add(14));
        }
    }

    // -----------------------------------------------------------------------
    // void BadPullDownSwitch_Draw(int k) {  // 85d7f9
    //   Five OAM entries, including the animated center pull segment.
    // }
    pub(super) fn bad_pull_down_switch_draw(&mut self, k: usize) {
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = (self.sprite_slot_view(k).graphics() as usize)
            .min(BAD_PULL_DOWN_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS.len() - 1);
        let yoff = BAD_PULL_SWITCH_CENTER_Y_OFFSETS
            [BAD_PULL_DOWN_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS[g] as usize];
        for i in (0..5).rev() {
            let y_delta =
                (BAD_PULL_DOWN_SWITCH_Y_OFFSETS[i] as i16) - if i == 2 { yoff as i16 } else { 0 };
            self.set_oam_plain_at(
                oam,
                x.wrapping_add(BAD_PULL_DOWN_SWITCH_X_OFFSETS[i] as i16 as u16) as u8,
                y.wrapping_add(y_delta as u16) as u8,
                BAD_PULL_DOWN_SWITCH_DRAW_CHARS[i],
                BAD_PULL_DOWN_SWITCH_DRAW_FLAGS[i] | 0x21,
                BAD_PULL_DOWN_SWITCH_DRAW_SIZES[i],
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries(k, 4, 0xff);
    }

    // -----------------------------------------------------------------------
    // void BadPullUpSwitch_Draw(int k) {  // 85d858
    //   Two large switch tiles with animated top offset.
    // }
    pub(super) fn bad_pull_up_switch_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = (self.sprite_slot_view(k).graphics() as usize)
            .min(BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS.len() - 1);
        let yoff = BAD_PULL_SWITCH_TOP_Y_OFFSETS
            [BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS[g] as usize];
        for i in (0..2).rev() {
            self.set_oam_helper0_at(
                oam,
                x,
                y.wrapping_sub(if i == 0 { yoff as u16 } else { 0 }),
                BAD_PULL_UP_SWITCH_CHARS[i],
                flags,
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void GoodPullSwitch_Draw(int k) {  // 85d953
    //   Two large switch tiles whose spacing follows sprite_graphics.
    // }
    pub(super) fn good_pull_switch_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let oam = self.game_state.oam.current_pointer_usize();
        let t = GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS[(self.sprite_slot_view(k).graphics() as usize)
            .min(GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS.len() - 1)];
        self.set_oam_helper0_at(oam, x, y.wrapping_sub(1), 0xee, flags, 2);
        self.set_oam_helper0_at(
            oam + 4,
            x,
            y.wrapping_sub(1).wrapping_add(t as u16),
            0xce,
            flags,
            2,
        );
        self.sprite_correct_oam_entries(k, 1, 2);
    }

    // -----------------------------------------------------------------------
    // void Overworld_DrawWoodenDoor(uint16 pos, bool unlocked) {  // 9bc952
    pub(super) fn overworld_draw_wooden_door(&mut self, pos: u16, unlocked: bool) {
        self.overworld_draw_map16_persist(pos, if unlocked { 0x0da5 } else { 0x0da4 });
        self.overworld_draw_map16_persist(
            pos.wrapping_add(2),
            if unlocked { 0x0da7 } else { 0x0da6 },
        );
        self.set_bg_vram_load_mode(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Chimney(int k) {  // 9d858f
    pub(super) fn sprite_chimney(&mut self, k: usize) {
        let value = 64;
        self.sprite_slot_view_mut(k).set_flags3(value);
        let value = 64;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_slot_view(k).delay_main() != 0 {
                return;
            }
            let value = 67;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xd1, &mut info);
            if j < 0 {
                return;
            }
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            let t = u16::from(info.r0_x as u8).wrapping_add(8);
            let value = t as u8;
            self.sprite_slot_view_mut(j).set_x_low(value);
            let value = (info.r2_y as u8)
                .wrapping_add(4)
                .wrapping_add((t >> 8) as u8);
            self.sprite_slot_view_mut(j).set_y_low(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_ai_state(value);
            let value = 67;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 67;
            self.sprite_slot_view_mut(j).set_flags3(value);
            let value = (-4i8) as u8;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = (-6i8) as u8;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
        } else {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_object_priority(value);
            self.chimney_smoke_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_move_xy_bank(k, 0x1de95d);
            self.sprite_slot_view_mut(k).add_subtype2(1);
            if (self.sprite_slot_view(k).subtype2() & 7) == 0 {
                let j = self.sprite_slot_view(k).direction() & 1;
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(if j != 0 { (-1i8) as u8 } else { 1 });
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let target = if j != 0 { (-4i8) as u8 } else { 4 };
                if self.sprite_slot_view(k).x_velocity() == target {
                    self.sprite_slot_view_mut(k).add_direction(1);
                }
            }
            if (self.sprite_slot_view(k).subtype2() & 31) == 0 {
                self.sprite_slot_view_mut(k).add_graphics(1);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_62_MasterSword(int k) {  // 8588c5
    pub(super) fn sprite_62_master_sword(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.master_sword_main(k),
            1 => self.sprite_master_sword_light_fountain(k),
            2 => self.sprite_master_sword_light_beam(k),
            3 => self.sprite_master_sword_prop(k),
            4 => self.sprite_master_sword_light_well(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_57_DesertStatue(int k) {  // 85956d
    pub(super) fn sprite_57_desert_statue(&mut self, k: usize) {
        self.desert_barrier_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:9573
        let dmg = self.sprite_check_damage_same_layer_long_marked(k);
        if dmg {
            self.sprite_nullify_hookshot_drag();
            self.sprite_repel_dash();
        }
        if self.sprite_slot_view(k).delay_main() != 0 || sign8(self.sprite_slot_view(k).ai_state())
        {
            return;
        }

        if self.sprite_slot_view(k).ai_state() == 0 {
            if self
                .game_state
                .messaging
                .runtime
                .message_or_sprite_state_cache()
                == 0
            {
                return;
            }
            let value = self
                .game_state
                .messaging
                .runtime
                .message_or_sprite_state_cache();
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.set_ambient_sound_effect(7);
        }

        if dmg && self.game_state.player.follower_link.incapacitated_timer() == 0 {
            self.follower_link_state_mut().set_incapacitated_timer(16);
            self.sprite_apply_speed_towards_link(k, 32);
        }

        let j = usize::from(self.sprite_slot_view(k).direction());
        let value = SPRITE_57_DESERT_STATUE_XV[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_57_DESERT_STATUE_YV[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        // ROM $05:95C9
        self.sprite_move_xy_marked(k, 0x05fa00);
        // ROM $05:95CC -> $05:B88D -> $06:E496
        if self.sprite_check_tile_collision_long(k) != 0 {
            let value =
                SPRITE_57_DESERT_STATUE_NEXT_D[usize::from(self.sprite_slot_view(k).direction())];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        self.follower_link_state_mut().immobilize();
        self.sprite_slot_view_mut(k).add_subtype2(1);
        if (self.sprite_slot_view(k).subtype2() & 1) == 0 {
            self.sprite_slot_view_mut(k).add_g(1);
            if self.sprite_slot_view(k).g() == 130 {
                let value = 128;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.follower_link_state_mut().clear_immobilized();
            }
        }
    }

    // -----------------------------------------------------------------------
    // void DesertBarrier_Draw(int k) {  // 859626
    pub(super) fn desert_barrier_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 1 {
            self.set_sound_effect_2(0x1b);
            self.set_ambient_sound_effect(5);
        }
        let x_step = (self.sprite_slot_view(k).delay_main() >> 1) & 1;
        self.sprite_workspace_mut().add_current_sprite_x_low(x_step);
        let (_, pt) = self.sprite_direction_and_offset_to_face_link(k);
        if pt.x.wrapping_add(0x20) < 0x40 && pt.y.wrapping_add(0x20) < 0x40 {
            self.oam_allocate_from_region_b(16);
        }
        self.sprite_draw_multiple(k, &DESERT_BARRIER_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void SageMantle_Draw(int k) {  // 85dc8a
    pub(in crate::zelda_rtl) fn sage_mantle_draw(&mut self, k: usize) {
        // Cycle ledger: SageMantle_Draw $05:DC8A (JSR target, m8 x8):
        // $05:DC8A-DC95 the table pointer stores, LDA $DB0,x, BNE (128); a
        // zero runs $05:DC97 LDA #$10, JSL Oam_AllocateFromRegionB (78), else
        // the BNE is taken (+6). $05:DC9D LDA #$04 : JMP Sprite_DrawMultiple_
        // (40) into $05:8AB1 JSL Sprite_DrawMultiple (62) ... RTS (42), a jump
        // target that charges into this scope.
        let _scope = crate::cycle_ledger::routine(0x05_dc8a);
        if self.sprite_slot_view(k).c() == 0 {
            crate::cycle_ledger::charge(128 + 78);
            self.oam_allocate_from_region_b(0x10);
        } else {
            crate::cycle_ledger::charge(128 + 6);
        }
        crate::cycle_ledger::charge(40 + 62);
        self.sprite_draw_multiple(k, &SAGE_MANTLE_DRAW_FRAMES);
        crate::cycle_ledger::charge(42);
    }

    // -----------------------------------------------------------------------
    // void Sprite_DashItem(int k) {  // 85fbf7
    pub(super) fn sprite_dash_item(&mut self, k: usize) {
        match self.sprite_slot_view(k).graphics() {
            0 => self.sprite_book_of_mudora(k),
            1 => self.sprite_bonk_key(k),
            2 => self.sprite_lumberjack_tree(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BonkKey(int k) {  // 85fc04
    pub(super) fn sprite_bonk_key(&mut self, k: usize) {
        self.sprite_draw_thin_and_tall(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:FC0B
        if self.sprite_check_damage_same_layer_long_marked(k) {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        self.dash_item_move_and_bounce(k, true);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_x()
                    .wrapping_sub(self.game_state.player.follower_link.x())
                    .wrapping_add(16)
                    < 33
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(self.game_state.player.follower_link.y())
                        .wrapping_add(24)
                        < 41
                    && (self.game_state.world.scroll.bg1_x_offset()
                        | self.game_state.world.scroll.bg1_y_offset())
                        != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = (-5i8) as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.set_sound_effect_2(27);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.sprite_slot_view(k).z() == 0 {
                    let value = self.game_state.player.follower_link.lower_level_state();
                    self.sprite_slot_view_mut(k).set_floor(value);
                }
            }
            3 => {
                self.player_resources_mut().increment_keys();
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                let bits = self.game_state.dungeon.savegame_state.savegame_state_bits()
                    | if self.sprite_slot_view(k).die_action() != 0 {
                        0x2000
                    } else {
                        0x4000
                    };
                self.dungeon_savegame_state_mut()
                    .set_savegame_state_bits(bits);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x2f);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BookOfMudora(int k) {  // 85fc9e
    pub(super) fn sprite_book_of_mudora(&mut self, k: usize) {
        // ROM $05:FC9E JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:FCA5
        if self.sprite_check_damage_same_layer_long_marked(k) {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        self.dash_item_move_and_bounce(k, false);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if !self.game_state.player.follower_link.has_facing()
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_x()
                        .wrapping_sub(self.game_state.player.follower_link.x())
                        .wrapping_add(39)
                        < 47
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(self.game_state.player.follower_link.y())
                        .wrapping_add(40)
                        < 46
                    && (self.game_state.world.scroll.bg1_x_offset()
                        | self.game_state.world.scroll.bg1_y_offset())
                        != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = (-5i8) as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.set_sound_effect_2(27);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.sprite_slot_view(k).z() == 0 {
                    let value = self.game_state.player.follower_link.lower_level_state();
                    self.sprite_slot_view_mut(k).set_floor(value);
                }
            }
            3 => {
                self.link_cancel_dash();
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x1d,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::BookOfMudora,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_book_of_mudora_item_receipt(k);
            }
            _ => {}
        }
    }

    /// Source suffix after Sprite_BookOfMudora's `Link_ReceiveItem(0x1d)`
    /// call (ROM `$85fc04`). A live timing authority suspends the
    /// decompressor there; only the sprite-kill tail remains.
    pub(in crate::zelda_rtl) fn complete_book_of_mudora_item_receipt(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
    }

    fn dash_item_move_and_bounce(&mut self, k: usize, sfx3: bool) {
        self.sprite_move_xy(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        self.sprite_move_z(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if (self.sprite_slot_view(k).z_velocity() & 254) != 0 {
                if sfx3 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x14);
                } else {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_LumberjackTree(int k) {  // 85fd4d
    pub(super) fn sprite_lumberjack_tree(&mut self, k: usize) {
        let value = 0x8f;
        self.sprite_slot_view_mut(k).set_flags2(value);
        let value = 0x47;
        self.sprite_slot_view_mut(k).set_flags4(value);
        self.dash_tree_top_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:FD5D
        if self.sprite_check_damage_same_layer_long_marked(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
        }
        // ROM $05:FD6F
        self.sprite_move_xy_marked(k, 0x05fa00);
        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        self.sprite_move_z(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_subtype2(value);
                if self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_x()
                    .wrapping_sub(self.game_state.player.follower_link.x())
                    .wrapping_add(24)
                    < 65
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(self.game_state.player.follower_link.y())
                        .wrapping_add(32)
                        < 81
                    && ((self.game_state.world.scroll.bg1_x_offset()
                        | self.game_state.world.scroll.bg1_y_offset())
                        & 0xff)
                        != 0
                {
                    let value = 20;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).z() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.set_sound_effect_2(0x1b);
                    let value = (-4i8) as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = (-4i8) as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let mut j = self.lumberjack_tree_spawn_leaves(k);
                    let ju = j as usize;
                    let value = 5;
                    self.sprite_slot_view_mut(ju).set_x_velocity(value);
                    let value = 5;
                    self.sprite_slot_view_mut(ju).set_y_velocity(value);
                    j = self.lumberjack_tree_spawn_leaves(k);
                    let ju = j as usize;
                    let value = 5;
                    self.sprite_slot_view_mut(ju).set_x_velocity(value);
                    let value = (-4i8) as u8;
                    self.sprite_slot_view_mut(ju).set_y_velocity(value);
                    j = self.lumberjack_tree_spawn_leaves(k);
                    let ju = j as usize;
                    let value = (-4i8) as u8;
                    self.sprite_slot_view_mut(ju).set_x_velocity(value);
                    let value = 4;
                    self.sprite_slot_view_mut(ju).set_y_velocity(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    if self.sprite_slot_view(k).subtype2() == 6 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_state(value);
                    }
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void DashTreeTop_Draw(int k) {  // 85fe6f
    pub(super) fn dash_tree_top_draw(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x05_fe6f));
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        // $05:FE6F..FF38, M8/X8: the two complete drawing paths.
        // Coordinate prep and CorrectOamEntries retain their own work.
        blocks.block(0x05fe6f);
        if self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet).is_none() {
            return;
        }
        blocks.block(0x05fe72);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let (x0, y0) = self
            .draw_scratch_position_mut()
            .offset_low_position(0u8.wrapping_sub(0x20), 0u8.wrapping_sub(0x20));
        if self.sprite_slot_view(k).subtype2() == 0 {
            blocks.block(0x05fe8a);
            for column in 0..4 {
                blocks.block(0x05fe95);
                if column != 3 { blocks.taken(0x05fea6); }
            }
            blocks.block(0x05fea8);
            for (i, char_flags) in DASH_TREE_TOP_GRID_CHAR_FLAGS.iter().enumerate() {
                blocks.block(0x05fedb);
                if i != 15 { blocks.taken(0x05fee9); }
                let entry_x = x0.wrapping_add(((i & 3) as u8) * 0x10);
                let entry_y = y0.wrapping_add(((i >> 2) as u8) * 0x10);
                self.oam_state_mut().set_entry_x(oam + i * 4, entry_x);
                self.oam_state_mut().set_entry_y(oam + i * 4, entry_y);
                self.oam_state_mut()
                    .set_entry_char_flags(oam + i * 4, *char_flags);
            }
            blocks.block(0x05feeb);
        } else {
            blocks.taken(0x05fe88);
            blocks.block(0x05fef7);
            let j = usize::from(self.sprite_slot_view(k).subtype2() - 1);
            for i in (0..16).rev() {
                blocks.block(0x05ff06);
                if i != 0 { blocks.taken(0x05ff2b); }
                self.oam_state_mut().write_entry(
                    oam,
                    x0.wrapping_add(DASH_TREE_TOP_DRAW_X_OFFSETS[i] as u8),
                    y0.wrapping_add(DASH_TREE_TOP_DRAW_Y_OFFSETS[i] as u8),
                    DASH_TREE_TOP_DRAW_CHARS[j],
                    DASH_TREE_TOP_DRAW_FL[j],
                );
                oam += 4;
            }
            blocks.block(0x05ff2d);
        }
        self.sprite_correct_oam_entries(k, 15, 2);
        blocks.block(if self.sprite_slot_view(k).subtype2() == 0 { 0x05fef6 } else { 0x05ff38 });
    }

    // -----------------------------------------------------------------------
    // void Sprite_1E_CrystalSwitch(int k) {  // 86b8d0
    pub(super) fn sprite_1_e_crystal_switch(&mut self, k: usize) {
        let value = (self.sprite_slot_view(k).oam_flags() & !0x0e)
            | CRYSTAL_SWITCH_PAL[usize::from(
                self.game_state
                    .dungeon
                    .environment
                    .orange_blue_barrier_state()
                    & 1,
            )];

        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.oam_allocate_defer_to_player(k);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let sparkle_y = u16::from(self.get_random_number() & 7);
            self.sprite_garnish_spawn_sparkle(
                k,
                u16::from(self.game_state.frame.frame_counter & 7),
                sparkle_y,
            );
            let value = 31;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        if self.sprite_slot_view(k).f() == 0 {
            if sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                self.sprite_check_damage_from_link(k);
            }
        } else {
            let old = self.sprite_slot_view(k).f();
            let value = old.wrapping_sub(1);
            self.sprite_slot_view_mut(k).set_f(value);
            if old == 11 {
                self.dungeon_environment_mut()
                    .toggle_orange_blue_barrier_state();
                self.set_submodule(22);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x25);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_21_WaterSwitch(int k) {  // 86b9fa
    pub(super) fn sprite_21_water_switch(&mut self, k: usize) {
        self.push_switch_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).c() != 0 {
                    self.sprite_slot_view_mut(k).subtract_b(1);
                    if self.sprite_slot_view(k).b() == 0 {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    if (self.game_state.frame.frame_counter & 3) == 0 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                    }
                } else {
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_b(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_a(1);
                    let j = usize::from(self.sprite_slot_view(k).a());
                    if j == 10 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.dungeon_environment_mut()
                            .increment_water_puzzle_state_changed();
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x25);
                    } else {
                        let value = SPRITE_21_WATER_SWITCH_DELAY[j];
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = SPRITE_21_WATER_SWITCH_DIR[j];
                        self.sprite_slot_view_mut(k).set_direction(value);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                    }
                }
            }
            2 => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_33_RupeePull(int k) {  // 86c017
    pub(super) fn sprite_33_rupee_pull(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.follower_link_state_mut()
                .set_pull_for_rupees_sprite_need();
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
        } else if self.sprite_slot_view(k).a() != 0 {
            self.follower_link_state_mut()
                .clear_pull_for_rupees_sprite_need();
            if self.game_state.player.follower_link.state_bits_has(1) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.rupee_pull_spawn_prize(k);
                self.sprite_spawn_poof_garnish(k);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_14_ThievesTownGrate(int k) {  // 86c01c
    pub(super) fn sprite_14_thieves_town_grate(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.follower_link_state_mut()
                .set_pull_for_rupees_sprite_need();
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
        } else {
            if self.sprite_slot_view(k).a() == 0 {
                return;
            }
            self.follower_link_state_mut()
                .clear_pull_for_rupees_sprite_need();
            if !self.game_state.player.follower_link.state_bits_has(1) {
                return;
            }
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
            self.OpenGargoylesDomain();
            let j = self.sprite_spawn_dust_cloud(k);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_x(j, self.sprite_get_x(k));
                self.sprite_set_y(j, self.sprite_get_y(k));
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_2D_TelepathicTile(int k) {  // 86c0b2
    pub(super) fn sprite_2_d_telepathic_tile(&mut self, _k: usize) {
        unreachable!("C Sprite_2D_TelepathicTile is an assert(0) sprite slot");
    }

    // -----------------------------------------------------------------------
    // void Sprite_38_EyeStatue(int k) {  // 86c03f
    pub(super) fn sprite_38_eye_statue(&mut self, k: usize) {
        if self.sprite_slot_view(k).b() == 0 {
            self.sprite_prep_oam_coord(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_direction_to_face_link(k) == 2
                && self.sprite_slot_view(k).draw_work_byte_2() == 9
            {
                self.dungeon_environment_mut()
                    .increment_water_puzzle_state_changed();
                let value = 1;
                self.sprite_slot_view_mut(k).set_b(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6C_MirrorPortal(int k) {  // 85af75
    pub(super) fn sprite_6_c_mirror_portal(&mut self, k: usize) {
        // Cycle ledger (an RTS-dispatch target of SpriteActive2_Main, m8 x8,
        // charging into the open Sprite_ExecuteSingle scope). $05:AF75 LDA
        // $7EF3CA : BNE (56); the dark world takes it (+6) into $05:AFDF STZ
        // $DD0,x : BRA $AFF1 (60).
        if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            crate::cycle_ledger::charge(56 + 6 + 60);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        } else {
            // $05:AF7B LDA $8A : CMP #$80 : BCC (56); a special area runs
            // the $05:AF81 RTS (42), else the BCC is taken (+6).
            if self.game_state.world.location.overworld_screen_index() >= 0x80 {
                crate::cycle_ledger::charge(56 + 56 + 42);
                return;
            }
            crate::cycle_ledger::charge(56 + 56 + 6);

            // $05:AF82 LDA $11 : CMP #$23 : BEQ (56; taken +6 in the mirror
            // submodule), $05:AF88 LDA $0FC6 : CMP #$03 : BCS (64; taken +6
            // with the sheets pending), else $05:AF8F JSL
            // Sprite_PrepAndDrawSingleLarge_ (62).
            if self.game_state.frame.submodule == 0x23 {
                crate::cycle_ledger::charge(56 + 6);
            } else if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
                crate::cycle_ledger::charge(56 + 64 + 6);
            } else {
                crate::cycle_ledger::charge(56 + 64 + 62);
            }
            if self.game_state.frame.submodule != 0x23
                && self.game_state.sprites.system.chr_halfslot_state() < 3
            {
                // ROM $05:AF8F JSL $06:DBF0
                self.sprite_draw_single_large_long(k);
            }
            // $05:AF93 JSR Sprite_ReturnIfInactive_ (46; a double return
            // when inactive), then $05:AF96-AFAC the OAM-flag rotation and
            // JSL Sprite_CheckIfLinkIsBusy : BCS (278; taken +6 into the
            // $05:B018 RTS, 42, when busy).
            crate::cycle_ledger::charge(46);
            if self.sprite_return_if_inactive_bank5(k) {
                return;
            }
            crate::cycle_ledger::charge(278);
            let j = usize::from((self.game_state.frame.frame_counter >> 2) & 3);
            let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | WARP_VORTEX_FLAGS[j];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            if self.sprite_check_if_link_is_busy() {
                crate::cycle_ledger::charge(6 + 42);
                return;
            }
            // $05:AFAE JSL Sprite_CheckDamageToPlayerSameLayer_ : BCC (78);
            // no touch takes it (+6) into $05:AFE4-AFEE (140). A touch runs
            // $05:AFB4 LDA $D90,x : BEQ (48; taken +6 to $05:AFF1), $05:AFB9
            // LDA $037B : ORA $031F : BNE (80; taken +6), $05:AFC1 LDA $02E4
            // : BNE (48; taken +6), then $05:AFC6-AFDD the mirror-warp setup
            // (264) and $05:AFDF STZ $DD0,x : BRA (60).
            // ROM $05:AFAE
            if self.sprite_check_damage_same_layer_long_marked(k) {
                let armed = self.sprite_slot_view(k).a() != 0;
                let vulnerable = (self
                    .game_state
                    .player
                    .follower_link
                    .sprite_damage_disable_timer()
                    | self.game_state.player.follower_link.blink_countdown())
                    == 0;
                let mobile = !self.game_state.player.follower_link.is_immobilized();
                crate::cycle_ledger::charge(if !armed {
                    78 + 48 + 6
                } else if !vulnerable {
                    78 + 48 + 80 + 6
                } else if !mobile {
                    78 + 48 + 80 + 48 + 6
                } else {
                    78 + 48 + 80 + 48 + 264 + 60
                });
                if armed && vulnerable && mobile {
                    self.set_submodule(0x23);
                    self.follower_link_state_mut().set_whirlpool_trigger();
                    self.set_subsubmodule(0);
                    self.follower_link_state_mut().set_actual_velocity_xy(0, 0);
                    self.follower_link_state_mut().set_handler_state(20);
                    let screen_bits =
                        self.game_state.world.location.overworld_screen_index() & 0x40;
                    self.world_palette_theme_mut()
                        .set_last_light_vs_dark_world(screen_bits);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
            } else {
                crate::cycle_ledger::charge(78 + 6 + 140);
                let value = 1;
                self.sprite_slot_view_mut(k).set_a(value);
            }
        }
        // $05:AFF1 INC $DA0,x : BNE (68; taken +6 unless the counter wrapped,
        // which runs $05:AFF6 LDA #$01 : STA $D90,x, 54), then $05:AFFB-B015
        // the bird-travel position copy (326) and $05:B018 RTS (42).
        self.sprite_slot_view_mut(k).add_b(1);
        if self.sprite_slot_view(k).b() == 0 {
            crate::cycle_ledger::charge(68 + 54);
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
        } else {
            crate::cycle_ledger::charge(68 + 6);
        }
        crate::cycle_ledger::charge(326 + 42);
        let bird = self
            .game_state
            .world
            .overworld
            .bird_travel_destinations
            .destination(15);
        let bird_x = bird.x;
        let bird_y = bird.y;
        let value = bird_x as u8;
        self.sprite_slot_view_mut(k).set_x_low(value);
        let value = (bird_x >> 8) as u8;
        self.sprite_slot_view_mut(k).set_x_high(value);
        let t = bird_y;
        let t = t.wrapping_add(8);
        let value = t as u8;
        self.sprite_slot_view_mut(k).set_y_low(value);
        let value = (t >> 8) as u8;
        self.sprite_slot_view_mut(k).set_y_high(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BA_Whirlpool(int k) {  // 9eee5a
    pub(super) fn sprite_ba_whirlpool(&mut self, k: usize) {
        if self.game_state.world.location.overworld_screen_index() == 0x1b {
            self.sprite_prep_oam_coord(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_sub(self.game_state.player.follower_link.x())
                .wrapping_add(0x40);
            let y = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_sub(self.game_state.player.follower_link.y())
                .wrapping_add(0x0f);
            if x < 0x51 && y < 0x12 {
                self.set_submodule(35);
                self.follower_link_state_mut().set_whirlpool_trigger();
                self.set_subsubmodule(0);
                self.follower_link_state_mut().set_actual_velocity_xy(0, 0);
                self.follower_link_state_mut().set_handler_state(20);
                let screen_bits = self.game_state.world.location.overworld_screen_index() & 0x40;
                self.world_palette_theme_mut()
                    .set_last_light_vs_dark_world(screen_bits);
            }
        } else {
            let j = usize::from((self.game_state.frame.frame_counter >> 3) & 3);
            let value =
                (self.sprite_slot_view(k).oam_flags() & 0x3f) | SPRITE_BA_WHIRLPOOL_OAM_FLAGS[j];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            self.oam_allocate_from_region_b(4);
            let cur_x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_sub(5);
            self.sprite_workspace_mut().set_current_sprite_x(cur_x);
            // ROM $1E:EED2 JSL $06:DBF0
            self.sprite_draw_single_large_long(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            // ROM $1E:EED9
            if self.sprite_check_damage_same_layer_long_marked(k) {
                if self.sprite_slot_view(k).a() == 0 {
                    self.set_submodule(46);
                    self.set_subsubmodule(0);
                }
            } else {
                let value = 0;
                self.sprite_slot_view_mut(k).set_a(value);
            }
        }
    }

    pub(super) fn push_switch_draw(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        let Some((_info_x, _info_y, _info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k)
        else {
            return;
        };
        let flags = if self.game_state.sprites.follower_runtime.palette_swap_flag() != 0 {
            self.sprite_slot_view(k).oam_flags() | 0x0e
        } else {
            self.sprite_slot_view(k).oam_flags() & !0x0e
        };
        let value = flags;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let r1 = (self.sprite_slot_view(k).b() >> 2) & 3;
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let oam = self.game_state.oam.current_pointer_usize();
        let base = usize::from(self.sprite_slot_view(k).direction()) * 5;
        let scratch = &self.game_state.sprites.draw_hitbox_work;
        let base_x = scratch.x_low();
        let base_y = scratch.y_low();
        let xv = base_x.wrapping_sub(r1);
        let yv = base_y.wrapping_sub(r1 >> 1);
        for i in 0..5 {
            let (x, y, ch, fl) = PUSH_SWITCH_DRAW_OAM[base + i];
            let add_x = if i == 4 { base_x } else { xv };
            let add_y = if i == 4 { base_y } else { yv };
            self.set_oam_plain_at(
                oam + i * 4,
                (x as u8).wrapping_add(add_x),
                (y as u8).wrapping_add(add_y),
                ch,
                fl | flags,
                if i == 4 { 2 } else { 0 },
            );
        }
        let big = self.game_state.oam.current_extended_pointer_usize();
        self.oam_state_mut()
            .set_extended_bytes_at(big, &[0, 0, 0, 0, 2]);
        self.sprite_correct_oam_entries(k, 4, 0xff);

        if self.sprite_slot_view(k).floor()
            == self.game_state.player.follower_link.lower_level_state()
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_c(value);
            let d = usize::from(self.sprite_slot_view(k).direction());
            let hitbox_base = d * 4;
            let x = self
                .sprite_get_x(k)
                .wrapping_add_signed(i16::from(PUSH_SWITCH_DRAW_OAM[hitbox_base].0));
            let y = self
                .sprite_get_y(k)
                .wrapping_add_signed(i16::from(PUSH_SWITCH_DRAW_OAM[hitbox_base].1));
            let mut hb = SpriteHitBox {
                r0_xlo: 0,
                r8_xhi: 0,
                r1_ylo: 0,
                r9_yhi: 0,
                r2: 0,
                r3: 0,
                r4_spr_xlo: x as u8,
                r10_spr_xhi: (x >> 8) as u8,
                r5_spr_ylo: y as u8,
                r11_spr_yhi: (y >> 8) as u8,
                r6_spr_xsize: PUSH_SWITCH_DRAW_WH[d * 2],
                r7_spr_ysize: PUSH_SWITCH_DRAW_WH[d * 2 + 1],
            };
            self.link_setup_hit_box(&mut hb);
            if self.check_if_hit_boxes_overlap(&hb) {
                let old_y = self.sprite_get_y(k);
                self.sprite_set_y(k, old_y.wrapping_add(19));
                let new_dir = self.sprite_direction_to_face_link(k);
                self.sprite_set_y(k, old_y);
                if new_dir == 0 && self.game_state.player.follower_link.facing() == 4 {
                    self.sprite_slot_view_mut(k).add_c(1);
                }
            } else if !self.sprite_check_damage_to_link_same_layer(k) {
                return;
            }
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
        }
    }
}
