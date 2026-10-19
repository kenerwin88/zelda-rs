//! Recoil classification and health arithmetic, with explicit remaining bodies.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecoilTail {
    Returned,
    Burn,
    Stun(u8),
    Death,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpriteRecoil {
    pc: u16,
    tail: Option<RecoilTail>,
}
impl Default for SpriteRecoil {
    fn default() -> Self {
        Self {
            pc: 0xeec8,
            tail: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sprite_recoil_classification_health_and_actual_return_match_source() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut tails = [0; 4];
        for case in 0..6912usize {
            let k = case % 16;
            let state = (case / 16 % 12) as u8;
            let damage = [0, 1, 10, 250, 251, 252, 253, 254, 255][case / 192 % 9];
            let health = [0, 1, 10, 255][case / 1728 % 4];
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            machine.ram[0xdd0 + k] = state;
            machine.ram[0xce2 + k] = damage;
            machine.ram[0xe50 + k] = health;
            machine.ram[0x1ea..0x1ec].copy_from_slice(&[0x80, 0x84]);
            let mut cpu = NativeNmiContext {
                a: 0xabcd,
                x: k as u16,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0xeec8,
                sp: 0x1e9,
                p: 0x30 | (case as u8 & 0xc3),
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
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(76, 1200),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                clock,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = SpriteRecoil::default();
            let mut owned = 0;
            while work.tail().is_none() {
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("recoil case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        owned += work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                instructions += 1;
            }
            let (tail, price) = if state < 9 {
                (RecoilTail::Returned, 112)
            } else if damage == 253 {
                (RecoilTail::Burn, 160)
            } else if damage >= 251 {
                (RecoilTail::Stun(damage), 198)
            } else if health > damage {
                (RecoilTail::Returned, 462)
            } else if health == damage {
                (RecoilTail::Death, 404)
            } else {
                (RecoilTail::Death, 414)
            };
            assert_eq!(work.tail(), Some(tail));
            assert_eq!(owned, price);
            let index = match tail {
                RecoilTail::Returned => 0,
                RecoilTail::Burn => 1,
                RecoilTail::Stun(_) => 2,
                RecoilTail::Death => 3,
            };
            tails[index] += 1;
            assert_eq!(
                cpu.sp,
                if tail == RecoilTail::Returned {
                    0x1eb
                } else {
                    0x1e9
                }
            );
            assert_eq!(
                cpu.pc,
                match tail {
                    RecoilTail::Returned => 0x8481,
                    RecoilTail::Burn => 0xeed9,
                    RecoilTail::Stun(_) => 0xeefc,
                    RecoilTail::Death => 0xef74,
                }
            );
        }
        assert!(tails.iter().all(|n| *n > 0));
        println!("native recoil prefix:6912 cases, {instructions} Source instructions, tails={tails:?}, actual returns and exact prefix prices");
    }
}
impl SpriteRecoil {
    pub(crate) fn tail(&self) -> Option<RecoilTail> {
        self.tail
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.tail.is_some() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (6, self.pc, 6, 0, 0x30)
        );
        assert!(cpu.x < 16);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xeec8 | 0xeed2 | 0xef61 | 0xef67 => {
                w.drain(16);
                let address = w.absolute(
                    (if self.pc == 0xeec8 {
                        0xdd0
                    } else if self.pc == 0xef61 {
                        0xe50
                    } else {
                        0xce2
                    }) + w.cpu.x,
                );
                let value = w.read(address, 1);
                if self.pc == 0xef67 {
                    w.sbc(value, 1);
                } else {
                    w.load(value, 1);
                }
                self.pc + 3
            }
            0xeecb | 0xeed5 | 0xeef8 => {
                w.compare_immediate(
                    match self.pc {
                        0xeecb => 9,
                        0xeed5 => 253,
                        _ => 251,
                    },
                    1,
                );
                self.pc + 2
            }
            0xeecd => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0xeef7
                } else {
                    0xeecf
                }
            }
            0xeed7 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xeef8
                } else {
                    self.tail = Some(RecoilTail::Burn);
                    0xeed9
                }
            }
            0xeefa => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    0xef61
                } else {
                    self.tail = Some(RecoilTail::Stun(w.cpu.a as u8));
                    0xeefc
                }
            }
            0xef70 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    self.tail = Some(RecoilTail::Death);
                    0xef74
                } else {
                    0xef72
                }
            }
            0xef72 => {
                if w.branch(w.cpu.p & CARRY != 0) {
                    0xefe7
                } else {
                    self.tail = Some(RecoilTail::Death);
                    0xef74
                }
            }
            0xeecf => {
                w.drain(16);
                w.write(w.absolute(0xfb5), w.cpu.a & 255, 1);
                0xeed2
            }
            0xef64 => {
                w.dp_store(0, w.cpu.a & 255, 1);
                0xef66
            }
            0xef66 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                0xef67
            }
            0xef6a | 0xef6d => {
                w.operand_then_internal(2);
                let address = w.absolute((if self.pc == 0xef6a { 0xe50 } else { 0xce2 }) + w.cpu.x);
                w.write(
                    address,
                    if self.pc == 0xef6a { w.cpu.a & 255 } else { 0 },
                    1,
                );
                self.pc + 3
            }
            0xeef7 | 0xefe7 => {
                w.rts();
                self.tail = Some(RecoilTail::Returned);
                w.cpu.pc
            }
            _ => unreachable!("recoil prefix {:04x}", self.pc),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}
