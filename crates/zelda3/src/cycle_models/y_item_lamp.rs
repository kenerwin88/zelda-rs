//! Lamp caller with its native activation guard and explicit effect children.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    y_item_button_guard::YItemButtonGuard,
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LampChild {
    MagicPayment,
    MagicPowder,
    LightTorch,
    LampFlame,
}
impl LampChild {
    fn return_pc(self) -> u16 {
        match self {
            Self::MagicPayment => 0xa261,
            Self::MagicPowder => 0xa26b,
            Self::LightTorch => 0xa26f,
            Self::LampFlame => 0xa277,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YItemLamp {
    pc: u16,
    guard: Option<YItemButtonGuard>,
    child: Option<LampChild>,
    complete: bool,
}
impl Default for YItemLamp {
    fn default() -> Self {
        Self {
            pc: 0xa24d,
            guard: None,
            child: None,
            complete: false,
        }
    }
}
impl YItemLamp {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn child(&self) -> Option<LampChild> {
        self.child
    }
    pub(crate) fn return_from_child(&mut self, cpu: &NativeNmiContext) {
        let child = self.child.take().expect("lamp child return");
        assert_eq!((cpu.bank, cpu.pc), (7, child.return_pc()));
        self.pc = cpu.pc;
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete || self.child.is_some() {
            return None;
        }
        if let Some(guard) = &mut self.guard {
            let cycles = guard.step(cpu, clock, bus);
            if guard.complete() {
                assert_eq!((cpu.bank, cpu.pc), (7, 0xa254));
                self.pc = cpu.pc;
                self.guard = None;
            }
            return cycles;
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
            0xa24d | 0xa280 => {
                let value = w.dp_read(if self.pc == 0xa24d { 0x6c } else { 0x3c }, 1);
                w.load(value, 1);
                self.pc + 2
            }
            0xa24f | 0xa254 | 0xa25a | 0xa261 | 0xa284 => {
                let taken = match self.pc {
                    0xa24f | 0xa284 => w.cpu.p & ZERO == 0,
                    0xa25a => w.cpu.p & ZERO != 0,
                    _ => w.cpu.p & CARRY == 0,
                };
                if w.branch(taken) {
                    if matches!(self.pc, 0xa25a | 0xa261) {
                        0xa277
                    } else {
                        0xa288
                    }
                } else {
                    self.pc + 2
                }
            }
            0xa251 => {
                w.item_call();
                self.guard = Some(YItemButtonGuard::default());
                0xb073
            }
            0xa256 => {
                w.drain(24);
                let value = w.read(0x7ef34a, 1);
                w.load(value, 1);
                0xa25a
            }
            0xa25c => {
                w.item_x(6, 1);
                0xa25e
            }
            0xa25e => {
                w.item_call();
                self.child = Some(LampChild::MagicPayment);
                0xb0ab
            }
            0xa263 | 0xa26f => {
                w.drain(8);
                w.cpu.y = if self.pc == 0xa263 { 0 } else { 2 };
                w.flags(w.cpu.y, 1);
                self.pc + 2
            }
            0xa265 | 0xa271 => {
                w.immediate(if self.pc == 0xa265 { 0x1a } else { 0x2f }, 1);
                self.pc + 2
            }
            0xa267 | 0xa26b | 0xa273 => {
                w.operand_then_internal(3);
                w.push(7, 1);
                w.push(self.pc + 3, 2);
                let (bank, target, child) = match self.pc {
                    0xa267 => (9, 0x92f0, LampChild::MagicPowder),
                    0xa26b => (1, 0xf3ec, LampChild::LightTorch),
                    _ => (9, 0x8f1c, LampChild::LampFlame),
                };
                w.cpu.bank = bank;
                self.child = Some(child);
                target
            }
            0xa277 => {
                w.drain(16);
                w.write(w.absolute(0x301), 0, 1);
                0xa27a
            }
            0xa27a | 0xa27c | 0xa27e | 0xa286 => {
                w.dp_store(
                    match self.pc {
                        0xa27a => 0x3a,
                        0xa27c => 0x3c,
                        0xa27e => 0x50,
                        _ => 0x5e,
                    },
                    0,
                    1,
                );
                self.pc + 2
            }
            0xa282 => {
                w.compare_immediate(9, 1);
                0xa284
            }
            0xa288 => {
                w.rts();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("native lamp {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
