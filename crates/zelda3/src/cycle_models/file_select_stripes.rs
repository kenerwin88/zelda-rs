//! File-menu stripe construction retains every live buffer write and leaves
//! upload requests for the actual NMI owner.
use super::{
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileSelectStripes {
    entry: u16,
    pc: u16,
    complete: bool,
}
impl FileSelectStripes {
    pub(crate) fn new(pc: u16) -> Self {
        assert!(matches!(pc, 0xce1b | 0xce53 | 0xcea5 | 0xceb1));
        Self {
            entry: pc,
            pc,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        0x0c0000
            | u32::from(if (0xce1b..=0xce52).contains(&self.pc) {
                0xce1b
            } else if (0xcec7..=0xcedb).contains(&self.pc) {
                0xcec7
            } else {
                self.entry
            })
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xce53 | 0xce54 | 0xcec7 | 0xcec8 => {
                w.drain(6);
                w.push(
                    u16::from(if matches!(self.pc, 0xce53 | 0xcec7) {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0xce55 | 0xcec9 | 0xceda | 0xd0a0 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xce56 | 0xce97 | 0xce9e | 0xceca | 0xced8 => {
                w.width(
                    if self.pc == 0xce56 {
                        0x30
                    } else if self.pc == 0xce97 {
                        0x20
                    } else {
                        0x10
                    },
                    matches!(self.pc, 0xce56 | 0xceca),
                );
                self.pc + 2
            }
            0xce58 | 0xceb1 => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                if self.pc == 0xce58 {
                    0xce1b
                } else {
                    0xcec7
                }
            }
            0xce5b => {
                w.drain(16);
                w.cpu.y = 0xde;
                w.flags(w.cpu.y, 2);
                0xce5e
            }
            0xce5e | 0xce37 => {
                w.drain(22);
                let v = w.read(
                    w.absolute(if self.pc == 0xce5e { 0xe1c8 } else { 0xce17 })
                        + u32::from(w.cpu.y),
                    2,
                );
                w.load(v, 2);
                self.pc + 3
            }
            0xcecf => {
                w.drain(22);
                let v = w.read(w.absolute(0xe358) + u32::from(w.cpu.x), 1);
                w.load(v, 1);
                0xced2
            }
            0xce61 => {
                w.drain(22);
                w.write(w.absolute(0x1806) + u32::from(w.cpu.y), w.cpu.a, 2);
                0xce64
            }
            0xced2 => {
                w.drain(22);
                w.write(w.absolute(0x1001) + u32::from(w.cpu.x), w.cpu.a & 255, 1);
                0xced5
            }
            0xce44 | 0xce77 | 0xce86 | 0xce8e | 0xce9b => {
                let width = if self.pc == 0xce9b { 1 } else { 2 };
                w.drain(22);
                w.write(
                    w.absolute(0x1006) + u32::from(w.cpu.x),
                    if width == 1 { w.cpu.a & 255 } else { w.cpu.a },
                    width,
                );
                self.pc + 3
            }
            0xce47 | 0xce48 | 0xce64 | 0xce65 | 0xce81 | 0xce82 | 0xce89 | 0xce8a | 0xce91
            | 0xce92 => {
                w.increment_x();
                self.pc + 1
            }
            0xce66 | 0xce67 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1);
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xced5 => {
                w.decrement_x();
                0xced6
            }
            0xce68 | 0xce95 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    if self.pc == 0xce68 {
                        0xce5e
                    } else {
                        0xce74
                    }
                } else {
                    self.pc + 2
                }
            }
            0xce50 | 0xced6 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    if self.pc == 0xce50 {
                        0xce2c
                    } else {
                        0xcecf
                    }
                } else {
                    self.pc + 2
                }
            }
            0xce1b | 0xce21 | 0xce6a | 0xce6f | 0xce83 | 0xce8b => {
                w.immediate(
                    match self.pc {
                        0xce1b => 0x10,
                        0xce21 => 0xff07,
                        0xce6a => 0x1103,
                        0xce6f => 0x11,
                        0xce83 => 0x3240,
                        _ => 0x347f,
                    },
                    2,
                );
                self.pc + 3
            }
            0xce99 | 0xceac | 0xceb4 | 0xd09c => {
                w.immediate(
                    match self.pc {
                        0xce99 => 255,
                        0xceac => 6,
                        0xceb4 => 15,
                        _ => 1,
                    },
                    1,
                );
                self.pc + 2
            }
            0xce1e | 0xce24 => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0xce1e { 0x1002 } else { 0x1004 }),
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xce27 => {
                w.dp_store(0, 0, 2);
                0xce29
            }
            0xce29 | 0xcecc => {
                w.drain(16);
                w.cpu.x = if self.pc == 0xce29 { 0 } else { 0xfd };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xce2c | 0xce4b | 0xce74 => {
                let v = w.dp_read(0, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xce2e => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                0xce2f
            }
            0xce2f | 0xce3d => {
                w.mask(if self.pc == 0xce2f { 0x20 } else { 1 }, 2);
                self.pc + 3
            }
            0xce32..=0xce35 => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | (w.cpu.a & 1) as u8;
                w.load(w.cpu.a >> 1, 2);
                self.pc + 1
            }
            0xce36 | 0xce41 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xce3a | 0xce6d | 0xce72 | 0xce7f => {
                w.dp_store(
                    if matches!(self.pc, 0xce3a | 0xce72) {
                        2
                    } else {
                        0
                    },
                    w.cpu.a,
                    2,
                );
                self.pc + 2
            }
            0xce3c => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                0xce3d
            }
            0xce40 => {
                w.shift_left(2);
                0xce41
            }
            0xce42 => {
                w.drain(8);
                let base = w.read(2, 2);
                w.drain(6);
                let v = w.read(w.absolute(base) + u32::from(w.cpu.y), 2);
                w.load(v, 2);
                0xce44
            }
            0xce49 => {
                w.dp_increment(0, 2, true);
                0xce4b
            }
            0xce4d => {
                w.compare_immediate(0x400, 2);
                0xce50
            }
            0xce52 | 0xcedb => {
                w.rts();
                if self.entry==0xce1b {self.complete=true;}
                w.cpu.pc
            }
            0xce76 | 0xce7a => {
                w.swap_bytes();
                self.pc + 1
            }
            0xce7b => {
                w.clear_carry();
                0xce7c
            }
            0xce7c => {
                w.drain(16);
                w.adc(0x20);
                0xce7f
            }
            0xce93 => {
                w.dp_increment(2, 2, false);
                0xce95
            }
            0xcea0 | 0xceaa => {
                w.dp_increment(0x11, 1, true);
                self.pc + 2
            }
            0xcea2 => {
                w.drain(16);
                0xd09c
            }
            0xcea5 => {
                w.drain(16);
                let v = w.read(w.absolute(0xb9d), 1);
                w.load(v, 1);
                0xcea8
            }
            0xcea8 | 0xceae | 0xceb6 | 0xd09e => {
                w.dp_store(
                    match self.pc {
                        0xcea8 => 0xc8,
                        0xceb6 => 0x13,
                        _ => 0x14,
                    },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xceb8 => {
                w.drain(16);
                w.write(w.absolute(0x710), 0, 1);
                0xcebb
            }
            0xcebb => {
                w.branch(true);
                0xceaa
            }
            0xceb0 | 0xd0a1 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("file stripe continuation {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn file_stripe_callers_match_every_instruction_and_buffer_store() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for entry in [0xce53, 0xcea5, 0xceb1] {
            for (v, h) in [(224, 1338), (261, 1350)] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8)) as u8;
                }
                machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
                let mut native = NativeNmiContext {
                    a: 0xab83,
                    x: 0x71,
                    y: 0x83,
                    d: 0,
                    db: 0,
                    bank: 12,
                    pc: entry,
                    sp: 0x1fa,
                    p: 0x31,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.k = native.bank;
                machine.cpu.pc = native.pc;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                let mut ram = machine.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(v == 261),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = FileSelectStripes::new(entry);
                let mut count = 0;
                while !work.complete() {
                    let label = format!("file stripes entry{entry:04x} pc{:04x}", native.pc);
                    super::super::native_work::compare_instruction(
                        &label,
                        &mut native,
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
                    assert!(count < 100000);
                }
                assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
            }
        }
    }
}
