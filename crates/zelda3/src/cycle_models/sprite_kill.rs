//! Native Sprite_KillSelf ($09:F1F8), including its overlapping scratch
//! pointer and the loaded-mask read/modify/write. Each step retires one
//! source instruction; loaded operands survive suspension and serialization.

#[derive(Clone, Copy)]
pub(crate) struct Inputs {
    pub slot: u8,
    pub deflection: u8,
    pub indoors: u8,
    pub block: u16,
    pub scratch: [u8; 4],
    pub loaded: u8,
}

pub(crate) enum Publication {
    ClearState,
    ScratchWord { offset: usize, value: u16 },
    PointerBank(u8),
    Loaded(u8),
    BlockLow(u8),
    BlockHigh(u8),
    IndoorBlock(u8),
}

pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    LoadDeflection, MaskDeflection, TestDeflection, LoadGuard, TestGuard,
    ClearState, LoadSlot, DoubleSlot, SelectBlock, Widen, LoadBlock, StoreBlock,
    CompareBlock, SaveCarry, Shift1, Shift2, Shift3, ClearCarry, AddBase,
    StorePointer, RestoreCarry, Narrow, TestBlock, LoadBank, StoreBank,
    SaveSlot, LoadBit, MaskBit, SelectBit, LoadLoaded, MaskLoaded, StoreLoaded,
    RestoreSlot, LoadIndoors, TestIndoors, LoadSlotAgain, DoubleSlotAgain,
    SelectBlockAgain, LoadAbsent, StoreLow, StoreHigh, ReturnOutdoor,
    LoadIndoorAbsent, StoreIndoor, Return, Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct KillWork {
    stage: Stage,
    value: u16,
    bit: u8,
    absent: bool,
}

impl KillWork {
    pub(crate) fn new() -> Self {
        Self { stage: Stage::LoadDeflection, value: 0, bit: 0, absent: false }
    }

    pub(crate) fn reads_loaded(&self) -> bool { self.stage == Stage::LoadLoaded }

    pub(crate) fn step(&mut self, input: Inputs) -> Option<Instruction> {
        use Stage::*;
        let mut publication = None;
        macro_rules! go { ($pc:expr, $cycles:expr, $next:expr) => { ($pc, $cycles, $next) }; }
        let (pc, master_cycles, next) = match self.stage {
            LoadDeflection => { self.value = u16::from(input.deflection); go!(0xf1f8, 32, MaskDeflection) }
            MaskDeflection => { self.value &= 0x40; go!(0xf1fb, 16, TestDeflection) }
            TestDeflection => go!(0xf1fd, if self.value != 0 { 22 } else { 16 }, if self.value != 0 { ClearState } else { LoadGuard }),
            LoadGuard => { self.value = u16::from(input.indoors); go!(0xf1ff, 24, TestGuard) }
            TestGuard => go!(0xf201, if self.value != 0 { 22 } else { 16 }, if self.value != 0 { Return } else { ClearState }),
            ClearState => { publication = Some(Publication::ClearState); go!(0xf203, 38, LoadSlot) }
            LoadSlot => { self.value = u16::from(input.slot); go!(0xf206, 14, DoubleSlot) }
            DoubleSlot => { self.value *= 2; go!(0xf207, 14, SelectBlock) }
            SelectBlock => go!(0xf208, 14, Widen),
            Widen => go!(0xf209, 22, LoadBlock),
            LoadBlock => { self.value = input.block; go!(0xf20b, 40, StoreBlock) }
            StoreBlock => { publication = Some(Publication::ScratchWord { offset: 0, value: self.value }); go!(0xf20e, 32, CompareBlock) }
            CompareBlock => { self.absent = self.value == 0xffff; go!(0xf210, 24, SaveCarry) }
            SaveCarry => go!(0xf213, 22, Shift1),
            Shift1 => { self.value >>= 1; go!(0xf214, 14, Shift2) }
            Shift2 => { self.value >>= 1; go!(0xf215, 14, Shift3) }
            Shift3 => { self.value >>= 1; go!(0xf216, 14, ClearCarry) }
            ClearCarry => go!(0xf217, 14, AddBase),
            AddBase => { self.value = self.value.wrapping_add(0xef80); go!(0xf218, 24, StorePointer) }
            StorePointer => { publication = Some(Publication::ScratchWord { offset: 1, value: self.value }); go!(0xf21b, 32, RestoreCarry) }
            RestoreCarry => go!(0xf21d, 28, Narrow),
            Narrow => { self.value &= 255; go!(0xf21e, 22, TestBlock) }
            TestBlock => go!(0xf220, if self.absent { 22 } else { 16 }, if self.absent { LoadIndoors } else { LoadBank }),
            LoadBank => { self.value = 0x7f; go!(0xf222, 16, StoreBank) }
            StoreBank => { publication = Some(Publication::PointerBank(self.value as u8)); go!(0xf224, 24, SaveSlot) }
            SaveSlot => go!(0xf226, 22, LoadBit),
            LoadBit => { self.value = u16::from(input.scratch[0]); go!(0xf227, 24, MaskBit) }
            MaskBit => { self.value &= 7; go!(0xf229, 16, SelectBit) }
            SelectBit => { self.bit = self.value as u8; go!(0xf22b, 14, LoadLoaded) }
            LoadLoaded => { self.value = u16::from(input.loaded); go!(0xf22c, 48, MaskLoaded) }
            MaskLoaded => { self.value &= u16::from(!(0x80u8 >> self.bit)); go!(0xf22e, 40, StoreLoaded) }
            StoreLoaded => { publication = Some(Publication::Loaded(self.value as u8)); go!(0xf232, 48, RestoreSlot) }
            RestoreSlot => go!(0xf234, 28, LoadIndoors),
            LoadIndoors => { self.value = u16::from(input.indoors); go!(0xf235, 24, TestIndoors) }
            TestIndoors => go!(0xf237, if self.value != 0 { 22 } else { 16 }, if self.value != 0 { LoadIndoorAbsent } else { LoadSlotAgain }),
            LoadSlotAgain => { self.value = u16::from(input.slot); go!(0xf239, 14, DoubleSlotAgain) }
            DoubleSlotAgain => { self.value *= 2; go!(0xf23a, 14, SelectBlockAgain) }
            SelectBlockAgain => go!(0xf23b, 14, LoadAbsent),
            LoadAbsent => { self.value = 0xff; go!(0xf23c, 16, StoreLow) }
            StoreLow => { publication = Some(Publication::BlockLow(self.value as u8)); go!(0xf23e, 38, StoreHigh) }
            StoreHigh => { publication = Some(Publication::BlockHigh(self.value as u8)); go!(0xf241, 38, ReturnOutdoor) }
            ReturnOutdoor => go!(0xf244, 44, Complete),
            LoadIndoorAbsent => { self.value = 0xff; go!(0xf245, 16, StoreIndoor) }
            StoreIndoor => { publication = Some(Publication::IndoorBlock(self.value as u8)); go!(0xf247, 38, Return) }
            Return => go!(0xf24a, 44, Complete),
            Complete => return None,
        };
        self.stage = next;
        let _ = pc;
        Some(Instruction { master_cycles, publication, #[cfg(test)] pc: 0x09_0000 | pc })
    }
}
