// Methods ported from zelda3/src/load_gfx.c and included inside ZeldaState.

use super::*;

use crate::chr_source;

mod load_gfx_shared;
use load_gfx_shared::*;
use load_gfx_shared::{
    aux_tileset as load_gfx_aux_tileset, main_tileset as load_gfx_main_tileset,
    sprite_tileset as load_gfx_sprite_tileset,
    MESSAGING_BUF_LOAD_GFX as LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
};

const LOAD_GFX_FEATURES0_MISC_BUG_FIXES: u32 = 4096;
const LOAD_GFX_FEATURES0_DIM_FLASHES: u32 = 65536;

const PALETTE_FILTERING_BITS: [u16; 64] = [
    0xffff, 0xffff, 0xfffe, 0xffff, 0x7fff, 0x7fff, 0x7fdf, 0xfbff, 0x7f7f, 0x7f7f, 0x7df7, 0xefbf,
    0x7bdf, 0x7bdf, 0x77bb, 0xddef, 0x7777, 0x7777, 0x6edd, 0xbb77, 0x6db7, 0x6db7, 0x5b6d, 0xb6db,
    0x5b5b, 0x5b5b, 0x56b6, 0xad6b, 0x5555, 0xad6b, 0x5555, 0xaaab, 0x5555, 0x5555, 0x2a55, 0x5555,
    0x2a55, 0x2a55, 0x294a, 0x5295, 0x2525, 0x2525, 0x2492, 0x4925, 0x1249, 0x1249, 0x1122, 0x4489,
    0x1111, 0x1111, 0x0844, 0x2211, 0x0421, 0x0421, 0x0208, 0x1041, 0x0101, 0x0101, 0x0020, 0x0401,
    0x0001, 0x0001, 0x0000, 0x0001,
];

const PALETTE_FILTER_UPPER_BITMASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010,
    0x0008, 0x0004, 0x0002, 0x0001,
];

const ITEM_ANIMATION_GFX_SOURCE_GROUPS: [usize; 10] = [0, 11, 8, 38, 42, 45, 34, 3, 33, 46];
const FOLLOWER_GFX_DECOMPRESSION_OFFSETS: [usize; 14] = [
    0, 0x600, 0x300, 0x300, 0x300, 0, 0, 0x900, 0x600, 0x600, 0x900, 0x900, 0x600, 0x900,
];
const SWORD_TYPE_TO_GFX_OFFSETS: [usize; 5] = [0, 0, 0x120, 0x120, 0x120];
const SHIELD_TYPE_TO_GFX_OFFSETS: [usize; 4] = [0x660, 0x660, 0x6f0, 0x900];

const INCREMENTAL_VRAM_UPLOAD_DESTINATIONS: [u8; 16] = [
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
];
const INCREMENTAL_VRAM_UPLOAD_SOURCES: [u16; 16] = [
    0x0000, 0x0200, 0x0400, 0x0600, 0x0800, 0x0a00, 0x0c00, 0x0e00, 0x1000, 0x1200, 0x1400, 0x1600,
    0x1800, 0x1a00, 0x1c00, 0x1e00,
];

const AGAHNIM_WARP_SHADOW_PALETTE_OFFSETS: [usize; 3] = [0x160, 0x180, 0x1a0];
const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
const SPOTLIGHT_DELTA_SIZE: [i8; 4] = [-7, 7, 7, 7];
const SPOTLIGHT_GOAL: [u16; 4] = [0, 126, 35, 126];
const SPOTLIGHT_CIRCLE_X_RADIUS_CURVE: [u8; 129] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfd, 0xfd, 0xfd, 0xfd, 0xfc, 0xfc, 0xfc, 0xfb, 0xfb, 0xfb, 0xfa, 0xfa, 0xf9, 0xf9, 0xf8, 0xf8,
    0xf7, 0xf7, 0xf6, 0xf6, 0xf5, 0xf5, 0xf4, 0xf3, 0xf3, 0xf2, 0xf1, 0xf1, 0xf0, 0xef, 0xee, 0xee,
    0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe9, 0xe8, 0xe7, 0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xdf, 0xde,
    0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7, 0xd6, 0xd5, 0xd3, 0xd2, 0xd0, 0xcf, 0xcd, 0xcc, 0xca, 0xc9,
    0xc7, 0xc6, 0xc4, 0xc2, 0xc1, 0xbf, 0xbd, 0xbb, 0xb9, 0xb7, 0xb6, 0xb4, 0xb1, 0xaf, 0xad, 0xab,
    0xa9, 0xa7, 0xa4, 0xa2, 0x9f, 0x9d, 0x9a, 0x97, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x82, 0x7f,
    0x7b, 0x78, 0x74, 0x70, 0x6c, 0x67, 0x63, 0x5e, 0x59, 0x53, 0x4d, 0x46, 0x3f, 0x37, 0x2d, 0x1f,
    0,
];

impl ZeldaState {
    fn sprite_gfx_subset(&self, slot: usize) -> u8 {
        self.game_state.sprites.workspace.graphics_subset(slot)
    }

    /// Public read-only accessor for the per-area sprite CHR identity: the 4
    /// `graphics_subset` packs (`slot` 0..4) loaded into VRAM by the sprite
    /// graphics loader. Used by the sprite-sheets probe/dump to key sprite cells
    /// by a stable per-area context (a hash of these 4 values). `slot >= 4`
    /// returns 0.
    pub fn parity_probe_sprite_graphics_subset(&self, slot: usize) -> u8 {
        if slot >= 4 {
            return 0;
        }
        self.sprite_gfx_subset(slot)
    }

    fn set_sprite_gfx_subset(&mut self, slot: usize, pack: u8) {
        self.sprite_workspace_mut().set_graphics_subset(slot, pack);
    }

    pub(super) fn world_map_setup_hdma(&mut self) {
        self.set_bg1_x(0x0080);
        self.set_bg1_y(0x00c8);
        self.set_mode7_center_y(0x01c9);
        self.set_mode7_center_x(0x0100);
        self.set_bg12_window_selection(0);
        self.set_bg34_window_selection(0);
        self.set_object_color_window_selection(0);
        self.set_main_screen_window_layers(0);
        self.set_sub_screen_window_layers(0);
        if self.game_state.frame.main_module == 20 {
            self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
            self.set_hdma_enable_mask(0xc0);
        } else if self.game_state.frame.submodule != 10 {
            self.set_mode7_zoom_step_counter(4);
            self.set_mode7_zoom_timer(12);
            self.set_overworld_map_flags(1);
            let y = self.game_state.player.special_exit_position.map_zoom_y();
            self.set_bg1_y(y);
            self.set_mode7_center_y(y.wrapping_add(0x100));
            let t0 = self
                .game_state
                .player
                .special_exit_position
                .map_zoom_x_offset();
            let abs_t0 = if (t0 as i16) < 0 {
                0u16.wrapping_sub(t0)
            } else {
                t0
            };
            let t1 = abs_t0.wrapping_mul(5) >> 1;
            let t2 = if (t0 as i16) < 0 {
                0u16.wrapping_sub(t1)
            } else {
                t1
            };
            self.set_bg1_x(t2.wrapping_add(0x80) & !1);
            self.OverworldMap_SetupHdma();
            self.set_hdma_enable_mask(0xc0);
        } else {
            self.set_mode7_zoom_step_counter(4);
            self.set_mode7_zoom_timer(33);
            self.set_overworld_map_flags(0);
            self.hdma_setup(0x0abdcf, 0x0abdcf, 0x42, 0x1b, 0x1e, 10);
            self.set_hdma_enable_mask(0xc0);
        }
    }

    pub(super) fn load_overworld_map_palette(&mut self) {
        if let Some(palette) = self.asset_raw(93).map(Vec::from) {
            let offset = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
                0x100
            } else {
                0
            };
            let len = 0x100.min(palette.len().saturating_sub(offset));
            self.copy_main_palette_bytes(&palette[offset..offset + len], len);
        }
    }

    pub(super) fn load_actual_gear_palettes(&mut self) {
        let inventory = &self.game_state.inventory.items;
        self.load_gear_palettes(
            inventory.sword_type(),
            inventory.shield_type(),
            inventory.armor(),
        );
        if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_MISC_BUG_FIXES)
        {
            self.palette_update_gloves_color();
        }
    }

    pub(super) fn palette_electro_themed_gear(&mut self) {
        self.load_gear_palettes(2, 2, 4);
    }

    pub(super) fn load_gear_palettes_bunny(&mut self) {
        self.load_gear_palettes(
            self.game_state.inventory.items.sword_type(),
            self.game_state.inventory.items.shield_type(),
            3,
        );
    }

    pub(super) fn load_gear_palettes(&mut self, sword: u8, shield: u8, armor: u8) {
        let sword_index = if sword != 0 && sword != 0xff {
            sword.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(82, sword_index * 3, 0x1b2, 2);

        let shield_index = if shield != 0 {
            shield.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(83, shield_index * 4, 0x1b8, 3);

        self.load_gear_palette_from_asset(81, armor as usize * 15, 0x1e2, 14);
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_update_gloves_color(&mut self) {
        let gloves = self.game_state.inventory.items.gloves();
        if gloves != 0 {
            let color = self.gloves_color(gloves.wrapping_sub(1) as usize);
            self.set_aux_color(0xfd, color);
            self.set_main_color(0xfd, color);
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_load_for_file_select(&mut self) {
        for slot in 0..3 {
            let src = slot * 0x500;
            let k = slot * 0x20;
            self.palette_load_for_file_select_armor(
                k,
                self.sram[src + KSRM_OFFS_ARMOR],
                self.sram[src + KSRM_OFFS_GLOVES],
            );
            self.palette_load_for_file_select_sword(k, self.sram[src + KSRM_OFFS_SWORD]);
            self.palette_load_for_file_select_shield(k, self.sram[src + KSRM_OFFS_SHIELD]);
        }

        let Some(main_sprite_palette) = self.asset_raw(80).map(Vec::from) else {
            return;
        };
        for i in 0..7 {
            let color = read_word_from_slice(&main_sprite_palette, (7 + i) * 2);
            self.set_aux_color(0xe8 + i, color);
            self.set_main_color(0xe8 + i, color);

            let color = read_word_from_slice(&main_sprite_palette, (15 + 7 + i) * 2);
            self.set_aux_color(0xf8 + i, color);
            self.set_main_color(0xf8 + i, color);
        }
    }

    pub(super) fn palette_load_for_file_select_armor(&mut self, k: usize, armor: u8, gloves: u8) {
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor as usize * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color(k + 0x81 + i, color);
            self.set_main_color(k + 0x81 + i, color);
        }
        if gloves != 0 {
            let color = self.gloves_color(gloves.wrapping_sub(1) as usize);
            self.set_aux_color(k + 0x8d, color);
            self.set_main_color(k + 0x8d, color);
        }
    }

    fn gloves_color(&self, glove_index: usize) -> u16 {
        self.replay_gloves_color(glove_index)
    }

    pub(super) fn palette_load_for_file_select_sword(&mut self, k: usize, sword: u8) {
        let Some(palette) = self.asset_raw(82).map(Vec::from) else {
            return;
        };
        let sword = if sword != 0 { sword.wrapping_sub(1) } else { 0 } as usize;
        let src = sword * 3 * 2;
        for i in 0..3 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color(k + 0x99 + i, color);
            self.set_main_color(k + 0x99 + i, color);
        }
    }

    pub(super) fn palette_load_for_file_select_shield(&mut self, k: usize, shield: u8) {
        let Some(palette) = self.asset_raw(83).map(Vec::from) else {
            return;
        };
        let shield = if shield != 0 {
            shield.wrapping_sub(1)
        } else {
            0
        } as usize;
        let src = shield * 4 * 2;
        for i in 0..4 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color(k + 0x9c + i, color);
            self.set_main_color(k + 0x9c + i, color);
        }
    }

    pub(super) fn load_gear_palette_from_asset(
        &mut self,
        asset: usize,
        color_offset: usize,
        dst: usize,
        x_ents: usize,
    ) {
        let Some(palette) = self.asset_raw(asset).map(Vec::from) else {
            return;
        };
        let base = color_offset * 2;
        let dst_index = dst >> 1;
        for i in 0..=x_ents {
            let color = read_word_from_slice(&palette, base + i * 2);
            self.set_aux_color(dst_index + i, color);
            self.set_main_color(dst_index + i, color);
        }
    }

    pub(super) fn reset_hud_palettes_4_and_5(&mut self) {
        for i in 0..8 {
            self.set_main_color(16 + i, 0);
        }
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_filter_history(&mut self) {
        self.palette_filter_range(0x10, 0x18);
        self.palette_filter_incr_countdown();
    }

    pub(super) fn palette_filter_range(&mut self, from: usize, to: usize) {
        let countdown = self.game_state.display.palette_filter.countdown_word();
        let load_ptr_offset = usize::from(countdown >= 0x10);
        let mask = PALETTE_FILTER_UPPER_BITMASKS[(countdown & 0x0f) as usize];
        let dt = if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word()
            != 0
        {
            1u16
        } else {
            0xffff
        };

        for j in from..to {
            let mut c = self.game_state.display.palette_buffer.main_color(j);
            let a = self.game_state.display.palette_buffer.aux_color(j);
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x001f) as usize) * 2] & mask == 0 {
                c = c.wrapping_add(dt);
            }
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x03e0) >> 4) as usize] & mask == 0 {
                c = c.wrapping_add(dt.wrapping_shl(5));
            }
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x7c00) >> 9) as usize] & mask == 0 {
                c = c.wrapping_add(dt.wrapping_shl(10));
            }
            self.set_main_color(j, c);
        }
    }

    pub(super) fn palette_filter_incr_countdown(&mut self) {
        let countdown = self
            .game_state
            .display
            .palette_filter
            .countdown_word()
            .wrapping_add(1);
        if countdown == 0x1f {
            self.set_countdown_word(0);
            let mode = self
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen_word()
                ^ 2;
            self.set_darkening_or_lightening_screen_word(mode);
            if mode != 0 {
                self.attract_scene_mut().increment_legend_flag();
            }
        } else {
            self.set_countdown_word(countdown);
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_fade_intro_one_step(&mut self) {
        self.palette_filter_restore_additive(0x100, 0x1a0);
        self.palette_filter_restore_additive(0x0c0, 0x100);
        self.decrement_countdown();
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_fade_intro2(&mut self) {
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.decrement_countdown();
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        let mut i = from >> 1;
        let end = to >> 1;
        while i != end {
            let c = self.game_state.display.palette_buffer.main_color(i);
            let d = self.game_state.display.palette_buffer.aux_color(i);
            let mut cx = c;
            if (c & 0x001f) != (d & 0x001f) {
                cx = cx.wrapping_add(1);
            }
            if (c & 0x03e0) != (d & 0x03e0) {
                cx = cx.wrapping_add(0x20);
            }
            if (c & 0x7c00) != (d & 0x7c00) {
                cx = cx.wrapping_add(0x400);
            }
            self.set_main_color(i, cx);
            i += 1;
        }
    }

    pub(super) fn filter_majorly_whiten_bg(&mut self) {
        for i in 32..128 {
            let color = self.game_state.display.palette_buffer.aux_color(i);
            let white = self.filter_majorly_whiten_color(color);
            self.set_main_color(i, white);
        }
        let color0 = if self.game_state.display.palette_buffer.aux_color(0) != 0 {
            self.game_state.display.palette_buffer.main_color(32)
        } else {
            0
        };
        self.set_main_color(0, color0);
    }

    fn filter_majorly_whiten_color(&self, color: u16) -> u16 {
        let amt = if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_DIM_FLASHES)
        {
            3
        } else {
            14
        };
        let r = ((color & 0x001f) + amt).min(0x001f);
        let g = ((color & 0x03e0) + (amt << 5)).min(0x03e0);
        let b = ((color & 0x7c00) + (amt << 10)).min(0x7c00);
        r | g | b
    }

    pub(super) fn palette_restore_bg_from_flash(&mut self) {
        for i in 32..128 {
            let color = self.game_state.display.palette_buffer.aux_color(i);
            self.set_main_color(i, color);
        }
        let color = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, color);
        self.palette_restore_coldata();
    }

    pub(super) fn palette_restore_bg_and_hud(&mut self) {
        let src = self.game_state.display.palette_buffer.aux_full_slice()[..0x100].to_vec();
        self.copy_main_palette_bytes(&src, 0x100);
        self.increment_cgram_update_flag();
        self.palette_restore_coldata();
    }

    pub(super) fn palette_restore_coldata(&mut self) {
        if self.game_state.world.location.is_indoors() {
            return;
        }
        let rgb = match self.game_state.world.location.overworld_screen_index() {
            3 | 5 | 7 => 0x8c4c26u32,
            0x43 | 0x45 | 0x47 => 0x874a26,
            0x5b => 0x894f33,
            _ => 0x804020,
        };
        self.set_fixed_color_red(rgb as u8);
        self.set_fixed_color_green((rgb >> 8) as u8);
        self.set_fixed_color_blue((rgb >> 16) as u8);
    }

    pub(super) fn overworld_load_all_palettes(&mut self) {
        self.clear_aux_sprite_subpalettes();
        self.clear_main_full();

        self.set_overworld_palette_mode(5);
        self.world_palette_theme_mut()
            .set_overworld_palette_aux1_hi(3);
        self.set_overworld_palette_aux2_hi(3);
        self.set_overworld_palette_aux3_lo(0);
        self.set_sp6r_indoors(5);
        self.set_sp0l(11);
        self.follower_state_mut().set_palette_swap_flag(0);
        self.clear_overworld_aux_or_main_offset();

        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_ow_bg_main();
        self.palette_load_ow_bg1();
        self.palette_load_ow_bg2();
        self.palette_load_ow_bg3();
        self.palette_load_sprite_environment_dungeon();
        self.palette_load_hud();

        for i in 0..8 {
            let color = self.game_state.display.palette_buffer.aux_color(0xe8 + i);
            self.set_main_color(0xd8 + i, color);
        }
    }

    pub(super) fn dungeon_load_palettes(&mut self) {
        self.clear_overworld_aux_or_main_offset();
        self.palette_bg_and_fixed_color_black();
        self.palette_load_sp0l();
        self.palette_load_sprite_main();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_sprite_environment();
        self.palette_load_link_armor_and_gloves();
        self.palette_load_hud();
        self.palette_load_dungeon_set();
        self.overworld_load_palettes_inner();
    }

    pub(super) fn overworld_load_palettes_inner(&mut self) {
        let main_indoors = self
            .game_state
            .display
            .palette_buffer
            .main_palette_indoors();
        let aux3_lo = self
            .game_state
            .display
            .palette_buffer
            .overworld_palette_aux3_lo();
        let main_indoors_copy = self
            .game_state
            .world
            .palette_theme
            .palette_main_indoors_copy();
        self.set_overworld_main_indoors_palette_backup(main_indoors);
        self.set_overworld_aux3_bg_palette_7_backup(aux3_lo);
        self.set_overworld_main_indoors_copy_palette_backup(main_indoors_copy);
        self.set_darkening_or_lightening_screen_word(2);
        self.set_countdown_word(0);
        self.clear_mosaic_target_level_word();
        self.overworld_copy_palettes_to_cache();
    }

    pub(super) fn palette_load_sp0l(&mut self) {
        let src = PALETTE_SPRITE_AUX3_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_0_left() as u32
                * 7
                * 2;
        let dst = if self.palette_swap_enabled() {
            0x1e2
        } else {
            0x102
        };
        self.palette_load_single(src, dst, 6);
    }

    pub(super) fn palette_load_sp5l(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_5_left() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1a2, 6);
    }

    pub(super) fn palette_load_sp6l(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_6_left() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1c2, 6);
    }

    pub(super) fn palette_load_sword(&mut self) {
        let sword = self.game_state.inventory.items.sword_type();
        let sword_index = if (sword as i8) > 0 {
            sword.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(82, sword_index * 3, 0x1b2, 2);
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_load_shield(&mut self) {
        let shield = self.game_state.inventory.items.shield_type();
        let shield_index = if shield != 0 {
            shield.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(83, shield_index * 4, 0x1b8, 3);
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_load_sprite_main(&mut self) {
        let offset = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            60 * 2
        } else {
            0
        };
        self.palette_load_multiple(PALETTE_MAIN_SPRITE_SNES_ADDR + offset, 0x122, 14, 3);
    }

    pub(super) fn palette_load_ow_bg_main(&mut self) {
        let src = PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_mode() as u32
                * 35
                * 2;
        self.palette_load_multiple(src, 0x42, 6, 4);
    }

    pub(super) fn palette_load_ow_bg1(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR
            + self
                .game_state
                .world
                .palette_theme
                .overworld_palette_aux1_hi() as u32
                * 21
                * 2;
        self.palette_load_multiple(src, 0x52, 6, 2);
    }

    pub(super) fn palette_load_ow_bg2(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_aux2_hi() as u32
                * 21
                * 2;
        self.palette_load_multiple(src, 0xb2, 6, 2);
    }

    pub(super) fn palette_load_ow_bg3(&mut self) {
        let src = PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_palette_aux3_lo() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0xe2, 6);
    }

    pub(super) fn palette_load_sprite_environment_dungeon(&mut self) {
        let src = PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR
            + self
                .game_state
                .display
                .palette_buffer
                .sprite_palette_6_right_indoors() as u32
                * 7
                * 2;
        self.palette_load_single(src, 0x1d2, 6);
    }

    pub(super) fn palette_load_sprite_environment(&mut self) {
        if self.game_state.world.location.is_indoors() {
            self.palette_load_sprite_environment_dungeon();
        } else {
            self.palette_misc_sprite_outdoors();
        }
    }

    pub(super) fn palette_misc_sprite_outdoors(&mut self) {
        let t = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            9
        } else {
            7
        };
        let src = PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR + t * 7 * 2;
        let dst = if self.palette_swap_enabled() {
            0x1f2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
        self.palette_load_single(src - 7 * 2, 0x1d2, 6);
    }

    pub(super) fn palette_load_hud(&mut self) {
        let src = HUD_PALETTE_SNES_ADDR
            + self.game_state.display.palette_buffer.hud_palette() as u32 * 32 * 2;
        self.palette_load_multiple(src, 0, 15, 1);
    }

    pub(super) fn palette_load_multiple_arbitrary_from_asset(
        &mut self,
        asset: usize,
        color_offset: usize,
        dst: usize,
        x_ents: usize,
    ) {
        let Some(palette) = self.asset_raw(asset).map(Vec::from) else {
            return;
        };
        let dst_index = dst >> 1;
        for i in 0..=x_ents {
            let color = read_word_from_slice(&palette, (color_offset + i) * 2);
            self.set_aux_color(dst_index + i, color);
            self.set_main_color(dst_index + i, color);
        }
    }

    pub(super) fn palette_load_multiple_arbitrary_snes(
        &mut self,
        src: u32,
        dst: usize,
        x_ents: usize,
    ) {
        let dst_index = dst >> 1;
        for i in 0..=x_ents {
            let Some(color) = self.rom_or_asset_word_snes(src + i as u32 * 2) else {
                return;
            };
            self.set_aux_color(dst_index + i, color);
            self.set_main_color(dst_index + i, color);
        }
    }

    pub(super) fn palette_load_dungeon_set(&mut self) {
        let src = PALETTE_DUNGEON_BG_MAIN_SNES_ADDR
            + (self
                .game_state
                .display
                .palette_buffer
                .main_palette_indoors()
                >> 1) as u32
                * 90
                * 2;
        self.palette_load_multiple(src, 0x42, 14, 5);
        let dst = if self.palette_swap_enabled() {
            0x1e2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
    }

    pub(super) fn palette_load_link_armor_and_gloves(&mut self) {
        let armor = self.game_state.inventory.items.armor() as usize;
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            self.set_aux_color((0x1e2 >> 1) + i, color);
            self.set_main_color((0x1e2 >> 1) + i, color);
        }
        self.palette_update_gloves_color();
    }

    pub(super) fn palette_load_single(&mut self, src: u32, dst: usize, x_ents: usize) {
        let base_byte = ((dst
            + self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize)
            >> 1)
            * 2;
        let base_idx = base_byte >> 1;
        for i in 0..=x_ents {
            let Some(color) = self.rom_or_asset_word_snes(src + i as u32 * 2) else {
                return;
            };
            self.set_aux_color(base_idx + i, color);
        }
    }

    pub(super) fn palette_load_multiple(
        &mut self,
        mut src: u32,
        mut dst: usize,
        x_ents: usize,
        y_pals: usize,
    ) {
        let count = x_ents + 1;
        for _ in 0..=y_pals {
            self.palette_load_single(src, dst, x_ents);
            src += count as u32 * 2;
            dst += 32;
        }
    }

    pub(super) fn load_item_gfx_into_wram_4bpp_buffer(&mut self) {
        let mut dst = 0x9480;
        dst = self.load_item_animation_gfx_one(dst, 7, 0, false);
        dst = self.load_item_animation_gfx_one(dst, 7, 1, false);
        dst = self.load_item_animation_gfx_one(dst, 3, 2, false);

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            95,
        );
        dst = self.load_item_animation_gfx_one(dst, 4, 3, true);
        dst = self.load_item_animation_gfx_one(dst, 3, 4, true);
        dst = self.load_item_animation_gfx_one(dst, 1, 5, true);
        dst = self.load_item_animation_gfx_one(dst, 4, 6, false);

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            96,
        );
        dst = self.load_item_animation_gfx_one(dst, 14, 7, true);
        dst = self.load_item_animation_gfx_one(dst, 7, 8, true);

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            95,
        );
        self.load_item_animation_gfx_one(dst, 2, 9, true);
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            84,
        );

        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.expand3_to_4_high_from_slice(0xa480, &tmp, 0, 0, 8);
        self.expand3_to_4_high_from_slice(0xa580, &tmp, 0x180, 0, 8);

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            96,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.expand3_to_4_high_from_slice(0xb280, &tmp, 0, 0, 3);
        self.expand3_to_4_high_from_slice(0xb2e0, &tmp, 0x180, 0, 3);

        self.load_item_gfx_auxiliary();
    }

    pub(super) fn load_item_animation_gfx_one(
        &mut self,
        dst: usize,
        num: usize,
        r12: usize,
        from_temp: bool,
    ) -> usize {
        let Some(source) = (if from_temp {
            Some(self.graphics_primary_decompression_buffer(0x600))
        } else {
            self.asset_bytes(64, 0).map(Vec::from)
        }) else {
            return dst + 0x40 * num;
        };

        let src = ITEM_ANIMATION_GFX_SOURCE_GROUPS[r12] * 24;
        self.expand3_to_4_high_from_slice(dst, &source, src, 0, num);
        self.expand3_to_4_high_from_slice(dst + 0x20 * num, &source, src + 0x180, 0, num);
        dst + 0x40 * num
    }

    pub(super) fn load_item_gfx_auxiliary(&mut self) {
        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x0f,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(PEG_TILE_GFX_BUFFER, &tmp, 0, 16);

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x58,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xb540, &tmp, 0, 32);

        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x05,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xbdc0, &tmp, 0x480, 2);
    }

    pub(super) fn load_follower_graphics(&mut self) {
        let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
        let mut yv = 0x64;
        if follower != 1 {
            yv = 0x66;
            if follower >= 9 {
                yv = 0x59;
                if follower >= 12 {
                    yv = 0x58;
                }
            }
        }

        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::secondary_buffer_offset(),
            yv,
        );
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x65,
        );
        let tmp = self.graphics_combined_decompression_buffers();
        let offset = FOLLOWER_GFX_DECOMPRESSION_OFFSETS
            .get(follower)
            .copied()
            .unwrap_or(0);
        self.do3_to_4_low_16bit_from_slice(0xb940, &tmp, offset, 0x20);
    }

    pub(super) fn decompress_sword_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-sword");
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::secondary_buffer_offset(),
            0x5f,
        );
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x5e,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        let sword = self.game_state.inventory.items.sword_type() as usize;
        let src = SWORD_TYPE_TO_GFX_OFFSETS.get(sword).copied().unwrap_or(0);
        self.expand3_to_4_high_from_slice(0x9000, &tmp, src, 0, 12);
        self.expand3_to_4_high_from_slice(0x9180, &tmp, src + 0x180, 0, 12);
        self.replay_trace_ram_watch("loadgfx-after-decompress-sword");
    }

    pub(super) fn decompress_shield_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-shield");
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::secondary_buffer_offset(),
            0x5f,
        );
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x5e,
        );
        let tmp = self.graphics_combined_decompression_buffers();
        let shield = self.game_state.inventory.items.shield_type() as usize;
        let src = SHIELD_TYPE_TO_GFX_OFFSETS
            .get(shield)
            .copied()
            .unwrap_or(0x660);
        self.expand3_to_4_high_from_slice(0x9300, &tmp, src, 0, 6);
        self.expand3_to_4_high_from_slice(0x93c0, &tmp, src + 0x180, 0, 6);
        self.replay_trace_ram_watch("loadgfx-after-decompress-shield");
    }

    pub(super) fn erase_tile_maps_normal(&mut self) {
        self.erase_tile_maps(0x007f, 0x01ec);
    }

    pub(super) fn erase_tile_maps_triforce(&mut self) {
        self.erase_tile_maps(0x00a9, 0x007f);
    }

    pub(super) fn erase_tile_maps(&mut self, r2: u16, r0: u16) {
        self.ppu.vram[..0x2000].fill(r0);
        self.ppu.vram[0x6000..0x6800].fill(r2);
    }

    pub(super) fn load_default_graphics(&mut self) {
        let Some(source) = self.asset_bytes(64, 0).map(Vec::from) else {
            return;
        };
        self.do3_to_4_high_to_vram(0x4000, &source, chr_source::CHR_KIND_SPRITE, 0);
        self.decompress_and_upload_2bpp(0x7000, 0x6a);
        self.decompress_and_upload_2bpp(0x7400, 0x6b);
        self.decompress_and_upload_2bpp(0x7800, 0x69);
    }

    pub(super) fn transfer_font_to_vram(&mut self) {
        let Some(font_pack) = self.asset_memblk(95, 0) else {
            return;
        };
        let font = find_index_in_memblk(font_pack, 0).ptr.to_vec();
        for i in 0..0x800 {
            self.ppu.vram[0x7000 + i] = read_word_from_slice(&font, i * 2);
        }
    }

    pub(super) fn decompress_and_upload_2bpp(&mut self, dst: usize, pack: usize) {
        let Some(data) = self.decompressed_sprite_graphics_data(pack) else {
            return;
        };
        self.copy_to_primary_decompression_buffer(&data);
        for i in 0..0x400 {
            self.ppu.vram[dst + i] = read_word_from_slice(&data, i * 2);
        }
    }

    pub(super) fn initialize_tilesets(&mut self) {
        let main_tile_set = load_gfx_main_tileset(
            self.game_state.world.palette_theme.main_tile_theme_index() as usize,
        );
        let aux_tile_set = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        );
        let sprite_tile_set =
            load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize);

        self.load_common_sprites();

        if sprite_tile_set[0] != 0 {
            self.sprite_workspace_mut()
                .set_graphics_subset(0, sprite_tile_set[0]);
        }
        if sprite_tile_set[1] != 0 {
            self.sprite_workspace_mut()
                .set_graphics_subset(1, sprite_tile_set[1]);
        }
        if sprite_tile_set[2] != 0 {
            self.sprite_workspace_mut()
                .set_graphics_subset(2, sprite_tile_set[2]);
        }
        if sprite_tile_set[3] != 0 {
            self.sprite_workspace_mut()
                .set_graphics_subset(3, sprite_tile_set[3]);
        }

        self.load_sprite_graphics(
            0x5000,
            self.game_state.sprites.workspace.graphics_subset(0) as usize,
            0x7800,
        );
        self.load_sprite_graphics(
            0x5400,
            self.game_state.sprites.workspace.graphics_subset(1) as usize,
            0x7e00,
        );
        self.load_sprite_graphics(
            0x5800,
            self.game_state.sprites.workspace.graphics_subset(2) as usize,
            0x8400,
        );
        self.load_sprite_graphics(
            0x5c00,
            self.game_state.sprites.workspace.graphics_subset(3) as usize,
            0x8a00,
        );

        self.world_palette_theme_mut().set_aux_bg_subset(
            0,
            if aux_tile_set[0] != 0 {
                aux_tile_set[0]
            } else {
                main_tile_set[3]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            1,
            if aux_tile_set[1] != 0 {
                aux_tile_set[1]
            } else {
                main_tile_set[4]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            2,
            if aux_tile_set[2] != 0 {
                aux_tile_set[2]
            } else {
                main_tile_set[5]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            3,
            if aux_tile_set[3] != 0 {
                aux_tile_set[3]
            } else {
                main_tile_set[6]
            },
        );

        self.load_background_graphics(
            0x2000,
            main_tile_set[0] as usize,
            7,
            GraphicsDecompressionScratch::primary_buffer_offset(),
        );
        self.load_background_graphics(
            0x2400,
            main_tile_set[1] as usize,
            6,
            GraphicsDecompressionScratch::primary_buffer_offset(),
        );
        self.load_background_graphics(
            0x2800,
            main_tile_set[2] as usize,
            5,
            GraphicsDecompressionScratch::primary_buffer_offset(),
        );
        self.load_background_graphics(
            0x2c00,
            self.game_state.world.palette_theme.aux_bg_subset(0) as usize,
            4,
            0x6000,
        );
        self.load_background_graphics(
            0x3000,
            self.game_state.world.palette_theme.aux_bg_subset(1) as usize,
            3,
            0x6600,
        );
        self.load_background_graphics(
            0x3400,
            self.game_state.world.palette_theme.aux_bg_subset(2) as usize,
            2,
            0x6c00,
        );
        self.load_background_graphics(
            0x3800,
            self.game_state.world.palette_theme.aux_bg_subset(3) as usize,
            1,
            0x7200,
        );
        self.load_background_graphics(
            0x3c00,
            main_tile_set[7] as usize,
            0,
            GraphicsDecompressionScratch::primary_buffer_offset(),
        );
    }

    pub(super) fn load_common_sprites(&mut self) {
        let misc_pack = self
            .game_state
            .world
            .palette_theme
            .misc_sprites_graphics_index() as usize;
        let Some(data) = self.asset_bytes(64, misc_pack).map(Vec::from) else {
            return;
        };
        self.do3_to_4_high_to_vram(0x4400, &data, chr_source::CHR_KIND_SPRITE, misc_pack as u16);

        if self.game_state.frame.main_module == 1 {
            self.load_sprite_graphics(
                0x4800,
                94,
                GraphicsDecompressionScratch::primary_buffer_offset(),
            );
            self.load_sprite_graphics(
                0x4c00,
                95,
                GraphicsDecompressionScratch::primary_buffer_offset(),
            );
            return;
        }

        let Some(data) = self.asset_bytes(64, 6).map(Vec::from) else {
            return;
        };
        self.do3_to_4_low_to_vram(0x4800, &data, chr_source::CHR_KIND_SPRITE, 6);

        let Some(data) = self.asset_bytes(64, 7).map(Vec::from) else {
            return;
        };
        self.do3_to_4_low_to_vram(0x4c00, &data, chr_source::CHR_KIND_SPRITE, 7);
    }

    pub(super) fn load_sprite_graphics(
        &mut self,
        dst: usize,
        gfx_pack: usize,
        decompression_buffer_offset: usize,
    ) {
        let Some(data) = self.decompressed_sprite_graphics_data(gfx_pack) else {
            return;
        };
        self.copy_decompressed_graphics_to(decompression_buffer_offset, &data);
        if matches!(gfx_pack, 0x52 | 0x53 | 0x5a | 0x5b | 0x5c | 0x5e | 0x5f) {
            self.do3_to_4_high_to_vram(dst, &data, chr_source::CHR_KIND_SPRITE, gfx_pack as u16);
        } else {
            self.do3_to_4_low_to_vram(dst, &data, chr_source::CHR_KIND_SPRITE, gfx_pack as u16);
        }
    }

    pub(super) fn load_background_graphics(
        &mut self,
        dst: usize,
        gfx_pack: usize,
        slot: usize,
        decompression_buffer_offset: usize,
    ) {
        let Some(data) = self.decompressed_background_graphics_data(gfx_pack) else {
            return;
        };
        self.copy_decompressed_graphics_to(decompression_buffer_offset, &data);
        let high = if self.game_state.world.palette_theme.main_tile_theme_index() >= 0x20 {
            matches!(slot, 7 | 2 | 3 | 4)
        } else {
            slot >= 4
        };
        if high {
            self.do3_to_4_high_to_vram(dst, &data, chr_source::CHR_KIND_BG, gfx_pack as u16);
        } else {
            self.do3_to_4_low_to_vram(dst, &data, chr_source::CHR_KIND_BG, gfx_pack as u16);
        }
    }

    pub(super) fn decompress_animated_dungeon_tiles(&mut self, pack: usize) {
        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            pack,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xa680, &tmp, 0, 48);

        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x5c,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xac80, &tmp, 0, 48);

        self.rotate_animated_dungeon_tile_planes();

        self.set_animated_tile_vram_destination_address(0x3b00);
    }

    pub(super) fn decompress_sprite_graphics_to_buffer(
        &mut self,
        dst: usize,
        mut gfx: usize,
    ) -> usize {
        if gfx < 12 {
            gfx = 12;
        }
        let Some(data) = self.decompressed_sprite_graphics_data(gfx) else {
            return 0;
        };
        self.copy_decompressed_graphics_to(dst, &data)
    }

    pub(super) fn decompress_background_graphics_to_buffer(
        &mut self,
        dst: usize,
        gfx: usize,
    ) -> usize {
        let Some(data) = self.decompressed_background_graphics_data(gfx) else {
            return 0;
        };
        self.copy_decompressed_graphics_to(dst, &data)
    }

    pub(super) fn decompressed_sprite_graphics_data(&self, mut gfx: usize) -> Option<Vec<u8>> {
        if gfx < 12 {
            gfx = 12;
        }
        let data = self.asset_bytes(64, gfx)?;
        if gfx < 103 && data.len() == 0x600 {
            Some(data.to_vec())
        } else {
            Some(decompress_asset(data))
        }
    }

    pub(super) fn decompressed_background_graphics_data(&self, gfx: usize) -> Option<Vec<u8>> {
        Some(decompress_asset(self.asset_bytes(65, gfx)?))
    }

    pub(super) fn expand3_to_4_high_from_slice(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        base: usize,
        num: usize,
    ) {
        for _ in 0..num {
            let src2 = src + 0x10;
            for i in 0..8 {
                let lo = data.get(src + i * 2).copied().unwrap_or(0);
                let hi = data.get(src + i * 2 + 1).copied().unwrap_or(0);
                let u = data.get(src2 + i).copied().unwrap_or(0);
                self.write_expanded_graphics_tile_row(dst, lo, hi, u, lo | hi | u);
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
            if (src.wrapping_sub(base) & 0x78) == 0 {
                src += 0x180;
            }
        }
    }

    #[track_caller]
    pub(super) fn do3_to_4_high_16bit_from_slice(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        num: usize,
    ) {
        for _ in 0..num {
            let src2 = src + 0x10;
            for i in 0..8 {
                let lo = data.get(src + i * 2).copied().unwrap_or(0);
                let hi = data.get(src + i * 2 + 1).copied().unwrap_or(0);
                let u = data.get(src2 + i).copied().unwrap_or(0);
                self.write_expanded_graphics_tile_row(dst, lo, hi, u, lo | hi | u);
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
        }
    }

    pub(super) fn do3_to_4_low_16bit_from_slice(
        &mut self,
        mut dst: usize,
        data: &[u8],
        mut src: usize,
        num: usize,
    ) {
        for _ in 0..num {
            let src2 = src + 0x10;
            for i in 0..8 {
                self.write_expanded_graphics_tile_row(
                    dst,
                    data.get(src + i * 2).copied().unwrap_or(0),
                    data.get(src + i * 2 + 1).copied().unwrap_or(0),
                    data.get(src2 + i).copied().unwrap_or(0),
                    0,
                );
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
        }
    }

    pub(super) fn do3_to_4_high_to_vram(
        &mut self,
        mut dst: usize,
        data: &[u8],
        chr_kind: u8,
        chr_pack: u16,
    ) {
        let start = dst;
        self.vram_chr_source
            .record_tiles(dst, 64, chr_kind, chr_pack);
        let mut src = 0usize;
        let mut tmp = [0u8; 8];
        for _ in 0..64 {
            for i in (0..8).rev() {
                let lo = data.get(src).copied().unwrap_or(0);
                let hi = data.get(src + 1).copied().unwrap_or(0);
                self.ppu.vram[dst] = lo as u16 | ((hi as u16) << 8);
                tmp[i] = lo | hi;
                self.set_dungeon_line_pointer_row0(i, tmp[i] as u16);
                dst += 1;
                src += 2;
            }
            for i in (0..8).rev() {
                let lo = data.get(src).copied().unwrap_or(0);
                self.ppu.vram[dst] = lo as u16 | (((lo | tmp[i]) as u16) << 8);
                dst += 1;
                src += 1;
            }
        }
        // OBJ CHR keyed by (pack, off) is heavily non-injective (subset numbers,
        // esp. pack 0, are reused with different gfx across areas). Re-tag sprite
        // tiles by content hash so the off-VRAM sprite path resolves the live cell.
        if chr_kind == chr_source::CHR_KIND_SPRITE {
            self.tag_stream_content_hash(start, 64);
        }
    }

    pub(super) fn do3_to_4_low_to_vram(
        &mut self,
        mut dst: usize,
        data: &[u8],
        chr_kind: u8,
        chr_pack: u16,
    ) {
        let start = dst;
        self.vram_chr_source
            .record_tiles(dst, 64, chr_kind, chr_pack);
        let mut src = 0usize;
        for _ in 0..64 {
            for _ in 0..8 {
                self.ppu.vram[dst] = read_word_from_slice(data, src);
                dst += 1;
                src += 2;
            }
            for _ in 0..8 {
                self.ppu.vram[dst] = data.get(src).copied().unwrap_or(0) as u16;
                dst += 1;
                src += 1;
            }
        }
        // See do3_to_4_high_to_vram: content-hash sprite CHR (subset key non-injective).
        if chr_kind == chr_source::CHR_KIND_SPRITE {
            self.tag_stream_content_hash(start, 64);
        }
    }

    pub(super) fn graphics_load_chr_half_slot(&mut self) {
        let k = self.game_state.display.chr_halfslot_request;
        if k == 0 {
            return;
        }

        let sp6 = GRAPHICS_LOAD_SP6[k as usize - 1];
        if sp6 >= 0 {
            self.set_sp6r_indoors(sp6 as u8);
            self.select_overworld_aux_palette_offset();
            if k == 1 {
                self.set_sp6r_indoors(10);
                self.Palette_Load_SpriteEnvironment();
            } else {
                self.Palette_Load_SpriteEnvironment_Dungeon();
            }
            self.increment_cgram_update_flag();
        }

        let mut tilebytes = 0x44;
        self.increment_chr_halfslot_request();
        if self.game_state.display.chr_halfslot_request & 1 != 0 {
            self.clear_chr_halfslot_request();
            if k != 18 {
                tilebytes = 0x46;
                if k == 2 {
                    self.clear_custom_spell_animation();
                }
            }
        }

        self.set_nmi_load_target_page(tilebytes);
        self.set_pending_nmi_subroutine(11);

        let mut pack = GRAPHICS_HALF_SLOT_PACKS[k as usize - 1] as usize;
        if pack == 1 {
            pack = self
                .game_state
                .world
                .palette_theme
                .misc_sprites_graphics_index() as usize;
        }
        let bank_offset = if tilebytes == 0x46 { 0x300 } else { 0 };
        self.load_chr_half_slot_pack(pack, bank_offset);
    }

    pub(super) fn load_chr_half_slot_pack(&mut self, pack: usize, bank_offset: usize) {
        let Some(data) = self.asset_bytes(64, pack) else {
            return;
        };
        let data = data.to_vec();

        let mut dst = NMI_BG_CHAR_HALF_BUFFER;
        let mut src = bank_offset;
        for _ in 0..32 {
            let mut sprite_data = [0u8; 24];
            for byte in &mut sprite_data {
                let Some(value) = data.get(src).copied() else {
                    return;
                };
                *byte = value;
                src += 1;
            }

            for i in 0..8 {
                let lo = sprite_data[i * 2];
                let hi = sprite_data[i * 2 + 1];
                let mask = sprite_data[16 + i];
                self.write_expanded_graphics_tile_row(dst, lo, hi, mask, lo | hi | mask);
                dst += 2;
            }
            dst += 16;
        }
    }

    pub(super) fn GetCompSpritePtr(&self, i: usize) -> u32 {
        COMP_SPRITE_PTRS[i]
    }

    pub(super) fn ApplyPaletteFilter_bounce(&mut self) {
        self.apply_palette_filter_bounce();
    }

    pub(super) fn PaletteFilter_Range(&mut self, from: usize, to: usize) {
        self.palette_filter_range(from, to);
    }

    pub(super) fn PaletteFilter_IncrCountdown(&mut self) {
        self.palette_filter_incr_countdown();
    }

    pub(super) fn LoadItemAnimationGfxOne(
        &mut self,
        dst: usize,
        num: usize,
        r12: usize,
        from_temp: bool,
    ) -> usize {
        self.load_item_animation_gfx_one(dst, num, r12, from_temp)
    }

    pub(super) fn snes_divide(&self, dividend: u16, divisor: u8) -> u16 {
        if divisor == 0 {
            0xffff
        } else {
            dividend / divisor as u16
        }
    }

    pub(super) fn EraseTileMaps_normal(&mut self) {
        self.erase_tile_maps_normal();
    }

    pub(super) fn DecompAndUpload2bpp(&mut self, vram_ptr: usize, pack: usize) {
        self.decompress_and_upload_2bpp(vram_ptr, pack);
    }

    pub(super) fn RecoverPegGFXFromMapping(&mut self) {
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.Dungeon_UpdatePegGFXBuffer(0x180, 0);
        } else {
            self.Dungeon_UpdatePegGFXBuffer(0, 0x180);
        }
    }

    pub(super) fn LoadOverworldMapPalette(&mut self) {
        self.load_overworld_map_palette();
    }

    pub(super) fn EraseTileMaps_triforce(&mut self) {
        self.erase_tile_maps_triforce();
    }

    pub(super) fn EraseTileMaps_dungeonmap(&mut self) {
        self.erase_tile_maps(0x007f, 0x0300);
    }

    pub(super) fn EraseTileMaps(&mut self, r2: u16, r0: u16) {
        self.erase_tile_maps(r2, r0);
    }

    pub(super) fn EnableForceBlank(&mut self) {
        self.enable_force_blank();
    }

    pub(super) fn LoadItemGFXIntoWRAM4BPPBuffer(&mut self) {
        self.load_item_gfx_into_wram_4bpp_buffer();
    }

    pub(super) fn DecompressSwordGraphics(&mut self) {
        self.decompress_sword_graphics();
    }

    pub(super) fn DecompressShieldGraphics(&mut self) {
        self.decompress_shield_graphics();
    }

    pub(super) fn DecompressAnimatedDungeonTiles(&mut self, a: u8) {
        self.decompress_animated_dungeon_tiles(a as usize);
    }

    pub(super) fn DecompressAnimatedOverworldTiles(&mut self, a: u8) {
        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            a as usize,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xa680, &tmp, 0, 64);

        self.decompress_background_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            a.wrapping_add(1) as usize,
        );
        let tmp = self.graphics_primary_decompression_buffer(0x600);
        self.do3_to_4_low_16bit_from_slice(0xae80, &tmp, 0, 32);
        self.set_animated_tile_vram_destination_address(0x3c00);
        // Remember the pack filling the animated-tile buffer so the per-frame
        // animated-tile DMA can tag its VRAM slots injectively (CHR_KIND_BG_ANIM).
        self.animated_tile_pack = a as u16;
    }

    pub(super) fn LoadItemGFX_Auxiliary(&mut self) {
        self.load_item_gfx_auxiliary();
    }

    pub(super) fn LoadFollowerGraphics(&mut self) {
        self.load_follower_graphics();
    }

    pub(super) fn WriteTo4BPPBuffer_at_7F4000(&mut self, a: u8) {
        let src = ANIMATED_SPRITE_TILE_SOURCE_OFFSETS
            .get(a as usize)
            .copied()
            .unwrap_or(0);
        let tmp = self.graphics_combined_decompression_buffers();
        self.expand3_to_4_high_from_slice(0xbd40, &tmp, src, 0, 2);
        self.expand3_to_4_high_from_slice(0xbd80, &tmp, src + 0x180, 0, 2);
    }

    pub(super) fn DecodeAnimatedSpriteTile_variable(&mut self, a: u8) {
        self.replay_trace_ram_watch("loadgfx-before-decode-animated-sprite-tile");
        let y = if a == 0x23 || a >= 0x37 {
            0x5d
        } else if a == 0x0c || a >= 0x24 {
            0x5c
        } else {
            0x5b
        };
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::secondary_buffer_offset(),
            y,
        );
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x5a,
        );
        self.WriteTo4BPPBuffer_at_7F4000(a);
        self.replay_trace_ram_watch("loadgfx-after-decode-animated-sprite-tile");
    }

    pub(super) fn Expand3To4High(&mut self, dst: usize, src: &[u8], base: &[u8], num: usize) {
        let base_off = if std::ptr::eq(src.as_ptr(), base.as_ptr()) {
            0
        } else {
            0
        };
        self.expand3_to_4_high_from_slice(dst, src, 0, base_off, num);
    }

    pub(super) fn LoadTransAuxGFX(&mut self) {
        let p = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize
        );
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.set_aux_bg_subset_pack(i, pack);
                assert_eq!(
                    self.decompress_background_graphics_to_buffer(
                        0x6000 + 0x600 * i,
                        pack as usize
                    ),
                    0x600
                );
            }
        }
        self.Gfx_LoadSpritesInner(0x7800);
    }

    pub(super) fn LoadTransAuxGFX_sprite(&mut self) {
        self.Gfx_LoadSpritesInner(0x7800);
    }

    pub(super) fn Gfx_LoadSpritesInner(&mut self, dst: usize) {
        let p = load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.set_sprite_gfx_subset(i, pack);
            }
            assert_eq!(
                self.decompress_sprite_graphics_to_buffer(
                    dst + 0x600 * i,
                    self.sprite_gfx_subset(i) as usize
                ),
                0x600
            );
        }
        self.reset_incremental_vram_upload_counter();
    }

    pub(super) fn ReloadPreviouslyLoadedSheets(&mut self) {
        self.decompress_background_graphics_to_buffer(
            0x6000,
            self.game_state.world.palette_theme.aux_bg_subset(0) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x6600,
            self.game_state.world.palette_theme.aux_bg_subset(1) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x6c00,
            self.game_state.world.palette_theme.aux_bg_subset(2) as usize,
        );
        self.decompress_background_graphics_to_buffer(
            0x7200,
            self.game_state.world.palette_theme.aux_bg_subset(3) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x7800,
            self.game_state.sprites.workspace.graphics_subset(0) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x7e00,
            self.game_state.sprites.workspace.graphics_subset(1) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x8400,
            self.game_state.sprites.workspace.graphics_subset(2) as usize,
        );
        self.decompress_sprite_graphics_to_buffer(
            0x8a00,
            self.game_state.sprites.workspace.graphics_subset(3) as usize,
        );
        self.reset_incremental_vram_upload_counter();
    }

    pub(super) fn Attract_DecompressStoryGFX(&mut self) {
        self.decompress_sprite_graphics_to_buffer(
            GraphicsDecompressionScratch::primary_buffer_offset(),
            0x67,
        );
        self.decompress_sprite_graphics_to_buffer(0x14800, 0x68);
    }

    pub(super) fn AnimateMirrorWarp(&mut self) {
        let st = self.overworld_map_state() as usize;
        self.increment_overworld_map_state();
        let nmi = MIRROR_WARP_LOAD_NEXT_NMI_LOAD.get(st).copied().unwrap_or(0);
        self.set_pending_nmi_subroutine(nmi);
        self.set_core_update_disable_flag(nmi);
        let xt = if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            8
        } else {
            0
        };

        match st {
            0 => {
                if self.mirror_warp_scratch_mut().increment_load_step_counter() != 32 {
                    self.set_overworld_map_state(0);
                } else {
                    self.SetTargetOverworldWarpToPyramid();
                }
            }
            1 => {
                self.AnimateMirrorWarp_DecompressNewTileSets();
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 1] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            2 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 2] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 3] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_high_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            3 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.world.palette_theme.aux_bg_subset(1) as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.world.palette_theme.aux_bg_subset(2) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            4 => {
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 4] as usize,
                );
                self.decompress_background_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    MIRROR_WARP_BG_PACK_SEQUENCE[xt + 5] as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            5 => {
                self.PreOverworld_LoadOverlays();
                if matches!(
                    self.game_state.world.location.overworld_screen_index(),
                    27 | 91
                ) {
                    self.set_sub_screen_layers(1);
                }
                self.decrement_submodule();
                self.set_pending_nmi_subroutine(12);
                self.set_core_update_disable_flag(12);
            }
            6 | 9 => {
                self.set_pending_nmi_subroutine(13);
                self.set_core_update_disable_flag(13);
            }
            7 => {
                self.Overworld_DrawScreenAtCurrentMirrorPosition();
                self.increment_core_update_disable_flag();
            }
            8 => {
                self.MirrorWarp_LoadSpritesAndColors();
                self.set_pending_nmi_subroutine(12);
                self.set_core_update_disable_flag(12);
            }
            10 => {
                let t = self.game_state.world.location.overworld_screen_index() & 0xbf;
                self.DecompressAnimatedOverworldTiles(if matches!(t, 3 | 5 | 7) {
                    0x58
                } else {
                    0x5a
                });
            }
            11 => {
                let t = self.game_state.world.location.overworld_screen_index();
                self.set_sub_screen_layers(matches!(
                    t,
                    0 | 0x70 | 0x40 | 0x5b | 3 | 5 | 7 | 0x43 | 0x45 | 0x47
                ) as u8);
                let Some(data) = self
                    .asset_bytes(64, MIRROR_WARP_BG_PACK_SEQUENCE[xt + 6] as usize)
                    .map(Vec::from)
                else {
                    return;
                };
                self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &data, 0, 64);
            }
            12 => {
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(0) as usize,
                );
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(1) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                if matches!(
                    self.game_state.sprites.workspace.graphics_subset(0),
                    0x52 | 0x53 | 0x5a | 0x5b
                ) {
                    self.do3_to_4_high_16bit_from_slice(
                        LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
                        &tmp,
                        0,
                        64,
                    );
                } else {
                    self.do3_to_4_low_16bit_from_slice(
                        LOAD_GFX_MESSAGING_BUF_LOAD_GFX,
                        &tmp,
                        0,
                        64,
                    );
                }
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            13 => {
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::primary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(2) as usize,
                );
                self.decompress_sprite_graphics_to_buffer(
                    GraphicsDecompressionScratch::secondary_buffer_offset(),
                    self.game_state.sprites.workspace.graphics_subset(3) as usize,
                );
                let tmp = self.graphics_combined_decompression_buffers();
                self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
                self.handle_followers_after_mirroring();
            }
            14 => {
                self.set_overworld_map_state(14);
            }
            _ => {}
        }
    }

    pub(super) fn AnimateMirrorWarp_DecompressNewTileSets(&mut self) {
        let main_tile_set = load_gfx_main_tileset(
            self.game_state.world.palette_theme.main_tile_theme_index() as usize,
        );
        let aux_tile_set = load_gfx_aux_tileset(
            self.game_state.world.palette_theme.aux_tile_theme_index() as usize,
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            0,
            if aux_tile_set[0] != 0 {
                aux_tile_set[0]
            } else {
                main_tile_set[3]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            1,
            if aux_tile_set[1] != 0 {
                aux_tile_set[1]
            } else {
                main_tile_set[4]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            2,
            if aux_tile_set[2] != 0 {
                aux_tile_set[2]
            } else {
                main_tile_set[5]
            },
        );
        self.world_palette_theme_mut().set_aux_bg_subset(
            3,
            if aux_tile_set[3] != 0 {
                aux_tile_set[3]
            } else {
                main_tile_set[6]
            },
        );

        let p = load_gfx_sprite_tileset(self.game_state.sprites.system.graphics_index() as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.set_sprite_gfx_subset(i, pack);
            }
        }
    }

    pub(super) fn Graphics_IncrementalVRAMUpload(&mut self) {
        let k = self
            .game_state
            .display
            .incremental_vram_upload_counter_usize();
        if k == 16 {
            return;
        }
        self.queue_tilemap_update(
            INCREMENTAL_VRAM_UPLOAD_DESTINATIONS[k],
            INCREMENTAL_VRAM_UPLOAD_SOURCES[k],
        );
        // Tag the per-frame animated-sprite CHR with an INJECTIVE logical source.
        // This streams the 4 per-area sprite gfx subsets (64 tiles each) to VRAM
        // 0x5000-0x5fff, 16 tiles per frame: chunk `k` fills VRAM page (0x50+k)
        // with subset `k/4`, sub-chunk `k%4`. `initialize_tilesets` already tagged
        // this region once via `load_sprite_graphics`, but with a possibly-stale /
        // zero `graphics_subset` (e.g. pack=0 when a subset was never set), while
        // the real animated content is re-DMA'd here every frame — leaving the
        // logical key (sprite, pack=0, tile_off) non-injective. Re-tag with the
        // LIVE subset pack so `(kind, pack, tile_off)` maps 1:1 to the pixels.
        let subset = k / 4;
        let pack = self.sprite_gfx_subset(subset) as u16;
        let dst_word = (INCREMENTAL_VRAM_UPLOAD_DESTINATIONS[k] as usize) << 8;
        let base_off = ((k % 4) * 16) as u16;
        self.vram_chr_source.record_tiles_from(
            dst_word,
            16,
            chr_source::CHR_KIND_SPRITE,
            pack,
            base_off,
        );
        self.increment_vram_upload_counter();
    }

    pub(super) fn PrepTransAuxGfx(&mut self) {
        let tmp = self.staged_bg_and_sprite_decompression_buffers();
        self.do3_to_4_high_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0x40);
        if self.game_state.world.palette_theme.aux_tile_theme_index() >= 32 {
            self.do3_to_4_high_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 0x80);
            self.do3_to_4_low_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        } else {
            self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 0xc0);
        }
    }

    pub(super) fn Do3To4High16Bit(&mut self, dst: usize, src: &[u8], num: usize) {
        self.do3_to_4_high_16bit_from_slice(dst, src, 0, num);
    }

    pub(super) fn Do3To4Low16Bit(&mut self, dst: usize, src: &[u8], num: usize) {
        self.do3_to_4_low_16bit_from_slice(dst, src, 0, num);
    }

    pub(super) fn LoadNewSpriteGFXSet(&mut self) {
        let tmp = self.graphics_sprite_decompression_buffer_tail();
        self.do3_to_4_low_16bit_from_slice(LOAD_GFX_MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0xc0);
        if matches!(
            self.game_state.sprites.workspace.graphics_subset(3),
            0x52 | 0x53 | 0x5a | 0x5b
        ) {
            self.do3_to_4_high_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        } else {
            self.do3_to_4_low_16bit_from_slice(0x11800, &tmp, 0x1200, 0x40);
        }
    }

    pub(super) fn InitializeTilesets(&mut self) {
        self.initialize_tilesets();
    }

    pub(super) fn LoadDefaultGraphics(&mut self) {
        self.load_default_graphics();
    }

    pub(super) fn Attract_LoadBG3GFX(&mut self) {
        self.decompress_and_upload_2bpp(0x7800, 0x67);
    }

    pub(super) fn Graphics_LoadChrHalfSlot(&mut self) {
        self.graphics_load_chr_half_slot();
    }

    pub(super) fn TransferFontToVRAM(&mut self) {
        self.transfer_font_to_vram();
    }

    pub(super) fn Do3To4High(&mut self, vram_ptr: usize, decompression_buffer_offset: &[u8]) {
        self.do3_to_4_high_to_vram(
            vram_ptr,
            decompression_buffer_offset,
            chr_source::CHR_KIND_NONE,
            0,
        );
    }

    pub(super) fn Do3To4Low(&mut self, vram_ptr: usize, decompression_buffer_offset: &[u8]) {
        self.do3_to_4_low_to_vram(
            vram_ptr,
            decompression_buffer_offset,
            chr_source::CHR_KIND_NONE,
            0,
        );
    }

    pub(super) fn LoadSpriteGraphics(
        &mut self,
        vram_ptr: usize,
        gfx_pack: usize,
        decompression_buffer_offset: usize,
    ) {
        self.load_sprite_graphics(vram_ptr, gfx_pack, decompression_buffer_offset);
    }

    pub(super) fn LoadBackgroundGraphics(
        &mut self,
        vram_ptr: usize,
        gfx_pack: usize,
        slot: usize,
        decompression_buffer_offset: usize,
    ) {
        self.load_background_graphics(vram_ptr, gfx_pack, slot, decompression_buffer_offset);
    }

    pub(super) fn LoadCommonSprites(&mut self) {
        self.load_common_sprites();
    }

    pub(super) fn Decomp_spr(&mut self, dst: usize, gfx: usize) -> usize {
        self.decompress_sprite_graphics_to_buffer(dst, gfx)
    }

    pub(super) fn Decomp_bg(&mut self, dst: usize, gfx: usize) -> usize {
        self.decompress_background_graphics_to_buffer(dst, gfx)
    }

    pub(super) fn Decompress(&self, dst: &mut [u8], src: &[u8]) -> usize {
        let data = decompress_asset(src);
        let len = dst.len().min(data.len());
        dst[..len].copy_from_slice(&data[..len]);
        len
    }

    pub(super) fn ResetHUDPalettes4and5(&mut self) {
        self.reset_hud_palettes_4_and_5();
    }

    pub(super) fn PaletteFilterHistory(&mut self) {
        self.palette_filter_history();
    }

    pub(super) fn PaletteFilter_WishPonds(&mut self) {
        self.set_sub_screen_layers(2);
        self.set_color_math_control(0x30);
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn PaletteFilter_Crystal(&mut self) {
        self.set_sub_screen_layers(1);
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn PaletteFilter_WishPonds_Inner(&mut self) {
        for i in 0..8 {
            self.set_main_color(0xd0 + i, 0);
        }
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_RestoreSP5F(&mut self) {
        for i in 0..8 {
            let color = self.game_state.display.palette_buffer.aux_color(208 + i);
            self.set_main_color(208 + i, color);
        }
        self.set_sub_screen_layers(0);
        self.set_color_math_control(32);
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_SP5F(&mut self) {
        for _ in 0..2 {
            self.palette_filter_range(208, 216);
            self.palette_filter_incr_countdown();
            if self.game_state.display.palette_filter.countdown_word() == 0 {
                break;
            }
        }
    }

    pub(super) fn KholdstareShell_PaletteFiltering(&mut self) {
        let t = if self
            .game_state
            .enhanced_features
            .has(LOAD_GFX_FEATURES0_MISC_BUG_FIXES)
        {
            0x50
        } else {
            0x40
        };
        if self.game_state.frame.subsubmodule == 0 {
            for i in 0..8 {
                let color = self.game_state.display.palette_buffer.aux_color(t + i);
                self.set_main_color(t + i, color);
            }
            self.set_countdown_word(0);
            self.set_darkening_or_lightening_screen_word(0);
            self.increment_cgram_update_flag();
            self.set_subsubmodule(1);
            return;
        }
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            self.palette_filter_incr_countdown();
            if self.game_state.display.palette_filter.countdown_word() == 0 {
                self.set_sub_screen_layers(0);
                break;
            }
        }
    }

    pub(super) fn AgahnimWarpShadowFilter(&mut self, k: usize) {
        let pal_countdown = self.agahnim_palette_word(k);
        let darkening_screen = self.agahnim_palette_word(k + 3);
        self.set_countdown_word(pal_countdown);
        self.set_darkening_or_lightening_screen_word(darkening_screen);
        let t = AGAHNIM_WARP_SHADOW_PALETTE_OFFSETS[k] >> 1;
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            let countdown = self
                .game_state
                .display
                .palette_filter
                .countdown_word()
                .wrapping_add(1);
            self.set_countdown_word(countdown);
            if countdown == 0x1f {
                self.set_countdown_word(0);
                let screen = self
                    .game_state
                    .display
                    .palette_filter
                    .darkening_or_lightening_screen_word()
                    ^ 2;
                self.set_darkening_or_lightening_screen_word(screen);
                break;
            }
        }
        let pal_countdown = self.game_state.display.palette_filter.countdown_word();
        let darkening_screen = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word();
        self.set_agahnim_palette_word(k, pal_countdown);
        self.set_agahnim_palette_word(k + 3, darkening_screen);
        self.increment_cgram_update_flag();
    }

    pub(super) fn Palette_FadeIntroOneStep(&mut self) {
        self.palette_fade_intro_one_step();
    }

    pub(super) fn Palette_FadeIntro2(&mut self) {
        self.palette_fade_intro2();
    }

    pub(super) fn PaletteFilter_RestoreAdditive(&mut self, from: usize, to: usize) {
        self.palette_filter_restore_additive(from, to);
    }

    pub(super) fn PaletteFilter_RestoreSubtractive(&mut self, from: usize, to: usize) {
        let mut i = from >> 1;
        let end = to >> 1;
        while i != end {
            let c = self.game_state.display.palette_buffer.main_color(i);
            let d = self.game_state.display.palette_buffer.aux_color(i);
            let mut cx = c;
            if (c & 0x001f) != (d & 0x001f) {
                cx = cx.wrapping_sub(1);
            }
            if (c & 0x03e0) != (d & 0x03e0) {
                cx = cx.wrapping_sub(0x20);
            }
            if (c & 0x7c00) != (d & 0x7c00) {
                cx = cx.wrapping_sub(0x400);
            }
            self.set_main_color(i, cx);
            i += 1;
        }
    }

    pub(super) fn PaletteFilter_InitializeWhiteFilter(&mut self) {
        for i in 0..256 {
            self.set_aux_color(i, 0x7fff);
        }
        let color = self.game_state.display.palette_buffer.main_color(0);
        self.set_main_color(32, color);
        self.set_countdown_word(0);
        self.set_darkening_or_lightening_screen_word(2);
        if self.game_state.world.location.overworld_screen_index() == 27 {
            self.set_aux_color(0, 0);
            self.set_aux_color(32, 0);
            self.set_main_color(0, 0);
            self.set_main_color(32, 0);
        }
        self.mirror_warp_scratch_mut().set_animation_counter(8);
        self.mirror_warp_scratch_mut().reset_load_step_counter();
    }

    pub(super) fn MirrorWarp_RunAnimationSubmodules(&mut self) {
        let ctr = self.mirror_warp_scratch_mut().decrement_animation_counter();
        if ctr != 0 {
            self.AnimateMirrorWarp();
            return;
        }
        self.mirror_warp_scratch_mut().set_animation_counter(2);
        self.PaletteFilter_BlindingWhite();
    }

    pub(super) fn PaletteFilter_BlindingWhite(&mut self) {
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0xff
        {
            return;
        }
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 2
        {
            self.palette_filter_restore_additive(0x40, 0x1b0);
            self.palette_filter_restore_additive(0x1c0, 0x1e0);
        } else {
            self.PaletteFilter_RestoreSubtractive(0x40, 0x1b0);
            self.PaletteFilter_RestoreSubtractive(0x1c0, 0x1e0);
        }
        self.PaletteFilter_StartBlindingWhite();
    }

    pub(super) fn PaletteFilter_StartBlindingWhite(&mut self) {
        let color = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, color);
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0
        {
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 66 {
                self.set_darkening_or_lightening_screen(0xff);
                self.mirror_warp_scratch_mut().set_animation_counter(32);
            }
        } else {
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                self.xor_darkening_or_lightening_screen(2);
                if self.game_state.frame.main_module != 21 {
                    return;
                }
                self.clear_hdma_enable_mask();
                for i in 0..240 {
                    self.set_spotlight_hdma_table_dynamic_entry(i, 0x0778);
                }
                self.set_hdma_enable_mask(0xc0);
            }
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_BlindingWhiteTriforce(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x200);
        self.PaletteFilter_StartBlindingWhite();
    }

    pub(super) fn PaletteFilter_WhirlpoolBlue(&mut self) {
        if self.game_state.frame.frame_counter & 1 != 0 {
            for i in 0x20..0x100 {
                let mut color = self.game_state.display.palette_buffer.main_color(i);
                if (color & 0x7c00) != 0x7c00 {
                    color = color.wrapping_add(0x400);
                }
                self.set_main_color(i, color);
            }
            let color = self.game_state.display.palette_buffer.main_color(32);
            self.set_main_color(0, color);
            if self.game_state.display.palette_filter.countdown() & 1 == 0 {
                self.increment_mosaic_level_by(16);
            }
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                self.set_countdown(0);
                self.increment_subsubmodule();
                self.set_mosaic_level(0xf0);
            }
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_IsolateWhirlpoolBlue(&mut self) {
        for i in 0x20..0x100 {
            let mut color = self.game_state.display.palette_buffer.main_color(i);
            if color & 0x03e0 != 0 {
                color = color.wrapping_sub(0x20);
            }
            if color & 0x001f != 0 {
                color = color.wrapping_sub(1);
            }
            self.set_main_color(i, color);
        }
        let color = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, color);
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 31 {
            self.set_countdown(0);
            self.increment_subsubmodule();
            self.set_mosaic_level(0xf0);
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_WhirlpoolRestoreBlue(&mut self) {
        if self.game_state.frame.frame_counter & 1 != 0 {
            for i in 0x20..0x100 {
                let aux = self.game_state.display.palette_buffer.aux_color(i) & 0x7c00;
                let mut color = self.game_state.display.palette_buffer.main_color(i);
                if color & 0x7c00 != aux {
                    color = color.wrapping_sub(0x400);
                }
                self.set_main_color(i, color);
            }
            let color = self.game_state.display.palette_buffer.main_color(32);
            self.set_main_color(0, color);
            if self.game_state.display.palette_filter.countdown() & 1 == 0 {
                self.decrement_mosaic_level_by(16);
            }
            self.increment_countdown();
            if self.game_state.display.palette_filter.countdown() == 31 {
                self.set_countdown(0);
                self.increment_subsubmodule();
                self.clear_mosaic_level();
            }
        }
        self.set_bg_mode(9);
        self.set_mosaic_copy_from_level_or(3);
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_WhirlpoolRestoreRedGreen(&mut self) {
        for i in 0x20..0x100 {
            let aux = self.game_state.display.palette_buffer.aux_color(i);
            let mut color = self.game_state.display.palette_buffer.main_color(i);
            if color & 0x03e0 != aux & 0x03e0 {
                color = color.wrapping_add(0x20);
            }
            if color & 0x001f != aux & 0x001f {
                color = color.wrapping_add(1);
            }
            self.set_main_color(i, color);
        }
        let color = self.game_state.display.palette_buffer.main_color(32);
        self.set_main_color(0, color);
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 31 {
            self.set_countdown(0);
            self.increment_subsubmodule();
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_RestoreBGSubstractiveStrict(&mut self) {
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen()
            == 0xff
        {
            return;
        }
        self.PaletteFilter_RestoreSubtractive(0x40, 0x100);
        self.increment_countdown();
        if self.game_state.display.palette_filter.countdown() == 0x20 {
            self.set_darkening_or_lightening_screen(0xff);
            self.clear_sub_screen_layers_word();
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn PaletteFilter_RestoreBGAdditiveStrict(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x100);
        self.increment_countdown();
        self.increment_cgram_update_flag();
    }

    pub(super) fn Trinexx_FlashShellPalette_Red(&mut self) {
        if self.game_state.display.trinexx_palette.red_shell_delay == 0 {
            for i in 0..7 {
                let v = self.game_state.display.palette_buffer.main_color(0x41 + i);
                let red = (v & 0x1f).wrapping_add(u16::from((v & 0x1f) != 0x1f));
                self.set_main_color(0x41 + i, (v & 0xffe0) | red);
            }
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_red_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_red_shell_palette_step(0);
                self.set_trinexx_red_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_red_shell_palette_delay(3);
        }
        self.decrement_trinexx_red_shell_palette_delay();
    }

    pub(super) fn Trinexx_UnflashShellPalette_Red(&mut self) {
        if self.game_state.display.trinexx_palette.red_shell_delay == 0 {
            for i in 0..7 {
                let u = self.game_state.display.palette_buffer.aux_color(0x41 + i);
                let v = self.game_state.display.palette_buffer.main_color(0x41 + i);
                let red = (v & 0x1f).wrapping_sub(u16::from((v & 0x1f) != (u & 0x1f)));
                self.set_main_color(0x41 + i, (v & 0xffe0) | red);
            }
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_red_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_red_shell_palette_step(0);
                self.set_trinexx_red_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_red_shell_palette_delay(3);
        }
        self.decrement_trinexx_red_shell_palette_delay();
    }

    pub(super) fn Trinexx_FlashShellPalette_Blue(&mut self) {
        if self.game_state.display.trinexx_palette.blue_shell_delay == 0 {
            for i in 0..7 {
                let v = self.game_state.display.palette_buffer.main_color(0x41 + i);
                let blue =
                    (v & 0x7c00).wrapping_add(if (v & 0x7c00) != 0x7c00 { 0x0400 } else { 0 });
                self.set_main_color(0x41 + i, (v & !0x7c00) | blue);
            }
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_blue_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_blue_shell_palette_step(0);
                self.set_trinexx_blue_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_blue_shell_palette_delay(3);
        }
        self.decrement_trinexx_blue_shell_palette_delay();
    }

    pub(super) fn Trinexx_UnflashShellPalette_Blue(&mut self) {
        if self.game_state.display.trinexx_palette.blue_shell_delay == 0 {
            for i in 0..7 {
                let u = self.game_state.display.palette_buffer.aux_color(0x41 + i);
                let v = self.game_state.display.palette_buffer.main_color(0x41 + i);
                let blue = (v & 0x7c00).wrapping_sub(if (v & 0x7c00) != (u & 0x7c00) {
                    0x0400
                } else {
                    0
                });
                self.set_main_color(0x41 + i, (v & !0x7c00) | blue);
            }
            self.increment_cgram_update_flag();
            let step = self.increment_trinexx_blue_shell_palette_step();
            if step >= 12 {
                self.set_trinexx_blue_shell_palette_step(0);
                self.set_trinexx_blue_shell_palette_delay(0);
                return;
            }
            self.set_trinexx_blue_shell_palette_delay(3);
        }
        self.decrement_trinexx_blue_shell_palette_delay();
    }

    pub(super) fn IrisSpotlight_close(&mut self) {
        self.spotlight_internal(0x7e, 0);
    }

    pub(super) fn Spotlight_open(&mut self) {
        self.spotlight_open();
    }

    pub(super) fn SpotlightInternal(&mut self) {
        self.spotlight_internal(0, 2);
    }

    pub(super) fn IrisSpotlight_ConfigureTable(&mut self) {
        self.iris_spotlight_configure_table();
    }

    pub(super) fn IrisSpotlight_ResetTable(&mut self) {
        self.iris_spotlight_reset_table();
    }

    pub(super) fn IrisSpotlight_CalculateCircleValue(&self, value: u16) -> u16 {
        self.iris_spotlight_calculate_circle_value(value as u8)
    }

    pub(super) fn AdjustWaterHDMAWindow(&mut self) {
        let r10 = self
            .game_state
            .display
            .water_hdma_window
            .window_y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        // WATER_HDMA_WINDOW_Y_RADIUS (0x684) is dual-modeled (DisplayState.water_hdma_window vs
        // DungeonEnvironmentState.water_hdma_y_radius written by the swamp/flood scene); the
        // DisplayState model is stale during the swamp scene, leaving the spotlight a scanline off
        // (f606748). Read the live RAM byte both scenes write, like the old clone.
        let y_radius = crate::types::read_le_u16(
            &self.ram,
            crate::game_state::constants::WATER_HDMA_WINDOW_Y_RADIUS,
        );
        self.set_spotlight_y_lower(r10.wrapping_sub(y_radius));
        self.set_spotlight_y_upper(r10.wrapping_add(y_radius));
        self.AdjustWaterHDMAWindow_X(r10);
    }

    pub(super) fn AdjustWaterHDMAWindow_X(&mut self, r10: u16) {
        let window_x_center = self
            .game_state
            .display
            .water_hdma_window
            .window_x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        self.set_spotlight_window_x_center(window_x_center);
        // WATER_HDMA_WINDOW_X_RADIUS (0x686) is dual-modeled: DisplayState.water_hdma_window
        // (watergate scene) AND DungeonEnvironmentState.water_hdma_x_radius (swamp/flood water,
        // written by Module07_0C). This reader only consulted the DisplayState model, which is
        // stale during the swamp scene, so the HDMA window was a pixel off (f606612). Read the live
        // RAM byte both scenes write, like the old clone.
        let r12 = crate::types::read_le_u16(
            &self.ram,
            crate::game_state::constants::WATER_HDMA_WINDOW_X_RADIUS,
        )
        .saturating_sub(1);
        let r2 = window_x_center.wrapping_add(r12).min(255);
        let r0 = window_x_center.wrapping_sub(r12).min(255);
        let r12_pair = r0 | (r2 << 8);

        let mut r6 = r10.wrapping_mul(2);
        if r6 < 0xe0 {
            r6 = 0xe0;
        }
        let mut r4 = r10.wrapping_mul(2).wrapping_sub(r6);
        loop {
            if (r4 as i16) >= 0 {
                let y_lower = self.game_state.display.spotlight_hdma.y_lower();
                let a = if (y_lower as i16) >= 0 && r4 < y_lower {
                    0x00ff
                } else {
                    r12_pair
                };
                if r4 < 240 {
                    self.set_spotlight_hdma_table_dynamic_entry(
                        r4 as usize,
                        if a != 0xffff { a } else { 0x00ff },
                    );
                }
            }

            let y_upper = self.game_state.display.spotlight_hdma.y_upper();
            let a = if r6 >= y_upper {
                0x00ff
            } else {
                if r6 >= 225
                    && self
                        .game_state
                        .display
                        .water_hdma_window
                        .watergate_spotlight_y_upper()
                        != 0
                {
                    self.water_hdma_window_mut()
                        .decrement_watergate_spotlight_y_upper();
                }
                r12_pair
            };
            if r6 < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(
                    r6 as usize,
                    if a != 0xffff { a } else { 0x00ff },
                );
            }
            if r10 == r4 {
                break;
            }
            r6 = r6.wrapping_sub(1);
            r4 = r4.wrapping_add(1);
        }
    }

    pub(super) fn FloodDam_PrepFloodHDMA(&mut self) {
        let lower = self
            .game_state
            .display
            .water_hdma_window
            .window_y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        self.set_spotlight_y_lower(lower);
        let window_x_center = self
            .game_state
            .display
            .water_hdma_window
            .window_x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        self.set_spotlight_window_x_center(window_x_center);
        let r14 = self.game_state.display.water_hdma_window.window_x_radius() ^ 1;
        let mut r4 = 0usize;
        let upper = self.game_state.display.spotlight_hdma.y_upper() as usize;
        while r4 != upper {
            self.set_spotlight_hdma_table_dynamic_entry(r4, 0xff00);
            r4 += 1;
        }
        let r12w = r14.wrapping_sub(7).wrapping_add(8);
        let r12_pair = (window_x_center.wrapping_add(r12w) << 8)
            | (window_x_center.wrapping_sub(r12w) & 0x00ff);
        let r10 = (self
            .game_state
            .display
            .spotlight_hdma
            .y_upper()
            .wrapping_add(self.game_state.display.water_hdma_window.window_y_radius()))
            ^ 1;
        while r4 < 225 {
            if r4 as u16 >= r10 {
                self.set_spotlight_hdma_table_dynamic_entry(r4, 0x00ff);
            } else {
                let mut a = r4 as u16;
                loop {
                    a = a.wrapping_mul(2);
                    if a < 480 {
                        break;
                    }
                }
                self.set_spotlight_hdma_table_dynamic_entry(
                    a as usize >> 1,
                    if r12_pair == 0xffff { 0x00ff } else { r12_pair },
                );
            }
            r4 += 1;
        }
    }

    pub(super) fn ResetStarTileGraphics(&mut self) {
        // C does a single `ram[STAR_TILE_RESTORE_PHASE] = 0` (0x4bc). 0x4bc is mode-reused:
        // DisplayState owns it as star_tile_restore_phase (projection gated to overworld) and
        // dungeon room_effects owns it as moving_wall_torch_blink_phase (projection gated to
        // indoors). clear_star_tile_restore_phase alone leaves ram[0x4bc] stale in a dungeon
        // (its projection is gated off) — the residual then drove a wrong star-tile gfx restore
        // (f317704). Clear BOTH owners so whichever projection is active writes 0, matching C.
        self.clear_star_tile_restore_phase();
        self.dungeon_room_effects_mut()
            .clear_moving_wall_torch_blink_phase();
        self.Dungeon_RestoreStarTileChr();
    }

    pub(super) fn Dungeon_RestoreStarTileChr(&mut self) {
        let (xx, yy) = self.dungeon_star_tile_restore_source_offsets();
        let src0 = 0xbdc0 + xx;
        let src1 = 0xbdc0 + yy;
        self.copy_graphics_message_rows(0, src0, src1, 32);
        self.set_pending_nmi_subroutine(0x18);
    }

    pub(super) fn LinkZap_HandleMosaic(&mut self) {
        let mut level = self.game_state.display.mosaic_level;
        if self.game_state.display.mosaic_direction == 0 {
            level = level.wrapping_add(0x10);
            if level == 0xc0 {
                self.set_mosaic_direction(1);
            }
        } else {
            level = level.wrapping_sub(0x10);
            if level == 0 {
                self.clear_mosaic_direction();
            }
        }
        self.set_mosaic_level(level);
        self.set_mosaic_copy((level >> 1) | 3);
        self.set_bg_mode(9);
    }

    pub(super) fn Player_SetCustomMosaicLevel(&mut self, level: u8) {
        self.clear_mosaic_direction();
        self.set_mosaic_level(level);
        self.set_mosaic_copy((level >> 1) | 3);
        self.set_bg_mode(9);
    }

    pub(super) fn Module07_16_UpdatePegs_Step1(&mut self) {
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.Dungeon_UpdatePegGFXBuffer(0x80, 0x100);
        } else {
            self.Dungeon_UpdatePegGFXBuffer(0x100, 0x80);
        }
    }

    pub(super) fn Module07_16_UpdatePegs_Step2(&mut self) {
        if self
            .game_state
            .dungeon
            .environment
            .orange_blue_barrier_state()
            != 0
        {
            self.Dungeon_UpdatePegGFXBuffer(0x100, 0x80);
        } else {
            self.Dungeon_UpdatePegGFXBuffer(0x80, 0x100);
        }
    }

    pub(super) fn Dungeon_UpdatePegGFXBuffer(&mut self, x: usize, y: usize) {
        self.copy_peg_tile_graphics_to_message_buffer(x, y);
        self.set_pending_nmi_subroutine(23);
    }

    pub(super) fn Dungeon_HandleTranslucencyAndPalette(&mut self) {
        if self.palette_swap_enabled() {
            self.Palette_RevertTranslucencySwap();
        }

        self.set_color_window_selection(2);
        self.set_color_math_control(0xb3);

        let mut torch = self.game_state.dungeon.torch.lit_torches();
        if self.game_state.dungeon.torch.wants_lights_out() == 0 {
            let mut a = 0x20;
            if self.game_state.dungeon.room_load.bg2_properties() != 0 {
                a = 0x32;
                if self.game_state.dungeon.room_load.bg2_properties() != 7 {
                    a = 0x62;
                    if self.game_state.dungeon.room_load.bg2_properties() != 4 {
                        a = 0x20;
                        if self.game_state.dungeon.room_load.bg2_properties() == 2 {
                            self.Palette_AssertTranslucencySwap();
                            if self.game_state.world.location.dungeon_room_index() == 13 {
                                self.clear_agahnim_palette_settings(6);
                                self.Palette_LoadAgahnim();
                            }
                            a = 0x70;
                        }
                    }
                }
            }
            self.set_color_math_control(a);
            torch = 3;
        }
        self.set_overworld_fixed_color_adjustment(
            LIT_TORCHES_COLOR_PLUS
                .get(torch as usize)
                .copied()
                .unwrap_or(0),
        );
        self.set_countdown(31);
        self.clear_mosaic_target_level();
        self.set_darkening_or_lightening_screen(2);
        self.clear_overworld_aux_or_main_offset();
        self.palette_load_dungeon_set();
        self.palette_load_sp0l();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.increment_subsubmodule();
    }

    pub(super) fn Overworld_LoadAllPalettes(&mut self) {
        self.overworld_load_all_palettes();
    }

    pub(super) fn Dungeon_LoadPalettes(&mut self) {
        self.dungeon_load_palettes();
    }

    pub(super) fn Overworld_LoadPalettesInner(&mut self) {
        self.overworld_load_palettes_inner();
    }

    pub(super) fn OverworldLoadScreensPaletteSet(&mut self) {
        let sc = self.game_state.world.location.overworld_screen_index() & 0x3f;
        let mut x = if matches!(sc, 3 | 5 | 7) { 2 } else { 0 };
        if self.game_state.world.location.overworld_screen_index() & 0x40 != 0 {
            x += 1;
        }
        self.Overworld_LoadAreaPalettesEx(x);
    }

    pub(super) fn Overworld_LoadAreaPalettesEx(&mut self, palette: u8) {
        self.set_overworld_palette_mode(palette);
        self.keep_overworld_aux_or_main_low_byte();
        self.palette_load_sprite_main();
        self.palette_load_sprite_environment();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_link_armor_and_gloves();
        let sp0l = if self.game_state.inventory.save_progress.dark_world_state() & 0x40 != 0 {
            3
        } else {
            1
        };
        self.set_sp0l(sp0l);
        self.palette_load_sp0l();
        self.palette_load_hud();
        self.palette_load_ow_bg_main();
    }

    pub(super) fn SpecialOverworld_CopyPalettesToCache(&mut self) {
        for i in 32..(32 * 8) {
            self.set_main_color(i, 0);
        }
        for i in 0..8 {
            for base in [0x00, 0x08, 0x10, 0x18, 0xd8, 0xe8, 0xf0, 0xf8] {
                let color = self.game_state.display.palette_buffer.aux_color(base + i);
                self.set_main_color(base + i, color);
            }
        }
        self.set_mosaic_copy(0xf7);
        self.set_mosaic_level(0xf7);
        self.increment_cgram_update_flag();
    }

    pub(super) fn Overworld_CopyPalettesToCache(&mut self) {
        self.overworld_copy_palettes_to_cache();
    }

    pub(super) fn Overworld_LoadPalettes(&mut self, bg: u8, spr: u8) {
        self.clear_overworld_aux_or_main_offset();
        let bg_base = bg as usize * 3;
        if OW_BG_PAL_INFO[bg_base] >= 0 {
            self.world_palette_theme_mut()
                .set_overworld_palette_aux1_hi(OW_BG_PAL_INFO[bg_base] as u8);
        }
        if OW_BG_PAL_INFO[bg_base + 1] >= 0 {
            self.set_overworld_palette_aux2_hi(OW_BG_PAL_INFO[bg_base + 1] as u8);
        }
        if OW_BG_PAL_INFO[bg_base + 2] >= 0 {
            self.set_overworld_palette_aux3_lo(OW_BG_PAL_INFO[bg_base + 2] as u8);
        }
        let spr_base = spr as usize * 2;
        if OW_SPR_PAL_INFO[spr_base] >= 0 {
            self.set_sp5l(OW_SPR_PAL_INFO[spr_base] as u8);
        }
        if OW_SPR_PAL_INFO[spr_base + 1] >= 0 {
            self.set_sp6l(OW_SPR_PAL_INFO[spr_base + 1] as u8);
        }
        self.palette_load_ow_bg1();
        self.palette_load_ow_bg2();
        self.palette_load_ow_bg3();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
    }

    pub(super) fn Palette_BgAndFixedColor_Black(&mut self) {
        self.palette_bg_and_fixed_color_black();
    }

    pub(super) fn Palette_SetBgAndFixedColor(&mut self, color: u16) {
        self.set_main_color(0, color);
        self.set_main_color(32, color);
        self.set_aux_color(0, color);
        self.set_aux_color(32, color);
        self.set_backdrop_color_black();
    }

    pub(super) fn SetBackdropcolorBlack(&mut self) {
        self.set_backdrop_color_black();
    }

    pub(super) fn Palette_SetOwBgColor(&mut self) {
        let color = self.Palette_GetOwBgColor();
        self.Palette_SetBgAndFixedColor(color);
    }

    pub(super) fn Palette_SpecialOw(&mut self) {
        let color = self.Palette_GetOwBgColor();
        self.set_aux_color(0, color);
        self.set_aux_color(32, color);
        self.set_backdrop_color_black();
    }

    pub(super) fn Palette_GetOwBgColor(&self) -> u16 {
        self.palette_get_ow_bg_color()
    }

    pub(super) fn Palette_AssertTranslucencySwap(&mut self) {
        self.Palette_SetTranslucencySwap(true);
    }

    pub(super) fn Palette_SetTranslucencySwap(&mut self, value: bool) {
        self.follower_state_mut().set_palette_swap_flag(value as u8);
        for i in 0..8 {
            for (a_base, b_base) in [(0x80, 0xf0), (0x88, 0xf8), (0xb8, 0xd8)] {
                let a = self.game_state.display.palette_buffer.aux_color(a_base + i);
                let b = self.game_state.display.palette_buffer.aux_color(b_base + i);
                self.set_aux_color(b_base + i, a);
                self.set_main_color(b_base + i, a);
                self.set_aux_color(a_base + i, b);
                self.set_main_color(a_base + i, b);
            }
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn Palette_RevertTranslucencySwap(&mut self) {
        self.Palette_SetTranslucencySwap(false);
    }

    pub(super) fn LoadActualGearPalettes(&mut self) {
        self.load_actual_gear_palettes();
    }

    pub(super) fn Palette_ElectroThemedGear(&mut self) {
        self.palette_electro_themed_gear();
    }

    pub(super) fn LoadGearPalettes_bunny(&mut self) {
        self.load_gear_palettes_bunny();
    }

    pub(super) fn LoadGearPalettes(&mut self, sword: u8, shield: u8, armor: u8) {
        self.load_gear_palettes(sword, shield, armor);
    }

    pub(super) fn LoadGearPalette(
        &mut self,
        asset: usize,
        color_offset: usize,
        dst: usize,
        x_ents: usize,
    ) {
        self.load_gear_palette_from_asset(asset, color_offset, dst, x_ents);
    }

    pub(super) fn Filter_Majorly_Whiten_Bg(&mut self) {
        self.filter_majorly_whiten_bg();
    }

    pub(super) fn Filter_Majorly_Whiten_Color(&self, color: u16) -> u16 {
        self.filter_majorly_whiten_color(color)
    }

    pub(super) fn Palette_Restore_BG_From_Flash(&mut self) {
        self.palette_restore_bg_from_flash();
    }

    pub(super) fn Palette_Restore_Coldata(&mut self) {
        self.palette_restore_coldata();
    }

    pub(super) fn Palette_Restore_BG_And_HUD(&mut self) {
        self.palette_restore_bg_and_hud();
    }

    pub(super) fn Palette_Load_Sp0L(&mut self) {
        self.palette_load_sp0l();
    }

    pub(super) fn Palette_Load_SpriteMain(&mut self) {
        self.palette_load_sprite_main();
    }

    pub(super) fn Palette_Load_Sp5L(&mut self) {
        self.palette_load_sp5l();
    }

    pub(super) fn Palette_Load_Sp6L(&mut self) {
        self.palette_load_sp6l();
    }

    pub(super) fn Palette_Load_Sword(&mut self) {
        self.palette_load_sword();
    }

    pub(super) fn Palette_Load_Shield(&mut self) {
        self.palette_load_shield();
    }

    pub(super) fn Palette_Load_SpriteEnvironment(&mut self) {
        self.palette_load_sprite_environment();
    }

    pub(super) fn Palette_Load_SpriteEnvironment_Dungeon(&mut self) {
        self.palette_load_sprite_environment_dungeon();
    }

    pub(super) fn Palette_MiscSprite_Outdoors(&mut self) {
        self.palette_misc_sprite_outdoors();
    }

    pub(super) fn Palette_Load_DungeonMapSprite(&mut self) {
        if let Some(palette) = self.asset_raw(91).map(Vec::from) {
            let aux_or_main = self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize;
            for pal in 0..=2 {
                for i in 0..=6 {
                    let color = read_word_from_slice(&palette, (pal * 7 + i) * 2);
                    let idx = ((0x182 + aux_or_main) >> 1) + pal * 0x10 + i;
                    self.set_aux_color(idx, color);
                }
            }
        }
    }

    pub(super) fn Palette_Load_LinkArmorAndGloves(&mut self) {
        self.palette_load_link_armor_and_gloves();
    }

    pub(super) fn Palette_UpdateGlovesColor(&mut self) {
        self.palette_update_gloves_color();
    }

    pub(super) fn Palette_Load_DungeonMapBG(&mut self) {
        if let Some(palette) = self.asset_raw(90).map(Vec::from) {
            let aux_or_main = self
                .game_state
                .display
                .palette_buffer
                .overworld_aux_or_main_offset() as usize;
            for pal in 0..=5 {
                for i in 0..=15 {
                    let color = read_word_from_slice(&palette, (pal * 16 + i) * 2);
                    let idx = ((0x40 + aux_or_main) >> 1) + pal * 0x10 + i;
                    self.set_aux_color(idx, color);
                }
            }
        }
    }

    pub(super) fn Palette_Load_HUD(&mut self) {
        self.palette_load_hud();
    }

    pub(super) fn Palette_Load_DungeonSet(&mut self) {
        self.palette_load_dungeon_set();
    }

    pub(super) fn Palette_Load_OWBG3(&mut self) {
        self.palette_load_ow_bg3();
    }

    pub(super) fn Palette_Load_OWBGMain(&mut self) {
        self.palette_load_ow_bg_main();
    }

    pub(super) fn Palette_Load_OWBG1(&mut self) {
        self.palette_load_ow_bg1();
    }

    pub(super) fn Palette_Load_OWBG2(&mut self) {
        self.palette_load_ow_bg2();
    }

    pub(super) fn Palette_LoadSingle(&mut self, src: u32, dst: usize, x_ents: usize) {
        self.palette_load_single(src, dst, x_ents);
    }

    pub(super) fn Palette_LoadMultiple(
        &mut self,
        src: u32,
        dst: usize,
        x_ents: usize,
        y_pals: usize,
    ) {
        self.palette_load_multiple(src, dst, x_ents, y_pals);
    }

    pub(super) fn Palette_LoadMultiple_Arbitrary(&mut self, src: u32, dst: usize, x_ents: usize) {
        self.palette_load_multiple_arbitrary_snes(src, dst, x_ents);
    }

    pub(super) fn Palette_LoadForFileSelect(&mut self) {
        self.palette_load_for_file_select();
    }

    pub(super) fn Palette_LoadForFileSelect_Armor(&mut self, k: usize, armor: u8, gloves: u8) {
        self.palette_load_for_file_select_armor(k, armor, gloves);
    }

    pub(super) fn Palette_LoadForFileSelect_Sword(&mut self, k: usize, sword: u8) {
        self.palette_load_for_file_select_sword(k, sword);
    }

    pub(super) fn Palette_LoadForFileSelect_Shield(&mut self, k: usize, shield: u8) {
        self.palette_load_for_file_select_shield(k, shield);
    }

    pub(super) fn Palette_LoadAgahnim(&mut self) {
        let src = PALETTE_SPRITE_AUX1_SNES_ADDR + 14 * 7 * 2;
        self.palette_load_multiple_arbitrary_snes(src, 0x162, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x182, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x1a2, 6);
        self.palette_load_multiple_arbitrary_snes(
            PALETTE_SPRITE_AUX1_SNES_ADDR + 21 * 7 * 2,
            0x1c2,
            6,
        );
        self.increment_cgram_update_flag();
    }

    pub(super) fn HandleScreenFlash(&mut self) {
        let j = self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count();
        if j == 0 || self.game_state.frame.submodule != 0 {
            return;
        }
        self.attract_scene_mut()
            .decrement_intro_palette_flash_count();
        if self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count()
            == 0
        {
            self.palette_restore_bg_and_hud();
            return;
        }
        if j & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn asset_bytes(&self, asset: usize, index: usize) -> Option<&[u8]> {
        Some(self.asset_memblk(asset, index)?.ptr)
    }
}

impl ZeldaState {
    pub(super) fn handle_screen_flash(&mut self) {
        let flash = self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count();
        if flash == 0 || self.game_state.frame.submodule != 0 {
            return;
        }
        self.attract_scene_mut()
            .set_intro_palette_flash_count(flash.wrapping_sub(1));
        if self
            .game_state
            .ending
            .attract_scene
            .intro_palette_flash_count()
            == 0
        {
            self.palette_restore_bg_and_hud();
            return;
        }
        if flash & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn apply_palette_filter_bounce(&mut self) {
        self.palette_filter_range(0, 1);
        self.palette_filter_range(0x20, 0xd8);
        self.palette_filter_range(0xe0, 0xf0);
        self.increment_cgram_update_flag();

        let countdown = self.game_state.display.palette_filter.countdown_word();
        let target = self.game_state.display.mosaic_target_level as u16;
        if self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word()
            == 0
        {
            let next = countdown.wrapping_add(1);
            self.set_countdown_word(next);
            if next != target {
                return;
            }
        } else {
            self.set_countdown_word(countdown.wrapping_sub(1));
            if countdown != target {
                return;
            }
        }
        let mode = self
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word()
            ^ 2;
        self.set_darkening_or_lightening_screen_word(mode);
        self.set_countdown_word(0);
        self.increment_subsubmodule();
    }

    pub(super) fn spotlight_open(&mut self) {
        self.spotlight_internal(0, 2);
    }

    pub(super) fn spotlight_internal(&mut self, x: u8, y: u8) {
        self.set_spotlight_window_radius(x as u16);
        self.set_spotlight_window_state(y as u16);
        self.hdma_setup(0x00f2fb, 0x00f2fb, 0x41, 0x26, 0x26, 0);
        self.set_bg12_window_selection(0x33);
        self.set_bg34_window_selection(3);
        self.set_object_color_window_selection(0x33);
        let main_screen = self.game_state.display.main_screen_layers;
        self.set_main_screen_window_layers(main_screen);
        let sub_screen = self.game_state.display.sub_screen_layers;
        self.set_sub_screen_window_layers(sub_screen);
        if self.game_state.world.location.is_outdoors() {
            self.set_fixed_color_red(0x20);
            self.set_fixed_color_green(0x40);
            self.set_fixed_color_blue(0x80);
        }
        self.iris_spotlight_configure_table();
        self.set_hdma_enable_mask(0x80);
        self.set_screen_brightness(0x0f);
    }

    pub(super) fn iris_spotlight_configure_table(&mut self) {
        let r14 = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
            .wrapping_add(12);
        let radius = self.game_state.display.spotlight_hdma.window_radius();
        self.set_spotlight_y_lower(r14.wrapping_sub(radius));
        self.set_spotlight_y_upper(r14.wrapping_add(radius));
        let x_center = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
            .wrapping_add(8);
        self.set_spotlight_window_x_center(x_center);
        self.set_spotlight_window_y_buffer(radius);

        let mut r6 = r14.wrapping_mul(2);
        if r6 < 224 {
            r6 = 224;
        }
        let mut r4 = r14.wrapping_mul(2).wrapping_sub(r6);
        loop {
            let mut r8 = 0x00ff;
            if r6 < self.game_state.display.spotlight_hdma.y_upper() {
                let t = self
                    .game_state
                    .display
                    .spotlight_hdma
                    .window_y_buffer_byte();
                if self.game_state.display.spotlight_hdma.window_y_buffer() != 0 {
                    self.decrement_spotlight_window_y_buffer();
                }
                r8 = self.iris_spotlight_calculate_circle_value(t);
            }
            if r4 < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(r4 as usize, r8);
            }
            if r6 < 240 {
                self.set_spotlight_hdma_table_dynamic_entry(r6 as usize, r8);
            }
            if r4 == r14 {
                break;
            }
            r4 = r4.wrapping_add(1);
            r6 = r6.wrapping_sub(1);
        }

        self.clear_spotlight_hdma_table_dynamic_range(224, 16);
        self.project_spotlight_dynamic_hdma_table_to_reserved(224);

        let idx = (self.game_state.display.spotlight_hdma.window_state() >> 1) as usize;
        let delta = SPOTLIGHT_DELTA_SIZE[idx] as i16 as u16;
        let next = self
            .game_state
            .display
            .spotlight_hdma
            .window_radius()
            .wrapping_add(delta);
        self.set_spotlight_window_radius(next);
        if next == SPOTLIGHT_GOAL[idx] {
            if self.game_state.display.spotlight_hdma.window_state() == 0 {
                self.set_screen_brightness(0x80);
            } else {
                self.iris_spotlight_reset_table();
            }
            self.set_subsubmodule(0);
            self.set_submodule(0);
            let main_module = self.game_state.frame.main_module;
            if main_module == 7 || main_module == 16 {
                if self.game_state.world.location.is_outdoors() {
                    let ambient = self.overworld_config_table().current_music() >> 4;
                    self.set_ambient_sound_effect(ambient);
                }
                if self.game_state.system_signals.queued_music_control() != 0xff {
                    let music = self.game_state.system_signals.queued_music_control();
                    self.set_music_control(music);
                }
            }
            let saved_module = self.game_state.frame.saved_module_for_menu;
            self.set_main_module(saved_module);
            if self.game_state.frame.main_module == 6 {
                self.sprite_reset_all();
            }
        }
    }

    pub(super) fn iris_spotlight_calculate_circle_value(&self, a: u8) -> u16 {
        let radius = self.game_state.display.spotlight_hdma.window_radius() as u8;
        let div = if radius == 0 {
            0xffff
        } else {
            ((a as u16) << 8) / radius as u16
        };
        let t = (div >> 1) as usize;
        let r10 = SPOTLIGHT_CIRCLE_X_RADIUS_CURVE[t];
        let p = 2 * (((r10 as u16) * (radius as u16) >> 8) as u8 as u16);
        if r10 == 0 {
            return 0x00ff;
        }
        let x_center = self.game_state.display.spotlight_hdma.window_x_center();
        let r2 = x_center.wrapping_add(p).min(255);
        let r0_raw = x_center.wrapping_sub(p);
        let r0 = if (r0_raw as i16) < 0 {
            0
        } else {
            r0_raw.min(255)
        };
        let result = r0 | (r2 << 8);
        if result == 0xffff {
            0x00ff
        } else {
            result
        }
    }

    pub(super) fn iris_spotlight_reset_table(&mut self) {
        for i in 0..240 {
            self.set_spotlight_hdma_table_dynamic_entry(i, 0xff00);
        }
    }

    pub(super) fn overworld_copy_palettes_to_cache(&mut self) {
        let aux = self
            .game_state
            .display
            .palette_buffer
            .aux_full_slice()
            .to_vec();
        self.copy_main_full_from(&aux);
        self.increment_cgram_update_flag();
    }

    pub(super) fn enable_force_blank(&mut self) {
        self.set_screen_brightness(0x80);
        self.clear_hdma_enable_mask();
    }

    // ----- Palette backdrop / OW background color helpers ---------------------
    // Port of zelda3/src/load_gfx.c:1820-1856 plus LoadGearPalette at 1913.

    pub(super) fn palette_bg_and_fixed_color_black(&mut self) {
        self.palette_set_bg_and_fixed_color(0);
    }

    pub(super) fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        self.set_main_color(0, color);
        self.set_main_color(32, color);
        self.set_aux_color(0, color);
        self.set_aux_color(32, color);
        self.set_backdropcolor_black();
    }

    pub(super) fn set_backdropcolor_black(&mut self) {
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
    }

    pub(super) fn palette_set_ow_bg_color(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.palette_set_bg_and_fixed_color(color);
    }

    pub(super) fn palette_special_ow(&mut self) {
        let c = self.palette_get_ow_bg_color();
        self.set_aux_color(0, c);
        self.set_aux_color(32, c);
        self.set_backdropcolor_black();
    }

    pub(super) fn palette_get_ow_bg_color(&self) -> u16 {
        let ow_screen = u16::from(self.game_state.world.location.overworld_screen_index());
        if ow_screen < 0x80 {
            return if ow_screen & 0x40 != 0 {
                0x2A32
            } else {
                0x2669
            };
        }
        let room = self.game_state.world.location.dungeon_room();
        if room == 0x180 || room == 0x182 || room == 0x183 {
            return 0x19C6;
        }
        0x2669
    }

    pub(super) fn palette_assert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(true);
    }

    pub(super) fn palette_set_translucency_swap(&mut self, v: bool) {
        self.follower_state_mut()
            .set_palette_swap_flag(if v { 1 } else { 0 });
        // Three banded swaps over 8 consecutive entries each: 0x80<->0xF0, 0x88<->0xF8, 0xB8<->0xD8.
        // For each pair (a from low band, b from high band), write `a` into the high
        // band slot of both buffers and `b` into the low band slot of both buffers.
        for i in 0..8 {
            let pairs: [(usize, usize); 3] = [
                (0x80 + i, 0xf0 + i),
                (0x88 + i, 0xf8 + i),
                (0xb8 + i, 0xd8 + i),
            ];
            for (low, high) in pairs {
                let a = self.game_state.display.palette_buffer.aux_color(low);
                let b = self.game_state.display.palette_buffer.aux_color(high);
                self.set_main_color(high, a);
                self.set_aux_color(high, a);
                self.set_main_color(low, b);
                self.set_aux_color(low, b);
            }
        }
        self.increment_cgram_update_flag();
    }

    pub(super) fn palette_revert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(false);
    }

    /// memcpy `n` u16 entries from `src` into both palette buffers at byte
    /// offset `dst`. Matches `LoadGearPalette(int dst, const uint16 *src, int n)`
    /// at load_gfx.c:1913; not called in the C codebase but ported for parity.
    pub(super) fn load_gear_palette(&mut self, dst: i32, src: &[u16], n: i32) {
        let base = (dst as usize) & !1;
        let base_idx = base >> 1;
        for i in 0..n as usize {
            let v = src[i];
            self.set_aux_color(base_idx + i, v);
            self.set_main_color(base_idx + i, v);
        }
    }
}
