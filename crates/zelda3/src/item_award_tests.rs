use super::*;
use sha2::{Digest, Sha256};

// Independent inventory layout frozen from 850202d4. The gaps are resources,
// not equipment. Keep the reserved byte lossless without assigning it behavior.
const EQUIPMENT_LAYOUT: &[(EquipmentItem, usize)] = &[
    (EquipmentItem::Bow, 0xf340),
    (EquipmentItem::Boomerang, 0xf341),
    (EquipmentItem::Hookshot, 0xf342),
    (EquipmentItem::Mushroom, 0xf344),
    (EquipmentItem::FireRod, 0xf345),
    (EquipmentItem::IceRod, 0xf346),
    (EquipmentItem::Bombos, 0xf347),
    (EquipmentItem::Ether, 0xf348),
    (EquipmentItem::Quake, 0xf349),
    (EquipmentItem::Torch, 0xf34a),
    (EquipmentItem::Hammer, 0xf34b),
    (EquipmentItem::Flute, 0xf34c),
    (EquipmentItem::BugNet, 0xf34d),
    (EquipmentItem::Book, 0xf34e),
    (EquipmentItem::CaneSomaria, 0xf350),
    (EquipmentItem::CaneByrna, 0xf351),
    (EquipmentItem::Cape, 0xf352),
    (EquipmentItem::Mirror, 0xf353),
    (EquipmentItem::Gloves, 0xf354),
    (EquipmentItem::Boots, 0xf355),
    (EquipmentItem::Flippers, 0xf356),
    (EquipmentItem::MoonPearl, 0xf357),
    (EquipmentItem::Reserved, 0xf358),
    (EquipmentItem::Sword, 0xf359),
    (EquipmentItem::Shield, 0xf35a),
    (EquipmentItem::Armor, 0xf35b),
];

#[test]
fn equipment_fields_preserve_frozen_layout_and_publish_only_the_awarded_byte() {
    fn restore<T: serde::de::DeserializeOwned>(_: &T, bytes: &[u8]) -> T {
        bincode::deserialize(bytes).unwrap()
    }
    let pattern: Vec<u8> = (0..0x20000).map(|i| (i * 37 + i / 251) as u8).collect();
    for &(equipment, address) in EQUIPMENT_LAYOUT {
        for value in [0, 1, 127, 128, 254, 255] {
            let mut game = ZeldaState::new();
            game.ram.clone_from(&pattern);
            game.game_state = GameState::load_from_ram(&game.ram);
            assert_eq!(
                game.game_state.inventory.items.equipment(equipment),
                pattern[address]
            );
            let captured = game.game_state.inventory.items;
            let encoded = bincode::serialize(&captured).unwrap();
            assert_eq!(
                encoded.len(),
                30,
                "no bomb or bottle-index copy in equipment"
            );
            assert_eq!(restore(&captured, &encoded), captured);
            let mut projected = vec![0xa5; 0x20000];
            captured.write_to_ram(&mut projected);
            let mut expected_projection = vec![0xa5; 0x20000];
            for &(_, offset) in EQUIPMENT_LAYOUT {
                expected_projection[offset] = pattern[offset];
            }
            expected_projection[0xf35c..0xf360].copy_from_slice(&pattern[0xf35c..0xf360]);
            assert!(
                projected == expected_projection,
                "equipment projection changed a foreign byte"
            );
            // Deliberately make all WRAM equipment stale. Awarding one item must
            // not bulk-project the other native equipment fields.
            for &(_, offset) in EQUIPMENT_LAYOUT {
                game.ram[offset] ^= 0xff;
            }
            let mut expected = game.ram.clone();
            expected[address] = value;
            game.inventory_items_mut().grant_equipment(equipment, value);
            assert!(game.ram == expected, "{equipment:?} changed another byte");
            assert_eq!(game.game_state.inventory.items.equipment(equipment), value);
            assert_eq!(bincode::serialize(&captured).unwrap(), encoded);
        }
    }
}

#[test]
fn inventory_resource_indices_read_the_authoritative_resource_state() {
    let mut game = ZeldaState::new();
    let equipment = game.game_state.inventory.items;
    for value in [0, 1, 127, 128, 255] {
        game.player_resources_mut().set_bombs(value);
        game.player_resources_mut().set_equipped_bottle_index(value);
        assert_eq!(game.inventory_item(3), value);
        assert_eq!(game.inventory_item(15), value);
        assert_eq!(game.game_state.inventory.items, equipment);
    }
    assert_eq!(game.inventory_item(28), 0);
    assert_eq!(game.inventory_item(usize::MAX), 0);
}

#[test]
fn conditional_equipment_award_preserves_the_legacy_compatibility_read() {
    let mut game = ZeldaState::new();
    game.inventory_items_mut()
        .grant_equipment(EquipmentItem::Armor, 7);
    game.ram[0xf35b] = 0;
    game.inventory_items_mut()
        .grant_equipment_if_empty(EquipmentItem::Armor, 1);
    assert_eq!(game.game_state.inventory.items.armor(), 1);
    assert_eq!(game.ram[0xf35b], 1);
    game.ram[0xf35b] = 2;
    game.inventory_items_mut()
        .grant_equipment_if_empty(EquipmentItem::Armor, 3);
    assert_eq!(game.game_state.inventory.items.armor(), 1);
    assert_eq!(game.ram[0xf35b], 2);
}

#[test]
fn chest_alternates_preserve_all_original_item_ids() {
    for item in 0..=255u8 {
        let expected = match item {
            12 => Some((EquipmentItem::Boomerang, 68)),
            18 => Some((EquipmentItem::Torch, 53)),
            42 => Some((EquipmentItem::Boomerang, 70)),
            _ => None,
        };
        assert_eq!(chest_item_alternate(item), expected, "chest item {item}");
    }
}

#[test]
fn item_awards_match_frozen_runtime_effects() {
    // Generated with the original address-based implementation at 850202d4.
    // Each digest covers all twelve method/inventory cases, before and after
    // native projection. Keep this independent of the candidate award table.
    let expected: Vec<_> = include_str!("testdata/item-award-effects-850202d4.txt")
        .lines()
        .collect();
    for item in 0..=255u8 {
        let mut digest = Sha256::new();
        for method in 0..4 {
            for seed in 0..3u8 {
                let mut game = ZeldaState::new();
                if item == 1 {
                    let mut ranges = vec![(0, 0); 113];
                    ranges[111] = (0, 256);
                    ranges[112] = (256, 384);
                    game.assets = Some(AssetPack::from_data_ranges(vec![0; 384], ranges));
                }
                // Fixed pre-migration cases: empty, equipped, and near-capacity.
                // Addresses deliberately do not use the candidate's mapping.
                for index in 0..28 {
                    game.ram[0xf340 + index] = seed.min(1);
                }
                game.ram[0xf343] = [0, 1, 99][seed as usize];
                game.ram[0xf34f] = seed;
                game.ram[0xf359] = seed;
                game.ram[0xf35a] = seed;
                game.ram[0xf35b] = seed;
                for (i, value) in [[0, 0, 0, 0], [2, 3, 2, 0], [6, 7, 8, 5]][seed as usize]
                    .into_iter()
                    .enumerate()
                {
                    game.ram[0xf35c + i] = value;
                }
                game.ram[0xf36b] = seed * 2;
                game.ram[0xf36c] = 0xa0;
                game.ram[0xf36d] = 0x18;
                game.ram[0xf36e] = 0x40;
                game.ram[0xf36f] = [0, 98, 99][seed as usize];
                game.ram[0xf374] = [0, 3, 6][seed as usize];
                game.ram[0xf375] = [0, 98, 99][seed as usize];
                game.ram[0xf376] = [0, 1, 255][seed as usize];
                game.ram[0x40c] = seed * 2;
                game.game_state = GameState::load_from_ram(&game.ram);
                game.follower_link_state_mut().set_receive_item_index(item);
                game.follower_link_state_mut()
                    .set_item_receipt_method(method);
                game.ancilla_add_item_receipt(0x22, 4, 0x0182);
                digest.update(&game.ram);
                game.game_state.write_to_ram(&mut game.ram);
                digest.update(&game.ram);
            }
        }
        assert_eq!(
            format!("{item:02x} {:x}", digest.finalize()),
            expected[item as usize],
            "item {item:02x} changed its runtime effects"
        );
    }
}
