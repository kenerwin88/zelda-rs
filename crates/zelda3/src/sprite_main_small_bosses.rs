//! Ported Trinexx / Vitreous / YellowStalfos boss handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source. The original
//! C body is reproduced as a comment block immediately above each port
//! so a reviewer can verify behavior line-by-line.
//!
//! Helpers that reach into the broader OAM/collision pipelines
//! (`Sprite_DrawSingleLarge`, `Sprite_MoveXYZ`,
//! `Sprite_CheckTileCollision`, `Sprite_CheckDamageToAndFromLink`,
//! `Sprite_CheckDamageFromLink`, `Sprite_Get16BitCoords`,
//! `Sprite_InitializedSegmented`, `SpritePrep_LoadProperties`,
//! `Sprite_SpawnDynamically(Ex)`, `Sprite_SpawnLightning`, and
//! `Sprite_TrinexxD_Draw`) are adapted with the `_for_small_bosses` suffix
//! so the data-state side of the handlers stays exercisable while the rest
//! of the C surface comes online.

use super::sprite_main_draw::{trinexx_head_sin, PrepOamCoordsRet as DrawPrepOamCoordsRet};
use super::*;
use crate::types::sign8;
use crate::zelda_rtl::sprite::DrawMultipleData;

/// Mirror of the C-side `PrepOamCoordsRet` (sprite.c). The canonical
/// `sprite::PrepOamCoordsRet` is module-private; copy it here so the
/// `YellowStalfos_DrawHead(int k, PrepOamCoordsRet *info)` signature
/// keeps its named struct shape.
#[derive(Copy, Clone, Default)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

// `Sprite_DelayAux3` is shared scratch (variables.h:0xee0).
const SPRITE_DELAY_AUX3_SB: usize = 0x0ee0;
// Shared scratch used by small-boss draw/update routines.
const SMALL_BOSS_SHARED_SCRATCH_A: usize = 0x0fb6;
const VITREOUS_EYEBALL_RELEASE_COUNT: usize = 0x0ff8;
// `overlord_x_hi` (variables.h:0xb10).
const OVERLORD_X_HI_SB: usize = 0x0b10;
// Moldorm history buffer is reused by Trinexx for its tail history
// (sprite_main.c:22, 24 #defines).
const MOLDORM_X_LO_SB: usize = 0x1fc00;
const MOLDORM_X_HI_SB: usize = 0x1fc80;
const MOLDORM_Y_LO_SB: usize = 0x1fd00;
const MOLDORM_Y_HI_SB: usize = 0x1fd80;
// Alternate-sprite tables Trinexx clears on phase-transition.
const ALT_SPRITE_TYPE_SB: usize = 0x1d10;
const ALT_SPRITE_X_HI_SB: usize = 0x1d30;
const ALT_SPRITE_Y_HI_SB: usize = 0x1d50;

// kSprite_TrinexxD_Gfx3 / Gfx — angle-to-graphic tables (sprite_main.c:16035-16038).
const K_SPRITE_TRINEXXD_GFX3: [u8; 8] = [6, 7, 0, 1, 2, 3, 4, 5];
const K_SPRITE_TRINEXXD_GFX: [u8; 8] = [7, 7, 1, 1, 3, 3, 5, 5];
const K_TRINEXXD_HIST_POS: [u8; 24] = [
    8, 0x0c, 0x10, 0x18, 0x20, 0x28, 0x30, 0x34, 0x38, 0x3c, 0x40, 0x44, 0x48, 0x4c, 0x50, 0x54,
    0x58, 0x5c, 0x60, 0x64, 0x68, 0x6c, 0x70, 0x74,
];
const K_SPRITE_TRINEXXD_GFX2: [u8; 24] = [
    2, 2, 2, 3, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const K_TRINEXXD_OAM_OFFS: [u16; 24] = [
    0x10, 4, 4, 4, 0x10, 0x10, 0x10, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
];
const K_SPRITE_TRINEXXD_XVEL: [i8; 4] = [0, -31, 0, 31];
const K_SPRITE_TRINEXXD_YVEL: [i8; 4] = [31, 0, -31, 0];
const K_TRINEXX_HEAD_XOFFS: [i8; 2] = [-14, 13];
const K_TRINEXX_HEAD_TARGET0: [u8; 45] = [
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x58, 0x64, 0x6a, 0x6f, 0x74, 0x7a, 0x7e,
    0x80, 0x80, 0x39, 0x48, 0x52, 0x5c, 0x65, 0x73, 0x77, 0x7a, 0x80, 0x1e, 0x24, 0x29, 0x2e, 0x34,
    0x3a, 0x44, 0x4d, 0x80, 0x0a, 0x11, 0x17, 0x1c, 0x22, 0x2a, 0x36, 0x3a, 0x80,
];
const K_TRINEXX_HEAD_TARGET1: [u8; 45] = [
    0x30, 0x28, 0x23, 0x1e, 0x19, 0x13, 0x0c, 6, 0, 0x2f, 0x26, 0x21, 0x1d, 0x18, 0x12, 0x0c, 6, 0,
    0x2f, 0x27, 0x22, 0x1d, 0x18, 0x12, 0x0c, 6, 0, 0x2f, 0x27, 0x22, 0x1d, 0x18, 0x12, 0x0c, 6, 0,
    0x48, 0x3a, 0x32, 0x29, 0x22, 0x19, 0x10, 7, 0,
];
const K_TRINEXX_HEAD_FRAME_MASK: [u8; 8] = [1, 1, 3, 3, 7, 0x0f, 0x1f, 0x1f];
const K_TRINEXX_HEAD_FIRST_PART_X: [i8; 5] = [-8, 8, -8, 8, 0];
const K_TRINEXX_HEAD_FIRST_PART_Y: [i8; 5] = [-8, -8, 8, 8, 2];
const K_TRINEXX_HEAD_FIRST_PART_CHAR: [u8; 5] = [4, 4, 0x24, 0x24, 0x0a];
const K_TRINEXX_HEAD_FIRST_PART_FLAGS: [u8; 5] = [0x40, 0, 0x40, 0, 0];

// kVitreous_Animate_Gfx (sprite_main.c:18532).
const K_VITREOUS_ANIMATE_GFX: [i8; 2] = [2, 1];

// kVitreous_WhichToActivate (sprite_main.c:18542).
const K_VITREOUS_WHICH_TO_ACTIVATE: [u8; 16] =
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 5, 6, 7, 8, 9, 10, 11];

// kVitreous_SpawnSmallerEyes_X/Y/Gfx (sprite_main.c:18157-18159).
const K_VITREOUS_SPAWN_SMALLER_EYES_X: [i8; 13] =
    [8, 22, -8, -22, 0, 14, 19, 33, 26, -14, -19, -33, -26];
const K_VITREOUS_SPAWN_SMALLER_EYES_Y: [i8; 13] =
    [-8, -12, -8, -12, 0, -20, -1, -12, -24, -20, -1, -12, -24];
const K_VITREOUS_SPAWN_SMALLER_EYES_GFX: [u8; 13] = [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// kVitreous_Dmd (sprite_main.c:18572) — 24 entries used by Vitreous_Draw.
// Stored as (dx:i8, dy:i8, char_flags:u16, ext:u8).
type SbDmd = (i8, i8, u16, u8);
const K_VITREOUS_DMD: [SbDmd; 24] = [
    (-8, -8, 0x01c0, 2),
    (8, -8, 0x41c0, 2),
    (-8, 8, 0x01e0, 2),
    (8, 8, 0x41e0, 2),
    (-8, -8, 0x01c8, 2),
    (8, -8, 0x01ca, 2),
    (-8, 8, 0x01e8, 2),
    (8, 8, 0x01ea, 2),
    (-8, -8, 0x41ca, 2),
    (8, -8, 0x41c8, 2),
    (-8, 8, 0x41ea, 2),
    (8, 8, 0x41e8, 2),
    (-8, -8, 0x01c2, 2),
    (8, -8, 0x41c2, 2),
    (-8, 8, 0x01e2, 2),
    (8, 8, 0x41e2, 2),
    (-8, -8, 0x01c4, 2),
    (8, -8, 0x41c4, 2),
    (-8, 8, 0x01e4, 2),
    (8, 8, 0x41e4, 2),
    (-7, -7, 0x01c4, 2),
    (7, -7, 0x41c4, 2),
    (-7, 7, 0x01e4, 2),
    (7, 7, 0x41e4, 2),
];
const K_SPRITE_LIGHTNING_GFX: [u8; 8] = [0, 1, 2, 3, 0, 1, 2, 3];
const K_SPRITE_LIGHTNING_OAM_FLAGS: [u8; 8] = [0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40];
const K_SPRITE_LIGHTNING_XOFF: [i8; 64] = [
    -15, 0, 0, -15, 0, -15, -15, 0, -15, 0, 0, -15, 0, -15, -15, 0, 0, 15, 15, 0, 15, 0, 0, 15, 0,
    15, 15, 0, 15, 0, 0, 15, 0, 15, 15, 0, 15, 0, 0, 15, 0, 15, 15, 0, 15, 0, 0, 15, -15, 0, 0,
    -15, 0, -15, -15, 0, -15, 0, 0, -15, 0, -15, -15, 0,
];
const K_AGAHNIM_LIGHTING_X: [i8; 8] = [-8, 8, 8, -8, 8, -8, -8, 8];

// kYellowStalfos_Gfx2 / Head_Char / Head_Flags / Dmd (sprite_main.c:22903, 22956-22957, 22921).
const K_YELLOW_STALFOS_GFX2: [u8; 4] = [6, 3, 1, 1];
const K_YELLOW_STALFOS_OBJ_PRIO: [u8; 6] = [0x30, 0, 0, 0, 0x30, 0];
const K_YELLOW_STALFOS_GFX: [u8; 32] = [
    8, 5, 1, 1, 8, 5, 1, 1, 8, 5, 1, 1, 7, 4, 2, 2, 7, 4, 2, 2, 7, 4, 2, 2, 7, 4, 2, 2, 7, 4, 2, 2,
];
const K_YELLOW_STALFOS_HEAD_X: [i8; 32] = [
    -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, -0x80, 0, 0, 0, 0,
    0, 0, 0, 0, -1, 0, 1, 0, -1, 0, 1, 0, 0, 0, 0, 0,
];
const K_YELLOW_STALFOS_HEAD_Y: [u8; 32] = [
    13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 12, 11, 10, 10, 10, 10, 10,
    10, 10, 10, 10, 10, 10, 10, 10,
];
const K_YELLOW_STALFOS_NEUTRALIZED_GFX: [u8; 16] =
    [1, 1, 1, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9];
const K_YELLOW_STALFOS_NEUTRALIZED_HEAD_Y: [u8; 16] =
    [10, 10, 10, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7];
const K_YELLOW_STALFOS_HEAD_CHAR: [u8; 4] = [2, 2, 0, 4];
const K_YELLOW_STALFOS_HEAD_FLAGS: [u8; 4] = [0x40, 0, 0, 0];
const K_YELLOW_STALFOS_DMD: [SbDmd; 22] = [
    (0, 0, 0x000a, 2),
    (0, 0, 0x000a, 2),
    (0, 0, 0x000c, 2),
    (0, 0, 0x000c, 2),
    (0, 0, 0x002c, 2),
    (0, 0, 0x002c, 2),
    (5, 5, 0x002e, 0),
    (0, 0, 0x0024, 2),
    (4, 1, 0x003e, 0),
    (0, 0, 0x0024, 2),
    (0, 0, 0x000e, 2),
    (0, 0, 0x000e, 2),
    (3, 5, 0x402e, 0),
    (0, 0, 0x4024, 2),
    (4, 1, 0x403e, 0),
    (0, 0, 0x4024, 2),
    (0, 0, 0x400e, 2),
    (0, 0, 0x400e, 2),
    (0, 0, 0x002a, 2),
    (0, 0, 0x002a, 2),
    (0, 0, 0x002a, 2),
    (0, 0, 0x002a, 2),
];

impl ZeldaState {
    // void Trinexx_RestoreXY(int k) {  // 9dad4f
    //   sprite_x_lo[k] = sprite_A[k];
    //   Sprite_SetY(k, (sprite_G[k] << 8) + sprite_C[k] + 12);
    // }
    pub(super) fn trinexx_restore_xy(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_A + k];
        let y = ((self.ram[SPRITE_G + k] as u16) << 8)
            .wrapping_add(self.ram[SPRITE_C + k] as u16)
            .wrapping_add(12);
        self.sprite_set_y(k, y);
    }

    // void Trinexx_CachePosition(int k) {  // 9dad8c
    //   sprite_A[k] = sprite_x_lo[k];
    //   sprite_B[k] = sprite_x_hi[k];
    //   sprite_C[k] = sprite_y_lo[k];
    //   sprite_G[k] = sprite_y_hi[k];
    // }
    pub(super) fn trinexx_cache_position(&mut self, k: usize) {
        self.ram[SPRITE_A + k] = self.ram[SPRITE_X_LO + k];
        self.ram[SPRITE_B + k] = self.ram[SPRITE_X_HI + k];
        self.ram[SPRITE_C + k] = self.ram[SPRITE_Y_LO + k];
        self.ram[SPRITE_G + k] = self.ram[SPRITE_Y_HI + k];
    }

    // void Sprite_Trinexx_FinalPhase(int k) {  // 9dadb5
    //   ... (see header comment block for full body).
    pub(super) fn sprite_trinexx_final_phase(&mut self, k: usize) {
        let x_vel = self.ram[SPRITE_X_VEL + k] as i8;
        let y_vel = self.ram[SPRITE_Y_VEL + k] as i8;
        let j_init = self.sprite_convert_velocity_to_angle_for_small_bosses(x_vel, y_vel) >> 1;
        let gfx_idx = (j_init as usize) & 7;
        let gfx = K_SPRITE_TRINEXXD_GFX3[gfx_idx];
        let chose_alt = self.ram[SPRITE_DELAY_AUX1 + k] != 0;
        self.ram[SPRITE_GRAPHICS + k] = if chose_alt {
            K_SPRITE_TRINEXXD_GFX[gfx as usize]
        } else {
            gfx
        };

        self.sprite_trinexxd_draw_for_small_bosses(k);
        if self.sprite_return_if_inactive_for_small_bosses(k) {
            return;
        }
        if (self.ram[SPRITE_AI_STATE + k] as i8).is_negative() {
            let t = self.ram[SPRITE_DELAY_MAIN + k];
            self.ram[SPRITE_HIT_TIMER + k] = t | 0xe0;
            if t == 0 {
                self.ram[SPRITE_DELAY_MAIN + k] = 12;
                if self.ram[SPRITE_ANIM_CLOCK + k] == 0 {
                    self.ram[SPRITE_HIT_TIMER + k] = 255;
                    self.sprite_schedule_boss_for_death_for_small_bosses(k);
                } else {
                    self.ram[SPRITE_ANIM_CLOCK + k] =
                        self.ram[SPRITE_ANIM_CLOCK + k].wrapping_sub(1);
                    self.sprite_make_boss_explosion_for_small_bosses(k);
                }
            }
            return;
        }
        if (self.ram[FRAME_COUNTER] & 7) == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x31);
        }

        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        let j = (self.ram[SPRITE_SUBTYPE2 + k] & 0x7f) as usize;
        self.ram[MOLDORM_X_LO_SB + j] = self.ram[SPRITE_X_LO + k];
        self.ram[MOLDORM_Y_LO_SB + j] = self.ram[SPRITE_Y_LO + k];
        self.ram[MOLDORM_X_HI_SB + j] = self.ram[SPRITE_X_HI + k];
        self.ram[MOLDORM_Y_HI_SB + j] = self.ram[SPRITE_Y_HI + k];

        if self.ram[SPRITE_F + k] == 14 {
            self.ram[SPRITE_F + k] = 8;
            if self.ram[SPRITE_AI_STATE + k] == 0 {
                self.ram[SPRITE_AI_STATE + k] = 2;
            }
        }
        self.sprite_move_xy_for_small_bosses(k);
        self.sprite_check_damage_to_and_from_link_for_small_bosses(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_sub(1);
                if self.ram[SPRITE_A + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 192;
                }
                self.sprite_get_16bit_coords_for_small_bosses(k);
                if self.sprite_check_tile_collision_for_small_bosses(k) {
                    self.ram[SPRITE_D + k] = (self.ram[SPRITE_D + k].wrapping_add(1)) & 3;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 8;
                }
                let j2 = (self.ram[SPRITE_D + k] & 3) as usize;
                self.ram[SPRITE_X_VEL + k] = K_SPRITE_TRINEXXD_XVEL[j2] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_SPRITE_TRINEXXD_YVEL[j2] as u8;
            }
            1 => {
                if (self.ram[FRAME_COUNTER] & 1) == 0 {
                    let pt = self.sprite_project_speed_towards_link(k, 31);
                    self.sprite_approach_target_speed_for_small_bosses(k, pt.x, pt.y);
                }
            }
            _ => {}
        }
    }

    // void Sprite_Trinexx_CheckDamageToFlashingSegment(int k) {  // 9db079
    //   uint16 old_x = Sprite_GetX(k);
    //   uint16 old_y = Sprite_GetY(k);
    //   Sprite_SetX(k, cur_sprite_x);
    //   Sprite_SetY(k, cur_sprite_y);
    //   sprite_defl_bits[k] = 0x80;
    //   sprite_flags3[k] = 0;
    //   Sprite_CheckDamageFromLink(k);
    //   sprite_defl_bits[k] = 0x84;
    //   sprite_flags3[k] = 0x40;
    //   Sprite_SetX(k, old_x);
    //   Sprite_SetY(k, old_y);
    // }
    pub(super) fn sprite_trinexx_check_damage_to_flashing_segment(&mut self, k: usize) {
        let old_x = self.sprite_get_x(k);
        let old_y = self.sprite_get_y(k);
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        self.sprite_set_x(k, cur_x);
        self.sprite_set_y(k, cur_y);
        self.ram[SPRITE_DEFL_BITS + k] = 0x80;
        self.ram[SPRITE_FLAGS3 + k] = 0;
        self.sprite_check_damage_from_link_for_small_bosses(k);
        self.ram[SPRITE_DEFL_BITS + k] = 0x84;
        self.ram[SPRITE_FLAGS3 + k] = 0x40;
        self.sprite_set_x(k, old_x);
        self.sprite_set_y(k, old_y);
    }

    // void Sprite_TrinexxD_Draw(int k) {  // 9daf84
    pub(super) fn sprite_trinexx_d_draw(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        let info = DrawPrepOamCoordsRet::default();
        self.sprite_draw_trinexx_rock_head(k, &info);

        for i in 0..usize::from(self.ram[SPRITE_ANIM_CLOCK + k]) {
            let j = (self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(K_TRINEXXD_HIST_POS[i]) & 0x7f)
                as usize;
            let cur_x = ((self.ram[MOLDORM_X_HI_SB + j] as u16) << 8)
                | self.ram[MOLDORM_X_LO_SB + j] as u16;
            let cur_y = ((self.ram[MOLDORM_Y_HI_SB + j] as u16) << 8)
                | self.ram[MOLDORM_Y_LO_SB + j] as u16;
            write_le_u16(&mut self.ram, CUR_SPRITE_X, cur_x);
            write_le_u16(&mut self.ram, CUR_SPRITE_Y, cur_y);

            let link_x = self.player_state_view().x();
            let link_y = self.player_state_view().y();
            if link_x.wrapping_sub(cur_x).wrapping_add(8) < 16
                && link_y.wrapping_sub(cur_y).wrapping_add(16) < 16
                && !sign8(self.ram[SPRITE_AI_STATE + k])
                && (self.ram[COUNTDOWN_FOR_BLINK]
                    | self.ram[LINK_DISABLE_SPRITE_DAMAGE]
                    | self.frame_control_view().submodule()
                    | self.ram[FLAG_UNK1])
                    == 0
            {
                self.ram[LINK_AUXILIARY_STATE] = 1;
                self.ram[LINK_GIVE_DAMAGE] = 8;
                self.ram[LINK_INCAPACITATED_TIMER] = 16;
                self.ram[LINK_ACTUAL_VEL_X] ^= 255;
                self.ram[LINK_ACTUAL_VEL_Y] ^= 255;
            }

            let oam = read_le_u16(&self.ram, OAM_CUR_PTR).wrapping_add(K_TRINEXXD_OAM_OFFS[i]);
            let ext =
                read_le_u16(&self.ram, OAM_EXT_CUR_PTR).wrapping_add(K_TRINEXXD_OAM_OFFS[i] >> 2);
            write_le_u16(&mut self.ram, OAM_CUR_PTR, oam);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext);

            self.ram[SPRITE_OAM_FLAGS + k] = 1;
            if i == 4 && self.ram[SPRITE_AI_STATE + k] != 0 {
                self.sprite_trinexx_check_damage_to_flashing_segment(k);
                self.ram[SPRITE_OAM_FLAGS + k] =
                    (self.ram[SPRITE_SUBTYPE2 + k] & 6) ^ self.ram[SPRITE_OAM_FLAGS + k];
            }

            self.ram[SPRITE_GRAPHICS + k] = K_SPRITE_TRINEXXD_GFX2[i];
            if self.ram[SPRITE_GRAPHICS + k] != 3 {
                self.sprite_draw_single_large(k);
            } else {
                self.ram[SPRITE_GRAPHICS + k] = 8;
                self.sprite_draw_trinexx_rock_head(k, &info);
            }
        }
        self.ram[SMALL_BOSS_SHARED_SCRATCH_A] = self.ram[SPRITE_ANIM_CLOCK + k];
    }

    // void Sprite_CB_TrinexxRockHead(int k) {  // 9db0ca
    pub(super) fn sprite_cb_trinexx_rock_head(&mut self, k: usize) {
        if self.ram[OVERLORD_X_HI_SB] != 0 {
            self.sprite_trinexx_final_phase(k);
            return;
        }
        self.ram[TM_COPY] = 0x17;
        self.ram[TS_COPY] = 0;
        self.sprite_draw_trinexx_rock_head_and_body_for_small_bosses(k);
        if self.sprite_return_if_inactive_for_small_bosses(k) {
            return;
        }
        if (self.ram[SPRITE_AI_STATE + k] as i8).is_negative() {
            self.ram[FLAG_BLOCK_LINK_MENU] = self.ram[SPRITE_AI_STATE + k];
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[OVERLORD_X_HI_SB] = self.ram[OVERLORD_X_HI_SB].wrapping_add(1);
                self.sprite_initialized_segmented_for_small_bosses(k);
                self.ram[SPRITE_SUBTYPE2 + k] = 0;
                self.ram[SPRITE_HEAD_DIR + k] = 0;
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
                self.ram[SPRITE_DEFL_BITS + k] = 0x80;
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_D + k] = 0;
                self.ram[SPRITE_A + k] = 128;
                self.ram[SPRITE_ANIM_CLOCK + k] = 16;
                self.ram[SPRITE_X_VEL + k] = 0;
                self.ram[SPRITE_Y_VEL + k] = 0;
                self.ram[DUNG_FLOOR_Y_VEL + 1] = 255;
            } else if self.ram[SPRITE_DELAY_MAIN + k] >= 0xff {
            } else if self.ram[SPRITE_DELAY_MAIN + k] >= 0xe0 {
                if (self.ram[SPRITE_DELAY_MAIN + k] & 3) == 0 {
                    write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL, 0xffff);
                    self.ram[DUNG_HDR_COLLISION_2_MIRROR] = 1;
                }
                self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                self.sprite_move_y(k);
                self.trinexx_cache_position(k);
                self.ram[SPRITE_C + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(12);
                self.ram[OVERLORD_X_LO + 7] = self.ram[OVERLORD_X_LO + 7].wrapping_add(2);
            } else if self.ram[SPRITE_DELAY_MAIN + k] < 0xe0 {
                if (self.ram[SPRITE_DELAY_MAIN + k] & 3) == 0 {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                }
                if (self.ram[SPRITE_DELAY_MAIN + k] & 1) == 0 {
                    const X0: [i8; 8] = [0, 8, 16, 24, -24, -16, -8, 0];
                    const Y0: [i8; 8] = [0, 8, 16, 24, -24, -16, -8, 0];
                    let xi = (self.get_random_number() & 7) as usize;
                    let yi = (self.get_random_number() & 7) as usize;
                    let x = self.sprite_get_x(k).wrapping_add_signed(i16::from(X0[xi]));
                    let y = self
                        .sprite_get_y(k)
                        .wrapping_add_signed(i16::from(Y0[yi]))
                        .wrapping_sub(8);
                    write_le_u16(&mut self.ram, CUR_SPRITE_X, x);
                    write_le_u16(&mut self.ram, CUR_SPRITE_Y, y);
                    self.sprite_make_boss_death_explosion_no_sound(k);
                }
                self.ram[SPRITE_HEAD_DIR + k] = 255;
            }
            return;
        }
        if (self.ram[SPRITE_STATE + 1] | self.ram[SPRITE_STATE + 2]) == 0
            && self.ram[SPRITE_AI_STATE + k] < 2
        {
            self.ram[SPRITE_DELAY_MAIN + k] = 255;
            self.ram[SPRITE_AI_STATE + k] = 255;
            self.ram[SOUND_EFFECT_2] = 0x22;
            return;
        }
        self.trinexx_wag_tail(k);
        self.trinexx_handle_shell_collision(k);
        self.sprite_check_damage_to_and_from_link_for_small_bosses(k);
        if (self.ram[FRAME_COUNTER] & 63) == 0 {
            let pair = self.sprite_is_right_of_link(k);
            self.ram[SPRITE_GRAPHICS + k] = if pair.b.wrapping_add(24) < 48 {
                0
            } else if pair.a != 0 {
                1
            } else {
                7
            };
        }
        if self.ram[OVERLORD_X_LO + 6] != 0 {
            if (self.ram[FRAME_COUNTER] & 1) == 0 {
                self.ram[OVERLORD_X_LO + 6] = self.ram[OVERLORD_X_LO + 6].wrapping_sub(1);
            }
            return;
        }
        if self.ram[SPRITE_STATE + 1] != 0 && self.ram[SPRITE_AI_STATE + 1] == 3 {
            return;
        }
        if self.ram[SPRITE_STATE + 2] != 0 && self.ram[SPRITE_AI_STATE + 2] == 3 {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let j = self.get_random_number() & 3;
                    if (self.ram[SPRITE_SUBTYPE + k] & 0x7f) == j {
                        return;
                    }
                    self.ram[SPRITE_ANIM_CLOCK + k] =
                        self.ram[SPRITE_ANIM_CLOCK + k].wrapping_add(1);
                    if self.ram[SPRITE_ANIM_CLOCK + k] == 2 {
                        self.ram[SPRITE_ANIM_CLOCK + k] = 0;
                        self.ram[SPRITE_AI_STATE + k] = 2;
                        self.ram[SPRITE_DELAY_MAIN + k] = 80;
                        return;
                    }
                    const TRINEXX_TAB0: [u8; 4] = [0x60, 0x78, 0x78, 0x90];
                    const TRINEXX_TAB1: [u8; 4] = [0x80, 0x70, 0x60, 0x80];
                    self.ram[OVERLORD_X_LO] = TRINEXX_TAB0[j as usize];
                    self.ram[OVERLORD_X_LO + 1] = TRINEXX_TAB1[j as usize];
                    self.ram[SPRITE_SUBTYPE + k] =
                        j.wrapping_add(u8::from((self.get_random_number() & 3) == 0) * 0x80);
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            }
            1 => {
                if self.ram[SPRITE_SUBTYPE + k] == 0xff
                    && (self.ram[SPRITE_DELAY_MAIN + k] == 0 || self.sprite_is_below_link(k).a == 0)
                {
                    self.ram[SPRITE_SUBTYPE + k] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                } else {
                    let x = ((self.ram[SPRITE_X_HI + k] as u16) << 8)
                        | u16::from(self.ram[OVERLORD_X_LO]);
                    let y = ((self.ram[SPRITE_Y_HI + k] as u16) << 8)
                        | u16::from(self.ram[OVERLORD_X_LO + 1]);
                    let speed = if (self.ram[SPRITE_SUBTYPE + k] as i8).is_negative() {
                        16
                    } else {
                        8
                    };
                    let pt = self.sprite_project_speed_towards_location(k, x, y, speed);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;

                    let bak_x = self.ram[SPRITE_X_LO + k];
                    let bak_y = self.ram[SPRITE_Y_LO + k];
                    self.sprite_move_xy(k);
                    let floor_y_vel =
                        bak_y.wrapping_sub(self.ram[SPRITE_Y_LO + k]) as i8 as i16 as u16;
                    let floor_x_vel =
                        bak_x.wrapping_sub(self.ram[SPRITE_X_LO + k]) as i8 as i16 as u16;
                    write_le_u16(&mut self.ram, DUNG_FLOOR_Y_VEL, floor_y_vel);
                    write_le_u16(&mut self.ram, DUNG_FLOOR_X_VEL, floor_x_vel);
                    self.ram[DUNG_HDR_COLLISION_2_MIRROR] = 1;
                    self.trinexx_cache_position(k);
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(12);
                    if self.ram[OVERLORD_X_LO]
                        .wrapping_sub(self.ram[SPRITE_X_LO + k])
                        .wrapping_add(2)
                        < 4
                        && self.ram[OVERLORD_X_LO + 1]
                            .wrapping_sub(self.ram[SPRITE_Y_LO + k])
                            .wrapping_add(2)
                            < 4
                    {
                        self.ram[SPRITE_AI_STATE + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 48;
                    }
                }

                let mut i = if (self.ram[SPRITE_SUBTYPE + k] as i8).is_negative() {
                    2
                } else {
                    1
                };
                loop {
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(
                        if (self.ram[SPRITE_X_VEL + k] as i8).is_negative() {
                            1
                        } else {
                            0xff
                        },
                    );
                    if (self.ram[SPRITE_SUBTYPE2 + k] & 0x0f) == 0 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                    }
                    i -= 1;
                    if i == 0 {
                        break;
                    }
                }
            }
            2 => {
                self.trinexx_wag_tail(k);
                self.trinexx_wag_tail(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.sprite_apply_speed_towards_link(k, 48);
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                    self.ram[SOUND_EFFECT_2] = 0x26;
                }
            }
            3 => {
                self.sprite_move_xy(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.trinexx_restore_xy(k);
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 0x20 {
                    self.ram[SPRITE_X_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                    self.ram[SPRITE_Y_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_Y_VEL + k]);
                }
            }
            _ => {}
        }
    }

    // void Trinexx_WagTail(int k) {  // 9db3b5
    //   if (!overlord_x_lo[5]) {
    //     if (!(++overlord_x_lo[4] & 3)) {
    //       int j = overlord_x_lo[3] & 1;
    //       overlord_x_lo[2] += j ? -1 : 1;
    //       if (overlord_x_lo[2] == (j ? 0 : 6)) {
    //         overlord_x_lo[3] += 1;
    //         overlord_x_lo[5] = 8;
    //       }
    //     }
    //   } else {
    //     --overlord_x_lo[5];
    //   }
    // }
    pub(super) fn trinexx_wag_tail(&mut self, _k: usize) {
        if self.ram[OVERLORD_X_LO + 5] == 0 {
            self.ram[OVERLORD_X_LO + 4] = self.ram[OVERLORD_X_LO + 4].wrapping_add(1);
            if (self.ram[OVERLORD_X_LO + 4] & 3) == 0 {
                let j = (self.ram[OVERLORD_X_LO + 3] & 1) as usize;
                let delta: i8 = if j != 0 { -1 } else { 1 };
                self.ram[OVERLORD_X_LO + 2] = self.ram[OVERLORD_X_LO + 2].wrapping_add(delta as u8);
                let limit: u8 = if j != 0 { 0 } else { 6 };
                if self.ram[OVERLORD_X_LO + 2] == limit {
                    self.ram[OVERLORD_X_LO + 3] = self.ram[OVERLORD_X_LO + 3].wrapping_add(1);
                    self.ram[OVERLORD_X_LO + 5] = 8;
                }
            }
        } else {
            self.ram[OVERLORD_X_LO + 5] = self.ram[OVERLORD_X_LO + 5].wrapping_sub(1);
        }
    }

    // void Trinexx_HandleShellCollision(int k) {  // 9db3e6
    //   uint16 x = sprite_A[k] | sprite_B[k] << 8;
    //   uint16 y = sprite_C[k] | sprite_G[k] << 8;
    //   if ((uint16)(x - link_x_coord + 40) < 80 && (uint16)(y - link_y_coord + 16) < 64 && !(countdown_for_blink | link_disable_sprite_damage)) {
    //     link_auxiliary_state = 1;
    //     link_give_damage = 8;
    //     link_incapacitated_timer = 16;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 32);
    //     link_actual_vel_x = pt.x;
    //     link_actual_vel_y = pt.y;
    //   }
    // }
    pub(super) fn trinexx_handle_shell_collision(&mut self, k: usize) {
        let x = (self.ram[SPRITE_A + k] as u16) | ((self.ram[SPRITE_B + k] as u16) << 8);
        let y = (self.ram[SPRITE_C + k] as u16) | ((self.ram[SPRITE_G + k] as u16) << 8);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        let xd = x.wrapping_sub(link_x).wrapping_add(40);
        let yd = y.wrapping_sub(link_y).wrapping_add(16);
        let no_block = (self.ram[COUNTDOWN_FOR_BLINK] | self.ram[LINK_DISABLE_SPRITE_DAMAGE]) == 0;
        if xd < 80 && yd < 64 && no_block {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_GIVE_DAMAGE] = 8;
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
            let pt = self.sprite_project_speed_towards_link(k, 32);
            self.ram[LINK_ACTUAL_VEL_X] = pt.x as u8;
            self.ram[LINK_ACTUAL_VEL_Y] = pt.y as u8;
        }
    }

    // void Sprite_Sidenexx(int k) {  // 9db8a7
    pub(super) fn sprite_sidenexx(&mut self, k: usize) {
        let idx = self.ram[SPRITE_TYPE + k].wrapping_sub(0xcc) as usize;
        let xx = ((self.ram[SPRITE_B] as u16) << 8) | self.ram[SPRITE_A] as u16;
        let xx = xx.wrapping_add_signed(i16::from(K_TRINEXX_HEAD_XOFFS[idx]));
        self.ram[SPRITE_A + k] = xx as u8;
        self.ram[SPRITE_B + k] = (xx >> 8) as u8;

        let yy =
            (((self.ram[SPRITE_G] as u16) << 8) | self.ram[SPRITE_C] as u16).wrapping_sub(0x20);
        self.ram[SPRITE_C + k] = yy as u8;
        self.ram[SPRITE_G + k] = (yy >> 8) as u8;
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.trinexx_head_draw(k);
        if self.sprite_return_if_inactive_for_small_bosses(k) {
            return;
        }
        if (self.ram[SPRITE_AI_STATE + k] as i8).is_negative() {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] = self.ram[SPRITE_AI_STATE + k];
            self.sidenexx_explode(k);
            return;
        }

        if self.ram[SPRITE_HIT_TIMER + k] != 0 && self.ram[SPRITE_AI_STATE + k] != 4 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_DELAY_MAIN + k] = 128;
            self.ram[SPRITE_AI_STATE + k] = 4;
            self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_OAM_FLAGS + k];
            self.ram[SPRITE_OAM_FLAGS + k] = 3;
        }
        self.sprite_check_damage_to_and_from_link_for_small_bosses(k);
        self.ram[SPRITE_DEFL_BITS + k] |= 4;
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_FLAGS3 + k] |= 0x40;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_SUBTYPE2 + k] = 9;
                    self.ram[SPRITE_FLAGS3 + k] &= !0x40;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let i = (self.get_random_number() & 7).wrapping_add(1);
                    let old_d = self.ram[SPRITE_D + k];
                    if i < 5 && old_d != i {
                        self.ram[SPRITE_D + k] = i;
                        self.ram[SPRITE_AI_STATE + k] = 2;
                        if old_d == 1
                            && (self.get_random_number() & 1) == 0
                            && self.ram[SPRITE_AI_STATE] < 2
                        {
                            self.ram[SPRITE_GRAPHICS + k] = 0;
                            self.ram[SPRITE_AI_STATE + k] = 3;
                            self.ram[SPRITE_DELAY_MAIN + k] = 127;
                        }
                    }
                }
            }
            2 => {
                let mut n = 0u8;
                let mut target = usize::from(self.ram[SPRITE_D + k]) * 9;
                let mut f = k * 9;
                for _ in (0..=8usize).rev() {
                    let a = self.ram[ALT_SPRITE_TYPE_SB + f];
                    let wanted = K_TRINEXX_HEAD_TARGET0[target];
                    if a != wanted {
                        self.ram[ALT_SPRITE_TYPE_SB + f] =
                            a.wrapping_add(if (a.wrapping_sub(wanted) as i8).is_negative() {
                                1
                            } else {
                                0xff
                            });
                        n = n.wrapping_add(1);
                    }
                    let a = self.ram[ALT_SPRITE_TYPE_SB + f];
                    if a != wanted {
                        self.ram[ALT_SPRITE_TYPE_SB + f] =
                            a.wrapping_add(if (a.wrapping_sub(wanted) as i8).is_negative() {
                                1
                            } else {
                                0xff
                            });
                        n = n.wrapping_add(1);
                    }
                    let b = self.ram[ALT_SPRITE_Y_HI_SB + f];
                    let wanted_y = K_TRINEXX_HEAD_TARGET1[target];
                    if b != wanted_y {
                        self.ram[ALT_SPRITE_Y_HI_SB + f] =
                            b.wrapping_add(if (b.wrapping_sub(wanted_y) as i8).is_negative() {
                                1
                            } else {
                                0xff
                            });
                        n = n.wrapping_add(1);
                    }
                    target += 1;
                    f += 1;
                }
                if n == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = self.get_random_number() & 15;
                }
            }
            3 => {
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    return;
                }
                if j == 64 {
                    self.sidenexx_exhale_danger(k);
                }
                self.ram[SPRITE_SUBTYPE + k] = if j < 8 {
                    j
                } else if j < 121 {
                    8
                } else {
                    !(j.wrapping_add(0x80))
                };
                if j >= 64
                    && (self.ram[FRAME_COUNTER]
                        & K_TRINEXX_HEAD_FRAME_MASK[((j - 64) >> 3) as usize])
                        == 0
                {
                    let x = ((self.get_random_number() & 0x0f) as i8).wrapping_sub(3);
                    let y = (self.get_random_number() & 0x0f).wrapping_add(12);
                    let sparkle =
                        self.sprite_garnish_spawn_sparkle(k, x as i16 as u16, u16::from(y));
                    if self.ram[SPRITE_TYPE + k] == 0xcc && sparkle >= 0 {
                        self.ram[GARNISH_TYPE + sparkle as usize] = 0x0e;
                    }
                }
            }
            4 => {
                self.ram[SPRITE_DEFL_BITS + k] &= !4;
                self.ram[SPRITE_SUBTYPE + k] = 0;
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_OAM_FLAGS + k] = self.ram[SPRITE_Z_VEL + k];
                    self.ram[SPRITE_HIT_TIMER + k] = 0;
                }
                if j >= 15 {
                    if (63..78).contains(&j) {
                        if self.ram[SPRITE_TYPE + k] == 0xcd {
                            self.Trinexx_FlashShellPalette_Blue();
                        } else {
                            self.Trinexx_FlashShellPalette_Red();
                        }
                    }
                } else if self.ram[SPRITE_TYPE + k] == 0xcd {
                    self.Trinexx_UnflashShellPalette_Blue();
                } else {
                    self.Trinexx_UnflashShellPalette_Red();
                }
            }
            _ => {}
        }
    }

    // void TrinexxHead_Draw(int k) {  // 9dbb70
    pub(super) fn trinexx_head_draw(&mut self, k: usize) {
        self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_A + k];
        self.ram[SPRITE_X_HI + k] = self.ram[SPRITE_B + k];
        self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_C + k];
        self.ram[SPRITE_Y_HI + k] = self.ram[SPRITE_G + k];
        self.sprite_get16_bit_coords(k);
        let Some((info_x, info_y, info_flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let count = usize::from(self.ram[SPRITE_SUBTYPE2 + k]);
        for i in 0..count {
            let j = i + k * 9;
            let angle = if k != 2 {
                0x100u16.wrapping_add((0u8).wrapping_sub(self.ram[ALT_SPRITE_TYPE_SB + j]) as u16)
            } else {
                u16::from(self.ram[ALT_SPRITE_TYPE_SB + j])
            };
            let radius = self.ram[ALT_SPRITE_Y_HI_SB + j];
            let x_delta = trinexx_head_sin(angle, radius) as u8;
            let y_delta = trinexx_head_sin(angle.wrapping_add(0x80), radius) as u8;
            self.ram[DUNGMAP_VAR7] = x_delta;
            self.ram[DUNGMAP_VAR7 + 1] = y_delta;

            if i == 0 {
                for m in 0..5 {
                    self.ram[CUR_SPRITE_X] = info_x.wrapping_add(u16::from(x_delta)) as u8;
                    let x =
                        self.ram[CUR_SPRITE_X].wrapping_add(K_TRINEXX_HEAD_FIRST_PART_X[m] as u8);
                    self.ram[CUR_SPRITE_Y] = info_y.wrapping_add(u16::from(y_delta)) as u8;
                    let y = self.ram[CUR_SPRITE_Y]
                        .wrapping_add(K_TRINEXX_HEAD_FIRST_PART_Y[m] as u8)
                        .wrapping_add(if m == 4 {
                            self.ram[SPRITE_SUBTYPE + k]
                        } else {
                            0
                        });
                    self.set_oam_plain_for_small_bosses(
                        oam,
                        x,
                        y,
                        K_TRINEXX_HEAD_FIRST_PART_CHAR[m],
                        info_flags | K_TRINEXX_HEAD_FIRST_PART_FLAGS[m],
                        2,
                    );
                    oam += 4;
                }
                let base_x = ((self.ram[SPRITE_B + k] as u16) << 8) | self.ram[SPRITE_A + k] as u16;
                let base_y = ((self.ram[SPRITE_G + k] as u16) << 8) | self.ram[SPRITE_C + k] as u16;
                self.sprite_set_x(k, base_x.wrapping_add(x_delta as i8 as i16 as u16));
                self.sprite_set_y(k, base_y.wrapping_add(y_delta as i8 as i16 as u16));
            } else {
                let x = info_x.wrapping_add(u16::from(x_delta)) as u8;
                let y = info_y.wrapping_add(u16::from(y_delta)) as u8;
                self.ram[CUR_SPRITE_X] = x;
                self.ram[CUR_SPRITE_Y] = y;
                self.set_oam_plain_for_small_bosses(oam, x, y, 8, info_flags, 2);
                oam += 4;
            }
        }
        self.ram[TMP_COUNTER] = self.ram[SPRITE_SUBTYPE2 + k];
        self.ram[SMALL_BOSS_SHARED_SCRATCH_A] = self.ram[SPRITE_SUBTYPE2 + k]
            .wrapping_mul(4)
            .wrapping_add(16);
        if self.frame_control_view().submodule() != 0 {
            self.sprite_correct_oam_entries(k, 4, 2);
        }
    }

    // void Sprite_CC(int k) {  // sprite_main.c:1539
    pub(super) fn sprite_cc(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] == 0 {
            self.sprite_sidenexx(k);
            return;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_trinexx_fire_add_fire_garnish(k);
        self.sprite_cc_cd_common(k);
    }

    // void Sprite_CD(int k) {  // sprite_main.c:1553
    pub(super) fn sprite_cd(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] == 0 {
            self.sprite_sidenexx(k);
            return;
        }
        let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let old_xvel = self.ram[SPRITE_X_VEL + k];
        self.ram[SPRITE_X_VEL + k] =
            self.ram[SPRITE_X_VEL + k].wrapping_add(self.ram[SPRITE_C + k]);
        self.sprite_move_xy(k);
        self.ram[SPRITE_X_VEL + k] = old_xvel;
        self.sprite_cd_spawn_garnish(k);
        self.sprite_cc_cd_common(k);
    }

    // void Sprite_CC_CD_Common(int k) {  // 9dbd44
    pub(super) fn sprite_cc_cd_common(&mut self, k: usize) {
        if (self.ram[FRAME_COUNTER] & 3) == 0 {
            let m: i8 = if self.sprite_is_right_of_link(k).a != 0 {
                -1
            } else {
                1
            };
            if self.ram[SPRITE_X_VEL + k] != (m * 16) as u8 {
                self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(m as u8);
            }
        }
        if self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void Sprite_TrinexxFire_AddFireGarnish(int k) {  // 9dbdd6
    pub(super) fn sprite_trinexx_fire_add_fire_garnish(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if (self.ram[SPRITE_SUBTYPE2 + k] & 7) != 0 {
            return;
        }
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
        self.garnish_flame_trail(k, false);
    }

    // void Vitreous_SpawnSmallerEyes(int k) {  // 9ddecb
    //   sprite_G[k] = 9;
    //   sprite_graphics[k] = 4;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamicallyEx(k, 0x4, &info, 13);
    //   ...
    pub(super) fn vitreous_spawn_smaller_eyes(&mut self, k: usize) {
        self.ram[SPRITE_G + k] = 9;
        self.ram[SPRITE_GRAPHICS + k] = 4;

        // SpriteSpawnInfo info; j = Sprite_SpawnDynamicallyEx(k, 4, &info, 13);
        // The 13-slot variant of SpawnDynamically isn't ported yet — surface
        // it via the local shim which returns the spawn-info x/y or None.
        let info = self.sprite_spawn_dynamically_ex_for_small_bosses(k, 0x4, 13);

        // for (j = 13; j != 0; j--) — the loop runs even if the spawn slot
        // wasn't available; the per-slot writes still happen against slots
        // 1..13 in the 16-slot active table. The C also reuses `info.r0_x`
        // / `info.r2_y` for the per-slot Set X/Y even if `j < 0`.
        let (r0_x, r2_y) = info;

        for j in (1usize..=13).rev() {
            self.ram[SPRITE_STATE + j] = 9;
            self.ram[SPRITE_TYPE + j] = 0xbe;
            self.sprite_prep_load_properties_for_small_bosses(j);
            self.ram[SPRITE_FLOOR + j] = 0;
            self.sprite_set_x(
                j,
                r0_x.wrapping_add(K_VITREOUS_SPAWN_SMALLER_EYES_X[j - 1] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                r2_y.wrapping_add(
                    (K_VITREOUS_SPAWN_SMALLER_EYES_Y[j - 1] as i16).wrapping_add(32) as u16,
                ),
            );
            self.ram[SPRITE_A + j] = self.ram[SPRITE_X_LO + j];
            self.ram[SPRITE_B + j] = self.ram[SPRITE_X_HI + j];
            self.ram[SPRITE_C + j] = self.ram[SPRITE_Y_LO + j];
            self.ram[SPRITE_D + j] = self.ram[SPRITE_Y_HI + j];
            let gfx = K_VITREOUS_SPAWN_SMALLER_EYES_GFX[j - 1];
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = gfx;
            self.ram[SPRITE_GRAPHICS + j] = gfx;
            let rand = self.get_random_number();
            self.ram[SPRITE_SUBTYPE2 + j] = (((j - 1) * 8) as u8).wrapping_add(rand);
        }
    }

    // void Vitreous_Animate(int k, uint8 a) {  // 9de563
    //   static const int8 kVitreous_Animate_Gfx[2] = {2, 1};
    //   if (a == 0x40 || a == 0x41 || a == 0x42)
    //     Sprite_SpawnLightning(k);
    //   sprite_graphics[k] = 0;
    //   PairU8 pair = Sprite_IsRightOfLink(k);
    //   if ((uint8)(pair.b + 16) >= 32)
    //     sprite_graphics[k] = kVitreous_Animate_Gfx[pair.a];
    // }
    pub(super) fn vitreous_animate(&mut self, k: usize, a: u8) {
        if a == 0x40 || a == 0x41 || a == 0x42 {
            self.sprite_spawn_lightning_for_small_bosses(k);
        }
        self.ram[SPRITE_GRAPHICS + k] = 0;
        let pair = self.sprite_is_right_of_link(k);
        if pair.b.wrapping_add(16) >= 32 {
            self.ram[SPRITE_GRAPHICS + k] = K_VITREOUS_ANIMATE_GFX[pair.a as usize] as u8;
        }
    }

    // void Vitreous_SetMinionsForth(int k) {  // 9de5da
    //   static const uint8 kVitreous_WhichToActivate[16] = {5, 6, 7, 8, 9, 10, 11, 12, 13, 5, 6, 7, 8, 9, 10, 11};
    //   if (!(++sprite_subtype2[k] & 63)) {
    //     int j = kVitreous_WhichToActivate[GetRandomNumber() & 15];
    //     if (sprite_ai_state[j] == 0) {
    //       sprite_ai_state[j] = 1;
    //       sound_effect_1 = 0x15;
    //     } else {
    //       sprite_subtype2[k]--;
    //     }
    //   }
    // }
    pub(super) fn vitreous_set_minions_forth(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if (self.ram[SPRITE_SUBTYPE2 + k] & 63) == 0 {
            let rand = self.get_random_number();
            let j = K_VITREOUS_WHICH_TO_ACTIVATE[(rand & 15) as usize] as usize;
            if self.ram[SPRITE_AI_STATE + j] == 0 {
                self.ram[SPRITE_AI_STATE + j] = 1;
                self.ram[SOUND_EFFECT_1] = 0x15;
            } else {
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
            }
        }
    }

    // void Vitreous_Draw(int k) {  // 9de716
    //   static const DrawMultipleData kVitreous_Dmd[24] = { ... };
    //   if (sprite_ai_state[k] == 2 && sprite_state[k] == 9)
    //     oam_cur_ptr = 0x800, oam_ext_cur_ptr = 0xa20;
    //   Sprite_DrawMultiple(k, &kVitreous_Dmd[sprite_graphics[k] * 4], 4, NULL);
    //   if (sprite_ai_state[k] == 2) {
    //     sprite_obj_prio[k] &= ~0xe;
    //     Sprite_DrawLargeShadow2(k);
    //   }
    // }
    pub(super) fn vitreous_draw(&mut self, k: usize) {
        if self.ram[SPRITE_AI_STATE + k] == 2 && self.ram[SPRITE_STATE + k] == 9 {
            write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x800);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa20);
        }
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        self.sprite_draw_multiple_for_small_bosses(k, &K_VITREOUS_DMD, g * 4, 4);
        if self.ram[SPRITE_AI_STATE + k] == 2 {
            self.ram[SPRITE_OBJ_PRIO + k] &= !0x0e;
            self.sprite_draw_large_shadow2_for_small_bosses(k);
        }
    }

    // void Sprite_BF_Lightning(int k) {  // 9de3ed
    pub(super) fn sprite_bf_lightning(&mut self, k: usize) {
        let j = (self.ram[SPRITE_A + k] & 7) as usize;
        self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & 0xb1)
            | K_SPRITE_LIGHTNING_OAM_FLAGS[j]
            | ((self.ram[FRAME_COUNTER] << 1) & 14);
        self.ram[SPRITE_GRAPHICS + k] = K_SPRITE_LIGHTNING_GFX[j]
            + if self.ram[DUNGEON_ROOM_INDEX2] == 0x20 {
                4
            } else {
                0
            };
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) || self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            return;
        }
        self.lightning_spawn_garnish(k);
        self.ram[SPRITE_DELAY_MAIN + k] = 2;
        let y = self.sprite_get_y(k).wrapping_add(16);
        self.sprite_set_y(k, y);
        if self.ram[SPRITE_Y_LO + k].wrapping_sub(self.ram[BG2VOFS_COPY2]) >= 0xd0 {
            self.ram[SPRITE_STATE + k] = 0;
            return;
        }
        let rr = self.get_random_number() & 7;
        let xoff =
            K_SPRITE_LIGHTNING_XOFF[((self.ram[SPRITE_A + k] & 7) as usize) * 8 + rr as usize];
        let x = self.sprite_get_x(k).wrapping_add_signed(i16::from(xoff));
        self.sprite_set_x(k, x);
        self.ram[SPRITE_A + k] = rr;
    }

    // void Sprite_BD_Vitreous(int k) {  // 9de4c8
    pub(super) fn sprite_bd_vitreous(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX4 + k] != 0 {
            self.ram[SPRITE_GRAPHICS + k] = 3;
        }
        self.vitreous_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.vitreous_set_minions_forth(k);
        self.sprite_check_damage_to_and_from_link(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[VITREOUS_EYEBALL_RELEASE_COUNT] = 0;
                self.ram[SPRITE_F + k] = 0;
                self.ram[SPRITE_FLAGS3 + k] |= 64;
                if (self.ram[FRAME_COUNTER] & 1) == 0 {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_sub(1);
                    if self.ram[SPRITE_A + k] == 0 {
                        self.ram[SPRITE_FLAGS3 + k] &= !0x40;
                        self.ram[SPRITE_DELAY_AUX4 + k] = 16;
                        self.ram[SPRITE_AI_STATE + k] = 1;
                        self.ram[SPRITE_DELAY_MAIN + k] = 128;
                        if self.ram[SPRITE_G + k] == 0 {
                            self.ram[SPRITE_AI_STATE + k] = 2;
                            self.ram[SPRITE_DELAY_MAIN + k] = 64;
                            self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                            self.ram[SOUND_EFFECT_1] = 0x35;
                            return;
                        }
                    }
                }
                self.ram[SPRITE_GRAPHICS + k] = if (self.ram[FRAME_COUNTER] & 0x30) != 0 {
                    4
                } else {
                    5
                };
            }
            1 => {
                const A_FROM_G: [u8; 10] =
                    [0x20, 0x20, 0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0, 0];
                self.ram[SPRITE_F + k] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_AUX4 + k] = 16;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_A + k] = A_FROM_G[self.ram[SPRITE_G + k] as usize];
                } else {
                    self.vitreous_animate(k, self.ram[SPRITE_DELAY_MAIN + k]);
                }
            }
            2 => {
                self.vitreous_animate(k, 0x8b);
                if self.sprite_return_if_recoiling(k) {
                    return;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    const XVEL: [i8; 2] = [8, -8];
                    self.ram[SPRITE_X_VEL + k] =
                        XVEL[((self.ram[SPRITE_DELAY_MAIN + k] & 2) >> 1) as usize] as u8;
                    self.sprite_move_x(k);
                } else {
                    self.sprite_move_xyz(k);
                    self.sprite_check_tile_collision(k);
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                    if (self.ram[SPRITE_Z + k] as i8) < 0 {
                        self.ram[SPRITE_Z + k] = 0;
                        self.ram[SPRITE_Z_VEL + k] = 32;
                        self.sprite_apply_speed_towards_link(k, 16);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
                    }
                }
            }
            _ => {}
        }
    }

    // void Sprite_SpawnLightning(int k) {  // 9de612
    pub(super) fn sprite_spawn_lightning(&mut self, k: usize) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xbf, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.ram[SOUND_EFFECT_2] = 0x26;
            self.sprite_set_spawned_coordinates(j, &info);
            let i = self.get_random_number() & 7;
            self.ram[SPRITE_A + j] = i;
            let t_full = i32::from(info.r0_x) + i32::from(K_AGAHNIM_LIGHTING_X[i as usize] as u16);
            self.sprite_set_x(j, t_full as u16);
            self.ram[SPRITE_Y_LO + j] = info
                .r2_y
                .wrapping_add(12)
                .wrapping_add((t_full >> 16) as u16) as u8;
            self.ram[SPRITE_DELAY_MAIN + j] = 2;
            self.ram[INTRO_TIMES_PAL_FLASH] = 32;
        }
    }

    // void Sprite_BE_VitreousEye(int k) {  // 9de773
    pub(super) fn sprite_be_vitreous_eye(&mut self, k: usize) {
        const DX: [i8; 4] = [1, 0, -1, 0];
        const DY: [i8; 4] = [0, 1, 0, -1];
        let j = ((self.ram[SPRITE_SUBTYPE2 + k] >> 4) & 3) as usize;
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X).wrapping_add_signed(i16::from(DX[j]));
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y).wrapping_add_signed(i16::from(DY[j]));
        write_le_u16(&mut self.ram, CUR_SPRITE_X, cur_x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, cur_y);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if self.ram[SPRITE_GRAPHICS + k] != 0 {
            return;
        }
        self.sprite_check_damage_from_link(k);
        self.sprite_check_damage_to_link(k);
        if self.ram[SPRITE_F + k] == 14 {
            self.ram[SPRITE_F + k] = 5;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_G + k] = self.ram[LINK_X_COORD];
                self.ram[SPRITE_HEAD_DIR + k] = self.ram[LINK_X_COORD + 1];
                self.ram[SPRITE_ANIM_CLOCK + k] = self.ram[LINK_Y_COORD];
                self.ram[SPRITE_SUBTYPE + k] = self.ram[LINK_Y_COORD + 1];
            }
            1 => {
                if self.sprite_return_if_recoiling(k) {
                    return;
                }
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 1) == 0 {
                    let x = ((self.ram[SPRITE_HEAD_DIR + k] as u16) << 8)
                        | self.ram[SPRITE_G + k] as u16;
                    let y = ((self.ram[SPRITE_SUBTYPE + k] as u16) << 8)
                        | self.ram[SPRITE_ANIM_CLOCK + k] as u16;
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;
                }
                self.sprite_move_xy(k);
                if self.ram[SPRITE_G + k]
                    .wrapping_sub(self.ram[SPRITE_X_LO + k])
                    .wrapping_add(4)
                    < 8
                    && self.ram[SPRITE_ANIM_CLOCK + k]
                        .wrapping_sub(self.ram[SPRITE_Y_LO + k])
                        .wrapping_add(4)
                        < 8
                {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                }
            }
            2 => {
                if self.sprite_return_if_recoiling(k) {
                    return;
                }
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 1) == 0 {
                    let x = ((self.ram[SPRITE_B + k] as u16) << 8) | self.ram[SPRITE_A + k] as u16;
                    let y = ((self.ram[SPRITE_D + k] as u16) << 8) | self.ram[SPRITE_C + k] as u16;
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;
                }
                self.sprite_move_xy(k);
                if self.ram[SPRITE_A + k]
                    .wrapping_sub(self.ram[SPRITE_X_LO + k])
                    .wrapping_add(4)
                    < 8
                    && self.ram[SPRITE_C + k]
                        .wrapping_sub(self.ram[SPRITE_Y_LO + k])
                        .wrapping_add(4)
                        < 8
                {
                    self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_A + k];
                    self.ram[SPRITE_X_HI + k] = self.ram[SPRITE_B + k];
                    self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_C + k];
                    self.ram[SPRITE_Y_HI + k] = self.ram[SPRITE_D + k];
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
            }
            _ => {}
        }
    }

    // void GenerateIceball(int k) {  // 9e94dd
    //   if (++sprite_subtype2[k] & 127 | sprite_delay_aux1[k])
    //     return;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0xa4, &info);
    //   if (j >= 0) {
    //     Sprite_SetX(j, link_x_coord);
    //     Sprite_SetY(j, link_y_coord);
    //     sprite_z[j] = -32;
    //     sprite_C[j] = -32;
    //     SpriteSfx_QueueSfx2WithPan(j, 0x20);
    //   }
    // }
    pub(super) fn generate_iceball(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if (self.ram[SPRITE_SUBTYPE2 + k] & 127) | self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            return;
        }
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa4, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_set_x(j, self.player_state_view().x());
            self.sprite_set_y(j, self.player_state_view().y());
            self.ram[SPRITE_Z + j] = (-32i8) as u8;
            self.ram[SPRITE_C + j] = (-32i8) as u8;
            self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        }
    }

    // void IceBall_Split(int k) {  // 9e97cf
    //   static const int8 kIceBall_Quadruplicate_Xvel[8] = {0, 32, 0, -32, 24, 24, -24, -24};
    //   static const int8 kIceBall_Quadruplicate_Yvel[8] = {-32, 0, 32, 0, -24, 24, -24, 24};
    //   SpriteSfx_QueueSfx2WithPan(k, 0x1f);
    //   int b = GetRandomNumber() & 4;
    //   for (int i = 3; i >= 0; i--) {
    //     SpriteSpawnInfo info;
    //     int j = Sprite_SpawnDynamically(k, 0xa4, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_ai_state[j] = 1;
    //       sprite_graphics[j] = 1;
    //       sprite_C[j] = 1;
    //       sprite_z_vel[j] = 32;
    //       sprite_x_vel[j] = kIceBall_Quadruplicate_Xvel[i + b];
    //       sprite_y_vel[j] = kIceBall_Quadruplicate_Yvel[i + b];
    //       sprite_flags4[j] = 0x1c;
    //     }
    //   }
    //   tmp_counter = 0xff;
    // }
    pub(super) fn ice_ball_split(&mut self, k: usize) {
        const XVEL: [i8; 8] = [0, 32, 0, -32, 24, 24, -24, -24];
        const YVEL: [i8; 8] = [-32, 0, 32, 0, -24, 24, -24, 24];

        self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
        let b = (self.get_random_number() & 4) as usize;
        for i in (0..=3usize).rev() {
            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xa4, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_AI_STATE + j] = 1;
                self.ram[SPRITE_GRAPHICS + j] = 1;
                self.ram[SPRITE_C + j] = 1;
                self.ram[SPRITE_Z_VEL + j] = 32;
                self.ram[SPRITE_X_VEL + j] = XVEL[i + b] as u8;
                self.ram[SPRITE_Y_VEL + j] = YVEL[i + b] as u8;
                self.ram[SPRITE_FLAGS4 + j] = 0x1c;
            }
        }
        self.ram[TMP_COUNTER] = 0xff;
    }

    // void RedBari_Split(int k) {  // 86a34e
    //   static const int8 kRedBari_SplitX[2] = {0, 8};
    //   static const int8 kRedBari_SplitXvel[2] = {-32, 32};
    //
    //   tmp_counter = 1;
    //   do {
    //     SpriteSpawnInfo info;
    //     int j = Sprite_SpawnDynamically(k, 0x23, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_flags3[j] = 0x33;
    //       sprite_oam_flags[j] = 3;
    //       sprite_flags4[j] = 1;
    //       sprite_C[j] = 1;
    //       Sprite_SetX(j, info.r0_x + kRedBari_SplitX[tmp_counter]);
    //       sprite_x_vel[j] = kRedBari_SplitXvel[tmp_counter];
    //       sprite_delay_aux2[j] = 8;
    //       sprite_delay_aux1[j] = 64;
    //     }
    //   } while (!sign8(--tmp_counter));
    //
    // }
    pub(super) fn red_bari_split(&mut self, k: usize) {
        const X_OFFSET: [i8; 2] = [0, 8];
        const X_VEL: [i8; 2] = [-32, 32];

        self.ram[TMP_COUNTER] = 1;
        loop {
            let idx = self.ram[TMP_COUNTER] as usize;
            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0x23, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_FLAGS3 + j] = 0x33;
                self.ram[SPRITE_OAM_FLAGS + j] = 3;
                self.ram[SPRITE_FLAGS4 + j] = 1;
                self.ram[SPRITE_C + j] = 1;
                self.sprite_set_x(j, info.r0_x.wrapping_add_signed(X_OFFSET[idx] as i16));
                self.ram[SPRITE_X_VEL + j] = X_VEL[idx] as u8;
                self.ram[SPRITE_DELAY_AUX2 + j] = 8;
                self.ram[SPRITE_DELAY_AUX1 + j] = 64;
            }
            self.ram[TMP_COUNTER] = self.ram[TMP_COUNTER].wrapping_sub(1);
            if (self.ram[TMP_COUNTER] as i8) < 0 {
                break;
            }
        }
    }

    // void Sidenexx_ExhaleDanger(int k) {  // 9dbae8
    //   SpriteSpawnInfo info;
    //   if (sprite_type[k] == 0xcd) {
    //     for (int i = 0; i < 2; i++) {
    //       int j = Sprite_SpawnDynamically(k, 0xcd, &info);
    //       if (j >= 0) {
    //         Sprite_SetSpawnedCoordinates(j, &info);
    //         sprite_C[j] = i ? 1 : -2;
    //         SpriteSfx_QueueSfx3WithPan(k, 0x19);
    //         sprite_ignore_projectile[j] = sprite_E[j] = 1;
    //         sprite_y_vel[j] = 24;
    //         sprite_flags2[j] = 0;
    //         sprite_flags3[j] = 0x40;
    //       }
    //     }
    //     SPRITE_SHARED_SCRATCH_A = 1;
    //   } else {
    //     int j = Sprite_SpawnDynamically(k, sprite_type[k], &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       SpriteSfx_QueueSfx2WithPan(k, 0x2a);
    //       sprite_ignore_projectile[j] = sprite_E[j] = 1;
    //       sprite_y_vel[j] = 24;
    //       sprite_flags2[j] = 0;
    //       sprite_flags3[j] = 0x40;
    //     }
    //   }
    // }
    pub(super) fn sidenexx_exhale_danger(&mut self, k: usize) {
        if self.ram[SPRITE_TYPE + k] == 0xcd {
            for i in 0..2usize {
                let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                let j = self.sprite_spawn_dynamically(k, 0xcd, &mut info);
                if j >= 0 {
                    let j = j as usize;
                    self.sprite_set_spawned_coordinates(j, &info);
                    self.ram[SPRITE_C + j] = if i != 0 { 1 } else { (-2i8) as u8 };
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
                    self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
                    self.ram[SPRITE_E + j] = 1;
                    self.ram[SPRITE_Y_VEL + j] = 24;
                    self.ram[SPRITE_FLAGS2 + j] = 0;
                    self.ram[SPRITE_FLAGS3 + j] = 0x40;
                }
            }
            self.ram[SMALL_BOSS_SHARED_SCRATCH_A] = 1;
        } else {
            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, self.ram[SPRITE_TYPE + k], &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x2a);
                self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
                self.ram[SPRITE_E + j] = 1;
                self.ram[SPRITE_Y_VEL + j] = 24;
                self.ram[SPRITE_FLAGS2 + j] = 0;
                self.ram[SPRITE_FLAGS3 + j] = 0x40;
            }
        }
    }

    // bool SpikeBlock_CheckStatueCollision(int k) {  // 9ebe19
    //   for (int j = 15; j >= 0; j--) {
    //     if (!((j ^ frame_counter) & 1) && sprite_state[j] && sprite_type[j] == 0x1c) {
    //       int x0 = Sprite_GetX(k), y0 = Sprite_GetY(k);
    //       int x1 = Sprite_GetX(j), y1 = Sprite_GetY(j);
    //       if ((uint16)(x0 - x1 + 16) < 32 && (uint16)(y0 - y1 + 16) < 32)
    //         return false;
    //     }
    //   }
    //   return true;
    // }
    pub(super) fn spike_block_check_statue_collision(&mut self, k: usize) -> bool {
        for j in (0..16usize).rev() {
            if (((j as u8) ^ self.ram[FRAME_COUNTER]) & 1) == 0
                && self.ram[SPRITE_STATE + j] != 0
                && self.ram[SPRITE_TYPE + j] == 0x1c
            {
                let x0 = self.sprite_get_x(k);
                let y0 = self.sprite_get_y(k);
                let x1 = self.sprite_get_x(j);
                let y1 = self.sprite_get_y(j);
                if x0.wrapping_sub(x1).wrapping_add(16) < 32
                    && y0.wrapping_sub(y1).wrapping_add(16) < 32
                {
                    return false;
                }
            }
        }
        true
    }

    // void Sidenexx_Explode(int k) {  // 9dbb3f
    pub(super) fn sidenexx_explode(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_DELAY_MAIN + k] = 12;
            if self.ram[SPRITE_SUBTYPE2 + k] == 1 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
            self.ram[CUR_SPRITE_X] = self.ram[CUR_SPRITE_X].wrapping_add(self.ram[BG2HOFS_COPY2]);
            self.ram[CUR_SPRITE_Y] = self.ram[CUR_SPRITE_Y].wrapping_add(self.ram[BG2VOFS_COPY2]);
            self.sprite_make_boss_explosion_for_small_bosses(k);
        }
    }

    // void Sprite_85_YellowStalfos(int k) {  // 9ec37f
    pub(super) fn sprite_85_yellow_stalfos(&mut self, k: usize) {
        if self.ram[SPRITE_A + k] == 0 {
            self.ram[SPRITE_X_VEL + k] = 1;
            self.ram[SPRITE_Y_VEL + k] = 1;
            if self.sprite_check_tile_collision(k) != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                return;
            }
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
            self.ram[SPRITE_C + k] = 10;
            self.ram[SPRITE_FLAGS3 + k] |= 64;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }

        let ai = self.ram[SPRITE_AI_STATE + k] as usize;
        if ai < K_YELLOW_STALFOS_OBJ_PRIO.len() {
            self.ram[SPRITE_OBJ_PRIO + k] |= K_YELLOW_STALFOS_OBJ_PRIO[ai];
        }
        self.yellow_stalfos_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[LINK_SWORD_TYPE] >= 3 {
            if self.sprite_return_if_recoiling(k) {
                return;
            }
        } else if self.ram[SPRITE_AI_STATE + k] != 5 && self.ram[SPRITE_HIT_TIMER + k] != 0 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_AI_STATE + k] = 5;
            self.ram[SPRITE_DELAY_MAIN + k] = 255;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_HEAD_DIR + k] = 2;
                let bak0 = self.ram[SPRITE_Z + k];
                self.sprite_move_z(k);
                if (self.ram[SPRITE_Z_VEL + k].wrapping_sub(192) as i8) >= 0 {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(3);
                }
                if (bak0 as i8) >= 0 && (self.ram[SPRITE_Z + k] as i8) < 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                    self.yellow_stalfos_animate(k);
                }
            }
            1 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_to_and_from_link(k);
                let dir = self.sprite_direction_to_face_link(k, None);
                self.ram[SPRITE_HEAD_DIR + k] = dir;
                self.ram[SPRITE_D + k] = dir;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 127;
                }
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
            }
            2 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_to_and_from_link(k);
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                    return;
                }
                if j == 48 {
                    self.yellow_stalfos_emancipate_head(k);
                }
                let gfx_idx = (((j >> 2) & !3) | self.ram[SPRITE_D + k]) as usize & 31;
                self.ram[SPRITE_GRAPHICS + k] = K_YELLOW_STALFOS_GFX[gfx_idx];
                let idx = (j >> 2) as usize & 31;
                self.ram[SPRITE_B + k] = K_YELLOW_STALFOS_HEAD_X[idx] as u8;
                self.ram[SPRITE_C + k] = K_YELLOW_STALFOS_HEAD_Y[idx];
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
            }
            3 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                }
                self.yellow_stalfos_animate(k);
            }
            4 => {
                self.ram[SPRITE_GRAPHICS + k] = 0;
                self.ram[SPRITE_HEAD_DIR + k] = 2;
                let old_z = self.ram[SPRITE_Z + k];
                self.sprite_move_z(k);
                if (self.ram[SPRITE_Z_VEL + k].wrapping_sub(64) as i8) < 0 {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_add(2);
                }
                if (old_z as i8) < 0 && (self.ram[SPRITE_Z + k] as i8) >= 0 {
                    self.ram[SPRITE_STATE + k] = 0;
                }
            }
            5 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_from_link(k);
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_sub(1);
                }
                let idx = (j >> 4) as usize & 15;
                self.ram[SPRITE_GRAPHICS + k] = K_YELLOW_STALFOS_NEUTRALIZED_GFX[idx];
                self.ram[SPRITE_C + k] = K_YELLOW_STALFOS_NEUTRALIZED_HEAD_Y[idx];
            }
            _ => {}
        }
    }

    // void YellowStalfos_Animate(int k) {  // 9ec509
    //   static const uint8 kYellowStalfos_Gfx2[4] = {6, 3, 1, 1};
    //   sprite_graphics[k] = kYellowStalfos_Gfx2[sprite_D[k]];
    //   sprite_flags3[k] &= ~0x40;
    // }
    pub(super) fn yellow_stalfos_animate(&mut self, k: usize) {
        let d = (self.ram[SPRITE_D + k] & 3) as usize;
        self.ram[SPRITE_GRAPHICS + k] = K_YELLOW_STALFOS_GFX2[d];
        self.ram[SPRITE_FLAGS3 + k] &= !0x40;
    }

    // void YellowStalfos_EmancipateHead(int k) {  // 9ec580
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 2, &info);
    //   if (j >= 0) {
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     sprite_z[j] = 13;
    //     Sprite_ApplySpeedTowardsLink(j, 16);
    //     sprite_delay_main[j] = 255;
    //     sprite_delay_aux1[j] = 32;
    //   }
    // }
    pub(super) fn yellow_stalfos_emancipate_head(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_small_bosses(k, 2) {
            self.sprite_set_spawned_coordinates_for_small_bosses(j, r0_x, r2_y);
            self.ram[SPRITE_Z + j] = 13;
            self.sprite_apply_speed_towards_link(j, 16);
            self.ram[SPRITE_DELAY_MAIN + j] = 255;
            self.ram[SPRITE_DELAY_AUX1 + j] = 32;
        }
    }

    // void YellowStalfos_Draw(int k) {  // 9ec655
    //   static const DrawMultipleData kYellowStalfos_Dmd[22] = { ... };
    //   oam_cur_ptr += 4, oam_ext_cur_ptr++;
    //   PrepOamCoordsRet info;
    //   Sprite_DrawMultiple(k, &kYellowStalfos_Dmd[sprite_graphics[k] * 2], 2, &info);
    //   oam_cur_ptr -= 4, oam_ext_cur_ptr--;
    //   if (!sprite_pause[k]) {
    //     YellowStalfos_DrawHead(k, &info);
    //     SpriteDraw_Shadow(k, &info);
    //   }
    // }
    pub(super) fn yellow_stalfos_draw(&mut self, k: usize) {
        let old_oam = read_le_u16(&self.ram, OAM_CUR_PTR);
        let old_ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR);
        write_le_u16(&mut self.ram, OAM_CUR_PTR, old_oam.wrapping_add(4));
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, old_ext.wrapping_add(1));
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        let info = self.sprite_draw_multiple_for_small_bosses(k, &K_YELLOW_STALFOS_DMD, g * 2, 2);
        write_le_u16(&mut self.ram, OAM_CUR_PTR, old_oam);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, old_ext);
        if self.ram[SPRITE_PAUSE + k] == 0 {
            self.yellow_stalfos_draw_head(k, &info);
            self.sprite_draw_shadow_for_small_bosses(k, &info);
        }
    }

    // void YellowStalfos_DrawHead(int k, PrepOamCoordsRet *info) {  // 9ec69a
    //   static const uint8 kYellowStalfos_Head_Char[4] = {2, 2, 0, 4};
    //   static const uint8 kYellowStalfos_Head_Flags[4] = {0x40, 0, 0, 0};
    //   OamEnt *oam = GetOamCurPtr();
    //   if (sprite_graphics[k] == 10 || sprite_B[k] == 0x80)
    //     return;
    //   int j = sprite_head_dir[k];
    //   SetOamHelper0(oam, info->x + (int8)sprite_B[k], info->y - sprite_C[k],
    //                 kYellowStalfos_Head_Char[j],
    //                 kYellowStalfos_Head_Flags[j] | info->flags, 2);
    // }
    pub(super) fn yellow_stalfos_draw_head(&mut self, k: usize, info: &PrepOamCoordsRet) {
        if self.ram[SPRITE_GRAPHICS + k] == 10 || self.ram[SPRITE_B + k] == 0x80 {
            return;
        }
        let j = (self.ram[SPRITE_HEAD_DIR + k] & 3) as usize;
        let x = info
            .x
            .wrapping_add(self.ram[SPRITE_B + k] as i8 as i16 as u16);
        let y = info.y.wrapping_sub(self.ram[SPRITE_C + k] as u16);
        let charnum = K_YELLOW_STALFOS_HEAD_CHAR[j];
        let flags = K_YELLOW_STALFOS_HEAD_FLAGS[j] | info.flags;
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        self.set_oam_helper0_for_small_bosses(oam, x, y, charnum, flags, 2);
    }

    // void Sprite_EvilBarrier(int k) {  // 9df06b
    pub(super) fn sprite_evil_barrier(&mut self, k: usize) {
        self.evil_barrier_draw(k);
        if self.ram[SPRITE_GRAPHICS + k] == 4 {
            return;
        }

        self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER] >> 1) & 3;
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_check_damage_from_link(k) != 0 && self.ram[LINK_SWORD_TYPE] < 2 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.sprite_attempt_damage_to_link_plus_recoil(k);
            if self.ram[COUNTDOWN_FOR_BLINK] == 0 {
                self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 64;
            }
        }

        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        if link_y.wrapping_sub(cur_y).wrapping_add(8) < 24
            && link_x.wrapping_sub(cur_x).wrapping_add(32) < 64
            && sign8(self.ram[LINK_ACTUAL_VEL_Y].wrapping_sub(1))
        {
            self.ram[LINK_ELECTROCUTE_ON_TOUCH] = 64;
            self.ram[LINK_INCAPACITATED_TIMER] = 12;
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_GIVE_DAMAGE] = 2;
            self.ram[LINK_ACTUAL_VEL_X] = 0;
            self.ram[LINK_ACTUAL_VEL_Y] = 48;
        }
    }

    // -----------------------------------------------------------------
    // Local helpers (named with `_for_small_bosses` suffix to keep call sites
    // aligned with the C source). These route to canonical helpers when
    // available; remaining OAM/collision gaps stay conservative.
    // -----------------------------------------------------------------

    fn sprite_return_if_inactive_for_small_bosses(&mut self, k: usize) -> bool {
        self.sprite_return_if_inactive(k)
    }

    fn sprite_move_xy_for_small_bosses(&mut self, k: usize) {
        self.sprite_move_xy(k);
    }

    fn sprite_check_damage_to_and_from_link_for_small_bosses(&mut self, k: usize) {
        // Rewired to canonical Sprite_CheckDamageToAndFromLink port.
        self.sprite_check_damage_to_and_from_link(k);
    }

    fn sprite_check_damage_from_link_for_small_bosses(&mut self, k: usize) {
        let _ = self.sprite_check_damage_from_link(k);
    }

    fn sprite_get_16bit_coords_for_small_bosses(&mut self, k: usize) {
        self.sprite_get16_bit_coords(k);
    }

    fn sprite_check_tile_collision_for_small_bosses(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_CheckTileCollision port. Boss callers
        // key off "any wall hit" via a non-zero check.
        self.sprite_check_tile_collision(k) != 0
    }

    fn sprite_approach_target_speed_for_small_bosses(&mut self, k: usize, tx: u8, ty: u8) {
        self.sprite_approach_target_speed(k, tx, ty);
    }

    fn sprite_convert_velocity_to_angle_for_small_bosses(&mut self, xv: i8, yv: i8) -> u8 {
        Self::sprite_convert_velocity_to_angle(xv as u8, yv as u8)
    }

    fn sprite_trinexxd_draw_for_small_bosses(&mut self, k: usize) {
        self.sprite_trinexx_d_draw(k);
    }

    fn sprite_draw_trinexx_rock_head_and_body_for_small_bosses(&mut self, k: usize) {
        self.sprite_draw_trinexx_rock_head_and_body(k);
    }

    fn sprite_initialized_segmented_for_small_bosses(&mut self, k: usize) {
        for i in 0..128 {
            self.ram[MOLDORM_X_LO_SB + i] = self.ram[SPRITE_X_LO + k];
            self.ram[MOLDORM_X_HI_SB + i] = self.ram[SPRITE_X_HI + k];
            self.ram[MOLDORM_Y_LO_SB + i] = self.ram[SPRITE_Y_LO + k];
            self.ram[MOLDORM_Y_HI_SB + i] = self.ram[SPRITE_Y_HI + k];
        }
    }

    fn sprite_schedule_boss_for_death_for_small_bosses(&mut self, k: usize) {
        self.sprite_schedule_boss_for_death(k);
    }

    fn sprite_make_boss_explosion_for_small_bosses(&mut self, k: usize) {
        self.sprite_make_boss_explosion(k);
    }

    fn sprite_draw_multiple_for_small_bosses(
        &mut self,
        k: usize,
        src: &[SbDmd],
        start: usize,
        count: usize,
    ) -> PrepOamCoordsRet {
        let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return PrepOamCoordsRet::default();
        };
        let entries: Vec<DrawMultipleData> = src
            .get(start..start.saturating_add(count))
            .unwrap_or(&[])
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect();
        self.sprite_draw_multiple_with_info(k, &entries, prepped);
        PrepOamCoordsRet {
            x: prepped.0,
            y: prepped.1,
            r4: 0,
            flags: prepped.2,
        }
    }

    fn sprite_draw_large_shadow2_for_small_bosses(&mut self, k: usize) {
        self.sprite_draw_large_shadow2(k);
    }

    fn sprite_draw_shadow_for_small_bosses(&mut self, k: usize, info: &PrepOamCoordsRet) {
        let mut canonical = crate::zelda_rtl::sprite::PrepOamCoordsRet {
            x: info.x,
            y: info.y,
            r4: info.r4,
            flags: info.flags,
        };
        self.sprite_draw_shadow_custom(k, &mut canonical, 10);
    }

    fn small_boss_garnish_alloc_overwrite_old(&mut self) -> Option<usize> {
        let j = self.garnish_alloc_overwrite_old();
        if j < 0 {
            None
        } else {
            Some(j as usize)
        }
    }

    fn set_oam_helper0_for_small_bosses(
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

    fn set_oam_plain_for_small_bosses(
        &mut self,
        oam: usize,
        x: u8,
        y: u8,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.ram[oam] = x;
        self.ram[oam + 1] = y;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = flags;
        let ext_index = (oam - OAM_BUF) / 4;
        self.ram[BYTEWISE_EXTENDED_OAM + ext_index] = big;
    }

    fn sprite_spawn_dynamically_for_small_bosses(
        &mut self,
        k: usize,
        what: u8,
    ) -> Option<(usize, u16, u16)> {
        // Rewired to canonical Sprite_SpawnDynamically port.
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, what, &mut info);
        if j < 0 {
            None
        } else {
            Some((j as usize, info.r0_x, info.r2_y))
        }
    }

    fn sprite_set_spawned_coordinates_for_small_bosses(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        // Rewired to canonical Sprite_SetSpawnedCoordinates port.
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    fn sprite_spawn_dynamically_ex_for_small_bosses(
        &mut self,
        k: usize,
        what: u8,
        slot_count: u8,
    ) -> (u16, u16) {
        // Rewired to canonical Sprite_SpawnDynamicallyEx port. The
        // `slot_count` is the inclusive upper-bound `j` from the C
        // signature (sprite.c:4244). Vitreous calls with slot_count=13,
        // matching the canonical use-site.
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let _j = self.sprite_spawn_dynamically_ex(k, what, &mut info, slot_count as i32);
        // The C body always feeds info.r0_x/r2_y to the subsequent per-slot
        // writes regardless of whether `j` was negative; mirror that.
        (info.r0_x, info.r2_y)
    }

    fn sprite_prep_load_properties_for_small_bosses(&mut self, k: usize) {
        // Direct adapter for the canonical SpritePrep_LoadProperties port.
        self.sprite_prep_load_properties(k);
    }

    fn sprite_spawn_lightning_for_small_bosses(&mut self, k: usize) {
        self.sprite_spawn_lightning(k);
    }
}

// Silence dead-code warnings on the unused tables / constants while the
// remaining segmented-history helpers are still being ported.
#[allow(dead_code)]
const _SMALL_BOSSES_TRINEXXD_GFX_KEEPALIVE: &[u8] = &K_SPRITE_TRINEXXD_GFX;
#[allow(dead_code)]
const _SMALL_BOSSES_GFX3_KEEPALIVE: &[u8] = &K_SPRITE_TRINEXXD_GFX3;
#[allow(dead_code)]
const _SMALL_BOSSES_TRINEXX_XVEL_KEEPALIVE: &[i8] = &K_SPRITE_TRINEXXD_XVEL;
#[allow(dead_code)]
const _SMALL_BOSSES_TRINEXX_YVEL_KEEPALIVE: &[i8] = &K_SPRITE_TRINEXXD_YVEL;
#[allow(dead_code)]
const _SMALL_BOSSES_HEAD_CHAR_KEEPALIVE: &[u8] = &K_YELLOW_STALFOS_HEAD_CHAR;
#[allow(dead_code)]
const _SMALL_BOSSES_HEAD_FLAGS_KEEPALIVE: &[u8] = &K_YELLOW_STALFOS_HEAD_FLAGS;
#[allow(dead_code)]
const _SMALL_BOSSES_GFX2_KEEPALIVE: &[u8] = &K_YELLOW_STALFOS_GFX2;
#[allow(dead_code)]
const _SMALL_BOSSES_VITREOUS_ANIMATE_GFX_KEEPALIVE: &[i8] = &K_VITREOUS_ANIMATE_GFX;
#[allow(dead_code)]
const _SMALL_BOSSES_VITREOUS_WHICH_KEEPALIVE: &[u8] = &K_VITREOUS_WHICH_TO_ACTIVATE;
#[allow(dead_code)]
const _SMALL_BOSSES_EYES_X_KEEPALIVE: &[i8] = &K_VITREOUS_SPAWN_SMALLER_EYES_X;
#[allow(dead_code)]
const _SMALL_BOSSES_EYES_Y_KEEPALIVE: &[i8] = &K_VITREOUS_SPAWN_SMALLER_EYES_Y;
#[allow(dead_code)]
const _SMALL_BOSSES_EYES_GFX_KEEPALIVE: &[u8] = &K_VITREOUS_SPAWN_SMALLER_EYES_GFX;
#[allow(dead_code)]
const _SMALL_BOSSES_SCRATCH_KEEPALIVE: &[usize] = &[
    SPRITE_DELAY_AUX3_SB,
    SMALL_BOSS_SHARED_SCRATCH_A,
    OVERLORD_X_HI_SB,
    ALT_SPRITE_TYPE_SB,
    ALT_SPRITE_X_HI_SB,
    ALT_SPRITE_Y_HI_SB,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    #[test]
    fn trinexx_cache_position_writes_all_four_components() {
        // Trinexx_CachePosition copies the current XY (lo/hi) into the
        // sprite scratch fields A/B/C/G. Verify the byte order matches
        // the C source 1:1.
        let mut s = fresh_state();
        let k = 3;
        s.ram[SPRITE_X_LO + k] = 0x40;
        s.ram[SPRITE_X_HI + k] = 0x01;
        s.ram[SPRITE_Y_LO + k] = 0x80;
        s.ram[SPRITE_Y_HI + k] = 0x02;
        s.trinexx_cache_position(k);
        assert_eq!(s.ram[SPRITE_A + k], 0x40);
        assert_eq!(s.ram[SPRITE_B + k], 0x01);
        assert_eq!(s.ram[SPRITE_C + k], 0x80);
        assert_eq!(s.ram[SPRITE_G + k], 0x02);
    }

    #[test]
    fn trinexx_restore_xy_recomputes_y_with_plus_12() {
        // Trinexx_RestoreXY restores X from sprite_A and Y from
        // (sprite_G << 8) + sprite_C + 12.
        let mut s = fresh_state();
        let k = 5;
        s.ram[SPRITE_A + k] = 0x77;
        s.ram[SPRITE_G + k] = 0x01;
        s.ram[SPRITE_C + k] = 0xf0;
        s.trinexx_restore_xy(k);
        assert_eq!(s.ram[SPRITE_X_LO + k], 0x77);
        // (1 << 8) + 0xf0 + 12 = 0x100 + 0xf0 + 0x0c = 0x1fc.
        assert_eq!(s.ram[SPRITE_Y_LO + k], 0xfc);
        assert_eq!(s.ram[SPRITE_Y_HI + k], 0x01);
    }

    #[test]
    fn trinexx_wag_tail_advances_through_cooldown() {
        // overlord_x_lo[5] is the cooldown timer. Non-zero ticks down by
        // one and leaves the rest of the tail state untouched.
        let mut s = fresh_state();
        s.ram[OVERLORD_X_LO + 5] = 4;
        s.ram[OVERLORD_X_LO + 4] = 0;
        s.trinexx_wag_tail(0);
        assert_eq!(s.ram[OVERLORD_X_LO + 5], 3);
        assert_eq!(s.ram[OVERLORD_X_LO + 4], 0);

        // With the cooldown cleared and the step counter at 3, the next
        // call bumps to 4 (the 0&3 branch fires), advances the swing
        // amount, and arms the cooldown when it hits the boundary (6).
        s.ram[OVERLORD_X_LO + 5] = 0;
        s.ram[OVERLORD_X_LO + 4] = 3;
        s.ram[OVERLORD_X_LO + 3] = 0; // direction bit: forward.
        s.ram[OVERLORD_X_LO + 2] = 5;
        s.trinexx_wag_tail(0);
        assert_eq!(s.ram[OVERLORD_X_LO + 4], 4);
        assert_eq!(s.ram[OVERLORD_X_LO + 2], 6);
        assert_eq!(s.ram[OVERLORD_X_LO + 3], 1);
        assert_eq!(s.ram[OVERLORD_X_LO + 5], 8);
    }

    #[test]
    fn vitreous_set_minions_forth_activates_dormant_minion() {
        // sprite_subtype2 increments every call; on the multiple-of-64
        // tick it tries to wake one of the kVitreous_WhichToActivate
        // slots. Use a deterministic random seed so we know which slot.
        let mut s = fresh_state();
        let k = 0;
        // Pre-arm subtype2 so the next increment hits 64.
        s.ram[SPRITE_SUBTYPE2 + k] = 63;
        // Use the get_random_number seed default; whichever minion slot
        // it picks should transition from 0 -> 1.
        let rand_peek = {
            // Snapshot the RNG by mirroring its current state via a clone.
            let mut clone = s.clone();
            clone.get_random_number()
        };
        let pick = K_VITREOUS_WHICH_TO_ACTIVATE[(rand_peek & 15) as usize] as usize;
        // Mark the picked slot dormant so we exercise the activation arm.
        s.ram[SPRITE_AI_STATE + pick] = 0;
        s.vitreous_set_minions_forth(k);
        assert_eq!(s.ram[SPRITE_AI_STATE + pick], 1);
        assert_eq!(s.ram[SOUND_EFFECT_1], 0x15);
        // subtype2 was 63 → bumped to 64 → kept (not rolled back).
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 64);
    }

    #[test]
    fn vitreous_set_minions_forth_rolls_back_when_minion_busy() {
        // Same setup as above but with the picked slot already active —
        // the C code decrements subtype2 back to 63 so the rate-limiter
        // keeps trying.
        let mut s = fresh_state();
        let k = 0;
        s.ram[SPRITE_SUBTYPE2 + k] = 63;
        let rand_peek = {
            let mut clone = s.clone();
            clone.get_random_number()
        };
        let pick = K_VITREOUS_WHICH_TO_ACTIVATE[(rand_peek & 15) as usize] as usize;
        s.ram[SPRITE_AI_STATE + pick] = 2; // anything non-zero.
        s.vitreous_set_minions_forth(k);
        assert_eq!(s.ram[SPRITE_AI_STATE + pick], 2);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 63);
    }

    #[test]
    fn generate_iceball_spawns_at_link_when_counter_wraps() {
        let mut s = fresh_state();
        let k = 1;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0340);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0450);
        s.ram[SPRITE_SUBTYPE2 + k] = 126;
        s.generate_iceball(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 127);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0);

        s.ram[SPRITE_SUBTYPE2 + k] = 127;
        s.generate_iceball(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 128);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0xa4);
        assert_eq!(s.sprite_get_x(15), 0x0340);
        assert_eq!(s.sprite_get_y(15), 0x0450);
        assert_eq!(s.ram[SPRITE_Z + 15], (-32i8) as u8);
        assert_eq!(s.ram[SPRITE_C + 15], (-32i8) as u8);
        assert_eq!(s.ram[SOUND_EFFECT_1] & 0x3f, 0x20);
    }

    #[test]
    fn ice_ball_split_spawns_four_shards_from_source_position() {
        let mut s = fresh_state();
        let k = 2;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0240);
        s.ram[SPRITE_Z + k] = 6;
        s.ice_ball_split(k);
        assert_eq!(s.ram[SOUND_EFFECT_1] & 0x3f, 0x1f);
        assert_eq!(s.ram[TMP_COUNTER], 0xff);

        let first_x = s.ram[SPRITE_X_VEL + 15];
        let b = if first_x == (-32i8) as u8 {
            0usize
        } else {
            4usize
        };
        let xvel = [0i8, 32, 0, -32, 24, 24, -24, -24];
        let yvel = [-32i8, 0, 32, 0, -24, 24, -24, 24];
        for (slot, i) in [(15usize, 3usize), (14, 2), (13, 1), (12, 0)] {
            assert_eq!(s.ram[SPRITE_TYPE + slot], 0xa4);
            assert_eq!(s.sprite_get_x(slot), 0x0120);
            assert_eq!(s.sprite_get_y(slot), 0x0240);
            assert_eq!(s.ram[SPRITE_Z + slot], 6);
            assert_eq!(s.ram[SPRITE_AI_STATE + slot], 1);
            assert_eq!(s.ram[SPRITE_GRAPHICS + slot], 1);
            assert_eq!(s.ram[SPRITE_C + slot], 1);
            assert_eq!(s.ram[SPRITE_Z_VEL + slot], 32);
            assert_eq!(s.ram[SPRITE_X_VEL + slot], xvel[i + b] as u8);
            assert_eq!(s.ram[SPRITE_Y_VEL + slot], yvel[i + b] as u8);
            assert_eq!(s.ram[SPRITE_FLAGS4 + slot], 0x1c);
        }
    }

    #[test]
    fn red_bari_split_spawns_two_children_with_recoil_state() {
        let mut s = fresh_state();
        let k = 4;
        s.sprite_set_x(k, 0x0180);
        s.sprite_set_y(k, 0x0240);
        s.ram[SPRITE_Z + k] = 7;
        s.red_bari_split(k);

        assert_eq!(s.ram[TMP_COUNTER], 0xff);
        for (slot, x, x_vel) in [(15usize, 0x0188u16, 32i8), (14, 0x0180, -32i8)] {
            assert_eq!(s.ram[SPRITE_TYPE + slot], 0x23);
            assert_eq!(s.ram[SPRITE_STATE + slot], 9);
            assert_eq!(s.sprite_get_x(slot), x);
            assert_eq!(s.sprite_get_y(slot), 0x0240);
            assert_eq!(s.ram[SPRITE_Z + slot], 7);
            assert_eq!(s.ram[SPRITE_FLAGS3 + slot], 0x33);
            assert_eq!(s.ram[SPRITE_OAM_FLAGS + slot], 3);
            assert_eq!(s.ram[SPRITE_FLAGS4 + slot], 1);
            assert_eq!(s.ram[SPRITE_C + slot], 1);
            assert_eq!(s.ram[SPRITE_X_VEL + slot], x_vel as u8);
            assert_eq!(s.ram[SPRITE_DELAY_AUX2 + slot], 8);
            assert_eq!(s.ram[SPRITE_DELAY_AUX1 + slot], 64);
        }
    }

    #[test]
    fn sidenexx_exhale_danger_spawns_two_blue_fire_heads() {
        let mut s = fresh_state();
        let k = 1;
        s.ram[SPRITE_TYPE + k] = 0xcd;
        s.ram[SPRITE_FLOOR + k] = 3;
        s.sprite_set_x(k, 0x0108);
        s.sprite_set_y(k, 0x0210);
        s.sidenexx_exhale_danger(k);

        assert_eq!(s.ram[SMALL_BOSS_SHARED_SCRATCH_A], 1);
        assert_eq!(s.ram[SOUND_EFFECT_2] & 0x3f, 0x19);
        for (slot, c) in [(15usize, (-2i8) as u8), (14, 1u8)] {
            assert_eq!(s.ram[SPRITE_TYPE + slot], 0xcd);
            assert_eq!(s.ram[SPRITE_STATE + slot], 9);
            assert_eq!(s.sprite_get_x(slot), 0x0108);
            assert_eq!(s.sprite_get_y(slot), 0x0210);
            assert_eq!(s.ram[SPRITE_FLOOR + slot], 3);
            assert_eq!(s.ram[SPRITE_C + slot], c);
            assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + slot], 1);
            assert_eq!(s.ram[SPRITE_E + slot], 1);
            assert_eq!(s.ram[SPRITE_Y_VEL + slot], 24);
            assert_eq!(s.ram[SPRITE_FLAGS2 + slot], 0);
            assert_eq!(s.ram[SPRITE_FLAGS3 + slot], 0x40);
        }
    }

    #[test]
    fn sidenexx_exhale_danger_spawns_single_matching_red_head() {
        let mut s = fresh_state();
        let k = 2;
        s.ram[SPRITE_TYPE + k] = 0xcc;
        s.sprite_set_x(k, 0x0130);
        s.sprite_set_y(k, 0x0228);
        s.ram[SPRITE_FLAGS2 + 15] = 0xff;
        s.sidenexx_exhale_danger(k);

        assert_eq!(s.ram[SMALL_BOSS_SHARED_SCRATCH_A], 0);
        assert_eq!(s.ram[SOUND_EFFECT_1] & 0x3f, 0x2a);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0xcc);
        assert_eq!(s.ram[SPRITE_STATE + 15], 9);
        assert_eq!(s.sprite_get_x(15), 0x0130);
        assert_eq!(s.sprite_get_y(15), 0x0228);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 15], 1);
        assert_eq!(s.ram[SPRITE_E + 15], 1);
        assert_eq!(s.ram[SPRITE_Y_VEL + 15], 24);
        assert_eq!(s.ram[SPRITE_FLAGS2 + 15], 0);
        assert_eq!(s.ram[SPRITE_FLAGS3 + 15], 0x40);
        assert_eq!(s.ram[SPRITE_TYPE + 14], 0);
    }

    #[test]
    fn spike_block_check_statue_collision_filters_by_frame_parity_and_overlap() {
        let mut s = fresh_state();
        let spike = 2;
        let statue_even = 4;
        let statue_odd = 5;
        s.ram[FRAME_COUNTER] = 0;
        s.sprite_set_x(spike, 0x0100);
        s.sprite_set_y(spike, 0x0200);

        s.ram[SPRITE_STATE + statue_even] = 9;
        s.ram[SPRITE_TYPE + statue_even] = 0x1c;
        s.sprite_set_x(statue_even, 0x010f);
        s.sprite_set_y(statue_even, 0x020f);
        assert!(!s.spike_block_check_statue_collision(spike));

        s.sprite_set_x(statue_even, 0x0120);
        assert!(s.spike_block_check_statue_collision(spike));

        s.ram[SPRITE_STATE + statue_odd] = 9;
        s.ram[SPRITE_TYPE + statue_odd] = 0x1c;
        s.sprite_set_x(statue_odd, 0x0100);
        s.sprite_set_y(statue_odd, 0x0200);
        assert!(s.spike_block_check_statue_collision(spike));

        s.ram[FRAME_COUNTER] = 1;
        assert!(!s.spike_block_check_statue_collision(spike));
    }

    #[test]
    fn yellow_stalfos_animate_maps_d_to_gfx2() {
        // YellowStalfos_Animate: graphics = kYellowStalfos_Gfx2[D];
        // flags3 has bit 0x40 cleared.
        let mut s = fresh_state();
        let k = 2;
        s.ram[SPRITE_FLAGS3 + k] = 0xff;
        for d in 0u8..4 {
            s.ram[SPRITE_D + k] = d;
            s.yellow_stalfos_animate(k);
            assert_eq!(
                s.ram[SPRITE_GRAPHICS + k],
                K_YELLOW_STALFOS_GFX2[d as usize]
            );
            assert_eq!(s.ram[SPRITE_FLAGS3 + k] & 0x40, 0);
            // The non-0x40 bits should remain set.
            assert_eq!(s.ram[SPRITE_FLAGS3 + k], 0xff & !0x40);
            // Restore for the next iteration.
            s.ram[SPRITE_FLAGS3 + k] = 0xff;
        }
    }
}
