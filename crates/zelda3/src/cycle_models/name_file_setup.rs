//! Name-entry setup shares the retained menu erase and checkerboard owners.
//! SRAM clearing and all following control stores retire on the same CPU.
#[cfg(test)]
use super::intro_clear::Access;
use super::native_work::WorkBus;
use super::{
    file_select_erase::FileSelectErase,
    file_select_stripes::FileSelectStripes,
    intro_palettes::PaletteSource,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::{CpuMasterTimeline, DmaState};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Erase(FileSelectErase),
    Checkerboard(FileSelectStripes),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NameFileSetup {
    entry: u16,
    pc: u16,
    child: Option<Child>,
    complete: bool,
}
impl NameFileSetup {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xd89c | 0xd911 | 0xd928));
        Self {
            entry,
            pc: entry,
            child: None,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match &self.child {
            Some(Child::Erase(w)) => w.routine(),
            Some(Child::Checkerboard(w)) => w.routine(),
            None => 0x0c0000 | u32::from(self.entry),
        }
    }
    pub(crate) fn take_dma_end(&mut self) -> Option<u64> {
        match &mut self.child {
            Some(Child::Erase(w)) => w.take_dma_end(),
            _ => None,
        }
    }
    pub(crate) fn palette_source(&self) -> PaletteSource {
        match &self.child {
            Some(Child::Erase(w)) => w.palette_source(),
            _ => PaletteSource::Constant,
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
        if let Some(child) = self.child.as_mut() {
            let (c, done) = match child {
                Child::Erase(w) => {
                    let c = w.step(cpu, clock, dma, bus);
                    (c, w.complete())
                }
                Child::Checkerboard(w) => {
                    let c = w.step(cpu, clock, bus);
                    (c, w.complete())
                }
            };
            if done {
                self.child = None;
                assert_eq!((cpu.bank, cpu.pc), (12, self.pc));
            }
            return c;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (12, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.pc = match self.pc {
            0xd89c => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(0xd89f, 2);
                w.cpu.pc = 0xcdf9;
                self.child = Some(Child::Erase(FileSelectErase::erase_triforce()));
                0xd8a0
            }
            0xd8a0 | 0xd8b4 | 0xd922 | 0xd928 | 0xd92d => {
                w.immediate(
                    match self.pc {
                        0xd8b4 => 0x83,
                        0xd928 => 5,
                        0xd92d => 15,
                        _ => 1,
                    },
                    1,
                );
                self.pc + 2
            }
            0xd8a2 | 0xd8b6 => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0xd8a2 { 0x128 } else { 0xb11 }),
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 3
            }
            0xd8a5 | 0xd8a8 | 0xd8ab | 0xd8ae | 0xd8b1 | 0xd931 => {
                w.drain(16);
                let a = match self.pc {
                    0xd8a5 => 0xb10,
                    0xd8a8 => 0xb12,
                    0xd8ab => 0xb15,
                    0xd8ae => 0xca,
                    0xd8b1 => 0xcc,
                    _ => 0x710,
                };
                w.write(w.absolute(a), 0, 1);
                self.pc + 3
            }
            0xd8b9 | 0xd8f3 | 0xd90e | 0xd914 | 0xd919 | 0xd91f => {
                match self.pc {
                    0xd8f3 => w.immediate(0xa9, 2),
                    0xd919 => w.immediate(65535, 2),
                    _ => w.width(0x30, matches!(self.pc, 0xd8b9 | 0xd914)),
                };
                self.pc
                    + if matches!(self.pc, 0xd8f3 | 0xd919) {
                        3
                    } else {
                        2
                    }
            }
            0xd8bb => {
                w.immediate(0x1f0, 2);
                self.pc + 3
            }
            0xd8be | 0xd8cb => {
                w.drain(16);
                w.write(
                    w.absolute(if self.pc == 0xd8be { 0x630 } else { 0x200 }),
                    w.cpu.a,
                    2,
                );
                self.pc + 3
            }
            0xd8c1 => {
                w.dp_store(0xe4, 0, 2);
                self.pc + 2
            }
            0xd8c3 => {
                let v = w.dp_read(0xc8, 2);
                w.load(v, 2);
                self.pc + 2
            }
            0xd8c5 => {
                w.shift_left(2);
                self.pc + 1
            }
            0xd8c6 | 0xd8ce => {
                w.tax_word();
                self.pc + 1
            }
            0xd8c7 => {
                w.drain(24);
                let v = w.read(0x848c + u32::from(w.cpu.x), 2);
                w.load(v, 2);
                self.pc + 4
            }
            0xd8cf => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                self.pc + 3
            }
            0xd8d2 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xd8d3 | 0xd8d7 | 0xd8db | 0xd8df | 0xd8e3 | 0xd8f6 | 0xd8fa | 0xd8fe | 0xd902
            | 0xd906 | 0xd90a => {
                let a = if self.pc <= 0xd8e3 {
                    0x700000 + u32::from((self.pc - 0xd8d3) / 4) * 0x100
                } else {
                    0x7003d9 + u32::from((self.pc - 0xd8f6) / 4) * 2
                };
                w.drain(24);
                w.write(a + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0xd8e7 | 0xd8e8 => {
                w.increment_x();
                self.pc + 1
            }
            0xd8e9 | 0xd8ea => {
                w.increment_y();
                self.pc + 1
            }
            0xd8eb => {
                w.drain(16);
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(w.cpu.y >= 256);
                w.flags(w.cpu.y.wrapping_sub(256), 2);
                self.pc + 3
            }
            0xd8ee => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xd8d3
                } else {
                    self.pc + 2
                }
            }
            0xd8f0 => {
                w.drain(16);
                w.cpu.x = w.read(w.absolute(0x200), 2);
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xd911 | 0xd912 => {
                w.drain(6);
                w.push(
                    u16::from(if self.pc == 0xd911 {
                        w.cpu.db
                    } else {
                        w.cpu.bank
                    }),
                    1,
                );
                self.pc + 1
            }
            0xd913 | 0xd921 => {
                w.drain(12);
                let v = w.pull(1);
                w.cpu.db = v as u8;
                w.flags(v, 1);
                self.pc + 1
            }
            0xd916 => {
                w.drain(22);
                w.push(0xd918, 2);
                w.cpu.pc = 0xce1b;
                self.child = Some(Child::Checkerboard(FileSelectStripes::new(0xce1b)));
                0xd919
            }
            0xd91c => {
                w.drain(22);
                w.write(w.absolute(0x1006) + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 3
            }
            0xd924 | 0xd92a => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                0xc52e
            }
            0xc52e | 0xd92f => {
                w.dp_store(
                    if self.pc == 0xc52e { 0x14 } else { 0x13 },
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 2
            }
            0xc530 => {
                w.dp_increment(0x11, 1, true);
                self.pc + 2
            }
            0xc532 => {
                w.rts();
                w.cpu.pc
            }
            0xd910 | 0xd927 | 0xd934 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("name setup continuation {:04x}", self.pc),
        };
        if self.child.is_none() {
            w.cpu.pc = self.pc;
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
    fn naming_setup_matches_source_sram_clear_and_checkerboard_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        // D8A0 is the return from the shared erase/palette child, whose whole
        // call is covered by independently cold integration tests. Start here
        // with nonzero SRAM to expose every parent store and every slot bound.
        for (entry, pc) in [(0xd89c, 0xd8a0), (0xd911, 0xd911), (0xd928, 0xd928)] {
            for slot in 0..3 {
                for (v, h) in [(224, 1338), (261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 8)) as u8;
                    }
                    for (i, b) in machine.cart.ram.iter_mut().enumerate() {
                        *b = (i.wrapping_mul(7) ^ (i >> 8)) as u8;
                    }
                    machine.ram[0xc8..0xca].copy_from_slice(&(slot as u16).to_le_bytes());
                    machine.ram[0x11] = if entry == 0xd89c {
                        1
                    } else if entry == 0xd911 {
                        1
                    } else {
                        2
                    };
                    machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x59, 0x80, 0]);
                    let mut native = NativeNmiContext {
                        a: 0xab83,
                        x: 0x71,
                        y: 0x83,
                        d: 0,
                        db: 0,
                        bank: 12,
                        pc,
                        sp: 0x1fa,
                        p: 0x31,
                    };
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.k = 12;
                    machine.cpu.pc = native.pc;
                    machine.cpu.sp = native.sp;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(native.p);
                    let mut ram = machine.ram.clone();
                    let mut sram = machine.cart.ram.clone();
                    let before = sram.clone();
                    let mut clock = CpuMasterTimeline::at_raster(
                        0,
                        CpuRasterPosition::new(v, h),
                        CpuBusWorkload::default(),
                        CpuFieldTiming::non_interlace(v == 261),
                    );
                    let mut source = RomCpuTimingProbe::new(
                        machine,
                        clock.clone(),
                        SourcePpuReadState::snes9x_reset(),
                    )
                    .unwrap();
                    clock.begin_synchronous_timeline().unwrap();
                    let mut work = NameFileSetup::new(entry);
                    work.pc = pc;
                    let mut dma = DmaState::default();
                    let mut count = 0;
                    while !work.complete() {
                        let label =
                            format!("name setup entry{entry:04x} slot{slot} pc{:04x}", native.pc);
                        super::super::native_work::compare_instruction_with_devices(
                            &label,
                            &mut native,
                            &mut clock,
                            &mut source,
                            &mut ram,
                            &rom,
                            None,
                            |_, a| match a {
                                Access::Read(a, width) if (0x700000..0x702000).contains(&a) => {
                                    let i = (a - 0x700000) as usize;
                                    Some(if width == 1 {
                                        u16::from(sram[i])
                                    } else {
                                        u16::from_le_bytes([sram[i], sram[i + 1]])
                                    })
                                }
                                Access::Write(a, value, width)
                                    if (0x700000..0x702000).contains(&a) =>
                                {
                                    let i = (a - 0x700000) as usize;
                                    sram[i..i + usize::from(width)].copy_from_slice(
                                        &value.to_le_bytes()[..usize::from(width)],
                                    );
                                    Some(0)
                                }
                                _ => None,
                            },
                            |cpu, clock, bus| {
                                work.step(cpu, clock, &mut dma, bus).unwrap();
                            },
                        );
                        assert_eq!(ram, source.snes().ram, "{label}");
                        assert_eq!(sram, source.snes().cart.ram, "{label}");
                        work = work.clone();
                        count += 1;
                        assert!(count < 30000);
                    }
                    assert_eq!((native.bank, native.pc, native.sp), (0, 0x805a, 0x1fd));
                    if entry == 0xd89c {
                        assert_eq!(&sram[..slot * 0x500], &before[..slot * 0x500]);
                        assert_eq!(&sram[(slot + 1) * 0x500..], &before[(slot + 1) * 0x500..]);
                        for (i, &b) in sram[slot * 0x500..(slot + 1) * 0x500].iter().enumerate() {
                            assert_eq!(
                                b,
                                if (0x3d9..0x3e5).contains(&i) && i & 1 != 0 {
                                    0xa9
                                } else {
                                    0
                                }
                            );
                        }
                    }
                }
            }
        }
    }
}
