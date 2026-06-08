#![allow(non_snake_case)]

// Methods ported from zelda3/src/hud.c and included inside ZeldaState.

use super::*;

const K_NEW_STYLE_INVENTORY: bool = false;
const K_HUD_ITEM_COUNT: usize = if K_NEW_STYLE_INVENTORY { 24 } else { 20 };

const K_FEATURES0_SWITCH_LR: u32 = 2;
const K_FEATURES0_DISABLE_LOW_HEALTH_BEEP: u32 = 64;
const K_FEATURES0_SHOW_MAX_ITEMS_IN_YELLOW: u32 = 256;
const K_FEATURES0_CARRY_MORE_RUPEES: u32 = 2048;
const K_FEATURES0_SWITCH_LR_LIMIT: u32 = 32768;

const K_JOYPAD_H_START: u8 = 0x10;
const K_JOYPAD_H_SELECT: u8 = 0x20;
const K_JOYPAD_H_Y: u8 = 0x40;
const K_JOYPAD_H_UP: u8 = 0x08;
const K_JOYPAD_H_DOWN: u8 = 0x04;
const K_JOYPAD_H_LEFT: u8 = 0x02;
const K_JOYPAD_H_RIGHT: u8 = 0x01;
const K_JOYPAD_L_X: u8 = 0x40;
const K_JOYPAD_L_L: u8 = 0x20;
const K_JOYPAD_L_R: u8 = 0x10;

const K_HUD_ITEM_BOTTLE_OLD: u8 = 16;
const K_HUD_ITEM_BOTTLE1: u8 = 21;
const K_HUD_ITEM_BOTTLE4: u8 = K_HUD_ITEM_BOTTLE1 + 3;
const K_HUD_ITEM_FLUTE: u8 = 13;
const K_HUD_ITEM_SHOVEL: u8 = 16;

type ItemBoxGfx = [u16; 4];

const K_MAX_BOMBS_FOR_LEVEL: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
const K_MAX_ARROWS_FOR_LEVEL: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];
const K_MAX_HEALTH_FOR_LEVEL: [u8; 21] = [
    9, 9, 9, 9, 9, 9, 9, 9, 17, 17, 17, 17, 17, 17, 17, 25, 25, 25, 25, 25, 25,
];

const K_HUD_ITEM_IN_VRAM_PTR_OLD: [usize; 20] = [
    hudxy(4, 7),
    hudxy(7, 7),
    hudxy(10, 7),
    hudxy(13, 7),
    hudxy(16, 7),
    hudxy(4, 10),
    hudxy(7, 10),
    hudxy(10, 10),
    hudxy(13, 10),
    hudxy(16, 10),
    hudxy(4, 13),
    hudxy(7, 13),
    hudxy(10, 13),
    hudxy(13, 13),
    hudxy(16, 13),
    hudxy(4, 16),
    hudxy(7, 16),
    hudxy(10, 16),
    hudxy(13, 16),
    hudxy(16, 16),
];

const K_HUD_ITEM_EMPTY: [ItemBoxGfx; 1] = [[0x20f5, 0x20f5, 0x20f5, 0x20f5]];
const K_HUD_ITEM_BOTTLES: [ItemBoxGfx; 9] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2044, 0x2045, 0x2046, 0x2047],
    [0x2837, 0x2838, 0x2cc3, 0x2cd3],
    [0x24d2, 0x64d2, 0x24e2, 0x24e3],
    [0x3cd2, 0x7cd2, 0x3ce2, 0x3ce3],
    [0x2cd2, 0x6cd2, 0x2ce2, 0x2ce3],
    [0x2855, 0x6855, 0x2c57, 0x2c5a],
    [0x2837, 0x2838, 0x2839, 0x283a],
    [0x2837, 0x2838, 0x2839, 0x283a],
];
const K_HUD_ITEM_BOW: [ItemBoxGfx; 5] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x28ba, 0x28e9, 0x28e8, 0x28cb],
    [0x28ba, 0x284a, 0x2849, 0x28cb],
    [0x28ba, 0x28e9, 0x28e8, 0x28cb],
    [0x28ba, 0x28bb, 0x24ca, 0x28cb],
];
const K_HUD_ITEM_BOOMERANG: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb8, 0x2cb9, 0x2cf5, 0x2cc9],
    [0x24b8, 0x24b9, 0x24f5, 0x24c9],
];
const K_HUD_ITEM_HOOKSHOT: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24f5, 0x24f6, 0x24c0, 0x24f5],
];
const K_HUD_ITEM_BOMBS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb2, 0x2cb3, 0x2cc2, 0x6cc2],
];
const K_HUD_ITEM_MUSHROOM: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2444, 0x2445, 0x2446, 0x2447],
    [0x203b, 0x203c, 0x203d, 0x203e],
];
const K_HUD_ITEM_FIRE_ROD: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24b0, 0x24b1, 0x24c0, 0x24c1],
];
const K_HUD_ITEM_ICE_ROD: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb0, 0x2cbe, 0x2cc0, 0x2cc1],
];
const K_HUD_ITEM_BOMBOS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x287d, 0x287e, 0xe87e, 0xe87d],
];
const K_HUD_ITEM_ETHER: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2876, 0x2877, 0xe877, 0xe876],
];
const K_HUD_ITEM_QUAKE: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2866, 0x2867, 0xe867, 0xe866],
];
const K_HUD_ITEM_TORCH: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24bc, 0x24bd, 0x24cc, 0x24cd],
];
const K_HUD_ITEM_HAMMER: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x20b6, 0x20b7, 0x20c6, 0x20c7],
];
const K_HUD_ITEM_FLUTE_GFX: [ItemBoxGfx; 4] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x20d0, 0x20d1, 0x20e0, 0x20e1],
    [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
    [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
];
const K_HUD_ITEM_BUG_NET: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3c40, 0x3c41, 0x2842, 0x3c43],
];
const K_HUD_ITEM_BOOK_MUDORA: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3ca5, 0x3ca6, 0x3cd8, 0x3cd9],
];
const K_HUD_ITEM_CANE_SOMARIA: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24dc, 0x24dd, 0x24ec, 0x24ed],
];
const K_HUD_ITEM_CANE_BYRNA: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cdc, 0x2cdd, 0x2cec, 0x2ced],
];
const K_HUD_ITEM_CAPE: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24b4, 0x24b5, 0x24c4, 0x24c5],
];
const K_HUD_ITEM_MIRROR: [ItemBoxGfx; 4] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x28de, 0x28df, 0x28ee, 0x28ef],
    [0x2c62, 0x2c63, 0x2c72, 0x2c73],
    [0x2886, 0x2887, 0x2888, 0x2889],
];
const K_HUD_ITEM_GLOVES: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2130, 0x2131, 0x2140, 0x2141],
    [0x28da, 0x28db, 0x28ea, 0x28eb],
];
const K_HUD_ITEM_BOOTS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3429, 0x342a, 0x342b, 0x342c],
];
const K_HUD_ITEM_FLIPPERS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2c9a, 0x2c9b, 0x2c9d, 0x2c9e],
];
const K_HUD_ITEM_MOON_PEARL: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2433, 0x2434, 0x2435, 0x2436],
];
const K_HUD_ITEM_SWORD: [ItemBoxGfx; 5] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2c64, 0x2cce, 0x2c75, 0x3d25],
    [0x2c8a, 0x2c65, 0x2474, 0x3d26],
    [0x248a, 0x2465, 0x3c74, 0x2d48],
    [0x288a, 0x2865, 0x2c74, 0x2d39],
];
const K_HUD_ITEM_SHIELD: [ItemBoxGfx; 4] = [
    [0x24f5, 0x24f5, 0x24f5, 0x24f5],
    [0x2cfd, 0x6cfd, 0x2cfe, 0x6cfe],
    [0x34ff, 0x74ff, 0x349f, 0x749f],
    [0x2880, 0x2881, 0x288d, 0x288e],
];
const K_HUD_ITEM_ARMOR: [ItemBoxGfx; 5] = [
    [0x3c68, 0x7c68, 0x3c78, 0x7c78],
    [0x2c68, 0x6c68, 0x2c78, 0x6c78],
    [0x2468, 0x6468, 0x2478, 0x6478],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

const K_UPDATE_MAGIC_POWER_TILEMAP: [[u16; 4]; 17] = [
    [0x3cf5, 0x3cf5, 0x3cf5, 0x3cf5],
    [0x3cf5, 0x3cf5, 0x3cf5, 0x3c5f],
    [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4c],
    [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4d],
    [0x3cf5, 0x3cf5, 0x3cf5, 0x3c4e],
    [0x3cf5, 0x3cf5, 0x3c5f, 0x3c5e],
    [0x3cf5, 0x3cf5, 0x3c4c, 0x3c5e],
    [0x3cf5, 0x3cf5, 0x3c4d, 0x3c5e],
    [0x3cf5, 0x3cf5, 0x3c4e, 0x3c5e],
    [0x3cf5, 0x3c5f, 0x3c5e, 0x3c5e],
    [0x3cf5, 0x3c4c, 0x3c5e, 0x3c5e],
    [0x3cf5, 0x3c4d, 0x3c5e, 0x3c5e],
    [0x3cf5, 0x3c4e, 0x3c5e, 0x3c5e],
    [0x3c5f, 0x3c5e, 0x3c5e, 0x3c5e],
    [0x3c4c, 0x3c5e, 0x3c5e, 0x3c5e],
    [0x3c4d, 0x3c5e, 0x3c5e, 0x3c5e],
    [0x3c4e, 0x3c5e, 0x3c5e, 0x3c5e],
];
const K_DUNG_FLOOR_INDICATOR_GFX0: [u16; 11] = [
    0x2508, 0x2509, 0x2509, 0x250a, 0x250b, 0x250c, 0x250d, 0x251d, 0xe51c, 0x250e, 0x7f,
];
const K_DUNG_FLOOR_INDICATOR_GFX1: [u16; 11] = [
    0x2518, 0x2519, 0xa509, 0x251a, 0x251b, 0x251c, 0x2518, 0xa51d, 0xe50c, 0xa50e, 0x7f,
];

impl ZeldaState {
    pub(super) fn hud_refresh_icon(&mut self) {
        self.hud_search_for_equipped_item();
        self.hud_update_hud();
        self.hud_rebuild();
        self.ram[OVERWORLD_MAP_STATE] = 0;
    }

    pub(super) fn check_palace_item_posession(&self) -> u8 {
        match self.ram[CUR_PALACE_INDEX_X2] >> 1 {
            2 => u8::from(self.ram[LINK_ITEM_BOW] != 0),
            3 => u8::from(self.ram[LINK_ITEM_GLOVES] != 0),
            5 => u8::from(self.ram[LINK_ITEM_HOOKSHOT] != 0),
            6 => u8::from(self.ram[LINK_ITEM_HAMMER] != 0),
            7 => u8::from(self.ram[LINK_ITEM_CANE_SOMARIA] != 0),
            8 => u8::from(self.ram[LINK_ITEM_FIRE_ROD] != 0),
            9 => u8::from(self.ram[LINK_ARMOR] != 0),
            10 => u8::from(self.ram[LINK_ITEM_MOON_PEARL] != 0),
            11 => u8::from(self.ram[LINK_ITEM_GLOVES] != 1),
            12 => u8::from(self.ram[LINK_SHIELD_TYPE] == 3),
            13 => u8::from(self.ram[LINK_ARMOR] == 2),
            _ => 0,
        }
    }

    fn hud_get_item_position(&self, item: u8) -> i32 {
        if item == 0 {
            return -1;
        }
        if self.ram[HUD_INVENTORY_ORDER] != 0 {
            for i in 0..K_HUD_ITEM_COUNT - 1 {
                if self.ram[HUD_INVENTORY_ORDER + i] == item {
                    return i as i32;
                }
            }
            (K_HUD_ITEM_COUNT - 1) as i32
        } else {
            i32::from(item) - 1
        }
    }

    fn hud_goto_prev_item(&self, item: &mut u8, first_item_index: u8) {
        if self.ram[HUD_INVENTORY_ORDER] != 0 {
            let pos = self.hud_get_item_position(*item);
            *item = if pos == 0 && first_item_index == 0 {
                0
            } else {
                let idx = ((if pos <= 0 {
                    K_HUD_ITEM_COUNT as i32
                } else {
                    pos
                }) - 1) as usize;
                self.ram[HUD_INVENTORY_ORDER + idx]
            };
        } else {
            *item = if *item > first_item_index {
                item.wrapping_sub(1)
            } else {
                K_HUD_ITEM_COUNT as u8
            };
        }
    }

    fn hud_goto_next_item(&self, item: &mut u8, first_item_index: u8) {
        if self.ram[HUD_INVENTORY_ORDER] != 0 {
            let i = self.hud_get_item_position(*item);
            let idx = if i as usize >= K_HUD_ITEM_COUNT - 1 {
                0
            } else {
                i as usize + 1
            };
            *item = self.ram[HUD_INVENTORY_ORDER + idx];
        } else {
            *item = if *item < K_HUD_ITEM_COUNT as u8 {
                item.wrapping_add(1)
            } else {
                first_item_index
            };
        }
    }

    pub(super) fn hud_floor_indicator(&mut self) {
        let mut a = u16::from(self.ram[HUD_FLOOR_CHANGED_TIMER]);
        if a == 0 {
            self.hud_remove_super_bomb_indicator();
            return;
        }
        a = a.wrapping_add(1);
        if a == 0xc0 {
            a = 0;
        }
        write_le_u16(&mut self.ram, HUD_FLOOR_CHANGED_TIMER, a);

        self.hud_buffer_set(0xf2 / 2, 0x251e);
        self.hud_buffer_set(0x134 / 2, 0x251f);
        self.hud_buffer_set(0x132 / 2, 0x2520);
        self.hud_buffer_set(0xf4 / 2, 0x250f);

        let mut k = 0usize;
        let j = if (self.ram[DUNG_CUR_FLOOR] as i8) >= 0 {
            if read_le_u16(&self.ram, DUNG_CUR_FLOOR) == 0
                && self.world_state_view().dungeon_room() != 2
                && self.ram[SRAM_PROGRESS_INDICATOR] < 2
            {
                self.ram[SOUND_EFFECT_AMBIENT] = 3;
            }
            self.ram[DUNG_CUR_FLOOR]
        } else {
            self.ram[SOUND_EFFECT_AMBIENT] = 5;
            k += 1;
            self.ram[DUNG_CUR_FLOOR] ^ 0xff
        } as usize;
        self.hud_buffer_set(k + 0xf2 / 2, K_DUNG_FLOOR_INDICATOR_GFX0[j]);
        self.hud_buffer_set(k + 0x132 / 2, K_DUNG_FLOOR_INDICATOR_GFX1[j]);
        self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
    }

    pub(super) fn hud_remove_super_bomb_indicator(&mut self) {
        self.hud_buffer_set(0xf2 / 2, 0x7f);
        self.hud_buffer_set(0x132 / 2, 0x7f);
        self.hud_buffer_set(0xf4 / 2, 0x7f);
        self.hud_buffer_set(0x134 / 2, 0x7f);
    }

    pub(super) fn hud_super_bomb_indicator(&mut self) {
        if self.ram[SUPER_BOMB_INDICATOR_COUNTER] == 0 {
            if (self.ram[SUPER_BOMB_INDICATOR_TIMER] as i8) < 0 {
                self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0xff;
                self.hud_remove_super_bomb_indicator();
                return;
            }
            self.ram[SUPER_BOMB_INDICATOR_TIMER] =
                self.ram[SUPER_BOMB_INDICATOR_TIMER].wrapping_sub(1);
            self.ram[SUPER_BOMB_INDICATOR_COUNTER] = 62;
        }
        self.ram[SUPER_BOMB_INDICATOR_COUNTER] =
            self.ram[SUPER_BOMB_INDICATOR_COUNTER].wrapping_sub(1);
        if (self.ram[SUPER_BOMB_INDICATOR_TIMER] as i8) < 0 {
            self.ram[SUPER_BOMB_INDICATOR_TIMER] = 0xff;
            self.hud_remove_super_bomb_indicator();
            return;
        }

        let r = self.ram[SUPER_BOMB_INDICATOR_TIMER] % 10;
        let q = self.ram[SUPER_BOMB_INDICATOR_TIMER] / 10;
        let j = if (r.wrapping_sub(1) as i8) < 0 {
            9
        } else {
            r - 1
        } as usize;
        self.hud_buffer_set(0xf4 / 2, K_DUNG_FLOOR_INDICATOR_GFX0[j]);
        self.hud_buffer_set(0x134 / 2, K_DUNG_FLOOR_INDICATOR_GFX1[j]);

        let j = if (q.wrapping_sub(1) as i8) < 0 {
            10
        } else {
            q - 1
        } as usize;
        self.hud_buffer_set(0xf2 / 2, K_DUNG_FLOOR_INDICATOR_GFX0[j]);
        self.hud_buffer_set(0x132 / 2, K_DUNG_FLOOR_INDICATOR_GFX1[j]);
    }

    fn max_rupees(&self) -> u16 {
        if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_CARRY_MORE_RUPEES != 0 {
            9999
        } else {
            999
        }
    }

    pub(super) fn hud_refill_logic(&mut self) {
        if self.ram[OVERWORLD_MAP_STATE] != 0 {
            return;
        }
        if self.ram[LINK_MAGIC_FILLER] != 0 {
            if self.ram[LINK_MAGIC_POWER] >= 128 {
                self.ram[LINK_MAGIC_POWER] = 128;
                self.ram[LINK_MAGIC_FILLER] = 0;
            } else {
                self.ram[LINK_MAGIC_FILLER] = self.ram[LINK_MAGIC_FILLER].wrapping_sub(1);
                self.ram[LINK_MAGIC_POWER] = self.ram[LINK_MAGIC_POWER].wrapping_add(1);
                if self.ram[FRAME_COUNTER] & 3 == 0 && self.ram[SOUND_EFFECT_1] == 0 {
                    self.ram[SOUND_EFFECT_1] = 45;
                }
            }
        }

        let mut a = read_le_u16(&self.ram, LINK_RUPEES_ACTUAL);
        let goal = read_le_u16(&self.ram, LINK_RUPEES_GOAL);
        if a != goal {
            if a >= goal {
                a = a.wrapping_sub(1);
                if (a as i16) < 0 {
                    a = 0;
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, 0);
                }
            } else {
                a = a.wrapping_add(1);
                let m = self.max_rupees();
                if a > m {
                    a = m;
                    write_le_u16(&mut self.ram, LINK_RUPEES_GOAL, m);
                }
            }
            write_le_u16(&mut self.ram, LINK_RUPEES_ACTUAL, a);
            if self.ram[SOUND_EFFECT_1] == 0 {
                let delay = self.ram[RUPEE_SFX_SOUND_DELAY];
                self.ram[RUPEE_SFX_SOUND_DELAY] = delay.wrapping_add(1);
                if delay & 7 == 0 {
                    self.ram[SOUND_EFFECT_1] = 41;
                }
            } else {
                self.ram[RUPEE_SFX_SOUND_DELAY] = 0;
            }
        } else {
            self.ram[RUPEE_SFX_SOUND_DELAY] = 0;
        }

        if self.ram[LINK_BOMB_FILLER] != 0 {
            self.ram[LINK_BOMB_FILLER] = self.ram[LINK_BOMB_FILLER].wrapping_sub(1);
            let max = K_MAX_BOMBS_FOR_LEVEL[self.ram[LINK_BOMB_UPGRADES] as usize];
            if self.ram[LINK_ITEM_BOMBS] != max {
                self.ram[LINK_ITEM_BOMBS] = self.ram[LINK_ITEM_BOMBS].wrapping_add(1);
            }
        }
        if self.ram[LINK_ARROW_FILLER] != 0 {
            self.ram[LINK_ARROW_FILLER] = self.ram[LINK_ARROW_FILLER].wrapping_sub(1);
            let max = K_MAX_ARROWS_FOR_LEVEL[self.ram[LINK_ARROW_UPGRADES] as usize];
            if self.ram[LINK_NUM_ARROWS] != max {
                self.ram[LINK_NUM_ARROWS] = self.ram[LINK_NUM_ARROWS].wrapping_add(1);
            }
            if self.ram[LINK_ITEM_BOW] != 0 && self.ram[LINK_ITEM_BOW] & 1 == 1 {
                self.ram[LINK_ITEM_BOW] = self.ram[LINK_ITEM_BOW].wrapping_add(1);
                self.hud_refresh_icon();
            }
        }

        let cap_idx = (self.ram[LINK_HEALTH_CAPACITY] >> 3) as usize;
        if self.ram[FLAG_IS_LINK_IMMOBILIZED] == 0
            && self.ram[LINK_HEARTS_FILLER] == 0
            && self.ram[LINK_HEALTH_CURRENT] < K_MAX_HEALTH_FOR_LEVEL[cap_idx]
        {
            if self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] != 0 {
                self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] =
                    self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP].wrapping_sub(1);
            } else if self.ram[SOUND_EFFECT_1] == 0 {
                if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_DISABLE_LOW_HEALTH_BEEP == 0
                {
                    self.ram[SOUND_EFFECT_1] = 43;
                }
                self.ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] = 31;
            }
        }

        if self.ram[IS_DOING_HEART_ANIMATION] != 0 {
            self.hud_update_magic();
            self.hud_update_inventory();
            self.hud_animate_heart_refill();
            self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
            return;
        }
        if self.ram[LINK_HEARTS_FILLER] != 0 {
            if self.ram[LINK_HEALTH_CURRENT] < self.ram[LINK_HEALTH_CAPACITY] {
                self.ram[LINK_HEALTH_CURRENT] = self.ram[LINK_HEALTH_CURRENT].wrapping_add(8);
                if self.ram[LINK_HEALTH_CURRENT] >= self.ram[LINK_HEALTH_CAPACITY] {
                    self.ram[LINK_HEALTH_CURRENT] = self.ram[LINK_HEALTH_CAPACITY];
                }
                if self.ram[SOUND_EFFECT_2] == 0 {
                    self.ram[SOUND_EFFECT_2] = 13;
                }
                self.ram[LINK_HEARTS_FILLER] = self.ram[LINK_HEARTS_FILLER].wrapping_sub(8);
                self.ram[IS_DOING_HEART_ANIMATION] =
                    self.ram[IS_DOING_HEART_ANIMATION].wrapping_add(1);
                self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] = 7;
                self.hud_update_magic();
                self.hud_update_inventory();
                self.hud_animate_heart_refill();
                self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
                return;
            }
            self.ram[LINK_HEALTH_CURRENT] = self.ram[LINK_HEALTH_CAPACITY];
            self.ram[LINK_HEARTS_FILLER] = 0;
        }
        self.hud_update_hearts();
        self.hud_update_magic();
        self.hud_update_inventory();
        self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
    }

    pub(super) fn hud_module_run(&mut self) {
        self.ram[HUD_MODULE_TICK_COUNTER] = self.ram[HUD_MODULE_TICK_COUNTER].wrapping_add(1);
        match self.ram[OVERWORLD_MAP_STATE] {
            0 => self.hud_clear_tile_map(),
            1 => self.hud_init(),
            2 => self.hud_bring_menu_down(),
            3 => self.hud_choose_next_mode(),
            4 => self.hud_normal_menu(),
            5 => self.hud_update_hud(),
            6 => self.hud_close_menu(),
            7 => self.hud_goto_bottle_menu(),
            8 => self.hud_init_bottle_menu(),
            9 => self.hud_expand_bottle_menu(),
            10 => self.hud_bottle_menu(),
            11 => self.hud_erase_bottle_menu(),
            12 => self.hud_restore_normal_menu(),
            // C Hud_Module_Run asserts outside states 0..=12.
            _ => panic!("invalid overworld_map_state for Hud_Module_Run"),
        }
    }

    pub(super) fn hud_clear_tile_map(&mut self) {
        for i in 0..1024 {
            write_le_u16(&mut self.ram, VRAM_UPLOAD_OFFSET + i * 2, 0x207f);
        }
        self.ram[SOUND_EFFECT_2] = 17;
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
    }

    fn hud_have_any_items(&self) -> bool {
        (0..20).any(|i| self.ram[LINK_ITEM_BOW + i] != 0)
    }

    pub(super) fn hud_init(&mut self) {
        self.hud_search_for_equipped_item();
        self.hud_draw_y_button_items();
        self.hud_draw_ability_box();
        self.hud_draw_progress_icons();
        self.hud_draw_equipment_box();
        self.hud_draw_selected_y_button_item();

        if self.hud_have_any_items() {
            let mut first_bottle = 0usize;
            while first_bottle < 4 && self.ram[LINK_BOTTLE_INFO + first_bottle] == 0 {
                first_bottle += 1;
            }
            if first_bottle == 4 {
                self.ram[LINK_ITEM_BOTTLE_INDEX] = 0;
            } else if self.ram[LINK_ITEM_BOTTLE_INDEX] == 0 {
                self.ram[LINK_ITEM_BOTTLE_INDEX] = first_bottle as u8 + 1;
            }

            if self.ram[HUD_CUR_ITEM] == K_HUD_ITEM_BOTTLE_OLD && !K_NEW_STYLE_INVENTORY {
                self.ram[TIMER_FOR_FLASHING_CIRCLE] = 16;
                self.hud_draw_bottle_menu();
            }
        }

        self.ram[TIMER_FOR_FLASHING_CIRCLE] = 16;
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
    }

    pub(super) fn hud_bring_menu_down(&mut self) {
        let bg3 = read_le_u16(&self.ram, BG3VOFS_COPY2).wrapping_sub(8);
        write_le_u16(&mut self.ram, BG3VOFS_COPY2, bg3);
        if bg3 == 0xff18 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
    }

    pub(super) fn hud_choose_next_mode(&mut self) {
        if self.hud_have_any_items() {
            self.ram[NMI_SUBROUTINE_INDEX] = 1;
            self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
            self.hud_draw_selected_y_button_item();
            self.ram[OVERWORLD_MAP_STATE] =
                if self.ram[HUD_CUR_ITEM] == K_HUD_ITEM_BOTTLE_OLD && !K_NEW_STYLE_INVENTORY {
                    10
                } else {
                    4
                };
        } else if self.ram[FILTERED_JOYPAD_H] != 0 {
            self.ram[OVERWORLD_MAP_STATE] = 5;
        }
    }

    fn hud_do_we_have_this_item(&self, item: u8) -> bool {
        if item == 0 {
            return true;
        }
        if item == K_HUD_ITEM_FLUTE && K_NEW_STYLE_INVENTORY {
            return self.ram[LINK_ITEM_FLUTE] >= 2;
        }
        if item == K_HUD_ITEM_SHOVEL && K_NEW_STYLE_INVENTORY {
            return self.ram[LINK_ITEM_FLUTE] >= 1;
        }
        if item >= K_HUD_ITEM_BOTTLE1 {
            return self.ram[LINK_BOTTLE_INFO + (item - K_HUD_ITEM_BOTTLE1) as usize] != 0;
        }
        self.ram[LINK_ITEM_BOW + (item - 1) as usize] != 0
    }

    fn hud_equip_prev_item(&self, item: &mut u8, is_hud_cur_item: bool) {
        loop {
            self.hud_goto_prev_item(item, u8::from(is_hud_cur_item));
            if self.hud_do_we_have_this_item(*item) {
                break;
            }
        }
    }

    fn hud_equip_next_item(&self, item: &mut u8, is_hud_cur_item: bool) {
        loop {
            self.hud_goto_next_item(item, u8::from(is_hud_cur_item));
            if self.hud_do_we_have_this_item(*item) {
                break;
            }
        }
    }

    fn hud_equip_item_above(&self, item: &mut u8) {
        loop {
            for _ in 0..if K_NEW_STYLE_INVENTORY { 6 } else { 5 } {
                self.hud_goto_prev_item(item, 1);
            }
            if self.hud_do_we_have_this_item(*item) {
                break;
            }
        }
    }

    fn hud_equip_item_below(&self, item: &mut u8) {
        let num = if *item == 0 {
            1
        } else if K_NEW_STYLE_INVENTORY {
            6
        } else {
            5
        };
        loop {
            for _ in 0..num {
                self.hud_goto_next_item(item, 1);
            }
            if self.hud_do_we_have_this_item(*item) {
                break;
            }
        }
    }

    pub(super) fn get_current_item_button_index(&self) -> usize {
        if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_SWITCH_LR != 0 {
            if self.ram[JOYPAD1L_LAST] & K_JOYPAD_L_X != 0 {
                1
            } else if self.ram[JOYPAD1L_LAST] & K_JOYPAD_L_L != 0 {
                2
            } else if self.ram[JOYPAD1L_LAST] & K_JOYPAD_L_R != 0 {
                3
            } else {
                0
            }
        } else {
            0
        }
    }

    fn get_current_item_button_ptr(&self, i: usize) -> usize {
        match i {
            0 => HUD_CUR_ITEM,
            1 => HUD_CUR_ITEM_X,
            2 => HUD_CUR_ITEM_L,
            _ => HUD_CUR_ITEM_R,
        }
    }

    pub(super) fn hud_normal_menu(&mut self) {
        self.ram[TIMER_FOR_FLASHING_CIRCLE] = self.ram[TIMER_FOR_FLASHING_CIRCLE].wrapping_add(1);
        if self.ram[JOYPAD1H_LAST] == 0 {
            self.ram[HUD_TMP1] = 0;
        }

        if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_START != 0 {
            self.ram[OVERWORLD_MAP_STATE] = 5;
            self.ram[SOUND_EFFECT_2] = 18;
            return;
        }

        if self.ram[JOYPAD1H_LAST] & K_JOYPAD_H_SELECT != 0
            && self.ram[SRAM_PROGRESS_INDICATOR] != 0
        {
            write_le_u16(&mut self.ram, BG3VOFS_COPY2, 0xfff8);
            self.hud_close_menu();
            self.DisplaySelectMenu();
            return;
        }

        if self.ram[JOYPAD1H_LAST] & K_JOYPAD_H_Y != 0
            && self.ram[JOYPAD1L_LAST] & K_JOYPAD_L_X == 0
            && self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_SWITCH_LR != 0
        {
            if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_UP != 0 {
                self.hud_reorder_item(if K_NEW_STYLE_INVENTORY { -6 } else { -5 });
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_DOWN != 0 {
                self.hud_reorder_item(if K_NEW_STYLE_INVENTORY { 6 } else { 5 });
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_LEFT != 0 {
                self.hud_reorder_item(-1);
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_RIGHT != 0 {
                self.hud_reorder_item(1);
            }
        } else if self.ram[HUD_TMP1] == 0 {
            let btn_index = self.get_current_item_button_index();
            let item_addr = self.get_current_item_button_ptr(btn_index);
            let mut item = self.ram[item_addr];
            let old_item = item;
            if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_UP != 0 {
                self.hud_equip_item_above(&mut item);
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_DOWN != 0 {
                self.hud_equip_item_below(&mut item);
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_LEFT != 0 {
                self.hud_equip_prev_item(&mut item, item_addr == HUD_CUR_ITEM);
            } else if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_RIGHT != 0 {
                self.hud_equip_next_item(&mut item, item_addr == HUD_CUR_ITEM);
            }
            self.ram[item_addr] = item;
            self.ram[HUD_TMP1] = self.ram[FILTERED_JOYPAD_H];
            if item != old_item {
                self.ram[TIMER_FOR_FLASHING_CIRCLE] = 16;
                self.ram[SOUND_EFFECT_2] = 32;
            }
        }

        self.hud_draw_y_button_items();
        self.hud_draw_selected_y_button_item();
        if self.ram[HUD_CUR_ITEM] == K_HUD_ITEM_BOTTLE_OLD && !K_NEW_STYLE_INVENTORY {
            self.ram[OVERWORLD_MAP_STATE] = 7;
        }

        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_update_hud(&mut self) {
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        self.hud_rebuild();
        self.hud_update_equipped_item();
    }

    pub(super) fn hud_lookup_inventory_item(&self, item: u8) -> u8 {
        const K_HUD_ITEM_TO_ITEM_ORG: [u8; 21] = [
            0, 3, 2, 14, 1, 10, 5, 6, 15, 16, 17, 9, 4, 8, 7, 12, 11, 18, 13, 19, 20,
        ];
        K_HUD_ITEM_TO_ITEM_ORG[item as usize]
    }

    pub(super) fn hud_update_equipped_item(&mut self) {
        if self.ram[HUD_CUR_ITEM] >= K_HUD_ITEM_BOTTLE1 {
            self.ram[LINK_ITEM_BOTTLE_INDEX] = self.ram[HUD_CUR_ITEM] - K_HUD_ITEM_BOTTLE1 + 1;
        }
        assert!(self.ram[HUD_CUR_ITEM] < 25);
        self.ram[CURRENT_ITEM_Y] = self.hud_lookup_inventory_item(self.ram[HUD_CUR_ITEM]);
    }

    pub(super) fn hud_close_menu(&mut self) {
        let bg3 = read_le_u16(&self.ram, BG3VOFS_COPY2).wrapping_add(8);
        write_le_u16(&mut self.ram, BG3VOFS_COPY2, bg3);
        if bg3 != 0 {
            return;
        }
        self.hud_rebuild();
        self.ram[OVERWORLD_MAP_STATE] = 0;
        self.frame_control_view_mut().set_submodule(0);
        let saved_module = self.ram[SAVED_MODULE_FOR_MENU];
        self.frame_control_view_mut().set_main_module(saved_module);
        if self.frame_control_view().submodule() != 0 {
            self.hud_restore_torch_background();
        }
        if self.ram[CURRENT_ITEM_Y] != 5 && self.ram[CURRENT_ITEM_Y] != 6 {
            self.ram[EQ_DEBUG_VARIABLE] = 2;
            self.ram[LINK_DEBUG_VALUE_1] = 0;
        } else {
            assert!(self.ram[LINK_DEBUG_VALUE_1] == 0);
            self.ram[EQ_DEBUG_VARIABLE] = 0;
        }
    }

    pub(super) fn hud_goto_bottle_menu(&mut self) {
        self.ram[BOTTLE_MENU_EXPAND_ROW] = 0;
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
    }

    pub(super) fn hud_init_bottle_menu(&mut self) {
        let r = self.ram[BOTTLE_MENU_EXPAND_ROW] as usize;
        for i in 21..=30 {
            self.menu_set(hudxy(i, 11 + r), 0x207f);
        }
        self.ram[BOTTLE_MENU_EXPAND_ROW] = self.ram[BOTTLE_MENU_EXPAND_ROW].wrapping_add(1);
        if self.ram[BOTTLE_MENU_EXPAND_ROW] == 19 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
            self.ram[BOTTLE_MENU_EXPAND_ROW] = 17;
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_expand_bottle_menu(&mut self) {
        const K_BOTTLE_MENU_TOP: [u16; 10] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb,
        ];
        const K_BOTTLE_MENU_TOP2: [u16; 10] = [
            0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc,
        ];
        const K_BOTTLE_MENU_BOTTOM: [u16; 10] = [
            0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xe8fb,
        ];
        let r = self.ram[BOTTLE_MENU_EXPAND_ROW] as usize;
        self.hud_draw_nx_n(0x1000, hudxy(21, 11 + r), &K_BOTTLE_MENU_TOP, 10, 1);
        self.hud_draw_nx_n(0x1000, hudxy(21, 12 + r), &K_BOTTLE_MENU_TOP2, 10, 1);
        self.hud_draw_nx_n(0x1000, hudxy(21, 29), &K_BOTTLE_MENU_BOTTOM, 10, 1);
        self.ram[BOTTLE_MENU_EXPAND_ROW] = self.ram[BOTTLE_MENU_EXPAND_ROW].wrapping_sub(1);
        if (self.ram[BOTTLE_MENU_EXPAND_ROW] as i8) < 0 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_bottle_menu(&mut self) {
        self.ram[TIMER_FOR_FLASHING_CIRCLE] = self.ram[TIMER_FOR_FLASHING_CIRCLE].wrapping_add(1);
        if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_START != 0 {
            self.ram[SOUND_EFFECT_2] = 18;
            self.ram[OVERWORLD_MAP_STATE] = 5;
        } else if self.ram[FILTERED_JOYPAD_H] & (K_JOYPAD_H_LEFT | K_JOYPAD_H_RIGHT) != 0 {
            let mut item = self.ram[HUD_CUR_ITEM];
            if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_LEFT != 0 {
                self.hud_equip_prev_item(&mut item, true);
            } else {
                self.hud_equip_next_item(&mut item, true);
            }
            self.ram[HUD_CUR_ITEM] = item;
            self.ram[TIMER_FOR_FLASHING_CIRCLE] = 16;
            self.ram[SOUND_EFFECT_2] = 32;
            self.hud_draw_y_button_items();
            self.hud_draw_selected_y_button_item();
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
            self.ram[BOTTLE_MENU_EXPAND_ROW] = 0;
            return;
        }
        self.hud_draw_bottle_menu_update();
        if self.ram[FILTERED_JOYPAD_H] & (K_JOYPAD_H_DOWN | K_JOYPAD_H_UP) != 0 {
            let old_val = self.ram[LINK_ITEM_BOTTLE_INDEX].wrapping_sub(1) & 3;
            let mut val = old_val;
            if self.ram[FILTERED_JOYPAD_H] & K_JOYPAD_H_UP != 0 {
                loop {
                    val = val.wrapping_sub(1) & 3;
                    if self.ram[LINK_BOTTLE_INFO + val as usize] != 0 {
                        break;
                    }
                }
            } else {
                loop {
                    val = val.wrapping_add(1) & 3;
                    if self.ram[LINK_BOTTLE_INFO + val as usize] != 0 {
                        break;
                    }
                }
            }
            if old_val != val {
                self.ram[LINK_ITEM_BOTTLE_INDEX] = val + 1;
                self.ram[TIMER_FOR_FLASHING_CIRCLE] = 16;
                self.ram[SOUND_EFFECT_2] = 32;
            }
        }
    }

    pub(super) fn hud_draw_bottle_menu_update(&mut self) {
        self.hud_draw_bottle_menu();
        self.hud_draw_selected_y_button_item();
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_erase_bottle_menu(&mut self) {
        let r = self.ram[BOTTLE_MENU_EXPAND_ROW] as usize;
        for i in 0..10 {
            self.menu_set(hudxy(21 + i, 11 + r), 0x207f);
        }
        self.ram[BOTTLE_MENU_EXPAND_ROW] = self.ram[BOTTLE_MENU_EXPAND_ROW].wrapping_add(1);
        if self.ram[BOTTLE_MENU_EXPAND_ROW] == 19 {
            self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_restore_normal_menu(&mut self) {
        self.hud_draw_progress_icons();
        self.hud_draw_equipment_box();
        self.ram[OVERWORLD_MAP_STATE] = 4;
        self.ram[NMI_SUBROUTINE_INDEX] = 1;
        self.ram[NMI_LOAD_TARGET_ADDR] = 0x22;
    }

    pub(super) fn hud_search_for_equipped_item(&mut self) {
        if !self.hud_have_any_items() {
            self.ram[HUD_CUR_ITEM] = 0;
            self.ram[HUD_VAR1] = 0;
        } else {
            if self.ram[HUD_CUR_ITEM] == 0 {
                self.ram[HUD_CUR_ITEM] = 1;
            }
            if !self.hud_do_we_have_this_item(self.ram[HUD_CUR_ITEM]) {
                let mut item = self.ram[HUD_CUR_ITEM];
                self.hud_equip_next_item(&mut item, true);
                self.ram[HUD_CUR_ITEM] = item;
            }
        }
    }

    fn hud_get_icon_for_item(&self, i: u8) -> ItemBoxGfx {
        if i == 0 {
            return K_HUD_ITEM_EMPTY[0];
        }
        if i >= K_HUD_ITEM_BOTTLE1 {
            return K_HUD_ITEM_BOTTLES
                [self.ram[LINK_BOTTLE_INFO + (i - K_HUD_ITEM_BOTTLE1) as usize] as usize];
        }
        let mut item_val = self.ram[LINK_ITEM_BOW + (i - 1) as usize] as usize;
        if i == 4 {
            item_val = usize::from(item_val != 0);
        } else if i == K_HUD_ITEM_BOTTLE_OLD && !K_NEW_STYLE_INVENTORY {
            item_val = if self.ram[LINK_ITEM_BOTTLE_INDEX] != 0 {
                self.ram[LINK_BOTTLE_INFO + self.ram[LINK_ITEM_BOTTLE_INDEX] as usize - 1] as usize
            } else {
                0
            };
        }
        Self::hud_item_box_gfx_ptr(i - 1)[item_val]
    }

    pub(super) fn hud_draw_y_button_items(&mut self) {
        let x = if K_NEW_STYLE_INVENTORY { 0 } else { 1 };
        let btn_index = self.get_current_item_button_index();
        self.copy_tiles_for_switch_lr(btn_index);
        self.hud_draw_box(0x1000, x, 5, 20 - x, 19, K_SWITCH_LR_PALETTES[btn_index]);
        const K_EQUIPMENT_LETTER_TILES: [[u16; 2]; 4] = [
            [0x3cf0, 0x3cf1],
            [0x2cf0, 0xacf0],
            [0x300e, 0x300f],
            [0x300c, 0x300d],
        ];

        if !K_NEW_STYLE_INVENTORY {
            self.menu_set(hudxy(2, 6), K_EQUIPMENT_LETTER_TILES[btn_index][0]);
            self.menu_set(hudxy(2, 7), K_EQUIPMENT_LETTER_TILES[btn_index][1]);
        }
        self.menu_set(hudxy(x + 2, 5), 0x246e);
        self.menu_set(hudxy(x + 3, 5), 0x246f);

        for i in 0..K_HUD_ITEM_COUNT {
            let j = self.ram[HUD_INVENTORY_ORDER + i];
            let item = if j == 0 { i as u8 + 1 } else { j };
            let icon = self.hud_get_icon_for_item(item);
            self.hud_draw_item(0x1000, K_HUD_ITEM_IN_VRAM_PTR_OLD[i], &icon);
        }
    }

    fn copy_tiles_for_switch_lr(&mut self, switch_lr: usize) {
        if switch_lr == 3 {
            const TOP_OF_R: [u16; 8] = [
                pv([1, 1, 1, 1, 1, 1, 3, 3]),
                pv([1, 1, 1, 1, 1, 1, 1, 3]),
                pv([1, 1, 1, 1, 1, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 1, 1, 1, 1, 3]),
            ];
            const BOTTOM_OF_R: [u16; 8] = [
                pv([1, 1, 1, 1, 1, 1, 3, 3]),
                pv([1, 1, 1, 3, 1, 1, 1, 3]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
                pv([1, 1, 1, 3, 3, 1, 1, 1]),
            ];
            self.ppu.vram[0x7000 + 0xc * 8..0x7000 + 0xd * 8].copy_from_slice(&TOP_OF_R);
            self.ppu.vram[0x7000 + 0xd * 8..0x7000 + 0xe * 8].copy_from_slice(&BOTTOM_OF_R);
        } else if switch_lr == 2 {
            const TOP_OF_L: [u16; 8] = [
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
            ];
            const BOTTOM_OF_L: [u16; 8] = [
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 3, 3, 3, 3, 3]),
                pv([1, 1, 1, 1, 1, 1, 1, 1]),
                pv([1, 1, 1, 1, 1, 1, 1, 1]),
                pv([1, 1, 1, 1, 1, 1, 1, 1]),
            ];
            self.ppu.vram[0x7000 + 0xe * 8..0x7000 + 0xf * 8].copy_from_slice(&TOP_OF_L);
            self.ppu.vram[0x7000 + 0xf * 8..0x7000 + 0x10 * 8].copy_from_slice(&BOTTOM_OF_L);
        }
    }

    pub(super) fn hud_draw_ability_box(&mut self) {
        const K_HUD_ABILITY_TEXT: [u16; 80] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d27, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d61, 0x2d54, 0x2d50, 0x2d53, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d63, 0x2d50, 0x2d5b, 0x2d5a, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x2cf5, 0x2cf5, 0x2c2e, 0x2cf5,
            0x2cf5, 0x2d5f, 0x2d64, 0x2d5b, 0x2d5b, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2d61, 0x2d64, 0x2d5d, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2d62, 0x2d66, 0x2d58, 0x2d5c, 0x2cf5, 0x2cf5, 0x2cf5, 0x207f, 0x207f, 0x2c01, 0x2c18,
            0x2c28, 0x207f, 0x207f,
        ];
        const K_HUD_GLOVES_TEXT: [u16; 20] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d28, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d29,
        ];
        let x = if K_NEW_STYLE_INVENTORY { 0 } else { 1 };
        self.hud_draw_box(0x1000, x, 21, 19, 29, 1);

        let mut flags = self.ram[LINK_ABILITY_FLAGS];
        for i in 0..2 {
            for j in 0..3 {
                if flags & 0x80 != 0 {
                    let src = &K_HUD_ABILITY_TEXT[i * 40 + j * 10..];
                    self.hud_draw_nx_n(0x1000, hudxy(4 + j * 5, 22 + i * 2), src, 5, 2);
                }
                flags <<= 1;
            }
            flags <<= 1;
        }
        if !K_NEW_STYLE_INVENTORY {
            self.menu_set(hudxy(2, 22), 0xa4f0);
            self.menu_set(hudxy(2, 23), 0x24f2);
        }
        self.menu_set(hudxy(x + 2, 21), 0x2482);
        self.menu_set(hudxy(x + 3, 21), 0x2483);

        self.hud_draw_item(
            0x1000,
            hudxy(8, 27),
            &K_HUD_ITEM_GLOVES[self.ram[LINK_ITEM_GLOVES] as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(4, 27),
            &K_HUD_ITEM_BOOTS[self.ram[LINK_ITEM_BOOTS] as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(12, 27),
            &K_HUD_ITEM_FLIPPERS[self.ram[LINK_ITEM_FLIPPERS] as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(16, 27),
            &K_HUD_ITEM_MOON_PEARL[self.ram[LINK_ITEM_MOON_PEARL] as usize],
        );
        if self.ram[LINK_ITEM_GLOVES] != 0 {
            let src = &K_HUD_GLOVES_TEXT[usize::from(self.ram[LINK_ITEM_GLOVES] != 1) * 10..];
            self.hud_draw_nx_n(0x1000, hudxy(4, 22), src, 5, 2);
        }
    }

    pub(super) fn hud_draw_progress_icons(&mut self) {
        if self.ram[SRAM_PROGRESS_INDICATOR] < 3 {
            self.hud_draw_progress_icons_pendants();
        } else {
            self.hud_draw_progress_icons_crystals();
        }
    }

    pub(super) fn hud_draw_progress_icons_pendants(&mut self) {
        const BG: [u16; 90] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb, 0x28fc,
            0x2521, 0x2522, 0x2523, 0x2524, 0x253f, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5,
            0x24f5, 0x213b, 0x213c, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5,
            0x213d, 0x213e, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x213b, 0x213c, 0x24f5, 0x24f5,
            0x213b, 0x213c, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x213d, 0x213e, 0x24f5, 0x24f5, 0x213d,
            0x213e, 0x24f5, 0x68fc, 0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9,
            0xa8f9, 0xe8fb,
        ];
        const P0: [ItemBoxGfx; 2] = [
            [0x313b, 0x313c, 0x313d, 0x313e],
            [0x252b, 0x252c, 0x252d, 0x252e],
        ];
        const P1: [ItemBoxGfx; 2] = [
            [0x313b, 0x313c, 0x313d, 0x313e],
            [0x2d2b, 0x2d2c, 0x2d2d, 0x2d2e],
        ];
        const P2: [ItemBoxGfx; 2] = [
            [0x313b, 0x313c, 0x313d, 0x313e],
            [0x3d2b, 0x3d2c, 0x3d2d, 0x3d2e],
        ];
        let dst = if K_NEW_STYLE_INVENTORY {
            hudxy(22, 11)
        } else {
            hudxy(21, 11)
        };
        self.hud_draw_nx_n(0x1000, dst, &BG, 10, 9);
        let f = self.ram[LINK_WHICH_PENDANTS];
        self.hud_draw_item(0x1000, dst + hudxy(4, 3), &P0[(f & 1) as usize]);
        self.hud_draw_item(0x1000, dst + hudxy(2, 6), &P1[((f >> 1) & 1) as usize]);
        self.hud_draw_item(0x1000, dst + hudxy(6, 6), &P2[((f >> 2) & 1) as usize]);
    }

    pub(super) fn hud_draw_progress_icons_crystals(&mut self) {
        const BG: [u16; 90] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb, 0x28fc,
            0x252f, 0x2534, 0x2535, 0x2536, 0x2537, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5,
            0x3146, 0x3147, 0x3146, 0x3147, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x3146, 0x3147, 0x3146,
            0x3147, 0x3146, 0x3147, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
            0x24f5, 0x24f5, 0x24f5, 0x68fc, 0x28fc, 0x24f5, 0x24f5, 0x3146, 0x3147, 0x3146, 0x3147,
            0x24f5, 0x24f5, 0x68fc, 0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9,
            0xa8f9, 0xe8fb,
        ];
        let dst = if K_NEW_STYLE_INVENTORY {
            hudxy(22, 11)
        } else {
            hudxy(21, 11)
        };
        self.hud_draw_nx_n(0x1000, dst, &BG, 10, 9);
        let f = self.ram[LINK_HAS_CRYSTALS];
        for (bit, x, y) in [
            (1, 3, 3),
            (2, 5, 3),
            (4, 2, 5),
            (8, 4, 5),
            (16, 6, 5),
            (32, 3, 7),
            (64, 5, 7),
        ] {
            if f & bit != 0 {
                self.menu_set(dst + hudxy(x, y), 0x2d44);
                self.menu_set(dst + hudxy(x + 1, y), 0x2d45);
            }
        }
    }

    pub(super) fn hud_draw_selected_y_button_item(&mut self) {
        let dst_box = if K_NEW_STYLE_INVENTORY { 1 } else { 0 };
        let btn_index = self.get_current_item_button_index();
        let item = self.ram[self.get_current_item_button_ptr(btn_index)];
        self.hud_draw_box(
            0x1000,
            21 + dst_box,
            5,
            30 + dst_box,
            10,
            K_SWITCH_LR_PALETTES[btn_index],
        );
        if item != 0 {
            let pos = self.hud_get_item_position(item);
            if pos >= 0 {
                let src = K_HUD_ITEM_IN_VRAM_PTR_OLD[pos as usize];
                self.hud_copy2x2(0x1000, hudxy(25 + dst_box, 6), 0x1000, src);
                if self.ram[TIMER_FOR_FLASHING_CIRCLE] & 0x10 != 0 {
                    self.hud_draw_flashing_circle(
                        0x1000,
                        src as i32,
                        K_SWITCH_LR_PALETTES[btn_index],
                    );
                }
            }
        }
        let text: &[u16] = if item == K_HUD_ITEM_BOTTLE_OLD
            && !K_NEW_STYLE_INVENTORY
            && self.ram[LINK_ITEM_BOTTLE_INDEX] != 0
        {
            let idx = (self.ram[LINK_BOTTLE_INFO + self.ram[LINK_ITEM_BOTTLE_INDEX] as usize - 1]
                as usize
                - 1)
                * 16;
            &K_HUD_BOTTLES_ITEM_TEXT[idx..idx + 16]
        } else if item == 5 && self.ram[LINK_ITEM_MUSHROOM] != 1 {
            let idx = (self.ram[LINK_ITEM_MUSHROOM] as usize - 2) * 16;
            &K_HUD_MUSHROOM_ITEM_TEXT[idx..idx + 16]
        } else if item == 20 && self.ram[LINK_ITEM_MIRROR] != 1 {
            let idx = (self.ram[LINK_ITEM_MIRROR] as usize - 2) * 16;
            &K_HUD_MIRROR_ITEM_TEXT[idx..idx + 16]
        } else if item == 13 && self.ram[LINK_ITEM_FLUTE] != 1 {
            let idx = (self.ram[LINK_ITEM_FLUTE] as usize - 2) * 16;
            &K_HUD_FLUTE_ITEM_TEXT[idx..idx + 16]
        } else if item == 1 && self.ram[LINK_ITEM_BOW] != 1 {
            let idx = (self.ram[LINK_ITEM_BOW] as usize - 2) * 16;
            &K_HUD_BOW_ITEM_TEXT[idx..idx + 16]
        } else if item >= K_HUD_ITEM_BOTTLE1 && item <= K_HUD_ITEM_BOTTLE4 {
            let idx = (self.ram[LINK_BOTTLE_INFO + (item - K_HUD_ITEM_BOTTLE1) as usize] as usize
                - 1)
                * 16;
            &K_HUD_BOTTLES_ITEM_TEXT[idx..idx + 16]
        } else if item == K_HUD_ITEM_SHOVEL {
            &K_HUD_ITEM_TEXT[12]
        } else if item == 0 && btn_index != 0 {
            &K_NOT_ASSIGNED_ITEM_TEXT
        } else if item == 0 {
            &K_HUD_ITEM_TEXT[19]
        } else {
            &K_HUD_ITEM_TEXT[item as usize - 1]
        };
        self.hud_draw_nx_n(0x1000, hudxy(22 + dst_box, 8), text, 8, 2);
    }

    pub(super) fn hud_draw_equipment_box(&mut self) {
        let dst = if K_NEW_STYLE_INVENTORY { 1 } else { 0 };
        self.hud_draw_box(0x1000, 21 + dst, 21, 30 + dst, 29, 2);
        for i in 0..8 {
            self.menu_set(hudxy(22 + dst + i, 25), 0x28d7);
        }
        const TEXT: [u16; 16] = [
            0x2479, 0x247a, 0x247b, 0x247c, 0x248c, 0x24f5, 0x24f5, 0x24f5, 0x2469, 0x246a, 0x246b,
            0x246c, 0x246d, 0x246e, 0x246f, 0x24f5,
        ];
        self.hud_draw_nx_n(0x1000, hudxy(22 + dst, 22), &TEXT[0..8], 8, 1);
        self.hud_draw_nx_n(0x1000, hudxy(22 + dst, 26), &TEXT[8..16], 8, 1);
        const HEART_PIECES: [ItemBoxGfx; 4] = [
            [0x2484, 0x6484, 0x2485, 0x6485],
            [0x24ad, 0x6484, 0x2485, 0x6485],
            [0x24ad, 0x6484, 0x24ae, 0x6485],
            [0x24ad, 0x64ad, 0x24ae, 0x6485],
        ];
        if self.ram[CUR_PALACE_INDEX_X2] == 0xff {
            for i in 0..8 {
                self.menu_set(hudxy(22 + dst + i, 26), 0x24f5);
            }
            self.hud_draw_item(
                0x1000,
                hudxy(25 + dst, 27),
                &HEART_PIECES[self.ram[LINK_HEART_PIECES] as usize],
            );
        }
        let sword = if self.ram[LINK_SWORD_TYPE] == 0xff {
            0
        } else {
            self.ram[LINK_SWORD_TYPE] as usize
        };
        self.hud_draw_item(0x1000, hudxy(22 + dst, 23), &K_HUD_ITEM_SWORD[sword]);
        self.hud_draw_item(
            0x1000,
            hudxy(25 + dst, 23),
            &K_HUD_ITEM_SHIELD[self.ram[LINK_SHIELD_TYPE] as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(28 + dst, 23),
            &K_HUD_ITEM_ARMOR[self.ram[LINK_ARMOR] as usize],
        );
        const PALACE_ITEM: [ItemBoxGfx; 2] = [
            [0x28d6, 0x68d6, 0x28e6, 0x28e7],
            [0x354b, 0x354c, 0x354d, 0x354e],
        ];
        const DUNGEON_MAP: ItemBoxGfx = [0x28de, 0x28df, 0x28ee, 0x28ef];
        const DUNGEON_COMPASS: ItemBoxGfx = [0x24bf, 0x64bf, 0x2ccf, 0x6ccf];
        let shift = self.ram[CUR_PALACE_INDEX_X2] >> 1;
        if self.ram[CUR_PALACE_INDEX_X2] != 0xff
            && (read_le_u16(&self.ram, LINK_BIGKEY) << shift) & 0x8000 != 0
        {
            self.hud_draw_item(
                0x1000,
                hudxy(28 + dst, 27),
                &PALACE_ITEM[self.check_palace_item_posession() as usize],
            );
        }
        if self.ram[CUR_PALACE_INDEX_X2] != 0xff
            && (read_le_u16(&self.ram, LINK_DUNGEON_MAP) << shift) & 0x8000 != 0
        {
            self.hud_draw_item(0x1000, hudxy(22 + dst, 27), &DUNGEON_MAP);
        }
        if self.ram[CUR_PALACE_INDEX_X2] != 0xff
            && (read_le_u16(&self.ram, LINK_COMPASS) << shift) & 0x8000 != 0
        {
            self.hud_draw_item(0x1000, hudxy(25 + dst, 27), &DUNGEON_COMPASS);
        }
    }

    pub(super) fn hud_draw_bottle_menu(&mut self) {
        let dst = if K_NEW_STYLE_INVENTORY { 1 } else { 0 };
        self.hud_draw_box(0x1000, 21 + dst, 11, 30 + dst, 29, 2);
        for i in 0..4 {
            self.hud_draw_item(
                0x1000,
                hudxy(25 + dst, 13 + i * 4),
                &K_HUD_ITEM_BOTTLES[self.ram[LINK_BOTTLE_INFO + i] as usize],
            );
        }
        let bottle_index = (self.ram[LINK_ITEM_BOTTLE_INDEX] as usize).wrapping_sub(1);
        let p = K_HUD_ITEM_BOTTLES[self.ram[LINK_BOTTLE_INFO + bottle_index] as usize];
        self.hud_draw_item(0x1000, K_HUD_ITEM_IN_VRAM_PTR_OLD[15], &p);
        if self.ram[TIMER_FOR_FLASHING_CIRCLE] & 0x10 != 0 {
            self.hud_draw_flashing_circle(0x1000, hudxy(25 + dst, 13 + bottle_index * 4) as i32, 7);
        }
    }

    pub(super) fn hud_refill_health(&mut self) -> bool {
        if self.ram[LINK_HEALTH_CURRENT] >= self.ram[LINK_HEALTH_CAPACITY] {
            self.ram[LINK_HEALTH_CURRENT] = self.ram[LINK_HEALTH_CAPACITY];
            self.ram[LINK_HEARTS_FILLER] = 0;
            return self.ram[IS_DOING_HEART_ANIMATION] == 0;
        }
        self.ram[LINK_HEARTS_FILLER] = 160;
        false
    }

    pub(super) fn hud_animate_heart_refill(&mut self) {
        self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] =
            self.ram[ANIMATE_HEART_REFILL_COUNTDOWN].wrapping_sub(1);
        if self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] != 0 {
            return;
        }
        let mut n =
            (((u16::from(self.ram[LINK_HEALTH_CURRENT] & !7)).wrapping_sub(1) >> 3) << 1) as usize;
        let mut p = hudxy(20, 1);
        if n >= 20 {
            n -= 20;
            p += 0x20;
        }
        n &= 0xff;
        self.ram[ANIMATE_HEART_REFILL_COUNTDOWN] = 1;
        const K_ANIM_HEART_PARTIAL: [u16; 4] = [0x24a3, 0x24a4, 0x24a3, 0x24a0];
        self.hud_buffer_set(
            p + (n >> 1),
            K_ANIM_HEART_PARTIAL[self.ram[ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS] as usize],
        );
        self.ram[ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS] =
            self.ram[ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS].wrapping_add(1) & 3;
        if self.ram[ANIMATE_HEART_REFILL_COUNTDOWN_SUBPOS] == 0 {
            self.hud_rebuild();
            self.ram[IS_DOING_HEART_ANIMATION] = 0;
        }
    }

    pub(super) fn hud_refill_magic_power(&mut self) -> bool {
        if self.ram[LINK_MAGIC_POWER] >= 0x80 {
            return true;
        }
        self.ram[LINK_MAGIC_FILLER] = 0x80;
        false
    }

    pub(super) fn hud_restore_torch_background(&mut self) {
        if self.ram[LINK_ITEM_TORCH] == 0
            || self.ram[DUNG_WANT_LIGHTS_OUT] == 0
            || self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] != 0
            || self.ram[DUNG_NUM_LIT_TORCHES] != 0
        {
            return;
        }
        self.ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
        if self.ram[DUNG_HDR_BG2_PROPERTIES] != 2 {
            self.ram[TS_COPY] = 1;
        }
    }

    pub(super) fn hud_rebuild_indoor(&mut self) {
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = 0;
        self.ram[LINK_NUM_KEYS] = 0xff;
        self.hud_rebuild();
    }

    pub(super) fn hud_rebuild(&mut self) {
        if read_le_u16(&self.ram, HUD_TILE_INDICES_BUFFER + hudxy(8, 2) * 2) == 0 {
            for i in 0..165 {
                self.hud_buffer_set(i, 0x207f);
            }
        }
        self.draw_hud_components(HUD_TILE_INDICES_BUFFER, 0, &HUD_TILEMAP_LEFT_PART, 8, 6);
        self.draw_hud_components(
            HUD_TILE_INDICES_BUFFER,
            hudxy(20, 0),
            &HUD_TILEMAP_RIGHT_PART,
            12,
            5,
        );
        self.hud_update_hearts();
        self.hud_update_magic();
        self.hud_update_inventory();
        self.hud_update_item_box();
        self.ram[FLAG_UPDATE_HUD_IN_NMI] = self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
    }

    pub(super) fn hud_get_item_box_table(&self, item: u8) -> &'static [ItemBoxGfx] {
        Self::hud_item_box_gfx_ptr(item)
    }

    pub(super) fn hud_get_item_box_ptr(&self, item: u8) -> &'static ItemBoxGfx {
        &Self::hud_item_box_gfx_ptr(item)[0]
    }

    pub(super) fn hud_handle_item_switch_inputs(&mut self) {
        if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_SWITCH_LR == 0 {
            return;
        }
        let direction = if self.ram[FILTERED_JOYPAD_L] & K_JOYPAD_L_L != 0
            && self.ram[HUD_CUR_ITEM_L] == 0
        {
            self.ram[HUD_CUR_ITEM_R] != 0
        } else if self.ram[FILTERED_JOYPAD_L] & K_JOYPAD_L_R != 0 && self.ram[HUD_CUR_ITEM_R] == 0 {
            true
        } else {
            return;
        };

        let mut item = self.ram[HUD_CUR_ITEM];
        for _ in 0..K_HUD_ITEM_COUNT {
            if !direction {
                self.hud_goto_prev_item(&mut item, 1);
            } else {
                self.hud_goto_next_item(&mut item, 1);
            }
            if self.hud_do_we_have_this_item(item)
                && (self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_SWITCH_LR_LIMIT == 0
                    || self.hud_get_item_position(item) <= 3)
            {
                if item != self.ram[HUD_CUR_ITEM] {
                    self.ram[HUD_CUR_ITEM] = item;
                    self.ram[SOUND_EFFECT_2] = 32;
                    self.hud_update_equipped_item();
                    self.hud_update_item_box();
                    self.ram[FLAG_UPDATE_HUD_IN_NMI] =
                        self.ram[FLAG_UPDATE_HUD_IN_NMI].wrapping_add(1);
                }
                break;
            }
        }
    }

    fn hud_reorder_item(&mut self, direction: i32) {
        if self.ram[HUD_INVENTORY_ORDER] == 0 {
            for i in 0..24 {
                self.ram[HUD_INVENTORY_ORDER + i] = i as u8 + 1;
            }
        }
        let old_pos = self.hud_get_item_position(self.ram[HUD_CUR_ITEM]);
        let mut new_pos = old_pos + direction;
        if new_pos < 0 {
            new_pos += K_HUD_ITEM_COUNT as i32;
        } else if new_pos >= K_HUD_ITEM_COUNT as i32 {
            new_pos -= K_HUD_ITEM_COUNT as i32;
        }
        let old = HUD_INVENTORY_ORDER + old_pos as usize;
        let new = HUD_INVENTORY_ORDER + new_pos as usize;
        self.ram.swap(old, new);
        self.hud_draw_y_button_items();
        self.ram[SOUND_EFFECT_2] = 32;
    }

    fn hud_update_item_box(&mut self) {
        if self.ram[HUD_CUR_ITEM] != 0 {
            let icon = self.hud_get_icon_for_item(self.ram[HUD_CUR_ITEM]);
            self.hud_draw_item(HUD_TILE_INDICES_BUFFER, hudxy(5, 1), &icon);
        }
    }

    fn hud_update_hearts(&mut self) {
        const FULL: [u16; 3] = [0x24a2, 0x24a2, 0x24a2];
        const CURRENT: [u16; 3] = [0x24a2, 0x24a1, 0x24a0];
        self.hud_update_hearts_inner(hudxy(20, 1), &FULL, self.ram[LINK_HEALTH_CAPACITY] as i32);
        self.hud_update_hearts_inner(
            hudxy(20, 1),
            &CURRENT,
            (i32::from(self.ram[LINK_HEALTH_CURRENT]) + 3) & !3,
        );
    }

    fn hud_update_magic(&mut self) {
        let dst = hudxy(2, 0);
        if self.ram[LINK_MAGIC_CONSUMPTION] >= 1 {
            self.hud_buffer_set(dst + hudxy(0, 0), 0x28f7);
            self.hud_buffer_set(dst + hudxy(1, 0), 0x2851);
            self.hud_buffer_set(dst + hudxy(2, 0), 0x28fa);
        }
        let src = K_UPDATE_MAGIC_POWER_TILEMAP[(usize::from(self.ram[LINK_MAGIC_POWER]) + 7) >> 3];
        for (y, tile) in src.iter().enumerate() {
            self.hud_buffer_set(dst + hudxy(1, y + 1), *tile);
        }
    }

    fn hud_update_inventory(&mut self) {
        const K_HUD_INVENTORY_BG: [u16; 26] = [
            0x207f, 0x207f, 0x3ca8, 0x207f, 0x207f, 0x2c88, 0x2c89, 0x207f, 0x20a7, 0x20a9, 0x207f,
            0x2871, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f,
        ];
        let d = hud_int_to_decimal(read_le_u16(&self.ram, LINK_RUPEES_ACTUAL) as u32);
        let inv_offs = usize::from(d[0] == 0x90);
        let dst = hudxy(8, 0);
        for i in 0..12 {
            self.hud_buffer_set(dst + hudxy(i, 0), K_HUD_INVENTORY_BG[inv_offs + i]);
            self.hud_buffer_set(dst + hudxy(i, 1), K_HUD_INVENTORY_BG[13 + inv_offs + i]);
        }
        if self.ram[LINK_ITEM_BOW] != 0 {
            if self.ram[LINK_ITEM_BOW] >= 3 {
                self.hud_buffer_set(hudxy(15, 0), 0x2486);
                self.hud_buffer_set(hudxy(16, 0), 0x2487);
                self.ram[LINK_ITEM_BOW] = if self.ram[LINK_NUM_ARROWS] != 0 { 4 } else { 3 };
            } else {
                self.ram[LINK_ITEM_BOW] = if self.ram[LINK_NUM_ARROWS] != 0 { 2 } else { 1 };
            }
        }

        let base_tiles = [
            0x2400,
            if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_SHOW_MAX_ITEMS_IN_YELLOW != 0 {
                0x3400
            } else {
                0x2400
            },
        ];
        let base_tile = base_tiles
            [usize::from(read_le_u16(&self.ram, LINK_RUPEES_ACTUAL) == self.max_rupees())];
        let digit_x = usize::from(inv_offs == 0);
        if inv_offs == 0 {
            self.hud_buffer_set(dst + hudxy(0, 1), base_tile | d[0] as u16);
        }
        self.hud_buffer_set(dst + hudxy(digit_x, 1), base_tile | d[1] as u16);
        self.hud_buffer_set(dst + hudxy(digit_x + 1, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(digit_x + 2, 1), base_tile | d[3] as u16);

        let d = hud_int_to_decimal(self.ram[LINK_ITEM_BOMBS] as u32);
        let base_tile = base_tiles[usize::from(
            self.ram[LINK_ITEM_BOMBS]
                == K_MAX_BOMBS_FOR_LEVEL[self.ram[LINK_BOMB_UPGRADES] as usize],
        )];
        self.hud_buffer_set(dst + hudxy(4, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(5, 1), base_tile | d[3] as u16);

        let d = hud_int_to_decimal(self.ram[LINK_NUM_ARROWS] as u32);
        let base_tile = base_tiles[usize::from(
            self.ram[LINK_NUM_ARROWS]
                == K_MAX_ARROWS_FOR_LEVEL[self.ram[LINK_ARROW_UPGRADES] as usize],
        )];
        self.hud_buffer_set(dst + hudxy(7, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(8, 1), base_tile | d[3] as u16);

        let mut d = [0u8; 4];
        d[3] = 0x7f;
        if self.ram[LINK_NUM_KEYS] != 0xff {
            d = hud_int_to_decimal(self.ram[LINK_NUM_KEYS] as u32);
        }
        let key = 0x2400 | d[3] as u16;
        self.hud_buffer_set(dst + hudxy(10, 1), key);
        if key == 0x247f {
            self.hud_buffer_set(dst + hudxy(10, 0), 0x247f);
        }
    }

    fn hud_update_hearts_inner(&mut self, dst: usize, src: &[u16; 3], mut n: i32) {
        let mut x = 0usize;
        let mut row = 0usize;
        while n > 0 {
            if x >= 10 {
                row += 1;
                x = 0;
            }
            self.hud_buffer_set(dst + hudxy(x, row), src[usize::from(n >= 5) + 1]);
            x += 1;
            n -= 8;
        }
    }

    fn draw_hud_components(&mut self, base: usize, dst: usize, src: &[u16], w: usize, h: usize) {
        self.hud_draw_nx_n(base, dst, src, w, h);
    }

    fn hud_draw_item(&mut self, base: usize, dst: usize, src: &ItemBoxGfx) {
        self.write_tile(base, dst as i32, src[0]);
        self.write_tile(base, dst as i32 + 1, src[1]);
        self.write_tile(base, dst as i32 + 32, src[2]);
        self.write_tile(base, dst as i32 + 33, src[3]);
    }

    fn hud_draw_nx_n(&mut self, base: usize, dst: usize, src: &[u16], w: usize, h: usize) {
        for y in 0..h {
            for x in 0..w {
                self.write_tile(base, (dst + hudxy(x, y)) as i32, src[y * w + x]);
            }
        }
    }

    fn hud_copy2x2(&mut self, dst_base: usize, dst: usize, src_base: usize, src: usize) {
        let v0 = self.read_tile(src_base, src as i32);
        let v1 = self.read_tile(src_base, src as i32 + 1);
        let v2 = self.read_tile(src_base, src as i32 + 32);
        let v3 = self.read_tile(src_base, src as i32 + 33);
        self.write_tile(dst_base, dst as i32, v0);
        self.write_tile(dst_base, dst as i32 + 1, v1);
        self.write_tile(dst_base, dst as i32 + 32, v2);
        self.write_tile(dst_base, dst as i32 + 33, v3);
    }

    fn hud_draw_box(
        &mut self,
        base: usize,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        palette: u8,
    ) {
        let mut t = 0x20fb | ((palette as u16) << 10);
        self.write_tile(base, hudxy(x1, y1) as i32, t);
        self.write_tile(base, hudxy(x2, y1) as i32, t + 0x4000);
        self.write_tile(base, hudxy(x1, y2) as i32, t + 0x8000);
        self.write_tile(base, hudxy(x2, y2) as i32, t + 0xc000);

        t = 0x20fc | ((palette as u16) << 10);
        for y in y1 + 1..y2 {
            self.write_tile(base, hudxy(x1, y) as i32, t);
            self.write_tile(base, hudxy(x2, y) as i32, t + 0x4000);
        }

        t = 0x20f9 | ((palette as u16) << 10);
        for x in x1 + 1..x2 {
            self.write_tile(base, hudxy(x, y1) as i32, t);
            self.write_tile(base, hudxy(x, y2) as i32, t + 0x8000);
        }

        for y in y1 + 1..y2 {
            for x in x1 + 1..x2 {
                self.write_tile(base, hudxy(x, y) as i32, 0x24f5);
            }
        }
    }

    fn hud_draw_flashing_circle(&mut self, base: usize, p: i32, palette: u8) {
        let pp = (palette as u16) << 10;
        for (offs, tile) in [
            (hudxyi(0, -1), pp | 0x2061),
            (hudxyi(1, -1), pp | 0x2061 | 0x4000),
            (hudxyi(-1, 0), pp | 0x2070),
            (hudxyi(2, 0), pp | 0x2070 | 0x4000),
            (hudxyi(-1, 1), pp | 0xa070),
            (hudxyi(2, 1), pp | 0xa070 | 0x4000),
            (hudxyi(0, 2), pp | 0xa061),
            (hudxyi(1, 2), pp | 0xa061 | 0x4000),
            (hudxyi(-1, -1), pp | 0x2060),
            (hudxyi(2, -1), pp | 0x2060 | 0x4000),
            (hudxyi(2, 2), pp | 0x2060 | 0xc000),
            (hudxyi(-1, 2), pp | 0x2060 | 0x8000),
        ] {
            self.write_tile(base, p + offs, tile);
        }
    }

    fn menu_set(&mut self, tile: usize, value: u16) {
        self.write_tile(0x1000, tile as i32, value);
    }

    fn hud_buffer_set(&mut self, tile: usize, value: u16) {
        self.write_tile(HUD_TILE_INDICES_BUFFER, tile as i32, value);
    }

    fn write_tile(&mut self, base: usize, tile: i32, value: u16) {
        let addr = (base as i32 + tile * 2) as usize;
        write_le_u16(&mut self.ram, addr, value);
    }

    fn read_tile(&self, base: usize, tile: i32) -> u16 {
        let addr = (base as i32 + tile * 2) as usize;
        read_le_u16(&self.ram, addr)
    }

    fn hud_item_box_gfx_ptr(item: u8) -> &'static [ItemBoxGfx] {
        match item {
            0 => &K_HUD_ITEM_BOW,
            1 => &K_HUD_ITEM_BOOMERANG,
            2 => &K_HUD_ITEM_HOOKSHOT,
            3 => &K_HUD_ITEM_BOMBS,
            4 => &K_HUD_ITEM_MUSHROOM,
            5 => &K_HUD_ITEM_FIRE_ROD,
            6 => &K_HUD_ITEM_ICE_ROD,
            7 => &K_HUD_ITEM_BOMBOS,
            8 => &K_HUD_ITEM_ETHER,
            9 => &K_HUD_ITEM_QUAKE,
            10 => &K_HUD_ITEM_TORCH,
            11 => &K_HUD_ITEM_HAMMER,
            12 => &K_HUD_ITEM_FLUTE_GFX,
            13 => &K_HUD_ITEM_BUG_NET,
            14 => &K_HUD_ITEM_BOOK_MUDORA,
            15 => &K_HUD_ITEM_BOTTLES,
            16 => &K_HUD_ITEM_CANE_SOMARIA,
            17 => &K_HUD_ITEM_CANE_BYRNA,
            18 => &K_HUD_ITEM_CAPE,
            19 => &K_HUD_ITEM_MIRROR,
            20 => &K_HUD_ITEM_GLOVES,
            21 => &K_HUD_ITEM_BOOTS,
            22 => &K_HUD_ITEM_FLIPPERS,
            23 => &K_HUD_ITEM_MOON_PEARL,
            24 => &K_HUD_ITEM_EMPTY,
            25 => &K_HUD_ITEM_SWORD,
            26 => &K_HUD_ITEM_SHIELD,
            27 => &K_HUD_ITEM_ARMOR,
            28..=31 => &K_HUD_ITEM_BOTTLES,
            // C indexes kHudItemBoxGfxPtrs directly; this is the Rust bounds
            // guard for that table.
            _ => panic!("invalid Hud_GetItemBoxPtr item {item}"),
        }
    }
}

const fn hudxy(x: usize, y: usize) -> usize {
    y * 32 + x
}

const fn hudxyi(x: i32, y: i32) -> i32 {
    y * 32 + x
}

const fn pv(a: [u16; 8]) -> u16 {
    ((a[0] & 1) << 7)
        | (((a[0] >> 1) & 1) << 15)
        | ((a[1] & 1) << 6)
        | (((a[1] >> 1) & 1) << 14)
        | ((a[2] & 1) << 5)
        | (((a[2] >> 1) & 1) << 13)
        | ((a[3] & 1) << 4)
        | (((a[3] >> 1) & 1) << 12)
        | ((a[4] & 1) << 3)
        | (((a[4] >> 1) & 1) << 11)
        | ((a[5] & 1) << 2)
        | (((a[5] >> 1) & 1) << 10)
        | ((a[6] & 1) << 1)
        | (((a[6] >> 1) & 1) << 9)
        | (a[7] & 1)
        | (((a[7] >> 1) & 1) << 8)
}

fn hud_int_to_decimal(number: u32) -> [u8; 4] {
    [
        (number / 1000) as u8 + 0x90,
        ((number % 1000) / 100) as u8 + 0x90,
        ((number % 100) / 10) as u8 + 0x90,
        (number % 10) as u8 + 0x90,
    ]
}

const K_SWITCH_LR_PALETTES: [u8; 4] = [7, 3, 4, 4];

const K_HUD_BOTTLES_ITEM_TEXT: [u16; 128] = [
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562, 0x2557,
    0x2561, 0x255e, 0x255e, 0x255c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x2551, 0x255e, 0x2563, 0x2563, 0x255b, 0x2554, 0x24f5, 0x24f5, 0x255b, 0x2558, 0x2555, 0x2554,
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2554, 0x2553, 0x2558, 0x2552, 0x2558, 0x255d, 0x2554,
    0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2554, 0x2553, 0x2558,
    0x2552, 0x2558, 0x255d, 0x2554, 0x2552, 0x2564, 0x2561, 0x2554, 0x256a, 0x2550, 0x255b, 0x255b,
    0x255c, 0x2554, 0x2553, 0x2558, 0x2552, 0x2558, 0x255d, 0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2555, 0x2550, 0x2554, 0x2561, 0x2558, 0x2554, 0x24f5, 0x24f5,
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x2554, 0x2554, 0x24f5,
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x2556, 0x255e, 0x255e, 0x2553, 0x24f5, 0x2551, 0x2554, 0x2554,
];

const K_HUD_MUSHROOM_ITEM_TEXT: [u16; 16] = [
    0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255f, 0x255e, 0x2566,
    0x2553, 0x2554, 0x2561, 0x24f5,
];

const K_HUD_FLUTE_ITEM_TEXT: [u16; 32] = [
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2555, 0x255b, 0x2564, 0x2563,
    0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x2555, 0x255b, 0x2564, 0x2563, 0x2554, 0x24f5, 0x24f5, 0x24f5,
];

const K_HUD_MIRROR_ITEM_TEXT: [u16; 16] = [
    0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2558,
    0x2561, 0x2561, 0x255e, 0x2561,
];

const K_HUD_BOW_ITEM_TEXT: [u16; 48] = [
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x256e, 0x256f,
    0x257c, 0x257d, 0x257e, 0x257f, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x256b, 0x256c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x24f5, 0x256e,
    0x256f, 0x24f5, 0x24f5, 0x24f5, 0x2578, 0x2579, 0x257a, 0x257b, 0x257c, 0x257d, 0x257e, 0x257f,
];

const K_NOT_ASSIGNED_ITEM_TEXT: [u16; 16] = [
    0x255d, 0x255e, 0x2563, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2550, 0x2562, 0x2562, 0x2558,
    0x2556, 0x255d, 0x2554, 0x2553,
];

const K_HUD_ITEM_TEXT: [[u16; 16]; 20] = [
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x24f5,
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2570, 0x2571, 0x2572,
        0x2573, 0x2574, 0x2575, 0x2576, 0x2577,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2557, 0x255e, 0x255e,
        0x255a, 0x2562, 0x2557, 0x255e, 0x2563,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x255e, 0x255c,
        0x2551, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562,
        0x2557, 0x2561, 0x255e, 0x255e, 0x255c,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2555, 0x2558, 0x2561,
        0x2554, 0x2561, 0x255e, 0x2553, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2558, 0x2552, 0x2554,
        0x2561, 0x255e, 0x2553, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2551, 0x255e, 0x255c,
        0x2551, 0x255e, 0x2562, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2554, 0x2563, 0x2557,
        0x2554, 0x2561, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2560, 0x2564, 0x2550,
        0x255a, 0x2554, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255b, 0x2550, 0x255c,
        0x255f, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2557,
        0x2550, 0x255c, 0x255c, 0x2554, 0x2561,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2562, 0x2557, 0x255e,
        0x2565, 0x2554, 0x255b, 0x24f5, 0x24f5,
    ],
    [
        0x2400, 0x2401, 0x2402, 0x2403, 0x2404, 0x2405, 0x2406, 0x2407, 0x2408, 0x2409, 0x24f5,
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    ],
    [
        0x2551, 0x255e, 0x255e, 0x255a, 0x24f5, 0x255e, 0x2555, 0x24f5, 0x255c, 0x2564, 0x2553,
        0x255e, 0x2561, 0x2550, 0x24f5, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2564, 0x2562,
        0x2557, 0x2561, 0x255e, 0x255e, 0x255c,
    ],
    [
        0x2552, 0x2550, 0x255d, 0x2554, 0x24f5, 0x255e, 0x2555, 0x24f5, 0x24f5, 0x2562, 0x255e,
        0x255c, 0x2550, 0x2561, 0x2558, 0x2550,
    ],
    [
        0x2552, 0x2550, 0x255d, 0x2554, 0x24f5, 0x255e, 0x2555, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
        0x2551, 0x2568, 0x2561, 0x255d, 0x2550,
    ],
    [
        0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
        0x2552, 0x2550, 0x255f, 0x2554, 0x24f5,
    ],
    [
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
        0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    ],
];

const HUD_TILEMAP_LEFT_PART: [u16; 8 * 6] = [
    0x207f, 0x207f, 0x2850, 0xa856, 0x2852, 0x285b, 0x285b, 0x285c, 0x207f, 0x207f, 0x2854, 0x2871,
    0x2858, 0x207f, 0x207f, 0x285d, 0x207f, 0x207f, 0x2854, 0x304e, 0x2858, 0x207f, 0x207f, 0x285d,
    0x207f, 0x207f, 0x2854, 0x305e, 0x2859, 0xa85b, 0xa85b, 0xa85c, 0x207f, 0x207f, 0x2854, 0x305e,
    0x6854, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0xa850, 0x2856, 0xe850, 0x0000, 0x0000, 0x0000,
];

const HUD_TILEMAP_RIGHT_PART: [u16; 12 * 5] = [
    0x207f, 0x207f, 0x288b, 0x288f, 0x24ab, 0x24ac, 0x688f, 0x688b, 0x207f, 0x207f, 0x207f, 0x207f,
    0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
    0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
    0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
    0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
];
