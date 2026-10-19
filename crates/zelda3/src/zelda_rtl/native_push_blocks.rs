//! Publication for native block drawing, motion, collision and camera work.
use super::*;
use crate::game_state::constants::{BIG_KEY_DOOR_MESSAGE_TRIGGERED, DIALOGUE_MESSAGE_INDEX, TILEDETECT_COLLISION_BITS};

impl ZeldaState {
    /// Camera stores have retired on the same bus as the dungeon caller.
    pub(super) fn native_dungeon_camera_store(&mut self, offset: usize, value: u16, width: u8) {
        assert!(matches!(width, 1 | 2));
        for byte in 0..usize::from(width) {
            let address = offset + byte;
            match address {
                0xb9a => { assert_eq!(value,0); self.archery_game_mut().clear_out_of_arrows(); },
                0..=3 => {
                    self.game_state
                        .sprites
                        .workspace
                        .write_zero_page_byte(address, (value >> (byte * 8)) as u8);

                }
                0xe..=0xf => self
                    .game_state
                    .player
                    .tile_detection
                    .set_collision_bits(read_le_u16(&self.ram, TILEDETECT_COLLISION_BITS)),
                4..=13 | 0x1c0..=0x1ff => {}
                0x27..=0x28 | 0x2c6 => {assert!(self.game_state.player.follower_link.write_push_block_byte(address,(value>>(byte*8))as u8));}
                0x22..=0x23 => {assert!(self.game_state.player.follower_link.write_sprite_player_byte(address,(value>>(byte*8))as u8));}
                0x20..=0x21 => {
                    let value = read_le_u16(&self.ram, LINK_Y_COORD);
                    assert!(self
                        .game_state
                        .player
                        .follower_link
                        .write_sprite_player_byte(address, (value >> ((address & 1) * 8)) as u8));
                }
                0x333 | 0x4f0..=0x4ff => {
                    assert!(self.game_state.dungeon.write_room_parent_byte(address,(value>>(byte*8)) as u8));
                }
                0x4b8..=0x4b9 => self.game_state.world.transient.set_big_key_door_message_triggered(read_le_u16(&self.ram,BIG_KEY_DOOR_MESSAGE_TRIGGERED)),
                0x4c7 => {
                    assert!(self
                        .game_state
                        .dungeon
                        .write_header_byte(address, (value >> (byte * 8)) as u8));
                }
                0x11e..=0x125 => {
                    let base = address & !1;
                    let word = read_le_u16(&self.ram, base);
                    let scroll = &mut self.game_state.display.ppu_scroll_copy;
                    match base {
                        0x11e => scroll.set_bg2_h_copy(word),
                        0x120 => scroll.set_bg1_h_copy(word),
                        0x122 => scroll.set_bg2_v_copy(word),
                        0x124 => scroll.set_bg1_v_copy(word),
                        _ => unreachable!(),
                    }
                }
                0xe0..=0xe3 | 0xe6..=0xe9 | 0x618..=0x623 => {
                    let base = address & !1;
                    let word = read_le_u16(&self.ram, base);
                    let scroll = &mut self.game_state.display.ppu_scroll_copy;
                    match base {
                        0xe0 => scroll.set_bg1_h_copy2(word),
                        0xe2 => scroll.set_bg2_h_copy2(word),
                        0xe6 => scroll.set_bg1_v_copy2(word),
                        0xe8 => scroll.set_bg2_v_copy2(word),
                        0x620 => scroll.set_bg1_h_subpixel(word),
                        0x622 => scroll.set_bg1_v_subpixel(word),
                        _ => {
                            let camera = &mut self.game_state.world.camera_boundaries;
                            match base {
                                0x618 => camera.set_camera_y_coord_scroll_low(word),
                                0x61a => camera.set_camera_y_coord_scroll_hi(word),
                                0x61c => camera.set_camera_x_coord_scroll_low(word),
                                0x61e => camera.set_camera_x_coord_scroll_hi(word),
                                _ => unreachable!(),
                            }
                        }
                    }
                }
                _ => panic!("unowned native dungeon camera byte {address:05x}"),
            }
        }
    }

    /// The bus has retired the actual WRAM store. Publish its bytes without
    /// importing a snapshot or projecting inactive scratch aliases over it.
    pub(super) fn native_push_block_store(&mut self, offset: usize, value: u16, width: u8) {
        assert!(matches!(width, 1 | 2));
        for byte in 0..usize::from(width) {
            let address = offset + byte;
            let value = (value >> (byte * 8)) as u8;
            if self.game_state.oam.write_drawing_byte(address, value) {
                continue;
            }
            if self
                .game_state
                .player
                .pushed_block
                .write_instruction_byte(address, value)
                || self
                    .game_state
                    .player
                    .follower_link
                    .write_push_block_byte(address, value)
                || self
                    .game_state
                    .dungeon
                    .write_room_parent_byte(address, value)
                || self.game_state.dungeon.write_header_byte(address, value)
                || self.game_state.dungeon.write_floor_byte(address, value)
                || self.game_state.dungeon.write_attribute_byte(address, value)
                || self.game_state.dungeon.write_entrance_byte(address, value)
            {
                continue;
            }
            match address {
                // Garnish slot views use these retired bytes as their sole storage.
                0x1f800..=0x1f81d | 0x1f90e..=0x1f92b => {},
                0x10 => self.set_main_module(value),
                0x11 => self.set_submodule(value),
                0x29e..=0x2a7 => self.ancilla_slot_view_mut(address-0x29e).set_z(value),
                0x39f..=0x3a8 => self.ancilla_slot_view_mut(address-0x39f).set_work_byte_3(value),
                0x3a9..=0x3ad => self.ancilla_slot_view_mut(address-0x3a4).set_work_byte_1(value),
                0xbf0..=0xbf9 => self.ancilla_slot_view_mut(address-0xbf0).set_work_byte_4(value),
                0xbfa..=0xc03 => self.ancilla_slot_view_mut(address-0xbfa).set_y_low(value),
                0xc04..=0xc0d => self.ancilla_slot_view_mut(address-0xc04).set_x_low(value),
                0xc0e..=0xc17 => self.ancilla_slot_view_mut(address-0xc0e).set_y_high(value),
                0xc18..=0xc21 => self.ancilla_slot_view_mut(address-0xc18).set_x_high(value),
                0xc22..=0xc2b => self.ancilla_slot_view_mut(address-0xc22).set_y_velocity(value),
                0xc2c..=0xc35 => self.ancilla_slot_view_mut(address-0xc2c).set_x_velocity(value),
                0xc36..=0xc3f => self.ancilla_slot_view_mut(address-0xc36).set_y_subpixel(value),
                0xc40..=0xc49 => self.ancilla_slot_view_mut(address-0xc40).set_x_subpixel(value),
                0x3b1..=0x3ba => self.ancilla_slot_view_mut(address-0x3b1).set_aux_timer(value),
                0xfc1 => self.set_modal_pause_flag(value),
                0xf3cc => self.follower_state_mut().set_indicator(value),
                0xc68..=0xc71 => self.ancilla_slot_view_mut(address-0xc68).set_timer(value),
                0xc86..=0xc8f => self.ancilla_slot_view_mut(address-0xc86).set_oam_index(value),
                0xc7c..=0xc80 => self.ancilla_slot_view_mut(address-0xc7c).set_floor(value),
                0x2cd..=0x2ce => self.game_state.messaging.shared_message_timer.timer = read_le_u16(&self.ram,SHARED_MESSAGE_TIMER),
                0x1cf0..=0x1cf1 => {let word=read_le_u16(&self.ram,DIALOGUE_MESSAGE_INDEX);self.dialogue_message_index_mut().set_value(word);},
                0xff9 => self.attract_scene_mut().set_intro_palette_flash_count(value),
                0x14 => self.set_bg_vram_load_mode(value),
                0..=3 => self
                    .game_state
                    .sprites
                    .workspace
                    .write_zero_page_byte(address, value),
                0xe..=0xf => self.game_state.player.tile_detection.set_collision_bits(read_le_u16(&self.ram,TILEDETECT_COLLISION_BITS)),
                4..=0xd | 0x51 | 0x72..=0x75 | 0x1c0..=0x1ff | 0x5fe..=0x5ff => {}
                0x560..=0x5df => {
                    let base = address & !1;
                    self.game_state
                        .dungeon
                        .write_saved_object_word(base, read_le_u16(&self.ram, base));
                }
                0x600..=0x60f => {
                    let bounds = &mut self.game_state.world.room_bounds;
                    let index = (address - 0x600) / 2;
                    let old = if index < 4 {
                        bounds.y_bound(index)
                    } else {
                        bounds.x_bound(index - 4)
                    };
                    let shift = (address & 1) * 8;
                    let word = (old & !(255 << shift)) | (u16::from(value) << shift);
                    if index < 4 {
                        bounds.set_y_bound(index, word);
                    } else {
                        bounds.set_x_bound(index - 4, word);
                    }
                }
                0x1000..=0x17ff => self.write_vram_upload_absolute_byte(address, value),
                0x6000..=0x8fff => {} // Adjacent graphics scratch is RAM-backed.
                0xf000..=0xf27f => {
                    let room = (address - 0xf000) / 2;
                    let progress = &mut self.game_state.inventory.save_progress;
                    let old = progress.dungeon_info_word(room);
                    let shift = (address & 1) * 8;
                    progress.set_dungeon_info_word(
                        room,
                        (old & !(255 << shift)) | (u16::from(value) << shift),
                    );
                }
                0x12e => {} // ItemBus publishes the live SFX latch.
                _ => assert!(
                    self.native_sprite_main_store(address as u32, u16::from(value), 1),
                    "unowned native push-block byte {address:05x}"
                ),
            }
        }
    }
}
