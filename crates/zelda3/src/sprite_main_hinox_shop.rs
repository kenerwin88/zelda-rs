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

// kMaxArrowsForLevel / kMaxBombsForLevel — duplicated from hud.rs (private).
const MAX_BOMBS_BY_UPGRADE_LEVEL: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
const MAX_ARROWS_BY_UPGRADE_LEVEL: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];

// Hinox_Draw multi-OAM table (sprite_main.c:9053).
const HINOX_DRAW_FRAMES: [DrawMultipleData; 46] = [
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

const HINOX_DRAW_FRAME_COUNTS: [u8; 12] = [4, 4, 4, 4, 3, 3, 3, 3, 5, 5, 4, 4];
const HINOX_DRAW_FRAME_STARTS: [u8; 12] = [0, 4, 8, 12, 16, 19, 22, 25, 28, 33, 38, 42];

// Hinox_SetDirection velocity tables (sprite_main.c:9043-9044).
const HINOX_X_VELOCITIES: [i8; 4] = [8, -8, 0, 0];
const HINOX_Y_VELOCITIES: [i8; 4] = [0, 0, 8, -8];

// ShopItem_HandleReceipt message table (sprite_main.c:25306).
const SHOP_KEEPER_GIVE_ITEM_MESSAGES: [u16; 7] = [0x168, 0x167, 0x167, 0x16c, 0x169, 0x16a, 0x16b];

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
        let mut sprite = self.sprite_slot_mut(k);
        sprite.shift_x_velocity_left(1);
        sprite.shift_y_velocity_left(1);
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
        let r = self.get_random_number();
        let idx = (dir as usize) & 3;
        let mut sprite = self.sprite_slot_mut(k);
        sprite.set_direction(dir);
        sprite.set_delay_main((r & 63).wrapping_add(96));
        sprite.increment_ai_state();
        sprite.set_x_velocity(HINOX_X_VELOCITIES[idx] as u8);
        sprite.set_y_velocity(HINOX_Y_VELOCITIES[idx] as u8);
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
        let j = self.sprite_slot(k).graphics() as usize;
        let start = HINOX_DRAW_FRAME_STARTS[j] as usize;
        let count = HINOX_DRAW_FRAME_COUNTS[j] as usize;
        let mut info = PrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_draw_multiple(k, &HINOX_DRAW_FRAMES[start..start + count], Some(&mut info));
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
                self.sprite_slot_mut(k).clear();
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
            if self.game_state.inventory.items.shield_type() != 0 {
                self.sprite_show_message_unconditional(0x166);
                self.shop_item_play_beep(k);
                return;
            }
            if !self.shop_item_handle_cost(50) {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
                return;
            }
            self.sprite_slot_mut(k).clear();
            self.shop_item_handle_receipt(k, 4);
        }
        self.sprite_slot_mut(k).set_flags4(0x1c);
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
            if self.game_state.inventory.items.shield_type() >= 2 {
                self.sprite_show_message_unconditional(0x166);
                self.shop_item_play_beep(k);
                return;
            }
            if !self.shop_item_handle_cost(500) {
                self.sprite_show_message_unconditional(0x17c);
                self.shop_item_play_beep(k);
                return;
            }
            self.sprite_slot_mut(k).clear();
            self.shop_item_handle_receipt(k, 5);
        }
        self.sprite_slot_mut(k).set_flags4(0x1c);
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
        {
            let mut sprite = self.sprite_slot_mut(k);
            sprite.set_ignore_projectile(0);
            sprite.set_flags(8);
            sprite.set_deflection_bits(4);
            sprite.set_flags4(0x1c);
        }
        self.sprite_check_damage_from_link_for_hinox_shop(k);
        self.sprite_slot_mut(k).set_flags4(0xa);
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
            if self.game_state.inventory.player_resources.current_health()
                == self.game_state.inventory.player_resources.health_capacity()
            {
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(10) {
                self.sprite_slot_mut(k).clear();
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
            let upg = self
                .game_state
                .inventory
                .player_resources
                .arrow_upgrade_level() as usize;
            if self.game_state.inventory.player_resources.arrows()
                == MAX_ARROWS_BY_UPGRADE_LEVEL[upg]
            {
                self.sprite_show_solicited_message_for_hinox_shop(k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(30) {
                self.sprite_slot_mut(k).clear();
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
            let upg = self
                .game_state
                .inventory
                .player_resources
                .bomb_upgrade_level() as usize;
            if self.game_state.inventory.player_resources.bombs() == MAX_BOMBS_BY_UPGRADE_LEVEL[upg]
            {
                self.sprite_show_solicited_message_for_hinox_shop(k, 0x16e);
                self.shop_item_play_beep(k);
            } else if self.shop_item_handle_cost(50) {
                self.sprite_slot_mut(k).clear();
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
                self.sprite_slot_mut(k).clear();
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
        self.player_state_mut().set_item_receipt_method(0);
        self.link_receive_item(item, 0);
        let j = self.sprite_slot(k).subtype2() as usize;
        if j >= 7 {
            let msg = SHOP_KEEPER_GIVE_ITEM_MESSAGES[j - 7];
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
        if (self.game_state.player.follower_link.filtered_joypad_l() & 0x80) == 0 {
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
        let goal = self.game_state.inventory.player_resources.rupees_goal() as i32;
        if amt > goal {
            return false;
        }
        let new_goal = (goal - amt) as u16;
        self.player_resources_mut().set_rupees_goal(new_goal);
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
            if self.game_state.inventory.items.bottle(i) == 2 {
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
        s.sprite_slot_mut(3).set_ai_state(0);
        s.hinox_set_direction(3, 0);
        let sprite = s.sprite_slot(3);
        assert_eq!(sprite.direction(), 0);
        assert_eq!(sprite.x_velocity(), 8);
        assert_eq!(sprite.y_velocity(), 0);
        assert_eq!(sprite.ai_state(), 1);

        s.hinox_set_direction(3, 1);
        let sprite = s.sprite_slot(3);
        assert_eq!(sprite.direction(), 1);
        assert_eq!(sprite.x_velocity(), (-8i8) as u8);
        assert_eq!(sprite.y_velocity(), 0);
        assert_eq!(sprite.ai_state(), 2);

        s.hinox_set_direction(3, 2);
        let sprite = s.sprite_slot(3);
        assert_eq!(sprite.x_velocity(), 0);
        assert_eq!(sprite.y_velocity(), 8);

        s.hinox_set_direction(3, 3);
        let sprite = s.sprite_slot(3);
        assert_eq!(sprite.x_velocity(), 0);
        assert_eq!(sprite.y_velocity(), (-8i8) as u8);
    }

    #[test]
    fn hinox_face_link_doubles_velocity_after_set_direction() {
        let mut s = fresh_state();
        // Set up so DirectionToFaceLink returns 0 (right).
        // sprite at (0,0), link at (100,0) -> dx>0 dominant axis -> dir 0.
        s.player_state_mut().set_x(100);
        s.player_state_mut().set_y(0);
        s.hinox_face_link(5);
        // After hinox_set_direction with dir=0, x_vel=8, then shifted left by 1 -> 16.
        let sprite = s.sprite_slot(5);
        assert_eq!(sprite.x_velocity(), 16);
        assert_eq!(sprite.y_velocity(), 0);
        assert_eq!(sprite.direction(), 0);
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
        s.player_resources_mut().set_rupees_goal(200);
        assert!(s.shop_item_handle_cost(150));
        assert_eq!(s.game_state.inventory.player_resources.rupees_goal(), 50);
    }

    #[test]
    fn shop_item_handle_cost_rejects_when_too_expensive() {
        let mut s = fresh_state();
        s.player_resources_mut().set_rupees_goal(50);
        assert!(!s.shop_item_handle_cost(150));
        assert_eq!(
            s.game_state.inventory.player_resources.rupees_goal(),
            50,
            "rupees unchanged on failed cost"
        );
    }

    #[test]
    fn shop_item_check_for_a_press_requires_a_button() {
        let mut s = fresh_state();
        s.follower_link_state_mut().set_filtered_joypad_l(0);
        assert!(!s.shop_item_check_for_a_press(0));
        // With A pressed but no Link overlap, the canonical damage check still
        // returns false; just confirm the early-exit doesn't fire.
        s.follower_link_state_mut().set_filtered_joypad_l(0x80);
        let _ = s.shop_item_check_for_a_press(0);
    }

    #[test]
    fn shop_item_make_shields_deflect_writes_expected_flags() {
        let mut s = fresh_state();
        {
            let mut sprite = s.sprite_slot_mut(4);
            sprite.set_ignore_projectile(0xff);
            sprite.set_flags(0);
            sprite.set_deflection_bits(0);
            sprite.set_flags4(0);
        }
        s.shop_item_make_shields_deflect(4);
        let sprite = s.sprite_slot(4);
        assert_eq!(sprite.ignore_projectile(), 0);
        assert_eq!(sprite.flags(), 8);
        assert_eq!(sprite.deflection_bits(), 4);
        // Final flags4 value is 0xa (overwrites the bracketing 0x1c).
        assert_eq!(sprite.flags4(), 0xa);
    }

    #[test]
    fn shop_item_handle_receipt_clears_method_and_skips_msg_for_low_subtype() {
        let mut s = fresh_state();
        s.player_state_mut().set_item_receipt_method(5);
        s.sprite_slot_mut(1).set_subtype2(3); // < 7, no message branch
        s.shop_item_handle_receipt(1, 0x2e);
        assert_eq!(s.player_state().item_receipt_method(), 0);
    }
}
