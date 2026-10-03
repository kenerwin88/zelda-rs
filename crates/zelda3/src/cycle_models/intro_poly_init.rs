//! Retained intro poly-thread initializer. The upload, thread stack copy,
//! palette and actor stores retire on the caller's live bus and clock.
use super::native_work::WorkBus;
use super::{
    common_sprite_graphics::CommonSpriteGraphics,
    native_work::Work,
    nmi_stack::{NativeNmiContext, NEGATIVE},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Caller(u8),
    Graphics(u8),
    Common,
    Helper(u8),
    Bootstrap(u8),
    Palette(u8),
    Complete,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroPolyInit {
    phase: Phase,
    common: Option<CommonSpriteGraphics>,
}
impl Default for IntroPolyInit {
    fn default() -> Self {
        Self {
            phase: Phase::Caller(0),
            common: None,
        }
    }
}
/// Immutable bootstrap stack image; these are data consumed by MVN, not an
/// injected CPU context. The NMI thread switch subsequently pulls this stack.
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0x09f7f7 => Some(0),
        0x09f809 => Some(9),
        0x09f810..=0x09f81c => {
            Some([9, 0, 0x1f, 0, 0, 0, 0, 0, 0, 0x30, 0x1d, 0xf8, 9][(address - 0x09f810) as usize])
        }
        _ => None,
    }
}
impl IntroPolyInit {
    pub(crate) fn complete(&self) -> bool {
        self.phase == Phase::Complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.phase {
            Phase::Common => self.common.as_ref().unwrap().routine(),
            Phase::Graphics(_) => 0xe384,
            Phase::Bootstrap(_) => 0x09f7de,
            Phase::Palette(_) => 0x0cc3bd,
            Phase::Helper(_) => 0x0cc36f,
            _ => 0x0cc33c,
        }
    }
    fn location(phase: Phase) -> (u8, u16) {
        use Phase::*;
        match phase {
            Caller(i) => (
                12,
                [
                    0xc33c, 0xc33e, 0xc341, 0xc345, 0xc348, 0xc34a, 0xc34d, 0xc350, 0xc353, 0xc355,
                    0xc358, 0xc35b, 0xc35e, 0xc360, 0xc363, 0xc365, 0xc368, 0xc36a, 0xc36c, 0xc36e,
                ][i as usize],
            ),
            Graphics(i) => (
                0,
                [
                    0xe384, 0xe385, 0xe386, 0xe387, 0xe389, 0xe38c, 0xe38f, 0xe391, 0xe394, 0xe397,
                    0xe398,
                ][i as usize],
            ),
            Bootstrap(i) => (
                9,
                [
                    0xf7de, 0xf7df, 0xf7e0, 0xf7e2, 0xf7e3, 0xf7e4, 0xf7e5, 0xf7e8, 0xf7ec, 0xf7ef,
                    0xf7f2, 0xf7f5, 0xf7f8, 0xf7fb, 0xf7fe, 0xf801, 0xf804, 0xf807, 0xf80a, 0xf80b,
                    0xf80c, 0xf80d, 0xf80e, 0xf80f,
                ][i as usize],
            ),
            Helper(i) => (
                12,
                [
                    0xc36f, 0xc373, 0xc376, 0xc378, 0xc37a, 0xc37c, 0xc37f, 0xc381, 0xc384, 0xc387,
                    0xc38a, 0xc38c, 0xc38f, 0xc391, 0xc394, 0xc396, 0xc399, 0xc39c, 0xc39f, 0xc3a2,
                    0xc3a4, 0xc3a7, 0xc3aa, 0xc3ad, 0xc3b0, 0xc3b3, 0xc3b6, 0xc3b9, 0xc3ba, 0xc3bc,
                ][i as usize],
            ),
            Palette(i) => (
                12,
                match i {
                    0 => 0xc3bd,
                    1..=16 => 0xc3bf + u16::from(i - 1) * 4,
                    17 => 0xc3ff,
                    18 => 0xc401,
                    19 => 0xc403,
                    _ => unreachable!(),
                },
            ),
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
        if self.phase == Common {
            let child = self.common.as_mut().unwrap();
            let cycles = child.step(cpu, clock, bus);
            if child.complete() {
                self.common = None;
                self.phase = Graphics(9);
            }
            return cycles;
        }
        assert_eq!((cpu.bank, cpu.pc), Self::location(self.phase));
        assert_eq!(cpu.d, 0);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match self.phase {
            Caller(i) => {
                match i {
                    0 | 4 | 8 | 12 | 14 | 16 => w.immediate(
                        match i {
                            0 => 8,
                            4 | 12 => 1,
                            8 => 0,
                            14 => 2,
                            _ => 15,
                        },
                        1,
                    ),
                    1 => store_abs(&mut w, 0xaa4, 1),
                    2 => {
                        call_long(&mut w, 0, 0xe384);
                        self.phase = Graphics(0);
                        return Some(w.cycles);
                    }
                    3 => {
                        call_short(&mut w, 0xc36f);
                        self.phase = Helper(0);
                        return Some(w.cycles);
                    }
                    5..=7 => store_abs(&mut w, 0x1e10 + u16::from(i - 5), 1),
                    9..=11 => store_abs(&mut w, 0x1e18 + u16::from(i - 9), 1),
                    13 => store_abs(&mut w, 0x1e14, 1),
                    15 => store_abs(&mut w, 0x1e1c, 1),
                    17 => w.dp_store(0x13, w.cpu.a & 255, 1),
                    18 => w.dp_increment(0x11, 1, true),
                    19 => {
                        w.rtl();
                        self.phase = Complete;
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Caller(i + 1)
            }
            Graphics(i) => {
                match i {
                    0 | 1 => {
                        w.drain(6);
                        w.push(u16::from(if i == 0 { w.cpu.db } else { w.cpu.bank }), 1);
                    }
                    2 | 9 => {
                        w.drain(12);
                        w.cpu.db = w.pull(1) as u8;
                        w.flags(u16::from(w.cpu.db), 1);
                    }
                    3 | 6 => w.immediate(if i == 3 { 0x80 } else { 0x44 }, 1),
                    4 | 5 | 7 => {
                        w.drain(16);
                        w.write_io(
                            match i {
                                4 => 0x2115,
                                5 => 0x2116,
                                _ => 0x2117,
                            },
                            if i == 5 { 0 } else { w.cpu.a & 255 },
                            1,
                        );
                    }
                    8 => {
                        call_short(&mut w, 0xe6b7);
                        self.common = Some(Default::default());
                        self.phase = Common;
                        return Some(w.cycles);
                    }
                    10 => {
                        w.rtl();
                        self.phase = Caller(3);
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Graphics(i + 1)
            }
            Bootstrap(i) => {
                match i {
                    0 | 1 | 3 | 4 | 5 => {
                        w.drain(6);
                        let (v, n) = match i {
                            0 => (u16::from(w.cpu.p), 1),
                            1 => (u16::from(w.cpu.db), 1),
                            3 => (w.cpu.a, 2),
                            4 => (w.cpu.x, 2),
                            _ => (w.cpu.y, 2),
                        };
                        w.push(v, n);
                    }
                    2 => w.width(0x30, true),
                    6 | 10 | 12 | 14 => w.immediate(
                        match i {
                            6 => 0,
                            10 => 0xfd,
                            12 => 0x1f31,
                            _ => 12,
                        },
                        2,
                    ),
                    7 => {
                        w.drain(24);
                        w.write(0x1f00, w.cpu.a, 2);
                    }
                    8 | 15 => w.item_x(if i == 8 { 0x1f00 } else { 0xf810 }, 2),
                    9 | 16 => w.item_y(if i == 9 { 0x1f02 } else { 0x1f32 }, 2),
                    11 | 17 => {
                        // One MVN byte is one interruptible instruction. Keep
                        // its live A/X/Y counters and the repeated instruction.
                        w.drain(8);
                        let source_bank = w.read(0x090000 | u32::from(w.cpu.pc + 2), 1);
                        assert_eq!(source_bank, if i == 17 { 9 } else { 0 });
                        let value =
                            w.read((if i == 17 { 0x90000 } else { 0 }) | u32::from(w.cpu.x), 1);
                        w.write(u32::from(w.cpu.y), value, 1);
                        w.drain(12);
                        w.cpu.db = 0;
                        w.cpu.x = w.cpu.x.wrapping_add(1);
                        w.cpu.y = w.cpu.y.wrapping_add(1);
                        w.cpu.a = w.cpu.a.wrapping_sub(1);
                        if w.cpu.a != 0xffff {
                            return Some(w.cycles);
                        }
                    }
                    13 => store_abs(&mut w, 0x1f0a, 2),
                    18..=22 => {
                        w.drain(12);
                        let n = if i <= 20 { 2 } else { 1 };
                        let v = w.pull(n);
                        match i {
                            18 => {
                                w.cpu.y = v;
                                w.flags(v, 2)
                            }
                            19 => {
                                w.cpu.x = v;
                                w.flags(v, 2)
                            }
                            20 => w.load(v, 2),
                            21 => {
                                w.cpu.db = v as u8;
                                w.flags(v, 1)
                            }
                            _ => {
                                w.cpu.p = v as u8;
                                if w.cpu.p & 0x10 != 0 {
                                    w.cpu.x &= 255;
                                    w.cpu.y &= 255;
                                }
                            }
                        }
                    }
                    23 => {
                        w.rtl();
                        self.phase = Helper(1);
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Bootstrap(i + 1)
            }
            Helper(i) => {
                match i {
                    0 => {
                        call_long(&mut w, 9, 0xf7de);
                        self.phase = Bootstrap(0);
                        return Some(w.cycles);
                    }
                    1 => {
                        call_short(&mut w, 0xc3bd);
                        self.phase = Palette(0);
                        return Some(w.cycles);
                    }
                    2 | 4 | 6 | 10 | 12 | 14 => w.immediate(
                        match i {
                            2 => 0x90,
                            4 => 255,
                            6 => 32,
                            10 => 0xa0,
                            12 => 0x60,
                            _ => 1,
                        },
                        1,
                    ),
                    3 => w.dp_store(0xff, w.cpu.a & 255, 1),
                    5 | 7..=9 | 11 | 13 | 15..=18 => store_abs(
                        &mut w,
                        match i {
                            5 => 0x1f02,
                            7..=9 => 0x1f06 + u16::from(i - 7),
                            11 => 0x1f04,
                            13 => 0x1f05,
                            15 => 0x1f01,
                            16 => 0x1f03,
                            17 => 0x12a,
                            _ => 0x1f00,
                        },
                        1,
                    ),
                    19 => w.item_x(15, 1),
                    20..=26 => {
                        w.drain(16);
                        w.drain(6);
                        w.write(
                            w.absolute(0x1e00 + u16::from(i - 20) * 16) + u32::from(w.cpu.x),
                            0,
                            1,
                        );
                    }
                    27 => {
                        w.drain(6);
                        w.cpu.x = u16::from((w.cpu.x as u8).wrapping_sub(1));
                        w.flags(w.cpu.x, 1);
                    }
                    28 => {
                        if w.branch(w.cpu.p & NEGATIVE == 0) {
                            self.phase = Helper(20);
                            w.cpu.pc = 0xc3a4;
                            return Some(w.cycles);
                        }
                    }
                    29 => {
                        w.rts();
                        self.phase = Caller(4);
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Helper(i + 1)
            }
            Palette(i) => {
                match i {
                    0 => w.width(0x20, true),
                    1..=16 => {
                        w.drain(24);
                        let offset = u32::from((i - 1) / 2) * 2;
                        if i % 2 == 1 {
                            let v = w.read(0x0cc425 + offset, 2);
                            w.load(v, 2);
                        } else {
                            w.write(0x7ec6a0 + offset, w.cpu.a, 2);
                        }
                    }
                    17 => w.width(0x30, false),
                    18 => w.dp_increment(0x15, 1, true),
                    19 => {
                        w.rts();
                        self.phase = Helper(2);
                        return Some(w.cycles);
                    }
                    _ => unreachable!(),
                }
                Palette(i + 1)
            }
            _ => unreachable!(),
        };
        self.phase = next;
        (w.cpu.bank, w.cpu.pc) = Self::location(next);
        Some(w.cycles)
    }
}
fn store_abs<B: WorkBus>(w: &mut Work<'_, B>, a: u16, width: u8) {
    w.drain(16);
    w.write(
        w.absolute(a),
        if width == 1 { w.cpu.a & 255 } else { w.cpu.a },
        width,
    );
}
fn call_short<B: WorkBus>(w: &mut Work<'_, B>, pc: u16) {
    w.drain(16);
    w.drain(6);
    w.push(w.cpu.pc + 2, 2);
    w.cpu.pc = pc;
}
fn call_long<B: WorkBus>(w: &mut Work<'_, B>, bank: u8, pc: u16) {
    w.drain(24);
    w.drain(6);
    w.push(u16::from(w.cpu.bank), 1);
    w.push(w.cpu.pc + 3, 2);
    w.cpu.bank = bank;
    w.cpu.pc = pc;
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn retained_poly_initializer_matches_live_stack_bus_and_registers() {
        let Ok(rom) = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        ) else {
            return;
        };
        for address in (0x09f810..=0x09f81c).chain([0x09f7f7, 0x09f809]) {
            assert_eq!(
                table_byte(address),
                Some(rom[((address >> 16) as usize) * 0x8000 + (address as usize & 0x7fff)])
            );
        }
        for theme in 0..3 {
            for (module, aux, sprites, _db, odd, v, h) in [
                (0, 0, 0, 12, false, 224, 1338),
                (1, 5, 11, 0, true, 261, 1350),
            ] {
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 7)) as u8;
                }
                machine.ram[0x10] = module;
                machine.ram[0xaa1..0xaa5].copy_from_slice(&[theme, aux, sprites, theme % 12]);
                machine.ram[0xc2fc..0xc300].copy_from_slice(&[12, 82, 90, 95]);
                machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                let mut work = IntroPolyInit::default();
                let mut native = NativeNmiContext {
                    a: 0xab44,
                    x: 0x78,
                    y: 0xff,
                    d: 0,
                    db: 0,
                    bank: 12,
                    pc: 0xc33c,
                    sp: 0x1fc,
                    p: 0xb0,
                };
                machine.cpu.a = native.a;
                machine.cpu.x = native.x;
                machine.cpu.y = native.y;
                machine.cpu.pc = native.pc;
                machine.cpu.k = native.bank;
                machine.cpu.db = 0;
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
                    let label=format!("tilesets theme={theme} module={module} aux={aux} sprites={sprites} phase={:?} pc={:04x}",work.phase,native.pc);
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
                    assert!(count < 1000000);
                }
                assert_eq!(
                    (native.bank, native.pc, native.sp, native.db),
                    (0, 0x805a, 0x1ff, 0)
                );
                assert_eq!(ram, source.snes().ram);
                assert_eq!(ppu.vram, source.snes().ppu.vram);
            }
        }
    }
}
