// Methods ported from zelda3/src/tile_detect.c and included inside ZeldaState.

use super::*;
use crate::game_state::constants::MENU_PREV_JOYPAD_H;
use crate::game_state::{
    CollisionAxis, CollisionDirection, MovementProbe, MovementProbeKind, PlayerFootprint,
    TileResult,
};
use crate::tile_definition::{NativeTile, TileBehavior};
const HOOKSHOT_SINGLE_LAYER_CHECK_X_OFFSETS: [u8; 8] = [0, 15, 0, 15, 0, 0, 8, 8];
const HOOKSHOT_SINGLE_LAYER_CHECK_Y_OFFSETS: [u8; 8] = [0, 0, 7, 7, 0, 15, 0, 15];
const DOOR_NUDGE_DETECT_Y_OFFSETS: [i8; 4] = [8, 23, 16, 16];
const DOOR_NUDGE_DETECT_X_OFFSETS: [i8; 4] = [8, 8, 0, 15];
const SWORD_DOORWAY_DETECT_X_OFFSETS: [i8; 4] = [8, 8, -1, 16];
const SWORD_DOORWAY_DETECT_Y_OFFSETS: [i8; 4] = [-1, 24, 16, 16];
const TILE_DETECT_DIAG_STATES: [u16; 4] = [4, 0, 6, 2];

impl ZeldaState {
    pub fn overworld_get_tile_attribute_at_location(&self, x: u16, y: u16) -> u8 {
        self.overworld_tile_definition_at_location(x, y)
            .cartridge_attribute()
    }

    pub(super) fn overworld_tile_definition_at_location(&self, x: u16, y: u16) -> NativeTile {
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
        self.assets
            .as_ref()
            .map(|assets| assets.outdoor_tile_definition(map8))
            .unwrap_or_default()
    }

    pub(super) fn detect_player_movement(
        &mut self,
        axis: CollisionAxis,
        direction: CollisionDirection,
        kind: MovementProbeKind,
    ) {
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let player = &self.game_state.player.follower_link;
        let probe = MovementProbe::new(player.x(), player.y(), axis, direction, kind);
        let points = probe.points();

        // Preserve the original scratch stores before any tile is executed.
        // Horizontal detection stores a Y position in probe_x; slope probes
        // leave both position words and the anchor untouched.
        if kind == MovementProbeKind::Cardinal {
            match axis {
                CollisionAxis::Vertical => {
                    self.tile_detect_position_mut().set_y(points[0].y);
                    self.tile_detect_position_mut()
                        .set_tile_probe_anchor((points[2].x & mask) >> 3);
                }
                CollisionAxis::Horizontal => {
                    self.tile_detect_position_mut().set_y(points[1].y);
                    self.tile_detect_position_mut().set_x(points[2].y);
                }
            }
        }
        // Result bits follow execution order. The high-side slope sample is
        // the second probe (bit 2), whereas a cardinal high side uses bit 4.
        for (index, point) in points.iter().enumerate() {
            self.tile_detection_execute((point.x & mask) >> 3, point.y & mask, 1 << index);
        }
    }

    pub(super) fn player_tile_detect_nearby(&mut self) {
        self.tile_detect_reset_state();
        self.tile_detect_position_mut().clear_pit_tile();
        self.detect_player_footprint(PlayerFootprint::Body);
    }

    fn detect_player_footprint(&mut self, footprint: PlayerFootprint) {
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let player = &self.game_state.player.follower_link;
        let corners = footprint.corners(player.x(), player.y());
        self.tile_detect_position_mut()
            .set_tile_probe_anchor(corners[0].1.y & mask);
        for (corner, point) in corners {
            self.tile_detection_execute((point.x & mask) >> 3, point.y & mask, corner.result_bit());
        }
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
        let base = dir as usize * 2;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let y0 = y.wrapping_add(HOOKSHOT_SINGLE_LAYER_CHECK_Y_OFFSETS[base] as u16) & mask;
        let y1 = y.wrapping_add(HOOKSHOT_SINGLE_LAYER_CHECK_Y_OFFSETS[base + 1] as u16) & mask;
        let x0 = (x.wrapping_add(HOOKSHOT_SINGLE_LAYER_CHECK_X_OFFSETS[base] as u16) & mask) >> 3;
        let x1 =
            (x.wrapping_add(HOOKSHOT_SINGLE_LAYER_CHECK_X_OFFSETS[base + 1] as u16) & mask) >> 3;
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
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(DOOR_NUDGE_DETECT_X_OFFSETS[y] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(DOOR_NUDGE_DETECT_Y_OFFSETS[y] as i16 as u16) & mask;
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
        self.detect_player_footprint(PlayerFootprint::MirrorClearance);
    }

    pub(super) fn tile_detect_sword_swing_deep_in_door(&mut self, dw: u8) {
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();
        let o = dw.wrapping_sub(1) as usize * 2;
        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let x0 = (link_x.wrapping_add(SWORD_DOORWAY_DETECT_X_OFFSETS[o] as i16 as u16) & mask) >> 3;
        let x1 =
            (link_x.wrapping_add(SWORD_DOORWAY_DETECT_X_OFFSETS[o + 1] as i16 as u16) & mask) >> 3;
        let y0 = link_y.wrapping_add(SWORD_DOORWAY_DETECT_Y_OFFSETS[o] as i16 as u16) & mask;
        let y1 = link_y.wrapping_add(SWORD_DOORWAY_DETECT_Y_OFFSETS[o + 1] as i16 as u16) & mask;
        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);
    }

    pub(super) fn tile_detect_reset_state(&mut self) {
        self.tile_detect_position_mut().reset_probe_results();
        self.dungeon_environment_mut()
            .clear_moving_floor_check_flags();
    }

    pub(super) fn tile_detection_execute(&mut self, x: u16, y: u16, bits: u16) {
        let mut offset = 0usize;
        let is_indoors = self.game_state.world.location.is_indoors();
        let tile = if is_indoors {
            self.follower_link_state_mut().clear_force_move_high_byte();
            offset = ((y & !7) as usize) * 8
                + (x as usize & 63)
                + if self.game_state.player.follower_link.lower_level_state() != 0 {
                    0x1000
                } else {
                    0
                };
            // TileDetection_Execute keeps the attribute offset in the $BD
            // scratch word (`STX $BD`, $07:DA23); the item menu's first-frame
            // d-pad gate reads that scratch byte (route host 1042435).
            self.ram[MENU_PREV_JOYPAD_H] = offset as u8;
            self.ram[MENU_PREV_JOYPAD_H + 1] = (offset >> 8) as u8;
            let mut tile = self.game_state.dungeon.bg2_attributes.bg2_tile(offset);
            if self
                .game_state
                .player
                .follower_link
                .cheat_walk_through_walls()
                != 0
            {
                tile = NativeTile::GROUND;
            }
            self.follower_link_state_mut()
                .set_tile_below(tile.cartridge_attribute());
            tile
        } else {
            self.overworld_tile_definition_at_location(x, y)
        };
        self.tile_detect_execute_definition(tile, offset as u16, bits, is_indoors);
    }

    #[cfg(test)]
    pub(super) fn tile_detect_execute_inner(
        &mut self,
        tile: u8,
        offs: u16,
        bits: u16,
        is_indoors: bool,
    ) {
        self.tile_detect_execute_definition(
            NativeTile::from_cartridge(tile),
            offs,
            bits,
            is_indoors,
        );
    }

    pub(super) fn tile_detect_execute_definition(
        &mut self,
        mut tile: NativeTile,
        offs: u16,
        bits: u16,
        is_indoors: bool,
    ) {
        use TileBehavior as B;
        use TileResult as R;
        if self
            .game_state
            .player
            .follower_link
            .cheat_walk_through_walls()
            != 0
        {
            tile = NativeTile::GROUND;
        }
        match tile.behavior(is_indoors) {
            B::Ignore => {}
            B::Surface { result, shift } => self.record_tile_result(result, bits << shift),
            B::Solid => self.record_tile_result(R::Collision, bits),
            B::DeepWaterEdge => {
                self.tile_detect_position_mut()
                    .set_interacting_tile(u16::from(tile.cartridge_attribute()));
                self.record_tile_result(R::DeepWater, bits << 4);
            }
            B::ConditionalFloorTrigger => {
                if self.tile_hazards_enabled() {
                    self.record_tile_result(R::SpikeTrigger, bits << 4);
                }
            }
            B::Slope { diagonal, shape } => {
                if diagonal {
                    self.record_tile_result(R::Diagonal, bits);
                }
                self.record_tile_result(R::Slope, bits);
                self.tile_detect_position_mut()
                    .set_diag_state(TILE_DETECT_DIAG_STATES[shape]);
            }
            B::InRoomStaircase { shift } => {
                self.tile_detect_position_mut()
                    .set_interacting_tile(u16::from(tile.cartridge_attribute()));
                self.record_tile_result(R::InRoomStaircase, bits << shift);
                self.record_tile_result(R::Stair, bits);
            }
            B::Pit => {
                if !self
                    .game_state
                    .player
                    .follower_link
                    .has_somaria_platform_state()
                {
                    self.record_tile_result(R::Pit, bits);
                }
            }
            B::SolidInteractable { misc_shift, cactus } => {
                self.record_solid_tile_interaction(bits, misc_shift);
                if cactus {
                    self.record_tile_result(R::SpikeCactus, bits << 4);
                }
            }
            B::Ledge { result, shift } => {
                self.tile_detect_position_mut()
                    .set_interacting_tile(u16::from(tile.cartridge_attribute()));
                self.record_tile_result(result, bits << shift);
            }
            B::Cactus => {
                let result = if self.tile_hazards_enabled() {
                    R::SpikeCactus
                } else {
                    R::Collision
                };
                self.record_tile_result(result, bits);
            }
            B::SpikedSolid => {
                self.record_tile_result(R::SpikeTrigger, bits);
                self.record_tile_result(R::Collision, bits);
            }
            B::Aftermath => {
                self.record_tile_result(R::Aftermath, bits);
                self.record_tile_result(R::Normal, bits);
            }
            B::Liftable { index, dashable } => {
                if dashable {
                    self.record_tile_result(R::Dashable, bits << 4);
                }
                self.record_tile_result(R::Readable, bits);
                self.tile_detect_position_mut()
                    .set_liftable_tile_index(index);
                self.record_solid_tile_interaction(bits, 0);
            }
            B::DashableSolid => {
                self.record_tile_result(R::Collision, bits);
                self.record_tile_result(R::Dashable, bits << 4);
            }
            B::Chest { index } => {
                // Chest lookup remains after the first two publications. Unlike
                // other solid interactions, chests publish Misc before Collision.
                self.record_tile_result(R::Misc, bits);
                self.tile_detect_position_mut()
                    .set_interacting_tile(u16::from(tile.cartridge_attribute()));
                if index.is_some_and(|index| {
                    self.game_state.dungeon.room_items.chest_location(index) >= 0x8000
                }) {
                    self.record_tile_result(R::Collision, bits);
                    self.record_tile_result(R::KeyLockGravestone, bits << 4);
                    if bits & 2 != 0 {
                        self.tile_detect_position_mut()
                            .set_tile_type(u16::from(tile.cartridge_attribute()));
                    }
                } else {
                    self.record_tile_result(R::Collision, bits);
                    self.record_tile_result(R::Chest, bits);
                }
            }
            B::NeighborDependent => {
                let shift = if self
                    .game_state
                    .dungeon
                    .bg2_attributes
                    .bg2_tile(usize::from(offs) + 64)
                    == tile
                {
                    8
                } else {
                    12
                };
                self.record_tile_result(R::Misc, bits << shift);
            }
            B::MovingFloorCheck { shift } => {
                self.dungeon_environment_mut()
                    .or_moving_floor_check_flags(bits << shift);
            }
            B::PushBlock { index } => {
                if bits & 2 != 0 {
                    self.record_tile_result(R::Block, 1 << index);
                }
                self.record_solid_tile_interaction(bits, 0);
            }
            B::Door {
                direction,
                forced_movement,
                transition,
                dashable,
            } => {
                if let Some(transition) = transition {
                    self.set_room_transitioning_flags(transition);
                }
                let collision = (bits << 4) | if forced_movement { bits << 8 } else { 0 };
                self.record_tile_result(R::Collision, collision);
                if dashable {
                    self.record_tile_result(R::Dashable, bits);
                    self.tile_detect_position_mut().clear_door_direction_flags();
                } else {
                    self.tile_detect_position_mut()
                        .set_door_direction_flags(direction);
                }
            }
            B::Gravestone => {
                self.record_tile_result(R::KeyLockGravestone, bits);
                self.record_tile_result(R::Collision, bits);
            }
        }
    }

    fn record_tile_result(&mut self, result: TileResult, bits: u16) {
        self.tile_detect_position_mut()
            .accumulate_result(result, bits);
    }

    fn record_solid_tile_interaction(&mut self, bits: u16, misc_shift: u32) {
        self.record_tile_result(TileResult::Collision, bits);
        self.record_tile_result(TileResult::Misc, bits << misc_shift);
    }

    fn tile_hazards_enabled(&self) -> bool {
        !self.game_state.player.follower_link.is_menu_blocked()
            && self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0
    }
}
