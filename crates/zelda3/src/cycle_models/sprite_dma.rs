//! Sprite DMA source selection, animation and pointer suffix after OAM packing.
//! Indexes are byte offsets. Keep loaded operands, branch decisions and carry
//! until their consuming instruction, including across a suspended native call.

#[derive(Clone, Copy, Debug)]
pub(crate) enum Table {
    Body,
    Head,
    Hand,
    Sword,
    Shield,
    Auxiliary,
    AuxiliaryGroup,
    PushedBlock,
    AnimationCountdown,
    AnimationSourceOffset,
}

impl Table {
    pub(crate) fn base(self) -> u16 {
        match self {
            Self::Body => 0x9396,
            Self::Head => 0x95f4,
            Self::Hand => 0x9852,
            Self::Sword => 0x849c,
            Self::Shield => 0x84ac,
            Self::Auxiliary => 0x84b2,
            Self::AuxiliaryGroup => 0x85b2,
            Self::PushedBlock => 0x8494,
            Self::AnimationCountdown => 0x85d2,
            Self::AnimationSourceOffset => 0x85de,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Operand {
    Graphics,
    LeftHand,
    RightHand,
    Sword,
    Shield,
    Staging,
    PushedBlock,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SourceWords {
    instruction: u8,
    accumulator: u16,
    index: u16,
    group: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub store: Option<(usize, u16)>,
}

enum SourceRead {
    Operand(Operand),
    Table(Table, u16),
}

pub(crate) const SOURCE_WORD_ADDRESSES: [u16; 14] = [
    0xacc, 0xace, 0xad0, 0xad2, 0xad4, 0xad6, 0xac0, 0xac2, 0xac4, 0xac6, 0xac8, 0xaca, 0xad8,
    0xada,
];

impl SourceWords {
    pub(crate) fn return_indices(self) -> (u16, u16) {
        assert!(self.is_complete());
        (self.index, u16::from(self.group))
    }

    pub(crate) fn is_complete(self) -> bool {
        self.instruction == 53
    }

    pub(crate) fn step(
        &mut self,
        mut read: impl FnMut(Operand) -> u16,
        mut table: impl FnMut(Table, u16) -> u16,
    ) -> Option<Instruction> {
        self.step_read(|request| match request {
            SourceRead::Operand(operand) => read(operand),
            SourceRead::Table(selected, index) => table(selected, index),
        })
    }

    fn step_read(&mut self, mut read: impl FnMut(SourceRead) -> u16) -> Option<Instruction> {
        let mut store = None;
        let cost = match self.instruction {
            0 | 17 => 22, // REP #$31; later SEP #$10
            1 | 11 | 14 => {
                self.index = read(SourceRead::Operand(match self.instruction {
                    1 => Operand::Graphics,
                    11 => Operand::LeftHand,
                    _ => Operand::RightHand,
                }));
                40
            }
            2 | 6 | 12 | 15 => {
                self.accumulator = read(SourceRead::Table(
                    match self.instruction {
                        2 => Table::Body,
                        6 => Table::Head,
                        _ => Table::Hand,
                    },
                    self.index,
                ));
                46 // 16-bit X always pays the indexed-read cycle
            }
            18 | 24 => {
                self.index = read(SourceRead::Operand(if self.instruction == 18 {
                    Operand::Sword
                } else {
                    Operand::Shield
                })) & 0xff;
                32
            }
            19 | 25 | 38 | 48 => {
                let selected = match self.instruction {
                    19 => Table::Sword,
                    25 => Table::Shield,
                    38 => Table::Auxiliary,
                    _ => Table::PushedBlock,
                };
                self.accumulator = read(SourceRead::Table(selected, self.index));
                40 + if (selected.base() & 0xff) + self.index > 0xff {
                    6
                } else {
                    0
                }
            }
            3 | 5 | 7 | 10 | 13 | 16 | 20 | 23 | 26 | 29 | 39 | 43 | 49 | 52 => {
                let word = match self.instruction {
                    3 => 0,
                    5 => 1,
                    7 => 2,
                    10 => 3,
                    13 => 4,
                    16 => 5,
                    20 => 6,
                    23 => 7,
                    26 => 8,
                    29 => 9,
                    39 => 10,
                    43 => 11,
                    49 => 12,
                    _ => 13,
                };
                store = Some((word, self.accumulator));
                40
            }
            4 | 9 | 22 | 28 | 51 => {
                // Each addition follows CLC (or the initial REP #$31).
                let value = match self.instruction {
                    4 | 9 => 0x200,
                    22 => 0x180,
                    28 => 0xc0,
                    _ => 0x100,
                };
                self.accumulator = self.accumulator.wrapping_add(value);
                24
            }
            8 | 21 | 27 | 40 | 50 => 14, // CLC
            30 | 35 | 44 => {
                self.accumulator = read(SourceRead::Operand(if self.instruction == 44 {
                    Operand::PushedBlock
                } else {
                    Operand::Staging
                }));
                40
            }
            31 | 45 => {
                self.accumulator &= if self.instruction == 31 { 0xf8 } else { 3 };
                24
            }
            32 | 33 => {
                self.accumulator >>= 1;
                14
            }
            34 => {
                self.group = self.accumulator as u8;
                14
            } // TAY, x8
            36 | 46 => {
                self.accumulator <<= 1;
                14
            }
            41 => {
                self.index = u16::from(self.group);
                14
            } // TYX
            42 => {
                self.accumulator = self
                    .accumulator
                    .wrapping_add(read(SourceRead::Table(Table::AuxiliaryGroup, self.index)));
                40 // masked group offset is at most $3e
            }
            // Eight-bit TAX wraps the auxiliary index independently of Y.
            37 | 47 => {
                self.index = self.accumulator & 0xff;
                14
            }
            53 => return None,
            _ => unreachable!(),
        };
        self.instruction += 1;
        Some(Instruction {
            master_cycles: cost,
            store,
        })
    }

    pub(crate) fn next_pc(self) -> u32 {
        const PCS: [u16; 54] = [
            0x865c, 0x865e, 0x8661, 0x8664, 0x8667, 0x866a, 0x866d, 0x8670, 0x8673, 0x8674, 0x8677,
            0x867a, 0x867d, 0x8680, 0x8683, 0x8686, 0x8689, 0x868c, 0x868e, 0x8691, 0x8694, 0x8697,
            0x8698, 0x869b, 0x869e, 0x86a1, 0x86a4, 0x86a7, 0x86a8, 0x86ab, 0x86ae, 0x86b1, 0x86b4,
            0x86b5, 0x86b6, 0x86b7, 0x86ba, 0x86bb, 0x86bc, 0x86bf, 0x86c2, 0x86c3, 0x86c4, 0x86c7,
            0x86ca, 0x86cd, 0x86d0, 0x86d1, 0x86d2, 0x86d5, 0x86d8, 0x86d9, 0x86dc, 0x86df,
        ];
        u32::from(PCS[usize::from(self.instruction)])
    }
}

impl SourceWords {
    /// Retire the shared source-selection continuation on its caller's real
    /// registers and bus. Loaded high bytes and arithmetic flags survive NMI.
    pub(crate) fn step_clocked(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        bus: impl FnMut(u64, super::intro_clear::Access) -> u16,
    ) -> Option<u64> {
        use super::nmi_stack::CARRY;
        if self.is_complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.d, cpu.db, cpu.pc),
            (0, 0, 0, self.next_pc() as u16)
        );
        assert_eq!(cpu.p & 8, 0, "sprite DMA caller requires binary arithmetic");
        if self.instruction == 0 {
            self.accumulator = cpu.a;
            self.index = cpu.x;
            self.group = cpu.y as u8;
        } else {
            assert_eq!(self.accumulator, cpu.a);
            assert_eq!(self.index, cpu.x);
        }
        let op = self.instruction;
        let old_a = cpu.a;
        let mut w = SourceClock {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut operand_value = None;
        let instruction = self
            .step_read(|request| {
                let (address, width, indexed) = match request {
                    SourceRead::Operand(operand) => {
                        let (address, width) = match operand {
                            Operand::Graphics => (0x100, 2),
                            Operand::LeftHand => (0x102, 2),
                            Operand::RightHand => (0x104, 2),
                            Operand::Sword => (0x107, 1),
                            Operand::Shield => (0x108, 1),
                            Operand::Staging => (0x109, 2),
                            Operand::PushedBlock => (0x2c3, 2),
                        };
                        (address, width, false)
                    }
                    SourceRead::Table(table, index) => {
                        assert_eq!(index, w.cpu.x);
                        let extra = w.cpu.p & 0x10 == 0 || (table.base() & 255) + index > 255;
                        (u32::from(table.base()) + u32::from(index), 2, extra)
                    }
                };
                w.drain(16);
                if indexed {
                    w.drain(6);
                }
                let value = (w.bus)(
                    w.clock.clock_master_cycles(),
                    super::intro_clear::Access::Read(address, width),
                );
                w.drain(u32::from(width) * 8);
                operand_value = Some(value);
                value
            })
            .unwrap();
        if let Some((index, value)) = instruction.store {
            w.drain(16);
            (w.bus)(
                w.clock.clock_master_cycles(),
                super::intro_clear::Access::Write(
                    u32::from(SOURCE_WORD_ADDRESSES[index]),
                    value,
                    2,
                ),
            );
            w.drain(16);
        } else {
            if w.cycles == 8 {
                w.drain(instruction.master_cycles - 8);
            }
            match op {
                0 => {
                    w.cpu.p &= !0x31;
                }
                17 => {
                    w.cpu.p |= 0x10;
                    w.cpu.x &= 255;
                    w.cpu.y &= 255;
                    self.index = w.cpu.x;
                }
                1 | 11 | 14 | 18 | 24 | 37 | 41 | 47 => {
                    w.cpu.x = self.index;
                    w.flags(w.cpu.x, if w.cpu.p & 0x10 == 0 { 2 } else { 1 });
                }
                34 => {
                    w.cpu.y = u16::from(self.group);
                    w.flags(w.cpu.y, 1);
                }
                8 | 21 | 27 | 40 | 50 => w.cpu.p &= !CARRY,
                4 | 9 | 22 | 28 | 42 | 51 => {
                    assert_eq!(
                        w.cpu.p & CARRY,
                        0,
                        "source addition must retain preceding CLC/REP"
                    );
                    let rhs = match op {
                        4 | 9 => 0x200,
                        22 => 0x180,
                        28 => 0xc0,
                        42 => operand_value.unwrap(),
                        _ => 0x100,
                    };
                    let value = self.accumulator;
                    w.cpu.p = (w.cpu.p & !(CARRY | 0x40))
                        | u8::from(u32::from(old_a) + u32::from(rhs) > 0xffff)
                        | if (!(old_a ^ rhs) & (old_a ^ value) & 0x8000) != 0 {
                            0x40
                        } else {
                            0
                        };
                    w.cpu.a = value;
                    w.flags(value, 2);
                }
                _ => {
                    if matches!(op, 32 | 33 | 36 | 46) {
                        let carry = if op == 32 || op == 33 {
                            old_a & 1
                        } else {
                            old_a >> 15
                        };
                        w.cpu.p = (w.cpu.p & !CARRY) | carry as u8;
                    }
                    w.cpu.a = self.accumulator;
                    w.flags(w.cpu.a, 2);
                }
            }
        }
        assert_eq!(w.cycles, u64::from(instruction.master_cycles));
        w.cpu.pc = self.next_pc() as u16;
        Some(w.cycles)
    }
}

struct SourceClock<'a, B> {
    cpu: &'a mut super::nmi_stack::NativeNmiContext,
    clock: &'a mut snes::CpuMasterTimeline,
    bus: B,
    cycles: u64,
}
impl<B: FnMut(u64, super::intro_clear::Access) -> u16> SourceClock<'_, B> {
    fn fetch(&mut self) {
        self.cycles += 8;
        self.clock.advance_synchronous_pcbase_opcode_fetch(8);
    }
    fn drain(&mut self, n: u32) {
        self.cycles += u64::from(n);
        let bus = &mut self.bus;
        self.clock
            .advance_synchronous_after_semantics_with(n, |event, _| {
                if let snes::cpu_timeline::CpuSynchronousTimelineEvent::HMax {
                    completed_field_index,
                    completed_scanline: 224,
                    event_timestamp,
                    ..
                } = event
                {
                    bus(
                        event_timestamp.master_cycles(),
                        super::intro_clear::Access::Vblank {
                            field: completed_field_index,
                            at: event_timestamp.master_cycles(),
                        },
                    );
                }
                Ok::<u32, String>(0)
            })
            .unwrap();
    }
    fn flags(&mut self, value: u16, width: u8) {
        use super::nmi_stack::{NEGATIVE, ZERO};
        let value = if width == 1 { value & 255 } else { value };
        self.cpu.p = (self.cpu.p & !(NEGATIVE | ZERO))
            | if value == 0 { ZERO } else { 0 }
            | if width == 1 {
                value as u8 & NEGATIVE
            } else {
                (value >> 8) as u8 & NEGATIVE
            };
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AnimationOperand {
    BackgroundCountdown,
    Overlay,
    BackgroundOffset,
    LinkCountdown,
    LinkOffset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnimationStore {
    BackgroundCountdown,
    BackgroundOffset,
    BackgroundSource,
    LinkCountdown,
    LinkOffset,
    LinkSourceUpper,
    LinkSourceLower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AnimationInstruction {
    pub master_cycles: u32,
    pub store: Option<(AnimationStore, u16)>,
}

/// Final three pointer pairs, SEP and RTS. These are word loads even though
/// their producers usually update only the low byte. ASL supplies carry to the
/// first ADC; that ADC in turn supplies carry to the second one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PointerWords {
    instruction: u8,
    accumulator: u16,
    carry: bool,
    #[serde(default)]
    overflow: bool,
}

impl PointerWords {
    pub(crate) fn return_registers(self) -> (u16, bool, bool) {
        assert!(self.is_complete());
        (self.accumulator, self.carry, self.overflow)
    }

    pub(crate) fn is_complete(self) -> bool {
        self.instruction == 20
    }

    pub(crate) fn step(&mut self, mut read: impl FnMut(usize) -> u16) -> Option<Instruction> {
        let mut store = None;
        let cost = match self.instruction {
            0..=17 => {
                let pair = usize::from(self.instruction / 6);
                match self.instruction % 6 {
                    0 => {
                        self.accumulator = read(pair);
                        40
                    }
                    1 => {
                        self.carry = self.accumulator & 0x8000 != 0;
                        self.accumulator <<= 1;
                        14
                    }
                    2 | 4 => {
                        let value = if self.instruction % 6 == 4 {
                            0x200
                        } else if pair == 2 {
                            0xb540
                        } else {
                            0xb940
                        };
                        let sum = u32::from(self.accumulator) + value + u32::from(self.carry);
                        self.overflow = (!(self.accumulator ^ value as u16)
                            & (self.accumulator ^ sum as u16) & 0x8000) != 0;
                        self.accumulator = sum as u16;
                        self.carry = sum > 0xffff;
                        24
                    }
                    3 | 5 => {
                        store = Some((
                            pair * 2 + usize::from(self.instruction % 6 == 5),
                            self.accumulator,
                        ));
                        40
                    }
                    _ => unreachable!(),
                }
            }
            18 => 22,
            19 => 42,
            20 => return None,
            _ => unreachable!(),
        };
        self.instruction += 1;
        Some(Instruction {
            master_cycles: cost,
            store,
        })
    }

    pub(crate) fn next_pc(self) -> u32 {
        match self.instruction {
            0..=17 => {
                0x874e
                    + u32::from(self.instruction / 6) * 0x10
                    + [0, 3, 4, 7, 10, 13][usize::from(self.instruction % 6)]
            }
            18 => 0x877e,
            19 => 0x8780,
            20 => 0x8036, // test caller's return address
            _ => unreachable!(),
        }
    }
}

/// Shared animation work after the fourteen sprite-source stores. Each load,
/// decrement, comparison and publication retires separately. In particular,
/// the branch decision survives changes to shared RAM after its comparison.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Animations {
    instruction: u8,
    accumulator: u16,
    index: u8,
    zero: bool,
}

impl Animations {
    pub(crate) fn with_entry_index(index: u16) -> Self {
        Self { index: index as u8, ..Self::default() }
    }

    pub(crate) fn return_index(self) -> u16 {
        assert!(self.is_complete());
        u16::from(self.index)
    }

    pub(crate) fn is_complete(self) -> bool {
        self.instruction == 44
    }

    pub(crate) fn step(
        &mut self,
        mut read: impl FnMut(AnimationOperand) -> u16,
        mut table: impl FnMut(Table, u16) -> u16,
    ) -> Option<AnimationInstruction> {
        self.step_read(|request| match request {
            AnimationRead::Operand(operand) => read(operand),
            AnimationRead::Table(selected, index) => table(selected, index),
        })
    }

    fn step_read(
        &mut self,
        mut read: impl FnMut(AnimationRead) -> u16,
    ) -> Option<AnimationInstruction> {
        let mut next = self.instruction + 1;
        let mut store = None;
        let cost = match self.instruction {
            0 | 12 | 22 | 26 => {
                self.accumulator = read(AnimationRead::Operand(match self.instruction {
                    0 => AnimationOperand::BackgroundCountdown,
                    12 => AnimationOperand::BackgroundOffset,
                    22 => AnimationOperand::LinkCountdown,
                    _ => AnimationOperand::LinkOffset,
                }));
                48
            }
            1 | 23 => {
                self.accumulator = self.accumulator.wrapping_sub(1);
                self.zero = self.accumulator == 0;
                14
            }
            2 | 11 | 18 | 24 | 34 | 36 => {
                store = Some((
                    match self.instruction {
                        2 | 11 => AnimationStore::BackgroundCountdown,
                        18 => AnimationStore::BackgroundOffset,
                        24 | 36 => AnimationStore::LinkCountdown,
                        _ => AnimationStore::LinkOffset,
                    },
                    self.accumulator,
                ));
                48
            }
            3 | 7 | 9 | 16 | 25 | 31 => {
                let taken = if self.instruction == 7 {
                    self.zero
                } else {
                    !self.zero
                };
                if taken {
                    next = match self.instruction {
                        3 => 22,
                        7 => 10,
                        9 => 11,
                        16 => 18,
                        25 => 44,
                        _ => 33,
                    };
                }
                if taken {
                    22
                } else {
                    16
                }
            }
            4 | 10 | 17 | 37 => {
                self.accumulator = match self.instruction {
                    4 => 9,
                    10 => 0x17,
                    17 => 0,
                    _ => 0xb280,
                };
                24
            }
            5 => {
                self.index = read(AnimationRead::Operand(AnimationOperand::Overlay)) as u8;
                24
            }
            6 | 8 | 30 => {
                self.zero = self.index
                    == match self.instruction {
                        6 => 0xb5,
                        8 => 0xbc,
                        _ => 12,
                    };
                16
            }
            13 | 19 | 38 | 41 => 14, // CLC
            14 | 20 | 42 => {
                self.accumulator = self.accumulator.wrapping_add(match self.instruction {
                    14 => 0x400,
                    20 => 0xa680,
                    _ => 0x60,
                });
                24
            }
            15 => {
                self.zero = self.accumulator == 0xc00;
                24
            }
            21 | 40 | 43 => {
                store = Some((
                    match self.instruction {
                        21 => AnimationStore::BackgroundSource,
                        40 => AnimationStore::LinkSourceUpper,
                        _ => AnimationStore::LinkSourceLower,
                    },
                    self.accumulator,
                ));
                40
            }
            27 => {
                self.index = self.accumulator as u8;
                14
            }
            28 | 29 => {
                self.index = self.index.wrapping_add(1);
                14
            }
            32 => {
                self.index = 0;
                16
            }
            33 => {
                self.accumulator = u16::from(self.index);
                14
            }
            35 | 39 => {
                let selected = if self.instruction == 35 {
                    Table::AnimationCountdown
                } else {
                    Table::AnimationSourceOffset
                };
                let value = read(AnimationRead::Table(selected, u16::from(self.index)));
                self.accumulator = if self.instruction == 35 {
                    value
                } else {
                    self.accumulator.wrapping_add(value)
                };
                40 + if (selected.base() & 0xff) + u16::from(self.index) > 0xff {
                    6
                } else {
                    0
                }
            }
            44 => return None,
            _ => unreachable!(),
        };
        self.instruction = next;
        Some(AnimationInstruction {
            master_cycles: cost,
            store,
        })
    }

    pub(crate) fn next_pc(self) -> u32 {
        const PCS: [u16; 45] = [
            0x86df, 0x86e3, 0x86e4, 0x86e8, 0x86ea, 0x86ed, 0x86ef, 0x86f1, 0x86f3, 0x86f5, 0x86f7,
            0x86fa, 0x86fe, 0x8702, 0x8703, 0x8706, 0x8709, 0x870b, 0x870e, 0x8712, 0x8713, 0x8716,
            0x8719, 0x871d, 0x871e, 0x8722, 0x8724, 0x8728, 0x8729, 0x872a, 0x872b, 0x872d, 0x872f,
            0x8731, 0x8732, 0x8736, 0x8739, 0x873d, 0x8740, 0x8741, 0x8744, 0x8747, 0x8748, 0x874b,
            0x874e,
        ];
        u32::from(PCS[usize::from(self.instruction)])
    }
}

enum AnimationRead {
    Operand(AnimationOperand),
    Table(Table, u16),
}

impl AnimationStore {
    pub(crate) fn address(self) -> u32 {
        match self {
            Self::BackgroundCountdown => 0x7ec00d,
            Self::BackgroundOffset => 0x7ec00f,
            Self::BackgroundSource => 0xadc,
            Self::LinkCountdown => 0x7ec013,
            Self::LinkOffset => 0x7ec015,
            Self::LinkSourceUpper => 0xae0,
            Self::LinkSourceLower => 0xae2,
        }
    }
}

impl Animations {
    pub(crate) fn step_clocked(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        bus: impl FnMut(u64, super::intro_clear::Access) -> u16,
    ) -> Option<u64> {
        use super::intro_clear::Access;
        use super::nmi_stack::{CARRY, ZERO};
        if self.is_complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.db, cpu.d, cpu.pc, cpu.p & 0x38),
            (0, 0, 0, self.next_pc() as u16, 0x10)
        );
        if self.instruction == 0 {
            self.accumulator = cpu.a;
            self.index = cpu.x as u8;
            self.zero = cpu.p & ZERO != 0;
        }
        assert_eq!((self.accumulator, u16::from(self.index)), (cpu.a, cpu.x));
        let op = self.instruction;
        if matches!(op, 3 | 7 | 9 | 16 | 25 | 31) {
            assert_eq!(self.zero, cpu.p & ZERO != 0);
        }
        let old_a = cpu.a;
        let mut w = SourceClock {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut loaded = None;
        let instruction = self
            .step_read(|request| {
                let (address, width, operands, extra) = match request {
                    AnimationRead::Operand(operand) => match operand {
                        AnimationOperand::BackgroundCountdown => (0x7ec00d, 2, 24, 0),
                        AnimationOperand::BackgroundOffset => (0x7ec00f, 2, 24, 0),
                        AnimationOperand::LinkCountdown => (0x7ec013, 2, 24, 0),
                        AnimationOperand::LinkOffset => (0x7ec015, 2, 24, 0),
                        AnimationOperand::Overlay => (0x8c, 1, 8, 0),
                    },
                    AnimationRead::Table(table, index) => {
                        assert_eq!(index, w.cpu.x);
                        (
                            u32::from(table.base()) + u32::from(index),
                            2,
                            16,
                            if (table.base() & 255) + index > 255 {
                                6
                            } else {
                                0
                            },
                        )
                    }
                };
                w.drain(operands + extra);
                let value = (w.bus)(w.clock.clock_master_cycles(), Access::Read(address, width));
                w.drain(u32::from(width) * 8);
                loaded = Some(value);
                value
            })
            .unwrap();
        if let Some((destination, value)) = instruction.store {
            let address = destination.address();
            w.drain(if address > 0xffff { 24 } else { 16 });
            (w.bus)(
                w.clock.clock_master_cycles(),
                Access::Write(address, value, 2),
            );
            w.drain(16);
        } else {
            if w.cycles == 8 {
                w.drain(instruction.master_cycles - 8);
            }
            match op {
                3 | 7 | 9 | 16 | 25 | 31 => {}
                6 | 8 | 30 => {
                    let rhs = match op {
                        6 => 0xb5,
                        8 => 0xbc,
                        _ => 12,
                    };
                    let lhs = w.cpu.x as u8;
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(lhs >= rhs);
                    w.flags(u16::from(lhs.wrapping_sub(rhs)), 1);
                }
                15 => {
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= 0xc00);
                    w.flags(w.cpu.a.wrapping_sub(0xc00), 2);
                }
                5 | 27 | 28 | 29 | 32 => {
                    w.cpu.x = u16::from(self.index);
                    w.flags(w.cpu.x, 1);
                }
                13 | 19 | 38 | 41 => w.cpu.p &= !CARRY,
                14 | 20 | 39 | 42 => {
                    assert_eq!(w.cpu.p & CARRY, 0);
                    let rhs = match op {
                        14 => 0x400,
                        20 => 0xa680,
                        39 => loaded.unwrap(),
                        _ => 0x60,
                    };
                    w.adc_flags(old_a, rhs, false, self.accumulator);
                    w.cpu.a = self.accumulator;
                }
                _ => {
                    w.cpu.a = self.accumulator;
                    w.flags(w.cpu.a, 2);
                }
            }
        }
        assert_eq!(w.cycles, u64::from(instruction.master_cycles));
        w.cpu.pc = self.next_pc() as u16;
        Some(w.cycles)
    }
}

pub(crate) const POINTER_WORD_ADDRESSES: [u16; 6] = [0xaec, 0xaee, 0xaf0, 0xaf2, 0xaf6, 0xaf8];
impl PointerWords {
    pub(crate) fn step_clocked(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        bus: impl FnMut(u64, super::intro_clear::Access) -> u16,
    ) -> Option<u64> {
        use super::intro_clear::Access;
        use super::nmi_stack::CARRY;
        if self.is_complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.d, cpu.db, cpu.pc),
            (0, 0, 0, self.next_pc() as u16)
        );
        assert_eq!(
            cpu.p & 0x38,
            if self.instruction < 19 { 0x10 } else { 0x30 }
        );
        if self.instruction == 0 {
            self.accumulator = cpu.a;
            self.carry = cpu.p & CARRY != 0;
        }
        assert_eq!((self.accumulator, self.carry), (cpu.a, cpu.p & CARRY != 0));
        let op = self.instruction;
        let old_a = cpu.a;
        let old_c = self.carry;
        let mut w = SourceClock {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let instruction = self
            .step(|pair| {
                w.drain(16);
                let value = (w.bus)(
                    w.clock.clock_master_cycles(),
                    Access::Read([0xae8, 0xaea, 0xaf4][pair], 2),
                );
                w.drain(16);
                value
            })
            .unwrap();
        if let Some((index, value)) = instruction.store {
            w.drain(16);
            (w.bus)(
                w.clock.clock_master_cycles(),
                Access::Write(u32::from(POINTER_WORD_ADDRESSES[index]), value, 2),
            );
            w.drain(16);
        } else if op == 19 {
            // Actual RTS: pull the caller word, then increment its PC.
            w.drain(12);
            let returned = (w.bus)(
                w.clock.clock_master_cycles(),
                Access::Read(u32::from(w.cpu.sp.wrapping_add(1)), 2),
            );
            w.drain(16);
            w.cpu.sp = w.cpu.sp.wrapping_add(2);
            w.cpu.pc = returned.wrapping_add(1);
            w.drain(6);
        } else {
            if w.cycles == 8 {
                w.drain(instruction.master_cycles - 8);
            }
            if op == 18 {
                w.cpu.p |= 0x20;
            }
            // SEP #$20, X is already byte-wide
            else {
                w.cpu.a = self.accumulator;
                match op % 6 {
                    1 => {
                        w.cpu.p = (w.cpu.p & !CARRY) | u8::from(self.carry);
                        w.flags(w.cpu.a, 2);
                    }
                    2 | 4 => {
                        let rhs = if op % 6 == 4 {
                            0x200
                        } else if op / 6 == 2 {
                            0xb540
                        } else {
                            0xb940
                        };
                        w.adc_flags(old_a, rhs, old_c, self.accumulator);
                    }
                    0 => w.flags(w.cpu.a, 2),
                    _ => unreachable!(),
                }
            }
        }
        assert_eq!(w.cycles, u64::from(instruction.master_cycles));
        if !self.is_complete() {
            w.cpu.pc = self.next_pc() as u16;
        }
        Some(w.cycles)
    }
}

impl<B: FnMut(u64, super::intro_clear::Access) -> u16> SourceClock<'_, B> {
    fn adc_flags(&mut self, lhs: u16, rhs: u16, carry: bool, value: u16) {
        use super::nmi_stack::CARRY;
        self.cpu.p = (self.cpu.p & !(CARRY | 0x40))
            | u8::from(u32::from(lhs) + u32::from(rhs) + u32::from(carry) > 0xffff)
            | if (!(lhs ^ rhs) & (lhs ^ value) & 0x8000) != 0 {
                0x40
            } else {
                0
            };
        self.flags(value, 2);
    }
}

/// Remaining NMI_PrepareSprites body and its main caller's STZ/BRA.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MainSpriteTail {
    animation: Animations,
    pointers: PointerWords,
    caller_instruction: u8,
    body_master_cycles: u64,
}
impl MainSpriteTail {
    pub(crate) fn complete(&self) -> bool {
        self.caller_instruction == 2
    }
    pub(crate) fn body_master_cycles(&self) -> u64 {
        self.body_master_cycles
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut super::nmi_stack::NativeNmiContext,
        clock: &mut snes::CpuMasterTimeline,
        bus: impl FnMut(u64, super::intro_clear::Access) -> u16,
    ) -> Option<(u64, u32)> {
        if self.complete() {
            return None;
        }
        let body = if !self.animation.is_complete() {
            self.animation.step_clocked(cpu, clock, bus)
        } else if !self.pointers.is_complete() {
            self.pointers.step_clocked(cpu, clock, bus)
        } else {
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.p & 0x30),
                (
                    0,
                    if self.caller_instruction == 0 {
                        0x805d
                    } else {
                        0x805f
                    },
                    0x30
                )
            );
            let mut w = SourceClock {
                cpu,
                clock,
                bus,
                cycles: 0,
            };
            w.fetch();
            w.drain(8);
            if self.caller_instruction == 0 {
                (w.bus)(
                    w.clock.clock_master_cycles(),
                    super::intro_clear::Access::Write(0x12, 0, 1),
                );
                w.drain(8);
                w.cpu.pc = 0x805f;
            } else {
                w.drain(6);
                w.cpu.pc = 0x8034;
            }
            self.caller_instruction += 1;
            return Some((w.cycles, 0x00_8034));
        };
        let cycles = body.expect("active sprite tail");
        self.body_master_cycles += cycles;
        Some((cycles, 0x00_85fc))
    }
}
