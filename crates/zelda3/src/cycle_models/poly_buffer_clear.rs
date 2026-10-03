//! Poly worker's live bitmap clear, including its caller and interruptible
//! MVN. The next projection call remains a separate native continuation.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Call,
    Body(u8),
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PolyBufferClear {
    phase: Phase,
}
impl Default for PolyBufferClear {
    fn default() -> Self {
        Self { phase: Phase::Call }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    // MVN's source-bank operand is an observable bus read in the source.
    (address == 0x09fd1a).then_some(0x7e)
}
impl PolyBufferClear {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Call => 0xf825,
            Phase::Body(i) => [
                0xfd04, 0xfd05, 0xfd06, 0xfd08, 0xfd0b, 0xfd0f, 0xfd12, 0xfd15, 0xfd18, 0xfd1b,
                0xfd1c, 0xfd1d,
            ][usize::from(i)],
            Phase::Complete => 0xf829,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc), (9, self.pc()));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Phase::Call => {
                w.drain(24);
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                Phase::Body(0)
            }
            Phase::Body(i) => {
                match i {
                    0 | 1 => {
                        w.drain(6);
                        w.push(u16::from(if i == 0 { w.cpu.p } else { w.cpu.db }), 1);
                    }
                    2 => w.width(0x30, true),
                    3 | 7 => w.immediate(if i == 3 { 0 } else { 0x7fd }, 2),
                    4 => {
                        w.drain(24);
                        w.write(0x7ee800, w.cpu.a, 2);
                    }
                    5 | 6 => {
                        w.drain(16);
                        let value = if i == 5 { 0xe800 } else { 0xe802 };
                        if i == 5 {
                            w.cpu.x = value;
                        } else {
                            w.cpu.y = value;
                        }
                        w.flags(value, 2);
                    }
                    8 => {
                        w.drain(8);
                        let bank = w.read(0x09fd1a, 1);
                        assert_eq!(bank, 0x7e);
                        let value = w.read(0x7e0000 | u32::from(w.cpu.x), 1);
                        w.write(0x7e0000 | u32::from(w.cpu.y), value, 1);
                        w.drain(12);
                        w.cpu.db = 0x7e;
                        w.cpu.x = w.cpu.x.wrapping_add(1);
                        w.cpu.y = w.cpu.y.wrapping_add(1);
                        w.cpu.a = w.cpu.a.wrapping_sub(1);
                        if w.cpu.a != 0xffff {
                            return Some(w.cycles);
                        }
                    }
                    9 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    10 => {
                        w.drain(12);
                        w.cpu.p = w.pull(1) as u8;
                        if w.cpu.p & 0x10 != 0 {
                            w.cpu.x &= 255;
                            w.cpu.y &= 255;
                        }
                    }
                    11 => {
                        w.rtl();
                        assert_eq!(w.cpu.pc, 0xf829);
                        self.phase = Phase::Complete;
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Phase::Body(i + 1)
            }
            Phase::Complete => unreachable!(),
        };
        w.cpu.pc = self.pc();
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
    fn poly_bitmap_clear_matches_every_byte_and_retained_register() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        assert_eq!(table_byte(0x09fd1a), Some(rom[9 * 0x8000 + 0x7d1a]));
        for (status, v, h) in [(0x30, 235, 358), (0x31, 261, 1300), (0x34, 100, 510)] {
            let mut cpu = NativeNmiContext {
                a: 0xab01,
                x: 0x45,
                y: 0x67,
                d: 0x1f00,
                db: 9,
                bank: 9,
                pc: 0xf825,
                sp: 0x1f3e,
                p: status,
            };
            let mut machine = Snes::new();
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 8) ^ 0xa5) as u8;
            }
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
            let mut clock = CpuMasterTimeline::at_raster(
                0,
                CpuRasterPosition::new(v, h),
                CpuBusWorkload::default(),
                CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source =
                RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                    .unwrap();
            clock.begin_synchronous_timeline().unwrap();
            let mut clear = PolyBufferClear::default();
            while !clear.complete() {
                let label = format!("{status:02x} {v}:{h} {:?}", clear.phase);
                crate::cycle_models::native_work::compare_instruction(
                    &label,
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        clear.step(cpu, clock, bus).unwrap();
                    },
                );
                clear = clear.clone();
            }
            assert_eq!(ram, source.snes().ram);
            assert!(ram[0xe800..0xf000].iter().all(|&b| b == 0));
            assert_eq!(
                (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d, cpu.p),
                (9, 0xf829, 0x1f3e, 9, 0x1f00, status)
            );
        }
    }
}
