//! Retained animated-item caller, including both sheet decoders and both
//! conversions. Child returns consume the same live stack and hardware clock.
use super::{
    item_graphics::ItemConversion,
    native_decompress::NativeDecompression,
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Decompression(NativeDecompression),
    Conversion(ItemConversion),
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnimatedItemGraphics {
    phase: Phase,
    child_return: Option<u8>,
}

impl Default for AnimatedItemGraphics {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            child_return: None,
        }
    }
}

impl AnimatedItemGraphics {
    const CALLER_PCS: [u16; 38] = [
        0xd4ed, 0xd4ee, 0xd4ef, 0xd4f0, 0xd4f1, 0xd4f3, 0xd4f5, 0xd4f7, 0xd4f9, 0xd4fb, 0xd4fd,
        0xd4ff, 0xd501, 0xd503, 0xd505, 0xd507, 0xd50a, 0xd50c, 0xd50f, 0xd510, 0xd512, 0xd515,
        0xd516, 0xd517, 0xd519, 0xd51c, 0xd51e, 0xd521, 0xd524, 0xd525, 0xd528, 0xd529, 0xd52a,
        0xd52d, 0xd530, 0xd533, 0xd535, 0xd536,
    ];

    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }

    pub(crate) fn routine(&self) -> u32 {
        match &self.phase {
            Phase::Decompression(work) => work.routine(),
            Phase::Conversion(work) => work.routine(),
            _ => 0xd4ed,
        }
    }

    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        let (cycles, child_complete) = match &mut self.phase {
            Phase::Complete => return None,
            Phase::Decompression(work) => {
                let cycles = work.step(cpu, clock, bus);
                (cycles, work.complete())
            }
            Phase::Conversion(work) => {
                let cycles = work.step(cpu, clock, bus);
                (cycles, work.complete())
            }
            Phase::Caller(i) => {
                let i = *i;
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.d),
                    (0, Self::CALLER_PCS[i as usize], 0)
                );
                let mut w = Work {
                    cpu,
                    clock,
                    bus,
                    cycles: 0,
                };
                w.fetch();
                let mut next = i + 1;
                let mut child = None;
                match i {
                    0 | 1 | 3 | 28 => {
                        w.drain(6);
                        let (value, width) = match i {
                            0 => (u16::from(w.cpu.db), 1),
                            1 => (u16::from(w.cpu.bank), 1),
                            3 => (w.cpu.a & 255, 1),
                            _ => (w.cpu.a, 2),
                        };
                        w.push(value, width);
                    }
                    2 | 36 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    4 | 9 | 14 | 16 => w.item_y(
                        match i {
                            4 => 0x5d,
                            9 => 0x5c,
                            14 => 0x5b,
                            _ => 0x5a,
                        },
                        1,
                    ),
                    5 | 7 | 10 | 12 => w.compare_immediate(
                        match i {
                            5 => 0x23,
                            7 => 0x37,
                            10 => 0x0c,
                            _ => 0x24,
                        },
                        1,
                    ),
                    6 | 8 | 11 | 13 => {
                        let taken = w.cpu.p & if i == 6 || i == 11 { ZERO } else { CARRY } != 0;
                        if w.branch(taken) {
                            next = 15;
                        }
                    }
                    15 | 17 => {
                        let work = if i == 15 {
                            NativeDecompression::sprite_secondary()
                        } else {
                            NativeDecompression::new(true)
                        };
                        w.item_call();
                        w.cpu.pc = work.routine() as u16;
                        child = Some(Phase::Decompression(work));
                    }
                    18 | 30 => {
                        w.drain(12);
                        let value = w.pull(if i == 18 { 1 } else { 2 });
                        w.load(value, if i == 18 { 1 } else { 2 });
                    }
                    19 => w.width(0x21, true),
                    20 => w.mask(0xff, 2),
                    21 => {
                        w.drain(6);
                        let old = w.cpu.a;
                        w.cpu.a = old.wrapping_shl(1);
                        w.cpu.p = (w.cpu.p & !CARRY) | u8::from(old & 0x8000 != 0);
                        w.flags(w.cpu.a, 2);
                    }
                    22 => {
                        w.drain(6);
                        w.cpu.x = w.cpu.a & 255;
                        w.flags(w.cpu.x, 1);
                    }
                    23 => {
                        let value = w.dp_read(0, 2);
                        w.load(value, 2);
                    }
                    24 => {
                        w.drain(16);
                        if (0xd469u16 & 255) + w.cpu.x > 255 {
                            w.drain(6);
                        }
                        let value = w.read(w.absolute(0xd469) + u32::from(w.cpu.x), 2);
                        w.adc(value);
                    }
                    25 => w.width(0x10, true),
                    26 => w.item_x(0x2d40, 2),
                    27 | 33 => w.item_y(2, 2),
                    29 | 34 => {
                        let work = ItemConversion::new(true);
                        w.item_call();
                        w.cpu.pc = work.routine() as u16;
                        child = Some(Phase::Conversion(work));
                    }
                    31 => w.clear_carry(),
                    32 => {
                        w.drain(16);
                        w.adc(0x180);
                    }
                    35 => w.width(0x30, false),
                    37 => w.rtl(),
                    _ => unreachable!(),
                }
                self.phase = if i == 37 {
                    Phase::Complete
                } else if let Some(child) = child {
                    assert!(self.child_return.replace(next).is_none());
                    child
                } else {
                    w.cpu.pc = Self::CALLER_PCS[next as usize];
                    Phase::Caller(next)
                };
                return Some(w.cycles);
            }
        };
        if child_complete {
            let next = self.child_return.take().unwrap();
            assert_eq!(
                (cpu.bank, cpu.pc),
                (0, Self::CALLER_PCS[next as usize]),
                "animated item child return stack"
            );
            self.phase = Phase::Caller(next);
        }
        cycles
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
    fn animated_item_caller_matches_every_source_instruction_and_live_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .expect("animated item source fixture");
        for address in 0xd469..0xd4db {
            assert_eq!(
                crate::zelda_rtl::native_animated_item_table_byte(address),
                Some(rom[address as usize - 0x8000])
            );
        }
        for address in 0xcf80..0xd21d {
            if let Some(value) = super::super::graphics_sources::table_byte(address) {
                assert_eq!(value, rom[address as usize - 0x8000]);
            }
        }
        let mut total_instructions = 0;
        for tile in 0..57u8 {
            for variant in 0..if tile == 0x14 { 3 } else { 2 } {
                let odd = variant == 1;
                // The additional fixture uses the cold-route caller's live
                // entry and return stack. These are test inputs, not a runtime
                // phase seed. This fixture isolates caller work; the live
                // continuation's NMI composition remains pending.
                let route = variant == 2;
                let sp = if route { 0x1e1 } else { 0x1dc };
                let db = if route {
                    9
                } else if odd {
                    0x0d
                } else {
                    0
                };
                let return_stack = if route {
                    [0xa3, 0x87, 9]
                } else {
                    [0x0a, 0xb0, 0x0d]
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, byte) in machine.ram.iter_mut().enumerate() {
                    *byte = (i ^ (i >> 7) ^ usize::from(tile)) as u8;
                }
                machine.ram[sp as usize + 1..sp as usize + 4].copy_from_slice(&return_stack);
                let mut native = NativeNmiContext {
                    a: (if route { 0x1b00 } else { 0xad00 }) | u16::from(tile),
                    x: if route { 0xff } else { 0xfc },
                    y: if route { 0x12 } else { 0xff },
                    d: 0,
                    db,
                    bank: 0,
                    pc: 0xd4ed,
                    sp,
                    p: if route { 0xb0 } else { 0xf1 },
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
                let (v, h) = if route {
                    (14, 966)
                } else if odd {
                    (261, 1350)
                } else {
                    (224, 1338)
                };
                let field = if route { 4582 } else { 0 };
                let mut clock = CpuMasterTimeline::at_raster(
                    field,
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
                let mut work = AnimatedItemGraphics::default();
                let mut count = 0;
                while !work.complete() {
                    let label = format!("tile={tile:02x} variant={variant} pc={:04x}", native.pc);
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
                    (
                        return_stack[2],
                        u16::from_le_bytes([return_stack[0], return_stack[1]]) + 1,
                        sp + 3,
                        db
                    )
                );
                assert_eq!(ram, source.snes().ram);
                assert!(
                    clock.field_index() >= field + 3,
                    "decoder must cross hardware fields"
                );
                total_instructions += count;
            }
        }
        println!("animated item source comparison: 115 cases, {total_instructions} instructions");
    }
}
