//! Shared six-region OAM allocator with retained overflow and pointer work.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OamAllocate {
    pc: u16,
    complete: bool,
}
impl OamAllocate {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(
            entry,
            0xba80 | 0xba84 | 0xba88 | 0xba8c | 0xba90 | 0xba94
        ));
        Self {
            pc: entry,
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
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x18),
            (13, self.pc, 0, 0x10)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        self.pc = match pc {
            0xba80 | 0xba84 | 0xba88 | 0xba8c | 0xba90 | 0xba94 => {
                w.drain(8);
                w.cpu.y = (pc - 0xba80) / 2;
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            0xba82 | 0xba86 | 0xba8a | 0xba8e | 0xba92 => {
                w.branch(true);
                0xba96
            }
            0xba96 | 0xba97 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xba96 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xba98 | 0xba9c => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xba99 => {
                w.operand_then_internal(2);
                w.push(0xba9b, 2);
                0xbb0a
            }
            0xba9d => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xbb5a => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xba9c);
                w.cpu.pc
            }
            0xbb0a | 0xbb14 | 0xbb20 | 0xbb2f | 0xbb3c | 0xbb4d | 0xbb54 => {
                let a = match pc {
                    0xbb0a | 0xbb2f => 14,
                    0xbb14 | 0xbb3c | 0xbb54 => 0x90,
                    0xbb20 => 13,
                    _ => 0x92,
                };
                w.dp_store(
                    a,
                    if pc == 0xbb20 {
                        0
                    } else {
                        w.cpu.a & if m == 1 { 255 } else { 65535 }
                    },
                    m,
                );
                pc + 2
            }
            0xbb0c => {
                w.drain(16);
                w.write(w.absolute(15), 0, 1);
                pc + 3
            }
            0xbb0f | 0xbb56 => {
                w.width(0x20, pc == 0xbb0f);
                pc + 2
            }
            0xbb11 | 0xbb19 | 0xbb22 | 0xbb39 => {
                let base = match pc {
                    0xbb11 => 0xfe0,
                    0xbb19 => 0xba9e,
                    0xbb22 => 0xfec,
                    _ => 0xbaaa,
                };
                w.drain(16);
                if (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(base) + u32::from(w.cpu.y), 2);
                if pc == 0xbb19 {
                    w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a >= v);
                    w.flags(w.cpu.a.wrapping_sub(v), 2);
                } else {
                    w.load(v, 2);
                }
                pc + 3
            }
            0xbb16 | 0xbb49 | 0xbb50 => {
                w.clear_carry();
                pc + 1
            }
            0xbb17 | 0xbb36 => {
                let v = w.dp_read(14, 2);
                w.adc(v);
                pc + 2
            }
            0xbb1c => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0xbb41
                } else {
                    pc + 2
                }
            }
            0xbb1e => {
                w.dp_store(12, w.cpu.y, 1);
                pc + 2
            }
            0xbb25 | 0xbb46 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
                pc + 1
            }
            0xbb26 => {
                w.increment_a(2);
                pc + 1
            }
            0xbb27 | 0xbb41 => {
                w.operand_then_internal(2);
                w.write(
                    w.absolute(if pc == 0xbb27 { 0xfec } else { 0xfe0 }) + u32::from(w.cpu.y),
                    w.cpu.a,
                    2,
                );
                pc + 3
            }
            0xbb2a | 0xbb4f => {
                w.drain(12);
                let v = w.pull(2);
                w.load(v, 2);
                pc + 1
            }
            0xbb2b => {
                w.mask(7, 2);
                pc + 3
            }
            0xbb2e | 0xbb33 | 0xbb34 | 0xbb35 => {
                w.shift_left(2);
                pc + 1
            }
            0xbb31 | 0xbb44 => {
                let v = w.dp_read(if pc == 0xbb31 { 12 } else { 0x90 }, 2);
                w.load(v, 2);
                pc + 2
            }
            0xbb38 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0xbb3e => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0xbb3f => {
                w.branch(true);
                0xbb44
            }
            0xbb47 | 0xbb48 => {
                w.drain(6);
                let v = w.cpu.a;
                w.cpu.p = (w.cpu.p & !CARRY) | (v & 1) as u8;
                w.load(v >> 1, 2);
                pc + 1
            }
            0xbb4a | 0xbb51 => {
                w.drain(16);
                w.adc(if pc == 0xbb4a { 0xa20 } else { 0x800 });
                pc + 3
            }
            0xbb58 => {
                w.cpu.y = w.dp_read(0x90, 1);
                w.flags(w.cpu.y, 1);
                pc + 2
            }
            _ => unreachable!("OAM allocator {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oam_allocator_matches_source_all_regions_and_overflow_cursors() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in 0x0dba9e..0x0dbb0a {
            assert_eq!(
                crate::ZeldaState::native_oam_table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        let mut instructions = 0;
        for region in 0..6usize {
            for case in 0..64usize {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                machine.ram.fill(0x37);
                let limit = u16::from_le_bytes([
                    crate::ZeldaState::native_oam_table_byte(0x0dba9e + region as u32 * 2).unwrap(),
                    crate::ZeldaState::native_oam_table_byte(0x0dba9f + region as u32 * 2).unwrap(),
                ]);
                let base = [
                    0,
                    limit - 17,
                    limit - 16,
                    limit - 1,
                    limit,
                    limit + 1,
                    0xfff0,
                    0xffff,
                ][case / 8];
                machine.ram[0xfe0 + region * 2..0xfe2 + region * 2]
                    .copy_from_slice(&base.to_le_bytes());
                machine.ram[0xfec + region * 2..0xfee + region * 2].copy_from_slice(
                    &(if case & 1 == 0 {
                        case as u16
                    } else {
                        0xfff8 + case as u16 % 8
                    })
                    .to_le_bytes(),
                );
                machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 8]);
                let mut cpu = NativeNmiContext {
                    a: 0xab00 | [0, 1, 16, 255][case % 4],
                    x: 9,
                    y: 0,
                    d: 0,
                    db: 8,
                    bank: 13,
                    pc: 0xba80 + region as u16 * 4,
                    sp: 0x1fc,
                    p: 0x30 | ((case as u8) & 0xc3),
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc;
                machine.cpu.k = cpu.bank;
                machine.cpu.db = cpu.db;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(
                    [0, 1, 30002, 30003][case % 4],
                    snes::CpuRasterPosition::new(224, 1340),
                    snes::CpuBusWorkload::default(),
                    snes::CpuFieldTiming::NON_INTERLACE_EVEN,
                );
                let mut source = snes::RomCpuTimingProbe::new(
                    machine,
                    timeline,
                    snes::SourcePpuReadState::snes9x_reset(),
                )
                .unwrap();
                let mut clock = source.timeline().clone();
                let mut work = OamAllocate::new(cpu.pc);
                for _ in 0..100 {
                    if work.complete() {
                        break;
                    }
                    let label = format!("region={region} case={case} pc={:04x}", cpu.pc);
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
                    instructions += 1;
                }
                assert!(work.complete());
                assert_eq!(ram, source.snes().ram);
                assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (8, 0x805a, 0x1ff, 8));
            }
        }
        eprintln!("384 OAM allocation fixtures, {instructions} instruction boundaries");
    }
}
