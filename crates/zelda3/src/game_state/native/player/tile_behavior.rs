//! Attribute decoding and native probe-result accumulation. Publication stays
//! at the tile-detection bridge; no additional result state is stored here.

use super::{NativeTileDetectionBridgeMut, TileDetectionState};

#[derive(Clone, Copy, Debug)]
pub(crate) enum TileResult {
    Collision,
    Normal,
    DeepWater,
    ShallowWater,
    MovingFloor,
    Ice,
    WaterStaircase,
    Grass,
    Stair,
    InRoomStaircase,
    Pit,
    Misc,
    Slope,
    Diagonal,
    VerticalLedge,
    HorizontalLedge,
    DownwardSideLedge,
    DiagonalLedge,
    SpikeTrigger,
    SpikeCactus,
    Aftermath,
    Readable,
    Dashable,
    Chest,
    KeyLockGravestone,
    Block,
}

#[derive(Clone, Copy, Debug)]
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

impl TileBehavior {
    pub(crate) fn decode(attribute: u8, indoors: bool) -> Self {
        use TileResult as R;
        let surface = |result, shift| Self::Surface { result, shift };
        let ground = if indoors {
            Self::Ignore
        } else {
            surface(R::Normal, 0)
        };
        match attribute {
            0x00
            | 0x05..=0x07
            | 0x14..=0x17
            | 0x21
            | 0x23..=0x25
            | 0x38..=0x3c
            | 0x41
            | 0x45
            | 0x47
            | 0x49
            | 0x5e..=0x5f
            | 0x61..=0x62
            | 0x64..=0x66
            | 0xa6..=0xa7
            | 0xbe..=0xbf
            | 0xd0..=0xef => ground,
            0x01..=0x03 | 0x26 | 0x43 => Self::Solid,
            0x04 => {
                if indoors {
                    Self::Solid
                } else {
                    surface(R::Grass, 0)
                }
            }
            0x08 => surface(R::DeepWater, 0),
            0x09 => surface(R::ShallowWater, 0),
            0x0a => surface(R::Normal, 0),
            0x0b => {
                if indoors {
                    Self::Solid
                } else {
                    Self::DeepWaterEdge
                }
            }
            0x0c => surface(R::MovingFloor, 0),
            0x0d => Self::ConditionalFloorTrigger,
            0x0e..=0x0f => surface(R::Ice, u32::from(attribute & 1) * 4),
            0x10..=0x13 | 0x18..=0x1b => Self::Slope {
                diagonal: attribute & 8 != 0,
                shape: usize::from(attribute & 3),
            },
            0x1c => surface(R::WaterStaircase, 0),
            0x1d..=0x1f => Self::InRoomStaircase { shift: 0 },
            0x20 | 0xb0..=0xbd => Self::Pit,
            0x22 | 0x30..=0x37 => surface(R::Stair, 0),
            0x27 | 0xc0..=0xcf => Self::SolidInteractable {
                misc_shift: 0,
                cactus: false,
            },
            0x28..=0x29 => Self::Ledge {
                result: R::VerticalLedge,
                shift: u32::from(attribute & 1) * 4,
            },
            0x2a..=0x2b => Self::Ledge {
                result: R::HorizontalLedge,
                shift: 0,
            },
            0x2c | 0x2e => Self::Ledge {
                result: R::HorizontalLedge,
                shift: 4,
            },
            0x2d | 0x2f => Self::Ledge {
                result: R::DownwardSideLedge,
                shift: 0,
            },
            0x3d..=0x3f => Self::InRoomStaircase { shift: 4 },
            0x40 => surface(R::Grass, 0),
            0x42 => {
                if indoors {
                    Self::Ignore
                } else {
                    Self::Gravestone
                }
            }
            0x44 => Self::Cactus,
            0x46 => Self::SpikedSolid,
            0x48 | 0x4a => Self::Aftermath,
            0x4b => surface(R::Grass, 4),
            0x4c..=0x4f => {
                if indoors {
                    Self::Ignore
                } else {
                    Self::Ledge {
                        result: R::DiagonalLedge,
                        shift: u32::from(attribute & 2) * 2,
                    }
                }
            }
            // Original liftable ordering: 54, 52, 50, 51, 53, 55, 56.
            0x50..=0x56 => Self::Liftable {
                index: [4, 6, 2, 8, 0, 10, 12][usize::from(attribute - 0x50)],
                dashable: attribute <= 0x51,
            },
            0x57 => Self::DashableSolid,
            0x58..=0x5d => Self::Chest {
                index: Some(usize::from(attribute - 0x58)),
            },
            0x60 => {
                if indoors {
                    Self::NeighborDependent
                } else {
                    surface(R::Normal, 0)
                }
            }
            0x63 => Self::Chest { index: None },
            0x67 => Self::SolidInteractable {
                misc_shift: 0,
                cactus: true,
            },
            0x68..=0x6b => Self::MovingFloorCheck {
                shift: u32::from(attribute & 3) * 4,
            },
            0x6c..=0x6f => {
                if indoors {
                    Self::Solid
                } else {
                    surface(R::Normal, 0)
                }
            }
            0x70..=0x7f => Self::PushBlock {
                index: attribute & 15,
            },
            0x80..=0x8d => Self::Door {
                direction: 2 * u16::from(attribute & 1),
                forced_movement: matches!(attribute, 0x82 | 0x83),
                transition: None,
                dashable: false,
            },
            0x8e..=0x8f => Self::Door {
                direction: 0,
                forced_movement: false,
                transition: None,
                dashable: true,
            },
            0x90..=0x9f | 0xa8..=0xaf => Self::Door {
                direction: 2 * u16::from(attribute & 1),
                forced_movement: true,
                transition: Some(if attribute < 0x98 { 1 } else { 3 }),
                dashable: false,
            },
            0xa0..=0xa5 => Self::Door {
                direction: 2 * u16::from(attribute & 1),
                forced_movement: matches!(attribute, 0xa2 | 0xa3),
                transition: Some(2),
                dashable: false,
            },
            0xf0..=0xff => Self::SolidInteractable {
                misc_shift: 4,
                cactus: false,
            },
        }
    }
}

impl TileDetectionState {
    fn accumulate_result(&mut self, result: TileResult, bits: u16) {
        use TileResult::*;
        match result {
            Collision => self.collision_bits |= bits,
            Normal => self.normal_tiles |= bits,
            DeepWater => self.deepwater |= bits,
            ShallowWater => self.shallow_water |= bits,
            MovingFloor => self.moving_floor_tiles |= bits,
            Ice => self.icy_floor |= bits,
            WaterStaircase => self.water_staircase |= bits,
            Grass => self.thick_grass |= bits,
            Stair => self.stair_tile |= bits as u8,
            InRoomStaircase => self.inroom_staircase |= bits,
            Pit => self.or_pit_tile(bits as u8),
            Misc => self.misc_tiles |= bits,
            Slope => self.slope_collision_bits |= bits,
            Diagonal => self.diagonal_tile |= bits,
            VerticalLedge => self.vertical_ledge |= bits as u8,
            HorizontalLedge => self.horizontal_ledge |= bits as u8,
            DownwardSideLedge => self.ledges_down_leftright |= bits as u8,
            DiagonalLedge => self.diagonal_ledge_tiles |= bits as u8,
            SpikeTrigger => self.spike_floor_and_triggers |= bits as u8,
            SpikeCactus => self.or_spike_cactus_tiles(bits as u8),
            Aftermath => self.destruction_aftermath |= bits,
            Readable => self.read_something |= bits,
            Dashable => self.dashable_tiles |= bits as u8,
            Chest => self.chest |= bits,
            KeyLockGravestone => self.or_key_lock_gravestones(bits as u8),
            Block => self.block_flags |= bits,
        }
    }
}

impl NativeTileDetectionBridgeMut<'_> {
    pub(crate) fn accumulate_result(&mut self, result: TileResult, bits: u16) {
        self.state.accumulate_result(result, bits);
        self.sync();
    }

    pub(crate) fn reset_probe_results(&mut self) {
        // Retain each intermediate projection, including the shared key-lock/spike
        // word's two clears. Scratch positions and interaction identity survive.
        self.clear_slope_collision_bits();
        self.clear_collision_bits();
        self.clear_diagonal_tile();
        self.clear_stair_tile();
        self.clear_pit_tile();
        self.clear_inroom_staircase();
        self.clear_block_flags();
        self.clear_door_direction_flags();
        self.clear_moving_floor_tiles();
        self.clear_deepwater();
        self.clear_normal_tiles();
        self.clear_icy_floor();
        self.clear_water_staircase();
        self.clear_thick_grass();
        self.clear_shallow_water();
        self.clear_destruction_aftermath();
        self.clear_read_something();
        self.clear_vertical_ledge();
        self.clear_horizontal_ledge();
        self.clear_ledges_down_leftright();
        self.clear_diagonal_ledge_tiles();
        self.clear_chest();
        self.clear_key_lock_gravestones();
        self.clear_spike_cactus_tiles();
        self.clear_spike_floor_and_triggers();
        self.clear_dashable_tiles();
        self.clear_misc_tiles();
    }
}
