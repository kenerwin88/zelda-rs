use super::*;
use crate::dialogue_ir::{
    DialogueIrKind, TEXT_CMD_2, TEXT_CMD_COLOR, TEXT_CMD_END_MESSAGE, TEXT_CMD_NAME,
    TEXT_CMD_NUMBER, TEXT_CMD_WAIT, TEXT_COMMAND_START_US,
};
use crate::game_state::constants::{
    ANCILLA_TYPE, ANCILLA_X_LO, ANCILLA_X_VELOCITY, ANIMATED_TILE_DATA_SRC,
    BG_TILE_ANIMATION_COUNTDOWN, DIALOGUE_MESSAGE_INDEX, DMA_SOURCE_ADDR_0, DMA_SOURCE_ADDR_1,
    DMA_SOURCE_ADDR_10, DMA_SOURCE_ADDR_11, DMA_SOURCE_ADDR_12, DMA_SOURCE_ADDR_13,
    DMA_SOURCE_ADDR_14, DMA_SOURCE_ADDR_15, DMA_SOURCE_ADDR_16, DMA_SOURCE_ADDR_17,
    DMA_SOURCE_ADDR_18, DMA_SOURCE_ADDR_19, DMA_SOURCE_ADDR_2, DMA_SOURCE_ADDR_20,
    DMA_SOURCE_ADDR_21, DMA_SOURCE_ADDR_3, DMA_SOURCE_ADDR_4, DMA_SOURCE_ADDR_5, DMA_SOURCE_ADDR_6,
    DMA_SOURCE_ADDR_7, DMA_SOURCE_ADDR_8, DMA_SOURCE_ADDR_9, TM_COPY, TS_COPY,
};
use crate::game_state::constants::{MAP16_LOAD_DST_OFF, MAP16_LOAD_SRC_OFF, MAP16_LOAD_Y_UNIT};

fn test_sync_all(state: &mut ZeldaState) {
    state.ram[0x42] = state.ram[0x42].wrapping_add(1);
}

fn link_test_byte(state: &ZeldaState, addr: usize) -> u8 {
    state.ram[addr]
}

fn set_link_test_byte(state: &mut ZeldaState, addr: usize, value: u8) {
    state.ram[addr] = value;
}

fn link_test_word(state: &ZeldaState, addr: usize) -> u16 {
    read_le_u16(&state.ram, addr)
}

fn set_link_test_word(state: &mut ZeldaState, addr: usize, value: u16) {
    write_le_u16(&mut state.ram, addr, value);
}

fn put_test_asset(data: &mut Vec<u8>, ranges: &mut [(usize, usize)], index: usize, bytes: Vec<u8>) {
    let start = data.len();
    data.extend(bytes);
    ranges[index] = (start, data.len());
}

fn pack_test_memblk_arrays(items: &[Vec<u8>]) -> Vec<u8> {
    assert!(!items.is_empty());
    let payload_before_last = items[..items.len() - 1].iter().map(Vec::len).sum::<usize>();
    let wide_offsets = payload_before_last >= 65536;
    let mut data = Vec::new();
    let mut offset = 0usize;
    for item in &items[..items.len() - 1] {
        offset += item.len();
        if wide_offsets {
            data.extend_from_slice(&(offset as u32).to_le_bytes());
        } else {
            data.extend_from_slice(&(offset as u16).to_le_bytes());
        }
    }
    for item in items {
        data.extend_from_slice(item);
    }
    let marker = if wide_offsets {
        8192 + items.len() - 1
    } else {
        items.len() - 1
    };
    data.extend_from_slice(&(marker as u16).to_le_bytes());
    data
}

fn dialogue_source_sidecar_asset(messages: &[Vec<u8>]) -> Vec<u8> {
    let state = ZeldaState::new();
    let table = messages
        .iter()
        .map(|message| state.dialogue_ir_for_decoded_bytes(message))
        .collect::<Vec<_>>();
    let mut asset = DIALOGUE_SOURCE_SIDECAR_MAGIC.to_vec();
    asset.extend(bincode::serialize(&table).unwrap());
    asset
}

fn asset_pack_with_named_assets(
    data: Vec<u8>,
    ranges: Vec<(usize, usize)>,
    named_assets: &[(usize, &str)],
) -> AssetPack {
    let mut names = vec![String::new(); ranges.len()];
    for &(index, name) in named_assets {
        names[index] = name.to_string();
    }
    AssetPack::from_named_data_ranges(data, ranges, names)
}

fn test_asset_pack_bytes(named_assets: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut key_signature = Vec::new();
    for &(name, _) in named_assets {
        key_signature.extend_from_slice(name.as_bytes());
        key_signature.push(0);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ASSET_SIGNATURE_PREFIX);
    bytes.extend_from_slice(&[0; 64]);
    bytes.extend_from_slice(&(named_assets.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(key_signature.len() as u32).to_le_bytes());
    for (_, payload) in named_assets {
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    }
    bytes.extend_from_slice(&key_signature);
    for (_, payload) in named_assets {
        while bytes.len() & 3 != 0 {
            bytes.push(0);
        }
        bytes.extend_from_slice(payload);
    }
    bytes
}

fn probe_entrance_asset_pack(entrance_index: usize, room: u16) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 56];
    let byte_len = entrance_index + 1;
    let word_len = (entrance_index + 1) * 2;

    let mut rooms = vec![0; word_len];
    write_le_u16(&mut rooms, entrance_index * 2, room);
    put_test_asset(&mut data, &mut ranges, 11, rooms);

    put_test_asset(&mut data, &mut ranges, 12, vec![0; byte_len * 8]);
    for asset in [13, 14, 15, 16, 17, 18, 26] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; word_len]);
    }
    for asset in [19, 20, 21, 22, 23, 24, 25, 27] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; byte_len]);
    }
    put_test_asset(&mut data, &mut ranges, 53, Vec::new());
    put_test_asset(&mut data, &mut ranges, 54, vec![0; 116]);
    put_test_asset(&mut data, &mut ranges, 55, Vec::new());

    AssetPack::from_data_ranges(data, ranges)
}

fn probe_overworld_asset_pack(screen: usize) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 109];
    put_test_asset(&mut data, &mut ranges, 107, vec![1; screen + 1]);
    put_test_asset(&mut data, &mut ranges, 108, vec![0; screen + 1]);
    AssetPack::from_data_ranges(data, ranges)
}

#[test]
fn ancilla_slot_accessors_keep_native_state_and_ram_synced() {
    let mut state = ZeldaState::new();

    state.ancilla_slot_view_mut(3).set_x_velocity(0x80);
    state.ancilla_slot_view_mut(3).set_x_low(0x44);
    state.ancilla_slot_view_mut(0).increment_ancilla_type();

    assert_eq!(state.ancilla_slot_view(3).x_velocity(), 0x80);
    assert_eq!(state.ancilla_slot_view(3).x_low(), 0x44);
    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 1);
    assert_eq!(state.ram[ANCILLA_X_VELOCITY + 3], 0x80);
    assert_eq!(state.ram[ANCILLA_X_LO + 3], 0x44);
    assert_eq!(state.ram[ANCILLA_TYPE], 1);
}

#[test]
fn owns_oracle_compared_memory_regions() {
    let state = ZeldaState::new();
    assert_eq!(state.ram.len(), WRAM_SIZE);
    assert_eq!(state.sram.len(), SRAM_SIZE);
    assert_eq!(state.vram().len(), VRAM_WORDS);
}

#[test]
fn bg3_vwf_glyph_runs_track_unaligned_glyphs_and_scroll() {
    let mut state = ZeldaState::new();
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[0, 1, 2, 0x78, 3]);
    for i in 0..126 {
        state.set_vwf_tile_word_at_byte_offset(i * 2, 0x3980 + i as u16);
    }

    state.record_bg3_vwf_glyph_run(1, 7, 0, 8, 1);
    assert_eq!(
        state.bg3_vwf_glyph_runs(),
        &[Bg3VwfGlyphRun {
            glyph_code: 1,
            origin_tile_number: 0x180,
            x: 7,
            y: 0,
            width: 8,
        }]
    );
    assert_eq!(state.bg3_vwf_glyph_run_dialogue_offsets(), &[1]);
    assert_eq!(
        state.bg3_vwf_glyph_run_dialogue_ir(0).map(|op| op.kind),
        Some(DialogueIrKind::Glyph { code: 1 })
    );
    state.scroll_bg3_vwf_glyph_runs_up_one_pixel();
    assert_eq!(state.bg3_vwf_glyph_runs()[0].y, -1);
    assert_eq!(state.bg3_vwf_glyph_run_dialogue_offsets(), &[1]);

    state.clear_bg3_vwf_glyph_runs();
    assert!(state.bg3_vwf_glyph_runs().is_empty());
    assert!(state.bg3_vwf_glyph_run_dialogue_offsets().is_empty());
}

#[test]
fn dialogue_ir_for_decoded_bytes_uses_runtime_dialogue_flags() {
    let state = ZeldaState::new();

    let ops = state.dialogue_ir_for_decoded_bytes(&[
        0,
        TEXT_COMMAND_START_US + TEXT_CMD_2,
        TEXT_COMMAND_START_US + TEXT_CMD_WAIT,
        3,
    ]);

    assert_eq!(ops[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(ops[1].kind, DialogueIrKind::Line { line: 2 });
    assert_eq!(ops[2].kind, DialogueIrKind::Wait { duration: 3 });
}

#[test]
fn current_dialogue_message_id_setter_updates_native_state_and_ram() {
    let mut state = ZeldaState::new();

    state.set_current_dialogue_message_id(0x00c8);

    assert_eq!(state.current_dialogue_message_id(), 0x00c8);
    assert_eq!(read_le_u16(&state.ram, DIALOGUE_MESSAGE_INDEX), 0x00c8);
}

#[test]
fn source_dialogue_ir_uses_authored_message_before_runtime_substitution() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            0,
            TEXT_COMMAND_START_US + TEXT_CMD_NAME,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state.messaging_text_mut().load_decoded_dialogue(&[
        0,
        1,
        TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
    ]);

    let source_ir = state.current_source_dialogue_ir();
    let rendered_ir = state.current_dialogue_ir();

    assert_eq!(state.current_dialogue_message_id(), 0);
    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::PlayerName);
    assert_eq!(source_ir[2].kind, DialogueIrKind::EndMessage);
    assert_eq!(rendered_ir[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(rendered_ir[1].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(rendered_ir[2].kind, DialogueIrKind::EndMessage);
}

#[test]
fn source_dialogue_ir_requires_named_semantic_sidecar_asset() {
    let legacy_dictionary = pack_test_memblk_arrays(&[vec![]]);
    let legacy_messages =
        pack_test_memblk_arrays(&[vec![0, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]);
    let legacy_asset = pack_test_memblk_arrays(&[legacy_dictionary, legacy_messages]);
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(&mut data, &mut ranges, 94, legacy_asset);
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    let source_ir = state.current_source_dialogue_ir();

    assert!(source_ir.is_empty());
}

#[test]
fn source_dialogue_ir_reads_named_semantic_sidecar_asset() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));

    let source_ir = state.current_source_dialogue_ir();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::EndMessage);
}

#[test]
fn asset_pack_parse_requires_named_dialogue_semantic_sidecar_when_kdialogue_exists() {
    let err = AssetPack::parse(&test_asset_pack_bytes(&[("kDialogue", vec![0])]))
        .err()
        .unwrap();

    assert!(err.contains("missing required kDialogueSourceSemantic"));
}

#[test]
fn asset_pack_parse_rejects_malformed_named_dialogue_semantic_sidecar() {
    let err = AssetPack::parse(&test_asset_pack_bytes(&[
        ("kDialogue", vec![0]),
        ("kDialogueSourceSemantic", b"not semantic".to_vec()),
    ]))
    .err()
    .unwrap();

    assert!(err.contains("invalid semantic sidecar magic"));
}

#[test]
fn source_render_dialogue_ir_expands_runtime_number_commands() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            TEXT_COMMAND_START_US + TEXT_CMD_NUMBER,
            1,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state
        .game_state
        .messaging
        .dialogue_number
        .set_packed_digits(0x42, 0);

    let source_ir = state.current_source_dialogue_ir();
    let render_ir = state.current_source_render_dialogue_ir();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Number { slot: 1 });
    assert_eq!(render_ir[0].kind, DialogueIrKind::Glyph { code: 0x38 });
    assert_eq!(render_ir[1].kind, DialogueIrKind::EndMessage);
}

#[test]
fn visible_source_render_dialogue_ir_tracks_live_read_position() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            TEXT_COMMAND_START_US + TEXT_CMD_COLOR,
            2,
            0,
            1,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state.messaging_text_mut().load_decoded_dialogue(&[
        0,
        1,
        TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
    ]);
    state.messaging_state_mut().set_dialogue_msg_read_pos(1);

    let render_ir = state.current_visible_source_render_dialogue_ir();
    let visible_kinds = render_ir.iter().map(|op| &op.kind).collect::<Vec<_>>();

    assert_eq!(
        visible_kinds,
        vec![
            &DialogueIrKind::Color { color: 2 },
            &DialogueIrKind::Glyph { code: 0 },
        ]
    );
    assert_eq!(
        render_ir.iter().map(|op| op.offset).collect::<Vec<_>>(),
        vec![0, 0]
    );
}

#[test]
fn scratch_word_high_does_not_alias_nmi_subroutine_index() {
    let mut state = ZeldaState::new();
    state.scratch_word_mut().set_word(0x0200);
    state.set_pending_nmi_subroutine(11);

    assert_eq!(state.scratch_word_mut().decrement_high(), 1);

    assert_eq!(state.game_state.dungeon.scratch_word.word(), 0x0100);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
}

#[test]
fn screen_layer_helpers_keep_world_transient_layer_copy_coherent() {
    let mut state = ZeldaState::new();
    state.ram[TM_COPY] = 0x15;
    state.ram[TS_COPY] = 0x00;
    state.sync_native_game_state_from_ram();

    state.set_main_screen_layers(0x16);
    state.set_sub_screen_layers(0x01);
    state.set_quadrant_fullsize_x(2);

    assert_eq!(state.game_state.display.main_screen_layers, 0x16);
    assert_eq!(state.game_state.display.sub_screen_layers, 0x01);
    assert_eq!(state.game_state.world.transient.tilemap_layer_copy, 0x0116);
    assert_eq!(state.ram[TM_COPY], 0x16);
    assert_eq!(state.ram[TS_COPY], 0x01);
    state.assert_native_display_state_matches_ram();
}

#[test]
fn reset_can_preserve_sram() {
    let mut state = ZeldaState::new();
    state.ram[1] = 1;
    state.sram[1] = 2;
    state.vram_mut()[1] = 3;

    state.reset(true);

    assert_eq!(state.ram[1], 0);
    assert_eq!(state.sram[1], 2);
    assert_eq!(state.vram()[1], 0);
}

#[test]
fn sram_path_prefers_explicit_save_dir() {
    assert_eq!(
        ZeldaState::sram_path_from_env(
            Some("/tmp/z3-save".into()),
            Some("/tmp/xdg".into()),
            Some("/tmp/home".into()),
        ),
        PathBuf::from("/tmp/z3-save/sram.dat")
    );
}

#[test]
fn sram_path_uses_xdg_data_home_for_deck_safe_default() {
    assert_eq!(
        ZeldaState::sram_path_from_env(None, Some("/tmp/xdg".into()), Some("/tmp/home".into())),
        PathBuf::from("/tmp/xdg/zelda3-rs/saves/sram.dat")
    );
}

#[test]
fn sram_path_falls_back_to_home_data_dir() {
    assert_eq!(
        ZeldaState::sram_path_from_env(None, None, Some("/tmp/home".into())),
        PathBuf::from("/tmp/home/.local/share/zelda3-rs/saves/sram.dat")
    );
}

#[test]
fn rom_palette_words_fall_back_to_generated_assets_without_rom() {
    let mut state = ZeldaState::new();
    let ranges: [(u32, usize, u16); 14] = [
        (PALETTE_DUNGEON_BG_MAIN_SNES_ADDR, 79, 0x1111),
        (PALETTE_MAIN_SPRITE_SNES_ADDR, 80, 0x2222),
        (PALETTE_ARMOR_AND_GLOVES_SNES_ADDR, 81, 0x3333),
        (PALETTE_SWORD_SNES_ADDR, 82, 0x4444),
        (PALETTE_SHIELD_SNES_ADDR, 83, 0x5555),
        (PALETTE_SPRITE_AUX3_SNES_ADDR, 84, 0x6666),
        (PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR, 85, 0x7777),
        (PALETTE_SPRITE_AUX1_SNES_ADDR, 86, 0x8888),
        (PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR, 87, 0x9999),
        (PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 88, 0xaaaa),
        (PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR, 89, 0xbbbb),
        (PALETTE_PALACE_MAP_BG_SNES_ADDR, 90, 0xcccc),
        (PALETTE_PALACE_MAP_SPRITE_SNES_ADDR, 91, 0xdddd),
        (HUD_PALETTE_SNES_ADDR, 92, 0xeeee),
    ];
    let mut data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 93];
    for &(_, asset, value) in &ranges {
        let start = data.len();
        data.extend_from_slice(&value.to_le_bytes());
        data.extend_from_slice(&(value ^ 0xffff).to_le_bytes());
        asset_ranges[asset] = (start, data.len());
    }
    state.assets = Some(AssetPack::from_data_ranges(data, asset_ranges));

    for &(base, _, value) in &ranges {
        assert_eq!(state.rom_or_asset_word_snes(base), Some(value));
        assert_eq!(state.rom_or_asset_word_snes(base + 2), Some(value ^ 0xffff));
    }
}

#[test]
fn parity_probe_direct_entrance_loads_room_from_entrance_assets() {
    let mut state = ZeldaState::new();
    state.assets = Some(probe_entrance_asset_pack(0x2a, 0x0122));

    let room = state.parity_probe_direct_entrance(0x2a);

    assert_eq!(room, 0x0122);
    assert_eq!(read_le_u16(&state.ram, 0x048e), 0x0122);
    assert_eq!(state.ram[0x001b], 1);
}

#[test]
fn parity_probe_overworld_screen_loads_screen_properties() {
    let mut state = ZeldaState::new();
    state.assets = Some(probe_overworld_asset_pack(0x005a));
    state.set_indoor_flag(1);

    let screen = state.parity_probe_overworld_screen(0x005a);

    assert_eq!(screen, 0x005a);
    assert_eq!(read_le_u16(&state.ram, 0x008a), 0x005a);
    assert_eq!(state.ram[0x001b], 0);
}

#[test]
fn parity_probe_dungeon_room_marks_room_as_indoor_surface() {
    let mut state = ZeldaState::new();

    let room = state.parity_probe_dungeon_room(0x002d);

    assert_eq!(room, 0x002d);
    assert_eq!(read_le_u16(&state.ram, 0x048e), 0x002d);
    assert_eq!(read_le_u16(&state.ram, 0x00a0), 0x002d);
    assert_eq!(state.ram[0x001b], 1);
}

#[test]
fn player_layer_collision_helpers_preserve_unrelated_flags() {
    let mut state = ZeldaState::new();
    state.set_player_layer_collision_flags(0xf0);

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG1,
        true,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf1
    );
    assert!(!state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG2,
        true,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf3
    );
    assert!(state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG1,
        false,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf2
    );
    assert!(!state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));
}

#[test]
fn intro_background_settings_write_ppu_tilemap_regs() {
    let mut state = ZeldaState::new();
    state.ppu.bg_layer[0].tilemap_adr = 0;
    state.ppu.bg_layer[1].tilemap_adr = 0;
    state.ppu.bg_layer[2].tilemap_adr = 0;

    state.intro_initialize_background_settings();

    assert_eq!(state.game_state.display.bg_mode, 9);
    assert_eq!(state.game_state.display.mosaic_copy, 0);
    assert_eq!(state.ppu.bg_layer[0].tilemap_adr, 0x1000);
    assert!(state.ppu.bg_layer[0].tilemap_wider);
    assert!(state.ppu.bg_layer[0].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0);
    assert!(state.ppu.bg_layer[1].tilemap_wider);
    assert!(state.ppu.bg_layer[1].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[2].tilemap_adr, 0x6000);
    assert!(state.ppu.bg_layer[2].tilemap_wider);
    assert!(state.ppu.bg_layer[2].tilemap_higher);
}

#[test]
fn triforce_poly_step0_falls_through_once_like_c() {
    let mut state = ZeldaState::new();
    state.attract_scene_mut().set_intro_step_index(0);
    state.poly_runtime_mut().set_config1(10);
    state.set_subsubmodule(8);
    state.poly_runtime_mut().set_angle_a(7);
    state.poly_runtime_mut().set_angle_b(11);

    state.triforce_room_handle_poly();

    assert_eq!(state.game_state.poly.runtime.config1(), 8);
    assert_eq!(state.game_state.ending.attract_scene.intro_step_index(), 0);
    assert_eq!(state.game_state.frame.subsubmodule, 8);
    assert_eq!(state.game_state.poly.runtime.angle_a(), 8);
    assert_eq!(state.game_state.poly.runtime.angle_b(), 13);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
    assert_eq!(state.ram[0x1e02], 0);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_frame_counter(),
        1
    );
}

#[test]
fn credits_module_sets_oam_region_words_like_c() {
    let mut state = ZeldaState::new();
    state.set_submodule(38);
    for offset in 0..6 {
        state.ram[0x0fe0 + offset] = 0xff;
    }

    state.module1_a_credits();

    assert_eq!(read_le_u16(&state.ram, 0x0fe0), 0x0030);
    assert_eq!(read_le_u16(&state.ram, 0x0fe2), 0x01d0);
    assert_eq!(read_le_u16(&state.ram, 0x0fe4), 0x0000);
}

#[test]
fn credits_prep_resets_sprite_properties_before_scene_setup() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    let k = 15;
    for base in [
        SPRITE_PAUSE,
        SPRITE_E,
        SPRITE_X_VEL,
        SPRITE_Y_VEL,
        SPRITE_AI_STATE,
        SPRITE_A,
        SPRITE_DELAY_MAIN,
        SPRITE_OAM_FLAGS,
        SPRITE_STATE,
        SPRITE_FLAGS5,
        SPRITE_DEFL_BITS,
    ] {
        state.ram[base + k] = 0xa5;
    }

    state.credits_prep_and_load_sprites();

    for base in [
        SPRITE_PAUSE,
        SPRITE_E,
        SPRITE_X_VEL,
        SPRITE_Y_VEL,
        SPRITE_AI_STATE,
        SPRITE_A,
        SPRITE_DELAY_MAIN,
        SPRITE_OAM_FLAGS,
        SPRITE_STATE,
        SPRITE_FLAGS5,
        SPRITE_DEFL_BITS,
    ] {
        assert_eq!(state.ram[base + k], 0, "base ${base:04x}");
    }
}

#[test]
fn credits_scene_fade_advances_scratch_when_fade_not_complete() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    state.set_screen_brightness(2);
    state.ending_scratch_mut().set_primary_word(0x0300);

    state.credits_handle_scene_fade();

    assert_eq!(state.game_state.display.screen_brightness, 1);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x0301);
    assert_eq!(state.game_state.frame.submodule, 0);
}

#[test]
fn credits_scene_fade_holds_scratch_when_fade_completes() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    state.set_screen_brightness(1);
    state.ending_scratch_mut().set_primary_word(0x0300);

    state.credits_handle_scene_fade();

    assert_eq!(state.game_state.display.screen_brightness, 0);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x0300);
    assert_eq!(state.game_state.frame.submodule, 1);
}

#[test]
fn overworld_map16_wram_slots_are_bridge_only() {
    let source = concat!(
        include_str!("overworld.rs"),
        include_str!("overworld_shared.rs")
    );
    for symbol in [
        "MAP16_LOAD_SRC_OFF_OVERWORLD",
        "MAP16_LOAD_DST_OFF_OVERWORLD",
        "MAP16_LOAD_Y_UNIT_OVERWORLD",
        "MAP16_LOAD_SRC_OFF_PREV_OVERWORLD",
        "MAP16_LOAD_Y_UNIT_PREV_OVERWORLD",
        "MAP16_LOAD_DST_OFF_PREV_OVERWORLD",
        "MAP16_LOAD_SRC_OFF_SPEXIT_OVERWORLD",
        "MAP16_LOAD_SRC_OFF_EXIT_OVERWORLD",
        "ORANGE_BLUE_BARRIER_STATE_OVERWORLD",
        "SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF",
        "SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT",
    ] {
        let count = source.matches(symbol).count();
        assert!(
            (1..=2).contains(&count),
            "{symbol} should appear only at its const declaration and optional bridge write"
        );
    }
}

#[test]
fn migrated_link_world_reads_use_semantic_views() {
    for (path, source) in [
        ("ancilla.rs", include_str!("ancilla.rs")),
        ("dungeon.rs", include_str!("dungeon.rs")),
        ("ending.rs", include_str!("ending.rs")),
        ("hud.rs", include_str!("hud.rs")),
        ("load_gfx.rs", include_str!("load_gfx.rs")),
        ("messaging.rs", include_str!("messaging.rs")),
        ("misc.rs", include_str!("misc.rs")),
        ("overlord.rs", include_str!("overlord.rs")),
        ("overworld.rs", include_str!("overworld.rs")),
        ("player.rs", include_str!("player.rs")),
        ("player_oam.rs", include_str!("player_oam.rs")),
        ("sprite.rs", include_str!("sprite.rs")),
        ("sprite_main.rs", include_str!("sprite_main.rs")),
        ("sprite_main_blind.rs", include_str!("sprite_main_blind.rs")),
        ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
        (
            "sprite_main_dungeon_npcs.rs",
            include_str!("sprite_main_dungeon_npcs.rs"),
        ),
        ("sprite_main_ganon.rs", include_str!("sprite_main_ganon.rs")),
        ("sprite_main_guard.rs", include_str!("sprite_main_guard.rs")),
        (
            "sprite_main_helmasaur_king.rs",
            include_str!("sprite_main_helmasaur_king.rs"),
        ),
        (
            "sprite_main_hinox_shop.rs",
            include_str!("sprite_main_hinox_shop.rs"),
        ),
        (
            "sprite_main_mothula.rs",
            include_str!("sprite_main_mothula.rs"),
        ),
        ("sprite_main_npcs.rs", include_str!("sprite_main_npcs.rs")),
        ("sprite_main_prep.rs", include_str!("sprite_main_prep.rs")),
        (
            "sprite_main_small_bosses.rs",
            include_str!("sprite_main_small_bosses.rs"),
        ),
        ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
        ("tile_detect.rs", include_str!("tile_detect.rs")),
    ] {
        for needle in [
            concat!("self.", "raw_", "ram().word_at(LINK_X_COORD)"),
            concat!("self.", "raw_", "ram().word_at(LINK_Y_COORD)"),
            concat!("self.", "raw_", "ram().word_at(LINK_Z_COORD)"),
            concat!("self.", "raw_", "ram().word_at(OVERWORLD_SCREEN_INDEX)"),
            concat!("self.", "raw_", "ram().word_at(DUNGEON_ROOM_INDEX)"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use typed semantic views for {needle}"
            );
        }
    }
}

#[test]
fn migrated_player_coordinate_writes_use_semantic_views() {
    for (path, source) in [
        ("ancilla.rs", include_str!("ancilla.rs")),
        ("dungeon.rs", include_str!("dungeon.rs")),
        ("overworld.rs", include_str!("overworld.rs")),
        ("player.rs", include_str!("player.rs")),
        ("player_oam.rs", include_str!("player_oam.rs")),
        ("sprite.rs", include_str!("sprite.rs")),
        ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
        (
            "sprite_main_dungeon_npcs.rs",
            include_str!("sprite_main_dungeon_npcs.rs"),
        ),
        (
            "sprite_main_mothula.rs",
            include_str!("sprite_main_mothula.rs"),
        ),
        ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
        ("zelda_rtl.rs", include_str!("zelda_rtl.rs")),
    ] {
        for needle in [
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_X_COORD,"),
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_Y_COORD,"),
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_Z_COORD,"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use typed semantic views for {needle}"
            );
        }
    }
}

#[test]
fn lanmola_draw_uses_named_flat_trail_reader() {
    let source = include_str!("sprite_main_draw.rs");
    for needle in [
        "self.ram[MOLDORM_HISTORY_X_LO +",
        "self.ram[MOLDORM_HISTORY_Y_LO +",
        "self.ram[BEAMOS_LASER_HISTORY_X_HI +",
        "self.ram[BEAMOS_LASER_HISTORY_Y_HI +",
    ] {
        assert!(
            !source.contains(needle),
            "sprite_main_draw.rs should use lanmola_flat_trail_entry instead of {needle}"
        );
    }
    assert!(
        source.contains("lanmola_flat_trail_entry("),
        "sprite_main_draw.rs should route Lanmola trail reads through the named API"
    );
}

#[test]
fn migrated_select_file_frame_state_uses_semantic_accessors() {
    for (path, source) in [
        ("ancilla.rs", include_str!("ancilla.rs")),
        ("attract.rs", include_str!("attract.rs")),
        ("audio.rs", include_str!("audio.rs")),
        ("dungeon.rs", include_str!("dungeon.rs")),
        ("ending.rs", include_str!("ending.rs")),
        ("hud.rs", include_str!("hud.rs")),
        ("load_gfx.rs", include_str!("load_gfx.rs")),
        ("messaging.rs", include_str!("messaging.rs")),
        ("misc.rs", include_str!("misc.rs")),
        ("overlord.rs", include_str!("overlord.rs")),
        ("overworld.rs", include_str!("overworld.rs")),
        ("player.rs", include_str!("player.rs")),
        ("player_oam.rs", include_str!("player_oam.rs")),
        ("select_file.rs", include_str!("select_file.rs")),
        ("sprite.rs", include_str!("sprite.rs")),
        ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
        (
            "sprite_main_dungeon_npcs.rs",
            include_str!("sprite_main_dungeon_npcs.rs"),
        ),
        ("sprite_main_ganon.rs", include_str!("sprite_main_ganon.rs")),
        ("sprite_main_guard.rs", include_str!("sprite_main_guard.rs")),
        (
            "sprite_main_mothula.rs",
            include_str!("sprite_main_mothula.rs"),
        ),
        ("sprite_main_npcs.rs", include_str!("sprite_main_npcs.rs")),
        ("sprite_main_prep.rs", include_str!("sprite_main_prep.rs")),
        (
            "sprite_main_small_bosses.rs",
            include_str!("sprite_main_small_bosses.rs"),
        ),
        ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
    ] {
        for needle in [
            concat!("self.", "ram[MAIN_MODULE_INDEX]"),
            concat!("self.", "ram[SUBMODULE_INDEX]"),
            concat!("self.", "ram[SUBSUBMODULE_INDEX]"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use native frame state for {needle}"
            );
        }
    }
}

#[test]
fn emu_callback_setup_syncs_whole_state_and_regions() {
    let mut state = ZeldaState::new();
    state.zelda_setup_emu_callbacks(Some(vec![0; 16]), None, Some(test_sync_all));

    state.emu_synchronize_whole_state();
    assert_eq!(state.ram[0x42], 1);

    state.ram[0x1234..0x1238].copy_from_slice(&[1, 2, 3, 4]);
    state.emu_sync_memory_region(0x1234, 4);
    let emu = state.emu_memory_ptr.as_ref().unwrap();
    assert_eq!(&emu[0x1234..0x1238], &[1, 2, 3, 4]);
}

#[test]
fn byte_array_append_vl_matches_c_encoding() {
    let mut arr = ByteArray::default();

    ZeldaState::byte_array_append_vl(&mut arr, 0);
    ZeldaState::byte_array_append_vl(&mut arr, 254);
    ZeldaState::byte_array_append_vl(&mut arr, 255);
    ZeldaState::byte_array_append_vl(&mut arr, 511);

    assert_eq!(arr.data, vec![0, 254, 255, 0, 255, 255, 1]);

    let mut pos = 0usize;
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 0);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 254);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 255);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 511);
    assert_eq!(pos, arr.data.len());
}

#[test]
fn save_and_load_func_append_and_copy_bytes() {
    let mut arr = ByteArray::default();
    let mut src = [1, 2, 3, 4];
    ZeldaState::save_func(&mut arr, &mut src);
    assert_eq!(arr.data, src);

    let mut st = LoadFuncState::new(&arr.data);
    let mut dst = [0; 4];
    ZeldaState::load_func(&mut st, &mut dst);
    assert_eq!(dst, src);
    assert_eq!(st.remaining(), 0);
}

#[test]
fn snes_state_save_load_roundtrips_runtime_regions() {
    let mut state = ZeldaState::new();
    state.ram[0x100] = 0x12;
    write_le_u16(&mut state.ram, MAP16_LOAD_SRC_OFF, 0x1390);
    write_le_u16(&mut state.ram, MAP16_LOAD_DST_OFF, 0x001f);
    write_le_u16(&mut state.ram, MAP16_LOAD_Y_UNIT, 0x000e);
    state.sync_overworld_map16_state_from_ram();
    state.set_spotlight_hdma_table_dynamic_entry(0, 0x00ab);
    state.sram[0x22] = 0x34;
    state.ppu.cgram[7] = 0x2468;
    state.ppu.bg_layer[1].tilemap_higher = true;
    state.ppu.bg_layer[1].tilemap_adr = 0x1357;
    state.dma.channel[6].a_adr = 0x4567;

    let mut arr = ByteArray::default();
    let mut save = SaveLoadFunc::Save(&mut arr);
    state.save_snes_state(&mut save);
    assert_eq!(state.ram[0x1b00], 0xab);

    state.ram[0x100] = 0;
    write_le_u16(&mut state.ram, MAP16_LOAD_SRC_OFF, 0);
    write_le_u16(&mut state.ram, MAP16_LOAD_DST_OFF, 0);
    write_le_u16(&mut state.ram, MAP16_LOAD_Y_UNIT, 0);
    state.sync_overworld_map16_state_from_ram();
    state.set_spotlight_hdma_table_dynamic_entry(0, 0);
    state.sram[0x22] = 0;
    state.ppu.cgram[7] = 0;
    state.ppu.bg_layer[1].tilemap_higher = false;
    state.ppu.bg_layer[1].tilemap_adr = 0;
    state.dma.channel[6].a_adr = 0;

    let mut st = LoadFuncState::new(&arr.data);
    let mut load = SaveLoadFunc::Load(&mut st);
    state.load_snes_state(&mut load);

    assert_eq!(state.ram[0x100], 0x12);
    assert_eq!(
        state.game_state.world.overworld.map16.active_load,
        OverworldMap16LoadState {
            src_off: 0x1390,
            dst_off: 0x001f,
            y_unit: 0x000e
        }
    );
    assert_eq!(state.ram[HDMA_TABLE_DYNAMIC], 0xab);
    assert_eq!(state.sram[0x22], 0x34);
    assert_eq!(state.ppu.cgram[7], 0x2468);
    assert!(state.ppu.bg_layer[1].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0x1357);
    assert_eq!(state.dma.channel[6].a_adr, 0x4567);
}

#[test]
fn state_recorder_records_input_edges_like_c() {
    let mut sr = StateRecorder {
        last_inputs: 0xffff,
        frames_since_last: 99,
        total_frames: 88,
        replay_mode: true,
        log: ByteArray { data: vec![1, 2] },
        base_snapshot: ByteArray { data: vec![3] },
        ..StateRecorder::default()
    };
    ZeldaState::state_recorder_init(&mut sr);
    assert_eq!(sr, StateRecorder::default());

    ZeldaState::state_recorder_record(&mut sr, 0x0001);
    ZeldaState::state_recorder_record(&mut sr, 0x0001);
    ZeldaState::state_recorder_record(&mut sr, 0x0003);

    assert_eq!(sr.last_inputs, 0x0003);
    assert_eq!(sr.frames_since_last, 1);
    assert_eq!(sr.total_frames, 3);
    assert_eq!(sr.log.data, vec![0x00, 0x12]);
}

#[test]
fn state_recorder_records_long_waits_and_patch_bytes() {
    let mut sr = StateRecorder {
        frames_since_last: 20,
        ..StateRecorder::default()
    };
    ZeldaState::state_recorder_record_cmd(&mut sr, 0x00);
    assert_eq!(sr.frames_since_last, 0);
    assert_eq!(sr.log.data, vec![0x0f, 5]);

    ZeldaState::state_recorder_record_patch_byte(
        &mut sr,
        0x10020,
        &[0xaa, 0xbb, 0xcc, 0xdd, 0xee],
        5,
    );
    assert_eq!(
        sr.log.data,
        vec![0x0f, 5, 0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]
    );
}

#[test]
fn state_recorder_replays_input_edges_like_c() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 3,
        log: ByteArray {
            data: vec![0x00, 0x12],
        },
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
    assert!(sr.replay_mode);
    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
    assert!(sr.replay_mode);
    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0003);
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_replay_input_override_advances_replay_but_substitutes_input() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 3,
        log: ByteArray {
            data: vec![0x00, 0x12],
        },
        ..StateRecorder::default()
    };

    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, None),
        0x0001
    );
    assert!(sr.replay_mode);
    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, Some(0x0080)),
        0x0080
    );
    assert!(sr.replay_mode);
    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, None),
        0x0003
    );
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_replays_patch_bytes_and_can_stop() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 1,
        log: ByteArray {
            data: vec![0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee],
        },
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
    assert_eq!(
        &state.ram[0x10020..0x10025],
        &[0xaa, 0xbb, 0xcc, 0xdd, 0xee]
    );
    assert!(!sr.replay_mode);

    sr.replay_mode = true;
    sr.replay_frame_counter = 7;
    sr.replay_pos_last_complete = 3;
    ZeldaState::state_recorder_stop_replay(&mut sr);
    assert!(!sr.replay_mode);
    assert_eq!(sr.total_frames, 7);
    assert_eq!(sr.log.data, vec![0xce, 1, 0x00]);
}

#[test]
fn state_recorder_replays_snapshot_boundary_commands() {
    let mut state = ZeldaState::new();
    state.ram[0x1234] = 0x5a;
    state.sram[0x234] = 0x6b;
    state.ppu.cgram[3] = 0x1357;

    let mut snapshot = ByteArray::default();
    let mut save = SaveLoadFunc::Save(&mut snapshot);
    state.save_snes_state(&mut save);

    state.ram[0x1234] = 0;
    state.sram[0x234] = 0;
    state.ppu.cgram[3] = 0;

    let mut log = ByteArray::default();
    ByteArray_AppendByte(&mut log, 0xd0);
    ZeldaState::byte_array_append_vl(&mut log, snapshot.size() as u32);
    ByteArray_AppendData(&mut log, &snapshot.data);

    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 1,
        last_inputs: 0xffff,
        log,
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
    assert_eq!(state.ram[0x1234], 0x5a);
    assert_eq!(state.sram[0x234], 0x6b);
    assert_eq!(state.ppu.cgram[3], 0x1357);
    assert_eq!(sr.last_inputs, 0);
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_clear_key_log_rebases_snapshot_and_active_inputs() {
    let mut state = ZeldaState::new();
    state.ram[0x100] = 0x56;
    let mut sr = StateRecorder {
        last_inputs: 0x0003,
        frames_since_last: 5,
        total_frames: 12,
        log: ByteArray {
            data: vec![0xaa, 0xbb],
        },
        ..StateRecorder::default()
    };

    state.state_recorder_clear_key_log(&mut sr);

    assert!(!sr.base_snapshot.data.is_empty());
    assert_eq!(sr.log.data, vec![0x00, 0x10]);
    assert_eq!(sr.frames_since_last, 0);
    assert_eq!(sr.total_frames, 0);
}

#[test]
fn read_from_file_and_state_recorder_save_load_match_c_layout() {
    let mut cursor = std::io::Cursor::new(vec![1, 2, 3, 4]);
    let mut bytes = [0; 4];
    ZeldaState::read_from_file(&mut cursor, &mut bytes);
    assert_eq!(bytes, [1, 2, 3, 4]);

    let mut state = ZeldaState::new();
    state.ram[0x123] = 0x45;
    state.sram[0x234] = 0x67;
    let mut sr = StateRecorder {
        last_inputs: 0x00ff,
        frames_since_last: 7,
        total_frames: 9,
        log: ByteArray {
            data: vec![0x01, 0x23],
        },
        ..StateRecorder::default()
    };
    let mut out = Vec::new();
    state.state_recorder_save(&mut sr, &mut out);

    state.ram[0x123] = 0;
    state.sram[0x234] = 0;
    let mut loaded = StateRecorder::default();
    state.state_recorder_load(&mut loaded, &mut std::io::Cursor::new(out), false);

    assert_eq!(loaded.last_inputs, 0x00ff);
    assert_eq!(loaded.frames_since_last, 7);
    assert_eq!(loaded.total_frames, 9);
    assert_eq!(loaded.log.data, vec![0x01, 0x23]);
    assert_eq!(state.ram[0x123], 0x45);
    assert_eq!(state.sram[0x234], 0x67);
}

#[test]
fn zelda_run_frame_sanitizes_inputs_and_records_features() {
    let mut state = ZeldaState::new();
    state.wanted_zelda_features = 0x1000;
    state.set_animated_tile_data_source_address(1);

    let was_replay = state.zelda_run_frame(0x30 | 0xc0 | 1);

    assert!(!was_replay);
    assert_eq!(state.frame_ctr_dbg, 1);
    assert_eq!(state.state_recorder.last_inputs, 1);
    assert_eq!(state.ram[RAM_BUGS_FIXED], BUGFIX_LATEST);
    assert_eq!(
        state.debug_compatibility_ram_u32(ENHANCED_FEATURES0),
        0x1000
    );
}

#[test]
fn language_and_save_slot_shells_match_defaults() {
    let mut state = ZeldaState::new();
    state.dialogue_blk_index = 7;
    state.dialogue_font_blk_index = 8;
    state.dialogue_flags = 9;

    state.zelda_set_language(None);

    assert_eq!(state.dialogue_blk_index, 0);
    assert_eq!(state.dialogue_font_blk_index, 0);
    assert_eq!(state.dialogue_flags, 0);
    assert_eq!(
        ZeldaState::save_slot_path(SaveLoadCommand::Load, 3).unwrap(),
        PathBuf::from("saves/save3.sav")
    );
    assert!(ZeldaState::save_slot_path(SaveLoadCommand::Save, 256).is_none());
    assert_eq!(
        ZeldaState::save_slot_path(SaveLoadCommand::Replay, 256).unwrap(),
        Path::new("saves/ref").join("Chapter 1 - Zelda's Rescue.sav")
    );
}

#[test]
fn multi_patch_and_patch_command_update_ram_and_log() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder::default();
    let mut mp = StateRecoderMultiPatch::default();
    ZeldaState::state_recoder_multi_patch_init(&mut mp);

    state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x20, 0xaa);
    state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x21, 0xbb);
    ZeldaState::state_recoder_multi_patch_commit(&mut sr, &mut mp);

    assert_eq!(&state.ram[0x20..0x22], &[0xaa, 0xbb]);
    assert_eq!(sr.log.data, vec![0xc4, 0x00, 0x20, 0xaa, 0xbb]);

    state.patch_command('w');
    assert_eq!(state.ram[0xf372], 80);
    assert_eq!(state.ram[0xf373], 80);
    assert!(!state.state_recorder.log.data.is_empty());
}

#[test]
fn item_receipt_places_chest_item_with_c_offsets() {
    let mut state = ZeldaState::new();
    state.ram[ITEM_RECEIPT_METHOD] = 1;
    state
        .dungeon_room_load_mut()
        .set_loading_bg_offsets(0x1200, 0x3400);

    set_link_test_byte(&mut state, LINK_RECEIVEITEM_INDEX, 0);
    state.ancilla_add_item_receipt(0x22, 4, 0x0182);

    assert_eq!(state.ram[ANCILLA_X_LO + 4], 0x0c);
    assert_eq!(state.ram[ANCILLA_X_HI + 4], 0x12);
    assert_eq!(state.ram[ANCILLA_Y_LO + 4], 0x13);
    assert_eq!(state.ram[ANCILLA_Y_HI + 4], 0x34);
}

#[test]
fn receive_item_enters_hold_item_state_for_normal_receipts() {
    let mut state = ZeldaState::new();
    state.ram[ITEM_RECEIPT_METHOD] = 0;
    state.follower_link_state_mut().set_auxiliary_state(1);
    state.follower_link_state_mut().set_incapacitated_timer(7);
    state.ram[COUNTDOWN_FOR_BLINK] = 8;
    state.follower_link_state_mut().set_state_bits(0xff);
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0xff);
    state.follower_link_state_mut().set_button_b_frames(0xff);
    state.follower_link_state_mut().set_speed_setting(0xff);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);
    state.follower_link_state_mut().set_item_in_hand(0xff);
    state.follower_link_state_mut().set_position_mode(0xff);
    state.ram[PLAYER_HANDLER_TIMER] = 0xff;
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 0);

    state.link_receive_item(0x20, 0);

    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 0);
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        0
    );
    assert_eq!(state.ram[COUNTDOWN_FOR_BLINK], 0);
    assert_eq!(link_test_byte(&state, LINK_RECEIVEITEM_INDEX), 0x20);
    assert_eq!(link_test_byte(&state, LINK_ITEM_HOLDING_TIMER), 0x60);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(state.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 21);
    assert_eq!(link_test_byte(&state, LINK_POSE_FOR_ITEM), 2);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
}

#[test]
fn overworld_tile_attribute_uses_map16_and_map8_assets() {
    let mut state = ZeldaState::new();
    state.set_overworld_offset_base_y(0x20);
    state.set_overworld_offset_mask_y(0x1f);
    state.set_overworld_offset_base_x(3);
    state.set_overworld_offset_mask_x(0x3f);
    state.dungeon_room_tilemaps_mut().set_bg2_tile(32, 5);

    let mut data = vec![0; 0x100];
    write_le_u16(&mut data, (5 * 4 + 2) * 2, 0x4007);
    data[0x80 + 7] = 0x10;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    assert_eq!(
        state.overworld_get_tile_attribute_at_location(4, 0x28),
        0x11
    );
}

#[test]
fn outdoor_y_collision_starts_falling_into_pit() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(0);
    state.tile_detect_position_mut().or_pit_tile(5);

    state.start_movement_collision_checks_y_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_SPRITE_OAM_STATE_TIMER), 9);
    assert_eq!(state.game_state.player.follower_link.near_pit_state(), 1);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 1);
}

#[test]
fn outdoor_x_deepwater_without_flippers_hops_from_safe_return() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(0);
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 3);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);
    state.tile_detect_position_mut().set_deepwater(4);
    set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_LO, 0x34);
    set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_HI, 0x12);
    set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_LO, 0x78);
    set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_HI, 0x56);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x1234);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x5678);
    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        3
    );
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        16
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        24
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        16
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 6);
}

#[test]
fn outdoor_y_spike_damage_rebounds_and_unequips_cape() {
    let mut state = ZeldaState::new();
    state.tile_detect_position_mut().set_spike_cactus_tiles(1);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x40);

    state.start_movement_collision_checks_y_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 32);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_y_velocity(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        36
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
}

#[test]
fn outdoor_x_spike_damage_applies_tile_rebound() {
    let mut state = ZeldaState::new();
    state.tile_detect_position_mut().set_spike_cactus_tiles(1);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 2);
    set_link_test_word(&mut state, LINK_X_COORD, 0x40);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        36
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
}

#[test]
fn outdoor_x_misc_bugfix_runs_slope_check_while_dashing_vertically() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    state.follower_link_state_mut().set_facing(0);
    set_link_test_byte(&mut state, LINK_X_VEL, 1);
    state.tile_detect_position_mut().set_slope_collision_bits(5);
    set_link_test_word(&mut state, LINK_X_COORD, 0x44);
    state.enhanced_features_mut().set_bits(0x1000);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x40);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0x25);
}

#[test]
fn perform_dash_sets_start_dash_state_and_tagalong_timeout() {
    let mut state = ZeldaState::new();
    state
        .follower_link_state_mut()
        .set_somaria_platform_state(1);
    state.link_perform_dash();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);

    state
        .follower_link_state_mut()
        .set_somaria_platform_state(0);
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0xff);
    state.follower_link_state_mut().set_button_mask_b_y(0x7f);
    state.follower_link_state_mut().set_state_bits(0x7f);
    state.follower_link_state_mut().set_item_in_hand(3);
    state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
    set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 0xff);
    state.follower_link_state_mut().set_speed_setting(5);
    state.follower_state_mut().set_indicator(2);

    state.link_perform_dash();

    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 29);
    assert_eq!(link_test_byte(&state, LINK_DASH_CTR), 64);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 17);
    assert_eq!(state.game_state.player.follower_link.running_state(), 1);
    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(state.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS], 0);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(read_le_u16(&state.ram, TIMER_TAGALONG_REACQUIRE), 64);
}

#[test]
fn ledge_hop_timer_restores_previous_position_until_triggered() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_Y_COORD, 0x120);
    set_link_test_word(&mut state, LINK_X_COORD, 0x240);
    set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x100);
    set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x200);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 3);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 4);
    set_link_test_byte(&mut state, LINK_TIMER_JUMP_LEDGE, 2);

    assert!(!state.run_ledge_hop_timer());
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x100);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x200);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
}

#[test]
fn dash_repel_applies_tile_rebound_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);

    state.repel_dash();

    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        0u8.wrapping_sub(24)
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        36
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
    assert_eq!(link_test_byte(&state, LINK_WANT_MAKE_NOISE_WHEN_DASHED), 1);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 1);
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(2),
        256
    );
}

#[test]
fn sprite_repel_dash_uses_facing_as_rebound_direction() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
    state.follower_link_state_mut().set_facing(4);

    state.sprite_repel_dash();

    assert_eq!(link_test_byte(&state, LINK_LAST_DIRECTION_MOVED_TOWARDS), 2);
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
}

#[test]
fn flag67_with_directions_derives_direction_from_actual_velocity() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION, 0xff);
    state.follower_link_state_mut().set_actual_y_velocity(0xf0);
    state.follower_link_state_mut().set_actual_x_velocity(2);

    state.flag67_with_directions();

    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 9);
}

#[test]
fn move_position_applies_sand_drag_to_velocity_delta() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.follower_link_state_mut().set_actual_x_velocity(16);
    state
        .follower_link_state_mut()
        .set_actual_y_velocity(0u8.wrapping_sub(16));
    write_le_u16(&mut state.ram, DRAG_PLAYER_X, 1);
    write_le_u16(&mut state.ram, DRAG_PLAYER_Y, 0xffff);

    state.link_move_position();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01fe);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xfe);
    assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_LO), 0x00);
    assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_HI), 0x01);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_LO), 0x00);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_HI), 0x02);
}

#[test]
fn move_position_applies_moving_floor_before_velocity_delta() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.dungeon_room_load_mut().set_header_collision(1);
    state.set_player_layer_collision_flags(
        crate::game_state::constants::player::LAYER_COLLISION_BOTH,
    );
    state.dungeon_moving_floor_mut().set_floor_x_velocity(2);
    state
        .dungeon_moving_floor_mut()
        .set_floor_y_velocity(0xffff);

    state.link_move_position();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01ff);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x09);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xff);
}

#[test]
fn swim_stroke_updates_subpixels_and_actual_velocity() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER, 1);
    write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER + 2, 1);
    state.swim_acceleration_mut().set_mode(0, 0);
    state.swim_acceleration_mut().set_mode(2, 0);
    state.swim_acceleration_mut().set_acceleration(0, 4);
    state.swim_acceleration_mut().set_acceleration(2, 4);
    state.swim_acceleration_mut().set_max_speed(0, 32);
    state.swim_acceleration_mut().set_max_speed(2, 32);
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(0, 1);
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(2, 1);

    state.handle_swim_stroke_and_subpixels();

    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x05);
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(0),
        12
    );
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(2),
        12
    );
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 12);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 12);
    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
}

#[test]
fn moving_animation_uses_some_direction_bits_when_flag_moving() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.follower_link_state_mut().set_swim_direction_flags(8);
    state.follower_link_state_mut().set_joypad1h_last(8);

    state.link_handle_moving_animation_full_long_entry();

    assert_eq!(state.game_state.player.follower_link.facing(), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
}

#[test]
fn moving_animation_dash_advances_dash_cycle() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 32);
    set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 1);

    state.link_handle_moving_animation_full_long_entry();

    assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
}

#[test]
fn edge_transition_recoil_guard_restores_previous_position() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    set_link_test_byte(&mut state, LINK_X_VEL, 1);
    state.follower_link_state_mut().set_incapacitated_timer(5);
    state.follower_link_state_mut().set_actual_x_velocity(12);
    state.follower_link_state_mut().set_actual_y_velocity(34);
    set_link_test_word(&mut state, LINK_X_COORD, 0x01e9);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0123);
    set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x0088);
    set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x0099);

    state.Dungeon_TryScreenEdgeTransition();

    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(link_test_byte(&state, LINK_RECOILMODE_TIMER), 3);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0088);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x0099);
    assert_eq!(state.game_state.frame.submodule, 0);
}

#[test]
fn recoil_z_velocity_shift_matches_c_do_while_condition() {
    fn run_recoil_step(initial_recoil_timer: u8) -> (u8, u8) {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_handler_state(2);
        state.follower_link_state_mut().set_auxiliary_state(1);
        state.follower_link_state_mut().set_incapacitated_timer(8);
        set_link_test_byte(&mut state, LINK_RECOILMODE_TIMER, initial_recoil_timer);
        state.follower_link_state_mut().set_actual_z_velocity(0xf8);
        set_link_test_byte(&mut state, LINK_ACTUAL_VEL_Z_COPY, 0x24);
        set_link_test_word(&mut state, LINK_Z_COORD, 0xffff);

        state.link_state_recoil();

        (
            link_test_byte(&state, LINK_RECOILMODE_TIMER),
            state.game_state.player.follower_link.actual_z_velocity(),
        )
    }

    assert_eq!(run_recoil_step(0), (1, 0x09));
    assert_eq!(run_recoil_step(1), (2, 0x12));
    assert_eq!(run_recoil_step(2), (3, 0x12));
}

#[test]
fn cache_camera_properties_if_outdoors_snapshots_scroll_state() {
    let mut state = ZeldaState::new();
    state.set_bg2_x(0x1111);
    state.set_bg2_y(0x2222);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x3333);
    set_link_test_word(&mut state, LINK_X_COORD, 0x4444);
    state.room_bounds_mut().set_y_bound(0, 0x5555);
    state.room_bounds_mut().set_x_bound(2, 0x6666);
    state.set_up_down_scroll_target(0x7777);
    state.set_left_right_scroll_target_end(0x8888);
    state.set_camera_y_coord_scroll_low(0x9999);
    state.set_quadrant_fullsize_y(2);
    set_link_test_byte(&mut state, LINK_QUADRANT_Y, 2);
    state.follower_link_state_mut().set_facing(8);
    state.follower_link_state_mut().set_lower_level_state(1);
    state.ram[IS_STANDING_IN_DOORWAY] = 2;
    state.dungeon_stair_movement_mut().set_current_floor(0xff);

    state.cache_camera_properties_if_outdoors();

    assert_eq!(
        state
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2_cached(),
        0x1111
    );
    assert_eq!(
        state
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2_cached(),
        0x2222
    );
    assert_eq!(link_test_word(&state, LINK_Y_COORD_CACHED), 0x3333);
    assert_eq!(link_test_word(&state, LINK_X_COORD_CACHED), 0x4444);
    assert_eq!(
        state.game_state.world.transient.cached_room_bounds_y_start,
        0x5555
    );
    assert_eq!(
        state.game_state.world.transient.cached_room_bounds_x_end,
        0x6666
    );
    assert_eq!(
        read_le_u16(&state.ram, UP_DOWN_SCROLL_TARGET_CACHED),
        0x7777
    );
    assert_eq!(
        read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET_END_CACHED),
        0x8888
    );
    assert_eq!(
        read_le_u16(&state.ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED),
        0x9999
    );
    assert_eq!(state.ram[QUADRANT_FULLSIZE_Y_CACHED], 2);
    assert_eq!(link_test_byte(&state, LINK_QUADRANT_Y_CACHED), 2);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION_FACING_CACHED), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_CACHED), 1);
    assert_eq!(state.ram[IS_STANDING_IN_DOORWAY_CACHED], 2);
    assert_eq!(state.ram[DUNG_CUR_FLOOR_CACHED], 0xff);
}

#[test]
fn dungeon_layer_change_updates_floor_room_and_visited_flags() {
    let mut state = ZeldaState::new();
    state.set_dungeon_room(0x0104);
    state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
    state.set_quadrant_fullsize_y(1);
    state.set_quadrant_fullsize_x(1);
    set_link_test_byte(&mut state, LINK_QUADRANT_Y, 1);
    set_link_test_byte(&mut state, LINK_QUADRANT_X, 1);

    state.dungeon_handle_layer_change();

    assert_eq!(state.game_state.world.location.dungeon_room(), 0x0114);
    assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_MIRROR), 1);
    assert_eq!(state.game_state.player.follower_link.lower_level_state(), 1);
    assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
    assert_ne!(read_le_u16(&state.ram, DUNG_QUADRANTS_VISITED), 0);

    state
        .dungeon_stair_movement_mut()
        .set_kind_of_in_room_staircase_word(2);
    state.follower_link_state_mut().set_lower_level_state(0);
    state.dungeon_handle_layer_change();
    assert_eq!(state.game_state.player.follower_link.lower_level_state(), 0);
}

#[test]
fn link_initialize_applies_misc_bugfix_cleanup() {
    let mut state = ZeldaState::new();
    state.enhanced_features_mut().set_bits(0x1000);
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
    set_link_test_byte(&mut state, LINK_IS_NEAR_MOVEABLE_STATUE, 1);
    set_link_test_byte(&mut state, LINK_ON_CONVEYOR_BELT, 1);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.set_bg1_y_offset(0x1234);
    state.set_bg1_x_offset(0x5678);
    state.save_progress_mut().set_dark_world_state(1);

    state.link_initialize();

    assert_eq!(state.game_state.player.follower_link.facing(), 2);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
    assert_eq!(link_test_byte(&state, LINK_IS_NEAR_MOVEABLE_STATUE), 0);
    assert_eq!(link_test_byte(&state, LINK_ON_CONVEYOR_BELT), 0);
    assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 0);
    assert_eq!(read_le_u16(&state.ram, BG1_Y_OFFSET), 0);
    assert_eq!(read_le_u16(&state.ram, BG1_X_OFFSET), 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 23);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 1);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 1);
}

#[test]
fn damaging_pit_reset_restores_ground_or_permabunny_state() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 0);
    state.follower_link_state_mut().set_swim_direction_flags(8);
    set_link_test_byte(&mut state, LINK_IS_IN_DEEP_WATER, 1);
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    state.follower_link_state_mut().set_pit_data_index(1);
    state.ram[SWIMMING_COUNTDOWN] = 7;
    state
        .swim_acceleration_mut()
        .set_speed_active_flag(0, 0x1234);

    state.link_reset_state_after_damaging_pit();

    assert_eq!(state.game_state.player.follower_link.handler_state(), 23);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION_LAST), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 0);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(state.game_state.player.follower_link.pit_data_index(), 0);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .speed_active_flag(0),
        0
    );

    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
    state.follower_link_state_mut().set_handler_state(6);
    state.link_reset_state_after_damaging_pit();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
}

#[test]
fn set_to_deep_water_resets_swim_state_and_latches_direction() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
    state.follower_link_state_mut().set_grabbing_wall(1);
    state.follower_link_state_mut().set_speed_setting(2);
    state.ram[SWIMMING_COUNTDOWN] = 7;
    state.swim_acceleration_mut().set_acceleration(0, 0x1234);

    state.link_set_to_deep_water();

    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
    assert_eq!(state.game_state.player.follower_link.grabbing_wall(), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
    assert_eq!(state.game_state.player.swim_acceleration.acceleration(0), 0);
}

#[test]
fn swim_accels_start_ramp_and_snap_to_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x0d);
    state.swim_acceleration_mut().set_acceleration(0, 0);
    state.swim_acceleration_mut().set_max_speed(0, 0);
    state.swim_acceleration_mut().set_acceleration(2, 260);
    state.swim_acceleration_mut().set_max_speed(2, 384);

    state.link_handle_swim_accels();

    assert_eq!(state.game_state.player.swim_acceleration.acceleration(0), 1);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 240);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(2), 288);

    state.link_handle_swim_accels();
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 384);
}

#[test]
fn swim_momentum_sets_direction_and_starting_accel() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x09);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
    state
        .follower_link_state_mut()
        .set_swim_direction_flags(0x04);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x08);
    state.swim_acceleration_mut().set_max_speed(2, 0x1234);

    state.link_set_momentum();

    assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER), 8);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 2);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 240);
    assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER + 2), 8);
    assert_eq!(state.game_state.player.swim_acceleration.mode(2), 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .acceleration_direction(2),
        1
    );
    assert_eq!(
        state.game_state.player.swim_acceleration.max_speed(2),
        0x1234
    );
}

#[test]
fn reset_all_acceleration_clears_swim_accel_pairs() {
    let mut state = ZeldaState::new();
    for offset in [0, 2] {
        state
            .swim_acceleration_mut()
            .set_speed_active_flag(offset, 0xffff);
        state.swim_acceleration_mut().set_mode(offset, 0xffff);
        state
            .swim_acceleration_mut()
            .set_acceleration(offset, 0xffff);
        state.swim_acceleration_mut().set_max_speed(offset, 0xffff);
    }
    for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
        write_le_u16(&mut state.ram, offset, 0xffff);
    }
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(0, 0xffff);

    state.reset_all_acceleration();

    for offset in [0, 2] {
        assert_eq!(
            state
                .game_state
                .player
                .swim_acceleration
                .speed_active_flag(offset),
            0
        );
        assert_eq!(state.game_state.player.swim_acceleration.mode(offset), 0);
        assert_eq!(
            state
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset),
            0
        );
        assert_eq!(
            state.game_state.player.swim_acceleration.max_speed(offset),
            0
        );
    }
    for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
        assert_eq!(read_le_u16(&state.ram, offset), 0);
    }
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .acceleration_direction(0),
        0xffff
    );
}

#[test]
fn swimming_handler_without_flippers_only_clears_action_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state.follower_link_state_mut().set_button_b_frames(9);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 7);
    set_link_test_byte(&mut state, LINK_SPIN_ATTACK_STEP_COUNTER, 6);
    state.follower_link_state_mut().set_state_bits(5);
    state.follower_link_state_mut().set_picking_throw_state(4);
    set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 0);

    state.player_handler_04_swimming();

    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 0);
    assert_eq!(link_test_byte(&state, LINK_SPIN_ATTACK_STEP_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(
        state.game_state.player.follower_link.picking_throw_state(),
        0
    );
}

#[test]
fn swimming_handler_starts_hard_stroke_and_advances_swim_animation() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(4);
    set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 1);
    set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 7);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_joypad1h_last(8);
    state.swim_acceleration_mut().set_acceleration(0, 1);

    state.player_handler_04_swimming();

    assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
    assert_eq!(state.ram[SWIM_STROKE_ANIM_STEP], 0);
    assert_eq!(link_test_byte(&state, LINK_SWIM_HARD_STROKE), 0x80);
    assert_eq!(link_test_byte(&state, LINK_MAYBE_SWIM_FASTER), 1);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 6);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 37);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
}

#[test]
fn swim_movement_without_input_resets_idle_flag_moving_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(4);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
    state.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
    state
        .swim_acceleration_mut()
        .set_speed_active_flag(0, 0x1111);

    state.link_handle_swim_movements();

    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
    assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS] & 0x0f, 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .speed_active_flag(0),
        0
    );
    assert_eq!(state.ram[PIT_CORRECTION_ACTIVE_FLAG], 0);
}

#[test]
fn handle_toss_clears_a_press_state_when_throwing() {
    let mut state = ZeldaState::new();
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0x80);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_item_action_step_var(7);
    state.follower_link_state_mut().set_throw_oam_state_index(8);
    state.follower_link_state_mut().set_y_button_action_step(9);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);

    assert!(state.link_handle_toss());

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .throw_oam_state_index(),
        0
    );
    assert_eq!(
        state.game_state.player.follower_link.y_button_action_step(),
        0
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0x80);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_picking_throw_state(1);
    assert!(!state.link_handle_toss());
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0x80
    );
}

#[test]
fn halt_link_when_using_items_stops_floor_and_platform_motion() {
    let mut state = ZeldaState::new();
    state.dungeon_room_load_mut().set_header_collision_2(2);
    state.set_player_layer_collision_flags(
        crate::game_state::constants::player::LAYER_COLLISION_BOTH,
    );
    set_link_test_byte(&mut state, LINK_Y_VEL, 0x80);
    set_link_test_byte(&mut state, LINK_X_VEL, 0x40);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 0x55);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 0xaa);
    set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 1);

    state.halt_link_when_using_items();

    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);

    state.ram[DUNG_HDR_COLLISION_2] = 0;
    state.set_player_layer_collision_flags(0);
    state
        .follower_link_state_mut()
        .set_somaria_platform_state(1);
    set_link_test_byte(&mut state, LINK_Y_VEL, 7);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
    state.halt_link_when_using_items();
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 7);
}

#[test]
fn cape_item_activation_and_no_magic_prompt_match_c_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 10);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 1);

    state.link_item_cape();

    assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 1);
    assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 8);
    assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 20);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 20);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );

    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.link_item_cape();

    assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 60);
    assert_eq!(
        state.game_state.messaging.dialogue_message_index.value(),
        123
    );
    assert_eq!(state.game_state.frame.main_module, 14);
}

#[test]
fn rod_hammer_and_bow_item_handlers_advance_c_timers() {
    let mut rod = ZeldaState::new();
    rod.follower_link_state_mut().set_filtered_joypad_h(0x40);
    rod.follower_link_state_mut().set_magic_power(20);
    rod.ram[EQ_SELECTED_ROD] = 1;
    rod.link_item_rod();
    assert_eq!(link_test_byte(&rod, LINK_MAGIC_POWER), 4);
    assert!(rod.game_state.player.follower_link.item_in_hand_has(1));
    assert_eq!(link_test_byte(&rod, LINK_DEBUG_VALUE_2), 1);
    assert_eq!(link_test_byte(&rod, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(rod.ancilla_slot_view(4).ancilla_type(), 2);

    let mut hammer = ZeldaState::new();
    hammer.follower_link_state_mut().set_filtered_joypad_h(0x40);
    hammer.link_item_hammer();
    assert_eq!(hammer.game_state.player.follower_link.item_in_hand(), 2);
    assert_eq!(link_test_byte(&hammer, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&hammer, LINK_DELAY_TIMER_SPIN_ATTACK), 2);

    let mut bow = ZeldaState::new();
    bow.follower_link_state_mut().set_button_mask_b_y(0x40);
    bow.follower_link_state_mut().set_item_in_hand(0x10);
    set_link_test_byte(&mut bow, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    bow.ram[PLAYER_HANDLER_TIMER] = 2;
    set_link_test_byte(&mut bow, LINK_CANT_CHANGE_DIRECTION, 1);
    bow.player_resources_mut().set_arrows(2);
    bow.follower_link_state_mut().set_button_b_frames(12);
    bow.link_item_bow();
    assert_eq!(link_test_byte(&bow, LINK_NUM_ARROWS), 1);
    assert!(!bow.game_state.player.follower_link.item_in_hand_has(0x10));
    assert_eq!(
        bow.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(bow.game_state.player.follower_link.button_b_frames(), 9);
    assert_eq!(bow.ancilla_slot_view(4).ancilla_type(), 9);
}

#[test]
fn boomerang_bombs_book_and_desert_prayer_match_c_state() {
    let mut boom = ZeldaState::new();
    boom.follower_link_state_mut().set_filtered_joypad_h(0x40);
    boom.inventory_items_mut().set_inventory_item(1, 1);
    boom.link_item_boomerang();
    assert_eq!(boom.game_state.player.follower_link.item_in_hand(), 0x80);
    assert_eq!(boom.ram[FLAG_FOR_BOOMERANG_IN_PLACE], 1);
    assert_eq!(link_test_byte(&boom, LINK_DELAY_TIMER_SPIN_ATTACK), 6);
    assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 1);

    set_link_test_byte(&mut boom, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    boom.ram[PLAYER_HANDLER_TIMER] = 1;
    boom.link_item_boomerang();
    assert_eq!(boom.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(boom.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        boom.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

    let mut bombs = ZeldaState::new();
    bombs.follower_link_state_mut().set_filtered_joypad_h(0x40);
    bombs.player_resources_mut().set_bombs(1);
    bombs.link_item_bombs();
    // C `AncillaAdd_Bomb(7, 1)` allocates via `Ancilla_AllocInit(7, 1)`, which
    // for ancilla types 7/8 walks slots [limit..0], so slot 1 receives the
    // bomb ancilla. See zelda3/src/ancilla.c:5763 and ancilla.c:6990.
    assert_eq!(bombs.ancilla_slot_view(1).ancilla_type(), 7);
    assert_eq!(
        bombs.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(link_test_byte(&bombs, LINK_ITEM_BOMBS), 0);
    assert_eq!(bombs.game_state.player.follower_link.item_in_hand(), 0);

    let mut book = ZeldaState::new();
    book.follower_link_state_mut().set_filtered_joypad_h(0x40);
    book.link_item_book();
    assert_eq!(book.game_state.system_signals.sound_effect_1() & 0x3f, 60);

    let mut prayer = ZeldaState::new();
    prayer.follower_link_state_mut().set_filtered_joypad_h(0x40);
    prayer.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = 1;
    prayer.set_main_module(9);
    set_link_test_byte(&mut prayer, LINK_DIRECTION, 0x0f);
    prayer.link_item_book();
    assert_eq!(prayer.game_state.frame.submodule, 5);
    assert_eq!(prayer.game_state.frame.saved_module_for_menu, 9);
    assert_eq!(prayer.game_state.frame.main_module, 14);
    assert_eq!(prayer.game_state.frame.modal_pause_flag, 1);
    assert_eq!(
        prayer
            .game_state
            .player
            .follower_link
            .y_button_action_timer(),
        22
    );
    assert_eq!(prayer.game_state.player.follower_link.state_bits(), 2);
    assert_eq!(link_test_byte(&prayer, LINK_DIRECTION), 0);
    assert_eq!(prayer.game_state.system_signals.ambient_sound_effect(), 17);
    assert_eq!(prayer.game_state.system_signals.music_control(), 242);
}

#[test]
fn lamp_powder_and_shovel_item_handlers_match_core_state() {
    let mut lamp = ZeldaState::new();
    lamp.follower_link_state_mut().set_filtered_joypad_h(0x40);
    lamp.inventory_items_mut().set_inventory_item(10, 1);
    lamp.follower_link_state_mut().set_magic_power(32);
    set_link_test_byte(&mut lamp, LINK_CANT_CHANGE_DIRECTION, 1);
    lamp.follower_link_state_mut().set_button_b_frames(9);
    lamp.link_item_lamp();
    assert_eq!(link_test_byte(&lamp, LINK_MAGIC_POWER), 28);
    assert_eq!(lamp.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(lamp.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(link_test_byte(&lamp, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(lamp.ancilla_slot_view(4).ancilla_type(), 0x1a);
    assert_eq!(lamp.ancilla_slot_view(3).ancilla_type(), 0x2f);

    let mut powder = ZeldaState::new();
    powder.follower_link_state_mut().set_filtered_joypad_h(0x40);
    powder.inventory_items_mut().set_mushroom(2);
    powder.follower_link_state_mut().set_magic_power(16);
    powder.link_item_powder();
    assert_eq!(link_test_byte(&powder, LINK_MAGIC_POWER), 8);
    assert_eq!(powder.game_state.player.follower_link.item_in_hand(), 0x40);
    assert_eq!(link_test_byte(&powder, LINK_DELAY_TIMER_SPIN_ATTACK), 1);
    assert_eq!(link_test_byte(&powder, LINK_DIRECTION), 0);

    let mut shovel = ZeldaState::new();
    shovel.follower_link_state_mut().set_filtered_joypad_h(0x40);
    shovel.link_item_shovel();
    assert_eq!(shovel.game_state.player.follower_link.position_mode(), 1);
    assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&shovel, LINK_DELAY_TIMER_SPIN_ATTACK), 6);

    set_link_test_byte(&mut shovel, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    shovel.follower_link_state_mut().set_item_action_step_var(2);
    shovel.link_item_shovel();
    assert_eq!(
        shovel
            .game_state
            .player
            .follower_link
            .item_action_step_var(),
        0
    );
    assert_eq!(shovel.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        shovel.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(shovel.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
}

#[test]
fn flute_item_countdown_and_weather_vane_branch_match_c_state() {
    let mut countdown = ZeldaState::new();
    countdown
        .follower_link_state_mut()
        .set_button_mask_b_y(0x40);
    countdown.ram[FLUTE_COUNTDOWN] = 2;
    countdown.link_item_flute();
    assert_eq!(countdown.ram[FLUTE_COUNTDOWN], 1);
    assert_eq!(
        countdown.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0x40
    );

    let mut flute = ZeldaState::new();
    flute.follower_link_state_mut().set_filtered_joypad_h(0x40);
    flute.inventory_items_mut().set_flute(2);
    flute.set_overworld_screen_word(0x18);
    set_link_test_word(&mut flute, LINK_Y_COORD, 0x780);
    set_link_test_word(&mut flute, LINK_X_COORD, 0x200);
    flute.link_item_flute();
    assert_eq!(flute.ram[FLUTE_COUNTDOWN], 128);
    assert_eq!(flute.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(flute.game_state.frame.submodule, 45);
    assert_eq!(flute.ancilla_slot_view(4).ancilla_type(), 55);

    let mut shovel_dispatch = ZeldaState::new();
    shovel_dispatch
        .follower_link_state_mut()
        .set_filtered_joypad_h(0x40);
    shovel_dispatch.inventory_items_mut().set_flute(1);
    shovel_dispatch.link_item_shovel_and_flute();
    assert_eq!(
        shovel_dispatch
            .game_state
            .player
            .follower_link
            .position_mode(),
        1
    );
}

#[test]
fn medallion_item_start_and_state_progression_match_core_state() {
    let mut ether = ZeldaState::new();
    ether.follower_link_state_mut().set_filtered_joypad_h(0x40);
    ether.inventory_items_mut().set_sword_type(1);
    ether.follower_link_state_mut().set_magic_power(64);
    ether.link_item_ether();
    assert_eq!(link_test_byte(&ether, LINK_MAGIC_POWER), 32);
    assert_eq!(ether.game_state.player.follower_link.handler_state(), 8);
    assert_eq!(link_test_byte(&ether, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&ether, LINK_DELAY_TIMER_SPIN_ATTACK), 5);
    assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 0);
    assert_eq!(ether.game_state.system_signals.sound_effect_2() & 0x3f, 35);

    ether
        .follower_link_state_mut()
        .set_spin_attack_delay_timer(0);
    ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 9;
    ether.link_state_using_ether();
    assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 10);
    assert_eq!(ether.ram[SPIN_ATTACK_SOUND_LATCH], 1);
    assert_eq!(ether.ancilla_slot_view(4).ancilla_type(), 24);

    let mut quake = ZeldaState::new();
    quake.follower_link_state_mut().set_filtered_joypad_h(0x40);
    quake.inventory_items_mut().set_sword_type(1);
    quake.follower_link_state_mut().set_magic_power(64);
    quake.link_item_quake();
    assert_eq!(quake.game_state.player.follower_link.handler_state(), 10);
    assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_MIRROR), 40);
    assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_COPY_MIRROR), 40);
    assert_eq!(link_test_byte(&quake, LINK_Z_COORD_MIRROR), 0);

    let mut blocked = ZeldaState::new();
    blocked
        .follower_link_state_mut()
        .set_filtered_joypad_h(0x40);
    blocked.follower_link_state_mut().set_magic_power(64);
    blocked.link_item_bombos();
    assert_eq!(blocked.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(
        blocked.game_state.system_signals.sound_effect_1() & 0x3f,
        60
    );
}

#[test]
fn mirror_item_crossing_and_follower_cleanup_match_core_state() {
    let mut mirror = ZeldaState::new();
    mirror.follower_link_state_mut().set_filtered_joypad_h(0x40);
    mirror.enhanced_features_mut().set_bits(8);
    mirror.set_overworld_screen_word(0x40);
    set_link_test_word(&mut mirror, LINK_Y_COORD, 0x1234);
    set_link_test_word(&mut mirror, LINK_X_COORD, 0x5678);
    mirror.follower_link_state_mut().set_actual_x_velocity(7);
    mirror.follower_link_state_mut().set_actual_y_velocity(9);
    mirror.link_item_mirror();
    assert_eq!(mirror.ram[LAST_LIGHT_VS_DARK_WORLD], 0x40);
    assert_eq!(mirror.bird_travel_destination(15).y, 0x1234);
    assert_eq!(mirror.bird_travel_destination(15).x, 0x5678);
    assert_eq!(mirror.game_state.frame.submodule, 35);
    assert_eq!(
        link_test_byte(&mirror, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE),
        1
    );
    assert_eq!(mirror.game_state.player.follower_link.handler_state(), 20);
    assert_eq!(
        mirror.game_state.player.follower_link.actual_x_velocity(),
        0
    );
    assert_eq!(
        mirror.game_state.player.follower_link.actual_y_velocity(),
        0
    );

    let mut crossing = ZeldaState::new();
    crossing.ram[LAST_LIGHT_VS_DARK_WORLD] = 0;
    crossing.set_overworld_screen_word(0x40);
    let mut data = vec![0; 0x100];
    data[0x80] = 1;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    crossing.assets = Some(AssetPack::from_data_ranges(data, ranges));
    crossing.link_state_crossing_worlds();
    assert_eq!(crossing.game_state.frame.submodule, 44);
    assert_eq!(crossing.game_state.player.follower_link.handler_state(), 20);

    let mut follower = ZeldaState::new();
    follower.follower_state_mut().set_indicator(13);
    follower.follower_state_mut().set_dropped(1);
    set_link_test_byte(&mut follower, LINK_CAPE_MODE, 1);
    follower.handle_followers_after_mirroring();
    assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_TIMER], 0xfe);
    assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_COUNTER], 0);
    assert_eq!(follower.ram[FOLLOWER_INDICATOR], 0);
    assert_eq!(link_test_byte(&follower, LINK_CAPE_MODE), 0);
    assert_eq!(link_test_byte(&follower, LINK_BUNNY_TRANSFORM_TIMER), 0);
}

#[test]
fn hookshot_item_and_timeout_state_match_core_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.follower_link_state_mut().set_facing(4);
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.swim_acceleration_mut().set_speed_active_flag(0, 1);

    state.link_item_hookshot();

    assert_eq!(state.game_state.player.follower_link.handler_state(), 19);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 4);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 7);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x1f);
    assert_eq!(
        state.game_state.messaging.runtime.game_over_letter_cursor(),
        4
    );
    assert_eq!(state.ram[ANCILLA_X_VEL + 4], 0xc0);
    assert_eq!(read_le_u16(&state.ram, ANCILLA_X_LO + 4), 0x00fc);

    state.ancilla_slot_view_mut(4).set_ancilla_type(0);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_button_b_frames(12);
    state.link_state_hookshotting();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert!(!state.game_state.player.follower_link.position_mode_has(4));
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 9);
}

#[test]
fn cane_of_somaria_start_consumes_magic_and_enters_item_pose() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 32);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

    state.link_item_cane_of_somaria();

    assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0x40
    );
    assert!(state.game_state.player.follower_link.position_mode_has(8));
    assert_eq!(link_test_byte(&state, LINK_DEBUG_VALUE_2), 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x2c);
}

#[test]
fn cane_of_byrna_start_and_finish_match_timer_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 40);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

    state.link_item_cane_of_byrna();

    assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x30);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 8);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 18);

    state.ancilla_slot_view_mut(4).set_ancilla_type(0);
    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.ram[PLAYER_HANDLER_TIMER] = 2;
    state.link_item_cane_of_byrna();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
}

#[test]
fn bug_net_start_and_finish_match_c_timer_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.follower_link_state_mut().set_facing(4);

    state.link_item_net();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 9);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 16);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 50);

    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_item_action_step_var(9);
    state.link_item_net();

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    assert_eq!(state.game_state.player.follower_link.oam_x_offset(), 0x80);
    assert_eq!(state.game_state.player.follower_link.oam_y_offset(), 0x80);
}

#[test]
fn bug_net_right_facing_finish_does_not_read_past_timer_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    state.follower_link_state_mut().set_facing(6);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_item_action_step_var(9);
    state.ram[PLAYER_HANDLER_TIMER] = 8;
    state.follower_link_state_mut().set_position_mode(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);

    state.link_item_net();

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    assert_eq!(state.game_state.player.follower_link.oam_x_offset(), 0x80);
    assert_eq!(state.game_state.player.follower_link.oam_y_offset(), 0x80);
}

#[test]
fn link_zap_mosaic_bounces_between_zero_and_c0() {
    let mut state = ZeldaState::new();
    state.set_mosaic_level(0xb0);

    state.LinkZap_HandleMosaic();

    assert_eq!(state.game_state.display.mosaic_level, 0xc0);
    assert_eq!(state.game_state.display.mosaic_direction, 1);
    assert_eq!(state.game_state.display.mosaic_copy, 0x63);
    assert_eq!(state.game_state.display.bg_mode, 9);

    state.set_mosaic_level(0x10);
    state.LinkZap_HandleMosaic();
    assert_eq!(state.game_state.display.mosaic_level, 0);
    assert_eq!(state.game_state.display.mosaic_direction, 0);
    assert_eq!(state.game_state.display.mosaic_copy, 3);
}

#[test]
fn zapped_state_advances_timer_and_finishes_on_eighth_pulse() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(7);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.ram[PLAYER_HANDLER_TIMER] = 7;
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
    state.follower_link_state_mut().set_auxiliary_state(1);
    state.set_mosaic_level(0x20);
    state.set_mosaic_direction(1);

    state.link_state_zapped();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 0);
    assert_eq!(state.game_state.display.mosaic_level, 0);
    assert_eq!(state.game_state.display.mosaic_copy, 3);
    assert_eq!(state.game_state.display.bg_mode, 9);
}

#[test]
fn load_actual_gear_palettes_applies_enhanced_glove_color() {
    let mut state = ZeldaState::new();
    state.enhanced_features_mut().set_bits(0x1000);
    state.inventory_items_mut().set_inventory_item(20, 2);

    state.load_actual_gear_palettes();

    assert_eq!(
        read_le_u16(&state.ram, AUX_PALETTE_BUFFER + 0xfd * 2),
        0x0376
    );
    assert_eq!(
        read_le_u16(&state.ram, MAIN_PALETTE_BUFFER + 0xfd * 2),
        0x0376
    );
    assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 2);
}

#[test]
fn cancel_dash_clears_running_state_and_dash_ancilla() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 12);
    state.follower_link_state_mut().set_speed_setting(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
    state.swim_acceleration_mut().set_mode(0, 0x1234);
    state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
    state.ancilla_slot_view_mut(4).set_ancilla_type(0x1e);

    state.link_cancel_dash();

    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0);
    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.game_state.player.follower_link.running_state(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 0);
}

#[test]
fn exiting_dash_resets_or_counts_down_like_c_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(18);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 3);

    state.link_state_exiting_dash();

    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 4);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 18);

    state.follower_link_state_mut().set_joypad1h_last(1);
    state.follower_link_state_mut().set_running_state(1);
    state.follower_link_state_mut().set_speed_setting(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
    state.follower_link_state_mut().set_button_b_frames(8);
    state.swim_acceleration_mut().set_mode(0, 0x1234);

    state.link_state_exiting_dash();

    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(state.game_state.player.follower_link.running_state(), 0);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
}

#[test]
fn item_tile_behavior_routes_overworld_attr_to_tile_execute() {
    let mut state = ZeldaState::new();
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state.set_overworld_offset_mask_y(0x1f);
    state.set_overworld_offset_mask_x(0x3f);
    state.dungeon_room_tilemaps_mut().set_bg2_tile(16, 7);

    let mut data = vec![0; 0x100];
    write_le_u16(&mut data, 7 * 4 * 2, 3);
    data[0x80 + 3] = 1;
    let mut ranges = vec![(0, 0); 164];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    state.tile_detect_main_handler(1);

    assert_eq!(state.game_state.player.tile_detection.collision_bits(), 0);
    assert_eq!(read_le_u16(&state.ram, TILEDETECT_NORMAL_TILES), 1);
}

#[test]
fn tile_main_handler_shallow_water_sets_ripple_and_slosh_sound() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_DIRECTION, 1);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x09);

    state.tile_detect_main_handler(0);

    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .water_ripple_or_grass_state(),
        1
    );
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 28);
    assert_eq!(state.ram[RAW_SFX_PAN_VALUE], 28);
}

#[test]
fn tile_main_handler_spike_trigger_applies_damage_and_bunny_reset() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
    set_link_test_byte(&mut state, LINK_IS_BUNNY_MIRROR, 1);
    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
    set_link_test_word(&mut state, LINK_TIMER_TEMPBUNNY, 0x1234);
    set_link_test_byte(&mut state, LINK_NEED_FOR_POOF_FOR_TRANSFORM, 1);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x0d);

    state.tile_detect_main_handler(0);

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 0);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 0);
    assert_eq!(link_test_byte(&state, LINK_NEED_FOR_POOF_FOR_TRANSFORM), 0);
    assert_eq!(link_test_word(&state, LINK_TIMER_TEMPBUNNY), 0);
}

#[test]
fn tile_main_handler_icy_floor_starts_sliding_state() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_DIRECTION, 4);
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x0e);

    state.tile_detect_main_handler(0);

    assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .water_ripple_or_grass_state(),
        0
    );
}

#[test]
fn push_block_target_flag_reads_dungeon_attr_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_lower_level_state(1);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x1000 + 0x145, 0x72);

    assert_eq!(state.push_block_get_target_tile_flag(5, 0x28), 0x72);
}

#[test]
fn push_block_attempt_checks_both_target_tiles() {
    let mut state = ZeldaState::new();
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 4, 0);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 5, 11);

    assert!(state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));

    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 5, 9);
    assert!(!state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));
}

#[test]
fn bottled_item_receipt_fills_first_open_bottle() {
    let mut state = ZeldaState::new();
    state.inventory_items_mut().set_bottle(0, 2);
    state.inventory_items_mut().set_bottle(1, 4);

    state.item_receipt_give_bottled_item(0x2f);

    assert_eq!(link_test_byte(&state, LINK_ITEM_BOTTLE_INFO), 4);
    assert_eq!(state.ram[LINK_ITEM_BOTTLE_INFO + 1], 4);
}

#[test]
fn first_frame_runs_startup_writes() {
    let mut state = ZeldaState::new();
    state.sram[0x03e5] = 0xaa;
    state.sram[0x03e6] = 0x55;
    state.sram[0x08e5] = 0x12;
    state.ram[MAIN_PALETTE_BUFFER] = 0xff;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(read_le_u16(&state.ram, ANIMATED_TILE_DATA_SRC), 0xa680);
    assert_eq!(
        state.game_state.display.animated_tile_data_source_address,
        0xa680
    );
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_9), 0xb280);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_14), 0xb2e0);
    assert_eq!(state.game_state.display.screen_brightness, 15);
    assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 0);
    assert_eq!(read_le_u16(&state.sram, 0x03e5), 0x55aa);
    assert_eq!(read_le_u16(&state.sram, 0x08e5), 0);
    assert_eq!(state.selected_save_slot_x2(), 0);
    assert_eq!(read_le_u16(&state.ram, MAIN_PALETTE_BUFFER), 0);
}

#[test]
fn rom_startup_preroll_matches_live_console_timing() {
    let mut state = ZeldaState::new();

    state.set_rom_startup_timing(true);

    assert_eq!(state.rom_reset_frame_delay, 82);
}

#[test]
fn checkpoint_resume_restores_live_timing_without_rephasing_audio() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    let encoded = bincode::serialize(&state).expect("serialize ZeldaState checkpoint");
    let mut restored: ZeldaState =
        bincode::deserialize(&encoded).expect("deserialize ZeldaState checkpoint");

    assert!(!restored.rom_startup_timing());
    let audio_before = restored.zelda_audio_snapshot_bytes();

    restored.restore_live_rom_timing_after_checkpoint();

    assert!(restored.rom_startup_timing());
    assert_eq!(restored.zelda_audio_snapshot_bytes(), audio_before);
}

#[test]
fn rom_startup_holds_game_loop_while_intro_sound_bank_bootstraps() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 1);

    for _ in 0..74 {
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert_eq!(state.game_state.frame.subsubmodule, 1);
    }

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 2);
}

#[test]
fn rom_intro_memory_initialization_wait_matches_live_console_timing() {
    assert_eq!(configured_intro_memory_initialization_frames(), 40);
}

#[test]
fn rom_intro_poly_thread_initializer_resumes_across_host_frames() {
    assert_eq!(rom_intro_poly_init_decision(3), (true, false, 2));
    assert_eq!(rom_intro_poly_init_decision(2), (false, false, 1));
    assert_eq!(rom_intro_poly_init_decision(1), (false, true, 0));
}

#[test]
fn attract_low_work_area_clear_refreshes_native_state_before_reuse() {
    let mut state = ZeldaState::new();
    state.attract_scene_mut().set_state(1);

    state.clear_attract_low_work_area();

    assert_eq!(state.game_state.ending.attract_scene.state(), 0);
}

#[test]
fn attract_graphics_initializer_resumes_at_semantic_work_boundaries() {
    assert_eq!(rom_attract_init_graphics_decision(4), (false, 3));
    assert_eq!(rom_attract_init_graphics_decision(3), (false, 2));
    assert_eq!(rom_attract_init_graphics_decision(2), (true, 1));
    assert_eq!(rom_attract_init_graphics_decision(1), (false, 0));
}

#[test]
fn attract_first_story_render_wait_is_armed_by_fade_completion() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_state(4);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 6);
}

#[test]
fn rom_timed_main_loop_observes_current_host_frame_input() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(20);
    state.attract_scene_mut().set_state(5);
    state.set_screen_brightness(15);
    state.set_bg_mode(9);

    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.game_state.ending.attract_scene.state(), 9);
    assert_eq!(state.game_state.display.screen_brightness, 14);
}

#[test]
fn rom_timed_audio_commands_written_by_main_wait_for_the_next_nmi() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(20);
    state.attract_scene_mut().set_state(9);
    state.set_screen_brightness(1);
    state.set_bg_mode(9);
    state.set_last_music_control(6);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.system_signals.music_control(), 0xf1);
    assert_eq!(state.game_state.system_signals.last_music_control(), 6);
}

#[test]
fn file_select_initial_graphics_work_resumes_before_the_next_module() {
    assert_eq!(
        rom_file_select_initial_graphics_decision(57),
        (true, false, 56)
    );
    assert_eq!(
        rom_file_select_initial_graphics_decision(2),
        (true, true, 1)
    );
    assert_eq!(
        rom_file_select_initial_graphics_decision(1),
        (false, false, 0)
    );
}

#[test]
fn selected_game_load_resumes_until_the_cpu_heavy_setup_finishes() {
    assert_eq!(rom_selected_game_load_decision(77), (false, 76));
    assert_eq!(rom_selected_game_load_decision(2), (false, 1));
    assert_eq!(rom_selected_game_load_decision(1), (true, 0));
    assert_eq!(rom_selected_game_load_decision(0), (false, 0));
}

#[test]
fn dungeon_landing_wipe_carries_work_into_the_following_display_frame() {
    assert!(rom_dungeon_landing_wipe_is_active(7, 15));
    assert!(!rom_dungeon_landing_wipe_is_active(7, 14));
    assert!(!rom_dungeon_landing_wipe_is_active(14, 15));
}

#[test]
fn normal_dialogue_initialization_is_a_resumable_engine_operation() {
    assert!(rom_normal_dialogue_initialization_is_active(14, 2, 0));
    assert!(!rom_normal_dialogue_initialization_is_active(14, 2, 1));
    assert!(!rom_normal_dialogue_initialization_is_active(20, 2, 0));
}

#[test]
fn rom_intro_poly_thread_begins_on_the_measured_frame() {
    assert_eq!(configured_intro_thread_start_delay(), 0);
}

#[test]
fn rom_intro_poly_thread_remains_concurrent_during_title_fade() {
    for submodule in [3, 4, 5, 7, 9, 11] {
        assert!(rom_intro_poly_thread_is_active(0, submodule));
    }
    assert!(!rom_intro_poly_thread_is_active(0, 6));
    assert!(!rom_intro_poly_thread_is_active(1, 5));
    assert!(rom_intro_wait_player_tears_down_poly_thread(0, 8, true));
    assert!(!rom_intro_wait_player_tears_down_poly_thread(0, 8, false));
    assert!(!rom_intro_wait_player_tears_down_poly_thread(0, 7, true));
    assert_eq!(
        [0, 1, 2, 0, 1, 2]
            .into_iter()
            .map(rom_intro_title_fade_runs_main)
            .collect::<Vec<_>>(),
        vec![true, true, false, true, true, false]
    );
    assert_eq!(
        [0, 1, 2, 0, 1, 2]
            .into_iter()
            .map(rom_intro_title_fade_should_yield_suffix)
            .collect::<Vec<_>>(),
        vec![false, true, false, false, true, false]
    );
}

#[test]
fn rom_intro_background_fade_preserves_cooperative_poly_cadence() {
    let mut carry_frames = 0;
    let mut poly_phase = 0;
    let mut decisions = Vec::new();
    let mut suffix_yields = Vec::new();

    for _ in 0..12 {
        let (run_main, yield_before_suffix, next_carry_frames, next_poly_phase) =
            rom_intro_bg_fade_main_decision(carry_frames, poly_phase);
        decisions.push(run_main);
        suffix_yields.push(yield_before_suffix);
        carry_frames = next_carry_frames;
        poly_phase = next_poly_phase;
    }

    assert_eq!(
        decisions,
        vec![true, true, true, true, true, true, false, true, true, true, true, false]
    );
    assert_eq!(
        suffix_yields,
        vec![false, false, false, false, false, true, false, false, false, false, true, false]
    );
    assert!(rom_intro_bg_fade_should_yield_suffix(true, 2, 5));
    assert!(rom_intro_bg_fade_should_yield_suffix(true, 2, 4));
    assert!(!rom_intro_bg_fade_should_yield_suffix(true, 2, 3));
    assert!(!rom_intro_bg_fade_should_yield_suffix(true, 4, 5));
    assert!(!rom_intro_bg_fade_should_yield_suffix(false, 2, 5));
}

#[test]
fn rom_title_fade_transition_resumes_without_another_main_loop_tick() {
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(5);
    state.set_frame_counter(105);
    state.intro_zelda_fade_transition_pending = true;

    state.complete_intro_zelda_fade_transition();

    assert_eq!(state.game_state.frame.frame_counter, 105);
    assert_eq!(state.game_state.frame.submodule, 6);
    assert_eq!(state.game_state.frame.subsubmodule, 42);
    assert!(!state.intro_zelda_fade_transition_pending);
}

#[test]
fn title_poly_thread_teardown_defers_the_next_main_loop_tick() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(42);
    state.activate_nmi_thread();

    state.intro_sword_coming_down();

    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert!(!state.game_state.display.nmi_thread_active);
    assert!(state.intro_poly_thread_teardown_pending);
}

#[test]
fn rom_intro_waits_for_poly_thread_completion_before_advancing_again() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_startup_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.intro_poly_upload_delay = 0;
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.poly_runtime_mut().set_config1(165);

    state.intro_animate_triforce();

    assert_eq!(state.game_state.poly.runtime.config1(), 165);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
}

#[test]
fn main_loop_does_not_complete_poly_work_that_was_not_scheduled() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_startup_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(4);
    state.set_frame_counter(0x86);
    state.set_bg_mode(9);
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.clear_pending_polyhedral_update();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
    assert!(!state.game_state.display.has_pending_polyhedral_update());
}

#[test]
fn poly_worker_budget_tracks_geometry_cost_instead_of_route_frames() {
    let mut state = ZeldaState::new();
    state.poly_runtime_mut().set_model(1);
    state.poly_runtime_mut().set_base_x(32);
    state.poly_runtime_mut().set_base_y(32);

    state.poly_runtime_mut().set_config1(175);
    state.poly_runtime_mut().set_angle_a(216);
    state.poly_runtime_mut().set_angle_b(104);
    state.poly_run_frame();
    assert_eq!(
        state.debug_last_poly_work(),
        PolyWorkMetrics {
            divide_calls: 12,
            divide_shifts: 12,
            faces: 5,
            visible_faces: 3,
            edge_segments: 11,
            scanlines: 24,
            span_words: 62,
        }
    );
    assert_eq!(state.debug_last_poly_work().worker_frames(), 1);

    state.poly_runtime_mut().set_config1(255);
    state.poly_runtime_mut().set_angle_a(244);
    state.poly_runtime_mut().set_angle_b(236);
    state.poly_run_frame();
    assert_eq!(
        state.debug_last_poly_work(),
        PolyWorkMetrics {
            divide_calls: 12,
            divide_shifts: 24,
            faces: 5,
            visible_faces: 3,
            edge_segments: 9,
            scanlines: 25,
            span_words: 62,
        }
    );
    assert_eq!(state.debug_last_poly_work().worker_frames(), 2);
}

#[test]
fn poly_worker_cost_model_handles_sparse_wide_faces() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0);
    state.set_submodule(4);
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.clear_pending_polyhedral_update();
    state.poly_runtime_mut().set_model(1);
    state.poly_runtime_mut().set_base_x(32);
    state.poly_runtime_mut().set_base_y(32);
    state.poly_runtime_mut().set_config1(67);
    state.poly_runtime_mut().set_angle_a(122);
    state.poly_runtime_mut().set_angle_b(118);

    state.zelda_run_poly_loop();

    assert_eq!(state.debug_last_poly_work().worker_frames(), 1);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        0
    );
    assert!(state.game_state.display.has_pending_polyhedral_update());
}

#[test]
fn game_loop_clears_oam_y_slots_and_keeps_nmi_update_latched() {
    let mut state = ZeldaState::new();
    state.latch_nmi_update();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 1);
    assert!(state.game_state.display.nmi_update_is_latched());
    for i in 4..128 {
        assert_eq!(state.ram[OAM_BUF + i * 4 + 1], 0xf0);
    }
}

#[test]
fn ppu_write_helpers_route_to_ppu_registers() {
    let mut state = ZeldaState::new();

    state.zelda_ppu_write(0x2100, 0x8f);
    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0x0f);

    state.zelda_ppu_write_word(0x2116, 0x1234);
    assert_eq!(state.ppu.vram_pointer, 0x1234);
}

#[test]
fn hdma_setup_and_simple_hdma_line_write_ppu() {
    let mut state = ZeldaState::new();
    state.hdma_setup(0x0cfa87, 0x0cfa94, 0, 0, 0, 0);

    assert_eq!(state.dma.channel[6].a_adr, 0xfa87);
    assert_eq!(state.dma.channel[6].a_bank, 0x0c);
    assert_eq!(state.dma.channel[6].b_adr, 0);
    assert_eq!(state.dma.channel[7].a_adr, 0xfa94);

    state.dma.channel[6].hdma_active = true;
    let mut hdma = SimpleHdma::default();
    state.simple_hdma_init(&mut hdma, &state.dma.channel[6]);
    state.simple_hdma_do_line(&mut hdma);

    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0x0f);
    assert_eq!(hdma.rep_count, 0x1f);
}

#[test]
fn cgram_capture_preserves_dma_state() {
    let mut state = ZeldaState::new();
    state.set_hdma_enable_mask(0);
    state.dma.channel[1].hdma_active = true;
    state.dma.channel[6].hdma_active = true;
    let dma_before = state.dma.save_c_saveload();

    state.cgram_after_first_hdma_line();

    assert_eq!(state.dma.save_c_saveload(), dma_before);
}

#[test]
fn repeated_display_captures_preserve_dma_and_ppu_latches() {
    let mut state = ZeldaState::new();
    state.set_hdma_enable_mask(1 << 6);
    state.hdma_setup(0x0cfa87, 0, 0, 0x0d, 0, 0);
    state.dma.channel[2].hdma_active = true;
    state.dma.channel[7].hdma_active = true;
    state.ppu.scroll_prev = 0x12;
    state.ppu.scroll_prev2 = 0x34;
    let dma_before = state.dma.save_c_saveload();
    let scroll_latches_before = (state.ppu.scroll_prev, state.ppu.scroll_prev2);
    let bg_scrolls_before: [(u16, u16); 4] = std::array::from_fn(|i| {
        (
            state.ppu.bg_layer[i].h_scroll,
            state.ppu.bg_layer[i].v_scroll,
        )
    });
    let mode7_before = (state.ppu.m7_matrix, state.ppu.m7_prev);

    for _ in 0..3 {
        state.cgram_after_first_hdma_line();
        state.ppu_scanline_windows();
        assert_eq!(state.dma.save_c_saveload(), dma_before);
        assert_eq!(
            (state.ppu.scroll_prev, state.ppu.scroll_prev2),
            scroll_latches_before
        );
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|i| {
                (
                    state.ppu.bg_layer[i].h_scroll,
                    state.ppu.bg_layer[i].v_scroll,
                )
            }),
            bg_scrolls_before
        );
        assert_eq!((state.ppu.m7_matrix, state.ppu.m7_prev), mode7_before);
    }
}

#[test]
fn scanline_capture_consumes_one_shot_vcounter_irq() {
    let mut state = ZeldaState::new();
    state.set_irq_control_flag(0x80);
    state.set_select_file_name_scroll_x(0x01f0);
    state.ppu.bg_layer[2].v_scroll = 0x0318;

    let first = state.ppu_scanline_windows();

    assert_eq!(first[126].6[2], 0x0318);
    assert_eq!(first[127].6[2], 0);
    assert_eq!(state.game_state.display.irq_control_flag, 0);
    assert_eq!(state.ram[0x0128], 0);

    let second = state.ppu_scanline_windows();
    assert_eq!(second[127].6[2], 0x0318);
}

#[test]
fn simple_hdma_get_ptr_maps_mode7_zoom_tables() {
    let state = ZeldaState::new();

    assert_eq!(
        state.simple_hdma_get_ptr(0x0add27).unwrap()[0..4],
        [0x77, 0x01, 0x76, 0x01]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0ade07).unwrap()[0..4],
        [0x35, 0x01, 0x35, 0x01]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0adee7).unwrap()[0..4],
        [0x88, 0x00, 0x88, 0x00]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0adfc7).unwrap()[0..4],
        [0x70, 0x00, 0x70, 0x00]
    );
}

#[test]
fn draw_ppu_frame_applies_mode7_perspective_correction() {
    let mut state = ZeldaState::new();
    let mut pixels = vec![0u8; 256 * 224 * 4];
    state.ppu.mode = 7;
    state.set_hdma_enable_mask(1 << 6);
    state.hdma_setup(0x0abdcf, 0, 0, 0, 0, 0x0a);

    state.zelda_draw_ppu_frame(&mut pixels, 256 * 4, PpuRenderFlags::MODE7_4X4);

    assert_eq!(state.ppu.mode7_perspective_low, 1.0 / 375.0);
    assert_eq!(state.ppu.mode7_perspective_high, 1.0 / 264.0);
}

#[test]
fn configure_ppu_side_space_matches_module_cases() {
    let mut state = ZeldaState::new();
    state.set_main_module(20);
    state.configure_ppu_side_space();
    assert_eq!(state.ppu.extra_left_cur, PPU_SIDE_SPACE_LIMIT as u8);
    assert_eq!(state.ppu.extra_right_cur, PPU_SIDE_SPACE_LIMIT as u8);
    assert_eq!(state.ppu.extra_bottom_cur, 16);

    state.set_main_module(7);
    state.set_bg2_x(0x0110);
    state.set_bg2_y(0x0108);
    state.room_bounds_mut().set_x_bound(0, 0x0100);
    state.room_bounds_mut().set_x_bound(2, 0x0140);
    state.room_bounds_mut().set_y_bound(2, 0x0120);
    state.ram[QUADRANT_FULLSIZE_X] = 0;
    state.ram[QUADRANT_FULLSIZE_Y] = 0;
    state.configure_ppu_side_space();
    assert_eq!(state.ppu.extra_left_cur, 0x10);
    assert_eq!(state.ppu.extra_right_cur, 0x30);
    assert_eq!(state.ppu.extra_bottom_cur, 16);
}

#[test]
fn draw_ppu_frame_runs_irq_and_hdma_side_effects() {
    let mut state = ZeldaState::new();
    let mut pixels = vec![0u8; 256 * 224 * 4];
    state.set_irq_control_flag(0x80);
    state.set_select_file_name_scroll_x(0x01f0);
    state.set_hdma_enable_mask(1 << 6);
    state.hdma_setup(0x0cfa87, 0, 0, 0, 0, 0);

    state.zelda_draw_ppu_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());

    assert_eq!(state.game_state.display.irq_control_flag, 0);
    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0x0f);
    assert_eq!(state.ppu.render_pitch, (PPU_X_PIXELS * 4) as u32);
    assert_eq!(
        state.ppu.render_buffer.as_ref().unwrap().len(),
        PPU_X_PIXELS * (224 + 1) * 4
    );
    assert_eq!(pixels.len(), 256 * 224 * 4);
}

#[test]
fn display_snapshot_draw_uses_c_style_current_vram_not_obj_latch() {
    let mut state = ZeldaState::new();
    let mut pixels = vec![0u8; 256 * 224 * 4];

    state.ppu.obj_vram_latch = Some(vec![0x1111; VRAM_WORDS]);
    state.capture_display_snapshot();
    state.zelda_draw_display_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());
    assert!(state.ppu.obj_vram_latch.is_none());

    state.ppu.obj_vram_latch = Some(vec![0x1111; VRAM_WORDS]);
    state.obj_vram_latch_generation = 1;
    state.capture_display_snapshot();
    state.obj_vram_latch_generation = 2;
    state.ppu.obj_vram_latch = Some(vec![0x2222; VRAM_WORDS]);
    state.zelda_draw_display_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());
    assert!(state.ppu.obj_vram_latch.is_none());
}

#[test]
fn first_intro_step_matches_top_level_state_writes() {
    let mut state = ZeldaState::new();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.frame.subsubmodule, 1);
    assert_eq!(state.game_state.display.screen_brightness, 15);
    assert_eq!(state.game_state.display.main_screen_layers, 16);
    assert_eq!(state.game_state.display.bg_mode, 9);
    assert_eq!(
        state
            .game_state
            .display
            .palette_filter
            .color_window_selection(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.color_math_control(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_red(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_green(),
        0x40
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_blue(),
        0x80
    );
    assert_eq!(state.game_state.display.core_update_disable_flag, 0x80);
    assert_eq!(state.game_state.display.nmi_load_target_page(), 0x46);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x1bfe);
    assert_eq!(
        state.game_state.dungeon.scratch_word.secondary_word(),
        0x17fe
    );
    assert_eq!(
        &state.ram[OAM_BUF..OAM_BUF + 16],
        &[
            0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
            0x6e, 0x32
        ]
    );
    assert_eq!(
        &state.ram[BYTEWISE_EXTENDED_OAM..BYTEWISE_EXTENDED_OAM + 4],
        &[2; 4]
    );
    assert_eq!(state.ram[EXTENDED_OAM], 0xaa);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_3), 0x8080);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_0), 0x8280);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_4), 0x8840);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_1), 0x8a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_5), 0x9a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_2), 0x9a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_6), 0x9000);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_11), 0x9180);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_7), 0x9300);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_12), 0x93c0);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_8), 0x9480);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_13), 0x9560);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_10), 0xa480);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_15), 0xa580);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_16), 0xb940);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_18), 0xbb40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_17), 0xb940);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_19), 0xbb40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_20), 0xb540);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_21), 0xb740);
    assert_eq!(read_le_u16(&state.ram, BG_TILE_ANIMATION_COUNTDOWN), 0xffff);
    assert_eq!(link_test_word(&state, LINK_DMA_COUNTDOWN), 0xffff);
    assert_eq!(state.ppu.cgram[144], 0x7fff);
}

#[test]
fn graphics_half_slot_transforms_uncompressed_sprite_pack() {
    let mut state = ZeldaState::new();
    let mut pack = vec![0; 0x300 + 24 * 32];
    for i in 0..24 * 32 {
        pack[0x300 + i] = i as u8;
    }
    let mut data = vec![0; 8 * 2];
    data.extend_from_slice(&pack);
    data.extend_from_slice(&8u16.to_le_bytes());
    let mut ranges = vec![(0, 0); 65];
    ranges[64] = (0, data.len());
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    state.set_chr_halfslot_request(20);
    state.graphics_load_chr_half_slot();

    assert_eq!(state.game_state.display.nmi_load_target_page(), 0x46);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
    assert_eq!(&state.ram[0x11000..0x11004], &[0, 1, 2, 3]);
    assert_eq!(&state.ram[0x11010..0x11014], &[16, 17, 17, 19]);
    assert_eq!(&state.ram[0x11020..0x11024], &[24, 25, 26, 27]);
}

#[test]
fn nmi_subroutine_11_uploads_bg_char_half_to_vram() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, 0x11000, 0x1234);
    write_le_u16(&mut state.ram, 0x11002, 0xabcd);
    state.set_nmi_load_target_page(0x46);
    state.set_pending_nmi_subroutine(11);

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[0x4600], 0x1234);
    assert_eq!(state.ppu.vram[0x4601], 0xabcd);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
}

#[test]
fn intro_submodule_one_continues_memory_clear_and_logo_oam() {
    let mut state = ZeldaState::new();
    state.run_frame_internal(0, crate::RUN_MAIN);
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.frame.subsubmodule, 2);
    assert_eq!(
        &state.ram[OAM_BUF..OAM_BUF + 16],
        &[
            0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
            0x6e, 0x32
        ]
    );
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x17fe);
    assert_eq!(
        state.game_state.dungeon.scratch_word.secondary_word(),
        0x13fe
    );
}

#[test]
fn intro_fade_in_bg_start_skips_to_file_select_loader() {
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(7);
    state.set_subsubmodule(0xf3);
    state.set_countdown(0);
    state.follower_link_state_mut().set_filtered_joypad_h(0x10);
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_Y_COORD, 0x12);
    state.ram[LINK_Y_COORD + 0x6f] = 0x34;
    state.save_progress_mut().set_dungeon_info_word(0, 0x56);

    state.module00_intro();

    assert_eq!(state.game_state.display.irq_control_flag, 0xff);
    assert_eq!(state.game_state.display.main_screen_layers, 0x15);
    assert_eq!(state.game_state.display.sub_screen_layers, 0);
    assert_eq!(state.game_state.world.location.indoor_flag(), 0);
    assert_eq!(state.game_state.system_signals.music_control(), 0xf1);
    assert_eq!(state.game_state.frame.main_module, 1);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.ram[RESTART_CHECK_FLAG], 1);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD), 0);
    assert_eq!(state.ram[LINK_Y_COORD + 0x6f], 0);
    assert_eq!(state.ram[SAVE_DUNG_INFO], 0);
}

#[test]
fn name_file_x_scroll_both_horizontal_bits_match_c_rom_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x03);
    state.set_select_file_name_column(21);

    state.name_file_check_for_scroll_input_x();

    let select_file = &state.game_state.messaging.select_file_menu;
    assert_eq!(select_file.name_column(), 53);
    assert_eq!(select_file.name_scroll_x_step(), 1);
    assert_eq!(select_file.name_scroll_x_direction(), 2);
}
