// Methods ported from zelda3/src/tile_detect.c and included inside ZeldaState.

use super::*;

const K_DETECT_TILES_TAB0: [u8; 4] = [8, 24, 0, 15];
const K_DETECT_TILES_TAB1: [u8; 4] = [0, 0, 8, 8];
const K_DETECT_TILES_TAB2: [u8; 4] = [8, 8, 16, 16];
const K_DETECT_TILES_TAB3: [u8; 4] = [15, 15, 23, 23];
const K_DETECT_TILES_TAB4: [i8; 4] = [7, 24, -1, 16];
const K_DETECT_TILES_TAB5: [u8; 4] = [0, 0, 8, 8];
const K_DETECT_TILES_TAB6: [u8; 4] = [15, 15, 23, 23];

impl ZeldaState {
    pub fn overworld_get_tile_attribute_at_location(&self, x: u16, y: u16) -> u8 {
        let pos = ((y.wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_Y))
            & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_Y))
            << 3)
            | (x.wrapping_sub(read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_X))
                & read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_X));
        let map16 = read_le_u16(&self.ram, DUNG_BG2 + ((pos >> 1) as usize) * 2);
        let map8_index = (map16 as usize) * 4 + (((y & 8) >> 2) | (x & 1)) as usize;
        let map8 = self.asset_u16(70, map8_index);
        let mut attr = self.asset_u8(163, (map8 & 0x01ff) as usize);
        if (0x10..0x1c).contains(&attr) {
            attr |= ((map8 >> 14) & 1) as u8;
        }
        if env::var("ZELDA3_REPLAY_TRACE_TILE").is_ok()
            && self.replay_trace_filter_matches_current_frame()
        {
            eprintln!(
                "tile-probe frame={} x=0x{:04x} y=0x{:04x} pos=0x{:04x} map16=0x{:04x} map8=0x{:04x} attr=0x{:02x} base=0x{:04x}/0x{:04x} mask=0x{:04x}/0x{:04x}",
                self.ram[FRAME_COUNTER],
                x,
                y,
                pos,
                map16,
                map8,
                attr,
                read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_X),
                read_le_u16(&self.ram, OVERWORLD_OFFSET_BASE_Y),
                read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_X),
                read_le_u16(&self.ram, OVERWORLD_OFFSET_MASK_Y),
            );
        }
        attr
    }

    pub(super) fn tile_detect_movement_y(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.ram[TILEDETECT_PIT_TILE] = 0;
        let direction = direction as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let detect_y = link_y.wrapping_add(K_DETECT_TILES_TAB0[direction] as u16);
        write_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS, detect_y);
        let y = detect_y & mask;
        let x0 = (link_x.wrapping_add(K_DETECT_TILES_TAB1[direction] as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(K_DETECT_TILES_TAB2[direction] as u16) & mask) >> 3;
        let x2 = (link_x.wrapping_add(K_DETECT_TILES_TAB3[direction] as u16) & mask) >> 3;
        write_le_u16(&mut self.ram, SCRATCH_1, x2);
        self.tile_detection_execute(x0, y, 1);
        self.tile_detection_execute(x1, y, 2);
        self.tile_detection_execute(x2, y, 4);
    }

    pub(super) fn tile_detect_movement_x(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.ram[TILEDETECT_PIT_TILE] = 0;
        let direction = direction as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x = (link_x.wrapping_add(K_DETECT_TILES_TAB0[direction] as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(K_DETECT_TILES_TAB1[direction] as u16) & mask;
        let y1_pos = link_y.wrapping_add(K_DETECT_TILES_TAB2[direction] as u16);
        write_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS, y1_pos);
        let y1 = y1_pos & mask;
        let y2_pos = link_y.wrapping_add(K_DETECT_TILES_TAB3[direction] as u16);
        write_le_u16(&mut self.ram, TILEDETECT_WHICH_Y_POS + 2, y2_pos);
        let y2 = y2_pos & mask;
        self.tile_detection_execute(x, y0, 1);
        self.tile_detection_execute(x, y1, 2);
        self.tile_detection_execute(x, y2, 4);
    }

    pub(super) fn tile_detect_movement_vertical_slopes(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.ram[TILEDETECT_PIT_TILE] = 0;
        let direction = direction as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let y = link_y.wrapping_add(K_DETECT_TILES_TAB4[direction] as i16 as u16) & mask;
        let x0 = (link_x.wrapping_add(K_DETECT_TILES_TAB5[direction] as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(K_DETECT_TILES_TAB6[direction] as u16) & mask) >> 3;
        self.tile_detection_execute(x0, y, 1);
        self.tile_detection_execute(x1, y, 2);
    }

    pub(super) fn tile_detect_movement_horizontal_slopes(&mut self, direction: u16) {
        assert!(direction < 4);
        self.tile_detect_reset_state();
        self.ram[TILEDETECT_PIT_TILE] = 0;
        let direction = direction as usize;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x = (link_x.wrapping_add(K_DETECT_TILES_TAB4[direction] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(K_DETECT_TILES_TAB5[direction] as u16) & mask;
        let y1 = link_y.wrapping_add(K_DETECT_TILES_TAB6[direction] as u16) & mask;
        self.tile_detection_execute(x, y0, 1);
        self.tile_detection_execute(x, y1, 2);
    }

    pub(super) fn player_tile_detect_nearby(&mut self) {
        self.tile_detect_reset_state();
        self.ram[TILEDETECT_PIT_TILE] = 0;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x0 = (link_x.wrapping_add(K_DETECT_TILES_TAB1[0] as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(K_DETECT_TILES_TAB3[0] as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(K_DETECT_TILES_TAB1[2] as u16) & mask;
        let y1 = link_y.wrapping_add(K_DETECT_TILES_TAB3[2] as u16) & mask;
        write_le_u16(&mut self.ram, SCRATCH_1, y0);
        self.tile_detection_execute(x0, y0, 8);
        self.tile_detection_execute(x0, y1, 2);
        self.tile_detection_execute(x1, y0, 4);
        self.tile_detection_execute(x1, y1, 1);
    }

    pub(super) fn hookshot_check_tile_collision(&mut self, k: i32) {
        let k = k as usize;
        let bak0 = self.ram[DUNGEON_ROOM_INDEX];
        let bak1 = self.ram[LINK_IS_ON_LOWER_LEVEL];
        if self.ram[ANCILLA_ARR1 + k] != 0 {
            if self.ram[KIND_OF_IN_ROOM_STAIRCASE] == 0 {
                self.ram[DUNGEON_ROOM_INDEX] = self.ram[DUNGEON_ROOM_INDEX].wrapping_add(0x10);
            }
            self.ram[LINK_IS_ON_LOWER_LEVEL] ^= 1;
        }
        let x = self.ancilla_x(k);
        let y = self.ancilla_y(k);
        let dir = self.ram[ANCILLA_DIR + k] as i32;
        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();
        if self.ram[DUNG_HDR_COLLISION] == 2 {
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 1;
            self.hookshot_check_single_layer_tile_collision(
                x.wrapping_add(read_le_u16(&self.ram, BG1HOFS_COPY2))
                    .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2)),
                y.wrapping_add(read_le_u16(&self.ram, BG1VOFS_COPY2))
                    .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2)),
                dir,
            );
            self.ram[LINK_IS_ON_LOWER_LEVEL] = 0;
        }
        self.hookshot_check_single_layer_tile_collision(x, y, dir);
        self.ram[LINK_IS_ON_LOWER_LEVEL] = bak1;
        self.ram[DUNGEON_ROOM_INDEX] = bak0;
    }

    pub(super) fn hookshot_check_single_layer_tile_collision(&mut self, x: u16, y: u16, dir: i32) {
        const CHECK_X: [u8; 8] = [0, 15, 0, 15, 0, 0, 8, 8];
        const CHECK_Y: [u8; 8] = [0, 0, 7, 7, 0, 15, 0, 15];
        let base = dir as usize * 2;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let y0 = y.wrapping_add(CHECK_Y[base] as u16) & mask;
        let y1 = y.wrapping_add(CHECK_Y[base + 1] as u16) & mask;
        let x0 = (x.wrapping_add(CHECK_X[base] as u16) & mask) >> 3;
        let x1 = (x.wrapping_add(CHECK_X[base + 1] as u16) & mask) >> 3;
        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);
    }

    pub(super) fn handle_nudging_in_a_door(&mut self, speed: i8) {
        let y = if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 != 0 {
            if (self.player_state_view().y() as u8) < 0x80 {
                1
            } else {
                0
            }
        } else if (self.player_state_view().x() as u8) < 0x80 {
            3
        } else {
            2
        };
        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();
        const DETECT_Y: [i8; 4] = [8, 23, 16, 16];
        const DETECT_X: [i8; 4] = [8, 8, 0, 15];
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x0 = (link_x.wrapping_add(DETECT_X[y] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(DETECT_Y[y] as i16 as u16) & mask;
        self.tile_detection_execute(x0, y0, 1);
        if ((read_le_u16(&self.ram, R14) | self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] as u16)
            & 3)
            == 0
            && ((self.ram[TILEDETECT_VERTICAL_LEDGE] | self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES])
                & 0x33)
                == 0
        {
            return;
        }
        if self.ram[LINK_LAST_DIRECTION_MOVED_TOWARDS] & 2 != 0 {
            let y = self.player_state_view().y();
            write_le_u16(
                &mut self.ram,
                LINK_Y_COORD,
                y.wrapping_sub(speed as i16 as u16),
            );
        } else {
            let x = self.player_state_view().x();
            write_le_u16(
                &mut self.ram,
                LINK_X_COORD,
                x.wrapping_sub(speed as i16 as u16),
            );
        }
    }

    pub(super) fn tile_check_for_mirror_bonk(&mut self) {
        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x0 = (link_x.wrapping_add(2) & mask) >> 3;
        let x1 = (link_x.wrapping_add(13) & mask) >> 3;
        let y0 = link_y.wrapping_add(10) & mask;
        let y1 = link_y.wrapping_add(21) & mask;
        write_le_u16(&mut self.ram, SCRATCH_1, y0);
        self.tile_detection_execute(x0, y0, 8);
        self.tile_detection_execute(x0, y1, 2);
        self.tile_detection_execute(x1, y0, 4);
        self.tile_detection_execute(x1, y1, 1);
    }

    pub(super) fn tile_detect_sword_swing_deep_in_door(&mut self, dw: u8) {
        self.ram[TILEDETECT_PIT_TILE] = 0;
        self.tile_detect_reset_state();
        const DOORWAY_DETECT_X: [i8; 4] = [8, 8, -1, 16];
        const DOORWAY_DETECT_Y: [i8; 4] = [-1, 24, 16, 16];
        let o = dw.wrapping_sub(1) as usize * 2;
        let mask = read_le_u16(&self.ram, TILEMAP_LOCATION_CALC_MASK);
        let link_y = self.player_state_view().y();
        let link_x = self.player_state_view().x();
        let x0 = (link_x.wrapping_add(DOORWAY_DETECT_X[o] as i16 as u16) & mask) >> 3;
        let x1 = (link_x.wrapping_add(DOORWAY_DETECT_X[o + 1] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(DOORWAY_DETECT_Y[o] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(DOORWAY_DETECT_Y[o + 1] as i16 as u16) & mask;
        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);
    }

    pub(super) fn tile_detect_reset_state(&mut self) {
        write_le_u16(&mut self.ram, R12, 0);
        write_le_u16(&mut self.ram, R14, 0);
        write_le_u16(&mut self.ram, TILEDETECT_DIAGONAL_TILE, 0);
        self.ram[TILEDETECT_STAIR_TILE] = 0;
        self.ram[TILEDETECT_PIT_TILE] = 0;
        write_le_u16(&mut self.ram, TILEDETECT_INROOM_STAIRCASE, 0);
        write_le_u16(&mut self.ram, TILEDETECT_BLOCK_FLAGS_LO, 0);
        write_le_u16(&mut self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, 0);
        write_le_u16(&mut self.ram, TILEDETECT_MOVING_FLOOR_TILES, 0);
        write_le_u16(&mut self.ram, TILEDETECT_DEEPWATER, 0);
        write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, 0);
        write_le_u16(&mut self.ram, TILEDETECT_ICY_FLOOR, 0);
        write_le_u16(&mut self.ram, TILEDETECT_WATER_STAIRCASE, 0);
        write_le_u16(&mut self.ram, TILEDETECT_THICK_GRASS, 0);
        write_le_u16(&mut self.ram, TILEDETECT_SHALLOW_WATER, 0);
        write_le_u16(&mut self.ram, TILEDETECT_DESTRUCTION_AFTERMATH, 0);
        write_le_u16(&mut self.ram, TILEDETECT_READ_SOMETHING, 0);
        self.ram[TILEDETECT_VERTICAL_LEDGE] = 0;
        self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] = 0;
        self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] = 0;
        self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] = 0;
        write_le_u16(&mut self.ram, TILEDETECT_CHEST, 0);
        self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] = 0;
        self.ram[BITFIELD_SPIKE_CACTUS_TILES] = 0;
        self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] = 0;
        self.ram[BITMASK_FOR_DASHABLE_TILES] = 0;
        write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, 0);
        write_le_u16(&mut self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, 0);
    }

    pub(super) fn tile_detection_execute(&mut self, x: u16, y: u16, bits: u16) {
        let mut offset = 0usize;
        let is_indoors = self.ram[PLAYER_IS_INDOORS] != 0;
        let trace = env::var("ZELDA3_REPLAY_TRACE_TILE").is_ok()
            && self.replay_trace_filter_matches_current_frame();
        let r14_before = read_le_u16(&self.ram, R14);
        let r12_before = read_le_u16(&self.ram, R12);
        let misc_before = read_le_u16(&self.ram, TILEDETECT_MISC_TILES);
        let normal_before = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES);
        let pit_before = self.ram[TILEDETECT_PIT_TILE];
        let diag_before = read_le_u16(&self.ram, TILEDETECT_DIAG_STATE);
        let below_before = self.ram[LINK_TILE_BELOW];
        let tile = if is_indoors {
            let force_move = read_le_u16(&self.ram, FORCE_MOVE_ANY_DIRECTION) & 0x00ff;
            write_le_u16(&mut self.ram, FORCE_MOVE_ANY_DIRECTION, force_move);
            offset = ((y & !7) as usize) * 8
                + (x as usize & 63)
                + if self.ram[LINK_IS_ON_LOWER_LEVEL] != 0 {
                    0x1000
                } else {
                    0
                };
            let mut tile = self.ram[DUNG_BG2_ATTR_TABLE + offset];
            if self.ram[CHEAT_WALK_THROUGH_WALLS] != 0 {
                tile = 0;
            }
            self.ram[LINK_TILE_BELOW] = tile;
            tile
        } else {
            self.overworld_get_tile_attribute_at_location(x, y)
        };
        self.tile_detect_execute_inner(tile, offset as u16, bits, is_indoors);
        if trace {
            eprintln!(
                "tile-exec frame={} x=0x{:04x} y=0x{:04x} bits=0x{:04x} indoors={} lower={} offs=0x{:04x} tile=0x{:02x} link=0x{:04x}/0x{:04x} speed=0x{:02x}/0x{:02x} r14=0x{:04x}->0x{:04x} r12=0x{:04x}->0x{:04x} misc=0x{:04x}->0x{:04x} normal=0x{:04x}->0x{:04x} pit=0x{:02x}->0x{:02x} diag=0x{:04x}->0x{:04x} below=0x{:02x}->0x{:02x}",
                self.ram[FRAME_COUNTER],
                x,
                y,
                bits,
                self.ram[PLAYER_IS_INDOORS],
                self.ram[LINK_IS_ON_LOWER_LEVEL],
                offset,
                tile,
                self.player_state_view().x(),
                self.player_state_view().y(),
                self.ram[LINK_SPEED_SETTING],
                self.ram[LINK_SPEED_MODIFIER],
                r14_before,
                read_le_u16(&self.ram, R14),
                r12_before,
                read_le_u16(&self.ram, R12),
                misc_before,
                read_le_u16(&self.ram, TILEDETECT_MISC_TILES),
                normal_before,
                read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES),
                pit_before,
                self.ram[TILEDETECT_PIT_TILE],
                diag_before,
                read_le_u16(&self.ram, TILEDETECT_DIAG_STATE),
                below_before,
                self.ram[LINK_TILE_BELOW],
            );
        }
    }

    #[rustfmt::skip]
    pub(super) fn tile_detect_execute_inner(&mut self, mut tile: u8, offs: u16, bits: u16, is_indoors: bool) {
        if self.ram[CHEAT_WALK_THROUGH_WALLS] != 0 {
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
                    let normal = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) | bits;
                    write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, normal);
                }
            }
            0x01 | 0x02 | 0x03 | 0x26 | 0x43 => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                write_le_u16(&mut self.ram, R14, r14);
            }
            0x04 => {
                if is_indoors {
                    let r14 = read_le_u16(&self.ram, R14) | bits;
                    write_le_u16(&mut self.ram, R14, r14);
                } else {
                    let grass = read_le_u16(&self.ram, TILEDETECT_THICK_GRASS) | bits;
                    write_le_u16(&mut self.ram, TILEDETECT_THICK_GRASS, grass);
                }
            }
            0x0b => {
                if is_indoors {
                    let r14 = read_le_u16(&self.ram, R14) | bits;
                    write_le_u16(&mut self.ram, R14, r14);
                } else {
                    write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                    let deepwater = read_le_u16(&self.ram, TILEDETECT_DEEPWATER) | (bits << 4);
                    write_le_u16(&mut self.ram, TILEDETECT_DEEPWATER, deepwater);
                }
            }
            0x08 => {
                let deepwater = read_le_u16(&self.ram, TILEDETECT_DEEPWATER) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_DEEPWATER, deepwater);
            }
            0x09 => {
                let shallow = read_le_u16(&self.ram, TILEDETECT_SHALLOW_WATER) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_SHALLOW_WATER, shallow);
            }
            0x0a => {
                let normal = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, normal);
            }
            0x0c => {
                let moving_floor = read_le_u16(&self.ram, TILEDETECT_MOVING_FLOOR_TILES) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_MOVING_FLOOR_TILES, moving_floor);
            }
            0x0d => {
                if self.ram[FLAG_BLOCK_LINK_MENU] == 0
                    && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0
                {
                    self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] |= (bits << 4) as u8;
                }
            }
            0x0e => {
                let icy = read_le_u16(&self.ram, TILEDETECT_ICY_FLOOR) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_ICY_FLOOR, icy);
            }
            0x0f => {
                let icy = read_le_u16(&self.ram, TILEDETECT_ICY_FLOOR) | (bits << 4);
                write_le_u16(&mut self.ram, TILEDETECT_ICY_FLOOR, icy);
            }
            0x6c..=0x6f if is_indoors => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                write_le_u16(&mut self.ram, R14, r14);
            }
            0x6c..=0x6f => {
                let normal = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, normal);
            }
            0x10..=0x13 => {
                const DIAG_STATE: [u16; 4] = [4, 0, 6, 2];
                let r12 = read_le_u16(&self.ram, R12) | bits;
                write_le_u16(&mut self.ram, R12, r12);
                write_le_u16(
                    &mut self.ram,
                    TILEDETECT_DIAG_STATE,
                    DIAG_STATE[(tile & 3) as usize],
                );
            }
            0x18..=0x1b => {
                const DIAG_STATE: [u16; 4] = [4, 0, 6, 2];
                let diagonal = read_le_u16(&self.ram, TILEDETECT_DIAGONAL_TILE) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_DIAGONAL_TILE, diagonal);
                let r12 = read_le_u16(&self.ram, R12) | bits;
                write_le_u16(&mut self.ram, R12, r12);
                write_le_u16(
                    &mut self.ram,
                    TILEDETECT_DIAG_STATE,
                    DIAG_STATE[(tile & 3) as usize],
                );
            }
            0x1c => {
                let water_stair = read_le_u16(&self.ram, TILEDETECT_WATER_STAIRCASE) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_WATER_STAIRCASE, water_stair);
            }
            0x1d..=0x1f => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                let stairs = read_le_u16(&self.ram, TILEDETECT_INROOM_STAIRCASE) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_INROOM_STAIRCASE, stairs);
                self.ram[TILEDETECT_STAIR_TILE] |= bits as u8;
            }
            0x20 | 0xb0..=0xbd => {
                if self.ram[PLAYER_ON_SOMARIA_PLATFORM] == 0 {
                    self.ram[TILEDETECT_PIT_TILE] |= bits as u8;
                }
            }
            0x22 | 0x30..=0x37 => {
                self.ram[TILEDETECT_STAIR_TILE] |= bits as u8;
            }
            0x27 => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
            }
            0x28 => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                self.ram[TILEDETECT_VERTICAL_LEDGE] |= bits as u8;
            }
            0x29 => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                self.ram[TILEDETECT_VERTICAL_LEDGE] |= (bits << 4) as u8;
            }
            0x2a | 0x2b => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] |= bits as u8;
            }
            0x2c | 0x2e => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                self.ram[DETECTION_OF_LEDGE_TILES_HORIZ_UPHORIZ] |= (bits << 4) as u8;
            }
            0x2d | 0x2f => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                self.ram[TILEDETECT_LEDGES_DOWN_LEFTRIGHT] |= bits as u8;
            }
            0x3d..=0x3f => {
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                let stairs = read_le_u16(&self.ram, TILEDETECT_INROOM_STAIRCASE) | (bits << 4);
                write_le_u16(&mut self.ram, TILEDETECT_INROOM_STAIRCASE, stairs);
                self.ram[TILEDETECT_STAIR_TILE] |= bits as u8;
            }
            0x40 => {
                let grass = read_le_u16(&self.ram, TILEDETECT_THICK_GRASS) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_THICK_GRASS, grass);
            }
            0x44 => {
                if self.ram[FLAG_BLOCK_LINK_MENU] == 0
                    && read_le_u16(&self.ram, DUNG_SAVEGAME_STATE_BITS) & 0x8000 == 0
                {
                    self.ram[BITFIELD_SPIKE_CACTUS_TILES] |= bits as u8;
                } else {
                    let r14 = read_le_u16(&self.ram, R14) | bits;
                    write_le_u16(&mut self.ram, R14, r14);
                }
            }
            0x46 => {
                self.ram[TILEDETECT_SPIKE_FLOOR_AND_TILE_TRIGGERS] |= bits as u8;
                let r14 = read_le_u16(&self.ram, R14) | bits;
                write_le_u16(&mut self.ram, R14, r14);
            }
            0x48 | 0x4a => {
                let aftermath = read_le_u16(&self.ram, TILEDETECT_DESTRUCTION_AFTERMATH) | bits;
                let normal = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_DESTRUCTION_AFTERMATH, aftermath);
                write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, normal);
            }
            0x4b => {
                let grass = read_le_u16(&self.ram, TILEDETECT_THICK_GRASS) | (bits << 4);
                write_le_u16(&mut self.ram, TILEDETECT_THICK_GRASS, grass);
            }
            0x4c | 0x4d => {
                if !is_indoors {
                    write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                    self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] |= bits as u8;
                }
            }
            0x4e | 0x4f => {
                if !is_indoors {
                    write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                    self.ram[DETECTION_OF_UNKNOWN_TILE_TYPES] |= (bits << 4) as u8;
                }
            }
            0x50..=0x56 => {
                const TILE50_DATA: [u8; 7] = [0x54, 0x52, 0x50, 0x51, 0x53, 0x55, 0x56];
                if let Some(i) = TILE50_DATA.iter().rposition(|&value| value == tile) {
                    if tile == 0x50 || tile == 0x51 {
                        self.ram[BITMASK_FOR_DASHABLE_TILES] |= (bits << 4) as u8;
                    }
                    let read_something = read_le_u16(&self.ram, TILEDETECT_READ_SOMETHING) | bits;
                    let r14 = read_le_u16(&self.ram, R14) | bits;
                    let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                    write_le_u16(&mut self.ram, TILEDETECT_READ_SOMETHING, read_something);
                    self.ram[INTERACTING_WITH_LIFTABLE_TILE_X2] = (i * 2) as u8;
                    write_le_u16(&mut self.ram, R14, r14);
                    write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
                }
            }
            0x57 => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                write_le_u16(&mut self.ram, R14, r14);
                self.ram[BITMASK_FOR_DASHABLE_TILES] |= (bits << 4) as u8;
            }
            0x58..=0x5d | 0x63 => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
                write_le_u16(&mut self.ram, INDEX_OF_INTERACTING_TILE, tile as u16);
                if tile != 0x63
                    && read_le_u16(&self.ram, DUNG_CHEST_LOCATIONS + (tile - 0x58) as usize * 2)
                        >= 0x8000
                {
                    write_le_u16(&mut self.ram, R14, r14);
                    self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] |= (bits << 4) as u8;
                    if bits & 2 != 0 {
                        write_le_u16(&mut self.ram, TILEDETECT_TILE_TYPE, tile as u16);
                    }
                } else {
                    let chest = read_le_u16(&self.ram, TILEDETECT_CHEST) | bits;
                    write_le_u16(&mut self.ram, R14, r14);
                    write_le_u16(&mut self.ram, TILEDETECT_CHEST, chest);
                }
            }
            0x60 => {
                if is_indoors {
                    let misc_bits = if self.ram[DUNG_BG2_ATTR_TABLE + offset + 64] == 0x60 {
                        bits << 8
                    } else {
                        bits << 12
                    };
                    let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | misc_bits;
                    write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
                } else {
                    let normal = read_le_u16(&self.ram, TILEDETECT_NORMAL_TILES) | bits;
                    write_le_u16(&mut self.ram, TILEDETECT_NORMAL_TILES, normal);
                }
            }
            0x67 => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
                self.ram[BITFIELD_SPIKE_CACTUS_TILES] |= (bits << 4) as u8;
            }
            0x68 => {
                let conveyor = read_le_u16(&self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | bits;
                write_le_u16(&mut self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, conveyor);
            }
            0x69 => {
                let conveyor = read_le_u16(&self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | (bits << 4);
                write_le_u16(&mut self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, conveyor);
            }
            0x6a => {
                let conveyor = read_le_u16(&self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | (bits << 8);
                write_le_u16(&mut self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, conveyor);
            }
            0x6b => {
                let conveyor = read_le_u16(&self.ram, MOVING_FLOOR_BG_CHECK_FLAGS) | (bits << 12);
                write_le_u16(&mut self.ram, MOVING_FLOOR_BG_CHECK_FLAGS, conveyor);
            }
            0x70..=0x7f => {
                if bits & 2 != 0 {
                    let var2 = read_le_u16(&self.ram, TILEDETECT_BLOCK_FLAGS_LO) | (1 << (tile & 0x0f));
                    write_le_u16(&mut self.ram, TILEDETECT_BLOCK_FLAGS_LO, var2);
                }
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
            }
            0x80..=0x8d => {
                let r14 = read_le_u16(&self.ram, R14)
                    | if tile == 0x82 || tile == 0x83 {
                        (bits << 4) | (bits << 8)
                    } else {
                        bits << 4
                    };
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, 2 * (tile as u16 & 1));
            }
            0x8e | 0x8f => {
                let r14 = read_le_u16(&self.ram, R14) | (bits << 4);
                write_le_u16(&mut self.ram, R14, r14);
                self.ram[BITMASK_FOR_DASHABLE_TILES] |= bits as u8;
                write_le_u16(&mut self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, 0);
            }
            0x90..=0x9f | 0xa8..=0xaf => {
                self.ram[ROOM_TRANSITIONING_FLAGS] = if tile < 0x98 { 1 } else { 3 };
                let r14 = read_le_u16(&self.ram, R14) | (bits << 4) | (bits << 8);
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, 2 * (tile as u16 & 1));
            }
            0xa0..=0xa5 => {
                self.ram[ROOM_TRANSITIONING_FLAGS] = 2;
                let r14 = read_le_u16(&self.ram, R14)
                    | if tile == 0xa2 || tile == 0xa3 {
                        (bits << 4) | (bits << 8)
                    } else {
                        bits << 4
                    };
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_DOOR_DIRECTION_FLAGS, 2 * (tile as u16 & 1));
            }
            0xc0..=0xcf => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | bits;
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
            }
            0xf0..=0xff => {
                let r14 = read_le_u16(&self.ram, R14) | bits;
                let misc = read_le_u16(&self.ram, TILEDETECT_MISC_TILES) | (bits << 4);
                write_le_u16(&mut self.ram, R14, r14);
                write_le_u16(&mut self.ram, TILEDETECT_MISC_TILES, misc);
            }
            0x42 => {
                if !is_indoors {
                    self.ram[TILEDETECT_KEY_LOCK_GRAVESTONES] |= bits as u8;
                    let r14 = read_le_u16(&self.ram, R14) | bits;
                    write_le_u16(&mut self.ram, R14, r14);
                }
            }
        }
    }
}
