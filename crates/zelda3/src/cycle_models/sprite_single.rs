//! Source-ordered native work for Sprite_PrepAndDrawSingleLargeNoPrep
//! ($06:DC13). The coordinate-preparation call belongs to the caller; the
//! selected shadow body tail-calls the existing shared shadow continuation.

#[derive(Clone, Copy, Debug)]
pub(crate) struct Inputs {
    pub x: u16,
    pub y: u16,
    pub flags: u8,
    pub sprite_type: u8,
    pub graphics: u8,
    pub flags3: u8,
    pub character_base: u8,
    pub character: u8,
    pub oam_pointer: u16,
    pub extended_pointer: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Publication {
    OamByte { address: u16, value: u8 },
    ExtendedByte { address: u16, value: u8 },
    EnterShadow,
}

pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    LoadX, StoreX, LoadXHigh, CompareXHigh, LoadSize, RotateSize, StoreSize,
    Widen, LoadY, NextY, ClearClipCarry, AddClipMargin, CompareClip, Narrow,
    TestClip, RestoreY, StoreY, SaveIndex, LoadType, LoadBase, ClearGraphicsCarry,
    AddGraphics, SelectCharacter, LoadCharacter, RestoreIndex, NextCharacter,
    StoreCharacter, LoadFlags, NextFlags, StoreFlags, LoadFlags3, MaskShadow,
    TestShadow, LoadShadowOffset, Return, Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SingleLargeWork {
    stage: Stage,
    value: u16,
    index: u8,
    saved_index: u8,
    carry: bool,
}

impl SingleLargeWork {
    pub(crate) fn new() -> Self {
        Self { stage: Stage::LoadX, value: 0, index: 0, saved_index: 0, carry: false }
    }

    /// The native caller resolves the ROM tables with this retained Y value,
    /// not by re-reading the sprite type/graphics after an interruption.
    pub(crate) fn table_index(&self) -> usize { usize::from(self.index) }

    pub(crate) fn step(&mut self, input: Inputs) -> Option<Instruction> {
        use Stage::*;
        let mut publication = None;
        macro_rules! go { ($pc:expr, $cycles:expr, $next:expr) => { ($pc, $cycles, $next) }; }
        macro_rules! store {
            () => { publication = Some(Publication::OamByte {
                address: input.oam_pointer.wrapping_add(u16::from(self.index)),
                value: self.value as u8,
            }) };
        }
        let (pc, master_cycles, next) = match self.stage {
            LoadX => { self.value = u16::from(input.x as u8); go!(0xdc13, 24, StoreX) }
            StoreX => { store!(); go!(0xdc15, 46, LoadXHigh) }
            LoadXHigh => { self.value = input.x >> 8; go!(0xdc17, 24, CompareXHigh) }
            CompareXHigh => { self.carry = self.value >= 1; go!(0xdc19, 16, LoadSize) }
            LoadSize => { self.value = 1; go!(0xdc1b, 16, RotateSize) }
            RotateSize => { self.value = 2 | u16::from(self.carry); go!(0xdc1d, 14, StoreSize) }
            StoreSize => {
                publication = Some(Publication::ExtendedByte {
                    address: input.extended_pointer, value: self.value as u8,
                });
                go!(0xdc1e, 40, Widen)
            }
            Widen => go!(0xdc20, 22, LoadY),
            LoadY => { self.value = input.y; go!(0xdc22, 32, NextY) }
            NextY => { self.index = self.index.wrapping_add(1); go!(0xdc24, 14, ClearClipCarry) }
            ClearClipCarry => { self.carry = false; go!(0xdc25, 14, AddClipMargin) }
            AddClipMargin => { self.value = self.value.wrapping_add(16); go!(0xdc26, 24, CompareClip) }
            CompareClip => { self.carry = self.value >= 256; go!(0xdc29, 24, Narrow) }
            Narrow => { self.value &= 255; go!(0xdc2c, 22, TestClip) }
            TestClip => go!(0xdc2e, if self.carry { 22 } else { 16 }, if self.carry { LoadFlags3 } else { RestoreY }),
            RestoreY => {
                self.value = u16::from((self.value as u8).wrapping_sub(15).wrapping_sub(u8::from(!self.carry)));
                go!(0xdc30, 16, StoreY)
            }
            StoreY => { store!(); go!(0xdc32, 46, SaveIndex) }
            SaveIndex => { self.saved_index = self.index; go!(0xdc34, 22, LoadType) }
            LoadType => { self.index = input.sprite_type; go!(0xdc35, 32, LoadBase) }
            LoadBase => {
                self.value = u16::from(input.character_base);
                go!(0xdc38, 32 + if self.index >= 252 { 6 } else { 0 }, ClearGraphicsCarry)
            }
            ClearGraphicsCarry => { self.carry = false; go!(0xdc3b, 14, AddGraphics) }
            AddGraphics => { self.value = u16::from((self.value as u8).wrapping_add(input.graphics)); go!(0xdc3c, 32, SelectCharacter) }
            SelectCharacter => { self.index = self.value as u8; go!(0xdc3f, 14, LoadCharacter) }
            LoadCharacter => {
                self.value = u16::from(input.character);
                go!(0xdc40, 32 + if self.index >= 247 { 6 } else { 0 }, RestoreIndex)
            }
            RestoreIndex => { self.index = self.saved_index; go!(0xdc43, 28, NextCharacter) }
            NextCharacter => { self.index = self.index.wrapping_add(1); go!(0xdc44, 14, StoreCharacter) }
            StoreCharacter => { store!(); go!(0xdc45, 46, LoadFlags) }
            LoadFlags => { self.value = u16::from(input.flags); go!(0xdc47, 24, NextFlags) }
            NextFlags => { self.index = self.index.wrapping_add(1); go!(0xdc49, 14, StoreFlags) }
            StoreFlags => { store!(); go!(0xdc4a, 46, LoadFlags3) }
            LoadFlags3 => { self.value = u16::from(input.flags3); go!(0xdc4c, 32, MaskShadow) }
            MaskShadow => { self.value &= 0x10; go!(0xdc4f, 16, TestShadow) }
            TestShadow => go!(0xdc51, if self.value != 0 { 22 } else { 16 }, if self.value != 0 { LoadShadowOffset } else { Return }),
            LoadShadowOffset => { publication = Some(Publication::EnterShadow); go!(0xdc64, 16, Complete) }
            Return => go!(0xdc53, 42, Complete),
            Complete => return None,
        };
        self.stage = next;
        let _ = pc;
        Some(Instruction { master_cycles, publication, #[cfg(test)] pc: 0x06_0000 | pc })
    }
}
