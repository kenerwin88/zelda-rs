//! Ported Guard-family handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 4366..5239). The original C body is reproduced as a comment block
//! immediately above each port so a reviewer can verify behavior line-by-line.
//!
//! Local `_for_guard` adapters at the bottom either route to canonical helpers
//! or hold guard-specific shims while the remaining shared helpers are filled in.

use super::*;
use crate::types::PointU8;

// --- Local copies of constant addresses needed here. These mirror what's
// already declared in zelda_rtl.rs / ending.rs but are module-private over
// there, so we duplicate them locally per the round-1 convention.
const OVERWORLD_AREA_INDEX_GUARD: usize = 0x40a;
const K_SPRITE_TUTORIAL_ENTITIES_TAB: [u8; 4] = [2, 1, 0, 3];
const K_SOLDIER_DIRECTION_LOCK_SETTINGS_GUARD: [u8; 4] = [3, 2, 0, 1];

/// Mirror of the C-side `PrepOamCoordsRet` struct (sprite.c). Held by-value
/// because the canonical `sprite::PrepOamCoordsRet` is module-private over
/// in sprite.rs. Fields match the same name/order as the C struct.
#[derive(Clone, Copy)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub flags: u8,
}

impl PrepOamCoordsRet {
    pub(super) fn from_tuple(t: (u16, u16, u8)) -> Self {
        Self {
            x: t.0,
            y: t.1,
            flags: t.2,
        }
    }

    pub(super) fn as_tuple(&self) -> (u16, u16, u8) {
        (self.x, self.y, self.flags)
    }
}

// --- Tables shared by the Guard cluster. Verbatim from
// zelda3/src/sprite_main.c lines 58..288.
const K_SOLDIER_DIRECTION_LOCK_SETTINGS: [u8; 4] = [3, 2, 0, 1];
const K_SOLDIER_GFX: [u8; 4] = [8, 0, 12, 5];
const K_SOLDIER_DELAY: [u8; 4] = [0x60, 0xc0, 0xff, 0x40];
const K_SOLDIER_XVEL: [i8; 4] = [8, -8, 0, 0];
const K_SOLDIER_YVEL: [i8; 4] = [0, 0, 8, -8];
const K_SOLDIER_GFX2: [u8; 32] = [
    11, 12, 13, 12, 4, 5, 6, 5, 0, 1, 2, 3, 7, 8, 9, 10, 17, 18, 17, 18, 7, 8, 7, 8, 3, 4, 3, 4,
    13, 14, 13, 14,
];
const K_SOLDIERB_XVEL: [i8; 8] = [1, 1, -1, -1, -1, -1, 1, 1];
const K_SOLDIERB_YVEL: [i8; 8] = [-1, 1, 1, -1, -1, 1, 1, -1];
const K_SOLDIERB_XVEL2: [i8; 8] = [8, 0, -8, 0, -8, 0, 8, 0];
const K_SOLDIERB_YVEL2: [i8; 8] = [0, 8, 0, -8, 0, 8, 0, -8];
const K_SOLDIERB_DIR: [u8; 8] = [0, 2, 1, 3, 1, 2, 0, 3];
const K_SOLDIERB_MASK2: [u8; 8] = [1, 4, 2, 8, 2, 4, 1, 8];
const K_SOLDIERB_MASK: [u8; 8] = [8, 1, 4, 2, 8, 2, 4, 1];
const K_SOLDIERB_NEXT_B2: [u8; 8] = [1, 2, 3, 0, 5, 6, 7, 4];
const K_SOLDIERB_NEXT_B: [u8; 8] = [3, 0, 1, 2, 7, 4, 5, 6];
const K_SOLDIER_HEAD_DIRS: [u8; 32] = [
    0, 2, 2, 2, 0, 3, 3, 3, 1, 3, 3, 3, 1, 2, 2, 2, 2, 0, 0, 0, 2, 1, 1, 1, 3, 1, 1, 1, 3, 0, 0, 0,
];
const K_SOLDIER_TAB1: [u8; 4] = [13, 13, 12, 12];
const K_SOLDIER_SET_TOWARDS_VEL: [i8; 6] = [14, -14, 0, 0, 14, -14];
const K_SPRITE_SPAWN_PROBE_STAGGERED_TAB: [u8; 4] = [0x10, 0x30, 0, 0x20];
const K_SPRITE_SIMPLIFIED_TILE_ATTR_GUARD: [u8; 256] = [
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];
const K_SPAWN_PROBE_XVEL: [i8; 64] = [
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0, 2, 4, 6, 8, 10,
    12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14, 12, 10, 8, 6, 4, 2,
    0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16, -16,
];
const K_SPAWN_PROBE_YVEL: [i8; 64] = [
    0, 2, 4, 6, 8, 10, 12, 14, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 14,
    12, 10, 8, 6, 4, 2, 0, -2, -4, -6, -8, -10, -12, -14, -16, -16, -16, -16, -16, -16, -16, -16,
    -16, -16, -16, -16, -16, -16, -16, -16, -16, -14, -12, -10, -8, -6, -4, -2, 0,
];
const K_JAVELIN_TROOPER_TAB2: [u8; 64] = [
    25, 25, 24, 24, 23, 23, 23, 23, 19, 19, 18, 18, 17, 17, 17, 17, 16, 16, 15, 15, 14, 14, 14, 14,
    22, 22, 21, 21, 20, 20, 20, 20, 20, 20, 18, 18, 18, 16, 16, 16, 21, 21, 8, 8, 8, 6, 6, 6, 22,
    22, 4, 4, 4, 3, 3, 3, 23, 23, 15, 15, 15, 11, 11, 11,
];
const K_RECRUIT_MOVING_HEAD_DIR: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
const SPRITE_TILETYPE_GUARD: usize = 0x0fa5;
const SPRITE_DELAY_AUX3_GUARD: usize = 0x0ee0;
const SPRITE_Y_RECOIL_GUARD: usize = 0x0f30;

impl ZeldaState {
    // void Sprite_TutorialGuardOrBarrier(int k) {  // 86bffe
    pub(super) fn sprite_tutorial_guard_or_barrier(&mut self, k: usize) {
        if self.ram[SPRITE_TYPE + k] == 0x40 {
            self.sprite_evil_barrier(k);
            return;
        }

        let jbak = self.ram[SPRITE_D + k];
        if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            self.ram[SPRITE_D + k] = K_SOLDIER_DIRECTION_LOCK_SETTINGS_GUARD[usize::from(jbak)];
        }
        self.ram[SPRITE_GRAPHICS + k] =
            K_SPRITE_TUTORIAL_ENTITIES_TAB[usize::from(self.ram[SPRITE_D + k])];
        self.tutorial_soldier_draw(k);
        self.ram[SPRITE_D + k] = jbak;

        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_from_link(k);

        if self.ram[OVERWORLD_AREA_INDEX_GUARD] == 0x1b
            && (self.ram[SPRITE_Y_LO + k] == 0x50 || self.ram[SPRITE_Y_LO + k] == 0x90)
        {
            self.sprite_tutorial_guard_show_message_on_contact(
                k,
                if self.ram[SPRITE_Y_LO + k] == 0x50 {
                    0xb2
                } else {
                    0xb3
                },
            );
        } else if self.sprite_tutorial_guard_show_message_on_contact(
            k,
            u16::from(self.ram[BLIND_HEAD_ANIM_COUNTER]) + 0x0f,
        ) {
            self.ram[BLIND_HEAD_ANIM_COUNTER] = if self.ram[BLIND_HEAD_ANIM_COUNTER] != 6 {
                self.ram[BLIND_HEAD_ANIM_COUNTER].wrapping_add(1)
            } else {
                0
            };
        }
        self.sprite_check_damage_to_and_from_link(k);
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 0x1f) == 0 {
            let jbak = self.ram[SPRITE_D + k];
            self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
            if self.ram[SPRITE_D + k] != jbak && ((self.ram[SPRITE_D + k] ^ jbak) & 2) == 0 {
                self.ram[SPRITE_DELAY_AUX1 + k] = 12;
            }
        }
    }

    // ------------------------------------------------------------------
    // BombGuard / thrown bomb — sprite_main.c:4394..4484
    // ------------------------------------------------------------------
    //
    // void Sprite_4A_BombGuard(int k) { ... }
    pub(super) fn sprite_4_a_bomb_guard(&mut self, k: usize) {
        if self.ram[SPRITE_C + k] == 0 {
            self.bomb_guard(k);
            return;
        }
        if self.ram[SPRITE_C + k] < 2 {
            self.sprite_bomb_explosion_incoming(k);
            return;
        }

        if self.ram[SPRITE_C + k] == 2 {
            for j in (0..=15usize).rev() {
                let cond = (((self.ram[FRAME_COUNTER] ^ j as u8) & 7)
                    | self.ram[SPRITE_HIT_TIMER + j])
                    == 0;
                if j != self.ram[CUR_OBJECT_INDEX] as usize
                    && self.ram[SPRITE_STATE + j] >= 9
                    && cond
                {
                    self.sprite_bomb_check_damage_to_sprite(k, j);
                }
            }
            self.sprite_check_damage_to_link(k);
        }
        self.sprite_draw_sprite_bomb_explosion(k);
        if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void SpriteBomb_CheckDamageToSprite(int k, int j) {  // 85be49
    pub(super) fn sprite_bomb_check_damage_to_sprite(&mut self, k: usize, j: usize) {
        let x = self.sprite_get_x(k).wrapping_sub(16);
        let y = self.sprite_get_y(k).wrapping_sub(16);
        let mut hb = crate::types::SpriteHitBox {
            r0_xlo: x as u8,
            r8_xhi: (x >> 8) as u8,
            r1_ylo: y as u8,
            r9_yhi: (y >> 8) as u8,
            r2: 48,
            r3: 48,
            r4_spr_xlo: 0,
            r10_spr_xhi: 0,
            r5_spr_ylo: 0,
            r11_spr_yhi: 0,
            r6_spr_xsize: 0,
            r7_spr_ysize: 0,
        };
        self.sprite_setup_hit_box(j, &mut hb);
        if !self.check_if_hit_boxes_overlap(&hb) || self.ram[SPRITE_TYPE + j] == 0x11 {
            return;
        }
        self.ancilla_check_damage_to_sprite_preset(j, 8);
        let target_x = self.sprite_get_x(j);
        let target_y = self
            .sprite_get_y(j)
            .wrapping_sub(self.ram[SPRITE_Z + j] as u16);
        let pt = self.sprite_project_speed_towards_location(k, target_x, target_y, 32);
        self.ram[SPRITE_Y_RECOIL_GUARD + j] = pt.y;
        self.ram[SPRITE_X_RECOIL + j] = pt.x;
    }

    // void SpriteBomb_ExplosionIncoming(int k) {  // 85bed3
    pub(super) fn sprite_bomb_explosion_incoming(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] != 0 {
            self.ram[SPRITE_OBJ_PRIO + k] |= 48;
        }
        self.sprite_draw_single_large(k);
        if self.ram[SPRITE_HIT_TIMER + k] != 0 || self.ram[SPRITE_DELAY_AUX1 + k] == 1 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            if self.ram[SPRITE_STATE + k] == 10 {
                self.ram[LINK_STATE_BITS] = 0;
                self.ram[LINK_PICKING_THROW_STATE] = 0;
            }
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
            self.ram[SPRITE_FLAGS4 + k] = 9;
            self.ram[SPRITE_OAM_FLAGS + k] = 2;
            self.ram[SPRITE_DELAY_AUX1 + k] = 31;
            self.ram[SPRITE_STATE + k] = 6;
            self.ram[SPRITE_FLAGS2 + k] = 3;
            return;
        }
        if self.ram[SPRITE_DELAY_AUX1 + k] < 64 {
            self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x0e)
                | ((self.ram[SPRITE_DELAY_AUX1 + k] >> 1) & 0x0e);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_AUX3_GUARD + k] == 0 {
            self.sprite_check_damage_from_link(k);
        }
        self.sprite_move_xy(k);
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.sprite_check_tile_collision(k);
        }
        self.thrown_sprite_tile_and_sprite_interaction(k);
    }

    // ------------------------------------------------------------------
    // Sprite_41_BlueGuard / Probe / Probe_CheckTileSolidity
    // ------------------------------------------------------------------
    //
    // void Sprite_41_BlueGuard(int k) {  // 85c155
    //   if (sprite_C[k]) Probe(k); else Guard_Main(k);
    // }
    pub(super) fn sprite_41_blue_guard(&mut self, k: usize) {
        if self.ram[SPRITE_C + k] != 0 {
            self.probe(k);
        } else {
            self.guard_main(k);
        }
    }

    // void Probe(int k) {  // 85c15d
    //   SpriteAddXY(k, (int8)sprite_x_vel[k], (int8)sprite_y_vel[k]);
    //   ...
    // }
    pub(super) fn probe(&mut self, k: usize) {
        self.sprite_add_xy(
            k,
            self.ram[SPRITE_X_VEL + k] as i8 as i32,
            self.ram[SPRITE_Y_VEL + k] as i8 as i32,
        );

        let parent = self.ram[SPRITE_C + k].wrapping_sub(1) as usize;
        let is_close = if self.ram[SPRITE_TYPE + parent] == 0xce {
            let x = read_le_u16(&self.ram, CUR_SPRITE_X)
                .wrapping_sub(self.player_state_view().x())
                .wrapping_add(16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_Y))
                .wrapping_add(24);
            x < 32 && y < 32
        } else {
            if (self.probe_check_tile_solidity(k) && self.ram[SPRITE_TILETYPE_GUARD] != 9)
                || self.ram[LINK_CAPE_MODE] != 0
            {
                self.ram[SPRITE_STATE + k] = 0;
                return;
            }
            let x = read_le_u16(&self.ram, CUR_SPRITE_X).wrapping_sub(self.player_state_view().x());
            let y = read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_sub(self.player_state_view().y());
            x < 16 && y < 16 && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
        };

        if is_close {
            if self.ram[SPRITE_AI_STATE + parent] != 3 {
                self.ram[SPRITE_AI_STATE + parent] = 3;
                if self.ram[SPRITE_TYPE + parent] != 0xce {
                    self.ram[SPRITE_DELAY_MAIN + parent] = 16;
                    self.ram[SPRITE_SUBTYPE2 + parent] = 0;
                }
            }
            self.ram[SPRITE_STATE + k] = 0;
        } else if let Some((x, y, _flags)) = self.sprite_prep_oam_coord_or_double_ret(k) {
            if (x | y) >= 256 {
                self.ram[SPRITE_STATE + k] = 0;
            }
        }
    }

    // bool Probe_CheckTileSolidity(int k) {  // 8dc26e
    //   ...same tile probe as C, caching sprite_tiletype...
    // }
    pub(super) fn probe_check_tile_solidity(&mut self, k: usize) -> bool {
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let tiletype = if self.ram[PLAYER_IS_INDOORS] != 0 {
            let mut t = if self.ram[SPRITE_FLOOR + k] >= 1 {
                0x1000
            } else {
                0
            };
            t += ((cur_x & 0x01f8) >> 3) as usize;
            t += ((cur_y & 0x01f8) << 3) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE + t]
        } else {
            let t = ((cur_x >> 3).wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_X))
                & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_X))
                | ((cur_y.wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_Y))
                    & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_Y))
                    << 3);
            let map16 = read_le_u16(&self.ram, DUNG_BG2 + ((t >> 1) as usize) * 2);
            self.asset_u8(164, map16 as usize)
        };
        self.ram[SPRITE_TILETYPE_GUARD] = tiletype;
        K_SPRITE_SIMPLIFIED_TILE_ATTR_GUARD[tiletype as usize] >= 1
    }

    // void Guard_SetGlanceTo12(int k) {  // 85c32b
    //   sprite_delay_aux1[k] = 12;
    // }
    pub(super) fn guard_set_glance_to12(&mut self, k: usize) {
        self.ram[SPRITE_DELAY_AUX1 + k] = 12;
    }

    // ------------------------------------------------------------------
    // GreenKnifeGuard_Moving — sprite_main.c:4366
    // ------------------------------------------------------------------
    //
    // void GreenKnifeGuard_Moving(int k) {  // 85bd1e
    //   uint8 t = 0x10;
    //   if (sprite_wallcoll[k] == 0) {
    //     if (sprite_delay_main[k] != 0)
    //       goto out;
    //     t = 0x30;
    //   }
    //   sprite_delay_main[k] = t;
    //   Sprite_ZeroVelocity_XY(k);
    //   sprite_head_dir[k] = kRecruit_Moving_HeadDir[sprite_D[k] * 2 | (GetRandomNumber() & 1)];
    //   sprite_ai_state[k] = 0;
    // out:
    //   sprite_subtype2[k] += (sprite_delay_aux1[k] != 0) ? 2 : 1;
    // }
    pub(super) fn green_knife_guard_moving(&mut self, k: usize) {
        let mut t: u8 = 0x10;
        let do_main = if self.ram[SPRITE_WALLCOLL + k] == 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                false
            } else {
                t = 0x30;
                true
            }
        } else {
            true
        };
        if do_main {
            self.ram[SPRITE_DELAY_MAIN + k] = t;
            self.sprite_zero_velocity_xy_for_guard(k);
            let rnd = self.get_random_number() & 1;
            let idx = ((self.ram[SPRITE_D + k] as usize) * 2) | (rnd as usize);
            self.ram[SPRITE_HEAD_DIR + k] = K_RECRUIT_MOVING_HEAD_DIR[idx & 7];
            self.ram[SPRITE_AI_STATE + k] = 0;
        }
        let inc: u8 = if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            2
        } else {
            1
        };
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(inc);
    }

    // ------------------------------------------------------------------
    // BombGuard_CreateBomb — sprite_main.c:4489
    // ------------------------------------------------------------------
    //
    // void BombGuard_CreateBomb(int k) {  // 85bfc1
    //   static const int8 kBombTrooperBomb_X[4] = {0, 1, 9, -8};
    //   static const int8 kBombTrooperBomb_Y[4] = {-12, -12, -15, -13};
    //   static const int8 kBombTrooperBomb_Zvel[16] = {32, 40, 48, 56, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64};
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x4a, &info);
    //   if (j >= 0) {
    //     int i = sprite_D[k];
    //     Sprite_SetX(j, info.r0_x + kBombTrooperBomb_X[i]);
    //     Sprite_SetY(j, info.r2_y + kBombTrooperBomb_Y[i]);
    //     Sprite_ApplySpeedTowardsLink(j, 16);
    //     PointU8 pt;
    //     sprite_C[j] = 1;
    //     Sprite_DirectionToFaceLink(j, &pt);
    //     if (sign8(pt.x)) pt.x = -pt.x;
    //     if (sign8(pt.y)) pt.y = -pt.y;
    //     sprite_z_vel[j] = kBombTrooperBomb_Zvel[(pt.y | pt.x) >> 4];
    //     sprite_flags3[j] = sprite_flags3[k] & 0xee | 0x18;
    //     sprite_oam_flags[j] = 8;
    //     sprite_delay_aux1[j] = 255;
    //     sprite_health[j] = 0;
    //     SpriteSfx_QueueSfx3WithPan(j, 0x13);
    //   }
    // }
    pub(super) fn bomb_guard_create_bomb(&mut self, k: usize) {
        const K_BOMB_TROOPER_BOMB_X: [i8; 4] = [0, 1, 9, -8];
        const K_BOMB_TROOPER_BOMB_Y: [i8; 4] = [-12, -12, -15, -13];
        const K_BOMB_TROOPER_BOMB_ZVEL: [u8; 16] = [
            32, 40, 48, 56, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        ];
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_guard(k, 0x4a) else {
            return;
        };
        let i = self.ram[SPRITE_D + k] as usize & 3;
        let new_x = r0_x.wrapping_add(K_BOMB_TROOPER_BOMB_X[i] as i16 as u16);
        self.sprite_set_x(j, new_x);
        let new_y = r2_y.wrapping_add(K_BOMB_TROOPER_BOMB_Y[i] as i16 as u16);
        self.sprite_set_y(j, new_y);
        self.sprite_apply_speed_towards_link_for_guard(j, 16);
        self.ram[SPRITE_C + j] = 1;
        let (px, py) = self.sprite_direction_to_face_link_pt_for_guard(j);
        let ax = if (px as i8) < 0 {
            (px as i8).wrapping_neg() as u8
        } else {
            px
        };
        let ay = if (py as i8) < 0 {
            (py as i8).wrapping_neg() as u8
        } else {
            py
        };
        let idx = ((ay | ax) >> 4) as usize & 15;
        self.ram[SPRITE_Z_VEL + j] = K_BOMB_TROOPER_BOMB_ZVEL[idx];
        self.ram[SPRITE_FLAGS3 + j] = (self.ram[SPRITE_FLAGS3 + k] & 0xee) | 0x18;
        self.ram[SPRITE_OAM_FLAGS + j] = 8;
        self.ram[SPRITE_DELAY_AUX1 + j] = 255;
        self.ram[SPRITE_HEALTH + j] = 0;
        self.sprite_sfx_queue_sfx3_with_pan(j, 0x13);
    }

    // ------------------------------------------------------------------
    // void BombGuard(int k) {  // 85bf51
    pub(super) fn bomb_guard(&mut self, k: usize) {
        self.bomb_trooper_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        let dir = self.sprite_direction_to_face_link(k, None);
        self.ram[SPRITE_HEAD_DIR + k] = dir;
        self.ram[SPRITE_D + k] = dir;
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_AI_STATE + k] = 1;
                self.ram[SPRITE_DELAY_MAIN + k] = 112;
            }
        } else {
            let j = self.ram[SPRITE_DELAY_MAIN + k];
            if j == 0 {
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_DELAY_MAIN + k] = 32;
                return;
            }
            self.ram[SPRITE_SUBTYPE2 + k] = u8::from(j >= 80);
            if j == 32 {
                self.bomb_guard_create_bomb(k);
            }
            let base = (usize::from(self.ram[SPRITE_D + k] & 3) << 3) | usize::from(j >> 4);
            self.ram[SPRITE_GRAPHICS + k] = K_JAVELIN_TROOPER_TAB2[(base + 32) & 63];
        }
    }

    // ------------------------------------------------------------------
    // SpriteDraw_BombGuard_Arm — sprite_main.c:4527
    // ------------------------------------------------------------------
    //
    // void SpriteDraw_BombGuard_Arm(int k, PrepOamCoordsRet *info) {  // 85c089
    //   static const int8 kBombTrooper_DrawArm_X[8] = {-1, 1, 2, 0, 9, 9, -8, -8};
    //   static const int8 kBombTrooper_DrawArm_Y[8] = {-12, -12, -12, -12, -16, -14, -12, -14};
    //   OamEnt *oam = GetOamCurPtr();
    //   int j = sprite_D[k] * 2 | sprite_subtype2[k];
    //   SetOamHelper0(oam,
    //                 info->x + kBombTrooper_DrawArm_X[j],
    //                 info->y + kBombTrooper_DrawArm_Y[j], 0x6e, info->flags & 0x30 | 0x8, 2);
    // }
    pub(super) fn sprite_draw_bomb_guard_arm(&mut self, k: usize, info: &PrepOamCoordsRet) {
        const K_BOMB_TROOPER_DRAW_ARM_X: [i8; 8] = [-1, 1, 2, 0, 9, 9, -8, -8];
        const K_BOMB_TROOPER_DRAW_ARM_Y: [i8; 8] = [-12, -12, -12, -12, -16, -14, -12, -14];
        let j = ((self.ram[SPRITE_D + k] as usize) * 2) | (self.ram[SPRITE_SUBTYPE2 + k] as usize);
        let j = j & 7;
        let oam_cur = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let x = info
            .x
            .wrapping_add(K_BOMB_TROOPER_DRAW_ARM_X[j] as i16 as u16);
        let y = info
            .y
            .wrapping_add(K_BOMB_TROOPER_DRAW_ARM_Y[j] as i16 as u16);
        let flags = (info.flags & 0x30) | 0x8;
        self.set_oam_helper0_at(oam_cur, x, y, 0x6e, flags, 2);
    }

    // ------------------------------------------------------------------
    // Guard_Main — sprite_main.c:4600
    // ------------------------------------------------------------------
    //
    // void Guard_Main(int k) {  // 85c227
    //   uint8 bak1 = sprite_graphics[k];
    //   uint8 bak2 = sprite_D[k];
    //   if (sprite_delay_aux1[k]) {
    //     sprite_D[k] = kSoldier_DirectionLockSettings[bak2];
    //     sprite_graphics[k] = kSoldier_Gfx[bak2];
    //   }
    //   Guard_HandleAllAnimation(k);
    //   sprite_D[k] = bak2;
    //   sprite_graphics[k] = bak1;
    //
    //   if (sprite_state[k] == 5) {
    //     if (submodule_index == 0) {
    //       sprite_subtype2[k]++;
    //       Guard_TickAndUpdateBody(k);
    //       sprite_subtype2[k]++;
    //       Guard_TickAndUpdateBody(k);
    //     }
    //     return;
    //   }
    //   if (Sprite_ReturnIfInactive(k)) return;
    //   Guard_ParrySwordAttacks(k);
    //   if ((Sprite_CheckDamageToLink(k) || sprite_alert_flag) && sprite_ai_state[k] < 3) {
    //     sprite_ai_state[k] = 3;
    //     Guard_SetTimerAndAssertTileHitBox(k, 0x20);
    //   } else if (sprite_F[k] != 0 && sprite_F[k] >= 4) {
    //     sprite_ai_state[k] = 4;
    //     Guard_SetTimerAndAssertTileHitBox(k, 0x80);
    //   }
    //   if (Sprite_ReturnIfRecoiling(k)) return;
    //   if ((sprite_subtype[k] & 7) < 5) {
    //     if (!sprite_wallcoll[k]) Sprite_MoveXY(k);
    //     Sprite_CheckTileCollision(k);
    //   } else {
    //     Sprite_MoveXY(k);
    //   }
    //   if (sprite_ai_state[k] != 4) sprite_G[k] = 0;
    //
    //   switch (sprite_ai_state[k]) {
    //   case 0: ...
    //   case 1: ...
    //   case 2: ...
    //   case 3: ...
    //   case 4: ...
    //   }
    // }
    pub(super) fn guard_main(&mut self, k: usize) {
        let bak1 = self.ram[SPRITE_GRAPHICS + k];
        let bak2 = self.ram[SPRITE_D + k];

        if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            self.ram[SPRITE_D + k] = K_SOLDIER_DIRECTION_LOCK_SETTINGS[(bak2 as usize) & 3];
            self.ram[SPRITE_GRAPHICS + k] = K_SOLDIER_GFX[(bak2 as usize) & 3];
        }
        self.guard_handle_all_animation(k);
        self.ram[SPRITE_D + k] = bak2;
        self.ram[SPRITE_GRAPHICS + k] = bak1;

        if self.ram[SPRITE_STATE + k] == 5 {
            if self.frame_control_view().submodule() == 0 {
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.guard_tick_and_update_body(k);
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.guard_tick_and_update_body(k);
            }
            return;
        }
        if self.sprite_return_if_inactive_for_guard(k) {
            return;
        }
        self.guard_parry_sword_attacks_for_guard(k);
        let dmg_link = self.sprite_check_damage_to_link_for_guard(k);
        let alert = self.ram[SPRITE_ALERT_FLAG] != 0;
        if (dmg_link || alert) && self.ram[SPRITE_AI_STATE + k] < 3 {
            self.ram[SPRITE_AI_STATE + k] = 3;
            self.guard_set_timer_and_assert_tile_hit_box(k, 0x20);
        } else if self.ram[SPRITE_F + k] != 0 && self.ram[SPRITE_F + k] >= 4 {
            self.ram[SPRITE_AI_STATE + k] = 4;
            self.guard_set_timer_and_assert_tile_hit_box(k, 0x80);
        }
        if self.sprite_return_if_recoiling_for_guard(k) {
            return;
        }
        if (self.ram[SPRITE_SUBTYPE + k] & 7) < 5 {
            if self.ram[SPRITE_WALLCOLL + k] == 0 {
                self.sprite_move_xy(k);
            }
            self.sprite_check_tile_collision_for_guard(k);
        } else {
            self.sprite_move_xy(k);
        }
        if self.ram[SPRITE_AI_STATE + k] != 4 {
            self.ram[SPRITE_G + k] = 0;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_zero_velocity_xy_for_guard(k);
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    return;
                }
                self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                if self.ram[SPRITE_SUBTYPE + k] != 0 && (self.ram[SPRITE_SUBTYPE + k] & 7) < 5 {
                    let idx = (self.ram[SPRITE_SUBTYPE + k] >> 3) & 3;
                    self.ram[SPRITE_DELAY_MAIN + k] = K_SOLDIER_DELAY[idx as usize];
                    self.ram[SPRITE_D + k] ^= 1;
                    self.ram[SPRITE_SUBTYPE2 + k] = 0;
                } else {
                    let r = self.get_random_number() & 0x3f;
                    self.ram[SPRITE_DELAY_MAIN + k] = r.wrapping_add(0x28);
                    let t = self.ram[SPRITE_D + k];
                    let u = self.get_random_number() & 3;
                    self.ram[SPRITE_D + k] = u;
                    if t == u || ((t ^ u) & 2) != 0 {
                        return;
                    }
                }
                self.ram[SPRITE_DELAY_AUX1 + k] = 12;
            }
            1 => {
                self.sprite_guard_send_out_probe(k);
                if (self.ram[SPRITE_SUBTYPE + k] & 7) >= 5 {
                    self.guard_shoot_probe_and_stuff(k);
                    return;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.sprite_zero_velocity_xy_for_guard(k);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 160;
                    return;
                }
                if (self.ram[SPRITE_SUBTYPE2 + k] & 1) == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        self.ram[SPRITE_DELAY_MAIN + k].wrapping_add(1);
                }
                if (self.ram[SPRITE_WALLCOLL + k] & 0xf) != 0 {
                    self.ram[SPRITE_D + k] ^= 1;
                    self.guard_set_glance_to12(k);
                }
                let dir = self.ram[SPRITE_D + k] as usize & 3;
                self.ram[SPRITE_X_VEL + k] = K_SOLDIER_XVEL[dir] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_SOLDIER_YVEL[dir] as u8;
                self.ram[SPRITE_HEAD_DIR + k] = dir as u8;
                self.guard_tick_and_update_body(k);
            }
            2 => {
                self.sprite_zero_velocity_xy_for_guard(k);
                self.sprite_guard_send_out_probe(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 0x20;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                } else if self.ram[SPRITE_DELAY_MAIN + k] < 0x80 {
                    let t = ((self.ram[SPRITE_D + k] as usize) * 8)
                        | ((self.ram[SPRITE_DELAY_MAIN + k] as usize >> 3) & 7);
                    self.ram[SPRITE_HEAD_DIR + k] = K_SOLDIER_HEAD_DIRS[t & 31];
                }
            }
            3 => {
                self.sprite_zero_velocity_xy_for_guard(k);
                self.ram[SPRITE_HEAD_DIR + k] = self.sprite_direction_to_face_link_for_guard(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.guard_set_timer_and_assert_tile_hit_box(k, 255);
                }
            }
            4 => {
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.soldier_func12_for_guard(k);
                } else {
                    let dir = self.ram[SPRITE_D + k] as usize & 3;
                    self.ram[SPRITE_ANIM_CLOCK + k] = K_SOLDIER_TAB1[dir];
                    self.sprite_zero_velocity_xy_for_guard(k);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 160;
                }
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // Guard_ShootProbeAndStuff — sprite_main.c:4724
    // ------------------------------------------------------------------
    //
    // void Guard_ShootProbeAndStuff(int k) {  // 85c3a1
    //   int i = sprite_B[k];
    //   sprite_x_vel[k] = kSoldierB_Xvel[i];
    //   sprite_y_vel[k] = kSoldierB_Yvel[i];
    //   Sprite_CheckTileCollision(k);
    //   if (sprite_delay_aux2[k]) {
    //     if (sprite_delay_aux2[k] == 44) sprite_B[k] = i = kSoldierB_NextB[i];
    //   } else if (!(sprite_wallcoll[k] & kSoldierB_Mask[i])) {
    //     sprite_delay_aux2[k] = 88;
    //   }
    //   if (sprite_wallcoll[k] & kSoldierB_Mask2[i])
    //     sprite_B[k] = i = kSoldierB_NextB2[i];
    //   sprite_x_vel[k] = kSoldierB_Xvel2[i];
    //   sprite_y_vel[k] = kSoldierB_Yvel2[i];
    //   sprite_head_dir[k] = sprite_D[k] = kSoldierB_Dir[i];
    //   Guard_TickAndUpdateBody(k);
    // }
    pub(super) fn guard_shoot_probe_and_stuff(&mut self, k: usize) {
        let mut i = (self.ram[SPRITE_B + k] as usize) & 7;
        self.ram[SPRITE_X_VEL + k] = K_SOLDIERB_XVEL[i] as u8;
        self.ram[SPRITE_Y_VEL + k] = K_SOLDIERB_YVEL[i] as u8;
        self.sprite_check_tile_collision_for_guard(k);
        if self.ram[SPRITE_DELAY_AUX2 + k] != 0 {
            if self.ram[SPRITE_DELAY_AUX2 + k] == 44 {
                let n = K_SOLDIERB_NEXT_B[i];
                self.ram[SPRITE_B + k] = n;
                i = n as usize & 7;
            }
        } else if (self.ram[SPRITE_WALLCOLL + k] & K_SOLDIERB_MASK[i]) == 0 {
            self.ram[SPRITE_DELAY_AUX2 + k] = 88;
        }
        if (self.ram[SPRITE_WALLCOLL + k] & K_SOLDIERB_MASK2[i]) != 0 {
            let n = K_SOLDIERB_NEXT_B2[i];
            self.ram[SPRITE_B + k] = n;
            i = n as usize & 7;
        }
        self.ram[SPRITE_X_VEL + k] = K_SOLDIERB_XVEL2[i] as u8;
        self.ram[SPRITE_Y_VEL + k] = K_SOLDIERB_YVEL2[i] as u8;
        let dir = K_SOLDIERB_DIR[i];
        self.ram[SPRITE_D + k] = dir;
        self.ram[SPRITE_HEAD_DIR + k] = dir;
        self.guard_tick_and_update_body(k);
    }

    // ------------------------------------------------------------------
    // Guard_TickAndUpdateBody — sprite_main.c:4743
    // ------------------------------------------------------------------
    //
    // void Guard_TickAndUpdateBody(int k) {  // 85c454
    //   sprite_subtype2[k]++;
    //   int t = sprite_D[k] * 4 + (sprite_subtype2[k] >> 3 & 3);
    //   sprite_graphics[k] = kSoldier_Gfx2[t];
    // }
    pub(super) fn guard_tick_and_update_body(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        let t = (self.ram[SPRITE_D + k] as usize).wrapping_mul(4)
            + ((self.ram[SPRITE_SUBTYPE2 + k] as usize >> 3) & 3);
        self.ram[SPRITE_GRAPHICS + k] = K_SOLDIER_GFX2[t & 31];
    }

    // ------------------------------------------------------------------
    // Guard_SetTimerAndAssertTileHitBox — sprite_main.c:4749
    // ------------------------------------------------------------------
    //
    // void Guard_SetTimerAndAssertTileHitBox(int k, uint8 a) {  // 85c4d7
    //   sprite_delay_main[k] = a;
    //   sprite_subtype[k] = 0;
    //   sprite_flags[k] = sprite_flags[k] & 0xf | 0x60;
    // }
    pub(super) fn guard_set_timer_and_assert_tile_hit_box(&mut self, k: usize, a: u8) {
        self.ram[SPRITE_DELAY_MAIN + k] = a;
        self.ram[SPRITE_SUBTYPE + k] = 0;
        self.ram[SPRITE_FLAGS + k] = (self.ram[SPRITE_FLAGS + k] & 0xf) | 0x60;
    }

    // ------------------------------------------------------------------
    // Guard_ApplySpeedInDirection — sprite_main.c:4769
    // ------------------------------------------------------------------
    //
    // void Guard_ApplySpeedInDirection(int k) {  // 85c542
    //   if (!sprite_wallcoll[k]) return;
    //   int i;
    //   if (sprite_wallcoll[k] & 3) {
    //     i = 2 + Sprite_IsBelowLink(k).a;
    //   } else {
    //     i = Sprite_IsRightOfLink(k).a;
    //   }
    //   sprite_x_vel[k] = kSoldier_SetTowardsVel[i];
    //   sprite_y_vel[k] = kSoldier_SetTowardsVel[i + 2];
    // }
    pub(super) fn guard_apply_speed_in_direction(&mut self, k: usize) {
        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            return;
        }
        let i: usize = if self.ram[SPRITE_WALLCOLL + k] & 3 != 0 {
            2 + self.sprite_is_below_link(k).a as usize
        } else {
            self.sprite_is_right_of_link(k).a as usize
        };
        let i = i & 7;
        self.ram[SPRITE_X_VEL + k] = K_SOLDIER_SET_TOWARDS_VEL[i.min(5)] as u8;
        self.ram[SPRITE_Y_VEL + k] = K_SOLDIER_SET_TOWARDS_VEL[(i + 2).min(5)] as u8;
    }

    // ------------------------------------------------------------------
    // Sprite_Guard_SendOutProbe — sprite_main.c:4782
    // ------------------------------------------------------------------
    //
    // void Sprite_Guard_SendOutProbe(int k) {  // 85c5f2
    //   if ((k + frame_counter & 3) | sprite_pause[k]) return;
    //   uint8 a = sprite_anim_clock[k]++;
    //   uint8 r15 = ((a & 0x1f) + kSprite_SpawnProbeStaggered_Tab[sprite_D[k]]) & 0x3f;
    //   Sprite_SpawnProbeAlways(k, r15);
    // }
    pub(super) fn sprite_guard_send_out_probe(&mut self, k: usize) {
        let lo = (k as u8).wrapping_add(self.ram[FRAME_COUNTER]) & 3;
        if lo != 0 || self.ram[SPRITE_PAUSE + k] != 0 {
            return;
        }
        let a = self.ram[SPRITE_ANIM_CLOCK + k];
        self.ram[SPRITE_ANIM_CLOCK + k] = a.wrapping_add(1);
        let dir = self.ram[SPRITE_D + k] as usize & 3;
        let r15 = (a & 0x1f).wrapping_add(K_SPRITE_SPAWN_PROBE_STAGGERED_TAB[dir]) & 0x3f;
        self.sprite_spawn_probe_always_for_guard(k, r15);
    }

    // ------------------------------------------------------------------
    // Guard_AnimateHead / Body / Weapon
    // ------------------------------------------------------------------
    //
    // These three C functions are already implemented as
    // `ZeldaState::guard_animate_head`, `guard_animate_body`,
    // `guard_animate_weapon` in attract.rs (where they were ported in
    // service of the attract-mode soldier simulation). The bodies match
    // sprite_main.c lines 4823 / 4830 / 4852 byte-for-byte; the only
    // difference is the Rust signature uses a `(u16, u16, u8)` tuple in
    // place of the C `PrepOamCoordsRet *poc` — which makes the
    // signature_drift regex skip them today. Re-porting under different
    // names would create duplicate `impl ZeldaState` methods, so we leave
    // those canonical methods in attract.rs and reference them from
    // `Guard_HandleAllAnimation` (sprite_main.rs).

    // ------------------------------------------------------------------
    // BoltGuard_TriggerChaseTheme — sprite_main.c:4887
    // ------------------------------------------------------------------
    //
    // void BoltGuard_TriggerChaseTheme(int k) {  // 85cc3c
    //   if (sprite_G[k] != 16 && sprite_G[k]++ == 15) {
    //     SpriteSfx_QueueSfx3WithPan(k, 0x4);
    //     if (sram_progress_indicator == 2 && BYTE(overworld_area_index) == 24)
    //       music_control = 12;
    //   }
    // }
    pub(super) fn bolt_guard_trigger_chase_theme(&mut self, k: usize) {
        if self.ram[SPRITE_G + k] != 16 {
            let old = self.ram[SPRITE_G + k];
            self.ram[SPRITE_G + k] = old.wrapping_add(1);
            if old == 15 {
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x4);
                let area_lo = self.ram[OVERWORLD_AREA_INDEX_GUARD];
                if self.ram[SRAM_PROGRESS_INDICATOR] == 2 && area_lo == 24 {
                    self.ram[MUSIC_CONTROL] = 12;
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Guard_LaunchProjectile — sprite_main.c:5090
    // ------------------------------------------------------------------
    //
    // void Guard_LaunchProjectile(int k) {  // 85d0c5
    //   static const int8 kJavelinProjectile_X[8] = {16, -8, 3, 11, 12, -4, 12, -4};
    //   static const int8 kJavelinProjectile_Y[8] = {2, 2, 16, -8, -2, -2, 2, -8};
    //   static const int8 kJavelinProjectile_Xvel[8] = {48, -48, 0, 0, 32, -32, 0, 0};
    //   static const int8 kJavelinProjectile_Yvel[8] = {0, 0, 48, -48, 0, 0, 32, -32};
    //   static const uint8 kJavelinProjectile_Flags4[4] = {5, 5, 6, 6};
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x1b, &info);
    //   if (j < 0) return;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x5);
    //   int i = sprite_D[k] + (sprite_type[k] >= 0x48 ? 4 : 0);
    //   Sprite_SetX(j, info.r0_x + kJavelinProjectile_X[i]);
    //   Sprite_SetY(j, info.r2_y + kJavelinProjectile_Y[i]);
    //   sprite_x_vel[j] = kJavelinProjectile_Xvel[i];
    //   sprite_y_vel[j] = kJavelinProjectile_Yvel[i];
    //   i &= 3;
    //   sprite_D[j] = i;
    //   sprite_flags4[j] = kJavelinProjectile_Flags4[i];
    //   sprite_z[j] = 0;
    //   sprite_A[j] = (sprite_type[k] >= 0x48);
    //   if (sprite_A[j] && link_shield_type == 0)
    //     sprite_flags5[j] &= ~0x20;
    // }
    pub(super) fn guard_launch_projectile(&mut self, k: usize) {
        const K_JAVELIN_PROJECTILE_X: [i8; 8] = [16, -8, 3, 11, 12, -4, 12, -4];
        const K_JAVELIN_PROJECTILE_Y: [i8; 8] = [2, 2, 16, -8, -2, -2, 2, -8];
        const K_JAVELIN_PROJECTILE_XVEL: [i8; 8] = [48, -48, 0, 0, 32, -32, 0, 0];
        const K_JAVELIN_PROJECTILE_YVEL: [i8; 8] = [0, 0, 48, -48, 0, 0, 32, -32];
        const K_JAVELIN_PROJECTILE_FLAGS4: [u8; 4] = [5, 5, 6, 6];

        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_guard(k, 0x1b) else {
            return;
        };
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x5);
        let big = self.ram[SPRITE_TYPE + k] >= 0x48;
        let mut i = (self.ram[SPRITE_D + k] as usize) + if big { 4 } else { 0 };
        i &= 7;
        let new_x = r0_x.wrapping_add(K_JAVELIN_PROJECTILE_X[i] as i16 as u16);
        self.sprite_set_x(j, new_x);
        let new_y = r2_y.wrapping_add(K_JAVELIN_PROJECTILE_Y[i] as i16 as u16);
        self.sprite_set_y(j, new_y);
        self.ram[SPRITE_X_VEL + j] = K_JAVELIN_PROJECTILE_XVEL[i] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_JAVELIN_PROJECTILE_YVEL[i] as u8;
        let i_low = i & 3;
        self.ram[SPRITE_D + j] = i_low as u8;
        self.ram[SPRITE_FLAGS4 + j] = K_JAVELIN_PROJECTILE_FLAGS4[i_low];
        self.ram[SPRITE_Z + j] = 0;
        let a_val: u8 = if big { 1 } else { 0 };
        self.ram[SPRITE_A + j] = a_val;
        if a_val != 0 && self.ram[LINK_SHIELD_TYPE] == 0 {
            self.ram[SPRITE_FLAGS5 + j] &= !0x20;
        }
    }

    // ------------------------------------------------------------------
    // Sprite_BushGuard_Main — sprite_main.c:5173
    // ------------------------------------------------------------------
    //
    // void Sprite_BushGuard_Main(int k) {  // 85d1d3
    //   int j;
    //   static const uint8 kBushSoldier_Gfx[32] = {
    //     4, 4, 4, 4, 4, 4, 4, 4, 0, 1, 0, 1, 0, 1, 0, 1,
    //     0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
    //   };
    //   static const uint8 kBushSoldier_Gfx2[16] = {0, 1, 0, 1, 0, 1, 0, 1, 0, 2, 3, 4, 4, 4, 4, 4};
    //   if (Sprite_ReturnIfInactive(k)) return;
    //   sprite_ignore_projectile[k] = 1;
    //   switch (sprite_ai_state[k]) {
    //   case 0: ...
    //   case 1: ...
    //   case 2: ...
    //   case 3: ...
    //   }
    // }
    pub(super) fn sprite_bush_guard_main(&mut self, k: usize) {
        const K_BUSH_SOLDIER_GFX: [u8; 32] = [
            4, 4, 4, 4, 4, 4, 4, 4, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
            1, 0, 1,
        ];
        const K_BUSH_SOLDIER_GFX2: [u8; 16] = [0, 1, 0, 1, 0, 1, 0, 1, 0, 2, 3, 4, 4, 4, 4, 4];
        if self.sprite_return_if_inactive_for_guard(k) {
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;

        // case 3 / case_3 fallthrough handler
        let do_case_3 = |state: &mut ZeldaState, k: usize| {
            state.sprite_check_damage_to_and_from_link_for_guard(k);
            if state.ram[SPRITE_DELAY_MAIN + k] == 0 {
                state.ram[SPRITE_AI_STATE + k] = 0;
                state.ram[SPRITE_DELAY_MAIN + k] = 64;
            } else {
                let idx = state.ram[SPRITE_DELAY_MAIN + k] as usize >> 2;
                state.ram[SPRITE_GRAPHICS + k] = K_BUSH_SOLDIER_GFX2[idx & 15];
            }
        };

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                }
            }
            1 => {
                self.sprite_check_damage_from_link_for_guard(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                    let face = self.sprite_direction_to_face_link_for_guard(k);
                    self.ram[SPRITE_D + k] = face;
                    self.ram[SPRITE_HEAD_DIR + k] = face;
                } else {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0x20 {
                        self.bush_guard_spawn_foliage(k);
                    }
                    let idx = self.ram[SPRITE_DELAY_MAIN + k] as usize >> 2;
                    self.ram[SPRITE_GRAPHICS + k] = K_BUSH_SOLDIER_GFX[idx & 31];
                }
            }
            2 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_to_and_from_link_for_guard(k);
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                    do_case_3(self, k);
                } else {
                    self.ram[SPRITE_A + k] = if j < 40 { 0xff } else { 0x00 };
                    if j == 16 {
                        self.guard_launch_projectile(k);
                    }
                    let base = (self.ram[SPRITE_D + k] as usize) * 8
                        + ((j as usize) >> 3)
                        + if self.ram[SPRITE_TYPE + k] == 0x49 {
                            32
                        } else {
                            0
                        };
                    self.ram[SPRITE_GRAPHICS + k] = K_JAVELIN_TROOPER_TAB2[base & 63];
                }
            }
            3 => {
                do_case_3(self, k);
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------------
    // BushGuard_SpawnFoliage — sprite_main.c:5229
    // ------------------------------------------------------------------
    //
    // void BushGuard_SpawnFoliage(int k) {  // 85d252
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0xec, &info);
    //   if (j < 0) return;
    //   Sprite_SetSpawnedCoordinates(j, &info);
    //   sprite_state[j] = 6;
    //   sprite_delay_main[j] = 32;
    //   sprite_flags2[j] += 3;
    //   sprite_C[j] = 2;
    // }
    pub(super) fn bush_guard_spawn_foliage(&mut self, k: usize) {
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_guard(k, 0xec) else {
            return;
        };
        self.sprite_set_spawned_coordinates_for_guard(j, r0_x, r2_y);
        self.ram[SPRITE_STATE + j] = 6;
        self.ram[SPRITE_DELAY_MAIN + j] = 32;
        self.ram[SPRITE_FLAGS2 + j] = self.ram[SPRITE_FLAGS2 + j].wrapping_add(3);
        self.ram[SPRITE_C + j] = 2;
    }

    // -----------------------------------------------------------------
    // Internal helpers — named with `_for_guard` suffix to keep this
    // translation unit close to the original C call sites.
    // -----------------------------------------------------------------

    // Rewired to canonical Sprite_ZeroVelocity_XY port.
    fn sprite_zero_velocity_xy_for_guard(&mut self, k: usize) {
        self.sprite_zero_velocity_xy(k);
    }

    // Rewired to canonical Sprite_ReturnIfInactive port.
    fn sprite_return_if_inactive_for_guard(&mut self, k: usize) -> bool {
        self.sprite_return_if_inactive(k)
    }

    // Rewired to canonical Sprite_ReturnIfRecoiling port.
    fn sprite_return_if_recoiling_for_guard(&mut self, k: usize) -> bool {
        self.sprite_return_if_recoiling(k)
    }

    // Rewired to canonical Sprite_CheckDamageToLink port.
    fn sprite_check_damage_to_link_for_guard(&mut self, k: usize) -> bool {
        self.sprite_check_damage_to_link(k)
    }

    // Rewired to canonical Sprite_CheckDamageFromLink entry point.
    fn sprite_check_damage_from_link_for_guard(&mut self, k: usize) {
        let _ = self.sprite_check_damage_from_link(k);
    }

    // Rewired to canonical Sprite_CheckDamageToAndFromLink port.
    fn sprite_check_damage_to_and_from_link_for_guard(&mut self, k: usize) {
        self.sprite_check_damage_to_and_from_link(k);
    }

    // Rewired to canonical Sprite_CheckTileCollision port.
    fn sprite_check_tile_collision_for_guard(&mut self, k: usize) {
        let _ = self.sprite_check_tile_collision(k);
    }

    // Rewired to canonical Guard_ParrySwordAttacks port.
    fn guard_parry_sword_attacks_for_guard(&mut self, k: usize) {
        self.guard_parry_sword_attacks(k);
    }

    // Rewired to canonical Sprite_DirectionToFaceLink port.
    fn sprite_direction_to_face_link_for_guard(&mut self, k: usize) -> u8 {
        self.sprite_direction_to_face_link(k, None)
    }

    // Returns the (x,y) byte pair from `Sprite_DirectionToFaceLink(j, &pt)`.
    fn sprite_direction_to_face_link_pt_for_guard(&mut self, k: usize) -> (u8, u8) {
        let mut pt = PointU8 { x: 0, y: 0 };
        let _ = self.sprite_direction_to_face_link(k, Some(&mut pt));
        (pt.x, pt.y)
    }

    // Rewired to canonical Sprite_ApplySpeedTowardsLink port.
    fn sprite_apply_speed_towards_link_for_guard(&mut self, k: usize, speed: u8) {
        self.sprite_apply_speed_towards_link(k, speed);
    }

    // Soldier_Func12 — proxy. Calls speed-toward-link/animation step.
    fn soldier_func12_for_guard(&mut self, k: usize) {
        if ((k as u8) ^ self.ram[FRAME_COUNTER]) & 0x1f == 0 {
            if self.ram[SPRITE_G + k] == 0 {
                self.ram[SPRITE_G + k] = 1;
                self.sprite_sfx_queue_sfx3_with_pan(k, 4);
            }
            self.sprite_apply_speed_towards_link_for_guard(k, 16);
            let face = self.sprite_direction_to_face_link_for_guard(k);
            self.ram[SPRITE_D + k] = face;
            self.ram[SPRITE_HEAD_DIR + k] = face;
        }
        self.guard_apply_speed_in_direction(k);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.guard_tick_and_update_body(k);
    }

    // ------------------------------------------------------------------
    // void Soldier_Func12(int k) {  // 85c500
    pub(super) fn soldier_func12(&mut self, k: usize) {
        self.soldier_func12_for_guard(k);
    }

    fn soldier_throwing_agitated_step(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if (self.ram[SPRITE_SUBTYPE2 + k] & 0x0f) == 0 {
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
            if self.ram[SPRITE_A + k] == 2 {
                self.ram[SPRITE_A + k] = 0;
            }
        }
        let base = usize::from(self.ram[SPRITE_D + k] & 3) * 4
            + usize::from(self.ram[SPRITE_A + k])
            + if self.ram[SPRITE_TYPE + k] == 0x48 {
                16
            } else {
                0
            };
        self.ram[SPRITE_GRAPHICS + k] = K_SOLDIER_GFX2[base & 31];
    }

    // ------------------------------------------------------------------
    // void SoldierThrowing_Common(int k) {  // 85ce23
    pub(super) fn soldier_throwing_common(&mut self, k: usize) {
        const DIR_FLAGS: [u8; 4] = [3, 3, 12, 12];
        const XD: [i8; 8] = [-80, 80, 0, -8, -80, 80, -8, 8];
        const YD: [i8; 8] = [8, 8, -80, 80, 8, 8, -80, 80];

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if (self.sprite_check_damage_to_and_from_link(k) || self.ram[SPRITE_ALERT_FLAG] != 0)
            && self.ram[SPRITE_AI_STATE + k] < 3
        {
            self.ram[SPRITE_AI_STATE + k] = 3;
            self.ram[SPRITE_DELAY_MAIN + k] = 32;
        }
        if self.ram[SPRITE_F + k] >= 4 {
            self.ram[SPRITE_AI_STATE + k] = 4;
            self.ram[SPRITE_DELAY_MAIN + k] = 60;
            self.ram[SPRITE_SUBTYPE2 + k] = 0;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_zero_velocity_xy(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        0x50u8.wrapping_add(self.get_random_number() & 0x7f);
                    let jbak = self.ram[SPRITE_D + k];
                    self.ram[SPRITE_D + k] = self.get_random_number() & 3;
                    if self.ram[SPRITE_D + k] != jbak && ((self.ram[SPRITE_D + k] ^ jbak) & 2) == 0
                    {
                        self.ram[SPRITE_DELAY_AUX1 + k] = 12;
                    }
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 160;
                    return;
                }
                self.sprite_guard_send_out_probe(k);
                if (self.ram[SPRITE_WALLCOLL + k] & 0x0f) != 0 {
                    self.ram[SPRITE_D + k] ^= 1;
                    self.guard_set_glance_to12(k);
                }
                let j = usize::from(self.ram[SPRITE_D + k] & 3);
                self.ram[SPRITE_X_VEL + k] = K_SOLDIER_XVEL[j] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_SOLDIER_YVEL[j] as u8;
                self.ram[SPRITE_HEAD_DIR + k] = j as u8;
                self.soldier_throwing_agitated_step(k);
            }
            2 => {
                self.sprite_zero_velocity_xy(k);
                self.sprite_guard_send_out_probe(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                } else if self.ram[SPRITE_DELAY_MAIN + k] < 0x80 {
                    let t = usize::from(self.ram[SPRITE_D + k] & 3) * 8
                        | usize::from((self.ram[SPRITE_DELAY_MAIN + k] >> 3) & 7);
                    self.ram[SPRITE_HEAD_DIR + k] = K_SOLDIER_HEAD_DIRS[t & 31];
                }
            }
            3 => {
                self.sprite_zero_velocity_xy(k);
                self.ram[SPRITE_HEAD_DIR + k] = self.sprite_direction_to_face_link(k, None);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[SPRITE_DELAY_MAIN + k] = 60;
                    self.ram[SPRITE_SUBTYPE2 + k] = 0;
                }
            }
            4 => {
                let mut j = usize::from(self.ram[SPRITE_D + k] & 3);
                if (self.ram[SPRITE_WALLCOLL + k] & DIR_FLAGS[j]) != 0
                    || self.ram[SPRITE_DELAY_MAIN + k] == 0
                {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 24;
                    return;
                }
                if ((self.ram[FRAME_COUNTER] ^ (k as u8)) & 7) == 0 {
                    let dir = self.sprite_direction_to_face_link(k, None);
                    self.ram[SPRITE_D + k] = dir;
                    self.ram[SPRITE_HEAD_DIR + k] = dir;
                    j = usize::from(dir & 3);
                    if self.ram[SPRITE_TYPE + k] == 0x48 {
                        j += 4;
                    }
                    let x = self
                        .player_state_view()
                        .x()
                        .wrapping_add(XD[j] as i16 as u16);
                    let y = self
                        .player_state_view()
                        .y()
                        .wrapping_add(YD[j] as i16 as u16);
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 24);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;
                    if pt.xdiff.wrapping_add(6) < 12 && pt.ydiff.wrapping_add(6) < 12 {
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_DELAY_MAIN + k] = 24;
                        return;
                    }
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.soldier_throwing_agitated_step(k);
            }
            5 => {
                let dir = usize::from(self.ram[SPRITE_D + k] & 3);
                self.ram[SPRITE_ANIM_CLOCK + k] = K_SOLDIER_TAB1[dir];
                self.sprite_zero_velocity_xy(k);
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 160;
                    return;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = if j >= 40 { 255 } else { 0 };
                if j == 12 {
                    self.guard_launch_projectile(k);
                }
                let base = dir * 8
                    + usize::from(j >> 3)
                    + if self.ram[SPRITE_TYPE + k] == 0x48 {
                        32
                    } else {
                        0
                    };
                self.ram[SPRITE_GRAPHICS + k] = K_JAVELIN_TROOPER_TAB2[base & 63];
            }
            _ => {}
        }
    }

    // Rewired to canonical Sprite_SpawnDynamically port.
    fn sprite_spawn_dynamically_for_guard(
        &mut self,
        k: usize,
        what: u8,
    ) -> Option<(usize, u16, u16)> {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, what, &mut info);
        if j < 0 {
            None
        } else {
            Some((j as usize, info.r0_x, info.r2_y))
        }
    }

    // Rewired to canonical Sprite_SetSpawnedCoordinates port.
    fn sprite_set_spawned_coordinates_for_guard(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    // Sprite_SpawnProbeAlways — sprite_main.c:4790.
    fn sprite_spawn_probe_always_for_guard(&mut self, k: usize, r15: u8) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, 0x41, &mut info, 10);
        if j < 0 {
            return;
        }
        let j = j as usize;
        {
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_x(info.r0_x.wrapping_add(8));
            sprite.set_y(info.r2_y.wrapping_add(4));
        }
        self.ram[SPRITE_D + j] = r15;
        let idx = r15 as usize & 63;
        self.ram[SPRITE_X_VEL + j] = K_SPAWN_PROBE_XVEL[idx] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_SPAWN_PROBE_YVEL[idx] as u8;
        self.ram[SPRITE_FLAGS2 + j] = (self.ram[SPRITE_FLAGS2 + j] & 0xf0) | 0xa0;
        self.ram[SPRITE_C + j] = (k as u8).wrapping_add(1);
        self.ram[SPRITE_IGNORE_PROJECTILE + j] = (k as u8).wrapping_add(1);
        self.ram[SPRITE_FLAGS4 + j] = 0x40;
        self.ram[SPRITE_FLAGS3 + j] = 0x40;
        self.ram[SPRITE_DEFL_BITS + j] = 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    #[test]
    fn guard_tick_and_update_body_advances_subtype_and_writes_gfx() {
        // sprite_subtype2 ++; t = sprite_D * 4 + (sprite_subtype2 >> 3 & 3);
        // sprite_graphics = kSoldier_Gfx2[t].
        let mut state = fresh_state();
        let k = 4;
        state.ram[SPRITE_D + k] = 2; // base index 8
        state.ram[SPRITE_SUBTYPE2 + k] = 0x10; // pre-incr -> 0x11 -> shift>>3 = 2 -> &3 = 2
        state.guard_tick_and_update_body(k);
        assert_eq!(state.ram[SPRITE_SUBTYPE2 + k], 0x11);
        // t = 2*4 + 2 = 10 -> kSoldier_Gfx2[10] = 2
        assert_eq!(state.ram[SPRITE_GRAPHICS + k], 2);
    }

    #[test]
    fn guard_set_timer_writes_main_subtype_and_flags() {
        // sprite_delay_main = a; sprite_subtype = 0;
        // sprite_flags = (sprite_flags & 0xf) | 0x60.
        let mut state = fresh_state();
        let k = 7;
        state.ram[SPRITE_FLAGS + k] = 0xab;
        state.ram[SPRITE_SUBTYPE + k] = 0x55;
        state.guard_set_timer_and_assert_tile_hit_box(k, 0x7f);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + k], 0x7f);
        assert_eq!(state.ram[SPRITE_SUBTYPE + k], 0);
        assert_eq!(state.ram[SPRITE_FLAGS + k], (0xab & 0xf) | 0x60);
    }

    #[test]
    fn green_knife_guard_moving_resets_when_wallcoll() {
        // wallcoll != 0 -> takes main branch with t = 0x10, zero velocity,
        // set head_dir from table, reset ai_state, and bump subtype2 by
        // (delay_aux1?2:1).
        let mut state = fresh_state();
        let k = 0;
        state.ram[SPRITE_WALLCOLL + k] = 0x01;
        state.ram[SPRITE_D + k] = 2; // table index base = 4
        state.ram[SPRITE_DELAY_AUX1 + k] = 0; // increment by 1
        state.ram[SPRITE_SUBTYPE2 + k] = 0x10;
        state.ram[SPRITE_X_VEL + k] = 12;
        state.ram[SPRITE_Y_VEL + k] = 33;
        state.ram[SPRITE_AI_STATE + k] = 9;

        state.green_knife_guard_moving(k);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + k], 0x10);
        assert_eq!(state.ram[SPRITE_X_VEL + k], 0);
        assert_eq!(state.ram[SPRITE_Y_VEL + k], 0);
        assert_eq!(state.ram[SPRITE_AI_STATE + k], 0);
        // get_random_number called once; rnd&1 in 0..=1, idx in 4..=5,
        // table at idx 4=0 or idx 5=1. We accept either; both ∈ {0,1}.
        let hd = state.ram[SPRITE_HEAD_DIR + k];
        assert!(hd == 0 || hd == 1);
        assert_eq!(state.ram[SPRITE_SUBTYPE2 + k], 0x11);
    }

    #[test]
    fn green_knife_guard_moving_skips_main_when_delay_active() {
        // wallcoll == 0 and delay_main != 0 -> jump to "out".
        // Expect: delay_main unchanged, ai_state unchanged, head_dir unchanged,
        // subtype2 incremented.
        let mut state = fresh_state();
        let k = 2;
        state.ram[SPRITE_WALLCOLL + k] = 0;
        state.ram[SPRITE_DELAY_MAIN + k] = 0x20;
        state.ram[SPRITE_AI_STATE + k] = 1;
        state.ram[SPRITE_HEAD_DIR + k] = 9;
        state.ram[SPRITE_SUBTYPE2 + k] = 5;
        state.ram[SPRITE_DELAY_AUX1 + k] = 1; // increment by 2

        state.green_knife_guard_moving(k);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + k], 0x20);
        assert_eq!(state.ram[SPRITE_AI_STATE + k], 1);
        assert_eq!(state.ram[SPRITE_HEAD_DIR + k], 9);
        assert_eq!(state.ram[SPRITE_SUBTYPE2 + k], 7);
    }

    #[test]
    fn bolt_guard_trigger_chase_theme_pings_sfx_and_music() {
        // When sprite_G == 15, postincrement fires SFX + sets music=12
        // (when sram_progress=2 and overworld_area_lo=24).
        let mut state = fresh_state();
        let k = 1;
        state.ram[SPRITE_G + k] = 15;
        state.ram[SRAM_PROGRESS_INDICATOR] = 2;
        state.ram[OVERWORLD_AREA_INDEX_GUARD] = 24;
        state.bolt_guard_trigger_chase_theme(k);
        assert_eq!(state.ram[SPRITE_G + k], 16);
        assert_eq!(state.ram[MUSIC_CONTROL], 12);
    }

    #[test]
    fn bolt_guard_trigger_chase_theme_does_nothing_at_cap() {
        // sprite_G == 16 already: short-circuit, no change.
        let mut state = fresh_state();
        let k = 3;
        state.ram[SPRITE_G + k] = 16;
        state.ram[MUSIC_CONTROL] = 7;
        state.bolt_guard_trigger_chase_theme(k);
        assert_eq!(state.ram[SPRITE_G + k], 16);
        assert_eq!(state.ram[MUSIC_CONTROL], 7);
    }

    #[test]
    fn guard_shoot_probe_writes_vel_dir_and_gfx() {
        // i = sprite_B (e.g. 0) -> Xvel=1, Yvel=-1; then with wallcoll = 0
        // and delay_aux2=0, falls through to set delay_aux2 = 88. Then
        // mask2[0] = 1, so if wallcoll & 1 = 0, B stays.
        // Final Xvel2[0]=8, Yvel2[0]=0; Dir[0]=0; calls tick+update.
        let mut state = fresh_state();
        let k = 6;
        state.ram[SPRITE_B + k] = 0;
        state.ram[SPRITE_WALLCOLL + k] = 0;
        state.ram[SPRITE_DELAY_AUX2 + k] = 0;
        state.ram[SPRITE_D + k] = 0;
        state.ram[SPRITE_SUBTYPE2 + k] = 0;
        state.ram[SPRITE_FLAGS2 + k] = 0x60;
        state.guard_shoot_probe_and_stuff(k);
        assert_eq!(state.ram[SPRITE_DELAY_AUX2 + k], 88);
        assert_eq!(state.ram[SPRITE_X_VEL + k], 8u8);
        assert_eq!(state.ram[SPRITE_Y_VEL + k], 0);
        assert_eq!(state.ram[SPRITE_D + k], 0);
        assert_eq!(state.ram[SPRITE_HEAD_DIR + k], 0);
        // tick_and_update bumped subtype2.
        assert_eq!(state.ram[SPRITE_SUBTYPE2 + k], 1);
    }
}
