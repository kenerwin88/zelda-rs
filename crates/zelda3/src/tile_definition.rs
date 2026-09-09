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

/// How a sprite, ancilla, or overlord probe treats a tile. The original
/// kept four attribute tables for these consumers; two were identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EntityCollision {
    Passable,
    Solid,
    /// Profiled by the entity slope of the probed scratch tile; the diagonal
    /// slopes carry this class without a profile and stay solid.
    Slope,
    /// In-room staircases and their landing: solid only from the other layer.
    LayerBoundary,
    /// Ledges: sprites outdoors record a ledge state and ancillae swap
    /// priority instead of colliding.
    Ledge,
}

/// What a dungeon room's logic recognises a tile as, beyond collision:
/// the objects it tracks, doors, switches, staircases, and torches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonRole {
    None,
    /// The three in-room staircase families, numbered as the room loader's
    /// `kind_of_in_room_staircase`.
    InRoomStaircase {
        kind: u8,
    },
    /// Pressure plates: the plain plate, then the two held variants.
    FloorSwitch {
        variant: u8,
    },
    /// Head tile of an inter-room spiral staircase.
    SpiralStairHead,
    /// Landing that selects the staircase index a transition uses.
    StairLanding {
        index: usize,
    },
    StraightStairHead {
        descending: bool,
    },
    StarSwitch {
        toggled: bool,
    },
    WallSpiralStairHead {
        second: bool,
    },
    BombableFloor,
    MinigameChest,
    /// Sword-cuttable curtain panels.
    Curtain {
        panel: usize,
    },
    /// A liftable or pushable object tracked by the room's replacement table.
    TrackedObject {
        slot: usize,
    },
    OpenDoor,
    SomariaPipe {
        junction: PipeJunction,
    },
    Torch {
        slot: usize,
    },
    ClosedDoor {
        slot: usize,
    },
}

/// How a Cane of Somaria platform or pipe traveller reacts on a pipe tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipeJunction {
    Straight,
    ZigZagRising,
    ZigZagFalling,
    Transit,
    TeeNoUp,
    TeeNoDown,
    TeeNoLeft,
    TeeNoRight,
    TransitNoBack,
    TransitQuestion,
    Endpoint,
    /// The last pipe identity: the platform poof counts it as pipe, but the
    /// path search never stops on it.
    Boundary,
}

/// Fine-position profile of the four straight slopes as entities see them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntitySlope {
    heights: [u8; 8],
    blocked_up_to_height: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TileDefinition {
    cartridge_attribute: u8,
    indoor: TileBehavior,
    outdoor: TileBehavior,
    sprite_probe: EntityCollision,
    blocks_sprites: bool,
    ancilla: EntityCollision,
    ancilla_ground_layer: EntityCollision,
    entity_slope: Option<EntitySlope>,
    dungeon_role: DungeonRole,
    transition_landing: u8,
    accepts_push_block: bool,
}

const fn definitions() -> [TileDefinition; 256] {
    let mut table = [TileDefinition {
        cartridge_attribute: 0,
        indoor: TileBehavior::Ignore,
        outdoor: TileBehavior::Ignore,
        sprite_probe: EntityCollision::Passable,
        blocks_sprites: false,
        ancilla: EntityCollision::Passable,
        ancilla_ground_layer: EntityCollision::Passable,
        entity_slope: None,
        dungeon_role: DungeonRole::None,
        transition_landing: 0,
        accepts_push_block: false,
    }; 256];
    let mut index = 0;
    while index < table.len() {
        let attribute = index as u8;
        table[index] = TileDefinition {
            cartridge_attribute: attribute,
            indoor: TileBehavior::decode(attribute, true),
            outdoor: TileBehavior::decode(attribute, false),
            sprite_probe: EntityCollision::decode_sprite_probe(attribute),
            blocks_sprites: EntityCollision::decode_blocks_sprites(attribute),
            ancilla: EntityCollision::decode_ancilla(attribute),
            ancilla_ground_layer: EntityCollision::decode_ancilla_ground_layer(attribute),
            entity_slope: EntitySlope::decode(attribute),
            dungeon_role: DungeonRole::decode(attribute),
            transition_landing: DungeonRole::decode_transition_landing(attribute),
            accepts_push_block: DungeonRole::decode_accepts_push_block(attribute),
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
    /// The shared identity when both cells agree, as the room logic's
    /// whole-word comparisons require.
    pub(crate) fn uniform(self) -> Option<NativeTile> {
        (self.0[0] == self.0[1]).then_some(self.0[0])
    }
    pub(crate) fn stair_landing(index: usize) -> Self {
        Self::repeated(NativeTile::stair_landing(index))
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
    pub(crate) const SHALLOW_WATER: Self = Self::from_cartridge(9);
    pub(crate) const MOVING_FLOOR: Self = Self::from_cartridge(0x0c);
    pub(crate) const WATER_STAIRCASE: Self = Self::from_cartridge(0x1c);
    pub(crate) const PIT: Self = Self::from_cartridge(0x20);
    pub(crate) const OPEN_CHEST: Self = Self::from_cartridge(0x27);
    pub(crate) const GRASS: Self = Self::from_cartridge(0x40);
    pub(crate) const SPIKE_CACTUS: Self = Self::from_cartridge(0x44);
    pub(crate) const SOLID_WALL: Self = Self::from_cartridge(2);
    /// Wall that the upper layer passes; ancillae treat it as a layer boundary.
    pub(crate) const LAYER_WALL: Self = Self::from_cartridge(3);
    /// Floor between layers beside a wet staircase.
    pub(crate) const LAYER_LANDING: Self = Self::from_cartridge(0x0a);
    pub(crate) const IN_ROOM_STAIR_PSEUDO_UP_NORTH: Self = Self::from_cartridge(0x1d);
    pub(crate) const PRESSURE_PLATE: Self = Self::from_cartridge(0x23);
    pub(crate) const SPIRAL_STAIR_HEAD: Self = Self::from_cartridge(0x26);
    pub(crate) const STRAIGHT_STAIR_UP_HEAD: Self = Self::from_cartridge(0x38);
    pub(crate) const STRAIGHT_STAIR_DOWN_HEAD: Self = Self::from_cartridge(0x39);
    pub(crate) const STAR_SWITCH: Self = Self::from_cartridge(0x3b);
    pub(crate) const WALL_SPIRAL_STAIR_HEAD: Self = Self::from_cartridge(0x5e);
    pub(crate) const WALL_SPIRAL_STAIR_HEAD_2: Self = Self::from_cartridge(0x5f);
    pub(crate) const MINIGAME_CHEST: Self = Self::from_cartridge(0x63);

    pub(crate) const fn stair_landing(index: usize) -> Self {
        Self::from_cartridge(0x30 | (index as u8 & 7))
    }

    pub(crate) const fn torch(slot: usize) -> Self {
        Self::from_cartridge(0xc0 | (slot as u8 & 15))
    }

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

    /// Sprite probes and the guard's forward probe share one classification.
    pub(crate) const fn sprite_probe(self) -> EntityCollision {
        self.0.sprite_probe
    }

    /// Whether a sprite's directional tile property treats the tile as an obstacle.
    pub(crate) const fn blocks_sprites(self) -> bool {
        self.0.blocks_sprites
    }

    pub(crate) const fn ancilla_collision(self) -> EntityCollision {
        self.0.ancilla
    }

    /// Ancillae in single-layer rooms use a separate classification.
    pub(crate) const fn ancilla_ground_layer_collision(self) -> EntityCollision {
        self.0.ancilla_ground_layer
    }

    /// Whether an entity at the fine position is inside the slope's solid
    /// half; `None` for tiles without an entity slope profile.
    pub(crate) fn entity_slope_blocks(self, x: u16, y: u16) -> Option<bool> {
        let slope = self.0.entity_slope?;
        let height = slope.heights[usize::from(x & 7)];
        let fine_y = (y & 7) as u8;
        Some(if slope.blocked_up_to_height {
            height >= fine_y
        } else {
            fine_y >= height
        })
    }

    /// The switch identities a pushed statue can settle on.
    pub(crate) const fn is_floor_switch(self) -> bool {
        matches!(
            self.dungeon_role(),
            DungeonRole::FloorSwitch { .. } | DungeonRole::StarSwitch { toggled: false }
        )
    }

    pub(crate) const fn dungeon_role(self) -> DungeonRole {
        self.0.dungeon_role
    }

    /// Whether a Somaria path search stops here.
    pub(crate) const fn is_pipe_path(self) -> bool {
        match self.0.dungeon_role {
            DungeonRole::SomariaPipe { junction } => !matches!(junction, PipeJunction::Boundary),
            _ => false,
        }
    }

    pub(crate) const fn pipe_junction(self) -> Option<PipeJunction> {
        match self.0.dungeon_role {
            DungeonRole::SomariaPipe { junction } => Some(junction),
            _ => None,
        }
    }

    /// Landing class a room transition reads under the player; the original
    /// masked every identity, not only doors, so this covers all 256.
    pub(crate) const fn transition_landing(self) -> u8 {
        self.0.transition_landing
    }

    /// Whether a pushed block may slide onto this tile.
    pub(crate) const fn accepts_push_block(self) -> bool {
        self.0.accepts_push_block
    }

    /// The slot nibble shared by tracked objects, torches, and closed doors.
    /// The original masked it without checking the family at some read
    /// points, so this stays available for every identity.
    pub(crate) const fn object_slot(self) -> usize {
        (self.cartridge_attribute() & 0x0f) as usize
    }

    /// Conveyor index of the four moving-floor directions.
    pub(crate) const fn conveyor_direction(self) -> Option<usize> {
        match self.0.indoor {
            TileBehavior::MovingFloorCheck { shift } => Some((shift / 4) as usize),
            _ => None,
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
