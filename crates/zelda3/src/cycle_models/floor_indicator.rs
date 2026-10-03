//! Dungeon floor-indicator tail, including removal, sound selection, tile
//! publications and the caller's actual far return.
use super::{
    native_work::{Work, WorkBus},
    nmi_stack::{NativeNmiContext, CARRY, NEGATIVE, ZERO},
};
use snes::CpuMasterTimeline;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FloorIndicator {
    pc: u16,
    complete: bool,
}
impl Default for FloorIndicator {
    fn default() -> Self {
        Self {
            pc: 0xfd0c,
            complete: false,
        }
    }
}
pub(crate) fn table_byte(address: u32) -> Option<u8> {
    crate::ZeldaState::native_floor_indicator_table_byte(address)
}
impl FloorIndicator {
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
        assert_eq!((cpu.bank, cpu.pc, cpu.d, cpu.p & 8), (10, self.pc, 0, 0));
        let mut w = Work {
            cpu,
            clock,
            bus,
            cycles: 0,
        };
        let m = if w.cpu.p & 0x20 != 0 { 1 } else { 2 };
        let mask = if m == 1 { 255 } else { 65535 };
        let pc = self.pc;
        w.fetch();
        self.pc = match pc {
            0xfd0c | 0xfd4f | 0xfd5e | 0xfd67 | 0xfd6e | 0xfd8a | 0xfd90 | 0xfda5 => {
                w.width(
                    if matches!(pc, 0xfd0c | 0xfd8a | 0xfda5) {
                        0x30
                    } else {
                        0x20
                    },
                    matches!(pc, 0xfd0c | 0xfd5e | 0xfd6e | 0xfd90),
                );
                pc + 2
            }
            0xfd0e => {
                w.drain(16);
                let value = w.read(w.absolute(0x4a0), m);
                w.load(value, m);
                pc + 3
            }
            0xfd11 | 0xfd62 => {
                w.mask(255, m);
                pc + 3
            }
            0xfd14 | 0xfd1a | 0xfd42 | 0xfd46 | 0xfd4d | 0xfd57 | 0xfd65 => {
                let (take, target) = match pc {
                    0xfd14 => (w.cpu.p & ZERO != 0, 0xfd90),
                    0xfd1a => (w.cpu.p & ZERO == 0, 0xfd1f),
                    0xfd42 => (w.cpu.p & NEGATIVE != 0, 0xfd67),
                    0xfd46 => (w.cpu.p & ZERO == 0, 0xfd60),
                    0xfd4d => (w.cpu.p & ZERO != 0, 0xfd60),
                    0xfd57 => (w.cpu.p & CARRY != 0, 0xfd5e),
                    _ => (true, 0xfd7a),
                };
                if w.branch(take) {
                    target
                } else {
                    pc + 2
                }
            }
            0xfd16 | 0xfd2c | 0xfd31 => {
                w.increment_a(m);
                pc + 1
            }
            0xfd17 | 0xfd4a | 0xfd55 => {
                w.compare_immediate(if pc == 0xfd17 { 0xc0 } else { 2 }, m);
                pc + 1 + u16::from(m)
            }
            0xfd1c | 0xfd25 | 0xfd36 | 0xfd59 | 0xfd69 | 0xfd92 => {
                w.drain(8 * u32::from(m));
                w.load(
                    match pc {
                        0xfd1c => 0,
                        0xfd25 => 0x251e,
                        0xfd36 => 0x250f,
                        0xfd59 => 3,
                        0xfd69 => 5,
                        _ => 0x7f,
                    },
                    m,
                );
                pc + 1 + u16::from(m)
            }
            0xfd1f | 0xfd5b | 0xfd6b => {
                w.drain(16);
                w.write(
                    w.absolute(if pc == 0xfd1f { 0x4a0 } else { 0x12d }),
                    w.cpu.a & mask,
                    m,
                );
                pc + 3
            }
            0xfd22 | 0xfd23 => {
                w.drain(6);
                w.push(
                    u16::from(if pc == 0xfd22 { w.cpu.db } else { w.cpu.bank }),
                    1,
                );
                pc + 1
            }
            0xfd24 | 0xfd8c => {
                w.drain(12);
                w.cpu.db = w.pull(1) as u8;
                w.flags(u16::from(w.cpu.db), 1);
                pc + 1
            }
            0xfd28 | 0xfd2d | 0xfd32 | 0xfd39 | 0xfd7f | 0xfd86 | 0xfd95 | 0xfd99 | 0xfd9d
            | 0xfda1 => {
                let base = match pc {
                    0xfd28 | 0xfd7f | 0xfd95 => 0x7ec7f2,
                    0xfd2d | 0xfda1 => 0x7ec834,
                    0xfd32 | 0xfd86 | 0xfd99 => 0x7ec832,
                    _ => 0x7ec7f4,
                };
                w.drain(24);
                w.write(
                    base + if matches!(pc, 0xfd7f | 0xfd86) {
                        u32::from(w.cpu.x)
                    } else {
                        0
                    },
                    w.cpu.a & mask,
                    m,
                );
                pc + 4
            }
            0xfd3d => {
                w.drain(16);
                w.cpu.x = 0;
                w.flags(0, 2);
                pc + 3
            }
            0xfd40 | 0xfd44 | 0xfd48 | 0xfd60 | 0xfd72 => {
                let value = w.dp_read(
                    match pc {
                        0xfd40 => 0xa3,
                        0xfd48 => 0xa0,
                        _ => 0xa4,
                    },
                    m,
                );
                w.load(value, m);
                pc + 2
            }
            0xfd51 => {
                w.drain(24);
                let value = w.read(0x7ef3c5, m);
                w.load(value, m);
                pc + 4
            }
            0xfd70 | 0xfd71 => {
                w.drain(6);
                w.cpu.x = w.cpu.x.wrapping_add(1);
                w.flags(w.cpu.x, 2);
                pc + 1
            }
            0xfd74 | 0xfd77 => {
                w.drain(16);
                w.load(
                    if pc == 0xfd74 {
                        w.cpu.a | 0xff00
                    } else {
                        w.cpu.a ^ 0xffff
                    },
                    m,
                );
                pc + 3
            }
            0xfd7a => {
                w.shift_left(m);
                pc + 1
            }
            0xfd7b => {
                w.drain(6);
                w.cpu.y = w.cpu.a;
                w.flags(w.cpu.y, 2);
                pc + 1
            }
            0xfd7c | 0xfd83 => {
                w.operand_then_internal(2);
                let value = w.read(
                    w.absolute(if pc == 0xfd7c { 0xfce0 } else { 0xfcf6 }) + u32::from(w.cpu.y),
                    m,
                );
                w.load(value, m);
                pc + 3
            }
            0xfd8d => {
                w.dp_increment(0x16, m, true);
                pc + 2
            }
            0xfd8f | 0xfda7 => {
                w.rtl();
                self.complete = true;
                w.cpu.pc
            }
            _ => unreachable!("floor indicator {pc:04x}"),
        };
        w.cpu.pc = self.pc;
        Some(w.cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{intro_clear::Access, native_work::compare_instruction};
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn floor_indicator_matches_source_timer_floor_tiles_sound_and_far_return() {
        let rom = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../saves/zelda3.sfc"),
        )
        .unwrap();
        let mut visited = BTreeSet::new();
        let mut instructions = 0;
        for case in 0..1408usize {
            let mut machine = snes::Snes::new();
            machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
            for (i, b) in machine.ram.iter_mut().enumerate() {
                *b = (i as u8).wrapping_mul(17).wrapping_add(case as u8);
            }
            let r = &mut machine.ram;
            r[0x4a0] = [0, 1, 0xbe, 0xbf, 0xc0, 0xfe, 0xff, 0x40][case % 8];
            r[0x4a1] = 0xaa;
            let floor = case / 8 % 22;
            r[0xa4] = if floor < 11 {
                floor as u8
            } else {
                255 - (floor - 11) as u8
            };
            r[0xa5] = 0;
            r[0xa3] = 0;
            r[0xa0] = if case / 176 % 2 == 0 { 2 } else { 3 };
            r[0xa1] = 0;
            r[0xf3c5] = [0, 1, 2, 3][case / 352 % 4];
            r[0x1fd..0x200].copy_from_slice(&[0x34, 0x81, 0]);
            let mut cpu = NativeNmiContext {
                a: 0xabcd,
                x: 13,
                y: 7,
                d: 0,
                db: 0,
                bank: 10,
                pc: 0xfd0c,
                sp: 0x1fc,
                p: 0x30,
            };
            machine.cpu.a = cpu.a;
            machine.cpu.x = cpu.x;
            machine.cpu.y = cpu.y;
            machine.cpu.k = cpu.bank;
            machine.cpu.pc = cpu.pc;
            machine.cpu.sp = cpu.sp;
            machine.cpu.e = false;
            machine.cpu.unpack_flags(cpu.p);
            let mut ram = machine.ram.clone();
            let timeline = CpuMasterTimeline::at_raster(
                800 + case as u64,
                snes::CpuRasterPosition::new(261, 1300),
                snes::CpuBusWorkload::default(),
                snes::CpuFieldTiming::NON_INTERLACE_EVEN,
            );
            let mut source = snes::RomCpuTimingProbe::new(
                machine,
                timeline,
                snes::SourcePpuReadState::snes9x_reset(),
            )
            .unwrap();
            let mut clock = source.timeline().clone();
            let mut work = FloorIndicator::default();
            for _ in 0..100 {
                if work.complete() {
                    break;
                }
                visited.insert(cpu.pc);
                compare_instruction(
                    &format!("floor indicator case={case} pc={:04x}", cpu.pc),
                    &mut cpu,
                    &mut clock,
                    &mut source,
                    &mut ram,
                    &rom,
                    |cpu, clock, bus| {
                        work.step(cpu, clock, |at, access| {
                            if let Access::Read(a, width) = access {
                                if a & 0x408000 == 0x8000 {
                                    for b in 0..width {
                                        let a = a + u32::from(b);
                                        assert_eq!(
                                            table_byte(a),
                                            Some(
                                                rom[(a >> 16) as usize * 0x8000
                                                    + (a as usize & 0x7fff)]
                                            ),
                                            "floor table {a:06x}"
                                        );
                                    }
                                }
                            }
                            bus(at, access)
                        })
                        .unwrap();
                    },
                );
                instructions += 1;
            }
            assert!(work.complete());
            assert_eq!(ram, source.snes().ram);
            assert_eq!((cpu.bank, cpu.pc, cpu.sp, cpu.db), (0, 0x8135, 0x1ff, 0));
        }
        for pc in [0xfd1c, 0xfd59, 0xfd69, 0xfd7f, 0xfd8d, 0xfd8f, 0xfda7] {
            assert!(visited.contains(&pc), "uncovered floor path {pc:04x}");
        }
        eprintln!(
            "1408 floor-indicator fixtures, {instructions} instruction boundaries, {} distinct PCs",
            visited.len()
        );
    }
}
