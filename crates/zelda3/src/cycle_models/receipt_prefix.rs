//! Item-receipt state dispatch at its actual ancilla body entry.
//! Selected helpers retain their source call stack and run only once.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiptTail {
    Draw,
    Movement,
    Message,
    LabelA,
    Finish,
    SongPoll,
    Rupees,
    Sound,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReceiptPrefix {
    native_movement: bool,
    movement: Option<super::receipt_movement::ReceiptMovement>,
    tail: Option<ReceiptTail>,
}
impl ReceiptPrefix {
    pub(crate) fn with_native_movement(mut self, enabled: bool) -> Self {
        self.native_movement = enabled;
        self
    }
    pub(crate) fn tail(&self) -> Option<ReceiptTail> {
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
        if let Some(work) = &mut self.movement {
            let cycles = work.step(cpu, clock, bus);
            if work.complete() {
                self.tail = Some(ReceiptTail::Draw);
            }
            return cycles;
        }
        assert_eq!(
            (cpu.bank, cpu.db, cpu.d, cpu.sp, cpu.p & 0x18),
            (8, 8, 0, 0x1e8, 0x10)
        );
        assert!(cpu.x < 10);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = w.cpu.pc;
        w.fetch();
        let next = match pc {
            0xc38a | 0xc3ac | 0xc3b8 | 0xc3bf | 0xc3c6 | 0xc3e5 | 0xc3ee | 0xc518 | 0xc528
            | 0xc535 => {
                let (address, indexed) = match pc {
                    0xc38a => (0x2e4, false),
                    0xc3ac | 0xc3bf | 0xc528 => (0xc54, true),
                    0xc3b8 | 0xc3ee | 0xc535 => (0xc5e, true),
                    0xc3c6 => (0xc68, true),
                    _ => (0x3b1, true),
                };
                w.drain(16);
                let a = address + if indexed { w.cpu.x } else { 0 };
                if indexed && address & 0xff00 != a & 0xff00 {
                    w.drain(6);
                }
                let v = w.read(w.absolute(a), 1);
                w.load(v, 1);
                pc + 3
            }
            0xc391 => {
                let v = w.dp_read(0x11, 1);
                w.load(v, 1);
                pc + 2
            }
            0xc38d | 0xc395 | 0xc399 | 0xc39d | 0xc3b1 | 0xc3bb | 0xc3c2 | 0xc3cb | 0xc3ea
            | 0xc3f1 | 0xc3f5 | 0xc3f9 | 0xc520 | 0xc524 | 0xc52b | 0xc538 => {
                let value = match pc {
                    0xc38d | 0xc39d | 0xc3c2 | 0xc52b => 2,
                    0xc395 => 0x2b,
                    0xc399 => 9,
                    0xc3b1 => 3,
                    0xc3bb | 0xc3ea => 1,
                    0xc3cb => 17,
                    0xc3f1 => 0x37,
                    0xc3f5 => 0x38,
                    0xc3f9 => 0x39,
                    0xc520 => 0,
                    0xc524 => 40,
                    _ => 0x17,
                };
                w.compare_immediate(value, 1);
                pc + 2
            }
            0xc38f | 0xc393 | 0xc397 | 0xc39b | 0xc39f | 0xc3af | 0xc3b3 | 0xc3bd | 0xc3c4
            | 0xc3c9 | 0xc3cd | 0xc3e8 | 0xc3ec | 0xc3f3 | 0xc3f7 | 0xc3fb | 0xc51b | 0xc522
            | 0xc526 | 0xc52d | 0xc53a | 0xc544 => {
                let z = w.cpu.p & ZERO != 0;
                let (take, target) = match pc {
                    0xc38f => (z, 0xc3a6),
                    0xc393 | 0xc397 | 0xc39b => (z, 0xc3a9),
                    0xc39f => (!z, 0xc3a6),
                    0xc3af | 0xc3b3 => (z, 0xc3b8),
                    0xc3bd => (!z, 0xc3e2),
                    0xc3c4 => (z, 0xc3e2),
                    0xc3c9 | 0xc3e8 => (z, 0xc40d),
                    0xc3cd | 0xc3ec => (!z, 0xc40a),
                    0xc3f3 | 0xc3f7 => (z, 0xc3fd),
                    0xc3fb => (!z, 0xc407),
                    0xc51b => (w.cpu.p & NEGATIVE == 0, 0xc520),
                    0xc522 => (z, 0xc546),
                    0xc526 | 0xc52d => (if pc == 0xc526 { !z } else { z }, 0xc53c),
                    0xc53a => (!z, 0xc53f),
                    _ => (true, 0xc53c),
                };
                w.branch(take);
                if take {
                    target
                } else {
                    pc + 2
                }
            }
            0xc3a1 | 0xc3d1 | 0xc3d9 | 0xc53f => {
                let (v, m) = match pc {
                    0xc3a1 => (16, 1),
                    0xc3d1 => (0xdf3, 2),
                    0xc3d9 => (14, 1),
                    _ => (15, 1),
                };
                w.drain(8 * u32::from(m));
                w.load(v, m);
                pc + 1 + u16::from(m)
            }
            0xc3a3 | 0xc3d4 | 0xc3db => {
                let (address, m) = match pc {
                    0xc3a3 => (w.absolute(0xc68 + w.cpu.x), 1),
                    0xc3d4 => (w.absolute(0x2cd), 2),
                    _ => (0x7ef3cc, 1),
                };
                w.drain(if pc == 0xc3db { 24 } else { 16 });
                if pc == 0xc3a3 {
                    w.drain(6);
                }
                w.write(address, if m == 1 { w.cpu.a & 255 } else { w.cpu.a }, m);
                pc + if pc == 0xc3db { 4 } else { 3 }
            }
            0xc3a9 | 0xc3e2 | 0xc515 => {
                w.drain(16);
                let a = w.absolute(if pc == 0xc3a9 { 0xfc1 } else { 0x3b1 + w.cpu.x });
                if pc != 0xc3a9 {
                    w.drain(6);
                }
                let old = w.read(a, 1);
                let v = if pc == 0xc3a9 {
                    old.wrapping_add(1)
                } else {
                    old.wrapping_sub(1)
                } & 255;
                w.drain(6);
                w.flags(v, 1);
                w.write(a, v, 1);
                pc + 3
            }
            0xc3cf | 0xc3d7 => {
                w.operand_then_internal(1);
                if pc == 0xc3cf {
                    w.cpu.p &= !0x20
                } else {
                    w.cpu.p |= 0x20
                };
                pc + 2
            }
            0xc3a6 | 0xc3b5 | 0xc3df | 0xc407 | 0xc40a | 0xc51d | 0xc53c => {
                w.operand_then_internal(2);
                match pc {
                    0xc3b5 => 0xc515,
                    0xc3df => 0xc546,
                    0xc407 => 0xc3df,
                    0xc51d => 0xc43b,
                    0xc53c => 0xc5e0,
                    _ => 0xc5f7,
                }
            }
            0xc52f | 0xc541 => {
                let far = pc == 0xc52f;
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(8, 1);
                }
                w.push(pc + u16::from(width), 2);
                w.cpu.bank = if far { 9 } else { 8 };
                if far {
                    0xad6c
                } else {
                    0x800e
                }
            }
            _ => unreachable!("receipt prefix pc={pc:04x}"),
        };
        w.cpu.pc = next;
        self.tail = match (w.cpu.bank, next) {
            (8, 0xc5f7) => Some(ReceiptTail::Draw),
            (8, 0xc5e0) => {
                if self.native_movement {
                    self.movement = Some(Default::default());
                    None
                } else {
                    Some(ReceiptTail::Movement)
                }
            }
            (8, 0xc546) => Some(ReceiptTail::Message),
            (8, 0xc40d) => Some(ReceiptTail::LabelA),
            (8, 0xc43b) => Some(ReceiptTail::Finish),
            (8, 0xc3fd) => Some(ReceiptTail::SongPoll),
            (9, 0xad6c) => Some(ReceiptTail::Rupees),
            (8, 0x800e) => Some(ReceiptTail::Sound),
            _ => None,
        };
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    let k = case % 10;
    let v = case / 10;
    r[0x2e4] = if v % 9 == 8 { 2 } else { 1 };
    r[0x11] = [0, 43, 9, 2, 1, 255][v % 6];
    r[0xc54 + k] = [0, 3, 1, 2, 255][v / 6 % 5];
    r[0xc5e + k] = [1, 0x17, 0x37, 0x38, 0x39, 0x20, 0x42][v / 30 % 7];
    r[0xc68 + k] = [0, 17, 1, 255][v / 210 % 4];
    r[0x3b1 + k] = [0, 1, 2, 41, 25, 128, 129, 255][v / 840 % 8];
    r[0xfc1] = case as u8;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_prefix_matches_source_all_gates_and_selected_handoff_stacks() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 8];
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..67200usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: (case % 10) as u16,
                y: 7,
                d: 0,
                db: 8,
                bank: 8,
                pc: 0xc38a,
                sp: 0x1e8,
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
                snes::CpuRasterPosition::new(58, 1334),
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
            let mut work = ReceiptPrefix::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("receipt prefix case={case} pc={pc:04x}"),
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
            tails[work.tail().unwrap() as usize] += 1;
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x08c38a, 0x08c3a3, 0x08c3d4, 0x08c3db, 0x08c3e2, 0x08c515, 0x08c52f,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails[..7].iter().all(|n| *n > 0));
        println!("native receipt prefix: 67200 cases, {count} source instructions, {} PCs, tails={tails:?}",visited.len());
    }
}
