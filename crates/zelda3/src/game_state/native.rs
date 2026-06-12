//! Native game-state model.
//!
//! Byte-backed views remain the compatibility surface while native state is
//! proven subsystem by subsystem. Native structs own domain fields and can be
//! projected to or loaded from WRAM during the transition.

mod display;
mod frame;
mod world;

pub(crate) use display::{DisplayState, NativeDisplayStateViewMut, NativeVramUploadDataViewMut};
pub(crate) use frame::{FrameState, NativeFrameStateView, NativeFrameStateViewMut};
pub(crate) use world::{NativeWorldLocationViewMut, WorldLocationState};

#[cfg(test)]
use crate::game_state::constants::*;
#[cfg(test)]
use crate::types::{read_le_u16, write_le_u16};

fn ram_byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GameState {
    pub(crate) frame: FrameState,
    pub(crate) world_location: WorldLocationState,
    pub(crate) display: DisplayState,
}

impl GameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            frame: FrameState::load_from_ram(ram),
            world_location: WorldLocationState::load_from_ram(ram),
            display: DisplayState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.frame.write_to_ram(ram);
        self.world_location.write_to_ram(ram);
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
    fn native_frame_mut_view_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 1;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 3;
        ram[FRAME_COUNTER] = 4;
        ram[SAVED_MODULE_FOR_MENU] = 8;
        ram[MODAL_PAUSE_FLAG] = 1;

        let mut frame = FrameState::default();
        {
            let mut view = NativeFrameStateViewMut::new(&mut frame, &mut ram);
            view.increment_submodule();
            view.set_subsubmodule(9);
            view.increment_frame_counter();
            view.save_main_module_for_menu();
            view.clear_saved_module_for_menu();
            view.save_submodule_for_menu();
            view.clear_modal_pause_flag();
            view.increment_modal_pause_flag();
            view.set_modal_pause_flag(6);
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
            let mut view = NativeWorldLocationViewMut::new(&mut world, &mut ram);
            view.increment_dungeon_room_index_by(2);
            view.set_overworld_screen(0x5b);
            view.set_indoor_flag(0);
        }

        assert_eq!(world.dungeon_room, 0x0126);
        assert_eq!(world.overworld_screen, 0x005b);
        assert_eq!(world.indoor_flag, 0);
        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0126);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
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
        ram[HDMAEN_COPY] = 0xc0;
        ram[MOSAIC_COPY] = 0x73;
        ram[MOSAIC_LEVEL] = 0x70;
        ram[MOSAIC_TARGET_LEVEL] = 0x1f;
        ram[MOSAIC_INC_OR_DEC] = 1;
        write_le_u16(&mut ram, NMI_LOAD_TARGET_ADDR, 0x2146);
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0124);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_DST_ADDR, 0x6040);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_BASE, 0x4841);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_LIMIT, 0x007f);
        write_le_u16(&mut ram, messaging::MESSAGE_DMA_TILE_SENTINEL, 0xffff);
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);

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
        assert_eq!(display.message_dma_destination_address, 0x6040);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(display.animated_tile_data_source_address, 0xa680);
        assert_eq!(display.animated_tile_data_source_usize(), 0xa680);
        assert!(display.has_animated_tile_data_source());
        assert_eq!(display.animated_tile_vram_destination_address, 0x3b00);
        assert_eq!(display.animated_tile_vram_destination_usize(), 0x3b00);

        display.screen_brightness = 0x80;
        display.nmi_update_latch = 0;
        display.core_update_disable_flag = 0;
        display.pending_nmi_subroutine = 0;
        display.bg_vram_load_mode = 0;
        display.pending_tilemap_update_destination_page = 0x40;
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
        display.animated_tile_data_source_address = 0xac80;
        display.animated_tile_vram_destination_address = 0x3c00;
        display.write_to_ram(&mut ram);

        assert_eq!(ram[INIDISP_COPY], 0x80);
        assert_eq!(ram[NMI_BOOLEAN], 0);
        assert_eq!(ram[NMI_DISABLE_CORE_UPDATES], 0);
        assert_eq!(ram[NMI_SUBROUTINE_INDEX], 0);
        assert_eq!(ram[NMI_LOAD_BG_FROM_VRAM], 0);
        assert_eq!(ram[NMI_UPDATE_TILEMAP_DST], 0x40);
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
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    }

    #[test]
    fn native_vram_upload_mut_view_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);

        let mut display = DisplayState::default();
        {
            let mut view = NativeVramUploadDataViewMut::new(&mut display, &mut ram);
            view.advance_offset_by(0x20);
            view.clear_offset();
            view.set_offset(0x0034);
        }

        assert_eq!(display.vram_upload_cursor, 0x0034);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0034);
    }

    #[test]
    fn native_display_mut_view_syncs_seeded_ram_and_dual_writes_brightness() {
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
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);

        let mut display = DisplayState::default();
        {
            let mut view = NativeDisplayStateViewMut::new(&mut display, &mut ram);
            view.increment_screen_brightness();
            view.decrement_screen_brightness();
            view.set_screen_brightness(0x80);
            view.clear_nmi_update_latch();
            view.latch_nmi_update();
            view.clear_core_update_disable_flag();
            view.set_core_update_disable_flag(7);
            assert_eq!(view.take_pending_nmi_subroutine(), 6);
            view.set_pending_nmi_subroutine(11);
            view.clear_bg_vram_load_mode();
            view.set_bg_vram_load_mode(5);
            view.queue_tilemap_update(0x52, 0x0400);
            view.clear_pending_tilemap_update_destination();
            view.queue_tilemap_update(0x54, 0x0800);
            view.set_bg_mode(9);
            view.set_layer_masks_word(0x0116);
            view.and_main_screen_layers(0x15);
            view.or_main_screen_layers(0x01);
            view.and_sub_screen_layers(0x0f);
            view.or_sub_screen_layers(0x10);
            view.clear_sub_screen_layers_word();
            view.set_main_screen_layers(0x11);
            view.set_sub_screen_layers(0x02);
            view.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            view.set_bg12_window_selection(0x11);
            view.set_bg34_window_selection(0x22);
            view.set_object_color_window_selection(0x30);
            view.set_main_screen_window_layers(0x04);
            view.set_sub_screen_window_layers(0x05);
            view.clear_window_main_sub_masks();
            view.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            view.clear_nmi_copy_packets_request();
            view.request_nmi_copy_packets();
            view.set_nmi_copy_packets_request(3);
            view.clear_pending_polyhedral_update();
            view.request_polyhedral_nmi_update();
            view.increment_chr_halfslot_request();
            view.clear_chr_halfslot_request();
            view.set_chr_halfslot_request(12);
            view.deactivate_nmi_thread();
            view.activate_nmi_thread();
            view.set_nmi_thread_stack_pointer(0x1f31);
            view.clear_irq_control_flag();
            view.set_irq_control_flag(0xff);
            view.set_vertical_irq_trigger(0x70);
            view.set_sprite_dma_head_pointer(0x40);
            view.set_sprite_dma_body_pointer(0x80);
            view.clear_hdma_enable_mask();
            view.set_hdma_enable_mask(0x80);
            view.set_mosaic_level(0x40);
            assert_eq!(view.increment_mosaic_level_by(0x10), 0x50);
            assert_eq!(view.decrement_mosaic_level_by(0x20), 0x30);
            view.set_mosaic_copy_from_level_or(3);
            view.set_mosaic_target_level_word(0x001f);
            view.clear_mosaic_target_level_word();
            view.set_mosaic_target_level(0x0f);
            view.set_mosaic_direction(1);
            view.clear_mosaic_direction();
            view.set_nmi_load_target_page(0x80);
            view.set_nmi_load_target_address(0x1234);
            assert_eq!(view.increment_vram_upload_counter(), 0xff);
            assert_eq!(view.increment_vram_upload_counter(), 0);
            view.reset_incremental_vram_upload_counter();
            view.set_message_dma_destination_address(0x6080);
            view.set_message_dma_tile_base(0x4841);
            view.set_message_dma_tile_limit(0x007f);
            view.set_message_dma_tile_sentinel(0xffff);
            view.set_overworld_fixed_color_adjustment(0x30);
            view.set_travel_bird_tile_offset(0x08);
            view.set_animated_tile_data_source_address(0xac80);
            view.set_animated_tile_vram_destination_address(0x3c00);
        }

        assert_eq!(display.screen_brightness, 0x80);
        assert_eq!(display.nmi_update_latch, 1);
        assert_eq!(display.core_update_disable_flag, 7);
        assert_eq!(display.pending_nmi_subroutine, 11);
        assert_eq!(display.bg_vram_load_mode, 5);
        assert_eq!(display.pending_tilemap_update_destination_page, 0x54);
        assert!(display.has_pending_tilemap_update());
        assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5400);
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
        assert_eq!(display.vram_upload_cursor, 0x0010);
        assert_eq!(display.incremental_vram_upload_counter, 0);
        assert_eq!(display.incremental_vram_upload_counter_usize(), 0);
        assert_eq!(display.message_dma_destination_address, 0x6080);
        assert_eq!(display.message_dma_destination_address_usize(), 0x6080);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(display.overworld_fixed_color_adjustment, 0x30);
        assert_eq!(display.travel_bird_tile_offset, 0x08);
        assert!(display.has_travel_bird_tile_upload());
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
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0010);
        assert_eq!(ram[INCREMENTAL_COUNTER_FOR_VRAM], 0);
        assert_eq!(read_le_u16(&ram, messaging::MESSAGE_DMA_DST_ADDR), 0x6080);
        assert_eq!(ram[OVERWORLD_FIXED_COLOR_PLUSMINUS], 0x30);
        assert_eq!(ram[FLAG_TRAVEL_BIRD], 0x08);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    }
}
