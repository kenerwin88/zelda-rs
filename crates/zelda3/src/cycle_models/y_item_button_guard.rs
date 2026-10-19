//! Shared Y-item activation guard. BIT and TSB keep their original flag and
//! read/modify/write behavior, and the result returns on the live caller stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YItemButtonGuard {
    pc: u16,
    complete: bool,
}
impl Default for YItemButtonGuard {
    fn default() -> Self {
        Self {
            pc: 0xb073,
            complete: false,
        }
    }
}
impl YItemButtonGuard {
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
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (7, self.pc, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xb073 => {
                let value = w.dp_read(0x3a, 1);
                w.cpu.p = (w.cpu.p & !0xc2)
                    | (value as u8 & 0xc0)
                    | if w.cpu.a & value & 255 == 0 { ZERO } else { 0 };
                0xb075
            }
            0xb075 | 0xb079 | 0xb07f => {
                let taken = match self.pc {
                    0xb075 => w.cpu.p & 0x40 != 0,
                    0xb079 => w.cpu.p & ZERO == 0,
                    _ => w.cpu.p & ZERO != 0,
                };
                if w.branch(taken) {
                    0xb085
                } else {
                    self.pc + 2
                }
            }
            0xb077 | 0xb07b => {
                let value = w.dp_read(if self.pc == 0xb077 { 0x46 } else { 0xf4 }, 1);
                w.load(value, 1);
                self.pc + 2
            }
            0xb07d => {
                w.mask(0x40, 1);
                0xb07f
            }
            0xb081 => {
                let value = w.dp_read(0x3a, 1);
                w.cpu.p = (w.cpu.p & !ZERO) | if value & w.cpu.a & 255 == 0 { ZERO } else { 0 };
                w.drain(6);
                w.write(0x3a, value | (w.cpu.a & 255), 1);
                0xb083
            }
            0xb083 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xb084
            }
            0xb085 => {
                w.clear_carry();
                0xb086
            }
            0xb084 | 0xb086 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("Y-item activation guard {:04x}", self.pc),
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
    fn y_item_activation_guard_matches_source_flags_and_bus() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut cases = 0;
        let mut instructions = 0;
        for accumulator in 0..=255u16 {
            for flags in [0, 0x40, 0x80, 0xc0] {
                for pressed in [0, 0x40, 0xff] {
                    for incapacitated in [0, 1] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        machine.ram[0x3a] = flags;
                        machine.ram[0xf4] = pressed;
                        machine.ram[0x46] = incapacitated;
                        machine.ram[0x1f1..0x1f3].copy_from_slice(&[0x53, 0xa2]);
                        let mut cpu = NativeNmiContext {
                            a: 0xaa00 | accumulator,
                            x: 0xf3,
                            y: 0xb8,
                            d: 0,
                            db: 7,
                            bank: 7,
                            pc: 0xb073,
                            sp: 0x1f0,
                            p: 0x30 | (accumulator as u8 & 0xc3),
                        };
                        machine.cpu.a = cpu.a;
                        machine.cpu.x = cpu.x;
                        machine.cpu.y = cpu.y;
                        machine.cpu.db = cpu.db;
                        machine.cpu.k = cpu.bank;
                        machine.cpu.pc = cpu.pc;
                        machine.cpu.sp = cpu.sp;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(cpu.p);
                        let mut ram = machine.ram.clone();
                        let mut clock = CpuMasterTimeline::at_raster(
                            4586,
                            CpuRasterPosition::new(
                                if accumulator & 1 == 0 { 21 } else { 261 },
                                1350,
                            ),
                            CpuBusWorkload::default(),
                            CpuFieldTiming::non_interlace(accumulator & 1 != 0),
                        );
                        let mut source = RomCpuTimingProbe::new(
                            machine,
                            clock.clone(),
                            SourcePpuReadState::snes9x_reset(),
                        )
                        .unwrap();
                        clock.begin_synchronous_timeline().unwrap();
                        let mut work = YItemButtonGuard::default();
                        while !work.complete() {
                            let label=format!("Y-button A={accumulator:02x} flags={flags:02x} pressed={pressed:02x} timer={incapacitated} pc={:04x}",cpu.pc);
                            super::super::native_work::compare_instruction_with_devices(
                                &label,
                                &mut cpu,
                                &mut clock,
                                &mut source,
                                &mut ram,
                                &rom,
                                None,
                                |_, _| None,
                                |cpu, clock, bus| {
                                    work.step(cpu, clock, bus).unwrap();
                                },
                            );
                            work = work.clone();
                            instructions += 1;
                        }
                        assert_eq!(ram, source.snes().ram);
                        assert_eq!((cpu.pc, cpu.sp), (0xa254, 0x1f2));
                        assert_eq!(
                            cpu.p & CARRY != 0,
                            flags & 0x40 == 0 && incapacitated == 0 && pressed & 0x40 != 0
                        );
                        cases += 1;
                    }
                }
            }
        }
        println!("Y-item activation guard: {cases} cases, {instructions} source instructions");
    }
}
