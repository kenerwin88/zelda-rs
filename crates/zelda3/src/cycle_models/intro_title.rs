//! Title fade, sword descent, input and exit callers. Child work returns through the live
//! stack, including palette passes and thread teardown interrupted by NMI.
use super::{
    intro_animation::IntroAnimation,
    intro_clear::Access,
    intro_palette_fade::IntroPaletteFade,
    intro_sword::IntroSword,
    native_work::Work,
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Child {
    Animation(IntroAnimation),
    Palette(IntroPaletteFade),
    Sword(IntroSword),
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };
    #[test]
    fn title_background_wait_and_exit_match_source_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for entry in [0xc25c, 0xc2ae, 0xc284, 0xc2d4, 0xc2f0] {
            for seed in 0..16u8 {
                let mut work = IntroTitle::new(entry);
                let mut cpu = NativeNmiContext {
                    a: 0xab00 | u16::from(seed),
                    x: 0x45,
                    y: 0x93,
                    d: 0,
                    db: 0,
                    bank: 12,
                    pc: entry,
                    sp: 0x1fc,
                    p: 0x30 | (seed & 0xc3),
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
                }
                machine.ram[0x1e00] = 5;
                machine.ram[0x1e10..0x1e18].fill(0);
                machine.ram[0x1f00] = seed & 1;
                machine.ram[0x1a] = seed;
                machine.ram[0x10] = 0;
                machine.ram[0x11] = 7;
                machine.ram[0xb0] = [0, 1, 2, 255][usize::from(seed) % 4];
                machine.ram[0xc007] = [0, 1, 14, 31][usize::from(seed) % 4];
                machine.ram[0xf4] = if seed & 4 != 0 { 0x10 } else { 0 };
                machine.ram[0xf6] = if seed & 8 != 0 { 0x80 } else { 0 };
                machine.ram[0xc8..0xca].copy_from_slice(&30u16.to_le_bytes());
                machine.ram[0xca] = seed % 4;
                machine.ram[0xcb] = seed % 7;
                machine.ram[0xcc] = (seed % 3) * 2;
                machine.ram[0xcd] = seed * 4;
                machine.ram[0xd0] = seed % 3;
                machine.ram[0xff9] = seed % 5;
                machine.ram[0x1fd..0x200].copy_from_slice(&[0x59, 0x80, 0]);
                machine.cpu.a = cpu.a;
                machine.cpu.x = cpu.x;
                machine.cpu.y = cpu.y;
                machine.cpu.dp = cpu.d;
                machine.cpu.db = cpu.db;
                machine.cpu.k = cpu.bank;
                machine.cpu.pc = cpu.pc;
                machine.cpu.sp = cpu.sp;
                machine.cpu.e = false;
                machine.cpu.unpack_flags(cpu.p);
                let mut ram = machine.ram.clone();
                let (v, h) = match seed % 3 {
                    0 => (100, 510),
                    1 => (224, 1340),
                    _ => (261, 1320),
                };
                let mut clock = CpuMasterTimeline::at_raster(
                    0,
                    CpuRasterPosition::new(v, h),
                    CpuBusWorkload::default(),
                    CpuFieldTiming::NON_INTERLACE_EVEN,
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
                    let label = format!("entry={entry:04x} seed={seed} pc={:06x}", work.pc);
                    super::super::native_work::compare_instruction(
                        &label,
                        &mut cpu,
                        &mut clock,
                        &mut source,
                        &mut ram,
                        &rom,
                        |cpu, clock, bus| {
                            work.step(cpu, clock, bus).unwrap();
                        },
                    );
                    assert_eq!(ram, source.snes().ram, "{label}");
                    work = work.clone();
                    count += 1;
                    instructions += 1;
                    assert!(count < 10000, "{label}");
                }
                assert_eq!((cpu.bank, cpu.pc, cpu.sp), (0, 0x805a, 0x1ff));
            }
        }
        eprintln!(
            "80 title parent cases, {instructions} independently checked instruction boundaries"
        );
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroTitle {
    pc: u32,
    child: Option<Child>,
    complete: bool,
    tail_palette: bool,
    entry: u32,
}
impl IntroTitle {
    pub(crate) fn new(entry: u16) -> Self {
        assert!(matches!(entry, 0xc25c | 0xc2ae | 0xc284 | 0xc2d4 | 0xc2f0));
        Self {
            pc: 0x0c0000 | u32::from(entry),
            child: None,
            complete: false,
            tail_palette: false,
            entry: 0x0c0000 | u32::from(entry),
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn routine(&self) -> u32 {
        match self.child {
            Some(Child::Animation(_)) => 0x0cc404,
            Some(Child::Palette(_)) => {
                if self.tail_palette {
                    0xed8f
                } else {
                    0xed7c
                }
            }
            Some(Child::Sword(_)) => 0x0cfe56,
            _ => self.entry,
        }
    }
    pub(crate) fn restoring_palette(&self) -> bool {
        matches!(self.child, Some(Child::Palette(_)))
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl FnMut(u64, Access) -> u16,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((u32::from(cpu.bank) << 16) | u32::from(cpu.pc), self.pc);
        if let Some(child) = self.child.as_mut() {
            let (cycles, complete) = match child {
                Child::Animation(w) => (w.step(cpu, clock, bus), w.complete()),
                Child::Palette(w) => (w.step(cpu, clock, bus), w.complete()),
                Child::Sword(w) => (w.step(cpu, clock, bus), w.complete()),
            };
            if complete {
                self.child = None;
                // JML transfers the parent's original long return frame to
                // Palette_FadeIntro2, whose RTL completes this caller too.
                if self.tail_palette {
                    self.complete = true;
                }
            }
            self.pc = (u32::from(cpu.bank) << 16) | u32::from(cpu.pc);
            return cycles;
        }
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match self.pc {
            0x0cc25c | 0x0cc2ae | 0x0cc265 | 0x0cc287 | 0x0cc2d4 => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                if self.pc == 0x0cc265 {
                    self.child = Some(Child::Palette(IntroPaletteFade::title()));
                    0xed7c
                } else {
                    self.child = Some(Child::Animation(Default::default()));
                    0x0cc404
                }
            }
            0x0cc260 | 0x0cc291 => {
                let value = w.dp_read(0x1a, 1);
                w.load(value, 1);
                self.pc + 2
            }
            0x0cc262 | 0x0cc293 => {
                w.drain(6);
                let a = w.cpu.a & 255;
                w.cpu.p = (w.cpu.p & !CARRY) | (a & 1) as u8;
                w.load(a >> 1, 1);
                self.pc + 1
            }
            0x0cc263 | 0x0cc294 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    if self.pc == 0x0cc263 {
                        0x0cc278
                    } else {
                        0x0cc2ad
                    }
                } else {
                    self.pc + 2
                }
            }
            0x0cc269 | 0x0cc28b => {
                w.drain(24);
                let value = w.read(0x7ec007, 1);
                w.load(value, 1);
                self.pc + 4
            }
            0x0cc26d | 0x0cc27b | 0x0cc2bd | 0x0cc2a9 | 0x0cc2e3 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    match self.pc {
                        0x0cc26d => 0x0cc279,
                        0x0cc27b => 0x0cc283,
                        0x0cc2a9 => 0x0cc2ad,
                        0x0cc2e3 => 0x0cc2ef,
                        _ => 0x0cc2d3,
                    }
                } else {
                    self.pc + 2
                }
            }
            0x0cc26f | 0x0cc27d | 0x0cc2c1 | 0x0cc2c5 | 0x0cc2c9 | 0x0cc2cf | 0x0cc2e7
            | 0x0cc2f0 | 0x0cc2f5 | 0x0cc2fd | 0x0cc332 | 0x0ed60b | 0x0ed60f | 0x0ed613 => {
                w.immediate(
                    match self.pc {
                        0x0cc26f => 42,
                        0x0cc27d => 0x15,
                        0x0cc2c5 => 0x22,
                        0x0cc2c9 => 31,
                        0x0cc2e7 => 20,
                        0x0cc2f0 => 0xff,
                        0x0cc2f5 => 0x15,
                        0x0cc2fd => 0xf1,
                        0x0cc332 => 1,
                        0x0ed60b => 0x20,
                        0x0ed60f => 0x40,
                        0x0ed613 => 0x80,
                        _ => 2,
                    },
                    1,
                );
                self.pc + 2
            }
            0x0cc271 | 0x0cc27f | 0x0cc281 | 0x0cc2c3 | 0x0cc2c7 | 0x0cc2d1 | 0x0cc2e9
            | 0x0cc2eb | 0x0cc2ed | 0x0cc2f7 | 0x0cc2f9 | 0x0cc2fb | 0x0cc334 | 0x0cc339
            | 0x0ed60d | 0x0ed611 | 0x0ed615 => {
                w.dp_store(
                    match self.pc {
                        0x0cc271 => 0xb0,
                        0x0cc27f => 0x1c,
                        0x0cc281 | 0x0cc2d1 => 0x1d,
                        0x0cc2c3 => 0x99,
                        0x0cc2e9 | 0x0cc334 => 0x10,
                        0x0cc2eb | 0x0cc339 => 0x11,
                        0x0cc2ed => 0x22,
                        0x0cc2f7 => 0x1c,
                        0x0cc2f9 => 0x1d,
                        0x0cc2fb => 0x1b,
                        0x0ed60d => 0x9c,
                        0x0ed611 => 0x9d,
                        0x0ed615 => 0x9e,
                        _ => 0x9a,
                    },
                    if matches!(
                        self.pc,
                        0x0cc281 | 0x0cc2eb | 0x0cc2ed | 0x0cc2f9 | 0x0cc2fb | 0x0cc339
                    ) {
                        0
                    } else {
                        w.cpu.a & 255
                    },
                    1,
                );
                self.pc + 2
            }
            0x0cc273 | 0x0cc2bf | 0x0cc2ab | 0x0cc2e5 => {
                w.dp_increment(0x11, 1, true);
                self.pc + 2
            }
            0x0cc275 | 0x0cc2b8 | 0x0cc284 | 0x0cc2de => {
                w.drain(22);
                w.push(w.cpu.pc + 2, 2);
                let setup = self.pc == 0x0cc275;
                self.child = Some(Child::Sword(if setup {
                    IntroSword::setup()
                } else {
                    IntroSword::periodic()
                }));
                if setup {
                    0x0cfe45
                } else {
                    0x0cfe56
                }
            }
            0x0cc278 | 0x0cc283 | 0x0cc2d3 | 0x0cc2ad | 0x0cc2ef | 0x0cc33b => {
                w.rtl();
                self.complete = true;
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x0cc279 => {
                w.compare_immediate(13, 1);
                self.pc + 2
            }
            0x0cc2b2 | 0x0cc2b5 | 0x0cc2d8 | 0x0cc2db => {
                w.drain(16);
                w.write(
                    w.absolute(if matches!(self.pc, 0x0cc2b2 | 0x0cc2d8) {
                        0x1f00
                    } else {
                        0x12a
                    }),
                    0,
                    1,
                );
                self.pc + 3
            }
            0x0cc2bb | 0x0cc2a7 | 0x0cc2e1 => {
                w.dp_increment(0xb0, 1, false);
                self.pc + 2
            }
            0x0cc2cb => {
                w.drain(24);
                w.write(0x7ec007, w.cpu.a & 255, 1);
                self.pc + 4
            }
            0x0cc28f | 0x0cc2a2 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    if self.pc == 0x0cc28f {
                        0x0cc29a
                    } else {
                        0x0cc2a7
                    }
                } else {
                    self.pc + 2
                }
            }
            0x0cc296 => {
                w.drain(24);
                self.child = Some(Child::Palette(IntroPaletteFade::scene()));
                self.tail_palette = true;
                0xed8f
            }
            0x0cc29a => {
                let value = w.dp_read(0xf6, 1);
                w.load(value, 1);
                self.pc + 2
            }
            0x0cc29c | 0x0cc2a0 => {
                w.mask(if self.pc == 0x0cc29c { 0xc0 } else { 0xd0 }, 1);
                self.pc + 2
            }
            0x0cc29e => {
                let value = w.dp_read(0xf4, 1);
                w.load((w.cpu.a | value) & 255, 1);
                self.pc + 2
            }
            0x0cc2a4 => {
                w.drain(16);
                self.entry = 0x0cc2f0;
                0x0cc2f0
            }
            0x0cc2f2 | 0x0cc2ff | 0x0cc336 => {
                w.drain(16);
                w.write(
                    w.absolute(match self.pc {
                        0x0cc2f2 => 0x128,
                        0x0cc2ff => 0x12c,
                        _ => 0x4aa,
                    }),
                    w.cpu.a & 255,
                    1,
                );
                self.pc + 3
            }
            0x0cc302 => {
                w.drain(30);
                w.push(u16::from(w.cpu.bank), 1);
                w.push(w.cpu.pc + 3, 2);
                0x0ed60b
            }
            0x0ed617 => {
                w.rtl();
                (u32::from(w.cpu.bank) << 16) | u32::from(w.cpu.pc)
            }
            0x0cc306 | 0x0cc330 => {
                w.width(0x30, self.pc == 0x0cc306);
                self.pc + 2
            }
            0x0cc308 | 0x0cc311 => {
                w.drain(16);
                w.cpu.x = if self.pc == 0x0cc308 { 0x6e } else { 0 };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0x0cc30b => {
                w.drain(14);
                w.write(u32::from(0x20 + w.cpu.x), 0, 2);
                self.pc + 2
            }
            0x0cc30d | 0x0cc30e => {
                w.decrement_x();
                self.pc + 1
            }
            0x0cc30f => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    0x0cc30b
                } else {
                    self.pc + 2
                }
            }
            0x0cc314 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
                self.pc + 1
            }
            0x0cc315 | 0x0cc319 | 0x0cc31d | 0x0cc321 | 0x0cc325 => {
                w.drain(24);
                let base = 0x7ef000 + ((self.pc - 0x0cc315) / 4) * 0x100;
                w.write(base + u32::from(w.cpu.x), w.cpu.a, 2);
                self.pc + 4
            }
            0x0cc329 | 0x0cc32a => {
                w.increment_x();
                self.pc + 1
            }
            0x0cc32b => {
                w.compare_x_immediate(0x100, 2);
                self.pc + 3
            }
            0x0cc32e => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0x0cc315
                } else {
                    self.pc + 2
                }
            }
            _ => unreachable!("unowned title continuation {:06x}", self.pc),
        };
        self.pc = next;
        w.cpu.bank = (next >> 16) as u8;
        w.cpu.pc = next as u16;
        Some(w.cycles)
    }
}
