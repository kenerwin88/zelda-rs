//! Retained follower sheet selection, decompression and conversion caller.
use super::{
    item_graphics::ItemConversion,
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

pub(crate) const SOURCES: &[(bool, usize, u32)] = &[
    (true, 100, 0x189dd2),
    (true, 102, 0x18a4ed),
    (true, 89, 0x17f06c),
    (true, 88, 0x17ed58),
    (true, 101, 0x18a03f),
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    for &(_, sheet, source) in SOURCES {
        for (i, table) in [0xcff3, 0xd0d2, 0xd1b1].into_iter().enumerate() {
            if address == table + sheet as u32 {
                return Some((source >> (16 - i * 8)) as u8);
            }
        }
    }
    let offset = address.checked_sub(0xd407).filter(|&v| v < 28)?;
    let offsets = [
        0u16, 0x600, 0x300, 0x300, 0x300, 0, 0, 0x900, 0x600, 0x600, 0x900, 0x900, 0x600, 0x900,
    ];
    Some(offsets[(offset / 2) as usize].to_le_bytes()[(offset & 1) as usize])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Decompressing { next: u8 },
    Converting,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FollowerGraphics {
    phase: Phase,
    decompression: Option<NativeDecompression>,
    conversion: Option<ItemConversion>,
}
impl Default for FollowerGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            decompression: None,
            conversion: None,
        }
    }
}
impl FollowerGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Decompressing { .. } => self.decompression.as_ref().unwrap().routine(),
            Phase::Converting => self.conversion.as_ref().unwrap().routine(),
            _ => 0xd423,
        }
    }
    fn caller_pc(index: u8) -> u16 {
        [
            0xd423, 0xd424, 0xd425, 0xd426, 0xd428, 0xd42c, 0xd42e, 0xd430, 0xd432, 0xd436, 0xd438,
            0xd43a, 0xd43c, 0xd43e, 0xd440, 0xd442, 0xd445, 0xd447, 0xd44a, 0xd44c, 0xd450, 0xd453,
            0xd454, 0xd455, 0xd457, 0xd458, 0xd45c, 0xd45f, 0xd462, 0xd465, 0xd467, 0xd468,
        ][usize::from(index)]
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
                    assert_eq!(
                        cpu.pc,
                        Self::caller_pc(next),
                        "follower decompression return stack"
                    );
                }
                return cycles;
            }
            Converting => {
                let child = self.conversion.as_mut().unwrap();
                let cycles = child.step(cpu, clock, bus);
                if child.complete() {
                    self.conversion = None;
                    self.phase = Caller(29);
                    assert_eq!(
                        cpu.pc,
                        Self::caller_pc(29),
                        "follower conversion return stack"
                    );
                }
                return cycles;
            }
            Caller(_) => {}
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
            2 | 30 => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
            }
            3 | 7 | 11 | 14 | 16 => w.item_y(
                match i {
                    3 => 100,
                    7 => 102,
                    11 => 89,
                    14 => 88,
                    _ => 101,
                },
                1,
            ),
            4 | 8 | 19 => {
                w.drain(24);
                let width = if i == 19 { 2 } else { 1 };
                let v = w.read(0x7ef3cc, width);
                w.load(v, width);
            }
            5 | 9 | 12 => w.compare_immediate(
                match i {
                    5 => 1,
                    9 => 9,
                    _ => 12,
                },
                1,
            ),
            6 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    next = 15;
                }
            }
            10 | 13 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    next = 15;
                }
            }
            15 | 17 => {
                let child = if i == 15 {
                    NativeDecompression::sprite_secondary()
                } else {
                    NativeDecompression::new(true)
                };
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.decompression = Some(child);
                self.phase = Decompressing { next };
                return Some(w.cycles);
            }
            18 => w.width(0x30, true),
            20 => w.mask(255, 2),
            21 => w.shift_left(2),
            22 => w.tax_word(),
            23 => {
                let v = w.dp_read(0, 2);
                w.load(v, 2);
            }
            24 => w.clear_carry(),
            25 => {
                w.drain(24);
                let v = w.read(0xd407 + u32::from(w.cpu.x), 2);
                w.adc(v);
            }
            26 => w.item_y(32, 2),
            27 => w.item_x(0x2940, 2),
            28 => {
                let child = ItemConversion::new(false);
                w.item_call();
                w.cpu.pc = child.routine() as u16;
                self.conversion = Some(child);
                self.phase = Converting;
                return Some(w.cycles);
            }
            29 => w.width(0x30, false),
            31 => {
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
    fn retained_follower_caller_matches_all_selections_and_nested_returns() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for address in 0xcff3..0xd423 {
            if let Some(value) = table_byte(address) {
                assert_eq!(value, rom[address as usize - 0x8000], "table{address:06x}");
            }
        }
        for follower in 0..14 {
            for db in [0, 0x7f] {
                for odd in [false, true] {
                    for (v, h) in [(238, 858), (224, 1338), (261, 1350)] {
                        let mut machine = Snes::new();
                        machine.cart.load(CartType::LoRom, &rom, 0x2000);
                        for (i, b) in machine.ram.iter_mut().enumerate() {
                            *b = (i ^ (i >> 7)) as u8;
                        }
                        machine.ram[0xf3cc] = follower;
                        machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                        let mut work = FollowerGraphics::default();
                        let mut native = NativeNmiContext {
                            a: 0x87d4,
                            x: 0xfc,
                            y: 0xff,
                            d: 0,
                            db,
                            bank: 0,
                            pc: 0xd423,
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
                                "follower={follower} db={db} phase={:?} pc={:04x}",
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
                            assert!(count < 100000);
                        }
                        assert_eq!(
                            (native.bank, native.pc, native.sp, native.db),
                            (0, 0x805a, 0x1ff, db)
                        );
                        assert_eq!(ram, source.snes().ram);
                        assert!(clock.field_index() >= 3);
                    }
                }
            }
        }
    }
}
