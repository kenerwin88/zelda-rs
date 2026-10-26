//! Beamos, lasers, traps, antifairies, firebars, and other moving hazards.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;
use crate::types::sign8;
use crate::types::SpriteHitBox;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn beamos_laser_draw_marked(&mut self, k: usize) {
        // $05:925B..92C5. The coordinate helper owns its own JSL body.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x05925b));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x05925b); b.block(0x05925e);
        for i in (0..32).rev() {
            let j = usize::from(self.sprite_slot_view(k).graphics()) * 32 + i;
            let y = self.game_state.effects.sprite_histories.beamos_laser_history(j).y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            b.block(0x05926d);
            if y.wrapping_add(16) < 256 { b.taken(0x0592a4); } else { b.block(0x0592a6); }
            b.block(0x0592aa);
            if i != 0 { b.taken(0x0592c2); }
        }
        b.block(0x0592c4);
        self.beamos_laser_draw(k);
    }



    // -----------------------------------------------------------------------
    // void SpriteDraw_Beamos_Eyeball(int k, PrepOamCoordsRet *info) {  // 859151
    //   sprite_main.c:2509 — 32-entry eyeball cycle drawn over a base offset.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_beamos_eyeball(&mut self, k: usize, info: &PrepOamCoordsRet) {
        let n: usize = if self.sprite_slot_view(k).direction() < 0x20 {
            0
        } else {
            2
        };
        let oam = (self.game_state.oam.current_pointer_usize()) + n * 4;
        let i = (self.sprite_slot_view(k).direction() >> 1) as usize;
        let dx = BEAMOS_EYEBALL_DRAW_X_OFFSETS[i].wrapping_sub(3);
        let dy = BEAMOS_EYEBALL_DRAW_Y_OFFSETS[i].wrapping_sub(18);
        self.draw_scratch_position_mut()
            .set_word_bytes(dx as u8, dy as u8);
        // oam.x = dx + info.x (low byte only — `SetOamHelper0` style write).
        let x = info.x.wrapping_add(dx as i16 as u16);
        let y = info.y.wrapping_add(dy as i16 as u16);
        // Mimic the C — write only oam.x/oam.y/oam.charnum/oam.flags. We
        // use set_oam_helper0_at to get the OAM-low/high split, then patch
        // the flags afterwards to match the C bit ops.
        self.set_oam_helper0_at(
            oam,
            x,
            y,
            BEAMOS_EYEBALL_DRAW_CHARS[i],
            (info.flags & 0x31) | 0xA | BEAMOS_EYEBALL_DRAW_FLAGS[i],
            2,
        );
        // oam_cur_ptr += n * 4; oam_ext_cur_ptr += n;
        self.oam_state_mut().add_current_pointer((n as u16) * 4);
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(n as u16));
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Antfairy(int k) {  // 9df395 — sprite_main.c:18841
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_antfairy(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_draw_antfairy_after_subtype2_increment(k);
    }

    fn sprite_draw_antfairy_after_subtype2_increment(&mut self, k: usize) {
        if (self.sprite_slot_view(k).subtype2() & 1)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag
            == 0
        {
            self.sprite_slot_view_mut(k).add_graphics(1);
            if self.sprite_slot_view(k).graphics() == 6 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
        }
        let base = (self.sprite_slot_view(k).graphics() as usize) * 5;
        self.sprite_draw_multiple(k, &DRAW_FOUR_AROUND_ONE_DRAW_FRAMES[base..base + 5]);
    }

    pub(in crate::zelda_rtl) fn complete_antfairy_after_subtype2_increment(
        &mut self,
        k: usize,
        continuation: AntfairyDrawContinuation,
    ) {
        self.sprite_draw_antfairy_after_subtype2_increment(k);
        match continuation {
            AntfairyDrawContinuation::BunnyBeam => self.sprite_bunny_beam_after_antfairy_draw(k),
            AntfairyDrawContinuation::Antifairy => self.sprite_15_antifairy_after_draw(k),
            AntfairyDrawContinuation::AntifairyCircle => {
                self.sprite_82_antifairy_circle_after_draw(k)
            }
        }
    }

    pub(in crate::zelda_rtl) fn antfairy_draw_continuation(&self, k: usize) -> AntfairyDrawContinuation {
        match self.sprite_slot_view(k).sprite_type() {
            0xd1 => {
                assert!(self.game_state.world.location.is_indoors());
                assert_ne!(
                    self.sprite_slot_view(k).ai_state(),
                    0,
                    "Bunny Beam Antfairy checkpoint requires its draw branch",
                );
                AntfairyDrawContinuation::BunnyBeam
            }
            0x15 => AntfairyDrawContinuation::Antifairy,
            0x82 => AntfairyDrawContinuation::AntifairyCircle,
            sprite_type => panic!(
                "Antfairy subtype checkpoint reached incompatible sprite type {sprite_type:#04x}",
            ),
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_61_Beamos(int k) {  // 858f54
    pub(super) fn sprite_61_beamos(&mut self, k: usize) {
        if self.sprite_slot_view(k).c() == 1 {
            self.sprite_beamos_laser(k);
            return;
        } else if self.sprite_slot_view(k).c() != 0 {
            self.sprite_beamos_laser_hit(k);
            return;
        }

        self.beamos_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:921E -> $05:B88D, $05:8F69 -> $05:B88D -> $06:E496
        let _ = self.sprite_check_tile_collision_long(k);
        // $05:8F6C JSL $06:F121: complete body, caller opcode unchanged.
        self.sprite_check_damage_to_link_long(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    self.sprite_spawn_probe_always(k, self.sprite_slot_view(k).direction());
                    self.sprite_slot_view_mut(k).add_direction(1);
                }
                self.sprite_slot_view_mut(k).and_direction(63);
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_prep_load_palette(k);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 15 {
                        self.beamos_fire_laser(k);
                    }
                    let value = (self.sprite_slot_view(k).delay_main() >> 1) & 0x0e;
                    self.sprite_slot_view_mut(k).xor_oam_flags(value);
                }
            }
            _ => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_SpawnProbeAlways(int k, uint8 r15) {  // 85c612
    pub(in crate::zelda_rtl) fn sprite_spawn_probe_always(&mut self, k: usize, r15: u8) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x41, &mut info, 10);
        if j < 0 {
            return;
        }
        let j = j as usize;
        let t = info.r0_x.wrapping_add(8);
        let value = t as u8;
        self.sprite_slot_view_mut(j).set_x_low(value);
        let value = (t >> 8) as u8;
        self.sprite_slot_view_mut(j).set_x_high(value);
        let t = info.r2_y.wrapping_add(4);
        let value = t as u8;
        self.sprite_slot_view_mut(j).set_y_low(value);
        let value = (t >> 8) as u8;
        self.sprite_slot_view_mut(j).set_y_high(value);
        let value = r15;
        self.sprite_slot_view_mut(j).set_direction(value);
        let value = SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_X_VELOCITIES[usize::from(r15)] as u8;
        self.sprite_slot_view_mut(j).set_x_velocity(value);
        let value = SPRITE_SPAWN_PROBE_ALWAYS_LOCAL_Y_VELOCITIES[usize::from(r15)] as u8;
        self.sprite_slot_view_mut(j).set_y_velocity(value);
        let value = (self.sprite_slot_view(j).flags2() & 0xf0) | 0xa0;
        self.sprite_slot_view_mut(j).set_flags2(value);
        let value = (k as u8).wrapping_add(1);
        self.sprite_slot_view_mut(j).set_c(value);
        let value = (k as u8).wrapping_add(1);
        self.sprite_slot_view_mut(j).set_ignore_projectile(value);
        let value = 0x40;
        self.sprite_slot_view_mut(j).set_flags4(value);
        let value = 0x40;
        self.sprite_slot_view_mut(j).set_flags3(value);
        let value = 2;
        self.sprite_slot_view_mut(j).set_deflection_bits(value);
    }

    // -----------------------------------------------------------------------
    // void Beamos_Draw(int k) {  // 859068
    //   Draw the two 16x16 body tiles, then overlay the rotating eyeball.
    // }
    pub(super) fn beamos_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet { x, y, r4: 0, flags };
        let spr_offs = if self.sprite_slot_view(k).direction() < 0x20 {
            self.oam_allocate_from_region_b(12);
            1
        } else {
            self.oam_allocate_from_region_c(12);
            0
        };
        let mut oam = self.game_state.oam.current_pointer_usize() + spr_offs * 4;
        for i in (0..2).rev() {
            self.set_oam_helper0_at(
                oam,
                info.x,
                info.y.wrapping_add(BEAMOS_DRAW_Y_OFFSETS[i] as i16 as u16),
                BEAMOS_DRAW_CHARS[i],
                info.flags,
                2,
            );
            oam += 4;
        }
        self.sprite_draw_beamos_eyeball(k, &info);
    }

    // -----------------------------------------------------------------------
    // void BeamosLaser_Draw(int k) {  // 85925b
    //   Replay the 32-sample beamos history buffer as small laser tiles.
    // }
    pub(super) fn beamos_laser_draw(&mut self, k: usize) {
        let info = self.sprite_prep_oam_coord_from(k, super::sprite::PrepOamCoordEntry::Bank5Safe);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        for i in (0..32).rev() {
            let j = g * 32 + i;
            let history = self
                .game_state
                .effects
                .sprite_histories
                .beamos_laser_history(j);
            let x = history
                .x()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = history
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.set_oam_helper0_at(oam, x, y, 0x5c, info.flags, 0);
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Beamos_Laser(int k) {  // 8591b5
    pub(super) fn sprite_beamos_laser(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        // ROM $05:91BA
        self.beamos_laser_draw_marked(k);
        if self.sprite_slot_view(k).state() == 0 {
            self.sprite_system_mut().decrement_limit_instance();
            return;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        for _ in (0..4).rev() {
            let t = usize::from(self.sprite_slot_view(k).subtype2() & 31)
                + usize::from(self.sprite_slot_view(k).graphics()) * 32;
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let x = self.sprite_slot_view(k).x();
            let y = self.sprite_slot_view(k).y();
            self.beamos_laser_history_mut(t).set_position(x, y);
            // ROM $05:9202
            self.sprite_move_xy_marked(k, 0x05fa00);
        }

        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_system_mut().decrement_limit_instance();
            }
            return;
        }
        // ROM $05:9218
        if !self.sprite_check_damage_same_layer_long_marked(k)
            // ROM $05:921E -> $05:B88D -> $06:E496
            && self.sprite_check_tile_collision_long(k) == 0
        {
            return;
        }
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
        let value = 16;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        self.sprite_zero_velocity_xy(k);
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x61, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            let value = 16;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 2;
            self.sprite_slot_view_mut(j).set_c(value);
            let value = 0x40;
            self.sprite_slot_view_mut(j).set_flags3(value);
        }
        let value = 128;
        self.sprite_slot_view_mut(k).set_y_high(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Beamos_LaserHit(int k) {  // 8592da
    pub(super) fn sprite_beamos_laser_hit(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            self.set_oam_helper0_at(
                oam,
                info_x.wrapping_add(SPRITE_BEAMOS_LASER_HIT_X_OFFSETS[i] as i16 as u16),
                info_y.wrapping_add(SPRITE_BEAMOS_LASER_HIT_Y_OFFSETS[i] as i16 as u16),
                0xd6,
                SPRITE_BEAMOS_LASER_HIT_FLAGS[i] | (info_flags & 0x30),
                0,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Landmine_Draw(int k) {  // 9d810c
    //   Two 8x8 tiles unless the sprite engine is in the high-slot phase.
    // }
    pub(super) fn landmine_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(8);
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            return;
        }
        self.sprite_draw_multiple(k, &LANDMINE_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void Pengator_Draw(int k) {  // 9ea415
    //   Two body tiles, optional foot tiles, then common shadow.
    // }
    pub(in crate::zelda_rtl) fn pengator_draw(&mut self, k: usize) {
        let gfx = self.sprite_slot_view(k).graphics() as usize;
        let base = (gfx * 2).min(PENGATOR_DRAW_FRAMES0.len() - 2);
        let mut info = self.sprite_draw_multiple(k, &PENGATOR_DRAW_FRAMES0[base..base + 2]);
        let extra = if gfx == 14 {
            Some(0usize)
        } else if gfx == 19 {
            Some(2usize)
        } else {
            None
        };
        if let Some(extra_base) = extra {
            let cur = self.game_state.oam.current_pointer();
            self.oam_state_mut()
                .set_current_pointer(cur.wrapping_add(8));
            let ext = self.game_state.oam.current_extended_pointer();
            self.oam_state_mut()
                .set_current_extended_pointer(ext.wrapping_add(2));
            info = self.sprite_draw_multiple(
                k,
                &PENGATOR_DRAW_FRAMES1[extra_base..extra_base + 2],
            );
        }
        // ROM $1E:A45D
        self.sprite_draw_shadow_long_marked(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SpikeRoller_Draw(int k) {  // 858ee3
    //   Four or eight repeated roller tiles selected by sprite_graphics.
    // }
    pub(super) fn spike_roller_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        let mut chr = SPIKE_ROLLER_DRAW_CHARS[(g * 8).min(SPIKE_ROLLER_DRAW_CHARS.len() - 1)];
        let max_i = if self.sprite_slot_view(k).ai_state() != 0 {
            7
        } else {
            3
        };
        for i in (0..=max_i).rev() {
            let j = (g * 8 + i).min(SPIKE_ROLLER_DRAW_CHARS.len() - 1);
            let charnum = if chr != 0 {
                chr
            } else {
                SPIKE_ROLLER_DRAW_CHARS[j]
            };
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(SPIKE_ROLLER_DRAW_X_OFFSETS[j] as u16),
                y.wrapping_add(SPIKE_ROLLER_DRAW_Y_OFFSETS[j] as u16),
                charnum,
                SPIKE_ROLLER_DRAW_FLAGS[j] | flags,
                2,
            );
            chr = 0;
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_93_Bumper(int k) {  // 9ea982
    pub(super) fn sprite_93_bumper(&mut self, k: usize) {
        self.bumper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $1E:A988 -> $1E:8B2E -> $06:E496
        self.sprite_check_tile_collision_bank1e_marked(k);
        if !self.game_state.player.follower_link.is_cape_active()
            // ROM $1E:A98F
            && self.sprite_check_damage_same_layer_long_marked(k)
        {
            self.link_cancel_dash();
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let pt = self.sprite_project_speed_towards_link(k, 0x30);
            let y_velocity = pt.y.wrapping_add(
                SPRITE_93_BUMPER_VELOCITIES
                    [usize::from((self.game_state.player.follower_link.joypad1h_last() >> 2) & 3)]
                    as u8,
            );
            let x_velocity = pt.x.wrapping_add(
                SPRITE_93_BUMPER_VELOCITIES
                    [usize::from(self.game_state.player.follower_link.joypad1h_last() & 3)]
                    as u8,
            );
            self.follower_link_state_mut()
                .set_actual_velocity_xy(x_velocity, y_velocity);
            self.follower_link_state_mut().set_incapacitated_timer(20);
            self.link_reset_swimming_state();
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
        }
        for j in (0..16).rev() {
            if ((((j as u8) ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(j).z())
                != 0
            {
                continue;
            }
            if self.sprite_slot_view(j).state() < 9
                || ((self.sprite_slot_view(j).flags3() | self.sprite_slot_view(j).flags4()) & 0x40)
                    != 0
            {
                continue;
            }
            let x = self.sprite_get_x(j);
            let y = self.sprite_get_y(j);
            let dx = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_sub(x)
                .wrapping_add(16);
            let dy = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_sub(y)
                .wrapping_add(16);
            if dx < 32 && dy < 32 {
                let value = 15;
                self.sprite_slot_view_mut(j).set_f(value);
                // ROM $1E:AA2B
                let pt = self.sprite_project_speed_towards_location_long_marked(k, x, y, 0x40);
                let value = pt.y;
                self.sprite_slot_view_mut(j).set_y_recoil(value);
                let value = pt.x;
                self.sprite_slot_view_mut(j).set_x_recoil(value);
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Bumper_Draw(int k) {  // 9eaa8b
    //   Four large tiles toggled by delay timer.
    // }
    pub(super) fn bumper_draw(&mut self, k: usize) {
        let base = (((self.sprite_slot_view(k).delay_main() >> 1) & 1) as usize) * 4;
        self.sprite_draw_multiple(k, &BUMPER_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D4_Landmine(int k) {  // 9d8099
    pub(super) fn sprite_d4_landmine(&mut self, k: usize) {
        self.landmine_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if !self.landmine_check_detonation_from_hammer(k) {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 4;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                // $1D:80AF JSL $06:F121: complete body, caller opcode unchanged.
                if self.sprite_check_damage_to_link_long(k) {
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                return;
            }
            if self.sprite_slot_view(k).delay_main() != 1 {
                let idx = usize::from((self.sprite_slot_view(k).delay_main() >> 1) & 3);
                let value = SPRITE_D4_LANDMINE_OAM_FLAGS[idx];
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                return;
            }
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        let j = self.sprite_spawn_bomb(k);
        if j >= 0 {
            let j = j as usize;
            let value = 6;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 2;
            self.sprite_slot_view_mut(j).set_c(value);
            let value = 2;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 9;
            self.sprite_slot_view_mut(j).set_flags4(value);
            let value = 31;
            self.sprite_slot_view_mut(j).set_delay_aux1(value);
            let value = 3;
            self.sprite_slot_view_mut(j).set_flags2(value);
            self.set_sound_effect_1_with_sprite_pan(k, 12);
        }
    }

    // -----------------------------------------------------------------------
    // bool Landmine_CheckDetonationFromHammer(int k) {  // 84ea81
    pub(super) fn landmine_check_detonation_from_hammer(&mut self, k: usize) -> bool {
        if !self.game_state.player.follower_link.item_in_hand_has(10)
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
        {
            return false;
        }
        let mut hb = SpriteHitBox {
            r0_xlo: 0,
            r8_xhi: 0,
            r1_ylo: 0,
            r9_yhi: 0,
            r2: 0,
            r3: 0,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.player_setup_action_hit_box(&mut hb);
        self.sprite_setup_hit_box(k, &mut hb);
        self.check_if_hit_boxes_overlap(&hb)
    }

    // -----------------------------------------------------------------------
    // void Sprite_D1_BunnyBeam(int k) {  // 9d858b
    pub(super) fn sprite_d1_bunny_beam(&mut self, k: usize) {
        if self.game_state.world.location.is_indoors() {
            self.sprite_bunny_beam(k);
        } else {
            self.sprite_chimney(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BunnyBeam(int k) {  // 9d85e0
    pub(super) fn sprite_bunny_beam(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_prep_oam_coord(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            // ROM $1D:85EC -> $1D:8094, $1D:8657 -> $1D:8094 -> $06:E496
            if self.sprite_check_tile_collision_long(k) == 0 {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = 128;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            return;
        }

        self.sprite_draw_antfairy(k);
        self.sprite_bunny_beam_after_antfairy_draw(k);
    }

    fn sprite_bunny_beam_after_antfairy_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).pause() == 0 {
            let oam = self.game_state.oam.current_pointer_usize();
            let charnum = RABBIT_BEAM_GFX[self.sprite_slot_view(k).graphics() as usize];
            for i in 0..5 {
                let pos = oam + i * 4;
                self.oam_state_mut().set_entry_char(pos, charnum);
                self.oam_state_mut().merge_entry_flags(pos, 0xf0, 2);
            }
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0x30;
            self.sprite_slot_view_mut(k).set_bump_damage(value);
            // $1D:8634 JSL $06:F121: complete body, caller opcode unchanged.
            if self.sprite_check_damage_to_link_long(k) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().set_temp_bunny_timer(256);
            }
            if self.game_state.player.follower_link.lower_level_state()
                == self.sprite_slot_view(k).floor()
            {
                self.sprite_apply_speed_towards_link(k, 16);
            }
            self.sprite_move_xy_bank(k, 0x1de95d);
            if self.sprite_check_tile_collision(k) != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_spawn_poof_garnish(k);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void FireBat_Draw(int k) {  // 9d8ca9
    //   Two large tiles sharing one animation char with per-frame flips.
    // }
    pub(in crate::zelda_rtl) fn fire_bat_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1dDoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        for i in (0..2).rev() {
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(FIREBAT_DRAW_X_OFFSETS[i] as i16 as u16),
                y,
                FIREBAT_DRAW_CHARS[g],
                FIREBAT_DRAW_FLAGS[g * 2 + i] | flags,
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void FirePhlegm_Draw(int k) {  // 9e9443
    //   Sprite_DrawMultiple(k, &kFirePhlegm_Dmd[sprite_D[k] * 4 +
    //                       sprite_graphics[k] * 2], 2, NULL);
    // }
    pub(in crate::zelda_rtl) fn fire_phlegm_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &FIRE_PHLEGM_DRAW_FRAMES[base..base + 2]);
    }

    // -----------------------------------------------------------------------
    // void FlyingTile_Draw(int k) {  // 9ebcca
    //   Four-tile tile plus normal sprite shadow.
    // }
    pub(in crate::zelda_rtl) fn flying_tile_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut info = self.sprite_draw_multiple(k, &FLYING_TILE_DRAW_FRAMES[base..base + 4]);
        // ROM $1E:BCE3
        self.sprite_draw_shadow_long_marked(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_15_Antifairy(int k) {  // 86a50c
    pub(in crate::zelda_rtl) fn sprite_15_antifairy(&mut self, k: usize) {
        self.sprite_draw_antfairy(k);
        self.sprite_15_antifairy_after_draw(k);
    }

    fn sprite_15_antifairy_after_draw(&mut self, k: usize) {
        if self.antifairy_after_draw_before_bounce(k) {
            self.sprite_bounce_from_tile_collision(k);
        }
    }

    pub(in crate::zelda_rtl) fn antifairy_before_bounce(&mut self, k: usize) -> bool {
        self.sprite_draw_antfairy(k);
        self.antifairy_after_draw_before_bounce(k)
    }

    fn antifairy_after_draw_before_bounce(&mut self, k: usize) -> bool {
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        if self.sprite_check_damage_to_link(k) && self.sprite_slot_view(k).delay_main() == 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let t = i16::from(self.game_state.inventory.player_resources.magic.amount()) - 8;
            if t < 0 {
                self.player_magic_mut().set_magic_power(0);
            } else {
                self.set_sound_effect_2(0x1d);
                self.player_magic_mut().set_magic_power(t as u8);
            }
        }
        // $06:A538 JSR $06:E92C: complete body, caller opcode unchanged.
        self.sprite_move_xy_marked(k, 0x06e93e);
        true
    }

    // -----------------------------------------------------------------------
    // void Sprite_82_AntifairyCircle(int k) {  // 9ecb97
    pub(super) fn sprite_82_antifairy_circle(&mut self, k: usize) {
        self.sprite_draw_antfairy(k);
        self.sprite_82_antifairy_circle_after_draw(k);
    }

    fn sprite_82_antifairy_circle_after_draw(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }

        let j = usize::from(self.sprite_slot_view(k).a() & 1);
        self.sprite_slot_view_mut(k)
            .add_x_velocity(SPRITE_82_ANTIFAIRY_CIRCLE_VEL[j] as u8);
        if self.sprite_slot_view(k).x_velocity() == ANTIFAIRY_CIRCLE_VELOCITY_TARGETS[j] {
            self.sprite_slot_view_mut(k).add_a(1);
        }

        let j = usize::from(self.sprite_slot_view(k).b() & 1);
        self.sprite_slot_view_mut(k)
            .add_y_velocity(SPRITE_82_ANTIFAIRY_CIRCLE_VEL[j] as u8);
        if self.sprite_slot_view(k).y_velocity() == ANTIFAIRY_CIRCLE_VELOCITY_TARGETS[j] {
            self.sprite_slot_view_mut(k).add_b(1);
        }

        self.sprite_move_xy_bank(k, 0x1eff34);
        if self.sprite_slot_view(k).x_velocity() != 0
            && self.sprite_slot_view(k).y_velocity() != 0
            && self.sprite_check_if_room_is_clear()
        {
            let value = 0x15;
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            let value = if sign8(self.sprite_slot_view(k).x_velocity()) {
                (-16i8) as u8
            } else {
                16
            };
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = if sign8(self.sprite_slot_view(k).y_velocity()) {
                (-16i8) as u8
            } else {
                16
            };
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }
        // $1E:CBFE JSR $1E:C26B: complete body, caller opcode unchanged.
        self.sprite_check_damage_to_link_bank1e(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_7E_Firebar_Clockwise(int k) {  // 9ed01a
    pub(super) fn sprite_7_e_firebar_clockwise(&mut self, k: usize) {
        self.firebar_main(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let j = usize::from(self.sprite_slot_view(k).sprite_type().wrapping_sub(0x7e))
            + if self.game_state.inventory.save_progress.palace_index_x2() == 18 {
                2
            } else {
                0
            };
        let t = ((u16::from(self.sprite_slot_view(k).a())
            | (u16::from(self.sprite_slot_view(k).b()) << 8)) as i16)
            .wrapping_add(SPRITE_7_E_FIREBAR_CLOCKWISE_INCR[j]) as u16
            & 0x01ff;
        let value = t as u8;
        self.sprite_slot_view_mut(k).set_a(value);
        let value = (t >> 8) as u8;
        self.sprite_slot_view_mut(k).set_b(value);
    }

    // -----------------------------------------------------------------------
    // void Firebar_Main(int k) {  // 9ed049
    pub(super) fn firebar_main(&mut self, k: usize) {
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        self.sprite_workspace_mut().set_shared_scratch_a(info_flags);
        self.draw_scratch_position_mut()
            .set_low_position(x as u8, y as u8);

        let angle = u16::from(self.sprite_slot_view(k).a())
            | (u16::from(self.sprite_slot_view(k).b()) << 8);
        let sinval = guruguru_bar_sin(angle, 0x40);
        let cosval = guruguru_bar_sin((angle.wrapping_add(0x80)) & 0x01ff, 0x40);
        let flags = ((self.sprite_slot_view(k).subtype2() << 4) & 0xc0) | info_flags;
        for i in (0..4usize).rev() {
            let m = (i + 1) as i16;
            let xo = i16::from(sinval) * m / 4;
            let yo = i16::from(cosval) * m / 4;
            self.set_oam_plain_at(
                oam,
                x.wrapping_add(xo as u16) as u8,
                y.wrapping_add(yo as u16) as u8,
                0x28,
                flags,
                2,
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries(k, 3, 0xff);

        if ((((k as u8) ^ self.game_state.frame.frame_counter) & 3)
            | self.game_state.frame.submodule
            | self.game_state.frame.modal_pause_flag)
            != 0
        {
            return;
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        for _ in 0..4 {
            let ext_index = (oam - OAM_BUF) / 4;
            if self.game_state.oam.extended_byte(ext_index) & 1 == 0 {
                let ox = self.game_state.oam.entry_x(oam);
                let oy = self.game_state.oam.entry_y(oam);
                let link_x = self.game_state.player.follower_link.x() as u8;
                let link_y = self.game_state.player.follower_link.y() as u8;
                if ox
                    .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2() as u8)
                    .wrapping_sub(link_x)
                    .wrapping_add(12)
                    < 24
                    && oy < 0xf0
                    && oy
                        .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2() as u8)
                        .wrapping_sub(link_y)
                        .wrapping_add(4)
                        < 16
                {
                    self.sprite_attempt_damage_to_link_plus_recoil(k);
                }
            }
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_5D_Roller_VerticalDownFirst(int k) {  // 858dde
    pub(super) fn sprite_5_d_roller_vertical_down_first(&mut self, k: usize) {
        let value = ((self.sprite_slot_view(k).subtype2() >> 1) & 1)
            | (self.sprite_slot_view(k).direction() & 2);

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.spike_roller_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // $05:8DF6 JSR $05:AB93: both complete damage calls.
        self.sprite_check_damage_both_marked(k, 0x05ab93);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 112;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).xor_direction(1);
        }
        let j = usize::from(self.sprite_slot_view(k).direction());
        let value = SPRITE_5_D_ROLLER_VERTICAL_DOWN_FIRST_XYVEL[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_5_D_ROLLER_VERTICAL_DOWN_FIRST_XYVEL[j + 2] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        // ROM $05:8E1A
        self.sprite_move_xy_marked(k, 0x05fa00);
        self.sprite_slot_view_mut(k).add_subtype2(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_66_WallCannonVerticalLeft(int k) {  // 858090
    pub(super) fn sprite_66_wall_cannon_vertical_left(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).direction());
        let value = SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_GRAPHICS[j]
            + u8::from(self.sprite_slot_view(k).delay_aux2() != 0);
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | SPRITE_66_WALL_CANNON_VERTICAL_LEFT_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:80AB JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).xor_a(1);
        }
        j = usize::from(self.sprite_slot_view(k).a());
        let value = SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_66_WALL_CANNON_VERTICAL_LEFT_LOCAL_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        // ROM $05:80D3
        self.sprite_move_xy_marked(k, 0x05fa00);

        if ((((k as u8) << 2).wrapping_add(self.game_state.frame.frame_counter)) & 31) == 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_aux2(value);
        }
        if self.sprite_slot_view(k).delay_aux2() != 1 || self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let mut info = SpriteSpawnInfo::default();
        let spawned = self.sprite_spawn_dynamically_ex(k, 0x6b, &mut info, 13);
        if spawned >= 0 {
            let j = spawned as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x07);
            let value = 1;
            self.sprite_slot_view_mut(j).set_c(value);
            let value = 1;
            self.sprite_slot_view_mut(j).set_graphics(value);
            let i = usize::from(self.sprite_slot_view(k).direction());
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add_signed(i16::from(WALKING_ZORA_SPAWN_X_OFFSETS[i])),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add_signed(i16::from(WALKING_ZORA_SPAWN_Y_OFFSETS[i])),
            );
            let value = WALKING_ZORA_SPAWN_X_VELOCITIES[i] as u8;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = WALKING_ZORA_SPAWN_Y_VELOCITIES[i] as u8;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = (self.sprite_slot_view(j).flags2() & 0xf0) | 1;
            self.sprite_slot_view_mut(j).set_flags2(value);
            self.sprite_slot_view_mut(j).or_flags3(0x47);
            self.sprite_slot_view_mut(j).or_deflection_bits(0x44);
            let value = 32;
            self.sprite_slot_view_mut(j).set_delay_main(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_5B_Spark_Clockwise(int k) {  // 85933f
    pub(super) fn sprite_5_b_spark_clockwise(&mut self, k: usize) {
        // ROM $05:933F JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if (self.game_state.frame.frame_counter & 1) == 0 {
            self.sprite_slot_view_mut(k).xor_oam_flags(6);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = 1;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            // ROM $05:9364 -> $05:B88D, $05:9370 -> $05:B88D, $05:93C4 -> $05:B88D -> $06:E496
            let mut coll = self.sprite_check_tile_collision_long(k);
            let value = (-1i8) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = (-1i8) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            // ROM $05:9364 -> $05:B88D, $05:9370 -> $05:B88D, $05:93C4 -> $05:B88D -> $06:E496
            coll |= self.sprite_check_tile_collision_long(k);
            let j = if coll < 4 {
                if (coll & 1) != 0 {
                    0
                } else {
                    1
                }
            } else if (coll & 4) != 0 {
                2
            } else {
                3
            };
            let value = SPRITE_5_B_SPARK_CLOCKWISE_DIRECTIONS
                [usize::from(u8::from(self.sprite_slot_view(k).sprite_type() != 0x5c) * 4 + j)];
            self.sprite_slot_view_mut(k).set_direction(value);
        }

        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f)
            | SPRITE_5_B_SPARK_CLOCKWISE_OAM_FLAGS
                [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];

        self.sprite_slot_view_mut(k).set_oam_flags(value);
        // ROM $05:93AE
        self.sprite_move_xy_marked(k, 0x05fa00);
        // $05:93B1 JSL $06:F121: complete body, caller opcode unchanged.
        self.sprite_check_damage_to_link_long(k);
        let mut j = usize::from(self.sprite_slot_view(k).direction());
        let value = SOLDIER_B_STEP_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SOLDIER_B_STEP_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        // ROM $05:9364 -> $05:B88D, $05:9370 -> $05:B88D, $05:93C4 -> $05:B88D -> $06:E496
        let _ = self.sprite_check_tile_collision_long(k);

        j = usize::from(self.sprite_slot_view(k).direction());
        if self.sprite_slot_view(k).delay_aux2() != 0 {
            if self.sprite_slot_view(k).delay_aux2() == 6 {
                j = usize::from(SOLDIER_B_NEXT_DIRECTIONS[j]);
            }
        } else if (self.sprite_slot_view(k).wall_collision() & SOLDIER_B_COLLISION_MASKS[j]) == 0 {
            let value = 10;
            self.sprite_slot_view_mut(k).set_delay_aux2(value);
        }
        if (self.sprite_slot_view(k).wall_collision() & SOLDIER_B_FAST_COLLISION_MASKS[j]) != 0 {
            j = usize::from(SOLDIER_B_FAST_NEXT_DIRECTIONS[j]);
        }
        let value = j as u8;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = (SOLDIER_B_FAST_X_VELOCITIES[j] * 2) as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = (SOLDIER_B_FAST_Y_VELOCITIES[j] * 2) as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_80_Firesnake(int k) {  // 9ed1d1
    pub(super) fn sprite_80_firesnake(&mut self, k: usize) {
        // ROM $1E:D1D1 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling_bank(k, 0x1eff34) {
            return;
        }
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f)
            | SPRITE_80_FIRESNAKE_OAM_FLAGS
                [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).a() != 0 {
            let value = self.sprite_slot_view(k).delay_main();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            return;
        }
        // $1E:D206 JSR $1E:C267: both complete damage calls.
        self.sprite_check_damage_both_marked(k, 0x1ec267);
        self.firesnake_spawn_fireball(k);
        if self.sprite_slot_view(k).wall_collision() == 0 {
            // ROM $1E:D211
            self.sprite_move_xy_bank(k, 0x1eff34);
        }
        // ROM $1E:D214 -> $1E:8B2E -> $06:E496
        if self.sprite_check_tile_collision_bank1e_marked(k) != 0 {
            let j = usize::from(
                self.sprite_slot_view(k).direction() * 2 + (self.get_random_number() & 1),
            );
            let value = ZAZAK_DIR2[j];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        let j = usize::from(self.sprite_slot_view(k).direction() & 3);
        let value = SPRITE_80_FIRESNAKE_LOCAL_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_80_FIRESNAKE_LOCAL_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_87_KodongoFire(int k) {  // 9ec274
    pub(super) fn sprite_87_kodongo_fire(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            // ROM $1E:C279 JSL $06:DBF0
            self.sprite_draw_single_large_long(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = (self.sprite_slot_view(k).oam_flags() & 0x3f)
                | SPRITE_87_KODONGO_FIRE_OAM_FLAGS
                    [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            if !self.sprite_check_damage_to_link(k) {
                // ROM $1E:C297
                self.sprite_move_xy_bank(k, 0x1eff34);
                // ROM $1E:C29A -> $1E:8B2E -> $06:E496
                if self.sprite_check_tile_collision_bank1e_marked(k) == 0 {
                    return;
                }
            }
            let value = 127;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).and_oam_flags(63);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
        } else {
            let carried =
                // ROM $1E:C2D4 JSL $06:F2AA
                (self.sprite_check_damage_from_link_long(k) & PLAYER_DAMAGE_CARRY_MASK_DRAW) != 0;
            let mut extinguish = false;
            if carried {
                self.sprite_slot_view_mut(k).subtract_delay_main(1);
                extinguish = self.sprite_slot_view(k).delay_main() == 0;
            }
            if extinguish || self.sprite_slot_view(k).delay_main() == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            let value = SPRITE_87_KODONGO_FIRE_LOCAL_GRAPHICS
                [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.flame_draw(k);
            self.sprite_check_damage_to_link(k);
        }
    }
}
