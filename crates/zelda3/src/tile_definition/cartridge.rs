//! Cartridge decoding, evaluated once for the native definition catalog.
use super::{TileBehavior, TileResult};

impl super::TilePair {
    pub(crate) const fn import_table<const N: usize>(words: [u16; N]) -> [Self; N] {
        let mut tiles = [Self::from_cartridge(0); N];
        let mut index = 0;
        while index < N {
            tiles[index] = Self::from_cartridge(words[index]);
            index += 1;
        }
        tiles
    }

    pub(crate) const fn from_cartridge(word: u16) -> Self {
        Self(super::NativeTile::import_pair(word))
    }

    fn cartridge_word(self) -> u16 {
        u16::from_le_bytes(self.0.map(super::NativeTile::cartridge_attribute))
    }

    pub(crate) fn next_identity(self) -> Self {
        Self(super::NativeTile::next_pair_identity(self.0))
    }

    pub(crate) fn descending_stair_sequence(self) -> Self {
        Self::from_cartridge((self.cartridge_word() & 0x0707) | 0x3434)
    }

    pub(crate) fn next_torch(self) -> Self {
        Self::from_cartridge((self.cartridge_word() & 0xefef).wrapping_add(0x0101))
    }

    pub(crate) fn closed_door(slot: usize) -> Self {
        Self::from_cartridge((0xf0u16.wrapping_add(slot as u16)).wrapping_mul(0x0101))
    }

    pub(crate) fn with_floor_transition(self) -> Self {
        Self::from_cartridge(self.cartridge_word() | 0x1010)
    }

    pub(crate) fn with_palace_transition(self) -> Self {
        Self::from_cartridge(self.cartridge_word() | 0x2020)
    }
}

impl super::NativeTile {
    /// Map tile flip bits extend the slope identity. Indoor imports pass both
    /// flip bits; the outdoor catalog supplies only the horizontal bit.
    pub(crate) const fn with_cartridge_orientation(self, flips: u8) -> Self {
        let attribute = self.cartridge_attribute();
        if attribute >= 0x10 && attribute < 0x1c {
            Self::from_cartridge(attribute | flips)
        } else {
            self
        }
    }
}

const fn surface(result: TileResult, shift: u32) -> TileBehavior {
    TileBehavior::Surface { result, shift }
}

impl TileBehavior {
    pub(super) const fn decode(attribute: u8, indoors: bool) -> Self {
        use TileResult as R;
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
            0x0e..=0x0f => surface(R::Ice, ((attribute & 1) as u32) * 4),
            0x10..=0x13 | 0x18..=0x1b => Self::Slope {
                diagonal: attribute & 8 != 0,
                shape: ((attribute & 3) as usize),
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
                shift: ((attribute & 1) as u32) * 4,
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
                        shift: ((attribute & 2) as u32) * 2,
                    }
                }
            }
            // Original liftable ordering: 54, 52, 50, 51, 53, 55, 56.
            0x50..=0x56 => Self::Liftable {
                index: [4, 6, 2, 8, 0, 10, 12][(attribute - 0x50) as usize],
                dashable: attribute <= 0x51,
            },
            0x57 => Self::DashableSolid,
            0x58..=0x5d => Self::Chest {
                index: Some((attribute - 0x58) as usize),
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
                shift: ((attribute & 3) as u32) * 4,
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
                direction: 2 * ((attribute & 1) as u16),
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
                direction: 2 * ((attribute & 1) as u16),
                forced_movement: true,
                transition: Some(if attribute < 0x98 { 1 } else { 3 }),
                dashable: false,
            },
            0xa0..=0xa5 => Self::Door {
                direction: 2 * ((attribute & 1) as u16),
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

impl super::NativeTile {
    pub(crate) fn toggled_crystal_peg(self) -> Option<Self> {
        match self.cartridge_attribute() {
            0x66 => Some(Self::from_cartridge(0x67)),
            0x67 => Some(Self::from_cartridge(0x66)),
            _ => None,
        }
    }

    pub(crate) fn next_pair_identity(pair: [Self; 2]) -> [Self; 2] {
        let word = u16::from_le_bytes(pair.map(Self::cartridge_attribute));
        // Preserve the original word carry, even across a low-byte wrap.
        Self::import_pair(word.wrapping_add(0x0101))
    }
}
