//! Native VWF row loops. Each step retires one instruction; loaded bitmap
//! bytes, shifted font bits, and loop decisions survive interruption.
//! This owner does not establish the caller's hardware clock.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Phase {
    Row,
    Column,
    Seed,
    RowEnd,
    BottomSetup,
    Return,
    Complete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Read {
    LineOffset,
    GlyphCursor,
    AdvanceWord(u16),
    BitmapByte(u16),
    ReadPosition,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom_cpu_timing::{lorom_offset, RomCpuCheckpoint, RomCpuTimingRun};

    #[test]
    fn drawing_instructions_match_source_at_every_serialized_boundary() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .expect("source ROM fixture");
        let font = &rom[lorom_offset(0x0e8000).unwrap()..];
        let mut boundaries = 0;
        for c in 0..0x63u8 {
            for advance in 0..8u8 {
                let width = c % 9;
                let line = [0, 0x20, 0x2a0][usize::from(c % 3)];
                let cursor = u16::from(c) + 1;
                let rows = crate::cycle_models::vwf::glyph_font_rows(font, c);
                let mut work = Drawing::new(c, width, advance, rows);
                let mut ram = vec![0u8; 0x20000];
                for (address, value) in [
                    (0, u16::from(advance) * 2),
                    (2, u16::from(width)),
                    (0xa, work.font_base),
                    (0xd, 0x8000),
                    (0x724, cursor),
                    (0x726, line),
                    (0xc22f + usize::from(cursor), 0xa500 | u16::from(advance)),
                    (0x1cd9, 0xffff),
                ] {
                    ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                }
                ram[0xf] = 0x0e;
                for (i, value) in ram[0x10000..0x11000].iter_mut().enumerate() {
                    *value = (i as u8).wrapping_mul(c.wrapping_add(1)) ^ advance;
                }
                let checkpoint = RomCpuCheckpoint {
                    entry_pc: 0x0ecbd1,
                    stop_pc: 0x0ecad8,
                    a: work.a,
                    x: 0,
                    y: work.y,
                    sp: 0x1fc,
                    dp: 0,
                    db: 0x0e,
                    carry: false,
                    zero: false,
                    overflow: false,
                    negative: false,
                    interrupt_disable: true,
                    decimal: false,
                    accumulator_is_8_bit: false,
                    index_is_8_bit: false,
                    emulation: false,
                    waiting: false,
                    stack_address: 0x1fd,
                    stack_bytes: &[0x0e, 0xd7, 0xca],
                };
                let mut source = RomCpuTimingRun::new(
                    &rom,
                    &ram,
                    &vec![0; 0x2000],
                    &snes::PpuState::default(),
                    &snes::DmaState::default(),
                    [0; 4],
                    checkpoint,
                )
                .unwrap();
                source.enable_cpu_write_trace();
                while !work.is_complete() {
                    work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
                    let pc = source.pc();
                    assert_eq!(work.next_pc(), pc);
                    let instruction = work
                        .step(|operand| {
                            let address = match operand {
                                Read::LineOffset => 0x726,
                                Read::GlyphCursor => 0x724,
                                Read::AdvanceWord(cursor) => 0xc22f + usize::from(cursor),
                                Read::ReadPosition => 0x1cd9,
                                Read::BitmapByte(offset) => {
                                    return u16::from(ram[0x10000 + usize::from(offset)])
                                }
                            };
                            u16::from_le_bytes([ram[address], ram[address + 1]])
                        })
                        .unwrap();
                    assert_eq!(
                        instruction.master_cycles,
                        source.step().master_cycles,
                        "c={c} advance={advance} pc={pc:06x}"
                    );
                    assert_eq!(work.next_pc(), source.pc(), "pc={pc:06x}");
                    assert_eq!(
                        (work.a, work.x, work.y),
                        (source.accumulator(), source.index_x(), source.index_y()),
                        "pc={pc:06x}"
                    );
                    let mut stores = Vec::new();
                    if let Some(write) = instruction.write {
                        let (address, bytes) = match write {
                            Write::BitmapByte(offset, value) => {
                                (0x10000 + usize::from(offset), vec![value])
                            }
                            Write::BitmapWord(offset, value) => {
                                (0x10000 + usize::from(offset), value.to_le_bytes().to_vec())
                            }
                            Write::ReadPosition(value) => (0x1cd9, value.to_le_bytes().to_vec()),
                        };
                        for (i, value) in bytes.into_iter().enumerate() {
                            ram[address + i] = value;
                            stores.push((address + i, value));
                        }
                        // The RMW instruction writes high then low; ordinary
                        // 16-bit STA writes low then high. Both retire atomically.
                        if matches!(write, Write::ReadPosition(_)) { stores.reverse(); }
                    }
                    let actual: Vec<_> = source
                        .take_cpu_wram_writes()
                        .into_iter()
                        .filter(|(address, _)| {
                            *address >= 0x10000 || (0x1cd9..0x1cdb).contains(address)
                        })
                        .collect();
                    assert_eq!(stores, actual, "c={c} advance={advance} pc={pc:06x}");
                    boundaries += 1;
                }
                assert!(source.is_complete());
                assert_eq!(
                    u64::from(work.elapsed()),
                    crate::cycle_models::vwf::glyph_drawing_master_cycles(width, advance, rows)
                );
                let completed = work;
                assert_eq!(work.step(|_| panic!("completed work read input")), None);
                assert_eq!(work, completed);
            }
        }
        eprintln!("VWF drawing: 792 fixtures, {boundaries} serialized instruction boundaries");
    }

    #[test]
    fn drawing_retains_bitmap_load_and_completed_instruction() {
        let mut work = Drawing::new(0, 1, 0, [0x8080; 16]);
        let mut bitmap = [0x55u8; 0x1000];
        while work.next_pc() != 0x0ecbfa {
            work.step(|read| match read {
                Read::BitmapByte(i) => u16::from(bitmap[usize::from(i)]),
                _ => 0,
            })
            .unwrap();
        }
        // NMI changed the destination after LDA, before EOR/STA. The saved A
        // must win; recomputing an XOR at publication would produce 0x2a.
        bitmap[0] = 0xaa;
        let cutoff = work.elapsed() + 1;
        work = bincode::deserialize(&bincode::serialize(&work).unwrap()).unwrap();
        while work.elapsed() < cutoff {
            work.step(|_| panic!("EOR needs no fresh bitmap read"))
                .unwrap();
        }
        assert_eq!(work.elapsed(), cutoff + 37);
        let store = work
            .step(|_| panic!("STA needs no fresh bitmap read"))
            .unwrap();
        assert_eq!(store.write, Some(Write::BitmapByte(0, 0xd5)));
        assert_ne!(store.write, Some(Write::BitmapByte(0, bitmap[0] ^ 0x80)));
        assert_eq!(work.next_pc(), 0x0ecc01);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Write {
    BitmapByte(u16, u8),
    BitmapWord(u16, u16),
    ReadPosition(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Instruction {
    pub master_cycles: u32,
    pub write: Option<Write>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Drawing {
    phase: Phase,
    instruction: u8,
    bottom: bool,
    a: u16,
    x: u16,
    y: u16,
    saved_y: u16,
    row: u16,
    top_advance: u16,
    bottom_base: u16,
    font_base: u16,
    width: u8,
    remaining: u8,
    bits: u16,
    set: bool,
    zero: bool,
    rows: [u16; 16],
    elapsed: u32,
}

impl Drawing {
    pub(crate) fn new(c: u8, width: u8, advance: u8, rows: [u16; 16]) -> Self {
        let font_base = u16::from((c & 0x70) * 2 + (c & 0x0f));
        Self {
            phase: Phase::Row,
            instruction: 0,
            bottom: false,
            a: font_base * 16,
            x: 0,
            y: font_base * 16,
            saved_y: 0,
            row: 0,
            top_advance: u16::from(advance) * 2,
            bottom_base: 0,
            font_base,
            width,
            remaining: 0,
            bits: 0,
            set: false,
            zero: false,
            rows,
            elapsed: 0,
        }
    }

    pub(crate) fn elapsed(self) -> u32 {
        self.elapsed
    }
    pub(crate) fn is_complete(self) -> bool {
        self.phase == Phase::Complete
    }

    fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.instruction = 0;
    }

    fn load_byte(&mut self, value: u8) {
        self.a = (self.a & 0xff00) | u16::from(value);
    }

    pub(crate) fn step(&mut self, mut read: impl FnMut(Read) -> u16) -> Option<Instruction> {
        let op = self.instruction;
        self.instruction += 1;
        let mut write = None;
        let cost = match self.phase {
            Phase::Row => {
                // The bottom loop reloads the glyph's advance before calculating
                // its bitmap address. The top loop uses its retained setup word.
                match op {
                    0 => {
                        let base = (self.font_base + if self.bottom { 16 } else { 0 }) * 16;
                        let index = usize::from(self.y.wrapping_sub(base) / 2)
                            + if self.bottom { 8 } else { 0 };
                        self.a = self.rows[index];
                        56
                    }
                    1 => {
                        self.bits = self.a;
                        32
                    }
                    2 => {
                        self.saved_y = self.y;
                        30
                    }
                    3 => {
                        self.row = self.x;
                        32
                    }
                    4 if self.bottom => {
                        self.x = read(Read::GlyphCursor);
                        40
                    }
                    5 if self.bottom => {
                        self.a = read(Read::AdvanceWord(self.x));
                        48
                    }
                    6 if self.bottom => {
                        self.a &= 0xff;
                        24
                    }
                    7 if self.bottom => {
                        self.a <<= 1;
                        14
                    }
                    8 if self.bottom => 14,
                    9 if self.bottom => {
                        self.a = self.a.wrapping_add(self.bottom_base);
                        32
                    }
                    4 => {
                        self.a = self.top_advance;
                        32
                    }
                    5 => 14,
                    6 => {
                        self.a = self.a.wrapping_add(read(Read::LineOffset));
                        40
                    }
                    _ => {
                        let shared = op - if self.bottom { 10 } else { 7 };
                        match shared {
                            0 => {
                                self.y = self.a;
                                14
                            }
                            1 => {
                                self.a &= 0x0ff0;
                                24
                            }
                            2 => 14,
                            3 => {
                                self.a = self.a.wrapping_add(self.row);
                                32
                            }
                            4 => {
                                self.x = self.a;
                                14
                            }
                            5 => {
                                self.a = self.y;
                                14
                            }
                            6 => {
                                self.a >>= 1;
                                14
                            }
                            7 => {
                                self.a &= 7;
                                24
                            }
                            8 => {
                                self.y = self.a;
                                14
                            }
                            9 => 22,
                            10 => {
                                self.load_byte(self.width);
                                24
                            }
                            11 => {
                                self.remaining = self.a as u8;
                                self.enter(Phase::Column);
                                24
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            Phase::Column => match op {
                0 | 6 => {
                    let shift = if op == 0 { 0 } else { 8 };
                    let byte = (self.bits >> shift) as u8;
                    self.set = byte & 0x80 != 0;
                    self.bits = (self.bits & !(0xff << shift)) | (u16::from(byte << 1) << shift);
                    38
                }
                1 | 7 => {
                    if self.set {
                        16
                    } else {
                        22
                    }
                }
                2 | 8 => {
                    self.load_byte(read(Read::BitmapByte(self.x + u16::from(op == 8))) as u8);
                    40
                }
                3 | 9 => {
                    let mask = 0x80u8 >> self.y;
                    self.load_byte(if self.set {
                        self.a as u8 ^ mask
                    } else {
                        self.a as u8 & !mask
                    });
                    38
                }
                4 | 10 => {
                    write = Some(Write::BitmapByte(
                        self.x + u16::from(op == 10),
                        self.a as u8,
                    ));
                    // The clear path falls through without a BRA instruction.
                    if !self.set {
                        self.instruction += 1;
                    }
                    40
                }
                5 | 11 => 22,
                12 => {
                    self.remaining = self.remaining.wrapping_sub(1);
                    self.zero = self.remaining == 0;
                    38
                }
                13 => {
                    if self.zero {
                        self.enter(Phase::Seed);
                        22
                    } else {
                        16
                    }
                }
                14 => {
                    self.y = self.y.wrapping_add(1);
                    14
                }
                15 => {
                    self.zero = self.y == 8;
                    24
                }
                16 => {
                    if self.zero {
                        self.enter(Phase::Seed);
                        16
                    } else {
                        self.enter(Phase::Column);
                        22
                    }
                }
                _ => unreachable!(),
            },
            Phase::Seed => match op {
                0 => 22,
                1 => {
                    self.a = self.x;
                    14
                }
                2 => 14,
                3 => {
                    self.a = self.a.wrapping_add(16);
                    24
                }
                4 => {
                    self.x = self.a;
                    14
                }
                5 => {
                    self.a = self.bits;
                    self.zero = self.a == 0;
                    32
                }
                6 => {
                    if self.zero {
                        self.enter(Phase::RowEnd);
                        22
                    } else {
                        16
                    }
                }
                7 => {
                    write = Some(Write::BitmapWord(self.x, self.a));
                    self.enter(Phase::RowEnd);
                    48
                }
                _ => unreachable!(),
            },
            Phase::RowEnd => match op {
                0 => {
                    self.y = self.saved_y;
                    36
                }
                1 | 2 => {
                    self.y = self.y.wrapping_add(1);
                    14
                }
                3 => {
                    self.x = self.row;
                    32
                }
                4 | 5 => {
                    self.x = self.x.wrapping_add(1);
                    14
                }
                6 => {
                    self.zero = self.x == 16;
                    24
                }
                7 if self.bottom => {
                    if self.zero {
                        self.enter(Phase::Return);
                        22
                    } else {
                        16
                    }
                }
                7 => {
                    if self.zero {
                        self.enter(Phase::BottomSetup);
                        16
                    } else {
                        self.enter(Phase::Row);
                        22
                    }
                }
                8 => {
                    self.enter(Phase::Row);
                    30
                }
                _ => unreachable!(),
            },
            Phase::BottomSetup => match op {
                0 => {
                    self.a = read(Read::LineOffset);
                    40
                }
                1 | 6 => 14,
                2 => {
                    self.a = self.a.wrapping_add(0x150);
                    24
                }
                3 => {
                    self.bottom_base = self.a;
                    32
                }
                4 => {
                    self.x = 0;
                    24
                }
                5 => {
                    self.a = self.font_base;
                    32
                }
                7 => {
                    self.a = self.a.wrapping_add(16);
                    24
                }
                8..=11 => {
                    self.a <<= 1;
                    14
                }
                12 => {
                    self.y = self.a;
                    self.bottom = true;
                    self.enter(Phase::Row);
                    14
                }
                _ => unreachable!(),
            },
            Phase::Return => match op {
                0 => {
                    write = Some(Write::ReadPosition(
                        read(Read::ReadPosition).wrapping_add(1),
                    ));
                    62
                }
                1 => {
                    self.x &= 0xff;
                    self.y &= 0xff;
                    22
                }
                2 => 28,
                3 => {
                    self.enter(Phase::Complete);
                    42
                }
                _ => unreachable!(),
            },
            Phase::Complete => {
                self.instruction = 0;
                return None;
            }
        };
        self.elapsed += cost;
        Some(Instruction {
            master_cycles: cost,
            write,
        })
    }

    #[cfg(test)]
    fn next_pc(self) -> u32 {
        let op = usize::from(self.instruction);
        let pc = match self.phase {
            Phase::Row if self.bottom => [
                0xcc67, 0xcc69, 0xcc6b, 0xcc6c, 0xcc6e, 0xcc71, 0xcc75, 0xcc78, 0xcc79, 0xcc7a,
                0xcc7c, 0xcc7d, 0xcc80, 0xcc81, 0xcc83, 0xcc84, 0xcc85, 0xcc86, 0xcc89, 0xcc8a,
                0xcc8c, 0xcc8e,
            ][op],
            Phase::Row => [
                0xcbd1, 0xcbd3, 0xcbd5, 0xcbd6, 0xcbd8, 0xcbda, 0xcbdb, 0xcbde, 0xcbdf, 0xcbe2,
                0xcbe3, 0xcbe5, 0xcbe6, 0xcbe7, 0xcbe8, 0xcbeb, 0xcbec, 0xcbee, 0xcbf0,
            ][op],
            Phase::Column => {
                let mut pc = [
                    0xcbf2, 0xcbf4, 0xcbf6, 0xcbfa, 0xcbfd, 0xcc01, 0xcc0e, 0xcc10, 0xcc12, 0xcc16,
                    0xcc19, 0xcc1d, 0xcc2a, 0xcc2c, 0xcc2e, 0xcc2f, 0xcc32,
                ][op];
                if matches!(op, 2..=4 | 8..=10) && !self.set {
                    pc += 13;
                }
                pc + if self.bottom { 0x9e } else { 0 }
            }
            Phase::Seed => {
                [
                    0xcc34, 0xcc36, 0xcc37, 0xcc38, 0xcc3b, 0xcc3c, 0xcc3e, 0xcc40,
                ][op]
                    + if self.bottom { 0x9e } else { 0 }
            }
            Phase::RowEnd => {
                [
                    0xcc44, 0xcc45, 0xcc46, 0xcc47, 0xcc49, 0xcc4a, 0xcc4b, 0xcc4e, 0xcc50,
                ][op]
                    + if self.bottom { 0x9e } else { 0 }
            }
            Phase::BottomSetup => [
                0xcc50, 0xcc53, 0xcc54, 0xcc57, 0xcc59, 0xcc5c, 0xcc5e, 0xcc5f, 0xcc62, 0xcc63,
                0xcc64, 0xcc65, 0xcc66,
            ][op],
            Phase::Return => [0xccf1, 0xccf4, 0xccf6, 0xccf7][op],
            Phase::Complete => 0xcad8,
        };
        0x0e0000 | pc
    }
}
