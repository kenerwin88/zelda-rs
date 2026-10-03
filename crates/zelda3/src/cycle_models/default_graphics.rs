//! Native default graphics caller: live raw planes, retained decompression,
//! and PPU-port publications through every nested call and return.
use super::{
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

pub(crate) const SOURCES: &[(bool, usize, u32)] = &[
    (true, 0, 0x10f000),
    (true, 105, 0x18af0d),
    (true, 106, 0x18b520),
    (true, 107, 0x18b953),
];
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    for &(_, sheet, source) in SOURCES {
        for (i, table) in [0xcff3, 0xd0d2, 0xd1b1].into_iter().enumerate() {
            if address == table + sheet as u32 {
                return Some((source >> (16 - i * 8)) as u8);
            }
        }
    }
    None
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Main(u8),
    Upload(u8),
    Decompressing,
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DefaultGraphics {
    phase: Phase,
    caller_return: Option<u8>,
    decompression: Option<NativeDecompression>,
}
impl Default for DefaultGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Main(0),
            caller_return: None,
            decompression: None,
        }
    }
}
impl DefaultGraphics {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Upload(_) => 0xe33b,
            Phase::Decompressing => self.decompression.as_ref().unwrap().routine(),
            _ => 0xe2d0,
        }
    }
    fn pc(&self) -> u16 {
        match self.phase {
            Phase::Main(i) => [
                0xe2d0, 0xe2d1, 0xe2d2, 0xe2d3, 0xe2d5, 0xe2d8, 0xe2db, 0xe2dd, 0xe2e0, 0xe2e2,
                0xe2e5, 0xe2e7, 0xe2e9, 0xe2ec, 0xe2ef, 0xe2f1, 0xe2f3, 0xe2f5, 0xe2f8, 0xe2f9,
                0xe2fb, 0xe2fe, 0xe300, 0xe302, 0xe304, 0xe305, 0xe306, 0xe308, 0xe30a, 0xe30c,
                0xe30f, 0xe311, 0xe313, 0xe314, 0xe316, 0xe319, 0xe31b, 0xe31c, 0xe31d, 0xe31f,
                0xe320, 0xe322, 0xe325, 0xe328, 0xe32a, 0xe32c, 0xe32f, 0xe331, 0xe334, 0xe336,
                0xe339, 0xe33a,
            ][usize::from(i)],
            Phase::Upload(i) => [
                0xe33b, 0xe33e, 0xe340, 0xe343, 0xe345, 0xe348, 0xe34a, 0xe34c, 0xe34e, 0xe351,
                0xe353, 0xe355, 0xe357, 0xe35a, 0xe35c, 0xe35e, 0xe360, 0xe363, 0xe365, 0xe367,
                0xe368, 0xe36a, 0xe36c,
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
        if self.phase == Decompressing {
            let child = self.decompression.as_mut().unwrap();
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.decompression = None;
                self.phase = Upload(1);
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
        let mut returned = false;
        self.phase = match self.phase {
            Main(i) => {
                let mut next = Main(i + 1);
                match i {
                    0 | 1 => {
                        w.drain(6);
                        w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
                    }
                    2 | 50 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    3 => w.immediate(0x80, 1),
                    4 => {
                        w.drain(16);
                        w.write_io(0x2115, w.cpu.a & 255, 1);
                    }
                    5 | 7 | 9 => {
                        w.drain(16);
                        let v = w.read([0xcff3, 0xd0d2, 0xd1b1][usize::from((i - 5) / 2)], 1);
                        w.load(v, 1);
                    }
                    6 | 8 | 10 => w.dp_store(u16::from((10 - i) / 2), w.cpu.a & 255, 1),
                    11 => w.width(0x20, true),
                    12 => w.immediate(0x4000, 2),
                    13 | 42 => {
                        w.drain(16);
                        w.write_io(0x2116, w.cpu.a, 2);
                    }
                    14 => w.item_y(0x40, 1),
                    15 | 27 => w.item_x(14, 1),
                    16 | 28 => {
                        let v = w.indirect_read(0, 2);
                        w.load(v, 2);
                    }
                    17 | 34 => {
                        w.drain(16);
                        w.write_io(0x2118, w.cpu.a, 2);
                    }
                    18 | 32 => w.swap_bytes(),
                    19 => {
                        let v = w.indirect_read(0, 2);
                        w.load(w.cpu.a | v, 2);
                    }
                    20 | 29 => w.mask(255, 2),
                    21 => {
                        w.drain(8);
                        w.drain(6);
                        w.write(0xbf + u32::from(w.cpu.x), w.cpu.a, 2);
                    }
                    22 | 23 | 35 => w.dp_increment(0, 2, true),
                    24 | 25 | 36 | 37 => {
                        w.drain(6);
                        w.cpu.x = u16::from((w.cpu.x as u8).wrapping_sub(1));
                        w.flags(w.cpu.x, 1);
                    }
                    26 | 38 => {
                        if w.branch(w.cpu.p & NEGATIVE == 0) {
                            next = Main(if i == 26 { 16 } else { 28 });
                        }
                    }
                    30 => w.dp_store(0xbd, w.cpu.a, 2),
                    31 => {
                        w.drain(8);
                        w.drain(6);
                        let v = w.read(0xbf + u32::from(w.cpu.x), 2);
                        w.load(w.cpu.a | v, 2);
                    }
                    33 => {
                        let v = w.dp_read(0xbd, 2);
                        w.load(w.cpu.a | v, 2);
                    }
                    39 => {
                        w.drain(6);
                        w.cpu.y = u16::from((w.cpu.y as u8).wrapping_sub(1));
                        w.flags(w.cpu.y, 1);
                    }
                    40 => {
                        if w.branch(w.cpu.p & ZERO == 0) {
                            next = Main(15);
                        }
                    }
                    41 => w.immediate(0x7000, 2),
                    43 => w.width(0x20, false),
                    44 | 46 | 48 => w.item_y(
                        match i {
                            44 => 106,
                            46 => 107,
                            _ => 105,
                        },
                        1,
                    ),
                    45 | 47 | 49 => {
                        assert!(self.caller_return.is_none());
                        self.caller_return = Some(i + 1);
                        w.item_call();
                        next = Upload(0);
                    }
                    51 => {
                        w.rtl();
                        next = Complete;
                    }
                    _ => unreachable!(),
                }
                next
            }
            Upload(i) => {
                let mut next = Upload(i + 1);
                match i {
                    0 => {
                        let child = NativeDecompression::new(true);
                        w.item_call();
                        w.cpu.pc = child.routine() as u16;
                        self.decompression = Some(child);
                        next = Decompressing;
                    }
                    1 => w.width(0x30, true),
                    2 => w.item_x(255, 2),
                    3 | 7 | 11 | 15 => {
                        let v = w.indirect_read(0, 2);
                        w.load(v, 2);
                    }
                    4 | 8 | 12 | 16 => {
                        w.drain(16);
                        w.write_io(0x2118, w.cpu.a, 2);
                    }
                    5 | 6 | 9 | 10 | 13 | 14 | 17 | 18 => w.dp_increment(0, 2, true),
                    19 => w.decrement_x(),
                    20 => {
                        if w.branch(w.cpu.p & NEGATIVE == 0) {
                            next = Upload(3);
                        }
                    }
                    21 => w.width(0x30, false),
                    22 => {
                        w.rts();
                        next = Main(self.caller_return.take().unwrap());
                        returned = true;
                    }
                    _ => unreachable!(),
                }
                next
            }
            _ => unreachable!(),
        };
        if !matches!(self.phase, Complete | Decompressing) {
            if returned {
                assert_eq!(w.cpu.pc, self.pc(), "default graphics upload return stack");
            }
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
    fn retained_default_graphics_matches_live_planes_ppu_ports_and_returns() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for address in 0xcff3..0xd231 {
            if let Some(v) = table_byte(address) {
                assert_eq!(v, rom[address as usize - 0x8000]);
            }
        }
        for db in [0, 12] {
            for odd in [false, true] {
                for (v, h) in [(101, 850), (224, 1338), (261, 1350)] {
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    for (i, b) in machine.ram.iter_mut().enumerate() {
                        *b = (i ^ (i >> 7)) as u8;
                    }
                    machine.ram[0x1fd..0x200].copy_from_slice(&[0x1d, 0xc2, 12]);
                    machine.ppu.forced_blank = true;
                    machine.ppu.vram.fill(0x55aa);
                    let mut ppu = machine.ppu.clone();
                    let mut work = DefaultGraphics::default();
                    let mut native = NativeNmiContext {
                        a: 0xab08,
                        x: 0xfc,
                        y: 0xff,
                        d: 0,
                        db,
                        bank: 0,
                        pc: 0xe2d0,
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
                        let label = format!("db={db} phase={:?} pc={:04x}", work.phase, native.pc);
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
                    assert_eq!(
                        (native.bank, native.pc, native.sp, native.db),
                        (12, 0xc21e, 0x1ff, db)
                    );
                    assert_eq!(ram, source.snes().ram);
                    assert_eq!(ppu.vram, source.snes().ppu.vram);
                    assert_eq!(
                        (
                            ppu.vram_pointer,
                            ppu.vram_increment,
                            ppu.vram_increment_on_high
                        ),
                        (
                            source.snes().ppu.vram_pointer,
                            source.snes().ppu.vram_increment,
                            source.snes().ppu.vram_increment_on_high
                        )
                    );
                }
            }
        }
    }
}
