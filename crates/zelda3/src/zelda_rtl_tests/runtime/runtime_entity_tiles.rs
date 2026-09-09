use super::*;
use crate::game_state::constants::SPRITE_TILETYPE;
use crate::tile_definition::{EntityCollision, NativeTile};

// The original entity tile tables, frozen verbatim as the decode contract.
// sprite.c kSprite_SimplifiedTileAttr (also Probe_CheckTileSolidity)
#[rustfmt::skip]
const SPRITE_SIMPLIFIED_TILE_ATTR: [u8; 256] = [
    0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

// sprite.c kSprite_Func5_Tab3; Sprite_CheckTileProperty tests only for zero
#[rustfmt::skip]
const SPRITE_TILE_PROPERTY_TABLE: [i8; 256] = [
    0, 1, 2, 3, 2, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 1, 2, -1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, -1, -1, -1, -1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 2, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1,
];

// ancilla.c kAncilla_TileColl_Attrs
#[rustfmt::skip]
const ANCILLA_TILE_COLLISION_ATTRS: [u8; 256] = [
    0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

// ancilla.c kAncilla_TileColl0_Attrs
#[rustfmt::skip]
const ANCILLA_TILE_COLLISION_ATTRS_LAYER0: [u8; 256] = [
    0, 1, 0, 3, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 0, 0, 0, 0, 2, 2, 2, 2, 0, 3, 3, 3,
    0, 0, 0, 0, 0, 0, 1, 1, 4, 4, 4, 4, 4, 4, 4, 4,
    1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 3, 3, 3,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

// sprite.c kSlopedTile
#[rustfmt::skip]
const SLOPED_TILE: [u8; 32] = [
    7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 7, 6, 5, 4, 3, 2, 1, 0,
];

fn collision(class: u8) -> EntityCollision {
    match class {
        0 => EntityCollision::Passable,
        1 => EntityCollision::Solid,
        2 => EntityCollision::Slope,
        3 => EntityCollision::LayerBoundary,
        4 => EntityCollision::Ledge,
        _ => unreachable!("the original tables hold classes 0..=4"),
    }
}

fn tile(attribute: u8) -> NativeTile {
    NativeTile::from_cartridge(attribute)
}

#[test]
fn entity_collision_families_match_the_original_attribute_tables() {
    for attribute in 0..=u8::MAX {
        let index = usize::from(attribute);
        let tile = tile(attribute);
        assert_eq!(
            tile.sprite_probe(),
            collision(SPRITE_SIMPLIFIED_TILE_ATTR[index]),
            "{attribute:#04x}"
        );
        assert_eq!(
            tile.blocks_sprites(),
            SPRITE_TILE_PROPERTY_TABLE[index] != 0,
            "{attribute:#04x}"
        );
        assert_eq!(
            tile.ancilla_collision(),
            collision(ANCILLA_TILE_COLLISION_ATTRS[index]),
            "{attribute:#04x}"
        );
        assert_eq!(
            tile.ancilla_ground_layer_collision(),
            collision(ANCILLA_TILE_COLLISION_ATTRS_LAYER0[index]),
            "{attribute:#04x}"
        );
        // Entity_CheckSlopedTileCollision indexes kSlopedTile by tile - 0x10;
        // only the four straight slopes stay inside the table.
        for x in 0..16u16 {
            for y in 0..16u16 {
                let expected = (0x10..0x14).contains(&attribute).then(|| {
                    let row = usize::from(attribute - 0x10);
                    let height = SLOPED_TILE[row * 8 + usize::from(x & 7)];
                    let fine_y = (y & 7) as u8;
                    if row < 2 {
                        height >= fine_y
                    } else {
                        fine_y >= height
                    }
                });
                assert_eq!(tile.entity_slope_blocks(x, y), expected, "{attribute:#04x}");
            }
        }
        // Sprite_ApplyConveyor(k, sprite_tiletype) subtracts 0x68.
        assert_eq!(
            tile.conveyor_direction(),
            (0x68..=0x6b)
                .contains(&attribute)
                .then(|| usize::from(attribute - 0x68))
        );
        // Statue_CheckForSwitch accepts 0x23, 0x24, 0x25, and 0x3b.
        assert_eq!(
            tile.is_floor_switch(),
            matches!(attribute, 0x23..=0x25 | 0x3b)
        );
    }
}

fn outdoor_state(map16_attribute: u8, map8_attribute: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(0);
    state.set_overworld_offset_base_y(0x20);
    state.set_overworld_offset_mask_y(0x1f);
    state.set_overworld_offset_base_x(3);
    state.set_overworld_offset_mask_x(0x3f);
    state.dungeon_room_tilemaps_mut().set_bg2_tile(32, 5);
    let mut data = vec![0; 0x110];
    write_le_u16(&mut data, (5 * 4 + 2) * 2, 0x4007);
    data[0x80..0x100].fill(map8_attribute);
    data[0x100 + 5] = map16_attribute;
    let mut ranges = vec![(0, 0); 165];
    ranges[70] = (0, 0x80);
    ranges[163] = (0x80, 0x100);
    ranges[164] = (0x100, 0x110);
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state
}

#[test]
fn entity_tile_lookup_selects_the_floor_layer_and_only_the_probe_publishes() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    let mut x = 0x0128u16;
    let y = 0x0030u16;
    let offset = usize::from((x & 0x01f8) >> 3) + usize::from((y & 0x01f8) << 3);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(offset, 0x27);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x1000 + offset, 0x72);
    state
        .sprite_workspace_mut()
        .set_tile(NativeTile::SPIKE_CACTUS);

    assert_eq!(state.entity_tile_at(0, &mut x, y), NativeTile::OPEN_CHEST);
    assert_eq!(state.entity_tile_at(1, &mut x, y), tile(0x72));
    assert_eq!(state.entity_tile_at(2, &mut x, y), tile(0x72));
    assert_eq!(x, 0x0128, "indoor lookups keep the pixel coordinate");
    assert_eq!(
        state.game_state.sprites.workspace.tile(),
        NativeTile::SPIKE_CACTUS
    );
    assert_eq!(state.ram[SPRITE_TILETYPE], 0x44);

    assert_eq!(state.probe_entity_tile(1, &mut x, y), tile(0x72));
    assert_eq!(state.game_state.sprites.workspace.tile(), tile(0x72));
    assert_eq!(state.ram[SPRITE_TILETYPE], 0x72);

    // Outdoors the map8 catalog answers, with the slope orientation bit,
    // and the caller's x becomes the map8 column.
    let mut state = outdoor_state(0x01, 0x10);
    let mut x = 4 << 3;
    assert_eq!(state.entity_tile_at(1, &mut x, 0x28), tile(0x11));
    assert_eq!(x, 4);
    assert_eq!(
        state.game_state.sprites.workspace.tile(),
        NativeTile::GROUND
    );
    let mut x = 4 << 3;
    assert_eq!(state.probe_entity_tile(0, &mut x, 0x28), tile(0x11));
    assert_eq!(state.game_state.sprites.workspace.tile(), tile(0x11));
    assert_eq!(state.ram[SPRITE_TILETYPE], 0x11);
}

#[test]
fn guard_probe_reads_the_map16_attribute_outdoors_and_publishes_it() {
    for (map16_attribute, solid) in [(0x00, false), (0x01, true), (0x0a, true), (0x28, true)] {
        let mut state = outdoor_state(map16_attribute, 0x01);
        state.sprite_workspace_mut().set_current_sprite_x(4 << 3);
        state.sprite_workspace_mut().set_current_sprite_y(0x28);
        assert_eq!(
            state.probe_check_tile_solidity(0),
            solid,
            "{map16_attribute:#04x}"
        );
        assert_eq!(
            state.game_state.sprites.workspace.tile(),
            tile(map16_attribute)
        );
        assert_eq!(state.ram[SPRITE_TILETYPE], map16_attribute);
    }

    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    state.sprite_slot_view_mut(0).set_floor(1);
    state.sprite_workspace_mut().set_current_sprite_x(0x0128);
    state.sprite_workspace_mut().set_current_sprite_y(0x0030);
    let offset =
        0x1000 + usize::from((0x0128u16 & 0x01f8) >> 3) + usize::from((0x0030u16 & 0x01f8) << 3);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(offset, 0x1d);
    assert!(state.probe_check_tile_solidity(0));
    assert_eq!(state.game_state.sprites.workspace.tile(), tile(0x1d));
}

#[test]
fn ancilla_probe_publishes_indoors_only_and_outdoor_slopes_read_the_stale_scratch() {
    // Indoors: the probe publishes the scratch tile and records it on the slot.
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    state.ancilla_set_xy(0, 0x40, 0x48);
    let offset = usize::from((0x40u16 & 0x01f8) >> 3) + usize::from((0x40u16 & 0x01f8) << 3);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(offset, 0x01);
    assert!(state.ancilla_check_tile_collision_class2(0));
    assert_eq!(state.game_state.sprites.workspace.tile(), tile(0x01));
    assert_eq!(state.ancilla_slot_view(0).tile_attribute(), 0x01);

    // Outdoors: the slot records the probed tile but the scratch stays
    // untouched, and a slope class consults that stale scratch profile.
    for (stale, expected) in [
        (NativeTile::SPIKE_CACTUS, true),
        (tile(0x10), tile(0x10).entity_slope_blocks(4, 0x28).unwrap()),
        (tile(0x12), tile(0x12).entity_slope_blocks(4, 0x28).unwrap()),
    ] {
        let mut state = outdoor_state(0x00, 0x18);
        state.sprite_workspace_mut().set_tile(stale);
        state.ancilla_set_xy(0, 4 << 3, 0x30);
        assert_eq!(state.ancilla_check_tile_collision_class2(0), expected);
        assert_eq!(state.ancilla_slot_view(0).tile_attribute(), 0x19);
        assert_eq!(state.game_state.sprites.workspace.tile(), stale);
    }
    let mut state = outdoor_state(0x00, 0x01);
    state.sprite_workspace_mut().set_tile(NativeTile::PIT);
    state.ancilla_set_xy(0, 4 << 3, 0x30);
    assert!(state.ancilla_check_tile_collision_class2(0));
    assert_eq!(state.game_state.sprites.workspace.tile(), NativeTile::PIT);
}

#[test]
fn sprite_tile_property_uses_the_entity_families() {
    fn indoor_state(attribute: u8) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        for offset in 0..0x2000 {
            state
                .dungeon_bg2_attributes_mut()
                .set_bg2_attr(offset, attribute);
        }
        state.sprite_workspace_mut().set_current_sprite_x(0x80);
        state.sprite_workspace_mut().set_current_sprite_y(0x80);
        state
    }
    // Deflecting sprites: ledges never collide, other classes do.
    for (attribute, expected) in [(0x28, false), (0x0a, true), (0x01, true), (0x05, false)] {
        let mut state = indoor_state(attribute);
        state.sprite_slot_view_mut(0).set_deflection_bits(8);
        assert_eq!(
            state.sprite_check_tile_property(0, 0x68),
            expected,
            "{attribute:#04x}"
        );
        assert_eq!(state.sprite_slot_view(0).e(), 0);
        assert_eq!(state.game_state.sprites.workspace.tile(), tile(attribute));
    }
    // Only outdoors does a ledge mark the deflecting sprite's ledge state.
    let mut state = outdoor_state(0x00, 0x28);
    state.garnish_state_mut().set_sprcoll_x_size(0x1000);
    state.garnish_state_mut().set_sprcoll_y_size(0x1000);
    state.sprite_workspace_mut().set_current_sprite_x(0x80);
    state.sprite_workspace_mut().set_current_sprite_y(0x80);
    state.sprite_slot_view_mut(0).set_deflection_bits(8);
    assert!(!state.sprite_check_tile_property(0, 0x68));
    assert_eq!(state.sprite_slot_view(0).e(), 4);
    assert_eq!(state.game_state.sprites.workspace.tile(), tile(0x28));

    // Ordinary sprites: the blocking table, then the pit and spike specials.
    for (attribute, expected) in [(0x05, false), (0x01, true), (0x28, true), (0x20, true)] {
        let mut state = indoor_state(attribute);
        assert_eq!(
            state.sprite_check_tile_property(0, 0x68),
            expected,
            "{attribute:#04x}"
        );
    }
    let mut state = indoor_state(0x20);
    state.sprite_slot_view_mut(0).set_flags(1);
    state.sprite_slot_view_mut(0).set_f(1);
    assert!(!state.sprite_check_tile_property(0, 0x68));
}
