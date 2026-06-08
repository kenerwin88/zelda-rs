//! Ported Blind-boss handlers from sprite_main.c.
use super::sprite::{DrawMultipleData, PrepOamCoordsRet};
use super::*;

// Tables shared by Blind head drawing (from sprite_main.c lines 400-401).
const K_BLIND_HEAD_DRAW_CHAR: [u8; 16] = [
    0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a, 0x88, 0x86,
];
const K_BLIND_HEAD_DRAW_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
];

// kBlindPoof_Dmd from sprite_main.c:15819.
const K_BLIND_POOF_DMD: [DrawMultipleData; 37] = [
    DrawMultipleData {
        x: -16,
        y: -20,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -11,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -26,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -20,
        y: -13,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -37,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -27,
        y: -31,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -28,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -20,
        y: -27,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -27,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: -17,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -13,
        char_flags: 0x0586,
        ext: 2,
    },
    DrawMultipleData {
        x: -18,
        y: -37,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -33,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -32,
        y: -32,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -31,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -24,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -23,
        y: -31,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -24,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -29,
        y: -22,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: -22,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -14,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -12,
        y: -32,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -26,
        y: -29,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -22,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: -20,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -26,
        y: -29,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -22,
        char_flags: 0x458a,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: -20,
        char_flags: 0x058a,
        ext: 2,
    },
    DrawMultipleData {
        x: -17,
        y: -27,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: -10,
        y: -26,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: -22,
        char_flags: 0x459b,
        ext: 0,
    },
    DrawMultipleData {
        x: -19,
        y: -16,
        char_flags: 0x459b,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: -12,
        char_flags: 0x059b,
        ext: 0,
    },
    DrawMultipleData {
        x: 0,
        y: 13,
        char_flags: 0x0b20,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 23,
        char_flags: 0x0b22,
        ext: 2,
    },
];

// kBlind_Dmd from sprite_main.c:15863.
const K_BLIND_DMD: [DrawMultipleData; 105] = [
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: 5,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -1,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -11,
        y: 9,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 23,
        char_flags: 0x0ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a88,
        ext: 2,
    },
    DrawMultipleData {
        x: 10,
        y: -2,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a84,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: 8,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -2,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -5,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 5,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a82,
        ext: 2,
    },
    DrawMultipleData {
        x: 18,
        y: 4,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -15,
        y: -1,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aa8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0a80,
        ext: 2,
    },
    DrawMultipleData {
        x: -19,
        y: 3,
        char_flags: 0x0aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: 19,
        y: 3,
        char_flags: 0x4aaa,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 7,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 7,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0ca0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4ca4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 9,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 9,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 2,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 16,
        char_flags: 0x0c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x4c8e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 9,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 16,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 20,
        char_flags: 0x0a8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 23,
        char_flags: 0x0cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 23,
        char_flags: 0x4cac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 23,
        char_flags: 0x0a8c,
        ext: 2,
    },
];

impl ZeldaState {
    // void Sprite_CE_Blind(int k) {  // 9da263
    pub(super) fn sprite_ce_blind(&mut self, k: usize) {
        if (self.ram[SPRITE_A + k] as i8).is_negative() {
            self.sprite_blind_laser(k);
        } else if self.ram[SPRITE_A + k] == 2 {
            self.sprite_blind_head(k);
        } else {
            self.sprite_blind_blind_blind(k);
        }
    }

    // void Sprite_BlindLaser(int k) {  // 9da268
    pub(super) fn sprite_blind_laser(&mut self, k: usize) {
        const GFX: [u8; 16] = [7, 7, 8, 9, 10, 9, 8, 7, 7, 7, 8, 9, 10, 9, 8, 7];
        const OAM_FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80,
        ];
        let j = (self.ram[SPRITE_HEAD_DIR + k] & 15) as usize;
        self.ram[SPRITE_GRAPHICS + k] = GFX[j];
        self.ram[SPRITE_OAM_FLAGS + k] = OAM_FLAGS[j] | 3;
        let mut info = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.sprite_return_if_inactive_for_blind(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            return;
        }
        self.sprite_check_damage_to_link_same_layer(k);
        let x = self
            .sprite_get_x(k)
            .wrapping_add_signed(i16::from(self.ram[SPRITE_X_VEL + k] as i8));
        let y = self
            .sprite_get_y(k)
            .wrapping_add_signed(i16::from(self.ram[SPRITE_Y_VEL + k] as i8));
        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        if self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_DELAY_MAIN + k] = 12;
        }
        self.blind_laser_spawn_trail_garnish(k);
    }

    // void SpritePrep_Blind_PrepareBattle(int k) {  // 9da081
    //   if (follower_indicator != 6 && dung_savegame_state_bits & 0x2000) {
    //     sprite_delay_aux2[k] = 96;
    //     sprite_C[k] = 1;
    //     sprite_D[k] = 2;
    //     sprite_head_dir[k] = 4;
    //     sprite_graphics[k] = 7;
    //     BLIND_HEAD_ANIM_COUNTER = 0;
    //   } else {
    //     sprite_state[k] = 0;
    //   }
    // }
    pub(super) fn sprite_prep_blind_prepare_battle(&mut self, k: usize) {
        let dung_state = read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS);
        if self.ram[FOLLOWER_INDICATOR] != 6 && (dung_state & 0x2000) != 0 {
            self.ram[SPRITE_DELAY_AUX2 + k] = 96;
            self.ram[SPRITE_C + k] = 1;
            self.ram[SPRITE_D + k] = 2;
            self.ram[SPRITE_HEAD_DIR + k] = 4;
            self.ram[SPRITE_GRAPHICS + k] = 7;
            self.ram[BLIND_HEAD_ANIM_COUNTER] = 0;
        } else {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void Sprite_Blind_Head(int k) {  // 9da118
    //   static const uint8 kBlindHead_XposLimit[2] = {0x98, 0x58};
    //   static const uint8 kBlindHead_YposLimit[2] = {0xb0, 0x50};
    //   static const int8 kBlindHead_YvelLimit[2] = {24, -24};
    //   static const int8 kBlindHead_XvelLimit[2] = {32, -32};
    //
    //   sprite_obj_prio[k] |= 48;
    //   SpriteDraw_SingleLarge(k);
    //   OamEnt *oam = GetOamCurPtr();
    //   int j = sprite_head_dir[k];
    //   oam->charnum = kBlindHead_Draw_Char[j];
    //   oam->flags = oam->flags & 0x3f | kBlindHead_Draw_Flags[j];
    //
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   if (sprite_F[k] == 14)
    //     sprite_F[k] = 8;
    //   if (Sprite_ReturnIfRecoiling(k))
    //     return;
    //   if (sign8(--sprite_subtype[k])) {
    //     sprite_subtype[k] = 2;
    //     sprite_head_dir[k] = sprite_head_dir[k] + 1 & 15;
    //   }
    //   if (sprite_delay_main[k])
    //     return;
    //   Sprite_CheckDamageToAndFromLink(k);
    //   sprite_subtype2[k]++;
    //   j = Blind_SpitFireball(k, 0x1f);
    //   if (j >= 0 && sign8(--sprite_z_subpos[k])) {
    //     sprite_z_subpos[k] = 4;
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLink(k, 32);
    //     sprite_x_vel[j] = pt.x;
    //     sprite_y_vel[j] = pt.y;
    //   }
    //   j = sprite_G[k] & 1;
    //   if (sprite_x_vel[k] != (uint8)kBlindHead_XvelLimit[j])
    //     sprite_x_vel[k] += j ? -1 : 1;
    //   if ((sprite_x_lo[k] & ~1) == kBlindHead_XposLimit[j])
    //     sprite_G[k]++;
    //   j = sprite_anim_clock[k] & 1;
    //   if (sprite_y_vel[k] != (uint8)kBlindHead_YvelLimit[j])
    //     sprite_y_vel[k] += j ? -1 : 1;
    //   if ((sprite_y_lo[k] & ~1) == kBlindHead_YposLimit[j])
    //     sprite_anim_clock[k]++;
    //   if (!sprite_F[k])
    //     Sprite_MoveXY(k);
    // }
    pub(super) fn sprite_blind_head(&mut self, k: usize) {
        const K_BLIND_HEAD_XPOS_LIMIT: [u8; 2] = [0x98, 0x58];
        const K_BLIND_HEAD_YPOS_LIMIT: [u8; 2] = [0xb0, 0x50];
        const K_BLIND_HEAD_YVEL_LIMIT: [i8; 2] = [24, -24];
        const K_BLIND_HEAD_XVEL_LIMIT: [i8; 2] = [32, -32];

        self.ram[SPRITE_OBJ_PRIO + k] |= 48;
        self.sprite_draw_single_large_for_blind(k);
        // OamEnt *oam = GetOamCurPtr(); oam->charnum = ...; oam->flags = ...;
        self.blind_head_apply_oam_for_blind(k);

        if self.sprite_return_if_inactive_for_blind(k) {
            return;
        }
        if self.ram[SPRITE_F + k] == 14 {
            self.ram[SPRITE_F + k] = 8;
        }
        if self.sprite_return_if_recoiling_for_blind(k) {
            return;
        }
        let new_sub = self.ram[SPRITE_SUBTYPE + k].wrapping_sub(1);
        self.ram[SPRITE_SUBTYPE + k] = new_sub;
        if (new_sub as i8) < 0 {
            self.ram[SPRITE_SUBTYPE + k] = 2;
            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1) & 15;
        }
        if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            return;
        }
        self.sprite_check_damage_to_and_from_link_for_blind(k);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        let j_ret = self.blind_spit_fireball(k, 0x1f);
        if j_ret >= 0 {
            let zsub = self.ram[SPRITE_Z_SUBPOS + k].wrapping_sub(1);
            self.ram[SPRITE_Z_SUBPOS + k] = zsub;
            if (zsub as i8) < 0 {
                self.ram[SPRITE_Z_SUBPOS + k] = 4;
                let pt = self.sprite_project_speed_towards_link(k, 32);
                let j = j_ret as usize;
                self.ram[SPRITE_X_VEL + j] = pt.x;
                self.ram[SPRITE_Y_VEL + j] = pt.y;
            }
        }
        let mut j = (self.ram[SPRITE_G + k] & 1) as usize;
        if self.ram[SPRITE_X_VEL + k] != K_BLIND_HEAD_XVEL_LIMIT[j] as u8 {
            let delta: i8 = if j != 0 { -1 } else { 1 };
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(delta as u8);
        }
        if (self.ram[SPRITE_X_LO + k] & !1) == K_BLIND_HEAD_XPOS_LIMIT[j] {
            self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
        }
        j = (self.ram[SPRITE_ANIM_CLOCK + k] & 1) as usize;
        if self.ram[SPRITE_Y_VEL + k] != K_BLIND_HEAD_YVEL_LIMIT[j] as u8 {
            let delta: i8 = if j != 0 { -1 } else { 1 };
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_add(delta as u8);
        }
        if (self.ram[SPRITE_Y_LO + k] & !1) == K_BLIND_HEAD_YPOS_LIMIT[j] {
            self.ram[SPRITE_ANIM_CLOCK + k] = self.ram[SPRITE_ANIM_CLOCK + k].wrapping_add(1);
        }
        if self.ram[SPRITE_F + k] == 0 {
            self.sprite_move_xy(k);
        }
    }

    // void Blind_SpawnHead(int k) {  // 9da1ed
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0xce, &info);
    //   if (j >= 0) {
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     sprite_flags3[j] = 0x5b;
    //     sprite_oam_flags[j] = 0x5b & 15;
    //     sprite_defl_bits[j] = 4;
    //     sprite_A[j] = 2;
    //     sprite_flags2[j] = 1;
    //     sprite_flags4[j] = 0;
    //     sprite_flags[j] = 0;
    //     sprite_z[j] = 23;
    //     sprite_y_lo[j] = 23 + info.r2_y;
    //     sprite_G[j] = (info.r0_x >> 7) & 1;
    //     sprite_anim_clock[j] = (info.r2_y >> 7) & 1;
    //     sprite_delay_main[j] = 48;
    //   }
    // }
    pub(super) fn blind_spawn_head(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_blind(k, 0xce) {
            self.sprite_set_spawned_coordinates_for_blind(j, r0_x, r2_y);
            self.ram[SPRITE_FLAGS3 + j] = 0x5b;
            self.ram[SPRITE_OAM_FLAGS + j] = 0x5b & 15;
            self.ram[SPRITE_DEFL_BITS + j] = 4;
            self.ram[SPRITE_A + j] = 2;
            self.ram[SPRITE_FLAGS2 + j] = 1;
            self.ram[SPRITE_FLAGS4 + j] = 0;
            self.ram[SPRITE_FLAGS + j] = 0;
            self.ram[SPRITE_Z + j] = 23;
            self.ram[SPRITE_Y_LO + j] = (23u16.wrapping_add(r2_y)) as u8;
            self.ram[SPRITE_G + j] = ((r0_x >> 7) & 1) as u8;
            self.ram[SPRITE_ANIM_CLOCK + j] = ((r2_y >> 7) & 1) as u8;
            self.ram[SPRITE_DELAY_MAIN + j] = 48;
        }
    }

    // void Sprite_Blind_Blind_Blind(int k) {  // 9da2d2
    //   (large; pasted body in comments below)
    pub(super) fn sprite_blind_blind_blind(&mut self, k: usize) {
        // sprite_obj_prio[k] |= 0x30;
        // Blind_Draw(k);
        // sprite_oam_flags[k] = 1;
        // if (Sprite_ReturnIfInactive(k)) return;
        // uint8 a = sprite_F[k]; if (a) sprite_F[k]--;
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.blind_draw(k);
        self.ram[SPRITE_OAM_FLAGS + k] = 1;
        if self.sprite_return_if_inactive_for_blind(k) {
            return;
        }
        let a = self.ram[SPRITE_F + k];
        if a != 0 {
            self.ram[SPRITE_F + k] = self.ram[SPRITE_F + k].wrapping_sub(1);
        }

        // if (a == 11) { ... }
        if a == 11 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_WALLCOLL + k] = 0;
            if self.ram[SPRITE_DELAY_AUX4 + k] == 0 {
                self.ram[SPRITE_HEALTH + k] = 128;
                self.ram[SPRITE_DELAY_AUX4 + k] = 48;
                self.ram[SPRITE_OAM_FLAGS + k] &= 1;
                let new_zsub = self.ram[SPRITE_Z_SUBPOS + k].wrapping_add(1);
                self.ram[SPRITE_Z_SUBPOS + k] = new_zsub;
                if new_zsub < 3 {
                    self.ram[SPRITE_WALLCOLL + k] = 96;
                    self.ram[SPRITE_SUBTYPE + k] = 1;
                } else {
                    self.ram[SPRITE_Z_SUBPOS + k] = 0;
                    let new_limit = self.ram[SPRITE_LIMIT_INSTANCE].wrapping_add(1);
                    self.ram[SPRITE_LIMIT_INSTANCE] = new_limit;
                    if new_limit == 3 {
                        self.sprite_kill_friends_for_blind();
                        self.ram[SPRITE_STATE + k] = 4;
                        self.ram[SPRITE_A + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 255;
                        self.ram[SPRITE_HIT_TIMER + k] = 255;
                        self.ram[FLAG_BLOCK_LINK_MENU] =
                            self.ram[FLAG_BLOCK_LINK_MENU].wrapping_add(1);
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x22);
                        return;
                    }
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_C + k] = 6;
                    self.ram[SPRITE_DELAY_AUX2 + k] = 255;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 255;
                    self.blind_spawn_head(k);
                }
            }
        }

        // if (sprite_A[k]) { ... return; }
        if self.ram[SPRITE_A + k] != 0 {
            const K_BLIND_GFX0: [u8; 7] = [20, 19, 18, 17, 16, 15, 15];
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            let idx = (self.ram[SPRITE_DELAY_MAIN + k] >> 3) as usize;
            self.ram[SPRITE_GRAPHICS + k] = K_BLIND_GFX0[idx.min(6)];
            return;
        }
        // if (!(++sprite_subtype2[k] & 1)) sprite_delay_main[k]++;
        let new_sub2 = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.ram[SPRITE_SUBTYPE2 + k] = new_sub2;
        if (new_sub2 & 1) == 0 {
            self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + k].wrapping_add(1);
        }

        // if (sprite_delay_aux1[k]) { ... return; }
        if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            self.ram[SPRITE_AI_STATE + k] = 0;
            if self.ram[SPRITE_DELAY_AUX1 + k] == 8 {
                self.blind_spawn_laser(k);
            }
            self.blind_check_bump_damage(k);
            return;
        }
        // BLIND_HEAD_ANIM_COUNTER++;
        self.ram[BLIND_HEAD_ANIM_COUNTER] = self.ram[BLIND_HEAD_ANIM_COUNTER].wrapping_add(1);
        // stunned/ai_state branch
        if self.ram[SPRITE_STUNNED + k] == 0 {
            if self.ram[SPRITE_AI_STATE + k] != 0 {
                self.ram[SPRITE_DELAY_AUX1 + k] = 16;
                self.ram[SPRITE_STUNNED + k] = 128;
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
        } else {
            self.ram[SPRITE_STUNNED + k] = self.ram[SPRITE_STUNNED + k].wrapping_sub(1);
            self.ram[SPRITE_AI_STATE + k] = 0;
        }
        // sprite_x_hi[k] = HIBYTE(link_x_coord); sprite_y_hi[k] = HIBYTE(link_y_coord);
        self.ram[SPRITE_X_HI + k] = self.ram[LINK_X_COORD + 1];
        self.ram[SPRITE_Y_HI + k] = self.ram[LINK_Y_COORD + 1];

        match self.ram[SPRITE_C + k] {
            0 => {
                // blinded
                self.ram[DMA_HEAD_POINTER] = 0;
                self.ram[DMA_BODY_POINTER] = 0xA0;
                if self.ram[SPRITE_DELAY_AUX2 + k] == 0 {
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_AUX2 + k] = 96;
                } else if self.ram[SPRITE_DELAY_AUX2 + k] == 80 {
                    write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x123);
                    self.sprite_show_message_minimal_for_blind();
                } else if self.ram[SPRITE_DELAY_AUX2 + k] == 24 {
                    self.spawn_boss_poof_for_blind(k);
                }
            }
            1 => {
                // retreat to back wall
                self.blind_check_bump_damage(k);
                self.ram[SPRITE_GRAPHICS + k] = 9;
                if self.ram[SPRITE_DELAY_AUX2 + k] == 0 {
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                } else if self.ram[SPRITE_DELAY_AUX2 + k] < 64 {
                    self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                    self.sprite_move_y(k);
                }
                self.blind_animate(k);
                self.ram[SPRITE_HEAD_DIR + k] = 4;
            }
            2 => {
                // oscillate
                const K_OSC_YVEL_TARGET: [i8; 2] = [18, -18];
                const K_OSC_XVEL_TARGET: [i8; 2] = [24, -24];
                const K_OSC_XPOS_TARGET: [u8; 2] = [164, 76];
                self.blind_check_bump_damage(k);
                self.blind_animate(k);
                let sub2 = self.ram[SPRITE_SUBTYPE2 + k];
                let below_a = self.sprite_is_below_link(k).a;
                let cond1 = (sub2 & 127) == 0 && below_a.wrapping_add(2) != self.ram[SPRITE_D + k];
                let cond2 = self.ram[SPRITE_DELAY_MAIN + k] == 0;
                if (cond1 || cond2) && self.ram[SPRITE_X_LO + k] < 0x78 {
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    self.ram[SPRITE_Y_VEL + k] &= !1;
                    self.ram[SPRITE_X_VEL + k] &= !1;
                    self.ram[SPRITE_DELAY_AUX2 + k] = 0x30;
                    return;
                }
                let mut j = (self.ram[SPRITE_B + k] & 1) as usize;
                let delta: i8 = if j != 0 { -1 } else { 1 };
                self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_add(delta as u8);
                if self.ram[SPRITE_Y_VEL + k] == K_OSC_YVEL_TARGET[j] as u8 {
                    self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
                }
                j = (self.ram[SPRITE_G + k] & 1) as usize;
                if self.ram[SPRITE_X_VEL + k] != K_OSC_XVEL_TARGET[j] as u8 {
                    let delta: i8 = if j != 0 { -1 } else { 1 };
                    self.ram[SPRITE_X_VEL + k] =
                        self.ram[SPRITE_X_VEL + k].wrapping_add(delta as u8);
                }
                if (self.ram[SPRITE_X_LO + k] & !1) == K_OSC_XPOS_TARGET[j] {
                    self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
                }
                self.sprite_move_xy(k);
                if self.ram[SPRITE_WALLCOLL + k] != 0 {
                    let wc = self.ram[SPRITE_WALLCOLL + k];
                    self.blind_fireball_flurry(k, wc);
                } else if (self.ram[SPRITE_SUBTYPE2 + k] & 7) == 0 {
                    let hd = self.ram[SPRITE_HEAD_DIR + k] << 2;
                    self.sprite_spawn_probe_always_for_blind(k, hd);
                }
            }
            3 => {
                // switch walls
                self.blind_check_bump_damage(k);
                if self.ram[SPRITE_DELAY_AUX2 + k] != 0 {
                    self.blind_decelerate_x(k);
                    self.sprite_move_x(k);
                    self.blind_decelerate_y(k);
                } else {
                    const K_SW_YVEL_TARGET: [i8; 2] = [64, -64];
                    const K_SW_YPOS_TARGET: [u8; 2] = [0x90, 0x50];
                    let j = (self.ram[SPRITE_D + k].wrapping_sub(2)) as usize;
                    if self.ram[SPRITE_Y_VEL + k] != K_SW_YVEL_TARGET[j] as u8 {
                        let delta: i8 = if j != 0 { -2 } else { 2 };
                        self.ram[SPRITE_Y_VEL + k] =
                            self.ram[SPRITE_Y_VEL + k].wrapping_add(delta as u8);
                    }
                    if (self.ram[SPRITE_Y_LO + k] & !3) == K_SW_YPOS_TARGET[j] {
                        self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                        self.ram[SPRITE_B + k] = self.ram[SPRITE_D + k].wrapping_sub(1);
                    }
                    self.sprite_move_xy(k);
                    self.blind_decelerate_x(k);
                }
            }
            4 => {
                // whirl around
                self.blind_check_bump_damage(k);
                if (self.ram[SPRITE_SUBTYPE2 + k] & 7) == 0 {
                    const K_WA_GFX: [u8; 2] = [0, 9];
                    let j = (self.ram[SPRITE_D + k].wrapping_sub(2)) as usize;
                    if self.ram[SPRITE_GRAPHICS + k] == K_WA_GFX[j] {
                        self.ram[SPRITE_DELAY_MAIN + k] = 254;
                        self.ram[SPRITE_C + k] = 2;
                        self.ram[SPRITE_D + k] ^= 1;
                        self.ram[SPRITE_G + k] = self.ram[SPRITE_X_LO + k] >> 7;
                    } else {
                        let delta: i8 = if j != 0 { 1 } else { -1 };
                        self.ram[SPRITE_GRAPHICS + k] =
                            self.ram[SPRITE_GRAPHICS + k].wrapping_add(delta as u8);
                    }
                }
                self.blind_decelerate_y(k);
            }
            5 => {
                // fireball reprisal
                self.blind_fireball_flurry(k, 0x65);
            }
            6 => {
                // behind the curtain
                self.ram[SPRITE_HIT_TIMER + k] = 0;
                self.ram[SPRITE_HEAD_DIR + k] = 12;
                let aux2 = self.ram[SPRITE_DELAY_AUX2 + k];
                if aux2 == 0 {
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_AUX2 + k] = 39;
                    self.sprite_sfx_queue_sfx1_with_pan(k, 0x13);
                } else if aux2 >= 224 {
                    const K_BC_GFX: [u8; 4] = [14, 13, 12, 10];
                    self.ram[SPRITE_GRAPHICS + k] = K_BC_GFX[((aux2 - 224) >> 3) as usize];
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = 14;
                }
            }
            7 => {
                // rerobe
                if self.ram[SPRITE_DELAY_AUX2 + k] == 0 {
                    self.ram[SPRITE_C + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 128;
                    self.ram[SPRITE_D + k] = (self.ram[SPRITE_Y_LO + k] >> 7).wrapping_add(2);
                    self.ram[SPRITE_G + k] =
                        (self.ram[SPRITE_X_LO + k] << 2) | (self.ram[SPRITE_X_LO + k] >> 7);
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                } else {
                    const K_RR_GFX: [u8; 5] = [10, 11, 12, 13, 14];
                    let idx = (self.ram[SPRITE_DELAY_AUX2 + k] >> 3) as usize;
                    self.ram[SPRITE_GRAPHICS + k] = K_RR_GFX[idx.min(4)];
                }
            }
            _ => {}
        }
    }

    // void Blind_AnimateRobes(int k) {  // 9da729
    //   static const uint8 kBlind_Gfx_Animate[8] = {7, 8, 9, 8, 0, 1, 2, 1};
    //   sprite_graphics[k] = kBlind_Gfx_Animate[(sprite_subtype2[k] >> 3 & 3) + ((sprite_D[k] - 2) << 2)];
    // }
    pub(super) fn blind_animate_robes(&mut self, k: usize) {
        const K_BLIND_GFX_ANIMATE: [u8; 8] = [7, 8, 9, 8, 0, 1, 2, 1];
        let s2 = (self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 3;
        let d_minus_2 = (self.ram[SPRITE_D + k] as i8).wrapping_sub(2);
        let idx = (s2 as i32) + ((d_minus_2 as i32) << 2);
        self.ram[SPRITE_GRAPHICS + k] = K_BLIND_GFX_ANIMATE[(idx as usize) & 7];
    }

    // void Blind_FireballFlurry(int k, uint8 a) {  // 9da465
    //   sprite_wallcoll[k]--;
    //   sprite_oam_flags[k] = (a & 7) * 2 + 1;
    //   if (sign8(--sprite_E[k])) {
    //     sprite_E[k] = sprite_subtype[k];
    //     sprite_head_dir[k] = sprite_head_dir[k] + 1 & 15;
    //   }
    //   if (!(sprite_subtype2[k] & 31) && sprite_subtype[k] != 5)
    //     sprite_subtype[k]++;
    //   Blind_AnimateRobes(k);
    //   Blind_SpitFireball(k, 0xf);
    // }
    pub(super) fn blind_fireball_flurry(&mut self, k: usize, a: u8) {
        self.ram[SPRITE_WALLCOLL + k] = self.ram[SPRITE_WALLCOLL + k].wrapping_sub(1);
        self.ram[SPRITE_OAM_FLAGS + k] = (a & 7).wrapping_mul(2).wrapping_add(1);
        let new_e = self.ram[SPRITE_E + k].wrapping_sub(1);
        self.ram[SPRITE_E + k] = new_e;
        if (new_e as i8) < 0 {
            self.ram[SPRITE_E + k] = self.ram[SPRITE_SUBTYPE + k];
            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_add(1) & 15;
        }
        if (self.ram[SPRITE_SUBTYPE2 + k] & 31) == 0 && self.ram[SPRITE_SUBTYPE + k] != 5 {
            self.ram[SPRITE_SUBTYPE + k] = self.ram[SPRITE_SUBTYPE + k].wrapping_add(1);
        }
        self.blind_animate_robes(k);
        let _ = self.blind_spit_fireball(k, 0xf);
    }

    // int Blind_SpitFireball(int k, uint8 a) {  // 9da49d
    //   static const int8 kBlindHead_SpawnFireball_Xvel[16] = {-32,-28,-24,-16,0,16,24,28,32,28,24,16,0,-16,-24,-28};
    //   static const int8 kBlindHead_SpawnFireball_Yvel[16] = {0,16,24,28,32,28,24,16,0,-16,-24,-28,-32,-28,-24,-16};
    //   if (sprite_subtype2[k] & a)
    //     return -1;
    //   int j = Sprite_SpawnFireball(k);
    //   if (j >= 0) {
    //     SpriteSfx_QueueSfx3WithPan(k, 0x19);
    //     int i = sprite_head_dir[k];
    //     sprite_x_vel[j] = kBlindHead_SpawnFireball_Xvel[i];
    //     sprite_y_vel[j] = kBlindHead_SpawnFireball_Yvel[i];
    //     sprite_defl_bits[j] |= 8;
    //     sprite_bump_damage[j] = 4;
    //   }
    //   return j;
    // }
    pub(super) fn blind_spit_fireball(&mut self, k: usize, a: u8) -> i32 {
        const K_BLIND_HEAD_SPAWN_FIREBALL_XVEL: [i8; 16] = [
            -32, -28, -24, -16, 0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28,
        ];
        const K_BLIND_HEAD_SPAWN_FIREBALL_YVEL: [i8; 16] = [
            0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
        ];
        if (self.ram[SPRITE_SUBTYPE2 + k] & a) != 0 {
            return -1;
        }
        let j = self.sprite_spawn_fireball(k);
        match j {
            j if j >= 0 => {
                let j = j as usize;
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
                let i = self.ram[SPRITE_HEAD_DIR + k] as usize;
                self.ram[SPRITE_X_VEL + j] = K_BLIND_HEAD_SPAWN_FIREBALL_XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = K_BLIND_HEAD_SPAWN_FIREBALL_YVEL[i] as u8;
                self.ram[SPRITE_DEFL_BITS + j] |= 8;
                self.ram[SPRITE_BUMP_DAMAGE + j] = 4;
                j as i32
            }
            _ => -1,
        }
    }

    // void Blind_Decelerate_X(int k) {  // 9da647
    //   if (sprite_x_vel[k] != 0)
    //     sprite_x_vel[k] += sign8(sprite_x_vel[k]) ? 2 : -2;
    //   Blind_AnimateRobes(k);
    //   if (sprite_wallcoll[k])
    //     Blind_FireballFlurry(k, sprite_wallcoll[k]);
    // }
    pub(super) fn blind_decelerate_x(&mut self, k: usize) {
        if self.ram[SPRITE_X_VEL + k] != 0 {
            let delta: i8 = if (self.ram[SPRITE_X_VEL + k] as i8) < 0 {
                2
            } else {
                -2
            };
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(delta as u8);
        }
        self.blind_animate_robes(k);
        if self.ram[SPRITE_WALLCOLL + k] != 0 {
            let wc = self.ram[SPRITE_WALLCOLL + k];
            self.blind_fireball_flurry(k, wc);
        }
    }

    // void Blind_Decelerate_Y(int k) {  // 9da6a4
    //   if (sprite_y_vel[k] != 0)
    //     sprite_y_vel[k] += sign8(sprite_y_vel[k]) ? 4 : -4;
    //   Sprite_MoveY(k);
    //   if (sprite_wallcoll[k])
    //     Blind_FireballFlurry(k, sprite_wallcoll[k]);
    // }
    pub(super) fn blind_decelerate_y(&mut self, k: usize) {
        if self.ram[SPRITE_Y_VEL + k] != 0 {
            let delta: i8 = if (self.ram[SPRITE_Y_VEL + k] as i8) < 0 {
                4
            } else {
                -4
            };
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_add(delta as u8);
        }
        self.sprite_move_y(k);
        if self.ram[SPRITE_WALLCOLL + k] != 0 {
            let wc = self.ram[SPRITE_WALLCOLL + k];
            self.blind_fireball_flurry(k, wc);
        }
    }

    // void Blind_CheckBumpDamage(int k) {  // 9da6c0
    //   if (!(sprite_delay_aux4[k] | sprite_F[k]))
    //     Sprite_CheckDamageToAndFromLink(k);
    //   if ((uint16)(link_x_coord - cur_sprite_x + 14) < 28 &&
    //       (uint16)(link_y_coord - cur_sprite_y) < 28 &&
    //       !(countdown_for_blink | link_disable_sprite_damage)) {
    //     link_auxiliary_state = 1;
    //     link_give_damage = 8;
    //     link_incapacitated_timer = 16;
    //     link_actual_vel_x ^= 255;
    //     link_actual_vel_y ^= 255;
    //   }
    // }
    pub(super) fn blind_check_bump_damage(&mut self, k: usize) {
        if (self.ram[SPRITE_DELAY_AUX4 + k] | self.ram[SPRITE_F + k]) == 0 {
            self.sprite_check_damage_to_and_from_link_for_blind(k);
        }
        let link_x = self.player_state_view().x();
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let link_y = self.player_state_view().y();
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let dx = link_x.wrapping_sub(cur_x).wrapping_add(14);
        let dy = link_y.wrapping_sub(cur_y);
        let blink_or_disable = self.ram[COUNTDOWN_FOR_BLINK] | self.ram[LINK_DISABLE_SPRITE_DAMAGE];
        if dx < 28 && dy < 28 && blink_or_disable == 0 {
            self.ram[LINK_AUXILIARY_STATE] = 1;
            self.ram[LINK_GIVE_DAMAGE] = 8;
            self.ram[LINK_INCAPACITATED_TIMER] = 16;
            self.ram[LINK_ACTUAL_VEL_X] ^= 255;
            self.ram[LINK_ACTUAL_VEL_Y] ^= 255;
        }
    }

    // void Blind_Animate(int k) {  // 9da6ef
    //   static const uint8 kBlind_HeadDir[17] = {0, 1, 2, 3, 4, 3, 2, 1, 0, 15, 14, 13, 12, 13, 14, 15, 0};
    //   static const uint8 kBlind_Animate_Tab[8] = {0, 1, 1, 2, 2, 3, 3, 4};
    //
    //   if (!sprite_wallcoll[k]) {
    //     int t1 = kBlind_Animate_Tab[BYTE(link_x_coord) >> 5];
    //     t1 = (sprite_D[k] == 3) ? -t1 : t1;
    //     int t0 = (sprite_D[k] - 2) * 8;
    //     int idx = (BLIND_HEAD_ANIM_COUNTER >> 3 & 7) + (BLIND_HEAD_ANIM_COUNTER >> 2 & 1) + t0;
    //     sprite_head_dir[k] = (kBlind_HeadDir[idx] + t1) & 15;
    //   }
    //   Blind_AnimateRobes(k);
    // }
    pub(super) fn blind_animate(&mut self, k: usize) {
        const K_BLIND_HEAD_DIR: [u8; 17] =
            [0, 1, 2, 3, 4, 3, 2, 1, 0, 15, 14, 13, 12, 13, 14, 15, 0];
        const K_BLIND_ANIMATE_TAB: [u8; 8] = [0, 1, 1, 2, 2, 3, 3, 4];
        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            let lx = self.ram[LINK_X_COORD];
            let t1_raw = K_BLIND_ANIMATE_TAB[(lx >> 5) as usize] as i32;
            let t1 = if self.ram[SPRITE_D + k] == 3 {
                -t1_raw
            } else {
                t1_raw
            };
            let t0 = (self.ram[SPRITE_D + k] as i32 - 2) * 8;
            let b = self.ram[BLIND_HEAD_ANIM_COUNTER] as i32;
            let idx = ((b >> 3) & 7) + ((b >> 2) & 1) + t0;
            // C reads kBlind_HeadDir[idx]; idx can be 0..16 (17 entries).
            let head_dir = K_BLIND_HEAD_DIR[(idx as usize) & 0xff] as i32;
            self.ram[SPRITE_HEAD_DIR + k] = ((head_dir + t1) & 15) as u8;
        }
        self.blind_animate_robes(k);
    }

    // void Blind_SpawnLaser(int k) {  // 9da765
    //   static const int8 kBlind_Laser_Xvel[16] = {-8,-8,-8,-4,0,4,8,8,8,8,8,4,0,-4,-8,-8};
    //   static const int8 kBlind_Laser_Yvel[16] = {0,0,4,8,8,8,4,0,0,0,-4,-8,-8,-8,-4,0};
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0xce, &info), i;
    //   if (j >= 0) {
    //     sound_effect_2 = Sprite_CalculateSfxPan(k) | 0x26;
    //     Sprite_SetSpawnedCoordinates(j, &info);
    //     sprite_x_lo[j] = info.r0_x + 4;
    //     sprite_head_dir[j] = i = sprite_head_dir[k];
    //     sprite_x_vel[j] = kBlind_Laser_Xvel[i];
    //     sprite_y_vel[j] = kBlind_Laser_Yvel[i];
    //     sprite_A[j] = 128;
    //     sprite_ignore_projectile[j] = 128;
    //     sprite_flags2[j] = 0x40;
    //     sprite_flags4[j] = 0x14;
    //   }
    // }
    pub(super) fn blind_spawn_laser(&mut self, k: usize) {
        const K_BLIND_LASER_XVEL: [i8; 16] =
            [-8, -8, -8, -4, 0, 4, 8, 8, 8, 8, 8, 4, 0, -4, -8, -8];
        const K_BLIND_LASER_YVEL: [i8; 16] = [0, 0, 4, 8, 8, 8, 4, 0, 0, 0, -4, -8, -8, -8, -4, 0];
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_blind(k, 0xce) {
            self.ram[SOUND_EFFECT_2] = self.sprite_calculate_sfx_pan(k) | 0x26;
            self.sprite_set_spawned_coordinates_for_blind(j, r0_x, r2_y);
            self.ram[SPRITE_X_LO + j] = r0_x.wrapping_add(4) as u8;
            let i = self.ram[SPRITE_HEAD_DIR + k];
            self.ram[SPRITE_HEAD_DIR + j] = i;
            let i_idx = i as usize;
            self.ram[SPRITE_X_VEL + j] = K_BLIND_LASER_XVEL[i_idx] as u8;
            self.ram[SPRITE_Y_VEL + j] = K_BLIND_LASER_YVEL[i_idx] as u8;
            self.ram[SPRITE_A + j] = 128;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 128;
            self.ram[SPRITE_FLAGS2 + j] = 0x40;
            self.ram[SPRITE_FLAGS4 + j] = 0x14;
        }
    }

    // void Blind_Draw(int k) {  // 9dac6c
    //   // Selects either kBlindPoof_Dmd (sprite_graphics >= 15) or kBlind_Dmd
    //   // (otherwise), draws it, then patches head-OAM unless wall-collision
    //   // suppression / certain sprite_C states apply.
    // }
    pub(super) fn blind_draw(&mut self, k: usize) {
        if self.ram[SPRITE_GRAPHICS + k] >= 15 {
            const K_OFFS: [u8; 8] = [0, 1, 5, 13, 23, 30, 35, 37];
            let j = (self.ram[SPRITE_GRAPHICS + k] - 15) as usize;
            let start = K_OFFS[j] as usize;
            let count = (K_OFFS[j + 1] - K_OFFS[j]) as usize;
            self.sprite_draw_multiple_for_blind(k, &K_BLIND_POOF_DMD[start..start + count]);
            return;
        }
        let gfx = self.ram[SPRITE_GRAPHICS + k] as usize;
        self.sprite_draw_multiple_for_blind(k, &K_BLIND_DMD[gfx * 7..gfx * 7 + 7]);

        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            if self.ram[SPRITE_C + k] == 6 {
                // oam[6].y = 0xf0;
                self.blind_draw_patch_oam_y_for_blind(k, 6, 0xf0);
                return;
            }
            if self.ram[SPRITE_C + k] == 4 {
                return;
            }
        }
        if self.ram[SPRITE_GRAPHICS + k] >= 10 {
            return;
        }
        const K_BLIND_OAM_IDX: [u8; 10] = [4, 4, 4, 5, 5, 0, 0, 0, 0, 0];
        let oam_off = K_BLIND_OAM_IDX[self.ram[SPRITE_GRAPHICS + k] as usize];
        let j = self.ram[SPRITE_HEAD_DIR + k] as usize;
        self.blind_draw_patch_oam_head_for_blind(
            k,
            oam_off,
            K_BLIND_HEAD_DRAW_CHAR[j & 15],
            K_BLIND_HEAD_DRAW_FLAGS[j & 15],
        );
    }

    // -----------------------------------------------------------------
    // Local helper adapters (named with `_for_blind` suffix to avoid colliding
    // with canonical Sprite_* helpers). Prefer the canonical helper whenever it
    // is available, keeping only the draw/presentation shims local.
    // -----------------------------------------------------------------

    fn sprite_draw_single_large_for_blind(&mut self, k: usize) {
        // Rewired to canonical Sprite_DrawSingleLarge port.
        self.sprite_draw_single_large(k);
    }

    fn blind_head_apply_oam_for_blind(&mut self, k: usize) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = (self.ram[SPRITE_HEAD_DIR + k] & 15) as usize;
        self.ram[oam + 2] = K_BLIND_HEAD_DRAW_CHAR[j];
        self.ram[oam + 3] = (self.ram[oam + 3] & 0x3f) | K_BLIND_HEAD_DRAW_FLAGS[j];
    }

    fn sprite_return_if_inactive_for_blind(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_ReturnIfInactive port.
        self.sprite_return_if_inactive(k)
    }

    fn sprite_return_if_recoiling_for_blind(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_ReturnIfRecoiling port.
        self.sprite_return_if_recoiling(k)
    }

    fn sprite_check_damage_to_and_from_link_for_blind(&mut self, k: usize) {
        // Rewired to canonical Sprite_CheckDamageToAndFromLink port.
        self.sprite_check_damage_to_and_from_link(k);
    }

    fn sprite_spawn_dynamically_for_blind(
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

    fn sprite_set_spawned_coordinates_for_blind(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        // Rewired to canonical Sprite_SetSpawnedCoordinates port.
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    fn sprite_kill_friends_for_blind(&mut self) {
        // Rewired to canonical Sprite_KillFriends port.
        self.sprite_kill_friends();
    }

    fn sprite_show_message_minimal_for_blind(&mut self) {
        self.sprite_show_message_minimal_c();
    }

    fn spawn_boss_poof_for_blind(&mut self, k: usize) {
        let _ = self.spawn_boss_poof(k);
    }

    fn sprite_spawn_probe_always_for_blind(&mut self, k: usize, dir: u8) {
        self.sprite_spawn_probe_always(k, dir);
    }

    fn sprite_draw_multiple_for_blind(&mut self, k: usize, src: &[DrawMultipleData]) {
        self.sprite_draw_multiple(k, src, None);
    }

    fn blind_draw_patch_oam_y_for_blind(&mut self, _k: usize, oam_idx: usize, y: u8) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize + oam_idx * 4;
        self.ram[oam + 1] = y;
    }

    fn blind_draw_patch_oam_head_for_blind(
        &mut self,
        _k: usize,
        oam_idx: u8,
        charnum: u8,
        flags: u8,
    ) {
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize + oam_idx as usize * 4;
        self.ram[oam + 2] = charnum;
        self.ram[oam + 3] = (self.ram[oam + 3] & 0x3f) | flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    #[test]
    fn prep_blind_sets_battle_fields_when_unlocked() {
        let mut s = fresh_state();
        s.ram[FOLLOWER_INDICATOR] = 0;
        write_le_u16(&mut s.ram, DUNG_SAVEGAME_STATE_BITS, 0x2000);
        // Mark our slot active and clear other slots.
        s.sprite_prep_blind_prepare_battle(3);
        assert_eq!(s.ram[SPRITE_DELAY_AUX2 + 3], 96);
        assert_eq!(s.ram[SPRITE_C + 3], 1);
        assert_eq!(s.ram[SPRITE_D + 3], 2);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + 3], 4);
        assert_eq!(s.ram[SPRITE_GRAPHICS + 3], 7);
        assert_eq!(s.ram[BLIND_HEAD_ANIM_COUNTER], 0);
    }

    #[test]
    fn prep_blind_kills_sprite_when_locked() {
        let mut s = fresh_state();
        s.ram[FOLLOWER_INDICATOR] = 6; // wrong indicator -> branch to else
        s.ram[SPRITE_STATE + 5] = 9;
        s.sprite_prep_blind_prepare_battle(5);
        assert_eq!(s.ram[SPRITE_STATE + 5], 0);
    }

    #[test]
    fn blind_spit_fireball_returns_minus_one_when_subtype2_masks() {
        let mut s = fresh_state();
        s.ram[SPRITE_SUBTYPE2 + 2] = 0xff;
        let r = s.blind_spit_fireball(2, 0x1f);
        assert_eq!(r, -1);
    }

    #[test]
    fn blind_spit_fireball_writes_velocity_table() {
        let mut s = fresh_state();
        // Zero all sprite states so allocation can succeed (slot 13 picked).
        s.ram[SPRITE_SUBTYPE2 + 0] = 0;
        s.ram[SPRITE_HEAD_DIR + 0] = 8; // xvel=32, yvel=0
        let r = s.blind_spit_fireball(0, 0xf);
        assert!(r >= 0, "expected fireball spawn slot, got {r}");
        let j = r as usize;
        assert_eq!(s.ram[SPRITE_X_VEL + j], 32);
        assert_eq!(s.ram[SPRITE_Y_VEL + j], 0);
        assert_eq!(s.ram[SPRITE_DEFL_BITS + j] & 8, 8);
        assert_eq!(s.ram[SPRITE_BUMP_DAMAGE + j], 4);
    }

    #[test]
    fn blind_decelerate_x_brings_velocity_toward_zero() {
        let mut s = fresh_state();
        // Negative velocity -> add +2.
        s.ram[SPRITE_X_VEL + 4] = (-5i8) as u8;
        s.ram[SPRITE_WALLCOLL + 4] = 0; // suppress flurry branch
        s.blind_decelerate_x(4);
        assert_eq!(s.ram[SPRITE_X_VEL + 4] as i8, -3);

        // Positive velocity -> subtract 2.
        s.ram[SPRITE_X_VEL + 4] = 7;
        s.blind_decelerate_x(4);
        assert_eq!(s.ram[SPRITE_X_VEL + 4], 5);

        // Zero velocity stays zero.
        s.ram[SPRITE_X_VEL + 4] = 0;
        s.blind_decelerate_x(4);
        assert_eq!(s.ram[SPRITE_X_VEL + 4], 0);
    }

    #[test]
    fn blind_animate_picks_head_dir_from_table() {
        let mut s = fresh_state();
        s.ram[SPRITE_WALLCOLL + 1] = 0;
        s.ram[SPRITE_D + 1] = 2; // t0 = 0, no negation
        s.ram[LINK_X_COORD] = 0; // tab idx 0 -> t1 = 0
        s.ram[BLIND_HEAD_ANIM_COUNTER] = 0; // idx 0 -> table[0] = 0
        s.blind_animate(1);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + 1], 0);

        // BLIND_HEAD_ANIM_COUNTER=8 -> (8>>3 & 7)=1, (8>>2 & 1)=0, idx=1 -> table[1] = 1
        s.ram[BLIND_HEAD_ANIM_COUNTER] = 8;
        s.blind_animate(1);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + 1], 1);
    }
}
