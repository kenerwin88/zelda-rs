//! Retained scene-sprite drawing and fixed-point movement. The CPU, stack,
//! live coordinates and shared OAM cursor remain owned across each instruction.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

const MOVE_OFFSETS: [u16; 25] = [
    0, 3, 5, 6, 7, 8, 9, 10, 13, 16, 19, 20, 21, 22, 23, 24, 26, 27, 29, 31, 32, 35, 38, 39, 42,
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MoveTriangle {
    axis: u8,
    instruction: u8,
}
impl MoveTriangle {
    pub(crate) fn complete(&self) -> bool {
        self.axis == 3
    }
    fn pc(&self) -> u16 {
        if self.axis == 2 {
            0xca4b
        } else {
            0xc9f1 + u16::from(self.axis) * 45 + MOVE_OFFSETS[self.instruction as usize]
        }
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
            (12, self.pc(), 12, 0, 0x30)
        );
        assert!(cpu.x < 8);
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        if self.axis == 2 {
            w.rts();
            self.axis = 3;
            return Some(w.cycles);
        }
        let velocity = 0x1e58 + u16::from(self.axis) * 8;
        let position = 0x1e28 + u16::from(self.axis) * 24;
        let mut next = self.instruction + 1;
        match self.instruction {
            0 | 9 => {
                let value = indexed_read(&mut w, velocity, 1);
                w.load(value, 1);
            }
            1 => {
                if w.branch(w.cpu.p & ZERO != 0) {
                    next = 25;
                }
            }
            2..=5 => w.shift_left(1),
            6 => w.clear_carry(),
            7 | 20 | 23 => {
                let value = indexed_read(
                    &mut w,
                    position
                        + match self.instruction {
                            7 => 0,
                            20 => 8,
                            _ => 16,
                        },
                    1,
                );
                w.adc_byte(value);
            }
            8 | 21 | 24 => {
                let value = w.cpu.a;
                indexed_store(
                    &mut w,
                    position
                        + match self.instruction {
                            8 => 0,
                            21 => 8,
                            _ => 16,
                        },
                    value,
                    1,
                );
            }
            10 => {
                w.drain(6);
                w.push(u16::from(w.cpu.p), 1);
            }
            11..=14 => shift_right(&mut w, 1),
            15 => {
                w.drain(8);
                w.cpu.y = 0;
                w.flags(0, 1);
            }
            16 => {
                w.drain(12);
                w.cpu.p = w.pull(1) as u8;
            }
            17 => {
                if w.branch(w.cpu.p & NEGATIVE == 0) {
                    next = 20;
                }
            }
            18 => {
                w.drain(8);
                w.load((w.cpu.a | 0xf0) & 255, 1);
            }
            19 => {
                w.drain(6);
                w.cpu.y = w.cpu.y.wrapping_sub(1) & 255;
                w.flags(w.cpu.y, 1);
            }
            22 => {
                w.drain(6);
                w.load(w.cpu.y, 1);
            }
            _ => unreachable!(),
        }
        self.instruction = next;
        if next == 25 {
            self.axis += 1;
            self.instruction = 0;
        }
        w.cpu.pc = self.pc();
        Some(w.cycles)
    }
}

const OAM_ADDRESSES: [u16; 68] = [
    0xc972, 0xc975, 0xc977, 0xc97a, 0xc97c, 0xc97f, 0xc981, 0xc984, 0xc986, 0xc988, 0xc98a, 0xc98b,
    0xc98d, 0xc990, 0xc993, 0xc995, 0xc996, 0xc997, 0xc99a, 0xc99d, 0xc99f, 0xc9a0, 0xc9a2, 0xc9a5,
    0xc9a8, 0xc9aa, 0xc9ab, 0xc9ac, 0xc9ae, 0xc9af, 0xc9b1, 0xc9b4, 0xc9b5, 0xc9b8, 0xc9bb, 0xc9bd,
    0xc9c0, 0xc9c3, 0xc9c4, 0xc9c5, 0xc9c7, 0xc9c9, 0xc9cc, 0xc9cd, 0xc9ce, 0xc9cf, 0xc9d2, 0xc9d3,
    0xc9d4, 0xc9d5, 0xc9d7, 0xc9d8, 0xc9d9, 0xc9da, 0xc9dc, 0xc9de, 0xc9e1, 0xc9e2, 0xc9e4, 0xc9e5,
    0xc9e6, 0xc9e7, 0xc9e8, 0xc9e9, 0xc9eb, 0xc9ed, 0xc9ef, 0xc9f0,
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AddObjectsToOam {
    instruction: u8,
}
impl AddObjectsToOam {
    pub(crate) fn complete(&self) -> bool {
        self.instruction as usize == OAM_ADDRESSES.len()
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
            (cpu.bank, cpu.pc, cpu.db, cpu.d, cpu.p & 8),
            (12, OAM_ADDRESSES[self.instruction as usize], 12, 0, 0)
        );
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        w.fetch();
        let mut next = self.instruction + 1;
        match self.instruction {
            0 | 2 | 4 | 6 => {
                let value = indexed_read(
                    &mut w,
                    [0x1e30, 0x1e38, 0x1e48, 0x1e50][self.instruction as usize / 2],
                    1,
                );
                w.load(value, 1);
            }
            1 | 3 | 5 | 7 => w.dp_store(u16::from(self.instruction) / 2, w.cpu.a & 255, 1),
            8 | 9 => w.dp_store(u16::from(self.instruction) - 4, 0, 1),
            10 | 42 => {
                w.drain(6);
                w.push(w.cpu.x, if self.instruction == 10 { 1 } else { 2 });
            }
            11 => w.width(0x30, true),
            12 => {
                w.drain(16);
                w.cpu.y = 0;
                w.flags(0, 2);
            }
            13 => {
                w.drain(16);
                w.cpu.x = w.read(w.absolute(0x1e08), 2);
                w.flags(w.cpu.x, 2);
            }
            14 => {
                let value = w.dp_read(6, 2);
                w.load(value, 2);
            }
            15 | 16 => w.shift_left(2),
            17 => {
                w.drain(16);
                let value = w.read(w.absolute(0x1e08), 2);
                w.adc(value);
            }
            18 => {
                w.drain(16);
                w.write(w.absolute(0x1e08), w.cpu.a, 2);
            }
            19 | 27 | 39 | 53 => {
                let width = if self.instruction == 53 { 1 } else { 2 };
                let value = table_read(&mut w, width);
                w.load(value, width);
            }
            20 | 28 | 31 => w.clear_carry(),
            21 | 29 => {
                let value = w.dp_read(if self.instruction == 21 { 0 } else { 2 }, 2);
                w.adc(value);
            }
            22 | 30 | 36 | 41 => {
                let value = w.cpu.a;
                indexed_store(
                    &mut w,
                    match self.instruction {
                        22 => 0,
                        30 | 36 => 1,
                        _ => 2,
                    },
                    value,
                    2,
                );
            }
            23 => w.mask(0x100, 2),
            24 => w.dp_store(0xc, w.cpu.a, 2),
            25 | 26 | 37 | 38 | 50 | 51 | 52 | 58 => w.increment_y(),
            32 => {
                w.drain(16);
                w.adc(0x10);
            }
            33 => w.compare_immediate(0x100, 2),
            34 => {
                if w.branch(w.cpu.p & CARRY == 0) {
                    next = 37;
                }
            }
            35 => w.immediate(0xf0, 2),
            40 => {
                let value = w.dp_read(4, 2);
                w.load(w.cpu.a ^ value, 2);
            }
            43 => {
                w.drain(6);
                w.load(w.cpu.x, 2);
            }
            44 => {
                w.drain(6);
                w.cpu.p |= CARRY;
            }
            45 => {
                w.drain(16);
                w.sbc(0x800, 2);
            }
            46 | 47 => shift_right(&mut w, 2),
            48 => w.tax_word(),
            49 => w.width(0x20, false),
            54 => {
                let value = w.dp_read(0xd, 1);
                w.load((w.cpu.a | value) & 255, 1);
            }
            55 => {
                let value = w.cpu.a;
                indexed_store(&mut w, 0xa20, value, 1);
            }
            56 | 66 => {
                let width = if self.instruction == 56 { 2 } else { 1 };
                w.drain(12);
                w.cpu.x = w.pull(width);
                w.flags(w.cpu.x, width);
            }
            57 => w.width(0x20, true),
            59..=62 => w.increment_x(),
            63 => w.dp_increment(6, 2, false),
            64 => {
                if w.branch(w.cpu.p & ZERO == 0) {
                    next = 19;
                }
            }
            65 => w.width(0x30, false),
            67 => w.rts(),
            _ => unreachable!(),
        }
        self.instruction = next;
        if !self.complete() {
            w.cpu.pc = OAM_ADDRESSES[next as usize];
        }
        Some(w.cycles)
    }
}
fn indexed_read<B: WorkBus>(w: &mut Work<'_, B>, base: u16, width: u8) -> u16 {
    w.drain(16);
    if w.cpu.p & 0x10 == 0 || (base & 255) + w.cpu.x > 255 {
        w.drain(6);
    }
    w.read(w.absolute(base.wrapping_add(w.cpu.x)), width)
}
fn indexed_store<B: WorkBus>(
    w: &mut Work<'_, B>,
    base: u16,
    value: u16,
    width: u8,
) {
    w.drain(22);
    w.write(
        w.absolute(base.wrapping_add(w.cpu.x)),
        if width == 1 { value & 255 } else { value },
        width,
    );
}
fn table_read<B: WorkBus>(w: &mut Work<'_, B>, width: u8) -> u16 {
    w.drain(8);
    let pointer = w.read(8, 2);
    if w.cpu.p & 0x10 == 0 || (pointer & 255) + w.cpu.y > 255 {
        w.drain(6);
    }
    w.read(w.absolute(pointer).wrapping_add(u32::from(w.cpu.y)), width)
}
fn shift_right<B: WorkBus>(w: &mut Work<'_, B>, width: u8) {
    w.drain(6);
    let value = w.cpu.a & if width == 1 { 255 } else { 65535 };
    w.cpu.p = (w.cpu.p & !CARRY) | (value & 1) as u8;
    w.load(value >> 1, width);
}

#[cfg(test)]
mod tests {
    use super::*;
    use snes::{
        CartType, CpuBusWorkload, CpuFieldTiming, CpuRasterPosition, RomCpuTimingProbe, Snes,
        SourcePpuReadState,
    };

    #[test]
    fn retained_intro_sprite_movement_and_oam_match_source_instructions() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut instructions = 0;
        let mut cases = 0;
        for slot in 0..8 {
            for seed in 0..=255u8 {
                for fraction in [0, 255] {
                    compare_case(&rom, slot, seed, Some(fraction), 0, 0, &mut instructions);
                    cases += 1;
                }
            }
            for seed in 0..=31u8 {
                for (count, pointer) in [
                    (16, 0xc60f),
                    (16, 0xc68f),
                    (13, 0xc868),
                    (1, 0xc936),
                    (128, 0x1000),
                    (2, 0xcfff),
                ] {
                    compare_case(&rom, slot, seed, None, count, pointer, &mut instructions);
                    cases += 1;
                }
            }
        }
        eprintln!("{cases} sprite work cases, {instructions} independently checked instruction boundaries");
    }
    fn compare_case(
        rom: &[u8],
        slot: u16,
        seed: u8,
        fraction: Option<u8>,
        count: u16,
        pointer: u16,
        instructions: &mut usize,
    ) {
        let moving = fraction.is_some();
        let mut cpu = NativeNmiContext {
            a: 0xab00 | u16::from(seed),
            x: slot,
            y: 0x93,
            d: 0,
            db: 12,
            bank: 12,
            pc: if moving { 0xc9f1 } else { 0xc972 },
            sp: 0x1fa,
            p: 0x30 | (seed & 0xc3),
        };
        let mut machine = Snes::new();
        machine.cart.load(CartType::LoRom, rom, 0x2000);
        for (i, b) in machine.ram.iter_mut().enumerate() {
            *b = (i ^ (i >> 8) ^ usize::from(seed)) as u8;
        }
        let positions = [
            0, 1, 0xff, 0x100, 0x7fff, 0x8000, 0xffff, 0xfff0, 0xef, 0xf0, 0xf1, 0xffef,
        ];
        let x = positions[usize::from(seed) % positions.len()];
        let y = positions[(usize::from(seed) + 3) % positions.len()];
        for (address, value) in [
            (0x1e30, x as u8),
            (0x1e38, (x >> 8) as u8),
            (0x1e48, y as u8),
            (0x1e50, (y >> 8) as u8),
            (0x1e28, fraction.unwrap_or(seed)),
            (0x1e40, fraction.unwrap_or(seed)),
            (0x1e58, seed),
            (0x1e60, seed ^ 0x80),
        ] {
            machine.ram[address + usize::from(slot)] = value;
        }
        machine.ram[0x1e08..0x1e0a].copy_from_slice(
            &(if count == 128 {
                0x800u16
            } else {
                0x800 + u16::from(seed) * 4
            })
            .to_le_bytes(),
        );
        machine.ram[6..8].copy_from_slice(&count.to_le_bytes());
        machine.ram[8..10].copy_from_slice(&pointer.to_le_bytes());
        machine.ram[0x1fb..0x1fd].copy_from_slice(&0xc5b6u16.to_le_bytes());
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
        let mut movement = MoveTriangle::default();
        let mut drawing = AddObjectsToOam::default();
        let mut n = 0;
        while if moving {
            !movement.complete()
        } else {
            !drawing.complete()
        } {
            let label=format!("moving={moving} slot={slot} seed={seed} fraction={fraction:?} count={count} pointer={pointer:04x} pc={:04x}",cpu.pc);
            super::super::native_work::compare_instruction(
                &label,
                &mut cpu,
                &mut clock,
                &mut source,
                &mut ram,
                rom,
                |cpu, clock, bus| {
                    if moving {
                        movement.step(cpu, clock, bus).unwrap();
                    } else {
                        drawing.step(cpu, clock, bus).unwrap();
                    }
                },
            );
            assert_eq!(ram, source.snes().ram, "{label}");
            movement = movement.clone();
            drawing = drawing.clone();
            n += 1;
            *instructions += 1;
            assert!(n < 10000, "{label}");
        }
        assert_eq!((cpu.bank, cpu.pc, cpu.sp), (12, 0xc5b7, 0x1fc));
        assert!(if moving {
            movement
                .step(&mut cpu, &mut clock, |_, _| {
                    panic!("completed movement touched bus")
                })
                .is_none()
        } else {
            drawing
                .step(&mut cpu, &mut clock, |_, _| {
                    panic!("completed drawing touched bus")
                })
                .is_none()
        });
    }
}
