//! Native sheet-loader continuation. The work composes the loader's own
//! instructions, live-buffer decompression, and the shared conversion. Its
//! caller supplies the hardware clock; no source CPU progress enters here.

use super::decompress::{
    decompress_instruction_timeline, stream_source_offset, DecompressAction, DecompressEntry,
    DecompressInstruction,
};
use super::graphics_convert::{ConversionWork, Publication as ConversionPublication};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Publication {
    Byte { address: usize, value: u8 },
    SourcePointer(u16),
    Conversion(ConversionPublication),
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub routine: u32,
    pub publication: Option<Publication>,
    #[cfg(any(test, feature = "rom-shadow-audit"))]
    #[serde(skip)]
    pub pc: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct SheetWork {
    destination: usize,
    high: bool,
    decompressor: u32,
    prefix: Vec<Instruction>,
    decompression: Vec<DecompressInstruction>,
    suffix: Vec<Instruction>,
    conversion: Option<ConversionWork>,
    progress: Progress,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct Progress {
    phase: u8,
    next: usize,
    back_reference: u8,
}

impl SheetWork {
    pub(crate) fn profile_call_path(&self, routine: u32) -> Vec<u32> {
        if !crate::cycle_ledger::profiling() { return Vec::new(); }
        let mut path = vec![self.prefix[0].routine];
        if routine == 0xe772 || routine == 0xe78f || routine == 0xe843 {
            path.push(self.decompressor);
        }
        if routine == 0xe843 { path.push(routine); }
        path
    }

    pub(crate) fn sprite(sheet: u8, destination: usize, stream: &[u8]) -> Self {
        let mut prefix = Builder::new(0xe583);
        prefix.byte(0xe583, 24, 0, 0);
        prefix.byte(0xe585, 24, 1, (destination >> 8) as u8);
        prefix.block(&[(0xe587, 22), (0xe588, 46)]);
        let mut suffix = Builder::new(0xe583);
        suffix.block(&[(0xe58b, 22), (0xe58d, 16), (0xe58f, 28)]);
        let mut high = false;
        for (index, candidate) in [0x52, 0x53, 0x5a, 0x5b, 0x5c, 0x5e, 0x5f]
            .into_iter()
            .enumerate()
        {
            let pc = 0xe590 + index as u32 * 4;
            suffix.block(&[(pc, 16), (pc + 2, if sheet == candidate { 22 } else { 16 })]);
            if sheet == candidate {
                high = true;
                break;
            }
        }
        if !high {
            suffix.block(&[(0xe5ac, 24)]);
        }
        Self::new(
            destination,
            high,
            DecompressEntry::Sprite,
            sheet,
            stream,
            prefix,
            suffix,
        )
    }

    pub(crate) fn background(
        sheet: u8,
        destination: usize,
        stream: &[u8],
        theme: u8,
        slot: u8,
    ) -> Self {
        let mut prefix = Builder::new(if destination == 0x14000 {
            0xe609
        } else {
            0xe60d
        });
        if destination == 0x14000 {
            prefix.block(&[(0xe609, 16), (0xe60b, 16)]);
        }
        prefix.byte(0xe60d, 24, 0, 0);
        prefix.byte(0xe60f, 24, 1, (destination >> 8) as u8);
        prefix.byte(0xe611, 24, 2, 0x7e + (destination >> 16) as u8);
        prefix.block(&[(0xe613, 46)]);
        let mut suffix = Builder::new(prefix.routine);
        suffix.block(&[
            (0xe616, 22),
            (0xe618, 16),
            (0xe61a, 32),
            (0xe61d, 16),
            (0xe61f, if theme < 0x20 { 22 } else { 16 }),
        ]);
        let high;
        if theme < 0x20 {
            high = slot >= 4;
            suffix.block(&[
                (0xe636, 24),
                (0xe638, 16),
                (0xe63a, if high { 22 } else { 16 }),
            ]);
            if high {
                suffix.block(&[(0xe633, 24)]);
            }
        } else {
            suffix.block(&[(0xe621, 24)]);
            let mut matched = false;
            for (pc, candidate) in [(0xe623, 7), (0xe627, 2), (0xe62b, 4)] {
                suffix.block(&[(pc, 16), (pc + 2, if slot == candidate { 22 } else { 16 })]);
                if slot == candidate {
                    matched = true;
                    break;
                }
            }
            high = matched || slot == 3;
            if !matched {
                suffix.block(&[(0xe62f, 16), (0xe631, if high { 16 } else { 22 })]);
                if high {
                    suffix.block(&[(0xe633, 24)]);
                }
            }
        }
        Self::new(
            destination,
            high,
            DecompressEntry::BackgroundTables,
            sheet,
            stream,
            prefix,
            suffix,
        )
    }

    fn new(
        destination: usize,
        high: bool,
        entry: DecompressEntry,
        sheet: u8,
        stream: &[u8],
        prefix: Builder,
        suffix: Builder,
    ) -> Self {
        assert_eq!(
            destination & 0xff,
            0,
            "sheet callers pass a page-aligned destination"
        );
        let (_, _, decompression) = decompress_instruction_timeline(
            entry,
            sheet,
            stream,
            stream_source_offset(entry, sheet),
            0x7e0000 + destination as u32,
        );
        Self {
            destination,
            high,
            decompressor: if entry == DecompressEntry::Sprite {
                0xe772
            } else {
                0xe78f
            },
            prefix: prefix.instructions,
            decompression,
            suffix: suffix.instructions,
            conversion: None,
            progress: Progress::default(),
        }
    }

    /// Return one retired instruction and its publication. Back-references
    /// sample live native RAM at LDA, retaining that value until the later
    /// STA. Conversion starts only after decompression returns and reads
    /// each word at its instruction, including the unmodified native tail.
    pub(crate) fn step(&mut self, read: impl Fn(usize) -> u8) -> Option<Instruction> {
        loop {
            match self.progress.phase {
                0 | 2 => {
                    let steps = if self.progress.phase == 0 {
                        &self.prefix
                    } else {
                        &self.suffix
                    };
                    if let Some(&step) = steps.get(self.progress.next) {
                        self.progress.next += 1;
                        return Some(step);
                    }
                }
                1 => {
                    if let Some(&step) = self.decompression.get(self.progress.next) {
                        self.progress.next += 1;
                        let publication = match step.action {
                            Some(DecompressAction::ReadBackReference(offset)) => {
                                // Long indirect indexed addressing carries into
                                // the next bank; only Y itself wraps at 16 bits.
                                self.progress.back_reference =
                                    read(self.destination + usize::from(offset));
                                None
                            }
                            Some(DecompressAction::StoreBackReference(offset)) => {
                                Some(Publication::Byte {
                                    address: self.destination + usize::from(offset),
                                    value: self.progress.back_reference,
                                })
                            }
                            Some(DecompressAction::StoreLiteral { offset, value }) => {
                                Some(Publication::Byte {
                                    address: self.destination + usize::from(offset),
                                    value,
                                })
                            }
                            None => None,
                        };
                        return Some(Instruction {
                            master_cycles: step.master_cycles,
                            routine: if step.in_get_next_byte {
                                0xe843
                            } else {
                                self.decompressor
                            },
                            publication,
                            #[cfg(any(test, feature = "rom-shadow-audit"))]
                            pc: step.pc,
                        });
                    }
                }
                3 => {
                    let conversion = self
                        .conversion
                        .get_or_insert_with(|| ConversionWork::new(self.high, &[], 64));
                    let step = conversion.step_with_reader(|offset| {
                        let address = (self.destination & !0xffff)
                            | ((self.destination + usize::from(offset)) & 0xffff);
                        u16::from_le_bytes([read(address), read(address + 1)])
                    })?;
                    let publication = step.publication.map(|p| match p {
                        ConversionPublication::SourceOffset(offset) => Publication::SourcePointer(
                            (self.destination as u16).wrapping_add(offset),
                        ),
                        p => Publication::Conversion(p),
                    });
                    return Some(Instruction {
                        master_cycles: step.master_cycles,
                        // $00:E5AC/E633 tail-jump to conversion: no new
                        // return address, so the ROM loader owns its clocks.
                        routine: self.prefix[0].routine,
                        publication,
                        #[cfg(any(test, feature = "rom-shadow-audit"))]
                        pc: step.audit_pc(),
                    });
                }
                _ => unreachable!(),
            }
            self.progress.phase += 1;
            self.progress.next = 0;
        }
    }

    #[cfg(test)]
    pub(crate) fn roundtrip_progress(&mut self) {
        self.progress = bincode::deserialize(&bincode::serialize(&self.progress).unwrap()).unwrap();
        if let Some(conversion) = &mut self.conversion {
            conversion.roundtrip_progress();
        }
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
    fn block(&mut self, steps: &[(u32, u32)]) {
        for &(_pc, master_cycles) in steps {
            self.instructions.push(Instruction {
                master_cycles,
                routine: self.routine,
                publication: None,
                #[cfg(any(test, feature = "rom-shadow-audit"))]
                pc: Some(_pc),
            });
        }
    }
    fn byte(&mut self, pc: u32, cycles: u32, address: usize, value: u8) {
        self.block(&[(pc, cycles)]);
        self.instructions.last_mut().unwrap().publication =
            Some(Publication::Byte { address, value });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_between_back_reference_load_and_store_retains_the_loaded_native_byte() {
        let mut ram = vec![0u8; 0x20000];
        ram[0x10800] = 0xa5;
        ram[0x0800] = 0x5a; // A 16-bit address wrap would read the wrong bank.
        let mut work = SheetWork::sprite(12, 0x7800, &[0x80, 0, 0x90, 0xff]);
        loop {
            let step = work.step(|address| ram[address]).unwrap();
            if let Some(Publication::Byte { address, value }) = step.publication {
                ram[address] = value;
            }
            if work.progress.phase == 1
                && matches!(
                    work.decompression[work.progress.next - 1].action,
                    Some(DecompressAction::ReadBackReference(0x9000))
                )
            {
                break;
            }
        }
        assert_eq!(ram[0x7800], 0, "LDA has not published the output store");
        let payload = bincode::serialize(&work).unwrap();
        ram[0x10800] = 0x99; // An intervening native owner changes the buffer.
        let mut restored: SheetWork = bincode::deserialize(&payload).unwrap();
        for _ in 0..2 {
            assert!(restored.step(|address| ram[address]).unwrap().publication.is_none()); // TYX, PLY
        }
        let store = restored.step(|address| ram[address]).unwrap();
        assert!(matches!(
            store.publication,
            Some(Publication::Byte {
                address: 0x7800,
                value: 0xa5
            })
        ));
        ram[0x7800] = 0x33; // Later owner; completed store must never replay.
        let mut tail_instructions = 0;
        while let Some(step) = restored.step(|address| ram[address]) {
            if let Some(Publication::Byte { address, value }) = step.publication {
                assert_ne!(address, 0x7800);
                ram[address] = value;
            }
            tail_instructions += 1;
        }
        assert!(
            tail_instructions > 1000,
            "the decompressor's last store is not the loader return"
        );
        assert_eq!(ram[0x7800], 0x33);
    }
}
