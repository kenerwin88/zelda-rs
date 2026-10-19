//! Room-maintenance caller and torch/door work after the room tags return.
use super::{
    dungeon_interactions::DungeonInteractions,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InteractionTail {
    Lamp,
    Returned,
    TorchExpired { index: u8 },
    KeyDoor { pos: u16 },
    EyeDoor,
    Slash,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonInteractionCaller {
    pc: u16,
    interactions: Option<DungeonInteractions>,
    tail: Option<InteractionTail>,
}
impl Default for DungeonInteractionCaller {
    fn default() -> Self {
        Self {
            pc: 0x87c9,
            interactions: None,
            tail: None,
        }
    }
}
impl DungeonInteractionCaller {
    pub(crate) fn tail(&self) -> Option<InteractionTail> {
        self.tail
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.interactions.is_some() {
            0x01ce70
        } else {
            0x0287c9
        }
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
        if let Some(work) = &mut self.interactions {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                self.tail = Some(InteractionTail::Returned);
            } else if let Some(pc) = work.pending_body() {
                self.tail = Some(match pc {
                    0x01f4a6 => InteractionTail::TorchExpired { index: cpu.x as u8 },
                    0x01ceed => InteractionTail::KeyDoor { pos: cpu.x },
                    0x01cff1 => InteractionTail::EyeDoor,
                    0x01d059 => InteractionTail::Slash,
                    _ => unreachable!("interaction body {pc:06x}"),
                });
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
        w.fetch();
        self.pc = match self.pc {
            0x87c9 => {
                let value = w.dp_read(0x11, 1);
                w.load(value, 1);
                0x87cb
            }
            0x87cb => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    self.tail = Some(InteractionTail::Lamp);
                    0x87e1
                } else {
                    0x87cd
                }
            }
            0x87cd => {
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push(0x87d0, 2);
                w.cpu.bank = 1;
                w.cpu.pc = 0xce70;
                self.interactions = Some(Default::default());
                0x87d1
            }
            _ => unreachable!("interaction caller {:04x}", self.pc),
        };
        if self.interactions.is_none() {
            w.cpu.pc = self.pc;
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize) {
    let kind = case % 6;
    let variant = case / 6;
    ram[0x11] = if kind == 0 {
        (variant % 255 + 1) as u8
    } else {
        0
    };
    ram[0x1a] = if kind == 2 || kind == 1 && variant % 3 == 0 {
        236
    } else {
        237
    };
    ram[0x112] = if kind == 1 && variant % 3 == 1 { 1 } else { 0 };
    ram[0x2e4] = if kind == 3 || kind == 1 && variant & 1 == 0 {
        0
    } else {
        1
    };
    ram[0x2f] = (variant % 4 * 2) as u8;
    ram[0xee] = (variant / 4 % 2) as u8;
    ram[0x436] = if kind == 4 { 0 } else { 255 };
    ram[0x437] = 0;
    ram[0x3a] = if kind == 5 { 0x80 } else { 0 };
    ram[0x20..0x24].copy_from_slice(&[0xfa, 0x21, 0x40, 9]);
    ram[0x4f0..0x500].fill(0);
    if kind == 2 {
        let index = variant % 16;
        for i in 0..index {
            ram[0x4f0 + i] = if i % 3 == 0 {
                0
            } else {
                (variant % 253 + 2) as u8
            };
        }
        ram[0x4f0 + index] = 1;
        for i in index + 1..16 {
            ram[0x4f0 + i] = 1;
        }
    } else if kind == 1 && variant % 3 == 0 {
        for i in 0..16 {
            ram[0x4f0 + i] = if i % 3 == 0 {
                0
            } else {
                (variant % 253 + 2) as u8
            };
        }
    }
    ram[0x12000..0x14000].fill(2);
    ram[0x4b8..0x4ba].copy_from_slice(&[0x77, 0x99]);
    if kind == 3 {
        let dir = variant % 4;
        let y = 0x21fau16.wrapping_add([7, 24, 8, 8][dir]);
        let x = 0x940u16.wrapping_add([0, 0, 65535, 17][dir]);
        let mut pos = ((y & 0x1f8) << 3) | ((x & 0x1f8) >> 3) | ((ram[0xee] as u16) << 12);
        if variant / 8 % 2 != 0 {
            pos = pos.wrapping_add([2, 2, 128, 128][dir]);
        }
        ram[0x12000 + pos as usize] = 0xf0 + (variant % 16) as u8;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_interaction_caller_matches_source_timers_probes_and_real_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0usize; 6];
        let mut expired = std::collections::BTreeSet::new();
        for case in 0..2048 {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0x1111,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87c9,
                sp: 0x1fc,
                p: 0x30 | (case as u8 & 0xc3),
            };
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let clock = CpuMasterTimeline::at_raster(
                4586,
                snes::CpuRasterPosition::new(49, 1298),
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
            let mut work = DungeonInteractionCaller::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("interaction caller case={case} pc={pc:04x}"),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, bus).unwrap();
                    },
                );
                work = work.clone();
                count += 1;
            }
            let (slot, bank, pc, sp) = match work.tail().unwrap() {
                InteractionTail::Lamp => (0, 2, 0x87e1, 0x1fc),
                InteractionTail::Returned => (1, 2, 0x87d1, 0x1fc),
                InteractionTail::TorchExpired { index } => {
                    expired.insert(index);
                    assert_eq!(index as usize, case / 6 % 16);
                    (2, 1, 0xf4a6, 0x1f5)
                }
                InteractionTail::KeyDoor { pos } => {
                    assert_eq!(pos, cpu.x);
                    (3, 1, 0xceed, 0x1f9)
                }
                InteractionTail::EyeDoor => (4, 1, 0xcff1, 0x1f9),
                InteractionTail::Slash => (5, 1, 0xd059, 0x1f9),
            };
            tails[slot] += 1;
            assert_eq!((cpu.bank, cpu.pc, cpu.sp), (bank, pc, sp));
            assert_eq!(ram, source.snes().ram);
        }
        assert!(tails.iter().all(|n| *n > 300));
        assert_eq!(expired.len(), 16);
        println!("native interaction caller: 2048 cases, {count} source instructions, tails={tails:?}, all 16 expiry indices");
    }
}
