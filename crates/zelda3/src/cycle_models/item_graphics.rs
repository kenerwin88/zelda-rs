//! Item graphics conversion with retained CPU, live scratch pointers and bus
//! publications. Phase transitions belong to this native routine; PC is only
//! a checked boundary, never an opcode or runtime dispatch input.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConvertPhase {
    Count,
    Source,
    CarrySource,
    AddSource,
    ThirdPointer,
    Rows,
    Pair,
    PairStore,
    PairSwap,
    PairMerge,
    PairMask,
    PairSave,
    SourceIncrement(u8),
    Third,
    ThirdMask,
    ThirdSave,
    ThirdMerge,
    ThirdSwap,
    ThirdMergeSaved,
    ThirdStore,
    ThirdIncrement,
    DestinationIncrement(u8),
    RowDecrement,
    RowBranch,
    Destination,
    CarryDestination,
    AddDestination,
    SaveDestination,
    StrideSource,
    StrideMask,
    StrideBranch,
    StrideReload,
    StrideCarry,
    StrideAdd,
    StrideSave,
    NextSource,
    CountDecrement,
    CountBranch,
    Return,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemConversion {
    phase: ConvertPhase,
    high: bool,
    second_row: bool,
}
impl ItemConversion {
    pub(crate) fn new(high: bool) -> Self {
        Self {
            phase: ConvertPhase::Count,
            high,
            second_row: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == ConvertPhase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.high {
            0xd61c
        } else {
            0xd5ce
        }
    }
    fn pc(&self) -> u16 {
        use ConvertPhase::*;
        if self.high {
            match self.phase {
                Count => 0xd61c,
                Source => 0xd61e,
                CarrySource => 0xd620,
                AddSource => 0xd621,
                ThirdPointer => 0xd624,
                Rows => 0xd626,
                Pair => 0xd629,
                PairStore => 0xd62b,
                PairSwap => 0xd62f,
                PairMerge => 0xd630,
                PairMask => 0xd632,
                PairSave => 0xd635,
                SourceIncrement(i) => 0xd637 + u16::from(i) * 2,
                Third => 0xd63b,
                ThirdMask => 0xd63d,
                ThirdSave => 0xd640,
                ThirdMerge => 0xd642,
                ThirdSwap => 0xd644,
                ThirdMergeSaved => 0xd645,
                ThirdStore => 0xd647,
                ThirdIncrement => 0xd64b,
                DestinationIncrement(i) => 0xd64d + u16::from(i),
                RowDecrement => 0xd64f,
                RowBranch => 0xd650,
                Destination => 0xd652,
                CarryDestination => 0xd653,
                AddDestination => 0xd654,
                SaveDestination => 0xd657,
                StrideSource => 0xd658,
                StrideMask => 0xd65a,
                StrideBranch => 0xd65d,
                StrideReload => 0xd65f,
                StrideCarry => 0xd661,
                StrideAdd => 0xd662,
                StrideSave => 0xd665,
                NextSource => 0xd667,
                CountDecrement => 0xd669,
                CountBranch => 0xd66b,
                Return => 0xd66d,
                Complete => unreachable!(),
            }
        } else {
            let row = if self.second_row { 0x17 } else { 0 };
            match self.phase {
                Count => 0xd5ce,
                Source => 0xd5d0,
                CarrySource => 0xd5d2,
                AddSource => 0xd5d3,
                ThirdPointer => 0xd5d6,
                Rows => 0xd5d8,
                Pair => 0xd5db + row,
                PairStore => 0xd5dd + row,
                SourceIncrement(i) => 0xd5e1 + row + u16::from(i) * 2,
                Third => 0xd5e5 + row,
                ThirdMask => 0xd5e7 + row,
                ThirdStore => 0xd5ea + row,
                ThirdIncrement => 0xd5ee + row,
                DestinationIncrement(i) => 0xd5f0 + row + u16::from(i),
                RowDecrement => 0xd609,
                RowBranch => 0xd60a,
                Destination => 0xd60c,
                CarryDestination => 0xd60d,
                AddDestination => 0xd60e,
                SaveDestination => 0xd611,
                NextSource => 0xd612,
                CountDecrement => 0xd614,
                CountBranch => 0xd616,
                Return => 0xd618,
                _ => unreachable!(),
            }
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        use ConvertPhase::*;
        if self.complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d, cpu.p & 0x38),
            (0, self.pc(), 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        self.phase = match self.phase {
            Count => {
                w.dp_store(14, w.cpu.y, 2);
                Source
            }
            Source => {
                w.dp_store(0, w.cpu.a, 2);
                CarrySource
            }
            CarrySource => {
                w.clear_carry();
                AddSource
            }
            AddSource => {
                w.drain(16);
                w.adc(16);
                ThirdPointer
            }
            ThirdPointer => {
                w.dp_store(3, w.cpu.a, 2);
                Rows
            }
            Rows => {
                w.drain(16);
                w.cpu.y = if self.high { 7 } else { 3 };
                w.flags(w.cpu.y, 2);
                self.second_row = false;
                Pair
            }
            Pair => {
                w.cpu.a = w.indirect_read(0, 2);
                w.flags(w.cpu.a, 2);
                PairStore
            }
            PairStore => {
                w.drain(24);
                w.write(0x7e9000 + u32::from(w.cpu.x), w.cpu.a, 2);
                if self.high {
                    PairSwap
                } else {
                    SourceIncrement(0)
                }
            }
            PairSwap => {
                w.swap_bytes();
                PairMerge
            }
            PairMerge => {
                w.cpu.a |= w.indirect_read(0, 2);
                w.flags(w.cpu.a, 2);
                PairMask
            }
            PairMask => {
                w.mask(255, 2);
                PairSave
            }
            PairSave => {
                w.dp_store(8, w.cpu.a, 2);
                SourceIncrement(0)
            }
            SourceIncrement(i) => {
                w.dp_increment(0, 2, true);
                if i == 0 {
                    SourceIncrement(1)
                } else {
                    Third
                }
            }
            Third => {
                w.cpu.a = w.indirect_read(3, 2);
                w.flags(w.cpu.a, 2);
                ThirdMask
            }
            ThirdMask => {
                w.mask(255, 2);
                if self.high {
                    ThirdSave
                } else {
                    ThirdStore
                }
            }
            ThirdSave => {
                w.dp_store(0xbd, w.cpu.a, 2);
                ThirdMerge
            }
            ThirdMerge => {
                let rhs = w.dp_read(8, 2);
                w.cpu.a |= rhs;
                w.flags(w.cpu.a, 2);
                ThirdSwap
            }
            ThirdSwap => {
                w.swap_bytes();
                ThirdMergeSaved
            }
            ThirdMergeSaved => {
                let rhs = w.dp_read(0xbd, 2);
                w.cpu.a |= rhs;
                w.flags(w.cpu.a, 2);
                ThirdStore
            }
            ThirdStore => {
                w.drain(24);
                w.write(0x7e9010 + u32::from(w.cpu.x), w.cpu.a, 2);
                ThirdIncrement
            }
            ThirdIncrement => {
                w.dp_increment(3, 2, true);
                DestinationIncrement(0)
            }
            DestinationIncrement(i) => {
                w.increment_x();
                if i == 0 {
                    DestinationIncrement(1)
                } else if !self.high && !self.second_row {
                    self.second_row = true;
                    Pair
                } else {
                    RowDecrement
                }
            }
            RowDecrement => {
                w.decrement_y();
                RowBranch
            }
            RowBranch => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    self.second_row = false;
                    Pair
                } else {
                    Destination
                }
            }
            Destination => {
                w.drain(6);
                w.cpu.a = w.cpu.x;
                w.flags(w.cpu.a, 2);
                CarryDestination
            }
            CarryDestination => {
                w.clear_carry();
                AddDestination
            }
            AddDestination => {
                w.drain(16);
                w.adc(16);
                SaveDestination
            }
            SaveDestination => {
                w.tax_word();
                if self.high {
                    StrideSource
                } else {
                    NextSource
                }
            }
            StrideSource => {
                w.cpu.a = w.dp_read(3, 2);
                w.flags(w.cpu.a, 2);
                StrideMask
            }
            StrideMask => {
                w.mask(0x78, 2);
                StrideBranch
            }
            StrideBranch => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    NextSource
                } else {
                    StrideReload
                }
            }
            StrideReload => {
                w.cpu.a = w.dp_read(3, 2);
                w.flags(w.cpu.a, 2);
                StrideCarry
            }
            StrideCarry => {
                w.clear_carry();
                StrideAdd
            }
            StrideAdd => {
                w.drain(16);
                w.adc(0x180);
                StrideSave
            }
            StrideSave => {
                w.dp_store(3, w.cpu.a, 2);
                NextSource
            }
            NextSource => {
                w.cpu.a = w.dp_read(3, 2);
                w.flags(w.cpu.a, 2);
                CountDecrement
            }
            CountDecrement => {
                w.dp_increment(14, 2, false);
                CountBranch
            }
            CountBranch => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    Source
                } else {
                    Return
                }
            }
            Return => {
                w.rts();
                Complete
            }
            Complete => unreachable!(),
        };
        if !self.complete() {
            w.cpu.pc = self.pc();
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
    fn retained_item_conversion_matches_live_source_instructions_and_stride() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for high in [false, true] {
            // The final case aliases source and destination, so later reads
            // consume this conversion's earlier writes instead of a snapshot.
            for (bank, pointer) in [
                (0x11, 0x8000),
                (0x7f, 0x4000),
                (0x7f, 0x4ff1),
                (0x7e, 0x9480),
            ] {
                for tiles in [1, 3, 7, 8, 14, 16, 32] {
                    for (v, h, odd) in [(238, 858, false), (224, 1338, true), (261, 1350, false)] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 7)) as u8;
                        }
                        machine.ram[2] = bank;
                        machine.ram[5] = bank;
                        machine.ram[0x1fa..0x1fc].copy_from_slice(&0xd571u16.to_le_bytes());
                        let mut work = ItemConversion::new(high);
                        let mut native = NativeNmiContext {
                            a: pointer,
                            x: 0x480,
                            y: tiles,
                            d: 0,
                            db: 0,
                            bank: 0,
                            pc: work.pc(),
                            sp: 0x1f9,
                            p: 0x43,
                        };
                        machine.cpu.a = native.a;
                        machine.cpu.x = native.x;
                        machine.cpu.y = native.y;
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
                            let label=format!("high={high} pointer={bank:02x}:{pointer:04x} tiles={tiles} phase={:?} second={}",work.phase,work.second_row);
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
                            assert!(count < 20000);
                        }
                        assert_eq!((native.pc, native.sp), (0xd572, 0x1fb));
                        assert_eq!(ram, source.snes().ram);
                    }
                }
            }
        }
    }
}
