//! Native 3bpp-to-4bpp conversion work. The caller owns the source bytes,
//! destination and hardware clock; this work retires complete instructions.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Publication {
    VramWord { offset: u16, value: u16 },
    LineWord { row: u8, value: u16 },
    ScratchWord(u16),
    SourceOffset(u16),
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub publication: Option<Publication>,
    source_read: Option<SourceRead>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ConversionWork {
    instructions: Vec<Instruction>,
    next: usize,
    live: LivePlanes,
    high: bool,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
enum SourceRead {
    Pair(u16),
    Mask { offset: u16, row: u8 },
    Third(u16),
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct LivePlanes {
    pair: u16,
    third: u16,
    lines: [u16; 8],
}

impl ConversionWork {
    /// LoadCommonSprites embeds the same high-plane loop at $E6CD. Its
    /// counter starts at 64 and uses BNE rather than starting at 63 with
    /// BPL; both retire 64 tiles. It falls through to its caller's module
    /// branch at $E724 and must not execute the standalone SEP/RTS.
    pub(crate) fn common_high(data: &[u8]) -> Self {
        let mut work = Self::new(true, data, 64);
        work.instructions.truncate(work.instructions.len() - 2);
        #[cfg(test)]
        for instruction in &mut work.instructions {
            instruction.pc += 0x11e; // $E5AF loop duplicated at $E6CD
        }
        work
    }

    pub(crate) fn step(&mut self) -> Option<Instruction> {
        let instruction = *self.instructions.get(self.next)?;
        self.next += 1;
        Some(instruction)
    }

    /// Read at the owning native instruction, retaining the loaded planes
    /// across suspension. Raw art can wrap into mirrored WRAM, including the
    /// source-pointer and line scratch this very routine is publishing.
    pub(crate) fn step_with_reader(
        &mut self,
        mut read_word: impl FnMut(u16) -> u16,
    ) -> Option<Instruction> {
        let mut instruction = self.step()?;
        match instruction.source_read {
            Some(SourceRead::Pair(offset)) => self.live.pair = read_word(offset),
            Some(SourceRead::Mask { offset, row }) => {
                self.live.lines[usize::from(row)] =
                    (self.live.pair.swap_bytes() | read_word(offset)) & 0xff;
            }
            Some(SourceRead::Third(offset)) => self.live.third = read_word(offset) & 0xff,
            None => {}
        }
        instruction.publication = instruction.publication.map(|p| match p {
            Publication::VramWord { offset, .. } => {
                let row = offset % 16;
                let value = if row < 8 {
                    self.live.pair
                } else if self.high {
                    self.live.third
                        | ((self.live.third | self.live.lines[usize::from(15 - row)]) << 8)
                } else {
                    self.live.third
                };
                Publication::VramWord { offset, value }
            }
            Publication::LineWord { row, .. } => Publication::LineWord {
                row,
                value: self.live.lines[usize::from(row)],
            },
            Publication::ScratchWord(_) => Publication::ScratchWord(self.live.third),
            p => p,
        });
        Some(instruction)
    }

    #[cfg(test)]
    pub(crate) fn roundtrip_progress(&mut self) {
        self.next = bincode::deserialize(&bincode::serialize(&self.next).unwrap()).unwrap();
        self.live = bincode::deserialize(&bincode::serialize(&self.live).unwrap()).unwrap();
    }

    pub(crate) fn new(high: bool, data: &[u8], tiles: u8) -> Self {
        assert!((1..=128).contains(&tiles));
        let mut b = Builder {
            instructions: Vec::new(),
            source: 0,
        };
        let byte = |offset: usize| u16::from(data.get(offset).copied().unwrap_or(0));
        for tile in 0..tiles {
            let source = usize::from(tile) * 24;
            let destination = u16::from(tile) * 16;
            let mut combined = [0u16; 8];
            if high {
                b.emit(0xe5af, 16);
            } // LDX #$0E (X remains eight-bit)
            for row in 0..8u16 {
                let offset = source + usize::from(row) * 2;
                let value = byte(offset) | (byte(offset + 1) << 8);
                combined[usize::from(row)] = (value | (value >> 8)) & 0xff;
                if high {
                    let pc = if row & 1 == 0 { 0xe5b1 } else { 0xe5c4 };
                    b.read(pc, SourceRead::Pair(offset as u16));
                    b.write(
                        pc + 2,
                        36,
                        Publication::VramWord {
                            offset: destination + row,
                            value,
                        },
                    );
                    b.emit(pc + 5, 20);
                    b.read(
                        pc + 6,
                        SourceRead::Mask {
                            offset: offset as u16,
                            row: 7 - row as u8,
                        },
                    );
                    b.emit(pc + 8, 24);
                    b.write(
                        pc + 11,
                        38,
                        Publication::LineWord {
                            row: 7 - row as u8,
                            value: combined[usize::from(row)],
                        },
                    );
                    b.increment(pc + 13);
                    b.increment(pc + 15);
                    b.block(&[(pc + 17, 14), (pc + 18, 14)]);
                    if row & 1 != 0 {
                        b.emit(0xe5d7, if row == 7 { 16 } else { 22 });
                    }
                } else {
                    let pc = 0xe63c + row * 9;
                    b.read(pc, SourceRead::Pair(offset as u16));
                    b.write(
                        pc + 2,
                        36,
                        Publication::VramWord {
                            offset: destination + row,
                            value,
                        },
                    );
                    b.increment(pc + 5);
                    b.increment(pc + 7);
                }
            }
            b.emit(if high { 0xe5d9 } else { 0xe684 }, 16);
            for row in 0..8u16 {
                let value = byte(source + 16 + usize::from(row));
                if high {
                    let pc = if row & 1 == 0 { 0xe5db } else { 0xe5ee };
                    b.read(
                        pc,
                        SourceRead::Third((source + 16 + usize::from(row)) as u16),
                    );
                    b.emit(pc + 2, 24);
                    // $BD belongs to the third-plane byte, not to the OR of
                    // the first two planes. It is written before the fourth
                    // plane is synthesized, and survives after the return.
                    b.write(pc + 5, 32, Publication::ScratchWord(value));
                    b.block(&[(pc + 7, 38), (pc + 9, 20), (pc + 10, 32)]);
                    b.write(
                        pc + 12,
                        36,
                        Publication::VramWord {
                            offset: destination + 8 + row,
                            value: value | ((value | combined[usize::from(row)]) << 8),
                        },
                    );
                    b.increment(pc + 15);
                    b.block(&[(pc + 17, 14), (pc + 18, 14)]);
                    if row & 1 != 0 {
                        b.emit(0xe601, if row == 7 { 16 } else { 22 });
                    }
                } else {
                    let pc = 0xe686 + (row % 4) * 10;
                    b.read(
                        pc,
                        SourceRead::Third((source + 16 + usize::from(row)) as u16),
                    );
                    b.emit(pc + 2, 24);
                    b.write(
                        pc + 5,
                        36,
                        Publication::VramWord {
                            offset: destination + 8 + row,
                            value,
                        },
                    );
                    b.increment(pc + 8);
                    if row % 4 == 3 {
                        b.emit(0xe6ae, 14);
                        b.emit(0xe6af, if row == 7 { 16 } else { 22 });
                    }
                }
            }
            b.emit(if high { 0xe603 } else { 0xe6b1 }, 14);
            b.emit(
                if high { 0xe604 } else { 0xe6b2 },
                if tile + 1 == tiles { 16 } else { 22 },
            );
        }
        b.emit(if high { 0xe606 } else { 0xe6b4 }, 22);
        b.emit(if high { 0xe608 } else { 0xe6b6 }, 42);
        Self {
            instructions: b.instructions,
            next: 0,
            live: LivePlanes::default(),
            high,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_conversion_retains_loaded_pair_but_reads_the_later_mask_at_its_own_instruction() {
        let mut work = ConversionWork::new(true, &[], 1);
        assert!(work
            .step_with_reader(|_| panic!("LDX does not read source pixels"))
            .unwrap()
            .publication
            .is_none());
        let load = work
            .step_with_reader(|offset| {
                assert_eq!(offset, 0);
                0x1234
            })
            .unwrap();
        assert!(load.publication.is_none());
        let mut resumed: ConversionWork =
            bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        let store = resumed
            .step_with_reader(|_| panic!("STA must retain the loaded word"))
            .unwrap();
        assert_eq!(
            store.publication,
            Some(Publication::VramWord {
                offset: 0,
                value: 0x1234
            })
        );
        resumed
            .step_with_reader(|_| panic!("XBA does not reread the source"))
            .unwrap();
        resumed
            .step_with_reader(|offset| {
                assert_eq!(offset, 0);
                0xabcd
            })
            .unwrap();
        resumed
            .step_with_reader(|_| panic!("AND does not reread the source"))
            .unwrap();
        let line = resumed
            .step_with_reader(|_| panic!("line STA uses its retained result"))
            .unwrap();
        assert_eq!(
            line.publication,
            Some(Publication::LineWord {
                row: 7,
                value: (0x3412 | 0xabcd) & 0xff
            })
        );
    }
}

struct Builder {
    instructions: Vec<Instruction>,
    source: u16,
}

impl Builder {
    fn emit(&mut self, _pc: u16, master_cycles: u32) {
        self.instructions.push(Instruction {
            master_cycles,
            publication: None,
            source_read: None,
            #[cfg(test)]
            pc: u32::from(_pc),
        });
    }

    fn read(&mut self, pc: u16, source_read: SourceRead) {
        self.emit(pc, 56);
        self.instructions.last_mut().unwrap().source_read = Some(source_read);
    }

    fn block(&mut self, instructions: &[(u16, u32)]) {
        for &(pc, cycles) in instructions {
            self.emit(pc, cycles);
        }
    }

    fn write(&mut self, pc: u16, cycles: u32, publication: Publication) {
        self.emit(pc, cycles);
        self.instructions.last_mut().unwrap().publication = Some(publication);
    }

    fn increment(&mut self, pc: u16) {
        self.source += 1;
        self.write(pc, 54, Publication::SourceOffset(self.source));
    }
}
