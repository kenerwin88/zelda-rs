use super::*;

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
    ram[CRYSTAL_ROTATION_COUNTER] = 0xf0;
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
    write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_DST_ADDR, 0x6040);
    write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_TILE_BASE, 0x4841);
    write_le_u16(
        &mut ram,
        messaging_constants::MESSAGE_DMA_TILE_LIMIT,
        0x007f,
    );
    write_le_u16(
        &mut ram,
        messaging_constants::MESSAGE_DMA_TILE_SENTINEL,
        0xffff,
    );
    ram[HUD_TILE_INDICES_BUFFER] = 0xbe;
    ram[HUD_TILE_INDICES_BUFFER + 1] = 0xef;
    ram[STAR_TILE_RESTORE_PHASE] = 1;
    write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
    write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);
    write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x0168);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0030);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
    write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
    ram[WATERGATE_POINTER] = 0x06;
    write_le_u16(&mut ram, WATERGATE_POS, 0x0780);
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
    assert_eq!(display.crystal_rotation_counter, 0xf0);
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
    assert_eq!(
        &display.animated_tile_dma_source_bytes(&ram)[..2],
        &[0xde, 0xad]
    );
    assert!(display.has_animated_tile_data_source());
    assert_eq!(display.animated_tile_vram_destination_address, 0x3b00);
    assert_eq!(display.animated_tile_vram_destination_usize(), 0x3b00);
    assert_eq!(display.attract_vram_destination_address, 0x0168);
    assert!(!display.attract_vram_destination_high_is_clear());
    assert_eq!(display.water_hdma_window.window_x(), 0x0120);
    assert_eq!(display.water_hdma_window.window_y(), 0x0140);
    assert_eq!(display.water_hdma_window.window_y_radius(), 0x0030);
    assert_eq!(display.water_hdma_window.window_x_radius(), 0x0040);
    assert_eq!(
        display.water_hdma_window.watergate_spotlight_y_upper(),
        0x0050
    );
    assert_eq!(display.water_hdma_window.watergate_pointer(), 0x06);
    assert_eq!(display.water_hdma_window.watergate_tilemap_pos_x2(), 0x0780);

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
    display.crystal_rotation_counter = 0x10;
    display.sprite_dma_head_pointer = 0x40;
    display.sprite_dma_body_pointer = 0x80;
    display.hdma_enable_mask = 0x80;
    display.mosaic_copy = 3;
    display.mosaic_level = 0x20;
    display.mosaic_target_level = 0;
    display.mosaic_direction = 0;
    display.nmi_load_target_address = 0x0080;
    display.vram_upload_cursor = 0x0042;
    // VRAM_UPLOAD_OFFSET (0x1000) is mode-reused as word 0 of the tilemap upload buffer
    // during room draw. write_to_ram must NOT project the cursor over it (the cursor is kept
    // RAM-coherent by its setters instead). Seed a tilemap-data sentinel and assert below
    // that write_to_ram leaves it intact — regression for the VRAM word-0 clobber.
    write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x3c15);
    display.message_dma_destination_address = 0x6080;
    display.message_dma_tile_base = 0x4842;
    display.message_dma_tile_limit = 0x0080;
    display.message_dma_tile_sentinel = 0xfffe;
    display.star_tile_restore_phase = 0;
    display.animated_tile_data_source_address = 0xac80;
    display.animated_tile_vram_destination_address = 0x3c00;
    display.attract_vram_destination_address = 0x0068;
    display.water_hdma_window.set_window_x(0x0220);
    display.water_hdma_window.set_window_y(0x0240);
    display.water_hdma_window.set_window_y_radius_byte(0x31);
    display.water_hdma_window.set_window_x_radius(0x0048);
    display
        .water_hdma_window
        .set_watergate_spotlight_y_upper(0x0058);
    display.water_hdma_window.set_watergate_pointer(0x07);
    display
        .water_hdma_window
        .set_watergate_tilemap_pos_x2(0x0880);
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
    assert_eq!(
        read_le_u16(&ram, messaging_constants::MESSAGE_DMA_DST_ADDR),
        0x6080
    );
    assert_eq!(
        read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_BASE),
        0x4842
    );
    assert_eq!(
        read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_LIMIT),
        0x0080
    );
    assert_eq!(
        read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_SENTINEL),
        0xfffe
    );
    assert_eq!(ram[NMI_FLAG_UPDATE_POLYHEDRAL], 0);
    assert_eq!(ram[LOAD_CHR_HALFSLOT_EVEN_ODD], 0);
    assert_eq!(ram[NMI_THREAD_ACTIVE], 0);
    assert_eq!(read_le_u16(&ram, POLY_THREAD_STACK), 0x1f31);
    assert_eq!(ram[IRQ_FLAG], 0);
    assert_eq!(ram[VIRQ_TRIGGER], 0x70);
    assert_eq!(ram[CRYSTAL_ROTATION_COUNTER], 0x10);
    assert_eq!(ram[DMA_HEAD_POINTER], 0x40);
    assert_eq!(ram[DMA_BODY_POINTER], 0x80);
    assert_eq!(ram[HDMAEN_COPY], 0x80);
    assert_eq!(ram[MOSAIC_COPY], 3);
    assert_eq!(ram[MOSAIC_LEVEL], 0x20);
    assert_eq!(ram[MOSAIC_TARGET_LEVEL], 0);
    assert_eq!(ram[MOSAIC_INC_OR_DEC], 0);
    assert_eq!(read_le_u16(&ram, NMI_LOAD_TARGET_ADDR), 0x0080);
    // write_to_ram left the seeded tilemap-data word at 0x1000 untouched (cursor not projected).
    assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x3c15);
    assert_eq!(ram[STAR_TILE_RESTORE_PHASE], 0);
    assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
    assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    // ATTRACT_VRAM_DST (0x30) overlaps LINK velocity and is NOT bulk-projected;
    // it's written only by the targeted attract-vram bridge (tested below).
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
    assert_eq!(ram[WATER_HDMA_WINDOW_Y_RADIUS], 0x31);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
    assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x0058);
    assert_eq!(ram[WATERGATE_POINTER], 0x07);
    assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
}

#[test]
fn native_attract_vram_destination_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x0160);

    let mut display = DisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativeAttractVramDestinationBridgeMut::new(&mut display, &mut ram);
        bridge.set_page_offset(0x70);
        bridge.decrement_page_offset();
        bridge.set_address(0x0068);
        assert_eq!(bridge.decrement_address(), 0x0067);
        bridge.clear_address();
        bridge.set_address(0x0068);
    }

    assert_eq!(display.attract_vram_destination_address, 0x0068);
    assert!(display.attract_vram_destination_high_is_clear());
    assert_eq!(read_le_u16(&ram, ATTRACT_VRAM_DST), 0x0068);
}

#[test]
fn display_state_owns_attract_vram_destination_behavior() {
    let mut display = DisplayState {
        attract_vram_destination_address: 0x01ff,
        ..DisplayState::default()
    };

    assert_eq!(display.attract_vram_destination_page_offset(), 0xff);
    assert_eq!(
        display.decrement_attract_vram_destination_page_offset(),
        0xfe
    );
    assert_eq!(display.attract_vram_destination_address, 0x01fe);

    display.set_attract_vram_destination_page_offset(0x34);
    assert_eq!(display.attract_vram_destination_address, 0x0134);
    assert_eq!(display.decrement_attract_vram_destination_address(), 0x0133);

    display.clear_attract_vram_destination_address();
    assert_eq!(display.attract_vram_destination_address, 0);
    assert!(display.attract_vram_destination_high_is_clear());
}

#[test]
fn native_attract_vram_destination_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x01aa);
    let mut display = DisplayState {
        attract_vram_destination_address: 0x0200,
        ..DisplayState::default()
    };

    {
        let mut bridge = NativeAttractVramDestinationBridgeMut::new(&mut display, &mut ram);
        bridge.set_page_offset(0x34);
        assert_eq!(bridge.decrement_address(), 0x0233);
    }

    assert_eq!(display.attract_vram_destination_address, 0x0233);
    assert_eq!(read_le_u16(&ram, ATTRACT_VRAM_DST), 0x0233);
}

#[test]
fn water_hdma_window_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0230);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
    write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
    ram[WATERGATE_POINTER] = 0x06;
    write_le_u16(&mut ram, WATERGATE_POS, 0x0780);

    let mut water = WaterHdmaWindowState::load_from_ram(&ram);
    assert_eq!(water.window_x(), 0x0120);
    assert_eq!(water.window_y(), 0x0140);
    assert_eq!(water.window_y_radius(), 0x0230);
    assert_eq!(water.window_x_radius(), 0x0040);
    assert_eq!(water.watergate_spotlight_y_upper(), 0x0050);
    assert_eq!(water.watergate_pointer(), 0x06);
    assert_eq!(water.watergate_tilemap_pos_x2(), 0x0780);

    water.set_window_x(0x0220);
    water.set_window_y(0x0240);
    water.set_window_x_radius(0x0048);
    water.set_window_y_radius_byte(0x31);
    water.decrement_watergate_spotlight_y_upper();
    assert_eq!(water.increment_watergate_pointer(), 0x07);
    water.set_watergate_tilemap_pos_x2(0x0880);
    water.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x0231);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
    assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x004f);
    assert_eq!(ram[WATERGATE_POINTER], 0x07);
    assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
}

#[test]
fn native_water_hdma_window_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0230);
    write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
    write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
    ram[WATERGATE_POINTER] = 0x06;
    write_le_u16(&mut ram, WATERGATE_POS, 0x0780);
    write_le_u16(&mut ram, SPOTLIGHT_Y_UPPER, 0x1111);
    write_le_u16(&mut ram, SPOTLIGHT_WINDOW_Y_BUFFER, 0x2210);

    let mut display = DisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativeWaterHdmaWindowBridgeMut::new(&mut display, &mut ram);
        bridge.set_window_x(0x0220);
        bridge.set_window_y(0x0240);
        bridge.set_window_x_radius(0x0048);
        bridge.set_window_y_radius_byte(0x31);
        assert_eq!(bridge.decrement_watergate_spotlight_y_upper(), 0x004f);
        bridge.set_watergate_spotlight_y_upper(0x0058);
        bridge.set_watergate_pointer(0x07);
        assert_eq!(bridge.increment_watergate_pointer(), 0x08);
        bridge.set_watergate_tilemap_pos_x2(0x0880);
        assert_eq!(bridge.advance_watergate_window_y_radius(), 0x51);
    }

    assert_eq!(display.water_hdma_window.window_x(), 0x0220);
    assert_eq!(display.water_hdma_window.window_y(), 0x0240);
    assert_eq!(display.water_hdma_window.window_y_radius(), 0x0251);
    assert_eq!(display.water_hdma_window.window_x_radius(), 0x0048);
    assert_eq!(
        display.water_hdma_window.watergate_spotlight_y_upper(),
        0x0058
    );
    assert_eq!(display.water_hdma_window.watergate_pointer(), 0x08);
    assert_eq!(display.water_hdma_window.watergate_tilemap_pos_x2(), 0x0880);
    assert_eq!(display.spotlight_hdma.y_upper(), 0x0058);
    assert_eq!(display.spotlight_hdma.window_y_buffer_byte(), 0x11);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x0251);
    assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
    assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x0058);
    assert_eq!(ram[WATERGATE_POINTER], 0x08);
    assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
    assert_eq!(ram[SPOTLIGHT_Y_UPPER], 0x58);
    assert_eq!(ram[SPOTLIGHT_WINDOW_Y_BUFFER], 0x11);
}

#[test]
fn native_spotlight_hdma_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SPOTLIGHT_Y_LOWER, 0x0010);
    write_le_u16(&mut ram, SPOTLIGHT_Y_UPPER, 0x0020);
    write_le_u16(&mut ram, SPOTLIGHT_WINDOW_RADIUS, 0x1234);
    write_le_u16(&mut ram, SPOTLIGHT_WINDOW_STATE, 0x5678);
    write_le_u16(&mut ram, SPOTLIGHT_WINDOW_Y_BUFFER, 0x9abc);
    write_le_u16(&mut ram, HDMA_TABLE_DYNAMIC + 6, 0xbeef);

    let mut spotlight = SpotlightHdmaState::load_from_ram(&ram);
    {
        let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
        bridge.set_y_lower(0x0030);
        bridge.set_y_upper(0x0040);
        bridge.set_window_radius_byte(0x80);
        bridge.shr_window_radius_byte(1);
        bridge.add_window_radius_byte(0x10);
        bridge.set_window_state_byte(0x02);
        assert_eq!(bridge.decrement_window_y_buffer(), 0x9abb);
        bridge.set_hdma_table_dynamic_entry(3, 0xcafe);
        bridge.clear_hdma_table_dynamic_range(3, 1);
    }

    assert_eq!(spotlight.y_lower(), 0x0030);
    assert_eq!(spotlight.y_upper(), 0x0040);
    assert_eq!(spotlight.window_radius(), 0x1250);
    assert_eq!(spotlight.window_state(), 0x5602);
    assert_eq!(spotlight.window_y_buffer(), 0x9abb);
    assert_eq!(spotlight.hdma_table_dynamic_entry(3), 0);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_Y_LOWER), 0x0030);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_Y_UPPER), 0x0040);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_RADIUS), 0x1250);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_STATE), 0x5602);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_Y_BUFFER), 0x9abb);
    assert_eq!(read_le_u16(&ram, HDMA_TABLE_DYNAMIC + 6), 0);
}

#[test]
fn native_spotlight_hdma_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SPOTLIGHT_Y_LOWER, 0x0010);
    write_le_u16(&mut ram, SPOTLIGHT_WINDOW_RADIUS, 0x0020);
    let mut spotlight = SpotlightHdmaState::default();
    spotlight.set_y_lower(0x1200);
    spotlight.set_window_radius(0x3456);

    {
        let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
        bridge.add_window_radius_byte(0x01);
    }

    assert_eq!(spotlight.y_lower(), 0x1200);
    assert_eq!(spotlight.window_radius(), 0x3457);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_Y_LOWER), 0x1200);
    assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_RADIUS), 0x3457);
}

#[test]
fn native_spotlight_hdma_bridge_projects_dynamic_table_to_reserved_hdma_table() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut spotlight = SpotlightHdmaState::default();
    spotlight.set_hdma_table_dynamic_entry(0, 0x1111);
    spotlight.set_hdma_table_dynamic_entry(1, 0x2222);
    write_le_u16(&mut ram, RESERVED_HDMA_TABLE, 0xaaaa);
    write_le_u16(&mut ram, RESERVED_HDMA_TABLE + 2, 0xbbbb);

    {
        let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
        bridge.project_dynamic_table_to_reserved_hdma_table(2);
    }

    assert_eq!(read_le_u16(&ram, RESERVED_HDMA_TABLE), 0x1111);
    assert_eq!(read_le_u16(&ram, RESERVED_HDMA_TABLE + 2), 0x2222);
}

#[test]
fn native_spotlight_hdma_bridge_restores_and_backs_up_saveload_table() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, SAVELOAD_HDMA_TABLE, 0x3333);
    write_le_u16(&mut ram, SAVELOAD_HDMA_TABLE + 2, 0x4444);
    let mut spotlight = SpotlightHdmaState::default();

    {
        let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
        bridge.restore_dynamic_table_from_saveload_buffer(2);
    }
    assert_eq!(spotlight.hdma_table_dynamic_entry(0), 0x3333);
    assert_eq!(spotlight.hdma_table_dynamic_entry(1), 0x4444);
    assert_eq!(read_le_u16(&ram, HDMA_TABLE_DYNAMIC), 0x3333);
    assert_eq!(read_le_u16(&ram, HDMA_TABLE_DYNAMIC + 2), 0x4444);

    spotlight.set_hdma_table_dynamic_entry(0, 0x5555);
    spotlight.set_hdma_table_dynamic_entry(1, 0x6666);
    {
        let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
        bridge.backup_dynamic_table_to_saveload_buffer(2);
    }
    assert_eq!(read_le_u16(&ram, SAVELOAD_HDMA_TABLE), 0x5555);
    assert_eq!(read_le_u16(&ram, SAVELOAD_HDMA_TABLE + 2), 0x6666);
}

#[test]
fn palette_buffer_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, MAIN_PALETTE_BUFFER + 4, 0x1234);
    write_le_u16(&mut ram, AUX_PALETTE_BUFFER + 6, 0x5678);
    ram[AUX_PALETTE_BUFFER + 255] = 0x9a;
    ram[MAPBAK_PALETTE + 511] = 0xbc;
    write_le_u16(&mut ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
    ram[PALETTE_SP0L] = 1;
    ram[PALETTE_SP5L] = 2;
    ram[PALETTE_SP6L] = 3;
    ram[PALETTE_MAIN_INDOORS] = 4;
    ram[HUD_PALETTE] = 5;
    ram[PALETTE_SP6R_INDOORS] = 6;
    ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 7;
    ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 8;
    ram[OVERWORLD_PALETTE_MODE] = 9;

    let palette = PaletteBufferState::load_from_ram(&ram);
    assert_eq!(palette.main_color(2), 0x1234);
    assert_eq!(palette.aux_color(3), 0x5678);
    assert_eq!(palette.aux_visible_slice()[255], 0x9a);
    assert_eq!(palette.overworld_palette_backup()[511], 0xbc);
    assert_eq!(palette.overworld_aux_or_main_offset(), 0x0200);

    let mut projected = vec![0; WRAM_SIZE];
    palette.write_to_ram(&mut projected);
    assert_eq!(PaletteBufferState::load_from_ram(&projected), palette);
    assert_eq!(projected[PALETTE_SP0L], 1);
    assert_eq!(projected[OVERWORLD_PALETTE_MODE], 9);
}

#[test]
fn palette_buffer_state_owns_color_and_palette_metadata_behavior() {
    let mut palette = PaletteBufferState::default();

    palette.set_main_color(2, 0x1234);
    palette.set_aux_color(3, 0x5678);
    palette.set_overworld_aux_or_main_offset(0x12ab);
    palette.keep_overworld_aux_or_main_low_byte();
    palette.select_overworld_aux_palette_offset();

    palette.copy_aux_visible_from(&vec![0x22; 256]);
    palette.copy_main_palette_bytes(&[0x11, 0x22, 0x33, 0x44], 4);
    palette.backup_overworld_palette_from(&vec![0x77; 512]);
    palette.clear_aux_sprite_subpalettes();

    palette.set_sprite_palette_0_left(1);
    palette.set_sprite_palette_5_left(2);
    palette.set_sprite_palette_6_left(3);
    palette.set_main_palette_indoors(4);
    palette.set_hud_palette(5);
    palette.set_sprite_palette_6_right_indoors(6);
    palette.set_overworld_palette_aux2_hi(7);
    palette.set_overworld_palette_aux3_lo(8);
    palette.set_overworld_palette_mode(9);

    assert_eq!(palette.main_color(0), 0x2211);
    assert_eq!(palette.main_color(1), 0x4433);
    assert_eq!(palette.main_color(2), 0x1234);
    assert_eq!(palette.aux_color(3), 0x2222);
    assert_eq!(palette.aux_full_slice()[0x180..0x200], [0; 0x80]);
    assert_eq!(palette.overworld_palette_backup()[511], 0x77);
    assert_eq!(palette.overworld_aux_or_main_offset(), 0x0200);
    assert_eq!(palette.sprite_palette_0_left(), 1);
    assert_eq!(palette.sprite_palette_5_left(), 2);
    assert_eq!(palette.sprite_palette_6_left(), 3);
    assert_eq!(palette.main_palette_indoors(), 4);
    assert_eq!(palette.hud_palette(), 5);
    assert_eq!(palette.sprite_palette_6_right_indoors(), 6);
    assert_eq!(palette.overworld_palette_aux2_hi(), 7);
    assert_eq!(palette.overworld_palette_aux3_lo(), 8);
    assert_eq!(palette.overworld_palette_mode(), 9);

    palette.clear_overworld_aux_or_main_offset();
    palette.clear_main_full();
    assert_eq!(palette.overworld_aux_or_main_offset(), 0);
    assert_eq!(palette.main_color(0), 0);
}

#[test]
fn native_palette_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0xff; WRAM_SIZE];
    write_le_u16(&mut ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x12ab);

    let mut display = DisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativePaletteBufferBridgeMut::new(&mut display, &mut ram);
        bridge.set_main_color(2, 0x1234);
        bridge.set_aux_color(3, 0x5678);
        bridge.keep_overworld_aux_or_main_low_byte();
        bridge.select_overworld_aux_palette_offset();
        bridge.copy_aux_visible_from(&vec![0x22; 256]);
        bridge.copy_main_palette_bytes(&[0x11, 0x22, 0x33, 0x44], 4);
        bridge.backup_overworld_palette_from(&vec![0x77; 512]);
        bridge.clear_aux_sprite_subpalettes();
        bridge.set_sp0l(1);
        bridge.set_sp5l(2);
        bridge.set_sp6l(3);
        bridge.set_palette_main_indoors(4);
        bridge.set_hud_palette(5);
        bridge.set_sp6r_indoors(6);
        bridge.set_overworld_palette_aux2_hi(7);
        bridge.set_overworld_palette_aux3_lo(8);
        bridge.set_bg_tile_animation_countdown(0x9abc);
        bridge.set_overworld_palette_mode(9);
    }

    assert_eq!(display.palette_buffer.main_color(0), 0x2211);
    assert_eq!(display.palette_buffer.main_color(1), 0x4433);
    assert_eq!(display.palette_buffer.aux_color(3), 0x2222);
    assert_eq!(display.palette_buffer.aux_visible_slice()[0], 0x22);
    assert_eq!(
        display.palette_buffer.aux_full_slice()[0x180..0x200],
        [0; 0x80]
    );
    assert_eq!(display.palette_buffer.overworld_palette_backup()[511], 0x77);
    assert_eq!(
        display.palette_buffer.overworld_aux_or_main_offset(),
        0x0200
    );
    assert_eq!(display.bg_tile_animation_countdown, 0x9abc);
    assert_eq!(read_le_u16(&ram, MAIN_PALETTE_BUFFER), 0x2211);
    assert_eq!(read_le_u16(&ram, AUX_PALETTE_BUFFER + 6), 0x2222);
    assert_eq!(ram[MAPBAK_PALETTE + 511], 0x77);
    assert_eq!(read_le_u16(&ram, OVERWORLD_PALETTE_AUX_OR_MAIN), 0x0200);
    assert_eq!(ram[PALETTE_SP0L], 1);
    assert_eq!(ram[PALETTE_SP5L], 2);
    assert_eq!(ram[PALETTE_SP6L], 3);
    assert_eq!(ram[PALETTE_MAIN_INDOORS], 4);
    assert_eq!(ram[HUD_PALETTE], 5);
    assert_eq!(ram[PALETTE_SP6R_INDOORS], 6);
    assert_eq!(ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI], 7);
    assert_eq!(ram[OVERWORLD_PALETTE_AUX3_BP7_LO], 8);
    assert_eq!(read_le_u16(&ram, BG_TILE_ANIMATION_COUNTDOWN), 0x9abc);
    assert_eq!(ram[OVERWORLD_PALETTE_MODE], 9);
}

#[test]
fn palette_filter_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, PALETTE_FILTER_COUNTDOWN, 0x1204);
    write_le_u16(&mut ram, DARKENING_OR_LIGHTENING_SCREEN, 0x34ff);
    ram[CGWSEL_COPY] = 0x20;
    ram[CGADSUB_COPY] = 0x31;
    ram[COLDATA_COPY0] = 0x21;
    ram[COLDATA_COPY1] = 0x43;
    ram[COLDATA_COPY2] = 0x85;

    let palette_filter = PaletteFilterState::load_from_ram(&ram);
    assert_eq!(palette_filter.countdown(), 4);
    assert_eq!(palette_filter.countdown_word(), 0x1204);
    assert_eq!(palette_filter.darkening_or_lightening_screen(), 0xff);
    assert_eq!(palette_filter.darkening_or_lightening_screen_word(), 0x34ff);
    assert_eq!(palette_filter.color_window_selection(), 0x20);
    assert_eq!(palette_filter.color_math_control(), 0x31);
    assert_eq!(palette_filter.color_window_and_math_word(), 0x3120);
    assert_eq!(palette_filter.fixed_color_red(), 0x21);
    assert_eq!(palette_filter.fixed_color_green(), 0x43);
    assert_eq!(palette_filter.fixed_color_blue(), 0x85);
    assert_eq!(palette_filter.fixed_color_component(0), 0x21);
    assert_eq!(palette_filter.fixed_color_component(3), 0);

    let mut projected = vec![0; WRAM_SIZE];
    // CGADSUB_COPY + 1 is HDMAEN_COPY, owned by a different system; the
    // palette filter must leave it untouched when projecting.
    projected[CGADSUB_COPY + 1] = 0xab;
    palette_filter.write_to_ram(&mut projected);
    assert_eq!(read_le_u16(&projected, PALETTE_FILTER_COUNTDOWN), 0x1204);
    assert_eq!(
        read_le_u16(&projected, DARKENING_OR_LIGHTENING_SCREEN),
        0x34ff
    );
    assert_eq!(projected[CGWSEL_COPY], 0x20);
    assert_eq!(projected[CGADSUB_COPY], 0x31);
    assert_eq!(projected[CGADSUB_COPY + 1], 0xab);
    assert_eq!(projected[COLDATA_COPY0], 0x21);
    assert_eq!(projected[COLDATA_COPY1], 0x43);
    assert_eq!(projected[COLDATA_COPY2], 0x85);
}

#[test]
fn palette_filter_state_owns_screen_filter_behavior() {
    let mut filter = PaletteFilterState::default();
    filter.set_countdown(0xff);
    filter.set_fixed_color_red(0x20);
    filter.set_fixed_color_green(0x40);
    filter.set_fixed_color_blue(0x80);

    filter.increment_countdown();
    filter.decrement_countdown();
    filter.set_countdown_word(0x5607);
    filter.xor_darkening_or_lightening_screen(0xff);
    filter.set_darkening_or_lightening_screen_word(0x7809);
    filter.set_color_window_and_math_word(0x3322);
    filter.set_color_window_selection(0x24);
    filter.set_color_math_control(0x35);
    filter.or_fixed_color_red(0x01);
    filter.subtract_fixed_color_red(2);
    filter.set_fixed_color_green(0x50);
    filter.or_fixed_color_green(0x0f);
    filter.subtract_fixed_color_green(1);
    filter.set_fixed_color_blue(0x90);
    filter.or_fixed_color_blue(0x0f);
    filter.subtract_fixed_color_blue(1);
    assert!(filter.set_fixed_color_component(2, 0x88));
    assert!(filter.or_fixed_color_component(0, 0x10));
    assert!(!filter.set_fixed_color_component(3, 0xaa));
    assert!(!filter.or_fixed_color_component(3, 0xaa));
    filter.set_fixed_color_red(0x22);

    assert_eq!(filter.countdown_word(), 0x5607);
    assert_eq!(filter.darkening_or_lightening_screen_word(), 0x7809);
    assert_eq!(filter.color_window_and_math_word(), 0x3524);
    assert_eq!(filter.fixed_color_red(), 0x22);
    assert_eq!(filter.fixed_color_green(), 0x5e);
    assert_eq!(filter.fixed_color_blue(), 0x88);
}

#[test]
fn native_palette_filter_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, PALETTE_FILTER_COUNTDOWN, 0x1200);
    write_le_u16(&mut ram, DARKENING_OR_LIGHTENING_SCREEN, 0x3401);
    ram[CGWSEL_COPY] = 0x20;
    ram[CGADSUB_COPY] = 0x31;
    ram[CGADSUB_COPY + 1] = 0x42;
    ram[COLDATA_COPY0] = 0x20;
    ram[COLDATA_COPY1] = 0x40;
    ram[COLDATA_COPY2] = 0x80;

    let mut display = DisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativePaletteFilterBridgeMut::new(&mut display, &mut ram);
        bridge.increment_countdown();
        bridge.decrement_countdown();
        bridge.set_countdown_word(0x5607);
        bridge.xor_darkening_or_lightening_screen(0xff);
        bridge.set_darkening_or_lightening_screen_word(0x7809);
        bridge.set_color_window_and_math_word(0x3322);
        bridge.set_color_window_selection(0x24);
        bridge.set_color_math_control(0x35);
        bridge.or_fixed_color_red(0x01);
        bridge.subtract_fixed_color_red(2);
        bridge.set_fixed_color_green(0x50);
        bridge.or_fixed_color_green(0x0f);
        bridge.subtract_fixed_color_green(1);
        bridge.set_fixed_color_blue(0x90);
        bridge.or_fixed_color_blue(0x0f);
        bridge.subtract_fixed_color_blue(1);
        bridge.set_fixed_color_component(2, 0x88);
        bridge.or_fixed_color_component(0, 0x10);
        bridge.set_fixed_color_red(0x22);
    }

    assert_eq!(display.palette_filter.countdown_word(), 0x5607);
    assert_eq!(
        display.palette_filter.darkening_or_lightening_screen_word(),
        0x7809
    );
    assert_eq!(display.palette_filter.color_window_and_math_word(), 0x3524);
    assert_eq!(display.palette_filter.fixed_color_red(), 0x22);
    assert_eq!(display.palette_filter.fixed_color_green(), 0x5e);
    assert_eq!(display.palette_filter.fixed_color_blue(), 0x88);
    assert_eq!(read_le_u16(&ram, PALETTE_FILTER_COUNTDOWN), 0x5607);
    assert_eq!(read_le_u16(&ram, DARKENING_OR_LIGHTENING_SCREEN), 0x7809);
    assert_eq!(ram[CGWSEL_COPY], 0x24);
    assert_eq!(ram[CGADSUB_COPY], 0x35);
    // CGADSUB_COPY + 1 is HDMAEN_COPY; the palette bridge leaves it untouched.
    assert_eq!(ram[CGADSUB_COPY + 1], 0x42);
    assert_eq!(ram[COLDATA_COPY0], 0x22);
    assert_eq!(ram[COLDATA_COPY1], 0x5e);
    assert_eq!(ram[COLDATA_COPY2], 0x88);
}

#[test]
fn trinexx_palette_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 2;
    ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 4;
    ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 6;
    ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 8;

    let mut palette = TrinexxPaletteState::load_from_ram(&ram);
    assert_eq!(
        palette,
        TrinexxPaletteState {
            red_shell_delay: 2,
            blue_shell_delay: 4,
            red_shell_step: 6,
            blue_shell_step: 8,
        }
    );

    palette.decrement_red_shell_delay();
    palette.decrement_blue_shell_delay();
    assert_eq!(palette.increment_red_shell_step(), 7);
    assert_eq!(palette.increment_blue_shell_step(), 9);
    palette.write_to_ram(&mut ram);

    assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_DELAY], 1);
    assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY], 3);
    assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_STEP], 7);
    assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_STEP], 9);
}

#[test]
fn trinexx_palette_state_owns_delay_and_step_behavior() {
    let mut palette = TrinexxPaletteState::default();

    palette.set_red_shell_delay(3);
    palette.set_blue_shell_delay(4);
    palette.decrement_red_shell_delay();
    palette.decrement_blue_shell_delay();
    palette.set_red_shell_step(0xff);
    palette.set_blue_shell_step(0xfe);

    assert_eq!(palette.increment_red_shell_step(), 0);
    assert_eq!(palette.increment_blue_shell_step(), 0xff);

    palette.set_red_shell_step(2);
    palette.set_blue_shell_step(5);
    assert_eq!(
        palette,
        TrinexxPaletteState {
            red_shell_delay: 2,
            blue_shell_delay: 3,
            red_shell_step: 2,
            blue_shell_step: 5,
        }
    );
}

#[test]
fn native_trinexx_palette_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 0;
    ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 0;
    ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 0xff;
    ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 0xfe;

    let mut display = DisplayState::load_from_ram(&ram);
    {
        let mut bridge = NativeTrinexxPaletteBridgeMut::new(&mut display, &mut ram);
        bridge.set_red_shell_delay(3);
        bridge.set_blue_shell_delay(4);
        bridge.decrement_red_shell_delay();
        bridge.decrement_blue_shell_delay();
        assert_eq!(bridge.increment_red_shell_step(), 0);
        assert_eq!(bridge.increment_blue_shell_step(), 0xff);
        bridge.set_red_shell_step(2);
        bridge.set_blue_shell_step(5);
    }

    assert_eq!(
        display.trinexx_palette,
        TrinexxPaletteState {
            red_shell_delay: 2,
            blue_shell_delay: 3,
            red_shell_step: 2,
            blue_shell_step: 5,
        }
    );
    assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_DELAY], 2);
    assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY], 3);
    assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_STEP], 2);
    assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_STEP], 5);
}

#[test]
fn hud_inventory_order_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    for index in 0..24 {
        ram[HUD_INVENTORY_ORDER + index] = 24 - index as u8;
    }

    let mut order = HudInventoryOrderState::load_from_ram(&ram);
    assert!(order.is_custom());
    assert_eq!(order.item(0), 24);
    assert_eq!(order.item(23), 1);
    assert_eq!(order.item(24), 0);

    order.initialize_default_order(24);
    order.swap_items(0, 23);
    order.write_to_ram(&mut ram);

    assert_eq!(ram[HUD_INVENTORY_ORDER], 24);
    assert_eq!(ram[HUD_INVENTORY_ORDER + 23], 1);
}

#[test]
fn native_hud_inventory_order_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut display = DisplayState::load_from_ram(&ram);

    {
        let mut bridge = NativeHudInventoryOrderBridgeMut::new(&mut display, &mut ram);
        bridge.initialize_default_order(24);
        bridge.swap_items(1, 22);
    }

    assert_eq!(display.hud_inventory_order.item(0), 1);
    assert_eq!(display.hud_inventory_order.item(1), 23);
    assert_eq!(display.hud_inventory_order.item(22), 2);
    assert_eq!(read_le_u16(&ram, HUD_INVENTORY_ORDER), 0x1701);
    assert_eq!(ram[HUD_INVENTORY_ORDER + 22], 2);
}

#[test]
fn hud_runtime_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[SUPER_BOMB_INDICATOR_TIMER] = 7;
    ram[SUPER_BOMB_INDICATOR_COUNTER] = 2;
    ram[RUPEE_SFX_SOUND_DELAY] = 5;
    ram[IS_DOING_HEART_ANIMATION] = 1;
    ram[HEART_REFILL_COUNTDOWN] = 6;
    ram[HEART_REFILL_ANIM_SUBPOS] = 0x80;
    ram[FLASHING_CIRCLE_TIMER] = 0x10;
    ram[MENU_PREV_JOYPAD_H] = 0x40;
    ram[EQUIPMENT_MENU_EXIT_STATE] = 3;
    ram[BOTTLE_MENU_ROW] = 9;
    ram[HUD_MODULE_TICK_COUNTER] = 0x33;

    let runtime = HudRuntimeState::load_from_ram(&ram);
    assert_eq!(runtime.super_bomb_indicator_timer(), 7);
    assert_eq!(runtime.super_bomb_indicator_counter(), 2);
    assert_eq!(runtime.rupee_sfx_sound_delay(), 5);
    assert!(runtime.is_doing_heart_animation());
    assert_eq!(runtime.is_doing_heart_animation_raw(), 1);
    assert_eq!(runtime.heart_refill_countdown(), 6);
    assert_eq!(runtime.heart_refill_anim_subpos(), 0x80);
    assert_eq!(runtime.flashing_circle_timer(), 0x10);
    assert_eq!(runtime.equipment_menu_exit_state(), 3);
    assert_eq!(runtime.bottle_menu_row(), 9);
    assert_eq!(runtime.tick_counter(), 0x33);

    let mut projected = vec![0; WRAM_SIZE];
    runtime.write_to_ram(&mut projected);
    assert_eq!(projected[SUPER_BOMB_INDICATOR_TIMER], 7);
    assert_eq!(projected[SUPER_BOMB_INDICATOR_COUNTER], 2);
    assert_eq!(projected[RUPEE_SFX_SOUND_DELAY], 5);
    assert_eq!(projected[IS_DOING_HEART_ANIMATION], 1);
    assert_eq!(projected[HEART_REFILL_COUNTDOWN], 6);
    assert_eq!(projected[HEART_REFILL_ANIM_SUBPOS], 0x80);
    assert_eq!(projected[FLASHING_CIRCLE_TIMER], 0x10);
    // $BD is ROM scratch shared with tile detection: never projected.
    assert_eq!(projected[MENU_PREV_JOYPAD_H], 0);
    assert_eq!(projected[EQUIPMENT_MENU_EXIT_STATE], 3);
    assert_eq!(projected[BOTTLE_MENU_ROW], 9);
    assert_eq!(projected[HUD_MODULE_TICK_COUNTER], 0x33);
}

#[test]
fn hud_runtime_state_owns_runtime_counter_behavior() {
    let mut runtime = HudRuntimeState::default();

    runtime.set_super_bomb_indicator_timer(8);
    runtime.set_super_bomb_indicator_counter(3);
    runtime.set_rupee_sfx_sound_delay(4);
    runtime.set_heart_animation_active(1);
    runtime.set_heart_refill_countdown(7);
    runtime.set_heart_refill_animation_subpixel(0x20);
    runtime.set_flashing_circle_timer(0x10);
    runtime.set_equipment_menu_exit_state(2);
    runtime.set_bottle_menu_row(5);
    assert_eq!(runtime.decrement_bottle_menu_row(), 4);
    runtime.set_tick_counter(0x44);
    runtime.clear_heart_animation_active();

    assert_eq!(runtime.super_bomb_indicator_timer(), 8);
    assert_eq!(runtime.super_bomb_indicator_counter(), 3);
    assert_eq!(runtime.rupee_sfx_sound_delay(), 4);
    assert!(!runtime.is_doing_heart_animation());
    assert_eq!(runtime.heart_refill_countdown(), 7);
    assert_eq!(runtime.heart_refill_anim_subpos(), 0x20);
    assert_eq!(runtime.flashing_circle_timer(), 0x10);
    assert_eq!(runtime.equipment_menu_exit_state(), 2);
    assert_eq!(runtime.bottle_menu_row(), 4);
    assert_eq!(runtime.tick_counter(), 0x44);
}

#[test]
fn hud_tilemap_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, HUD_FLOOR_CHANGED_TIMER, 0x1234);
    write_le_u16(&mut ram, HUD_TILE_INDICES_BUFFER + 4, 0xbeef);
    write_le_u16(&mut ram, MOVING_WALL_REPLACEMENT_BUFFER - 2, 0xabcd);

    let tilemap = HudTilemapState::load_from_ram(&ram);
    assert_eq!(tilemap.floor_changed_timer_low(), 0x34);
    assert_eq!(tilemap.tile_word(2), 0xbeef);
    assert_eq!(
        tilemap.tile_word((MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER) / 2 - 1),
        0xabcd
    );
    assert_eq!(
        tilemap.tile_word((MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER) / 2),
        0
    );

    let mut projected = vec![0; WRAM_SIZE];
    tilemap.write_to_ram(&mut projected);
    assert_eq!(HudTilemapState::load_from_ram(&projected), tilemap);
}

#[test]
fn native_hud_state_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[BOTTLE_MENU_ROW] = 5;
    ram[HUD_TILE_INDICES_BUFFER + 4] = 0x34;
    ram[HUD_TILE_INDICES_BUFFER + 5] = 0x12;
    let mut display = DisplayState::load_from_ram(&ram);

    {
        let mut bridge = NativeHudStateBridgeMut::new(&mut display, &mut ram);
        bridge.set_super_bomb_indicator_timer(8);
        bridge.set_super_bomb_indicator_counter(3);
        bridge.set_rupee_sfx_sound_delay(4);
        bridge.set_is_doing_heart_animation(1);
        bridge.set_heart_refill_countdown(7);
        bridge.set_heart_refill_anim_subpos(0x20);
        bridge.set_flashing_circle_timer(0x10);
        bridge.set_prev_joypad_h(0x80);
        bridge.set_equipment_menu_exit_state(2);
        assert_eq!(bridge.decrement_bottle_menu_row(), 4);
        bridge.set_tick_counter(0x44);
        bridge.set_floor_changed_timer(0x1234);
        bridge.clear_floor_changed_timer_low();
        bridge.set_tile_word(2, 0xbeef);
        bridge.clear_is_doing_heart_animation();
        bridge.clear_prev_joypad_h();
    }

    assert_eq!(display.hud_runtime.super_bomb_indicator_timer(), 8);
    assert_eq!(display.hud_runtime.super_bomb_indicator_counter(), 3);
    assert_eq!(display.hud_runtime.rupee_sfx_sound_delay(), 4);
    assert!(!display.hud_runtime.is_doing_heart_animation());
    assert_eq!(display.hud_runtime.heart_refill_countdown(), 7);
    assert_eq!(display.hud_runtime.heart_refill_anim_subpos(), 0x20);
    assert_eq!(display.hud_runtime.flashing_circle_timer(), 0x10);
    assert_eq!(display.hud_runtime.equipment_menu_exit_state(), 2);
    assert_eq!(display.hud_runtime.bottle_menu_row(), 4);
    assert_eq!(display.hud_runtime.tick_counter(), 0x44);
    assert_eq!(display.hud_tilemap.floor_changed_timer_low(), 0);
    assert_eq!(display.hud_tilemap.tile_word(2), 0xbeef);
    assert_eq!(ram[SUPER_BOMB_INDICATOR_TIMER], 8);
    assert_eq!(ram[SUPER_BOMB_INDICATOR_COUNTER], 3);
    assert_eq!(ram[RUPEE_SFX_SOUND_DELAY], 4);
    assert_eq!(ram[IS_DOING_HEART_ANIMATION], 0);
    assert_eq!(ram[MENU_PREV_JOYPAD_H], 0);
    assert_eq!(ram[HUD_FLOOR_CHANGED_TIMER], 0);
    assert_eq!(ram[HUD_FLOOR_CHANGED_TIMER + 1], 0x12);
    assert_eq!(read_le_u16(&ram, HUD_TILE_INDICES_BUFFER + 4), 0xbeef);
}

#[test]
fn native_vram_upload_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);

    let mut display = DisplayState::load_from_ram(&ram);
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
fn native_vram_upload_buffer_bridge_projects_native_cursor_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);
    let mut display = DisplayState {
        vram_upload_cursor: 0x1200,
        ..DisplayState::default()
    };

    {
        let mut bridge = NativeVramUploadBufferBridgeMut::new(&mut display, &mut ram);
        assert_eq!(bridge.advance_offset_by(0x30), 0x1230);
        bridge.clear_offset();
    }

    assert_eq!(display.vram_upload_cursor, 0);
    assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0);
}

#[test]
fn display_state_owns_vram_upload_cursor_and_counter_behavior() {
    let mut display = DisplayState {
        nmi_load_target_address: 0xab00,
        vram_upload_cursor: 0xfff0,
        incremental_vram_upload_counter: 0xfe,
        ..DisplayState::default()
    };

    display.set_nmi_load_target_page(0xcd);
    assert_eq!(display.nmi_load_target_address, 0xabcd);
    display.set_nmi_load_target_address(0x1234);
    assert_eq!(display.nmi_load_target_address, 0x1234);
    assert_eq!(display.nmi_load_target_page(), 0x34);

    assert_eq!(display.advance_vram_upload_cursor_by(0x20), 0x0010);
    display.set_vram_upload_cursor(0x1234);
    assert_eq!(
        display.apply_tilemap_upload_prefix_to_vram_cursor(&[0xab]),
        0x12ab
    );
    assert_eq!(
        display.apply_tilemap_upload_prefix_to_vram_cursor(&[0xcd, 0xef, 0x99]),
        0xefcd
    );
    display.clear_vram_upload_cursor();
    assert_eq!(display.vram_upload_cursor, 0);

    assert_eq!(display.increment_vram_upload_counter(), 0xff);
    assert_eq!(display.increment_vram_upload_counter(), 0);
    display.reset_incremental_vram_upload_counter();
    assert_eq!(display.incremental_vram_upload_counter, 0);
}

#[test]
fn display_state_owns_dma_and_upload_metadata_behavior() {
    let mut display = DisplayState {
        star_tile_restore_phase: 7,
        ..DisplayState::default()
    };

    display.set_link_dma_source(LinkDmaSourceSlot::BodyTop, 0x9000);
    assert_eq!(display.link_dma_source(LinkDmaSourceSlot::BodyTop), 0x9000);

    display.reset_bg_tile_animation_countdown(0xffff);
    assert_eq!(display.bg_tile_animation_countdown, 0xffff);

    display.set_message_dma_destination_address(0x6080);
    display.set_message_dma_tile_base(0x4841);
    display.set_message_dma_tile_limit(0x007f);
    display.set_message_dma_tile_sentinel(0xffff);
    assert_eq!(display.message_dma_destination_address_usize(), 0x6080);
    assert_eq!(display.message_dma_tile_base, 0x4841);
    assert_eq!(display.message_dma_tile_limit, 0x007f);
    assert_eq!(display.message_dma_tile_sentinel, 0xffff);

    display.set_travel_bird_tile_offset(0x08);
    assert!(display.has_travel_bird_tile_upload());

    display.clear_star_tile_restore_phase();
    assert_eq!(display.star_tile_restore_phase, 0);

    display.set_animated_tile_data_source_address(0xac80);
    display.set_animated_tile_vram_destination_address(0x3c00);
    assert_eq!(display.animated_tile_data_source_usize(), 0xac80);
    assert!(display.has_animated_tile_data_source());
    assert_eq!(display.animated_tile_vram_destination_usize(), 0x3c00);
}

#[test]
fn display_state_owns_basic_nmi_control_behavior() {
    let mut display = DisplayState {
        screen_brightness: 0xff,
        core_update_disable_flag: 0xff,
        pending_nmi_subroutine: 0x42,
        bg_vram_load_mode: 3,
        ..DisplayState::default()
    };

    assert_eq!(display.increment_screen_brightness(), 0);
    assert_eq!(display.decrement_screen_brightness(), 0xff);
    display.set_screen_brightness(0x80);
    assert_eq!(display.screen_brightness, 0x80);

    display.latch_nmi_update();
    assert!(display.nmi_update_is_latched());
    display.clear_nmi_update_latch();
    assert!(!display.nmi_update_is_latched());

    assert_eq!(display.increment_core_update_disable_flag(), 0);
    display.set_core_update_disable_flag_word(0x1234);
    assert_eq!(display.core_update_disable_flag, 0x34);
    display.clear_core_update_disable_flag();
    assert!(!display.core_updates_are_disabled());

    assert_eq!(display.take_pending_nmi_subroutine(), 0x42);
    assert_eq!(display.pending_nmi_subroutine, 0);
    display.clear_bg_vram_load_mode();
    assert!(!display.has_bg_vram_load());

    display.queue_tilemap_update(0x54, 0x0800);
    assert!(display.has_pending_tilemap_update());
    assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5400);
    assert_eq!(
        display.pending_tilemap_update_source_address(),
        crate::game_state::constants::nmi::BG_CHAR_BUFFER + 0x0800
    );
    display.clear_pending_tilemap_update_destination();
    assert!(!display.has_pending_tilemap_update());
    assert_eq!(display.pending_tilemap_update_source_offset, 0x0800);
}

#[test]
fn display_state_owns_layer_and_window_mask_behavior() {
    let mut display = DisplayState::default();

    display.set_bg_mode(9);
    display.set_layer_masks_word(0x0211);
    display.and_main_screen_layers(0x0f);
    display.or_main_screen_layers(0x80);
    display.and_sub_screen_layers(0x01);
    display.or_sub_screen_layers(0x40);
    display.set_window_layer_masks(0x10, 0x20, 0x30, 0x40, 0x50);

    assert_eq!(display.bg_mode, 9);
    assert_eq!(display.main_screen_layers, 0x81);
    assert_eq!(display.sub_screen_layers, 0x40);
    assert_eq!(display.layer_masks_word(), 0x4081);
    assert_eq!(display.bg12_window_selection, 0x10);
    assert_eq!(display.bg34_window_selection, 0x20);
    assert_eq!(display.object_color_window_selection, 0x30);
    assert_eq!(display.main_screen_window_layers, 0x40);
    assert_eq!(display.sub_screen_window_layers, 0x50);

    display.clear_window_main_sub_masks();
    assert_eq!(display.main_screen_window_layers, 0);
    assert_eq!(display.sub_screen_window_layers, 0);
    display.clear_window_layer_masks();
    assert_eq!(display.bg12_window_selection, 0);
    assert_eq!(display.bg34_window_selection, 0);
    assert_eq!(display.object_color_window_selection, 0);

    display.set_sub_screen_layers(0x77);
    display.set_main_screen_window_layers(0x88);
    display.clear_sub_screen_layers_word_alias();
    assert_eq!(display.sub_screen_layers, 0);
    assert_eq!(display.main_screen_window_layers, 0);
}

#[test]
fn display_state_owns_nmi_request_thread_irq_and_hdma_behavior() {
    let mut display = DisplayState {
        chr_halfslot_request: 0xff,
        irq_control_flag: 0x80,
        hdma_enable_mask: 0xc0,
        ..DisplayState::default()
    };

    display.request_nmi_copy_packets();
    assert!(display.has_nmi_copy_packets_request());
    display.clear_nmi_copy_packets_request();
    assert!(!display.has_nmi_copy_packets_request());
    display.set_nmi_copy_packets_request(3);
    assert_eq!(display.nmi_copy_packets_request, 3);

    display.request_polyhedral_nmi_update();
    assert!(display.has_pending_polyhedral_update());
    display.clear_pending_polyhedral_update();
    assert!(!display.has_pending_polyhedral_update());

    assert_eq!(display.increment_chr_halfslot_request(), 0);
    display.set_chr_halfslot_request(12);
    assert!(display.has_chr_halfslot_request());
    display.clear_chr_halfslot_request();
    assert!(!display.has_chr_halfslot_request());

    display.activate_nmi_thread();
    display.set_nmi_thread_stack_pointer(0x1f31);
    assert!(!display.nmi_thread_uses_poly_stack());
    display.set_nmi_thread_stack_pointer(0x1f30);
    assert!(display.nmi_thread_uses_poly_stack());
    display.deactivate_nmi_thread();
    assert!(!display.nmi_thread_uses_poly_stack());

    assert!(display.has_irq_control_flag());
    assert!(display.irq_control_has_vcounter_marker());
    display.clear_irq_control_flag();
    assert!(!display.has_irq_control_flag());
    display.set_irq_control_flag(0x7f);
    assert!(!display.irq_control_has_vcounter_marker());
    display.set_vertical_irq_trigger(0x70);
    assert_eq!(display.vertical_irq_trigger, 0x70);
    display.crystal_rotation_counter = 0xf0;
    assert!(display.advance_crystal_rotation_counter(0x20));
    assert_eq!(display.crystal_rotation_counter, 0x10);

    display.set_sprite_dma_head_pointer(0x40);
    display.set_sprite_dma_body_pointer(0x80);
    assert_eq!(display.sprite_dma_head_pointer, 0x40);
    assert_eq!(display.sprite_dma_body_pointer, 0x80);

    assert!(display.is_hdma_channel_enabled(6));
    assert!(display.is_hdma_channel_enabled(7));
    display.clear_hdma_enable_mask();
    assert!(!display.is_hdma_channel_enabled(7));
    display.set_hdma_enable_mask(0x80);
    assert!(display.is_hdma_channel_enabled(7));
}

#[test]
fn display_state_owns_mosaic_control_behavior() {
    let mut display = DisplayState {
        mosaic_level: 0xf0,
        mosaic_target_level: 0xff,
        mosaic_direction: 1,
        ..DisplayState::default()
    };

    assert_eq!(display.increment_mosaic_level_by(0x20), 0x10);
    assert_eq!(display.decrement_mosaic_level_by(0x30), 0xe0);
    display.set_mosaic_copy_from_level_or(0x03);
    assert_eq!(display.mosaic_copy, 0xe3);
    display.set_mosaic_copy(0x40);
    assert_eq!(display.mosaic_copy, 0x40);

    display.set_mosaic_target_level_word(0x1234);
    assert_eq!(display.mosaic_target_level, 0x34);
    assert_eq!(display.mosaic_target_level_word(), 0x34);
    display.clear_mosaic_target_level_word_alias();
    assert_eq!(display.mosaic_target_level, 0);

    display.clear_mosaic_level_word_alias();
    assert_eq!(display.mosaic_level, 0);
    display.set_mosaic_direction(1);
    display.clear_mosaic_direction();
    assert_eq!(display.mosaic_direction, 0);
}

#[test]
fn native_display_main_layer_setters_ignore_transient_subscreen_ram_lag() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[TS_COPY] = 0;

    let mut display = DisplayState::load_from_ram(&ram);
    display.sub_screen_layers = 1;

    {
        let mut bridge = NativeDisplayStateBridgeMut::new(&mut display, &mut ram);
        bridge.set_main_screen_layers(0x16);
        bridge.and_main_screen_layers(0x17);
        bridge.or_main_screen_layers(0x01);
    }

    assert_eq!(display.main_screen_layers, 0x17);
    assert_eq!(display.sub_screen_layers, 1);
    assert_eq!(ram[TM_COPY], 0x17);
    assert_eq!(ram[TS_COPY], 0);
}

#[test]
fn display_core_coherence_ignores_gameplay_velocity_reuse_of_attract_vram_dst() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x0100);
    let mut display = DisplayState::load_from_ram(&ram);
    display.attract_vram_destination_address = 0;

    display.debug_assert_core_matches_ram(&ram);
}

#[test]
fn display_core_coherence_ignores_indoor_moving_wall_reuse_of_star_phase() {
    let mut ram = vec![0; WRAM_SIZE];
    ram[PLAYER_IS_INDOORS] = 1;
    ram[STAR_TILE_RESTORE_PHASE] = 1;
    let mut display = DisplayState::load_from_ram(&ram);
    display.star_tile_restore_phase = 0;

    display.debug_assert_core_matches_ram(&ram);
}

#[test]
fn display_core_coherence_ignores_tilemap_buffer_reuse_of_vram_cursor() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x3c15);
    let mut display = DisplayState::load_from_ram(&ram);
    display.vram_upload_cursor = 0;

    display.debug_assert_core_matches_ram(&ram);
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
    ram[CRYSTAL_ROTATION_COUNTER] = 0xf0;
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
    write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_DST_ADDR, 0x6040);
    ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = 0x20;
    ram[FLAG_TRAVEL_BIRD] = 0x04;
    ram[STAR_TILE_RESTORE_PHASE] = 7;
    write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
    write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);

    let mut display = DisplayState::load_from_ram(&ram);
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
        assert!(bridge.advance_crystal_rotation_counter(0x20));
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
    assert_eq!(display.crystal_rotation_counter, 0x10);
    assert_eq!(ram[CRYSTAL_ROTATION_COUNTER], 0x10);
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
    assert_eq!(
        read_le_u16(&ram, messaging_constants::MESSAGE_DMA_DST_ADDR),
        0x6080
    );
    assert_eq!(ram[FLAG_TRAVEL_BIRD], 0x08);
    assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
    assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
}

#[test]
fn ppu_scroll_copy_state_loads_from_and_projects_to_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, BG1_H_SCROLL_COPY, 0x1234);
    write_le_u16(&mut ram, BG2_X_SCROLL, 0x0100);
    write_le_u16(&mut ram, BG2_Y_SCROLL, 0x0200);
    write_le_u16(&mut ram, MAPBAK_CGWSEL, 0xabcd);
    ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4].copy_from_slice(&[1, 2, 3, 4]);

    let mut scroll = PpuScrollCopyState::load_from_ram(&ram);
    assert_eq!(scroll.bg1_h_copy(), 0x1234);
    assert_eq!(scroll.bg1_h_copy_low(), 0x34);
    assert_eq!(scroll.bg2_h_copy2(), 0x0100);
    assert_eq!(scroll.bg2_v_copy2(), 0x0200);
    assert_eq!(scroll.mapbak_cgwsel_word(), 0xabcd);
    assert_eq!(&scroll.mapbak_palette_slice()[..4], &[1, 2, 3, 4]);

    scroll.add_bg2_h_copy2(0x10);
    scroll.add_bg2_copy2_for_axis_signed(true, -1);
    scroll.set_mapbak_cgwsel(0x55);
    scroll.copy_mapbak_palette_from(&[9, 8, 7]);
    scroll.write_to_ram(&mut ram);

    assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x0110);
    assert_eq!(read_le_u16(&ram, BG2_Y_SCROLL), 0x01ff);
    assert_eq!(read_le_u16(&ram, MAPBAK_CGWSEL), 0xab55);
    // copy_mapbak_palette_from updates the native field, but write_to_ram does NOT
    // project mapbak_palette (it is write-through via the bridge to avoid scroll-sync
    // clobbering a palette backup — f335672), so RAM[MAPBAK_PALETTE] is left untouched.
    assert_eq!(&scroll.mapbak_palette_slice()[..4], &[9, 8, 7, 4]);
    assert_eq!(&ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4], &[1, 2, 3, 4]);
}

#[test]
fn native_ppu_scroll_copy_bridge_syncs_seeded_ram_and_dual_writes_changes() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, BG2_X_SCROLL, 0x0060);
    write_le_u16(&mut ram, BG2_Y_SCROLL, 0x0070);
    write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0x1001);
    write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0x2002);
    ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4].copy_from_slice(&[1, 2, 3, 4]);

    let mut scroll = PpuScrollCopyState::load_from_ram(&ram);
    {
        let mut bridge = NativePpuScrollCopyBridgeMut::new(&mut scroll, &mut ram);
        bridge.cache_bg2_live_scroll();
        bridge.copy_bg2_live_to_bg1_live();
    }

    assert_eq!(scroll.bg1_h_copy2(), 0x0060);
    assert_eq!(scroll.bg1_v_copy2(), 0x0070);
    assert_eq!(read_le_u16(&ram, BG1_X_SCROLL), 0x0060);
    assert_eq!(read_le_u16(&ram, BG1_Y_SCROLL), 0x0070);
    assert_eq!(read_le_u16(&ram, BG2_H_SCROLL_COPY2_CACHED), 0x0060);
    assert_eq!(read_le_u16(&ram, BG2_V_SCROLL_COPY2_CACHED), 0x0070);
    // mapbak_palette is no longer projected by the scroll-copy sync (write-through via
    // the main bridge to avoid scroll-sync clobbering a palette backup — f335672), so a
    // bridge scroll sync leaves RAM[MAPBAK_PALETTE] untouched.
    assert_eq!(&ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4], &[1, 2, 3, 4]);
}

#[test]
fn native_ppu_scroll_copy_bridge_projects_native_state_over_stale_ram() {
    let mut ram = vec![0; WRAM_SIZE];
    write_le_u16(&mut ram, BG2_X_SCROLL, 0x0060);
    write_le_u16(&mut ram, MAPBAK_CGWSEL, 0x1234);
    let mut scroll = PpuScrollCopyState::default();
    scroll.set_bg2_h_copy2(0x2200);
    scroll.set_mapbak_cgwsel_word(0x5678);

    {
        let mut bridge = NativePpuScrollCopyBridgeMut::new(&mut scroll, &mut ram);
        bridge.add_bg2_h_copy2(0x10);
    }

    assert_eq!(scroll.bg2_h_copy2(), 0x2210);
    assert_eq!(scroll.mapbak_cgwsel_word(), 0x5678);
    assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x2210);
    assert_eq!(read_le_u16(&ram, MAPBAK_CGWSEL), 0x5678);
}

#[test]
fn native_ppu_scroll_copy_bridge_keeps_mapbak_tm_word_high_byte_in_sync_with_ts() {
    let mut ram = vec![0; WRAM_SIZE];
    let mut scroll = PpuScrollCopyState::default();

    {
        let mut bridge = NativePpuScrollCopyBridgeMut::new(&mut scroll, &mut ram);
        bridge.set_mapbak_ts(0x01);
        bridge.set_mapbak_tm(0x16);
    }

    assert_eq!(scroll.mapbak_tm(), 0x16);
    assert_eq!(scroll.mapbak_ts(), 0x01);
    assert_eq!(scroll.mapbak_tm_word(), 0x0116);
    assert_eq!(read_le_u16(&ram, MAPBAK_TM), 0x0116);
    assert_eq!(ram[MAPBAK_TS], 0x01);
}
