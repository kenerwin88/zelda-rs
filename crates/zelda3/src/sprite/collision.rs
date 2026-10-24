//! Sprite and player hit boxes, tile collision, damage checks, and recoil.

use super::*;

impl ZeldaState {

    pub(super) fn entity_apply_rumble_to_sprites(&mut self, hb: &mut SpriteHitBox) {
        for j in (0..=15).rev() {
            if self.sprite_slot_view(j).deflection_bits() & 2 == 0
                || self.sprite_slot_view(j).e() == 0
            {
                continue;
            }
            if self.game_state.sprites.system.chr_halfslot_state() != 0x0e {
                self.sprite_setup_hit_box(j, hb);
                if !self.check_if_hit_boxes_overlap(hb) {
                    continue;
                }
            }
            let value = 0;
            self.sprite_slot_view_mut(j).set_e(value);
            self.set_sound_effect_2(0x30);
            let value = 0x30;
            self.sprite_slot_view_mut(j).set_z_velocity(value);
            let value = 0x10;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = 0x30;
            self.sprite_slot_view_mut(j).set_delay_aux3(value);
            let value = 255;
            self.sprite_slot_view_mut(j).set_stunned(value);
            if self.sprite_slot_view(j).sprite_type() == 0xd8 {
                self.sprite_transmute_to_bomb(j);
            }
        }
    }

    pub(in crate::zelda_rtl) fn sprite_setup_hit_box(&self, k: usize, hb: &mut SpriteHitBox) {
        if (self.sprite_slot_view(k).z() as i8).is_negative() {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let i = (self.sprite_slot_view(k).flags4() & 0x1f) as usize;
        let t = u16::from(self.sprite_slot_view(k).x_low())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XLO[i] as u8));
        hb.r4_spr_xlo = t as u8;
        let t_hi = u16::from(self.sprite_slot_view(k).x_high())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XHI[i] as u8))
            .wrapping_add(t >> 8);
        hb.r10_spr_xhi = t_hi as u8;

        let t = u16::from(self.sprite_slot_view(k).y_low())
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YLO[i] as u8));
        let u = t >> 8;
        let ylo = (t as u8).wrapping_sub(self.sprite_slot_view(k).z());
        hb.r5_spr_ylo = ylo;
        let t_hi = u16::from(self.sprite_slot_view(k).y_high())
            .wrapping_sub(u16::from((t as u8) < self.sprite_slot_view(k).z()));
        hb.r11_spr_yhi = t_hi
            .wrapping_add(u)
            .wrapping_add(u16::from(SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YHI[i] as u8))
            as u8;

        hb.r6_spr_xsize = SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_XSIZE[i];
        hb.r7_spr_ysize = SPRITE_SETUP_HIT_BOX_SPRITE_HITBOX_YSIZE[i];
    }

    // bool Sprite_SetupHitBox00(int k) {  // 86f1f6
    //   return (uint16)(link_x_coord - cur_sprite_x + 11) < 23 &&
    //          (uint16)(link_y_coord - cur_sprite_y + sprite_z[k] + 16) < 24;
    // }
    pub(in crate::zelda_rtl) fn sprite_setup_hit_box00(&self, k: usize) -> bool {
        self.game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
            .wrapping_add(11)
            < 23
            && self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                .wrapping_add(self.sprite_slot_view(k).z() as u16)
                .wrapping_add(16)
                < 24
    }

    // void Sprite_PlaceWeaponTink(int k) {  // 86f6ca
    //   if (repulsespark_timer)
    //     return;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    //   Sprite_PlaceRupulseSpark_2(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_place_weapon_tink(&mut self, k: usize) {
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
        self.sprite_place_rupulse_spark_2(k);
    }

    // void Sprite_PlaceRupulseSpark_2(int k) {  // 86f6d5
    //   uint16 x = Sprite_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Sprite_GetY(k) - BG2VOFS_copy2;
    //   if (x & ~0xff || y & ~0xff)
    //     return;
    //   repulsespark_x_lo = sprite_x_lo[k];
    //   repulsespark_y_lo = sprite_y_lo[k];
    //   repulsespark_timer = 5;
    //   repulsespark_floor_status = sprite_floor[k];
    // }
    pub(in crate::zelda_rtl) fn sprite_place_rupulse_spark_2(&mut self, k: usize) {
        let x = self
            .sprite_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & !0xff != 0 || y & !0xff != 0 {
            return;
        }
        let x_low = self.sprite_slot_view(k).x_low();
        self.garnish_state_mut().set_repulsespark_x_lo(x_low);
        let y_low = self.sprite_slot_view(k).y_low();
        self.garnish_state_mut().set_repulsespark_y_lo(y_low);
        self.garnish_state_mut().set_repulsespark_timer(5);
        let floor = self.sprite_slot_view(k).floor();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
    }

    // void Link_PlaceWeaponTink() {  // 86f69f
    //   if (repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int t = (uint8)link_x_coord + player_oam_x_offset;
    //   repulsespark_x_lo = t;
    //   t = (uint8)link_y_coord + player_oam_y_offset + (t >> 8);  // carry wtf
    //   repulsespark_y_lo = t;
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = Link_CalculateSfxPan() | 5;
    // }
    pub(in crate::zelda_rtl) fn link_place_weapon_tink(&mut self) {
        if self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0 {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(5);
        let t = u16::from(self.game_state.player.follower_link.x() as u8)
            + u16::from(self.game_state.player.follower_link.oam_x_offset());
        let y = u16::from(self.game_state.player.follower_link.y() as u8)
            + u16::from(self.game_state.player.follower_link.oam_y_offset())
            + (t >> 8);
        self.garnish_state_mut().set_repulsespark_x_lo(t as u8);
        self.garnish_state_mut().set_repulsespark_y_lo(y as u8);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
        self.set_sound_effect_1_with_link_pan(5);
    }

    // void Sprite_ApplyRecoilToLink(int k, uint8 vel) {  // 86f688
    //   ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, vel);
    //   link_actual_vel_x = pt.x;
    //   link_actual_vel_y = pt.y;
    //   g_ram[0xc7] = link_actual_vel_z = vel >> 1;
    //   link_z_coord = 0;
    // }
    pub(in crate::zelda_rtl) fn sprite_apply_recoil_to_link(&mut self, k: usize, vel: u8) {
        let pt = self.sprite_project_speed_towards_link(k, vel);
        self.follower_link_state_mut()
            .set_actual_velocity_xy(pt.x, pt.y);
        {
            self.follower_link_state_mut()
                .set_actual_z_velocity(vel >> 1);
            self.follower_link_state_mut()
                .set_recoil_z_velocity(vel >> 1);
            self.follower_link_state_mut().set_z(0);
        }
    }

    fn player_action_hit_box_from_table(&self, hb: &mut SpriteHitBox, t: usize, shrink: bool) {
        let mut x = self.game_state.player.follower_link.x().wrapping_add(
            PLAYER_ACTION_HIT_BOX_FROM_TABLE_X_OFFSETS[t]
                .wrapping_add(self.game_state.player.follower_link.oam_x_offset_signed())
                as i16 as u16,
        );
        let mut y = self.game_state.player.follower_link.y().wrapping_add(
            PLAYER_ACTION_HIT_BOX_FROM_TABLE_Y_OFFSETS[t]
                .wrapping_add(self.game_state.player.follower_link.oam_y_offset_signed())
                as i16 as u16,
        );
        let mut w = PLAYER_ACTION_HIT_BOX_FROM_TABLE_WIDTHS[t];
        let mut h = PLAYER_ACTION_HIT_BOX_FROM_TABLE_HEIGHTS[t];
        if shrink {
            if w >= 2 {
                let r = w.wrapping_sub(2).min(6);
                w = w.wrapping_sub(r);
                x = x.wrapping_add(u16::from(r >> 1));
            }
            if h >= 2 {
                let r = h.wrapping_sub(2).min(6);
                h = h.wrapping_sub(r);
                y = y.wrapping_add(u16::from(r >> 1));
            }
        }
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r2 = w;
        hb.r3 = h;
    }

    // void Player_SetupActionHitBox(SpriteHitBox *hb) {  // 86f5e0
    pub(in crate::zelda_rtl) fn player_setup_action_hit_box(&self, hb: &mut SpriteHitBox) {
        if self.game_state.player.follower_link.is_running() {
            let j = self.game_state.player.follower_link.facing_index();
            let x = self.game_state.player.follower_link.x().wrapping_add(
                u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_LO[j])
                    | (u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_X_HI[j]) << 8),
            );
            let y = self.game_state.player.follower_link.y().wrapping_add(
                u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_LO[j])
                    | (u16::from(PLAYER_SETUP_ACTION_HIT_BOX_RUN_Y_HI[j]) << 8),
            );
            hb.r0_xlo = x as u8;
            hb.r8_xhi = (x >> 8) as u8;
            hb.r1_ylo = y as u8;
            hb.r9_yhi = (y >> 8) as u8;
            hb.r2 = 16;
            hb.r3 = 16;
            return;
        }

        let mut t = 0usize;
        if !self.game_state.player.follower_link.item_in_hand_has(10)
            && !self.game_state.player.follower_link.position_mode_has(0x10)
        {
            if sign8(self.game_state.player.follower_link.button_b_frames()) {
                let x = self.game_state.player.follower_link.x().wrapping_sub(14);
                let y = self.game_state.player.follower_link.y().wrapping_sub(10);
                hb.r0_xlo = x as u8;
                hb.r8_xhi = (x >> 8) as u8;
                hb.r1_ylo = y as u8;
                hb.r9_yhi = (y >> 8) as u8;
                hb.r2 = 44;
                hb.r3 = 45;
                return;
            } else if PLAYER_SETUP_ACTION_HIT_BOX_SWORD_ACTION_INACTIVE_FRAMES
                [usize::from(self.game_state.player.follower_link.button_b_frames())]
                != 0
            {
                hb.r8_xhi = 0x80;
                return;
            }
            t = usize::from(self.game_state.player.follower_link.facing()) * 8
                + usize::from(self.game_state.player.follower_link.button_b_frames())
                + 1;
        }
        self.player_action_hit_box_from_table(hb, t, false);
    }

    // void Link_UpdateHitBoxWithSword(SpriteHitBox *hb) {  // new
    pub(in crate::zelda_rtl) fn link_update_hit_box_with_sword(&self, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_step_counter()
            != 0
            || sign8(self.game_state.player.follower_link.button_b_frames())
            || LINK_UPDATE_HIT_BOX_WITH_SWORD_SWORD_ACTION_INACTIVE_FRAMES
                [usize::from(self.game_state.player.follower_link.button_b_frames())]
                != 0
        {
            return;
        }
        let t = usize::from(self.game_state.player.follower_link.facing()) * 8
            + usize::from(self.game_state.player.follower_link.button_b_frames())
            + 1;
        self.player_action_hit_box_from_table(hb, t, true);
    }

    // void Sprite_DoHitBoxesFast(int k, SpriteHitBox *hb) {  // 86f645
    //   if (HIBYTE(dungmap_var8) == 0x80) {
    //     hb->r10_spr_xhi = 0x80;
    //     return;
    //   }
    //   int t;
    //   t = Sprite_GetX(k) + (int8)HIBYTE(dungmap_var8);
    //   hb->r4_spr_xlo = t;
    //   hb->r10_spr_xhi = t >> 8;
    //   t = Sprite_GetY(k) + (int8)BYTE(dungmap_var8);
    //   hb->r5_spr_ylo = t;
    //   hb->r11_spr_yhi = t >> 8;
    //   hb->r6_spr_xsize = hb->r7_spr_ysize = (sprite_type[k] == 0x6a) ? 16 : 3;
    // }
    pub(in crate::zelda_rtl) fn sprite_do_hit_boxes_fast(&self, k: usize, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .sprites
            .draw_hitbox_work
            .hitbox_x_high_offset()
            == 0x80
        {
            hb.r10_spr_xhi = 0x80;
            return;
        }
        let x = self.sprite_get_x(k).wrapping_add(
            self.game_state
                .sprites
                .draw_hitbox_work
                .hitbox_x_high_offset() as i8 as i16 as u16,
        );
        hb.r4_spr_xlo = x as u8;
        hb.r10_spr_xhi = (x >> 8) as u8;
        let y = self.sprite_get_y(k).wrapping_add(
            self.game_state
                .sprites
                .draw_hitbox_work
                .hitbox_y_low_offset() as i8 as i16 as u16,
        );
        hb.r5_spr_ylo = y as u8;
        hb.r11_spr_yhi = (y >> 8) as u8;
        let size = if self.sprite_slot_view(k).sprite_type() == 0x6a {
            16
        } else {
            3
        };
        hb.r6_spr_xsize = size;
        hb.r7_spr_ysize = size;
    }

    // void Link_SetupHitBox_conditional(SpriteHitBox *hb) {  // 86f705
    //   if (link_disable_sprite_damage)
    //     hb->r9_yhi = 0x80;
    //   else
    //     Link_SetupHitBox(hb);
    // }
    pub(in crate::zelda_rtl) fn link_setup_hit_box_conditional(&self, hb: &mut SpriteHitBox) {
        if self
            .game_state
            .player
            .follower_link
            .sprite_damage_disable_timer()
            != 0
        {
            hb.r9_yhi = 0x80;
        } else {
            self.link_setup_hit_box(hb);
        }
    }

    // void Link_SetupHitBox(SpriteHitBox *hb) {  // 86f70a
    //   hb->r3 = hb->r2 = 8;
    //   uint16 x = link_x_coord + 4;
    //   hb->r0_xlo = x;
    //   hb->r8_xhi = x >> 8;
    //   uint16 y = link_y_coord + 8;
    //   hb->r1_ylo = y;
    //   hb->r9_yhi = y >> 8;
    // }
    pub(in crate::zelda_rtl) fn link_setup_hit_box(&self, hb: &mut SpriteHitBox) {
        hb.r2 = 8;
        hb.r3 = 8;
        let x = self.game_state.player.follower_link.x().wrapping_add(4);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        let y = self.game_state.player.follower_link.y().wrapping_add(8);
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
    }

    pub(in crate::zelda_rtl) fn check_if_hit_boxes_overlap(&self, hb: &SpriteHitBox) -> bool {
        if hb.r8_xhi == 0x80 || hb.r10_spr_xhi == 0x80 {
            return false;
        }

        let mut t = i32::from(hb.r5_spr_ylo) - i32::from(hb.r1_ylo);
        let r15 = (t + i32::from(hb.r7_spr_ysize)) as u8;
        let r12 = hb
            .r11_spr_yhi
            .wrapping_sub(hb.r9_yhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r3.wrapping_add(hb.r7_spr_ysize) < r15 {
            return false;
        }

        t = i32::from(hb.r4_spr_xlo) - i32::from(hb.r0_xlo);
        let r15 = (t + i32::from(hb.r6_spr_xsize)) as u8;
        let r12 = hb
            .r10_spr_xhi
            .wrapping_sub(hb.r8_xhi)
            .wrapping_sub(u8::from(t < 0));
        t = i32::from(r12) + (((t & 0xff) + 0x80) >> 8);
        if (t as u8) != 0 {
            return t >= 0x100;
        }
        if hb.r2.wrapping_add(hb.r6_spr_xsize) < r15 {
            return false;
        }

        true
    }

    // uint8 Sprite_CheckDamageFromLink(int k) {  // 86f2b4
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_from_link(&mut self, k: usize) -> u8 {
        if (self.sprite_slot_view(k).hit_timer() & 0x80) != 0
            || self.sprite_slot_view(k).floor()
                != self.game_state.player.follower_link.lower_level_state()
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
        {
            return 0;
        }

        let mut hb = empty_sprite_hit_box();
        self.player_setup_action_hit_box(&mut hb);
        self.sprite_setup_hit_box(k, &mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if !overlap {
            return 0;
        }

        self.sprite_battle_mut().clear_damaging_enemies_timer();
        if self.game_state.player.follower_link.position_mode_has(0x10) {
            return CHECK_DAMAGE_FROM_PLAYER_CARRY | CHECK_DAMAGE_FROM_PLAYER_NON_ELEMENTAL;
        }

        if self.game_state.player.follower_link.item_in_hand_has(10) {
            if self.sprite_slot_view(k).sprite_type() >= 0xd6 {
                return 0;
            }
            if self.sprite_slot_view(k).state() == 11
                && self.sprite_slot_view(k).draw_work_byte_5() != 0
            {
                let value = 2;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = (self.sprite_slot_view(k).flags2() & 0xe0) | 3;
                self.sprite_slot_view_mut(k).set_flags2(value);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
                return CHECK_DAMAGE_FROM_PLAYER_CARRY | CHECK_DAMAGE_FROM_PLAYER_NON_ELEMENTAL;
            }
        }

        let ty = self.sprite_slot_view(k).sprite_type();
        if ty == 0x7b {
            if !sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                return 0;
            }
        } else if ty == 9 {
            if self.sprite_slot_view(k).a() == 0 {
                self.sprite_apply_recoil_to_link(k, 48);
                self.sprite_battle_mut().set_damaging_enemies_timer(144);
                self.follower_link_state_mut().set_incapacitated_timer(16);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                let value = 48;
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
                let effect = if self.game_state.enhanced_features.has(4096) {
                    0x32
                } else {
                    0
                };
                self.set_sound_effect_2_with_sprite_pan(k, effect);
                self.link_place_weapon_tink();
                return CHECK_DAMAGE_FROM_PLAYER_CARRY;
            }
        } else if ty == 0x92 {
            if self.sprite_slot_view(k).c() >= 3 {
                self.sprite_apply_recoil_to_link(k, 32);
                self.sprite_battle_mut().set_damaging_enemies_timer(144);
                self.follower_link_state_mut().set_incapacitated_timer(16);
            } else {
                return self.sprite_check_damage_from_link_getting_out(k);
            }
        } else if ty == 0x26 || ty == 0x13 || ty == 2 {
            const SPRITE_DAMAGE_FACING_BY_DIRECTION: [u8; 4] = [4, 6, 0, 2];
            let cond = (ty == 0x13
                && SPRITE_DAMAGE_FACING_BY_DIRECTION
                    [usize::from(self.sprite_slot_view(k).direction() & 3)]
                    == self.game_state.player.follower_link.facing())
                || ty == 2;
            self.sprite_attempt_zap_damage(k);
            self.sprite_apply_recoil_to_link(k, 32);
            self.sprite_battle_mut().set_damaging_enemies_timer(16);
            self.follower_link_state_mut().set_incapacitated_timer(16);
            if cond {
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                self.link_place_weapon_tink();
            }
            return 0;
        } else if matches!(ty, 0xcb | 0xcd | 0xcc | 0xd6 | 0xd7 | 0xce | 0x54) {
            self.sprite_apply_recoil_to_link(k, 32);
            self.sprite_battle_mut().set_damaging_enemies_timer(144);
            self.follower_link_state_mut().set_incapacitated_timer(16);
        }

        if (self.sprite_slot_view(k).deflection_bits() & 4) == 0 {
            self.sprite_attempt_zap_damage(k);
            return CHECK_DAMAGE_FROM_PLAYER_CARRY;
        }

        self.sprite_check_damage_from_link_getting_out(k)
    }

    fn sprite_check_damage_from_link_getting_out(&mut self, k: usize) -> u8 {
        if self.game_state.sprite_battle.damaging_enemies_timer() == 0 {
            self.sprite_apply_recoil_to_link(k, 4);
            self.follower_link_state_mut().set_incapacitated_timer(16);
            self.sprite_battle_mut().set_damaging_enemies_timer(16);
        }
        self.link_place_weapon_tink();
        CHECK_DAMAGE_FROM_PLAYER_CARRY
    }

    // bool Sprite_CheckDamageToLink(int k) {  // 86f145
    //   if (link_disable_sprite_damage)
    //     return false;
    //   return Sprite_CheckDamageToPlayer_1(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_to_link(&mut self, k: usize) -> bool {
        self.game_state
            .player
            .follower_link
            .sprite_damage_disable_timer()
            == 0
            && self.sprite_check_damage_to_player_1(k)
    }

    // bool Sprite_CheckDamageToPlayer_1(int k) {  // 86f14a
    //   if ((k ^ frame_counter) & 3 | sprite_hit_timer[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_same_layer(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_to_player_1(&mut self, k: usize) -> bool {
        (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && self.sprite_slot_view(k).hit_timer() == 0
            && self.sprite_check_damage_to_link_same_layer(k)
    }

    // bool Sprite_CheckDamageToLink_same_layer(int k) {  // 86f154
    //   if (link_is_on_lower_level != sprite_floor[k])
    //     return false;
    //   return Sprite_CheckDamageToLink_ignore_layer(k);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_to_link_same_layer(&mut self, k: usize) -> bool {
        self.game_state.player.follower_link.lower_level_state() == self.sprite_slot_view(k).floor()
            && self.sprite_check_damage_to_link_ignore_layer(k)
    }

    // bool Sprite_CheckDamageToLink_ignore_layer(int k) {  // 86f15c
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_damage_to_link_ignore_layer(&mut self, k: usize) -> bool {
        let carry = if self.sprite_slot_view(k).flags4() != 0 {
            let mut hitbox = empty_sprite_hit_box();
            self.link_setup_hit_box(&mut hitbox);
            if (0xd8..=0xe6).contains(&self.sprite_slot_view(k).sprite_type())
                && self
                    .game_state
                    .enhanced_features
                    .has(FEATURES0_COLLECT_ITEMS_WITH_SWORD_SPRITE)
            {
                self.link_update_hit_box_with_sword(&mut hitbox);
            }
            self.sprite_setup_hit_box(k, &mut hitbox);
            self.check_if_hit_boxes_overlap(&hitbox)
        } else {
            self.sprite_setup_hit_box00(k)
        };

        if sign8(self.sprite_slot_view(k).flags2()) {
            return carry;
        }
        if !carry || self.game_state.player.follower_link.has_auxiliary_state() {
            return false;
        }

        if !self.game_state.player.follower_link.is_bunny_mirror()
            && !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
            && (self.sprite_slot_view(k).flags5() & 0x20) != 0
            && self.game_state.inventory.items.shield_type() != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            let t = if self.game_state.player.follower_link.button_b_frames() != 0 {
                SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SHIELD_BLOCK_FACING_TO_DIRECTION
                    [self.game_state.player.follower_link.facing_index() & 3]
            } else {
                self.game_state.player.follower_link.facing()
            };
            // ROM $06:F19F `LDY $0DE0,X`: the shield comparison indexes by the
            // sprite's direction and leaves it in Y (see
            // `rom_damage_check_y_register`).
            self.rom_damage_check_y_register = Some(self.sprite_slot_view(k).direction());
            if t == SPRITE_CHECK_DAMAGE_TO_LINK_IGNORE_LAYER_SPRITE_DAMAGE_FACING_BY_DIRECTION
                [usize::from(self.sprite_slot_view(k).direction() & 3)]
            {
                self.sprite_sfx_queue_sfx2_with_pan(k, 6);
                self.sprite_place_rupulse_spark_2(k);
                match self.sprite_slot_view(k).sprite_type() {
                    0x95 => {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                        return false;
                    }
                    0x9b => {
                        self.sprite_invert_xy_speeds(k);
                        self.sprite_slot_view_mut(k).xor_direction(1);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 9;
                        self.sprite_slot_view_mut(k).set_state(value);
                        return false;
                    }
                    0x1b => {
                        self.sprite_schedule_for_breakage(k);
                        return false;
                    }
                    0x0c => {
                        self.sprite_func3(k);
                        return true;
                    }
                    _ => return false,
                }
            }
        }

        self.sprite_attempt_damage_to_link_plus_recoil(k);
        if self.sprite_slot_view(k).sprite_type() == 0x0c {
            self.sprite_func3(k);
        }
        true
    }

    // void Sprite_AttemptDamageToLinkWithCollisionCheck(int k) {  // 86f3ca
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_attempt_damage_to_link_with_collision_check(&mut self, k: usize) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) != 0 {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        self.link_setup_hit_box_conditional(&mut hb);
        let overlap = self.check_if_hit_boxes_overlap(&hb);
        if overlap {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
    }

    // void Guard_ParrySwordAttacks(int k) {  // 86eb5e
    pub(in crate::zelda_rtl) fn guard_parry_sword_attacks(&mut self, k: usize) {
        if self.game_state.player.follower_link.lower_level_state()
            != self.sprite_slot_view(k).floor()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.has_auxiliary_state()
            || sign8(self.sprite_slot_view(k).hit_timer())
        {
            return;
        }
        let mut hb = empty_sprite_hit_box();
        self.sprite_do_hit_boxes_fast(k, &mut hb);
        if self.game_state.player.follower_link.position_mode_has(0x10)
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
        {
            self.sprite_attempt_damage_to_link_with_collision_check(k);
            return;
        }
        self.guard_parry_sword_attacks_after_hitbox(k, hb);
    }

    pub(in crate::zelda_rtl) fn guard_parry_sword_attacks_after_hitbox(
        &mut self,
        k: usize,
        mut hb: SpriteHitBox,
    ) {
        self.player_setup_action_hit_box(&mut hb);
        let button_neg = sign8(self.game_state.player.follower_link.button_b_frames());
        let action_overlap = self.check_if_hit_boxes_overlap(&hb);
        if button_neg || !action_overlap {
            self.sprite_setup_hit_box(k, &mut hb);
            let body_overlap = self.check_if_hit_boxes_overlap(&hb);
            if !body_overlap {
                self.sprite_attempt_damage_to_link_with_collision_check(k);
            } else {
                self.sprite_attempt_zap_damage(k);
            }
            return;
        }
        if self.sprite_slot_view(k).sprite_type() != 0x6a {
            let j = usize::from(self.get_random_number() & 7);
            let value = GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_HITBOX_SIZE_BY_DIRECTION[j];
            self.sprite_slot_view_mut(k).set_f(value);
        }
        let j = usize::from(self.get_random_number() & 7);
        self.follower_link_state_mut().set_incapacitated_timer(
            GUARD_PARRY_SWORD_ATTACKS_GUARD_PARRY_SWORD_STEP_BY_DIRECTION[j],
        );
        let fast_sword = sign8(
            self.game_state
                .player
                .follower_link
                .button_b_frames()
                .wrapping_sub(9),
        );
        let pt = self.sprite_project_speed_towards_link(k, if fast_sword { 32 } else { 24 });
        let value = 0u8.wrapping_sub(pt.x);
        self.sprite_slot_view_mut(k).set_x_recoil(value);
        let value = 0u8.wrapping_sub(pt.y);
        self.sprite_slot_view_mut(k).set_y_recoil(value);
        self.sprite_apply_recoil_to_link(k, if fast_sword { 8 } else { 16 });
        self.link_place_weapon_tink();
        self.sprite_battle_mut().set_damaging_enemies_timer(0x90);
    }

    // void Sprite_AttemptDamageToLinkPlusRecoil(int k) {  // 86f3db
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_attempt_damage_to_link_plus_recoil(&mut self, k: usize) {
        if (self.game_state.player.follower_link.blink_countdown()
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer())
            != 0
        {
            return;
        }

        self.follower_link_state_mut().set_incapacitated_timer(19);
        self.sprite_apply_recoil_to_link(k, 24);
        self.follower_link_state_mut().set_auxiliary_state(1);
        let idx = 3 * usize::from(self.sprite_slot_view(k).bump_damage() & 0x0f)
            + usize::from(self.game_state.inventory.items.armor());
        // ROM $06:F3FF `TAY`: the damage table index stays in Y for callers
        // that keep using Y afterwards (Sprite_C3_Gibo's pursuit target).
        self.rom_damage_check_y_register = Some(idx as u8);
        self.follower_link_state_mut()
            .set_given_damage(SPRITE_ATTEMPT_DAMAGE_TO_LINK_PLUS_RECOIL_PLAYER_DAMAGES[idx]);
        if self.sprite_slot_view(k).sprite_type() == 0x61 && self.sprite_slot_view(k).c() != 0 {
            let actual_x_velocity = self.sprite_slot_view(k).x_velocity().wrapping_mul(2);
            let actual_y_velocity = self.sprite_slot_view(k).y_velocity().wrapping_mul(2);
            self.follower_link_state_mut()
                .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
        }
    }

    // void Sprite_AttemptZapDamage(int k) {  // 86ec02
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_attempt_zap_damage(&mut self, k: usize) {
        let ty = self.sprite_slot_view(k).sprite_type();
        let electric = (ty == 0x7a
            || (ty == 0x0d && self.game_state.inventory.items.sword_type() < 4)
            || ((ty == 0x24 || ty == 0x23) && self.sprite_slot_view(k).delay_main() != 0))
            && self.sprite_slot_view(k).state() == 9;
        if electric {
            if self.game_state.player.follower_link.blink_countdown() == 0 {
                let value = 64;
                self.sprite_slot_view_mut(k).set_delay_aux1(value);
                self.follower_link_state_mut().set_electrocute_on_touch(64);
                self.sprite_attempt_damage_to_link_plus_recoil(k);
            }
        } else {
            let vel = if sign8(
                self.game_state
                    .player
                    .follower_link
                    .button_b_frames()
                    .wrapping_sub(9),
            ) {
                0x50
            } else {
                0x40
            };
            let pt = self.sprite_project_speed_towards_link(k, vel);
            let value = 0u8.wrapping_sub(pt.x);
            self.sprite_slot_view_mut(k).set_x_recoil(value);
            let value = 0u8.wrapping_sub(pt.y);
            self.sprite_slot_view_mut(k).set_y_recoil(value);
            self.sprite_calculate_sword_damage(k);
        }
    }

    // bool Sprite_CheckTileProperty(int k, int j) {  // 86e73c
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_tile_property(&mut self, k: usize, j: i32) -> bool {
        let j = (j >> 1) as usize;

        let (x, y, in_bounds) = self.sprite_tile_property_coordinates(j);
        if !in_bounds {
            if self.sprite_slot_view(k).flags2() & 0x40 != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                return false;
            }
            return true;
        }
        let (tile, x) = self.sprite_get_tile_attribute(k, x, y);
        self.sprite_classify_tile_property(k, x, y, tile)
    }

    fn sprite_tile_property_coordinates(&self, j: usize) -> (u16, u16, bool) {
        let x;
        let y;
        let in_bounds;
        if self.game_state.world.location.is_indoors() {
            x = (self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(8)
                & 0x01ff)
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_X[j] as i16 as u16)
                .wrapping_sub(8);
            y = (self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(8)
                & 0x01ff)
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_Y[j] as i16 as u16)
                .wrapping_sub(8);
            in_bounds = x < 0x0200 && y < 0x0200;
        } else {
            x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_X[j] as i16 as u16);
            y = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(SPRITE_CHECK_TILE_PROPERTY_FUNC5_Y[j] as i16 as u16);
            in_bounds = x.wrapping_sub(self.game_state.sprites.garnish_runtime.sprcoll_x_word())
                < self.game_state.sprites.garnish_runtime.sprcoll_x_size()
                && y.wrapping_sub(self.game_state.sprites.garnish_runtime.sprcoll_y_word())
                    < self.game_state.sprites.garnish_runtime.sprcoll_y_size();
        }
        (x, y, in_bounds)
    }

    fn sprite_classify_tile_property(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        tile: NativeTile,
    ) -> bool {
        if self.sprite_slot_view(k).deflection_bits() & 8 != 0 {
            match tile.sprite_probe() {
                EntityCollision::Ledge => {
                    if self.game_state.world.location.is_outdoors() {
                        let value = 4;
                        self.sprite_slot_view_mut(k).set_e(value);
                    }
                }
                EntityCollision::Passable => {}
                _ => return self.entity_sloped_tile_collision(x, y).unwrap_or(true),
            }
            return false;
        }

        if self.sprite_slot_view(k).flags5() & 0x40 != 0 {
            let typ = self.sprite_slot_view(k).sprite_type();
            if (typ == 0xd2 || typ == 0x8a) && tile == NativeTile::SHALLOW_WATER {
                return false;
            }
            if (typ == 0x94 && self.sprite_slot_view(k).e() == 0)
                || typ == 0xe3
                || typ == 0x8c
                || typ == 0x9a
                || typ == 0x81
            {
                return tile != NativeTile::DEEP_WATER && tile != NativeTile::SHALLOW_WATER;
            }
        }

        if !tile.blocks_sprites() {
            return false;
        }
        if let Some(blocked) = self.entity_sloped_tile_collision(x, y) {
            return blocked;
        }
        if self.game_state.sprites.workspace.tile() == NativeTile::SPIKE_CACTUS {
            if self.sprite_slot_view(k).f() != 0
                && !sign8(self.sprite_slot_view(k).incoming_damage())
            {
                if self.sprite_slot_view(k).sprite_type() == 0x88
                    && self.game_state.enhanced_features.has(4096)
                {
                    if self.sprite_slot_view(k).hit_timer() == 0 {
                        self.ancilla_check_damage_to_sprite_preset(k, 6);
                    }
                } else {
                    self.ancilla_check_damage_to_sprite_preset(k, 4);
                }
                if self.sprite_slot_view(k).hit_timer() != 0 {
                    let value = 153;
                    self.sprite_slot_view_mut(k).set_hit_timer(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_f(value);
                }
            }
        } else if self.game_state.sprites.workspace.tile() == NativeTile::PIT {
            return self.sprite_slot_view(k).flags() & 1 == 0 || self.sprite_slot_view(k).f() == 0;
        }
        true
    }

    pub(in crate::zelda_rtl) fn sprite_vertical_tile_attribute_loaded(&mut self, k: usize) {
        let direction = if sign8(self.sprite_slot_view(k).y_velocity()) {
            0
        } else {
            1
        };
        let j = usize::from((self.sprite_slot_view(k).flags() & 0xf0) >> 2) + direction;
        let (x, y, in_bounds) = self.sprite_tile_property_coordinates(j);
        assert!(
            in_bounds,
            "source tile attribute publication requires an in-bounds lookup"
        );
        self.sprite_get_tile_attribute(k, x, y);
    }

    pub(in crate::zelda_rtl) fn sprite_vertical_collision_after_attribute_loaded(&mut self, k: usize) {
        let direction = if sign8(self.sprite_slot_view(k).y_velocity()) {
            0
        } else {
            1
        };
        let j = usize::from((self.sprite_slot_view(k).flags() & 0xf0) >> 2) + direction;
        let (mut x, y, in_bounds) = self.sprite_tile_property_coordinates(j);
        assert!(in_bounds);
        if self.game_state.world.location.is_outdoors() {
            x >>= 3;
        }
        let tile = self.game_state.sprites.workspace.tile();
        if self.sprite_classify_tile_property(k, x, y, tile) {
            self.sprite_apply_vertical_tile_collision(k, direction as i32);
        }
    }

    // void Sprite_CheckForTileInDirection_horizontal(int k, int yy) {  // 86e5b8
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_for_tile_in_direction_horizontal(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }

        let idx = (yy as usize) & 3;
        self.sprite_slot_view_mut(k).or_wall_collision(
            SPRITE_CHECK_FOR_TILE_IN_DIRECTION_HORIZONTAL_SPRITE_TILE_DIRECTION_BITS[idx],
        );
        if (self.sprite_slot_view(k).subtype() & 7) < 5 {
            let n = if self.sprite_slot_view(k).f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(k, if (yy & 1) != 0 { -n } else { n }, 0);
        }
    }

    // void Sprite_CheckForTileInDirection_vertical(int k, int yy) {  // 86e5ee
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_for_tile_in_direction_vertical(&mut self, k: usize, yy: i32) {
        if !self.sprite_check_tile_in_direction(k, yy) {
            return;
        }

        self.sprite_apply_vertical_tile_collision(k, yy);
    }

    fn sprite_apply_vertical_tile_collision(&mut self, k: usize, yy: i32) {
        let idx = (yy as usize) & 3;
        self.sprite_slot_view_mut(k).or_wall_collision(
            SPRITE_CHECK_FOR_TILE_IN_DIRECTION_VERTICAL_SPRITE_TILE_DIRECTION_BITS[idx],
        );
        if (self.sprite_slot_view(k).subtype() & 7) < 5 {
            let n = if self.sprite_slot_view(k).f() != 0 {
                3
            } else {
                1
            };
            self.sprite_add_xy(k, 0, if (yy & 1) != 0 { -n } else { n });
        }
    }

    // bool Sprite_CheckTileInDirection(int k, int yy) {  // 86e72f
    //   uint8 t = (sprite_flags[k] & 0xf0);
    //   yy = 2 * ((t >> 2) + yy);
    //   return Sprite_CheckTileProperty(k, yy);
    // }
    pub(in crate::zelda_rtl) fn sprite_check_tile_in_direction(&mut self, k: usize, yy: i32) -> bool {
        let t = i32::from(self.sprite_slot_view(k).flags() & 0xf0);
        self.sprite_check_tile_property(k, 2 * ((t >> 2) + yy))
    }

    // bool Entity_CheckSlopedTileCollision(uint16 x, uint16 y) {  // 86e8fe
    //   ...see sprite.c...
    // }
    /// Slope test against the sprite scratch tile, which an outdoor ancilla
    /// probe leaves unpublished; a scratch tile without an entity slope
    /// profile reports `None`.
    pub(in crate::zelda_rtl) fn entity_sloped_tile_collision(&self, x: u16, y: u16) -> Option<bool> {
        self.game_state
            .sprites
            .workspace
            .tile()
            .entity_slope_blocks(x, y)
    }

    // void Sprite_DrawRippleIfInWater(int k) {  // 9eff8d
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_draw_ripple_if_in_water(&mut self, k: usize) {
        let lag_free = self.lag_free;
        let charge = |master| { if lag_free { crate::cycle_ledger::charge(master); } };
        // $1E:FF8D..FFDD, M8/X8; the child bodies retain their own prices.
        charge(72);
        if self.sprite_slot_view(k).draw_i() == 8 {
            charge(6); // $FF93 taken BEQ.
        } else {
            charge(32); // $FF95..FF97 second tile test.
        }
        if self.sprite_slot_view(k).draw_i() != 8 && self.sprite_slot_view(k).draw_i() != 9 {
            charge(6 + 44); // $FF97 taken BNE, $FFDD RTL.
            return;
        }
        charge(64); // $FF99..FF9E flags3 test.
        if self.sprite_slot_view(k).flags3() & 0x20 != 0 {
            charge(238); // $FFA0..FFB6 X adjustment and type test.
            let x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_sub(4);
            self.sprite_workspace_mut().set_current_sprite_x(x);
            if self.sprite_slot_view(k).sprite_type() == 0xdf {
                charge(174); // $FFB8..FFC6 Y adjustment.
                let y = self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_y()
                    .wrapping_sub(7);
                self.sprite_workspace_mut().set_current_sprite_y(y);
            } else {
                charge(6); // $FFB6 taken BNE.
            }
        } else {
            charge(6); // $FF9E taken BEQ.
        }
        charge(276 + 44); // $FFC9..FFDD calls, allocation size, RTL.
        self.sprite_draw_water_ripple(k);
        // $06:84BD..84C0 wrapper around the separately priced coordinate body.
        charge(90);
        self.sprite_get16_bit_coords(k);
        self.oam_allocate_from_region_a(((self.sprite_slot_view(k).flags2() & 0x1f) + 1) * 4);
    }

    // void ThrownSprite_CheckDamageToSprites(int k) {  // 86e172
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn thrown_sprite_check_damage_to_sprites(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux4() != 0
            || (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity()) == 0
        {
            return;
        }
        for i in (0..=15usize).rev() {
            if i != self.game_state.sprites.system.cur_object_index() as usize
                && self.sprite_slot_view(k).sprite_type() != 0xd2
                && self.sprite_slot_view(i).state() >= 9
                && ((((i as u8) ^ self.game_state.frame.frame_counter) & 3)
                    | self.sprite_slot_view(i).ignore_projectile()
                    | self.sprite_slot_view(i).hit_timer())
                    == 0
                && self.sprite_slot_view(k).floor() == self.sprite_slot_view(i).floor()
            {
                self.thrown_sprite_check_damage_to_single_sprite(k, i as i32);
            }
        }
    }

    // void ThrownSprite_CheckDamageToSingleSprite(int k, int j) {  // 86e1b2
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn thrown_sprite_check_damage_to_single_sprite(&mut self, k: usize, j: i32) {
        let j = j as usize;
        let t =
            i32::from(self.sprite_slot_view(k).y_low()) - i32::from(self.sprite_slot_view(k).z());
        let u = ((t & 0xff) + 8) as u8;
        let mut hb = SpriteHitBox {
            r0_xlo: self.sprite_slot_view(k).x_low(),
            r8_xhi: self.sprite_slot_view(k).x_high(),
            r1_ylo: u,
            r9_yhi: self
                .sprite_slot_view(k)
                .y_high()
                .wrapping_add(u8::from((t & 0xff) + 8 >= 0x100))
                .wrapping_sub(u8::from(t < 0)),
            r2: 15,
            r3: 8,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.sprite_setup_hit_box(j, &mut hb);
        if !self.check_if_hit_boxes_overlap(&hb) {
            return;
        }
        if self.sprite_slot_view(j).sprite_type() == 0x3f {
            self.sprite_place_weapon_tink(k);
        } else {
            let a = if self.sprite_slot_view(k).sprite_type() == 0xec
                && self.sprite_slot_view(k).c() == 2
                && self.game_state.world.location.is_outdoors()
            {
                1
            } else {
                3
            };
            self.ancilla_check_damage_to_sprite_preset(j, a);
            let value = self.sprite_slot_view(k).x_velocity().wrapping_mul(2);
            self.sprite_slot_view_mut(j).set_x_recoil(value);
            let value = self.sprite_slot_view(k).y_velocity().wrapping_mul(2);
            self.sprite_slot_view_mut(j).set_y_recoil(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
        }
        self.sprite_apply_ricochet(k);
    }

    // void Sprite_KillFriends() {
    //   for(int j = 15; j >= 0; j--) {
    //     if (j != cur_object_index && sprite_state[j] && !(sprite_defl_bits[j] & 2)
    //         && sprite_type[j] != 0x7a) {
    //       sprite_state[j] = 6;
    //       sprite_delay_main[j] = 15;
    //       sprite_flags3[j] = 0;
    //       sprite_flags5[j] = 0;
    //       sprite_flags2[j] = 3;
    //     }
    //   }
    // }
    pub(in crate::zelda_rtl) fn sprite_kill_friends(&mut self) {
        let cur = self.game_state.sprites.system.cur_object_index() as usize;
        for j in (0..=15usize).rev() {
            if j == cur {
                continue;
            }
            if self.sprite_slot_view(j).state() == 0
                || (self.sprite_slot_view(j).deflection_bits() & 2) != 0
                || self.sprite_slot_view(j).sprite_type() == 0x7a
            {
                continue;
            }
            let value = 6;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 15;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags3(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags5(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
        }
    }

    // void Sprite_Func8(int k) {  // 86e0ab
    //   sprite_state[k] = 1;
    //   sprite_delay_main[k] = 0x1f;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 0x20);
    // }
    pub(in crate::zelda_rtl) fn sprite_func8(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 0x1f;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
    }

    // void Sprite_Func22(int k) {  // 86e0f6
    //   sound_effect_1 = Sprite_CalculateSfxPan(k) | 0x28;
    //   sprite_state[k] = 3;
    //   sprite_delay_main[k] = 15;
    //   sprite_ai_state[k] = 0;
    //   GetRandomNumber(); // wtf
    //   sprite_flags2[k] = 3;
    // }
    pub(in crate::zelda_rtl) fn sprite_func22(&mut self, k: usize) {
        self.set_sound_effect_1_with_sprite_pan(k, 0x28);
        let value = 3;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 15;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_ai_state(value);
        self.get_random_number();
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_Func3(int k) {  // 86efda
    //   sprite_state[k] = 6;
    //   sprite_delay_main[k] = 31;
    //   sprite_flags2[k] = 3;
    // }
    pub(in crate::zelda_rtl) fn sprite_func3(&mut self, k: usize) {
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);
        let value = 31;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags2(value);
    }

    // void Sprite_SpawnSecret(int k) {  // 868264
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_secret(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            let roll = self.get_random_number();
            if (roll & 8) != 0 {
                return;
            }
        }
        let mut b = self.game_state.dungeon_secret.pending_kind();
        if !self.game_state.dungeon_secret.has_pending_kind() {
            return;
        }
        if b == 4 {
            b = 19 + (self.get_random_number() & 3);
        }
        let i = b.wrapping_sub(1) as usize;
        if i >= SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE.len()
            || SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE[i] == 0
        {
            return;
        }

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(
            k,
            SPRITE_SPAWN_SECRET_SECRET_SPAWN_ITEMS_BY_TILE[i],
            &mut info,
        );
        if j < 0 {
            return;
        }
        let ju = j as usize;
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_SPAWN_FLAGS[i];
        self.sprite_slot_view_mut(ju).set_ai_state(value);
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_IGNORE_PROJECTILE_FLAGS[i];
        self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
        let value = SPRITE_SPAWN_SECRET_SECRET_ITEM_Z_VELOCITIES[i];
        self.sprite_slot_view_mut(ju).set_z_velocity(value);
        self.sprite_set_x(
            ju,
            info.r0_x
                .wrapping_add(u16::from(SPRITE_SPAWN_SECRET_SECRET_ITEM_X_LOW_OFFSETS[i])),
        );
        self.sprite_set_y(ju, info.r2_y);
        let value = info.r4_z;
        self.sprite_slot_view_mut(ju).set_z(value);
        let value = 0;
        self.sprite_slot_view_mut(ju).set_graphics(value);
        let value = 32;
        self.sprite_slot_view_mut(ju).set_delay_aux4(value);
        let value = 48;
        self.sprite_slot_view_mut(ju).set_delay_aux2(value);

        let ty = self.sprite_slot_view(ju).sprite_type();
        if ty == 0xe4 {
            self.sprite_prep_small_key(ju);
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
        } else if ty == 0x0b {
            self.set_sound_effect_1(0x30);
            if self.game_state.dungeon.room_tracking.room_index2() == 1 {
                let value = 1;
                self.sprite_slot_view_mut(ju).set_subtype(value);
            }
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
        } else if ty == 0x41 || ty == 0x42 {
            self.set_sound_effect_2(4);
            let value = 0;
            self.sprite_slot_view_mut(ju).set_incoming_damage(value);
            let value = 160;
            self.sprite_slot_view_mut(ju).set_hit_timer(value);
        } else if ty == 0x3e {
            let value = 9;
            self.sprite_slot_view_mut(ju).set_oam_flags(value);
        } else {
            let value = 255;
            self.sprite_slot_view_mut(ju).set_stunned(value);
            if ty == 0x79 {
                let value = 32;
                self.sprite_slot_view_mut(ju).set_a(value);
            }
        }
    }

    // void Ancilla_CheckDamageToSprite_preset(int k, int a) {  // 86ece0
    //   if (a == 15 && sprite_z[k] != 0)
    //     return;
    //   if (a != 0 && a != 7) {
    //     Sprite_Func15(k, a);
    //     return;
    //   }
    //   Sprite_Func15(k, a);
    //   if (sprite_give_damage[k] || repulsespark_timer)
    //     return;
    //   repulsespark_timer = 5;
    //   int j = SPRITE_SHARED_WORK_A;
    //   repulsespark_x_lo = ancilla_x_lo[j] + 4;
    //   repulsespark_y_lo = ancilla_y_lo[j];
    //   repulsespark_floor_status = link_is_on_lower_level;
    //   sound_effect_1 = 0;
    //   SpriteSfx_QueueSfx2WithPan(k, 5);
    // }
    pub(in crate::zelda_rtl) fn ancilla_check_damage_to_sprite_preset(&mut self, k: usize, a: u8) {
        if a == 15 && self.sprite_slot_view(k).z() != 0 {
            return;
        }

        let carry = (a == 0 || a == 7) && self.sprite_func15_carry_out(k, a);
        self.sprite_func15(k, a);
        if a != 0 && a != 7 {
            return;
        }
        if self.sprite_slot_view(k).incoming_damage() != 0
            || self.game_state.sprites.garnish_runtime.repulsespark_timer() != 0
        {
            return;
        }
        self.garnish_state_mut().set_repulsespark_timer(5);
        let j = self.game_state.sprites.workspace.shared_scratch_a() as usize;
        let ancilla = self.ancilla_slot_view(j);
        let x_low = ancilla.x_low().wrapping_add(4 + u8::from(carry));
        let y_low = ancilla.y_low();
        self.garnish_state_mut().set_repulsespark_x_lo(x_low);
        self.garnish_state_mut().set_repulsespark_y_lo(y_low);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.garnish_state_mut()
            .set_repulsespark_floor_status(floor);
        self.set_sound_effect_1(0);
        self.sprite_sfx_queue_sfx2_with_pan(k, 5);
    }

    // void Sprite_MiniMoldorm_Recoil(int k) {  // 86eec8
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_mini_moldorm_recoil(&mut self, k: usize) {
        let _scope = crate::cycle_ledger::routine(0x06_eec8);
        if self.sprite_slot_view(k).state() < 9 {
            crate::cycle_ledger::charge(112); // LDA/CMP/BCC taken and actual RTS.
            return;
        }
        let state = self.sprite_slot_view(k).state();
        self.temp_counter_mut().set(state);
        let dmg = self.sprite_slot_view(k).incoming_damage();
        if dmg >= 251 {
            crate::cycle_ledger::charge(if dmg == 253 { 160 } else { 198 });
            self.sprite_mini_moldorm_recoil_after_classification(k, dmg);
            return;
        }
        let health = self.sprite_slot_view(k).health();
        let t = i32::from(health) - i32::from(dmg);
        self.sprite_slot_view_mut(k).set_health(t as u8);
        self.sprite_slot_view_mut(k).set_incoming_damage(0);
        // Classification and actual health arithmetic through the return or
        // the first death-processing instruction, independently Source checked.
        crate::cycle_ledger::charge(if t > 0 {
            462
        } else if t == 0 {
            404
        } else {
            414
        });
        if t <= 0 {
            self.sprite_mini_moldorm_recoil_after_health(k);
        }
    }

    pub(in crate::zelda_rtl) fn finish_native_sprite_recoil(
        &mut self,
        k: usize,
        tail: crate::cycle_models::sprite_recoil::RecoilTail,
    ) {
        use crate::cycle_models::sprite_recoil::RecoilTail;
        let _scope = crate::cycle_ledger::routine(0x06_eec8);
        match tail {
            RecoilTail::Burn => self.sprite_mini_moldorm_recoil_after_classification(k, 253),
            RecoilTail::Stun(dmg) => self.sprite_mini_moldorm_recoil_after_classification(k, dmg),
            RecoilTail::Death => self.sprite_mini_moldorm_recoil_after_health(k),
            RecoilTail::Returned => {
                unreachable!("actual native return already resumed the timer caller")
            }
        }
    }

    fn sprite_mini_moldorm_recoil_after_classification(&mut self, k: usize, dmg: u8) {
        assert!(dmg >= 251);
        if dmg == 253 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
            let value = 7;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0x70;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).flags2().wrapping_add(2);
            self.sprite_slot_view_mut(k).set_flags2(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            return;
        }

        if dmg >= 251 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_incoming_damage(value);
            if self.sprite_slot_view(k).state() == 11 {
                return;
            }
            let value = u8::from(dmg == 254);
            self.sprite_slot_view_mut(k).set_draw_work_byte_5(value);
            if self.sprite_slot_view(k).draw_work_byte_5() != 0 {
                self.sprite_slot_view_mut(k).or_deflection_bits(8);
                self.sprite_slot_view_mut(k).and_flags5(!0x80);
                self.sprite_sfx_queue_sfx2_with_pan(k, 15);
                let value = 24;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_slot_view_mut(k).and_bump_damage(!0x80);
                self.sprite_zero_velocity_xy(k);
            }
            let value = 11;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            const HIT_TIMER24_STUN_VALUES: [u8; 5] = [0x20, 0x80, 0, 0, 0xff];
            let value = HIT_TIMER24_STUN_VALUES[dmg.wrapping_add(5) as usize];
            self.sprite_slot_view_mut(k).set_stunned(value);
            if self.sprite_slot_view(k).sprite_type() == 0x23 {
                let value = 0x24;
                self.sprite_slot_view_mut(k).set_sprite_type(value);
            }
            return;
        }
    }

    fn sprite_mini_moldorm_recoil_after_health(&mut self, k: usize) {
        if self.sprite_slot_view(k).die_action() == 0 {
            if self.sprite_slot_view(k).state() == 11 {
                let value = 3;
                self.sprite_slot_view_mut(k).set_die_action(value);
            }
            if self.sprite_slot_view(k).draw_work_byte_1() != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_draw_work_byte_1(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_flags5(value);
            }
        }

        let ty = self.sprite_slot_view(k).sprite_type();
        if ty != 0x1b {
            self.sprite_sfx_queue_sfx3_with_pan(k, 9);
        }

        if ty == 0x40 {
            let screen = self.game_state.world.location.overworld_screen_index() as usize;
            self.set_overworld_event_bits(screen, 0x40);
        } else if ty == 0xec {
            if self.sprite_slot_view(k).c() == 2 {
                self.throwable_scenery_transmute_to_debris(k);
            }
            return;
        }

        if self.sprite_slot_view(k).state() == 10 {
            self.follower_link_state_mut().clear_state_bits();
            self.follower_link_state_mut().clear_picking_throw_state();
        }
        let value = 6;
        self.sprite_slot_view_mut(k).set_state(value);

        if ty == 0x0c {
            self.sprite_func3(k);
        } else if ty == 0x92 {
            self.sprite_kill_friends();
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcb {
            let value = 128;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0xcc || ty == 0xcd {
            let value = 128;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 96;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x53 {
            let value = 35;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common2(k);
        } else if ty == 0x54 {
            let value = 5;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 0xc0;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0xc0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x09 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 160;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x7a {
            self.sprite_kill_friends();
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if !self.game_state.world.region.is_in_dark_world() {
                let value = 10;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 255;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            } else {
                let value = 255;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 8;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 9;
                self.sprite_slot_view_mut(1).set_ai_state(value);
                let value = 9;
                self.sprite_slot_view_mut(2).set_ai_state(value);
                let value = 0;
                self.sprite_slot_view_mut(1).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(2).set_graphics(value);
            }
            self.sprite_mini_moldorm_recoil_out_common(k);
        } else if ty == 0x23 && self.sprite_slot_view(k).c() == 0 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
        } else if ty == 0x0f {
            let value = 0;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        } else if self.sprite_slot_view(k).flags() & 2 == 0 {
            let value = if self.sprite_slot_view(k).hit_timer() & 0x80 != 0 {
                31
            } else {
                15
            };
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).flags2().wrapping_add(4);
            self.sprite_slot_view_mut(k).set_flags2(value);
            if self.game_state.scratch_counter.value() == 11 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_flags5(value);
            }
        } else {
            if ty != 0xa2 {
                self.sprite_kill_friends();
            }
            let value = 4;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_a(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_hit_timer(value);
            self.sprite_mini_moldorm_recoil_out_common(k);
        }
    }
    fn sprite_mini_moldorm_recoil_out_common(&mut self, k: usize) {
        self.follower_link_state_mut().increment_menu_block_flag();
        self.sprite_mini_moldorm_recoil_out_common2(k);
    }

    fn sprite_mini_moldorm_recoil_out_common2(&mut self, k: usize) {
        self.set_sound_effect_2(0);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x22);
    }

    // bool Sprite_ReturnIfRecoiling(int k) {
    //   ...see sprite.c:3143...
    // }
    pub(in crate::zelda_rtl) fn sprite_return_if_recoiling(&mut self, k: usize) -> bool {
        if self.sprite_slot_view(k).f() == 0 {
            return false;
        }
        if self.sprite_slot_view(k).f() & 0x7f == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            return false;
        }
        let yvbak = self.sprite_slot_view(k).y_velocity();
        let xvbak = self.sprite_slot_view(k).x_velocity();
        let new_f = self.sprite_slot_view(k).f().wrapping_sub(1);
        let value = new_f;
        self.sprite_slot_view_mut(k).set_f(value);
        if new_f == 0
            && (self.sprite_slot_view(k).x_recoil().wrapping_add(0x20) >= 0x40
                || self.sprite_slot_view(k).y_recoil().wrapping_add(0x20) >= 0x40)
        {
            let value = 144;
            self.sprite_slot_view_mut(k).set_f(value);
        }
        let i = self.sprite_slot_view(k).f();
        // !sign8(i) -> top bit clear
        if (i & 0x80) == 0
            && (self.game_state.frame.frame_counter
                & SPRITE_RETURN_IF_RECOILING_SPRITE_RECOIL_DIRECTION_MASKS[(i >> 2) as usize])
                == 0
        {
            let value = self.sprite_slot_view(k).y_recoil();
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = self.sprite_slot_view(k).x_recoil();
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let bump = self.sprite_slot_view(k).bump_damage();
            let t = if (bump as i8) >= 0 {
                self.sprite_check_tile_collision(k) & 0xf
            } else {
                0
            };
            if (bump as i8) >= 0 && t != 0 {
                if t < 4 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_recoil(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_recoil(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            } else {
                self.sprite_move_xy(k);
            }
        }
        let value = yvbak;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let value = xvbak;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_slot_view(k).sprite_type() != 0x7a
    }

    // uint8 Sprite_CheckTileCollision(int k) {  // 85b88d
    //   Sprite_CheckTileCollision2(k);
    //   return sprite_wallcoll[k];
    // }
    //
    // Canonical 1:1 port. Runs the 2-layer collision pass and returns the
    // cached `sprite_wallcoll[k]` byte.
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision(&mut self, k: usize) -> u8 {
        self.sprite_check_tile_collision2(k);
        self.sprite_slot_view(k).wall_collision()
    }

    /// `Sprite_CheckTileCollision` on the single-layer path through its
    /// direction probes: the wall-collision clear and the vertical and
    /// horizontal `Sprite_CheckForTileInDirection` calls. The `$68` tile
    /// property probe and its consumers are left to
    /// `sprite_check_tile_collision_from_property_probe`.
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_until_property_probe(&mut self, k: usize) {
        self.sprite_check_tile_collision_clear_for_direction_probes(k);
        if self.sprite_slot_view(k).y_velocity() != 0 {
            self.sprite_check_for_tile_in_direction_vertical(
                k,
                if sign8(self.sprite_slot_view(k).y_velocity()) {
                    0
                } else {
                    1
                },
            );
        }
        if self.sprite_slot_view(k).x_velocity() != 0 {
            self.sprite_check_for_tile_in_direction_horizontal(
                k,
                if sign8(self.sprite_slot_view(k).x_velocity()) {
                    2
                } else {
                    3
                },
            );
        }
    }

    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_from_property_probe(&mut self, k: usize) -> u8 {
        self.sprite_check_tile_collision_after_directions(k);
        self.sprite_slot_view(k).wall_collision()
    }

    /// `Sprite_CheckTileCollision2`'s wall-collision clear on the single-layer
    /// direction-probe path that every checkpointed caller requires.
    fn sprite_check_tile_collision_clear_for_direction_probes(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_wall_collision(0);
        assert!(
            sign8(self.sprite_slot_view(k).flags4())
                || self.game_state.dungeon.room_load.header_collision() == 0,
            "tile-collision probe checkpoint requires the single-layer path"
        );
        assert_eq!(
            self.sprite_slot_view(k).flags2() & 0x20,
            0,
            "tile-collision probe checkpoint requires the direction probes"
        );
    }

    /// Runs `Sprite_CheckTileCollision` through `stage` for a checkpointed
    /// caller on the interrupted host.
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_until_stage(
        &mut self,
        k: usize,
        stage: crate::SpriteTileCollisionStage,
    ) {
        use crate::SpriteTileCollisionStage as S;
        match stage {
            S::Entered => {}
            S::Cleared => self.sprite_check_tile_collision_clear_for_direction_probes(k),
            S::VerticalProbeDone => {
                self.sprite_check_tile_collision_clear_for_direction_probes(k);
                if self.sprite_slot_view(k).y_velocity() != 0 {
                    self.sprite_check_for_tile_in_direction_vertical(
                        k,
                        if sign8(self.sprite_slot_view(k).y_velocity()) {
                            0
                        } else {
                            1
                        },
                    );
                }
            }
            S::ProbesCompleted => self.sprite_check_tile_collision_until_property_probe(k),
        }
    }

    /// Finishes `Sprite_CheckTileCollision` from `stage` on resume and returns
    /// the wall-collision byte the caller consumes.
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_from_stage(
        &mut self,
        k: usize,
        stage: crate::SpriteTileCollisionStage,
    ) -> u8 {
        use crate::SpriteTileCollisionStage as S;
        match stage {
            S::Entered | S::Cleared => self.sprite_check_tile_collision(k),
            S::VerticalProbeDone => {
                if self.sprite_slot_view(k).x_velocity() != 0 {
                    self.sprite_check_for_tile_in_direction_horizontal(
                        k,
                        if sign8(self.sprite_slot_view(k).x_velocity()) {
                            2
                        } else {
                            3
                        },
                    );
                }
                self.sprite_check_tile_collision_from_property_probe(k)
            }
            S::ProbesCompleted => self.sprite_check_tile_collision_from_property_probe(k),
        }
    }

    // void Sprite_CheckTileCollision2(int k) {  // 86e4ab
    //   sprite_wallcoll[k] = 0;
    //   if (sign8(sprite_flags4[k]) || !dung_hdr_collision) {
    //     Sprite_CheckTileCollisionSingleLayer(k);
    //     return;
    //   }
    //   SPRITE_SHARED_WORK_A = sprite_floor[k];
    //   sprite_floor[k] = 1;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   if (dung_hdr_collision == 4) {
    //     sprite_floor[k] = SPRITE_SHARED_WORK_A;
    //     return;
    //   }
    //   sprite_floor[k] = 0;
    //   Sprite_CheckTileCollisionSingleLayer(k);
    //   byte_7FFABC[k] = sprite_tiletype;
    // }
    //
    // 1:1 port of the dispatcher around `Sprite_CheckTileCollisionSingleLayer`.
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision2(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_wall_collision(value);
        let f4 = self.sprite_slot_view(k).flags4();
        let dung_coll = self.game_state.dungeon.room_load.header_collision();
        // sign8: top bit set.
        if (f4 & 0x80) != 0 || dung_coll == 0 {
            self.sprite_check_tile_collision_single_layer(k);
            return;
        }
        let floor = self.sprite_slot_view(k).floor();
        self.sprite_workspace_mut().set_shared_scratch_a(floor);
        let value = 1;
        self.sprite_slot_view_mut(k).set_floor(value);
        self.sprite_check_tile_collision_single_layer(k);
        if dung_coll == 4 {
            let value = self.game_state.sprites.workspace.shared_scratch_a();
            self.sprite_slot_view_mut(k).set_floor(value);
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_floor(value);
        self.sprite_check_tile_collision_single_layer(k);
        // byte_7FFABC[k] = sprite_tiletype — write-through to the
        // dual-layer cache so the next iteration sees the lower-layer tile.
        let tile = self.game_state.sprites.workspace.tile();
        self.dual_layer_tile_cache_mut().set_tile(k, tile);
    }

    // void Sprite_CheckTileCollisionSingleLayer(int k) {  // 86e4db
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_single_layer(&mut self, k: usize) {
        if self.sprite_slot_view(k).flags2() & 0x20 != 0 {
            if self.sprite_check_tile_property(k, 0x6a) {
                let value = self.sprite_slot_view(k).wall_collision().wrapping_add(1);
                self.sprite_slot_view_mut(k).set_wall_collision(value);
            }
            return;
        }

        if sign8(self.sprite_slot_view(k).flags4())
            || self.game_state.dungeon.room_load.header_collision() == 0
        {
            if self.sprite_slot_view(k).y_velocity() != 0 {
                self.sprite_check_for_tile_in_direction_vertical(
                    k,
                    if sign8(self.sprite_slot_view(k).y_velocity()) {
                        0
                    } else {
                        1
                    },
                );
            }
            if self.sprite_slot_view(k).x_velocity() != 0 {
                self.sprite_check_for_tile_in_direction_horizontal(
                    k,
                    if sign8(self.sprite_slot_view(k).x_velocity()) {
                        2
                    } else {
                        3
                    },
                );
            }
        } else {
            self.sprite_check_for_tile_in_direction_vertical(k, 1);
            self.sprite_check_for_tile_in_direction_vertical(k, 0);
            self.sprite_check_for_tile_in_direction_horizontal(k, 3);
            self.sprite_check_for_tile_in_direction_horizontal(k, 2);
        }

        self.sprite_check_tile_collision_after_directions(k);
    }

    pub(in crate::zelda_rtl) fn sprite_check_tile_collision_after_directions(&mut self, k: usize) {
        if sign8(self.sprite_slot_view(k).flags5()) || self.sprite_slot_view(k).z() != 0 {
            return;
        }

        self.sprite_check_tile_property(k, 0x68);
        let tile = self.game_state.sprites.workspace.tile();
        self.sprite_slot_view_mut(k)
            .set_draw_i(tile.cartridge_attribute());
        if tile == NativeTile::WATER_STAIRCASE {
            if self.game_state.oam.has_sprite_sorting() && self.sprite_slot_view(k).state() == 11 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_floor(value);
            }
        } else if tile == NativeTile::PIT {
            if self.sprite_slot_view(k).flags() & 1 != 0 {
                if self.game_state.world.location.is_outdoors() {
                    self.sprite_func8(k);
                } else {
                    let value = 5;
                    self.sprite_slot_view_mut(k).set_state(value);
                    if self.sprite_slot_view(k).sprite_type() == 0x13
                        || self.sprite_slot_view(k).sprite_type() == 0x26
                    {
                        self.sprite_slot_view_mut(k).and_oam_flags(!1);
                        let value = 63;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = 95;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
            }
        } else if tile == NativeTile::MOVING_FLOOR {
            if self.game_state.sprites.dual_layer_tile_cache.tile(k) == NativeTile::WATER_STAIRCASE
            {
                self.sprite_fall_adjust_position(k);
                self.sprite_slot_view_mut(k).or_wall_collision(0x20);
            }
        } else if let Some(direction) = tile.conveyor_direction() {
            self.sprite_apply_conveyor(k, direction);
        } else if tile == NativeTile::DEEP_WATER
            && self.game_state.dungeon.room_load.header_collision() == 4
        {
            // A water-current room carries sprites like the third conveyor.
            self.sprite_apply_conveyor(k, 2);
        }
    }

    // uint8 Sprite_GetTileAttribute(int k, uint16 *x, uint16 y) {  // 86e883
    //   return GetTileAttribute(sprite_floor[k], x, y);
    // }
    pub(in crate::zelda_rtl) fn sprite_get_tile_attribute(&mut self, k: usize, x: u16, y: u16) -> (NativeTile, u16) {
        self.probe_entity_tile(self.sprite_slot_view(k).floor(), x, y)
    }
}
