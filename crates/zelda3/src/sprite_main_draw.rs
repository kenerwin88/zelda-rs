//! Ported SpriteDraw_* helpers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source. The original C body
//! is reproduced as a comment block immediately above each port so a
//! reviewer can verify behavior line-by-line.
//!
//! Local `_for_draw` helpers at the bottom of the file adapt shared private
//! OAM helpers and overworld map-update bridges that this split module calls.
//!
//! `SpriteDraw_BombGuard_Arm` (sprite_main.c:4527) lives in
//! `sprite_main_guard.rs` already; this file does NOT redefine it.

use super::sprite::{PrepOamCoordsRet as SpritePrepOamCoordsRet, SpriteSpawnInfo};
use super::*;
use crate::rom_random::RomRandomResult;
use crate::types::{sign8, PointU8, ProjectSpeedRet, SpriteHitBox};

// ---------------------------------------------------------------------------
// File-local PrepOamCoordsRet copy.
// ---------------------------------------------------------------------------
// Mirror of the C-side `PrepOamCoordsRet` (sprite.c). The canonical
// `sprite::PrepOamCoordsRet` is module-private; keep a local copy so the
// `SpriteDraw_*(int k, PrepOamCoordsRet *info)` signatures retain their
// named-struct shape and stay convertible from the 3-tuple returned by
// `sprite_prep_oam_coord_or_double_ret`.
#[derive(Copy, Clone, Default)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

impl PrepOamCoordsRet {
    pub(super) fn from_tuple(t: (u16, u16, u8)) -> Self {
        Self {
            x: t.0,
            y: t.1,
            r4: 0,
            flags: t.2,
        }
    }
}

mod sprite_main_draw_shared;
use sprite_main_draw_shared::*;
pub(super) use sprite_main_draw_shared::{arrgi_sin, trinexx_head_sin};

/// ROM $05:a8f0 calls the RNG, then executes `AND #$7f; ADC #$40`.
/// `AND` preserves the carry produced by the RNG routine, so the delay is
/// occasionally one greater than a carry-blind source translation suggests.
const fn rat_random_run_delay(random: RomRandomResult) -> u8 {
    random.masked_adc(0x7f, 0x40)
}

const fn rope_random_run_delay(random: RomRandomResult) -> u8 {
    random.masked_adc(0x7f, 0x40)
}

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
    pub(super) fn sprite_draw_light_fountain(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        let idx = (self.sprite_slot_view(k).graphics() as usize) * 4
            + (self.sprite_slot_view(k).direction() as usize);
        self.sprite_draw_multiple(k, &MASTER_SWORD_LIGHT_BALL_DRAW_FRAMES[idx..idx + 1], None);
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
        self.set_oam_helper0_at_for_draw(
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
        self.sprite_correct_oam_entries_for_draw(k, 0, 0);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_WaterRipple_WithOamAdjust(int k) {  // 859fe5
    //   SpriteDraw_WaterRipple(k);
    //   oam_cur_ptr += 8;
    //   oam_ext_cur_ptr += 2;
    // }
    pub(super) fn sprite_draw_water_ripple_with_oam_adjust(&mut self, k: usize) {
        self.sprite_draw_water_ripple(k);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_AdjustShadow(int k) {  // 859edb
    pub(super) fn walking_zora_adjust_shadow(&mut self, k: usize) {
        let value = u8::from(
            self.sprite_slot_view(k).z() == 0 && self.game_state.sprites.workspace.tile_type() == 9,
        );
        self.sprite_slot_view_mut(k).set_anim_clock(value);
    }

    // -----------------------------------------------------------------------
    // void Zora_Draw(int k) {  // 8598f5
    pub(super) fn zora_draw(&mut self, k: usize) {
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        for i in (0..2usize).rev() {
            let j = d + i;
            let f = ZORA_DRAW_FLAGS[j];
            self.set_oam_helper0_at_for_draw(
                oam,
                x.wrapping_add(ZORA_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(ZORA_DRAW_Y_OFFSETS[j] as i16 as u16),
                ZORA_DRAW_CHARS[j],
                f | if f & 0x0f != 0 { 0 } else { info_flags },
                ZORA_DRAW_BIG[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_55_Zora(int k) {  // 85967b
    pub(super) fn sprite_55_zora(&mut self, k: usize) {
        if self.sprite_slot_view(k).e() != 0 {
            self.sprite_fireball(k);
        } else {
            self.sprite_zora_main(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Fireball(int k) {  // 859683
    pub(super) fn sprite_fireball(&mut self, k: usize) {
        let value = self.sprite_slot_view(k).e();

        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).delay_main() != 0 {
            self.oam_allocate_from_region_c(4);
        }
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.fireball_spawn_trail_garnish(k);
        if self.sprite_check_damage_to_link(k) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        self.sprite_move_xy(k);
        if self.game_state.world.location.is_indoors()
            && self.sprite_slot_view(k).delay_aux1() == 0
            && (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && self.sprite_check_tile_collision(k) != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }

        if self.game_state.player.follower_link.is_bunny_mirror()
            || self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer()
                != 0
            || sign8(self.game_state.player.follower_link.state_bits())
            || self.game_state.inventory.items.shield_type() < 2
            || self.game_state.player.follower_link.lower_level_state()
                != self.sprite_slot_view(k).floor()
        {
            return;
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
        self.sprite_setup_hit_box(k, &mut hb);
        let mut j = self.game_state.player.follower_link.facing_index() & 3;
        if self.game_state.player.follower_link.button_b_frames() != 0 {
            j = usize::from(SPRITE_FIREBALL_OFFSETS[j]);
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(SPRITE_FIREBALL_X_OFFSETS[j] as i16 as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(SPRITE_FIREBALL_Y_OFFSETS[j] as i16 as u16);
        hb.r0_xlo = x as u8;
        hb.r8_xhi = (x >> 8) as u8;
        hb.r2 = SPRITE_FIREBALL_WIDTHS[j];
        hb.r1_ylo = y as u8;
        hb.r9_yhi = (y >> 8) as u8;
        hb.r3 = SPRITE_FIREBALL_HEIGHTS[j];
        if self.check_if_hit_boxes_overlap(&hb) {
            self.sprite_place_rupulse_spark_2(k);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 6);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Zora_Main(int k) {  // 859725
    pub(super) fn sprite_zora_main(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        } else {
            self.zora_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let org_x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let org_y = u16::from(self.sprite_slot_view(k).c())
                        | (u16::from(self.sprite_slot_view(k).head_direction()) << 8);
                    let x_idx = usize::from(self.get_random_number() & 7);
                    self.sprite_set_x(k, org_x.wrapping_add(SURFACE_XY[x_idx] as i16 as u16));
                    let y_idx = usize::from(self.get_random_number() & 7);
                    self.sprite_set_y(k, org_y.wrapping_add(SURFACE_XY[y_idx] as i16 as u16));
                    self.sprite_get16_bit_coords(k);
                    self.sprite_check_tile_collision(k);
                    let spawn_anyway = self.game_state.sprites.workspace.tile_type() == 8
                        || (self.game_state.sprites.workspace.tile_type() == 9
                            && self.sprite_slot_view(k).delay_aux2() == 1
                            && self
                                .game_state
                                .enhanced_features
                                .has(FEATURE_GAME_CHANGING_BUG_FIXES_DRAW));
                    if spawn_anyway {
                        let value = 127;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.sprite_slot_view_mut(k).or_flags3(0x40);
                    } else if self
                        .game_state
                        .enhanced_features
                        .has(FEATURE_GAME_CHANGING_BUG_FIXES_DRAW)
                    {
                        self.sprite_set_x(k, org_x);
                        self.sprite_set_y(k, org_y);
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                        if self.sprite_slot_view(k).delay_aux2() == 0 {
                            let value = 32;
                            self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        }
                    }
                }
            }
            1 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                } else {
                    let value = ZORA_SURFACING_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 23;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 48 {
                        let _ = self.sprite_spawn_fireball(k);
                    }
                    let value = SPRITE_ZORA_MAIN_ATTACK_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    let value = SPRITE_ZORA_MAIN_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 2)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_SpawnBigSplash(int k) {  // 859b40
    pub(super) fn sprite_spawn_big_splash(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x24);
        for i in (0..8usize).rev() {
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 8, &mut info);
            if j >= 0 {
                let j = j as usize;
                let value = 3;
                self.sprite_slot_view_mut(j).set_state(value);
                self.sprite_set_x(
                    j,
                    info.r0_x
                        .wrapping_add(SPRITE_SPAWN_BIG_SPLASH_X_OFFSETS[i] as i16 as u16)
                        .wrapping_sub(4),
                );
                self.sprite_set_y(
                    j,
                    info.r2_y
                        .wrapping_add(SPRITE_SPAWN_BIG_SPLASH_Y_OFFSETS[i] as i16 as u16)
                        .wrapping_sub(4),
                );
                let value = SPRITE_SPAWN_BIG_SPLASH_LOCAL_X_VELOCITIES[i] as u8;
                self.sprite_slot_view_mut(j).set_x_velocity(value);
                let value = SPRITE_SPAWN_BIG_SPLASH_LOCAL_Y_VELOCITIES[i] as u8;
                self.sprite_slot_view_mut(j).set_y_velocity(value);
                let value = i as u8;
                self.sprite_slot_view_mut(j).set_a(value);
                // ROM Sprite_SpawnBigSplash $05:9B94: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                let value = self.get_random_number_with_carry().masked_adc(15, 24);
                self.sprite_slot_view_mut(j).set_z_velocity(value);
                let value = 1;
                self.sprite_slot_view_mut(j).set_ai_state(value);
                let value = 0;
                self.sprite_slot_view_mut(j).set_z(value);
                self.sprite_slot_view_mut(j).or_flags3(0x40);
                let value = self.sprite_slot_view(j).flags3();
                self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_52_KingZora(int k) {  // 85995b
    pub(super) fn sprite_52_king_zora(&mut self, k: usize) {
        assert!(!self.sprite_52_king_zora_impl(k, false));
    }

    /// Execute King Zora through the exact call site immediately before the
    /// purchased-flippers sheet decode. This is used only when the timing
    /// authority proves that the source CPU suspended inside that synchronous
    /// call; the returned `true` proves the live sprite took that path.
    pub(super) fn sprite_52_king_zora_before_flippers_graphics(&mut self, k: usize) -> bool {
        self.sprite_52_king_zora_impl(k, true)
    }

    fn sprite_52_king_zora_impl(&mut self, k: usize, stop_before_flippers_graphics: bool) -> bool {
        self.zora_king_draw(k);
        if self.sprite_return_if_inactive(k) {
            return false;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let dx = self.game_state.player.follower_link.x();
                let dy = self.game_state.player.follower_link.y();
                if dx.wrapping_sub(self.sprite_get_x(k)).wrapping_add(16) < 32
                    && dy.wrapping_sub(self.sprite_get_y(k)).wrapping_add(48) < 96
                {
                    self.link_cancel_dash();
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_sound_effect_1(0x35);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    for j in (0..16usize).rev() {
                        if j != k && (self.sprite_slot_view(j).deflection_bits() & 0x80) == 0 {
                            if self.sprite_slot_view(j).state() == 10 {
                                self.follower_link_state_mut().clear_state_bits();
                                self.follower_link_state_mut().clear_picking_throw_state();
                            }
                            self.sprite_kill_self(j);
                        }
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_bg1_x_offset(0);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    let offs = if self.sprite_slot_view(k).delay_main() & 1 != 0 {
                        (-1i16) as u16
                    } else {
                        1
                    };
                    self.set_bg1_x_offset(offs);
                    self.follower_link_state_mut().immobilize();
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if self.sprite_slot_view(k).delay_main() == 28 {
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        self.sprite_spawn_big_splash(k);
                    }
                    let value = SPRITE_52_KING_ZORA_SURFACING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 36;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return false;
                }
                let value = SPRITE_52_KING_ZORA_DIALOGUE_GFX[usize::from(j >> 4)];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if j == 80 {
                    self.dialogue_message_index_mut().set_value(0x142);
                    self.sprite_show_message_minimal_c();
                } else if j == 79 {
                    if self.multiselect_choice().value() == 0 {
                        self.dialogue_message_index_mut().set_value(0x143);
                        self.sprite_show_message_minimal_c();
                    } else {
                        self.dialogue_message_index_mut().set_value(0x146);
                        self.sprite_show_message_minimal_c();
                        let value = 0x30;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                } else if j == 78 {
                    if self.multiselect_choice().value() == 0
                        && self.game_state.inventory.player_resources.rupees_goal() >= 500
                    {
                        let rupees = self
                            .game_state
                            .inventory
                            .player_resources
                            .rupees_goal()
                            .wrapping_sub(500);
                        self.player_resources_mut().set_rupees_goal(rupees);
                        self.dialogue_message_index_mut().set_value(0x144);
                        self.sprite_show_message_minimal_c();
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_e(value);
                    } else {
                        self.dialogue_message_index_mut().set_value(0x145);
                        self.sprite_show_message_minimal_c();
                        let value = 0x30;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                } else if j == 77 && self.sprite_slot_view(k).e() != 0 {
                    if stop_before_flippers_graphics {
                        if self.sprite_zora_regurgitate_flippers_before_graphics(k) {
                            return true;
                        }
                    } else {
                        self.sprite_zora_regurgitate_flippers(k);
                    }
                }
            }
            4 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    self.sprite_kill_self(k);
                    self.follower_link_state_mut().clear_immobilized();
                } else {
                    if j == 29 {
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                        self.sprite_spawn_big_splash(k);
                    }
                    let value = SPRITE_52_KING_ZORA_SUBMERGE_GFX[usize::from(j >> 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
        false
    }

    // -----------------------------------------------------------------------
    // void Sprite_Zora_RegurgitateFlippers(int k) {  // 9de1aa
    pub(super) fn sprite_zora_regurgitate_flippers(&mut self, k: usize) {
        if self.sprite_zora_regurgitate_flippers_before_graphics(k) {
            self.DecodeAnimatedSpriteTile_variable(0x11);
        }
    }

    fn sprite_zora_regurgitate_flippers_before_graphics(&mut self, k: usize) -> bool {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc0, &mut info);
        if j < 0 {
            return false;
        }
        let j = j as usize;
        self.sprite_set_spawned_coordinates(j, &info);
        let value = 32;
        self.sprite_slot_view_mut(j).set_z_velocity(value);
        let value = 16;
        self.sprite_slot_view_mut(j).set_y_velocity(value);
        let value = 30;
        self.sprite_slot_view_mut(j).set_a(value);
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let value = 0x83;
        self.sprite_slot_view_mut(j).set_flags2(value);
        let value = 0x54;
        self.sprite_slot_view_mut(j).set_flags3(value);
        let value = 0x54 & 15;
        self.sprite_slot_view_mut(j).set_oam_flags(value);
        let value = 0x30;
        self.sprite_slot_view_mut(j).set_delay_aux3(value);
        true
    }

    // -----------------------------------------------------------------------
    // void ZoraKing_Draw(int k) {  // 859cab
    pub(super) fn zora_king_draw(&mut self, k: usize) {
        let Some((mut x, mut y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };

        if self.sprite_slot_view(k).ai_state() >= 2 {
            let mut oam = self.game_state.oam.current_pointer_usize();
            let g = usize::from(self.sprite_slot_view(k).graphics());
            for i in (0..4usize).rev() {
                let j = g * 4 + i;
                let f = ZORA_KING_DRAW_FLAGS_0[j];
                self.oam_state_mut().write_entry(
                    oam,
                    x.wrapping_add(ZORA_KING_DRAW_X_OFFSETS_0[j] as i16 as u16) as u8,
                    y.wrapping_add(ZORA_KING_DRAW_Y_OFFSETS_0[j] as i16 as u16) as u8,
                    ZORA_KING_DRAW_CHARS_0[j],
                    (if f & 0x0f != 0 { f } else { f | info_flags }) | 0x20,
                );
                oam += 4;
            }
            self.sprite_correct_oam_entries_for_draw(k, 3, 2);
            let Some(poc) = self.sprite_prep_oam_coord_or_double_ret(k) else {
                return;
            };
            x = poc.0;
            y = poc.1;
        }

        if self.sprite_slot_view(k).delay_aux2() == 0 {
            return;
        }

        self.oam_allocate_from_region_c(0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from((self.sprite_slot_view(k).delay_aux2() >> 1) & 4);
        for i in (0..4usize).rev() {
            let j = g + i;
            self.set_oam_plain_at_for_draw(
                oam,
                x.wrapping_add(ZORA_KING_DRAW_X_OFFSETS_1[j] as i16 as u16) as u8,
                y.wrapping_add(ZORA_KING_DRAW_Y_OFFSETS_1[j] as i16 as u16) as u8,
                ZORA_KING_DRAW_CHARS_1[j],
                ZORA_KING_DRAW_FLAGS_1[j] | 0x24,
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_C0_Catfish(int k) {  // 9ddf49
    pub(super) fn sprite_c0_catfish(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() & 0x80 != 0 {
            self.sprite_catfish_splash_of_water(k);
        } else if self.sprite_slot_view(k).a() == 0 {
            self.catfish_big_fish(k);
        } else {
            self.sprite_catfish_quake_medallion(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Catfish_QuakeMedallion(int k) {  // 9ddf54
    pub(super) fn sprite_catfish_quake_medallion(&mut self, k: usize) {
        if self.sprite_slot_view(k).z() == 0 {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
            if self.game_state.frame.submodule == 0
                && self.sprite_check_damage_to_link_same_layer(k)
            {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().set_item_receipt_method(0);
                let item = self.sprite_slot_view(k).a();
                if self
                    .link_receive_item_from(
                        item,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::CatfishMedallion,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_catfish_medallion_item_receipt(k);
                return;
            }
        }
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xyz(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = ((self.sprite_slot_view(k).x_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = ((self.sprite_slot_view(k).y_velocity() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let j = self.sprite_slot_view(k).ai_state();
            if j == 4 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            } else {
                self.sprite_slot_view_mut(k).add_ai_state(1);

                let value = SPRITE_CATFISH_QUAKE_MEDALLION_Z_VELOCITIES[usize::from(j)];
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if j < 2 {
                    let splash = self.sprite_spawn_water_splash(k);
                    if splash >= 0 {
                        let value = 16;
                        self.sprite_slot_view_mut(splash as usize)
                            .set_delay_main(value);
                    }
                }
            }
        }
    }

    /// Source suffix after Sprite_Catfish_QuakeMedallion's
    /// `Link_ReceiveItem(sprite_A, 0)` (ROM `$9ddf54`): the sprite was killed
    /// before the call, so only the OAM-region allocation and the single large
    /// draw run before `Sprite_ReturnIfInactive` returns.
    pub(super) fn complete_catfish_medallion_item_receipt(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        self.sprite_draw_single_large(k);
        debug_assert!(self.sprite_slot_view(k).state() == 0);
    }

    // -----------------------------------------------------------------------
    // void Catfish_BigFish(int k) {  // 9ddfd1
    pub(super) fn catfish_big_fish(&mut self, k: usize) {
        self.great_catfish_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                for j in (0..16usize).rev() {
                    if j == k || self.sprite_slot_view(j).state() != 3 {
                        continue;
                    }
                    if self
                        .sprite_get_x(k)
                        .wrapping_sub(self.sprite_get_x(j))
                        .wrapping_add(32)
                        < 64
                        && self
                            .sprite_get_y(k)
                            .wrapping_sub(self.sprite_get_y(j))
                            .wrapping_add(32)
                            < 64
                    {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        return;
                    }
                }
            }
            1 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.set_bg1_x_offset(0);
                    self.set_ambient_sound_effect(5);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    self.catfish_spawn_plop(k);
                } else if j < 0xc0 {
                    if j == 0xbf {
                        self.set_ambient_sound_effect(7);
                    }
                    let offs = if j & 1 != 0 { (-1i16) as u16 } else { 1 };
                    self.set_bg1_x_offset(offs);
                    self.follower_link_state_mut().immobilize();
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                self.sprite_move_xyz(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if self.sprite_slot_view(k).z_velocity() == (-48i8) as u8 {
                    self.catfish_spawn_plop(k);
                }
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let value = CATFISH_BIG_FISH_EMERGE_GFX
                    [usize::from(self.sprite_slot_view(k).subtype2() >> 2)];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            3 => {
                let j = self.sprite_slot_view(k).delay_main();
                if j == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                } else {
                    if matches!(j, 160 | 252 | 4) {
                        self.sprite_spawn_water_splash(k);
                    } else if j == 10 {
                        self.catfish_spawn_plop(k);
                    } else if j == 96 {
                        self.follower_link_state_mut().clear_immobilized();
                        let msg = if self.game_state.inventory.items.quake() != 0 {
                            0x12b
                        } else {
                            0x12a
                        };
                        self.dialogue_message_index_mut().set_value(msg);
                        self.sprite_show_message_minimal_c();
                        return;
                    } else if j == 80 {
                        if self.game_state.inventory.items.quake() != 0 {
                            if self.get_random_number() & 1 != 0 {
                                self.sprite_spawn_bomb(k);
                            } else {
                                self.sprite_spawn_fireball(k);
                            }
                        } else {
                            self.catfish_regurgitate_medallion(k);
                        }
                    }
                    if j < 160 {
                        let value = CATFISH_BIG_FISH_CONVERSATE_GFX[usize::from(j >> 3)];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Catfish_SplashOfWater(int k) {  // 9de37d
    pub(super) fn sprite_catfish_splash_of_water(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let base = usize::from(self.sprite_slot_view(k).delay_main() >> 3) * 2;
        self.sprite_draw_multiple(
            k,
            &SPRITE_CATFISH_SPLASH_OF_WATER_DRAW_FRAMES[base..base + 2],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_56_WalkingZora(int k) {  // 859d4a
    pub(super) fn sprite_56_walking_zora(&mut self, k: usize) {
        if self.sprite_slot_view(k).f() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_f(value);
            let value = 3;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = 192;
            self.sprite_slot_view_mut(k).set_g(value);
            let value = ((self.sprite_slot_view(k).x_recoil() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = ((self.sprite_slot_view(k).y_recoil() as i8) >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }

        match self.sprite_slot_view(k).b() {
            0 => {
                let mut info = SpritePrepOamCoordsRet {
                    x: 0,
                    y: 0,
                    r4: 0,
                    flags: 0,
                };
                self.sprite_prep_oam_coord(k, &mut info);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_b(1);
                    self.sprite_slot_view_mut(k).or_flags3(0x40);
                }
            }
            1 => {
                self.zora_draw(k);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                    self.sprite_slot_view_mut(k).add_b(1);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let dir = self.sprite_direction_to_face_link(k, None);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                } else {
                    let value = ZORA_SURFACING_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                let value = RECRUIT_GRAPHICS[usize::from(
                    ((self.sprite_slot_view(k).subtype2() >> 1) & 4)
                        + self.sprite_slot_view(k).direction(),
                )];
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.walking_zora_draw(k);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_z(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
                    if sign8(self.sprite_slot_view(k).z_velocity().wrapping_add(16)) {
                        self.sprite_zero_velocity_xy(k);
                    }
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                        let j = self.sprite_direction_to_face_link(k, None);
                        let value = j;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        if (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0 {
                            let value = j;
                            self.sprite_slot_view_mut(k).set_direction(value);
                            self.sprite_apply_speed_towards_link(k, 8);
                        }
                    }
                }
                self.sprite_move_xy(k);
                self.sprite_check_tile_collision(k);
                if sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
                    self.walking_zora_adjust_shadow(k);
                    if self.game_state.sprites.workspace.tile_type() == 8 {
                        self.sprite_kill_self(k);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_state(value);
                        let value = 15;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_flags2(value);
                    }
                }
                self.sprite_slot_view_mut(k).add_subtype2(1);
            }
            3 => {
                self.sprite_check_damage_from_link(k);
                if (self.game_state.frame.frame_counter & 3) == 0 {
                    let g = self.sprite_slot_view(k).g().wrapping_sub(1);
                    let value = g;
                    self.sprite_slot_view_mut(k).set_g(value);
                    if g == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_b(value);
                        if self.sprite_slot_view(k).state() == 10 {
                            self.follower_link_state_mut().clear_state_bits();
                            self.follower_link_state_mut().clear_picking_throw_state();
                        }
                        let value = 9;
                        self.sprite_slot_view_mut(k).set_state(value);
                    }
                }
                if self.sprite_slot_view(k).g() < 48
                    && (self.game_state.frame.frame_counter & 1) == 0
                {
                    let delta = if self.game_state.frame.frame_counter & 2 != 0 {
                        -1
                    } else {
                        1
                    };
                    self.sprite_set_x(k, self.sprite_get_x(k).wrapping_add(delta as i16 as u16));
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_wall_collision(value);
                self.walking_zora_draw_water_ripples(k);
                self.sprite_slot_view_mut(k).subtract_flags2(2);
                self.sprite_draw_single_large(k);
                self.sprite_slot_view_mut(k).add_flags2(2);
                let value = 0;
                self.sprite_slot_view_mut(k).set_anim_clock(value);
                if self.sprite_return_if_inactive(k) || self.sprite_return_if_recoiling(k) {
                    return;
                }
                self.sprite_move_xy(k);
                self.thrown_sprite_tile_and_sprite_interaction(k);
                self.walking_zora_adjust_shadow(k);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_Draw(int k) {  // 859f08
    pub(super) fn walking_zora_draw(&mut self, k: usize) {
        self.walking_zora_draw_water_ripples(k);
        let Some((x, mut y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        if g == 0 || g == 2 {
            y = y.wrapping_sub(1);
        }
        let i = usize::from(self.sprite_slot_view(k).head_direction());
        self.set_oam_helper0_at_for_draw(
            oam,
            x,
            y.wrapping_sub(6),
            WALKING_ZORA_DRAW_CHARS[i],
            flags | WALKING_ZORA_DRAW_FLAGS[i],
            2,
        );
        self.set_oam_helper0_at_for_draw(
            oam + 4,
            x,
            y.wrapping_add(2),
            WALKING_ZORA_DRAW_CHARS_2[g],
            flags | WALKING_ZORA_DRAW_FLAGS_2[g],
            2,
        );
        if self.sprite_slot_view(k).anim_clock() == 0 {
            let mut info = SpritePrepOamCoordsRet { x, y, r4: 0, flags };
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // -----------------------------------------------------------------------
    // void WalkingZora_DrawWaterRipples(int k) {  // 859fe0
    pub(super) fn walking_zora_draw_water_ripples(&mut self, k: usize) {
        if self.sprite_slot_view(k).anim_clock() != 0 {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_WaterRipple(int k) {  // 859ffa
    //   static const DrawMultipleData kWaterRipple_Dmd[6] = {
    //     {0, 10, 0x01d8, 0}, {8, 10, 0x41d8, 0},
    //     {0, 10, 0x01d9, 0}, {8, 10, 0x41d9, 0},
    //     {0, 10, 0x01da, 0}, {8, 10, 0x41da, 0},
    //   };
    //   static const uint8 kWaterRipple_Idx[4] = {0, 1, 2, 1};
    //   Sprite_DrawMultiple(k, &kWaterRipple_Dmd[kWaterRipple_Idx[frame_counter >> 2 & 3] * 2], 2, NULL);
    //   OamEnt *oam = GetOamCurPtr();
    //   uint8 t = (oam[0].flags & 0x30) | 0x4;
    //   oam[0].flags = t;
    //   oam[1].flags = t | 0x40;
    // }
    pub(super) fn sprite_draw_water_ripple(&mut self, k: usize) {
        let frame = self.game_state.frame.frame_counter;
        let base = (WATER_RIPPLE_FRAME_INDICES[((frame >> 2) & 3) as usize] as usize) * 2;
        self.sprite_draw_multiple(k, &WATER_RIPPLE_DRAW_FRAMES[base..base + 2], None);
        let oam = self.game_state.oam.current_pointer_usize();
        let t = (self.game_state.oam.entry_flags(oam) & 0x30) | 0x4;
        self.oam_state_mut().set_entry_flags(oam, t);
        self.oam_state_mut().set_entry_flags(oam + 4, t | 0x40);
    }

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
        self.set_oam_helper0_at_for_draw(
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
            self.set_oam_helper0_at_for_draw(
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
        self.set_oam_plain_at_for_draw(oam, x0, y0, 0x2a, 0x2d, 2);

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
            self.set_oam_plain_at_for_draw(oam_cur, x, y, 0x3f, 0x2d, 0);
            oam_cur += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries_for_draw(k, 4, 0xff);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_BigCannonball(int k) {  // 85b6a4 — sprite_main.c:4019
    //   Sprite_PrepOamCoordOrDoubleRet, then four large-tile OAM entries.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_big_cannonball(&mut self, k: usize) {
        let info = match self.sprite_prep_oam_coord_or_double_ret(k) {
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
            self.set_oam_helper0_at_for_draw(
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
    pub(super) fn sprite_draw_sprite_bomb_explosion(&mut self, k: usize) {
        let info = match self.sprite_prep_oam_coord_or_double_ret(k) {
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
        self.sprite_correct_oam_entries_for_draw(k, 3, 2);
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
        let r6 = (self.sprite_slot_view(k).direction() as u8)
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
            self.set_oam_helper0_at_for_draw(oam, x, y, charnum, flags, 0);
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
            self.set_oam_helper0_at_for_draw(
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
            self.set_oam_helper0_at_for_draw(
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
            self.set_oam_helper0_at_for_draw(oam, x, y, 0x6c, 0x24, 2);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void ThrowableScenery_ScatterIntoDebris(int k) {  // 86ac41
    pub(super) fn throwable_scenery_scatter_into_debris(&mut self, k: usize) {
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
    pub(super) fn throwable_scenery_scatter_after_state_clear(&mut self, k: usize) {
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
            self.set_oam_helper0_at_for_draw(oam, x, y, 0xbc, OCTOSTONE_DRAW_FLAGS[j] | 0x2d, 0);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_ZirroBomb(int k) {  // 8dd606 — sprite_main.c:13246
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_zirro_bomb(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let base = ((self.sprite_slot_view(k).delay_main() >> 2) as usize) * 3;
        self.sprite_draw_multiple(k, &BOMBER_PELLET_DRAW_FRAMES[base..base + 3], None);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Pikit_Tongue(int k, PrepOamCoordsRet *info) {  // 8dd74a — sprite_main.c:13316
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_pikit_tongue(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).ai_state() != 2 || self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let x = info.x.wrapping_add(4) as u8;
        let y = info.y.wrapping_add(3) as u8;
        // oam[5]: 20 byte offset from oam base.
        let head_x = x.wrapping_add(self.sprite_slot_view(k).a());
        let head_y = y.wrapping_add(self.sprite_slot_view(k).b());
        let mut oam_view = self.oam_state_mut();
        oam_view.write_entry(oam + 5 * 4, x, y, 0xfe, info.flags);
        // oam[0]:
        oam_view.write_entry(oam, head_x, head_y, 0xfe, info.flags);
        // oam++; loop over i=3..=0 starting at oam[1].
        let mut cur = oam + 4;
        let g = self.sprite_slot_view(k).direction() as usize;
        let mut i: i32 = 3;
        loop {
            let sa = self.sprite_slot_view(k).a() as i8 as i32;
            let sb = self.sprite_slot_view(k).b() as i8 as i32;
            let xv = x as i32 + (sa * PIKIT_TONGUE_LENGTH_MULTIPLIERS[i as usize] as i32) / 256;
            let yv = y as i32 + (sb * PIKIT_TONGUE_LENGTH_MULTIPLIERS[i as usize] as i32) / 256;
            self.oam_state_mut().write_entry(
                cur,
                xv as u8,
                yv as u8,
                PIKIT_TONGUE_DRAW_CHARS[g],
                PIKIT_TONGUE_DRAW_FLAGS[g] | info.flags,
            );
            cur += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries_for_draw(k, 5, 0);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_Pikit_Loot(int k, PrepOamCoordsRet *info) {  // 8dd858 — sprite_main.c:13341
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_pikit_loot(&mut self, k: usize, _info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).g() == 0 {
            return;
        }
        let mut g = (self.sprite_slot_view(k).g() as usize) - 1;
        if g == 3 {
            g = (self.sprite_slot_view(k).subtype() as usize) + 2;
        }
        self.oam_allocate_from_region_c(0x10);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let tmp_x = self.game_state.scratch_counter.value();
        let tmp_y = self.game_state.sprites.workspace.shared_scratch_a();
        let mut i: i32 = 3;
        loop {
            let j = g * 4 + i as usize;
            self.oam_state_mut().write_entry(
                oam,
                tmp_x.wrapping_add(PIKIT_GRABBED_ITEM_X_OFFSETS[j] as u8),
                tmp_y.wrapping_add(PIKIT_GRABBED_ITEM_Y_OFFSETS[j] as u8),
                PIKIT_GRABBED_ITEM_CHARS[j],
                PIKIT_GRABBED_ITEM_FLAGS[g],
            );
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
        self.sprite_correct_oam_entries_for_draw(k, 3, 0);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_TrinexxRockHead(int k, PrepOamCoordsRet *info) {  // 9db560
    //   sprite_main.c:16348 — 36-entry head OAM block.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_trinexx_rock_head(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        if !sign8(self.sprite_slot_view(k).ai_state()) {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
        }
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        let base = base.min(TRINEXX_DRAW1_DRAW_FRAMES.len() - 4);
        // Sprite_DrawMultiple(... info) writes the out-pointer before returning
        // early for offscreen sprites. Trinexx body drawing still consumes those
        // flags, so preserve the populated tuple even when no head OAM is drawn.
        let (prepped, out_of_bounds) = self.sprite_prep_oam_coord_or_double_ret_with_out_flag(k);
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
    pub(super) fn sprite_draw_trinexx_rock_head_and_body(&mut self, k: usize) {
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
                self.set_oam_plain_at_for_draw(oam, xv as u8, yv as u8, 0x28, info.flags, 2);
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
            self.set_oam_plain_at_for_draw(oam, x, y, 0xc, f, 2);
            self.set_oam_plain_at_for_draw(oam + 4, x, y.wrapping_add(16), 0x2a, f, 2);
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
        for i in 0..5 {
            let j = g * 5 + i;
            let jc = j.min(TRINEXX_DRAW_X_OFFSETS.len() - 1);
            let x = xb.wrapping_add(TRINEXX_DRAW_X_OFFSETS[jc] as u8);
            let y = yb
                .wrapping_sub(TRINEXX_DRAW_Y_OFFSETS[jc] as u16)
                .wrapping_sub(0x20)
                .wrapping_add(ov7);
            self.set_oam_helper0_at_for_draw(
                oam,
                x as u16,
                y,
                TRINEXX_DRAW_CHARS[i],
                info.flags,
                2,
            );
            oam += 4;
        }
        self.temp_counter_mut().set(0xff);

        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries_for_draw(k, 3, 2);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_BigShadow(int k, int anim) {  // 9dd1a8 — sprite_main.c:17651
    //   cur_sprite_y += sprite_z[k];
    //   oam_cur_ptr += 16;
    //   oam_ext_cur_ptr += 4;
    //   Sprite_DrawMultiple(k, &kLargeShadow_Dmd[anim * 3], 3, NULL);
    //   Sprite_Get16BitCoords(k);
    // }
    pub(super) fn sprite_draw_big_shadow(&mut self, k: usize, anim: i32) {
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let z = self.sprite_slot_view(k).z() as u16;
        self.sprite_workspace_mut()
            .set_current_sprite_y(cur_y.wrapping_add(z));
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(16));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(4));
        let base = (anim as usize) * 3;
        self.sprite_draw_multiple(k, &LARGE_SHADOW_DRAW_FRAMES[base..base + 3], None);
        self.sprite_get16_bit_coords(k);
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
            self.set_oam_plain_at_for_draw(cur_oam, x, y, ch, fl | 0x31, big);
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
        self.sprite_draw_multiple(k, &ALTAR_ZELDA_WARP_DRAW_FRAMES[base..base + 2], None);
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
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let base = base.min(ALTAR_ZELDA_DRAW_FRAMES.len() - 2);
        self.sprite_draw_multiple(k, &ALTAR_ZELDA_DRAW_FRAMES[base..base + 2], Some(&mut info));
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

    pub(super) fn sprite_agahnim_apply_motion_blur(&mut self, k: usize) -> i32 {
        if self.game_state.frame.frame_counter & 3 != 0 {
            return -1;
        }
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xc1, &mut info);
        if j >= 0 {
            let ju = j as usize;
            self.sprite_set_spawned_coordinates(ju, &info);
            let value = self.sprite_slot_view(k).graphics();
            self.sprite_slot_view_mut(ju).set_graphics(value);
            let value = 32;
            self.sprite_slot_view_mut(ju).set_delay_main(value);
            let value = 32;
            self.sprite_slot_view_mut(ju).set_ignore_projectile(value);
            let value = 32;
            self.sprite_slot_view_mut(ju).set_c(value);
        }
        j
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
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics()).min(17);
        for i in (0..=3).rev() {
            let j = g * 4 + i;
            self.set_oam_plain_at_for_draw(
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
            let mut shadow_info = SpritePrepOamCoordsRet {
                x: info_x,
                y: info_y,
                r4: 0,
                flags: info_flags,
            };
            self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
        }
        if self.game_state.frame.submodule != 0 {
            self.sprite_correct_oam_entries_for_draw(k, 3, 0xff);
        }

        let Some((info_x, info_y, _info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k)
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
            self.set_oam_plain_at_for_draw(
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
                if delay >= 239 || delay < 16 {
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
                    self.PaletteFilter_SP5F();
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

    // void SpriteActive_Main(int k) {  // 869271
    pub(super) fn sprite_active_main(&mut self, k: usize) {
        match self.sprite_slot_view(k).sprite_type() {
            0x00 => self.sprite_raven(k),
            0x01 => self.sprite_01_vulture_bounce(k),
            0x02 => self.sprite_02_stalfos_head(k),
            0x03 => {}
            0x04 => self.sprite_pull_switch_bounce(k),
            0x05 => self.sprite_pull_switch_bounce(k),
            0x06 => self.sprite_pull_switch_bounce(k),
            0x07 => self.sprite_pull_switch_bounce(k),
            0x08 => self.sprite_08_octorok(k),
            0x09 => self.sprite_09_giant_moldorm(k),
            0x0a => self.sprite_08_octorok(k),
            0x0b => self.sprite_0_b_cucco(k),
            0x0c => self.sprite_0_c_octorok_stone(k),
            0x0d => self.sprite_0_d_buzzblob(k),
            0x0e => self.sprite_0_e_snapdragon(k),
            0x0f => self.sprite_0_f_octoballoon(k),
            0x10 => self.sprite_10_octoballoon_baby(k),
            0x11 => self.sprite_11_hinox(k),
            0x12 => self.sprite_12_moblin(k),
            0x13 => self.sprite_13_mini_helmasaur(k),
            0x14 => self.sprite_14_thieves_town_grate(k),
            0x15 => self.sprite_15_antifairy(k),
            0x16 => self.sprite_16_elder_bounce(k),
            0x17 => self.sprite_17_hoarder(k),
            0x18 => self.sprite_18_mini_moldorm(k),
            0x19 => self.sprite_19_poe(k),
            0x1a => self.sprite_1_a_smithy(k),
            0x1b => self.sprite_1_b_arrow(k),
            0x1c => self.sprite_1_c_statue(k),
            0x1d => self.sprite_1_d_flute_quest(k),
            0x1e => self.sprite_1_e_crystal_switch(k),
            0x1f => self.sprite_1_f_sick_kid(k),
            0x20 => self.sprite_20_sluggula(k),
            0x21 => self.sprite_21_water_switch(k),
            0x22 => self.sprite_22_ropa(k),
            0x23 => self.sprite_23_red_bari(k),
            0x24 => self.sprite_23_red_bari(k),
            0x25 => self.sprite_25_talking_tree(k),
            0x26 => self.sprite_26_hardhat_beetle(k),
            0x27 => self.sprite_27_deadrock(k),
            0x28 => self.sprite_28_dark_world_hint_npc(k),
            0x29 => self.sprite_human_multi_1(k),
            0x2a => self.sprite_sweeping_lady(k),
            0x2b => self.sprite_2_b_hobo(k),
            0x2c => self.sprite_lumberjacks(k),
            0x2d => self.sprite_2_d_telepathic_tile(k),
            0x2e => self.sprite_2_e_flute_kid(k),
            0x2f => self.sprite_maze_game_lady(k),
            0x30 => self.sprite_maze_game_guy(k),
            0x31 => self.sprite_fortune_teller(k),
            0x32 => self.sprite_quarrel_bros(k),
            0x33 => self.sprite_33_rupee_pull(k),
            0x34 => self.sprite_young_snitch_lady(k),
            0x35 => self.sprite_inn_keeper(k),
            0x36 => self.sprite_witch(k),
            0x37 => self.sprite_37_waterfall(k),
            0x38 => self.sprite_38_eye_statue(k),
            0x39 => self.sprite_39_locksmith(k),
            0x3a => self.sprite_3_a_magic_bat(k),
            0x3b => self.sprite_dash_item(k),
            0x3c => self.sprite_trough_boy(k),
            0x3d => self.sprite_old_snitch_lady(k),
            0x3e => self.sprite_17_hoarder(k),
            0x3f => self.sprite_tutorial_guard_or_barrier(k),
            0x40 => self.sprite_tutorial_guard_or_barrier(k),
            0x41 => self.sprite_41_blue_guard(k),
            0x42 => self.sprite_41_blue_guard(k),
            0x43 => self.sprite_41_blue_guard(k),
            0x44 => self.sprite_44_bluesain_bolt(k),
            0x45 => self.sprite_45_hog_spear_man(k),
            0x46 => self.sprite_46_blue_archer(k),
            0x47 => self.sprite_47_green_bush_guard(k),
            0x48 => self.sprite_48_red_javelin_guard(k),
            0x49 => self.sprite_49_red_bush_guard(k),
            0x4a => self.sprite_4_a_bomb_guard(k),
            0x4b => self.sprite_4_b_green_knife_guard(k),
            0x4c => self.sprite_4_c_geldman(k),
            0x4d => self.sprite_4_d_toppo(k),
            0x4e => self.sprite_4_e_popo(k),
            0x4f => self.sprite_4_e_popo(k),
            0x50 => self.sprite_50_cannonball(k),
            0x51 => self.sprite_51_armos_statue(k),
            0x52 => self.sprite_52_king_zora(k),
            0x53 => self.sprite_53_armos_knight(k),
            0x54 => self.sprite_54_lanmolas(k),
            0x55 => self.sprite_55_zora(k),
            0x56 => self.sprite_56_walking_zora(k),
            0x57 => self.sprite_57_desert_statue(k),
            0x58 => self.sprite_58_crab(k),
            0x59 => self.sprite_59_lost_woods_bird(k),
            0x5a => self.sprite_5_a_lost_woods_squirrel(k),
            0x5b => self.sprite_5_b_spark_clockwise(k),
            0x5c => self.sprite_5_b_spark_clockwise(k),
            0x5d => self.sprite_5_d_roller_vertical_down_first(k),
            0x5e => self.sprite_5_d_roller_vertical_down_first(k),
            0x5f => self.sprite_5_d_roller_vertical_down_first(k),
            0x60 => self.sprite_5_d_roller_vertical_down_first(k),
            0x61 => self.sprite_61_beamos(k),
            0x62 => self.sprite_62_master_sword(k),
            0x63 => self.sprite_63_debirando_pit(k),
            0x64 => self.sprite_64_debirando(k),
            0x65 => self.sprite_65_archery_game(k),
            0x66 => self.sprite_66_wall_cannon_vertical_left(k),
            0x67 => self.sprite_66_wall_cannon_vertical_left(k),
            0x68 => self.sprite_66_wall_cannon_vertical_left(k),
            0x69 => self.sprite_66_wall_cannon_vertical_left(k),
            0x6a => self.sprite_6_a_ball_n_chain(k),
            0x6b => self.sprite_6_b_cannon_trooper(k),
            0x6c => self.sprite_6_c_mirror_portal(k),
            0x6d => self.sprite_6_d_rat(k),
            0x6e => self.sprite_6_e_rope(k),
            0x6f => self.sprite_6_f_keese(k),
            0x70 => self.sprite_70_king_helmasaur_fireball(k),
            0x71 => self.sprite_71_leever(k),
            0x72 => self.sprite_72_fairy_pond(k),
            0x73 => self.sprite_73_uncle_and_priest(k),
            0x74 => self.sprite_running_man(k),
            0x75 => self.sprite_bottle_vendor(k),
            0x76 => self.sprite_76_zelda(k),
            0x77 => self.sprite_15_antifairy(k),
            0x78 => self.sprite_78_mrs_sahasrahla(k),
            0x79 => self.sprite_79_bee(k),
            0x7a => self.sprite_7_a_agahnim(k),
            0x7b => self.sprite_7_b_agahnim_balls(k),
            0x7c => self.sprite_7_c_green_stalfos(k),
            0x7d => self.sprite_7_d_big_spike(k),
            0x7e => self.sprite_7_e_firebar_clockwise(k),
            0x7f => self.sprite_7_e_firebar_clockwise(k),
            0x80 => self.sprite_80_firesnake(k),
            0x81 => self.sprite_81_hover(k),
            0x82 => self.sprite_82_antifairy_circle(k),
            0x83 => self.sprite_83_green_eyegore(k),
            0x84 => self.sprite_83_green_eyegore(k),
            0x85 => self.sprite_85_yellow_stalfos(k),
            0x86 => self.sprite_86_kodongo(k),
            0x87 => self.sprite_87_kodongo_fire(k),
            0x88 => self.sprite_88_mothula(k),
            0x89 => self.sprite_89_mothula_beam(k),
            0x8a => self.sprite_8_a_spike_block(k),
            0x8b => self.sprite_8_b_gibdo(k),
            0x8c => self.sprite_8_c_arrghus(k),
            0x8d => self.sprite_8_d_arrghi(k),
            0x8e => self.sprite_8_e_terrorpin(k),
            0x8f => self.sprite_8_f_blob(k),
            0x90 => self.sprite_90_wallmaster(k),
            0x91 => self.sprite_91_stalfos_knight(k),
            0x92 => self.sprite_92_helmasaur_king(k),
            0x93 => self.sprite_93_bumper(k),
            0x94 => self.sprite_94_pirogusu(k),
            0x95 => self.sprite_95_laser_eye_left(k),
            0x96 => self.sprite_95_laser_eye_left(k),
            0x97 => self.sprite_95_laser_eye_left(k),
            0x98 => self.sprite_95_laser_eye_left(k),
            0x99 => self.sprite_99_pengator(k),
            0x9a => self.sprite_9_a_kyameron(k),
            0x9b => self.sprite_9_b_wizzrobe(k),
            0x9c => self.sprite_9_c_zoro(k),
            0x9d => self.sprite_9_c_zoro(k),
            0x9e => self.sprite_9_e_haunted_grove_ostritch(k),
            0x9f => self.sprite_9_f_haunted_grove_rabbit(k),
            0xa0 => self.sprite_a0_haunted_grove_bird(k),
            0xa1 => self.sprite_a1_freezor(k),
            0xa2 => self.sprite_a2_kholdstare(k),
            0xa3 => self.sprite_a3_kholdstare_shell(k),
            0xa4 => self.sprite_a4_falling_ice(k),
            0xa5 => self.sprite_zazak_main(k),
            0xa6 => self.sprite_zazak_main(k),
            0xa7 => self.sprite_a7_stalfos(k),
            0xa8 => self.sprite_a8_green_zirro(k),
            0xa9 => self.sprite_a8_green_zirro(k),
            0xaa => self.sprite_aa_pikit(k),
            0xab => self.sprite_ab_crystal_maiden(k),
            0xac => self.sprite_ac_apple(k),
            0xad => self.sprite_ad_old_man(k),
            0xae => self.sprite_ae_pipe_down(k),
            0xaf => self.sprite_ae_pipe_down(k),
            0xb0 => self.sprite_ae_pipe_down(k),
            0xb1 => self.sprite_ae_pipe_down(k),
            0xb2 => self.sprite_b2_player_bee(k),
            0xb3 => self.sprite_b3_pedestal_plaque(k),
            0xb4 => self.sprite_b4_purple_chest(k),
            0xb5 => self.sprite_b5_bomb_shop(k),
            0xb6 => self.sprite_b6_kiki(k),
            0xb7 => self.sprite_b7_blind_maiden(k),
            0xb8 => self.sprite_b8_dialogue_tester(k),
            0xb9 => self.sprite_b9_bully_and_pink_ball(k),
            0xba => self.sprite_ba_whirlpool(k),
            0xbb => self.sprite_bb_shopkeeper(k),
            0xbc => self.sprite_bc_drunkard(k),
            0xbd => self.sprite_bd_vitreous(k),
            0xbe => self.sprite_be_vitreous_eye(k),
            0xbf => self.sprite_bf_lightning(k),
            0xc0 => self.sprite_c0_catfish(k),
            0xc1 => self.sprite_c1_cutscene_agahnim(k),
            0xc2 => self.sprite_c2_boulder(k),
            0xc3 => self.sprite_c3_gibo(k),
            0xc4 => self.sprite_c4_thief(k),
            0xc5 => self.sprite_c5_medusa(k),
            0xc6 => self.sprite_c6_4_way_shooter(k),
            0xc7 => self.sprite_c7_pokey(k),
            0xc8 => self.sprite_c8_big_fairy(k),
            0xc9 => self.sprite_c9_tektite(k),
            0xca => self.sprite_ca_chain_chomp(k),
            0xcb => self.sprite_cb_trinexx_rock_head(k),
            0xcc => self.sprite_cc(k),
            0xcd => self.sprite_cd(k),
            0xce => self.sprite_ce_blind(k),
            0xcf => self.sprite_cf_swamola(k),
            0xd0 => self.sprite_d0_lynel(k),
            0xd1 => self.sprite_d1_bunny_beam(k),
            0xd2 => self.sprite_d2_flopping_fish(k),
            0xd3 => self.sprite_d3_stal(k),
            0xd4 => self.sprite_d4_landmine(k),
            0xd5 => self.sprite_d5_dig_game_guy(k),
            0xd6 => self.sprite_d6_ganon(k),
            0xd7 => self.sprite_d6_ganon(k),
            0xd8 => self.sprite_d8_heart(k),
            0xd9 => self.sprite_d9_green_rupee(k),
            0xda => self.sprite_d9_green_rupee(k),
            0xdb => self.sprite_d9_green_rupee(k),
            0xdc => self.sprite_d9_green_rupee(k),
            0xdd => self.sprite_d9_green_rupee(k),
            0xde => self.sprite_d9_green_rupee(k),
            0xdf => self.sprite_d9_green_rupee(k),
            0xe0 => self.sprite_d9_green_rupee(k),
            0xe1 => self.sprite_d9_green_rupee(k),
            0xe2 => self.sprite_d9_green_rupee(k),
            0xe3 => self.sprite_e3_fairy(k),
            0xe4 => self.sprite_e4_small_key(k),
            0xe5 => self.sprite_e4_small_key(k),
            0xe6 => self.sprite_d9_green_rupee(k),
            0xe7 => self.sprite_e7_mushroom(k),
            0xe8 => self.sprite_e8_fake_sword(k),
            0xe9 => self.sprite_e9_potion_shop(k),
            0xea => self.sprite_heart_container(k),
            0xeb => self.sprite_heart_piece(k),
            0xec => self.sprite_ec_thrown_item(k),
            0xed => self.sprite_ed_somaria_platform(k),
            0xee => self.sprite_ee_movable_mantle(k),
            0xef => self.sprite_ed_somaria_platform(k),
            0xf0 => self.sprite_ed_somaria_platform(k),
            0xf1 => self.sprite_ed_somaria_platform(k),
            0xf2 => self.sprite_f2_medallion_tablet(k),
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
        let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
        self.sprite_draw_multiple(k, &GIANT_MOLDORM_SEG_A_DRAW_FRAMES[base..base + 4], None);
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
        self.sprite_draw_multiple(k, &GIANT_MOLDORM_HEAD_DRAW_FRAMES[base..base + 4], None);
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
            let y = info.y.wrapping_add(GIANT_MOLDORM_EYE_Y_OFFSETS[idx] as u16);
            let cf_idx = ((r6 + r7 as i32) & 0xf) as usize;
            self.set_oam_helper0_at_for_draw(
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
        self.sprite_draw_multiple(k, &DRAW_FOUR_AROUND_ONE_DRAW_FRAMES[base..base + 5], None);
    }

    pub(super) fn complete_antfairy_after_subtype2_increment(
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

    pub(super) fn antfairy_draw_continuation(&self, k: usize) -> AntfairyDrawContinuation {
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
    // void SpriteDraw_KingHelmasaur_Eyes(int k, PrepOamCoordsRet *info) {  // 9e856b
    //   sprite_main.c:19586.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_king_helmasaur_eyes(&mut self, k: usize, info: &PrepOamCoordsRet) {
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
            self.set_oam_plain_at_for_draw(
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
            self.sprite_correct_oam_entries_for_draw(k, 1, 0);
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Body(int k, PrepOamCoordsRet *info) {  // 9e87e5
    //   sprite_main.c:19661.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_king_helmasaur_body(
        &mut self,
        k: usize,
        info: &mut PrepOamCoordsRet,
    ) {
        let mut prepped = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(k, &HELMASAUR_KING_DRAW_D_DRAW_FRAMES, Some(&mut prepped));
        info.x = prepped.x;
        info.y = prepped.y;
        info.r4 = prepped.r4;
        info.flags = prepped.flags;
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Legs(int k, PrepOamCoordsRet *info) {  // 9e8805
    //   sprite_main.c:19686.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_king_helmasaur_legs(&mut self, k: usize, info: &PrepOamCoordsRet) {
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
                .wrapping_add(
                    self.game_state
                        .sprites
                        .overlord_slots
                        .slot(i as usize)
                        .x_low() as u16,
                ) as u8;
            let f = HELMASAUR_KING_EXPLOSION_FLAGS[i as usize] ^ info.flags;
            self.set_oam_plain_at_for_draw(
                oam,
                x,
                y,
                HELMASAUR_KING_EXPLOSION_CHARS[i as usize],
                f,
                2,
            );
            self.set_oam_plain_at_for_draw(
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
            self.sprite_correct_oam_entries_for_draw(k, 7, 2);
            // Sprite_PrepOamCoordOrDoubleRet(k, info) — refresh the out-ref.
            if let Some(p) = self.sprite_prep_oam_coord_or_double_ret(k) {
                let info_ptr = info as *const PrepOamCoordsRet as *mut PrepOamCoordsRet;
                unsafe {
                    (*info_ptr).x = p.0;
                    (*info_ptr).y = p.1;
                    (*info_ptr).flags = p.2;
                    (*info_ptr).r4 = 0;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_KingHelmasaur_Mouth(int k, PrepOamCoordsRet *info) {  // 9e88bc
    //   sprite_main.c:19709.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_king_helmasaur_mouth(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).delay_aux2() == 0 {
            return;
        }
        let yd_idx = (self.sprite_slot_view(k).delay_aux2() >> 2) as usize;
        let yd = HELMASAUR_KING_FIREBALL_Y_OFFSETS[yd_idx];
        self.oam_allocate_from_region_b(4);
        let oam = self.game_state.oam.current_pointer_usize();
        let x = info.x as u8;
        let y = (info.y as u8).wrapping_add(yd).wrapping_add(0x13);
        self.set_oam_plain_at_for_draw(oam, x, y, 0xaa, info.flags ^ 0xb, 2);
    }

    // void KingHelmasaurMask(int k, PrepOamCoordsRet *info) {  // 9e8686
    pub(super) fn king_helmasaur_mask(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
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
        let mut prepped = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(
            k,
            &HELMASAUR_MASK_DRAW_FRAMES[start..start + 8],
            Some(&mut prepped),
        );
        info.x = prepped.x;
        info.y = prepped.y;
        info.r4 = prepped.r4;
        info.flags = prepped.flags;
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
    pub(super) fn king_helmasaur_operate_tail(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        for i in 0..16usize {
            let j = i + if self.sprite_slot_view(k).anim_clock() != 0 {
                16
            } else {
                0
            };
            let rs = self
                .game_state
                .sprites
                .overlord_slots
                .slot(5)
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
                * u16::from(KING_HELMASAUR_OPERATE_TAIL_MULT_B[i]))
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
        self.sprite_correct_oam_entries_for_draw(k, 16, 2);
        if let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret(k) {
            info.x = prepped.0;
            info.y = prepped.1;
            info.r4 = 0;
            info.flags = prepped.2;
        }
        self.temp_counter_mut().set(16);
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_StalfosKnight_Head(int k, PrepOamCoordsRet *info) {  // 9eae4e
    //   sprite_main.c:21654.
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_stalfos_knight_head(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.sprite_slot_view(k).graphics() == 2 {
            return;
        }
        let i = (self.sprite_slot_view(k).head_direction() & 3) as usize;
        let oam = self.game_state.oam.current_pointer_usize();
        let y = info
            .y
            .wrapping_add(self.sprite_slot_view(k).c() as u16)
            .wrapping_sub(12);
        self.set_oam_helper0_at_for_draw(
            oam,
            info.x,
            y,
            STALFOS_KNIGHT_HEAD_CHARS[i],
            info.flags | STALFOS_KNIGHT_HEAD_FLAGS[i],
            2,
        );
    }

    // -----------------------------------------------------------------------
    // void SpriteDraw_ShopItem(int k) {  // 9ef4ce — sprite_main.c:25333
    // -----------------------------------------------------------------------
    pub(super) fn sprite_draw_shop_item(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).subtype2().wrapping_sub(7) as usize) * 5;
        self.sprite_draw_multiple_player_deferred(
            k,
            &SHOP_KEEPER_ITEM_WITH_PRICE_DRAW_FRAMES[base..base + 5],
            None,
        );
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
        let _ = self.sprite_check_tile_collision(k);
        self.sprite_check_damage_to_link(k);
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
    pub(super) fn sprite_spawn_probe_always(&mut self, k: usize, r15: u8) {
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
    // void Beamos_Draw(int k) {  // 859068
    //   Draw the two 16x16 body tiles, then overlay the rotating eyeball.
    // }
    pub(super) fn beamos_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_helper0_at_for_draw(
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
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
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
            self.set_oam_helper0_at_for_draw(oam, x, y, 0x5c, info.flags, 0);
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Beamos_Laser(int k) {  // 8591b5
    pub(super) fn sprite_beamos_laser(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        self.beamos_laser_draw(k);
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
            self.sprite_move_xy(k);
        }

        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_system_mut().decrement_limit_instance();
            }
            return;
        }
        if !self.sprite_check_damage_to_link_same_layer(k)
            && self.sprite_check_tile_collision(k) == 0
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
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            self.set_oam_helper0_at_for_draw(
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
    // void Crab_Draw(int k) {  // 859510
    //   Two mirrored 16x16 body tiles plus the common shadow.
    // }
    pub(super) fn crab_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = (self.sprite_slot_view(k).graphics() as usize) * 2;
        for i in (0..2).rev() {
            let j = d + i;
            self.set_oam_helper0_at_for_draw(
                oam,
                x.wrapping_add(CRAB_DRAW_X_OFFSETS[j] as u16),
                y,
                CRAB_DRAW_CHARS[j],
                CRAB_DRAW_FLAGS[j] | flags,
                2,
            );
            oam += 4;
        }
        let mut info = SpritePrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Poe_Draw(int k) {  // 869786
    //   Single small flame body tile.
    // }
    pub(super) fn poe_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let d = usize::from(self.sprite_slot_view(k).direction() & 1);
        let ch = POE_DRAW_CHARS[((self.sprite_slot_view(k).subtype2() >> 3) & 3) as usize];
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_helper0_at_for_draw(
            oam,
            x.wrapping_add(POE_DRAW_X_OFFSETS[d] as i16 as u16),
            y.wrapping_add(9),
            ch,
            (flags & 0xf0) | 2,
            0,
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_23_RedBari(int k) {  // 86a23d
    //   Red Bari movement, electrocuted phase, and split behavior.
    // }
    pub(super) fn sprite_23_red_bari(&mut self, k: usize) {
        if sign8(self.sprite_slot_view(k).c()) {
            if self.sprite_slot_view(k).head_direction() != 16 {
                self.sprite_slot_view_mut(k).add_head_direction(1);
            } else {
                let value = 255;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 255;
                self.sprite_slot_view_mut(k).set_subtype(value);
                self.sprite_check_tile_collision2(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_subtype(value);
                if self.game_state.sprites.workspace.tile_type() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                    self.red_bari_set_electrocute_delay(k);
                } else {
                    let value = self.game_state.sprites.workspace.tile_type();
                    self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                }
            }
            return;
        }
        if self.sprite_slot_view(k).c() != 0 {
            self.sprite_draw_single_small(k);
        } else if self.sprite_slot_view(k).graphics() >= 2 {
            self.sprite_draw_single_large(k);
        } else {
            self.red_bari_draw(k);
        }

        self.sprite_23_red_bari_after_draw(k);
    }

    pub(super) fn sprite_23_red_bari_after_draw(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let recoil_from_split = self.sprite_slot_view(k).delay_aux2() != 0;

        if self.sprite_slot_view(k).ai_state() == 2 {
            let value = self.sprite_slot_view(k).ai_state();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            let value = SPRITE_23_RED_BARI_BARI_IDLE_X_VELOCITIES
                [usize::from((self.game_state.frame.frame_counter >> 1) & 1)];
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            self.sprite_move_x(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.red_bari_split(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            return;
        }

        if !recoil_from_split {
            self.sprite_check_damage_to_and_from_link(k);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                let value = self.sprite_slot_view(k).a().wrapping_add(
                    if (self.sprite_slot_view(k).b() & 1) != 0 {
                        0xff
                    } else {
                        1
                    },
                );
                self.sprite_slot_view_mut(k).set_a(value);
                if (self.get_random_number() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
            }
            let j = usize::from(self.sprite_slot_view(k).a() & 15);
            let value = BARI_BURST_X_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = BARI_BURST_Y_VELOCITIES[j];
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }

        if recoil_from_split
            || ((((k as u8) ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(k).delay_main())
                == 0
        {
            if self.sprite_slot_view(k).wall_collision() == 0 {
                self.sprite_move_xy(k);
            }
            self.sprite_check_tile_collision2(k);
        }
        let j = usize::from(self.sprite_slot_view(k).c());
        let value = ((self.game_state.frame.frame_counter >> 3) & 1) + BARI_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).delay_main() != 0 {
                let value = ((self.game_state.frame.frame_counter >> 1) & 2) + BARI_GRAPHICS[j];
                self.sprite_slot_view_mut(k).set_graphics(value);
                return;
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        } else if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        } else if (self.get_random_number() & 1) == 0 {
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            return;
        }
        self.red_bari_set_electrocute_delay(k);
    }

    fn red_bari_set_electrocute_delay(&mut self, k: usize) {
        // ROM Sprite_23_RedBari $06:A342: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
        let value = self.get_random_number_with_carry().masked_adc(63, 128);
        self.sprite_slot_view_mut(k).set_delay_aux1(value);
    }

    // -----------------------------------------------------------------------
    // void RedBari_Draw(int k) {  // 86a3dc
    //   Four 8x8 body tiles selected by sprite_graphics.
    // }
    pub(super) fn red_bari_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.sprite_slot_view(k).graphics() as usize).min(1) * 4;
        self.sprite_draw_multiple(k, &RED_BARI_DRAW_FRAMES[base..base + 4], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void HardHatBeetle_Draw(int k) {  // 86a4f2
    //   Two large body tiles, with a shadow only for shadow-enabled sprites.
    // }
    pub(super) fn hard_hat_beetle_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.sprite_slot_view(k).graphics() as usize).min(1) * 2;
        self.sprite_draw_multiple(
            k,
            &HARD_HAT_BEETLE_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
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
            self.set_oam_helper0_at_for_draw(
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
    // void Octorock_Draw(int k) {  // 86d54a
    //   Optional mouth tile followed by the canonical large body tile.
    // }
    pub(super) fn octorock_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = SpritePrepOamCoordsRet { x, y, r4: 0, flags };
        if self.sprite_slot_view(k).direction() != 3 {
            let oam = self.game_state.oam.current_pointer_usize();
            let j = (self.sprite_slot_view(k).c() as usize) * 3
                + self.sprite_slot_view(k).direction() as usize;
            self.set_oam_helper0_at_for_draw(
                oam,
                x.wrapping_add(OCTOROCK_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(OCTOROCK_DRAW_Y_OFFSETS[j] as i16 as u16),
                OCTOROCK_DRAW_CHARS[j],
                OCTOROCK_DRAW_FLAGS[j] | flags,
                0,
            );
        }
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_prep_and_draw_single_large_no_prep(k, &mut info);
        self.sprite_slot_view_mut(k).add_flags2(1);
    }

    // -----------------------------------------------------------------------
    // uint8 FluteBoy_Draw(int k) {  // 8dcfd9
    //   Four-tile body draw; returns whether the prepped coordinate was offscreen.
    // }
    pub(super) fn flute_boy_draw(&mut self, k: usize) -> u8 {
        self.oam_allocate_from_region_b(0x10);
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.sprite_slot_view(k).direction() as usize) * 8
            + (self.sprite_slot_view(k).graphics() as usize) * 4;
        self.sprite_draw_multiple(k, &FLUTE_BOY_DRAW_FRAMES[base..base + 4], Some(&mut info));
        ((info.x | info.y) >> 8) as u8
    }

    // -----------------------------------------------------------------------
    // void FluteAardvark_Draw(int k) {  // 8dd040
    //   Two deferred large tiles selected by sprite_graphics.
    // }
    pub(super) fn flute_aardvark_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &FLUTE_AARDVARK_DRAW_FRAMES[base..base + 2],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void DustCloud_Draw(int k) {  // 8dd120
    //   Four cloud tiles selected by sprite_graphics.
    // }
    pub(super) fn dust_cloud_draw(&mut self, k: usize) {
        let value = 0x14;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        self.sprite_draw_multiple(k, &DUST_CLOUD_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_DustCloud(int k) {  // 85f2b2
    pub(super) fn sprite_dust_cloud(&mut self, k: usize) {
        self.dust_cloud_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        let value = 5;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let a = self.sprite_slot_view(k).a() as usize;
        if !sign8(DUST_CLOUD_GRAPHICS[a]) {
            let value = DUST_CLOUD_GRAPHICS[a];
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_slot_view_mut(k).add_a(1);
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
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
        self.sprite_draw_multiple(k, &LANDMINE_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void Armos_Draw(int k) {  // 85b7ef
    //   Two large stacked tiles plus a common shadow.
    // }
    pub(super) fn armos_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(k, &ARMOS_DRAW_FRAMES, Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void ArmosKnight_Draw(int k) {  // 85a274
    //   Four body tiles, plus the ground shadow when standing.
    // }
    pub(super) fn armos_knight_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        if self.sprite_slot_view(k).a() == 0 && self.game_state.frame.submodule != 7 {
            self.oam_allocate_defer_to_player(k);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        for i in (0..4).rev() {
            let j = g * 4 + i;
            self.set_oam_helper0_at_for_draw(
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
        self.set_oam_helper0_at_for_draw(
            oam + 16,
            x.wrapping_sub(8).wrapping_add(z as u16),
            y.wrapping_add(12),
            0xe4,
            0x25,
            2,
        );
        self.set_oam_helper0_at_for_draw(
            oam + 20,
            x.wrapping_add(8).wrapping_sub(z as u16),
            y.wrapping_add(12),
            0xe4,
            0x65,
            2,
        );
    }

    // -----------------------------------------------------------------------
    // void Boulder_Draw(int k) {  // 9dd185
    //   Four large rotating tiles plus large shadow.
    // }
    pub(super) fn boulder_draw(&mut self, k: usize) {
        let base = (((self.sprite_slot_view(k).subtype2() >> 3) & 3) as usize) * 4;
        self.sprite_draw_multiple(k, &BOULDER_DRAW_FRAMES[base..base + 4], None);
        self.sprite_draw_large_shadow2(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_DrawLargeShadow2(int k) {  // 9dd1af
    //   Clamp z-derived animation and delegate to SpriteDraw_BigShadow.
    // }
    pub(super) fn sprite_draw_large_shadow2(&mut self, k: usize) {
        let z = (self.sprite_slot_view(k).z() >> 3).min(4);
        self.sprite_draw_big_shadow(k, z as i32);
    }

    // -----------------------------------------------------------------------
    // void Flame_Draw(int k) {  // 9ec35c
    //   Two flame tiles selected by sprite_graphics.
    // }
    pub(super) fn flame_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        self.sprite_draw_multiple(k, &FLAME_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void SeekerEnergyBall_Draw(int k) {  // 9edc3e
    //   Four small orb tiles toggled by subtype animation.
    // }
    pub(super) fn seeker_energy_ball_draw(&mut self, k: usize) {
        let base = (((self.sprite_slot_view(k).subtype2() >> 2) & 1) as usize) * 4;
        self.sprite_draw_multiple(k, &ENERGY_BALL_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void Wizzbeam_Draw(int k) {  // 8dbe68
    //   Two small tiles selected by sprite_D.
    // }
    pub(super) fn wizzbeam_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).direction() as usize) * 2;
        self.sprite_draw_multiple(k, &WIZZBEAM_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void Freezor_Draw(int k) {  // 8dbfa6
    //   Four/eight-tile ice body selected by sprite_graphics.
    // }
    pub(super) fn freezor_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).graphics() != 7 {
            let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
            self.sprite_draw_multiple(k, &FREEZOR_DRAW_FRAMES0[base..base + 4], None);
        } else {
            self.sprite_draw_multiple(k, &FREEZOR_DRAW_FRAMES1, None);
        }
    }

    // -----------------------------------------------------------------------
    // void Ropa_Draw(int k) {  // 869ee5
    //   Three body tiles plus common shadow.
    // }
    pub(super) fn ropa_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base =
            ((self.sprite_slot_view(k).graphics() as usize) * 3).min(ROPA_DRAW_FRAMES.len() - 3);
        self.sprite_draw_multiple(k, &ROPA_DRAW_FRAMES[base..base + 3], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Zazak_Draw(int k) {  // 8dc0a6
    //   Three body tiles, head-char patch, and common shadow.
    // }
    pub(super) fn zazak_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.sprite_slot_view(k).graphics() as usize) * 3;
        self.sprite_draw_multiple(k, &ZAZAK_DRAW_FRAMES[base..base + 3], Some(&mut info));
        if self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let i = (self.sprite_slot_view(k).head_direction() as usize)
            + if self.sprite_slot_view(k).delay_aux1() == 0 {
                0
            } else {
                4
            };
        let oam = self.game_state.oam.current_pointer_usize();
        self.oam_state_mut()
            .set_entry_char(oam, ZAZAK_DRAW_CHARS[i]);
        self.oam_state_mut()
            .merge_entry_flags(oam, !0x40, ZAZAK_DRAW_FLAGS[i]);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Leever_Draw(int k) {  // 86ce45
    //   Variable one/four tile body selected by sprite_graphics.
    // }
    pub(super) fn leever_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = (self.sprite_slot_view(k).graphics() as usize).min(13);
        for i in (0..=LEEVER_DRAW_COUNTS[d] as usize).rev() {
            let j = d * 4 + i;
            let charnum = LEEVER_DRAW_CHARS[j];
            let mut f = flags;
            if charnum >= 0x60 || charnum == 0x28 || charnum == 0x38 {
                f &= 0xf0;
            }
            self.set_oam_helper0_at_for_draw(
                oam,
                x.wrapping_add(LEEVER_DRAW_X_OFFSETS[j] as i16 as u16),
                y.wrapping_add(LEEVER_DRAW_Y_OFFSETS[j] as i16 as u16),
                charnum,
                LEEVER_DRAW_FLAGS[j] | f,
                LEEVER_DRAW_SIZES[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Pengator_Draw(int k) {  // 9ea415
    //   Two body tiles, optional foot tiles, then common shadow.
    // }
    pub(super) fn pengator_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let gfx = self.sprite_slot_view(k).graphics() as usize;
        let base = (gfx * 2).min(PENGATOR_DRAW_FRAMES0.len() - 2);
        self.sprite_draw_multiple(k, &PENGATOR_DRAW_FRAMES0[base..base + 2], Some(&mut info));
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
            self.sprite_draw_multiple(
                k,
                &PENGATOR_DRAW_FRAMES1[extra_base..extra_base + 2],
                Some(&mut info),
            );
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SpikeRoller_Draw(int k) {  // 858ee3
    //   Four or eight repeated roller tiles selected by sprite_graphics.
    // }
    pub(super) fn spike_roller_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_helper0_at_for_draw(
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
    // void MedallionTablet_Draw(int k) {  // 8dd1e2
    //   Four deferred tablet tiles selected by sprite_graphics.
    // }
    pub(super) fn medallion_tablet_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &MEDALLION_TABLET_DRAW_FRAMES[base..base + 4],
            None,
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
        self.sprite_move_xy(k);
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
            let j = self.sprite_direction_to_face_link(k, None) as usize;
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
                let j = self.sprite_direction_to_face_link(k, None) as usize;
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
            let mut x = self
                .sprite_get_x(k)
                .wrapping_add(u16::from(MOVABLE_STATUE_SWITCH_X_OFFSETS[j]));
            let y = self
                .sprite_get_y(k)
                .wrapping_add(u16::from(MOVABLE_STATUE_SWITCH_Y_OFFSETS[j]));
            let t = self.GetTileAttribute(self.sprite_slot_view(k).floor(), &mut x, y);
            if t != 0x23 && t != 0x24 && t != 0x25 && t != 0x3b {
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
        self.sprite_draw_multiple_player_deferred(k, &MOVABLE_STATUE_DRAW_FRAMES, None);
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
                let pt = self.sprite_project_speed_towards_location(k, x, y, 32);
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
    pub(super) fn movable_mantle_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(0x20);
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
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
        }
        self.sprite_correct_oam_entries_for_draw(k, 5, 2);
    }

    // -----------------------------------------------------------------------
    // void Sprite_EE_MovableMantle(int k) {  // 85e819
    pub(super) fn sprite_ee_movable_mantle(&mut self, k: usize) {
        self.movable_mantle_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if !self.sprite_check_damage_to_link_same_layer(k) {
            return;
        }
        self.sprite_nullify_hookshot_drag();
        self.sprite_repel_dash();

        if self.game_state.sprites.follower_runtime.indicator() != 1
            || self.game_state.inventory.items.torch() == 0
            || self.game_state.player.follower_link.is_running()
            || self.sprite_slot_view(k).g() == 0x90
            || sign8(
                self.game_state
                    .player
                    .follower_link
                    .actual_x_velocity()
                    .wrapping_sub(24),
            )
        {
            return;
        }

        self.save_progress_mut().set_which_starting_point(4);
        self.sprite_slot_view_mut(k).add_subtype2(1);

        if (self.sprite_slot_view(k).subtype2() & 1) == 0 {
            self.sprite_slot_view_mut(k).add_g(1);
        }

        if self.sprite_slot_view(k).g() < 8 {
            return;
        }
        if self.game_state.system_signals.sound_effect_1() == 0 {
            self.set_sound_effect_1(34);
        }
        let value = 2;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy(k);
    }

    // -----------------------------------------------------------------------
    // void Fish_Draw(int k) {  // 9d8483
    //   Two fish body tiles plus three splash tiles at z-adjusted y.
    // }
    pub(super) fn fish_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).graphics() == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
            return;
        }
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        self.sprite_workspace_mut()
            .set_current_sprite_x(cur_x.wrapping_add(4));
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics().wrapping_sub(1)) * 2;
        self.sprite_draw_multiple(k, &FISH_DRAW_FRAMES[base..base + 2], Some(&mut info));
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let z = self.sprite_slot_view(k).z();
        self.sprite_workspace_mut()
            .set_current_sprite_y(cur_y.wrapping_add(z as u16));
        let j = (z >> 2).min(2) as usize;
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
        self.sprite_draw_multiple(k, &FISH_DRAW_FRAMES2[j * 3..j * 3 + 3], Some(&mut info));
        self.sprite_get16_bit_coords(k);
    }

    // -----------------------------------------------------------------------
    // void ChimneySmoke_Draw(int k) {  // 9d8531
    //   Four smoke tiles selected by low sprite_graphics bit.
    // }
    pub(super) fn chimney_smoke_draw(&mut self, k: usize) {
        let base = ((self.sprite_slot_view(k).graphics() & 1) as usize) * 4;
        self.sprite_draw_multiple(k, &CHIMNEY_SMOKE_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void Vulture_Draw(int k) {  // 9ddd5e
    //   Two large wing/body tiles plus common shadow.
    // }
    pub(super) fn vulture_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &VULTURE_DRAW_FRAMES[base..base + 2], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Raven(int k) {  // 9ddd85
    //   Raven wait, ascend, attack, and flee state machine.
    // }
    pub(super) fn sprite_raven(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let r = self.sprite_is_right_of_link(k);
                let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | r.a.wrapping_mul(0x40);
                self.sprite_slot_view_mut(k).set_oam_flags(value);
                let x = i32::from(self.game_state.player.follower_link.x())
                    - i32::from(self.game_state.sprites.workspace.current_sprite_x());
                let y = i32::from(self.game_state.player.follower_link.y())
                    - i32::from(self.game_state.sprites.workspace.current_sprite_y());
                if ((x + 0x50 + i32::from(x >= 0)) as u16) < 0x00a0
                    && ((y + 0x58 + i32::from(y >= 0)) as u16) < 0x00a0
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 24;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let j = usize::from(self.sprite_slot_view(k).a());
                    let value = RAVEN_ASCEND_TIMERS[j];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_apply_speed_towards_link(k, 32);
                }
                self.sprite_slot_view_mut(k).add_z(1);
                let value = ((self.game_state.frame.frame_counter >> 1) & 1).wrapping_add(1);
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0
                    && !(self.game_state.world.region.is_in_dark_world()
                        && self.sprite_slot_view(k).a() != 0)
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
                self.raven_fly(k, false);
            }
            3 => {
                self.raven_fly(k, true);
            }
            _ => {}
        }
    }

    fn raven_fly(&mut self, k: usize, fleeing: bool) {
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) == 0 {
            let mut pt = self.sprite_project_speed_towards_link(k, if fleeing { 48 } else { 32 });
            if fleeing {
                pt.x = pt.x.wrapping_neg();
                pt.y = pt.y.wrapping_neg();
            }
            let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt.x);
            if dx != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(if sign8(dx) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt.y);
            if dy != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(if sign8(dy) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        let value = ((self.game_state.frame.frame_counter >> 1) & 1).wrapping_add(1);
        self.sprite_slot_view_mut(k).set_graphics(value);
        let j = (self.sprite_slot_view(k).x_velocity() >> 7) & 1;
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | j.wrapping_mul(0x40);
        self.sprite_slot_view_mut(k).set_oam_flags(value);
    }

    // -----------------------------------------------------------------------
    // void MagicPowderItem_Draw(int k) {  // 85f67b
    //   Two deferred large tiles.
    // }
    pub(super) fn magic_powder_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &MAGIC_POWDER_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void GreenPotionItem_Draw(int k) {  // 85f718
    //   Bottle body and green price/label tiles.
    // }
    pub(super) fn green_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &GREEN_POTION_ITEM_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void BluePotionItem_Draw(int k) {  // 85f7bd
    //   Bottle body and blue price/label tiles.
    // }
    pub(super) fn blue_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &BLUE_POTION_ITEM_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void RedPotionItem_Draw(int k) {  // 85f86d
    //   Bottle body and red price/label tiles.
    // }
    pub(super) fn red_potion_item_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &RED_POTION_ITEM_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_E9_PotionShop(int k) {  // 85f633
    pub(super) fn sprite_e9_potion_shop(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_magic_shop_assistant_main(k),
            1 => self.sprite_bag_of_powder(k),
            2 => self.sprite_green_cauldron(k),
            3 => self.sprite_blue_cauldron(k),
            4 => self.sprite_red_cauldron(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_F2_MedallionTablet(int k) {  // 86c00d
    pub(super) fn sprite_f2_medallion_tablet(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.medallion_tablet_main(k),
            1 => self.sprite_dust_cloud(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BagOfPowder(int k) {  // 85f644
    pub(super) fn sprite_bag_of_powder(&mut self, k: usize) {
        self.magic_powder_item_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if !self.sprite_check_damage_to_link_same_layer(k)
            || (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0
        {
            return;
        }
        self.link_cancel_dash();
        self.follower_link_state_mut().set_item_receipt_method(0);
        if self
            .link_receive_item_from(
                0x0d,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::PotionShopPowder,
                },
            )
            .is_suspended()
        {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_GreenCauldron(int k) {  // 85f68e
    pub(super) fn sprite_green_cauldron(&mut self, k: usize) {
        self.green_potion_item_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.potion_cauldron_purchase(k, 60, 0x2f);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BlueCauldron(int k) {  // 85f72b
    pub(super) fn sprite_blue_cauldron(&mut self, k: usize) {
        self.blue_potion_item_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.potion_cauldron_purchase(k, 160, 0x30);
    }

    // -----------------------------------------------------------------------
    // void Sprite_RedCauldron(int k) {  // 85f7d0
    pub(super) fn sprite_red_cauldron(&mut self, k: usize) {
        self.red_potion_item_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.potion_cauldron_purchase(k, 120, 0x2e);
    }

    fn potion_cauldron_purchase(&mut self, k: usize, price: u16, item: u8) {
        self.sprite_behave_as_barrier(k);
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        if !self.potion_cauldron_check_bottles() {
            if self.sprite_show_message_on_contact(k, 0x4f) & 0x100 != 0 {
                self.potion_cauldron_go_beep(k);
            }
            return;
        }
        if !self.sprite_check_damage_to_link_same_layer(k)
            || (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0
        {
            return;
        }
        if self.game_state.inventory.player_resources.rupees_goal() < price {
            self.sprite_show_message_unconditional(0x17c);
            self.potion_cauldron_go_beep(k);
            return;
        }
        if self.sprite_find_empty_bottle() < 0 {
            self.sprite_show_message_unconditional(0x50);
            self.potion_cauldron_go_beep(k);
            return;
        }
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x1d);
        let value = 64;
        self.sprite_slot_view_mut(k).set_delay_main(value);
        let rupees = self
            .game_state
            .inventory
            .player_resources
            .rupees_goal()
            .wrapping_sub(price);
        self.player_resources_mut().set_rupees_goal(rupees);
        self.follower_link_state_mut().set_item_receipt_method(0);
        let _ = self.link_receive_item_from(
            item,
            0,
            ItemReceiptCaller::SpriteMainDirect {
                sprite_slot: k as u8,
                suffix: SpriteMainItemReceiptSuffix::PotionCauldron,
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_MagicShopAssistant_Main(int k) {  // 85f893
    pub(super) fn sprite_magic_shop_assistant_main(&mut self, k: usize) {
        self.shopkeeper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.player_resources_mut().set_heart_filler(160);
            let value = 0;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        let value = self.game_state.frame.frame_counter >> 5 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let msg = if self.game_state.inventory.items.bottle(0) >= 2
            || self.game_state.inventory.items.bottle(1) >= 2
            || self.game_state.inventory.items.bottle(2) >= 2
            || self.game_state.inventory.items.bottle(3) >= 2
            || !self.game_state.world.region.flag_overworld_area_changed()
        {
            0x4e
        } else {
            0x4d
        };
        if self.sprite_show_solicited_message(k, msg) & 0x100 != 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
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
        if self.sprite_check_damage_to_link_same_layer(k) {
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
            || self.sprite_direction_to_face_link(k, None) != 2
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
            || self.sprite_direction_to_face_link(k, None) != 2
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
        if !self.sprite_check_damage_to_link_same_layer(k) {
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
        if !self.sprite_check_damage_to_link_same_layer(k) {
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
        let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_plain_at_for_draw(
                oam,
                x.wrapping_add(BAD_PULL_DOWN_SWITCH_X_OFFSETS[i] as i16 as u16) as u8,
                y.wrapping_add(y_delta as u16) as u8,
                BAD_PULL_DOWN_SWITCH_DRAW_CHARS[i],
                BAD_PULL_DOWN_SWITCH_DRAW_FLAGS[i] | 0x21,
                BAD_PULL_DOWN_SWITCH_DRAW_SIZES[i],
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries_for_draw(k, 4, 0xff);
    }

    // -----------------------------------------------------------------------
    // void BadPullUpSwitch_Draw(int k) {  // 85d858
    //   Two large switch tiles with animated top offset.
    // }
    pub(super) fn bad_pull_up_switch_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = (self.sprite_slot_view(k).graphics() as usize)
            .min(BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS.len() - 1);
        let yoff = BAD_PULL_SWITCH_TOP_Y_OFFSETS
            [BAD_PULL_UP_SWITCH_DRAW_BAD_PULL_SWITCH_Y_OFFSET_INDEX_BY_GRAPHICS[g] as usize];
        for i in (0..2).rev() {
            self.set_oam_helper0_at_for_draw(
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
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let oam = self.game_state.oam.current_pointer_usize();
        let t = GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS[(self.sprite_slot_view(k).graphics() as usize)
            .min(GOOD_PULL_SWITCH_BOTTOM_Y_OFFSETS.len() - 1)];
        self.set_oam_helper0_at_for_draw(oam, x, y.wrapping_sub(1), 0xee, flags, 2);
        self.set_oam_helper0_at_for_draw(
            oam + 4,
            x,
            y.wrapping_sub(1).wrapping_add(t as u16),
            0xce,
            flags,
            2,
        );
        self.sprite_correct_oam_entries_for_draw(k, 1, 2);
    }

    // -----------------------------------------------------------------------
    // void BugNetKid_Draw(int k) {  // 8dd47b
    //   Six deferred tiles selected by sprite_graphics.
    // }
    pub(super) fn bug_net_kid_draw(&mut self, k: usize) {
        let base = (self.sprite_slot_view(k).graphics() as usize) * 6;
        self.sprite_draw_multiple_player_deferred(
            k,
            &BUG_NET_KID_DRAW_FRAMES[base..base + 6],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void Bomber_Draw(int k) {  // 8dd56c
    //   Two body tiles selected by sprite_graphics plus common shadow.
    // }
    pub(super) fn bomber_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.sprite_slot_view(k).graphics() as usize) * 2;
        self.sprite_draw_multiple(k, &BOMBER_DRAW_FRAMES[base..base + 2], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_93_Bumper(int k) {  // 9ea982
    pub(super) fn sprite_93_bumper(&mut self, k: usize) {
        self.bumper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_tile_collision(k);
        if !self.game_state.player.follower_link.is_cape_active()
            && self.sprite_check_damage_to_link_same_layer(k)
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
                let pt = self.sprite_project_speed_towards_location(k, x, y, 0x40);
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
        self.sprite_draw_multiple(k, &BUMPER_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void FakeSword_Draw(int k) {  // 85eee6
    //   Sprite_DrawMultiplePlayerDeferred(k, kFakeSword_Dmd, 2, NULL);
    // }
    pub(super) fn fake_sword_draw(&mut self, k: usize) {
        self.sprite_draw_multiple_player_deferred(k, &FAKE_SWORD_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_E7_Mushroom(int k) {  // 85ee78
    pub(super) fn sprite_e7_mushroom(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_check_if_link_is_busy() {
            return;
        }

        if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_DRAW)
            && self.game_state.frame.submodule != 0
        {
            return;
        }

        if self.sprite_check_damage_to_link_same_layer(k) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_link_state_mut().set_item_receipt_method(0);
            // Nothing follows the source call; the typed suffix is empty.
            let _ = self.link_receive_item_from(
                0x29,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::Mushroom,
                },
            );
        } else if (self.game_state.frame.frame_counter & 0x1f) == 0 {
            self.sprite_slot_view_mut(k).xor_oam_flags(0x40);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_E8_FakeSword(int k) {  // 85eeaf
    pub(super) fn sprite_e8_fake_sword(&mut self, k: usize) {
        self.fake_sword_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.sprite_slot_view(k).draw_work_byte_3() == 3 {
            if self.sprite_slot_view(k).c() == 0 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_c(value);
                self.sprite_show_message_unconditional(0x6f);
            }
        } else {
            self.sprite_move_xy(k);
            self.thrown_sprite_tile_and_sprite_interaction(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Aginah(int k) {  // 85f0ea
    pub(super) fn sprite_aginah(&mut self, k: usize) {
        let msg = if (self.game_state.inventory.save_progress.progress_flags() & 0x20) == 0 {
            self.save_progress_mut().or_progress_flags(0x20);
            0x125
        } else if self.game_state.inventory.items.sword_type() >= 2 {
            0x128
        } else if (self.game_state.inventory.player_resources.pendant_flags() & 7) == 7 {
            0x126
        } else if (self.game_state.inventory.player_resources.pendant_flags() & 2) != 0 {
            0x129
        } else if self.game_state.inventory.items.book() != 0 {
            0x127
        } else {
            self.save_progress_mut().or_progress_flags(0x20);
            0x125
        };
        self.sprite_show_solicited_message(k, msg);
        let value = (self.game_state.frame.frame_counter >> 5) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Sahasrahla(int k) {  // 85f14d
    pub(super) fn sprite_sahasrahla(&mut self, k: usize) {
        match self.sprite_slot_view(k).ai_state() {
            0 => self.sasha_idle(k),
            1 => {
                self.sprite_show_message_unconditional(0x33);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.save_progress_mut().set_map_icons_indicator(3);
            }
            2 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x4b,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::SahasrahlaBoots,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_sahasrahla_boots_item_receipt(k);
            }
            3 => {
                self.sprite_show_message_unconditional(0x37);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    /// Source suffix after Sprite_Sahasrahla's boots handout
    /// `Link_ReceiveItem(0x4b)` call (ROM `$85f14d`). A live timing authority
    /// suspends the decompressor there; only the AI-state advance and the
    /// map-icon indicator remain.
    pub(super) fn complete_sahasrahla_boots_item_receipt(&mut self, k: usize) {
        let value = 3;
        self.sprite_slot_view_mut(k).set_ai_state(value);
        self.save_progress_mut().set_map_icons_indicator(3);
    }

    // -----------------------------------------------------------------------
    // void Sprite_16_Elder_bounce(int k) {  // 86c08a
    //   Dispatcher for Sahasrahla / Aginah variants.
    // }
    pub(super) fn sprite_16_elder_bounce(&mut self, k: usize) {
        self.elder_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_sahasrahla(k),
            1 => self.sprite_aginah(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Waterfall(int k) {  // 9af5b8
    pub(super) fn waterfall(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            if self.game_state.world.location.overworld_screen_index() == 0x43 {
                self.ancilla_add_gt_cutscene();
            } else {
                self.ancilla_add_waterfall_splash();
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_37_Waterfall(int k) {  // 86c03a
    pub(super) fn sprite_37_waterfall(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.waterfall(k),
            1 => self.sprite_bat_crash(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BatCrash(int k) {  // 9af5d9
    pub(super) fn sprite_bat_crash(&mut self, k: usize) {
        self.retreat_bat_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.bat_crash_draw_hardcoded_garbage(k);
        self.set_bg1_y_offset(0);
        if self.sprite_slot_view(k).delay_aux3() != 0 {
            if self.sprite_slot_view(k).delay_aux3() == 1 {
                self.set_ambient_sound_effect(5);
            }
            let offs = if (self.sprite_slot_view(k).delay_aux3() & 1) != 0 {
                1u16
            } else {
                (-1i16) as u16
            };
            self.set_bg1_y_offset(offs);
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).graphics() == 0 && self.sprite_slot_view(k).ai_state() < 2 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 3);
            }
            let value = SPRITE_BAT_CRASH_DELAY[usize::from(self.sprite_slot_view(k).direction())];
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let j = usize::from(self.sprite_slot_view(k).a());
                if SPRITE_BAT_CRASH_X_POSITIONS[j]
                    < self.game_state.sprites.workspace.current_sprite_x()
                {
                    if j >= 2 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 208;
                        self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    }
                    self.sprite_slot_view_mut(k).add_a(1);
                    self.sprite_slot_view_mut(k).add_direction(1);
                }
                self.retreat_bat_update_pos(k, j, &SPRITE_BAT_CRASH_Y_POSITIONS);
            }
            1 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                    self.sprite_slot_view_mut(k).add_direction(1);
                    let value = 232;
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value = 7;
                    self.sprite_slot_view_mut(k).set_x_high(value);
                    let value = 224;
                    self.sprite_slot_view_mut(k).set_y_low(value);
                    let value = 5;
                    self.sprite_slot_view_mut(k).set_y_high(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 64;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 45;
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                } else {
                    if (self.game_state.frame.frame_counter & 3) == 0 {
                        self.sprite_slot_view_mut(k).subtract_x_velocity(1);
                    }
                    let j = usize::from(self.sprite_slot_view(k).a());
                    self.retreat_bat_update_pos(k, j, &SPRITE_BAT_CRASH_Y_POSITIONS);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 96;
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
                if self.sprite_slot_view(k).delay_aux1() == 9 {
                    self.bat_crash_spawn_debris(k);
                    self.create_pyramid_hole_for_draw();
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                    self.increment_overworld_map_state();
                }
            }
            _ => {}
        }
    }

    fn retreat_bat_update_pos(&mut self, k: usize, j: usize, ypos: &[u16; 4]) {
        if (self.game_state.frame.frame_counter & 7) == 0 {
            let delta = if ypos[j] >= self.game_state.sprites.workspace.current_sprite_y() {
                1u8
            } else {
                (-1i8) as u8
            };
            self.sprite_slot_view_mut(k).add_y_velocity(delta);
        }
        if (self.game_state.frame.frame_counter & 15) == 0 {
            self.sprite_slot_view_mut(k).add_x_velocity(1);
        }
    }

    // -----------------------------------------------------------------------
    // void BatCrash_SpawnDebris(int k) {  // 9af7e5
    pub(super) fn bat_crash_spawn_debris(&mut self, k: usize) {
        for j in (0..30).rev() {
            self.garnish_spawn_pyramid_debris(
                BAT_CRASH_SPAWN_DEBRIS_X_OFFSETS[j],
                BAT_CRASH_SPAWN_DEBRIS_Y_OFFSETS[j],
                BAT_CRASH_SPAWN_DEBRIS_LOCAL_X_VELOCITIES[j],
                BAT_CRASH_SPAWN_DEBRIS_LOCAL_Y_VELOCITIES[j],
            );
        }
        let value = 32;
        self.sprite_slot_view_mut(k).set_delay_aux3(value);
    }

    // -----------------------------------------------------------------------
    // void Overworld_DrawWoodenDoor(uint16 pos, bool unlocked) {  // 9bc952
    pub(super) fn overworld_draw_wooden_door(&mut self, pos: u16, unlocked: bool) {
        self.overworld_draw_map16_persist_for_draw(pos, if unlocked { 0x0da5 } else { 0x0da4 });
        self.overworld_draw_map16_persist_for_draw(
            pos.wrapping_add(2),
            if unlocked { 0x0da7 } else { 0x0da6 },
        );
        self.set_bg_vram_load_mode(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_ChickenLady(int k) {  // 9afed3
    pub(super) fn sprite_chicken_lady(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_direction(value);
        self.lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 1 {
            self.dialogue_message_index_mut().set_value(0x17d);
            self.sprite_show_message_minimal_c();
        }
        let value = (self.game_state.frame.frame_counter >> 4) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
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
                if self.sprite_check_damage_to_link(k) {
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
    // void Sprite_CF_Swamola(int k) {  // 9d9cb0
    pub(super) fn sprite_cf_swamola(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            if sign8(self.sprite_slot_view(k).ai_state()) {
                self.sprite_swamola_ripples(k);
                return;
            }
            self.swamola_draw(k);
        }
        self.sprite_get16_bit_coords(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_check_damage_to_and_from_link(k);
        let old_vel = self.sprite_slot_view(k).y_velocity();
        let value = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_add(self.sprite_slot_view(k).z_velocity());
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_move_xy(k);
        let value = old_vel;
        self.sprite_slot_view_mut(k).set_y_velocity(value);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = SPRITE_CF_SWAMOLA_TARGET_DIR[usize::from(self.get_random_number() & 7)];
                    if j == self.sprite_slot_view(k).direction() {
                        return;
                    }
                    let t = (u16::from(self.sprite_slot_view(k).b()) << 8)
                        | u16::from(self.sprite_slot_view(k).a());
                    let target_x =
                        t.wrapping_add_signed(i16::from(SWAMOLA_TARGET_X_OFFSETS[usize::from(j)]));
                    let t = (u16::from(self.sprite_slot_view(k).head_direction()) << 8)
                        | u16::from(self.sprite_slot_view(k).c());
                    let target_y =
                        t.wrapping_add_signed(i16::from(SWAMOLA_TARGET_Y_OFFSETS[usize::from(j)]));
                    self.swamola_target_mut(k).set_position(target_x, target_y);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = (-15i8) as u8;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.swamola_spawn_ripples(k);
                }
            }
            1 => {
                if (self.sprite_slot_view(k).subtype2() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_z_velocity(1);
                    if self.sprite_slot_view(k).z_velocity() == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let pt = self.swamola_project_velocity_towards_target(k);
                    self.sprite_approach_target_speed(k, pt.x, pt.y);
                }
            }
            2 => {
                let j = usize::from(self.sprite_slot_view(k).g() & 1);
                let value = self
                    .sprite_slot_view(k)
                    .z_velocity()
                    .wrapping_add(SPRITE_CF_SWAMOLA_Z_ACCEL[j] as u8);
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if self.sprite_slot_view(k).z_velocity() == SPRITE_CF_SWAMOLA_Z_VEL_TARGET[j] as u8
                {
                    self.sprite_slot_view_mut(k).add_g(1);
                }
                let x = self
                    .game_state
                    .effects
                    .sprite_histories
                    .swamola_target(k)
                    .x();
                let y = self
                    .game_state
                    .effects
                    .sprite_histories
                    .swamola_target(k)
                    .y();
                if self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_x()
                    .wrapping_sub(x)
                    .wrapping_add(8)
                    < 16
                    && self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(y)
                        .wrapping_add(8)
                        < 16
                {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                let pt = self.swamola_project_velocity_towards_target(k);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = pt.y;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            3 => {
                if (self.sprite_slot_view(k).subtype2() & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_z_velocity(1);
                    if self.sprite_slot_view(k).z_velocity() == 16 {
                        let value = 4;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.swamola_spawn_ripples(k);
                        let value = 128;
                        self.sprite_slot_view_mut(k).set_y_high(value);
                        let value = 80;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                    self.sprite_approach_target_speed(k, 0, 0);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = SPRITE_CF_SWAMOLA_TARGET_DIR[usize::from(self.get_random_number() & 7)];
                    let value = j;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let x = (u16::from(self.sprite_slot_view(k).b()) << 8)
                        | u16::from(self.sprite_slot_view(k).a());
                    self.sprite_set_x(
                        k,
                        x.wrapping_add_signed(i16::from(SWAMOLA_TARGET_X_OFFSETS[usize::from(j)])),
                    );
                    let y = (u16::from(self.sprite_slot_view(k).head_direction()) << 8)
                        | u16::from(self.sprite_slot_view(k).c());
                    self.sprite_set_y(
                        k,
                        y.wrapping_add_signed(i16::from(SWAMOLA_TARGET_Y_OFFSETS[usize::from(j)])),
                    );
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // ProjectSpeedRet Swamola_ProjectVelocityTowardsTarget(int k) {  // 9d9e13
    pub(super) fn swamola_project_velocity_towards_target(&mut self, k: usize) -> ProjectSpeedRet {
        let x = self
            .game_state
            .effects
            .sprite_histories
            .swamola_target(k)
            .x();
        let y = self
            .game_state
            .effects
            .sprite_histories
            .swamola_target(k)
            .y();
        self.sprite_project_speed_towards_location(k, x, y, 15)
    }

    // -----------------------------------------------------------------------
    // void Swamola_SpawnRipples(int k) {  // 9d9eaa
    pub(super) fn swamola_spawn_ripples(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xcf, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_spawned_coordinates(j, &info);
            let value = 128;
            self.sprite_slot_view_mut(j).set_ai_state(value);
            let value = 32;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 4;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            let value = 0;
            self.sprite_slot_view_mut(j).set_flags2(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Swamola_Ripples(int k) {  // 9d9ece
    pub(super) fn sprite_swamola_ripples(&mut self, k: usize) {
        self.swamola_ripples_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void SwamolaRipples_Draw(int k) {  // 9d9f1d
    pub(super) fn swamola_ripples_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(8);
        let base = usize::from((self.sprite_slot_view(k).delay_main() >> 2) & 3) * 2;
        self.sprite_draw_multiple(k, &SWAMOLA_RIPPLES_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void Swamola_Draw(int k) {  // 9d9f64
    pub(super) fn swamola_draw(&mut self, k: usize) {
        let j = usize::from(Self::sprite_convert_velocity_to_angle(
            self.sprite_slot_view(k).x_velocity(),
            self.sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(self.sprite_slot_view(k).z_velocity()),
        ));
        let value = SWAMOLA_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 63) | SWAMOLA_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);

        let hist = usize::from(self.sprite_slot_view(k).subtype2() & 0x1f) + k * 32;
        let x = self.sprite_slot_view(k).x();
        let y = self.sprite_slot_view(k).y();
        self.swamola_history_mut(hist).set_position(x, y);

        let (mut oam_step, mut ext_step): (i16, i16) =
            if sign8(self.sprite_slot_view(k).y_velocity()) {
                (5 * 4, 5)
            } else {
                (0, 0)
            };
        let delta: i16 = if sign8(self.sprite_slot_view(k).y_velocity()) {
            -1
        } else {
            1
        };
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add_signed(oam_step));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add_signed(ext_step));

        for i in 0..4 {
            let value = SWAMOLA_DRAW_SECONDARY_GRAPHICS[i];
            self.sprite_slot_view_mut(k).set_graphics(value);
            let j = usize::from(
                self.sprite_slot_view(k)
                    .subtype2()
                    .wrapping_sub(SWAMOLA_DRAW_HIST_OFFS[i])
                    & 31,
            ) + k * 32;
            let x = self
                .game_state
                .effects
                .sprite_histories
                .swamola_history(j)
                .x();
            let y = self
                .game_state
                .effects
                .sprite_histories
                .swamola_history(j)
                .y();
            self.sprite_workspace_mut().set_current_sprite_x(x);
            self.sprite_workspace_mut().set_current_sprite_y(y);
            oam_step = delta * 4;
            ext_step = delta;
            let cur = self.game_state.oam.current_pointer();
            self.oam_state_mut()
                .set_current_pointer(cur.wrapping_add_signed(oam_step));
            let ext = self.game_state.oam.current_extended_pointer();
            self.oam_state_mut()
                .set_current_extended_pointer(ext.wrapping_add_signed(ext_step));
            self.sprite_draw_single_large(k);
        }
        self.sprite_workspace_mut().set_shared_scratch_a(4);
    }

    // -----------------------------------------------------------------------
    // void WallMaster_Draw(int k) {  // 9eafe4
    pub(super) fn wall_master_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &WALL_MASTER_DRAW_FRAMES[base..base + 4], None);
        self.sprite_draw_large_shadow2(k);
    }

    // -----------------------------------------------------------------------
    // void Zol_Draw(int k) {  // 9eb1c5
    pub(super) fn zol_draw(&mut self, k: usize) {
        if (self.sprite_slot_view(k).oam_flags() & 1) == 0
            && self.game_state.sprites.system.chr_halfslot_state() >= 3
        {
            return;
        }
        if self.sprite_slot_view(k).delay_aux4() != 0 {
            self.oam_allocate_from_region_b(8);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
            return;
        }
        let gfx = self.sprite_slot_view(k).graphics();
        if gfx < 4 {
            let bak1 = self.sprite_slot_view(k).oam_flags();
            let value = bak1 ^ ZOL_DRAW_OAM_FLAGS[usize::from(gfx)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = gfx.wrapping_add(((self.sprite_slot_view(k).oam_flags() & 1) ^ 1) << 2);
            self.sprite_slot_view_mut(k).set_graphics(value);
            self.sprite_draw_single_large(k);
            let value = gfx;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = bak1;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let base = usize::from(gfx.wrapping_sub(4)) * 2;
            self.sprite_draw_multiple(k, &ZOL_DRAW_FRAMES[base..base + 2], None);
        }
    }

    // -----------------------------------------------------------------------
    // void Zoro(int k) {  // 9e9bd0
    pub(super) fn zoro(&mut self, k: usize) {
        if self.sprite_slot_view(k).c() == 0 {
            self.sprite_slot_view_mut(k).add_c(1);
            if self.sprite_is_below_link(k).a != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                return;
            }
        }
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 1) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            ZORO_X_VELOCITIES[usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 1)] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).delay_main() == 0 && self.sprite_check_tile_collision(k) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        if (self.sprite_slot_view(k).subtype2() & 3) != 0 {
            return;
        }
        let j = self.garnish_alloc();
        if j < 0 {
            return;
        }
        let j = j as usize;
        let value = 6;
        self.garnish_slot_view_mut(j).set_garnish_type(value);
        self.garnish_state_mut().set_active_type(6);
        self.garnish_set_x(j, self.sprite_get_x(k));
        self.garnish_set_y(j, self.sprite_get_y(k).wrapping_add(16));
        let value = 10;
        self.garnish_slot_view_mut(j).set_countdown(value);
        let value = k as u8;
        self.garnish_slot_view_mut(j).set_sprite(value);
        let value = self.sprite_slot_view(k).floor();
        self.garnish_slot_view_mut(j).set_floor(value);
    }

    // -----------------------------------------------------------------------
    // void Wizzrobe_Draw(int k) {  // 8dbe06
    pub(super) fn wizzrobe_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &WIZZROBE_DRAW_FRAMES[base..base + 3], None);
    }

    // -----------------------------------------------------------------------
    // void Wizzrobe_FireBeam(int k) {  // 9e9e15
    pub(super) fn wizzrobe_fire_beam(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x9b, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
            let value = 1;
            self.sprite_slot_view_mut(j).set_c(value);
            let value = 1;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            self.sprite_set_x(j, info.r0_x.wrapping_add(4));
            self.sprite_set_y(j, info.r2_y);
            let i = usize::from(self.sprite_slot_view(k).direction());
            let value = WIZZROBE_BEAM_XY_VELOCITIES[i] as u8;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = WIZZROBE_BEAM_XY_VELOCITIES[i + 2] as u8;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = 0x48;
            self.sprite_slot_view_mut(j).set_deflection_bits(value);
            let value = 2;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = if self.game_state.inventory.items.shield_type() == 3 {
                0x20
            } else {
                0
            };
            self.sprite_slot_view_mut(j).set_flags5(value);
            let value = 0x14;
            self.sprite_slot_view_mut(j).set_flags4(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_StalfosBone(int k) {  // 9e8fdf
    pub(super) fn sprite_stalfos_bone(&mut self, k: usize) {
        self.stalfos_bone_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).delay_main() == 0 && self.sprite_check_tile_collision(k) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_place_weapon_tink(k);
        }
    }

    // -----------------------------------------------------------------------
    // void StalfosBone_Draw(int k) {  // 9e9040
    pub(super) fn stalfos_bone_draw(&mut self, k: usize) {
        let base = usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3) * 2;
        self.sprite_draw_multiple(k, &STALFOS_BONE_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void Stalfos_ThrowBone(int k) {  // 9e9379
    pub(super) fn stalfos_throw_bone(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa7, &mut info);
        if j >= 0 {
            let j = j as usize;
            let value = 1;
            self.sprite_slot_view_mut(j).set_a(value);
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_apply_speed_towards_link(j, 32);
            let value = 0x21;
            self.sprite_slot_view_mut(j).set_flags2(value);
            let value = 33;
            self.sprite_slot_view_mut(j).set_ignore_projectile(value);
            self.sprite_slot_view_mut(j).or_flags3(0x40);
            let value = 0x48;
            self.sprite_slot_view_mut(j).set_deflection_bits(value);
            let value = 16;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = 0x14;
            self.sprite_slot_view_mut(j).set_flags4(value);
            let value = 7;
            self.sprite_slot_view_mut(j).set_oam_flags(value);
            let value = 32;
            self.sprite_slot_view_mut(j).set_bump_damage(value);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x02);
        }
    }

    // -----------------------------------------------------------------------
    // void Stalfos_Draw(int k) {  // 8dc21c
    pub(super) fn stalfos_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        if self.sprite_slot_view(k).delay_aux2() != 0 {
            self.sprite_prep_oam_coord(k, &mut info);
            return;
        }
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &STALFOS_DRAW_FRAMES[base..base + 3], Some(&mut info));
        if self.sprite_slot_view(k).graphics() < 8 && self.sprite_slot_view(k).pause() == 0 {
            let oam = self.game_state.oam.current_pointer_usize();
            let i = usize::from(self.sprite_slot_view(k).head_direction());
            self.oam_state_mut()
                .set_entry_char(oam, STALFOS_DRAW_CHARS[i]);
            self.oam_state_mut()
                .merge_entry_flags(oam, !0x70, STALFOS_DRAW_FLAGS[i]);
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void StalfosKnight_Draw(int k) {  // 9eae04
    pub(super) fn stalfos_knight_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_stalfos_knight_head(k, &info);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 5;
        self.sprite_draw_multiple_with_info(k, &STALFOS_KNIGHT_DRAW_FRAMES[base..base + 5], tuple);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_sub(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_sub(1));
        let mut shadow_info = SpritePrepOamCoordsRet {
            x: info.x,
            y: info.y,
            r4: info.r4,
            flags: info.flags,
        };
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
    }

    // -----------------------------------------------------------------------
    // void Trident_Draw(int k) {  // 9d9c1c
    pub(super) fn trident_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).g();
        if g == 0 {
            return;
        }
        let j = if g == 9 {
            3
        } else if g >= 9 {
            4
        } else {
            usize::from(self.sprite_slot_view(k).direction())
        };
        let cur_x = self
            .game_state
            .sprites
            .workspace
            .current_sprite_x()
            .wrapping_add_signed(i16::from(TRIDENT_DRAW_X_OFFSETS[j]));
        let cur_y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add_signed(i16::from(TRIDENT_DRAW_Y_OFFSETS[j]));
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
        let bak = self.sprite_slot_view(k).object_priority();
        self.sprite_slot_view_mut(k).and_object_priority(!0x0f);
        let base = usize::from(g.wrapping_sub(1)) * 5;
        self.sprite_draw_multiple(k, &TRIDENT_DRAW_FRAMES[base..base + 5], None);
        let value = bak;
        self.sprite_slot_view_mut(k).set_object_priority(value);
        self.sprite_get16_bit_coords(k);
    }

    // -----------------------------------------------------------------------
    // void TutorialSoldier_Draw(int k) {  // 85d64b
    pub(super) fn tutorial_soldier_draw(&mut self, k: usize) {
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_helper0_at_for_draw(
                oam,
                x.wrapping_add_signed(TUTORIAL_SOLDIER_DRAW_X_OFFSETS[j]),
                y.wrapping_add_signed(TUTORIAL_SOLDIER_DRAW_Y_OFFSETS[j]),
                TUTORIAL_SOLDIER_DRAW_CHARS[j],
                flags,
                TUTORIAL_SOLDIER_DRAW_BIG[j],
            );
            oam += 4;
        }
        let mut info = SpritePrepOamCoordsRet {
            x,
            y,
            r4: 0,
            flags: info_flags,
        };
        self.sprite_draw_shadow_custom(k, &mut info, 12);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D3_Stal(int k) {  // 9d8129
    pub(super) fn sprite_d3_stal(&mut self, k: usize) {
        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            if self.sprite_slot_view(k).ai_state() == 0 {
                self.oam_allocate_from_region_b(4);
            }
            self.stal_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = 64;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                    }
                }
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let delay = self.sprite_slot_view(k).delay_main().wrapping_sub(1);
                    if delay != 0 {
                        let value = delay | 64;
                        self.sprite_slot_view_mut(k).set_hit_timer(value);
                    } else {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_hit_timer(value);
                        self.sprite_slot_view_mut(k).and_flags3(!0x40);
                        self.sprite_slot_view_mut(k).and_flags2(!0x80);
                    }
                }
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xyz(k);
                self.sprite_check_tile_collision(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.sprite_apply_speed_towards_link(k, 12);
                }
                if (self.game_state.frame.frame_counter & 3) == 0 {
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    if self.sprite_slot_view(k).subtype2() == 5 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                    }
                }
                let value = SPRITE_D3_STAL_STAL_GFX[self.sprite_slot_view(k).subtype2() as usize];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Stal_Draw(int k) {  // 9d820c
    pub(super) fn stal_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let n = if self.sprite_slot_view(k).ai_state() != 0 {
            2
        } else {
            1
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &STAL_DRAW_FRAMES[base..base + n], Some(&mut info));
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
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
            self.sprite_move_xy(k);
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
    // void Sprite_BunnyBeam(int k) {  // 9d85e0
    pub(super) fn sprite_bunny_beam(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_check_tile_collision(k) == 0 {
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
            if self.sprite_check_damage_to_link(k) {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().set_temp_bunny_timer(256);
            }
            if self.game_state.player.follower_link.lower_level_state()
                == self.sprite_slot_view(k).floor()
            {
                self.sprite_apply_speed_towards_link(k, 16);
            }
            self.sprite_move_xy(k);
            if self.sprite_check_tile_collision(k) != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_spawn_poof_garnish(k);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_D0_Lynel(int k) {  // 9d866a
    pub(super) fn sprite_d0_lynel(&mut self, k: usize) {
        self.lynel_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let value = self.sprite_direction_to_face_link(k, None);
        self.sprite_slot_view_mut(k).set_direction(value);
        self.sprite_check_damage_to_and_from_link(k);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).direction());
                    let x = self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(SPRITE_D0_LYNEL_X_TARGETS[j] as i16 as u16);
                    let value = x as u8;
                    self.sprite_slot_view_mut(k).set_a(value);
                    let value = (x >> 8) as u8;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let y = self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(SPRITE_D0_LYNEL_Y_TARGETS[j] as i16 as u16);
                    let value = y as u8;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = (y >> 8) as u8;
                    self.sprite_slot_view_mut(k).set_e(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 80;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let idx = usize::from(
                    (self.sprite_slot_view(k).subtype2() & 4)
                        | self.sprite_slot_view(k).direction(),
                );
                let value = SPRITE_D0_LYNEL_LOCAL_GRAPHICS[idx];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                        let x = u16::from(self.sprite_slot_view(k).a())
                            | (u16::from(self.sprite_slot_view(k).b()) << 8);
                        let y = u16::from(self.sprite_slot_view(k).c())
                            | (u16::from(self.sprite_slot_view(k).e()) << 8);
                        if x.wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                            .wrapping_add(5)
                            < 10
                            && y.wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                                .wrapping_add(5)
                                < 10
                        {
                            self.lynel_increment_state(k);
                            return;
                        }
                        let pt = self.sprite_project_speed_towards_location(k, x, y, 24);
                        let value = pt.x;
                        self.sprite_slot_view_mut(k).set_x_velocity(value);
                        let value = pt.y;
                        self.sprite_slot_view_mut(k).set_y_velocity(value);
                    }
                    self.sprite_move_xy(k);
                    if self.sprite_check_tile_collision(k) != 0 {
                        self.lynel_increment_state(k);
                        return;
                    }
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    let idx = usize::from(
                        (self.sprite_slot_view(k).subtype2() & 4)
                            | self.sprite_slot_view(k).direction(),
                    );
                    let value = SPRITE_D0_LYNEL_LOCAL_GRAPHICS[idx];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    self.lynel_increment_state(k);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Sprite_D0_Lynel $1D:8745: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(15, 16);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 16 {
                    let j = self.sprite_spawn_fire_phlegm(k);
                    if j >= 0 && self.game_state.inventory.items.shield_type() != 3 {
                        let value = 0;
                        self.sprite_slot_view_mut(j as usize).set_flags5(value);
                    }
                }
                let value =
                    SPRITE_D0_LYNEL_ATTACK_GFX[self.sprite_slot_view(k).direction() as usize];
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_check_tile_collision(k);
            }
            _ => {}
        }
    }

    fn lynel_increment_state(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).add_ai_state(1);
        let value = 32;
        self.sprite_slot_view_mut(k).set_delay_main(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_E4_SmallKey(int k) {  // 86d032
    pub(super) fn sprite_e4_small_key(&mut self, k: usize) {
        let idx = usize::from(self.sprite_slot_view(k).die_action());
        if (self.game_state.dungeon.savegame_state.savegame_state_bits()
            & (ABSORB_BIG_KEY_MASKS_DRAW[idx] << 8))
            != 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        self.sprite_draw_ripple_if_in_water(k);
        if self.sprite_draw_absorbable_transient(k, false) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HeartContainer(int k) {  // 85ef47
    pub(super) fn sprite_heart_container(&mut self, k: usize) {
        if self.game_state.inventory.save_progress.palace_index_x2() == 26 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        let value = self.sprite_slot_view(k).g();
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).g() == 0 {
            self.DecodeAnimatedSpriteTile_variable(3);
            self.sprite_get16_bit_coords(k);
            let value = 1;
            self.sprite_slot_view_mut(k).set_g(value);
        }

        if self.game_state.dungeon.room_tracking.room_index2() == 6
            && self.sprite_slot_view(k).z() == 0
        {
            self.sprite_draw_water_ripple_with_oam_adjust(k);
        }
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        self.sprite_move_z(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if self.game_state.dungeon.room_tracking.room_index2() == 6
                && self.sprite_slot_view(k).subtype() == 0
            {
                self.sprite_slot_view_mut(k).add_flags2(2);
                let value = 1;
                self.sprite_slot_view_mut(k).set_subtype(value);
                self.sprite_spawn_water_splash(k);
            }
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        if !self.sprite_check_damage_to_link_same_layer(k) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        if self.sprite_slot_view(k).a() != 0 {
            self.follower_link_state_mut().set_item_receipt_method(2);
            if self
                .link_receive_item_from(
                    0x3e,
                    0,
                    ItemReceiptCaller::SpriteMainDirect {
                        sprite_slot: k as u8,
                        suffix: SpriteMainItemReceiptSuffix::HeartContainerFull,
                    },
                )
                .is_suspended()
            {
                return;
            }
            self.complete_heart_container_full_item_receipt();
            return;
        }
        self.link_cancel_dash();
        self.follower_link_state_mut().set_item_receipt_method(0);
        if self
            .link_receive_item_from(
                0x26,
                0,
                ItemReceiptCaller::SpriteMainDirect {
                    sprite_slot: k as u8,
                    suffix: SpriteMainItemReceiptSuffix::HeartContainerUpgrade,
                },
            )
            .is_suspended()
        {
            return;
        }
        self.complete_heart_container_upgrade_item_receipt(k);
    }

    /// Source suffix after Sprite_HeartContainer's crystal-branch
    /// `Link_ReceiveItem(0x3e)` call (ROM `$85ef47`). A live timing authority
    /// suspends the decompressor there; only the savegame-state bit remains.
    pub(super) fn complete_heart_container_full_item_receipt(&mut self) {
        let bits = self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x8000;
        self.dungeon_savegame_state_mut()
            .set_savegame_state_bits(bits);
    }

    /// Source suffix after Sprite_HeartContainer's piece-upgrade
    /// `Link_ReceiveItem(0x26)` call (ROM `$85ef47`).
    pub(super) fn complete_heart_container_upgrade_item_receipt(&mut self, k: usize) {
        self.heart_upgrade_set_obtained_flag(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HeartPiece(int k) {  // 85f020
    pub(super) fn sprite_heart_piece(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_slot_view_mut(k).add_ai_state(1);
            self.heart_upgrade_check_if_already_obtained(k);
            if self.sprite_slot_view(k).state() == 0 {
                return;
            }
        }
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_if_link_is_busy() {
            return;
        }

        if (self.sprite_check_tile_collision(k) & 3) != 0 {
            let value = 0u8.wrapping_sub(self.sprite_slot_view(k).x_velocity());
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }

        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        self.sprite_move_z(k);
        self.sprite_move_xy(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = ((self.sprite_slot_view(k).z_velocity() ^ 255) & 248) >> 1;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = (self.sprite_slot_view(k).x_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }

        if self.sprite_slot_view(k).delay_aux4() != 0
            || !self.sprite_check_damage_to_link_same_layer(k)
        {
            return;
        }

        let heart_pieces = self.player_resources_mut().advance_heart_piece_count();
        if heart_pieces == 0 {
            self.link_cancel_dash();
            self.follower_link_state_mut().set_item_receipt_method(0);
            if self
                .link_receive_item_from(
                    0x26,
                    0,
                    ItemReceiptCaller::SpriteMainDirect {
                        sprite_slot: k as u8,
                        suffix: SpriteMainItemReceiptSuffix::HeartPiece,
                    },
                )
                .is_suspended()
            {
                return;
            }
        } else {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2d);
            self.sprite_show_message_unconditional(HEART_PIECE_MESSAGES[usize::from(heart_pieces)]);
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_state(value);
        self.heart_upgrade_set_obtained_flag(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_D8_Heart(int k) {  // 86cec0
    pub(super) fn sprite_d8_heart(&mut self, k: usize) {
        if self.sprite_draw_absorbable_transient(k, true) {
            return;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_absorption_by_player(k);

        if self.sprite_slot_view(k).state() == 0
            && self
                .game_state
                .enhanced_features
                .has(FEATURE_MISC_BUG_FIXES_DRAW)
        {
            return;
        }

        if self.sprite_handle_dragging_by_ancilla(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.sprite_slot_view_mut(k).and_oam_flags(!0x40);
        if !sign8(self.sprite_slot_view(k).x_velocity()) {
            self.sprite_slot_view_mut(k).or_oam_flags(0x40);
        }
        match self.sprite_slot_view(k).ai_state().min(3) {
            0 => {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = 18;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 20;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                } else {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 253;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).direction() & 1);
                    let value = self
                        .sprite_slot_view(k)
                        .x_velocity()
                        .wrapping_add(HEART_REFILL_X_ACCELERATIONS[j] as u8);
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if self.sprite_slot_view(k).x_velocity()
                        == HEART_REFILL_X_VELOCITY_TARGETS[j] as u8
                    {
                        self.sprite_slot_view_mut(k).add_direction(1);
                        let value = 8;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
            }
            _ => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_E3_Fairy(int k) {  // 86cf94
    pub(super) fn sprite_e3_fairy(&mut self, k: usize) {
        let value = 1;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.game_state.world.location.is_outdoors() {
                let value = 48;
                self.sprite_slot_view_mut(k).set_object_priority(value);
            }
            if self.sprite_draw_absorbable_transient(k, true) {
                return;
            }
        }
        self.fairy_check_if_touchable(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_aux4() == 0 {
                    if self.sprite_check_damage_to_link(k) {
                        self.sprite_handle_absorption_by_player(k);
                    } else if (self.sprite_check_damage_from_link(k)
                        & PLAYER_DAMAGE_NONZERO_MASK_DRAW)
                        != 0
                    {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.sprite_show_message_unconditional(0xc9);
                        return;
                    }
                }
                if self.sprite_slot_view(k).state() == 0
                    && self
                        .game_state
                        .enhanced_features
                        .has(FEATURE_MISC_BUG_FIXES_DRAW)
                {
                    return;
                }
                if self.sprite_handle_dragging_by_ancilla(k) {
                    return;
                }
                self.faerie_handle_movement(k);
            }
            1 => {
                if self.multiselect_choice().value() == 0 {
                    let j = self.sprite_find_empty_bottle();
                    if j >= 0 {
                        let value = 6;
                        self.inventory_items_mut().set_bottle(j as usize, value);
                        self.hud_refresh_icon();
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_state(value);
                        return;
                    }
                    self.sprite_show_message_unconditional(0xca);
                }
                let value = 48;
                self.sprite_slot_view_mut(k).set_delay_aux4(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_D9_GreenRupee(int k) {  // 86d04a
    pub(super) fn sprite_d9_green_rupee(&mut self, k: usize) {
        self.sprite_draw_ripple_if_in_water(k);
        if self.sprite_draw_absorbable_transient(k, true) {
            return;
        }
        self.sprite_absorbable_main(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Absorbable_Main(int k) {  // 86d051
    pub(super) fn sprite_absorbable_main(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_z(k);
        self.sprite_move_xy(k);
        if self.sprite_main_cpu_boundary
            == Some(SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot: k as u8 })
        {
            assert_eq!(self.sprite_slot_view(k).delay_aux3(), 0);
            assert!(self.game_state.world.location.is_outdoors());
            assert_eq!(self.game_state.dungeon.room_load.header_collision(), 0);
            assert_eq!(self.sprite_slot_view(k).flags2() & 0x20, 0);
            self.sprite_slot_view_mut(k).set_wall_collision(0);
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
            assert_ne!(self.sprite_slot_view(k).x_velocity(), 0);
            return;
        }
        if self.sprite_slot_view(k).delay_aux3() == 0 {
            let _ = self.sprite_check_tile_collision2(k);
            self.sprite_bounce_off_wall(k);
        }
        self.sprite_absorbable_after_collision(k);
    }

    pub(super) fn sprite_absorbable_after_horizontal_lookup(&mut self, k: usize) {
        self.sprite_check_for_tile_in_direction_horizontal(
            k,
            if sign8(self.sprite_slot_view(k).x_velocity()) {
                2
            } else {
                3
            },
        );
        self.sprite_check_tile_collision_after_directions(k);
        self.sprite_bounce_off_wall(k);
        self.sprite_absorbable_after_collision(k);
    }

    fn sprite_absorbable_after_collision(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = (self.sprite_slot_view(k).x_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = (self.sprite_slot_view(k).y_velocity() as i8 >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let t = (0u8).wrapping_sub(self.sprite_slot_view(k).z_velocity()) >> 1;
            if t < 9 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                let value = t;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                if self.sprite_slot_view(k).draw_i() == 8 || self.sprite_slot_view(k).draw_i() == 9
                {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    let j = self.sprite_spawn_small_splash(k);
                    if j >= 0 && (self.sprite_slot_view(k).flags3() & 0x20) != 0 {
                        let j = j as usize;
                        // The ROM subtracts with SBC and no SEC (the C port's
                        // "wtf carry propagation"): the carry is whatever
                        // Sprite_CalculateSfxPan's final compare left inside
                        // SpriteSfx_QueueSfx2WithPan — clear for a centered
                        // pan, set otherwise — and the y subtraction chains
                        // from the x borrow (route host 450016: the splash
                        // lands at x-5/y-4 where the C port placed x-4/y-4).
                        let carry_in = self.sprite_calculate_sfx_pan(k) != 0;
                        let (x, carry) = crate::types::sbc_u16(self.sprite_get_x(j), 4, carry_in);
                        self.sprite_set_x(j, x);
                        let (y, _) = crate::types::sbc_u16(self.sprite_get_y(j), 4, carry);
                        self.sprite_set_y(j, y);
                    }
                } else if self.sprite_slot_view(k).sprite_type() >= 0xe4
                    && self.game_state.world.location.is_indoors()
                {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 5);
                }
            }
        }
        if self.sprite_handle_dragging_by_ancilla(k) {
            return;
        }
        self.sprite_check_absorption_by_player(k);
    }

    // -----------------------------------------------------------------------
    // void Gibo_Draw(int k) {  // 9dcf5e
    //   Optional single-large nucleus plus four segment tiles.
    // }
    pub(super) fn gibo_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() == 0 {
            let bak0 = self.sprite_slot_view(k).flags2();
            let value = 1;
            self.sprite_slot_view_mut(k).set_flags2(value);
            let bak1 = self.sprite_slot_view(k).oam_flags();
            let value = GIBO_OAM_FLAGS[((self.sprite_slot_view(k).anim_clock() >> 2) & 3) as usize]
                | GIBO_ALT_OAM_FLAGS[((self.sprite_slot_view(k).delay_aux1() >> 2) & 1) as usize];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            self.sprite_draw_single_large(k);
            let value = bak1;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            let value = bak0;
            self.sprite_slot_view_mut(k).set_flags2(value);
        }
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(8));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(2));
        let base = (usize::from(self.sprite_slot_view(k).subtype2())
            + usize::from(self.sprite_slot_view(k).direction()))
            * 4;
        self.sprite_draw_multiple(k, &GIBO_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void LaserEye_Draw(int k) {  // 9ea708
    //   Three tiles selected by direction and active beam blink state.
    // }
    pub(super) fn laser_eye_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            let value = u8::from(self.sprite_slot_view(k).delay_aux4() == 0);
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        let value = 0x30;
        self.sprite_slot_view_mut(k).set_object_priority(value);
        let base = (usize::from(self.sprite_slot_view(k).graphics())
            + usize::from(self.sprite_slot_view(k).direction()) * 2)
            * 3;
        self.sprite_draw_multiple(k, &LASER_EYE_DRAW_FRAMES[base..base + 3], None);
    }

    // -----------------------------------------------------------------------
    // void Gibdo_Draw(int k) {  // 9ebb20
    //   Sprite_DrawMultiple(k, &kGibdo_Dmd[sprite_graphics[k] * 2], 2, &info);
    //   if (!sprite_pause[k]) SpriteDraw_Shadow(k, &info);
    // }
    pub(super) fn gibdo_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &GIBDO_DRAW_FRAMES[base..base + 2], Some(&mut info));
        if self.sprite_slot_view(k).pause() == 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 10);
        }
    }

    // -----------------------------------------------------------------------
    // void FireBat_Draw(int k) {  // 9d8ca9
    //   Two large tiles sharing one animation char with per-frame flips.
    // }
    pub(super) fn fire_bat_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = self.sprite_slot_view(k).graphics() as usize;
        for i in (0..2).rev() {
            self.set_oam_helper0_at_for_draw(
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
    pub(super) fn fire_phlegm_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &FIRE_PHLEGM_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void FlyingTile_Draw(int k) {  // 9ebcca
    //   Four-tile tile plus normal sprite shadow.
    // }
    pub(super) fn flying_tile_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &FLYING_TILE_DRAW_FRAMES[base..base + 4], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Bully_Draw(int k) {  // 9eed9e
    //   Sprite_DrawMultiplePlayerDeferred(k, &kBully_Dmd[sprite_D[k] * 4 +
    //                                     sprite_graphics[k] * 2], 2, &info);
    //   SpriteDraw_Shadow(k, &info);
    // }
    pub(super) fn bully_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &BULLY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B9_BullyAndPinkBall(int k) {  // 9eeb33
    pub(super) fn sprite_b9_bully_and_pink_ball(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_pink_ball(k),
            1 => self.pink_ball_distress(k),
            2 => self.sprite_bully(k),
            // C Sprite_B9_BullyAndPinkBall asserts for all other subtypes.
            _ => panic!("Sprite_B9_BullyAndPinkBall invalid subtype"),
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B6_Kiki(int k) {  // 9ee2ef
    pub(super) fn sprite_b6_kiki(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.kiki_lying_inwait(k),
            1 => self.kiki_offer_entrance_service(k),
            2 => self.kiki_offer_initial_service(k),
            3 => self.kiki_flee(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B7_BlindMaiden(int k) {  // 9ee8b6
    pub(super) fn sprite_b7_blind_maiden(&mut self, k: usize) {
        if !self.sprite_b7_blind_maiden_before_follower_graphics(k) {
            return;
        }
        self.load_follower_graphics();
        self.sprite_b7_blind_maiden_after_follower_graphics(k);
    }

    pub(super) fn sprite_b7_blind_maiden_before_follower_graphics(&mut self, k: usize) -> bool {
        self.crystal_maiden_draw(k);
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.sprite_show_message_on_contact(k, 0x0122) & 0x100 != 0 {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            false
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_state_mut().set_indicator(6);
            true
        }
    }

    pub(super) fn sprite_b7_blind_maiden_after_follower_graphics(&mut self, k: usize) {
        self.sprite_become_follower(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B8_DialogueTester(int k) {  // 9eeae7
    pub(super) fn sprite_b8_dialogue_tester(&mut self, _k: usize) {
        // C Sprite_B8_DialogueTester is an assert(0) sprite slot.
        assert!(false);
    }

    // -----------------------------------------------------------------------
    // void Sprite_PinkBall(int k) {  // 9eeb40
    pub(super) fn sprite_pink_ball(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.pink_ball_handle_message(k);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x80)
            | self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_move_xyz(k);
        let t = self.sprite_check_tile_collision(k);
        if t != 0 {
            if (t & 3) == 0 {
                let value = self.sprite_slot_view(k).y_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if self.sprite_slot_view(k).e() != 0 {
                    self.ball_guy_play_bounce_noise(k);
                }
            } else {
                let value = self.sprite_slot_view(k).x_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                if self.sprite_slot_view(k).e() != 0 {
                    self.ball_guy_play_bounce_noise(k);
                }
            }
        }

        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = self.sprite_slot_view(k).z_velocity().wrapping_neg() >> 2;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            if (self.sprite_slot_view(k).z_velocity() & 0xfc) != 0 {
                self.ball_guy_play_bounce_noise(k);
            }
            self.pink_ball_handle_deceleration(k);
        }
        if self.sprite_slot_view(k).e() == 0 {
            if self.sprite_slot_view(k).head_direction() == 0 {
                self.pink_ball_distress(k);
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x3f) == 0 {
                    let x = (self.game_state.player.follower_link.x() & 0xff00)
                        | u16::from(self.get_random_number());
                    let y = (self.game_state.player.follower_link.y() & 0xff00)
                        | u16::from(self.get_random_number());
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 8);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if pt.y != 0 {
                        self.sprite_slot_view_mut(k).or_oam_flags(0x40);
                        let value = (self.sprite_slot_view(k).x_velocity() >> 1) & 0x40;
                        self.sprite_slot_view_mut(k).xor_oam_flags(value);
                    }
                }
                let value = self.sprite_slot_view(k).b();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = self.sprite_slot_view(k).a();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                self.pink_ball_distress(k);
                if ((k as u8) ^ self.game_state.frame.frame_counter) != 0 {
                    let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
        } else if (self.sprite_slot_view(k).x_velocity() | self.sprite_slot_view(k).y_velocity())
            == 0
        {
            let value = 0;
            self.sprite_slot_view_mut(k).set_e(value);
        } else {
            let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 2) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let value = (((k as u8) ^ self.game_state.frame.frame_counter) << 2) & 0x80;
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Bully(int k) {  // 9eec7c
    pub(super) fn sprite_bully(&mut self, k: usize) {
        self.bully_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.bully_handle_message(k);
        self.sprite_move_xyz(k);
        let t = self.sprite_check_tile_collision(k);
        if t != 0 {
            if (t & 3) == 0 {
                let value = self.sprite_slot_view(k).y_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            } else {
                let value = self.sprite_slot_view(k).x_velocity().wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let j = usize::from(self.sprite_slot_view(k).head_direction());
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x1f) == 0 {
                    let pt = self.sprite_project_speed_towards_location(
                        k,
                        self.sprite_get_x(j),
                        self.sprite_get_y(j),
                        14,
                    );
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if pt.x != 0 {
                        let value = self.sprite_slot_view(k).x_velocity() >> 7;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
                if self.sprite_slot_view(j).z() == 0
                    && self
                        .sprite_slot_view(k)
                        .x_low()
                        .wrapping_sub(self.sprite_slot_view(j).x_low())
                        .wrapping_add(8)
                        < 16
                    && self
                        .sprite_slot_view(k)
                        .y_low()
                        .wrapping_sub(self.sprite_slot_view(j).y_low())
                        .wrapping_add(8)
                        < 16
                {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.ball_guy_play_bounce_noise(k);
                }
            }
            1 => {
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let j = usize::from(self.sprite_slot_view(k).head_direction());
                let value = self.sprite_slot_view(k).x_velocity() << 1;
                self.sprite_slot_view_mut(j).set_x_velocity(value);
                let value = self.sprite_slot_view(k).y_velocity() << 1;
                self.sprite_slot_view_mut(j).set_y_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = self.get_random_number() & 31;
                self.sprite_slot_view_mut(j).set_z_velocity(value);
                let value = 96;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let value = 1;
                self.sprite_slot_view_mut(j).set_e(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void BuzzBlob_Draw(int k) {  // 86d953
    pub(super) fn buzz_blob_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = SpritePrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let ch = BUZZ_BLOB_DRAW_CHARS[g * 3 + i];
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(BUZZ_BLOB_DRAW_X_OFFSETS[i]),
                info_y.wrapping_add(BUZZ_BLOB_DRAW_Y_OFFSETS[i] as u16),
                ch,
                BUZZ_BLOB_DRAW_FL[g * 3 + i] | info_flags,
                BUZZ_BLOB_DRAW_EXT[i],
            );
            if ch == 0 {
                self.oam_state_mut().hide_entry(oam);
            }
            oam += 4;
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Babusu_Draw(int k) {  // 8dbd20
    pub(super) fn babusu_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics();
        if g != 0xff {
            let base = usize::from(g) * 2;
            self.sprite_draw_multiple(k, &BABUSU_DRAW_FRAMES[base..base + 2], None);
        } else {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        }
    }

    // -----------------------------------------------------------------------
    // void Lady_Draw(int k) {  // 9af92c
    pub(super) fn lady_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &LADY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void YoungSnitchLady_Draw(int k) {  // 85e37f
    pub(super) fn young_snitch_lady_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &YOUNG_SNITCH_LADY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_YoungSnitchLady(int k) {  // 85e2f2
    pub(super) fn sprite_young_snitch_lady(&mut self, k: usize) {
        self.sprite_old_snitch_lady(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_OldSnitchLady(int k) {  // 85e6aa
    pub(super) fn sprite_old_snitch_lady(&mut self, k: usize) {
        if self.sprite_slot_view(k).sprite_type() == 0x34 {
            if self.sprite_slot_view(k).ai_state() < 2 {
                self.young_snitch_lady_draw(k);
            }
        } else {
            if self.sprite_slot_view(k).subtype() != 0 {
                self.sprite_chicken_lady(k);
                return;
            }
            if self.sprite_slot_view(k).ai_state() < 3 {
                self.lady_draw(k);
            }
        }

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).ai_state() < 3 {
            if self.game_state.world.location.is_indoors() {
                self.sprite_track_body_to_head(k);
                let value = self.sprite_direction_to_face_link(k, None) ^ 3;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                self.sprite_show_solicited_message(k, 0x00ad);
                return;
            }
            if self.sprite_slot_view(k).ai_state() == 0
                && self.sprite_check_damage_to_link_same_layer(k)
            {
                let value = self.sprite_direction_to_face_link(k, None) ^ 3;
                self.sprite_slot_view_mut(k).set_direction(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else if self.sprite_track_body_to_head(k) {
                self.sprite_move_xy(k);
            } else {
                let value = 1;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
        }

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let home_x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let idx = usize::from(self.sprite_slot_view(k).c() & 1);
                    let t = home_x.wrapping_add_signed(i16::from(SPRITE_OLD_SNITCH_LADY_XD[idx]));
                    if t == self.sprite_get_x(k) {
                        let j = self.sprite_slot_view(k).direction() ^ 1;
                        let value = j;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        let value = SPRITE_OLD_SNITCH_LADY_LOCAL_X_VELOCITIES[usize::from(j)] as u8;
                        self.sprite_slot_view_mut(k).set_x_velocity(value);
                        let value = SPRITE_OLD_SNITCH_LADY_LOCAL_Y_VELOCITIES[usize::from(j)] as u8;
                        self.sprite_slot_view_mut(k).set_y_velocity(value);
                        self.sprite_slot_view_mut(k).xor_c(1);
                    }
                }
                let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 4) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let bak0 = self.sprite_slot_view(k).flags4();
                let value = 3;
                self.sprite_slot_view_mut(k).set_flags4(value);
                let j = self.sprite_show_message_on_contact(k, 0x002f);
                let value = bak0;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if (j & 0x0100) != 0 {
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    self.snitch_spawn_guard(k);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let idx = usize::from(
                    self.game_state
                        .sprites
                        .garnish_runtime
                        .active_overlord_index(),
                );
                let overlord = self.overlord_slot_view(idx);
                let ovx = overlord.x();
                let ovy = overlord.y();
                if ovy >= self.sprite_get_y(k) {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_flags4(value);
                    let base_y = self.game_state.world.scroll.overworld_offset_base_y();
                    let mask_y = self.game_state.world.scroll.overworld_offset_mask_y();
                    let base_x = self.game_state.world.scroll.overworld_offset_base_x();
                    let mask_x = self.game_state.world.scroll.overworld_offset_mask_x();
                    let pos = ((ovy.wrapping_sub(base_y) & mask_y) * 8)
                        .wrapping_add((ovx >> 3).wrapping_sub(base_x) & mask_x);
                    self.overworld_draw_wooden_door(pos, false);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.follower_link_state_mut().immobilize();
                    let pt = self.sprite_project_speed_towards_location(k, ovx, ovy, 64);
                    let value = pt.x;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = pt.y;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = (((k as u8) ^ self.game_state.frame.frame_counter) >> 3) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let idx = usize::from(
                        self.game_state
                            .sprites
                            .garnish_runtime
                            .active_overlord_index(),
                    );
                    let overlord = self.overlord_slot_view(idx);
                    let ovx = overlord.x();
                    let ovy = overlord.y();
                    self.sprite_set_x(k, ovx);
                    self.sprite_set_y(k, ovy);
                    let base_y = self.game_state.world.scroll.overworld_offset_base_y();
                    let mask_y = self.game_state.world.scroll.overworld_offset_mask_y();
                    let base_x = self.game_state.world.scroll.overworld_offset_base_x();
                    let mask_x = self.game_state.world.scroll.overworld_offset_mask_x();
                    let pos = ((ovy.wrapping_sub(base_y) & mask_y) * 8)
                        .wrapping_add((ovx >> 3).wrapping_sub(base_x) & mask_x);
                    self.overworld_draw_wooden_door(pos, true);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                self.sprite_move_xy(k);
            }
            3 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.follower_link_state_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Cukeman_Draw(int k) {  // 9afb0e
    pub(super) fn cukeman_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &CUKEMAN_DRAW_FRAMES[base..base + 3], None);
    }

    // -----------------------------------------------------------------------
    // void Hokbok_Draw(int k) {  // 9dc77d
    pub(super) fn hokbok_draw(&mut self, k: usize) {
        let Some((info_x, mut y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = SpritePrepOamCoordsRet {
            x: info_x,
            y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize() + 12;
        let d = self.sprite_slot_view(k).b();
        for i in (0..=self.sprite_slot_view(k).a()).rev() {
            let ch =
                (if i == 0 { 0xa2u8 } else { 0xa0u8 }).wrapping_sub(if d < 7 { 0x20 } else { 0 });
            self.set_oam_helper0_at_for_draw(oam, info_x, y, ch, info_flags, 2);
            y = y.wrapping_sub(u16::from(d));
            oam = oam.wrapping_sub(4);
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SnapDragon_Draw(int k) {  // 869e02
    pub(super) fn snap_dragon_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &SNAP_DRAGON_DRAW_FRAMES[base..base + 4], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Lynel_Draw(int k) {  // 9d8880
    pub(super) fn lynel_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple(k, &LYNEL_DRAW_FRAMES[base..base + 3], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Goriya_Draw(int k) {  // 9df589
    pub(super) fn goriya_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 && self.sprite_slot_view(k).direction() != 3 {
            let d = usize::from(self.sprite_slot_view(k).direction());
            self.sprite_draw_multiple(k, &GORIYA_DRAW_FRAMES_2[d..d + 1], None);
        }

        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        let g = usize::from(self.sprite_slot_view(k).graphics());
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(
            k,
            &GORIYA_DRAW_FRAMES[GORIYA_DRAW_OFFSETS[g]..GORIYA_DRAW_OFFSETS[g + 1]],
            Some(&mut info),
        );
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        self.sprite_slot_view_mut(k).add_flags2(1);
    }

    // -----------------------------------------------------------------------
    // void Kyameron_Draw(int k) {  // 9ea158
    pub(super) fn kyameron_draw(&mut self, k: usize) {
        let j = self.sprite_slot_view(k).graphics();
        if j < 12 {
            let bak = self.sprite_slot_view(k).oam_flags();
            let value =
                (self.sprite_slot_view(k).oam_flags() & 0x3f) | KYAMERON_DRAW_FLAGS[usize::from(j)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            self.sprite_draw_single_large(k);
            let value = bak;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let base = (usize::from(j) - 12) * 4;
            self.sprite_draw_multiple(k, &KYAMERON_DRAW_FRAMES[base..base + 4], None);
        }
    }

    // -----------------------------------------------------------------------
    // void Pirogusu_Draw(int k) {  // 9ea93b
    pub(super) fn pirogusu_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).a());
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | PIROGUSU_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = PIROGUSU_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        if j < 4 {
            let x = self
                .game_state
                .sprites
                .workspace
                .current_sprite_x()
                .wrapping_add(4);
            let y = self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(4);
            self.sprite_workspace_mut().set_current_sprite_x(x);
            self.sprite_workspace_mut().set_current_sprite_y(y);
            self.sprite_draw_single_small(k);
        } else {
            self.sprite_draw_single_large(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Hobo_Draw(int k) {  // 84ea60
    pub(super) fn hobo_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple_player_deferred(k, &HOBO_DRAW_FRAMES[base..base + 4], None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_2B_Hobo(int k) {  // 86bdc1
    pub(super) fn sprite_2_b_hobo(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_hobo_bum(k),
            1 => self.sprite_hobo_bubble(k),
            2 => self.sprite_hobo_fire(k),
            3 => self.sprite_hobo_smoke(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Bum(int k) {  // 86bdd0
    pub(super) fn sprite_hobo_bum(&mut self, k: usize) {
        self.hobo_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = 3;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 7;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if self.sprite_check_damage_to_link_same_layer(k)
                    && (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let j = usize::from(self.sprite_slot_view(k).e());
                    let value = 4;
                    self.sprite_slot_view_mut(j).set_delay_main(value);
                    self.follower_link_state_mut().immobilize();
                }
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    let value = 160;
                    self.sprite_slot_view_mut(k).set_delay_aux2(value);
                    let j = self.hobo_spawn_bubble(k);
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_e(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = usize::from(self.sprite_slot_view(k).a());
                    if j != 7 {
                        let value = SPRITE_HOBO_BUM_LOCAL_GRAPHICS[j];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                        let value = SPRITE_HOBO_BUM_DELAY[j];
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_slot_view_mut(k).add_a(1);
                    } else {
                        self.sprite_show_message_unconditional(0xd7);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                }
            }
            2 => {
                let value = 3;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let screen = usize::from(self.game_state.world.location.overworld_screen_index());
                self.set_overworld_event_bits(screen, 0x20);
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x16,
                        0,
                        ItemReceiptCaller::SpriteMainDirect {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::HoboBottle,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_hobo_bottle_item_receipt(k);
            }
            3 => {
                self.follower_link_state_mut().clear_immobilized();
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 160;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.hobo_spawn_bubble(k);
                }
            }
            _ => {}
        }
    }

    /// Source suffix after Sprite_Hobo_Bum's `Link_ReceiveItem(0x16, 0)`
    /// (ROM `$86bdd0` case 2): only the progress-indicator bit remains.
    pub(super) fn complete_hobo_bottle_item_receipt(&mut self, _k: usize) {
        self.save_progress_mut().or_progress_indicator_3(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Bubble(int k) {  // 86beb4
    pub(super) fn sprite_hobo_bubble(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = ((self.game_state.frame.frame_counter >> 4) & 1) + 2;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            self.sprite_slot_view_mut(k).add_graphics(1);
            self.sprite_move_z(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        }
        if self.sprite_slot_view(k).delay_main() < 4 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Fire(int k) {  // 86bf15
    pub(super) fn sprite_hobo_fire(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = (self.game_state.frame.frame_counter >> 3) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_slot_view_mut(k).and_oam_flags(!0x40);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.hobo_spawn_smoke(k);
            let value = 47;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hobo_Smoke(int k) {  // 86bf81
    pub(super) fn sprite_hobo_smoke(&mut self, k: usize) {
        let value = 6;

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        let flags = SPRITE_HOBO_SMOKE_OAM_FLAGS
            [usize::from((self.game_state.frame.frame_counter >> 4) & 3)];
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | flags;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_RunningMan(int k) {  // 85e8b2
    pub(super) fn sprite_running_man(&mut self, k: usize) {
        self.running_man_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        self.sprite_behave_as_barrier(k);
        let value = 255;
        self.sprite_slot_view_mut(k).set_subtype(value);
        self.sprite_check_tile_collision(k);
        let bak0 = self.sprite_slot_view(k).flags4();
        let value = 7;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            let value = self.sprite_slot_view(k).ai_state();
            self.sprite_slot_view_mut(k).set_c(value);
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
        let value = bak0;
        self.sprite_slot_view_mut(k).set_flags4(value);

        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_track_body_to_head(k);
                let j = self.sprite_direction_to_face_link(k, None);
                let value = j ^ 3;
                self.sprite_slot_view_mut(k).set_head_direction(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.link_cancel_dash();
                    let value = j ^ 3;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = j | 2;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = (j & 1) + 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = RUNNING_MAN_RECOIL_X_VELOCITIES[usize::from(j & 1)] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            1 | 2 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let value = (self.game_state.frame.frame_counter >> 3) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    self.sprite_move_xy(k);
                } else {
                    self.running_boy_spawn_dust_garnish(k);
                    let value = (self.game_state.frame.frame_counter >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let j = usize::from(self.sprite_slot_view(k).head_direction());
                    let value = SPRITE_RUNNING_MAN_LOCAL_X_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_RUNNING_MAN_LOCAL_Y_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                    self.sprite_move_xy(k);
                    if self.sprite_slot_view(k).a() != 0 {
                        self.sprite_slot_view_mut(k).subtract_a(1);
                    } else if self.sprite_slot_view(k).ai_state() == 1 {
                        let value = 255;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    } else {
                        let j = usize::from(self.sprite_slot_view(k).b());
                        self.sprite_slot_view_mut(k).add_b(1);
                        let value = SPRITE_RUNNING_MAN_A[j];
                        self.sprite_slot_view_mut(k).set_a(value);
                        if sign8(SPRITE_RUNNING_MAN_DIRECTIONS[j] as u8) {
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_ai_state(value);
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_subtype2(value);
                        } else {
                            let value = SPRITE_RUNNING_MAN_DIRECTIONS[j] as u8;
                            self.sprite_slot_view_mut(k).set_head_direction(value);
                        }
                    }
                }
            }
            3 => {
                self.sprite_show_message_unconditional(0x00a6);
                if self.game_state.player.follower_link.handler_state()
                    >= SPRITE_RUNNING_MAN_PLAYER_STATE_RECOIL_WALL_LOCAL
                {
                    let value = self.game_state.player.follower_link.handler_state();
                    self.sprite_slot_view_mut(k).set_direction(value);
                }
                let value = self.sprite_slot_view(k).c();
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void RunningMan_Draw(int k) {  // 85ea4d
    pub(super) fn running_man_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics()) * 2)
            & 0xf;
        self.sprite_draw_multiple_player_deferred(
            k,
            &RUNNING_MAN_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Elder_Draw(int k) {  // 85f23a
    pub(super) fn elder_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &ELDER_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Shopkeeper_Draw(int k) {  // 85f91b
    pub(super) fn shopkeeper_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &SHOPKEEPER_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void FluteBoyFather_Draw(int k) {  // 8dc3e1
    pub(super) fn flute_boy_father_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &FLUTE_BOY_FATHER_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void FluteBoyOstrich_Draw(int k) {  // 9e9a4b
    pub(super) fn flute_boy_ostrich_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(
            k,
            &FLUTE_BOY_OSTRICH_DRAW_FRAMES[base..base + 4],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 18);
    }

    // -----------------------------------------------------------------------
    // void OldMountainMan_Draw(int k) {  // 9dff0e
    pub(super) fn old_mountain_man_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).subtype2() != 2 {
            let j = usize::from(self.sprite_slot_view(k).direction()) * 4
                + usize::from(self.sprite_slot_view(k).graphics()) * 2;
            self.set_sprite_dma_head_pointer(OLD_MOUNTAIN_MAN_DRAW_DMA[j]);
            self.set_sprite_dma_body_pointer(OLD_MOUNTAIN_MAN_DRAW_DMA[j + 1]);
            self.sprite_draw_multiple_player_deferred(
                k,
                &OLD_MOUNTAIN_MAN_DRAW_FRAMES_1[j..j + 2],
                None,
            );
        } else {
            self.sprite_draw_multiple_player_deferred(k, &OLD_MOUNTAIN_MAN_DRAW_FRAMES_0, None);
        }
    }

    // -----------------------------------------------------------------------
    // void InnKeeper_Draw(int k) {  // 85e3dc
    pub(super) fn inn_keeper_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple_player_deferred(k, &INN_KEEPER_DRAW_FRAMES, Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_InnKeeper(int k) {  // 85e3af
    pub(super) fn sprite_inn_keeper(&mut self, k: usize) {
        self.inn_keeper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(
            k,
            if self.game_state.inventory.items.flippers() != 0 {
                0x0183
            } else {
                0x0182
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_Witch(int k) {  // 85e3fb
    pub(super) fn sprite_witch(&mut self, k: usize) {
        self.witch_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let bak0 = self.sprite_slot_view(k).flags4();
        let value = 2;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        let value = bak0;
        self.sprite_slot_view_mut(k).set_flags4(value);
        if self.game_state.frame.frame_counter == 0 {
            let value = (self.get_random_number() & 1) + 2;
            self.sprite_slot_view_mut(k).set_a(value);
        }
        let shift = self.sprite_slot_view(k).a().wrapping_add(1);
        let value = (self.game_state.frame.frame_counter >> shift) & 7;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_check_if_link_is_busy() {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => match self.game_state.inventory.items.mushroom() {
                0 => {
                    if self
                        .game_state
                        .inventory
                        .save_progress
                        .dungeon_info_word(0x109)
                        & 0x80
                        != 0
                    {
                        self.sprite_show_solicited_message(k, 0x004b);
                    } else {
                        self.sprite_show_solicited_message(k, 0x004a);
                    }
                }
                1 => {
                    if self.game_state.player.follower_link.joypad1h_last() & 0x40 == 0 {
                        self.sprite_show_solicited_message(k, 0x004c);
                    } else if self.sprite_check_damage_to_link_same_layer(k)
                        && self.game_state.inventory.save_progress.hud_current_item()
                            == HUD_ITEM_MUSHROOM_DRAW
                    {
                        self.witch_accept_shroom(k);
                    }
                }
                _ => {
                    self.sprite_show_solicited_message(k, 0x004a);
                }
            },
            1 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.follower_link_state_mut().set_item_receipt_method(0);
                self.link_receive_item(0x18, 0);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Witch_AcceptShroom(int k) {  // 85e4cf
    pub(super) fn witch_accept_shroom(&mut self, k: usize) {
        self.inventory_items_mut().set_mushroom(0);
        let dung_info = self
            .game_state
            .inventory
            .save_progress
            .dungeon_info_word(0x109)
            | 0x80;
        self.save_progress_mut()
            .set_dungeon_info_word(0x109, dung_info);
        self.set_sound_effect_1(0);
        self.hud_refresh_icon();
        self.sprite_show_message_unconditional(0x004b);
        self.sprite_sfx_queue_sfx1_with_pan(k, 0x0d);
        self.clear_flag_overworld_area_changed();
    }

    // -----------------------------------------------------------------------
    // void Witch_Draw(int k) {  // 85e55d
    pub(super) fn witch_draw(&mut self, k: usize) {
        let Some((_, _, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        self.oam_allocate_defer_to_player(k);
        let oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        let scratch = &self.game_state.sprites.draw_hitbox_work;
        let x = scratch.x_low();
        let y = scratch.y_low();

        for i in 0..2 {
            let (dx, dy, chr, _fl) = WITCH_DRAW_DATA_A[g + i];
            self.set_oam_plain_at_for_draw(
                oam + i * 4,
                x.wrapping_add(dx as u8),
                y.wrapping_add(dy as u8),
                chr,
                flags,
                0,
            );
        }
        for (i, &(dx, dy, chr, fl)) in WITCH_DRAW_DATA_B.iter().enumerate() {
            self.set_oam_plain_at_for_draw(
                oam + (i + 2) * 4,
                x.wrapping_add(dx as u8),
                y.wrapping_add(dy as u8),
                chr,
                flags ^ fl,
                2,
            );
        }
        let i = usize::from((g as u16).wrapping_sub(6) < 6);
        let (dx, dy, chr, _fl) = WITCH_DRAW_DATA_C[i];
        self.set_oam_plain_at_for_draw(
            oam + 5 * 4,
            x.wrapping_add(dx as u8),
            y.wrapping_add(dy as u8),
            chr,
            flags,
            2,
        );
        self.sprite_correct_oam_entries_for_draw(k, 5, 0xff);
    }

    // -----------------------------------------------------------------------
    // void ElderWife_Draw(int k) {  // 85f505
    pub(super) fn elder_wife_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(k, &ELDER_WIFE_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_78_MrsSahasrahla(int k) {  // 86c071
    pub(super) fn sprite_78_mrs_sahasrahla(&mut self, k: usize) {
        self.elder_wife_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.inventory.items.sword_type() < 2 {
                    if self.sprite_show_solicited_message(k, 0x002b) & 0x100 != 0 {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                } else {
                    self.sprite_show_solicited_message(k, 0x002e);
                }
                let value = self.game_state.frame.frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            1 => {
                self.sprite_show_message_unconditional(0x002c);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.multiselect_choice().value() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_show_message_unconditional(0x002d);
                } else {
                    self.sprite_show_message_unconditional(0x002c);
                }
            }
            3 => {
                self.sprite_show_solicited_message(k, 0x002d);
                let value = self.game_state.frame.frame_counter >> 4 & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MiddleAgedMan_Draw(int k) {  // 86bdac
    pub(super) fn middle_aged_man_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple_player_deferred(k, &MIDDLE_AGED_MAN_DRAW_FRAMES, Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void BlindHideoutGuy_Draw(int k) {  // 8dc481
    pub(super) fn blind_hideout_guy_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &BLIND_HIDEOUT_GUY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_HumanMulti_1(int k) {  // 8dc2d9
    pub(super) fn sprite_human_multi_1(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_flute_dad(k),
            1 => self.sprite_thief_hideout_guy(k),
            2 => self.sprite_blinds_hut_guy(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BlindsHutGuy(int k) {  // 8dc2e6
    pub(super) fn sprite_blinds_hut_guy(&mut self, k: usize) {
        self.blind_hideout_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_track_body_to_head(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        let j = self.sprite_show_solicited_message(k, 0x0172);
        if j & 0x100 != 0 {
            let dir = j as u8;
            let value = dir;
            self.sprite_slot_view_mut(k).set_direction(value);
            let value = dir;
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_ThiefHideoutGuy(int k) {  // 8dc308
    pub(super) fn sprite_thief_hideout_guy(&mut self, k: usize) {
        if self.game_state.frame.frame_counter & 3 == 0 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let dir = self.sprite_direction_to_face_link(k, None);
            let value = if dir == 3 { 2 } else { dir };
            self.sprite_slot_view_mut(k).set_head_direction(value);
        }
        let value = 15;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.oam_allocate_defer_to_player(k);
        self.thief_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(k, 0x0171);
        let value = 2;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_FluteDad(int k) {  // 8dc343
    pub(super) fn sprite_flute_dad(&mut self, k: usize) {
        self.flute_boy_father_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        let value = if self.game_state.frame.frame_counter < 48 {
            2
        } else {
            (self.game_state.frame.frame_counter >> 7) & 1
        };
        self.sprite_slot_view_mut(k).set_graphics(value);

        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_show_solicited_message(k, 0x00a3);
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
        } else if self.game_state.inventory.items.flute() < 2 {
            self.sprite_show_solicited_message(k, 0x00a1);
        } else if self.sprite_show_solicited_message(k, 0x00a4) & 0x100 == 0
            && self.game_state.inventory.save_progress.hud_current_item() == HUD_ITEM_FLUTE_DRAW
            && self.game_state.player.follower_link.joypad1h_last() & 0x40 != 0
            && self.sprite_check_damage_to_link_same_layer(k)
        {
            self.sprite_show_message_unconditional(0x00a2);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 2;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void SweepingLady_Draw(int k) {  // 8dc4eb
    pub(super) fn sweeping_lady_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &SWEEPING_LADY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_SweepingLady(int k) {  // 8dc4ad
    pub(super) fn sprite_sweeping_lady(&mut self, k: usize) {
        self.sweeping_lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_show_solicited_message(k, 0x00a5);
        self.sprite_behave_as_barrier(k);
        let value = self.game_state.frame.frame_counter >> 4 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_FortuneTeller(int k) {  // 8dc762
    pub(super) fn sprite_fortune_teller(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => {
                self.fortune_teller_draw(k);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                self.fortune_teller_light_or_dark_world(
                    k,
                    ((self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1) != 0,
                );
            }
            1 => {
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.link_cancel_dash();
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_LightOrDarkWorld(int k, bool dark_world) {
    pub(super) fn fortune_teller_light_or_dark_world(&mut self, k: usize, dark_world: bool) {
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                let j = self.get_random_number() & 3;
                let value = j << 1;
                self.sprite_slot_view_mut(k).set_a(value);
                if self.game_state.inventory.player_resources.rupees_goal()
                    < u16::from(FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES[usize::from(j)])
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                self.sprite_show_solicited_message(k, 0x00f2);
            }
            2 => {
                if self.sprite_show_solicited_message(k, 0x00f3) & 0x100 != 0 {
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.follower_link_state_mut().immobilize();
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            3 => {
                if self.multiselect_choice().value() == 0 {
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let value = (self.game_state.frame.frame_counter >> 4) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                } else {
                    self.sprite_show_message_unconditional(0x00f5);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.follower_link_state_mut().clear_immobilized();
                }
            }
            4 => {
                self.fortune_teller_perform_pseudo_science(k);
            }
            5 => {
                if !dark_world {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
                let j = FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES
                    [usize::from(self.sprite_slot_view(k).a() >> 1)];
                self.dialogue_number_mut()
                    .set_packed_digits((j / 10) | ((j % 10) << 4), 0);
                self.sprite_show_message_unconditional(0x00f4);
                self.sprite_slot_view_mut(k).add_ai_state(1);
            }
            6 => {
                let price = u16::from(
                    FORTUNE_TELLER_LIGHT_OR_DARK_WORLD_PRICES
                        [usize::from(self.sprite_slot_view(k).a() >> 1)],
                );
                let rupees = self
                    .game_state
                    .inventory
                    .player_resources
                    .rupees_goal()
                    .wrapping_sub(price);
                self.player_resources_mut().set_rupees_goal(rupees);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                self.player_resources_mut().set_heart_filler(160);
                self.follower_link_state_mut().clear_immobilized();
            }
            7 => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_PerformPseudoScience(int k) {  // 8dc849
    pub(super) fn fortune_teller_perform_pseudo_science(&mut self, k: usize) {
        let value = 0;

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_slot_view_mut(k).add_ai_state(1);

        let mut slots = [0u8; 2];
        let mut n = 0usize;
        if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            >= 3
        {
            let inventory = &self.game_state.inventory.items;
            let resources = &self.game_state.inventory.player_resources;
            if inventory.book() == 0 {
                slots[n] = 2;
                n += 1;
            }
            if n < 2 && (resources.pendant_flags() & 2) == 0 {
                slots[n] = 1;
                n += 1;
            }
            if n < 2 && inventory.mushroom() < 2 {
                slots[n] = 3;
                n += 1;
            }
            if n < 2 && inventory.flippers() == 0 {
                slots[n] = 4;
                n += 1;
            }
            if n < 2 && inventory.moon_pearl() == 0 {
                slots[n] = 5;
                n += 1;
            }
            if n < 2 && self.game_state.inventory.save_progress.progress_indicator() < 3 {
                slots[n] = 6;
                n += 1;
            }
            if n < 2 && resources.magic_consumption_level() == 0 {
                slots[n] = 7;
                n += 1;
            }
            if n < 2 && inventory.bombos() == 0 {
                slots[n] = 8;
                n += 1;
            }
            if n < 2
                && (self
                    .game_state
                    .inventory
                    .save_progress
                    .progress_indicator_3()
                    & 0x10)
                    == 0
            {
                slots[n] = 9;
                n += 1;
            }
            if n < 2
                && (self
                    .game_state
                    .inventory
                    .save_progress
                    .progress_indicator_3()
                    & 0x20)
                    == 0
            {
                slots[n] = 10;
                n += 1;
            }
            if n < 2 && inventory.cape() == 0 {
                slots[n] = 11;
                n += 1;
            }
            if n < 2 && (self.game_state.world.overworld.event_info.event_info(0x5b) & 2) == 0 {
                slots[n] = 12;
                n += 1;
            }
            if n < 2 && inventory.sword_type() < 4 {
                slots[n] = 13;
                n += 1;
            }
            if n < 2 {
                slots[n] = 14;
                n += 1;
            }
            if n < 2 {
                slots[n] = 15;
            }
        }

        self.save_progress_mut().xor_progress_flags(0x40);
        let j = usize::from((self.game_state.inventory.save_progress.progress_flags() & 0x40) != 0);
        self.sprite_show_message_unconditional(u16::from(
            FORTUNE_TELLER_PERFORM_PSEUDO_SCIENCE_READINGS[usize::from(slots[j])],
        ));
    }

    // -----------------------------------------------------------------------
    // void FortuneTeller_Draw(int k) {  // 8dcb01
    pub(super) fn fortune_teller_draw(&mut self, k: usize) {
        let j = usize::from((self.game_state.inventory.save_progress.dark_world_state() >> 6) & 1)
            * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple(k, &FORTUNE_TELLER_DRAW_FRAMES[j * 3..j * 3 + 3], None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_MazeGameLady(int k) {  // 8dcb5c
    pub(super) fn sprite_maze_game_lady(&mut self, k: usize) {
        self.lady_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).x_low() < self.game_state.player.follower_link.x() as u8
                {
                    let j = self.sprite_show_message_on_contact(k, 0x00cc);
                    if j & 0x100 != 0 {
                        let dir = j as u8;
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_direction(value);
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        self.maze_game_timer_mut().clear_elapsed();
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_a(value);
                        self.clear_flag_overworld_area_changed();
                    }
                } else {
                    self.sprite_show_message_on_contact(k, 0x00d0);
                }
            }
            1 => {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x07);
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                self.sprite_slot_view_mut(k).add_a(1);
                if self.sprite_slot_view(k).a() == 63 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_a(value);
                    let t = self.maze_game_timer_mut().increment_elapsed_low();
                    if t == 0 {
                        self.maze_game_timer_mut().increment_elapsed_high();
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_MazeGameGuy(int k) {  // 8dcbf2
    pub(super) fn sprite_maze_game_guy(&mut self, k: usize) {
        self.maze_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_track_body_to_head(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_head_direction(value);
        self.sprite_behave_as_barrier(k);
        let value = self.game_state.frame.frame_counter >> 3 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.game_state.world.region.flag_overworld_area_changed() {
            self.sprite_show_message_on_contact(k, 0x00d0);
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let elapsed = self.game_state.sprites.maze_game_timer.elapsed_low();
                self.maze_game_timer_mut().capture_snapshot();
                let mut t = elapsed % 6000;
                let a = t / 600;
                t %= 600;
                let b = t / 60;
                t %= 60;
                let c = t / 10;
                t %= 10;
                self.dialogue_number_mut()
                    .set_packed_digits((t | (c << 4)) as u8, (b | (a << 4)) as u8);
                let j = self.sprite_show_message_on_contact(k, 0x00cb);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let screen = usize::from(self.game_state.world.location.overworld_screen_index());
                if self
                    .game_state
                    .world
                    .overworld
                    .event_info
                    .event_info(screen)
                    & 0x40
                    != 0
                {
                    self.sprite_show_message_unconditional(0x00cf);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else if self.game_state.sprites.maze_game_timer.snapshot_low() < 16 {
                    self.sprite_show_message_unconditional(0x00cd);
                    let dir = self.game_state.player.follower_link.handler_state();
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x00ce);
                    let dir = self.game_state.player.follower_link.handler_state();
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                let j = self.sprite_show_message_on_contact(k, 0x00ce);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            3 => {
                let j = self.sprite_show_solicited_message(k, 0x00cd);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            4 => {
                let j = self.sprite_show_solicited_message(k, 0x00cf);
                if j & 0x100 != 0 {
                    let dir = j as u8;
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = dir;
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MazeGameGuy_Draw(int k) {  // 8dcda7
    pub(super) fn maze_game_guy_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &MAZE_GAME_GUY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void DrinkingGuy_Draw(int k) {  // 9af88c
    pub(super) fn drinking_guy_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple_player_deferred(
            k,
            &DRINKING_GUY_DRAW_FRAMES[base..base + 3],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_25_TalkingTree(int k) {  // 86c0d5
    pub(super) fn sprite_25_talking_tree(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.talking_tree_mouth(k),
            1 => self.talking_tree_eye(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Mouth(int k) {  // 9df956
    pub(super) fn talking_tree_mouth(&mut self, k: usize) {
        self.talking_tree_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_flags4(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.link_cancel_dash();
                    let pt = self.sprite_project_speed_towards_link(k, 48);
                    self.follower_link_state_mut().set_incapacitated_timer(16);
                    self.follower_link_state_mut()
                        .set_actual_velocity_xy(pt.x, pt.y);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let value = (self.sprite_slot_view(k).delay_main() >> 1) & 3;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            2 => {
                // The ROM indexes kTalkingTree_Gfx2 with the sprite slot, not
                // the shifted delay it just computed (see
                // TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT).
                let _ = TALKING_TREE_MOUTH_SECONDARY_GRAPHICS;
                let value = TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT[k];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 7 {
                    self.talking_tree_spawn_bomb(k);
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            3 => {
                let value = 7;
                self.sprite_slot_view_mut(k).set_flags4(value);
                if self.sprite_slot_view(k).a() == 0 {
                    let j = ((self.sprite_slot_view(k).x_low() >> 4) & 1) ^ 1;
                    let value = j;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if self
                        .sprite_show_solicited_message(k, TALKING_TREE_MOUTH_MSGS_2[usize::from(j)])
                        & 0x100
                        == 0
                    {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_a(value);
                    }
                } else {
                    let j = TALKING_TREE_MOUTH_SCREENS
                        .iter()
                        .rposition(|&screen| {
                            screen == self.game_state.world.location.overworld_screen_index()
                        })
                        .unwrap_or(0);
                    self.sprite_show_message_unconditional(TALKING_TREE_MOUTH_MSGS[j]);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_a(value);
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let j = self.sprite_slot_view(k).b().wrapping_add(1) & 7;
                    let value = j;
                    self.sprite_slot_view_mut(k).set_b(value);
                    let value = TALKING_TREE_MOUTH_LOCAL_GRAPHICS[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = TALKING_TREE_MOUTH_DELAY[usize::from(j)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Draw(int k) {  // 9dfadb
    pub(super) fn talking_tree_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics().wrapping_sub(1);
        if sign8(g) {
            return;
        }
        let base = usize::from(g) * 4;
        if base + 4 > TALKING_TREE_DRAW_FRAMES.len() {
            // Slots 6, 7, and 14 receive instruction bytes from the ROM's
            // slot-indexed kTalkingTree_Gfx2 read
            // (TALKING_TREE_MOUTH_SECONDARY_GRAPHICS_BY_SLOT); the ROM then
            // draws from past the end of kTalkingTree_Dmd. That garbage is
            // not modeled yet; skip the draw instead of indexing out of range.
            return;
        }
        self.sprite_draw_multiple_player_deferred(
            k,
            &TALKING_TREE_DRAW_FRAMES[base..base + 4],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void TalkingTree_Eye(int k) {  // 9dfb0a
    pub(super) fn talking_tree_eye(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let mut j = usize::from(self.sprite_slot_view(k).head_direction());
        let x = u16::from(self.sprite_slot_view(k).a())
            | (u16::from(self.sprite_slot_view(k).b()) << 8);
        let y = u16::from(self.sprite_slot_view(k).c())
            | (u16::from(self.sprite_slot_view(k).e()) << 8);
        self.sprite_set_x(
            k,
            x.wrapping_add_signed(i16::from(TALKING_TREE_EYE_BASE_X_OFFSETS[j])),
        );
        self.sprite_set_y(k, y);
        let pt = self.sprite_project_speed_towards_link(k, 2);
        if !sign8(pt.y) {
            let value = pt.x.wrapping_add(2);
            self.sprite_slot_view_mut(k).set_direction(value);
        } else if self.sprite_slot_view(k).direction() != 2 {
            let value = self.sprite_slot_view(k).direction().wrapping_add(
                if self.sprite_slot_view(k).direction() >= 2 {
                    0xff
                } else {
                    1
                },
            );
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        j = usize::from(self.sprite_slot_view(k).direction());
        self.sprite_set_x(
            k,
            x.wrapping_add_signed(i16::from(TALKING_TREE_EYE_X_OFFSETS_1[j])),
        );
        self.sprite_set_y(
            k,
            y.wrapping_add_signed(i16::from(TALKING_TREE_EYE_Y_OFFSETS_1[j])),
        );
    }

    // -----------------------------------------------------------------------
    // void DiggingGameGuy_Draw(int k) {  // 9dfe4b
    pub(super) fn digging_game_guy_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 3;
        self.sprite_draw_multiple_player_deferred(
            k,
            &DIGGING_GAME_GUY_DRAW_FRAMES[base..base + 3],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Bot_Draw(int k) {  // 85b89a
    pub(super) fn bot_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).a());
        let value = BOT_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | BOT_DRAW_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
    }

    // -----------------------------------------------------------------------
    // void BombShopEntity_Draw(int k) {  // 9ee2c6
    pub(super) fn bomb_shop_entity_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).subtype2()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple_player_deferred(
            k,
            &BOMB_SHOP_ENTITY_DRAW_FRAMES[base..base + 1],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Sprite_B3_PedestalPlaque(int k) {  // 9ee044
    pub(super) fn sprite_b3_pedestal_plaque(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if !self.game_state.player.follower_link.is_immobilized()
            && self.sprite_check_if_link_is_busy()
        {
            return;
        }

        self.follower_link_state_mut()
            .clear_position_mode_bits(0x20);
        if self.game_state.world.location.overworld_screen_index() != 48 {
            if self.game_state.player.follower_link.has_facing()
                || !self.sprite_check_damage_to_link_same_layer(k)
            {
                return;
            }
            if self.game_state.inventory.save_progress.hud_current_item()
                != HUD_ITEM_BOOK_MUDORA_DRAW
                || (self.game_state.player.follower_link.filtered_joypad_h() & 0x40) == 0
            {
                if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0 {
                    return;
                }
                self.sprite_show_message_unconditional(0x00b6);
            } else {
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut().set_position_mode(32);
                self.set_sound_effect_1(0);
                self.sprite_show_message_unconditional(0x00b7);
            }
        } else {
            if self.game_state.player.follower_link.has_facing()
                || !self.sprite_check_damage_to_link_same_layer(k)
            {
                return;
            }
            if self.game_state.inventory.save_progress.hud_current_item()
                != HUD_ITEM_BOOK_MUDORA_DRAW
                || (self.game_state.player.follower_link.filtered_joypad_h() & 0x40) == 0
            {
                if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0 {
                    return;
                }
                self.sprite_show_message_unconditional(0x00bc);
            } else {
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut().set_position_mode(32);
                self.set_sound_effect_1(0);
                self.follower_link_state_mut().set_button_b_frames(1);
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
                self.follower_link_state_mut()
                    .set_handler_state(PLAYER_STATE_OPENING_DESERT_PALACE_DRAW);
                self.sprite_show_message_unconditional(0x00bd);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B4_PurpleChest(int k) {  // 9ee0dd
    pub(super) fn sprite_b4_purple_chest(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            if self.sprite_show_message_on_contact(k, 0x0116) & 0x100 != 0
                && self.game_state.sprites.follower_runtime.indicator() == 0
            {
                let value = 1;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            self.follower_state_mut().set_indicator(12);
            self.load_follower_graphics();
            self.sprite_become_follower(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_B5_BombShop(int k) {  // 9ee111
    pub(super) fn sprite_b5_bomb_shop(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.sprite_bomb_shop_clerk(k),
            1 => self.sprite_bomb_shop_bomb(k),
            2 => self.sprite_bomb_shop_super_bomb(k),
            3 => self.sprite_bomb_shop_huff(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Clerk(int k) {  // 9ee134
    pub(super) fn sprite_bomb_shop_clerk(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let j = usize::from(self.sprite_slot_view(k).e());
            let value = self.sprite_slot_view(k).e().wrapping_add(1) & 7;
            self.sprite_slot_view_mut(k).set_e(value);
            let value = SPRITE_BOMB_SHOP_CLERK_DELAY[j];
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = SPRITE_BOMB_SHOP_CLERK_LOCAL_GRAPHICS[j];
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).graphics() == 0 {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x11);
                self.bomb_shop_clerk_exhalation(k);
            } else {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x12);
            }
        }
        let flag = (self.game_state.inventory.player_resources.crystal_flags() & 5) == 5
            && (self
                .game_state
                .inventory
                .save_progress
                .progress_indicator_3()
                & 32)
                != 0;
        self.sprite_show_solicited_message(k, if flag { 0x0118 } else { 0x0117 });
        self.sprite_behave_as_barrier(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Bomb(int k) {  // 9ee190
    pub(super) fn sprite_bomb_shop_bomb(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if !self.shop_item_check_for_a_press(k) {
            return;
        }

        let resources = &self.game_state.inventory.player_resources;
        let upgrade = resources.bomb_upgrade_level() as usize;
        if resources.bombs() != MAX_BOMBS_FOR_LEVEL_DRAW[upgrade] {
            if !self.shop_item_handle_cost(100) {
                self.sprite_show_message_unconditional(0x017c);
                self.shop_item_play_beep(k);
            } else {
                self.player_resources_mut().set_bomb_filler(27);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_show_message_unconditional(0x0119);
                self.shop_item_handle_receipt(k, 0x28);
            }
        } else {
            self.sprite_show_message_unconditional(0x016e);
            self.shop_item_play_beep(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_SuperBomb(int k) {  // 9ee1df
    pub(super) fn sprite_bomb_shop_super_bomb(&mut self, k: usize) {
        self.bomb_shop_entity_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.shop_item_check_for_a_press(k) {
            if !self.shop_item_handle_cost(100) {
                self.sprite_show_message_unconditional(0x017c);
                self.shop_item_play_beep(k);
            } else {
                self.follower_state_mut().set_indicator(13);
                self.load_follower_graphics();
                self.sprite_become_follower(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
                self.sprite_show_message_unconditional(0x011a);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BombShop_Huff(int k) {  // 9ee21a
    pub(super) fn sprite_bomb_shop_huff(&mut self, k: usize) {
        self.oam_allocate_from_region_c(4);
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).and_oam_flags(0x30);
        let value = SPRITE_BOMB_SHOP_HUFF_OAM_FLAGS
            [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
        self.sprite_slot_view_mut(k).or_oam_flags(value);
        self.sprite_slot_view_mut(k).add_z_velocity(1);
        self.sprite_move_z(k);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        let value = self.sprite_slot_view(k).delay_main() >> 3 & 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_BB_Shopkeeper(int k) {  // 9eeeef
    pub(super) fn sprite_bb_shopkeeper(&mut self, k: usize) {
        match self.sprite_slot_view(k).subtype2() {
            0 => self.shopkeeper_standard_clerk(k),
            1 => self.chest_game_guy(k),
            2 => self.nice_thief_with_gift(k),
            3 => self.mini_chest_game_guy(k),
            4 => self.lost_woods_chest_game_guy(k),
            5 | 6 => self.nice_thief_under_rock(k),
            7 => self.shop_item_red_potion150(k),
            8 => self.shop_item_fighter_shield(k),
            9 => self.shop_item_fire_shield(k),
            10 => self.shop_item_heart(k),
            11 => self.shop_item_arrows(k),
            12 => self.shop_item_bombs(k),
            13 => self.shop_item_bee(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Shopkeeper_StandardClerk(int k) {  // 9eef12
    pub(super) fn shopkeeper_standard_clerk(&mut self, k: usize) {
        if self.game_state.world.region.is_in_dark_world() {
            self.oam_allocate_defer_to_player(k);
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = (self.sprite_slot_view(k).oam_flags() & 63)
                | ((self.game_state.frame.frame_counter << 3) & 64);
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        } else {
            let value = 7;
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            self.shopkeeper_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = (self.game_state.frame.frame_counter >> 4) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        self.sprite_behave_as_barrier(k);
        let msg = if !self.game_state.world.region.is_in_dark_world() {
            0x0165
        } else {
            0x015f
        };
        self.sprite_show_solicited_message(k, msg);
        if self.sprite_slot_view(k).ai_state() == 0
            && self
                .game_state
                .sprites
                .workspace
                .current_sprite_y()
                .wrapping_add(0x60)
                >= self.game_state.player.follower_link.y()
        {
            self.sprite_show_message_unconditional(msg);
            let value = 1;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void ChestGameGuy(int k) {  // 9eef90
    pub(super) fn chest_game_guy(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        let value = (self.sprite_slot_view(k).oam_flags() & 63)
            | ((self.game_state.frame.frame_counter << 3) & 64);
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x0160) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(30) {
                    self.minigame_state_mut().set_credits(2);
                    self.sprite_show_message_unconditional(0x0164);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0161);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                if self.game_state.minigame.credits() == 0 {
                    self.sprite_show_solicited_message(k, 0x0163);
                } else {
                    self.sprite_show_solicited_message(k, 0x017f);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void NiceThiefWithGift(int k) {  // 9ef038
    pub(super) fn nice_thief_with_gift(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_show_solicited_message(k, 0x0176) & 0x100 != 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x4000 == 0 {
                    let bits =
                        self.game_state.dungeon.savegame_state.savegame_state_bits() | 0x4000;
                    self.dungeon_savegame_state_mut()
                        .set_savegame_state_bits(bits);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.shop_item_handle_receipt(k, 0x46);
                } else {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void MiniChestGameGuy(int k) {  // 9ef078
    pub(super) fn mini_chest_game_guy(&mut self, k: usize) {
        let value = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.maze_game_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x017e) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(20) {
                    self.minigame_state_mut().set_credits(1);
                    self.sprite_show_message_unconditional(0x017f);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0180);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_show_solicited_message(
                    k,
                    if self.game_state.minigame.credits() == 0 {
                        0x0163
                    } else {
                        0x017f
                    },
                );
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void LostWoodsChestGameGuy(int k) {  // 9ef0f3
    pub(super) fn lost_woods_chest_game_guy(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.game_state.minigame.credits().wrapping_sub(1) >= 2
                    && self.sprite_show_solicited_message(k, 0x0181) & 0x100 != 0
                {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.multiselect_choice().value() == 0 && self.shop_item_handle_cost(100) {
                    self.minigame_state_mut().set_credits(1);
                    self.sprite_show_message_unconditional(0x017f);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0180);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.sprite_show_solicited_message(
                    k,
                    if self.game_state.minigame.credits() == 0 {
                        0x0163
                    } else {
                        0x017f
                    },
                );
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void NiceThiefUnderRock(int k) {  // 9ef14f
    pub(super) fn nice_thief_under_rock(&mut self, k: usize) {
        self.nice_thief_animate(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_show_solicited_message(
            k,
            if self.sprite_slot_view(k).subtype2() == 5 {
                0x0177
            } else {
                0x0178
            },
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_BC_Drunkard(int k) {  // 9ef603
    pub(super) fn sprite_bc_drunkard(&mut self, k: usize) {
        self.drinking_guy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.get_random_number() == 0 {
            let value = 32;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        let value = u8::from(self.sprite_slot_view(k).delay_main() != 0);
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_show_solicited_message(k, 0x0175) & 0x100 != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_17_Hoarder(int k) {  // 86a86c
    pub(super) fn sprite_17_hoarder(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_hoarder_frantic(k);
        } else {
            self.sprite_hoarder_covered(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hoarder_Covered(int k) {  // 86a874
    pub(super) fn sprite_hoarder_covered(&mut self, k: usize) {
        self.covered_rupee_crab_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_graphics(value);
        let mut pt = PointU8 { x: 0, y: 0 };
        let dir = self.sprite_direction_to_face_link(k, Some(&mut pt)) as usize;
        if self.sprite_slot_view(k).delay_main() != 0
            || (pt.y.wrapping_add(0x30) < 0x60 && pt.x.wrapping_add(0x20) < 0x40)
        {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 32;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            let value = SPRITE_HOARDER_COVERED_LOCAL_X_VELOCITIES[dir] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_HOARDER_COVERED_LOCAL_Y_VELOCITIES[dir] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            if self.sprite_slot_view(k).wall_collision() == 0 {
                self.sprite_move_xy(k);
            }
            self.sprite_check_tile_collision2(k);
            self.sprite_check_damage_from_link(k);
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let value = SPRITE_HOARDER_COVERED_LOCAL_GRAPHICS
                [usize::from(self.sprite_slot_view(k).subtype2() >> 1) & 3];
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        if self.sprite_slot_view(k).sprite_type() != 0x3e
            || self.game_state.inventory.items.gloves() >= 1
        {
            self.sprite_return_if_lifted_permissive(k);
        }
        if self.sprite_slot_view(k).state() != 9 {
            let value = if self.sprite_slot_view(k).sprite_type() == 0x17 {
                2
            } else {
                1
            };
            self.sprite_slot_view_mut(k).set_c(value);
            let value = 0xec;
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            self.sprite_slot_view_mut(k).and_oam_flags(!1);
            let value = 0;
            self.sprite_slot_view_mut(k).set_graphics(value);
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x3e, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_view_mut(j).and_flags2(!0x80);
                let value = 128;
                self.sprite_slot_view_mut(j).set_delay_aux2(value);
                let value = 9;
                self.sprite_slot_view_mut(j).set_oam_flags(value);
                let value = 9;
                self.sprite_slot_view_mut(j).set_ai_state(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Hoarder_Frantic(int k) {  // 86a91d
    pub(super) fn sprite_hoarder_frantic(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_from_link(k);
        if self.sprite_slot_view(k).delay_aux2() == 0 {
            self.sprite_check_damage_to_link(k);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let mut j = usize::from(self.sprite_slot_view(k).subtype2() >> 1) & 3;
        let value = SPRITE_HOARDER_FRANTIC_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_HOARDER_FRANTIC_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).wall_collision() != 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_aux4(value);
            j = usize::from(self.get_random_number() & 3);
            let value = SPRITE_HOARDER_FRANTIC_LOCAL_X_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_HOARDER_FRANTIC_LOCAL_Y_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        } else {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision2(k);
        if self.sprite_slot_view(k).delay_aux4() == 0
            && (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0
        {
            let pt = self.sprite_project_speed_towards_link(k, 16);
            let value = pt.y.wrapping_neg();
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = pt.x.wrapping_neg();
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        if (self.game_state.frame.frame_counter & 1) != 0 {
            return;
        }

        self.sprite_slot_view_mut(k).add_g(1);
        let (end, typ) = if self.sprite_slot_view(k).g() == 192 {
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 6;
            self.sprite_slot_view_mut(k).set_state(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            (1, 0xd9)
        } else {
            if (self.sprite_slot_view(k).g() & 15) != 0 {
                return;
            }
            (
                0,
                if self.sprite_slot_view(k).head_direction() == 6 {
                    0xdb
                } else {
                    0xd9
                },
            )
        };
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, typ, &mut info, end);
        if j >= 0 {
            let j = j as usize;
            self.sprite_slot_view_mut(k).add_head_direction(1);
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_set_x(j, info.r0_x.wrapping_add(8));
            let value = 32;
            self.sprite_slot_view_mut(j).set_z_velocity(value);
            let value = 16;
            self.sprite_slot_view_mut(j).set_delay_aux4(value);
            let pt = self.sprite_project_speed_towards_link(j, 16);
            let value = !pt.y;
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = !pt.x;
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x30);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_EC_ThrownItem(int k) {  // 86aae0
    pub(super) fn sprite_ec_thrown_item(&mut self, k: usize) {
        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            if self.game_state.oam.has_sprite_sorting() && self.sprite_slot_view(k).floor() != 0 {
                let spr_slot = 0x2c + (k & 3);
                self.oam_state_mut()
                    .set_current_pointer((0x0800 + spr_slot * 4) as u16);
                self.oam_state_mut()
                    .set_current_extended_pointer((0x0a20 + spr_slot) as u16);
            }
            let value = self.sprite_slot_view(k).state();
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            if self.sprite_slot_view(k).c() >= 6 {
                self.sprite_draw_thrown_item_gigantic(k);
            } else {
                self.sprite_draw_single_large(k);
                let oam = self.game_state.oam.current_pointer_usize();
                let t = self
                    .game_state
                    .world
                    .location
                    .indoor_flag
                    .wrapping_add(self.game_state.world.region.dark_world_region_index());
                let j = usize::from(self.sprite_slot_view(k).c());
                let charnum = THROWABLE_SCENERY_CHARS[j + if t >= 2 { 6 } else { 0 }];
                self.oam_state_mut().set_entry_char(oam, charnum);
                self.oam_state_mut()
                    .merge_entry_flags(oam, 0xf0, THROWABLE_SCENERY_FLAGS[j]);
                let value = (self.sprite_slot_view(k).oam_flags() & 0xc0)
                    | (self.game_state.oam.entry_flags(oam) & 0x0f);
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            }
        }
        if self.sprite_slot_view(k).state() == 9 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.throwable_scenery_interact_with_sprites_and_tiles(k);
        }
    }

    // -----------------------------------------------------------------------
    // void CoveredRupeeCrab_Draw(int k) {  // 86aa48
    pub(super) fn covered_rupee_crab_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        if self.game_state.sprites.system.chr_halfslot_state() >= 3 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let r7 = if self.sprite_slot_view(k).sprite_type() == 0x17 {
            2
        } else {
            0
        };
        let r6 = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        for i in (0..2).rev() {
            let j = i + r6;
            let ch = COVERED_RUPEE_CRAB_DRAW_CHARS[j];
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x,
                info_y.wrapping_add(COVERED_RUPEE_CRAB_DRAW_Y_OFFSETS[j] as i16 as u16),
                ch.wrapping_add(if ch == 0x44 { r7 } else { 0 }),
                (info_flags & !1) | COVERED_RUPEE_CRAB_DRAW_FL[j],
                2,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_28_DarkWorldHintNPC(int k) {  // 86ad6f
    pub(super) fn sprite_28_dark_world_hint_npc(&mut self, k: usize) {
        self.story_teller_1_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = (self.game_state.frame.frame_counter >> 4) & 1;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }

        match self.sprite_slot_view(k).subtype2() {
            0 => self.dark_world_hint_npc_run_state(k, 0x00ff),
            1 => self.dark_world_hint_npc_run_state(k, 0x0101),
            2 => self.dark_world_hint_npc_run_state(k, 0x0102),
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    if (self.game_state.frame.frame_counter & 0x3f) == 0 {
                        self.sprite_slot_view_mut(k).xor_oam_flags(0x40);
                    }
                    if self.get_random_number() == 0 {
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
                self.sprite_show_solicited_message(k, 0x0149);
            }
            4 => {
                let value = (self.game_state.frame.frame_counter >> 1) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_move_z(k);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                }
                let value = self.sprite_slot_view(k).z_velocity().wrapping_add(
                    if self.sprite_slot_view(k).z() >= 4 {
                        0xff
                    } else {
                        1
                    },
                );
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.dark_world_hint_npc_run_state(k, 0x0103);
            }
            _ => {}
        }
    }

    fn dark_world_hint_npc_run_state(&mut self, k: usize, paid_msg: u16) {
        match self.sprite_slot_view(k).ai_state() {
            0 => self.dark_world_hint_npc_idle(k),
            1 => {
                if self.multiselect_choice().value() == 0
                    && self.dark_world_hint_npc_handle_payment()
                {
                    self.sprite_show_message_unconditional(paid_msg);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                } else {
                    self.sprite_show_message_unconditional(0x0100);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => self.dark_world_hint_npc_restore_health(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void StoryTeller_1_Draw(int k) {  // 86af1a
    pub(super) fn story_teller_1_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).subtype2()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple_player_deferred(
            k,
            &STORY_TELLER_1_DRAW_FRAMES[base..base + 1],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SmithyFrog_Draw(int k) {  // 86b339
    pub(super) fn smithy_frog_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple_player_deferred(k, &SMITHY_FROG_DRAW_FRAMES, Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void SmithySpark_Draw(int k) {  // 86b72c
    pub(super) fn smithy_spark_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(8);
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &SMITHY_SPARK_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void GerudoMan_Draw(int k) {  // 85ba24
    pub(super) fn gerudo_man_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let j = g * 3 + i;
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(GERUDO_MAN_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(GERUDO_MAN_DRAW_Y_OFFSETS[j] as i16 as u16),
                GERUDO_MAN_DRAW_CHARS[j],
                GERUDO_MAN_DRAW_FL[j] | info_flags,
                GERUDO_MAN_DRAW_BIG[j],
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Recruit_Draw(int k) {  // 85bd7e
    pub(super) fn recruit_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = SpritePrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let hd = usize::from(self.sprite_slot_view(k).head_direction() & 3);
        self.set_oam_helper0_at_for_draw(
            oam,
            info_x,
            info_y.wrapping_sub(11),
            RECRUIT_DRAW_SOLDIER_CH[hd],
            RECRUIT_DRAW_SOLDIER_FL[hd] | info_flags,
            2,
        );
        let r6 = usize::from(self.sprite_slot_view(k).graphics());
        self.set_oam_helper0_at_for_draw(
            oam + 4,
            info_x.wrapping_add(RECRUIT_DRAW_X_OFFSETS[r6] as u16),
            info_y,
            RECRUIT_DRAW_CHARS[r6],
            RECRUIT_DRAW_FL[r6] | info_flags,
            2,
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void QuarrelBros_Draw(int k) {  // 85e17f
    pub(super) fn quarrel_bros_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2
            + usize::from(self.sprite_slot_view(k).direction()) * 4;
        self.sprite_draw_multiple_player_deferred(
            k,
            &QUARREL_BROS_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Lumberjacks_Draw(int k) {  // 8dc6ba
    pub(super) fn lumberjacks_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 11;
        self.sprite_draw_multiple(k, &LUMBERJACKS_DRAW_FRAMES[base..base + 11], None);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Lumberjacks(int k) {  // 8dc51b
    pub(super) fn sprite_lumberjacks(&mut self, k: usize) {
        self.lumberjacks_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.lumberjack_check_proximity(k, 0) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
        }
        if !self.sprite_check_if_link_is_busy()
            && self.lumberjack_check_proximity(k, 1)
            && self.game_state.player.follower_link.filtered_joypad_l() & 0x80 != 0
        {
            let msg = usize::from(
                self.game_state.player.follower_link.x() as u8 >= self.sprite_slot_view(k).x_low(),
            ) + usize::from(self.game_state.inventory.items.sword_type() >= 2) * 2;
            self.sprite_show_message_unconditional(SPRITE_LUMBERJACKS_MESSAGES[msg]);
        }
        let value = self.game_state.frame.frame_counter >> 5 & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void GreatCatfish_Draw(int k) {  // 9de320
    pub(super) fn great_catfish_draw(&mut self, k: usize) {
        let g = self.sprite_slot_view(k).graphics();
        if g != 0 {
            let base = usize::from(g - 1) * 4;
            self.sprite_draw_multiple(k, &GREAT_CATFISH_DRAW_FRAMES[base..base + 4], None);
        }
    }

    // -----------------------------------------------------------------------
    // void BigFaerie_Draw(int k) {  // 9dc5d0
    pub(super) fn big_faerie_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &BIG_FAERIE_DRAW_FRAMES[base..base + 4], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void FaerieCloud_Draw(int k) {  // 9dc616
    pub(super) fn faerie_cloud_draw(&mut self, k: usize) {
        if !sign8(self.sprite_slot_view(k).a())
            && (self.sprite_slot_view(k).a() & self.sprite_slot_view(k).subtype2()) == 0
        {
            let x = FAERIE_CLOUD_DRAW_XY[usize::from(self.get_random_number() & 7)];
            let y = FAERIE_CLOUD_DRAW_XY[usize::from(self.get_random_number() & 7)];
            self.sprite_garnish_spawn_sparkle(k, x, y);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_C8_BigFairy(int k) {  // 9dc414
    pub(super) fn sprite_c8_big_fairy(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() != 0 {
            self.sprite_fairy_cloud(k);
        } else {
            self.sprite_big_fairy(k);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_FairyCloud(int k) {  // 9dc41c
    pub(super) fn sprite_fairy_cloud(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.faerie_cloud_draw(k);
        if (self.sprite_slot_view(k).subtype2() & 31) == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x31);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_a(value);
                self.sprite_apply_speed_towards_link(k, 8);
                self.sprite_move_xy(k);
                self.sprite_get16_bit_coords(k);
                let x = self
                    .game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                    .wrapping_add(3);
                let y = self
                    .game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                    .wrapping_add(11);
                if x < 6 && y < 6 {
                    self.player_resources_mut()
                        .increment_heart_filler_word_by(0x00a0);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                let resources = &self.game_state.inventory.player_resources;
                if resources.current_health() == resources.health_capacity() {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    self.sprite_slot_view_mut(0).set_delay_aux2(112);
                }
            }
            2 => {
                if (self.sprite_slot_view(k).subtype2() & 15) != 0
                    || sign8(self.sprite_slot_view(k).a())
                {
                    return;
                }
                let value = self.sprite_slot_view(k).a().wrapping_mul(2).wrapping_add(1);
                self.sprite_slot_view_mut(k).set_a(value);
                if self.sprite_slot_view(k).a() >= 0x80 {
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_a(value);
                    self.follower_link_state_mut().clear_immobilized();
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_BigFairy(int k) {  // 9dc4bf
    pub(super) fn sprite_big_fairy(&mut self, k: usize) {
        let mut i = self.sprite_slot_view(k).delay_aux2();
        if i != 0 && i < 0x40 {
            i = i.wrapping_sub(1);
            if i == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            if (i & 1) != 0 {
                return;
            }
        }
        self.big_faerie_draw(k);
        self.sprite_slot_view_mut(k).subtract_g(1);
        if sign8(self.sprite_slot_view(k).g()) {
            let value = 5;
            self.sprite_slot_view_mut(k).set_g(value);
            let value = self.sprite_slot_view(k).graphics().wrapping_add(1) & 3;
            self.sprite_slot_view_mut(k).set_graphics(value);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.faerie_cloud_draw(k);
                let value = 1;
                self.sprite_slot_view_mut(k).set_a(value);
                let mut pt = PointU8 { x: 0, y: 0 };
                self.sprite_direction_to_face_link(k, Some(&mut pt));
                if pt.x.wrapping_add(0x30) < 0x60 && pt.y.wrapping_add(0x30) < 0x60 {
                    self.link_cancel_dash();
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.dialogue_message_index_mut().set_value(0x015a);
                    self.sprite_show_message_minimal_c();
                    self.follower_link_state_mut().immobilize();
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0xc8, &mut info);
                    assert!(
                        j >= 0,
                        "Sprite_BigFairy expected Sprite_SpawnDynamically to succeed"
                    );
                    let j = j as usize;
                    self.sprite_set_spawned_coordinates(j, &info);
                    let value = 1;
                    self.sprite_slot_view_mut(j).set_head_direction(value);
                    let value = self
                        .sprite_slot_view(j)
                        .y_low()
                        .wrapping_sub(self.sprite_slot_view(k).z());
                    self.sprite_slot_view_mut(j).set_y_low(value);
                    let value = 0;
                    self.sprite_slot_view_mut(j).set_z(value);
                }
            }
            1 => {}
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_AC_Apple(int k) {  // 9ef515
    pub(super) fn sprite_ac_apple(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_apple(k);
            return;
        }
        if self.sprite_slot_view(k).e() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            let mut n = (self.get_random_number() & 3).wrapping_add(2);
            loop {
                self.spawn_apple(k);
                if n == 0 {
                    break;
                }
                n = n.wrapping_sub(1);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Apple(int k) {  // 9ef57c
    pub(super) fn sprite_apple(&mut self, k: usize) {
        if self.sprite_slot_view(k).a() >= 16 || (self.game_state.frame.frame_counter & 2) != 0 {
            self.sprite_draw_single_large(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).a() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        self.sprite_move_xyz(k);
        if self.sprite_check_damage_to_link(k) {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x0b);
            self.player_resources_mut().increment_heart_filler_by(8);
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
            return;
        }
        if (self.game_state.frame.frame_counter & 1) == 0 {
            self.sprite_slot_view_mut(k).subtract_a(1);
        }

        if !sign8(self.sprite_slot_view(k).z().wrapping_sub(1)) {
            self.sprite_slot_view_mut(k).subtract_z_velocity(1);
            return;
        }
        let value = 0;
        self.sprite_slot_view_mut(k).set_z(value);
        let a = if sign8(self.sprite_slot_view(k).z_velocity()) {
            self.sprite_slot_view(k).z_velocity()
        } else {
            0
        };
        let value = a.wrapping_neg() >> 1;
        self.sprite_slot_view_mut(k).set_z_velocity(value);
        if self.sprite_slot_view(k).x_velocity() != 0 {
            let value = self.sprite_slot_view(k).x_velocity().wrapping_add(
                if sign8(self.sprite_slot_view(k).x_velocity()) {
                    1
                } else {
                    0xff
                },
            );
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        if self.sprite_slot_view(k).y_velocity() != 0 {
            let value = self.sprite_slot_view(k).y_velocity().wrapping_add(
                if sign8(self.sprite_slot_view(k).y_velocity()) {
                    1
                } else {
                    0xff
                },
            );
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }
    }

    // -----------------------------------------------------------------------
    // void PsychoTrooper_Draw(int k) {  // 85ccd5
    pub(super) fn psycho_trooper_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 3);
        self.sprite_draw_bnc_body(k, &info, 2);
        self.sprite_draw_guard_spear(k, &info, 0);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = SpritePrepOamCoordsRet {
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
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 3);
        self.sprite_draw_bnc_body(k, &info, 2);
        if self.sprite_slot_view(k).graphics() < 20 {
            self.sprite_draw_guard_spear(k, &info, 0);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = SpritePrepOamCoordsRet {
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
        self.sprite_draw_single_large(k);
        self.sprite_workspace_mut().set_current_sprite_y(bak2);
        let value = bak1;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = bak0;
        self.sprite_slot_view_mut(k).set_graphics(value);

        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        let guard_info = SpritePrepOamCoordsRet {
            x: tuple.0,
            y: tuple.1,
            r4: 0,
            flags: tuple.2,
        };
        self.guard_animate_head(k, 0x10 / 4, &guard_info);
        self.sprite_draw_bnc_body(k, &info, 0x0c / 4);
        if self.sprite_slot_view(k).graphics() < 20 {
            self.sprite_draw_guard_spear(k, &info, 4 / 4);
        }
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = SpritePrepOamCoordsRet {
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
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        let guard_info = SpritePrepOamCoordsRet {
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
            let mut shadow_info = SpritePrepOamCoordsRet {
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

    pub(super) fn hog_spear_man_through_body_increments(&mut self, k: usize) -> bool {
        self.guard_handle_all_animation(k);
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
            let dir = self.sprite_direction_to_face_link(k, None);
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
        self.sprite_check_tile_collision(k);
        self.sprite_check_damage_to_link(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
            let dir = self.sprite_direction_to_face_link(k, None);
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
    pub(super) fn spike_trap_draw(&mut self, k: usize) {
        self.sprite_draw_multiple(k, &SPIKE_TRAP_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void ChainBallTrooper_Draw(int k) {  // sprite_main.c:1403
    pub(super) fn chain_ball_trooper_draw(&mut self, k: usize) {
        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        self.sprite_draw_guard_head(k, &info, 0x18 / 4);
        self.sprite_draw_bnc_body(k, &info, 0x14 / 4);
        self.sprite_draw_bnc_flail(k, &info);

        let Some(tuple) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet::from_tuple(tuple);
        if self.sprite_slot_view(k).flags3() & 0x10 != 0 {
            let mut shadow_info = SpritePrepOamCoordsRet {
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
    // void Sprite_63_DebirandoPit(int k) {  // 858531
    pub(super) fn sprite_63_debirando_pit(&mut self, k: usize) {
        let mut pt = PointU8 { x: 0, y: 0 };
        self.sprite_direction_to_face_link(k, Some(&mut pt));
        if pt.y.wrapping_add(0x20) < 0x40 && pt.x.wrapping_add(0x20) < 0x40 {
            self.oam_allocate_from_region_b(16);
        }

        self.debirando_pit_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let j = usize::from(self.sprite_slot_view(k).head_direction());
        if self.sprite_slot_view(j).state() == 6 {
            let value = self.sprite_slot_view(j).state();
            self.sprite_slot_view_mut(k).set_state(value);
            let value = self.sprite_slot_view(j).delay_main();
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            return;
        }
        if self.sprite_slot_view(k).graphics() < 3 && self.sprite_check_damage_to_link_same_layer(k)
        {
            self.link_cancel_dash();
            if (self.game_state.player.follower_link.filtered_joypad_l() & 16) == 0 {
                self.follower_link_state_mut().prevent_movement();
            }
            self.sprite_apply_speed_towards_link(k, 16);
            let v = self.sprite_slot_view(k).y_velocity();
            let mag = if sign8(v) { v.wrapping_neg() } else { v };
            let t = u16::from(mag) + u16::from(self.sprite_slot_view(k).a());
            let value = t as u8;
            self.sprite_slot_view_mut(k).set_a(value);
            if t >= 256 {
                self.follower_link_state_mut()
                    .set_drag_player_y(if sign8(v) { 1 } else { (-1i16) as u16 });
            }

            let v = self.sprite_slot_view(k).x_velocity();
            let mag = if sign8(v) { v.wrapping_neg() } else { v };
            let t = u16::from(mag) + u16::from(self.sprite_slot_view(k).b());
            let value = t as u8;
            self.sprite_slot_view_mut(k).set_b(value);
            if t >= 256 {
                self.follower_link_state_mut()
                    .set_drag_player_x(if sign8(v) { 1 } else { (-1i16) as u16 });
            }
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 6;
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 63;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 255;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_63_DEBIRANDO_PIT_OPENING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                if (self.game_state.frame.frame_counter & 15) == 0 {
                    self.sprite_slot_view_mut(k).add_graphics(1);
                    if self.sprite_slot_view(k).graphics() >= 3 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 63;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_63_DEBIRANDO_PIT_CLOSING_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_64_Debirando(int k) {  // 85874d
    pub(super) fn sprite_64_debirando(&mut self, k: usize) {
        self.debirando_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 128;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_64_DEBIRANDO_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                } else {
                    if (self.sprite_slot_view(k).delay_main() & 31)
                        | self.sprite_slot_view(k).g()
                        | self.game_state.frame.submodule
                        | self.sprite_slot_view(k).pause()
                        | self.game_state.frame.modal_pause_flag
                        == 0
                    {
                        let _ = self.sprite_spawn_fireball(k);
                    }
                    self.sprite_slot_view_mut(k).add_subtype2(1);
                    let value = ((self.sprite_slot_view(k).subtype2() >> 3) & 1) + 2;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 223;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_64_DEBIRANDO_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 4)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void DebirandoPit_Draw(int k) {  // 8586e4
    pub(super) fn debirando_pit_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let g = usize::from(self.sprite_slot_view(k).graphics());
        if g == 6 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let big = DEBIRANDO_PIT_DRAW_BIG[g];
        for i in (0..4).rev() {
            let j = g * 4 + i;
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(DEBIRANDO_PIT_DRAW_X_OFFSETS[j] as u16),
                info_y.wrapping_add(DEBIRANDO_PIT_DRAW_Y_OFFSETS[j] as u16),
                DEBIRANDO_PIT_DRAW_CHARS[j],
                DEBIRANDO_PIT_DRAW_FL[j] | info_flags,
                big,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Debirando_Draw(int k) {  // 858857
    pub(super) fn debirando_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            return;
        }

        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let d = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        for i in (0..4).rev() {
            let j = d + i;
            let f = DEBIRANDO_DRAW_FL[j];
            let flags = (f ^ info_flags) & if (f & 0x0f) == 0 { 0xf0 } else { 0xff };
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(DEBIRANDO_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(DEBIRANDO_DRAW_Y_OFFSETS[j] as i16 as u16),
                DEBIRANDO_DRAW_CHARS[j],
                flags,
                DEBIRANDO_DRAW_BIG[j],
            );
            oam += 4;
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
        let dmg = self.sprite_check_damage_to_link_same_layer(k);
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
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
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
        let mut pt = PointU8 { x: 0, y: 0 };
        self.sprite_direction_to_face_link(k, Some(&mut pt));
        if pt.x.wrapping_add(0x20) < 0x40 && pt.y.wrapping_add(0x20) < 0x40 {
            self.oam_allocate_from_region_b(16);
        }
        self.sprite_draw_multiple(k, &DESERT_BARRIER_DRAW_FRAMES, None);
    }

    // -----------------------------------------------------------------------
    // void SageMantle_Draw(int k) {  // 85dc8a
    pub(super) fn sage_mantle_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).c() == 0 {
            self.oam_allocate_from_region_b(0x10);
        }
        self.sprite_draw_multiple(k, &SAGE_MANTLE_DRAW_FRAMES, None);
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
        if self.sprite_check_damage_to_link_same_layer(k) {
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
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
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
    pub(super) fn complete_book_of_mudora_item_receipt(&mut self, k: usize) {
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
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.follower_link_state_mut().set_speed_setting(0);
            self.sprite_repel_dash();
        }
        self.sprite_move_xy(k);
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
        if self.sprite_prep_oam_coord_or_double_ret(k).is_none() {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        let (x0, y0) = self
            .draw_scratch_position_mut()
            .offset_low_position(0u8.wrapping_sub(0x20), 0u8.wrapping_sub(0x20));
        if self.sprite_slot_view(k).subtype2() == 0 {
            for i in 0..16 {
                let entry_x = x0.wrapping_add(((i & 3) as u8) * 0x10);
                let entry_y = y0.wrapping_add(((i >> 2) as u8) * 0x10);
                self.oam_state_mut().set_entry_x(oam + i * 4, entry_x);
                self.oam_state_mut().set_entry_y(oam + i * 4, entry_y);
                self.oam_state_mut()
                    .set_entry_char_flags(oam + i * 4, DASH_TREE_TOP_GRID_CHAR_FLAGS[i]);
            }
        } else {
            let j = usize::from(self.sprite_slot_view(k).subtype2() - 1);
            for i in (0..16).rev() {
                self.oam_state_mut().write_entry(
                    oam,
                    x0.wrapping_add(DASH_TREE_TOP_DRAW_X_OFFSETS[i] as u8),
                    y0.wrapping_add(DASH_TREE_TOP_DRAW_Y_OFFSETS[i] as u8),
                    DASH_TREE_TOP_DRAW_CHARS[j],
                    DASH_TREE_TOP_DRAW_FL[j],
                );
                oam += 4;
            }
        }
        self.sprite_correct_oam_entries_for_draw(k, 15, 2);
    }

    // -----------------------------------------------------------------------
    // void Sprite_TroughBoy(int k) {  // 85ff66
    pub(super) fn sprite_trough_boy(&mut self, k: usize) {
        self.trough_boy_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier(k);
        self.sprite_track_body_to_head(k);
        let value = self.sprite_direction_to_face_link(k, None) ^ 3;
        self.sprite_slot_view_mut(k).set_head_direction(value);

        if self
            .game_state
            .inventory
            .save_progress
            .map_icons_indicator()
            < 3
        {
            if self.sprite_show_solicited_message(k, 0x147) & 0x100 != 0 {
                self.save_progress_mut().set_map_icons_indicator(2);
            }
        } else {
            self.sprite_show_solicited_message(k, 0x148);
        }
    }

    // -----------------------------------------------------------------------
    // void TroughBoy_Draw(int k) {  // 85ffdf
    pub(super) fn trough_boy_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).direction()) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &TROUGH_BOY_DRAW_FRAMES[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void RetreatBat_Draw(int k) {  // 9af833
    pub(super) fn retreat_bat_draw(&mut self, k: usize) {
        self.oam_state_mut().set_current_pointer(0x960);
        self.oam_state_mut().set_current_extended_pointer(0xa78);
        let j = usize::from(self.sprite_slot_view(k).direction()) * 4
            + usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple(
            k,
            &RETREAT_BAT_DRAW_FRAMES[RETREAT_BAT_DRAW_OFFSETS[j]
                ..RETREAT_BAT_DRAW_OFFSETS[j] + RETREAT_BAT_DRAW_COUNT[j]],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void GanonBat_Draw(int k) {  // 9d89eb
    pub(super) fn ganon_bat_draw(&mut self, k: usize) {
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &GANON_BAT_DRAW_FRAMES[base..base + 2], None);
    }

    // -----------------------------------------------------------------------
    // void EvilBarrier_Draw(int k) {  // 9df249
    pub(super) fn evil_barrier_draw(&mut self, k: usize) {
        let y = self
            .game_state
            .sprites
            .workspace
            .current_sprite_y()
            .wrapping_add(8);
        self.sprite_workspace_mut().set_current_sprite_y(y);
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 9;
        self.sprite_draw_multiple(k, &EVIL_BARRIER_DRAW_FRAMES[base..base + 9], None);
        self.sprite_get16_bit_coords(k);
    }

    // -----------------------------------------------------------------------
    // void ChattyAgahnim_Draw(int k, PrepOamCoordsRet *info) {  // 9dd451
    pub(super) fn chatty_agahnim_draw(&mut self, k: usize, info: &mut PrepOamCoordsRet) {
        if self.sprite_slot_view(k).delay_aux4() & 1 != 0 {
            return;
        }
        if self.sprite_slot_view(k).c() == 0 {
            self.oam_state_mut().set_current_pointer(0x900);
            self.oam_state_mut().set_current_extended_pointer(0xa60);
        }
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        let mut shadow_info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(
            k,
            &CHATTY_AGAHNIM_DRAW_FRAMES[base..base + 4],
            Some(&mut shadow_info),
        );
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 18);
        info.x = shadow_info.x;
        info.y = shadow_info.y;
        info.r4 = shadow_info.r4;
        info.flags = shadow_info.flags;
    }

    // -----------------------------------------------------------------------
    // void FaerieQueen_Draw(int k) {  // 86cb26
    pub(super) fn faerie_queen_draw(&mut self, k: usize) {
        if self.game_state.inventory.save_progress.dark_world_state() == 0 {
            let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k)
            else {
                return;
            };
            let mut oam = self.game_state.oam.current_pointer_usize();
            let g = usize::from(self.sprite_slot_view(k).graphics());
            for i in (0..12).rev() {
                let j = g * 12 + i;
                self.set_oam_plain_at_for_draw(
                    oam,
                    FAERIE_QUEEN_DRAW_X_OFFSETS[j].wrapping_add(info_x as u8),
                    FAERIE_QUEEN_DRAW_Y_OFFSETS[j].wrapping_add(info_y as u8),
                    FAERIE_QUEEN_DRAW_CHARS[j],
                    info_flags | FAERIE_QUEEN_DRAW_FL[j],
                    FAERIE_QUEEN_DRAW_BIG[j],
                );
                oam += 4;
            }
            self.sprite_correct_oam_entries_for_draw(k, 11, 0xff);
        } else {
            let base = usize::from(self.sprite_slot_view(k).graphics()) * 10;
            self.sprite_draw_multiple(k, &FAERIE_QUEEN_DRAW_FRAMES[base..base + 10], None);
        }
    }

    // -----------------------------------------------------------------------
    // void CrystalMaiden_Draw(int k) {  // 8dce5f
    pub(super) fn crystal_maiden_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).direction()) * 2
            + usize::from(self.sprite_slot_view(k).graphics());
        self.set_sprite_dma_head_pointer(CRYSTAL_MAIDEN_DRAW_DMA[j * 2]);
        self.set_sprite_dma_body_pointer(CRYSTAL_MAIDEN_DRAW_DMA[j * 2 + 1]);
        self.sprite_draw_multiple_player_deferred(
            k,
            &CRYSTAL_MAIDEN_DRAW_FRAMES[j * 2..j * 2 + 2],
            None,
        );
    }

    // -----------------------------------------------------------------------
    // void Sprite_0F_Octoballoon(int k) {  // 86d6aa
    pub(super) fn sprite_0_f_octoballoon(&mut self, k: usize) {
        let value =
            SPRITE_0_F_OCTOBALLOON_Z[usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 7)];

        self.sprite_slot_view_mut(k).set_z(value);
        self.octoballoon_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).delay_main() == 0 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            if !self.octoballoon_find() {
                let value = 6;
                self.sprite_slot_view_mut(k).set_state(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_hit_timer(value);
                let value = 15;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                return;
            }
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        if ((k as u8) ^ self.game_state.frame.frame_counter) & 15 == 0 {
            let pt = self.sprite_project_speed_towards_link(k, 4);
            let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt.x);
            if dx != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(if sign8(dx) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt.y);
            if dy != 0 {
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(if sign8(dy) { 1 } else { 0xff });
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
        }
        self.sprite_move_xy(k);
        if self.sprite_check_damage_to_link(k) {
            self.octoballoon_recoil_link(k);
        }
        self.sprite_check_damage_from_link(k);
        let _ = self.sprite_check_tile_collision(k);
        self.sprite_bounce_off_wall(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_10_OctoballoonBaby(int k) {  // 86d853
    pub(super) fn sprite_10_octoballoon_baby(&mut self, k: usize) {
        if self.sprite_slot_view(k).subtype2() == 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        if self.sprite_slot_view(k).subtype2() >= 64
            || (self.sprite_slot_view(k).subtype2() & 1) == 0
        {
            self.sprite_draw_single_small(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_subtype2(1);
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_slot_view_mut(k).subtract_z_velocity(1);
        self.sprite_move_z(k);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
        }
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        self.sprite_bounce_off_wall(k);
        self.sprite_check_damage_to_and_from_link(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_0D_Buzzblob(int k) {  // 86d89a
    pub(super) fn sprite_0_d_buzzblob(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = (self.sprite_slot_view(k).object_priority() & 0xf1)
                | SPRITE_0_D_BUZZBLOB_OBJ_PRIO
                    [usize::from((self.sprite_slot_view(k).delay_aux1() >> 1) & 3)];
            self.sprite_slot_view_mut(k).set_object_priority(value);
        }
        self.sprite_cukeman(k);
        self.buzz_blob_draw(k);
        let value = SPRITE_0_D_BUZZBLOB_LOCAL_GRAPHICS
            [usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 3)]
            + if self.sprite_slot_view(k).delay_aux1() != 0 {
                3
            } else {
                0
            };
        self.sprite_slot_view_mut(k).set_graphics(value);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.buzzblob_select_new_direction(k);
        }
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            self.sprite_move_xy(k);
        }
        let _ = self.sprite_check_tile_collision(k);
        self.sprite_bounce_off_wall(k);
        self.sprite_check_damage_to_and_from_link(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_08_Octorok(int k) {  // 86d377
    pub(super) fn sprite_08_octorok(&mut self, k: usize) {
        let mut j = self.sprite_slot_view(k).direction();
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            let value = SPRITE_08_OCTOROK_DIRECTIONS[usize::from(j)];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | SPRITE_08_OCTOROK_OAM_FLAGS[usize::from(j)]
            | if self.sprite_slot_view(k).graphics() == 7 {
                0x40
            } else {
                0
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.octorock_draw(k);
        let value = j;
        self.sprite_slot_view_mut(k).set_direction(value);

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_check_damage_to_and_from_link(k);
        if (self.sprite_slot_view(k).ai_state() & 1) == 0 {
            self.sprite_slot_view_mut(k).add_subtype2(1);
            let value = ((self.sprite_slot_view(k).subtype2() >> 3) & 3)
                | ((self.sprite_slot_view(k).direction() & 2) << 1);
            self.sprite_slot_view_mut(k).set_graphics(value);
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = if self.sprite_slot_view(k).sprite_type() == 8 {
                    60
                } else {
                    160
                };
                self.sprite_slot_view_mut(k).set_delay_main(value);
            } else {
                j = self.sprite_slot_view(k).direction();
                let value = SPRITE_08_OCTOROK_LOCAL_X_VELOCITIES[usize::from(j)] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_08_OCTOROK_LOCAL_Y_VELOCITIES[usize::from(j)] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_view_mut(k).xor_direction(1);
                }
            }
            return;
        }

        self.sprite_zero_velocity_xy(k);
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.sprite_slot_view_mut(k).add_ai_state(1);
            // ROM $86D416: `AND #$3F : ADC #$30` inherits the RNG routine's
            // final LSR carry (oracle sample 0x8D/C=1 wrote 0x3E at route
            // frame 71350).
            let value = self.get_random_number_with_carry().masked_adc(0x3f, 48);
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).delay_main() & 3;
            self.sprite_slot_view_mut(k).set_direction(value);
        } else {
            match self.sprite_slot_view(k).sprite_type() {
                8 => {
                    j = self.sprite_slot_view(k).delay_main();
                    if j == 28 {
                        self.octorok_fire_loogie(k);
                    }
                    let value = OCTOROK_SPIT_POSE_BY_DELAY[usize::from(j >> 3)];
                    self.sprite_slot_view_mut(k).set_c(value);
                }
                10 => {
                    j = self.sprite_slot_view(k).delay_main();
                    if j < 128 {
                        if (j & 15) == 0 {
                            let value = SPRITE_08_OCTOROK_NEXT_DIR
                                [usize::from(self.sprite_slot_view(k).direction())];
                            self.sprite_slot_view_mut(k).set_direction(value);
                        }
                        if (j & 15) == 8 {
                            self.octorok_fire_loogie(k);
                        }
                    }
                    let value = FOUR_WAY_OCTOROK_SPIT_POSE_BY_DELAY[usize::from(j >> 4)];
                    self.sprite_slot_view_mut(k).set_c(value);
                }
                _ => {}
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_02_StalfosHead(int k) {  // 86ddb7
    pub(super) fn sprite_02_stalfos_head(&mut self, k: usize) {
        let value = self.game_state.player.follower_link.lower_level_state();

        self.sprite_slot_view_mut(k).set_floor(value);
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            self.oam_allocate_from_region_c(8);
        }
        let j = usize::from((self.sprite_slot_view(k).subtype2() >> 3) & 3);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_02_STALFOS_HEAD_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = SPRITE_02_STALFOS_HEAD_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = 0x30;
        self.sprite_slot_view_mut(k).set_object_priority(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_zero_velocity_xy(k);
        }
        self.sprite_move_xy(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let (pt_x, pt_y) = if self.sprite_slot_view(k).delay_main() != 0 {
            if (self.sprite_slot_view(k).delay_main() & 1) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 16);
            (pt.x, pt.y)
        } else {
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) != 0 {
                return;
            }
            let pt = self.sprite_project_speed_towards_link(k, 16);
            (pt.x.wrapping_neg(), pt.y.wrapping_neg())
        };
        let dx = self.sprite_slot_view(k).x_velocity().wrapping_sub(pt_x);
        if dx != 0 {
            let value = self
                .sprite_slot_view(k)
                .x_velocity()
                .wrapping_add(if sign8(dx) { 1 } else { 0xff });
            self.sprite_slot_view_mut(k).set_x_velocity(value);
        }
        let dy = self.sprite_slot_view(k).y_velocity().wrapping_sub(pt_y);
        if dy != 0 {
            let value = self
                .sprite_slot_view(k)
                .y_velocity()
                .wrapping_add(if sign8(dy) { 1 } else { 0xff });
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_0C_OctorokStone(int k) {  // 86d5b9
    pub(super) fn sprite_0_c_octorok_stone(&mut self, k: usize) {
        if self.sprite_slot_view(k).state() == 6 {
            self.sprite_draw_octorok_stone_crumbling(k);
            if self.sprite_return_if_paused(k) {
                return;
            }
            if self.sprite_slot_view(k).delay_main() == 30 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
            }
        } else {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_link(k);
            self.sprite_move_xy(k);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
                && self.sprite_check_tile_collision(k) != 0
            {
                self.sprite_func3(k);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_0E_Snapdragon(int k) {  // 869c24
    pub(super) fn sprite_0_e_snapdragon(&mut self, k: usize) {
        let value = self.sprite_slot_view(k).b().wrapping_add(
            SPRITE_0_E_SNAPDRAGON_LOCAL_GRAPHICS[usize::from(self.sprite_slot_view(k).direction())],
        );

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.snap_dragon_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_b(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = SPRITE_0_E_SNAPDRAGON_DELAY
                        [usize::from((self.get_random_number() & 12) >> 2)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let new_a = self.sprite_slot_view(k).a().wrapping_sub(1);
                    let value = new_a;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if sign8(new_a) {
                        let value = 3;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = 96;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_slot_view_mut(k).add_c(1);
                        let value =
                            self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    } else {
                        let value = self.get_random_number() & 3;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                } else if (self.sprite_slot_view(k).delay_main() & 0x18) != 0 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_b(1);
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_view_mut(k).xor_direction(3);
                }
                let j = usize::from(
                    self.sprite_slot_view(k).direction()
                        + if self.sprite_slot_view(k).c() != 0 {
                            4
                        } else {
                            0
                        },
                );
                let value = SPRITE_0_E_SNAPDRAGON_LOCAL_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_0_E_SNAPDRAGON_LOCAL_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                self.sprite_move_z(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(4);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    if self.sprite_slot_view(k).delay_main() == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_c(value);
                        let value = 63;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = 20;
                        self.sprite_slot_view_mut(k).set_z_velocity(value);
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_18_MiniMoldorm(int k) {  // 869808
    pub(super) fn sprite_18_mini_moldorm(&mut self, k: usize) {
        self.moldorm_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_prep_mini_moldorm_bounce(k);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let j = usize::from(self.sprite_slot_view(k).direction());
        let value = SPRITE_18_MINI_MOLDORM_LOCAL_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_18_MINI_MOLDORM_LOCAL_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            if (self.get_random_number() & 1) != 0 {
                let value = self.sprite_slot_view(k).head_direction().wrapping_neg();
                self.sprite_slot_view_mut(k).set_head_direction(value);
            }
            let value =
                SPRITE_18_MINI_MOLDORM_NEXT_DIR[usize::from(self.sprite_slot_view(k).direction())];
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_g(1);
                    if self.sprite_slot_view(k).g() == 6 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_g(value);
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    } else {
                        let value = 1;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                    }
                    let value = (self.get_random_number() & 2).wrapping_sub(1);
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                    // ROM Sprite_18_MiniMoldorm $06:9881: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(0x1f, 0x20);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM Sprite_18_MiniMoldorm $06:9892: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
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
                    self.sprite_apply_speed_towards_link(k, 31);
                    let d = Self::sprite_convert_velocity_to_angle(
                        self.sprite_slot_view(k).x_velocity(),
                        self.sprite_slot_view(k).y_velocity(),
                    )
                    .wrapping_sub(self.sprite_slot_view(k).direction());
                    if d == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 48;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = self
                            .sprite_slot_view(k)
                            .direction()
                            .wrapping_add(if sign8(d) { 0xff } else { 1 })
                            & 0x0f;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_11_Hinox(int k) {  // 869f05
    pub(super) fn sprite_11_hinox(&mut self, k: usize) {
        self.hinox_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).f() != 0 {
            self.hinox_face_link(k);
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 48;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    if (self.get_random_number() & 3) == 0 {
                        let value = 2;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 64;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        self.sprite_slot_view_mut(k).add_c(1);
                        if self.sprite_slot_view(k).c() == 4 {
                            let value = 0;
                            self.sprite_slot_view_mut(k).set_c(value);
                            self.hinox_face_link(k);
                        } else {
                            let idx = usize::from(
                                self.sprite_slot_view(k).direction() * 2
                                    + (self.get_random_number() & 1),
                            );
                            self.hinox_set_direction(k, HINOX_RANDOM_DIRECTIONS[idx]);
                        }
                    }
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let new_a = self.sprite_slot_view(k).a().wrapping_sub(1);
                    let value = new_a;
                    self.sprite_slot_view_mut(k).set_a(value);
                    if sign8(new_a) {
                        let value = 11;
                        self.sprite_slot_view_mut(k).set_a(value);
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                    }
                    self.sprite_move_xy(k);
                    if self.sprite_check_tile_collision(k) == 0 {
                        let value = SPRITE_11_HINOX_WALK_GFX
                            [usize::from(self.sprite_slot_view(k).direction())]
                            + (self.sprite_slot_view(k).subtype2() & 1);
                        self.sprite_slot_view_mut(k).set_graphics(value);
                        return;
                    }
                }
                let value = 16;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    return;
                }
                if self.sprite_slot_view(k).delay_main() == 32 {
                    let mut info = SpriteSpawnInfo::default();
                    let j = self.sprite_spawn_dynamically(k, 0x4a, &mut info);
                    if j >= 0 {
                        let j = j as usize;
                        self.sprite_transmute_to_bomb(j);
                        let value = 64;
                        self.sprite_slot_view_mut(j).set_delay_aux1(value);
                        let i = usize::from(self.sprite_slot_view(k).direction());
                        self.sprite_set_x(
                            j,
                            info.r0_x
                                .wrapping_add_signed(i16::from(HINOX_BOMB_X_OFFSETS[i])),
                        );
                        self.sprite_set_y(
                            j,
                            info.r2_y
                                .wrapping_add_signed(i16::from(HINOX_BOMB_Y_OFFSETS[i])),
                        );
                        let value = HINOX_BOMB_X_VELOCITIES[i] as u8;
                        self.sprite_slot_view_mut(j).set_x_velocity(value);
                        let value = HINOX_BOMB_Y_VELOCITIES[i] as u8;
                        self.sprite_slot_view_mut(j).set_y_velocity(value);
                        let value = 40;
                        self.sprite_slot_view_mut(j).set_z_velocity(value);
                    }
                } else {
                    let i = usize::from(
                        self.sprite_slot_view(k).direction()
                            + if self.sprite_slot_view(k).delay_main() < 32 {
                                4
                            } else {
                                0
                            },
                    );
                    let value = SPRITE_11_HINOX_LOCAL_GRAPHICS[i];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_13_MiniHelmasaur(int k) {  // 86a409
    pub(super) fn sprite_13_mini_helmasaur(&mut self, k: usize) {
        if self.sprite_13_mini_helmasaur_through_subtype2_increment(k) {
            self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(k);
        }
    }

    pub(super) fn sprite_13_mini_helmasaur_through_subtype2_increment(&mut self, k: usize) -> bool {
        let j = usize::from(
            ((self.sprite_slot_view(k).subtype2() >> 2) & 1)
                | (self.sprite_slot_view(k).direction() << 1),
        );
        let value = SPRITE_13_MINI_HELMASAUR_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_13_MINI_HELMASAUR_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
            let mut x = self.sprite_slot_view(k).x_velocity();
            if sign8(x) {
                x = x.wrapping_neg();
            }
            let mut y = self.sprite_slot_view(k).y_velocity();
            if sign8(y) {
                y = y.wrapping_neg();
            }
            let value = if x >= y {
                self.sprite_slot_view(k).x_velocity() >> 7
            } else {
                (self.sprite_slot_view(k).y_velocity() >> 7) + 2
            };
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        self.sprite_draw_single_large(k);
        self.helmasaur_hard_hat_beetle_common_through_subtype2_increment(k)
    }

    // -----------------------------------------------------------------------
    // void Sprite_26_HardhatBeetle(int k) {  // 86a460
    pub(super) fn sprite_26_hardhat_beetle(&mut self, k: usize) {
        if self.sprite_26_hardhat_beetle_through_subtype2_increment(k) {
            self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(k);
        }
    }

    pub(super) fn sprite_26_hardhat_beetle_through_subtype2_increment(&mut self, k: usize) -> bool {
        let value = (self.sprite_slot_view(k).subtype2() >> 2) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.hard_hat_beetle_draw(k);
        self.helmasaur_hard_hat_beetle_common_through_subtype2_increment(k)
    }

    // -----------------------------------------------------------------------
    // void HelmasaurHardHatBeetleCommon(int k) {  // 86a46d
    pub(super) fn helmasaur_hard_hat_beetle_common(&mut self, k: usize) {
        if self.helmasaur_hard_hat_beetle_common_through_subtype2_increment(k) {
            self.helmasaur_hard_hat_beetle_common_after_subtype2_increment(k);
        }
    }

    fn helmasaur_hard_hat_beetle_common_through_subtype2_increment(&mut self, k: usize) -> bool {
        if self.sprite_return_if_inactive(k) {
            return false;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        true
    }

    pub(super) fn helmasaur_hard_hat_beetle_common_after_subtype2_increment(&mut self, k: usize) {
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if (self.sprite_slot_view(k).wall_collision() & 15) != 0 {
            if (self.sprite_slot_view(k).wall_collision() & 3) != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
        } else {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 31) == 0 {
            let pt = self.sprite_project_speed_towards_link(k, self.sprite_slot_view(k).a());
            let value = pt.y;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = pt.x;
            self.sprite_slot_view_mut(k).set_c(value);
        }
        if (((k as u8) ^ self.game_state.frame.frame_counter) & self.sprite_slot_view(k).ai_state())
            != 0
        {
            return;
        }
        let dy = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_sub(self.sprite_slot_view(k).b());
        let value = self
            .sprite_slot_view(k)
            .y_velocity()
            .wrapping_add(if sign8(dy) { 1 } else { 0xff });
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let dx = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_sub(self.sprite_slot_view(k).c());
        let value = self
            .sprite_slot_view(k)
            .x_velocity()
            .wrapping_add(if sign8(dx) { 1 } else { 0xff });
        self.sprite_slot_view_mut(k).set_x_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_15_Antifairy(int k) {  // 86a50c
    pub(super) fn sprite_15_antifairy(&mut self, k: usize) {
        self.sprite_draw_antfairy(k);
        self.sprite_15_antifairy_after_draw(k);
    }

    fn sprite_15_antifairy_after_draw(&mut self, k: usize) {
        if self.antifairy_after_draw_before_bounce(k) {
            self.sprite_bounce_from_tile_collision(k);
        }
    }

    pub(super) fn antifairy_before_bounce(&mut self, k: usize) -> bool {
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
            let t = i16::from(self.game_state.player.follower_link.magic_power()) - 8;
            if t < 0 {
                self.follower_link_state_mut().set_magic_power(0);
            } else {
                self.set_sound_effect_2(0x1d);
                self.follower_link_state_mut().set_magic_power(t as u8);
            }
        }
        self.sprite_move_xy(k);
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

        self.sprite_move_xy(k);
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
        self.sprite_check_damage_to_link(k);
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
        let Some((x, y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_plain_at_for_draw(
                oam,
                x.wrapping_add(xo as u16) as u8,
                y.wrapping_add(yo as u16) as u8,
                0x28,
                flags,
                2,
            );
            oam += 4;
        }
        self.sprite_correct_oam_entries_for_draw(k, 3, 0xff);

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
    // void Sprite_0B_Cucco(int k) {  // 86a5c2
    pub(super) fn sprite_0_b_cucco(&mut self, k: usize) {
        if self.sprite_slot_view(k).x_velocity() != 0 {
            let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
                | if sign8(self.sprite_slot_view(k).x_velocity()) {
                    0
                } else {
                    0x40
                };
            self.sprite_slot_view_mut(k).set_oam_flags(value);
        }

        self.sprite_draw_single_large(k);
        if self.sprite_slot_view(k).head_direction() != 0 {
            let value = 0x3d;
            self.sprite_slot_view_mut(k).set_sprite_type(value);
            self.sprite_prep_load_properties(k);
            self.sprite_slot_view_mut(k).add_subtype(1);
            let value = 48;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.set_sound_effect_1(21);
            let value = 21;
            self.sprite_slot_view_mut(k).set_ignore_projectile(value);
            return;
        }
        let mut helper_ordinal = 0;
        if self.sprite_slot_view(k).state() == 10 {
            let value = 3;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            if self.game_state.frame.submodule == 0 {
                if self.chicken_incr_subtype2_for_draw(k, 3, 0, CuccoSubtypeContinuation::State10) {
                    return;
                }
                self.cucco_draw_panic(k);
                if (self.game_state.frame.frame_counter & 0x0f) == 0 {
                    self.bawk_bawk_for_draw(k);
                }
                helper_ordinal = 1;
            }
        }
        self.complete_cucco_after_state10_branch(k, helper_ordinal);
    }

    /// Resume the state-10 Cucco immediately after its initial `subtype2 += 3`
    /// publication. This is the exact remaining C suffix; neither the slot
    /// timers/OAM prefix nor the three subtype stores are replayed.
    pub(super) fn complete_cucco_after_subtype_increments(
        &mut self,
        k: usize,
        completed: u8,
        total: u8,
        continuation: CuccoSubtypeContinuation,
    ) {
        assert!(completed <= total);
        for _ in completed..total {
            self.chicken_add_subtype2_for_draw(k, 1);
        }
        self.chicken_publish_graphics_for_draw(k);
        self.chicken_finish_subtype2_for_draw(k);
        self.complete_cucco_helper_caller(k, continuation);
    }

    /// Advance an already-suspended `Chicken_IncrSubtype2` call to a later
    /// source-observed increment without publishing graphics or running its
    /// caller suffix. This is used when consecutive live-owner host receipts
    /// refine the same semantic helper invocation.
    pub(super) fn advance_cucco_subtype_increment_checkpoint(
        &mut self,
        k: usize,
        completed: u8,
        refined_completed: u8,
        total: u8,
    ) {
        assert!(completed <= refined_completed);
        assert!(refined_completed <= total);
        for _ in completed..refined_completed {
            self.chicken_add_subtype2_for_draw(k, 1);
        }
    }

    /// Advance the same suspended helper through its remaining increments and
    /// graphics publication, stopping before `Sprite_ReturnIfLifted` and the
    /// caller-specific suffix.
    pub(super) fn advance_cucco_subtype_checkpoint_to_graphics(
        &mut self,
        k: usize,
        completed: u8,
        total: u8,
    ) {
        self.advance_cucco_subtype_increment_checkpoint(k, completed, total, total);
        self.chicken_publish_graphics_for_draw(k);
    }

    pub(super) fn complete_cucco_after_graphics_publication(
        &mut self,
        k: usize,
        continuation: CuccoSubtypeContinuation,
    ) {
        self.chicken_finish_subtype2_for_draw(k);
        self.complete_cucco_helper_caller(k, continuation);
    }

    fn complete_cucco_helper_caller(&mut self, k: usize, continuation: CuccoSubtypeContinuation) {
        match continuation {
            CuccoSubtypeContinuation::State10 => {
                self.cucco_draw_panic(k);
                if (self.game_state.frame.frame_counter & 0x0f) == 0 {
                    self.bawk_bawk_for_draw(k);
                }
                self.complete_cucco_after_state10_branch(k, 1);
            }
            CuccoSubtypeContinuation::Flee | CuccoSubtypeContinuation::CarriedLanding => {
                self.cucco_draw_panic(k);
            }
            CuccoSubtypeContinuation::ActiveC
            | CuccoSubtypeContinuation::Hopping
            | CuccoSubtypeContinuation::CarriedAirborne => {}
        }
    }

    fn complete_cucco_after_state10_branch(&mut self, k: usize, helper_ordinal: u8) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).c() != 0 {
            self.sprite_slot_view_mut(k).or_oam_flags(0x10);
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterActiveCuccoX {
                    slot,
                    helper_ordinal: expected_ordinal,
                }) if slot == k as u8 && expected_ordinal == helper_ordinal
            ) {
                assert_ne!(
                    self.sprite_slot_view(k).x_velocity(),
                    0,
                    "an active-Cucco X publication requires Sprite_MoveX to enter its source body",
                );
                self.sprite_move_x(k);
                return;
            }
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                    slot,
                    helper_ordinal: expected_ordinal,
                    y_low: None,
                    y_high: None,
                }) if slot == k as u8 && expected_ordinal == helper_ordinal
            ) {
                let (y_low, y_high) = self.sprite_move_xy_through_y_subpixel(k);
                self.sprite_main_cpu_boundary =
                    Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
                        slot: k as u8,
                        helper_ordinal,
                        y_low: Some(y_low),
                        y_high: Some(y_high),
                    });
                return;
            }
            self.sprite_move_xy(k);
            self.complete_active_cucco_after_movement(k, helper_ordinal);
        } else {
            let value = 255;
            self.sprite_slot_view_mut(k).set_health(value);
            if self.sprite_slot_view(k).b() >= 35 {
                self.cucco_summon_avenger(k);
            }
            if self.sprite_slot_view(k).f() != 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_f(value);
                if self.sprite_slot_view(k).b() < 35 {
                    self.sprite_slot_view_mut(k).add_b(1);
                    self.bawk_bawk_for_draw(k);
                }
                let value = 2;
                self.sprite_slot_view_mut(k).set_ai_state(value);
            }
            self.sprite_check_damage_from_link(k);
            match self.sprite_slot_view(k).ai_state() {
                0 => self.cucco_calm(k),
                1 => {
                    self.chicken_hopping(k, helper_ordinal);
                }
                2 => {
                    self.cucco_flee(k, helper_ordinal);
                }
                3 => {
                    self.cucco_carried(k, helper_ordinal);
                }
                _ => {}
            }
        }
    }

    fn complete_active_cucco_after_movement(&mut self, k: usize, helper_ordinal: u8) {
        self.advance_active_cucco_after_movement_to_helper(k);
        self.chicken_incr_subtype2_for_draw(
            k,
            4,
            helper_ordinal,
            CuccoSubtypeContinuation::ActiveC,
        );
    }

    pub(super) fn complete_active_cucco_after_x(&mut self, k: usize, helper_ordinal: u8) {
        self.sprite_move_y(k);
        self.complete_active_cucco_after_movement(k, helper_ordinal);
    }

    pub(super) fn advance_active_cucco_x_to_y_subpixel(&mut self, k: usize) -> (u8, u8) {
        self.sprite_move_y_through_subpixel(k)
    }

    pub(super) fn advance_active_cucco_x_to_helper(&mut self, k: usize) {
        self.sprite_move_y(k);
        self.advance_active_cucco_after_movement_to_helper(k);
    }

    fn advance_active_cucco_after_movement_to_helper(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).set_z(12);
        self.sprite_slot_view_mut(k).set_ignore_projectile(12);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
            self.sprite_check_damage_to_link(k);
        }
    }

    pub(super) fn complete_active_cucco_after_y_subpixel(
        &mut self,
        k: usize,
        helper_ordinal: u8,
        y_low: u8,
        y_high: u8,
    ) {
        self.complete_sprite_move_y_after_subpixel(k, y_low, y_high);
        self.complete_active_cucco_after_movement(k, helper_ordinal);
    }

    pub(super) fn advance_active_cucco_y_subpixel_to_helper(
        &mut self,
        k: usize,
        y_low: u8,
        y_high: u8,
    ) {
        self.complete_sprite_move_y_after_subpixel(k, y_low, y_high);
        self.advance_active_cucco_after_movement_to_helper(k);
    }

    // Local duplicate of private dungeon NPC helper used by the original
    // top-level Cucco actor.
    pub(super) fn chicken_incr_subtype2_for_draw(
        &mut self,
        k: usize,
        increments: u8,
        helper_ordinal: u8,
        continuation: CuccoSubtypeContinuation,
    ) -> bool {
        if let Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
            slot,
            helper_ordinal: expected_ordinal,
            completed,
            total: 0,
            continuation: None,
        }) = self.sprite_main_cpu_boundary
        {
            if slot == k as u8 && expected_ordinal == helper_ordinal {
                assert!(
                    completed <= increments,
                    "source Cucco helper published increment {completed}, but native call has only {increments}",
                );
            }
        }
        for completed in 1..=increments {
            self.chicken_add_subtype2_for_draw(k, 1);
            if matches!(
                self.sprite_main_cpu_boundary,
                Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                    slot,
                    helper_ordinal: expected_ordinal,
                    completed: expected_completed,
                    total: 0,
                    continuation: None,
                }) if slot == k as u8
                    && expected_ordinal == helper_ordinal
                    && expected_completed == completed
            ) {
                self.sprite_main_cpu_boundary =
                    Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
                        slot: k as u8,
                        helper_ordinal,
                        completed,
                        total: increments,
                        continuation: Some(continuation),
                    });
                return true;
            }
        }
        self.chicken_publish_graphics_for_draw(k);
        if matches!(
            self.sprite_main_cpu_boundary,
            Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal: expected_ordinal,
                continuation: None,
            }) if slot == k as u8 && expected_ordinal == helper_ordinal
        ) {
            self.sprite_main_cpu_boundary =
                Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
                    slot: k as u8,
                    helper_ordinal,
                    continuation: Some(continuation),
                });
            return true;
        }
        self.chicken_finish_subtype2_for_draw(k);
        false
    }

    pub(super) fn chicken_add_subtype2_for_draw(&mut self, k: usize, j: u8) {
        self.sprite_slot_view_mut(k).add_subtype2(j);
    }

    pub(super) fn chicken_publish_graphics_for_draw(&mut self, k: usize) {
        let value = (self.sprite_slot_view(k).subtype2() >> 4) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    pub(super) fn chicken_finish_subtype2_for_draw(&mut self, k: usize) {
        self.sprite_return_if_lifted(k);
    }

    // Local duplicate of private dungeon NPC helper used by the original
    // top-level Cucco actor.
    fn bawk_bawk_for_draw(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x30);
    }

    // -----------------------------------------------------------------------
    // void Sprite_01_Vulture_bounce(int k) {  // 869473
    //   Dormant vulture that begins circling and tracking Link.
    // }
    pub(super) fn sprite_01_vulture_bounce(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        self.vulture_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).subtype2() == 160 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                let value =
                    VULTURE_GRAPHICS[usize::from((self.game_state.frame.frame_counter >> 1) & 3)];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).delay_main() != 0 {
                    self.sprite_slot_view_mut(k).add_z(1);
                    return;
                }
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 1) != 0 {
                    return;
                }
                let pt =
                    self.sprite_project_speed_towards_link(k, ((k & 0x0f) as u8).wrapping_add(24));
                let value = pt.y.wrapping_neg();
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = pt.x;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                if pt.xdiff.wrapping_add(0x28) < 0x50 && pt.ydiff.wrapping_add(0x28) < 0x50 {
                    return;
                }
                let value = self
                    .sprite_slot_view(k)
                    .y_velocity()
                    .wrapping_add(((pt.y as i8) >> 2) as u8);
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = self
                    .sprite_slot_view(k)
                    .x_velocity()
                    .wrapping_add(((pt.x as i8) >> 2) as u8);
                self.sprite_slot_view_mut(k).set_x_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_27_Deadrock(int k) {  // 86948a
    //   Deadrock walking/petrified state machine.
    // }
    pub(super) fn sprite_27_deadrock(&mut self, k: usize) {
        let j = if if self.sprite_slot_view(k).delay_aux2() != 0 {
            (self.sprite_slot_view(k).delay_aux2() & 4) != 0
        } else {
            self.sprite_slot_view(k).ai_state() != 2
        } {
            self.sprite_slot_view(k).a()
        } else {
            8
        };
        let j = usize::from(j);
        let value = DEAD_ROCK_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | DEAD_ROCK_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).f() == 0
            && (self.sprite_check_damage_from_link(k) & PLAYER_DAMAGE_CARRY_MASK_DRAW) != 0
            && self.game_state.system_signals.sound_effect_1() == 0
        {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0b);
        }
        if self.sprite_check_damage_to_link_same_layer(k) {
            self.sprite_nullify_hookshot_drag();
            self.sprite_repel_dash();
        }
        if self.sprite_slot_view(k).f() == 14 {
            let value = 2;
            self.sprite_slot_view_mut(k).set_ai_state(value);
            let value = 255;
            self.sprite_slot_view_mut(k).set_delay_aux1(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_aux2(value);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).and_flags2(!0x80);
                    self.sprite_slot_view_mut(k).and_deflection_bits(!4);
                    self.sprite_slot_view_mut(k).and_flags3(!0x40);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    // ROM Sprite_27_Deadrock $06:9524: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(31, 32);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let dir = if self.sprite_slot_view(k).b().wrapping_add(1) == 4 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_b(value);
                        self.sprite_direction_to_face_link(k, None)
                    } else {
                        self.sprite_slot_view_mut(k).add_b(1);
                        self.get_random_number() & 3
                    };
                    self.deadrock_set_dir(k, dir, &DEAD_ROCK_X_VELOCITIES, &DEAD_ROCK_Y_VELOCITIES);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.sprite_move_xy(k);
                    if self.sprite_check_tile_collision(k) != 0 {
                        let dir = self.sprite_slot_view(k).direction() ^ 1;
                        self.deadrock_set_dir(
                            k,
                            dir,
                            &DEAD_ROCK_X_VELOCITIES,
                            &DEAD_ROCK_Y_VELOCITIES,
                        );
                    } else {
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        let value = (self.sprite_slot_view(k).direction() << 1)
                            | ((self.sprite_slot_view(k).subtype2() >> 2) & 1);
                        self.sprite_slot_view_mut(k).set_a(value);
                    }
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).or_flags2(0x80);
                self.sprite_slot_view_mut(k).or_deflection_bits(4);
                self.sprite_slot_view_mut(k).or_flags3(0x40);
                if (self.game_state.frame.frame_counter & 1) == 0 {
                    if self.sprite_slot_view(k).delay_aux1() == 0 {
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 16;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else if self.sprite_slot_view(k).delay_aux1() == 0x20 {
                        let value = 0x40;
                        self.sprite_slot_view_mut(k).set_delay_aux2(value);
                    }
                } else {
                    let value = self.sprite_slot_view(k).delay_aux1().wrapping_add(1);
                    self.sprite_slot_view_mut(k).set_delay_aux1(value);
                }
            }
            _ => {}
        }
    }

    fn deadrock_set_dir(&mut self, k: usize, dir: u8, xvel: &[u8; 4], yvel: &[u8; 4]) {
        let value = dir;
        self.sprite_slot_view_mut(k).set_direction(value);
        let idx = usize::from(dir);
        let value = xvel[idx];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = yvel[idx];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_20_Sluggula(int k) {  // 8695d9
    pub(super) fn sprite_20_sluggula(&mut self, k: usize) {
        let mut j = usize::from(
            (self.sprite_slot_view(k).direction() << 1)
                | ((self.sprite_slot_view(k).subtype2() & 8) >> 3),
        );
        let value = SPRITE_20_SLUGGULA_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 191) | SPRITE_20_SLUGGULA_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    // ROM Sprite_20_Sluggula $06:9623: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                    let value = self.get_random_number_with_carry().masked_adc(31, 32);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    j = usize::from(self.sprite_slot_view(k).delay_main() & 3);
                    let value = j as u8;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let value = SPRITE_20_SLUGGULA_XYVEL[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_20_SLUGGULA_XYVEL[j + 2] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                } else if self.sprite_slot_view(k).delay_main() == 16
                    && (self.get_random_number() & 1) == 0
                {
                    self.sluggula_drop_bomb(k);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) == 0 {
                    return;
                }
                self.sprite_slot_view_mut(k).xor_direction(1);
                j = usize::from(self.sprite_slot_view(k).direction());
                let value = SPRITE_20_SLUGGULA_XYVEL[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = SPRITE_20_SLUGGULA_XYVEL[j + 2] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_22_Ropa(int k) {  // 869e1f
    pub(super) fn sprite_22_ropa(&mut self, k: usize) {
        self.ropa_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 3) & 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_apply_speed_towards_link(k, 16);
                    // ROM $86:9E6C: `JSL GetRandomNumber : AND #$0F : ADC #$14`
                    // — the ADC consumes GetRandomNumber's carry-out (the C
                    // port drops it). Route boundary 21 (frame 567894): the
                    // hop left at $1C, the carry-less port at $1B.
                    let value = self.get_random_number_with_carry().masked_adc(15, 20);
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                }
            }
            1 => {
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_zero_velocity_xy(k);
                }
                self.sprite_move_z(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_19_Poe(int k) {  // 869688
    pub(super) fn sprite_19_poe(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).x_velocity() >> 7);
        let value = j as u8;
        self.sprite_slot_view_mut(k).set_direction(value);
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_19_POE_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        if self.sprite_slot_view(k).e() == 0 {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
        }
        self.poe_draw(k);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_view_mut(k).subtract_flags2(1);
        self.sprite_draw_single_large(k);
        self.sprite_slot_view_mut(k).add_flags2(1);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.sprite_slot_view(k).e() != 0 {
            self.sprite_slot_view_mut(k).add_z(1);
            if self.sprite_slot_view(k).z() == 12 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_e(value);
            }
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        self.sprite_move_xy(k);
        if (self.game_state.frame.frame_counter & 1) == 0 {
            j = usize::from(self.sprite_slot_view(k).g() & 1);
            self.sprite_slot_view_mut(k)
                .add_z_velocity(SPRITE_19_POE_ACCEL[j] as u8);
            if self.sprite_slot_view(k).z_velocity() == CHOMP_Z_VELOCITY_TARGETS[j] as u8 {
                self.sprite_slot_view_mut(k).add_g(1);
            }
        }
        self.sprite_move_z(k);
        let value = 0;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    if (self.get_random_number() & 12) == 0 {
                        let value = self.sprite_is_below_link(k).a;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    } else {
                        let value = self.get_random_number() & 1;
                        self.sprite_slot_view_mut(k).set_head_direction(value);
                    }
                }
            }
            1 => {
                if (self.game_state.frame.frame_counter & 1) == 0 {
                    j = usize::from(
                        (self.sprite_slot_view(k).anim_clock() & 1)
                            + self.game_state.world.region.dark_world_region_index() * 2,
                    );
                    let value = self
                        .sprite_slot_view(k)
                        .x_velocity()
                        .wrapping_add(SPRITE_19_POE_ACCEL[j] as u8);
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    if self.sprite_slot_view(k).x_velocity() == CHOMP_X_VELOCITY_TARGETS[j] as u8 {
                        let value = self.sprite_slot_view(k).anim_clock().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_anim_clock(value);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        // ROM Sprite_19_Poe $06:9769: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                        let value = self.get_random_number_with_carry().masked_adc(31, 16);
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    }
                }
                let value = SPRITE_19_POE_LOCAL_Y_VELOCITIES
                    [usize::from(self.sprite_slot_view(k).head_direction())]
                    as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_2E_FluteKid(int k) {  // 86af3b
    pub(super) fn sprite_2_e_flute_kid(&mut self, k: usize) {
        match self.sprite_slot_view(k).head_direction() {
            0 => match self.sprite_slot_view(k).subtype2() {
                0 => self.flute_kid_human(k),
                1 => self.sprite_flute_kid_stumpy(k),
                _ => {}
            },
            1 => self.sprite_flute_kid_quaver(k),
            _ => {}
        }
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
    // void Sprite_1F_SickKid(int k) {  // 86b94c
    pub(super) fn sprite_1_f_sick_kid(&mut self, k: usize) {
        self.bug_net_kid_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_check_if_link_is_busy()
                    || !self.sprite_check_damage_to_link_same_layer(k)
                {
                    return;
                }
                let bottles = self.game_state.inventory.items.bottle(0)
                    | self.game_state.inventory.items.bottle(1)
                    | self.game_state.inventory.items.bottle(2)
                    | self.game_state.inventory.items.bottle(3);
                if bottles < 2 {
                    self.sprite_show_solicited_message(k, 0x104);
                } else {
                    self.follower_link_state_mut().increment_immobilized_flag();
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() != 0 {
                    return;
                }
                let j = usize::from(self.sprite_slot_view(k).a());
                if SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS[j] >= 0 {
                    let value = SPRITE_1_F_SICK_KID_LOCAL_GRAPHICS[j] as u8;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    let value = SPRITE_1_F_SICK_KID_DELAY[j];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_a(1);
                } else {
                    self.sprite_show_message_unconditional(0x105);
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            2 => {
                self.follower_link_state_mut().set_item_receipt_method(0);
                if self
                    .link_receive_item_from(
                        0x21,
                        0,
                        ItemReceiptCaller::SpriteMain {
                            sprite_slot: k as u8,
                            suffix: SpriteMainItemReceiptSuffix::SickKid,
                        },
                    )
                    .is_suspended()
                {
                    return;
                }
                self.complete_sick_kid_item_receipt(k);
            }
            3 => {
                let value = 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_show_solicited_message(k, 0x106);
            }
            _ => {}
        }
    }

    /// Source suffix after SickKid's synchronous `Link_ReceiveItem` call.
    ///
    /// ROM $06:b9d0 restores X, increments the sprite AI state, then clears
    /// Link's immobilized flag. Keeping the suffix typed lets a live timing
    /// authority suspend the decompressor without publishing either write.
    pub(super) fn complete_sick_kid_item_receipt(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).increment_ai_state();
        self.follower_link_state_mut().clear_immobilized();
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
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
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
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
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
            self.open_gargoyles_domain_for_draw();
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
        // C Sprite_2D_TelepathicTile is an assert(0) sprite slot.
        assert!(false);
    }

    // -----------------------------------------------------------------------
    // void Sprite_38_EyeStatue(int k) {  // 86c03f
    pub(super) fn sprite_38_eye_statue(&mut self, k: usize) {
        if self.sprite_slot_view(k).b() == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_direction_to_face_link(k, None) == 2
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
    // void Sprite_6D_Rat(int k) {  // 85a8b0
    pub(super) fn sprite_6_d_rat(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).a());
        let value = RAT_GRAPHICS_BY_ANIM_STATE[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | RAT_OAM_FLIP_BY_ANIM_STATE[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                if !self.game_state.world.region.is_in_dark_world() {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x17);
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 80;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            j = usize::from(self.sprite_slot_view(k).direction());
            if self.sprite_slot_view(k).wall_collision() != 0 {
                let value = RAT_WALL_TURN_DIRECTIONS[j];
                self.sprite_slot_view_mut(k).set_direction(value);
                j = usize::from(self.sprite_slot_view(k).direction());
            }
            let value = SPRITE_6_D_RAT_LOCAL_X_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = SPRITE_6_D_RAT_LOCAL_Y_VELOCITIES[j] as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            let value = RAT_RUN_ANIM_STATES[usize::from(
                self.sprite_slot_view(k).direction() * 2
                    + ((self.game_state.frame.frame_counter >> 2) & 1),
            )];
            self.sprite_slot_view_mut(k).set_a(value);
        } else {
            self.sprite_zero_velocity_xy(k);
            if self.sprite_slot_view(k).delay_main() == 0 {
                let random = self.get_random_number_with_carry();
                let a = random.value();
                let value = a & 3;
                self.sprite_slot_view_mut(k).set_direction(value);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                let value = rat_random_run_delay(random);
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            let value = RAT_IDLE_ANIM_STATES[usize::from(
                self.sprite_slot_view(k).direction() * 2
                    + ((self.game_state.frame.frame_counter >> 3) & 1),
            )];
            self.sprite_slot_view_mut(k).set_a(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6E_Rope(int k) {  // 85a973
    pub(super) fn sprite_6_e_rope(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).a());
        let value = ROPE_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | ROPE_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).e() != 0 {
            let oam = self.game_state.oam.current_pointer_usize();
            self.oam_state_mut().or_entry_flags(oam, 0x30);

            let old_z = self.sprite_slot_view(k).z();
            self.sprite_move_z(k);
            if !sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0xc0)) {
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
            }
            if !sign8(old_z ^ self.sprite_slot_view(k).z()) || !sign8(self.sprite_slot_view(k).z())
            {
                return;
            }
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_e(value);
            self.sprite_slot_view_mut(k).and_flags3(!0x10);
        } else {
            let value = 0;
            self.sprite_slot_view_mut(k).set_flags2(value);
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_move_xy(k);
            let _ = self.sprite_check_tile_collision(k);
            if self.sprite_slot_view(k).ai_state() != 0 {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                j = usize::from(self.sprite_slot_view(k).direction());
                if self.sprite_slot_view(k).wall_collision() != 0 {
                    let value = ROPE_IDLE_GFX_BY_DIRECTION[j];
                    self.sprite_slot_view_mut(k).set_direction(value);
                    j = usize::from(self.sprite_slot_view(k).direction());
                }

                j += usize::from(self.sprite_slot_view(k).g());
                let value = ROPE_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = ROPE_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);

                let mut i = self.game_state.frame.frame_counter;
                if j < 4 {
                    i >>= 1;
                }

                let value = ROPE_FAST_GFX_BY_DIRECTION
                    [usize::from(self.sprite_slot_view(k).direction() * 2 + ((i >> 1) & 1))];

                self.sprite_slot_view_mut(k).set_a(value);
            } else {
                self.sprite_zero_velocity_xy(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_g(value);
                    let random = self.get_random_number_with_carry();
                    let a = random.value();
                    let value = a & 3;
                    self.sprite_slot_view_mut(k).set_direction(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = rope_random_run_delay(random);
                    self.sprite_slot_view_mut(k).set_delay_main(value);

                    let mut pt = PointU8 { x: 0, y: 0 };
                    let dir = self.sprite_direction_to_face_link(k, Some(&mut pt));
                    if pt.y.wrapping_add(0x10) < 0x20 || pt.x.wrapping_add(0x18) < 0x20 {
                        let value = 4;
                        self.sprite_slot_view_mut(k).set_g(value);
                        let value = dir;
                        self.sprite_slot_view_mut(k).set_direction(value);
                    }
                }
                let value = ROPE_FAST_GFX_BY_DIRECTION[usize::from(
                    self.sprite_slot_view(k).direction() * 2
                        + ((self.game_state.frame.frame_counter >> 3) & 1),
                )];
                self.sprite_slot_view_mut(k).set_a(value);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_6F_Keese(int k) {  // 85aa8b
    pub(super) fn sprite_6_f_keese(&mut self, k: usize) {
        self.sprite_slot_view_mut(k).or_object_priority(0x30);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).ai_state() != 0 {
            if self.sprite_slot_view(k).delay_main() == 0 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 64;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_zero_velocity_xy(k);
            } else {
                if (self.sprite_slot_view(k).delay_main() & 7) == 0 {
                    let value = self.sprite_slot_view(k).a().wrapping_add(
                        KEESE_ORBIT_DIRECTION_STEPS[usize::from(self.sprite_slot_view(k).b() & 1)]
                            as u8,
                    );
                    self.sprite_slot_view_mut(k).set_a(value);
                    if (self.get_random_number() & 3) == 0 {
                        self.sprite_slot_view_mut(k).add_b(1);
                    }
                }
                let j = usize::from(self.sprite_slot_view(k).a() & 0x0f);
                let value = KEESE_ATTACK_X_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_x_velocity(value);
                let value = KEESE_ATTACK_Y_VELOCITIES[j] as u8;
                self.sprite_slot_view_mut(k).set_y_velocity(value);
                let value = ((self.game_state.frame.frame_counter >> 2) & 1) + 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
        } else {
            if ((((k as u8) ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(k).delay_main())
                != 0
            {
                return;
            }

            let mut pt = PointU8 { x: 0, y: 0 };
            let dir = self.sprite_direction_to_face_link(k, Some(&mut pt));
            if pt.y.wrapping_add(0x28) >= 0x50 || pt.x.wrapping_add(0x28) >= 0x50 {
                return;
            }
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x1e);
            self.sprite_slot_view_mut(k).add_ai_state(1);
            let value = 64;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = 64;
            self.sprite_slot_view_mut(k).set_b(value);
            let value = KEESE_ATTACK_START_PHASES[usize::from(dir)] as u8;
            self.sprite_slot_view_mut(k).set_a(value);
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
        assert!(false);
    }

    // -----------------------------------------------------------------------
    // void Sprite_Cannonball(int k) {  // 85ab54
    pub(super) fn sprite_cannonball(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).delay_main() == 30 {
            self.sprite_spawn_poof_garnish(k);
        } else if self.sprite_slot_view(k).delay_main() == 0
            && self.sprite_check_tile_collision(k) != 0
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
    // void Sprite_6C_MirrorPortal(int k) {  // 85af75
    pub(super) fn sprite_6_c_mirror_portal(&mut self, k: usize) {
        if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        } else {
            if self.game_state.world.location.overworld_screen_index() >= 0x80 {
                return;
            }

            if self.game_state.frame.submodule != 0x23
                && self.game_state.sprites.system.chr_halfslot_state() < 3
            {
                self.sprite_draw_single_large(k);
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let j = usize::from((self.game_state.frame.frame_counter >> 2) & 3);
            let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | WARP_VORTEX_FLAGS[j];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            if self.sprite_check_if_link_is_busy() {
                return;
            }
            if self.sprite_check_damage_to_link_same_layer(k) {
                if self.sprite_slot_view(k).a() != 0
                    && (self
                        .game_state
                        .player
                        .follower_link
                        .sprite_damage_disable_timer()
                        | self.game_state.player.follower_link.blink_countdown())
                        == 0
                    && !self.game_state.player.follower_link.is_immobilized()
                {
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
                let value = 1;
                self.sprite_slot_view_mut(k).set_a(value);
            }
        }
        self.sprite_slot_view_mut(k).add_b(1);
        if self.sprite_slot_view(k).b() == 0 {
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
        }
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
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
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
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_check_damage_to_link_same_layer(k) {
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
        let _ = self.sprite_check_tile_collision(k);
        self.sprite_move_xy(k);
        self.sprite_check_damage_to_link(k);

        let mut pt = PointU8 { x: 0, y: 0 };
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 0x0f) == 0 {
            let value = self.sprite_direction_to_face_link(k, Some(&mut pt));
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
    // void Sprite_5D_Roller_VerticalDownFirst(int k) {  // 858dde
    pub(super) fn sprite_5_d_roller_vertical_down_first(&mut self, k: usize) {
        let value = ((self.sprite_slot_view(k).subtype2() >> 1) & 1)
            | (self.sprite_slot_view(k).direction() & 2);

        self.sprite_slot_view_mut(k).set_graphics(value);
        self.spike_roller_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
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
        self.sprite_move_xy(k);
        self.sprite_slot_view_mut(k).add_subtype2(1);
    }

    // -----------------------------------------------------------------------
    // void Sprite_50_Cannonball(int k) {  // 85b648
    pub(super) fn sprite_50_cannonball(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_draw_single_large(k);
        } else {
            self.sprite_draw_big_cannonball(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 2) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_move_xy(k);
        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() == 1 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.sprite_slot_view(k).delay_aux2() == 0 && self.sprite_check_tile_collision(k) != 0 {
            let value = 16;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_51_ArmosStatue(int k) {  // 85b703
    pub(super) fn sprite_51_armos_statue(&mut self, k: usize) {
        self.armos_draw(k);
        if self.sprite_slot_view(k).f() != 0 {
            self.sprite_zero_velocity_xy(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_z(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 0;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            self.sprite_zero_velocity_xy(k);
        }
        if self.sprite_slot_view(k).ai_state() == 0 {
            self.sprite_slot_view_mut(k).or_flags3(0x40);
            if self.sprite_slot_view(k).delay_main() == 1 {
                self.sprite_slot_view_mut(k).and_flags3(!0x40);
                self.sprite_slot_view_mut(k).add_ai_state(1);
                self.sprite_slot_view_mut(k).and_flags2(!0x80);
                self.sprite_slot_view_mut(k).and_flags3(!0x40);
                let value = 0x0b;
                self.sprite_slot_view_mut(k).set_oam_flags(value);
            } else {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                        .wrapping_add(31)
                        < 62
                    && self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(8)
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                        .wrapping_add(48)
                        < 88
                    && self.sprite_slot_view(k).delay_main() == 0
                {
                    let value = 48;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x22);
                }
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                }
                if self.sprite_slot_view(k).delay_main() != 0 {
                    let value = (self.sprite_slot_view(k).delay_main() >> 1) & 0x0e;
                    self.sprite_slot_view_mut(k).xor_oam_flags(value);
                }
            }
        } else {
            self.sprite_check_damage_to_and_from_link(k);
            if self.sprite_return_if_recoiling(k) {
                return;
            }
            self.sprite_move_xy(k);
            let _ = self.sprite_check_tile_collision(k);
            if (self.sprite_slot_view(k).delay_main() | self.sprite_slot_view(k).z()) == 0 {
                let value = 8;
                self.sprite_slot_view_mut(k).set_delay_main(value);
                let value = 16;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
                self.sprite_apply_speed_towards_link(k, 12);
            }
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_4E_Popo(int k) {  // 85b80a
    pub(super) fn sprite_4_e_popo(&mut self, k: usize) {
        self.bot_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> 4) & 3;
        self.sprite_slot_view_mut(k).set_a(value);
        self.sprite_check_damage_to_and_from_link(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 105;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    // ROM $85B84B: `AND #$3F : ADC #$80` inherits the RNG
                    // routine's final LSR carry (oracle sample 0x43/C=1 wrote
                    // 0x84 at route frame 84855).
                    let value = self.get_random_number_with_carry().masked_adc(63, 128);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let j = usize::from(self.get_random_number() & 15);
                    let value = ((i16::from(CUCCO_CALM_CIRCLE_X_VELOCITIES[j])) << 2) as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = ((i16::from(CUCCO_CALM_CIRCLE_Y_VELOCITIES[j])) << 2) as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            2 => {
                self.sprite_slot_view_mut(k).add_subtype2(1);
                if self.sprite_slot_view(k).delay_main() != 0 {
                    if (((k as u8) ^ self.game_state.frame.frame_counter)
                        & self.sprite_slot_view(k).b())
                        != 0
                    {
                        let _ = self.sprite_check_tile_collision(k);
                        return;
                    }
                    self.sprite_move_xy(k);
                    if self.sprite_slot_view(k).wall_collision() == 0 {
                        let _ = self.sprite_check_tile_collision(k);
                        return;
                    }
                }
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                let value = 80;
                self.sprite_slot_view_mut(k).set_delay_main(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_4D_Toppo(int k) {  // 85ba85
    pub(super) fn sprite_4_d_toppo(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.sprite_slot_view_mut(k).or_object_priority(0x30);
            self.toppo_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    let j = usize::from(self.get_random_number() & 3);
                    let x = u16::from(self.sprite_slot_view(k).a())
                        | (u16::from(self.sprite_slot_view(k).b()) << 8);
                    let y = u16::from(self.sprite_slot_view(k).c())
                        | (u16::from(self.sprite_slot_view(k).head_direction()) << 8);
                    self.sprite_set_x(k, x.wrapping_add(SPRITE_4_D_TOPPO_X_OFFS[j] as i16 as u16));
                    self.sprite_set_y(k, y.wrapping_add(SPRITE_4_D_TOPPO_Y_OFFS[j] as i16 as u16));
                }
            }
            1 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = (self.sprite_slot_view(k).delay_main() >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    self.toppo_verify_tile(k);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 64;
                    self.sprite_slot_view_mut(k).set_z_velocity(value);
                }
                let value = 2;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.toppo_verify_tile(k);
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_z(k);
                self.sprite_slot_view_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot_view(k).z()) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_z(value);
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.toppo_verify_tile(k);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 32;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = (self.sprite_slot_view(k).delay_main() >> 2) & 1;
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
                self.toppo_verify_tile(k);
            }
            5 => self.toppo_flustered(k),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_VerifyTile(int k) {  // 85bb72
    pub(super) fn toppo_verify_tile(&mut self, k: usize) {
        let mut x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        if self.GetTileAttribute(0, &mut x, y) != 0x40 {
            let value = 5;
            self.sprite_slot_view_mut(k).set_ai_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_Draw(int k) {  // 85bbff
    pub(super) fn toppo_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        let ybase = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        for i in (0..=2).rev() {
            let j = i + g * 3;
            let big = TOPPO_DRAW_BIG[j];
            let mut flags = TOPPO_DRAW_FLAGS[j] | info_flags;
            if big == 0 {
                flags = (flags & !0x0f) | 2;
            }
            let base_y = if big != 0 { info_y } else { ybase };
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(TOPPO_DRAW_X_OFFSETS[j] as i16 as u16),
                base_y.wrapping_add(TOPPO_DRAW_Y_OFFSETS[j] as i16 as u16),
                TOPPO_DRAW_CHARS[j],
                flags,
                big,
            );
            oam += 4;
        }
    }

    // -----------------------------------------------------------------------
    // void Toppo_Flustered(int k) {  // 9df3d4
    pub(super) fn toppo_flustered(&mut self, k: usize) {
        let value = 130;
        self.sprite_slot_view_mut(k).set_flags2(value);
        let value = 130;
        self.sprite_slot_view_mut(k).set_ignore_projectile(value);
        let value = 73;
        self.sprite_slot_view_mut(k).set_flags3(value);
        if self.sprite_slot_view(k).subtype() == 0 {
            if self.sprite_check_damage_to_link(k) {
                self.dialogue_message_index_mut().set_value(0x174);
                self.sprite_show_message_minimal_c();
                let value = 1;
                self.sprite_slot_view_mut(k).set_subtype(value);
            }
        } else if self.sprite_slot_view(k).subtype() < 16 {
            self.sprite_slot_view_mut(k).add_subtype(1);
        } else if self.sprite_slot_view(k).subtype() == 16 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_flags5(value);
            let value = 6;
            self.sprite_slot_view_mut(k).set_state(value);
            let value = 15;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).add_flags2(4);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x4d, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.force_prize_drop(j, 6, 6);
            }
            self.sprite_slot_view_mut(k).add_subtype(1);
        }
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = ((self.sprite_slot_view(k).subtype2() & 4) >> 2) + 3;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_4B_GreenKnifeGuard(int k) {  // 85bca2
    pub(super) fn sprite_4_b_green_knife_guard(&mut self, k: usize) {
        let value = RECRUIT_GRAPHICS[usize::from(
            self.sprite_slot_view(k)
                .direction()
                .wrapping_add((self.sprite_slot_view(k).subtype2() >> 1) & 4),
        )];
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.recruit_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.green_knife_guard_moving(k);
            return;
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }

        // ROM $05:bccd calls the RNG, then executes `AND #$3f; ADC #$30`
        // without clearing carry between them.
        let value = self.get_random_number_with_carry().masked_adc(0x3f, 0x30);

        self.sprite_slot_view_mut(k).set_delay_main(value);
        self.sprite_slot_view_mut(k).add_ai_state(1);
        let value = self.sprite_slot_view(k).head_direction();
        self.sprite_slot_view_mut(k).set_direction(value);
        let mut pt = PointU8 { x: 0, y: 0 };
        let mut j = self.sprite_slot_view(k).direction();
        if j == self.sprite_direction_to_face_link(k, Some(&mut pt))
            && (pt.x.wrapping_add(0x10) < 0x20 || pt.y.wrapping_add(0x10) < 0x20)
        {
            j = j.wrapping_add(4);
            let value = 128;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        let value = RECRUIT_X_VELOCITIES[usize::from(j)];
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = RECRUIT_Y_VELOCITIES[usize::from(j)];
        self.sprite_slot_view_mut(k).set_y_velocity(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_4C_Geldman(int k) {  // 85b8b3
    pub(super) fn sprite_4_c_geldman(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() < 2 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        } else {
            self.gerudo_man_draw(k);
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
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = self.sprite_slot_view(k).a();
                    self.sprite_slot_view_mut(k).set_x_low(value);
                    let value = self.sprite_slot_view(k).b();
                    self.sprite_slot_view_mut(k).set_x_high(value);
                    let value = self.sprite_slot_view(k).c();
                    self.sprite_slot_view_mut(k).set_y_low(value);
                    let value = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_y_high(value);
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0
                    && self
                        .game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                        .wrapping_add(0x30)
                        < 0x60
                    && self
                        .game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                        .wrapping_add(0x30)
                        < 0x60
                {
                    let value = 2;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 31;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
            }
            2 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 3;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 96;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_apply_speed_towards_link(k, 16);
                } else {
                    let value = SPRITE_4_C_GELDMAN_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 2)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            3 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 4;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 8;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_4_C_GELDMAN_PURSUE_GFX
                        [usize::from((self.sprite_slot_view(k).delay_main() >> 2) & 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                    self.sprite_check_damage_to_and_from_link(k);
                    self.sprite_move_xy(k);
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 16;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_4_C_GELDMAN_SUBMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 1)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
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
        self.sprite_draw_single_large(k);
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
        self.sprite_move_xy(k);

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
        self.sprite_draw_single_large(k);
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
            let mut coll = self.sprite_check_tile_collision(k);
            let value = (-1i8) as u8;
            self.sprite_slot_view_mut(k).set_x_velocity(value);
            let value = (-1i8) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(value);
            coll |= self.sprite_check_tile_collision(k);
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
        self.sprite_move_xy(k);
        self.sprite_check_damage_to_link(k);
        let mut j = usize::from(self.sprite_slot_view(k).direction());
        let value = SOLDIER_B_STEP_X_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SOLDIER_B_STEP_Y_VELOCITIES[j] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        let _ = self.sprite_check_tile_collision(k);

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
    // void Sprite_59_LostWoodsBird(int k) {  // 85940e
    pub(super) fn sprite_59_lost_woods_bird(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | if sign8(self.sprite_slot_view(k).x_velocity()) {
                0
            } else {
                0x40
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = 0;
                self.sprite_slot_view_mut(k).set_graphics(value);
                self.sprite_slot_view_mut(k).subtract_z_velocity(1);
                if sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0xf1)) {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
            }
            1 => {
                self.sprite_slot_view_mut(k).add_z_velocity(2);
                if !sign8(self.sprite_slot_view(k).z_velocity().wrapping_sub(0x10)) {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                self.sprite_slot_view_mut(k).add_subtype2(1);
                let value = (self.sprite_slot_view(k).subtype2() >> 1) & 1;
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_5A_LostWoodsSquirrel(int k) {  // 859468
    pub(super) fn sprite_5_a_lost_woods_squirrel(&mut self, k: usize) {
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            return;
        }
        let value = (self.sprite_slot_view(k).oam_flags() & !0x40)
            | if sign8(self.sprite_slot_view(k).x_velocity()) {
                0
            } else {
                0x40
            };
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        self.sprite_slot_view_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot_view(k).z()) {
            let value = 0;
            self.sprite_slot_view_mut(k).set_z(value);
            let value = 16;
            self.sprite_slot_view_mut(k).set_z_velocity(value);
            let value = 12;
            self.sprite_slot_view_mut(k).set_delay_main(value);
        }
        let value = u8::from(self.sprite_slot_view(k).delay_main() != 0);
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_58_Crab(int k) {  // 8594b5
    pub(super) fn sprite_58_crab(&mut self, k: usize) {
        self.crab_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 || self.sprite_slot_view(k).delay_main() == 0 {
            // ROM Sprite_58_Crab $05:94CE: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
            let value = self.get_random_number_with_carry().masked_adc(63, 32);
            self.sprite_slot_view_mut(k).set_delay_main(value);
            let value = self.sprite_slot_view(k).delay_main() & 3;
            self.sprite_slot_view_mut(k).set_direction(value);
        }
        let j = self.sprite_slot_view(k).direction();
        let value = SPRITE_58_CRAB_LOCAL_X_VELOCITIES[usize::from(j & 3)] as u8;
        self.sprite_slot_view_mut(k).set_x_velocity(value);
        let value = SPRITE_58_CRAB_LOCAL_Y_VELOCITIES[usize::from(j & 3)] as u8;
        self.sprite_slot_view_mut(k).set_y_velocity(value);
        self.sprite_slot_view_mut(k).add_subtype2(1);
        let value = (self.sprite_slot_view(k).subtype2() >> if j < 2 { 1 } else { 3 }) & 1;
        self.sprite_slot_view_mut(k).set_graphics(value);
    }

    // -----------------------------------------------------------------------
    // void Sprite_80_Firesnake(int k) {  // 9ed1d1
    pub(super) fn sprite_80_firesnake(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
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
        self.sprite_check_damage_to_and_from_link(k);
        self.firesnake_spawn_fireball(k);
        if self.sprite_slot_view(k).wall_collision() == 0 {
            self.sprite_move_xy(k);
        }
        if self.sprite_check_tile_collision(k) != 0 {
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
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = (self.sprite_slot_view(k).oam_flags() & 0x3f)
                | SPRITE_87_KODONGO_FIRE_OAM_FLAGS
                    [usize::from((self.game_state.frame.frame_counter >> 2) & 3)];
            self.sprite_slot_view_mut(k).set_oam_flags(value);
            if !self.sprite_check_damage_to_link(k) {
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) == 0 {
                    return;
                }
            }
            let value = 127;
            self.sprite_slot_view_mut(k).set_delay_main(value);
            self.sprite_slot_view_mut(k).and_oam_flags(63);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
        } else {
            let carried =
                (self.sprite_check_damage_from_link(k) & PLAYER_DAMAGE_CARRY_MASK_DRAW) != 0;
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

    // -----------------------------------------------------------------------
    // void Sprite_7C_GreenStalfos(int k) {  // 9ed299
    pub(super) fn sprite_7_c_green_stalfos(&mut self, k: usize) {
        let mut j = usize::from(self.sprite_slot_view(k).direction() & 3);
        let value =
            (self.sprite_slot_view(k).oam_flags() & !0x40) | SPRITE_7_C_GREEN_STALFOS_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        let value = SPRITE_7_C_GREEN_STALFOS_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        j = usize::from(self.sprite_direction_to_face_link(k, None) & 3);
        if SPRITE_7_C_GREEN_STALFOS_DIRECTIONS[j] != self.game_state.player.follower_link.facing() {
            let value = 0;
            self.sprite_slot_view_mut(k).set_a(value);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
                let vel = self.sprite_slot_view(k).b();
                if vel != 4 {
                    self.sprite_slot_view_mut(k).add_b(1);
                }
                self.sprite_apply_speed_towards_link(k, vel);
                let value = self.sprite_is_right_of_link(k).a;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
        } else {
            let value = 1;
            self.sprite_slot_view_mut(k).set_a(value);
            if (((k as u8) ^ self.game_state.frame.frame_counter) & 15) == 0 {
                let vel = self.sprite_slot_view(k).b();
                if vel != 0 {
                    self.sprite_slot_view_mut(k).subtract_b(1);
                }
                self.sprite_apply_speed_towards_link(k, vel);
                let value = self.sprite_is_right_of_link(k).a;
                self.sprite_slot_view_mut(k).set_direction(value);
            }
        }
        self.sprite_move_xy(k);
    }

    // -----------------------------------------------------------------------
    // void Sprite_71_Leever(int k) {  // 86cba2
    pub(super) fn sprite_71_leever(&mut self, k: usize) {
        if self.sprite_slot_view(k).ai_state() != 0 {
            self.leever_draw(k);
        } else {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        }
        if self.sprite_slot_view(k).pause() != 0 {
            let value = 8;
            self.sprite_slot_view_mut(k).set_state(value);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 1;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    self.sprite_apply_speed_towards_link(k, 16);
                    self.sprite_move_xy(k);
                    self.sprite_check_tile_collision2(k);
                }
            }
            1 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    // ROM `AND #63 : ADC #160` inherits GetRandomInt's
                    // final-LSR carry (route frame 125288).
                    let value = self.get_random_number_with_carry().masked_adc(63, 160);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_zero_velocity_xy(k);
                } else {
                    let value = SPRITE_71_LEEVER_EMERGE_GFX
                        [usize::from(self.sprite_slot_view(k).delay_main() >> 3)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = 127;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    if (self.sprite_slot_view(k).subtype2() & 7) == 0 {
                        self.sprite_apply_speed_towards_link(
                            k,
                            SPRITE_71_LEEVER_ATTACK_SPD[usize::from(self.sprite_slot_view(k).a())],
                        );
                    }
                    self.sprite_move_xy(k);
                    if self.sprite_check_tile_collision(k) != 0 {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 127;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                    } else {
                        let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_subtype2(value);
                        let value = SPRITE_71_LEEVER_ATTACK_GFX
                            [usize::from((self.sprite_slot_view(k).subtype2() >> 2) & 3)];
                        self.sprite_slot_view_mut(k).set_graphics(value);
                    }
                }
            }
            3 => {
                let value = self.sprite_slot_view(k).delay_main();
                self.sprite_slot_view_mut(k).set_ignore_projectile(value);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                    // ROM `AND #31 : ADC #64` inherits GetRandomInt's
                    // final-LSR carry.
                    let value = self.get_random_number_with_carry().masked_adc(31, 64);
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = SPRITE_71_LEEVER_SUBMERGE_GFX
                        [usize::from((self.sprite_slot_view(k).delay_main() >> 3) ^ 15)];
                    self.sprite_slot_view_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_1D_FluteQuest(int k) {  // 86c2e5
    pub(super) fn sprite_1_d_flute_quest(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.game_state.world.location.overworld_screen_index() == 0x18 {
            if self.game_state.inventory.items.flute() == 3 {
                let value = 0;
                self.sprite_slot_view_mut(k).set_state(value);
            }
        } else if (self.game_state.inventory.items.flute() & 2) != 0 {
            let value = 0;
            self.sprite_slot_view_mut(k).set_state(value);
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_12_Moblin(int k) {  // 8698e4
    pub(super) fn sprite_12_moblin(&mut self, k: usize) {
        self.moblin_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let _ = self.sprite_check_tile_collision(k);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = SPRITE_12_MOBLIN_DELAY[usize::from(self.get_random_number() & 3)];
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                    self.sprite_slot_view_mut(k).add_ai_state(1);
                    let value = self.sprite_slot_view(k).head_direction();
                    self.sprite_slot_view_mut(k).set_direction(value);
                    let j = usize::from(self.sprite_slot_view(k).head_direction());
                    let value = SPRITE_12_MOBLIN_LOCAL_X_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_x_velocity(value);
                    let value = SPRITE_12_MOBLIN_LOCAL_Y_VELOCITIES[j] as u8;
                    self.sprite_slot_view_mut(k).set_y_velocity(value);
                }
            }
            1 => {
                let value = (self.sprite_slot_view(k).subtype2() & 1)
                    + SPRITE_12_MOBLIN_LOCAL_GRAPHICS
                        [usize::from(self.sprite_slot_view(k).direction())];
                self.sprite_slot_view_mut(k).set_graphics(value);
                if self.sprite_slot_view(k).wall_collision() == 0 {
                    if self.sprite_slot_view(k).delay_main() != 0 {
                        let new_e = self.sprite_slot_view(k).e().wrapping_sub(1);
                        let value = new_e;
                        self.sprite_slot_view_mut(k).set_e(value);
                        if sign8(new_e) {
                            let value = 11;
                            self.sprite_slot_view_mut(k).set_e(value);
                            let value = self.sprite_slot_view(k).subtype2().wrapping_add(1);
                            self.sprite_slot_view_mut(k).set_subtype2(value);
                        }
                        return;
                    }
                    if self.sprite_slot_view(k).direction()
                        == self.sprite_direction_to_face_link(k, None)
                    {
                        let value = self.sprite_slot_view(k).ai_state().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_ai_state(value);
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_main(value);
                        self.sprite_zero_velocity_xy(k);
                        let value = 0;
                        self.sprite_slot_view_mut(k).set_z_velocity(value);
                        return;
                    }
                    let value = 0x10;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                } else {
                    let value = 0x0c;
                    self.sprite_slot_view_mut(k).set_delay_main(value);
                }
                let dir_idx = usize::from(
                    (self.sprite_slot_view(k).direction() << 1) | (self.get_random_number() & 1),
                );
                let value = SPRITE_12_MOBLIN_DIRS[dir_idx];
                self.sprite_slot_view_mut(k).set_head_direction(value);
                let value = 0;
                self.sprite_slot_view_mut(k).set_ai_state(value);
                self.sprite_slot_view_mut(k).add_c(1);
                if self.sprite_slot_view(k).c() == 4 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_c(value);
                    let value = self.sprite_direction_to_face_link(k, None);
                    self.sprite_slot_view_mut(k).set_head_direction(value);
                }
                self.sprite_zero_velocity_xy(k);
                let value = 0;
                self.sprite_slot_view_mut(k).set_z_velocity(value);
            }
            2 => {
                let mut j = self.sprite_slot_view(k).direction();
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let value = 0;
                    self.sprite_slot_view_mut(k).set_ai_state(value);
                }
                if self.sprite_slot_view(k).delay_main() < 16 {
                    if self.sprite_slot_view(k).delay_main() == 15 {
                        self.moblin_materialize_spear(k);
                        let value = 32;
                        self.sprite_slot_view_mut(k).set_delay_aux1(value);
                    }
                    j = j.wrapping_add(4);
                }
                let value = SPRITE_12_MOBLIN_SECONDARY_GRAPHICS[usize::from(j)];
                self.sprite_slot_view_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // void Sprite_Cukeman(int k) {  // 9afa0c
    pub(super) fn sprite_cukeman(&mut self, k: usize) {
        if self.sprite_slot_view(k).head_direction() == 0 {
            return;
        }

        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        if self.sprite_slot_view(k).state() == 9
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
            && cur_x.wrapping_sub(link_x).wrapping_add(0x18) < 0x30
            && link_y.wrapping_sub(cur_y).wrapping_add(0x20) < 0x30
            && (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) != 0
        {
            let subtype = self.sprite_slot_view(k).subtype();
            self.dialogue_message_index_mut()
                .set_value(0x017a + u16::from(subtype & 1));
            let value = subtype.wrapping_add(1);
            self.sprite_slot_view_mut(k).set_subtype(value);
            self.sprite_show_message_minimal_c();
        }

        let old = self.sprite_slot_view(k).oam_flags() & 0xf0;
        let value = old | 8;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.cukeman_draw(k);
        let value = old | 0x0d;
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.oam_allocate_from_region_a(0x10);
    }

    // -----------------------------------------------------------------------
    // void Octoballoon_Draw(int k) {  // 86d784
    pub(super) fn octoballoon_draw(&mut self, k: usize) {
        let mut d = 0usize;
        if self.sprite_slot_view(k).state() == 6 {
            if self.sprite_slot_view(k).delay_main() == 6 && self.game_state.frame.submodule == 0 {
                self.octoballoon_form_babby(k);
            }
            d = usize::from((self.sprite_slot_view(k).delay_main() >> 1) & 4) + 4;
        }
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut info = SpritePrepOamCoordsRet {
            x: info_x,
            y: info_y,
            r4: 0,
            flags: info_flags,
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..4).rev() {
            let j = d + i;
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(OCTOBALLOON_DRAW_X_OFFSETS[j] as i16 as u16),
                info_y.wrapping_add(OCTOBALLOON_DRAW_Y_OFFSETS[j] as i16 as u16),
                OCTOBALLOON_DRAW_CHARS[j],
                OCTOBALLOON_DRAW_FL[j] | info_flags,
                2,
            );
            oam += 4;
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // void Kholdstare_Draw(int k) {  // 8dd98f
    pub(super) fn kholdstare_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.sprite_slot_view(k).a());
        self.set_oam_helper0_at_for_draw(
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
    // void Octoballoon_RecoilLink(int k) {  // 86d72b
    pub(super) fn octoballoon_recoil_link(&mut self, k: usize) {
        if self.game_state.player.follower_link.incapacitated_timer() == 0 {
            self.follower_link_state_mut().set_incapacitated_timer(4);
            self.sprite_apply_recoil_to_link(k, 16);
            self.sprite_invert_speed_xy(k);
        }
    }

    // -----------------------------------------------------------------------
    // void BatCrash_DrawHardcodedGarbage(int k) {  // 9af750
    pub(super) fn bat_crash_draw_hardcoded_garbage(&mut self, _k: usize) {
        let mut oam = OAM_BUF + 76 * 4;
        for &(x, y, ch, fl) in &BAT_CRASH_DRAW_HARDCODED_GARBAGE_OAMS {
            self.oam_state_mut()
                .write_entry(oam, x as u8, y as u8, ch, fl);
            oam += 4;
        }
        for i in 0..9 {
            let value = 2;
            self.oam_state_mut().set_extended_byte(76 + i, value);
        }
    }

    // -----------------------------------------------------------------------
    // void Moldorm_Draw(int k) {  // 9df822
    pub(super) fn moldorm_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut base = self.sprite_slot_view(k).direction().wrapping_sub(1);
        for _ in (0..2).rev() {
            let idx = usize::from(base & 0x0f);
            self.set_oam_helper0_at_for_draw(
                oam,
                info_x.wrapping_add(MOLDORM_DRAW_X_OFFSETS[idx] as i16 as u16),
                info_y.wrapping_add(MOLDORM_DRAW_Y_OFFSETS[idx] as i16 as u16),
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
            self.set_oam_helper0_at_for_draw(
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
    pub(super) fn sprite_54_lanmolas(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
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

    pub(super) fn complete_lanmola_after_subtype2_increment(
        &mut self,
        k: usize,
        continuation: LanmolaDrawContinuation,
    ) {
        self.lanmola_draw_after_subtype2_increment(k, continuation);
        self.sprite_54_lanmolas_after_draw(k);
    }

    pub(super) fn lanmola_prep_and_draw_through_subtype2_increment(
        &mut self,
        k: usize,
    ) -> LanmolaDrawContinuation {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        self.lanmola_draw_through_subtype2_increment(k)
    }

    pub(super) fn lanmola_draw_through_subtype2_increment(
        &mut self,
        k: usize,
    ) -> LanmolaDrawContinuation {
        let spr_offs = usize::from(LANMOLA_DRAW_SPR_OFFS[k]);
        self.oam_state_mut()
            .set_current_pointer((0x800 + spr_offs * 4) as u16);
        self.oam_state_mut()
            .set_current_extended_pointer((0x0a20 + spr_offs) as u16);

        let value = Self::sprite_convert_velocity_to_angle(
            self.sprite_slot_view(k).x_velocity(),
            self.sprite_slot_view(k)
                .y_velocity()
                .wrapping_sub(self.sprite_slot_view(k).z_velocity()),
        );

        self.sprite_slot_view_mut(k).set_graphics(value);
        let r2 = self.sprite_slot_view(k).subtype2();
        let r5 = r2;
        let j = k * 64 + usize::from(r2);
        let x_low = self.sprite_slot_view(k).x_low();
        let y_low = self.sprite_slot_view(k).y_low();
        self.moldorm_history_mut(j).set_low_position(x_low, y_low);
        let z_offset = self.sprite_slot_view(k).z();
        let direction = self.sprite_slot_view(k).graphics();
        let mut segment = self.lanmola_segment_motion_mut(j);
        segment.set_z_offset(z_offset);
        segment.set_direction(direction);
        if self.sprite_slot_view(k).state() == 9
            && (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) == 0
        {
            let value = self.sprite_slot_view(k).subtype2().wrapping_add(1) & 63;
            self.sprite_slot_view_mut(k).set_subtype2(value);
        }

        LanmolaDrawContinuation { r2, r5 }
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
            self.set_oam_plain_at_for_draw(
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
                self.set_oam_plain_at_for_draw(
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

    pub(super) fn eyegore_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 4;
        self.sprite_draw_multiple(k, &EYEGORE_DRAW_FRAMES[base..base + 4], Some(&mut info));
        if self.sprite_slot_view(k).pause() == 0 {
            self.sprite_draw_shadow_custom(k, &mut info, 14);
        }
    }

    pub(super) fn bomb_trooper_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_helper0_at_for_draw(
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
        let mut shadow_info = SpritePrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 10);
    }

    pub(super) fn pikit_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let info = PrepOamCoordsRet { x, y, r4: 0, flags };
        let mut shadow_info = SpritePrepOamCoordsRet { x, y, r4: 0, flags };
        self.sprite_draw_pikit_tongue(k, &info);
        let oam = self.game_state.oam.current_pointer_usize();
        let oam_byte = self.game_state.oam.entry_x(oam);
        self.temp_counter_mut().set(oam_byte);
        let shared_scratch_a = self.game_state.oam.entry_y(oam);
        self.sprite_workspace_mut()
            .set_shared_scratch_a(shared_scratch_a);
        self.oam_state_mut()
            .set_current_pointer((oam as u16).wrapping_add(24));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(6));
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(
            k,
            &PIKIT_DRAW_FRAMES[base..base + 2],
            Some(&mut shadow_info),
        );
        let bak = self.sprite_slot_view(k).flags2();
        self.sprite_slot_view_mut(k).subtract_flags2(6);
        self.sprite_draw_shadow_custom(k, &mut shadow_info, 10);
        let value = bak;
        self.sprite_slot_view_mut(k).set_flags2(value);
        self.sprite_draw_pikit_loot(k, &info);
    }

    pub(super) fn chain_chomp_draw(&mut self, k: usize) {
        let j = usize::from(self.sprite_slot_view(k).direction() & 0x0f);
        let value = CHAIN_CHOMP_DRAW_LOCAL_GRAPHICS[j];
        self.sprite_slot_view_mut(k).set_graphics(value);
        let value = (self.sprite_slot_view(k).oam_flags() & 0x3f) | CHAIN_CHOMP_DRAW_OAM_FLAGS[j];
        self.sprite_slot_view_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);
        let mut oam = self.game_state.oam.current_pointer_usize() + 4;
        let flags =
            self.sprite_slot_view(k).oam_flags() ^ self.sprite_slot_view(k).object_priority();
        let r8 = u16::from(self.sprite_slot_view(k).delay_aux1() & 1) + 4;
        let mut pos = k * 8;
        for _ in (0..6).rev() {
            let x = self
                .game_state
                .sprites
                .chain_chomp_history
                .x(pos)
                .wrapping_add(r8)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .game_state
                .sprites
                .chain_chomp_history
                .y(pos)
                .wrapping_add(r8)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.set_oam_helper0_at_for_draw(oam, x, y, 0x8b, (flags & 0xf0) | 0x0d, 0);
            pos += 1;
            oam += 4;
        }
    }

    pub(super) fn moblin_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let gfx = usize::from(self.sprite_slot_view(k).graphics());
        self.sprite_draw_multiple(
            k,
            &MOBLIN_DRAW_FRAMES[gfx * 4..gfx * 4 + 4],
            Some(&mut info),
        );
        if self.sprite_slot_view(k).pause() != 0 {
            return;
        }
        let mut oam = self.game_state.oam.current_pointer_usize();
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            for _ in 0..4 {
                let ext_index = (oam - OAM_BUF) / 4;
                if self.game_state.oam.extended_byte(ext_index) & 2 == 0 {
                    self.oam_state_mut().hide_entry(oam);
                }
                oam += 4;
            }
        }
        let oam = self.game_state.oam.current_pointer_usize()
            + usize::from(MOBLIN_DRAW_OBJ_OFFS[gfx.min(MOBLIN_DRAW_OBJ_OFFS.len() - 1)]) * 4;
        let j = usize::from(self.sprite_slot_view(k).head_direction() & 3);
        self.oam_state_mut()
            .set_entry_char(oam, MOBLIN_DRAW_HEAD_CHAR[j]);
        self.oam_state_mut()
            .merge_entry_flags(oam, !0x40, ALTAR_ZELDA_HEAD_FLAGS[j]);
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    pub(super) fn altar_zelda_draw_body(&mut self, k: usize, info: &PrepOamCoordsRet) {
        self.oam_allocate_from_region_a(8);
        let z = self.sprite_slot_view(k).z().min(31);
        let xoffs = u16::from(ALTAR_ZELDA_DRAW_BODY_X_OFFS[usize::from(z >> 1)]);
        let y = self
            .sprite_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        let oam = self.game_state.oam.current_pointer_usize();
        self.set_oam_helper0_at_for_draw(
            oam,
            info.x.wrapping_add(xoffs),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
        self.set_oam_helper0_at_for_draw(
            oam + 4,
            info.x.wrapping_sub(xoffs),
            y.wrapping_add(7),
            0x6c,
            0x24,
            2,
        );
    }

    pub(super) fn tektite_draw(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = usize::from(self.sprite_slot_view(k).graphics()) * 2;
        self.sprite_draw_multiple(k, &TEKTITE_DRAW_FRAMES[base..base + 2], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    pub(super) fn archery_game_draw_prize(&mut self, k: usize) {
        let Some((info_x, info_y, _info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k)
        else {
            return;
        };
        let mut oam = self.game_state.oam.current_pointer_usize() + 4;
        let b = usize::from(self.sprite_slot_view(k).b()).wrapping_sub(1);
        for i in (0..5).rev() {
            let charnum = if i == 4 {
                ARCHERY_GAME_DRAW_PRIZE_CHARS_4[b]
            } else if i == 3 {
                ARCHERY_GAME_DRAW_PRIZE_CHARS_3[b]
            } else {
                ARCHERY_GAME_DRAW_PRIZE_CHARS[i]
            };
            let flags = ARCHERY_GAME_DRAW_PRIZE_FLAGS[i] & if charnum < 0x7c { 0xff } else { 0xfe };
            self.set_oam_plain_at_for_draw(
                oam,
                info_x.wrapping_add(ARCHERY_GAME_DRAW_PRIZE_X_OFFSETS[i] as i16 as u16) as u8,
                info_y.wrapping_add(ARCHERY_GAME_DRAW_PRIZE_Y_OFFSETS[i] as i16 as u16) as u8,
                charnum,
                flags,
                0,
            );
            oam += 4;
        }
        self.sprite_draw_distress_custom(info_x, info_y, self.game_state.frame.frame_counter);
    }

    pub(super) fn bush_soldier_common_draw(&mut self, k: usize) {
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
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
            self.set_oam_helper0_at_for_draw(
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

    pub(super) fn archery_game_guy_draw(&mut self, k: usize) {
        self.oam_allocate_defer_to_player(k);
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let g = usize::from(self.sprite_slot_view(k).graphics());
        for i in (0..3).rev() {
            let j = g * 3 + i;
            self.set_oam_plain_at_for_draw(
                oam,
                info.x
                    .wrapping_add(ARCHERY_GAME_GUY_DRAW_X_OFFSETS[j] as i16 as u16)
                    as u8,
                info.y
                    .wrapping_add(ARCHERY_GAME_GUY_DRAW_Y_OFFSETS[j] as i16 as u16)
                    as u8,
                ARCHERY_GAME_GUY_DRAW_CHARS[j],
                ARCHERY_GAME_GUY_DRAW_FLAGS[j] | info.flags,
                ARCHERY_GAME_GUY_DRAW_BIG[j],
            );
            oam += 4;
        }
        self.sprite_draw_shadow_custom(k, &mut info, 10);
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
            self.set_oam_plain_at_for_draw(
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
        self.sprite_correct_oam_entries_for_draw(k, 4, 0xff);

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
                let new_dir = self.sprite_direction_to_face_link(k, None);
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

    // -----------------------------------------------------------------------
    // Local helper adapters with `_for_draw` suffix. Each names the canonical
    // helper or split-module bridge it represents.
    // -----------------------------------------------------------------------

    /// Mirrors `SetOamHelper0` (sprite.h:50) but the canonical `set_oam_helper0_at`
    /// in zelda_rtl.rs is module-private, so route the call through the same
    /// implementation via the public `set_oam_helper0_at` (which the round-2
    /// agents proved is visible from sibling sprite_main_* modules).
    fn set_oam_helper0_at_for_draw(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.set_oam_helper0_at(oam, x, y, charnum, flags, big);
    }

    /// Mirrors `SetOamPlain` (sprite.h:66) — writes x/y/charnum/flags + a
    /// raw big byte directly into the OAM buffer at a byte address.
    fn set_oam_plain_at_for_draw(
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

    /// Rewired to canonical Sprite_CorrectOamEntries port.
    fn sprite_correct_oam_entries_for_draw(&mut self, k: usize, count: u8, mask: u8) {
        self.sprite_correct_oam_entries(k, count as i32, mask);
    }

    /// OpenGargoylesDomain (overworld.c:3527) — local bridge for
    /// `Sprite_14_ThievesTownGrate`; keeps the canonical overworld function
    /// name unclaimed until that module is ported.
    fn open_gargoyles_domain_for_draw(&mut self) {
        self.overworld_draw_map16_persist_for_draw(0x0d3e, 0x0e1b);
        self.overworld_draw_map16_persist_for_draw(0x0d40, 0x0e1c);
        self.overworld_draw_map16_persist_for_draw(0x0dbe, 0x0e1d);
        self.overworld_draw_map16_persist_for_draw(0x0dc0, 0x0e1e);
        self.overworld_draw_map16_persist_for_draw(0x0e3e, 0x0e1f);
        self.overworld_draw_map16_persist_for_draw(0x0e40, 0x0e20);
        self.set_overworld_event_bits(0x58, 0x20);
        self.set_sound_effect_2(0x1b);
        self.set_bg_vram_load_mode(1);
    }

    /// CreatePyramidHole (overworld.c:3539) — local bridge for
    /// `Sprite_BatCrash`; keeps the canonical overworld function name
    /// unclaimed until that module is ported.
    fn create_pyramid_hole_for_draw(&mut self) {
        self.overworld_draw_map16_persist_for_draw(0x03bc, 0x0e3f);
        self.overworld_draw_map16_persist_for_draw(0x03be, 0x0e40);
        self.overworld_draw_map16_persist_for_draw(0x03c0, 0x0e41);
        self.overworld_draw_map16_persist_for_draw(0x043c, 0x0e42);
        self.overworld_draw_map16_persist_for_draw(0x043e, 0x0e43);
        self.overworld_draw_map16_persist_for_draw(0x0440, 0x0e44);
        self.overworld_draw_map16_persist_for_draw(0x04bc, 0x0e45);
        self.overworld_draw_map16_persist_for_draw(0x04be, 0x0e46);
        self.overworld_draw_map16_persist_for_draw(0x04c0, 0x0e47);
        self.set_ambient_sound_effect_word(0x3515);
        self.set_overworld_event_bits(0x5b, 0x20);
        self.set_sound_effect_2(3);
        self.set_bg_vram_load_mode(1);
    }

    fn overworld_draw_map16_persist_for_draw(&mut self, pos: u16, value: u16) {
        self.dungeon_room_tilemaps_mut()
            .set_bg2_tile_by_byte_pos(pos, value);
        self.overworld_draw_map16_for_draw(pos, value);
    }

    fn overworld_draw_map16_for_draw(&mut self, pos: u16, value: u16) {
        let vram_pos = overworld_find_map16_vram_address_for_draw(pos);
        let dst = self.game_state.display.current_vram_upload_data_address();
        let src = value as usize * 4;
        let tile0 = self.asset_u16(70, src);
        let tile1 = self.asset_u16(70, src + 1);
        let tile2 = self.asset_u16(70, src + 2);
        let tile3 = self.asset_u16(70, src + 3);
        self.write_vram_upload_absolute_word(dst, vram_pos.swap_bytes());
        self.write_vram_upload_absolute_word(dst + 2, 0x0300);
        self.write_vram_upload_absolute_word(dst + 4, tile0);
        self.write_vram_upload_absolute_word(dst + 6, tile1);
        self.write_vram_upload_absolute_word(dst + 8, vram_pos.wrapping_add(0x20).swap_bytes());
        self.write_vram_upload_absolute_word(dst + 10, 0x0300);
        self.write_vram_upload_absolute_word(dst + 12, tile2);
        self.write_vram_upload_absolute_word(dst + 14, tile3);
        self.write_vram_upload_absolute_word(dst + 16, 0xffff);
        self.advance_vram_upload_cursor_by(16);
    }
}

#[cfg(test)]
#[path = "sprite_main_draw_tests.rs"]
mod tests;
