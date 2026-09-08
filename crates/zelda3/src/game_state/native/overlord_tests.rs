use super::{OverlordSlotMut, OverlordSlotView};
use crate::game_state::GameState;
use crate::zelda_rtl::ZeldaState;

// Frozen from OverlordSlotsState and constants at 1f4936c8. These fixtures must
// not use the production address map: incorrect wiring must fail the contracts.
const OWNED_RANGES: [std::ops::Range<usize>; 2] = [0xb00..0xb58, 0xcca..0xcd2];

#[derive(Clone, Copy)]
enum ByteEffect {
    Set,
    Add,
    Subtract,
    Increment,
    Clear,
}

struct ByteOperation {
    name: &'static str,
    base: usize,
    effect: ByteEffect,
    apply: for<'a> fn(&mut OverlordSlotMut<'a>, u8),
}

const BYTE_OPERATIONS: [ByteOperation; 17] = [
    ByteOperation {
        name: "set_x_low",
        base: 0xb08,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_x_low(v),
    },
    ByteOperation {
        name: "set_x_high",
        base: 0xb10,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_x_high(v),
    },
    ByteOperation {
        name: "increment_x_high",
        base: 0xb10,
        effect: ByteEffect::Increment,
        apply: |s, _| s.increment_x_high(),
    },
    ByteOperation {
        name: "add_x_low",
        base: 0xb08,
        effect: ByteEffect::Add,
        apply: |s, v| s.add_x_low(v),
    },
    ByteOperation {
        name: "set_y_low",
        base: 0xb18,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_y_low(v),
    },
    ByteOperation {
        name: "set_y_high",
        base: 0xb20,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_y_high(v),
    },
    ByteOperation {
        name: "subtract_x_low",
        base: 0xb08,
        effect: ByteEffect::Subtract,
        apply: |s, v| s.subtract_x_low(v),
    },
    ByteOperation {
        name: "set_overlord_type",
        base: 0xb00,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_overlord_type(v),
    },
    ByteOperation {
        name: "clear",
        base: 0xb00,
        effect: ByteEffect::Clear,
        apply: |s, _| s.clear(),
    },
    ByteOperation {
        name: "set_gen1",
        base: 0xb28,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_gen1(v),
    },
    ByteOperation {
        name: "add_gen1",
        base: 0xb28,
        effect: ByteEffect::Add,
        apply: |s, v| s.add_gen1(v),
    },
    ByteOperation {
        name: "subtract_gen1",
        base: 0xb28,
        effect: ByteEffect::Subtract,
        apply: |s, v| s.subtract_gen1(v),
    },
    ByteOperation {
        name: "set_gen2",
        base: 0xb30,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_gen2(v),
    },
    ByteOperation {
        name: "add_gen2",
        base: 0xb30,
        effect: ByteEffect::Add,
        apply: |s, v| s.add_gen2(v),
    },
    ByteOperation {
        name: "set_gen3",
        base: 0xb38,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_gen3(v),
    },
    ByteOperation {
        name: "set_floor",
        base: 0xb40,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_floor(v),
    },
    ByteOperation {
        name: "set_spawned_area",
        base: 0xcca,
        effect: ByteEffect::Set,
        apply: |s, v| s.set_spawned_area(v),
    },
];

#[derive(Clone, Copy)]
enum WordEffect {
    Set,
    Add,
    SubtractAndReturn,
}

struct WordOperation {
    name: &'static str,
    low: usize,
    high: usize,
    stride: usize,
    effect: WordEffect,
    apply: for<'a> fn(&mut OverlordSlotMut<'a>, u16) -> Option<u16>,
}

const WORD_OPERATIONS: [WordOperation; 9] = [
    WordOperation {
        name: "set_x",
        low: 0xb08,
        high: 0xb10,
        stride: 1,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_x(v);
            None
        },
    },
    WordOperation {
        name: "set_adjacent_x_low_word",
        low: 0xb08,
        high: 0xb09,
        stride: 1,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_adjacent_x_low_word(v);
            None
        },
    },
    WordOperation {
        name: "subtract_adjacent_x_low_word",
        low: 0xb08,
        high: 0xb09,
        stride: 1,
        effect: WordEffect::SubtractAndReturn,
        apply: |s, v| Some(s.subtract_adjacent_x_low_word(v)),
    },
    WordOperation {
        name: "set_circle_x",
        low: 0xb10,
        high: 0xb20,
        stride: 1,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_circle_x(v);
            None
        },
    },
    WordOperation {
        name: "set_circle_y",
        low: 0xb30,
        high: 0xb40,
        stride: 1,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_circle_y(v);
            None
        },
    },
    WordOperation {
        name: "set_y",
        low: 0xb18,
        high: 0xb20,
        stride: 1,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_y(v);
            None
        },
    },
    WordOperation {
        name: "add_gen1_word",
        low: 0xb28,
        high: 0xb29,
        stride: 1,
        effect: WordEffect::Add,
        apply: |s, v| {
            s.add_gen1_word(v);
            None
        },
    },
    WordOperation {
        name: "add_gen2_word",
        low: 0xb30,
        high: 0xb31,
        stride: 1,
        effect: WordEffect::Add,
        apply: |s, v| {
            s.add_gen2_word(v);
            None
        },
    },
    WordOperation {
        name: "set_sprite_block_pos",
        low: 0xb48,
        high: 0xb49,
        stride: 2,
        effect: WordEffect::Set,
        apply: |s, v| {
            s.set_sprite_block_pos(v);
            None
        },
    },
];

struct ReadField {
    name: &'static str,
    low: usize,
    high: Option<usize>,
    stride: usize,
    read: for<'a> fn(&OverlordSlotView<'a>) -> u16,
}

const READ_FIELDS: [ReadField; 16] = [
    ReadField {
        name: "x",
        low: 0xb08,
        high: Some(0xb10),
        stride: 1,
        read: |s| s.x(),
    },
    ReadField {
        name: "y",
        low: 0xb18,
        high: Some(0xb20),
        stride: 1,
        read: |s| s.y(),
    },
    ReadField {
        name: "x_low",
        low: 0xb08,
        high: None,
        stride: 1,
        read: |s| s.x_low().into(),
    },
    ReadField {
        name: "adjacent_x_low_word",
        low: 0xb08,
        high: Some(0xb09),
        stride: 1,
        read: |s| s.adjacent_x_low_word(),
    },
    ReadField {
        name: "x_high",
        low: 0xb10,
        high: None,
        stride: 1,
        read: |s| s.x_high().into(),
    },
    ReadField {
        name: "y_low",
        low: 0xb18,
        high: None,
        stride: 1,
        read: |s| s.y_low().into(),
    },
    ReadField {
        name: "y_high",
        low: 0xb20,
        high: None,
        stride: 1,
        read: |s| s.y_high().into(),
    },
    ReadField {
        name: "overlord_type",
        low: 0xb00,
        high: None,
        stride: 1,
        read: |s| s.overlord_type().into(),
    },
    ReadField {
        name: "gen1",
        low: 0xb28,
        high: None,
        stride: 1,
        read: |s| s.gen1().into(),
    },
    ReadField {
        name: "gen1_word",
        low: 0xb28,
        high: Some(0xb29),
        stride: 1,
        read: |s| s.gen1_word(),
    },
    ReadField {
        name: "gen2",
        low: 0xb30,
        high: None,
        stride: 1,
        read: |s| s.gen2().into(),
    },
    ReadField {
        name: "gen2_word",
        low: 0xb30,
        high: Some(0xb31),
        stride: 1,
        read: |s| s.gen2_word(),
    },
    ReadField {
        name: "gen3",
        low: 0xb38,
        high: None,
        stride: 1,
        read: |s| s.gen3().into(),
    },
    ReadField {
        name: "floor",
        low: 0xb40,
        high: None,
        stride: 1,
        read: |s| s.floor().into(),
    },
    ReadField {
        name: "spawned_area",
        low: 0xcca,
        high: None,
        stride: 1,
        read: |s| s.spawned_area().into(),
    },
    ReadField {
        name: "sprite_block_pos",
        low: 0xb48,
        high: Some(0xb49),
        stride: 2,
        read: |s| s.sprite_block_pos(),
    },
];

fn patterned_ram() -> Vec<u8> {
    (0..0x20000)
        .map(|offset| ((offset * 37 + offset / 251) % 255 + 1) as u8)
        .collect()
}

fn assert_ram_matches(actual: &[u8], expected: &[u8], slot: usize, operation: &str) {
    assert_eq!(actual.len(), expected.len());
    if actual != expected {
        let difference = actual.iter().zip(expected).position(|(a, b)| a != b);
        panic!("slot {slot}, {operation}: unexpected WRAM byte at {difference:?}");
    }
}

#[test]
fn overlord_byte_operations_preserve_frozen_addresses_and_wrapping() {
    let pattern = patterned_ram();
    for operation in &BYTE_OPERATIONS {
        // Boss scratch indices deliberately continue into subsequent arrays.
        for slot in [0, 1, 2, 3, 4, 5, 6, 7, 8, 15, 20] {
            if operation.base == 0xcca && slot >= 8 {
                continue;
            }
            let address = operation.base + slot;
            for initial in [0, 1, 0x7f, 0x80, 0xfe, 0xff] {
                for argument in [0, 1, 0x7f, 0x80, 0xfe, 0xff] {
                    let mut ram = pattern.clone();
                    ram[address] = initial;
                    let mut expected = ram.clone();
                    expected[address] = match operation.effect {
                        ByteEffect::Set => argument,
                        ByteEffect::Add => initial.wrapping_add(argument),
                        ByteEffect::Subtract => initial.wrapping_sub(argument),
                        ByteEffect::Increment => initial.wrapping_add(1),
                        ByteEffect::Clear => 0,
                    };
                    (operation.apply)(&mut OverlordSlotMut::new(&mut ram, slot), argument);
                    assert_ram_matches(&ram, &expected, slot, operation.name);
                }
            }
        }
    }
}

#[test]
fn overlord_word_operations_preserve_split_adjacent_circle_and_return_contracts() {
    let pattern = patterned_ram();
    for operation in &WORD_OPERATIONS {
        for slot in [0, 1, 2, 3, 4, 5, 6, 7, 8, 15, 20] {
            let low = operation.low + slot * operation.stride;
            let high = operation.high + slot * operation.stride;
            if high >= 0xb58 {
                continue;
            }
            for initial in [0u16, 0x00ff, 0x0100, 0x7fff, 0x8000, 0xff00, 0xffff] {
                for argument in [0u16, 1, 0x00ff, 0x0100, 0x7fff, 0x8000, 0xffff] {
                    let mut ram = pattern.clone();
                    [ram[low], ram[high]] = initial.to_le_bytes();
                    let mut expected = ram.clone();
                    let result = match operation.effect {
                        WordEffect::Set => argument,
                        WordEffect::Add => initial.wrapping_add(argument),
                        WordEffect::SubtractAndReturn => initial.wrapping_sub(argument),
                    };
                    [expected[low], expected[high]] = result.to_le_bytes();
                    let returned =
                        (operation.apply)(&mut OverlordSlotMut::new(&mut ram, slot), argument);
                    assert_eq!(
                        returned,
                        matches!(operation.effect, WordEffect::SubtractAndReturn).then_some(result),
                        "{} return",
                        operation.name
                    );
                    assert_ram_matches(&ram, &expected, slot, operation.name);
                }
            }
        }
    }
}

#[test]
fn overlord_getters_reborrow_raw_wram_and_keep_legacy_cross_array_indices() {
    let mut ram = patterned_ram();
    for field in &READ_FIELDS {
        for slot in [0, 1, 2, 3, 4, 5, 6, 7, 8, 15, 20, 23, 24, 47, 79, 80, 88] {
            if field.low == 0xcca && slot >= 8 {
                continue;
            }
            let low = field.low + slot * field.stride;
            let high = field.high.map(|base| base + slot * field.stride);
            for value in [0u16, 0x0180, 0xff7f, 0xffff] {
                ram[low] = value as u8;
                if let Some(high) = high {
                    ram[high] = (value >> 8) as u8;
                }
                // Out-of-work-bank bytes remain invisible even when RAM holds
                // nonzero sprite state at the same computed address.
                let expected_byte = |address| {
                    if (0xb00..0xb58).contains(&address) || field.low == 0xcca {
                        u16::from(ram[address])
                    } else {
                        0
                    }
                };
                let expected =
                    expected_byte(low) | high.map_or(0, |address| expected_byte(address) << 8);
                assert_eq!(
                    (field.read)(&OverlordSlotView::new(&ram, slot)),
                    expected,
                    "{} slot {slot}",
                    field.name
                );
            }
        }
    }

    OverlordSlotMut::new(&mut ram, 8).set_x_low(0xa5);
    assert_eq!(OverlordSlotView::new(&ram, 0).x_high(), 0xa5);
    OverlordSlotMut::new(&mut ram, 7).set_adjacent_x_low_word(0x1234);
    assert_eq!(OverlordSlotView::new(&ram, 7).x_low(), 0x34);
    assert_eq!(OverlordSlotView::new(&ram, 0).x_high(), 0x12);
}

#[test]
fn overlord_last_owned_bytes_are_writable_without_touching_neighbors() {
    let original = patterned_ram();
    for operation in &BYTE_OPERATIONS {
        let last = if operation.base == 0xcca {
            0xcd1
        } else {
            0xb57
        };
        let slot = last - operation.base;
        let mut ram = original.clone();
        let mut expected = original.clone();
        expected[last] = match operation.effect {
            ByteEffect::Set => 0x80,
            ByteEffect::Add => original[last].wrapping_add(0x80),
            ByteEffect::Subtract => original[last].wrapping_sub(0x80),
            ByteEffect::Increment => original[last].wrapping_add(1),
            ByteEffect::Clear => 0,
        };
        (operation.apply)(&mut OverlordSlotMut::new(&mut ram, slot), 0x80);
        assert_ram_matches(&ram, &expected, slot, operation.name);
    }
    let mut ram = original.clone();
    let mut expected = original;
    expected[0xb56] = 0xff;
    expected[0xb57] = 0x80;
    OverlordSlotMut::new(&mut ram, 7).set_sprite_block_pos(0x80ff);
    assert_ram_matches(&ram, &expected, 7, "last sprite_block_pos");
}

#[test]
fn overlord_out_of_bank_writes_panic_without_publishing_partial_or_foreign_bytes() {
    let original = patterned_ram();
    for operation in &BYTE_OPERATIONS {
        let end = if operation.base == 0xcca {
            0xcd2
        } else {
            0xb58
        };
        let slot = end - operation.base;
        let mut ram = original.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (operation.apply)(&mut OverlordSlotMut::new(&mut ram, slot), 0xa5);
        }));
        assert!(
            result.is_err(),
            "{} must reject slot {slot}",
            operation.name
        );
        assert_ram_matches(&ram, &original, slot, operation.name);
    }
    for operation in &WORD_OPERATIONS {
        // For split and adjacent words, low may still belong to the work bank
        // while high does not. The old bridge could not sync after that panic.
        let slot = (0xb58 - operation.high).div_ceil(operation.stride);
        let mut ram = original.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (operation.apply)(&mut OverlordSlotMut::new(&mut ram, slot), 0xa55a);
        }));
        assert!(
            result.is_err(),
            "{} must reject slot {slot}",
            operation.name
        );
        assert_ram_matches(&ram, &original, slot, operation.name);
    }
    assert!(
        std::panic::catch_unwind(|| OverlordSlotView::new(&original, 8).spawned_area()).is_err()
    );
}

#[test]
fn sprite_state_projection_cannot_restore_stale_overlord_work_or_spawned_bytes() {
    let mut ram = patterned_ram();
    let native = GameState::load_from_ram(&ram);
    for range in OWNED_RANGES {
        for byte in &mut ram[range] {
            *byte ^= 0xff;
        }
    }
    let expected = ram.clone();
    // SpriteState formerly projected the duplicate overlord bank. Other
    // GameState projections intentionally reuse parts of this range in menus.
    native.sprites.write_to_ram(&mut ram);
    for range in OWNED_RANGES {
        assert_eq!(
            &ram[range.clone()],
            &expected[range],
            "SpriteState projection restored stale overlord bytes"
        );
    }
}

#[test]
fn overlord_snapshots_are_independent_and_survive_sprite_state_projection() {
    let mut live = ZeldaState::new();
    let pattern = patterned_ram();
    for range in OWNED_RANGES {
        live.ram[range.clone()].copy_from_slice(&pattern[range]);
    }
    let frozen = live.ram.clone();
    let cloned = live.clone();
    let encoded = bincode::serialize(&live).expect("serialize overlord WRAM owner");
    // The legacy type accessor can address every byte in the shared work bank.
    for index in 0..88 {
        OverlordSlotMut::new(&mut live.ram, index).set_overlord_type(frozen[0xb00 + index] ^ 0xff);
    }
    for index in 0..8 {
        OverlordSlotMut::new(&mut live.ram, index).set_spawned_area(frozen[0xcca + index] ^ 0xff);
    }
    let mut restored: ZeldaState =
        bincode::deserialize(&encoded).expect("restore overlord WRAM owner");
    assert_ram_matches(&cloned.ram, &frozen, 0, "cloned WRAM");
    assert_ram_matches(&restored.ram, &frozen, 0, "serialized WRAM");
    // Check the migrated owner's former SpriteState projection independently
    // of other mode-specific projections that deliberately alias these bytes.
    restored.game_state.sprites.write_to_ram(&mut restored.ram);
    for range in OWNED_RANGES {
        assert_eq!(
            &restored.ram[range.clone()],
            &frozen[range.clone()],
            "SpriteState projection changed restored overlord bytes"
        );
        for address in range {
            assert_eq!(live.ram[address], frozen[address] ^ 0xff);
        }
    }
}
