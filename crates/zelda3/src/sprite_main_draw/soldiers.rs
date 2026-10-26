//! Guards, archers, troopers, their weapons, and soldier draw helpers.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::types::sign8;
use crate::types::PointU8;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_GuardHead(int k, PrepOamCoordsRet *info, int spr_offs) {  // 85b160
    //   int j = sprite_head_dir[k];
    //   OamEnt *oam = GetOamCurPtr() + spr_offs;
    //   SetOamHelper0(oam, info->x, info->y - 9, kChainBallTrooperHead_Char[j],
    //                 info->flags | kChainBallTrooperHead_Flags[j], 2);
    // }
    pub(super) fn sprite_draw_guard_head(
        &mut self,
        k: usize,
        info: &PrepOamCoordsRet,
        spr_offs: i32,
    ) {
        let j = (self.sprite_slot_view(k).head_direction() & 3) as usize;
        let oam = ((self.game_state.oam.current_pointer() as i32) + spr_offs * 4) as usize;
        let y = info.y.wrapping_sub(9);
        let flags = info.flags | CHAIN_BALL_TROOPER_HEAD_FLAGS[j];
        self.set_oam_helper0_at(
            oam,
            info.x,
            y,
            CHAIN_BALL_TROOPER_HEAD_CHARS[j],
            flags,
            2,
        );
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_BNCBody(int k, PrepOamCoordsRet *info, int spr_offs) {  // 85b3cb
    //   int g = sprite_graphics[k];
    //   spr_offs += kFlailTrooperBody_SprOffs[g] >> 2;
    //   OamEnt *oam = GetOamCurPtr() + spr_offs;
    //   int n = kFlailTrooperBody_Num[g];
    //   do {
    //     int j = g * 3 + n;
    //     SetOamHelper0(oam, info->x + kFlailTrooperBody_X[j], info->y + kFlailTrooperBody_Y[j],
    //                   kFlailTrooperBody_Char[j], info->flags | kFlailTrooperBody_Flags[j],
    //                   kFlailTrooperBody_Big[j]);
    //     if (n == 2)
    //       oam++;
    //   } while (oam++, --n >= 0);
    // }
    pub(super) fn sprite_draw_bnc_body(
        &mut self,
        k: usize,
        info: &PrepOamCoordsRet,
        spr_offs: i32,
    ) {
        let g = self.sprite_slot_view(k).graphics() as usize;
        let spr_offs = spr_offs + ((FLAIL_TROOPER_BODY_SPRITE_OFFSETS[g] >> 2) as i32);
        let mut oam = ((self.game_state.oam.current_pointer() as i32) + spr_offs * 4) as usize;
        let mut n: i32 = FLAIL_TROOPER_BODY_SEGMENT_COUNTS[g] as i32;
        loop {
            let j = (g * 3) + (n as usize);
            let x = info
                .x
                .wrapping_add(FLAIL_TROOPER_BODY_X_OFFSETS[j] as i16 as u16);
            let y = info
                .y
                .wrapping_add(FLAIL_TROOPER_BODY_Y_OFFSETS[j] as i16 as u16);
            self.set_oam_helper0_at(
                oam,
                x,
                y,
                FLAIL_TROOPER_BODY_CHARS[j],
                info.flags | FLAIL_TROOPER_BODY_FLAGS[j],
                FLAIL_TROOPER_BODY_SIZES[j],
            );
            if n == 2 {
                oam += 4;
            }
            oam += 4;
            n -= 1;
            if n < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_BNCFlail(int k, PrepOamCoordsRet *info) {  // 85b468
    //   See sprite_main.c:3962 — chain-ball flail with sin-wave segments.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_bnc_flail(&mut self, k: usize, info: &PrepOamCoordsRet) {
        let oam = self.game_state.oam.current_pointer_usize();

        self.draw_scratch_position_mut()
            .set_low_position(info.x as u8, info.y as u8);

        let r0 =
            (self.sprite_slot_view(k).a() as u16) | ((self.sprite_slot_view(k).b() as u16) << 8);
        let qq = if self.sprite_slot_view(k).ai_state() < 2 {
            0u8
        } else {
            FLAIL_TROOPER_CHAIN_RADIUS_BY_FRAME
                [(self.sprite_slot_view(k).delay_aux2() & 0x1f) as usize]
        };
        let r12 = FLAIL_TROOPER_WEAPON_X_CENTER_BY_DIRECTION
            [(self.sprite_slot_view(k).direction() & 3) as usize] as u8;
        let r13 = FLAIL_TROOPER_WEAPON_Y_CENTER_BY_DIRECTION
            [(self.sprite_slot_view(k).direction() & 3) as usize] as u8;

        let r2 = (r0.wrapping_add(0x80)) & 0x1ff;

        let r14 = chain_ball_mult_draw(SHARED_SINE_LOOKUP_TABLE[(r0 & 0xff) as usize], qq);
        let r4: u8 = if (r0 & 0x100) != 0 {
            0u8.wrapping_sub(r14)
        } else {
            r14
        };

        let r15 = chain_ball_mult_draw(SHARED_SINE_LOOKUP_TABLE[(r2 & 0xff) as usize], qq);
        let r6: u8 = if (r2 & 0x100) != 0 {
            0u8.wrapping_sub(r15)
        } else {
            r15
        };

        let hib_dv8 = r4.wrapping_sub(4).wrapping_add(r12);
        let lob_dv8 = r6.wrapping_sub(4).wrapping_add(r13);
        self.hitbox_scratch_offset_mut()
            .set_offsets(lob_dv8, hib_dv8);

        let dv7_byte = info.x as u8;
        let dv7_hibyte = info.y as u8;
        let x0 = hib_dv8.wrapping_add(dv7_byte);
        let y0 = lob_dv8.wrapping_add(dv7_hibyte);
        self.set_oam_plain_at(oam, x0, y0, 0x2a, 0x2d, 2);

        let mut oam_cur = oam + 4;
        // for (int i = 3; i >= 0; i--, oam++) {
        let mut i: i32 = 3;
        loop {
            let mult = FLAIL_TROOPER_CHAIN_SEGMENT_SCALES[i as usize];
            let t = (mult as u16).wrapping_mul(r14 as u16) >> 8;
            let mut t = t as u8;
            if sign8(r4) {
                t = 0u8.wrapping_sub(t);
            }
            let x = t.wrapping_add(dv7_byte).wrapping_add(r12);
            let ty = (mult as u16).wrapping_mul(r15 as u16) >> 8;
            let mut ty = ty as u8;
            if sign8(r6) {
                ty = 0u8.wrapping_sub(ty);
            }
            let y = ty.wrapping_add(dv7_hibyte).wrapping_add(r13);
            self.set_oam_plain_at(oam_cur, x, y, 0x3f, 0x2d, 0);
            oam_cur += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries(k, 4, 0xff);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_BigCannonball(int k) {  // 85b6a4 — sprite_main.c:4019
    //   Sprite_PrepOamCoordOrDoubleRet, then four large-tile OAM entries.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_big_cannonball(&mut self, k: usize) {
        let info = match self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) {
            Some(p) => PrepOamCoordsRet::from_tuple(p),
            None => return,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = (self.sprite_slot_view(k).graphics() & 1) as usize;
        let mut i: i32 = 3;
        loop {
            let x = info
                .x
                .wrapping_add(METAL_BALL_LARGE_X_OFFSETS[i as usize] as i16 as u16);
            let y = info
                .y
                .wrapping_add(METAL_BALL_LARGE_Y_OFFSETS[i as usize] as i16 as u16);
            self.set_oam_helper0_at(
                oam,
                x,
                y,
                METAL_BALL_LARGE_CHARS[g * 4 + i as usize],
                METAL_BALL_LARGE_FLAGS[i as usize] | info.flags,
                2,
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // SpriteDraw_BombGuard_Arm (sprite_main.c:4527) is already ported in
    // crates/zelda3/src/sprite_main_guard.rs; this file does NOT re-port it.

    // -----------------------------------------------------------------------
    // void SpriteDraw_SpriteBombExplosion(int k) {  // 85c113 — sprite_main.c:4537
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_sprite_bomb_explosion(&mut self, k: usize) {
        let info = match self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) {
            Some(p) => PrepOamCoordsRet::from_tuple(p),
            None => return,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let base = ((self.sprite_slot_view(k).delay_aux1() >> 1) & 0xc) as usize;
        let mut i: i32 = 3;
        loop {
            let j = base + i as usize;
            // The C writes only x/y/charnum/flags (no SetOamHelper0). Mirror.
            self.oam_state_mut().write_entry(
                oam,
                info.x
                    .wrapping_add(ENEMY_BOMB_EXPLOSION_X_OFFSETS[j] as i16 as u16)
                    as u8,
                info.y
                    .wrapping_add(ENEMY_BOMB_EXPLOSION_Y_OFFSETS[j] as i16 as u16)
                    as u8,
                ENEMY_BOMB_EXPLOSION_CHARS[j],
                ENEMY_BOMB_EXPLOSION_FLAGS[j] | info.flags,
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries(k, 3, 2);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_GuardSpear(int k, PrepOamCoordsRet *info, int spr_offs) {  // 85cd54
    //   sprite_main.c:4926 — javelin/spear OAM emitter.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_guard_spear(
        &mut self,
        k: usize,
        info: &PrepOamCoordsRet,
        spr_offs: i32,
    ) {
        let mut oam = ((self.game_state.oam.current_pointer() as i32) + spr_offs * 4) as usize;
        let r6 = self
            .sprite_slot_view(k)
            .direction()
            .wrapping_mul(4)
            .wrapping_add(((self.sprite_slot_view(k).a() ^ 1) << 1) & 2);
        let mut i: i32 = 1;
        loop {
            let j = (r6.wrapping_add(i as u8) & 0xf) as usize;
            let x = info
                .x
                .wrapping_add(SOLDIER_THROWING_DRAW_X_OFFSETS[j] as i16 as u16);
            let y = info
                .y
                .wrapping_add(SOLDIER_THROWING_DRAW_Y_OFFSETS[j] as i16 as u16);
            self.hitbox_scratch_offset_mut().set_offsets(
                SOLDIER_THROWING_DRAW_Y_OFFSETS[j] as u8,
                SOLDIER_THROWING_DRAW_X_OFFSETS[j] as u8,
            );
            let char_off = if self.sprite_slot_view(k).sprite_type() >= 0x48 {
                3u8
            } else {
                0u8
            };
            let charnum = SOLDIER_THROWING_DRAW_CHARS[j].wrapping_sub(char_off);
            let flags = ((SOLDIER_THROWING_DRAW_FLAGS[j] | info.flags) & 0xf1) | 8;
            self.set_oam_helper0_at(oam, x, y, charnum, flags, 0);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Archer_Weapon(int k, int spr_offs, PrepOamCoordsRet *info) {  // 85d4d4
    //   sprite_main.c:5273 — 48-entry archer-weapon OAM cycle.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_archer_weapon(
        &mut self,
        k: usize,
        spr_offs: i32,
        info: &PrepOamCoordsRet,
    ) {
        let mut oam = ((self.game_state.oam.current_pointer() as i32) + spr_offs * 4) as usize;
        let g_signed = self.sprite_slot_view(k).graphics() as i32 - 14;
        let base: usize = if g_signed < 0 {
            ARCHER_WEAPON_BASE_FRAME_BY_DIRECTION
                [(self.sprite_slot_view(k).direction() & 3) as usize] as usize
        } else {
            g_signed as usize
        };
        let mut i: i32 = 3;
        loop {
            let j = base * 4 + i as usize;
            let x = info
                .x
                .wrapping_add(ARCHER_SOLDIER_DRAW_X_OFFSETS[j] as i16 as u16);
            let y = info
                .y
                .wrapping_add(ARCHER_SOLDIER_DRAW_Y_OFFSETS[j] as i16 as u16);
            self.set_oam_helper0_at(
                oam,
                x,
                y,
                ARCHER_SOLDIER_DRAW_CHARS[j],
                ARCHER_SOLDIER_DRAW_FLAGS[j] | 0x20,
                0,
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_1B_Arrow(int k) {  // 86b754
    //   Runtime for enemy arrows, including embedded / ricochet states.
    // }
    pub(super) fn sprite_1_b_arrow(&mut self, k: usize) {
        self.enemy_arrow_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.sprite_slot_view(k).state() == 9 {
            let mut j = self.sprite_slot_view(k).delay_main();
            if j != 0 {
                j = j.wrapping_sub(1);
                if j == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                } else if j >= 32 && (j & 1) == 0 {
                    let idx = (((u16::from(self.game_state.frame.frame_counter) << 1) & 4)
                        | u16::from(self.sprite_slot_view(k).direction()))
                        as usize;
                    let value = ENEMY_ARROW_X_VELOCITIES[idx];
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = ENEMY_ARROW_Y_VELOCITIES[idx];
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    self.sprite_move_xy(k);
                }
                return;
            }
            self.sprite_check_damage_to_link_same_layer(k);
            if self.sprite_slot_view(k).e() == 0 && self.sprite_check_tile_collision(k) != 0 {
                if self.sprite_slot_view(k).a() != 0 {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x5);
                    self.sprite_schedule_for_breakage(k);
                    self.sprite_place_weapon_tink(k);
                } else {
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_a(value);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x8);
                }
            } else {
                self.sprite_move_xy(k);
            }
        } else {
            if self.sprite_slot_view(k).ai_state() == 0 {
                self.sprite_apply_ricochet(k);
                let value = 24;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 255;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
            }
            let value =
                ENEMY_ARROW_DIRECTIONS[((self.sprite_slot_view(k).delay_main() >> 3) & 3) as usize];
            self.sprite_slot_view_mut(k).set_direction(value);
            self.sprite_move_z(k);
            self.sprite_move_xy(k);
            self.sprite_slot_view_mut(k).subtract_z_velocity(2);
            if sign8(self.sprite_slot_view(k).z()) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void EnemyArrow_Draw(int k) {  // 86b867
    //   Two 8x8 arrow tiles selected by direction and arrow type.
    // }
    pub(super) fn enemy_arrow_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r6 = (self.sprite_slot_view(k).direction() as usize) * 2;
        let r7 = (self.sprite_slot_view(k).a() as usize) * 8;
        for i in (0..2).rev() {
            let j = r6 + i;
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(ENEMY_ARROW_DRAW_X_OFFSETS[j] as u16),
                y.wrapping_add(ENEMY_ARROW_DRAW_Y_OFFSETS[j] as u16),
                ENEMY_ARROW_DRAW_CHARS[j + r7],
                ENEMY_ARROW_DRAW_FLAGS[j + r7] | flags,
                0,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void TutorialSoldier_Draw(int k) {  // 85d64b
    pub(in crate::zelda_rtl) fn tutorial_soldier_draw(&mut self, k: usize) {
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = usize::from(self.sprite_slot_view(k).graphics()) * 5;
        for i in (0..5).rev() {
            let j = d + i;
            let mut flags = TUTORIAL_SOLDIER_DRAW_FLAGS[j] | info_flags;
            if TUTORIAL_SOLDIER_DRAW_CHARS[j] < 0x40 {
                flags = (flags & 0xf1) | 8;
            }
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add_signed(TUTORIAL_SOLDIER_DRAW_X_OFFSETS[j]),
                y.wrapping_add_signed(TUTORIAL_SOLDIER_DRAW_Y_OFFSETS[j]),
                TUTORIAL_SOLDIER_DRAW_CHARS[j],
                flags,
                TUTORIAL_SOLDIER_DRAW_BIG[j],
            );
            oam += 4;
        }
        let mut info = PrepOamCoordsRet {
            x,
            y,
            r4: 0,
            flags: info_flags,
        };
        self.sprite_draw_shadow_custom(k, &mut info, 12);
    }

    // -----------------------------------------------------------------------
    // void PsychoTrooper_Draw(int k) {  // 85ccd5
    pub(super) fn psycho_trooper_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 3);
        self.sprite_draw_bnc_body(k, &info, 2);
        self.sprite_draw_guard_spear(k, &info, 0);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info.x,
                y: info.y,
                r4: info.r4,
                flags: info.flags,
            };
            self.sprite_draw_shadow_custom(
                k,
                &mut shadow_info,
                PSYCHO_TROOPER_DRAW_SHADOW[usize::from(self.sprite_slot_view(k).direction() & 3)],
            );
        }
    }

    // -----------------------------------------------------------------------
    // void JavelinTrooper_Draw(int k) {  // 85d192
    pub(super) fn javelin_trooper_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 3);
        self.sprite_draw_bnc_body(k, &info, 2);
        if self.sprite_slot_view(k).graphics() < 20 {
            self.sprite_draw_guard_spear(k, &info, 0);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info.x,
                y: info.y,
                r4: info.r4,
                flags: info.flags,
            };
            self.sprite_draw_shadow_custom(
                k,
                &mut shadow_info,
                JAVELIN_TROOPER_DRAW_SHADOW[usize::from(self.sprite_slot_view(k).direction() & 3)],
            );
        }
    }

    // -----------------------------------------------------------------------
    // void BushJavelinSoldier_Draw(int k) {  // 85d141
    pub(super) fn bush_javelin_soldier_draw(&mut self, k: usize) {
        let bak0 = self.sprite_slot_view(k).graphics();
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let bak1 = self.sprite_slot_view(k).oam_flags();
        let value = (self.sprite_slot_view(k).oam_flags() & 0xf1) | 2;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let bak2 = self.game_state.sprites.workspace.current_sprite_y();
        self.sprite_workspace_mut()
            .set_current_sprite_y(bak2.wrapping_add(8));
        // ROM $05:D162 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        self.sprite_workspace_mut().set_current_sprite_y(bak2);
        let value = bak1;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = bak0;
        self.sprite_slot_view_mut(k).set_graphics(value);

        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        let guard_info = PrepOamCoordsRet {
            x: tuple.0,
            y: tuple.1,
            r4: 0,
            flags: tuple.2,
        };
        self.guard_animate_head(k, 0x10 / 4, &guard_info);
        self.sprite_draw_bnc_body(k, &info, 0x0c / 4);
        if self.sprite_slot_view(k).graphics() < 20 {
            self.sprite_draw_guard_spear(k, &info, 1);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info.x,
                y: info.y,
                r4: info.r4,
                flags: info.flags,
            };
            self.sprite_draw_shadow_custom(
                k,
                &mut shadow_info,
                BUSH_JAVELIN_SOLDIER_DRAW_SHADOW
                    [usize::from(self.sprite_slot_view(k).direction() & 3)],
            );
        }
    }

    // -----------------------------------------------------------------------
    // void ArcherSoldier_Draw(int k) {  // 85d38c
    pub(super) fn archer_soldier_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        let guard_info = PrepOamCoordsRet {
            x: tuple.0,
            y: tuple.1,
            r4: 0,
            flags: tuple.2,
        };
        let d = usize::from(self.sprite_slot_view(k).direction() & 3);
        self.guard_animate_head(k, ARCHER_SOLDIER_DRAW_HEAD_OAM_OFFS[d] >> 2, &guard_info);
        self.guard_animate_body(k, ARCHER_SOLDIER_DRAW_BODY_OAM_OFFS[d] >> 2, &guard_info);
        self.sprite_draw_archer_weapon(
            k,
            i32::from(ARCHER_SOLDIER_DRAW_WEAPON_OAM_OFFS[d] >> 2),
            &info,
        );
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info.x,
                y: info.y,
                r4: info.r4,
                flags: info.flags,
            };
            self.sprite_draw_shadow_custom(k, &mut shadow_info, ARCHER_SOLDIER_DRAW_SHADOW[d]);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_45_HogSpearMan(int k) {  // 85cbe0
    pub(super) fn sprite_45_hog_spear_man(&mut self, k: usize) {
        if self.hog_spear_man_through_body_increments(k) {
            self.guard_update_body_graphics(k);
        }
    }

    pub(in crate::zelda_rtl) fn hog_spear_man_through_body_increments(&mut self, k: usize) -> bool {
        self.guard_handle_all_animation(k);
        self.hog_spear_man_after_animation_through_body_increments(k)
    }

    pub(in crate::zelda_rtl) fn hog_spear_man_after_animation_through_body_increments(
        &mut self,
        k: usize,
    ) -> bool {
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        self.bolt_guard_trigger_chase_theme(k);
        self.guard_parry_sword_attacks(k);
        if self.sprite_return_if_recoiling(k) {
            return false;
        }
        if self.sprite_slot_view(k).wall_collision() == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision(k);
        self.sprite_check_damage_to_link(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
            let dir = self.sprite_direction_to_face_link(k);
            let value = dir;
            self.sprite_slot_view_mut(k).set_head_direction(value);
            let value = dir;
            self.sprite_slot_view_mut(k).set_direction(value);
            self.sprite_apply_speed_towards_link(k, 18);
            self.guard_apply_speed_in_direction(k);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_slot_view_mut(k).increment_subtype2();
        true
    }

    // -----------------------------------------------------------------------
    // void Sprite_44_BluesainBolt(int k) {  // 85cc65
    pub(super) fn sprite_44_bluesain_bolt(&mut self, k: usize) {
        self.psycho_trooper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.bolt_guard_trigger_chase_theme(k);
        self.guard_parry_sword_attacks(k);
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.sprite_slot_view(k).wall_collision() == 0 {
            self.sprite_move_xy(k);
        }
        // ROM $05:CC78 -> $05:B88D -> $06:E496
        self.sprite_check_tile_collision_long(k);
        self.sprite_check_damage_to_link(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
            let dir = self.sprite_direction_to_face_link(k);
            let value = dir;
            self.sprite_slot_view_mut(k).set_direction(value);
            let value = dir;
            self.sprite_slot_view_mut(k).set_head_direction(value);
            self.sprite_apply_speed_towards_link(k, 18);
            self.guard_apply_speed_in_direction(k);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let j = usize::from(
            ((self.sprite_slot_view(k).subtype2() >> 1) & 7)
                | (self.sprite_slot_view(k).direction() << 3),
        );
        let value = FLAIL_TROOPER_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_49_RedBushGuard(int k) {  // 85d1ac
    pub(super) fn sprite_49_red_bush_guard(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).ai_state() == 2 {
                self.bush_javelin_soldier_draw(k);
            } else {
                self.bush_soldier_common_draw(k);
            }
        }
        self.sprite_bush_guard_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_47_GreenBushGuard(int k) {  // 85d1bf
    pub(super) fn sprite_47_green_bush_guard(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).graphics() >= 14 {
                self.archer_soldier_draw(k);
            } else {
                self.bush_soldier_common_draw(k);
            }
        }
        self.sprite_bush_guard_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_48_RedJavelinGuard(int k) {  // 85cde1
    pub(super) fn sprite_48_red_javelin_guard(&mut self, k: usize) {
        let bak0 = self.sprite_slot_view(k).graphics();
        let j = usize::from(self.sprite_slot_view(k).direction() & 3);
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = SPRITE_48_RED_JAVELIN_GUARD_DIR_LOCK[j];
            self.sprite_slot_view_mut(k).set_direction(value);
            let value = SPRITE_48_RED_JAVELIN_GUARD_LOCAL_GRAPHICS[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.javelin_trooper_draw(k);
        let value = j as u8;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = bak0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.soldier_throwing_common(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_46_BlueArcher(int k) {  // 85cdff
    pub(super) fn sprite_46_blue_archer(&mut self, k: usize) {
        let bak0 = self.sprite_slot_view(k).graphics();
        let j = usize::from(self.sprite_slot_view(k).direction() & 3);
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = SPRITE_46_BLUE_ARCHER_DIR_LOCK[j];
            self.sprite_slot_view_mut(k).set_direction(value);
            let value = SPRITE_46_BLUE_ARCHER_LOCAL_GRAPHICS[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.archer_soldier_draw(k);
        let value = j as u8;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = bak0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.soldier_throwing_common(k);
    }

    // -----------------------------------------------------------------------
    // void SpikeTrap_Draw(int k) {  // 9ecfff
    pub(in crate::zelda_rtl) fn spike_trap_draw(&mut self, k: usize) {
        self.sprite_draw_multiple(k, &SPIKE_TRAP_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void ChainBallTrooper_Draw(int k) {  // sprite_main.c:1403
    pub(super) fn chain_ball_trooper_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 0x18 / 4);
        self.sprite_draw_bnc_body(k, &info, 0x14 / 4);
        self.sprite_draw_bnc_flail(k, &info);

        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info.x,
                y: info.y,
                r4: info.r4,
                flags: info.flags,
            };
            self.sprite_draw_shadow_custom(
                k,
                &mut shadow_info,
                CHAIN_BALL_TROOPER_DRAW_SHADOW
                    [usize::from(self.sprite_slot_view(k).direction() & 3)],
            );
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6B_CannonTrooper(int k) {  // sprite_main.c:1418
    pub(super) fn sprite_6_b_cannon_trooper(&mut self, k: usize) {
        if self.sprite_slot_view(k).c() != 0 {
            self.sprite_cannonball(k);
            return;
        }
        // C Sprite_6B_CannonTrooper asserts when this slot is not a cannonball.
        panic!("assertion failed: false");
    }

    // -----------------------------------------------------------------------
    // void Sprite_Cannonball(int k) {  // 85ab54
    pub(super) fn sprite_cannonball(&mut self, k: usize) {
        // ROM $05:AB54 JSL $06:DBF0
        self.sprite_draw_single_large_long(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        // ROM $05:AB5B
        self.sprite_move_xy_marked(k, 0x05fa00);
        if self.sprite_slot_view(k).delay_main() == 30 {
            self.sprite_spawn_poof_garnish(k);
        } else if self.sprite_slot_view(k).delay_main() == 0
            // ROM $05:AB6F -> $05:B88D -> $06:E496
            && self.sprite_check_tile_collision_long(k) != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_slot_view_mut(k).add_x_low(4);
            self.sprite_slot_view_mut(k).add_y_low(4);
            self.sprite_place_rupulse_spark_2(k);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x05);
        }
        self.sprite_check_damage_to_and_from_link(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_6A_BallNChain(int k) {  // 85b01b
    pub(super) fn sprite_6_a_ball_n_chain(&mut self, k: usize) {
        self.chain_ball_trooper_draw(k);
        if self.sprite_slot_view(k).ai_state() < 2 {
            self.draw_scratch_position_mut().set_flags_high(0x80);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.guard_parry_sword_attacks(k);

        let mut t = (u16::from(self.sprite_slot_view(k).b()) << 8)
            | u16::from(self.sprite_slot_view(k).a());
        t = t.wrapping_add(u16::from(
            CHAIN_BALL_TROOPER_BODY_CHAR_BY_STATE[usize::from(self.sprite_slot_view(k).ai_state())],
        ));
        let value = t as u8;
        self.sprite_slot_view_mut(k).set_a(value);
        let value = ((t >> 8) & 1) as u8;
        self.sprite_slot_view_mut(k).set_b(value);
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        // ROM $05:B04B -> $05:B88D -> $06:E496
        let _ = self.sprite_check_tile_collision_long(k);
        // ROM $05:B04E
        self.sprite_move_xy_marked(k, 0x05fa00);
        self.sprite_check_damage_to_link(k);

        let mut pt = PointU8 { x: 0, y: 0 };
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x0f) == 0 {
            let (value, offset) = self.sprite_direction_and_offset_to_face_link(k);
            pt = offset;
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x0f) == 0 {
                    let value = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_direction(value);
                    if pt.y.wrapping_add(0x40) < 0x68 && pt.x.wrapping_add(0x30) < 0x60 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 24;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        return;
                    }
                    self.sprite_apply_speed_towards_link(k, 8);
                }
                self.ball_n_chain_animate(k);
            }
            1 => {
                self.sprite_zero_velocity_xy(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            2 => {
                let attack_idx = usize::from(
                    ((self.sprite_slot_view(k).a() >> 7) & 1) + self.sprite_slot_view(k).b() * 2,
                );
                if self.sprite_slot_view(k).delay_main() == 0
                    && self.sprite_slot_view(k).head_direction()
                        == FLAIL_TROOPER_ATTACK_DIRECTIONS[attack_idx]
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_aux2(value);
                }
                self.sprite_slot_view_mut(k).add_subtype2(1);
                self.ball_n_chain_animate(k);
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x0f) == 0 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 6);
                }
            }
            3 => {
                self.sprite_zero_velocity_xy(k);
                let t = self.sprite_slot_view(k).delay_aux2();
                if t == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    self.ball_n_chain_animate(k);
                } else if t >= 0x10 {
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    self.ball_n_chain_animate(k);
                    if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x0f) == 0 {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 6);
                    }
                } else {
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    self.ball_n_chain_animate(k);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void BallNChain_Animate(int k) {  // 85b0ab
    pub(super) fn ball_n_chain_animate(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let j = usize::from(
            self.sprite_slot_view(k).direction() * 8
                + ((self.sprite_slot_view(k).subtype2() >> 2) & 7),
        );
        let value = FLAIL_TROOPER_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_50_Cannonball(int k) {  // 85b648
    pub(super) fn sprite_50_cannonball(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            // ROM $05:B64D JSL $06:DBF0
            self.sprite_draw_single_large_long(k);
        } else {
            self.sprite_draw_big_cannonball(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 2) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        // ROM $05:B666
        self.sprite_move_xy_marked(k, 0x05fa00);
        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        // ROM $05:B67D -> $05:B88D -> $06:E496
        if self.sprite_slot_view(k).delay_aux2() == 0 && self.sprite_check_tile_collision_long(k) != 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_4B_GreenKnifeGuard(int k) {  // 85bca2
    pub(super) fn sprite_4_b_green_knife_guard(&mut self, k: usize) {
        self.green_knife_guard_before_recruit_oam_prep(k);
        self.green_knife_guard_from_recruit_draw(k);
    }

    pub(in crate::zelda_rtl) fn green_knife_guard_before_recruit_oam_prep(&mut self, k: usize) {
        // Cycle ledger (an RTS-dispatch target of SpriteActive2_Main, m8 x8,
        // charging into the open Sprite_ExecuteSingle scope): $05:BCA2-BCAF
        // the graphics-index setup (178), $05:BCB2 JSR Recruit_Draw (46).
        crate::cycle_ledger::charge(178 + 46);
        let value = RECRUIT_GRAPHICS[usize::from(
            self.sprite_slot_view(k)
                .direction()
                .wrapping_add((self.sprite_slot_view(k).subtype2() >> 1) & 4),
        )];
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    pub(in crate::zelda_rtl) fn green_knife_guard_from_recruit_draw(&mut self, k: usize) {
        self.recruit_draw(k);
        // $05:BCB5 JSR Sprite_ReturnIfInactive_ (46), $05:BCB8 JSR
        // Sprite_ReturnIfRecoiling_ (46): each double-returns when it fires.
        crate::cycle_ledger::charge(46);
        if self.sprite_return_if_inactive_bank5(k) {
            return;
        }
        crate::cycle_ledger::charge(46);
        if self.sprite_return_if_recoiling(k) {
            // Sprite_ReturnIfRecoiling_ $05:F971 is unannotated, but its
            // double return's second PLA and RTS (70) are measured in this
            // frame, as for every bank-5 double-return wrapper.
            crate::cycle_ledger::charge(28 + 42);
            return;
        }
        // $05:BCBB-BCC7 JSR Sprite_CheckDamageToAndFromLink, JSR Sprite_Move_,
        // JSR Sprite_CheckTileCollision, LDA $D80,x, BNE (186; taken +6 into
        // GreenKnifeGuard_Moving).
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        if self.sprite_slot_view(k).ai_state() != 0 {
            crate::cycle_ledger::charge(186 + 6);
            self.green_knife_guard_moving(k);
            return;
        }
        // $05:BCC9 LDA $DF0,x : BNE (48; taken +6 into the $05:BD15 RTS, 42).
        if self.sprite_slot_view(k).delay_main() != 0 {
            crate::cycle_ledger::charge(186 + 48 + 6 + 42);
            return;
        }
        // $05:BCCE-BCEC (394: JSL GetRandomNumber, the delay/state/direction
        // stores, JSR Sprite_DirectionToFacePlayer__, TYA, LDY $DE0,x, CMP
        // $DE0,x, BNE).
        crate::cycle_ledger::charge(186 + 48 + 394);

        // ROM $05:bccd calls the RNG, then executes `AND #$3f; ADC #$30`
        // without clearing carry between them.
        let value = self.get_random_number_with_carry().masked_adc(0x3f, 0x30);

        self.sprite_slot_view_mut(k).set_delay_main(value);
        self.sprite_slot_view_mut(k).add_ai_state(1);
        let value = self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_direction(value);
        let mut j = self.sprite_slot_view(k).direction();
        let (face, pt) = self.sprite_direction_and_offset_to_face_link(k);
        // Facing runs $05:BCEE LDA $0E, CLC, ADC #$10, CMP #$20, BCC (86;
        // taken +6 into $05:BD00 when x is near), else $05:BCF7 the same
        // test on $0F (86; taken +6 to $05:BD09 when y is far, else
        // $05:BD00-BD06 INY x4, LDA #$80, STA $DF0,x, 110). Not facing takes
        // the $05:BCEC BNE (+6). $05:BD09-BD12 (140) stores the velocities
        // and $05:BD15 RTS (42).
        let x_near = pt.x.wrapping_add(0x10) < 0x20;
        let y_near = pt.y.wrapping_add(0x10) < 0x20;
        crate::cycle_ledger::charge(if j == face {
            if x_near {
                86 + 6 + 110
            } else if y_near {
                86 + 86 + 110
            } else {
                86 + 86 + 6
            }
        } else {
            6
        });
        if j == face && (x_near || y_near) {
            j = j.wrapping_add(4);
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        crate::cycle_ledger::charge(140 + 42);
        let value = RECRUIT_X_VELOCITIES[usize::from(j)];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = RECRUIT_Y_VELOCITIES[usize::from(j)];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    pub(in crate::zelda_rtl) fn bomb_trooper_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_guard_head(k, &info, 2);
        self.sprite_draw_bnc_body(k, &info, 1);
        if self.sprite_slot_view(k).graphics() < 20 {
            let j = ((usize::from(self.sprite_slot_view(k).direction()) * 2)
                | usize::from(self.sprite_slot_view(k).subtype2()))
                & 7;
            let oam = self.game_state.oam.current_pointer_usize();
            self.set_oam_helper0_at(
                oam,
                info.x
                    .wrapping_add(BOMB_TROOPER_DRAW_ARM_X_OFFSETS[j] as i16 as u16),
                info.y
                    .wrapping_add(BOMB_TROOPER_DRAW_ARM_Y_OFFSETS[j] as i16 as u16),
                0x6e,
                (info.flags & 0x30) | 0x8,
                2,
            );
        }
        let mut shadow_info = PrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 10);
    }

    pub(super) fn bush_soldier_common_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        for i in (0..2).rev() {
            let j = g + i;
            let mut flags = BUSH_SOLDIER_COMMON_DRAW_FLAGS[j] | 0x20;
            if i == 0 {
                flags = (flags & !0x0e) | info_flags;
            }
            self.set_oam_helper0_at(
                oam,
                info_x,
                info_y.wrapping_add(BUSH_SOLDIER_COMMON_DRAW_Y_OFFSETS[j] as i16 as u16),
                BUSH_SOLDIER_COMMON_DRAW_CHARS[j],
                flags,
                2,
            );
            oam += 4;
        }
    }
}
