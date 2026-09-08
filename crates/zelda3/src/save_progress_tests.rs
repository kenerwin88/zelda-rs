use super::*;
use crate::game_state::save_format::{write_sram_checksum, LiveSave};
use crate::game_state::EquipmentItem;
use sha2::{Digest, Sha256};

// Captured from the pre-refactor 38a9515b implementation. Each case hashes
// the complete WRAM after interleaved owner writes, save import, and reset.
#[test]
fn save_progress_effects_match_frozen_baseline() {
    let mut results = String::new();
    for seed in 0..32u16 {
        let mut game = ZeldaState::new();
        for (index, byte) in game.ram.iter_mut().enumerate() {
            *byte = (index as u16).wrapping_mul(37).wrapping_add(seed * 13) as u8;
        }
        game.sync_native_game_state_from_ram();
        let mut digest = Sha256::new();
        for room in 0..320 {
            game.save_progress_mut()
                .or_dungeon_info_word(room, (room as u16).wrapping_mul(73));
        }
        game.player_resources_mut().set_current_health(seed as u8);
        game.inventory_items_mut()
            .grant_equipment(EquipmentItem::Bow, seed as u8);
        game.save_progress_mut().xor_palace_index_x2(7);
        game.save_progress_mut()
            .set_hud_current_item_slot(8, seed as u8);
        game.save_progress_mut().set_progress_indicator(seed as u8);
        game.save_progress_mut().or_progress_flags(0x81);
        game.save_progress_mut().xor_progress_flags(0x40);
        game.save_progress_mut().or_progress_indicator_3(0x92);
        game.save_progress_mut()
            .clear_progress_indicator_3_bits(0x10);
        game.save_progress_mut().set_map_icons_indicator(seed as u8);
        game.save_progress_mut().set_which_starting_point(3);
        game.save_progress_mut().xor_dark_world_state(0x40);
        game.save_progress_mut()
            .increment_pending_death_save_counter();
        for palace in 0..16 {
            game.save_progress_mut()
                .set_death_count_for_palace(palace, (palace as u16) * 113 + seed);
        }
        game.save_progress_mut()
            .set_total_death_save_counter(0xffff);
        game.save_progress_mut().set_dungeon_info_checksum(0x5a5a);
        digest.update(&game.ram);
        game.game_state
            .inventory
            .save_progress
            .write_to_ram(&mut game.ram);
        digest.update(&game.ram);
        let save: Vec<u8> = (0..0x500)
            .map(|i| (i as u16 * 19 + seed * 7) as u8)
            .collect();
        game.replace_live_save(&save);
        digest.update(&game.ram);
        game.save_progress_mut().request_post_message_refresh();
        digest.update(&game.ram);
        game.clear_live_save();
        digest.update(&game.ram);
        game.save_progress_mut().set_progress_indicator(2);
        game.save_progress_mut().or_dungeon_info_word(0x109, 0x82);
        digest.update(&game.ram);
        results.push_str(&format!("{:x}\n", digest.finalize()));
    }
    assert_eq!(
        results,
        include_str!("testdata/save-progress-effects-38a9515b.txt")
    );
}

#[test]
fn cartridge_save_copies_precede_live_checksum_continuations() {
    // Freeze the old SaveGameFile contract independently of the codec constants:
    // two 0x500-byte copies, then 639 little-endian words summed modulo 65536.
    // Every possible prefix is a suspension boundary, including the endpoints.
    let initial: Vec<u8> = (0..0x20000).map(|i| (i * 37 + i / 251) as u8).collect();
    for prefix in 0..=639u16 {
        let mut ram = initial.clone();
        let mut sram = vec![0xa5; 0x2000];
        let slot = usize::from(prefix % 3) * 0x500;
        let before = &initial[0xf000..0xf500];
        let save = LiveSave::from_wram(&ram);
        save.copy_to_sram(&mut sram, slot);
        let sum = save.sum_words(0..prefix, 0);
        // Change both already-read and not-yet-read words, as well as checksum
        // bytes that must never participate in the sum.
        for byte in &mut ram[0xf000..0xf500] {
            *byte ^= 0x97;
        }
        let sum = LiveSave::from_wram(&ram).sum_words(prefix..639, sum);
        let checksum = LiveSave::checksum(sum);
        let mut expected_sum = 0u16;
        for word in 0..639 {
            let bytes = if word < usize::from(prefix) {
                &initial
            } else {
                &ram
            };
            expected_sum = expected_sum.wrapping_add(u16::from_le_bytes([
                bytes[0xf000 + word * 2],
                bytes[0xf001 + word * 2],
            ]));
        }
        assert_eq!(checksum, 0x5a5au16.wrapping_sub(expected_sum));
        write_sram_checksum(&mut sram, slot, checksum);
        let mut expected = vec![0xa5; 0x2000];
        for offset in [slot, slot + 0xf00] {
            expected[offset..offset + 0x500].copy_from_slice(before);
            expected[offset + 0x4fe..offset + 0x500].copy_from_slice(&checksum.to_le_bytes());
        }
        assert_eq!(
            sram, expected,
            "prefix {prefix}: copy/checksum order or foreign bytes changed"
        );
    }
}

#[test]
fn cartridge_save_skips_incomplete_primary_and_backup_blocks() {
    let ram = vec![0x39; 0x20000];
    for length in [0, 0x4ff, 0x500, 0x9ff, 0xa00, 0x13ff, 0x1400, 0x2000] {
        for slot in [0, 0x500, 0xa00] {
            let mut sram = vec![0xa5; length];
            LiveSave::from_wram(&ram).copy_to_sram(&mut sram, slot);
            write_sram_checksum(&mut sram, slot, 0x1234);
            let mut expected = vec![0xa5; length];
            for offset in [slot, slot + 0xf00] {
                if offset + 0x500 <= length {
                    expected[offset..offset + 0x500].fill(0x39);
                    expected[offset + 0x4fe] = 0x34;
                    expected[offset + 0x4ff] = 0x12;
                }
            }
            assert_eq!(sram, expected);
        }
    }
}

#[test]
fn native_progress_owns_only_its_records_and_mutates_without_save_bank_reload() {
    let mut game = ZeldaState::new();
    for (index, byte) in game.ram.iter_mut().enumerate() {
        *byte = (index * 37 + index / 251) as u8;
    }
    game.sync_native_game_state_from_ram();
    let captured = game.game_state.inventory.save_progress.clone();
    let encoded = bincode::serialize(&captured).unwrap();
    assert_eq!(
        encoded.len(),
        686,
        "only 320 room records, named counters/flags, and six runtime bytes"
    );
    let restored: crate::game_state::SaveProgressState = bincode::deserialize(&encoded).unwrap();
    assert_eq!(restored, captured);
    let mut projected = vec![0xa5; 0x20000];
    captured.encode_progress_to_ram(&mut projected);
    let mut expected = vec![0xa5; 0x20000];
    for range in [
        0xf000..0xf280,
        0xf3c5..0xf3cb,
        0xf3e7..0xf407,
        0xf4fe..0xf500,
    ] {
        expected[range.clone()].copy_from_slice(&game.ram[range]);
    }
    assert_eq!(
        projected, expected,
        "progress must not encode another owner's bytes"
    );
    let old_flags = captured.progress_flags();
    let old_room = captured.dungeon_info_word(319);
    game.ram[0xf3c6] ^= 0xff;
    game.ram[0xf27e] ^= 0xff;
    game.player_resources_mut().set_current_health(0x38);
    let foreign_before = game.ram[0xf280..0xf3c5].to_vec();
    game.save_progress_mut().or_progress_flags(0x80);
    game.save_progress_mut().or_dungeon_info_word(319, 0x8000);
    assert_eq!(
        game.game_state.inventory.save_progress.progress_flags(),
        old_flags | 0x80
    );
    assert_eq!(
        game.game_state
            .inventory
            .save_progress
            .dungeon_info_word(319),
        old_room | 0x8000
    );
    assert_eq!(game.ram[0xf3c6], old_flags | 0x80);
    assert_eq!(read_le_u16(&game.ram, 0xf27e), old_room | 0x8000);
    assert_eq!(&game.ram[0xf280..0xf3c5], foreign_before);
    assert_eq!(
        bincode::serialize(&captured).unwrap(),
        encoded,
        "captured snapshots must remain independent"
    );
}

#[test]
fn extended_room_indices_preserve_the_source_save_word_contract() {
    // The former SaveProgressState bank admitted all 640 words, not just
    // the 320 room records. Cover every alias and the first rejected index.
    let pattern: Vec<u8> = (0..0x20000).map(|i| (i * 37 + i / 251) as u8).collect();
    for room in 0..=640 {
        let mut game = ZeldaState::new();
        game.ram.clone_from(&pattern);
        game.sync_native_game_state_from_ram();
        let resources_before = game.game_state.inventory.player_resources.clone();
        let before = if room < 640 {
            read_le_u16(&pattern, 0xf000 + room * 2)
        } else {
            0
        };
        assert_eq!(game.saved_room_flags(room), before);
        let flags = before | 0x800f;
        assert_eq!(game.add_saved_room_flags(room, 0x800f), flags);
        let mut expected = pattern.clone();
        if room < 640 {
            write_le_u16(&mut expected, 0xf000 + room * 2, flags);
        }
        assert_eq!(
            game.ram, expected,
            "room index {room}: missing or extra write"
        );
        assert_eq!(
            game.saved_room_flags(room),
            if room < 640 { flags } else { 0 }
        );
        assert_eq!(
            game.game_state.inventory.save_progress,
            crate::game_state::SaveProgressState::load_from_ram(&game.ram)
        );
        assert_eq!(
            game.game_state.inventory.player_resources, resources_before,
            "aliased resource writes must retain their original import timing"
        );
    }
}
