//! Source-ordered Sprite_PrepOamCoordOrDoubleRet ($06:E41E). The caller
//! retains the optional KillSelf callee separately from this return path.

#[derive(Clone, Copy)]
pub(crate) struct Inputs {
    pub world_x: u16,
    pub world_y: u16,
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub height: u8,
    pub flags4: u8,
    pub deflection: u8,
    pub flags: u8,
    pub priority: u8,
    pub pause: u8,
    pub scratch: [u8; 6],
    pub extended_screen: bool,
}

pub(crate) enum Publication {
    ScratchByte { offset: usize, value: u8 },
    ScratchWord { offset: usize, value: u16 },
    Pause(u8),
    DrawX(u8),
    DrawY(u8),
    EnterKill,
}

pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    ClearPause, WidenX, LoadX, SetXCarry, SubtractX, StoreX, ClearXCarry,
    AddXMargin, CompareX, NarrowX, TestX, LoadHeight, StoreHeight, ClearHeightHigh,
    WidenY, LoadY, SetYCarry, SubtractY, SaveY, SetHeightCarry, SubtractHeight,
    StoreY, RestoreY, ClearYCarry, AddYMargin, CompareY, NarrowY, TestY,
    LoadFlags4, MaskFlags4, TestFlags4, ClearReturnCarry, LoadFlags, XorPriority,
    StoreFlags, ClearScratch, LoadDrawX, StoreDrawX, LoadDrawY, StoreDrawY,
    ClearIndex, Return, WidenOffscreen, LoadOffscreenY, SetOffscreenCarry,
    SubtractOffscreenY, SetOffscreenHeightCarry, SubtractOffscreenHeight,
    StoreOffscreenY, NarrowOffscreen, IncrementPause, LoadDeflection,
    TestDeflection, CallKill, DiscardReturnLow, DiscardReturnHigh,
    SetReturnCarry, JumpReturn, Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PrepWork {
    stage: Stage,
    value: u16,
    saved_y: u16,
    carry: bool,
    out: bool,
}

impl PrepWork {
    pub(crate) fn new() -> Self {
        Self { stage: Stage::ClearPause, value: 0, saved_y: 0, carry: false, out: false }
    }

    pub(crate) fn out_of_bounds(&self) -> bool { self.out }

    pub(crate) fn step(&mut self, input: Inputs) -> Option<Instruction> {
        use Stage::*;
        let mut publication = None;
        let word = |offset| u16::from_le_bytes([input.scratch[offset], input.scratch[offset + 1]]);
        macro_rules! go { ($pc:expr, $cycles:expr, $next:expr) => { ($pc, $cycles, $next) }; }
        macro_rules! byte { ($offset:expr) => { publication = Some(Publication::ScratchByte { offset: $offset, value: self.value as u8 }) }; }
        macro_rules! word { ($offset:expr) => { publication = Some(Publication::ScratchWord { offset: $offset, value: self.value }) }; }
        let (pc, master_cycles, next) = match self.stage {
            ClearPause => { publication = Some(Publication::Pause(0)); go!(0xe41e, 38, WidenX) }
            WidenX => go!(0xe421, 22, LoadX),
            LoadX => { self.value = input.world_x; go!(0xe423, 40, SetXCarry) }
            SetXCarry => go!(0xe426, 14, SubtractX),
            SubtractX => { self.value = self.value.wrapping_sub(input.scroll_x); go!(0xe427, 32, StoreX) }
            StoreX => { word!(0); go!(0xe429, 32, ClearXCarry) }
            ClearXCarry => go!(0xe42b, 14, AddXMargin),
            AddXMargin => { self.value = self.value.wrapping_add(if input.extended_screen { 0x80 } else { 0x40 }); go!(0xe42c, 24, CompareX) }
            CompareX => { self.carry = self.value >= if input.extended_screen { 0x1f0 } else { 0x170 }; go!(0xe42f, 24, NarrowX) }
            NarrowX => { self.value &= 255; go!(0xe432, 22, TestX) }
            TestX => go!(0xe434, if self.carry { 22 } else { 16 }, if self.carry { WidenOffscreen } else { LoadHeight }),
            LoadHeight => { self.value = u16::from(input.height); go!(0xe436, 32, StoreHeight) }
            StoreHeight => { byte!(4); go!(0xe439, 24, ClearHeightHigh) }
            ClearHeightHigh => { publication = Some(Publication::ScratchByte { offset: 5, value: 0 }); go!(0xe43b, 24, WidenY) }
            WidenY => go!(0xe43d, 22, LoadY),
            LoadY => { self.value = input.world_y; go!(0xe43f, 40, SetYCarry) }
            SetYCarry => go!(0xe442, 14, SubtractY),
            SubtractY => { self.value = self.value.wrapping_sub(input.scroll_y); go!(0xe443, 32, SaveY) }
            SaveY => { self.saved_y = self.value; go!(0xe445, 30, SetHeightCarry) }
            SetHeightCarry => go!(0xe446, 14, SubtractHeight),
            SubtractHeight => { self.value = self.value.wrapping_sub(word(4)); go!(0xe447, 32, StoreY) }
            StoreY => { word!(2); go!(0xe449, 32, RestoreY) }
            RestoreY => { self.value = self.saved_y; go!(0xe44b, 36, ClearYCarry) }
            ClearYCarry => go!(0xe44c, 14, AddYMargin),
            AddYMargin => { self.value = self.value.wrapping_add(0x40); go!(0xe44d, 24, CompareY) }
            CompareY => { self.carry = self.value >= 0x170; go!(0xe450, 24, NarrowY) }
            NarrowY => { self.value &= 255; go!(0xe453, 22, TestY) }
            TestY => go!(0xe455, if !self.carry { 22 } else { 16 }, if !self.carry { ClearReturnCarry } else { LoadFlags4 }),
            LoadFlags4 => { self.value = u16::from(input.flags4); go!(0xe457, 32, MaskFlags4) }
            MaskFlags4 => { self.value &= 0x20; go!(0xe45a, 16, TestFlags4) }
            TestFlags4 => go!(0xe45c, if self.value == 0 { 22 } else { 16 }, if self.value == 0 { IncrementPause } else { ClearReturnCarry }),
            ClearReturnCarry => go!(0xe45e, 14, LoadFlags),
            LoadFlags => { self.value = u16::from(input.flags); go!(0xe45f, 32, XorPriority) }
            XorPriority => { self.value ^= u16::from(input.priority); go!(0xe462, 32, StoreFlags) }
            StoreFlags => { byte!(5); go!(0xe465, 24, ClearScratch) }
            ClearScratch => { publication = Some(Publication::ScratchByte { offset: 4, value: 0 }); go!(0xe467, 24, LoadDrawX) }
            LoadDrawX => { self.value = u16::from(input.scratch[0]); go!(0xe469, 24, StoreDrawX) }
            StoreDrawX => { publication = Some(Publication::DrawX(self.value as u8)); go!(0xe46b, 32, LoadDrawY) }
            LoadDrawY => { self.value = u16::from(input.scratch[2]); go!(0xe46e, 24, StoreDrawY) }
            StoreDrawY => { publication = Some(Publication::DrawY(self.value as u8)); go!(0xe470, 32, ClearIndex) }
            ClearIndex => go!(0xe473, 16, Return),
            Return => go!(0xe475, 42, Complete),
            WidenOffscreen => go!(0xe476, 22, LoadOffscreenY),
            LoadOffscreenY => { self.value = input.world_y; go!(0xe478, 40, SetOffscreenCarry) }
            SetOffscreenCarry => go!(0xe47b, 14, SubtractOffscreenY),
            SubtractOffscreenY => { self.value = self.value.wrapping_sub(input.scroll_y); go!(0xe47c, 32, SetOffscreenHeightCarry) }
            SetOffscreenHeightCarry => go!(0xe47e, 14, SubtractOffscreenHeight),
            SubtractOffscreenHeight => { self.value = self.value.wrapping_sub(word(4)); go!(0xe47f, 32, StoreOffscreenY) }
            StoreOffscreenY => { word!(2); go!(0xe481, 32, NarrowOffscreen) }
            NarrowOffscreen => { self.value &= 255; go!(0xe483, 22, IncrementPause) }
            IncrementPause => { self.out = true; publication = Some(Publication::Pause(input.pause.wrapping_add(1))); go!(0xe485, 52, LoadDeflection) }
            LoadDeflection => { self.value = u16::from(input.deflection); go!(0xe488, 32, TestDeflection) }
            TestDeflection => go!(0xe48b, if self.value & 0x80 != 0 { 22 } else { 16 }, if self.value & 0x80 != 0 { DiscardReturnLow } else { CallKill }),
            CallKill => { publication = Some(Publication::EnterKill); go!(0xe48d, 62, DiscardReturnLow) }
            DiscardReturnLow => go!(0xe491, 28, DiscardReturnHigh),
            DiscardReturnHigh => go!(0xe492, 28, SetReturnCarry),
            SetReturnCarry => go!(0xe493, 14, JumpReturn),
            JumpReturn => go!(0xe494, 22, LoadFlags),
            Complete => return None,
        };
        self.stage = next;
        let _ = pc;
        Some(Instruction { master_cycles, publication, #[cfg(test)] pc: 0x06_0000 | pc })
    }
}
