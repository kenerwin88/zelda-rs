#![allow(non_snake_case)]

// Methods ported from zelda3/src/hud.c and included inside ZeldaState.

use super::*;

const USE_NEW_STYLE_INVENTORY: bool = false;
const HUD_ITEM_COUNT: usize = if USE_NEW_STYLE_INVENTORY { 24 } else { 20 };

const FEATURE_SWITCH_LR: u32 = 2;
const FEATURE_DISABLE_LOW_HEALTH_BEEP: u32 = 64;
const FEATURE_SHOW_MAX_ITEMS_IN_YELLOW: u32 = 256;
const FEATURE_CARRY_MORE_RUPEES: u32 = 2048;
const FEATURE_SWITCH_LR_LIMIT: u32 = 32768;

const JOYPAD_HIGH_START: u8 = 0x10;
const JOYPAD_HIGH_SELECT: u8 = 0x20;
const JOYPAD_HIGH_Y: u8 = 0x40;
const JOYPAD_HIGH_UP: u8 = 0x08;
const JOYPAD_HIGH_DOWN: u8 = 0x04;
const JOYPAD_HIGH_LEFT: u8 = 0x02;
const JOYPAD_HIGH_RIGHT: u8 = 0x01;
const JOYPAD_LOW_X: u8 = 0x40;
const JOYPAD_LOW_L: u8 = 0x20;
const JOYPAD_LOW_R: u8 = 0x10;

const HUD_ITEM_BOTTLE_LEGACY: u8 = 16;
const HUD_ITEM_BOTTLE_FIRST: u8 = 21;
const HUD_ITEM_BOTTLE_LAST: u8 = HUD_ITEM_BOTTLE_FIRST + 3;
const HUD_ITEM_FLUTE: u8 = 13;
const HUD_ITEM_SHOVEL: u8 = 16;

type ItemBoxGfx = [u16; 4];

const MAX_BOMBS_BY_UPGRADE_LEVEL: [u8; 8] = [10, 15, 20, 25, 30, 35, 40, 50];
const MAX_ARROWS_BY_UPGRADE_LEVEL: [u8; 8] = [30, 35, 40, 45, 50, 55, 60, 70];
const MAX_HEALTH_BY_CAPACITY_LEVEL: [u8; 21] = [
    9, 9, 9, 9, 9, 9, 9, 9, 17, 17, 17, 17, 17, 17, 17, 25, 25, 25, 25, 25, 25,
];

const HUD_ITEM_VRAM_POSITIONS_LEGACY: [usize; 20] = [
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

const HUD_ITEM_EMPTY_GRAPHICS: [ItemBoxGfx; 1] = [[0x20f5, 0x20f5, 0x20f5, 0x20f5]];
const HUD_ITEM_BOTTLE_GRAPHICS: [ItemBoxGfx; 9] = [
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
const HUD_ITEM_BOW_GRAPHICS: [ItemBoxGfx; 5] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x28ba, 0x28e9, 0x28e8, 0x28cb],
    [0x28ba, 0x284a, 0x2849, 0x28cb],
    [0x28ba, 0x28e9, 0x28e8, 0x28cb],
    [0x28ba, 0x28bb, 0x24ca, 0x28cb],
];
const HUD_ITEM_BOOMERANG_GRAPHICS: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb8, 0x2cb9, 0x2cf5, 0x2cc9],
    [0x24b8, 0x24b9, 0x24f5, 0x24c9],
];
const HUD_ITEM_HOOKSHOT_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24f5, 0x24f6, 0x24c0, 0x24f5],
];
const HUD_ITEM_BOMBS_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb2, 0x2cb3, 0x2cc2, 0x6cc2],
];
const HUD_ITEM_MUSHROOM_GRAPHICS: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2444, 0x2445, 0x2446, 0x2447],
    [0x203b, 0x203c, 0x203d, 0x203e],
];
const HUD_ITEM_FIRE_ROD_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24b0, 0x24b1, 0x24c0, 0x24c1],
];
const HUD_ITEM_ICE_ROD_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cb0, 0x2cbe, 0x2cc0, 0x2cc1],
];
const HUD_ITEM_BOMBOS_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x287d, 0x287e, 0xe87e, 0xe87d],
];
const HUD_ITEM_ETHER_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2876, 0x2877, 0xe877, 0xe876],
];
const HUD_ITEM_QUAKE_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2866, 0x2867, 0xe867, 0xe866],
];
const HUD_ITEM_TORCH_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24bc, 0x24bd, 0x24cc, 0x24cd],
];
const HUD_ITEM_HAMMER_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x20b6, 0x20b7, 0x20c6, 0x20c7],
];
const HUD_ITEM_FLUTE_GRAPHICS: [ItemBoxGfx; 4] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x20d0, 0x20d1, 0x20e0, 0x20e1],
    [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
    [0x2cd4, 0x2cd5, 0x2ce4, 0x2ce5],
];
const HUD_ITEM_BUG_NET_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3c40, 0x3c41, 0x2842, 0x3c43],
];
const HUD_ITEM_BOOK_MUDORA_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3ca5, 0x3ca6, 0x3cd8, 0x3cd9],
];
const HUD_ITEM_CANE_SOMARIA_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24dc, 0x24dd, 0x24ec, 0x24ed],
];
const HUD_ITEM_CANE_BYRNA_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2cdc, 0x2cdd, 0x2cec, 0x2ced],
];
const HUD_ITEM_CAPE_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x24b4, 0x24b5, 0x24c4, 0x24c5],
];
const HUD_ITEM_MIRROR_GRAPHICS: [ItemBoxGfx; 4] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x28de, 0x28df, 0x28ee, 0x28ef],
    [0x2c62, 0x2c63, 0x2c72, 0x2c73],
    [0x2886, 0x2887, 0x2888, 0x2889],
];
const HUD_ITEM_GLOVES_GRAPHICS: [ItemBoxGfx; 3] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2130, 0x2131, 0x2140, 0x2141],
    [0x28da, 0x28db, 0x28ea, 0x28eb],
];
const HUD_ITEM_BOOTS_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x3429, 0x342a, 0x342b, 0x342c],
];
const HUD_ITEM_FLIPPERS_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2c9a, 0x2c9b, 0x2c9d, 0x2c9e],
];
const HUD_ITEM_MOON_PEARL_GRAPHICS: [ItemBoxGfx; 2] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2433, 0x2434, 0x2435, 0x2436],
];
const HUD_ITEM_SWORD_GRAPHICS: [ItemBoxGfx; 5] = [
    [0x20f5, 0x20f5, 0x20f5, 0x20f5],
    [0x2c64, 0x2cce, 0x2c75, 0x3d25],
    [0x2c8a, 0x2c65, 0x2474, 0x3d26],
    [0x248a, 0x2465, 0x3c74, 0x2d48],
    [0x288a, 0x2865, 0x2c74, 0x2d39],
];
const HUD_ITEM_SHIELD_GRAPHICS: [ItemBoxGfx; 4] = [
    [0x24f5, 0x24f5, 0x24f5, 0x24f5],
    [0x2cfd, 0x6cfd, 0x2cfe, 0x6cfe],
    [0x34ff, 0x74ff, 0x349f, 0x749f],
    [0x2880, 0x2881, 0x288d, 0x288e],
];
const HUD_ITEM_ARMOR_GRAPHICS: [ItemBoxGfx; 5] = [
    [0x3c68, 0x7c68, 0x3c78, 0x7c78],
    [0x2c68, 0x6c68, 0x2c78, 0x6c78],
    [0x2468, 0x6468, 0x2478, 0x6478],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

const MAGIC_METER_TILEMAP_BY_LEVEL: [[u16; 4]; 17] = [
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
const DUNGEON_FLOOR_INDICATOR_TOP_TILES: [u16; 11] = [
    0x2508, 0x2509, 0x2509, 0x250a, 0x250b, 0x250c, 0x250d, 0x251d, 0xe51c, 0x250e, 0x7f,
];
const DUNGEON_FLOOR_INDICATOR_BOTTOM_TILES: [u16; 11] = [
    0x2518, 0x2519, 0xa509, 0x251a, 0x251b, 0x251c, 0x2518, 0xa51d, 0xe50c, 0xa50e, 0x7f,
];

impl ZeldaState {
    pub(super) fn hud_refresh_icon(&mut self) {
        self.hud_search_for_equipped_item();
        self.hud_update_hud();
        self.hud_rebuild();
        self.set_overworld_map_state(0);
    }

    pub(super) fn check_palace_item_posession(&self) -> u8 {
        let inventory = self.inventory_items();
        match self.save_progress_view().palace_index_x2() >> 1 {
            2 => u8::from(inventory.bow() != 0),
            3 => u8::from(inventory.gloves() != 0),
            5 => u8::from(inventory.hookshot() != 0),
            6 => u8::from(inventory.hammer() != 0),
            7 => u8::from(inventory.cane_somaria() != 0),
            8 => u8::from(inventory.fire_rod() != 0),
            9 => u8::from(inventory.armor() != 0),
            10 => u8::from(inventory.moon_pearl() != 0),
            11 => u8::from(inventory.gloves() != 1),
            12 => u8::from(inventory.shield_type() == 3),
            13 => u8::from(inventory.armor() == 2),
            _ => 0,
        }
    }

    fn hud_get_item_position(&self, item: u8) -> i32 {
        if item == 0 {
            return -1;
        }
        if self.hud_inventory_order_state().is_custom() {
            for i in 0..HUD_ITEM_COUNT - 1 {
                if self.hud_inventory_order_state().item(i) == item {
                    return i as i32;
                }
            }
            (HUD_ITEM_COUNT - 1) as i32
        } else {
            i32::from(item) - 1
        }
    }

    fn hud_goto_prev_item(&self, item: &mut u8, first_item_index: u8) {
        if self.hud_inventory_order_state().is_custom() {
            let pos = self.hud_get_item_position(*item);
            *item = if pos == 0 && first_item_index == 0 {
                0
            } else {
                let idx = ((if pos <= 0 { HUD_ITEM_COUNT as i32 } else { pos }) - 1) as usize;
                self.hud_inventory_order_state().item(idx)
            };
        } else {
            *item = if *item > first_item_index {
                item.wrapping_sub(1)
            } else {
                HUD_ITEM_COUNT as u8
            };
        }
    }

    fn hud_goto_next_item(&self, item: &mut u8, first_item_index: u8) {
        if self.hud_inventory_order_state().is_custom() {
            let i = self.hud_get_item_position(*item);
            let idx = if i as usize >= HUD_ITEM_COUNT - 1 {
                0
            } else {
                i as usize + 1
            };
            *item = self.hud_inventory_order_state().item(idx);
        } else {
            *item = if *item < HUD_ITEM_COUNT as u8 {
                item.wrapping_add(1)
            } else {
                first_item_index
            };
        }
    }

    pub(super) fn hud_floor_indicator(&mut self) {
        let mut a = u16::from(self.hud_state_view().floor_changed_timer_low());
        if a == 0 {
            self.hud_remove_super_bomb_indicator();
            return;
        }
        a = a.wrapping_add(1);
        if a == 0xc0 {
            a = 0;
        }
        self.hud_state_view_mut().set_floor_changed_timer(a);

        self.hud_buffer_set(0xf2 / 2, 0x251e);
        self.hud_buffer_set(0x134 / 2, 0x251f);
        self.hud_buffer_set(0x132 / 2, 0x2520);
        self.hud_buffer_set(0xf4 / 2, 0x250f);

        let mut k = 0usize;
        let j = if (self.dungeon_state_view().current_floor() as i8) >= 0 {
            if self.dungeon_state_view().current_floor_word() == 0
                && self.world_location_state().dungeon_room != 2
                && self.save_progress_view().progress_indicator() < 2
            {
                self.system_signals_view_mut().set_ambient_sound_effect(3);
            }
            self.dungeon_state_view().current_floor()
        } else {
            self.system_signals_view_mut().set_ambient_sound_effect(5);
            k += 1;
            self.dungeon_state_view().current_floor() ^ 0xff
        } as usize;
        self.hud_buffer_set(k + 0xf2 / 2, DUNGEON_FLOOR_INDICATOR_TOP_TILES[j]);
        self.hud_buffer_set(k + 0x132 / 2, DUNGEON_FLOOR_INDICATOR_BOTTOM_TILES[j]);
        self.system_signals_view_mut().increment_hud_update_flag();
    }

    pub(super) fn hud_remove_super_bomb_indicator(&mut self) {
        self.hud_buffer_set(0xf2 / 2, 0x7f);
        self.hud_buffer_set(0x132 / 2, 0x7f);
        self.hud_buffer_set(0xf4 / 2, 0x7f);
        self.hud_buffer_set(0x134 / 2, 0x7f);
    }

    pub(super) fn hud_super_bomb_indicator(&mut self) {
        if self.hud_state_view().super_bomb_indicator_counter() == 0 {
            if (self.hud_state_view().super_bomb_indicator_timer() as i8) < 0 {
                self.hud_state_view_mut()
                    .set_super_bomb_indicator_timer(0xff);
                self.hud_remove_super_bomb_indicator();
                return;
            }
            let t = self
                .hud_state_view()
                .super_bomb_indicator_timer()
                .wrapping_sub(1);
            self.hud_state_view_mut().set_super_bomb_indicator_timer(t);
            self.hud_state_view_mut()
                .set_super_bomb_indicator_counter(62);
        }
        let c = self
            .hud_state_view()
            .super_bomb_indicator_counter()
            .wrapping_sub(1);
        self.hud_state_view_mut()
            .set_super_bomb_indicator_counter(c);
        if (self.hud_state_view().super_bomb_indicator_timer() as i8) < 0 {
            self.hud_state_view_mut()
                .set_super_bomb_indicator_timer(0xff);
            self.hud_remove_super_bomb_indicator();
            return;
        }

        let r = self.hud_state_view().super_bomb_indicator_timer() % 10;
        let q = self.hud_state_view().super_bomb_indicator_timer() / 10;
        let j = if (r.wrapping_sub(1) as i8) < 0 {
            9
        } else {
            r - 1
        } as usize;
        self.hud_buffer_set(0xf4 / 2, DUNGEON_FLOOR_INDICATOR_TOP_TILES[j]);
        self.hud_buffer_set(0x134 / 2, DUNGEON_FLOOR_INDICATOR_BOTTOM_TILES[j]);

        let j = if (q.wrapping_sub(1) as i8) < 0 {
            10
        } else {
            q - 1
        } as usize;
        self.hud_buffer_set(0xf2 / 2, DUNGEON_FLOOR_INDICATOR_TOP_TILES[j]);
        self.hud_buffer_set(0x132 / 2, DUNGEON_FLOOR_INDICATOR_BOTTOM_TILES[j]);
    }

    fn max_rupees(&self) -> u16 {
        if self.enhanced_features_view().has(FEATURE_CARRY_MORE_RUPEES) {
            9999
        } else {
            999
        }
    }

    pub(super) fn hud_refill_logic(&mut self) {
        if self.overworld_map_state() != 0 {
            return;
        }
        if self.player_resources_view().magic_filler() != 0 {
            if self.player_resources_view().magic_power() >= 128 {
                let mut resources = self.player_resources_view_mut();
                resources.set_magic_power(128);
                resources.clear_magic_filler();
            } else {
                {
                    let mut resources = self.player_resources_view_mut();
                    resources.decrement_magic_filler();
                    resources.increment_magic_power();
                }
                if self.frame_state().frame_counter & 3 == 0
                    && !self.system_signals_view().has_sound_effect_1()
                {
                    self.system_signals_view_mut().set_sound_effect_1(45);
                }
            }
        }

        let mut a = self.player_resources_view().rupees_actual();
        let goal = self.player_resources_view().rupees_goal();
        if a != goal {
            if a >= goal {
                a = a.wrapping_sub(1);
                if (a as i16) < 0 {
                    a = 0;
                    self.player_resources_view_mut().set_rupees_goal(0);
                }
            } else {
                a = a.wrapping_add(1);
                let m = self.max_rupees();
                if a > m {
                    a = m;
                    self.player_resources_view_mut().set_rupees_goal(m);
                }
            }
            self.player_resources_view_mut().set_rupees_actual(a);
            if !self.system_signals_view().has_sound_effect_1() {
                let delay = self.hud_state_view().rupee_sfx_sound_delay();
                self.hud_state_view_mut()
                    .set_rupee_sfx_sound_delay(delay.wrapping_add(1));
                if delay & 7 == 0 {
                    self.system_signals_view_mut().set_sound_effect_1(41);
                }
            } else {
                self.hud_state_view_mut().set_rupee_sfx_sound_delay(0);
            }
        } else {
            self.hud_state_view_mut().set_rupee_sfx_sound_delay(0);
        }

        if self.player_resources_view().bomb_filler() != 0 {
            self.player_resources_view_mut().decrement_bomb_filler();
            let max = MAX_BOMBS_BY_UPGRADE_LEVEL
                [self.player_resources_view().bomb_upgrade_level() as usize];
            if self.player_resources_view().bombs() != max {
                self.player_resources_view_mut().increment_bombs();
            }
        }
        if self.player_resources_view().arrow_filler() != 0 {
            self.player_resources_view_mut().decrement_arrow_filler();
            let max = MAX_ARROWS_BY_UPGRADE_LEVEL
                [self.player_resources_view().arrow_upgrade_level() as usize];
            if self.player_resources_view().arrows() != max {
                self.player_resources_view_mut().increment_arrows();
            }
            let bow = self.inventory_items().bow();
            if bow != 0 && bow & 1 == 1 {
                self.inventory_items_mut()
                    .set_inventory_item(0, bow.wrapping_add(1));
                self.hud_refresh_icon();
            }
        }

        let cap_idx = (self.player_resources_view().health_capacity() >> 3) as usize;
        if !self.player_state_view().is_immobilized()
            && self.player_resources_view().heart_filler() == 0
            && self.player_resources_view().current_health() < MAX_HEALTH_BY_CAPACITY_LEVEL[cap_idx]
        {
            if self.player_resources_view().low_health_beep_timer() != 0 {
                self.player_resources_view_mut()
                    .decrement_low_health_beep_timer();
            } else if !self.system_signals_view().has_sound_effect_1() {
                if !self
                    .enhanced_features_view()
                    .has(FEATURE_DISABLE_LOW_HEALTH_BEEP)
                {
                    self.system_signals_view_mut().set_sound_effect_1(43);
                }
                self.player_resources_view_mut()
                    .set_low_health_beep_timer(31);
            }
        }

        if self.hud_state_view().is_doing_heart_animation() {
            self.hud_update_magic();
            self.hud_update_inventory();
            self.hud_animate_heart_refill();
            self.system_signals_view_mut().increment_hud_update_flag();
            return;
        }
        if self.player_resources_view().heart_filler() != 0 {
            if self.player_resources_view().current_health()
                < self.player_resources_view().health_capacity()
            {
                self.player_resources_view_mut()
                    .increment_current_health_by(8);
                if self.player_resources_view().current_health()
                    >= self.player_resources_view().health_capacity()
                {
                    let capacity = self.player_resources_view().health_capacity();
                    self.player_resources_view_mut()
                        .set_current_health(capacity);
                }
                if !self.system_signals_view().has_sound_effect_2() {
                    self.system_signals_view_mut().set_sound_effect_2(13);
                }
                self.player_resources_view_mut()
                    .decrement_heart_filler_by(8);
                let h = self
                    .hud_state_view()
                    .is_doing_heart_animation_raw()
                    .wrapping_add(1);
                self.hud_state_view_mut().set_is_doing_heart_animation(h);
                self.hud_state_view_mut().set_heart_refill_countdown(7);
                self.hud_update_magic();
                self.hud_update_inventory();
                self.hud_animate_heart_refill();
                self.system_signals_view_mut().increment_hud_update_flag();
                return;
            }
            let capacity = self.player_resources_view().health_capacity();
            let mut resources = self.player_resources_view_mut();
            resources.set_current_health(capacity);
            resources.set_heart_filler(0);
        }
        self.hud_update_hearts();
        self.hud_update_magic();
        self.hud_update_inventory();
        self.system_signals_view_mut().increment_hud_update_flag();
    }

    pub(super) fn hud_module_run(&mut self) {
        let tick = self.hud_state_view().tick_counter().wrapping_add(1);
        self.hud_state_view_mut().set_tick_counter(tick);
        match self.overworld_map_state() {
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
            self.write_vram_upload_tilemap_word(i * 2, 0x207f);
        }
        self.system_signals_view_mut().set_sound_effect_2(17);
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
        self.increment_overworld_map_state();
    }

    fn hud_have_any_items(&self) -> bool {
        (0..20).any(|i| self.inventory_items().inventory_item(i) != 0)
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
            while first_bottle < 4 && self.inventory_items().bottle(first_bottle) == 0 {
                first_bottle += 1;
            }
            if first_bottle == 4 {
                self.player_resources_view_mut()
                    .set_equipped_bottle_index(0);
            } else if self.player_resources_view().equipped_bottle_index() == 0 {
                self.player_resources_view_mut()
                    .set_equipped_bottle_index(first_bottle as u8 + 1);
            }

            if self.save_progress_view().hud_current_item() == HUD_ITEM_BOTTLE_LEGACY
                && !USE_NEW_STYLE_INVENTORY
            {
                self.hud_state_view_mut().set_flashing_circle_timer(16);
                self.hud_draw_bottle_menu();
            }
        }

        self.hud_state_view_mut().set_flashing_circle_timer(16);
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
        self.increment_overworld_map_state();
    }

    pub(super) fn hud_bring_menu_down(&mut self) {
        let bg3 = self.ppu_scroll_copy_view().bg3_v_copy2().wrapping_sub(8);
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2(bg3);
        if bg3 == 0xff18 {
            self.increment_overworld_map_state();
        }
    }

    pub(super) fn hud_choose_next_mode(&mut self) {
        if self.hud_have_any_items() {
            self.set_pending_nmi_subroutine(1);
            self.set_nmi_load_target_page(0x22);
            self.hud_draw_selected_y_button_item();
            let overworld_map_state = if self.save_progress_view().hud_current_item()
                == HUD_ITEM_BOTTLE_LEGACY
                && !USE_NEW_STYLE_INVENTORY
            {
                10
            } else {
                4
            };
            self.set_overworld_map_state(overworld_map_state);
        } else if self.player_state_view().filtered_joypad_h() != 0 {
            self.set_overworld_map_state(5);
        }
    }

    fn hud_do_we_have_this_item(&self, item: u8) -> bool {
        if item == 0 {
            return true;
        }
        if item == HUD_ITEM_FLUTE && USE_NEW_STYLE_INVENTORY {
            return self.inventory_items().flute() >= 2;
        }
        if item == HUD_ITEM_SHOVEL && USE_NEW_STYLE_INVENTORY {
            return self.inventory_items().flute() >= 1;
        }
        if item >= HUD_ITEM_BOTTLE_FIRST {
            return self
                .inventory_items()
                .bottle((item - HUD_ITEM_BOTTLE_FIRST) as usize)
                != 0;
        }
        self.inventory_items().inventory_item((item - 1) as usize) != 0
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
            for _ in 0..if USE_NEW_STYLE_INVENTORY { 6 } else { 5 } {
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
        } else if USE_NEW_STYLE_INVENTORY {
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
        if self.enhanced_features_view().has(FEATURE_SWITCH_LR) {
            if self.player_state_view().joypad1l_last() & JOYPAD_LOW_X != 0 {
                1
            } else if self.player_state_view().joypad1l_last() & JOYPAD_LOW_L != 0 {
                2
            } else if self.player_state_view().joypad1l_last() & JOYPAD_LOW_R != 0 {
                3
            } else {
                0
            }
        } else {
            0
        }
    }

    pub(super) fn hud_normal_menu(&mut self) {
        let tc = self
            .hud_state_view()
            .flashing_circle_timer()
            .wrapping_add(1);
        self.hud_state_view_mut().set_flashing_circle_timer(tc);
        if self.player_state_view().joypad1h_last() == 0 {
            self.hud_state_view_mut().clear_prev_joypad_h();
        }

        if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_START != 0 {
            self.set_overworld_map_state(5);
            self.system_signals_view_mut().set_sound_effect_2(18);
            return;
        }

        if self.player_state_view().joypad1h_last() & JOYPAD_HIGH_SELECT != 0
            && self.save_progress_view().progress_indicator() != 0
        {
            self.ppu_scroll_copy_view_mut().set_bg3_v_copy2(0xfff8);
            self.hud_close_menu();
            self.DisplaySelectMenu();
            return;
        }

        if self.player_state_view().joypad1h_last() & JOYPAD_HIGH_Y != 0
            && self.player_state_view().joypad1l_last() & JOYPAD_LOW_X == 0
            && self.enhanced_features_view().has(FEATURE_SWITCH_LR)
        {
            if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_UP != 0 {
                self.hud_reorder_item(if USE_NEW_STYLE_INVENTORY { -6 } else { -5 });
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_DOWN != 0 {
                self.hud_reorder_item(if USE_NEW_STYLE_INVENTORY { 6 } else { 5 });
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_LEFT != 0 {
                self.hud_reorder_item(-1);
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_RIGHT != 0 {
                self.hud_reorder_item(1);
            }
        } else if self.hud_state_view().prev_joypad_h() == 0 {
            let btn_index = self.get_current_item_button_index();
            let mut item = self.inventory_items().equipped_button_item(btn_index);
            let old_item = item;
            if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_UP != 0 {
                self.hud_equip_item_above(&mut item);
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_DOWN != 0 {
                self.hud_equip_item_below(&mut item);
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_LEFT != 0 {
                self.hud_equip_prev_item(&mut item, btn_index == 0);
            } else if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_RIGHT != 0 {
                self.hud_equip_next_item(&mut item, btn_index == 0);
            }
            self.inventory_items_mut()
                .set_equipped_button_item(btn_index, item);
            let jh = self.player_state_view().filtered_joypad_h();
            self.hud_state_view_mut().set_prev_joypad_h(jh);
            if item != old_item {
                self.hud_state_view_mut().set_flashing_circle_timer(16);
                self.system_signals_view_mut().set_sound_effect_2(32);
            }
        }

        self.hud_draw_y_button_items();
        self.hud_draw_selected_y_button_item();
        if self.save_progress_view().hud_current_item() == HUD_ITEM_BOTTLE_LEGACY
            && !USE_NEW_STYLE_INVENTORY
        {
            self.set_overworld_map_state(7);
        }

        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_update_hud(&mut self) {
        self.increment_overworld_map_state();
        self.hud_rebuild();
        self.hud_update_equipped_item();
    }

    pub(super) fn hud_lookup_inventory_item(&self, item: u8) -> u8 {
        const HUD_ITEM_ORDER_TO_INVENTORY_ITEM: [u8; 21] = [
            0, 3, 2, 14, 1, 10, 5, 6, 15, 16, 17, 9, 4, 8, 7, 12, 11, 18, 13, 19, 20,
        ];
        HUD_ITEM_ORDER_TO_INVENTORY_ITEM[item as usize]
    }

    pub(super) fn hud_update_equipped_item(&mut self) {
        if self.save_progress_view().hud_current_item() >= HUD_ITEM_BOTTLE_FIRST {
            let bottle_index =
                self.save_progress_view().hud_current_item() - HUD_ITEM_BOTTLE_FIRST + 1;
            self.player_resources_view_mut()
                .set_equipped_bottle_index(bottle_index);
        }
        assert!(self.save_progress_view().hud_current_item() < 25);
        let ciy = self.hud_lookup_inventory_item(self.save_progress_view().hud_current_item());
        self.player_state_view_mut().set_current_item_y(ciy); // writes CURRENT_ITEM_Y (0x303)
    }

    pub(super) fn hud_close_menu(&mut self) {
        let bg3 = self.ppu_scroll_copy_view().bg3_v_copy2().wrapping_add(8);
        self.ppu_scroll_copy_view_mut().set_bg3_v_copy2(bg3);
        if bg3 != 0 {
            return;
        }
        self.hud_rebuild();
        self.set_overworld_map_state(0);
        self.set_submodule(0);
        let saved_module = self.frame_state().saved_module_for_menu;
        self.set_main_module(saved_module);
        if self.frame_state().submodule != 0 {
            self.hud_restore_torch_background();
        }
        if self.player_state_view().current_item_y() != 5
            && self.player_state_view().current_item_y() != 6
        {
            self.hud_state_view_mut().set_equipment_menu_exit_state(2);
            self.player_state_view_mut().clear_item_debug_value_1();
        } else {
            assert!(self.player_state_view().item_debug_value_1() == 0);
            self.hud_state_view_mut().set_equipment_menu_exit_state(0);
        }
    }

    pub(super) fn hud_goto_bottle_menu(&mut self) {
        self.hud_state_view_mut().set_bottle_menu_row(0);
        self.increment_overworld_map_state();
    }

    pub(super) fn hud_init_bottle_menu(&mut self) {
        let r = self.hud_state_view().bottle_menu_row() as usize;
        for i in 21..=30 {
            self.menu_set(hudxy(i, 11 + r), 0x207f);
        }
        let row = self.hud_state_view().bottle_menu_row().wrapping_add(1);
        self.hud_state_view_mut().set_bottle_menu_row(row);
        if row == 19 {
            self.increment_overworld_map_state();
            self.hud_state_view_mut().set_bottle_menu_row(17);
        }
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_expand_bottle_menu(&mut self) {
        const BOTTLE_MENU_TOP_ROW_TILES: [u16; 10] = [
            0x28fb, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x28f9, 0x68fb,
        ];
        const BOTTLE_MENU_MIDDLE_ROW_TILES: [u16; 10] = [
            0x28fc, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x68fc,
        ];
        const BOTTLE_MENU_BOTTOM_ROW_TILES: [u16; 10] = [
            0xa8fb, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xa8f9, 0xe8fb,
        ];
        let r = self.hud_state_view().bottle_menu_row() as usize;
        self.hud_draw_nx_n(0x1000, hudxy(21, 11 + r), &BOTTLE_MENU_TOP_ROW_TILES, 10, 1);
        self.hud_draw_nx_n(
            0x1000,
            hudxy(21, 12 + r),
            &BOTTLE_MENU_MIDDLE_ROW_TILES,
            10,
            1,
        );
        self.hud_draw_nx_n(0x1000, hudxy(21, 29), &BOTTLE_MENU_BOTTOM_ROW_TILES, 10, 1);
        let row = self.hud_state_view().bottle_menu_row().wrapping_sub(1);
        self.hud_state_view_mut().set_bottle_menu_row(row);
        if (row as i8) < 0 {
            self.increment_overworld_map_state();
        }
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_bottle_menu(&mut self) {
        let tc = self
            .hud_state_view()
            .flashing_circle_timer()
            .wrapping_add(1);
        self.hud_state_view_mut().set_flashing_circle_timer(tc);
        if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_START != 0 {
            self.system_signals_view_mut().set_sound_effect_2(18);
            self.set_overworld_map_state(5);
        } else if self.player_state_view().filtered_joypad_h()
            & (JOYPAD_HIGH_LEFT | JOYPAD_HIGH_RIGHT)
            != 0
        {
            let mut item = self.save_progress_view().hud_current_item();
            if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_LEFT != 0 {
                self.hud_equip_prev_item(&mut item, true);
            } else {
                self.hud_equip_next_item(&mut item, true);
            }
            self.save_progress_view_mut().set_hud_current_item(item);
            self.hud_state_view_mut().set_flashing_circle_timer(16);
            self.system_signals_view_mut().set_sound_effect_2(32);
            self.hud_draw_y_button_items();
            self.hud_draw_selected_y_button_item();
            self.increment_overworld_map_state();
            self.hud_state_view_mut().set_bottle_menu_row(0);
            return;
        }
        self.hud_draw_bottle_menu_update();
        if self.player_state_view().filtered_joypad_h() & (JOYPAD_HIGH_DOWN | JOYPAD_HIGH_UP) != 0 {
            let old_val = self
                .player_resources_view()
                .equipped_bottle_index()
                .wrapping_sub(1)
                & 3;
            let mut val = old_val;
            if self.player_state_view().filtered_joypad_h() & JOYPAD_HIGH_UP != 0 {
                loop {
                    val = val.wrapping_sub(1) & 3;
                    if self.inventory_items().bottle(val as usize) != 0 {
                        break;
                    }
                }
            } else {
                loop {
                    val = val.wrapping_add(1) & 3;
                    if self.inventory_items().bottle(val as usize) != 0 {
                        break;
                    }
                }
            }
            if old_val != val {
                self.player_resources_view_mut()
                    .set_equipped_bottle_index(val + 1);
                self.hud_state_view_mut().set_flashing_circle_timer(16);
                self.system_signals_view_mut().set_sound_effect_2(32);
            }
        }
    }

    pub(super) fn hud_draw_bottle_menu_update(&mut self) {
        self.hud_draw_bottle_menu();
        self.hud_draw_selected_y_button_item();
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_erase_bottle_menu(&mut self) {
        let r = self.hud_state_view().bottle_menu_row() as usize;
        for i in 0..10 {
            self.menu_set(hudxy(21 + i, 11 + r), 0x207f);
        }
        let row = self.hud_state_view().bottle_menu_row().wrapping_add(1);
        self.hud_state_view_mut().set_bottle_menu_row(row);
        if row == 19 {
            self.increment_overworld_map_state();
        }
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_restore_normal_menu(&mut self) {
        self.hud_draw_progress_icons();
        self.hud_draw_equipment_box();
        self.set_overworld_map_state(4);
        self.set_pending_nmi_subroutine(1);
        self.set_nmi_load_target_page(0x22);
    }

    pub(super) fn hud_search_for_equipped_item(&mut self) {
        if !self.hud_have_any_items() {
            self.save_progress_view_mut().set_hud_current_item(0);
            self.save_progress_view_mut()
                .clear_post_message_refresh_flag();
        } else {
            if self.save_progress_view().hud_current_item() == 0 {
                self.save_progress_view_mut().set_hud_current_item(1);
            }
            if !self.hud_do_we_have_this_item(self.save_progress_view().hud_current_item()) {
                let mut item = self.save_progress_view().hud_current_item();
                self.hud_equip_next_item(&mut item, true);
                self.save_progress_view_mut().set_hud_current_item(item);
            }
        }
    }

    fn hud_get_icon_for_item(&self, i: u8) -> ItemBoxGfx {
        if i == 0 {
            return HUD_ITEM_EMPTY_GRAPHICS[0];
        }
        if i >= HUD_ITEM_BOTTLE_FIRST {
            return HUD_ITEM_BOTTLE_GRAPHICS[self
                .inventory_items()
                .bottle((i - HUD_ITEM_BOTTLE_FIRST) as usize)
                as usize];
        }
        let mut item_val = self.inventory_items().inventory_item((i - 1) as usize) as usize;
        if i == 4 {
            item_val = usize::from(item_val != 0);
        } else if i == HUD_ITEM_BOTTLE_LEGACY && !USE_NEW_STYLE_INVENTORY {
            item_val = if self.player_resources_view().equipped_bottle_index() != 0 {
                let bottle_index =
                    self.player_resources_view().equipped_bottle_index() as usize - 1;
                self.inventory_items().bottle(bottle_index) as usize
            } else {
                0
            };
        }
        Self::hud_item_box_gfx_ptr(i - 1)[item_val]
    }

    pub(super) fn hud_draw_y_button_items(&mut self) {
        let x = if USE_NEW_STYLE_INVENTORY { 0 } else { 1 };
        let btn_index = self.get_current_item_button_index();
        self.copy_tiles_for_switch_lr(btn_index);
        self.hud_draw_box(0x1000, x, 5, 20 - x, 19, SWITCH_LR_PALETTES[btn_index]);
        const EQUIPMENT_BUTTON_LETTER_TILES: [[u16; 2]; 4] = [
            [0x3cf0, 0x3cf1],
            [0x2cf0, 0xacf0],
            [0x300e, 0x300f],
            [0x300c, 0x300d],
        ];

        if !USE_NEW_STYLE_INVENTORY {
            self.menu_set(hudxy(2, 6), EQUIPMENT_BUTTON_LETTER_TILES[btn_index][0]);
            self.menu_set(hudxy(2, 7), EQUIPMENT_BUTTON_LETTER_TILES[btn_index][1]);
        }
        self.menu_set(hudxy(x + 2, 5), 0x246e);
        self.menu_set(hudxy(x + 3, 5), 0x246f);

        for i in 0..HUD_ITEM_COUNT {
            let j = self.hud_inventory_order_state().item(i);
            let item = if j == 0 { i as u8 + 1 } else { j };
            let icon = self.hud_get_icon_for_item(item);
            self.hud_draw_item(0x1000, HUD_ITEM_VRAM_POSITIONS_LEGACY[i], &icon);
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
        const HUD_ABILITY_TEXT_TILES: [u16; 80] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d27, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d61, 0x2d54, 0x2d50, 0x2d53, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d63, 0x2d50, 0x2d5b, 0x2d5a, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x2cf5, 0x2cf5, 0x2c2e, 0x2cf5,
            0x2cf5, 0x2d5f, 0x2d64, 0x2d5b, 0x2d5b, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2cf5, 0x2d61, 0x2d64, 0x2d5d, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5,
            0x2d62, 0x2d66, 0x2d58, 0x2d5c, 0x2cf5, 0x2cf5, 0x2cf5, 0x207f, 0x207f, 0x2c01, 0x2c18,
            0x2c28, 0x207f, 0x207f,
        ];
        const HUD_GLOVES_TEXT_TILES: [u16; 20] = [
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d28, 0x2cf5,
            0x2cf5, 0x2cf5, 0x2cf5, 0x2cf5, 0x2d5b, 0x2d58, 0x2d55, 0x2d63, 0x2d29,
        ];
        let x = if USE_NEW_STYLE_INVENTORY { 0 } else { 1 };
        self.hud_draw_box(0x1000, x, 21, 19, 29, 1);

        let mut flags = self.player_resources_view().ability_flags();
        for i in 0..2 {
            for j in 0..3 {
                if flags & 0x80 != 0 {
                    let src = &HUD_ABILITY_TEXT_TILES[i * 40 + j * 10..];
                    self.hud_draw_nx_n(0x1000, hudxy(4 + j * 5, 22 + i * 2), src, 5, 2);
                }
                flags <<= 1;
            }
            flags <<= 1;
        }
        if !USE_NEW_STYLE_INVENTORY {
            self.menu_set(hudxy(2, 22), 0xa4f0);
            self.menu_set(hudxy(2, 23), 0x24f2);
        }
        self.menu_set(hudxy(x + 2, 21), 0x2482);
        self.menu_set(hudxy(x + 3, 21), 0x2483);

        self.hud_draw_item(
            0x1000,
            hudxy(8, 27),
            &HUD_ITEM_GLOVES_GRAPHICS[self.inventory_items().gloves() as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(4, 27),
            &HUD_ITEM_BOOTS_GRAPHICS[self.inventory_items().boots() as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(12, 27),
            &HUD_ITEM_FLIPPERS_GRAPHICS[self.inventory_items().flippers() as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(16, 27),
            &HUD_ITEM_MOON_PEARL_GRAPHICS[self.inventory_items().moon_pearl() as usize],
        );
        if self.inventory_items().gloves() != 0 {
            let src =
                &HUD_GLOVES_TEXT_TILES[usize::from(self.inventory_items().gloves() != 1) * 10..];
            self.hud_draw_nx_n(0x1000, hudxy(4, 22), src, 5, 2);
        }
    }

    pub(super) fn hud_draw_progress_icons(&mut self) {
        if self.save_progress_view().progress_indicator() < 3 {
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
        let dst = if USE_NEW_STYLE_INVENTORY {
            hudxy(22, 11)
        } else {
            hudxy(21, 11)
        };
        self.hud_draw_nx_n(0x1000, dst, &BG, 10, 9);
        let f = self.player_resources_view().pendant_flags();
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
        let dst = if USE_NEW_STYLE_INVENTORY {
            hudxy(22, 11)
        } else {
            hudxy(21, 11)
        };
        self.hud_draw_nx_n(0x1000, dst, &BG, 10, 9);
        let f = self.player_resources_view().crystal_flags();
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
        let dst_box = if USE_NEW_STYLE_INVENTORY { 1 } else { 0 };
        let btn_index = self.get_current_item_button_index();
        let item = self.inventory_items().equipped_button_item(btn_index);
        self.hud_draw_box(
            0x1000,
            21 + dst_box,
            5,
            30 + dst_box,
            10,
            SWITCH_LR_PALETTES[btn_index],
        );
        if item != 0 {
            let pos = self.hud_get_item_position(item);
            if pos >= 0 {
                let src = HUD_ITEM_VRAM_POSITIONS_LEGACY[pos as usize];
                self.hud_copy2x2(0x1000, hudxy(25 + dst_box, 6), 0x1000, src);
                if self.hud_state_view().flashing_circle_timer() & 0x10 != 0 {
                    self.hud_draw_flashing_circle(
                        0x1000,
                        src as i32,
                        SWITCH_LR_PALETTES[btn_index],
                    );
                }
            }
        }
        let text: &[u16] = if item == HUD_ITEM_BOTTLE_LEGACY
            && !USE_NEW_STYLE_INVENTORY
            && self.player_resources_view().equipped_bottle_index() != 0
        {
            let bottle_index = self.player_resources_view().equipped_bottle_index() as usize - 1;
            let idx = (self.inventory_items().bottle(bottle_index) as usize - 1) * 16;
            &HUD_BOTTLES_ITEM_TEXT[idx..idx + 16]
        } else if item == 5 && self.inventory_items().mushroom() != 1 {
            let idx = (self.inventory_items().mushroom() as usize - 2) * 16;
            &HUD_MUSHROOM_ITEM_TEXT[idx..idx + 16]
        } else if item == 20 && self.inventory_items().mirror() != 1 {
            let idx = (self.inventory_items().mirror() as usize - 2) * 16;
            &HUD_MIRROR_ITEM_TEXT[idx..idx + 16]
        } else if item == 13 && self.inventory_items().flute() != 1 {
            let idx = (self.inventory_items().flute() as usize - 2) * 16;
            &HUD_FLUTE_ITEM_TEXT[idx..idx + 16]
        } else if item == 1 && self.inventory_items().bow() != 1 {
            let idx = (self.inventory_items().bow() as usize - 2) * 16;
            &HUD_BOW_ITEM_TEXT[idx..idx + 16]
        } else if item >= HUD_ITEM_BOTTLE_FIRST && item <= HUD_ITEM_BOTTLE_LAST {
            let idx = (self
                .inventory_items()
                .bottle((item - HUD_ITEM_BOTTLE_FIRST) as usize) as usize
                - 1)
                * 16;
            &HUD_BOTTLES_ITEM_TEXT[idx..idx + 16]
        } else if item == HUD_ITEM_SHOVEL {
            &HUD_ITEM_TEXT[12]
        } else if item == 0 && btn_index != 0 {
            &HUD_NOT_ASSIGNED_ITEM_TEXT
        } else if item == 0 {
            &HUD_ITEM_TEXT[19]
        } else {
            &HUD_ITEM_TEXT[item as usize - 1]
        };
        self.hud_draw_nx_n(0x1000, hudxy(22 + dst_box, 8), text, 8, 2);
    }

    pub(super) fn hud_draw_equipment_box(&mut self) {
        let dst = if USE_NEW_STYLE_INVENTORY { 1 } else { 0 };
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
        if self.save_progress_view().palace_index_x2() == 0xff {
            for i in 0..8 {
                self.menu_set(hudxy(22 + dst + i, 26), 0x24f5);
            }
            self.hud_draw_item(
                0x1000,
                hudxy(25 + dst, 27),
                &HEART_PIECES[self.player_resources_view().heart_pieces() as usize],
            );
        }
        let sword = if self.inventory_items().sword_type() == 0xff {
            0
        } else {
            self.inventory_items().sword_type() as usize
        };
        self.hud_draw_item(0x1000, hudxy(22 + dst, 23), &HUD_ITEM_SWORD_GRAPHICS[sword]);
        self.hud_draw_item(
            0x1000,
            hudxy(25 + dst, 23),
            &HUD_ITEM_SHIELD_GRAPHICS[self.inventory_items().shield_type() as usize],
        );
        self.hud_draw_item(
            0x1000,
            hudxy(28 + dst, 23),
            &HUD_ITEM_ARMOR_GRAPHICS[self.inventory_items().armor() as usize],
        );
        const PALACE_ITEM: [ItemBoxGfx; 2] = [
            [0x28d6, 0x68d6, 0x28e6, 0x28e7],
            [0x354b, 0x354c, 0x354d, 0x354e],
        ];
        const DUNGEON_MAP: ItemBoxGfx = [0x28de, 0x28df, 0x28ee, 0x28ef];
        const DUNGEON_COMPASS: ItemBoxGfx = [0x24bf, 0x64bf, 0x2ccf, 0x6ccf];
        let shift = self.save_progress_view().palace_index_x2() >> 1;
        if self.save_progress_view().palace_index_x2() != 0xff
            && self.player_resources_view().has_big_key_at_shift(shift)
        {
            self.hud_draw_item(
                0x1000,
                hudxy(28 + dst, 27),
                &PALACE_ITEM[self.check_palace_item_posession() as usize],
            );
        }
        if self.save_progress_view().palace_index_x2() != 0xff
            && self.player_resources_view().has_dungeon_map_at_shift(shift)
        {
            self.hud_draw_item(0x1000, hudxy(22 + dst, 27), &DUNGEON_MAP);
        }
        if self.save_progress_view().palace_index_x2() != 0xff
            && self.player_resources_view().has_compass_at_shift(shift)
        {
            self.hud_draw_item(0x1000, hudxy(25 + dst, 27), &DUNGEON_COMPASS);
        }
    }

    pub(super) fn hud_draw_bottle_menu(&mut self) {
        let dst = if USE_NEW_STYLE_INVENTORY { 1 } else { 0 };
        self.hud_draw_box(0x1000, 21 + dst, 11, 30 + dst, 29, 2);
        for i in 0..4 {
            self.hud_draw_item(
                0x1000,
                hudxy(25 + dst, 13 + i * 4),
                &HUD_ITEM_BOTTLE_GRAPHICS[self.inventory_items().bottle(i) as usize],
            );
        }
        let bottle_index =
            (self.player_resources_view().equipped_bottle_index() as usize).wrapping_sub(1);
        let p = HUD_ITEM_BOTTLE_GRAPHICS[self.inventory_items().bottle(bottle_index) as usize];
        self.hud_draw_item(0x1000, HUD_ITEM_VRAM_POSITIONS_LEGACY[15], &p);
        if self.hud_state_view().flashing_circle_timer() & 0x10 != 0 {
            self.hud_draw_flashing_circle(0x1000, hudxy(25 + dst, 13 + bottle_index * 4) as i32, 7);
        }
    }

    pub(super) fn hud_refill_health(&mut self) -> bool {
        if self.player_resources_view().current_health()
            >= self.player_resources_view().health_capacity()
        {
            let capacity = self.player_resources_view().health_capacity();
            let mut resources = self.player_resources_view_mut();
            resources.set_current_health(capacity);
            resources.set_heart_filler(0);
            return !self.hud_state_view().is_doing_heart_animation();
        }
        self.player_resources_view_mut().set_heart_filler(160);
        false
    }

    pub(super) fn hud_animate_heart_refill(&mut self) {
        let cd = self
            .hud_state_view()
            .heart_refill_countdown()
            .wrapping_sub(1);
        self.hud_state_view_mut().set_heart_refill_countdown(cd);
        if cd != 0 {
            return;
        }
        let mut n =
            (((u16::from(self.player_resources_view().current_health() & !7)).wrapping_sub(1) >> 3)
                << 1) as usize;
        let mut p = hudxy(20, 1);
        if n >= 20 {
            n -= 20;
            p += 0x20;
        }
        n &= 0xff;
        self.hud_state_view_mut().set_heart_refill_countdown(1);
        const PARTIAL_HEART_ANIMATION_TILES: [u16; 4] = [0x24a3, 0x24a4, 0x24a3, 0x24a0];
        let subpos = self.hud_state_view().heart_refill_anim_subpos();
        self.hud_buffer_set(p + (n >> 1), PARTIAL_HEART_ANIMATION_TILES[subpos as usize]);
        let subpos = subpos.wrapping_add(1) & 3;
        self.hud_state_view_mut()
            .set_heart_refill_anim_subpos(subpos);
        if subpos == 0 {
            self.hud_rebuild();
            self.hud_state_view_mut().clear_is_doing_heart_animation();
        }
    }

    pub(super) fn hud_refill_magic_power(&mut self) -> bool {
        if self.player_resources_view().magic_power() >= 0x80 {
            return true;
        }
        self.player_resources_view_mut().set_magic_filler(0x80);
        false
    }

    pub(super) fn hud_restore_torch_background(&mut self) {
        if self.inventory_items().torch() == 0
            || self.dungeon_state_view().wants_lights_out() == 0
            || self.hud_state_view().dungeon_dark_with_lantern()
            || self.dungeon_state_view().lit_torches() != 0
        {
            return;
        }
        self.hud_state_view_mut().set_dungeon_dark_with_lantern();
        if self.dungeon_state_view().bg2_properties() != 2 {
            self.set_sub_screen_layers(1);
        }
    }

    pub(super) fn hud_rebuild_indoor(&mut self) {
        self.set_overworld_fixed_color_adjustment(0);
        self.player_resources_view_mut().set_keys(0xff);
        self.hud_rebuild();
    }

    pub(super) fn hud_rebuild(&mut self) {
        if self.hud_state_view().tile_word(hudxy(8, 2)) == 0 {
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
        self.system_signals_view_mut().increment_hud_update_flag();
    }

    pub(super) fn hud_get_item_box_table(&self, item: u8) -> &'static [ItemBoxGfx] {
        Self::hud_item_box_gfx_ptr(item)
    }

    pub(super) fn hud_get_item_box_ptr(&self, item: u8) -> &'static ItemBoxGfx {
        &Self::hud_item_box_gfx_ptr(item)[0]
    }

    pub(super) fn hud_handle_item_switch_inputs(&mut self) {
        if !self.enhanced_features_view().has(FEATURE_SWITCH_LR) {
            return;
        }
        let direction = if self.player_state_view().filtered_joypad_l() & JOYPAD_LOW_L != 0
            && self.save_progress_view().hud_current_item_slot(2) == 0
        {
            self.save_progress_view().hud_current_item_slot(3) != 0
        } else if self.player_state_view().filtered_joypad_l() & JOYPAD_LOW_R != 0
            && self.save_progress_view().hud_current_item_slot(3) == 0
        {
            true
        } else {
            return;
        };

        let mut item = self.save_progress_view().hud_current_item();
        for _ in 0..HUD_ITEM_COUNT {
            if !direction {
                self.hud_goto_prev_item(&mut item, 1);
            } else {
                self.hud_goto_next_item(&mut item, 1);
            }
            if self.hud_do_we_have_this_item(item)
                && (!self.enhanced_features_view().has(FEATURE_SWITCH_LR_LIMIT)
                    || self.hud_get_item_position(item) <= 3)
            {
                if item != self.save_progress_view().hud_current_item() {
                    self.save_progress_view_mut().set_hud_current_item(item);
                    self.system_signals_view_mut().set_sound_effect_2(32);
                    self.hud_update_equipped_item();
                    self.hud_update_item_box();
                    self.system_signals_view_mut().increment_hud_update_flag();
                }
                break;
            }
        }
    }

    fn hud_reorder_item(&mut self, direction: i32) {
        if !self.hud_inventory_order_state().is_custom() {
            self.initialize_default_hud_inventory_order(24);
        }
        let old_pos = self.hud_get_item_position(self.save_progress_view().hud_current_item());
        let mut new_pos = old_pos + direction;
        if new_pos < 0 {
            new_pos += HUD_ITEM_COUNT as i32;
        } else if new_pos >= HUD_ITEM_COUNT as i32 {
            new_pos -= HUD_ITEM_COUNT as i32;
        }
        self.swap_hud_inventory_order_items(old_pos as usize, new_pos as usize);
        self.hud_draw_y_button_items();
        self.system_signals_view_mut().set_sound_effect_2(32);
    }

    fn hud_update_item_box(&mut self) {
        if self.save_progress_view().hud_current_item() != 0 {
            let icon = self.hud_get_icon_for_item(self.save_progress_view().hud_current_item());
            self.hud_draw_item(HUD_TILE_INDICES_BUFFER, hudxy(5, 1), &icon);
        }
    }

    fn hud_update_hearts(&mut self) {
        const FULL: [u16; 3] = [0x24a2, 0x24a2, 0x24a2];
        const CURRENT: [u16; 3] = [0x24a2, 0x24a1, 0x24a0];
        self.hud_update_hearts_inner(
            hudxy(20, 1),
            &FULL,
            self.player_resources_view().health_capacity() as i32,
        );
        self.hud_update_hearts_inner(
            hudxy(20, 1),
            &CURRENT,
            (i32::from(self.player_resources_view().current_health()) + 3) & !3,
        );
    }

    fn hud_update_magic(&mut self) {
        let dst = hudxy(2, 0);
        if self.player_resources_view().magic_consumption_level() >= 1 {
            self.hud_buffer_set(dst + hudxy(0, 0), 0x28f7);
            self.hud_buffer_set(dst + hudxy(1, 0), 0x2851);
            self.hud_buffer_set(dst + hudxy(2, 0), 0x28fa);
        }
        let src = MAGIC_METER_TILEMAP_BY_LEVEL
            [(usize::from(self.player_resources_view().magic_power()) + 7) >> 3];
        for (y, tile) in src.iter().enumerate() {
            self.hud_buffer_set(dst + hudxy(1, y + 1), *tile);
        }
    }

    fn hud_update_inventory(&mut self) {
        const HUD_INVENTORY_BACKGROUND_TILES: [u16; 26] = [
            0x207f, 0x207f, 0x3ca8, 0x207f, 0x207f, 0x2c88, 0x2c89, 0x207f, 0x20a7, 0x20a9, 0x207f,
            0x2871, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f, 0x207f,
            0x207f, 0x207f, 0x207f, 0x207f,
        ];
        let d = hud_int_to_decimal(self.player_resources_view().rupees_actual() as u32);
        let inv_offs = usize::from(d[0] == 0x90);
        let dst = hudxy(8, 0);
        for i in 0..12 {
            self.hud_buffer_set(
                dst + hudxy(i, 0),
                HUD_INVENTORY_BACKGROUND_TILES[inv_offs + i],
            );
            self.hud_buffer_set(
                dst + hudxy(i, 1),
                HUD_INVENTORY_BACKGROUND_TILES[13 + inv_offs + i],
            );
        }
        let bow = self.inventory_items().bow();
        if bow != 0 {
            let has_arrows = self.player_resources_view().arrows() != 0;
            if bow >= 3 {
                self.hud_buffer_set(hudxy(15, 0), 0x2486);
                self.hud_buffer_set(hudxy(16, 0), 0x2487);
                self.inventory_items_mut()
                    .set_inventory_item(0, if has_arrows { 4 } else { 3 });
            } else {
                self.inventory_items_mut()
                    .set_inventory_item(0, if has_arrows { 2 } else { 1 });
            }
        }

        let base_tiles = [
            0x2400,
            if self
                .enhanced_features_view()
                .has(FEATURE_SHOW_MAX_ITEMS_IN_YELLOW)
            {
                0x3400
            } else {
                0x2400
            },
        ];
        let base_tile = base_tiles
            [usize::from(self.player_resources_view().rupees_actual() == self.max_rupees())];
        let digit_x = usize::from(inv_offs == 0);
        if inv_offs == 0 {
            self.hud_buffer_set(dst + hudxy(0, 1), base_tile | d[0] as u16);
        }
        self.hud_buffer_set(dst + hudxy(digit_x, 1), base_tile | d[1] as u16);
        self.hud_buffer_set(dst + hudxy(digit_x + 1, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(digit_x + 2, 1), base_tile | d[3] as u16);

        let d = hud_int_to_decimal(self.player_resources_view().bombs() as u32);
        let base_tile = base_tiles[usize::from(
            self.player_resources_view().bombs()
                == MAX_BOMBS_BY_UPGRADE_LEVEL
                    [self.player_resources_view().bomb_upgrade_level() as usize],
        )];
        self.hud_buffer_set(dst + hudxy(4, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(5, 1), base_tile | d[3] as u16);

        let d = hud_int_to_decimal(self.player_resources_view().arrows() as u32);
        let base_tile = base_tiles[usize::from(
            self.player_resources_view().arrows()
                == MAX_ARROWS_BY_UPGRADE_LEVEL
                    [self.player_resources_view().arrow_upgrade_level() as usize],
        )];
        self.hud_buffer_set(dst + hudxy(7, 1), base_tile | d[2] as u16);
        self.hud_buffer_set(dst + hudxy(8, 1), base_tile | d[3] as u16);

        let mut d = [0u8; 4];
        d[3] = 0x7f;
        if self.player_resources_view().keys() != 0xff {
            d = hud_int_to_decimal(self.player_resources_view().keys() as u32);
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
        if base == HUD_TILE_INDICES_BUFFER {
            self.hud_state_view_mut()
                .set_tile_word(tile as usize, value);
        } else {
            self.write_vram_upload_absolute_word(addr, value);
        }
    }

    fn read_tile(&self, base: usize, tile: i32) -> u16 {
        let addr = (base as i32 + tile * 2) as usize;
        if base == HUD_TILE_INDICES_BUFFER {
            self.hud_state_view().tile_word(tile as usize)
        } else {
            self.vram_upload_tilemap_word(addr - 0x1000)
        }
    }

    fn hud_item_box_gfx_ptr(item: u8) -> &'static [ItemBoxGfx] {
        match item {
            0 => &HUD_ITEM_BOW_GRAPHICS,
            1 => &HUD_ITEM_BOOMERANG_GRAPHICS,
            2 => &HUD_ITEM_HOOKSHOT_GRAPHICS,
            3 => &HUD_ITEM_BOMBS_GRAPHICS,
            4 => &HUD_ITEM_MUSHROOM_GRAPHICS,
            5 => &HUD_ITEM_FIRE_ROD_GRAPHICS,
            6 => &HUD_ITEM_ICE_ROD_GRAPHICS,
            7 => &HUD_ITEM_BOMBOS_GRAPHICS,
            8 => &HUD_ITEM_ETHER_GRAPHICS,
            9 => &HUD_ITEM_QUAKE_GRAPHICS,
            10 => &HUD_ITEM_TORCH_GRAPHICS,
            11 => &HUD_ITEM_HAMMER_GRAPHICS,
            12 => &HUD_ITEM_FLUTE_GRAPHICS,
            13 => &HUD_ITEM_BUG_NET_GRAPHICS,
            14 => &HUD_ITEM_BOOK_MUDORA_GRAPHICS,
            15 => &HUD_ITEM_BOTTLE_GRAPHICS,
            16 => &HUD_ITEM_CANE_SOMARIA_GRAPHICS,
            17 => &HUD_ITEM_CANE_BYRNA_GRAPHICS,
            18 => &HUD_ITEM_CAPE_GRAPHICS,
            19 => &HUD_ITEM_MIRROR_GRAPHICS,
            20 => &HUD_ITEM_GLOVES_GRAPHICS,
            21 => &HUD_ITEM_BOOTS_GRAPHICS,
            22 => &HUD_ITEM_FLIPPERS_GRAPHICS,
            23 => &HUD_ITEM_MOON_PEARL_GRAPHICS,
            24 => &HUD_ITEM_EMPTY_GRAPHICS,
            25 => &HUD_ITEM_SWORD_GRAPHICS,
            26 => &HUD_ITEM_SHIELD_GRAPHICS,
            27 => &HUD_ITEM_ARMOR_GRAPHICS,
            28..=31 => &HUD_ITEM_BOTTLE_GRAPHICS,
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

const SWITCH_LR_PALETTES: [u8; 4] = [7, 3, 4, 4];

const HUD_BOTTLES_ITEM_TEXT: [u16; 128] = [
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

const HUD_MUSHROOM_ITEM_TEXT: [u16; 16] = [
    0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255f, 0x255e, 0x2566,
    0x2553, 0x2554, 0x2561, 0x24f5,
];

const HUD_FLUTE_ITEM_TEXT: [u16; 32] = [
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2555, 0x255b, 0x2564, 0x2563,
    0x2554, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x2555, 0x255b, 0x2564, 0x2563, 0x2554, 0x24f5, 0x24f5, 0x24f5,
];

const HUD_MIRROR_ITEM_TEXT: [u16; 16] = [
    0x255c, 0x2550, 0x2556, 0x2558, 0x2552, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x255c, 0x2558,
    0x2561, 0x2561, 0x255e, 0x2561,
];

const HUD_BOW_ITEM_TEXT: [u16; 48] = [
    0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x256e, 0x256f,
    0x257c, 0x257d, 0x257e, 0x257f, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5,
    0x256b, 0x256c, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x256b, 0x256c, 0x24f5, 0x256e,
    0x256f, 0x24f5, 0x24f5, 0x24f5, 0x2578, 0x2579, 0x257a, 0x257b, 0x257c, 0x257d, 0x257e, 0x257f,
];

const HUD_NOT_ASSIGNED_ITEM_TEXT: [u16; 16] = [
    0x255d, 0x255e, 0x2563, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x24f5, 0x2550, 0x2562, 0x2562, 0x2558,
    0x2556, 0x255d, 0x2554, 0x2553,
];

const HUD_ITEM_TEXT: [[u16; 16]; 20] = [
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
