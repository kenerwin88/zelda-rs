//! Garnish effect drawing, collision, spawning, and execution.

use super::*;

impl ZeldaState {

    // uint16 Garnish_GetX(int k) {
    //   return garnish_x_lo[k] | garnish_x_hi[k] << 8;
    // }
    pub(in crate::zelda_rtl) fn garnish_get_x(&self, k: usize) -> u16 {
        self.garnish_slot_view(k).x()
    }

    // uint16 Garnish_GetY(int k) {
    //   return garnish_y_lo[k] | garnish_y_hi[k] << 8;
    // }
    pub(in crate::zelda_rtl) fn garnish_get_y(&self, k: usize) -> u16 {
        self.garnish_slot_view(k).y()
    }

    // bool Garnish_ReturnIfPrepFails(int k, Point16U *pt) {  // 86e75e
    //   uint16 x = Garnish_GetX(k) - BG2HOFS_copy2;
    //   uint16 y = Garnish_GetY(k) - BG2VOFS_copy2;
    //   if (x >= 256 || y >= 256) {
    //     garnish_type[k] = 0;
    //     return true;
    //   }
    //   pt->x = x;
    //   pt->y = y - 16;
    //   return false;
    // }
    pub(in crate::zelda_rtl) fn garnish_return_if_prep_fails(&mut self, k: usize, pt: &mut Point16U) -> bool {
        let x = self
            .garnish_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .garnish_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x >= 256 || y >= 256 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return true;
        }
        pt.x = x;
        pt.y = y.wrapping_sub(16);
        false
    }

    pub(in crate::zelda_rtl) fn set_oam_plain_at(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut().write_entry(oam, x, y, charnum, flags);
        let ext_index = (oam - OAM_BUF) / 4;
        let value = big;
        self.oam_state_mut().set_extended_byte(ext_index, value);
    }

    // void Garnish_SparkleCommon(int k, uint8 shift) {  // 86dfb1
    //   static const uint8 kGarnishSparkle_Char[4] = {0x83, 0xc7, 0x80, 0xb7};
    //   uint8 t = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   int j = garnish_sprite[k];
    //   SetOamPlain(oam, pt.x, pt.y, kGarnishSparkle_Char[t],
    //                (sprite_oam_flags[j] | sprite_obj_prio[j]) & 0xf0 | 4, 0);
    // }
    pub(in crate::zelda_rtl) fn garnish_sparkle_common(&mut self, k: usize, shift: u8) {
        let t = usize::from(self.garnish_slot_view(k).countdown() >> shift);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        let flags = (self.sprite_slot_view(j).oam_flags()
            | self.sprite_slot_view(j).object_priority())
            & 0xf0
            | 4;
        self.set_oam_plain_at(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH_SPARKLE_COMMON_GARNISH_SPARKLE_CHAR[t],
            flags,
            0,
        );
    }

    // void Garnish_DustCommon(int k, uint8 shift) {  // 86dfdc
    //   static const uint8 kRunningManDust_Char[3] = {0xdf, 0xcf, 0xa9};
    //   tmp_counter = garnish_countdown[k] >> shift;
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   OamEnt *oam = GetOamCurPtr();
    //   SetOamPlain(oam, pt.x, pt.y, kRunningManDust_Char[tmp_counter], 0x24, 0);
    // }
    pub(in crate::zelda_rtl) fn garnish_dust_common(&mut self, k: usize, shift: u8) {
        let counter = self.garnish_slot_view(k).countdown() >> shift;
        self.temp_counter_mut().set(counter);
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH_DUST_COMMON_RUNNING_MAN_DUST_CHAR
                [usize::from(self.game_state.scratch_counter.value())],
            0x24,
            0,
        );
    }

    // void Garnish12_Sparkle(int k) { Garnish_SparkleCommon(k, 2); }
    pub(in crate::zelda_rtl) fn garnish12_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 2);
    }

    // void Garnish_SimpleSparkle(int k) { Garnish_SparkleCommon(k, 3); }
    pub(in crate::zelda_rtl) fn garnish_simple_sparkle(&mut self, k: usize) {
        self.garnish_sparkle_common(k, 3);
    }

    // void Garnish14_KakKidDashDust(int k) { Garnish_DustCommon(k, 2); }
    pub(in crate::zelda_rtl) fn garnish14_kak_kid_dash_dust(&mut self, k: usize) {
        self.garnish_dust_common(k, 2);
    }

    // void Garnish_WaterTrail(int k) { Garnish_DustCommon(k, 3); }
    pub(in crate::zelda_rtl) fn garnish_water_trail(&mut self, k: usize) {
        self.garnish_dust_common(k, 3);
    }

    // void Garnish04_LaserTrail(int k) {  // 86e000
    //   static const uint8 kLaserBeamTrail_Char[2] = {0xd2, 0xf3};
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, kLaserBeamTrail_Char[garnish_oam_flags[k]], 0x25, 0);
    // }
    pub(in crate::zelda_rtl) fn garnish04_laser_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at(
            oam,
            pt.x as u8,
            pt.y as u8,
            GARNISH04_LASER_TRAIL_LASER_BEAM_TRAIL_CHAR
                [usize::from(self.garnish_slot_view(k).oam_flags())],
            0x25,
            0,
        );
    }

    // void Garnish06_ZoroTrail(int k) {  // 86e025
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x75, sprite_oam_flags[j] | sprite_obj_prio[j], 0);
    // }
    pub(in crate::zelda_rtl) fn garnish06_zoro_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x75,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            0,
        );
    }

    // void Garnish01_FireSnakeTail(int k) {  // 86e03e
    //   Point16U pt;
    //   if (Garnish_ReturnIfPrepFails(k, &pt))
    //     return;
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), pt.x, pt.y, 0x28, sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(in crate::zelda_rtl) fn garnish01_fire_snake_tail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            oam,
            pt.x as u8,
            pt.y as u8,
            0x28,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            2,
        );
    }

    // void Garnish02_MothulaBeamTrail(int k) {  // 86e057
    //   int j = garnish_sprite[k];
    //   SetOamPlain(GetOamCurPtr(), garnish_x_lo[k] - BG2HOFS_copy2, garnish_y_lo[k] - BG2VOFS_copy2, 0xaa,
    //                sprite_oam_flags[j] | sprite_obj_prio[j], 2);
    // }
    pub(in crate::zelda_rtl) fn garnish02_mothula_beam_trail(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            oam,
            self.garnish_slot_view(k)
                .x_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low()),
            self.garnish_slot_view(k)
                .y_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low()),
            0xaa,
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            2,
        );
    }

    // void Garnish_CheckPlayerCollision(int k, int x, int y) {  // 89b459
    //   if ((k ^ frame_counter) & 7 | countdown_for_blink | link_disable_sprite_damage)
    //     return;
    //
    //   if ((uint8)(link_x_coord - BG2HOFS_copy2 - x + 12) < 24 &&
    //       (uint8)(link_y_coord - BG2VOFS_copy2 - y + 22) < 28) {
    //     link_auxiliary_state = 1;
    //     link_incapacitated_timer = 16;
    //     link_give_damage = 16;
    //     link_actual_vel_x ^= 255;
    //     link_actual_vel_y ^= 255;
    //   }
    // }
    /// `Garnish_CheckPlayerCollision` ($09:B459). The ROM chains 8-bit
    /// `SBC`/`ADC` without resetting the carry: the x test starts from the
    /// caller's carry (`carry_in`), each `SBC` borrows into the next, the
    /// `ADC #$0C` adds the final borrow flag, and the y test starts from the
    /// `CMP #$18` result (a passing x leaves the carry clear, so the y chain
    /// begins one lower). Route host 1555077: Ganon's bat flame at screen
    /// (0xa1, 0x7c) misses Link at (0xa3, 0x66) in the ROM (y chain yields
    /// 0xff), while the carry-free `link_y - bg - y + 22 < 28` form hits.
    pub(in crate::zelda_rtl) fn garnish_check_player_collision(
        &mut self,
        k: usize,
        x: i32,
        y: i32,
        carry_in: bool,
    ) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 7)
            | self.game_state.player.follower_link.blink_countdown()
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
            != 0
        {
            return;
        }

        fn sbc8(a: u8, b: u8, carry: bool) -> (u8, bool) {
            let r = i32::from(a) - i32::from(b) - i32::from(!carry);
            (r as u8, r >= 0)
        }
        fn adc8(a: u8, b: u8, carry: bool) -> (u8, bool) {
            let r = u32::from(a) + u32::from(b) + u32::from(carry);
            (r as u8, r > 0xff)
        }

        let link_x = self.game_state.player.follower_link.x() as u8;
        let link_y = self.game_state.player.follower_link.y() as u8;
        let bg_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2() as u8;
        let bg_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2() as u8;
        let (dx, carry) = sbc8(link_x, bg_x, carry_in);
        let (dx, carry) = sbc8(dx, x as u8, carry);
        let (dx, _) = adc8(dx, 0x0c, carry);
        if dx >= 0x18 {
            return;
        }
        // `CMP #$18` with dx < 0x18 clears the carry for the y chain.
        let (dy, carry) = sbc8(link_y, bg_y, false);
        let (dy, carry) = sbc8(dy, y as u8, carry);
        let (dy, _) = adc8(dy, 0x16, carry);
        if dy >= 0x1c {
            return;
        }
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().set_given_damage(16);
        self.follower_link_state_mut().xor_actual_velocity_xy(255);
    }

    // void Garnish15_ArrghusSplash(int k) {  // 89b178
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish15_arrghus_splash(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from((self.garnish_slot_view(k).countdown() >> 1) & 6);
        for i in (0..=1).rev() {
            let j = i + g;
            self.set_oam_plain_at(
                oam,
                pt.x.wrapping_add(GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_X[j] as i16 as u16) as u8,
                pt.y.wrapping_add(GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_Y[j] as i16 as u16) as u8,
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_CHAR[j],
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_FLAGS[j],
                GARNISH15_ARRGHUS_SPLASH_ARRGHUS_SPLASH_EXT[j],
            );
            oam += 4;
        }
    }

    // void Garnish13_PyramidDebris(int k) {  // 89b216
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish13_pyramid_debris(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();

        let garnish = self.garnish_slot_view(k);
        let y = (i32::from(garnish.y_low()) << 8)
            + i32::from(garnish.y_subpixel())
            + ((garnish.y_velocity() as i8 as i32) << 4);
        let value = y as u8;
        self.garnish_slot_view_mut(k).set_y_subpixel(value);
        let value = (y >> 8) as u8;
        self.garnish_slot_view_mut(k).set_y_low(value);

        let garnish = self.garnish_slot_view(k);
        let x = (i32::from(garnish.x_low()) << 8)
            + i32::from(garnish.x_subpixel())
            + ((garnish.x_velocity() as i8 as i32) << 4);
        let value = x as u8;
        self.garnish_slot_view_mut(k).set_x_subpixel(value);
        let value = (x >> 8) as u8;
        self.garnish_slot_view_mut(k).set_x_low(value);

        let value = self.garnish_slot_view(k).y_velocity().wrapping_add(3);
        self.garnish_slot_view_mut(k).set_y_velocity(value);
        let t = self
            .garnish_slot_view(k)
            .x_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
        if t >= 248 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return;
        }
        self.oam_state_mut().set_entry_x(oam, t);
        let t = self
            .garnish_slot_view(k)
            .y_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        if t >= 240 {
            let value = 0;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
            return;
        }
        self.oam_state_mut().set_entry_y(oam, t);
        self.oam_state_mut().set_entry_char(oam, 0x5c);
        let flags = (self.game_state.frame.frame_counter << 3) & 0xc0 | 0x34;
        self.oam_state_mut().set_entry_flags(oam, flags);
        let ext_index = (oam - OAM_BUF) / 4;
        let value = 0;
        self.oam_state_mut().set_extended_byte(ext_index, value);
    }

    // void Garnish11_WitheringGanonBatFlame(int k) {  // 89b2b2
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish11_withering_ganon_bat_flame(&mut self, k: usize) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0 {
            let y = self.garnish_get_y(k).wrapping_sub(1);
            self.garnish_set_y(k, y);
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_plain_at(oam, pt.x as u8, pt.y as u8, 0xa4, 0x22, 0);
        self.set_oam_plain_at(
            oam + 4,
            pt.x.wrapping_add(8) as u8,
            pt.y as u8,
            0xa5,
            0x22,
            0,
        );
    }

    // void Garnish10_GanonBatFlame(int k) {  // 89b306
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish10_ganon_bat_flame(&mut self, k: usize) {
        if self.garnish_slot_view(k).countdown() == 8 {
            let value = 0x11;
            self.garnish_slot_view_mut(k).set_garnish_type(value);
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_IDX
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
        );
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_CHAR[j],
            GARNISH10_GANON_BAT_FLAME_GANON_BAT_FLAME_FLAGS[j] | 0x22,
            2,
        );
        // The ROM reaches the collision check with the carry left by the
        // `LSR A` x3 that indexed the frame table: bit 2 of the countdown.
        let carry_in = self.garnish_slot_view(k).countdown() & 0x04 != 0;
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y), carry_in);
    }

    // void Garnish0A_CannonSmoke(int k) {  // 89b3ee
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish0_a_cannon_smoke(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
            GARNISH0_A_CANNON_SMOKE_GARNISH_CANNON_POOF_FLAGS[j] | 4,
            2,
        );
    }

    // void Garnish0C_TrinexxIceBreath(int k) {  // 89b34f
    pub(in crate::zelda_rtl) fn garnish0_c_trinexx_ice_breath(&mut self, k: usize) {
        if self.garnish_slot_view(k).countdown() == 0x50
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
        {
            self.dungeon_update_tile_map_with_common_tile(
                self.garnish_get_x(k),
                self.garnish_get_y(k).wrapping_sub(16),
                18,
            );
        }
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 4)],
            GARNISH0_C_TRINEXX_ICE_BREATH_TRINEXX_ICE_FLAGS
                [usize::from((self.garnish_slot_view(k).countdown() >> 2) & 3)]
                | 0x35,
            2,
        );
    }

    // void Garnish09_LightningTrail(int k) {  // 89b429
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish09_lightning_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        let room_offset = if self.game_state.dungeon.room_tracking.room_index2() == 0x20 {
            0x80
        } else {
            0
        };
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_CHAR[j].wrapping_sub(room_offset),
            (self.game_state.frame.frame_counter << 1) & 0x0e
                | GARNISH09_LIGHTNING_TRAIL_LIGHTNING_TRAIL_FLAGS[j],
            2,
        );
        // The ROM reaches the collision check with the carry left by the
        // `ASL A` on the frame counter: its bit 7.
        let carry_in = self.game_state.frame.frame_counter & 0x80 != 0;
        self.garnish_check_player_collision(k, i32::from(pt.x), i32::from(pt.y), carry_in);
    }

    // void Garnish03_FallingTile(int k) {  // 89b627
    pub(in crate::zelda_rtl) fn garnish03_falling_tile(&mut self, k: usize) {
        let mut j = self.garnish_slot_view(k).countdown();
        if j == 0x1e {
            j = self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag;
            if j == 0 {
                self.dungeon_update_tile_map_with_common_tile(
                    self.garnish_get_x(k),
                    self.garnish_get_y(k).wrapping_sub(16),
                    4,
                );
            }
        }
        let j = usize::from(j >> 3);
        let x = self
            .garnish_get_x(k)
            .wrapping_add(u16::from(GARNISH03_FALLING_TILE_CRUMBLE_TILE_XY[j]))
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .garnish_get_y(k)
            .wrapping_add(u16::from(GARNISH03_FALLING_TILE_CRUMBLE_TILE_XY[j]))
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x < 256 && y < 256 {
            self.set_oam_plain_at(
                self.game_state.oam.current_pointer_usize(),
                x as u8,
                y.wrapping_sub(16) as u8,
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_CHAR[j],
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_FLAGS[j],
                GARNISH03_FALLING_TILE_CRUMBLE_TILE_EXT[j],
            );
        }
    }

    // void Garnish07_BabasuFlash(int k) {  // 89b49e
    pub(in crate::zelda_rtl) fn garnish07_babasu_flash(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).countdown() >> 3);
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH07_BABASU_FLASH_BABUSU_FLASH_CHAR[j],
            GARNISH07_BABASU_FLASH_BABUSU_FLASH_FLAGS[j],
            2,
        );
    }

    // void Garnish08_KholdstareTrail(int k) {  // 89b4c6
    pub(in crate::zelda_rtl) fn garnish08_kholdstare_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let i = usize::from(self.garnish_slot_view(k).countdown() >> 2);
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x.wrapping_add(GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            pt.y.wrapping_add(GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_XY[i] as i16 as u16) as u8,
            GARNISH08_KHOLDSTARE_TRAIL_GARNISH_NEBULE_CHAR[i],
            (self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority())
                & !1,
            0,
        );
    }

    // void Garnish0E_TrinexxFireBreath(int k) {  // 89b55d
    pub(in crate::zelda_rtl) fn garnish0_e_trinexx_fire_breath(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_E_TRINEXX_FIRE_BREATH_TRINEXX_LAVA_BUBBLE_CHAR
                [usize::from(self.garnish_slot_view(k).countdown() >> 3)],
            (self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority())
                & 0xf0
                | 0x0e,
            0,
        );
    }

    // void Garnish0F_BlindLaserTrail(int k) {  // 89b591
    pub(in crate::zelda_rtl) fn garnish0_f_blind_laser_trail(&mut self, k: usize) {
        let mut pt = Point16U { x: 0, y: 0 };
        if self.garnish_return_if_prep_fails(k, &mut pt) {
            return;
        }
        let j = usize::from(self.garnish_slot_view(k).sprite());
        self.set_oam_plain_at(
            self.game_state.oam.current_pointer_usize(),
            pt.x as u8,
            pt.y as u8,
            GARNISH0_F_BLIND_LASER_TRAIL_BLIND_LASER_TRAIL_CHAR
                [usize::from(self.garnish_slot_view(k).oam_flags().wrapping_sub(7))],
            self.sprite_slot_view(j).oam_flags() | self.sprite_slot_view(j).object_priority(),
            0,
        );
    }

    // void Garnish_ExecuteUpperSlots() {  // 89b08c
    //   HandleScreenFlash();
    //
    //   if (garnish_active) {
    //     for (int i = 29; i >= 15; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(in crate::zelda_rtl) fn garnish_execute_upper_slots(&mut self) {
        // Cycle ledger: this is Garnish_ExecuteUpperSlots_ $09:B06E (m8 x8,
        // the JSL target Sprite_Main calls): JSL HandleScreenFlash, LDA $0fb4,
        // BEQ (110); an active garnish table runs $09:B077-B07D PHB PHK PLB
        // JSR Garnish_ExecuteUpperSlots PLB (146) around the JSR target
        // $09:B08C (LDX #$1d 16, fifteen passes of JSR Garnish_ExecuteSingle
        // DEX CPX #$0e BNE 92 with the BNE taken fourteen times, RTS 42:
        // 1,522), else the BEQ is taken (+6); then $09:B07E RTL (44).
        let _scope = crate::cycle_ledger::routine(0x09_b06e);
        crate::cycle_ledger::charge(62);
        self.handle_screen_flash();
        self.garnish_execute_upper_after_flash();
    }

    fn garnish_execute_upper_after_flash(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x09_b06e);
        crate::cycle_ledger::charge(48); // LDA active : BEQ after the actual flash return.
        if self.game_state.sprites.garnish_runtime.active_type() != 0 {
            crate::cycle_ledger::charge(146);
            let _slots = crate::cycle_ledger::routine(0x09_b08c);
            crate::cycle_ledger::charge(16);
            for i in (15..=29).rev() {
                crate::cycle_ledger::charge(if i > 15 { 92 + 6 } else { 92 });
                self.garnish_execute_single(i);
            }
            crate::cycle_ledger::charge(42);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(44);
    }

    pub(super) fn finish_native_garnish_caller(
        &mut self,
        tail: crate::cycle_models::dungeon_garnish::GarnishTail,
    ) {
        use crate::cycle_models::dungeon_garnish::GarnishTail;
        match tail {
            GarnishTail::Returned => {}
            GarnishTail::PaletteRestore
            | GarnishTail::PaletteWhiten
            | GarnishTail::PaletteFromFlash => {
                self.finish_native_screen_flash(tail);
                self.garnish_execute_upper_after_flash();
            }
            GarnishTail::Slot { slot, kind } => {
                let _outer = crate::cycle_ledger::routine(0x09_b06e);
                let _loop = crate::cycle_ledger::routine(0x09_b08c);
                {
                    let _single = crate::cycle_ledger::routine(0x09_b0b6);
                    self.garnish_execute_single_after_type(usize::from(slot), kind);
                }
                crate::cycle_ledger::charge(46 + if slot > 15 { 6 } else { 0 }); // DEX : CPX : BNE after the selected body.
                for k in (15..usize::from(slot)).rev() {
                    crate::cycle_ledger::charge(if k > 15 { 98 } else { 92 });
                    self.garnish_execute_single(k);
                }
                crate::cycle_ledger::charge(42 + 28 + 44); // Real loop RTS, restored DB, outer RTL.
            }
        }
    }

    // void Garnish_ExecuteLowerSlots() {  // 89b097
    //   if (garnish_active) {
    //     for (int i = 14; i >= 0; i--)
    //       Garnish_ExecuteSingle(i);
    //   }
    // }
    pub(in crate::zelda_rtl) fn garnish_execute_lower_slots(&mut self) {
        // Cycle ledger: Garnish_ExecuteLowerSlots_ $09:B07F (m8 x8, the JSL
        // target): LDA $0fb4 : BEQ (48); an active table runs $09:B084-B08A
        // PHB PHK PLB JSR Garnish_ExecuteLowerSlots PLB (146) around the JSR
        // target $09:B097 (LDX #$0e 16, fifteen passes of JSR
        // Garnish_ExecuteSingle DEX BPL 76 with the BPL taken fourteen times,
        // RTS 42: 1,282), else the BEQ is taken (+6); then $09:B08B RTL (44).
        let _scope = crate::cycle_ledger::routine(0x09_b07f);
        crate::cycle_ledger::charge(48);
        if self.game_state.sprites.garnish_runtime.active_type() != 0 {
            crate::cycle_ledger::charge(146);
            let _slots = crate::cycle_ledger::routine(0x09_b097);
            crate::cycle_ledger::charge(16);
            for i in (0..=14).rev() {
                crate::cycle_ledger::charge(if i > 0 { 76 + 6 } else { 76 });
                self.garnish_execute_single(i);
            }
            crate::cycle_ledger::charge(42);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(44);
    }

    // void Garnish_ExecuteSingle(int k) {  // 89b0b6
    //   ...see sprite.c...
    // }
    pub(in crate::zelda_rtl) fn garnish_execute_single(&mut self, k: usize) {
        // Cycle ledger: Garnish_ExecuteSingle $09:B0B6 (m8 x8, JSR target;
        // its RTS dispatch runs the handler inside this scope).
        let _scope = crate::cycle_ledger::routine(0x09_b0b6);
        self.sprite_system_mut().set_cur_object_index(k as u8);
        let type_ = self.garnish_slot_view(k).garnish_type();
        // $09:B0B6-B0BD STX $0fa0 LDA long $7ff800,x BEQ (88): an empty slot
        // takes the BEQ (+6) to $09:B123 RTS (42).
        if type_ == 0 {
            crate::cycle_ledger::charge(88 + 6 + 42);
            return;
        }
        crate::cycle_ledger::charge(88);
        self.garnish_execute_single_after_type(k,type_);
    }

    fn garnish_execute_single_after_type(&mut self, k: usize, type_: u8) {
        assert_ne!(type_, 0);
        // $09:B0BF-B0C1 CMP #$05 : BEQ (32): type 5 takes it (+6) to the
        // countdown test; else $09:B0C3-B0C8 LDA $11 ORA $0fc1 BNE (72),
        // taken (+6) past the countdown when a submodule or pause is active.
        let counts_down = type_ == 5
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0;
        crate::cycle_ledger::charge(if type_ == 5 {
            32 + 6
        } else if counts_down {
            32 + 72
        } else {
            32 + 72 + 6
        });
        if counts_down {
            // $09:B0CA-B0CE LDA long $7ff90e,x : BEQ (56), taken (+6) for a
            // zero countdown; else $09:B0D0-B0D5 DEC STA long BNE (70),
            // taken (+6) while nonzero, else $09:B0D7-B0DB STA long : BRA
            // (62) into the RTS (42).
            crate::cycle_ledger::charge(if self.garnish_slot_view(k).countdown() == 0 {
                56 + 6
            } else if self.garnish_slot_view(k).countdown() == 1 {
                56 + 70 + 62 + 42
            } else {
                56 + 70 + 6
            });
        }
        if counts_down && self.garnish_slot_view(k).countdown() != 0 {
            let value = self.garnish_slot_view(k).countdown().wrapping_sub(1);
            self.garnish_slot_view_mut(k).set_countdown(value);
            if self.garnish_slot_view(k).countdown() == 0 {
                let value = 0;
                self.garnish_slot_view_mut(k).set_garnish_type(value);
                return;
            }
        }

        let sprsize = GARNISH_EXECUTE_SINGLE_GARNISH_OAM_MEM_SIZE[usize::from(type_)];
        // $09:B0DD-B0E0 LDY $0fb3 : BEQ (48): sorting off takes it (+6) into
        // $09:B104-B10C LDA TAY LDA $b09f,y JSL RegionA (148); sorting on
        // runs $09:B0E2-B0E6 LDA long floor : BEQ (56) then $09:B0E8 (170,
        // RegionF + BRA) or the taken BEQ (+6) into $09:B0F6 (170, RegionD +
        // BRA). Then the dispatch $09:B110-B121 (240) and its RTS (42).
        if self.game_state.oam.has_sprite_sorting() {
            if self.garnish_slot_view(k).floor() != 0 {
                crate::cycle_ledger::charge(48 + 56 + 170);
                self.oam_allocate_from_region_f(sprsize);
            } else {
                crate::cycle_ledger::charge(48 + 56 + 6 + 170);
                self.oam_allocate_from_region_d(sprsize);
            }
        } else {
            crate::cycle_ledger::charge(48 + 6 + 148);
            self.oam_allocate_from_region_a(sprsize);
        }
        crate::cycle_ledger::charge(240 + 42);
        self.garnish_execute_body(k,type_);
    }

    fn garnish_execute_body(&mut self,k:usize,type_:u8) {
        match type_ {
            1 => self.garnish01_fire_snake_tail(k),
            2 => self.garnish02_mothula_beam_trail(k),
            3 => self.garnish03_falling_tile(k),
            4 => self.garnish04_laser_trail(k),
            5 => self.garnish_simple_sparkle(k),
            6 => self.garnish06_zoro_trail(k),
            7 => self.garnish07_babasu_flash(k),
            8 => self.garnish08_kholdstare_trail(k),
            9 => self.garnish09_lightning_trail(k),
            10 => self.garnish0_a_cannon_smoke(k),
            11 => self.garnish_water_trail(k),
            12 => self.garnish0_c_trinexx_ice_breath(k),
            13 => {}
            14 => self.garnish0_e_trinexx_fire_breath(k),
            15 => self.garnish0_f_blind_laser_trail(k),
            16 => self.garnish10_ganon_bat_flame(k),
            17 => self.garnish11_withering_ganon_bat_flame(k),
            18 => self.garnish12_sparkle(k),
            19 => self.garnish13_pyramid_debris(k),
            20 => self.garnish14_kak_kid_dash_dust(k),
            21 => self.garnish15_arrghus_splash(k),
            22 => self.garnish16_thrown_item_debris(k),
            _ => {}
        }
    }

    // int Sprite_SpawnSimpleSparkleGarnishEx(int k, uint16 x, uint16 y, int limit) {
    //   int j = GarnishAllocLimit(limit);
    //   if (j >= 0) {
    //     garnish_type[j] = 5;
    //     garnish_active = 5;
    //     Garnish_SetX(j, Sprite_GetX(k) + x);
    //     Garnish_SetY(j, Sprite_GetY(k) + y - sprite_z[k] + 16);
    //     garnish_countdown[j] = 31;
    //     garnish_sprite[j] = k;
    //     garnish_floor[j] = sprite_floor[k];
    //   }
    //   g_ram[15] = j;
    //   return j;
    // }
    pub(in crate::zelda_rtl) fn sprite_spawn_simple_sparkle_garnish_ex(
        &mut self,
        k: usize,
        x: u16,
        y: u16,
        limit: i32,
    ) -> i32 {
        let j = self.garnish_alloc_limit(limit as usize);
        if j >= 0 {
            let j = j as usize;
            let value = 5;
            self.garnish_slot_view_mut(j).set_garnish_type(value);
            self.garnish_state_mut().set_active_type(5);
            self.garnish_set_x(j, self.sprite_get_x(k).wrapping_add(x));
            self.garnish_set_y(
                j,
                self.sprite_get_y(k)
                    .wrapping_add(y)
                    .wrapping_sub(self.sprite_slot_view(k).z() as u16)
                    .wrapping_add(16),
            );
            let value = 31;
            self.garnish_slot_view_mut(j).set_countdown(value);
            let value = k as u8;
            self.garnish_slot_view_mut(j).set_sprite(value);
            let value = self.sprite_slot_view(k).floor();
            self.garnish_slot_view_mut(j).set_floor(value);
        }
        // The original parks the slot index in R15, the high byte of the
        // collision word; nothing reads it back.
        self.tile_detect_position_mut().set_last_garnish_index(j as u8);
        j
    }

    // void Sprite_GarnishSpawn_Sparkle_limited(int k, uint16 x, uint16 y) {  // 9ea001
    //   Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 14);
    // }
    pub(in crate::zelda_rtl) fn sprite_garnish_spawn_sparkle_limited(&mut self, k: usize, x: u16, y: u16) {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 14);
    }

    // int Sprite_GarnishSpawn_Sparkle(int k, uint16 x, uint16 y) {  // 9ea007
    //   return Sprite_SpawnSimpleSparkleGarnishEx(k, x, y, 29);
    // }
    pub(in crate::zelda_rtl) fn sprite_garnish_spawn_sparkle(&mut self, k: usize, x: u16, y: u16) -> i32 {
        self.sprite_spawn_simple_sparkle_garnish_ex(k, x, y, 29)
    }
    pub(super) fn finish_native_lower_garnish(&mut self,tail:crate::cycle_models::dungeon_lower_garnish::LowerGarnishTail) {
        use crate::cycle_models::dungeon_lower_garnish::LowerGarnishTail;
        match tail {
            LowerGarnishTail::Returned=>{},
            LowerGarnishTail::Body {slot,kind,..}=>{
                let _outer=crate::cycle_ledger::routine(0x09_b07f);
                let _slots=crate::cycle_ledger::routine(0x09_b097);
                {let _single=crate::cycle_ledger::routine(0x09_b0b6);self.garnish_execute_body(usize::from(slot),kind);}
                crate::cycle_ledger::charge(if slot==0 {30}else{36});
                for k in (0..usize::from(slot)).rev() {
                    crate::cycle_ledger::charge(if k==0 {76}else{82});
                    self.garnish_execute_single(k);
                }
                crate::cycle_ledger::charge(42+28+44);
            },
        }
    }
}
