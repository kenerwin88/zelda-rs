//! EraseTileMaps' native instruction and fixed-source DMA continuation.
//! CPU instructions, DMA selection and individual bytes remain separate:
//! the enclosing hardware clock owns DMA synchronization and channel clocks.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Pattern {
    Normal,
    Triforce,
    DungeonMap,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    Cpu {
        master_cycles: u32,
        write: Option<(u16, u16, bool)>,
        pc: u16,
    },
    DmaByte {
        source: u16,
        register: u8,
        last: bool,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TilemapEraseWork {
    pattern: Pattern,
    instruction: u8,
    accumulator: u16,
    in_dma: bool,
}

impl TilemapEraseWork {
    pub(crate) fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            instruction: 0,
            accumulator: 0,
            in_dma: false,
        }
    }

    pub(crate) fn step(&mut self, ram: &[u8], dma: &snes::DmaState) -> Option<Step> {
        if self.in_dma {
            let channel = &dma.channel[1];
            assert!(channel.dma_active && channel.size != 0);
            assert!(channel.fixed && !channel.from_b && channel.mode == 0);
            assert!(channel.a_bank == 0 && channel.a_adr <= 1);
            assert!(matches!(channel.b_adr, 0x18 | 0x19));
            let last = channel.size == 1;
            self.in_dma = !last;
            return Some(Step::DmaByte {
                source: channel.a_adr,
                register: channel.b_adr,
                last,
            });
        }
        let mut write = None;
        let (pc, master_cycles) = match self.instruction {
            0 => (self.entry_pc(), 22), // REP #$20, index width stays byte.
            1 => {
                self.accumulator = match self.pattern {
                    Pattern::Triforce => 0xa9,
                    _ => 0x7f,
                };
                (self.entry_pc() + 2, 24)
            }
            2 => {
                write = Some((2, self.accumulator, true));
                (self.entry_pc() + 5, 32)
            }
            3 => {
                self.accumulator = match self.pattern {
                    Pattern::Normal => 0x1ec,
                    Pattern::Triforce => 0x7f,
                    Pattern::DungeonMap => 0x300,
                };
                if matches!(self.pattern, Pattern::Normal) {
                    self.instruction += 1;
                }
                (self.entry_pc() + 7, 24)
            }
            4 => (self.entry_pc() + 10, 22), // BRA to shared store.
            5 => {
                write = Some((0, self.accumulator, true));
                (0x8355, 32)
            }
            6 => {
                write = Some((0x2115, 0, true));
                (0x8357, 36)
            }
            7 => {
                write = Some((0x2116, 0, true));
                (0x835a, 36)
            }
            8 => {
                self.accumulator = 0x1808;
                (0x835d, 24)
            }
            9 => {
                write = Some((0x4310, self.accumulator, true));
                (0x8360, 36)
            }
            10 => {
                write = Some((0x4314, 0, true));
                (0x8363, 36)
            }
            11 => {
                self.accumulator = 0;
                (0x8366, 24)
            }
            12 => {
                write = Some((0x4312, self.accumulator, true));
                (0x8369, 36)
            }
            13 => {
                self.accumulator = 0x2000;
                (0x836c, 24)
            }
            14 => {
                write = Some((0x4315, self.accumulator, true));
                (0x836f, 36)
            }
            15 => (0x8372, 16), // LDY #2.
            16 | 25 | 37 | 46 => {
                write = Some((0x420b, 2, false));
                self.in_dma = true;
                (
                    match self.instruction {
                        16 => 0x8374,
                        25 => 0x838e,
                        37 => 0x83b0,
                        _ => 0x83cb,
                    },
                    30,
                )
            }
            17 => (0x8377, 16), // LDX #$80.
            18 => {
                write = Some((0x2115, 0x80, false));
                (0x8379, 30)
            }
            19 => {
                write = Some((0x2116, 0, true));
                (0x837c, 36)
            }
            20 => {
                write = Some((0x4315, self.accumulator, true));
                (0x837f, 36)
            }
            21 => {
                self.accumulator = 0x1908;
                (0x8382, 24)
            }
            22 => {
                write = Some((0x4310, self.accumulator, true));
                (0x8385, 36)
            }
            23 => {
                self.accumulator = 1;
                (0x8388, 24)
            }
            24 => {
                write = Some((0x4312, self.accumulator, true));
                (0x838b, 36)
            }
            26 => {
                self.accumulator = u16::from_le_bytes([ram[2], ram[3]]);
                (0x8391, 32)
            }
            27 => {
                write = Some((0, self.accumulator, true));
                (0x8393, 32)
            }
            28 => {
                write = Some((0x2115, 0, true));
                (0x8395, 36)
            }
            29 => {
                self.accumulator = 0x6000;
                (0x8398, 24)
            }
            30 => {
                write = Some((0x2116, self.accumulator, true));
                (0x839b, 36)
            }
            31 => {
                self.accumulator = 0x1808;
                (0x839e, 24)
            }
            32 => {
                write = Some((0x4310, self.accumulator, true));
                (0x83a1, 36)
            }
            33 => {
                self.accumulator = 0;
                (0x83a4, 24)
            }
            34 => {
                write = Some((0x4312, self.accumulator, true));
                (0x83a7, 36)
            }
            35 => {
                self.accumulator = 0x800;
                (0x83aa, 24)
            }
            36 => {
                write = Some((0x4315, self.accumulator, true));
                (0x83ad, 36)
            }
            38 => {
                write = Some((0x2115, 0x80, false));
                (0x83b3, 30)
            }
            39 => {
                write = Some((0x4315, self.accumulator, true));
                (0x83b6, 36)
            }
            40 => {
                self.accumulator = 0x6000;
                (0x83b9, 24)
            }
            41 => {
                write = Some((0x2116, self.accumulator, true));
                (0x83bc, 36)
            }
            42 => {
                self.accumulator = 0x1908;
                (0x83bf, 24)
            }
            43 => {
                write = Some((0x4310, self.accumulator, true));
                (0x83c2, 36)
            }
            44 => {
                self.accumulator = 1;
                (0x83c5, 24)
            }
            45 => {
                write = Some((0x4312, self.accumulator, true));
                (0x83c8, 36)
            }
            47 => (0x83ce, 22),
            48 => (0x83d0, 44),
            _ => return None,
        };
        self.instruction += 1;
        Some(Step::Cpu {
            master_cycles,
            write,
            pc,
        })
    }

    pub(crate) fn entry_pc(&self) -> u16 {
        match self.pattern {
            Pattern::Normal => 0x834b,
            Pattern::Triforce => 0x8333,
            Pattern::DungeonMap => 0x833f,
        }
    }
}
