//! Ported NPC handlers from sprite_main.c — Bee / Cucco / vendor clusters.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 1426, 9367..10286, 12877, 13269, 23975..24166). The original C body is
//! reproduced as a comment block immediately above each port so a reviewer
//! can verify behavior line-by-line.
//!
//! Functions whose helper graph reaches outside the currently-ported subset
//! (e.g. full draw/move-Z pipelines, hit-box damage checks, follower spawning
//! chains) are deliberately not ported here. They are listed near the bottom
//! with a one-line skip note so the signature-drift scoreboard tells the truth.

use super::*;
use crate::types::{Point16U, SpriteHitBox};
use crate::zelda_rtl::sprite::{DrawMultipleData, PrepOamCoordsRet, SpriteSpawnInfo};

// Local constants needed by these handlers. Not yet referenced elsewhere in
// the crate, so they live here rather than in zelda_rtl.rs.
const SPRITE_Y_RECOIL: usize = 0x0f30;

// kSpriteDistress_X / kSpriteDistress_Y from sprite.c:435/436 — used by
// Sprite_DrawDistress_custom, which Cucco_DrawPANIC calls.
const K_SPRITE_DISTRESS_X: [i8; 4] = [-3, 2, 7, 11];
const K_SPRITE_DISTRESS_Y: [i8; 4] = [-5, -7, -7, -5];
const K_SPAWN_BEE_INIT_DELAY: [u8; 4] = [64, 64, 255, 255];
const K_SPAWN_BEE_INIT_VEL: [i8; 8] = [15, 5, -5, -15, 20, 10, -10, -20];
const K_CHECK_DAMAGE_FROM_PLAYER_NE: u8 = 2;
const SRAM_PROGRESS_INDICATOR_3_NPCS: usize = 0x0f3c9;
const K_BOTTLE_VENDOR_DMD: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: 0,
        y: -7,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0088,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -6,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x00a2,
        ext: 2,
    },
];

impl ZeldaState {
    // ----- Bee cluster ---------------------------------------------------

    // void Sprite_79_Bee(int k) {  // 9edc5b
    //   switch (sprite_ai_state[k]) {
    //   case 0:
    //     Bee_DormantHive(k);
    //     break;
    //   case 1:
    //     Bee_Main(k);
    //     break;
    //   case 2:
    //     Bee_PutInBottle(k);
    //     break;
    //   }
    // }
    pub(super) fn sprite_79_bee(&mut self, k: usize) {
        match self.ram[SPRITE_AI_STATE + k] {
            0 => self.bee_dormant_hive(k),
            1 => self.bee_main(k),
            2 => self.bee_put_in_bottle(k),
            _ => {}
        }
    }

    // void Bee_DormantHive(int k) {  // 9edc68
    //   if (sprite_E[k])
    //     return;
    //   sprite_state[k] = 0;
    //   for (int i = 11; i >= 0; i--)
    //     SpawnBeeFromHive(k);
    // }
    pub(super) fn bee_dormant_hive(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] != 0 {
            return;
        }
        self.ram[SPRITE_STATE + k] = 0;
        for _ in (0..=11).rev() {
            self.spawn_bee_from_hive(k);
        }
    }

    // void SpawnBeeFromHive(int k) {  // 9edc8f
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x79, &info);
    //   if (j >= 0) {
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     InitializeSpawnedBee(j);
    //   }
    // }
    pub(super) fn spawn_bee_from_hive(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x79, &mut info);
        if j >= 0 {
            let ju = j as usize;
            self.sprite_set_spawned_coordinates(ju, &info);
            self.initialize_spawned_bee(ju);
        }
    }

    // void InitializeSpawnedBee(int k) {  // 9edc9b
    //   sprite_ai_state[k] = 1;
    //   sprite_A[k] = sprite_delay_main[k] = kSpawnBee_InitDelay[k & 3];
    //   sprite_delay_aux4[k] = 96;
    //   sprite_x_vel[k] = kSpawnBee_InitVel[GetRandomNumber() & 7];
    //   sprite_y_vel[k] = kSpawnBee_InitVel[GetRandomNumber() & 7];
    // }
    pub(super) fn initialize_spawned_bee(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = 1;
        let delay = K_SPAWN_BEE_INIT_DELAY[k & 3];
        self.ram[SPRITE_DELAY_MAIN + k] = delay;
        self.ram[SPRITE_A + k] = delay;
        self.ram[SPRITE_DELAY_AUX4 + k] = 96;
        self.ram[SPRITE_X_VEL + k] =
            K_SPAWN_BEE_INIT_VEL[(self.get_random_number() & 7) as usize] as u8;
        self.ram[SPRITE_Y_VEL + k] =
            K_SPAWN_BEE_INIT_VEL[(self.get_random_number() & 7) as usize] as u8;
    }

    // int ReleaseBeeFromBottle(int x_value) {  // 9edccf
    //   static const int8 kSpawnBee_XY[8] = {8, 2, -2, -8, 10, 5, -5, -10};
    //
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(x_value, 0xb2, &info);
    //   if (j >= 0) {
    //     sprite_floor[j] = link_is_on_lower_level;
    //     Sprite_SetX(j, link_x_coord + 8);
    //     Sprite_SetY(j, link_y_coord + 16);
    //     if (link_bottle_info[link_item_bottle_index - 1] == 8)
    //       sprite_head_dir[j] = 1;
    //     InitializeSpawnedBee(j);
    //     sprite_x_vel[j] = kSpawnBee_XY[GetRandomNumber() & 7];
    //     sprite_y_vel[j] = kSpawnBee_XY[GetRandomNumber() & 7];
    //     sprite_delay_main[j] = 64;
    //     sprite_A[j] = 64;
    //   }
    //   return j;
    // }
    pub(super) fn release_bee_from_bottle(&mut self, x_value: usize) -> i32 {
        const K_SPAWN_BEE_XY: [i8; 8] = [8, 2, -2, -8, 10, 5, -5, -10];

        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(x_value, 0xb2, &mut info);
        if j >= 0 {
            let ju = j as usize;
            self.ram[SPRITE_FLOOR + ju] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.sprite_set_x(ju, self.player_state_view().x().wrapping_add(8));
            self.sprite_set_y(ju, self.player_state_view().y().wrapping_add(16));
            let bottle = self.ram[LINK_ITEM_BOTTLE_INDEX].wrapping_sub(1) as usize;
            if self.ram[LINK_BOTTLE_INFO + bottle] == 8 {
                self.ram[SPRITE_HEAD_DIR + ju] = 1;
            }
            self.initialize_spawned_bee(ju);
            self.ram[SPRITE_X_VEL + ju] =
                K_SPAWN_BEE_XY[(self.get_random_number() & 7) as usize] as u8;
            self.ram[SPRITE_Y_VEL + ju] =
                K_SPAWN_BEE_XY[(self.get_random_number() & 7) as usize] as u8;
            self.ram[SPRITE_DELAY_MAIN + ju] = 64;
            self.ram[SPRITE_A + ju] = 64;
        }
        j
    }

    // void Bee_Main(int k) {  // 9edd45
    //   Bee_HandleZ(k);
    //   SpriteDraw_SingleSmall(k);
    //   Bee_HandleInteractions(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   if (Sprite_ReturnIfRecoiling(k))
    //     return;
    //   if (sprite_head_dir[k])
    //     Sprite_SpawnSparkleGarnish(k);
    //   Bee_Bzzt(k);
    //   Sprite_MoveXY(k);
    //   sprite_graphics[k] = (k ^ frame_counter) >> 1 & 1;
    //   if (!sprite_delay_aux4[k]) {
    //     Sprite_CheckDamageToLink(k);
    //     if (Sprite_CheckDamageFromLink(k) & kCheckDamageFromPlayer_Ne) {
    //       Sprite_ShowMessageUnconditional(0xc8);
    //       sprite_ai_state[k] = 2; // put in bottle
    //       return;
    //     }
    //   }
    //
    //   if (!frame_counter && sprite_A[k] != 16)
    //     sprite_A[k] -= 8;
    //
    //   if (sprite_delay_main[k] == 0) {
    //     uint16 x = link_x_coord + (GetRandomNumber() & 3) * 5;
    //     uint16 y = link_y_coord + (GetRandomNumber() & 3) * 5;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, x, y, 20);
    //     sprite_y_vel[k] = pt.y;
    //     sprite_x_vel[k] = pt.x;
    //     sprite_oam_flags[k] = sprite_oam_flags[k] & ~0x40 | (sign8(pt.x) ? 0 : 0x40);
    //     sprite_delay_main[k] = k + sprite_A[k];
    //   }
    // }
    pub(super) fn bee_main(&mut self, k: usize) {
        self.bee_handle_z(k);
        self.sprite_draw_single_small_for_npcs(k);
        self.bee_handle_interactions(k);
        if self.sprite_return_if_inactive_npcs(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.ram[SPRITE_HEAD_DIR + k] != 0 {
            self.sprite_spawn_sparkle_garnish_for_npcs(k);
        }
        self.bee_bzzt(k);
        self.sprite_move_xy(k);
        self.ram[SPRITE_GRAPHICS + k] = (((k as u8) ^ self.ram[FRAME_COUNTER]) >> 1) & 1;
        if self.ram[SPRITE_DELAY_AUX4 + k] == 0 {
            self.sprite_check_damage_to_link_for_npcs(k);
            if (self.sprite_check_damage_from_link_for_npcs(k) & K_CHECK_DAMAGE_FROM_PLAYER_NE) != 0
            {
                self.sprite_show_message_unconditional(0xc8);
                self.ram[SPRITE_AI_STATE + k] = 2;
                return;
            }
        }

        if self.ram[FRAME_COUNTER] == 0 && self.ram[SPRITE_A + k] != 16 {
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_sub(8);
        }

        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            let x = self
                .player_state_view()
                .x()
                .wrapping_add(u16::from(self.get_random_number() & 3) * 5);
            let y = self
                .player_state_view()
                .y()
                .wrapping_add(u16::from(self.get_random_number() & 3) * 5);
            let pt = self.sprite_project_speed_towards_location(k, x, y, 20);
            self.ram[SPRITE_Y_VEL + k] = pt.y;
            self.ram[SPRITE_X_VEL + k] = pt.x;
            self.ram[SPRITE_OAM_FLAGS + k] =
                (self.ram[SPRITE_OAM_FLAGS + k] & !0x40) | if (pt.x as i8) < 0 { 0 } else { 0x40 };
            self.ram[SPRITE_DELAY_MAIN + k] = (k as u8).wrapping_add(self.ram[SPRITE_A + k]);
        }
    }

    // void Bee_HandleZ(int k) {  // sprite_main.c:24139
    //   sprite_z[k] = 16;
    //   if (sprite_head_dir[k])
    //     sprite_oam_flags[k] = (sprite_oam_flags[k] & 0xf1) |
    //                           (((frame_counter >> 4 & 3) + 1) << 1);
    // }
    pub(super) fn bee_handle_z(&mut self, k: usize) {
        self.ram[SPRITE_Z + k] = 16;
        if self.ram[SPRITE_HEAD_DIR + k] != 0 {
            let palette = (((self.ram[FRAME_COUNTER] >> 4) & 3).wrapping_add(1)) << 1;
            self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & 0xf1) | palette;
        }
    }

    // void PlayerBee_HoneInOnTarget(int j, int k) {  // sprite_main.c:13269
    //   if (sprite_type[j] != 0x88 && (sprite_flags[j] & 2))
    //     return;
    //   uint16 x = Sprite_GetX(j);
    //   uint16 y = Sprite_GetY(j);
    //   if ((uint16)(cur_sprite_x - x + 16) >= 24 ||
    //       (uint16)(cur_sprite_y - y - 8) >= 24)
    //     return;
    //   if (sprite_type[j] == 0x75) {
    //     sprite_E[j] = k + 1;
    //     return;
    //   }
    //   Ancilla_CheckDamageToSprite_preset(j, 1);
    //   sprite_F[j] = 15;
    //   sprite_x_recoil[j] = sprite_x_vel[k] << 1;
    //   sprite_y_recoil[j] = sprite_y_vel[k] << 1;
    //   sprite_B[k]++;
    // }
    pub(super) fn player_bee_hone_in_on_target(&mut self, j: usize, k: usize) {
        if self.ram[SPRITE_TYPE + j] != 0x88 && (self.ram[SPRITE_FLAGS + j] & 2) != 0 {
            return;
        }
        let x = self.sprite_get_x(j);
        let y = self.sprite_get_y(j);
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        if cur_x.wrapping_sub(x).wrapping_add(16) >= 24
            || cur_y.wrapping_sub(y).wrapping_sub(8) >= 24
        {
            return;
        }
        if self.ram[SPRITE_TYPE + j] == 0x75 {
            self.ram[SPRITE_E + j] = (k as u8).wrapping_add(1);
            return;
        }
        self.ancilla_check_damage_to_sprite_preset(j, 1);
        self.ram[SPRITE_F + j] = 15;
        self.ram[SPRITE_X_RECOIL + j] = self.ram[SPRITE_X_VEL + k].wrapping_shl(1);
        self.ram[SPRITE_Y_RECOIL + j] = self.ram[SPRITE_Y_VEL + k].wrapping_shl(1);
        self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
    }

    // bool PlayerBee_FindTarget(int k, Point16U *pt) {  // sprite_main.c:24145
    //   int n = 16;
    //   int j = k * 4 & 0xf;
    //   do {
    //     if (j == k || sprite_state[j] < 9 || sprite_pause[j])
    //       continue;
    //     if (!(sprite_flags2[j] & 0x80)) {
    //       if (sprite_floor[k] != sprite_floor[j] ||
    //           sprite_flags4[j] & 0x40 ||
    //           sprite_ignore_projectile[j])
    //         continue;
    //     } else {
    //       if (!sprite_head_dir[k] || !(sprite_bump_damage[j] & 0x40))
    //         continue;
    //     }
    //     PlayerBee_HoneInOnTarget(j, k);
    //     pt->x = Sprite_GetX(j) + (GetRandomNumber() & 3) * 5;
    //     pt->y = Sprite_GetY(j) + (GetRandomNumber() & 3) * 5;
    //     return true;
    //   } while (j = (j - 1) & 0xf, --n);
    //   return false;
    // }
    pub(super) fn player_bee_find_target(&mut self, k: usize, pt: &mut Point16U) -> bool {
        let mut n: i32 = 16;
        let mut j: usize = (k * 4) & 0xf;
        loop {
            // C `continue` body — evaluate filter; if it passes, return.
            let skip = j == k || self.ram[SPRITE_STATE + j] < 9 || self.ram[SPRITE_PAUSE + j] != 0;
            let inner_skip = if !skip {
                if self.ram[SPRITE_FLAGS2 + j] & 0x80 == 0 {
                    self.ram[SPRITE_FLOOR + k] != self.ram[SPRITE_FLOOR + j]
                        || self.ram[SPRITE_FLAGS4 + j] & 0x40 != 0
                        || self.ram[SPRITE_IGNORE_PROJECTILE + j] != 0
                } else {
                    self.ram[SPRITE_HEAD_DIR + k] == 0
                        || (self.ram[SPRITE_BUMP_DAMAGE + j] & 0x40) == 0
                }
            } else {
                true
            };
            if !inner_skip {
                self.player_bee_hone_in_on_target(j, k);
                let r1 = (self.get_random_number() & 3) as u16 * 5;
                let r2 = (self.get_random_number() & 3) as u16 * 5;
                pt.x = self.sprite_get_x(j).wrapping_add(r1);
                pt.y = self.sprite_get_y(j).wrapping_add(r2);
                return true;
            }
            // C `while (j = (j - 1) & 0xf, --n);` — decrement n; loop while n != 0.
            j = j.wrapping_sub(1) & 0xf;
            n -= 1;
            if n == 0 {
                break;
            }
        }
        false
    }

    // void Bee_HandleInteractions(int k) {  // 9ede44
    //   if (submodule_index == 2 && (dialogue_message_index == 0xc8 || dialogue_message_index == 0xca))
    //     sprite_delay_aux4[k] = 40;
    // }
    pub(super) fn bee_handle_interactions(&mut self, k: usize) {
        let dmi = self.ram[DIALOGUE_MESSAGE_INDEX] as u16
            | ((self.ram[DIALOGUE_MESSAGE_INDEX + 1] as u16) << 8);
        if self.frame_control_view().submodule() == 2 && (dmi == 0xc8 || dmi == 0xca) {
            self.ram[SPRITE_DELAY_AUX4 + k] = 40;
        }
    }

    // void Sprite_B2_PlayerBee(int k) {  // 9ede63
    //   static const uint8 kGoodBee_Tab0[2] = {0xa, 0x14};
    //
    //   switch (sprite_ai_state[k]) {
    //   case 0:  // wait
    //     if (!sprite_E[k]) {
    //       sprite_state[k] = 0;
    //       uint8 or_bottle = link_bottle_info[0] | link_bottle_info[1] | link_bottle_info[2] | link_bottle_info[3];
    //       if (!(or_bottle & 8))
    //         GoldBee_SpawnSelf(k);
    //     }
    //     break;
    //   case 1: {// activated
    //     sprite_ignore_projectile[k] = 1;
    //     Bee_HandleZ(k);
    //     SpriteDraw_SingleSmall(k);
    //     Bee_HandleInteractions(k);
    //     if (Sprite_ReturnIfInactive(k))
    //       return;
    //     Bee_Bzzt(k);
    //     Sprite_MoveXY(k);
    //     sprite_graphics[k] = (k ^ frame_counter) >> 1 & 1;
    //     if (sprite_head_dir[k])
    //       Sprite_SpawnSparkleGarnish(k);
    //     if (sprite_B[k] >= kGoodBee_Tab0[sprite_head_dir[k]]) {
    //       sprite_defl_bits[k] = 64;
    //       return;
    //     }
    //     if (sprite_delay_aux4[k])
    //       return;
    //     if (Sprite_CheckDamageFromLink(k) & kCheckDamageFromPlayer_Ne) {
    //       Sprite_ShowMessageUnconditional(0xc8);
    //       sprite_ai_state[k]++;
    //       return;
    //     }
    //     if ((k ^ frame_counter) & 3)
    //       return;
    //     Point16U pt2;
    //     if (!PlayerBee_FindTarget(k, &pt2)) {
    //       pt2.x = link_x_coord + (GetRandomNumber() & 3) * 5;
    //       pt2.y = link_y_coord + (GetRandomNumber() & 3) * 5;
    //     }
    //     if ((k ^ frame_counter) & 7)
    //       return;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, pt2.x, pt2.y, 32);
    //     sprite_x_vel[k] = pt.x;
    //     sprite_y_vel[k] = pt.y;
    //     sprite_oam_flags[k] = sprite_oam_flags[k] & ~0x40 | (sign8(pt.x) ? 0 : 0x40);
    //     break;
    //   }
    //   case 2:  // bottle
    //     Bee_PutInBottle(k);
    //     break;
    //   }
    // }
    pub(super) fn sprite_b2_player_bee(&mut self, k: usize) {
        const K_GOOD_BEE_TAB0: [u8; 2] = [0x0a, 0x14];

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_E + k] == 0 {
                    self.ram[SPRITE_STATE + k] = 0;
                    let or_bottle = self.ram[LINK_BOTTLE_INFO]
                        | self.ram[LINK_BOTTLE_INFO + 1]
                        | self.ram[LINK_BOTTLE_INFO + 2]
                        | self.ram[LINK_BOTTLE_INFO + 3];
                    if (or_bottle & 8) == 0 {
                        self.gold_bee_spawn_self(k);
                    }
                }
            }
            1 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;
                self.bee_handle_z(k);
                self.sprite_draw_single_small_for_npcs(k);
                self.bee_handle_interactions(k);
                if self.sprite_return_if_inactive_npcs(k) {
                    return;
                }
                self.bee_bzzt(k);
                self.sprite_move_xy(k);
                self.ram[SPRITE_GRAPHICS + k] = (((k as u8) ^ self.ram[FRAME_COUNTER]) >> 1) & 1;
                if self.ram[SPRITE_HEAD_DIR + k] != 0 {
                    self.sprite_spawn_sparkle_garnish_for_npcs(k);
                }
                let head = (self.ram[SPRITE_HEAD_DIR + k] & 1) as usize;
                if self.ram[SPRITE_B + k] >= K_GOOD_BEE_TAB0[head] {
                    self.ram[SPRITE_DEFL_BITS + k] = 64;
                    return;
                }
                if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
                    return;
                }
                if (self.sprite_check_damage_from_link_for_npcs(k) & K_CHECK_DAMAGE_FROM_PLAYER_NE)
                    != 0
                {
                    self.sprite_show_message_unconditional(0xc8);
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    return;
                }
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) != 0 {
                    return;
                }
                let mut pt2 = Point16U { x: 0, y: 0 };
                if !self.player_bee_find_target(k, &mut pt2) {
                    pt2.x = self
                        .player_state_view()
                        .x()
                        .wrapping_add(u16::from(self.get_random_number() & 3) * 5);
                    pt2.y = self
                        .player_state_view()
                        .y()
                        .wrapping_add(u16::from(self.get_random_number() & 3) * 5);
                }
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 7) != 0 {
                    return;
                }
                let pt = self.sprite_project_speed_towards_location(k, pt2.x, pt2.y, 32);
                self.ram[SPRITE_X_VEL + k] = pt.x;
                self.ram[SPRITE_Y_VEL + k] = pt.y;
                self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x40)
                    | if (pt.x as i8) < 0 { 0 } else { 0x40 };
            }
            2 => self.bee_put_in_bottle(k),
            _ => {}
        }
    }

    // void GoldBee_SpawnSelf(int k) {  // 9ede90
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x79, &info);
    //   if (j >= 0) {
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     sprite_ai_state[j] = 1;
    //     sprite_delay_main[j] = 64;
    //     sprite_A[j] = 64;
    //     sprite_delay_aux4[j] = 96;
    //     sprite_head_dir[j] = 1;
    //     sprite_x_vel[j] = kSpawnBee_InitVel[GetRandomNumber() & 7];
    //     sprite_y_vel[j] = kSpawnBee_InitVel[GetRandomNumber() & 7];
    //   }
    // }
    pub(super) fn gold_bee_spawn_self(&mut self, k: usize) {
        let mut info = SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0x79, &mut info);
        if j >= 0 {
            let ju = j as usize;
            self.sprite_set_spawned_coordinates(ju, &info);
            self.ram[SPRITE_AI_STATE + ju] = 1;
            self.ram[SPRITE_DELAY_MAIN + ju] = 64;
            self.ram[SPRITE_A + ju] = 64;
            self.ram[SPRITE_DELAY_AUX4 + ju] = 96;
            self.ram[SPRITE_HEAD_DIR + ju] = 1;
            self.ram[SPRITE_X_VEL + ju] =
                K_SPAWN_BEE_INIT_VEL[(self.get_random_number() & 7) as usize] as u8;
            self.ram[SPRITE_Y_VEL + ju] =
                K_SPAWN_BEE_INIT_VEL[(self.get_random_number() & 7) as usize] as u8;
        }
    }

    // void Bee_Bzzt(int k) {  // 9ee02e
    //   if (!((k ^ frame_counter) & 31))
    //     SpriteSfx_QueueSfx3WithPan(k, 0x2c);
    // }
    pub(super) fn bee_bzzt(&mut self, k: usize) {
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 31) == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2c);
        }
    }

    // void Bee_PutInBottle(int k) {  // sprite_main.c:1426
    //   Bee_HandleInteractions(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   if (!choice_in_multiselect_box) {
    //     int j = Sprite_Find_EmptyBottle();
    //     if (j >= 0) {
    //       link_bottle_info[j] = 7 + sprite_head_dir[k];
    //       Hud_RefreshIcon();
    //       sprite_state[k] = 0;
    //       return;
    //     }
    //     Sprite_ShowMessageUnconditional(0xca);
    //   }
    //   sprite_delay_aux4[k] = 64;
    //   sprite_ai_state[k] = 1;
    // }
    pub(super) fn bee_put_in_bottle(&mut self, k: usize) {
        self.bee_handle_interactions(k);
        if self.sprite_return_if_inactive_npcs(k) {
            return;
        }
        if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
            let j = self.sprite_find_empty_bottle();
            if j >= 0 {
                self.ram[LINK_BOTTLE_INFO + j as usize] =
                    7u8.wrapping_add(self.ram[SPRITE_HEAD_DIR + k]);
                self.hud_refresh_icon();
                self.ram[SPRITE_STATE + k] = 0;
                return;
            }
            self.sprite_show_message_unconditional(0xca);
        }
        self.ram[SPRITE_DELAY_AUX4 + k] = 64;
        self.ram[SPRITE_AI_STATE + k] = 1;
    }

    // int Sprite_Find_EmptyBottle() {  // sprite_main.c:24056
    //   for (int i = 0; i != 4; i++)
    //     if (link_bottle_info[i] == 2)
    //       return i;
    //   return -1;
    // }
    pub(super) fn sprite_find_empty_bottle(&self) -> i32 {
        for i in 0..4 {
            if self.ram[LINK_BOTTLE_INFO + i] == 2 {
                return i as i32;
            }
        }
        -1
    }

    // void Sprite_BottleVendor(int k) {  // 85ea79
    //   int j;
    //
    //   sprite_A[k] = BottleVendor_Draw(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   BottleMerchant_DetectFish(k);
    //   Sprite_BehaveAsBarrier(k);
    //   if (Sprite_CheckIfLinkIsBusy())
    //     return;
    //   if (GetRandomNumber() == 0) {
    //     sprite_delay_main[k] = 20;
    //     sprite_graphics[k] = 1;
    //   } else if (!sprite_delay_main[k]) {
    //     sprite_graphics[k] = 0;
    //   }
    //   switch (sprite_ai_state[k]) {
    //   case 0:  // base
    //     if (!sprite_A[k] && sprite_E[k])
    //       sprite_ai_state[k] = 3;
    //     else if (sram_progress_indicator_3 & 2)
    //       Sprite_ShowSolicitedMessage(k, 0xd4);
    //     else if (Sprite_ShowSolicitedMessage(k, 0xd1) & 0x100)
    //       sprite_ai_state[k] = 1;
    //     break;
    //   case 1:  // selling
    //     if (choice_in_multiselect_box == 0 && link_rupees_goal >= 100) {
    //       Sprite_ShowMessageUnconditional(0xd2);
    //       sprite_ai_state[k] = 2;
    //     } else {
    //       Sprite_ShowMessageUnconditional(0xd3);
    //       sprite_ai_state[k] = 0;
    //     }
    //     break;
    //   case 2:  // giving
    //     item_receipt_method = 0;
    //     Link_ReceiveItem(0x16, 0);
    //     sram_progress_indicator_3 |= 2;
    //     link_rupees_goal -= 100;
    //     sprite_ai_state[k] = 0;
    //     break;
    //   case 3:  // buying
    //     if (!sign8(sprite_E[k]))
    //       Sprite_ShowMessageUnconditional(0xd5);
    //     else
    //       Sprite_ShowMessageUnconditional(0xd6);
    //     sprite_ai_state[k] = 4;
    //     break;
    //   case 4:  // reward
    //     j = sprite_E[k];
    //     if (!sign8(j)) {
    //       sprite_state[j - 1] = 0;
    //       BottleMerchant_BuyBee(k);
    //     } else {
    //       sprite_state[j & 0xf] = 0;
    //       BottleMerchant_BuyFish(k);
    //     }
    //     sprite_E[k] = 0;
    //     sprite_ai_state[k] = 0;
    //     break;
    //   }
    // }
    pub(super) fn sprite_bottle_vendor(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = self.bottle_vendor_draw(k);
        if self.sprite_return_if_inactive_npcs(k) {
            return;
        }
        self.bottle_merchant_detect_fish(k);
        self.sprite_behave_as_barrier_for_npcs(k);
        if self.sprite_check_if_link_is_busy_for_npcs() {
            return;
        }
        if self.get_random_number() == 0 {
            self.ram[SPRITE_DELAY_MAIN + k] = 20;
            self.ram[SPRITE_GRAPHICS + k] = 1;
        } else if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_GRAPHICS + k] = 0;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_A + k] == 0 && self.ram[SPRITE_E + k] != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                } else if (self.ram[SRAM_PROGRESS_INDICATOR_3_NPCS] & 2) != 0 {
                    self.sprite_show_solicited_message(k, 0xd4);
                } else if (self.sprite_show_solicited_message(k, 0xd1) & 0x100) != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            }
            1 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0
                    && read_le_u16(&self.ram, LINK_RUPEES_GOAL) >= 100
                {
                    self.sprite_show_message_unconditional(0xd2);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                } else {
                    self.sprite_show_message_unconditional(0xd3);
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
            }
            2 => {
                self.ram[ITEM_RECEIPT_METHOD] = 0;
                self.link_receive_item(0x16, 0);
                self.ram[SRAM_PROGRESS_INDICATOR_3_NPCS] |= 2;
                let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL).wrapping_sub(100);
                write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, rupees);
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            3 => {
                if (self.ram[SPRITE_E + k] as i8) >= 0 {
                    self.sprite_show_message_unconditional(0xd5);
                } else {
                    self.sprite_show_message_unconditional(0xd6);
                }
                self.ram[SPRITE_AI_STATE + k] = 4;
            }
            4 => {
                let j = self.ram[SPRITE_E + k];
                if (j as i8) >= 0 {
                    self.ram[SPRITE_STATE + j as usize - 1] = 0;
                    self.bottle_merchant_buy_bee(k);
                } else {
                    self.ram[SPRITE_STATE + (j & 0x0f) as usize] = 0;
                    self.bottle_merchant_buy_fish(k);
                }
                self.ram[SPRITE_E + k] = 0;
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            _ => {}
        }
    }

    // uint8 BottleVendor_Draw(int k) {  // 85eba7
    //   PrepOamCoordsRet info;
    //   static const DrawMultipleData kBottleVendor_Dmd[4] = {
    //     {0, -7, 0x00ac, 2},
    //     {0,  0, 0x0088, 2},
    //     {0, -6, 0x00ac, 2},
    //     {0,  0, 0x00a2, 2},
    //   };
    //   Sprite_DrawMultiplePlayerDeferred(k, &kBottleVendor_Dmd[sprite_graphics[k] * 2], 2, &info);
    //   SpriteDraw_Shadow(k, &info);
    //   return (info.x | info.y) >> 8;
    // }
    pub(super) fn bottle_vendor_draw(&mut self, k: usize) -> u8 {
        let mut info = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        let base = (self.ram[SPRITE_GRAPHICS + k] as usize) * 2;
        self.sprite_draw_multiple_player_deferred(
            k,
            &K_BOTTLE_VENDOR_DMD[base..base + 2],
            Some(&mut info),
        );
        self.sprite_draw_shadow_custom(k, &mut info, 10);
        ((info.x | info.y) >> 8) as u8
    }

    // void BottleMerchant_DetectFish(int k) {  // 868000
    //   for (int i = 15; i >= 0; i--) {
    //     if (sprite_state[i] && sprite_type[i] == 0xd2) {
    //       SpriteHitBox hb;
    //       hb.r0_xlo = sprite_x_lo[k];
    //       hb.r8_xhi = sprite_x_hi[k];
    //       hb.r2 = 16;
    //       hb.r1_ylo = sprite_y_lo[k];
    //       hb.r9_yhi = sprite_y_hi[k];
    //       hb.r3 = 16;
    //       Sprite_SetupHitBox(i, &hb);
    //       if (CheckIfHitBoxesOverlap(&hb))
    //         sprite_E[k] = 0x80 | i;
    //       return;
    //     }
    //   }
    // }
    pub(super) fn bottle_merchant_detect_fish(&mut self, k: usize) {
        for i in (0..=15usize).rev() {
            if self.ram[SPRITE_STATE + i] != 0 && self.ram[SPRITE_TYPE + i] == 0xd2 {
                let mut hb = SpriteHitBox {
                    r0_xlo: self.ram[SPRITE_X_LO + k],
                    r8_xhi: self.ram[SPRITE_X_HI + k],
                    r1_ylo: self.ram[SPRITE_Y_LO + k],
                    r9_yhi: self.ram[SPRITE_Y_HI + k],
                    r2: 16,
                    r3: 16,
                    r4_spr_xlo: 0,
                    r10_spr_xhi: 0,
                    r5_spr_ylo: 0,
                    r11_spr_yhi: 0,
                    r6_spr_xsize: 0,
                    r7_spr_ysize: 0,
                };
                self.sprite_setup_hit_box(i, &mut hb);
                if self.check_if_hit_boxes_overlap(&hb) {
                    self.ram[SPRITE_E + k] = 0x80 | i as u8;
                }
                return;
            }
        }
    }

    // void BottleMerchant_BuyFish(int k) {  // 868054
    //   static const uint8 kBottleVendor_FishRewardType[5] = {0xdb, 0xe0, 0xde, 0xe2, 0xd9};
    //   static const int8 kBottleVendor_FishRewardXv[5] = {-6, -3, 0, 4, 7};
    //   static const int8 kBottleVendor_FishRewardYv[5] = {11, 14, 16, 14, 11};
    //   SpriteSpawnInfo info;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x13);
    //   tmp_counter = 4;
    //   do {
    //     int j = Sprite_SpawnDynamically(k, kBottleVendor_FishRewardType[tmp_counter], &info);
    //     if (j < 0)
    //       return;
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     sprite_x_lo[j] = info.r0_x + 4;
    //     sprite_stunned[j] = 0xff;
    //     sprite_x_vel[j] = kBottleVendor_FishRewardXv[tmp_counter];
    //     sprite_y_vel[j] = kBottleVendor_FishRewardYv[tmp_counter];
    //     sprite_z_vel[j] = 32;
    //     sprite_delay_aux4[j] = 32;
    //   } while (!sign8(--tmp_counter));
    // }
    pub(super) fn bottle_merchant_buy_fish(&mut self, k: usize) {
        const FISH_REWARD_TYPE: [u8; 5] = [0xdb, 0xe0, 0xde, 0xe2, 0xd9];
        const FISH_REWARD_XV: [i8; 5] = [-6, -3, 0, 4, 7];
        const FISH_REWARD_YV: [i8; 5] = [11, 14, 16, 14, 11];

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
        self.ram[TMP_COUNTER] = 4;
        loop {
            let i = self.ram[TMP_COUNTER] as usize;
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, FISH_REWARD_TYPE[i], &mut info);
            if j < 0 {
                return;
            }
            self.initialize_bottle_merchant_reward(
                j as usize,
                &info,
                FISH_REWARD_XV[i],
                FISH_REWARD_YV[i],
            );
            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if (self.ram[TMP_COUNTER] as i8) < 0 {
                break;
            }
        }
    }

    // void BottleMerchant_BuyBee(int k) {  // 9afe88
    //   static const int8 kBottleVendor_GoodBeeX[5] = {-6, -3, 0, 4, 7};
    //   static const int8 kBottleVendor_GoodBeeY[5] = {11, 14, 16, 14, 11};
    //   SpriteSpawnInfo info;
    //   SpriteSfx_QueueSfx3WithPan(k, 0x13);
    //   tmp_counter = 4;
    //   do {
    //     int j = Sprite_SpawnDynamically(k, 0xdb, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_x_lo[j] = info.r0_x + 4;
    //       sprite_stunned[j] = 0xff;
    //       sprite_x_vel[j] = kBottleVendor_GoodBeeX[tmp_counter];
    //       sprite_y_vel[j] = kBottleVendor_GoodBeeY[tmp_counter];
    //       sprite_z_vel[j] = 32;
    //       sprite_delay_aux4[j] = 32;
    //     }
    //   } while (!sign8(--tmp_counter));
    // }
    pub(super) fn bottle_merchant_buy_bee(&mut self, k: usize) {
        const GOOD_BEE_X: [i8; 5] = [-6, -3, 0, 4, 7];
        const GOOD_BEE_Y: [i8; 5] = [11, 14, 16, 14, 11];

        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
        self.ram[TMP_COUNTER] = 4;
        loop {
            let i = self.ram[TMP_COUNTER] as usize;
            let mut info = SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xdb, &mut info);
            if j >= 0 {
                self.initialize_bottle_merchant_reward(
                    j as usize,
                    &info,
                    GOOD_BEE_X[i],
                    GOOD_BEE_Y[i],
                );
            }
            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if (self.ram[TMP_COUNTER] as i8) < 0 {
                break;
            }
        }
    }

    fn initialize_bottle_merchant_reward(
        &mut self,
        j: usize,
        info: &SpriteSpawnInfo,
        xvel: i8,
        yvel: i8,
    ) {
        self.sprite_set_spawned_coordinates(j, info);
        self.ram[SPRITE_X_LO + j] = (info.r0_x as u8).wrapping_add(4);
        self.ram[SPRITE_STUNNED + j] = 0xff;
        self.ram[SPRITE_X_VEL + j] = xvel as u8;
        self.ram[SPRITE_Y_VEL + j] = yvel as u8;
        self.ram[SPRITE_Z_VEL + j] = 32;
        self.ram[SPRITE_DELAY_AUX4 + j] = 32;
    }

    // ----- Cucco cluster -------------------------------------------------

    // void Cucco_DrawPANIC(int k) {  // sprite_main.c:9408
    //   PrepOamCoordsRet info;
    //   if (Sprite_PrepOamCoordOrDoubleRet(k, &info))
    //     return;
    //   Sprite_DrawDistress_custom(info.x, info.y, frame_counter);
    // }
    pub(super) fn cucco_draw_panic(&mut self, k: usize) {
        let Some(info) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let time = self.ram[FRAME_COUNTER];
        self.sprite_draw_distress_custom_for_npcs(info.0, info.1, time);
    }

    // void Sprite_DrawDistress_custom(uint16 xin, uint16 yin, uint8 time) {
    //   Oam_AllocateFromRegionA(0x10);
    //   if (!(time & 0x18))
    //     return;
    //   int i = 3;
    //   OamEnt *oam = GetOamCurPtr();
    //   do {
    //     SetOamHelper0(oam, xin + kSpriteDistress_X[i],
    //                        yin + kSpriteDistress_Y[i], 0x83, 0x22, 0);
    //   } while (oam++, --i >= 0);
    // }
    pub(super) fn sprite_draw_distress_custom_for_npcs(&mut self, xin: u16, yin: u16, time: u8) {
        self.oam_allocate_from_region_a(0x10);
        if time & 0x18 == 0 {
            return;
        }
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let mut i: i32 = 3;
        loop {
            let x = xin.wrapping_add(K_SPRITE_DISTRESS_X[i as usize] as i16 as u16);
            let y = yin.wrapping_add(K_SPRITE_DISTRESS_Y[i as usize] as i16 as u16);
            self.set_oam_helper0_at(oam, x, y, 0x83, 0x22, 0);
            oam += 4;
            i -= 1;
            if i < 0 {
                break;
            }
        }
    }

    // ----- Internal helpers ported to back the above handlers -----------

    // Rewired to canonical Sprite_ReturnIfInactive port (sprite.c:1493).
    fn sprite_return_if_inactive_npcs(&mut self, k: usize) -> bool {
        self.sprite_return_if_inactive(k)
    }

    // Rewired to canonical SpriteDraw_SingleSmall port.
    fn sprite_draw_single_small_for_npcs(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
    }

    // Rewired to canonical Sprite_SpawnSparkleGarnish port.
    fn sprite_spawn_sparkle_garnish_for_npcs(&mut self, k: usize) {
        self.sprite_spawn_sparkle_garnish(k);
    }

    // Rewired to canonical Sprite_CheckDamageToLink port (sprite.c:2523).
    fn sprite_check_damage_to_link_for_npcs(&mut self, k: usize) {
        let _ = self.sprite_check_damage_to_link(k);
    }

    // Rewired to canonical Sprite_CheckDamageFromLink port (sprite.c:2639).
    fn sprite_check_damage_from_link_for_npcs(&mut self, k: usize) -> u8 {
        self.sprite_check_damage_from_link(k)
    }

    // Rewired to canonical Sprite_BehaveAsBarrier port (sprite.c:4288).
    fn sprite_behave_as_barrier_for_npcs(&mut self, k: usize) {
        self.sprite_behave_as_barrier(k);
    }

    fn sprite_check_if_link_is_busy_for_npcs(&self) -> bool {
        self.sprite_check_if_link_is_busy()
    }

    // ----- Skipped functions (helpers too complex / not yet ported) -----
    //
    // The following sprite_main.c entries from the Bee / Cucco / vendor
    // cluster are intentionally NOT ported in this file. Each depends on a
    // helper chain that has not yet landed in the Rust port. Reviving them
    // requires porting the listed helper(s) first.
    //
    //   Cucco_Calm                    — needs Sprite_ReturnIfLifted.
    //   Cucco_Flee                    — needs Sprite_ReturnIfLifted,
    //                                   Cucco_DoMovement_XY, Chicken_IncrSubtype2.
    //   Cucco_Carried                 — needs Sprite_MoveZ, Sprite_HalveSpeed_XY,
    //                                   Cucco_DoMovement_XY, Chicken_IncrSubtype2.
    //   Cucco_SummonAvenger           — needs Sprite_SpawnDynamicallyEx,
    //                                   Sprite_ApplySpeedTowardsLink.
    //
    // Note: the Bee state machines are ported above. Draw/damage/garnish
    // edges still use explicit local shims until those shared helpers land.
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, OAM_CUR_PTR, OAM_BUF as u16);
        write_le_u16(
            &mut state.ram,
            OAM_EXT_CUR_PTR,
            BYTEWISE_EXTENDED_OAM as u16,
        );
        state
    }

    fn make_link_idle(state: &mut ZeldaState) {
        state.ram[FLAG_UNK1] = 0;
        state.ram[SUBMODULE_INDEX] = 0;
        state.ram[LINK_AUXILIARY_STATE] = 0;
        state.ram[LINK_POSE_FOR_ITEM] = 0;
        state.ram[LINK_STATE_BITS] = 0;
        state.ram[ANCILLA_TYPE..ANCILLA_TYPE + 5].fill(0);
        write_le_u16(&mut state.ram, LINK_X_COORD, 0x1000);
        write_le_u16(&mut state.ram, LINK_Y_COORD, 0x1000);
    }

    #[test]
    fn bee_handle_z_sets_z_and_palette_when_head_dir() {
        // Bee_HandleZ: sprite_z[k] = 16; if (head_dir) palette bits from
        // frame_counter are written into sprite_oam_flags.
        let mut state = fresh_state();
        let k = 3;
        state.ram[FRAME_COUNTER] = 0x20; // (0x20 >> 4) & 3 == 2 → palette = (2+1)<<1 = 6
        state.ram[SPRITE_HEAD_DIR + k] = 1;
        state.ram[SPRITE_OAM_FLAGS + k] = 0xff;
        state.bee_handle_z(k);
        assert_eq!(state.ram[SPRITE_Z + k], 16);
        assert_eq!(state.ram[SPRITE_OAM_FLAGS + k], (0xff & 0xf1) | 6);
    }

    #[test]
    fn bee_handle_z_skips_palette_when_head_dir_zero() {
        let mut state = fresh_state();
        let k = 5;
        state.ram[FRAME_COUNTER] = 0xff;
        state.ram[SPRITE_HEAD_DIR + k] = 0;
        state.ram[SPRITE_OAM_FLAGS + k] = 0xaa;
        state.bee_handle_z(k);
        assert_eq!(state.ram[SPRITE_Z + k], 16);
        assert_eq!(state.ram[SPRITE_OAM_FLAGS + k], 0xaa);
    }

    #[test]
    fn initialize_spawned_bee_sets_active_state_and_delay() {
        let mut state = fresh_state();
        let k = 6; // k & 3 == 2 -> 255

        state.initialize_spawned_bee(k);

        assert_eq!(state.ram[SPRITE_AI_STATE + k], 1);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + k], 255);
        assert_eq!(state.ram[SPRITE_A + k], 255);
        assert_eq!(state.ram[SPRITE_DELAY_AUX4 + k], 96);
        assert!(K_SPAWN_BEE_INIT_VEL.contains(&(state.ram[SPRITE_X_VEL + k] as i8)));
        assert!(K_SPAWN_BEE_INIT_VEL.contains(&(state.ram[SPRITE_Y_VEL + k] as i8)));
    }

    #[test]
    fn release_bee_from_bottle_spawns_at_link_and_marks_good_bee() {
        let mut state = fresh_state();
        write_le_u16(&mut state.ram, LINK_X_COORD, 0x120);
        write_le_u16(&mut state.ram, LINK_Y_COORD, 0x230);
        state.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
        state.ram[LINK_ITEM_BOTTLE_INDEX] = 1;
        state.ram[LINK_BOTTLE_INFO] = 8;

        let j = state.release_bee_from_bottle(0);

        assert_eq!(j, 15);
        let ju = j as usize;
        assert_eq!(state.ram[SPRITE_TYPE + ju], 0xb2);
        assert_eq!(state.ram[SPRITE_FLOOR + ju], 1);
        assert_eq!(state.sprite_get_x(ju), 0x128);
        assert_eq!(state.sprite_get_y(ju), 0x240);
        assert_eq!(state.ram[SPRITE_HEAD_DIR + ju], 1);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + ju], 64);
        assert_eq!(state.ram[SPRITE_A + ju], 64);
    }

    #[test]
    fn sprite_79_bee_routes_dormant_hive_state() {
        let mut state = fresh_state();
        let k = 0;
        state.ram[SPRITE_AI_STATE + k] = 0;
        state.ram[SPRITE_STATE + k] = 9;
        state.sprite_79_bee(k);

        assert_eq!(state.ram[SPRITE_STATE + k], 0);
        assert_eq!(state.ram[SPRITE_TYPE + 15], 0x79);
        assert_eq!(state.ram[SPRITE_AI_STATE + 15], 1);
    }

    #[test]
    fn bee_main_updates_motion_and_retarget_timer() {
        let mut state = fresh_state();
        let k = 4;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_A + k] = 64;
        state.ram[SPRITE_DELAY_AUX4 + k] = 1; // skip damage path
        state.ram[SPRITE_DELAY_MAIN + k] = 0;
        state.ram[FRAME_COUNTER] = 4;
        write_le_u16(&mut state.ram, LINK_X_COORD, 0x120);
        write_le_u16(&mut state.ram, LINK_Y_COORD, 0x220);

        state.bee_main(k);

        assert_eq!(state.ram[SPRITE_Z + k], 16);
        assert_eq!(state.ram[SPRITE_GRAPHICS + k], 0);
        assert_eq!(state.ram[SPRITE_DELAY_MAIN + k], 68);
        assert_eq!(state.ram[SPRITE_OAM_FLAGS + k] & 0x40, 0x40);
    }

    #[test]
    fn player_bee_state_one_caps_after_enough_hits() {
        let mut state = fresh_state();
        let k = 3;
        state.ram[SPRITE_AI_STATE + k] = 1;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_HEAD_DIR + k] = 1;
        state.ram[SPRITE_B + k] = 0x14;

        state.sprite_b2_player_bee(k);

        assert_eq!(state.ram[SPRITE_IGNORE_PROJECTILE + k], 1);
        assert_eq!(state.ram[SPRITE_DEFL_BITS + k], 64);
    }

    #[test]
    fn bottle_merchant_detect_fish_sets_negative_sprite_index() {
        let mut state = fresh_state();
        let vendor = 2;
        let fish = 9;
        state.sprite_set_x(vendor, 0x40);
        state.sprite_set_y(vendor, 0x50);
        state.sprite_set_x(fish, 0x40);
        state.sprite_set_y(fish, 0x50);
        state.ram[SPRITE_STATE + fish] = 9;
        state.ram[SPRITE_TYPE + fish] = 0xd2;

        state.bottle_merchant_detect_fish(vendor);

        assert_eq!(state.ram[SPRITE_E + vendor], 0x80 | fish as u8);
    }

    #[test]
    fn bottle_merchant_buy_bee_spawns_five_rewards() {
        let mut state = fresh_state();
        let k = 1;
        state.sprite_set_x(k, 0x30);
        state.sprite_set_y(k, 0x60);

        state.bottle_merchant_buy_bee(k);

        for j in 11..=15 {
            assert_eq!(state.ram[SPRITE_TYPE + j], 0xdb);
            assert_eq!(state.ram[SPRITE_STUNNED + j], 0xff);
            assert_eq!(state.ram[SPRITE_Z_VEL + j], 32);
            assert_eq!(state.ram[SPRITE_DELAY_AUX4 + j], 32);
        }
        assert_eq!(state.ram[TMP_COUNTER], 0xff);
    }

    #[test]
    fn bottle_merchant_buy_fish_spawns_reward_types() {
        let mut state = fresh_state();
        let k = 1;
        state.sprite_set_x(k, 0x30);
        state.sprite_set_y(k, 0x60);

        state.bottle_merchant_buy_fish(k);

        assert_eq!(state.ram[SPRITE_TYPE + 15], 0xd9);
        assert_eq!(state.ram[SPRITE_TYPE + 14], 0xe2);
        assert_eq!(state.ram[SPRITE_TYPE + 13], 0xde);
        assert_eq!(state.ram[SPRITE_TYPE + 12], 0xe0);
        assert_eq!(state.ram[SPRITE_TYPE + 11], 0xdb);
        assert_eq!(state.ram[SPRITE_DELAY_AUX4 + 11], 32);
        assert_eq!(state.ram[TMP_COUNTER], 0xff);
    }

    #[test]
    fn sprite_bottle_vendor_base_detects_trade_offer() {
        let mut state = fresh_state();
        let k = 2;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_AI_STATE + k] = 0;
        state.ram[SPRITE_E + k] = 3;
        make_link_idle(&mut state);

        state.sprite_bottle_vendor(k);

        assert_eq!(state.ram[SPRITE_AI_STATE + k], 3);
    }

    #[test]
    fn sprite_bottle_vendor_selling_accepts_when_affordable() {
        let mut state = fresh_state();
        let k = 2;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_AI_STATE + k] = 1;
        state.ram[CHOICE_IN_MULTISELECT_BOX] = 0;
        write_le_u16(&mut state.ram, LINK_RUPEES_GOAL, 100);
        make_link_idle(&mut state);

        state.sprite_bottle_vendor(k);

        assert_eq!(state.ram[SPRITE_AI_STATE + k], 2);
        assert_eq!(read_le_u16(&state.ram, DIALOGUE_MESSAGE_INDEX), 0xd2);
    }

    #[test]
    fn sprite_bottle_vendor_giving_marks_bottle_bought_and_charges_rupees() {
        let mut state = fresh_state();
        let k = 2;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_AI_STATE + k] = 2;
        write_le_u16(&mut state.ram, LINK_RUPEES_GOAL, 150);
        make_link_idle(&mut state);

        state.sprite_bottle_vendor(k);

        assert_eq!(state.ram[SPRITE_AI_STATE + k], 0);
        assert_eq!(state.ram[SRAM_PROGRESS_INDICATOR_3_NPCS] & 2, 2);
        assert_eq!(read_le_u16(&state.ram, LINK_RUPEES_GOAL), 50);
    }

    #[test]
    fn sprite_bottle_vendor_reward_clears_fish_and_spawns_rewards() {
        let mut state = fresh_state();
        let k = 2;
        let fish = 5;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_AI_STATE + k] = 4;
        state.ram[SPRITE_E + k] = 0x80 | fish as u8;
        state.ram[SPRITE_STATE + fish] = 9;
        make_link_idle(&mut state);

        state.sprite_bottle_vendor(k);

        assert_eq!(state.ram[SPRITE_STATE + fish], 0);
        assert_eq!(state.ram[SPRITE_E + k], 0);
        assert_eq!(state.ram[SPRITE_AI_STATE + k], 0);
        assert_eq!(state.ram[SPRITE_TYPE + 15], 0xd9);
    }

    #[test]
    fn sprite_find_empty_bottle_locates_value_two() {
        // Sprite_Find_EmptyBottle returns first slot whose value is 2.
        let mut state = fresh_state();
        state.ram[LINK_BOTTLE_INFO + 0] = 1;
        state.ram[LINK_BOTTLE_INFO + 1] = 1;
        state.ram[LINK_BOTTLE_INFO + 2] = 2;
        state.ram[LINK_BOTTLE_INFO + 3] = 2;
        assert_eq!(state.sprite_find_empty_bottle(), 2);

        // None empty → returns -1.
        state.ram[LINK_BOTTLE_INFO + 2] = 1;
        state.ram[LINK_BOTTLE_INFO + 3] = 1;
        assert_eq!(state.sprite_find_empty_bottle(), -1);
    }

    #[test]
    fn bee_put_in_bottle_stores_bottle_and_clears_state() {
        // First branch: choice_in_multiselect_box == 0 and an empty bottle
        // exists. Expect: link_bottle_info[j] = 7 + sprite_head_dir[k],
        // sprite_state[k] = 0.
        let mut state = fresh_state();
        // Make Sprite_ReturnIfInactive(k) return false: state=9, no flags.
        let k = 1;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[FLAG_UNK1] = 0;
        state.ram[SUBMODULE_INDEX] = 0;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_PAUSE + k] = 0;
        state.ram[CHOICE_IN_MULTISELECT_BOX] = 0;
        state.ram[LINK_BOTTLE_INFO + 0] = 1;
        state.ram[LINK_BOTTLE_INFO + 1] = 2; // first empty
        state.ram[SPRITE_HEAD_DIR + k] = 0;

        state.bee_put_in_bottle(k);
        assert_eq!(state.ram[LINK_BOTTLE_INFO + 1], 7);
        assert_eq!(state.ram[SPRITE_STATE + k], 0);
    }

    #[test]
    fn bee_put_in_bottle_arms_delay_when_no_bottle() {
        // No empty bottle → calls Sprite_ShowMessageUnconditional(0xca)
        // and falls through to delay+ai_state writes.
        let mut state = fresh_state();
        let k = 2;
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[CHOICE_IN_MULTISELECT_BOX] = 0;
        for i in 0..4 {
            state.ram[LINK_BOTTLE_INFO + i] = 1;
        }

        state.bee_put_in_bottle(k);
        // delay and ai_state are written unconditionally after the bottle
        // check fails.
        assert_eq!(state.ram[SPRITE_DELAY_AUX4 + k], 64);
        assert_eq!(state.ram[SPRITE_AI_STATE + k], 1);
        // Sprite_ShowMessageUnconditional(0xca) wrote dialogue index and
        // bumped main_module_index to 14.
        assert_eq!(read_le_u16(&state.ram, DIALOGUE_MESSAGE_INDEX), 0xca);
        assert_eq!(state.ram[MAIN_MODULE_INDEX], 14);
    }

    #[test]
    fn player_bee_hone_in_on_target_bumps_sprite_b_and_recoil() {
        // For a normal target (type != 0x88, flags&2 == 0, type != 0x75)
        // within range, expect F=15, recoil = vel<<1, sprite_B[k] +=1.
        let mut state = fresh_state();
        let k = 2;
        let j = 5;
        state.ram[SPRITE_TYPE + j] = 0x10;
        state.ram[SPRITE_FLAGS + j] = 0; // bit 1 clear
                                         // x=y=0; cur_x=0,cur_y=0 → (0-0+16)=16<24 ✓; (0-0-8)=0xfff8 huge — fails.
                                         // Set Sprite_GetX/Y to large so the deltas pass.
        state.ram[SPRITE_X_LO + j] = 0x10;
        state.ram[SPRITE_Y_LO + j] = 0x10;
        write_le_u16(&mut state.ram, CUR_SPRITE_X, 0x10);
        write_le_u16(&mut state.ram, CUR_SPRITE_Y, 0x18);
        // cur_x - x + 16 = 0 + 16 = 16  (<24 ✓)
        // cur_y - y - 8  = 8         (<24 ✓)
        state.ram[SPRITE_X_VEL + k] = 3;
        state.ram[SPRITE_Y_VEL + k] = 2;
        state.ram[SPRITE_B + k] = 7;

        state.player_bee_hone_in_on_target(j, k);
        assert_eq!(state.ram[SPRITE_F + j], 15);
        assert_eq!(state.ram[SPRITE_X_RECOIL + j], 6);
        assert_eq!(state.ram[SPRITE_Y_RECOIL + j], 4);
        assert_eq!(state.ram[SPRITE_B + k], 8);
    }
}
