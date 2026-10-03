//! Animated dungeon sheet caller and four-buffer rotation. Loaded words stay
//! in CPU/stack ownership through the intervening stores and interrupts.
use super::{
    item_graphics::ItemConversion,
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Decompressing { next: u8 },
    Converting { next: u8 },
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnimatedDungeonGraphics {
    phase: Phase,
    decompression: Option<NativeDecompression>,
    conversion: Option<ItemConversion>,
}
impl Default for AnimatedDungeonGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            decompression: None,
            conversion: None,
        }
    }
}
impl AnimatedDungeonGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Decompressing { .. } => self.decompression.as_ref().unwrap().routine(),
            Phase::Converting { .. } => self.conversion.as_ref().unwrap().routine(),
            _ => 0xd337,
        }
    }
    fn caller_pc(i: u8) -> u16 {
        [
            0xd337, 0xd338, 0xd339, 0xd33a, 0xd33d, 0xd33f, 0xd341, 0xd344, 0xd347, 0xd34a, 0xd34c,
            0xd34e, 0xd351, 0xd353, 0xd355, 0xd358, 0xd35b, 0xd35e, 0xd361, 0xd365, 0xd366, 0xd36a,
            0xd36e, 0xd372, 0xd376, 0xd37a, 0xd37e, 0xd37f, 0xd383, 0xd384, 0xd385, 0xd388, 0xd38a,
            0xd38d, 0xd390, 0xd392, 0xd393,
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
            Decompressing { next } => {
                let child = self.decompression.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.decompression = None;
                    self.phase = Caller(next);
                    assert_eq!(cpu.pc, Self::caller_pc(next));
                }
                return cycles;
            }
            Converting { next } => {
                let child = self.conversion.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.conversion = None;
                    self.phase = Caller(next);
                    assert_eq!(cpu.pc, Self::caller_pc(next));
                }
                return cycles;
            }
            _ => {}
        }
        let Caller(i) = self.phase else {
            unreachable!()
        };
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, Self::caller_pc(i), 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = i + 1;
        match i {
            0 | 1 => {
                w.drain(6);
                w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
            }
            2 | 35 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
            }
            3 | 11 => {
                let child = NativeDecompression::new(false);
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.decompression = Some(child);
                self.phase = Decompressing { next };
                return Some(w.cycles);
            }
            4 | 12 => w.width(0x30, true),
            5 | 13 => {
                let v = w.dp_read(0, 2);
                w.load(v, 2);
            }
            6 | 14 => w.item_y(48, 2),
            7 | 15 => w.item_x(if i == 7 { 0x1680 } else { 0x1c80 }, 2),
            8 | 16 => {
                let child = ItemConversion::new(false);
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.conversion = Some(child);
                self.phase = Converting { next };
                return Some(w.cycles);
            }
            9 | 34 => w.width(0x30, false),
            10 => w.item_y(92, 1),
            17 => w.item_x(0, 2),
            18 | 20 | 22 | 24 => {
                w.drain(24);
                let base = match i {
                    18 => 0x7ea880,
                    20 => 0x7eac80,
                    22 => 0x7eae80,
                    _ => 0x7eaa80,
                };
                let value = w.read(base + u32::from(w.cpu.x), 2);
                w.load(value, 2);
            }
            19 => {
                w.drain(6);
                w.push(w.cpu.a, 2);
            }
            21 | 23 | 25 | 27 => {
                w.drain(24);
                let base = match i {
                    21 => 0x7ea880,
                    23 => 0x7eac80,
                    25 => 0x7eae80,
                    _ => 0x7eaa80,
                };
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
            }
            26 => {
                w.drain(12);
                let value = w.pull(2);
                w.load(value, 2);
            }
            28 | 29 => w.increment_x(),
            30 => w.compare_x_immediate(0x200, 2),
            31 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    next = 18;
                }
            }
            32 => w.immediate(0x3b00, 2),
            33 => {
                w.drain(16);
                w.write(0x134, w.cpu.a, 2);
            }
            36 => {
                w.rtl();
                self.phase = Complete;
                return Some(w.cycles);
            }
            _ => unreachable!(),
        }
        self.phase = Caller(next);
        w.cpu.pc = Self::caller_pc(next);
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
    fn retained_animated_dungeon_graphics_matches_sheet_calls_and_live_rotation() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for sheet in [0, 5, 15, 92, 93, 113] {
            for db in [0, 12] {
                for odd in [false, true] {
                    for (v, h) in [(101, 850), (224, 1338), (261, 1350)] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 7)) as u8;
                        }
                        machine.ram[0x1fd..0x200].copy_from_slice(&[0x27, 0xc2, 12]);
                        let mut work = AnimatedDungeonGraphics::default();
                        let mut native = NativeNmiContext {
                            a: 0xab08,
                            x: 0xfc,
                            y: sheet,
                            d: 0,
                            db,
                            bank: 0,
                            pc: 0xd337,
                            sp: 0x1fc,
                            p: 0xb0,
                        };
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
                        machine.cpu.db = db;
                        machine.cpu.pc = native.pc;
                        machine.cpu.sp = native.sp;
                        machine.cpu.e = false;
                        machine.cpu.unpack_flags(native.p);
                        let mut ram = machine.ram.clone();
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
                                "sheet={sheet} db={db} phase={:?} pc={:04x}",
                                work.phase, native.pc
                            );
                            super::super::native_work::compare_instruction(
                                &label,
                                &mut native,
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
                            assert!(count < 500000);
                        }
                        assert_eq!(
                            (native.bank, native.pc, native.sp, native.db),
                            (12, 0xc228, 0x1ff, db)
                        );
                        assert_eq!(ram, source.snes().ram);
                    }
                }
            }
        }
    }
}
