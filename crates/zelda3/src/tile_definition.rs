//! Native tile identities and their predecoded behavior.

mod cartridge;
use crate::game_state::TileResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileBehavior {
    Ignore,
    Surface {
        result: TileResult,
        shift: u32,
    },
    Solid,
    DeepWaterEdge,
    ConditionalFloorTrigger,
    Slope {
        diagonal: bool,
        shape: usize,
    },
    InRoomStaircase {
        shift: u32,
    },
    Pit,
    SolidInteractable {
        misc_shift: u32,
        cactus: bool,
    },
    Ledge {
        result: TileResult,
        shift: u32,
    },
    Cactus,
    SpikedSolid,
    Aftermath,
    Liftable {
        index: u8,
        dashable: bool,
    },
    DashableSolid,
    Chest {
        index: Option<usize>,
    },
    NeighborDependent,
    MovingFloorCheck {
        shift: u32,
    },
    PushBlock {
        index: u8,
    },
    Door {
        direction: u16,
        forced_movement: bool,
        transition: Option<u8>,
        dashable: bool,
    },
    Gravestone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TileDefinition {
    cartridge_attribute: u8,
    indoor: TileBehavior,
    outdoor: TileBehavior,
}

const fn definitions() -> [TileDefinition; 256] {
    let mut table = [TileDefinition {
        cartridge_attribute: 0,
        indoor: TileBehavior::Ignore,
        outdoor: TileBehavior::Ignore,
    }; 256];
    let mut index = 0;
    while index < table.len() {
        table[index] = TileDefinition {
            cartridge_attribute: index as u8,
            indoor: TileBehavior::decode(index as u8, true),
            outdoor: TileBehavior::decode(index as u8, false),
        };
        index += 1;
    }
    table
}

static DEFINITIONS: [TileDefinition; 256] = definitions();

/// Two adjacent cells, published together by the room layout writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TilePair(pub(crate) [NativeTile; 2]);

impl TilePair {
    pub(crate) const fn repeated(tile: NativeTile) -> Self {
        Self([tile; 2])
    }
    pub(crate) const fn with_ground_on_right(self) -> Self {
        Self([self.0[0], NativeTile::GROUND])
    }
    pub(crate) const fn with_ground_on_left(self) -> Self {
        Self([NativeTile::GROUND, self.0[1]])
    }
}

/// Shared immutable definition: no separately writable byte/behavior mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativeTile(&'static TileDefinition);

impl NativeTile {
    pub(crate) fn import_slice(tiles: &mut [Self], bytes: &[u8]) {
        assert_eq!(tiles.len(), bytes.len());
        for (tile, &byte) in tiles.iter_mut().zip(bytes) {
            *tile = Self::from_cartridge(byte);
        }
    }

    pub(crate) fn export_slice(tiles: &[Self], bytes: &mut [u8]) {
        assert_eq!(tiles.len(), bytes.len());
        for (tile, byte) in tiles.iter().zip(bytes) {
            *byte = tile.cartridge_attribute();
        }
    }

    pub(crate) const fn import_pair(word: u16) -> [Self; 2] {
        [
            Self::from_cartridge(word as u8),
            Self::from_cartridge((word >> 8) as u8),
        ]
    }
    pub(crate) const GROUND: Self = Self::from_cartridge(0);
    pub(crate) const DEEP_WATER: Self = Self::from_cartridge(8);
    pub(crate) const PIT: Self = Self::from_cartridge(0x20);
    pub(crate) const OPEN_CHEST: Self = Self::from_cartridge(0x27);
    pub(crate) const SOLID_WALL: Self = Self::from_cartridge(2);

    /// Import an identity by selecting its predecoded definition.
    pub(crate) const fn from_cartridge(attribute: u8) -> Self {
        Self(&DEFINITIONS[attribute as usize])
    }

    pub(crate) const fn cartridge_attribute(self) -> u8 {
        self.0.cartridge_attribute
    }

    pub(crate) const fn behavior(self, indoors: bool) -> TileBehavior {
        if indoors {
            self.0.indoor
        } else {
            self.0.outdoor
        }
    }
}

impl Default for NativeTile {
    fn default() -> Self {
        Self::GROUND
    }
}

// Retain checkpoint wire layout; deserialization is an import boundary.
impl serde::Serialize for NativeTile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.cartridge_attribute())
    }
}

impl<'de> serde::Deserialize<'de> for NativeTile {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_cartridge(
            <u8 as serde::Deserialize>::deserialize(deserializer)?,
        ))
    }
}
