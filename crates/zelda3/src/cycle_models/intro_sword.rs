//! Sword setup, drawing, flash and sparkle continuations on the live native bus.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroSword {
    pc: u32,
    complete: bool,
}
impl IntroSword {
    pub(crate) fn setup() -> Self {
        Self {
            pc: 0x0cfe45,
            complete: false,
        }
    }
    pub(crate) fn periodic() -> Self {
        Self {
            pc: 0x0cfe56,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        assert_eq!(cpu.d, 0);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let width = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let next = match self.pc {
            0x0cfe45 | 0x0cfe6e | 0x0cfe86 | 0x0cfe91 | 0x0cfead | 0x0cff20 | 0x0cff32
            | 0x0cff37 | 0x0cff3c | 0x0cff5d | 0x0cff6b | 0x0cff7c | 0x0ed60b | 0x0ed60f
            | 0x0ed613 => {
                let value = match self.pc {
                    0x0cfe45 => 7,
                    0x0cfe6e => 31,
                    0x0cfe86 | 0x0cff20 => 2,
                    0x0cfe91 => 0x21,
                    0x0cfead => 0xf8,
                    0x0cff32 => 0x44,
                    0x0cff37 => 0x43,
                    0x0cff3c => 0x25,
                    0x0cff5d => 0x42,
                    0x0cff6b => 0x4f,
                    0x0cff7c => 0x23,
                    0x0ed60b => 0x20,
                    0x0ed60f => 0x40,
                    _ => 0x80,
                };
                w.immediate(value, 1);
                self.pc + 2
            }
            0x0cfe47 | 0x0cfe49 | 0x0cfe4b | 0x0cfe52 | 0x0cfed0 | 0x0cfee0 | 0x0cff1e
            | 0x0cff22 | 0x0cff2d | 0x0cff99 | 0x0ed60d | 0x0ed611 | 0x0ed615 => {
                let address = match self.pc {
                    0x0cfe47 | 0x0cff1e => 0xcb,
                    0x0cfe49 => 0xcc,
                    0x0cfe4b | 0x0cff99 => 0xcd,
                    0x0cfe52 | 0x0cfee0 => 0xc8,
                    0x0cfed0 => 0xd0,
                    0x0cff22 | 0x0cff2d => 0xca,
                    0x0ed60d => 0x9c,
                    0x0ed611 => 0x9d,
                    _ => 0x9e,
                };
                let value = if matches!(self.pc, 0x0cfe49 | 0x0cfe4b | 0x0cfed0 | 0x0cff1e) {
                    0
                } else {
                    w.cpu.a & if width == 1 { 255 } else { 65535 }
                };
                w.dp_store(address, value, width);
                self.pc + 2
            }
            0x0cfe4d | 0x0cfea0 | 0x0cfebb | 0x0cfef4 => {
                w.width(0x20, true);
                self.pc + 2
            }
            0x0cfe54 | 0x0cfea8 | 0x0cfee2 | 0x0cfefb => {
                w.width(0x20, false);
                self.pc + 2
            }
            0x0cfe4f => {
                w.immediate(0xff7e, 2);
                self.pc + 3
            }
            0x0cfe56 | 0x0cfe57 => {
                w.drain(6);
                w.push(
                    u16::from(if self.pc == 0x0cfe56 {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0x0cfe58 | 0x0cff03 | 0x0cff47 | 0x0cffaf => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                self.pc + 1
            }
            0x0cfe59 | 0x0cfea2 | 0x0cfebd | 0x0cfef6 | 0x0cff15 | 0x0cff65 | 0x0cff90
            | 0x0cff94 => {
                let address = match self.pc {
                    0x0cfea2 | 0x0cfebd | 0x0cfef6 => 0xc8,
                    0x0cff65 | 0x0cff94 => 0xcd,
                    _ => 0xca,
                };
                let value = w.dp_read(address, width);
                w.load(value, width);
                self.pc + 2
            }
            0x0cfe5b | 0x0cfe66 | 0x0cfe6a | 0x0cfeab | 0x0cfec2 | 0x0cfec9 | 0x0cff9d
            | 0x0cffa1 | 0x0cffa5 => {
                let target = match self.pc {
                    0x0cfe5b => 0x0cfe5f,
                    0x0cfe66 => 0x0cfe80,
                    0x0cfe6a => 0x0cfe7d,
                    0x0cfeab => 0x0cfeb0,
                    0x0cfec2 => 0x0cfee2,
                    0x0cfec9 => 0x0cfed9,
                    _ => 0x0cffab,
                };
                if w.branch(w.cpu.p & ZERO != 0) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x0cfe77 | 0x0cfece | 0x0cfef2 | 0x0cfefd | 0x0cff17 | 0x0cff92 | 0x0cffa9 => {
                let target = match self.pc {
                    0x0cfe77 => 0x0cfe7b,
                    0x0cfece => 0x0cfedc,
                    0x0cfef2 | 0x0cfefd => 0x0cff03,
                    0x0cff17 => 0x0cff2f,
                    _ => 0x0cffaf,
                };
                if w.branch(w.cpu.p & ZERO == 0) {
                    target
                } else {
                    self.pc + 2
                }
            }
            0x0cfe5d => {
                w.dp_increment(0xca, 1, false);
                self.pc + 2
            }
            0x0cfe5f => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                0x0ed60b
            }
            0x0ed617 => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x0cfe63 | 0x0cfeef => {
                w.drain(16);
                let value = w.read(w.absolute(0xff9), 1);
                w.load(value, 1);
                self.pc + 3
            }
            0x0cfe68 => {
                w.mask(3, 1);
                self.pc + 2
            }
            0x0cfe6c | 0x0cfee4 | 0x0cff13 | 0x0cff51 => {
                w.cpu.x = w.dp_read(
                    match self.pc {
                        0x0cfe6c => 0xd0,
                        0x0cfee4 => 0xcc,
                        _ => 0xcb,
                    },
                    1,
                );
                w.flags(w.cpu.x, 1);
                self.pc + 2
            }
            0x0cfe70 => {
                w.drain(14);
                let value = w.read(u32::from(0x9c + w.cpu.x), 1);
                w.load((w.cpu.a | value) & 255, 1);
                self.pc + 2
            }
            0x0cfe72 => {
                w.drain(14);
                w.write(u32::from(0x9c + w.cpu.x), w.cpu.a & 255, 1);
                self.pc + 2
            }
            0x0cfe74 | 0x0cff19 => {
                w.drain(6);
                w.cpu.x = if self.pc == 0x0cfe74 {
                    w.cpu.x.wrapping_add(1)
                } else {
                    w.cpu.x.wrapping_sub(1)
                } & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x0cfe75 | 0x0cff53 => {
                w.compare_x_immediate(if self.pc == 0x0cfe75 { 3 } else { 7 }, 1);
                self.pc + 2
            }
            0x0cfe79 | 0x0cfed2 => {
                w.drain(8);
                w.cpu.x = if self.pc == 0x0cfe79 { 0 } else { 0x20 };
                w.flags(w.cpu.x, 1);
                self.pc + 2
            }
            0x0cfe7b | 0x0cff1a => {
                w.dp_store(if self.pc == 0x0cfe7b { 0xd0 } else { 0xcb }, w.cpu.x, 1);
                self.pc + 2
            }
            0x0cfe7d => {
                w.drain(16);
                let a = w.absolute(0xff9);
                let value = w.read(a, 1).wrapping_sub(1) & 255;
                w.drain(6);
                w.flags(value, 1);
                w.write(a, value, 1);
                self.pc + 3
            }
            0x0cfe80 | 0x0cfec4 | 0x0cfed7 => {
                w.drain(8);
                w.cpu.y = match self.pc {
                    0x0cfe80 => 9,
                    0x0cfec4 => 1,
                    _ => 0x2c,
                };
                w.flags(w.cpu.y, 1);
                self.pc + 2
            }
            0x0cfe82 | 0x0cfe9d => {
                w.drain(6);
                w.load(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cfe83 | 0x0cfe84 | 0x0cfe9e => {
                w.shift_left(1);
                self.pc + 1
            }
            0x0cfe85 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                self.pc + 1
            }
            0x0cfe9f => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cfe88 | 0x0cfe8e | 0x0cfe93 | 0x0cfe99 | 0x0cfeb4 => {
                let (base, index) = match self.pc {
                    0x0cfe88 => (0xa72, w.cpu.y),
                    0x0cfe8e => (0x94a, w.cpu.x),
                    0x0cfe93 => (0x94b, w.cpu.x),
                    0x0cfe99 => (0x948, w.cpu.x),
                    _ => (0x949, w.cpu.x),
                };
                w.drain(22);
                w.write(w.absolute(base + index), w.cpu.a & 255, 1);
                self.pc + 3
            }
            0x0cfe8b | 0x0cfe96 | 0x0cff2a | 0x0cff41 | 0x0cff84 | 0x0cff8a => {
                let (base, index) = match self.pc {
                    0x0cfe8b => (0xfe1c, w.cpu.y),
                    0x0cfe96 => (0xfe26, w.cpu.y),
                    0x0cff2a => (0xff05, w.cpu.x),
                    0x0cff41 => (0xff0c, w.cpu.x),
                    0x0cff84 => (0xff49, w.cpu.x),
                    _ => (0xff4a, w.cpu.x),
                };
                w.drain(16);
                if (base & 255) + index > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(base + index), 1);
                w.load(value, 1);
                self.pc + 3
            }
            0x0cfe9c => {
                w.drain(6);
                w.push(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cfeb7 => {
                w.drain(12);
                w.cpu.y = w.pull(1);
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cfea4 | 0x0cfedc | 0x0cff6d | 0x0cff70 | 0x0cff76 | 0x0cff96 => {
                w.clear_carry();
                self.pc + 1
            }
            0x0cfea5 => {
                w.drain(16);
                if 0x30 + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(w.absolute(0xfe30 + w.cpu.y), 2);
                w.adc(value);
                self.pc + 3
            }
            0x0cfeaa | 0x0cfeaf | 0x0cfeb0 => {
                w.swap_bytes();
                self.pc + 1
            }
            0x0cfeb1 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                self.pc + 1
            }
            0x0cfeb2 => {
                w.drain(8);
                w.sbc(8, 1);
                self.pc + 2
            }
            0x0cfeb8 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                self.pc + 1
            }
            0x0cfeb9 | 0x0cff1c => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    if self.pc == 0x0cfeb9 {
                        0x0cfe82
                    } else {
                        0x0cff2a
                    }
                } else {
                    self.pc + 2
                }
            }
            0x0cfebf | 0x0cfec6 | 0x0cfecb | 0x0cfef8 => {
                w.compare_immediate(
                    match self.pc {
                        0x0cfec6 => 0xffbe,
                        0x0cfecb => 0xe,
                        _ => 0x1e,
                    },
                    2,
                );
                self.pc + 3
            }
            0x0cfed4 | 0x0cfed9 => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0x0cfed4 { 0xff9 } else { 0x12e }),
                    if self.pc == 0x0cfed4 {
                        w.cpu.x
                    } else {
                        w.cpu.y
                    },
                    1,
                );
                self.pc + 3
            }
            0x0cfedd => {
                w.drain(16);
                w.adc(0x10);
                self.pc + 3
            }
            0x0cfee6 => {
                w.drain(22);
                let target = w.read(0x0cfee9 + u32::from(w.cpu.x), 2);
                assert!(matches!(target, 0xfeef | 0xff13 | 0xff51));
                0x0c0000 | u32::from(target)
            }
            0x0cfeff | 0x0cff01 | 0x0cff24 | 0x0cff26 | 0x0cffab | 0x0cffad => {
                w.dp_increment(if self.pc >= 0x0cffab { 0xcb } else { 0xcc }, 1, true);
                self.pc + 2
            }
            0x0cff04 | 0x0cff48 | 0x0cffb0 => {
                w.rts();
                self.complete = true;
                0x0c0000 | u32::from(w.cpu.pc)
            }
            0x0cff28 => {
                w.branch(true);
                0x0cff47
            }
            0x0cff2f | 0x0cff57 | 0x0cff5a | 0x0cff34 | 0x0cff39 | 0x0cff3e | 0x0cff44
            | 0x0cff5f | 0x0cff62 | 0x0cff73 | 0x0cff79 | 0x0cff7e | 0x0cff81 | 0x0cff87
            | 0x0cff8d => {
                let a = match self.pc {
                    0x0cff2f | 0x0cff57 => 0xa70,
                    0x0cff5a => 0xa71,
                    0x0cff34 | 0x0cff5f => 0x940,
                    0x0cff39 | 0x0cff73 => 0x941,
                    0x0cff3e | 0x0cff7e => 0x943,
                    0x0cff44 | 0x0cff87 => 0x942,
                    0x0cff62 => 0x944,
                    0x0cff79 => 0x945,
                    0x0cff81 => 0x947,
                    _ => 0x946,
                };
                w.drain(16);
                w.write(w.absolute(a), if a >= 0xa70 { 0 } else { w.cpu.a & 255 }, 1);
                self.pc + 3
            }
            0x0cff55 => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    0x0cffaf
                } else {
                    self.pc + 2
                }
            }
            0x0cff67 | 0x0cff9b | 0x0cff9f | 0x0cffa3 | 0x0cffa7 => {
                w.compare_immediate(
                    match self.pc {
                        0x0cff67 => 0x50,
                        0x0cff9b => 4,
                        0x0cff9f => 0x48,
                        0x0cffa3 => 0x4c,
                        _ => 0x58,
                    },
                    1,
                );
                self.pc + 2
            }
            0x0cff69 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0x0cff6d
                } else {
                    self.pc + 2
                }
            }
            0x0cff6e => {
                let value = w.dp_read(0xc8, 1);
                w.adc_byte(value);
                self.pc + 2
            }
            0x0cff71 | 0x0cff77 | 0x0cff97 => {
                w.drain(8);
                w.adc_byte(match self.pc {
                    0x0cff71 => 0x31,
                    0x0cff77 => 8,
                    _ => 4,
                });
                self.pc + 2
            }
            _ => unreachable!("unowned sword continuation {:06x}", self.pc),
        };
        self.pc = next;
        w.cpu.bank = (next >> 16) as u8;
        w.cpu.pc = next as u16;
        Some(w.cycles)
    }
}

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    let (base, bytes): (u32, &[u8]) = match address {
        0x0cfe1c..=0x0cfe25 => (0x0cfe1c, &[0, 2, 32, 34, 4, 6, 8, 10, 12, 14]),
        0x0cfe26..=0x0cfe2f => (0x0cfe26, &[64, 64, 48, 80, 64, 64, 64, 64, 64, 64]),
        0x0cfe30..=0x0cfe43 => (
            0x0cfe30,
            &[
                16, 0, 32, 0, 40, 0, 40, 0, 48, 0, 64, 0, 80, 0, 96, 0, 112, 0, 128, 0,
            ],
        ),
        0x0cfee9..=0x0cfeee => (0x0cfee9, &[0xef, 0xfe, 0x13, 0xff, 0x51, 0xff]),
        0x0cff05..=0x0cff0b => (0x0cff05, &[4, 4, 6, 6, 6, 4, 4]),
        0x0cff0c..=0x0cff12 => (0x0cff0c, &[40, 55, 39, 54, 39, 55, 40]),
        0x0cff49..=0x0cff50 => (0x0cff49, &[38, 32, 36, 52, 37, 32, 53, 32]),
        _ => return None,
    };
    Some(bytes[(address - base) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn sword_setup_and_all_sparkle_phases_match_source_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0cfe00..0x0d0000 {
            if let Some(value) = table_byte(a) {
                assert_eq!(value, rom[0x60000 + (a as usize & 0x7fff)], "table {a:06x}");
            }
        }
        let mut instructions = 0;
        for phase in 0..4 {
            for seed in 0..32u8 {
                let mut work = if phase == 3 {
                    IntroSword::setup()
                } else {
                    IntroSword::periodic()
                };
                let mut cpu = NativeNmiContext {
                    a: 0xab00 | u16::from(seed),
                    x: 0x45,
                    y: 0x93,
                    d: 0,
                    db: seed,
                    bank: 12,
                    pc: work.pc as u16,
                    sp: 0x1fa,
                    p: 0x30 | (seed & 0xc3),
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
                }
                let ypos: u16 = [
                    0xff7e, 0xffbe, 0x000e, 0x001e, 0xfff0, 0x00f0, 0x7ff0, 0xfffe,
                ][usize::from(seed) % 8];
                machine.ram[0xc8..0xca].copy_from_slice(&ypos.to_le_bytes());
                machine.ram[0xca] = [0, 1, 2, 255][usize::from(seed) % 4];
                machine.ram[0xcb] = if phase == 2 { seed % 8 } else { seed % 7 };
                machine.ram[0xcc] = phase * 2;
                machine.ram[0xcd] =
                    [0, 0x44, 0x48, 0x54, 0x50, 0xff, 0x4f, 0x10][usize::from(seed) % 8];
                machine.ram[0xd0] = seed % 3;
                machine.ram[0xff9] = [0, 1, 2, 4, 0x20][usize::from(seed) % 5];
                machine.ram[0x1fb..0x1fd].copy_from_slice(&[0x77, 0xc2]);
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.dp = cpu.d;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let (v, h) = match seed % 3 {
                    0 => (100, 510),
                    1 => (224, 1340),
                    _ => (261, 1320),
                };
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut count = 0;
                while !work.complete() {
                    let label = format!("phase={phase} seed={seed} pc={:06x}", work.pc);
                    super::super::native_work::compare_instruction(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    work = work.clone();
                    count += 1;
                    instructions += 1;
                    assert!(count < 1000, "{label}");
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                    (12, 0xc278, 0x1fc, seed)
                );
            }
        }
        eprintln!("128 sword cases, {instructions} independently checked instruction boundaries");
    }
}
