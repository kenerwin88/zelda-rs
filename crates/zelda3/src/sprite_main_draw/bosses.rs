//! Agahnim, Helmasaur, Moldorm, Lanmolas, and other boss draw routines.

use super::*;
use crate::zelda_rtl::sprite::PrepOamCoordsRet;
use crate::zelda_rtl::sprite::SpriteSpawnInfo;
use crate::types::sign8;
use crate::types::SpriteHitBox;

impl ZeldaState {

    // -----------------------------------------------------------------------
    // void SpriteDraw_TrinexxRockHead(int k, PrepOamCoordsRet *info) {  // 9db560
    //   sprite_main.c:16348 — 36-entry head OAM block.
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_trinexx_rock_head(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        if !sign8(self.sprite_slot_view(k).ai_state()) {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
        }
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        let base = base.min(TRINEXX_DRAW1_DRAW_FRAMES.len() - 4);
        // Sprite_DrawMultiple(... info) writes the out-pointer before returning
        // early for offscreen sprites. Trinexx body drawing still consumes those
        // flags, so preserve the populated tuple even when no head OAM is drawn.
        let (prepped, out_of_bounds) = self.sprite_prep_oam_coord_or_double_ret_with_out_flag_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet);
        *info = PrepOamCoordsRet::from_tuple(prepped);
        if out_of_bounds {
            return;
        }
        // Manually emit the four DrawMultipleData entries so we don't have to thread
        // a mutable canonical-struct reference here.
        self.sprite_draw_multiple_with_info(k, &TRINEXX_DRAW1_DRAW_FRAMES[base..base + 4], prepped);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_TrinexxRockHeadAndBody(int k) {  // 9db587 — sprite_main.c:16395
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_trinexx_rock_head_and_body(&mut self, k: usize) {
        if sign8(self.sprite_slot_view(k).head_direction()) {
            return;
        }

        let mut info = PrepOamCoordsRet::default();
        self.sprite_draw_trinexx_rock_head(k, &mut info);
        info.flags &= !0x10;

        if self.sprite_slot_view(k).ai_state() == 3 {
            let mut oam = (self.game_state.oam.current_pointer_usize()) + 4 * 4;
            let xb = self
                .sprite_slot_view(k)
                .a()
                .wrapping_sub(self.sprite_slot_view(k).x_low());
            let yb = self
                .sprite_slot_view(k)
                .c()
                .wrapping_sub(self.sprite_slot_view(k).y_low());
            let mut i: i32 = 7;
            loop {
                let xv =
                    info.x
                        .wrapping_add(trinexx_mult_draw(xb, TRINEXX_SCALE_MULTIPLIERS[i as usize])
                            as i8 as i16 as u16);
                let yv =
                    info.y
                        .wrapping_add(trinexx_mult_draw(yb, TRINEXX_SCALE_MULTIPLIERS[i as usize])
                            as i8 as i16 as u16);
                self.set_oam_plain_at(oam, xv as u8, yv as u8, 0x28, info.flags, 2);
                oam += 4;
                i -= 1;
                if i < 0 {
                    break;
                }
            }
            self.sprite_workspace_mut().set_shared_scratch_a(0x30);
        }
        self.oam_state_mut().set_current_pointer(0x9f0);
        self.oam_state_mut().set_current_extended_pointer(0xa9c);

        let mut oam = self.game_state.oam.current_pointer_usize();
        let xb = self
            .sprite_slot_view(k)
            .a()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2() as u8);
        let yb = ((self.sprite_slot_view(k).c() as u16)
            | ((self.sprite_slot_view(k).y_high() as u16) << 8))
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());

        let xvel = self.sprite_slot_view(k).x_velocity() as i8;
        let xidx: u8 = if (xvel.wrapping_add(3) as u8) < 7 {
            0
        } else {
            self.sprite_slot_view(k).subtype2() >> 2
        };
        let yidx: u8 = self.sprite_slot_view(k).subtype2() >> 2;

        let mut i: i32 = 1;
        loop {
            let xshift: u8 = if i != 0 { 0u8.wrapping_sub(28) } else { 28 };
            let off_x = TRINEXX_WIGGLE_X_OFFSETS
                [(xidx.wrapping_add(((1 - i) as u8).wrapping_mul(8)) & 0xf) as usize];
            let off_y = TRINEXX_WIGGLE_Y_OFFSETS
                [(yidx.wrapping_add((i as u8).wrapping_mul(8)) & 0xf) as usize];
            let x = xb.wrapping_add(xshift).wrapping_add(off_x as u8);
            let y = yb.wrapping_sub(8).wrapping_add(off_y as i16 as u16) as u8;
            let f = info.flags | (if i != 0 { 0 } else { 0x40 });
            self.set_oam_plain_at(oam, x, y, 0xc, f, 2);
            self.set_oam_plain_at(oam + 4, x, y.wrapping_add(16), 0x2a, f, 2);
            oam += 8;
            i -= 1;
            if i < 0 {
                break;
            }
        }

        // oam = (OamEnt *)&g_ram[0x800] + 91;
        let mut oam = OAM_BUF + 91 * 4;
        let g = self.overlord_slot_view(2).x_low() as usize;
        let ov7 = u16::from(self.overlord_slot_view(7).x_low())
            | (u16::from(self.overlord_slot_view(8).x_low()) << 8);
        for (i, &chr) in TRINEXX_DRAW_CHARS.iter().enumerate() {
            let j = g * 5 + i;
            let jc = j.min(TRINEXX_DRAW_X_OFFSETS.len() - 1);
            let x = xb.wrapping_add(TRINEXX_DRAW_X_OFFSETS[jc] as u8);
            let y = yb
                .wrapping_sub(TRINEXX_DRAW_Y_OFFSETS[jc] as u16)
                .wrapping_sub(0x20)
                .wrapping_add(ov7);
            self.set_oam_helper0_at(oam, x as u16, y, chr, info.flags, 2);
            oam += 4;
        }
        self.temp_counter_mut().set(0xff);

        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries(k, 3, 2);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_CutsceneAgahnimSpell(int k, PrepOamCoordsRet *info) {  // 9dd516
    //   sprite_main.c:17829.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_cutscene_agahnim_spell(&mut self, k: usize, info: &PrepOamCoordsRet) {
        // 28-entry table compiled below. Layout: (x:i8, y:i8, charnum, flags).
        self.oam_allocate_from_region_a(0x38);
        let mut data_off: usize = 0;
        if self.game_state.frame.frame_counter & 2 == 0 {
            data_off += 14;
        }
        if self.sprite_slot_view(k).subtype2() == 0 {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let kn = self.sprite_slot_view(k).subtype2().wrapping_sub(1);
        let end = self.sprite_slot_view(k).subtype();
        let t = end.wrapping_add(1) as usize;
        let mut cur_oam = oam + t * 4;
        let mut data_idx = data_off + t;
        let mut big_idx = t;
        let mut kn_var = kn;
        loop {
            let (dx, dy, ch, fl) = CHATTY_AGAHNIM_TELEWARP_DRAW_DATA[data_idx % 28];
            let big = TELEWARP_DRAW_SIZES[big_idx % 14];
            let x = info.x.wrapping_add(dx as i16 as u16) as u8;
            let y = info.y.wrapping_add(dy as i16 as u16).wrapping_sub(8) as u8;
            self.set_oam_plain_at(cur_oam, x, y, ch, fl | 0x31, big);
            data_idx = data_idx.wrapping_add(1);
            big_idx = big_idx.wrapping_add(1);
            cur_oam += 4;
            kn_var = kn_var.wrapping_sub(1);
            if kn_var == end {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_AltarZeldaWarp(int k) {  // 9dd6b1 — sprite_main.c:17912
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_altar_zelda_warp(&mut self, k: usize) {
        self.oam_allocate_from_region_a(8);
        let base = ((self.sprite_slot_view(k).delay_main() >> 2) as usize) * 2;
        let base = base.min(ALTAR_ZELDA_WARP_DRAW_FRAMES.len() - 2);
        self.sprite_draw_multiple(k, &ALTAR_ZELDA_WARP_DRAW_FRAMES[base..base + 2]);
    }

    pub(super) fn sprite_cutscene_agahnim_zelda(&mut self, k: usize) {
        let j = self.sprite_slot_view(k).delay_main();
        if j != 0 {
            self.sprite_draw_altar_zelda_warp(k);
            if j == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if j < 12 {
                return;
            }
        }
        self.oam_allocate_from_region_a(8);
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let base = base.min(ALTAR_ZELDA_DRAW_FRAMES.len() - 2);
        let info = self.sprite_draw_multiple(k, &ALTAR_ZELDA_DRAW_FRAMES[base..base + 2]);
        let local_info = PrepOamCoordsRet {
            x: info.x,
            y: info.y,
            r4: info.r4,
            flags: info.flags,
        };
        self.altar_zelda_draw_body(k, &local_info);
    }

    // -----------------------------------------------------------------------
    // void Sprite_C1_CutsceneAgahnim(int k) {  // 9dd234
    pub(super) fn sprite_c1_cutscene_agahnim(&mut self, k: usize) {
        match self.sprite_slot_view(k).a() {
            0 => self.cutscene_agahnim_agahnim(k),
            1 => self.sprite_cutscene_agahnim_zelda(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void CutsceneAgahnim_Agahnim(int k) {  // 9dd23f
    pub(super) fn cutscene_agahnim_agahnim(&mut self, k: usize) {
        let mut info = PrepOamCoordsRet::default();
        if self.sprite_slot_view(k).c() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if (self.sprite_slot_view(k).delay_main() & 1) == 0 {
                self.chatty_agahnim_draw(k, &mut info);
            }
            return;
        }

        self.chatty_agahnim_draw(k, &mut info);
        self.sprite_draw_cutscene_agahnim_spell(k, &info);
        if self.sprite_slot_view(k).pause() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.follower_link_state_mut().immobilize();
                    self.dialogue_message_index_mut().set_value(0x013d);
                    self.sprite_show_message_minimal_c();
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_b(1);
                let j = self.sprite_slot_view(k).b();
                let value = if self.sprite_slot_view(15).z() < 16 {
                    LEVITATE_GFX[usize::from((j >> 5) & 3)]
                } else {
                    1
                };
                self.sprite_slot_view_mut(k).set_graphics(value);
                if (j & 15) == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(15).set_graphics(value);
                    self.sprite_slot_view_mut(15).add_z(1);
                    if self.sprite_slot_view(15).z() == 22 {
                        self.set_sound_effect_2(0x27);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_subtype(value);
                    }
                }
            }
            2 => {
                let delay = self.sprite_slot_view(k).delay_main();
                if delay == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else if delay == 120 {
                    self.attract_scene_mut().set_intro_palette_flash_count(120);
                } else if delay < 128 && (delay & 3) == 0 {
                    self.set_sound_effect_2(0x2b);
                    if self.sprite_slot_view(k).subtype2() != 14 {
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(4);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                    }
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    if (self.sprite_slot_view(k).delay_main() & 3) == 0
                        && self.sprite_slot_view(k).subtype() != 9
                    {
                        self.sprite_slot_view_mut(k).add_subtype(2);
                    }
                } else {
                    let value = 19;
                    self.sprite_slot_view_mut(15).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_subtype2(value);
                    self.set_sound_effect_1(0x33);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.dialogue_message_index_mut().set_value(0x013e);
                    self.sprite_show_message_minimal_c();
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            5 => {
                if self.sprite_slot_view(k).delay_main() == 1 {
                    self.set_sound_effect_2(0x28);
                }
                let value = (-32i8) as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.sprite_move_y(k);
                if self.sprite_slot_view(k).y_low() < 48 {
                    let value = 66;
                    self.sprite_slot_view_mut(k).set_delay_aux4(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
                self.sprite_agahnim_apply_motion_blur(k);
            }
            6 => {
                if self.sprite_slot_view(k).delay_aux4() == 0 {
                    self.follower_link_state_mut().clear_immobilized();
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    self.sprite_manually_set_death_flag_uw(k);
                    let bits =
                        self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x4000;
                    self.dungeon_savegame_state_mut()
                        .set_savegame_state_bits(bits);
                }
            }
            _ => {}
        }
    }

    pub(in crate::zelda_rtl) fn sprite_agahnim_apply_motion_blur(&mut self, k: usize) -> i32 {
        if self.game_state.frame.frame_counter & 3 != 0 {
            return -1;
        }
        if let Some(SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
            slot,
            spawned_slot,
            progress,
            bound: false,
        }) = self.sprite_main_cpu_boundary
        {
            if usize::from(slot) == k {
                // The source host stopped inside this spawn's shared helper:
                // publish through the checkpoint and park; the resumed host
                // finishes the spawn, the afterimage setup, and the caller's
                // suffix (which sees no spawned slot on this host).
                let mut info = SpriteSpawnInfo::default();
                self.sprite_spawn_dynamically_selected_prefix(
                    k,
                    0xc1,
                    &mut info,
                    usize::from(spawned_slot),
                    progress,
                );
                self.sprite_main_cpu_boundary =
                    Some(SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
                        slot,
                        spawned_slot,
                        progress,
                        bound: true,
                    });
                return -1;
            }
        }
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc1, &mut info);
        if j >= 0 {
            self.agahnim_motion_blur_after_spawn(k, j as usize, &info);
        }
        j
    }

    /// `Sprite_Agahnim_ApplyMotionBlur` after its spawn selected `j`.
    fn agahnim_motion_blur_after_spawn(&mut self, k: usize, ju: usize, info: &SpriteSpawnInfo) {
        self.sprite_set_spawned_coordinates(ju, info);
        let value = self.sprite_slot_view(k).graphics();
        self.sprite_slot_view_mut(ju).set_graphics(value);
        let value = 32;
        self.sprite_slot_view_mut(ju).set_delay_main(value);
        let value = 32;
        self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
        let value = 32;
        self.sprite_slot_view_mut(ju).set_c(value);
    }

    /// Resumes an Agahnim motion-blur spawn from `progress`: the rest of the
    /// shared spawn helper, the afterimage setup, and the clone-dash caller's
    /// `sprite_oam_flags[j] = 4` (Sprite_7A_Agahnim state 7 with a running
    /// anim clock is the only caller that reads the spawned slot).
    pub(in crate::zelda_rtl) fn resume_agahnim_motion_blur_spawn(
        &mut self,
        k: usize,
        spawned: usize,
        progress: crate::SpriteDynamicSpawnProgress,
    ) {
        let mut info = SpriteSpawnInfo::default();
        self.sprite_spawn_dynamically_selected_from(k, &mut info, spawned, progress);
        self.agahnim_motion_blur_after_spawn(k, spawned, &info);
        if self.sprite_slot_view(k).sprite_type() == 0x7a
            && self.sprite_slot_view(k).ai_state() == 7
            && self.sprite_slot_view(k).anim_clock() != 0
        {
            self.sprite_slot_view_mut(spawned).set_oam_flags(4);
        }
    }

    pub(super) fn agahnim_perform_attack(&mut self, k: usize) {
        if k == 0 {
            self.sprite_slot_view_mut(k).add_subtype(1);
            if self.game_state.world.region.is_in_dark_world() {
                self.sprite_slot_view_mut(k).and_subtype(3);
            }
        }
        if self.sprite_slot_view(k).subtype() == 5 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_subtype(value);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
            for _ in 0..4 {
                self.sprite_spawn_lightning(k);
            }
        } else {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x7b, &mut info);
            if j >= 0 {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x29);
                let ju = j as usize;
                let i = usize::from(self.sprite_slot_view(k).direction())
                    .min(AGAHNIM_SHADOW_X_OFFSETS.len() - 1);
                self.sprite_set_x(
                    ju,
                    info.r0_x
                        .wrapping_add(AGAHNIM_SHADOW_X_OFFSETS[i] as i16 as u16),
                );
                self.sprite_set_y(
                    ju,
                    info.r2_y
                        .wrapping_add(AGAHNIM_SHADOW_Y_OFFSETS[i] as i16 as u16),
                );
                let value = self.sprite_slot_view(ju).y_high();
                self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
                let value = self.sprite_slot_view(k).x_velocity();
                self.sprite_slot_view_mut(ju).set_x_velocity(value);
                let value = self.sprite_slot_view(k).y_velocity();
                self.sprite_slot_view_mut(ju).set_y_velocity(value);
                if self.sprite_slot_view(k).subtype() >= 2 && (self.get_random_number() & 1) == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(ju).set_b(value);
                    let value = 32;
                    self.sprite_slot_view_mut(ju).set_delay_main(value);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Agahnim_Draw(int k) {  // 9ed978
    pub(super) fn agahnim_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics()).min(17);
        for i in (0..=3).rev() {
            let j = g * 4 + i;
            self.set_oam_plain_at(
                oam,
                info_x.wrapping_add(AGAHNIM_DRAW_X_OFFSETS_0[j] as i16 as u16) as u8,
                info_y.wrapping_add(AGAHNIM_DRAW_Y_OFFSETS_0[j] as i16 as u16) as u8,
                AGAHNIM_DRAW_CHARS_0[j],
                info_flags | AGAHNIM_DRAW_FLAGS_0[j],
                if (0x40..0x44).contains(&j) { 0 } else { 2 },
            );
            oam += 4;
        }
        if g < 12 {
            let mut shadow_info = PrepOamCoordsRet {
                x: info_x,
                y: info_y,
                r4: 0,
                flags: info_flags,
            };
            self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
        }
        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries(k, 3, 0xff);
        }

        let Some((info_x, info_y, _info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet)
        else {
            return;
        };
        if self.sprite_slot_view(k).direction() != 0 {
            self.oam_allocate_from_region_c(8);
        } else {
            self.oam_allocate_from_region_b(8);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        if self.sprite_slot_view(k).head_direction() == 0 {
            return;
        }
        let g = usize::from(self.sprite_slot_view(k).head_direction().wrapping_sub(1)).min(35);
        let flags =
            (((self.game_state.frame.frame_counter >> 1) & 2).wrapping_add(2)).wrapping_add(0x31);
        for i in (0..=1).rev() {
            let j = g * 2 + i;
            self.set_oam_plain_at(
                oam,
                info_x.wrapping_add(AGAHNIM_DRAW_X_OFFSETS_1[j] as i16 as u16) as u8,
                info_y.wrapping_add(AGAHNIM_DRAW_Y_OFFSETS_1[j] as i16 as u16) as u8,
                AGAHNIM_DRAW_CHARS_1[g],
                flags,
                AGAHNIM_DRAW_BIG_1[g],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_7A_Agahnim(int k) {  // 9ed330
    pub(super) fn sprite_7_a_agahnim(&mut self, k: usize) {
        self.agahnim_draw(k);
        if self.sprite_slot_view(k).pause() != 0 {
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = 3;
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let value = 1;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = SPRITE_7_A_AGAHNIM_START_STATE
                    [usize::from(self.game_state.world.region.is_in_dark_world())];
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.dialogue_message_index_mut().set_value(0x013f);
                    self.sprite_show_message_minimal_c();
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            2 => {
                self.sprite_workspace_mut().clear_agahnim_phase_scratch();
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_7_A_AGAHNIM_GRAPHICS_0
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3).min(4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                let delay = self.sprite_slot_view(k).delay_main();
                if delay == 192 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x27);
                }
                if !(16..239).contains(&delay) {
                    let filter_k = if self.game_state.world.region.is_in_dark_world() {
                        k
                    } else {
                        2
                    };
                    self.AgahnimWarpShadowFilter(filter_k);
                } else if k == 0 {
                    self.sprite_check_damage_to_and_from_link(k);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 39;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return;
                }
                if self.sprite_slot_view(k).delay_main() >= 128 {
                    self.sprite_apply_speed_towards_link(k, 2);
                    let idx = ((self.sprite_slot_view(k).y_velocity() as i8 as i32 + 2) * 5
                        + 2
                        + self.sprite_slot_view(k).x_velocity() as i8 as i32)
                        .clamp(0, 24) as usize;
                    let value = SPRITE_7_A_AGAHNIM_DIRECTIONS[idx];
                    self.sprite_slot_view_mut(k).set_direction(value);
                    self.sprite_apply_speed_towards_link(k, 32);
                    if self.sprite_slot_view(k).subtype() == 4 {
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                } else if self.sprite_slot_view(k).delay_main() == 112 {
                    self.agahnim_perform_attack(k);
                }
                let j = usize::from(self.sprite_slot_view(k).delay_main() >> 4);
                let value = ATTACK_POSE_OFFSETS_BY_DELAY[j];
                self.sprite_slot_view_mut(k).set_a(value);
                let t = HEAD_TURN_STEPS_BY_DELAY[j];
                let value = if t != 0 {
                    t.wrapping_add(
                        HEAD_TURN_DIRECTION_BASES
                            [usize::from(self.sprite_slot_view(k).direction()).min(5)],
                    )
                } else {
                    t
                };
                self.sprite_slot_view_mut(k).set_head_direction(value);
                let value = SPRITE_7_A_AGAHNIM_GRAPHICS_1
                    [usize::from(self.sprite_slot_view(k).direction()).min(5)]
                    + self.sprite_slot_view(k).a();
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            4 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let j = if self.sprite_slot_view(k).subtype() == 4 {
                        4
                    } else {
                        self.get_random_number() & 0x0f
                    };
                    let value = TELEPORT_TARGET_X_LOW[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = TELEPORT_TARGET_Y_LOW[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_e(value);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_g(value);
                } else {
                    let value = SPRITE_7_A_AGAHNIM_SECONDARY_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3).min(4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            5 => {
                if self
                    .sprite_slot_view(k)
                    .x_low()
                    .wrapping_sub(self.sprite_slot_view(k).c())
                    .wrapping_add(7)
                    < 14
                    && self
                        .sprite_slot_view(k)
                        .y_low()
                        .wrapping_sub(self.sprite_slot_view(k).e())
                        .wrapping_add(7)
                        < 14
                {
                    let value = self.sprite_slot_view(k).c();
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value = self.sprite_slot_view(k).e();
                    self.sprite_slot_view_mut(k).set_y_low(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 39;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return;
                }
                let x = (u16::from(self.sprite_slot_view(k).x_high()) << 8)
                    | u16::from(self.sprite_slot_view(k).c());
                let y = (u16::from(self.sprite_slot_view(k).y_high()) << 8)
                    | u16::from(self.sprite_slot_view(k).e());
                let pt = self.sprite_project_speed_towards_location(
                    k,
                    x,
                    y,
                    self.sprite_slot_view(k).g(),
                );
                let value = pt.y;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                if self.sprite_slot_view(k).g() < 64 {
                    self.sprite_slot_view_mut(k).add_g(1);
                }
                self.sprite_move_xy(k);
            }
            6 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.dialogue_message_index_mut().set_value(0x0141);
                    self.sprite_show_message_minimal_c();
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            7 => {
                if self.sprite_slot_view(k).anim_clock() != 0 {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let idx = k.saturating_sub(1).min(1);
                        let value = CLONE_DASH_X_VELOCITIES[idx] as u8;
                        self.sprite_slot_view_mut(k).set_x_velocity(value);
                        self.sprite_slot_view_mut(k).add_y_velocity(2);
                        self.sprite_move_xy(k);
                        let j = self.sprite_agahnim_apply_motion_blur(k);
                        if j >= 0 {
                            let value = 4;
                            self.sprite_slot_view_mut(j as usize).set_oam_flags(value);
                        }
                    }
                } else if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else if self.sprite_slot_view(k).delay_main() == 64 {
                    self.set_sound_effect_2(0x28);
                    self.temp_counter_mut().set(1);
                    loop {
                        let mut info = SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically_ex(k, 0x7a, &mut info, 2);
                        if j >= 0 {
                            let ju = j as usize;
                            self.sprite_set_spawned_coordinates(ju, &info);
                            let value = CLONE_FLAGS3_BY_SLOT[ju.saturating_sub(1).min(1)];
                            self.sprite_slot_view_mut(ju).set_flags3(value);
                            let value = self.sprite_slot_view(ju).flags3() & 15;
                            self.sprite_slot_view_mut(ju).set_anim_clock(value);
                            let value = self.sprite_slot_view(ju).anim_clock();
                            self.sprite_slot_view_mut(ju).set_oam_flags(value);
                            let value = self.sprite_slot_view(k).ai_state();
                            self.sprite_slot_view_mut(ju).set_ai_state(value);
                            let value = 32;
                            self.sprite_slot_view_mut(ju).set_delay_main(value);
                        }
                        self.temp_counter_mut().decrement();
                        if sign8(self.game_state.scratch_counter.value()) {
                            break;
                        }
                    }
                }
            }
            8 => {
                self.follower_link_state_mut().set_menu_block_flag(2);
                let value = 0;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                if self.sprite_slot_view(k).delay_main() >= 64 {
                    self.sprite_slot_view_mut(k).or_hit_timer(0xe0);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 1 {
                        self.sprite_spawn_phantom_ganon(k);
                        self.set_music_control(0x1d);
                    }
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_hit_timer(value);
                    let value = 17;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            9 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                let x = self.sprite_get_x(0);
                let y = self.sprite_get_y(0);
                let cur_x = self.sprite_get_x(k);
                let cur_y = self.sprite_get_y(k);
                if cur_x.wrapping_sub(x).wrapping_add(4) < 8
                    && cur_y.wrapping_sub(y).wrapping_add(4) < 8
                {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
                let pt = self.sprite_project_speed_towards_location(k, x, y, 0x20);
                let value = pt.y;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                self.sprite_move_xy(k);
                self.sprite_agahnim_apply_motion_blur(k);
            }
            10 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    self.prepare_dungeon_exit_from_boss_fight();
                }
                if self.sprite_slot_view(k).delay_main() < 16 {
                    self.set_color_math_control(0x7f);
                    self.set_main_screen_layers(6);
                    self.set_sub_screen_layers(0x10);
                    self.palette_filter_sp5f();
                }
                if self.sprite_slot_view(k).z_velocity() != 0xff {
                    self.sprite_slot_view_mut(k).add_z_velocity(1);
                }
                let z = u16::from(self.sprite_slot_view(k).z_subpixel())
                    + u16::from(self.sprite_slot_view(k).z_velocity());
                let value = z as u8;
                self.sprite_slot_view_mut(k).set_z_subpixel(value);
                if z & 0x100 != 0 {
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    if self.sprite_slot_view(k).subtype2() == 7 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x04);
                    }
                }
                let value = SPRITE_7_A_AGAHNIM_GRAPHICS_3
                    [usize::from(self.sprite_slot_view(k).subtype2()).min(6)];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    pub(super) fn sprite_7_b_agahnim_balls(&mut self, k: usize) {
        if self.sprite_slot_view(k).b() != 0 {
            if self.sprite_slot_view(k).delay_main() != 0 {
                self.sprite_apply_speed_towards_link(k, 32);
            }
            let value = 5;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let value = ((self.game_state.frame.frame_counter >> 1) & 2).wrapping_add(3);
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        }

        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).graphics() != 2 {
                self.sprite_draw_single_large(k);
            } else {
                self.sprite_draw_single_small(k);
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = self.sprite_slot_view(k).delay_main();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            } else if self.sprite_slot_view(k).delay_main() == 6 {
                let value = 64;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 64;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.sprite_move_xy(k);
            }
            let value = ENERGY_BALL_GRAPHICS[self.sprite_slot_view(k).delay_main() as usize];
            self.sprite_slot_view_mut(k).set_graphics(value);
            return;
        }

        if self.sprite_slot_view(k).b() != 0 {
            self.seeker_energy_ball_draw(k);
        } else {
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            if self.sprite_slot_view(k).b() != 0 {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
                self.create_six_blue_balls(k);
                return;
            }
        }

        if self.sprite_slot_view(k).a() != 0 && self.sprite_slot_view(0).ignore_projectile() == 0 {
            let mut hb = SpriteHitBox {
                r0_xlo: self.sprite_slot_view(k).x_low(),
                r8_xhi: self.sprite_slot_view(k).x_high(),
                r1_ylo: self.sprite_slot_view(k).y_low(),
                r9_yhi: self.sprite_slot_view(k).y_high(),
                r2: 15,
                r3: 15,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.sprite_setup_hit_box(0, &mut hb);
            if self.check_if_hit_boxes_overlap(&hb) {
                self.sprite_give_damage(0, 16, 0xa0);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                let x_velocity = self.sprite_slot_view(k).x_velocity();
                let y_velocity = self.sprite_slot_view(k).y_velocity();
                self.sprite_slot_view_mut(0).set_x_recoil(x_velocity);
                self.sprite_slot_view_mut(0).set_y_recoil(y_velocity);
            }
        } else {
            self.sprite_check_damage_to_link(k);
            if (self.sprite_check_damage_from_link(k) & PLAYER_DAMAGE_CARRY_MASK_DRAW) != 0 {
                if self.sprite_slot_view(k).b() != 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
                    self.create_six_blue_balls(k);
                    return;
                }
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x05);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x29);
                self.sprite_apply_speed_towards_link(k, 0x30);
                let value = 0u8.wrapping_sub(self.sprite_slot_view(k).x_velocity());
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0u8.wrapping_sub(self.sprite_slot_view(k).y_velocity());
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.sprite_slot_view_mut(k).add_a(1);
            }
        }

        if ((((k as u8) ^ self.game_state.frame.frame_counter) & 3) | self.sprite_slot_view(k).b())
            != 0
        {
            return;
        }
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x7b, &mut info);
        if j < 0 {
            return;
        }
        let ju = j as usize;
        self.sprite_set_spawned_coordinates(ju, &info);
        let value = 15;
        self.sprite_slot_view_mut(ju).set_delay_main(value);
        let value = 15;
        self.sprite_slot_view_mut(ju).set_ai_state(value);
        let value = self.sprite_slot_view(k).b();
        self.sprite_slot_view_mut(ju).set_b(value);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Moldorm_Head(int k) {  // 9dd993 — sprite_main.c:17976
    // -----------------------------------------------------------------------
    pub(super) fn sprite_09_giant_moldorm(&mut self, k: usize) {
        self.giant_moldorm_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).ai_state() == 3 {
            if self.sprite_slot_view(k).delay_aux4() == 0 {
                let value = 4;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_a(value);
                let value = 224;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else {
                let value = self.sprite_slot_view(k).delay_aux4() | 224;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
            }
            return;
        }

        self.sprite_check_damage_from_link(k);
        let low_health = self.sprite_slot_view(k).health() < 3;
        let value = self
            .sprite_slot_view(k)
            .subtype2()
            .wrapping_add(if low_health { 2 } else { 1 });
        self.sprite_slot_view_mut(k).set_subtype2(value);
        if (self.game_state.frame.frame_counter & if low_health { 3 } else { 7 }) == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x31);
        }

        if self.sprite_slot_view(k).f() != 0 {
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_aux2(value);
            if (self.game_state.frame.frame_counter & 3) == 0 {
                self.sprite_slot_view_mut(k).subtract_f(1);
            }
            return;
        }

        if self.game_state.player.follower_link.incapacitated_timer() == 0
            && self.sprite_check_damage_to_link(k)
        {
            self.link_cancel_dash();
            let pt = self.sprite_project_speed_towards_link(k, 0x28);
            self.follower_link_state_mut()
                .set_actual_velocity_xy(pt.x, pt.y);
            self.follower_link_state_mut().set_incapacitated_timer(24);
            let value = 48;
            self.sprite_slot_view_mut(k).set_delay_aux1(value);
            let effect = if self
                .game_state
                .enhanced_features
                .has(FEATURE_MISC_BUG_FIXES_DRAW)
            {
                0x32
            } else {
                0
            };
            self.set_sound_effect_2_with_sprite_pan(k, effect);
        }

        let j = usize::from(self.sprite_slot_view(k).direction()) + if low_health { 16 } else { 0 };
        let value = GIANT_MOLDORM_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = GIANT_MOLDORM_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            let value =
                GIANT_MOLDORM_NEXT_DIRECTIONS[self.sprite_slot_view(k).direction() as usize];
            self.sprite_slot_view_mut(k).set_direction(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let mut j = 1;
                    self.sprite_slot_view_mut(k).add_g(1);
                    if self.sprite_slot_view(k).g() == 3 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_g(value);
                        j = 2;
                    }
                    let value = j;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = (self.get_random_number() & 2).wrapping_sub(1);
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    // ROM Sprite_09_GiantMoldorm $1D:D821: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(31, 32);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Sprite_09_GiantMoldorm $1D:D832: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(15, 8);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else if (self.sprite_slot_view(k).delay_main() & 3) == 0 {
                    let value = self
                        .sprite_slot_view(k)
                        .direction()
                        .wrapping_add(self.sprite_slot_view(k).head_direction())
                        & 0x0f;
                    self.sprite_slot_view_mut(k).set_direction(value);
                }
            }
            2 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    self.sprite_apply_speed_towards_link(k, 0x1f);
                    let dir = Self::sprite_convert_velocity_to_angle(
                        self.sprite_slot_view(k).x_velocity(),
                        self.sprite_slot_view(k).y_velocity(),
                    )
                    .wrapping_sub(self.sprite_slot_view(k).direction());
                    if dir == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 48;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = self
                            .sprite_slot_view(k)
                            .direction()
                            .wrapping_add(if sign8(dir) { 0xff } else { 1 });
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn giant_moldorm_draw(&mut self, k: usize) {
        let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1dDoubleRet) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(prepped);
        let value = 11;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_moldorm_eyeballs(k, &info);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));

        let j = usize::from(self.sprite_slot_view(k).subtype2() & 0x7f);
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        self.moldorm_history_mut(j).set_position(x, y);

        self.sprite_draw_moldorm_head(k);
        if self.sprite_slot_view(k).b() < 4 {
            self.giant_moldorm_draw_segment_ab(k, 16);
            if self.sprite_slot_view(k).b() < 3 {
                self.giant_moldorm_draw_segment_ab(k, 28);
                if self.sprite_slot_view(k).b() < 2 {
                    self.sprite_draw_moldorm_segment_c(k);
                    if self.sprite_slot_view(k).b() == 0 {
                        self.moldorm_handle_tail(k);
                    }
                }
            }
        }
        self.giant_moldorm_incremental_segment_explosion(k);
        let cur_x = u16::from(self.sprite_slot_view(k).x_low())
            | (u16::from(self.sprite_slot_view(k).x_high()) << 8);
        let cur_y = u16::from(self.sprite_slot_view(k).y_low())
            | (u16::from(self.sprite_slot_view(k).y_high()) << 8);
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
    }

    pub(super) fn giant_moldorm_draw_segment_ab(&mut self, k: usize, lookback: i32) {
        let j =
            (i32::from(self.sprite_slot_view(k).subtype2()).wrapping_sub(lookback) & 0x7f) as usize;
        let cur_x = self
            .game_state
            .effects
            .sprite_histories
            .moldorm_history(j)
            .x();
        let cur_y = self
            .game_state
            .effects
            .sprite_histories
            .moldorm_history(j)
            .y();
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(0x10));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(4));
        let base = usize::from((self.sprite_slot_view(k).subtype2() >> 1) & 1) * 4;
        self.sprite_draw_multiple(k, &GIANT_MOLDORM_SEG_A_DRAW_FRAMES[base..base + 4]);
    }

    pub(super) fn giant_moldorm_draw_segment_c_or_tail(&mut self, k: usize, lookback: i32) {
        let j =
            (i32::from(self.sprite_slot_view(k).subtype2()).wrapping_sub(lookback) & 0x7f) as usize;
        let cur_x = self
            .game_state
            .effects
            .sprite_histories
            .moldorm_history(j)
            .x();
        let cur_y = self
            .game_state
            .effects
            .sprite_histories
            .moldorm_history(j)
            .y();
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
        let bak = self.sprite_slot_view(k).oam_flags();
        let value = (bak & 0x3f)
            | GIANT_MOLDORM_OAM_FLAGS[((self.sprite_slot_view(k).subtype2() >> 1) & 3) as usize];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        let value = bak;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    pub(super) fn sprite_draw_moldorm_head(&mut self, k: usize) {
        let t = ((self.sprite_slot_view(k).subtype2() >> 1) & 1)
            + (self.sprite_slot_view(k).delay_aux1() & 2);
        let base = (t as usize) * 4;
        let base = base.min(GIANT_MOLDORM_HEAD_DRAW_FRAMES.len() - 4);
        self.sprite_draw_multiple(k, &GIANT_MOLDORM_HEAD_DRAW_FRAMES[base..base + 4]);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Moldorm_SegmentC(int k) {  // 9dda5f — sprite_main.c:17999
    //   sprite_graphics[k] = 0;
    //   oam_cur_ptr += 0x10;
    //   oam_ext_cur_ptr += 4;
    //   GiantMoldorm_DrawSegment_C_OrTail(k, 0x28);
    // }
    pub(super) fn sprite_draw_moldorm_segment_c(&mut self, k: usize) {
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(0x10));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(4));
        self.giant_moldorm_draw_segment_c_or_tail(k, 0x28);
    }

    pub(super) fn moldorm_handle_tail(&mut self, k: usize) {
        self.sprite_draw_moldorm_tail(k);
        if self.sprite_slot_view(k).delay_aux2() == 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_flags4(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_deflection_bits(value);
            let oldx = self.sprite_get_x(k);
            let oldy = self.sprite_get_y(k);
            self.sprite_set_x(k, self.game_state.sprites.workspace.current_sprite_x());
            self.sprite_set_y(k, self.game_state.sprites.workspace.current_sprite_y());
            self.sprite_check_damage_from_link(k);
            let value = 0;
            self.sprite_slot_view_mut(k).set_a(value);
            let value = 9;
            self.sprite_slot_view_mut(k).set_flags4(value);
            let value = 4;
            self.sprite_slot_view_mut(k).set_deflection_bits(value);
            self.sprite_set_x(k, oldx);
            self.sprite_set_y(k, oldy);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Moldorm_Tail(int k) {  // 9ddb17 — sprite_main.c:18025
    //   oam_cur_ptr += 4;
    //   oam_ext_cur_ptr += 1;
    //   sprite_graphics[k]++;
    //   sprite_oam_flags[k] = 13;
    //   GiantMoldorm_DrawSegment_C_OrTail(k, 0x30);
    // }
    pub(super) fn sprite_draw_moldorm_tail(&mut self, k: usize) {
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_view_mut(k).add_graphics(1);
        let value = 13;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.giant_moldorm_draw_segment_c_or_tail(k, 0x30);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Moldorm_Eyeballs(int k, PrepOamCoordsRet *info) {  // 9ddb9e
    //   sprite_main.c:18033 — two eyeballs cycling around a 16-entry circle.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_moldorm_eyeballs(&mut self, k: usize, info: &PrepOamCoordsRet) {
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r7: u8 = if self.sprite_slot_view(k).f() != 0 {
            self.game_state.frame.frame_counter
        } else {
            0
        };
        let mut r6: i32 = (self.sprite_slot_view(k).direction() as i32).wrapping_sub(1);
        let mut i: i32 = 1;
        loop {
            let idx = (r6 & 0xf) as usize;
            let x = info.x.wrapping_add(GIANT_MOLDORM_EYE_X_OFFSETS[idx] as u16);
            let y = moldorm_eye_oam_y(info.y, GIANT_MOLDORM_EYE_Y_OFFSETS[idx] as u16);
            let cf_idx = ((r6 + r7 as i32) & 0xf) as usize;
            self.set_oam_helper1_at(
                oam,
                x,
                y,
                GIANT_MOLDORM_EYE_CHARS[cf_idx],
                info.flags | GIANT_MOLDORM_EYE_FLAGS[cf_idx],
                2,
            );
            oam += 4;
            r6 += 2;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Eyes(int k, PrepOamCoordsRet *info) {  // 9e856b
    //   sprite_main.c:19586.
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_king_helmasaur_eyes(&mut self, k: usize, info: &PrepOamCoordsRet) {
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(0x40));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(0x10));
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut i: i32 = 1;
        loop {
            let j = ((self.overlord_slot_view(4).x_low() >> 2) & 7) as usize;
            let x = info
                .x
                .wrapping_add(HELMASAUR_KING_MASK_X_OFFSETS[i as usize] as i16 as u16);
            let y = info.y.wrapping_add(0x14);
            self.set_oam_plain_at(
                oam,
                x as u8,
                y as u8,
                HELMASAUR_KING_MASK_CHARS[j],
                HELMASAUR_KING_MASK_FLAGS[i as usize],
                0,
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries(k, 1, 0);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Body(int k, PrepOamCoordsRet *info) {  // 9e87e5
    //   sprite_main.c:19661.
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_king_helmasaur_body(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        *info = self.sprite_draw_multiple(k, &HELMASAUR_KING_DRAW_D_DRAW_FRAMES);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Legs(int k, PrepOamCoordsRet *info) {  // 9e8805
    //   sprite_main.c:19686.
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_king_helmasaur_legs(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(19 * 4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(19));
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut i: i32 = 3;
        loop {
            let x = info
                .x
                .wrapping_add(HELMASAUR_KING_EXPLOSION_X_OFFSETS[i as usize] as i16 as u16)
                as u8;
            let y = info
                .y
                .wrapping_add(HELMASAUR_KING_EXPLOSION_Y_OFFSETS[i as usize] as i16 as u16)
                .wrapping_add(self.overlord_slot_view(i as usize).x_low() as u16)
                as u8;
            let f = HELMASAUR_KING_EXPLOSION_FLAGS[i as usize] ^ info.flags;
            self.set_oam_plain_at(
                oam,
                x,
                y,
                HELMASAUR_KING_EXPLOSION_CHARS[i as usize],
                f,
                2,
            );
            self.set_oam_plain_at(
                oam + 4,
                x,
                y.wrapping_add(16),
                HELMASAUR_KING_EXPLOSION_CHARS[i as usize].wrapping_add(2),
                f,
                2,
            );
            oam += 8;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.temp_counter_mut().set(0xff);
        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries(k, 7, 2);
            // Sprite_PrepOamCoordOrDoubleRet(k, info) refreshes the caller's coordinates.
            if let Some(p) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet) {
                info.x = p.0;
                info.y = p.1;
                info.flags = p.2;
                info.r4 = 0;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Mouth(int k, PrepOamCoordsRet *info) {  // 9e88bc
    //   sprite_main.c:19709.
    // -----------------------------------------------------------------------
    pub(in crate::zelda_rtl) fn sprite_draw_king_helmasaur_mouth(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).delay_aux2() == 0 {
            return;
        }
        let yd_idx = (self.sprite_slot_view(k).delay_aux2() >> 2) as usize;
        let yd = HELMASAUR_KING_FIREBALL_Y_OFFSETS[yd_idx];
        self.oam_allocate_from_region_b(4);
        let oam = self.game_state.oam.current_pointer_usize();
        let x = info.x as u8;
        let y = (info.y as u8).wrapping_add(yd).wrapping_add(0x13);
        self.set_oam_plain_at(oam, x, y, 0xaa, info.flags ^ 0xb, 2);
    }

    // void KingHelmasaurMask(int k, PrepOamCoordsRet *info) {  // 9e8686
    pub(in crate::zelda_rtl) fn king_helmasaur_mask(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
        if self.sprite_slot_view(k).c() >= 3 {
            return;
        }
        let start = self.sprite_slot_view(k).c() as usize * 8;
        *info = self.sprite_draw_multiple(
            k,
            &HELMASAUR_MASK_DRAW_FRAMES[start..start + 8],
        );
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(0x20));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(8));
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            return;
        }
        for i in (0..=1usize).rev() {
            let bomb = self.ancilla_slot_view(i);
            if bomb.ancilla_type() == 7 && (bomb.x_velocity() | bomb.y_velocity()) != 0 {
                self.king_helmasaur_check_bomb_damage(k, i);
            }
        }
    }

    // void KingHelmasaur_CheckBombDamage(int k, int j) {  // 9e86e5
    pub(super) fn king_helmasaur_check_bomb_damage(&mut self, k: usize, j: usize) {
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
        self.sprite_setup_hit_box(k, &mut hb);
        let bomb = self.ancilla_slot_view(j);
        let x = (((bomb.x_high() as u16) << 8) | bomb.x_low() as u16).wrapping_sub(6);
        let y = (((bomb.y_high() as u16) << 8) | bomb.y_low() as u16).wrapping_sub(bomb.z() as u16);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r2 = 2;
        hb.r3 = 15;
        if self.check_if_hit_boxes_overlap(&hb) {
            let x_velocity = 0u8.wrapping_sub(self.ancilla_slot_view(j).x_velocity());
            self.ancilla_slot_view_mut(j).set_x_velocity(x_velocity);
            let y_velocity =
                ((0u8.wrapping_sub(self.ancilla_slot_view(j).y_velocity()) as i8) >> 1) as u8;
            self.ancilla_slot_view_mut(j).set_y_velocity(y_velocity);
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
            self.garnish_state_mut().set_repulsespark_timer(5);
            let spark_x = self.ancilla_slot_view(j).x_low();
            self.garnish_state_mut().set_repulsespark_x_lo(spark_x);
            let spark_y = self
                .ancilla_slot_view(j)
                .y_low()
                .wrapping_sub(self.ancilla_slot_view(j).z());
            self.garnish_state_mut().set_repulsespark_y_lo(spark_y);
            self.set_sound_effect_1(5);
        }
    }

    // void KingHelmasaur_OperateTail(int k, PrepOamCoordsRet *info) {  // 9e8920
    pub(in crate::zelda_rtl) fn king_helmasaur_operate_tail(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        for (i, &radius_multiplier) in KING_HELMASAUR_OPERATE_TAIL_MULT_B.iter().enumerate() {
            let j = i + if self.sprite_slot_view(k).anim_clock() != 0 {
                16
            } else {
                0
            };
            let rs = self
                .overlord_slot_view(5)
                .gen1_word()
                .wrapping_add(self.overlord_slot_view(1).gen2_word());
            let f = ((rs >> 8) as u8).wrapping_sub(1);
            let abs_rs = if sign8(f) { 0u16.wrapping_sub(rs) } else { rs };
            let r6 = ((u16::from(abs_rs as u8)
                * u16::from(KING_HELMASAUR_OPERATE_TAIL_MULTIPLIERS[j]))
                >> 8) as u8;
            let angle = (rs & 0xff00)
                | if sign8(f) {
                    u16::from(r6 ^ 0xff)
                } else {
                    u16::from(r6)
                };
            let r15 = ((u16::from(self.overlord_slot_view(7).gen1())
                * u16::from(radius_multiplier))
                >> 8) as u8;
            let mut orbit = self.overlord_slot_view_mut(i + 5);
            orbit.set_x_low(helmasaur_sin(angle, r15) as u8);
            orbit.set_y_low(helmasaur_sin(angle.wrapping_add(0x80), r15).wrapping_sub(40) as u8);
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        let start = self.overlord_slot_view(3).gen2() as usize;
        let mut is_hit = false;
        for i in start..16usize {
            let orbit = self.overlord_slot_view(i + 5);
            let x = orbit.x_low().wrapping_add(info.x as u8);
            let y = orbit.y_low().wrapping_add(info.y as u8);
            self.oam_state_mut().write_entry(
                oam,
                x,
                y,
                if i == start { 0xe4 } else { 0xac },
                info.flags ^ 0x1b,
            );
            if self.game_state.player.follower_link.blink_countdown() == 0
                && self.sprite_slot_view(k).anim_clock() != 0
            {
                let link_x = (self.game_state.player.follower_link.x() as u8)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
                let link_y = (self.game_state.player.follower_link.y() as u8)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low())
                    .wrapping_add(8);
                if link_x.wrapping_sub(x).wrapping_add(12) < 24
                    && link_y.wrapping_sub(y).wrapping_add(8) < 16
                {
                    is_hit = true;
                    self.follower_link_state_mut().set_actual_velocity_xy(0, 56);
                }
            }
            oam += 4;
        }
        if is_hit && !self.game_state.player.follower_link.is_menu_blocked() {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
        self.sprite_correct_oam_entries(k, 16, 2);
        if let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet) {
            info.x = prepped.0;
            info.y = prepped.1;
            info.r4 = 0;
            info.flags = prepped.2;
        }
        self.temp_counter_mut().set(16);
    }

    // -----------------------------------------------------------------------
    // void Sprite_70_KingHelmasaurFireball(int k) {  // 85807f
    pub(super) fn sprite_70_king_helmasaur_fireball(&mut self, k: usize) {
        let oam = self.game_state.oam.current_pointer_usize();
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let flags = SPRITE_70_KING_HELMASAUR_FIREBALL_FLAGS
            [usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 1)];
        let x = self
            .sprite_slot_view(k)
            .x_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
        self.oam_state_mut().set_entry_x(oam, x);
        if x.wrapping_add(32) < 64 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        let y = self
            .sprite_slot_view(k)
            .y_low()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
        self.oam_state_mut().set_entry_y(oam, y);
        if y.wrapping_add(16) < 32 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        let charnum = SPRITE_70_KING_HELMASAUR_FIREBALL_CHARS
            [usize::from(self.sprite_slot_view(k).graphics())];
        self.oam_state_mut().set_entry_char(oam, charnum);
        self.oam_state_mut().set_entry_flags(oam, flags);
        let value = 2;
        self.oam_state_mut()
            .set_extended_byte((oam - OAM_BUF) / 4, value);

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                .wrapping_add(8)
                < 16
            && self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                .wrapping_add(16)
                < 16
        {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 18;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 36;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                self.sprite_slot_view_mut(k).subtract_y_velocity(2);
                self.sprite_move_y(k);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.helmasaur_fireball_tri_split(k);
                } else {
                    let value = SPRITE_70_KING_HELMASAUR_FIREBALL_LOCAL_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.helmasaur_fireball_quad_split(k);
                } else if self.sprite_slot_view(k).head_direction() < 20 {
                    self.sprite_slot_view_mut(k).add_head_direction(1);
                    self.sprite_move_xy(k);
                }
            }
            4 => self.sprite_move_xy(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void ArmosKnight_Draw(int k) {  // 85a274
    //   Four body tiles, plus the ground shadow when standing.
    // }
    pub(in crate::zelda_rtl) fn armos_knight_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank5DoubleRet) else {
            return;
        };
        if self.sprite_slot_view(k).a() == 0 && self.game_state.frame.submodule != 7 {
            self.oam_allocate_defer_to_player(k);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        for i in (0..4).rev() {
            let j = g * 4 + i;
            self.set_oam_helper0_at(
                oam,
                x.wrapping_add(ARMOS_KNIGHT_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(ARMOS_KNIGHT_DRAW_Y_OFFSETS[j] as i16 as u16),
                ARMOS_KNIGHT_DRAW_CHARS[j],
                ARMOS_KNIGHT_DRAW_FLAGS[j] | flags,
                ARMOS_KNIGHT_DRAW_SIZES[j],
            );
            oam += 4;
        }
        if g != 0 {
            return;
        }
        if self.sprite_slot_view(k).a() != 0 {
            let spr_idx = 76 + k * 2;
            self.oam_state_mut()
                .set_current_pointer(0x800 + (spr_idx as u16) * 4);
            self.oam_state_mut()
                .set_current_extended_pointer(0x0a20 + spr_idx as u16);
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let z = (self.sprite_slot_view(k).z().min(32)) >> 3;
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        self.set_oam_helper0_at(
            oam + 16,
            x.wrapping_sub(8).wrapping_add(z as u16),
            y.wrapping_add(12),
            0xe4,
            0x25,
            2,
        );
        self.set_oam_helper0_at(
            oam + 20,
            x.wrapping_add(8).wrapping_sub(z as u16),
            y.wrapping_add(12),
            0xe4,
            0x65,
            2,
        );
    }

    // -----------------------------------------------------------------------
    // void Kholdstare_Draw(int k) {  // 8dd98f
    pub(in crate::zelda_rtl) fn kholdstare_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Long) else {
            return;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.sprite_slot_view(k).a());
        self.set_oam_helper0_at(
            oam,
            info_x.wrapping_add(KHOLDSTARE_DRAW_X_OFFSETS[j] as i16 as u16),
            info_y.wrapping_add(KHOLDSTARE_DRAW_Y_OFFSETS[j] as i16 as u16),
            KHOLDSTARE_DRAW_CHARS[j],
            KHOLDSTARE_DRAW_FL[j] | info_flags,
            2,
        );
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple_with_info(
            k,
            &KHOLDSTARE_DRAW_FRAMES[base..base + 4],
            (info_x, info_y, info_flags),
        );
    }

    // -----------------------------------------------------------------------
    // void Moldorm_Draw(int k) {  // 9df822
    pub(super) fn moldorm_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Long) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut base = self.sprite_slot_view(k).direction().wrapping_sub(1);
        for _ in (0..2).rev() {
            let idx = usize::from(base & 0x0f);
            let y = moldorm_eye_oam_y(info_y, MOLDORM_DRAW_Y_OFFSETS[idx] as i16 as u16);
            self.set_oam_helper1_at(
                oam,
                info_x.wrapping_add(MOLDORM_DRAW_X_OFFSETS[idx] as i16 as u16),
                y,
                0x4d,
                info_flags,
                0,
            );
            oam += 4;
            base = base.wrapping_add(2);
        }
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));

        let j = (usize::from(self.sprite_slot_view(k).subtype2() & 0x1f)) + k * 32;
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        self.moldorm_history_mut(j).set_position(x, y);

        for i in (0..3).rev() {
            let j = usize::from(
                self.sprite_slot_view(k)
                    .subtype2()
                    .wrapping_add(MOLDORM_DRAW_GET_OFFS[i])
                    & 0x1f,
            ) + k * 32;
            let history = self.game_state.effects.sprite_histories.moldorm_history(j);
            let x = history
                .x()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                .wrapping_add(MOLDORM_DRAW_XY[i] as i16 as u16);
            let y = history
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_add(MOLDORM_DRAW_XY[i] as i16 as u16);
            self.set_oam_helper0_at(
                oam,
                x,
                y,
                MOLDORM_DRAW_CHARS[i],
                info_flags,
                MOLDORM_DRAW_BIG[i],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_54_Lanmolas(int k) {  // 85a3a2
    pub(in crate::zelda_rtl) fn sprite_54_lanmolas(&mut self, k: usize) {
        self.sprite_prep_oam_coord(k);
        self.lanmola_draw(k);
        self.sprite_54_lanmolas_after_draw(k);
    }

    fn sprite_54_lanmolas_after_draw(&mut self, k: usize) {
        if self.sprite_return_if_paused(k) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if (self.sprite_slot_view(k).delay_main() | self.sprite_slot_view(k).pause()) == 0 {
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x35);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.lanmola_spawn_shrapnel(k);
                    self.set_ambient_sound_effect(0x13);
                    let value =
                        SPRITE_54_LANMOLAS_RAND_B[usize::from(self.get_random_number() & 7)];
                    self.sprite_slot_view_mut(k).set_b(value);
                    let value =
                        SPRITE_54_LANMOLAS_RAND_C[usize::from(self.get_random_number() & 7)];
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 24;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_anim_clock(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_g(value);
                    self.lanmola_store_burrow_anchor(k);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_z(k);
                if self.sprite_slot_view(k).anim_clock() == 0 {
                    self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                    if self.sprite_slot_view(k).z_velocity() == 0 {
                        let value = self.sprite_slot_view(k).anim_clock().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_anim_clock(value);
                    }
                } else if (self.game_state.frame.frame_counter & 1) == 0 {
                    let j = usize::from(self.sprite_slot_view(k).g() & 1);
                    let value = self
                        .sprite_slot_view(k)
                        .z_velocity()
                        .wrapping_add(SPRITE_54_LANMOLAS_ZVEL[j] as u8);
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    if self.sprite_slot_view(k).z_velocity() == DESERT_BARRIER_X_VELOCITY_TARGETS[j]
                    {
                        self.sprite_slot_view_mut(k).add_g(1);
                    }
                }

                let x = self.sprite_get_x(k);
                let y = self.sprite_get_y(k);
                let x2 = (u16::from(self.sprite_slot_view(k).x_high()) << 8)
                    | u16::from(self.sprite_slot_view(k).b());
                let y2 = (u16::from(self.sprite_slot_view(k).y_high()) << 8)
                    | u16::from(self.sprite_slot_view(k).c());
                if x.wrapping_sub(x2).wrapping_add(2) < 4 && y.wrapping_sub(y2).wrapping_add(2) < 4
                {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                let pt = self.sprite_project_speed_towards_location(k, x2, y2, 10);
                let value = pt.y;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                self.sprite_move_xy(k);
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xy(k);
                self.sprite_move_z(k);
                if !sign8(self.sprite_slot_view(k).z_velocity().wrapping_add(20)) {
                    self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                }
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.lanmola_store_burrow_anchor(k);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value =
                        SPRITE_54_LANMOLAS_RAND_B[usize::from(self.get_random_number() & 7)];
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value =
                        SPRITE_54_LANMOLAS_RAND_C[usize::from(self.get_random_number() & 7)];
                    self.sprite_slot_view_mut(k).set_y_low(value);
                }
            }
            5 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    if self.sprite_check_if_screen_is_clear() {
                        let mut info = SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically(k, 0xea, &mut info);
                        debug_assert!(j >= 0);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            let value = 32;
                            self.sprite_slot_view_mut(j).set_z_velocity(value);
                            let value = 3;
                            self.sprite_slot_view_mut(j).set_a(value);
                        }
                    }
                }

                let delay = self.sprite_slot_view(k).delay_main();
                if (32..160).contains(&delay) && (delay & 15) == 0 {
                    let i = usize::from(
                        self.sprite_slot_view(k)
                            .subtype2()
                            .wrapping_sub(self.garnish_slot_view(k).y_low().wrapping_mul(8))
                            & 0x3f,
                    ) + k * 0x40;
                    let trail = self.lanmola_flat_trail_entry(i);
                    let xlo = trail
                        .x_low()
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
                    let ylo = trail
                        .y_low()
                        .wrapping_sub(trail.z_offset())
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x00, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.set_chr_halfslot_request(11);
                        let value = 4;
                        self.sprite_slot_view_mut(j).set_state(value);
                        let value = 31;
                        self.sprite_slot_view_mut(j).set_delay_main(value);
                        let value = 31;
                        self.sprite_slot_view_mut(j).set_a(value);
                        self.sprite_set_x(
                            j,
                            self.game_state
                                .display
                                .ppu_scroll_copy
                                .bg2_h_copy2()
                                .wrapping_add(u16::from(xlo)),
                        );
                        self.sprite_set_y(
                            j,
                            self.game_state
                                .display
                                .ppu_scroll_copy
                                .bg2_v_copy2()
                                .wrapping_add(u16::from(ylo)),
                        );
                        let value = 3;
                        self.sprite_slot_view_mut(j).set_flags2(value);
                        let value = 0x0c;
                        self.sprite_slot_view_mut(j).set_oam_flags(value);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                        if !sign8(self.garnish_slot_view(k).y_low()) {
                            let value = self.garnish_slot_view(k).y_low().wrapping_sub(1);
                            self.garnish_slot_view_mut(k).set_y_low(value);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn lanmola_store_burrow_anchor(&mut self, k: usize) {
        let value = self.sprite_slot_view(k).x_low();
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = self.sprite_slot_view(k).y_low();
        self.sprite_slot_view_mut(k).set_wall_collision(value);
        let value = 74;
        self.sprite_slot_view_mut(k).set_delay_aux1(value);
    }

    // -----------------------------------------------------------------------
    // void Lanmola_Draw(int k) {  // 85a64a
    pub(super) fn lanmola_draw(&mut self, k: usize) {
        let continuation = self.lanmola_draw_through_subtype2_increment(k);
        self.lanmola_draw_after_subtype2_increment(k, continuation);
    }

    pub(in crate::zelda_rtl) fn complete_lanmola_after_subtype2_increment(
        &mut self,
        k: usize,
        continuation: LanmolaDrawContinuation,
    ) {
        self.lanmola_draw_after_subtype2_increment(k, continuation);
        self.sprite_54_lanmolas_after_draw(k);
    }

    pub(in crate::zelda_rtl) fn lanmola_prep_and_draw_through_subtype2_increment(
        &mut self,
        k: usize,
    ) -> LanmolaDrawContinuation {
        self.sprite_prep_oam_coord(k);
        self.lanmola_draw_through_subtype2_increment(k)
    }

    pub(super) fn lanmola_draw_through_subtype2_increment(
        &mut self,
        k: usize,
    ) -> LanmolaDrawContinuation {
        self.lanmola_draw_set_oam_pointers(k);
        self.lanmola_draw_prefix_stores(k, 0, LANMOLA_DRAW_PREFIX_STORES);
        self.lanmola_draw_subtype2_increment(k)
    }

    fn lanmola_draw_set_oam_pointers(&mut self, k: usize) {
        let spr_offs = usize::from(LANMOLA_DRAW_SPR_OFFS[k]);
        self.oam_state_mut()
            .set_current_pointer((0x800 + spr_offs * 4) as u16);
        self.oam_state_mut()
            .set_current_extended_pointer((0x0a20 + spr_offs) as u16);
    }

    /// `Lanmola_Draw`'s prologue as its five source stores: the graphics
    /// angle, then the trail entry's direction, z offset, y and x bytes
    /// (the ROM pushes x, y, z and graphics and pops them in that order).
    pub(in crate::zelda_rtl) fn lanmola_draw_prefix_stores(&mut self, k: usize, from: u8, to: u8) {
        assert!(from <= to && to <= LANMOLA_DRAW_PREFIX_STORES);
        let j = k * 64 + usize::from(self.sprite_slot_view(k).subtype2());
        for store in from..to {
            match store {
                0 => {
                    let value = Self::sprite_convert_velocity_to_angle(
                        self.sprite_slot_view(k).x_velocity(),
                        self.sprite_slot_view(k)
                            .y_velocity()
                            .wrapping_sub(self.sprite_slot_view(k).z_velocity()),
                    );
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
                1 => {
                    let direction = self.sprite_slot_view(k).graphics();
                    self.lanmola_segment_motion_mut(j).set_direction(direction);
                }
                2 => {
                    let z_offset = self.sprite_slot_view(k).z();
                    self.lanmola_segment_motion_mut(j).set_z_offset(z_offset);
                }
                3 => {
                    // The flat trail for heads 2 and 3 lives past the
                    // 128-slot Moldorm model; store one byte, never a
                    // model-read companion (f1515314 zeroed head 2's Y trail).
                    let y_low = self.sprite_slot_view(k).y_low();
                    self.moldorm_history_mut(j).set_y_low(y_low);
                }
                4 => {
                    let x_low = self.sprite_slot_view(k).x_low();
                    self.moldorm_history_mut(j).set_x_low(x_low);
                }
                _ => unreachable!(),
            }
        }
    }

    fn lanmola_draw_subtype2_increment(&mut self, k: usize) -> LanmolaDrawContinuation {
        let r2 = self.sprite_slot_view(k).subtype2();
        let r5 = r2;
        if self.sprite_slot_view(k).state() == 9
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
        {
            let value = self.sprite_slot_view(k).subtype2().wrapping_add(1) & 63;
            self.sprite_slot_view_mut(k).set_subtype2(value);
        }
        LanmolaDrawContinuation { r2, r5 }
    }

    /// Runs `Sprite_54_Lanmolas` on the interrupted host through
    /// `Lanmola_Draw`'s first `completed_stores` prologue stores.
    pub(in crate::zelda_rtl) fn begin_lanmola_draw_prefix_checkpoint(&mut self, k: usize, completed_stores: u8) {
        assert!(completed_stores <= LANMOLA_DRAW_PREFIX_STORES);
        self.sprite_prep_oam_coord(k);
        self.lanmola_draw_set_oam_pointers(k);
        self.lanmola_draw_prefix_stores(k, 0, completed_stores);
    }

    pub(in crate::zelda_rtl) fn resume_lanmola_draw_prefix(&mut self, k: usize, completed_stores: u8) {
        self.lanmola_draw_prefix_stores(k, completed_stores, LANMOLA_DRAW_PREFIX_STORES);
        let continuation = self.lanmola_draw_subtype2_increment(k);
        self.complete_lanmola_after_subtype2_increment(k, continuation);
    }

    fn lanmola_draw_after_subtype2_increment(
        &mut self,
        k: usize,
        continuation: LanmolaDrawContinuation,
    ) {
        let mut r2 = continuation.r2;
        let mut r5 = continuation.r5;

        let r3 = self.sprite_slot_view(k).oam_flags() | self.sprite_slot_view(k).object_priority();
        let n = self.garnish_slot_view(k).y_low();
        if sign8(n) {
            return;
        }

        let reverse = sign8(self.sprite_slot_view(k).y_velocity());
        let mut oam = self.game_state.oam.current_pointer_usize() + if reverse { 7 * 4 } else { 0 };
        let oam_step: isize = if reverse { -4 } else { 4 };
        let mut i = n;
        loop {
            let hist = usize::from(r2) + k * 64;
            r2 = r2.wrapping_sub(8) & 63;
            let trail = self.lanmola_flat_trail_entry(hist);
            let entry_x = trail
                .x_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
            self.oam_state_mut().set_entry_x(oam, entry_x);
            if !sign8(trail.z_offset()) {
                let entry_y = trail
                    .y_low()
                    .wrapping_sub(trail.z_offset())
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
                self.oam_state_mut().set_entry_y(oam, entry_y);
            }
            let dir = usize::from(trail.direction());
            let charnum = if n != 7 || i != 0 {
                if n == i {
                    LANMOLA_DRAW_CHARS_1[dir]
                } else {
                    0xc6
                }
            } else {
                LANMOLA_DRAW_CHARS_0[dir]
            };
            self.oam_state_mut().set_entry_char(oam, charnum);
            self.oam_state_mut()
                .set_entry_flags(oam, LANMOLA_DRAW_FLAGS[dir] | r3);
            let value = 2;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam = (oam as isize + oam_step) as usize;
            i = i.wrapping_sub(1);
            if sign8(i) {
                break;
            }
        }

        oam = self.game_state.oam.current_pointer_usize() + 8 * 4;
        i = n;
        loop {
            let hist = usize::from(r5) + k * 64;
            r5 = r5.wrapping_sub(8) & 63;
            let trail = self.lanmola_flat_trail_entry(hist);
            let entry_x = trail
                .x_low()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
            self.oam_state_mut().set_entry_x(oam, entry_x);
            if !sign8(trail.z_offset()) {
                let entry_y = trail
                    .y_low()
                    .wrapping_add(10)
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
                self.oam_state_mut().set_entry_y(oam, entry_y);
            }
            self.oam_state_mut().set_entry_char(oam, 0x6c);
            self.oam_state_mut().set_entry_flags(oam, 0x34);
            let value = 2;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
            i = i.wrapping_sub(1);
            if sign8(i) {
                break;
            }
        }

        if self.sprite_slot_view(k).ai_state() == 1 {
            self.oam_allocate_from_region_b(4);
            let oam = self.game_state.oam.current_pointer_usize();
            let j = usize::from(
                LANMOLA_DRAW_BODY_FRAME_INDICES
                    [usize::from(self.sprite_slot_view(k).delay_main() >> 3).min(15)],
            );
            self.set_oam_plain_at(
                oam,
                self.sprite_slot_view(k)
                    .x_low()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low()),
                self.sprite_slot_view(k)
                    .y_low()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low()),
                LANMOLA_DRAW_BODY_CHARS[j],
                LANMOLA_DRAW_BODY_FLAGS[j] | 0x31,
                2,
            );
        } else if self.sprite_slot_view(k).ai_state() != 5
            && self.sprite_slot_view(k).delay_aux1() != 0
        {
            if (((self.sprite_slot_view(k).y_velocity() >> 6)
                ^ self.sprite_slot_view(k).ai_state())
                & 2)
                != 0
            {
                self.oam_allocate_from_region_b(8);
            } else {
                self.oam_allocate_from_region_c(8);
            }
            let mut oam = self.game_state.oam.current_pointer_usize();
            let r6 = usize::from((((self.sprite_slot_view(k).delay_aux1() >> 2) & 3) ^ 3) * 2);
            let x = self
                .sprite_slot_view(k)
                .direction()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
            let y = self
                .sprite_slot_view(k)
                .wall_collision()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
            for i in (0..=1).rev() {
                let j = i + r6;
                self.set_oam_plain_at(
                    oam,
                    x.wrapping_add(LANMOLA_DRAW_SPLASH_X_OFFSETS[j] as u8),
                    y.wrapping_add(LANMOLA_DRAW_SPLASH_Y_OFFSETS[j] as u8),
                    LANMOLA_DRAW_SPLASH_CHARS[j],
                    LANMOLA_DRAW_SPLASH_FLAGS[j] | 0x31,
                    LANMOLA_DRAW_SPLASH_SIZES[j],
                );
                oam += 4;
            }
        }
    }
}
