//! Retained intro palette passes and their shared additive restoration loop.
//! Each color is published at its store; an interrupt retains the live CPU,
//! scratch components, palette index and real caller stack.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroPaletteFade {
    pc: u16,
    complete: bool,
}
impl IntroPaletteFade {
    pub(crate) fn title() -> Self {
        Self {
            pc: 0xed7c,
            complete: false,
        }
    }
    pub(crate) fn scene() -> Self {
        Self {
            pc: 0xed8f,
            complete: false,
        }
    }
    pub(crate) fn complete(&self) -> bool {
        self.complete
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete {
            return None;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.d), (0, self.pc, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let next = match self.pc {
            0xed7c | 0xed8f => {
                w.width(0x30, true);
                self.pc + 2
            }
            0xed7e | 0xed87 | 0xed91 | 0xed9a => {
                w.drain(16);
                w.cpu.x = match self.pc {
                    0xed7e => 0x100,
                    0xed87 => 0xc0,
                    _ => 0x40,
                };
                w.flags(w.cpu.x, 2);
                self.pc + 3
            }
            0xed81 | 0xed8a | 0xed94 | 0xed9d => {
                w.immediate(
                    match self.pc {
                        0xed81 => 0x1a0,
                        0xed8a => 0x100,
                        _ => 0xc0,
                    },
                    2,
                );
                self.pc + 3
            }
            0xed84 | 0xed97 | 0xeda0 => {
                w.drain(22);
                w.push(self.pc + 2, 2);
                0xedca
            }
            0xed8d => {
                w.branch(true);
                0xeda0
            }
            0xeda3 => {
                w.width(0x30, false);
                0xeda5
            }
            0xeda5 => {
                w.drain(24);
                let value = w.read(0x7ec007, 1);
                w.load(value, 1);
                0xeda9
            }
            0xeda9 => {
                w.drain(6);
                w.load(w.cpu.a.wrapping_sub(1) & 255, 1);
                0xedaa
            }
            0xedaa => {
                w.drain(24);
                w.write(0x7ec007, w.cpu.a & 255, 1);
                0xedae
            }
            0xedae => {
                w.dp_increment(0x15, 1, true);
                0xedb0
            }
            0xedb0 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            0xedca | 0xedd4 | 0xedda | 0xede0 => {
                let address = match self.pc {
                    0xedca => 0x0e,
                    0xedd4 => 8,
                    0xedda => 10,
                    _ => 12,
                };
                w.dp_store(address, w.cpu.a, 2);
                self.pc + 2
            }
            0xedcc | 0xede2 | 0xedf3 | 0xee04 => {
                w.drain(24);
                let base = if self.pc == 0xedcc {
                    0x7ec500
                } else {
                    0x7ec300
                };
                let value = w.read(base + u32::from(w.cpu.x), 2);
                w.load(value, 2);
                self.pc + 4
            }
            0xedd0 | 0xedf2 | 0xee03 | 0xee14 => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                self.pc + 1
            }
            0xedd6 | 0xeddc | 0xeded | 0xedfe | 0xee0f | 0xee15 => {
                w.drain(6);
                w.load(w.cpu.y, 2);
                self.pc + 1
            }
            0xedd1 | 0xede6 | 0xedd7 | 0xedf7 | 0xeddd | 0xee08 => {
                w.mask(
                    match self.pc {
                        0xedd1 | 0xede6 => 0x1f,
                        0xedd7 | 0xedf7 => 0x3e0,
                        _ => 0x7c00,
                    },
                    2,
                );
                self.pc + 3
            }
            0xede9 | 0xedfa | 0xee0b | 0xee1c => {
                let value = w.dp_read(
                    match self.pc {
                        0xede9 => 8,
                        0xedfa => 10,
                        0xee0b => 12,
                        _ => 14,
                    },
                    2,
                );
                let left = if self.pc == 0xee1c { w.cpu.x } else { w.cpu.a };
                w.cpu.p = (w.cpu.p & !CARRY) | u8::from(left >= value);
                w.flags(left.wrapping_sub(value), 2);
                self.pc + 2
            }
            0xedeb | 0xedfc | 0xee0d => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    self.pc + 8
                } else {
                    self.pc + 2
                }
            }
            0xedee | 0xedff | 0xee10 => {
                w.clear_carry();
                self.pc + 1
            }
            0xedef | 0xee00 | 0xee11 => {
                w.drain(16);
                w.adc(match self.pc {
                    0xedef => 1,
                    0xee00 => 0x20,
                    _ => 0x400,
                });
                self.pc + 3
            }
            0xee16 => {
                w.drain(24);
                w.write(0x7ec500 + u32::from(w.cpu.x), w.cpu.a, 2);
                0xee1a
            }
            0xee1a | 0xee1b => {
                w.increment_x();
                self.pc + 1
            }
            0xee1e => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    0xedcc
                } else {
                    0xee20
                }
            }
            0xee20 => {
                w.rts();
                w.cpu.pc
            }
            _ => unreachable!("unowned intro palette continuation {:04x}", self.pc),
        };
        self.pc = next;
        w.cpu.pc = next;
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
    fn intro_palette_passes_match_source_at_every_retained_boundary() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        for title in [false, true] {
            for seed in 0..16u8 {
                let mut work = if title {
                    IntroPaletteFade::title()
                } else {
                    IntroPaletteFade::scene()
                };
                let mut cpu = NativeNmiContext {
                    a: 0xab10,
                    x: 0xfc,
                    y: 0xff,
                    d: 0,
                    db: seed,
                    bank: 0,
                    pc: work.pc,
                    sp: 0x1fa,
                    p: 0x30 | (seed & 3),
                };
                let mut machine = Snes::new();
                machine.cart.load(CartType::LoRom, &rom, 0x2000);
                for (i, b) in machine.ram.iter_mut().enumerate() {
                    *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
                }
                // Equal components, all eight combinations of differing components,
                // and word carries including bit 15 and wraparound.
                for i in 0..256 {
                    let main: u16 = [0, 0x1f, 0x3e0, 0x7c00, 0x7fff, 0x8000, 0xffff, 0x4321]
                        [(i + usize::from(seed)) % 8];
                    let mask: u16 = (if i & 1 != 0 { 1 } else { 0 })
                        | (if i & 2 != 0 { 0x20 } else { 0 })
                        | (if i & 4 != 0 { 0x400 } else { 0 });
                    machine.ram[0xc500 + i * 2..0xc502 + i * 2]
                        .copy_from_slice(&main.to_le_bytes());
                    machine.ram[0xc300 + i * 2..0xc302 + i * 2]
                        .copy_from_slice(&(main ^ mask).to_le_bytes());
                }
                machine.ram[0xc007] = [0, 1, 13, 31][usize::from(seed) % 4];
                machine.ram[0x1fb..0x1fe].copy_from_slice(&[0x68, 0xc2, 12]);
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
                    let label = format!("title={title} seed={seed} pc={:04x}", work.pc);
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
                assert_eq!(
                    (cpu.bank, cpu.pc, cpu.sp, cpu.db),
                    (12, 0xc269, 0x1fd, seed)
                );
            }
        }
        eprintln!("32 palette cases, {instructions} independently checked instruction boundaries");
    }
}
