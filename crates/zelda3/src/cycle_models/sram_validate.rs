//! Live save validation, backup repair, and descending sprite-table clear.
//! Checksums and every partially completed store remain CPU-owned.
use super::{
    intro_clear::Access,
    native_work::Work,
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
pub(crate) fn table_byte(a: u32) -> Option<u8> {
    a.checked_sub(0x00848c)
        .and_then(|i| [0, 0, 0, 5, 0, 10].get(i as usize).copied())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SramValidate {
    pc: u16,
    complete: bool,
}
impl Default for SramValidate {
    fn default() -> Self {
        Self {
            pc: 0x8054,
            complete: false,
        }
    }
}
impl SramValidate {
    pub(crate) fn complete(&self) -> bool {
        self.complete
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (2, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x8054 => {
                w.width(0x30, true);
                0x8056
            }
            0x8056 => {
                w.dp_store(0, 0, 2);
                0x8058
            }
            0x8058 | 0x80c6 => {
                w.cpu.x = w.dp_read(0, 2);
                w.flags(w.cpu.x, 2);
                self.pc + 2
            }
            0x805a => {
                w.drain(24);
                let v = w.read(0x00848c + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                0x805e
            }
            0x805e => {
                w.tax_word();
                0x805f
            }
            0x805f | 0x8077 => {
                w.drain(6);
                w.push(w.cpu.x, 2);
                self.pc + 1
            }
            0x8060 | 0x8078 | 0x808f | 0x80e0 => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                self.pc + 3
            }
            0x8063 | 0x807b | 0x80e3 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0x8064 | 0x807c => {
                w.clear_carry();
                self.pc + 1
            }
            0x8065 | 0x807d => {
                w.drain(24);
                let v = w.read(
                    (if self.pc == 0x8065 {
                        0x700000
                    } else {
                        0x700f00
                    }) + u32::from(w.cpu.x),
                    2,
                );
                w.adc(v);
                self.pc + 4
            }
            0x8069 | 0x806a | 0x8081 | 0x8082 | 0x80ba | 0x80bb | 0x810c | 0x810d => {
                w.increment_x();
                self.pc + 1
            }
            0x806b | 0x8083 | 0x80bc | 0x810e => {
                w.increment_y();
                self.pc + 1
            }
            0x806c | 0x8084 | 0x80bd | 0x810f => {
                w.drain(16);
                let operand = if self.pc < 0x808f { 0x280 } else { 0x80 };
                w.cpu.p = (w.cpu.p & !1) | u8::from(w.cpu.y >= operand);
                w.flags(w.cpu.y.wrapping_sub(operand), 2);
                self.pc + 3
            }
            0x806f | 0x8087 | 0x80c0 | 0x80cb | 0x8112 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x806f => 0x8064,
                        0x8087 => 0x807c,
                        0x80c0 => 0x8092,
                        0x80cb => 0x8058,
                        _ => 0x80e4,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x8071 | 0x8089 => {
                w.drain(12);
                w.cpu.x = w.pull(2);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x8072 | 0x808a => {
                w.compare_immediate(0x5a5a, 2);
                self.pc + 3
            }
            0x8075 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    0x80c2
                } else {
                    0x8077
                }
            }
            0x808d => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x80e0
                } else {
                    0x808f
                }
            }
            0x8092 | 0x809a | 0x80a2 | 0x80aa | 0x80b2 => {
                w.drain(24);
                let v = w.read(
                    0x700f00 + u32::from((self.pc - 0x8092) / 8) * 0x100 + u32::from(w.cpu.x),
                    2,
                );
                w.load(v, 2);
                self.pc + 4
            }
            0x8096 | 0x809e | 0x80a6 | 0x80ae | 0x80b6 => {
                w.drain(24);
                w.write(
                    0x700000 + u32::from((self.pc - 0x8096) / 8) * 0x100 + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0x80e4 | 0x80e8 | 0x80ec | 0x80f0 | 0x80f4 | 0x80f8 | 0x80fc | 0x8100 | 0x8104
            | 0x8108 => {
                let i = u32::from((self.pc - 0x80e4) / 4);
                w.drain(24);
                w.write(
                    0x700000
                        + (i / 2) * 0x100
                        + (if i % 2 == 0 { 0xf00 } else { 0 })
                        + u32::from(w.cpu.x),
                    w.cpu.a,
                    2,
                );
                self.pc + 4
            }
            0x80c2 | 0x80c4 => {
                w.dp_increment(0, 2, true);
                self.pc + 2
            }
            0x80c8 => {
                w.compare_x_immediate(6, 2);
                0x80cb
            }
            0x80cd => {
                w.drain(16);
                w.cpu.x = 0xfe;
                w.flags(w.cpu.x, 2);
                0x80d0
            }
            0x80d0 | 0x80d3 | 0x80d6 => {
                w.drain(22);
                w.write(
                    w.absolute(0xd00 + ((self.pc - 0x80d0) / 3) * 0x100)
                        .wrapping_add(u32::from(w.cpu.x)),
                    0,
                    2,
                );
                self.pc + 3
            }
            0x80d9 | 0x80da => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_sub(1);
                w.flags(w.cpu.x, 2);
                self.pc + 1
            }
            0x80db => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0x80d0
                } else {
                    0x80dd
                }
            }
            0x80dd => {
                w.width(0x30, false);
                0x80df
            }
            0x80df => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0x8114 => {
                w.branch(true);
                0x80c2
            }
            _ => unreachable!("SRAM validation continuation {:04x}", self.pc),
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
    fn sram_validation_matches_primary_backup_and_clear_at_every_instruction() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for a in 0x848c..0x8492 {
            assert_eq!(table_byte(a), Some(rom[a as usize - 0x8000]));
        }
        for variant in 0..3 {
            for (v, h) in [(224, 1338), (261, 1350)] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8)) as u8;
                }
                for (i, b) in machine.cart.ram.iter_mut().enumerate() {
                    *b = (i * 19 + i / 256) as u8;
                }
                for slot in 0..3 {
                    // Rotate valid primary, valid backup, and both-invalid cases
                    // so all three slot offsets exercise each path.
                    for backup in [false, true] {
                        let start = slot * 0x500 + if backup { 0xf00 } else { 0 };
                        let valid = (slot + variant) % 3 == usize::from(backup);
                        let sum = machine.cart.ram[start..start + 0x4fe]
                            .chunks_exact(2)
                            .fold(0u16, |s, b| {
                                s.wrapping_add(u16::from_le_bytes([b[0], b[1]]))
                            });
                        let checksum =
                            (if valid { 0x5a5au16 } else { 0x1234u16 }).wrapping_sub(sum);
                        machine.cart.ram[start + 0x4fe..start + 0x500]
                            .copy_from_slice(&checksum.to_le_bytes());
                    }
                }
                machine.ram[0x1f8..0x1fb].copy_from_slice(&[0xed, 0xcd, 12]);
                let mut native = NativeNmiContext {
                    a: 0xf03c,
                    x: 0x72,
                    y: 0x83,
                    d: 0,
                    db: 0,
                    bank: 2,
                    pc: 0x8054,
                    sp: 0x1f7,
                    p: 0x31,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.k = 2;
                machine.cpu.pc = native.pc;
                machine.cpu.sp = native.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(native.p);
                let mut ram = machine.ram.clone();
                let mut sram = machine.cart.ram.clone();
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::non_interlace(variant & 1 != 0),
                );
                let mut source = RomCpuTimingProbe::new(
                    machine,
                    clock.clone(),
                    SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                clock.begin_synchronous_timeline().unwrap();
                let mut work = SramValidate::default();
                let mut count = 0;
                while !work.complete() {
                    let label = format!("SRAM variant={variant} pc={:04x}", work.pc);
                    super::super::native_work::compare_instruction_with_devices(
                        &label,
                        &mut native,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        None,
                        |_, access| match access {
                            Access::Read(a, 2) if (0x700000..0x702000).contains(&a) => {
                                let i = (a - 0x700000) as usize;
                                Some(u16::from_le_bytes([sram[i], sram[i + 1]]))
                            }
                            Access::Write(a, value, 2) if (0x700000..0x702000).contains(&a) => {
                                let i = (a - 0x700000) as usize;
                                sram[i..i + 2].copy_from_slice(&value.to_le_bytes());
                                Some(0)
                            }
                            _ => None,
                        },
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(sram, source.snes().cart.ram, "{label}");
                    assert_eq!(ram, source.snes().ram, "{label}");
                    count += 1;
                    assert!(count < 100000);
                }
                assert_eq!((native.bank, native.pc, native.sp), (12, 0xcdee, 0x1fa));
            }
        }
    }
}
