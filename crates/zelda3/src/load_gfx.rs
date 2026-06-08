// Methods ported from zelda3/src/load_gfx.c and included inside ZeldaState.

use super::*;

const INCREMENTAL_COUNTER_FOR_VRAM: usize = 0x412;
const NMI_UPDATE_TILEMAP_DST_LOAD_GFX: usize = 0x19;
const NMI_UPDATE_TILEMAP_SRC_LOAD_GFX: usize = 0x118;
const STAR_TILE_RESTORE_PHASE: usize = 0x4bc;
const TRINEXX_RED_SHELL_PALETTE_DELAY: usize = 0x4be;
const TRINEXX_BLUE_SHELL_PALETTE_DELAY: usize = 0x4bf;
const TRINEXX_RED_SHELL_PALETTE_STEP: usize = 0x4c0;
const TRINEXX_BLUE_SHELL_PALETTE_STEP: usize = 0x4c1;
const WATERGATE_SPOTLIGHT_Y_UPPER: usize = 0x678;
const WATER_HDMA_WINDOW_X: usize = 0x680;
const WATER_HDMA_WINDOW_Y: usize = 0x682;
const WATER_HDMA_WINDOW_Y_RADIUS: usize = 0x684;
const WATER_HDMA_WINDOW_X_RADIUS: usize = 0x686;
const MIRROR_VARS_LOAD_GFX: usize = 0x06a0;
const MIRROR_CTR2_LOAD_GFX: usize = MIRROR_VARS_LOAD_GFX + 0x1a;
const MIRROR_CTR_LOAD_GFX: usize = MIRROR_VARS_LOAD_GFX + 0x1b;
const MESSAGING_BUF_LOAD_GFX: usize = 0x10000;
const BG_DECOMP_BUFFER_LOAD_GFX: usize = 0x6000;
const SPRITE_DECOMP_BUFFER_LOAD_GFX: usize = 0x7800;
const GRAPHICS_DECOMP_BUFFER_END: usize = 0x9000;

const DECODE_ANIMATED_SPRITE_TILE_TAB: [usize; 57] = [
    0x9c0, 0x30, 0x60, 0x90, 0xc0, 0x300, 0x318, 0x330, 0x348, 0x360, 0x378, 0x390, 0x930, 0x3f0,
    0x420, 0x450, 0x468, 0x600, 0x630, 0x660, 0x690, 0x6c0, 0x6f0, 0x720, 0x750, 0x768, 0x900,
    0x930, 0x960, 0x990, 0x9f0, 0, 0xf0, 0xa20, 0xa50, 0x660, 0x600, 0x618, 0x630, 0x648, 0x678,
    0x6d8, 0x6a8, 0x708, 0x738, 0x768, 0x960, 0x900, 0x3c0, 0x990, 0x9a8, 0x9c0, 0x9d8, 0xa08,
    0xa38, 0x600, 0x630,
];

const OW_BG_PAL_INFO: [i8; 93] = [
    0, -1, 7, 0, 1, 7, 0, 2, 7, 0, 3, 7, 0, 4, 7, 0, 5, 7, 0, 6, 7, 7, 6, 5, 0, 8, 7, 0, 9, 7, 0,
    10, 7, 0, 11, 7, 0, -1, 7, 0, -1, 7, 3, 4, 7, 4, 4, 3, 16, -1, 6, 16, 1, 6, 16, 17, 6, 16, 3,
    6, 16, 4, 6, 16, 5, 6, 16, 6, 6, 18, 19, 4, 18, 5, 4, 16, 9, 6, 16, 11, 6, 16, 12, 6, 16, 13,
    6, 16, 14, 6, 16, 15, 6,
];

const OW_SPR_PAL_INFO: [i8; 40] = [
    -1, -1, 3, 10, 3, 6, 3, 1, 0, 2, 3, 14, 3, 2, 19, 1, 11, 12, 17, 1, 7, 5, 17, 0, 9, 11, 15, 5,
    3, 5, 3, 7, 15, 2, 10, 2, 5, 1, 12, 14,
];
const GRAPHICS_LOAD_SP6: [i8; 20] = [
    10, -1, 3, -1, 0, -1, -1, -1, 1, -1, 2, -1, 0, -1, -1, -1, -1, -1, -1, -1,
];

const VARIOUS_PACKS_LOAD_GFX: [u8; 16] = [
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x5b, 0x01, 0x5a, 0x42, 0x43, 0x44, 0x45, 0x3f, 0x59, 0x0b, 0x5a,
];

const MIRROR_WARP_LOAD_NEXT_NMI_LOAD: [u8; 15] =
    [0, 14, 15, 16, 17, 0, 0, 0, 0, 0, 0, 18, 19, 20, 0];

const MAIN_TILESETS: [[u8; 8]; 37] = [
    [0, 1, 16, 6, 14, 31, 24, 15],
    [0, 1, 16, 8, 14, 34, 27, 15],
    [0, 1, 16, 6, 14, 31, 24, 15],
    [0, 1, 19, 7, 14, 35, 28, 15],
    [0, 1, 16, 7, 14, 33, 24, 15],
    [0, 1, 16, 9, 14, 32, 25, 15],
    [2, 3, 18, 11, 14, 33, 26, 15],
    [0, 1, 17, 12, 14, 36, 27, 15],
    [0, 1, 17, 8, 14, 34, 27, 15],
    [0, 1, 17, 12, 14, 37, 26, 15],
    [0, 1, 17, 12, 14, 38, 27, 15],
    [0, 1, 20, 10, 14, 39, 29, 15],
    [0, 1, 17, 10, 14, 40, 30, 15],
    [2, 3, 18, 11, 14, 41, 22, 15],
    [0, 1, 21, 13, 14, 42, 24, 15],
    [0, 1, 16, 7, 14, 35, 28, 15],
    [0, 1, 19, 7, 14, 4, 5, 15],
    [0, 1, 19, 7, 14, 4, 5, 15],
    [0, 1, 16, 9, 14, 32, 27, 15],
    [0, 1, 16, 9, 14, 42, 23, 15],
    [2, 3, 18, 11, 14, 33, 28, 15],
    [0, 8, 17, 27, 34, 46, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [58, 59, 60, 61, 83, 77, 62, 91],
    [66, 67, 68, 69, 32, 43, 63, 93],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [0, 8, 16, 24, 32, 43, 93, 91],
    [113, 114, 113, 114, 32, 43, 93, 91],
    [58, 59, 60, 61, 83, 77, 62, 91],
    [66, 67, 68, 69, 32, 43, 63, 89],
    [0, 114, 113, 114, 32, 43, 93, 15],
    [22, 57, 29, 23, 64, 65, 57, 30],
    [0, 70, 57, 114, 64, 65, 57, 15],
];

const AUX_TILESETS: [[u8; 4]; 82] = [
    [6, 0, 31, 24],
    [8, 0, 34, 27],
    [6, 0, 31, 24],
    [7, 0, 35, 28],
    [7, 0, 33, 24],
    [9, 0, 32, 25],
    [11, 0, 33, 26],
    [12, 0, 36, 25],
    [8, 0, 34, 27],
    [12, 0, 37, 27],
    [12, 0, 38, 27],
    [10, 0, 39, 29],
    [10, 0, 40, 30],
    [11, 0, 41, 22],
    [13, 0, 42, 24],
    [7, 0, 35, 28],
    [7, 0, 4, 5],
    [7, 0, 4, 5],
    [9, 0, 32, 27],
    [9, 0, 42, 23],
    [11, 0, 33, 28],
    [9, 0, 32, 25],
    [11, 0, 33, 26],
    [9, 0, 36, 27],
    [8, 0, 34, 27],
    [9, 0, 37, 27],
    [9, 0, 38, 27],
    [10, 0, 39, 29],
    [9, 0, 40, 30],
    [12, 0, 41, 22],
    [13, 0, 42, 23],
    [114, 0, 43, 93],
    [0, 0, 0, 0],
    [0, 87, 76, 0],
    [0, 86, 79, 0],
    [0, 83, 77, 0],
    [0, 82, 73, 0],
    [0, 85, 74, 0],
    [0, 83, 84, 0],
    [0, 81, 78, 0],
    [0, 0, 0, 0],
    [0, 80, 75, 0],
    [0, 83, 77, 0],
    [0, 85, 84, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 71, 72, 0],
    [0, 0, 0, 0],
    [0, 87, 76, 0],
    [0, 86, 79, 0],
    [0, 83, 77, 0],
    [0, 82, 73, 0],
    [0, 85, 74, 0],
    [0, 83, 84, 0],
    [0, 81, 78, 0],
    [0, 0, 0, 0],
    [0, 80, 75, 0],
    [0, 83, 0, 0],
    [0, 53, 54, 0],
    [0, 96, 52, 0],
    [0, 43, 44, 0],
    [0, 45, 46, 0],
    [0, 47, 48, 0],
    [0, 55, 56, 0],
    [0, 51, 52, 0],
    [0, 49, 50, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [114, 113, 114, 113],
    [23, 64, 65, 57],
];

const SPRITE_TILESETS: [[u8; 4]; 144] = [
    [0, 73, 0, 0],
    [70, 73, 12, 29],
    [72, 73, 19, 29],
    [70, 73, 19, 14],
    [72, 73, 12, 17],
    [72, 73, 12, 16],
    [79, 73, 74, 80],
    [14, 73, 74, 17],
    [70, 73, 18, 0],
    [0, 73, 0, 80],
    [0, 73, 0, 17],
    [72, 73, 12, 0],
    [0, 0, 55, 54],
    [72, 73, 76, 17],
    [93, 44, 12, 68],
    [0, 0, 78, 0],
    [15, 0, 18, 16],
    [0, 0, 0, 76],
    [0, 13, 23, 0],
    [22, 13, 23, 27],
    [22, 13, 23, 20],
    [21, 13, 23, 21],
    [22, 13, 24, 25],
    [22, 13, 23, 25],
    [22, 13, 0, 0],
    [22, 13, 24, 27],
    [15, 73, 74, 17],
    [75, 42, 92, 21],
    [22, 73, 23, 29],
    [0, 0, 0, 21],
    [22, 13, 23, 16],
    [22, 73, 18, 0],
    [22, 73, 12, 17],
    [0, 0, 18, 16],
    [22, 13, 0, 17],
    [22, 73, 12, 0],
    [22, 13, 76, 17],
    [14, 13, 74, 17],
    [22, 26, 23, 27],
    [79, 52, 74, 80],
    [53, 77, 101, 54],
    [74, 52, 78, 0],
    [14, 52, 74, 17],
    [81, 52, 93, 89],
    [75, 73, 76, 17],
    [45, 0, 0, 0],
    [93, 0, 18, 89],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [71, 73, 43, 45],
    [70, 73, 28, 82],
    [0, 73, 28, 82],
    [93, 73, 0, 82],
    [70, 73, 19, 82],
    [75, 77, 74, 90],
    [71, 73, 28, 82],
    [75, 77, 57, 54],
    [31, 44, 46, 82],
    [31, 44, 46, 29],
    [47, 44, 46, 82],
    [47, 44, 46, 49],
    [31, 30, 48, 82],
    [81, 73, 19, 0],
    [79, 73, 19, 80],
    [79, 77, 74, 80],
    [75, 73, 76, 43],
    [31, 32, 34, 83],
    [85, 61, 66, 67],
    [31, 30, 35, 82],
    [31, 30, 57, 58],
    [31, 30, 58, 62],
    [31, 30, 60, 61],
    [64, 30, 39, 63],
    [85, 26, 66, 67],
    [31, 30, 42, 82],
    [31, 30, 56, 82],
    [31, 32, 40, 82],
    [31, 32, 38, 82],
    [31, 44, 37, 82],
    [31, 32, 39, 82],
    [31, 30, 41, 82],
    [31, 44, 59, 82],
    [70, 73, 36, 82],
    [33, 65, 69, 51],
    [31, 44, 40, 49],
    [31, 13, 41, 82],
    [31, 30, 39, 82],
    [31, 32, 39, 83],
    [72, 73, 19, 82],
    [14, 30, 74, 80],
    [31, 32, 38, 83],
    [21, 0, 0, 0],
    [31, 0, 42, 82],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [50, 0, 0, 8],
    [93, 73, 0, 82],
    [85, 73, 66, 67],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 86, 87, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
    [97, 86, 87, 80],
    [97, 86, 99, 80],
    [97, 86, 87, 80],
    [97, 86, 51, 80],
    [97, 86, 87, 80],
    [97, 98, 99, 80],
    [97, 98, 99, 80],
];

fn main_tileset(index: usize) -> [u8; 8] {
    MAIN_TILESETS.get(index).copied().unwrap_or([0; 8])
}

fn aux_tileset(index: usize) -> [u8; 4] {
    AUX_TILESETS.get(index).copied().unwrap_or([0; 4])
}

fn sprite_tileset(index: usize) -> [u8; 4] {
    SPRITE_TILESETS.get(index).copied().unwrap_or([0; 4])
}

impl ZeldaState {
    pub(super) fn world_map_setup_hdma(&mut self) {
        write_le_u16(&mut self.ram, BG1HOFS_COPY2, 0x0080);
        write_le_u16(&mut self.ram, BG1VOFS_COPY2, 0x00c8);
        write_le_u16(&mut self.ram, M7Y_COPY, 0x01c9);
        write_le_u16(&mut self.ram, M7X_COPY, 0x0100);
        self.ram[W12SEL_COPY] = 0;
        self.ram[W34SEL_COPY] = 0;
        self.ram[WOBJSEL_COPY] = 0;
        self.ram[TMW_COPY] = 0;
        self.ram[TSW_COPY] = 0;
        if self.frame_control_view().main_module() == 20 {
            self.hdma_setup(0x0abddd, 0x0abddd, 0x42, 0x1b, 0x1e, 0);
            self.ram[HDMAEN_COPY] = 0xc0;
        } else if self.frame_control_view().submodule() != 10 {
            self.ram[MODE7_ZOOM_STEP_COUNTER] = 4;
            self.ram[TIMER_FOR_MODE7_ZOOM] = 12;
            self.ram[OVERWORLD_MAP_FLAGS] = 1;
            let y = ((read_le_u16(&self.ram, LINK_Y_COORD_SPEXIT) >> 4).wrapping_sub(0x48)) & !1;
            write_le_u16(&mut self.ram, BG1VOFS_COPY2, y);
            write_le_u16(&mut self.ram, M7Y_COPY, y.wrapping_add(0x100));
            let t0 = (read_le_u16(&self.ram, LINK_X_COORD_SPEXIT) >> 4).wrapping_sub(0x80);
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
            write_le_u16(&mut self.ram, BG1HOFS_COPY2, t2.wrapping_add(0x80) & !1);
            self.OverworldMap_SetupHdma();
            self.ram[HDMAEN_COPY] = 0xc0;
        } else {
            self.ram[MODE7_ZOOM_STEP_COUNTER] = 4;
            self.ram[TIMER_FOR_MODE7_ZOOM] = 33;
            self.ram[OVERWORLD_MAP_FLAGS] = 0;
            self.hdma_setup(0x0abdcf, 0x0abdcf, 0x42, 0x1b, 0x1e, 10);
            self.ram[HDMAEN_COPY] = 0xc0;
        }
    }

    pub(super) fn load_overworld_map_palette(&mut self) {
        if let Some(palette) = self.asset_raw(93).map(Vec::from) {
            let offset = if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
                0x100
            } else {
                0
            };
            let len = 0x100.min(palette.len().saturating_sub(offset));
            self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + len]
                .copy_from_slice(&palette[offset..offset + len]);
        }
    }

    pub(super) fn load_actual_gear_palettes(&mut self) {
        let sword = self.ram[LINK_SWORD_TYPE];
        self.load_gear_palettes(sword, self.ram[LINK_SHIELD_TYPE], self.ram[LINK_ARMOR]);
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
            self.palette_update_gloves_color();
        }
    }

    pub(super) fn palette_electro_themed_gear(&mut self) {
        self.load_gear_palettes(2, 2, 4);
    }

    pub(super) fn load_gear_palettes_bunny(&mut self) {
        self.load_gear_palettes(self.ram[LINK_SWORD_TYPE], self.ram[LINK_SHIELD_TYPE], 3);
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
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_update_gloves_color(&mut self) {
        if self.ram[LINK_ITEM_GLOVES] != 0 {
            let color = self.gloves_color(self.ram[LINK_ITEM_GLOVES].wrapping_sub(1) as usize);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 0xfd * 2, color);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 0xfd * 2, color);
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
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
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + (0xe8 + i) * 2, color);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (0xe8 + i) * 2, color);

            let color = read_word_from_slice(&main_sprite_palette, (15 + 7 + i) * 2);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + (0xf8 + i) * 2, color);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (0xf8 + i) * 2, color);
        }
    }

    pub(super) fn palette_load_for_file_select_armor(&mut self, k: usize, armor: u8, gloves: u8) {
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor as usize * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (k + 0x81 + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (k + 0x81 + i) * 2,
                color,
            );
        }
        if gloves != 0 {
            let color = self.gloves_color(gloves.wrapping_sub(1) as usize);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + (k + 0x8d) * 2, color);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (k + 0x8d) * 2, color);
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
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (k + 0x99 + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (k + 0x99 + i) * 2,
                color,
            );
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
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (k + 0x9c + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (k + 0x9c + i) * 2,
                color,
            );
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
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
        }
    }

    pub(super) fn reset_hud_palettes_4_and_5(&mut self) {
        for i in 0..8 {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (16 + i) * 2, 0);
        }
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 2);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_filter_history(&mut self) {
        self.palette_filter_range(0x10, 0x18);
        self.palette_filter_incr_countdown();
    }

    pub(super) fn palette_filter_range(&mut self, from: usize, to: usize) {
        const PALETTE_FILTERING_BITS: [u16; 64] = [
            0xffff, 0xffff, 0xfffe, 0xffff, 0x7fff, 0x7fff, 0x7fdf, 0xfbff, 0x7f7f, 0x7f7f, 0x7df7,
            0xefbf, 0x7bdf, 0x7bdf, 0x77bb, 0xddef, 0x7777, 0x7777, 0x6edd, 0xbb77, 0x6db7, 0x6db7,
            0x5b6d, 0xb6db, 0x5b5b, 0x5b5b, 0x56b6, 0xad6b, 0x5555, 0xad6b, 0x5555, 0xaaab, 0x5555,
            0x5555, 0x2a55, 0x5555, 0x2a55, 0x2a55, 0x294a, 0x5295, 0x2525, 0x2525, 0x2492, 0x4925,
            0x1249, 0x1249, 0x1122, 0x4489, 0x1111, 0x1111, 0x0844, 0x2211, 0x0421, 0x0421, 0x0208,
            0x1041, 0x0101, 0x0101, 0x0020, 0x0401, 0x0001, 0x0001, 0x0000, 0x0001,
        ];
        const UPPER_BITMASKS: [u16; 16] = [
            0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020,
            0x0010, 0x0008, 0x0004, 0x0002, 0x0001,
        ];

        let countdown = read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN);
        let load_ptr_offset = usize::from(countdown >= 0x10);
        let mask = UPPER_BITMASKS[(countdown & 0x0f) as usize];
        let dt = if read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN) != 0 {
            1u16
        } else {
            0xffff
        };

        for j in from..to {
            let main = MAIN_PALETTE_BUFFER + j * 2;
            let aux = AUX_PALETTE_BUFFER + j * 2;
            let mut c = read_le_u16(&self.ram, main);
            let a = read_le_u16(&self.ram, aux);
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x001f) as usize) * 2] & mask == 0 {
                c = c.wrapping_add(dt);
            }
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x03e0) >> 4) as usize] & mask == 0 {
                c = c.wrapping_add(dt.wrapping_shl(5));
            }
            if PALETTE_FILTERING_BITS[load_ptr_offset + ((a & 0x7c00) >> 9) as usize] & mask == 0 {
                c = c.wrapping_add(dt.wrapping_shl(10));
            }
            write_le_u16(&mut self.ram, main, c);
        }
    }

    pub(super) fn palette_filter_incr_countdown(&mut self) {
        let countdown = read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
        if countdown == 0x1f {
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
            let mode = read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN) ^ 2;
            write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, mode);
            if mode != 0 {
                let value = read_le_u16(&self.ram, ATTRACT_LEGEND_FLAG).wrapping_add(1);
                write_le_u16(&mut self.ram, ATTRACT_LEGEND_FLAG, value);
            }
        } else {
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, countdown);
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_fade_intro_one_step(&mut self) {
        self.palette_filter_restore_additive(0x100, 0x1a0);
        self.palette_filter_restore_additive(0x0c0, 0x100);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_sub(1);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_fade_intro2(&mut self) {
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.palette_filter_restore_additive(0x40, 0xc0);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_sub(1);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_filter_restore_additive(&mut self, from: usize, to: usize) {
        let mut i = from >> 1;
        let end = to >> 1;
        while i != end {
            let c = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            let d = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2);
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
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, cx);
            i += 1;
        }
    }

    pub(super) fn filter_majorly_whiten_bg(&mut self) {
        for i in 32..128 {
            let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2);
            let white = self.filter_majorly_whiten_color(color);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, white);
        }
        let color0 = if read_le_u16(&self.ram, AUX_PALETTE_BUFFER) != 0 {
            read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2)
        } else {
            0
        };
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color0);
    }

    fn filter_majorly_whiten_color(&self, color: u16) -> u16 {
        const FEATURES0_DIM_FLASHES: u32 = 65536;
        let amt = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_DIM_FLASHES != 0 {
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
            let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, color);
        }
        let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        self.palette_restore_coldata();
    }

    pub(super) fn palette_restore_bg_and_hud(&mut self) {
        let src = AUX_PALETTE_BUFFER;
        let dst = MAIN_PALETTE_BUFFER;
        for i in 0..0x100 {
            self.ram[dst + i] = self.ram[src + i];
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        self.palette_restore_coldata();
    }

    pub(super) fn palette_restore_coldata(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            return;
        }
        let rgb = match self.ram[OVERWORLD_SCREEN_INDEX] {
            3 | 5 | 7 => 0x8c4c26u32,
            0x43 | 0x45 | 0x47 => 0x874a26,
            0x5b => 0x894f33,
            _ => 0x804020,
        };
        self.ram[COLDATA_COPY0] = rgb as u8;
        self.ram[COLDATA_COPY1] = (rgb >> 8) as u8;
        self.ram[COLDATA_COPY2] = (rgb >> 16) as u8;
    }

    pub(super) fn overworld_load_all_palettes(&mut self) {
        self.ram[AUX_PALETTE_BUFFER + 0x180..AUX_PALETTE_BUFFER + 0x200].fill(0);
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 0x200].fill(0);

        self.ram[OVERWORLD_PALETTE_MODE] = 5;
        self.ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = 3;
        self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 3;
        self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 0;
        self.ram[PALETTE_SP6R_INDOORS] = 5;
        self.ram[PALETTE_SP0L] = 11;
        self.ram[PALETTE_SWAP_FLAG] = 0;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);

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
            let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (0xe8 + i) * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (0xd8 + i) * 2, color);
        }
    }

    pub(super) fn dungeon_load_palettes(&mut self) {
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
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
        self.ram[OVERWORLD_PAL_UNK1] = self.ram[PALETTE_MAIN_INDOORS];
        self.ram[OVERWORLD_PAL_UNK2] = self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO];
        self.ram[OVERWORLD_PAL_UNK3] = self.ram[PALETTE_MAIN_INDOORS_COPY];
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 2);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        write_le_u16(&mut self.ram, MOSAIC_TARGET_LEVEL, 0);
        self.overworld_copy_palettes_to_cache();
    }

    pub(super) fn palette_load_sp0l(&mut self) {
        let src = K_PALETTE_SPRITE_AUX3 + self.ram[PALETTE_SP0L] as u32 * 7 * 2;
        let dst = if self.ram[PALETTE_SWAP_FLAG] != 0 {
            0x1e2
        } else {
            0x102
        };
        self.palette_load_single(src, dst, 6);
    }

    pub(super) fn palette_load_sp5l(&mut self) {
        let src = K_PALETTE_SPRITE_AUX1 + self.ram[PALETTE_SP5L] as u32 * 7 * 2;
        self.palette_load_single(src, 0x1a2, 6);
    }

    pub(super) fn palette_load_sp6l(&mut self) {
        let src = K_PALETTE_SPRITE_AUX1 + self.ram[PALETTE_SP6L] as u32 * 7 * 2;
        self.palette_load_single(src, 0x1c2, 6);
    }

    pub(super) fn palette_load_sword(&mut self) {
        let sword = self.ram[LINK_SWORD_TYPE];
        let sword_index = if (sword as i8) > 0 {
            sword.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(82, sword_index * 3, 0x1b2, 2);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_load_shield(&mut self) {
        let shield = self.ram[LINK_SHIELD_TYPE];
        let shield_index = if shield != 0 {
            shield.wrapping_sub(1) as usize
        } else {
            0
        };
        self.load_gear_palette_from_asset(83, shield_index * 4, 0x1b8, 3);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_load_sprite_main(&mut self) {
        let offset = if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
            60 * 2
        } else {
            0
        };
        self.palette_load_multiple(K_PALETTE_MAIN_SPR + offset, 0x122, 14, 3);
    }

    pub(super) fn palette_load_ow_bg_main(&mut self) {
        let src = K_PALETTE_OVERWORLD_BG_MAIN + self.ram[OVERWORLD_PALETTE_MODE] as u32 * 35 * 2;
        self.palette_load_multiple(src, 0x42, 6, 4);
    }

    pub(super) fn palette_load_ow_bg1(&mut self) {
        let src = K_PALETTE_OVERWORLD_BG_AUX12
            + self.ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] as u32 * 21 * 2;
        self.palette_load_multiple(src, 0x52, 6, 2);
    }

    pub(super) fn palette_load_ow_bg2(&mut self) {
        let src = K_PALETTE_OVERWORLD_BG_AUX12
            + self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] as u32 * 21 * 2;
        self.palette_load_multiple(src, 0xb2, 6, 2);
    }

    pub(super) fn palette_load_ow_bg3(&mut self) {
        let src =
            K_PALETTE_OVERWORLD_BG_AUX3 + self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] as u32 * 7 * 2;
        self.palette_load_single(src, 0xe2, 6);
    }

    pub(super) fn palette_load_sprite_environment_dungeon(&mut self) {
        let src = K_PALETTE_MISC_SPRITE_INDOORS + self.ram[PALETTE_SP6R_INDOORS] as u32 * 7 * 2;
        self.palette_load_single(src, 0x1d2, 6);
    }

    pub(super) fn palette_load_sprite_environment(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            self.palette_load_sprite_environment_dungeon();
        } else {
            self.palette_misc_sprite_outdoors();
        }
    }

    pub(super) fn palette_misc_sprite_outdoors(&mut self) {
        let t = if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
            9
        } else {
            7
        };
        let src = K_PALETTE_MISC_SPRITE_INDOORS + t * 7 * 2;
        let dst = if self.ram[PALETTE_SWAP_FLAG] != 0 {
            0x1f2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
        self.palette_load_single(src - 7 * 2, 0x1d2, 6);
    }

    pub(super) fn palette_load_hud(&mut self) {
        let src = K_HUD_PAL_DATA + self.ram[HUD_PALETTE] as u32 * 32 * 2;
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
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
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
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + (dst_index + i) * 2,
                color,
            );
        }
    }

    pub(super) fn palette_load_dungeon_set(&mut self) {
        let src = K_PALETTE_DUNG_BG_MAIN + (self.ram[PALETTE_MAIN_INDOORS] >> 1) as u32 * 90 * 2;
        self.palette_load_multiple(src, 0x42, 14, 5);
        let dst = if self.ram[PALETTE_SWAP_FLAG] != 0 {
            0x1e2
        } else {
            0x112
        };
        self.palette_load_single(src, dst, 6);
    }

    pub(super) fn palette_load_link_armor_and_gloves(&mut self) {
        let armor = self.ram[LINK_ARMOR] as usize;
        let Some(palette) = self.asset_raw(81).map(Vec::from) else {
            return;
        };
        let src = armor * 15 * 2;
        for i in 0..15 {
            let color = read_word_from_slice(&palette, src + i * 2);
            write_le_u16(
                &mut self.ram,
                AUX_PALETTE_BUFFER + ((0x1e2 >> 1) + i) * 2,
                color,
            );
            write_le_u16(
                &mut self.ram,
                MAIN_PALETTE_BUFFER + ((0x1e2 >> 1) + i) * 2,
                color,
            );
        }
        self.palette_update_gloves_color();
    }

    pub(super) fn palette_load_single(&mut self, src: u32, dst: usize, x_ents: usize) {
        let base =
            ((dst + read_le_u16(&self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) as usize) >> 1) * 2;
        for i in 0..=x_ents {
            let Some(color) = self.rom_or_asset_word_snes(src + i as u32 * 2) else {
                return;
            };
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + base + i * 2, color);
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

        self.decomp_spr_to_ram(DECOMP_BUFFER, 95);
        dst = self.load_item_animation_gfx_one(dst, 4, 3, true);
        dst = self.load_item_animation_gfx_one(dst, 3, 4, true);
        dst = self.load_item_animation_gfx_one(dst, 1, 5, true);
        dst = self.load_item_animation_gfx_one(dst, 4, 6, false);

        self.decomp_spr_to_ram(DECOMP_BUFFER, 96);
        dst = self.load_item_animation_gfx_one(dst, 14, 7, true);
        dst = self.load_item_animation_gfx_one(dst, 7, 8, true);

        self.decomp_spr_to_ram(DECOMP_BUFFER, 95);
        self.load_item_animation_gfx_one(dst, 2, 9, true);
        self.decomp_spr_to_ram(DECOMP_BUFFER, 84);

        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.expand3_to_4_high_from_slice(0xa480, &tmp, 0, 0, 8);
        self.expand3_to_4_high_from_slice(0xa580, &tmp, 0x180, 0, 8);

        self.decomp_spr_to_ram(DECOMP_BUFFER, 96);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
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
        const INTRO_LOAD_GFX_TAB: [usize; 10] = [0, 11, 8, 38, 42, 45, 34, 3, 33, 46];
        let Some(source) = (if from_temp {
            Some(self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec())
        } else {
            self.asset_bytes(64, 0).map(Vec::from)
        }) else {
            return dst + 0x40 * num;
        };

        let src = INTRO_LOAD_GFX_TAB[r12] * 24;
        self.expand3_to_4_high_from_slice(dst, &source, src, 0, num);
        self.expand3_to_4_high_from_slice(dst + 0x20 * num, &source, src + 0x180, 0, num);
        dst + 0x40 * num
    }

    pub(super) fn load_item_gfx_auxiliary(&mut self) {
        self.decomp_bg_to_ram(DECOMP_BUFFER, 0x0f);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(PEG_TILE_GFX_BUFFER, &tmp, 0, 16);

        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x58);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xb540, &tmp, 0, 32);

        self.decomp_bg_to_ram(DECOMP_BUFFER, 0x05);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xbdc0, &tmp, 0x480, 2);
    }

    pub(super) fn load_follower_graphics(&mut self) {
        const TAGALONG_WHICH: [usize; 14] = [
            0, 0x600, 0x300, 0x300, 0x300, 0, 0, 0x900, 0x600, 0x600, 0x900, 0x900, 0x600, 0x900,
        ];

        let follower = self.ram[FOLLOWER_INDICATOR] as usize;
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

        self.decomp_spr_to_ram(DECOMP_BUFFER_SECOND, yv);
        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x65);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
        let offset = TAGALONG_WHICH.get(follower).copied().unwrap_or(0);
        self.do3_to_4_low_16bit_from_slice(0xb940, &tmp, offset, 0x20);
    }

    pub(super) fn decompress_sword_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-sword");
        const SWORD_TYPE_TO_GFX_OFFS: [usize; 5] = [0, 0, 0x120, 0x120, 0x120];
        self.decomp_spr_to_ram(DECOMP_BUFFER_SECOND, 0x5f);
        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x5e);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        let sword = self.ram[LINK_SWORD_TYPE] as usize;
        let src = SWORD_TYPE_TO_GFX_OFFS.get(sword).copied().unwrap_or(0);
        self.expand3_to_4_high_from_slice(0x9000, &tmp, src, 0, 12);
        self.expand3_to_4_high_from_slice(0x9180, &tmp, src + 0x180, 0, 12);
        self.replay_trace_ram_watch("loadgfx-after-decompress-sword");
    }

    pub(super) fn decompress_shield_graphics(&mut self) {
        self.replay_trace_ram_watch("loadgfx-before-decompress-shield");
        const SHIELD_TYPE_TO_GFX_OFFS: [usize; 4] = [0x660, 0x660, 0x6f0, 0x900];
        self.decomp_spr_to_ram(DECOMP_BUFFER_SECOND, 0x5f);
        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x5e);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
        let shield = self.ram[LINK_SHIELD_TYPE] as usize;
        let src = SHIELD_TYPE_TO_GFX_OFFS
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
        self.do3_to_4_high_to_vram(0x4000, &source);
        self.decomp_and_upload_2bpp(0x7000, 0x6a);
        self.decomp_and_upload_2bpp(0x7400, 0x6b);
        self.decomp_and_upload_2bpp(0x7800, 0x69);
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

    pub(super) fn decomp_and_upload_2bpp(&mut self, dst: usize, pack: usize) {
        let Some(data) = self.decomp_spr_data(pack) else {
            return;
        };
        let len = data.len().min(self.ram.len().saturating_sub(DECOMP_BUFFER));
        self.ram[DECOMP_BUFFER..DECOMP_BUFFER + len].copy_from_slice(&data[..len]);
        for i in 0..0x400 {
            self.ppu.vram[dst + i] = read_word_from_slice(&data, i * 2);
        }
    }

    pub(super) fn initialize_tilesets(&mut self) {
        let main_tileset = main_tileset(self.ram[MAIN_TILE_THEME_INDEX] as usize);
        let aux_tileset = aux_tileset(self.ram[AUX_TILE_THEME_INDEX] as usize);
        let sprite_tileset = sprite_tileset(self.ram[SPRITE_GRAPHICS_INDEX] as usize);

        self.load_common_sprites();

        if sprite_tileset[0] != 0 {
            self.ram[SPRITE_GFX_SUBSET_0] = sprite_tileset[0];
        }
        if sprite_tileset[1] != 0 {
            self.ram[SPRITE_GFX_SUBSET_1] = sprite_tileset[1];
        }
        if sprite_tileset[2] != 0 {
            self.ram[SPRITE_GFX_SUBSET_2] = sprite_tileset[2];
        }
        if sprite_tileset[3] != 0 {
            self.ram[SPRITE_GFX_SUBSET_3] = sprite_tileset[3];
        }

        self.load_sprite_graphics(0x5000, self.ram[SPRITE_GFX_SUBSET_0] as usize, 0x7800);
        self.load_sprite_graphics(0x5400, self.ram[SPRITE_GFX_SUBSET_1] as usize, 0x7e00);
        self.load_sprite_graphics(0x5800, self.ram[SPRITE_GFX_SUBSET_2] as usize, 0x8400);
        self.load_sprite_graphics(0x5c00, self.ram[SPRITE_GFX_SUBSET_3] as usize, 0x8a00);

        self.ram[AUX_BG_SUBSET_0] = if aux_tileset[0] != 0 {
            aux_tileset[0]
        } else {
            main_tileset[3]
        };
        self.ram[AUX_BG_SUBSET_1] = if aux_tileset[1] != 0 {
            aux_tileset[1]
        } else {
            main_tileset[4]
        };
        self.ram[AUX_BG_SUBSET_2] = if aux_tileset[2] != 0 {
            aux_tileset[2]
        } else {
            main_tileset[5]
        };
        self.ram[AUX_BG_SUBSET_3] = if aux_tileset[3] != 0 {
            aux_tileset[3]
        } else {
            main_tileset[6]
        };

        self.load_background_graphics(0x2000, main_tileset[0] as usize, 7, DECOMP_BUFFER);
        self.load_background_graphics(0x2400, main_tileset[1] as usize, 6, DECOMP_BUFFER);
        self.load_background_graphics(0x2800, main_tileset[2] as usize, 5, DECOMP_BUFFER);
        self.load_background_graphics(0x2c00, self.ram[AUX_BG_SUBSET_0] as usize, 4, 0x6000);
        self.load_background_graphics(0x3000, self.ram[AUX_BG_SUBSET_1] as usize, 3, 0x6600);
        self.load_background_graphics(0x3400, self.ram[AUX_BG_SUBSET_2] as usize, 2, 0x6c00);
        self.load_background_graphics(0x3800, self.ram[AUX_BG_SUBSET_3] as usize, 1, 0x7200);
        self.load_background_graphics(0x3c00, main_tileset[7] as usize, 0, DECOMP_BUFFER);
    }

    pub(super) fn load_common_sprites(&mut self) {
        let Some(data) = self
            .asset_bytes(64, self.ram[MISC_SPRITES_GRAPHICS_INDEX] as usize)
            .map(Vec::from)
        else {
            return;
        };
        self.do3_to_4_high_to_vram(0x4400, &data);

        if self.frame_control_view().main_module() == 1 {
            self.load_sprite_graphics(0x4800, 94, DECOMP_BUFFER);
            self.load_sprite_graphics(0x4c00, 95, DECOMP_BUFFER);
            return;
        }

        let Some(data) = self.asset_bytes(64, 6).map(Vec::from) else {
            return;
        };
        self.do3_to_4_low_to_vram(0x4800, &data);

        let Some(data) = self.asset_bytes(64, 7).map(Vec::from) else {
            return;
        };
        self.do3_to_4_low_to_vram(0x4c00, &data);
    }

    pub(super) fn load_sprite_graphics(&mut self, dst: usize, gfx_pack: usize, decomp_dst: usize) {
        let Some(data) = self.decomp_spr_data(gfx_pack) else {
            return;
        };
        let len = data.len().min(self.ram.len().saturating_sub(decomp_dst));
        self.ram[decomp_dst..decomp_dst + len].copy_from_slice(&data[..len]);
        if matches!(gfx_pack, 0x52 | 0x53 | 0x5a | 0x5b | 0x5c | 0x5e | 0x5f) {
            self.do3_to_4_high_to_vram(dst, &data);
        } else {
            self.do3_to_4_low_to_vram(dst, &data);
        }
    }

    pub(super) fn load_background_graphics(
        &mut self,
        dst: usize,
        gfx_pack: usize,
        slot: usize,
        decomp_dst: usize,
    ) {
        let Some(data) = self.decomp_bg_data(gfx_pack) else {
            return;
        };
        let len = data.len().min(self.ram.len().saturating_sub(decomp_dst));
        self.ram[decomp_dst..decomp_dst + len].copy_from_slice(&data[..len]);
        let high = if self.ram[MAIN_TILE_THEME_INDEX] >= 0x20 {
            matches!(slot, 7 | 2 | 3 | 4)
        } else {
            slot >= 4
        };
        if high {
            self.do3_to_4_high_to_vram(dst, &data);
        } else {
            self.do3_to_4_low_to_vram(dst, &data);
        }
    }

    pub(super) fn decompress_animated_dungeon_tiles(&mut self, pack: usize) {
        self.decomp_bg_to_ram(DECOMP_BUFFER, pack);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xa680, &tmp, 0, 48);

        self.decomp_bg_to_ram(DECOMP_BUFFER, 0x5c);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xac80, &tmp, 0, 48);

        for i in 0..256 {
            let base = 0x9000 + i * 2;
            let x = read_le_u16(&self.ram, base + 0x1880);
            let a = read_le_u16(&self.ram, base + 0x1c80);
            let b = read_le_u16(&self.ram, base + 0x1e80);
            let c = read_le_u16(&self.ram, base + 0x1a80);
            write_le_u16(&mut self.ram, base + 0x1880, a);
            write_le_u16(&mut self.ram, base + 0x1c80, b);
            write_le_u16(&mut self.ram, base + 0x1e80, c);
            write_le_u16(&mut self.ram, base + 0x1a80, x);
        }

        write_le_u16(&mut self.ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);
    }

    pub(super) fn decomp_spr_to_ram(&mut self, dst: usize, mut gfx: usize) -> usize {
        if gfx < 12 {
            gfx = 12;
        }
        let Some(data) = self.decomp_spr_data(gfx) else {
            return 0;
        };
        let len = data.len().min(self.ram.len().saturating_sub(dst));
        self.ram[dst..dst + len].copy_from_slice(&data[..len]);
        len
    }

    pub(super) fn decomp_bg_to_ram(&mut self, dst: usize, gfx: usize) -> usize {
        let Some(data) = self.decomp_bg_data(gfx) else {
            return 0;
        };
        let len = data.len().min(self.ram.len().saturating_sub(dst));
        self.ram[dst..dst + len].copy_from_slice(&data[..len]);
        len
    }

    pub(super) fn decomp_spr_data(&self, mut gfx: usize) -> Option<Vec<u8>> {
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

    pub(super) fn decomp_bg_data(&self, gfx: usize) -> Option<Vec<u8>> {
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
                self.ram[dst] = lo;
                self.ram[dst + 1] = hi;
                self.ram[dst + 0x10] = u;
                self.ram[dst + 0x11] = lo | hi | u;
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
            if (src.wrapping_sub(base) & 0x78) == 0 {
                src += 0x180;
            }
        }
    }

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
                self.ram[dst] = lo;
                self.ram[dst + 1] = hi;
                self.ram[dst + 0x10] = u;
                self.ram[dst + 0x11] = lo | hi | u;
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
                self.ram[dst] = data.get(src + i * 2).copied().unwrap_or(0);
                self.ram[dst + 1] = data.get(src + i * 2 + 1).copied().unwrap_or(0);
                self.ram[dst + 0x10] = data.get(src2 + i).copied().unwrap_or(0);
                self.ram[dst + 0x11] = 0;
                dst += 2;
            }
            dst += 16;
            src = src2 + 8;
        }
    }

    pub(super) fn do3_to_4_high_to_vram(&mut self, mut dst: usize, data: &[u8]) {
        let mut src = 0usize;
        let mut tmp = [0u8; 8];
        for _ in 0..64 {
            for i in (0..8).rev() {
                let lo = data.get(src).copied().unwrap_or(0);
                let hi = data.get(src + 1).copied().unwrap_or(0);
                self.ppu.vram[dst] = lo as u16 | ((hi as u16) << 8);
                tmp[i] = lo | hi;
                write_le_u16(&mut self.ram, DUNG_LINE_PTRS_ROW0 + i * 2, tmp[i] as u16);
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
    }

    pub(super) fn do3_to_4_low_to_vram(&mut self, mut dst: usize, data: &[u8]) {
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
    }

    pub(super) fn graphics_load_chr_half_slot(&mut self) {
        let k = self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD];
        if k == 0 {
            return;
        }

        let sp6 = GRAPHICS_LOAD_SP6[k as usize - 1];
        if sp6 >= 0 {
            self.ram[PALETTE_SP6R_INDOORS] = sp6 as u8;
            write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
            if k == 1 {
                self.ram[PALETTE_SP6R_INDOORS] = 10;
                self.Palette_Load_SpriteEnvironment();
            } else {
                self.Palette_Load_SpriteEnvironment_Dungeon();
            }
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
        }

        let mut tilebytes = 0x44;
        self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD].wrapping_add(1);
        if self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] & 1 != 0 {
            self.ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 0;
            if k != 18 {
                tilebytes = 0x46;
                if k == 2 {
                    self.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0;
                }
            }
        }

        self.ram[NMI_LOAD_TARGET_ADDR] = tilebytes;
        self.ram[NMI_SUBROUTINE_INDEX] = 11;

        let mut pack = GRAPHICS_HALF_SLOT_PACKS[k as usize - 1] as usize;
        if pack == 1 {
            pack = self.ram[MISC_SPRITES_GRAPHICS_INDEX] as usize;
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
                self.ram[dst] = lo;
                self.ram[dst + 1] = hi;
                self.ram[dst + 16] = mask;
                self.ram[dst + 17] = lo | hi | mask;
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
        self.decomp_and_upload_2bpp(vram_ptr, pack);
    }

    pub(super) fn RecoverPegGFXFromMapping(&mut self) {
        if self.ram[ORANGE_BLUE_BARRIER_STATE] != 0 {
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
        self.decomp_bg_to_ram(DECOMP_BUFFER, a as usize);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xa680, &tmp, 0, 64);

        self.decomp_bg_to_ram(DECOMP_BUFFER, a.wrapping_add(1) as usize);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0x600].to_vec();
        self.do3_to_4_low_16bit_from_slice(0xae80, &tmp, 0, 32);
        write_le_u16(&mut self.ram, ANIMATED_TILE_VRAM_ADDR, 0x3c00);
    }

    pub(super) fn LoadItemGFX_Auxiliary(&mut self) {
        self.load_item_gfx_auxiliary();
    }

    pub(super) fn LoadFollowerGraphics(&mut self) {
        self.load_follower_graphics();
    }

    pub(super) fn WriteTo4BPPBuffer_at_7F4000(&mut self, a: u8) {
        let src = DECODE_ANIMATED_SPRITE_TILE_TAB
            .get(a as usize)
            .copied()
            .unwrap_or(0);
        let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
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
        self.decomp_spr_to_ram(DECOMP_BUFFER_SECOND, y);
        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x5a);
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
        let p = aux_tileset(self.ram[AUX_TILE_THEME_INDEX] as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.ram[AUX_BG_SUBSET_0 + i] = pack;
                assert_eq!(
                    self.decomp_bg_to_ram(0x6000 + 0x600 * i, pack as usize),
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
        let p = sprite_tileset(self.ram[SPRITE_GRAPHICS_INDEX] as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.ram[SPRITE_GFX_SUBSET_0 + i] = pack;
            }
            assert_eq!(
                self.decomp_spr_to_ram(dst + 0x600 * i, self.ram[SPRITE_GFX_SUBSET_0 + i] as usize),
                0x600
            );
        }
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0;
    }

    pub(super) fn ReloadPreviouslyLoadedSheets(&mut self) {
        self.decomp_bg_to_ram(0x6000, self.ram[AUX_BG_SUBSET_0] as usize);
        self.decomp_bg_to_ram(0x6600, self.ram[AUX_BG_SUBSET_1] as usize);
        self.decomp_bg_to_ram(0x6c00, self.ram[AUX_BG_SUBSET_2] as usize);
        self.decomp_bg_to_ram(0x7200, self.ram[AUX_BG_SUBSET_3] as usize);
        self.decomp_spr_to_ram(0x7800, self.ram[SPRITE_GFX_SUBSET_0] as usize);
        self.decomp_spr_to_ram(0x7e00, self.ram[SPRITE_GFX_SUBSET_1] as usize);
        self.decomp_spr_to_ram(0x8400, self.ram[SPRITE_GFX_SUBSET_2] as usize);
        self.decomp_spr_to_ram(0x8a00, self.ram[SPRITE_GFX_SUBSET_3] as usize);
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0;
    }

    pub(super) fn Attract_DecompressStoryGFX(&mut self) {
        self.decomp_spr_to_ram(DECOMP_BUFFER, 0x67);
        self.decomp_spr_to_ram(0x14800, 0x68);
    }

    pub(super) fn AnimateMirrorWarp(&mut self) {
        let st = self.ram[OVERWORLD_MAP_STATE] as usize;
        self.ram[OVERWORLD_MAP_STATE] = self.ram[OVERWORLD_MAP_STATE].wrapping_add(1);
        let nmi = MIRROR_WARP_LOAD_NEXT_NMI_LOAD.get(st).copied().unwrap_or(0);
        self.ram[NMI_SUBROUTINE_INDEX] = nmi;
        self.ram[NMI_DISABLE_CORE_UPDATES] = nmi;
        let xt = if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
            8
        } else {
            0
        };

        match st {
            0 => {
                self.ram[MIRROR_CTR2_LOAD_GFX] = self.ram[MIRROR_CTR2_LOAD_GFX].wrapping_add(1);
                if self.ram[MIRROR_CTR2_LOAD_GFX] != 32 {
                    self.ram[OVERWORLD_MAP_STATE] = 0;
                } else {
                    self.SetTargetOverworldWarpToPyramid();
                }
            }
            1 => {
                self.AnimateMirrorWarp_DecompressNewTileSets();
                self.decomp_bg_to_ram(DECOMP_BUFFER, VARIOUS_PACKS_LOAD_GFX[xt] as usize);
                self.decomp_bg_to_ram(
                    DECOMP_BUFFER_SECOND,
                    VARIOUS_PACKS_LOAD_GFX[xt + 1] as usize,
                );
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                self.do3_to_4_high_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            2 => {
                self.decomp_bg_to_ram(DECOMP_BUFFER, VARIOUS_PACKS_LOAD_GFX[xt + 2] as usize);
                self.decomp_bg_to_ram(
                    DECOMP_BUFFER_SECOND,
                    VARIOUS_PACKS_LOAD_GFX[xt + 3] as usize,
                );
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                self.do3_to_4_low_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                self.do3_to_4_high_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            3 => {
                self.decomp_bg_to_ram(DECOMP_BUFFER, self.ram[AUX_BG_SUBSET_1] as usize);
                self.decomp_bg_to_ram(DECOMP_BUFFER_SECOND, self.ram[AUX_BG_SUBSET_2] as usize);
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                self.do3_to_4_high_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            4 => {
                self.decomp_bg_to_ram(DECOMP_BUFFER, VARIOUS_PACKS_LOAD_GFX[xt + 4] as usize);
                self.decomp_bg_to_ram(
                    DECOMP_BUFFER_SECOND,
                    VARIOUS_PACKS_LOAD_GFX[xt + 5] as usize,
                );
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                self.do3_to_4_low_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
            }
            5 => {
                self.PreOverworld_LoadOverlays();
                if matches!(self.ram[OVERWORLD_SCREEN_INDEX], 27 | 91) {
                    self.ram[TS_COPY] = 1;
                }
                self.frame_control_view_mut().decrement_submodule();
                self.ram[NMI_SUBROUTINE_INDEX] = 12;
                self.ram[NMI_DISABLE_CORE_UPDATES] = 12;
            }
            6 | 9 => {
                self.ram[NMI_SUBROUTINE_INDEX] = 13;
                self.ram[NMI_DISABLE_CORE_UPDATES] = 13;
            }
            7 => {
                self.Overworld_DrawScreenAtCurrentMirrorPosition();
                self.ram[NMI_DISABLE_CORE_UPDATES] =
                    self.ram[NMI_DISABLE_CORE_UPDATES].wrapping_add(1);
            }
            8 => {
                self.MirrorWarp_LoadSpritesAndColors();
                self.ram[NMI_SUBROUTINE_INDEX] = 12;
                self.ram[NMI_DISABLE_CORE_UPDATES] = 12;
            }
            10 => {
                let t = self.ram[OVERWORLD_SCREEN_INDEX] & 0xbf;
                self.DecompressAnimatedOverworldTiles(if matches!(t, 3 | 5 | 7) {
                    0x58
                } else {
                    0x5a
                });
            }
            11 => {
                let t = self.ram[OVERWORLD_SCREEN_INDEX];
                self.ram[TS_COPY] =
                    matches!(t, 0 | 0x70 | 0x40 | 0x5b | 3 | 5 | 7 | 0x43 | 0x45 | 0x47) as u8;
                let Some(data) = self
                    .asset_bytes(64, VARIOUS_PACKS_LOAD_GFX[xt + 6] as usize)
                    .map(Vec::from)
                else {
                    return;
                };
                self.do3_to_4_high_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &data, 0, 64);
            }
            12 => {
                self.decomp_spr_to_ram(DECOMP_BUFFER, self.ram[SPRITE_GFX_SUBSET_0] as usize);
                self.decomp_spr_to_ram(
                    DECOMP_BUFFER_SECOND,
                    self.ram[SPRITE_GFX_SUBSET_1] as usize,
                );
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                if matches!(self.ram[SPRITE_GFX_SUBSET_0], 0x52 | 0x53 | 0x5a | 0x5b) {
                    self.do3_to_4_high_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                } else {
                    self.do3_to_4_low_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 64);
                }
                self.do3_to_4_low_16bit_from_slice(NMI_BG_CHAR_BUFFER_1, &tmp, 0x600, 64);
            }
            13 => {
                self.decomp_spr_to_ram(DECOMP_BUFFER, self.ram[SPRITE_GFX_SUBSET_2] as usize);
                self.decomp_spr_to_ram(
                    DECOMP_BUFFER_SECOND,
                    self.ram[SPRITE_GFX_SUBSET_3] as usize,
                );
                let tmp = self.ram[DECOMP_BUFFER..DECOMP_BUFFER + 0xc00].to_vec();
                self.do3_to_4_low_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 128);
                self.handle_followers_after_mirroring();
            }
            14 => {
                self.ram[OVERWORLD_MAP_STATE] = 14;
            }
            _ => {}
        }
    }

    pub(super) fn AnimateMirrorWarp_DecompressNewTileSets(&mut self) {
        let main_tileset = main_tileset(self.ram[MAIN_TILE_THEME_INDEX] as usize);
        let aux_tileset = aux_tileset(self.ram[AUX_TILE_THEME_INDEX] as usize);
        self.ram[AUX_BG_SUBSET_0] = if aux_tileset[0] != 0 {
            aux_tileset[0]
        } else {
            main_tileset[3]
        };
        self.ram[AUX_BG_SUBSET_1] = if aux_tileset[1] != 0 {
            aux_tileset[1]
        } else {
            main_tileset[4]
        };
        self.ram[AUX_BG_SUBSET_2] = if aux_tileset[2] != 0 {
            aux_tileset[2]
        } else {
            main_tileset[5]
        };
        self.ram[AUX_BG_SUBSET_3] = if aux_tileset[3] != 0 {
            aux_tileset[3]
        } else {
            main_tileset[6]
        };

        let p = sprite_tileset(self.ram[SPRITE_GRAPHICS_INDEX] as usize);
        for (i, pack) in p.iter().copied().enumerate() {
            if pack != 0 {
                self.ram[SPRITE_GFX_SUBSET_0 + i] = pack;
            }
        }
    }

    pub(super) fn Graphics_IncrementalVRAMUpload(&mut self) {
        const DST: [u8; 16] = [
            0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d,
            0x5e, 0x5f,
        ];
        const SRC: [u16; 16] = [
            0x0000, 0x0200, 0x0400, 0x0600, 0x0800, 0x0a00, 0x0c00, 0x0e00, 0x1000, 0x1200, 0x1400,
            0x1600, 0x1800, 0x1a00, 0x1c00, 0x1e00,
        ];

        let k = self.ram[INCREMENTAL_COUNTER_FOR_VRAM] as usize;
        if k == 16 {
            return;
        }
        self.ram[NMI_UPDATE_TILEMAP_DST_LOAD_GFX] = DST[k];
        write_le_u16(&mut self.ram, NMI_UPDATE_TILEMAP_SRC_LOAD_GFX, SRC[k]);
        self.ram[INCREMENTAL_COUNTER_FOR_VRAM] =
            self.ram[INCREMENTAL_COUNTER_FOR_VRAM].wrapping_add(1);
    }

    pub(super) fn PrepTransAuxGfx(&mut self) {
        let tmp = self.ram[BG_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec();
        self.do3_to_4_high_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0x40);
        if self.ram[AUX_TILE_THEME_INDEX] >= 32 {
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
        let tmp = self.ram[SPRITE_DECOMP_BUFFER_LOAD_GFX..GRAPHICS_DECOMP_BUFFER_END].to_vec();
        self.do3_to_4_low_16bit_from_slice(MESSAGING_BUF_LOAD_GFX, &tmp, 0, 0xc0);
        if matches!(self.ram[SPRITE_GFX_SUBSET_3], 0x52 | 0x53 | 0x5a | 0x5b) {
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
        self.decomp_and_upload_2bpp(0x7800, 0x67);
    }

    pub(super) fn Graphics_LoadChrHalfSlot(&mut self) {
        self.graphics_load_chr_half_slot();
    }

    pub(super) fn TransferFontToVRAM(&mut self) {
        self.transfer_font_to_vram();
    }

    pub(super) fn Do3To4High(&mut self, vram_ptr: usize, decomp_addr: &[u8]) {
        self.do3_to_4_high_to_vram(vram_ptr, decomp_addr);
    }

    pub(super) fn Do3To4Low(&mut self, vram_ptr: usize, decomp_addr: &[u8]) {
        self.do3_to_4_low_to_vram(vram_ptr, decomp_addr);
    }

    pub(super) fn LoadSpriteGraphics(
        &mut self,
        vram_ptr: usize,
        gfx_pack: usize,
        decomp_addr: usize,
    ) {
        self.load_sprite_graphics(vram_ptr, gfx_pack, decomp_addr);
    }

    pub(super) fn LoadBackgroundGraphics(
        &mut self,
        vram_ptr: usize,
        gfx_pack: usize,
        slot: usize,
        decomp_addr: usize,
    ) {
        self.load_background_graphics(vram_ptr, gfx_pack, slot, decomp_addr);
    }

    pub(super) fn LoadCommonSprites(&mut self) {
        self.load_common_sprites();
    }

    pub(super) fn Decomp_spr(&mut self, dst: usize, gfx: usize) -> usize {
        self.decomp_spr_to_ram(dst, gfx)
    }

    pub(super) fn Decomp_bg(&mut self, dst: usize, gfx: usize) -> usize {
        self.decomp_bg_to_ram(dst, gfx)
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
        self.ram[TS_COPY] = 2;
        self.ram[CGADSUB_COPY] = 0x30;
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn PaletteFilter_Crystal(&mut self) {
        self.ram[TS_COPY] = 1;
        self.PaletteFilter_WishPonds_Inner();
    }

    pub(super) fn PaletteFilter_WishPonds_Inner(&mut self) {
        for i in 0..8 {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (0xd0 + i) * 2, 0);
        }
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 2);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_RestoreSP5F(&mut self) {
        for i in 0..8 {
            let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (208 + i) * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (208 + i) * 2, color);
        }
        self.ram[TS_COPY] = 0;
        self.ram[CGADSUB_COPY] = 32;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_SP5F(&mut self) {
        for _ in 0..2 {
            self.palette_filter_range(208, 216);
            self.palette_filter_incr_countdown();
            if read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN) == 0 {
                break;
            }
        }
    }

    pub(super) fn KholdstareShell_PaletteFiltering(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        let t = if self.read_u32_ram(ENHANCED_FEATURES0) & FEATURES0_MISC_BUG_FIXES != 0 {
            0x50
        } else {
            0x40
        };
        if self.frame_control_view().subsubmodule() == 0 {
            for i in 0..8 {
                let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (t + i) * 2);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (t + i) * 2, color);
            }
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
            write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 0);
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.frame_control_view_mut().set_subsubmodule(1);
            return;
        }
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            self.palette_filter_incr_countdown();
            if read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN) == 0 {
                self.ram[TS_COPY] = 0;
                break;
            }
        }
    }

    pub(super) fn AgahnimWarpShadowFilter(&mut self, k: usize) {
        const PALETTE_FILTER_AGAHNIM_TAB: [usize; 3] = [0x160, 0x180, 0x1a0];

        let pal_setting = AGAHNIM_PAL_SETTING + k * 2;
        let darkening_setting = AGAHNIM_PAL_SETTING + (k + 3) * 2;
        let pal_countdown = read_le_u16(&self.ram, pal_setting);
        let darkening_screen = read_le_u16(&self.ram, darkening_setting);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, pal_countdown);
        write_le_u16(
            &mut self.ram,
            DARKENING_OR_LIGHTENING_SCREEN,
            darkening_screen,
        );
        let t = PALETTE_FILTER_AGAHNIM_TAB[k] >> 1;
        for _ in 0..2 {
            self.palette_filter_range(t, t + 8);
            let countdown = read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN).wrapping_add(1);
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, countdown);
            if countdown == 0x1f {
                write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
                let screen = read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN) ^ 2;
                write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, screen);
                break;
            }
        }
        let pal_countdown = read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN);
        let darkening_screen = read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN);
        write_le_u16(&mut self.ram, pal_setting, pal_countdown);
        write_le_u16(&mut self.ram, darkening_setting, darkening_screen);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
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
            let c = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            let d = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2);
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
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, cx);
            i += 1;
        }
    }

    pub(super) fn PaletteFilter_InitializeWhiteFilter(&mut self) {
        for i in 0..256 {
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + i * 2, 0x7fff);
        }
        let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 32 * 2, color);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, 2);
        if self.ram[OVERWORLD_SCREEN_INDEX] == 27 {
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, 0);
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 32 * 2, 0);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, 0);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 32 * 2, 0);
        }
        self.ram[MIRROR_CTR_LOAD_GFX] = 8;
        self.ram[MIRROR_CTR2_LOAD_GFX] = 0;
    }

    pub(super) fn MirrorWarp_RunAnimationSubmodules(&mut self) {
        let ctr = self.ram[MIRROR_CTR_LOAD_GFX].wrapping_sub(1);
        self.ram[MIRROR_CTR_LOAD_GFX] = ctr;
        if ctr != 0 {
            self.AnimateMirrorWarp();
            return;
        }
        self.ram[MIRROR_CTR_LOAD_GFX] = 2;
        self.PaletteFilter_BlindingWhite();
    }

    pub(super) fn PaletteFilter_BlindingWhite(&mut self) {
        if self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 0xff {
            return;
        }
        if self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 2 {
            self.palette_filter_restore_additive(0x40, 0x1b0);
            self.palette_filter_restore_additive(0x1c0, 0x1e0);
        } else {
            self.PaletteFilter_RestoreSubtractive(0x40, 0x1b0);
            self.PaletteFilter_RestoreSubtractive(0x1c0, 0x1e0);
        }
        self.PaletteFilter_StartBlindingWhite();
    }

    pub(super) fn PaletteFilter_StartBlindingWhite(&mut self) {
        let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        if self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 0 {
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 66 {
                self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0xff;
                self.ram[MIRROR_CTR_LOAD_GFX] = 32;
            }
        } else {
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 31 {
                self.ram[DARKENING_OR_LIGHTENING_SCREEN] ^= 2;
                if self.frame_control_view().main_module() != 21 {
                    return;
                }
                self.ram[HDMAEN_COPY] = 0;
                for i in 0..240 {
                    write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0x0778);
                }
                self.ram[HDMAEN_COPY] = 0xc0;
            }
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_BlindingWhiteTriforce(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x200);
        self.PaletteFilter_StartBlindingWhite();
    }

    pub(super) fn PaletteFilter_WhirlpoolBlue(&mut self) {
        if self.ram[FRAME_COUNTER] & 1 != 0 {
            for i in 0x20..0x100 {
                let mut color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
                if (color & 0x7c00) != 0x7c00 {
                    color = color.wrapping_add(0x400);
                }
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, color);
            }
            let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
            if self.ram[PALETTE_FILTER_COUNTDOWN] & 1 == 0 {
                self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_add(16);
            }
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 31 {
                self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
                self.frame_control_view_mut().increment_subsubmodule();
                self.ram[MOSAIC_LEVEL] = 0xf0;
            }
        }
        self.ram[BGMODE_COPY] = 9;
        self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 3;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_IsolateWhirlpoolBlue(&mut self) {
        for i in 0x20..0x100 {
            let mut color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            if color & 0x03e0 != 0 {
                color = color.wrapping_sub(0x20);
            }
            if color & 0x001f != 0 {
                color = color.wrapping_sub(1);
            }
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, color);
        }
        let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
        if self.ram[PALETTE_FILTER_COUNTDOWN] == 31 {
            self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
            self.frame_control_view_mut().increment_subsubmodule();
            self.ram[MOSAIC_LEVEL] = 0xf0;
        }
        self.ram[BGMODE_COPY] = 9;
        self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 3;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_WhirlpoolRestoreBlue(&mut self) {
        if self.ram[FRAME_COUNTER] & 1 != 0 {
            for i in 0x20..0x100 {
                let aux = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2) & 0x7c00;
                let mut color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
                if color & 0x7c00 != aux {
                    color = color.wrapping_sub(0x400);
                }
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, color);
            }
            let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
            if self.ram[PALETTE_FILTER_COUNTDOWN] & 1 == 0 {
                self.ram[MOSAIC_LEVEL] = self.ram[MOSAIC_LEVEL].wrapping_sub(16);
            }
            self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
            if self.ram[PALETTE_FILTER_COUNTDOWN] == 31 {
                self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
                self.frame_control_view_mut().increment_subsubmodule();
                self.ram[MOSAIC_LEVEL] = 0;
            }
        }
        self.ram[BGMODE_COPY] = 9;
        self.ram[MOSAIC_COPY] = self.ram[MOSAIC_LEVEL] | 3;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_WhirlpoolRestoreRedGreen(&mut self) {
        for i in 0x20..0x100 {
            let aux = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + i * 2);
            let mut color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + i * 2);
            if color & 0x03e0 != aux & 0x03e0 {
                color = color.wrapping_add(0x20);
            }
            if color & 0x001f != aux & 0x001f {
                color = color.wrapping_add(1);
            }
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, color);
        }
        let color = read_le_u16(&self.ram, MAIN_PALETTE_BUFFER + 32 * 2);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
        if self.ram[PALETTE_FILTER_COUNTDOWN] == 31 {
            self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
            self.frame_control_view_mut().increment_subsubmodule();
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_RestoreBGSubstractiveStrict(&mut self) {
        if self.ram[DARKENING_OR_LIGHTENING_SCREEN] == 0xff {
            return;
        }
        self.PaletteFilter_RestoreSubtractive(0x40, 0x100);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
        if self.ram[PALETTE_FILTER_COUNTDOWN] == 0x20 {
            self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0xff;
            write_le_u16(&mut self.ram, TS_COPY, 0);
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn PaletteFilter_RestoreBGAdditiveStrict(&mut self) {
        self.palette_filter_restore_additive(0x40, 0x100);
        self.ram[PALETTE_FILTER_COUNTDOWN] = self.ram[PALETTE_FILTER_COUNTDOWN].wrapping_add(1);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn Trinexx_FlashShellPalette_Red(&mut self) {
        if self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] == 0 {
            for i in 0..7 {
                let addr = MAIN_PALETTE_BUFFER + (0x41 + i) * 2;
                let v = read_le_u16(&self.ram, addr);
                let red = (v & 0x1f).wrapping_add(u16::from((v & 0x1f) != 0x1f));
                write_le_u16(&mut self.ram, addr, (v & 0xffe0) | red);
            }
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] =
                self.ram[TRINEXX_RED_SHELL_PALETTE_STEP].wrapping_add(1);
            if self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] >= 12 {
                self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 0;
                self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 0;
                return;
            }
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 3;
        }
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY].wrapping_sub(1);
    }

    pub(super) fn Trinexx_UnflashShellPalette_Red(&mut self) {
        if self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] == 0 {
            for i in 0..7 {
                let addr = MAIN_PALETTE_BUFFER + (0x41 + i) * 2;
                let u = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (0x41 + i) * 2);
                let v = read_le_u16(&self.ram, addr);
                let red = (v & 0x1f).wrapping_sub(u16::from((v & 0x1f) != (u & 0x1f)));
                write_le_u16(&mut self.ram, addr, (v & 0xffe0) | red);
            }
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] =
                self.ram[TRINEXX_RED_SHELL_PALETTE_STEP].wrapping_add(1);
            if self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] >= 12 {
                self.ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 0;
                self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 0;
                return;
            }
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 3;
        }
        self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_RED_SHELL_PALETTE_DELAY].wrapping_sub(1);
    }

    pub(super) fn Trinexx_FlashShellPalette_Blue(&mut self) {
        if self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] == 0 {
            for i in 0..7 {
                let addr = MAIN_PALETTE_BUFFER + (0x41 + i) * 2;
                let v = read_le_u16(&self.ram, addr);
                let blue =
                    (v & 0x7c00).wrapping_add(if (v & 0x7c00) != 0x7c00 { 0x0400 } else { 0 });
                write_le_u16(&mut self.ram, addr, (v & !0x7c00) | blue);
            }
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] =
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP].wrapping_add(1);
            if self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] >= 12 {
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 0;
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 0;
                return;
            }
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 3;
        }
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY].wrapping_sub(1);
    }

    pub(super) fn Trinexx_UnflashShellPalette_Blue(&mut self) {
        if self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] == 0 {
            for i in 0..7 {
                let addr = MAIN_PALETTE_BUFFER + (0x41 + i) * 2;
                let u = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (0x41 + i) * 2);
                let v = read_le_u16(&self.ram, addr);
                let blue = (v & 0x7c00).wrapping_sub(if (v & 0x7c00) != (u & 0x7c00) {
                    0x0400
                } else {
                    0
                });
                write_le_u16(&mut self.ram, addr, (v & !0x7c00) | blue);
            }
            self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] =
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP].wrapping_add(1);
            if self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] >= 12 {
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 0;
                self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 0;
                return;
            }
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 3;
        }
        self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] =
            self.ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY].wrapping_sub(1);
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
        let r10 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        let var2 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS);
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_LOWER, r10.wrapping_sub(var2));
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_UPPER, r10.wrapping_add(var2));
        self.AdjustWaterHDMAWindow_X(r10);
    }

    pub(super) fn AdjustWaterHDMAWindow_X(&mut self, r10: u16) {
        let window_x_center = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_X_CENTER, window_x_center);
        let r12 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_RADIUS).saturating_sub(1);
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
                let a = if (read_le_u16(&self.ram, SPOTLIGHT_Y_LOWER) as i16) >= 0
                    && r4 < read_le_u16(&self.ram, SPOTLIGHT_Y_LOWER)
                {
                    0x00ff
                } else {
                    r12_pair
                };
                if r4 < 240 {
                    write_le_u16(
                        &mut self.ram,
                        HDMA_TABLE_DYNAMIC + r4 as usize * 2,
                        if a != 0xffff { a } else { 0x00ff },
                    );
                }
            }

            let a = if r6 >= read_le_u16(&self.ram, SPOTLIGHT_Y_UPPER) {
                0x00ff
            } else {
                if r6 >= 225 && read_le_u16(&self.ram, WATERGATE_SPOTLIGHT_Y_UPPER) != 0 {
                    let value = read_le_u16(&self.ram, WATERGATE_SPOTLIGHT_Y_UPPER).wrapping_sub(1);
                    write_le_u16(&mut self.ram, WATERGATE_SPOTLIGHT_Y_UPPER, value);
                }
                r12_pair
            };
            if r6 < 240 {
                write_le_u16(
                    &mut self.ram,
                    HDMA_TABLE_DYNAMIC + r6 as usize * 2,
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
        let lower = read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_LOWER, lower);
        let window_x_center = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_X_CENTER, window_x_center);
        let r14 = read_le_u16(&self.ram, WATER_HDMA_WINDOW_X_RADIUS) ^ 1;
        let mut r4 = 0usize;
        let upper = read_le_u16(&self.ram, SPOTLIGHT_Y_UPPER) as usize;
        while r4 != upper {
            write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + r4 * 2, 0xff00);
            r4 += 1;
        }
        let r12w = r14.wrapping_sub(7).wrapping_add(8);
        let r12_pair = (window_x_center.wrapping_add(r12w) << 8)
            | (window_x_center.wrapping_sub(r12w) & 0x00ff);
        let r10 = (read_le_u16(&self.ram, SPOTLIGHT_Y_UPPER)
            .wrapping_add(read_le_u16(&self.ram, WATER_HDMA_WINDOW_Y_RADIUS)))
            ^ 1;
        while r4 < 225 {
            if r4 as u16 >= r10 {
                write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + r4 * 2, 0x00ff);
            } else {
                let mut a = r4 as u16;
                loop {
                    a = a.wrapping_mul(2);
                    if a < 480 {
                        break;
                    }
                }
                write_le_u16(
                    &mut self.ram,
                    HDMA_TABLE_DYNAMIC + (a as usize >> 1) * 2,
                    if r12_pair == 0xffff { 0x00ff } else { r12_pair },
                );
            }
            r4 += 1;
        }
    }

    pub(super) fn ResetStarTileGraphics(&mut self) {
        self.ram[STAR_TILE_RESTORE_PHASE] = 0;
        self.Dungeon_RestoreStarTileChr();
    }

    pub(super) fn Dungeon_RestoreStarTileChr(&mut self) {
        let (xx, yy) = if self.ram[STAR_TILE_RESTORE_PHASE] != 0 {
            (32, 0)
        } else {
            (0, 32)
        };
        let src0 = 0xbdc0 + xx;
        let src1 = 0xbdc0 + yy;
        for i in 0..32 {
            self.ram[MESSAGING_BUF_LOAD_GFX + i] = self.ram[src0 + i];
            self.ram[MESSAGING_BUF_LOAD_GFX + 32 + i] = self.ram[src1 + i];
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 0x18;
    }

    pub(super) fn LinkZap_HandleMosaic(&mut self) {
        let mut level = self.ram[MOSAIC_LEVEL];
        if self.ram[MOSAIC_INC_OR_DEC] == 0 {
            level = level.wrapping_add(0x10);
            if level == 0xc0 {
                self.ram[MOSAIC_INC_OR_DEC] = 1;
            }
        } else {
            level = level.wrapping_sub(0x10);
            if level == 0 {
                self.ram[MOSAIC_INC_OR_DEC] = 0;
            }
        }
        self.ram[MOSAIC_LEVEL] = level;
        self.ram[MOSAIC_COPY] = (level >> 1) | 3;
        self.ram[BGMODE_COPY] = 9;
    }

    pub(super) fn Player_SetCustomMosaicLevel(&mut self, level: u8) {
        self.ram[MOSAIC_INC_OR_DEC] = 0;
        self.ram[MOSAIC_LEVEL] = level;
        self.ram[MOSAIC_COPY] = (level >> 1) | 3;
        self.ram[BGMODE_COPY] = 9;
    }

    pub(super) fn Module07_16_UpdatePegs_Step1(&mut self) {
        if self.ram[ORANGE_BLUE_BARRIER_STATE] != 0 {
            self.Dungeon_UpdatePegGFXBuffer(0x80, 0x100);
        } else {
            self.Dungeon_UpdatePegGFXBuffer(0x100, 0x80);
        }
    }

    pub(super) fn Module07_16_UpdatePegs_Step2(&mut self) {
        if self.ram[ORANGE_BLUE_BARRIER_STATE] != 0 {
            self.Dungeon_UpdatePegGFXBuffer(0x100, 0x80);
        } else {
            self.Dungeon_UpdatePegGFXBuffer(0x80, 0x100);
        }
    }

    pub(super) fn Dungeon_UpdatePegGFXBuffer(&mut self, x: usize, y: usize) {
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (x >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (y >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + (64 + i) * 2, color);
        }
        self.ram[NMI_SUBROUTINE_INDEX] = 23;
    }

    pub(super) fn Dungeon_HandleTranslucencyAndPalette(&mut self) {
        if self.ram[PALETTE_SWAP_FLAG] != 0 {
            self.Palette_RevertTranslucencySwap();
        }

        self.ram[CGWSEL_COPY] = 2;
        self.ram[CGADSUB_COPY] = 0xb3;

        let mut torch = self.ram[DUNG_NUM_LIT_TORCHES];
        if self.ram[DUNG_WANT_LIGHTS_OUT] == 0 {
            let mut a = 0x20;
            if self.ram[DUNG_HDR_BG2_PROPERTIES] != 0 {
                a = 0x32;
                if self.ram[DUNG_HDR_BG2_PROPERTIES] != 7 {
                    a = 0x62;
                    if self.ram[DUNG_HDR_BG2_PROPERTIES] != 4 {
                        a = 0x20;
                        if self.ram[DUNG_HDR_BG2_PROPERTIES] == 2 {
                            self.Palette_AssertTranslucencySwap();
                            if self.ram[DUNGEON_ROOM_INDEX] == 13 {
                                for i in 0..6 {
                                    self.ram[AGAHNIM_PAL_SETTING + i] = 0;
                                }
                                self.Palette_LoadAgahnim();
                            }
                            a = 0x70;
                        }
                    }
                }
            }
            self.ram[CGADSUB_COPY] = a;
            torch = 3;
        }
        const LIT_TORCHES_COLOR_PLUS: [u8; 4] = [31, 8, 4, 0];
        self.ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = LIT_TORCHES_COLOR_PLUS
            .get(torch as usize)
            .copied()
            .unwrap_or(0);
        self.ram[PALETTE_FILTER_COUNTDOWN] = 31;
        self.ram[MOSAIC_TARGET_LEVEL] = 0;
        self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 2;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
        self.palette_load_dungeon_set();
        self.palette_load_sp0l();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.frame_control_view_mut().increment_subsubmodule();
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
        let sc = self.ram[OVERWORLD_SCREEN_INDEX] & 0x3f;
        let mut x = if matches!(sc, 3 | 5 | 7) { 2 } else { 0 };
        if self.ram[OVERWORLD_SCREEN_INDEX] & 0x40 != 0 {
            x += 1;
        }
        self.Overworld_LoadAreaPalettesEx(x);
    }

    pub(super) fn Overworld_LoadAreaPalettesEx(&mut self, palette: u8) {
        self.ram[OVERWORLD_PALETTE_MODE] = palette;
        let aux_or_main = read_le_u16(&self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) & 0xff;
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, aux_or_main);
        self.palette_load_sprite_main();
        self.palette_load_sprite_environment();
        self.palette_load_sp5l();
        self.palette_load_sp6l();
        self.palette_load_sword();
        self.palette_load_shield();
        self.palette_load_link_armor_and_gloves();
        self.ram[PALETTE_SP0L] = if self.ram[SAVEGAME_IS_DARKWORLD] & 0x40 != 0 {
            3
        } else {
            1
        };
        self.palette_load_sp0l();
        self.palette_load_hud();
        self.palette_load_ow_bg_main();
    }

    pub(super) fn SpecialOverworld_CopyPalettesToCache(&mut self) {
        for i in 32..(32 * 8) {
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + i * 2, 0);
        }
        for i in 0..8 {
            for base in [0x00, 0x08, 0x10, 0x18, 0xd8, 0xe8, 0xf0, 0xf8] {
                let color = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (base + i) * 2);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (base + i) * 2, color);
            }
        }
        self.ram[MOSAIC_COPY] = 0xf7;
        self.ram[MOSAIC_LEVEL] = 0xf7;
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn Overworld_CopyPalettesToCache(&mut self) {
        self.overworld_copy_palettes_to_cache();
    }

    pub(super) fn Overworld_LoadPalettes(&mut self, bg: u8, spr: u8) {
        write_le_u16(&mut self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0);
        let bg_base = bg as usize * 3;
        if OW_BG_PAL_INFO[bg_base] >= 0 {
            self.ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = OW_BG_PAL_INFO[bg_base] as u8;
        }
        if OW_BG_PAL_INFO[bg_base + 1] >= 0 {
            self.ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = OW_BG_PAL_INFO[bg_base + 1] as u8;
        }
        if OW_BG_PAL_INFO[bg_base + 2] >= 0 {
            self.ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = OW_BG_PAL_INFO[bg_base + 2] as u8;
        }
        let spr_base = spr as usize * 2;
        if OW_SPR_PAL_INFO[spr_base] >= 0 {
            self.ram[PALETTE_SP5L] = OW_SPR_PAL_INFO[spr_base] as u8;
        }
        if OW_SPR_PAL_INFO[spr_base + 1] >= 0 {
            self.ram[PALETTE_SP6L] = OW_SPR_PAL_INFO[spr_base + 1] as u8;
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
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 32 * 2, color);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, color);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 32 * 2, color);
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
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, color);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 32 * 2, color);
        self.set_backdrop_color_black();
    }

    pub(super) fn Palette_GetOwBgColor(&self) -> u16 {
        self.palette_get_ow_bg_color()
    }

    pub(super) fn Palette_AssertTranslucencySwap(&mut self) {
        self.Palette_SetTranslucencySwap(true);
    }

    pub(super) fn Palette_SetTranslucencySwap(&mut self, value: bool) {
        self.ram[PALETTE_SWAP_FLAG] = value as u8;
        for i in 0..8 {
            for (a_base, b_base) in [(0x80, 0xf0), (0x88, 0xf8), (0xb8, 0xd8)] {
                let a = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (a_base + i) * 2);
                let b = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + (b_base + i) * 2);
                write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + (b_base + i) * 2, a);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (b_base + i) * 2, a);
                write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + (a_base + i) * 2, b);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + (a_base + i) * 2, b);
            }
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
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
            let aux_or_main = read_le_u16(&self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) as usize;
            for pal in 0..=2 {
                for i in 0..=6 {
                    let color = read_word_from_slice(&palette, (pal * 7 + i) * 2);
                    write_le_u16(
                        &mut self.ram,
                        AUX_PALETTE_BUFFER + (((0x182 + aux_or_main) >> 1) + pal * 0x10 + i) * 2,
                        color,
                    );
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
            let aux_or_main = read_le_u16(&self.ram, OVERWORLD_PALETTE_AUX_OR_MAIN) as usize;
            for pal in 0..=5 {
                for i in 0..=15 {
                    let color = read_word_from_slice(&palette, (pal * 16 + i) * 2);
                    write_le_u16(
                        &mut self.ram,
                        AUX_PALETTE_BUFFER + (((0x40 + aux_or_main) >> 1) + pal * 0x10 + i) * 2,
                        color,
                    );
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
        let src = K_PALETTE_SPRITE_AUX1 + 14 * 7 * 2;
        self.palette_load_multiple_arbitrary_snes(src, 0x162, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x182, 6);
        self.palette_load_multiple_arbitrary_snes(src, 0x1a2, 6);
        self.palette_load_multiple_arbitrary_snes(K_PALETTE_SPRITE_AUX1 + 21 * 7 * 2, 0x1c2, 6);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn HandleScreenFlash(&mut self) {
        let j = self.ram[INTRO_TIMES_PAL_FLASH];
        if j == 0 || self.frame_control_view().submodule() != 0 {
            return;
        }
        self.ram[INTRO_TIMES_PAL_FLASH] = self.ram[INTRO_TIMES_PAL_FLASH].wrapping_sub(1);
        if self.ram[INTRO_TIMES_PAL_FLASH] == 0 {
            self.palette_restore_bg_and_hud();
            return;
        }
        if j & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn asset_bytes(&self, asset: usize, index: usize) -> Option<&[u8]> {
        Some(self.asset_memblk(asset, index)?.ptr)
    }
}

impl ZeldaState {
    pub(super) fn handle_screen_flash(&mut self) {
        let flash = self.ram[INTRO_TIMES_PAL_FLASH];
        if flash == 0 || self.frame_control_view().submodule() != 0 {
            return;
        }
        self.ram[INTRO_TIMES_PAL_FLASH] = flash.wrapping_sub(1);
        if self.ram[INTRO_TIMES_PAL_FLASH] == 0 {
            self.palette_restore_bg_and_hud();
            return;
        }
        if flash & 1 != 0 {
            self.filter_majorly_whiten_bg();
        } else {
            self.palette_restore_bg_from_flash();
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn apply_palette_filter_bounce(&mut self) {
        self.palette_filter_range(0, 1);
        self.palette_filter_range(0x20, 0xd8);
        self.palette_filter_range(0xe0, 0xf0);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);

        let countdown = read_le_u16(&self.ram, PALETTE_FILTER_COUNTDOWN);
        let target = self.ram[MOSAIC_TARGET_LEVEL] as u16;
        if read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN) == 0 {
            let next = countdown.wrapping_add(1);
            write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, next);
            if next != target {
                return;
            }
        } else {
            write_le_u16(
                &mut self.ram,
                PALETTE_FILTER_COUNTDOWN,
                countdown.wrapping_sub(1),
            );
            if countdown != target {
                return;
            }
        }
        let mode = read_le_u16(&self.ram, DARKENING_OR_LIGHTENING_SCREEN) ^ 2;
        write_le_u16(&mut self.ram, DARKENING_OR_LIGHTENING_SCREEN, mode);
        write_le_u16(&mut self.ram, PALETTE_FILTER_COUNTDOWN, 0);
        self.frame_control_view_mut().increment_subsubmodule();
    }

    pub(super) fn spotlight_open(&mut self) {
        self.spotlight_internal(0, 2);
    }

    pub(super) fn spotlight_internal(&mut self, x: u8, y: u8) {
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_RADIUS, x as u16);
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_STATE, y as u16);
        self.hdma_setup(0x00f2fb, 0x00f2fb, 0x41, 0x26, 0x26, 0);
        self.ram[W12SEL_COPY] = 0x33;
        self.ram[W34SEL_COPY] = 3;
        self.ram[WOBJSEL_COPY] = 0x33;
        self.ram[TMW_COPY] = self.ram[TM_COPY];
        self.ram[TSW_COPY] = self.ram[TS_COPY];
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.ram[COLDATA_COPY0] = 0x20;
            self.ram[COLDATA_COPY1] = 0x40;
            self.ram[COLDATA_COPY2] = 0x80;
        }
        self.iris_spotlight_configure_table();
        self.ram[HDMAEN_COPY] = 0x80;
        self.ram[INIDISP_COPY] = 0x0f;
    }

    pub(super) fn iris_spotlight_configure_table(&mut self) {
        const SPOTLIGHT_DELTA_SIZE: [i8; 4] = [-7, 7, 7, 7];
        const SPOTLIGHT_GOAL: [u16; 4] = [0, 126, 35, 126];

        let r14 = self
            .player_state_view()
            .y()
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
            .wrapping_add(12);
        let radius = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_RADIUS);
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_LOWER, r14.wrapping_sub(radius));
        write_le_u16(&mut self.ram, SPOTLIGHT_Y_UPPER, r14.wrapping_add(radius));
        let x_center = self
            .player_state_view()
            .x()
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2))
            .wrapping_add(8);
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_X_CENTER, x_center);
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, radius);

        let mut r6 = r14.wrapping_mul(2);
        if r6 < 224 {
            r6 = 224;
        }
        let mut r4 = r14.wrapping_mul(2).wrapping_sub(r6);
        loop {
            let mut r8 = 0x00ff;
            if r6 < read_le_u16(&self.ram, SPOTLIGHT_Y_UPPER) {
                let t = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_Y_BUFFER) as u8;
                if read_le_u16(&self.ram, SPOTLIGHT_WINDOW_Y_BUFFER) != 0 {
                    let next = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_Y_BUFFER).wrapping_sub(1);
                    write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_Y_BUFFER, next);
                }
                r8 = self.iris_spotlight_calculate_circle_value(t);
            }
            if r4 < 240 {
                write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + r4 as usize * 2, r8);
            }
            if r6 < 240 {
                write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + r6 as usize * 2, r8);
            }
            if r4 == r14 {
                break;
            }
            r4 = r4.wrapping_add(1);
            r6 = r6.wrapping_sub(1);
        }

        for i in 224..240 {
            write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0);
        }
        let src = HDMA_TABLE_DYNAMIC;
        let dst = HDMA_TABLE_UNUSED;
        let bytes = 224 * 2;
        let tmp = self.ram[src..src + bytes].to_vec();
        self.ram[dst..dst + bytes].copy_from_slice(&tmp);

        let idx = (read_le_u16(&self.ram, SPOTLIGHT_WINDOW_STATE) >> 1) as usize;
        let delta = SPOTLIGHT_DELTA_SIZE[idx] as i16 as u16;
        let next = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_RADIUS).wrapping_add(delta);
        write_le_u16(&mut self.ram, SPOTLIGHT_WINDOW_RADIUS, next);
        if next == SPOTLIGHT_GOAL[idx] {
            if read_le_u16(&self.ram, SPOTLIGHT_WINDOW_STATE) == 0 {
                self.ram[INIDISP_COPY] = 0x80;
            } else {
                self.iris_spotlight_reset_table();
            }
            self.frame_control_view_mut().set_subsubmodule(0);
            self.frame_control_view_mut().set_submodule(0);
            let main_module = self.frame_control_view().main_module();
            if main_module == 7 || main_module == 16 {
                if self.ram[PLAYER_IS_INDOORS] == 0 {
                    self.ram[SOUND_EFFECT_AMBIENT] =
                        self.ram[OVERWORLD_MUSIC + self.ram[OVERWORLD_SCREEN_INDEX] as usize] >> 4;
                }
                if self.ram[QUEUED_MUSIC_CONTROL] != 0xff {
                    self.ram[MUSIC_CONTROL] = self.ram[QUEUED_MUSIC_CONTROL];
                }
            }
            let saved_module = self.ram[SAVED_MODULE_FOR_MENU];
            self.frame_control_view_mut().set_main_module(saved_module);
            if self.frame_control_view().main_module() == 6 {
                self.sprite_reset_all();
            }
        }
    }

    pub(super) fn iris_spotlight_calculate_circle_value(&self, a: u8) -> u16 {
        const CONFIGURE_SPOTLIGHT_TABLE_HELPER: [u8; 129] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe,
            0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0xfc, 0xfc, 0xfc, 0xfb, 0xfb, 0xfb, 0xfa, 0xfa,
            0xf9, 0xf9, 0xf8, 0xf8, 0xf7, 0xf7, 0xf6, 0xf6, 0xf5, 0xf5, 0xf4, 0xf3, 0xf3, 0xf2,
            0xf1, 0xf1, 0xf0, 0xef, 0xee, 0xee, 0xed, 0xec, 0xeb, 0xea, 0xe9, 0xe9, 0xe8, 0xe7,
            0xe6, 0xe5, 0xe4, 0xe3, 0xe2, 0xe1, 0xdf, 0xde, 0xdd, 0xdc, 0xdb, 0xda, 0xd8, 0xd7,
            0xd6, 0xd5, 0xd3, 0xd2, 0xd0, 0xcf, 0xcd, 0xcc, 0xca, 0xc9, 0xc7, 0xc6, 0xc4, 0xc2,
            0xc1, 0xbf, 0xbd, 0xbb, 0xb9, 0xb7, 0xb6, 0xb4, 0xb1, 0xaf, 0xad, 0xab, 0xa9, 0xa7,
            0xa4, 0xa2, 0x9f, 0x9d, 0x9a, 0x97, 0x95, 0x92, 0x8f, 0x8c, 0x89, 0x86, 0x82, 0x7f,
            0x7b, 0x78, 0x74, 0x70, 0x6c, 0x67, 0x63, 0x5e, 0x59, 0x53, 0x4d, 0x46, 0x3f, 0x37,
            0x2d, 0x1f, 0,
        ];

        let radius = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_RADIUS) as u8;
        let div = if radius == 0 {
            0xffff
        } else {
            ((a as u16) << 8) / radius as u16
        };
        let t = (div >> 1) as usize;
        let r10 = CONFIGURE_SPOTLIGHT_TABLE_HELPER[t];
        let p = 2 * (((r10 as u16) * (radius as u16) >> 8) as u8 as u16);
        if r10 == 0 {
            return 0x00ff;
        }
        let x_center = read_le_u16(&self.ram, SPOTLIGHT_WINDOW_X_CENTER);
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
            write_le_u16(&mut self.ram, HDMA_TABLE_DYNAMIC + i * 2, 0xff00);
        }
    }

    pub(super) fn overworld_copy_palettes_to_cache(&mut self) {
        let aux = self.ram[AUX_PALETTE_BUFFER..AUX_PALETTE_BUFFER + 512].to_vec();
        self.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 512].copy_from_slice(&aux);
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn enable_force_blank(&mut self) {
        self.ram[INIDISP_COPY] = 0x80;
        self.ram[HDMAEN_COPY] = 0;
    }

    // ----- Palette backdrop / OW background color helpers ---------------------
    // Port of zelda3/src/load_gfx.c:1820-1856 plus LoadGearPalette at 1913.

    pub(super) fn palette_bg_and_fixed_color_black(&mut self) {
        self.palette_set_bg_and_fixed_color(0);
    }

    pub(super) fn palette_set_bg_and_fixed_color(&mut self, color: u16) {
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER, color);
        write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + 32 * 2, color);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, color);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 32 * 2, color);
        self.set_backdropcolor_black();
    }

    pub(super) fn set_backdropcolor_black(&mut self) {
        self.ram[COLDATA_COPY0] = 0x20;
        self.ram[COLDATA_COPY1] = 0x40;
        self.ram[COLDATA_COPY2] = 0x80;
    }

    pub(super) fn palette_set_ow_bg_color(&mut self) {
        let color = self.palette_get_ow_bg_color();
        self.palette_set_bg_and_fixed_color(color);
    }

    pub(super) fn palette_special_ow(&mut self) {
        let c = self.palette_get_ow_bg_color();
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER, c);
        write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + 32 * 2, c);
        self.set_backdropcolor_black();
    }

    pub(super) fn palette_get_ow_bg_color(&self) -> u16 {
        let ow_screen = u16::from(self.world_state_view().overworld_screen());
        if ow_screen < 0x80 {
            return if ow_screen & 0x40 != 0 {
                0x2A32
            } else {
                0x2669
            };
        }
        let room = self.world_state_view().dungeon_room();
        if room == 0x180 || room == 0x182 || room == 0x183 {
            return 0x19C6;
        }
        0x2669
    }

    pub(super) fn palette_assert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(true);
    }

    pub(super) fn palette_set_translucency_swap(&mut self, v: bool) {
        self.ram[PALETTE_SWAP_FLAG] = if v { 1 } else { 0 };
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
                let a = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + low * 2);
                let b = read_le_u16(&self.ram, AUX_PALETTE_BUFFER + high * 2);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + high * 2, a);
                write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + high * 2, a);
                write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + low * 2, b);
                write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + low * 2, b);
            }
        }
        self.ram[FLAG_UPDATE_CGRAM_IN_NMI] = self.ram[FLAG_UPDATE_CGRAM_IN_NMI].wrapping_add(1);
    }

    pub(super) fn palette_revert_translucency_swap(&mut self) {
        self.palette_set_translucency_swap(false);
    }

    /// memcpy `n` u16 entries from `src` into both palette buffers at byte
    /// offset `dst`. Matches `LoadGearPalette(int dst, const uint16 *src, int n)`
    /// at load_gfx.c:1913; not called in the C codebase but ported for parity.
    pub(super) fn load_gear_palette(&mut self, dst: i32, src: &[u16], n: i32) {
        let base = (dst as usize) & !1;
        for i in 0..n as usize {
            let v = src[i];
            write_le_u16(&mut self.ram, AUX_PALETTE_BUFFER + base + i * 2, v);
            write_le_u16(&mut self.ram, MAIN_PALETTE_BUFFER + base + i * 2, v);
        }
    }
}
