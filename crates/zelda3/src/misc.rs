// Methods ported from zelda3/src/misc.c and included inside ZeldaState.

use super::*;
use crate::game_state::{DungeonItem, EquipmentItem};

#[path = "item_awards.rs"]
mod item_awards;
pub(super) use item_awards::chest_item_alternate;

#[cfg(test)]
#[path = "item_award_tests.rs"]
mod item_award_tests;
#[cfg(test)]
#[path = "misc_sprite_dma_tests.rs"]
mod sprite_dma_tests;

/// One owning call through all four portions of NMI_PrepareSprites. Phase
/// changes add no instructions and cannot discard a child's loaded operand.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(super) enum SpritePreparationWork {
    Packing(crate::cycle_models::oam_pack::ExtendedOamPacking),
    Sources(crate::cycle_models::sprite_dma::SourceWords),
    Animations(crate::cycle_models::sprite_dma::Animations),
    Pointers(crate::cycle_models::sprite_dma::PointerWords),
    Complete,
}

impl Default for SpritePreparationWork {
    fn default() -> Self { Self::Packing(Default::default()) }
}

impl SpritePreparationWork {
    pub(super) fn is_complete(self) -> bool { matches!(self, Self::Complete) }

    #[cfg(test)]
    fn next_pc(self) -> u32 {
        match self {
            Self::Packing(work) => work.next_pc(),
            Self::Sources(work) => work.next_pc(),
            Self::Animations(work) => work.next_pc(),
            Self::Pointers(work) => work.next_pc(),
            Self::Complete => 0x805d, // actual main caller, after its JSR
        }
    }
}
// Runtime source pointers selected by the ROM's LoadSongBank entry points at
// $80:8913, $80:8925, and $80:8931. The intro entry at $80:8901 also uploads
// the resident driver and samples; it is not the runtime overworld bank.
// The extracted sound-bank assets describe
// the final SPC RAM image and may normalize block order or trailing zeroes;
// the CPU<->SPC upload protocol must retain the original stream byte-for-byte
// because block lengths and order determine its acknowledgment timing.
const SONG_BANK_UPLOAD_STREAM_SNES_ADDRS: [u32; 3] = [0x1a_9ef5, 0x1b_8000, 0x1a_d380];

#[test]
#[ignore = "requires the local pinned Zelda ROM"]
fn runtime_song_bank_streams_follow_the_rom_loader_pointers() {
    let rom = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
    )
    .expect("read the local pinned Zelda ROM");
    for (asset, entry) in [0x8913usize, 0x8925, 0x8931].into_iter().enumerate() {
        // Each source loader writes its 24-bit stream pointer into $00-$02.
        let code = &rom[entry & 0x7fff..];
        assert_eq!(&code[..4], &[0xa9, code[1], 0x85, 0x00]);
        assert_eq!(&code[4..8], &[0xa9, code[5], 0x85, 0x01]);
        assert_eq!(code[8], 0xa9);
        let pointer = u32::from(code[1]) | (u32::from(code[5]) << 8) | (u32::from(code[9]) << 16);
        assert_eq!(SONG_BANK_UPLOAD_STREAM_SNES_ADDRS[asset], pointer);
        let offset = ((pointer >> 16) as usize) * 0x8000 + (pointer as usize & 0x7fff);
        // Runtime streams update music data, not the resident SPC program.
        assert_eq!(&rom[offset + 2..offset + 4], &[0x00, 0xd0]);
    }
}

// Static lookup tables ported from misc.c.
const RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
    0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
];
const RECEIVE_ITEM_DRAW_Y_OFFSETS: [i8; 76] = [
    -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4, -4, -4,
    -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4, -2, -4, -4,
    -4, -5, -4, -4,
];
const RECEIVE_ITEM_PALETTE_BITS: [u8; 76] = [
    4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
];
const RECEIVE_ITEM_GRAPHICS: [u8; 76] = [
    6, 0x18, 0x18, 0x18, 0x2d, 0x20, 0x2e, 9, 9, 0x0a, 8, 5, 0x10, 0x0b, 0x2c, 0x1b, 0x1a, 0x1c,
    0x14, 0x19, 0x0c, 7, 0x1d, 0x2f, 7, 0x15, 0x12, 0x0d, 0x0d, 0x0e, 0x11, 0x17, 0x28, 0x27, 4, 4,
    0x0f, 0x16, 3, 0x13, 1, 0x1e, 0x10, 0, 0, 0, 0, 0, 0, 0x30, 0x22, 0x21, 0x24, 0x24, 0x24, 0x23,
    0x23, 0x23, 0x29, 0x2a, 0x2c, 0x2b, 3, 3, 0x34, 0x35, 0x31, 0x33, 2, 0x32, 0x36, 0x37, 0x2c, 6,
    0x0c, 0x38,
];
const LIT_TORCH_COLOR_PLUS_BY_COUNT: [u8; 4] = [31, 8, 4, 0];
pub(crate) fn native_item_receipt_table_byte(address: u32) -> Option<u8> {
    match address {
        0x09836c..=0x0983b7 => Some(RECEIVE_ITEM_DRAW_Y_OFFSETS[(address - 0x09836c) as usize] as u8),
        0x0983b8..=0x098403 => Some(RECEIVE_ITEM_PALETTE_BITS[(address - 0x0983b8) as usize]),
        0x098404..=0x09844f => Some(RECEIVE_ITEM_GRAPHICS[(address - 0x098404) as usize]),
        0x098450..=0x09849b => Some(RECEIVE_ITEM_OAM_EXT_SIZES[(address - 0x098450) as usize]),
        0x0dbb5b..=0x0dbb5d => Some([0, 0x80, 0x40][(address - 0x0dbb5b) as usize]),
        _ => None,
    }
}
const ITEM_RECEIPT_BOTTLED_ITEM_IDS: [u8; 7] = [0x16, 0x2b, 0x2c, 0x2d, 0x3d, 0x3c, 0x48];
const ITEM_RECEIPT_POTION_ITEM_IDS: [u8; 5] = [0x2e, 0x2f, 0x30, 0xff, 0x0e];
const ARBITRARY_SFX_PAN_BY_SCREEN_COLUMN: [u8; 8] = [0x80, 0x80, 0x80, 0, 0, 0x40, 0x40, 0x40];
const LINK_BODY_DMA_BASE_SOURCES: [u16; 303] = [
    0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8000, 0x9440, 0x8080, 0x8080, 0x8080, 0x9400, 0x8040, 0x80c0, 0x80c0,
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x8080, 0x8080,
    0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0, 0xa900,
    0x9100, 0x8080, 0x8080, 0x90c0, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x9a00,
    0x9140, 0x9180, 0x8000, 0x9500, 0x9480, 0x94c0, 0x94c0, 0x9ae0, 0x8080, 0x8080, 0x9a60, 0x80c0,
    0x80c0, 0x9aa0, 0x8000, 0x8000, 0x9aa0, 0x8000, 0x8000, 0x8080, 0x8080, 0x8100, 0x8100, 0x85c0,
    0x8000, 0x8000, 0x85c0, 0x8000, 0x8000, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xadc0, 0xad40, 0xad40,
    0xad40, 0xad40, 0xad40, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0xad80, 0x8040, 0x9400, 0x8040,
    0x8000, 0x8080, 0x8080, 0x9440, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8040, 0x8040, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0xc440, 0x8140, 0x8140, 0xca40, 0x8000, 0x8000, 0x8000,
    0x8000, 0x8000, 0x8000, 0x8040, 0x85c0, 0x8040, 0x85c0, 0x8100, 0x80c0, 0x91c0, 0x8080, 0x8080,
    0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000, 0x8080, 0x8080, 0x9100, 0xa0c0, 0xa100, 0xa100,
    0xa1c0, 0xa400, 0xa440, 0xa1c0, 0xa400, 0xa440, 0x8080, 0xc480, 0x8080, 0x8040, 0x8040, 0xca80,
    0xca80, 0xca00, 0xc400, 0xca00, 0xc400, 0x81c0, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080, 0x8080,
    0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0xa8c0, 0xa900,
    0x8000, 0x8000, 0xa8c0, 0xa900, 0x8000, 0xa8c0, 0xa900, 0x8000, 0x8000, 0xa8c0, 0xa900, 0x8040,
    0x8040, 0x8040, 0x8080, 0x8080, 0x8040, 0x8040, 0x8040, 0x8040, 0x8000, 0x8000, 0x8000, 0x8000,
    0xd080, 0x8080, 0x90c0, 0xd000, 0x9080, 0xd040, 0x9080, 0xd040, 0xd080, 0xd080, 0xd080, 0xd080,
    0xd080, 0xd000, 0xd000, 0xd000, 0xd000, 0xd000, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040, 0xd040,
    0x8040, 0xd000, 0x85c0, 0x85c0, 0x85c0, 0xdc40, 0xdc40, 0xdc40, 0x85c0, 0x85c0, 0x85c0, 0xdc40,
    0xdc40, 0xdc40, 0xe1c0, 0xd000, 0x8000, 0xe400, 0xe400, 0xe440, 0x90c0, 0x90c0, 0xd000, 0x8000,
    0x8000, 0xd040, 0x8000, 0x8000, 0xd040, 0xe400, 0xe400, 0xe400, 0x9080, 0xa5c0, 0xac40, 0xe480,
    0x8180, 0x90c0, 0x80c0, 0xe180, 0xd000, 0xe4c0, 0xe4c0, 0xe840, 0xe840, 0xe840, 0xe540, 0xe540,
    0xe540, 0xe900, 0xe900, 0xe900, 0xe900, 0x8080, 0x8080, 0x8000, 0xa9c0, 0x8080, 0x8140, 0x91c0,
    0x8040, 0xa800, 0xa840,
];
const LINK_HEAD_DMA_BASE_SOURCES: [u16; 303] = [
    0x8840, 0x8800, 0x8580, 0x8800, 0x8580, 0x84c0, 0x8500, 0x8540, 0x8500, 0x8540, 0x8400, 0x8440,
    0x8480, 0x8400, 0x8440, 0x8480, 0x9640, 0x8c40, 0x8c80, 0xad00, 0x9600, 0x8980, 0x8c00, 0xacc0,
    0x8880, 0x88c0, 0x8900, 0x8940, 0x8880, 0x88c0, 0x8900, 0x8940, 0xb0c0, 0xb100, 0xb140, 0xb100,
    0xb140, 0xb000, 0xb040, 0xb080, 0xec80, 0xecc0, 0xb180, 0xd440, 0xb1c0, 0xb180, 0xd440, 0xb1c0,
    0x8c80, 0xad00, 0x95c0, 0x99c0, 0xb440, 0x9580, 0xb480, 0xb4c0, 0x9580, 0xb480, 0xb4c0, 0x9c20,
    0x8000, 0x8000, 0x8000, 0x9700, 0x9680, 0x96c0, 0x96c0, 0x9ce0, 0x8c80, 0xb540, 0x9c60, 0xb580,
    0x8c00, 0x9ca0, 0x8900, 0xb500, 0x9ca0, 0x8900, 0xb500, 0x8c40, 0xec40, 0x8c00, 0xec00, 0x8dc0,
    0x9540, 0x89c0, 0x8dc0, 0x9540, 0x89c0, 0xb940, 0xb980, 0xb9c0, 0xb980, 0xb9c0, 0xb5c0, 0xb800,
    0xb840, 0xb800, 0xb840, 0xb880, 0xb8c0, 0xb900, 0xb880, 0xb8c0, 0xb900, 0x8980, 0x9600, 0xbcc0,
    0x8400, 0xbc80, 0x8c40, 0x9640, 0xa040, 0xa080, 0xa000, 0xbc40, 0xbd40, 0x8500, 0xbd00, 0xbd80,
    0xbd80, 0x88c0, 0x8900, 0xe9c0, 0x8900, 0xc640, 0xc040, 0xc000, 0xcc40, 0x8940, 0x88c0, 0x8900,
    0xe9c0, 0x8900, 0x8940, 0x8d40, 0x8d80, 0x8d40, 0x8d80, 0xbd00, 0xb000, 0xb000, 0xa480, 0xa480,
    0xa480, 0xa480, 0xac00, 0xac00, 0xac00, 0xac00, 0xa140, 0xa180, 0xa180, 0xa4c0, 0xa4c0, 0xa500,
    0x9d40, 0x9d80, 0x9dc0, 0x9d40, 0x9d80, 0x9dc0, 0x8d00, 0xc680, 0xc180, 0xc140, 0x8c00, 0xcc80,
    0xcc80, 0xcc00, 0xc600, 0xcc00, 0xc600, 0xbd00, 0x8580, 0x8800, 0xc9c0, 0xccc0, 0xcdc0, 0xcd00,
    0xcd40, 0xcd80, 0x8500, 0x8540, 0xc940, 0xc980, 0x8540, 0xc940, 0xc980, 0x8440, 0x8480, 0xc1c0,
    0xc900, 0xc580, 0xc5c0, 0xc8c0, 0x8440, 0x8480, 0xc1c0, 0xc900, 0xc580, 0xc5c0, 0xc8c0, 0xbd00,
    0xacc0, 0xc040, 0xd540, 0xd580, 0xd4c0, 0xd500, 0xd4c0, 0xd500, 0xd440, 0xd480, 0xd440, 0xd480,
    0xd1c0, 0xd400, 0xd100, 0xd100, 0xd140, 0xd180, 0xd140, 0xd180, 0xb0c0, 0xb100, 0xb140, 0xb100,
    0xb140, 0xdd40, 0xdd80, 0xddc0, 0xdd80, 0xddc0, 0xdc80, 0xdcc0, 0xdd00, 0xdc80, 0xdcc0, 0xdd00,
    0xd100, 0xd100, 0xe000, 0xe040, 0xe080, 0xe0c0, 0xe100, 0xe140, 0xe000, 0xe040, 0xe080, 0xe0c0,
    0xe100, 0xe140, 0x8000, 0xd0c0, 0x8000, 0xb940, 0xb980, 0xb940, 0xdd40, 0xdd80, 0xdd40, 0xdc80,
    0xdcc0, 0xc0c0, 0xdc80, 0xdcc0, 0xc0c0, 0xb9c0, 0xb980, 0xb9c0, 0xa560, 0xa5a0, 0xac80, 0xed00,
    0x8000, 0x8cc0, 0xbd00, 0xe380, 0xbdc0, 0xe500, 0xe500, 0xe880, 0xe8c0, 0xe8c0, 0xe800, 0xe5c0,
    0xe5c0, 0xe940, 0xe980, 0xe940, 0xe980, 0xbd40, 0x8c80, 0xa080, 0x8000, 0xa980, 0xbd00, 0xbdc0,
    0xb400, 0xa880, 0xedc0,
];
const LINK_HAND_DMA_BASE_SOURCES: [u16; 27] = [
    0x9a40, 0x9e00, 0x9d20, 0x9f20, 0x9b20, 0xbc20, 0xbc20, 0xbe20, 0xbe20, 0xbe00, 0xbe00, 0xbe00,
    0xbe00, 0xa540, 0xa540, 0xa540, 0xa540, 0xbc00, 0xbc00, 0xbc00, 0xbc00, 0xa740, 0xa740, 0xa740,
    0xa740, 0xe780, 0xe780,
];
const LINK_SWORD_DMA_BASE_SOURCES: [u16; 8] = [
    0x9000, 0x9020, 0x9060, 0x91e0, 0x90a0, 0x90c0, 0x9100, 0x9140,
];
// The final shield word is also the first auxiliary-table word.
const LINK_SHIELD_DMA_BASE_SOURCES: [u16; 4] = [0x9300, 0x9340, 0x9380, 0x9480];
const LINK_AUX_DMA_BASE_SOURCES: [u16; 128] = [
    0x9480, 0x94c0, 0x94e0, 0x95c0, 0x9500, 0x9520, 0x9540, 0x9480, 0x9640, 0x9680, 0x96a0, 0x9780,
    0x96c0, 0x96e0, 0x9700, 0x9480, 0x9800, 0x9840, 0x98a0, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9ac0, 0x9b00, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9bc0, 0x9c00, 0x9c40, 0x9c80,
    0x9cc0, 0x9d00, 0x9d40, 0x9480, 0x9f40, 0x9f80, 0x9fc0, 0x9fe0, 0xa000, 0x9480, 0x9480, 0x9480,
    0xa100, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x98c0, 0x9900, 0x99c0, 0x99e0, 0x9a00, 0x9a20, 0x9a40, 0x9a60,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9a80, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
    0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480, 0x9480,
];
const LINK_AUX_DMA_GROUP_OFFSETS: [u16; 16] = [
    0x00e0, 0x00e0, 0x0060, 0x0080, 0x01c0, 0x00e0, 0x0040, 0x0000, 0x0080, 0x0000, 0x0040, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000,
];
const LINK_PUSH_BLOCK_DMA_BASE_SOURCES: [u16; 4] = [0xa480, 0xa4c0, 0xa500, 0xa540];
const LINK_ANIMATED_TILE_DMA_COUNTDOWNS: [u16; 6] = [14, 4, 6, 16, 6, 8];
const LINK_ANIMATED_TILE_DMA_SOURCE_OFFSETS: [u16; 15] = [
    0, 0x20, 0x40, 0, 0x20, 0x40, 0, 0x40, 0x80, 0, 0x40, 0x80, 0xb340, 0xb400, 0xb4c0,
];

// Preserve the contiguous layout of native constant data. The source indexes
// these tables in bytes, including odd offsets and reads spanning fragments.
const LINK_DMA_TABLE_FRAGMENTS: &[(u16, &[u16])] = &[
    (0x8494, &LINK_PUSH_BLOCK_DMA_BASE_SOURCES),
    (0x849c, &LINK_SWORD_DMA_BASE_SOURCES),
    (0x84ac, &LINK_SHIELD_DMA_BASE_SOURCES),
    (0x84b2, &LINK_AUX_DMA_BASE_SOURCES),
    (0x85b2, &LINK_AUX_DMA_GROUP_OFFSETS),
    (0x85d2, &LINK_ANIMATED_TILE_DMA_COUNTDOWNS),
    (0x85de, &LINK_ANIMATED_TILE_DMA_SOURCE_OFFSETS),
    (0x9396, &LINK_BODY_DMA_BASE_SOURCES),
    (0x95f4, &LINK_HEAD_DMA_BASE_SOURCES),
    (0x9852, &LINK_HAND_DMA_BASE_SOURCES),
];

pub(crate) fn link_dma_data_byte(address: u32) -> Option<u8> {
    LINK_DMA_TABLE_FRAGMENTS.iter().find_map(|&(base, words)| {
        let offset = address.checked_sub(u32::from(base))? as usize;
        words.get(offset / 2).map(|word| word.to_le_bytes()[offset & 1])
    })
}

fn link_dma_table_word(table: crate::cycle_models::sprite_dma::Table, byte_index: u16) -> u16 {
    let address = u32::from(table.base()) + u32::from(byte_index);
    let read = |address| link_dma_data_byte(address).unwrap_or_else(|| {
        panic!("sprite DMA {table:?} byte index {byte_index:#06x}: native data missing at {address:#08x}")
    });
    u16::from_le_bytes([read(address), read(address + 1)])
}

fn receive_item_tab1_misc(item: u8) -> u8 {
    RECEIVE_ITEM_OAM_EXT_SIZES
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab2_misc(item: u8) -> i8 {
    RECEIVE_ITEM_DRAW_Y_OFFSETS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab3_misc(item: u8) -> u8 {
    RECEIVE_ITEM_PALETTE_BITS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

pub(super) fn receive_item_gfx_misc(item: u8) -> u8 {
    RECEIVE_ITEM_GRAPHICS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

impl ZeldaState {
    #[track_caller]
    pub(super) fn get_random_number(&mut self) -> u8 {
        self.get_random_number_with_carry().value()
    }

    #[track_caller]
    pub(super) fn get_random_number_with_carry(&mut self) -> crate::rom_random::RomRandomResult {
        // $0D:BA71: two system-bank PPU reads, ADC $1A, ADC/STA $0FA1,
        // then RTL. The JSL belongs to the caller. RNG replay supplies the
        // input/result but does not remove the native routine's CPU work.
        crate::cycle_ledger::charge_routine(0x0d_ba71, 30 + 30 + 24 + 32 + 32 + 44);
        let before = self.game_state.world.region.rng_seed();
        let result = self.rom_random_replay.take_next().unwrap_or_else(|| {
            let value = before.wrapping_add(self.game_state.frame.frame_counter);
            let value = if value & 1 != 0 {
                value >> 1
            } else {
                (value >> 1) ^ 0xb8
            };
            // The translated engine's deterministic fallback is not a CPU
            // instruction stream and historically exposed only the byte.
            crate::rom_random::RomRandomResult::new(value, false)
        });
        let t = result.value();
        if crate::debug_env::var_os("ZELDA3_DEBUG_ROM_RANDOM_TRACE").is_some() {
            eprintln!(
                "rom_random_ctx host={} module={:02x}/{:02x}/{:02x} cur_sprite={} value={:02x}",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
                self.game_state.sprites.system.cur_object_index(),
                t,
            );
        }
        self.set_rng_seed(t);
        result
    }

    pub(super) fn module_reserved_12(&mut self) {
        // C Module_Unknown0 is an assert(0) main-module slot.
        panic!("module_reserved_12 reached");
    }

    pub(super) fn module_reserved_13(&mut self) {
        // C Module_Unknown1 is an assert(0) main-module slot.
        panic!("module_reserved_13 reached");
    }

    pub(crate) fn native_song_upload_data(&self) -> Result<std::collections::BTreeMap<u32, u8>, String> {
        let assets = self.assets.as_ref().ok_or("native song upload needs assets")?;
        let mut data = std::collections::BTreeMap::new();
        for (name, start) in [
            ("kSpcBootstrapUpload", 0x198000u32),
            ("kSpcRuntimeUpload0", SONG_BANK_UPLOAD_STREAM_SNES_ADDRS[0]),
            ("kSpcRuntimeUpload1", SONG_BANK_UPLOAD_STREAM_SNES_ADDRS[1]),
            ("kSpcRuntimeUpload2", SONG_BANK_UPLOAD_STREAM_SNES_ADDRS[2]),
        ] {
            let stream = assets.asset_by_name(name).ok_or_else(|| format!("missing {name}; regenerate assets"))?;
            for (i, &value) in stream.iter().enumerate() {
                let offset = (start & 0x7fff) + i as u32;
                let address = ((start >> 16) + offset / 0x8000) << 16 | 0x8000 | (offset & 0x7fff);
                if let Some(old) = data.insert(address, value) { assert_eq!(old, value, "overlapping song data"); }
            }
        }
        Ok(data)
    }

    pub(super) fn sound_load_intro_song_bank(&mut self) {
        self.store_intro_song_upload_pointer();
        self.select_song_bank_asset(0);
    }

    /// Native caller's upload pointer stores at $8901-$890B. The following
    /// memory initializer preserves its low word; it does not recreate
    /// bytes sampled from a later NMI boundary.
    pub(super) fn store_intro_song_upload_pointer(&mut self) {
        self.ram[..3].copy_from_slice(&[0x00, 0x80, 0x19]);
        if let Some(cpu) = self.native_startup_cpu.as_ref() {
            // The upload owner advances the bank when its Y cursor wraps.
            self.ram[..3].copy_from_slice(&cpu.upload_pointer);
        }
    }

    pub(super) fn load_overworld_songs(&mut self) {
        self.begin_song_bank_transfer(0);
    }

    pub(super) fn load_dungeon_songs(&mut self) {
        self.begin_song_bank_transfer(1);
    }

    pub(super) fn load_credits_songs(&mut self) {
        self.begin_song_bank_transfer(2);
    }

    fn begin_song_bank_transfer(&mut self, asset: usize) {
        // Runtime bank changes first ask the resident SPC driver to enter its
        // upload receiver with a direct main-CPU APUI0 write. It is not an NMI
        // music latch: the real driver observes the one-shot write on its own
        // instruction boundary and performs the voice reset. The intro bank is
        // different because the IPL upload receiver is already active.
        let asset_stream = self
            .asset_raw(asset)
            .expect("runtime song bank asset must be loaded")
            .to_vec();
        let stream = self
            .rom_song_bank_upload_stream(asset)
            .unwrap_or_else(|| asset_stream.clone());
        if !self.begin_runtime_song_bank_transfer(asset as u8, &stream) {
            // The semantic sequencer has no SPC receiver to acknowledge the
            // byte stream. Preserve its established atomic bank publication.
            self.load_song_bank(&asset_stream);
        }
    }

    fn rom_song_bank_upload_stream(&self, asset: usize) -> Option<Vec<u8>> {
        let mut address = *SONG_BANK_UPLOAD_STREAM_SNES_ADDRS.get(asset)?;
        let mut stream = Vec::new();
        loop {
            let length_low = self.rom_byte_snes(address)?;
            address = next_snes_addr(address);
            let length_high = self.rom_byte_snes(address)?;
            address = next_snes_addr(address);
            stream.extend_from_slice(&[length_low, length_high]);
            let length = u16::from_le_bytes([length_low, length_high]);
            if length == 0 {
                return Some(stream);
            }

            for _ in 0..usize::from(length) + 2 {
                stream.push(self.rom_byte_snes(address)?);
                address = next_snes_addr(address);
            }
        }
    }

    pub(super) fn select_song_bank_asset(&mut self, asset: usize) {
        self.select_modern_sample_bank(asset as u8);
    }

    pub fn zelda_debug_song_bank_summary(&self, asset: usize) -> String {
        let Some(bank) = self.asset_raw(asset) else {
            return format!("song_bank[{asset}]=missing");
        };
        let mut ram = [0u8; 0x10000];
        let mut cursor = 0usize;
        let mut blocks = 0usize;
        let mut total_bytes = 0usize;
        let mut first_target = None;
        let mut min_target = 0xffffusize;
        let mut max_written = 0usize;
        while cursor + 4 <= bank.len() {
            let num_bytes = u16::from_le_bytes([bank[cursor], bank[cursor + 1]]) as usize;
            if num_bytes == 0 {
                cursor += 2;
                break;
            }
            let mut target = u16::from_le_bytes([bank[cursor + 2], bank[cursor + 3]]) as usize;
            cursor += 4;
            if cursor + num_bytes > bank.len() {
                return format!(
                    "song_bank[{asset}] malformed blocks={blocks} target={target:04x} len={num_bytes} remaining={}",
                    bank.len().saturating_sub(cursor)
                );
            }
            first_target.get_or_insert(target);
            min_target = min_target.min(target);
            for &byte in &bank[cursor..cursor + num_bytes] {
                ram[target & 0xffff] = byte;
                target = target.wrapping_add(1);
            }
            max_written = max_written.max(target.wrapping_sub(1) & 0xffff);
            cursor += num_bytes;
            blocks += 1;
            total_bytes += num_bytes;
        }
        let first_target = first_target.unwrap_or(0);
        let reset_vector = u16::from_le_bytes([ram[0xfffe], ram[0xffff]]);
        let code800 = &ram[0x800..0x808];
        let code878 = &ram[0x878..0x880];
        format!(
            "song_bank[{asset}] size={} blocks={blocks} total_bytes={total_bytes} first_target={first_target:04x} range={min_target:04x}-{max_written:04x} reset={reset_vector:04x} ram0800={code800:02x?} ram0878={code878:02x?} parse_end={cursor}",
            bank.len()
        )
    }

    fn kill_agahnim_load_music(&mut self) {
        self.clear_core_update_disable_flag();
        self.increment_overworld_map_state();
        self.increment_submodule();
        self.LoadOWMusicIfNeeded();
    }

    fn kill_aghanim_init(&mut self) {
        self.set_music_control(8);
        self.set_edge_transition_direction_bits(8);
        self.InitializeMirrorHDMA();
        self.set_overworld_map_state(0);
        self.palette_filter_initialize_white_filter();
        self.overworld_load_gfx_and_screen_size();
        self.increment_submodule();
        self.follower_link_state_mut().set_handler_state(20);
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        self.dungeon_savegame_state_mut()
            .clear_savegame_state_bits();
        self.follower_link_state_mut().clear_movement_velocity();
        self.set_main_color_constant(0, 0x7fff);
        self.set_main_color_constant(32, 0x7fff);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }

    fn kill_aghanim_func2(&mut self) {
        self.set_hdma_enable_mask(192);
        self.mirror_warp_build_waving_hdma_table();
        self.increment_submodule();
        self.set_subsubmodule(0);
    }

    fn kill_aghanim_func3(&mut self) {
        self.mirror_warp_build_waving_hdma_table();
        if self.game_state.frame.subsubmodule != 0 {
            self.set_subsubmodule(0);
            self.increment_submodule();
        }
    }

    fn kill_aghanim_func4(&mut self) {
        self.mirror_warp_build_dewaving_hdma_table();
        if self.game_state.frame.subsubmodule != 0 {
            self.set_subsubmodule(0);
            self.increment_submodule();
        }
    }

    fn kill_aghanim_func5(&mut self) {
        self.hdma_setup(0, 0xf2fb, 0x41, 0, 0x26, 0);
        for i in 0..240 {
            self.set_spotlight_hdma_table_dynamic_entry(i, 0xff00);
        }
        self.set_countdown(0);
        self.set_darkening_or_lightening_screen(0);
        self.dialogue_message_index_mut().set_value(0x35);
        self.main_show_text_message();
        if self.rom_startup_timing() {
            // The seven-sheet reload spans the wire's held vblanks under the
            // dialogue module the message call just entered (route host
            // 315323); the HUD rebuild and the Module15/6 return follow.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishModule09LongLoad {
                    step: crate::zelda_rtl::Module09LongLoadStep::Module15ReloadSheetsAfterMessage,
                },
                15,
            );
            return;
        }
        self.ReloadPreviouslyLoadedSheets();
        self.hud_rebuild_indoor();
        self.set_hdma_enable_mask(0x80);
        self.set_main_module(21);
        self.set_submodule(6);
        self.set_subsubmodule(24);
    }

    fn kill_aghanim_func6(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule == 0 {
            self.increment_submodule();
            self.set_ambient_sound_effect(9);
        }
    }

    fn kill_aghanim_func7(&mut self) {
        self.RenderText();
        if self.game_state.frame.submodule == 0 {
            self.set_overworld_map_state(0);
            self.set_ambient_sound_effect(5);
            if !self.game_state.inventory.items.has_moon_pearl() {
                self.dialogue_message_index_mut().set_value(0x36);
                self.main_show_text_message();
                self.set_ambient_sound_effect(0);
                self.set_main_module(21);
                self.set_submodule(8);
            } else {
                self.set_submodule(9);
            }
        }
    }

    fn kill_aghanim_func8(&mut self) {
        self.RenderText();
        if self.game_state.frame.submodule == 0 {
            self.set_subsubmodule(32);
            self.set_submodule(12);
        }
    }

    fn kill_aghanim_func12(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule != 0 {
            return;
        }
        self.ResetAncillaAndCutscene();
        self.overworld_set_song_list();
        self.set_overworld_event_bits(0x1b, 32);
        self.save_progress_mut().set_palace_index_x2(255);
        self.set_submodule(0);
        self.set_overworld_map_state(0);
        self.clear_core_update_disable_flag();
        self.set_main_module(9);
        self.set_bg1_y_low(0);
        let music = if self.game_state.inventory.items.has_moon_pearl() {
            9
        } else {
            4
        };
        self.set_music_control(music);
        self.save_progress_mut().set_map_icons_indicator(6);
    }

    pub(super) fn dungeon_light_torch(&mut self) {
        if !self.game_state.dungeon.torch.targets_torch() {
            self.dungeon_torch_mut().clear_target();
            return;
        }

        let r8 = if (self.game_state.world.location.dungeon_room() as u8) == 0 {
            0x80
        } else {
            0xc0
        };
        let i = self.game_state.dungeon.torch.attr_index()
            + ((self.game_state.dungeon.torch.torches_start_index() >> 1) as usize);
        // DUNG_OBJECT_POS_IN_OBJDATA (0x520) is owned by the dungeon object parser
        // (DungeonObjectTrackingState); read the live value from that sole owner
        // (matches the ROM reading $0520 directly) rather than a torch-held copy.
        let opos = self
            .game_state
            .dungeon
            .object_tracking
            .object_pos_in_objdata(i) as usize;
        let mut tilemap_pos = self
            .game_state
            .dungeon
            .object_tracking
            .object_tilemap_pos(i);
        if tilemap_pos & 0x8000 != 0 {
            return;
        }
        tilemap_pos |= 0x8000;
        self.dungeon_object_tracking_mut()
            .set_object_tilemap_pos(i, tilemap_pos);
        if r8 == 0 {
            self.dungeon_torch_mut()
                .set_torch_data_word_index(opos, tilemap_pos);
        }

        let x = tilemap_pos & 0x3fff;
        self.room_draw_adjust_torch_lighting_change(x, 0x0eca, x);
        let sfx = 42 | self.calculate_sfx_pan_arbitrary(((x & 0x7f) * 2) as u8);
        self.set_sound_effect_1(sfx);
        self.request_nmi_copy_packets();

        if self.game_state.dungeon.torch.wants_lights_out() != 0 {
            let lit = self.game_state.dungeon.torch.lit_torches();
            self.dungeon_torch_mut().increment_lit_torches();
            if lit < 3 {
                let color_plus = LIT_TORCH_COLOR_PLUS_BY_COUNT
                    [self.game_state.dungeon.torch.lit_torches() as usize];
                self.set_sub_screen_layers(0);
                self.set_overworld_fixed_color_adjustment(color_plus);
                self.set_submodule(10);
                self.set_subsubmodule(0);
            }
        }

        let torch_timer = self.game_state.dungeon.torch.attr_index();
        self.dungeon_torch_mut().set_timer(torch_timer, r8);
        self.dungeon_torch_mut().clear_target();
    }

    pub(super) fn room_draw_adjust_torch_lighting_change(&mut self, x: u16, y: u16, r8: u16) {
        let ptr = self.src_ptr(y);
        let x = (x >> 1) as usize;
        self.set_overworld_tile_attribute_word(x, ptr[0]);
        self.set_overworld_tile_attribute_word(x + 64, ptr[1]);
        self.set_overworld_tile_attribute_word(x + 1, ptr[2]);
        self.set_overworld_tile_attribute_word(x + 65, ptr[3]);
        // These writes hit the BG2 tilemap RAM directly (OVERWORLD_TILE_ATTR_BUFFER aliases
        // DUNG_BG2) but bypass the live dungeon bg2 cache. Mirror the 2x2 block back so the
        // frame-end bg2_tiles projection doesn't re-stamp the stale pre-change tiles (f309018).
        let dst = crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + x * 2;
        self.game_state
            .dungeon
            .room_tilemaps
            .mirror_decoded_map32_from_ram(&self.ram, dst);
        self.dungeon_prep_overlay_dma_next_prep(0, r8);
    }

    pub(super) fn dungeon_prep_overlay_dma_next_prep(&mut self, dst: usize, r8: u16) -> usize {
        let r6 = 0x0880 + u16::from((r8 & 0x003f) >= 0x003a);
        self.dungeon_prep_overlay_dma_watergate(dst, r8, r6, 4)
    }

    #[rustfmt::skip]
    pub(super) fn dungeon_prep_overlay_dma_watergate(&mut self, dst: usize, r8: u16, r6: u16, loops: usize) -> usize {
        let mut dst = dst;
        let mut r8 = r8;
        for _ in 0..loops {
            let x = (r8 >> 1) as usize;
            self.set_overworld_tile_upload_word(dst, ((r8 & 0x40) << 4) | ((r8 & 0x303f) >> 1) | ((r8 & 0x0f80) >> 2));
            self.set_overworld_tile_upload_word(dst + 1, r6);
            let attr0 = self.overworld_tile_attribute_word(x);
            self.set_overworld_tile_upload_word(dst + 2, attr0);
            if r6 & 1 == 0 {
                for j in 1..=3 {
                    let attr = self.overworld_tile_attribute_word(x + j);
                    self.set_overworld_tile_upload_word(dst + 2 + j, attr);
                }
                r8 = r8.wrapping_add(128);
            } else {
                for (j, offset) in [64usize, 128, 192].into_iter().enumerate() {
                    let attr = self.overworld_tile_attribute_word(x + offset);
                    self.set_overworld_tile_upload_word(dst + 3 + j, attr);
                }
                r8 = r8.wrapping_add(2);
            }
            dst += 6;
        }
        self.terminate_overworld_tile_upload_words(dst);
        dst
    }

    fn src_ptr(&self, src: u16) -> [u16; 4] {
        let Some(data) = self.asset_raw(69) else {
            return [0; 4];
        };
        let offset = src as usize;
        [
            read_word_from_slice(data, offset),
            read_word_from_slice(data, offset + 2),
            read_word_from_slice(data, offset + 4),
            read_word_from_slice(data, offset + 6),
        ]
    }

    pub(super) fn module05_load_file(&mut self) {
        if self.try_native_load_file_entry() {
            self.run_native_intro_clear_until_boundary();
            return;
        }
        if self.rom_startup_timing() {
            self.begin_selected_game_load();
            return;
        }

        self.complete_module05_load_file();
    }

    /// Publish only the retained load-game parent's stores. Each child has its
    /// own state owner; returning from a child does not import a source image.
    pub(crate) fn native_load_file_parent_store(&mut self, address: u32, value: u16, width: u8) -> bool {
        let offset = if (0x7e0000..0x800000).contains(&address) {
            address as usize - 0x7e0000
        } else { address as usize };
        match offset {
            0x10 | 0x11 | 0x13 | 0x1c | 0x1d | 0x3a | 0x3c | 0x57 |
            0x94 | 0x95 | 0x99 | 0x9a | 0x9c..=0x9e | 0xa0..=0xa3 | 0xb0 |
            0x10a | 0x10c | 0x12d | 0x223 | 0x2f0 | 0x402 | 0x458 | 0x45a |
            0x4aa | 0x4b4 | 0x4c7 | 0x62c | 0x62e | 0xaa4 | 0xaa9 | 0xab1 |
            0x1cd8 | 0x1cf0 | 0xc005..=0xc00b | 0xc011 | 0xc017 |
            0xc019 | 0xc01b | 0xc01d | 0xc01f | 0xc021 | 0xc023 | 0xf3cc => {}
            _ => return false,
        }
        assert!(matches!(width, 1 | 2));
        for byte in 0..usize::from(width) {
            let at = offset + byte;
            let v = (value >> (byte * 8)) as u8;
            let mut owned = self.game_state.dungeon.write_load_parent_byte(at, v);
            owned |= self.game_state.display.write_load_parent_byte(at, v);
            owned |= self.game_state.player.follower_link.write_load_parent_byte(at, v);
            match at {
                0x10 => self.game_state.frame.set_main_module(v),
                0x11 => self.game_state.frame.set_submodule(v),
                0xb0 => self.game_state.frame.set_subsubmodule(v),
                0x10c => self.game_state.frame.set_saved_module_for_menu(v),
                0x10a => self.game_state.system_signals.set_game_over_check_flag(v),
                0x4aa => self.game_state.system_signals.set_restart_check_flag(v),
                0x12d => self.game_state.system_signals.set_ambient_sound_effect(v),
                0x12e => self.game_state.system_signals.set_sound_effect_1(v),
                0xa0..=0xa1 => { assert!(self.game_state.world.write_entrance_byte(at, v)); }
                0x223 => self.game_state.world.transient.tile_interaction_shared_flag = v,
                0xaa4 => self.game_state.world.palette_theme.set_misc_sprites_graphics_index(v),
                0x2f0 => self.game_state.messaging.runtime.message_or_sprite_state_cache = v,
                0x1cd8 => self.game_state.messaging.runtime.module = v,
                0x1cf0..=0x1cf1 => {
                    let index = &mut self.game_state.messaging.dialogue_message_index;
                    let shift = (at & 1) * 8;
                    index.set_value((index.value() & !(255 << shift)) | (u16::from(v) << shift));
                }
                0xf3cc => self.game_state.sprites.follower_runtime.set_indicator(v),
                // The palette settings are retained WRAM words read by the
                // native Agahnim palette functions, not a projected state.
                0xc019..=0xc024 => {}
                _ => { assert!(owned, "unowned load-game parent byte {at:04x}"); continue; }
            }
        }
        self.ram[offset..offset + usize::from(width)]
            .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
        true
    }

    pub(crate) fn native_load_file_data(&self) -> Option<std::collections::BTreeMap<u32, u8>> {
        let mut data = std::collections::BTreeMap::new();
        let mut put = |address, value| {
            if let Some(old) = data.insert(address, value) {
                assert_eq!(old, value, "overlapping native load-game data {address:06x}");
            }
        };
        for part in [self.native_dungeon_upload_data()?, self.native_dungeon_attribute_data()?,
            self.native_palette_data()?, self.native_dungeon_sprite_data()?,
            self.native_song_upload_data().expect("native song upload assets")] {
            for (address, value) in part { put(address, value); }
        }
        for range in [0x02c813..0x02d8b3, 0x02db6e..0x02dc55, 0x04f1de..0x04f4ea] {
            for address in range { put(address, self.native_dungeon_entrance_table_byte(address)?); }
        }
        for (base, asset) in [(0x00fa41, 161), (0x00fb41, 162)] {
            for (i, &value) in self.asset_raw(asset)?.iter().enumerate() {
                put(base + i as u32, value);
            }
        }
        for (i, &value) in DUNGEON_DEFAULT_ATTR.iter().enumerate() { put(0x0e9659 + i as u32, value); }
        for address in (0x00d2be..0x00d2c8).chain(0x00d300..0x00d308) {
            put(address, super::load_gfx::native_gear_graphics_table_byte(address)?);
        }
        Some(data)
    }

    pub(super) fn complete_module05_load_file(&mut self) {
        self.complete_module05_load_file_inner();
    }

    pub(super) fn begin_selected_game_load_pre_dungeon_audio(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        if destination == SelectedGameLoadDestination::Dungeon {
            self.complete_module05_load_file_common_after_force_blank();
            self.selected_game_load_dungeon_hud_prefix();
            self.module_pre_dungeon_audio_prefix();
        }
    }

    /// Resume the selected-game caller at the exact source checkpoint emitted
    /// after `Sprite_DisableAll`. The Module05/common and HUD/room prefixes ran
    /// at the pre-dungeon-audio boundary and must not be replayed here.
    pub(super) fn complete_selected_game_load_through_sprite_disable_all(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        assert!(
            destination == SelectedGameLoadDestination::Dungeon,
            "only a dungeon selected-game destination owns Sprite_ResetAll progress",
        );
        self.module_pre_dungeon_after_entrance_prefix();
        self.complete_module_pre_dungeon_through_sprite_disable_all();
    }

    /// Finish the selected-game caller after the source has proved that its
    /// nested Sprite_ResetAll prefix already completed. This resumes at
    /// `Sprite_ResetAll_noDisable` and never replays the one-time load prefix.
    pub(super) fn complete_selected_game_load_after_sprite_disable_all(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        assert!(
            destination == SelectedGameLoadDestination::Dungeon,
            "a non-dungeon selected-game destination has no Sprite_ResetAll suffix",
        );
        self.complete_module_pre_dungeon_after_sprite_disable_all();
        self.complete_module_pre_dungeon_authoritative_return();
    }

    pub(super) fn complete_selected_game_load_without_pre_dungeon_sprite_reset(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        assert_ne!(
            destination,
            SelectedGameLoadDestination::Dungeon,
            "a dungeon selected-game destination cannot skip Sprite_ResetAll",
        );
        self.complete_module05_load_file_common_after_force_blank();
        self.complete_module05_load_file_destination(destination);
    }

    /// Publish the exact mid-call state reached when Module05's Message path
    /// returns from `Main_ShowTextMessage`. The common load prefix and message
    /// selection have completed; palette loading and the final Module1B
    /// publication remain suspended in the source caller.
    pub(super) fn publish_selected_game_load_message_interface(&mut self) {
        self.complete_module05_load_file_common_after_force_blank();
        self.begin_module05_load_file_message_destination();
    }

    /// Resume Module05 immediately after its already-published
    /// `Main_ShowTextMessage` call, without replaying the common load prefix or
    /// the dialogue-interface writes.
    pub(super) fn complete_selected_game_load_message_after_interface_publication(&mut self) {
        self.finish_module05_load_file_message_destination();
    }

    pub(super) fn complete_selected_game_load_from_frozen_continuation(
        &mut self,
        destination: SelectedGameLoadDestination,
        sprite_reset: PreDungeonSpriteResetContinuation,
    ) {
        match (destination, sprite_reset) {
            (SelectedGameLoadDestination::Dungeon, PreDungeonSpriteResetContinuation::Pending) => {
                self.module_pre_dungeon_after_entrance_prefix();
                self.complete_module_pre_dungeon_before_return_suffix();
                self.complete_module_pre_dungeon_authoritative_return();
            }
            (
                SelectedGameLoadDestination::Dungeon,
                PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted,
            ) => self.complete_selected_game_load_after_sprite_disable_all(destination),

            (
                SelectedGameLoadDestination::DarkWorldOverworld
                | SelectedGameLoadDestination::Message,
                PreDungeonSpriteResetContinuation::NotApplicable,
            ) => self.complete_selected_game_load_without_pre_dungeon_sprite_reset(destination),
            _ => {
                panic!("selected-game destination and nested Sprite_ResetAll continuation diverged")
            }
        }
    }

    /// Selected-game-load pre-dungeon-audio frame: run the ROM's entry room-load so
    /// DUNGEON_ROOM ($A0) is set at the ROM's frame rather than the completion
    /// boundary (bug class 5). Mirrors the pre-draw prefix of
    /// `module_pre_dungeon_after_audio_prefix_with_song_bank_timing`. The
    /// selected-load continuation resumes at the post-entrance prefix, so the
    /// entrance call runs exactly once while the room draw remains pending.
    pub(super) fn selected_game_load_entry_room_load(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        if destination != SelectedGameLoadDestination::Dungeon {
            return;
        }
        self.module_pre_dungeon_initial_entrance();
    }

    pub(super) fn complete_pending_selected_game_load_entry_room_load(&mut self) {
        let Some(destination) = self
            .game_execution_scheduler
            .selected_game_load_pending_entry_room_load()
        else {
            return;
        };
        self.selected_game_load_entry_room_load(destination);
        self.game_execution_scheduler
            .mark_selected_game_load_entry_room_load_completed();
    }

    pub(super) fn selected_game_load_destination(&self) -> SelectedGameLoadDestination {
        if self.game_state.inventory.save_progress.dark_world_state() != 0 {
            if self.game_state.world.location.is_indoors() {
                SelectedGameLoadDestination::Dungeon
            } else {
                SelectedGameLoadDestination::DarkWorldOverworld
            }
        } else if self.game_state.display.mosaic_level != 0
            || (self.game_state.system_signals.game_over_check_flag() != 0
                && self.game_state.system_signals.restart_check_flag() == 0)
            || self.game_state.inventory.save_progress.progress_indicator() < 2
            || self
                .game_state
                .inventory
                .save_progress
                .which_starting_point()
                == 5
        {
            SelectedGameLoadDestination::Dungeon
        } else {
            SelectedGameLoadDestination::Message
        }
    }

    fn complete_module05_load_file_inner(&mut self) {
        let destination = self.selected_game_load_destination();
        self.enable_force_blank();
        self.complete_module05_load_file_common_after_force_blank();
        self.complete_module05_load_file_destination(destination);
    }

    fn complete_module05_load_file_common_after_force_blank(&mut self) {
        self.set_overworld_map_state(0);
        self.dungeon_environment_mut()
            .clear_somaria_block_bg_check_flag();
        self.follower_state_mut().clear_tagalong_shared_state_a();
        self.follower_state_mut().clear_draw_anim_frame();
        self.follower_state_mut().set_appearance_none_flag(0);
        self.follower_link_state_mut()
            .clear_player_pose_draw_counter();
        self.follower_link_state_mut()
            .clear_player_special_draw_flag();
        self.erase_tile_maps_normal();
        self.load_default_graphics();
        self.sprite_load_graphics_properties();
        self.init_load_default_tile_attr();
        self.decompress_sword_graphics();
        self.decompress_shield_graphics();
        self.link_initialize();
        self.load_follower_graphics();
        self.sprite_workspace_mut().set_graphics_subset(0, 70);
        self.sprite_workspace_mut().set_graphics_subset(1, 70);
        self.sprite_workspace_mut().set_graphics_subset(2, 70);
        self.sprite_workspace_mut().set_graphics_subset(3, 70);
        self.start_shared_message_timer(0x0200);
        self.set_vertical_irq_trigger(48);
    }

    fn complete_module05_load_file_destination(
        &mut self,
        destination: SelectedGameLoadDestination,
    ) {
        match destination {
            SelectedGameLoadDestination::Dungeon => {
                self.load_dungeon_room_rebuild_hud();
            }
            SelectedGameLoadDestination::DarkWorldOverworld => {
                self.hud_search_for_equipped_item();
                self.hud_rebuild();
                self.hud_update_equipped_item();
                self.clear_game_over_check_flag();
                self.set_dungeon_room(32);
                self.set_main_module(8);
                self.set_submodule(0);
                self.set_subsubmodule(0);
                self.clear_restart_check_flag();
            }
            SelectedGameLoadDestination::Message => {
                self.begin_module05_load_file_message_destination();
                self.finish_module05_load_file_message_destination();
            }
        }
    }

    fn begin_module05_load_file_message_destination(&mut self) {
        let message = if self.game_state.inventory.items.mirror() == 2 {
            0x0185
        } else {
            0x0184
        };
        self.dialogue_message_index_mut().set_value(message);
        self.main_show_text_message();
    }

    fn finish_module05_load_file_message_destination(&mut self) {
        self.dungeon_load_palettes();
        self.set_screen_brightness(15);
        self.set_main_screen_layers(4);
        self.set_sub_screen_layers(0);
        self.set_main_module(27);
    }

    pub(super) fn load_dungeon_room_rebuild_hud(&mut self) {
        self.selected_game_load_dungeon_hud_prefix();
        self.module_pre_dungeon();
    }

    fn selected_game_load_dungeon_hud_prefix(&mut self) {
        self.clear_mosaic_level();
        self.set_mosaic_copy(7);
        self.hud_search_for_equipped_item();
        self.hud_rebuild();
        self.hud_update_equipped_item();
    }

    pub(super) fn load_pre_dungeon_keys(&mut self) {
        let d = self
            .game_state
            .inventory
            .save_progress
            .palace_index_x2_word();
        let keys = if d != 0x00ff {
            let index = if d == 2 { 0 } else { (d >> 1) as usize };
            self.game_state
                .inventory
                .dungeon_key_slots
                .keys_earned_slot(index)
        } else {
            0xff
        };
        self.player_resources_mut().set_keys(keys);
    }

    pub(super) fn init_load_default_tile_attr(&mut self) {
        self.dungeon_room_parser_mut()
            .copy_default_tile_attrs_head(&DUNGEON_DEFAULT_ATTR[..0x140]);
        self.dungeon_room_parser_mut()
            .copy_default_tile_attrs_tail(&DUNGEON_DEFAULT_ATTR[0x140..0x180]);
    }

    pub(super) fn module13_boss_victory_pendant(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.dungeon_prep_exit_with_spotlight(),
            5 => self.spotlight_configure_table_and_control_without_caller_interruption(),
            _ => {}
        }
        if let Some((boundary, resume_boundary)) =
            self.take_original_timing_sprite_main_boundary_for_fresh_caller()
        {
            // The spotlight/victory submodule can consume the frame so that
            // vblank interrupts the shared Sprite_Main at a source-named
            // boundary; the Link/OAM suffix resumes across the accepted NMI
            // (route host 103998).
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::BossVictory {
                    boundary: resume_boundary,
                },
            );
        } else if let Some(boundary) = self.take_original_timing_sprite_main_progress() {
            // SCAN_KEYS returned the host while the source was inside
            // Sprite_Main without accepting an NMI (route host 104004). The
            // interrupting NMI arrives as the NEXT host's leading one; the
            // scheduled-work resume publishes that handler before the
            // resumed body.
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::BossVictory {
                    boundary: crate::zelda_rtl::OriginalTimingBoundary::HostReturn,
                },
            );
        } else if matches!(
            self.original_timing_owner,
            crate::zelda_rtl::OriginalTimingOwnerState::Live
        ) && self
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty())
            && self.original_timing_expected_nmi_update_gates.is_empty()
            && self.original_timing_sprite_main_return_claims_remaining == Some(0)
            && !self.original_timing_owes_sprite_main_return()
            && !self.original_timing_owes_sprite_main_progress()
            && self.original_timing_main_loop_return_timeline().is_none()
        {
            // An open-ended wire hold: the fresh iteration enters the
            // spotlight build and the host ends mid-call with no trailing
            // acceptance — the interrupting NMI arrives as the next host's
            // leading one, and the zero-claim plan proves Sprite_Main never
            // runs this host (route host 104000; the warp-pad analogue is
            // route host 92533). The suspension resumes across that
            // acceptance, not at this host's return.
            self.arm_authoritative_sprite_main_cpu_continuation(
                crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                SpriteMainCpuCaller::BossVictory {
                    boundary: crate::zelda_rtl::OriginalTimingBoundary::NmiAccepted,
                },
            );
        }
        self.sprite_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.link_oam_main();
    }

    pub(super) fn boss_victory_heal(&mut self) {
        if !self.hud_refill_magic_power() {
            self.increment_overworld_map_state();
        }
        if !self.hud_refill_health() {
            self.increment_overworld_map_state();
        }
        if self.overworld_map_state() == 0 {
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.Dungeon_ResetTorchBackgroundAndPlayerInner();
            self.follower_link_state_mut().set_facing(2);
            self.follower_link_state_mut().set_last_direction(2 << 1);
            self.increment_hud_update_flag();
            self.increment_submodule();
            self.set_subsubmodule(16);
            self.follower_link_state_mut().increment_immobilized_flag();
        }
        self.set_overworld_map_state(0);
        self.hud_refill_logic();
    }

    pub(super) fn dungeon_start_victory_spin(&mut self) {
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule != 0 {
            return;
        }
        self.follower_link_state_mut().clear_immobilized();
        self.follower_link_state_mut().set_facing(2);
        self.link_animate_victory_spin();
        self.ancilla_terminate_select_interactives(0);
        self.ancilla_add_victory_spin();
        self.increment_submodule();
    }

    pub(super) fn dungeon_run_victory_spin(&mut self) {
        self.link_main();
        if self.game_state.player.follower_link.handler_state() != 0 {
            return;
        }
        if self.game_state.inventory.items.sword_type().wrapping_add(1) & 0xfe != 0 {
            self.set_sound_effect_1(0x2c);
        }
        self.follower_link_state_mut().force_hold_sword_up();
        self.set_subsubmodule(32);
        self.increment_submodule();
    }

    pub(super) fn dungeon_close_victory_spin(&mut self) {
        if self.trace_frame_matches(241475) {
            eprintln!(
                "[VSPIN-NEW] ssm_before={}",
                self.game_state.frame.subsubmodule
            );
        }
        self.decrement_subsubmodule();
        if self.game_state.frame.subsubmodule != 0 {
            return;
        }
        self.increment_submodule();
        self.follower_link_state_mut().clear_movement_velocity();
        self.set_overworld_fixed_color_adjustment(0);
    }

    pub(super) fn module15_mirror_warp_from_aga(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.kill_agahnim_load_music(),
            1 => self.kill_aghanim_init(),
            2 => self.kill_aghanim_func2(),
            3 => self.kill_aghanim_func3(),
            4 => self.kill_aghanim_func4(),
            5 => self.kill_aghanim_func5(),
            6 => self.kill_aghanim_func6(),
            7 => self.kill_aghanim_func7(),
            8 => self.kill_aghanim_func8(),
            9 => self.boss_victory_heal(),
            10 => self.dungeon_start_victory_spin(),
            11 => self.dungeon_run_victory_spin(),
            12 => self.kill_aghanim_func12(),
            _ => {}
        }
        if self.rom_startup_timing() && self.game_execution_scheduler.work_is_pending() {
            // A submodule's long load suspended the caller (KillAghanim_Func5).
            return;
        }
        if self.game_state.frame.submodule < 2 || self.game_state.frame.submodule >= 5 {
            self.arm_live_victory_module_sprite_main_boundary();
            self.sprite_main();
            if self
                .game_execution_scheduler
                .work_suspends_translated_call_stack()
            {
                return;
            }
            self.link_oam_main();
        }
    }

    /// The Agahnim-warp and crystal-victory modules share Module13's
    /// Sprite_Main/LinkOam suffix. Under the live wire their long submodule
    /// bodies can consume the host before Sprite_Main runs (route host
    /// 315065), or vblank can name a mid-loop boundary; arm the same
    /// BossVictory caller continuation Module13 uses.
    pub(super) fn arm_live_victory_module_sprite_main_boundary(&mut self) {
        if let Some((boundary, resume_boundary)) =
            self.take_original_timing_sprite_main_boundary_for_fresh_caller()
        {
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::BossVictory {
                    boundary: resume_boundary,
                },
            );
        } else if let Some(boundary) = self.take_original_timing_sprite_main_progress() {
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::BossVictory {
                    boundary: crate::zelda_rtl::OriginalTimingBoundary::HostReturn,
                },
            );
        } else if let Some(boundary) =
            self.original_timing_fresh_iteration_interrupted_before_sprite_main()
        {
            self.arm_authoritative_sprite_main_cpu_continuation(
                crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                SpriteMainCpuCaller::BossVictory { boundary },
            );
        }
    }

    pub(super) fn module16_boss_victory_crystal(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.boss_victory_heal(),
            1 => self.dungeon_start_victory_spin(),
            2 => self.dungeon_run_victory_spin(),
            3 => self.dungeon_close_victory_spin(),
            4 => self.module16_04_fade_and_end(),
            _ => {}
        }
        self.arm_live_victory_module_sprite_main_boundary();
        self.sprite_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.link_oam_main();
    }

    pub(super) fn module16_04_fade_and_end(&mut self) {
        self.decrement_screen_brightness();
        if self.game_state.display.screen_brightness != 0 {
            return;
        }
        self.set_bg1_x_offset(0);
        self.set_bg1_y_offset(0);
        self.follower_link_state_mut().set_y_velocity(0);
        self.follower_link_state_mut().clear_immobilized();
        self.palette_revert_translucency_swap();
        self.follower_link_state_mut().set_handler_state(0);
        {
            self.follower_link_state_mut().set_receive_item_index(0);
            self.follower_link_state_mut().clear_item_hold_pose();
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
        }
        let saved_module = self.game_state.frame.saved_module_for_menu;
        self.set_main_module(saved_module);
        self.set_submodule(0);
        self.set_subsubmodule(0);
        self.open_spotlight_next2();
    }

    pub(super) fn triforce_room_link_approach_triforce(&mut self) {
        let y = self.game_state.player.follower_link.y() as u8;
        if y < 152 {
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut()
                .set_direction_and_last_direction(0);
            if self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer()
                == 0
            {
                self.follower_link_state_mut().set_item_hold_pose(2);
                self.increment_subsubmodule();
            }
        } else {
            if y < 169 {
                self.follower_link_state_mut().set_speed_setting(0x14);
            }
            self.follower_link_state_mut()
                .set_direction_and_last_direction(8);
            self.follower_link_state_mut().set_facing(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(64);
        }
    }

    pub(super) fn ancilla_add_item_receipt(&mut self, ain: u8, yin: u8, chest_pos: u16) {
        let _ = self.ancilla_add_item_receipt_from(
            ain,
            yin,
            chest_pos,
            ItemReceiptCaller::AtomicCaller,
        );
    }

    pub(super) fn ancilla_add_item_receipt_from(
        &mut self,
        ain: u8,
        yin: u8,
        chest_pos: u16,
        caller: ItemReceiptCaller,
    ) -> GameCallStatus {
        let Some(k) = self.ancilla_add_simple(ain, yin) else {
            return GameCallStatus::Returned;
        };
        let item = self.game_state.player.follower_link.receive_item_index();

        self.follower_link_state_mut()
            .set_immobilized_flag(if item == 0x20 { 2 } else { 1 });
        self.grant_initial_item_award(item);

        if item == 0x1f {
            self.follower_link_state_mut().clear_bunny_body_state();
        } else if item == 0x4b || item == 0x1e {
            self.player_resources_mut()
                .add_ability_flags(if item == 0x4b { 4 } else { 2 });
        }

        if item == 0x1b || item == 0x1c {
            self.palette_update_gloves_color();
        } else if matches!(item, 0x37..=0x39) {
            let bit = match item {
                0x37 => 4,
                0x38 => 1,
                _ => 2,
            };
            let value = self.game_state.inventory.player_resources.pendant_flags() | bit;
            self.player_resources_mut().set_pendant_flags(value);
            if value & 7 == 7 {
                self.save_progress_mut().set_map_icons_indicator(4);
            }
            self.increment_overworld_map_state();
        } else if item == 0x22 {
            self.inventory_items_mut()
                .grant_equipment_if_empty(EquipmentItem::Armor, 1);
        } else if matches!(item, 0x25 | 0x32 | 0x33) {
            let mask = 0x8000u16
                >> ((self.game_state.inventory.save_progress.palace_index_x2() >> 1) as u16);
            let reward = match item {
                0x25 => DungeonItem::Compass,
                0x32 => DungeonItem::BigKey,
                _ => DungeonItem::Map,
            };
            self.player_resources_mut().grant_dungeon_item(reward, mask);
        } else if item == 0x3e {
            if self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
            {
                self.follower_link_state_mut().set_picking_throw_state(2);
            }
        } else if item == 0x20 {
            self.increment_overworld_map_state();
            for i in (0..=4).rev() {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 7 | 0x2c) {
                    self.ancilla_slot_view_mut(i).clear();
                    self.follower_link_state_mut().clear_state_bits();
                    self.follower_link_state_mut().clear_picking_throw_state();
                }
            }
            if self.game_state.player.follower_link.is_cape_active() {
                self.follower_link_state_mut().set_cape_transform_timer(32);
                {
                    self.follower_link_state_mut()
                        .clear_sprite_damage_disable_timer();
                    self.follower_link_state_mut().clear_cape_mode();
                }
                self.ancilla_add_cape_poof(0x23, 4);
                let sfx = 0x15 | self.link_calculate_sfx_pan();
                self.set_sound_effect_1(sfx);
            }
        } else if item == 0x29 {
            if self.game_state.inventory.items.mushroom() != 2 {
                self.inventory_items_mut()
                    .grant_equipment(EquipmentItem::Mushroom, 1);
                self.hud_refresh_icon();
            }
        } else if item == 0x24
            || (self.game_state.player.follower_link.item_receipt_method() != 2
                && matches!(item, 0x27 | 0x28 | 0x31))
        {
            let add = match item {
                0x28 => 3,
                0x31 => 10,
                _ => 1,
            };
            if item == 0x24 {
                let keys = self.game_state.inventory.player_resources.keys();
                self.player_resources_mut()
                    .set_keys(keys.saturating_add(add).min(99));
            } else {
                let filler = self.game_state.inventory.player_resources.bomb_filler();
                self.player_resources_mut()
                    .set_bomb_filler(filler.saturating_add(add).min(99));
            }
            self.hud_refresh_icon();
        } else if item == 0x17 {
            self.player_resources_mut().advance_heart_piece_count();
            let sfx = 0x2d | self.link_calculate_sfx_pan();
            self.set_sound_effect_2(sfx);
        } else if item == 1 {
            self.overworld_set_song_list();
        } else {
            self.item_receipt_give_bottled_item(item);
        }

        let mut gfx = receive_item_gfx_misc(item);
        if gfx == 0xff {
            gfx = 0;
        } else if gfx == 0x20 || gfx == 0x2d || gfx == 0x2e {
            self.DecompressShieldGraphics();
            self.palette_load_shield();
        }
        let receipt = ItemReceiptReturn {
            ancilla_slot: k as u8, item, chest_position: chest_pos,
        };
        if caller == ItemReceiptCaller::GroundApress
            && self.begin_native_ground_item_graphics(gfx, receipt) {
            return GameCallStatus::Suspended;
        }
        self.decode_animated_sprite_tile_variable(gfx);
        if (gfx == 6 || gfx == 0x18) && item != 0 {
            self.DecompressSwordGraphics();
            self.palette_load_sword();
        }
        let receipt = ItemReceiptReturn {
            ancilla_slot: k as u8,
            item,
            chest_position: chest_pos,
        };
        let call_status = self.begin_item_receipt_graphics_work(gfx, receipt, caller);
        if call_status.is_suspended() {
            return call_status;
        }
        self.complete_ancilla_add_item_receipt(receipt);
        GameCallStatus::Returned
    }

    pub(super) fn complete_ancilla_add_item_receipt(&mut self, receipt: ItemReceiptReturn) {
        let k = receipt.ancilla_slot as usize;
        let item = receipt.item;
        let chest_pos = receipt.chest_position;
        {
            let mut receipt = self.ancilla_slot_view_mut(k);
            receipt.set_item_to_link(item);
            receipt.set_work_byte_1(0);
        }
        if item == 1 && self.game_state.player.follower_link.item_receipt_method() != 2 {
            self.ancilla_slot_view_mut(k).set_timer(160);
            self.set_submodule(43);
            self.set_countdown(0);
            self.ancilla_add_ms_cutscene(0x35, 4);
            self.ancilla_slot_view_mut(k).set_work_byte_3(2);
        } else {
            self.ancilla_slot_view_mut(k).set_work_byte_3(9);
        }
        let aux_timer = if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
            0x68
        } else if item == 0x26 {
            0x02
        } else if self.game_state.player.follower_link.item_receipt_method() != 0 {
            0x38
        } else {
            0x60
        };
        let item_receipt_method = self.game_state.player.follower_link.item_receipt_method();
        let mut receipt = self.ancilla_slot_view_mut(k);
        receipt.set_work_byte_4(5);
        receipt.set_step(item_receipt_method);
        receipt.set_aux_timer(aux_timer);

        let (x, y) = if self.game_state.player.follower_link.item_receipt_method() == 1 {
            let y = ((chest_pos & 0x1f80) >> 4)
                .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_v() & 0xff00)
                .wrapping_add(receive_item_tab2_misc(item) as i16 as u16);
            let x = ((chest_pos & 0x007e) << 2)
                .wrapping_add(self.game_state.dungeon.room_load.loading_bg_offset_h() & 0xff00)
                .wrapping_add(receive_item_tab3_misc(item) as u16);
            (x, y)
        } else {
            if self.ancilla_slot_view(k).step() == 0 && item == 1 {
                let sfx = self.link_calculate_sfx_pan() | 0x2c;
                self.set_sound_effect_1(sfx);
            } else if matches!(item, 0x20 | 0x37 | 0x38 | 0x39) {
                let music = self.link_calculate_sfx_pan() | 0x13;
                self.set_music_control(music);
            } else if item != 0x3e && item != 0x17 {
                let sfx = self.link_calculate_sfx_pan() | 0x0f;
                self.publish_or_defer_item_receipt_sound_effect_2(sfx);
            }
            let method = if self.game_state.player.follower_link.item_receipt_method() == 3 {
                0
            } else {
                self.game_state.player.follower_link.item_receipt_method()
            };
            let x_offset = if method != 0 {
                receive_item_tab3_misc(item) as u16
            } else if receive_item_tab1_misc(item) == 0 {
                10
            } else if item == 0x20 {
                0
            } else {
                6
            };
            let y_offset = if method != 0 {
                receive_item_tab2_misc(item) as i16
            } else {
                -14
            } + if method == 2 { -8 } else { 0 };
            (
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(x_offset),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(y_offset as u16),
            )
        };
        self.ancilla_set_xy(k, x, y);
    }

    pub(super) fn item_receipt_give_bottled_item(&mut self, item: u8) {
        if let Some(j) = ITEM_RECEIPT_BOTTLED_ITEM_IDS
            .iter()
            .position(|&candidate| candidate == item)
        {
            if self
                .inventory_items_mut()
                .fill_first_empty_bottle_with(j as u8 + 2)
            {
                return;
            }
        }

        if let Some(j) = ITEM_RECEIPT_POTION_ITEM_IDS
            .iter()
            .position(|&candidate| candidate == item)
        {
            if self
                .inventory_items_mut()
                .replace_first_empty_bottle_with(j as u8 + 3)
            {}
        }
    }

    pub(super) fn module17_save_and_quit(&mut self) {
        match self.game_state.frame.submodule {
            0 => {
                self.increment_submodule();
            }
            1 => {}
            _ => {}
        }
        if self.game_state.frame.submodule == 1 {
            self.decrement_screen_brightness();
            if self.game_state.display.screen_brightness == 0 {
                self.set_mosaic_copy(15);
                self.set_subsubmodule(1);
                if self.rom_startup_timing()
                    && matches!(
                        self.original_timing_owner,
                        crate::zelda_rtl::OriginalTimingOwnerState::Live
                    )
                {
                    // The ROM completes Death_Func15's fast prefix (through
                    // Sprite_ResetAll and the save write) before the long
                    // reset hold — the intro WRAM clear inside Death_Func31
                    // plus the overworld song-bank upload — which holds the
                    // wire for tens of latch-held slices before Module17's
                    // Sprite_Main/LinkOam suffix runs (route hosts
                    // 159333-159401). Run the prefix now and defer the slow
                    // remainder to the wire's terminal return; the held
                    // hosts are consumed by the save-quit reset plan.
                    self.death_func15_common_prefix(false);
                    self.death_func15_save_quit_pre_hold();
                    self.save_quit_reset_state_published = false;
                    self.save_quit_reset_writes_applied = false;
                    self.save_quit_reset_hold = true;
                    return;
                }
                self.death_func15(false);
            }
        }
        if let Some((boundary, resume_boundary)) =
            self.take_original_timing_sprite_main_boundary_for_fresh_caller()
        {
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::SaveAndQuit {
                    boundary: resume_boundary,
                },
            );
        } else if let Some(boundary) = self.take_original_timing_sprite_main_progress() {
            self.arm_authoritative_sprite_main_cpu_continuation(
                boundary,
                SpriteMainCpuCaller::SaveAndQuit {
                    boundary: crate::zelda_rtl::OriginalTimingBoundary::HostReturn,
                },
            );
        } else if matches!(
            self.original_timing_owner,
            crate::zelda_rtl::OriginalTimingOwnerState::Live
        ) && self
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty())
            && self.original_timing_expected_nmi_update_gates.is_empty()
            && self.original_timing_sprite_main_return_claims_remaining == Some(0)
            && !self.original_timing_owes_sprite_main_return()
            && !self.original_timing_owes_sprite_main_progress()
            && self.original_timing_main_loop_return_timeline().is_none()
        {
            // An open-ended wire hold: the Death_Func15 save-quit reset
            // consumes the whole host with no trailing acceptance — the
            // interrupting NMI arrives as the next host's leading one, and
            // the zero-claim plan proves Sprite_Main never runs this host
            // (route host 159333; the boss-victory analogue is route host
            // 104000). The suspension resumes across that acceptance.
            self.arm_authoritative_sprite_main_cpu_continuation(
                crate::zelda_rtl::SpriteMainCpuBoundary::BeforeFirstSlot,
                SpriteMainCpuCaller::SaveAndQuit {
                    boundary: crate::zelda_rtl::OriginalTimingBoundary::NmiAccepted,
                },
            );
        }
        self.sprite_main();
        if self
            .game_execution_scheduler
            .work_suspends_translated_call_stack()
        {
            return;
        }
        self.link_oam_main();
    }

    pub(super) fn wall_master_send_player_to_last_entrance(&mut self) {
        self.wall_master_send_player_through_reset_fixed_prefix();
        self.wall_master_send_player_after_reset_fixed_prefix();
    }

    pub(super) fn wall_master_send_player_through_reset_fixed_prefix(&mut self) {
        self.save_dungeon_keys_misc();
        self.Dungeon_FlagRoomData_Quadrants();
        self.sprite_disable_all();
        self.sprite_reset_all_no_disable_fixed_prefix();
    }

    pub(super) fn wall_master_send_player_after_reset_fixed_prefix(&mut self) {
        self.sprite_reset_all_no_disable_after_fixed_prefix();
        self.wall_master_send_player_after_sprite_reset();
    }

    pub(super) fn wall_master_send_player_after_sprite_reset(&mut self) {
        self.clear_restart_check_flag();
        self.set_main_module(17);
        self.set_submodule(0);
        self.clear_bg_vram_load_mode();
        self.reset_some_things_after_death(17);
    }

    pub(super) fn sprite_sfx_queue_sfx1_with_pan(&mut self, k: usize, a: u8) {
        if self
            .game_state
            .system_signals
            .ambient_sound_effect_is_clear()
        {
            let x = self.sprite_slot_view(k).x();
            let sfx = a | Self::calculate_sfx_pan_with_scroll(
                x,
                self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
            );
            self.set_ambient_sound_effect(sfx);
        }
    }

    /// ROM: `SaveDungeonKeys_misc`
    pub(super) fn save_dungeon_keys_misc(&mut self) {
        let idx = self.game_state.inventory.save_progress.palace_index_x2();
        if idx == 0xff {
            return;
        }
        let slot = if idx == 2 { 0 } else { (idx >> 1) as usize };
        let keys = self.game_state.inventory.player_resources.keys();
        self.dungeon_key_slots_mut()
            .set_keys_earned_slot(slot, keys);
    }

    pub(super) fn handle_item_tile_action_overworld(
        &mut self,
        x: u16,
        y: u16,
    ) -> crate::tile_definition::NativeTile {
        let attribute = if self.game_state.world.location.is_indoors() {
            self.HandleItemTileAction_Dungeon(x, y)
        } else {
            self.overworld_tool_and_tile_interaction(x, y) as u8
        };
        crate::tile_definition::NativeTile::from_cartridge(attribute)
    }
}

pub(super) const DUNG_ANIMATED_TILES: [u8; 24] = [
    0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5f, 0x5d, 0x5f, 0x5f, 0x5e, 0x5f, 0x5e, 0x5e, 0x5d,
    0x5d, 0x5e, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d, 0x5d,
];

const DUNGEON_DEFAULT_ATTR: [u8; 384] = [
    1, 1, 1, 0, 2, 1, 2, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 1, 0, 0, 2, 0, 0, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0,
    0, 0, 0, 0x2a, 1, 0x20, 1, 1, 4, 1, 1, 0x18, 1, 2, 0x1c, 1, 0x28, 0x28, 0x2a, 0x2a, 1, 2, 1, 1,
    4, 0, 0, 0, 0x28, 1, 0xa, 0, 1, 1, 0xc, 0xc, 2, 2, 2, 2, 0x28, 0x2a, 0x20, 0x20, 0x20, 2, 8, 0,
    4, 4, 1, 1, 1, 2, 2, 2, 0, 0, 0x20, 0x20, 0, 2, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x18, 0x10, 0x10, 1, 1, 1, 1, 1, 4, 4, 4, 4, 4, 4, 1, 2, 2,
    0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0x62, 0x62, 0, 0, 0x24, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x62, 0x62, 0x27, 2, 2, 2,
    0x27, 0x27, 1, 0, 0, 0, 0, 0x24, 0, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x10, 2, 1, 0, 0, 0,
    0x24, 0, 0, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 2, 2, 2, 0x24, 0, 0, 0, 0, 0x27, 0x27,
    0x27, 0x27, 0x27, 0x20, 2, 2, 1, 2, 2, 0x23, 2, 0, 0, 0, 0x27, 0x27, 0x27, 0x27, 0x27, 0x20, 2,
    0x27, 2, 0x54, 0, 0, 0x27, 2, 2, 2, 0x27, 0x27, 0x27, 0x27, 0x27, 0x27, 2, 0x27, 2, 0x54, 0, 0,
    0x27, 2, 2, 2, 0x27, 0x27, 0, 0x27, 0x60, 0x60, 1, 1, 1, 1, 2, 2, 0xd, 0, 0, 0x4b, 0x67, 0x67,
    0x67, 0x67, 0x66, 0x66, 0x66, 0x66, 0, 0, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x27, 0x63, 0x27,
    0x55, 0x55, 1, 0x44, 0, 1, 0x20, 2, 2, 0x1c, 0x3a, 0x3b, 0, 0x27, 0x63, 0x27, 0x53, 0x53, 1,
    0x44, 1, 0xd, 0, 0, 0, 9, 9, 9, 9,
];

impl ZeldaState {
    pub(super) fn calculate_sfx_pan_with_scroll(x: u16, bg2hofs: u16) -> u8 {
        let relative = x.wrapping_sub(bg2hofs.wrapping_add(80));
        if relative < 80 {
            0
        } else if (relative as i16) >= 0 {
            0x40
        } else {
            0x80
        }
    }

    pub(super) fn link_calculate_sfx_pan(&self) -> u8 {
        Self::calculate_sfx_pan_with_scroll(
            self.game_state.player.follower_link.x(),
            self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        )
    }

    pub(super) fn play_sfx_set_pan(&mut self, a: u8) -> u8 {
        self.set_raw_sfx_pan_value(a);

        a | self.link_calculate_sfx_pan()
    }

    pub(super) fn ancilla_sfx2_near(&mut self, a: u8) -> u8 {
        let panned = self.play_sfx_set_pan(a);
        self.set_sound_effect_1(panned);
        panned
    }

    pub(super) fn ancilla_sfx3_near(&mut self, a: u8) {
        let panned = self.play_sfx_set_pan(a);
        self.set_sound_effect_2(panned);
    }

    pub(super) fn calculate_sfx_pan_arbitrary(&self, a: u8) -> u8 {
        ARBITRARY_SFX_PAN_BY_SCREEN_COLUMN[((a
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low())
            >> 5)
            & 7) as usize]
    }

    pub(super) fn sprite_calculate_sfx_pan(&self, k: usize) -> u8 {
        let x = self.sprite_slot_view(k).x();
        Self::calculate_sfx_pan_with_scroll(
            x,
            self.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        )
    }

    pub(super) fn sprite_sfx_queue_sfx2_with_pan(&mut self, k: usize, a: u8) {
        if !self.game_state.system_signals.has_sound_effect_1() {
            let sfx = a | self.sprite_calculate_sfx_pan(k);
            self.set_sound_effect_1(sfx);
        }
    }

    #[track_caller]
    pub(super) fn sprite_sfx_queue_sfx3_with_pan(&mut self, k: usize, a: u8) {
        if !self.game_state.system_signals.has_sound_effect_2() {
            let sfx = a | self.sprite_calculate_sfx_pan(k);
            self.set_sound_effect_2(sfx);
        }
    }

    pub(super) fn main_show_text_message(&mut self) {
        let _scope = crate::cycle_ledger::routine(0x0f_fdaa);
        crate::cycle_ledger::charge(56); // Main-module load, compare and BEQ.
        if self.game_state.frame.main_module != 14 {
            crate::cycle_ledger::charge(200); // Reset text state and publish module 14/2.
            self.clear_tile_interaction_shared_flag();
            self.messaging_state_mut().clear_module();
            self.set_submodule(2);
            self.save_main_module_for_menu();
            self.set_main_module(14);
        } else {
            crate::cycle_ledger::charge(6);
        }
        crate::cycle_ledger::charge(44); // RTL.
    }

    pub(super) fn nmi_prepare_sprites(&mut self) {
        self.enter_main_loop_sprite_preparation();
        if crate::debug_env::var_os("ZELDA3_DEBUG_PREP").is_some() {
            eprintln!(
                "[PREP] replay_frame={} fc_dbg={} fc={} cd={:04x} mod={:02x}/{:02x}",
                self.state_recorder.replay_frame_counter,
                self.frame_ctr_dbg,
                self.game_state.frame.frame_counter,
                self.game_state.display.bg_tile_animation_countdown,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            if crate::debug_env::var("ZELDA3_DEBUG_PREP_BT")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|target| target == self.frame_ctr_dbg)
            {
                eprintln!("{}", std::backtrace::Backtrace::force_capture());
            }
        }
        // Cycle ledger: NMI_PrepareSprites ($00:85FC), entered m8 x8 from the
        // main loop. $00:85FC `LDY #$1C` is 16; the packing loop and the
        // source-table suffix charge themselves below.
        let ledger_before = crate::cycle_ledger::master();
        {
            let _scope = crate::cycle_ledger::routine(0x00_85fc);
            let mut work = SpritePreparationWork::default();
            while self.step_sprite_preparation_work(&mut work).is_some() {}
        }
        // The dialogue caller-suffix threshold prices the next
        // NMI_PrepareSprites by this complete call's charge.
        self.last_nmi_prepare_sprites_master_cycles = Some(
            u32::try_from(crate::cycle_ledger::master() - ledger_before).unwrap_or(u32::MAX),
        );
    }

    pub(super) fn step_sprite_preparation_work(&mut self, work: &mut SpritePreparationWork) -> Option<u32> {
        let cycles = match work {
            SpritePreparationWork::Packing(child) => {
                let instruction = child.step(|index| self.game_state.oam.extended_byte(index)).expect("active OAM packing");
                if let Some((index, value)) = instruction.store {
                    self.oam_state_mut().set_packed_extended_oam_byte(index, value);
                }
                if child.is_complete() { *work = SpritePreparationWork::Sources(Default::default()); }
                instruction.master_cycles
            }
            SpritePreparationWork::Sources(child) => {
                let instruction = self.step_sprite_dma_source_words(child).expect("active sprite sources");
                if let Some((index, value)) = instruction.store {
                    self.display_core_mut().set_sprite_preparation_source_word(index, value);
                }
                if child.is_complete() {
                    let (x, y) = child.return_indices();
                    if let Some(clock) = self.native_dialogue_caller_clock.as_mut() {
                        clock.return_x = Some(x);
                        clock.return_y = Some(y);
                    }
                    *work = SpritePreparationWork::Animations(
                        crate::cycle_models::sprite_dma::Animations::with_entry_index(x));
                }
                instruction.master_cycles
            }
            SpritePreparationWork::Animations(child) => {
                let instruction = self.step_sprite_dma_animations(child).expect("active sprite animation");
                if child.is_complete() {
                    if let Some(clock) = self.native_dialogue_caller_clock.as_mut() {
                        clock.return_x = Some(child.return_index());
                    }
                    *work = SpritePreparationWork::Pointers(Default::default());
                }
                instruction.master_cycles
            }
            SpritePreparationWork::Pointers(child) => {
                let instruction = self.step_sprite_dma_pointer_words(child).expect("active sprite pointers");
                if let Some((index, value)) = instruction.store {
                    self.display_core_mut().set_sprite_preparation_pointer_word(index, value);
                }
                if child.is_complete() {
                    if let Some(clock) = self.native_dialogue_caller_clock.as_mut() {
                        let (a, carry, overflow) = child.return_registers();
                        clock.return_a = Some(a);
                        clock.return_carry = carry;
                        clock.return_overflow = overflow;
                    }
                    *work = SpritePreparationWork::Complete;
                }
                instruction.master_cycles
            }
            SpritePreparationWork::Complete => return None,
        };
        crate::cycle_ledger::charge(u64::from(cycles));
        self.advance_native_dialogue_caller_clock(cycles);
        Some(cycles)
    }

    fn nmi_prepare_sprites_pack_extended_oam_group(&mut self, group_start: usize) {
        debug_assert!(group_start <= 28 && group_start & 3 == 0);
        // $00:85FE-$00:865A, one pass of the extended-OAM packing loop: 1128
        // (X = Y*4 never carries `LDA $0A23,X` past the $0A page). The closing
        // `BPL $0085FE` is taken (+6) after every group except the last
        // (Y = 0 - 4 sets N).
        let mut work = crate::cycle_models::oam_pack::PackingGroup::new(group_start as u8);
        while let Some(instruction) = work.step(|index| self.game_state.oam.extended_byte(index)) {
            self.retire_extended_oam_packing_instruction(instruction);
        }
    }

    fn retire_extended_oam_packing_instruction(&mut self, instruction: crate::cycle_models::oam_pack::Instruction) {
        crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
        if let Some((index, value)) = instruction.store {
            self.oam_state_mut().set_packed_extended_oam_byte(index, value);
        }
    }

    /// Execute the original 65816 extended-OAM packing groups which completed
    /// before `next_group_start`. The 65816 loop visits `28, 24, ..., 0`.
    pub(super) fn nmi_prepare_sprites_through_extended_oam_packing(
        &mut self,
        next_group_start: u8,
    ) {
        self.nmi_prepare_sprites_through_packing_progress(
            ExtendedOamPackingProgress::before_group(next_group_start));
    }

    pub(super) fn nmi_prepare_sprites_through_packing_progress(
        &mut self, progress: ExtendedOamPackingProgress,
    ) {
        progress.validate();
        self.enter_main_loop_sprite_preparation();
        // Cycle ledger: the prefix of NMI_PrepareSprites ($00:85FC) up to the
        // interrupted packing group; the resume below records the remainder
        // under the same address, so a sliced call appears as two entries.
        let _scope = crate::cycle_ledger::routine(0x00_85fc);
        crate::cycle_ledger::charge(16);
        for group_start in [28usize, 24, 20, 16, 12, 8, 4, 0] {
            if group_start == usize::from(progress.group_start) {
                break;
            }
            self.nmi_prepare_sprites_pack_extended_oam_group(group_start);
        }
        crate::cycle_ledger::charge(u64::from(progress.group_master_cycles));
        for byte in 0..progress.completed_bytes {
            let i = usize::from(progress.group_start + byte);
            let value = self.game_state.oam.packed_extended_oam_byte(i);
            self.oam_state_mut().set_packed_extended_oam_byte(i, value);
        }
    }

    /// Resume the remaining pure packing groups, then execute the stateful
    /// `NMI_PrepareSprites` suffix exactly once.
    pub(super) fn nmi_prepare_sprites_resume_after_extended_oam_packing(
        &mut self,
        next_group_start: u8,
    ) {
        self.nmi_prepare_sprites_resume_after_packing_progress(
            ExtendedOamPackingProgress::before_group(next_group_start));
    }

    pub(super) fn nmi_prepare_sprites_resume_after_packing_progress(
        &mut self, progress: ExtendedOamPackingProgress,
    ) {
        progress.validate();
        // Cycle ledger: the remainder of a sliced NMI_PrepareSprites
        // ($00:85FC); the prefix was recorded by
        // `nmi_prepare_sprites_through_extended_oam_packing`.
        let _scope = crate::cycle_ledger::routine(0x00_85fc);
        let group_total = if progress.group_start == 0 { 1128 } else { 1134 };
        crate::cycle_ledger::charge(group_total - u64::from(progress.group_master_cycles));
        for byte in progress.completed_bytes..4 {
            let i = usize::from(progress.group_start + byte);
            let value = self.game_state.oam.packed_extended_oam_byte(i);
            self.oam_state_mut().set_packed_extended_oam_byte(i, value);
        }
        for group_start in [28usize, 24, 20, 16, 12, 8, 4, 0]
            .into_iter()
            .skip_while(|&group_start| group_start >= usize::from(progress.group_start))
        {
            self.nmi_prepare_sprites_pack_extended_oam_group(group_start);
        }
        self.nmi_prepare_sprites_after_extended_oam();
    }

    pub(super) fn nmi_prepare_sprites_through_progress(&mut self, progress: SpritePreparationProgress) {
        self.enter_main_loop_sprite_preparation();
        match progress {
            SpritePreparationProgress::ExtendedOam(progress) =>
                self.nmi_prepare_sprites_through_packing_progress(progress),
            SpritePreparationProgress::SourceWords(progress) => {
                let _scope = crate::cycle_ledger::routine(0x00_85fc);
                crate::cycle_ledger::charge(16);
                for group in [28, 24, 20, 16, 12, 8, 4, 0] {
                    self.nmi_prepare_sprites_pack_extended_oam_group(group);
                }
                assert!(progress.completed_words <= 14);
                crate::cycle_ledger::charge(u64::from(progress.master_cycles));
                self.nmi_prepare_sprites_publish_source_words(0, usize::from(progress.completed_words));
            }
            SpritePreparationProgress::PointerTail(progress) => {
                let _scope = crate::cycle_ledger::routine(0x00_85fc);
                crate::cycle_ledger::charge(16);
                for group in [28, 24, 20, 16, 12, 8, 4, 0] {
                    self.nmi_prepare_sprites_pack_extended_oam_group(group);
                }
                self.nmi_prepare_sprites_before_pointer_tail();
                crate::cycle_ledger::charge(u64::from(progress.master_cycles));
                self.nmi_prepare_sprites_publish_pointer_words(0, progress.completed_words());
            }
        }
    }

    pub(super) fn nmi_prepare_sprites_resume_after_progress(&mut self, progress: SpritePreparationProgress) {
        match progress {
            SpritePreparationProgress::ExtendedOam(progress) =>
                self.nmi_prepare_sprites_resume_after_packing_progress(progress),
            SpritePreparationProgress::SourceWords(progress) => {
                let _scope = crate::cycle_ledger::routine(0x00_85fc);
                self.nmi_prepare_sprites_before_pointer_tail_from_source_progress(progress);
                crate::cycle_ledger::charge(610);
                self.nmi_prepare_sprites_publish_pointer_words(0, 6);
            }
            SpritePreparationProgress::PointerTail(progress) => {
                let _scope = crate::cycle_ledger::routine(0x00_85fc);
                let completed = progress.completed_words();
                crate::cycle_ledger::charge(610 - u64::from(progress.master_cycles));
                self.nmi_prepare_sprites_publish_pointer_words(completed, 6);
            }
        }
    }

    fn nmi_prepare_sprites_after_extended_oam(&mut self) {
        self.nmi_prepare_sprites_before_pointer_tail();
        let mut work = crate::cycle_models::sprite_dma::PointerWords::default();
        while let Some(instruction) = self.step_sprite_dma_pointer_words(&mut work) {
            crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
            if let Some((index, value)) = instruction.store {
                self.display_core_mut().set_sprite_preparation_pointer_word(index, value);
            }
        }
    }

    fn nmi_prepare_sprites_before_pointer_tail(&mut self) {
        let mut work = crate::cycle_models::sprite_dma::SourceWords::default();
        while let Some(instruction) = self.step_sprite_dma_source_words(&mut work) {
            crate::cycle_ledger::charge(u64::from(instruction.master_cycles));
            if let Some((index, value)) = instruction.store {
                self.display_core_mut().set_sprite_preparation_source_word(index, value);
            }
        }
        self.nmi_prepare_sprites_animations(0);
    }

    fn step_sprite_dma_source_words(
        &self, work: &mut crate::cycle_models::sprite_dma::SourceWords,
    ) -> Option<crate::cycle_models::sprite_dma::Instruction> {
        use crate::cycle_models::sprite_dma::Operand;
        let link = &self.game_state.player.follower_link;
        work.step(|operand| match operand {
            Operand::Graphics => link.link_dma_graphics_index_word(),
            Operand::LeftHand => link.link_dma_left_sprite_bank_word(),
            Operand::RightHand => link.link_dma_right_sprite_bank_word(),
            Operand::Sword => u16::from(link.sword_dma_graphics_index()),
            Operand::Shield => u16::from(link.shield_dma_graphics_index()),
            // Both source LDA instructions are m16, but their consumers mask
            // the high byte (AND #$00F8, or ASL followed by eight-bit TAX).
            Operand::Staging => u16::from(link.link_dma_staging_index()),
            Operand::PushedBlock => u16::from(self.game_state.player.pushed_block.animation_mode()),
        }, link_dma_table_word)
    }

    fn nmi_prepare_sprites_source_words(&self) -> [u16; 14] {
        let mut work = crate::cycle_models::sprite_dma::SourceWords::default();
        let mut words = [0; 14];
        while let Some(instruction) = self.step_sprite_dma_source_words(&mut work) {
            if let Some((index, value)) = instruction.store {
                words[index] = value;
            }
        }
        words
    }

    fn nmi_prepare_sprites_publish_source_words(&mut self, first: usize, end: usize) {
        let words = self.nmi_prepare_sprites_source_words();
        for index in first..end {
            self.display_core_mut().set_sprite_preparation_source_word(index, words[index]);
        }
    }

    fn nmi_prepare_sprites_before_pointer_tail_from_source_progress(
        &mut self, progress: SpritePreparationSourceProgress,
    ) {
        assert!(progress.completed_words <= 14);
        // $865C-$86E8 includes the source stores and first animation decrement.
        // Preserve the existing complete charge, retiring only the measured
        // instruction prefix on the interrupted field. Indexed page crossings
        // in the sword, shield, and auxiliary tables add six cycles each.
        let link = &self.game_state.player.follower_link;
        let aux_index = link.link_dma_staging_index().wrapping_mul(2) & 0xff;
        let master_cycles = 1768
            + if link.sword_dma_graphics_index() >= 0x64 { 6 } else { 0 }
            + if link.shield_dma_graphics_index() >= 0x54 { 6 } else { 0 }
            + if aux_index >= 0x4e { 6 } else { 0 };
        assert!(u64::from(progress.master_cycles) <= master_cycles);
        crate::cycle_ledger::charge(master_cycles - u64::from(progress.master_cycles));
        self.nmi_prepare_sprites_publish_source_words(usize::from(progress.completed_words), 14);

        self.nmi_prepare_sprites_animations(126);
    }

    fn step_sprite_dma_animations(
        &mut self, work: &mut crate::cycle_models::sprite_dma::Animations,
    ) -> Option<crate::cycle_models::sprite_dma::AnimationInstruction> {
        use crate::cycle_models::sprite_dma::{AnimationOperand as Read, AnimationStore as Store};
        let instruction = work.step(|operand| match operand {
            Read::BackgroundCountdown => self.game_state.display.bg_tile_animation_countdown,
            Read::Overlay => u16::from(self.game_state.world.region.overlay_index()),
            Read::BackgroundOffset => self.player_state_mut().link_dma_source_offset(),
            Read::LinkCountdown => self.player_state_mut().link_dma_countdown(),
            Read::LinkOffset => self.player_state_mut().link_dma_tile_offset(),
        }, link_dma_table_word)?;
        if let Some((destination, value)) = instruction.store {
            match destination {
                Store::BackgroundCountdown => self.reset_bg_tile_animation_countdown(value),
                Store::BackgroundOffset => self.player_state_mut().set_link_dma_source_offset(value),
                Store::BackgroundSource => self.set_animated_tile_data_source_address(value),
                Store::LinkCountdown => self.player_state_mut().set_link_dma_countdown(value),
                Store::LinkOffset => self.player_state_mut().set_link_dma_tile_offset(value),
                Store::LinkSourceUpper => self.display_core_mut().set_sprite_preparation_animation_word(0, value),
                Store::LinkSourceLower => self.display_core_mut().set_sprite_preparation_animation_word(1, value),
            }
        }
        Some(instruction)
    }

    fn nmi_prepare_sprites_animations(&mut self, mut prepaid_master_cycles: u32) {
        let mut work = crate::cycle_models::sprite_dma::Animations::default();
        while let Some(instruction) = self.step_sprite_dma_animations(&mut work) {
            // Only the legacy source-progress adapter has already charged the
            // first countdown's load/decrement/store and untaken branch.
            let prepaid = prepaid_master_cycles.min(instruction.master_cycles);
            prepaid_master_cycles -= prepaid;
            crate::cycle_ledger::charge(u64::from(instruction.master_cycles - prepaid));
        }
        assert_eq!(prepaid_master_cycles, 0);
    }

    fn step_sprite_dma_pointer_words(
        &mut self, work: &mut crate::cycle_models::sprite_dma::PointerWords,
    ) -> Option<crate::cycle_models::sprite_dma::Instruction> {
        work.step(|pointer| self.display_core_mut().sprite_preparation_pointer_word(pointer))
    }

    fn nmi_prepare_sprites_pointer_words(&mut self) -> [u16; 6] {
        let mut work = crate::cycle_models::sprite_dma::PointerWords::default();
        let mut words = [0; 6];
        while let Some(instruction) = self.step_sprite_dma_pointer_words(&mut work) {
            if let Some((index, value)) = instruction.store { words[index] = value; }
        }
        words
    }

    fn nmi_prepare_sprites_publish_pointer_words(&mut self, first: usize, end: usize) {
        let words = self.nmi_prepare_sprites_pointer_words();
        for word in first..end {
            self.display_core_mut().set_sprite_preparation_pointer_word(word, words[word]);
        }
    }


    pub(super) fn module_main_routing(&mut self) {
        // `Module_MainRouting` (`$00:80B5`): `LDY $10`, three `LDA $80xx,Y
        // : STA $0x` table loads of the module's long pointer (Y is at most
        // 27, no page crossing) and `JML [$03]`: 24 + 3 x (32 + 24) + 48.
        // A host that resumes a suspended dialogue call stack re-enters the
        // translated module through here, but the ROM's RTI returns straight
        // into the suspended routine: no routing runs.
        if !self.dialogue_fast_forward_hold_active {
            crate::cycle_ledger::charge_routine(0x00_80b5, 240);
        }
        match self.game_state.frame.main_module {
            0 => self.module00_intro(),
            1 => self.module01_file_select(),
            2 => self.module02_copy_file(),
            3 => self.module03_kill_file(),
            4 => self.module04_name_file(),
            5 => self.module05_load_file(),
            6 => self.module_pre_dungeon(),
            7 => self.module07_dungeon(),
            8 | 10 => self.module08_overworld_load(),
            9 | 11 => self.module09_overworld(),
            12 => self.module_reserved_12(),
            13 => self.module_reserved_13(),
            14 => self.module0e_interface(),
            15 => self.module0f_spotlight_close(),
            16 => self.module10_spotlight_open(),
            17 => self.Module11_DungeonFallingEntrance(),
            18 => self.module12_game_over(),
            19 => self.module13_boss_victory_pendant(),
            20 => self.module14_attract(),
            21 => self.module15_mirror_warp_from_aga(),
            22 => self.module16_boss_victory_crystal(),
            23 => self.module17_save_and_quit(),
            24 => self.module18_ganon_emerges(),
            25 => self.module19_triforce_room(),
            26 => self.module1_a_credits(),
            27 => self.module1b_spawn_select(),
            _ => {}
        }
    }
}
