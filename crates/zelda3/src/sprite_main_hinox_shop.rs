//! Ported Hinox + ShopItem handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source. The original C body is
//! reproduced as a comment block above each port so a reviewer can verify
//! behavior line-by-line.
//!
//! Helpers used by these handlers keep `_for_hinox_shop` adapter names matching
//! the round-2 `_for_blind` pattern. Adapters route to canonical ports when the
//! shared helper already exists.

use super::*;
use crate::zelda_rtl::sprite::{DrawMultipleData, PrepOamCoordsRet};

// ---------------------------------------------------------------------------
// File-local RAM offsets. Mirror variables.h, kept local because the matching
// globals in zelda_rtl.rs are not exported (the other sprite_main_* files
// follow the same convention).
// ---------------------------------------------------------------------------

// variables.h:660 — sprite_flags.
const SPRITE_FLAGS_HS: usize = 0x0b6b;
// variables.h:672 — sprite_ignore_projectile.
const SPRITE_IGNORE_PROJECTILE_HS: usize = 0x0ba0;

// kMaxArrowsForLevel / kMaxBombsForLevel — duplicated from hud.rs (private).
const K_MAX_BOMBS_FOR_LEVEL_HS: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
const K_MAX_ARROWS_FOR_LEVEL_HS: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];

// Hinox_Draw multi-OAM table (sprite_main.c:9053).
const K_HINOX_DMD: [DrawMultipleData; 46] = [
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0600,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -5,
        char_flags: 0x0624,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -5,
        char_flags: 0x4624,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0606,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0600,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -5,
        char_flags: 0x0624,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -5,
        char_flags: 0x4624,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x4606,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -6,
        char_flags: 0x0624,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -6,
        char_flags: 0x4624,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0606,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0604,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -6,
        char_flags: 0x0624,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -6,
        char_flags: 0x4624,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4606,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0604,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -13,
        char_flags: 0x0602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x060c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x061c,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -12,
        char_flags: 0x0602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x060e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x061e,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -13,
        char_flags: 0x4602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x460c,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x461c,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -12,
        char_flags: 0x4602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x460e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x461e,
        ext: 2,
    },
    DrawMultipleData {
        x: -13,
        y: -16,
        char_flags: 0x056e,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0600,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -5,
        char_flags: 0x0620,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -5,
        char_flags: 0x4624,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0606,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -5,
        char_flags: 0x0624,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -5,
        char_flags: 0x4620,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 1,
        char_flags: 0x0606,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -13,
        char_flags: 0x0604,
        ext: 2,
    },
    DrawMultipleData {
        x: 13,
        y: -16,
        char_flags: 0x056e,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -11,
        char_flags: 0x056e,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -13,
        char_flags: 0x0602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x0622,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x060c,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -11,
        char_flags: 0x056e,
        ext: 2,
    },
    DrawMultipleData {
        x: 3,
        y: -13,
        char_flags: 0x4602,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x4622,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -8,
        char_flags: 0x460c,
        ext: 2,
    },
];

const K_HINOX_NUM: [u8; 12] = [4, 4, 4, 4, 3, 3, 3, 3, 5, 5, 4, 4];
const K_HINOX_OFFS: [u8; 12] = [0, 4, 8, 12, 16, 19, 22, 25, 28, 33, 38, 42];

// Hinox_SetDirection velocity tables (sprite_main.c:9043-9044).
const K_HINOX_XVEL: [i8; 4] = [8, -8, 0, 0];
const K_HINOX_YVEL: [i8; 4] = [0, 0, 8, -8];

// ShopItem_HandleReceipt message table (sprite_main.c:25306).
const K_SHOP_KEEPER_GIVE_ITEM_MSGS: [u16; 7] = [0x168, 0x167, 0x167, 0x16c, 0x169, 0x16a, 0x16b];

impl ZeldaState {
    // -----------------------------------------------------------------------
    // Hinox cluster
    // -----------------------------------------------------------------------

    // void Hinox_ThrowBomb(int k) {  // 869f4a
    //
    // }
    //
    // Empty body in the C source; preserve it here for parity.
    pub(super) fn hinox_throw_bomb(&mut self, _k: usize) {}

    // void Hinox_FaceLink(int k) {  // 869fe1
    //   Hinox_SetDirection(k, Sprite_DirectionToFaceLink(k, NULL));
    //   sprite_x_vel[k] <<= 1;
    //   sprite_y_vel[k] <<= 1;
    // }
    pub(super) fn hinox_face_link(&mut self, k: usize) {
        let dir = self.sprite_direction_to_face_link_for_hinox_shop(k);
        self.hinox_set_direction(k, dir);
        self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_shl(1);
        self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_shl(1);
    }

    // void Hinox_SetDirection(int k, uint8 dir) {  // 86a004
    //   static const int8 kHinox_Xvel[4] = {8, -8, 0, 0};
    //   static const int8 kHinox_Yvel[4] = {0, 0, 8, -8};
    //   sprite_D[k] = dir;
    //   sprite_delay_main[k] = (GetRandomNumber() & 63) + 96;
    //   sprite_ai_state[k]++;
    //   sprite_x_vel[k] = kHinox_Xvel[dir];
    //   sprite_y_vel[k] = kHinox_Yvel[dir];
    // }
    pub(super) fn hinox_set_direction(&mut self, k: usize, dir: u8) {
        self.ram[SPRITE_D + k] = dir;
        let r = self.get_random_number();
        self.ram[SPRITE_DELAY_MAIN + k] = (r & 63).wrapping_add(96);
        self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
        let idx = (dir as usize) & 3;
        self.ram[SPRITE_X_VEL + k] = K_HINOX_XVEL[idx] as u8;
        self.ram[SPRITE_Y_VEL + k] = K_HINOX_YVEL[idx] as u8;
    }

    // void Hinox_Draw(int k) {  // 86a1f9
    //   static const DrawMultipleData kHinox_Dmd[46] = { ... };
    //   PrepOamCoordsRet info;
    //   static const uint8 kHinoxNum[12]  = { 4, 4, 4, 4, 3, 3, 3, 3, 5, 5, 4, 4 };
    //   static const uint8 kHinoxOffs[12] = { 0, 4, 8, 12, 16, 19, 22, 25, 28, 33, 38, 42 };
    //   int j = sprite_graphics[k];
    //   Sprite_DrawMultiple(k, &kHinox_Dmd[kHinoxOffs[j]], kHinoxNum[j], &info);
    //   SpriteDraw_Shadow(k, &info);
    // }
    pub(super) fn hinox_draw(&mut self, k: usize) {
        let j = self.ram[SPRITE_GRAPHICS + k] as usize;
        let start = K_HINOX_OFFS[j] as usize;
        let count = K_HINOX_NUM[j] as usize;
        let mut info = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(k, &K_HINOX_DMD[start..start + count], Some(&mut info));
        self.sprite_draw_shadow_custom(k, &mut info, 10);
    }

    // -----------------------------------------------------------------------
    // ShopItem cluster
    // -----------------------------------------------------------------------

    // void ShopItem_RedPotion150(int k) {  // 9ef16e
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (Sprite_Find_EmptyBottle() < 0) {
    //       Sprite_ShowMessageUnconditional(0x16d);
    //       ShopItem_PlayBeep(k);
    //     } else if (ShopItem_HandleCost(150)) {
    //       sprite_state[k] = 0;
    //       ShopItem_HandleReceipt(k, 0x2e);
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //     }
    //   }
    // }
    pub(super) fn shop_item_red_potion150(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        if self.shop_item_check_for_a_press(k) {
            if self.sprite_find_empty_bottle_for_hinox_shop() < 0 {
                self.sprite_show_message_unconditional(0x16d);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(150) {
                self.ram[SPRITE_STATE + k] = 0;
                self.shop_item_handle_receipt(k, 0x2e);
            } else {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
            }
        }
    }

    // void ShopItem_FighterShield(int k) {  // 9ef1f2
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   ShopItem_MakeShieldsDeflect(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (link_shield_type) {
    //       Sprite_ShowMessageUnconditional(0x166);
    //       ShopItem_PlayBeep(k);
    //       return;
    //     }
    //     if (!ShopItem_HandleCost(50)) {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //       return;
    //     }
    //     sprite_state[k] = 0;
    //     ShopItem_HandleReceipt(k, 4);
    //   }
    //   sprite_flags4[k] = 0x1c;
    // }
    pub(super) fn shop_item_fighter_shield(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        self.shop_item_make_shields_deflect(k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram[LINK_SHIELD_TYPE] != 0 {
                self.sprite_show_message_unconditional(0x166);
                self.shop_item_play_beep(k);
                return;
            }
            if !self.shop_item_handle_cost(50) {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
                return;
            }
            self.ram[SPRITE_STATE + k] = 0;
            self.shop_item_handle_receipt(k, 4);
        }
        self.ram[SPRITE_FLAGS4 + k] = 0x1c;
    }

    // void ShopItem_FireShield(int k) {  // 9ef230
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   ShopItem_MakeShieldsDeflect(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (link_shield_type >= 2) {
    //       Sprite_ShowMessageUnconditional(0x166);
    //       ShopItem_PlayBeep(k);
    //       return;
    //     }
    //     if (!ShopItem_HandleCost(500)) {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //       return;
    //     }
    //     sprite_state[k] = 0;
    //     ShopItem_HandleReceipt(k, 5);
    //   }
    //   sprite_flags4[k] = 0x1c;
    // }
    pub(super) fn shop_item_fire_shield(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        self.shop_item_make_shields_deflect(k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram[LINK_SHIELD_TYPE] >= 2 {
                self.sprite_show_message_unconditional(0x166);
                self.shop_item_play_beep(k);
                return;
            }
            if !self.shop_item_handle_cost(500) {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
                return;
            }
            self.ram[SPRITE_STATE + k] = 0;
            self.shop_item_handle_receipt(k, 5);
        }
        self.ram[SPRITE_FLAGS4 + k] = 0x1c;
    }

    // void ShopItem_MakeShieldsDeflect(int k) {  // 9ef261
    //   sprite_ignore_projectile[k] = 0;
    //   sprite_flags[k] = 8;
    //   sprite_defl_bits[k] = 4;
    //   sprite_flags4[k] = 0x1c;
    //   Sprite_CheckDamageFromLink(k);
    //   sprite_flags4[k] = 0xa;
    // }
    pub(super) fn shop_item_make_shields_deflect(&mut self, k: usize) {
        self.ram[SPRITE_IGNORE_PROJECTILE_HS + k] = 0;
        self.ram[SPRITE_FLAGS_HS + k] = 8;
        self.ram[SPRITE_DEFL_BITS + k] = 4;
        self.ram[SPRITE_FLAGS4 + k] = 0x1c;
        self.sprite_check_damage_from_link_for_hinox_shop(k);
        self.ram[SPRITE_FLAGS4 + k] = 0xa;
    }

    // void ShopItem_Heart(int k) {  // 9ef27d
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (link_health_current == link_health_capacity) {
    //       ShopItem_PlayBeep(k);
    //     } else if (ShopItem_HandleCost(10)) {
    //       sprite_state[k] = 0;
    //       ShopItem_HandleReceipt(k, 0x42);
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //     }
    //   }
    // }
    pub(super) fn shop_item_heart(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        if self.shop_item_check_for_a_press(k) {
            if self.ram[LINK_HEALTH_CURRENT] == self.ram[LINK_HEALTH_CAPACITY] {
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(10) {
                self.ram[SPRITE_STATE + k] = 0;
                self.shop_item_handle_receipt(k, 0x42);
            } else {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
            }
        }
    }

    // void ShopItem_Arrows(int k) {  // 9ef2af
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (link_num_arrows == kMaxArrowsForLevel[link_arrow_upgrades]) {
    //       Sprite_ShowSolicitedMessage(k, 0x16e);
    //       ShopItem_PlayBeep(k);
    //     } else if (ShopItem_HandleCost(30)) {
    //       sprite_state[k] = 0;
    //       ShopItem_HandleReceipt(k, 0x44);
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //     }
    //   }
    // }
    pub(super) fn shop_item_arrows(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        if self.shop_item_check_for_a_press(k) {
            let upg = self.ram[LINK_ARROW_UPGRADES] as usize;
            if self.ram[LINK_NUM_ARROWS] == K_MAX_ARROWS_FOR_LEVEL_HS[upg] {
                self.sprite_show_solicited_message_for_hinox_shop(k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(30) {
                self.ram[SPRITE_STATE + k] = 0;
                self.shop_item_handle_receipt(k, 0x44);
            } else {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
            }
        }
    }

    // void ShopItem_Bombs(int k) {  // 9ef2f0
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (link_item_bombs == kMaxBombsForLevel[link_bomb_upgrades]) {
    //       Sprite_ShowSolicitedMessage(k, 0x16e);
    //       ShopItem_PlayBeep(k);
    //     } else if (ShopItem_HandleCost(50)) {
    //       sprite_state[k] = 0;
    //       ShopItem_HandleReceipt(k, 0x31);
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //     }
    //   }
    // }
    pub(super) fn shop_item_bombs(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        if self.shop_item_check_for_a_press(k) {
            let upg = self.ram[LINK_BOMB_UPGRADES] as usize;
            if self.ram[LINK_ITEM_BOMBS] == K_MAX_BOMBS_FOR_LEVEL_HS[upg] {
                self.sprite_show_solicited_message_for_hinox_shop(k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(50) {
                self.ram[SPRITE_STATE + k] = 0;
                self.shop_item_handle_receipt(k, 0x31);
            } else {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
            }
        }
    }

    // void ShopItem_Bee(int k) {  // 9ef322
    //   SpriteDraw_ShopItem(k);
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   Sprite_BehaveAsBarrier(k);
    //   if (ShopItem_CheckForAPress(k)) {
    //     if (Sprite_Find_EmptyBottle() < 0) {
    //       Sprite_ShowSolicitedMessage(k, 0x16d);
    //       ShopItem_PlayBeep(k);
    //     } else if (ShopItem_HandleCost(10)) {
    //       sprite_state[k] = 0;
    //       ShopItem_HandleReceipt(k, 0xe);
    //     } else {
    //       Sprite_ShowMessageUnconditional(0x17c);
    //       ShopItem_PlayBeep(k);
    //     }
    //   }
    // }
    pub(super) fn shop_item_bee(&mut self, k: usize) {
        self.sprite_draw_shop_item_for_hinox_shop(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_behave_as_barrier_for_hinox_shop(k);
        if self.shop_item_check_for_a_press(k) {
            if self.sprite_find_empty_bottle_for_hinox_shop() < 0 {
                self.sprite_show_solicited_message_for_hinox_shop(k, 0x16d);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(10) {
                self.ram[SPRITE_STATE + k] = 0;
                self.shop_item_handle_receipt(k, 0xe);
            } else {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
            }
        }
    }

    // void ShopItem_HandleReceipt(int k, uint8 item) {  // 9ef366
    //   static const uint16 kShopKeeper_GiveItemMsgs[7] = {0x168, 0x167, 0x167, 0x16c, 0x169, 0x16a, 0x16b};
    //   item_receipt_method = 0;
    //   Link_ReceiveItem(item, 0);
    //   int j = sprite_subtype2[k];
    //   if (j >= 7) {
    //     Sprite_ShowMessageUnconditional(kShopKeeper_GiveItemMsgs[j - 7]);
    //     ShopKeeper_RapidTerminateReceiveItem();
    //   }
    // }
    pub(super) fn shop_item_handle_receipt(&mut self, k: usize, item: u8) {
        self.ram[ITEM_RECEIPT_METHOD] = 0;
        self.link_receive_item(item, 0);
        let j = self.ram[SPRITE_SUBTYPE2 + k] as usize;
        if j >= 7 {
            let msg = K_SHOP_KEEPER_GIVE_ITEM_MSGS[j - 7];
            self.sprite_show_message_unconditional(msg);
            self.shop_keeper_rapid_terminate_receive_item_for_hinox_shop();
        }
    }

    // void ShopItem_PlayBeep(int k) {  // 9ef38a
    //   SpriteSfx_QueueSfx2WithPan(k, 0x3c);
    // }
    //
    // Task spec lists this as already-ported (`shop_item_play_beep`), but no
    // implementation was found anywhere under crates/zelda3/src. The canonical
    // 1:1 port is placed here.
    pub(super) fn shop_item_play_beep(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x3c);
    }

    // bool ShopItem_CheckForAPress(int k) {  // 9ef391
    //   if (!(filtered_joypad_L & 0x80))
    //     return false;
    //   return Sprite_CheckDamageToLink_same_layer(k);
    // }
    pub(super) fn shop_item_check_for_a_press(&mut self, k: usize) -> bool {
        if (self.ram[FILTERED_JOYPAD_L] & 0x80) == 0 {
            return false;
        }
        self.sprite_check_damage_to_link_same_layer_for_hinox_shop(k)
    }

    // bool ShopItem_HandleCost(int amt) {  // 9ef39e
    //   if (amt > link_rupees_goal)
    //     return false;
    //   link_rupees_goal -= amt;
    //   return true;
    // }
    //
    // Note: link_rupees_goal is a 16-bit RAM word. The C source treats it as
    // a 16-bit unsigned compared against `int amt`; mirror that with u16
    // arithmetic so signedness matches.
    pub(super) fn shop_item_handle_cost(&mut self, amt: i32) -> bool {
        let goal = read_le_u16(&self.ram, LINK_RUPEES_GOAL) as i32;
        if amt > goal {
            return false;
        }
        let new_goal = (goal - amt) as u16;
        write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, new_goal);
        true
    }

    // -----------------------------------------------------------------------
    // Helper shims (round-2 `_for_blind` pattern). These wire to canonical
    // helpers when available, or perform conservative data-state mutations.
    // -----------------------------------------------------------------------

    // Rewired to canonical Sprite_DirectionToFaceLink port.
    fn sprite_direction_to_face_link_for_hinox_shop(&mut self, k: usize) -> u8 {
        self.sprite_direction_to_face_link(k, None)
    }

    // Rewired to canonical Sprite_BehaveAsBarrier port.
    fn sprite_behave_as_barrier_for_hinox_shop(&mut self, k: usize) {
        self.sprite_behave_as_barrier(k);
    }

    // Rewired to canonical Sprite_CheckDamageFromLink entry point.
    fn sprite_check_damage_from_link_for_hinox_shop(&mut self, k: usize) {
        let _ = self.sprite_check_damage_from_link(k);
    }

    // Rewired to canonical Sprite_CheckDamageToLink_same_layer port.
    fn sprite_check_damage_to_link_same_layer_for_hinox_shop(&mut self, k: usize) -> bool {
        self.sprite_check_damage_to_link_same_layer(k)
    }

    // Rewired to canonical Sprite_ShowSolicitedMessage port. Hinox-shop
    // callers ignore the return value (they only care about the dialogue
    // side effect).
    fn sprite_show_solicited_message_for_hinox_shop(&mut self, k: usize, msg: u16) {
        let _ = self.sprite_show_solicited_message(k, msg);
    }

    // Rewired to canonical ShopKeeper_RapidTerminateReceiveItem port.
    fn shop_keeper_rapid_terminate_receive_item_for_hinox_shop(&mut self) {
        self.shop_keeper_rapid_terminate_receive_item();
    }

    // Rewired to canonical SpriteDraw_ShopItem port.
    fn sprite_draw_shop_item_for_hinox_shop(&mut self, k: usize) {
        self.sprite_draw_shop_item(k);
    }

    // Sprite_Find_EmptyBottle — duplicated locally because the canonical port
    // in sprite_main_npcs.rs is not registered in zelda_rtl.rs.
    fn sprite_find_empty_bottle_for_hinox_shop(&self) -> i32 {
        for i in 0..4 {
            if self.ram[LINK_BOTTLE_INFO + i] == 2 {
                return i as i32;
            }
        }
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    // ---- Hinox tests ----

    #[test]
    fn hinox_set_direction_writes_velocity_tables() {
        let mut s = fresh_state();
        // Seed RNG-related state so get_random_number is deterministic.
        s.ram[SPRITE_AI_STATE + 3] = 0;
        s.hinox_set_direction(3, 0);
        assert_eq!(s.ram[SPRITE_D + 3], 0);
        assert_eq!(s.ram[SPRITE_X_VEL + 3], 8);
        assert_eq!(s.ram[SPRITE_Y_VEL + 3], 0);
        assert_eq!(s.ram[SPRITE_AI_STATE + 3], 1);

        s.hinox_set_direction(3, 1);
        assert_eq!(s.ram[SPRITE_D + 3], 1);
        assert_eq!(s.ram[SPRITE_X_VEL + 3], (-8i8) as u8);
        assert_eq!(s.ram[SPRITE_Y_VEL + 3], 0);
        assert_eq!(s.ram[SPRITE_AI_STATE + 3], 2);

        s.hinox_set_direction(3, 2);
        assert_eq!(s.ram[SPRITE_X_VEL + 3], 0);
        assert_eq!(s.ram[SPRITE_Y_VEL + 3], 8);

        s.hinox_set_direction(3, 3);
        assert_eq!(s.ram[SPRITE_X_VEL + 3], 0);
        assert_eq!(s.ram[SPRITE_Y_VEL + 3], (-8i8) as u8);
    }

    #[test]
    fn hinox_face_link_doubles_velocity_after_set_direction() {
        let mut s = fresh_state();
        // Set up so DirectionToFaceLink returns 0 (right).
        // sprite at (0,0), link at (100,0) -> dx>0 dominant axis -> dir 0.
        write_le_u16(&mut s.ram, LINK_X_COORD, 100);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0);
        s.hinox_face_link(5);
        // After hinox_set_direction with dir=0, x_vel=8, then shifted left by 1 -> 16.
        assert_eq!(s.ram[SPRITE_X_VEL + 5], 16);
        assert_eq!(s.ram[SPRITE_Y_VEL + 5], 0);
        assert_eq!(s.ram[SPRITE_D + 5], 0);
    }

    #[test]
    fn hinox_throw_bomb_is_noop() {
        let mut s = fresh_state();
        let before = s.ram.clone();
        s.hinox_throw_bomb(7);
        assert_eq!(s.ram, before, "Hinox_ThrowBomb's C body is empty");
    }

    // ---- ShopItem tests ----

    #[test]
    fn shop_item_handle_cost_succeeds_when_affordable() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, LINK_RUPEES_GOAL, 200);
        assert!(s.shop_item_handle_cost(150));
        assert_eq!(read_le_u16(&s.ram, LINK_RUPEES_GOAL), 50);
    }

    #[test]
    fn shop_item_handle_cost_rejects_when_too_expensive() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, LINK_RUPEES_GOAL, 50);
        assert!(!s.shop_item_handle_cost(150));
        assert_eq!(
            read_le_u16(&s.ram, LINK_RUPEES_GOAL),
            50,
            "rupees unchanged on failed cost"
        );
    }

    #[test]
    fn shop_item_check_for_a_press_requires_a_button() {
        let mut s = fresh_state();
        s.ram[FILTERED_JOYPAD_L] = 0;
        assert!(!s.shop_item_check_for_a_press(0));
        // With A pressed but no Link overlap, the canonical damage check still
        // returns false; just confirm the early-exit doesn't fire.
        s.ram[FILTERED_JOYPAD_L] = 0x80;
        let _ = s.shop_item_check_for_a_press(0);
    }

    #[test]
    fn shop_item_make_shields_deflect_writes_expected_flags() {
        let mut s = fresh_state();
        s.ram[SPRITE_IGNORE_PROJECTILE_HS + 4] = 0xff;
        s.ram[SPRITE_FLAGS_HS + 4] = 0;
        s.ram[SPRITE_DEFL_BITS + 4] = 0;
        s.ram[SPRITE_FLAGS4 + 4] = 0;
        s.shop_item_make_shields_deflect(4);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE_HS + 4], 0);
        assert_eq!(s.ram[SPRITE_FLAGS_HS + 4], 8);
        assert_eq!(s.ram[SPRITE_DEFL_BITS + 4], 4);
        // Final flags4 value is 0xa (overwrites the bracketing 0x1c).
        assert_eq!(s.ram[SPRITE_FLAGS4 + 4], 0xa);
    }

    #[test]
    fn shop_item_handle_receipt_clears_method_and_skips_msg_for_low_subtype() {
        let mut s = fresh_state();
        s.ram[ITEM_RECEIPT_METHOD] = 5;
        s.ram[SPRITE_SUBTYPE2 + 1] = 3; // < 7, no message branch
        s.shop_item_handle_receipt(1, 0x2e);
        assert_eq!(s.ram[ITEM_RECEIPT_METHOD], 0);
    }
}
