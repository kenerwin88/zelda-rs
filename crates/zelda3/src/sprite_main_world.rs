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
const PLAYER_STATE_SPIN_ATTACKING: u8 = 3;
const PLAYER_STATE_HOOKSHOT: u8 = 19;

// `sprite_main.c:25503` - SomariaPlatform drag-link motion table.
const SOMARIA_PLATFORM_DRAG_X_OFFSETS: [i8; 8] = [0, 0, -1, 1, -1, 1, 1, -1];
const SOMARIA_PLATFORM_DRAG_Y_OFFSETS: [i8; 8] = [-1, 1, 0, 0, -1, 1, -1, 1];
const PIPE_EXIT_DIRECTION_BITS: [u8; 4] = [8, 4, 2, 1];
const ALT_SPRITE_SPAWNED_FLAG_WORLD: usize = 0x1de0;

// SomariaPlatform draw atlas (sprite_main.c:25539-25555).
const SOMARIA_PLATFORM_DRAW_FRAMES: [DrawMultipleData; 16] = [
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
const MASTER_SWORD_DRAW_X_OFFSETS: [i8; 6] = [-8, 0, -8, 0, -8, 0];
const MASTER_SWORD_DRAW_Y_OFFSETS: [i8; 6] = [-8, -8, 0, 0, 8, 8];
const MASTER_SWORD_DRAW_CHARS: [u8; 6] = [0xc3, 0xc4, 0xd3, 0xd4, 0xe0, 0xf0];

// MasterSword pendant motion tables (sprite_main.c:2302-2303).
const MASTER_SWORD_PENDANT_X_VELOCITIES: [i8; 4] = [-4, 4, 0, 0];
const MASTER_SWORD_PENDANT_Y_VELOCITIES: [i8; 4] = [-2, -2, -4, -4];

// MasterSword light-beam tables (sprite_main.c:2233-2244).
const MASTER_SWORD_LIGHT_BEAM_LEFT_X_VELOCITIES: [i8; 2] = [0, -48];
const MASTER_SWORD_LIGHT_BEAM_RIGHT_X_VELOCITIES: [i8; 2] = [0, 48];
const MASTER_SWORD_LIGHT_BEAM_UPPER_LEFT_X_VELOCITIES: [i8; 2] = [-96, -48];
const MASTER_SWORD_LIGHT_BEAM_UPPER_RIGHT_X_VELOCITIES: [i8; 2] = [96, 48];
const MASTER_SWORD_LIGHT_BEAM_LEFT_Y_VELOCITIES: [i8; 2] = [-96, -48];
const MASTER_SWORD_LIGHT_BEAM_RIGHT_Y_VELOCITIES: [i8; 2] = [96, 48];
const MASTER_SWORD_LIGHT_BEAM_UPPER_LEFT_Y_VELOCITIES: [i8; 2] = [0, 48];
const MASTER_SWORD_LIGHT_BEAM_UPPER_RIGHT_Y_VELOCITIES: [i8; 2] = [0, -48];
const MASTER_SWORD_LIGHT_BEAM_SIDE_GRAPHICS: [u8; 2] = [1, 0];
const MASTER_SWORD_LIGHT_BEAM_UPPER_GRAPHICS: [u8; 2] = [3, 2];
const MASTER_SWORD_LIGHT_BEAM_SIDE_OAM_FLAGS: [u8; 2] = [5, 0x45];
const MASTER_SWORD_LIGHT_BEAM_UPPER_OAM_FLAGS: [u8; 2] = [5, 5];

// MasterSword light-fountain animation tables (sprite_main.c:2143-2144).
const MASTER_SWORD_FOUNTAIN_GRAPHICS: [u8; 9] = [0, 1, 1, 2, 2, 2, 1, 1, 0];
const MASTER_SWORD_FOUNTAIN_LIGHT_BEAM_COUNTS: [u8; 9] = [0, 0, 1, 1, 2, 2, 0, 0, 0];

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
            self.sprite_slot_view_mut(k).set_e(tiletype);
            if (0xb0..0xbf).contains(&tiletype) {
                break;
            }
            let x = self.sprite_get_x(k).wrapping_add(8);
            let y = self.sprite_get_y(k).wrapping_add(8);
            self.sprite_set_x(k, x);
            self.sprite_set_y(k, y);
        }
        let x_low = (self.sprite_slot_view(k).x_low() & !7).wrapping_add(4);
        let y_low = (self.sprite_slot_view(k).y_low() & !7).wrapping_add(4);
        let direction = self.sprite_slot_view(k).direction();
        self.sprite_slot_view_mut(k).set_x_low(x_low);
        self.sprite_slot_view_mut(k).set_y_low(y_low);
        self.sprite_slot_view_mut(k).set_head_direction(direction);
        self.somaria_platform_and_pipe_handle_movement(k);
        let ignore_projectile = self.sprite_slot_view(k).ignore_projectile().wrapping_add(1);
        self.sprite_slot_view_mut(k)
            .set_ignore_projectile(ignore_projectile);
        self.player_state_view_mut().clear_somaria_platform_state();
        self.sprite_slot_view_mut(k).set_delay_aux4(14);
        let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1);
        self.sprite_slot_view_mut(k).set_graphics(graphics);
    }

    // void Sprite_ED_SomariaPlatform(int k) {  // 9ef6d4
    pub(super) fn sprite_ed_somaria_platform(&mut self, k: usize) {
        match self.sprite_slot_view(k).graphics() {
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
                let drag_x = self.player_state_view().drag_player_x();
                let drag_y = self.player_state_view().drag_player_y();
                if (drag_x | drag_y) == 0
                    && sign8(self.player_state_view().near_pit_state().wrapping_sub(2))
                    && self.sprite_check_damage_to_link_ignore_layer(k)
                {
                    self.sprite_slot_view_mut(k).set_c(1);
                    self.link_cancel_dash();
                    if self.player_state_view().handler_state() != PLAYER_STATE_HOOKSHOT
                        && self.player_state_view().handler_state() != PLAYER_STATE_SPIN_ATTACKING
                    {
                        if self.sprite_slot_view(k).ai_state() != 0 {
                            self.somaria_platform_and_pipe_handle_movement(k);
                            return;
                        }
                        self.sprite_slot_view_mut(k).increment_a();
                        self.player_state_view_mut().set_somaria_platform_state(2);
                        if (self.sprite_slot_view(k).a() & 7) == 0 {
                            let a = self.somaria_platform_and_pipe_check_tile(k);
                            if a != self.sprite_slot_view(k).e() {
                                self.sprite_slot_view_mut(k).set_e(a);
                                let direction = self.sprite_slot_view(k).direction();
                                self.sprite_slot_view_mut(k).set_head_direction(direction);
                                self.somaria_platform_and_pipe_handle_movement(k);
                                self.somaria_platform_handle_drag(k);
                            }
                        }
                        if self.world_location_state().dungeon_room_index() != 36 {
                            let j = usize::from(self.sprite_slot_view(k).direction());
                            let dx = self
                                .player_state_view()
                                .drag_player_x()
                                .wrapping_add(SOMARIA_PLATFORM_DRAG_X_OFFSETS[j] as i16 as u16);
                            let dy = self
                                .player_state_view()
                                .drag_player_y()
                                .wrapping_add(SOMARIA_PLATFORM_DRAG_Y_OFFSETS[j] as i16 as u16);
                            self.player_state_view_mut().set_drag_player_x(dx);
                            self.player_state_view_mut().set_drag_player_y(dy);
                            self.sprite_move_xy(k);
                            self.somaria_platform_drag_link(k);
                        } else {
                            self.player_state_view_mut().set_somaria_platform_state(1);
                        }
                        return;
                    }
                }
                if self.sprite_slot_view(k).c() != 0 {
                    self.player_state_view_mut().clear_somaria_platform_state();
                    self.sprite_slot_view_mut(k).set_c(0);
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
        let start = (self.sprite_slot_view(k).delay_aux4() & 12) as usize;
        let end = start + 4;
        self.sprite_draw_multiple(k, &SOMARIA_PLATFORM_DRAW_FRAMES[start..end], None);
    }

    // void SomariaPlatform_HandleJunctions(int k) {  // 9ef87d
    //   See sprite_main.c:25561..25690 for the full switch on sprite_E[k]
    //   covering 0xb2..0xbe.
    pub(super) fn somaria_platform_handle_junctions(&mut self, k: usize) {
        match self.sprite_slot_view(k).e() {
            0xb2 | 0xb5 => {
                // ZigZagRisingSlope
                self.sprite_slot_view_mut(k).xor_direction(3);
            }
            0xb3 | 0xb4 => {
                // ZigZagFallingSlope
                self.sprite_slot_view_mut(k).xor_direction(2);
            }
            0xb6 => {
                // TransitTile
                const TRANSIT_DIR: [u8; 4] = [4, 8, 1, 2];
                self.sprite_slot_view_mut(k).set_ai_state(1);
                let d = self.sprite_slot_view(k).direction() as usize;
                if !self.player_state_view().has_auxiliary_state()
                    && (self.player_state_view().joypad1h_last() & TRANSIT_DIR[d]) != 0
                {
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                    self.sprite_slot_view_mut(k).xor_direction(1);
                }
                {
                    let mut player = self.player_state_view_mut();
                    player.set_visibility_status(0);
                    player.set_somaria_platform_state(1);
                }
            }
            0xb7 => {
                // Tjunc_NoUp
                const KEYS1: [u8; 4] = [3, 7, 6, 5];
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS1[d];
                if (t & 8) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                } else if (t & 4) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(1);
                } else if (t & 2) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                } else if (t & 1) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(3);
                } else if self.sprite_slot_view(k).direction() == 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                }
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            0xb8 => {
                // Tjunc_NoDown
                const KEYS2: [u8; 4] = [11, 3, 10, 9];
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS2[d];
                if (t & 8) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                } else if (t & 4) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(1);
                } else if (t & 2) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                } else if (t & 1) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(3);
                } else if self.sprite_slot_view(k).direction() == 1 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                }
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            0xb9 => {
                // Tjunc_NoLeft
                const KEYS3: [u8; 4] = [9, 5, 12, 13];
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS3[d];
                if (t & 8) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                } else if (t & 4) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(1);
                } else if (t & 2) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                } else if (t & 1) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(3);
                } else if self.sprite_slot_view(k).direction() == 2 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                }
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            0xba => {
                // Tjunc_NoRight
                const KEYS4: [u8; 4] = [0xa, 6, 0xe, 0xc];
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS4[d];
                if (t & 8) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                } else if (t & 4) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(1);
                } else if (t & 2) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                } else if (t & 1) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(3);
                } else if self.sprite_slot_view(k).direction() == 3 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                }
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            0xbb => {
                // TransitTileNoBack
                const KEYS5: [u8; 4] = [0xb, 7, 0xe, 0xd];
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS5[d];
                if (t & 8) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(0);
                } else if (t & 4) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(1);
                } else if (t & 2) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(2);
                } else if (t & 1) != 0 {
                    self.sprite_slot_view_mut(k).set_direction(3);
                }
            }
            0xbc => {
                // TransitTileQuestion
                const KEYS6: [u8; 4] = [0xc, 0xc, 3, 3];
                self.sprite_slot_view_mut(k).set_ai_state(1);
                let d = self.sprite_slot_view(k).direction() as usize;
                let t = self.player_state_view().joypad1h_last() & KEYS6[d];
                if t != 0 {
                    if (t & 8) != 0 {
                        self.sprite_slot_view_mut(k).set_direction(0);
                    } else if (t & 4) != 0 {
                        self.sprite_slot_view_mut(k).set_direction(1);
                    } else if (t & 2) != 0 {
                        self.sprite_slot_view_mut(k).set_direction(2);
                    } else {
                        self.sprite_slot_view_mut(k).set_direction(3);
                    }
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
                self.player_state_view_mut().set_somaria_platform_state(1);
            }
            0xbe => {
                // endpoint
                self.sprite_slot_view_mut(k).set_ai_state(0);
                self.sprite_slot_view_mut(k).xor_direction(1);
                {
                    let mut player = self.player_state_view_mut();
                    player.set_visibility_status(0);
                    player.set_somaria_platform_state(1);
                }
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
        if ((self.sprite_slot_view(k).direction() ^ self.sprite_slot_view(k).head_direction()) & 2)
            != 0
        {
            let x = (self.sprite_slot_view(k).x_low() & !7).wrapping_add(4);
            let t = x.wrapping_sub(self.sprite_slot_view(k).x_low());
            if t == 0 {
                return;
            }
            // drag_player_x is a 16-bit value at 0x0b7c — write t sign-extended.
            let v = t as i8 as i16 as u16;
            self.player_state_view_mut().set_drag_player_x(v);
            self.sprite_slot_view_mut(k).set_x_low(x);
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
        if ((self.sprite_slot_view(k).direction() ^ self.sprite_slot_view(k).head_direction()) & 2)
            != 0
        {
            let y = (self.sprite_slot_view(k).y_low() & !7).wrapping_add(4);
            let t = y.wrapping_sub(self.sprite_slot_view(k).y_low());
            if t == 0 {
                return;
            }
            let v = t as i8 as i16 as u16;
            self.player_state_view_mut().set_drag_player_y(v);
            self.sprite_slot_view_mut(k).set_y_low(y);
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
        let cur_x = self.sprite_workspace_view().current_sprite_x();
        let cur_y = self.sprite_workspace_view().current_sprite_y();
        let lx = self.player_state_view().x();
        let ly = self.player_state_view().y();
        let x = cur_x.wrapping_sub(8).wrapping_sub(lx);
        if x != 0 {
            let delta: u16 = if sign16(x) { 0xffff } else { 1 };
            self.player_state_view_mut().add_drag_player_x(delta);
        }
        let y = cur_y.wrapping_sub(16).wrapping_sub(ly);
        if y != 0 {
            let delta: u16 = if sign16(y) { 0xffff } else { 1 };
            self.player_state_view_mut().add_drag_player_y(delta);
        }
    }

    // void Sprite_AE_Pipe_Down(int k) {  // 9efb7e
    pub(super) fn sprite_ae_pipe_down(&mut self, k: usize) {
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).graphics() {
            0 => {
                self.sprite_system_view_mut()
                    .set_alt_sprite_spawned_flag(255);
                let direction = self.sprite_slot_view(k).sprite_type().wrapping_sub(0xae);
                self.sprite_slot_view_mut(k).set_direction(direction);
                self.somaria_platform_locate_path(k);
            }
            1 => {
                let mut t = self.somaria_platform_and_pipe_check_tile(k);
                if t == 0xbe {
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1);
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                    self.sprite_slot_view_mut(k).xor_direction(1);
                    t = self.sprite_slot_view(k).direction();
                }
                self.sprite_slot_view_mut(k).set_e(t);
                let direction = self.sprite_slot_view(k).direction();
                self.sprite_slot_view_mut(k).set_head_direction(direction);
                self.somaria_platform_and_pipe_handle_movement(k);
                self.sprite_move_xy(k);
            }
            2 => {
                if self.sprite_system_view().alt_sprite_spawned_flag() == 255
                    && self.sprite_check_damage_to_link_ignore_layer(k)
                {
                    if !self.pipe_validate_entry() {
                        let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1);
                        self.sprite_slot_view_mut(k).set_graphics(graphics);
                        self.sprite_slot_view_mut(k).set_delay_aux1(4);
                        self.link_reset_properties_a();
                        self.player_state_view_mut().immobilize();
                        self.player_state_view_mut()
                            .set_sprite_damage_disable_timer(1);
                        self.sprite_system_view_mut()
                            .set_alt_sprite_spawned_flag(k as u8);
                    } else {
                        self.sprite_halt_all_movement();
                    }
                }
            }
            3 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1);
                    self.sprite_slot_view_mut(k).set_graphics(graphics);
                    self.player_state_view_mut().set_visibility_status(12);
                } else {
                    self.player_state_view_mut().immobilize();
                    self.player_state_view_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.pipe_handle_player_movement(
                        PIPE_EXIT_DIRECTION_BITS[usize::from(self.sprite_slot_view(k).direction())],
                    );
                }
            }
            4 => {
                self.sprite_slot_view_mut(k).set_subtype2(3);
                self.player_state_view_mut()
                    .cache_safe_return_position_from_current();
                loop {
                    self.sprite_slot_view_mut(k).increment_a();
                    if (self.sprite_slot_view(k).a() & 7) == 0 {
                        let t = self.somaria_platform_and_pipe_check_tile(k);
                        if (0xb2..0xb6).contains(&t) {
                            self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
                        }
                        if t != self.sprite_slot_view(k).e() {
                            self.sprite_slot_view_mut(k).set_e(t);
                            if t == 0xbe {
                                let graphics = self.sprite_slot_view(k).graphics().wrapping_add(1);
                                self.sprite_slot_view_mut(k).set_graphics(graphics);
                                self.sprite_slot_view_mut(k).set_delay_aux1(24);
                            }
                            let direction = self.sprite_slot_view(k).direction();
                            self.sprite_slot_view_mut(k).set_head_direction(direction);
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
                    if self.sprite_slot_view_mut(k).decrement_subtype2() == 0 {
                        break;
                    }
                }
                {
                    let mut player = self.player_state_view_mut();
                    player.set_x_velocity_from_safe_return_delta();
                    player.set_y_velocity_from_safe_return_delta_unless_ledge_hopping();
                }
                let last_direction =
                    PIPE_EXIT_DIRECTION_BITS[usize::from(self.sprite_slot_view(k).direction())];
                self.player_state_view_mut()
                    .set_last_direction(last_direction);
                self.link_handle_moving_animation_full_long_entry();
                self.handle_indoor_camera_and_doors();
                self.link_cancel_dash();
            }
            5 => {
                if self.sprite_slot_view(k).delay_aux1() == 0 {
                    self.player_state_view_mut().clear_immobilized();
                    {
                        let mut player = self.player_state_view_mut();
                        player.clear_somaria_platform_state();
                        player.clear_sprite_damage_disable_timer();
                        player.set_visibility_status(0);
                        player.clear_movement_velocity();
                    }
                    self.sprite_system_view_mut()
                        .set_alt_sprite_spawned_flag(255);
                    self.sprite_slot_view_mut(k).set_graphics(2);
                } else {
                    self.pipe_handle_player_movement(
                        PIPE_EXIT_DIRECTION_BITS
                            [usize::from(self.sprite_slot_view(k).direction() ^ 1)],
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
        let ow = self.world_location_state().overworld_screen_index() as usize;
        if self.frame_state().main_module != 26
            && (self.overworld_event_info_view().event_info(ow) & 0x40) != 0
        {
            self.sprite_slot_view_mut(k).set_state(0);
            return;
        }
        if self.sprite_slot_view(k).ai_state() != 5 {
            self.master_sword_draw(k);
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                // waiting
                if self.sprite_check_if_link_is_busy()
                    || !self.sprite_check_damage_to_link_same_layer_for_world(k)
                    || self.player_state_view().facing() != 2
                    || (self.player_state_view().filtered_joypad_l() & 0x80) == 0
                    || (self.player_resources_view().pendant_flags() & 7) != 7
                {
                    return;
                }
                self.system_signals_view_mut().set_music_control(10);
                self.player_state_view_mut()
                    .set_sprite_damage_disable_timer(1);
                self.master_sword_spawn_pendant_prop(k, 9);
                self.master_sword_spawn_pendant_prop(k, 11);
                self.master_sword_spawn_pendant_prop(k, 15);
                self.master_sword_spawn_light_well(k);
                self.sprite_slot_view_mut(k).set_ai_state(1);
                self.sprite_slot_view_mut(k).set_delay_main(240);
            }
            1 => {
                // pendants transfer
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.master_sword_spawn_light_fountain(k);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.sprite_slot_view_mut(k).set_delay_main(192);
                }
                self.player_state_view_mut().set_pull_action_state(10);
                self.player_state_view_mut().immobilize();
            }
            2 => {
                // light show
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.master_sword_spawn_light_beam(k, 0, 0xff);
                    self.sprite_slot_view_mut(k).set_ai_state(3);
                    self.sprite_slot_view_mut(k).set_delay_main(8);
                }
                self.player_state_view_mut().set_pull_action_state(10);
                self.player_state_view_mut().immobilize();
            }
            3 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.master_sword_spawn_light_beam(k, 1, 0xff);
                    self.sprite_slot_view_mut(k).set_ai_state(4);
                    self.sprite_slot_view_mut(k).set_delay_main(16);
                }
                self.player_state_view_mut().set_pull_action_state(11);
                self.player_state_view_mut().immobilize();
            }
            4 => {
                // give to player
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.overworld_event_info_view_mut()
                        .set_event_bits(ow, 0x40);
                    self.player_state_view_mut().set_item_receipt_method(0);
                    self.link_receive_item(1, 0);
                    self.save_progress_view_mut().set_map_icons_indicator(5);
                    self.player_state_view_mut().set_pull_action_state(0);
                    self.sprite_slot_view_mut(k).set_ai_state(5);
                }
            }
            5 => {
                // stop
                self.sprite_slot_view_mut(k).set_state(0);
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
        let new_a = self.sprite_slot_view(k).a().wrapping_add(1);
        self.sprite_slot_view_mut(k).set_a(new_a);
        if new_a == 0 {
            self.sprite_slot_view_mut(k).increment_c();
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).set_direction((new_a >> 2) & 3);
        let j = ((new_a >> 5) & 7) as usize;
        self.sprite_slot_view_mut(k)
            .set_graphics(MASTER_SWORD_FOUNTAIN_GRAPHICS[j]);
        if MASTER_SWORD_FOUNTAIN_LIGHT_BEAM_COUNTS[j] != 0 {
            let ain = (new_a >> 2) & 1;
            self.master_sword_spawn_light_beam(k, ain, MASTER_SWORD_FOUNTAIN_LIGHT_BEAM_COUNTS[j]);
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
        let new_a = self.sprite_slot_view(k).a().wrapping_add(1);
        self.sprite_slot_view_mut(k).set_a(new_a);
        if new_a == 0 {
            self.sprite_slot_view_mut(k).increment_c();
            self.sprite_slot_view_mut(k).set_state(0);
        }
        self.sprite_slot_view_mut(k).set_direction((new_a >> 2) & 3);
        self.sprite_slot_view_mut(k).set_graphics(0);
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
            self.sprite_slot_view_mut(j).set_subtype2(4);
            self.sprite_slot_view_mut(j).set_oam_flags(5);
            self.sprite_slot_view_mut(j).set_flags2(0);
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
            self.sprite_slot_view_mut(j).set_subtype2(1);
            self.sprite_slot_view_mut(j).set_oam_flags(5);
            self.sprite_slot_view_mut(j).set_flags2(0);
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
        if self.sprite_slot_view(k).a() != 0 {
            self.sprite_move_xy(k);
            if (self.frame_state().frame_counter & 3) == 0 {
                self.master_sword_spawn_replacement_light_beam(k);
            } else {
                return;
            }
        }
        let new_b = self.sprite_slot_view(k).b().wrapping_sub(1);
        self.sprite_slot_view_mut(k).set_b(new_b);
        if new_b == 0 {
            self.sprite_slot_view_mut(k).set_state(0);
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
        self.sprite_slot_view_mut(j).set_subtype2(2);
        self.sprite_slot_view_mut(j).set_b(3);
        let graphics = self.sprite_slot_view(k).graphics();
        let oam_flags = self.sprite_slot_view(k).oam_flags();
        self.sprite_slot_view_mut(j).set_graphics(graphics);
        self.sprite_slot_view_mut(j).set_oam_flags(oam_flags);
        self.sprite_slot_view_mut(j).set_flags2(0);
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
        self.sprite_slot_view_mut(j).set_subtype2(2);
        self.sprite_slot_view_mut(j).set_a(2);
        self.sprite_slot_view_mut(j).set_flags2(0);
        self.sprite_slot_view_mut(j)
            .set_x_velocity(MASTER_SWORD_LIGHT_BEAM_LEFT_X_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_y_velocity(MASTER_SWORD_LIGHT_BEAM_LEFT_Y_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_graphics(MASTER_SWORD_LIGHT_BEAM_SIDE_GRAPHICS[ai]);
        self.sprite_slot_view_mut(j)
            .set_oam_flags(MASTER_SWORD_LIGHT_BEAM_SIDE_OAM_FLAGS[ai]);
        self.sprite_slot_view_mut(j).set_b(yin);

        // Spawn 2
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.sprite_slot_view_mut(j).set_subtype2(2);
        self.sprite_slot_view_mut(j).set_a(2);
        self.sprite_slot_view_mut(j).set_flags2(0);
        self.sprite_slot_view_mut(j)
            .set_x_velocity(MASTER_SWORD_LIGHT_BEAM_RIGHT_X_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_y_velocity(MASTER_SWORD_LIGHT_BEAM_RIGHT_Y_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_graphics(MASTER_SWORD_LIGHT_BEAM_SIDE_GRAPHICS[ai]);
        self.sprite_slot_view_mut(j)
            .set_oam_flags(MASTER_SWORD_LIGHT_BEAM_SIDE_OAM_FLAGS[ai]);
        self.sprite_slot_view_mut(j).set_b(yin);

        // Spawn 3
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.sprite_slot_view_mut(j).set_subtype2(2);
        self.sprite_slot_view_mut(j).set_a(2);
        self.sprite_slot_view_mut(j).set_flags2(0);
        self.sprite_slot_view_mut(j)
            .set_x_velocity(MASTER_SWORD_LIGHT_BEAM_UPPER_LEFT_X_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_y_velocity(MASTER_SWORD_LIGHT_BEAM_UPPER_LEFT_Y_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_graphics(MASTER_SWORD_LIGHT_BEAM_UPPER_GRAPHICS[ai]);
        self.sprite_slot_view_mut(j)
            .set_oam_flags(MASTER_SWORD_LIGHT_BEAM_UPPER_OAM_FLAGS[ai]);
        self.sprite_slot_view_mut(j).set_b(yin);

        // Spawn 4
        let Some((j, r0_x, r2_y)) = self.sprite_spawn_dynamically_for_world(k, 0x62) else {
            return;
        };
        self.sprite_set_x(j, r0_x.wrapping_sub(4));
        self.sprite_set_y(j, r2_y.wrapping_add(4));
        self.sprite_slot_view_mut(j).set_subtype2(2);
        self.sprite_slot_view_mut(j).set_a(2);
        self.sprite_slot_view_mut(j).set_flags2(0);
        self.sprite_slot_view_mut(j)
            .set_x_velocity(MASTER_SWORD_LIGHT_BEAM_UPPER_RIGHT_X_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_y_velocity(MASTER_SWORD_LIGHT_BEAM_UPPER_RIGHT_Y_VELOCITIES[ai] as u8);
        self.sprite_slot_view_mut(j)
            .set_graphics(MASTER_SWORD_LIGHT_BEAM_UPPER_GRAPHICS[ai]);
        self.sprite_slot_view_mut(j)
            .set_oam_flags(MASTER_SWORD_LIGHT_BEAM_UPPER_OAM_FLAGS[ai]);
        self.sprite_slot_view_mut(j).set_b(yin);
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
        self.sprite_slot_view_mut(j).set_oam_flags(ain);
        let lx = self.player_state_view().x();
        let ly = self.player_state_view().y();
        self.sprite_set_x(j, lx);
        self.sprite_set_y(j, ly.wrapping_add(8));
        self.sprite_slot_view_mut(j).set_graphics(4);
        self.sprite_slot_view_mut(j).set_subtype2(3);
        self.sprite_slot_view_mut(j).set_flags2(64);
        self.sprite_slot_view_mut(j).set_delay_main(228);
        let i = ((ain >> 1) & 3) as usize;
        self.sprite_slot_view_mut(j)
            .set_x_velocity(MASTER_SWORD_PENDANT_X_VELOCITIES[i] as u8);
        self.sprite_slot_view_mut(j)
            .set_y_velocity(MASTER_SWORD_PENDANT_Y_VELOCITIES[i] as u8);
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
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                self.sprite_move_xy(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                    self.sprite_slot_view_mut(k).set_delay_main(208);
                    let oam_flags = self.sprite_slot_view(k).oam_flags();
                    self.sprite_slot_view_mut(k).set_a(oam_flags);
                }
            }
            1 => {
                let mix = (((k as u8) << 1) ^ self.frame_state().frame_counter) & 0xe;
                let oam_flags = (self.sprite_slot_view(k).oam_flags() & !0xe) | mix;
                self.sprite_slot_view_mut(k).set_oam_flags(oam_flags);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    let a = self.sprite_slot_view(k).a();
                    self.sprite_slot_view_mut(k).set_oam_flags(a);
                }
            }
            2 => {
                self.sprite_move_xy(k);
                if self.sprite_slot_view(k).delay_main() == 0 {
                    let x_velocity = self.sprite_slot_view(k).x_velocity().wrapping_shl(1);
                    let y_velocity = self.sprite_slot_view(k).y_velocity().wrapping_shl(1);
                    self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
                    self.sprite_slot_view_mut(k).set_y_velocity(y_velocity);
                    self.sprite_slot_view_mut(k).set_delay_main(6);
                }
                if self.sprite_slot_view_mut(k).increment_e() == 0 {
                    self.sprite_slot_view_mut(k).set_state(0);
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
        let mut oam = self.oam_state_view().current_pointer_usize();
        for i in (0..=5usize).rev() {
            let ex = x.wrapping_add(MASTER_SWORD_DRAW_X_OFFSETS[i] as i16 as u16);
            let ey = y.wrapping_add(MASTER_SWORD_DRAW_Y_OFFSETS[i] as i16 as u16);
            self.set_oam_helper0_at(oam, ex, ey, MASTER_SWORD_DRAW_CHARS[i], flags, 0);
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
        if self.sprite_slot_view(k).ai_state() != 3 {
            let c = self.flute_boy_draw_for_world(k);
            self.sprite_slot_view_mut(k).set_c(c);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot_view(k).c() == 0 && self.sprite_slot_view(k).b() == 0 {
            self.system_signals_view_mut().set_ambient_sound_effect(11);
            self.sprite_slot_view_mut(k).set_b(11);
        }
        let graphics = (self.frame_state().frame_counter >> 5) & 1;
        self.sprite_slot_view_mut(k).set_graphics(graphics);
        match self.sprite_slot_view(k).ai_state() {
            0 => {
                let close = self.flute_boy_check_if_player_close(k);
                if self.inventory_state_view().flute() >= 2 || close {
                    self.sprite_slot_view_mut(k).set_ai_state(1);
                    self.sprite_slot_view_mut(k).increment_direction();
                    self.garnish_state_view_mut()
                        .increment_haunted_grove_flute_event_latch();
                    self.sprite_slot_view_mut(k).set_delay_main(176);
                    self.player_state_view_mut().immobilize();
                }
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.sprite_slot_view_mut(k).set_delay_main(25);
                    self.flute_kid_spawn_quaver(k);
                }
            }
            1 => {
                self.player_state_view_mut().immobilize();
                if self.sprite_slot_view(k).delay_main() == 0 {
                    self.set_sub_screen_layers(2);
                    self.palette_filter_view_mut().set_color_math_control(48);
                    self.palette_filter_view_mut().set_countdown(0);
                    self.palette_filter_view_mut()
                        .set_darkening_or_lightening_screen(0);
                    self.palette_assert_translucency_swap();
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                    self.system_signals_view_mut().set_ambient_sound_effect(128);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x33);
                }
            }
            2 => {
                if (self.frame_state().frame_counter & 15) == 0 {
                    self.palette_filter_sp5f_for_world();
                    if self.palette_filter_view().countdown() == 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(3);
                    }
                }
            }
            3 => {
                self.palette_filter_restore_sp5f_for_world();
                self.palette_revert_translucency_swap();
                self.sprite_slot_view_mut(k).set_state(0);
                self.player_state_view_mut().clear_immobilized();
            }
            _ => {}
        }
    }

    // void Sprite_FluteKid_Stumpy(int k) {  // 86b040
    //   See sprite_main.c:9881..9954 for the conversation / shovel sequence.
    pub(super) fn sprite_flute_kid_stumpy(&mut self, k: usize) {
        const FLUTE_AARDVARK_GRAPHICS_SEQUENCE: [i8; 20] =
            [1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2, 3, 2, 3, 2, 3, 2, 3, 2, -1];
        const FLUTE_AARDVARK_FRAME_DELAYS: [i8; 19] = [
            -1, -1, -1, 16, 2, 12, 6, 8, 10, 4, 14, 2, 10, 6, 6, 10, 2, 14, 2,
        ];
        self.flute_aardvark_draw_for_world(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot_view(k).ai_state() {
            0 => match self.inventory_state_view().flute() & 3 {
                0 => {
                    // supplicate
                    if (self.sprite_show_solicited_message_for_world(k, 0xe5) & 0x100) != 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(1);
                    }
                }
                1 => {
                    // give me flute
                    self.sprite_show_solicited_message_for_world(k, 0xe8);
                }
                2 => {
                    // thanks
                    self.sprite_slot_view_mut(k).set_graphics(1);
                    if (self.sprite_show_solicited_message_for_world(k, 0xe9) & 0x100) != 0 {
                        self.sprite_slot_view_mut(k).set_ai_state(3);
                    }
                }
                3 => {
                    // already did
                    self.sprite_slot_view_mut(k).set_graphics(3);
                }
                _ => {}
            },
            1 => {
                if self.multiselect_choice_view().value() == 0 {
                    self.sprite_show_message_unconditional(0xe6);
                    self.sprite_slot_view_mut(k).set_ai_state(2);
                } else {
                    self.sprite_show_message_unconditional(0xe7);
                    self.sprite_slot_view_mut(k).set_ai_state(0);
                }
            }
            2 => {
                // grant shovel
                self.player_state_view_mut().set_item_receipt_method(0);
                self.link_receive_item(0x13, 0);
                self.sprite_slot_view_mut(k).set_ai_state(0);
            }
            3 => {
                // wait for music
                if self.save_progress_view().hud_current_item() == HUD_ITEM_FLUTE
                    && (self.player_state_view().joypad1h_last() & 0x40) != 0
                {
                    self.sprite_slot_view_mut(k).set_ai_state(4);
                    self.system_signals_view_mut().set_music_control(0xf2);
                    self.system_signals_view_mut().set_sound_effect_1(0);
                    self.system_signals_view_mut().set_ambient_sound_effect(23);
                    self.player_state_view_mut().increment_immobilized_flag();
                }
            }
            4 => {
                if self.sprite_slot_view(k).delay_main() == 0 {
                    if self.sprite_slot_view(k).a() >= 3 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x33);
                    }
                    let idx = self.sprite_slot_view(k).a() as usize;
                    self.sprite_slot_view_mut(k).increment_a();
                    let gfx = FLUTE_AARDVARK_GRAPHICS_SEQUENCE[idx];
                    if gfx >= 0 {
                        self.sprite_slot_view_mut(k).set_graphics(gfx as u8);
                        self.sprite_slot_view_mut(k)
                            .set_delay_main(FLUTE_AARDVARK_FRAME_DELAYS[idx] as u8);
                    } else {
                        self.system_signals_view_mut().set_music_control(0xf3);
                        self.sprite_slot_view_mut(k).set_ai_state(5);
                        self.player_state_view_mut().clear_immobilized();
                    }
                }
            }
            5 => {
                // done
                self.sprite_slot_view_mut(k).set_graphics(3);
                self.save_progress_view_mut().or_progress_indicator_3(8);
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
        if self.sprite_slot_view(k).delay_main() == 0 {
            self.sprite_slot_view_mut(k).set_state(0);
        }
        if (self.frame_state().frame_counter & 1) == 0 {
            let cur = self.sprite_system_view().cur_object_index();
            let bit = ((self.frame_state().frame_counter >> 5) ^ cur) & 1;
            let delta: u8 = if bit != 0 { 0xff } else { 1 };
            self.sprite_slot_view_mut(k).add_x_velocity(delta);
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
            self.sprite_slot_view_mut(j).set_head_direction(1);
            self.sprite_slot_view_mut(j).set_z_velocity(8);
            self.sprite_slot_view_mut(j).set_delay_main(96);
            self.sprite_slot_view_mut(j).set_ignore_projectile(96);
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
        const LOCAL_X_VELOCITIES: [i8; 8] = [0, 0, -16, 16, -16, 16, 16, -16];
        const LOCAL_Y_VELOCITIES: [i8; 8] = [-16, 16, 0, 0, -16, 16, -16, 16];
        self.somaria_platform_handle_junctions(k);
        let j = self.sprite_slot_view(k).direction() as usize;
        self.sprite_slot_view_mut(k)
            .set_x_velocity(LOCAL_X_VELOCITIES[j] as u8);
        self.sprite_slot_view_mut(k)
            .set_y_velocity(LOCAL_Y_VELOCITIES[j] as u8);
    }

    // void Pipe_HandlePlayerMovement(uint8 dir) {  // 9efcff
    //   link_direction_last = link_direction = dir;
    //   Link_HandleVelocity();
    //   Link_HandleMovingAnimation_FullLongEntry();
    //   HandleIndoorCameraAndDoors();
    // }
    pub(super) fn pipe_handle_player_movement(&mut self, dir: u8) {
        self.player_state_view_mut()
            .set_direction_and_last_direction(dir);
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
        let graphics = (self.frame_state().frame_counter >> 3) & 1;
        self.sprite_slot_view_mut(k).set_graphics(graphics);
        if self.world_location_state().is_indoors() && self.sprite_slot_view(k).delay_aux1() == 0 {
            if (self.sprite_check_tile_collision(k) & 3) != 0 {
                let x_velocity = 0u8.wrapping_sub(self.sprite_slot_view(k).x_velocity());
                let direction = 0u8.wrapping_sub(self.sprite_slot_view(k).direction());
                self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
                self.sprite_slot_view_mut(k).set_direction(direction);
                self.sprite_slot_view_mut(k).set_delay_aux1(32);
            }
            if (self.sprite_slot_view(k).wall_collision() & 12) != 0 {
                let y_velocity = 0u8.wrapping_sub(self.sprite_slot_view(k).y_velocity());
                let a = 0u8.wrapping_sub(self.sprite_slot_view(k).a());
                self.sprite_slot_view_mut(k).set_y_velocity(y_velocity);
                self.sprite_slot_view_mut(k).set_a(a);
                self.sprite_slot_view_mut(k).set_delay_aux1(32);
            }
        }
        if self.sprite_slot_view(k).x_velocity() != 0 {
            if (self.sprite_slot_view(k).x_velocity() as i8).is_negative() {
                self.sprite_slot_view_mut(k).and_oam_flags(!0x40);
            } else {
                self.sprite_slot_view_mut(k).or_oam_flags(0x40);
            }
        }
        self.sprite_move_xy(k);
        if (self.frame_state().frame_counter & 63) == 0 {
            let x = (self.player_state_view().x() & !0xff)
                .wrapping_add(u16::from(self.get_random_number()));
            let y = (self.player_state_view().y() & !0xff)
                .wrapping_add(u16::from(self.get_random_number()));
            let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
            self.sprite_slot_view_mut(k).set_a(pt.y);
            self.sprite_slot_view_mut(k).set_direction(pt.x);
        }
        if (self.frame_state().frame_counter & 15) == 0 {
            let y_velocity = (((self.sprite_slot_view(k).a() as i8)
                + (self.sprite_slot_view(k).y_velocity() as i8))
                >> 1) as u8;
            let x_velocity = (((self.sprite_slot_view(k).direction() as i8)
                + (self.sprite_slot_view(k).x_velocity() as i8))
                >> 1) as u8;
            self.sprite_slot_view_mut(k).set_y_velocity(y_velocity);
            self.sprite_slot_view_mut(k).set_x_velocity(x_velocity);
        }
        self.sprite_move_z(k);
        let z_velocity_delta = if (self.get_random_number() & 1) != 0 {
            -1i8
        } else {
            1i8
        } as u8;
        self.sprite_slot_view_mut(k)
            .add_z_velocity(z_velocity_delta);
        if self.sprite_slot_view(k).z() < 8 {
            self.sprite_slot_view_mut(k).set_z(8);
            self.sprite_slot_view_mut(k).set_z_velocity(5);
        } else if self.sprite_slot_view(k).z() >= 24 {
            self.sprite_slot_view_mut(k).set_z(24);
            self.sprite_slot_view_mut(k).set_z_velocity((-5i8) as u8);
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
const HUD_ITEM_FLUTE: u8 = 13;

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state
            .oam_state_view_mut()
            .set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_view_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        state
    }

    #[test]
    fn somaria_platform_handle_drag_x_writes_drag_when_dirs_oppose() {
        let mut s = fresh_state();
        // sprite_D[k] = 0, sprite_head_dir[k] = 2 → XOR&2 == 2 (truthy).
        s.sprite_slot_view_mut(3).set_direction(0);
        s.sprite_slot_view_mut(3).set_head_direction(2);
        // sprite_x_lo[k] = 0 — x will become 4, t = 4.
        s.sprite_slot_view_mut(3).set_x_low(0);
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        s.somaria_platform_handle_drag_x(3);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 4);
        assert_eq!(s.sprite_slot_view(3).x_low(), 4);
    }

    #[test]
    fn somaria_platform_handle_drag_returns_when_aligned() {
        let mut s = fresh_state();
        // Bit 2 of XOR is zero → no drag.
        s.sprite_slot_view_mut(1).set_direction(0);
        s.sprite_slot_view_mut(1).set_head_direction(1);
        s.sprite_slot_view_mut(1).set_x_low(0);
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        s.somaria_platform_handle_drag_x(1);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 0);
        assert_eq!(s.sprite_slot_view(1).x_low(), 0);
    }

    #[test]
    fn somaria_platform_handle_junctions_b2_xors_d_with_3() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(5).set_e(0xb2);
        s.sprite_slot_view_mut(5).set_direction(1);
        s.somaria_platform_handle_junctions(5);
        assert_eq!(s.sprite_slot_view(5).direction(), 1 ^ 3);
    }

    #[test]
    fn somaria_platform_handle_junctions_b6_clears_ai_state_when_correct_key_pressed() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(0).set_e(0xb6);
        s.sprite_slot_view_mut(0).set_direction(0);
        s.player_state_view_mut().clear_auxiliary_state();
        // kSomariaPlatform_TransitDir[0] = 4; press that bit.
        s.player_state_view_mut().set_joypad1h_last(4);
        s.somaria_platform_handle_junctions(0);
        assert_eq!(s.sprite_slot_view(0).ai_state(), 0);
        assert_eq!(s.sprite_slot_view(0).direction(), 0 ^ 1);
        assert_eq!(s.player_state_view().on_somaria_platform(), 1);
    }

    #[test]
    fn master_sword_main_clears_state_when_event_bit_set() {
        let mut s = fresh_state();
        s.ram[MAIN_MODULE_INDEX] = 9; // not 26
        s.world_state_view_mut().set_overworld_screen(0x02);
        s.overworld_event_info_view_mut().set_event_info(0x02, 0x40);
        s.sprite_slot_view_mut(4).set_state(9);
        s.master_sword_main(4);
        assert_eq!(s.sprite_slot_view(4).state(), 0);
    }

    #[test]
    fn master_sword_spawn_pendant_prop_sets_velocities_from_ain() {
        let mut s = fresh_state();
        s.player_state_view_mut().set_x(100);
        s.player_state_view_mut().set_y(50);
        // Canonical Sprite_SpawnDynamically walks j_in=15 down; the highest
        // free slot in 0..=15 wins.
        for j in 0..=15 {
            s.sprite_slot_view_mut(j).set_state(0);
        }
        s.master_sword_spawn_pendant_prop(0, 9);
        // ain=9 → (9>>1)&3 = 4 & 3 = 0 → xv = -4, yv = -2.
        // The spawn lands in slot 15 (highest free under canonical helper).
        assert_eq!(s.sprite_slot_view(15).graphics(), 4);
        assert_eq!(s.sprite_slot_view(15).subtype2(), 3);
        assert_eq!(s.sprite_slot_view(15).flags2(), 64);
        assert_eq!(s.sprite_slot_view(15).delay_main(), 228);
        assert_eq!(s.sprite_slot_view(15).oam_flags(), 9);
        assert_eq!(s.sprite_slot_view(15).x_velocity() as i8, -4);
        assert_eq!(s.sprite_slot_view(15).y_velocity() as i8, -2);
    }

    #[test]
    fn master_sword_prop_state_2_doubles_velocity_when_delay_zero() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(6).set_ai_state(2);
        s.sprite_slot_view_mut(6).set_delay_main(0);
        s.sprite_slot_view_mut(6).set_x_velocity(3);
        s.sprite_slot_view_mut(6).set_y_velocity(5);
        s.sprite_slot_view_mut(6).set_e(7);
        s.sprite_master_sword_prop(6);
        assert_eq!(s.sprite_slot_view(6).x_velocity(), 6);
        assert_eq!(s.sprite_slot_view(6).y_velocity(), 10);
        assert_eq!(s.sprite_slot_view(6).delay_main(), 6);
        assert_eq!(s.sprite_slot_view(6).e(), 8);
    }

    #[test]
    fn flute_boy_check_if_player_close_true_when_within_48() {
        let mut s = fresh_state();
        // Place sprite at (100, 100) → yy = 84. link at (130, 130).
        s.sprite_slot_view_mut(2).set_x_low(100);
        s.sprite_slot_view_mut(2).set_y_low(100);
        s.sprite_slot_view_mut(2).set_x_high(0);
        s.sprite_slot_view_mut(2).set_y_high(0);
        s.player_state_view_mut().set_x(130);
        s.player_state_view_mut().set_y(130);
        assert!(s.flute_boy_check_if_player_close(2));
    }

    #[test]
    fn flute_boy_check_if_player_close_false_when_distant() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(2).set_x_low(0);
        s.sprite_slot_view_mut(2).set_y_low(0);
        s.player_state_view_mut().set_x(200);
        s.player_state_view_mut().set_y(200);
        assert!(!s.flute_boy_check_if_player_close(2));
    }

    #[test]
    fn flute_kid_spawn_quaver_initializes_z_vel_and_delay() {
        let mut s = fresh_state();
        // Canonical Sprite_SpawnDynamically uses j_in=15 and reads coords
        // from sprite_x_lo[k]/sprite_x_hi[k] (Sprite_GetX), so seed slot 0.
        s.sprite_slot_view_mut(0).set_x_low(0x00);
        s.sprite_slot_view_mut(0).set_x_high(0x01); // = 0x100
        s.sprite_slot_view_mut(0).set_y_low(0x80);
        s.sprite_slot_view_mut(0).set_y_high(0x00); // = 0x080
        for j in 0..=15 {
            s.sprite_slot_view_mut(j).set_state(0);
        }
        s.flute_kid_spawn_quaver(0);
        // Canonical helper picks slot 15 (highest free in 0..=15).
        assert_eq!(s.sprite_slot_view(15).head_direction(), 1);
        assert_eq!(s.sprite_slot_view(15).z_velocity(), 8);
        assert_eq!(s.sprite_slot_view(15).delay_main(), 96);
        assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 96);
        assert_eq!(s.sprite_slot_view(15).sprite_type(), 0x2e);
    }

    #[test]
    fn sprite_flute_kid_quaver_zeros_state_when_delay_zero() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(4).set_state(9);
        s.sprite_slot_view_mut(4).set_delay_main(0);
        s.set_frame_counter(0); // even → adjust x_vel
        s.sprite_slot_view_mut(4).set_x_velocity(10);
        s.ram[CUR_OBJECT_INDEX] = 0;
        s.sprite_flute_kid_quaver(4);
        // Sprite_ReturnIfInactive is false (state != 0 and other guards), so
        // the body executes. Delay==0 → state cleared.
        assert_eq!(s.sprite_slot_view(4).state(), 0);
    }

    #[test]
    fn sprite_flute_kid_stumpy_starts_music_when_flute_equipped_and_y_pressed() {
        let mut s = fresh_state();
        let k = 3;

        s.sprite_slot_view_mut(k).set_state(9);
        s.sprite_slot_view_mut(k).set_ai_state(3);
        s.ram[HUD_CUR_ITEM] = HUD_ITEM_FLUTE;
        s.player_state_view_mut().set_joypad1h_last(0x40);

        s.sprite_flute_kid_stumpy(k);

        assert_eq!(s.sprite_slot_view(k).ai_state(), 4);
        assert_eq!(s.system_signals_view().music_control(), 0xf2);
        assert_eq!(s.system_signals_view().sound_effect_1(), 0);
        assert_eq!(s.system_signals_view().ambient_sound_effect(), 23);
        assert_eq!(s.player_state_view().immobilized_flag(), 1);
    }

    #[test]
    fn somaria_platform_drag_link_subtracts_when_link_north_west_of_sprite() {
        let mut s = fresh_state();
        // cur_sprite=(100,200), link=(20,80). x = 92-20 = 72 > 0 (no high
        // bit) → +1. y = 184-80 = 104 > 0 → +1.
        s.sprite_workspace_view_mut().set_current_sprite_x(100);
        s.sprite_workspace_view_mut().set_current_sprite_y(200);
        s.player_state_view_mut().set_x(20);
        s.player_state_view_mut().set_y(80);
        write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
        write_le_u16(&mut s.ram, DRAG_PLAYER_Y, 0);
        s.somaria_platform_drag_link(0);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 1);
        assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_Y), 1);
    }

    #[test]
    fn somaria_platform_handle_junctions_bc_no_input_sets_ai_state_1() {
        let mut s = fresh_state();
        s.sprite_slot_view_mut(2).set_e(0xbc);
        s.sprite_slot_view_mut(2).set_direction(0);
        s.player_state_view_mut().set_joypad1h_last(0);
        s.somaria_platform_handle_junctions(2);
        // KEYS6[0] = 0xc; t = 0; ai_state stays 1, player_on flag set.
        assert_eq!(s.sprite_slot_view(2).ai_state(), 1);
        assert_eq!(s.player_state_view().on_somaria_platform(), 1);
    }

    #[test]
    fn pipe_handle_player_movement_sets_direction_and_runs_player_motion_tail() {
        let mut s = fresh_state();
        s.world_state_view_mut().set_indoor_flag(0);
        s.player_state_view_mut().set_speed_setting(0);
        s.player_state_view_mut().set_facing(0);
        s.player_state_view_mut()
            .set_direction_and_last_direction(0);
        s.player_state_view_mut().set_x(0x0120);
        s.player_state_view_mut().set_y(0x0230);

        s.pipe_handle_player_movement(2);

        assert_eq!(s.player_state_view().direction(), 2);
        assert_eq!(s.player_state_view().last_direction(), 2);
        // Link_HandleMovingAnimation_FullLongEntry maps left/right movement
        // onto facing 4/6 when direction changes are allowed.
        assert_eq!(s.player_state_view().facing(), 4);
        assert_eq!(s.player_state_view().x(), 0x011e);
        assert_eq!(s.player_state_view().y(), 0x0230);
    }

    #[test]
    fn faerie_handle_movement_animates_moves_and_clamps_low_z() {
        let mut s = fresh_state();
        let k = 3;
        s.set_frame_counter(8);
        s.sprite_set_x(k, 0x0100);
        s.sprite_set_y(k, 0x0200);
        s.sprite_slot_view_mut(k).set_x_velocity(16);
        s.sprite_slot_view_mut(k).set_y_velocity(0);
        s.sprite_slot_view_mut(k).set_z(7);
        s.sprite_slot_view_mut(k).set_z_velocity(0);
        s.sprite_slot_view_mut(k).set_oam_flags(0);

        s.faerie_handle_movement(k);

        assert_eq!(s.sprite_slot_view(k).graphics(), 1);
        assert_eq!(s.sprite_slot_view(k).oam_flags() & 0x40, 0x40);
        assert_eq!(s.sprite_get_x(k), 0x0101);
        assert_eq!(s.sprite_get_y(k), 0x0200);
        assert_eq!(s.sprite_slot_view(k).z(), 8);
        assert_eq!(s.sprite_slot_view(k).z_velocity(), 5);
    }

    #[test]
    fn faerie_handle_movement_averages_velocities_on_16_frame_tick() {
        let mut s = fresh_state();
        let k = 4;
        s.set_frame_counter(16);
        s.sprite_set_x(k, 0x0120);
        s.sprite_set_y(k, 0x0230);
        s.sprite_slot_view_mut(k).set_x_velocity(10);
        s.sprite_slot_view_mut(k).set_y_velocity((-10i8) as u8);
        s.sprite_slot_view_mut(k).set_direction(20);
        s.sprite_slot_view_mut(k).set_a((-20i8) as u8);
        s.sprite_slot_view_mut(k).set_z(12);
        s.sprite_slot_view_mut(k).set_z_velocity(0);

        s.faerie_handle_movement(k);

        assert_eq!(s.sprite_slot_view(k).graphics(), 0);
        assert_eq!(s.sprite_slot_view(k).x_velocity(), 15);
        assert_eq!(s.sprite_slot_view(k).y_velocity(), (-15i8) as u8);
        assert_eq!(s.sprite_get_x(k), 0x0120);
        assert_eq!(s.sprite_get_y(k), 0x022f);
        assert_eq!(s.sprite_slot_view(k).z(), 12);
    }
}
