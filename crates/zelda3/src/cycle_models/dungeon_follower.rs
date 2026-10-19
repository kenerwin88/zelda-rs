//! Real Tagalong wrapper and timed follower dispatch, including message freedom.
//! Trigger, movement and dialogue bodies retain their actual call stacks.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FollowerTail {
    Returned,
    Trigger,
    NoTimedMessage,
    Message { index: u8 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonFollower {
    tail: Option<FollowerTail>,
    message_index: u8,
}
impl Default for DungeonFollower {
    fn default() -> Self {
        Self {
            tail: None,
            message_index: 0,
        }
    }
}
impl DungeonFollower {
    pub(crate) fn tail(&self) -> Option<FollowerTail> {
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
        assert_eq!((cpu.d, cpu.p & 0x18), (0, 0x10));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let pc = (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc);
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        w.fetch();
        let next = match pc {
            0x068365 | 0x099f94 | 0x099ffa | 0x09a01f => {
                let (target, far) = match pc {
                    0x068365 => (0x099f91, true),
                    0x099f94 => (0x099fc4, false),
                    0x099ffa => (0x07f46f, true),
                    _ => (0x0ffdaa, true),
                };
                let width = if far { 3 } else { 2 };
                w.operand_then_internal(width);
                if far {
                    w.push(u16::from(w.cpu.bank), 1)
                }
                w.push(pc as u16 + u16::from(width), 2);
                target
            }
            0x099f91 | 0x099f92 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0x099f92 { w.cpu.bank } else { w.cpu.db }),
                    1,
                );
                pc + 1
            }
            0x099f93 | 0x099f97 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0x099fc4 | 0x099fd4 => {
                w.drain(24);
                let v = w.read(0x7ef3cc, 1);
                w.load(v, 1);
                pc + 4
            }
            0x099fcb | 0x07f471 | 0x07f475 | 0x07f479 => {
                w.compare_immediate(
                    match pc {
                        0x099fcb => 14,
                        0x07f471 => 0,
                        0x07f475 => 4,
                        _ => 17,
                    },
                    1,
                );
                pc + 2
            }
            0x099fd2 => {
                w.drain(8);
                w.cpu.y = 2;
                w.flags(2, 1);
                pc + 2
            }
            0x099fd8 => {
                w.drain(16);
                let rhs = w.read(w.absolute(0x9fb5 + w.cpu.y), 1);
                let lhs = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(lhs >= rhs);
                w.flags(lhs.wrapping_sub(rhs) & 255, 1);
                pc + 3
            }
            0x099fdd => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x099fc8 | 0x099fcd | 0x099fdb | 0x099fde | 0x099fe5 | 0x099fe9 | 0x099fef
            | 0x099ff6 | 0x099ffe | 0x09a006 | 0x09a028 | 0x07f473 | 0x07f477 | 0x07f47b
            | 0x07f496 => {
                let z = w.cpu.p & ZERO != 0;
                let n = w.cpu.p & NEGATIVE != 0;
                let c = w.cpu.p & CARRY != 0;
                let (take, target) = match pc {
                    0x099fc8 => (!z, 0x099fcb),
                    0x099fcd => (!z, 0x099fd2),
                    0x099fdb => (z, 0x099fe3),
                    0x099fde => (!n, 0x099fd4),
                    0x099fe5 => (!z, 0x09a024),
                    0x099fe9 => (!z, 0x099ff1),
                    0x099fef => (!z, 0x09a024),
                    0x099ff6 => (!n, 0x09a024),
                    0x099ffe => (c, 0x09a008),
                    0x09a006 => (true, 0x09a024),
                    0x09a028 => (!z, 0x09a02b),
                    0x07f473 | 0x07f477 => (z, 0x07f47d),
                    0x07f47b | 0x07f496 => (!z, 0x07f49a),
                    _ => unreachable!(),
                };
                w.branch(take);
                if take {
                    target
                } else {
                    pc + 2
                }
            }
            0x099fcf | 0x099fe0 => {
                w.operand_then_internal(2);
                if pc == 0x099fcf {
                    0x09a59e
                } else {
                    0x09a02b
                }
            }
            0x099fe3 | 0x099feb | 0x07f46f | 0x07f47d => {
                let v = w.dp_read(
                    match pc {
                        0x099fe3 => 0x11,
                        0x099feb => 0x8a,
                        0x07f46f => 0x5d,
                        _ => 0x3a,
                    },
                    1,
                );
                w.load(v, 1);
                pc + 2
            }
            0x099fe7 | 0x09a026 => {
                w.drain(8);
                let rhs = if pc == 0x099fe7 { 2 } else { 0 };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= rhs);
                w.flags(w.cpu.y.wrapping_sub(rhs) & 255, 1);
                pc + 2
            }
            0x099fed | 0x07f47f | 0x09a00c => {
                w.drain(8 * u32::from(m));
                w.load(
                    w.cpu.a
                        & match pc {
                            0x099fed => 0x40,
                            0x07f47f => 0x80,
                            _ => 0xff,
                        },
                    m,
                );
                pc + 1 + u32::from(m)
            }
            0x099ff1 | 0x099ff8 | 0x09a008 | 0x09a01d | 0x09a024 => {
                w.operand_then_internal(1);
                if matches!(pc, 0x099ff1 | 0x09a008) {
                    w.cpu.p &= !0x20
                } else {
                    w.cpu.p |= 0x20
                }
                pc + 2
            }
            0x099ff3 => {
                assert_eq!(m, 2);
                w.drain(16);
                let a = w.absolute(0x2cd);
                let v = w.read(a, 2).wrapping_sub(1);
                w.drain(6);
                w.write_high_low(a, v, 2);
                w.flags(v, 2);
                pc + 3
            }
            0x09a000 | 0x09a003 => {
                w.drain(16);
                w.write(w.absolute(if pc == 0x09a000 { 0x2cd } else { 0x2ce }), 0, 1);
                pc + 3
            }
            0x07f481 | 0x07f484 | 0x07f487 | 0x07f48a | 0x07f48d | 0x07f490 | 0x07f493 => {
                w.drain(16);
                let a = match pc {
                    0x07f481 => 0x377,
                    0x07f484 => 0x301,
                    0x07f487 => 0x37a,
                    0x07f48a => 0x2ec,
                    0x07f48d => 0x314,
                    0x07f490 => 0x308,
                    _ => 0x376,
                };
                let v = w.read(w.absolute(a), 1);
                w.load((w.cpu.a & 255) | v, 1);
                pc + 3
            }
            0x07f498 | 0x07f49a => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(pc == 0x07f498);
                pc + 1
            }
            0x09a00a => {
                self.message_index = w.cpu.y as u8;
                w.drain(6);
                w.push(w.cpu.y, 1);
                pc + 1
            }
            0x09a00b => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                pc + 1
            }
            0x09a00f => {
                w.drain(6);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.a & 0x8000 != 0);
                w.load(w.cpu.a << 1, 2);
                pc + 1
            }
            0x09a010 => {
                w.drain(6);
                w.cpu.y = w.cpu.a & 255;
                w.flags(w.cpu.y, 1);
                pc + 1
            }
            0x09a011 | 0x09a017 => {
                w.drain(16);
                let v = w.read(
                    w.absolute(if pc == 0x09a011 { 0x9fb8 } else { 0x9fbe }) + u32::from(w.cpu.y),
                    2,
                );
                w.load(v, 2);
                pc + 3
            }
            0x09a014 | 0x09a01a => {
                w.drain(16);
                w.write(
                    w.absolute(if pc == 0x09a014 { 0x2cd } else { 0x1cf0 }),
                    w.cpu.a,
                    2,
                );
                pc + 3
            }
            0x099fca | 0x09a02a => {
                w.rts();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x099f98 | 0x07f499 | 0x07f49b => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            _ => unreachable!("native follower pc={pc:06x}"),
        };
        w.cpu.bank = (next >> 16) as u8;
        w.cpu.pc = next as u16;
        self.tail = match next {
            0x068369 => Some(FollowerTail::Returned),
            0x09a59e => Some(FollowerTail::Trigger),
            0x09a02b => Some(FollowerTail::NoTimedMessage),
            0x0ffdaa => {
                let index = self.message_index;
                Some(FollowerTail::Message { index })
            }
            _ => None,
        };
        Some(w.cycles)
    }
}

#[cfg(test)]
pub(crate) fn seed_case(r: &mut [u8], case: usize) {
    let variant = case / 16;
    r[0xf3cc] = if case < 8192 {
        (case % 16) as u8
    } else {
        (case % 256) as u8
    };
    r[0x11] = (variant / 256 % 2) as u8;
    r[0x8a] = (variant / 128 % 2 * 64) as u8;
    let timer = [0u16, 1, 2, 0x7fff, 0x8000, 0x8001, 0xfffe, 0xffff][variant % 8];
    r[0x2cd..0x2cf].copy_from_slice(&timer.to_le_bytes());
    r[0x5d] = 0;
    r[0x3a] = 0;
    for a in [0x377, 0x301, 0x37a, 0x2ec, 0x314, 0x308, 0x376] {
        r[a] = 0;
    }
    match variant / 8 % 16 {
        1 => r[0x3a] = 0x80,
        k @ 2..=8 => r[[0x377, 0x301, 0x37a, 0x2ec, 0x314, 0x308, 0x376][k - 2]] = 1,
        9 => r[0x5d] = 1,
        10 => r[0x5d] = 4,
        11 => r[0x5d] = 17,
        12 => r[0x5d] = 255,
        13 => r[0x3a] = 0x7f,
        14 => r[0x5d] = 8,
        _ => {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dungeon_follower_matches_source_dispatch_timers_validation_messages_and_wrapper_returns() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut count = 0;
        let mut tails = [0; 4];
        let mut messages = std::collections::BTreeSet::new();
        let mut visited = std::collections::BTreeSet::new();
        for case in 0..9216usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(13).wrapping_add(case as u8);
            }
            seed_case(&mut machine.ram, case);
            let mut cpu = NativeNmiContext {
                a: 0xab00,
                x: 3,
                y: 7,
                d: 0,
                db: 6,
                bank: 6,
                pc: 0x8365,
                sp: 0x1f0,
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
                snes::CpuRasterPosition::new(55, 1280),
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
            let mut work = DungeonFollower::default();
            while work.tail().is_none() {
                let pc = cpu.pc;
                visited.insert((u32::from(cpu.bank) << 16) | u32::from(pc));
                super::super::native_work::compare_instruction(
                    &format!("follower caller case={case} pc={pc:04x}"),
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
                FollowerTail::Returned => tails[0] += 1,
                FollowerTail::Trigger => tails[1] += 1,
                FollowerTail::NoTimedMessage => tails[2] += 1,
                FollowerTail::Message { index } => {
                    tails[3] += 1;
                    messages.insert(index);
                }
            }
            assert_eq!(&ram[0x1f5..0x1fd], &source.snes().ram[0x1f5..0x1fd]);
        }
        for pc in [
            0x068365, 0x099f91, 0x099fc4, 0x099fcf, 0x099fd8, 0x099ff3, 0x099ffa, 0x07f46f,
            0x07f498, 0x07f49a, 0x09a000, 0x09a008, 0x09a01f, 0x09a02a, 0x099f98,
        ] {
            assert!(visited.contains(&pc), "{pc:06x}");
        }
        assert!(tails.iter().all(|n| *n > 0));
        assert_eq!(messages.len(), 3);
        println!("native dungeon follower: 9216 cases, {count} source instructions, {} PCs, tails={tails:?}, all three messages",visited.len());
    }
}
