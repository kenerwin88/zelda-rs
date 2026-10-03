//! Shared sprite graphics upload. The module branch selects raw common planes
//! or nested compressed sprite sheets using the caller's live state.
use super::{
    native_tileset_sheet::{NativeTilesetSheet, SheetEntry},
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
    ppu_plane_conversion::{PlaneLoop, PpuPlaneConversion},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    High,
    Low,
    Sheet { tail: bool },
    Return,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommonSpriteGraphics {
    phase: Phase,
    conversion: Option<PpuPlaneConversion>,
    sheet: Option<NativeTilesetSheet>,
}
impl Default for CommonSpriteGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            conversion: None,
            sheet: None,
        }
    }
}
impl CommonSpriteGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::High | Phase::Low => self.conversion.as_ref().unwrap().routine(),
            Phase::Sheet { .. } => self.sheet.as_ref().unwrap().routine(),
            _ => 0xe6b7,
        }
    }
    fn pc(i: u8) -> u16 {
        [
            0xe6b7, 0xe6ba, 0xe6bd, 0xe6bf, 0xe6c2, 0xe6c4, 0xe6c7, 0xe6c9, 0xe6cb, 0xe726, 0xe728,
            0xe72a, 0xe72c, 0xe72e, 0xe731, 0xe733, 0xe736, 0xe738, 0xe73b, 0xe73d, 0xe73f, 0xe741,
            0xe744, 0xe746, 0xe748, 0xe74a, 0xe74c, 0xe74e, 0xe750, 0xe753, 0xe755, 0xe757, 0xe759,
        ][usize::from(i)]
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
            High | Low => {
                let child = self.conversion.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.conversion = None;
                    self.phase = if self.phase == High {
                        Caller(9)
                    } else {
                        Return
                    };
                }
                return cycles;
            }
            Sheet { tail } => {
                let child = self.sheet.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.sheet = None;
                    self.phase = if tail { Complete } else { Caller(29) };
                }
                return cycles;
            }
            Return => {
                assert_eq!(cpu.pc, 0xe6b6);
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.db), (0, Self::pc(i), 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = i + 1;
        match i {
            0 => {
                w.drain(16);
                w.cpu.y = w.read(0xaa4, 1);
                w.flags(w.cpu.y, 1);
            }
            1 | 3 | 5 | 13 | 15 | 17 => {
                let slot = if i < 9 { (i - 1) / 2 } else { (i - 13) / 2 };
                let base = [0xcff3u16, 0xd0d2, 0xd1b1][usize::from(slot)];
                w.drain(16);
                if (base & 255) + w.cpu.y > 255 {
                    w.drain(6);
                }
                let value = w.read(u32::from(base) + u32::from(w.cpu.y), 1);
                w.load(value, 1);
            }
            2 | 4 | 6 | 14 | 16 | 18 => {
                let slot = if i < 9 { (i - 2) / 2 } else { (i - 14) / 2 };
                w.dp_store(2 - u16::from(slot), w.cpu.a & 255, 1);
            }
            7 | 19 => w.width(0x20, true),
            8 | 20 => {
                w.item_y(if i == 8 { 64 } else { 127 }, 1);
                if i == 8 {
                    let child = PpuPlaneConversion::new(PlaneLoop::HighCommon);
                    w.cpu.pc = child.routine() as u16;
                    self.conversion = Some(child);
                    self.phase = High;
                    return Some(w.cycles);
                }
            }
            9 => {
                let v = w.dp_read(0x10, 1);
                w.load(v, 1);
            }
            10 => w.compare_immediate(1, 1),
            11 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    next = 22;
                }
            }
            12 => w.item_y(6, 1),
            21 => {
                w.drain(16);
                let child = PpuPlaneConversion::new(PlaneLoop::Low);
                w.cpu.pc = child.routine() as u16;
                self.conversion = Some(child);
                self.phase = Low;
                return Some(w.cycles);
            }
            22 => w.dp_store(15, 0, 1),
            23 | 29 => w.item_y(if i == 23 { 94 } else { 95 }, 1),
            24 | 30 => w.dp_store(6, w.cpu.y, 1),
            25 => w.immediate(0x7f, 1),
            26 => w.dp_store(2, w.cpu.a & 255, 1),
            27 | 31 => w.item_x(0x40, 1),
            28 | 32 => {
                if i == 28 {
                    w.item_call();
                } else {
                    w.drain(16);
                }
                let child = NativeTilesetSheet::new(SheetEntry::Sprite);
                w.cpu.pc = child.routine() as u16;
                self.sheet = Some(child);
                self.phase = Sheet { tail: i == 32 };
                return Some(w.cycles);
            }
            _ => unreachable!(),
        }
        self.phase = Caller(next);
        w.cpu.pc = Self::pc(next);
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
    fn retained_common_sprites_match_raw_pointer_wrap_and_nested_module_branch() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for sheet in 0..12 {
            for module in [0, 1, 7] {
                for (odd, v, h) in [(false, 101, 850), (true, 224, 1338), (false, 261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 7)) as u8;
                    }
                    machine.ram[0x10] = module;
                    machine.ram[0xaa4] = sheet;
                    machine.ram[0x1fe..0x200].copy_from_slice(&[0xad, 0xe1]);
                    machine.ppu.write(0x15, 0x80);
                    machine.ppu.write(0x16, 0);
                    machine.ppu.write(0x17, 0x44);
                    let mut work = CommonSpriteGraphics::default();
                    let mut native = NativeNmiContext {
                        a: 0xab44,
                        x: 0x78,
                        y: 0xff,
                        d: 0,
                        db: 0,
                        bank: 0,
                        pc: 0xe6b7,
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
                        let label = format!(
                            "common sheet={sheet} module={module} phase={:?} pc={:04x}",
                            work.phase, native.pc
                        );
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
                    assert_eq!((native.bank, native.pc, native.sp), (0, 0xe1ae, 0x1ff));
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!(ppu.vram, source.snes().ppu.vram);
                }
            }
        }
    }
}
