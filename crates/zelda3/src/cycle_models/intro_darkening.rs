//! Intro darkening and zero-brightness initialization. DMA and graphics calls
//! retain their own progress until the shared module return.
use super::{
    animated_dungeon_graphics::AnimatedDungeonGraphics,
    default_graphics::DefaultGraphics,
    intro_clear::Access,
    native_tilemap_erase::NativeTilemapErase,
    native_tilesets::NativeTilesets,
    native_work::Work,
    nmi_stack::{NativeNmiContext, ZERO},
    tilemap_erase::Pattern,
};
use snes::{CpuMasterTimeline, DmaState};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    ForceBlank(u8),
    Erase,
    DefaultGraphics,
    Tilesets,
    Animated,
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
    #[test]
    fn retained_darkening_matches_full_initializer_dma_and_live_graphics_callees() {
        let Ok(mut rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        // The fixture spins to a hardware-owned raster with NMI disabled.
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let normalize = |a: u32| {
            if a & 0x400000 == 0 && a & 65535 < 0x2000 {
                a & 65535
            } else {
                a
            }
        };
        for (brightness, wanted) in [
            (15, 101 * 1364 + 850),
            (2, 224 * 1364 + 1338),
            (1, 101 * 1364 + 850),
            (1, 357366 + 224 * 1364 + 1338),
        ] {
            let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
            while source.machine().timestamp().master_cycles() < wanted {
                source.step().unwrap();
            }
            if source.machine().timestamp().master_cycles() >= 225 * 1364 {
                assert_eq!(
                    source
                        .run_until_main_loop_return()
                        .unwrap()
                        .instruction_count,
                    0
                );
            }
            let mut native = NativeNmiContext {
                a: 0xab10,
                x: 0xfc,
                y: 0xff,
                d: 0,
                db: 0,
                bank: 12,
                pc: 0xc1f5,
                sp: 0x1fc,
                p: 0xb1,
            };
            let mut registers = source.machine().snes().cpu.clone();
            registers.a = native.a;
            registers.x = native.x;
            registers.y = native.y;
            registers.dp = 0;
            registers.db = 0;
            registers.k = 12;
            registers.pc = native.pc;
            registers.sp = native.sp;
            registers.e = false;
            registers.unpack_flags(native.p);
            let mut ram = source.machine().snes().ram.clone();
            for (i, b) in ram.iter_mut().enumerate() {
                *b = (i ^ (i >> 7)) as u8;
            }
            ram[0x10] = 0;
            ram[0x11] = 1;
            ram[0x13] = brightness;
            ram[0xc2fc..0xc300].copy_from_slice(&[12, 82, 90, 95]);
            ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
            let at = source.machine().timestamp();
            let sram = source.machine().snes().cart.ram.clone();
            source
                .rebase_private_timing_state_at(at, &registers, &ram, &sram)
                .unwrap();
            let mut clock = source.machine().timeline().clone();
            let mut ppu = source.machine().snes().ppu.clone();
            let mut dma = source.machine().snes().dma.clone();
            let mut work = IntroDarkening::default();
            let mut count = 0;
            while !work.complete() {
                let label = format!(
                    "brightness={brightness} phase={:?} pc={:02x}:{:04x}",
                    work.phase, native.bank, native.pc
                );
                let operand = (u32::from(native.bank) << 16) | u32::from(native.pc + 1);
                let mut first = true;
                let expected: Vec<_> = source
                    .step()
                    .unwrap()
                    .accesses
                    .into_iter()
                    .filter_map(|a| {
                        let access = match a.kind {
                            Kind::Read { value, width } => {
                                let skip = first && a.address == operand;
                                first = false;
                                if skip {
                                    return None;
                                }
                                (Access::Read(normalize(a.address), width), value)
                            }
                            Kind::Write { value, width } => {
                                (Access::Write(normalize(a.address), value, width), 0)
                            }
                            _ => return None,
                        };
                        Some((a.timestamp.master_cycles(), access.0, access.1))
                    })
                    .collect();
                let is_dma = expected
                    .iter()
                    .any(|(_, a, _)| matches!(a, Access::Write(0x420b, _, _)));
                let mut actual = Vec::new();
                let mut vblanks = Vec::new();
                work.step(&mut native, &mut clock, &mut dma, |at, access| {
                    let (access, value) = match access {
                        Access::Read(a, width) => {
                            let read = |a: u32| {
                                let a = normalize(a);
                                if a < 0x2000 {
                                    ram[a as usize]
                                } else if (0x7e0000..0x800000).contains(&a) {
                                    ram[a as usize - 0x7e0000]
                                } else {
                                    assert_ne!(a & 0x8000, 0);
                                    rom[((a >> 16) as usize & 127) * 0x8000 + (a as usize & 0x7fff)]
                                }
                            };
                            let value = if width == 1 {
                                u16::from(read(a))
                            } else {
                                u16::from_le_bytes([read(a), read(a + 1)])
                            };
                            (Access::Read(normalize(a), width), value)
                        }
                        Access::Write(a, value, width) => {
                            let a = normalize(a);
                            if (0x2100..0x2140).contains(&a) {
                                assert_eq!(width, 1);
                                ppu.write(a as u8, value as u8);
                            } else if matches!(a, 0x420b | 0x420c | 0x4300..=0x437f) {
                            } else {
                                let offset = if a >= 0x7e0000 {
                                    a as usize - 0x7e0000
                                } else {
                                    assert!(a < 0x2000);
                                    a as usize
                                };
                                ram[offset..offset + usize::from(width)]
                                    .copy_from_slice(&value.to_le_bytes()[..usize::from(width)]);
                            }
                            (Access::Write(a, value, width), 0)
                        }
                        Access::Vblank { field, at } => {
                            vblanks.push((field, at));
                            return 0;
                        }
                    };
                    if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                        actual.push((at, access, value));
                    }
                    value
                })
                .unwrap();
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
                if !vblanks.is_empty() {
                    assert_eq!(vblanks.len(), 1);
                    let host = source.run_until_main_loop_return().unwrap();
                    assert_eq!(host.instruction_count, 0, "{label}");
                    assert_eq!(
                        host.vblank_event_at.master_cycles(),
                        vblanks[0].1,
                        "{label}"
                    );
                }
                work = work.clone();
                count += 1;
                assert!(count < 1500000);
            }
            assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1ff));
            assert_eq!(ram, source.machine().snes().ram);
            assert_eq!(ppu.vram, source.machine().snes().ppu.vram);
            assert_eq!(
                dma.save_c_saveload(),
                source.machine().snes().dma.save_c_saveload()
            );
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroDarkening {
    phase: Phase,
    erase: Option<NativeTilemapErase>,
    default_graphics: Option<DefaultGraphics>,
    tilesets: Option<NativeTilesets>,
    animated: Option<AnimatedDungeonGraphics>,
}
impl Default for IntroDarkening {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            erase: None,
            default_graphics: None,
            tilesets: None,
            animated: None,
        }
    }
}
impl IntroDarkening {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        self.erase.as_mut().and_then(|w| w.take_dma_end())
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::ForceBlank(_) => 0x893d,
            Phase::Erase => self.erase.as_ref().unwrap().routine(),
            Phase::DefaultGraphics => self.default_graphics.as_ref().unwrap().routine(),
            Phase::Tilesets => self.tilesets.as_ref().unwrap().routine(),
            Phase::Animated => self.animated.as_ref().unwrap().routine(),
            _ => 0x0cc1f5,
        }
    }
    fn pc(i: u8) -> u16 {
        [
            0xc1f5, 0xc1f7, 0xc1f9, 0xc1fd, 0xc201, 0xc203, 0xc206, 0xc208, 0xc20b, 0xc20d, 0xc210,
            0xc212, 0xc215, 0xc217, 0xc21a, 0xc21e, 0xc222, 0xc224, 0xc228, 0xc22a, 0xc22e, 0xc230,
            0xc234, 0xc236, 0xc239, 0xc23c, 0xc23e, 0xc240, 0xc242, 0xc244, 0xc246, 0xc24a, 0xc24c,
            0xc250, 0xc252, 0xc256, 0xc259, 0xc25b,
        ][usize::from(i)]
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        use Phase::*;
        match self.phase {
            Complete => return None,
            Erase => {
                let w = self.erase.as_mut().unwrap();
                let cycles = w.step(cpu, clock, dma, bus);
                if w.complete() {
                    self.phase = Caller(4);
                }
                return cycles;
            }
            DefaultGraphics => {
                let w = self.default_graphics.as_mut().unwrap();
                let cycles = w.step(cpu, clock, bus);
                if w.complete() {
                    self.default_graphics = None;
                    self.phase = Caller(15);
                }
                return cycles;
            }
            Tilesets => {
                let w = self.tilesets.as_mut().unwrap();
                let cycles = w.step(cpu, clock, bus);
                if w.complete() {
                    self.tilesets = None;
                    self.phase = Caller(16);
                }
                return cycles;
            }
            Animated => {
                let w = self.animated.as_mut().unwrap();
                let cycles = w.step(cpu, clock, bus);
                if w.complete() {
                    self.animated = None;
                    self.phase = Caller(18);
                }
                return cycles;
            }
            _ => {}
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        if let ForceBlank(i) = self.phase {
            assert_eq!(
                (w.cpu.bank, w.cpu.pc),
                (
                    0,
                    [0x893d, 0x893f, 0x8942, 0x8944, 0x8947, 0x8949][usize::from(i)]
                )
            );
            w.fetch();
            match i {
                0 => w.immediate(0x80, 1),
                1 => {
                    w.drain(16);
                    w.write_io(0x2100, w.cpu.a & 255, 1);
                }
                2 => w.dp_store(0x13, w.cpu.a & 255, 1),
                3 => {
                    w.drain(16);
                    w.write_io(0x420c, 0, 1);
                    for channel in &mut dma.channel {
                        channel.hdma_active = false;
                    }
                }
                4 => w.dp_store(0x9b, 0, 1),
                5 => {
                    w.rtl();
                    self.phase = Caller(3);
                    return Some(w.cycles);
                }
                _ => unreachable!(),
            }
            self.phase = ForceBlank(i + 1);
            w.cpu.pc = [0x893d, 0x893f, 0x8942, 0x8944, 0x8947, 0x8949][usize::from(i + 1)];
            return Some(w.cycles);
        }
        let Caller(i) = self.phase else {
            unreachable!()
        };
        assert_eq!(
            (w.cpu.bank, w.cpu.pc, w.cpu.d, w.cpu.db, w.cpu.p & 0x30),
            (12, Self::pc(i), 0, 0, 0x30)
        );
        w.fetch();
        let mut next = i + 1;
        match i {
            0 => w.dp_increment(0x13, 1, false),
            1 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    next = 37;
                }
            }
            2 | 3 | 14 | 15 | 17 => {
                w.drain(24);
                w.drain(6);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                w.cpu.bank = 0;
                let (phase, pc) = match i {
                    2 => (ForceBlank(0), 0x893d),
                    3 => {
                        self.erase = Some(NativeTilemapErase::new(Pattern::Normal));
                        (Erase, 0x834b)
                    }
                    14 => {
                        self.default_graphics =
                            Some(super::default_graphics::DefaultGraphics::default());
                        (DefaultGraphics, 0xe2d0)
                    }
                    15 => {
                        self.tilesets = Some(NativeTilesets::default());
                        (Tilesets, 0xe19b)
                    }
                    _ => {
                        self.animated = Some(AnimatedDungeonGraphics::default());
                        (Animated, 0xd337)
                    }
                };
                self.phase = phase;
                w.cpu.pc = pc;
                return Some(w.cycles);
            }
            4 | 6 | 8 | 10 | 12 | 18 | 20 | 29 | 31 | 33 => w.immediate(
                match i {
                    4 | 18 | 29 => 2,
                    6 => 0x23,
                    8 => 0x7d,
                    10 => 0x51,
                    12 => 8,
                    31 => 0x1f,
                    _ => 0,
                },
                1,
            ),
            5 => {
                w.drain(16);
                w.write_io(0x2101, w.cpu.a & 255, 1);
            }
            7 | 9 | 11 | 13 => {
                w.drain(16);
                w.write(
                    match i {
                        7 => 0xaa1,
                        9 => 0xaa3,
                        11 => 0xaa2,
                        _ => 0xaa4,
                    },
                    w.cpu.a & 255,
                    1,
                );
            }
            16 => w.item_y(93, 1),
            19 | 21 | 30 | 32 | 34 => {
                w.drain(24);
                w.write(
                    match i {
                        19 => 0x7ec00d,
                        21 => 0x7ec00e,
                        30 => 0x7ec009,
                        32 => 0x7ec007,
                        _ => 0x7ec00b,
                    },
                    w.cpu.a & 255,
                    1,
                );
            }
            22 | 25 | 26 | 27 | 28 => w.dp_store(
                match i {
                    22 => 0x8a,
                    25 => 0xc8,
                    26 => 0xc9,
                    27 => 0xca,
                    _ => 0xcb,
                },
                0,
                1,
            ),
            23 | 24 | 35 => {
                w.drain(16);
                w.write(
                    match i {
                        23 => 0xab6,
                        24 => 0xab8,
                        _ => 0xaa6,
                    },
                    0,
                    1,
                );
            }
            36 => w.dp_increment(0x11, 1, true),
            37 => {
                w.rtl();
                self.phase = Complete;
                return Some(w.cycles);
            }
            _ => unreachable!(),
        }
        self.phase = Caller(next);
        w.cpu.pc = Self::pc(next);
        Some(w.cycles)
    }
}
