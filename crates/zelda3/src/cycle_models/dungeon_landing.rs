//! Landing-wipe caller. Child calls retain the real far return on the stack;
//! the enclosing dungeon owner must finish each child before resuming here.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
    spotlight::Spotlight,
};
use snes::CpuMasterTimeline;

pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0x029319..=0x02931c => Some([0x2d, 0x93, 0x34, 0x93][(address - 0x029319) as usize]),
        0x029323 => Some(0x93),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LandingChild {
    Sprites,
    PlayerAnimation,
    LinkOam,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonLanding {
    pc: u16,
    spotlight: Option<Spotlight>,
    child: Option<LandingChild>,
    complete: bool,
}
impl Default for DungeonLanding {
    fn default() -> Self {
        Self {
            pc: 0x931d,
            spotlight: None,
            child: None,
            complete: false,
        }
    }
}
impl DungeonLanding {
    pub(crate) fn child(&self) -> Option<LandingChild> {
        self.child
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }

    /// The child, including its RTL, must run on the inherited CPU and clock.
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        assert!(self.child.take().is_some());
        assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
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
        assert!(
            self.child.is_none(),
            "landing child must retain CPU ownership through its return"
        );
        if let Some(spotlight) = self.spotlight.as_mut() {
            let cycles = spotlight.step(cpu, clock, bus);
            if spotlight.complete() {
                self.spotlight = None;
                assert_eq!((cpu.bank, cpu.pc), (2, self.pc));
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x38),
            (2, self.pc, 0, 0, 0x30)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0x931d | 0x933c => {
                let v = w.dp_read(if pc == 0x931d { 0xb0 } else { 0x11 }, 1);
                w.load(v, 1);
                pc + 2
            }
            0x931f => {
                w.shift_left(1);
                pc + 1
            }
            0x9320 => {
                w.drain(6);
                w.cpu.x = w.cpu.a & 255;
                w.flags(w.cpu.x, 1);
                pc + 1
            }
            0x9321 => {
                w.drain(8);
                assert_eq!(w.read(0x029323, 1), 0x93);
                w.drain(6);
                let target = w.read(0x029319 + u32::from(w.cpu.x), 2);
                w.push(0x9323, 2);
                target
            }
            0x9324 | 0x9328 | 0x932d | 0x9334 | 0x9338 => {
                let target = match pc {
                    0x9324 => {
                        self.child = Some(LandingChild::PlayerAnimation);
                        0x07e6a6
                    }
                    0x9328 => {
                        self.child = Some(LandingChild::LinkOam);
                        0x0da18e
                    }
                    0x9334 => {
                        self.child = Some(LandingChild::Sprites);
                        0x068328
                    }
                    0x932d => 0x00f295,
                    _ => 0x00f312,
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(pc + 3, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                if self.child.is_none() {
                    self.spotlight = Some(Spotlight::new(target as u16));
                }
                pc + 4
            }
            0x9331 => {
                w.dp_increment(0xb0, 1, true);
                pc + 2
            }
            0x932c | 0x9333 | 0x9356 => {
                w.rts();
                self.complete = pc == 0x932c;
                w.cpu.pc
            }
            0x933e | 0x9351 => {
                let take = (w.cpu.p & ZERO != 0) == (pc == 0x9351);
                if w.branch(take) {
                    0x9356
                } else {
                    pc + 2
                }
            }
            0x9340..=0x934a => {
                let a = [0x96, 0x97, 0x98, 0x1e, 0x1f, 0xb0][usize::from((pc - 0x9340) / 2)];
                w.dp_store(a, 0, 1);
                pc + 2
            }
            0x934c => {
                w.drain(16);
                let v = w.read(0x132, 1);
                w.load(v, 1);
                pc + 3
            }
            0x934f => {
                w.compare_immediate(255, 1);
                pc + 2
            }
            0x9353 => {
                w.drain(16);
                w.write(0x12c, w.cpu.a & 255, 1);
                pc + 3
            }
            _ => unreachable!("landing caller {pc:04x}"),
        };
        if self.child.is_none() && self.spotlight.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn landing_caller_retains_source_stack_at_child_boundaries() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for a in [0x029319, 0x02931a, 0x02931b, 0x02931c, 0x029323] {
            assert_eq!(
                table_byte(a),
                Some(rom[(a >> 16) as usize * 0x8000 + (a as usize & 0x7fff)])
            );
        }
        for entry in [0x931d, 0x9331, 0x9338, 0x933c, 0x9324, 0x9328, 0x932c] {
            for variant in 0..4 {
                let mut machine = snes::Snes::new();
                machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                machine.ram.fill(0x35);
                machine.ram[0xb0] = variant & 1;
                machine.ram[0x11] = variant & 1;
                machine.ram[0x132] = if variant & 2 == 0 { 0xff } else { 0x11 };
                let returned: u16 = if matches!(entry, 0x9331 | 0x933c) {
                    0x9323
                } else {
                    0x87ae
                };
                machine.ram[0x1fd..0x1ff].copy_from_slice(&returned.to_le_bytes());
                let mut cpu = NativeNmiContext {
                    a: 0xab01,
                    x: 3,
                    y: 7,
                    d: 0,
                    db: 0,
                    bank: 2,
                    pc: entry,
                    sp: 0x1fc,
                    p: 0x30,
                };
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.pc = cpu.pc;
                machine.cpu.k = cpu.bank;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let timeline = CpuMasterTimeline::at_raster(
                    30003,
                    snes::CpuRasterPosition::new(224, 1300),
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
                let mut work = DungeonLanding {
                    pc: entry,
                    ..Default::default()
                };
                for _ in 0..40 {
                    if work.complete() || work.child().is_some() || work.spotlight.is_some() {
                        break;
                    }
                    let label = format!("entry={entry:04x} variant={variant} pc={:04x}", cpu.pc);
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
                }
                assert!(work.complete() || work.child().is_some() || work.spotlight.is_some());
                assert_eq!(ram, source.snes().ram);
            }
        }
    }
}
