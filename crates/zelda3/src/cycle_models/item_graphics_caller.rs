//! Item-sheet caller composition. Native callees keep their live CPU/scratch
//! ownership; the return stack and retained caller phase must agree.
use super::{
    item_graphics::ItemConversion, native_decompress::NativeDecompression,
    native_work::{Work, WorkBus}, nmi_stack::NativeNmiContext,
};
use snes::CpuMasterTimeline;

pub(crate) const SOURCES: &[(bool, usize, u32)] = &[
    (true, 0, 0x10f000),
    (true, 95, 0x188d64),
    (true, 96, 0x18919d),
    (true, 84, 0x17e1cc),
    (true, 88, 0x17ed58),
    (false, 15, 0x11fc21),
    (false, 5, 0x11cf4c),
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    for &(sprite, sheet, source) in SOURCES {
        let tables = if sprite {
            [0xcff3, 0xd0d2, 0xd1b1]
        } else {
            [0xcf80, 0xd05f, 0xd13e]
        };
        for (i, table) in tables.into_iter().enumerate() {
            if address == table + sheet as u32 {
                return Some((source >> (16 - i * 8)) as u8);
            }
        }
    }
    if let Some(offset) = address.checked_sub(0xd21d).filter(|&n| n < 20) {
        let groups = [0u16, 11, 8, 38, 42, 45, 34, 3, 33, 46];
        return Some((groups[(offset / 2) as usize] * 24).to_le_bytes()[(offset & 1) as usize]);
    }
    None
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Main(u8),
    Raw(u8),
    Unpack(u8),
    Temp(u8),
    Expand(u8),
    Aux(u8),
    Converting,
    Decompressing,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ItemGraphics {
    phase: Phase,
    sheet_return: Option<Phase>,
    child_return: Option<Phase>,
    conversion: Option<ItemConversion>,
    decompression: Option<NativeDecompression>,
}
impl Default for ItemGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Main(0),
            sheet_return: None,
            child_return: None,
            conversion: None,
            decompression: None,
        }
    }
}
impl ItemGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Main(_) => 0xd231,
            Phase::Raw(_) => 0xd537,
            Phase::Unpack(_) => 0xd54e,
            Phase::Temp(_) | Phase::Expand(_) => 0xd553,
            Phase::Aux(_) => 0xd3c6,
            Phase::Converting => self.conversion.as_ref().unwrap().routine(),
            Phase::Decompressing => self.decompression.as_ref().unwrap().routine(),
            Phase::Complete => 0xd231,
        }
    }
    fn pc(&self) -> u16 {
        Self::phase_pc(self.phase)
    }
    fn phase_pc(phase: Phase) -> u16 {
        match phase {
            Phase::Main(i) => [
                0xd231, 0xd232, 0xd233, 0xd234, 0xd236, 0xd238, 0xd23a, 0xd23d, 0xd23f, 0xd241,
                0xd243, 0xd246, 0xd248, 0xd24b, 0xd24d, 0xd250, 0xd252, 0xd254, 0xd257, 0xd259,
                0xd25c, 0xd25e, 0xd261, 0xd263, 0xd266, 0xd268, 0xd26a, 0xd26d, 0xd26f, 0xd272,
                0xd274, 0xd276, 0xd279, 0xd27b, 0xd27e, 0xd280, 0xd282, 0xd285, 0xd286, 0xd289,
                0xd28c, 0xd28d, 0xd28e, 0xd291, 0xd294, 0xd297, 0xd299, 0xd29b, 0xd29e, 0xd2a0,
                0xd2a2, 0xd2a5, 0xd2a8, 0xd2a9, 0xd2ac, 0xd2ad, 0xd2ae, 0xd2b1, 0xd2b4, 0xd2b7,
                0xd2b9, 0xd2bc, 0xd2bd,
            ][usize::from(i)],
            Phase::Raw(i) => [
                0xd537, 0xd539, 0xd53b, 0xd53e, 0xd540, 0xd542, 0xd545, 0xd547, 0xd54a, 0xd54c,
            ][usize::from(i)],
            Phase::Unpack(i) => [0xd54e, 0xd54f, 0xd552][usize::from(i)],
            Phase::Temp(i) => 0xd553 + u16::from(i) * 2,
            Phase::Expand(i) => [
                0xd561, 0xd563, 0xd565, 0xd567, 0xd56a, 0xd56c, 0xd56e, 0xd56f, 0xd572, 0xd573,
                0xd574, 0xd577, 0xd579, 0xd57c, 0xd57e, 0xd580, 0xd582, 0xd584,
            ][usize::from(i)],
            Phase::Aux(i) => [
                0xd3c6, 0xd3c8, 0xd3cb, 0xd3cd, 0xd3cf, 0xd3d2, 0xd3d5, 0xd3d8, 0xd3da, 0xd3dc,
                0xd3df, 0xd3e1, 0xd3e3, 0xd3e6, 0xd3e9, 0xd3ec, 0xd3ee, 0xd3f0, 0xd3f3, 0xd3f5,
                0xd3f7, 0xd3f8, 0xd3fb, 0xd3fe, 0xd401, 0xd404, 0xd406,
            ][usize::from(i)],
            _ => unreachable!(),
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
        if self.phase == Converting {
            let child = self.conversion.as_mut().unwrap();
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.conversion = None;
                self.phase = self.child_return.take().unwrap();
                assert_eq!(cpu.pc, self.pc());
            }
            return cycles;
        }
        if self.phase == Decompressing {
            let child = self.decompression.as_mut().unwrap();
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.decompression = None;
                self.phase = self.child_return.take().unwrap();
                assert_eq!(cpu.pc, self.pc());
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, self.pc(), 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut sheet = None;
        let mut conversion = None;
        let mut decompression = None;
        let mut returned = false;
        self.phase = match self.phase {
            Main(i) => {
                let mut next = Main(i + 1);
                match i {
                    0 | 1 => {
                        w.drain(6);
                        w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
                    }
                    2 | 61 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    3 => w.width(0x20, true),
                    4 => w.dp_store(10, 0, 2),
                    5 => w.dp_store(12, 0, 2),
                    6 => w.immediate(0x480, 2),
                    7 => w.dp_store(6, w.cpu.a, 2),
                    8 => w.width(0x20, false),
                    9 | 11 | 13 | 16 | 18 | 20 | 22 | 25 | 27 | 30 => w.immediate(
                        match i {
                            9 | 11 | 27 => 7,
                            13 | 18 => 3,
                            16 | 22 => 4,
                            20 => 1,
                            25 => 14,
                            _ => 2,
                        },
                        1,
                    ),
                    10 | 12 | 14 | 23 => sheet = Some((Raw(0), next)),
                    17 | 26 | 31 => sheet = Some((Unpack(0), next)),
                    19 | 21 | 28 => sheet = Some((Temp(0), next)),
                    15 | 24 | 29 | 32 | 46 => w.item_y(
                        match i {
                            15 | 29 => 0x5f,
                            24 | 46 => 0x60,
                            _ => 0x54,
                        },
                        1,
                    ),
                    33 | 47 => decompression = Some((true, next)),
                    34 | 48 => w.width(0x30, true),
                    35 | 49 => {
                        let value = w.dp_read(0, 2);
                        w.load(value, 2);
                    }
                    36 | 50 => w.item_x(if i == 36 { 0x1480 } else { 0x2280 }, 2),
                    37 | 52 => {
                        w.drain(6);
                        w.push(w.cpu.a, 2);
                    }
                    38 | 43 | 51 | 57 => w.item_y(if i < 45 { 8 } else { 3 }, 2),
                    39 | 44 | 53 | 58 => conversion = Some((true, next)),
                    40 | 54 => {
                        w.drain(12);
                        let value = w.pull(2);
                        w.load(value, 2);
                    }
                    41 | 55 => w.clear_carry(),
                    42 | 56 => {
                        w.drain(16);
                        w.adc(0x180);
                    }
                    45 | 59 => w.width(0x30, false),
                    60 => sheet = Some((Aux(0), next)),
                    62 => {
                        w.rtl();
                        next = Complete;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Raw(i) => {
                match i {
                    0 => w.dp_store(10, w.cpu.a & 255, 1),
                    1 => w.item_y(0, 1),
                    2 | 5 | 7 => {
                        let base = match i {
                            2 => 0xcff3,
                            5 => 0xd0d2,
                            _ => 0xd1b1,
                        };
                        let value = w.item_absolute_y(base, 1);
                        w.load(value, 1);
                    }
                    3 | 4 | 6 | 8 => w.dp_store(
                        match i {
                            3 => 2,
                            4 => 5,
                            6 => 1,
                            _ => 0,
                        },
                        w.cpu.a & 255,
                        1,
                    ),
                    9 => {
                        w.branch(true);
                    }
                    _ => unreachable!(),
                }
                if i == 9 {
                    Expand(0)
                } else {
                    Raw(i + 1)
                }
            }
            Unpack(i) => {
                match i {
                    0 => {
                        w.drain(6);
                        w.push(w.cpu.a & 255, 1);
                    }
                    1 => decompression = Some((true, Unpack(2))),
                    2 => {
                        w.drain(12);
                        let value = w.pull(1);
                        w.load(value, 1);
                    }
                    _ => unreachable!(),
                }
                if i == 2 {
                    Temp(0)
                } else {
                    Unpack(i + 1)
                }
            }
            Temp(i) => {
                match i {
                    0 => w.dp_store(10, w.cpu.a & 255, 1),
                    1 => w.dp_store(0, 0, 1),
                    2 => w.immediate(0x40, 1),
                    3 => w.dp_store(1, w.cpu.a & 255, 1),
                    4 => w.immediate(0x7f, 1),
                    5 => w.dp_store(2, w.cpu.a & 255, 1),
                    6 => w.dp_store(5, w.cpu.a & 255, 1),
                    _ => unreachable!(),
                }
                if i == 6 {
                    Expand(0)
                } else {
                    Temp(i + 1)
                }
            }
            Expand(i) => {
                let mut next = Expand(i + 1);
                match i {
                    0 => w.width(0x31, true),
                    1 | 5 | 11 => {
                        w.cpu.y = w.dp_read(if i == 1 { 12 } else { 10 }, 2);
                        w.flags(w.cpu.y, 2);
                    }
                    2 => {
                        let value = w.dp_read(0, 2);
                        w.load(value, 2);
                    }
                    3 => {
                        let rhs = w.item_absolute_y(0xd21d, 2);
                        w.adc(rhs);
                    }
                    4 => {
                        w.cpu.x = w.dp_read(6, 2);
                        w.flags(w.cpu.x, 2);
                    }
                    6 => {
                        w.drain(6);
                        w.push(w.cpu.a, 2);
                    }
                    7 | 12 => conversion = Some((true, next)),
                    8 => {
                        w.drain(12);
                        let value = w.pull(2);
                        w.load(value, 2);
                    }
                    9 => w.clear_carry(),
                    10 => {
                        w.drain(16);
                        w.adc(0x180);
                    }
                    13 | 14 => w.dp_increment(12, 2, true),
                    15 => w.dp_store(6, w.cpu.x, 2),
                    16 => w.width(0x30, false),
                    17 => {
                        w.rts();
                        next = self.sheet_return.take().unwrap();
                        returned = true;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Aux(i) => {
                let mut next = Aux(i + 1);
                match i {
                    0 | 8 | 16 => w.item_y(
                        match i {
                            0 => 15,
                            8 => 88,
                            _ => 5,
                        },
                        1,
                    ),
                    1 | 9 | 17 => decompression = Some((i == 9, next)),
                    2 | 10 | 18 => w.width(0x30, true),
                    3 | 11 | 19 => {
                        let value = w.dp_read(0, 2);
                        w.load(value, 2);
                    }
                    4 | 12 | 22 => w.item_y(
                        match i {
                            4 => 16,
                            12 => 32,
                            _ => 2,
                        },
                        2,
                    ),
                    5 | 13 | 23 => w.item_x(
                        match i {
                            5 => 0x2340,
                            13 => 0x2540,
                            _ => 0x2dc0,
                        },
                        2,
                    ),
                    6 | 14 | 24 => conversion = Some((false, next)),
                    7 | 15 | 25 => w.width(0x30, false),
                    20 => w.clear_carry(),
                    21 => {
                        w.drain(16);
                        w.adc(0x480);
                    }
                    26 => {
                        w.rts();
                        next = self.sheet_return.take().unwrap();
                        returned = true;
                    }
                    _ => unreachable!(),
                }
                next
            }
            _ => unreachable!(),
        };
        if let Some((entry, next)) = sheet {
            assert!(self.sheet_return.is_none());
            self.sheet_return = Some(next);
            w.item_call();
            self.phase = entry;
        }
        if let Some((high, next)) = conversion {
            assert!(self.child_return.is_none());
            self.child_return = Some(next);
            let child = ItemConversion::new(high);
            w.item_call();
            w.cpu.pc = child.routine() as u16;
            self.conversion = Some(child);
            self.phase = Converting;
        }
        if let Some((sprite, next)) = decompression {
            assert!(self.child_return.is_none());
            self.child_return = Some(next);
            let child = NativeDecompression::new(sprite);
            w.item_call();
            w.cpu.pc = child.routine() as u16;
            self.decompression = Some(child);
            self.phase = Decompressing;
        }
        if !matches!(self.phase, Complete | Converting | Decompressing) {
            if returned {
                assert_eq!(w.cpu.pc, self.pc(), "item graphics caller return stack");
            }
            w.cpu.pc = self.pc();
        }
        Some(w.cycles)
    }
}
impl<B: WorkBus> Work<'_, B> {
    pub(super) fn item_x(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        self.cpu.x = value;
        self.flags(value, width);
    }
    pub(super) fn item_y(&mut self, value: u16, width: u8) {
        self.drain(8 * u32::from(width));
        self.cpu.y = value;
        self.flags(value, width);
    }
    pub(super) fn item_call(&mut self) {
        self.drain(16);
        self.drain(6);
        self.push(self.cpu.pc + 2, 2);
    }
    fn item_absolute_y(&mut self, base: u16, width: u8) -> u16 {
        self.drain(16);
        if self.cpu.p & 0x10 == 0 || (base & 255) + self.cpu.y > 255 {
            self.drain(6);
        }
        self.read(
            self.absolute(base).wrapping_add(u32::from(self.cpu.y)),
            width,
        )
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
    fn retained_item_caller_matches_every_source_instruction_and_nested_return() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for address in 0xcf80..0xd231 {
            if let Some(value) = table_byte(address) {
                assert_eq!(value, rom[address as usize - 0x8000], "table{address:06x}");
            }
        }
        for db in [0, 1] {
            for odd in [false, true] {
                for (v, h) in [(238, 858), (224, 1338), (261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 7)) as u8;
                    }
                    machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                    let mut work = ItemGraphics::default();
                    let mut native = NativeNmiContext {
                        a: 0xd2,
                        x: 0xfc,
                        y: 0xff,
                        d: 0,
                        db,
                        bank: 0,
                        pc: 0xd231,
                        sp: 0x1fc,
                        p: 0xb0,
                    };
                    machine.cpu.a = native.a;
                    machine.cpu.x = native.x;
                    machine.cpu.y = native.y;
                    machine.cpu.db = native.db;
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
                        let label = format!("db={db} phase={:?} pc={:04x}", work.phase, native.pc);
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
                        (0, 0x805a, 0x1ff, db)
                    );
                    assert_eq!(ram, source.snes().ram);
                    assert!(
                        clock.field_index() > 10,
                        "complete item graphics work must cross its hardware fields"
                    );
                }
            }
        }
    }
}
