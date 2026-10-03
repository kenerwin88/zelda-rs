//! Scene actor initializers. Their retained CPU owns each indexed operand and
//! partial coordinate/velocity publication until the real RTS returns.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Initializer {
    Triangle,
    Copyright,
    Sparkle,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IntroSpriteInit {
    kind: Initializer,
    instruction: u8,
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    match address {
        0x0cc572..=0x0cc57d => {
            Some([218, 255, 95, 0, 230, 0, 200, 0, 189, 255, 200, 0][(address - 0x0cc572) as usize])
        }
        0x0cc5ca..=0x0cc5cf => Some([1, 0, 255, 255, 1, 255][(address - 0x0cc5ca) as usize]),
        0x0cc8fd..=0x0cc904 => {
            Some([194, 152, 111, 52, 124, 84, 124, 87][(address - 0x0cc8fd) as usize])
        }
        _ => None,
    }
}
impl IntroSpriteInit {
    pub(crate) fn new(kind: Initializer) -> Self {
        Self {
            kind,
            instruction: 0,
        }
    }
    fn addresses(&self) -> &'static [u16] {
        match self.kind {
            Initializer::Triangle => &[
                0xc57e, 0xc57f, 0xc580, 0xc581, 0xc584, 0xc587, 0xc58a, 0xc58d, 0xc590, 0xc593,
                0xc596, 0xc599, 0xc59c, 0xc59d, 0xc5a0, 0xc5a3, 0xc5a6, 0xc5a7, 0xc5aa, 0xc5ad,
                0xc5b0,
            ],
            Initializer::Copyright => &[
                0xc850, 0xc852, 0xc855, 0xc858, 0xc85a, 0xc85d, 0xc860, 0xc863,
            ],
            Initializer::Sparkle => &[
                0xc8e2, 0xc8e5, 0xc8e6, 0xc8e7, 0xc8e8, 0xc8e9, 0xc8ea, 0xc8ec, 0xc8ed, 0xc8f0,
                0xc8f3, 0xc8f6, 0xc8f9, 0xc8fc,
            ],
        }
    }
    pub(crate) fn complete(&self) -> bool {
        usize::from(self.instruction) == self.addresses().len()
    }
    pub(crate) fn step(
        &mut self,
        cpu: &mut NativeNmiContext,
        clock: &mut CpuMasterTimeline,
        bus: impl WorkBus,
    ) -> Option<u64> {
        if self.complete() {
            return None;
        }
        assert_eq!(
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 0x38),
            (12, self.addresses()[self.instruction as usize], 12, 0, 0x30)
        );
        assert!(
            cpu.x
                < if self.kind == Initializer::Triangle {
                    3
                } else {
                    8
                }
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let i = self.instruction;
        match self.kind {
            Initializer::Triangle => match i {
                0 => {
                    w.drain(6);
                    w.load(w.cpu.x, 1);
                }
                1 => w.shift_left(1),
                2 => to_y(&mut w),
                3 | 5 | 7 | 9 => {
                    let y = w.cpu.y;
                    let value = indexed_read(
                        &mut w,
                        match i {
                            3 => 0xc572,
                            5 => 0xc573,
                            7 => 0xc578,
                            9 => 0xc579,
                            _ => unreachable!(),
                        },
                        y,
                    );
                    w.load(value, 1);
                }
                4 | 6 | 8 | 10 | 14 | 18 => {
                    let value = w.cpu.a;
                    indexed_store(
                        &mut w,
                        match i {
                            4 => 0x1e30,
                            6 => 0x1e38,
                            8 => 0x1e48,
                            10 => 0x1e50,
                            14 => 0x1e58,
                            18 => 0x1e60,
                            _ => unreachable!(),
                        },
                        value,
                    );
                }
                11 | 15 => {
                    let x = w.cpu.x;
                    let value = indexed_read(&mut w, if i == 11 { 0xc5ca } else { 0xc5cd }, x);
                    w.load(value, 1);
                }
                12 | 16 => w.clear_carry(),
                13 | 17 => {
                    let x = w.cpu.x;
                    let value = indexed_read(&mut w, if i == 13 { 0x1e58 } else { 0x1e60 }, x);
                    w.adc_byte(value);
                }
                19 => increment_phase(&mut w),
                20 => w.rts(),
                _ => unreachable!(),
            },
            Initializer::Copyright => match i {
                0 | 3 => w.immediate(if i == 0 { 76 } else { 184 }, 1),
                1 | 4 => {
                    let value = w.cpu.a;
                    indexed_store(&mut w, if i == 1 { 0x1e30 } else { 0x1e48 }, value);
                }
                2 | 5 => indexed_store(&mut w, if i == 2 { 0x1e38 } else { 0x1e50 }, 0),
                6 => increment_phase(&mut w),
                7 => w.rts(),
                _ => unreachable!(),
            },
            Initializer::Sparkle => match i {
                0 => {
                    w.drain(16);
                    let value = w.read(w.absolute(0x1e0a), 1);
                    w.load(value, 1);
                }
                1..=5 => {
                    w.drain(6);
                    let a = w.cpu.a & 255;
                    w.cpu.p = (w.cpu.p & !CARRY) | ((a & 1) as u8);
                    w.load(a >> 1, 1);
                }
                6 => w.mask(3, 1),
                7 => to_y(&mut w),
                8 | 10 => {
                    let y = w.cpu.y;
                    let value = indexed_read(&mut w, if i == 8 { 0xc8fd } else { 0xc901 }, y);
                    w.load(value, 1);
                }
                9 | 11 => {
                    let value = w.cpu.a;
                    indexed_store(&mut w, if i == 9 { 0x1e30 } else { 0x1e48 }, value);
                }
                12 => increment_phase(&mut w),
                13 => w.rts(),
                _ => unreachable!(),
            },
        }
        self.instruction += 1;
        if !self.complete() {
            w.cpu.pc = self.addresses()[self.instruction as usize];
        }
        Some(w.cycles)
    }
}
fn to_y<B: WorkBus>(w: &mut Work<'_, B>) {
    w.drain(6);
    w.cpu.y = w.cpu.a & 255;
    w.flags(w.cpu.y, 1);
}
fn indexed_read<B: WorkBus>(w: &mut Work<'_, B>, base: u16, index: u16) -> u16 {
    w.drain(16);
    if (base & 255) + index > 255 {
        w.drain(6);
    }
    w.read(w.absolute(base.wrapping_add(index)), 1)
}
fn indexed_store<B: WorkBus>(w: &mut Work<'_, B>, base: u16, value: u16) {
    w.drain(22);
    w.write(w.absolute(base + w.cpu.x), value & 255, 1);
}
fn increment_phase<B: WorkBus>(w: &mut Work<'_, B>) {
    w.drain(22);
    let address = w.absolute(0x1e10 + w.cpu.x);
    let value = w.read(address, 1).wrapping_add(1) & 255;
    w.drain(6);
    w.flags(value, 1);
    w.write(address, value, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn retained_intro_sprite_initializers_match_rom_bus_registers_and_clock() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        for range in [
            0x0cc572..=0x0cc57d,
            0x0cc5ca..=0x0cc5cf,
            0x0cc8fd..=0x0cc904,
        ] {
            for address in range {
                assert_eq!(
                    table_byte(address),
                    Some(rom[0x60000 + (address as usize & 0x7fff)])
                );
            }
        }
        let mut instructions = 0;
        let mut cases = 0;
        for (kind, slots) in [
            (Initializer::Triangle, 3),
            (Initializer::Copyright, 8),
            (Initializer::Sparkle, 8),
        ] {
            for slot in 0..slots {
                for seed in 0..=255u8 {
                    compare_case(&rom, kind, slot, seed, &mut instructions);
                    cases += 1;
                }
            }
        }
        eprintln!("{cases} initializer cases, {instructions} independently checked instruction boundaries");
    }

    fn compare_case(rom: &[u8], kind: Initializer, slot: u16, seed: u8, instructions: &mut usize) {
        let mut work = IntroSpriteInit::new(kind);
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(seed),
            x: slot,
            y: 0x93,
            d: 0,
            db: 12,
            bank: 12,
            pc: work.addresses()[0],
            sp: 0x1fa,
            p: 0x30 | (seed & 0xc3),
        };
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
        }
        machine.ram[0x1e0a] = seed;
        machine.ram[0x1e10 + usize::from(slot)] = seed;
        machine.ram[0x1e58 + usize::from(slot)] = seed;
        machine.ram[0x1e60 + usize::from(slot)] = seed ^ 0x80;
        machine.ram[0x1fb..0x1fd].copy_from_slice(&0xc420u16.to_le_bytes());
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
        let mut source =
            RomCpuTimingProbe::new(machine, clock.clone(), SourcePpuReadState::snes9x_reset())
                .unwrap();
        clock.begin_synchronous_timeline().unwrap();
        while !work.complete() {
            let label = format!(
                "kind={kind:?} slot={slot} seed={seed} instruction={}",
                work.instruction
            );
            super::super::native_work::compare_instruction(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| {
                    work.step(cpu, clock, bus).unwrap();
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            work = work.clone();
            *instructions += 1;
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (12, 0xc421, 0x1fc));
        assert!(work
            .step(&mut cpu, &mut clock, |_, _| panic!(
                "completed initializer touched bus"
            ))
            .is_none());
    }
}
