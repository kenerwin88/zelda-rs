use super::*;
use crate::game_state::constants::DUNGEON_BG2_ATTR_TABLE;
use crate::tile_definition::{NativeTile, TileBehavior, TilePair};

#[test]
fn native_tile_catalog_preserves_every_attribute_flip_and_checkpoint_identity() {
    let bytes: Vec<u8> = (0..=255).collect();
    let tiles: Vec<_> = bytes
        .iter()
        .copied()
        .map(NativeTile::from_cartridge)
        .collect();
    let wire = bincode::serialize(&tiles).unwrap();
    assert_eq!(wire, bincode::serialize(&bytes).unwrap());
    assert_eq!(
        bincode::deserialize::<Vec<NativeTile>>(&wire).unwrap(),
        tiles
    );
    for attribute in bytes {
        let tile = NativeTile::from_cartridge(attribute);
        for flips in 0..4 {
            let expected = if (0x10..0x1c).contains(&attribute) {
                attribute | flips
            } else {
                attribute
            };
            assert_eq!(
                tile.with_cartridge_orientation(flips).cartridge_attribute(),
                expected
            );
        }
        let toggled = tile
            .toggled_crystal_peg()
            .map(NativeTile::cartridge_attribute);
        assert_eq!(
            toggled,
            if attribute & !1 == 0x66 {
                Some(attribute ^ 1)
            } else {
                None
            }
        );
    }
    // The translated layout cursor adds a whole word, including the carry
    // from the first identity into the second; independent byte increments differ.
    for word in 0..=u16::MAX {
        let pair = NativeTile::import_pair(word);
        assert_eq!(
            pair.map(NativeTile::cartridge_attribute),
            word.to_le_bytes()
        );
        assert_eq!(
            NativeTile::next_pair_identity(pair).map(NativeTile::cartridge_attribute),
            word.wrapping_add(0x0101).to_le_bytes()
        );
        let pair = TilePair(pair);
        let encoded =
            |pair: TilePair| u16::from_le_bytes(pair.0.map(NativeTile::cartridge_attribute));
        assert_eq!(
            encoded(pair.descending_stair_sequence()),
            (word & 0x0707) | 0x3434
        );
        assert_eq!(
            encoded(pair.next_torch()),
            (word & 0xefef).wrapping_add(0x0101)
        );
        assert_eq!(encoded(pair.with_floor_transition()), word | 0x1010);
        assert_eq!(encoded(pair.with_palace_transition()), word | 0x2020);
        assert_eq!(encoded(pair.with_ground_on_right()), word & 0x00ff);
        assert_eq!(encoded(pair.with_ground_on_left()), word & 0xff00);
    }
}

#[test]
fn native_tile_writes_publish_exact_pairs_and_survive_layer_spills_and_restore() {
    let mut game = ZeldaState::new();
    for (index, byte) in game.ram.iter_mut().enumerate() {
        *byte = index.wrapping_mul(73) as u8;
    }
    game.sync_native_game_state_from_ram();
    for offset in [0, 63, 0x0fff, 0x1000, 0x1ffe] {
        for attribute in 0..=u8::MAX {
            let pair = [
                NativeTile::from_cartridge(attribute),
                NativeTile::from_cartridge(attribute.wrapping_add(31)),
            ];
            let mut expected = game.ram.clone();
            expected[DUNGEON_BG2_ATTR_TABLE + offset..DUNGEON_BG2_ATTR_TABLE + offset + 2]
                .copy_from_slice(&pair.map(NativeTile::cartridge_attribute));
            game.dungeon_bg2_attributes_mut()
                .set_bg2_tiles(offset, pair);
            assert_eq!(game.ram, expected);
            assert_eq!(
                game.game_state.dungeon.bg2_attributes.bg2_tile(offset),
                pair[0]
            );
            assert_eq!(
                game.game_state.dungeon.bg2_attributes.bg2_tile(offset + 1),
                pair[1]
            );
        }
    }
    let owner = &game.game_state.dungeon.bg2_attributes;
    let legacy = game.ram[DUNGEON_BG2_ATTR_TABLE..DUNGEON_BG2_ATTR_TABLE + 0x2000].to_vec();
    let wire = bincode::serialize(owner).unwrap();
    assert_eq!(wire, bincode::serialize(&legacy).unwrap());
    fn restore<T: serde::Serialize + serde::de::DeserializeOwned>(owner: &T) -> T {
        bincode::deserialize(&bincode::serialize(owner).unwrap()).unwrap()
    }
    let restored = restore(owner);
    assert_eq!(&restored, owner);
    assert_eq!(restored.bg2_tile(0x2000), NativeTile::GROUND);
}

#[test]
fn outdoor_catalog_matches_asset_lookup_and_refreshes_after_edit_and_restore() {
    let data: Vec<u8> = (0..512).map(|index| index as u8).collect();
    let mut ranges = vec![(0, 0); 164];
    ranges[163] = (0, data.len());
    let mut pack = AssetPack::from_data_ranges(data.clone(), ranges);
    for map8 in 0..=u16::MAX {
        let mut expected = data[(map8 & 0x01ff) as usize];
        if (0x10..0x1c).contains(&expected) {
            expected |= ((map8 >> 14) & 1) as u8;
        }
        assert_eq!(
            pack.outdoor_tile_definition(map8).cartridge_attribute(),
            expected
        );
    }
    let original = pack.clone();
    pack.asset_mut(163).unwrap()[0] = 0x20;
    assert_eq!(
        pack.outdoor_tile_definition(0).behavior(false),
        TileBehavior::Pit
    );
    assert_eq!(original.outdoor_tile_definition(0), NativeTile::GROUND);
    let restored: AssetPack = bincode::deserialize(&bincode::serialize(&pack).unwrap()).unwrap();
    assert_eq!(restored.outdoor_tile_definition(0), NativeTile::PIT);
}

#[test]
fn dungeon_catalog_preserves_live_graphics_aliases_and_both_orientation_bits() {
    use crate::game_state::constants::ATTRIBUTES_FOR_TILE_PLAYER;
    let mut game = ZeldaState::new();
    for index in 0..0x400 {
        game.ram[ATTRIBUTES_FOR_TILE_PLAYER + index] = index as u8;
    }
    game.sync_native_game_state_from_ram();
    // The upper half is live graphics memory, outside the parser's ownership.
    game.ram[ATTRIBUTES_FOR_TILE_PLAYER + 0x234] = 0x1a;
    for map8 in 0..=u16::MAX {
        let expected = game.ram[ATTRIBUTES_FOR_TILE_PLAYER + (map8 as usize & 0x03ff)];
        assert_eq!(
            game.dungeon_tile_definition(map8 as usize)
                .cartridge_attribute(),
            expected
        );
    }
}
