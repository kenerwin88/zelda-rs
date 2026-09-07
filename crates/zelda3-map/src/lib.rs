//! Canonical dungeon content. No ROM offsets, filesystem access, packed asset
//! reader or engine dependency. Ordered vectors preserve execution order;
//! named resources express sharing explicitly.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const FORMAT: &str = "zelda3_dungeon_world_v1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DungeonWorld {
    pub format: String,
    pub rooms: Vec<RoomDefinition>,
    pub headers: BTreeMap<String, RoomSettings>,
    pub programs: BTreeMap<String, RoomProgram>,
    pub actors: BTreeMap<String, ActorProgram>,
    pub door_references: BTreeMap<String, Vec<Door>>,
    pub default_layouts: Vec<String>,
    pub overlays: Vec<String>,
    pub overlay_programs: BTreeMap<String, Vec<OverlayObject>>,
    pub object_lists: BTreeMap<String, ObjectPass>,
    pub secrets: BTreeMap<String, Vec<Secret>>,
    pub entrances: Vec<Entrance>,
    pub starting_points: Vec<StartingPoint>,
    pub chests: Vec<Chest>,
    pub damaging_pit_rooms: Vec<u16>,
    pub attribute_themes: Vec<String>,
    pub attributes: BTreeMap<String, Vec<u8>>,
    pub movable_blocks: Vec<MovableBlock>,
    pub torches: Vec<TorchRoom>,
    /// These words are copied by the original loader after the torch table.
    /// Their meaning is not inferred; retain their order and all bits.
    pub torch_tail_words: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomDefinition {
    pub id: u16,
    pub header: String,
    pub program: String,
    pub actors: String,
    /// Adjacent-room door lookups can use a suffix of a room program.
    pub door_reference: String,
    pub secrets: String,
    pub telepathy_message: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomSettings {
    pub background_mode: u8,
    pub collision_mode: u8,
    pub dark: bool,
    pub reserved_flag: bool,
    pub palette: u8,
    pub tile_theme: u8,
    pub sprite_graphics: u8,
    pub collision_effect: u8,
    pub tags: [u8; 2],
    /// Five 2-bit travel planes. The upper six bits of the second source byte
    /// are retained separately, since the room loader copies that byte whole.
    pub travel_planes: [u8; 5],
    pub travel_plane_reserved: u8,
    pub travel_destinations: [u8; 5],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomProgram {
    pub floor_low: u8,
    pub floor_high: u8,
    pub default_layout: u8,
    pub starting_quadrant: u8,
    pub passes: [ObjectPass; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectPass {
    pub objects: Vec<RoomObject>,
    /// None and Some(empty) have distinct legacy stream representations.
    pub doors: Option<Vec<Door>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RoomObject {
    #[serde(rename = "type1")]
    Type1 {
        id: u8,
        x: u8,
        y: u8,
        width_bits: u8,
        height_bits: u8,
    },
    #[serde(rename = "type2")]
    Type2 { id: u8, x: u8, y: u8 },
    #[serde(rename = "type3")]
    Type3 { id: u8, x: u8, y: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Door {
    pub direction: Direction,
    pub slot: u8,
    pub door_type: u8,
    pub reserved_bits: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorProgram {
    pub sorting: u8,
    pub records: Vec<ActorRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ActorRecord {
    Actor {
        id: u8,
        x: u8,
        y: u8,
        x_flags: u8,
        y_flags: u8,
    },
    Control {
        command: u8,
        argument: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entrance {
    pub room: u16,
    pub relative_coords: [u8; 8],
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub player_x: u16,
    pub player_y: u16,
    pub camera_x: u16,
    pub camera_y: u16,
    pub blockset: u8,
    pub floor: i8,
    pub palace: i8,
    pub doorway_orientation: u8,
    pub starting_bg: u8,
    pub quadrant1: u8,
    pub quadrant2: u8,
    pub door_settings: u16,
    pub music_track: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartingPoint {
    pub entrance: u8,
    pub spawn: Entrance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chest {
    pub room: u16,
    pub big: bool,
    pub item: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Secret {
    pub position_word: u16,
    pub item: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovableBlock {
    pub room: u16,
    pub tilemap_position: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TorchRoom {
    pub room: u16,
    pub positions: Vec<u16>,
}

/// Overlay commands use their own decoder, not the three room-object families.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayObject {
    pub x: u8,
    pub y: u8,
    pub command: u8,
    pub unused_x_bits: u8,
    pub unused_y_bits: u8,
}

impl DungeonWorld {
    /// Validate content without consulting legacy placements or an engine.
    /// Allocation sizes and byte-sharing conflicts belong to the compatibility
    /// exporter, not to this portable domain model.
    pub fn validate(&self) -> Result<(), String> {
        let require = |ok: bool, message: &str| {
            if ok {
                Ok(())
            } else {
                Err(message.to_owned())
            }
        };
        require(self.format == FORMAT, "unsupported dungeon world format")?;
        let ids: std::collections::BTreeSet<_> = self.rooms.iter().map(|r| r.id).collect();
        require(
            self.rooms.len() == 320 && ids.iter().copied().eq(0..320),
            "room IDs must contain 0..319 exactly once",
        )?;
        for r in &self.rooms {
            require(
                self.headers.contains_key(&r.header),
                "room references missing header",
            )?;
            require(
                self.programs.contains_key(&r.program),
                "room references missing program",
            )?;
            require(
                self.actors.contains_key(&r.actors),
                "room references missing actors",
            )?;
            require(
                self.door_references.contains_key(&r.door_reference),
                "room references missing door resource",
            )?;
            require(
                self.secrets.contains_key(&r.secrets),
                "room references missing secrets",
            )?;
        }
        for header in self.headers.values() {
            require(
                header.background_mode < 8
                    && header.collision_mode < 8
                    && header.travel_plane_reserved < 64
                    && header.travel_planes.iter().all(|p| *p < 4),
                "header fields out of range",
            )?;
        }
        for p in self.programs.values() {
            require(
                p.floor_low < 16
                    && p.floor_high < 16
                    && p.starting_quadrant < 4
                    && p.default_layout < 64
                    && (p.default_layout as usize) < self.default_layouts.len(),
                "room prefix/default layout out of range",
            )?;
        }
        for id in &self.default_layouts {
            require(
                self.object_lists.contains_key(id),
                "missing inherited object list",
            )?;
        }
        require(
            self.overlays
                .iter()
                .all(|id| self.overlay_programs.contains_key(id)),
            "missing overlay program",
        )?;
        require(
            self.overlay_programs.values().flatten().all(|o| {
                o.x < 64
                    && o.y < 64
                    && o.unused_x_bits < 4
                    && o.unused_y_bits < 4
                    && !(o.x == 63 && o.y == 63 && o.unused_x_bits == 3 && o.unused_y_bits == 3)
            }),
            "invalid overlay command",
        )?;
        for pass in self
            .programs
            .values()
            .flat_map(|p| p.passes.iter())
            .chain(self.object_lists.values())
        {
            for object in &pass.objects {
                let valid = match *object {
                    RoomObject::Type1 {
                        id,
                        x,
                        y,
                        width_bits,
                        height_bits,
                    } => {
                        id < 248
                            && x < 63
                            && y < 64
                            && width_bits < 4
                            && height_bits < 4
                            && !(x == 60 && y == 63 && width_bits == 0 && height_bits == 3)
                    }
                    RoomObject::Type2 { id, x, y } => {
                        id < 64 && x < 64 && y < 64 && !(x == 63 && y >= 60)
                    }
                    RoomObject::Type3 { id, x, y } => {
                        id < 128 && x < 63 && y < 64 && !(x == 60 && y == 63 && id & 15 == 12)
                    }
                };
                require(
                    valid,
                    "room object fields out of range or encode a stream terminator",
                )?;
            }
        }
        for door in self.door_references.values().flatten().chain(
            self.programs
                .values()
                .flat_map(|p| p.passes.iter())
                .chain(self.object_lists.values())
                .filter_map(|p| p.doors.as_ref())
                .flatten(),
        ) {
            require(
                door.slot < 16
                    && door.reserved_bits < 4
                    && !(door.door_type == 255
                        && door.slot == 15
                        && door.reserved_bits == 3
                        && door.direction == Direction::East),
                "door fields out of range or encode a terminator",
            )?;
        }
        for record in self.actors.values().flat_map(|p| p.records.iter()) {
            match *record {
                ActorRecord::Control { command, .. } => require(
                    command == 0xfd || command == 0xfe,
                    "unknown actor control command",
                )?,
                ActorRecord::Actor {
                    id,
                    x,
                    y,
                    x_flags,
                    y_flags,
                } => {
                    require(
                        x < 32 && y < 32 && x_flags % 32 == 0 && y_flags % 32 == 0,
                        "actor position/flags out of range",
                    )?;
                    require(
                        y | y_flags != 255
                            && !(id == 0xe4 && [0xfd, 0xfe].contains(&(y | y_flags))),
                        "actor encodes a control or terminator",
                    )?;
                }
            }
        }
        require(
            self.entrances
                .iter()
                .chain(self.starting_points.iter().map(|p| &p.spawn))
                .all(|e| e.room < 320),
            "entrance room out of range",
        )?;
        require(
            self.starting_points
                .iter()
                .all(|p| (p.entrance as usize) < self.entrances.len()),
            "starting point references missing entrance",
        )?;
        require(
            self.chests.iter().all(|c| c.room < 320)
                && self.damaging_pit_rooms.iter().all(|r| *r < 320),
            "reward/pit room out of range",
        )?;
        require(
            self.secrets
                .values()
                .flatten()
                .all(|s| s.position_word < 0x8000),
            "secret position has reserved high bit",
        )?;
        require(
            self.attribute_themes
                .iter()
                .all(|id| self.attributes.contains_key(id))
                && self.attributes.values().all(|a| a.len() == 128),
            "missing or malformed attribute resource",
        )?;
        require(
            self.movable_blocks.iter().all(|b| b.room < 320),
            "movable block room out of range",
        )?;
        require(
            self.torches.iter().all(|t| {
                (t.room < 320 || (t.room == 0xffff && t.positions.is_empty()))
                    && t.positions.iter().all(|p| *p != 0xffff)
            }),
            "invalid torch record",
        )
    }
}
