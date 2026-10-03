//! The common Module05 initialization prefix, ending at its live destination
//! decision at 02:8192. Completion preserves the caller's stack and CPU;
//! it does not pretend the containing module has returned.
#[cfg(test)]
use super::intro_clear::Access;
use super::{
    default_graphics::DefaultGraphics, follower_graphics::FollowerGraphics,
    gear_graphics::GearGraphics, load_file_state::LoadFileState,
    native_tilemap_erase::NativeTilemapErase, native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext,
    tilemap_erase::Pattern,
};
use snes::{CpuMasterTimeline, DmaState};
#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Erase(NativeTilemapErase),
    Default(DefaultGraphics),
    State(LoadFileState),
    Gear(GearGraphics),
    Follower(FollowerGraphics),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LoadFileCommon {
    pc: u32,
    child: Option<Child>,
    complete: bool,
}
impl Default for LoadFileCommon {
    fn default() -> Self {
        Self {
            pc: 0x028136,
            child: None,
            complete: false,
        }
    }
}
impl LoadFileCommon {
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Erase(w)) => w.routine(),
            Some(Child::Default(w)) => w.routine(),
            Some(Child::State(w)) => w.routine(),
            Some(Child::Gear(w)) => w.routine(),
            Some(Child::Follower(w)) => w.routine(),
            _ => 0x028136,
        }
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        match &mut self.child {
            Some(Child::Erase(w)) => w.take_dma_end(),
            _ => None,
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        if let Some(child) = &mut self.child {
            let (c, done) = match child {
                Child::Erase(w) => {
                    let c = w.step(cpu, clock, dma, bus);
                    (c, w.complete())
                }
                Child::Default(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::State(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Gear(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
                Child::Follower(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if done {
                self.child = None;
                assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
            }
            return c;
        }
        assert_eq!(
            ((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), cpu.d),
            (self.pc, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0x028136 | 0x02814f | 0x028158 | 0x02815c | 0x028160 | 0x028164 | 0x028168
            | 0x02816c | 0x028170 => {
                let target = match self.pc {
                    0x028136 => 0x00893d,
                    0x02814f => 0x00834b,
                    0x028158 => 0x00e2d0,
                    0x02815c => 0x00fc41,
                    0x028160 => 0x0e97d9,
                    0x028164 => 0x00d2c8,
                    0x028168 => 0x00d308,
                    0x02816c => 0x07f13c,
                    _ => 0x00d423,
                };
                w.operand_then_internal(3);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(self.pc as u16 + 3, 2);
                w.cpu.bank = (target >> 16) as u8;
                w.cpu.pc = target as u16;
                self.child = match target {
                    0x00893d => None,
                    0x00834b => Some(Child::Erase(NativeTilemapErase::new(Pattern::Normal))),
                    0x00e2d0 => Some(Child::Default(Default::default())),
                    0x00fc41 | 0x0e97d9 | 0x07f13c => {
                        Some(Child::State(LoadFileState::new(target)))
                    }
                    0x00d2c8 | 0x00d308 => Some(Child::Gear(GearGraphics::new(target == 0x00d2c8))),
                    _ => Some(Child::Follower(Default::default())),
                };
                if target == 0x00893d {
                    target
                } else {
                    self.pc + 4
                }
            }
            0x00893d | 0x028153 | 0x028174 | 0x028189 | 0x02818e => {
                w.immediate(
                    match self.pc {
                        0x00893d => 0x80,
                        0x028174 => 0x46,
                        0x02818e => 0x30,
                        _ => 2,
                    },
                    1,
                );
                self.pc + 2
            }
            0x00893f | 0x008944 | 0x028155 => {
                w.drain(16);
                w.write_io(
                    match self.pc {
                        0x00893f => 0x2100,
                        0x008944 => 0x420c,
                        _ => 0x2101,
                    },
                    if self.pc == 0x008944 {
                        0
                    } else {
                        w.cpu.a & 255
                    },
                    1,
                );
                self.pc + 3
            }
            0x008942 | 0x008947 | 0x028190 => {
                w.dp_store(
                    match self.pc {
                        0x008942 => 0x13,
                        0x008947 => 0x9b,
                        _ => 0xff,
                    },
                    if self.pc == 0x008947 {
                        0
                    } else {
                        w.cpu.a & 255
                    },
                    1,
                );
                self.pc + 2
            }
            0x008949 => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x02813a | 0x02813d | 0x028140 | 0x028143 | 0x028146 | 0x028149 | 0x02814c
            | 0x028186 | 0x02818b => {
                let a = match self.pc {
                    0x02813a => 0x200,
                    0x02813d => 0x3f4,
                    0x028140 => 0x2d4,
                    0x028143 => 0x2d7,
                    0x028146 => 0x2f9,
                    0x028149 => 0x379,
                    0x02814c => 0x3fd,
                    0x028186 => 0x2cd,
                    _ => 0x2ce,
                };
                w.drain(16);
                w.write(
                    w.absolute(a),
                    if self.pc == 0x02818b {
                        w.cpu.a & 255
                    } else {
                        0
                    },
                    1,
                );
                self.pc + 3
            }
            0x028176 | 0x02817a | 0x02817e | 0x028182 => {
                w.drain(24);
                w.write(0x7ec2fc + (self.pc - 0x028176) / 4, w.cpu.a & 255, 1);
                self.pc + 4
            }
            _ => unreachable!("common load continuation {:06x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc as u16;
            w.cpu.bank = (self.pc >> 16) as u8;
        }
        self.complete = self.pc == 0x028192;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_bus_tests::ApuFixture;
    use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
    #[test]
    fn common_load_prefix_matches_source_gear_properties_resets_and_dma() {
        let mut rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // Isolated routine fixture: warm the devices with interrupts disabled,
        // then give both executions the same input state at the source entry.
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let normalize = |a: u32| {
            if a & 0x400000 == 0 && a & 65535 < 0x6000 {
                a & 65535
            } else {
                a
            }
        };
        let mut instructions = 0;
        let mut scanlines = 0;
        for case in 0..20 {
            let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
            let wanted = if case & 1 == 0 {
                224 * 1364 + 1338
            } else {
                261 * 1364 + 1350
            };
            let mut apu = ApuFixture::new();
            apu.warm_to(&mut source, wanted);
            let mut native = NativeNmiContext {
                a: 0xab05,
                x: 0x71,
                y: 0x83,
                d: 0,
                db: 0,
                bank: 2,
                pc: 0x8136,
                sp: 0x1fa,
                p: 0x31,
            };
            let mut registers = source.machine().snes().cpu.clone();
            registers.a = native.a;
            registers.x = native.x;
            registers.y = native.y;
            registers.dp = 0;
            registers.db = 0;
            registers.k = 2;
            registers.pc = native.pc;
            registers.sp = native.sp;
            registers.e = false;
            registers.unpack_flags(native.p);
            let mut ram = source.machine().snes().ram.clone();
            for (i, b) in ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7) ^ case) as u8;
            }
            ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
            ram[0xf359] = (case / 4) as u8;
            ram[0xf35a] = (case % 4) as u8;
            ram[0xf35b] = (case % 3) as u8;
            ram[0xf3c5] = (case % 5) as u8;
            ram[0xf3cc] = (case % 14) as u8;
            let sram = source.machine().snes().cart.ram.clone();
            source
                .rebase_private_timing_state_at(
                    source.machine().timestamp(),
                    &registers,
                    &ram,
                    &sram,
                )
                .unwrap();
            let mut clock = source.machine().timeline().clone();
            let mut ppu = source.machine().snes().ppu.clone();
            let mut dma = source.machine().snes().dma.clone();
            let mut work = LoadFileCommon::default();
            let mut dma_count = 0;
            let mut count = 0;
            while !work.complete() {
                let origin = (u32::from(native.bank) << 16) | u32::from(native.pc);
                let receipt = source.step().unwrap();
                assert!(receipt.accepted_interrupt.is_none());
                let expected: Vec<_> = receipt
                    .accesses
                    .into_iter()
                    .filter_map(|a| match a.kind {
                        Kind::Read { value, width } if a.address != origin + 1 => Some((
                            a.timestamp.master_cycles(),
                            Access::Read(normalize(a.address), width),
                            value,
                        )),
                        Kind::Write { value, width } => Some((
                            a.timestamp.master_cycles(),
                            Access::Write(normalize(a.address), value, width),
                            0,
                        )),
                        _ => None,
                    })
                    .collect();
                let is_dma = expected
                    .iter()
                    .any(|(_, a, _)| matches!(a, Access::Write(0x420b, _, _)));
                let mut actual = Vec::new();
                work.step(&mut native, &mut clock, &mut dma, apu.bus(|at, access| {
                    let read = |a: u32| {
                        let a = normalize(a);
                        if a < 0x2000 {
                            ram[a as usize]
                        } else if (0x7e0000..0x800000).contains(&a) {
                            ram[(a - 0x7e0000) as usize]
                        } else {
                            assert_ne!(a & 0x8000, 0, "load fixture read {a:06x}");
                            rom[((a >> 16) as usize & 127) * 0x8000 + (a as usize & 0x7fff)]
                        }
                    };
                    let (access, value) = match access {
                        Access::Read(a, width) => {
                            let v = if width == 1 {
                                u16::from(read(a))
                            } else {
                                u16::from_le_bytes([read(a), read(a + 1)])
                            };
                            (Access::Read(normalize(a), width), v)
                        }
                        Access::Write(a, value, width) => {
                            let a = normalize(a);
                            if a < 0x2000 || (0x7e0000..0x800000).contains(&a) {
                                let i = if a < 0x2000 {
                                    a as usize
                                } else {
                                    (a - 0x7e0000) as usize
                                };
                                ram[i..i + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            } else if (0x2100..0x2140).contains(&a) {
                                assert_eq!(width, 1);
                                ppu.write(a as u8, value as u8);
                            } else {
                                assert!(
                                    matches!(a, 0x420b | 0x420c | 0x4300..=0x437f),
                                    "load fixture store {a:06x}"
                                );
                            }
                            (Access::Write(a, value, width), 0)
                        }
                        Access::Vblank { .. } => return 0,
                    };
                    if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                        actual.push((at, access, value));
                    }
                    value
                }))
                .unwrap();
                let label = format!("load prefix case{case} pc{origin:06x} owner={:06x}", work.routine());
                apu.assert_matches(&source, &label);
                assert_eq!(actual, expected, "{label}");
                let c = &source.machine().snes().cpu;
                assert_eq!(
                    native,
                    NativeNmiContext {
                        a: c.a,
                        x: c.x,
                        y: c.y,
                        d: c.dp,
                        db: c.db,
                        bank: c.k,
                        pc: c.pc,
                        sp: c.sp,
                        p: c.pack_flags()
                    },
                    "{label}"
                );
                assert_eq!(clock, *source.machine().timeline(), "{label}");
                assert_eq!(ram, source.machine().snes().ram, "{label}");
                assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "{label}");
                assert_eq!(
                    dma.save_c_saveload(),
                    source.machine().snes().dma.save_c_saveload(),
                    "{label}"
                );
                if is_dma {
                    assert!(work.take_dma_end().unwrap() < clock.clock_master_cycles());
                    dma_count += 1;
                }
                work = work.clone();
                count += 1;
                assert!(count < 500000, "{label}");
            }
            assert_eq!((native.bank, native.pc, native.sp), (2, 0x8192, 0x1fa));
            assert_eq!(&ram[0x1fb..0x1fe], &[0x59, 0x80, 0]);
            assert_eq!(dma_count, 4);
            assert_eq!(&ram[0xc2fc..0xc300], &[0x46; 4]);
            assert_eq!(ram[0xff], 0x30);
            assert_eq!(ram[0x2ce], 2);
            instructions += count;
            scanlines += apu.lines;
        }
        eprintln!("20 common load fixtures matched {instructions} instructions through 02:8192 with 80 DMAs");
        eprintln!("20 common load fixtures retained independent APU hardware across {scanlines} scanlines");
    }
}
