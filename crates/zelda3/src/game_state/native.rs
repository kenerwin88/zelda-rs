//! Native game-state model.
//!
//! Byte-backed views remain the compatibility surface while native state is
//! proven subsystem by subsystem. Native structs own domain fields and can be
//! projected to or loaded from WRAM during the transition.

mod display;
mod frame;
mod world;

pub(crate) use display::{
    DisplayState, LinkDmaSourceSlot, NativeDisplayStateBridgeMut, NativeVramUploadBufferBridgeMut,
};
pub(crate) use frame::{FrameState, NativeFrameStateBridgeMut};
pub(crate) use world::{
    NativeOverworldEntranceBridgeMut, NativeOverworldExitBridgeMut, NativeOverworldMap16BridgeMut,
    NativeOverworldMapUiBridgeMut, NativeOverworldMapZoomBridgeMut,
    NativeOverworldScreenSizeBridgeMut, NativeOverworldTransitionBridgeMut,
    NativeWorldLocationBridgeMut, OverworldMap16State, WorldLocationState, WorldState,
};
pub use world::{OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState};

#[cfg(test)]
use crate::game_state::constants::*;
#[cfg(test)]
use crate::types::{read_le_u16, write_le_u16};
#[cfg(test)]
use world::{
    OverworldEntranceState, OverworldExitState, OverworldMapUiState, OverworldMapZoomState,
    OverworldScreenSizeState, OverworldTransitionState,
};

fn ram_byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GameState {
    pub(crate) frame: FrameState,
    pub(crate) world: WorldState,
    pub(crate) display: DisplayState,
}

impl GameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            frame: FrameState::load_from_ram(ram),
            world: WorldState::load_from_ram(ram),
            display: DisplayState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.frame.write_to_ram(ram);
        self.world.write_to_ram(ram);
        self.display.write_to_ram(ram);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::WRAM_SIZE;

    #[test]
    fn frame_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 7;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 9;
        ram[FRAME_COUNTER] = 0x42;
        ram[SAVED_MODULE_FOR_MENU] = 5;
        ram[MODAL_PAUSE_FLAG] = 1;

        let mut frame = FrameState::load_from_ram(&ram);
        assert_eq!(frame.main_module, 7);
        assert_eq!(frame.main_module_word(), 0x0207);
        assert_eq!(frame.submodule, 2);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 0x42);
        assert_eq!(frame.saved_module_for_menu, 5);
        assert_eq!(frame.modal_pause_flag, 1);

        frame.main_module = 14;
        frame.submodule = 3;
        frame.subsubmodule = 1;
        frame.frame_counter = 0x80;
        frame.saved_module_for_menu = 7;
        frame.modal_pause_flag = 2;
        frame.write_to_ram(&mut ram);

        assert_eq!(ram[MAIN_MODULE], 14);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 1);
        assert_eq!(ram[FRAME_COUNTER], 0x80);
        assert_eq!(ram[SAVED_MODULE_FOR_MENU], 7);
        assert_eq!(ram[MODAL_PAUSE_FLAG], 2);
    }

    #[test]
    fn native_frame_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 1;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 3;
        ram[FRAME_COUNTER] = 4;
        ram[SAVED_MODULE_FOR_MENU] = 8;
        ram[MODAL_PAUSE_FLAG] = 1;

        let mut frame = FrameState::default();
        {
            let mut bridge = NativeFrameStateBridgeMut::new(&mut frame, &mut ram);
            bridge.increment_submodule();
            bridge.set_subsubmodule(9);
            bridge.increment_frame_counter();
            bridge.save_main_module_for_menu();
            bridge.clear_saved_module_for_menu();
            bridge.save_submodule_for_menu();
            bridge.clear_modal_pause_flag();
            bridge.increment_modal_pause_flag();
            bridge.set_modal_pause_flag(6);
        }

        assert_eq!(frame.main_module, 1);
        assert_eq!(frame.submodule, 3);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 5);
        assert_eq!(frame.saved_module_for_menu, 3);
        assert_eq!(frame.modal_pause_flag, 6);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 9);
        assert_eq!(ram[FRAME_COUNTER], 5);
        assert_eq!(ram[SAVED_MODULE_FOR_MENU], 3);
        assert_eq!(ram[MODAL_PAUSE_FLAG], 6);
    }

    #[test]
    fn world_location_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::load_from_ram(&ram);
        assert_eq!(world.dungeon_room, 0x0124);
        assert_eq!(world.dungeon_room_index(), 0x24);
        assert_eq!(world.overworld_screen, 0x0040);
        assert_eq!(world.overworld_screen_index(), 0x40);
        assert!(world.is_indoors());
        assert!(!world.is_outdoors());

        world.dungeon_room = 0x0181;
        world.overworld_screen = 0x005b;
        world.indoor_flag = 0;
        world.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }

    #[test]
    fn native_world_location_mut_view_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::default();
        {
            let mut bridge = NativeWorldLocationBridgeMut::new(&mut world, &mut ram);
            bridge.increment_dungeon_room_index_by(2);
            bridge.set_overworld_screen(0x5b);
            bridge.set_indoor_flag(0);
        }

        assert_eq!(world.dungeon_room, 0x0126);
        assert_eq!(world.overworld_screen, 0x005b);
        assert_eq!(world.indoor_flag, 0);
        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0126);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }

    #[test]
    fn game_state_loads_grouped_world_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0206);
        ram[OVERWORLD_MAP_FLAGS] = 0x03;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0004);
        ram[MODE7_ZOOM_STEP_COUNTER] = 2;
        ram[TIMER_FOR_MODE7_ZOOM] = 12;
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        ram[TRIGGER_SPECIAL_ENTRANCE] = 1;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 3;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0022);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0033);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0004);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0008);
        ram[OVERWORLD_TRANSITION_DIR] = 2;

        let mut state = GameState::load_from_ram(&ram);
        assert_eq!(state.world.location.dungeon_room, 0x0124);
        assert_eq!(state.world.location.overworld_screen, 0x0040);
        assert_eq!(state.world.location.indoor_flag, 1);
        assert_eq!(state.world.overworld.map_ui.map_state_word(), 0x0206);
        assert_eq!(state.world.overworld.map_zoom.timer, 12);
        assert_eq!(state.world.overworld.screen_size.is_big_area_word(), 0x0120);
        assert!(state.world.overworld.screen_size.is_big_area());
        assert_eq!(
            state.world.overworld.screen_size.right_bottom_bound_word(),
            0x03e4
        );
        assert_eq!(state.world.overworld.map16.active_load.src_off, 0x1234);
        assert_eq!(state.world.overworld.entrance.sequence_counter, 3);
        assert_eq!(state.world.overworld.exit.special_exit_screen, 0x0033);
        assert_eq!(
            state.world.overworld.transition.direction_bits_word(),
            0x0008
        );

        state.world.location.dungeon_room = 0x0181;
        state.world.location.overworld_screen = 0x005b;
        state.world.location.indoor_flag = 0;
        state.world.overworld.map_ui.map_flags = 0x81;
        state.world.overworld.map_zoom.timer = 4;
        state.world.overworld.screen_size.big_area = 0x0020;
        state.world.overworld.screen_size.big_area_backup = 0x20;
        state.world.overworld.screen_size.right_bottom_scroll_bound = 0x01e4;
        state.world.overworld.map16.active_load.src_off = 0x4567;
        state.world.overworld.entrance.sequence_counter = 9;
        state.world.overworld.exit.exit_screen = 0x0044;
        state.world.overworld.transition.direction_enum = 3;
        state.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x81);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x4567);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 9);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0044);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 3);
    }

    #[test]
    fn overworld_map_ui_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
        ram[OVERWORLD_MAP_FLAGS] = 0x81;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);

        let mut map_ui = OverworldMapUiState::load_from_ram(&ram);
        assert_eq!(map_ui.map_state(), 5);
        assert_eq!(map_ui.map_state_word(), 0x0205);
        assert_eq!(map_ui.map_flags, 0x81);
        assert_eq!(map_ui.birdtravel_status(), 7);
        assert_eq!(map_ui.birdtravel_status_word(), 0x0307);

        map_ui.map_state = 0x0104;
        map_ui.map_flags = 0x40;
        map_ui.birdtravel_status = 0x0008;
        map_ui.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0104);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x40);
        assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0008);
    }

    #[test]
    fn native_overworld_map_ui_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
        ram[OVERWORLD_MAP_FLAGS] = 0x81;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);

        let mut map_ui = OverworldMapUiState::default();
        {
            let mut bridge = NativeOverworldMapUiBridgeMut::new(&mut map_ui, &mut ram);
            bridge.increment_map_state();
            bridge.and_map_flags(!0x80);
            bridge.or_map_flags(0x02);
            bridge.increment_birdtravel_status();
            bridge.and_birdtravel_status(7);
            bridge.set_birdtravel_status_word(0x0004);
        }

        assert_eq!(map_ui.map_state_word(), 0x0206);
        assert_eq!(map_ui.map_flags, 0x03);
        assert_eq!(map_ui.birdtravel_status_word(), 0x0004);
        assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0206);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x03);
        assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0004);
    }

    #[test]
    fn overworld_map_zoom_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MODE7_ZOOM_STEP_COUNTER] = 4;
        ram[TIMER_FOR_MODE7_ZOOM] = 12;

        let mut zoom = OverworldMapZoomState::load_from_ram(&ram);
        assert_eq!(zoom.step_counter, 4);
        assert_eq!(zoom.timer, 12);

        zoom.step_counter = 7;
        zoom.timer = 33;
        zoom.write_to_ram(&mut ram);

        assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 7);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 33);
    }

    #[test]
    fn native_overworld_map_zoom_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MODE7_ZOOM_STEP_COUNTER] = 2;
        ram[TIMER_FOR_MODE7_ZOOM] = 1;

        let mut zoom = OverworldMapZoomState::default();
        {
            let mut bridge = NativeOverworldMapZoomBridgeMut::new(&mut zoom, &mut ram);
            bridge.set_step_counter(4);
            bridge.decrement_timer();
            bridge.set_timer(12);
        }

        assert_eq!(zoom.step_counter, 4);
        assert_eq!(zoom.timer, 12);
        assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 4);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 12);
    }

    #[test]
    fn overworld_screen_size_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);

        let mut screen_size = OverworldScreenSizeState::load_from_ram(&ram);
        assert_eq!(screen_size.is_big_area_word(), 0x0120);
        assert!(screen_size.is_big_area());
        assert_eq!(screen_size.big_area_backup, 0x20);
        assert_eq!(screen_size.right_bottom_bound_word(), 0x03e4);

        screen_size.big_area = 0x0020;
        screen_size.big_area_backup = 0x20;
        screen_size.right_bottom_scroll_bound = 0x01e4;
        screen_size.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
    }

    #[test]
    fn native_overworld_screen_size_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x11;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x02c0);

        let mut screen_size = OverworldScreenSizeState::default();
        {
            let mut bridge = NativeOverworldScreenSizeBridgeMut::new(&mut screen_size, &mut ram);
            bridge.backup_big_area_low();
            bridge.clear_big_area_high();
            bridge.set_big_area_low(0x20);
            bridge.set_right_bottom_bound_low(0xe4);
            bridge.set_right_bottom_bound_high(0x01);
        }

        assert_eq!(screen_size.big_area, 0x0020);
        assert_eq!(screen_size.big_area_backup, 0x20);
        assert_eq!(screen_size.right_bottom_scroll_bound, 0x01e4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
    }

    #[test]
    fn overworld_map16_load_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
        write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

        let mut map16 = OverworldMap16State::load_from_ram(&ram);
        assert_eq!(map16.active_load.src_off, 0x1234);
        assert_eq!(map16.active_load.dst_off, 0x0056);
        assert_eq!(map16.active_load.y_unit, 0x0007);
        assert_eq!(map16.previous_load.src_off, 0x2345);
        assert_eq!(map16.previous_load.dst_off, 0x0067);
        assert_eq!(map16.previous_load.y_unit, 0x0008);
        assert_eq!(map16.special_exit_src_off, 0x3456);
        assert_eq!(map16.exit_src_off, 0x4567);
        assert_eq!(map16.small_scroll_backup.src_off, 0x5678);
        assert_eq!(map16.small_scroll_backup.dst_off, 0x0079);
        assert_eq!(map16.small_scroll_backup.y_unit, 0x000a);

        map16.active_load.src_off = 0x2222;
        map16.active_load.dst_off = 0x0034;
        map16.active_load.y_unit = 0x0009;
        map16.previous_load.src_off = 0x3333;
        map16.previous_load.dst_off = 0x0045;
        map16.previous_load.y_unit = 0x000b;
        map16.special_exit_src_off = 0x4444;
        map16.exit_src_off = 0x5555;
        map16.small_scroll_backup = SmallOverworldMap16ScrollBackupState {
            src_off: 0x6666,
            dst_off: 0x0056,
            y_unit: 0x000c,
        };
        map16.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x2222);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0034);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x0009);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x3333);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0045);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x4444);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x5555);
        assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x6666);
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            0x0056
        );
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
            0x000c
        );
    }

    #[test]
    fn native_overworld_map16_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
        write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

        let mut map16 = OverworldMap16State::default();
        {
            let mut bridge = NativeOverworldMap16BridgeMut::new(&mut map16, &mut ram);
            bridge.set_active_load(OverworldMap16LoadState {
                src_off: 0x3456,
                dst_off: 0x0078,
                y_unit: 0x000a,
            });
            bridge.set_previous_load(OverworldMap16LoadState {
                src_off: 0x4567,
                dst_off: 0x0089,
                y_unit: 0x000b,
            });
            bridge.set_special_exit_src_off(0x5678);
            bridge.set_exit_src_off(0x6789);
            bridge.set_small_scroll_backup(SmallOverworldMap16ScrollBackupState {
                src_off: 0x789a,
                dst_off: 0x009b,
                y_unit: 0x000c,
            });
        }

        assert_eq!(map16.active_load.src_off, 0x3456);
        assert_eq!(map16.active_load.dst_off, 0x0078);
        assert_eq!(map16.active_load.y_unit, 0x000a);
        assert_eq!(map16.previous_load.src_off, 0x4567);
        assert_eq!(map16.previous_load.dst_off, 0x0089);
        assert_eq!(map16.previous_load.y_unit, 0x000b);
        assert_eq!(map16.special_exit_src_off, 0x5678);
        assert_eq!(map16.exit_src_off, 0x6789);
        assert_eq!(map16.small_scroll_backup.src_off, 0x789a);
        assert_eq!(map16.small_scroll_backup.dst_off, 0x009b);
        assert_eq!(map16.small_scroll_backup.y_unit, 0x000c);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x3456);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0078);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x000a);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x4567);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0089);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x5678);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x6789);
        assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x789a);
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            0x009b
        );
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
            0x000c
        );
    }

    #[test]
    fn overworld_entrance_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 9;

        let mut entrance = OverworldEntranceState::load_from_ram(&ram);
        assert_eq!(entrance.special_entrance_trigger, 5);
        assert_eq!(entrance.sequence_counter, 9);

        entrance.special_entrance_trigger = 2;
        entrance.sequence_counter = 7;
        entrance.write_to_ram(&mut ram);

        assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 2);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 7);
    }

    #[test]
    fn native_overworld_entrance_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 0xff;

        let mut entrance = OverworldEntranceState::default();
        {
            let mut bridge = NativeOverworldEntranceBridgeMut::new(&mut entrance, &mut ram);
            bridge.set_special_entrance_trigger(3);
            assert_eq!(bridge.increment_sequence_counter(), 0);
            assert_eq!(bridge.decrement_sequence_counter(), 0xff);
            bridge.clear_special_entrance_trigger();
            bridge.clear_sequence_counter();
        }

        assert_eq!(entrance.special_entrance_trigger, 0);
        assert_eq!(entrance.sequence_counter, 0);
        assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 0);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 0);
    }

    #[test]
    fn overworld_exit_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0123);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0045);

        let mut exit = OverworldExitState::load_from_ram(&ram);
        assert_eq!(exit.exit_screen, 0x0123);
        assert_eq!(exit.special_exit_screen, 0x0045);

        exit.exit_screen = 0x0067;
        exit.special_exit_screen = 0x0089;
        exit.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0067);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0089);
    }

    #[test]
    fn native_overworld_exit_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0111);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0222);

        let mut exit = OverworldExitState::default();
        {
            let mut bridge = NativeOverworldExitBridgeMut::new(&mut exit, &mut ram);
            bridge.set_exit_screen(0x0033);
            bridge.set_special_exit_screen(0x0044);
        }

        assert_eq!(exit.exit_screen, 0x0033);
        assert_eq!(exit.special_exit_screen, 0x0044);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0033);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0044);
    }

    #[test]
    fn overworld_transition_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0302);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
        ram[OVERWORLD_TRANSITION_DIR] = 6;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
        ram[TRANSITION_COUNTER] = 9;
        ram[OW_COUNTDOWN_TRANSITION] = 12;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV, 0x0004);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV, 0x0002);
        ram[OVERWORLD_SCREEN_TRANSITION_PREV] = 7;

        let mut transition = OverworldTransitionState::load_from_ram(&ram);
        assert_eq!(transition.edge_direction_bits(), 2);
        assert_eq!(transition.edge_direction_bits, 0x0302);
        assert_eq!(transition.direction_bits(), 8);
        assert_eq!(transition.direction_bits_word(), 0x0108);
        assert_eq!(transition.direction_enum(), 6);
        assert!(transition.has_direction_bits());
        assert_eq!(transition.screen_transition(), 3);
        assert_eq!(transition.screen_transition_word(), 0x0203);
        assert_eq!(transition.transition_counter, 9);
        assert_eq!(transition.countdown(), 12);
        assert_eq!(transition.previous_direction_bits, 4);
        assert_eq!(transition.previous_direction_bits2, 2);
        assert_eq!(transition.previous_screen_transition, 7);

        transition.edge_direction_bits = 0x0003;
        transition.direction_bits = 0x0001;
        transition.direction_enum = 4;
        transition.screen_transition = 0x0002;
        transition.transition_counter = 5;
        transition.countdown = 11;
        transition.previous_direction_bits = 0x0008;
        transition.previous_direction_bits2 = 0x0004;
        transition.previous_screen_transition = 6;
        transition.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 3);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 1);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 2);
        assert_eq!(ram[TRANSITION_COUNTER], 5);
        assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 11);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 8);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV), 4);
        assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
    }

    #[test]
    fn native_overworld_transition_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0102);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
        ram[OVERWORLD_TRANSITION_DIR] = 6;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
        ram[TRANSITION_COUNTER] = 9;
        ram[OW_COUNTDOWN_TRANSITION] = 1;

        let mut transition = OverworldTransitionState::default();
        {
            let mut bridge = NativeOverworldTransitionBridgeMut::new(&mut transition, &mut ram);
            bridge.and_direction_bits(0x0b);
            bridge.or_direction_bits(0x04);
            assert_eq!(bridge.or_direction_bits_word(0x0100), 0x010c);
            bridge.set_direction_enum(4);
            bridge.set_screen_transition(5);
            bridge.increment_transition_counter();
            assert_eq!(bridge.decrement_countdown(), 0);
            bridge.set_countdown(12);
            bridge.save_previous_direction_bits();
            bridge.set_edge_direction_bits(0x04);
            bridge.clear_direction_bits_word();
            bridge.restore_previous_direction_bits();
            bridge.set_previous_screen_transition(6);
        }

        assert_eq!(transition.edge_direction_bits(), 2);
        assert_eq!(transition.edge_direction_bits, 2);
        assert_eq!(transition.direction_bits_word(), 0x010c);
        assert_eq!(transition.direction_enum(), 4);
        assert_eq!(transition.screen_transition_word(), 0x0205);
        assert_eq!(transition.transition_counter, 10);
        assert_eq!(transition.countdown(), 12);
        assert_eq!(transition.previous_direction_bits, 2);
        assert_eq!(transition.previous_direction_bits2, 0x010c);
        assert_eq!(transition.previous_screen_transition, 6);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 2);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 0x010c);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 0x0205);
        assert_eq!(ram[TRANSITION_COUNTER], 10);
        assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 12);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 2);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV),
            0x010c
        );
        assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
    }

    #[test]
    fn display_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INIDISP_COPY] = 0x0f;
        ram[NMI_BOOLEAN] = 1;
        ram[NMI_DISABLE_CORE_UPDATES] = 4;
        ram[NMI_SUBROUTINE_INDEX] = 11;
        ram[NMI_LOAD_BG_FROM_VRAM] = 3;
        ram[NMI_UPDATE_TILEMAP_DST] = 0x50;
        write_le_u16(&mut ram, NMI_UPDATE_TILEMAP_SRC, 0x0200);
        ram[BGMODE_COPY] = 7;
        ram[TM_COPY] = 0x16;
        ram[TS_COPY] = 0x01;
        ram[W12SEL_COPY] = 0x33;
        ram[W34SEL_COPY] = 3;
        ram[WOBJSEL_COPY] = 0xb0;
        ram[TMW_COPY] = 0x16;
        ram[TSW_COPY] = 1;
        ram[NMI_COPY_PACKETS_FLAG] = 1;
        ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0xff;
        ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 9;
        ram[NMI_THREAD_ACTIVE] = 1;
        write_le_u16(&mut ram, POLY_THREAD_STACK, 0x01f2);
        ram[IRQ_FLAG] = 0x80;
        ram[VIRQ_TRIGGER] = 0x90;
        ram[DMA_HEAD_POINTER] = 0x20;
        ram[DMA_BODY_POINTER] = 0xa0;
        ram[OAM_BUF] = 0xca;
        ram[OAM_BUF + 1] = 0xfe;
        ram[HDMAEN_COPY] = 0xc0;
        ram[MOSAIC_COPY] = 0x73;
        ram[MOSAIC_LEVEL] = 0x70;
        ram[MOSAIC_TARGET_LEVEL] = 0x1f;
        ram[MOSAIC_INC_OR_DEC] = 1;
        write_le_u16(&mut ram, NMI_LOAD_TARGET_ADDR, 0x2146);
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0124);
        ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF] = 0xfa;
        ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 1] = 0xce;
        write_le_u16(
            &mut ram,
            crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + 6,
            0x4567,
        );
        ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B] = 0x56;
        ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B + 1] = 0x78;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER] = 0x9a;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER + 1] = 0xbc;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1] = 0xde;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1 + 1] = 0xf0;
        ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER] = 0x13;
        ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER + 1] = 0x57;
        ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER] = 0x24;
        ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER + 1] = 0x68;
        ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER] = 0xac;
        ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER + 1] = 0xe0;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER] = 0x31;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER + 1] = 0x42;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER] = 0x53;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER + 1] = 0x64;
        ram[POLYHEDRAL_BUFFER] = 0x75;
        ram[POLYHEDRAL_BUFFER + 1] = 0x86;
        write_le_u16(
            &mut ram,
            crate::game_state::constants::nmi::ARBITRARY_TILEMAP_DST_BUFFER + 4,
            0x789a,
        );
        ram[DUNGEON_BG2_ATTR_TABLE] = 0xa5;
        ram[DUNGEON_BG2_ATTR_TABLE + 1] = 0x5a;
        ram[DUNGEON_BG1_ATTR_TABLE] = 0xc3;
        ram[DUNGEON_BG1_ATTR_TABLE + 1] = 0x3c;
        ram[0x4567] = 0x81;
        ram[0x4568] = 0x18;
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_DST_ADDR, 0x6040);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_BASE, 0x4841);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_LIMIT, 0x007f);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_SENTINEL, 0xffff);
        ram[HUD_TILE_INDICES_BUFFER] = 0xbe;
        ram[HUD_TILE_INDICES_BUFFER + 1] = 0xef;
        ram[STAR_TILE_RESTORE_PHASE] = 1;
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);
        ram[0xa680] = 0xde;
        ram[0xa681] = 0xad;

        let mut display = DisplayState::load_from_ram(&ram);
        assert_eq!(display.screen_brightness, 0x0f);
        assert_eq!(display.nmi_update_latch, 1);
        assert!(display.nmi_update_is_latched());
        assert_eq!(display.core_update_disable_flag, 4);
        assert!(display.core_updates_are_disabled());
        assert_eq!(display.pending_nmi_subroutine, 11);
        assert_eq!(display.bg_vram_load_mode, 3);
        assert!(display.has_bg_vram_load());
        assert_eq!(display.pending_tilemap_update_destination_page, 0x50);
        assert!(display.has_pending_tilemap_update());
        assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5000);
        assert_eq!(display.pending_tilemap_update_source_offset, 0x0200);
        assert_eq!(
            display.pending_tilemap_update_source_address(),
            crate::game_state::constants::nmi::BG_CHAR_BUFFER + 0x0200
        );
        assert_eq!(display.bg_mode, 7);
        assert_eq!(display.main_screen_layers, 0x16);
        assert_eq!(display.sub_screen_layers, 0x01);
        assert_eq!(display.layer_masks_word(), 0x0116);
        assert_eq!(display.bg12_window_selection, 0x33);
        assert_eq!(display.bg34_window_selection, 3);
        assert_eq!(display.object_color_window_selection, 0xb0);
        assert_eq!(display.main_screen_window_layers, 0x16);
        assert_eq!(display.sub_screen_window_layers, 1);
        assert_eq!(display.nmi_copy_packets_request, 1);
        assert!(display.has_nmi_copy_packets_request());
        assert_eq!(display.pending_polyhedral_update, 0xff);
        assert!(display.has_pending_polyhedral_update());
        assert_eq!(display.chr_halfslot_request, 9);
        assert!(display.has_chr_halfslot_request());
        assert!(display.nmi_thread_active);
        assert_eq!(display.nmi_thread_stack_pointer, 0x01f2);
        assert!(display.nmi_thread_uses_poly_stack());
        assert_eq!(display.irq_control_flag, 0x80);
        assert!(display.has_irq_control_flag());
        assert!(display.irq_control_has_vcounter_marker());
        assert_eq!(display.vertical_irq_trigger, 0x90);
        assert_eq!(display.sprite_dma_head_pointer, 0x20);
        assert_eq!(display.sprite_dma_body_pointer, 0xa0);
        assert_eq!(&display.sprite_oam_shadow_buffer(&ram)[..2], &[0xca, 0xfe]);
        assert_eq!(display.hdma_enable_mask, 0xc0);
        assert!(display.is_hdma_channel_enabled(6));
        assert!(display.is_hdma_channel_enabled(7));
        assert!(!display.is_hdma_channel_enabled(5));
        assert_eq!(display.mosaic_copy, 0x73);
        assert_eq!(display.mosaic_level, 0x70);
        assert_eq!(display.mosaic_target_level, 0x1f);
        assert_eq!(display.mosaic_target_level_word(), 0x1f);
        assert_eq!(display.mosaic_direction, 1);
        assert_eq!(display.nmi_load_target_address, 0x2146);
        assert_eq!(display.nmi_load_target_page(), 0x46);
        assert_eq!(display.vram_upload_cursor, 0x0124);
        assert_eq!(display.vram_upload_cursor_usize(), 0x0124);
        assert_eq!(
            display.current_vram_upload_data_address(),
            VRAM_UPLOAD_DATA + 0x0124
        );
        assert_eq!(&display.nmi_vram_packet_buffer(&ram)[..2], &[0xfa, 0xce]);
        assert_eq!(display.overworld_tile_upload_word(&ram, 0), 0xcefa);
        assert_eq!(display.overworld_tile_attribute_word(&ram, 3), 0x4567);
        assert_eq!(
            &display.tilemap_upload_stripe_buffer(&ram)[..2],
            &[0x24, 0x01]
        );
        assert_eq!(
            &display.secondary_stripe_upload_buffer(&ram)[..2],
            &[0x56, 0x78]
        );
        assert_eq!(
            &display.background_character_buffer(&ram)[..2],
            &[0x9a, 0xbc]
        );
        assert_eq!(
            &display.background_character_secondary_buffer(&ram)[..2],
            &[0xde, 0xf0]
        );
        assert_eq!(
            &display.background_character_half_buffer(&ram)[..2],
            &[0x13, 0x57]
        );
        assert_eq!(
            &display.bg1_wall_top_tilemap_buffer(&ram)[..2],
            &[0x24, 0x68]
        );
        assert_eq!(
            &display.bg1_wall_bottom_tilemap_buffer(&ram)[..2],
            &[0xac, 0xe0]
        );
        assert_eq!(
            &display.game_over_text_tile_buffer(&ram)[..2],
            &[0x31, 0x42]
        );
        assert_eq!(
            &display.game_over_text_tail_tile_buffer(&ram)[..2],
            &[0x53, 0x64]
        );
        assert_eq!(&display.polyhedral_tile_buffer(&ram)[..2], &[0x75, 0x86]);
        assert_eq!(display.arbitrary_tilemap_destination(&ram, 2), 0x789a);
        assert_eq!(
            &display.dungeon_bg2_attribute_table(&ram)[..2],
            &[0xa5, 0x5a]
        );
        assert_eq!(
            &display.dungeon_bg1_attribute_table(&ram)[..2],
            &[0xc3, 0x3c]
        );
        assert_eq!(
            display.vram_dma_source_bytes(&ram, 0x4567, 2),
            &[0x81, 0x18]
        );
        assert_eq!(display.message_dma_destination_address, 0x6040);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(&display.message_dma_tile_indices(&ram)[..2], &[0xbe, 0xef]);
        assert_eq!(display.star_tile_restore_phase, 1);
        assert_eq!(display.star_tile_restore_source_offsets(), (32, 0));
        assert_eq!(display.animated_tile_data_source_address, 0xa680);
        assert_eq!(display.animated_tile_data_source_usize(), 0xa680);
        assert_eq!(&display.animated_tile_data(&ram)[..2], &[0xde, 0xad]);
        assert!(display.has_animated_tile_data_source());
        assert_eq!(display.animated_tile_vram_destination_address, 0x3b00);
        assert_eq!(display.animated_tile_vram_destination_usize(), 0x3b00);

        display.screen_brightness = 0x80;
        display.nmi_update_latch = 0;
        display.core_update_disable_flag = 0;
        display.pending_nmi_subroutine = 0;
        display.bg_vram_load_mode = 0;
        display.pending_tilemap_update_destination_page = 0x40;
        display.pending_tilemap_update_source_offset = 0x0600;
        display.bg_mode = 9;
        display.main_screen_layers = 0x11;
        display.sub_screen_layers = 0;
        display.bg12_window_selection = 0;
        display.bg34_window_selection = 0;
        display.object_color_window_selection = 0x30;
        display.main_screen_window_layers = 3;
        display.sub_screen_window_layers = 0;
        display.nmi_copy_packets_request = 0;
        display.pending_polyhedral_update = 0;
        display.chr_halfslot_request = 0;
        display.nmi_thread_active = false;
        display.nmi_thread_stack_pointer = 0x1f31;
        display.irq_control_flag = 0;
        display.vertical_irq_trigger = 0x70;
        display.sprite_dma_head_pointer = 0x40;
        display.sprite_dma_body_pointer = 0x80;
        display.hdma_enable_mask = 0x80;
        display.mosaic_copy = 3;
        display.mosaic_level = 0x20;
        display.mosaic_target_level = 0;
        display.mosaic_direction = 0;
        display.nmi_load_target_address = 0x0080;
        display.vram_upload_cursor = 0x0042;
        display.message_dma_destination_address = 0x6080;
        display.message_dma_tile_base = 0x4842;
        display.message_dma_tile_limit = 0x0080;
        display.message_dma_tile_sentinel = 0xfffe;
        display.star_tile_restore_phase = 0;
        display.animated_tile_data_source_address = 0xac80;
        display.animated_tile_vram_destination_address = 0x3c00;
        display.write_to_ram(&mut ram);

        assert_eq!(ram[INIDISP_COPY], 0x80);
        assert_eq!(ram[NMI_BOOLEAN], 0);
        assert_eq!(ram[NMI_DISABLE_CORE_UPDATES], 0);
        assert_eq!(ram[NMI_SUBROUTINE_INDEX], 0);
        assert_eq!(ram[NMI_LOAD_BG_FROM_VRAM], 0);
        assert_eq!(ram[NMI_UPDATE_TILEMAP_DST], 0x40);
        assert_eq!(read_le_u16(&ram, NMI_UPDATE_TILEMAP_SRC), 0x0600);
        assert_eq!(ram[BGMODE_COPY], 9);
        assert_eq!(ram[TM_COPY], 0x11);
        assert_eq!(ram[TS_COPY], 0);
        assert_eq!(ram[W12SEL_COPY], 0);
        assert_eq!(ram[W34SEL_COPY], 0);
        assert_eq!(ram[WOBJSEL_COPY], 0x30);
        assert_eq!(ram[TMW_COPY], 3);
        assert_eq!(ram[TSW_COPY], 0);
        assert_eq!(ram[NMI_COPY_PACKETS_FLAG], 0);
        assert_eq!(read_le_u16(&ram, messaging::MESSAGE_DMA_DST_ADDR), 0x6080);
        assert_eq!(read_le_u16(&ram, messaging::MESSAGE_DMA_TILE_BASE), 0x4842);
        assert_eq!(read_le_u16(&ram, messaging::MESSAGE_DMA_TILE_LIMIT), 0x0080);
        assert_eq!(
            read_le_u16(&ram, messaging::MESSAGE_DMA_TILE_SENTINEL),
            0xfffe
        );
        assert_eq!(ram[NMI_FLAG_UPDATE_POLYHEDRAL], 0);
        assert_eq!(ram[LOAD_CHR_HALFSLOT_EVEN_ODD], 0);
        assert_eq!(ram[NMI_THREAD_ACTIVE], 0);
        assert_eq!(read_le_u16(&ram, POLY_THREAD_STACK), 0x1f31);
        assert_eq!(ram[IRQ_FLAG], 0);
        assert_eq!(ram[VIRQ_TRIGGER], 0x70);
        assert_eq!(ram[DMA_HEAD_POINTER], 0x40);
        assert_eq!(ram[DMA_BODY_POINTER], 0x80);
        assert_eq!(ram[HDMAEN_COPY], 0x80);
        assert_eq!(ram[MOSAIC_COPY], 3);
        assert_eq!(ram[MOSAIC_LEVEL], 0x20);
        assert_eq!(ram[MOSAIC_TARGET_LEVEL], 0);
        assert_eq!(ram[MOSAIC_INC_OR_DEC], 0);
        assert_eq!(read_le_u16(&ram, NMI_LOAD_TARGET_ADDR), 0x0080);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0042);
        assert_eq!(ram[STAR_TILE_RESTORE_PHASE], 0);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    }

    #[test]
    fn native_vram_upload_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeVramUploadBufferBridgeMut::new(&mut display, &mut ram);
            bridge.advance_offset_by(0x20);
            bridge.clear_offset();
            bridge.set_offset(0x0034);
            bridge.write_buffer_byte(40, 0xaa);
            bridge.write_buffer_word(42, 0xbbcc);
            bridge.write_tilemap_word(80, 0x1234);
            bridge.write_overworld_vram_word(3, 0x5678);
            bridge.write_absolute_byte(0x2000, 0xdd);
            bridge.write_absolute_word(0x2002, 0xeeff);
            bridge.copy_buffer_bytes(44, &[1, 2, 3, 4]);
            bridge.terminate_buffer_at(48);
            bridge.write_level_label_tiles(&[0x11; 14], &[0x22; 14]);
            bridge.write_map16_update_packet(0x2100, 0x1234, [0x1000, 0x1001, 0x1002, 0x1003]);
            bridge.write_single_tile_stripe_packet(0x2120, 0x3456, 0x2000);
            bridge.write_tile_stripe_sentinel(0x2130);
        }

        assert_eq!(display.vram_upload_cursor, 0x0034);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0034);
        assert_eq!(ram[VRAM_UPLOAD_DATA], 0x11);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 13], 0x11);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 16], 0x22);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 29], 0x22);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 32], 0xff);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 40], 0xaa);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_DATA + 42), 0xbbcc);
        assert_eq!(
            &ram[VRAM_UPLOAD_DATA + 44..VRAM_UPLOAD_DATA + 48],
            &[1, 2, 3, 4]
        );
        assert_eq!(ram[VRAM_UPLOAD_DATA + 48], 0xff);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET + 80), 0x1234);
        assert_eq!(read_le_u16(&ram, UVRAM_DATA + 6), 0x5678);
        assert_eq!(ram[0x2000], 0xdd);
        assert_eq!(read_le_u16(&ram, 0x2002), 0xeeff);
        assert_eq!(read_le_u16(&ram, 0x2100), 0x3412);
        assert_eq!(read_le_u16(&ram, 0x2102), 0x0300);
        assert_eq!(read_le_u16(&ram, 0x2104), 0x1000);
        assert_eq!(read_le_u16(&ram, 0x2106), 0x1001);
        assert_eq!(read_le_u16(&ram, 0x2108), 0x5412);
        assert_eq!(read_le_u16(&ram, 0x210a), 0x0300);
        assert_eq!(read_le_u16(&ram, 0x210c), 0x1002);
        assert_eq!(read_le_u16(&ram, 0x210e), 0x1003);
        assert_eq!(read_le_u16(&ram, 0x2110), 0xffff);
        assert_eq!(read_le_u16(&ram, 0x2120), 0x3456);
        assert_eq!(read_le_u16(&ram, 0x2122), 0x0100);
        assert_eq!(read_le_u16(&ram, 0x2124), 0x2000);
        assert_eq!(read_le_u16(&ram, 0x2130), 0xffff);
    }

    #[test]
    fn link_dma_source_slots_read_named_source_addresses() {
        let mut ram = vec![0; WRAM_SIZE];
        let slots = [
            (LinkDmaSourceSlot::BodyTop, DMA_SOURCE_ADDR_3),
            (LinkDmaSourceSlot::BodyBottom, DMA_SOURCE_ADDR_0),
            (LinkDmaSourceSlot::HeadTop, DMA_SOURCE_ADDR_4),
            (LinkDmaSourceSlot::HeadBottom, DMA_SOURCE_ADDR_1),
            (LinkDmaSourceSlot::HandLeft, DMA_SOURCE_ADDR_5),
            (LinkDmaSourceSlot::HandRight, DMA_SOURCE_ADDR_2),
            (LinkDmaSourceSlot::SwordUpper, DMA_SOURCE_ADDR_6),
            (LinkDmaSourceSlot::SwordLower, DMA_SOURCE_ADDR_11),
            (LinkDmaSourceSlot::ShieldUpper, DMA_SOURCE_ADDR_7),
            (LinkDmaSourceSlot::ShieldLower, DMA_SOURCE_ADDR_12),
            (LinkDmaSourceSlot::AuxUpper, DMA_SOURCE_ADDR_8),
            (LinkDmaSourceSlot::AuxLower, DMA_SOURCE_ADDR_13),
            (LinkDmaSourceSlot::PushUpper, DMA_SOURCE_ADDR_10),
            (LinkDmaSourceSlot::PushLower, DMA_SOURCE_ADDR_15),
            (LinkDmaSourceSlot::AnimatedTileUpper, DMA_SOURCE_ADDR_9),
            (LinkDmaSourceSlot::AnimatedTileLower, DMA_SOURCE_ADDR_14),
            (LinkDmaSourceSlot::HeadPointerUpper, DMA_SOURCE_ADDR_16),
            (LinkDmaSourceSlot::HeadPointerLower, DMA_SOURCE_ADDR_18),
            (LinkDmaSourceSlot::BodyPointerUpper, DMA_SOURCE_ADDR_17),
            (LinkDmaSourceSlot::BodyPointerLower, DMA_SOURCE_ADDR_19),
            (LinkDmaSourceSlot::TravelBirdUpper, DMA_SOURCE_ADDR_20),
            (LinkDmaSourceSlot::TravelBirdLower, DMA_SOURCE_ADDR_21),
        ];

        for (index, (_, address)) in slots.iter().copied().enumerate() {
            write_le_u16(&mut ram, address, 0x9000 + index as u16);
        }

        let display = DisplayState::load_from_ram(&ram);
        for (index, (slot, _)) in slots.iter().copied().enumerate() {
            assert_eq!(display.link_dma_source(slot), 0x9000 + index as u16);
        }

        let mut projected = vec![0; WRAM_SIZE];
        display.write_to_ram(&mut projected);
        for (index, (_, address)) in slots.iter().copied().enumerate() {
            assert_eq!(read_le_u16(&projected, address), 0x9000 + index as u16);
        }
    }

    #[test]
    fn native_display_bridge_syncs_seeded_ram_and_dual_writes_brightness() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INIDISP_COPY] = 4;
        ram[NMI_BOOLEAN] = 1;
        ram[NMI_DISABLE_CORE_UPDATES] = 2;
        ram[NMI_SUBROUTINE_INDEX] = 6;
        ram[NMI_LOAD_BG_FROM_VRAM] = 2;
        ram[NMI_UPDATE_TILEMAP_DST] = 0x50;
        write_le_u16(&mut ram, NMI_UPDATE_TILEMAP_SRC, 0x0200);
        ram[BGMODE_COPY] = 7;
        ram[TM_COPY] = 0x16;
        ram[TS_COPY] = 0x01;
        ram[W12SEL_COPY] = 0x33;
        ram[W34SEL_COPY] = 3;
        ram[WOBJSEL_COPY] = 0xb0;
        ram[TMW_COPY] = 0x16;
        ram[TSW_COPY] = 1;
        ram[NMI_COPY_PACKETS_FLAG] = 1;
        ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0xff;
        ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 3;
        ram[NMI_THREAD_ACTIVE] = 1;
        write_le_u16(&mut ram, POLY_THREAD_STACK, 0x01f2);
        ram[IRQ_FLAG] = 0x80;
        ram[VIRQ_TRIGGER] = 0x90;
        ram[DMA_HEAD_POINTER] = 0x20;
        ram[DMA_BODY_POINTER] = 0xa0;
        ram[HDMAEN_COPY] = 0xc0;
        ram[MOSAIC_COPY] = 0x73;
        ram[MOSAIC_LEVEL] = 0x70;
        ram[MOSAIC_TARGET_LEVEL] = 0x1f;
        ram[MOSAIC_INC_OR_DEC] = 1;
        write_le_u16(&mut ram, NMI_LOAD_TARGET_ADDR, 0x2146);
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);
        ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0xfe;
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_DST_ADDR, 0x6040);
        ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = 0x20;
        ram[FLAG_TRAVEL_BIRD] = 0x04;
        ram[STAR_TILE_RESTORE_PHASE] = 7;
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeDisplayStateBridgeMut::new(&mut display, &mut ram);
            bridge.increment_screen_brightness();
            bridge.decrement_screen_brightness();
            bridge.set_screen_brightness(0x80);
            bridge.clear_nmi_update_latch();
            bridge.latch_nmi_update();
            bridge.clear_core_update_disable_flag();
            bridge.set_core_update_disable_flag(7);
            assert_eq!(bridge.take_pending_nmi_subroutine(), 6);
            bridge.set_pending_nmi_subroutine(11);
            bridge.clear_bg_vram_load_mode();
            bridge.set_bg_vram_load_mode(5);
            bridge.queue_tilemap_update(0x52, 0x0400);
            bridge.clear_pending_tilemap_update_destination();
            bridge.queue_tilemap_update(0x54, 0x0800);
            bridge.set_bg_mode(9);
            bridge.set_layer_masks_word(0x0116);
            bridge.and_main_screen_layers(0x15);
            bridge.or_main_screen_layers(0x01);
            bridge.and_sub_screen_layers(0x0f);
            bridge.or_sub_screen_layers(0x10);
            bridge.clear_sub_screen_layers_word();
            bridge.set_main_screen_layers(0x11);
            bridge.set_sub_screen_layers(0x02);
            bridge.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            bridge.set_bg12_window_selection(0x11);
            bridge.set_bg34_window_selection(0x22);
            bridge.set_object_color_window_selection(0x30);
            bridge.set_main_screen_window_layers(0x04);
            bridge.set_sub_screen_window_layers(0x05);
            bridge.clear_window_main_sub_masks();
            bridge.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            bridge.clear_nmi_copy_packets_request();
            bridge.request_nmi_copy_packets();
            bridge.set_nmi_copy_packets_request(3);
            bridge.clear_pending_polyhedral_update();
            bridge.request_polyhedral_nmi_update();
            bridge.increment_chr_halfslot_request();
            bridge.clear_chr_halfslot_request();
            bridge.set_chr_halfslot_request(12);
            bridge.deactivate_nmi_thread();
            bridge.activate_nmi_thread();
            bridge.set_nmi_thread_stack_pointer(0x1f31);
            bridge.clear_irq_control_flag();
            bridge.set_irq_control_flag(0xff);
            bridge.set_vertical_irq_trigger(0x70);
            bridge.set_sprite_dma_head_pointer(0x40);
            bridge.set_sprite_dma_body_pointer(0x80);
            bridge.clear_hdma_enable_mask();
            bridge.set_hdma_enable_mask(0x80);
            bridge.set_mosaic_level(0x40);
            assert_eq!(bridge.increment_mosaic_level_by(0x10), 0x50);
            assert_eq!(bridge.decrement_mosaic_level_by(0x20), 0x30);
            bridge.set_mosaic_copy_from_level_or(3);
            bridge.set_mosaic_target_level_word(0x001f);
            bridge.clear_mosaic_target_level_word();
            bridge.set_mosaic_target_level(0x0f);
            bridge.set_mosaic_direction(1);
            bridge.clear_mosaic_direction();
            bridge.set_nmi_load_target_page(0x80);
            bridge.set_nmi_load_target_address(0x1234);
            assert_eq!(bridge.increment_vram_upload_counter(), 0xff);
            assert_eq!(bridge.increment_vram_upload_counter(), 0);
            bridge.reset_incremental_vram_upload_counter();
            bridge.set_link_body_dma_sources(0x9000, 0x9001);
            bridge.set_link_head_dma_sources(0x9002, 0x9003);
            bridge.set_link_hand_dma_sources(0x9004, 0x9005);
            bridge.set_link_sword_dma_sources(0x9006, 0x9007);
            bridge.set_link_shield_dma_sources(0x9008, 0x9009);
            bridge.set_link_aux_dma_sources(0x900a, 0x900b);
            bridge.set_link_push_dma_sources(0x900c, 0x900d);
            bridge.set_link_animated_tile_dma_sources(0x900e, 0x900f);
            bridge.set_link_head_pointer_dma_sources(0x9010, 0x9011);
            bridge.set_link_body_pointer_dma_sources(0x9012, 0x9013);
            bridge.set_travel_bird_dma_sources(0x9014, 0x9015);
            bridge.reset_bg_tile_animation_countdown(0xffff);
            bridge.set_message_dma_destination_address(0x6080);
            bridge.set_message_dma_tile_base(0x4841);
            bridge.set_message_dma_tile_limit(0x007f);
            bridge.set_message_dma_tile_sentinel(0xffff);
            bridge.set_overworld_fixed_color_adjustment(0x30);
            bridge.set_travel_bird_tile_offset(0x08);
            bridge.clear_star_tile_restore_phase();
            bridge.set_animated_tile_data_source_address(0xac80);
            bridge.set_animated_tile_vram_destination_address(0x3c00);
            bridge.set_overworld_tile_attribute_word(7, 0x1234);
            bridge.set_overworld_tile_upload_word(2, 0x5678);
            bridge.terminate_overworld_tile_upload_words(3);
            bridge.copy_tilemap_upload_stripe_bytes(&[0xaa, 0xbb, 0xcc]);
        }

        assert_eq!(display.screen_brightness, 0x80);
        assert_eq!(display.nmi_update_latch, 1);
        assert_eq!(display.core_update_disable_flag, 7);
        assert_eq!(display.pending_nmi_subroutine, 11);
        assert_eq!(display.bg_vram_load_mode, 5);
        assert_eq!(display.pending_tilemap_update_destination_page, 0x54);
        assert!(display.has_pending_tilemap_update());
        assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5400);
        assert_eq!(display.pending_tilemap_update_source_offset, 0x0800);
        assert_eq!(
            display.pending_tilemap_update_source_address(),
            crate::game_state::constants::nmi::BG_CHAR_BUFFER + 0x0800
        );
        assert_eq!(display.bg_mode, 9);
        assert_eq!(display.main_screen_layers, 0x11);
        assert_eq!(display.sub_screen_layers, 0x02);
        assert_eq!(display.layer_masks_word(), 0x0211);
        assert_eq!(display.bg12_window_selection, 0x33);
        assert_eq!(display.bg34_window_selection, 3);
        assert_eq!(display.object_color_window_selection, 0x33);
        assert_eq!(display.main_screen_window_layers, 0x11);
        assert_eq!(display.sub_screen_window_layers, 0x02);
        assert_eq!(display.nmi_copy_packets_request, 3);
        assert_eq!(display.pending_polyhedral_update, 0xff);
        assert!(display.has_pending_polyhedral_update());
        assert_eq!(display.chr_halfslot_request, 12);
        assert!(display.nmi_thread_active);
        assert_eq!(display.nmi_thread_stack_pointer, 0x1f31);
        assert!(!display.nmi_thread_uses_poly_stack());
        assert_eq!(display.irq_control_flag, 0xff);
        assert!(display.irq_control_has_vcounter_marker());
        assert_eq!(display.vertical_irq_trigger, 0x70);
        assert_eq!(display.sprite_dma_head_pointer, 0x40);
        assert_eq!(display.sprite_dma_body_pointer, 0x80);
        assert_eq!(display.hdma_enable_mask, 0x80);
        assert!(display.is_hdma_channel_enabled(7));
        assert!(!display.is_hdma_channel_enabled(6));
        assert_eq!(display.mosaic_level, 0x30);
        assert_eq!(display.mosaic_copy, 0x33);
        assert_eq!(display.mosaic_target_level, 0x0f);
        assert_eq!(display.mosaic_direction, 0);
        assert_eq!(display.nmi_load_target_address, 0x1234);
        assert_eq!(display.vram_upload_cursor, 0xbbaa);
        assert_eq!(display.incremental_vram_upload_counter, 0);
        assert_eq!(display.incremental_vram_upload_counter_usize(), 0);
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::BodyTop), 0x9000);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::BodyBottom),
            0x9001
        );
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::HeadTop), 0x9002);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::HeadBottom),
            0x9003
        );
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::HandLeft), 0x9004);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::HandRight),
            0x9005
        );
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::TravelBirdUpper),
            0x9014
        );
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::TravelBirdLower),
            0x9015
        );
        assert_eq!(display.bg_tile_animation_countdown, 0xffff);
        assert_eq!(display.message_dma_destination_address, 0x6080);
        assert_eq!(display.message_dma_destination_address_usize(), 0x6080);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(display.overworld_fixed_color_adjustment, 0x30);
        assert_eq!(display.travel_bird_tile_offset, 0x08);
        assert!(display.has_travel_bird_tile_upload());
        assert_eq!(display.star_tile_restore_phase, 0);
        assert_eq!(display.star_tile_restore_source_offsets(), (0, 32));
        assert_eq!(display.animated_tile_data_source_address, 0xac80);
        assert_eq!(display.animated_tile_data_source_usize(), 0xac80);
        assert!(display.has_animated_tile_data_source());
        assert_eq!(display.animated_tile_vram_destination_address, 0x3c00);
        assert_eq!(display.animated_tile_vram_destination_usize(), 0x3c00);
        assert_eq!(ram[INIDISP_COPY], 0x80);
        assert_eq!(ram[NMI_BOOLEAN], 1);
        assert_eq!(ram[NMI_DISABLE_CORE_UPDATES], 7);
        assert_eq!(ram[NMI_SUBROUTINE_INDEX], 11);
        assert_eq!(ram[NMI_LOAD_BG_FROM_VRAM], 5);
        assert_eq!(ram[NMI_UPDATE_TILEMAP_DST], 0x54);
        assert_eq!(read_le_u16(&ram, NMI_UPDATE_TILEMAP_SRC), 0x0800);
        assert_eq!(ram[BGMODE_COPY], 9);
        assert_eq!(ram[TM_COPY], 0x11);
        assert_eq!(ram[TS_COPY], 0x02);
        assert_eq!(ram[W12SEL_COPY], 0x33);
        assert_eq!(ram[W34SEL_COPY], 3);
        assert_eq!(ram[WOBJSEL_COPY], 0x33);
        assert_eq!(ram[TMW_COPY], 0x11);
        assert_eq!(ram[TSW_COPY], 0x02);
        assert_eq!(ram[NMI_COPY_PACKETS_FLAG], 3);
        assert_eq!(ram[NMI_FLAG_UPDATE_POLYHEDRAL], 0xff);
        assert_eq!(ram[LOAD_CHR_HALFSLOT_EVEN_ODD], 12);
        assert_eq!(ram[NMI_THREAD_ACTIVE], 1);
        assert_eq!(read_le_u16(&ram, POLY_THREAD_STACK), 0x1f31);
        assert_eq!(ram[IRQ_FLAG], 0xff);
        assert_eq!(ram[VIRQ_TRIGGER], 0x70);
        assert_eq!(ram[DMA_HEAD_POINTER], 0x40);
        assert_eq!(ram[DMA_BODY_POINTER], 0x80);
        assert_eq!(ram[HDMAEN_COPY], 0x80);
        assert_eq!(ram[MOSAIC_LEVEL], 0x30);
        assert_eq!(ram[MOSAIC_COPY], 0x33);
        assert_eq!(ram[MOSAIC_TARGET_LEVEL], 0x0f);
        assert_eq!(ram[MOSAIC_INC_OR_DEC], 0);
        assert_eq!(read_le_u16(&ram, NMI_LOAD_TARGET_ADDR), 0x1234);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0xbbaa);
        assert_eq!(
            ram[crate::game_state::constants::nmi::VRAM_UPLOAD_DATA],
            0xcc
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + 14
            ),
            0x1234
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 4
            ),
            0x5678
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 6
            ),
            0xffff
        );
        assert_eq!(ram[INCREMENTAL_COUNTER_FOR_VRAM], 0);
        assert_eq!(ram[STAR_TILE_RESTORE_PHASE], 0);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_3), 0x9000);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_0), 0x9001);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_4), 0x9002);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_1), 0x9003);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_5), 0x9004);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_2), 0x9005);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_20), 0x9014);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_21), 0x9015);
        assert_eq!(read_le_u16(&ram, BG_TILE_ANIMATION_COUNTDOWN), 0xffff);
        assert_eq!(read_le_u16(&ram, messaging::MESSAGE_DMA_DST_ADDR), 0x6080);
        assert_eq!(ram[OVERWORLD_FIXED_COLOR_PLUSMINUS], 0x30);
        assert_eq!(ram[FLAG_TRAVEL_BIRD], 0x08);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    }
}
