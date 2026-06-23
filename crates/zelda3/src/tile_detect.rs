// Methods ported from zelda3/src/tile_detect.c and included inside ZeldaState.

use super::*;

const TILE_DETECT_CARDINAL_AXIS_OFFSETS: [u8; 4] = [8, 24, 0, 15];
const TILE_DETECT_CARDINAL_LOW_SIDE_OFFSETS: [u8; 4] = [0, 0, 8, 8];
const TILE_DETECT_CARDINAL_CENTER_OFFSETS: [u8; 4] = [8, 8, 16, 16];
const TILE_DETECT_CARDINAL_HIGH_SIDE_OFFSETS: [u8; 4] = [15, 15, 23, 23];
const TILE_DETECT_SLOPE_AXIS_OFFSETS: [i8; 4] = [7, 24, -1, 16];
const TILE_DETECT_SLOPE_LOW_SIDE_OFFSETS: [u8; 4] = [0, 0, 8, 8];
const TILE_DETECT_SLOPE_HIGH_SIDE_OFFSETS: [u8; 4] = [15, 15, 23, 23];

impl ZeldaState {
    pub fn overworld_get_tile_attribute_at_location(&self, x: u16, y: u16) -> u8 {
        let world = &self.game_state.world.scroll;
        let pos = ((y.wrapping_sub(world.overworld_offset_base_y())
            & world.overworld_offset_mask_y())
            << 3)
            | (x.wrapping_sub(world.overworld_offset_base_x()) & world.overworld_offset_mask_x());
        let map16 = self
            .game_state
            .dungeon
            .room_tilemaps
            .bg2_tile_by_byte_pos(pos);
        let map8_index = (map16 as usize) * 4 + (((y & 8) >> 2) | (x & 1)) as usize;
        let map8 = self.asset_u16(70, map8_index);
        let mut attr = self.asset_u8(163, (map8 & 0x01ff) as usize);
        if (0x10..0x1c).contains(&attr) {
            attr |= ((map8 >> 14) & 1) as u8;
        }
        if env::var("ZELDA3_REPLAY_TRACE_TILE").is_ok()
            && self.replay_trace_filter_matches_current_frame()
        {
            let world = &self.game_state.world.scroll;
            eprintln!(
                "tile-probe frame={} x=0x{:04x} y=0x{:04x} pos=0x{:04x} map16=0x{:04x} map8=0x{:04x} attr=0x{:02x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x}",
                self.game_state.frame.frame_counter,
                x,
                y,
                pos,
                map16,
                map8,
                attr,
                world.overworld_offset_base_x(),
                world.overworld_offset_base_y(),
                world.overworld_offset_mask_x(),
                world.overworld_offset_mask_y(),
            );
        }
        attr
    }

    pub(super) fn tile_detect_movement_y(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let direction = direction as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let detect_y = link_y.wrapping_add(TILE_DETECT_CARDINAL_AXIS_OFFSETS[direction] as u16);
        self.tile_detect_position_mut().set_y(detect_y);
        let y = detect_y & mask;
        let x0 = (link_x.wrapping_add(TILE_DETECT_CARDINAL_LOW_SIDE_OFFSETS[direction] as u16)
            & mask)
            >> 3;
        let x1 = (link_x.wrapping_add(TILE_DETECT_CARDINAL_CENTER_OFFSETS[direction] as u16)
            & mask)
            >> 3;
        let x2 = (link_x.wrapping_add(TILE_DETECT_CARDINAL_HIGH_SIDE_OFFSETS[direction] as u16)
            & mask)
            >> 3;
        self.tile_detect_position_mut().set_tile_probe_anchor(x2);
        self.tile_detection_execute(x0, y, 1);
        self.tile_detection_execute(x1, y, 2);
        self.tile_detection_execute(x2, y, 4);
    }

    pub(super) fn tile_detect_movement_x(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let direction = direction as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x =
            (link_x.wrapping_add(TILE_DETECT_CARDINAL_AXIS_OFFSETS[direction] as u16) & mask) >> 3;
        let y0 =
            link_y.wrapping_add(TILE_DETECT_CARDINAL_LOW_SIDE_OFFSETS[direction] as u16) & mask;
        let y1_pos = link_y.wrapping_add(TILE_DETECT_CARDINAL_CENTER_OFFSETS[direction] as u16);
        self.tile_detect_position_mut().set_y(y1_pos);
        let y1 = y1_pos & mask;
        let y2_pos = link_y.wrapping_add(TILE_DETECT_CARDINAL_HIGH_SIDE_OFFSETS[direction] as u16);
        self.tile_detect_position_mut().set_x(y2_pos);
        let y2 = y2_pos & mask;
        self.tile_detection_execute(x, y0, 1);
        self.tile_detection_execute(x, y1, 2);
        self.tile_detection_execute(x, y2, 4);
    }

    pub(super) fn tile_detect_movement_vertical_slopes(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let direction = direction as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let y = link_y.wrapping_add(TILE_DETECT_SLOPE_AXIS_OFFSETS[direction] as i16 as u16) & mask;
        let x0 =
            (link_x.wrapping_add(TILE_DETECT_SLOPE_LOW_SIDE_OFFSETS[direction] as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(TILE_DETECT_SLOPE_HIGH_SIDE_OFFSETS[direction] as u16)
            & mask)
            >> 3;
        self.tile_detection_execute(x0, y, 1);
        self.tile_detection_execute(x1, y, 2);
    }

    pub(super) fn tile_detect_movement_horizontal_slopes(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let direction = direction as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x = (link_x.wrapping_add(TILE_DETECT_SLOPE_AXIS_OFFSETS[direction] as i16 as u16)
            & mask)
            >> 3;
        let y0 = link_y.wrapping_add(TILE_DETECT_SLOPE_LOW_SIDE_OFFSETS[direction] as u16) & mask;
        let y1 = link_y.wrapping_add(TILE_DETECT_SLOPE_HIGH_SIDE_OFFSETS[direction] as u16) & mask;
        self.tile_detection_execute(x, y0, 1);
        self.tile_detection_execute(x, y1, 2);
    }

    pub(super) fn player_tile_detect_nearby(&mut self) {
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(TILE_DETECT_CARDINAL_LOW_SIDE_OFFSETS[0] as u16) & mask) >> 3;
        let x1 =
            (link_x.wrapping_add(TILE_DETECT_CARDINAL_HIGH_SIDE_OFFSETS[0] as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(TILE_DETECT_CARDINAL_LOW_SIDE_OFFSETS[2] as u16) & mask;
        let y1 = link_y.wrapping_add(TILE_DETECT_CARDINAL_HIGH_SIDE_OFFSETS[2] as u16) & mask;
        self.tile_detect_position_mut().set_tile_probe_anchor(y0);
        self.tile_detection_execute(x0, y0, 8);
        self.tile_detection_execute(x0, y1, 2);
        self.tile_detection_execute(x1, y0, 4);
        self.tile_detection_execute(x1, y1, 1);
    }

    pub(super) fn hookshot_check_tile_collision(&mut self, k: i32) {
        let k = k as usize;
        let bak0 = self.game_state.world.location.dungeon_room_index();
        let bak1 = self.game_state.player.follower_link.lower_level_state();
        if self.ancilla_slot_view(k).work_byte_1() != 0 {
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                == 0
            {
                self.increment_dungeon_room_index_by(0x10);
            }
            self.follower_link_state_mut()
                .set_lower_level_state(bak1 ^ 1);
        }
        let x = self.ancilla_x(k);
        let y = self.ancilla_y(k);
        let dir = self.ancilla_slot_view(k).direction() as i32;
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();
        if self.game_state.dungeon.room_load.header_collision() == 2 {
            self.follower_link_state_mut().set_lower_level_state(1);
            self.hookshot_check_single_layer_tile_collision(
                x.wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                y.wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_v_copy2())
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
                dir,
            );
            self.follower_link_state_mut().set_lower_level_state(0);
        }
        self.hookshot_check_single_layer_tile_collision(x, y, dir);
        self.follower_link_state_mut().set_lower_level_state(bak1);
        self.set_dungeon_room_index(bak0);
    }

    pub(super) fn hookshot_check_single_layer_tile_collision(&mut self, x: u16, y: u16, dir: i32) {
        const CHECK_X: [u8; 8] = [0, 15, 0, 15, 0, 0, 8, 8];
        const CHECK_Y: [u8; 8] = [0, 0, 7, 7, 0, 15, 0, 15];
        let base = dir as usize * 2;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let y0 = y.wrapping_add(CHECK_Y[base] as u16) & mask;
        let y1 = y.wrapping_add(CHECK_Y[base + 1] as u16) & mask;
        let x0 = (x.wrapping_add(CHECK_X[base] as u16) & mask) >> 3;
        let x1 = (x.wrapping_add(CHECK_X[base + 1] as u16) & mask) >> 3;
        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);
    }

    pub(super) fn handle_nudging_in_a_door(&mut self, speed: i8) {
        let y = if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 2
            != 0
        {
            if (self.game_state.player.follower_link.y() as u8) < 0x80 {
                1
            } else {
                0
            }
        } else if (self.game_state.player.follower_link.x() as u8) < 0x80 {
            3
        } else {
            2
        };
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();
        const DETECT_Y: [i8; 4] = [8, 23, 16, 16];
        const DETECT_X: [i8; 4] = [8, 8, 0, 15];
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(DETECT_X[y] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(DETECT_Y[y] as i16 as u16) & mask;
        self.tile_detection_execute(x0, y0, 1);
        if ((self.game_state.player.tile_detection.collision_bits()
            | self.game_state.player.tile_detection.horizontal_ledge() as u16)
            & 3)
            == 0
            && ((self.game_state.player.tile_detection.vertical_ledge()
                | self.game_state.player.tile_detection.diagonal_ledge_tiles())
                & 0x33)
                == 0
        {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 2
            != 0
        {
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut()
                .set_y(y.wrapping_sub(speed as i16 as u16));
        } else {
            let x = self.game_state.player.follower_link.x();
            self.follower_link_state_mut()
                .set_x(x.wrapping_sub(speed as i16 as u16));
        }
    }

    pub(super) fn tile_check_for_mirror_bonk(&mut self) {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(2) & mask) >> 3;
        let x1 = (link_x.wrapping_add(13) & mask) >> 3;
        let y0 = link_y.wrapping_add(10) & mask;
        let y1 = link_y.wrapping_add(21) & mask;
        self.tile_detect_position_mut().set_tile_probe_anchor(y0);
        self.tile_detection_execute(x0, y0, 8);
        self.tile_detection_execute(x0, y1, 2);
        self.tile_detection_execute(x1, y0, 4);
        self.tile_detection_execute(x1, y1, 1);
    }

    pub(super) fn tile_detect_sword_swing_deep_in_door(&mut self, dw: u8) {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();
        const DOORWAY_DETECT_X: [i8; 4] = [8, 8, -1, 16];
        const DOORWAY_DETECT_Y: [i8; 4] = [-1, 24, 16, 16];
        let o = dw.wrapping_sub(1) as usize * 2;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(DOORWAY_DETECT_X[o] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(DOORWAY_DETECT_X[o + 1] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(DOORWAY_DETECT_Y[o] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(DOORWAY_DETECT_Y[o + 1] as i16 as u16) & mask;
        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);
    }

    pub(super) fn tile_detect_reset_state(&mut self) {
        self.tile_detect_position_mut().clear_slope_collision_bits();
        self.tile_detect_position_mut().clear_collision_bits();
        self.tile_detect_position_mut().clear_diagonal_tile();
        self.tile_detect_position_mut().clear_stair_tile();
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_position_mut().clear_inroom_staircase();
        self.tile_detect_position_mut().clear_block_flags();
        self.tile_detect_position_mut().clear_door_direction_flags();
        self.tile_detect_position_mut().clear_moving_floor_tiles();
        self.tile_detect_position_mut().clear_deepwater();
        self.tile_detect_position_mut().clear_normal_tiles();
        self.tile_detect_position_mut().clear_icy_floor();
        self.tile_detect_position_mut().clear_water_staircase();
        self.tile_detect_position_mut().clear_thick_grass();
        self.tile_detect_position_mut().clear_shallow_water();
        self.tile_detect_position_mut()
            .clear_destruction_aftermath();
        self.tile_detect_position_mut().clear_read_something();
        self.tile_detect_position_mut().clear_vertical_ledge();
        self.tile_detect_position_mut().clear_horizontal_ledge();
        self.tile_detect_position_mut()
            .clear_ledges_down_leftright();
        self.tile_detect_position_mut().clear_diagonal_ledge_tiles();
        self.tile_detect_position_mut().clear_chest();
        self.tile_detect_position_mut().clear_key_lock_gravestones();
        self.tile_detect_position_mut().clear_spike_cactus_tiles();
        self.tile_detect_position_mut()
            .clear_spike_floor_and_triggers();
        self.tile_detect_position_mut().clear_dashable_tiles();
        self.tile_detect_position_mut().clear_misc_tiles();
        self.dungeon_environment_mut()
            .clear_moving_floor_check_flags();
    }

    pub(super) fn tile_detection_execute(&mut self, x: u16, y: u16, bits: u16) {
        let mut offset = 0usize;
        let is_indoors = self.game_state.world.location.is_indoors();
        let trace = env::var("ZELDA3_REPLAY_TRACE_TILE").is_ok()
            && self.replay_trace_filter_matches_current_frame();
        let r14_before = self.game_state.player.tile_detection.collision_bits();
        let r12_before = self.game_state.player.tile_detection.slope_collision_bits();
        let misc_before = self.game_state.player.tile_detection.misc_tiles();
        let normal_before = self.game_state.player.tile_detection.normal_tiles();
        let pit_before = self.game_state.player.tile_detection.pit_tile();
        let diag_before = self.game_state.player.tile_detection.diag_state();
        let below_before = self.game_state.player.follower_link.tile_below();
        let tile = if is_indoors {
            self.follower_link_state_mut().clear_force_move_high_byte();
            offset = ((y & !7) as usize) * 8
                + (x as usize & 63)
                + if self.game_state.player.follower_link.lower_level_state() != 0 {
                    0x1000
                } else {
                    0
                };
            let mut tile = self.game_state.dungeon.bg2_attributes.bg2_attr(offset);
            if self
                .game_state
                .player
                .follower_link
                .cheat_walk_through_walls()
                != 0
            {
                tile = 0;
            }
            self.follower_link_state_mut().set_tile_below(tile);
            tile
        } else {
            self.overworld_get_tile_attribute_at_location(x, y)
        };
        self.tile_detect_execute_inner(tile, offset as u16, bits, is_indoors);
        if trace {
            eprintln!(
                "tile-exec frame={} x=0x{:04x} y=0x{:04x} bits=0x{:04x} indoors={} lower={} offs=0x{:04x} tile=0x{:02x} link=0x{:04x}/0x{:04x} speed=0x{:02x}/0x{:02x} r14=0x{:04x}->0x{:04x} r12=0x{:04x}->0x{:04x} misc=0x{:04x}->0x{:04x} normal=0x{:04x}->0x{:04x} pit=0x{:02x}->0x{:02x} diag=0x{:04x}->0x{:04x} below=0x{:02x}->0x{:02x}",
                self.game_state.frame.frame_counter,
                x,
                y,
                bits,
                self.game_state.world.location.indoor_flag(),
                self.game_state.player.follower_link.lower_level_state(),
                offset,
                tile,
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
                self.game_state.player.follower_link.speed_setting(),
                self.game_state.player.follower_link.speed_modifier(),
                r14_before,
                self.game_state.player.tile_detection.collision_bits(),
                r12_before,
                self.game_state.player.tile_detection.slope_collision_bits(),
                misc_before,
                self.game_state.player.tile_detection.misc_tiles(),
                normal_before,
                self.game_state.player.tile_detection.normal_tiles(),
                pit_before,
                self.game_state.player.tile_detection.pit_tile(),
                diag_before,
                self.game_state.player.tile_detection.diag_state(),
                below_before,
                self.game_state.player.follower_link.tile_below(),
            );
        }
    }

    #[rustfmt::skip]
    pub(super) fn tile_detect_execute_inner(&mut self, mut tile: u8, offs: u16, bits: u16, is_indoors: bool) {
        if self.game_state.player.follower_link.cheat_walk_through_walls() != 0 {
            tile = 0;
        }
        let offset = offs as usize;
        match tile {
            0x00
            | 0x05
            | 0x06
            | 0x07
            | 0x14
            | 0x15
            | 0x16
            | 0x17
            | 0x21
            | 0x23
            | 0x24
            | 0x25
            | 0x38
            | 0x39
            | 0x3a
            | 0x3b
            | 0x3c
            | 0x41
            | 0x45
            | 0x47
            | 0x49
            | 0x5e
            | 0x5f
            | 0x61
            | 0x62
            | 0x64
            | 0x65
            | 0x66
            | 0xa6
            | 0xa7
            | 0xbe
            | 0xbf
            | 0xd0..=0xef => {
                if !is_indoors {
                    let normal = self.game_state.player.tile_detection.normal_tiles() | bits;
                    self.tile_detect_position_mut().set_normal_tiles(normal);
                }
            }
            0x01 | 0x02 | 0x03 | 0x26 | 0x43 => {
                self.tile_detect_position_mut().or_collision_bits(bits);
            }
            0x04 => {
                if is_indoors {
                self.tile_detect_position_mut().or_collision_bits(bits);
                } else {
                    let grass = self.game_state.player.tile_detection.thick_grass() | bits;
                    self.tile_detect_position_mut().set_thick_grass(grass);
                }
            }
            0x0b => {
                if is_indoors {
                self.tile_detect_position_mut().or_collision_bits(bits);
                } else {
                    self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                    let deepwater = self.game_state.player.tile_detection.deepwater() | (bits << 4);
                    self.tile_detect_position_mut().set_deepwater(deepwater);
                }
            }
            0x08 => {
                let deepwater = self.game_state.player.tile_detection.deepwater() | bits;
                self.tile_detect_position_mut().set_deepwater(deepwater);
            }
            0x09 => {
                let shallow = self.game_state.player.tile_detection.shallow_water() | bits;
                self.tile_detect_position_mut().set_shallow_water(shallow);
            }
            0x0a => {
                let normal = self.game_state.player.tile_detection.normal_tiles() | bits;
                self.tile_detect_position_mut().set_normal_tiles(normal);
            }
            0x0c => {
                let moving_floor = self.game_state.player.tile_detection.moving_floor_tiles() | bits;
                self.tile_detect_position_mut().set_moving_floor_tiles(moving_floor);
            }
            0x0d => {
                if !self.game_state.player.follower_link.is_menu_blocked()
                    && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0
                {
                    self.tile_detect_position_mut().or_spike_floor_and_triggers((bits << 4) as u8);
                }
            }
            0x0e => {
                let icy = self.game_state.player.tile_detection.icy_floor() | bits;
                self.tile_detect_position_mut().set_icy_floor(icy);
            }
            0x0f => {
                let icy = self.game_state.player.tile_detection.icy_floor() | (bits << 4);
                self.tile_detect_position_mut().set_icy_floor(icy);
            }
            0x6c..=0x6f if is_indoors => {
                self.tile_detect_position_mut().or_collision_bits(bits);
            }
            0x6c..=0x6f => {
                let normal = self.game_state.player.tile_detection.normal_tiles() | bits;
                self.tile_detect_position_mut().set_normal_tiles(normal);
            }
            0x10..=0x13 => {
                const DIAG_STATE: [u16; 4] = [4, 0, 6, 2];
                self.tile_detect_position_mut().or_slope_collision_bits(bits);
                self.tile_detect_position_mut()
                    .set_diag_state(DIAG_STATE[(tile & 3) as usize]);
            }
            0x18..=0x1b => {
                const DIAG_STATE: [u16; 4] = [4, 0, 6, 2];
                let diagonal = self.game_state.player.tile_detection.diagonal_tile() | bits;
                self.tile_detect_position_mut().set_diagonal_tile(diagonal);
                self.tile_detect_position_mut().or_slope_collision_bits(bits);
                self.tile_detect_position_mut()
                    .set_diag_state(DIAG_STATE[(tile & 3) as usize]);
            }
            0x1c => {
                let water_stair = self.game_state.player.tile_detection.water_staircase() | bits;
                self.tile_detect_position_mut().set_water_staircase(water_stair);
            }
            0x1d..=0x1f => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_inroom_staircase(bits);
                self.tile_detect_position_mut().or_stair_tile(bits as u8);
            }
            0x20 | 0xb0..=0xbd => {
                if !self.game_state.player.follower_link.has_somaria_platform_state() {
                    self.tile_detect_position_mut().or_pit_tile(bits as u8);
                }
            }
            0x22 | 0x30..=0x37 => {
                self.tile_detect_position_mut().or_stair_tile(bits as u8);
            }
            0x27 => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_misc_tiles(misc);
            }
            0x28 => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_vertical_ledge(bits as u8);
            }
            0x29 => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_vertical_ledge((bits << 4) as u8);
            }
            0x2a | 0x2b => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_horizontal_ledge(bits as u8);
            }
            0x2c | 0x2e => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_horizontal_ledge((bits << 4) as u8);
            }
            0x2d | 0x2f => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_ledges_down_leftright(bits as u8);
            }
            0x3d..=0x3f => {
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                self.tile_detect_position_mut().or_inroom_staircase(bits << 4);
                self.tile_detect_position_mut().or_stair_tile(bits as u8);
            }
            0x40 => {
                let grass = self.game_state.player.tile_detection.thick_grass() | bits;
                self.tile_detect_position_mut().set_thick_grass(grass);
            }
            0x44 => {
                if !self.game_state.player.follower_link.is_menu_blocked()
                    && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0
                {
                    self.tile_detect_position_mut().or_spike_cactus_tiles(bits as u8);
                } else {
                self.tile_detect_position_mut().or_collision_bits(bits);
                }
            }
            0x46 => {
                self.tile_detect_position_mut().or_spike_floor_and_triggers(bits as u8);
                self.tile_detect_position_mut().or_collision_bits(bits);
            }
            0x48 | 0x4a => {
                let aftermath = self.game_state.player.tile_detection.destruction_aftermath() | bits;
                let normal = self.game_state.player.tile_detection.normal_tiles() | bits;
                self.tile_detect_position_mut().set_destruction_aftermath(aftermath);
                self.tile_detect_position_mut().set_normal_tiles(normal);
            }
            0x4b => {
                let grass = self.game_state.player.tile_detection.thick_grass() | (bits << 4);
                self.tile_detect_position_mut().set_thick_grass(grass);
            }
            0x4c | 0x4d => {
                if !is_indoors {
                    self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                    self.tile_detect_position_mut().or_diagonal_ledge_tiles(bits as u8);
                }
            }
            0x4e | 0x4f => {
                if !is_indoors {
                    self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                    self.tile_detect_position_mut().or_diagonal_ledge_tiles((bits << 4) as u8);
                }
            }
            0x50..=0x56 => {
                const TILE50_DATA: [u8; 7] = [0x54, 0x52, 0x50, 0x51, 0x53, 0x55, 0x56];
                if let Some(i) = TILE50_DATA.iter().rposition(|&value| value == tile) {
                    if tile == 0x50 || tile == 0x51 {
                        self.tile_detect_position_mut().or_dashable_tiles((bits << 4) as u8);
                    }
                    let read_something = self.game_state.player.tile_detection.read_something() | bits;
                    let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                    let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                    self.tile_detect_position_mut().set_read_something(read_something);
                    self.tile_detect_position_mut().set_liftable_tile_index((i * 2) as u8);
                    self.tile_detect_position_mut().set_collision_bits(r14);
                    self.tile_detect_position_mut().set_misc_tiles(misc);
                }
            }
            0x57 => {
                self.tile_detect_position_mut().or_collision_bits(bits);
                self.tile_detect_position_mut().or_dashable_tiles((bits << 4) as u8);
            }
            0x58..=0x5d | 0x63 => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                self.tile_detect_position_mut().set_misc_tiles(misc);
                self.tile_detect_position_mut().set_interacting_tile(tile as u16);
                if tile != 0x63
                    && self.game_state.dungeon.room_items.chest_location((tile - 0x58) as usize)
                        >= 0x8000
                {
                    self.tile_detect_position_mut().set_collision_bits(r14);
                    self.tile_detect_position_mut().or_key_lock_gravestones((bits << 4) as u8);
                    if bits & 2 != 0 {
                        self.tile_detect_position_mut().set_tile_type(tile as u16);
                    }
                } else {
                    let chest = self.game_state.player.tile_detection.chest() | bits;
                    self.tile_detect_position_mut().set_collision_bits(r14);
                    self.tile_detect_position_mut().set_chest(chest);
                }
            }
            0x60 => {
                if is_indoors {
                    let misc_bits = if self.game_state.dungeon.bg2_attributes.bg2_attr(offset + 64) == 0x60 {
                        bits << 8
                    } else {
                        bits << 12
                    };
                    let misc = self.game_state.player.tile_detection.misc_tiles() | misc_bits;
                    self.tile_detect_position_mut().set_misc_tiles(misc);
                } else {
                    let normal = self.game_state.player.tile_detection.normal_tiles() | bits;
                    self.tile_detect_position_mut().set_normal_tiles(normal);
                }
            }
            0x67 => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_misc_tiles(misc);
                self.tile_detect_position_mut().or_spike_cactus_tiles((bits << 4) as u8);
            }
            0x68 => {
                self.dungeon_environment_mut().or_moving_floor_check_flags(bits);
            }
            0x69 => {
                self.dungeon_environment_mut().or_moving_floor_check_flags(bits << 4);
            }
            0x6a => {
                self.dungeon_environment_mut().or_moving_floor_check_flags(bits << 8);
            }
            0x6b => {
                self.dungeon_environment_mut().or_moving_floor_check_flags(bits << 12);
            }
            0x70..=0x7f => {
                if bits & 2 != 0 {
                    let block_flags =
                        self.game_state.player.tile_detection.block_flags() | (1 << (tile & 0x0f));
                    self.tile_detect_position_mut().set_block_flags(block_flags);
                }
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_misc_tiles(misc);
            }
            0x80..=0x8d => {
                let r14 = self.game_state.player.tile_detection.collision_bits()
                    | if tile == 0x82 || tile == 0x83 {
                        (bits << 4) | (bits << 8)
                    } else {
                        bits << 4
                    };
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_door_direction_flags(2 * (tile as u16 & 1));
            }
            0x8e | 0x8f => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | (bits << 4);
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().or_dashable_tiles(bits as u8);
                self.tile_detect_position_mut().clear_door_direction_flags();
            }
            0x90..=0x9f | 0xa8..=0xaf => {
                self.set_room_transitioning_flags(if tile < 0x98 { 1 } else { 3 });
                let r14 = self.game_state.player.tile_detection.collision_bits() | (bits << 4) | (bits << 8);
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_door_direction_flags(2 * (tile as u16 & 1));
            }
            0xa0..=0xa5 => {
                self.set_room_transitioning_flags(2);
                let r14 = self.game_state.player.tile_detection.collision_bits()
                    | if tile == 0xa2 || tile == 0xa3 {
                        (bits << 4) | (bits << 8)
                    } else {
                        bits << 4
                    };
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_door_direction_flags(2 * (tile as u16 & 1));
            }
            0xc0..=0xcf => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | bits;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_misc_tiles(misc);
            }
            0xf0..=0xff => {
                let r14 = self.game_state.player.tile_detection.collision_bits() | bits;
                let misc = self.game_state.player.tile_detection.misc_tiles() | (bits << 4);
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.tile_detect_position_mut().set_misc_tiles(misc);
            }
            0x42 => {
                if !is_indoors {
                    self.tile_detect_position_mut().or_key_lock_gravestones(bits as u8);
                self.tile_detect_position_mut().or_collision_bits(bits);
                }
            }
        }
    }
}
