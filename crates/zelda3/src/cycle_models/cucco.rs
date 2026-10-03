//! Cucco's native facing and draw-call prefix ($06:A5C2..A5D7).
//! The velocity, selected facing and loaded flags belong to the suspended
//! caller. Admission performs no work before the caller's clock is supplied.

pub(crate) enum Publication {
    Flags(u8),
    EnterDraw,
}

pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    LoadVelocity,
    TestVelocity,
    ShiftVelocity,
    RotateSign,
    MaskSign,
    SelectFacing,
    LoadFlags,
    ClearFacing,
    AddFacing,
    StoreFlags,
    CallDraw,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DrawPrefixWork {
    stage: Stage,
    value: u8,
    carry: bool,
    facing: u8,
}

impl DrawPrefixWork {
    pub(crate) fn new() -> Self {
        Self { stage: Stage::LoadVelocity, value: 0, carry: false, facing: 0 }
    }

    pub(crate) fn is_complete(&self) -> bool { self.stage == Stage::Complete }

    pub(crate) fn step(&mut self, velocity: u8, flags: u8) -> Option<Instruction> {
        use Stage::*;
        let mut publication = None;
        let (pc, master_cycles, next) = match self.stage {
            LoadVelocity => {
                self.value = velocity;
                (0xa5c2, 32, TestVelocity)
            }
            TestVelocity => {
                if self.value == 0 { (0xa5c5, 22, CallDraw) }
                else { (0xa5c5, 16, ShiftVelocity) }
            }
            ShiftVelocity => {
                self.carry = self.value & 0x80 != 0;
                self.value <<= 1;
                (0xa5c7, 14, RotateSign)
            }
            RotateSign => {
                self.value = (self.value << 1) | u8::from(self.carry);
                (0xa5c8, 14, MaskSign)
            }
            MaskSign => { self.value &= 1; (0xa5c9, 16, SelectFacing) }
            SelectFacing => { self.facing = self.value; (0xa5cb, 14, LoadFlags) }
            LoadFlags => { self.value = flags; (0xa5cc, 32, ClearFacing) }
            ClearFacing => { self.value &= 0xbf; (0xa5cf, 16, AddFacing) }
            AddFacing => {
                self.value |= [0x40, 0][usize::from(self.facing)];
                (0xa5d1, 32, StoreFlags)
            }
            StoreFlags => {
                publication = Some(Publication::Flags(self.value));
                (0xa5d4, 38, CallDraw)
            }
            CallDraw => {
                publication = Some(Publication::EnterDraw);
                (0xa5d7, 46, Complete)
            }
            Complete => return None,
        };
        self.stage = next;
        let _ = pc;
        Some(Instruction { master_cycles, publication, #[cfg(test)] pc: 0x06_0000 | pc })
    }
}
