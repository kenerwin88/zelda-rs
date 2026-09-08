use super::{GarnishSlotMut, GarnishSlotView};
use crate::game_state::GameState;
use crate::zelda_rtl::ZeldaState;

// Frozen from GarnishSlotState::load_from_ram/write_to_ram and the constants at
// 3a220049536169fe1b138e94e41280b08727d495. Keep these offsets independent of the
// production accessor map: a miswired field must fail these contracts.
struct ByteField {
    name: &'static str,
    base: usize,
    read: for<'a> fn(&GarnishSlotView<'a>) -> u8,
    write: for<'a> fn(&mut GarnishSlotMut<'a>, u8),
}

const BYTE_FIELDS: [ByteField; 13] = [
    ByteField {
        name: "garnish_type",
        base: 0x1f800,
        read: |slot| slot.garnish_type(),
        write: |slot, value| slot.set_garnish_type(value),
    },
    ByteField {
        name: "x_low",
        base: 0x1f83c,
        read: |slot| slot.x_low(),
        write: |slot, value| slot.set_x_low(value),
    },
    ByteField {
        name: "x_high",
        base: 0x1f878,
        read: |slot| slot.x_high(),
        write: |slot, value| slot.set_x_high(value),
    },
    ByteField {
        name: "y_low",
        base: 0x1f81e,
        read: |slot| slot.y_low(),
        write: |slot, value| slot.set_y_low(value),
    },
    ByteField {
        name: "y_high",
        base: 0x1f85a,
        read: |slot| slot.y_high(),
        write: |slot, value| slot.set_y_high(value),
    },
    ByteField {
        name: "x_velocity",
        base: 0x1f8b4,
        read: |slot| slot.x_velocity(),
        write: |slot, value| slot.set_x_velocity(value),
    },
    ByteField {
        name: "y_velocity",
        base: 0x1f896,
        read: |slot| slot.y_velocity(),
        write: |slot, value| slot.set_y_velocity(value),
    },
    ByteField {
        name: "x_subpixel",
        base: 0x1f8f0,
        read: |slot| slot.x_subpixel(),
        write: |slot, value| slot.set_x_subpixel(value),
    },
    ByteField {
        name: "y_subpixel",
        base: 0x1f8d2,
        read: |slot| slot.y_subpixel(),
        write: |slot, value| slot.set_y_subpixel(value),
    },
    ByteField {
        name: "countdown",
        base: 0x1f90e,
        read: |slot| slot.countdown(),
        write: |slot, value| slot.set_countdown(value),
    },
    ByteField {
        name: "sprite",
        base: 0x1f92c,
        read: |slot| slot.sprite(),
        write: |slot, value| slot.set_sprite(value),
    },
    ByteField {
        name: "floor",
        base: 0x1f968,
        read: |slot| slot.floor(),
        write: |slot, value| slot.set_floor(value),
    },
    ByteField {
        name: "oam_flags",
        base: 0x1f9fe,
        read: |slot| slot.oam_flags(),
        write: |slot, value| slot.set_oam_flags(value),
    },
];

fn patterned_ram() -> Vec<u8> {
    (0..0x20000)
        .map(|offset| ((offset * 37 + offset / 251) % 255 + 1) as u8)
        .collect()
}

fn assert_ram_matches(actual: &[u8], expected: &[u8], slot: usize, field: &str) {
    assert_eq!(actual.len(), expected.len());
    let first_difference = actual
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected);
    assert_eq!(
        first_difference, None,
        "slot {slot} field {field}: unexpected WRAM byte"
    );
}

#[test]
fn all_garnish_byte_setters_write_only_the_frozen_owned_address() {
    let original = patterned_ram();
    for slot in 0..30 {
        for field in &BYTE_FIELDS {
            for value in [0, 1, 0x7f, 0x80, 0xfe, 0xff] {
                let mut ram = original.clone();
                let mut expected = original.clone();
                expected[field.base + slot] = value;

                (field.write)(&mut GarnishSlotMut::new(&mut ram, slot), value);

                assert_ram_matches(&ram, &expected, slot, field.name);
                assert_eq!(
                    (field.read)(&GarnishSlotView::new(&ram, slot)),
                    value,
                    "slot {slot} field {}",
                    field.name
                );
            }
        }
    }
}

#[test]
fn garnish_views_read_external_wram_changes_without_state_reload() {
    let mut ram = patterned_ram();
    for slot in 0..30 {
        for field in &BYTE_FIELDS {
            let address = field.base + slot;
            assert_eq!(
                (field.read)(&GarnishSlotView::new(&ram, slot)),
                ram[address]
            );
            for value in [0, 0x80, 0xff] {
                // A new borrow must observe the other writer immediately; no
                // load_from_ram or frame-boundary projection is involved.
                ram[address] = value;
                let view = GarnishSlotView::new(&ram, slot);
                assert_eq!((field.read)(&view), value, "{} slot {slot}", field.name);
                assert_eq!(view.is_empty(), ram[0x1f800 + slot] == 0);
                assert_eq!(
                    view.x(),
                    u16::from_le_bytes([ram[0x1f83c + slot], ram[0x1f878 + slot]])
                );
                assert_eq!(
                    view.y(),
                    u16::from_le_bytes([ram[0x1f81e + slot], ram[0x1f85a + slot]])
                );
            }
        }
    }
}

#[test]
fn garnish_word_setters_touch_only_the_two_split_coordinate_bytes() {
    let original = patterned_ram();
    for slot in 0..30 {
        for value in [0x0000u16, 0x00ff, 0x0100, 0x7fff, 0x8000, 0xff00, 0xffff] {
            for is_x in [true, false] {
                let mut ram = original.clone();
                let mut expected = original.clone();
                let (low, high) = if is_x {
                    (0x1f83c, 0x1f878)
                } else {
                    (0x1f81e, 0x1f85a)
                };
                let [low_byte, high_byte] = value.to_le_bytes();
                expected[low + slot] = low_byte;
                expected[high + slot] = high_byte;
                let mut writer = GarnishSlotMut::new(&mut ram, slot);
                if is_x {
                    writer.set_x(value);
                } else {
                    writer.set_y(value);
                }

                assert_ram_matches(&ram, &expected, slot, if is_x { "x" } else { "y" });
                let view = GarnishSlotView::new(&ram, slot);
                assert_eq!(if is_x { view.x() } else { view.y() }, value);
            }
        }
    }
}

#[test]
fn last_garnish_slot_preserves_neighboring_arrays_and_scratch_bytes() {
    let mut ram = patterned_ram();
    let original = ram.clone();
    let mut expected = original.clone();
    for (index, field) in BYTE_FIELDS.iter().enumerate() {
        let value = (index as u8).wrapping_mul(19);
        expected[field.base + 29] = value;
        (field.write)(&mut GarnishSlotMut::new(&mut ram, 29), value);
    }

    assert_ram_matches(&ram, &expected, 29, "all fields");
    for field in &BYTE_FIELDS {
        assert_eq!(
            ram[field.base + 30],
            original[field.base + 30],
            "{} overflow",
            field.name
        );
    }
    // The gap after the sprite array and the following sprite work bank are
    // independent owners. Slot 29 must not extend into either region.
    assert_eq!(&ram[0x1f94a..0x1f968], &original[0x1f94a..0x1f968]);
    assert_eq!(&ram[0x1fa1c..0x1fa2c], &original[0x1fa1c..0x1fa2c]);
}

#[test]
#[should_panic]
fn garnish_read_view_rejects_slot_30() {
    let ram = patterned_ram();
    let _ = GarnishSlotView::new(&ram, 30);
}

#[test]
fn garnish_writer_rejects_slot_30_before_changing_ram() {
    let mut ram = patterned_ram();
    let original = ram.clone();
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = GarnishSlotMut::new(&mut ram, 30);
    }));
    assert!(rejected.is_err(), "slot 30 must not alias the next array");
    assert_ram_matches(&ram, &original, 30, "invalid constructor");
}

#[test]
fn native_projection_cannot_restore_stale_garnish_slot_bytes() {
    let mut ram = patterned_ram();
    let native = GameState::load_from_ram(&ram);
    for field in &BYTE_FIELDS {
        for slot in 0..30 {
            // Change every byte after the old native-load boundary. Retaining
            // even one duplicate field in the projection must fail below.
            ram[field.base + slot] ^= 0xff;
        }
    }
    let expected = ram.clone();

    native.write_to_ram(&mut ram);

    for field in &BYTE_FIELDS {
        assert_eq!(
            &ram[field.base..field.base + 30],
            &expected[field.base..field.base + 30],
            "native projection re-stamped {}",
            field.name
        );
    }
}

#[test]
fn garnish_wram_survives_clone_and_bincode_without_shared_live_storage() {
    let mut live = ZeldaState::new();
    let pattern = patterned_ram();
    // Leave the other subsystems in their valid frame-zero state. Garnish is
    // deliberately updated after GameState was created, without reloading it.
    for field in &BYTE_FIELDS {
        live.ram[field.base..field.base + 30]
            .copy_from_slice(&pattern[field.base..field.base + 30]);
    }
    let expected_frozen = live.ram.clone();
    let cloned = live.clone();
    let encoded = bincode::serialize(&live).expect("serialize garnish WRAM owner");

    for field in &BYTE_FIELDS {
        for slot in 0..30 {
            let changed = expected_frozen[field.base + slot] ^ 0xff;
            (field.write)(&mut GarnishSlotMut::new(&mut live.ram, slot), changed);
        }
    }
    let mut restored: ZeldaState =
        bincode::deserialize(&encoded).expect("restore garnish WRAM owner");
    assert_ram_matches(&cloned.ram, &expected_frozen, 0, "cloned WRAM");
    assert_ram_matches(&restored.ram, &expected_frozen, 0, "serialized WRAM");

    restored.game_state.write_to_ram(&mut restored.ram);

    for field in &BYTE_FIELDS {
        for slot in 0..30 {
            let original = expected_frozen[field.base + slot];
            assert_eq!(
                (field.read)(&GarnishSlotView::new(&cloned.ram, slot)),
                original
            );
            assert_eq!(
                (field.read)(&GarnishSlotView::new(&restored.ram, slot)),
                original,
                "restored projection changed {} slot {slot}",
                field.name
            );
            assert_eq!(
                (field.read)(&GarnishSlotView::new(&live.ram, slot)),
                original ^ 0xff
            );
        }
    }
}
