//! Ported SomariaPlatform / MasterSword / Flute handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c lines
//! 1010, 2077..2368, 9828..9979, 25449..25726). The original C body is
//! reproduced as a comment block immediately above each port so a reviewer
//! can verify behavior line-by-line.

use super::sprite::DrawMultipleData;
use super::*;
use crate::types::{sign16, sign8};

const SRAM_PROGRESS_INDICATOR_3: usize = 0x0f3c9;

// `kPlayerState_SpinAttacking = 3` and `kPlayerState_Hookshot = 19` from
// `player.h`.
const K_PLAYER_STATE_SPIN_ATTACKING: u8 = 3;
const K_PLAYER_STATE_HOOKSHOT: u8 = 19;

// `sprite_main.c:25503` - SomariaPlatform drag-link motion table.
const K_SOMARIA_PLATFORM_DRAG_X: [i8; 8] = [0, 0, -1, 1, -1, 1, 1, -1];
const K_SOMARIA_PLATFORM_DRAG_Y: [i8; 8] = [-1, 1, 0, 0, -1, 1, -1, 1];
const K_PIPE_DIRS: [u8; 4] = [8, 4, 2, 1];
const ALT_SPRITE_SPAWNED_FLAG_WORLD: usize = 0x1de0;

// SomariaPlatform draw atlas (sprite_main.c:25539-25555).
const K_SOMARIA_PLATFORM_DMD: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -16,
        y: -16,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -16,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 0,
        char_flags: 0x80ac,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0xc0ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -13,
        y: -13,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -13,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -13,
        y: -3,
        char_flags: 0x80ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -3,
        y: -3,
        char_flags: 0xc0ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -10,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -10,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -10,
        y: -6,
        char_flags: 0x80ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0xc0ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x00ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x40ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x80ac,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc0ac,
        ext: 2,
    },
];

// MasterSword draw constants (sprite_main.c:2354-2356).
const K_MASTER_SWORD_DRAW_X: [i8; 6] = [-8, 0, -8, 0, -8, 0];
const K_MASTER_SWORD_DRAW_Y: [i8; 6] = [-8, -8, 0, 0, 8, 8];
const K_MASTER_SWORD_DRAW_CHAR: [u8; 6] = [0xc3, 0xc4, 0xd3, 0xd4, 0xe0, 0xf0];

// MasterSword pendant motion tables (sprite_main.c:2302-2303).
const K_MASTER_SWORD_PENDANT_XV: [i8; 4] = [-4, 4, 0, 0];
const K_MASTER_SWORD_PENDANT_YV: [i8; 4] = [-2, -2, -4, -4];

// MasterSword light-beam tables (sprite_main.c:2233-2244).
const K_MASTER_SWORD_LIGHT_BEAM_XV0: [i8; 2] = [0, -48];
const K_MASTER_SWORD_LIGHT_BEAM_XV1: [i8; 2] = [0, 48];
const K_MASTER_SWORD_LIGHT_BEAM_XV2: [i8; 2] = [-96, -48];
const K_MASTER_SWORD_LIGHT_BEAM_XV3: [i8; 2] = [96, 48];
const K_MASTER_SWORD_LIGHT_BEAM_YV0: [i8; 2] = [-96, -48];
const K_MASTER_SWORD_LIGHT_BEAM_YV1: [i8; 2] = [96, 48];
const K_MASTER_SWORD_LIGHT_BEAM_YV2: [i8; 2] = [0, 48];
const K_MASTER_SWORD_LIGHT_BEAM_YV3: [i8; 2] = [0, -48];
const K_MASTER_SWORD_LIGHT_BEAM_GFX0: [u8; 2] = [1, 0];
const K_MASTER_SWORD_LIGHT_BEAM_GFX2: [u8; 2] = [3, 2];
const K_MASTER_SWORD_LIGHT_BEAM_FLAGS0: [u8; 2] = [5, 0x45];
const K_MASTER_SWORD_LIGHT_BEAM_FLAGS2: [u8; 2] = [5, 5];

// MasterSword light-fountain animation tables (sprite_main.c:2143-2144).
const K_MASTER_SWORD_GFX1: [u8; 9] = [0, 1, 1, 2, 2, 2, 1, 1, 0];
const K_MASTER_SWORD_NUM_LIGHT_BEAMS: [u8; 9] = [0, 0, 1, 1, 2, 2, 0, 0, 0];

impl ZeldaState {
    // ============================================================
    // SomariaPlatform cluster (sprite_main.c:25449..25726)
    // ============================================================

    // void SomariaPlatform_LocatePath(int k) {  // 9ef640
    //   for (;;) {
    //     uint8 tiletype = SomariaPlatformAndPipe_CheckTile(k);
    //     sprite_E[k] = tiletype;
    //     if (tiletype >= 0xb0 && tiletype < 0xbf)
    //       break;
    //     Sprite_SetX(k, Sprite_GetX(k) + 8);
    //     Sprite_SetY(k, Sprite_GetY(k) + 8);
    //   }
    //   sprite_x_lo[k] = (sprite_x_lo[k] & ~7) + 4;
    //   sprite_y_lo[k] = (sprite_y_lo[k] & ~7) + 4;
    //   sprite_head_dir[k] = sprite_D[k];
    //   SomariaPlatformAndPipe_HandleMovement(k);
    //   sprite_ignore_projectile[k]++;
    //   player_on_somaria_platform = 0;
    //   sprite_delay_aux4[k] = 14;
    //   sprite_graphics[k]++;
    // }
    pub(super) fn somaria_platform_locate_path(&mut self, k: usize) {
        loop {
            let tiletype = self.somaria_platform_and_pipe_check_tile_for_world(k);
            self.ram[SPRITE_E + k] = tiletype;
            if (0xb0..0xbf).contains(&tiletype) {
                break;
            }
            let x = self.sprite_get_x(k).wrapping_add(8);
            let y = self.sprite_get_y(k).wrapping_add(8);
            self.sprite_set_x(k, x);
            self.sprite_set_y(k, y);
        }
        self.ram[SPRITE_X_LO + k] = (self.ram[SPRITE_X_LO + k] & !7).wrapping_add(4);
        self.ram[SPRITE_Y_LO + k] = (self.ram[SPRITE_Y_LO + k] & !7).wrapping_add(4);
        self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_D + k];
        self.somaria_platform_and_pipe_handle_movement(k);
        self.ram[SPRITE_IGNORE_PROJECTILE + k] =
            self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
        self.ram[SPRITE_DELAY_AUX4 + k] = 14;
        self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
    }

    // void Sprite_ED_SomariaPlatform(int k) {  // 9ef6d4
    pub(super) fn sprite_ed_somaria_platform(&mut self, k: usize) {
        match self.ram[SPRITE_GRAPHICS + k] {
            0 => {
                self.somaria_platform_locate_path(k);
                let j = self.sprite_spawn_superficial_bomb_blast(k);
                if j >= 0 {
                    let j = j as usize;
                    self.sprite_set_x(j, self.sprite_get_x(j).wrapping_sub(8));
                    self.sprite_set_y(j, self.sprite_get_y(j).wrapping_sub(8));
                }
            }
            1 => {
                self.somaria_platform_draw(k);
                if self.sprite_return_if_inactive(k) {
                    return;
                }
                let drag_x = read_le_u16(&self.ram, DRAG_PLAYER_X);
                let drag_y = read_le_u16(&self.ram, DRAG_PLAYER_Y);
                if (drag_x | drag_y) == 0
                    && sign8(self.ram[PLAYER_NEAR_PIT_STATE].wrapping_sub(2))
                    && self.sprite_check_damage_to_link_ignore_layer(k)
                {
                    self.ram[SPRITE_C + k] = 1;
                    self.link_cancel_dash();
                    if self.ram[LINK_PLAYER_HANDLER_STATE] != K_PLAYER_STATE_HOOKSHOT
                        && self.ram[LINK_PLAYER_HANDLER_STATE] != K_PLAYER_STATE_SPIN_ATTACKING
                    {
                        if self.ram[SPRITE_AI_STATE + k] != 0 {
                            self.somaria_platform_and_pipe_handle_movement(k);
                            return;
                        }
                        self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                        self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 2;
                        if (self.ram[SPRITE_A + k] & 7) == 0 {
                            let a = self.somaria_platform_and_pipe_check_tile(k);
                            if a != self.ram[SPRITE_E + k] {
                                self.ram[SPRITE_E + k] = a;
                                self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_D + k];
                                self.somaria_platform_and_pipe_handle_movement(k);
                                self.somaria_platform_handle_drag(k);
                            }
                        }
                        if self.ram[DUNGEON_ROOM_INDEX] != 36 {
                            let j = usize::from(self.ram[SPRITE_D + k]);
                            let dx = read_le_u16(&self.ram, DRAG_PLAYER_X)
                                .wrapping_add(K_SOMARIA_PLATFORM_DRAG_X[j] as i16 as u16);
                            let dy = read_le_u16(&self.ram, DRAG_PLAYER_Y)
                                .wrapping_add(K_SOMARIA_PLATFORM_DRAG_Y[j] as i16 as u16);
                            write_le_u16(&mut self.ram, DRAG_PLAYER_X, dx);
                            write_le_u16(&mut self.ram, DRAG_PLAYER_Y, dy);
                            self.sprite_move_xy(k);
                            self.somaria_platform_drag_link(k);
                        } else {
                            self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 1;
                        }
                        return;
                    }
                }
                if self.ram[SPRITE_C + k] != 0 {
                    self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
                    self.ram[SPRITE_C + k] = 0;
                }
            }
            _ => {}
        }
    }

    // void SomariaPlatform_Draw(int k) {  // 9ef860
    //   static const DrawMultipleData kSomariaPlatform_Dmd[16] = { ... };
    //   Oam_AllocateFromRegionB(0x10);
    //   Sprite_DrawMultiple(k, &kSomariaPlatform_Dmd[sprite_delay_aux4[k] & 12], 4, NULL);
    // }
    pub(super) fn somaria_platform_draw(&mut self, k: usize) {
        self.oam_allocate_from_region_b(0x10);
        let start = (self.ram[SPRITE_DELAY_AUX4 + k] & 12) as usize;
        let end = start + 4;
        self.sprite_draw_multiple(k, &K_SOMARIA_PLATFORM_DMD[start..end], None);
    }

    // void SomariaPlatform_HandleJunctions(int k) {  // 9ef87d
    //   See sprite_main.c:25561..25690 for the full switch on sprite_E[k]
    //   covering 0xb2..0xbe.
    pub(super) fn somaria_platform_handle_junctions(&mut self, k: usize) {
        match self.ram[SPRITE_E + k] {
            0xb2 | 0xb5 => {
                // ZigZagRisingSlope
                self.ram[SPRITE_D + k] ^= 3;
            }
            0xb3 | 0xb4 => {
                // ZigZagFallingSlope
                self.ram[SPRITE_D + k] ^= 2;
            }
            0xb6 => {
                // TransitTile
                const TRANSIT_DIR: [u8; 4] = [4, 8, 1, 2];
                self.ram[SPRITE_AI_STATE + k] = 1;
                let d = self.ram[SPRITE_D + k] as usize;
                if self.ram[LINK_AUXILIARY_STATE] == 0
                    && (self.ram[JOYPAD1H_LAST] & TRANSIT_DIR[d]) != 0
                {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_D + k] ^= 1;
                }
                self.ram[LINK_VISIBILITY_STATUS] = 0;
                self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 1;
            }
            0xb7 => {
                // Tjunc_NoUp
                const KEYS1: [u8; 4] = [3, 7, 6, 5];
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS1[d];
                if (t & 8) != 0 {
                    self.ram[SPRITE_D + k] = 0;
                } else if (t & 4) != 0 {
                    self.ram[SPRITE_D + k] = 1;
                } else if (t & 2) != 0 {
                    self.ram[SPRITE_D + k] = 2;
                } else if (t & 1) != 0 {
                    self.ram[SPRITE_D + k] = 3;
                } else if self.ram[SPRITE_D + k] == 0 {
                    self.ram[SPRITE_D + k] = 2;
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            0xb8 => {
                // Tjunc_NoDown
                const KEYS2: [u8; 4] = [11, 3, 10, 9];
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS2[d];
                if (t & 8) != 0 {
                    self.ram[SPRITE_D + k] = 0;
                } else if (t & 4) != 0 {
                    self.ram[SPRITE_D + k] = 1;
                } else if (t & 2) != 0 {
                    self.ram[SPRITE_D + k] = 2;
                } else if (t & 1) != 0 {
                    self.ram[SPRITE_D + k] = 3;
                } else if self.ram[SPRITE_D + k] == 1 {
                    self.ram[SPRITE_D + k] = 2;
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            0xb9 => {
                // Tjunc_NoLeft
                const KEYS3: [u8; 4] = [9, 5, 12, 13];
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS3[d];
                if (t & 8) != 0 {
                    self.ram[SPRITE_D + k] = 0;
                } else if (t & 4) != 0 {
                    self.ram[SPRITE_D + k] = 1;
                } else if (t & 2) != 0 {
                    self.ram[SPRITE_D + k] = 2;
                } else if (t & 1) != 0 {
                    self.ram[SPRITE_D + k] = 3;
                } else if self.ram[SPRITE_D + k] == 2 {
                    self.ram[SPRITE_D + k] = 0;
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            0xba => {
                // Tjunc_NoRight
                const KEYS4: [u8; 4] = [0xa, 6, 0xe, 0xc];
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS4[d];
                if (t & 8) != 0 {
                    self.ram[SPRITE_D + k] = 0;
                } else if (t & 4) != 0 {
                    self.ram[SPRITE_D + k] = 1;
                } else if (t & 2) != 0 {
                    self.ram[SPRITE_D + k] = 2;
                } else if (t & 1) != 0 {
                    self.ram[SPRITE_D + k] = 3;
                } else if self.ram[SPRITE_D + k] == 3 {
                    self.ram[SPRITE_D + k] = 0;
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            0xbb => {
                // TransitTileNoBack
                const KEYS5: [u8; 4] = [0xb, 7, 0xe, 0xd];
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS5[d];
                if (t & 8) != 0 {
                    self.ram[SPRITE_D + k] = 0;
                } else if (t & 4) != 0 {
                    self.ram[SPRITE_D + k] = 1;
                } else if (t & 2) != 0 {
                    self.ram[SPRITE_D + k] = 2;
                } else if (t & 1) != 0 {
                    self.ram[SPRITE_D + k] = 3;
                }
            }
            0xbc => {
                // TransitTileQuestion
                const KEYS6: [u8; 4] = [0xc, 0xc, 3, 3];
                self.ram[SPRITE_AI_STATE + k] = 1;
                let d = self.ram[SPRITE_D + k] as usize;
                let t = self.ram[JOYPAD1H_LAST] & KEYS6[d];
                if t != 0 {
                    if (t & 8) != 0 {
                        self.ram[SPRITE_D + k] = 0;
                    } else if (t & 4) != 0 {
                        self.ram[SPRITE_D + k] = 1;
                    } else if (t & 2) != 0 {
                        self.ram[SPRITE_D + k] = 2;
                    } else {
                        self.ram[SPRITE_D + k] = 3;
                    }
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
                self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 1;
            }
            0xbe => {
                // endpoint
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_D + k] ^= 1;
                self.ram[LINK_VISIBILITY_STATUS] = 0;
                self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 1;
            }
            _ => {}
        }
    }

    // void SomariaPlatform_HandleDragX(int k) {  // 9ef8ad
    //   if ((sprite_D[k] ^ sprite_head_dir[k]) & 2) {
    //     uint8 x = (sprite_x_lo[k] & ~7) + 4;
    //     uint8 t = x - sprite_x_lo[k];
    //     if (!t) return;
    //     drag_player_x = (int8)t;
    //     sprite_x_lo[k] = x;
    //   }
    // }
    pub(super) fn somaria_platform_handle_drag_x(&mut self, k: usize) {
        if ((self.ram[SPRITE_D + k] ^ self.ram[SPRITE_HEAD_DIR + k]) & 2) != 0 {
            let x = (self.ram[SPRITE_X_LO + k] & !7).wrapping_add(4);
            let t = x.wrapping_sub(self.ram[SPRITE_X_LO + k]);
            if t == 0 {
                return;
            }
            // drag_player_x is a 16-bit value at 0x0b7c — write t sign-extended.
            let v = t as i8 as i16 as u16;
            write_le_u16(&mut self.ram, DRAG_PLAYER_X, v);
            self.ram[SPRITE_X_LO + k] = x;
        }
    }

    // void SomariaPlatform_HandleDragY(int k) {  // 9ef8d7
    //   if ((sprite_D[k] ^ sprite_head_dir[k]) & 2) {
    //     uint8 y = (sprite_y_lo[k] & ~7) + 4;
    //     uint8 t = y - sprite_y_lo[k];
    //     if (!t) return;
    //     drag_player_y = (int8)t;
    //     sprite_y_lo[k] = y;
    //   }
    // }
    pub(super) fn somaria_platform_handle_drag_y(&mut self, k: usize) {
        if ((self.ram[SPRITE_D + k] ^ self.ram[SPRITE_HEAD_DIR + k]) & 2) != 0 {
            let y = (self.ram[SPRITE_Y_LO + k] & !7).wrapping_add(4);
            let t = y.wrapping_sub(self.ram[SPRITE_Y_LO + k]);
            if t == 0 {
                return;
            }
            let v = t as i8 as i16 as u16;
            write_le_u16(&mut self.ram, DRAG_PLAYER_Y, v);
            self.ram[SPRITE_Y_LO + k] = y;
        }
    }

    // void SomariaPlatform_HandleDrag(int k) {  // 9ef901
    //   SomariaPlatform_HandleDragX(k);
    //   SomariaPlatform_HandleDragY(k);
    // }
    pub(super) fn somaria_platform_handle_drag(&mut self, k: usize) {
        self.somaria_platform_handle_drag_x(k);
        self.somaria_platform_handle_drag_y(k);
    }

    // void SomariaPlatform_DragLink(int k) {  // 9efb49
    //   uint16 x = cur_sprite_x - 8 - link_x_coord;
    //   if (x) drag_player_x += sign16(x) ? -1 : 1;
    //   uint16 y = cur_sprite_y - 16 - link_y_coord;
    //   if (y) drag_player_y += sign16(y) ? -1 : 1;
    // }
    pub(super) fn somaria_platform_drag_link(&mut self, _k: usize) {
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let lx = self.player_state_view().x();
        let ly = self.player_state_view().y();
        let x = cur_x.wrapping_sub(8).wrapping_sub(lx);
        if x != 0 {
            let delta: u16 = if sign16(x) { 0xffff } else { 1 };
            let cur = read_le_u16(&self.ram, DRAG_PLAYER_X);
            write_le_u16(&mut self.ram, DRAG_PLAYER_X, cur.wrapping_add(delta));
        }
        let y = cur_y.wrapping_sub(16).wrapping_sub(ly);
        if y != 0 {
            let delta: u16 = if sign16(y) { 0xffff } else { 1 };
            let cur = read_le_u16(&self.ram, DRAG_PLAYER_Y);
            write_le_u16(&mut self.ram, DRAG_PLAYER_Y, cur.wrapping_add(delta));
        }
    }

    // void Sprite_AE_Pipe_Down(int k) {  // 9efb7e
    pub(super) fn sprite_ae_pipe_down(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.ram[SPRITE_GRAPHICS + k] {
            0 => {
                self.ram[ALT_SPRITE_SPAWNED_FLAG_WORLD] = 255;
                self.ram[SPRITE_D + k] = self.ram[SPRITE_TYPE + k].wrapping_sub(0xae);
                self.somaria_platform_locate_path(k);
            }
            1 => {
                let mut t = self.somaria_platform_and_pipe_check_tile(k);
                if t == 0xbe {
                    self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
                    self.ram[SPRITE_D + k] ^= 1;
                    t = self.ram[SPRITE_D + k];
                }
                self.ram[SPRITE_E + k] = t;
                self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_D + k];
                self.somaria_platform_and_pipe_handle_movement(k);
                self.sprite_move_xy(k);
            }
            2 => {
                if self.ram[ALT_SPRITE_SPAWNED_FLAG_WORLD] == 255
                    && self.sprite_check_damage_to_link_ignore_layer(k)
                {
                    if !self.pipe_validate_entry() {
                        self.ram[SPRITE_GRAPHICS + k] =
                            self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
                        self.ram[SPRITE_DELAY_AUX1 + k] = 4;
                        self.link_reset_properties_a();
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                        self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                        self.ram[ALT_SPRITE_SPAWNED_FLAG_WORLD] = k as u8;
                    } else {
                        self.sprite_halt_all_movement();
                    }
                }
            }
            3 => {
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
                    self.ram[LINK_VISIBILITY_STATUS] = 12;
                } else {
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                    self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                    self.pipe_handle_player_movement(
                        K_PIPE_DIRS[usize::from(self.ram[SPRITE_D + k])],
                    );
                }
            }
            4 => {
                self.ram[SPRITE_SUBTYPE2 + k] = 3;
                self.ram[LINK_X_COORD_SAFE_RETURN_LO] = self.ram[LINK_X_COORD];
                self.ram[LINK_X_COORD_SAFE_RETURN_HI] = self.ram[LINK_X_COORD + 1];
                self.ram[LINK_Y_COORD_SAFE_RETURN_LO] = self.ram[LINK_Y_COORD];
                self.ram[LINK_Y_COORD_SAFE_RETURN_HI] = self.ram[LINK_Y_COORD + 1];
                loop {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                    if (self.ram[SPRITE_A + k] & 7) == 0 {
                        let t = self.somaria_platform_and_pipe_check_tile(k);
                        if (0xb2..0xb6).contains(&t) {
                            self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
                        }
                        if t != self.ram[SPRITE_E + k] {
                            self.ram[SPRITE_E + k] = t;
                            if t == 0xbe {
                                self.ram[SPRITE_GRAPHICS + k] =
                                    self.ram[SPRITE_GRAPHICS + k].wrapping_add(1);
                                self.ram[SPRITE_DELAY_AUX1 + k] = 24;
                            }
                            self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_D + k];
                            self.somaria_platform_and_pipe_handle_movement(k);
                            self.somaria_platform_handle_drag(k);
                        }
                    }
                    self.sprite_move_xy(k);
                    let x = self.sprite_get_x(k).wrapping_sub(8);
                    let y = self.sprite_get_y(k).wrapping_sub(14);
                    let link_x = self.player_state_view().x();
                    let link_y = self.player_state_view().y();
                    if x != link_x {
                        let next = link_x.wrapping_add(if x < link_x { 0xffff } else { 1 });
                        self.player_state_view_mut().set_x(next);
                    }
                    if y != link_y {
                        let next = link_y.wrapping_add(if y < link_y { 0xffff } else { 1 });
                        self.player_state_view_mut().set_y(next);
                    }
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
                    if self.ram[SPRITE_SUBTYPE2 + k] == 0 {
                        break;
                    }
                }
                self.ram[LINK_X_VEL] =
                    self.ram[LINK_X_COORD].wrapping_sub(self.ram[LINK_X_COORD_SAFE_RETURN_LO]);
                self.ram[LINK_Y_VEL] =
                    self.ram[LINK_Y_COORD].wrapping_sub(self.ram[LINK_Y_COORD_SAFE_RETURN_LO]);
                self.ram[LINK_DIRECTION_LAST] = K_PIPE_DIRS[usize::from(self.ram[SPRITE_D + k])];
                self.link_handle_moving_animation_full_long_entry();
                self.handle_indoor_camera_and_doors();
                self.link_cancel_dash();
            }
            5 => {
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                    self.ram[PLAYER_ON_SOMARIA_PLATFORM] = 0;
                    self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                    self.ram[LINK_VISIBILITY_STATUS] = 0;
                    self.ram[LINK_Y_VEL] = 0;
                    self.ram[LINK_X_VEL] = 0;
                    self.ram[ALT_SPRITE_SPAWNED_FLAG_WORLD] = 255;
                    self.ram[SPRITE_GRAPHICS + k] = 2;
                } else {
                    self.pipe_handle_player_movement(
                        K_PIPE_DIRS[usize::from(self.ram[SPRITE_D + k] ^ 1)],
                    );
                }
            }
            _ => {}
        }
    }

    // ============================================================
    // MasterSword cluster (sprite_main.c:2077..2368)
    // ============================================================

    // void MasterSword_Main(int k) {  // 8588d6
    //   See sprite_main.c:2077..2140 for the full switch on sprite_ai_state[k]
    //   driving the pendant -> sword scene.
    pub(super) fn master_sword_main(&mut self, k: usize) {
        let ow = self.ram[OVERWORLD_SCREEN_INDEX] as usize;
        if self.frame_control_view().main_module() != 26
            && (self.ram[SAVE_OW_EVENT_INFO + ow] & 0x40) != 0
        {
            self.ram[SPRITE_STATE + k] = 0;
            return;
        }
        if self.ram[SPRITE_AI_STATE + k] != 5 {
            self.master_sword_draw(k);
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                // waiting
                if self.sprite_check_if_link_is_busy()
                    || !self.sprite_check_damage_to_link_same_layer_for_world(k)
                    || self.ram[LINK_DIRECTION_FACING] != 2
                    || (self.ram[FILTERED_JOYPAD_L] & 0x80) == 0
                    || (self.ram[LINK_WHICH_PENDANTS] & 7) != 7
                {
                    return;
                }
                self.ram[MUSIC_CONTROL] = 10;
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
                self.master_sword_spawn_pendant_prop(k, 9);
                self.master_sword_spawn_pendant_prop(k, 11);
                self.master_sword_spawn_pendant_prop(k, 15);
                self.master_sword_spawn_light_well(k);
                self.ram[SPRITE_AI_STATE + k] = 1;
                self.ram[SPRITE_DELAY_MAIN + k] = 240;
            }
            1 => {
                // pendants transfer
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.master_sword_spawn_light_fountain(k);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 192;
                }
                self.ram[LINK_PULL_ACTION_STATE] = 10;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            }
            2 => {
                // light show
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.master_sword_spawn_light_beam(k, 0, 0xff);
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_DELAY_MAIN + k] = 8;
                }
                self.ram[LINK_PULL_ACTION_STATE] = 10;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            }
            3 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.master_sword_spawn_light_beam(k, 1, 0xff);
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                }
                self.ram[LINK_PULL_ACTION_STATE] = 11;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            }
            4 => {
                // give to player
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SAVE_OW_EVENT_INFO + ow] |= 0x40;
                    self.ram[ITEM_RECEIPT_METHOD] = 0;
                    self.link_receive_item(1, 0);
                    self.ram[SAVEGAME_MAP_ICONS_INDICATOR] = 5;
                    self.ram[LINK_PULL_ACTION_STATE] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 5;
                }
            }
            5 => {
                // stop
                self.ram[SPRITE_STATE + k] = 0;
            }
            _ => {}
        }
    }

    // void Sprite_MasterSword_LightFountain(int k) {  // 8589dc
    //   static const uint8 kMasterSword_Gfx1[9] = {0, 1, 1, 2, 2, 2, 1, 1, 0};
    //   static const uint8 kMasterSword_NumLightBeams[9] = {0, 0, 1, 1, 2, 2, 0, 0, 0};
    //   SpriteDraw_LightFountain(k);
    //   sprite_A[k]++;
    //   if (!sprite_A[k]) {
    //     sprite_C[k]++;
    //     sprite_state[k] = 0;
    //   }
    //   sprite_D[k] = sprite_A[k] >> 2 & 3;
    //   int j = sprite_A[k] >> 5 & 7;
    //   sprite_graphics[k] = kMasterSword_Gfx1[j];
    //   if (kMasterSword_NumLightBeams[j])
    //     MasterSword_SpawnLightBeam(k, sprite_A[k] >> 2 & 1, kMasterSword_NumLightBeams[j]);
    // }
    pub(super) fn sprite_master_sword_light_fountain(&mut self, k: usize) {
        self.sprite_draw_light_fountain_for_world(k);
        let new_a = self.ram[SPRITE_A + k].wrapping_add(1);
        self.ram[SPRITE_A + k] = new_a;
        if new_a == 0 {
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
            self.ram[SPRITE_STATE + k] = 0;
        }
        self.ram[SPRITE_D + k] = (self.ram[SPRITE_A + k] >> 2) & 3;
        let j = ((self.ram[SPRITE_A + k] >> 5) & 7) as usize;
        self.ram[SPRITE_GRAPHICS + k] = K_MASTER_SWORD_GFX1[j];
        if K_MASTER_SWORD_NUM_LIGHT_BEAMS[j] != 0 {
            let ain = (self.ram[SPRITE_A + k] >> 2) & 1;
            self.master_sword_spawn_light_beam(k, ain, K_MASTER_SWORD_NUM_LIGHT_BEAMS[j]);
        }
    }

    // void Sprite_MasterSword_LightWell(int k) {  // 858a16
    //   SpriteDraw_LightFountain(k);
    //   sprite_A[k]++;
    //   if (!sprite_A[k]) {
    //     sprite_C[k]++;
    //     sprite_state[k] = 0;
    //   }
    //   sprite_D[k] = sprite_A[k] >> 2 & 3;
    //   sprite_graphics[k] = 0;
    // }
    pub(super) fn sprite_master_sword_light_well(&mut self, k: usize) {
        self.sprite_draw_light_fountain_for_world(k);
        let new_a = self.ram[SPRITE_A + k].wrapping_add(1);
        self.ram[SPRITE_A + k] = new_a;
        if new_a == 0 {
            self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
            self.ram[SPRITE_STATE + k] = 0;
        }
        self.ram[SPRITE_D + k] = (self.ram[SPRITE_A + k] >> 2) & 3;
        self.ram[SPRITE_GRAPHICS + k] = 0;
    }

    // void MasterSword_SpawnLightWell(int k) {  // 858ab6
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x62, &info);
    //   Sprite_SetSpawnedCoordinates(j, &info);
    //   sprite_subtype2[j] = 4;
    //   sprite_oam_flags[j] = 5;
    //   sprite_flags2[j] = 0;
    // }
    pub(super) fn master_sword_spawn_light_well(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) {
            self.sprite_set_spawned_coordinates_for_world(j, r0_x, r2_y);
            self.ram[SPRITE_SUBTYPE2 + j] = 4;
            self.ram[SPRITE_OAM_FLAGS + j] = 5;
            self.ram[SPRITE_FLAGS2 + j] = 0;
        }
    }

    // void MasterSword_SpawnLightFountain(int k) {  // 858ad0
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x62, &info);
    //   Sprite_SetSpawnedCoordinates(j, &info);
    //   sprite_subtype2[j] = 1;
    //   sprite_oam_flags[j] = 5;
    //   sprite_flags2[j] = 0;
    // }
    pub(super) fn master_sword_spawn_light_fountain(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) {
            self.sprite_set_spawned_coordinates_for_world(j, r0_x, r2_y);
            self.ram[SPRITE_SUBTYPE2 + j] = 1;
            self.ram[SPRITE_OAM_FLAGS + j] = 5;
            self.ram[SPRITE_FLAGS2 + j] = 0;
        }
    }

    // void Sprite_MasterSword_LightBeam(int k) {  // 858aea
    //   SpriteDraw_SingleLarge(k);
    //   if (sprite_A[k]) {
    //     Sprite_MoveXY(k);
    //     if (frame_counter & 3) return;
    //     MasterSword_SpawnReplacementLightBeam(k);
    //   }
    //   if (!--sprite_B[k]) sprite_state[k] = 0;
    // }
    pub(super) fn sprite_master_sword_light_beam(&mut self, k: usize) {
        self.sprite_draw_single_large_for_world(k);
        if self.ram[SPRITE_A + k] != 0 {
            self.sprite_move_xy(k);
            if (self.ram[FRAME_COUNTER] & 3) == 0 {
                self.master_sword_spawn_replacement_light_beam(k);
            } else {
                return;
            }
        }
        let new_b = self.ram[SPRITE_B + k].wrapping_sub(1);
        self.ram[SPRITE_B + k] = new_b;
        if new_b == 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void MasterSword_SpawnReplacementLightBeam(int k) {  // 858b20
    //   SpriteSpawnInfo info;
    //   int j;
    //   if ((j = Sprite_SpawnDynamically(k, 0x62, &info)) < 0) return;
    //   Sprite_SetX(j, info.r0_x);
    //   Sprite_SetY(j, info.r2_y);
    //   sprite_subtype2[j] = 2;
    //   sprite_B[j] = 3;
    //   sprite_graphics[j] = sprite_graphics[k];
    //   sprite_oam_flags[j] = sprite_oam_flags[k];
    //   sprite_flags2[j] = 0;
    // }
    pub(super) fn master_sword_spawn_replacement_light_beam(&mut self, k: usize) {
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x);
        self.sprite_set_y(j, r2_y);
        self.ram[SPRITE_SUBTYPE2 + j] = 2;
        self.ram[SPRITE_B + j] = 3;
        self.ram[SPRITE_GRAPHICS + j] = self.ram[SPRITE_GRAPHICS + k];
        self.ram[SPRITE_OAM_FLAGS + j] = self.ram[SPRITE_OAM_FLAGS + k];
        self.ram[SPRITE_FLAGS2 + j] = 0;
    }

    // void MasterSword_SpawnLightBeam(int k, uint8 ain, uint8 yin) {  // 858b62
    //   See sprite_main.c:2232..2299 — spawns four light-beam sprites with
    //   directional vel/gfx/flags tables.
    pub(super) fn master_sword_spawn_light_beam(&mut self, k: usize, ain: u8, yin: u8) {
        let ai = ain as usize;

        // Spawn 1
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.ram[SPRITE_SUBTYPE2 + j] = 2;
        self.ram[SPRITE_A + j] = 2;
        self.ram[SPRITE_FLAGS2 + j] = 0;
        self.ram[SPRITE_X_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_XV0[ai] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_YV0[ai] as u8;
        self.ram[SPRITE_GRAPHICS + j] = K_MASTER_SWORD_LIGHT_BEAM_GFX0[ai];
        self.ram[SPRITE_OAM_FLAGS + j] = K_MASTER_SWORD_LIGHT_BEAM_FLAGS0[ai];
        self.ram[SPRITE_B + j] = yin;

        // Spawn 2
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.ram[SPRITE_SUBTYPE2 + j] = 2;
        self.ram[SPRITE_A + j] = 2;
        self.ram[SPRITE_FLAGS2 + j] = 0;
        self.ram[SPRITE_X_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_XV1[ai] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_YV1[ai] as u8;
        self.ram[SPRITE_GRAPHICS + j] = K_MASTER_SWORD_LIGHT_BEAM_GFX0[ai];
        self.ram[SPRITE_OAM_FLAGS + j] = K_MASTER_SWORD_LIGHT_BEAM_FLAGS0[ai];
        self.ram[SPRITE_B + j] = yin;

        // Spawn 3
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.ram[SPRITE_SUBTYPE2 + j] = 2;
        self.ram[SPRITE_A + j] = 2;
        self.ram[SPRITE_FLAGS2 + j] = 0;
        self.ram[SPRITE_X_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_XV2[ai] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_YV2[ai] as u8;
        self.ram[SPRITE_GRAPHICS + j] = K_MASTER_SWORD_LIGHT_BEAM_GFX2[ai];
        self.ram[SPRITE_OAM_FLAGS + j] = K_MASTER_SWORD_LIGHT_BEAM_FLAGS2[ai];
        self.ram[SPRITE_B + j] = yin;

        // Spawn 4
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.ram[SPRITE_SUBTYPE2 + j] = 2;
        self.ram[SPRITE_A + j] = 2;
        self.ram[SPRITE_FLAGS2 + j] = 0;
        self.ram[SPRITE_X_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_XV3[ai] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_MASTER_SWORD_LIGHT_BEAM_YV3[ai] as u8;
        self.ram[SPRITE_GRAPHICS + j] = K_MASTER_SWORD_LIGHT_BEAM_GFX2[ai];
        self.ram[SPRITE_OAM_FLAGS + j] = K_MASTER_SWORD_LIGHT_BEAM_FLAGS2[ai];
        self.ram[SPRITE_B + j] = yin;
    }

    // void MasterSword_SpawnPendantProp(int k, uint8 ain) {  // 858cd3
    //   static const int8 kMasterSword_Pendant_Xv[4] = {-4, 4, 0, 0};
    //   static const int8 kMasterSword_Pendant_Yv[4] = {-2, -2, -4, -4};
    //   SpriteSpawnInfo info;
    //   int j;
    //   if ((j = Sprite_SpawnDynamically(k, 0x62, &info)) < 0) return;
    //   sprite_oam_flags[j] = ain;
    //   Sprite_SetX(j, link_x_coord);
    //   Sprite_SetY(j, link_y_coord + 8);
    //   sprite_graphics[j] = 4;
    //   sprite_subtype2[j] = 3;
    //   sprite_flags2[j] = 64;
    //   sprite_delay_main[j] = 228;
    //   int i = ain >> 1 & 3;
    //   sprite_x_vel[j] = kMasterSword_Pendant_Xv[i];
    //   sprite_y_vel[j] = kMasterSword_Pendant_Yv[i];
    // }
    pub(super) fn master_sword_spawn_pendant_prop(&mut self, k: usize, ain: u8) {
        let Some((j, _r0_x, _r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.ram[SPRITE_OAM_FLAGS + j] = ain;
        let lx = self.player_state_view().x();
        let ly = self.player_state_view().y();
        self.sprite_set_x(j, lx);
        self.sprite_set_y(j, ly.wrapping_add(8));
        self.ram[SPRITE_GRAPHICS + j] = 4;
        self.ram[SPRITE_SUBTYPE2 + j] = 3;
        self.ram[SPRITE_FLAGS2 + j] = 64;
        self.ram[SPRITE_DELAY_MAIN + j] = 228;
        let i = ((ain >> 1) & 3) as usize;
        self.ram[SPRITE_X_VEL + j] = K_MASTER_SWORD_PENDANT_XV[i] as u8;
        self.ram[SPRITE_Y_VEL + j] = K_MASTER_SWORD_PENDANT_YV[i] as u8;
    }

    // void Sprite_MasterSword_Prop(int k) {  // 858d29
    //   Oam_AllocateFromRegionB(4);
    //   SpriteDraw_SingleLarge(k);
    //   switch (sprite_ai_state[k]) {
    //   case 0:  // drifting away
    //     Sprite_MoveXY(k);
    //     if (!sprite_delay_main[k]) {
    //       sprite_ai_state[k] = 1;
    //       sprite_delay_main[k] = 208;
    //       sprite_A[k] = sprite_oam_flags[k];
    //     }
    //     break;
    //   case 1:  // flashing
    //     sprite_oam_flags[k] = (sprite_oam_flags[k] & ~0xe) | ((k << 1 ^ frame_counter) & 0xe);
    //     if (!sprite_delay_main[k]) {
    //       sprite_ai_state[k] = 2;
    //       sprite_oam_flags[k] = sprite_A[k];
    //     }
    //     break;
    //   case 2:  // fly
    //     Sprite_MoveXY(k);
    //     if (!sprite_delay_main[k]) {
    //       sprite_x_vel[k] <<= 1;
    //       sprite_y_vel[k] <<= 1;
    //       sprite_delay_main[k] = 6;
    //     }
    //     sprite_E[k]++;
    //     if (sprite_E[k] == 0) sprite_state[k] = 0;
    //     break;
    //   }
    // }
    pub(super) fn sprite_master_sword_prop(&mut self, k: usize) {
        self.oam_allocate_from_region_b(4);
        self.sprite_draw_single_large_for_world(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_move_xy(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 208;
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_OAM_FLAGS + k];
                }
            }
            1 => {
                let mix = (((k as u8) << 1) ^ self.ram[FRAME_COUNTER]) & 0xe;
                self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0xe) | mix;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_OAM_FLAGS + k] = self.ram[SPRITE_A + k];
                }
            }
            2 => {
                self.sprite_move_xy(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_shl(1);
                    self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_shl(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 6;
                }
                self.ram[SPRITE_E + k] = self.ram[SPRITE_E + k].wrapping_add(1);
                if self.ram[SPRITE_E + k] == 0 {
                    self.ram[SPRITE_STATE + k] = 0;
                }
            }
            _ => {}
        }
    }

    // void MasterSword_Draw(int k) {  // 858da8
    //   See sprite_main.c:2353..2368 — writes 6 OAM entries by hand and then
    //   calls Sprite_CorrectOamEntries(k, 5, 0).
    pub(super) fn master_sword_draw(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        // Write six OAM entries (i = 5 down to 0). Each entry is 4 bytes at
        // OAM_CUR_PTR; we advance after each write to match the C pointer
        // increment.
        let mut oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        for i in (0..=5usize).rev() {
            let ex = x.wrapping_add(K_MASTER_SWORD_DRAW_X[i] as i16 as u16);
            let ey = y.wrapping_add(K_MASTER_SWORD_DRAW_Y[i] as i16 as u16);
            self.set_oam_helper0_at(oam, ex, ey, K_MASTER_SWORD_DRAW_CHAR[i], flags, 0);
            oam += 4;
        }
        self.sprite_correct_oam_entries_for_world(k, 5, 0);
    }

    // ============================================================
    // Flute cluster (sprite_main.c:1010, 9828..9979)
    // ============================================================

    // bool FluteBoy_CheckIfPlayerClose(int k) {  // sprite_main.c:1010
    //   int xx = Sprite_GetX(k);
    //   int yy = Sprite_GetY(k) - 16;
    //   int x = link_x_coord - xx - (yy < 0); // zelda bug: carry
    //   int y = link_y_coord - yy - (x < 0);
    //   if (sign16(x)) x = ~x;
    //   if (sign16(y)) y = ~y;
    //   return (uint16)x < 48 && (uint16)y < 48;
    // }
    pub(super) fn flute_boy_check_if_player_close(&self, k: usize) -> bool {
        let xx = self.sprite_get_x(k) as i32;
        let yy = (self.sprite_get_y(k) as i32) - 16;
        let lx = self.player_state_view().x() as i32;
        let ly = self.player_state_view().y() as i32;
        let yy_neg = i32::from(yy < 0);
        let mut x = lx - xx - yy_neg;
        let x_neg = i32::from(x < 0);
        let mut y = ly - yy - x_neg;
        if sign16(x as u16) {
            x = !x;
        }
        if sign16(y as u16) {
            y = !y;
        }
        (x as u16) < 48 && (y as u16) < 48
    }

    // void FluteKid_Human(int k) {  // 86af51
    //   See sprite_main.c:9828..9879 for the full state machine.
    pub(super) fn flute_kid_human(&mut self, k: usize) {
        if self.ram[SPRITE_AI_STATE + k] != 3 {
            self.ram[SPRITE_C + k] = self.flute_boy_draw_for_world(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_C + k] == 0 && self.ram[SPRITE_B + k] == 0 {
            self.ram[SOUND_EFFECT_AMBIENT] = 11;
            self.ram[SPRITE_B + k] = 11;
        }
        self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER] >> 5) & 1;
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let close = self.flute_boy_check_if_player_close(k);
                if self.ram[LINK_ITEM_FLUTE] >= 2 || close {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k].wrapping_add(1);
                    self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] =
                        self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 176;
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 25;
                    self.flute_kid_spawn_quaver(k);
                }
            }
            1 => {
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[TS_COPY] = 2;
                    self.ram[CGADSUB_COPY] = 48;
                    self.ram[PALETTE_FILTER_COUNTDOWN] = 0;
                    self.ram[DARKENING_OR_LIGHTENING_SCREEN] = 0;
                    self.palette_assert_translucency_swap();
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SOUND_EFFECT_AMBIENT] = 128;
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x33);
                }
            }
            2 => {
                if (self.ram[FRAME_COUNTER] & 15) == 0 {
                    self.palette_filter_sp5f_for_world();
                    if self.ram[PALETTE_FILTER_COUNTDOWN] == 0 {
                        self.ram[SPRITE_AI_STATE + k] = 3;
                    }
                }
            }
            3 => {
                self.palette_filter_restore_sp5f_for_world();
                self.palette_revert_translucency_swap();
                self.ram[SPRITE_STATE + k] = 0;
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            }
            _ => {}
        }
    }

    // void Sprite_FluteKid_Stumpy(int k) {  // 86b040
    //   See sprite_main.c:9881..9954 for the conversation / shovel sequence.
    pub(super) fn sprite_flute_kid_stumpy(&mut self, k: usize) {
        const K_FLUTE_AARDVARK_GFX: [i8; 20] =
            [1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2, 3, 2, 3, 2, 3, 2, 3, 2, -1];
        const K_FLUTE_AARDVARK_DELAY: [i8; 19] = [
            -1, -1, -1, 16, 2, 12, 6, 8, 10, 4, 14, 2, 10, 6, 6, 10, 2, 14, 2,
        ];
        self.flute_aardvark_draw_for_world(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => match self.ram[LINK_ITEM_FLUTE] & 3 {
                0 => {
                    // supplicate
                    if (self.sprite_show_solicited_message_for_world(k, 0xe5) & 0x100) != 0 {
                        self.ram[SPRITE_AI_STATE + k] = 1;
                    }
                }
                1 => {
                    // give me flute
                    self.sprite_show_solicited_message_for_world(k, 0xe8);
                }
                2 => {
                    // thanks
                    self.ram[SPRITE_GRAPHICS + k] = 1;
                    if (self.sprite_show_solicited_message_for_world(k, 0xe9) & 0x100) != 0 {
                        self.ram[SPRITE_AI_STATE + k] = 3;
                    }
                }
                3 => {
                    // already did
                    self.ram[SPRITE_GRAPHICS + k] = 3;
                }
                _ => {}
            },
            1 => {
                if self.ram[CHOICE_IN_MULTISELECT_BOX] == 0 {
                    self.sprite_show_message_unconditional(0xe6);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                } else {
                    self.sprite_show_message_unconditional(0xe7);
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
            }
            2 => {
                // grant shovel
                self.ram[ITEM_RECEIPT_METHOD] = 0;
                self.link_receive_item(0x13, 0);
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
            3 => {
                // wait for music
                if self.ram[HUD_CUR_ITEM] == K_HUD_ITEM_FLUTE
                    && (self.ram[JOYPAD1H_LAST] & 0x40) != 0
                {
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[MUSIC_CONTROL] = 0xf2;
                    self.ram[SOUND_EFFECT_1] = 0;
                    self.ram[SOUND_EFFECT_AMBIENT] = 23;
                    self.ram[FLAG_IS_LINK_IMMOBILIZED] =
                        self.ram[FLAG_IS_LINK_IMMOBILIZED].wrapping_add(1);
                }
            }
            4 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    if self.ram[SPRITE_A + k] >= 3 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x33);
                    }
                    let idx = self.ram[SPRITE_A + k] as usize;
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                    let gfx = K_FLUTE_AARDVARK_GFX[idx];
                    if gfx >= 0 {
                        self.ram[SPRITE_GRAPHICS + k] = gfx as u8;
                        self.ram[SPRITE_DELAY_MAIN + k] = K_FLUTE_AARDVARK_DELAY[idx] as u8;
                    } else {
                        self.ram[MUSIC_CONTROL] = 0xf3;
                        self.ram[SPRITE_AI_STATE + k] = 5;
                        self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                    }
                }
            }
            5 => {
                // done
                self.ram[SPRITE_GRAPHICS + k] = 3;
                self.ram[SRAM_PROGRESS_INDICATOR_3] |= 8;
            }
            _ => {}
        }
    }

    // void Sprite_FluteKid_Quaver(int k) {  // 86b173
    //   SpriteDraw_SingleSmall(k);
    //   if (Sprite_ReturnIfInactive(k)) return;
    //   Sprite_MoveXY(k);
    //   Sprite_MoveZ(k);
    //   if (!sprite_delay_main[k]) sprite_state[k] = 0;
    //   if (!(frame_counter & 1))
    //     sprite_x_vel[k] += (frame_counter >> 5 ^ cur_object_index) & 1 ? -1 : 1;
    // }
    pub(super) fn sprite_flute_kid_quaver(&mut self, k: usize) {
        self.sprite_draw_single_small_for_world(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
        if (self.ram[FRAME_COUNTER] & 1) == 0 {
            let cur = self.ram[CUR_OBJECT_INDEX];
            let bit = ((self.ram[FRAME_COUNTER] >> 5) ^ cur) & 1;
            let delta: u8 = if bit != 0 { 0xff } else { 1 };
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(delta);
        }
    }

    // void FluteKid_SpawnQuaver(int k) {  // 86b1a5
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x2e, &info);
    //   if (j >= 0) {
    //     Sprite_SetX(j, info.r0_x + 4);
    //     Sprite_SetY(j, info.r2_y - 4);
    //     sprite_head_dir[j] = 1;
    //     sprite_z_vel[j] = 8;
    //     sprite_delay_main[j] = 96;
    //     sprite_ignore_projectile[j] = 96;
    //   }
    // }
    pub(super) fn flute_kid_spawn_quaver(&mut self, k: usize) {
        if let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x2e) {
            self.sprite_set_x(j, r0_x.wrapping_add(4));
            self.sprite_set_y(j, r2_y.wrapping_sub(4));
            self.ram[SPRITE_HEAD_DIR + j] = 1;
            self.ram[SPRITE_Z_VEL + j] = 8;
            self.ram[SPRITE_DELAY_MAIN + j] = 96;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 96;
        }
    }

    // ------------------------------------------------------------------
    // Helper used by SomariaPlatform_LocatePath / Sprite_AE_Pipe_Down.
    // Mirrors SomariaPlatformAndPipe_HandleMovement (sprite_main.c:25524) so
    // SomariaPlatform_LocatePath stays self-contained.
    // ------------------------------------------------------------------
    // void SomariaPlatformAndPipe_HandleMovement(int k) {  // 9ef7af
    //   static const int8 kSomariaPlatform_Xvel[8] = {0, 0, -16, 16, -16, 16, 16, -16};
    //   static const int8 kSomariaPlatform_Yvel[8] = {-16, 16, 0, 0, -16, 16, -16, 16};
    //   SomariaPlatform_HandleJunctions(k);
    //   int j = sprite_D[k];
    //   sprite_x_vel[k] = kSomariaPlatform_Xvel[j];
    //   sprite_y_vel[k] = kSomariaPlatform_Yvel[j];
    // }
    fn somaria_platform_and_pipe_handle_movement(&mut self, k: usize) {
        const XVEL: [i8; 8] = [0, 0, -16, 16, -16, 16, 16, -16];
        const YVEL: [i8; 8] = [-16, 16, 0, 0, -16, 16, -16, 16];
        self.somaria_platform_handle_junctions(k);
        let j = self.ram[SPRITE_D + k] as usize;
        self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
        self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
    }

    // void Pipe_HandlePlayerMovement(uint8 dir) {  // 9efcff
    //   link_direction_last = link_direction = dir;
    //   Link_HandleVelocity();
    //   Link_HandleMovingAnimation_FullLongEntry();
    //   HandleIndoorCameraAndDoors();
    // }
    pub(super) fn pipe_handle_player_movement(&mut self, dir: u8) {
        self.ram[LINK_DIRECTION] = dir;
        self.ram[LINK_DIRECTION_LAST] = dir;
        self.link_handle_velocity();
        self.link_handle_moving_animation_full_long_entry();
        self.handle_indoor_camera_and_doors();
    }

    // void Faerie_HandleMovement(int k) {  // 9efd1c
    //   sprite_graphics[k] = frame_counter >> 3 & 1;
    //   if (player_is_indoors && !sprite_delay_aux1[k]) {
    //     if (Sprite_CheckTileCollision(k) & 3) {
    //       sprite_x_vel[k] = -sprite_x_vel[k];
    //       sprite_D[k] = -sprite_D[k];
    //       sprite_delay_aux1[k] = 32;
    //     }
    //     if (sprite_wallcoll[k] & 12) {
    //       sprite_y_vel[k] = -sprite_y_vel[k];
    //       sprite_A[k] = -sprite_A[k];
    //       sprite_delay_aux1[k] = 32;
    //     }
    //   }
    //   if (sprite_x_vel[k]) {
    //     if (sign8(sprite_x_vel[k]))
    //       sprite_oam_flags[k] &= ~0x40;
    //     else
    //       sprite_oam_flags[k] |= 0x40;
    //   }
    //   Sprite_MoveXY(k);
    //   if (!(frame_counter & 63)) {
    //     uint16 x = (link_x_coord & ~0xff) + GetRandomNumber();
    //     uint16 y = (link_y_coord & ~0xff) + GetRandomNumber();
    //     ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, x, y, 16);
    //     sprite_A[k] = pt.y;
    //     sprite_D[k] = pt.x;
    //   }
    //   if (!(frame_counter & 15)) {
    //     sprite_y_vel[k] = ((int8)sprite_A[k] + (int8)sprite_y_vel[k]) >> 1;
    //     sprite_x_vel[k] = ((int8)sprite_D[k] + (int8)sprite_x_vel[k]) >> 1;
    //   }
    //   Sprite_MoveZ(k);
    //   sprite_z_vel[k] += (GetRandomNumber() & 1) ? -1 : 1;
    //   if (sprite_z[k] < 8) {
    //     sprite_z[k] = 8;
    //     sprite_z_vel[k] = 5;
    //   } else if (sprite_z[k] >= 24) {
    //     sprite_z[k] = 24;
    //     sprite_z_vel[k] = -5;
    //   }
    // }
    pub(super) fn faerie_handle_movement(&mut self, k: usize) {
        self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER] >> 3) & 1;
        if self.ram[PLAYER_IS_INDOORS] != 0 && self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
            if (self.sprite_check_tile_collision(k) & 3) != 0 {
                self.ram[SPRITE_X_VEL + k] = 0u8.wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                self.ram[SPRITE_D + k] = 0u8.wrapping_sub(self.ram[SPRITE_D + k]);
                self.ram[SPRITE_DELAY_AUX1 + k] = 32;
            }
            if (self.ram[SPRITE_WALLCOLL + k] & 12) != 0 {
                self.ram[SPRITE_Y_VEL + k] = 0u8.wrapping_sub(self.ram[SPRITE_Y_VEL + k]);
                self.ram[SPRITE_A + k] = 0u8.wrapping_sub(self.ram[SPRITE_A + k]);
                self.ram[SPRITE_DELAY_AUX1 + k] = 32;
            }
        }
        if self.ram[SPRITE_X_VEL + k] != 0 {
            if (self.ram[SPRITE_X_VEL + k] as i8).is_negative() {
                self.ram[SPRITE_OAM_FLAGS + k] &= !0x40;
            } else {
                self.ram[SPRITE_OAM_FLAGS + k] |= 0x40;
            }
        }
        self.sprite_move_xy(k);
        if (self.ram[FRAME_COUNTER] & 63) == 0 {
            let x = (self.player_state_view().x() & !0xff)
                .wrapping_add(u16::from(self.get_random_number()));
            let y = (self.player_state_view().y() & !0xff)
                .wrapping_add(u16::from(self.get_random_number()));
            let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
            self.ram[SPRITE_A + k] = pt.y;
            self.ram[SPRITE_D + k] = pt.x;
        }
        if (self.ram[FRAME_COUNTER] & 15) == 0 {
            self.ram[SPRITE_Y_VEL + k] =
                (((self.ram[SPRITE_A + k] as i8) + (self.ram[SPRITE_Y_VEL + k] as i8)) >> 1) as u8;
            self.ram[SPRITE_X_VEL + k] =
                (((self.ram[SPRITE_D + k] as i8) + (self.ram[SPRITE_X_VEL + k] as i8)) >> 1) as u8;
        }
        self.sprite_move_z(k);
        self.ram[SPRITE_Z_VEL + k] =
            self.ram[SPRITE_Z_VEL + k].wrapping_add(if (self.get_random_number() & 1) != 0 {
                -1i8
            } else {
                1i8
            } as u8);
        if self.ram[SPRITE_Z + k] < 8 {
            self.ram[SPRITE_Z + k] = 8;
            self.ram[SPRITE_Z_VEL + k] = 5;
        } else if self.ram[SPRITE_Z + k] >= 24 {
            self.ram[SPRITE_Z + k] = 24;
            self.ram[SPRITE_Z_VEL + k] = (-5i8) as u8;
        }
    }

    // ------------------------------------------------------------------
    // `_for_world` shim helpers. They stand in for canonical helpers that
    // either haven't been ported yet or whose call graph reaches outside
    // the world cluster. Each rewires to the closest canonical port when
    // one exists.
    // ------------------------------------------------------------------

    // uint8 SomariaPlatformAndPipe_CheckTile(int k) {  // 9ef7c2
    //   uint16 x = Sprite_GetX(k), y = Sprite_GetY(k);
    //   return GetTileAttribute(0, &x, y);
    // }
    pub(super) fn somaria_platform_and_pipe_check_tile(&mut self, k: usize) -> u8 {
        let mut x = self.sprite_get_x(k);
        let y = self.sprite_get_y(k);
        self.GetTileAttribute(0, &mut x, y)
    }

    fn somaria_platform_and_pipe_check_tile_for_world(&mut self, k: usize) -> u8 {
        self.somaria_platform_and_pipe_check_tile(k)
    }

    // Rewired to canonical Sprite_SpawnDynamically port. The C variant
    // uses j_max=15; the prior `_for_world` shim historically used 13
    // (matching SpawnDynamicallyEx's 13-slot variant). The canonical
    // helper uses j_max=15 to match the sprite.c entry point.
    fn sprite_spawn_dynamically_for_world(
        &mut self,
        k: usize,
        what: u8,
    ) -> Option<(usize, u16, u16)> {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, what, &mut info);
        if j < 0 {
            None
        } else {
            Some((j as usize, info.r0_x, info.r2_y))
        }
    }

    fn sprite_set_spawned_coordinates_for_world(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    // Rewired to canonical single-tile draw ports.
    fn sprite_draw_single_large_for_world(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
    }
    fn sprite_draw_single_small_for_world(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
    }

    // Rewired to canonical Sprite_CorrectOamEntries port.
    fn sprite_correct_oam_entries_for_world(&mut self, k: usize, count: u8, mask: u8) {
        self.sprite_correct_oam_entries(k, count as i32, mask);
    }

    // Rewired to canonical SpriteDraw_LightFountain port.
    fn sprite_draw_light_fountain_for_world(&mut self, k: usize) {
        self.sprite_draw_light_fountain(k);
    }

    // Rewired to canonical Sprite_CheckDamageToLink_same_layer port
    // (sprite.c:2535).
    fn sprite_check_damage_to_link_same_layer_for_world(&mut self, k: usize) -> bool {
        self.sprite_check_damage_to_link_same_layer(k)
    }

    // Rewired to canonical FluteBoy_Draw port.
    fn flute_boy_draw_for_world(&mut self, k: usize) -> u8 {
        self.flute_boy_draw(k)
    }

    // Rewired to canonical FluteAardvark_Draw port.
    fn flute_aardvark_draw_for_world(&mut self, k: usize) {
        self.flute_aardvark_draw(k);
    }

    // Rewired to canonical Sprite_ShowSolicitedMessage port.
    fn sprite_show_solicited_message_for_world(&mut self, k: usize, msg: u16) -> u16 {
        self.sprite_show_solicited_message(k, msg)
    }

    // Rewired to canonical PaletteFilter_SP5F / PaletteFilter_RestoreSP5F ports.
    fn palette_filter_sp5f_for_world(&mut self) {
        self.PaletteFilter_SP5F();
    }

    fn palette_filter_restore_sp5f_for_world(&mut self) {
        self.PaletteFilter_RestoreSP5F();
    }
}

// hud.h:9 - kHudItem_Flute.
const K_HUD_ITEM_FLUTE: u8 = 13;

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

    #[test]
    fn somaria_platform_handle_drag_x_writes_drag_when_dirs_oppose() {
        let mut s = fresh_state();
        // sprite_D[k] = 0, sprite_head_dir[k] = 2 → XOR&2 == 2 (truthy).
        s.ram[SPRITE_D + 3] = 0;
        s.ram[SPRITE_HEAD_DIR + 3] = 2;
        // sprite_x_lo[k] = 0 — x will become 4, t = 4.
        s.ram[SPRITE_X_LO + 3] = 0;
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        s.somaria_platform_handle_drag_x(3);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 4);
        assert_eq!(s.ram[SPRITE_X_LO + 3], 4);
    }

    #[test]
    fn somaria_platform_handle_drag_returns_when_aligned() {
        let mut s = fresh_state();
        // Bit 2 of XOR is zero → no drag.
        s.ram[SPRITE_D + 1] = 0;
        s.ram[SPRITE_HEAD_DIR + 1] = 1;
        s.ram[SPRITE_X_LO + 1] = 0;
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        s.somaria_platform_handle_drag_x(1);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 0);
        assert_eq!(s.ram[SPRITE_X_LO + 1], 0);
    }

    #[test]
    fn somaria_platform_handle_junctions_b2_xors_d_with_3() {
        let mut s = fresh_state();
        s.ram[SPRITE_E + 5] = 0xb2;
        s.ram[SPRITE_D + 5] = 1;
        s.somaria_platform_handle_junctions(5);
        assert_eq!(s.ram[SPRITE_D + 5], 1 ^ 3);
    }

    #[test]
    fn somaria_platform_handle_junctions_b6_clears_ai_state_when_correct_key_pressed() {
        let mut s = fresh_state();
        s.ram[SPRITE_E + 0] = 0xb6;
        s.ram[SPRITE_D + 0] = 0;
        s.ram[LINK_AUXILIARY_STATE] = 0;
        // kSomariaPlatform_TransitDir[0] = 4; press that bit.
        s.ram[JOYPAD1H_LAST] = 4;
        s.somaria_platform_handle_junctions(0);
        assert_eq!(s.ram[SPRITE_AI_STATE + 0], 0);
        assert_eq!(s.ram[SPRITE_D + 0], 0 ^ 1);
        assert_eq!(s.ram[PLAYER_ON_SOMARIA_PLATFORM], 1);
    }

    #[test]
    fn master_sword_main_clears_state_when_event_bit_set() {
        let mut s = fresh_state();
        s.ram[MAIN_MODULE_INDEX] = 9; // not 26
        s.ram[OVERWORLD_SCREEN_INDEX] = 0x02;
        s.ram[SAVE_OW_EVENT_INFO + 0x02] = 0x40;
        s.ram[SPRITE_STATE + 4] = 9;
        s.master_sword_main(4);
        assert_eq!(s.ram[SPRITE_STATE + 4], 0);
    }

    #[test]
    fn master_sword_spawn_pendant_prop_sets_velocities_from_ain() {
        let mut s = fresh_state();
        write_le_u16(&mut s.ram, LINK_X_COORD, 100);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 50);
        // Canonical Sprite_SpawnDynamically walks j_in=15 down; the highest
        // free slot in 0..=15 wins.
        for j in 0..=15 {
            s.ram[SPRITE_STATE + j] = 0;
        }
        s.master_sword_spawn_pendant_prop(0, 9);
        // ain=9 → (9>>1)&3 = 4 & 3 = 0 → xv = -4, yv = -2.
        // The spawn lands in slot 15 (highest free under canonical helper).
        assert_eq!(s.ram[SPRITE_GRAPHICS + 15], 4);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + 15], 3);
        assert_eq!(s.ram[SPRITE_FLAGS2 + 15], 64);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + 15], 228);
        assert_eq!(s.ram[SPRITE_OAM_FLAGS + 15], 9);
        assert_eq!(s.ram[SPRITE_X_VEL + 15] as i8, -4);
        assert_eq!(s.ram[SPRITE_Y_VEL + 15] as i8, -2);
    }

    #[test]
    fn master_sword_prop_state_2_doubles_velocity_when_delay_zero() {
        let mut s = fresh_state();
        s.ram[SPRITE_AI_STATE + 6] = 2;
        s.ram[SPRITE_DELAY_MAIN + 6] = 0;
        s.ram[SPRITE_X_VEL + 6] = 3;
        s.ram[SPRITE_Y_VEL + 6] = 5;
        s.ram[SPRITE_E + 6] = 7;
        s.sprite_master_sword_prop(6);
        assert_eq!(s.ram[SPRITE_X_VEL + 6], 6);
        assert_eq!(s.ram[SPRITE_Y_VEL + 6], 10);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + 6], 6);
        assert_eq!(s.ram[SPRITE_E + 6], 8);
    }

    #[test]
    fn flute_boy_check_if_player_close_true_when_within_48() {
        let mut s = fresh_state();
        // Place sprite at (100, 100) → yy = 84. link at (130, 130).
        s.ram[SPRITE_X_LO + 2] = 100;
        s.ram[SPRITE_Y_LO + 2] = 100;
        s.ram[SPRITE_X_HI + 2] = 0;
        s.ram[SPRITE_Y_HI + 2] = 0;
        write_le_u16(&mut s.ram, LINK_X_COORD, 130);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 130);
        assert!(s.flute_boy_check_if_player_close(2));
    }

    #[test]
    fn flute_boy_check_if_player_close_false_when_distant() {
        let mut s = fresh_state();
        s.ram[SPRITE_X_LO + 2] = 0;
        s.ram[SPRITE_Y_LO + 2] = 0;
        write_le_u16(&mut s.ram, LINK_X_COORD, 200);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 200);
        assert!(!s.flute_boy_check_if_player_close(2));
    }

    #[test]
    fn flute_kid_spawn_quaver_initializes_z_vel_and_delay() {
        let mut s = fresh_state();
        // Canonical Sprite_SpawnDynamically uses j_in=15 and reads coords
        // from sprite_x_lo[k]/sprite_x_hi[k] (Sprite_GetX), so seed slot 0.
        s.ram[SPRITE_X_LO + 0] = 0x00;
        s.ram[SPRITE_X_HI + 0] = 0x01; // = 0x100
        s.ram[SPRITE_Y_LO + 0] = 0x80;
        s.ram[SPRITE_Y_HI + 0] = 0x00; // = 0x080
        for j in 0..=15 {
            s.ram[SPRITE_STATE + j] = 0;
        }
        s.flute_kid_spawn_quaver(0);
        // Canonical helper picks slot 15 (highest free in 0..=15).
        assert_eq!(s.ram[SPRITE_HEAD_DIR + 15], 1);
        assert_eq!(s.ram[SPRITE_Z_VEL + 15], 8);
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + 15], 96);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 15], 96);
        assert_eq!(s.ram[SPRITE_TYPE + 15], 0x2e);
    }

    #[test]
    fn sprite_flute_kid_quaver_zeros_state_when_delay_zero() {
        let mut s = fresh_state();
        s.ram[SPRITE_STATE + 4] = 9;
        s.ram[SPRITE_DELAY_MAIN + 4] = 0;
        s.ram[FRAME_COUNTER] = 0; // even → adjust x_vel
        s.ram[SPRITE_X_VEL + 4] = 10;
        s.ram[CUR_OBJECT_INDEX] = 0;
        s.sprite_flute_kid_quaver(4);
        // Sprite_ReturnIfInactive is false (state != 0 and other guards), so
        // the body executes. Delay==0 → state cleared.
        assert_eq!(s.ram[SPRITE_STATE + 4], 0);
    }

    #[test]
    fn sprite_flute_kid_stumpy_starts_music_when_flute_equipped_and_y_pressed() {
        let mut s = fresh_state();
        let k = 3;

        s.ram[SPRITE_STATE + k] = 9;
        s.ram[SPRITE_AI_STATE + k] = 3;
        s.ram[HUD_CUR_ITEM] = K_HUD_ITEM_FLUTE;
        s.ram[JOYPAD1H_LAST] = 0x40;

        s.sprite_flute_kid_stumpy(k);

        assert_eq!(s.ram[SPRITE_AI_STATE + k], 4);
        assert_eq!(s.ram[MUSIC_CONTROL], 0xf2);
        assert_eq!(s.ram[SOUND_EFFECT_1], 0);
        assert_eq!(s.ram[SOUND_EFFECT_AMBIENT], 23);
        assert_eq!(s.ram[FLAG_IS_LINK_IMMOBILIZED], 1);
    }

    #[test]
    fn somaria_platform_drag_link_subtracts_when_link_north_west_of_sprite() {
        let mut s = fresh_state();
        // cur_sprite=(100,200), link=(20,80). x = 92-20 = 72 > 0 (no high
        // bit) → +1. y = 184-80 = 104 > 0 → +1.
        write_le_u16(&mut s.ram, CUR_SPRITE_X, 100);
        write_le_u16(&mut s.ram, CUR_SPRITE_Y, 200);
        write_le_u16(&mut s.ram, LINK_X_COORD, 20);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 80);
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        write_le_u16(&mut s.ram, DRAG_PLAYER_Y, 0);
        s.somaria_platform_drag_link(0);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 1);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_Y), 1);
    }

    #[test]
    fn somaria_platform_handle_junctions_bc_no_input_sets_ai_state_1() {
        let mut s = fresh_state();
        s.ram[SPRITE_E + 2] = 0xbc;
        s.ram[SPRITE_D + 2] = 0;
        s.ram[JOYPAD1H_LAST] = 0;
        s.somaria_platform_handle_junctions(2);
        // KEYS6[0] = 0xc; t = 0; ai_state stays 1, player_on flag set.
        assert_eq!(s.ram[SPRITE_AI_STATE + 2], 1);
        assert_eq!(s.ram[PLAYER_ON_SOMARIA_PLATFORM], 1);
    }

    #[test]
    fn pipe_handle_player_movement_sets_direction_and_runs_player_motion_tail() {
        let mut s = fresh_state();
        s.ram[PLAYER_IS_INDOORS] = 0;
        s.ram[LINK_SPEED_SETTING] = 0;
        s.ram[LINK_DIRECTION_FACING] = 0;
        s.ram[LINK_DIRECTION] = 0;
        s.ram[LINK_DIRECTION_LAST] = 0;
        write_le_u16(&mut s.ram, LINK_X_COORD, 0x0120);
        write_le_u16(&mut s.ram, LINK_Y_COORD, 0x0230);

        s.pipe_handle_player_movement(2);

        assert_eq!(s.ram[LINK_DIRECTION], 2);
        assert_eq!(s.ram[LINK_DIRECTION_LAST], 2);
        // Link_HandleMovingAnimation_FullLongEntry maps left/right movement
        // onto facing 4/6 when direction changes are allowed.
        assert_eq!(s.ram[LINK_DIRECTION_FACING], 4);
        assert_eq!(read_le_u16(&s.ram, LINK_X_COORD), 0x011e);
        assert_eq!(read_le_u16(&s.ram, LINK_Y_COORD), 0x0230);
    }

    #[test]
    fn faerie_handle_movement_animates_moves_and_clamps_low_z() {
        let mut s = fresh_state();
        let k = 3;
        s.ram[FRAME_COUNTER] = 8;
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.ram[SPRITE_X_VEL + k] = 16;
        s.ram[SPRITE_Y_VEL + k] = 0;
        s.ram[SPRITE_Z + k] = 7;
        s.ram[SPRITE_Z_VEL + k] = 0;
        s.ram[SPRITE_OAM_FLAGS + k] = 0;

        s.faerie_handle_movement(k);

        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 1);
        assert_eq!(s.ram[SPRITE_OAM_FLAGS + k] & 0x40, 0x40);
        assert_eq!(s.sprite_get_x(k), 0x0101);
        assert_eq!(s.sprite_get_y(k), 0x0200);
        assert_eq!(s.ram[SPRITE_Z + k], 8);
        assert_eq!(s.ram[SPRITE_Z_VEL + k], 5);
    }

    #[test]
    fn faerie_handle_movement_averages_velocities_on_16_frame_tick() {
        let mut s = fresh_state();
        let k = 4;
        s.ram[FRAME_COUNTER] = 16;
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.ram[SPRITE_X_VEL + k] = 10;
        s.ram[SPRITE_Y_VEL + k] = (-10i8) as u8;
        s.ram[SPRITE_D + k] = 20;
        s.ram[SPRITE_A + k] = (-20i8) as u8;
        s.ram[SPRITE_Z + k] = 12;
        s.ram[SPRITE_Z_VEL + k] = 0;

        s.faerie_handle_movement(k);

        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 0);
        assert_eq!(s.ram[SPRITE_X_VEL + k], 15);
        assert_eq!(s.ram[SPRITE_Y_VEL + k], (-15i8) as u8);
        assert_eq!(s.sprite_get_x(k), 0x0120);
        assert_eq!(s.sprite_get_y(k), 0x022f);
        assert_eq!(s.ram[SPRITE_Z + k], 12);
    }
}
