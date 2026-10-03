//! Native InitializeTilesets and LoadCommonSprites call composition. Every
//! callee retains its own instruction progress; the enclosing game-thread
//! clock decides when to suspend. All selections and bytes are native inputs.

use super::graphics_convert::{ConversionWork, Publication as ConversionPublication};
use super::graphics_load::{
    Instruction as SheetInstruction, Publication as SheetPublication, SheetWork,
};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Publication {
    Graphics {
        destination: usize,
        publication: SheetPublication,
    },
    PpuByte {
        register: u8,
        value: u8,
    },
    VramAddress(u16),
    SpriteSubset {
        slot: u8,
        value: u8,
    },
    BackgroundSubset {
        slot: u8,
        value: u8,
    },
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub routine: u32,
    pub publication: Option<Publication>,
    #[cfg(test)]
    pub pc: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum Operation {
    Caller {
        instructions: Vec<Instruction>,
        next: usize,
    },
    RawConversion {
        work: ConversionWork,
        source: Vec<u8>,
        base: u32,
        destination: usize,
        high: bool,
    },
    Sheet {
        work: SheetWork,
        destination: usize,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct TilesetWork {
    operations: Vec<Operation>,
    next: usize,
}

pub(crate) struct Inputs {
    pub module: u8,
    pub misc: u8,
    pub main: [u8; 8],
    pub aux: [u8; 4],
    pub sprites: [u8; 4],
    pub previous_sprites: [u8; 4],
    pub main_index: u8,
}

impl TilesetWork {
    pub(crate) fn common(
        module: u8,
        misc: u8,
        mut asset: impl FnMut(u8) -> Option<Vec<u8>>,
    ) -> Option<Self> {
        let mut work = Self {
            operations: Vec::new(),
            next: 0,
        };
        work.append_common(module, misc, &mut asset)?;
        Some(work)
    }

    fn append_common(
        &mut self,
        module: u8,
        misc: u8,
        asset: &mut impl FnMut(u8) -> Option<Vec<u8>>,
    ) -> Option<()> {
        // Raw art is twelve contiguous 0x600-byte sheets spanning LoROM
        // $10:F000..$11:B7FF. The conversion's 16-bit pointer does not use
        // GetNextByte's bank-increment path when it wraps.
        assert!(misc < 12, "common sprites select a raw art sheet");
        let linear = 0x7000 + u32::from(misc) * 0x600;
        let base = ((0x10 + linear / 0x8000) << 16) | (0x8000 + linear % 0x8000);
        let mut b = Builder::new(0xe6b7);
        b.emit(0xe6b7, 32);
        for (pc, table_low, byte, address) in [
            (0xe6ba, 0xf3u16, (base >> 16) as u8, 2),
            (0xe6bf, 0xd2, (base >> 8) as u8, 1),
            (0xe6c4, 0xb1, base as u8, 0),
        ] {
            b.emit(pc, 32 + u32::from(table_low + u16::from(misc) > 255) * 6);
            b.byte(pc + 3, 24, address, byte);
        }
        b.block(&[(0xe6c9, 22), (0xe6cb, 16)]);
        self.caller(b);
        self.operations.push(Operation::RawConversion {
            work: ConversionWork::common_high(&[]),
            source: asset(misc)?,
            base,
            destination: 0x4400,
            high: true,
        });
        let mut b = Builder::new(0xe6b7);
        b.block(&[
            (0xe724, 22),
            (0xe726, 24),
            (0xe728, 16),
            (0xe72a, if module == 1 { 22 } else { 16 }),
        ]);
        if module != 1 {
            b.block(&[(0xe72c, 16), (0xe72e, 32)]);
            b.byte(0xe731, 24, 2, 0x11);
            b.emit(0xe733, 32);
            b.byte(0xe736, 24, 1, 0x94);
            b.emit(0xe738, 32);
            b.byte(0xe73b, 24, 0, 0);
            b.block(&[(0xe73d, 22), (0xe73f, 16), (0xe741, 24)]);
            self.caller(b);
            // The native raw window includes sheet 7 immediately after 6.
            self.operations.push(Operation::RawConversion {
                work: ConversionWork::new(false, &[], 128),
                source: asset(6)?,
                base: 0x119400,
                destination: 0x4800,
                high: false,
            });
        } else {
            b.byte(0xe744, 24, 0x0f, 0);
            b.emit(0xe746, 16);
            b.byte(0xe748, 24, 0x06, 0x5e);
            b.emit(0xe74a, 16);
            b.byte(0xe74c, 24, 2, 0x7f);
            b.block(&[(0xe74e, 16), (0xe750, 46)]);
            self.caller(b);
            self.operations.push(Operation::Sheet {
                work: SheetWork::sprite(0x5e, 0x14000, &asset(0x5e)?),
                destination: 0x4800,
            });
            let mut b = Builder::new(0xe6b7);
            b.emit(0xe753, 16);
            b.byte(0xe755, 24, 6, 0x5f);
            b.block(&[(0xe757, 16), (0xe759, 24)]);
            self.caller(b);
            self.operations.push(Operation::Sheet {
                work: SheetWork::sprite(0x5f, 0x14000, &asset(0x5f)?),
                destination: 0x4c00,
            });
        }
        Some(())
    }

    pub(crate) fn initialize(
        inputs: Inputs,
        mut asset: impl FnMut(bool, u8) -> Option<Vec<u8>>,
    ) -> Option<Self> {
        let mut work = Self {
            operations: Vec::new(),
            next: 0,
        };
        let mut b = Builder::new(0xe19b);
        b.block(&[(0xe19b, 22), (0xe19c, 22), (0xe19d, 28), (0xe19e, 16)]);
        b.write(
            0xe1a0,
            30,
            Publication::PpuByte {
                register: 0x15,
                value: 0x80,
            },
        );
        b.write(
            0xe1a3,
            30,
            Publication::PpuByte {
                register: 0x16,
                value: 0,
            },
        );
        b.emit(0xe1a6, 16);
        b.write(
            0xe1a8,
            30,
            Publication::PpuByte {
                register: 0x17,
                value: 0x44,
            },
        );
        b.emit(0xe1ab, 46);
        work.caller(b);
        work.append_common(inputs.module, inputs.misc, &mut |sheet| asset(true, sheet))?;
        let mut b = Builder::new(0xe19b);
        b.block(&[
            (0xe1ae, 22),
            (0xe1b0, 40),
            (0xe1b3, 24),
            (0xe1b6, 14),
            (0xe1b7, 14),
            (0xe1b8, 14),
            (0xe1b9, 22),
        ]);
        let selected = std::array::from_fn::<_, 4, _>(|i| {
            if inputs.sprites[i] == 0 {
                inputs.previous_sprites[i]
            } else {
                inputs.sprites[i]
            }
        });
        for slot in 0..4u8 {
            let pc = 0xe1bb + u32::from(slot) * 15;
            let value = selected[usize::from(slot)];
            b.emit(pc, 38);
            b.emit(
                pc + 3,
                if inputs.sprites[usize::from(slot)] == 0 {
                    22
                } else {
                    16
                },
            );
            if inputs.sprites[usize::from(slot)] != 0 {
                b.write(pc + 5, 40, Publication::SpriteSubset { slot, value });
            }
            b.emit(pc + 9, 40);
            b.byte(pc + 13, 24, 9 - usize::from(slot), value);
        }
        b.block(&[(0xe1f7, 22), (0xe1f9, 24), (0xe1fb, 16)]);
        b.byte(0xe1fd, 24, 2, 0x7e);
        b.block(&[(0xe1ff, 16), (0xe201, 46)]);
        work.caller(b);
        for slot in 0..4usize {
            if slot != 0 {
                let pc = 0xe204 + (slot as u32 - 1) * 7;
                let mut b = Builder::new(0xe19b);
                b.block(&[(pc, 24), (pc + 2, 16), (pc + 4, 46)]);
                work.caller(b);
            }
            work.operations.push(Operation::Sheet {
                work: SheetWork::sprite(
                    selected[slot],
                    0x7800 + slot * 0x600,
                    &asset(true, selected[slot])?,
                ),
                destination: 0x5000 + slot * 0x400,
            });
        }
        let mut b = Builder::new(0xe19b);
        b.block(&[(0xe219, 22), (0xe21b, 24)]);
        b.write(0xe21e, 36, Publication::VramAddress(0x2000));
        b.block(&[
            (0xe221, 40),
            (0xe224, 24),
            (0xe227, 14),
            (0xe228, 14),
            (0xe229, 14),
            (0xe22a, 14),
            (0xe22b, 40),
            (0xe22e, 24),
            (0xe231, 14),
            (0xe232, 14),
            (0xe233, 14),
            (0xe234, 22),
        ]);
        for slot in 0..3usize {
            b.emit(0xe236 + slot as u32 * 5, 38);
            b.byte(0xe239 + slot as u32 * 5, 24, 13 - slot, inputs.main[slot]);
        }
        let background: [u8; 8] = std::array::from_fn(|slot| {
            if (3..7).contains(&slot) && inputs.aux[slot - 3] != 0 {
                inputs.aux[slot - 3]
            } else {
                inputs.main[slot]
            }
        });
        for slot in 0..4u8 {
            let pc = 0xe245 + u32::from(slot) * 14;
            let value = background[usize::from(slot) + 3];
            b.emit(pc, 38);
            b.emit(
                pc + 3,
                if inputs.aux[usize::from(slot)] != 0 {
                    22
                } else {
                    16
                },
            );
            if inputs.aux[usize::from(slot)] == 0 {
                b.emit(pc + 5, 38);
            }
            b.write(pc + 8, 40, Publication::BackgroundSubset { slot, value });
            b.byte(pc + 12, 24, 10 - usize::from(slot), value);
        }
        b.emit(0xe27d, 38);
        b.byte(0xe280, 24, 6, background[7]);
        b.block(&[(0xe282, 22), (0xe284, 16)]);
        b.byte(0xe286, 24, 0x0f, 7);
        b.block(&[(0xe288, 24), (0xe28a, 46)]);
        work.caller(b);
        for slot in 0..8usize {
            let destination = if (3..7).contains(&slot) {
                0x6000 + (slot - 3) * 0x600
            } else {
                0x14000
            };
            if slot != 0 {
                let pc = [0, 0xe28d, 0xe294, 0xe29b, 0xe2a6, 0xe2b1, 0xe2bc, 0xe2c7][slot];
                let mut b = Builder::new(0xe19b);
                b.byte(pc, 38, 0x0f, (7 - slot) as u8);
                b.emit(pc + 2, 24);
                if (3..7).contains(&slot) {
                    b.block(&[(pc + 4, 16), (pc + 6, 16), (pc + 8, 46)]);
                } else {
                    b.emit(pc + 4, 46);
                }
                work.caller(b);
            }
            work.operations.push(Operation::Sheet {
                work: SheetWork::background(
                    background[slot],
                    destination,
                    &asset(false, background[slot])?,
                    inputs.main_index,
                    (7 - slot) as u8,
                ),
                destination: 0x2000 + slot * 0x400,
            });
        }
        let mut b = Builder::new(0xe19b);
        b.block(&[(0xe2ce, 28), (0xe2cf, 44)]);
        work.caller(b);
        Some(work)
    }

    fn caller(&mut self, builder: Builder) {
        self.operations.push(Operation::Caller {
            instructions: builder.instructions,
            next: 0,
        });
    }

    pub(crate) fn step(&mut self, ram: &[u8]) -> Option<Instruction> {
        while let Some(operation) = self.operations.get_mut(self.next) {
            let step = match operation {
                Operation::Caller { instructions, next } => {
                    instructions.get(*next).copied().inspect(|_| *next += 1)
                }
                Operation::Sheet { work, destination } => {
                    work.step(ram).map(|step| convert_step(step, *destination))
                }
                Operation::RawConversion {
                    work,
                    source,
                    base,
                    destination,
                    high,
                } => {
                    let step = work.step_with_reader(|offset| {
                        let byte = |offset: usize| {
                            let low = ((*base as usize & 0xffff) + offset) & 0xffff;
                            if low < 0x2000 {
                                ram[low]
                            } else {
                                source.get(offset).copied().unwrap_or(0)
                            }
                        };
                        u16::from_le_bytes([
                            byte(usize::from(offset)),
                            byte(usize::from(offset) + 1),
                        ])
                    });
                    step.map(|step| Instruction {
                        master_cycles: step.master_cycles,
                        routine: if *high { 0xe6b7 } else { 0xe63c },
                        publication: step.publication.map(|p| Publication::Graphics {
                            destination: *destination,
                            publication: match p {
                                ConversionPublication::SourceOffset(offset) => {
                                    SheetPublication::SourcePointer(
                                        (*base as u16).wrapping_add(offset),
                                    )
                                }
                                p => SheetPublication::Conversion(p),
                            },
                        }),
                        #[cfg(test)]
                        pc: Some(step.pc),
                    })
                }
            };
            if step.is_some() {
                return step;
            }
            self.next += 1;
        }
        None
    }

    #[cfg(test)]
    pub(crate) fn roundtrip_progress(&mut self) {
        self.next = bincode::deserialize(&bincode::serialize(&self.next).unwrap()).unwrap();
        match self.operations.get_mut(self.next) {
            Some(Operation::Caller { next, .. }) => {
                *next = bincode::deserialize(&bincode::serialize(next).unwrap()).unwrap()
            }
            Some(Operation::RawConversion { work, .. }) => work.roundtrip_progress(),
            Some(Operation::Sheet { work, .. }) => work.roundtrip_progress(),
            None => {}
        }
    }
}

fn convert_step(step: SheetInstruction, destination: usize) -> Instruction {
    Instruction {
        master_cycles: step.master_cycles,
        routine: step.routine,
        publication: step.publication.map(|publication| Publication::Graphics {
            destination,
            publication,
        }),
        #[cfg(test)]
        pc: step.pc,
    }
}

struct Builder {
    routine: u32,
    instructions: Vec<Instruction>,
}
impl Builder {
    fn new(routine: u32) -> Self {
        Self {
            routine,
            instructions: Vec::new(),
        }
    }
    fn emit(&mut self, _pc: u32, master_cycles: u32) {
        self.instructions.push(Instruction {
            master_cycles,
            routine: self.routine,
            publication: None,
            #[cfg(test)]
            pc: Some(_pc),
        });
    }
    fn block(&mut self, steps: &[(u32, u32)]) {
        for &(pc, cost) in steps {
            self.emit(pc, cost);
        }
    }
    fn write(&mut self, pc: u32, cost: u32, publication: Publication) {
        self.emit(pc, cost);
        self.instructions.last_mut().unwrap().publication = Some(publication);
    }
    fn byte(&mut self, pc: u32, cost: u32, address: usize, value: u8) {
        self.write(
            pc,
            cost,
            Publication::Graphics {
                destination: 0,
                publication: SheetPublication::Byte { address, value },
            },
        );
    }
}
