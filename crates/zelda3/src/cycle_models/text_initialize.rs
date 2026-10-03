//! Native instruction continuations on both sides of Text_LoadCharacterBuffer.
//! Stores and return paths retire separately. No ROM execution supplies progress.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum PrefixPhase {
    Copy { index: u8, instruction: u8 },
    CallVwf,
    Vwf(u8),
    CallWindow,
    Window(u8),
    Tilemap(u8),
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InitializationPrefix {
    phase: PrefixPhase,
    loaded: u16,
    carry: bool,
}

impl Default for InitializationPrefix {
    fn default() -> Self {
        Self {
            phase: PrefixPhase::Copy {
                index: 0,
                instruction: 0,
            },
            loaded: 0,
            carry: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrefixPublication {
    StateByte { index: u8, value: u8 },
    ClearVwfByte(u8),
    Window(u16),
    Tilemap(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrefixInstruction {
    pub master_cycles: u32,
    pub routine: u32,
    pub publication: Option<PrefixPublication>,
}

impl InitializationPrefix {
    pub(crate) fn is_complete(&self) -> bool {
        self.phase == PrefixPhase::Complete
    }

    /// Entry follows the caller's LDX #0. Live native inputs are sampled only
    /// by their load/subtract instruction; a suspension retains each operand.
    pub(crate) fn step(
        &mut self,
        link_y: u16,
        scroll_y: u16,
        initialization_data: &[u8; 32],
        positions: &[u16; 2],
    ) -> Option<PrefixInstruction> {
        use PrefixPhase::*;
        let mut publication = None;
        let (master_cycles, routine, next) = match self.phase {
            Copy { index, instruction } => {
                let cost = match instruction {
                    0 => {
                        self.loaded = u16::from(initialization_data[usize::from(index)]);
                        32
                    }
                    1 => {
                        publication = Some(PrefixPublication::StateByte {
                            index,
                            value: self.loaded as u8,
                        });
                        38
                    }
                    2 => 14,
                    3 => 16,
                    4 => {
                        if index == 31 {
                            16
                        } else {
                            22
                        }
                    }
                    _ => unreachable!(),
                };
                let next = if instruction != 4 {
                    Copy {
                        index,
                        instruction: instruction + 1,
                    }
                } else if index != 31 {
                    Copy {
                        index: index + 1,
                        instruction: 0,
                    }
                } else {
                    CallVwf
                };
                (cost, 0x0e_c448, next)
            }
            CallVwf => (46, 0x0e_c448, Vwf(0)),
            Vwf(i) => {
                if i < 8 {
                    publication = Some(PrefixPublication::ClearVwfByte(
                        [2, 3, 0, 1, 4, 5, 6, 7][usize::from(i)],
                    ));
                }
                (
                    if i == 8 { 42 } else { 32 },
                    0x0e_c4c9,
                    if i == 8 { CallWindow } else { Vwf(i + 1) },
                )
            }
            CallWindow => (46, 0x0e_c448, Window(0)),
            Window(i) => {
                let cost = [22, 32, 14, 32, 24, 14, 24, 24, 14, 14, 46, 40, 22, 42][usize::from(i)];
                match i {
                    1 => self.loaded = link_y,
                    3 => self.loaded = self.loaded.wrapping_sub(scroll_y),
                    4 => self.carry = self.loaded >= 0x78,
                    5 => self.loaded = (self.loaded << 1) | u16::from(self.carry),
                    6 => self.loaded ^= 1,
                    7 => self.loaded &= 1,
                    8 => self.loaded <<= 1,
                    10 => self.loaded = positions[usize::from(self.loaded >> 1)],
                    11 => publication = Some(PrefixPublication::Window(self.loaded)),
                    _ => {}
                }
                (
                    cost,
                    0x0e_d280,
                    if i == 13 { Tilemap(0) } else { Window(i + 1) },
                )
            }
            Tilemap(i) => {
                let cost = [22, 24, 24, 24, 40, 22][usize::from(i)];
                match i {
                    1 => self.loaded = 0x387f,
                    2 => self.loaded &= 0xff00,
                    3 => self.loaded |= 0x0180,
                    4 => publication = Some(PrefixPublication::Tilemap(self.loaded)),
                    _ => {}
                }
                (
                    cost,
                    0x0e_c448,
                    if i == 5 { Complete } else { Tilemap(i + 1) },
                )
            }
            Complete => return None,
        };
        self.phase = next;
        Some(PrefixInstruction {
            master_cycles,
            routine,
            publication,
        })
    }

    #[cfg(test)]
    pub(crate) fn next_pc(self) -> u32 {
        use PrefixPhase::*;
        match self.phase {
            Copy { instruction, .. } => {
                [0x0e_c493, 0x0e_c496, 0x0e_c499, 0x0e_c49a, 0x0e_c49c][usize::from(instruction)]
            }
            CallVwf => 0x0e_c49e,
            Vwf(i) => 0x0e_c4c9 + u32::from(i) * 3,
            CallWindow => 0x0e_c4a1,
            Window(i) => [
                0x0e_d280, 0x0e_d282, 0x0e_d284, 0x0e_d285, 0x0e_d287, 0x0e_d28a, 0x0e_d28b,
                0x0e_d28e, 0x0e_d291, 0x0e_d292, 0x0e_d293, 0x0e_d296, 0x0e_d299, 0x0e_d29b,
            ][usize::from(i)],
            Tilemap(i) => [
                0x0e_c4a4, 0x0e_c4a6, 0x0e_c4a9, 0x0e_c4ac, 0x0e_c4af, 0x0e_c4b2,
            ][usize::from(i)],
            Complete => 0x0e_c4b4,
        }
    }
}

/// RenderText's caller of Messaging_Text_Near resumes at $0E:C446 only
/// after Text_Initialize has returned: PLB (28), then RTL (44).
pub(crate) const RENDER_TEXT_RETURN_MASTER_CYCLES: u64 = 28 + 44;

/// The two instructions remain distinct if NMI interrupts after PLB.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RenderTextReturn {
    instruction: u8,
}

impl RenderTextReturn {
    pub(crate) fn step(&mut self) -> Option<u32> {
        let cost = *[28, 44].get(usize::from(self.instruction))?;
        self.instruction += 1;
        Some(cost)
    }

    #[cfg(test)]
    pub(crate) fn next_pc(self) -> u32 {
        [0x0e_c446, 0x0e_c447, 0x00_f84e][usize::from(self.instruction)]
    }
}

/// Module0E's four scroll loads/additions/stores and its RTL. The accumulated
/// word belongs to this continuation, not to the live scroll state after NMI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InterfaceReturn {
    instruction: u8,
    loaded: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InterfaceReturnInstruction {
    pub master_cycles: u32,
    /// Word order: BG2 horizontal/vertical, BG1 horizontal/vertical.
    pub scroll_store: Option<(usize, u16)>,
}

impl InterfaceReturn {
    pub(crate) fn step(
        &mut self,
        scroll: [u16; 4],
        offsets: [u16; 2],
    ) -> Option<InterfaceReturnInstruction> {
        let cost = *[
            22, 32, 40, 40, 32, 14, 40, 40, 32, 14, 40, 40, 32, 14, 40, 40, 22, 44,
        ]
        .get(usize::from(self.instruction))?;
        let mut scroll_store = None;
        match self.instruction {
            1 | 4 | 8 | 12 => {
                let axis = usize::from(self.instruction / 4);
                self.loaded = scroll[axis];
            }
            2 | 6 | 10 | 14 => {
                let axis = usize::from((self.instruction / 4) & 1);
                self.loaded = self.loaded.wrapping_add(offsets[axis]);
            }
            3 | 7 | 11 | 15 => {
                scroll_store = Some((usize::from(self.instruction / 4), self.loaded))
            }
            _ => {}
        }
        self.instruction += 1;
        Some(InterfaceReturnInstruction {
            master_cycles: cost,
            scroll_store,
        })
    }

    #[cfg(test)]
    pub(crate) fn next_pc(self) -> u32 {
        [
            0x00_f84e, 0x00_f850, 0x00_f852, 0x00_f855, 0x00_f858, 0x00_f85a, 0x00_f85b, 0x00_f85e,
            0x00_f861, 0x00_f863, 0x00_f864, 0x00_f867, 0x00_f86a, 0x00_f86c, 0x00_f86d, 0x00_f870,
            0x00_f873, 0x00_f875, 0x00_805a,
        ][usize::from(self.instruction)]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum InitializationTail {
    #[default]
    CallClear,
    ClearPrefix(u8),
    ClearRow {
        base: u16,
        instruction: u8,
    },
    ClearReturn(u8),
    CallerReturn(u8),
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Publication {
    ClearRenderWord(u16),
    ClearSourceOffset,
    IncrementReadPosition,
    ClearNextPosition,
    ClearReadPosition,
    RequestTextDma,
    DisableCoreUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub routine: u32,
    pub publication: Option<Publication>,
}

impl InitializationTail {
    /// Retire one whole source instruction. An NMI may suspend the returned
    /// state; neither a completed store nor a callee return ends the caller.
    pub(crate) fn step(&mut self) -> Option<Instruction> {
        use InitializationTail::*;
        use Publication::*;
        let clear = 0x0e_d1f9;
        let caller = 0x0e_c448;
        let (master_cycles, routine, publication, next) = match *self {
            CallClear => (46, caller, None, ClearPrefix(0)),
            ClearPrefix(i) => {
                let costs = [22, 16, 22, 28, 22, 24, 14];
                (
                    costs[usize::from(i)],
                    clear,
                    None,
                    if i == 6 {
                        ClearRow {
                            base: 0x7d0,
                            instruction: 0,
                        }
                    } else {
                        ClearPrefix(i + 1)
                    },
                )
            }
            ClearRow { base, instruction } => {
                let next = if instruction != 11 {
                    ClearRow {
                        base,
                        instruction: instruction + 1,
                    }
                } else if base != 0 {
                    ClearRow {
                        base: base - 0x10,
                        instruction: 0,
                    }
                } else {
                    ClearReturn(0)
                };
                let (cost, publication) = match instruction {
                    0..=7 => (46, Some(ClearRenderWord(base + u16::from(instruction) * 2))),
                    8 | 10 => (14, None),
                    9 => (24, None),
                    11 => (if base == 0 { 16 } else { 22 }, None),
                    _ => unreachable!(),
                };
                (cost, clear, publication, next)
            }
            ClearReturn(i) => {
                let (cost, publication) = match i {
                    0 => (28, None), // PLB
                    1 => (40, Some(ClearSourceOffset)),
                    2 => (62, Some(IncrementReadPosition)),
                    3 => (22, None), // SEP #$30
                    4 => (32, Some(ClearNextPosition)),
                    5 => (42, None), // RTS to Text_Initialize
                    _ => unreachable!(),
                };
                (
                    cost,
                    clear,
                    publication,
                    if i == 5 {
                        CallerReturn(0)
                    } else {
                        ClearReturn(i + 1)
                    },
                )
            }
            CallerReturn(i) => {
                let (cost, publication) = match i {
                    0 | 2 => (22, None), // REP/SEP #$30
                    1 => (40, Some(ClearReadPosition)),
                    3 => (16, None), // LDA #$02
                    4 => (24, Some(RequestTextDma)),
                    5 => (32, Some(DisableCoreUpdate)),
                    6 => (42, None), // RTS to Messaging_Text_Near
                    _ => unreachable!(),
                };
                (
                    cost,
                    caller,
                    publication,
                    if i == 6 {
                        Complete
                    } else {
                        CallerReturn(i + 1)
                    },
                )
            }
            Complete => return None,
        };
        *self = next;
        Some(Instruction {
            master_cycles,
            routine,
            publication,
        })
    }

    #[cfg(test)]
    pub(crate) fn next_pc(self) -> u32 {
        use InitializationTail::*;
        match self {
            CallClear => 0x0e_c4b7,
            ClearPrefix(i) => [
                0x0e_d1f9, 0x0e_d1fa, 0x0e_d1fc, 0x0e_d1fd, 0x0e_d1fe, 0x0e_d200, 0x0e_d203,
            ][usize::from(i)],
            ClearRow { instruction, .. } => [
                0x0e_d204, 0x0e_d207, 0x0e_d20a, 0x0e_d20d, 0x0e_d210, 0x0e_d213, 0x0e_d216,
                0x0e_d219, 0x0e_d21c, 0x0e_d21d, 0x0e_d220, 0x0e_d221,
            ][usize::from(instruction)],
            ClearReturn(i) => [
                0x0e_d223, 0x0e_d224, 0x0e_d227, 0x0e_d22a, 0x0e_d22c, 0x0e_d22f,
            ][usize::from(i)],
            CallerReturn(i) => [
                0x0e_c4ba, 0x0e_c4bc, 0x0e_c4bf, 0x0e_c4c1, 0x0e_c4c3, 0x0e_c4c5, 0x0e_c4c8,
            ][usize::from(i)],
            Complete => 0x0e_8036,
        }
    }
}
