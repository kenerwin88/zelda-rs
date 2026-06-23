//! Ported Blind-boss handlers from sprite_main.c.
use super::sprite::{DrawMultipleData, PrepOamCoordsRet};
use super::*;

// Tables shared by Blind head drawing (from sprite_main.c lines 400-401).
const BLIND_HEAD_DRAW_CHARS: [u8; 16] = [
    0x86, 0x86, 0x84, 0x82, 0x80, 0x82, 0x84, 0x86, 0x86, 0x86, 0x88, 0x8a, 0x8c, 0x8a, 0x88, 0x86,
];
const BLIND_HEAD_DRAW_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0, 0, 0, 0,
];

// kBlindPoof_Dmd from sprite_main.c:15819.
const BLIND_POOF_DRAW_FRAMES: [DrawMultipleData; 37] = [
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
const BLIND_DRAW_FRAMES: [DrawMultipleData; 105] = [
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
        let sprite = self.sprite_slot_view(k);
        if (sprite.a() as i8).is_negative() {
            self.sprite_blind_laser(k);
        } else if sprite.a() == 2 {
            self.sprite_blind_head(k);
        } else {
            self.sprite_blind_blind_blind(k);
        }
    }

    // void Sprite_BlindLaser(int k) {  // 9da268
    pub(super) fn sprite_blind_laser(&mut self, k: usize) {
        const LOCAL_GRAPHICS: [u8; 16] = [7, 7, 8, 9, 10, 9, 8, 7, 7, 7, 8, 9, 10, 9, 8, 7];
        const OAM_FLAGS: [u8; 16] = [
            0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80,
        ];
        let j = (self.sprite_slot_view(k).head_direction() & 15) as usize;
        {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_graphics(LOCAL_GRAPHICS[j]);
            sprite.set_oam_flags(OAM_FLAGS[j] | 3);
        }
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
        if self.sprite_slot_view(k).delay_main() != 0 {
            if self.sprite_slot_view(k).delay_main() == 1 {
                self.sprite_slot_view_mut(k).clear();
            }
            return;
        }
        self.sprite_check_damage_to_link_same_layer(k);
        let x = self
            .sprite_get_x(k)
            .wrapping_add_signed(i16::from(self.sprite_slot_view(k).x_velocity() as i8));
        let y = self
            .sprite_get_y(k)
            .wrapping_add_signed(i16::from(self.sprite_slot_view(k).y_velocity() as i8));
        self.sprite_set_x(k, x);
        self.sprite_set_y(k, y);
        if self.sprite_check_tile_collision(k) != 0 {
            self.sprite_slot_view_mut(k).set_delay_main(12);
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
        let dung_state = self.game_state.dungeon.savegame_state.savegame_state_bits();
        if self.game_state.sprites.follower_runtime.indicator() != 6 && (dung_state & 0x2000) != 0 {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_delay_aux2(96);
            sprite.set_c(1);
            sprite.set_direction(2);
            sprite.set_head_direction(4);
            sprite.set_graphics(7);
            self.sprite_system_mut().set_blind_head_anim_counter(0);
        } else {
            self.sprite_slot_view_mut(k).clear();
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
        const BLIND_HEAD_X_POSITION_LIMITS: [u8; 2] = [0x98, 0x58];
        const BLIND_HEAD_Y_POSITION_LIMITS: [u8; 2] = [0xb0, 0x50];
        const BLIND_HEAD_Y_VELOCITY_LIMITS: [i8; 2] = [24, -24];
        const BLIND_HEAD_X_VELOCITY_LIMITS: [i8; 2] = [32, -32];

        self.sprite_slot_view_mut(k).or_object_priority_bits(48);
        self.sprite_draw_single_large_for_blind(k);
        // OamEnt *oam = GetOamCurPtr(); oam->charnum = ...; oam->flags = ...;
        self.blind_head_apply_oam_for_blind(k);

        if self.sprite_return_if_inactive_for_blind(k) {
            return;
        }
        if self.sprite_slot_view(k).f() == 14 {
            self.sprite_slot_view_mut(k).set_f(8);
        }
        if self.sprite_return_if_recoiling_for_blind(k) {
            return;
        }
        let new_sub = self.sprite_slot_view(k).subtype().wrapping_sub(1);
        self.sprite_slot_view_mut(k).set_subtype(new_sub);
        if (new_sub as i8) < 0 {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_subtype(2);
            sprite.increment_head_direction_mod16();
        }
        if self.sprite_slot_view(k).delay_main() != 0 {
            return;
        }
        self.sprite_check_damage_to_and_from_link_for_blind(k);
        self.sprite_slot_view_mut(k).increment_subtype2();
        let j_ret = self.blind_spit_fireball(k, 0x1f);
        if j_ret >= 0 {
            let zsub = self.sprite_slot_view_mut(k).decrement_z_subpixel();
            if (zsub as i8) < 0 {
                self.sprite_slot_view_mut(k).set_z_subpixel(4);
                let pt = self.sprite_project_speed_towards_link(k, 32);
                let j = j_ret as usize;
                let mut fireball = self.sprite_slot_view_mut(j);
                fireball.set_x_velocity(pt.x);
                fireball.set_y_velocity(pt.y);
            }
        }
        let mut j = (self.sprite_slot_view(k).g() & 1) as usize;
        if self.sprite_slot_view(k).x_velocity() != BLIND_HEAD_X_VELOCITY_LIMITS[j] as u8 {
            let delta: i8 = if j != 0 { -1 } else { 1 };
            self.sprite_slot_view_mut(k).add_x_velocity(delta as u8);
        }
        if (self.sprite_slot_view(k).x_low() & !1) == BLIND_HEAD_X_POSITION_LIMITS[j] {
            self.sprite_slot_view_mut(k).increment_g();
        }
        j = (self.sprite_slot_view(k).anim_clock() & 1) as usize;
        if self.sprite_slot_view(k).y_velocity() != BLIND_HEAD_Y_VELOCITY_LIMITS[j] as u8 {
            let delta: i8 = if j != 0 { -1 } else { 1 };
            self.sprite_slot_view_mut(k).add_y_velocity(delta as u8);
        }
        if (self.sprite_slot_view(k).y_low() & !1) == BLIND_HEAD_Y_POSITION_LIMITS[j] {
            self.sprite_slot_view_mut(k).increment_anim_clock();
        }
        if self.sprite_slot_view(k).f() == 0 {
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
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_flags3(0x5b);
            sprite.set_oam_flags(0x5b & 15);
            sprite.set_deflection_bits(4);
            sprite.set_a(2);
            sprite.set_flags2(1);
            sprite.set_flags4(0);
            sprite.set_flags(0);
            sprite.set_z(23);
            sprite.set_y_low(23u16.wrapping_add(r2_y) as u8);
            sprite.set_g(((r0_x >> 7) & 1) as u8);
            sprite.set_anim_clock(((r2_y >> 7) & 1) as u8);
            sprite.set_delay_main(48);
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
        self.sprite_slot_view_mut(k).or_object_priority_bits(0x30);
        self.blind_draw(k);
        self.sprite_slot_view_mut(k).set_oam_flags(1);
        if self.sprite_return_if_inactive_for_blind(k) {
            return;
        }
        let a = self.sprite_slot_view(k).f();
        if a != 0 {
            self.sprite_slot_view_mut(k).decrement_f();
        }

        // if (a == 11) { ... }
        if a == 11 {
            {
                let mut sprite = self.sprite_slot_view_mut(k);
                sprite.set_hit_timer(0);
                sprite.set_wall_collision(0);
            }
            if self.sprite_slot_view(k).delay_aux4() == 0 {
                let new_zsub = {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_health(128);
                    sprite.set_delay_aux4(48);
                    sprite.and_oam_flags(1);
                    sprite.increment_z_subpixel()
                };
                if new_zsub < 3 {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_wall_collision(96);
                    sprite.set_subtype(1);
                } else {
                    self.sprite_slot_view_mut(k).set_z_subpixel(0);
                    let new_limit = self
                        .game_state
                        .sprites
                        .system
                        .limit_instance()
                        .wrapping_add(1);
                    self.sprite_system_mut().set_limit_instance(new_limit);
                    if new_limit == 3 {
                        self.sprite_kill_friends_for_blind();
                        {
                            let mut sprite = self.sprite_slot_view_mut(k);
                            sprite.set_state(4);
                            sprite.set_a(0);
                            sprite.set_delay_main(255);
                            sprite.set_hit_timer(255);
                        }
                        self.follower_link_state_mut().increment_menu_block_flag();
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x22);
                        return;
                    }
                    {
                        let mut sprite = self.sprite_slot_view_mut(k);
                        sprite.set_y_velocity(0);
                        sprite.set_x_velocity(0);
                        sprite.set_c(6);
                        sprite.set_delay_aux2(255);
                        sprite.set_ignore_projectile(255);
                    }
                    self.blind_spawn_head(k);
                }
            }
        }

        // if (sprite_A[k]) { ... return; }
        if self.sprite_slot_view(k).a() != 0 {
            const BLIND_DEFEAT_GRAPHICS_SEQUENCE: [u8; 7] = [20, 19, 18, 17, 16, 15, 15];
            if self.sprite_slot_view(k).delay_main() == 0 {
                self.sprite_slot_view_mut(k).clear();
            }
            let idx = (self.sprite_slot_view(k).delay_main() >> 3) as usize;
            self.sprite_slot_view_mut(k)
                .set_graphics(BLIND_DEFEAT_GRAPHICS_SEQUENCE[idx.min(6)]);
            return;
        }
        // if (!(++sprite_subtype2[k] & 1)) sprite_delay_main[k]++;
        self.sprite_slot_view_mut(k).increment_subtype2();
        let new_sub2 = self.sprite_slot_view(k).subtype2();
        if (new_sub2 & 1) == 0 {
            self.sprite_slot_view_mut(k).increment_delay_main();
        }

        // if (sprite_delay_aux1[k]) { ... return; }
        if self.sprite_slot_view(k).delay_aux1() != 0 {
            self.sprite_slot_view_mut(k).set_ai_state(0);
            if self.sprite_slot_view(k).delay_aux1() == 8 {
                self.blind_spawn_laser(k);
            }
            self.blind_check_bump_damage(k);
            return;
        }
        // BLIND_HEAD_ANIM_COUNTER++;
        self.sprite_system_mut().increment_blind_head_anim_counter();
        // stunned/ai_state branch
        if self.sprite_slot_view(k).stunned() == 0 {
            if self.sprite_slot_view(k).ai_state() != 0 {
                let mut sprite = self.sprite_slot_view_mut(k);
                sprite.set_delay_aux1(16);
                sprite.set_stunned(128);
                sprite.set_ai_state(0);
            }
        } else {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.decrement_stunned();
            sprite.set_ai_state(0);
        }
        // sprite_x_hi[k] = HIBYTE(link_x_coord); sprite_y_hi[k] = HIBYTE(link_y_coord);
        let link_x_high = self.game_state.player.follower_link.x_high();
        let link_y_high = self.game_state.player.follower_link.y_high();
        {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_x_high(link_x_high);
            sprite.set_y_high(link_y_high);
        }

        match self.sprite_slot_view(k).c() {
            0 => {
                // blinded
                self.set_sprite_dma_head_pointer(0);
                self.set_sprite_dma_body_pointer(0xA0);
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.increment_c();
                    sprite.set_delay_aux2(96);
                } else if self.sprite_slot_view(k).delay_aux2() == 80 {
                    self.dialogue_message_index_mut().set_value(0x123);
                    self.sprite_show_message_minimal_for_blind();
                } else if self.sprite_slot_view(k).delay_aux2() == 24 {
                    self.spawn_boss_poof_for_blind(k);
                }
            }
            1 => {
                // retreat to back wall
                self.blind_check_bump_damage(k);
                self.sprite_slot_view_mut(k).set_graphics(9);
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.increment_c();
                    sprite.set_delay_main(255);
                    sprite.set_ignore_projectile(0);
                } else if self.sprite_slot_view(k).delay_aux2() < 64 {
                    self.sprite_slot_view_mut(k).set_y_velocity((-8i8) as u8);
                    self.sprite_move_y(k);
                }
                self.blind_animate(k);
                self.sprite_slot_view_mut(k).set_head_direction(4);
            }
            2 => {
                // oscillate
                const BLIND_OSCILLATION_Y_VELOCITY_TARGETS: [i8; 2] = [18, -18];
                const BLIND_OSCILLATION_X_VELOCITY_TARGETS: [i8; 2] = [24, -24];
                const BLIND_OSCILLATION_X_POSITION_TARGETS: [u8; 2] = [164, 76];
                self.blind_check_bump_damage(k);
                self.blind_animate(k);
                let sub2 = self.sprite_slot_view(k).subtype2();
                let below_a = self.sprite_is_below_link(k).a;
                let sprite = self.sprite_slot_view(k);
                let cond1 = (sub2 & 127) == 0 && below_a.wrapping_add(2) != sprite.direction();
                let cond2 = sprite.delay_main() == 0;
                if (cond1 || cond2) && sprite.x_low() < 0x78 {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.increment_c();
                    sprite.and_y_velocity(!1);
                    sprite.and_x_velocity(!1);
                    sprite.set_delay_aux2(0x30);
                    return;
                }
                let mut j = (self.sprite_slot_view(k).b() & 1) as usize;
                let delta: i8 = if j != 0 { -1 } else { 1 };
                self.sprite_slot_view_mut(k).add_y_velocity(delta as u8);
                if self.sprite_slot_view(k).y_velocity()
                    == BLIND_OSCILLATION_Y_VELOCITY_TARGETS[j] as u8
                {
                    self.sprite_slot_view_mut(k).increment_b();
                }
                j = (self.sprite_slot_view(k).g() & 1) as usize;
                if self.sprite_slot_view(k).x_velocity()
                    != BLIND_OSCILLATION_X_VELOCITY_TARGETS[j] as u8
                {
                    let delta: i8 = if j != 0 { -1 } else { 1 };
                    self.sprite_slot_view_mut(k).add_x_velocity(delta as u8);
                }
                if (self.sprite_slot_view(k).x_low() & !1)
                    == BLIND_OSCILLATION_X_POSITION_TARGETS[j]
                {
                    self.sprite_slot_view_mut(k).increment_g();
                }
                self.sprite_move_xy(k);
                if self.sprite_slot_view(k).wall_collision() != 0 {
                    let wc = self.sprite_slot_view(k).wall_collision();
                    self.blind_fireball_flurry(k, wc);
                } else if (self.sprite_slot_view(k).subtype2() & 7) == 0 {
                    let hd = self.sprite_slot_view(k).head_direction() << 2;
                    self.sprite_spawn_probe_always_for_blind(k, hd);
                }
            }
            3 => {
                // switch walls
                self.blind_check_bump_damage(k);
                if self.sprite_slot_view(k).delay_aux2() != 0 {
                    self.blind_decelerate_x(k);
                    self.sprite_move_x(k);
                    self.blind_decelerate_y(k);
                } else {
                    const BLIND_SWITCH_WALL_Y_VELOCITY_TARGETS: [i8; 2] = [64, -64];
                    const BLIND_SWITCH_WALL_Y_POSITION_TARGETS: [u8; 2] = [0x90, 0x50];
                    let j = (self.sprite_slot_view(k).direction().wrapping_sub(2)) as usize;
                    if self.sprite_slot_view(k).y_velocity()
                        != BLIND_SWITCH_WALL_Y_VELOCITY_TARGETS[j] as u8
                    {
                        let delta: i8 = if j != 0 { -2 } else { 2 };
                        self.sprite_slot_view_mut(k).add_y_velocity(delta as u8);
                    }
                    if (self.sprite_slot_view(k).y_low() & !3)
                        == BLIND_SWITCH_WALL_Y_POSITION_TARGETS[j]
                    {
                        let b = self.sprite_slot_view(k).direction().wrapping_sub(1);
                        let mut sprite = self.sprite_slot_view_mut(k);
                        sprite.increment_c();
                        sprite.set_b(b);
                    }
                    self.sprite_move_xy(k);
                    self.blind_decelerate_x(k);
                }
            }
            4 => {
                // whirl around
                self.blind_check_bump_damage(k);
                if (self.sprite_slot_view(k).subtype2() & 7) == 0 {
                    const BLIND_WHIRL_AROUND_GRAPHICS_TARGETS: [u8; 2] = [0, 9];
                    let j = (self.sprite_slot_view(k).direction().wrapping_sub(2)) as usize;
                    if self.sprite_slot_view(k).graphics() == BLIND_WHIRL_AROUND_GRAPHICS_TARGETS[j]
                    {
                        let g = self.sprite_slot_view(k).x_low() >> 7;
                        let direction = self.sprite_slot_view(k).direction() ^ 1;
                        let mut sprite = self.sprite_slot_view_mut(k);
                        sprite.set_delay_main(254);
                        sprite.set_c(2);
                        sprite.set_direction(direction);
                        sprite.set_g(g);
                    } else {
                        let delta: i8 = if j != 0 { 1 } else { -1 };
                        let graphics = self
                            .sprite_slot_view(k)
                            .graphics()
                            .wrapping_add(delta as u8);
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
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
                {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_hit_timer(0);
                    sprite.set_head_direction(12);
                }
                let aux2 = self.sprite_slot_view(k).delay_aux2();
                if aux2 == 0 {
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.increment_c();
                    sprite.set_delay_aux2(39);
                    self.sprite_sfx_queue_sfx1_with_pan(k, 0x13);
                } else if aux2 >= 224 {
                    const BLIND_BEHIND_CURTAIN_GRAPHICS: [u8; 4] = [14, 13, 12, 10];
                    self.sprite_slot_view_mut(k)
                        .set_graphics(BLIND_BEHIND_CURTAIN_GRAPHICS[((aux2 - 224) >> 3) as usize]);
                } else {
                    self.sprite_slot_view_mut(k).set_graphics(14);
                }
            }
            7 => {
                // rerobe
                if self.sprite_slot_view(k).delay_aux2() == 0 {
                    let direction = (self.sprite_slot_view(k).y_low() >> 7).wrapping_add(2);
                    let g = (self.sprite_slot_view(k).x_low() << 2)
                        | (self.sprite_slot_view(k).x_low() >> 7);
                    let mut sprite = self.sprite_slot_view_mut(k);
                    sprite.set_c(2);
                    sprite.set_delay_main(128);
                    sprite.set_direction(direction);
                    sprite.set_g(g);
                    sprite.set_x_velocity(0);
                    sprite.set_y_velocity(0);
                    sprite.set_ignore_projectile(0);
                } else {
                    const BLIND_REROBE_GRAPHICS: [u8; 5] = [10, 11, 12, 13, 14];
                    let idx = (self.sprite_slot_view(k).delay_aux2() >> 3) as usize;
                    self.sprite_slot_view_mut(k)
                        .set_graphics(BLIND_REROBE_GRAPHICS[idx.min(4)]);
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
        const BLIND_ROBE_ANIMATION_GRAPHICS: [u8; 8] = [7, 8, 9, 8, 0, 1, 2, 1];
        let sprite = self.sprite_slot_view(k);
        let s2 = (sprite.subtype2() >> 3) & 3;
        let d_minus_2 = (sprite.direction() as i8).wrapping_sub(2);
        let idx = (s2 as i32) + ((d_minus_2 as i32) << 2);
        self.sprite_slot_view_mut(k)
            .set_graphics(BLIND_ROBE_ANIMATION_GRAPHICS[(idx as usize) & 7]);
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
        let new_e = {
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.decrement_wall_collision();
            sprite.set_oam_flags((a & 7).wrapping_mul(2).wrapping_add(1));
            sprite.decrement_e()
        };
        if (new_e as i8) < 0 {
            let subtype = self.sprite_slot_view(k).subtype();
            let mut sprite = self.sprite_slot_view_mut(k);
            sprite.set_e(subtype);
            sprite.increment_head_direction_mod16();
        }
        let sprite = self.sprite_slot_view(k);
        if (sprite.subtype2() & 31) == 0 && sprite.subtype() != 5 {
            self.sprite_slot_view_mut(k).increment_subtype();
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
        const BLIND_FIREBALL_X_VELOCITIES_BY_HEAD_DIR: [i8; 16] = [
            -32, -28, -24, -16, 0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28,
        ];
        const BLIND_FIREBALL_Y_VELOCITIES_BY_HEAD_DIR: [i8; 16] = [
            0, 16, 24, 28, 32, 28, 24, 16, 0, -16, -24, -28, -32, -28, -24, -16,
        ];
        if (self.sprite_slot_view(k).subtype2() & a) != 0 {
            return -1;
        }
        let j = self.sprite_spawn_fireball(k);
        match j {
            j if j >= 0 => {
                let j = j as usize;
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
                let i = self.sprite_slot_view(k).head_direction() as usize;
                let mut fireball = self.sprite_slot_view_mut(j);
                fireball.set_x_velocity(BLIND_FIREBALL_X_VELOCITIES_BY_HEAD_DIR[i] as u8);
                fireball.set_y_velocity(BLIND_FIREBALL_Y_VELOCITIES_BY_HEAD_DIR[i] as u8);
                fireball.or_deflection_bits(8);
                fireball.set_bump_damage(4);
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
        if self.sprite_slot_view(k).x_velocity() != 0 {
            let delta: i8 = if (self.sprite_slot_view(k).x_velocity() as i8) < 0 {
                2
            } else {
                -2
            };
            self.sprite_slot_view_mut(k).add_x_velocity(delta as u8);
        }
        self.blind_animate_robes(k);
        if self.sprite_slot_view(k).wall_collision() != 0 {
            let wc = self.sprite_slot_view(k).wall_collision();
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
        if self.sprite_slot_view(k).y_velocity() != 0 {
            let delta: i8 = if (self.sprite_slot_view(k).y_velocity() as i8) < 0 {
                4
            } else {
                -4
            };
            self.sprite_slot_view_mut(k).add_y_velocity(delta as u8);
        }
        self.sprite_move_y(k);
        if self.sprite_slot_view(k).wall_collision() != 0 {
            let wc = self.sprite_slot_view(k).wall_collision();
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
        let sprite = self.sprite_slot_view(k);
        if (sprite.delay_aux4() | sprite.f()) == 0 {
            self.sprite_check_damage_to_and_from_link_for_blind(k);
        }
        let link_x = self.game_state.player.follower_link.x();
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let link_y = self.game_state.player.follower_link.y();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let dx = link_x.wrapping_sub(cur_x).wrapping_add(14);
        let dy = link_y.wrapping_sub(cur_y);
        let blink_or_disable = self.game_state.player.follower_link.blink_countdown()
            | self
                .game_state
                .player
                .follower_link
                .sprite_damage_disable_timer();
        if dx < 28 && dy < 28 && blink_or_disable == 0 {
            self.follower_link_state_mut().set_given_damage(8);
            self.follower_link_state_mut().set_auxiliary_state(1);
            self.follower_link_state_mut().set_incapacitated_timer(16);
            self.follower_link_state_mut().xor_actual_velocity_xy(255);
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
        const BLIND_HEAD_DIRECTION_BASES: [u8; 17] =
            [0, 1, 2, 3, 4, 3, 2, 1, 0, 15, 14, 13, 12, 13, 14, 15, 0];
        const BLIND_HEAD_FRAME_BY_DISTANCE: [u8; 8] = [0, 1, 1, 2, 2, 3, 3, 4];
        if self.sprite_slot_view(k).wall_collision() == 0 {
            let lx = self.game_state.player.follower_link.x() as u8;
            let t1_raw = BLIND_HEAD_FRAME_BY_DISTANCE[(lx >> 5) as usize] as i32;
            let direction = self.sprite_slot_view(k).direction();
            let t1 = if direction == 3 { -t1_raw } else { t1_raw };
            let t0 = (direction as i32 - 2) * 8;
            let b = self.game_state.sprites.system.blind_head_anim_counter() as i32;
            let idx = ((b >> 3) & 7) + ((b >> 2) & 1) + t0;
            // C reads kBlind_HeadDir[idx]; idx can be 0..16 (17 entries).
            let head_dir = BLIND_HEAD_DIRECTION_BASES[(idx as usize) & 0xff] as i32;
            self.sprite_slot_view_mut(k)
                .set_head_direction(((head_dir + t1) & 15) as u8);
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
        const BLIND_LASER_X_VELOCITIES_BY_HEAD_DIR: [i8; 16] =
            [-8, -8, -8, -4, 0, 4, 8, 8, 8, 8, 8, 4, 0, -4, -8, -8];
        const BLIND_LASER_Y_VELOCITIES_BY_HEAD_DIR: [i8; 16] =
            [0, 0, 4, 8, 8, 8, 4, 0, 0, 0, -4, -8, -8, -8, -4, 0];
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_blind(k, 0xce) {
            let sfx = self.sprite_calculate_sfx_pan(k) | 0x26;
            self.set_sound_effect_2(sfx);
            self.sprite_set_spawned_coordinates_for_blind(j, r0_x, r2_y);
            let i = self.sprite_slot_view(k).head_direction();
            let i_idx = i as usize;
            let mut laser = self.sprite_slot_view_mut(j);
            laser.set_x_low(r0_x.wrapping_add(4) as u8);
            laser.set_head_direction(i);
            laser.set_x_velocity(BLIND_LASER_X_VELOCITIES_BY_HEAD_DIR[i_idx] as u8);
            laser.set_y_velocity(BLIND_LASER_Y_VELOCITIES_BY_HEAD_DIR[i_idx] as u8);
            laser.set_a(128);
            laser.set_ignore_projectile(128);
            laser.set_flags2(0x40);
            laser.set_flags4(0x14);
        }
    }

    // void Blind_Draw(int k) {  // 9dac6c
    //   // Selects either kBlindPoof_Dmd (sprite_graphics >= 15) or kBlind_Dmd
    //   // (otherwise), draws it, then patches head-OAM unless wall-collision
    //   // suppression / certain sprite_C states apply.
    // }
    pub(super) fn blind_draw(&mut self, k: usize) {
        if self.sprite_slot_view(k).graphics() >= 15 {
            const BLIND_POOF_DRAW_FRAME_STARTS: [u8; 8] = [0, 1, 5, 13, 23, 30, 35, 37];
            let j = (self.sprite_slot_view(k).graphics() - 15) as usize;
            let start = BLIND_POOF_DRAW_FRAME_STARTS[j] as usize;
            let count =
                (BLIND_POOF_DRAW_FRAME_STARTS[j + 1] - BLIND_POOF_DRAW_FRAME_STARTS[j]) as usize;
            self.sprite_draw_multiple_for_blind(k, &BLIND_POOF_DRAW_FRAMES[start..start + count]);
            return;
        }
        let gfx = self.sprite_slot_view(k).graphics() as usize;
        self.sprite_draw_multiple_for_blind(k, &BLIND_DRAW_FRAMES[gfx * 7..gfx * 7 + 7]);

        if self.sprite_slot_view(k).wall_collision() == 0 {
            if self.sprite_slot_view(k).c() == 6 {
                // oam[6].y = 0xf0;
                self.blind_draw_patch_oam_y_for_blind(k, 6, 0xf0);
                return;
            }
            if self.sprite_slot_view(k).c() == 4 {
                return;
            }
        }
        if self.sprite_slot_view(k).graphics() >= 10 {
            return;
        }
        const BLIND_HEAD_OAM_OFFSETS_BY_GRAPHICS: [u8; 10] = [4, 4, 4, 5, 5, 0, 0, 0, 0, 0];
        let oam_off =
            BLIND_HEAD_OAM_OFFSETS_BY_GRAPHICS[self.sprite_slot_view(k).graphics() as usize];
        let j = self.sprite_slot_view(k).head_direction() as usize;
        self.blind_draw_patch_oam_head_for_blind(
            k,
            oam_off,
            BLIND_HEAD_DRAW_CHARS[j & 15],
            BLIND_HEAD_DRAW_FLAGS[j & 15],
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
        let oam = self.game_state.oam.current_pointer_usize();
        let j = (self.sprite_slot_view(k).head_direction() & 15) as usize;
        self.oam_state_mut()
            .set_entry_char(oam, BLIND_HEAD_DRAW_CHARS[j]);
        self.oam_state_mut()
            .merge_entry_flags(oam, 0x3f, BLIND_HEAD_DRAW_FLAGS[j]);
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
        let oam = self.game_state.oam.current_pointer_usize() + oam_idx * 4;
        self.oam_state_mut().set_entry_y(oam, y);
    }

    fn blind_draw_patch_oam_head_for_blind(
        &mut self,
        _k: usize,
        oam_idx: u8,
        charnum: u8,
        flags: u8,
    ) {
        let oam = self.game_state.oam.current_pointer_usize() + oam_idx as usize * 4;
        self.oam_state_mut().set_entry_char(oam, charnum);
        self.oam_state_mut().merge_entry_flags(oam, 0x3f, flags);
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
        s.follower_state_mut().set_indicator(0);
        s.dungeon_savegame_state_mut()
            .set_savegame_state_bits(0x2000);
        // Mark our slot active and clear other slots.
        s.sprite_prep_blind_prepare_battle(3);
        let sprite = s.sprite_slot_view(3);
        assert_eq!(sprite.delay_aux2(), 96);
        assert_eq!(sprite.c(), 1);
        assert_eq!(sprite.direction(), 2);
        assert_eq!(sprite.head_direction(), 4);
        assert_eq!(sprite.graphics(), 7);
        assert_eq!(s.ram[BLIND_HEAD_ANIM_COUNTER], 0);
    }

    #[test]
    fn prep_blind_kills_sprite_when_locked() {
        let mut s = fresh_state();
        s.follower_state_mut().set_indicator(6); // wrong indicator -> branch to else
        s.sprite_slot_view_mut(5).set_state(9);
        s.sprite_prep_blind_prepare_battle(5);
        assert_eq!(s.sprite_slot_view(5).state(), 0);
    }

    #[test]
    fn blind_spit_fireball_returns_minus_one_when_subtype2_masks() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(2).set_subtype2(0xff);
        let r = s.blind_spit_fireball(2, 0x1f);
        assert_eq!(r, -1);
    }

    #[test]
    fn blind_spit_fireball_writes_velocity_table() {
        let mut s = fresh_state();
        // Zero all sprite states so allocation can succeed (slot 13 picked).
        {
            let mut sprite = s.sprite_slot_view_mut(0);
            sprite.set_subtype2(0);
            sprite.set_head_direction(8); // xvel=32, yvel=0
        }
        let r = s.blind_spit_fireball(0, 0xf);
        assert!(r >= 0, "expected fireball spawn slot, got {r}");
        let j = r as usize;
        let fireball = s.sprite_slot_view(j);
        assert_eq!(fireball.x_velocity(), 32);
        assert_eq!(fireball.y_velocity(), 0);
        assert_eq!(fireball.deflection_bits() & 8, 8);
        assert_eq!(fireball.bump_damage(), 4);
    }

    #[test]
    fn blind_decelerate_x_brings_velocity_toward_zero() {
        let mut s = fresh_state();
        // Negative velocity -> add +2.
        {
            let mut sprite = s.sprite_slot_view_mut(4);
            sprite.set_x_velocity((-5i8) as u8);
            sprite.set_wall_collision(0); // suppress flurry branch
        }
        s.blind_decelerate_x(4);
        assert_eq!(s.sprite_slot_view(4).x_velocity() as i8, -3);

        // Positive velocity -> subtract 2.
        s.sprite_slot_view_mut(4).set_x_velocity(7);
        s.blind_decelerate_x(4);
        assert_eq!(s.sprite_slot_view(4).x_velocity(), 5);

        // Zero velocity stays zero.
        s.sprite_slot_view_mut(4).set_x_velocity(0);
        s.blind_decelerate_x(4);
        assert_eq!(s.sprite_slot_view(4).x_velocity(), 0);
    }

    #[test]
    fn blind_animate_picks_head_dir_from_table() {
        let mut s = fresh_state();
        {
            let mut sprite = s.sprite_slot_view_mut(1);
            sprite.set_wall_collision(0);
            sprite.set_direction(2); // t0 = 0, no negation
        }
        s.follower_link_state_mut().set_x(0); // tab idx 0 -> t1 = 0
        s.sprite_system_mut().set_blind_head_anim_counter(0); // idx 0 -> table[0] = 0
        s.blind_animate(1);
        assert_eq!(s.sprite_slot_view(1).head_direction(), 0);

        // BLIND_HEAD_ANIM_COUNTER=8 -> (8>>3 & 7)=1, (8>>2 & 1)=0, idx=1 -> table[1] = 1
        s.sprite_system_mut().set_blind_head_anim_counter(8);
        s.blind_animate(1);
        assert_eq!(s.sprite_slot_view(1).head_direction(), 1);
    }
}
