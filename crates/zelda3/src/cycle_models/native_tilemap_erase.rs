//! Retained CPU owner of EraseTileMaps. Hardware DMA reads the live bus and
//! publishes every byte; its channel-end clock remains available to NMI.
use super::{
    intro_clear::Access, native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext, tilemap_erase::Pattern,
};
use snes::{CpuMasterTimeline, DmaState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeTilemapErase {
    pattern: Pattern,
    instruction: u8,
    dma_end: Option<u64>,
}
impl NativeTilemapErase {
    pub(crate) fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            instruction: 0,
            dma_end: None,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.instruction == 49
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        self.dma_end.take()
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.pattern {
            Pattern::Normal => 0x834b,
            Pattern::Triforce => 0x8333,
            Pattern::DungeonMap => 0x833f,
        }
    }
    fn pc(&self) -> u16 {
        let i = self.instruction;
        if i < 5 {
            return self.routine() as u16 + [0, 2, 5, 7, 10][usize::from(i)];
        }
        [
            0x8355, 0x8357, 0x835a, 0x835d, 0x8360, 0x8363, 0x8366, 0x8369, 0x836c, 0x836f, 0x8372,
            0x8374, 0x8377, 0x8379, 0x837c, 0x837f, 0x8382, 0x8385, 0x8388, 0x838b, 0x838e, 0x8391,
            0x8393, 0x8395, 0x8398, 0x839b, 0x839e, 0x83a1, 0x83a4, 0x83a7, 0x83aa, 0x83ad, 0x83b0,
            0x83b3, 0x83b6, 0x83b9, 0x83bc, 0x83bf, 0x83c2, 0x83c5, 0x83c8, 0x83cb, 0x83ce, 0x83d0,
        ][usize::from(i - 5)]
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        dma: &mut DmaState,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.db, cpu.p & 0x10),
            (0, self.pc(), 0, 0, 0x10)
        );
        self.dma_end = None;
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = self.instruction + 1;
        let mut io = None;
        match self.instruction {
            0 => w.width(0x20, true),
            1 => w.immediate(
                if self.pattern == Pattern::Triforce {
                    0xa9
                } else {
                    0x7f
                },
                2,
            ),
            2 => w.dp_store(2, w.cpu.a, 2),
            3 => {
                w.immediate(
                    match self.pattern {
                        Pattern::Normal => 0x1ec,
                        Pattern::Triforce => 0x7f,
                        Pattern::DungeonMap => 0x300,
                    },
                    2,
                );
                if self.pattern == Pattern::Normal {
                    next = 5;
                }
            }
            4 => {
                w.branch(true);
            }
            5 | 27 => w.dp_store(0, w.cpu.a, 2),
            6 | 28 => io = Some((0x2115, 0, 2)),
            7 | 19 => io = Some((0x2116, 0, 2)),
            8 | 31 => w.immediate(0x1808, 2),
            9 | 22 | 32 | 43 => io = Some((0x4310, w.cpu.a, 2)),
            10 => io = Some((0x4314, 0, 2)),
            11 | 33 => w.immediate(0, 2),
            12 | 24 | 34 | 45 => io = Some((0x4312, w.cpu.a, 2)),
            13 => w.immediate(0x2000, 2),
            14 | 20 | 36 | 39 => io = Some((0x4315, w.cpu.a, 2)),
            15 => w.item_y(2, 1),
            16 | 25 | 37 | 46 => {
                w.drain(16);
                w.bus.access(
                    w.clock.clock_master_cycles(),
                    Access::Write(0x420b, w.cpu.y & 255, 1),
                );
                let start = w.clock.clock_master_cycles();
                let bus = std::cell::RefCell::new(&mut w.bus);
                self.dma_end = dma.run_synchronous_forward_ppu_with_timed_events(
                    w.clock,
                    w.cpu.y as u8,
                    |at, _, source, register| {
                        let mut bus = bus.borrow_mut();
                        let value = bus.access(at, Access::Read(source, 1));
                        bus.access(at, Access::Write(0x2100 + u32::from(register), value, 1));
                    },
                    |event, at| bus.borrow_mut().timeline_event(event, at),
                );
                w.cycles += w.clock.clock_master_cycles() - start;
            }
            17 => w.item_x(0x80, 1),
            18 | 38 => io = Some((0x2115, w.cpu.x, 1)),
            21 | 42 => w.immediate(0x1908, 2),
            23 | 44 => w.immediate(1, 2),
            26 => {
                let value = w.dp_read(2, 2);
                w.load(value, 2);
            }
            29 | 40 => w.immediate(0x6000, 2),
            30 | 41 => io = Some((0x2116, w.cpu.a, 2)),
            35 => w.immediate(0x800, 2),
            47 => w.width(0x20, false),
            48 => w.rtl(),
            _ => unreachable!(),
        }
        if let Some((address, value, width)) = io {
            w.drain(16);
            w.write_io(address, value, width);
            if address >= 0x4300 {
                for byte in 0..width {
                    dma.write_register(address + u16::from(byte), (value >> (byte * 8)) as u8);
                }
            }
        }
        self.instruction = next;
        if !self.complete() {
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_bus_tests::ApuFixture;
    use snes::{Snes9xColdCpuExecutor, SourceCpuBusAccessKind as Kind};
    #[test]
    fn retained_tilemap_cpu_and_live_dma_match_source_at_video_boundaries() {
        let Ok(mut rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        rom[..9].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x80, 0x8d, 0, 0x21, 0x80, 0xfe]);
        let (mut cases, mut scanlines, mut partial_presentations) = (0, 0, 0);
        for pattern in [Pattern::Normal, Pattern::Triforce, Pattern::DungeonMap] {
            for field in [0u64, 1] {
                for (v, h) in [(101, 850), (220, 900), (224, 1338), (261, 1350)] {
                    let mut source = Snes9xColdCpuExecutor::from_lorom_reset(&rom).unwrap();
                    let wanted = field * 357366 + v * 1364 + h;
                    let mut apu = ApuFixture::new();
                    apu.warm_to(&mut source, wanted);
                    if source.machine().timestamp().master_cycles() >= 225 * 1364 {
                        assert_eq!(
                            source
                                .run_until_main_loop_return()
                                .unwrap()
                                .instruction_count,
                            0
                        );
                    }
                    let mut work = NativeTilemapErase::new(pattern);
                    let mut native = NativeNmiContext {
                        a: 0xab08,
                        x: 0xfc,
                        y: 0xff,
                        d: 0,
                        db: 0,
                        bank: 0,
                        pc: work.pc(),
                        sp: 0x1fc,
                        p: 0xb1,
                    };
                    let mut registers = source.machine().snes().cpu.clone();
                    registers.a = native.a;
                    registers.x = native.x;
                    registers.y = native.y;
                    registers.pc = native.pc;
                    registers.sp = native.sp;
                    registers.dp = 0;
                    registers.db = 0;
                    registers.k = 0;
                    registers.e = false;
                    registers.unpack_flags(native.p);
                    let mut ram = source.machine().snes().ram.clone();
                    for (i, b) in ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 7)) as u8;
                    }
                    ram[0x1fd..0x200].copy_from_slice(&[0, 0xc2, 12]);
                    let at = source.machine().timestamp();
                    let sram = source.machine().snes().cart.ram.clone();
                    source
                        .rebase_private_timing_state_at(at, &registers, &ram, &sram)
                        .unwrap();
                    let mut clock = source.machine().timeline().clone();
                    let mut ppu = source.machine().snes().ppu.clone();
                    let mut dma = source.machine().snes().dma.clone();
                    let mut dma_count = 0;
                    while !work.complete() {
                        let pc = native.pc;
                        let label = format!("{pattern:?} field{field} v{v} h{h} pc{pc:04x}");
                        let is_dma = matches!(work.instruction, 16 | 25 | 37 | 46);
                        let expected: Vec<_> = source
                            .step()
                            .unwrap()
                            .accesses
                            .into_iter()
                            .filter_map(|a| match a.kind {
                                Kind::Read { value, width } if a.address != u32::from(pc) + 1 => {
                                    Some((
                                        a.timestamp.master_cycles(),
                                        Access::Read(a.address, width),
                                        value,
                                    ))
                                }
                                Kind::Write { value, width } => Some((
                                    a.timestamp.master_cycles(),
                                    Access::Write(a.address, value, width),
                                    0,
                                )),
                                _ => None,
                            })
                            .collect();
                        let mut actual = Vec::new();
                        let mut vblanks = Vec::new();
                        let expected_bytes = usize::from(dma.channel[1].size);
                        let mut dma_writes = 0;
                        work.step(&mut native, &mut clock, &mut dma, apu.bus(|at, access| {
                            let value = match access {
                                Access::Read(a, width) => {
                                    assert!(a < 0x2000);
                                    if width == 1 {
                                        u16::from(ram[a as usize])
                                    } else {
                                        u16::from_le_bytes([ram[a as usize], ram[a as usize + 1]])
                                    }
                                }
                                Access::Write(a, value, width) => {
                                    if a < 0x2000 {
                                        ram[a as usize..a as usize + usize::from(width)]
                                            .copy_from_slice(
                                                &value.to_le_bytes()[..usize::from(width)],
                                            );
                                    } else if (0x2100..0x2140).contains(&a) {
                                        assert_eq!(width, 1);
                                        ppu.write(a as u8, value as u8);
                                        if is_dma { dma_writes += 1; }
                                    } else {
                                        assert!(a == 0x420b || (0x4300..0x4380).contains(&a));
                                    }
                                    0
                                }
                                Access::Vblank { field, at } => {
                                    if is_dma {
                                        assert!(dma_writes > 0 && dma_writes < expected_bytes,
                                            "{label}: video must observe partially erased VRAM");
                                        partial_presentations += 1;
                                    }
                                    vblanks.push((field, at));
                                    return 0;
                                }
                            };
                            if !is_dma || matches!(access, Access::Write(0x420b, _, _)) {
                                actual.push((at, access, value));
                            }
                            value
                        }))
                        .unwrap();
                        apu.assert_matches(&source, &label);
                        if is_dma { assert_eq!(dma_writes, expected_bytes); }
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
                            assert_eq!(vblanks.len(), 1, "{label}");
                            let host = source.run_until_main_loop_return().unwrap();
                            assert_eq!(
                                host.instruction_count, 0,
                                "video event missing at DMA boundary {label}"
                            );
                            assert_eq!(
                                host.vblank_event_at.master_cycles(),
                                vblanks[0].1,
                                "{label}"
                            );
                        }
                        assert_eq!(ram, source.machine().snes().ram, "{label}");
                        assert_eq!(ppu.vram, source.machine().snes().ppu.vram, "{label}");
                        assert_eq!(
                            dma.save_c_saveload(),
                            source.machine().snes().dma.save_c_saveload(),
                            "{label}"
                        );
                        if is_dma {
                            let end = work.take_dma_end().unwrap();
                            assert!(end < clock.clock_master_cycles());
                            dma_count += 1;
                        }
                        work = work.clone();
                    }
                    assert_eq!(dma_count, 4);
                    assert_eq!((native.bank, native.pc, native.sp), (12, 0xc201, 0x1ff));
                    cases += 1;
                    scanlines += apu.lines;
                }
            }
        }
        assert!(partial_presentations > 0);
        eprintln!("{cases} tilemap erase fixtures retained independent APU hardware across {scanlines} scanlines and {partial_presentations} partial DMA presentations");
    }
}
