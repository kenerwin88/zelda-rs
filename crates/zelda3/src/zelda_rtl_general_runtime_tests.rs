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
    DMA_SOURCE_ADDR_7, DMA_SOURCE_ADDR_8, DMA_SOURCE_ADDR_9, DUNG_BG2, TM_COPY, TS_COPY,
};
use crate::game_state::constants::{MAP16_LOAD_DST_OFF, MAP16_LOAD_SRC_OFF, MAP16_LOAD_Y_UNIT};
use crate::game_state::FrameState;

#[test]
fn attract_map_mode7_brightness_override_ends_with_fade_in() {
    assert!(rom_attract_world_map_mode7_brightness_is_early_published(
        20, 0, 1, 4
    ));
    assert!(!rom_attract_world_map_mode7_brightness_is_early_published(
        20, 0, 1, 5
    ));
}

#[test]
fn attract_map_exit_drains_the_final_projection_before_the_tilemap_clear_returns() {
    let pending = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractWorldMapExit,
        ATTRACT_WORLD_MAP_EXIT_NMI_SLICES,
    );
    assert_eq!(
        pending.in_flight_display_snapshot_publication_override(),
        Some(DisplaySnapshotPublication::AdvanceStaged)
    );
}

#[test]
fn attract_map_projection_generation_follows_the_cpu_hdma_race() {
    let first_current = (0..ATTRACT_MAP_PROJECTION_WORDS)
        .find(|&line| attract_map_projection_current_word_is_visible(line));
    assert_eq!(first_current, Some(45));

    let mut ram = vec![0x55; 0x20000];
    let before_projection = vec![0xaa; ZeldaState::HDMA_DYNAMIC_TABLE_LEN];
    DisplayHdmaTableGeneration::AttractMapProjectionDuringScanout { before_projection }
        .compose_into(&mut ram);

    assert_eq!(
        &ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 45 * 2],
        vec![0xaa; 45 * 2]
    );
    assert_eq!(
        &ram[HDMA_TABLE_DYNAMIC + 45 * 2..HDMA_TABLE_DYNAMIC + ATTRACT_MAP_PROJECTION_WORDS * 2],
        vec![0x55; (ATTRACT_MAP_PROJECTION_WORDS - 45) * 2]
    );
}

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
fn deserialized_asset_pack_caches_dialogue_semantic_sidecar_after_first_read() {
    let sidecar =
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]);
    let bytes = test_asset_pack_bytes(&[(DIALOGUE_SOURCE_SIDECAR_ASSET_NAME, sidecar)]);
    let asset_pack = AssetPack::parse(&bytes).unwrap();
    let restored: AssetPack =
        bincode::deserialize(&bincode::serialize(&asset_pack).unwrap()).unwrap();

    assert!(restored.dialogue_source_ir_table.get().is_none());

    let source_ir = restored.source_dialogue_ir_for_message(0).unwrap();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::EndMessage);
    assert!(restored.dialogue_source_ir_table.get().is_some());
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
fn asset_pack_resolves_spc_driver_timing_program_by_name() {
    let pack = AssetPack::parse(&test_asset_pack_bytes(&[
        ("kUnused", vec![0]),
        ("kSpcDriverTimingProgram", vec![0x20, 0xcd, 0xcf, 0xbd]),
    ]))
    .unwrap();

    assert_eq!(
        pack.asset_by_name("kSpcDriverTimingProgram"),
        Some(&[0x20, 0xcd, 0xcf, 0xbd][..])
    );
    assert_eq!(pack.asset_by_name("missing"), None);
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
fn new_state_exposes_first_nmi_link_dma_source() {
    let state = ZeldaState::new();

    assert_eq!(&state.ram[0..3], &[0x00, 0x80, 0x00]);
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
fn overworld_map16_stripes_follow_rom_long_indexed_wram_reads_past_bg2_page() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 71];
    put_test_asset(&mut data, &mut ranges, 70, vec![0; 9 * 8]);

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state.set_overworld_map16_load_state(OverworldMap16LoadState {
        src_off: 0x1802,
        dst_off: 0x001a,
        y_unit: 0x0008,
    });
    state.set_screen_transition_direction_bits(1);

    let crossed_page_words = [
        0x0dc4, 0x0c68, 0x0270, 0x0271, 0x0c6c, 0x0272, 0x0273, 0x0c69,
    ];
    for (index, value) in crossed_page_words.into_iter().enumerate() {
        let source_offset = 0x2032 + index * 0x80;
        write_le_u16(&mut state.ram, DUNG_BG2 + source_offset, value);
    }

    state.BufferAndBuildMap16Stripes_X(0);

    let captured = std::array::from_fn(|index| {
        state
            .game_state
            .world
            .transient
            .dung_replacement_tile_state(index)
    });
    assert_eq!(captured, crossed_page_words);
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
    assert_eq!(state.state_recorder.last_inputs, 0x00a1);
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
fn room_61_all_rescue_follower_trigger_zones_open_the_message() {
    for (x, y, event_bit) in [
        (0x025b, 0x0cf8, 1),
        (0x039d, 0x0cf8, 2),
        (0x0238, 0x0c78, 4),
    ] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_submodule(0);
        state.set_rom_startup_timing(true);
        state.set_indoor_flag(1);
        state.set_dungeon_room(0x0061);
        state.follower_state_mut().set_indicator(1);
        state.follower_link_state_mut().set_x(x);
        state.follower_link_state_mut().set_y(y);

        state.follower_handle_trigger();

        assert_eq!(state.game_state.frame.main_module, 14);
        assert_eq!(state.game_state.frame.submodule, 2);
        assert_eq!(
            state.game_state.messaging.dialogue_message_index.value(),
            0x21
        );
        assert_eq!(
            state.game_state.sprites.follower_runtime.event_flags(),
            event_bit
        );
        assert!(state.next_display_obj_memory_generation.is_none());
        assert_eq!(
            state.next_display_obj_scanout_generation.is_some(),
            event_bit == 2
        );
        if event_bit == 2 {
            assert_eq!(
                state.next_display_obj_scanout_generation,
                Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
                    link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                })
            );
        }
    }
}

#[test]
fn room_61_rescue_follower_trigger_rejects_a_different_follower() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_indoor_flag(1);
    state.set_dungeon_room(0x0061);
    state.follower_state_mut().set_indicator(2);
    state.follower_link_state_mut().set_x(0x039d);
    state.follower_link_state_mut().set_y(0x0cf8);

    state.follower_handle_trigger();

    assert_eq!(state.game_state.frame.main_module, 7);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.game_state.sprites.follower_runtime.event_flags(), 0);
    assert!(state.next_display_obj_memory_generation.is_none());
    assert!(state.next_display_obj_scanout_generation.is_none());
}

#[test]
fn rescue_follower_message_obj_selector_handles_rollover_and_rejects_neighbors() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };

    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 2, 0x21),
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(
            crate::game_state::FrameState {
                frame_counter: 0xff,
                ..frame
            },
            0x61,
            2,
            0x21
        ),
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 1, 0x21),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 2, 0x22),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x52, 2, 0x21),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(
            crate::game_state::FrameState {
                main_module: 14,
                submodule: 2,
                ..Default::default()
            },
            0x61,
            2,
            0x21
        ),
        None
    );
}

#[test]
fn rescue_follower_message_rollover_stages_live_link_graphics_only() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.game_state.frame.frame_counter = 0xff;
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(1);
    state.set_dungeon_room(0x0061);
    state.follower_state_mut().set_indicator(1);
    state.follower_link_state_mut().set_x(0x039d);
    state.follower_link_state_mut().set_y(0x0cf8);
    state.follower_handle_trigger();

    assert!(state.next_display_obj_memory_generation.is_none());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );
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
fn link_velocity_coordinate_boundary_resumes_to_the_atomic_c_result() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(0x0f);
        state.set_submodule(1);
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        set_link_test_byte(&mut state, LINK_DIRECTION, 8);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
        state.follower_link_state_mut().set_speed_setting(6);
        state
    }

    let mut atomic = configured_state();
    atomic.link_handle_velocity();

    let mut resumed = configured_state();
    let position_return = resumed
        .link_handle_velocity_until_position_integrated()
        .expect("ordinary Module0F movement must reach Player_MovePosition1_");

    // Player_MovePosition1_ has integrated coordinates, but the C calls to
    // moving-floor, conveyor, and drag/velocity-delta handling are still on
    // the suspended stack.
    assert_eq!(link_test_byte(&resumed, LINK_Y_VEL), 0);
    resumed.complete_link_move_position_after_coordinates(position_return);

    assert_eq!(resumed.ram, atomic.ram);
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
    // Exact Snes9x DMA trace at the first NMI upload: channels 3 and 8 read
    // from $7E:0000. These pointers remain zero until ROM code assigns them.
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_9), 0);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_14), 0);
    assert_eq!(&state.ram[0..3], &[0x00, 0x80, 0x00]);
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

    assert_eq!(state.rom_reset_frame_delay, 81);
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
fn dialogue_scroll_checkpoint_projection_preserves_stable_phases_and_normalizes_transients() {
    fn round_trip(state: &ZeldaState) -> ZeldaState {
        bincode::deserialize(
            &bincode::serialize(state).expect("serialize dialogue-scroll checkpoint"),
        )
        .expect("deserialize dialogue-scroll checkpoint")
    }

    fn scrolling_state(completion_timing: DialogueScrollCompletionTiming) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.ppu.vram[0x7c00] = 0x1234;
        state.begin_dialogue_scroll(DialogueTextGeneration::PublishedDisplay, completion_timing);
        state
    }

    let copying = round_trip(&scrolling_state(
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    ));
    assert_eq!(
        copying.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    assert_eq!(
        copying
            .dialogue_scroll_frozen_scanout
            .as_ref()
            .unwrap()
            .vram[0],
        0x1234
    );

    let mut return_only = scrolling_state(DialogueScrollCompletionTiming::AfterReturnBoundary);
    return_only.finish_dialogue_scroll_remaining_pixels();
    let return_only = round_trip(&return_only);
    assert_eq!(
        return_only.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly
    );

    let mut pending = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    pending.finish_dialogue_scroll_remaining_pixels();
    let pending = round_trip(&pending);
    assert_eq!(
        pending.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );

    let mut staged = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    staged.finish_dialogue_scroll_remaining_pixels();
    staged.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    let mut staged = round_trip(&staged);
    staged.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        staged.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );

    let mut completed = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    completed.finish_dialogue_scroll_remaining_pixels();
    completed.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    completed.advance_dialogue_scroll_display_boundary();
    let mut completed = round_trip(&completed);
    completed.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        completed.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );

    let mut staged_after_return =
        scrolling_state(DialogueScrollCompletionTiming::AfterReturnBoundary);
    staged_after_return.finish_dialogue_scroll_remaining_pixels();
    staged_after_return.finish_dialogue_scroll_return();
    staged_after_return
        .stage_dialogue_scroll_completion_after_return(DialogueTextScanout::default());
    let mut staged_after_return = round_trip(&staged_after_return);
    staged_after_return.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        staged_after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot
    );
}

#[test]
fn rom_startup_holds_only_the_interrupted_intro_initialization_frame() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 2);
}

#[test]
fn first_rom_frame_publishes_intro_audio_on_the_following_nmi() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.zelda_debug_apu_write_ports(), [0, 0, 0, 0]);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.zelda_debug_apu_write_ports(), [0, 0, 0, 10]);
}

#[test]
fn rom_intro_memory_initialization_wait_matches_live_console_timing() {
    assert_eq!(configured_intro_memory_initialization_frames(), 41);
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
fn game_execution_scheduler_transitions_between_typed_continuations() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 2);
    assert_eq!(
        scheduler.current_work(),
        Some(GameWorkContinuation::FinishAttractThroneRoom)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishAttractThroneRoom
        ))
    );

    scheduler.schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert!(!scheduler.is_idle());
    assert_eq!(
        scheduler.take_pre_main_nmi_resume(),
        Some(PreMainNmiResume::OverworldAuxGraphicsReturn)
    );

    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert!(scheduler.pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn));
    scheduler.finish_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert_eq!(scheduler, GameExecutionScheduler::default());
}

#[test]
fn game_execution_scheduler_preserves_non_work_continuations_when_advanced() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishAttractThroneRoom
        ))
    );

    scheduler.schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert_eq!(
        scheduler.take_pre_main_nmi_resume(),
        Some(PreMainNmiResume::OverworldAuxGraphicsReturn)
    );

    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert!(scheduler.pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn));
    scheduler.finish_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);

    scheduler.schedule_file_select_graphics();
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::FileSelectWaiting)
    );
    scheduler.reset();

    scheduler.schedule_selected_game_load();
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::SelectedGameLoadWaiting)
    );
}

#[test]
fn post_trailing_nmi_work_cannot_be_resumed_at_the_next_host_entry() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_post_trailing_nmi(GameWorkContinuation::FinishDungeonMapRoomDrawing);

    assert!(scheduler.work_is_pending());
    assert!(scheduler.work_suspends_translated_call_stack());
    assert_eq!(
        scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonMapRoomDrawing)
    );
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.take_post_trailing_nmi(),
        Some(GameWorkContinuation::FinishDungeonMapRoomDrawing)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn dungeon_map_room_drawing_uses_the_post_trailing_nmi_phase() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_dungeon_room_index(0x41);

    state.Module0E_03_01_03_DrawRooms();

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonMapRoomDrawing)
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice(),
        None,
        "the one-NMI room builder returns after the current trailing NMI, not at the next host entry",
    );
}

#[test]
fn room_72_dungeon_map_room_drawing_remains_suspended_through_next_host_entry() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_dungeon_room_index(0x72);

    state.Module0E_03_01_03_DrawRooms();

    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonMapRoomDrawing,
        )),
    );
}

#[test]
fn paired_resume_requires_every_execution_continuation_to_be_idle() {
    let mut state = ZeldaState::new();
    assert!(state.paired_resume_cpu_boundary_is_quiescent());

    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_file_select_graphics();
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state.game_execution_scheduler.schedule_selected_game_load();
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
}

#[test]
#[should_panic(expected = "cannot schedule")]
fn game_execution_scheduler_rejects_parallel_continuation_kinds() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
}

#[test]
fn throne_room_rom_work_resumes_only_after_every_intervening_nmi_slice() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractThroneRoom,
        ATTRACT_THRONE_ROOM_NMI_SLICES,
    );

    for _ in 0..ATTRACT_THRONE_ROOM_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractThroneRoom)
    );
    assert!(work.is_complete());
}

#[test]
fn dungeon_falling_entrance_work_resumes_at_measured_cpu_boundaries() {
    let stages = [
        DungeonFallingEntranceWork::RoomAndTilesets,
        DungeonFallingEntranceWork::SpriteGraphics,
    ];

    for stage in stages {
        let continuation = GameWorkContinuation::FinishDungeonFallingEntrance { work: stage };
        let mut work = ScheduledGameWork::schedule(continuation, stage.nmi_slices());
        for _ in 0..stage.nmi_slices() - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
    }
}

#[test]
fn dungeon_supertile_transition_resumes_at_rom_call_boundaries() {
    let mut state = ZeldaState::new();
    assert!(state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::RoomLoad,
        },
        DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
    );
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());

    let stages = [
        (
            DungeonSupertileTransitionWork::RoomLoad,
            DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
            DUNGEON_SUPERTILE_AUX_SPRITE_GFX_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpriteConversion,
            DUNGEON_SUPERTILE_SPRITE_CONVERSION_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::RoomLoadCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpriteConversionCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
            DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
            DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
            DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpiralRoomCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpiralBgCharacters34,
            DUNGEON_SPIRAL_BG_CHARACTERS_34_NMI_SLICES,
        ),
    ];

    for (stage, nmi_slices) in stages {
        assert_eq!(stage.nmi_slices(), nmi_slices);
        let continuation = GameWorkContinuation::FinishDungeonSupertileTransition { work: stage };
        let mut work = ScheduledGameWork::schedule(continuation, stage.nmi_slices());
        assert!(work.suspends_translated_call_stack());
        for _ in 0..stage.nmi_slices() - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
        assert!(work.is_complete());
    }
    assert!(!DungeonSupertileTransitionWork::RoomLoadCallerResume
        .next_module_resumes_after_pre_main_nmi());
    assert!(DungeonSupertileTransitionWork::SpriteConversionCallerResume
        .next_module_resumes_after_pre_main_nmi());
    assert!(!DungeonSupertileTransitionWork::SpiralRoomCallerResume
        .next_module_resumes_after_pre_main_nmi());
}

#[test]
fn spiral_initializer_schedule_does_not_depend_on_room_or_staircase_identity() {
    for (room, staircase) in [(0x42, 0x34), (0x42, 0x30), (0x41, 0x34)] {
        let mut state = spiral_cpu_test_state(3, injected_dungeon_cpu_schedule(2, 0));
        state.set_dungeon_room_index(room);
        state
            .dungeon_stair_movement_mut()
            .set_staircase_index(staircase);
        assert!(state.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpiralRoomInitialization,
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
}

#[test]
fn straight_interroom_room_initialization_crosses_nineteen_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(2);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
    ));
    assert!(matches!(
        state.active_display_obj_generation,
        DisplayObjGeneration::RetainCapturedOam { .. }
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
            },
        )),
    );
}

#[test]
fn straight_interroom_bg_character_conversion_crosses_four_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(3);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
            },
        )),
    );
}

#[test]
fn straight_interroom_sprite_graphics_cross_four_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(9);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
            },
        )),
    );
}

#[test]
fn straight_interroom_state5_waits_for_its_leading_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x51);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);

    state.schedule_straight_interroom_state5_after_leading_nmi();
    assert_eq!(
        state.game_execution_scheduler.take_pre_main_nmi_resume(),
        Some(PreMainNmiResume::StraightInterroomState5LeadingNmi),
    );
}

#[test]
fn room_32_staircase_35_sprite_reset_exposes_disabled_slots_for_one_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x32);
    state.dungeon_stair_movement_mut().set_staircase_index(0x35);
    state.sprite_slot_view_mut(0).set_state(9);
    state.sprite_slot_view_mut(0).set_sprite_type(0x6e);

    assert!(state.suspend_straight_interroom_sprite_reset_before_room_load());
    assert_eq!(state.sprite_slot_view(0).state(), 0);
    assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x6e);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishStraightInterroomSpriteReset),
    );

    let mut unmeasured = ZeldaState::new();
    unmeasured.restore_live_rom_timing_after_checkpoint();
    unmeasured.set_main_module(7);
    unmeasured.set_submodule(0x12);
    unmeasured.set_subsubmodule(5);
    unmeasured.set_dungeon_room_index(0x51);
    unmeasured
        .dungeon_stair_movement_mut()
        .set_staircase_index(0x30);
    assert!(!unmeasured.suspend_straight_interroom_sprite_reset_before_room_load());
}

#[test]
fn room_82_supertile_load_crosses_one_additional_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_dungeon_room_index(0x82);

    assert!(
        state.begin_dungeon_supertile_transition_work(DungeonSupertileTransitionWork::RoomLoad,)
    );
    for _ in 0..DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::RoomLoad,
            },
        )),
    );
}

#[test]
fn room_72_supertile_room_load_caller_resume_crosses_one_additional_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_dungeon_room_index(0x72);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::RoomLoadCallerResume,
    ));
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
            },
        )),
    );
}

#[test]
fn c_room_load_caller_clears_12_before_the_next_leading_nmi() {
    // C ZeldaRunGameLoop returns through NMI_PrepareSprites and then executes
    // `nmi_boolean = 0` before waiting. The next Interrupt_NMI therefore runs
    // NMI_DoUpdates exactly once; hardware cadence belongs to the scheduler,
    // not to a synthetic retained software latch.
    let continuation = GameWorkContinuation::FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
    };
    let mut state = ZeldaState::new();
    state
        .game_execution_scheduler
        .schedule_work(continuation, 1);
    state.game_execution_scheduler.begin_host_frame();
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(continuation)),
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());

    state.latch_nmi_update();
    state.finish_dungeon_room_load_caller_at_main_wait();

    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());

    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .main_return_requires_leading_nmi());

    // nmi.c NMI_DoUpdates performs one complete $220-byte OAM copy and
    // consumes pending BG work when that cleared latch reaches the NMI.
    let expected_oam = (0..0x220)
        .map(|index| (index as u8).wrapping_mul(37).wrapping_add(11))
        .collect::<Vec<_>>();
    state.ram[OAM_BUF..OAM_BUF + expected_oam.len()].copy_from_slice(&expected_oam);
    state.ppu.oam.fill(0xdead);
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(0x22);
    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);

    let resident_oam = state
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    assert_eq!(resident_oam, expected_oam);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert!(state.game_state.display.nmi_update_is_latched());
}

#[test]
fn dungeon_room_load_and_sprite_conversion_route_contracts() {
    assert_eq!(
        PreMainNmiResume::DungeonSupertileQuadrantUploads.nmi_latch_clear_phase(),
        Some(NmiPhase::BeforeNmi),
    );
    assert_eq!(
        PreMainNmiResume::OverworldAuxGraphicsReturn.nmi_latch_clear_phase(),
        None,
    );

    let room_load_return = FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 2,
        ..FrameState::default()
    };
    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        room_load_return,
        0x61,
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        room_load_return,
        0x52,
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        FrameState {
            subsubmodule: 3,
            ..room_load_return
        },
        0x61,
    ));

    assert!(room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversion,
        0x61,
    ));
    assert!(room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversionCallerResume,
        0x61,
    ));
    assert!(!room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversion,
        0x52,
    ));
    assert!(!room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
        0x61,
    ));
}

#[test]
fn filtered_state_9_interrupts_the_caller_suffix_from_cpu_work() {
    assert_eq!(
        dungeon_supertile_state_9_cpu_advance(None),
        CpuPhaseSequenceAdvance::Complete,
    );
    assert!(matches!(
        dungeon_supertile_state_9_cpu_advance(Some(168_420)),
        CpuPhaseSequenceAdvance::InterruptedAtNmi { phase_index: 1, .. }
    ));

    let entry = FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 9,
        ..FrameState::default()
    };
    let exit = FrameState {
        subsubmodule: 10,
        ..entry
    };
    assert_eq!(
        dungeon_supertile_state_9_caller_continuation(entry, exit, Some(168_420)),
        Some(PreMainNmiResume::DungeonSupertileCallerReturnNmi),
    );
    assert_eq!(
        PreMainNmiResume::DungeonSupertileCallerReturnNmi.nmi_latch_clear_phase(),
        Some(NmiPhase::AfterNmi),
    );
    assert_eq!(
        dungeon_supertile_state_9_caller_continuation(entry, exit, None),
        None,
    );
}

#[test]
fn filtered_state_10_interrupts_the_quadrant_body_from_cpu_work() {
    assert!(matches!(
        dungeon_supertile_state_10_cpu_advance(187_620),
        CpuPhaseSequenceAdvance::InterruptedAtNmi { phase_index: 0, .. }
    ));
}

#[test]
fn room_load_sprite_main_preserves_every_measured_nmi_slice() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_dungeon_room_index(0x22);
    for slot in 0..=6 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterSlot(5));
    state.sprite_main_cpu_nmi_slices = 4;

    state.sprite_main();

    assert_eq!(state.sprite_slot_view(6).state(), 9);
    assert_eq!(state.sprite_slot_view(5).state(), 9);
    for slot in 0..5 {
        assert_eq!(state.sprite_slot_view(slot).state(), 8);
    }
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(5),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
    for _ in 0..3 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::AfterSlot(5),
                caller: SpriteMainCpuCaller::DungeonModule07,
            },
        )),
    );
}

#[test]
fn room_72_quadrant_builder_completes_inside_rom_timed_dispatcher_iteration() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x72);

    state.Dungeon_InterRoomTrans_notDarkRoom();

    assert_eq!(state.game_state.frame.subsubmodule, 6);
    assert!(state.paired_resume_cpu_boundary_is_quiescent());
}

#[test]
fn filtered_state_10_quadrant_work_is_not_room_specific() {
    for room in [0x21, 0x22, 0x41] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(10);
        state.set_dungeon_room_index(room);

        assert!(state.begin_dungeon_supertile_transition_work_with_palette(
            DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild,
            Some(187_620),
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild,
                },
            )),
        );
    }
}

#[test]
fn filtered_state_11_quadrant_work_is_not_room_specific() {
    assert!(matches!(
        dungeon_supertile_state_11_cpu_advance(189_976),
        CpuPhaseSequenceAdvance::InterruptedAtNmi { phase_index: 0, .. }
    ));

    for room in [0x21, 0x22, 0x41] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(11);
        state.set_dungeon_room_index(room);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.set_countdown_word(29);
        for index in [0..1, 0x20..0xd8, 0xe0..0xf0].into_iter().flatten() {
            state.set_aux_color_constant(index, 0x7fff);
        }
        let palette_work = palette_filter_bounce_loop_master_cycles(&state);
        assert!(matches!(
            dungeon_supertile_state_11_cpu_advance(palette_work),
            CpuPhaseSequenceAdvance::InterruptedAtNmi { phase_index: 0, .. }
        ));

        state.Dungeon_InterRoomTrans_State9();

        assert_eq!(state.game_state.frame.subsubmodule, 11);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::QuadrantUploadCallerReturn,
            })
        );
    }
}

#[test]
fn module_7_cpu_model_starts_before_translated_state_mutations() {
    assert_eq!(DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc, 0x00_8051);
    assert_eq!(DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.stop_pc, 0x00_8036);
}

#[test]
fn state_13_suspends_common_module_suffix_when_rom_run_reaches_nmi_after_module() {
    for phase in [
        ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        ModuleCpuPhase::InterruptedAfterModule,
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(13);
        state.set_dungeon_room_index(0x41);
        state.set_countdown_word(24);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 13,
            palette_countdown: 24,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Dungeon_InterRoomTrans_State13();

        assert_eq!(state.game_state.frame.subsubmodule, 13);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::State13CallerReturn,
            })
        );
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
    }

    let state = ZeldaState::new();
    let completion =
        GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::State13CallerReturn,
        });
    assert!(scheduled_work_completion_clears_nmi_latch_after_interrupt(
        completion
    ));
    assert_eq!(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::State13CallerReturn,
        }
        .completion_publication(BgScrollRegisterScanout::capture(&state.ppu)),
        GameWorkCompletionPublication {
            bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
            obj: Some(ObjScanoutGenerations {
                oam: OamScanoutSource::RetainCapturedBeforeNmi,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            }),
        }
    );
}

#[test]
fn state_13_nmi_after_sprite_main_resumes_only_the_common_suffix() {
    for phase in [
        ModuleCpuPhase::InterruptedAfterSpriteMain,
        ModuleCpuPhase::InterruptedInLinkOam,
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(13);
        state.set_dungeon_room_index(0x72);
        state.set_countdown_word(26);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 13,
            palette_countdown: 26,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });
        state.game_execution_scheduler.begin_host_frame();
        state.game_execution_scheduler.begin_main_loop_iteration();

        state.Dungeon_InterRoomTrans_State13();
        assert_eq!(state.game_state.frame.subsubmodule, 13);
        assert!(state.dungeon_post_sprite_main_return_pending);
        assert!(state.game_execution_scheduler.is_idle());

        state.complete_module07_dungeon_after_submodule();
        assert!(!state.dungeon_post_sprite_main_return_pending);
        assert!(state.active_dungeon_sprite_main_return.is_some());
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
        );
    }
}

#[test]
fn state_13_caller_return_accepts_post_call_state_14_in_any_room() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x42);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::State13CallerReturn,
    ));
}

#[test]
fn state_12_nmi_inside_sprite_preparation_resumes_only_that_caller() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(12);
    state.set_overworld_map_state(5);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 13,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 13);
    assert!(!state.dungeon_nmi_prepare_sprites_return_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonNmiPrepareSpritesCallerReturn),
    );
}

#[test]
fn state_13_finishes_common_module_suffix_when_rom_run_completes_before_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(13);
    state.set_dungeon_room_index(0x41);
    state.set_countdown_word(22);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 13,
        palette_countdown: 22,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Dungeon_InterRoomTrans_State13();

    assert_eq!(state.game_state.frame.subsubmodule, 13);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.dungeon_state_13_atomic_caller_return_publication_host_frame,
        Some(state.frame_ctr_dbg)
    );
}

#[test]
fn spiral_return_main_loop_reentry_advances_frame_and_clears_oam() {
    let mut state = ZeldaState::new();
    state.set_frame_counter(0x12);
    state.oam_state_mut().set_entry_y(OAM_BUF, 0x44);
    state.begin_spiral_stair_return_main_loop_reentry();

    assert_eq!(FrameState::load_from_ram(&state.ram).frame_counter, 0x13);
    assert_eq!(state.ram[OAM_BUF + 1], 0xf0);
}

#[test]
fn room_1_staircase_30_spiral_bg_character_graphics_cross_three_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_dungeon_room_index(0x01);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state
        .dungeon_stair_movement_mut()
        .set_staircase_lower_level_status(0);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralBgCharacters34,
    ));
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::SpiralBgCharacters34,
            },
        )),
    );
}

fn injected_dungeon_cpu_schedule(
    submodule_nmis: u8,
    caller_nmis: u8,
) -> DungeonSubmoduleCpuSchedule {
    DungeonSubmoduleCpuSchedule {
        submodule_nmis,
        caller_nmis,
        caller_sprite_main_nmis: 0,
        caller_suffix_nmis: caller_nmis,
        caller_first_nmi_phase: None,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
        reenters_main_loop_before_nmi: false,
    }
}

fn spiral_cpu_test_state(subsubmodule: u8, schedule: DungeonSubmoduleCpuSchedule) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(subsubmodule);
    state.dungeon_submodule_cpu_schedule = Some(schedule);
    state
}

#[test]
fn spiral_room_initialization_uses_cpu_derived_submodule_slices() {
    let mut state = spiral_cpu_test_state(3, injected_dungeon_cpu_schedule(2, 0));
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralRoomInitialization,
    ));
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
}

#[test]
fn spiral_sprite_graphics_uses_cpu_derived_submodule_slices() {
    let mut state = spiral_cpu_test_state(6, injected_dungeon_cpu_schedule(4, 0));
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralSpriteGraphics,
    ));
    for _ in 0..3 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
}

#[test]
fn zero_slice_spiral_background_schedule_does_not_suspend() {
    let mut state = spiral_cpu_test_state(7, injected_dungeon_cpu_schedule(0, 0));
    assert!(!state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralBackgroundSync,
    ));
    assert!(state.dungeon_submodule_cpu_schedule.is_none());
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn spiral_sprite_schedule_does_not_depend_on_room_identity() {
    for room in [0x70, 0x71, 0x80] {
        let mut state = spiral_cpu_test_state(6, injected_dungeon_cpu_schedule(2, 0));
        state.set_dungeon_room_index(room);
        assert!(state.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpiralSpriteGraphics,
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
        assert!(matches!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(_)),
        ));
    }
}

#[test]
fn spiral_cpu_schedule_keeps_caller_nmis_separate_from_submodule_nmis() {
    let schedule = injected_dungeon_cpu_schedule(1, 4);
    let mut state = spiral_cpu_test_state(3, schedule);
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralRoomInitialization,
    ));
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
    assert_eq!(state.dungeon_submodule_cpu_schedule, Some(schedule));
}

#[test]
fn dungeon_quadrant_hold_uses_the_dma_latched_resident_oam_without_copying_it() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.resident_oam_dma = Some(vec![0x1111; state.ppu.oam.len()]);
    state.ppu.oam.fill(0x2222);

    state.stage_dungeon_supertile_quadrant_upload_obj_scanout();

    assert_eq!(state.next_display_obj_memory_generation, None);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );

    state.capture_display_snapshot();
    let display = state.display_snapshot.as_deref().unwrap();
    assert!(display.ppu.oam.iter().all(|&word| word == 0x1111));
    assert_eq!(
        display.obj_generation,
        DisplayObjGeneration::FollowModuleCadence
    );
}

#[test]
fn big_key_drop_publishes_entry_dma_then_holds_it_across_waiting_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.last_presented_oam = Some(vec![0x1111; 272]);
    state.staged_presented_oam = Some(vec![0x2222; 272]);
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        bg2_x: 1,
        bg2_y: 2,
        bg1_x: 3,
        bg1_y: 4,
    });

    assert!(state.begin_big_key_drop_graphics_work(2));
    assert_eq!(state.next_display_obj_memory_generation, None);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );

    state.stage_big_key_drop_waiting_obj_scanout();
    assert_eq!(
        state.next_display_obj_memory_generation,
        Some(DisplayObjGeneration::RetainCapturedOam {
            oam: vec![0x2222; 272],
        })
    );
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishBigKeyDropGraphics {
            sprite_slot: 2,
            dungeon: DungeonSpriteMainReturn {
                bg2_x: 1,
                bg2_y: 2,
                bg1_x: 3,
                bg1_y: 4,
            },
        })
    );
}

#[test]
fn pre_dungeon_work_resumes_at_room_and_song_bank_transfer_boundaries() {
    assert_eq!(PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES, 58);
    let stages = [
        (
            GameWorkContinuation::FinishPreDungeonEntranceLoad,
            PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreDungeonSongBankTransfer,
            PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES,
        ),
    ];

    for (continuation, nmi_slices) in stages {
        let mut work = ScheduledGameWork::schedule(continuation, nmi_slices);
        for _ in 0..nmi_slices - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
        assert!(work.is_complete());
    }
}

#[test]
fn pre_dungeon_return_leaves_the_landing_cpu_trace_to_own_the_first_boundary() {
    let mut state = ZeldaState::new();

    state.finish_pre_dungeon_caller_at_main_wait();

    assert_eq!(state.sprite_main_cpu_boundary, None);
    assert_eq!(state.sprite_main_cpu_nmi_slices, 0);
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
}

#[test]
fn pre_dungeon_publishes_entrance_before_scheduled_room_construction() {
    let mut expected = ZeldaState::new();
    expected.assets = Some(probe_entrance_asset_pack(0, 0x0061));
    expected.set_which_entrance(0);
    expected.save_progress_mut().set_progress_indicator(2);
    expected.Dungeon_LoadEntrance();
    let expected_room = expected.game_state.world.location.dungeon_room();
    let expected_x = expected.game_state.player.follower_link.x();
    let expected_y = expected.game_state.player.follower_link.y();

    let mut state = ZeldaState::new();
    state.assets = Some(probe_entrance_asset_pack(0, 0x0061));
    state.set_main_module(6);
    state.set_which_entrance(0);
    state.save_progress_mut().set_progress_indicator(2);
    state.set_dungeon_room(0x0055);
    state.follower_link_state_mut().set_x(0x07f8);
    state.follower_link_state_mut().set_y(0x06f8);
    state.set_rom_startup_timing(true);

    state.module_pre_dungeon();

    assert_eq!(state.game_state.frame.main_module, 6);
    assert_eq!(
        state.game_state.world.location.dungeon_room(),
        expected_room
    );
    assert_eq!(state.game_state.player.follower_link.x(), expected_x);
    assert_eq!(state.game_state.player.follower_link.y(), expected_y);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishPreDungeonEntranceLoad)
    );
}

#[test]
fn throne_room_work_budget_follows_the_retained_sprite_tileset() {
    assert_eq!(attract_throne_room_nmi_slices(19), 42);
    assert_eq!(attract_throne_room_nmi_slices(66), 44);
}

#[test]
fn prison_room_rom_work_resumes_after_its_room_build_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractZeldaPrison,
        ATTRACT_ZELDA_PRISON_NMI_SLICES,
    );

    for _ in 0..ATTRACT_ZELDA_PRISON_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractZeldaPrison)
    );
}

#[test]
fn maiden_warp_room_rom_work_resumes_after_its_room_build_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractMaidenWarp,
        ATTRACT_MAIDEN_WARP_NMI_SLICES,
    );

    for _ in 0..ATTRACT_MAIDEN_WARP_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractMaidenWarp)
    );
}

#[test]
fn end_of_story_rom_work_resumes_after_memory_and_palette_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractEndOfStory,
        ATTRACT_END_OF_STORY_NMI_SLICES,
    );

    for _ in 0..ATTRACT_END_OF_STORY_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractEndOfStory)
    );
}

#[test]
fn intro_poly_initialization_resumes_for_cold_start_and_attract_restart() {
    assert!(rom_intro_poly_initialization_is_active(0, 2));
    assert!(rom_intro_poly_initialization_is_active(0, 10));
    assert!(!rom_intro_poly_initialization_is_active(0, 11));
    assert!(!rom_intro_poly_initialization_is_active(20, 10));
}

#[test]
fn attract_first_story_render_wait_is_armed_by_fade_completion() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_state(4);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 7);
}

#[test]
fn world_map_fade_completion_runs_the_first_mode7_tick_immediately() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_sequence(1);
    state.attract_scene_mut().set_state(4);
    state.attract_scene_mut().set_mode7_zoom_timer(0xff);
    state.attract_scene_mut().set_scene_timer(1);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 0);
    assert_eq!(
        state.game_state.ending.attract_scene.mode7_zoom_timer(),
        0xfe
    );
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(0), 0x0174);
}

#[test]
fn world_map_rom_work_completes_after_the_five_snes9x_observed_nmi_slices() {
    let mut work = ScheduledGameWork::schedule(GameWorkContinuation::FinishAttractWorldMap, 5);

    for _ in 0..4 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMap)
    );
}

#[test]
fn player_world_map_load_completes_after_the_five_post_entry_nmi_slices() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishWorldMapLightLoad,
        WORLD_MAP_LIGHT_LOAD_NMI_SLICES,
    );

    for _ in 0..WORLD_MAP_LIGHT_LOAD_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapLightLoad)
    );
}

#[test]
fn world_map_exit_tilesets_resume_after_the_measured_nmi_slices() {
    assert_eq!(WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES, 33);
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishWorldMapExitTilesets,
        WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES,
    );

    for _ in 0..WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapExitTilesets)
    );
}

#[test]
fn module09_world_map_bodies_use_the_original_rom_cpu_schedule() {
    // Production captures these plans by executing the original ROM from its
    // $00:8034 main wait. Cold Snes9x proves $09/$20 crosses six body NMIs and
    // one caller NMI after Sprite_Main slot 8, while $09/$21 crosses only three.
    for (submodule, schedule, expected_work, expected_slices) in [
        (
            0x20,
            Module09CpuSchedule {
                submodule_nmis: 6,
                caller_nmis: 1,
                caller_sprite_main_nmis: 1,
                caller_suffix_nmis: 0,
                caller_first_nmi_phase: Some(ModuleCpuPhase::InterruptedInSpriteMain),
                sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(8)),
            },
            GameWorkContinuation::FinishWorldMapOverlayReload,
            6,
        ),
        (
            0x21,
            Module09CpuSchedule {
                submodule_nmis: 3,
                ..Module09CpuSchedule::default()
            },
            GameWorkContinuation::FinishWorldMapAmbientMap8,
            3,
        ),
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(9);
        state.set_submodule(submodule);
        state.module09_cpu_schedule = Some(schedule);
        state.game_execution_scheduler.begin_host_frame();
        state
            .game_execution_scheduler
            .mark_main_iteration_after_leading_nmi();
        state.game_execution_scheduler.begin_main_loop_iteration();

        if submodule == 0x20 {
            state.Overworld_LoadOverlays2();
        } else {
            state.Overworld_LoadAmbientOverlayFalse();
        }

        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(expected_work)
        );
        assert_eq!(
            state
                .game_execution_scheduler
                .scheduled_work_slices_remaining(),
            Some(expected_slices),
        );
        assert_eq!(state.module09_cpu_schedule, Some(schedule));
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
    }
}

#[test]
fn c_module09_overlay_return_has_no_self_healing_counter_transient() {
    // C Module09_Overworld keeps its four scroll locals live across
    // Sprite_Main. The ROM trace for run 6168 enters Sprite_ExecuteSingle with
    // X=7 and $12=1, proving slots 15..8 returned before NMI. Run 6169 resumes
    // the caller and clears $12 while $1a remains $69. Only run 6170's leading
    // NMI precedes INC $1a.
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_indoor_flag(0);
    state.set_main_module(9);
    state.set_submodule(0x21);
    state.set_frame_counter(0x69);
    state.set_bg2_h_copy2(0x1010);
    state.set_bg2_v_copy2(0x2020);
    state.set_bg1_h_copy2(0x3030);
    state.set_bg1_v_copy2(0x4040);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    state.latch_nmi_update();
    state.set_pending_nmi_subroutine(4);
    state.set_core_update_disable_flag(4);

    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishWorldMapOverlayReload, 1);
    state.game_execution_scheduler.begin_host_frame();
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishWorldMapOverlayReload,
        )),
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());

    state.begin_world_map_overlay_module09_sprite_return(SpriteMainCpuBoundary::AfterSlot(8), 1);
    let caller = Module09SpriteMainReturn {
        bg2_x: 0x1010,
        bg2_y: 0x2020,
        bg1_x: 0x3030,
        bg1_y: 0x4040,
    };
    let continuation = GameWorkContinuation::FinishSpriteMain {
        boundary: SpriteMainCpuBoundary::AfterSlot(8),
        caller: SpriteMainCpuCaller::WorldMapOverlayReload { module09: caller },
    };
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(continuation)
    );

    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 4);

    assert_eq!(
        state
            .game_execution_scheduler
            .take_after_current_trailing_nmi(),
        Some(continuation),
    );
    state.complete_post_trailing_nmi_continuation(continuation, 0, false);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 4);
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        0x1010
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        0x2020
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg1_h_copy2(),
        0x3030
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg1_v_copy2(),
        0x4040
    );
    state
        .game_execution_scheduler
        .finish_call_stack_at_main_wait_before_nmi();

    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .main_return_requires_leading_nmi());
    assert_eq!(state.game_state.frame.frame_counter, 0x69);

    state.module09_cpu_schedule = Some(Module09CpuSchedule {
        submodule_nmis: 3,
        ..Module09CpuSchedule::default()
    });
    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.frame.frame_counter, 0x69);

    state.zelda_run_game_loop_after_leading_nmi();
    assert_eq!(state.game_state.frame.frame_counter, 0x6a);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishWorldMapAmbientMap8),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(3),
    );
}

#[test]
fn enemy_drop_receipt_sound_retires_at_the_measured_graphics_return() {
    let continuation = GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x22,
            ground_apress_tail: None,
        },
    };
    let mut state = ZeldaState::new();
    state.set_sound_effect_2(0x2f);
    state
        .game_execution_scheduler
        .schedule_work(continuation, ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES);

    state.publish_or_defer_item_receipt_sound_effect_2(0x0f);

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x2f);
    assert_eq!(
        state.enemy_drop_item_graphics_deferred_sound_effect_2,
        Some(0x0f)
    );
    state.game_execution_scheduler.finish_work();
    state.retire_enemy_drop_item_graphics_sound_effect_2();

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x0f);
    assert_eq!(state.enemy_drop_item_graphics_deferred_sound_effect_2, None);
}

#[test]
fn ordinary_item_receipt_sound_publishes_without_enemy_drop_work() {
    let mut state = ZeldaState::new();
    state.set_sound_effect_2(0x2f);

    state.publish_or_defer_item_receipt_sound_effect_2(0x0f);

    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x0f);
    assert_eq!(state.enemy_drop_item_graphics_deferred_sound_effect_2, None);
}

#[test]
fn standard_item_receipt_graphics_hold_the_four_snes9x_observed_nmi_slices() {
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x14),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x06),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x0c),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(
        rom_item_receipt_graphics_nmi_slices(0x24),
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES
    );
    assert_eq!(rom_item_receipt_graphics_nmi_slices(0x23), 0);
    // The fourth slice completes gfx $14's OBJ upload and the caller prepares
    // player OAM before scanout; keep this receipt aligned with the observed
    // live/live boundary instead of an older retained-CHR assumption.
    assert_eq!(
        atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None
            },
        ),
        ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLivePlayerOamAfterMain,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        }
    );
    assert_eq!(
        atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x22,
                ground_apress_tail: None
            },
        ),
        ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        }
    );

    let continuation = GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x14,
            ground_apress_tail: None,
        },
    };
    let mut work =
        ScheduledGameWork::schedule(continuation, ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES);
    assert!(!work.suspends_translated_call_stack());
    assert_eq!(work.in_flight_display_snapshot_publication_override(), None);
    let mut waiting_publication = work;
    assert_eq!(
        waiting_publication.advance_one_nmi_slice(),
        GameWorkStep::Waiting
    );
    assert_eq!(
        waiting_publication.in_flight_display_snapshot_publication_override(),
        Some(DisplaySnapshotPublication::RetainPublished)
    );
    let suspended = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                receipt: ItemReceiptReturn {
                    ancilla_slot: 4,
                    item: 0,
                    chest_position: 0,
                },
                sprite_slot: 0,
                dungeon: DungeonSpriteMainReturn {
                    bg2_x: 1,
                    bg2_y: 2,
                    bg1_x: 3,
                    bg1_y: 4,
                },
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );
    assert!(suspended.suspends_translated_call_stack());
    assert_eq!(
        suspended.in_flight_display_snapshot_publication_override(),
        None
    );
    let chest_receipt = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x10,
                ground_apress_tail: Some(ItemReceiptReturn {
                    ancilla_slot: 4,
                    item: 0x24,
                    chest_position: 0x0182,
                }),
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );
    assert!(
        chest_receipt.suspends_translated_call_stack(),
        "the synchronous chest decompressor must retain Module 7's pre-Sprite_Main stack",
    );
    for _ in 0..ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(continuation)
    );
}

#[test]
fn dungeon_exit_spotlight_models_measured_circle_and_suffix_boundaries() {
    // 189-row calibration center (dungeon landing): $70 publishes in-slice.
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(0x7e, 36));
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(0x77, 36));
    assert!(!rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x70, 36
    ));
    // Maximal 239-row table (Link's-house entrance, vertical center 238): the
    // $70 build still crosses vblank.
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x70, 238
    ));
    assert!(!rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x69, 238
    ));
    let cpu_plan = |interrupted_pc| DungeonExitSpotlightCpuPlan {
        interrupted_pc,
        interrupted_return_address: 0,
        iterations_before_nmi: 0,
        link_position_integrated_before_first_nmi: false,
        returned_to_main_wait_before_first_nmi: false,
        main_loop_sprite_preparation_completed_before_second_nmi: false,
        active_window_words: [0; SPOTLIGHT_VISIBLE_SCANLINES],
        following_window_words: [0; SPOTLIGHT_VISIBLE_SCANLINES],
        next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
        next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
    };
    assert!(cpu_plan(0x00_f38d).interrupted_during_table_build_or_copy());
    assert!(!cpu_plan(0x00_f38d).interrupted_during_table_copy());
    assert!(cpu_plan(0x00_f3be).interrupted_during_table_build_or_copy());
    assert!(cpu_plan(0x00_f3be).interrupted_during_table_copy());
    assert!(!cpu_plan(0x00_f3c5).interrupted_during_table_build_or_copy());
    assert_eq!(
        rom_display_snapshot_publication(0x0f, 0),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(0x0f, 1),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable).completion_publication(),
        DisplaySnapshotPublication::RetainPublished
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    let entry_with_rom_following_field =
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication)
            .with_rom_following_field_after_staged_active([0x00ff; SPOTLIGHT_VISIBLE_SCANLINES]);
    let entry_after_rom_following_field =
        entry_with_rom_following_field.after_rom_following_field_was_staged();
    assert!(entry_after_rom_following_field
        .rom_following_field_receipt()
        .is_none());
    assert_eq!(
        entry_after_rom_following_field.completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryAfterTablePublication)
            .completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::opening().in_flight_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(
            SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            true,
        )
        .in_flight_publication(),
        DisplaySnapshotPublication::RetainPublished
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .in_flight_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .projection_uses_published_prefix()
    );
    assert!(
        !SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .after_game_over_build()
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        !SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .projects_following_table_tail_on_completion()
    );
    assert!(!SpotlightIteration::opening().publishes_completed_hdma_table_to_active_scanout());
    assert!(!rom_display_memory_publication_is_deferred(7, 15, 0, false));
    assert_eq!(
        SpotlightIteration::opening().completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 42),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 41),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x38, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x38, 42),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x07, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0, 0),
        SpotlightIterationPhase::WholeTable
    );
    assert_eq!(DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES, 2);
    assert_eq!(TRAILING_NMI_FORCE_BLANK_SCANLINE, 224);
    assert_eq!(
        rom_graphics_dma_plan(6, 0).oam_scanout,
        OamScanoutSource::ComposeLiveAfterNmi,
    );
    assert_eq!(
        rom_graphics_dma_plan(6, 0).link_obj_scanout,
        GraphicsDmaGeneration::LiveAfterMain,
    );
    assert_eq!(spotlight_mixed_scanout_live_tail_start(36, 0x38), 221);
    assert_eq!(spotlight_mixed_scanout_live_tail_start(238, 0x38), 224);
    assert_eq!(spotlight_mixed_scanout_live_tail_start(238, 0x31), 221);

    let closing_iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: closing_iteration,
        },
        SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
    );
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
            iteration: closing_iteration,
        })
    );
    let goal_iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication);
    let goal_continuation = GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller {
        iteration: goal_iteration,
    };
    let mut goal_work = ScheduledGameWork::schedule(
        goal_continuation,
        DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES,
    );
    assert!(goal_work.suspends_translated_call_stack());
    assert_eq!(goal_work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    assert_eq!(
        goal_work.advance_one_nmi_slice(),
        GameWorkStep::Complete(goal_continuation)
    );
    assert!(rom_dungeon_landing_goal_transition_waits_for_caller_return(
        7, 15
    ));
    assert!(!rom_dungeon_landing_goal_transition_waits_for_caller_return(16, 1));
}

#[test]
fn interrupted_spotlight_suffix_matches_c_writes_and_stages_the_rom_receipt() {
    let mut state = ZeldaState::new();
    let measured = std::array::from_fn(|row| 0x00ff_u16.wrapping_add(row as u16));
    let ram_before = state.ram.clone();

    state.spotlight_internal_after_table_during_active_rom_field(&measured);

    // zelda3/src/load_gfx.c:SpotlightInternal writes exactly these two RAM
    // mirrors after IrisSpotlight_ConfigureTable returns. Display-generation
    // provenance is hardware state and must not manufacture another RAM write.
    let changed_ram = ram_before
        .iter()
        .zip(&state.ram)
        .enumerate()
        .filter_map(|(address, (&before, &after))| (before != after).then_some((address, after)))
        .collect::<Vec<_>>();
    assert_eq!(
        changed_ram,
        vec![
            (crate::game_state::constants::INIDISP_COPY, 0x0f),
            (crate::game_state::constants::HDMAEN_COPY, 0x80),
        ]
    );

    let staged = state
        .spotlight_scanout_after_active_field
        .as_ref()
        .expect("ROM following field");
    assert!(staged.authoritative_rom_hdma_receipt);
    for (row, word) in measured.into_iter().enumerate() {
        let offset = row * 2;
        assert_eq!(
            u16::from_le_bytes(
                staged.hdma_tables[0][offset..offset + 2]
                    .try_into()
                    .unwrap()
            ),
            word
        );
    }
}

#[test]
fn module10_opening_goal_completes_before_its_caller_returns() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(16);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.follower_link_state_mut().set_facing(2);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x77);

    // In C, IrisSpotlight_ConfigureTable clears the module indices before
    // Spotlight_ConfigureTableAndControl immediately calls OpenSpotlight_Next2.
    let (caller_interrupted, reached_spotlight_goal) =
        state.spotlight_configure_table_and_control(false);

    assert!(!caller_interrupted);
    assert!(reached_spotlight_goal);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x7e
    );
    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 0x0a);
    assert_eq!(state.game_state.frame.subsubmodule, 0);
    assert!(!state.dungeon_landing_goal_transition_pending);
}

#[test]
fn dungeon_landing_goal_authors_final_circle_before_resetting_the_live_table() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(7);
    state.set_submodule(0x0f);
    state.set_saved_module_for_menu(7);
    state.follower_link_state_mut().set_x(0x80);
    state.follower_link_state_mut().set_y(0x70);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x77);

    assert!(state.iris_spotlight_configure_table());
    assert!(state.dungeon_landing_goal_transition_pending);

    let dynamic = state.ram
        [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .to_vec();
    let reserved = state.ram
        [RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .to_vec();
    assert_eq!(dynamic, reserved, "the C memcpy preserves the final circle");
    assert!(dynamic.chunks_exact(2).any(|word| word != [0x00, 0xff]));
    state.capture_display_snapshot();

    state.complete_iris_spotlight_goal_transition();

    assert!(
        state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .chunks_exact(2)
            .all(|word| word == [0x00, 0xff])
    );
    assert_eq!(
        &state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN],
        reserved,
        "the reset is a later live-table generation, not a rewrite of the final circle",
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ram
            [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .to_vec()),
        dynamic,
        "the C reset cannot rewrite the field that already captured the final circle",
    );

    state.complete_dungeon_landing_goal_active_scanout(Vec::new());
    let reset_stream =
        state.with_display_snapshot(|display| spotlight_hdma_tables_from_ram(&display.ram));
    assert!(reset_stream
        .iter()
        .all(|table| table.chunks_exact(2).all(|word| word == [0x00, 0xff])));

    state.capture_display_snapshot();
    assert!(state
        .with_display_snapshot(|display| display.ram
            [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .to_vec())
        .chunks_exact(2)
        .all(|word| word == [0x00, 0xff]));
}

#[test]
fn spotlight_reset_field_is_published_only_when_the_caller_crosses_one_nmi() {
    let mut one_crossing_rows = [Some(true); SPOTLIGHT_VISIBLE_SCANLINES];
    one_crossing_rows[..4].fill(Some(false));

    // The pinned Snes9x frame-2328 trace retains rows 0..3 from the final C
    // circle and consumes IrisSpotlight_ResetTable below it. Its translated
    // caller crosses exactly one NMI before returning.
    assert_eq!(
        spotlight_reset_prefix_scanlines(&one_crossing_rows, 1),
        Some(4)
    );

    let mut two_crossing_rows = [Some(true); SPOTLIGHT_VISIBLE_SCANLINES];
    two_crossing_rows[21..32].fill(Some(false));
    two_crossing_rows[53..64].fill(Some(false));

    // At frame 11,595 the same C reset work crosses two NMIs. Snes9x presents
    // the final circle through 11,596 and the fully reset following field at
    // 11,597; the discarded in-flight store pattern owns neither publication.
    assert_eq!(
        spotlight_reset_prefix_scanlines(&two_crossing_rows, 2),
        None
    );
}

#[test]
fn spotlight_reset_prefix_expires_with_its_cpu_advance() {
    let mut state = ZeldaState::new();
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
        submodule_nmi_slices: 1,
        subsubmodule: 1,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    };
    state.dungeon_landing_cpu_advance_pending = Some(advance);
    state.dungeon_landing_spotlight_reset_prefix_scanlines = Some(4);

    assert_eq!(state.take_dungeon_landing_cpu_advance(), Some(advance));
    assert_eq!(
        state.active_dungeon_landing_spotlight_reset_prefix_scanlines,
        Some(4)
    );
    assert_eq!(state.dungeon_landing_spotlight_reset_prefix_scanlines, None);

    // The next two-NMI timing result has no presented reset field. Consuming
    // that exact continuation must replace, not retain, the earlier prefix.
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        submodule_nmi_slices: 2,
        ..advance
    });
    state.dungeon_landing_spotlight_reset_prefix_scanlines = None;
    state.take_dungeon_landing_cpu_advance();
    assert_eq!(
        state.active_dungeon_landing_spotlight_reset_prefix_scanlines,
        None
    );
}

#[test]
fn module10_goal_return_publishes_the_rom_vblank_generation() {
    let plan = OverworldSpotlightCpuPlan {
        interrupted_pc: 0x00_f3b7,
        interrupted_return_address: 0,
        iterations_before_nmi: 0,
        // This is captured when the ROM reaches the main-loop wait after
        // IrisSpotlight_ResetTable. Later scanouts collected by the timing
        // plan must not overwrite the already-proven module-exit boundary.
        nmis_before_module_exit: Some(1),
        active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
        following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
        next_entry_earliest: None,
        next_entry_latest: None,
    };

    assert!(plan.exits_module_before_next_nmi());
    let iteration = SpotlightIteration::opening_from_rom_cpu_plan(Some(plan));
    assert_eq!(
        iteration.completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged,
    );
    assert!(iteration.completed_hdma_table_owns_active_scanout());
    assert_eq!(
        SpotlightIteration::opening_from_rom_cpu_plan(None).completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged,
    );
}

#[test]
fn interrupted_dungeon_exit_spotlight_publishes_the_rom_prefix_before_waiting() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_screen_layers(0x16);
    state.set_sub_screen_layers(0x01);
    state.follower_link_state_mut().set_y(94);
    for row in 0..240 {
        state.set_spotlight_hdma_table_dynamic_entry(row, 0x00ff);
    }

    assert!(state.begin_dungeon_exit_spotlight_entry(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f38d,
            interrupted_return_address: 0,
            iterations_before_nmi: 19,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: false,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        }),
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication,),
    ));

    assert_eq!(state.game_state.display.bg12_window_selection, 0x33);
    assert_eq!(state.game_state.display.bg34_window_selection, 0x03);
    assert_eq!(state.game_state.display.object_color_window_selection, 0x33);
    assert_eq!(state.game_state.display.main_screen_window_layers, 0x16);
    assert_eq!(state.game_state.display.sub_screen_window_layers, 0x01);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x7e
    );
    assert_eq!(state.game_state.display.spotlight_hdma.window_state(), 0);
    assert!(state.next_display_spotlight_scanout.is_some());
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
    ));
    // The C loop has authored 19 paired rows at the interrupt boundary. For
    // center 106, the upper cursor wraps through rows 0..6 while the lower
    // cursor covers rows 224..206.
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(0), 0x00ff);
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(206), 0x00ff);
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(7), 0x00ff);
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(205), 0x00ff);
}

#[test]
fn interrupted_dungeon_exit_build_retains_the_c_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x70);
    state.set_spotlight_window_state(0);
    for row in 0..240 {
        state.set_spotlight_hdma_table_dynamic_entry(row, 0x00ff);
    }
    let iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .with_main_loop_sprite_preparation_before_second_nmi();
    assert!(state.begin_dungeon_exit_spotlight_build(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f3be,
            interrupted_return_address: 0,
            iterations_before_nmi: usize::MAX,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: true,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        }),
        iteration,
    ));

    // The C builder and copy have completed, but the radius write and Module0F
    // Link/OAM suffix remain on the interrupted call stack.
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(0), 0x00ff);
    assert_eq!(
        read_le_u16(&state.ram, RESERVED_HDMA_TABLE),
        state.spotlight_hdma_table_dynamic_entry(0),
    );
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x70
    );
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            iteration: pending,
            ..
        }) if pending == iteration
    ));

    // Model the long-close path which already returned through the C main-loop
    // sprite preparation before this scheduled publication continuation.
    state.nmi_prepare_sprites_for_main_loop();
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.complete_dungeon_exit_spotlight_build(
        SpotlightTableBuildContinuation::default(),
        true,
        iteration,
    );
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x69
    );
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 9);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 7);
}

#[test]
fn interrupted_dungeon_exit_table_build_defers_the_radius_write_until_return() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x77);
    state.set_spotlight_window_state(0);
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
        .with_main_loop_sprite_preparation_before_second_nmi();

    assert!(state.begin_dungeon_exit_spotlight_build(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f38d,
            interrupted_return_address: 0,
            iterations_before_nmi: 19,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: true,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        }),
        iteration,
    ));

    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x77
    );
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration: pending,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("interrupted table build did not retain its C continuation");
    };
    assert!(!projection_completed);
    assert_eq!(pending, iteration);

    state.game_execution_scheduler.finish_work();
    state.complete_dungeon_exit_spotlight_build(table_build, projection_completed, iteration);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x70
    );
}

#[test]
fn interrupted_dungeon_exit_build_runs_the_unfinished_c_main_loop_sprite_prep() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x70);
    state.set_spotlight_window_state(0);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.complete_dungeon_exit_spotlight_build(
        SpotlightTableBuildContinuation::default(),
        true,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .with_main_loop_sprite_preparation_before_second_nmi(),
    );

    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x69
    );
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);
}

#[test]
fn rom_spotlight_following_field_uses_the_next_unowned_publication_slot() {
    let active = [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut following = active;
    following[221] = 0xe818;

    assert_eq!(
        spotlight_following_field_publication(&active, &following, false),
        SpotlightFollowingFieldPublication::WithCompletionCapture,
    );
    assert_eq!(
        spotlight_following_field_publication(&active, &following, true),
        SpotlightFollowingFieldPublication::AfterCompletionCapture,
    );
    assert_eq!(
        spotlight_following_field_publication(&active, &active, false),
        SpotlightFollowingFieldPublication::AfterCompletionCapture,
    );
}

#[test]
fn dungeon_exit_crossing_publishes_the_completed_oam_dma_receipt() {
    let mut state = ZeldaState::new();
    state.ppu.oam[0] = 0x1111;
    state.capture_display_snapshot();
    let mut following = *state.display_snapshot.take().unwrap();
    following.ppu.oam[0] = 0x2222;
    following.completed_oam_dma_after_capture = Some(vec![0x3333; state.ppu.oam.len()]);
    following.closed_oam_boundary_receipt = Some(ClosedOamBoundaryReceipt {
        publication_host_frame: following.publication_host_frame,
        active_oam: vec![0x4444; state.ppu.oam.len()],
    });
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            dungeon_exit_crosses_nmi_boundary: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    state.compose_display_oam(&following, &plan);
    assert_eq!(state.ppu.oam[0], 0x3333);

    following.oam_scanout_source = OamScanoutSource::ComposeCompletedWorkAfterNmi;
    state.oam_law_visible = Some(vec![0x5555; state.ppu.oam.len()]);
    state.display_snapshot = Some(Box::new(following));
    let presented = state.with_display_snapshot(|display| display.ppu.oam[0]);
    assert_eq!(presented, 0x3333);

    let snapshot = state.display_snapshot.as_mut().unwrap();
    snapshot.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: Vec::new(),
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: Some(vec![0x6666; state.ppu.oam.len()]),
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    let presented = state.with_display_snapshot(|display| display.ppu.oam[0]);
    assert_eq!(presented, 0x6666);

    let return_scanout = dungeon_exit_spotlight_entry_return_obj_scanout();
    assert_eq!(
        return_scanout.oam,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    assert_eq!(
        return_scanout.link_obj,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        return_scanout.link_obj_sources,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
}

#[test]
fn overworld_animated_bg_vram_generation_follows_scanout_authority() {
    assert_eq!(
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi.resolve_live_override(false),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi.resolve_live_override(true),
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        rom_graphics_dma_plan(7, 0x0f).animated_bg_scanout,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
}

#[test]
fn spotlight_projection_generation_is_a_scanout_local_table_mix() {
    let len = ZeldaState::HDMA_DYNAMIC_TABLE_LEN;
    let before_projection = [vec![0x11; len], vec![0x33; len]];
    let after_projection = [vec![0x22; len], vec![0x44; len]];
    let mut ram = vec![0; WRAM_SIZE];

    DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
        before_projection,
        after_projection,
        live_tail_start: SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START,
    }
    .compose_into(&mut ram);

    let split = SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START * 2;
    for (table_base, [before, after]) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
        .into_iter()
        .zip([[0x11, 0x22], [0x33, 0x44]])
    {
        assert!(ram[table_base..table_base + split]
            .iter()
            .all(|&byte| byte == before));
        assert!(ram[table_base + split..table_base + len]
            .iter()
            .all(|&byte| byte == after));
    }
}

#[test]
fn animated_bg_phase_change_retains_the_completed_scanout_generation() {
    let gameplay = rom_graphics_dma_plan(7, 0);
    let brightness = rom_graphics_dma_plan(7, 10);
    let spiral_stairs = rom_graphics_dma_plan(7, 0x0e);

    assert_eq!(
        gameplay.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(gameplay.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
    assert_eq!(
        gameplay.link_obj_scanout,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        gameplay.link_obj_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        gameplay.animated_bg_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    let landing = crate::game_state::FrameState {
        main_module: 7,
        submodule: 1,
        ..Default::default()
    };
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x72,
            8,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x71,
            8,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x72,
            4,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        animated_bg_scanout_across_main(gameplay, gameplay),
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(brightness, brightness),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(gameplay, brightness),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(brightness, gameplay),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        spiral_stairs.animated_bg_scanout,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    let spiral_frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        ..Default::default()
    };
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 1, 0xaa80,),
        Some(0xae80)
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 2, 0xaa80,),
        None
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, false, 1, 0xaa80,),
        None
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 1, 0xae80,),
        Some(0xa680)
    );
}

#[test]
fn dungeon_map_oam_shadow_reaches_the_following_scanout() {
    let map = rom_graphics_dma_plan(14, 3);

    assert_eq!(map.oam_operands, GraphicsDmaGeneration::LiveAfterMain);
    assert_eq!(map.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
    assert_eq!(map.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);
}

#[test]
fn room_01_spiral_state_8_marks_the_pending_hud_dma_live() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 8,
        ..Default::default()
    };
    assert!(rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x01, true,
    ));
    assert!(!rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x02, true,
    ));
    assert!(!rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x01, false,
    ));
}

#[test]
fn resumed_spiral_state_7_publishes_only_its_new_audio_ports_after_main() {
    let state_7 = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 7,
        ..Default::default()
    };
    let state_8 = crate::game_state::FrameState {
        subsubmodule: 8,
        ..state_7
    };
    assert!(resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
        1,
        state_7,
        state_8,
    ));
    assert!(!resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileCallerReturnNmi,
        1,
        state_7,
        state_8,
    ));
    assert!(!resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
        2,
        state_7,
        state_8,
    ));
    assert!(!resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
        1,
        crate::game_state::FrameState {
            subsubmodule: 6,
            ..state_7
        },
        state_8,
    ));
    assert!(!resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
        1,
        state_7,
        crate::game_state::FrameState {
            subsubmodule: 9,
            ..state_7
        },
    ));
    assert!(!resumed_dungeon_spiral_state_7_publishes_audio_after_main(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
        1,
        state_7,
        crate::game_state::FrameState {
            submodule: 0x02,
            ..state_8
        },
    ));

    let mut runtime = ZeldaState::new();
    runtime.set_ambient_sound_effect(3);
    runtime.set_sound_effect_2(36);
    runtime.publish_resumed_spiral_audio_after_main();
    assert_eq!(runtime.zelda_audio_route_state().queue.write, [0, 3, 0, 36]);
    assert_eq!(runtime.game_state.system_signals.ambient_sound_effect(), 3);
    assert_eq!(runtime.game_state.system_signals.sound_effect_2(), 0);
}

#[test]
fn nmi_copy_packets_publish_only_after_the_dma_boundary() {
    let mut state = ZeldaState::new();
    let packet_base = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF;
    state.ppu.vram[0x2000] = 0x1111;
    state.ppu.vram[0x2001] = 0x2222;
    state.ppu.vram[0x2020] = 0x3333;
    write_le_u16(&mut state.ram, packet_base, 0x2000);
    state.ram[packet_base + 2] = 0x80;
    state.ram[packet_base + 3] = 2;
    state.ram[packet_base + 4..packet_base + 6].copy_from_slice(&[0xaa, 0xaa]);
    write_le_u16(&mut state.ram, packet_base + 6, 0x2020);
    state.ram[packet_base + 8] = 0x81;
    state.ram[packet_base + 9] = 2;
    state.ram[packet_base + 10..packet_base + 12].copy_from_slice(&[0xbb, 0xbb]);
    write_le_u16(&mut state.ram, packet_base + 12, 0xffff);
    state.ram[NMI_COPY_PACKETS_FLAG] = 1;
    state.sync_native_game_state_from_ram();
    state.capture_display_snapshot();

    state.ppu.vram[0x2000] = 0xaaaa;
    state.ppu.vram[0x2001] = 0x9999;
    state.ppu.vram[0x2020] = 0xbbbb;

    let presented = state.with_display_snapshot(|display| {
        [
            display.ppu.vram[0x2000],
            display.ppu.vram[0x2001],
            display.ppu.vram[0x2020],
        ]
    });

    assert_eq!(presented, [0x1111, 0x9999, 0x3333]);
    assert_eq!(state.ppu.vram[0x2000], 0xaaaa);
    assert_eq!(state.ppu.vram[0x2020], 0xbbbb);
}

#[test]
fn animated_bg_uses_the_snapshot_owned_host_boundary_at_either_destination() {
    for destination in [0x3b00, 0x3c00] {
        let mut state = ZeldaState::new();
        state.set_animated_tile_vram_destination_address(destination as u16);
        state.ppu.vram[destination..destination + 0x200].fill(0x1111);
        state.capture_display_snapshot();

        state
            .display_snapshot
            .as_mut()
            .unwrap()
            .host_boundary_animated_bg_scanout = Some(AnimatedBgScanout {
            destination_address: destination,
            vram: vec![0x2222; 0x200],
            logical_sources: crate::chr_source::VramChrSourceTable::default(),
            preview_sources: crate::chr_source::VramChrSourceTable::default(),
        });
        state.ppu.vram[destination..destination + 0x200].fill(0x3333);

        let presented_word = state.with_display_snapshot(|display| display.ppu.vram[destination]);

        assert_eq!(presented_word, 0x2222);
        assert_eq!(state.ppu.vram[destination], 0x3333);
    }
}

#[test]
fn dungeon_entrance_publishes_the_animated_bg_written_by_its_leading_nmi() {
    let mut state = ZeldaState::new();
    let destination = 0x3b00;
    state.set_main_module(0x11);
    state.set_submodule(7);
    state.set_animated_tile_vram_destination_address(destination as u16);
    state.ppu.vram[destination..destination + 0x200].fill(0x1111);
    state.capture_display_snapshot();

    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .host_boundary_animated_bg_scanout = Some(AnimatedBgScanout {
        destination_address: destination,
        vram: vec![0x2222; 0x200],
        logical_sources: crate::chr_source::VramChrSourceTable::default(),
        preview_sources: crate::chr_source::VramChrSourceTable::default(),
    });
    state.ppu.vram[destination..destination + 0x200].fill(0x3333);

    let presented_word = state.with_display_snapshot(|display| display.ppu.vram[destination]);

    assert_eq!(presented_word, 0x3333);
    assert_eq!(state.ppu.vram[destination], 0x3333);
}

#[test]
fn animated_bg_operand_generation_is_explicit_not_receipt_owned() {
    const CAPTURED_SOURCE: usize = 0xaa80;
    const LIVE_SOURCE: usize = 0xae80;
    const DESTINATION: usize = 0x3b00;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_animated_tile_data_source_address(LIVE_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].fill(0x11);
    state.ram[LIVE_SOURCE..LIVE_SOURCE + 0x400].fill(0x22);
    state.capture_display_snapshot();

    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: Some(PreMainAnimatedTileDma {
            source_address: CAPTURED_SOURCE,
            destination_address: DESTINATION,
            data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
        }),
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });

    let mut leading_nmi_plan = rom_graphics_dma_plan(7, 0x0e);
    leading_nmi_plan.animated_bg_operands = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    state.nmi_core_animated_bg_update(leading_nmi_plan);

    assert_eq!(state.ppu.vram[DESTINATION], 0x1111);
    assert_eq!(
        state.game_state.display.animated_tile_data_source_usize(),
        LIVE_SOURCE
    );

    state.pre_main_graphics_dma.as_mut().unwrap().animated_tile = Some(PreMainAnimatedTileDma {
        source_address: CAPTURED_SOURCE,
        destination_address: DESTINATION,
        data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
    });
    state.ppu.vram[DESTINATION..DESTINATION + 0x200].fill(0);
    state.begin_effective_presented_dma();
    state.nmi_core_animated_bg_update(rom_graphics_dma_plan(7, 0x0e));

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
}

#[test]
fn measured_nmi_prepare_interruption_records_captured_dma_for_active_scanout_once() {
    const CAPTURED_SOURCE: usize = 0xaa80;
    const LIVE_SOURCE: usize = 0xae80;
    const DESTINATION: usize = 0x3000;

    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(LIVE_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].fill(0x11);
    state.ram[LIVE_SOURCE..LIVE_SOURCE + 0x400].fill(0x22);
    state.ppu.vram[DESTINATION..DESTINATION + 0x200].fill(0x3333);
    state.capture_display_snapshot();

    let entry_frame = state.game_state.frame;
    let captured_dma = PreMainAnimatedTileDma {
        source_address: CAPTURED_SOURCE,
        destination_address: DESTINATION,
        data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
    };
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: Some(captured_dma),
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });
    state.next_core_nmi_active_scanout_uses_host_animated_bg_operands = Some(true);

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .next_core_nmi_active_scanout_uses_host_animated_bg_operands
        .is_none());
    assert_eq!(
        state.game_state.display.animated_tile_data_source_usize(),
        LIVE_SOURCE
    );
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
    let receipt = state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .as_ref()
        .unwrap();
    assert!(receipt.vram_writes.is_empty());
    assert_eq!(receipt.decoded_bg_vram_writes[0], (DESTINATION, 0x1111));
    assert_eq!(receipt.decoded_bg_vram_writes.len(), 0x200);

    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .effective_presented_dma = None;

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
}

#[test]
fn gameplay_leading_nmi_does_not_restore_stale_animated_bg_over_full_tilemap() {
    let mut state = ZeldaState::new();
    let destination = 0x3b00;
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_animated_tile_vram_destination_address(destination as u16);
    state.ppu.vram[destination..destination + 0x200].fill(0x1111);
    state.capture_display_snapshot();

    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .host_boundary_animated_bg_scanout = Some(AnimatedBgScanout {
        destination_address: destination,
        vram: vec![0x2222; 0x200],
        logical_sources: crate::chr_source::VramChrSourceTable::default(),
        preview_sources: crate::chr_source::VramChrSourceTable::default(),
    });
    state.ppu.vram[destination..destination + 0x200].fill(0x3333);

    let presented_word = state.with_display_snapshot(|display| display.ppu.vram[destination]);

    assert_eq!(presented_word, 0x3333);
    assert_eq!(state.ppu.vram[destination], 0x3333);
}

#[test]
fn animated_bg_scanout_requires_a_captured_dma_source() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x2222;

    let presented_word = state.with_display_snapshot(|display| display.ppu.vram[0]);

    assert_eq!(presented_word, 0x2222);
}

#[test]
fn pre_overworld_load_models_measured_snes9x_nmi_boundaries() {
    let screen_build_workload = OverworldMapGraphicsWorkload {
        map32_definition_changes: 796,
    };
    let screen_build_timing = overworld_map_and_sprite_graphics_timing(screen_build_workload);
    let sprite_reset = GameWorkContinuation::PreOverworldPropertiesSpriteReset {
        overworld_screen: 0x00,
        animated_tiles: 0x58,
    };
    let stages = [
        (
            sprite_reset,
            PRE_OVERWORLD_PROPERTIES_TO_SPRITE_RESET_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreOverworldProperties {
                overworld_screen: 0x00,
            },
            PRE_OVERWORLD_PROPERTIES_AFTER_SPRITE_RESET_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreOverworldOverlays,
            PRE_OVERWORLD_OVERLAYS_NMI_SLICES,
        ),
    ];

    for (continuation, nmi_slices) in stages {
        let mut work = ScheduledGameWork::schedule(continuation, nmi_slices);
        for _ in 0..nmi_slices - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
    }

    let screen_build_nmi_slices = screen_build_timing.quadrant_load_nmi_slices
        + screen_build_timing.map16_to_map8_tail_nmi_slices;
    let continuation = GameWorkContinuation::FinishPreOverworldScreenBuild;
    let mut work =
        ScheduledGameWork::schedule_before_trailing_nmi(continuation, screen_build_nmi_slices);
    // Module08_02 starts before the entry frame's trailing NMI, so only the
    // subsequent 16 boundaries are consumed by future host calls.
    for _ in 1..screen_build_nmi_slices - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(continuation)
    );
}

#[test]
fn pre_overworld_sprite_reset_phase_clears_old_sprite_slots_before_the_reload_tail() {
    let mut state = ZeldaState::new();
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_sprite_type(0x73);
        sprite.set_state(9);
    }

    state.sprite_begin_reload_all_overworld();

    assert_eq!(state.sprite_slot_view(0).state(), 0);
    assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x73);
}

#[test]
fn world_map_fade_publishes_the_previous_scanout_snapshot() {
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 1, 3
    ));
    assert!(rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 1, 4
    ));
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        20, 0, 0, 4
    ));
    assert!(!rom_attract_world_map_display_is_one_frame_deferred(
        0, 0, 1, 4
    ));
}

#[test]
fn throne_room_story_resumes_one_nmi_slice_sooner_than_the_first_story() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_sequence(2);
    state.attract_scene_mut().set_state(4);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 5);
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
fn resumed_caller_audio_write_precedes_the_following_nmi_sample() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_music_control(0);
    state.set_current_music_control(0);
    state.set_last_music_control(0xf1);
    state.dungeon_landing_entry_started_after_leading_nmi = true;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        1,
    );
    state.game_execution_scheduler.begin_host_frame();

    // The final scheduled slice still owns the interrupted C stack. Its
    // caller suffix must run before the following NMI samples audio commands.
    state.prepare_audio_nmi_for_main_boundary(state.game_state.frame);
    assert!(!state.audio_nmi_processed_before_main);
    let audio_follows_host_publication =
        state.resumed_dungeon_caller_audio_follows_host_publication();
    assert!(audio_follows_host_publication);

    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        ))
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());
    state
        .game_execution_scheduler
        .mark_audio_nmi_after_host_publication();

    // Model the resumed C suffix's write. The host audio batch has already
    // been published when its following NMI becomes reachable.
    state.set_music_control(0x10);
    state.zelda_push_apu_state();
    assert!(state
        .game_execution_scheduler
        .take_audio_nmi_after_host_publication());
    state.interrupt_nmi_audio_parts();
    state.audio_nmi_processed_before_main = true;

    assert!(state.audio_nmi_processed_before_main);
    assert_eq!(state.game_state.system_signals.music_control(), 0);
    assert_eq!(
        state.game_state.system_signals.current_music_control(),
        0x10
    );
    assert_eq!(state.game_state.system_signals.last_music_control(), 0x10);
    assert_eq!(state.zelda_debug_apu_write_ports()[0], 0x10);
    assert!(!state
        .game_execution_scheduler
        .take_audio_nmi_after_host_publication());
}

#[test]
fn landing_entry_before_leading_nmi_keeps_ordinary_audio_publication() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_music_control(0x10);
    state.set_current_music_control(0);
    state.set_last_music_control(0xf1);
    state.dungeon_landing_entry_started_after_leading_nmi = false;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        1,
    );
    state.game_execution_scheduler.begin_host_frame();

    assert!(!state.resumed_dungeon_caller_audio_follows_host_publication());
    state.prepare_audio_nmi_for_main_boundary(state.game_state.frame);

    assert!(state.audio_nmi_processed_before_main);
    assert_eq!(state.game_state.system_signals.music_control(), 0);
    assert_eq!(state.game_state.system_signals.last_music_control(), 0x10);
}

#[test]
fn file_select_graphics_resumes_the_module_after_every_intervening_nmi() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_file_select_graphics();

    for _ in 0..FILE_SELECT_GRAPHICS_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::FileSelectWaiting)
        );
    }
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics)
    );
    assert!(!scheduler.is_idle());
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::ResumeFileSelectModule)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn startup_dispatcher_runs_pre_dungeon_audio_at_the_measured_boundary() {
    let mut selected_game = ZeldaState::new();
    selected_game.set_rom_startup_timing(true);
    selected_game.rom_reset_frame_delay = 0;
    selected_game.initialized = true;
    selected_game.set_animated_tile_data_source_address(1);
    selected_game
        .game_execution_scheduler
        .schedule_selected_game_load();

    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        selected_game.run_frame_internal(0, crate::RUN_MAIN);
    }
    assert_ne!(
        selected_game
            .game_state
            .system_signals
            .ambient_sound_effect(),
        5
    );
    selected_game.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        selected_game
            .game_state
            .system_signals
            .ambient_sound_effect(),
        5
    );
    assert_eq!(
        selected_game
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES
    );
    assert!(selected_game.display_snapshot.is_some());
}

#[test]
fn name_player_tilemap_finishes_after_the_intervening_nmi_slice() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(4);
    state.set_submodule(1);

    state.module_name_player_1();

    assert_eq!(state.game_state.frame.submodule, 1);
    assert!(
        state.pre_main_caller_continuation_is(PreMainCallerContinuation::NamePlayerTilemapUpload)
    );
    assert!(state.rom_load_partial_nmi_this_frame);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 2);
    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);
    let terminator = state.game_state.display.vram_upload_buffer_base()
        + 4
        + select_file::SELECT_FILE_CHECKERBOARD_TILE_COUNT * 2;
    assert_eq!(read_le_u16(&state.ram, terminator), 0xffff);
}

#[test]
fn file_select_checkerboard_finishes_through_the_pre_main_dispatcher() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(3);
    state.set_submodule(1);

    state.module_erase_file_1();

    assert!(state
        .pre_main_caller_continuation_is(PreMainCallerContinuation::FileSelectCheckerboardUpload));
    assert_eq!(state.game_state.frame.submodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert_eq!(state.game_state.frame.submodule, 2);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 0);
}

#[test]
#[should_panic(expected = "cannot schedule")]
fn pre_main_scheduler_rejects_parallel_caller_suffixes() {
    let mut state = ZeldaState::new();
    state.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    state.schedule_pre_main_caller_continuation(
        PreMainCallerContinuation::FileSelectCheckerboardUpload,
    );
}

#[test]
fn resumed_caller_does_not_relabel_future_link_dma_as_presented() {
    assert_eq!(
        link_obj_dma_generations_for_cpu_phase(true, GraphicsDmaGeneration::LiveAfterMain,),
        LinkObjDmaPhaseGenerations {
            presented: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            following_nmi: GraphicsDmaGeneration::LiveAfterMain,
        },
    );
    assert_eq!(
        link_obj_dma_generations_for_cpu_phase(false, GraphicsDmaGeneration::LiveAfterMain,),
        LinkObjDmaPhaseGenerations {
            presented: GraphicsDmaGeneration::LiveAfterMain,
            following_nmi: GraphicsDmaGeneration::LiveAfterMain,
        },
    );
}

#[test]
fn dungeon_faded_filter_second_pass_resumes_without_a_new_main_iteration() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(0);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 4]
        .copy_from_slice(&[0x11, 0x11, 0x22, 0x22]);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
        submodule_nmi_slices: 1,
        subsubmodule: 2,
        palette_countdown: 1,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(
            PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
                resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
            }
        )
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        state
            .next_display_cgram_override
            .as_deref()
            .map(|cgram| &cgram[..2]),
        Some(&[0x1111, 0x2222][..])
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
            resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
        },
    );
    state.complete_dungeon_faded_filter_second_palette_pass();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn resumed_dungeon_fade_runs_the_next_iteration_after_a_leading_nmi() {
    for (room, entry_countdown) in [(0x21, 0), (0x21, 2), (0x21, 16), (0x21, 24), (0x22, 0)] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.set_dungeon_room_index(room);
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(2);
        state.set_countdown_word(entry_countdown);
        state.set_mosaic_target_level(31);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase: ModuleCpuPhase::InterruptedInSubmodule,
            resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
            submodule_nmi_slices: 1,
            subsubmodule: 2,
            palette_countdown: entry_countdown as u8 + 1,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Module07_02_FadedFilter();

        assert_eq!(
            state
                .game_execution_scheduler
                .pre_main_caller_continuation(),
            Some(
                PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
                    resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
                }
            )
        );
        assert!(state.resume_pre_main_caller_continuation(0, None));

        assert_eq!(state.game_state.frame.subsubmodule, 2);
        assert_eq!(
            u16::from(state.game_state.display.palette_filter.countdown()),
            entry_countdown + 2
        );
        assert_eq!(
            state.game_execution_scheduler.pre_main_nmi_resume(),
            Some(PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi),
            "room={room:#04x} entry_countdown={entry_countdown}",
        );
    }
}

#[test]
fn nmi_inside_sprite_preparation_resumes_without_replaying_dungeon_main() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_dungeon_room_index(0x21);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(16);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::InterruptedInNmiPrepareSprites),
        submodule_nmi_slices: 1,
        subsubmodule: 2,
        palette_countdown: 17,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    let entry_frame_counter = state.game_state.frame.frame_counter;

    state.Module07_02_FadedFilter();
    assert!(state.resume_pre_main_caller_continuation(0, None));

    assert_eq!(state.game_state.frame.frame_counter, entry_frame_counter);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 18);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonNmiPrepareSpritesCallerReturn)
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
}

#[test]
fn completed_dungeon_fade_waits_for_leading_nmi_before_state_three() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 3);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert_eq!(
        state.game_execution_scheduler.pre_main_nmi_resume(),
        Some(PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi)
    );
    assert!(!state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi.nmi_latch_clear_phase(),
        Some(NmiPhase::AfterNmi)
    );
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeHostBoundaryShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
}

#[test]
fn faded_filter_uses_the_interior_sprite_boundary_as_the_single_nmi_owner() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 3,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 3);
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert_eq!(
        state.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterSlot(0)),
    );

    state.complete_module07_dungeon_after_submodule();
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(0),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
}

#[test]
fn sprite_main_interruption_preserves_the_c_zelda_initializer_prefix() {
    assert_eq!(
        sprite_main_cpu_interruption_boundary(
            Some(0),
            Some(1),
            Some(0),
            ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS,
        ),
        Some(SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(0)),
    );
}

#[test]
fn sprite_main_interruption_does_not_promote_an_unreturned_slot() {
    assert_eq!(
        sprite_main_cpu_interruption_boundary(Some(0), Some(1), Some(0), 0x00_e7a6),
        Some(SpriteMainCpuBoundary::AfterSlot(1)),
    );
    assert_eq!(
        sprite_main_cpu_interruption_boundary(
            Some(0),
            Some(1),
            Some(2),
            ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS,
        ),
        Some(SpriteMainCpuBoundary::AfterSlot(1)),
    );
}

#[test]
fn faded_filter_preserves_nmi_prepare_sprites_caller_phase() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(!state.dungeon_nmi_prepare_sprites_return_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonNmiPrepareSpritesCallerReturn),
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
}

#[test]
fn landing_nmi_inside_link_oam_resumes_only_the_common_suffix() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInLinkOam,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(state.dungeon_post_sprite_main_return_pending);

    state.complete_module07_dungeon_after_submodule();
    assert!(!state.dungeon_post_sprite_main_return_pending);
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
    );

    state.game_execution_scheduler.begin_host_frame();
    let continuation = match state.game_execution_scheduler.advance_work_one_nmi_slice() {
        Some(GameWorkStep::Complete(continuation)) => continuation,
        step => panic!("Link OAM return must resume after one interrupting NMI: {step:?}"),
    };
    state.capture_display_snapshot();
    let captured_palette = state.display_snapshot.as_ref().unwrap().ppu.cgram.clone();
    state.complete_post_trailing_nmi_continuation(continuation, 0, false);

    assert!(state.active_dungeon_sprite_main_return.is_none());
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|display| display.effective_presented_dma.is_none()
            && display.ppu.cgram == captured_palette));
}

#[test]
fn landing_nmi_inside_sprite_main_resumes_at_the_cpu_slot_boundary() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);
    for slot in 0..=4 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(3)),
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(
        state.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterSlot(3)),
    );

    state.complete_module07_dungeon_after_submodule();
    assert_eq!(state.sprite_slot_view(4).state(), 9);
    assert_eq!(state.sprite_slot_view(3).state(), 9);
    for slot in 0..3 {
        assert_eq!(state.sprite_slot_view(slot).state(), 8);
    }
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
}

#[test]
fn cached_sprite_copy_boundary_supersedes_the_coarse_sprite_slot_boundary() {
    let mut state = ZeldaState::new();
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: Some(CachedSpriteCpuInterruption::Loading {
            slot: 2,
            copied_fields: 12,
        }),
    };

    assert!(state.arm_dungeon_sprite_main_cpu_continuation(advance));
    assert_eq!(state.sprite_main_cpu_boundary, None);
    assert_eq!(
        state.dungeon_cached_sprite_cpu_interruption_pending,
        Some(CachedSpriteCpuInterruption::Loading {
            slot: 2,
            copied_fields: 12,
        })
    );
}

#[test]
fn interrupted_landing_fades_suspend_their_caller_return_before_state_fifteen_main() {
    for room in [0x21, 0x22, 0x41, 0x60] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(14);
        state.set_dungeon_room_index(room);
        state.set_countdown_word(30);
        state.set_mosaic_target_level(31);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase: ModuleCpuPhase::InterruptedAfterModule,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 15,
            palette_countdown: 0,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Module07_02_FadedFilter();

        assert_eq!(state.game_state.frame.subsubmodule, 15, "room={room:#04x}");
        assert_eq!(
            state.game_state.display.palette_filter.countdown(),
            0,
            "room={room:#04x}"
        );
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
            }),
            "room={room:#04x}"
        );
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
        assert_eq!(
            state.dungeon_faded_filter_palette_completion_host_frame,
            Some(state.frame_ctr_dbg)
        );
        assert_eq!(
            state.next_display_obj_scanout_generation,
            Some(ObjScanoutGenerations {
                oam: OamScanoutSource::ComposeHostBoundaryShadowDma,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            })
        );
    }
}

#[test]
fn completed_landing_fade_without_cpu_interruption_returns_atomically() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x60);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.dungeon_faded_filter_palette_completion_host_frame,
        Some(state.frame_ctr_dbg)
    );
    assert_eq!(state.dungeon_post_landing_leading_nmi_room, None);
}

#[test]
fn penultimate_landing_palette_zero_suspends_its_pre_completion_caller_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x41);
    state.set_countdown_word(1);
    state.set_darkening_or_lightening_screen_word(2);
    state.set_mosaic_target_level_word(0);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedAfterModule,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 14,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert_eq!(state.game_state.frame.subsubmodule, 14);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn,
        })
    );
    assert_eq!(
        state.dungeon_faded_filter_palette_completion_host_frame,
        Some(state.frame_ctr_dbg)
    );
}

#[test]
fn landing_fade_no_request_suspends_only_the_state_fifteen_caller_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x41);
    state.dungeon_torch_mut().set_lights_out_request(0);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedAfterModule,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
        })
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());

    let completion =
        GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
        });
    assert!(scheduled_work_completion_clears_nmi_latch_after_interrupt(
        completion
    ));
    assert_eq!(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
        }
        .completion_publication(BgScrollRegisterScanout::capture(&state.ppu)),
        GameWorkCompletionPublication {
            bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
            obj: Some(ObjScanoutGenerations {
                oam: OamScanoutSource::RetainCapturedBeforeNmi,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            }),
        }
    );
}

#[test]
fn room_41_landing_tail_uses_leading_nmi_order_and_post_interrupt_latch_release() {
    let state_15 = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 15,
        ..crate::game_state::FrameState::default()
    };
    let shutter_tail = crate::game_state::FrameState {
        submodule: 5,
        subsubmodule: 0,
        ..state_15
    };

    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        state_15, 0x41
    ));
    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        shutter_tail,
        0x41
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        state_15, 0x42
    ));
    let player_control = crate::game_state::FrameState {
        submodule: 0,
        subsubmodule: 0,
        ..state_15
    };
    assert!(post_landing_player_control_runs_after_leading_nmi(
        Some(0x41),
        player_control,
        0x41,
    ));
    assert!(!post_landing_player_control_runs_after_leading_nmi(
        Some(0x41),
        player_control,
        0x42,
    ));
    assert!(
        faded_filter_palette_completion_clears_nmi_latch_after_interrupt(Some(27248), 27248, true)
    );
    assert!(
        !faded_filter_palette_completion_clears_nmi_latch_after_interrupt(
            Some(27249),
            27249,
            false
        )
    );
}

#[test]
fn only_interrupted_dungeon_fade_selects_the_host_boundary_oam_shadow() {
    let dungeon_fade = Some(
        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
            resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
        },
    );
    let spiral_fade = Some(PreMainCallerContinuation::SpiralStairsSecondPaletteFilter);

    assert!(pre_main_caller_uses_host_boundary_shadow_oam(
        dungeon_fade,
        1
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(
        dungeon_fade,
        3
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(
        spiral_fade,
        1
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(None, 1));
    assert_eq!(
        dungeon_faded_filter_first_pass_oam_scanout(),
        OamScanoutSource::RetainCapturedBeforeNmi
    );

    for subsubmodule in [2, 14] {
        assert!(interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
            FrameState {
                main_module: 7,
                submodule: 2,
                subsubmodule,
                ..FrameState::default()
            },
            dungeon_fade,
        ));
    }
    assert!(!interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
        FrameState {
            main_module: 7,
            submodule: 2,
            subsubmodule: 13,
            ..FrameState::default()
        },
        dungeon_fade,
    ));
}

#[test]
fn spiral_stair_double_palette_pass_resumes_without_a_new_main_iteration() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(2);
    state.set_countdown_word(0);
    state
        .dungeon_stair_movement_mut()
        .set_staircase_move_counter(8);

    state.Module07_0E_02_ApplyFilterIf();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondPaletteFilter)
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        state
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter(),
        8
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
    );
    state.complete_spiral_stairs_second_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
    assert_eq!(
        state
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter(),
        7
    );
}

#[test]
fn spiral_stair_grayscale_pass_resumes_on_its_second_palette_walk() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(15);
    state.set_countdown_word(0);

    state.Dungeon_DoubleApplyAndIncrementGrayscale();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter)
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
    );
    state.complete_spiral_stairs_second_grayscale_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
}

#[test]
fn spiral_stair_second_palette_return_defers_only_animated_bg_operands() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        ..Default::default()
    };

    assert!(
        !rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
            frame, true, 1, 0xaa80,
        )
    );
    assert!(
        rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
            frame, true, 1, 0xae80,
        )
    );
}

#[test]
fn straight_interroom_grayscale_pass_uses_the_shared_second_palette_return() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);

    state.Dungeon_DoubleApplyAndIncrementGrayscale();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter)
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
    );
    state.complete_spiral_stairs_second_grayscale_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
}

#[test]
fn spiral_stair_grayscale_return_releases_core_dma_after_retaining_its_scanout() {
    let mut state = ZeldaState::new();
    state.set_core_update_disable_flag(1);

    let animated_bg_operands = state.stage_spiral_stairs_second_grayscale_nmi();

    assert!(!state.game_state.display.core_updates_are_disabled());
    assert_eq!(
        animated_bg_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
    );
}

#[test]
fn spiral_stair_grayscale_return_publishes_an_advanced_animated_bg_batch() {
    let mut state = ZeldaState::new();
    state.set_core_update_disable_flag(1);
    state.set_bg_tile_animation_countdown(1);

    let animated_bg_operands = state.stage_spiral_stairs_second_grayscale_nmi();

    assert!(!state.game_state.display.core_updates_are_disabled());
    assert_eq!(animated_bg_operands, GraphicsDmaGeneration::LiveAfterMain);
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
    );
}

#[test]
fn file_select_main_publishes_display_memory_at_the_following_nmi() {
    assert!(rom_display_memory_publication_is_deferred(1, 5, 0, true));
    assert!(!rom_display_memory_publication_is_deferred(1, 4, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(2, 5, 0, false));

    let mut state = ZeldaState::new();
    state.set_main_module(1);
    state.set_submodule(5);
    state.ppu.vram[0] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.ppu.cgram[0] = 0x3333;
    state.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.cgram[0] = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.oam[0],
            display.ppu.cgram[0],
        )
    });

    assert_eq!(captured, (0x1111, 0x2222, 0x3333));
    assert_eq!(state.ppu.vram[0], 0xaaaa);
    assert_eq!(state.ppu.oam[0], 0xbbbb);
    assert_eq!(state.ppu.cgram[0], 0xcccc);
}

#[test]
fn dungeon_landing_wipe_uses_typed_snapshot_generation_without_menu_retention() {
    // Pre-dungeon staging now advances through its typed snapshot generation;
    // it does not use the menu-stripe memory-retention rule.
    assert!(!rom_display_memory_publication_is_deferred(7, 15, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(7, 14, 0, false));
    assert_eq!(
        rom_display_snapshot_publication(7, 15),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(16, 1),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(16, 0),
        DisplaySnapshotPublication::PublishCaptured
    );

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(15);
    state.ppu.vram[0] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x2222;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x2222
    );

    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x3333;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x3333
    );

    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x4444;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x4444
    );
}

#[test]
fn dialogue_character_tiles_publish_at_the_following_nmi() {
    assert!(rom_display_memory_publication_is_deferred(14, 2, 3, false));
    assert!(rom_display_memory_publication_is_deferred(4, 3, 0, true));
    assert!(!rom_display_memory_publication_is_deferred(4, 3, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(14, 1, 0, false));
}

#[test]
fn full_tilemap_upload_publishes_vram_at_the_following_nmi() {
    assert!(rom_full_tilemap_scanout_retains_uploaded_region(true, 0));
    assert!(!rom_full_tilemap_scanout_retains_uploaded_region(true, 1));
    assert!(!rom_full_tilemap_scanout_retains_uploaded_region(false, 0));

    let mut state = ZeldaState::new();
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(14);
    let (tilemap_start, tilemap_words) =
        full_tilemap_nmi_vram_region(14).expect("valid NMI tilemap destination");
    let outside_tilemap = tilemap_start + tilemap_words;
    state.ppu.vram[tilemap_start] = 0x1111;
    state.ppu.vram[outside_tilemap] = 0xaaaa;
    state.capture_display_snapshot();
    state.ppu.vram[tilemap_start] = 0x2222;
    state.ppu.vram[outside_tilemap] = 0xbbbb;
    assert_eq!(
        state.with_display_snapshot(|display| [
            display.ppu.vram[tilemap_start],
            display.ppu.vram[outside_tilemap],
        ]),
        [0x1111, 0xbbbb],
        "only the pending tilemap DMA destination retains its pre-NMI words"
    );

    state.nmi_forced_blank_scanlines_pending = 1;
    state.ppu.vram[tilemap_start] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.vram[tilemap_start] = 0x4444;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[tilemap_start]),
        0x4444
    );
}

#[test]
fn explicit_force_blank_event_owns_the_active_display_suffix() {
    // C WorldMap_FadeOut calls EnableForceBlank from the main thread. The ROM
    // reaches that routine at V=49 on the standard route, so the direct $2100
    // write owns output row 48 onward even though the previously published PPU
    // generation was not blank.
    assert!(live_forced_blank_for_scanout(false, None, Some(48), false));
    assert!(!live_forced_blank_for_scanout(false, None, Some(48), true));
    assert_eq!(
        resolve_active_display_blanking_scanout(false, Some(30), true),
        ActiveDisplayBlankingScanout {
            suffix_start_scanline: Some(30),
            retain_prior_surface: true,
        }
    );
    assert_eq!(
        resolve_active_display_blanking_scanout(true, None, false),
        ActiveDisplayBlankingScanout {
            suffix_start_scanline: None,
            retain_prior_surface: true,
        }
    );
}

#[test]
fn c_enable_force_blank_requests_the_following_field_from_row_zero() {
    let mut state = ZeldaState::new();

    // zelda3/src/load_gfx.c:EnableForceBlank assigns INIDISP_copy=$80 for
    // WritePpuRegisters at the immediately following NMI. This is distinct
    // from an arbitrary INIDISP_copy assignment made after a field began.
    state.enable_force_blank();

    assert_eq!(state.active_display_force_blank_event, Some(0));
    assert!(live_forced_blank_for_scanout(
        false,
        None,
        state.active_display_force_blank_event,
        false,
    ));
}

#[test]
fn deleted_file_return_blanks_after_the_first_active_scanline() {
    let mut state = ZeldaState::new();
    state.set_main_module(3);
    state.set_submodule(4);
    state.set_subsubmodule(0);
    state.clear_select_file_cursor();
    state.follower_link_state_mut().set_filtered_joypad_h(0x10);

    // zelda3/src/select_file.c:SelectFile_Func16 clears both SRAM copies and
    // calls ZeldaWriteSram before ReturnToFileSelect. Snes9x reaches that
    // return at V=58 and the following EnableForceBlank at V=1.
    state.select_file_func16();
    assert_eq!(
        state.pending_file_select_force_blank_output_scanline,
        Some(1)
    );

    state.file_select_erase_triforce();

    assert_eq!(state.pending_file_select_force_blank_output_scanline, None);
    assert_eq!(state.active_display_force_blank_event, Some(1));
}

#[test]
fn world_map_fade_out_uses_the_preceding_sprite_main_workload() {
    let mut state = ZeldaState::new();
    state.ppu.forced_blank = false;
    state.set_screen_brightness(1);
    state.set_overworld_map_state(0);
    let mut workload = SpriteMainTimingWorkload::default();
    workload.record_active_sprite(0x6c, 0);
    workload.record_active_sprite(0x3f, 0);
    workload.record_active_sprite(0x3f, 0);
    workload.record_garnish_table(true, 0);
    state.last_sprite_main_timing_workload = Some(workload);

    state.WorldMap_FadeOut();

    assert_eq!(state.game_state.display.screen_brightness, 0x80);
    assert_eq!(state.overworld_map_state(), 1);
    assert!(!state.ppu.forced_blank);
    assert_eq!(state.active_display_force_blank_event, Some(48));
    assert!(live_forced_blank_for_scanout(
        state.ppu.forced_blank,
        None,
        state.active_display_force_blank_event,
        false,
    ));
    assert_eq!(state.ppu.forced_blank_from_scanline, None);
}

#[test]
fn overworld_sprite_reload_timing_tracks_the_measured_rom_workload() {
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 2,
                in_bounds_proximity_checks: 18,
            },
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 3,
            post_return_hold_nmi_slices: 1,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::AfterNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(
                NmiPhase::BeforeNmi,
            ),
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 4,
                in_bounds_proximity_checks: 90,
            },
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 4,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::AfterNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::AfterNmi,),
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 8,
                in_bounds_proximity_checks: 66,
            },
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::AfterNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
        }
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            OverworldSpriteReloadWorkload {
                sprite_records: 2,
                in_bounds_proximity_checks: 18,
            },
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        ),
        OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
            return_phase: NmiPhase::BeforeNmi,
            epilogue_phase: NmiPhase::BeforeNmi,
            resume_boundary: OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
        }
    );
}

#[test]
fn overworld_graphics_timing_uses_measured_work_receipts() {
    assert_eq!(
        overworld_aux_graphics_timing(OverworldAuxGraphicsWorkload {
            background_packs_to_decompress: 0,
        }),
        OverworldAuxGraphicsTiming {
            load_nmi_slices: 11,
        }
    );
    assert_eq!(
        overworld_aux_graphics_timing(OverworldAuxGraphicsWorkload {
            background_packs_to_decompress: 2,
        }),
        OverworldAuxGraphicsTiming {
            load_nmi_slices: 15,
        }
    );

    let light_map_timing = overworld_map_and_sprite_graphics_timing(OverworldMapGraphicsWorkload {
        map32_definition_changes: 670,
    });
    assert_eq!(
        light_map_timing,
        OverworldMapAndSpriteGraphicsTiming {
            quadrant_load_nmi_slices: 13,
            map16_to_map8_tail_nmi_slices: 3,
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        }
    );
    assert_eq!(
        overworld_map_and_sprite_graphics_timing(OverworldMapGraphicsWorkload {
            map32_definition_changes: 796,
        }),
        OverworldMapAndSpriteGraphicsTiming {
            quadrant_load_nmi_slices: 14,
            map16_to_map8_tail_nmi_slices: 3,
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        }
    );

    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishOverworldMapQuadrants {
            scroll_map_and_sprite_gfx_tail_nmi_slices: light_map_timing
                .scroll_map_and_sprite_gfx_tail_nmi_slices,
        },
        light_map_timing.quadrant_load_nmi_slices,
    );
    for _ in 1..light_map_timing.quadrant_load_nmi_slices {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishOverworldMapQuadrants {
            scroll_map_and_sprite_gfx_tail_nmi_slices: 4,
        })
    );
}

#[test]
fn bg_scroll_scanout_replays_the_nmi_register_write_order() {
    let mut ppu = snes::ppu::PpuState::default();
    ppu.scroll_prev = 0x91;
    ppu.scroll_prev2 = 0x35;
    let register_bytes = [
        [0x91, 0x24, 0x00, 0x41],
        [0x18, 0x02, 0x00, 0x02],
        [0x00, 0x00, 0x00, 0x00],
    ];
    let predicted = BgScrollRegisterScanout::after_nmi_writes(&ppu, register_bytes);

    for (layer, [h_low, h_high, v_low, v_high]) in register_bytes.into_iter().enumerate() {
        let h_register = 0x0d + (layer as u8) * 2;
        let v_register = h_register + 1;
        ppu.write(h_register, h_low);
        ppu.write(h_register, h_high);
        ppu.write(v_register, v_low);
        ppu.write(v_register, v_high);
    }

    assert_eq!(predicted, BgScrollRegisterScanout::capture(&ppu));
    assert_eq!(predicted.offsets[0], [0x2491, 0x4100]);
    assert_eq!(predicted.offsets[1], [0x0218, 0x0200]);
}

#[test]
fn dialogue_scroll_freezes_the_published_hardware_generation() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x7c00] = 0x1111;
    state.ram[0x10000] = 0x22;
    state.ram[0x10001] = 0x22;
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 2,
        ..Bg3VwfGlyphRun::default()
    }];
    state.published_bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 1,
        ..Bg3VwfGlyphRun::default()
    }];
    state.published_dialogue_msg_read_pos = 0x34;
    state.published_dialogue_message_id = 0x56;

    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );

    let frozen = state
        .dialogue_scroll_frozen_scanout
        .as_ref()
        .expect("published dialogue scanout");
    assert_eq!(frozen.vram[0], 0x1111);
    assert_eq!(frozen.glyph_runs[0].glyph_code, 1);
    assert_eq!(frozen.dialogue_msg_read_pos, 0x34);
    assert_eq!(frozen.dialogue_message_id, 0x56);
}

#[test]
fn dialogue_scroll_completion_timing_follows_measured_vblank_headroom() {
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(255_000),
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(262_662),
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(283_400),
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );

    let mut state = ZeldaState::new();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    assert_eq!(
        state.finish_dialogue_scroll_remaining_pixels(),
        DialogueScrollCompletionTiming::BeforeNextVblank
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    state.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );
}

#[test]
fn dialogue_completion_before_vblank_uses_text_dma_sampled_at_publication() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.ppu.vram[0x7c00] = 0x1111;
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    state.ppu.vram[0x7c00] = 0x2222;
    state.finish_dialogue_scroll_remaining_pixels();

    state.capture_display_snapshot();

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x1111
    );

    state.stage_early_dialogue_scroll_completion(DialogueTextScanout {
        vram: vec![0x3333; 0x3f0],
        ..DialogueTextScanout::default()
    });
    state.capture_display_snapshot();

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x3333
    );

    // The next boundary samples the actual NMI text-DMA source instead of
    // projecting the earlier staged CPU buffer for a second scanout.
    write_le_u16(&mut state.ram, 0x10000, 0x4444);
    state.capture_display_snapshot();
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x4444
    );
}

#[test]
fn dialogue_scroll_machine_has_closed_hardware_boundary_sequences() {
    let mut after_return = ZeldaState::new();
    after_return.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    after_return.finish_dialogue_scroll_remaining_pixels();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly
    );
    after_return.finish_dialogue_scroll_return();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );
    after_return.stage_dialogue_scroll_completion_after_return(DialogueTextScanout::default());
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    let mut adjacent_scroll = after_return.clone();
    adjacent_scroll.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        adjacent_scroll.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );

    let mut before_vblank = ZeldaState::new();
    before_vblank.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    before_vblank.finish_dialogue_scroll_remaining_pixels();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    before_vblank.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );
}

#[test]
fn dungeon_falling_entry_retains_the_pre_transition_obj_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.ppu.oam[204] = 0x6c8a;
    state.ppu.vram[0x4000] = 0x1234;
    state.capture_display_snapshot();

    // The ROM module switch reaches WRAM before the native frame projection is
    // synchronized by NMI. Exercise that real publication-boundary ownership.
    state.ram[crate::game_state::constants::MAIN_MODULE] = 0x11;
    state.ram[crate::game_state::constants::SUBMODULE] = 0;
    assert_eq!(state.game_state.frame.main_module, 9);
    state.ppu.oam[204] = 0xf08a;
    state.ppu.vram[0x4000] = 0x5678;
    state.capture_display_snapshot();

    assert!(rom_dungeon_falling_entry_retains_published_obj_generation(
        9, 0, 0x11, 0,
    ));
    assert!(!rom_dungeon_falling_entry_retains_published_obj_generation(
        9, 1, 0x11, 0,
    ));
    assert_eq!(
        state.with_display_snapshot(|display| (display.ppu.oam[204], display.ppu.vram[0x4000])),
        (0x6c8a, 0x1234),
    );
    assert_eq!(state.ppu.oam[204], 0xf08a);
    assert_eq!(state.ppu.vram[0x4000], 0x5678);
}

#[test]
fn completed_overworld_reload_uses_its_measured_return_phase() {
    let cpu_slice_entry = BgScrollRegisterScanout {
        offsets: [[0x1111, 0x2222]; 4],
    };
    assert_eq!(
        GameWorkContinuation::FinishOverworldAuxGraphics
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
    );
    // The completion boundary still owns the scroll generation measured at
    // the CPU return. Entry geometry belongs to the following resume boundary.
    for post_return_hold_nmi_slices in [0, 1] {
        for (return_phase, generation) in [
            (
                NmiPhase::BeforeNmi,
                Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
            ),
            (
                NmiPhase::AfterNmi,
                Some(DisplayBgScrollGeneration::RetainCpuSliceEntry(
                    cpu_slice_entry,
                )),
            ),
        ] {
            let publication = GameWorkContinuation::FinishOverworldSpriteReloadTail {
                post_return_hold_nmi_slices,
                return_phase,
                epilogue_phase: NmiPhase::BeforeNmi,
                resume_scanout: OverworldSpriteReloadResumeScanout::CpuSliceEntry {
                    scroll: cpu_slice_entry,
                    bg1_generation: OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn,
                },
            }
            .completion_publication(cpu_slice_entry);
            assert_eq!(publication.bg_scroll, generation);
            assert_eq!(
                publication.obj,
                Some(ObjScanoutGenerations::coherent(
                    GraphicsDmaGeneration::HostBoundaryBeforeMain,
                )),
            );
        }
    }
    assert_eq!(
        GameWorkContinuation::HoldOverworldSpriteReloadReturn
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
    );
    assert_eq!(
        GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail
            .completion_publication(cpu_slice_entry)
            .bg_scroll,
        None,
    );
}

#[test]
fn suspended_spiral_palette_filter_holds_core_nmi_updates() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);

    assert!(state
        .suspend_spiral_staircase_palette_filter(SpiralStaircasePaletteTail::PrepareNextQuadrant,));
    assert!(state.game_state.display.core_updates_are_disabled());
}

#[test]
fn overworld_reload_timing_keeps_resume_geometry_separate_from_return_phase() {
    let light = OverworldSpriteReloadWorkload {
        sprite_records: 2,
        in_bounds_proximity_checks: 18,
    };
    let heavy = OverworldSpriteReloadWorkload {
        sprite_records: 4,
        in_bounds_proximity_checks: 90,
    };

    assert_eq!(
        overworld_sprite_reload_timing(
            heavy,
            OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::CpuSliceEntryNmiRegisters,
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            heavy,
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::AfterNmi),
    );
    assert_eq!(
        overworld_sprite_reload_timing(
            light,
            OverworldSpriteReloadEntryPhase::OrdinaryModuleIteration,
        )
        .resume_boundary,
        OverworldSpriteReloadResumeBoundary::ByReturnPhase(NmiPhase::BeforeNmi),
    );
}

#[test]
fn overworld_reload_scanout_keeps_prepublished_rain_out_of_its_bg1_generation() {
    let entry = BgScrollRegisterScanout {
        offsets: [[0x1111, 0x2222], [0x3333, 0x4444], [0, 0], [0, 0]],
    };
    let returned = BgScrollRegisterScanout {
        offsets: [[0xaaaa, 0xbbbb], [0xcccc, 0xdddd], [0, 0], [0, 0]],
    };

    for (bg1_generation, expected_bg1) in [
        (
            OverworldSpriteReloadBg1Generation::RetainBeforePrepublishedRain,
            entry.offsets[0],
        ),
        (
            OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn,
            returned.offsets[0],
        ),
    ] {
        let completed = OverworldSpriteReloadResumeScanout::CpuSliceEntry {
            scroll: entry,
            bg1_generation,
        }
        .complete_transition_return(returned);
        let OverworldSpriteReloadResumeScanout::CpuSliceEntry { scroll, .. } = completed else {
            panic!("CPU-slice scanout changed variant");
        };
        assert_eq!(scroll.offsets[0], expected_bg1);
        assert_eq!(scroll.offsets[1], returned.offsets[1]);
    }
}

#[test]
fn c_overworld_handle_rain_color_math_branches_are_source_exact() {
    // C `src/overworld.c::OverworldOverlay_HandleRain` has exactly these
    // color-math branches: 3/88 -> $32, 5/44/90 -> $72, 36 -> SFX $36 and
    // $32, with every other frame leaving both values alone.
    for (frame_counter, expected_color_math, expected_sfx) in [
        (3, 0x32, 0x00),
        (88, 0x32, 0x00),
        (5, 0x72, 0x00),
        (44, 0x72, 0x00),
        (90, 0x72, 0x00),
        (36, 0x32, 0x36),
        (37, 0x55, 0x00),
    ] {
        let mut state = ZeldaState::new();
        state.game_state.frame.set_frame_counter(frame_counter);
        state.set_color_math_control(0x55);
        state.set_sound_effect_1(0);

        state.OverworldOverlay_HandleRain();

        assert_eq!(
            state.game_state.display.palette_filter.color_math_control(),
            expected_color_math,
            "frame counter {frame_counter}",
        );
        assert_eq!(
            state.game_state.system_signals.sound_effect_1(),
            expected_sfx,
            "frame counter {frame_counter}",
        );
    }
}

#[test]
fn graphics_dma_plan_separates_operands_from_visible_scanout() {
    assert!(!rom_display_oam_publication_is_deferred(
        7, 0, 0, false, false
    ));
    // The overworld transition pipeline (submodules 1..=8) holds the main loop
    // across its load slices, so the OAM scanout keeps the retained boundary
    // cadence (measured at route frames 6913/6931/6944/6948).
    assert_eq!(
        rom_graphics_dma_plan(9, 1),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(9, 5),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(0x11, 7),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::LiveAfterNmi,
        },
    );
    for submodule in [8, 0x10] {
        let plan = rom_graphics_dma_plan(7, submodule);
        assert_eq!(plan.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
        assert_eq!(plan.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);
        assert_eq!(
            plan.link_obj_operands,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        );
    }
    let shutter_plan = rom_graphics_dma_plan(7, 5);
    assert_eq!(
        shutter_plan.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        shutter_plan.oam_scanout,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert_eq!(
        shutter_plan.link_obj_operands,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        rom_graphics_dma_plan(9, 0x0a),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(14, 7),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::ComposeLiveAfterNmi,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    let mut subtile_landing = crate::game_state::FrameState::default();
    subtile_landing.main_module = 7;
    subtile_landing.submodule = 1;
    subtile_landing.subsubmodule = 4;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainResidentPpuOam,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    subtile_landing.subsubmodule = 5;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 4,
            ..subtile_landing
        }),
    );
    subtile_landing.subsubmodule = 6;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 5,
            ..subtile_landing
        }),
    );
    subtile_landing.subsubmodule = 7;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 6,
            ..subtile_landing
        }),
    );
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, true, false
    ));
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, false, true
    ));
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, false, false
    ));
    assert!(rom_display_oam_publication_is_deferred(
        14, 7, 0, false, false
    ));
    assert!(!rom_display_oam_publication_is_deferred(
        4, 2, 0, false, false
    ));
    assert!(!rom_display_memory_publication_is_deferred(7, 0, 0, false));
    assert!(rom_display_memory_publication_is_deferred(14, 2, 3, false));
    assert!(!rom_display_memory_publication_is_deferred(14, 2, 4, false));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 1, false
    ));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, true
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, false
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 1, 0x0f, 1, true
    ));
    assert_eq!(
        GraphicsDmaGeneration::HostBoundaryBeforeMain.resolve_live_override(false),
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert_eq!(
        GraphicsDmaGeneration::HostBoundaryBeforeMain.resolve_live_override(true),
        GraphicsDmaGeneration::LiveAfterMain,
    );

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x4000] = 0x4444;
    state.ppu.oam[0] = 0x2222;
    state.ppu.cgram[0] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x4000] = 0xdddd;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.cgram[0] = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.vram[0x4000],
            display.ppu.oam[0],
            display.ppu.cgram[0],
        )
    });

    assert_eq!(captured, (0xaaaa, 0xdddd, 0xbbbb, 0xcccc));
}

#[test]
fn retained_display_memory_still_publishes_live_dungeon_obj_chr() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x4000] = 0x2222;
    state.ppu.vram[0x4050] = 0x3333;
    state.ppu.vram[0x4100] = 0x4444;
    state.ppu.vram[0x4150] = 0x5555;
    state.capture_display_snapshot();
    state.display_snapshot.as_mut().unwrap().vram_generation =
        DisplayVramGeneration::RetainCapturedBeforeNmi;

    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x4000] = 0xbbbb;
    state.ppu.vram[0x4050] = 0xcccc;
    state.ppu.vram[0x4100] = 0xdddd;
    state.ppu.vram[0x4150] = 0xeeee;

    assert_eq!(
        state.with_display_snapshot(|display| {
            (
                display.ppu.vram[0],
                display.ppu.vram[0x4000],
                display.ppu.vram[0x4050],
                display.ppu.vram[0x4100],
                display.ppu.vram[0x4150],
            )
        }),
        (0x1111, 0xbbbb, 0x3333, 0xdddd, 0x5555),
    );
}

#[test]
fn retained_display_memory_resolves_link_obj_from_its_snapshot_dma_words() {
    let mut link_graphics = vec![0; 0x7000];
    link_graphics[0x4d80..0x4d82].copy_from_slice(&[0x22, 0x23]);
    link_graphics[0x4dc0..0x4dc2].copy_from_slice(&[0x33, 0x34]);
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 58];
    put_test_asset(&mut data, &mut ranges, 57, link_graphics);

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state.set_main_module(7);
    state.set_submodule(0);
    write_le_u16(&mut state.ram, DMA_SOURCE_ADDR_4, 0xcd80);
    state.ppu.vram[0x4020] = 0x1111;
    state.capture_display_snapshot();
    state.display_snapshot.as_mut().unwrap().vram_generation =
        DisplayVramGeneration::RetainCapturedBeforeNmi;

    // The coarse following slice can skip NMI_DoUpdates after latching long
    // work. Its PPU is stale and its CPU source has already advanced again;
    // the retained scanout owns the source word captured with the snapshot.
    write_le_u16(&mut state.ram, DMA_SOURCE_ADDR_4, 0xcdc0);
    state.ppu.vram[0x4020] = 0x1111;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None,
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );

    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x4020]),
        0x2322,
    );
}

#[test]
fn display_snapshot_keeps_link_chr_identity_with_its_vram_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(6);
    let link_slot = 0x4000 / 16;
    let background_slot = 0x2000 / 16;
    let record = |table: &mut crate::chr_source::VramChrSourceTable, slot, hash| {
        table.record_tile_content_hash(slot, crate::chr_source::CHR_KIND_LINK_CONTENT, hash);
    };

    state.ppu.vram[0x4000] = 0x1111;
    record(&mut state.vram_chr_source, link_slot, 0x1111_1111);
    record(&mut state.vram_chr_preview_source, link_slot, 0x2222_2222);
    state.capture_display_snapshot();

    state.ppu.vram[0x4000] = 0xaaaa;
    record(&mut state.vram_chr_source, link_slot, 0xaaaa_aaaa);
    record(&mut state.vram_chr_preview_source, link_slot, 0xbbbb_bbbb);
    record(&mut state.vram_chr_source, background_slot, 0xcccc_cccc);

    let visible = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x4000],
            display.vram_chr_source.get(link_slot),
            display.vram_chr_preview_source.get(link_slot),
            display.vram_chr_source.get(background_slot),
        )
    });
    assert_eq!(
        (
            visible.0,
            visible.1.pack,
            visible.1.tile_off,
            visible.2.pack,
            visible.2.tile_off,
            visible.3.pack,
            visible.3.tile_off,
        ),
        (0x1111, 0x1111, 0x1111, 0x2222, 0x2222, 0xcccc, 0xcccc),
    );
    assert_eq!(state.ppu.vram[0x4000], 0xaaaa);
    assert_eq!(state.vram_chr_source.get(link_slot).pack, 0xaaaa);
}

#[test]
fn dungeon_exit_nmi_publishes_coherent_oam_link_tiles_and_scroll() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.ppu.bg_layer[1].v_scroll = 0x3333;
    state.capture_display_snapshot();

    state.set_submodule(1);
    state.ppu.vram[0x4000] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.bg_layer[1].v_scroll = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x4000],
            display.ppu.oam[0],
            display.ppu.bg_layer[1].v_scroll,
        )
    });

    assert_eq!(captured, (0xaaaa, 0xbbbb, 0xcccc));
}

#[test]
fn staged_spotlight_scanout_publishes_one_coherent_hardware_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_bg12_window_selection(0x33);
    state.set_bg34_window_selection(0x03);
    state.set_object_color_window_selection(0x33);
    state.set_main_screen_window_layers(0x16);
    state.set_sub_screen_window_layers(0x00);
    state.set_hdma_enable_mask(0xc0);
    state.dma.channel[6].b_adr = 0x26;
    state.dma.channel[7].b_adr = 0x26;
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xff00);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE, 0xfe01);
    state.stage_spotlight_scanout_for_next_display();

    // The ordinary snapshot is still the pre-NMI hardware generation. The
    // staged iris domain must replace all of its coupled controls together.
    state.set_bg12_window_selection(0);
    state.set_bg34_window_selection(0);
    state.set_object_color_window_selection(0);
    state.set_main_screen_window_layers(0);
    state.set_sub_screen_window_layers(0);
    state.set_hdma_enable_mask(0);
    state.dma.channel[6].b_adr = 0x20;
    state.dma.channel[7].b_adr = 0x21;
    state.set_spotlight_hdma_table_dynamic_entry(0, 0x00ff);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE, 0x01fe);
    state.capture_display_snapshot();

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.windowsel,
            display.ppu.screen_windowed,
            display.ram[crate::game_state::constants::HDMAEN_COPY],
            [display.dma.channel[6].b_adr, display.dma.channel[7].b_adr],
            read_le_u16(&display.ram, HDMA_TABLE_DYNAMIC),
            read_le_u16(&display.ram, RESERVED_HDMA_TABLE),
        )
    });

    assert_eq!(
        captured,
        (
            0x0033_0333,
            [0x16, 0x00],
            0xc0,
            [0x26, 0x26],
            0xff00,
            0xfe01,
        )
    );
}

#[test]
fn completed_spotlight_table_projection_overlays_the_staged_scanout_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_bg12_window_selection(0x33);
    state.set_bg34_window_selection(0x03);
    state.set_object_color_window_selection(0x33);
    state.set_main_screen_window_layers(0x16);
    state.set_sub_screen_window_layers(0x00);
    state.set_hdma_enable_mask(0xc0);
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xff00);
    state.stage_spotlight_scanout_for_next_display();
    state.capture_display_snapshot();

    state.set_spotlight_hdma_table_dynamic_entry(0, 0xfe01);
    state
        .display_snapshot
        .as_mut()
        .expect("captured display")
        .hdma_table_generation = DisplayHdmaTableGeneration::SpotlightPublishedAheadOfSnapshot {
        active_table: {
            let mut table = vec![0; ZeldaState::HDMA_DYNAMIC_TABLE_LEN];
            table[..2].copy_from_slice(&0xfe01_u16.to_le_bytes());
            table
        },
    };

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.windowsel,
            display.ppu.screen_windowed,
            display.ram[crate::game_state::constants::HDMAEN_COPY],
            read_le_u16(&display.ram, HDMA_TABLE_DYNAMIC),
        )
    });

    assert_eq!(captured, (0x0033_0333, [0x16, 0x00], 0xc0, 0xfe01));
}

#[test]
fn spotlight_return_keeps_obj_dma_on_the_pre_return_boundary() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.capture_display_snapshot();

    state.ppu.vram[0x4000] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;

    let captured =
        state.with_display_snapshot(|display| (display.ppu.vram[0x4000], display.ppu.oam[0]));

    assert_eq!(captured, (0x1111, 0x2222));
}

#[test]
fn spotlight_hdma_can_publish_ahead_of_retained_obj_domains() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xea0e);
    state.ppu.oam[0] = 0x2222;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);

    // Mirror the ordinary staged boundary immediately before the next circle
    // iteration. The whole-display generation remains the old one.
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    state.set_spotlight_hdma_table_dynamic_entry(0, 0xe612);
    state.ppu.oam[0] = 0xbbbb;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: SpotlightIteration::closing(
                SpotlightIterationPhase::WholeTableAfterTablePublication,
            ),
        },
        SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
    );
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::AdvanceStaged));

    let display = state.display_snapshot.as_ref().unwrap();
    let mut composed_ram = display.ram.clone();
    display
        .hdma_table_generation
        .compose_into(&mut composed_ram);

    assert_eq!(read_le_u16(&composed_ram, HDMA_TABLE_DYNAMIC), 0xe612);
    assert_eq!(display.ppu.oam[0], 0x2222);
}

#[test]
fn completed_short_spotlight_build_projects_its_authored_table_tail() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_spotlight_window_radius(0x70);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);
    state.set_spotlight_hdma_table_dynamic_entry(221, 0xae4a);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE + 221 * 2, 0xae4a);

    state.project_following_spotlight_tail_to_active_scanout(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
        false,
    );

    let display = state.display_snapshot.as_ref().unwrap();
    let mut composed_ram = display.ram.clone();
    display
        .hdma_table_generation
        .compose_into(&mut composed_ram);
    assert_eq!(
        read_le_u16(&composed_ram, HDMA_TABLE_DYNAMIC + 221 * 2),
        0xae4a
    );
    assert_eq!(
        read_le_u16(&composed_ram, RESERVED_HDMA_TABLE + 221 * 2),
        0xae4a
    );
}

#[test]
fn trailing_nmi_force_blank_preserves_the_completed_fields_visible_rows() {
    let mut state = ZeldaState::new();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);
    state.schedule_dungeon_exit_spotlight_goal_caller(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
    ));
    state.set_screen_brightness(0x80);
    state.ppu.forced_blank = true;
    state.ppu.brightness = 0;
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    let scanout = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.forced_blank_from_scanline,
            display.ppu.brightness,
            display.ppu.scanout_brightness_override,
        )
    });

    assert_eq!(
        scanout,
        (true, Some(TRAILING_NMI_FORCE_BLANK_SCANLINE), 0, Some(15),)
    );

    // The same two-slice C continuation is still scheduled, but the late
    // INIDISP event was already attached to the retiring scanout above. A new
    // capture owns the live forced-blank register without replaying that
    // one-shot brightness generation.
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    let following = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.brightness,
            display.ppu.scanout_brightness_override,
        )
    });
    assert_eq!(following, (true, 0, None));
}

#[test]
fn nmi_operand_consumption_preserves_the_scanout_plan() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(1);
    state.set_subsubmodule(6);
    let entry_plan = rom_graphics_dma_plan_at_host_boundary(state.game_state.frame);
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame: state.game_state.frame,
        entry_plan,
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: state.sprite_oam_shadow_buffer().to_vec(),
    });

    state.nmi_do_updates();

    assert_eq!(
        state
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_plan),
        Some(entry_plan),
    );
}

#[test]
fn leading_nmi_uses_the_captured_link_high_plane_staging_buffers() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN]
        .fill(0x12);
    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN]
        .fill(0x34);
    let entry_plan = rom_graphics_dma_plan_at_host_boundary(state.game_state.frame);
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame: state.game_state.frame,
        entry_plan,
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: state.sprite_oam_shadow_buffer().to_vec(),
    });

    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN]
        .fill(0x56);
    state.nmi_do_updates();

    assert!(state.ppu.vram[0x4240..0x4260]
        .iter()
        .all(|&word| word == 0x1212));
    assert!(state.ppu.vram[0x4340..0x4360]
        .iter()
        .all(|&word| word == 0x3434));
}

#[test]
fn dungeon_exit_prep_oam_scanout_uses_the_published_shadow_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 7;
    let mut exit = entry;
    exit.main_module = 0x0f;
    let entry_scanout = rom_graphics_dma_plan(entry.main_module, entry.submodule).oam_scanout;
    let transition_scanout = oam_scanout_across_main(entry, exit, entry_scanout);

    assert_eq!(
        rom_graphics_dma_plan(exit.main_module, exit.submodule).oam_operands,
        GraphicsDmaGeneration::LiveAfterMain,
    );
    assert_eq!(
        transition_scanout,
        OamScanoutSource::ComposePublishedShadowDma,
    );
    assert_eq!(
        transition_scanout.resolve_live_override(false),
        OamScanoutSource::ComposePublishedShadowDma,
    );
    assert_eq!(
        transition_scanout.resolve_live_override(true),
        OamScanoutSource::ComposeLiveAfterNmi,
    );
}

#[test]
fn game_over_fade_oam_scanout_uses_the_published_shadow_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 0x12;
    entry.submodule = 4;
    entry.frame_counter = 126;
    for exit_submodule in [4, 5] {
        let mut exit = entry;
        exit.submodule = exit_submodule;
        exit.frame_counter = if exit_submodule == 4 { 127 } else { 201 };

        assert_eq!(
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }
}

#[test]
fn moving_game_over_letters_publish_the_entry_oam_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 0x12;
    entry.submodule = 7;
    entry.frame_counter = 43;
    for exit_submodule in [7, 8] {
        let mut exit = entry;
        exit.submodule = exit_submodule;
        exit.frame_counter = 44;

        assert_eq!(
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }
}

#[test]
fn game_over_menu_retains_the_captured_resident_oam() {
    let frame = crate::game_state::FrameState {
        main_module: 0x12,
        submodule: 9,
        frame_counter: 134,
        ..crate::game_state::FrameState::default()
    };

    assert!(game_over_menu_retains_resident_oam(
        frame,
        frame,
        OamScanoutSource::RetainCapturedBeforeNmi,
    ));
    assert!(!game_over_menu_retains_resident_oam(
        frame,
        frame,
        OamScanoutSource::ComposePublishedShadowDma,
    ));
}

#[test]
fn game_over_menu_oam_dma_samples_the_host_boundary_shadow() {
    let plan = rom_graphics_dma_plan(0x12, 9);
    assert_eq!(
        plan.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(plan.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
}

#[test]
fn game_over_text_uploads_run_on_successive_leading_nmi_boundaries() {
    let mut frame = crate::game_state::FrameState::default();
    frame.main_module = 0x12;
    frame.submodule = 5;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 0, 31, 0,
    ));
    assert!(!game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 1, 31, 0,
    ));
    frame.submodule = 6;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 22,
    ));
    frame.submodule = 7;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 11,
    ));
    assert!(!game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 0,
    ));
}

#[test]
fn game_over_text_draw_authors_oam_without_rescheduling_the_chr_upload() {
    let mut state = ZeldaState::new();
    state.minigame_state_mut().set_flag_boomerang_in_place(1);
    state.ancilla_slot_view_mut(0).set_x_low(0x68);
    state.ancilla_slot_view_mut(1).set_x_low(0x98);
    state.set_pending_nmi_subroutine(0x77);

    state.GameOverText_Draw();

    assert_eq!(
        &state.sprite_oam_shadow_buffer()[..16],
        &[
            0x98, 0x57, 0x41, 0x3c, 0x98, 0x5f, 0x51, 0x3c, 0x68, 0x57, 0x40, 0x3c, 0x68, 0x5f,
            0x50, 0x3c,
        ]
    );
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0x77);
}

#[test]
fn game_over_text_outer_loop_defers_the_menu_until_all_five_calls_return() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x12);
    state.set_submodule(8);
    state.game_over_text_render_calls_remaining = 5;

    for remaining_after_return in (1..=4).rev() {
        state.game_over_text_render_call_in_flight = true;
        state.finish_game_over_text_render_call();
        assert_eq!(state.game_state.frame.submodule, 8);
        assert_eq!(
            state.game_over_text_render_calls_remaining,
            remaining_after_return
        );
    }

    state.game_over_text_render_call_in_flight = true;
    state.finish_game_over_text_render_call();
    assert_eq!(state.game_state.frame.submodule, 9);
    assert_eq!(state.game_state.messaging.runtime.menu_animation_timer(), 2);
    assert_eq!(state.game_state.system_signals.music_control(), 11);
}

#[test]
fn game_over_text_cpu_slice_hides_stale_gameplay_oam_before_drawing_letters() {
    let mut state = ZeldaState::new();
    state
        .oam_state_mut()
        .write_indexed_entry_with_extended(102, 48, 117, 0, 0x1e, 2);
    state.prepare_game_over_text_oam();

    assert_eq!(state.sprite_oam_shadow_buffer()[102 * 4 + 1], 0xf0);
}

#[test]
fn dungeon_transition_handoffs_use_the_published_oam_shadow() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 7;
    let entry_scanout = rom_graphics_dma_plan(entry.main_module, entry.submodule).oam_scanout;

    for submodule in [1, 4, 0x11, 0x12, 0x13] {
        let mut exit = entry;
        exit.submodule = submodule;
        assert_eq!(
            oam_scanout_across_main(entry, exit, entry_scanout),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }

    let mut subtile_entry = entry;
    subtile_entry.submodule = 1;
    let mut subtile_exit = subtile_entry;
    subtile_exit.subsubmodule = 1;
    assert_eq!(
        oam_scanout_across_main(subtile_entry, subtile_exit, entry_scanout),
        OamScanoutSource::ComposePublishedShadowDma,
    );
}

#[test]
fn straight_interroom_stairs_keep_the_host_boundary_display_generation() {
    let plan = rom_graphics_dma_plan(7, 0x12);
    assert_eq!(plan.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
    assert_eq!(
        plan.link_obj_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(plan.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);

    for submodule in [0x11, 0x13] {
        let neighbor = rom_graphics_dma_plan(7, submodule);
        assert_eq!(neighbor.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
        assert_eq!(
            neighbor.link_obj_operands,
            GraphicsDmaGeneration::LiveAfterMain
        );
    }
}

#[test]
fn spiral_stairs_first_steady_slice_consumes_the_host_boundary_link_operands() {
    // Measured at route frame 28837 (host 28838, entry $0e/$00 -> exit $0e/$01):
    // that slice advances Link's body-pointer source words $0af0/$0af2 by 0x40
    // while Snes9x's OBJ CHR at VRAM $4220/$4320 still holds the pre-advance
    // data, so the NMI must consume the host-boundary operands. The phase begins
    // at subsubmodule $01, not $02.
    let mut exit = crate::game_state::FrameState::default();
    exit.main_module = 7;
    exit.submodule = 0x0e;
    let mut entry = exit;
    let live = rom_graphics_dma_plan(7, 0x0e).link_obj_operands;
    assert_eq!(live, GraphicsDmaGeneration::LiveAfterMain);

    for subsubmodule in 1..=3 {
        entry.subsubmodule = subsubmodule - 1;
        exit.subsubmodule = subsubmodule;
        assert_eq!(
            link_obj_operands_across_main(entry, exit, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
            "spiral-stair subsubmodule {subsubmodule:#x} must use host-boundary operands",
        );
    }

    // Subsubmodule $00 is still authored before its own NMI, so an already
    // resident $0e/$00 slice keeps the live operands; only the module entry
    // from submodule $00 overrides it.
    entry.subsubmodule = 0;
    exit.subsubmodule = 0;
    assert_eq!(
        link_obj_operands_across_main(entry, exit, live),
        GraphicsDmaGeneration::LiveAfterMain
    );
    let mut module_entry = entry;
    module_entry.submodule = 0;
    assert_eq!(
        link_obj_operands_across_main(module_entry, exit, live),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
}

#[test]
fn supertile_scroll_keeps_the_pre_main_link_generation_from_palette_entry_through_scroll() {
    let live = GraphicsDmaGeneration::LiveAfterMain;
    let state_7 = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 7,
        ..Default::default()
    };
    let state_8 = crate::game_state::FrameState {
        subsubmodule: 8,
        ..state_7
    };

    for entry in [state_7, state_8] {
        assert!(dungeon_supertile_scroll_nmi_precedes_link_animation(
            entry, state_8,
        ));
        assert_eq!(
            link_obj_scanout_across_main(entry, state_8, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
        assert_eq!(
            link_obj_operands_across_main(entry, state_8, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }

    let state_6 = crate::game_state::FrameState {
        subsubmodule: 6,
        ..state_7
    };
    let state_9 = crate::game_state::FrameState {
        subsubmodule: 9,
        ..state_8
    };
    assert!(!dungeon_supertile_scroll_nmi_precedes_link_animation(
        state_6, state_7,
    ));
    assert!(!dungeon_supertile_scroll_nmi_precedes_link_animation(
        state_8, state_9,
    ));
}

#[test]
fn host_boundary_link_oam_composition_keeps_unrelated_live_sprites() {
    let mut live_oam = vec![0x1111; 0x110];
    let host_boundary_oam = vec![0x2222; 0x110];

    compose_host_boundary_link_oam(&mut live_oam, Some(&host_boundary_oam));

    for entry in HOST_BOUNDARY_LINK_OAM_ENTRIES {
        assert_eq!(&live_oam[entry * 2..entry * 2 + 2], &[0x2222, 0x2222]);
        let high_word = 256 + entry / 8;
        let high_shift = (entry % 8) * 2;
        let high_mask = 0b11 << high_shift;
        assert_eq!(
            live_oam[high_word] & high_mask,
            host_boundary_oam[high_word] & high_mask
        );
    }
    for entry in [101, 104, 106, 108, 109, 114] {
        assert_eq!(&live_oam[entry * 2..entry * 2 + 2], &[0x1111, 0x1111]);
    }
}

#[test]
fn subtile_palette_filter_schedules_only_instruction_level_interruptions() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();

    // The terminal lightening pass has already toggled back to darkening and
    // advanced state 1 -> 2, but the common Module 7 return is still in flight
    // when NMI arrives. Countdown/direction cannot identify this boundary.
    state.set_subsubmodule(2);
    state.set_countdown(0);
    state.set_darkening_or_lightening_screen(0);
    state.dungeon_palette_cpu_advance_pending = Some(DungeonPaletteCpuAdvance {
        work: CpuWorkAdvance::InterruptedAtNmi {
            remaining_work_master_cycles: 0,
        },
        pc: 0x0d_fcd1,
        subsubmodule: 2,
        palette_countdown: 0,
    });
    state.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSubtilePaletteFilter)
    );
    assert!(
        state
            .game_execution_scheduler
            .work_suspends_translated_call_stack(),
        "the interrupted palette loop must retain Module07's caller suffix until its return"
    );

    state.game_execution_scheduler.finish_work();
    state.set_subsubmodule(1);
    state.set_countdown(2);
    state.dungeon_palette_cpu_advance_pending = Some(DungeonPaletteCpuAdvance {
        work: CpuWorkAdvance::Complete,
        pc: 0x00_8036,
        subsubmodule: 1,
        palette_countdown: 2,
    });
    state.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
    assert!(!state.game_execution_scheduler.work_is_pending());

    assert_eq!(
        dungeon_subtile_palette_filter_return_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        },
        "the return keeps resident OAM/raw CHR while its NMI publishes a newer decoded Link cache"
    );
}

#[test]
fn straight_interroom_fadeout_schedules_only_measured_caller_return_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(1);
    state.set_dungeon_room_index(0x51);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);

    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(1);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishStraightInterroomFadeoutSuffix)
    );
    assert!(
        state
            .game_execution_scheduler
            .work_suspends_translated_call_stack(),
        "the held frame must resume Module 7's caller suffix rather than run a fresh iteration",
    );
    assert_eq!(
        straight_interroom_fadeout_return_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        },
        "the held return must keep the resident OAM and decoded Link OBJ generation",
    );
    assert_eq!(
        straight_interroom_fadeout_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        },
        "the filter scanout keeps live raw OBJ bytes tied to the host-boundary source identity",
    );
    assert_eq!(
        straight_interroom_fadeout_following_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        },
        "the following scanout retains OAM while publishing the independently completed Link DMA",
    );
    state.next_display_obj_scanout_generation =
        Some(straight_interroom_fadeout_following_obj_scanout());
    state.stage_straight_interroom_fadeout_obj_source();
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(straight_interroom_fadeout_following_obj_scanout()),
        "ordinary fadeout staging must not overwrite a resumed-caller handoff",
    );

    state.game_execution_scheduler.finish_work();
    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(2);
    assert!(!state.game_execution_scheduler.work_is_pending());

    state.set_dungeon_room_index(0x52);
    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(1);
    assert!(!state.game_execution_scheduler.work_is_pending());
}

#[test]
fn hud_tilemap_upload_publishes_at_the_following_scanout() {
    let mut state = ZeldaState::new();
    state.set_message_dma_destination_address(0x60b9);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x60b9] = 0x2222;
    state.increment_hud_update_flag();
    state.capture_display_snapshot();

    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x60b9] = 0xbbbb;

    let captured =
        state.with_display_snapshot(|display| (display.ppu.vram[0], display.ppu.vram[0x60b9]));

    assert_eq!(captured, (0xaaaa, 0x2222));
}

#[test]
fn dialogue_exit_bg_packet_waits_for_the_following_nmi() {
    let mut state = ZeldaState::new();
    state.set_bg_mode(9);
    state.write_vram_upload_buffer_word(0, 0xffff);
    state.set_bg_vram_load_mode(1);

    state.interrupt_nmi(0, None, true);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 1);

    state.clear_nmi_update_latch();
    state.interrupt_nmi(0, None, false);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);
}

#[test]
fn selected_game_load_resumes_until_the_cpu_heavy_setup_finishes() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_selected_game_load();
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_NMI_SLICES
    );

    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES + 1
    );
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::BeginPreDungeonAudio)
    );
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES
    );

    for _ in 0..SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteSelectedGameLoad)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn dungeon_landing_wipe_table_projection_follows_spotlight_row_workload() {
    assert!(rom_dungeon_landing_wipe_is_active(7, 15));
    assert!(!rom_dungeon_landing_wipe_is_active(7, 14));
    assert!(!rom_dungeon_landing_wipe_is_active(14, 15));
    assert_eq!(spotlight_vertical_center(0x215a, 0x2110), 86);
    assert_eq!(spotlight_table_row_pairs(86), 139);

    assert_eq!(spotlight_table_row_pairs(42), 183);
    assert_eq!(spotlight_table_row_pairs(182), 183);
    assert!(!spotlight_table_has_long_nmi_workload(42));
    assert!(!spotlight_table_has_long_nmi_workload(182));

    assert_eq!(spotlight_table_row_pairs(41), 184);
    assert_eq!(spotlight_table_row_pairs(183), 184);
    assert!(spotlight_table_has_long_nmi_workload(41));
    assert!(spotlight_table_has_long_nmi_workload(183));
    assert!(spotlight_opening_projects_live_tail_before_hdma(0x3f, 183));
    assert!(!spotlight_opening_projects_live_tail_before_hdma(0x46, 183));
    assert!(!spotlight_opening_projects_live_tail_before_hdma(0x3f, 182));
}

#[test]
fn spotlight_close_entry_publication_follows_circle_workload() {
    let short_entry = SpotlightIterationPhase::for_close_iteration(0, 0x7e, 42);
    assert_eq!(
        short_entry,
        SpotlightIterationPhase::CloseEntryAfterTablePublication
    );
    assert_eq!(
        short_entry.close_completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );

    let long_entry = SpotlightIterationPhase::for_close_iteration(0, 0x7e, 41);
    assert_eq!(
        long_entry,
        SpotlightIterationPhase::CloseEntryBeforeTablePublication
    );
    assert_eq!(
        long_entry.close_completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x77),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x70),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x69),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert!(game_over_spotlight_build_uses_live_oam(Some(
        GameWorkContinuation::FinishGameOverSpotlightBuild {
            iteration: SpotlightIteration::game_over_closing(
                SpotlightIterationPhase::MixedTailAfterReturn,
                false,
            ),
        },
    )));
    assert!(!game_over_spotlight_build_uses_live_oam(Some(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: SpotlightIteration::game_over_closing(
                SpotlightIterationPhase::MixedTailAfterReturn,
                false,
            ),
        },
    )));
    let mut build_entry = crate::game_state::FrameState::default();
    build_entry.main_module = 0x12;
    build_entry.submodule = 3;
    build_entry.frame_counter = 105;
    let mut after_leading_nmi = build_entry;
    after_leading_nmi.frame_counter = 106;
    assert!(game_over_spotlight_build_entry_uses_live_oam(
        build_entry,
        after_leading_nmi,
    ));
    assert!(!game_over_spotlight_build_entry_uses_live_oam(
        after_leading_nmi,
        after_leading_nmi,
    ));
    assert!(game_over_spotlight_return_boundary_uses_live_oam(
        after_leading_nmi,
        0x70,
        None,
    ));
    assert!(!game_over_spotlight_return_boundary_uses_live_oam(
        after_leading_nmi,
        0x77,
        None,
    ));
    assert!(SpotlightIteration::game_over_closing(
        SpotlightIterationPhase::MixedTailAfterReturn,
        false,
    )
    .game_over_build_needs_deferred_caller_return());
    assert!(!SpotlightIteration::game_over_closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
        false,
    )
    .game_over_build_needs_deferred_caller_return());
    let mut iris_goal = after_leading_nmi;
    iris_goal.submodule = 4;
    assert!(game_over_iris_goal_scanout_is_closed(
        after_leading_nmi,
        iris_goal,
    ));
    assert!(!game_over_iris_goal_scanout_is_closed(iris_goal, iris_goal,));
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
fn legacy_poly_stack_marker_cannot_own_a_frame_after_worker_shutdown() {
    assert!(legacy_poly_scheduler_is_active(0, false, true));
    assert!(!legacy_poly_scheduler_is_active(0, false, false));
    assert!(!legacy_poly_scheduler_is_active(
        BUGFIX_POLY_RENDERER,
        false,
        true,
    ));
    assert!(!legacy_poly_scheduler_is_active(0, true, true));
}

#[test]
fn file_select_teardown_shares_the_handoff_frame_with_outgoing_poly_worker() {
    assert!(rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 0, true, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 1, true, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 0, false, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        0, 7, true, true
    ));
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
fn retiring_window_hdma_scanout_commits_its_final_ppu_latches() {
    let mut state = ZeldaState::new();
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 4].copy_from_slice(&[1, 0, 255, 0]);
    state.hdma_setup(0, 0x001b00, 1, 0, 0x26, 0);
    state.set_hdma_enable_mask(1 << 7);
    state.ppu.window1_left = 1;
    state.ppu.window1_right = 0;

    state.capture_display_snapshot();
    state.set_hdma_enable_mask(0);
    state.capture_display_snapshot();

    assert_eq!((state.ppu.window1_left, state.ppu.window1_right), (0, 255));
}

#[test]
fn retained_window_hdma_scanout_does_not_retire_twice() {
    let mut state = ZeldaState::new();
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 4].copy_from_slice(&[1, 0, 255, 0]);
    state.hdma_setup(0, 0x001b00, 1, 0, 0x26, 0);
    state.set_hdma_enable_mask(1 << 7);
    state.capture_display_snapshot();

    state.set_hdma_enable_mask(0);
    state.ppu.window1_left = 7;
    state.ppu.window1_right = 8;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);

    assert_eq!((state.ppu.window1_left, state.ppu.window1_right), (7, 8));
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
fn renderer_capture_observes_pre_nmi_state_without_rewinding_live_state() {
    let mut state = ZeldaState::new();
    state.ppu.brightness = 3;
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x5800] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.brightness = 12;
    state.ppu.vram[0] = 0x2222;
    state.ppu.vram[0x5800] = 0x4444;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.brightness,
            display.ppu.vram[0],
            display.ppu.vram[0x5800],
        )
    });

    assert_eq!(captured, (3, 0x2222, 0x3333));
    assert_eq!(state.ppu.brightness, 12);
    assert_eq!(state.ppu.vram[0], 0x2222);
    assert_eq!(state.ppu.vram[0x5800], 0x4444);
    assert!(state.display_snapshot.is_some());
}

#[test]
fn renderer_publication_exposes_consumed_dialogue_clear_without_advancing_menu_stripes() {
    let mut ordinary = ZeldaState::new();
    ordinary.ppu.vram[0] = 0x1111;
    ordinary.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    ordinary.capture_display_snapshot();
    ordinary.ppu.vram[0] = 0x2222;
    ordinary.ram[NMI_LOAD_BG_FROM_VRAM] = 0;

    assert_eq!(
        ordinary.with_display_snapshot(|display| display.ppu.vram[0]),
        0x1111
    );

    let mut dialogue_clear = ZeldaState::new();
    dialogue_clear.ppu.vram[0] = 0x3333;
    dialogue_clear.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    dialogue_clear.ram[crate::game_state::constants::VRAM_UPLOAD_DATA..][..8]
        .copy_from_slice(&[0x62, 0x44, 0x42, 0x2e, 0x7f, 0x38, 0xff, 0xff]);
    dialogue_clear.capture_display_snapshot();
    dialogue_clear.ppu.vram[0] = 0x4444;
    dialogue_clear.ram[NMI_LOAD_BG_FROM_VRAM] = 0;

    assert_eq!(
        dialogue_clear.with_display_snapshot(|display| display.ppu.vram[0]),
        0x4444
    );
}

#[test]
fn retained_display_memory_keeps_dialogue_metadata_with_its_vram_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    let pre_nmi_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.ppu.vram[0x7c00] = 0x1111;
    state.published_bg3_vwf_glyph_runs = vec![pre_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.published_dialogue_msg_read_pos = 0x2d;
    state.published_dialogue_message_id = 32;
    state.capture_display_snapshot();

    let post_nmi_run = Bg3VwfGlyphRun {
        y: 0,
        ..pre_nmi_run
    };
    state.ppu.vram[0x7c00] = 0x2222;
    state.published_bg3_vwf_glyph_runs = vec![post_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2e];
    state.published_dialogue_msg_read_pos = 0x2e;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
        )
    });

    assert_eq!(captured, (0x1111, vec![pre_nmi_run], vec![0x2d], 0x2d));
    assert_eq!(state.ppu.vram[0x7c00], 0x2222);
    assert_eq!(state.published_bg3_vwf_glyph_runs, vec![post_nmi_run]);
}

#[test]
fn recomposed_display_memory_publishes_post_nmi_dialogue_metadata_with_vram() {
    let mut state = ZeldaState::new();
    state.set_main_module(6);
    state.set_submodule(0);
    let pre_nmi_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.ppu.vram[0x7c00] = 0x1111;
    state.published_bg3_vwf_glyph_runs = vec![pre_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.published_dialogue_msg_read_pos = 0x2d;
    state.published_dialogue_message_id = 32;
    state.capture_display_snapshot();

    let post_nmi_run = Bg3VwfGlyphRun {
        y: 0,
        ..pre_nmi_run
    };
    state.ppu.vram[0x7c00] = 0x2222;
    state.published_bg3_vwf_glyph_runs = vec![post_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2e];
    state.published_dialogue_msg_read_pos = 0x2e;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
        )
    });

    assert_eq!(captured, (0x2222, vec![post_nmi_run], vec![0x2e], 0x2e));
    assert_eq!(state.ppu.vram[0x7c00], 0x2222);
    assert_eq!(state.published_bg3_vwf_glyph_runs, vec![post_nmi_run]);
}

#[test]
fn dialogue_scroll_completion_pairs_retired_dma_with_completion_metadata() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.capture_display_snapshot();
    let scroll_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.stage_dialogue_scroll_completion_after_return(DialogueTextScanout {
        vram: vec![0x3333; 0x3f0],
        glyph_runs: vec![scroll_run],
        glyph_run_dialogue_offsets: vec![0x2d],
        dialogue_msg_read_pos: 0x2d,
        dialogue_message_id: 32,
    });
    state.capture_display_snapshot();
    let staged = state.with_display_snapshot(|display| display.ppu.vram[0x7c00]);
    assert_eq!(staged, 0x3333);

    write_le_u16(&mut state.ram, 0x10000, 0x4444);
    state.bg3_vwf_glyph_runs = vec![scroll_run];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x2d);
    state.capture_display_snapshot();

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
            display.published_dialogue_message_id,
        )
    });

    assert_eq!(captured, (0x4444, vec![scroll_run], vec![0x2d], 0x2d, 32));
}

#[test]
fn nmi_force_blank_gates_the_pre_nmi_display_snapshot() {
    let mut state = ZeldaState::new();
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // zelda3/src/nmi.c:WritePpuRegisters publishes INIDISP_copy through the
    // leading NMI. The completed register receipt, rather than the unrelated
    // future live PPU latch, owns this retiring scanout.
    state.set_screen_brightness(0x80);
    state.interrupt_nmi_for_active_scanout(0, None, false);

    assert!(state
        .display_snapshot
        .as_ref()
        .and_then(|display| display.effective_presented_dma.as_ref())
        .and_then(|receipt| receipt.completed_ppu_registers)
        .is_some_and(|registers| registers.inidisp.forced_blank));
    let captured_forced_blank = state.with_display_snapshot(|display| display.ppu.forced_blank);

    assert!(captured_forced_blank);
    assert!(state.ppu.forced_blank);
}

#[test]
fn retained_spotlight_goal_scanout_rejects_the_following_main_force_blank() {
    let mut state = ZeldaState::new();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // zelda3/src/load_gfx.c:IrisSpotlight_ConfigureTable writes
    // INIDISP_copy=$80 when the closing radius reaches zero. On the standard
    // route Snes9x enters that goal call at V=21 of internal frame 11,478,
    // after it has already presented internal frame 11,477. The caller-return
    // capture therefore retains the visible field; the new live latch belongs
    // to the following scanout.
    state.set_screen_brightness(0x80);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);
    state.interrupt_nmi_for_active_scanout(0, None, false);

    assert!(state.ppu.forced_blank);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|display| !display.accepts_nmi_dma_receipts
            && display.effective_presented_dma.is_none()));

    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.brightness,
            display.ram[crate::game_state::constants::INIDISP_COPY],
        )
    });

    assert_eq!(presented, (false, 0x0f, 0x0f));
    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0);
}

#[test]
fn spotlight_goal_nmi_cannot_attach_to_the_field_that_precedes_its_cpu_work() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.ppu.forced_blank = false;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    // zelda3/src/load_gfx.c:IrisSpotlight_ConfigureTable reaches the closing
    // goal at V=21 in the pinned Snes9x trace. Its INIDISP_copy=$80 assignment
    // and first NMI crossing therefore occur after this active field began.
    state.schedule_dungeon_exit_spotlight_goal_caller(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
    ));
    assert!(state
        .game_execution_scheduler
        .active_field_precedes_current_scheduled_work());
    state.set_screen_brightness(0x80);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    state.interrupt_nmi(0, None, false);

    let published = state.display_snapshot.as_ref().unwrap();
    assert!(!published.ppu.forced_blank);
    assert!(published
        .effective_presented_dma
        .as_ref()
        .and_then(|receipt| receipt.completed_ppu_registers)
        .is_none());
    assert!(!state.with_display_snapshot(|display| display.ppu.forced_blank));
    assert!(state.ppu.forced_blank);
}

#[test]
fn staged_attract_exit_does_not_rewrite_the_drained_projection() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.forced_blank = false;

    // Seed the staged pipeline, then author the final visible field.
    state.set_screen_brightness(3);
    state.ppu.brightness = 3;
    state.ppu.vram[0] = 0xaaaa;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    state.set_screen_brightness(2);
    state.ppu.brightness = 2;
    state.ppu.vram[0] = 0x1111;
    state.attract_map_hdma_projection_before = Some(vec![0; ATTRACT_MAP_PROJECTION_WORDS * 2]);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    // The following CPU generation requests force-blank and clears VRAM before
    // its NMI. Advancing the staged pipeline must drain the visible field
    // without merging either future write back into it.
    state.set_screen_brightness(0x80);
    state.ppu.vram[0] = 0;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    assert_eq!(state.display_snapshot.as_ref().unwrap().ppu.vram[0], 0x1111);
    assert!(!state.display_snapshot.as_ref().unwrap().ppu.forced_blank);
    assert_eq!(
        state.deferred_display_snapshot.as_ref().unwrap().ram
            [crate::game_state::constants::INIDISP_COPY],
        0x80
    );
    state.ppu.forced_blank = true;
    state.ppu.brightness = 0;

    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.forced_blank,
            display.ppu.brightness,
        )
    });

    assert_eq!(presented, (0x1111, false, 2));
    assert_eq!(state.ppu.vram[0], 0);
    assert!(state.ppu.forced_blank);
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

#[test]
fn frame_input_opposites_match_snes9x_libretro_report_order() {
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x0130), 0x0120);
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x00c0), 0x0080);
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x00f0), 0x00a0);
}

#[test]
fn frame_input_opposite_resolution_preserves_non_direction_buttons() {
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x0f30), 0x0f20);
}

#[test]
fn rom_cpu_timing_dma_materializes_native_hdma_enable_mask() {
    let mut state = ZeldaState::new();
    state.dma.channel[0].hdma_active = true;
    state.dma.channel[7].hdma_active = false;
    state.set_hdma_enable_mask(0x80);

    let timing_dma = state.dma_with_native_hdma_enable();

    assert!(!timing_dma.channel[0].hdma_active);
    assert!(timing_dma.channel[7].hdma_active);
    assert!(state.dma.channel[0].hdma_active);
    assert!(!state.dma.channel[7].hdma_active);
}
