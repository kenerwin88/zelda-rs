//! Sprite/background sheet callers, including their live destination selection
//! and high/low plane branches. Each callee returns through the CPU stack.
use super::{
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
    ppu_plane_conversion::{PlaneLoop, PpuPlaneConversion},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SheetEntry {
    Sprite,
    Background,
    BackgroundDestination,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Decompressing,
    Converting,
    Return,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeTilesetSheet {
    entry: SheetEntry,
    phase: Phase,
    decompression: Option<NativeDecompression>,
    conversion: Option<PpuPlaneConversion>,
}
impl NativeTilesetSheet {
    pub(crate) fn new(entry: SheetEntry) -> Self {
        Self {
            entry,
            phase: Phase::Caller(if entry == SheetEntry::BackgroundDestination {
                2
            } else {
                0
            }),
            decompression: None,
            conversion: None,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Decompressing => self.decompression.as_ref().unwrap().routine(),
            Phase::Converting => self.conversion.as_ref().unwrap().routine(),
            _ => match self.entry {
                SheetEntry::Sprite => 0xe583,
                SheetEntry::Background => 0xe609,
                SheetEntry::BackgroundDestination => 0xe60d,
            },
        }
    }
    fn pc(&self, i: u8) -> u16 {
        if self.entry == SheetEntry::Sprite {
            match i {
                0..=6 => [0xe583, 0xe585, 0xe587, 0xe588, 0xe58b, 0xe58d, 0xe58f][usize::from(i)],
                7..=20 => 0xe590 + u16::from(i - 7) * 2,
                21 => 0xe5ac,
                _ => unreachable!(),
            }
        } else {
            [
                0xe609, 0xe60b, 0xe60d, 0xe60f, 0xe611, 0xe613, 0xe616, 0xe618, 0xe61a, 0xe61d,
                0xe61f, 0xe621, 0xe623, 0xe625, 0xe627, 0xe629, 0xe62b, 0xe62d, 0xe62f, 0xe631,
                0xe633, 0xe636, 0xe638, 0xe63a,
            ][usize::from(i)]
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        use Phase::*;
        match self.phase {
            Complete => return None,
            Decompressing => {
                let child = self.decompression.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.decompression = None;
                    self.phase = Caller(if self.entry == SheetEntry::Sprite {
                        4
                    } else {
                        6
                    });
                }
                return cycles;
            }
            Converting => {
                let child = self.conversion.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.phase = Return;
                }
                return cycles;
            }
            Return => {
                assert!(matches!(cpu.pc, 0xe608 | 0xe6b6));
                let mut w = Work {
                    cpu,
                    clock,
                    bus,
                    cycles: 0,
                };
                w.fetch();
                w.rts();
                self.phase = Complete;
                return Some(w.cycles);
            }
            _ => {}
        }
        let Caller(i) = self.phase else {
            unreachable!()
        };
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (0, self.pc(i), 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = i + 1;
        let mut convert = None;
        let decompress = if self.entry == SheetEntry::Sprite {
            match i {
                0 => w.dp_store(0, 0, 1),
                1 => w.dp_store(1, w.cpu.x, 1),
                2 => {
                    w.drain(6);
                    w.push(w.cpu.y, 1);
                }
                3 => {}
                4 => w.width(0x20, true),
                5 => w.item_y(63, 1),
                6 => {
                    w.drain(12);
                    w.cpu.x = w.pull(1);
                    w.flags(w.cpu.x, 1);
                }
                7..=20 => {
                    if i % 2 == 1 {
                        w.compare_x_immediate(
                            [82, 83, 90, 91, 92, 94, 95][usize::from((i - 7) / 2)],
                            1,
                        );
                    } else if w.branch(w.cpu.p & ZERO != 0) {
                        convert = Some(PlaneLoop::HighSheet);
                    }
                }
                21 => {
                    w.drain(16);
                    convert = Some(PlaneLoop::Low);
                }
                _ => unreachable!(),
            }
            i == 3
        } else {
            match i {
                0 => w.immediate(0x7f, 1),
                1 => w.item_x(0x40, 1),
                2 => w.dp_store(0, 0, 1),
                3 => w.dp_store(1, w.cpu.x, 1),
                4 => w.dp_store(2, w.cpu.a & 255, 1),
                5 => {}
                6 => w.width(0x20, true),
                7 => w.item_y(63, 1),
                8 => {
                    w.drain(16);
                    w.cpu.x = w.read(0xaa1, 1);
                    w.flags(w.cpu.x, 1);
                }
                9 => w.compare_x_immediate(32, 1),
                10 => {
                    if w.branch(w.cpu.p & CARRY == 0) {
                        next = 21;
                    }
                }
                11 | 21 => {
                    w.cpu.x = w.dp_read(0xf, 1);
                    w.flags(w.cpu.x, 1);
                }
                12 | 14 | 16 | 18 | 22 => w.compare_x_immediate(
                    match i {
                        12 => 7,
                        14 => 2,
                        16 | 22 => 4,
                        _ => 3,
                    },
                    1,
                ),
                13 | 15 | 17 => {
                    if w.branch(w.cpu.p & ZERO != 0) {
                        convert = Some(PlaneLoop::HighSheet);
                    }
                }
                19 => {
                    if w.branch(w.cpu.p & ZERO == 0) {
                        convert = Some(PlaneLoop::Low);
                    }
                }
                20 => {
                    w.drain(16);
                    convert = Some(PlaneLoop::HighSheet);
                }
                23 => {
                    if w.branch(w.cpu.p & CARRY != 0) {
                        next = 20;
                    } else {
                        convert = Some(PlaneLoop::Low);
                    }
                }
                _ => unreachable!(),
            }
            i == 5
        };
        if decompress {
            let child = NativeDecompression::table_entry(self.entry == SheetEntry::Sprite);
            w.item_call();
            w.cpu.pc = child.routine() as u16;
            self.decompression = Some(child);
            self.phase = Decompressing;
        } else if let Some(kind) = convert {
            let child = PpuPlaneConversion::new(kind);
            w.cpu.pc = child.routine() as u16;
            self.conversion = Some(child);
            self.phase = Converting;
        } else {
            self.phase = Caller(next);
            w.cpu.pc = self.pc(next);
        }
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn retained_tileset_sheets_match_destinations_plane_branches_and_ppu_writes() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        let cases = [
            (SheetEntry::Sprite, 81, 0, 0),
            (SheetEntry::Sprite, 82, 0, 0),
            (SheetEntry::Sprite, 83, 0, 0),
            (SheetEntry::Sprite, 90, 0, 0),
            (SheetEntry::Sprite, 91, 0, 0),
            (SheetEntry::Sprite, 92, 0, 0),
            (SheetEntry::Sprite, 94, 0, 0),
            (SheetEntry::Sprite, 95, 0, 0),
            (SheetEntry::Sprite, 107, 0, 0),
            (SheetEntry::Background, 0, 0, 3),
            (SheetEntry::Background, 5, 31, 4),
            (SheetEntry::BackgroundDestination, 15, 32, 7),
            (SheetEntry::BackgroundDestination, 92, 36, 2),
            (SheetEntry::BackgroundDestination, 93, 32, 4),
            (SheetEntry::BackgroundDestination, 113, 36, 3),
            (SheetEntry::BackgroundDestination, 93, 32, 1),
        ];
        for (entry, sheet, theme, slot) in cases {
            for odd in [false, true] {
                for (v, h) in [(101, 850), (224, 1338), (261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 7)) as u8;
                    }
                    machine.ram[2] = 0x7e;
                    machine.ram[0xf] = slot;
                    machine.ram[0xaa1] = theme;
                    machine.ram[0x1fe..0x200].copy_from_slice(&[0x43, 0xe2]);
                    machine.ppu.write(0x15, 0x80);
                    machine.ppu.write(0x16, 0);
                    machine.ppu.write(0x17, 0x20);
                    let mut work = NativeTilesetSheet::new(entry);
                    let mut native = NativeNmiContext {
                        a: 0xab7e,
                        x: 0x78,
                        y: sheet,
                        d: 0,
                        db: 0,
                        bank: 0,
                        pc: work.routine() as u16,
                        sp: 0x1fd,
                        p: 0xb0,
                    };
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.pc = native.pc;
                    machine.cpu.sp = native.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(native.p);
                    let mut ram = machine.ram.clone();
                    let mut ppu = machine.ppu.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::non_interlace(odd),
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut count = 0;
                    while !work.complete() {
                        let label=format!("{entry:?} sheet={sheet} theme={theme} slot={slot} phase={:?} pc={:04x}",work.phase,native.pc);
                        super::super::native_work::compare_instruction_with_ppu(
                            &label,
                            &mut native,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            Some(&mut ppu),
                            |cpu, clock, bus| {
                                work.step(cpu, clock, bus).unwrap();
                            },
                        );
                        work = work.clone();
                        count += 1;
                        assert!(count < 500000);
                    }
                    assert_eq!((native.bank, native.pc, native.sp), (0, 0xe244, 0x1ff));
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!(ppu.vram, source.snes().ppu.vram);
                }
            }
        }
    }
}
