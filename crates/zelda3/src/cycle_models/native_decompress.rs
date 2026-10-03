//! Graphics decompression as retained native command/loop work. Every command
//! byte, live back-reference, scratch update and nested call is owned here.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    SecondaryDestination(u8),
    Destination(u8),
    Tables(u8),
    Width,
    OutputIndex,
    Header,
    StopCompare,
    StopBranch,
    ReturnWidth,
    Return,
    Command(u8),
    Short(u8),
    Long(u8),
    Length(u8),
    Dispatch(u8),
    Literal(u8),
    Fill(u8),
    Incrementing(u8),
    Word(u8),
    Reference(u8),
    Next(u8),
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeDecompression {
    phase: Phase,
    sprite: bool,
    secondary: bool,
    table_entry: bool,
    bank02: bool,
    byte_return: Option<Phase>,
}
impl NativeDecompression {
    pub(crate) fn new(sprite: bool) -> Self {
        Self {
            phase: Phase::Destination(0),
            sprite,
            secondary: false,
            table_entry: false,
            bank02: false,
            byte_return: None,
        }
    }
    pub(crate) fn sprite_secondary() -> Self {
        Self {
            phase: Phase::SecondaryDestination(0),
            secondary: true,
            ..Self::new(true)
        }
    }
    /// Callers which have already selected the destination retain that pointer.
    pub(crate) fn table_entry(sprite: bool) -> Self {
        Self {
            phase: Phase::Tables(0),
            table_entry: true,
            ..Self::new(sprite)
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    /// Bank 02 callers supply both pointers. Their back-reference offsets
    /// are big endian; the graphics entry has an additional XBA.
    pub(crate) fn bank02() -> Self {
        Self {
            phase: Phase::Width,
            bank02: true,
            ..Self::new(false)
        }
    }
    pub(crate) fn routine(&self) -> u32 {
        if self.bank02 {
            if matches!(self.phase, Phase::Next(_)) {
                0x02ff5f
            } else {
                0x02febb
            }
        } else if matches!(self.phase, Phase::Next(_)) {
            0xe843
        } else if self.table_entry {
            if self.sprite {
                0xe772
            } else {
                0xe78f
            }
        } else if self.secondary {
            0xe75c
        } else if self.sprite {
            0xe766
        } else {
            0xe783
        }
    }
    fn pc(&self) -> u16 {
        self.continuation_pc(self.phase)
    }
    fn continuation_pc(&self, phase: Phase) -> u16 {
        use Phase::*;
        if !self.bank02 {
            return Self::phase_pc(phase, self.sprite);
        }
        match phase {
            Width => 0xfebb,
            OutputIndex => 0xfebd,
            Header => 0xfec0,
            StopCompare => 0xfec3,
            StopBranch => 0xfec5,
            ReturnWidth => 0xfec7,
            Return => 0xfec9,
            Command(i) => [0xfeca, 0xfecc, 0xfece, 0xfed0][usize::from(i)],
            Short(i) => [0xfed2, 0xfed3, 0xfed5, 0xfed7, 0xfeda][usize::from(i)],
            Long(i) => [
                0xfedc, 0xfede, 0xfedf, 0xfee0, 0xfee1, 0xfee3, 0xfee4, 0xfee6, 0xfee8, 0xfee9,
                0xfeec,
            ][usize::from(i)],
            Length(i) => [0xfeee, 0xfeef, 0xfef1, 0xfef3][usize::from(i)],
            Dispatch(i) => [0xfef4, 0xfef6, 0xfef8, 0xfef9, 0xfefb, 0xfefc][usize::from(i)],
            Incrementing(i) => [
                0xfefe, 0xff01, 0xff03, 0xff05, 0xff06, 0xff07, 0xff08, 0xff0a,
            ][usize::from(i)],
            Literal(i) => [
                0xff0c, 0xff0f, 0xff11, 0xff12, 0xff14, 0xff15, 0xff17, 0xff19,
            ][usize::from(i)],
            Fill(i) => [0xff1b, 0xff1e, 0xff20, 0xff22, 0xff23, 0xff24, 0xff26][usize::from(i)],
            Word(i) => [
                0xff28, 0xff2b, 0xff2c, 0xff2f, 0xff31, 0xff32, 0xff34, 0xff35, 0xff36, 0xff38,
                0xff39, 0xff3b, 0xff3c, 0xff3d, 0xff3f,
            ][usize::from(i)],
            Reference(3) => unreachable!("bank 02 has no second byte swap"),
            Reference(i) => [
                0xff42, 0xff45, 0xff46, 0, 0xff49, 0xff4a, 0xff4b, 0xff4c, 0xff4e, 0xff4f, 0xff50,
                0xff52, 0xff53, 0xff54, 0xff56, 0xff58, 0xff5a, 0xff5c,
            ][usize::from(i)],
            Next(i) => [
                0xff5f, 0xff61, 0xff63, 0xff64, 0xff66, 0xff69, 0xff6b, 0xff6d,
            ][usize::from(i)],
            _ => unreachable!("bank 02 decompression phase {phase:?}"),
        }
    }
    fn phase_pc(phase: Phase, sprite: bool) -> u16 {
        use Phase::*;
        match phase {
            SecondaryDestination(i) => 0xe75c + u16::from(i) * 2,
            Destination(i) => (if sprite { 0xe766 } else { 0xe783 }) + u16::from(i) * 2,
            Tables(i) => {
                (if sprite { 0xe772 } else { 0xe78f }) + [0, 3, 5, 8, 10, 13, 15][usize::from(i)]
            }
            Width => 0xe79e,
            OutputIndex => 0xe7a0,
            Header => 0xe7a3,
            StopCompare => 0xe7a6,
            StopBranch => 0xe7a8,
            ReturnWidth => 0xe7aa,
            Return => 0xe7ac,
            Command(i) => [0xe7ad, 0xe7af, 0xe7b1, 0xe7b3][usize::from(i)],
            Short(i) => [0xe7b5, 0xe7b6, 0xe7b8, 0xe7ba, 0xe7bd][usize::from(i)],
            Long(i) => [
                0xe7bf, 0xe7c1, 0xe7c2, 0xe7c3, 0xe7c4, 0xe7c6, 0xe7c7, 0xe7c9, 0xe7cb, 0xe7cc,
                0xe7cf,
            ][usize::from(i)],
            Length(i) => [0xe7d1, 0xe7d2, 0xe7d4, 0xe7d6][usize::from(i)],
            Dispatch(i) => [0xe7d7, 0xe7d9, 0xe7db, 0xe7dc, 0xe7de, 0xe7df][usize::from(i)],
            Literal(i) => [
                0xe7ef, 0xe7f2, 0xe7f4, 0xe7f5, 0xe7f7, 0xe7f8, 0xe7fa, 0xe7fc,
            ][usize::from(i)],
            Fill(i) => [0xe7fe, 0xe801, 0xe803, 0xe805, 0xe806, 0xe807, 0xe809][usize::from(i)],
            Incrementing(i) => [
                0xe7e1, 0xe7e4, 0xe7e6, 0xe7e8, 0xe7e9, 0xe7ea, 0xe7eb, 0xe7ed,
            ][usize::from(i)],
            Word(i) => [
                0xe80b, 0xe80e, 0xe80f, 0xe812, 0xe814, 0xe815, 0xe817, 0xe818, 0xe819, 0xe81b,
                0xe81c, 0xe81e, 0xe81f, 0xe820, 0xe822,
            ][usize::from(i)],
            Reference(i) => [
                0xe825, 0xe828, 0xe829, 0xe82c, 0xe82d, 0xe82e, 0xe82f, 0xe830, 0xe832, 0xe833,
                0xe834, 0xe836, 0xe837, 0xe838, 0xe83a, 0xe83c, 0xe83e, 0xe840,
            ][usize::from(i)],
            Next(i) => [
                0xe843, 0xe845, 0xe847, 0xe848, 0xe84a, 0xe84d, 0xe84f, 0xe851,
            ][usize::from(i)],
            Complete => unreachable!(),
        }
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        use Phase::*;
        if self.complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.d),
            (if self.bank02 { 2 } else { 0 }, self.pc(), 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut call = None;
        self.phase = match self.phase {
            SecondaryDestination(i) => {
                match i {
                    0 => w.dp_store(0, 0, 1),
                    1 => w.immediate(0x46, 1),
                    2 => w.dp_store(1, w.cpu.a & 255, 1),
                    3 => w.immediate(0x7f, 1),
                    4 => {
                        w.branch(true);
                    }
                    _ => unreachable!(),
                }
                if i == 4 {
                    Destination(4)
                } else {
                    SecondaryDestination(i + 1)
                }
            }
            Destination(i) => {
                match i {
                    0 => w.dp_store(0, 0, 1),
                    1 => w.immediate(0x40, 1),
                    2 => w.dp_store(1, w.cpu.a & 255, 1),
                    3 => w.immediate(0x7f, 1),
                    4 => w.dp_store(2, w.cpu.a & 255, 1),
                    5 => w.dp_store(5, w.cpu.a & 255, 1),
                    _ => unreachable!(),
                }
                if i == 5 {
                    Tables(0)
                } else {
                    Destination(i + 1)
                }
            }
            Tables(i) => {
                if i == 6 {
                    w.branch(true);
                    Width
                } else {
                    let table = if self.sprite {
                        [0xcff3u16, 0xd0d2, 0xd1b1]
                    } else {
                        [0xcf80, 0xd05f, 0xd13e]
                    };
                    if i % 2 == 0 {
                        let base = table[usize::from(i / 2)];
                        w.drain(16);
                        if w.cpu.p & 0x10 == 0 || (base & 255) + w.cpu.y > 255 {
                            w.drain(6);
                        }
                        let value = w.read(w.absolute(base).wrapping_add(u32::from(w.cpu.y)), 1);
                        w.load(value, 1);
                    } else {
                        w.dp_store(0xca - u16::from(i / 2), w.cpu.a & 255, 1);
                    }
                    if i == 5 && !self.sprite {
                        Width
                    } else {
                        Tables(i + 1)
                    }
                }
            }
            Width => {
                w.width(0x10, true);
                OutputIndex
            }
            OutputIndex => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
                Header
            }
            Header => {
                call = Some(StopCompare);
                Next(0)
            }
            StopCompare => {
                w.compare_immediate(255, 1);
                StopBranch
            }
            StopBranch => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    Command(0)
                } else {
                    ReturnWidth
                }
            }
            ReturnWidth => {
                w.width(0x10, false);
                Return
            }
            Return => {
                w.rts();
                Complete
            }
            Command(i) => {
                match i {
                    0 => w.dp_store(0xcd, w.cpu.a & 255, 1),
                    1 => w.mask(0xe0, 1),
                    2 => w.compare_immediate(0xe0, 1),
                    3 => {
                        if w.branch(w.cpu.p & ZERO != 0) {
                            self.phase = Long(0);
                        } else {
                            self.phase = Short(0);
                        }
                    }
                    _ => unreachable!(),
                }
                if i == 3 {
                    self.phase
                } else {
                    Command(i + 1)
                }
            }
            Short(i) => {
                match i {
                    0 => {
                        w.drain(6);
                        w.push(w.cpu.a & 255, 1);
                    }
                    1 => {
                        let value = w.dp_read(0xcd, 1);
                        w.load(value, 1);
                    }
                    2 => w.width(0x20, true),
                    3 => w.mask(0x1f, 2),
                    4 => {
                        w.branch(true);
                    }
                    _ => unreachable!(),
                }
                if i == 4 {
                    Length(0)
                } else {
                    Short(i + 1)
                }
            }
            Long(i) => {
                match i {
                    0 | 6 => {
                        let value = w.dp_read(0xcd, 1);
                        w.load(value, 1);
                    }
                    1..=3 => w.shift_left(1),
                    4 => w.mask(0xe0, 1),
                    5 => {
                        w.drain(6);
                        w.push(w.cpu.a & 255, 1);
                    }
                    7 => w.mask(3, 1),
                    8 => w.swap_bytes(),
                    9 => call = Some(Long(10)),
                    10 => w.width(0x20, true),
                    _ => unreachable!(),
                }
                if i == 9 {
                    Next(0)
                } else if i == 10 {
                    Length(0)
                } else {
                    Long(i + 1)
                }
            }
            Length(i) => {
                match i {
                    0 => w.increment_a(2),
                    1 => w.dp_store(0xcb, w.cpu.a, 2),
                    2 => w.width(0x20, false),
                    3 => {
                        w.drain(12);
                        let value = w.pull(1);
                        w.load(value, 1);
                    }
                    _ => unreachable!(),
                }
                if i == 3 {
                    Dispatch(0)
                } else {
                    Length(i + 1)
                }
            }
            Dispatch(i) => match i {
                0 => {
                    if w.branch(w.cpu.p & ZERO != 0) {
                        Literal(0)
                    } else {
                        Dispatch(1)
                    }
                }
                1 => {
                    if w.branch(w.cpu.p & NEGATIVE != 0) {
                        Reference(0)
                    } else {
                        Dispatch(2)
                    }
                }
                2 => {
                    w.shift_left(1);
                    Dispatch(3)
                }
                3 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        Fill(0)
                    } else {
                        Dispatch(4)
                    }
                }
                4 => {
                    w.shift_left(1);
                    Dispatch(5)
                }
                5 => {
                    if w.branch(w.cpu.p & NEGATIVE == 0) {
                        Word(0)
                    } else {
                        Incrementing(0)
                    }
                }
                _ => unreachable!(),
            },
            Literal(i) => {
                let mut next = Literal(i + 1);
                match i {
                    0 => {
                        call = Some(next);
                        next = Next(0);
                    }
                    1 => w.decompress_store(),
                    2 => w.increment_y(),
                    3 => w.decompress_count(),
                    4 => w.decrement_x(),
                    5 => w.dp_store(0xcb, w.cpu.x, 2),
                    6 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Literal(0)
                        }
                    }
                    7 => {
                        w.branch(true);
                        next = Header;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Fill(i) => {
                let mut next = Fill(i + 1);
                match i {
                    0 => {
                        call = Some(next);
                        next = Next(0);
                    }
                    1 => w.decompress_count(),
                    2 => w.decompress_store(),
                    3 => w.increment_y(),
                    4 => w.decrement_x(),
                    5 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Fill(2)
                        }
                    }
                    6 => {
                        w.branch(true);
                        next = Header;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Incrementing(i) => {
                let mut next = Incrementing(i + 1);
                match i {
                    0 => {
                        call = Some(next);
                        next = Next(0);
                    }
                    1 => w.decompress_count(),
                    2 => w.decompress_store(),
                    3 => w.increment_a(1),
                    4 => w.increment_y(),
                    5 => w.decrement_x(),
                    6 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Incrementing(2)
                        }
                    }
                    7 => {
                        w.branch(true);
                        next = Header;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Word(i) => {
                let mut next = Word(i + 1);
                match i {
                    0 | 2 => {
                        call = Some(next);
                        next = Next(0);
                    }
                    1 | 4 | 9 => w.swap_bytes(),
                    3 => w.decompress_count(),
                    5 | 10 => w.decompress_store(),
                    6 | 11 => w.increment_y(),
                    7 | 12 => w.decrement_x(),
                    8 => {
                        if w.branch(w.cpu.p & ZERO != 0) {
                            next = Word(14)
                        }
                    }
                    13 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Word(4)
                        }
                    }
                    14 => {
                        w.drain(16);
                        next = Header;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Reference(i) => {
                let mut next = Reference(if self.bank02 && i == 2 { 4 } else { i + 1 });
                match i {
                    0 | 2 => {
                        call = Some(next);
                        next = Next(0);
                    }
                    1 | 3 => w.swap_bytes(),
                    4 => w.tax_word(),
                    5 => {
                        w.drain(6);
                        w.push(w.cpu.y, 2);
                    }
                    6 => {
                        w.drain(6);
                        w.cpu.y = w.cpu.x;
                        w.flags(w.cpu.y, 2);
                    }
                    7 => {
                        let address = w.indirect_y_address(0);
                        let value = w.read(address, 1);
                        w.load(value, 1);
                    }
                    8 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.y;
                        w.flags(w.cpu.x, 2);
                    }
                    9 => {
                        w.drain(12);
                        w.cpu.y = w.pull(2);
                        w.flags(w.cpu.y, 2);
                    }
                    10 => w.decompress_store(),
                    11 => w.increment_y(),
                    12 => w.increment_x(),
                    13 => w.width(0x20, true),
                    14 => w.dp_increment(0xcb, 2, false),
                    15 => w.width(0x20, false),
                    16 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Reference(5)
                        }
                    }
                    17 => {
                        w.drain(16);
                        next = Header;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Next(i) => {
                let mut next = Next(i + 1);
                match i {
                    0 => {
                        let value = w.indirect_read(0xc8, 1);
                        w.load(value, 1);
                    }
                    1 => {
                        w.cpu.x = w.dp_read(0xc8, 2);
                        w.flags(w.cpu.x, 2);
                    }
                    2 => w.increment_x(),
                    3 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Next(6)
                        }
                    }
                    4 => {
                        w.drain(16);
                        w.cpu.x = 0x8000;
                        w.flags(w.cpu.x, 2);
                    }
                    5 => w.dp_increment(0xca, 1, true),
                    6 => w.dp_store(0xc8, w.cpu.x, 2),
                    7 => {
                        w.rts();
                        next = self.byte_return.take().unwrap();
                        assert_eq!(
                            w.cpu.pc,
                            self.continuation_pc(next),
                            "decompression byte-call return stack"
                        );
                    }
                    _ => unreachable!(),
                }
                next
            }
            Complete => unreachable!(),
        };
        if let Some(return_phase) = call {
            assert!(self.byte_return.is_none());
            self.byte_return = Some(return_phase);
            w.drain(16);
            w.drain(6);
            w.push(w.cpu.pc + 2, 2);
        }
        if !self.complete() {
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}
impl<B: WorkBus> Work<'_, B> {
    fn decompress_store(&mut self) {
        let address = self.indirect_y_address(0);
        self.write(address, self.cpu.a & 255, 1);
    }
    fn decompress_count(&mut self) {
        self.cpu.x = self.dp_read(0xcb, 2);
        self.flags(self.cpu.x, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    fn compare(rom: &[u8], sprite: bool, sheet: u16, odd: bool, bank02_source: Option<u32>) {
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 7)) as u8;
        }
        machine.ram[0x1fa..0x1fc].copy_from_slice(&0xd551u16.to_le_bytes());
        let mut work = NativeDecompression::new(sprite);
        if let Some(address) = bank02_source {
            work = NativeDecompression::bank02();
            machine.ram[0..3].copy_from_slice(&[0, 0x40, 0x7f]);
            machine.ram[0xc8..0xcb].copy_from_slice(&address.to_le_bytes()[..3]);
        }
        let mut native = NativeNmiContext {
            a: 0xa547,
            x: 0xfc,
            y: sheet,
            d: 0,
            db: 0,
            bank: if bank02_source.is_some() { 2 } else { 0 },
            pc: work.pc(),
            sp: 0x1f9,
            p: 0xf1,
        };
        machine.cpu.a = native.a;
        machine.cpu.x = native.x;
        machine.cpu.y = native.y;
        machine.cpu.pc = native.pc;
        machine.cpu.k = native.bank;
        machine.cpu.sp = native.sp;
        machine.cpu.e = false;
        machine.cpu.unpack_flags(native.p);
        let mut ram = machine.ram.clone();
        let mut clock = CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(if odd { 224 } else { 261 }, 1338),
            CpuBusWorkload::default(),
            CpuFieldTiming::non_interlace(odd),
        );
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        let mut count = 0;
        while !work.complete() {
            let label = format!("sprite={sprite} sheet={sheet} phase={:?}", work.phase);
            super::super::native_work::compare_instruction(
                &label,
                &mut native,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| {
                    work.step(cpu, clock, bus).unwrap();
                },
            );
            work = work.clone();
            count += 1;
            assert!(count < 500000);
        }
        assert_eq!((native.pc, native.sp), (0xd552, 0x1fb));
        assert_eq!(ram, source.snes().ram);
    }
    #[test]
    fn retained_decompression_matches_all_compressed_sheets_and_command_edges() {
        let Ok(original) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for sprite in [false, true] {
            for sheet in if sprite { 12..108 } else { 0..114 } {
                for odd in [false, true] {
                    compare(&original, sprite, sheet, odd, None);
                }
            }
        }
        for command in 0u8..8 {
            for length in [1usize, 2, 31, 32, 33, 255, 256, 257, 1024] {
                let mut rom = original.clone();
                let mut stream = Vec::new();
                if length <= 32 && command < 7 {
                    stream.push((command << 5) | ((length - 1) as u8));
                } else {
                    stream.push(0xe0 | (command << 2) | (((length - 1) >> 8) as u8));
                    stream.push((length - 1) as u8);
                }
                match command {
                    0 => stream.extend((0..length).map(|i| i as u8)),
                    1 | 3 => stream.push(0x7f),
                    2 => stream.extend([0x3f, 0x80]),
                    _ => stream.extend([0, 0]),
                }
                stream.push(0xff);
                let start = 0x11 * 0x8000 + 0x7ff0;
                rom[start..start + stream.len()].copy_from_slice(&stream);
                for (table, value) in [(0xcff3, 0x11), (0xd0d2, 0xff), (0xd1b1, 0xf0)] {
                    rom[table - 0x8000 + 95] = value;
                }
                compare(&rom, true, 95, length & 1 != 0, None);
                compare(&rom, false, 0, length & 1 != 0, Some(0x11fff0));
            }
        }
        for odd in [false, true] {
            compare(&original, false, 0, odd, Some(0x03e800));
        }
    }
}
