//! Native work and publications for the original US character-buffer decoder.
//! Inputs are immutable native dialogue assets and the caller's substitutions.
//! No source CPU state or execution is used to build or advance this work.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum Publication {
    Buffer { offset: u16, value: u16, width: u8 },
    SourcePosition(u16),
    OutputPosition(u16),
    Window(u8),
    Position(u16),
    Color(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub routine: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct CharacterBufferWork {
    instructions: Vec<Instruction>,
    progress: DecodeProgress,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct DecodeProgress {
    next: usize,
    pub source_position: u16,
    pub output_position: u16,
}

impl CharacterBufferWork {
    pub(crate) fn is_complete(&self) -> bool {
        self.progress.next == self.instructions.len()
    }

    pub(crate) fn step(&mut self) -> Option<Instruction> {
        let instruction = *self.instructions.get(self.progress.next)?;
        self.progress.next += 1;
        match instruction.publication {
            Some(Publication::SourcePosition(value)) => self.progress.source_position = value,
            Some(Publication::OutputPosition(value)) => self.progress.output_position = value,
            _ => {}
        }
        Some(instruction)
    }

    pub(crate) fn output_position(&self) -> u16 {
        self.progress.output_position
    }

    #[cfg(test)]
    pub(crate) fn roundtrip_progress(&mut self) {
        self.progress = bincode::deserialize(&bincode::serialize(&self.progress).unwrap()).unwrap();
    }

    /// Build the native command/dictionary expansion. Unsupported asset
    /// layouts are rejected before any state is published. A caller can
    /// retain `next` at any instruction boundary, including after the final
    /// buffer store but before the decoder's SEP/RTS.
    pub(crate) fn new(
        message: &[u8],
        dictionary: &[Vec<u8>],
        name: [u16; 6],
        number_pairs: &[u8],
    ) -> Option<Self> {
        use Publication::*;
        let mut b = Builder(Vec::new());
        b.block(&[
            (0xc4e2, 22),
            (0xc4e4, 40),
            (0xc4e7, 14),
            (0xc4e8, 40),
            (0xc4eb, 14),
            (0xc4ec, 48),
            (0xc4f0, 32),
            (0xc4f2, 48),
            (0xc4f6, 32),
            (0xc4f8, 24),
        ]);
        b.write(
            0xc4fb,
            48,
            Buffer {
                offset: 0,
                value: 0x7f7f,
                width: 2,
            },
        );
        b.block(&[(0xc4ff, 24), (0xc502, 14)]);
        b.write(0xc503, 40, OutputPosition(0));
        b.write(0xc506, 40, SourcePosition(0));
        b.emit(0xc509, 22);
        let mut src = 0usize;
        let mut dst = 0u16;
        loop {
            let byte = message.get(src).copied().unwrap_or(0x7f);
            // The native asset omits the ROM terminator. It is still a real
            // decoder instruction and store, followed by SEP and RTS.
            b.emit(0xc50b, 48);
            if byte >= 0x80 {
                let word = dictionary.get(usize::from(byte.wrapping_sub(0x88) & 0x7f))?;
                if word.is_empty() {
                    return None;
                }
                b.block(&[
                    (0xc50d, 22),
                    (0xc530, 14),
                    (0xc531, 16),
                    (0xc533, 46),
                    (0xc6da, 22),
                ]);
                src += 1;
                b.write(0xc6dc, 62, SourcePosition(src.try_into().ok()?));
                b.block(&[
                    (0xc6df, 40),
                    (0xc6e2, 14),
                    (0xc6e3, 24),
                    (0xc6e6, 14),
                    (0xc6e7, 46),
                    (0xc6ea, 32),
                    (0xc6ec, 46),
                    (0xc6ef, 14),
                    (0xc6f0, 22),
                ]);
                for (i, &value) in word.iter().enumerate() {
                    b.emit(0xc6f2, 38);
                    b.write(
                        0xc6f5,
                        40,
                        Buffer {
                            offset: dst,
                            value: u16::from(value),
                            width: 1,
                        },
                    );
                    dst = dst.checked_add(1)?;
                    b.block(&[(0xc6f9, 14), (0xc6fa, 14), (0xc6fb, 32)]);
                    b.emit(0xc6fd, if i + 1 == word.len() { 16 } else { 22 });
                }
                b.write(0xc6ff, 40, OutputPosition(dst));
                b.block(&[(0xc702, 42), (0xc536, 40), (0xc539, 40), (0xc53c, 22)]);
                continue;
            }
            b.block(&[(0xc50d, 16), (0xc50f, 16)]);
            if byte < 0x67 {
                b.emit(0xc511, 16);
                b.write(
                    0xc513,
                    40,
                    Buffer {
                        offset: dst,
                        value: u16::from(byte),
                        width: 1,
                    },
                );
                src += 1;
                b.emit(0xc517, 14);
                b.write(0xc518, 40, SourcePosition(src.try_into().ok()?));
                dst = dst.checked_add(1)?;
                b.emit(0xc51b, 14);
                b.write(0xc51c, 40, OutputPosition(dst));
                b.emit(0xc51f, 22);
                continue;
            }
            b.block(&[(0xc511, 22), (0xc521, 16)]);
            if byte == 0x7f {
                b.block(&[(0xc523, 22), (0xc53e, 16)]);
                b.write(
                    0xc540,
                    40,
                    Buffer {
                        offset: dst,
                        value: 0x7f,
                        width: 1,
                    },
                );
                b.block(&[(0xc544, 22), (0xc546, 42)]);
                break;
            }
            b.block(&[
                (0xc523, 16),
                (0xc525, 46),
                (0xc547, 22),
                (0xc549, 16),
                (0xc54b, 62),
            ]);
            b.jump_table();
            let parameter = message.get(src + 1).copied().unwrap_or(0x7f);
            match byte {
                0x6a => {
                    b.name(name, &mut src, &mut dst)?;
                }
                0x6b => {
                    b.block(&[(0xc657, 22), (0xc659, 40), (0xc65c, 14), (0xc65d, 48)]);
                    b.write(0xc65f, 32, Window(parameter));
                    src += 2;
                    b.emit(0xc662, 14);
                    b.write(0xc663, 40, SourcePosition(src.try_into().ok()?));
                    b.emit(0xc666, 42);
                }
                0x6c => {
                    let pair = *number_pairs.get(usize::from(parameter >> 1))?;
                    let nibble = if parameter & 1 != 0 {
                        pair >> 4
                    } else {
                        pair & 15
                    };
                    b.block(&[
                        (0xc667, 22),
                        (0xc669, 40),
                        (0xc66c, 40),
                        (0xc66f, 56),
                        (0xc671, 14),
                        (0xc672, 14),
                    ]);
                    src += 2;
                    b.write(0xc673, 40, SourcePosition(src.try_into().ok()?));
                    b.block(&[
                        (0xc676, 20),
                        (0xc677, 24),
                        (0xc67a, 14),
                        (0xc67b, 22),
                        (0xc67c, 14),
                        (0xc67d, 46),
                        (0xc680, 28),
                    ]);
                    b.emit(0xc681, if parameter & 1 == 0 { 22 } else { 16 });
                    if parameter & 1 != 0 {
                        b.block(&[(0xc683, 14), (0xc684, 14), (0xc685, 14), (0xc686, 14)]);
                    }
                    b.block(&[(0xc687, 24), (0xc68a, 14), (0xc68b, 24), (0xc68e, 24)]);
                    b.write(
                        0xc691,
                        48,
                        Buffer {
                            offset: dst,
                            value: (u16::from(nibble) + 4) | 0x30,
                            width: 2,
                        },
                    );
                    dst = dst.checked_add(1)?;
                    b.emit(0xc695, 14);
                    b.write(0xc696, 40, OutputPosition(dst));
                    b.block(&[(0xc699, 22), (0xc69b, 42)]);
                }
                0x6d => {
                    let position = *[0x6125, 0x6244].get(usize::from(parameter))?;
                    b.block(&[
                        (0xc69c, 22),
                        (0xc69e, 40),
                        (0xc6a1, 14),
                        (0xc6a2, 56),
                        (0xc6a4, 24),
                        (0xc6a7, 14),
                        (0xc6a8, 14),
                        (0xc6a9, 46),
                    ]);
                    b.write(0xc6ac, 40, Position(position));
                    src += 2;
                    b.emit(0xc6af, 14);
                    b.write(0xc6b0, 40, SourcePosition(src.try_into().ok()?));
                    b.block(&[(0xc6b3, 22), (0xc6b5, 42)]);
                }
                0x77 => {
                    b.block(&[
                        (0xc6b6, 22),
                        (0xc6b8, 40),
                        (0xc6bb, 56),
                        (0xc6bd, 14),
                        (0xc6be, 14),
                        (0xc6bf, 24),
                        (0xc6c2, 32),
                        (0xc6c4, 24),
                        (0xc6c7, 24),
                        (0xc6ca, 24),
                        (0xc6cd, 32),
                    ]);
                    b.write(
                        0xc6cf,
                        40,
                        Color(0x2180 | ((u16::from(parameter) << 10) & 0x3c00)),
                    );
                    src += 2;
                    b.block(&[(0xc6d2, 14), (0xc6d3, 14)]);
                    b.write(0xc6d4, 40, SourcePosition(src.try_into().ok()?));
                    b.block(&[(0xc6d7, 22), (0xc6d9, 42)]);
                }
                0x6e | 0x78..=0x7a => {
                    b.block(&[(0xc598, 22), (0xc59a, 40), (0xc59d, 40), (0xc5a0, 56)]);
                    b.write(
                        0xc5a2,
                        48,
                        Buffer {
                            offset: dst,
                            value: u16::from_le_bytes([byte, parameter]),
                            width: 2,
                        },
                    );
                    src += 2;
                    dst = dst.checked_add(2)?;
                    b.block(&[(0xc5a6, 14), (0xc5a7, 14), (0xc5a8, 14), (0xc5a9, 14)]);
                    b.write(0xc5aa, 40, OutputPosition(dst));
                    b.write(0xc5ad, 40, SourcePosition(src.try_into().ok()?));
                    b.block(&[(0xc5b0, 22), (0xc5b2, 42)]);
                }
                _ => {
                    b.block(&[(0xc581, 22), (0xc583, 40), (0xc586, 40), (0xc589, 48)]);
                    b.write(
                        0xc58b,
                        40,
                        Buffer {
                            offset: dst,
                            value: u16::from(byte),
                            width: 1,
                        },
                    );
                    src += 1;
                    dst = dst.checked_add(1)?;
                    b.block(&[(0xc58f, 14), (0xc590, 14)]);
                    b.write(0xc591, 40, OutputPosition(dst));
                    b.write(0xc594, 40, SourcePosition(src.try_into().ok()?));
                    b.emit(0xc597, 42);
                }
            }
            b.block(&[(0xc528, 40), (0xc52b, 40), (0xc52e, 22)]);
        }
        Some(Self {
            instructions: b.0,
            progress: DecodeProgress::default(),
        })
    }
}

struct Builder(Vec<Instruction>);

impl Builder {
    fn emit(&mut self, pc: u16, master_cycles: u32) {
        let routine = match pc {
            0x8781..=0x8799 => 0x00_8781,
            0xc639..=0xc656 => 0x0e_c639,
            0xc547..=0xc6d9 => 0x0e_c547,
            0xc6da..=0xc702 => 0x0e_c6da,
            _ => 0x0e_c4e2,
        };
        self.0.push(Instruction {
            master_cycles,
            routine,
            publication: None,
            #[cfg(test)]
            pc: if pc < 0xc000 {
                u32::from(pc)
            } else {
                0x0e_0000 | u32::from(pc)
            },
        });
    }
    fn block(&mut self, steps: &[(u16, u32)]) {
        for &(pc, cost) in steps {
            self.emit(pc, cost);
        }
    }
    fn write(&mut self, pc: u16, cost: u32, publication: Publication) {
        self.emit(pc, cost);
        self.0.last_mut().unwrap().publication = Some(publication);
    }
    fn jump_table(&mut self) {
        self.block(&[
            (0x8781, 24),
            (0x8783, 28),
            (0x8784, 24),
            (0x8786, 22),
            (0x8788, 24),
            (0x878b, 14),
            (0x878c, 14),
            (0x878d, 36),
            (0x878e, 32),
            (0x8790, 14),
            (0x8791, 56),
            (0x8793, 32),
            (0x8795, 22),
            (0x8797, 24),
            (0x8799, 48),
        ]);
    }
    fn name(&mut self, name: [u16; 6], src: &mut usize, dst: &mut u16) -> Option<()> {
        use Publication::*;
        self.block(&[
            (0xc5b3, 22),
            (0xc5b5, 48),
            (0xc5b9, 14),
            (0xc5ba, 48),
            (0xc5be, 14),
            (0xc5bf, 24),
        ]);
        for i in 0..6 {
            self.block(&[
                (0xc5c2, 48),
                (0xc5c6, 30),
                (0xc5c7, 24),
                (0xc5ca, 46),
                (0xc5cd, 36),
                (0xc5ce, 14),
                (0xc5cf, 24),
                (0xc5d2, 46),
                (0xc5d5, 46),
                (0xc5d8, 14),
                (0xc5d9, 14),
                (0xc5da, 14),
                (0xc5db, 24),
            ]);
            self.emit(0xc5de, if i == 5 { 16 } else { 22 });
        }
        self.block(&[(0xc5e0, 22), (0xc5e2, 24)]);
        let mut letters = name.map(|word| ((word & 15) | ((word >> 1) & 0xfff0)) as u8);
        for (i, letter) in letters.iter_mut().enumerate() {
            self.block(&[(0xc5e5, 38), (0xc5e8, 46), (0xc639, 16)]);
            if *letter < 0x5f {
                self.block(&[(0xc63b, 22), (0xc653, 42)]);
            } else {
                self.block(&[(0xc63b, 16), (0xc63d, 16)]);
                if *letter >= 0x76 {
                    self.block(&[(0xc63f, 22), (0xc654, 16), (0xc656, 42)]);
                    *letter = letter.wrapping_sub(0x42);
                } else {
                    self.emit(0xc63f, 16);
                    for (pc, probe, replacement) in [
                        (0xc641, 0x5f, 0x08),
                        (0xc647, 0x60, 0x22),
                        (0xc64d, 0x61, 0x3e),
                    ] {
                        self.emit(pc, 16);
                        self.emit(pc + 2, if *letter == probe { 16 } else { 22 });
                        if *letter == probe {
                            self.emit(pc + 4, 16);
                            *letter = replacement;
                        }
                    }
                    self.emit(0xc653, 42);
                }
            }
            self.block(&[(0xc5eb, 38), (0xc5ee, 14), (0xc5ef, 24)]);
            self.emit(0xc5f2, if i == 5 { 16 } else { 22 });
        }
        self.block(&[
            (0xc5f4, 22),
            (0xc5f6, 40),
            (0xc5f9, 14),
            (0xc5fa, 24),
            (0xc5fd, 14),
        ]);
        *src += 1;
        self.write(0xc5fe, 62, SourcePosition((*src).try_into().ok()?));
        self.emit(0xc601, 22);
        for (i, &value) in letters.iter().enumerate() {
            let pc = 0xc603 + i as u16 * 6;
            self.emit(pc, 24);
            self.write(
                pc + 2,
                40,
                Buffer {
                    offset: dst.checked_add(i as u16)?,
                    value: u16::from(value),
                    width: 1,
                },
            );
        }
        self.emit(0xc627, 24);
        let mut length = 6;
        for i in (0..6).rev() {
            self.block(&[(0xc62a, 38), (0xc62d, 16)]);
            if letters[i] != 0x59 {
                self.emit(0xc62f, 22);
                break;
            }
            self.block(&[(0xc62f, 16), (0xc631, 14), (0xc632, 14)]);
            self.emit(0xc633, if i == 0 { 16 } else { 22 });
            length -= 1;
        }
        *dst = dst.checked_add(length)?;
        self.write(0xc635, 40, OutputPosition(*dst));
        self.emit(0xc638, 42);
        Some(())
    }
}
