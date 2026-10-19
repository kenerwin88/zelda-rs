//! Dungeon maintenance suffix, axis selection and the recoil predicate.
//! Selected exploding-wall and room-movement bodies retain their real calls.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeTail {
    Lamp,
    BlastWall,
    Movement { dir: u8 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonEdgeCaller {
    pc: u32,
    tail: Option<EdgeTail>,
}
impl Default for DungeonEdgeCaller {
    fn default() -> Self {
        Self {
            pc: 0x0287d1,
            tail: None,
        }
    }
}
impl DungeonEdgeCaller {
    pub(crate) fn tail(&self) -> Option<EdgeTail> {
        self.tail
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.pc >> 16 == 7 {
            0x07f439
        } else if self.pc >= 0x02885e {
            0x02885e
        } else {
            0x0287d1
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
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        assert_eq!((cpu.d, cpu.p & 0x18), (0, 0x10));
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
            0x0287d1 | 0x07f45d | 0x07f462 => {
                w.drain(16);
                let a = match pc {
                    0x0287d1 => 0x454,
                    0x07f45d => 0xfc2,
                    _ => 0xfc4,
                };
                let value = w.read(a, m);
                w.load(value, m);
                pc + 3
            }
            0x0287da | 0x028860 | 0x02886e | 0x028881 | 0x02888f | 0x028867 | 0x028888
            | 0x0288aa | 0x0288b4 | 0x07f43c | 0x07f44e => {
                let a = match pc {
                    0x0287da => 0x6c,
                    0x028860 => 0x30,
                    0x028881 => 0x31,
                    0x02886e => 0x20,
                    0x02888f => 0x22,
                    0x028867 | 0x028888 => 0x67,
                    0x07f43c => 0x5d,
                    0x07f44e => 0x46,
                    _ => 0x10,
                };
                let value = w.dp_read(a, m);
                w.load(value, m);
                pc + 2
            }
            0x0287d4 | 0x0287dc | 0x028865 | 0x028886 | 0x028878 | 0x02887f | 0x028899
            | 0x0288a0 | 0x0288a8 | 0x0288ae | 0x0288b8 | 0x07f440 | 0x07f444 | 0x07f448
            | 0x07f44c | 0x07f450 => {
                let z = w.cpu.p & ZERO != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0x0287d4 => (z, 0x0287da),
                    0x0287dc => (!z, 0x0287e1),
                    0x028865 => (z, 0x028881),
                    0x028886 => (z, 0x0288be),
                    0x028878 | 0x028899 => (!c, 0x0288a2),
                    0x02887f => (c, 0x0288a2),
                    0x0288a0 => (!c, 0x0288be),
                    0x0288a8 => (c, 0x0288be),
                    0x0288ae | 0x0288b8 => (!z, 0x0288be),
                    0x07f450 => (z, 0x07f46c),
                    _ => (z, 0x07f452),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0x0287d6 | 0x0288a4 | 0x0288b0 => {
                w.operand_then_internal(3);
                w.push(2, 1);
                w.push((pc + 3) as u16, 2);
                let target = match pc {
                    0x0287d6 => {
                        self.tail = Some(EdgeTail::BlastWall);
                        0x01d6c1
                    }
                    0x0288a4 => 0x07f439,
                    _ => {
                        self.tail = Some(EdgeTail::Movement { dir: w.cpu.x as u8 });
                        0x0288c5
                    }
                };
                target
            }
            0x0287de => {
                w.operand_then_internal(2);
                w.push(0x87e0, 2);
                0x02885e
            }
            0x02885e | 0x0288a2 | 0x0288be | 0x07f45b | 0x07f467 => {
                w.width(0x20, pc == 0x02885e || pc == 0x07f45b);
                pc + 2
            }
            0x028862 | 0x028883 | 0x028869 | 0x028870 | 0x02888a | 0x028891 => {
                w.mask(
                    match pc {
                        0x028869 => 12,
                        0x02888a => 3,
                        0x028870 | 0x028891 => 0x1ff,
                        _ => 255,
                    },
                    m,
                );
                pc + 3
            }
            0x02886c | 0x02888d | 0x0288bc | 0x07f460 | 0x07f465 => {
                let a = match pc {
                    0x02886c | 0x02888d => 0,
                    0x0288bc => 0x11,
                    0x07f460 => 0x22,
                    _ => 0x20,
                };
                w.dp_store(a, w.cpu.a & if m == 1 { 255 } else { 65535 }, m);
                pc + 2
            }
            0x028873 | 0x02887a | 0x028894 | 0x02889b => {
                w.drain(8);
                w.cpu.x = match pc {
                    0x028873 => 3,
                    0x02887a => 2,
                    0x028894 => 1,
                    _ => 0,
                };
                w.flags(w.cpu.x, 1);
                pc + 2
            }
            0x028875 | 0x02887c | 0x028896 | 0x02889d | 0x0288ac | 0x0288b6 | 0x07f43e
            | 0x07f442 | 0x07f446 | 0x07f44a => {
                w.compare_immediate(
                    match pc {
                        0x028875 => 4,
                        0x02887c => 476,
                        0x028896 => 8,
                        0x02889d => 489,
                        0x07f43e => 3,
                        0x07f442 => 8,
                        0x07f446 => 9,
                        0x07f44a => 10,
                        _ => 7,
                    },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x0288ba | 0x07f456 => {
                w.immediate(if pc == 0x0288ba { 2 } else { 3 }, 1);
                pc + 2
            }
            0x0288c0 => {
                w.rts();
                self.tail = Some(EdgeTail::Lamp);
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x07f439 | 0x07f43a => {
                w.drain(6);
                w.push(
                    if pc == 0x07f439 {
                        u16::from(w.cpu.db)
                    } else {
                        7
                    },
                    1,
                );
                pc + 1
            }
            0x07f43b | 0x07f46a | 0x07f46d => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x07f452 | 0x07f454 => {
                w.dp_store(if pc == 0x07f452 { 0x27 } else { 0x28 }, 0, 1);
                pc + 2
            }
            0x07f458 => {
                w.drain(16);
                w.write(0x2c6, w.cpu.a & 255, 1);
                pc + 3
            }
            0x07f469 => {
                w.drain(6);
                w.cpu.p |= CARRY;
                pc + 1
            }
            0x07f46c => {
                w.clear_carry();
                pc + 1
            }
            0x07f46b | 0x07f46e => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            _ => unreachable!("dungeon edge caller {pc:06x}"),
        };
        if self.pc == 0x0287e1 {
            self.tail = Some(EdgeTail::Lamp);
        }
        w.cpu.bank = (self.pc >> 16) as u8;
        w.cpu.pc = self.pc as u16;
        Some(w.cycles)
    }
}
#[cfg(test)]
pub(crate) fn seed_case(ram: &mut [u8], case: usize) {
    let xs = [0u16, 7, 8, 488, 489, 511, 65535];
    let ys = [0u16, 3, 4, 475, 476, 511, 65535];
    ram[0x454] = if case % 16 == 0 { 1 } else { 0 };
    ram[0x455] = 0xff;
    ram[0x6c] = if case % 16 == 1 { 1 } else { 0 };
    ram[0x30] = (case / 2 % 2) as u8;
    ram[0x31] = (case % 2) as u8;
    ram[0x32] = 0xff;
    ram[0x20..0x22].copy_from_slice(&ys[case / 4 % 7].to_le_bytes());
    ram[0x22..0x24].copy_from_slice(&xs[case / 28 % 7].to_le_bytes());
    ram[0x67] = (case / 196 % 16) as u8;
    ram[0x5d] = [0, 3, 8, 9, 10, 255][case / 16 % 6];
    ram[0x46] = (case / 96 % 2) as u8;
    ram[0x10] = if case / 192 % 2 == 0 { 7 } else { 6 };
    ram[0x11] = 0;
    ram[0x27] = 0xff;
    ram[0x28] = 127;
    ram[0xfc2..0xfc6].copy_from_slice(&[0xef, 0xbe, 0x23, 0x01]);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_edge_caller_matches_source_guards_axis_thresholds_and_recoil_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0usize; 3];
        let mut dirs = std::collections::BTreeSet::new();
        for case in 0..4096usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0x2100,
                x: 255,
                y: 8,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x87d1,
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
                snes::CpuRasterPosition::new(50, 372),
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
            let mut work = DungeonEdgeCaller::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                super::super::native_work::compare_instruction(
                    &format!("edge case={case} pc={pc:04x}"),
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
            match work.tail().unwrap() {
                EdgeTail::Lamp => {
                    tails[0] += 1;
                    assert_eq!(
                        (cpu.bank, cpu.pc, cpu.sp, cpu.db, cpu.p & 0x30),
                        (2, 0x87e1, 0x1fc, 0, 0x30)
                    );
                }
                EdgeTail::BlastWall => {
                    tails[1] += 1;
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (1, 0xd6c1, 0x1f9));
                }
                EdgeTail::Movement { dir } => {
                    dirs.insert(dir);
                    tails[2] += 1;
                    assert_eq!((cpu.bank, cpu.pc, cpu.sp), (2, 0x88c5, 0x1f7));
                }
            }
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(dirs.len(), 4);
        println!("native dungeon edge: 4096 cases, {count} source instructions, tails={tails:?}, all four movement directions");
    }
}
