//! Live sine/cosine publication and Mode 7 multiplier work for the poly worker.
use super::{native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Call,
    ByteWidth,
    Angle(u8),
    Trig {
        term: u8,
        step: u8,
    },
    WordWidth,
    MaskIrq,
    Product {
        term: u8,
        step: u8,
    },
    UnmaskIrq,
    Return,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn rotation_matches_live_multiplier_and_every_instruction() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // Zero, all sign combinations, and both indexed table page crossings.
        for (a, b) in [
            (0, 0),
            (32, 96),
            (64, 192),
            (128, 255),
            (200, 40),
            (255, 255),
        ] {
            for (status, v, h) in [(0x00, 100, 510), (0xc5, 224, 1300)] {
                let mut cpu = NativeNmiContext {
                    a: 0xffff,
                    x: 0xff98,
                    y: 0x1f45,
                    d: 0x1f00,
                    db: 9,
                    bank: 9,
                    pc: 0xf82c,
                    sp: 0x1f3e,
                    p: status,
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8) ^ 0x5a) as u8;
                }
                machine.ram[0x1f04] = a;
                machine.ram[0x1f05] = b;
                machine.ppu.m7_matrix = [0x1234, -765, 321, 0, 0, 0, 0, 0];
                machine.ppu.m7_prev = 0xa7;
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
                let mut ppu = machine.ppu.clone();
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
                let mut rotation = PolyRotation::default();
                while !rotation.complete() {
                    assert_eq!(rotation.defers_irq_mask_change(),
                        matches!(rom[9 * 0x8000 + (cpu.pc as usize & 0x7fff)], 0x58 | 0x78));
                    let label = format!("angles={a},{b} p={status:02x} {:?}", rotation.phase);
                    crate::cycle_models::native_work::compare_instruction_with_ppu(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        Some(&mut ppu),
                        |cpu, clock, bus| {
                            rotation.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    assert_eq!(ppu.m7_matrix, source.snes().ppu.m7_matrix, "{label}");
                    assert_eq!(ppu.m7_prev, source.snes().ppu.m7_prev, "{label}");
                    rotation = rotation.clone();
                }
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.d),
                    (9, 0xf82f, 0x1f3e, 9, 0x1f00)
                );
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PolyRotation {
    phase: Phase,
}

impl PolyRotation {
    pub(super) fn defers_irq_mask_change(&self) -> bool {
        matches!(self.phase, Phase::MaskIrq | Phase::UnmaskIrq) // SEI / CLI
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }

    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Call => 0xf82c,
            Phase::ByteWidth => 0xf864,
            Phase::Angle(angle) => [0xf866, 0xf882][usize::from(angle)],
            Phase::Trig { term, step } => {
                [0xf868, 0xf875, 0xf884, 0xf891][usize::from(term)]
                    + [0, 3, 5, 7, 9, 11][usize::from(step)]
            }
            Phase::WordWidth => 0xf89e,
            Phase::MaskIrq => 0xf8a0,
            Phase::Product { term, step } => {
                0xf8a1
                    + u16::from(term) * 22
                    + [0, 2, 5, 7, 10, 12, 15, 18, 19, 20][usize::from(step)]
            }
            Phase::UnmaskIrq => 0xf8f9,
            Phase::Return => 0xf8fa,
            Phase::Complete => 0xf82f,
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (9, self.pc(), 0x1f00, 9));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Phase::Call => {
                w.drain(16);
                w.drain(6);
                w.push(w.cpu.pc + 2, 2);
                Phase::ByteWidth
            }
            Phase::ByteWidth => {
                w.width(0x30, false);
                Phase::Angle(0)
            }
            Phase::Angle(angle) => {
                let value = w.dp_read(w.cpu.d + 4 + u16::from(angle), 1);
                w.cpu.y = value;
                w.flags(value, 1);
                Phase::Trig {
                    term: angle * 2,
                    step: 0,
                }
            }
            Phase::Trig { term, step } => {
                let target = w.cpu.d + 0x50 + u16::from(term) * 2;
                match step {
                    0 => {
                        let base = if term % 2 == 0 { 0xfb6d_u16 } else { 0xfbad };
                        let address = base.wrapping_add(w.cpu.y);
                        w.drain(16);
                        if base & 0xff00 != address & 0xff00 {
                            w.drain(6);
                        }
                        let value = w.read(w.absolute(address), 1);
                        w.load(value, 1);
                    }
                    1 => w.dp_store(target, w.cpu.a & 255, 1),
                    2 => w.compare_immediate(0x80, 1),
                    3 => {
                        let rhs = w.dp_read(target, 1);
                        w.sbc(rhs, 1);
                    }
                    4 => {
                        w.drain(8);
                        w.load((w.cpu.a ^ 255) & 255, 1);
                    }
                    5 => w.dp_store(target + 1, w.cpu.a & 255, 1),
                    _ => unreachable!(),
                }
                if step < 5 {
                    Phase::Trig {
                        term,
                        step: step + 1,
                    }
                } else {
                    match term {
                        0 | 2 => Phase::Trig {
                            term: term + 1,
                            step: 0,
                        },
                        1 => Phase::Angle(1),
                        3 => Phase::WordWidth,
                        _ => unreachable!(),
                    }
                }
            }
            Phase::WordWidth => {
                w.width(0x20, true);
                Phase::MaskIrq
            }
            Phase::MaskIrq => {
                w.drain(6);
                w.cpu.p |= 4;
                Phase::Product { term: 0, step: 0 }
            }
            Phase::Product { term, step } => {
                let (lhs, rhs, target) = [
                    (0x54, 0x50, 0x58),
                    (0x56, 0x52, 0x5e),
                    (0x56, 0x50, 0x5a),
                    (0x54, 0x52, 0x5c),
                ][usize::from(term)];
                match step {
                    0 | 2 | 4 => {
                        let at = w.cpu.d
                            + if step == 4 {
                                rhs
                            } else {
                                lhs + u16::from(step == 2)
                            };
                        let value = w.dp_read(at, 1);
                        w.cpu.x = value;
                        w.flags(value, 1);
                    }
                    1 | 3 | 5 => {
                        w.drain(16);
                        w.write_io(if step == 5 { 0x211c } else { 0x211b }, w.cpu.x, 1);
                    }
                    6 => {
                        w.drain(16);
                        let value = w.read_io(0x2135, 2);
                        w.load(value, 2);
                    }
                    7 | 8 => w.shift_left(2),
                    9 => w.dp_store(w.cpu.d + target, w.cpu.a, 2),
                    _ => unreachable!(),
                }
                if step < 9 {
                    Phase::Product {
                        term,
                        step: step + 1,
                    }
                } else if term < 3 {
                    Phase::Product {
                        term: term + 1,
                        step: 0,
                    }
                } else {
                    Phase::UnmaskIrq
                }
            }
            Phase::UnmaskIrq => {
                w.drain(6);
                w.cpu.p &= !4;
                Phase::Return
            }
            Phase::Return => {
                w.rts();
                assert_eq!(w.cpu.pc, 0xf82f);
                self.phase = Phase::Complete;
                return Some(w.cycles);
            }
            Phase::Complete => unreachable!(),
        };
        w.cpu.pc = self.pc();
        Some(w.cycles)
    }
}
