//! Ported HelmasaurKing-boss handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 19275..19584). The original C body is reproduced as a comment block
//! immediately above each port so a reviewer can verify behaviour line-by-line.
//!
//! Some helpers reached from these handlers (Sprite_PrepOamCoordOrDoubleRet
//! plus the KingHelmasaur_*/SpriteDraw_KingHelmasaur_* draw helpers) are still
//! wrapped behind local `_for_helmasaur_king` shims until their canonical OAM
//! paths are ported.

use crate::zelda_rtl::sprite::{
    SpriteDamageFromEntry, SpriteHitBoxEntry, SpriteInactiveEntry, SpriteMoveEntry,
    SpriteSingleLargeEntry,
};
use super::sprite::PrepOamCoordsRet;
use super::*;
use crate::types::{sign8, SpriteHitBox};

// kHelmasaur_Tab0 from sprite_main.c:390 — 32-entry triangle wave used by
// HelmasaurKing_Reinitialize to write overlord_x_lo[0..4].
const HELMASAUR_MASK_X_OFFSETS: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 7, 6, 5, 4, 3, 2, 1,
];

// kFluteBoyAnimal_Xvel from sprite_main.c:16 — used by HelmasaurKing_SwingTail.
const HELMASAUR_TAIL_SWING_X_VELOCITY_TARGETS: [i8; 4] = [16, -16, 0, 0];

// State-machine and debris tables promoted from C-local statics.
const HELMASAUR_MASK_DAMAGE_STAGE_BY_HEALTH: [u8; 13] = [3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 1, 1, 0];
const HELMASAUR_KING_PATROL_X_VELOCITIES: [i8; 8] = [-12, -12, -4, 0, 4, 12, 12, 0];
const HELMASAUR_KING_PATROL_Y_VELOCITIES: [i8; 8] = [0, 4, 12, 12, 12, 4, 0, 12];
const HELMASAUR_FIREBALL_TRI_SPLIT_X_VELOCITIES: [i8; 3] = [0, 28, -28];
const HELMASAUR_FIREBALL_TRI_SPLIT_Y_VELOCITIES: [i8; 3] = [-32, 24, 24];
const HELMASAUR_FIREBALL_TRI_SPLIT_DELAYS: [u8; 6] = [32, 80, 128, 32, 80, 128];
const HELMASAUR_FIREBALL_QUAD_SPLIT_X_VELOCITIES: [i8; 4] = [32, 32, -32, -32];
const HELMASAUR_FIREBALL_QUAD_SPLIT_Y_VELOCITIES: [i8; 4] = [-32, 32, -32, 32];
const HELMASAUR_MASK_DEBRIS_X_OFFSETS: [i8; 10] = [-16, 0, 16, -16, 0, 16, -8, 8, -16, 16];
const HELMASAUR_MASK_DEBRIS_Y_OFFSETS: [i8; 10] = [24, 27, 24, 24, 27, 24, 27, 27, 24, 24];
const HELMASAUR_MASK_DEBRIS_Z_OFFSETS: [i8; 10] = [29, 32, 29, 13, 16, 13, 0, 0, 13, 13];
const HELMASAUR_MASK_DEBRIS_X_VELOCITIES: [i8; 10] = [-16, -4, 14, -12, 4, 18, -2, 2, -12, 18];
const HELMASAUR_MASK_DEBRIS_Y_VELOCITIES: [i8; 10] = [-8, -4, -6, 4, 2, 7, 6, 8, 4, 7];
const HELMASAUR_MASK_DEBRIS_Z_VELOCITIES: [i8; 10] = [32, 40, 36, 37, 39, 34, 30, 33, 37, 34];
const HELMASAUR_MASK_DEBRIS_OAM_FLAGS: [u8; 10] = [0, 0, 0x40, 0, 0, 0x40, 0, 0x40, 0, 0x40];
const HELMASAUR_MASK_DEBRIS_GRAPHICS: [u8; 10] = [0, 1, 0, 2, 3, 2, 4, 4, 5, 5];

impl ZeldaState {
    // void HelmasaurKing_Initialize(int k) {  // sprite_main.c:19275
    //   overlord_gen1[7] = 0x30;
    //   overlord_gen1[5] = 0x80;
    //   overlord_gen1[6] = 0;
    //   overlord_gen2[0] = 0;
    //   overlord_gen2[3] = 0;
    //   overlord_gen2[1] = 0;
    //   overlord_gen2[2] = 0;
    //   HelmasaurKing_Reinitialize(k);
    // }
    pub(super) fn helmasaur_king_initialize(&mut self, k: usize) {
        self.overlord_slot_view_mut(7).set_gen1(0x30);
        self.overlord_slot_view_mut(5).set_gen1(0x80);
        self.overlord_slot_view_mut(6).set_gen1(0);
        self.overlord_slot_view_mut(0).set_gen2(0);
        self.overlord_slot_view_mut(3).set_gen2(0);
        self.overlord_slot_view_mut(1).set_gen2(0);
        self.overlord_slot_view_mut(2).set_gen2(0);
        self.helmasaur_king_reinitialize(k);
    }

    // void HelmasaurKing_Reinitialize(int k) {  // sprite_main.c:19286
    //   uint8 t = sprite_subtype2[k];
    //   for (int i = 3; i >= 0; i--) {
    //     overlord_x_lo[i] = kHelmasaur_Tab0[t + i * 8 & 0x1f];
    //   }
    // }
    pub(super) fn helmasaur_king_reinitialize(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e8019));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e8019);
        let t = self.sprite_slot_view(k).subtype2() as usize;
        for i in (0..=3usize).rev() {
            b.block(0x1e8024);
            let idx = (t.wrapping_add(i.wrapping_mul(8))) & 0x1f;
            self.overlord_slot_view_mut(i)
                .set_x_low(HELMASAUR_MASK_X_OFFSETS[idx]);
            if i != 0 { b.taken(0x1e8034); }
        }
        b.block(0x1e8036);
    }

    // void Sprite_92_HelmasaurKing(int k) {  // sprite_main.c:19293
    //   ...see sprite_main.c...
    // }
    pub(super) fn sprite_92_helmasaur_king(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e8039));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e8039);
        if sign8(self.sprite_slot_view(k).c()) {
            b.block(0x1e803e);
            if self.sprite_slot_view(k).delay_main() == 0 { b.taken(0x1e8041); }
            else {
                b.block(0x1e8043);
                if self.sprite_slot_view(k).delay_main() != 1 { b.taken(0x1e8044); }
            }
            if self.sprite_slot_view(k).delay_main() == 1 {
                b.block(0x1e8046);
                self.sprite_slot_view_mut(k).clear();
            }
            b.block(0x1e8049);
            // ROM $1E:8049 JSL $06:DBF0
            self.sprite_draw_single_large(k, SpriteSingleLargeEntry::Rom06DBF0);
            b.block(0x1e804d);
            if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
                return;
            }
            b.block(0x1e8050);
            if ((self.game_state.frame.frame_counter & 7) | self.sprite_slot_view(k).delay_aux1())
                == 0
            {
                b.block(0x1e8059);
                self.sprite_slot_view_mut(k).xor_oam_flags(0x40);
            } else { b.taken(0x1e8057); }
            b.block(0x1e8061);
            self.sprite_move_xyz(k, SpriteMoveEntry::Rom1EFF34);
            b.block(0x1e8064);
            self.sprite_slot_view_mut(k).subtract_z_velocity(2);
            if sign8(self.sprite_slot_view(k).z()) {
                b.block(0x1e806f);
                let mut sprite = self.sprite_slot_view_mut(k);
                sprite.set_z(0);
                sprite.set_delay_main(12);
                sprite.set_z_velocity(24);
                sprite.set_graphics(6);
            } else { b.taken(0x1e806d); }
            b.block(0x1e8081);
            return;
        }

        b.taken(0x1e803c); b.block(0x1e8082);
        if self.sprite_slot_view(k).c() < 3 {
            b.block(0x1e8086);
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.clear_object_priority_bits(0x0e);
            sprite.set_flags(0x0a);
        } else {
            b.taken(0x1e8084); b.block(0x1e8095);
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_flags4(0x1f);
            sprite.set_flags(2);
        }
        b.block(0x1e809f);
        self.helmasaur_king_draw(k);
        b.block(0x1e80a2);
        if self.sprite_slot_view(k).state() == 6 {
            b.block(0x1e80a9);
            let t = self.sprite_slot_view(k).delay_main();
            if t == 0 {
                b.block(0x1e80ae);
                self.sprite_schedule_boss_for_death(k);
                return;
            }
            b.taken(0x1e80ac); b.block(0x1e80bc);
            self.sprite_slot_view_mut(k).set_hit_timer(t | 0xf0);
            let j = self.overlord_slot_view(3).gen2();
            if t >= 128 { b.taken(0x1e80c5); }
            else {
                b.block(0x1e80c7);
                if t & 7 != 0 { b.taken(0x1e80c9); }
                else { b.block(0x1e80cb); if j==0x10 { b.taken(0x1e80d0); } }
            }
            if t < 128 && (t & 7) == 0 && j != 0x10 {
                b.block(0x1e80d2);
                if sign8(self.overlord_slot_view(5+usize::from(j)).x_low()) { b.block(0x1e80dc); }
                else { b.taken(0x1e80da); }
                b.block(0x1e80de);
                if sign8(self.overlord_slot_view(5+usize::from(j)).y_low()) { b.block(0x1e80f4); }
                else { b.taken(0x1e80f2); }
                b.block(0x1e80f6);
                self.overlord_slot_view_mut(3).add_gen2(1);
                let j = j as usize;
                let x = self
                    .sprite_get_x(k)
                    .wrapping_add(self.overlord_slot_view(5 + j).x_low() as i8 as i16 as u16);
                let y = self
                    .sprite_get_y(k)
                    .wrapping_add(self.overlord_slot_view(5 + j).y_low() as i8 as i16 as u16);
                self.sprite_workspace_mut().set_current_sprite_x(x);
                self.sprite_workspace_mut().set_current_sprite_y(y);
                self.sprite_make_boss_explosion(k);
            }
            b.block(0x1e8109);
            return;
        }

        b.taken(0x1e80a7); b.block(0x1e8117);
        if self.sprite_return_if_inactive(k, SpriteInactiveEntry::Rom06D9EC) {
            return;
        }

        b.block(0x1e811a);
        let t = HELMASAUR_MASK_DAMAGE_STAGE_BY_HEALTH
            [(self.sprite_slot_view(k).health() >> 2) as usize];
        self.sprite_slot_view_mut(k).set_c(t);
        if t == 3 {
            b.block(0x1e812a);
            if t != self.sprite_slot_view(k).e() {
                b.block(0x1e812f);
                self.sprite_slot_view_mut(k).set_hit_timer(0);
                self.helmasaur_king_explode_mask(k);
                b.block(0x1e8135);
            } else { b.taken(0x1e812d); }
        } else {
            b.taken(0x1e8128); b.block(0x1e8137);
            if t != self.sprite_slot_view(k).e() {
                b.block(0x1e813c); self.helmasaur_king_chip_away_at_mask(k);
            } else { b.taken(0x1e813a); }
        }
        b.block(0x1e813f);
        let c = self.sprite_slot_view(k).c();
        self.sprite_slot_view_mut(k).set_e(c);

        // ROM $1E:8145 JSL $06:F2AA
        self.sprite_check_damage_from_link(k, SpriteDamageFromEntry::Rom06F2AA);
        b.block(0x1e8149);
        self.helmasaur_king_swing_tail(k);
        b.block(0x1e814c);
        self.helmasaur_king_attempt_damage(k);
        b.block(0x1e814f);
        self.helmasaur_king_check_mask_damage_from_hammer(k);

        b.block(0x1e8152);
        if self.sprite_slot_view(k).delay_aux1() == 0 {
            b.taken(0x1e8155); b.block(0x1e815c);
            if self.sprite_slot_view(k).delay_aux2() != 0 {
                b.block(0x1e8161);
                if self.sprite_slot_view(k).delay_aux2() == 0x40 {
                    b.block(0x1e8165);
                    self.helmasaur_king_spit_fireball(k);
                    b.block(0x1e8168);
                    if self.sprite_slot_view(k).c() >= 3 {
                        self.helmasaur_king_begin_anim_clock(k);
                    } else { b.taken(0x1e816d); }
                } else { b.taken(0x1e8163); }
                b.block(0x1e817c);
                return;
            }
            b.taken(0x1e815f);
        } else {
            b.block(0x1e8157);
            if self.sprite_slot_view(k).delay_aux1() == 96 {
                b.taken(0x1e8159);
                self.helmasaur_king_begin_anim_clock(k);
                b.block(0x1e817c);
            } else { b.block(0x1e815b); }
            return;
        }

        b.block(0x1e817d);
        { let _dispatch = self.lag_free.then(|| crate::cycle_ledger::routine(0x008781)); b.block(0x008781); }
        b.block(0x008784); b.block(0x008799);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                b.block(0x1e819c);
                if self.sprite_slot_view(k).hit_timer()!=0 { b.taken(0x1e819f); }
                else { b.block(0x1e81a1); if self.sprite_slot_view(k).delay_main()!=0 { b.taken(0x1e81a4); } }
                let sprite = self.sprite_slot_view(k);
                if sprite.hit_timer() != 0 || sprite.delay_main() == 0 {
                  b.block(0x1e81a6);
                  if !self.helmasaur_king_maybe_fireball(k) {
                    b.block(0x1e81a9);
                    let j = (self.get_random_number() & 7) as usize;
                    b.block(0x1e81ad);
                    let damaged_mask = self.sprite_slot_view(k).c() >= 3;
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_x_velocity(HELMASAUR_KING_PATROL_X_VELOCITIES[j] as u8);
                    sprite.set_y_velocity(HELMASAUR_KING_PATROL_Y_VELOCITIES[j] as u8);
                    sprite.set_delay_main(64);
                    if damaged_mask {
                        b.block(0x1e81c8);
                        sprite.shift_x_velocity_left(1);
                        sprite.shift_y_velocity_left(1);
                        sprite.halve_delay_main();
                    } else { b.taken(0x1e81c6); }
                    b.block(0x1e81d1);
                    sprite.increment_ai_state();
                    b.block(0x1e81d4);
                  }
                } else { b.block(0x1e81d4); }
            }
            1 => {
                b.block(0x1e81d5);
                self.helmasaur_king_handle_movement(k);
                b.block(0x1e81d8);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    b.block(0x1e81dd);
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_delay_main(32);
                    sprite.increment_ai_state();
                } else { b.taken(0x1e81db); }
                b.block(0x1e81e5);
            }
            2 => {
                b.block(0x1e8210);
                if self.sprite_slot_view(k).hit_timer()!=0 { b.taken(0x1e8213); }
                else { b.block(0x1e8215); if self.sprite_slot_view(k).delay_main()!=0 { b.taken(0x1e8218); } }
                let sprite = self.sprite_slot_view(k);
                if sprite.hit_timer() != 0 || sprite.delay_main() == 0 {
                  b.block(0x1e821a);
                  if !self.helmasaur_king_maybe_fireball(k) {
                    b.block(0x1e821d);
                    let damaged_mask = self.sprite_slot_view(k).e() >= 3;
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_delay_main(64);
                    if damaged_mask {
                        b.block(0x1e8229);
                        sprite.halve_delay_main();
                    } else { b.taken(0x1e8227); }
                    b.block(0x1e822c);
                    sprite.negate_x_velocity();
                    sprite.negate_y_velocity();
                    sprite.increment_ai_state();
                    b.block(0x1e8241);
                  }
                } else { b.block(0x1e8241); }
            }
            3 => {
                b.block(0x1e8242);
                self.helmasaur_king_handle_movement(k);
                b.block(0x1e8245);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    b.block(0x1e824a);
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_ai_state(0);
                    sprite.set_delay_main(64);
                } else { b.taken(0x1e8248); }
                b.block(0x1e8252);
            }
            _ => {}
        }
    }

    fn helmasaur_king_begin_anim_clock(&mut self, k: usize) {
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e816f);
        if self.sprite_slot_view(k).anim_clock() == 0 {
            b.block(0x1e8174);
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.increment_anim_clock();
            sprite.set_delay_aux3(32);
        } else { b.taken(0x1e8172); }
    }

    // void HelmasaurKing_HandleMovement(int k) {  // sprite_main.c:19422
    //   int n = 1 + ((frame_counter & 3) == 0) + (sprite_C[k] >= 3);
    //   do {
    //     if (!(++sprite_subtype2[k] & 15))
    //       sound_effect_1 = 0x21;
    //   } while (--n);
    //   Sprite_MoveXY(k);
    // }
    pub(super) fn helmasaur_king_handle_movement(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e81e6));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e81e6);
        self.helmasaur_king_step_animation(k);
        b.block(0x1e81e9);
        if self.game_state.frame.frame_counter & 3 == 0 {
            b.block(0x1e81ef); self.helmasaur_king_step_animation(k);
        } else { b.taken(0x1e81ed); }
        b.block(0x1e81f2);
        if self.sprite_slot_view(k).c() >= 3 {
            b.block(0x1e81f9); self.helmasaur_king_step_animation(k);
        } else { b.taken(0x1e81f7); }
        b.block(0x1e81fc);
        // ROM $1E:81FC
        self.sprite_move_xy(k, SpriteMoveEntry::Rom1EFF34);
        b.block(0x1e81ff);
    }

    fn helmasaur_king_step_animation(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e8200));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e8200);
        self.sprite_slot_view_mut(k).increment_subtype2();
        if self.sprite_slot_view(k).subtype2() & 15 == 0 {
            b.block(0x1e820a); self.set_sound_effect_1(0x21);
        } else { b.taken(0x1e8208); }
        b.block(0x1e820f);
    }

    // bool HelmasaurKing_MaybeFireball(int k) {  // sprite_main.c:19431
    //   if (++sprite_subtype[k] != 4)
    //     return false;
    //   sprite_subtype[k] = 0;
    //   if (GetRandomNumber() & 1) {
    //     sprite_delay_aux2[k] = 127;
    //     SpriteSfx_QueueSfx3WithPan(k, 0x2a);
    //   } else {
    //     sprite_delay_aux1[k] = 160;
    //   }
    //   return true;
    // }
    pub(super) fn helmasaur_king_maybe_fireball(&mut self, k: usize) -> bool {
        let scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e8253));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e8253);
        self.sprite_slot_view_mut(k).increment_subtype();
        if self.sprite_slot_view(k).subtype() != 4 {
            b.taken(0x1e825b); b.block(0x1e827b);
            return false;
        }
        b.block(0x1e825d);
        // First PLA closes the child frame. Its remaining instructions run
        // in the actual caller's frame, as in the ROM routine profiler.
        drop(scope);
        b.block(0x1e825e);
        self.sprite_slot_view_mut(k).set_subtype(0);
        let random = self.get_random_number();
        b.block(0x1e8266);
        if (random & 1) != 0 {
            b.block(0x1e826a);
            self.sprite_slot_view_mut(k).set_delay_aux2(127);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2a);
            b.block(0x1e8275);
        } else {
            b.taken(0x1e8268); b.block(0x1e8276);
            self.sprite_slot_view_mut(k).set_delay_aux1(160);
            b.block(0x1e827b);
        }
        true
    }

    // void HelmasaurFireball_TriSplit(int k) {  // sprite_main.c:18666
    //   static const int8 kHelmasaurFireball_TriSplit_Xvel[3] = {0, 28, -28};
    //   static const int8 kHelmasaurFireball_TriSplit_Yvel[3] = {-32, 24, 24};
    //   static const uint8 kHelmasaurFireball_TriSplit_Delay[6] = {32, 80, 128, 32, 80, 128};
    //   SpriteSfx_QueueSfx3WithPan(k, 0x36);
    //   sprite_state[k] = 0;
    //   SPRITE_SHARED_WORK_A = GetRandomNumber();
    //   for (int i = 2; i >= 0; i--) {
    //     SpriteSpawnInfo info;
    //     int j = Sprite_SpawnDynamically(k, 0x70, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_x_vel[j] = kHelmasaurFireball_TriSplit_Xvel[i];
    //       sprite_y_vel[j] = kHelmasaurFireball_TriSplit_Yvel[i];
    //       sprite_ai_state[j] = 3;
    //       sprite_ignore_projectile[j] = 3;
    //       sprite_delay_main[j] = kHelmasaurFireball_TriSplit_Delay[(SPRITE_SHARED_WORK_A & 3) + i];
    //       sprite_head_dir[j] = 0;
    //       sprite_graphics[j] = 1;
    //     }
    //   }
    //   tmp_counter = -1;
    // }
    pub(super) fn helmasaur_fireball_tri_split(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1deed3));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        // $1D:EED3..EF2D: tri-split tail, including actual
        // spawn success/failure loops and RTS. Child bodies stay separate.
        b.block(0x1deed3);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
        b.block(0x1deed9);
        self.sprite_slot_view_mut(k).clear();
        let random = self.get_random_number();
        b.block(0x1deee5);
        self.sprite_workspace_mut().set_shared_scratch_a(random);
        for i in (0..=2usize).rev() {
            b.block(0x1deee8);
            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x70, &mut info);
            b.block(0x1deeee);
            if j >= 0 {
                b.block(0x1deef0);
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                b.block(0x1deef4);
                let delay = HELMASAUR_FIREBALL_TRI_SPLIT_DELAYS
                    [((self.game_state.sprites.workspace.shared_scratch_a() & 3) as usize) + i];
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_x_velocity(HELMASAUR_FIREBALL_TRI_SPLIT_X_VELOCITIES[i] as u8);
                sprite.set_y_velocity(HELMASAUR_FIREBALL_TRI_SPLIT_Y_VELOCITIES[i] as u8);
                sprite.set_ai_state(3);
                sprite.set_ignore_projectile(3);
                sprite.set_delay_main(delay);
                sprite.set_head_direction(0);
                sprite.set_graphics(1);
            } else { b.taken(0x1deeee); }
            b.block(0x1def27);
            if i != 0 { b.taken(0x1def2a); }
        }
        b.block(0x1def2c);
        self.temp_counter_mut().set(0xff);
    }

    // void HelmasaurFireball_QuadSplit(int k) {  // sprite_main.c:18692
    //   static const int8 kHelmasaurFireball_QuadSplit_Xvel[4] = {32, 32, -32, -32};
    //   static const int8 kHelmasaurFireball_QuadSplit_Yvel[4] = {-32, 32, -32, 32};
    //   SpriteSfx_QueueSfx3WithPan(k, 0x36);
    //   sprite_state[k] = 0;
    //   for (int i = 3; i >= 0; i--) {
    //     SpriteSpawnInfo info;
    //     int j = Sprite_SpawnDynamically(k, 0x70, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_x_vel[j] = kHelmasaurFireball_QuadSplit_Xvel[i];
    //       sprite_y_vel[j] = kHelmasaurFireball_QuadSplit_Yvel[i];
    //       sprite_ai_state[j] = 4;
    //       sprite_ignore_projectile[j] = 4;
    //     }
    //   }
    //   tmp_counter = -1;
    // }
    pub(super) fn helmasaur_fireball_quad_split(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1def3d));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        // $1D:EF3D..EF76: quad-split tail, including actual
        // spawn success/failure loops and RTS. Child bodies stay separate.
        b.block(0x1def3d);
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
        b.block(0x1def43);
        self.sprite_slot_view_mut(k).clear();
        for i in (0..=3usize).rev() {
            b.block(0x1def4b);
            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x70, &mut info);
            b.block(0x1def51);
            if j >= 0 {
                b.block(0x1def53);
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                b.block(0x1def57);
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_x_velocity(HELMASAUR_FIREBALL_QUAD_SPLIT_X_VELOCITIES[i] as u8);
                sprite.set_y_velocity(HELMASAUR_FIREBALL_QUAD_SPLIT_Y_VELOCITIES[i] as u8);
                sprite.set_ai_state(4);
                sprite.set_ignore_projectile(4);
            } else { b.taken(0x1def51); }
            b.block(0x1def70);
            if i != 0 { b.taken(0x1def73); }
        }
        b.block(0x1def75);
        self.temp_counter_mut().set(0xff);
    }

    // void HelmasaurKing_SwingTail(int k) {  // sprite_main.c:19444
    //   overlord_x_lo[4]++;
    //   HelmasaurKing_Reinitialize(k);
    //   uint8 mask = sprite_anim_clock[k] ? 0 : 1;
    //   if (!(frame_counter & mask)) {
    //     int j = sprite_D[k] & 1;
    //     overlord_gen2[0] += j ? -1 : 1;
    //     if (overlord_gen2[0] == (uint8)kFluteBoyAnimal_Xvel[j])
    //       sprite_D[k]++;
    //     WORD(overlord_gen1[5]) += (int8)overlord_gen2[0];
    //   }
    //   if (!sprite_anim_clock[k])
    //     return;
    //   if (!overlord_gen2[0])
    //     SpriteSfx_QueueSfx3WithPan(k, 0x6);
    //
    //   if (sprite_anim_clock[k] == 2) {
    //     int j = sprite_head_dir[k];
    //     WORD(overlord_gen2[1]) += j ? -4 : 4;
    //     if (overlord_gen2[1] == (uint8)(j ? -124 : 124))
    //       sprite_anim_clock[k] = 3;
    //     overlord_gen1[7] += 3;
    //   } else if (sprite_anim_clock[k] == 3) {
    //     int j = sprite_head_dir[k] ^ 1;
    //     WORD(overlord_gen2[1]) += j ? -4 : 4;
    //     if (overlord_gen2[1] == 0)
    //       sprite_anim_clock[k] = 0;
    //     overlord_gen1[7] -= 3;
    //   } else {
    //     if (!(overlord_gen2[0] | sprite_delay_aux3[k])) {
    //       sprite_head_dir[k] = overlord_gen1[6] & 1;
    //       uint8 dir = Sprite_IsRightOfLink(k).a ^ 1;
    //       if (dir == sprite_head_dir[k]) {
    //         sprite_anim_clock[k] = 2;
    //         sound_effect_2 = Sprite_CalculateSfxPan(k) | 0x26;
    //       }
    //     }
    //   }
    // }
    pub(super) fn helmasaur_king_swing_tail(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e82a0));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e82a0);
        self.overlord_slot_view_mut(4).add_x_low(1);
        self.helmasaur_king_reinitialize(k);
        b.block(0x1e82a7);
        let mask: u8 = if self.sprite_slot_view(k).anim_clock() != 0 {
            b.block(0x1e82ae);
            0
        } else {
            b.taken(0x1e82ac);
            1
        };
        b.block(0x1e82b0);
        if (self.game_state.frame.frame_counter & mask) == 0 {
            b.block(0x1e82b4);
            let j = (self.sprite_slot_view(k).direction() & 1) as usize;
            // overlord_gen2[0] += j ? -1 : 1
            let delta: u8 = if j != 0 { 0xffu8 } else { 1u8 };
            self.overlord_slot_view_mut(0).add_gen2(delta);
            if self.overlord_slot_view(0).gen2() == HELMASAUR_TAIL_SWING_X_VELOCITY_TARGETS[j] as u8
            {
                b.block(0x1e82c9);
                self.sprite_slot_view_mut(k).increment_direction();
            } else { b.taken(0x1e82c7); }
            b.block(0x1e82cc);
            if sign8(self.overlord_slot_view(0).gen2()) { b.block(0x1e82d3); }
            else { b.taken(0x1e82d1); }
            b.block(0x1e82d4);
            // WORD(overlord_gen1[5]) += (int8)overlord_gen2[0]
            let delta_w = (self.overlord_slot_view(0).gen2() as i8) as i16 as u16;
            self.overlord_slot_view_mut(5).add_gen1_word(delta_w);
        } else { b.taken(0x1e82b2); }
        b.block(0x1e82e4);
        if self.sprite_slot_view(k).anim_clock() == 0 {
            b.taken(0x1e82e7); b.block(0x1e8326);
            return;
        }
        b.block(0x1e82e9);
        if self.overlord_slot_view(0).gen2() == 0 {
            b.block(0x1e82ee);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x6);
        } else { b.taken(0x1e82ec); }

        b.block(0x1e82f4);
        if self.sprite_slot_view(k).anim_clock() == 2 {
            b.taken(0x1e82f9); b.block(0x1e832d);
            let j = self.sprite_slot_view(k).head_direction();
            let dw: u16 = if j != 0 { (-4i16) as u16 } else { 4u16 };
            self.overlord_slot_view_mut(1).add_gen2_word(dw);
            let cmp = if j != 0 { (-124i8) as u8 } else { 124u8 };
            if self.overlord_slot_view(1).gen2() == cmp {
                b.block(0x1e834a);
                self.sprite_slot_view_mut(k).set_anim_clock(3);
            } else { b.taken(0x1e8348); }
            b.block(0x1e834d);
            self.overlord_slot_view_mut(7).add_gen1(3);
        } else {
          b.block(0x1e82fb);
          if self.sprite_slot_view(k).anim_clock() == 3 {
            b.taken(0x1e82fd); b.block(0x1e8357);
            let j = self.sprite_slot_view(k).head_direction() ^ 1;
            let dw: u16 = if j != 0 { (-4i16) as u16 } else { 4u16 };
            self.overlord_slot_view_mut(1).add_gen2_word(dw);
            if self.overlord_slot_view(1).gen2() == 0 {
                b.block(0x1e8376);
                self.sprite_slot_view_mut(k).set_anim_clock(0);
            } else { b.taken(0x1e8374); }
            b.block(0x1e8379);
            self.overlord_slot_view_mut(7).subtract_gen1(3);
            b.block(0x1e8382);
        } else {
            b.block(0x1e82ff);
            if (self.overlord_slot_view(0).gen2() | self.sprite_slot_view(k).delay_aux3()) == 0 {
                b.block(0x1e8307);
                let head_direction = self.overlord_slot_view(6).gen1() & 1;
                self.sprite_slot_view_mut(k)
                    .set_head_direction(head_direction);
                b.block(0x1e8312);
                let dir = self.sprite_is_right_of_link(k).a ^ 1;
                if dir == self.sprite_slot_view(k).head_direction() {
                    b.block(0x1e831a);
                    self.sprite_slot_view_mut(k).set_anim_clock(2);
                    let sfx = self.sprite_calculate_sfx_pan(k) | 0x26;
                    b.block(0x1e8321);
                    self.set_sound_effect_2(sfx);
                } else { b.taken(0x1e8318); }
                b.block(0x1e8326);
            } else { b.taken(0x1e8305); b.block(0x1e8382); }
          }
        }
    }

    // void HelmasaurKing_CheckMaskDamageFromHammer(int k) {  // sprite_main.c:19484
    //   if (sprite_C[k] >= 3 || !(link_item_in_hand & 10) || (player_oam_y_offset == 0x80))
    //     return;
    //   SpriteHitBox hb;
    //   Player_SetupActionHitBox(&hb);
    //   uint8 bak  = sprite_y_lo[k];
    //   sprite_y_lo[k] += 8;
    //   Sprite_SetupHitBox(k, &hb);
    //   sprite_y_lo[k] = bak;
    //   if (CheckIfHitBoxesOverlap(&hb)) {
    //     sprite_health[k]--;
    //     sound_effect_2 = 0x21;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 0x30);
    //     link_actual_vel_y = pt.y;
    //     link_actual_vel_x = pt.x;
    //     link_incapacitated_timer = 8;
    //     if (!repulsespark_timer) {
    //       repulsespark_x_lo = pt.y;
    //       repulsespark_y_lo = pt.x;
    //       repulsespark_timer = 5;
    //     }
    //     SpriteSfx_QueueSfx2WithPan(k, 0x5);
    //   }
    // }
    pub(super) fn helmasaur_king_check_mask_damage_from_hammer(&mut self, k: usize) {
        if self.sprite_slot_view(k).c() >= 3
            || !self.game_state.player.follower_link.item_in_hand_has(10)
            || self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
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
        self.player_setup_action_hit_box(&mut hb);
        let bak = self.sprite_slot_view(k).y_low();
        self.sprite_slot_view_mut(k).add_y_low(8);
        self.sprite_setup_hit_box(k, &mut hb, SpriteHitBoxEntry::Unpriced);
        self.sprite_slot_view_mut(k).set_y_low(bak);
        if self.check_if_hit_boxes_overlap(&hb) {
            self.sprite_slot_view_mut(k).decrement_health();
            self.set_sound_effect_2(0x21);
            let pt = self.sprite_project_speed_towards_link(k, 0x30);
            self.follower_link_state_mut()
                .set_actual_velocity_xy(pt.x, pt.y);
            self.follower_link_state_mut().set_incapacitated_timer(8);
            if self.game_state.sprites.garnish_runtime.repulsespark_timer() == 0 {
                let mut garnish = self.garnish_state_mut();
                garnish.set_repulsespark_x_lo(pt.y);
                garnish.set_repulsespark_y_lo(pt.x);
                garnish.set_repulsespark_timer(5);
            }
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x5);
        }
    }

    // void HelmasaurKing_AttemptDamage(int k) {  // sprite_main.c:19509
    //   if (!(frame_counter & 7) &&
    //       (uint16)(link_x_coord - cur_sprite_x + 36) < 72 &&
    //       (uint16)(link_y_coord - cur_sprite_y + 40) < 64)
    //     Sprite_AttemptDamageToLinkPlusRecoil(k);
    // }
    pub(super) fn helmasaur_king_attempt_damage(&mut self, k: usize) {
        if (self.game_state.frame.frame_counter & 7) != 0 {
            return;
        }
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        if link_x.wrapping_sub(cur_x).wrapping_add(36) < 72
            && link_y.wrapping_sub(cur_y).wrapping_add(40) < 64
        {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
    }

    // void HelmasaurKing_ChipAwayAtMask(int k) {  // sprite_main.c:19516
    //   tmp_counter = sprite_C[k] + 7;
    //   HelmasaurKing_SpawnMaskDebris(k);
    //   SpriteSfx_QueueSfx2WithPan(k, 0x1f);
    // }
    pub(super) fn helmasaur_king_chip_away_at_mask(&mut self, k: usize) {
        let counter = self.sprite_slot_view(k).c().wrapping_add(7);
        self.temp_counter_mut().set(counter);
        self.helmasaur_king_spawn_mask_debris(k);
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
    }

    // void HelmasaurKing_ExplodeMask(int k) {  // sprite_main.c:19522
    //   for (int j = 1; j < 16; j++)
    //     sprite_state[j] = 0;
    //   tmp_counter = 7;
    //   do {
    //     HelmasaurKing_SpawnMaskDebris(k);
    //   } while (!sign8(--tmp_counter));
    //   SpriteSfx_QueueSfx2WithPan(k, 0x1f);
    // }
    pub(super) fn helmasaur_king_explode_mask(&mut self, k: usize) {
        for j in 1..16 {
            self.sprite_slot_view_mut(j).clear();
        }
        self.temp_counter_mut().set(7);
        loop {
            self.helmasaur_king_spawn_mask_debris(k);
            self.temp_counter_mut().decrement();
            if (self.game_state.scratch_counter.value() as i8) < 0 {
                break;
            }
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
    }

    // void HelmasaurKing_SpawnMaskDebris(int k) {  // sprite_main.c:19532
    //   static const int8 kHelmasaurKing_Mask_X[10] = {-16, 0, 16, -16, 0, 16, -8, 8, -16, 16};
    //   static const int8 kHelmasaurKing_Mask_Y[10] = {24, 27, 24, 24, 27, 24, 27, 27, 24, 24};
    //   static const int8 kHelmasaurKing_Mask_Z[10] = {29, 32, 29, 13, 16, 13, 0, 0, 13, 13};
    //   static const int8 kHelmasaurKing_Mask_Xvel[10] = {-16, -4, 14, -12, 4, 18, -2, 2, -12, 18};
    //   static const int8 kHelmasaurKing_Mask_Yvel[10] = {-8, -4, -6, 4, 2, 7, 6, 8, 4, 7};
    //   static const int8 kHelmasaurKing_Mask_Zvel[10] = {32, 40, 36, 37, 39, 34, 30, 33, 37, 34};
    //   static const uint8 kHelmasaurKing_Mask_OamFlags[10] = {0, 0, 0x40, 0, 0, 0x40, 0, 0x40, 0, 0x40};
    //   static const uint8 kHelmasaurKing_Mask_Gfx[10] = {0, 1, 0, 2, 3, 2, 4, 4, 5, 5};
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x92, &info);
    //   if (j >= 0) {
    //     int i = tmp_counter;
    //     Sprite_SetX(j, info.r0_x + kHelmasaurKing_Mask_X[i]);
    //     Sprite_SetY(j, info.r2_y + kHelmasaurKing_Mask_Y[i]);
    //     sprite_z[j] = kHelmasaurKing_Mask_Z[i];
    //     sprite_x_vel[j] = kHelmasaurKing_Mask_Xvel[i];
    //     sprite_y_vel[j] = kHelmasaurKing_Mask_Yvel[i];
    //     sprite_z_vel[j] = kHelmasaurKing_Mask_Zvel[i];
    //     sprite_oam_flags[j] = kHelmasaurKing_Mask_OamFlags[i] | 13;
    //     sprite_graphics[j] = kHelmasaurKing_Mask_Gfx[i];
    //     sprite_C[j] = 128;
    //     sprite_flags2[j] = 0;
    //     sprite_delay_aux1[j] = 12;
    //     sprite_ignore_projectile[j] = 12;
    //     sprite_subtype[j] = tmp_counter;
    //   }
    // }
    pub(super) fn helmasaur_king_spawn_mask_debris(&mut self, k: usize) {
        if let Some((j, info)) = self.spawn_sprite_dynamically(k, 0x92) {
            let i = self.game_state.scratch_counter.value() as usize;
            self.sprite_set_x(
                j,
                info.r0_x.wrapping_add(HELMASAUR_MASK_DEBRIS_X_OFFSETS[i] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                info.r2_y.wrapping_add(HELMASAUR_MASK_DEBRIS_Y_OFFSETS[i] as i16 as u16),
            );
            let tmp_counter = self.game_state.scratch_counter.value();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(HELMASAUR_MASK_DEBRIS_Z_OFFSETS[i] as u8);
            sprite.set_x_velocity(HELMASAUR_MASK_DEBRIS_X_VELOCITIES[i] as u8);
            sprite.set_y_velocity(HELMASAUR_MASK_DEBRIS_Y_VELOCITIES[i] as u8);
            sprite.set_z_velocity(HELMASAUR_MASK_DEBRIS_Z_VELOCITIES[i] as u8);
            sprite.set_oam_flags(HELMASAUR_MASK_DEBRIS_OAM_FLAGS[i] | 13);
            sprite.set_graphics(HELMASAUR_MASK_DEBRIS_GRAPHICS[i]);
            sprite.set_c(128);
            sprite.set_flags2(0);
            sprite.set_delay_aux1(12);
            sprite.set_ignore_projectile(12);
            sprite.set_subtype(tmp_counter);
        }
    }

    // void HelmasaurKing_SpitFireball(int k) {  // sprite_main.c:19561
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x70, &info);
    //   if (j >= 0) {
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     Sprite_SetY(j, info.info.r2_y + 28);
    //     sprite_delay_main[j] = 32;
    //     sprite_ignore_projectile[j] = 32;
    //   }
    // }
    pub(super) fn helmasaur_king_spit_fireball(&mut self, k: usize) {
        if let Some((j, info)) = self.spawn_sprite_dynamically(k, 0x70) {
            self.sprite_set_spawned_coordinates(j, &info);
            self.sprite_set_y(j, info.r2_y.wrapping_add(28));
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_delay_main(32);
            sprite.set_ignore_projectile(32);
        }
    }

    // void HelmasaurKing_Draw(int k) {  // sprite_main.c:19572
    //   oam_cur_ptr = 0x89c;
    //   oam_ext_cur_ptr = 0xa47;
    //   PrepOamCoordsRet info;
    //   if (Sprite_PrepOamCoordOrDoubleRet(k, &info))
    //     return;
    //   KingHelmasaur_OperateTail(k, &info);
    //   SpriteDraw_KingHelmasaur_Eyes(k, &info);
    //   KingHelmasaurMask(k, &info);
    //   SpriteDraw_KingHelmasaur_Body(k, &info);
    //   SpriteDraw_KingHelmasaur_Legs(k, &info);
    //   SpriteDraw_KingHelmasaur_Mouth(k, &info);
    // }
    pub(super) fn helmasaur_king_draw(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x1e853b));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x1e853b);
        self.oam_state_mut().set_current_pointer(0x89c);
        self.oam_state_mut().set_current_extended_pointer(0xa47);
        let Some(mut info) = self.sprite_prep_oam_coord_or_double_ret_for_helmasaur_king(k) else {
            return;
        };
        b.block(0x1e854c);
        self.king_helmasaur_operate_tail(k, &mut info);
        b.block(0x1e854f);
        self.sprite_draw_king_helmasaur_eyes(k, &info);
        b.block(0x1e8552);
        self.king_helmasaur_mask(k, &mut info);
        b.block(0x1e8555);
        self.sprite_draw_king_helmasaur_body(k, &mut info);
        b.block(0x1e8558);
        self.sprite_draw_king_helmasaur_legs(k, &mut info);
        b.block(0x1e855b);
        self.sprite_draw_king_helmasaur_mouth(k, &info);
        b.block(0x1e855e);
    }

    // void Sprite_ScheduleBossForDeath(int k) {  // sprite_main.c:18047
    pub(super) fn sprite_schedule_boss_for_death(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_state(4);
        sprite.set_a(0);
        sprite.set_delay_main(224);
    }

    // void Sprite_MakeBossExplosion(int k) {  // sprite_main.c:18054
    pub(super) fn sprite_make_boss_explosion(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
        self.sprite_make_boss_death_explosion_no_sound(k);
    }

    // void Sprite_MakeBossDeathExplosion_NoSound(int k) {  // sprite_main.c:18059
    pub(super) fn sprite_make_boss_death_explosion_no_sound(&mut self, k: usize) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x00, &mut info);
        if j < 0 {
            return;
        }
        self.sprite_make_boss_death_explosion_after_spawn(j as usize);
    }

    /// `Sprite_MakeBossDeathExplosion_NoSound` after its dynamic spawn
    /// selected slot `j`: the explosion's graphics request and slot setup.
    pub(super) fn sprite_make_boss_death_explosion_after_spawn(&mut self, j: usize) {
        self.set_chr_halfslot_request(11);
        {
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_state(4);
            sprite.set_flags2(3);
            sprite.set_oam_flags(12);
        }
        self.sprite_set_x(j, self.game_state.sprites.workspace.current_sprite_x());
        self.sprite_set_y(j, self.game_state.sprites.workspace.current_sprite_y());
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_delay_main(31);
        sprite.set_a(31);
        sprite.set_floor(2);
    }

    // -----------------------------------------------------------------
    // Local helper adapters that keep split-module call signatures while
    // routing through canonical helper implementations.
    // -----------------------------------------------------------------

    fn sprite_prep_oam_coord_or_double_ret_for_helmasaur_king(
        &mut self,
        k: usize,
    ) -> Option<PrepOamCoordsRet> {
        self.sprite_prep_oam_coord_or_double_ret_from(k, super::sprite::PrepOamCoordEntry::Bank1eDoubleRet)
            .map(PrepOamCoordsRet::from_tuple)
    }

}

#[cfg(test)]
#[path = "sprite_main_helmasaur_king_tests.rs"]
mod tests;
